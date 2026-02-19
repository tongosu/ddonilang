use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

use ddonirang_core::{
    ArithmeticFaultKind, ExprTrace, FaultContext, Fixed64, InputSnapshot, ResourceHandle, Signal,
    SourceSpan, UnitDim, UnitError, UnitValue, unit_spec_from_symbol,
};
use libm;
use ddonirang_core::signals::DiagEvent;
use crate::gate0_registry;
use crate::preprocess::{preprocess_source_for_parse, split_file_meta, FileMeta};
use ddonirang_core::platform::{
    EntityId, NuriWorld, Origin, Patch, PatchOp, ResourceMapEntry, ResourceValue,
};
use ddonirang_lang::{
    canonicalize, parse_with_mode, AtSuffix, Body, CanonProgram, Expr, ExprKind, Formula,
    FormulaDialect, Literal, ParamPin, ParseError, ParseMode, SeedDef, SeedKind, Stmt,
    TemplateFormat, TemplatePart, TypeRef,
};
use ddonirang_lang::runtime::{
    input_just_pressed, input_pressed, list_add, list_len, list_nth, list_remove, list_set,
    map_get, map_key_canon,
    string_concat, string_contains, string_ends, string_join, string_len, string_split, string_starts,
    string_to_number, InputState, LambdaValue, MapEntry, RuntimeError, Value,
};

static LAMBDA_SEQ: AtomicU64 = AtomicU64::new(1);
static DEFAULT_PARSE_MODE: AtomicU64 = AtomicU64::new(1);

fn next_lambda_id() -> u64 {
    LAMBDA_SEQ.fetch_add(1, Ordering::Relaxed)
}

fn encode_parse_mode(mode: ParseMode) -> u64 {
    match mode {
        ParseMode::Strict => 1,
    }
}

fn decode_parse_mode(value: u64) -> ParseMode {
    let _ = value;
    ParseMode::Strict
}

pub fn default_parse_mode() -> ParseMode {
    decode_parse_mode(DEFAULT_PARSE_MODE.load(Ordering::Relaxed))
}

pub fn set_default_parse_mode(mode: ParseMode) {
    DEFAULT_PARSE_MODE.store(encode_parse_mode(mode), Ordering::Relaxed);
}

#[derive(Clone)]
pub struct DdnProgram {
    #[allow(dead_code)]
    program: CanonProgram,
    functions: HashMap<String, SeedDef>,
    #[allow(dead_code)]
    file_meta: FileMeta,
}

impl DdnProgram {
    pub fn from_source(source: &str, file_path: &str) -> Result<Self, String> {
        Self::from_source_with_mode(source, file_path, default_parse_mode())
    }

    pub fn from_source_with_mode(
        source: &str,
        file_path: &str,
        mode: ParseMode,
    ) -> Result<Self, String> {
        let meta_parse = split_file_meta(source);
        let cleaned = preprocess_source_for_parse(&meta_parse.stripped)?;
        let mut program = parse_with_mode(&cleaned, file_path, mode)
            .map_err(|e| format_parse_error(&cleaned, &e))?;
        let _report =
            canonicalize(&mut program).map_err(|e| format_parse_error(&cleaned, &e))?;
        let mut functions = HashMap::new();
        let tails = ["기", "고", "면"];
        for item in &program.items {
            let ddonirang_lang::TopLevelItem::SeedDef(seed) = item;
            let name = seed.canonical_name.clone();
            functions.insert(name.clone(), seed.clone());
            for tail in tails {
                let alias = format!("{name}{tail}");
                if !functions.contains_key(&alias) {
                    functions.insert(alias, seed.clone());
                }
            }
        }
        Ok(Self {
            program,
            functions,
            file_meta: meta_parse.meta,
        })
    }
}

pub struct DdnRunner {
    program: DdnProgram,
    update_name: String,
    prev_keys_pressed: u64,
}

pub struct DdnRunOutput {
    pub patch: Patch,
    pub resources: HashMap<String, Value>,
}

impl DdnRunner {
    pub fn new(program: DdnProgram, update_name: &str) -> Self {
        Self {
            program,
            update_name: update_name.to_string(),
            prev_keys_pressed: 0,
        }
    }

    pub fn run_update(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
    ) -> Result<DdnRunOutput, String> {
        let input_state = InputState::new(input.keys_pressed, self.prev_keys_pressed);
        self.prev_keys_pressed = input.keys_pressed;

        let mut ctx = EvalContext::new(
            &self.program,
            world,
            defaults,
            input_state,
            input.rng_seed,
            input.tick_id,
        );
        ctx.resources
            .insert("입력키".to_string(), Value::String(input.last_key_name.clone()));
        let update = if let Some(update) = ctx.program.functions.get(&self.update_name) {
            update
        } else if self.update_name == "매마디" {
            ctx.program
                .functions
                .get("매틱")
                .ok_or_else(|| format!("업데이트 함수 '{}'를 찾을 수 없습니다", self.update_name))?
        } else {
            return Err(format!("업데이트 함수 '{}'를 찾을 수 없습니다", self.update_name));
        };
        ctx.eval_seed(update, Vec::new())
            .map_err(|err| err.to_string())?;
        Ok(DdnRunOutput {
            patch: Patch {
                ops: ctx.patch_ops,
                origin: Origin::system("ddn"),
            },
            resources: ctx.resources,
        })
    }

    #[allow(dead_code)]
    pub fn reset_transient_state(&mut self) {
        self.prev_keys_pressed = 0;
    }
}

struct EvalContext<'a> {
    program: &'a DdnProgram,
    world: &'a NuriWorld,
    defaults: &'a HashMap<String, Value>,
    resources: HashMap<String, Value>,
    input: InputState,
    patch_ops: Vec<PatchOp>,
    guard_rejected: bool,
    aborted: bool,
    current_seed_name: Option<String>,
    rng_state: u64,
    flow_stack: Vec<Option<Value>>,
    tick_id: u64,
    const_scopes: Vec<HashSet<String>>,
}

enum ThunkResult {
    Value(Value),
    Return(Value),
    Break(ddonirang_lang::Span),
}

enum FlowControl {
    Continue,
    Return(Value),
    Break(ddonirang_lang::Span),
}

#[derive(Debug)]
enum EvalError {
    Message(String),
    UnitMismatch { left: UnitDim, right: UnitDim },
    DivisionByZero,
}

#[derive(Debug)]
struct TypeMismatchDetail {
    expected: String,
    actual: String,
}

