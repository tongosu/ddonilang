use crate::core::fixed64::Fixed64;
use crate::core::state::Key;
use crate::core::trace::Trace;
use crate::core::unit::{eval_unit_expr, format_dim, UnitDim};
use crate::core::value::{LambdaValue, ListValue, MapEntry, MapValue, PackValue, Quantity, SetValue, TemplateValue, Value};
use crate::core::State;
use crate::lang::ast::{BinaryOp, Binding, ContractKind, ContractMode, DeclKind, Expr, FormulaDialect, HookKind, Literal, Path, Program, SeedKind, Stmt, UnaryOp};
use crate::runtime::detmath;
use crate::runtime::error::RuntimeError;
use crate::runtime::open::OpenRuntime;
use crate::runtime::formula::{analyze_formula, eval_formula_body, format_formula_body, FormulaError};
use crate::runtime::template::{match_template, render_template};
use ddonirang_core::ResourceHandle;
use blake3;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Evaluator {
    state: State,
    trace: Trace,
    bogae_requested: bool,
    bogae_requested_tick: bool,
    aborted: bool,
    contract_diags: Vec<ContractDiag>,
    rng_state: Cell<u64>,
    current_madi: Cell<u64>,
    open_seq: Cell<u64>,
    open_source: String,
    open: OpenRuntime,
    user_seeds: BTreeMap<String, UserSeed>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FlowControl {
    Continue,
    Break(crate::lang::span::Span),
    Return(Value, crate::lang::span::Span),
}

static LAMBDA_SEQ: AtomicU64 = AtomicU64::new(1);

fn next_lambda_id() -> u64 {
    LAMBDA_SEQ.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone)]
enum Callable {
    Name(String),
    Lambda(LambdaValue),
}

#[derive(Clone)]
struct UserSeed {
    kind: SeedKind,
    params: Vec<String>,
    body: Vec<Stmt>,
    span: crate::lang::span::Span,
}

fn ensure_no_break(flow: FlowControl) -> Result<(), RuntimeError> {
    match flow {
        FlowControl::Continue => Ok(()),
        FlowControl::Break(span) => Err(RuntimeError::BreakOutsideLoop { span }),
        FlowControl::Return(_, span) => Err(RuntimeError::ReturnOutsideSeed { span }),
    }
}

impl Evaluator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_state(State::new())
    }

    pub fn with_state(state: State) -> Self {
        Self::with_state_and_seed(state, 0)
    }

    pub fn with_state_and_seed(state: State, seed: u64) -> Self {
        Self::with_state_seed_open(state, seed, OpenRuntime::deny(), "<memory>".to_string())
    }

    pub fn with_state_seed_open(
        state: State,
        seed: u64,
        open: OpenRuntime,
        open_source: String,
    ) -> Self {
        Self {
            state,
            trace: Trace::new(),
            bogae_requested: false,
            bogae_requested_tick: false,
            aborted: false,
            contract_diags: Vec::new(),
            rng_state: Cell::new(seed),
            current_madi: Cell::new(0),
            open_seq: Cell::new(0),
            open_source,
            open,
            user_seeds: BTreeMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn run(self, program: &Program) -> Result<EvalOutput, RuntimeError> {
        self.run_with_ticks(program, 1)
    }

    pub fn run_with_ticks(self, program: &Program, ticks: u64) -> Result<EvalOutput, RuntimeError> {
        self.run_with_ticks_observe(program, ticks, |_, _, _| {})
    }

    pub fn run_with_ticks_observe<F>(
        self,
        program: &Program,
        ticks: u64,
        on_tick: F,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
    {
        self.run_with_ticks_internal(program, ticks, no_op_before_tick, no_op_should_stop, on_tick)
    }

    pub fn run_with_ticks_observe_and_inject<F, G>(
        self,
        program: &Program,
        ticks: u64,
        before_tick: G,
        on_tick: F,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State) -> Result<(), RuntimeError>,
    {
        self.run_with_ticks_internal(program, ticks, before_tick, no_op_should_stop, on_tick)
    }

    pub fn run_with_ticks_observe_and_inject_stop<F, G, H>(
        self,
        program: &Program,
        ticks: u64,
        before_tick: G,
        on_tick: F,
        should_stop: H,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State) -> Result<(), RuntimeError>,
        H: FnMut(u64, &State) -> bool,
    {
        self.run_with_ticks_internal(program, ticks, before_tick, should_stop, on_tick)
    }

    fn run_with_ticks_internal<F, G, H>(
        mut self,
        program: &Program,
        ticks: u64,
        mut before_tick: G,
        mut should_stop: H,
        mut on_tick: F,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State) -> Result<(), RuntimeError>,
        H: FnMut(u64, &State) -> bool,
    {
        let mut init_stmts = Vec::new();
        let mut start_hooks: Vec<&Vec<Stmt>> = Vec::new();
        let mut every_hooks: Vec<&Vec<Stmt>> = Vec::new();
        let mut seed_defs: Vec<&Stmt> = Vec::new();

        for stmt in &program.stmts {
            match stmt {
                Stmt::SeedDef { .. } => seed_defs.push(stmt),
                Stmt::Hook { kind, body, .. } => match kind {
                    HookKind::Start => start_hooks.push(body),
                    HookKind::EveryMadi => every_hooks.push(body),
                },
                other => init_stmts.push(other),
            }
        }

        for stmt in seed_defs {
            let Stmt::SeedDef { name, params, kind, body, span } = stmt else {
                continue;
            };
            self.user_seeds.insert(
                name.clone(),
                UserSeed {
                    kind: *kind,
                    params: params.clone(),
                    body: body.clone(),
                    span: *span,
                },
            );
        }

        for stmt in init_stmts {
            ensure_no_break(self.eval_stmt(stmt)?)?;
        }

        for hook in start_hooks {
            ensure_no_break(self.eval_block(hook)?)?;
        }

        for madi in 0..ticks {
            self.current_madi.set(madi);
            if should_stop(madi, &self.state) {
                break;
            }
            before_tick(madi, &mut self.state)?;
            if should_stop(madi, &self.state) {
                break;
            }
            for hook in &every_hooks {
                ensure_no_break(self.eval_block(hook)?)?;
            }
            if let Some(seed) = self.user_seeds.get("매틱").cloned() {
                if matches!(seed.kind, SeedKind::Umjikssi) {
                    let _ = self.eval_user_seed(&seed, &[], seed.span)?;
                }
            }
            let tick_requested = self.bogae_requested_tick;
            on_tick(madi, &self.state, tick_requested);
            self.bogae_requested_tick = false;
        }

        Ok(EvalOutput {
            state: self.state,
            trace: self.trace,
            bogae_requested: self.bogae_requested,
            contract_diags: self.contract_diags,
        })
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<FlowControl, RuntimeError> {
        if self.aborted {
            return Ok(FlowControl::Continue);
        }
        match stmt {
            Stmt::DeclBlock { kind, items, .. } => {
                for item in items {
                    let value = if let Some(expr) = &item.value {
                        self.eval_expr(expr)?
                    } else if matches!(kind, DeclKind::Butbak) {
                        return Err(RuntimeError::Pack {
                            message: format!("붙박이마련에는 초기값이 필요합니다: {}", item.name),
                            span: item.span,
                        });
                    } else {
                        Value::None
                    };
                    self.state.set(Key::new(item.name.clone()), value);
                }
                Ok(FlowControl::Continue)
            }
            Stmt::SeedDef {
                name,
                params,
                kind,
                body,
                span,
            } => {
                self.user_seeds.insert(
                    name.clone(),
                    UserSeed {
                        kind: *kind,
                        params: params.clone(),
                        body: body.clone(),
                        span: *span,
                    },
                );
                Ok(FlowControl::Continue)
            }
            Stmt::Assign { target, value, .. } => {
                if let Some(root) = target.segments.first() {
                    if root == "샘" {
                        return Err(RuntimeError::InvalidPath {
                            path: target.segments.join("."),
                            span: target.span,
                        });
                    }
                    if matches!(root.as_str(), "살림" | "바탕")
                        && target.segments.get(1).map(|seg| seg.as_str()) == Some("입력상태")
                    {
                        return Err(RuntimeError::InvalidPath {
                            path: target.segments.join("."),
                            span: target.span,
                        });
                    }
                }
                let val = self.eval_expr(value)?;
                let key = self.path_to_key(target)?;
                self.state.set(key, val);
                Ok(FlowControl::Continue)
            }
            Stmt::Show { value, .. } => {
                let val = self.eval_expr(value)?;
                self.trace.log(val.display());
                Ok(FlowControl::Continue)
            }
            Stmt::Expr { value, .. } => {
                let _ = self.eval_expr(value)?;
                Ok(FlowControl::Continue)
            }
            Stmt::Return { value, span } => {
                let val = self.eval_expr(value)?;
                Ok(FlowControl::Return(val, *span))
            }
            Stmt::BogaeDraw { .. } => {
                self.bogae_requested = true;
                self.bogae_requested_tick = true;
                Ok(FlowControl::Continue)
            }
            Stmt::Hook { .. } => Ok(FlowControl::Continue),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let value = self.eval_expr(condition)?;
                if is_truthy(&value, condition.span())? {
                    return self.eval_block(then_body);
                }
                if let Some(body) = else_body {
                    return self.eval_block(body);
                }
                Ok(FlowControl::Continue)
            }
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    let value = self.eval_expr(&branch.condition)?;
                    if is_truthy(&value, branch.condition.span())? {
                        return self.eval_block(&branch.body);
                    }
                }
                self.eval_block(else_body)
            }
            Stmt::Repeat { body, .. } => {
                loop {
                    if self.aborted {
                        break;
                    }
                    match self.eval_block(body)? {
                        FlowControl::Continue => {}
                        FlowControl::Break(_) => break,
                        FlowControl::Return(value, span) => {
                            return Ok(FlowControl::Return(value, span));
                        }
                    }
                }
                Ok(FlowControl::Continue)
            }
            Stmt::While {
                condition,
                body,
                ..
            } => {
                loop {
                    if self.aborted {
                        break;
                    }
                    let value = self.eval_expr(condition)?;
                    if !is_truthy(&value, condition.span())? {
                        break;
                    }
                    match self.eval_block(body)? {
                        FlowControl::Continue => {}
                        FlowControl::Break(_) => break,
                        FlowControl::Return(value, span) => {
                            return Ok(FlowControl::Return(value, span));
                        }
                    }
                }
                Ok(FlowControl::Continue)
            }
            Stmt::ForEach {
                item,
                iterable,
                body,
                ..
            } => {
                let iter_value = self.eval_expr(iterable)?;
                let items = match iter_value {
                    Value::List(list) => list.items,
                    Value::Set(set) => set.items.into_values().collect(),
                    Value::Map(map) => map
                        .entries
                        .into_values()
                        .map(|entry| {
                            Value::List(ListValue {
                                items: vec![entry.key, entry.value],
                            })
                        })
                        .collect(),
                    _ => {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "iterable list/set/map",
                            span: iterable.span(),
                        })
                    }
                };
                if item == "입력상태" {
                    return Err(RuntimeError::InvalidPath {
                        path: format!("살림.{}", item),
                        span: iterable.span(),
                    });
                }
                let key = Key::new(item.clone());
                let prev = self.state.get(&key).cloned();
                for value in items {
                    if self.aborted {
                        break;
                    }
                    self.state.set(key.clone(), value);
                    match self.eval_block(body)? {
                        FlowControl::Continue => {}
                        FlowControl::Break(_) => break,
                        FlowControl::Return(value, span) => {
                            if let Some(prev_value) = prev.clone() {
                                self.state.set(key, prev_value);
                            } else {
                                self.state.resources.remove(&key);
                            }
                            return Ok(FlowControl::Return(value, span));
                        }
                    }
                }
                if let Some(prev) = prev {
                    self.state.set(key, prev);
                } else {
                    self.state.resources.remove(&key);
                }
                Ok(FlowControl::Continue)
            }
            Stmt::Break { span } => Ok(FlowControl::Break(*span)),
            Stmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
                ..
            } => {
                let value = self.eval_expr(condition)?;
                let mut ok = is_truthy(&value, condition.span())?;
                match kind {
                    ContractKind::Pre => {
                        if ok {
                            if let Some(body) = then_body {
                                match self.eval_block(body)? {
                                    FlowControl::Continue => {}
                                    FlowControl::Break(span) => return Ok(FlowControl::Break(span)),
                                    FlowControl::Return(value, span) => {
                                        return Ok(FlowControl::Return(value, span));
                                    }
                                }
                            }
                        } else {
                            match self.eval_block(else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Break(span) => return Ok(FlowControl::Break(span)),
                                FlowControl::Return(value, span) => {
                                    return Ok(FlowControl::Return(value, span));
                                }
                            }
                            self.emit_contract_violation(*kind, *mode, condition.span());
                            if matches!(mode, ContractMode::Abort) {
                                self.aborted = true;
                            }
                        }
                    }
                    ContractKind::Post => {
                        if !ok {
                            match self.eval_block(else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Break(span) => return Ok(FlowControl::Break(span)),
                                FlowControl::Return(value, span) => {
                                    return Ok(FlowControl::Return(value, span));
                                }
                            }
                            let value = self.eval_expr(condition)?;
                            ok = is_truthy(&value, condition.span())?;
                            if !ok {
                                self.emit_contract_violation(*kind, *mode, condition.span());
                                if matches!(mode, ContractMode::Abort) {
                                    self.aborted = true;
                                }
                                return Ok(FlowControl::Continue);
                            }
                        }
                        if let Some(body) = then_body {
                            match self.eval_block(body)? {
                                FlowControl::Continue => {}
                                FlowControl::Break(span) => return Ok(FlowControl::Break(span)),
                                FlowControl::Return(value, span) => {
                                    return Ok(FlowControl::Return(value, span));
                                }
                            }
                        }
                    }
                }
                Ok(FlowControl::Continue)
            }
        }
    }

    fn eval_block(&mut self, stmts: &[Stmt]) -> Result<FlowControl, RuntimeError> {
        if self.aborted {
            return Ok(FlowControl::Continue);
        }
        for stmt in stmts {
            if self.aborted {
                return Ok(FlowControl::Continue);
            }
            match self.eval_stmt(stmt)? {
                FlowControl::Continue => {}
                FlowControl::Break(span) => return Ok(FlowControl::Break(span)),
                FlowControl::Return(value, span) => {
                    return Ok(FlowControl::Return(value, span));
                }
            }
        }
        Ok(FlowControl::Continue)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit, span) => self.literal_to_value(lit, *span),
            Expr::Path(path) => self.eval_path(path),
            Expr::FieldAccess { target, field, span } => {
                let base = self.eval_expr(target)?;
                let Value::Pack(pack) = base else {
                    return Err(RuntimeError::Pack {
                        message: "묶음 필드 접근은 묶음에서만 가능합니다".to_string(),
                        span: *span,
                    });
                };
                let Some(value) = pack.fields.get(field) else {
                    return Err(RuntimeError::Pack {
                        message: format!("필드 '{}'가 없습니다", field),
                        span: *span,
                    });
                };
                if matches!(value, Value::None) {
                    return Err(RuntimeError::Pack {
                        message: format!("필드 '{}' 값이 없음입니다", field),
                        span: *span,
                    });
                }
                Ok(value.clone())
            }
            Expr::Atom { text, .. } => Ok(Value::Str(text.clone())),
            Expr::Unary { op, expr, span } => {
                let value = self.eval_expr(expr)?;
                self.eval_unary(op, value, *span)
            }
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary(op, left_val, right_val, *span)
            }
            Expr::SeedLiteral { param, body, .. } => Ok(Value::Lambda(LambdaValue {
                id: next_lambda_id(),
                param: param.clone(),
                body: (*body.clone()),
            })),
            Expr::Call { name, args, span } => self.eval_call(name, args, *span),
            Expr::Formula { dialect, body, span } => self.eval_formula_value(*dialect, body, *span),
            Expr::FormulaEval {
                dialect,
                body,
                bindings,
                span,
            } => self.eval_formula_eval(*dialect, body, bindings, *span),
            Expr::Template { body, span: _ } => Ok(Value::Template(TemplateValue { body: body.clone() })),
            Expr::TemplateFill {
                template,
                bindings,
                span,
            } => self.eval_template_fill(template, bindings, *span),
            Expr::Pack { bindings, span } => self.eval_pack(bindings, *span),
            Expr::FormulaFill {
                formula,
                bindings,
                span,
            } => self.eval_formula_fill(formula, bindings, *span),
        }
    }

    fn path_to_key(&self, path: &Path) -> Result<Key, RuntimeError> {
        if path.segments.len() < 2 {
            return Err(RuntimeError::InvalidPath {
                path: path.segments.join("."),
                span: path.span,
            });
        }
        let root = path.segments[0].as_str();
        if root == "살림" || root == "바탕" {
            return Ok(Key::new(path.segments[1..].join(".")));
        }
        if root == "샘" {
            return Ok(Key::new(format!("샘.{}", path.segments[1..].join("."))));
        }
        Err(RuntimeError::InvalidPath {
            path: path.segments.join("."),
            span: path.span,
        })
    }

    fn eval_path(&self, path: &Path) -> Result<Value, RuntimeError> {
        let key = self.path_to_key(path)?;
        if let Some(value) = self.state.get(&key) {
            return Ok(value.clone());
        }
        if path.segments.len() == 2 {
            let root = path.segments[0].as_str();
            if (root == "살림" || root == "바탕") && path.segments[1] == "입력키" {
                return Ok(Value::Str(String::new()));
            }
        }
        if path.segments.len() < 3 {
            return Err(RuntimeError::Undefined {
                path: path.segments.join("."),
                span: path.span,
            });
        }
        let root = path.segments[0].as_str();
        let base_key = match root {
            "살림" | "바탕" => Key::new(path.segments[1].clone()),
            "샘" => Key::new(format!("샘.{}", path.segments[1])),
            _ => {
                return Err(RuntimeError::InvalidPath {
                    path: path.segments.join("."),
                    span: path.span,
                })
            }
        };
        let mut current = self.state.get(&base_key).cloned().ok_or_else(|| RuntimeError::Undefined {
            path: path.segments.join("."),
            span: path.span,
        })?;
        for field in path.segments.iter().skip(2) {
            let Value::Pack(pack) = current else {
                return Err(RuntimeError::Pack {
                    message: "묶음 필드 접근은 묶음에서만 가능합니다".to_string(),
                    span: path.span,
                });
            };
            let Some(value) = pack.fields.get(field) else {
                return Err(RuntimeError::Pack {
                    message: format!("필드 '{}'가 없습니다", field),
                    span: path.span,
                });
            };
            if matches!(value, Value::None) {
                return Err(RuntimeError::Pack {
                    message: format!("필드 '{}' 값이 없음입니다", field),
                    span: path.span,
                });
            }
            current = value.clone();
        }
        Ok(current)
    }

    fn literal_to_value(
        &self,
        lit: &Literal,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        match lit {
            Literal::None => Ok(Value::None),
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Num(n) => {
                let base = Fixed64::from_raw(n.raw);
                if let Some(unit_expr) = &n.unit {
                    let (dim, scale) = eval_unit_expr(unit_expr).map_err(|err| {
                        let unit = match err {
                            crate::core::unit::UnitError::Unknown(name) => name,
                            crate::core::unit::UnitError::Overflow => "overflow".to_string(),
                        };
                        RuntimeError::UnitUnknown { unit, span }
                    })?;
                    let value = scale.apply(base);
                    Ok(Value::Num(Quantity::new(value, dim)))
                } else {
                    Ok(Value::Num(Quantity::new(base, UnitDim::zero())))
                }
            }
            Literal::Str(s) => Ok(Value::Str(s.clone())),
        }
    }

    fn eval_unary(&self, op: &UnaryOp, value: Value, span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
        match op {
            UnaryOp::Neg => match value {
                Value::Num(qty) => Ok(Value::Num(Quantity::new(
                    Fixed64::from_raw(qty.raw.raw().saturating_neg()),
                    qty.dim,
                ))),
                _ => Err(RuntimeError::TypeMismatch {
                    expected: "number",
                    span,
                }),
            },
            UnaryOp::Not => {
                let truthy = is_truthy(&value, span)?;
                Ok(Value::Bool(!truthy))
            }
        }
    }

    fn eval_binary(
        &self,
        op: &BinaryOp,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        match op {
            BinaryOp::Add => self.eval_add(left, right, span),
            BinaryOp::Sub => self.eval_sub(left, right, span),
            BinaryOp::Mul => self.eval_mul(left, right, span),
            BinaryOp::Div => self.eval_div(left, right, span),
            BinaryOp::And => {
                let left_ok = is_truthy(&left, span)?;
                let right_ok = is_truthy(&right, span)?;
                Ok(Value::Bool(left_ok && right_ok))
            }
            BinaryOp::Or => {
                let left_ok = is_truthy(&left, span)?;
                let right_ok = is_truthy(&right, span)?;
                Ok(Value::Bool(left_ok || right_ok))
            }
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Lte
            | BinaryOp::Gt
            | BinaryOp::Gte => self.eval_compare(op, left, right, span),
        }
    }

    fn eval_call(&mut self, name: &str, args: &[Expr], span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
        let mut values = Vec::new();
        for arg in args {
            values.push(self.eval_expr(arg)?);
        }
        self.eval_call_values(name, &values, span)
    }

    fn eval_call_values(
        &mut self,
        name: &str,
        values: &[Value],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let name = Self::canonicalize_stdlib_alias(name);
        match name {
            "열림.시각.지금" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                let site_id = self.open_site_id("clock");
                self.open.open_clock(&site_id, span)
            }
            "열림.파일.읽기" => {
                let path = expect_open_path(values, span)?;
                let site_id = self.open_site_id("file_read");
                self.open.open_file_read(&site_id, &path, span)
            }
            "열림.난수.뽑기" | "열림.난수.하나" | "열림.난수" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                let site_id = self.open_site_id("rand");
                self.open.open_rand(&site_id, span)
            }
            "열림.네트워크.요청" => {
                let args = expect_open_net_args(values, span)?;
                let site_id = self.open_site_id("net");
                self.open
                    .open_net(&site_id, &args.url, &args.method, args.body.as_deref(), args.response.as_deref(), span)
            }
            "열림.호스트FFI.호출" => {
                let args = expect_open_ffi_args(values, span)?;
                let site_id = self.open_site_id("ffi");
                self.open
                    .open_ffi(&site_id, &args.name, args.args.as_deref(), args.result.as_deref(), span)
            }
            "열림.GPU.실행" => {
                let args = expect_open_gpu_args(values, span)?;
                let site_id = self.open_site_id("gpu");
                self.open
                    .open_gpu(&site_id, &args.kernel, args.payload.as_deref(), args.result.as_deref(), span)
            }
            "sqrt" => {
                let qty = expect_quantity(&values, 1, span)?;
                if qty.raw.raw() < 0 {
                    return Err(RuntimeError::MathDomain {
                        message: "음수의 제곱근은 지원하지 않습니다",
                        span,
                    });
                }
                let raw = qty.raw.sqrt().ok_or(RuntimeError::MathDomain {
                    message: "제곱근 계산 실패",
                    span,
                })?;
                let dim = sqrt_dim(qty.dim, span)?;
                Ok(Value::Num(Quantity::new(raw, dim)))
            }
            "abs" => {
                let qty = expect_quantity(&values, 1, span)?;
                let raw = Fixed64::from_raw(qty.raw.raw().abs());
                Ok(Value::Num(Quantity::new(raw, qty.dim)))
            }
            "sin" => {
                let qty = expect_quantity(&values, 1, span)?;
                ensure_angle_dim(qty.dim, span)?;
                let raw = detmath::sin(qty.raw);
                Ok(Value::Num(Quantity::new(raw, UnitDim::zero())))
            }
            "cos" => {
                let qty = expect_quantity(&values, 1, span)?;
                ensure_angle_dim(qty.dim, span)?;
                let raw = detmath::cos(qty.raw);
                Ok(Value::Num(Quantity::new(raw, UnitDim::zero())))
            }
            "min" => {
                let (a, b) = expect_two_quantities(&values, span)?;
                ensure_same_dim(&a, &b, span)?;
                Ok(Value::Num(if a.raw <= b.raw { a } else { b }))
            }
            "max" => {
                let (a, b) = expect_two_quantities(&values, span)?;
                ensure_same_dim(&a, &b, span)?;
                Ok(Value::Num(if a.raw >= b.raw { a } else { b }))
            }
            "clamp" => {
                let (value, min, max) = expect_three_quantities(&values, span)?;
                ensure_same_dim(&value, &min, span)?;
                ensure_same_dim(&value, &max, span)?;
                let clamped = if value.raw < min.raw {
                    min
                } else if value.raw > max.raw {
                    max
                } else {
                    value
                };
                Ok(Value::Num(clamped))
            }
            "powi" => {
                let (base, exp) = expect_two_quantities(&values, span)?;
                if !exp.dim.is_dimensionless() {
                    return Err(RuntimeError::UnitMismatch { span });
                }
                let exp_raw = exp.raw.raw();
                if exp_raw & 0xFFFF_FFFF != 0 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "integer exponent",
                        span,
                    });
                }
                let exp_i32 = (exp_raw >> 32) as i32;
                let raw = base.raw.powi(exp_i32);
                let dim = base.dim.scale(exp_i32);
                Ok(Value::Num(Quantity::new(raw, dim)))
            }
            "바닥" => {
                let qty = expect_quantity(&values, 1, span)?;
                let raw = fixed64_floor(qty.raw);
                Ok(Value::Num(Quantity::new(raw, qty.dim)))
            }
            "천장" => {
                let qty = expect_quantity(&values, 1, span)?;
                let raw = fixed64_ceil(qty.raw);
                Ok(Value::Num(Quantity::new(raw, qty.dim)))
            }
            "반올림" => {
                let qty = expect_quantity(&values, 1, span)?;
                let raw = fixed64_round_even(qty.raw);
                Ok(Value::Num(Quantity::new(raw, qty.dim)))
            }
            "합계" => {
                let list = expect_list(&values, span)?;
                if list.items.is_empty() {
                    return Ok(Value::Num(Quantity::new(Fixed64::from_int(0), UnitDim::zero())));
                }
                let mut total = expect_quantity_value(&list.items[0], span)?;
                for item in list.items.iter().skip(1) {
                    let next = expect_quantity_value(item, span)?;
                    ensure_same_dim(&total, &next, span)?;
                    total.raw = total.raw.saturating_add(next.raw);
                }
                Ok(Value::Num(total))
            }
            "평균" => {
                let list = expect_list(&values, span)?;
                if list.items.is_empty() {
                    return Ok(Value::None);
                }
                let mut total = expect_quantity_value(&list.items[0], span)?;
                for item in list.items.iter().skip(1) {
                    let next = expect_quantity_value(item, span)?;
                    ensure_same_dim(&total, &next, span)?;
                    total.raw = total.raw.saturating_add(next.raw);
                }
                let count = Fixed64::from_int(list.items.len() as i64);
                let avg_raw = total
                    .raw
                    .checked_div(count)
                    .ok_or(RuntimeError::MathDivZero { span })?;
                Ok(Value::Num(Quantity::new(avg_raw, total.dim)))
            }
            "길이" => match values {
                [Value::Str(text)] => {
                    let len = text.chars().count() as i64;
                    Ok(Value::Num(Quantity::new(Fixed64::from_int(len), UnitDim::zero())))
                }
                [Value::List(list)] => {
                    let len = list.items.len() as i64;
                    Ok(Value::Num(Quantity::new(Fixed64::from_int(len), UnitDim::zero())))
                }
                [value] => Err(type_mismatch_detail("string or list", value, span)),
                _ => Err(RuntimeError::TypeMismatch {
                    expected: "string or list",
                    span,
                }),
            },
            "범위" => {
                let (start, end, step) = match values.len() {
                    2 => {
                        let (start, end) = expect_two_quantities(values, span)?;
                        ensure_same_dim(&start, &end, span)?;
                        let step = Quantity::new(Fixed64::from_int(1), start.dim);
                        (start, end, step)
                    }
                    3 => {
                        let (start, end, step) = expect_three_quantities(values, span)?;
                        ensure_same_dim(&start, &end, span)?;
                        ensure_same_dim(&start, &step, span)?;
                        (start, end, step)
                    }
                    _ => {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "range(start, end, step?)",
                            span,
                        })
                    }
                };
                if step.raw.raw() == 0 {
                    return Err(RuntimeError::MathDomain {
                        message: "범위 간격은 0이 될 수 없습니다",
                        span,
                    });
                }
                let mut items = Vec::new();
                let mut current = start.raw;
                let step_raw = step.raw;
                if step_raw.raw() > 0 {
                    while current.raw() <= end.raw.raw() {
                        items.push(Value::Num(Quantity::new(current, start.dim)));
                        current = current.saturating_add(step_raw);
                    }
                } else {
                    while current.raw() >= end.raw.raw() {
                        items.push(Value::Num(Quantity::new(current, start.dim)));
                        current = current.saturating_add(step_raw);
                    }
                }
                Ok(Value::List(ListValue { items }))
            }
            "표준.범위" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "range(start, end, inclusive)",
                        span,
                    });
                }
                let (start, end, include) = expect_three_quantities(values, span)?;
                ensure_same_dim(&start, &end, span)?;
                if include.dim != UnitDim::zero() {
                    return Err(RuntimeError::UnitMismatch { span });
                }
                let flag = include.raw.raw();
                if flag != 0 && flag != Fixed64::from_int(1).raw() {
                    return Err(RuntimeError::MathDomain {
                        message: "끝포함은 0 또는 1이어야 합니다",
                        span,
                    });
                }
                let include_end = flag != 0;
                let step = Quantity::new(Fixed64::from_int(1), start.dim);
                let mut items = Vec::new();
                let mut current = start.raw;
                if start.raw.raw() <= end.raw.raw() {
                    if include_end {
                        while current.raw() <= end.raw.raw() {
                            items.push(Value::Num(Quantity::new(current, start.dim)));
                            current = current.saturating_add(step.raw);
                        }
                    } else {
                        while current.raw() < end.raw.raw() {
                            items.push(Value::Num(Quantity::new(current, start.dim)));
                            current = current.saturating_add(step.raw);
                        }
                    }
                }
                Ok(Value::List(ListValue { items }))
            }
            "대문자로바꾸기" => {
                let text = expect_single_string(values, span)?;
                Ok(Value::Str(text.to_uppercase()))
            }
            "소문자로바꾸기" => {
                let text = expect_single_string(values, span)?;
                Ok(Value::Str(text.to_lowercase()))
            }
            "다듬기" => {
                let text = expect_single_string(values, span)?;
                Ok(Value::Str(text.trim().to_string()))
            }
            "되풀이하기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "string, count",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let count = require_nonnegative(expect_int(&values[1], span)?, span)?;
                Ok(Value::Str(text.repeat(count)))
            }
            "글자뽑기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "string, index",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let index = require_nonnegative(expect_int(&values[1], span)?, span)?;
                let ch = text.chars().nth(index);
                Ok(ch.map(|c| Value::Str(c.to_string())).unwrap_or(Value::None))
            }
            "글바꾸기" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "string, index, string",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let index = require_nonnegative(expect_int(&values[1], span)?, span)?;
                let replacement = match &values[2] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let mut start = None;
                let mut end = None;
                for (idx, (byte_idx, ch)) in text.char_indices().enumerate() {
                    if idx == index {
                        start = Some(byte_idx);
                        end = Some(byte_idx + ch.len_utf8());
                        break;
                    }
                }
                let Some(start) = start else {
                    return Ok(Value::Str(text));
                };
                let end = end.unwrap_or(start);
                let mut out = String::with_capacity(text.len() + replacement.len());
                out.push_str(&text[..start]);
                out.push_str(&replacement);
                out.push_str(&text[end..]);
                Ok(Value::Str(out))
            }
            "찾기" => {
                let (text, pattern) = expect_two_strings(values, span)?;
                let idx = text
                    .find(&pattern)
                    .map(|byte_idx| text[..byte_idx].chars().count() as i64)
                    .unwrap_or(-1);
                Ok(Value::Num(Quantity::new(Fixed64::from_int(idx), UnitDim::zero())))
            }
            "첫번째" => {
                let list = expect_list(values, span)?;
                Ok(list.items.first().cloned().unwrap_or(Value::None))
            }
            "마지막" => {
                let list = expect_list(values, span)?;
                Ok(list.items.last().cloned().unwrap_or(Value::None))
            }
            "추가" => {
                let (mut list, item) = expect_list_and_item(values, span)?;
                list.items.push(item);
                Ok(Value::List(list))
            }
            "제거" => {
                let (mut list, index) = expect_list_and_index(values, span)?;
                if index < list.items.len() {
                    list.items.remove(index);
                }
                Ok(Value::List(list))
            }
            "정렬" => {
                let (list, func) = expect_list_and_func(values, span)?;
                let mut keyed = Vec::with_capacity(list.items.len());
                for (idx, item) in list.items.iter().cloned().enumerate() {
                    let key = self.eval_callable(&func, &[item.clone()], span)?;
                    keyed.push((idx, key, item));
                }
                keyed.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
                let items = keyed.into_iter().map(|(_, _, item)| item).collect();
                Ok(Value::List(ListValue { items }))
            }
            "거르기" => {
                let (list, func) = expect_list_and_func(values, span)?;
                let mut items = Vec::new();
                for item in list.items.iter().cloned() {
                    let verdict = self.eval_callable(&func, &[item.clone()], span)?;
                    if is_truthy(&verdict, span)? {
                        items.push(item);
                    }
                }
                Ok(Value::List(ListValue { items }))
            }
            "변환" => {
                let (list, func) = expect_list_and_func(values, span)?;
                let mut items = Vec::with_capacity(list.items.len());
                for item in list.items.iter().cloned() {
                    let mapped = self.eval_callable(&func, &[item], span)?;
                    items.push(mapped);
                }
                Ok(Value::List(ListValue { items }))
            }
            "바꾸기" => {
                if values.len() == 3 {
                    let text = match &values[0] {
                        Value::Str(text) => text.clone(),
                        value => return Err(type_mismatch_detail("string", value, span)),
                    };
                    let from = match &values[1] {
                        Value::Str(text) => text.clone(),
                        value => return Err(type_mismatch_detail("string", value, span)),
                    };
                    let to = match &values[2] {
                        Value::Str(text) => text.clone(),
                        value => return Err(type_mismatch_detail("string", value, span)),
                    };
                    return Ok(Value::Str(text.replace(&from, &to)));
                }
                let (list, func) = expect_list_and_func(values, span)?;
                let mut items = Vec::with_capacity(list.items.len());
                for item in list.items.iter().cloned() {
                    let mapped = self.eval_callable(&func, &[item], span)?;
                    items.push(mapped);
                }
                Ok(Value::List(ListValue { items }))
            }
            "토막내기" => {
                let (list, start, end) = expect_list_slice(values, span)?;
                if start >= list.items.len() || end <= start {
                    return Ok(Value::List(ListValue { items: Vec::new() }));
                }
                let end = end.min(list.items.len());
                let items = list.items[start..end].to_vec();
                Ok(Value::List(ListValue { items }))
            }
            "들어있나" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "list, value",
                        span,
                    });
                }
                let list = match &values[0] {
                    Value::List(list) => list.clone(),
                    value => return Err(type_mismatch_detail("list", value, span)),
                };
                let target = &values[1];
                Ok(Value::Bool(list.items.iter().any(|item| item == target)))
            }
            "찾아보기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "list, value",
                        span,
                    });
                }
                let list = match &values[0] {
                    Value::List(list) => list.clone(),
                    value => return Err(type_mismatch_detail("list", value, span)),
                };
                let target = &values[1];
                let idx = list
                    .items
                    .iter()
                    .position(|item| item == target)
                    .map(|i| i as i64)
                    .unwrap_or(-1);
                Ok(Value::Num(Quantity::new(Fixed64::from_int(idx), UnitDim::zero())))
            }
            "뒤집기" => {
                let list = expect_list(values, span)?;
                let mut items = list.items;
                items.reverse();
                Ok(Value::List(ListValue { items }))
            }
            "펼치기" => {
                let list = expect_list(values, span)?;
                let mut out = Vec::new();
                for item in list.items {
                    let Value::List(inner) = item else {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "list<list>",
                            span,
                        });
                    };
                    out.extend(inner.items);
                }
                Ok(Value::List(ListValue { items: out }))
            }
            "각각돌며" => {
                let (list, func) = expect_list_and_func(values, span)?;
                for item in list.items.iter().cloned() {
                    let _ = self.eval_callable(&func, &[item], span)?;
                }
                Ok(Value::None)
            }
            "합치기" => {
                if values.len() == 2 {
                    match (&values[0], &values[1]) {
                        (Value::Str(left), Value::Str(right)) => {
                            let mut out = left.clone();
                            out.push_str(right);
                            return Ok(Value::Str(out));
                        }
                        (Value::Pack(left), Value::Pack(right)) => {
                            let mut fields = left.fields.clone();
                            for (key, value) in right.fields.iter() {
                                fields.insert(key.clone(), value.clone());
                            }
                            return Ok(Value::Pack(PackValue { fields }));
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "string or pack",
                                span,
                            })
                        }
                    }
                }
                let (list, initial, func) = expect_list_reduce(values, span)?;
                let mut acc = initial;
                for item in list.items.iter().cloned() {
                    acc = self.eval_callable(&func, &[acc, item], span)?;
                }
                Ok(acc)
            }
            "자르기" => {
                let (text, delim) = expect_two_strings(values, span)?;
                let items = if delim.is_empty() {
                    text.chars().map(|ch| Value::Str(ch.to_string())).collect()
                } else {
                    text.split(&delim)
                        .map(|part| Value::Str(part.to_string()))
                        .collect()
                };
                Ok(Value::List(ListValue { items }))
            }
            "붙이기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "list join or list concat",
                        span,
                    });
                }
                match (&values[0], &values[1]) {
                    (Value::List(left), Value::Str(delim)) => {
                        let mut out = String::new();
                        for (idx, item) in left.items.iter().enumerate() {
                            if idx > 0 {
                                out.push_str(delim);
                            }
                            let text = match item {
                                Value::Str(text) => text,
                                _ => {
                                    return Err(RuntimeError::TypeMismatch {
                                        expected: "string list",
                                        span,
                                    })
                                }
                            };
                            out.push_str(text);
                        }
                        Ok(Value::Str(out))
                    }
                    (Value::List(left), Value::List(right)) => {
                        let mut items = left.items.clone();
                        items.extend(right.items.clone());
                        Ok(Value::List(ListValue { items }))
                    }
                    _ => Err(RuntimeError::TypeMismatch {
                        expected: "list join or list concat",
                        span,
                    }),
                }
            }
            "포함하나" => {
                let (text, pattern) = expect_two_strings(values, span)?;
                Ok(Value::Bool(text.contains(&pattern)))
            }
            "시작하나" => {
                let (text, pattern) = expect_two_strings(values, span)?;
                Ok(Value::Bool(text.starts_with(&pattern)))
            }
            "끝나나" => {
                let (text, pattern) = expect_two_strings(values, span)?;
                Ok(Value::Bool(text.ends_with(&pattern)))
            }
            "숫자로" => {
                let text = expect_single_string(values, span)?;
                let parsed = Fixed64::parse_literal(&text);
                Ok(match parsed {
                    Some(raw) => Value::Num(Quantity::new(raw, UnitDim::zero())),
                    None => Value::None,
                })
            }
            "글로" => {
                let value = expect_any(values, span)?;
                Ok(Value::Str(value.display()))
            }
            "풀기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "formula, pack",
                        span,
                    });
                }
                let math = match &values[0] {
                    Value::Math(math) => math.clone(),
                    value => return Err(type_mismatch_detail("formula", value, span)),
                };
                let pack = match &values[1] {
                    Value::Pack(pack) => pack.clone(),
                    value => return Err(type_mismatch_detail("pack", value, span)),
                };
                let dialect = FormulaDialect::from_tag(&math.dialect).ok_or(RuntimeError::TypeMismatch {
                    expected: "formula",
                    span,
                })?;
                let analysis =
                    analyze_formula(&math.body, dialect).map_err(|err| map_formula_error(err, span))?;
                let required = analysis.vars;
                let provided: BTreeSet<String> = pack.fields.keys().cloned().collect();
                let missing: Vec<String> = required.difference(&provided).cloned().collect();
                if !missing.is_empty() {
                    return Err(RuntimeError::Pack {
                        message: format!("풀기: 주입 키가 누락되었습니다: {}", missing.join(", ")),
                        span,
                    });
                }
                let extra: Vec<String> = provided.difference(&required).cloned().collect();
                if !extra.is_empty() {
                    return Err(RuntimeError::Pack {
                        message: format!("풀기: 주입 키가 여분입니다: {}", extra.join(", ")),
                        span,
                    });
                }
                let mut map: BTreeMap<String, Quantity> = BTreeMap::new();
                for (key, value) in pack.fields.iter() {
                    if matches!(value, Value::None) {
                        return Err(RuntimeError::Pack {
                            message: format!("풀기: 키 '{}' 값이 없습니다", key),
                            span,
                        });
                    }
                    let qty = expect_quantity_value(value, span)?;
                    map.insert(key.clone(), qty);
                }
                let qty =
                    eval_formula_body(&math.body, dialect, &map).map_err(|err| map_formula_error(err, span))?;
                Ok(Value::Num(qty))
            }
            "눌렸나" => {
                let key = expect_single_string(values, span)?;
                let pressed = read_state_flag(&self.state, &format!("샘.키보드.누르고있음.{}", key))
                    || read_state_flag(&self.state, &format!("입력상태.키_누르고있음.{}", key));
                Ok(Value::Bool(pressed))
            }
            "막눌렸나" => {
                let key = expect_single_string(values, span)?;
                let pressed = read_state_flag(&self.state, &format!("샘.키보드.눌림.{}", key))
                    || read_state_flag(&self.state, &format!("입력상태.키_눌림.{}", key));
                Ok(Value::Bool(pressed))
            }
            "무작위" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                let value = self.next_rng_fixed64();
                Ok(Value::Num(Quantity::new(value, UnitDim::zero())))
            }
            "무작위정수" => {
                let (min, max) = expect_two_ints(values, span)?;
                if min > max {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "min <= max",
                        span,
                    });
                }
                let range = (max - min + 1) as u64;
                let value = (self.next_rng_u64() % range) as i64 + min;
                Ok(Value::Num(Quantity::new(Fixed64::from_int(value), UnitDim::zero())))
            }
            "무작위선택" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "list",
                        span,
                    });
                }
                let list = match &values[0] {
                    Value::List(list) => list.clone(),
                    value => return Err(type_mismatch_detail("list", value, span)),
                };
                if list.items.is_empty() {
                    return Ok(Value::None);
                }
                let idx = (self.next_rng_u64() % list.items.len() as u64) as usize;
                Ok(list.items[idx].clone())
            }
            "자원" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "string or resource",
                        span,
                    });
                }
                match &values[0] {
                    Value::Str(path) => Ok(Value::ResourceHandle(ResourceHandle::from_path(path))),
                    Value::ResourceHandle(handle) => Ok(Value::ResourceHandle(*handle)),
                    value => Err(type_mismatch_detail("string or resource", value, span)),
                }
            }
            "감정더하기" => {
                let (dv, da) = expect_two_quantities(values, span)?;
                ensure_dimensionless(&dv, span)?;
                ensure_dimensionless(&da, span)?;
                let mut valence = Fixed64::from_int(0);
                let mut arousal = Fixed64::from_int(0);
                if let Some(value) = self.state.get(&Key::new("감정씨")) {
                    let pack = match value {
                        Value::Pack(pack) => pack,
                        _ => return Err(type_mismatch_detail("pack", value, span)),
                    };
                    let valence_value = pack.fields.get("valence").ok_or(RuntimeError::Pack {
                        message: "감정씨.valence가 없습니다".to_string(),
                        span,
                    })?;
                    let arousal_value = pack.fields.get("arousal").ok_or(RuntimeError::Pack {
                        message: "감정씨.arousal가 없습니다".to_string(),
                        span,
                    })?;
                    valence = expect_quantity_value(valence_value, span)?.raw;
                    arousal = expect_quantity_value(arousal_value, span)?.raw;
                }
                let min_valence = Fixed64::from_int(-1);
                let max_valence = Fixed64::from_int(1);
                let min_arousal = Fixed64::from_int(0);
                let max_arousal = Fixed64::from_int(1);
                valence = clamp_fixed64(valence.saturating_add(dv.raw), min_valence, max_valence);
                arousal = clamp_fixed64(arousal.saturating_add(da.raw), min_arousal, max_arousal);
                let mut fields = BTreeMap::new();
                fields.insert(
                    "valence".to_string(),
                    Value::Num(Quantity::new(valence, UnitDim::zero())),
                );
                fields.insert(
                    "arousal".to_string(),
                    Value::Num(Quantity::new(arousal, UnitDim::zero())),
                );
                let pack = PackValue { fields };
                self.state.set(Key::new("감정씨"), Value::Pack(pack.clone()));
                Ok(Value::Pack(pack))
            }
            "말결값" => {
                let text = expect_single_string(values, span)?;
                let value = nuance_weight(&text);
                Ok(Value::Num(Quantity::new(value, UnitDim::zero())))
            }
            "기억하기" => {
                if values.len() < 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "kind, text, a?, b? (optional target first)",
                        span,
                    });
                }
                let (target, offset) = match values.first() {
                    Some(Value::Str(_)) => (None, 0),
                    Some(_) => (Some(values[0].clone()), 1),
                    None => (None, 0),
                };
                if values.len() < offset + 2 || values.len() > offset + 4 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "kind, text, a?, b? (optional target first)",
                        span,
                    });
                }
                let kind = match &values[offset] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let text = match &values[offset + 1] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let a_value = if values.len() > offset + 2 {
                    Some(values[offset + 2].clone())
                } else {
                    None
                };
                let b_value = if values.len() > offset + 3 {
                    Some(values[offset + 3].clone())
                } else {
                    None
                };
                if let Some(value) = &a_value {
                    let _ = expect_quantity_value(value, span)?;
                }
                if let Some(value) = &b_value {
                    let _ = expect_quantity_value(value, span)?;
                }
                let mut events = Vec::new();
                if let Some(value) = self.state.get(&Key::new("기억씨")) {
                    let pack = match value {
                        Value::Pack(pack) => pack,
                        _ => return Err(type_mismatch_detail("pack", value, span)),
                    };
                    let list_value = pack.fields.get("events").ok_or(RuntimeError::Pack {
                        message: "기억씨.events가 없습니다".to_string(),
                        span,
                    })?;
                    let list = match list_value {
                        Value::List(list) => list,
                        other => return Err(type_mismatch_detail("list", other, span)),
                    };
                    events = list.items.clone();
                }
                let madi = self.current_madi.get();
                let mut fields = BTreeMap::new();
                fields.insert(
                    "madi".to_string(),
                    Value::Num(Quantity::new(Fixed64::from_int(madi as i64), UnitDim::zero())),
                );
                fields.insert("kind".to_string(), Value::Str(kind));
                fields.insert("text".to_string(), Value::Str(text));
                fields.insert(
                    "a".to_string(),
                    a_value.unwrap_or(Value::None),
                );
                fields.insert(
                    "b".to_string(),
                    b_value.unwrap_or(Value::None),
                );
                fields.insert(
                    "target".to_string(),
                    target.unwrap_or(Value::None),
                );
                let event_pack = PackValue { fields };
                events.push(Value::Pack(event_pack.clone()));
                let mut memory_fields = BTreeMap::new();
                memory_fields.insert("events".to_string(), Value::List(ListValue { items: events }));
                let memory_pack = PackValue { fields: memory_fields };
                self.state.set(Key::new("기억씨"), Value::Pack(memory_pack));
                Ok(Value::Pack(event_pack))
            }
            "마지막기억" => {
                if values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "kind (optional target first)",
                        span,
                    });
                }
                let (target, offset) = match values.first() {
                    Some(Value::Str(_)) => (None, 0),
                    Some(_) => (Some(values[0].clone()), 1),
                    None => (None, 0),
                };
                if values.len() != offset + 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "kind (optional target first)",
                        span,
                    });
                }
                let kind = match &values[offset] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let Some(value) = self.state.get(&Key::new("기억씨")) else {
                    return Ok(Value::None);
                };
                let pack = match value {
                    Value::Pack(pack) => pack,
                    _ => return Err(type_mismatch_detail("pack", value, span)),
                };
                let list_value = pack.fields.get("events").ok_or(RuntimeError::Pack {
                    message: "기억씨.events가 없습니다".to_string(),
                    span,
                })?;
                let list = match list_value {
                    Value::List(list) => list,
                    other => return Err(type_mismatch_detail("list", other, span)),
                };
                for event in list.items.iter().rev() {
                    let Value::Pack(event_pack) = event else {
                        return Err(type_mismatch_detail("pack", event, span));
                    };
                    let kind_value = event_pack.fields.get("kind").ok_or(RuntimeError::Pack {
                        message: "기억씨.events.kind가 없습니다".to_string(),
                        span,
                    })?;
                    let event_kind = match kind_value {
                        Value::Str(text) => text,
                        other => return Err(type_mismatch_detail("string", other, span)),
                    };
                    if event_kind != &kind {
                        continue;
                    }
                    if let Some(target_value) = &target {
                        let stored = event_pack.fields.get("target").unwrap_or(&Value::None);
                        if stored != target_value {
                            continue;
                        }
                    }
                    return Ok(Value::Pack(event_pack.clone()));
                }
                Ok(Value::None)
            }
            "미분하기" => {
                let (math, options) = expect_formula_transform(values, span, "미분하기")?;
                let transformed = transform_formula_value(math, options, "diff", span)?;
                Ok(Value::Math(transformed))
            }
            "적분하기" => {
                let (math, options) = expect_formula_transform(values, span, "적분하기")?;
                let transformed = transform_formula_value(math, options, "int", span)?;
                Ok(Value::Math(transformed))
            }
            "맞추기" => {
                let (template, target) = expect_template_and_text(values, span)?;
                let matched = match_template(&template.body, &target, span)?;
                if let Some(captures) = matched {
                    let mut entries = BTreeMap::new();
                    for (name, value) in captures {
                        let key = Value::Str(name.clone());
                        entries.insert(
                            key.canon(),
                            MapEntry {
                                key,
                                value,
                            },
                        );
                    }
                    Ok(Value::Map(MapValue { entries }))
                } else {
                    Ok(Value::None)
                }
            }
            "차림" | "목록" => Ok(Value::List(ListValue { items: values.to_vec() })),
            "차림.값" => {
                let (list, index) = expect_list_and_index(values, span)?;
                Ok(list.items.get(index).cloned().unwrap_or(Value::None))
            }
            "차림.바꾼값" => {
                let (list, index, value) = expect_list_index_and_value(values, span)?;
                if index >= list.items.len() {
                    return Err(RuntimeError::IndexOutOfRange { span });
                }
                let mut items = list.items.clone();
                items[index] = value;
                Ok(Value::List(ListValue { items }))
            }
            "텐서.형상" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "tensor",
                        span,
                    });
                }
                let tensor = expect_tensor(&values[0], span)?;
                Ok(Value::List(tensor.shape))
            }
            "텐서.자료" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "tensor",
                        span,
                    });
                }
                let tensor = expect_tensor(&values[0], span)?;
                Ok(Value::List(tensor.data))
            }
            "텐서.배치" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "tensor",
                        span,
                    });
                }
                let tensor = expect_tensor(&values[0], span)?;
                Ok(Value::Str(tensor.layout))
            }
            "텐서.값" => {
                let (tensor, row, col) = expect_tensor_and_indices(values, span)?;
                if row >= tensor.rows || col >= tensor.cols {
                    return Ok(Value::None);
                }
                let index = match row.checked_mul(tensor.cols).and_then(|base| base.checked_add(col)) {
                    Some(index) => index,
                    None => return Ok(Value::None),
                };
                Ok(tensor.data.items.get(index).cloned().unwrap_or(Value::None))
            }
            "텐서.바꾼값" => {
                let (tensor, row, col, value) = expect_tensor_index_value(values, span)?;
                if row >= tensor.rows || col >= tensor.cols {
                    return Err(RuntimeError::IndexOutOfRange { span });
                }
                let index = match row.checked_mul(tensor.cols).and_then(|base| base.checked_add(col)) {
                    Some(index) => index,
                    None => return Err(RuntimeError::IndexOutOfRange { span }),
                };
                if index >= tensor.data.items.len() {
                    return Err(RuntimeError::IndexOutOfRange { span });
                }
                let mut items = tensor.data.items.clone();
                items[index] = value;
                let mut fields = tensor.pack.fields.clone();
                fields.insert("자료".to_string(), Value::List(ListValue { items }));
                Ok(Value::Pack(PackValue { fields }))
            }
            "모음" => {
                let mut items = BTreeMap::new();
                for item in values {
                    let key = item.canon();
                    items.entry(key).or_insert_with(|| item.clone());
                }
                Ok(Value::Set(SetValue { items }))
            }
            "짝맞춤" => {
                if values.len() % 2 != 0 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "even key/value args",
                        span,
                    });
                }
                let mut entries = BTreeMap::new();
                for chunk in values.chunks(2) {
                    let key_value = chunk[0].clone();
                    let entry = MapEntry {
                        key: key_value.clone(),
                        value: chunk[1].clone(),
                    };
                    entries.insert(key_value.canon(), entry);
                }
                Ok(Value::Map(MapValue { entries }))
            }
            "묶음값" => {
                let (pack, key) = expect_pack_and_key(&values, span)?;
                Ok(pack.fields.get(&key).cloned().unwrap_or(Value::None))
            }
            "키목록" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "pack",
                        span,
                    });
                }
                let pack = match &values[0] {
                    Value::Pack(pack) => pack.clone(),
                    value => return Err(type_mismatch_detail("pack", value, span)),
                };
                let items = pack
                    .fields
                    .keys()
                    .cloned()
                    .map(Value::Str)
                    .collect::<Vec<_>>();
                Ok(Value::List(ListValue { items }))
            }
            "값목록" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "pack",
                        span,
                    });
                }
                let pack = match &values[0] {
                    Value::Pack(pack) => pack.clone(),
                    value => return Err(type_mismatch_detail("pack", value, span)),
                };
                let items = pack.fields.values().cloned().collect::<Vec<_>>();
                Ok(Value::List(ListValue { items }))
            }
            "쌍목록" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "pack",
                        span,
                    });
                }
                let pack = match &values[0] {
                    Value::Pack(pack) => pack.clone(),
                    value => return Err(type_mismatch_detail("pack", value, span)),
                };
                let items = pack
                    .fields
                    .iter()
                    .map(|(key, value)| {
                        Value::List(ListValue {
                            items: vec![Value::Str(key.clone()), value.clone()],
                        })
                    })
                    .collect::<Vec<_>>();
                Ok(Value::List(ListValue { items }))
            }
            "tetris_board_new" => {
                let (width, height) = expect_two_ints(&values, span)?;
                let width = require_nonnegative(width, span)?;
                let height = require_nonnegative(height, span)?;
                let size = width
                    .checked_mul(height)
                    .ok_or(RuntimeError::TypeMismatch {
                        expected: "board size",
                        span,
                    })?;
                let board = ".".repeat(size);
                Ok(Value::Str(board))
            }
            "tetris_can_place" => {
                let args = expect_tetris_args(&values, span)?;
                let ok = tetris_can_place(&args, span)?;
                Ok(Value::Bool(ok))
            }
            "tetris_drop_y" => {
                let args = expect_tetris_args(&values, span)?;
                validate_board(&args, span)?;
                let mut y = args.y;
                while tetris_can_place_at(&args, args.x, y + 1)? {
                    y += 1;
                }
                Ok(Value::Num(Quantity::new(Fixed64::from_int(i64::from(y)), UnitDim::zero())))
            }
            "tetris_lock" => {
                let args = expect_tetris_args(&values, span)?;
                let (board, cleared, locked) = tetris_lock(&args, span)?;
                let mut fields = BTreeMap::new();
                fields.insert("보드".to_string(), Value::Str(board));
                fields.insert(
                    "지운줄".to_string(),
                    Value::Num(Quantity::new(Fixed64::from_int(cleared), UnitDim::zero())),
                );
                fields.insert("놓기됨".to_string(), Value::Bool(locked));
                Ok(Value::Pack(PackValue { fields }))
            }
            "tetris_board_cell" => {
                let args = expect_tetris_cell_args(&values, span)?;
                let (uri, color) = tetris_board_cell(&args, span)?;
                let mut fields = BTreeMap::new();
                fields.insert("uri".to_string(), Value::Str(uri));
                fields.insert("color".to_string(), Value::Str(color));
                Ok(Value::Pack(PackValue { fields }))
            }
            "tetris_board_drawlist" => {
                let args = expect_tetris_drawlist_args(&values, span)?;
                let list = tetris_board_drawlist(&args, span)?;
                Ok(Value::List(list))
            }
            "tetris_piece_block" => {
                let args = expect_tetris_block_args(&values, span)?;
                let (dx, dy) = tetris_piece_block(&args);
                let mut fields = BTreeMap::new();
                fields.insert(
                    "dx".to_string(),
                    Value::Num(Quantity::new(Fixed64::from_int(i64::from(dx)), UnitDim::zero())),
                );
                fields.insert(
                    "dy".to_string(),
                    Value::Num(Quantity::new(Fixed64::from_int(i64::from(dy)), UnitDim::zero())),
                );
                Ok(Value::Pack(PackValue { fields }))
            }
            _ => {
                if let Some(seed) = self.user_seeds.get(name).cloned() {
                    return self.eval_user_seed(&seed, values, span);
                }
                Err(RuntimeError::TypeMismatch {
                    expected: "known function",
                    span,
                })
            }
        }
    }

    fn next_rng_u64(&self) -> u64 {
        let state = self.rng_state.get().wrapping_add(0x9E3779B97F4A7C15);
        self.rng_state.set(state);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_rng_fixed64(&self) -> Fixed64 {
        let value = self.next_rng_u64();
        Fixed64::from_raw((value & 0xFFFF_FFFF) as i64)
    }

    fn canonicalize_stdlib_alias(name: &str) -> &str {
        match name {
            "갈라놓기" => "자르기",
            "이어붙이기" => "붙이기",
            "길이세기" => "길이",
            "값뽑기" => "차림.값",
            "번째" => "차림.값",
            "올림" => "천장",
            "내림" => "바닥",
            "절댓값" => "abs",
            "제곱근" => "sqrt",
            "제곱" => "powi",
            "거듭제곱" => "powi",
            "최댓값" => "max",
            "최솟값" => "min",
            _ => name,
        }
    }

    fn eval_formula_value(
        &self,
        dialect: FormulaDialect,
        body: &str,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let formatted = format_formula_body(body, dialect).map_err(|err| map_formula_error(err, span))?;
        Ok(Value::Math(crate::core::value::MathValue {
            dialect: dialect.tag().to_string(),
            body: formatted,
        }))
    }

    fn eval_formula_eval(
        &mut self,
        dialect: FormulaDialect,
        body: &str,
        bindings: &[Binding],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        self.eval_formula_bindings(dialect, body, bindings, span)
    }

    fn eval_formula_bindings(
        &mut self,
        dialect: FormulaDialect,
        body: &str,
        bindings: &[Binding],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let mut map: BTreeMap<String, Quantity> = BTreeMap::new();
        for binding in bindings {
            if dialect == FormulaDialect::Ascii1 && binding.name.chars().count() != 1 {
                return Err(RuntimeError::FormulaIdentNotAscii1 { span: binding.span });
            }
            let value = self.eval_expr(&binding.value)?;
            let qty = match value {
                Value::Num(qty) => qty,
                _ => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "number",
                        span: binding.span,
                    })
                }
            };
            map.insert(binding.name.clone(), qty);
        }
        let qty = eval_formula_body(body, dialect, &map).map_err(|err| map_formula_error(err, span))?;
        Ok(Value::Num(qty))
    }

    fn eval_formula_fill(
        &mut self,
        formula: &Expr,
        bindings: &[Binding],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(formula)?;
        let math = match value {
            Value::Math(math) => math,
            _ => {
                return Err(RuntimeError::TypeMismatch {
                    expected: "formula",
                    span,
                })
            }
        };
        let dialect = FormulaDialect::from_tag(&math.dialect).ok_or(RuntimeError::TypeMismatch {
            expected: "formula",
            span,
        })?;
        self.eval_formula_bindings(dialect, &math.body, bindings, span)
    }

    fn eval_template_fill(
        &mut self,
        template: &Expr,
        bindings: &[Binding],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let template_value = self.eval_expr(template)?;
        let body = match template_value {
            Value::Template(template) => template.body,
            _ => {
                return Err(RuntimeError::Template {
                    message: "글무늬값이 필요합니다".to_string(),
                    span,
                })
            }
        };

        let mut map = BTreeMap::new();
        for binding in bindings {
            if map.contains_key(&binding.name) {
                return Err(RuntimeError::Template {
                    message: format!("주입 키 중복: {}", binding.name),
                    span: binding.span,
                });
            }
            let value = self.eval_expr(&binding.value)?;
            map.insert(binding.name.clone(), value);
        }

        let rendered = render_template(&body, &map, span)?;
        Ok(Value::Str(rendered))
    }

    fn eval_pack(&mut self, bindings: &[Binding], _span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
        let mut map = BTreeMap::new();
        for binding in bindings {
            if map.contains_key(&binding.name) {
                return Err(RuntimeError::Pack {
                    message: format!("묶음 키 중복: {}", binding.name),
                    span: binding.span,
                });
            }
            let value = self.eval_expr(&binding.value)?;
            map.insert(binding.name.clone(), value);
        }
        Ok(Value::Pack(PackValue { fields: map }))
    }

    fn eval_add(&self, left: Value, right: Value, span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        if l.dim != r.dim {
            return Err(RuntimeError::UnitMismatch { span });
        }
        Ok(Value::Num(Quantity::new(
            l.raw.saturating_add(r.raw),
            l.dim,
        )))
    }

    fn eval_sub(&self, left: Value, right: Value, span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        if l.dim != r.dim {
            return Err(RuntimeError::UnitMismatch { span });
        }
        Ok(Value::Num(Quantity::new(
            l.raw.saturating_sub(r.raw),
            l.dim,
        )))
    }

    fn eval_mul(&self, left: Value, right: Value, span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        Ok(Value::Num(Quantity::new(
            l.raw.saturating_mul(r.raw),
            l.dim.add(r.dim),
        )))
    }

    fn eval_div(&self, left: Value, right: Value, span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        let raw = l.raw.checked_div(r.raw).ok_or(RuntimeError::MathDivZero { span })?;
        let dim = l.dim.add(r.dim.scale(-1));
        Ok(Value::Num(Quantity::new(raw, dim)))
    }

    fn eval_callable(
        &mut self,
        callable: &Callable,
        args: &[Value],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        match callable {
            Callable::Name(name) => self.eval_call_values(name, args, span),
            Callable::Lambda(lambda) => self.eval_lambda(lambda, args, span),
        }
    }

    fn eval_lambda(
        &mut self,
        lambda: &LambdaValue,
        args: &[Value],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 1 {
            return Err(RuntimeError::TypeMismatch {
                expected: "lambda argument",
                span,
            });
        }
        let mut state = self.state.clone();
        state.set(Key::new(lambda.param.clone()), args[0].clone());
        let mut evaluator = Evaluator::with_state_and_seed(state, self.rng_state.get());
        evaluator.current_madi.set(self.current_madi.get());
        let result = evaluator.eval_expr(&lambda.body)?;
        self.rng_state.set(evaluator.rng_state.get());
        Ok(result)
    }

    fn eval_user_seed(
        &mut self,
        seed: &UserSeed,
        args: &[Value],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != seed.params.len() {
            return Err(RuntimeError::TypeMismatch {
                expected: "seed arguments",
                span,
            });
        }
        let mut prev = Vec::new();
        for (param, value) in seed.params.iter().zip(args.iter()) {
            let key = Key::new(param.clone());
            let prev_value = self.state.get(&key).cloned();
            prev.push((key.clone(), prev_value));
            self.state.set(key, value.clone());
        }

        let flow = self.eval_block(&seed.body);

        for (key, prev_value) in prev {
            if let Some(value) = prev_value {
                self.state.set(key, value);
            } else {
                self.state.resources.remove(&key);
            }
        }

        match flow? {
            FlowControl::Continue => Ok(Value::None),
            FlowControl::Return(value, _) => Ok(value),
            FlowControl::Break(span) => Err(RuntimeError::BreakOutsideLoop { span }),
        }
    }

    fn open_site_id(&self, open_kind: &str) -> String {
        let seq = self.open_seq.get();
        self.open_seq.set(seq.saturating_add(1));
        let madi = self.current_madi.get();
        let raw = format!(
            "{}#{}#{}#{}",
            self.open_source, madi, seq, open_kind
        );
        let hash = blake3::hash(raw.as_bytes()).to_hex();
        format!("blake3:{}", hash)
    }

    fn eval_compare(
        &self,
        op: &BinaryOp,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        if matches!(op, BinaryOp::Eq | BinaryOp::NotEq) {
            match (&left, &right) {
                (Value::Str(a), Value::Str(b)) => {
                    let equal = a == b;
                    return Ok(Value::Bool(if matches!(op, BinaryOp::Eq) { equal } else { !equal }));
                }
                (Value::Bool(a), Value::Bool(b)) => {
                    let equal = a == b;
                    return Ok(Value::Bool(if matches!(op, BinaryOp::Eq) { equal } else { !equal }));
                }
                (Value::None, Value::None) => {
                    return Ok(Value::Bool(matches!(op, BinaryOp::Eq)));
                }
                _ => {}
            }
        }
        let (l, r) = self.require_numbers(left, right, span)?;
        if l.dim != r.dim {
            return Err(RuntimeError::UnitMismatch { span });
        }
        let ordering = l.raw.raw().cmp(&r.raw.raw());
        let result = match op {
            BinaryOp::Eq => ordering == std::cmp::Ordering::Equal,
            BinaryOp::NotEq => ordering != std::cmp::Ordering::Equal,
            BinaryOp::Lt => ordering == std::cmp::Ordering::Less,
            BinaryOp::Lte => ordering != std::cmp::Ordering::Greater,
            BinaryOp::Gt => ordering == std::cmp::Ordering::Greater,
            BinaryOp::Gte => ordering != std::cmp::Ordering::Less,
            _ => false,
        };
        Ok(Value::Bool(result))
    }

    fn require_numbers(
        &self,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<(Quantity, Quantity), RuntimeError> {
        match (left, right) {
            (Value::Num(l), Value::Num(r)) => Ok((l, r)),
            (Value::Num(_), other) => Err(type_mismatch_detail("number", &other, span)),
            (other, _) => Err(type_mismatch_detail("number", &other, span)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContractDiag {
    pub kind: ContractKind,
    pub mode: ContractMode,
    pub message: String,
    pub span: crate::lang::span::Span,
}

impl Evaluator {
    fn emit_contract_violation(&mut self, kind: ContractKind, mode: ContractMode, span: crate::lang::span::Span) {
        let message = match kind {
            ContractKind::Pre => "전제하에 조건이 실패했습니다".to_string(),
            ContractKind::Post => "보장하고 조건이 실패했습니다".to_string(),
        };
        self.contract_diags.push(ContractDiag {
            kind,
            mode,
            message,
            span,
        });
    }
}

fn no_op_before_tick(_: u64, _: &mut State) -> Result<(), RuntimeError> {
    Ok(())
}

fn no_op_should_stop(_: u64, _: &State) -> bool {
    false
}

pub struct EvalOutput {
    pub state: State,
    pub trace: Trace,
    pub bogae_requested: bool,
    pub contract_diags: Vec<ContractDiag>,
}

fn map_formula_error(err: FormulaError, span: crate::lang::span::Span) -> RuntimeError {
    match err {
        FormulaError::Parse(message) => RuntimeError::FormulaParse { message, span },
        FormulaError::Undefined(name) => RuntimeError::FormulaUndefined { name, span },
        FormulaError::IdentNotAscii1 => RuntimeError::FormulaIdentNotAscii1 { span },
        FormulaError::ExtUnsupported { name } => RuntimeError::FormulaExtUnsupported { name, span },
        FormulaError::UnitMismatch => RuntimeError::UnitMismatch { span },
        FormulaError::DivZero => RuntimeError::MathDivZero { span },
    }
}

fn type_mismatch_detail(
    expected: &'static str,
    actual: &Value,
    span: crate::lang::span::Span,
) -> RuntimeError {
    RuntimeError::TypeMismatchDetail {
        expected,
        actual: value_type_name(actual),
        span,
    }
}

fn value_type_name(value: &Value) -> String {
    match value {
        Value::None => "none".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Num(qty) => number_type_name(qty),
        Value::Str(_) => "string".to_string(),
        Value::ResourceHandle(_) => "resource".to_string(),
        Value::Math(_) => "formula".to_string(),
        Value::Template(_) => "template".to_string(),
        Value::Lambda(_) => "seed".to_string(),
        Value::Pack(_) => "pack".to_string(),
        Value::List(_) => "list".to_string(),
        Value::Set(_) => "set".to_string(),
        Value::Map(_) => "map".to_string(),
    }
}

fn read_state_flag(state: &State, key: &str) -> bool {
    let Some(value) = state.get(&Key::new(key)) else {
        return false;
    };
    match value {
        Value::Bool(flag) => *flag,
        Value::Num(qty) => qty.raw.raw() != 0,
        _ => false,
    }
}

fn number_type_name(qty: &Quantity) -> String {
    if qty.dim.is_dimensionless() {
        if qty.raw.raw() % Fixed64::SCALE == 0 {
            "integer".to_string()
        } else {
            "number".to_string()
        }
    } else {
        let unit = format_dim(qty.dim);
        if unit.is_empty() {
            "number".to_string()
        } else {
            format!("number@{}", unit)
        }
    }
}

fn expect_quantity(values: &[Value], expected: usize, span: crate::lang::span::Span) -> Result<Quantity, RuntimeError> {
    if values.len() != expected {
        return Err(RuntimeError::TypeMismatch {
            expected: "number",
            span,
        });
    }
    match values.first() {
        Some(Value::Num(qty)) => Ok(qty.clone()),
        Some(value) => Err(type_mismatch_detail("number", value, span)),
        None => Err(RuntimeError::TypeMismatch {
            expected: "number",
            span,
        }),
    }
}

fn expect_two_quantities(values: &[Value], span: crate::lang::span::Span) -> Result<(Quantity, Quantity), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "two numbers",
            span,
        });
    }
    match (&values[0], &values[1]) {
        (Value::Num(a), Value::Num(b)) => Ok((a.clone(), b.clone())),
        (Value::Num(_), other) => Err(type_mismatch_detail("number", other, span)),
        (other, _) => Err(type_mismatch_detail("number", other, span)),
    }
}