impl EvalError {
    fn message(&self) -> String {
        match self {
            EvalError::Message(message) => message.clone(),
            EvalError::UnitMismatch { left, right } => {
                format!("단위 차원이 다릅니다: {} vs {}", left.format(), right.format())
            }
            EvalError::DivisionByZero => "0으로 나눌 수 없습니다".to_string(),
        }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl From<String> for EvalError {
    fn from(message: String) -> Self {
        EvalError::Message(message)
    }
}

impl From<UnitError> for EvalError {
    fn from(err: UnitError) -> Self {
        match err {
            UnitError::DimensionMismatch { left, right } => EvalError::UnitMismatch { left, right },
            UnitError::DivisionByZero => EvalError::DivisionByZero,
        }
    }
}

impl From<RuntimeError> for EvalError {
    fn from(err: RuntimeError) -> Self {
        EvalError::Message(runtime_error_message(err))
    }
}

impl<'a> EvalContext<'a> {
    fn new(
        program: &'a DdnProgram,
        world: &'a NuriWorld,
        defaults: &'a HashMap<String, Value>,
        input: InputState,
        rng_seed: u64,
        tick_id: u64,
    ) -> Self {
        Self {
            program,
            world,
            defaults,
            resources: HashMap::new(),
            input,
            patch_ops: Vec::new(),
            guard_rejected: false,
            aborted: false,
            current_seed_name: None,
            rng_state: rng_seed,
            flow_stack: Vec::new(),
            tick_id,
            const_scopes: Vec::new(),
        }
    }

    fn enter_const_scope(&mut self) {
        self.const_scopes.push(HashSet::new());
    }

    fn exit_const_scope(&mut self) {
        self.const_scopes.pop();
    }

    fn declare_const(&mut self, name: &str) {
        if let Some(scope) = self.const_scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn is_const(&self, name: &str) -> bool {
        self.const_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn eval_seed(&mut self, seed: &SeedDef, args: Vec<Value>) -> Result<Value, EvalError> {
        let prev_seed = self.current_seed_name.clone();
        self.current_seed_name = Some(seed.canonical_name.clone());
        self.enter_const_scope();
        let result = (|| {
            if self.aborted {
                return Ok(Value::None);
            }
            let mut locals = HashMap::new();
            if seed.params.len() != args.len() {
                return Err(format!(
                    "함수 '{}' 인자 수 불일치: 기대 {}, 실제 {}",
                    seed.canonical_name,
                    seed.params.len(),
                    args.len()
                )
                .into());
            }
            for (param, arg) in seed.params.iter().zip(args) {
                self.check_param_type(param, &arg)?;
                locals.insert(param.pin_name.clone(), arg);
            }
            let Some(body) = &seed.body else {
                return Ok(Value::None);
            };
            let out = self.eval_body(&mut locals, body)?;
            let out_value = match out {
                FlowControl::Continue => None,
                FlowControl::Return(value) => Some(value),
                FlowControl::Break(_) => {
                    return Err("멈추기는 반복 안에서만 사용할 수 있습니다".to_string().into())
                }
            };
            if matches!(seed.seed_kind, SeedKind::Umjikssi) {
                Ok(Value::None)
            } else {
                Ok(out_value.unwrap_or(Value::None))
            }
        })();
        self.exit_const_scope();
        self.current_seed_name = prev_seed;
        result
    }

    fn check_param_type(&self, param: &ParamPin, value: &Value) -> Result<(), EvalError> {
        if matches!(param.type_ref, TypeRef::Infer) {
            return Ok(());
        }
        if param.optional && matches!(value, Value::None) {
            return Ok(());
        }
        match check_value_type(value, &param.type_ref) {
            Ok(()) => Ok(()),
            Err(detail) => {
                let expected = if param.optional {
                    format!("{}?", detail.expected)
                } else {
                    detail.expected
                };
                Err(type_mismatch_error(&param.pin_name, &expected, &detail.actual))
            }
        }
    }

    fn eval_body(
        &mut self,
        locals: &mut HashMap<String, Value>,
        body: &Body,
    ) -> Result<FlowControl, EvalError> {
        if self.aborted {
            return Ok(FlowControl::Continue);
        }
        for stmt in &body.stmts {
            if self.aborted {
                return Ok(FlowControl::Continue);
            }
            match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => {}
                other => return Ok(other),
            }
        }
        Ok(FlowControl::Continue)
    }

    fn eval_body_for_value(
        &mut self,
        locals: &mut HashMap<String, Value>,
        body: &Body,
    ) -> Result<Value, EvalError> {
        match self.eval_body_for_value_inner(locals, body)? {
            ThunkResult::Value(v) | ThunkResult::Return(v) => Ok(v),
            ThunkResult::Break(_) => Err("멈추기는 반복 안에서만 사용할 수 있습니다".to_string().into()),
        }
    }

    fn eval_body_for_value_inner(
        &mut self,
        locals: &mut HashMap<String, Value>,
        body: &Body,
    ) -> Result<ThunkResult, EvalError> {
        let mut last = Value::None;
        if self.aborted {
            return Ok(ThunkResult::Value(Value::None));
        }
        for stmt in &body.stmts {
            if self.aborted {
                return Ok(ThunkResult::Value(Value::None));
            }
            match self.eval_stmt_for_value(locals, stmt)? {
                ThunkResult::Return(v) => return Ok(ThunkResult::Return(v)),
                ThunkResult::Break(span) => return Ok(ThunkResult::Break(span)),
                ThunkResult::Value(v) => last = v,
            }
        }
        Ok(ThunkResult::Value(last))
    }

    fn eval_stmt_for_value(
        &mut self,
        locals: &mut HashMap<String, Value>,
        stmt: &Stmt,
    ) -> Result<ThunkResult, EvalError> {
        if self.aborted {
            return Ok(ThunkResult::Value(Value::None));
        }
        match stmt {
            Stmt::DeclBlock { .. } => {
                match self.eval_stmt(locals, stmt)? {
                    FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                    FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                    FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
                }
            }
            Stmt::Mutate { .. } => {
                match self.eval_stmt(locals, stmt)? {
                    FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                    FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                    FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
                }
            }
            Stmt::Expr { expr, .. } => Ok(ThunkResult::Value(self.eval_expr(locals, expr)?)),
            Stmt::MetaBlock { .. } => Ok(ThunkResult::Value(Value::None)),
            Stmt::Pragma { .. } => Ok(ThunkResult::Value(Value::None)),
            Stmt::Return { value, .. } => Ok(ThunkResult::Return(self.eval_expr(locals, value)?)),
            Stmt::If { condition, then_body, else_body, .. } => {
                let cond = self.eval_expr(locals, condition)?;
                if is_truthy(&cond)? {
                    self.eval_body_for_value_inner(locals, then_body)
                } else if let Some(body) = else_body {
                    self.eval_body_for_value_inner(locals, body)
                } else {
                    Ok(ThunkResult::Value(Value::None))
                }
            }
            Stmt::Try { action, body, .. } => {
                let value = self.eval_expr(locals, action)?;
                let prev = locals.insert("그것".to_string(), value);
                let out = self.eval_body_for_value_inner(locals, body);
                if let Some(prev) = prev {
                    locals.insert("그것".to_string(), prev);
                } else {
                    locals.remove("그것");
                }
                out
            }
            Stmt::Choose { branches, else_body, .. } => {
                for branch in branches {
                    let cond = self.eval_expr(locals, &branch.condition)?;
                    if is_truthy(&cond)? {
                        return self.eval_body_for_value_inner(locals, &branch.body);
                    }
                }
                self.eval_body_for_value_inner(locals, else_body)
            }
            Stmt::Repeat { body, .. } => {
                loop {
                    match self.eval_body_for_value_inner(locals, body)? {
                        ThunkResult::Value(_) => {}
                        ThunkResult::Return(value) => return Ok(ThunkResult::Return(value)),
                        ThunkResult::Break(_) => break,
                    }
                }
                Ok(ThunkResult::Value(Value::None))
            }
            Stmt::While { condition, body, .. } => {
                loop {
                    let cond = self.eval_expr(locals, condition)?;
                    if !is_truthy(&cond)? {
                        break;
                    }
                    match self.eval_body_for_value_inner(locals, body)? {
                        ThunkResult::Value(_) => {}
                        ThunkResult::Return(value) => return Ok(ThunkResult::Return(value)),
                        ThunkResult::Break(_) => break,
                    }
                }
                Ok(ThunkResult::Value(Value::None))
            }
            Stmt::ForEach { item, iterable, body, .. } => {
                let iter_value = self.eval_expr(locals, iterable)?;
                let items = match iter_value {
                    Value::List(items) => items,
                    Value::Set(items) => items.into_values().collect(),
                    Value::Map(entries) => entries
                        .into_values()
                        .map(|entry| Value::List(vec![entry.key, entry.value]))
                        .collect(),
                    _ => return Err("순회 대상은 차림/모음/짝맞춤이어야 합니다".to_string().into()),
                };
                let prev = locals.get(item).cloned();
                for value in items {
                    locals.insert(item.clone(), value);
                    match self.eval_body_for_value_inner(locals, body)? {
                        ThunkResult::Value(_) => {}
                        ThunkResult::Return(value) => {
                            if let Some(prev) = prev {
                                locals.insert(item.clone(), prev);
                            } else {
                                locals.remove(item);
                            }
                            return Ok(ThunkResult::Return(value));
                        }
                        ThunkResult::Break(_) => break,
                    }
                }
                if let Some(prev) = prev {
                    locals.insert(item.clone(), prev);
                } else {
                    locals.remove(item);
                }
                Ok(ThunkResult::Value(Value::None))
            }
            Stmt::Break { span, .. } => Ok(ThunkResult::Break(*span)),
            Stmt::Contract { kind, mode, condition, then_body, else_body, .. } => {
                let cond_value = self.eval_expr(locals, condition)?;
                let ok = is_truthy(&cond_value)?;
                match kind {
                    ddonirang_lang::ContractKind::Pre => {
                        if ok {
                            if let Some(body) = then_body {
                                self.eval_body_for_value_inner(locals, body)
                            } else {
                                Ok(ThunkResult::Value(Value::None))
                            }
                        } else {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Return(value) => return Ok(ThunkResult::Return(value)),
                                FlowControl::Break(span) => return Ok(ThunkResult::Break(span)),
                            }
                            self.emit_contract_violation(
                                ddonirang_lang::ContractKind::Pre,
                                *mode,
                                condition,
                                "전제하에 조건이 실패했습니다".to_string(),
                            );
                            Ok(ThunkResult::Value(Value::None))
                        }
                    }
                    ddonirang_lang::ContractKind::Post => {
                        if !ok {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Return(value) => return Ok(ThunkResult::Return(value)),
                                FlowControl::Break(span) => return Ok(ThunkResult::Break(span)),
                            }
                            let cond_value = self.eval_expr(locals, condition)?;
                            if !is_truthy(&cond_value)? {
                                self.emit_contract_violation(
                                    ddonirang_lang::ContractKind::Post,
                                    *mode,
                                    condition,
                                    "보장하고 조건이 실패했습니다".to_string(),
                                );
                                return Ok(ThunkResult::Value(Value::None));
                            }
                        }
                        if let Some(body) = then_body {
                            self.eval_body_for_value_inner(locals, body)
                        } else {
                            Ok(ThunkResult::Value(Value::None))
                        }
                    }
                }
            }
            Stmt::Guard { .. } => {
                match self.eval_stmt(locals, stmt)? {
                    FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                    FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                    FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
                }
            }
        }
    }

    fn eval_stmt(&mut self, locals: &mut HashMap<String, Value>, stmt: &Stmt) -> Result<FlowControl, EvalError> {
        if self.aborted {
            return Ok(FlowControl::Continue);
        }
        match stmt {
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    let value = if let Some(expr) = &item.value {
                        match self.eval_expr(locals, expr) {
                            Ok(val) => val,
                            Err(EvalError::UnitMismatch { left, right }) => {
                                let trace = Some(self.arith_trace(expr, "arith:UNIT_MISMATCH"));
                                let source_span = self.source_span_for_expr(expr);
                                self.emit_unit_mismatch(
                                    left,
                                    right,
                                    Some(format!("var:{}", item.name)),
                                    source_span,
                                    trace,
                                );
                                continue;
                            }
                            Err(EvalError::DivisionByZero) => {
                                let trace = Some(self.arith_trace(expr, "arith:DIV0"));
                                let source_span = self.source_span_for_expr(expr);
                                self.emit_arith_fault(
                                    ArithmeticFaultKind::DivByZero,
                                    Some(format!("var:{}", item.name)),
                                    source_span,
                                    trace,
                                );
                                continue;
                            }
                            Err(err) => return Err(err),
                        }
                    } else if matches!(item.kind, ddonirang_lang::DeclKind::Butbak) {
                        return Err(format!("채비에서 '=' 항목은 초기값이 필요합니다: {}", item.name).into());
                    } else {
                        Value::None
                    };
                    locals.insert(item.name.clone(), value);
                    if matches!(item.kind, ddonirang_lang::DeclKind::Butbak) {
                        self.declare_const(&item.name);
                    }
                }
                Ok(FlowControl::Continue)
            }
            Stmt::Mutate { target, value, .. } => {
                if self.guard_rejected {
                    return Ok(FlowControl::Continue);
                }
                if let ExprKind::Var(name) = &target.kind {
                    if self.is_const(name) {
                        return Err(format!("붙박이는 재대입할 수 없습니다: {}", name).into());
                    }
                }
                let val = match self.eval_expr(locals, value) {
                    Ok(val) => val,
                    Err(EvalError::UnitMismatch { left, right }) => {
                        let target_label = match &target.kind {
                            ExprKind::Var(name) => Some(format!("var:{}", name)),
                            _ => None,
                        };
                        let trace = Some(self.arith_trace(value, "arith:UNIT_MISMATCH"));
                        let source_span = self.source_span_for_expr(value);
                        self.emit_unit_mismatch(left, right, target_label, source_span, trace);
                        return Ok(FlowControl::Continue);
                    }
                    Err(EvalError::DivisionByZero) => {
                        let target_label = match &target.kind {
                            ExprKind::Var(name) => Some(format!("var:{}", name)),
                            _ => None,
                        };
                        let trace = Some(self.arith_trace(value, "arith:DIV0"));
                        let source_span = self.source_span_for_expr(value);
                        self.emit_arith_fault(
                            ArithmeticFaultKind::DivByZero,
                            target_label,
                            source_span,
                            trace,
                        );
                        return Ok(FlowControl::Continue);
                    }
                    Err(err) => return Err(err),
                };
                match &target.kind {
                    ExprKind::Var(name) => {
                        if locals.contains_key(name) || !self.resource_exists(name) {
                            locals.insert(name.clone(), val);
                        } else {
                            match self.set_resource(name, val) {
                                Ok(()) => {}
                                Err(EvalError::UnitMismatch { left, right }) => {
                                    let trace = Some(self.arith_trace(value, "arith:UNIT_MISMATCH"));
                                    let source_span = self.source_span_for_expr(value);
                                    self.emit_unit_mismatch(
                                        left,
                                        right,
                                        Some(format!("resource:{}", name)),
                                        source_span,
                                        trace,
                                    );
                                    return Ok(FlowControl::Continue);
                                }
                                Err(EvalError::DivisionByZero) => {
                                    let trace = Some(self.arith_trace(value, "arith:DIV0"));
                                    let source_span = self.source_span_for_expr(value);
                                    self.emit_arith_fault(
                                        ArithmeticFaultKind::DivByZero,
                                        Some(format!("resource:{}", name)),
                                        source_span,
                                        trace,
                                    );
                                    return Ok(FlowControl::Continue);
                                }
                                Err(err) => return Err(err),
                            }
                        }
                    }
                    _ => return Err("대입 대상은 변수만 지원합니다".to_string().into()),
                }
                Ok(FlowControl::Continue)
            }
            Stmt::Expr { expr, .. } => {
                match self.eval_expr(locals, expr) {
                    Ok(_) => {}
                    Err(EvalError::UnitMismatch { left, right }) => {
                        let trace = Some(self.arith_trace(expr, "arith:UNIT_MISMATCH"));
                        let source_span = self.source_span_for_expr(expr);
                        self.emit_unit_mismatch(left, right, None, source_span, trace);
                    }
                    Err(EvalError::DivisionByZero) => {
                        let trace = Some(self.arith_trace(expr, "arith:DIV0"));
                        let source_span = self.source_span_for_expr(expr);
                        self.emit_arith_fault(
                            ArithmeticFaultKind::DivByZero,
                            None,
                            source_span,
                            trace,
                        );
                    }
                    Err(err) => return Err(err),
                };
                Ok(FlowControl::Continue)
            }
            Stmt::MetaBlock { .. } => Ok(FlowControl::Continue),
            Stmt::Pragma { .. } => Ok(FlowControl::Continue),
            Stmt::Return { value, .. } => {
                let val = self.eval_expr(locals, value)?;
                Ok(FlowControl::Return(val))
            }
            Stmt::If { condition, then_body, else_body, .. } => {
                let cond = self.eval_expr(locals, condition)?;
                if is_truthy(&cond)? {
                    return Ok(self.eval_body(locals, then_body)?);
                } else if let Some(body) = else_body {
                    return Ok(self.eval_body(locals, body)?);
                }
                Ok(FlowControl::Continue)
            }
            Stmt::Try { action, body, .. } => {
                let value = self.eval_expr(locals, action)?;
                let prev = locals.insert("그것".to_string(), value);
                let out = self.eval_body(locals, body)?;
                if let Some(prev) = prev {
                    locals.insert("그것".to_string(), prev);
                } else {
                    locals.remove("그것");
                }
                Ok(out)
            }
            Stmt::Choose { branches, else_body, .. } => {
                for branch in branches {
                    let cond = self.eval_expr(locals, &branch.condition)?;
                    if is_truthy(&cond)? {
                        return Ok(self.eval_body(locals, &branch.body)?);
                    }
                }
                Ok(self.eval_body(locals, else_body)?)
            }
            Stmt::Repeat { body, .. } => {
                loop {
                    match self.eval_body(locals, body)? {
                        FlowControl::Continue => {}
                        FlowControl::Return(value) => return Ok(FlowControl::Return(value)),
                        FlowControl::Break(_) => break,
                    }
                }
                Ok(FlowControl::Continue)
            }
            Stmt::While { condition, body, .. } => {
                loop {
                    let cond = self.eval_expr(locals, condition)?;
                    if !is_truthy(&cond)? {
                        break;
                    }
                    match self.eval_body(locals, body)? {
                        FlowControl::Continue => {}
                        FlowControl::Return(value) => return Ok(FlowControl::Return(value)),
                        FlowControl::Break(_) => break,
                    }
                }
                Ok(FlowControl::Continue)
            }
            Stmt::ForEach { item, iterable, body, .. } => {
                let iter_value = self.eval_expr(locals, iterable)?;
                let items = match iter_value {
                    Value::List(items) => items,
                    Value::Set(items) => items.into_values().collect(),
                    Value::Map(entries) => entries
                        .into_values()
                        .map(|entry| Value::List(vec![entry.key, entry.value]))
                        .collect(),
                    _ => return Err("순회 대상은 차림/모음/짝맞춤이어야 합니다".to_string().into()),
                };
                let prev = locals.get(item).cloned();
                for value in items {
                    locals.insert(item.clone(), value);
                    match self.eval_body(locals, body)? {
                        FlowControl::Continue => {}
                        FlowControl::Return(value) => {
                            if let Some(prev) = prev {
                                locals.insert(item.clone(), prev);
                            } else {
                                locals.remove(item);
                            }
                            return Ok(FlowControl::Return(value));
                        }
                        FlowControl::Break(_) => break,
                    }
                }
                if let Some(prev) = prev {
                    locals.insert(item.clone(), prev);
                } else {
                    locals.remove(item);
                }
                Ok(FlowControl::Continue)
            }
            Stmt::Break { span, .. } => Ok(FlowControl::Break(*span)),
            Stmt::Contract { kind, mode, condition, then_body, else_body, .. } => {
                let cond_value = self.eval_expr(locals, condition)?;
                let ok = is_truthy(&cond_value)?;
                match kind {
                    ddonirang_lang::ContractKind::Pre => {
                        if ok {
                            if let Some(body) = then_body {
                                return Ok(self.eval_body(locals, body)?);
                            }
                            Ok(FlowControl::Continue)
                        } else {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                other => return Ok(other),
                            }
                            self.emit_contract_violation(
                                ddonirang_lang::ContractKind::Pre,
                                *mode,
                                condition,
                                "전제하에 조건이 실패했습니다".to_string(),
                            );
                            Ok(FlowControl::Continue)
                        }
                    }
                    ddonirang_lang::ContractKind::Post => {
                        if !ok {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                other => return Ok(other),
                            }
                            let cond_value = self.eval_expr(locals, condition)?;
                            if !is_truthy(&cond_value)? {
                                self.emit_contract_violation(
                                    ddonirang_lang::ContractKind::Post,
                                    *mode,
                                    condition,
                                    "보장하고 조건이 실패했습니다".to_string(),
                                );
                                return Ok(FlowControl::Continue);
                            }
                        }
                        if let Some(body) = then_body {
                            return Ok(self.eval_body(locals, body)?);
                        }
                        Ok(FlowControl::Continue)
                    }
                }
            }
            Stmt::Guard { condition, body, .. } => {
                let cond_value = self.eval_expr(locals, condition)?;
                if is_truthy(&cond_value)? {
                    self.guard_rejected = true;
                    self.patch_ops.clear();
                    self.emit_guard_violation(condition.id);
                    return Ok(self.eval_body(locals, body)?);
                }
                Ok(FlowControl::Continue)
            }
        }
    }

    fn eval_expr(&mut self, locals: &mut HashMap<String, Value>, expr: &Expr) -> Result<Value, EvalError> {
        match &expr.kind {
            ExprKind::Literal(Literal::Resource(path)) => {
                let handle = gate0_registry::resolve_asset_handle(path)?;
                Ok(Value::ResourceHandle(handle))
            }
            ExprKind::Literal(lit) => Ok(literal_to_value(lit)),
            ExprKind::Var(name) => {
                if let Some(value) = locals.get(name) {
                    Ok(value.clone())
                } else if name == "참" {
                    Ok(Value::Bool(true))
                } else if name == "거짓" {
                    Ok(Value::Bool(false))
                } else if name == "없음" {
                    Ok(Value::None)
                } else if let Some(value) = self.get_resource(name) {
                    Ok(value)
                } else {
                    Err(format!("정의되지 않은 변수: {}", name).into())
                }
            }
            ExprKind::FieldAccess { target, field } => {
                let base = self.eval_expr(locals, target)?;
                match base {
                    Value::Pack(pack) => {
                        let Some(value) = pack.get(field) else {
                            return Err(format!("FATAL:PACK_FIELD_MISSING:{}", field).into());
                        };
                        if matches!(value, Value::None) {
                            return Err(format!("FATAL:PACK_FIELD_NONE:{}", field).into());
                        }
                        Ok(value.clone())
                    }
                    Value::Map(entries) => {
                        let key = Value::String(field.clone());
                        let value = map_get(&entries, &key);
                        if matches!(value, Value::None) {
                            return Err(format!("FATAL:MAP_KEY_MISSING:{}", field).into());
                        }
                        Ok(value)
                    }
                    _ => Err("묶음/짝맞춤 필드 접근만 가능합니다".to_string().into()),
                }
            }
            ExprKind::Call { args, func } => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.eval_expr(locals, &arg.expr)?);
                }
                self.eval_call(func, values)
            }
            ExprKind::SeedLiteral { param, body } => Ok(Value::Lambda(LambdaValue {
                id: next_lambda_id(),
                param: param.clone(),
                body: (*body.clone()),
                captured: locals.clone(),
            })),
            ExprKind::Infix { left, op, right } => {
                let l = self.eval_expr(locals, left)?;
                let r = self.eval_expr(locals, right)?;
                self.eval_infix(op, l, r)
            }
            ExprKind::Suffix { value, at } => {
                let base = self.eval_expr(locals, value)?;
                apply_suffix_value(base, at)
            }
            ExprKind::Thunk(_) => Err("Thunk는 즉시 평가 표지가 필요합니다".to_string().into()),
            ExprKind::Eval { thunk, mode } => {
                let ExprKind::Thunk(body) = &thunk.kind else {
                    return Err("평가 표지는 안은문장에만 붙일 수 있습니다".to_string().into());
                };
                match mode {
                    ddonirang_lang::ThunkEvalMode::Value => {
                        let value = self.eval_body_for_value(locals, body)?;
                        Ok(value)
                    }
                    ddonirang_lang::ThunkEvalMode::Bool => {
                        let value = self.eval_body_for_value(locals, body)?;
                        Ok(Value::Bool(is_truthy(&value)?))
                    }
                    ddonirang_lang::ThunkEvalMode::Not => {
                        let value = self.eval_body_for_value(locals, body)?;
                        Ok(Value::Bool(!is_truthy(&value)?))
                    }
                    ddonirang_lang::ThunkEvalMode::Do => {
                        let _ = self.eval_body(locals, body)?;
                        Ok(Value::None)
                    }
                    ddonirang_lang::ThunkEvalMode::Pipe => {
                        let value = self.eval_body_for_value(locals, body)?;
                        Ok(value)
                    }
                }
            }
            ExprKind::Pipe { stages } => self.eval_pipe(locals, stages),
            ExprKind::FlowValue => {
                if let Some(Some(value)) = self.flow_stack.last() {
                    Ok(value.clone())
                } else {
                    Ok(Value::None)
                }
            }
            ExprKind::Pack { fields } => {
                let mut map = std::collections::BTreeMap::new();
                for (name, value) in fields {
                    if map.contains_key(name) {
                        return Err(format!("묶음 키 '{}'가 중복되었습니다", name).into());
                    }
                    let v = self.eval_expr(locals, value)?;
                    map.insert(name.clone(), v);
                }
                Ok(Value::Pack(map))
            }
            ExprKind::Formula(formula) => Ok(Value::Formula(formula.clone())),
            ExprKind::Template(template) => Ok(Value::Template(template.clone())),
            ExprKind::TemplateRender { template, inject } => {
                let pack = self.eval_inject_fields(locals, inject)?;
                self.eval_call("채우기", vec![Value::Template(template.clone()), pack])
            }
            ExprKind::FormulaEval { formula, inject } => {
                let pack = self.eval_inject_fields(locals, inject)?;
                self.eval_call("풀기", vec![Value::Formula(formula.clone()), pack])
            }
            ExprKind::Nuance { expr, .. } => self.eval_expr(locals, expr),
        }
    }

    fn eval_callable(&mut self, callable: &Value, args: Vec<Value>) -> Result<Value, EvalError> {
        match callable {
            Value::Lambda(lambda) => self.eval_lambda(lambda, &args),
            Value::String(name) => {
                let func = name.trim_start_matches('#');
                self.eval_call(func, args)
            }
            _ => Err("함수/씨앗 인자가 필요합니다".to_string().into()),
        }
    }

    fn eval_lambda(&mut self, lambda: &LambdaValue, args: &[Value]) -> Result<Value, EvalError> {
        if args.len() != 1 {
            return Err("씨앗 인자는 1개여야 합니다".to_string().into());
        }
        let mut locals = lambda.captured.clone();
        locals.insert(lambda.param.clone(), args[0].clone());
        self.eval_expr(&mut locals, &lambda.body)
    }

    fn eval_inject_fields(
        &mut self,
        locals: &mut HashMap<String, Value>,
        inject: &[(String, Expr)],
    ) -> Result<Value, EvalError> {
        let mut map = std::collections::BTreeMap::new();
        for (name, value) in inject {
            if map.contains_key(name) {
                return Err(format!("주입 키 '{}'가 중복되었습니다", name).into());
            }
            let v = self.eval_expr(locals, value)?;
            map.insert(name.clone(), v);
        }
        Ok(Value::Pack(map))
    }

    fn eval_pipe(&mut self, locals: &mut HashMap<String, Value>, stages: &[Expr]) -> Result<Value, EvalError> {
        self.flow_stack.push(None);
        let idx = self.flow_stack.len() - 1;
        for stage in stages {
            let value = self.eval_expr(locals, stage)?;
            if !matches!(value, Value::None) {
                self.flow_stack[idx] = Some(value);
            }
        }
        let out = self.flow_stack.pop().unwrap_or(None);
        Ok(out.unwrap_or(Value::None))
    }

    fn next_rng_u64(&mut self) -> u64 {
        self.rng_state = self.rng_state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.rng_state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_rng_fixed64(&mut self) -> Fixed64 {
        let value = self.next_rng_u64();
        Fixed64::from_raw_i64((value & 0xFFFF_FFFF) as i64)
    }

    fn canonicalize_stdlib_alias(name: &str) -> &str {
        ddonirang_lang::stdlib::canonicalize_stdlib_alias(name)
    }

    fn eval_call(&mut self, func: &str, args: Vec<Value>) -> Result<Value, EvalError> {
        let func = Self::canonicalize_stdlib_alias(func);
        match func {
            "아님" | "아니다" => {
                if args.len() != 1 {
                    return Err("아님은 인자 1개를 받습니다".to_string().into());
                }
                let value = is_truthy(&args[0])?;
                Ok(Value::Bool(!value))
            }
            "그리고" => {
                if args.len() != 2 {
                    return Err("그리고는 인자 2개를 받습니다".to_string().into());
                }
                let left = is_truthy(&args[0])?;
                let right = is_truthy(&args[1])?;
                Ok(Value::Bool(left && right))
            }
            "또는" => {
                if args.len() != 2 {
                    return Err("또는은 인자 2개를 받습니다".to_string().into());
                }
                let left = is_truthy(&args[0])?;
                let right = is_truthy(&args[1])?;
                Ok(Value::Bool(left || right))
            }
            "차림" | "목록" => Ok(Value::List(args)),
            "모음" => {
                let mut items = BTreeMap::new();
                for item in args {
                    let key = value_canon(&item);
                    items.entry(key).or_insert(item);
                }
                Ok(Value::Set(items))
            }
            "짝맞춤" => {
                if args.len() % 2 != 0 {
                    return Err("짝맞춤은 열쇠/값 쌍 인자를 받습니다".to_string().into());
                }
                let mut entries = BTreeMap::new();
                for chunk in args.chunks(2) {
                    let key_value = chunk[0].clone();
                    let entry = MapEntry {
                        key: key_value.clone(),
                        value: chunk[1].clone(),
                    };
                    entries.insert(map_key_canon(&key_value), entry);
                }
                Ok(Value::Map(entries))
            }
            "짝맞춤.값" => {
                if args.len() != 2 {
                    return Err("짝맞춤.값은 인자 2개를 받습니다".to_string().into());
                }
                let Value::Map(map) = &args[0] else {
                    return Err("짝맞춤.값은 짝맞춤 인자가 필요합니다".to_string().into());
                };
                Ok(map_get(map, &args[1]))
            }
            "짝맞춤.바꾼값" => {
                if args.len() != 3 {
                    return Err("짝맞춤.바꾼값은 인자 3개를 받습니다".to_string().into());
                }
                let Value::Map(map) = &args[0] else {
                    return Err("짝맞춤.바꾼값은 짝맞춤 인자가 필요합니다".to_string().into());
                };
                let mut entries = map.clone();
                entries.insert(
                    map_key_canon(&args[1]),
                    MapEntry {
                        key: args[1].clone(),
                        value: args[2].clone(),
                    },
                );
                Ok(Value::Map(entries))
            }
            "키목록" => {
                if args.len() != 1 {
                    return Err("키목록은 인자 1개를 받습니다".to_string().into());
                }
                let Value::Pack(pack) = &args[0] else {
                    return Err("키목록은 묶음 인자를 받습니다".to_string().into());
                };
                let keys = pack
                    .keys()
                    .cloned()
                    .map(Value::String)
                    .collect::<Vec<_>>();
                Ok(Value::List(keys))
            }
            "값목록" => {
                if args.len() != 1 {
                    return Err("값목록은 인자 1개를 받습니다".to_string().into());
                }
                let Value::Pack(pack) = &args[0] else {
                    return Err("값목록은 묶음 인자를 받습니다".to_string().into());
                };
                let values = pack.values().cloned().collect::<Vec<_>>();
                Ok(Value::List(values))
            }
            "쌍목록" => {
                if args.len() != 1 {
                    return Err("쌍목록은 인자 1개를 받습니다".to_string().into());
                }
                let Value::Pack(pack) = &args[0] else {
                    return Err("쌍목록은 묶음 인자를 받습니다".to_string().into());
                };
                let pairs = pack
                    .iter()
                    .map(|(key, value)| {
                        Value::List(vec![Value::String(key.clone()), value.clone()])
                    })
                    .collect::<Vec<_>>();
                Ok(Value::List(pairs))
            }
            "채우기" => {
                if args.len() != 2 {
                    return Err("채우기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::Template(template) = &args[0] else {
                    return Err("채우기는 글무늬 인자가 필요합니다".to_string().into());
                };
                let Value::Pack(pack) = &args[1] else {
                    return Err("채우기는 묶음 주입 인자가 필요합니다".to_string().into());
                };
                let rendered = render_template(template, pack)?;
                Ok(Value::String(rendered))
            }
            "풀기" => {
                if args.len() != 2 {
                    return Err("풀기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::Formula(formula) = &args[0] else {
                    return Err("풀기는 수식값 인자가 필요합니다".to_string().into());
                };
                let Value::Pack(pack) = &args[1] else {
                    return Err("풀기는 묶음 주입 인자가 필요합니다".to_string().into());
                };
                if matches!(formula.dialect, FormulaDialect::Latex | FormulaDialect::Other(_)) {
                    return Err("FATAL:FORMULA_DIALECT_UNSUPPORTED".to_string().into());
                }
                let parsed = parse_formula_value(formula)?;
                let required = &parsed.vars;
                let provided: BTreeSet<String> = pack.keys().cloned().collect();
                let missing: Vec<String> = required.difference(&provided).cloned().collect();
                if !missing.is_empty() {
                    return Err(format!("풀기: 주입 키가 누락되었습니다: {}", missing.join(", ")).into());
                }
                let extra: Vec<String> = provided.difference(&required).cloned().collect();
                if !extra.is_empty() {
                    return Err(format!("풀기: 주입 키가 여분입니다: {}", extra.join(", ")).into());
                }
                for (key, value) in pack {
                    if matches!(value, Value::None) {
                        return Err(format!("풀기: 키 '{}' 값이 없습니다", key).into());
                    }
                }
                let value = eval_formula_expr(&parsed.expr, pack)?;
                Ok(unit_value_to_value(value))
            }
            "미분하기" => {
                let (formula, options) = expect_formula_transform(&args, "미분하기")?;
                let transformed = transform_formula_value(formula, options, "diff", "미분하기")?;
                Ok(Value::Formula(transformed))
            }
            "적분하기" => {
                let (formula, options) = expect_formula_transform(&args, "적분하기")?;
                let transformed = transform_formula_value(formula, options, "int", "적분하기")?;
                Ok(Value::Formula(transformed))
            }
            "번째" => {
                if args.len() != 2 {
                    return Err("번째는 인자 2개를 받습니다".to_string().into());
                }
                list_nth(&args[0], &args[1]).map_err(runtime_error)
            }
            "차림.값" => {
                if args.len() != 2 {
                    return Err("차림.값은 인자 2개를 받습니다".to_string().into());
                }
                list_nth(&args[0], &args[1]).map_err(runtime_error)
            }
            "차림.바꾼값" => {
                if args.len() != 3 {
                    return Err("차림.바꾼값은 인자 3개를 받습니다".to_string().into());
                }
                list_set(&args[0], &args[1], args[2].clone()).map_err(runtime_error)
            }
            "길이" => {
                if args.len() != 1 {
                    return Err("길이는 인자 1개를 받습니다".to_string().into());
                }
                match args[0] {
                    Value::List(_) => list_len(&args[0]).map_err(runtime_error),
                    Value::String(_) => string_len(&args[0]).map_err(runtime_error),
                    _ => Err("길이는 차림/글에만 적용됩니다".to_string().into()),
                }
            }
            "범위" => {
                let (start, end, step) = match args.len() {
                    2 => {
                        let start = unit_value_from_value(&args[0])?;
                        let end = unit_value_from_value(&args[1])?;
                        if start.dim != end.dim {
                            return Err(unit_error(UnitError::DimensionMismatch {
                                left: start.dim,
                                right: end.dim,
                            }));
                        }
                        let step = UnitValue { value: Fixed64::from_i64(1), dim: start.dim };
                        (start, end, step)
                    }
                    3 => {
                        let start = unit_value_from_value(&args[0])?;
                        let end = unit_value_from_value(&args[1])?;
                        let step = unit_value_from_value(&args[2])?;
                        if start.dim != end.dim {
                            return Err(unit_error(UnitError::DimensionMismatch {
                                left: start.dim,
                                right: end.dim,
                            }));
                        }
                        if start.dim != step.dim {
                            return Err(unit_error(UnitError::DimensionMismatch {
                                left: start.dim,
                                right: step.dim,
                            }));
                        }
                        (start, end, step)
                    }
                    _ => {
                        return Err("범위는 인자 2개 또는 3개를 받습니다".to_string().into());
                    }
                };
                if step.value.raw_i64() == 0 {
                    return Err("범위 간격은 0이 될 수 없습니다".to_string().into());
                }
                let mut items = Vec::new();
                let mut current = start.value;
                let step_value = step.value;
                if step_value.raw_i64() > 0 {
                    while current.raw_i64() <= end.value.raw_i64() {
                        items.push(unit_value_to_value(UnitValue { value: current, dim: start.dim }));
                        current = current.saturating_add(step_value);
                    }
                } else {
                    while current.raw_i64() >= end.value.raw_i64() {
                        items.push(unit_value_to_value(UnitValue { value: current, dim: start.dim }));
                        current = current.saturating_add(step_value);
                    }
                }
                Ok(Value::List(items))
            }
            "표준.범위" => {
                if args.len() != 3 {
                    return Err("표준.범위는 인자 3개를 받습니다".to_string().into());
                }
                let start = unit_value_from_value(&args[0])?;
                let end = unit_value_from_value(&args[1])?;
                if start.dim != end.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: start.dim,
                        right: end.dim,
                    }));
                }
                let include = unit_value_from_value(&args[2])?;
                if include.dim != UnitDim::NONE {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: include.dim,
                        right: UnitDim::NONE,
                    }));
                }
                let flag = include.value.raw_i64();
                let one = Fixed64::from_i64(1).raw_i64();
                if flag != 0 && flag != one {
                    return Err("끝포함은 0 또는 1이어야 합니다".to_string().into());
                }
                let include_end = flag == one;
                let step = UnitValue { value: Fixed64::from_i64(1), dim: start.dim };
                let mut items = Vec::new();
                let mut current = start.value;
                if start.value.raw_i64() <= end.value.raw_i64() {
                    if include_end {
                        while current.raw_i64() <= end.value.raw_i64() {
                            items.push(unit_value_to_value(UnitValue { value: current, dim: start.dim }));
                            current = current.saturating_add(step.value);
                        }
                    } else {
                        while current.raw_i64() < end.value.raw_i64() {
                            items.push(unit_value_to_value(UnitValue { value: current, dim: start.dim }));
                            current = current.saturating_add(step.value);
                        }
                    }
                }
                Ok(Value::List(items))
            }
            "대문자로바꾸기" => {
                if args.len() != 1 {
                    return Err("대문자로바꾸기는 인자 1개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("대문자로바꾸기는 글 인자를 받습니다".to_string().into());
                };
                Ok(Value::String(text.to_uppercase()))
            }
            "소문자로바꾸기" => {
                if args.len() != 1 {
                    return Err("소문자로바꾸기는 인자 1개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("소문자로바꾸기는 글 인자를 받습니다".to_string().into());
                };
                Ok(Value::String(text.to_lowercase()))
            }
            "다듬기" => {
                if args.len() != 1 {
                    return Err("다듬기는 인자 1개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("다듬기는 글 인자를 받습니다".to_string().into());
                };
                Ok(Value::String(text.trim().to_string()))
            }
            "되풀이하기" => {
                if args.len() != 2 {
                    return Err("되풀이하기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("되풀이하기 첫 인자는 글이어야 합니다".to_string().into());
                };
                let count = value_to_i64(&args[1])?;
                if count < 0 {
                    return Err("되풀이하기 횟수는 0 이상이어야 합니다".to_string().into());
                }
                Ok(Value::String(text.repeat(count as usize)))
            }
            "글자뽑기" => {
                if args.len() != 2 {
                    return Err("글자뽑기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("글자뽑기는 글 인자를 받습니다".to_string().into());
                };
                let idx = value_to_index(&args[1])?;
                let ch = text.chars().nth(idx).map(|c| c.to_string());
                Ok(ch.map(Value::String).unwrap_or(Value::None))
            }
            "찾기" => {
                if args.len() != 2 {
                    return Err("찾기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("찾기는 글 인자를 받습니다".to_string().into());
                };
                let Value::String(pattern) = &args[1] else {
                    return Err("찾기는 글 인자를 받습니다".to_string().into());
                };
                let idx = text.find(pattern).map(|byte_idx| text[..byte_idx].chars().count() as i64);
                Ok(Value::Fixed64(Fixed64::from_i64(idx.unwrap_or(-1))))
            }
            "첫번째" => {
                if args.len() != 1 {
                    return Err("첫번째는 인자 1개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("첫번째는 차림 인자를 받습니다".to_string().into());
                };
                Ok(items.first().cloned().unwrap_or(Value::None))
            }
            "마지막" => {
                if args.len() != 1 {
                    return Err("마지막은 인자 1개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("마지막은 차림 인자를 받습니다".to_string().into());
                };
                Ok(items.last().cloned().unwrap_or(Value::None))
            }
            "추가" => {
                if args.len() != 2 {
                    return Err("추가는 인자 2개를 받습니다".to_string().into());
                }
                list_add(&args[0], args[1].clone()).map_err(runtime_error)
            }
            "제거" => {
                if args.len() != 2 {
                    return Err("제거는 인자 2개를 받습니다".to_string().into());
                }
                list_remove(&args[0], &args[1]).map_err(runtime_error)
            }
            "정렬" => {
                if args.len() != 2 {
                    return Err("정렬은 인자 2개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("정렬은 차림 인자를 받습니다".to_string().into());
                };
                let func = args[1].clone();
                let mut keyed = Vec::with_capacity(items.len());
                for (idx, item) in items.iter().cloned().enumerate() {
                    let key = self.eval_callable(&func, vec![item.clone()])?;
                    keyed.push((idx, key, item));
                }
                keyed.sort_by(|a, b| value_cmp(&a.1, &b.1).then_with(|| a.0.cmp(&b.0)));
                let items = keyed.into_iter().map(|(_, _, item)| item).collect();
                Ok(Value::List(items))
            }
            "거르기" => {
                if args.len() != 2 {
                    return Err("거르기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("거르기는 차림 인자를 받습니다".to_string().into());
                };
                let func = args[1].clone();
                let mut out = Vec::new();
                for item in items.iter().cloned() {
                    let verdict = self.eval_callable(&func, vec![item.clone()])?;
                    if is_truthy(&verdict)? {
                        out.push(item);
                    }
                }
                Ok(Value::List(out))
            }
            "변환" => {
                if args.len() != 2 {
                    return Err("변환은 인자 2개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("변환은 차림 인자를 받습니다".to_string().into());
                };
                let func = args[1].clone();
                let mut out = Vec::with_capacity(items.len());
                for item in items.iter().cloned() {
                    let mapped = self.eval_callable(&func, vec![item])?;
                    out.push(mapped);
                }
                Ok(Value::List(out))
            }
            "바꾸기" => {
                if args.len() == 3 {
                    let Value::String(text) = &args[0] else {
                        return Err("바꾸기 첫 인자는 글이어야 합니다".to_string().into());
                    };
                    let Value::String(from) = &args[1] else {
                        return Err("바꾸기 둘째 인자는 글이어야 합니다".to_string().into());
                    };
                    let Value::String(to) = &args[2] else {
                        return Err("바꾸기 셋째 인자는 글이어야 합니다".to_string().into());
                    };
                    return Ok(Value::String(text.replace(from, to)));
                }
                if args.len() != 2 {
                    return Err("바꾸기는 인자 2개 또는 3개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("바꾸기는 차림 인자를 받습니다".to_string().into());
                };
                let func = args[1].clone();
                let mut out = Vec::with_capacity(items.len());
                for item in items.iter().cloned() {
                    let mapped = self.eval_callable(&func, vec![item])?;
                    out.push(mapped);
                }
                Ok(Value::List(out))
            }
            "토막내기" => {
                if args.len() != 3 {
                    return Err("토막내기는 인자 3개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("토막내기는 차림 인자를 받습니다".to_string().into());
                };
                let start = value_to_index(&args[1])?;
                let end = value_to_index(&args[2])?;
                if start >= items.len() || end <= start {
                    return Ok(Value::List(Vec::new()));
                }
                let end = end.min(items.len());
                Ok(Value::List(items[start..end].to_vec()))
            }
            "들어있나" => {
                if args.len() != 2 {
                    return Err("들어있나는 인자 2개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("들어있나는 차림 인자를 받습니다".to_string().into());
                };
                let target = &args[1];
                let exists = items.iter().any(|item| values_equal(item, target));
                Ok(Value::Bool(exists))
            }
            "찾아보기" => {
                if args.len() != 2 {
                    return Err("찾아보기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("찾아보기는 차림 인자를 받습니다".to_string().into());
                };
                let target = &args[1];
                let idx = items
                    .iter()
                    .position(|item| values_equal(item, target))
                    .map(|i| i as i64)
                    .unwrap_or(-1);
                Ok(Value::Fixed64(Fixed64::from_i64(idx)))
            }
            "뒤집기" => {
                if args.len() != 1 {
                    return Err("뒤집기는 인자 1개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("뒤집기는 차림 인자를 받습니다".to_string().into());
                };
                let mut out = items.clone();
                out.reverse();
                Ok(Value::List(out))
            }
            "펼치기" => {
                if args.len() != 1 {
                    return Err("펼치기는 인자 1개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("펼치기는 차림 인자를 받습니다".to_string().into());
                };
                let mut out = Vec::new();
                for item in items {
                    let Value::List(inner) = item else {
                        return Err("펼치기는 차림<차림> 인자를 받습니다".to_string().into());
                    };
                    out.extend(inner.clone());
                }
                Ok(Value::List(out))
            }
            "각각돌며" => {
                if args.len() != 2 {
                    return Err("각각돌며는 인자 2개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("각각돌며는 차림 인자를 받습니다".to_string().into());
                };
                let func = args[1].clone();
                for item in items.iter().cloned() {
                    let _ = self.eval_callable(&func, vec![item])?;
                }
                Ok(Value::None)
            }
            "자르기" => {
                if args.len() != 2 {
                    return Err("자르기는 인자 2개를 받습니다".to_string().into());
                }
                string_split(&args[0], &args[1]).map_err(runtime_error)
            }
            "합치기" => {
                if args.len() == 2 {
                    return match (&args[0], &args[1]) {
                        (Value::String(_), Value::String(_)) => {
                            string_concat(&args[0], &args[1]).map_err(runtime_error)
                        }
                        (Value::List(_), Value::String(_)) => {
                            string_join(&args[0], &args[1]).map_err(runtime_error)
                        }
                        (Value::Pack(left), Value::Pack(right)) => {
                            let mut merged = left.clone();
                            for (key, value) in right {
                                merged.insert(key.clone(), value.clone());
                            }
                            Ok(Value::Pack(merged))
                        }
                        _ => Err("합치기는 글/차림/묶음 조합만 지원합니다".to_string().into()),
                    };
                }
                if args.len() != 3 {
                    return Err("합치기는 인자 2개 또는 3개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("합치기는 차림 인자를 받습니다".to_string().into());
                };
                let func = args[2].clone();
                let mut acc = args[1].clone();
                for item in items.iter().cloned() {
                    acc = self.eval_callable(&func, vec![acc, item])?;
                }
                Ok(acc)
            }
            "붙이기" => {
                if args.len() != 2 {
                    return Err("붙이기는 인자 2개를 받습니다".to_string().into());
                }
                match (&args[0], &args[1]) {
                    (Value::List(_), Value::String(_)) => {
                        string_join(&args[0], &args[1]).map_err(runtime_error)
                    }
                    (Value::List(left), Value::List(right)) => {
                        let mut out = left.clone();
                        out.extend(right.clone());
                        Ok(Value::List(out))
                    }
                    _ => Err("붙이기는 차림/글 조합 또는 차림/차림 조합만 지원합니다".to_string().into()),
                }
            }
            "포함하나" => {
                if args.len() != 2 {
                    return Err("포함하나는 인자 2개를 받습니다".to_string().into());
                }
                string_contains(&args[0], &args[1]).map_err(runtime_error)
            }
            "시작하나" => {
                if args.len() != 2 {
                    return Err("시작하나는 인자 2개를 받습니다".to_string().into());
                }
                string_starts(&args[0], &args[1]).map_err(runtime_error)
            }
            "끝나나" => {
                if args.len() != 2 {
                    return Err("끝나나는 인자 2개를 받습니다".to_string().into());
                }
                string_ends(&args[0], &args[1]).map_err(runtime_error)
            }
            "숫자로" => {
                if args.len() != 1 {
                    return Err("숫자로는 인자 1개를 받습니다".to_string().into());
                }
                string_to_number(&args[0]).map_err(runtime_error)
            }
            "글바꾸기" => {
                if args.len() != 3 {
                    return Err("글바꾸기는 인자 3개를 받습니다".to_string().into());
                }
                let base = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err("글바꾸기 첫 인자는 글이어야 합니다".to_string().into()),
                };
                let idx = value_to_index(&args[1])?;
                let replacement = match &args[2] {
                    Value::String(s) => s.clone(),
                    _ => return Err("글바꾸기 새글은 글이어야 합니다".to_string().into()),
                };
                let mut out = String::new();
                let mut replaced = false;
                for (i, ch) in base.chars().enumerate() {
                    if i == idx {
                        out.push_str(&replacement);
                        replaced = true;
                    } else {
                        out.push(ch);
                    }
                }
                if !replaced {
                    return Ok(Value::String(base));
                }
                Ok(Value::String(out))
            }
            "바닥" => {
                if args.len() != 1 {
                    return Err("바닥은 인자 1개를 받습니다".to_string().into());
                }
                let qty = unit_value_from_value(&args[0])?;
                let value = fixed64_floor(qty.value);
                Ok(unit_value_to_value(UnitValue { value, dim: qty.dim }))
            }
            "천장" => {
                if args.len() != 1 {
                    return Err("천장은 인자 1개를 받습니다".to_string().into());
                }
                let qty = unit_value_from_value(&args[0])?;
                let value = fixed64_ceil(qty.value);
                Ok(unit_value_to_value(UnitValue { value, dim: qty.dim }))
            }
            "반올림" => {
                if args.len() != 1 {
                    return Err("반올림은 인자 1개를 받습니다".to_string().into());
                }
                let qty = unit_value_from_value(&args[0])?;
                let value = fixed64_round_even(qty.value);
                Ok(unit_value_to_value(UnitValue { value, dim: qty.dim }))
            }
            "합계" => {
                if args.len() != 1 {
                    return Err("합계는 인자 1개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("합계는 차림 인자를 받습니다".to_string().into());
                };
                if items.is_empty() {
                    return Ok(Value::Fixed64(Fixed64::from_i64(0)));
                }
                let mut total = unit_value_from_value(&items[0])?;
                for item in items.iter().skip(1) {
                    let next = unit_value_from_value(item)?;
                    total = total.add(next).map_err(unit_error)?;
                }
                Ok(unit_value_to_value(total))
            }
            "평균" => {
                if args.len() != 1 {
                    return Err("평균은 인자 1개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("평균은 차림 인자를 받습니다".to_string().into());
                };
                if items.is_empty() {
                    return Ok(Value::None);
                }
                let mut total = unit_value_from_value(&items[0])?;
                for item in items.iter().skip(1) {
                    let next = unit_value_from_value(item)?;
                    total = total.add(next).map_err(unit_error)?;
                }
                let count = Fixed64::from_i64(items.len() as i64);
                let avg = total.div_scalar(count).map_err(unit_error)?;
                Ok(unit_value_to_value(avg))
            }
            "abs" => {
                if args.len() != 1 {
                    return Err("abs는 인자 1개를 받습니다".to_string().into());
                }
                let qty = unit_value_from_value(&args[0])?;
                let value = fixed64_abs(qty.value);
                Ok(unit_value_to_value(UnitValue { value, dim: qty.dim }))
            }
            "min" => {
                if args.len() != 2 {
                    return Err("min은 인자 2개를 받습니다".to_string().into());
                }
                let left = unit_value_from_value(&args[0])?;
                let right = unit_value_from_value(&args[1])?;
                if left.dim != right.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: left.dim,
                        right: right.dim,
                    }));
                }
                let out = if left.value <= right.value { left } else { right };
                Ok(unit_value_to_value(out))
            }
            "max" => {
                if args.len() != 2 {
                    return Err("max는 인자 2개를 받습니다".to_string().into());
                }
                let left = unit_value_from_value(&args[0])?;
                let right = unit_value_from_value(&args[1])?;
                if left.dim != right.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: left.dim,
                        right: right.dim,
                    }));
                }
                let out = if left.value >= right.value { left } else { right };
                Ok(unit_value_to_value(out))
            }
            "clamp" => {
                if args.len() != 3 {
                    return Err("clamp는 인자 3개를 받습니다".to_string().into());
                }
                let value = unit_value_from_value(&args[0])?;
                let min = unit_value_from_value(&args[1])?;
                let max = unit_value_from_value(&args[2])?;
                if value.dim != min.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: value.dim,
                        right: min.dim,
                    }));
                }
                if value.dim != max.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: value.dim,
                        right: max.dim,
                    }));
                }
                let clamped = if value.value < min.value {
                    min
                } else if value.value > max.value {
                    max
                } else {
                    value
                };
                Ok(unit_value_to_value(clamped))
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Err("sqrt는 인자 1개를 받습니다".to_string().into());
                }
                let qty = unit_value_from_value(&args[0])?;
                if qty.value.raw_i64() < 0 {
                    return Err("음수의 제곱근은 지원하지 않습니다".to_string().into());
                }
                let Some(dim) = qty.dim.sqrt() else {
                    return Err("제곱근은 짝수 차원만 지원합니다".to_string().into());
                };
                let Some(value) = fixed64_sqrt(qty.value) else {
                    return Err("제곱근 계산 실패".to_string().into());
                };
                Ok(unit_value_to_value(UnitValue { value, dim }))
            }
            "powi" => {
                if args.len() != 2 {
                    return Err("powi는 인자 2개를 받습니다".to_string().into());
                }
                let base = unit_value_from_value(&args[0])?;
                let exp = unit_value_from_value(&args[1])?;
                if !exp.is_dimensionless() {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: exp.dim,
                        right: UnitDim::NONE,
                    }));
                }
                if exp.value.frac_part() != 0 {
                    return Err("powi는 정수 지수만 지원합니다".to_string().into());
                }
                let value = unit_powi(base, exp.value.int_part())?;
                Ok(unit_value_to_value(value))
            }
            "글로" => {
                if args.len() != 1 {
                    return Err("글로는 인자 1개를 받습니다".to_string().into());
                }
                Ok(Value::String(value_to_string(&args[0])))
            }
            "자원" => {
                if args.len() != 1 {
                    return Err("자원은 인자 1개를 받습니다".to_string().into());
                }
                match &args[0] {
                    Value::String(path) => Ok(Value::ResourceHandle(ResourceHandle::from_path(path))),
                    Value::ResourceHandle(handle) => Ok(Value::ResourceHandle(*handle)),
                    _ => Err("자원은 글 인자를 받습니다".to_string().into()),
                }
            }
            "눌렸나" => {
                if args.len() != 1 {
                    return Err("눌렸나는 인자 1개를 받습니다".to_string().into());
                }
                let key = value_to_string(&args[0]);
                Ok(input_pressed(&self.input, &key))
            }
            "막눌렸나" => {
                if args.len() != 1 {
                    return Err("막눌렸나는 인자 1개를 받습니다".to_string().into());
                }
                let key = value_to_string(&args[0]);
                Ok(input_just_pressed(&self.input, &key))
            }
            "무작위" => {
                if !args.is_empty() {
                    return Err("무작위는 인자 0개를 받습니다".to_string().into());
                }
                Ok(Value::Fixed64(self.next_rng_fixed64()))
            }
            "무작위정수" => {
                if args.len() != 2 {
                    return Err("무작위정수는 인자 2개를 받습니다".to_string().into());
                }
                let min = value_to_i64(&args[0])?;
                let max = value_to_i64(&args[1])?;
                if min > max {
                    return Err("무작위정수의 최소/최대가 올바르지 않습니다".to_string().into());
                }
                let range = (max - min + 1) as u64;
                let value = (self.next_rng_u64() % range) as i64 + min;
                Ok(Value::Fixed64(Fixed64::from_i64(value)))
            }
            "무작위선택" => {
                if args.len() != 1 {
                    return Err("무작위선택은 인자 1개를 받습니다".to_string().into());
                }
                let Value::List(items) = &args[0] else {
                    return Err("무작위선택은 차림 인자를 받습니다".to_string().into());
                };
                if items.is_empty() {
                    return Ok(Value::None);
                }
                let idx = (self.next_rng_u64() % items.len() as u64) as usize;
                Ok(items[idx].clone())
            }

            _ => {
                let seed = self
                    .program
                    .functions
                    .get(func)
                    .ok_or_else(|| format!("함수 '{}'를 찾을 수 없습니다", func))?
                    .clone();
                self.eval_seed(&seed, args)
            }
        }
    }

    fn eval_infix(&self, op: &str, left: Value, right: Value) -> Result<Value, EvalError> {
        match op {
            "+" => {
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                let sum = l.add(r).map_err(unit_error)?;
                Ok(unit_value_to_value(sum))
            }
            "-" => {
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                let diff = l.sub(r).map_err(unit_error)?;
                Ok(unit_value_to_value(diff))
            }
            "*" => {
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                Ok(unit_value_to_value(l.mul(r)))
            }
            "/" => {
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                let div = l.div(r).map_err(unit_error)?;
                Ok(unit_value_to_value(div))
            }
            "%" => {
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                if l.dim != r.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: l.dim,
                        right: r.dim,
                    }));
                }
                if r.value.raw_i64() == 0 {
                    return Err(unit_error(UnitError::DivisionByZero));
                }
                let raw = l.value.raw_i64() % r.value.raw_i64();
                Ok(unit_value_to_value(UnitValue {
                    value: Fixed64::from_raw_i64(raw),
                    dim: l.dim,
                }))
            }
            "==" | "!=" => {
                if let (Some(l), Some(r)) = (numeric_value(&left), numeric_value(&right)) {
                    if l.dim != r.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: l.dim,
                            right: r.dim,
                        }));
                    }
                    let eq = l.value.raw_i64() == r.value.raw_i64();
                    return Ok(Value::Bool(if op == "==" { eq } else { !eq }));
                }
                let eq = values_equal(&left, &right);
                Ok(Value::Bool(if op == "==" { eq } else { !eq }))
            }
            "<" | "<=" | ">" | ">=" => {
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                if l.dim != r.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: l.dim,
                        right: r.dim,
                    }));
                }
                let cmp = match op {
                    "<" => l.value < r.value,
                    "<=" => l.value <= r.value,
                    ">" => l.value > r.value,
                    ">=" => l.value >= r.value,
                    _ => false,
                };
                Ok(Value::Bool(cmp))
            }
            "&&" | "그리고" => {
                let l = is_truthy(&left)?;
                let r = is_truthy(&right)?;
                Ok(Value::Bool(l && r))
            }
            "||" | "또는" => {
                let l = is_truthy(&left)?;
                let r = is_truthy(&right)?;
                Ok(Value::Bool(l || r))
            }
            _ => Err(format!("지원하지 않는 연산자: {}", op).into()),
        }
    }

    fn emit_unit_mismatch(
        &mut self,
        left: UnitDim,
        right: UnitDim,
        target: Option<String>,
        source_span: Option<SourceSpan>,
        expr: Option<ExprTrace>,
    ) {
        self.emit_arith_fault(
            ArithmeticFaultKind::DimensionMismatch { left, right },
            target,
            source_span,
            expr,
        );
    }

    fn emit_arith_fault(
        &mut self,
        kind: ArithmeticFaultKind,
        target: Option<String>,
        source_span: Option<SourceSpan>,
        expr: Option<ExprTrace>,
    ) {
        let location = match kind {
            ArithmeticFaultKind::DimensionMismatch { .. } => "ddn:unit_mismatch",
            ArithmeticFaultKind::DivByZero => "ddn:div0",
        };
        let ctx = FaultContext {
            tick_id: self.tick_id,
            location,
            source_span,
            expr,
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::ArithmeticFault { ctx, kind },
            targets: target.map(|label| vec![label]).unwrap_or_default(),
        });
    }

    fn emit_contract_violation(
        &mut self,
        kind: ddonirang_lang::ContractKind,
        mode: ddonirang_lang::ContractMode,
        condition: &Expr,
        message: String,
    ) {
        let (reason, sub_reason, contract_kind, tag) = match kind {
            ddonirang_lang::ContractKind::Pre => (
                "CONTRACT_PRE",
                "PRE_VIOLATION",
                "pre",
                "contract:pre",
            ),
            ddonirang_lang::ContractKind::Post => (
                "CONTRACT_POST",
                "POST_VIOLATION",
                "post",
                "contract:post",
            ),
        };
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: reason.to_string(),
            rule_id: "L0-CONTRACT-01".to_string(),
            reason: reason.to_string(),
            sub_reason: Some(sub_reason.to_string()),
            mode: Some(match mode {
                ddonirang_lang::ContractMode::Alert => "알림".to_string(),
                ddonirang_lang::ContractMode::Abort => "중단".to_string(),
            }),
            contract_kind: Some(contract_kind.to_string()),
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span: self.source_span_for_expr(condition),
            expr: Some(ExprTrace {
                tag: tag.to_string(),
                text: None,
            }),
            message: Some(message),
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
        if matches!(mode, ddonirang_lang::ContractMode::Abort) {
            self.aborted = true;
        }
    }

    fn emit_guard_violation(&mut self, expr_id: u64) {
        self.patch_ops.push(PatchOp::GuardViolation {
            entity: EntityId(0),
            rule_id: format!("RULE_GUARD#{}", expr_id),
        });
    }

    fn arith_trace(&self, _expr: &Expr, tag: &str) -> ExprTrace {
        ExprTrace {
            tag: tag.to_string(),
            text: None,
        }
    }

    fn source_span_for_expr(&self, expr: &Expr) -> Option<SourceSpan> {
        self.source_span_from_span(&expr.span)
    }

    fn source_span_from_span(&self, span: &ddonirang_lang::Span) -> Option<SourceSpan> {
        let origin = &self.program.program.origin;
        if origin.source.is_empty() {
            return None;
        }
        if span.start > origin.source.len() || span.end > origin.source.len() {
            return None;
        }
        let (start_line, start_col) = position_to_line_col(&origin.source, span.start);
        let (end_line, end_col) = position_to_line_col(&origin.source, span.end);
        Some(SourceSpan {
            file: origin.file_path.clone(),
            start_line,
            start_col: Some(start_col),
            end_line,
            end_col: Some(end_col),
        })
    }

    fn resource_exists(&self, name: &str) -> bool {
        if self.resources.contains_key(name) || self.defaults.contains_key(name) {
            return true;
        }
        self.world.get_resource_value(name).is_some()
            || self.world.get_resource_fixed64(name).is_some()
            || self.world.get_resource_handle(name).is_some()
            || self.world.get_resource_json(name).is_some()
    }

    fn get_resource(&mut self, name: &str) -> Option<Value> {
        if let Some(value) = self.resources.get(name) {
            return Some(value.clone());
        }
        if let Some(value) = self.world.get_resource_value(name) {
            let v = value_from_resource_value(&value);
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        if let Some(value) = self.world.get_resource_fixed64(name) {
            if let Some(spec) = parse_resource_unit_tag(name) {
                let unit_value = UnitValue::from_spec(value, spec);
                let v = Value::Unit(unit_value);
                self.resources.insert(name.to_string(), v.clone());
                return Some(v);
            }
            let v = Value::Fixed64(value);
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        if let Some(handle) = self.world.get_resource_handle(name) {
            let v = Value::ResourceHandle(handle);
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        if let Some(value) = self.world.get_resource_json(name) {
            let v = if value == "참" {
                Value::Bool(true)
            } else if value == "거짓" {
                Value::Bool(false)
            } else {
                Value::String(value)
            };
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        if let Some(value) = self.defaults.get(name) {
            let v = value.clone();
            self.resources.insert(name.to_string(), v.clone());
            return Some(v);
        }
        None
    }

    fn set_resource(&mut self, name: &str, value: Value) -> Result<(), EvalError> {
        match value {
            Value::Fixed64(v) => {
                if let Some(spec) = parse_resource_unit_tag(name) {
                    self.patch_ops.push(PatchOp::SetResourceFixed64 {
                        tag: name.to_string(),
                        value: v,
                    });
                    let unit_value = UnitValue::from_spec(v, spec);
                    self.resources.insert(name.to_string(), Value::Unit(unit_value));
                } else {
                    self.patch_ops.push(PatchOp::SetResourceFixed64 {
                        tag: name.to_string(),
                        value: v,
                    });
                    self.resources.insert(name.to_string(), Value::Fixed64(v));
                }
            }
            Value::Unit(unit) => {
                if let Some(spec) = parse_resource_unit_tag(name) {
                    if unit.dim != spec.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: unit.dim,
                            right: spec.dim,
                        }));
                    }
                    let stored = unit.to_unit(spec).map_err(unit_error)?;
                    self.patch_ops.push(PatchOp::SetResourceFixed64 {
                        tag: name.to_string(),
                        value: stored,
                    });
                    self.resources.insert(name.to_string(), Value::Unit(unit));
                } else if unit.is_dimensionless() {
                    self.patch_ops.push(PatchOp::SetResourceFixed64 {
                        tag: name.to_string(),
                        value: unit.value,
                    });
                    self.resources.insert(name.to_string(), Value::Fixed64(unit.value));
                } else {
                    return Err("단위가 있는 값은 단위 태그 자원에만 저장할 수 있습니다".to_string().into());
                }
            }
            Value::String(s) => {
                self.patch_ops.push(PatchOp::SetResourceJson {
                    tag: name.to_string(),
                    json: s.clone(),
                });
                self.resources.insert(name.to_string(), Value::String(s));
            }
            Value::Bool(b) => {
                let s = if b { "참" } else { "거짓" }.to_string();
                self.patch_ops.push(PatchOp::SetResourceJson {
                    tag: name.to_string(),
                    json: s.clone(),
                });
                self.resources.insert(name.to_string(), Value::Bool(b));
            }
            Value::ResourceHandle(handle) => {
                self.patch_ops.push(PatchOp::SetResourceHandle {
                    tag: name.to_string(),
                    handle,
                });
                self.resources
                    .insert(name.to_string(), Value::ResourceHandle(handle));
            }
            Value::None => return Err("없음은 자원에 대입할 수 없습니다".to_string().into()),
            Value::List(_) | Value::Set(_) | Value::Map(_) | Value::Pack(_) => {
                if parse_resource_unit_tag(name).is_some() {
                    return Err(
                        "단위 태그 자원에는 차림/모음/짝맞춤/묶음을 저장할 수 없습니다"
                            .to_string()
                            .into(),
                    );
                }
                let resource_value = resource_value_from_value(&value)?;
                self.patch_ops.push(PatchOp::SetResourceValue {
                    tag: name.to_string(),
                    value: resource_value,
                });
                self.resources.insert(name.to_string(), value);
            }
            Value::Formula(_) => return Err("수식은 자원에 대입할 수 없습니다".to_string().into()),
            Value::Template(_) => return Err("글무늬는 자원에 대입할 수 없습니다".to_string().into()),
            Value::Lambda(_) => return Err("씨앗은 자원에 대입할 수 없습니다".to_string().into()),
        }
        Ok(())
    }
}

fn literal_to_value(lit: &Literal) -> Value {
    match lit {
        Literal::Int(n) => Value::Fixed64(Fixed64::from_i64(*n)),
        Literal::Fixed64(n) => Value::Fixed64(*n),
        Literal::String(s) => Value::String(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Atom(a) => Value::String(a.clone()),
        Literal::Resource(path) => Value::ResourceHandle(ResourceHandle::from_path(path)),
        Literal::None => Value::None,
    }
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Fixed64(a), Value::Fixed64(b)) => a.raw_i64() == b.raw_i64(),
        (Value::Fixed64(a), Value::Unit(b)) if b.is_dimensionless() => a.raw_i64() == b.value.raw_i64(),
        (Value::Unit(a), Value::Fixed64(b)) if a.is_dimensionless() => a.value.raw_i64() == b.raw_i64(),
        (Value::Unit(a), Value::Unit(b)) => a.dim == b.dim && a.value.raw_i64() == b.value.raw_i64(),
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::ResourceHandle(a), Value::ResourceHandle(b)) => a == b,
        (Value::None, Value::None) => true,
        (Value::Pack(a), Value::Pack(b)) => a == b,
        (Value::Formula(a), Value::Formula(b)) => a == b,
        (Value::Template(a), Value::Template(b)) => a.raw == b.raw && a.tag == b.tag,
        (Value::Lambda(a), Value::Lambda(b)) => a.id == b.id,
        _ => false,
    }
}