fn expect_three_quantities(values: &[Value], span: crate::lang::span::Span) -> Result<(Quantity, Quantity, Quantity), RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "three numbers",
            span,
        });
    }
    match (&values[0], &values[1], &values[2]) {
        (Value::Num(a), Value::Num(b), Value::Num(c)) => Ok((a.clone(), b.clone(), c.clone())),
        (Value::Num(_), Value::Num(_), other) => Err(type_mismatch_detail("number", other, span)),
        (Value::Num(_), other, _) => Err(type_mismatch_detail("number", other, span)),
        (other, _, _) => Err(type_mismatch_detail("number", other, span)),
    }
}

fn expect_quantity_value(value: &Value, span: crate::lang::span::Span) -> Result<Quantity, RuntimeError> {
    match value {
        Value::Num(qty) => Ok(qty.clone()),
        other => Err(type_mismatch_detail("number", other, span)),
    }
}

fn expect_pack_and_key(values: &[Value], span: crate::lang::span::Span) -> Result<(PackValue, String), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "pack, string",
            span,
        });
    }
    let pack = match &values[0] {
        Value::Pack(pack) => pack.clone(),
        value => return Err(type_mismatch_detail("pack", value, span)),
    };
    let key = match &values[1] {
        Value::Str(text) => text.clone(),
        value => return Err(type_mismatch_detail("string key", value, span)),
    };
    Ok((pack, key))
}

fn expect_any(values: &[Value], span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "single value",
            span,
        });
    }
    Ok(values[0].clone())
}

fn expect_single_string(values: &[Value], span: crate::lang::span::Span) -> Result<String, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "string",
            span,
        });
    }
    match &values[0] {
        Value::Str(text) => Ok(text.clone()),
        value => Err(type_mismatch_detail("string", value, span)),
    }
}

fn expect_open_path(values: &[Value], span: crate::lang::span::Span) -> Result<String, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "string or pack{경로}",
            span,
        });
    }
    match &values[0] {
        Value::Str(text) => Ok(text.clone()),
        Value::Pack(pack) => {
            let Some(value) = pack.fields.get("경로") else {
                return Err(RuntimeError::TypeMismatch {
                    expected: "pack with 경로",
                    span,
                });
            };
            match value {
                Value::Str(text) => Ok(text.clone()),
                other => Err(type_mismatch_detail("string", other, span)),
            }
        }
        value => Err(type_mismatch_detail("string or pack", value, span)),
    }
}

struct OpenNetArgs {
    url: String,
    method: String,
    body: Option<String>,
    response: Option<String>,
}

struct OpenFfiArgs {
    name: String,
    args: Option<Vec<String>>,
    result: Option<String>,
}

struct OpenGpuArgs {
    kernel: String,
    payload: Option<String>,
    result: Option<String>,
}