fn value_rank(value: &Value) -> u8 {
    match value {
        Value::None => 0,
        Value::Bool(_) => 1,
        Value::Fixed64(_) | Value::Unit(_) => 2,
        Value::String(_) => 3,
        Value::ResourceHandle(_) => 4,
        Value::Formula(_) => 5,
        Value::Template(_) => 6,
        Value::Lambda(_) => 7,
        Value::Pack(_) => 8,
        Value::List(_) => 9,
        Value::Set(_) => 10,
        Value::Map(_) => 11,
    }
}

fn value_cmp(left: &Value, right: &Value) -> std::cmp::Ordering {
    let rank_left = value_rank(left);
    let rank_right = value_rank(right);
    if rank_left != rank_right {
        return rank_left.cmp(&rank_right);
    }
    match (left, right) {
        (Value::None, Value::None) => std::cmp::Ordering::Equal,
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        (Value::Fixed64(a), Value::Fixed64(b)) => a.raw_i64().cmp(&b.raw_i64()),
        (Value::Unit(a), Value::Unit(b)) => {
            if a.dim != b.dim {
                return a.dim.format().cmp(&b.dim.format());
            }
            a.value.raw_i64().cmp(&b.value.raw_i64())
        }
        (Value::Fixed64(a), Value::Unit(b)) if b.is_dimensionless() => a.raw_i64().cmp(&b.value.raw_i64()),
        (Value::Unit(a), Value::Fixed64(b)) if a.is_dimensionless() => a.value.raw_i64().cmp(&b.raw_i64()),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::ResourceHandle(a), Value::ResourceHandle(b)) => a.cmp(b),
        (Value::Formula(a), Value::Formula(b)) => a.raw.cmp(&b.raw),
        (Value::Template(a), Value::Template(b)) => a.raw.cmp(&b.raw),
        (Value::Lambda(a), Value::Lambda(b)) => a.id.cmp(&b.id),
        _ => value_canon(left).cmp(&value_canon(right)),
    }
}