fn expect_open_net_args(values: &[Value], span: crate::lang::span::Span) -> Result<OpenNetArgs, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "string or pack{주소}",
            span,
        });
    }
    match &values[0] {
        Value::Str(text) => Ok(OpenNetArgs {
            url: text.clone(),
            method: "GET".to_string(),
            body: None,
            response: None,
        }),
        Value::Pack(pack) => {
            let url = expect_pack_string(pack, span, &["주소", "url"])?;
            let method = match get_pack_string(pack, span, &["방법", "method"])? {
                Some(value) => value,
                None => "GET".to_string(),
            };
            let body = get_pack_string(pack, span, &["본문", "body"])?;
            let response = get_pack_string(pack, span, &["응답", "response"])?;
            Ok(OpenNetArgs {
                url,
                method,
                body,
                response,
            })
        }
        value => Err(type_mismatch_detail("string or pack", value, span)),
    }
}

fn expect_open_ffi_args(values: &[Value], span: crate::lang::span::Span) -> Result<OpenFfiArgs, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "string or pack{이름}",
            span,
        });
    }
    match &values[0] {
        Value::Str(text) => Ok(OpenFfiArgs {
            name: text.clone(),
            args: None,
            result: None,
        }),
        Value::Pack(pack) => {
            let name = expect_pack_string(pack, span, &["이름", "함수", "name"])?;
            let args = match pack.fields.get("인자").or_else(|| pack.fields.get("args")) {
                Some(Value::List(list)) => {
                    let mut items = Vec::new();
                    for item in &list.items {
                        match item {
                            Value::Str(text) => items.push(text.clone()),
                            other => return Err(type_mismatch_detail("string", other, span)),
                        }
                    }
                    Some(items)
                }
                Some(Value::Str(text)) => Some(vec![text.clone()]),
                Some(other) => return Err(type_mismatch_detail("list or string", other, span)),
                None => None,
            };
            let result = get_pack_string(pack, span, &["결과", "result"])?;
            Ok(OpenFfiArgs { name, args, result })
        }
        value => Err(type_mismatch_detail("string or pack", value, span)),
    }
}