fn apply_suffix_value(value: Value, suffix: &AtSuffix) -> Result<Value, EvalError> {
    match suffix {
        AtSuffix::Unit(unit) => apply_unit_suffix(value, unit),
        AtSuffix::Asset(path) => {
            let handle = gate0_registry::resolve_asset_handle(path)?;
            Ok(Value::ResourceHandle(handle))
        }
    }
}

fn apply_unit_suffix(value: Value, unit: &str) -> Result<Value, EvalError> {
    let Some(spec) = unit_spec_from_symbol(unit) else {
        return Err(format!("단위 '{}'를 알 수 없습니다", unit).into());
    };
    match value {
        Value::Fixed64(v) => Ok(Value::Unit(UnitValue::from_spec(v, spec))),
        Value::Unit(existing) => {
            if existing.dim != spec.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: existing.dim,
                    right: spec.dim,
                }));
            }
            Ok(Value::Unit(existing))
        }
        _ => Err("단위는 수치에만 붙일 수 있습니다".to_string().into()),
    }
}

fn parse_resource_unit_tag(tag: &str) -> Option<ddonirang_core::UnitSpec> {
    let (name, unit) = tag.rsplit_once('@')?;
    if name.is_empty() {
        return None;
    }
    unit_spec_from_symbol(unit)
}

fn unit_value_from_value(value: &Value) -> Result<UnitValue, EvalError> {
    match value {
        Value::Fixed64(v) => Ok(UnitValue { value: *v, dim: UnitDim::NONE }),
        Value::Unit(unit) => Ok(*unit),
        _ => Err("수치가 필요합니다".to_string().into()),
    }
}

fn numeric_value(value: &Value) -> Option<UnitValue> {
    match value {
        Value::Fixed64(v) => Some(UnitValue { value: *v, dim: UnitDim::NONE }),
        Value::Unit(unit) => Some(*unit),
        _ => None,
    }
}

fn unit_value_to_value(value: UnitValue) -> Value {
    if value.is_dimensionless() {
        Value::Fixed64(value.value)
    } else {
        Value::Unit(value)
    }
}

fn unit_error(err: UnitError) -> EvalError {
    EvalError::from(err)
}

fn fixed64_floor(value: Fixed64) -> Fixed64 {
    Fixed64::from_i64(value.int_part())
}

fn fixed64_to_f64(value: Fixed64) -> f64 {
    value.raw_i64() as f64 / Fixed64::SCALE_F64
}

fn fixed64_ceil(value: Fixed64) -> Fixed64 {
    let int_part = value.int_part();
    if value.frac_part() == 0 {
        Fixed64::from_i64(int_part)
    } else {
        Fixed64::from_i64(int_part.saturating_add(1))
    }
}

fn fixed64_round_even(value: Fixed64) -> Fixed64 {
    let raw = value.raw_i64() as i128;
    if raw == 0 {
        return Fixed64::from_i64(0);
    }
    let sign = if raw < 0 { -1 } else { 1 };
    let abs = raw.abs();
    let frac_mask = (1_i128 << Fixed64::FRAC_BITS) - 1;
    let int_part = abs >> Fixed64::FRAC_BITS;
    let frac = abs & frac_mask;
    let half = 1_i128 << (Fixed64::FRAC_BITS - 1);
    let mut rounded = int_part;
    if frac > half || (frac == half && (int_part & 1) != 0) {
        rounded += 1;
    }
    Fixed64::from_i64((rounded * sign) as i64)
}

fn fixed64_abs(value: Fixed64) -> Fixed64 {
    let raw = value.raw_i64();
    let abs = if raw == i64::MIN { i64::MAX } else { raw.abs() };
    Fixed64::from_raw_i64(abs)
}

fn fixed64_sqrt(value: Fixed64) -> Option<Fixed64> {
    if value.raw_i64() < 0 {
        return None;
    }
    let scaled = (value.raw_i64() as i128) << Fixed64::FRAC_BITS;
    let root = int_sqrt(scaled);
    let capped = if root > i64::MAX as i128 {
        i64::MAX
    } else {
        root as i64
    };
    Some(Fixed64::from_raw_i64(capped))
}

fn int_sqrt(value: i128) -> i128 {
    if value <= 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) >> 1;
    while y < x {
        x = y;
        y = (x + value / x) >> 1;
    }
    x
}

fn unit_powi(base: UnitValue, exp: i64) -> Result<UnitValue, EvalError> {
    if exp == 0 {
        return Ok(UnitValue {
            value: Fixed64::from_i64(1),
            dim: UnitDim::NONE,
        });
    }
    let mut result = base;
    let abs_exp = exp.abs() as i64;
    for _ in 1..abs_exp {
        result = result.mul(base);
    }
    if exp < 0 {
        let one = UnitValue {
            value: Fixed64::from_i64(1),
            dim: UnitDim::NONE,
        };
        return one.div(result).map_err(unit_error);
    }
    Ok(result)
}

fn value_to_i64(value: &Value) -> Result<i64, EvalError> {
    match value {
        Value::Fixed64(n) => Ok(n.int_part()),
        Value::Unit(unit) if unit.is_dimensionless() => Ok(unit.value.int_part()),
        _ => Err("정수 값이 필요합니다".to_string().into()),
    }
}

fn value_to_index(value: &Value) -> Result<usize, EvalError> {
    match value {
        Value::Fixed64(n) => {
            let idx = n.int_part();
            if idx < 0 {
                Err("인덱스는 0 이상이어야 합니다".to_string().into())
            } else {
                Ok(idx as usize)
            }
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            let idx = unit.value.int_part();
            if idx < 0 {
                Err("인덱스는 0 이상이어야 합니다".to_string().into())
            } else {
                Ok(idx as usize)
            }
        }
        _ => Err("인덱스는 정수여야 합니다".to_string().into()),
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Fixed64(n) => n.to_string(),
        Value::Unit(unit) => {
            let suffix = unit
                .display_symbol()
                .map(|s| s.to_string())
                .unwrap_or_else(|| unit.dim.format());
            format!("{}@{}", unit.value, suffix)
        }
        Value::Bool(b) => {
            if *b {
                "참".to_string()
            } else {
                "거짓".to_string()
            }
        }
        Value::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
        Value::None => String::new(),
        Value::List(_) => "[차림]".to_string(),
        Value::Set(_) => "[모음]".to_string(),
        Value::Map(_) => "[짝맞춤]".to_string(),
        Value::Pack(items) => {
            let mut out = String::from("(");
            let mut first = true;
            for (key, value) in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(key);
                out.push_str(": ");
                out.push_str(&value_to_string(value));
            }
            out.push(')');
            out
        }
        Value::Formula(formula) => formula.raw.clone(),
        Value::Template(template) => template.raw.clone(),
        Value::Lambda(lambda) => format!("<씨앗#{}>", lambda.id),
    }
}

fn resource_value_from_value(value: &Value) -> Result<ResourceValue, EvalError> {
    match value {
        Value::None => Ok(ResourceValue::None),
        Value::Bool(value) => Ok(ResourceValue::Bool(*value)),
        Value::Fixed64(value) => Ok(ResourceValue::Fixed64(*value)),
        Value::Unit(value) => Ok(ResourceValue::Unit(*value)),
        Value::String(value) => Ok(ResourceValue::String(value.clone())),
        Value::ResourceHandle(value) => Ok(ResourceValue::ResourceHandle(*value)),
        Value::List(items) => {
            let mut converted = Vec::with_capacity(items.len());
            for item in items {
                converted.push(resource_value_from_value(item)?);
            }
            Ok(ResourceValue::List(converted))
        }
        Value::Set(items) => {
            let mut converted = BTreeMap::new();
            for (key, value) in items {
                converted.insert(key.clone(), resource_value_from_value(value)?);
            }
            Ok(ResourceValue::Set(converted))
        }
        Value::Map(entries) => {
            let mut converted = BTreeMap::new();
            for (key, entry) in entries {
                let entry_key = resource_value_from_value(&entry.key)?;
                let entry_value = resource_value_from_value(&entry.value)?;
                converted.insert(
                    key.clone(),
                    ResourceMapEntry {
                        key: entry_key,
                        value: entry_value,
                    },
                );
            }
            Ok(ResourceValue::Map(converted))
        }
        Value::Pack(entries) => {
            let mut converted = BTreeMap::new();
            for (key, value) in entries {
                let entry_key = ResourceValue::String(key.clone());
                let entry_value = resource_value_from_value(value)?;
                converted.insert(
                    entry_key.canon_key(),
                    ResourceMapEntry {
                        key: entry_key,
                        value: entry_value,
                    },
                );
            }
            Ok(ResourceValue::Map(converted))
        }
        Value::Formula(_) => Err("수식은 자원에 대입할 수 없습니다".to_string().into()),
        Value::Template(_) => Err("글무늬는 자원에 대입할 수 없습니다".to_string().into()),
        Value::Lambda(_) => Err("씨앗은 자원에 대입할 수 없습니다".to_string().into()),
    }
}

fn value_from_resource_value(value: &ResourceValue) -> Value {
    match value {
        ResourceValue::None => Value::None,
        ResourceValue::Bool(value) => Value::Bool(*value),
        ResourceValue::Fixed64(value) => Value::Fixed64(*value),
        ResourceValue::Unit(value) => Value::Unit(*value),
        ResourceValue::String(value) => Value::String(value.clone()),
        ResourceValue::ResourceHandle(value) => Value::ResourceHandle(*value),
        ResourceValue::List(items) => Value::List(
            items
                .iter()
                .map(value_from_resource_value)
                .collect::<Vec<_>>(),
        ),
        ResourceValue::Set(items) => {
            let mut converted = BTreeMap::new();
            for (key, value) in items {
                converted.insert(key.clone(), value_from_resource_value(value));
            }
            Value::Set(converted)
        }
        ResourceValue::Map(entries) => {
            let mut converted = BTreeMap::new();
            for (key, entry) in entries {
                converted.insert(
                    key.clone(),
                    MapEntry {
                        key: value_from_resource_value(&entry.key),
                        value: value_from_resource_value(&entry.value),
                    },
                );
            }
            Value::Map(converted)
        }
    }
}

fn escape_canon_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

fn value_canon(value: &Value) -> String {
    match value {
        Value::None => "없음".to_string(),
        Value::Bool(true) => "참".to_string(),
        Value::Bool(false) => "거짓".to_string(),
        Value::Fixed64(n) => n.to_string(),
        Value::Unit(unit) => {
            let suffix = unit
                .display_symbol()
                .map(|s| s.to_string())
                .unwrap_or_else(|| unit.dim.format());
            format!("{}@{}", unit.value, suffix)
        }
        Value::String(s) => format!("\"{}\"", escape_canon_string(s)),
        Value::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
        Value::List(items) => {
            let mut out = String::from("차림[");
            let mut first = true;
            for item in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&value_canon(item));
            }
            out.push(']');
            out
        }
        Value::Set(items) => {
            let mut out = String::from("모음{");
            let mut first = true;
            for (_canon_key, value) in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&value_canon(value));
            }
            out.push('}');
            out
        }
        Value::Map(entries) => {
            let mut out = String::from("짝맞춤{");
            let mut first = true;
            for (_canon_key, entry) in entries {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&value_canon(&entry.key));
                out.push_str("=>");
                out.push_str(&value_canon(&entry.value));
            }
            out.push('}');
            out
        }
        Value::Pack(items) => {
            let mut out = String::from("묶음{");
            let mut first = true;
            for (key, value) in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(key);
                out.push('=');
                out.push_str(&value_canon(value));
            }
            out.push('}');
            out
        }
        Value::Formula(formula) => formula.raw.clone(),
        Value::Template(template) => template.raw.clone(),
        Value::Lambda(lambda) => format!("<씨앗#{}>", lambda.id),
    }
}