fn expect_open_gpu_args(values: &[Value], span: crate::lang::span::Span) -> Result<OpenGpuArgs, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "string or pack{커널}",
            span,
        });
    }
    match &values[0] {
        Value::Str(text) => Ok(OpenGpuArgs {
            kernel: text.clone(),
            payload: None,
            result: None,
        }),
        Value::Pack(pack) => {
            let kernel = expect_pack_string(pack, span, &["커널", "kernel"])?;
            let payload = get_pack_string(pack, span, &["입력", "payload"])?;
            let result = get_pack_string(pack, span, &["결과", "result"])?;
            Ok(OpenGpuArgs {
                kernel,
                payload,
                result,
            })
        }
        value => Err(type_mismatch_detail("string or pack", value, span)),
    }
}

fn expect_pack_string(
    pack: &PackValue,
    span: crate::lang::span::Span,
    keys: &[&str],
) -> Result<String, RuntimeError> {
    let value = find_pack_value(pack, keys).ok_or(RuntimeError::TypeMismatch {
        expected: "pack with required field",
        span,
    })?;
    match value {
        Value::Str(text) => Ok(text.clone()),
        other => Err(type_mismatch_detail("string", other, span)),
    }
}

fn get_pack_string(
    pack: &PackValue,
    span: crate::lang::span::Span,
    keys: &[&str],
) -> Result<Option<String>, RuntimeError> {
    let Some(value) = find_pack_value(pack, keys) else {
        return Ok(None);
    };
    match value {
        Value::Str(text) => Ok(Some(text.clone())),
        other => Err(type_mismatch_detail("string", other, span)),
    }
}

fn find_pack_value<'a>(pack: &'a PackValue, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(value) = pack.fields.get(*key) {
            return Some(value);
        }
    }
    None
}

fn expect_two_strings(values: &[Value], span: crate::lang::span::Span) -> Result<(String, String), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "two strings",
            span,
        });
    }
    let left = match &values[0] {
        Value::Str(text) => text.clone(),
        value => return Err(type_mismatch_detail("string", value, span)),
    };
    let right = match &values[1] {
        Value::Str(text) => text.clone(),
        value => return Err(type_mismatch_detail("string", value, span)),
    };
    Ok((left, right))
}

fn expect_list(values: &[Value], span: crate::lang::span::Span) -> Result<ListValue, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list",
            span,
        });
    }
    match &values[0] {
        Value::List(list) => Ok(list.clone()),
        value => Err(type_mismatch_detail("list", value, span)),
    }
}

fn expect_list_and_item(values: &[Value], span: crate::lang::span::Span) -> Result<(ListValue, Value), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list, item",
            span,
        });
    }
    let list = match &values[0] {
        Value::List(list) => list.clone(),
        value => return Err(type_mismatch_detail("list", value, span)),
    };
    Ok((list, values[1].clone()))
}

fn expect_list_and_index(values: &[Value], span: crate::lang::span::Span) -> Result<(ListValue, usize), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list, index",
            span,
        });
    }
    let list = match &values[0] {
        Value::List(list) => list.clone(),
        value => return Err(type_mismatch_detail("list", value, span)),
    };
    let raw_index = expect_int(&values[1], span)?;
    let index = require_nonnegative(raw_index, span)?;
    Ok((list, index))
}

fn expect_list_index_and_value(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(ListValue, usize, Value), RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list, index, value",
            span,
        });
    }
    let list = match &values[0] {
        Value::List(list) => list.clone(),
        value => return Err(type_mismatch_detail("list", value, span)),
    };
    let raw_index = expect_int(&values[1], span)?;
    let index = require_nonnegative(raw_index, span)?;
    Ok((list, index, values[2].clone()))
}

fn expect_list_slice(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(ListValue, usize, usize), RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list, start, end",
            span,
        });
    }
    let list = match &values[0] {
        Value::List(list) => list.clone(),
        value => return Err(type_mismatch_detail("list", value, span)),
    };
    let start = require_nonnegative(expect_int(&values[1], span)?, span)?;
    let end = require_nonnegative(expect_int(&values[2], span)?, span)?;
    Ok((list, start, end))
}

struct TensorValue {
    pack: PackValue,
    shape: ListValue,
    data: ListValue,
    layout: String,
    rows: usize,
    cols: usize,
}

fn expect_tensor(value: &Value, span: crate::lang::span::Span) -> Result<TensorValue, RuntimeError> {
    let pack = match value {
        Value::Pack(pack) => pack.clone(),
        other => return Err(type_mismatch_detail("tensor pack", other, span)),
    };
    let shape_value = pack.fields.get("형상").ok_or(RuntimeError::TypeMismatch {
        expected: "tensor shape",
        span,
    })?;
    let (shape, rows, cols) = expect_tensor_shape(shape_value, span)?;
    let data_value = pack.fields.get("자료").ok_or(RuntimeError::TypeMismatch {
        expected: "tensor data",
        span,
    })?;
    let data = match data_value {
        Value::List(list) => list.clone(),
        other => return Err(type_mismatch_detail("list", other, span)),
    };
    let layout_value = pack.fields.get("배치").ok_or(RuntimeError::TypeMismatch {
        expected: "tensor layout",
        span,
    })?;
    let layout = match layout_value {
        Value::Str(text) => text.clone(),
        other => return Err(type_mismatch_detail("string", other, span)),
    };
    Ok(TensorValue {
        pack,
        shape,
        data,
        layout,
        rows,
        cols,
    })
}

fn expect_tensor_shape(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<(ListValue, usize, usize), RuntimeError> {
    let shape = match value {
        Value::List(list) => list.clone(),
        other => return Err(type_mismatch_detail("list", other, span)),
    };
    if shape.items.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "shape list",
            span,
        });
    }
    let rows = require_nonnegative(expect_int(&shape.items[0], span)?, span)?;
    let cols = require_nonnegative(expect_int(&shape.items[1], span)?, span)?;
    Ok((shape, rows, cols))
}

fn expect_tensor_and_indices(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(TensorValue, usize, usize), RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "tensor, row, col",
            span,
        });
    }
    let tensor = expect_tensor(&values[0], span)?;
    let row = require_nonnegative(expect_int(&values[1], span)?, span)?;
    let col = require_nonnegative(expect_int(&values[2], span)?, span)?;
    Ok((tensor, row, col))
}

fn expect_tensor_index_value(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(TensorValue, usize, usize, Value), RuntimeError> {
    if values.len() != 4 {
        return Err(RuntimeError::TypeMismatch {
            expected: "tensor, row, col, value",
            span,
        });
    }
    let tensor = expect_tensor(&values[0], span)?;
    let row = require_nonnegative(expect_int(&values[1], span)?, span)?;
    let col = require_nonnegative(expect_int(&values[2], span)?, span)?;
    Ok((tensor, row, col, values[3].clone()))
}

fn expect_list_and_func(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(ListValue, Callable), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list, function",
            span,
        });
    }
    let list = match &values[0] {
        Value::List(list) => list.clone(),
        value => return Err(type_mismatch_detail("list", value, span)),
    };
    let callable = expect_callable(&values[1], span)?;
    Ok((list, callable))
}