fn render_template(
    template: &ddonirang_lang::Template,
    pack: &BTreeMap<String, Value>,
) -> Result<String, EvalError> {
    let mut required = BTreeSet::new();
    for part in &template.parts {
        let TemplatePart::Placeholder(placeholder) = part else {
            continue;
        };
        if let Some(root) = placeholder.path.first() {
            required.insert(root.clone());
        }
    }
    let provided: BTreeSet<String> = pack.keys().cloned().collect();
    let missing: Vec<String> = required.difference(&provided).cloned().collect();
    if !missing.is_empty() {
        return Err(format!("채우기: 주입 키가 누락되었습니다: {}", missing.join(", ")).into());
    }
    let extra: Vec<String> = provided.difference(&required).cloned().collect();
    if !extra.is_empty() {
        return Err(format!("채우기: 주입 키가 여분입니다: {}", extra.join(", ")).into());
    }
    let mut out = String::new();
    for part in &template.parts {
        match part {
            TemplatePart::Text(text) => out.push_str(text),
            TemplatePart::Placeholder(placeholder) => {
                let value = resolve_template_value(pack, &placeholder.path)?;
                let rendered = format_template_value(value, placeholder.format.as_ref())?;
                out.push_str(&rendered);
            }
        }
    }
    Ok(out)
}

fn resolve_template_value<'a>(
    pack: &'a BTreeMap<String, Value>,
    path: &[String],
) -> Result<&'a Value, EvalError> {
    let Some(root) = path.first() else {
        return Err("글무늬 자리표시자 경로가 비었습니다".to_string().into());
    };
    let mut current = pack
        .get(root)
        .ok_or_else(|| format!("채우기: 키 '{}'가 없습니다", root))?;
    if matches!(current, Value::None) {
        return Err(format!("채우기: 키 '{}' 값이 없습니다", root).into());
    }
    let mut current_path = root.clone();
    for seg in path.iter().skip(1) {
        let next_path = format!("{}.{}", current_path, seg);
        let Value::Pack(map) = current else {
            return Err(format!("채우기: '{}'는 묶음이 아닙니다", current_path).into());
        };
        current = map
            .get(seg)
            .ok_or_else(|| format!("채우기: 필드 '{}'가 없습니다", next_path))?;
        if matches!(current, Value::None) {
            return Err(format!("채우기: 필드 '{}' 값이 없습니다", next_path).into());
        }
        current_path = next_path;
    }
    Ok(current)
}

fn format_template_value(value: &Value, format: Option<&TemplateFormat>) -> Result<String, EvalError> {
    let Some(format) = format else {
        if matches!(value, Value::None) {
            return Err("채우기: 자리표시자 값이 없습니다".to_string().into());
        }
        return Ok(value_to_string(value));
    };
    let (number, suffix) = match value {
        Value::Fixed64(v) => {
            if format.unit.is_some() {
                return Err("글무늬 포맷 단위는 단위 값에만 적용됩니다".to_string().into());
            }
            (*v, None)
        }
        Value::Unit(unit) => {
            if let Some(unit_str) = &format.unit {
                let Some(spec) = unit_spec_from_symbol(unit_str) else {
                    return Err(format!("글무늬 포맷 단위 '{}'를 알 수 없습니다", unit_str).into());
                };
                let converted = unit.to_unit(spec).map_err(unit_error)?;
                (converted, Some(unit_str.clone()))
            } else {
                let symbol = unit
                    .display_symbol()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| unit.dim.format());
                (unit.value, Some(symbol))
            }
        }
        Value::None => {
            return Err("채우기: 자리표시자 값이 없습니다".to_string().into());
        }
        _ => {
            return Err("글무늬 포맷은 수치에만 적용됩니다".to_string().into());
        }
    };
    let precision = format.precision.unwrap_or(0);
    let mut out = format_fixed64_with_precision(number, precision);
    if let Some(width) = format.width {
        out = apply_width(out, width, format.zero_pad);
    }
    if let Some(unit) = suffix {
        out.push('@');
        out.push_str(&unit);
    }
    Ok(out)
}

fn format_fixed64_with_precision(value: Fixed64, precision: u8) -> String {
    let raw = value.raw_i64();
    let negative = raw < 0;
    let abs_raw = if raw == i64::MIN {
        (i64::MAX as i128) + 1
    } else {
        raw.abs() as i128
    };
    let scale = 10i128.pow(precision as u32);
    let den = 1i128 << 32;
    let num = abs_raw * scale;
    let mut q = num / den;
    let r = num % den;
    let half = den / 2;
    if r > half || (r == half && (q & 1) == 1) {
        q += 1;
    }
    let int_part = q / scale;
    let frac_part = q % scale;
    let mut out = if precision == 0 {
        int_part.to_string()
    } else {
        format!(
            "{}.{:0width$}",
            int_part,
            frac_part,
            width = precision as usize
        )
    };
    if negative {
        out.insert(0, '-');
    }
    out
}

fn apply_width(value: String, width: usize, zero_pad: bool) -> String {
    let len = value.len();
    if len >= width {
        return value;
    }
    let pad_len = width - len;
    if zero_pad {
        if value.starts_with('-') {
            let digits = value[1..].to_string();
            format!("-{}{}", "0".repeat(pad_len), digits)
        } else {
            format!("{}{}", "0".repeat(pad_len), value)
        }
    } else {
        format!("{}{}", " ".repeat(pad_len), value)
    }
}

#[derive(Debug, Clone)]
struct ParsedFormula {
    expr: FormulaExpr,
    vars: BTreeSet<String>,
}

#[derive(Default)]
struct FormulaTransformOptions {
    var_name: Option<String>,
    order: Option<i64>,
    include_const: Option<bool>,
}

struct FormulaAnalysis {
    assign_name: Option<String>,
    expr: FormulaExpr,
    vars: BTreeSet<String>,
}

#[derive(Debug, Clone)]
enum FormulaExpr {
    Number(UnitValue),
    Var(String),
    Func { name: String, args: Vec<FormulaExpr> },
    Unary { op: FormulaOp, expr: Box<FormulaExpr> },
    Binary { op: FormulaOp, left: Box<FormulaExpr>, right: Box<FormulaExpr> },
}

#[derive(Debug, Clone, Copy)]
enum FormulaOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone)]
enum FormulaToken {
    Number(Fixed64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Eq,
    Comma,
}

struct FormulaParser {
    tokens: Vec<FormulaToken>,
    pos: usize,
    dialect: FormulaDialect,
    vars: BTreeSet<String>,
}

impl FormulaParser {
    fn new(tokens: Vec<FormulaToken>, dialect: FormulaDialect) -> Self {
        Self {
            tokens,
            pos: 0,
            dialect,
            vars: BTreeSet::new(),
        }
    }

    fn parse_expr(&mut self) -> Result<FormulaExpr, EvalError> {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Result<FormulaExpr, EvalError> {
        let mut expr = self.parse_mul_div()?;
        loop {
            if self.consume(&FormulaToken::Plus) {
                let rhs = self.parse_mul_div()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Add,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            if self.consume(&FormulaToken::Minus) {
                let rhs = self.parse_mul_div()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Sub,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_mul_div(&mut self) -> Result<FormulaExpr, EvalError> {
        let mut expr = self.parse_pow()?;
        loop {
            if self.consume(&FormulaToken::Star) {
                let rhs = self.parse_pow()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Mul,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            if self.consume(&FormulaToken::Slash) {
                let rhs = self.parse_pow()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Div,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            if self.consume(&FormulaToken::Percent) {
                let rhs = self.parse_pow()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Mod,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            if matches!(self.dialect, FormulaDialect::Ascii1) && self.next_starts_factor() {
                let rhs = self.parse_pow()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Mul,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_pow(&mut self) -> Result<FormulaExpr, EvalError> {
        let mut expr = self.parse_unary()?;
        if self.consume(&FormulaToken::Caret) {
            let rhs = self.parse_pow()?;
            expr = FormulaExpr::Binary {
                op: FormulaOp::Pow,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<FormulaExpr, EvalError> {
        if self.consume(&FormulaToken::Plus) {
            return self.parse_unary();
        }
        if self.consume(&FormulaToken::Minus) {
            let expr = self.parse_unary()?;
            return Ok(FormulaExpr::Unary {
                op: FormulaOp::Sub,
                expr: Box::new(expr),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<FormulaExpr, EvalError> {
        let Some(token) = self.next() else {
            return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
        };
        match token {
            FormulaToken::Number(value) => Ok(FormulaExpr::Number(UnitValue {
                value,
                dim: UnitDim::NONE,
            })),
            FormulaToken::Ident(name) => {
                if self.consume(&FormulaToken::LParen) {
                    if self.consume(&FormulaToken::RParen) {
                        return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
                    }
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_expr()?);
                        if self.consume(&FormulaToken::Comma) {
                            continue;
                        }
                        if self.consume(&FormulaToken::RParen) {
                            break;
                        }
                        return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
                    }
                    Ok(FormulaExpr::Func { name, args })
                } else {
                    self.vars.insert(name.clone());
                    Ok(FormulaExpr::Var(name))
                }
            }
            FormulaToken::LParen => {
                let expr = self.parse_expr()?;
                if !self.consume(&FormulaToken::RParen) {
                    return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
                }
                Ok(expr)
            }
            _ => Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string())),
        }
    }

    fn consume(&mut self, expected: &FormulaToken) -> bool {
        if let Some(token) = self.peek() {
            if std::mem::discriminant(token) == std::mem::discriminant(expected) {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    fn next(&mut self) -> Option<FormulaToken> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn peek(&self) -> Option<&FormulaToken> {
        self.tokens.get(self.pos)
    }

    fn next_starts_factor(&self) -> bool {
        matches!(
            self.peek(),
            Some(FormulaToken::Number(_))
                | Some(FormulaToken::Ident(_))
                | Some(FormulaToken::LParen)
        )
    }
}

fn parse_formula_value(formula: &Formula) -> Result<ParsedFormula, EvalError> {
    let tokens = tokenize_formula(&formula.raw, &formula.dialect)?;
    let eq_index = find_top_level_eq(&tokens)?;
    let (expr_tokens, vars) = if let Some(eq_index) = eq_index {
        if eq_index == 0 || eq_index + 1 >= tokens.len() {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        }
        let lhs = &tokens[..eq_index];
        let rhs = tokens[eq_index + 1..].to_vec();
        if lhs.len() != 1 {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        }
        match &lhs[0] {
            FormulaToken::Ident(_) => {}
            _ => {
                return Err(EvalError::Message(
                    "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
                ))
            }
        }
        (rhs, BTreeSet::new())
    } else {
        (tokens, BTreeSet::new())
    };
    let mut parser = FormulaParser::new(expr_tokens, formula.dialect.clone());
    parser.vars = vars;
    let expr = parser.parse_expr()?;
    if parser.pos != parser.tokens.len() {
        return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
    }
    Ok(ParsedFormula {
        expr,
        vars: parser.vars,
    })
}

fn parse_formula_with_vars(
    body: &str,
    dialect: &FormulaDialect,
) -> Result<(Option<String>, FormulaExpr, BTreeSet<String>), EvalError> {
    let tokens = tokenize_formula(body, dialect)?;
    let eq_index = find_top_level_eq(&tokens)?;
    let (expr_tokens, assign_name) = if let Some(eq_index) = eq_index {
        if eq_index == 0 || eq_index + 1 >= tokens.len() {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        }
        let lhs = &tokens[..eq_index];
        if lhs.len() != 1 {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        }
        let FormulaToken::Ident(name) = &lhs[0] else {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        };
        (tokens[eq_index + 1..].to_vec(), Some(name.clone()))
    } else {
        (tokens, None)
    };
    let mut parser = FormulaParser::new(expr_tokens, dialect.clone());
    let expr = parser.parse_expr()?;
    if parser.pos != parser.tokens.len() {
        return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
    }
    Ok((assign_name, expr, parser.vars))
}

fn analyze_formula_for_transform(formula: &Formula) -> Result<FormulaAnalysis, EvalError> {
    let (assign_name, expr, mut vars) = parse_formula_with_vars(&formula.raw, &formula.dialect)?;
    if let Some(name) = &assign_name {
        vars.remove(name);
    }
    Ok(FormulaAnalysis {
        assign_name,
        expr,
        vars,
    })
}

fn format_formula_body(body: &str, dialect: &FormulaDialect) -> Result<String, EvalError> {
    let (assign_name, expr, _) = parse_formula_with_vars(body, dialect)?;
    let expr_text = format_formula_expr(&expr, 0);
    if let Some(assign) = assign_name {
        Ok(format!("{} = {}", assign, expr_text))
    } else {
        Ok(expr_text)
    }
}

fn format_formula_expr(expr: &FormulaExpr, parent_prec: u8) -> String {
    match expr {
        FormulaExpr::Number(value) => value.value.to_string(),
        FormulaExpr::Var(name) => name.clone(),
        FormulaExpr::Func { name, args } => {
            let rendered = args
                .iter()
                .map(|arg| format_formula_expr(arg, 0))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, rendered)
        }
        FormulaExpr::Unary { op: FormulaOp::Sub, expr } => {
            let inner = format_formula_expr(expr, 3);
            format!("-{}", inner)
        }
        FormulaExpr::Unary { expr, .. } => format_formula_expr(expr, parent_prec),
        FormulaExpr::Binary { op, left, right } => {
            let (prec, op_str, spaced, right_assoc) = match op {
                FormulaOp::Add => (1, "+", true, false),
                FormulaOp::Sub => (1, "-", true, false),
                FormulaOp::Mul => (2, "*", false, false),
                FormulaOp::Div => (2, "/", false, false),
                FormulaOp::Mod => (2, "%", false, false),
                FormulaOp::Pow => (3, "^", false, true),
            };
            let (left_prec, right_prec) = if right_assoc {
                (prec + 1, prec)
            } else {
                (prec, prec + 1)
            };
            let left_str = format_formula_expr(left, left_prec);
            let right_str = format_formula_expr(right, right_prec);
            let joined = if spaced {
                format!("{} {} {}", left_str, op_str, right_str)
            } else {
                format!("{}{}{}", left_str, op_str, right_str)
            };
            if prec < parent_prec {
                format!("({})", joined)
            } else {
                joined
            }
        }
    }
}

fn expect_formula_transform(
    values: &[Value],
    label: &'static str,
) -> Result<(Formula, FormulaTransformOptions), EvalError> {
    if values.is_empty() || values.len() > 2 {
        return Err(EvalError::Message(
            "formula[, variable|options]".to_string(),
        ));
    }
    let formula = match &values[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(EvalError::Message(format!(
                "{}는 수식값 인자가 필요합니다",
                label
            )))
        }
    };
    if values.len() == 1 {
        return Ok((formula, FormulaTransformOptions::default()));
    }
    let options = parse_formula_transform_arg(&values[1], label)?;
    Ok((formula, options))
}

fn parse_formula_transform_arg(
    value: &Value,
    label: &'static str,
) -> Result<FormulaTransformOptions, EvalError> {
    match value {
        Value::String(_) => Ok(FormulaTransformOptions {
            var_name: Some(parse_formula_var_name(value, label)?),
            ..FormulaTransformOptions::default()
        }),
        Value::Pack(pack) => parse_formula_transform_pack(pack, label),
        _ => Err(EvalError::Message("string variable or pack options".to_string())),
    }
}

fn parse_formula_transform_pack(
    pack: &BTreeMap<String, Value>,
    label: &'static str,
) -> Result<FormulaTransformOptions, EvalError> {
    let mut options = FormulaTransformOptions::default();
    for key in pack.keys() {
        match key.as_str() {
            "변수" | "차수" | "상수포함" => {}
            _ => {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: {} 옵션을 지원하지 않습니다",
                    key
                )))
            }
        }
    }

    if let Some(value) = pack.get("변수") {
        if !matches!(value, Value::None) {
            options.var_name = Some(parse_formula_var_name(value, label)?);
        }
    }
    if let Some(value) = pack.get("차수") {
        if !matches!(value, Value::None) {
            let order = expect_int_strict(value)?;
            if order < 1 {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_BAD_ORDER: {} 차수는 1 이상이어야 합니다",
                    label
                )));
            }
            options.order = Some(order);
        }
    }
    if let Some(value) = pack.get("상수포함") {
        if !matches!(value, Value::None) {
            let include = match value {
                Value::Bool(flag) => *flag,
                _ => {
                    return Err(EvalError::Message(
                        "상수포함은 참/거짓이어야 합니다".to_string(),
                    ))
                }
            };
            options.include_const = Some(include);
        }
    }
    Ok(options)
}

fn parse_formula_var_name(value: &Value, label: &'static str) -> Result<String, EvalError> {
    let raw = match value {
        Value::String(text) => text.clone(),
        _ => {
            return Err(EvalError::Message(format!(
                "{} 변수 이름이 글이어야 합니다",
                label
            )))
        }
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(EvalError::Message(format!(
            "{} 변수 이름이 비어 있습니다",
            label
        )));
    }
    Ok(trimmed.trim_start_matches('#').to_string())
}

fn expect_int_strict(value: &Value) -> Result<i64, EvalError> {
    match value {
        Value::Fixed64(n) => {
            if n.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(n.int_part())
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            if unit.value.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(unit.value.int_part())
        }
        _ => Err("정수 값이 필요합니다".to_string().into()),
    }
}

fn transform_formula_value(
    formula: Formula,
    options: FormulaTransformOptions,
    call_name: &'static str,
    label: &'static str,
) -> Result<Formula, EvalError> {
    let dialect = formula.dialect.clone();
    if !matches!(dialect, FormulaDialect::Ascii) {
        return Err(EvalError::Message(format!(
            "{}는 #ascii 수식만 지원합니다",
            label
        )));
    }

    let analysis = analyze_formula_for_transform(&formula)?;
    let vars = analysis.vars.clone();
    let var_name = match options.var_name {
        Some(name) => {
            if !vars.contains(&name) {
                return Err(EvalError::Message(format!(
                    "E_CALC_FREEVAR_NOT_FOUND: {} 변수 '{}'가 수식에 없습니다",
                    label, name
                )));
            }
            name
        }
        None => infer_single_var(&vars, label)?,
    };
    ensure_formula_ident(&var_name, &dialect, label)?;

    if call_name == "diff" && options.include_const.is_some() {
        return Err(EvalError::Message(
            "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: 미분하기는 상수포함을 지원하지 않습니다"
                .to_string(),
        ));
    }
    if call_name == "int" && options.order.is_some() {
        return Err(EvalError::Message(
            "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: 적분하기는 차수를 지원하지 않습니다"
                .to_string(),
        ));
    }

    let expr = if call_name == "diff" {
        let order = options.order.unwrap_or(1);
        let mut out = analysis.expr.clone();
        for _ in 0..order {
            out = diff_formula_expr(&out, &var_name, label)?;
        }
        out
    } else {
        integrate_formula_expr(&analysis.expr, &var_name, label)?
    };

    let mut expr_text = format_formula_expr(&expr, 0);
    if call_name == "int" && options.include_const.unwrap_or(false) {
        expr_text = format!("{} + C", expr_text);
    }

    let body = if let Some(assign) = analysis.assign_name {
        format!("{} = {}", assign, expr_text)
    } else {
        expr_text
    };
    let formatted = format_formula_body(&body, &dialect)?;
    Ok(Formula {
        raw: formatted,
        dialect,
        explicit_tag: formula.explicit_tag,
    })
}

fn diff_formula_expr(expr: &FormulaExpr, var: &str, label: &'static str) -> Result<FormulaExpr, EvalError> {
    use FormulaExpr::*;
    use FormulaOp::*;

    let d = match expr {
        Number(_) => num_zero(),
        Var(name) => {
            if name == var {
                num_one()
            } else {
                num_zero()
            }
        }
        Unary { op: Sub, expr } => unary_sub(diff_formula_expr(expr, var, label)?),
        Unary { expr, .. } => diff_formula_expr(expr, var, label)?,
        Binary { op: Add, left, right } => add(
            diff_formula_expr(left, var, label)?,
            diff_formula_expr(right, var, label)?,
        ),
        Binary { op: Sub, left, right } => sub(
            diff_formula_expr(left, var, label)?,
            diff_formula_expr(right, var, label)?,
        ),
        Binary { op: Mul, left, right } => {
            let dl = diff_formula_expr(left, var, label)?;
            let dr = diff_formula_expr(right, var, label)?;
            add(mul(dl, right.as_ref().clone()), mul(left.as_ref().clone(), dr))
        }
        Binary { op: Div, left, right } => {
            let dl = diff_formula_expr(left, var, label)?;
            let dr = diff_formula_expr(right, var, label)?;
            let num = sub(mul(dl, right.as_ref().clone()), mul(left.as_ref().clone(), dr));
            let denom = pow(right.as_ref().clone(), num_i(2));
            div(num, denom)
        }
        Binary { op: Pow, left, right } => {
            if let Some(exp) = number_as_int(right) {
                if exp == 0 {
                    num_zero()
                } else {
                    let coeff = num_i(exp);
                    let pow_expr = pow(left.as_ref().clone(), num_i(exp - 1));
                    let dl = diff_formula_expr(left, var, label)?;
                    mul(coeff, mul(pow_expr, dl))
                }
            } else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 거듭제곱 미분을 지원하지 않습니다",
                    label
                )));
            }
        }
        Binary { op: Mod, .. } => {
            return Err(EvalError::Message(format!(
                "E_CALC_TRANSFORM_UNSUPPORTED: {}는 나머지 미분을 지원하지 않습니다",
                label
            )));
        }
        Func { name, args } => {
            let Some(first) = args.first() else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 빈 함수 미분을 지원하지 않습니다",
                    label
                )));
            };
            let du = diff_formula_expr(first, var, label)?;
            match name.as_str() {
                "sin" => mul(func("cos", vec![first.clone()]), du),
                "cos" => mul(unary_sub(func("sin", vec![first.clone()])), du),
                "tan" => {
                    let cos_u = func("cos", vec![first.clone()]);
                    let denom = pow(cos_u, num_i(2));
                    div(du, denom)
                }
                "exp" => mul(func("exp", vec![first.clone()]), du),
                "ln" => div(du, first.clone()),
                "log10" => {
                    let denom = mul(first.clone(), num_ln10());
                    div(du, denom)
                }
                "log2" => {
                    let denom = mul(first.clone(), num_ln2());
                    div(du, denom)
                }
                "sqrt" => {
                    let denom = mul(num_i(2), func("sqrt", vec![first.clone()]));
                    div(du, denom)
                }
                _ => {
                    return Err(EvalError::Message(format!(
                        "E_CALC_TRANSFORM_UNSUPPORTED: {}는 함수 '{}' 미분을 지원하지 않습니다",
                        label, name
                    )))
                }
            }
        }
    };
    Ok(d)
}

fn integrate_formula_expr(expr: &FormulaExpr, var: &str, label: &'static str) -> Result<FormulaExpr, EvalError> {
    use FormulaExpr::*;
    use FormulaOp::*;

    if !expr_contains_var(expr, var) {
        return Ok(mul(expr.clone(), Var(var.to_string())));
    }

    let out = match expr {
        Number(_) => mul(expr.clone(), Var(var.to_string())),
        Var(name) => {
            if name == var {
                div(pow(Var(var.to_string()), num_i(2)), num_i(2))
            } else {
                mul(expr.clone(), Var(var.to_string()))
            }
        }
        Unary { op: Sub, expr } => unary_sub(integrate_formula_expr(expr, var, label)?),
        Unary { expr, .. } => integrate_formula_expr(expr, var, label)?,
        Binary { op: Add, left, right } => add(
            integrate_formula_expr(left, var, label)?,
            integrate_formula_expr(right, var, label)?,
        ),
        Binary { op: Sub, left, right } => sub(
            integrate_formula_expr(left, var, label)?,
            integrate_formula_expr(right, var, label)?,
        ),
        Binary { op: Mul, left, right } => {
            if !expr_contains_var(left, var) {
                mul(left.as_ref().clone(), integrate_formula_expr(right, var, label)?)
            } else if !expr_contains_var(right, var) {
                mul(right.as_ref().clone(), integrate_formula_expr(left, var, label)?)
            } else if let Some(exp) = detect_var_power(expr, var) {
                let next = exp + 1;
                let num = pow(Var(var.to_string()), num_i(next));
                div(num, num_i(next))
            } else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 곱셈 적분을 지원하지 않습니다",
                    label
                )));
            }
        }
        Binary { op: Pow, left, right } => {
            if let (FormulaExpr::Var(name), Some(exp)) = (&**left, number_as_int(right)) {
                if name == var && exp != -1 {
                    let next = exp + 1;
                    let num = pow(Var(var.to_string()), num_i(next));
                    div(num, num_i(next))
                } else {
                    return Err(EvalError::Message(format!(
                        "E_CALC_TRANSFORM_UNSUPPORTED: {}는 거듭제곱 적분을 지원하지 않습니다",
                        label
                    )));
                }
            } else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 거듭제곱 적분을 지원하지 않습니다",
                    label
                )));
            }
        }
        Binary { op: Div, .. } | Binary { op: Mod, .. } => {
            return Err(EvalError::Message(format!(
                "E_CALC_TRANSFORM_UNSUPPORTED: {}는 나눗셈 적분을 지원하지 않습니다",
                label
            )));
        }
        Func { name, args } => {
            let Some(first) = args.first() else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 빈 함수 적분을 지원하지 않습니다",
                    label
                )));
            };
            match name.as_str() {
                "sin" => unary_sub(func("cos", vec![first.clone()])),
                "cos" => func("sin", vec![first.clone()]),
                "exp" => func("exp", vec![first.clone()]),
                _ => {
                    return Err(EvalError::Message(format!(
                        "E_CALC_TRANSFORM_UNSUPPORTED: {}는 함수 '{}' 적분을 지원하지 않습니다",
                        label, name
                    )))
                }
            }
        }
    };
    Ok(out)
}