fn expect_list_reduce(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(ListValue, Value, Callable), RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list, initial, function",
            span,
        });
    }
    let list = match &values[0] {
        Value::List(list) => list.clone(),
        value => return Err(type_mismatch_detail("list", value, span)),
    };
    let callable = expect_callable(&values[2], span)?;
    Ok((list, values[1].clone(), callable))
}

#[allow(dead_code)]
fn expect_list_and_delim(values: &[Value], span: crate::lang::span::Span) -> Result<(ListValue, String), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list, delimiter",
            span,
        });
    }
    let list = match &values[0] {
        Value::List(list) => list.clone(),
        value => return Err(type_mismatch_detail("list", value, span)),
    };
    let delim = match &values[1] {
        Value::Str(text) => text.clone(),
        value => return Err(type_mismatch_detail("string delimiter", value, span)),
    };
    Ok((list, delim))
}

fn expect_callable(value: &Value, span: crate::lang::span::Span) -> Result<Callable, RuntimeError> {
    match value {
        Value::Str(text) => {
            let trimmed = text.strip_prefix('#').unwrap_or(text);
            Ok(Callable::Name(trimmed.to_string()))
        }
        Value::Lambda(lambda) => Ok(Callable::Lambda(lambda.clone())),
        _ => Err(type_mismatch_detail("function or seed literal", value, span)),
    }
}

fn expect_two_ints(values: &[Value], span: crate::lang::span::Span) -> Result<(i64, i64), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "two integers",
            span,
        });
    }
    Ok((expect_int(&values[0], span)?, expect_int(&values[1], span)?))
}

fn expect_int(value: &Value, span: crate::lang::span::Span) -> Result<i64, RuntimeError> {
    let qty = match value {
        Value::Num(qty) => qty,
        _ => return Err(type_mismatch_detail("number", value, span)),
    };
    if !qty.dim.is_dimensionless() {
        return Err(RuntimeError::UnitMismatch { span });
    }
    let raw = qty.raw.raw();
    if raw & 0xFFFF_FFFF != 0 {
        return Err(type_mismatch_detail("integer", value, span));
    }
    Ok(raw >> 32)
}

fn require_nonnegative(value: i64, span: crate::lang::span::Span) -> Result<usize, RuntimeError> {
    if value < 0 {
        return Err(RuntimeError::TypeMismatch {
            expected: "non-negative integer",
            span,
        });
    }
    usize::try_from(value).map_err(|_| RuntimeError::TypeMismatch {
        expected: "integer range",
        span,
    })
}

fn expect_template_and_text(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(TemplateValue, String), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "template and string",
            span,
        });
    }
    let template = match &values[0] {
        Value::Template(value) => value.clone(),
        _ => {
            return Err(RuntimeError::Template {
                message: "글무늬값이 필요합니다".to_string(),
                span,
            })
        }
    };
    let text = match &values[1] {
        Value::Str(value) => value.clone(),
        value => return Err(type_mismatch_detail("string", value, span)),
    };
    Ok((template, text))
}

#[derive(Default)]
struct FormulaTransformOptions {
    var_name: Option<String>,
    order: Option<i64>,
    include_const: Option<bool>,
}

fn expect_formula_transform(
    values: &[Value],
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<(crate::core::value::MathValue, FormulaTransformOptions), RuntimeError> {
    if values.is_empty() || values.len() > 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "formula[, variable|options]",
            span,
        });
    }
    let formula = match &values[0] {
        Value::Math(value) => value.clone(),
        value => return Err(type_mismatch_detail("formula", value, span)),
    };
    if values.len() == 1 {
        return Ok((formula, FormulaTransformOptions::default()));
    }
    let options = parse_formula_transform_arg(&values[1], label, span)?;
    Ok((formula, options))
}

fn parse_formula_transform_arg(
    value: &Value,
    label: &'static str,
    span: crate::lang::span::Span,
) -> Result<FormulaTransformOptions, RuntimeError> {
    match value {
        Value::Str(_) => Ok(FormulaTransformOptions {
            var_name: Some(parse_formula_var_name(value, label, span)?),
            ..FormulaTransformOptions::default()
        }),
        Value::Pack(pack) => parse_formula_transform_pack(pack, label, span),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "string variable or pack options",
            span,
        }),
    }
}

fn parse_formula_transform_pack(
    pack: &PackValue,
    label: &'static str,
    span: crate::lang::span::Span,
) -> Result<FormulaTransformOptions, RuntimeError> {
    let mut options = FormulaTransformOptions::default();
    for key in pack.fields.keys() {
        match key.as_str() {
            "변수" | "차수" | "상수포함" => {}
            _ => {
                return Err(RuntimeError::FormulaParse {
                    message: format!(
                        "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: {} 옵션을 지원하지 않습니다",
                        key
                    ),
                    span,
                })
            }
        }
    }

    if let Some(value) = pack.fields.get("변수") {
        if !matches!(value, Value::None) {
            options.var_name = Some(parse_formula_var_name(value, label, span)?);
        }
    }
    if let Some(value) = pack.fields.get("차수") {
        if !matches!(value, Value::None) {
            let order = expect_int(value, span)?;
            if order < 1 {
                return Err(RuntimeError::FormulaParse {
                    message: format!("E_CALC_TRANSFORM_BAD_ORDER: {} 차수는 1 이상이어야 합니다", label),
                    span,
                });
            }
            options.order = Some(order);
        }
    }
    if let Some(value) = pack.fields.get("상수포함") {
        if !matches!(value, Value::None) {
            let include = match value {
                Value::Bool(flag) => *flag,
                other => return Err(type_mismatch_detail("boolean", other, span)),
            };
            options.include_const = Some(include);
        }
    }
    Ok(options)
}

fn parse_formula_var_name(
    value: &Value,
    label: &'static str,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let raw = match value {
        Value::Str(text) => text.clone(),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                expected: "string variable name",
                span,
            })
        }
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(RuntimeError::FormulaParse {
            message: format!("{} 변수 이름이 비어 있습니다", label),
            span,
        });
    }
    Ok(trimmed.trim_start_matches('#').to_string())
}

fn transform_formula_value(
    math: crate::core::value::MathValue,
    options: FormulaTransformOptions,
    call_name: &'static str,
    span: crate::lang::span::Span,
) -> Result<crate::core::value::MathValue, RuntimeError> {
    let Some(dialect) = FormulaDialect::from_tag(&math.dialect) else {
        return Err(RuntimeError::FormulaParse {
            message: format!("알 수 없는 수식 방언: {}", math.dialect),
            span,
        });
    };
    if dialect != FormulaDialect::Ascii {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "{}는 #ascii 수식만 지원합니다",
                if call_name == "diff" { "미분하기" } else { "적분하기" }
            ),
            span,
        });
    }

    let analysis = analyze_formula(&math.body, dialect).map_err(|err| map_formula_error(err, span))?;
    let mut vars = analysis.vars.clone();
    if let Some(assign) = &analysis.assign_name {
        vars.remove(assign);
    }
    let var_name = match options.var_name {
        Some(name) => {
            if !vars.contains(&name) {
                return Err(RuntimeError::FormulaParse {
                    message: format!(
                        "E_CALC_FREEVAR_NOT_FOUND: {} 변수 '{}'가 수식에 없습니다",
                        if call_name == "diff" { "미분하기" } else { "적분하기" },
                        name
                    ),
                    span,
                });
            }
            name
        }
        None => infer_single_var(&vars, call_name, span)?,
    };
    ensure_formula_ident(&var_name, dialect, call_name, span)?;

    if call_name == "diff" && options.include_const.is_some() {
        return Err(RuntimeError::FormulaParse {
            message: "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: 미분하기는 상수포함을 지원하지 않습니다"
                .to_string(),
            span,
        });
    }
    if call_name == "int" && options.order.is_some() {
        return Err(RuntimeError::FormulaParse {
            message: "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: 적분하기는 차수를 지원하지 않습니다"
                .to_string(),
            span,
        });
    }

    let mut expr = if call_name == "diff" {
        if let Some(order) = options.order {
            format!("{}({}, {}, {})", call_name, analysis.expr_text, var_name, order)
        } else {
            format!("{}({}, {})", call_name, analysis.expr_text, var_name)
        }
    } else {
        format!("{}({}, {})", call_name, analysis.expr_text, var_name)
    };
    if call_name == "int" && options.include_const.unwrap_or(false) {
        expr = format!("{} + C", expr);
    }
    let mut body = expr;
    if let Some(assign) = analysis.assign_name {
        body = format!("{} = {}", assign, body);
    }
    let formatted = format_formula_body(&body, dialect).map_err(|err| map_formula_error(err, span))?;
    Ok(crate::core::value::MathValue {
        dialect: math.dialect,
        body: formatted,
    })
}

fn infer_single_var(
    vars: &BTreeSet<String>,
    call_name: &'static str,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    match vars.len() {
        1 => Ok(vars.iter().next().unwrap().to_string()),
        0 => Err(RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_FREEVAR_AMBIGUOUS: {}는 변수 이름을 지정해야 합니다",
                if call_name == "diff" { "미분하기" } else { "적분하기" }
            ),
            span,
        }),
        _ => Err(RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_FREEVAR_AMBIGUOUS: {} 변수 이름이 여러 개입니다",
                if call_name == "diff" { "미분하기" } else { "적분하기" }
            ),
            span,
        }),
    }
}

fn ensure_formula_ident(
    name: &str,
    dialect: FormulaDialect,
    call_name: &'static str,
    span: crate::lang::span::Span,
) -> Result<(), RuntimeError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "{} 변수 이름이 비어 있습니다",
                if call_name == "diff" { "미분하기" } else { "적분하기" }
            ),
            span,
        });
    };
    if !first.is_ascii_alphabetic() {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "{} 변수 이름이 올바르지 않습니다: {}",
                if call_name == "diff" { "미분하기" } else { "적분하기" },
                name
            ),
            span,
        });
    }
    let valid = match dialect {
        FormulaDialect::Ascii => chars.all(|ch| ch.is_ascii_alphanumeric()),
        FormulaDialect::Ascii1 => chars.all(|ch| ch.is_ascii_digit()),
    };
    if !valid {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "{} 변수 이름이 올바르지 않습니다: {}",
                if call_name == "diff" { "미분하기" } else { "적분하기" },
                name
            ),
            span,
        });
    }
    Ok(())
}

struct TetrisArgs {
    board: String,
    width: usize,
    height: usize,
    piece_id: i32,
    rotation: i32,
    x: i32,
    y: i32,
}

struct TetrisCellArgs {
    board: String,
    width: usize,
    height: usize,
    x: i32,
    y: i32,
}

struct TetrisDrawListArgs {
    board: String,
    width: usize,
    height: usize,
    origin_x: i32,
    origin_y: i32,
    cell: i32,
}

struct TetrisBlockArgs {
    piece_id: i32,
    rotation: i32,
    index: usize,
}

fn expect_tetris_args(values: &[Value], span: crate::lang::span::Span) -> Result<TetrisArgs, RuntimeError> {
    if values.len() != 7 {
        return Err(RuntimeError::TypeMismatch {
            expected: "board, width, height, piece_id, rot, x, y",
            span,
        });
    }
    let board = match &values[0] {
        Value::Str(text) => text.clone(),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                expected: "board string",
                span,
            })
        }
    };
    let width = require_nonnegative(expect_int(&values[1], span)?, span)?;
    let height = require_nonnegative(expect_int(&values[2], span)?, span)?;
    let piece_id = i32::try_from(expect_int(&values[3], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "piece id",
        span,
    })?;
    let rotation = i32::try_from(expect_int(&values[4], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "rotation",
        span,
    })?;
    let x = i32::try_from(expect_int(&values[5], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "x",
        span,
    })?;
    let y = i32::try_from(expect_int(&values[6], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "y",
        span,
    })?;
    Ok(TetrisArgs {
        board,
        width,
        height,
        piece_id,
        rotation,
        x,
        y,
    })
}

fn expect_tetris_cell_args(values: &[Value], span: crate::lang::span::Span) -> Result<TetrisCellArgs, RuntimeError> {
    if values.len() != 5 {
        return Err(RuntimeError::TypeMismatch {
            expected: "board, width, height, x, y",
            span,
        });
    }
    let board = match &values[0] {
        Value::Str(text) => text.clone(),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                expected: "board string",
                span,
            })
        }
    };
    let width = require_nonnegative(expect_int(&values[1], span)?, span)?;
    let height = require_nonnegative(expect_int(&values[2], span)?, span)?;
    let x = i32::try_from(expect_int(&values[3], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "x",
        span,
    })?;
    let y = i32::try_from(expect_int(&values[4], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "y",
        span,
    })?;
    Ok(TetrisCellArgs {
        board,
        width,
        height,
        x,
        y,
    })
}

fn expect_tetris_drawlist_args(values: &[Value], span: crate::lang::span::Span) -> Result<TetrisDrawListArgs, RuntimeError> {
    if values.len() != 6 {
        return Err(RuntimeError::TypeMismatch {
            expected: "board, width, height, origin_x, origin_y, cell",
            span,
        });
    }
    let board = match &values[0] {
        Value::Str(text) => text.clone(),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                expected: "board string",
                span,
            })
        }
    };
    let width = require_nonnegative(expect_int(&values[1], span)?, span)?;
    let height = require_nonnegative(expect_int(&values[2], span)?, span)?;
    let origin_x = i32::try_from(expect_int(&values[3], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "origin_x",
        span,
    })?;
    let origin_y = i32::try_from(expect_int(&values[4], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "origin_y",
        span,
    })?;
    let cell = require_nonnegative(expect_int(&values[5], span)?, span)?;
    let cell = i32::try_from(cell).map_err(|_| RuntimeError::TypeMismatch {
        expected: "cell",
        span,
    })?;
    Ok(TetrisDrawListArgs {
        board,
        width,
        height,
        origin_x,
        origin_y,
        cell,
    })
}