fn expr_contains_var(expr: &FormulaExpr, var: &str) -> bool {
    match expr {
        FormulaExpr::Var(name) => name == var,
        FormulaExpr::Number(_) => false,
        FormulaExpr::Func { args, .. } => args.iter().any(|arg| expr_contains_var(arg, var)),
        FormulaExpr::Unary { expr, .. } => expr_contains_var(expr, var),
        FormulaExpr::Binary { left, right, .. } => expr_contains_var(left, var) || expr_contains_var(right, var),
    }
}

fn detect_var_power(expr: &FormulaExpr, var: &str) -> Option<i64> {
    match expr {
        FormulaExpr::Var(name) if name == var => Some(1),
        FormulaExpr::Binary { op: FormulaOp::Pow, left, right } => {
            if let FormulaExpr::Var(name) = &**left {
                if name == var {
                    return number_as_int(right);
                }
            }
            None
        }
        FormulaExpr::Binary { op: FormulaOp::Mul, left, right } => {
            if let (Some(a), Some(b)) = (detect_var_power(left, var), detect_var_power(right, var)) {
                return Some(a + b);
            }
            if let (Some(a), FormulaExpr::Var(name)) = (detect_var_power(left, var), &**right) {
                if name == var {
                    return Some(a + 1);
                }
            }
            if let (FormulaExpr::Var(name), Some(b)) = (&**left, detect_var_power(right, var)) {
                if name == var {
                    return Some(b + 1);
                }
            }
            None
        }
        _ => None,
    }
}

fn number_as_int(expr: &FormulaExpr) -> Option<i64> {
    match expr {
        FormulaExpr::Number(value) if value.dim == UnitDim::NONE => {
            if value.value.frac_part() == 0 {
                Some(value.value.int_part())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn num_zero() -> FormulaExpr {
    FormulaExpr::Number(UnitValue { value: Fixed64::from_i64(0), dim: UnitDim::NONE })
}

fn num_one() -> FormulaExpr {
    FormulaExpr::Number(UnitValue { value: Fixed64::from_i64(1), dim: UnitDim::NONE })
}

fn num_i(value: i64) -> FormulaExpr {
    FormulaExpr::Number(UnitValue { value: Fixed64::from_i64(value), dim: UnitDim::NONE })
}

fn num_ln10() -> FormulaExpr {
    FormulaExpr::Number(UnitValue { value: Fixed64::from_f64_lossy(libm::log(10.0)), dim: UnitDim::NONE })
}

fn num_ln2() -> FormulaExpr {
    FormulaExpr::Number(UnitValue { value: Fixed64::from_f64_lossy(libm::log(2.0)), dim: UnitDim::NONE })
}

fn func(name: &str, args: Vec<FormulaExpr>) -> FormulaExpr {
    FormulaExpr::Func { name: name.to_string(), args }
}

fn unary_sub(expr: FormulaExpr) -> FormulaExpr {
    FormulaExpr::Unary { op: FormulaOp::Sub, expr: Box::new(expr) }
}

fn add(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) { return right; }
    if is_zero(&right) { return left; }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.add(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary { op: FormulaOp::Add, left: Box::new(left), right: Box::new(right) }
}

fn sub(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&right) { return left; }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.sub(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary { op: FormulaOp::Sub, left: Box::new(left), right: Box::new(right) }
}

fn mul(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) || is_zero(&right) { return num_zero(); }
    if is_one(&left) { return right; }
    if is_one(&right) { return left; }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        return FormulaExpr::Number(l.mul(r));
    }
    FormulaExpr::Binary { op: FormulaOp::Mul, left: Box::new(left), right: Box::new(right) }
}

fn div(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) { return num_zero(); }
    if is_one(&right) { return left; }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.div(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary { op: FormulaOp::Div, left: Box::new(left), right: Box::new(right) }
}

fn pow(base: FormulaExpr, exp: FormulaExpr) -> FormulaExpr {
    if let Some(e) = number_as_int(&exp) {
        if e == 0 { return num_one(); }
        if e == 1 { return base; }
    }
    FormulaExpr::Binary { op: FormulaOp::Pow, left: Box::new(base), right: Box::new(exp) }
}

fn as_number(expr: &FormulaExpr) -> Option<UnitValue> {
    if let FormulaExpr::Number(value) = expr { Some(*value) } else { None }
}

fn is_zero(expr: &FormulaExpr) -> bool {
    matches!(expr, FormulaExpr::Number(value) if value.value.raw_i64() == 0)
}

fn is_one(expr: &FormulaExpr) -> bool {
    matches!(expr, FormulaExpr::Number(value) if value.dim == UnitDim::NONE && value.value.raw_i64() == Fixed64::from_i64(1).raw_i64())
}

fn infer_single_var(vars: &BTreeSet<String>, label: &'static str) -> Result<String, EvalError> {
    match vars.len() {
        1 => Ok(vars.iter().next().unwrap().to_string()),
        0 => Err(EvalError::Message(format!(
            "E_CALC_FREEVAR_AMBIGUOUS: {}는 변수 이름을 지정해야 합니다",
            label
        ))),
        _ => Err(EvalError::Message(format!(
            "E_CALC_FREEVAR_AMBIGUOUS: {} 변수 이름이 여러 개입니다",
            label
        ))),
    }
}

fn ensure_formula_ident(
    name: &str,
    dialect: &FormulaDialect,
    label: &'static str,
) -> Result<(), EvalError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(EvalError::Message(format!(
            "{} 변수 이름이 비어 있습니다",
            label
        )));
    };
    if !first.is_ascii_alphabetic() {
        return Err(EvalError::Message(format!(
            "{} 변수 이름이 올바르지 않습니다: {}",
            label, name
        )));
    }
    let valid = match dialect {
        FormulaDialect::Ascii => chars.all(|ch| ch.is_ascii_alphanumeric()),
        FormulaDialect::Ascii1 => chars.all(|ch| ch.is_ascii_digit()),
        _ => false,
    };
    if !valid {
        return Err(EvalError::Message(format!(
            "{} 변수 이름이 올바르지 않습니다: {}",
            label, name
        )));
    }
    Ok(())
}

fn tokenize_formula(body: &str, dialect: &FormulaDialect) -> Result<Vec<FormulaToken>, EvalError> {
    let mut tokens = Vec::new();
    let mut chars = body.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        match ch {
            '0'..='9' => {
                let mut raw = String::new();
                let mut seen_dot = false;
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        raw.push(next);
                        chars.next();
                        continue;
                    }
                    if next == '.' && !seen_dot {
                        seen_dot = true;
                        raw.push(next);
                        chars.next();
                        continue;
                    }
                    break;
                }
                let value = raw
                    .parse::<f64>()
                    .map_err(|_| EvalError::Message("FATAL:FORMULA_TOKEN_INVALID".to_string()))?;
                tokens.push(FormulaToken::Number(Fixed64::from_f64_lossy(value)));
            }
            'a'..='z' | 'A'..='Z' => {
                let ident = read_formula_ident(&mut chars, dialect)?;
                tokens.push(FormulaToken::Ident(ident));
            }
            '+' => {
                chars.next();
                tokens.push(FormulaToken::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(FormulaToken::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(FormulaToken::Star);
            }
            '/' => {
                chars.next();
                tokens.push(FormulaToken::Slash);
            }
            '%' => {
                chars.next();
                tokens.push(FormulaToken::Percent);
            }
            ',' => {
                chars.next();
                tokens.push(FormulaToken::Comma);
            }
            '^' => {
                chars.next();
                tokens.push(FormulaToken::Caret);
            }
            '(' => {
                chars.next();
                tokens.push(FormulaToken::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(FormulaToken::RParen);
            }
            '=' => {
                chars.next();
                tokens.push(FormulaToken::Eq);
            }
            _ => {
                return Err(EvalError::Message(
                    "FATAL:FORMULA_TOKEN_INVALID".to_string(),
                ))
            }
        }
    }
    Ok(tokens)
}

fn read_formula_ident<I>(
    chars: &mut std::iter::Peekable<I>,
    dialect: &FormulaDialect,
) -> Result<String, EvalError>
where
    I: Iterator<Item = char>,
{
    let Some(first) = chars.next() else {
        return Err(EvalError::Message("FATAL:FORMULA_TOKEN_INVALID".to_string()));
    };
    let mut ident = String::new();
    ident.push(first);
    if matches!(dialect, FormulaDialect::Ascii1) {
        while let Some(&next) = chars.peek() {
            if next.is_ascii_alphanumeric() {
                ident.push(next);
                chars.next();
                continue;
            }
            if next == '_' {
                return Err(EvalError::Message(
                    "FATAL:FORMULA_ASCII1_VAR".to_string(),
                ));
            }
            break;
        }
        return Ok(ident);
    }
    while let Some(&next) = chars.peek() {
        if next.is_ascii_alphanumeric() || next == '_' {
            ident.push(next);
            chars.next();
        } else {
            break;
        }
    }
    Ok(ident)
}

fn find_top_level_eq(tokens: &[FormulaToken]) -> Result<Option<usize>, EvalError> {
    let mut depth = 0i32;
    let mut eq_index = None;
    for (idx, token) in tokens.iter().enumerate() {
        match token {
            FormulaToken::LParen => depth += 1,
            FormulaToken::RParen => depth -= 1,
            FormulaToken::Eq if depth == 0 => {
                if eq_index.is_some() {
                    return Err(EvalError::Message(
                        "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
                    ));
                }
                eq_index = Some(idx);
            }
            _ => {}
        }
    }
    Ok(eq_index)
}