fn expect_tetris_block_args(values: &[Value], span: crate::lang::span::Span) -> Result<TetrisBlockArgs, RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "piece_id, rot, index",
            span,
        });
    }
    let piece_id = i32::try_from(expect_int(&values[0], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "piece id",
        span,
    })?;
    let rotation = i32::try_from(expect_int(&values[1], span)?).map_err(|_| RuntimeError::TypeMismatch {
        expected: "rotation",
        span,
    })?;
    let index = require_nonnegative(expect_int(&values[2], span)?, span)?;
    Ok(TetrisBlockArgs {
        piece_id,
        rotation,
        index,
    })
}

fn tetris_can_place(args: &TetrisArgs, span: crate::lang::span::Span) -> Result<bool, RuntimeError> {
    validate_board(args, span)?;
    tetris_can_place_at(args, args.x, args.y)
}

fn tetris_can_place_at(args: &TetrisArgs, x: i32, y: i32) -> Result<bool, RuntimeError> {
    let blocks = piece_blocks(args.piece_id, args.rotation);
    let bytes = args.board.as_bytes();
    for (dx, dy) in blocks {
        let cx = x + dx;
        let cy = y + dy;
        if cx < 0 || cy < 0 {
            return Ok(false);
        }
        let ux = cx as usize;
        let uy = cy as usize;
        if ux >= args.width || uy >= args.height {
            return Ok(false);
        }
        let idx = uy * args.width + ux;
        if idx >= bytes.len() {
            return Ok(false);
        }
        if bytes[idx] != b'.' {
            return Ok(false);
        }
    }
    Ok(true)
}

fn tetris_lock(args: &TetrisArgs, span: crate::lang::span::Span) -> Result<(String, i64, bool), RuntimeError> {
    validate_board(args, span)?;
    if !tetris_can_place_at(args, args.x, args.y)? {
        return Ok((args.board.clone(), 0, false));
    }
    let symbol = piece_symbol_char(args.piece_id);
    let mut cells = args.board.as_bytes().to_vec();
    for (dx, dy) in piece_blocks(args.piece_id, args.rotation) {
        let cx = (args.x + dx) as usize;
        let cy = (args.y + dy) as usize;
        let idx = cy * args.width + cx;
        if idx < cells.len() {
            cells[idx] = symbol;
        }
    }

    let mut new_rows = Vec::new();
    let mut cleared = 0i64;
    for row in 0..args.height {
        let start = row * args.width;
        let end = start + args.width;
        let full = cells[start..end].iter().all(|ch| *ch != b'.');
        if full {
            cleared += 1;
        } else {
            new_rows.extend_from_slice(&cells[start..end]);
        }
    }
    let mut result = Vec::new();
    let empty_row = vec![b'.'; args.width];
    for _ in 0..cleared {
        result.extend_from_slice(&empty_row);
    }
    result.extend_from_slice(&new_rows);
    let board = String::from_utf8(result).map_err(|_| RuntimeError::TypeMismatch {
        expected: "board string",
        span,
    })?;
    Ok((board, cleared, true))
}

fn tetris_board_cell(args: &TetrisCellArgs, span: crate::lang::span::Span) -> Result<(String, String), RuntimeError> {
    validate_board_dims(&args.board, args.width, args.height, span)?;
    if args.x < 0 || args.y < 0 {
        return Ok((default_board_uri(), empty_board_color()));
    }
    let ux = args.x as usize;
    let uy = args.y as usize;
    if ux >= args.width || uy >= args.height {
        return Ok((default_board_uri(), empty_board_color()));
    }
    let idx = uy * args.width + ux;
    let ch = args.board.as_bytes().get(idx).copied().unwrap_or(b'.');
    if let Some(uri) = board_cell_uri(ch) {
        Ok((uri.to_string(), filled_board_color()))
    } else {
        Ok((default_board_uri(), empty_board_color()))
    }
}

fn tetris_board_drawlist(args: &TetrisDrawListArgs, span: crate::lang::span::Span) -> Result<ListValue, RuntimeError> {
    validate_board_dims(&args.board, args.width, args.height, span)?;
    let mut items = Vec::with_capacity(args.width.saturating_mul(args.height));
    let cell = i64::from(args.cell);
    let origin_x = i64::from(args.origin_x);
    let origin_y = i64::from(args.origin_y);
    let bytes = args.board.as_bytes();
    for y in 0..args.height {
        for x in 0..args.width {
            let idx = y * args.width + x;
            let ch = bytes.get(idx).copied().unwrap_or(b'.');
            let (uri, color) = if let Some(uri) = board_cell_uri(ch) {
                (uri.to_string(), filled_board_color())
            } else {
                (default_board_uri(), empty_board_color())
            };
            let px = origin_x + (x as i64).saturating_mul(cell);
            let py = origin_y + (y as i64).saturating_mul(cell);
            let mut fields = BTreeMap::new();
            fields.insert("id".to_string(), Value::Str(format!("보드셀_{}_{}", y, x)));
            fields.insert("결".to_string(), Value::Str("#보개/2D.Sprite".to_string()));
            fields.insert(
                "x".to_string(),
                Value::Num(Quantity::new(Fixed64::from_int(px), UnitDim::zero())),
            );
            fields.insert(
                "y".to_string(),
                Value::Num(Quantity::new(Fixed64::from_int(py), UnitDim::zero())),
            );
            fields.insert(
                "w".to_string(),
                Value::Num(Quantity::new(Fixed64::from_int(cell), UnitDim::zero())),
            );
            fields.insert(
                "h".to_string(),
                Value::Num(Quantity::new(Fixed64::from_int(cell), UnitDim::zero())),
            );
            fields.insert("uri".to_string(), Value::Str(uri));
            fields.insert("색".to_string(), Value::Str(color));
            items.push(Value::Pack(PackValue { fields }));
        }
    }
    Ok(ListValue { items })
}

fn tetris_piece_block(args: &TetrisBlockArgs) -> (i32, i32) {
    let blocks = piece_blocks(args.piece_id, args.rotation);
    blocks.get(args.index).copied().unwrap_or((0, 0))
}

fn validate_board(args: &TetrisArgs, span: crate::lang::span::Span) -> Result<(), RuntimeError> {
    validate_board_dims(&args.board, args.width, args.height, span)
}

fn validate_board_dims(board: &str, width: usize, height: usize, span: crate::lang::span::Span) -> Result<(), RuntimeError> {
    let expected = width
        .checked_mul(height)
        .ok_or(RuntimeError::TypeMismatch {
            expected: "board size",
            span,
        })?;
    if board.len() != expected {
        return Err(RuntimeError::TypeMismatch {
            expected: "board size",
            span,
        });
    }
    Ok(())
}

fn piece_symbol_char(piece_id: i32) -> u8 {
    match piece_id {
        0 => b'I',
        1 => b'O',
        2 => b'T',
        3 => b'S',
        4 => b'Z',
        5 => b'J',
        6 => b'L',
        _ => b'#',
    }
}

fn board_cell_uri(ch: u8) -> Option<&'static str> {
    match ch {
        b'I' => Some("sym:tetris.I"),
        b'O' => Some("sym:tetris.O"),
        b'T' => Some("sym:tetris.T"),
        b'S' => Some("sym:tetris.S"),
        b'Z' => Some("sym:tetris.Z"),
        b'J' => Some("sym:tetris.J"),
        b'L' => Some("sym:tetris.L"),
        b'#' => Some("sym:tetris.I"),
        _ => None,
    }
}

fn default_board_uri() -> String {
    "sym:tetris.I".to_string()
}

fn filled_board_color() -> String {
    "#ffffffff".to_string()
}

fn empty_board_color() -> String {
    "#00000000".to_string()
}

fn piece_blocks(piece_id: i32, rotation: i32) -> [(i32, i32); 4] {
    let rot = ((rotation % 4) + 4) % 4;
    match piece_id {
        0 => match rot {
            0 => [(0, 1), (1, 1), (2, 1), (3, 1)],
            1 => [(2, 0), (2, 1), (2, 2), (2, 3)],
            2 => [(0, 2), (1, 2), (2, 2), (3, 2)],
            _ => [(1, 0), (1, 1), (1, 2), (1, 3)],
        },
        1 => [(1, 0), (2, 0), (1, 1), (2, 1)],
        2 => match rot {
            0 => [(1, 0), (0, 1), (1, 1), (2, 1)],
            1 => [(1, 0), (1, 1), (2, 1), (1, 2)],
            2 => [(0, 1), (1, 1), (2, 1), (1, 2)],
            _ => [(1, 0), (0, 1), (1, 1), (1, 2)],
        },
        3 => match rot {
            0 => [(1, 0), (2, 0), (0, 1), (1, 1)],
            1 => [(1, 0), (1, 1), (2, 1), (2, 2)],
            2 => [(1, 1), (2, 1), (0, 2), (1, 2)],
            _ => [(0, 0), (0, 1), (1, 1), (1, 2)],
        },
        4 => match rot {
            0 => [(0, 0), (1, 0), (1, 1), (2, 1)],
            1 => [(2, 0), (1, 1), (2, 1), (1, 2)],
            2 => [(0, 1), (1, 1), (1, 2), (2, 2)],
            _ => [(1, 0), (0, 1), (1, 1), (0, 2)],
        },
        5 => match rot {
            0 => [(0, 0), (0, 1), (1, 1), (2, 1)],
            1 => [(1, 0), (2, 0), (1, 1), (1, 2)],
            2 => [(0, 1), (1, 1), (2, 1), (2, 2)],
            _ => [(1, 0), (1, 1), (0, 2), (1, 2)],
        },
        _ => match rot {
            0 => [(2, 0), (0, 1), (1, 1), (2, 1)],
            1 => [(1, 0), (1, 1), (1, 2), (2, 2)],
            2 => [(0, 1), (1, 1), (2, 1), (0, 2)],
            _ => [(0, 0), (1, 0), (1, 1), (1, 2)],
        },
    }
}

fn ensure_same_dim(a: &Quantity, b: &Quantity, span: crate::lang::span::Span) -> Result<(), RuntimeError> {
    if a.dim != b.dim {
        return Err(RuntimeError::UnitMismatch { span });
    }
    Ok(())
}

fn ensure_dimensionless(value: &Quantity, span: crate::lang::span::Span) -> Result<(), RuntimeError> {
    if !value.dim.is_dimensionless() {
        return Err(RuntimeError::UnitMismatch { span });
    }
    Ok(())
}

fn clamp_fixed64(value: Fixed64, min: Fixed64, max: Fixed64) -> Fixed64 {
    if value.raw() < min.raw() {
        min
    } else if value.raw() > max.raw() {
        max
    } else {
        value
    }
}

fn nuance_weight(tag: &str) -> Fixed64 {
    match tag {
        "매우" => Fixed64::from_ratio(9, 10),
        "꽤" => Fixed64::from_ratio(7, 10),
        "조금" => Fixed64::from_ratio(1, 4),
        "약간" => Fixed64::from_ratio(3, 20),
        "거의" => Fixed64::from_ratio(1, 20),
        _ => Fixed64::one(),
    }
}

fn fixed64_floor(value: Fixed64) -> Fixed64 {
    let int_part = value.raw() >> Fixed64::SCALE_BITS;
    Fixed64::from_int(int_part)
}

fn fixed64_ceil(value: Fixed64) -> Fixed64 {
    let raw = value.raw();
    let int_part = raw >> Fixed64::SCALE_BITS;
    if (raw & (Fixed64::SCALE - 1)) == 0 {
        Fixed64::from_int(int_part)
    } else {
        Fixed64::from_int(int_part.saturating_add(1))
    }
}

fn fixed64_round_even(value: Fixed64) -> Fixed64 {
    let raw = value.raw() as i128;
    if raw == 0 {
        return Fixed64::from_int(0);
    }
    let sign = if raw < 0 { -1 } else { 1 };
    let abs = raw.abs();
    let frac_mask = (1_i128 << Fixed64::SCALE_BITS) - 1;
    let int_part = abs >> Fixed64::SCALE_BITS;
    let frac = abs & frac_mask;
    let half = 1_i128 << (Fixed64::SCALE_BITS - 1);
    let mut rounded = int_part;
    if frac > half || (frac == half && (int_part & 1) != 0) {
        rounded += 1;
    }
    Fixed64::from_int((rounded * sign) as i64)
}

fn ensure_angle_dim(dim: UnitDim, span: crate::lang::span::Span) -> Result<(), RuntimeError> {
    if dim.length != 0 || dim.time != 0 || dim.mass != 0 || dim.pixel != 0 {
        return Err(RuntimeError::UnitMismatch { span });
    }
    if dim.angle != 0 && dim.angle != 1 {
        return Err(RuntimeError::UnitMismatch { span });
    }
    Ok(())
}

fn sqrt_dim(dim: UnitDim, span: crate::lang::span::Span) -> Result<UnitDim, RuntimeError> {
    if dim.length % 2 != 0
        || dim.time % 2 != 0
        || dim.mass % 2 != 0
        || dim.angle % 2 != 0
        || dim.pixel % 2 != 0
    {
        return Err(RuntimeError::UnitMismatch { span });
    }
    Ok(UnitDim {
        length: dim.length / 2,
        time: dim.time / 2,
        mass: dim.mass / 2,
        angle: dim.angle / 2,
        pixel: dim.pixel / 2,
    })
}

fn is_truthy(value: &Value, span: crate::lang::span::Span) -> Result<bool, RuntimeError> {
    match value {
        Value::Bool(flag) => Ok(*flag),
        Value::Num(qty) => {
            if !qty.dim.is_dimensionless() {
                return Err(RuntimeError::TypeMismatch {
                    expected: "dimensionless number",
                    span,
                });
            }
            Ok(qty.raw.raw() != 0)
        }
        Value::None => Ok(false),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "condition",
            span,
        }),
    }
}