fn eval_formula_expr(
    expr: &FormulaExpr,
    env: &BTreeMap<String, Value>,
) -> Result<UnitValue, EvalError> {
    match expr {
        FormulaExpr::Number(value) => Ok(*value),
        FormulaExpr::Var(name) => {
            if let Some(value) = env.get(name) {
                return unit_value_from_value(value);
            }
            if name == "pi" {
                return Ok(UnitValue {
                    value: Fixed64::from_f64_lossy(std::f64::consts::PI),
                    dim: UnitDim::NONE,
                });
            }
            if name == "e" {
                return Ok(UnitValue {
                    value: Fixed64::from_f64_lossy(std::f64::consts::E),
                    dim: UnitDim::NONE,
                });
            }
            Err(EvalError::Message(format!("풀기: 키 '{}'가 없습니다", name)))
        }
        FormulaExpr::Func { name, args } => {
            let mut evaluated = Vec::with_capacity(args.len());
            for arg in args {
                evaluated.push(eval_formula_expr(arg, env)?);
            }
            eval_formula_func(name, evaluated)
        }
        FormulaExpr::Unary { op, expr } => {
            let value = eval_formula_expr(expr, env)?;
            match op {
                FormulaOp::Sub => Ok(UnitValue {
                    value: -value.value,
                    dim: value.dim,
                }),
                _ => Ok(value),
            }
        }
        FormulaExpr::Binary { op, left, right } => {
            let l = eval_formula_expr(left, env)?;
            let r = eval_formula_expr(right, env)?;
            match op {
                FormulaOp::Add => l.add(r).map_err(unit_error),
                FormulaOp::Sub => l.sub(r).map_err(unit_error),
                FormulaOp::Mul => Ok(l.mul(r)),
                FormulaOp::Div => l.div(r).map_err(unit_error),
                FormulaOp::Mod => {
                    if l.dim != r.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: l.dim,
                            right: r.dim,
                        }));
                    }
                    let raw_r = r.value.raw_i64();
                    if raw_r == 0 {
                        return Err(EvalError::Message("FATAL:FORMULA_MOD_ZERO".to_string()));
                    }
                    Ok(UnitValue {
                        value: Fixed64::from_raw_i64(l.value.raw_i64() % raw_r),
                        dim: l.dim,
                    })
                }
                FormulaOp::Pow => eval_formula_pow(l, r),
            }
        }
    }
}

fn eval_formula_func(name: &str, args: Vec<UnitValue>) -> Result<UnitValue, EvalError> {
    let expect_args = |expected: usize| -> Result<Vec<UnitValue>, EvalError> {
        if args.len() != expected {
            return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
        }
        Ok(args.clone())
    };

    match name {
        "sin" | "cos" | "tan" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !(arg.is_dimensionless() || arg.dim == UnitDim::ANGLE) {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::ANGLE,
                }));
            }
            let angle = fixed64_to_f64(arg.value);
            let out = match name {
                "sin" => libm::sin(angle),
                "cos" => libm::cos(angle),
                _ => libm::tan(angle),
            };
            return Ok(UnitValue {
                value: Fixed64::from_f64_lossy(out),
                dim: UnitDim::NONE,
            });
        }
        "asin" | "acos" | "atan" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            let out = match name {
                "asin" => libm::asin(value),
                "acos" => libm::acos(value),
                _ => libm::atan(value),
            };
            return Ok(UnitValue {
                value: Fixed64::from_f64_lossy(out),
                dim: UnitDim::NONE,
            });
        }
        "atan2" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let y = args[0];
            let x = args[1];
            if !y.is_dimensionless() || !x.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: y.dim,
                    right: UnitDim::NONE,
                }));
            }
            let out = libm::atan2(fixed64_to_f64(y.value), fixed64_to_f64(x.value));
            return Ok(UnitValue {
                value: Fixed64::from_f64_lossy(out),
                dim: UnitDim::NONE,
            });
        }
        "sinh" | "cosh" | "tanh" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            let out = match name {
                "sinh" => libm::sinh(value),
                "cosh" => libm::cosh(value),
                _ => libm::tanh(value),
            };
            return Ok(UnitValue {
                value: Fixed64::from_f64_lossy(out),
                dim: UnitDim::NONE,
            });
        }
        "asinh" | "acosh" | "atanh" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            let out = match name {
                "asinh" => libm::asinh(value),
                "acosh" => libm::acosh(value),
                _ => libm::atanh(value),
            };
            return Ok(UnitValue {
                value: Fixed64::from_f64_lossy(out),
                dim: UnitDim::NONE,
            });
        }
        "abs" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: fixed64_abs(arg.value),
                dim: arg.dim,
            });
        }
        "sign" => {
            let args = expect_args(1)?;
            let arg = args[0];
            let raw = arg.value.raw_i64();
            let sign = if raw > 0 { 1 } else if raw < 0 { -1 } else { 0 };
            return Ok(UnitValue {
                value: Fixed64::from_i64(sign),
                dim: UnitDim::NONE,
            });
        }
        "floor" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: fixed64_floor(arg.value),
                dim: arg.dim,
            });
        }
        "ceil" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: fixed64_ceil(arg.value),
                dim: arg.dim,
            });
        }
        "round" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: fixed64_round_even(arg.value),
                dim: arg.dim,
            });
        }
        "trunc" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: Fixed64::from_i64(arg.value.int_part()),
                dim: arg.dim,
            });
        }
        "fract" => {
            let args = expect_args(1)?;
            let arg = args[0];
            let floor = fixed64_floor(arg.value);
            let frac = arg.value - floor;
            return Ok(UnitValue {
                value: frac,
                dim: arg.dim,
            });
        }
        "sqrt" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let Some(value) = fixed64_sqrt(arg.value) else {
                return Err(EvalError::Message("FATAL:FORMULA_SQRT_INVALID".to_string()));
            };
            return Ok(UnitValue {
                value,
                dim: UnitDim::NONE,
            });
        }
        "cbrt" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            return Ok(UnitValue {
                value: Fixed64::from_f64_lossy(libm::cbrt(value)),
                dim: UnitDim::NONE,
            });
        }
        "powi" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let base = args[0];
            let exp = args[1];
            if !exp.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: exp.dim,
                    right: UnitDim::NONE,
                }));
            }
            let exp_i = exp.value.int_part();
            return unit_powi(base, exp_i);
        }
        "pow" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let base = args[0];
            let exp = args[1];
            if !exp.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: exp.dim,
                    right: UnitDim::NONE,
                }));
            }
            if exp.value.frac_part() == 0 {
                return unit_powi(base, exp.value.int_part());
            }
            if !base.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: base.dim,
                    right: UnitDim::NONE,
                }));
            }
            let out = libm::pow(fixed64_to_f64(base.value), fixed64_to_f64(exp.value));
            return Ok(UnitValue {
                value: Fixed64::from_f64_lossy(out),
                dim: UnitDim::NONE,
            });
        }
        "exp" | "ln" | "log10" | "log2" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            let out = match name {
                "exp" => libm::exp(value),
                "ln" => libm::log(value),
                "log10" => libm::log10(value),
                _ => libm::log2(value),
            };
            return Ok(UnitValue {
                value: Fixed64::from_f64_lossy(out),
                dim: UnitDim::NONE,
            });
        }
        "min" | "max" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let a = args[0];
            let b = args[1];
            if a.dim != b.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: a.dim,
                    right: b.dim,
                }));
            }
            let pick = if name == "min" {
                if a.value <= b.value { a } else { b }
            } else if a.value >= b.value {
                a
            } else {
                b
            };
            return Ok(pick);
        }
        "clamp" => {
            if args.len() != 3 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let x = args[0];
            let lo = args[1];
            let hi = args[2];
            if x.dim != lo.dim || x.dim != hi.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: x.dim,
                    right: lo.dim,
                }));
            }
            let mut out = x;
            if out.value < lo.value {
                out = lo;
            }
            if out.value > hi.value {
                out = hi;
            }
            return Ok(out);
        }
        "mod" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let a = args[0];
            let b = args[1];
            if a.dim != b.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: a.dim,
                    right: b.dim,
                }));
            }
            let raw_b = b.value.raw_i64();
            if raw_b == 0 {
                return Err(EvalError::Message("FATAL:FORMULA_MOD_ZERO".to_string()));
            }
            let raw = a.value.raw_i64() % raw_b;
            return Ok(UnitValue {
                value: Fixed64::from_raw_i64(raw),
                dim: a.dim,
            });
        }
        _ => {}
    }
    Err(EvalError::Message(format!(
        "FATAL:FORMULA_FUNC_UNKNOWN:{}",
        name
    )))
}

fn eval_formula_pow(base: UnitValue, exponent: UnitValue) -> Result<UnitValue, EvalError> {
    if exponent.dim != UnitDim::NONE {
        return Err(EvalError::Message(
            "FATAL:FORMULA_POW_INVALID".to_string(),
        ));
    }
    if exponent.value.frac_part() != 0 {
        return Err(EvalError::Message(
            "FATAL:FORMULA_POW_INVALID".to_string(),
        ));
    }
    let exp = exponent.value.int_part();
    if exp < 0 {
        return Err(EvalError::Message(
            "FATAL:FORMULA_POW_INVALID".to_string(),
        ));
    }
    if exp == 0 {
        return Ok(UnitValue {
            value: Fixed64::from_i64(1),
            dim: UnitDim::NONE,
        });
    }
    let mut result = base;
    for _ in 1..exp {
        result = result.mul(base);
    }
    Ok(result)
}

fn is_truthy(value: &Value) -> Result<bool, EvalError> {
    match value {
        Value::Bool(b) => Ok(*b),
        Value::Fixed64(n) => Ok(n.raw_i64() != 0),
        Value::Unit(unit) if unit.is_dimensionless() => Ok(unit.value.raw_i64() != 0),
        Value::Unit(_) => Err("단위 값은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::None => Ok(false),
        Value::String(_) => Err("글은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::ResourceHandle(_) => Err("자원은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::List(_) => Err("차림은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Set(_) => Err("모음은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Map(_) => Err("짝맞춤은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Pack(_) => Err("묶음은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Formula(_) => Err("수식은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Template(_) => Err("글무늬는 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Lambda(_) => Err("씨앗은 조건식에 사용할 수 없습니다".to_string().into()),
    }
}

fn runtime_error(err: RuntimeError) -> EvalError {
    EvalError::Message(runtime_error_message(err))
}

fn runtime_error_message(err: RuntimeError) -> String {
    match err {
        RuntimeError::TypeMismatch { expected } => format!("타입 오류: {}", expected),
        RuntimeError::IndexOutOfRange => "FATAL:CHARIM_INDEX_OUT_OF_RANGE".to_string(),
    }
}

fn type_mismatch_error(pin: &str, expected: &str, actual: &str) -> EvalError {
    EvalError::Message(format!(
        "[E_RUNTIME_TYPE_MISMATCH] 핀={} 기대={} 실제={}",
        pin, expected, actual
    ))
}

fn format_type_ref(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name) => canonical_type_name(name),
        TypeRef::Applied { name, args } => {
            let name = canonical_type_name(name);
            if name == "수" && args.len() == 1 {
                if let TypeRef::Named(unit) = &args[0] {
                    return format!("수@{}", unit);
                }
            }
            let args_out: Vec<String> = args.iter().map(format_type_ref).collect();
            format!("({}){}", args_out.join(", "), name)
        }
        TypeRef::Infer => "_".to_string(),
    }
}

fn canonical_type_name(name: &str) -> String {
    match name {
        "목록" => "차림".to_string(),
        other => other.to_string(),
    }
}

fn check_value_type(value: &Value, type_ref: &TypeRef) -> Result<(), TypeMismatchDetail> {
    match type_ref {
        TypeRef::Infer => Ok(()),
        TypeRef::Named(name) => check_named_type(value, name),
        TypeRef::Applied { name, args } => check_applied_type(value, name, args),
    }
}

fn check_named_type(value: &Value, name: &str) -> Result<(), TypeMismatchDetail> {
    if let Some((base, unit)) = name.split_once('@') {
        return check_unit_type(value, base, unit);
    }
    let canonical = canonical_type_name(name);
    match canonical.as_str() {
        "값" => Ok(()),
        "수" => match value {
            Value::Fixed64(_) => Ok(()),
            Value::Unit(unit) if unit.is_dimensionless() => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "정수" => match value {
            Value::Fixed64(n) if n.frac_part() == 0 => Ok(()),
            Value::Unit(unit) if unit.is_dimensionless() && unit.value.frac_part() == 0 => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "글" => match value {
            Value::String(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "참거짓" => match value {
            Value::Bool(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "없음" => match value {
            Value::None => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "차림" => match value {
            Value::List(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "모음" => match value {
            Value::Set(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "짝맞춤" => match value {
            Value::Map(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "묶음" => match value {
            Value::Pack(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "자원" | "자원핸들" => match value {
            Value::ResourceHandle(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "수식값" => match value {
            Value::Formula(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "글무늬값" => match value {
            Value::Template(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        _ => Err(type_mismatch_detail(&canonical, value)),
    }
}

fn check_applied_type(
    value: &Value,
    name: &str,
    args: &[TypeRef],
) -> Result<(), TypeMismatchDetail> {
    let canonical = canonical_type_name(name);
    match canonical.as_str() {
        "수" if args.len() == 1 => {
            if let TypeRef::Named(unit) = &args[0] {
                return check_unit_type(value, "수", unit);
            }
            Err(type_mismatch_detail("수", value))
        }
        "차림" => {
            let Value::List(items) = value else {
                return Err(type_mismatch_detail(&canonical, value));
            };
            if args.len() != 1 {
                return Ok(());
            }
            for item in items {
                if let Err(detail) = check_value_type(item, &args[0]) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "차림".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("차림(요소: {})", detail.actual),
                    });
                }
            }
            Ok(())
        }
        "모음" => {
            let Value::Set(items) = value else {
                return Err(type_mismatch_detail(&canonical, value));
            };
            if args.len() != 1 {
                return Ok(());
            }
            for item in items.values() {
                if let Err(detail) = check_value_type(item, &args[0]) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "모음".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("모음(요소: {})", detail.actual),
                    });
                }
            }
            Ok(())
        }
        "짝맞춤" => {
            let Value::Map(entries) = value else {
                return Err(type_mismatch_detail(&canonical, value));
            };
            if args.len() != 2 {
                return Ok(());
            }
            for entry in entries.values() {
                if let Err(detail) = check_value_type(&entry.key, &args[0]) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "짝맞춤".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("짝맞춤(열쇠: {})", detail.actual),
                    });
                }
                if let Err(detail) = check_value_type(&entry.value, &args[1]) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "짝맞춤".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("짝맞춤(값: {})", detail.actual),
                    });
                }
            }
            Ok(())
        }
        _ => Err(type_mismatch_detail(&canonical, value)),
    }
}

fn check_unit_type(
    value: &Value,
    base: &str,
    unit: &str,
) -> Result<(), TypeMismatchDetail> {
    let canonical_base = canonical_type_name(base);
    if canonical_base != "수" {
        return Err(type_mismatch_detail(&canonical_base, value));
    }
    let Some(spec) = unit_spec_from_symbol(unit) else {
        return Err(type_mismatch_detail(&format!("수@{}", unit), value));
    };
    match value {
        Value::Unit(unit_value) if unit_value.dim == spec.dim => Ok(()),
        _ => Err(type_mismatch_detail(&format!("수@{}", unit), value)),
    }
}

fn type_mismatch_detail(expected: &str, actual: &Value) -> TypeMismatchDetail {
    TypeMismatchDetail {
        expected: expected.to_string(),
        actual: value_type_name(actual),
    }
}

fn value_type_name(value: &Value) -> String {
    match value {
        Value::None => "없음".to_string(),
        Value::Bool(_) => "참거짓".to_string(),
        Value::Fixed64(n) => {
            if n.frac_part() == 0 {
                "정수".to_string()
            } else {
                "수".to_string()
            }
        }
        Value::Unit(unit) => {
            if unit.is_dimensionless() {
                if unit.value.frac_part() == 0 {
                    "정수".to_string()
                } else {
                    "수".to_string()
                }
            } else {
                format!("수@{}", unit.dim.format())
            }
        }
        Value::String(_) => "글".to_string(),
        Value::ResourceHandle(_) => "자원핸들".to_string(),
        Value::List(_) => "차림".to_string(),
        Value::Set(_) => "모음".to_string(),
        Value::Map(_) => "짝맞춤".to_string(),
        Value::Pack(_) => "묶음".to_string(),
        Value::Formula(_) => "수식값".to_string(),
        Value::Template(_) => "글무늬값".to_string(),
        Value::Lambda(_) => "씨앗".to_string(),
    }
}

fn format_parse_error(source: &str, err: &ParseError) -> String {
    let mut line = 1usize;
    let mut col = 1usize;
    let mut line_start = 0usize;
    for (idx, ch) in source.char_indices() {
        if idx >= err.span.start {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
            line_start = idx + ch.len_utf8();
        } else {
            col += 1;
        }
    }
    let line_end = source[line_start..]
        .find('\n')
        .map(|offset| line_start + offset)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];
    let caret = " ".repeat(col.saturating_sub(1)) + "^";
    format!("파싱 실패: {} ({}:{})\n{}\n{}", err.message, line, col, line_text, caret)
}

fn position_to_line_col(source: &str, byte_pos: usize) -> (u32, u32) {
    let target = byte_pos.min(source.len());
    let mut line = 1u32;
    let mut col = 0u32;
    let mut idx = 0usize;
    for ch in source.chars() {
        if idx >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
        idx += ch.len_utf8();
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_input() -> InputSnapshot {
        InputSnapshot {
            tick_id: 0,
            dt: Fixed64::from_i64(1),
            keys_pressed: 0,
            last_key_name: String::new(),
            pointer_x_i32: 0,
            pointer_y_i32: 0,
            ai_injections: Vec::new(),
            net_events: Vec::new(),
            rng_seed: 0,
        }
    }

    #[test]
    fn map_dot_access_existing_key_runs() {
        let script = r#"
매틱:움직씨 = {
    m <- ("a", 1) 짝맞춤.
    값 <- m.a.
}
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
    }

    #[test]
    fn map_dot_access_missing_key_is_fatal() {
        let script = r#"
매틱:움직씨 = {
    m <- ("a", 1) 짝맞춤.
    값 <- m.b.
}
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("missing key must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("FATAL:MAP_KEY_MISSING:b"),
            "unexpected error: {err}"
        );
    }
}
