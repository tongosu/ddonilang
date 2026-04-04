use crate::core::fixed64::Fixed64;
use crate::core::state::Key;
use crate::core::trace::Trace;
use crate::core::unit::{eval_unit_expr, format_dim, temperature_dim, UnitDim, UnitExpr};
use crate::core::value::{
    AssertionValue, DiceValue, LambdaValue, ListValue, MapEntry, MapValue, PackValue, Quantity,
    SetValue, TemplateValue, Value,
};
use crate::core::State;
use crate::lang::ast::{
    ArgBinding, BinaryOp, Binding, ContractKind, ContractMode, DeclKind, Expr, FormulaDialect,
    HookKind, Literal, ParamPin, Path, Program, SeedKind, Stmt, UnaryOp,
};
use crate::lang::lexer::Lexer;
use crate::lang::parser::Parser;
use crate::runtime::detmath;
use crate::runtime::error::RuntimeError;
use crate::runtime::formula::{
    analyze_formula, eval_formula_body, format_formula_body, FormulaError,
};
use crate::runtime::open::{OpenCheckpoint, OpenRuntime, OpenSolverOp, OpenSolverReply};
use crate::runtime::template::{match_template, render_template};
use ddonirang_core::ResourceHandle;
use regex::{Regex, RegexBuilder};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

const PROOF_GUARD_REGISTRY_KEY: &str = "__proof.guard_registry";
const EXACT_NUMERIC_KIND_FIELD: &str = "__정확수종류";
const EXACT_NUMERIC_BIGINT_FIELD: &str = "값";
const EXACT_NUMERIC_RATIONAL_NUM_FIELD: &str = "분자";
const EXACT_NUMERIC_RATIONAL_DEN_FIELD: &str = "분모";
const EXACT_NUMERIC_FACTOR_VALUE_FIELD: &str = "값";
const NUMERIC_KIND_BIG_INT: &str = "큰바른수";
const NUMERIC_KIND_RATIONAL: &str = "나눔수";
const NUMERIC_KIND_FACTOR: &str = "곱수";

pub struct Evaluator {
    state: State,
    trace: Trace,
    bogae_requested: bool,
    bogae_requested_tick: bool,
    aborted: bool,
    contract_diags: Vec<ContractDiag>,
    diagnostics: Vec<DiagnosticRecord>,
    diagnostic_failures: Vec<DiagnosticFailure>,
    proof_runtime: Vec<ProofRuntimeEvent>,
    rng_state: Cell<u64>,
    current_madi: Cell<u64>,
    open_source: String,
    open_source_lines: Vec<String>,
    open: OpenRuntime,
    user_seeds: BTreeMap<String, UserSeed>,
    import_aliases: BTreeMap<String, String>,
    current_entity_stack: Vec<String>,
    pending_signals: VecDeque<PendingSignal>,
    processing_signal_queue: bool,
    deferred_assign_frames: Vec<Vec<DeferredAssign>>,
    contract_stack: Vec<ContractFrame>,
    next_contract_frame_id: u64,
    nuri_reset_snapshot: Option<NuriResetSnapshot>,
    lifecycle_pan_units: Vec<Vec<Stmt>>,
    lifecycle_madang_units: Vec<Vec<Stmt>>,
    lifecycle_pan_name_to_index: BTreeMap<String, usize>,
    lifecycle_madang_name_to_index: BTreeMap<String, usize>,
    lifecycle_active_pan: Option<usize>,
    lifecycle_active_madang: Option<usize>,
}

pub struct EvalFailure {
    pub error: RuntimeError,
    pub output: EvalOutput,
}

#[derive(Clone, Debug)]
pub struct DiagnosticRecord {
    pub tick: u64,
    pub name: String,
    pub lhs: String,
    pub rhs: String,
    pub delta: String,
    pub threshold: String,
    pub result: String,
    pub error_code: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DiagnosticFailure {
    pub code: String,
    pub tick: u64,
    pub name: String,
    pub delta: String,
    pub threshold: String,
    pub span: crate::lang::span::Span,
}

#[derive(Clone, Debug)]
pub enum ProofRuntimeEvent {
    ProofBlock {
        tick: u64,
        name: String,
        result: String,
        error_code: Option<String>,
    },
    ProofCheck {
        tick: u64,
        target: String,
        binding_count: u64,
        passed: bool,
        error_code: Option<String>,
        span: crate::lang::span::Span,
    },
    SolverCheck {
        tick: u64,
        query: String,
        satisfied: Option<bool>,
        error_code: Option<String>,
        span: crate::lang::span::Span,
    },
    SolverSearch {
        tick: u64,
        operation: String,
        query: String,
        found: Option<bool>,
        value: Option<String>,
        error_code: Option<String>,
        span: crate::lang::span::Span,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FlowControl {
    Continue,
    Break(crate::lang::span::Span),
    Return(Value, crate::lang::span::Span),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LifecycleFamily {
    Pan,
    Madang,
}

impl LifecycleFamily {
    fn as_korean(self) -> &'static str {
        match self {
            LifecycleFamily::Pan => "판",
            LifecycleFamily::Madang => "마당",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LifecycleTransitionVerb {
    Start,
    Next,
    Call,
}

impl LifecycleTransitionVerb {
    fn as_korean(self) -> &'static str {
        match self {
            LifecycleTransitionVerb::Start => "시작하기",
            LifecycleTransitionVerb::Next => "넘어가기",
            LifecycleTransitionVerb::Call => "불러오기",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LifecycleFamilyHint {
    None,
    Pan,
    Madang,
    Ambiguous,
}

impl LifecycleFamilyHint {
    fn has_hint(self) -> bool {
        !matches!(self, LifecycleFamilyHint::None)
    }

    fn as_family(self) -> Option<LifecycleFamily> {
        match self {
            LifecycleFamilyHint::Pan => Some(LifecycleFamily::Pan),
            LifecycleFamilyHint::Madang => Some(LifecycleFamily::Madang),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiagnosticOp {
    Approx,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
}

impl DiagnosticOp {
    fn as_str(self) -> &'static str {
        match self {
            DiagnosticOp::Approx => "≈",
            DiagnosticOp::Eq => "=",
            DiagnosticOp::Lt => "<",
            DiagnosticOp::Le => "<=",
            DiagnosticOp::Gt => ">",
            DiagnosticOp::Ge => ">=",
        }
    }
}

#[derive(Clone, Debug)]
struct DiagnosticCondition {
    lhs_raw: String,
    rhs_raw: String,
    op: DiagnosticOp,
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
    params: Vec<ParamPin>,
    body: Vec<Stmt>,
}

#[derive(Clone)]
struct PendingSignal {
    receiver_name: String,
    event_kind: String,
    sender_name: String,
    event_payload: PackValue,
    span: crate::lang::span::Span,
}

#[derive(Clone)]
struct DeferredAssign {
    key: Key,
    value: Value,
}

#[derive(Clone)]
struct ContractFrame {
    frame_id: u64,
    persistent_snapshot: State,
    pending_signals_len: usize,
    deferred_assign_frames: Vec<Vec<DeferredAssign>>,
    open_checkpoint: OpenCheckpoint,
    depth: u32,
}

#[derive(Clone)]
struct NuriResetSnapshot {
    state: State,
    pending_signals: VecDeque<PendingSignal>,
    deferred_assign_frames: Vec<Vec<DeferredAssign>>,
    current_entity_stack: Vec<String>,
    aborted: bool,
    bogae_requested: bool,
    bogae_requested_tick: bool,
    lifecycle_pan_name_to_index: BTreeMap<String, usize>,
    lifecycle_madang_name_to_index: BTreeMap<String, usize>,
    lifecycle_active_pan: Option<usize>,
    lifecycle_active_madang: Option<usize>,
}

struct GuardCheckOutcome {
    passed: bool,
    abort_failed: bool,
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
        Self::with_state_seed_open(
            state,
            seed,
            OpenRuntime::deny(),
            "<memory>".to_string(),
            None,
        )
    }

    pub fn with_state_seed_open(
        state: State,
        seed: u64,
        open: OpenRuntime,
        open_source: String,
        open_source_text: Option<String>,
    ) -> Self {
        let mut open = open;
        open.set_tick(0);
        let open_source_lines = open_source_text
            .map(|text| text.split('\n').map(|line| line.to_string()).collect())
            .unwrap_or_default();
        Self {
            state,
            trace: Trace::new(),
            bogae_requested: false,
            bogae_requested_tick: false,
            aborted: false,
            contract_diags: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_failures: Vec::new(),
            proof_runtime: Vec::new(),
            rng_state: Cell::new(seed),
            current_madi: Cell::new(0),
            open_source,
            open_source_lines,
            open,
            user_seeds: BTreeMap::new(),
            import_aliases: BTreeMap::new(),
            current_entity_stack: Vec::new(),
            pending_signals: VecDeque::new(),
            processing_signal_queue: false,
            deferred_assign_frames: Vec::new(),
            contract_stack: Vec::new(),
            next_contract_frame_id: 1,
            nuri_reset_snapshot: None,
            lifecycle_pan_units: Vec::new(),
            lifecycle_madang_units: Vec::new(),
            lifecycle_pan_name_to_index: BTreeMap::new(),
            lifecycle_madang_name_to_index: BTreeMap::new(),
            lifecycle_active_pan: None,
            lifecycle_active_madang: None,
        }
    }

    #[allow(dead_code)]
    pub fn run(self, program: &Program) -> Result<EvalOutput, RuntimeError> {
        self.run_with_ticks(program, 1)
    }

    pub fn run_with_ticks(self, program: &Program, ticks: u64) -> Result<EvalOutput, RuntimeError> {
        self.run_with_ticks_capture_failure(program, ticks)
            .map_err(|failure| failure.error)
    }

    pub fn run_with_ticks_capture_failure(
        self,
        program: &Program,
        ticks: u64,
    ) -> Result<EvalOutput, EvalFailure> {
        self.run_with_ticks_observe_capture_failure(program, ticks, |_, _, _| {})
    }

    #[allow(dead_code)]
    pub fn run_with_ticks_observe<F>(
        self,
        program: &Program,
        ticks: u64,
        on_tick: F,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
    {
        self.run_with_ticks_observe_capture_failure(program, ticks, on_tick)
            .map_err(|failure| failure.error)
    }

    pub fn run_with_ticks_observe_capture_failure<F>(
        self,
        program: &Program,
        ticks: u64,
        on_tick: F,
    ) -> Result<EvalOutput, EvalFailure>
    where
        F: FnMut(u64, &State, bool),
    {
        self.run_with_ticks_internal(
            program,
            ticks,
            no_op_before_tick_with_open,
            no_op_should_stop,
            on_tick,
        )
    }

    pub fn run_with_ticks_observe_and_inject<F, G>(
        self,
        program: &Program,
        ticks: u64,
        mut before_tick: G,
        on_tick: F,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State) -> Result<(), RuntimeError>,
    {
        self.run_with_ticks_internal(
            program,
            ticks,
            move |madi, state, _open| before_tick(madi, state),
            no_op_should_stop,
            on_tick,
        )
        .map_err(|failure| failure.error)
    }

    #[allow(dead_code)]
    pub fn run_with_ticks_observe_and_inject_stop<F, G, H>(
        self,
        program: &Program,
        ticks: u64,
        mut before_tick: G,
        on_tick: F,
        should_stop: H,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State) -> Result<(), RuntimeError>,
        H: FnMut(u64, &State) -> bool,
    {
        self.run_with_ticks_internal(
            program,
            ticks,
            move |madi, state, _open| before_tick(madi, state),
            should_stop,
            on_tick,
        )
        .map_err(|failure| failure.error)
    }

    #[allow(dead_code)]
    pub fn run_with_ticks_observe_and_inject_open<F, G>(
        self,
        program: &Program,
        ticks: u64,
        before_tick: G,
        on_tick: F,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State, &'a mut OpenRuntime) -> Result<(), RuntimeError>,
    {
        self.run_with_ticks_observe_and_inject_open_capture_failure(
            program,
            ticks,
            before_tick,
            on_tick,
        )
        .map_err(|failure| failure.error)
    }

    pub fn run_with_ticks_observe_and_inject_open_capture_failure<F, G>(
        self,
        program: &Program,
        ticks: u64,
        before_tick: G,
        on_tick: F,
    ) -> Result<EvalOutput, EvalFailure>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State, &'a mut OpenRuntime) -> Result<(), RuntimeError>,
    {
        self.run_with_ticks_internal(program, ticks, before_tick, no_op_should_stop, on_tick)
    }

    #[allow(dead_code)]
    pub fn run_with_ticks_observe_and_inject_stop_open<F, G, H>(
        self,
        program: &Program,
        ticks: u64,
        before_tick: G,
        on_tick: F,
        should_stop: H,
    ) -> Result<EvalOutput, RuntimeError>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State, &'a mut OpenRuntime) -> Result<(), RuntimeError>,
        H: FnMut(u64, &State) -> bool,
    {
        self.run_with_ticks_observe_and_inject_stop_open_capture_failure(
            program,
            ticks,
            before_tick,
            on_tick,
            should_stop,
        )
        .map_err(|failure| failure.error)
    }

    pub fn run_with_ticks_observe_and_inject_stop_open_capture_failure<F, G, H>(
        self,
        program: &Program,
        ticks: u64,
        before_tick: G,
        on_tick: F,
        should_stop: H,
    ) -> Result<EvalOutput, EvalFailure>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State, &'a mut OpenRuntime) -> Result<(), RuntimeError>,
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
    ) -> Result<EvalOutput, EvalFailure>
    where
        F: FnMut(u64, &State, bool),
        G: for<'a> FnMut(u64, &'a mut State, &'a mut OpenRuntime) -> Result<(), RuntimeError>,
        H: FnMut(u64, &State) -> bool,
    {
        self.capture_nuri_reset_snapshot();
        self.lifecycle_pan_units.clear();
        self.lifecycle_madang_units.clear();
        self.lifecycle_pan_name_to_index.clear();
        self.lifecycle_madang_name_to_index.clear();
        self.lifecycle_active_pan = None;
        self.lifecycle_active_madang = None;
        let mut init_stmts = Vec::new();
        let mut start_hooks: Vec<&Vec<Stmt>> = Vec::new();
        let mut end_hooks: Vec<&Vec<Stmt>> = Vec::new();
        let mut every_hooks: Vec<&Vec<Stmt>> = Vec::new();
        let mut every_n_hooks: Vec<(u64, &Vec<Stmt>)> = Vec::new();
        let mut becomes_hooks: Vec<(&Expr, &Vec<Stmt>)> = Vec::new();
        let mut while_hooks: Vec<(&Expr, &Vec<Stmt>)> = Vec::new();
        let mut seed_defs: Vec<&Stmt> = Vec::new();

        for stmt in &program.stmts {
            match stmt {
                Stmt::SeedDef { .. } => seed_defs.push(stmt),
                Stmt::Hook { kind, body, .. } => match kind {
                    HookKind::Start => start_hooks.push(body),
                    HookKind::End => end_hooks.push(body),
                    HookKind::EveryMadi => every_hooks.push(body),
                    HookKind::EveryNMadi(interval) => every_n_hooks.push((*interval, body)),
                },
                Stmt::HookWhenBecomes {
                    condition, body, ..
                } => becomes_hooks.push((condition, body)),
                Stmt::HookWhile {
                    condition, body, ..
                } => while_hooks.push((condition, body)),
                Stmt::LifecycleBlock {
                    name, kind, body, ..
                } => match kind {
                    crate::lang::ast::LifecycleUnitKind::Pan => {
                        let index = self.lifecycle_pan_units.len();
                        self.lifecycle_pan_units.push(body.clone());
                        if let Some(name) = name {
                            self.lifecycle_pan_name_to_index.insert(name.clone(), index);
                        }
                    }
                    crate::lang::ast::LifecycleUnitKind::Madang => {
                        let index = self.lifecycle_madang_units.len();
                        self.lifecycle_madang_units.push(body.clone());
                        if let Some(name) = name {
                            self.lifecycle_madang_name_to_index
                                .insert(name.clone(), index);
                        }
                    }
                },
                other => init_stmts.push(other),
            }
        }

        for stmt in seed_defs {
            let Stmt::SeedDef {
                name,
                params,
                kind,
                body,
                span: _,
            } = stmt
            else {
                continue;
            };
            self.user_seeds.insert(
                name.clone(),
                UserSeed {
                    kind: kind.clone(),
                    params: params.clone(),
                    body: body.clone(),
                },
            );
        }

        for stmt in init_stmts {
            let flow = match self.eval_stmt(stmt) {
                Ok(flow) => flow,
                Err(error) => return Err(self.into_failure(error)),
            };
            if let Err(error) = ensure_no_break(flow) {
                return Err(self.into_failure(error));
            }
        }

        let imja_seeds: Vec<(String, UserSeed)> = self
            .user_seeds
            .iter()
            .filter_map(|(name, seed)| {
                matches!(&seed.kind, SeedKind::Named(kind) if kind == "임자")
                    .then(|| (name.clone(), seed.clone()))
            })
            .collect();
        for (name, seed) in imja_seeds {
            if let Err(error) = self.eval_imja_init(&name, &seed) {
                return Err(self.into_failure(error));
            }
        }

        for hook in start_hooks {
            let flow = match self.eval_block(hook) {
                Ok(flow) => flow,
                Err(error) => return Err(self.into_failure(error)),
            };
            if let Err(error) = ensure_no_break(flow) {
                return Err(self.into_failure(error));
            }
        }
        self.capture_nuri_reset_snapshot();
        let mut becomes_prev_values: Vec<bool> = vec![false; becomes_hooks.len()];

        for madi in 0..ticks {
            self.current_madi.set(madi);
            self.open.set_tick(madi);
            if should_stop(madi, &self.state) {
                break;
            }
            let tick_span = crate::lang::span::Span::new(0, 0, 0, 0);
            let tick_frame = match self.begin_contract_frame(tick_span) {
                Ok(frame) => frame,
                Err(error) => return Err(self.into_failure(error)),
            };
            if let Err(error) = before_tick(madi, &mut self.state, &mut self.open) {
                return Err(self.into_failure(error));
            }
            if should_stop(madi, &self.state) {
                break;
            }
            for hook in &every_hooks {
                let flow = match self.eval_block(hook) {
                    Ok(flow) => flow,
                    Err(error) => return Err(self.into_failure(error)),
                };
                if let Err(error) = ensure_no_break(flow) {
                    return Err(self.into_failure(error));
                }
            }
            for (interval, hook) in &every_n_hooks {
                if madi % *interval != 0 {
                    continue;
                }
                let flow = match self.eval_block(hook) {
                    Ok(flow) => flow,
                    Err(error) => return Err(self.into_failure(error)),
                };
                if let Err(error) = ensure_no_break(flow) {
                    return Err(self.into_failure(error));
                }
            }
            for (index, (condition, hook)) in becomes_hooks.iter().enumerate() {
                let value = match self.eval_expr(condition) {
                    Ok(value) => value,
                    Err(error) => return Err(self.into_failure(error)),
                };
                let current = match is_truthy(&value, condition.span()) {
                    Ok(value) => value,
                    Err(error) => return Err(self.into_failure(error)),
                };
                if current && !becomes_prev_values[index] {
                    let flow = match self.eval_block(hook) {
                        Ok(flow) => flow,
                        Err(error) => return Err(self.into_failure(error)),
                    };
                    if let Err(error) = ensure_no_break(flow) {
                        return Err(self.into_failure(error));
                    }
                }
                becomes_prev_values[index] = current;
            }
            for (condition, hook) in &while_hooks {
                let value = match self.eval_expr(condition) {
                    Ok(value) => value,
                    Err(error) => return Err(self.into_failure(error)),
                };
                let current = match is_truthy(&value, condition.span()) {
                    Ok(value) => value,
                    Err(error) => return Err(self.into_failure(error)),
                };
                if current {
                    let flow = match self.eval_block(hook) {
                        Ok(flow) => flow,
                        Err(error) => return Err(self.into_failure(error)),
                    };
                    if let Err(error) = ensure_no_break(flow) {
                        return Err(self.into_failure(error));
                    }
                }
            }
            let rollback_tick = match self.eval_registered_proof_guards_for_tick(tick_span) {
                Ok(value) => value,
                Err(error) => return Err(self.into_failure(error)),
            };
            if rollback_tick {
                if let Err(error) = self.rollback_contract_frame(tick_frame.frame_id, tick_span) {
                    return Err(self.into_failure(error));
                }
            } else {
                self.commit_contract_frame(tick_frame.frame_id);
            }
            self.aborted = false;
            let tick_requested = self.bogae_requested_tick;
            on_tick(madi, &self.state, tick_requested);
            self.bogae_requested_tick = false;
        }

        for hook in end_hooks {
            let flow = match self.eval_block(hook) {
                Ok(flow) => flow,
                Err(error) => return Err(self.into_failure(error)),
            };
            if let Err(error) = ensure_no_break(flow) {
                return Err(self.into_failure(error));
            }
        }

        Ok(self.into_output())
    }

    fn into_output(self) -> EvalOutput {
        EvalOutput {
            state: self.state,
            trace: self.trace,
            bogae_requested: self.bogae_requested,
            contract_diags: self.contract_diags,
            diagnostics: self.diagnostics,
            diagnostic_failures: self.diagnostic_failures,
            proof_runtime: self.proof_runtime,
        }
    }

    fn into_failure(self, error: RuntimeError) -> EvalFailure {
        EvalFailure {
            error,
            output: self.into_output(),
        }
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<FlowControl, RuntimeError> {
        if self.aborted {
            return Ok(FlowControl::Continue);
        }
        match stmt {
            Stmt::ImportBlock { items, .. } => {
                for item in items {
                    self.import_aliases
                        .insert(item.alias.clone(), item.path.clone());
                }
                Ok(FlowControl::Continue)
            }
            Stmt::ExportBlock { .. } => Ok(FlowControl::Continue),
            Stmt::DeclBlock { items, .. } => {
                for item in items {
                    let value = if let Some(expr) = &item.value {
                        self.eval_expr(expr)?
                    } else if matches!(item.kind, DeclKind::Butbak) {
                        return Err(RuntimeError::Pack {
                            message: format!(
                                "채비에서 '=' 항목은 초기값이 필요합니다: {}",
                                item.name
                            ),
                            span: item.span,
                        });
                    } else {
                        Value::None
                    };
                    if item.value.is_some() {
                        self.check_decl_initializer_type(&item.type_name, &value, item.span)?;
                    }
                    self.state.set(self.scoped_decl_key(&item.name), value);
                }
                Ok(FlowControl::Continue)
            }
            Stmt::SeedDef {
                name,
                params,
                kind,
                body,
                span: _,
            } => {
                self.user_seeds.insert(
                    name.clone(),
                    UserSeed {
                        kind: kind.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                Ok(FlowControl::Continue)
            }
            Stmt::Pragma { name, args, span } => {
                self.eval_pragma(name, args, *span)?;
                Ok(FlowControl::Continue)
            }
            Stmt::Assign {
                target,
                value,
                deferred,
                ..
            } => {
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
                if *deferred {
                    self.record_deferred_assign(key, val);
                } else {
                    self.state.set(key, val);
                }
                Ok(FlowControl::Continue)
            }
            Stmt::Show { value, .. } => {
                let val = self.eval_expr(value)?;
                self.trace.log(val.display());
                Ok(FlowControl::Continue)
            }
            Stmt::Inspect { value, .. } => {
                let val = self.eval_expr(value)?;
                self.trace.log(val.display());
                Ok(FlowControl::Continue)
            }
            Stmt::Expr { value, .. } => {
                let _ = self.eval_expr(value)?;
                Ok(FlowControl::Continue)
            }
            Stmt::Receive { .. } => Ok(FlowControl::Continue),
            Stmt::Send {
                sender,
                payload,
                receiver,
                span,
            } => {
                self.dispatch_signal_send(sender.as_ref(), payload, receiver, *span)?;
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
            Stmt::Hook { .. }
            | Stmt::HookWhenBecomes { .. }
            | Stmt::HookWhile { .. }
            | Stmt::LifecycleBlock { .. } => {
                Ok(FlowControl::Continue)
            }
            Stmt::OpenBlock { body, span } => {
                self.trace.log(format!(
                    "open_block_enter {}:{}",
                    span.start_line, span.start_col
                ));
                let flow = self.eval_block(body)?;
                self.trace.log(format!(
                    "open_block_exit {}:{}",
                    span.start_line, span.start_col
                ));
                Ok(flow)
            }
            Stmt::BeatBlock { body, .. } => self.eval_beat_block(body),
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
                exhaustive,
                span,
                ..
            } => {
                for branch in branches {
                    let value = self.eval_expr(&branch.condition)?;
                    if is_truthy(&value, branch.condition.span())? {
                        return self.eval_block(&branch.body);
                    }
                }
                if let Some(body) = else_body {
                    return self.eval_block(body);
                }
                if *exhaustive {
                    return Err(RuntimeError::ProofIncomplete { span: *span });
                }
                Ok(FlowControl::Continue)
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
                condition, body, ..
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
            Stmt::Quantifier { .. } => Ok(FlowControl::Continue),
            Stmt::Break { span } => Ok(FlowControl::Break(*span)),
            Stmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
                ..
            } => {
                let frame = self.begin_contract_frame(condition.span())?;
                let value = self.eval_expr(condition)?;
                let mut ok = is_truthy(&value, condition.span())?;
                match kind {
                    ContractKind::Pre => {
                        if ok {
                            if let Some(body) = then_body {
                                match self.eval_block(body)? {
                                    FlowControl::Continue => {}
                                    FlowControl::Break(span) => {
                                        self.commit_contract_frame(frame.frame_id);
                                        return Ok(FlowControl::Break(span));
                                    }
                                    FlowControl::Return(value, span) => {
                                        self.commit_contract_frame(frame.frame_id);
                                        return Ok(FlowControl::Return(value, span));
                                    }
                                }
                                if self.aborted {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::Continue);
                                }
                            }
                            self.commit_contract_frame(frame.frame_id);
                        } else {
                            match self.eval_block(else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Break(span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::Break(span));
                                }
                                FlowControl::Return(value, span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::Return(value, span));
                                }
                            }
                            self.emit_contract_violation(*kind, *mode, condition.span());
                            if matches!(mode, ContractMode::Abort) {
                                self.rollback_contract_frame(frame.frame_id, condition.span())?;
                                self.aborted = true;
                            } else {
                                self.commit_contract_frame(frame.frame_id);
                            }
                        }
                    }
                    ContractKind::Post => {
                        if !ok {
                            match self.eval_block(else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Break(span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::Break(span));
                                }
                                FlowControl::Return(value, span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::Return(value, span));
                                }
                            }
                            if self.aborted {
                                self.commit_contract_frame(frame.frame_id);
                                return Ok(FlowControl::Continue);
                            }
                            let value = self.eval_expr(condition)?;
                            ok = is_truthy(&value, condition.span())?;
                            if !ok {
                                self.emit_contract_violation(*kind, *mode, condition.span());
                                if matches!(mode, ContractMode::Abort) {
                                    self.rollback_contract_frame(frame.frame_id, condition.span())?;
                                    self.aborted = true;
                                } else {
                                    self.commit_contract_frame(frame.frame_id);
                                }
                                return Ok(FlowControl::Continue);
                            }
                        }
                        if let Some(body) = then_body {
                            match self.eval_block(body)? {
                                FlowControl::Continue => {}
                                FlowControl::Break(span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::Break(span));
                                }
                                FlowControl::Return(value, span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::Return(value, span));
                                }
                            }
                            if self.aborted {
                                self.commit_contract_frame(frame.frame_id);
                                return Ok(FlowControl::Continue);
                            }
                        }
                        self.commit_contract_frame(frame.frame_id);
                    }
                }
                Ok(FlowControl::Continue)
            }
        }
    }

    fn check_decl_initializer_type(
        &self,
        type_name: &str,
        value: &Value,
        span: crate::lang::span::Span,
    ) -> Result<(), RuntimeError> {
        let canonical = ddonirang_lang::stdlib::canonicalize_type_alias(type_name);
        match canonical {
            "_" | "값" => Ok(()),
            "수" => match value {
                Value::Num(qty) if qty.dim.is_dimensionless() => Ok(()),
                Value::Num(_) => Err(RuntimeError::UnitMismatch { span }),
                _ => Err(type_mismatch_detail("수", value, span)),
            },
            "바른수" => {
                if is_dimensionless_integer_value(value) {
                    Ok(())
                } else {
                    Err(type_mismatch_detail("바른수", value, span))
                }
            }
            "큰바른수" => {
                if exact_numeric_kind(value) == Some(NUMERIC_KIND_BIG_INT)
                    || is_dimensionless_integer_value(value)
                {
                    Ok(())
                } else {
                    Err(type_mismatch_detail("큰바른수", value, span))
                }
            }
            "나눔수" => {
                if exact_numeric_kind(value) == Some(NUMERIC_KIND_RATIONAL)
                    || is_dimensionless_integer_value(value)
                {
                    Ok(())
                } else {
                    Err(type_mismatch_detail("나눔수", value, span))
                }
            }
            "곱수" => {
                if exact_numeric_kind(value) == Some(NUMERIC_KIND_FACTOR)
                    || is_dimensionless_integer_value(value)
                {
                    Ok(())
                } else {
                    Err(type_mismatch_detail("곱수", value, span))
                }
            }
            "참거짓" => match value {
                Value::Bool(_) => Ok(()),
                _ => Err(type_mismatch_detail("참거짓", value, span)),
            },
            "글" => match value {
                Value::Str(_) => Ok(()),
                _ => Err(type_mismatch_detail("글", value, span)),
            },
            "차림" => match value {
                Value::List(_) => Ok(()),
                _ => Err(type_mismatch_detail("차림", value, span)),
            },
            "모음" => match value {
                Value::Set(_) => Ok(()),
                _ => Err(type_mismatch_detail("모음", value, span)),
            },
            "짝맞춤" => match value {
                Value::Map(_) => Ok(()),
                _ => Err(type_mismatch_detail("짝맞춤", value, span)),
            },
            "묶음" => match value {
                Value::Pack(_) => Ok(()),
                _ => Err(type_mismatch_detail("묶음", value, span)),
            },
            "없음" => match value {
                Value::None => Ok(()),
                _ => Err(type_mismatch_detail("없음", value, span)),
            },
            _ => Ok(()),
        }
    }

    fn eval_pragma(
        &mut self,
        name: &str,
        args: &str,
        span: crate::lang::span::Span,
    ) -> Result<(), RuntimeError> {
        let (pragma_name, pragma_args) = parse_pragma_invocation(name, args);
        if pragma_name != "진단" {
            return Ok(());
        }
        self.eval_diagnostic_pragma(&pragma_args, span)
    }

    fn eval_diagnostic_pragma(
        &mut self,
        args: &str,
        span: crate::lang::span::Span,
    ) -> Result<(), RuntimeError> {
        let parts = split_top_level_args(args);
        if parts.is_empty() {
            return Err(RuntimeError::Pack {
                message: "#진단 인자가 비었습니다".to_string(),
                span,
            });
        }

        let condition = parse_diagnostic_condition(&parts[0], span)?;
        let mut threshold: Option<Fixed64> = None;
        let mut custom_name: Option<String> = None;
        let mut error_code_override: Option<String> = None;

        for part in parts.iter().skip(1) {
            let Some((raw_key, raw_value)) = split_option_pair(part) else {
                continue;
            };
            let key = raw_key.trim();
            let value = raw_value.trim();
            match key {
                "허용오차" | "threshold" => {
                    let Some(parsed) = Fixed64::parse_literal(value) else {
                        return Err(RuntimeError::Pack {
                            message: format!("#진단 허용오차 파싱 실패: {}", value),
                            span,
                        });
                    };
                    threshold = Some(parsed);
                }
                "이름" | "name" => {
                    custom_name = Some(parse_string_option(value));
                }
                "오류코드" | "error_code" => {
                    let parsed = parse_string_option(value);
                    if !is_supported_diagnostic_error_code(&parsed) {
                        return Err(RuntimeError::Pack {
                            message: format!(
                                "#진단 오류코드는 E_ECO_DIVERGENCE_DETECTED 또는 E_SFC_IDENTITY_VIOLATION만 허용됩니다: {}",
                                parsed
                            ),
                            span,
                        });
                    }
                    error_code_override = Some(parsed);
                }
                _ => {}
            }
        }

        let threshold = match condition.op {
            DiagnosticOp::Approx => threshold.unwrap_or(Fixed64::zero()),
            DiagnosticOp::Eq => threshold.unwrap_or(Fixed64::zero()),
            _ => threshold.unwrap_or(Fixed64::zero()),
        };
        let lhs = self.resolve_diagnostic_value(&condition.lhs_raw, span)?;
        let rhs = self.resolve_diagnostic_value(&condition.rhs_raw, span)?;
        let delta = fixed64_abs(lhs.saturating_sub(rhs));
        let passed = match condition.op {
            DiagnosticOp::Approx => delta.raw() <= threshold.raw(),
            DiagnosticOp::Eq => delta.raw() <= threshold.raw(),
            DiagnosticOp::Lt => lhs.raw() < rhs.raw(),
            DiagnosticOp::Le => lhs.raw() <= rhs.raw(),
            DiagnosticOp::Gt => lhs.raw() > rhs.raw(),
            DiagnosticOp::Ge => lhs.raw() >= rhs.raw(),
        };
        let name = custom_name.unwrap_or_else(|| {
            format!(
                "{} {} {}",
                condition.lhs_raw,
                condition.op.as_str(),
                condition.rhs_raw
            )
        });
        let failure_code = if passed {
            None
        } else {
            Some(infer_diagnostic_failure_code(
                error_code_override.as_deref(),
                &name,
                &condition,
            ))
        };

        self.diagnostics.push(DiagnosticRecord {
            tick: self.current_madi.get(),
            name: name.clone(),
            lhs: lhs.format(),
            rhs: rhs.format(),
            delta: delta.format(),
            threshold: threshold.format(),
            result: if passed {
                "수렴".to_string()
            } else {
                "발산".to_string()
            },
            error_code: failure_code.clone(),
        });

        if let Some(code) = failure_code {
            self.diagnostic_failures.push(DiagnosticFailure {
                code,
                tick: self.current_madi.get(),
                name,
                delta: delta.format(),
                threshold: threshold.format(),
                span,
            });
        }
        Ok(())
    }

    fn resolve_diagnostic_value(
        &self,
        raw: &str,
        span: crate::lang::span::Span,
    ) -> Result<Fixed64, RuntimeError> {
        let trimmed = raw.trim();
        if let Some(number) = Fixed64::parse_literal(trimmed) {
            return Ok(number);
        }
        let key = normalize_diag_state_key(trimmed);
        let Some(value) = self.state.get(&Key::new(key.clone())) else {
            return Err(RuntimeError::Undefined { path: key, span });
        };
        match value {
            Value::Num(quantity) => Ok(quantity.raw),
            _ => Err(type_mismatch_detail("number", value, span)),
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

    fn capture_nuri_reset_snapshot(&mut self) {
        self.nuri_reset_snapshot = Some(NuriResetSnapshot {
            state: self.state.clone(),
            pending_signals: self.pending_signals.clone(),
            deferred_assign_frames: self.deferred_assign_frames.clone(),
            current_entity_stack: self.current_entity_stack.clone(),
            aborted: self.aborted,
            bogae_requested: self.bogae_requested,
            bogae_requested_tick: self.bogae_requested_tick,
            lifecycle_pan_name_to_index: self.lifecycle_pan_name_to_index.clone(),
            lifecycle_madang_name_to_index: self.lifecycle_madang_name_to_index.clone(),
            lifecycle_active_pan: self.lifecycle_active_pan,
            lifecycle_active_madang: self.lifecycle_active_madang,
        });
    }

    fn apply_nuri_reset(&mut self) {
        let Some(snapshot) = self.nuri_reset_snapshot.clone() else {
            return;
        };
        self.state = snapshot.state;
        self.pending_signals = snapshot.pending_signals;
        self.deferred_assign_frames = snapshot.deferred_assign_frames;
        self.current_entity_stack = snapshot.current_entity_stack;
        self.aborted = snapshot.aborted;
        self.bogae_requested = snapshot.bogae_requested;
        self.bogae_requested_tick = snapshot.bogae_requested_tick;
        self.lifecycle_pan_name_to_index = snapshot.lifecycle_pan_name_to_index;
        self.lifecycle_madang_name_to_index = snapshot.lifecycle_madang_name_to_index;
        self.lifecycle_active_pan = snapshot.lifecycle_active_pan;
        self.lifecycle_active_madang = snapshot.lifecycle_active_madang;
    }

    fn infer_lifecycle_family(
        &self,
        target_name: Option<&str>,
        default_family: LifecycleFamily,
    ) -> LifecycleFamily {
        let Some(target_name) = target_name else {
            return default_family;
        };
        let named_as_pan = self.lifecycle_pan_name_to_index.contains_key(target_name);
        let named_as_madang = self
            .lifecycle_madang_name_to_index
            .contains_key(target_name);
        match (named_as_pan, named_as_madang) {
            (true, false) => return LifecycleFamily::Pan,
            (false, true) => return LifecycleFamily::Madang,
            _ => {}
        }
        if target_name.ends_with("마당") || target_name.contains("마당") {
            return LifecycleFamily::Madang;
        }
        if target_name.ends_with("판") || target_name.contains("판") {
            return LifecycleFamily::Pan;
        }
        default_family
    }

    fn hinted_lifecycle_family(&self, target_name: &str) -> LifecycleFamilyHint {
        let has_madang = target_name.ends_with("마당") || target_name.contains("마당");
        let has_pan = target_name.ends_with("판") || target_name.contains("판");
        match (has_pan, has_madang) {
            (false, false) => LifecycleFamilyHint::None,
            (true, false) => LifecycleFamilyHint::Pan,
            (false, true) => LifecycleFamilyHint::Madang,
            (true, true) => LifecycleFamilyHint::Ambiguous,
        }
    }

    fn lifecycle_units_len(&self, family: LifecycleFamily) -> usize {
        match family {
            LifecycleFamily::Pan => self.lifecycle_pan_units.len(),
            LifecycleFamily::Madang => self.lifecycle_madang_units.len(),
        }
    }

    fn lifecycle_active_index(&self, family: LifecycleFamily) -> Option<usize> {
        match family {
            LifecycleFamily::Pan => self.lifecycle_active_pan,
            LifecycleFamily::Madang => self.lifecycle_active_madang,
        }
    }

    fn set_lifecycle_active_index(&mut self, family: LifecycleFamily, index: Option<usize>) {
        match family {
            LifecycleFamily::Pan => self.lifecycle_active_pan = index,
            LifecycleFamily::Madang => self.lifecycle_active_madang = index,
        }
    }

    fn lifecycle_unit_body(&self, family: LifecycleFamily, index: usize) -> Option<Vec<Stmt>> {
        match family {
            LifecycleFamily::Pan => self.lifecycle_pan_units.get(index).cloned(),
            LifecycleFamily::Madang => self.lifecycle_madang_units.get(index).cloned(),
        }
    }

    fn run_lifecycle_unit(
        &mut self,
        family: LifecycleFamily,
        index: usize,
    ) -> Result<(), RuntimeError> {
        let Some(body) = self.lifecycle_unit_body(family, index) else {
            return Ok(());
        };
        let flow = self.eval_block(&body)?;
        ensure_no_break(flow)
    }

    fn eval_lifecycle_transition(
        &mut self,
        verb: LifecycleTransitionVerb,
        target_name: Option<&str>,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let default_family = match verb {
            LifecycleTransitionVerb::Start | LifecycleTransitionVerb::Next => {
                LifecycleFamily::Madang
            }
            LifecycleTransitionVerb::Call => LifecycleFamily::Pan,
        };
        let family = self.infer_lifecycle_family(target_name, default_family);
        let (pan_named_index, madang_named_index, explicit_target_index) = match target_name {
            Some(target_name) => {
                let pan_named_index = self
                    .lifecycle_pan_name_to_index
                    .get(target_name)
                    .copied();
                let madang_named_index = self
                    .lifecycle_madang_name_to_index
                    .get(target_name)
                    .copied();
                let explicit_target_index = match family {
                    LifecycleFamily::Pan => pan_named_index,
                    LifecycleFamily::Madang => madang_named_index,
                };
                (pan_named_index, madang_named_index, explicit_target_index)
            }
            None => (None, None, None),
        };
        if let Some(target_name) = target_name {
            let hinted_family = self.hinted_lifecycle_family(target_name);
            let has_family_hint = hinted_family.has_hint();
            let has_any_lifecycle_units =
                !self.lifecycle_pan_units.is_empty() || !self.lifecycle_madang_units.is_empty();
            if matches!(hinted_family, LifecycleFamilyHint::Ambiguous) && has_any_lifecycle_units {
                return Err(RuntimeError::LifecycleTargetFamilyAmbiguous {
                    verb: verb.as_korean(),
                    target: target_name.to_string(),
                    span,
                });
            }
            let declared_family = match (pan_named_index, madang_named_index) {
                (Some(_), None) => Some(LifecycleFamily::Pan),
                (None, Some(_)) => Some(LifecycleFamily::Madang),
                _ => None,
            };
            if let (Some(hinted_family), Some(declared_family)) =
                (hinted_family.as_family(), declared_family)
            {
                if hinted_family != declared_family {
                    return Err(RuntimeError::LifecycleTargetFamilyConflict {
                        verb: verb.as_korean(),
                        target: target_name.to_string(),
                        hint_family: hinted_family.as_korean(),
                        declared_family: declared_family.as_korean(),
                        span,
                    });
                }
            }
            if explicit_target_index.is_none() && !has_family_hint && has_any_lifecycle_units {
                return Err(RuntimeError::LifecycleTargetUnknown {
                    verb: verb.as_korean(),
                    target: target_name.to_string(),
                    family: family.as_korean(),
                    span,
                });
            }
        }
        let units_len = self.lifecycle_units_len(family);
        if units_len == 0 {
            return Ok(Value::None);
        }
        match verb {
            LifecycleTransitionVerb::Start => {
                let index = explicit_target_index.unwrap_or(0);
                self.set_lifecycle_active_index(family, Some(index));
                self.run_lifecycle_unit(family, index)?;
            }
            LifecycleTransitionVerb::Next => {
                let next_index = explicit_target_index.unwrap_or_else(|| match self.lifecycle_active_index(family) {
                    Some(active) if active + 1 < units_len => active + 1,
                    Some(_) | None => 0,
                });
                self.set_lifecycle_active_index(family, Some(next_index));
                self.run_lifecycle_unit(family, next_index)?;
            }
            LifecycleTransitionVerb::Call => {
                let index = explicit_target_index
                    .or_else(|| self.lifecycle_active_index(family))
                    .unwrap_or(0);
                self.run_lifecycle_unit(family, index)?;
            }
        }
        Ok(Value::None)
    }

    fn begin_contract_frame(
        &mut self,
        span: crate::lang::span::Span,
    ) -> Result<ContractFrame, RuntimeError> {
        let frame = ContractFrame {
            frame_id: self.next_contract_frame_id,
            persistent_snapshot: self.state.clone(),
            pending_signals_len: self.pending_signals.len(),
            deferred_assign_frames: self.deferred_assign_frames.clone(),
            open_checkpoint: self.open.checkpoint(span)?,
            depth: (self.contract_stack.len() as u32).saturating_add(1),
        };
        self.next_contract_frame_id = self.next_contract_frame_id.saturating_add(1);
        self.contract_stack.push(frame.clone());
        Ok(frame)
    }

    fn commit_contract_frame(&mut self, frame_id: u64) {
        let frame = self
            .contract_stack
            .pop()
            .expect("contract frame stack underflow on commit");
        debug_assert_eq!(frame.frame_id, frame_id);
        debug_assert_eq!(frame.depth as usize, self.contract_stack.len() + 1);
    }

    fn rollback_contract_frame(
        &mut self,
        frame_id: u64,
        span: crate::lang::span::Span,
    ) -> Result<(), RuntimeError> {
        let frame = self
            .contract_stack
            .pop()
            .expect("contract frame stack underflow on rollback");
        debug_assert_eq!(frame.frame_id, frame_id);
        debug_assert_eq!(frame.depth as usize, self.contract_stack.len() + 1);
        self.state = frame.persistent_snapshot;
        self.pending_signals.truncate(frame.pending_signals_len);
        self.deferred_assign_frames = frame.deferred_assign_frames;
        self.open.restore(frame.open_checkpoint, span)?;
        Ok(())
    }

    fn record_deferred_assign(&mut self, key: Key, value: Value) {
        if let Some(frame) = self.deferred_assign_frames.last_mut() {
            frame.push(DeferredAssign { key, value });
            return;
        }
        self.state.set(key, value);
    }

    fn eval_beat_block(&mut self, body: &[Stmt]) -> Result<FlowControl, RuntimeError> {
        let outer_state = self.state.clone();
        let outer_aborted = self.aborted;
        self.state = outer_state.clone();
        self.deferred_assign_frames.push(Vec::new());
        let flow = self.eval_block(body);
        let deferred = self.deferred_assign_frames.pop().unwrap_or_default();
        match flow {
            Err(err) => {
                self.state = outer_state;
                self.aborted = outer_aborted;
                Err(err)
            }
            Ok(flow) => {
                if self.aborted && !outer_aborted {
                    self.state = outer_state;
                    return Ok(FlowControl::Continue);
                }
                for assign in deferred {
                    self.state.set(assign.key, assign.value);
                }
                Ok(flow)
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit, span) => self.literal_to_value(lit, *span),
            Expr::Path(path) => self.eval_path(path),
            Expr::FieldAccess {
                target,
                field,
                span,
            } => {
                let base = self.eval_expr(target)?;
                self.eval_member_access(base, field, *span)
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
            Expr::Formula {
                dialect,
                body,
                span,
            } => self.eval_formula_value(*dialect, body, *span),
            Expr::Assertion { assertion, .. } => Ok(Value::Assertion(AssertionValue {
                body_source: assertion.body_source.clone(),
                canon: assertion.canon.clone(),
            })),
            Expr::FormulaEval {
                dialect,
                body,
                bindings,
                span,
            } => self.eval_formula_eval(*dialect, body, bindings, *span),
            Expr::Template { body, span: _ } => {
                Ok(Value::Template(TemplateValue { body: body.clone() }))
            }
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

    fn current_entity_name(&self) -> Option<&str> {
        self.current_entity_stack.last().map(|name| name.as_str())
    }

    fn scoped_decl_key(&self, name: &str) -> Key {
        if let Some(entity) = self.current_entity_name() {
            Key::new(format!("{}.{}", entity, name))
        } else {
            Key::new(name.to_string())
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
        if root == "제" {
            if let Some(entity) = self.current_entity_name() {
                return Ok(Key::new(format!(
                    "{}.{}",
                    entity,
                    path.segments[1..].join(".")
                )));
            }
            return Err(RuntimeError::JeOutsideImja { span: path.span });
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
                return Ok(Value::Str(
                    read_input_key_from_state(&self.state).unwrap_or_default(),
                ));
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
        let mut current =
            self.state
                .get(&base_key)
                .cloned()
                .ok_or_else(|| RuntimeError::Undefined {
                    path: path.segments.join("."),
                    span: path.span,
                })?;
        for field in path.segments.iter().skip(2) {
            current = self.eval_member_access(current, field, path.span)?;
        }
        Ok(current)
    }

    fn eval_member_access(
        &self,
        base: Value,
        field: &str,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        match base {
            Value::Pack(pack) => {
                let Some(value) = pack.fields.get(field) else {
                    return Err(RuntimeError::Pack {
                        message: format!("필드 '{}'가 없습니다", field),
                        span,
                    });
                };
                if matches!(value, Value::None) {
                    return Err(RuntimeError::Pack {
                        message: format!("필드 '{}' 값이 없음입니다", field),
                        span,
                    });
                }
                Ok(value.clone())
            }
            Value::Map(map) => Self::map_get_required(&map, Value::Str(field.to_string()), span),
            Value::List(list) => {
                let index = parse_list_segment_index(field, span)?;
                list.items.get(index).cloned().ok_or(RuntimeError::Pack {
                    message: format!("차림 인덱스 '{}'가 범위를 벗어났습니다", field),
                    span,
                })
            }
            _ => Err(RuntimeError::Pack {
                message: "묶음/짝맞춤/차림 접근만 가능합니다".to_string(),
                span,
            }),
        }
    }

    fn map_get_required(
        map: &MapValue,
        key: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let value = map.map_get(&key);
        if matches!(value, Value::None) {
            return Err(RuntimeError::MapDotKeyMissing {
                key: key.display(),
                span,
            });
        }
        Ok(value)
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
                    if let Some(value) = normalize_temperature_literal(unit_expr, base) {
                        return Ok(Value::Num(Quantity::new(value, temperature_dim())));
                    }
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

    fn eval_unary(
        &self,
        op: &UnaryOp,
        value: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
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
            BinaryOp::Mod => self.eval_mod(left, right, span),
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

    fn eval_call(
        &mut self,
        name: &str,
        args: &[ArgBinding],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let resolved_name = self.resolve_module_call_name(name);
        let canon_name = Self::canonicalize_stdlib_alias(&resolved_name);
        if matches!(
            canon_name,
            "주사위씨.만들기" | "주사위씨.뽑기" | "주사위씨.실수뽑기" | "주사위씨.골라뽑기"
        ) {
            return self.eval_dice_call(canon_name, args, span);
        }
        let mut values = Vec::new();
        for arg in args {
            values.push(self.eval_expr(&arg.expr)?);
        }
        if Self::is_builtin_name(canon_name) {
            return self.eval_call_values(&resolved_name, &values, span);
        }
        if let Some(seed) = self.user_seeds.get(&resolved_name).cloned() {
            let bound = self.bind_seed_args(&seed, args, &values, span)?;
            return self.eval_user_seed(&resolved_name, &seed, &bound, span);
        }
        // try stripping call tails (short and long forms) to find seed stem
        for tail in &["하면서", "면서", "하기", "기", "하고", "고", "하면", "면"] {
            if let Some(stem) = resolved_name.strip_suffix(tail) {
                if let Some(seed) = self.user_seeds.get(stem).cloned() {
                    let bound = self.bind_seed_args(&seed, args, &values, span)?;
                    return self.eval_user_seed(stem, &seed, &bound, span);
                }
            }
        }
        Err(RuntimeError::TypeMismatch {
            expected: "known function",
            span,
        })
    }

    fn resolve_module_call_name(&self, name: &str) -> String {
        let Some((alias, rest)) = name.split_once('.') else {
            return name.to_string();
        };
        let Some(path) = self.import_aliases.get(alias) else {
            return name.to_string();
        };
        // V1: module alias call is syntactic namespace only.
        //      런타임은 alias 접두를 제거한 함수 이름으로 해석한다.
        if rest.is_empty() {
            return name.to_string();
        }
        if path.starts_with("./")
            || path.starts_with("표준/")
            || path.starts_with("나눔/")
            || path.starts_with("내/")
            || path.starts_with("벌림/")
        {
            return rest.to_string();
        }
        name.to_string()
    }

    fn eval_dice_call(
        &mut self,
        name: &str,
        args: &[ArgBinding],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        match name {
            "주사위씨.만들기" => {
                let seed = dice_seed_from_args(self, args, span)?;
                Ok(Value::Dice(DiceValue {
                    seed,
                    state: seed,
                    draws: 0,
                }))
            }
            "주사위씨.뽑기" => {
                let (key, dice, offset) = self.dice_target_from_args(args, span)?;
                let min = dice_number_arg(self, args, offset, &["최소"], span)?;
                let max = dice_number_arg(self, args, offset + 1, &["최대"], span)?;
                if !min.dim.is_dimensionless() || !max.dim.is_dimensionless() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "dimensionless integer range",
                        span,
                    });
                }
                let min_int = quantity_to_int(&min, span)?;
                let max_int = quantity_to_int(&max, span)?;
                if min_int > max_int {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "min <= max",
                        span,
                    });
                }
                let (next_dice, sample) = advance_dice(&dice);
                if let Some(key) = key {
                    self.state.set(key, Value::Dice(next_dice));
                }
                let range = (max_int - min_int + 1) as u64;
                let value = (sample % range) as i64 + min_int;
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(value),
                    UnitDim::zero(),
                )))
            }
            "주사위씨.실수뽑기" => {
                let (key, dice, offset) = self.dice_target_from_args(args, span)?;
                let min = dice_number_arg(self, args, offset, &["최소"], span)?;
                let max = dice_number_arg(self, args, offset + 1, &["최대"], span)?;
                if min.dim != max.dim {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "same-unit min/max",
                        span,
                    });
                }
                let (next_dice, sample) = advance_dice(&dice);
                if let Some(key) = key {
                    self.state.set(key, Value::Dice(next_dice));
                }
                let fraction = Fixed64::from_raw((sample & 0xFFFF_FFFF) as i64);
                let delta = max.raw.saturating_sub(min.raw).saturating_mul(fraction);
                let value = min.raw.saturating_add(delta);
                Ok(Value::Num(Quantity::new(value, min.dim)))
            }
            "주사위씨.골라뽑기" => {
                let (key, dice, offset) = self.dice_target_from_args(args, span)?;
                let list = dice_list_arg(self, args, offset, &["후보", "후보들"], span)?;
                let (next_dice, sample) = advance_dice(&dice);
                if let Some(key) = key {
                    self.state.set(key, Value::Dice(next_dice));
                }
                if list.items.is_empty() {
                    return Ok(Value::None);
                }
                let idx = (sample % list.items.len() as u64) as usize;
                Ok(list.items[idx].clone())
            }
            _ => Err(RuntimeError::TypeMismatch {
                expected: "known dice function",
                span,
            }),
        }
    }

    fn dice_target_from_args(
        &mut self,
        args: &[ArgBinding],
        span: crate::lang::span::Span,
    ) -> Result<(Option<Key>, DiceValue, usize), RuntimeError> {
        let first = args.first().ok_or(RuntimeError::TypeMismatch {
            expected: "주사위씨, ...",
            span,
        })?;
        let value = self.eval_expr(&first.expr)?;
        let dice = match value {
            Value::Dice(dice) => dice,
            other => return Err(type_mismatch_detail("주사위씨", &other, span)),
        };
        let key = match &first.expr {
            Expr::Path(path) => Some(self.path_to_key(path)?),
            _ => None,
        };
        Ok((key, dice, 1))
    }

    fn eval_call_values(
        &mut self,
        name: &str,
        values: &[Value],
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let name = Self::canonicalize_stdlib_alias(name);
        match name {
            "마당다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                self.apply_nuri_reset();
                Ok(Value::None)
            }
            "판다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                self.apply_nuri_reset();
                Ok(Value::None)
            }
            "누리다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                self.apply_nuri_reset();
                Ok(Value::None)
            }
            "보개다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                Ok(Value::None)
            }
            "모두다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                self.apply_nuri_reset();
                Ok(Value::None)
            }
            "시작하기" | "넘어가기" | "불러오기" => {
                let verb = match name {
                    "시작하기" => LifecycleTransitionVerb::Start,
                    "넘어가기" => LifecycleTransitionVerb::Next,
                    "불러오기" => LifecycleTransitionVerb::Call,
                    _ => unreachable!("lifecycle transition verb already matched"),
                };
                let target_name = match values {
                    [] => None,
                    [Value::Str(target_name)] => Some(target_name.as_str()),
                    [other] => return Err(type_mismatch_detail("글", other, span)),
                    _ => {
                        return Err(RuntimeError::LifecycleTargetArity {
                            verb: verb.as_korean(),
                            got: values.len(),
                            span,
                        });
                    }
                };
                self.eval_lifecycle_transition(verb, target_name, span)
            }
            "살피기" => {
                let (assertion, binding_values) = expect_assertion_check_values(values, span)?;
                self.eval_assertion_check(&assertion, &binding_values, span)
            }
            "지키기" => {
                let assertion = expect_single_assertion(values, span)?;
                self.register_proof_guard(assertion);
                Ok(Value::None)
            }
            "지키기끔" => {
                let assertion = expect_single_assertion(values, span)?;
                self.unregister_proof_guard(&assertion);
                Ok(Value::None)
            }
            "수" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "값",
                        span,
                    });
                }
                match &values[0] {
                    Value::Num(qty) => {
                        if !qty.dim.is_dimensionless() {
                            return Err(RuntimeError::UnitMismatch { span });
                        }
                        Ok(Value::Num(qty.clone()))
                    }
                    Value::Str(text) => {
                        let raw = Fixed64::parse_literal(text).ok_or(RuntimeError::TypeMismatch {
                            expected: "number string",
                            span,
                        })?;
                        Ok(Value::Num(Quantity::new(raw, UnitDim::zero())))
                    }
                    other => Err(type_mismatch_detail("number|string", other, span)),
                }
            }
            "바른수" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "값",
                        span,
                    });
                }
                let number = parse_integer_like_value(&values[0], span)?;
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(number),
                    UnitDim::zero(),
                )))
            }
            "큰바른수" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "값",
                        span,
                    });
                }
                let raw = parse_integer_like_text(&values[0], span)?;
                Ok(make_exact_numeric_value(NUMERIC_KIND_BIG_INT, &[(EXACT_NUMERIC_BIGINT_FIELD, raw)]))
            }
            "나눔수" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "분자, 분모",
                        span,
                    });
                }
                let numerator = parse_integer_like_text(&values[0], span)?;
                let denominator = parse_integer_like_text(&values[1], span)?;
                if is_zero_integer_text(&denominator) {
                    return Err(RuntimeError::MathDivZero { span });
                }
                Ok(make_exact_numeric_value(
                    NUMERIC_KIND_RATIONAL,
                    &[
                        (EXACT_NUMERIC_RATIONAL_NUM_FIELD, numerator),
                        (EXACT_NUMERIC_RATIONAL_DEN_FIELD, denominator),
                    ],
                ))
            }
            "곱수" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "값",
                        span,
                    });
                }
                let raw = parse_integer_like_text(&values[0], span)?;
                Ok(make_exact_numeric_value(
                    NUMERIC_KIND_FACTOR,
                    &[(EXACT_NUMERIC_FACTOR_VALUE_FIELD, raw)],
                ))
            }
            "아님" | "아니다" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "condition",
                        span,
                    });
                }
                let truthy = is_truthy(&values[0], span)?;
                Ok(Value::Bool(!truthy))
            }
            "그리고" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "condition, condition",
                        span,
                    });
                }
                let left_ok = is_truthy(&values[0], span)?;
                let right_ok = is_truthy(&values[1], span)?;
                Ok(Value::Bool(left_ok && right_ok))
            }
            "또는" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "condition, condition",
                        span,
                    });
                }
                let left_ok = is_truthy(&values[0], span)?;
                let right_ok = is_truthy(&values[1], span)?;
                Ok(Value::Bool(left_ok || right_ok))
            }
            "열림.시각.지금" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                let site_id = self.open_site_id(span);
                self.open.open_clock(&site_id, span)
            }
            "열림.파일.읽기" => {
                let path = expect_open_path(values, span)?;
                let site_id = self.open_site_id(span);
                self.open.open_file_read(&site_id, &path, span)
            }
            "열림.난수.뽑기" | "열림.난수.하나" | "열림.난수" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                let site_id = self.open_site_id(span);
                self.open.open_rand(&site_id, span)
            }
            "열림.네트워크.요청" => {
                let args = expect_open_net_args(values, span)?;
                let site_id = self.open_site_id(span);
                self.open.open_net(
                    &site_id,
                    &args.url,
                    &args.method,
                    args.body.as_deref(),
                    args.response.as_deref(),
                    span,
                )
            }
            "열림.호스트FFI.호출" => {
                let args = expect_open_ffi_args(values, span)?;
                let site_id = self.open_site_id(span);
                self.open.open_ffi(
                    &site_id,
                    &args.name,
                    args.args.as_deref(),
                    args.result.as_deref(),
                    span,
                )
            }
            "열림.GPU.실행" => {
                let args = expect_open_gpu_args(values, span)?;
                let site_id = self.open_site_id(span);
                self.open.open_gpu(
                    &site_id,
                    &args.kernel,
                    args.payload.as_deref(),
                    args.result.as_deref(),
                    span,
                )
            }
            "열림.풀이.확인" => {
                let args = expect_open_solver_check_args(values, span)?;
                let site_id = self.open_site_id(span);
                let result = self.open.open_solver(
                    &site_id,
                    OpenSolverOp::Check,
                    &args.query,
                    args.reply,
                    span,
                );
                match &result {
                    Ok(Value::Bool(satisfied)) => {
                        self.record_solver_check_runtime(&args.query, Some(*satisfied), None, span);
                    }
                    Ok(other) => {
                        self.record_solver_check_runtime(
                            &args.query,
                            None,
                            Some(format!(
                                "E_RUNTIME_TYPE_MISMATCH:{}",
                                value_type_name(other)
                            )),
                            span,
                        );
                    }
                    Err(err) => {
                        self.record_solver_check_runtime(
                            &args.query,
                            None,
                            Some(err.code().to_string()),
                            span,
                        );
                    }
                }
                result
            }
            "반례찾기" => {
                let args = expect_open_solver_search_args(values, span)?;
                let site_id = self.open_site_id(span);
                let result = self.open.open_solver(
                    &site_id,
                    OpenSolverOp::Counterexample,
                    &args.query,
                    args.reply,
                    span,
                );
                match &result {
                    Ok(value) => {
                        let (found, found_value) = extract_solver_search_runtime(value);
                        self.record_solver_search_runtime(
                            "counterexample",
                            &args.query,
                            found,
                            found_value,
                            None,
                            span,
                        );
                    }
                    Err(err) => {
                        self.record_solver_search_runtime(
                            "counterexample",
                            &args.query,
                            None,
                            None,
                            Some(err.code().to_string()),
                            span,
                        );
                    }
                }
                result
            }
            "해찾기" => {
                let args = expect_open_solver_search_args(values, span)?;
                let site_id = self.open_site_id(span);
                let result = self.open.open_solver(
                    &site_id,
                    OpenSolverOp::Solve,
                    &args.query,
                    args.reply,
                    span,
                );
                match &result {
                    Ok(value) => {
                        let (found, found_value) = extract_solver_search_runtime(value);
                        self.record_solver_search_runtime(
                            "solve",
                            &args.query,
                            found,
                            found_value,
                            None,
                            span,
                        );
                    }
                    Err(err) => {
                        self.record_solver_search_runtime(
                            "solve",
                            &args.query,
                            None,
                            None,
                            Some(err.code().to_string()),
                            span,
                        );
                    }
                }
                result
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
                    return Ok(Value::Num(Quantity::new(
                        Fixed64::from_int(0),
                        UnitDim::zero(),
                    )));
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
            "지니" => {
                let values = expect_quantity_list(values, span)?;
                if values.is_empty() {
                    return Ok(Value::None);
                }
                let mut total = Fixed64::zero();
                for value in &values {
                    if value.raw.raw() < 0 {
                        return Err(RuntimeError::MathDomain {
                            message: "지니 입력은 음수가 될 수 없습니다",
                            span,
                        });
                    }
                    total = total.saturating_add(value.raw);
                }
                if total.raw() == 0 {
                    return Ok(Value::Num(Quantity::new(Fixed64::zero(), UnitDim::zero())));
                }
                let mut pair_sum = Fixed64::zero();
                for i in 0..values.len() {
                    for j in (i + 1)..values.len() {
                        let diff = values[i].raw.saturating_sub(values[j].raw);
                        pair_sum = pair_sum.saturating_add(fixed64_abs(diff));
                    }
                }
                let count = Fixed64::from_int(values.len() as i64);
                let gini = pair_sum
                    .checked_div(count.saturating_mul(total))
                    .ok_or(RuntimeError::MathDivZero { span })?;
                Ok(Value::Num(Quantity::new(gini, UnitDim::zero())))
            }
            "분위수" => {
                let (list, p, mode) = expect_list_and_percentile(values, span)?;
                if list.is_empty() {
                    return Ok(Value::None);
                }
                if list.len() == 1 {
                    return Ok(Value::Num(list[0].clone()));
                }
                let mut sorted = list;
                sorted.sort_by(|a, b| a.raw.raw().cmp(&b.raw.raw()));
                match mode {
                    PercentileMode::Linear => {
                        let max_index = Fixed64::from_int((sorted.len() - 1) as i64);
                        let pos = p.saturating_mul(max_index);
                        let low = fixed64_floor(pos);
                        let high = fixed64_ceil(pos);
                        let low_index = fixed64_to_nonnegative_index(low, span)?;
                        let high_index = fixed64_to_nonnegative_index(high, span)?;
                        if low_index >= sorted.len() || high_index >= sorted.len() {
                            return Err(RuntimeError::MathDomain {
                                message: "분위수 인덱스가 범위를 벗어났습니다",
                                span,
                            });
                        }
                        if low_index == high_index {
                            return Ok(Value::Num(sorted[low_index].clone()));
                        }
                        let frac = pos.saturating_sub(low);
                        let base = sorted[low_index].clone();
                        let next = sorted[high_index].clone();
                        let delta = next.raw.saturating_sub(base.raw);
                        let step = delta.saturating_mul(frac);
                        Ok(Value::Num(Quantity::new(
                            base.raw.saturating_add(step),
                            base.dim,
                        )))
                    }
                    PercentileMode::NearestRank => {
                        let index = nearest_rank_index(p, sorted.len(), span)?;
                        Ok(Value::Num(sorted[index].clone()))
                    }
                }
            }
            "적분.오일러" => {
                let (value, rate, dt) = expect_three_quantities(values, span)?;
                let delta = Quantity::new(rate.raw.saturating_mul(dt.raw), rate.dim.add(dt.dim));
                ensure_same_dim(&value, &delta, span)?;
                Ok(Value::Num(Quantity::new(
                    value.raw.saturating_add(delta.raw),
                    value.dim,
                )))
            }
            "적분.반암시적오일러" => {
                if values.len() != 4 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "position, velocity, acceleration, dt",
                        span,
                    });
                }
                let position = expect_quantity_value(&values[0], span)?;
                let velocity = expect_quantity_value(&values[1], span)?;
                let acceleration = expect_quantity_value(&values[2], span)?;
                let dt = expect_quantity_value(&values[3], span)?;

                let delta_v = Quantity::new(
                    acceleration.raw.saturating_mul(dt.raw),
                    acceleration.dim.add(dt.dim),
                );
                ensure_same_dim(&velocity, &delta_v, span)?;
                let next_velocity =
                    Quantity::new(velocity.raw.saturating_add(delta_v.raw), velocity.dim);

                let delta_x = Quantity::new(
                    next_velocity.raw.saturating_mul(dt.raw),
                    next_velocity.dim.add(dt.dim),
                );
                ensure_same_dim(&position, &delta_x, span)?;
                let next_position =
                    Quantity::new(position.raw.saturating_add(delta_x.raw), position.dim);

                Ok(Value::List(ListValue {
                    items: vec![Value::Num(next_position), Value::Num(next_velocity)],
                }))
            }
            "보간.선형" => {
                let (start, end, t) = expect_three_quantities(values, span)?;
                ensure_same_dim(&start, &end, span)?;
                ensure_dimensionless(&t, span)?;
                let delta = end.raw.saturating_sub(start.raw);
                let step = delta.saturating_mul(t.raw);
                Ok(Value::Num(Quantity::new(
                    start.raw.saturating_add(step),
                    start.dim,
                )))
            }
            "보간.계단" => {
                if values.len() != 4 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "start, end, t, threshold",
                        span,
                    });
                }
                let start = expect_quantity_value(&values[0], span)?;
                let end = expect_quantity_value(&values[1], span)?;
                let t = expect_quantity_value(&values[2], span)?;
                let threshold = expect_quantity_value(&values[3], span)?;
                ensure_same_dim(&start, &end, span)?;
                ensure_dimensionless(&t, span)?;
                ensure_dimensionless(&threshold, span)?;
                if t.raw.raw() < threshold.raw.raw() {
                    Ok(Value::Num(start))
                } else {
                    Ok(Value::Num(end))
                }
            }
            "필터.이동평균" => {
                let (window, next_value) = expect_list_and_item(values, span)?;
                let next_qty = expect_quantity_value(&next_value, span)?;
                let mut next_items = Vec::with_capacity(window.items.len() + 1);
                let mut total = next_qty.raw;
                for item in &window.items {
                    let qty = expect_quantity_value(item, span)?;
                    ensure_same_dim(&next_qty, &qty, span)?;
                    total = total.saturating_add(qty.raw);
                    next_items.push(Value::Num(qty));
                }
                next_items.push(Value::Num(next_qty.clone()));
                let count = Fixed64::from_int(next_items.len() as i64);
                let avg = total
                    .checked_div(count)
                    .ok_or(RuntimeError::MathDivZero { span })?;
                Ok(Value::List(ListValue {
                    items: vec![
                        Value::List(ListValue { items: next_items }),
                        Value::Num(Quantity::new(avg, next_qty.dim)),
                    ],
                }))
            }
            "필터.지수평활" => {
                let (prev, next, alpha) = expect_three_quantities(values, span)?;
                ensure_same_dim(&prev, &next, span)?;
                ensure_dimensionless(&alpha, span)?;
                if alpha.raw.raw() < Fixed64::zero().raw() || alpha.raw.raw() > Fixed64::one().raw()
                {
                    return Err(RuntimeError::MathDomain {
                        message: "필터.지수평활 alpha는 0..1 범위여야 합니다",
                        span,
                    });
                }
                let keep = Fixed64::one().saturating_sub(alpha.raw);
                let prev_part = prev.raw.saturating_mul(keep);
                let next_part = next.raw.saturating_mul(alpha.raw);
                Ok(Value::Num(Quantity::new(
                    prev_part.saturating_add(next_part),
                    prev.dim,
                )))
            }
            "미분.중앙차분" => {
                let (math, var_name, point, step) =
                    expect_numeric_derivative_args(values, span, "미분.중앙차분")?;
                let prepared = prepare_numeric_formula(&math, &var_name, span, "미분.중앙차분")?;
                let two = Fixed64::from_int(2);

                let x_plus = Quantity::new(point.raw.saturating_add(step.raw), point.dim);
                let x_minus = Quantity::new(point.raw.saturating_sub(step.raw), point.dim);
                let y_plus = eval_numeric_formula_at(&prepared, x_plus, span, "미분.중앙차분")?;
                let y_minus = eval_numeric_formula_at(&prepared, x_minus, span, "미분.중앙차분")?;
                ensure_same_dim(&y_plus, &y_minus, span)?;
                let denom = step.raw.saturating_mul(two);
                let d_h = y_plus
                    .raw
                    .saturating_sub(y_minus.raw)
                    .checked_div(denom)
                    .ok_or(RuntimeError::MathDivZero { span })?;

                let step2 = step.raw.saturating_mul(two);
                let x_plus2 = Quantity::new(point.raw.saturating_add(step2), point.dim);
                let x_minus2 = Quantity::new(point.raw.saturating_sub(step2), point.dim);
                let y_plus2 = eval_numeric_formula_at(&prepared, x_plus2, span, "미분.중앙차분")?;
                let y_minus2 = eval_numeric_formula_at(&prepared, x_minus2, span, "미분.중앙차분")?;
                ensure_same_dim(&y_plus2, &y_minus2, span)?;
                let denom2 = step2.saturating_mul(two);
                let d_2h = y_plus2
                    .raw
                    .saturating_sub(y_minus2.raw)
                    .checked_div(denom2)
                    .ok_or(RuntimeError::MathDivZero { span })?;
                let three = Fixed64::from_int(3);
                let richardson_delta = d_h.saturating_sub(d_2h);
                let richardson_correction = richardson_delta
                    .checked_div(three)
                    .ok_or(RuntimeError::MathDivZero { span })?;
                let richardson_value = d_h.saturating_add(richardson_correction);
                let richardson_error = fixed64_abs(richardson_delta)
                    .checked_div(three)
                    .ok_or(RuntimeError::MathDivZero { span })?;

                let diff_dim = y_plus.dim.add(step.dim.scale(-1));
                Ok(Value::List(ListValue {
                    items: vec![
                        Value::Num(Quantity::new(richardson_value, diff_dim)),
                        Value::Num(Quantity::new(richardson_error, diff_dim)),
                        Value::Str("중앙차분".to_string()),
                    ],
                }))
            }
            "적분.사다리꼴" => {
                let (math, var_name, start, end, step) =
                    expect_numeric_integral_args(values, span, "적분.사다리꼴")?;
                let prepared = prepare_numeric_formula(&math, &var_name, span, "적분.사다리꼴")?;
                let (lower, upper, sign) = if end.raw.raw() < start.raw.raw() {
                    (end, start, -1_i64)
                } else {
                    (start, end, 1_i64)
                };

                let interval = upper.raw.saturating_sub(lower.raw);
                let n_est = interval
                    .checked_div(step.raw)
                    .ok_or(RuntimeError::MathDivZero { span })?;
                let mut n = fixed64_to_nonnegative_index(fixed64_ceil(n_est), span)?;
                if n == 0 {
                    n = 1;
                }
                let coarse = numeric_trapezoid_integral(&prepared, &lower, &upper, n, span)?;
                let refined = numeric_trapezoid_integral(
                    &prepared,
                    &lower,
                    &upper,
                    n.saturating_mul(2).max(2),
                    span,
                )?;
                ensure_same_dim(&coarse, &refined, span)?;
                let three = Fixed64::from_int(3);
                let richardson_delta = refined.raw.saturating_sub(coarse.raw);
                let richardson_correction = richardson_delta
                    .checked_div(three)
                    .ok_or(RuntimeError::MathDivZero { span })?;
                let richardson_value = refined.raw.saturating_add(richardson_correction);
                let richardson_error = fixed64_abs(richardson_delta)
                    .checked_div(three)
                    .ok_or(RuntimeError::MathDivZero { span })?;

                let mut approx = refined.clone();
                approx.raw = richardson_value;
                if sign < 0 {
                    approx.raw = Fixed64::zero().saturating_sub(approx.raw);
                }
                let err = Quantity::new(richardson_error, coarse.dim);
                Ok(Value::List(ListValue {
                    items: vec![
                        Value::Num(approx),
                        Value::Num(err),
                        Value::Str("사다리꼴".to_string()),
                    ],
                }))
            }
            "길이" => match values {
                [Value::Str(text)] => {
                    let len = text.chars().count() as i64;
                    Ok(Value::Num(Quantity::new(
                        Fixed64::from_int(len),
                        UnitDim::zero(),
                    )))
                }
                [Value::List(list)] => {
                    let len = list.items.len() as i64;
                    Ok(Value::Num(Quantity::new(
                        Fixed64::from_int(len),
                        UnitDim::zero(),
                    )))
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
            "글바꾸기!" => {
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
                let start = start.ok_or(RuntimeError::StringIndexOutOfRange { span })?;
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
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(idx),
                    UnitDim::zero(),
                )))
            }
            "찾기?" => {
                let (map, key) = expect_map_and_key(values, span)?;
                Ok(map.map_get(&key))
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
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(idx),
                    UnitDim::zero(),
                )))
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
            "채우기" => {
                let (template, pack) = expect_template_and_pack(values, span)?;
                let rendered = render_template(&template.body, &pack.fields, span)?;
                Ok(Value::Str(rendered))
            }
            "정규식" => {
                if values.is_empty() || values.len() > 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "pattern[, flags]",
                        span,
                    });
                }
                let pattern = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let flags = if values.len() == 2 {
                    match &values[1] {
                        Value::Str(text) => text.clone(),
                        value => return Err(type_mismatch_detail("string", value, span)),
                    }
                } else {
                    String::new()
                };
                let _ = compile_regex(&pattern, &flags, span)?;
                Ok(make_regex_value(pattern, flags))
            }
            "정규맞추기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "text, regex",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let (pattern, flags) = extract_regex_pattern(&values[1], span)?;
                let regex = compile_regex(&pattern, &flags, span)?;
                let matched = regex
                    .find(&text)
                    .map(|item| item.start() == 0 && item.end() == text.len())
                    .unwrap_or(false);
                Ok(Value::Bool(matched))
            }
            "정규찾기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "text, regex",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let (pattern, flags) = extract_regex_pattern(&values[1], span)?;
                let regex = compile_regex(&pattern, &flags, span)?;
                match regex.find(&text) {
                    Some(matched) => Ok(Value::Str(matched.as_str().to_string())),
                    None => Ok(Value::None),
                }
            }
            "정규캡처하기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "text, regex",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let (pattern, flags) = extract_regex_pattern(&values[1], span)?;
                let regex = compile_regex(&pattern, &flags, span)?;
                let items = regex_capture_first(&regex, &text)
                    .into_iter()
                    .map(Value::Str)
                    .collect::<Vec<_>>();
                Ok(Value::List(ListValue { items }))
            }
            "정규이름캡처하기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "text, regex",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let (pattern, flags) = extract_regex_pattern(&values[1], span)?;
                let regex = compile_regex(&pattern, &flags, span)?;
                Ok(Value::Map(regex_named_capture_first(&regex, &text)))
            }
            "정규바꾸기" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "text, regex, replacement",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let replacement = match &values[2] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let (pattern, flags) = extract_regex_pattern(&values[1], span)?;
                let regex = compile_regex(&pattern, &flags, span)?;
                validate_regex_replacement(&regex, &replacement, span)?;
                Ok(Value::Str(
                    regex.replace_all(&text, replacement.as_str()).to_string(),
                ))
            }
            "정규나누기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "text, regex",
                        span,
                    });
                }
                let text = match &values[0] {
                    Value::Str(text) => text.clone(),
                    value => return Err(type_mismatch_detail("string", value, span)),
                };
                let (pattern, flags) = extract_regex_pattern(&values[1], span)?;
                let regex = compile_regex(&pattern, &flags, span)?;
                let items = regex
                    .split(&text)
                    .map(|part| Value::Str(part.to_string()))
                    .collect::<Vec<_>>();
                Ok(Value::List(ListValue { items }))
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
                let dialect =
                    FormulaDialect::from_tag(&math.dialect).ok_or(RuntimeError::TypeMismatch {
                        expected: "formula",
                        span,
                    })?;
                let analysis = analyze_formula(&math.body, dialect)
                    .map_err(|err| map_formula_error(err, span))?;
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
                let qty = eval_formula_body(&math.body, dialect, &map)
                    .map_err(|err| map_formula_error(err, span))?;
                Ok(Value::Num(qty))
            }
            "눌렸나" => {
                let key = expect_single_string(values, span)?;
                let pressed =
                    read_state_flag(&self.state, &format!("샘.키보드.누르고있음.{}", key))
                        || read_state_flag(&self.state, &format!("입력상태.키_누르고있음.{}", key));
                Ok(Value::Bool(pressed))
            }
            "막눌렸나" => {
                let key = expect_single_string(values, span)?;
                let pressed = read_state_flag(&self.state, &format!("샘.키보드.눌림.{}", key))
                    || read_state_flag(&self.state, &format!("입력상태.키_눌림.{}", key));
                Ok(Value::Bool(pressed))
            }
            "입력키" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                Ok(Value::Str(
                    read_input_key_from_state(&self.state).unwrap_or_default(),
                ))
            }
            "입력키?" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                match read_input_key_from_state(&self.state) {
                    Some(key) => Ok(Value::Str(key)),
                    None => Ok(Value::None),
                }
            }
            "입력키!" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                match read_input_key_from_state(&self.state) {
                    Some(key) => Ok(Value::Str(key)),
                    None => Err(RuntimeError::InputKeyMissing { span }),
                }
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
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(value),
                    UnitDim::zero(),
                )))
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
                self.state
                    .set(Key::new("감정씨"), Value::Pack(pack.clone()));
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
                    Value::Num(Quantity::new(
                        Fixed64::from_int(madi as i64),
                        UnitDim::zero(),
                    )),
                );
                fields.insert("kind".to_string(), Value::Str(kind));
                fields.insert("text".to_string(), Value::Str(text));
                fields.insert("a".to_string(), a_value.unwrap_or(Value::None));
                fields.insert("b".to_string(), b_value.unwrap_or(Value::None));
                fields.insert("target".to_string(), target.unwrap_or(Value::None));
                let event_pack = PackValue { fields };
                events.push(Value::Pack(event_pack.clone()));
                let mut memory_fields = BTreeMap::new();
                memory_fields.insert(
                    "events".to_string(),
                    Value::List(ListValue { items: events }),
                );
                let memory_pack = PackValue {
                    fields: memory_fields,
                };
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
                        entries.insert(key.canon(), MapEntry { key, value });
                    }
                    Ok(Value::Map(MapValue { entries }))
                } else {
                    Ok(Value::None)
                }
            }
            "차림" | "목록" => Ok(Value::List(ListValue {
                items: values.to_vec(),
            })),
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
                let index = match row
                    .checked_mul(tensor.cols)
                    .and_then(|base| base.checked_add(col))
                {
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
                let index = match row
                    .checked_mul(tensor.cols)
                    .and_then(|base| base.checked_add(col))
                {
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
            "짝맞춤.값" => {
                let (map, key) = expect_map_and_key(values, span)?;
                Ok(map.map_get(&key))
            }
            "짝맞춤.필수값" => {
                let (map, key) = expect_map_and_key(values, span)?;
                Self::map_get_required(&map, key, span)
            }
            "짝맞춤.바꾼값" => {
                let (map, key, value) = expect_map_key_and_value(values, span)?;
                Ok(Value::Map(map.map_set(key, value)))
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
                Ok(Value::Num(Quantity::new(
                    Fixed64::from_int(i64::from(y)),
                    UnitDim::zero(),
                )))
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
                    Value::Num(Quantity::new(
                        Fixed64::from_int(i64::from(dx)),
                        UnitDim::zero(),
                    )),
                );
                fields.insert(
                    "dy".to_string(),
                    Value::Num(Quantity::new(
                        Fixed64::from_int(i64::from(dy)),
                        UnitDim::zero(),
                    )),
                );
                Ok(Value::Pack(PackValue { fields }))
            }
            _ => {
                if let Some(seed) = self.user_seeds.get(name).cloned() {
                    return self.eval_user_seed(name, &seed, values, span);
                }
                Err(RuntimeError::TypeMismatch {
                    expected: "known function",
                    span,
                })
            }
        }
    }

    fn next_rng_u64(&self) -> u64 {
        let (state, value) = splitmix64_next(self.rng_state.get());
        self.rng_state.set(state);
        value
    }

    fn next_rng_fixed64(&self) -> Fixed64 {
        let value = self.next_rng_u64();
        Fixed64::from_raw((value & 0xFFFF_FFFF) as i64)
    }

    fn canonicalize_stdlib_alias(name: &str) -> &str {
        ddonirang_lang::stdlib::canonicalize_stdlib_alias(name)
    }

    fn is_builtin_name(name: &str) -> bool {
        matches!(
            name,
            "abs"
                | "clamp"
                | "cos"
                | "max"
                | "min"
                | "powi"
                | "sin"
                | "tan"
                | "sqrt"
                | "asin"
                | "acos"
                | "atan"
                | "atan2"
                | "tetris_board_cell"
                | "tetris_board_drawlist"
                | "tetris_board_new"
                | "tetris_can_place"
                | "tetris_drop_y"
                | "tetris_lock"
                | "tetris_piece_block"
                | "각각돌며"
                | "감정더하기"
                | "값목록"
                | "거르기"
                | "그리고"
                | "글로"
                | "글바꾸기!"
                | "글바꾸기"
                | "글자뽑기"
                | "기억하기"
                | "길이"
                | "끝나나"
                | "입력키"
                | "입력키?"
                | "입력키!"
                | "눌렸나"
                | "다듬기"
                | "대문자로바꾸기"
                | "되풀이하기"
                | "뒤집기"
                | "들어있나"
                | "다음으로"
                | "또는"
                | "수"
                | "바른수"
                | "큰바른수"
                | "나눔수"
                | "곱수"
                | "마당다시"
                | "판다시"
                | "누리다시"
                | "보개다시"
                | "모두다시"
                | "시작하기"
                | "넘어가기"
                | "불러오기"
                | "마지막"
                | "마지막기억"
                | "막눌렸나"
                | "말결값"
                | "맞추기"
                | "모음"
                | "목록"
                | "무작위"
                | "무작위선택"
                | "무작위정수"
                | "묶음값"
                | "미분하기"
                | "미분.중앙차분"
                | "바꾸기"
                | "바닥"
                | "반올림"
                | "범위"
                | "변환"
                | "붙이기"
                | "소문자로바꾸기"
                | "숫자로"
                | "시작하나"
                | "쌍목록"
                | "아님"
                | "아니다"
                | "열림.GPU.실행"
                | "열림.난수"
                | "열림.난수.뽑기"
                | "열림.난수.하나"
                | "열림.네트워크.요청"
                | "열림.풀이.확인"
                | "반례찾기"
                | "해찾기"
                | "열림.시각.지금"
                | "열림.파일.읽기"
                | "열림.호스트FFI.호출"
                | "자르기"
                | "자원"
                | "적분.오일러"
                | "적분.반암시적오일러"
                | "적분.사다리꼴"
                | "적분하기"
                | "정렬"
                | "정규식"
                | "정규나누기"
                | "정규캡처하기"
                | "정규이름캡처하기"
                | "정규맞추기"
                | "정규바꾸기"
                | "정규찾기"
                | "지금상태"
                | "주사위씨.골라뽑기"
                | "주사위씨.만들기"
                | "주사위씨.뽑기"
                | "주사위씨.실수뽑기"
                | "제거"
                | "짝맞춤"
                | "짝맞춤.값"
                | "짝맞춤.필수값"
                | "짝맞춤.바꾼값"
                | "차림"
                | "차림.값"
                | "차림.바꾼값"
                | "흐름.만들기"
                | "흐름.밀어넣기"
                | "흐름.차림"
                | "흐름.최근값"
                | "흐름.길이"
                | "흐름.용량"
                | "흐름.비우기"
                | "흐름.잘라보기"
                | "채우기"
                | "찾기"
                | "찾기?"
                | "찾아보기"
                | "천장"
                | "처음으로"
                | "첫번째"
                | "추가"
                | "키목록"
                | "텐서.값"
                | "텐서.바꾼값"
                | "텐서.배치"
                | "텐서.자료"
                | "텐서.형상"
                | "토막내기"
                | "펼치기"
                | "평균"
                | "보간.선형"
                | "보간.계단"
                | "분위수"
                | "필터.이동평균"
                | "필터.지수평활"
                | "포함하나"
                | "표준.범위"
                | "풀기"
                | "합계"
                | "합치기"
                | "살피기"
                | "지키기"
                | "지키기끔"
                | "지니"
        )
    }

    fn eval_formula_value(
        &self,
        dialect: FormulaDialect,
        body: &str,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let formatted =
            format_formula_body(body, dialect).map_err(|err| map_formula_error(err, span))?;
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
        let qty =
            eval_formula_body(body, dialect, &map).map_err(|err| map_formula_error(err, span))?;
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
        let dialect =
            FormulaDialect::from_tag(&math.dialect).ok_or(RuntimeError::TypeMismatch {
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

    fn eval_pack(
        &mut self,
        bindings: &[Binding],
        _span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
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

    fn parse_assertion_program(
        &self,
        assertion: &AssertionValue,
        span: crate::lang::span::Span,
    ) -> Result<Vec<Stmt>, RuntimeError> {
        let wrapped = format!("검사:셈씨 = {{{}}}.", assertion.body_source);
        let tokens = Lexer::tokenize(&wrapped).map_err(|err| RuntimeError::Pack {
            message: format!("세움 파싱 실패: {}", err.code()),
            span,
        })?;
        let program =
            Parser::parse_with_default_root(tokens, "살림").map_err(|err| RuntimeError::Pack {
                message: format!("세움 파싱 실패: {}", err.code()),
                span,
            })?;
        let Some(Stmt::SeedDef { body, .. }) = program.stmts.first() else {
            return Err(RuntimeError::Pack {
                message: "세움 본문을 읽지 못했습니다".to_string(),
                span,
            });
        };
        Ok(body.clone())
    }

    fn eval_assertion_check(
        &mut self,
        assertion: &AssertionValue,
        values: &PackValue,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let body = self.parse_assertion_program(assertion, span)?;
        let saved_state = self.state.clone();
        let saved_trace = self.trace.clone();
        let saved_aborted = self.aborted;
        let saved_contract_len = self.contract_diags.len();
        let saved_diag_len = self.diagnostics.len();
        let saved_failure_len = self.diagnostic_failures.len();

        self.aborted = false;
        for (name, value) in &values.fields {
            self.state.set(Key::new(name.clone()), value.clone());
        }

        let result = self.eval_block(&body);
        let new_contracts = self.contract_diags.len().saturating_sub(saved_contract_len);
        let new_failures = self
            .diagnostic_failures
            .len()
            .saturating_sub(saved_failure_len);
        let passed = matches!(
            result,
            Ok(FlowControl::Continue) | Ok(FlowControl::Return(_, _))
        ) && !self.aborted
            && new_contracts == 0
            && new_failures == 0;

        self.state = saved_state;
        self.trace = saved_trace;
        self.aborted = saved_aborted;
        self.contract_diags.truncate(saved_contract_len);
        self.diagnostics.truncate(saved_diag_len);
        self.diagnostic_failures.truncate(saved_failure_len);

        let binding_count = Fixed64::from_int(values.fields.len() as i64);
        let failure_code = (!passed).then(|| "E_PROOF_CHECK_FAILED".to_string());
        self.diagnostics.push(DiagnosticRecord {
            tick: self.current_madi.get(),
            name: format!("살피기 {}", assertion.canon),
            lhs: if passed { "1" } else { "0" }.to_string(),
            rhs: "1".to_string(),
            delta: if passed { "0" } else { "1" }.to_string(),
            threshold: binding_count.format(),
            result: if passed {
                "성공".to_string()
            } else {
                "실패".to_string()
            },
            error_code: failure_code.clone(),
        });
        if let Some(code) = failure_code {
            self.diagnostic_failures.push(DiagnosticFailure {
                code,
                tick: self.current_madi.get(),
                name: format!("살피기 {}", assertion.canon),
                delta: if passed { "0" } else { "1" }.to_string(),
                threshold: binding_count.format(),
                span,
            });
        }
        self.proof_runtime.push(ProofRuntimeEvent::ProofCheck {
            tick: self.current_madi.get(),
            target: assertion.canon.clone(),
            binding_count: values.fields.len() as u64,
            passed,
            error_code: (!passed).then(|| "E_PROOF_CHECK_FAILED".to_string()),
            span,
        });
        Ok(Value::Bool(passed))
    }

    fn eval_guard_assertion_check(
        &mut self,
        assertion: &AssertionValue,
        span: crate::lang::span::Span,
    ) -> Result<GuardCheckOutcome, RuntimeError> {
        let body = self.parse_assertion_program(assertion, span)?;
        let saved_state = self.state.clone();
        let saved_trace = self.trace.clone();
        let saved_aborted = self.aborted;
        let saved_contract_len = self.contract_diags.len();
        let saved_diag_len = self.diagnostics.len();
        let saved_failure_len = self.diagnostic_failures.len();

        self.aborted = false;
        let result = self.eval_block(&body);
        let new_contracts = &self.contract_diags[saved_contract_len..];
        let new_failures = self
            .diagnostic_failures
            .len()
            .saturating_sub(saved_failure_len);
        let passed = matches!(
            result,
            Ok(FlowControl::Continue) | Ok(FlowControl::Return(_, _))
        ) && !self.aborted
            && new_contracts.is_empty()
            && new_failures == 0;
        let abort_failed = new_contracts
            .iter()
            .any(|diag| matches!(diag.mode, ContractMode::Abort));

        self.state = saved_state;
        self.trace = saved_trace;
        self.aborted = saved_aborted;
        self.diagnostics.truncate(saved_diag_len);
        self.diagnostic_failures.truncate(saved_failure_len);

        let failure_code = (!passed).then(|| "E_PROOF_CHECK_FAILED".to_string());
        self.diagnostics.push(DiagnosticRecord {
            tick: self.current_madi.get(),
            name: format!("지키기 {}", assertion.canon),
            lhs: if passed { "1" } else { "0" }.to_string(),
            rhs: "1".to_string(),
            delta: if passed { "0" } else { "1" }.to_string(),
            threshold: "0".to_string(),
            result: if passed {
                "성공".to_string()
            } else {
                "실패".to_string()
            },
            error_code: failure_code.clone(),
        });
        if let Some(code) = failure_code {
            self.diagnostic_failures.push(DiagnosticFailure {
                code,
                tick: self.current_madi.get(),
                name: format!("지키기 {}", assertion.canon),
                delta: if passed { "0" } else { "1" }.to_string(),
                threshold: "0".to_string(),
                span,
            });
        }
        self.proof_runtime.push(ProofRuntimeEvent::ProofCheck {
            tick: self.current_madi.get(),
            target: assertion.canon.clone(),
            binding_count: 0,
            passed,
            error_code: (!passed).then(|| "E_PROOF_CHECK_FAILED".to_string()),
            span,
        });
        Ok(GuardCheckOutcome {
            passed,
            abort_failed,
        })
    }

    fn eval_registered_proof_guards_for_tick(
        &mut self,
        span: crate::lang::span::Span,
    ) -> Result<bool, RuntimeError> {
        let guards = self.load_proof_guard_registry();
        for assertion in guards.values() {
            let outcome = self.eval_guard_assertion_check(assertion, span)?;
            if !outcome.passed && outcome.abort_failed {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn register_proof_guard(&mut self, assertion: AssertionValue) {
        let mut guards = self.load_proof_guard_registry();
        guards.insert(assertion.canon.clone(), assertion);
        self.save_proof_guard_registry(guards);
    }

    fn unregister_proof_guard(&mut self, assertion: &AssertionValue) {
        let mut guards = self.load_proof_guard_registry();
        guards.remove(&assertion.canon);
        self.save_proof_guard_registry(guards);
    }

    fn load_proof_guard_registry(&self) -> BTreeMap<String, AssertionValue> {
        let Some(value) = self
            .state
            .get(&Key::new(PROOF_GUARD_REGISTRY_KEY.to_string()))
        else {
            return BTreeMap::new();
        };
        let Value::Pack(pack) = value else {
            return BTreeMap::new();
        };
        pack.fields
            .iter()
            .filter_map(|(name, value)| match value {
                Value::Assertion(assertion) => Some((name.clone(), assertion.clone())),
                _ => None,
            })
            .collect()
    }

    fn save_proof_guard_registry(&mut self, guards: BTreeMap<String, AssertionValue>) {
        let key = Key::new(PROOF_GUARD_REGISTRY_KEY.to_string());
        if guards.is_empty() {
            self.state.resources.remove(&key);
            return;
        }
        let fields = guards
            .into_iter()
            .map(|(name, assertion)| (name, Value::Assertion(assertion)))
            .collect();
        self.state.set(key, Value::Pack(PackValue { fields }));
    }

    fn eval_add(
        &self,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        if l.dim != r.dim {
            return Err(RuntimeError::UnitMismatch { span });
        }
        Ok(Value::Num(Quantity::new(
            l.raw.saturating_add(r.raw),
            l.dim,
        )))
    }

    fn eval_sub(
        &self,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        if l.dim != r.dim {
            return Err(RuntimeError::UnitMismatch { span });
        }
        Ok(Value::Num(Quantity::new(
            l.raw.saturating_sub(r.raw),
            l.dim,
        )))
    }

    fn eval_mul(
        &self,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        Ok(Value::Num(Quantity::new(
            l.raw.saturating_mul(r.raw),
            l.dim.add(r.dim),
        )))
    }

    fn eval_div(
        &self,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        let raw = l
            .raw
            .checked_div(r.raw)
            .ok_or(RuntimeError::MathDivZero { span })?;
        let dim = l.dim.add(r.dim.scale(-1));
        Ok(Value::Num(Quantity::new(raw, dim)))
    }

    fn eval_mod(
        &self,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let (l, r) = self.require_numbers(left, right, span)?;
        if l.dim != r.dim {
            return Err(RuntimeError::UnitMismatch { span });
        }
        if r.raw.raw() == 0 {
            return Err(RuntimeError::MathDivZero { span });
        }
        let raw = Fixed64::from_raw(l.raw.raw() % r.raw.raw());
        Ok(Value::Num(Quantity::new(raw, l.dim)))
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

    fn bind_seed_args(
        &self,
        seed: &UserSeed,
        args: &[ArgBinding],
        values: &[Value],
        span: crate::lang::span::Span,
    ) -> Result<Vec<Value>, RuntimeError> {
        let mut bound: Vec<Option<Value>> = vec![None; seed.params.len()];
        let mut next_pos = 0usize;

        for (idx, arg) in args.iter().enumerate() {
            let value = values.get(idx).cloned().ok_or(RuntimeError::Pack {
                message: "인자 개수가 잘못되었습니다".to_string(),
                span,
            })?;
            if let Some(pin) = &arg.resolved_pin {
                let position = seed
                    .params
                    .iter()
                    .position(|param| param.name == *pin)
                    .ok_or(RuntimeError::Pack {
                        message: format!("알 수 없는 핀 '{}'", pin),
                        span,
                    })?;
                if bound[position].is_some() {
                    return Err(RuntimeError::Pack {
                        message: format!("핀 '{}'가 중복되었습니다", pin),
                        span,
                    });
                }
                bound[position] = Some(value);
                continue;
            }
            if let Some(josa) = &arg.josa {
                let mut candidates: Vec<usize> = seed
                    .params
                    .iter()
                    .enumerate()
                    .filter(|(_, param)| param.josa_list.iter().any(|item| item == josa))
                    .map(|(idx, _)| idx)
                    .collect();
                if candidates.is_empty() {
                    return Err(RuntimeError::Pack {
                        message: format!("조사 '{}'에 대응하는 매개변수가 없습니다", josa),
                        span,
                    });
                }
                let mut available: Vec<usize> = candidates
                    .drain(..)
                    .filter(|idx| bound[*idx].is_none())
                    .collect();
                if available.len() != 1 {
                    return Err(RuntimeError::Pack {
                        message: format!("조사 '{}'에 해당하는 매개변수가 여러 개입니다", josa),
                        span,
                    });
                }
                let position = available.pop().unwrap();
                bound[position] = Some(value);
                continue;
            }

            while next_pos < bound.len() && bound[next_pos].is_some() {
                next_pos += 1;
            }
            if next_pos >= bound.len() {
                return Err(RuntimeError::Pack {
                    message: "인자 개수가 너무 많습니다".to_string(),
                    span,
                });
            }
            bound[next_pos] = Some(value);
            next_pos += 1;
        }

        if bound.iter().any(|value| value.is_none()) {
            return Err(RuntimeError::Pack {
                message: "인자가 부족합니다".to_string(),
                span,
            });
        }

        Ok(bound.into_iter().map(|value| value.unwrap()).collect())
    }

    fn eval_user_seed(
        &mut self,
        seed_name: &str,
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
            let key = Key::new(param.name.clone());
            let prev_value = self.state.get(&key).cloned();
            prev.push((key.clone(), prev_value));
            self.state.set(key, value.clone());
        }

        let is_imja = matches!(&seed.kind, SeedKind::Named(kind) if kind == "임자");
        let is_immediate_proof = matches!(&seed.kind, SeedKind::Balhigi);
        let proof_contract_start = self.contract_diags.len();
        let proof_failure_start = self.diagnostic_failures.len();
        if is_imja {
            self.current_entity_stack.push(seed_name.to_string());
        }
        let flow = self.eval_block(&seed.body);
        if is_imja {
            self.current_entity_stack.pop();
        }

        for (key, prev_value) in prev {
            if let Some(value) = prev_value {
                self.state.set(key, value);
            } else {
                self.state.resources.remove(&key);
            }
        }

        match flow {
            Ok(FlowControl::Continue) => {
                if is_immediate_proof {
                    let error_code = self
                        .diagnostic_failures
                        .get(proof_failure_start)
                        .map(|failure| failure.code.clone())
                        .or_else(|| {
                            (self.contract_diags.len() > proof_contract_start)
                                .then(|| "E_PROOF_CHECK_FAILED".to_string())
                        });
                    let result = if error_code.is_some() {
                        "실패"
                    } else {
                        "성공"
                    };
                    self.record_immediate_proof_diag(seed_name, result, error_code);
                }
                Ok(Value::None)
            }
            Ok(FlowControl::Return(value, _)) => {
                if is_immediate_proof {
                    let error_code = self
                        .diagnostic_failures
                        .get(proof_failure_start)
                        .map(|failure| failure.code.clone())
                        .or_else(|| {
                            (self.contract_diags.len() > proof_contract_start)
                                .then(|| "E_PROOF_CHECK_FAILED".to_string())
                        });
                    let result = if error_code.is_some() {
                        "실패"
                    } else {
                        "성공"
                    };
                    self.record_immediate_proof_diag(seed_name, result, error_code);
                }
                Ok(value)
            }
            Ok(FlowControl::Break(span)) => {
                if is_immediate_proof {
                    self.record_immediate_proof_diag(
                        seed_name,
                        "실패",
                        Some("E_RUNTIME_BREAK_OUTSIDE_LOOP".to_string()),
                    );
                }
                Err(RuntimeError::BreakOutsideLoop { span })
            }
            Err(err) => {
                if is_immediate_proof {
                    self.record_immediate_proof_diag(
                        seed_name,
                        "실패",
                        Some(err.code().to_string()),
                    );
                }
                Err(err)
            }
        }
    }

    fn record_immediate_proof_diag(
        &mut self,
        seed_name: &str,
        result: &str,
        error_code: Option<String>,
    ) {
        self.diagnostics.push(DiagnosticRecord {
            tick: self.current_madi.get(),
            name: format!("밝히기 {}", seed_name),
            lhs: "0".to_string(),
            rhs: "0".to_string(),
            delta: "0".to_string(),
            threshold: "0".to_string(),
            result: result.to_string(),
            error_code: error_code.clone(),
        });
        self.proof_runtime.push(ProofRuntimeEvent::ProofBlock {
            tick: self.current_madi.get(),
            name: seed_name.to_string(),
            result: result.to_string(),
            error_code,
        });
    }

    fn record_solver_check_runtime(
        &mut self,
        query: &str,
        satisfied: Option<bool>,
        error_code: Option<String>,
        span: crate::lang::span::Span,
    ) {
        self.proof_runtime.push(ProofRuntimeEvent::SolverCheck {
            tick: self.current_madi.get(),
            query: query.to_string(),
            satisfied,
            error_code,
            span,
        });
    }

    fn record_solver_search_runtime(
        &mut self,
        operation: &str,
        query: &str,
        found: Option<bool>,
        value: Option<String>,
        error_code: Option<String>,
        span: crate::lang::span::Span,
    ) {
        self.proof_runtime.push(ProofRuntimeEvent::SolverSearch {
            tick: self.current_madi.get(),
            operation: operation.to_string(),
            query: query.to_string(),
            found,
            value,
            error_code,
            span,
        });
    }

    fn eval_imja_init(&mut self, seed_name: &str, seed: &UserSeed) -> Result<(), RuntimeError> {
        self.current_entity_stack.push(seed_name.to_string());
        for stmt in &seed.body {
            if matches!(stmt, Stmt::Receive { .. }) {
                continue;
            }
            ensure_no_break(self.eval_stmt(stmt)?)?;
        }
        self.current_entity_stack.pop();
        Ok(())
    }

    fn dispatch_signal_send(
        &mut self,
        sender_expr: Option<&Expr>,
        payload_expr: &Expr,
        receiver_expr: &Expr,
        span: crate::lang::span::Span,
    ) -> Result<(), RuntimeError> {
        let receiver_name = self.resolve_entity_name(receiver_expr, span)?;
        let sender_name = match sender_expr {
            Some(expr) => self.resolve_entity_name(expr, span)?,
            None => self
                .current_entity_name()
                .map(|name| name.to_string())
                .unwrap_or_else(|| "누리".to_string()),
        };
        let (event_kind, event_payload) = self.build_signal_payload(payload_expr, span)?;
        let Some(seed) = self.user_seeds.get(&receiver_name).cloned() else {
            return Err(RuntimeError::TypeMismatch {
                expected: "known 임자 receiver",
                span,
            });
        };
        if !matches!(&seed.kind, SeedKind::Named(kind) if kind == "임자") {
            return Err(RuntimeError::TypeMismatch {
                expected: "임자 receiver",
                span,
            });
        }
        self.pending_signals.push_back(PendingSignal {
            receiver_name,
            event_kind,
            sender_name,
            event_payload,
            span,
        });
        self.drain_signal_queue()
    }

    fn drain_signal_queue(&mut self) -> Result<(), RuntimeError> {
        if self.processing_signal_queue {
            return Ok(());
        }
        self.processing_signal_queue = true;
        let mut result = Ok(());
        while let Some(signal) = self.pending_signals.pop_front() {
            if let Err(err) = self.dispatch_pending_signal(signal) {
                self.pending_signals.clear();
                result = Err(err);
                break;
            }
        }
        self.processing_signal_queue = false;
        result
    }

    fn dispatch_pending_signal(&mut self, signal: PendingSignal) -> Result<(), RuntimeError> {
        let Some(seed) = self.user_seeds.get(&signal.receiver_name).cloned() else {
            return Err(RuntimeError::TypeMismatch {
                expected: "known 임자 receiver",
                span: signal.span,
            });
        };
        if !matches!(&seed.kind, SeedKind::Named(kind) if kind == "임자") {
            return Err(RuntimeError::TypeMismatch {
                expected: "임자 receiver",
                span: signal.span,
            });
        }
        self.eval_receive_handlers(
            &signal.receiver_name,
            &seed,
            &signal.event_kind,
            &signal.sender_name,
            signal.event_payload,
        )
    }

    fn build_signal_payload(
        &mut self,
        payload_expr: &Expr,
        span: crate::lang::span::Span,
    ) -> Result<(String, PackValue), RuntimeError> {
        let Expr::Call { name, args, .. } = payload_expr else {
            return Err(RuntimeError::TypeMismatch {
                expected: "알림씨 payload call",
                span,
            });
        };
        if !matches!(
            self.user_seeds.get(name),
            Some(UserSeed {
                kind: SeedKind::Named(kind),
                ..
            }) if kind == "알림씨"
        ) {
            return Err(RuntimeError::TypeMismatch {
                expected: "알림씨 payload call",
                span,
            });
        }
        if args.len() != 1 {
            return Err(RuntimeError::TypeMismatch {
                expected: "single pack payload argument",
                span,
            });
        }
        let Value::Pack(payload) = self.eval_expr(&args[0].expr)? else {
            return Err(RuntimeError::TypeMismatch {
                expected: "pack payload",
                span,
            });
        };
        Ok((name.clone(), payload))
    }

    fn resolve_entity_name(
        &self,
        expr: &Expr,
        span: crate::lang::span::Span,
    ) -> Result<String, RuntimeError> {
        match expr {
            Expr::Path(path) => {
                if path.segments.first().map(|s| s.as_str()) == Some("제") {
                    return self
                        .current_entity_name()
                        .map(|name| name.to_string())
                        .ok_or(RuntimeError::JeOutsideImja { span });
                }
                if path.segments.len() == 2
                    && matches!(path.segments[0].as_str(), "살림" | "바탕" | "샘")
                {
                    return Ok(path.segments[1].clone());
                }
                Err(RuntimeError::TypeMismatch {
                    expected: "entity path",
                    span,
                })
            }
            _ => Err(RuntimeError::TypeMismatch {
                expected: "entity path",
                span,
            }),
        }
    }

    fn eval_receive_handlers(
        &mut self,
        receiver_name: &str,
        seed: &UserSeed,
        event_kind: &str,
        sender_name: &str,
        event_payload: PackValue,
    ) -> Result<(), RuntimeError> {
        self.current_entity_stack.push(receiver_name.to_string());
        for rank in 0..4 {
            for stmt in &seed.body {
                let Stmt::Receive {
                    kind,
                    binding,
                    condition,
                    body,
                    ..
                } = stmt
                else {
                    continue;
                };
                if !self.receive_stmt_matches_rank(
                    rank,
                    kind.as_deref(),
                    binding,
                    condition,
                    event_kind,
                ) {
                    continue;
                }
                self.eval_receive_stmt(
                    binding.as_deref(),
                    condition.as_ref(),
                    body,
                    kind.as_deref(),
                    event_kind,
                    sender_name,
                    &event_payload,
                )?;
            }
        }
        self.current_entity_stack.pop();
        Ok(())
    }

    fn receive_stmt_matches_rank(
        &self,
        rank: u8,
        kind: Option<&str>,
        binding: &Option<String>,
        condition: &Option<Expr>,
        event_kind: &str,
    ) -> bool {
        let kind_matches = match kind {
            Some(kind_name) => kind_name == event_kind,
            None => true,
        };
        if !kind_matches {
            return false;
        }
        match rank {
            0 => kind.is_some() && binding.is_some() && condition.is_some(),
            1 => kind.is_some() && binding.is_none() && condition.is_none(),
            2 => kind.is_none() && binding.is_some() && condition.is_some(),
            3 => kind.is_none() && binding.is_none() && condition.is_none(),
            _ => false,
        }
    }

    fn eval_receive_stmt(
        &mut self,
        binding: Option<&str>,
        condition: Option<&Expr>,
        body: &[Stmt],
        kind: Option<&str>,
        event_kind: &str,
        sender_name: &str,
        event_payload: &PackValue,
    ) -> Result<(), RuntimeError> {
        let bound_value = match kind {
            Some(_) => Value::Pack(event_payload.clone()),
            None => {
                let mut fields = BTreeMap::new();
                fields.insert("이름".to_string(), Value::Str(event_kind.to_string()));
                fields.insert("보낸이".to_string(), Value::Str(sender_name.to_string()));
                fields.insert("정보".to_string(), Value::Pack(event_payload.clone()));
                Value::Pack(PackValue { fields })
            }
        };

        let restore = if let Some(name) = binding {
            let key = Key::new(name.to_string());
            let prev = self.state.get(&key).cloned();
            self.state.set(key.clone(), bound_value);
            Some((key, prev))
        } else {
            None
        };

        let should_run = if let Some(expr) = condition {
            let value = self.eval_expr(expr)?;
            is_truthy(&value, expr.span())?
        } else {
            true
        };
        if should_run {
            ensure_no_break(self.eval_block(body)?)?;
        }

        if let Some((key, prev)) = restore {
            if let Some(value) = prev {
                self.state.set(key, value);
            } else {
                self.state.resources.remove(&key);
            }
        }
        Ok(())
    }

    fn open_site_id(&self, span: crate::lang::span::Span) -> String {
        if self.open_source == "<memory>" {
            return "unknown".to_string();
        }
        let line_idx = span.start_line.saturating_sub(1);
        let Some(line) = self.open_source_lines.get(line_idx) else {
            return "unknown".to_string();
        };
        let col = utf16_col_for_line(line, span.start_col);
        format!("{}:{}:{}", self.open_source, line_idx, col)
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
                    return Ok(Value::Bool(if matches!(op, BinaryOp::Eq) {
                        equal
                    } else {
                        !equal
                    }));
                }
                (Value::Bool(a), Value::Bool(b)) => {
                    let equal = a == b;
                    return Ok(Value::Bool(if matches!(op, BinaryOp::Eq) {
                        equal
                    } else {
                        !equal
                    }));
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

fn utf16_col_for_line(line: &str, col1: usize) -> usize {
    if col1 <= 1 {
        return 0;
    }
    let mut col_utf16 = 0usize;
    let mut idx = 1usize;
    for ch in line.chars() {
        if idx >= col1 {
            break;
        }
        col_utf16 = col_utf16.saturating_add(ch.len_utf16());
        idx += 1;
    }
    col_utf16
}

#[derive(Clone, Debug)]
pub struct ContractDiag {
    pub kind: ContractKind,
    pub mode: ContractMode,
    pub message: String,
    pub span: crate::lang::span::Span,
}

impl Evaluator {
    fn emit_contract_violation(
        &mut self,
        kind: ContractKind,
        mode: ContractMode,
        span: crate::lang::span::Span,
    ) {
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

fn parse_pragma_invocation(name: &str, args: &str) -> (String, String) {
    let trimmed_name = name.trim();
    let trimmed_args = args.trim();
    if !trimmed_args.is_empty() {
        return (
            trimmed_name.to_string(),
            trim_outer_parens(trimmed_args).to_string(),
        );
    }
    if let Some(open) = trimmed_name.find('(') {
        if trimmed_name.ends_with(')') && open < trimmed_name.len() - 1 {
            let head = trimmed_name[..open].trim();
            let body = &trimmed_name[open + 1..trimmed_name.len() - 1];
            return (head.to_string(), body.trim().to_string());
        }
    }
    (trimmed_name.to_string(), trimmed_args.to_string())
}

fn trim_outer_parens(text: &str) -> &str {
    let trimmed = text.trim();
    if trimmed.starts_with('(') && trimmed.ends_with(')') && trimmed.len() >= 2 {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

fn split_top_level_args(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut in_quote = false;
    let mut escaped = false;
    let chars: Vec<char> = text.chars().collect();
    for (idx, ch) in chars.iter().enumerate() {
        if in_quote {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_quote = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => in_quote = true,
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                let part: String = chars[start..idx].iter().collect();
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
                start = idx + 1;
            }
            _ => {}
        }
    }
    if start < chars.len() {
        let part: String = chars[start..].iter().collect();
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }
    parts
}

fn split_option_pair(text: &str) -> Option<(&str, &str)> {
    let mut in_quote = false;
    let mut escaped = false;
    for (idx, ch) in text.char_indices() {
        if in_quote {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_quote = false,
                _ => {}
            }
            continue;
        }
        if ch == '"' {
            in_quote = true;
            continue;
        }
        if ch == '=' {
            return Some((&text[..idx], &text[idx + 1..]));
        }
    }
    None
}

fn parse_string_option(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return trimmed[1..trimmed.len() - 1].replace("\\\"", "\"");
    }
    trimmed.to_string()
}

fn parse_diagnostic_condition(
    text: &str,
    span: crate::lang::span::Span,
) -> Result<DiagnosticCondition, RuntimeError> {
    let trimmed = text.trim();
    let operators = [
        ("≈", DiagnosticOp::Approx),
        ("<=", DiagnosticOp::Le),
        (">=", DiagnosticOp::Ge),
        ("=", DiagnosticOp::Eq),
        ("<", DiagnosticOp::Lt),
        (">", DiagnosticOp::Gt),
    ];
    for (token, op) in operators {
        if let Some((lhs, rhs)) = trimmed.split_once(token) {
            let lhs = lhs.trim();
            let rhs = rhs.trim();
            if lhs.is_empty() || rhs.is_empty() {
                break;
            }
            return Ok(DiagnosticCondition {
                lhs_raw: lhs.to_string(),
                rhs_raw: rhs.to_string(),
                op,
            });
        }
    }
    Err(RuntimeError::Pack {
        message: format!("#진단 조건 해석 실패: {}", text),
        span,
    })
}

fn normalize_diag_state_key(input: &str) -> String {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("살림.") {
        return rest.to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("바탕.") {
        return rest.to_string();
    }
    trimmed.to_string()
}

fn splitmix64_next(state: u64) -> (u64, u64) {
    let next_state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = next_state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    (next_state, z ^ (z >> 31))
}

fn advance_dice(dice: &DiceValue) -> (DiceValue, u64) {
    let (state, value) = splitmix64_next(dice.state);
    (
        DiceValue {
            seed: dice.seed,
            state,
            draws: dice.draws.saturating_add(1),
        },
        value,
    )
}

fn quantity_to_int(qty: &Quantity, span: crate::lang::span::Span) -> Result<i64, RuntimeError> {
    if !qty.dim.is_dimensionless() {
        return Err(RuntimeError::TypeMismatch {
            expected: "dimensionless integer",
            span,
        });
    }
    let raw = qty.raw.raw();
    if raw & 0xFFFF_FFFF != 0 {
        return Err(RuntimeError::TypeMismatch {
            expected: "integer",
            span,
        });
    }
    Ok(raw >> 32)
}

fn find_dice_arg_index(args: &[ArgBinding], pin_names: &[&str]) -> Option<usize> {
    args.iter().position(|arg| {
        arg.resolved_pin
            .as_deref()
            .is_some_and(|pin| pin_names.iter().any(|candidate| pin == *candidate))
    })
}

fn dice_seed_from_args(
    evaluator: &mut Evaluator,
    args: &[ArgBinding],
    span: crate::lang::span::Span,
) -> Result<u64, RuntimeError> {
    let seed_index = find_dice_arg_index(args, &["시앗"]).unwrap_or(0);
    let arg = args.get(seed_index).ok_or(RuntimeError::TypeMismatch {
        expected: "시앗",
        span,
    })?;
    let value = evaluator.eval_expr(&arg.expr)?;
    let qty = match value {
        Value::Num(qty) => qty,
        Value::Pack(pack) => match pack.fields.get("시앗") {
            Some(Value::Num(qty)) => qty.clone(),
            Some(other) => return Err(type_mismatch_detail("number", other, span)),
            None => {
                return Err(RuntimeError::TypeMismatch {
                    expected: "시앗",
                    span,
                })
            }
        },
        other => return Err(type_mismatch_detail("number", &other, span)),
    };
    let seed = quantity_to_int(&qty, span)?;
    Ok(seed as u64)
}

fn dice_number_arg(
    evaluator: &mut Evaluator,
    args: &[ArgBinding],
    fallback_index: usize,
    pin_names: &[&str],
    span: crate::lang::span::Span,
) -> Result<Quantity, RuntimeError> {
    let index = find_dice_arg_index(args, pin_names).unwrap_or(fallback_index);
    let arg = args.get(index).ok_or(RuntimeError::TypeMismatch {
        expected: "number",
        span,
    })?;
    let value = evaluator.eval_expr(&arg.expr)?;
    match value {
        Value::Num(qty) => Ok(qty),
        other => Err(type_mismatch_detail("number", &other, span)),
    }
}

fn dice_list_arg(
    evaluator: &mut Evaluator,
    args: &[ArgBinding],
    fallback_index: usize,
    pin_names: &[&str],
    span: crate::lang::span::Span,
) -> Result<ListValue, RuntimeError> {
    let index = find_dice_arg_index(args, pin_names).unwrap_or(fallback_index);
    let arg = args.get(index).ok_or(RuntimeError::TypeMismatch {
        expected: "list",
        span,
    })?;
    let value = evaluator.eval_expr(&arg.expr)?;
    match value {
        Value::List(list) => Ok(list),
        other => Err(type_mismatch_detail("list", &other, span)),
    }
}

fn infer_diagnostic_failure_code(
    override_code: Option<&str>,
    name: &str,
    condition: &DiagnosticCondition,
) -> String {
    if let Some(code) = override_code {
        let trimmed = code.trim();
        if is_supported_diagnostic_error_code(trimmed) {
            return trimmed.to_string();
        }
    }
    if matches!(condition.op, DiagnosticOp::Eq) && is_sfc_condition(name, condition) {
        "E_SFC_IDENTITY_VIOLATION".to_string()
    } else {
        "E_ECO_DIVERGENCE_DETECTED".to_string()
    }
}

fn is_sfc_condition(name: &str, condition: &DiagnosticCondition) -> bool {
    let upper_name = name.to_ascii_uppercase();
    if upper_name.contains("SFC") {
        return true;
    }
    let lhs = condition.lhs_raw.as_str();
    let rhs = condition.rhs_raw.as_str();
    (lhs.contains("총수입") && rhs.contains("총지출"))
        || (lhs.contains("총지출") && rhs.contains("총수입"))
}

fn fixed64_abs(value: Fixed64) -> Fixed64 {
    if value.raw() < 0 {
        Fixed64::from_raw(value.raw().saturating_neg())
    } else {
        value
    }
}

fn is_supported_diagnostic_error_code(code: &str) -> bool {
    matches!(
        code.trim(),
        "E_ECO_DIVERGENCE_DETECTED" | "E_SFC_IDENTITY_VIOLATION"
    )
}

fn no_op_before_tick_with_open(
    _: u64,
    _: &mut State,
    _: &mut OpenRuntime,
) -> Result<(), RuntimeError> {
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
    pub diagnostics: Vec<DiagnosticRecord>,
    pub diagnostic_failures: Vec<DiagnosticFailure>,
    pub proof_runtime: Vec<ProofRuntimeEvent>,
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

fn is_dimensionless_integer_value(value: &Value) -> bool {
    match value {
        Value::Num(qty) => qty.dim.is_dimensionless() && (qty.raw.raw() & 0xFFFF_FFFF == 0),
        _ => false,
    }
}

fn parse_integer_like_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<i64, RuntimeError> {
    let text = parse_integer_like_text(value, span)?;
    text.parse::<i64>().map_err(|_| RuntimeError::TypeMismatch {
        expected: "integer range",
        span,
    })
}

fn parse_integer_like_text(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    match value {
        Value::Num(qty) => {
            if !qty.dim.is_dimensionless() {
                return Err(RuntimeError::UnitMismatch { span });
            }
            let raw = qty.raw.raw();
            if raw & 0xFFFF_FFFF != 0 {
                return Err(type_mismatch_detail("integer", value, span));
            }
            Ok((raw >> 32).to_string())
        }
        Value::Str(text) => normalize_integer_text(text).ok_or(RuntimeError::TypeMismatch {
            expected: "integer string",
            span,
        }),
        _ => Err(type_mismatch_detail("integer|string", value, span)),
    }
}

fn normalize_integer_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (sign, digits) = match trimmed.as_bytes()[0] {
        b'+' | b'-' => (&trimmed[0..1], &trimmed[1..]),
        _ => ("", trimmed),
    };
    if digits.is_empty() {
        return None;
    }
    let mut normalized = String::with_capacity(trimmed.len());
    normalized.push_str(sign);
    let mut has_digit = false;
    for ch in digits.chars() {
        if ch == '_' {
            continue;
        }
        if !ch.is_ascii_digit() {
            return None;
        }
        has_digit = true;
        normalized.push(ch);
    }
    if !has_digit {
        return None;
    }
    Some(normalized)
}

fn is_zero_integer_text(text: &str) -> bool {
    text.parse::<i128>().map(|value| value == 0).unwrap_or(false)
}

fn make_exact_numeric_value(kind: &'static str, fields: &[(&'static str, String)]) -> Value {
    let mut out = BTreeMap::new();
    out.insert(
        EXACT_NUMERIC_KIND_FIELD.to_string(),
        Value::Str(kind.to_string()),
    );
    for (key, value) in fields {
        out.insert((*key).to_string(), Value::Str(value.clone()));
    }
    Value::Pack(PackValue { fields: out })
}

fn exact_numeric_kind(value: &Value) -> Option<&str> {
    let Value::Pack(pack) = value else {
        return None;
    };
    match pack.fields.get(EXACT_NUMERIC_KIND_FIELD) {
        Some(Value::Str(kind)) => Some(kind.as_str()),
        _ => None,
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
        Value::Assertion(_) => "세움값".to_string(),
        Value::Lambda(_) => "seed".to_string(),
        Value::Dice(_) => "주사위씨".to_string(),
        Value::Pack(_) => exact_numeric_kind(value)
            .map(str::to_string)
            .unwrap_or_else(|| "pack".to_string()),
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

fn read_state_text(state: &State, key: &str) -> Option<String> {
    let value = state.get(&Key::new(key))?;
    match value {
        Value::Str(text) if !text.is_empty() => Some(text.clone()),
        _ => None,
    }
}

fn read_input_key_from_state(state: &State) -> Option<String> {
    if let Some(text) = read_state_text(state, "샘.입력키") {
        return Some(text);
    }
    if let Some(text) = read_state_text(state, "입력상태.입력키") {
        return Some(text);
    }
    if let Some(text) = read_state_text(state, "입력키") {
        return Some(text);
    }

    const INPUT_SCAN_ORDER: [&str; 9] = [
        "ArrowLeft",
        "ArrowRight",
        "ArrowDown",
        "ArrowUp",
        "Space",
        "Enter",
        "Escape",
        "KeyZ",
        "KeyX",
    ];

    for key in INPUT_SCAN_ORDER {
        if read_state_flag(state, &format!("샘.키보드.눌림.{}", key))
            || read_state_flag(state, &format!("입력상태.키_눌림.{}", key))
        {
            return Some(key.to_string());
        }
    }
    None
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

fn expect_quantity(
    values: &[Value],
    expected: usize,
    span: crate::lang::span::Span,
) -> Result<Quantity, RuntimeError> {
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

fn expect_two_quantities(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(Quantity, Quantity), RuntimeError> {
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

fn expect_three_quantities(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(Quantity, Quantity, Quantity), RuntimeError> {
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

fn expect_quantity_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Quantity, RuntimeError> {
    match value {
        Value::Num(qty) => Ok(qty.clone()),
        other => Err(type_mismatch_detail("number", other, span)),
    }
}

fn expect_pack_and_key(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(PackValue, String), RuntimeError> {
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

fn expect_map_and_key(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(MapValue, Value), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "map, key",
            span,
        });
    }
    let map = match &values[0] {
        Value::Map(map) => map.clone(),
        value => return Err(type_mismatch_detail("map", value, span)),
    };
    Ok((map, values[1].clone()))
}

fn expect_map_key_and_value(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(MapValue, Value, Value), RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "map, key, value",
            span,
        });
    }
    let map = match &values[0] {
        Value::Map(map) => map.clone(),
        value => return Err(type_mismatch_detail("map", value, span)),
    };
    Ok((map, values[1].clone(), values[2].clone()))
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

fn expect_single_string(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
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

fn expect_open_path(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
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

struct OpenSolverArgs {
    query: String,
    reply: Option<OpenSolverReply>,
}

struct OpenSolverSearchArgs {
    query: String,
    reply: Option<OpenSolverReply>,
}

fn expect_open_net_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<OpenNetArgs, RuntimeError> {
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

fn expect_open_ffi_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<OpenFfiArgs, RuntimeError> {
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

fn expect_open_gpu_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<OpenGpuArgs, RuntimeError> {
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

fn expect_open_solver_check_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<OpenSolverArgs, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "string or pack{질의}",
            span,
        });
    }
    match &values[0] {
        Value::Str(text) => Ok(OpenSolverArgs {
            query: text.clone(),
            reply: None,
        }),
        Value::Pack(pack) => {
            let query = expect_pack_string(pack, span, &["질의", "query"])?;
            let satisfied = get_pack_bool(pack, span, &["결과", "참", "satisfied"])?;
            Ok(OpenSolverArgs {
                query,
                reply: satisfied.map(|flag| OpenSolverReply::Check { satisfied: flag }),
            })
        }
        value => Err(type_mismatch_detail("string or pack", value, span)),
    }
}

fn expect_open_solver_search_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<OpenSolverSearchArgs, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "string or pack{질의}",
            span,
        });
    }
    match &values[0] {
        Value::Str(text) => Ok(OpenSolverSearchArgs {
            query: text.clone(),
            reply: None,
        }),
        Value::Pack(pack) => {
            let query = expect_pack_string(pack, span, &["질의", "query"])?;
            let found = get_pack_bool(pack, span, &["찾음", "found"])?;
            let value = get_pack_string(pack, span, &["값", "value"])?;
            let reply = found.map(|flag| OpenSolverReply::Search { found: flag, value });
            Ok(OpenSolverSearchArgs { query, reply })
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

fn get_pack_bool(
    pack: &PackValue,
    span: crate::lang::span::Span,
    keys: &[&str],
) -> Result<Option<bool>, RuntimeError> {
    let Some(value) = find_pack_value(pack, keys) else {
        return Ok(None);
    };
    match value {
        Value::Bool(flag) => Ok(Some(*flag)),
        other => Err(type_mismatch_detail("bool", other, span)),
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

fn expect_assertion_check_values(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(AssertionValue, PackValue), RuntimeError> {
    match values {
        [Value::Assertion(assertion), Value::Pack(pack)] => Ok((assertion.clone(), pack.clone())),
        [Value::Pack(pack)] => {
            let assertion = match find_pack_value(pack, &["세움", "assertion", "검사", "target"])
            {
                Some(Value::Assertion(assertion)) => assertion.clone(),
                Some(other) => return Err(type_mismatch_detail("세움값", other, span)),
                None => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "세움값",
                        span,
                    })
                }
            };
            let bindings = match find_pack_value(pack, &["값들", "bindings"]) {
                Some(Value::Pack(bindings)) => bindings.clone(),
                Some(other) => return Err(type_mismatch_detail("묶음값", other, span)),
                None => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "묶음값",
                        span,
                    })
                }
            };
            Ok((assertion, bindings))
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "세움값, 묶음값",
            span,
        }),
    }
}

fn expect_single_assertion(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<AssertionValue, RuntimeError> {
    match values {
        [Value::Assertion(assertion)] => Ok(assertion.clone()),
        [other] => Err(type_mismatch_detail("세움값", other, span)),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "세움값",
            span,
        }),
    }
}

fn extract_solver_search_runtime(value: &Value) -> (Option<bool>, Option<String>) {
    let Value::Pack(pack) = value else {
        return (None, None);
    };
    let found = match find_pack_value(pack, &["찾음", "found"]) {
        Some(Value::Bool(flag)) => Some(*flag),
        _ => None,
    };
    let found_value = match find_pack_value(pack, &["값", "value"]) {
        Some(Value::Str(text)) => Some(text.clone()),
        _ => None,
    };
    (found, found_value)
}

fn expect_two_strings(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(String, String), RuntimeError> {
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

fn make_regex_value(pattern: String, flags: String) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("__kind".to_string(), Value::Str("regex".to_string()));
    fields.insert("pattern".to_string(), Value::Str(pattern));
    fields.insert("flags".to_string(), Value::Str(flags));
    Value::Pack(PackValue { fields })
}

fn extract_regex_pattern(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<(String, String), RuntimeError> {
    match value {
        Value::Pack(pack) => {
            let pattern = match pack.fields.get("pattern") {
                Some(Value::Str(text)) => text.clone(),
                Some(other) => return Err(type_mismatch_detail("string", other, span)),
                None => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "regex",
                        span,
                    })
                }
            };
            let flags = match pack.fields.get("flags") {
                Some(Value::Str(text)) => text.clone(),
                Some(other) => return Err(type_mismatch_detail("string", other, span)),
                None => String::new(),
            };
            Ok((pattern, flags))
        }
        Value::Str(pattern) => Ok((pattern.clone(), String::new())),
        other => Err(type_mismatch_detail("regex", other, span)),
    }
}

fn compile_regex(
    pattern: &str,
    flags: &str,
    span: crate::lang::span::Span,
) -> Result<Regex, RuntimeError> {
    let mut builder = RegexBuilder::new(pattern);
    for flag in flags.chars() {
        match flag {
            'i' => {
                builder.case_insensitive(true);
            }
            'm' => {
                builder.multi_line(true);
            }
            's' => {
                builder.dot_matches_new_line(true);
            }
            'x' => {
                builder.ignore_whitespace(true);
            }
            'U' => {
                builder.swap_greed(true);
            }
            'u' => {
                builder.unicode(true);
            }
            _ => {
                return Err(RuntimeError::RegexFlagsInvalid {
                    flags: flags.to_string(),
                    span,
                });
            }
        }
    }
    builder
        .build()
        .map_err(|err| RuntimeError::RegexPatternInvalid {
            message: err.to_string(),
            span,
        })
}

fn regex_replacement_ref_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn validate_regex_replacement(
    regex: &Regex,
    replacement: &str,
    span: crate::lang::span::Span,
) -> Result<(), RuntimeError> {
    let names: BTreeSet<&str> = regex.capture_names().flatten().collect();
    let chars: Vec<char> = replacement.chars().collect();
    let mut idx = 0usize;
    while idx < chars.len() {
        if chars[idx] != '$' {
            idx += 1;
            continue;
        }
        if idx + 1 >= chars.len() {
            return Err(RuntimeError::RegexReplacementInvalid {
                replacement: replacement.to_string(),
                message: "치환 참조가 '$'에서 끝났습니다".to_string(),
                span,
            });
        }
        let next = chars[idx + 1];
        if next == '$' {
            idx += 2;
            continue;
        }
        if next == '{' {
            let mut end = idx + 2;
            while end < chars.len() && chars[end] != '}' {
                end += 1;
            }
            if end >= chars.len() {
                return Err(RuntimeError::RegexReplacementInvalid {
                    replacement: replacement.to_string(),
                    message: "치환 참조의 닫는 '}'가 없습니다".to_string(),
                    span,
                });
            }
            let token: String = chars[idx + 2..end].iter().collect();
            if token.is_empty() {
                return Err(RuntimeError::RegexReplacementInvalid {
                    replacement: replacement.to_string(),
                    message: "빈 치환 참조는 사용할 수 없습니다".to_string(),
                    span,
                });
            }
            if token.chars().all(|ch| ch.is_ascii_digit()) {
                let capture_index = token.parse::<usize>().unwrap_or(usize::MAX);
                if capture_index >= regex.captures_len() {
                    return Err(RuntimeError::RegexReplacementInvalid {
                        replacement: replacement.to_string(),
                        message: format!("알 수 없는 캡처 번호입니다: {}", token),
                        span,
                    });
                }
            } else if !names.contains(token.as_str()) {
                return Err(RuntimeError::RegexReplacementInvalid {
                    replacement: replacement.to_string(),
                    message: format!("알 수 없는 이름 캡처입니다: {}", token),
                    span,
                });
            }
            idx = end + 1;
            continue;
        }
        if next.is_ascii_digit() {
            let mut end = idx + 1;
            while end < chars.len() && chars[end].is_ascii_digit() {
                end += 1;
            }
            let token: String = chars[idx + 1..end].iter().collect();
            let capture_index = token.parse::<usize>().unwrap_or(usize::MAX);
            if capture_index >= regex.captures_len() {
                return Err(RuntimeError::RegexReplacementInvalid {
                    replacement: replacement.to_string(),
                    message: format!("알 수 없는 캡처 번호입니다: {}", token),
                    span,
                });
            }
            idx = end;
            continue;
        }
        if regex_replacement_ref_name_char(next) {
            let mut end = idx + 1;
            while end < chars.len() && regex_replacement_ref_name_char(chars[end]) {
                end += 1;
            }
            let token: String = chars[idx + 1..end].iter().collect();
            if !names.contains(token.as_str()) {
                return Err(RuntimeError::RegexReplacementInvalid {
                    replacement: replacement.to_string(),
                    message: format!("알 수 없는 이름 캡처입니다: {}", token),
                    span,
                });
            }
            idx = end;
            continue;
        }
        return Err(RuntimeError::RegexReplacementInvalid {
            replacement: replacement.to_string(),
            message: format!("지원하지 않는 치환 참조 시작입니다: ${}", next),
            span,
        });
    }
    Ok(())
}

fn regex_capture_first(regex: &Regex, text: &str) -> Vec<String> {
    let Some(captures) = regex.captures(text) else {
        return Vec::new();
    };
    captures
        .iter()
        .map(|item| item.map(|m| m.as_str().to_string()).unwrap_or_default())
        .collect()
}

fn regex_named_capture_first(regex: &Regex, text: &str) -> MapValue {
    let Some(captures) = regex.captures(text) else {
        return MapValue {
            entries: BTreeMap::new(),
        };
    };
    let mut entries = BTreeMap::new();
    for name in regex.capture_names().flatten() {
        let key = Value::Str(name.to_string());
        let value = Value::Str(
            captures
                .name(name)
                .map(|item| item.as_str().to_string())
                .unwrap_or_default(),
        );
        entries.insert(key.canon(), MapEntry { key, value });
    }
    MapValue { entries }
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

fn expect_quantity_list(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Vec<Quantity>, RuntimeError> {
    let list = expect_list(values, span)?;
    let mut out = Vec::with_capacity(list.items.len());
    for item in &list.items {
        out.push(expect_quantity_value(item, span)?);
    }
    if let Some(head) = out.first() {
        for item in out.iter().skip(1) {
            ensure_same_dim(head, item, span)?;
        }
    }
    Ok(out)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PercentileMode {
    Linear,
    NearestRank,
}

fn expect_list_and_percentile(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(Vec<Quantity>, Fixed64, PercentileMode), RuntimeError> {
    if values.len() != 2 && values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "list, percentile, mode?",
            span,
        });
    }
    let list = expect_quantity_list(&values[0..1], span)?;
    let percentile = expect_quantity_value(&values[1], span)?;
    if !percentile.dim.is_dimensionless() {
        return Err(RuntimeError::UnitMismatch { span });
    }
    let zero = Fixed64::zero();
    let one = Fixed64::one();
    if percentile.raw.raw() < zero.raw() || percentile.raw.raw() > one.raw() {
        return Err(RuntimeError::MathDomain {
            message: "분위수 p는 0..1 범위여야 합니다",
            span,
        });
    }
    let mode = if values.len() == 3 {
        parse_percentile_mode(&values[2], span)?
    } else {
        PercentileMode::Linear
    };
    Ok((list, percentile.raw, mode))
}

fn fixed64_to_nonnegative_index(
    value: Fixed64,
    span: crate::lang::span::Span,
) -> Result<usize, RuntimeError> {
    let raw = value.raw();
    if raw < 0 || (raw & ((1_i64 << Fixed64::SCALE_BITS) - 1)) != 0 {
        return Err(RuntimeError::MathDomain {
            message: "분위수 인덱스는 0 이상의 정수여야 합니다",
            span,
        });
    }
    Ok((raw >> Fixed64::SCALE_BITS) as usize)
}

fn parse_list_segment_index(
    field: &str,
    span: crate::lang::span::Span,
) -> Result<usize, RuntimeError> {
    field.parse::<usize>().map_err(|_| RuntimeError::Pack {
        message: format!("차림 인덱스는 숫자(.0/.1)만 허용됩니다: .{}", field),
        span,
    })
}

fn parse_percentile_mode(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<PercentileMode, RuntimeError> {
    let text = match value {
        Value::Str(text) => text.trim().to_string(),
        other => return Err(type_mismatch_detail("string mode", other, span)),
    };
    match text.as_str() {
        "선형보간" => Ok(PercentileMode::Linear),
        "최근순위" => Ok(PercentileMode::NearestRank),
        _ => Err(RuntimeError::MathDomain {
            message: "분위수 mode는 선형보간 또는 최근순위여야 합니다",
            span,
        }),
    }
}

fn nearest_rank_index(
    p: Fixed64,
    len: usize,
    span: crate::lang::span::Span,
) -> Result<usize, RuntimeError> {
    if len == 0 {
        return Err(RuntimeError::MathDomain {
            message: "분위수 입력 목록이 비어 있습니다",
            span,
        });
    }
    let p_raw = p.raw() as i128;
    let scale = Fixed64::SCALE as i128;
    let n = len as i128;
    let rank = if p_raw <= 0 {
        1
    } else {
        ((p_raw * n) + (scale - 1)) / scale
    };
    let clamped_rank = rank.clamp(1, n);
    Ok((clamped_rank - 1) as usize)
}

fn expect_list_and_item(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(ListValue, Value), RuntimeError> {
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

fn expect_list_and_index(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(ListValue, usize), RuntimeError> {
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

fn expect_tensor(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<TensorValue, RuntimeError> {
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
fn expect_list_and_delim(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(ListValue, String), RuntimeError> {
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
        _ => Err(type_mismatch_detail(
            "function or seed literal",
            value,
            span,
        )),
    }
}

fn expect_two_ints(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(i64, i64), RuntimeError> {
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

fn expect_template_and_pack(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<(TemplateValue, PackValue), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "template and pack",
            span,
        });
    }
    let template = match &values[0] {
        Value::Template(value) => value.clone(),
        value => return Err(type_mismatch_detail("template", value, span)),
    };
    let pack = match &values[1] {
        Value::Pack(value) => value.clone(),
        value => return Err(type_mismatch_detail("pack", value, span)),
    };
    Ok((template, pack))
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
                    message: format!(
                        "E_CALC_TRANSFORM_BAD_ORDER: {} 차수는 1 이상이어야 합니다",
                        label
                    ),
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
                if call_name == "diff" {
                    "미분하기"
                } else {
                    "적분하기"
                }
            ),
            span,
        });
    }

    let analysis =
        analyze_formula(&math.body, dialect).map_err(|err| map_formula_error(err, span))?;
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
                        if call_name == "diff" {
                            "미분하기"
                        } else {
                            "적분하기"
                        },
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
            format!(
                "{}({}, {}, {})",
                call_name, analysis.expr_text, var_name, order
            )
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
    let formatted =
        format_formula_body(&body, dialect).map_err(|err| map_formula_error(err, span))?;
    Ok(crate::core::value::MathValue {
        dialect: math.dialect,
        body: formatted,
    })
}

struct NumericFormulaPrepared {
    dialect: FormulaDialect,
    expr_text: String,
    var_name: String,
}

fn expect_numeric_derivative_args(
    values: &[Value],
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<(crate::core::value::MathValue, String, Quantity, Quantity), RuntimeError> {
    if values.len() != 4 {
        return Err(RuntimeError::TypeMismatch {
            expected: "formula, var, point, step",
            span,
        });
    }
    let math = match &values[0] {
        Value::Math(value) => value.clone(),
        other => return Err(type_mismatch_detail("formula", other, span)),
    };
    let raw_var = match &values[1] {
        Value::Str(text) => text.clone(),
        other => return Err(type_mismatch_detail("string var", other, span)),
    };
    let var_name = raw_var.trim().trim_start_matches('#').to_string();
    if var_name.is_empty() {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_NUMERIC_BAD_VAR: {} 변수 이름이 비어 있습니다",
                label
            ),
            span,
        });
    }
    let point = expect_quantity_value(&values[2], span)?;
    let step = expect_quantity_value(&values[3], span)?;
    ensure_same_dim(&point, &step, span)?;
    if step.raw.raw() == 0 {
        return Err(RuntimeError::MathDomain {
            message: "E_CALC_NUMERIC_BAD_STEP: 스텝은 0이 될 수 없습니다",
            span,
        });
    }
    Ok((math, var_name, point, step))
}

fn expect_numeric_integral_args(
    values: &[Value],
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<
    (
        crate::core::value::MathValue,
        String,
        Quantity,
        Quantity,
        Quantity,
    ),
    RuntimeError,
> {
    if values.len() != 5 {
        return Err(RuntimeError::TypeMismatch {
            expected: "formula, var, start, end, step",
            span,
        });
    }
    let math = match &values[0] {
        Value::Math(value) => value.clone(),
        other => return Err(type_mismatch_detail("formula", other, span)),
    };
    let raw_var = match &values[1] {
        Value::Str(text) => text.clone(),
        other => return Err(type_mismatch_detail("string var", other, span)),
    };
    let var_name = raw_var.trim().trim_start_matches('#').to_string();
    if var_name.is_empty() {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_NUMERIC_BAD_VAR: {} 변수 이름이 비어 있습니다",
                label
            ),
            span,
        });
    }
    let start = expect_quantity_value(&values[2], span)?;
    let end = expect_quantity_value(&values[3], span)?;
    let step = expect_quantity_value(&values[4], span)?;
    ensure_same_dim(&start, &end, span)?;
    ensure_same_dim(&start, &step, span)?;
    if step.raw.raw() <= 0 {
        return Err(RuntimeError::MathDomain {
            message: "E_CALC_NUMERIC_BAD_STEP: 스텝은 0보다 커야 합니다",
            span,
        });
    }
    Ok((math, var_name, start, end, step))
}

fn prepare_numeric_formula(
    math: &crate::core::value::MathValue,
    var_name: &str,
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<NumericFormulaPrepared, RuntimeError> {
    let Some(dialect) = FormulaDialect::from_tag(&math.dialect) else {
        return Err(RuntimeError::FormulaParse {
            message: format!("알 수 없는 수식 방언: {}", math.dialect),
            span,
        });
    };
    if dialect != FormulaDialect::Ascii {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_NUMERIC_DIALECT_UNSUPPORTED: {}는 #ascii 수식만 지원합니다",
                label
            ),
            span,
        });
    }
    ensure_formula_ident(var_name, dialect, "diff", span)?;
    let analysis =
        analyze_formula(&math.body, dialect).map_err(|err| map_formula_error(err, span))?;
    let mut vars = analysis.vars.clone();
    if let Some(assign) = &analysis.assign_name {
        vars.remove(assign);
    }
    if !vars.contains(var_name) {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_NUMERIC_FREEVAR_NOT_FOUND: {} 변수 '{}'가 수식에 없습니다",
                label, var_name
            ),
            span,
        });
    }
    if vars.len() != 1 {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_NUMERIC_FREEVAR_AMBIGUOUS: {} 변수 이름이 여러 개입니다",
                label
            ),
            span,
        });
    }
    Ok(NumericFormulaPrepared {
        dialect,
        expr_text: analysis.expr_text,
        var_name: var_name.to_string(),
    })
}

fn eval_numeric_formula_at(
    prepared: &NumericFormulaPrepared,
    x: Quantity,
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<Quantity, RuntimeError> {
    let mut env = BTreeMap::new();
    env.insert(prepared.var_name.clone(), x);
    eval_formula_body(&prepared.expr_text, prepared.dialect, &env).map_err(|err| {
        RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_NUMERIC_EVAL_FAIL: {} 수식 평가 실패 ({:?})",
                label, err
            ),
            span,
        }
    })
}

fn numeric_trapezoid_integral(
    prepared: &NumericFormulaPrepared,
    start: &Quantity,
    end: &Quantity,
    segments: usize,
    span: crate::lang::span::Span,
) -> Result<Quantity, RuntimeError> {
    if segments == 0 {
        return Err(RuntimeError::MathDomain {
            message: "E_CALC_NUMERIC_BAD_STEP: 적분.사다리꼴 구간 수가 0입니다",
            span,
        });
    }
    let interval = end.raw.saturating_sub(start.raw);
    let seg = Fixed64::from_int(segments as i64);
    let h = interval
        .checked_div(seg)
        .ok_or(RuntimeError::MathDivZero { span })?;

    let mut y_dim: Option<UnitDim> = None;
    let mut sum_raw = Fixed64::zero();
    let two = Fixed64::from_int(2);
    for idx in 0..=segments {
        let idx_raw = Fixed64::from_int(idx as i64);
        let x_raw = start.raw.saturating_add(h.saturating_mul(idx_raw));
        let x = Quantity::new(x_raw, start.dim);
        let y = eval_numeric_formula_at(prepared, x, span, "적분.사다리꼴")?;
        if let Some(dim) = y_dim {
            if y.dim != dim {
                return Err(RuntimeError::UnitMismatch { span });
            }
        } else {
            y_dim = Some(y.dim);
        }
        let weight = if idx == 0 || idx == segments {
            Fixed64::one()
        } else {
            two
        };
        sum_raw = sum_raw.saturating_add(y.raw.saturating_mul(weight));
    }
    let half = Fixed64::from_int(2);
    let approx = sum_raw
        .saturating_mul(h)
        .checked_div(half)
        .ok_or(RuntimeError::MathDivZero { span })?;
    Ok(Quantity::new(
        approx,
        y_dim.unwrap_or(UnitDim::zero()).add(start.dim),
    ))
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
                if call_name == "diff" {
                    "미분하기"
                } else {
                    "적분하기"
                }
            ),
            span,
        }),
        _ => Err(RuntimeError::FormulaParse {
            message: format!(
                "E_CALC_FREEVAR_AMBIGUOUS: {} 변수 이름이 여러 개입니다",
                if call_name == "diff" {
                    "미분하기"
                } else {
                    "적분하기"
                }
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
                if call_name == "diff" {
                    "미분하기"
                } else {
                    "적분하기"
                }
            ),
            span,
        });
    };
    if !first.is_ascii_alphabetic() {
        return Err(RuntimeError::FormulaParse {
            message: format!(
                "{} 변수 이름이 올바르지 않습니다: {}",
                if call_name == "diff" {
                    "미분하기"
                } else {
                    "적분하기"
                },
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
                if call_name == "diff" {
                    "미분하기"
                } else {
                    "적분하기"
                },
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

fn expect_tetris_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<TetrisArgs, RuntimeError> {
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
    let piece_id =
        i32::try_from(expect_int(&values[3], span)?).map_err(|_| RuntimeError::TypeMismatch {
            expected: "piece id",
            span,
        })?;
    let rotation =
        i32::try_from(expect_int(&values[4], span)?).map_err(|_| RuntimeError::TypeMismatch {
            expected: "rotation",
            span,
        })?;
    let x =
        i32::try_from(expect_int(&values[5], span)?).map_err(|_| RuntimeError::TypeMismatch {
            expected: "x",
            span,
        })?;
    let y =
        i32::try_from(expect_int(&values[6], span)?).map_err(|_| RuntimeError::TypeMismatch {
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

fn expect_tetris_cell_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<TetrisCellArgs, RuntimeError> {
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
    let x =
        i32::try_from(expect_int(&values[3], span)?).map_err(|_| RuntimeError::TypeMismatch {
            expected: "x",
            span,
        })?;
    let y =
        i32::try_from(expect_int(&values[4], span)?).map_err(|_| RuntimeError::TypeMismatch {
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

fn expect_tetris_drawlist_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<TetrisDrawListArgs, RuntimeError> {
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
    let origin_x =
        i32::try_from(expect_int(&values[3], span)?).map_err(|_| RuntimeError::TypeMismatch {
            expected: "origin_x",
            span,
        })?;
    let origin_y =
        i32::try_from(expect_int(&values[4], span)?).map_err(|_| RuntimeError::TypeMismatch {
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

fn expect_tetris_block_args(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<TetrisBlockArgs, RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "piece_id, rot, index",
            span,
        });
    }
    let piece_id =
        i32::try_from(expect_int(&values[0], span)?).map_err(|_| RuntimeError::TypeMismatch {
            expected: "piece id",
            span,
        })?;
    let rotation =
        i32::try_from(expect_int(&values[1], span)?).map_err(|_| RuntimeError::TypeMismatch {
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

fn tetris_can_place(
    args: &TetrisArgs,
    span: crate::lang::span::Span,
) -> Result<bool, RuntimeError> {
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

fn tetris_lock(
    args: &TetrisArgs,
    span: crate::lang::span::Span,
) -> Result<(String, i64, bool), RuntimeError> {
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

fn tetris_board_cell(
    args: &TetrisCellArgs,
    span: crate::lang::span::Span,
) -> Result<(String, String), RuntimeError> {
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

fn tetris_board_drawlist(
    args: &TetrisDrawListArgs,
    span: crate::lang::span::Span,
) -> Result<ListValue, RuntimeError> {
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

fn validate_board_dims(
    board: &str,
    width: usize,
    height: usize,
    span: crate::lang::span::Span,
) -> Result<(), RuntimeError> {
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

fn ensure_same_dim(
    a: &Quantity,
    b: &Quantity,
    span: crate::lang::span::Span,
) -> Result<(), RuntimeError> {
    if a.dim != b.dim {
        return Err(RuntimeError::UnitMismatch { span });
    }
    Ok(())
}

fn ensure_dimensionless(
    value: &Quantity,
    span: crate::lang::span::Span,
) -> Result<(), RuntimeError> {
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
    if dim.length != 0 || dim.time != 0 || dim.mass != 0 || dim.pixel != 0 || dim.temperature != 0 {
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
        || dim.temperature % 2 != 0
    {
        return Err(RuntimeError::UnitMismatch { span });
    }
    Ok(UnitDim {
        length: dim.length / 2,
        time: dim.time / 2,
        mass: dim.mass / 2,
        angle: dim.angle / 2,
        pixel: dim.pixel / 2,
        temperature: dim.temperature / 2,
    })
}

fn normalize_temperature_literal(unit_expr: &UnitExpr, base: Fixed64) -> Option<Fixed64> {
    let factor = match unit_expr.factors.as_slice() {
        [factor] if factor.exp == 1 => factor,
        _ => return None,
    };
    match factor.name.as_str() {
        "C" => Some(base.saturating_add(fixed_literal("273.15"))),
        "F" => {
            let shifted = base.saturating_sub(fixed_literal("32"));
            let scaled = shifted
                .saturating_mul(fixed_literal("5"))
                .checked_div(fixed_literal("9"))
                .unwrap_or_else(|| Fixed64::from_raw(i64::MAX));
            Some(scaled.saturating_add(fixed_literal("273.15")))
        }
        _ => None,
    }
}

fn fixed_literal(text: &str) -> Fixed64 {
    Fixed64::parse_literal(text).expect("fixed literal")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::lexer::Lexer;
    use crate::lang::parser::Parser;
    use crate::runtime::open::{OpenInputFrame, OpenMode, OpenRuntime};

    fn run_source_once(source: &str) -> Result<EvalOutput, RuntimeError> {
        let tokens = Lexer::tokenize(source).expect("lex");
        let default_root = Parser::default_root_for_source(source);
        let program = Parser::parse_with_default_root(tokens, default_root).expect("parse");
        let evaluator = Evaluator::with_state_and_seed(State::new(), 42);
        evaluator.run_with_ticks(&program, 1)
    }

    fn run_source_ticks(source: &str, ticks: u64) -> Result<EvalOutput, RuntimeError> {
        let tokens = Lexer::tokenize(source).expect("lex");
        let default_root = Parser::default_root_for_source(source);
        let program = Parser::parse_with_default_root(tokens, default_root).expect("parse");
        let evaluator = Evaluator::with_state_and_seed(State::new(), 42);
        evaluator.run_with_ticks(&program, ticks)
    }

    fn state_num(output: &EvalOutput, key: &str) -> Fixed64 {
        let value = output
            .state
            .get(&Key::new(key.to_string()))
            .expect("state key");
        match value {
            Value::Num(quantity) => quantity.raw,
            other => panic!("expected number, got {}", other.display()),
        }
    }

    fn state_bool(output: &EvalOutput, key: &str) -> bool {
        let value = output
            .state
            .get(&Key::new(key.to_string()))
            .expect("state key");
        match value {
            Value::Bool(flag) => *flag,
            other => panic!("expected bool, got {}", other.display()),
        }
    }

    fn state_str(output: &EvalOutput, key: &str) -> String {
        let value = output
            .state
            .get(&Key::new(key.to_string()))
            .expect("state key");
        match value {
            Value::Str(text) => text.clone(),
            other => panic!("expected string, got {}", other.display()),
        }
    }

    fn state_display(output: &EvalOutput, key: &str) -> String {
        output
            .state
            .get(&Key::new(key.to_string()))
            .expect("state key")
            .display()
    }

    fn trace_logs(output: &EvalOutput) -> Vec<String> {
        output
            .trace
            .log_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect()
    }

    fn state_list_strings(output: &EvalOutput, key: &str) -> Vec<String> {
        let value = output
            .state
            .get(&Key::new(key.to_string()))
            .expect("state key");
        let Value::List(list) = value else {
            panic!("expected list, got {}", value.display());
        };
        list.items
            .iter()
            .map(|item| match item {
                Value::Str(text) => text.clone(),
                other => panic!("expected string item, got {}", other.display()),
            })
            .collect()
    }

    fn fixed(text: &str) -> Fixed64 {
        Fixed64::parse_literal(text).expect("fixed literal")
    }

    fn state_quantity(output: &EvalOutput, key: &str) -> Quantity {
        let value = output
            .state
            .get(&Key::new(key.to_string()))
            .expect("state key");
        match value {
            Value::Num(quantity) => quantity.clone(),
            other => panic!("expected quantity, got {}", other.display()),
        }
    }

    fn assert_fixed_close(actual: Fixed64, expected: Fixed64, max_raw_diff: i64) {
        let diff = (actual.raw() - expected.raw()).abs();
        assert!(
            diff <= max_raw_diff,
            "fixed mismatch actual={} expected={} diff={}",
            actual.format(),
            expected.format(),
            diff
        );
    }

    #[test]
    fn hook_every_n_madi_runs_on_interval_ticks() {
        let source = r#"
전체 <- 0.
주기 <- 0.
(매마디)마다 {
  전체 <- 전체 + 1.
}.
(2마디)마다 {
  주기 <- 주기 + 1.
}.
"#;
        let output = run_source_ticks(source, 5).expect("run");
        assert_eq!(state_num(&output, "전체"), Fixed64::from_int(5));
        assert_eq!(state_num(&output, "주기"), Fixed64::from_int(3));
    }

    #[test]
    fn condition_hooks_becomes_and_while_work_on_tick_flow() {
        let source = r#"
값 <- 0.
경계횟수 <- 0.
동안횟수 <- 0.
(값 >= 2)이 될때 {
  경계횟수 <- 경계횟수 + 1.
}.
(값 >= 2)인 동안 {
  동안횟수 <- 동안횟수 + 1.
}.
(매마디)마다 {
  값 <- 값 + 1.
}.
"#;
        let output = run_source_ticks(source, 5).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(5));
        assert_eq!(state_num(&output, "경계횟수"), Fixed64::from_int(1));
        assert_eq!(state_num(&output, "동안횟수"), Fixed64::from_int(4));
    }

    #[test]
    fn end_hook_runs_after_tick_loop() {
        let source = r#"
값 <- 0.
끝값 <- -1.
(매마디)마다 {
  값 <- 값 + 1.
}.
(끝)할때 {
  끝값 <- 값.
}.
"#;
        let output = run_source_ticks(source, 3).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(3));
        assert_eq!(state_num(&output, "끝값"), Fixed64::from_int(3));
    }

    #[test]
    fn end_hook_runs_even_with_zero_ticks() {
        let source = r#"
끝값 <- 0.
(끝)할때 {
  끝값 <- 7.
}.
"#;
        let output = run_source_ticks(source, 0).expect("run");
        assert_eq!(state_num(&output, "끝값"), Fixed64::from_int(7));
    }

    #[test]
    fn nuri_reset_restores_initial_world_snapshot() {
        let source = r#"
기준 <- 7.
값 <- 0.
(시작)할때 {
  값 <- 기준.
}.
(매마디)마다 {
  값 <- 값 + 1.
  누리다시.
}.
"#;
        let output = run_source_ticks(source, 3).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(7));
    }

    #[test]
    fn madang_reset_restores_initial_world_snapshot() {
        let source = r#"
기준 <- 11.
값 <- 0.
(시작)할때 {
  값 <- 기준.
}.
(매마디)마다 {
  값 <- 값 + 2.
  마당다시.
}.
"#;
        let output = run_source_ticks(source, 3).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(11));
    }

    #[test]
    fn pan_reset_restores_initial_world_snapshot() {
        let source = r#"
기준 <- 13.
값 <- 0.
(시작)할때 {
  값 <- 기준.
}.
(매마디)마다 {
  값 <- 값 + 3.
  판다시.
}.
"#;
        let output = run_source_ticks(source, 3).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(13));
    }

    #[test]
    fn all_reset_restores_initial_world_snapshot() {
        let source = r#"
기준 <- 17.
값 <- 0.
(시작)할때 {
  값 <- 기준.
}.
(매마디)마다 {
  값 <- 값 + 4.
  모두다시.
}.
"#;
        let output = run_source_ticks(source, 3).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(17));
    }

    #[test]
    fn lifecycle_blocks_are_parse_only_in_gate0_runtime() {
        let source = r#"
값 <- 1.
판 {
  값 <- 99.
}.
마당 {
  값 <- 77.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(1));
    }

    #[test]
    fn lifecycle_transition_verbs_are_noop_in_gate0_runtime() {
        let source = r#"
값 <- 1.
진자마당 시작하기.
다음마당 넘어가기.
연습판 불러오기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(1));
    }

    #[test]
    fn bare_lifecycle_transition_verbs_are_accepted_as_zero_arg_calls() {
        let source = r#"
시작하기.
넘어가기.
불러오기.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "끝"), Fixed64::from_int(1));
    }

    #[test]
    fn lifecycle_transition_verb_call_accepts_string_target() {
        let source = r#"
("다음마당") 넘어가기.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "끝"), Fixed64::from_int(1));
    }

    #[test]
    fn lifecycle_transition_verb_call_rejects_non_string_target() {
        let source = r#"
(1) 시작하기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_TYPE_MISMATCH");
    }

    #[test]
    fn lifecycle_transition_rejects_more_than_one_target_arg() {
        let source = r#"
("첫마당", "둘째마당") 시작하기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_LIFECYCLE_START_TARGET_ARITY");
    }

    #[test]
    fn lifecycle_next_rejects_more_than_one_target_arg() {
        let source = r#"
("첫마당", "둘째마당") 넘어가기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_LIFECYCLE_NEXT_TARGET_ARITY");
    }

    #[test]
    fn lifecycle_call_rejects_more_than_one_target_arg() {
        let source = r#"
("첫판", "둘째판") 불러오기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_LIFECYCLE_CALL_TARGET_ARITY");
    }

    #[test]
    fn lifecycle_transition_rejects_unknown_target_without_family_hint() {
        let source = r#"
값 <- 0.
마당 {
  값 <- 값 + 10.
}.
("없는대상") 시작하기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_LIFECYCLE_START_TARGET_UNKNOWN");
    }

    #[test]
    fn lifecycle_next_rejects_unknown_target_without_family_hint() {
        let source = r#"
값 <- 0.
마당 {
  값 <- 값 + 10.
}.
("없는대상") 넘어가기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_LIFECYCLE_NEXT_TARGET_UNKNOWN");
    }

    #[test]
    fn lifecycle_call_rejects_unknown_target_without_family_hint() {
        let source = r#"
값 <- 0.
판 {
  값 <- 값 + 1.
}.
("없는대상") 불러오기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_LIFECYCLE_CALL_TARGET_UNKNOWN");
    }

    #[test]
    fn lifecycle_start_rejects_target_family_conflict() {
        let source = r#"
값 <- 0.
이상한마당 = 판 {
  값 <- 값 + 1.
}.
("이상한마당") 시작하기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(
            err.code(),
            "E_RUNTIME_LIFECYCLE_START_TARGET_FAMILY_CONFLICT"
        );
    }

    #[test]
    fn lifecycle_next_rejects_target_family_conflict() {
        let source = r#"
값 <- 0.
이상한마당 = 판 {
  값 <- 값 + 1.
}.
("이상한마당") 넘어가기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(
            err.code(),
            "E_RUNTIME_LIFECYCLE_NEXT_TARGET_FAMILY_CONFLICT"
        );
    }

    #[test]
    fn lifecycle_call_rejects_target_family_conflict() {
        let source = r#"
값 <- 0.
이상한판 = 마당 {
  값 <- 값 + 10.
}.
("이상한판") 불러오기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(
            err.code(),
            "E_RUNTIME_LIFECYCLE_CALL_TARGET_FAMILY_CONFLICT"
        );
    }

    #[test]
    fn lifecycle_start_rejects_target_family_ambiguous() {
        let source = r#"
값 <- 0.
마당 {
  값 <- 값 + 10.
}.
("혼합판마당") 시작하기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(
            err.code(),
            "E_RUNTIME_LIFECYCLE_START_TARGET_FAMILY_AMBIGUOUS"
        );
    }

    #[test]
    fn lifecycle_next_rejects_target_family_ambiguous() {
        let source = r#"
값 <- 0.
마당 {
  값 <- 값 + 10.
}.
("혼합판마당") 넘어가기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(
            err.code(),
            "E_RUNTIME_LIFECYCLE_NEXT_TARGET_FAMILY_AMBIGUOUS"
        );
    }

    #[test]
    fn lifecycle_call_rejects_target_family_ambiguous() {
        let source = r#"
값 <- 0.
판 {
  값 <- 값 + 1.
}.
("혼합판마당") 불러오기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(
            err.code(),
            "E_RUNTIME_LIFECYCLE_CALL_TARGET_FAMILY_AMBIGUOUS"
        );
    }

    #[test]
    fn lifecycle_transition_allows_unknown_target_when_no_units_exist() {
        let source = r#"
("없는대상") 시작하기.
("없는대상") 넘어가기.
("없는대상") 불러오기.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "끝"), Fixed64::from_int(1));
    }

    #[test]
    fn lifecycle_transition_allows_ambiguous_target_when_no_units_exist() {
        let source = r#"
("혼합판마당") 시작하기.
("혼합판마당") 넘어가기.
("혼합판마당") 불러오기.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "끝"), Fixed64::from_int(1));
    }

    #[test]
    fn lifecycle_start_executes_first_madang_unit() {
        let source = r#"
값 <- 0.
마당 {
  값 <- 값 + 10.
}.
마당 {
  값 <- 값 + 100.
}.
시작하기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(10));
    }

    #[test]
    fn lifecycle_next_advances_to_next_madang_unit() {
        let source = r#"
값 <- 0.
마당 {
  값 <- 값 + 10.
}.
마당 {
  값 <- 값 + 100.
}.
시작하기.
넘어가기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(110));
    }

    #[test]
    fn lifecycle_call_replays_active_pan_unit_without_advancing() {
        let source = r#"
값 <- 0.
판 {
  값 <- 값 + 1.
}.
판 {
  값 <- 값 + 100.
}.
("연습판") 시작하기.
("연습판") 불러오기.
("연습판") 넘어가기.
("연습판") 불러오기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(202));
    }

    #[test]
    fn lifecycle_transition_target_suffix_routes_family() {
        let source = r#"
값 <- 0.
판 {
  값 <- 값 + 1.
}.
마당 {
  값 <- 값 + 10.
}.
("진자마당") 시작하기.
("연습판") 시작하기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(11));
    }

    #[test]
    fn lifecycle_start_uses_named_madang_mapping() {
        let source = r#"
값 <- 0.
첫마당 = 마당 {
  값 <- 값 + 10.
}.
다음마당 = 마당 {
  값 <- 값 + 100.
}.
("다음마당") 시작하기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(100));
    }

    #[test]
    fn lifecycle_next_uses_named_madang_mapping() {
        let source = r#"
값 <- 0.
첫마당 = 마당 {
  값 <- 값 + 10.
}.
다음마당 = 마당 {
  값 <- 값 + 100.
}.
("첫마당") 시작하기.
("다음마당") 넘어가기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(110));
    }

    #[test]
    fn lifecycle_next_wraps_to_first_madang_when_active_is_last() {
        let source = r#"
값 <- 0.
첫마당 = 마당 {
  값 <- 값 + 10.
}.
다음마당 = 마당 {
  값 <- 값 + 100.
}.
("다음마당") 시작하기.
넘어가기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(110));
    }

    #[test]
    fn lifecycle_call_uses_named_pan_mapping() {
        let source = r#"
값 <- 0.
첫판 = 판 {
  값 <- 값 + 1.
}.
다음판 = 판 {
  값 <- 값 + 100.
}.
("다음판") 시작하기.
("다음판") 불러오기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(200));
    }

    #[test]
    fn lifecycle_next_wraps_to_first_pan_when_active_is_last() {
        let source = r#"
값 <- 0.
판 {
  값 <- 값 + 1.
}.
판 {
  값 <- 값 + 100.
}.
("연습판") 시작하기.
("연습판") 넘어가기.
("연습판") 넘어가기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(102));
    }

    #[test]
    fn bogae_reset_does_not_change_state_hash() {
        let plain = run_source_once(
            r#"
값 <- 1.
"#,
        )
        .expect("plain run");
        let with_reset = run_source_once(
            r#"
값 <- 1.
보개다시.
"#,
        )
        .expect("with reset run");
        assert_eq!(
            crate::core::hash::state_hash(&plain.state),
            crate::core::hash::state_hash(&with_reset.state)
        );
        assert_eq!(state_num(&plain, "값"), state_num(&with_reset, "값"));
    }

    #[test]
    fn proof_guard_registration_runs_each_tick_and_unregisters() {
        let source = r#"
x <- 0.
검사 <- 세움{
  { x >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
(시작)할때 {
  검사 지키기.
}.
(매마디)마다 {
  x <- x + 1.
}.
"#;
        let output = run_source_ticks(source, 2).expect("run");
        assert_eq!(output.proof_runtime.len(), 2);
        assert!(output
            .proof_runtime
            .iter()
            .all(|event| matches!(event, ProofRuntimeEvent::ProofCheck { passed: true, .. })));
        assert!(output
            .state
            .get(&Key::new(PROOF_GUARD_REGISTRY_KEY.to_string()))
            .is_some());

        let unregister_source = r#"
x <- 0.
검사 <- 세움{
  { x >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
(시작)할때 {
  검사 지키기.
  검사 지키기 끔.
}.
(매마디)마다 {
  x <- x + 1.
}.
"#;
        let unregistered = run_source_ticks(unregister_source, 2).expect("run");
        assert!(unregistered.proof_runtime.is_empty());
        assert!(unregistered
            .state
            .get(&Key::new(PROOF_GUARD_REGISTRY_KEY.to_string()))
            .is_none());
    }

    #[test]
    fn proof_guard_abort_rolls_back_tick_state() {
        let source = r#"
x <- 0.
검사 <- 세움{
  { x <= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
(시작)할때 {
  검사 지키기.
}.
(매마디)마다 {
  x <- 1.
}.
"#;
        let output = run_source_ticks(source, 1).expect("run");
        assert_eq!(state_num(&output, "x"), Fixed64::from_int(0));
        assert_eq!(output.proof_runtime.len(), 1);
        match &output.proof_runtime[0] {
            ProofRuntimeEvent::ProofCheck { passed, .. } => assert!(!passed),
            other => panic!("unexpected event: {other:?}"),
        }
        assert_eq!(output.contract_diags.len(), 1);
        assert!(matches!(output.contract_diags[0].mode, ContractMode::Abort));
    }

    #[test]
    fn proof_guard_alert_keeps_tick_state() {
        let source = r#"
x <- 0.
검사 <- 세움{
  { x <= 0 }인것 바탕으로(알림) 아니면 {
    없음.
  }.
}.
(시작)할때 {
  검사 지키기.
}.
(매마디)마다 {
  x <- 1.
}.
"#;
        let output = run_source_ticks(source, 1).expect("run");
        assert_eq!(state_num(&output, "x"), Fixed64::from_int(1));
        assert_eq!(output.proof_runtime.len(), 1);
        match &output.proof_runtime[0] {
            ProofRuntimeEvent::ProofCheck { passed, .. } => assert!(!passed),
            other => panic!("unexpected event: {other:?}"),
        }
        assert_eq!(output.contract_diags.len(), 1);
        assert!(matches!(output.contract_diags[0].mode, ContractMode::Alert));
    }

    #[test]
    fn input_key_compat_returns_empty_string_when_missing() {
        let source = r#"
값 <- () 입력키.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "값"), "");
    }

    #[test]
    fn input_key_option_returns_none_when_missing() {
        let source = r#"
값 <- () 입력키?.
"#;
        let output = run_source_once(source).expect("run");
        let value = output
            .state
            .get(&Key::new("값".to_string()))
            .expect("state key");
        assert!(matches!(value, Value::None));
    }

    #[test]
    fn input_key_strict_fails_when_missing() {
        let source = r#"
값 <- () 입력키!.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_INPUTKEY_MISSING");
    }

    #[test]
    fn input_key_reads_existing_state_value() {
        let source = r#"
살림.입력키 <- "ArrowRight".
값 <- () 입력키!.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "값"), "ArrowRight");
    }

    #[test]
    fn string_replace_at_safe_returns_original_when_out_of_range() {
        let source = r#"
값 <- ("가나다", 9, "라") 글바꾸기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "값"), "가나다");
    }

    #[test]
    fn string_replace_at_strict_returns_dedicated_oob_error() {
        let source = r#"
값 <- ("가나다", 9, "라") 글바꾸기!.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_STR_INDEX_OOB");
    }

    #[test]
    fn dice_builtins_mutate_stateful_seed_path_deterministically() {
        let source = r##"
운명 <- (시앗=1234) 주사위씨.만들기.
첫 <- (운명, 최소=1, 최대=10) 주사위씨.뽑기.
둘 <- (운명, 최소=1, 최대=10) 주사위씨.뽑기.
셋 <- (운명, 후보=["검", "활", "창"]) 주사위씨.골라뽑기.
넷 <- (운명, 최소=0.0, 최대=1.0) 주사위씨.실수뽑기.
"##;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "첫"), fixed("6"));
        assert_eq!(state_num(&output, "둘"), fixed("5"));
        assert_eq!(state_str(&output, "셋"), "활");
        assert_eq!(state_num(&output, "넷"), Fixed64::from_raw(1_389_404_211));
        assert_eq!(
            state_display(&output, "운명"),
            "주사위씨{시앗=0x00000000000004d2, 상태=0x78dde6e5fd29f526, 횟수=4}"
        );
    }

    #[test]
    fn dice_make_with_same_seed_produces_same_state_hash() {
        let source = r#"
운명 <- (시앗=99) 주사위씨.만들기.
값1 <- (운명, 최소=1, 최대=6) 주사위씨.뽑기.
값2 <- (운명, 최소=1, 최대=6) 주사위씨.뽑기.
"#;
        let first = run_source_once(source).expect("run first");
        let second = run_source_once(source).expect("run second");
        assert_eq!(
            crate::core::hash::state_hash(&first.state),
            crate::core::hash::state_hash(&second.state)
        );
        assert_eq!(
            state_display(&first, "운명"),
            state_display(&second, "운명")
        );
        assert_eq!(state_num(&first, "값1"), state_num(&second, "값1"));
        assert_eq!(state_num(&first, "값2"), state_num(&second, "값2"));
    }

    #[test]
    fn regex_builtins_run_basic_contract() {
        let source = r#"
패턴 <- 정규식{"^[A-Z]{2}[0-9]+$", "i"}.
맞음 <- ("ab12", 패턴) 정규맞추기.
첫매치 <- ("x12y34", 정규식{"[0-9]+"}) 정규찾기.
캡처 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}) 정규캡처하기.
이름캡처 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}) 정규이름캡처하기.
이름선택캡처 <- ("ab", 정규식{"(?P<word>[a-z]+)(?:-(?P<num>[0-9]+))?", "i"}) 정규이름캡처하기.
이름캡처없음 <- ("12", 정규식{"(?P<word>[a-z]+)", "i"}) 정규이름캡처하기.
바꿈 <- ("a1b2", 정규식{"[0-9]+"}, "_") 정규바꾸기.
재바꿈 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}, "$2:$1") 정규바꾸기.
이름재바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "${num}:${word}") 정규바꾸기.
이름짧은재바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "$num:$word") 정규바꾸기.
조각 <- ("a1b22c", 정규식{"[0-9]+"}) 정규나누기.
캡처조각 <- ("ab-12-cd", 정규식{"(-)([0-9]+)(-)"}) 정규나누기.
없음찾기 <- ("abc", 정규식{"[0-9]+"}) 정규찾기.
"#;
        let output = run_source_once(source).expect("run");
        assert!(state_bool(&output, "맞음"));
        assert_eq!(state_str(&output, "첫매치"), "12");
        assert_eq!(
            state_list_strings(&output, "캡처"),
            vec!["ab-12".to_string(), "ab".to_string(), "12".to_string()]
        );
        assert_eq!(
            state_display(&output, "이름캡처"),
            "짝맞춤{num=>12, word=>ab}"
        );
        assert_eq!(
            state_display(&output, "이름선택캡처"),
            "짝맞춤{num=>, word=>ab}"
        );
        assert_eq!(state_display(&output, "이름캡처없음"), "짝맞춤{}");
        assert_eq!(state_str(&output, "바꿈"), "a_b_");
        assert_eq!(state_str(&output, "재바꿈"), "12:ab");
        assert_eq!(state_str(&output, "이름재바꿈"), "12:ab");
        assert_eq!(state_str(&output, "이름짧은재바꿈"), "12:ab");
        assert_eq!(
            state_list_strings(&output, "조각"),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(
            state_list_strings(&output, "캡처조각"),
            vec!["ab".to_string(), "cd".to_string()]
        );
        let none_value = output
            .state
            .get(&Key::new("없음찾기".to_string()))
            .expect("state key");
        assert!(matches!(none_value, Value::None));
    }

    #[test]
    fn regex_flags_invalid_returns_dedicated_error() {
        let source = r#"
패턴 <- 정규식{"[0-9]+", "z"}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_REGEX_FLAGS_INVALID");
    }

    #[test]
    fn choose_exhaustive_without_match_returns_proof_incomplete() {
        let source = r#"
고르기:
{ 1 == 2 } 인 경우 {
  살림.x <- 1.
}
모든 경우 다룸.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_PROOF_INCOMPLETE");
    }

    #[test]
    fn choose_canonical_else_runs_fallback_body() {
        let source = r#"
고르기:
{ 1 == 2 } 인 경우 {
  살림.x <- 1.
}
그밖의 경우 {
  살림.x <- 7.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "x"), fixed("7"));
    }

    #[test]
    fn immediate_proof_records_success_diagnostic_without_state_mutation() {
        let source = r#"
검사 밝히기 {
  n 이 자연수 낱낱에 대해 {
    없음.
  }.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(output.diagnostics[0].name, "밝히기 검사");
        assert_eq!(output.diagnostics[0].result, "성공");
        assert!(output.diagnostics[0].error_code.is_none());
        assert!(output.state.get(&Key::new("검사".to_string())).is_none());
    }

    #[test]
    fn assertion_check_restores_outer_state_and_records_success_diag() {
        let source = r#"
검사 <- 세움{
  임시 <- 99.
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
바깥 <- 1.
판정 <- (거리=3)인 검사 살피기.
"#;
        let output = run_source_once(source).expect("run");
        assert!(state_bool(&output, "판정"));
        assert!(output.state.get(&Key::new("거리".to_string())).is_none());
        assert!(output.state.get(&Key::new("임시".to_string())).is_none());
        assert!(output.state.len() >= 2);
        assert_eq!(output.diagnostics.len(), 1);
        assert!(output.diagnostics[0].name.starts_with("살피기 세움{"));
        assert_eq!(output.diagnostics[0].result, "성공");
        assert!(output.diagnostic_failures.is_empty());
    }

    #[test]
    fn immediate_proof_with_failed_assertion_marks_proof_as_failed() {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=-1)) 살피기.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(output.contract_diags.len(), 0);
        assert_eq!(output.diagnostic_failures.len(), 1);
        assert_eq!(output.diagnostic_failures[0].code, "E_PROOF_CHECK_FAILED");
        assert_eq!(output.diagnostics.len(), 2);
        assert!(output.diagnostics[0].name.starts_with("살피기 세움{"));
        assert_eq!(output.diagnostics[0].result, "실패");
        assert_eq!(
            output.diagnostics[0].error_code.as_deref(),
            Some("E_PROOF_CHECK_FAILED")
        );
        assert_eq!(output.diagnostics[1].name, "밝히기 검사");
        assert_eq!(output.diagnostics[1].result, "실패");
        assert_eq!(
            output.diagnostics[1].error_code.as_deref(),
            Some("E_PROOF_CHECK_FAILED")
        );
    }

    #[test]
    fn immediate_proof_collects_solver_runtime_summary() {
        let source = r#"
실행정책 { 열림허용: 풀이. }.

검사 밝히기 {
  ((질의="forall n. n = n", 결과=참)) 열림.풀이.확인.
  ((질의="x * x = 4", 찾음=참, 값="x=2")) 해찾기.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("tokenize");
        let default_root = Parser::default_root_for_source(source);
        let program = Parser::parse_with_default_root(tokens, default_root).expect("parse");
        let mut log_path = std::env::temp_dir();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        log_path.push(format!("teul_solver_runtime_{nonce}.jsonl"));
        let open = OpenRuntime::new(
            crate::runtime::open::OpenMode::Record,
            Some(log_path.clone()),
            vec!["solver".to_string()],
            None,
        )
        .expect("open");
        let evaluator = Evaluator::with_state_seed_open(
            State::new(),
            42,
            open,
            "proof_runtime.ddn".to_string(),
            Some(source.to_string()),
        );
        let output = evaluator.run_with_ticks(&program, 1).expect("run");
        let _ = std::fs::remove_file(log_path);
        assert_eq!(output.proof_runtime.len(), 3);
        match &output.proof_runtime[0] {
            ProofRuntimeEvent::SolverCheck {
                query, satisfied, ..
            } => {
                assert_eq!(query, "forall n. n = n");
                assert_eq!(*satisfied, Some(true));
            }
            other => panic!("unexpected event: {other:?}"),
        }
        match &output.proof_runtime[1] {
            ProofRuntimeEvent::SolverSearch {
                operation,
                query,
                found,
                value,
                ..
            } => {
                assert_eq!(operation, "solve");
                assert_eq!(query, "x * x = 4");
                assert_eq!(*found, Some(true));
                assert_eq!(value.as_deref(), Some("x=2"));
            }
            other => panic!("unexpected event: {other:?}"),
        }
        match &output.proof_runtime[2] {
            ProofRuntimeEvent::ProofBlock {
                name,
                result,
                error_code,
                ..
            } => {
                assert_eq!(name, "검사");
                assert_eq!(result, "성공");
                assert!(error_code.is_none());
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn immediate_proof_failed_run_still_records_block_failure_runtime() {
        let source = r#"
거리_0이상 <- 세움{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
검사 밝히기 {
  (세움=거리_0이상, 값들=(거리=-1)) 살피기.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(output.proof_runtime.len(), 2);
        match &output.proof_runtime[0] {
            ProofRuntimeEvent::ProofCheck { passed, .. } => assert!(!passed),
            other => panic!("unexpected event: {other:?}"),
        }
        match &output.proof_runtime[1] {
            ProofRuntimeEvent::ProofBlock {
                name,
                result,
                error_code,
                ..
            } => {
                assert_eq!(name, "검사");
                assert_eq!(result, "실패");
                assert_eq!(error_code.as_deref(), Some("E_PROOF_CHECK_FAILED"));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn temperature_literals_normalize_to_kelvin_dimension() {
        let source = r#"
섭씨 <- 25@C.
화씨 <- 77@F.
켈빈 <- 298.15@K.
"#;
        let output = run_source_once(source).expect("run");
        let celsius = state_quantity(&output, "섭씨");
        let fahrenheit = state_quantity(&output, "화씨");
        let kelvin = state_quantity(&output, "켈빈");

        assert!(celsius.display().ends_with("@K"));
        assert!(fahrenheit.display().ends_with("@K"));
        assert!(kelvin.display().ends_with("@K"));
        assert_eq!(celsius.dim, kelvin.dim);
        assert_eq!(fahrenheit.dim, kelvin.dim);
        assert_fixed_close(celsius.raw, fixed("298.15"), 1);
        assert_fixed_close(fahrenheit.raw, fixed("298.15"), 1);
        assert_fixed_close(kelvin.raw, fixed("298.15"), 1);
    }

    #[test]
    fn temperature_literals_compare_and_subtract_after_kelvin_normalization() {
        let source = r#"
같음 <- 25@C == 77@F.
차이 <- 30@C - 20@C.
"#;
        let output = run_source_once(source).expect("run");
        assert!(state_bool(&output, "같음"));
        let diff = state_quantity(&output, "차이");
        assert_eq!(diff.display(), "10@K");
        assert_fixed_close(diff.raw, fixed("10"), 1);
    }

    #[test]
    fn template_format_can_render_temperature_in_celsius_and_fahrenheit() {
        let source = r#"
섭씨텍스트 <- (t=298.15@K) 글무늬{"섭씨={t|@.1C}"}.
화씨텍스트 <- (t=298.15@K) 글무늬{"화씨={t|@.1F}"}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "섭씨텍스트"), "섭씨=25.0@C");
        assert_eq!(state_str(&output, "화씨텍스트"), "화씨=77.0@F");
    }

    #[test]
    fn template_format_defaults_temperature_display_to_kelvin() {
        let source = r#"
기본텍스트 <- (t=25@C) 글무늬{"온도={t}"}.
"#;
        let output = run_source_once(source).expect("run");
        let text = state_str(&output, "기본텍스트");
        assert!(text.starts_with("온도=298.149999"));
        assert!(text.ends_with("@K"));
    }

    #[test]
    fn temperature_literals_freezing_point_equivalence_across_units() {
        let source = r#"
동결동치 <- 0@C == 32@F.
켈빈동치 <- 0@C == 273.15@K.
단위혼합차이 <- 32@F - 0@C.
"#;
        let output = run_source_once(source).expect("run");
        assert!(state_bool(&output, "동결동치"));
        assert!(state_bool(&output, "켈빈동치"));
        let diff = state_quantity(&output, "단위혼합차이");
        assert_eq!(diff.display(), "0@K");
        assert_fixed_close(diff.raw, fixed("0"), 1);
    }

    #[test]
    fn regex_pattern_invalid_returns_dedicated_error() {
        let source = r#"
패턴 <- 정규식{"[0-9+"}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_REGEX_PATTERN_INVALID");
    }

    #[test]
    fn regex_replacement_invalid_returns_dedicated_error() {
        let source = r#"
바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "${missing}") 정규바꾸기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_REGEX_REPLACEMENT_INVALID");
    }

    #[test]
    fn regex_replacement_numeric_backref_is_greedy_and_invalid_when_missing() {
        let source = r#"
바꿈 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}, "$10") 정규바꾸기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_REGEX_REPLACEMENT_INVALID");
    }

    #[test]
    fn regex_replacement_invalid_subcases_return_dedicated_error() {
        let cases = [
            r#"
바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "${}") 정규바꾸기.
"#,
            r#"
바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "$") 정규바꾸기.
"#,
            r#"
바꿈 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}, "$999999999999999999999999") 정규바꾸기.
"#,
        ];
        for source in cases {
            let err = match run_source_once(source) {
                Ok(_) => panic!("must fail"),
                Err(err) => err,
            };
            assert_eq!(err.code(), "E_REGEX_REPLACEMENT_INVALID");
        }
    }

    #[test]
    fn quantile_linear_and_nearest_rank_modes_are_deterministic() {
        let source = r#"
값목록 <- [20, 25, 30, 35, 40].
선형 <- (값목록, 0.9, "선형보간") 분위수.
최근 <- (값목록, 0.9, "최근순위") 분위수.
"#;
        let output = run_source_once(source).expect("run");
        assert_fixed_close(state_num(&output, "선형"), fixed("37.9999999981"), 2);
        assert_eq!(state_num(&output, "최근"), fixed("40"));
    }

    #[test]
    fn quantile_mode_invalid_returns_math_domain() {
        let source = r#"
값목록 <- [1, 2, 3].
분위 <- (값목록, 0.5, "linear") 분위수.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_MATH_DOMAIN");
    }

    #[test]
    fn gini_zero_sum_and_negative_guard() {
        let ok_source = r#"
값목록 <- [0, 0, 0, 0].
지니값 <- (값목록) 지니.
"#;
        let ok = run_source_once(ok_source).expect("run zero sum");
        assert_eq!(state_num(&ok, "지니값"), fixed("0"));

        let bad_source = r#"
값목록 <- [1, -2, 3].
지니값 <- (값목록) 지니.
"#;
        let err = match run_source_once(bad_source) {
            Ok(_) => panic!("negative must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_MATH_DOMAIN");
    }

    #[test]
    fn stdlib_l1_integrator_and_interpolation_builtins() {
        let source = r#"
값1 <- (1, 2, 0.5) 적분.오일러.
쌍값 <- (1, 0, -1, 0.25) 적분.반암시적오일러.
x2 <- (쌍값, 0) 차림.값.
v2 <- (쌍값, 1) 차림.값.
보간1 <- (0, 10, 0.25) 보간.선형.
보간2 <- (0, 10, 0.25, 0.5) 보간.계단.
보간3 <- (0, 10, 0.75, 0.5) 보간.계단.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_num(&out, "값1"), fixed("2"));
        assert_eq!(state_num(&out, "x2"), fixed("0.9375"));
        assert_eq!(state_num(&out, "v2"), fixed("-0.25"));
        assert_eq!(state_num(&out, "보간1"), fixed("2.5"));
        assert_eq!(state_num(&out, "보간2"), fixed("0"));
        assert_eq!(state_num(&out, "보간3"), fixed("10"));
    }

    #[test]
    fn stdlib_l1_filter_builtins() {
        let source = r#"
창0 <- () 차림.
결과1 <- (창0, 1) 필터.이동평균.
창1 <- (결과1, 0) 차림.값.
평균1 <- (결과1, 1) 차림.값.
결과2 <- (창1, 2) 필터.이동평균.
창2 <- (결과2, 0) 차림.값.
평균2 <- (결과2, 1) 차림.값.
결과3 <- (창2, 3) 필터.이동평균.
창3 <- (결과3, 0) 차림.값.
평균3 <- (결과3, 1) 차림.값.
결과4 <- (창3, 4) 필터.이동평균.
평균4 <- (결과4, 1) 차림.값.
y1 <- (0, 1, 0.5) 필터.지수평활.
y2 <- (y1, 1, 0.5) 필터.지수평활.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_num(&out, "평균1"), fixed("1"));
        assert_eq!(state_num(&out, "평균2"), fixed("1.5"));
        assert_eq!(state_num(&out, "평균3"), fixed("2"));
        assert_eq!(state_num(&out, "평균4"), fixed("2.5"));
        assert_eq!(state_num(&out, "y1"), fixed("0.5"));
        assert_eq!(state_num(&out, "y2"), fixed("0.75"));
    }

    #[test]
    fn seed_tilde_alias_dispatches_to_same_body() {
        let source = r#"
막~막으:움직씨 = {
살림.카운트 <- 살림.카운트 + 1.
}.

살림.카운트 <- 0.
() 막.
첫 <- 살림.카운트.
() 막으.
둘 <- 살림.카운트.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "첫"), fixed("1"));
        assert_eq!(state_num(&output, "둘"), fixed("2"));
    }

    #[test]
    fn signal_send_dispatches_typed_and_generic_receive_hooks() {
        let source = r#"
(온도:수, 풍속:수) 기상특보:알림씨 = {
}.

관제탑:임자 = {
  제.경보상태 <- 거짓.
  제.최근보낸이 <- "".
  제.최근온도 <- 0.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    제.경보상태 <- 참.
  }.

  (알림 알림.이름 == "기상특보")인 알림을 받으면 {
    제.최근보낸이 <- 알림.보낸이.
    제.최근온도 <- 알림.정보.온도.
  }.
}.

(시작)할때 {
  (기상청)의 (온도:41, 풍속:12@m/s) 기상특보 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert!(state_bool(&output, "관제탑.경보상태"));
        assert_eq!(state_str(&output, "관제탑.최근보낸이"), "기상청");
        assert_eq!(state_num(&output, "관제탑.최근온도"), fixed("41"));
    }

    #[test]
    fn signal_send_dispatch_order_is_typed_conditional_then_typed_then_generic_conditional_then_generic(
    ) {
        let source = r#"
(온도:수, 풍속:수) 기상특보:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  기상특보를 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
  }.

  (정보 정보.온도 > 40)인 기상특보를 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  (알림 알림.이름 == "기상특보")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.
}.

(시작)할때 {
  (기상청)의 (온도:41, 풍속:12@m/s) 기상특보 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "관제탑.순서"), fixed("2143"));
    }

    #[test]
    fn signal_send_dispatch_same_rank_preserves_declaration_order() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  (정보 정보.값 > 0)인 첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
  }.

  (정보 정보.값 < 10)인 첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.

  알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 5.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "관제탑.순서"), fixed("12345"));
    }

    #[test]
    fn signal_send_without_sender_uses_nuri_outside_imja() {
        let source = r#"
(온도:수) 기상특보:알림씨 = {
}.

관제탑:임자 = {
  제.최근보낸이 <- "".
  (알림 참)인 알림을 받으면 {
    제.최근보낸이 <- 알림.보낸이.
  }.
}.

(시작)할때 {
  (온도:25) 기상특보 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "관제탑.최근보낸이"), "누리");
    }

    #[test]
    fn signal_send_without_sender_uses_current_imja_inside_imja() {
        let source = r#"
(온도:수) 기상특보:알림씨 = {
}.

관제탑:임자 = {
  제.최근보낸이 <- "".
  (알림 참)인 알림을 받으면 {
    제.최근보낸이 <- 알림.보낸이.
  }.
}.

기상청:임자 = {
  (온도:25) 기상특보 ~~> 관제탑.
}.

(시작)할때 {
  () 기상청.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "관제탑.최근보낸이"), "기상청");
    }

    #[test]
    fn signal_send_inside_receive_hook_is_processed_in_next_reactive_pass() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    (값:0) 둘알림 ~~> 제.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "관제탑.순서"), fixed("123"));
    }

    #[test]
    fn signal_send_from_hook_does_not_reenter_current_dispatch_and_preserves_fifo() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    (값:0) 둘알림 ~~> 제.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "관제탑.순서"), fixed("12434"));
    }

    #[test]
    fn signal_send_inside_hook_without_sender_uses_current_imja_name() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

관제탑:임자 = {
  제.최근보낸이 <- "".

  첫알림을 받으면 {
    (값:0) 둘알림 ~~> 제.
  }.

  (알림 알림.이름 == "둘알림")인 알림을 받으면 {
    제.최근보낸이 <- 알림.보낸이.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "관제탑.최근보낸이"), "관제탑");
    }

    #[test]
    fn signal_send_inside_hook_to_non_imja_receiver_fails_in_queued_dispatch() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

수신기:움직씨 = {
}.

관제탑:임자 = {
  첫알림을 받으면 {
    (값:0) 둘알림 ~~> 수신기.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_TYPE_MISMATCH");
    }

    #[test]
    fn signal_send_multiple_enqueued_events_preserve_fifo_order() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

(값:수) 셋알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    (값:0) 둘알림 ~~> 제.
    (값:0) 셋알림 ~~> 제.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  셋알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "관제탑.순서"), fixed("1234"));
    }

    #[test]
    fn signal_send_nested_enqueues_follow_breadth_first_fifo_order() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

(값:수) 셋알림:알림씨 = {
}.

(값:수) 넷알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    (값:0) 둘알림 ~~> 제.
    (값:0) 셋알림 ~~> 제.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
    (값:0) 넷알림 ~~> 제.
  }.

  셋알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.

  넷알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 5.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "관제탑.순서"), fixed("12345"));
    }

    #[test]
    fn signal_send_same_kind_enqueued_inside_hook_does_not_reenter_current_event() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  (정보 정보.값 == 1)인 첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    (값:0) 첫알림 ~~> 제.
  }.

  (정보 정보.값 == 0)인 첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "관제탑.순서"), fixed("1323"));
    }

    #[test]
    fn signal_send_dispatch_error_clears_remaining_pending_queue() {
        let source = r#"
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

(값:수) 셋알림:알림씨 = {
}.

(값:수) 넷알림:알림씨 = {
}.

수신기:움직씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    (값:0) 둘알림 ~~> 제.
    (값:0) 셋알림 ~~> 수신기.
    (값:0) 넷알림 ~~> 제.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  넷알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"#;
        let tokens = Lexer::tokenize(source).expect("lex");
        let default_root = Parser::default_root_for_source(source);
        let program = Parser::parse_with_default_root(tokens, default_root).expect("parse");
        let evaluator = Evaluator::with_state_and_seed(State::new(), 42);
        let failure = match evaluator.run_with_ticks_capture_failure(&program, 1) {
            Ok(_) => panic!("must fail"),
            Err(failure) => failure,
        };
        assert_eq!(failure.error.code(), "E_RUNTIME_TYPE_MISMATCH");
        assert_eq!(state_num(&failure.output, "관제탑.순서"), fixed("1"));
    }

    #[test]
    fn signal_send_dispatch_error_clears_pending_queue_tail() {
        let mut evaluator = Evaluator::with_state_and_seed(State::new(), 42);
        let span = crate::lang::span::Span::new(1, 1, 1, 1);
        evaluator.user_seeds.insert(
            "관제탑".to_string(),
            UserSeed {
                kind: SeedKind::Named("임자".to_string()),
                params: Vec::new(),
                body: Vec::new(),
            },
        );
        evaluator.user_seeds.insert(
            "수신기".to_string(),
            UserSeed {
                kind: SeedKind::Named("움직씨".to_string()),
                params: Vec::new(),
                body: Vec::new(),
            },
        );
        evaluator.pending_signals.push_back(PendingSignal {
            receiver_name: "관제탑".to_string(),
            event_kind: "첫알림".to_string(),
            sender_name: "누리".to_string(),
            event_payload: PackValue {
                fields: BTreeMap::new(),
            },
            span,
        });
        evaluator.pending_signals.push_back(PendingSignal {
            receiver_name: "수신기".to_string(),
            event_kind: "둘알림".to_string(),
            sender_name: "누리".to_string(),
            event_payload: PackValue {
                fields: BTreeMap::new(),
            },
            span,
        });
        evaluator.pending_signals.push_back(PendingSignal {
            receiver_name: "관제탑".to_string(),
            event_kind: "셋알림".to_string(),
            sender_name: "누리".to_string(),
            event_payload: PackValue {
                fields: BTreeMap::new(),
            },
            span,
        });
        let err = evaluator.drain_signal_queue().expect_err("must fail");
        assert_eq!(err.code(), "E_RUNTIME_TYPE_MISMATCH");
        assert!(evaluator.pending_signals.is_empty());
    }

    #[test]
    fn je_path_outside_imja_fails_with_dedicated_error() {
        let source = r#"
(시작)할때 {
  값 <- 제.경보상태.
}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_SELF_OUTSIDE_IMJA");
    }

    #[test]
    fn je_sender_outside_imja_fails_with_dedicated_error() {
        let source = r#"
(온도:수) 기상특보:알림씨 = {
}.

관제탑:임자 = {
  알림을 받으면 {
  }.
}.

(시작)할때 {
  (제)의 (온도:25) 기상특보 ~~> 관제탑.
}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_SELF_OUTSIDE_IMJA");
    }

    #[test]
    fn numeric_family_constructors_are_callable() {
        let source = r#"
큰 <- ("9007199254740993") 큰바른수.
나눔 <- (6, 9) 나눔수.
곱 <- (72) 곱수.
"#;
        let output = run_source_once(source).expect("run");
        let big = output
            .state
            .get(&Key::new("큰".to_string()))
            .expect("big state key");
        let rational = output
            .state
            .get(&Key::new("나눔".to_string()))
            .expect("rational state key");
        let factor = output
            .state
            .get(&Key::new("곱".to_string()))
            .expect("factor state key");
        assert_eq!(exact_numeric_kind(big), Some(NUMERIC_KIND_BIG_INT));
        assert_eq!(exact_numeric_kind(rational), Some(NUMERIC_KIND_RATIONAL));
        assert_eq!(exact_numeric_kind(factor), Some(NUMERIC_KIND_FACTOR));
    }

    #[test]
    fn decl_block_initializer_typecheck_rejects_exact_numeric_kind_mismatch() {
        let source = r#"
채비 {
  값:나눔수 <- (12) 곱수.
}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_TYPE_MISMATCH");
    }

    #[test]
    fn decl_block_initializer_typecheck_accepts_exact_numeric_kind_match() {
        let source = r#"
채비 {
  값:나눔수 <- (6, 9) 나눔수.
}.
"#;
        let output = run_source_once(source).expect("run");
        let value = output
            .state
            .get(&Key::new("값".to_string()))
            .expect("state key");
        match exact_numeric_kind(value) {
            Some(kind) => assert_eq!(kind, NUMERIC_KIND_RATIONAL),
            None => panic!("expected exact numeric pack, got {}", value.display()),
        }
    }

    #[test]
    fn decl_block_initializer_typecheck_accepts_boolean_list_map_pack_aliases() {
        let source = r#"
채비 {
  논값:boolean <- 참.
  목록값:list <- [1, 2].
  짝값:map <- ("x", 1) 짝맞춤.
  묶음값:pack <- (이름="또니").
}.
"#;
        run_source_once(source).expect("run");
    }

    #[test]
    fn decl_block_initializer_typecheck_rejects_boolean_alias_mismatch_with_canonical_expected() {
        let source = r#"
채비 {
  논값:boolean <- 1.
}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        match err {
            RuntimeError::TypeMismatchDetail { expected, .. } => {
                assert_eq!(expected, "참거짓");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn decl_block_initializer_typecheck_rejects_pack_alias_mismatch_with_canonical_expected() {
        let source = r#"
채비 {
  묶음값:pack <- 1.
}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        match err {
            RuntimeError::TypeMismatchDetail { expected, .. } => {
                assert_eq!(expected, "묶음");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn decl_block_initializer_typecheck_accepts_korean_aliases_and_string_none_aliases() {
        let source = r#"
채비 {
  글값:string <- "또니".
  없음값:none <- 없음.
  비움값:non <- 없음.
  목록값:목록 <- [1, 2].
  모둠값:모둠 <- (1, 2, 2) 모음.
  그림표값:그림표 <- ("x", 1) 짝맞춤.
  꾸러미값:값꾸러미 <- (이름="또니").
}.
"#;
        run_source_once(source).expect("run");
    }

    #[test]
    fn decl_block_initializer_typecheck_rejects_korean_alias_mismatch_with_canonical_expected() {
        let source = r#"
채비 {
  목록값:목록 <- 1.
}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        match err {
            RuntimeError::TypeMismatchDetail { expected, .. } => {
                assert_eq!(expected, "차림");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn decl_block_initializer_typecheck_rejects_non_alias_mismatch_with_canonical_expected() {
        let source = r#"
채비 {
  비움값:non <- 1.
}.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        match err {
            RuntimeError::TypeMismatchDetail { expected, .. } => {
                assert_eq!(expected, "없음");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn numeric_calculus_v1_builtins_return_value_error_method() {
        let source = r#"
f1 <- (#ascii) 수식{x^3}.
d1 <- (f1, "x", 1, 0.5) 미분.중앙차분.
d1값 <- (d1, 0) 차림.값.
d1오차 <- (d1, 1) 차림.값.
d1방법 <- (d1, 2) 차림.값.

f2 <- (#ascii) 수식{x^2}.
i1 <- (f2, "x", 0, 1, 0.5) 적분.사다리꼴.
i1값 <- (i1, 0) 차림.값.
i1오차 <- (i1, 1) 차림.값.
i1방법 <- (i1, 2) 차림.값.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_num(&out, "d1값"), fixed("3"));
        assert_eq!(state_num(&out, "d1오차"), fixed("0.25"));
        assert_eq!(state_str(&out, "d1방법"), "중앙차분");
        assert_eq!(state_num(&out, "i1값"), Fixed64::from_raw(1_431_655_766));
        assert_eq!(state_num(&out, "i1오차"), Fixed64::from_raw(44_739_242));
        assert_eq!(state_str(&out, "i1방법"), "사다리꼴");
    }

    #[test]
    fn numeric_calculus_v1_rejects_zero_step() {
        let source = r#"
f <- (#ascii) 수식{x^2}.
d <- (f, "x", 1, 0) 미분.중앙차분.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_MATH_DOMAIN");
    }

    #[test]
    fn logic_builtins_are_callable() {
        let source = r##"
원본 <- [참, 거짓].
거짓만 <- (원본, "#아님") 거르기.
아님개수 <- (거짓만) 길이.
그리고값 <- ([참, 거짓], 참, "#그리고") 합치기.
또는값 <- ([참, 거짓], 거짓, "#또는") 합치기.
"##;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "아님개수"), fixed("1"));
        assert!(!state_bool(&output, "그리고값"));
        assert!(state_bool(&output, "또는값"));
    }

    #[test]
    fn template_fill_builtin_call_works() {
        let source = r##"
주입들 <- [(name="또니", score=7)].
출력 <- (주입들, 글무늬{"이름:{name} 점수:{score}"}, "#채우기") 합치기.
"##;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "출력"), "이름:또니 점수:7");
    }

    #[test]
    fn list_dot_index_path_access_works() {
        let span = crate::lang::span::Span::new(1, 1, 1, 1);
        let evaluator = Evaluator::with_state(State::new());
        let list = Value::List(ListValue {
            items: vec![
                Value::Num(Quantity::new(fixed("11"), UnitDim::zero())),
                Value::Num(Quantity::new(fixed("22"), UnitDim::zero())),
                Value::Num(Quantity::new(fixed("33"), UnitDim::zero())),
            ],
        });

        let value = evaluator
            .eval_member_access(list, "1", span)
            .expect("list index access");
        match value {
            Value::Num(quantity) => assert_eq!(quantity.raw, fixed("22")),
            other => panic!("expected number, got {}", other.display()),
        }
    }

    #[test]
    fn list_dot_index_requires_numeric_segment() {
        let span = crate::lang::span::Span::new(1, 1, 1, 1);
        let evaluator = Evaluator::with_state(State::new());
        let list = Value::List(ListValue {
            items: vec![
                Value::Num(Quantity::new(fixed("10"), UnitDim::zero())),
                Value::Num(Quantity::new(fixed("20"), UnitDim::zero())),
            ],
        });

        let err = match evaluator.eval_member_access(list, "첫째", span) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_PACK");
    }

    #[test]
    fn map_dot_nested_write_path_access_works() {
        let source = r#"
살림.공 <- ("속도", ("x", 0) 짝맞춤) 짝맞춤.
살림.공.속도.x <- 9.
값 <- 살림.공.속도.x.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "값"), fixed("9"));
    }

    #[test]
    fn map_dot_nested_write_missing_parent_key_fails_map_dot_key_missing() {
        let source = r#"
살림.공 <- () 짝맞춤.
살림.공.속도.x <- 9.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_MAP_DOT_KEY_MISSING");
    }

    #[test]
    fn map_dot_read_missing_key_fails_map_dot_key_missing() {
        let source = r#"
살림.공 <- () 짝맞춤.
값 <- 살림.공.속도.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_MAP_DOT_KEY_MISSING");
    }

    #[test]
    fn map_optional_lookup_returns_none_when_missing() {
        let source = r#"
살림.공 <- ("속도", 9) 짝맞춤.
있음 <- (살림.공, "속도") 찾기?.
없음값 <- (살림.공, "가속도") 찾기?.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "있음"), fixed("9"));
        let value = output
            .state
            .get(&Key::new("없음값".to_string()))
            .expect("state key");
        assert!(matches!(value, Value::None));
    }

    #[test]
    fn module_alias_call_resolves_to_builtin_function() {
        let source = r#"
쓰임 {
  수학: "표준/수학".
}.
길 <- ("abc") 수학.길이.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "길"), fixed("3"));
    }

    #[test]
    fn contract_pre_abort_restores_state_before_else_body() {
        let source = r#"
x <- 5.
{ x > 10 }인것 바탕으로(물림) 아니면 {
  x <- (x + 1).
}.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "x"), fixed("5"));
        assert!(output.state.get(&Key::new("끝".to_string())).is_none());
        assert_eq!(output.contract_diags.len(), 1);
        assert!(matches!(output.contract_diags[0].kind, ContractKind::Pre));
        assert!(matches!(output.contract_diags[0].mode, ContractMode::Abort));
    }

    #[test]
    fn contract_post_abort_restores_state_before_repair_body() {
        let source = r#"
y <- 5.
{ y < 5 }인것 다짐하고(물림) 아니면 {
  y <- (y + 1).
}.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "y"), fixed("5"));
        assert!(output.state.get(&Key::new("끝".to_string())).is_none());
        assert_eq!(output.contract_diags.len(), 1);
        assert!(matches!(output.contract_diags[0].kind, ContractKind::Post));
        assert!(matches!(output.contract_diags[0].mode, ContractMode::Abort));
    }

    #[test]
    fn contract_alert_keeps_else_body_state_changes() {
        let source = r#"
z <- 5.
{ z > 10 }인것 바탕으로(알림) 아니면 {
  z <- (z + 1).
}.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "z"), fixed("6"));
        assert_eq!(state_num(&output, "끝"), fixed("1"));
        assert_eq!(output.contract_diags.len(), 1);
        assert!(matches!(output.contract_diags[0].kind, ContractKind::Pre));
        assert!(matches!(output.contract_diags[0].mode, ContractMode::Alert));
    }

    #[test]
    fn nested_contract_abort_restores_enclosing_block_state() {
        let source = r#"
w <- 5.
{ 참 }인것 바탕으로(물림) 아니면 {
  w <- 99.
} 맞으면 {
  w <- (w + 1).
  { w > 10 }인것 바탕으로(물림) 아니면 {
    w <- (w + 10).
  }.
}.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "w"), fixed("6"));
        assert!(output.state.get(&Key::new("끝".to_string())).is_none());
        assert_eq!(output.contract_diags.len(), 1);
        assert!(matches!(output.contract_diags[0].kind, ContractKind::Pre));
        assert!(matches!(output.contract_diags[0].mode, ContractMode::Abort));
    }

    #[test]
    fn contract_frame_rollback_restores_pending_signal_queue_len() {
        let mut evaluator = Evaluator::with_state_and_seed(State::new(), 42);
        let span = crate::lang::span::Span::new(1, 1, 1, 1);
        evaluator.pending_signals.push_back(PendingSignal {
            receiver_name: "관제탑".to_string(),
            event_kind: "첫알림".to_string(),
            sender_name: "누리".to_string(),
            event_payload: PackValue {
                fields: BTreeMap::new(),
            },
            span,
        });
        let frame = evaluator.begin_contract_frame(span).expect("frame");
        evaluator.pending_signals.push_back(PendingSignal {
            receiver_name: "관제탑".to_string(),
            event_kind: "둘알림".to_string(),
            sender_name: "누리".to_string(),
            event_payload: PackValue {
                fields: BTreeMap::new(),
            },
            span,
        });
        evaluator
            .rollback_contract_frame(frame.frame_id, span)
            .expect("rollback");
        assert_eq!(evaluator.pending_signals.len(), 1);
        let front = evaluator.pending_signals.front().expect("front");
        assert_eq!(front.event_kind, "첫알림");
    }

    #[test]
    fn contract_frame_rollback_truncates_open_record_file() {
        let span = crate::lang::span::Span::new(1, 1, 1, 1);
        let mut log_path = std::env::temp_dir();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        log_path.push(format!("teul_contract_abort_open_{nonce}.jsonl"));
        let open = OpenRuntime::new(
            crate::runtime::open::OpenMode::Record,
            Some(log_path.clone()),
            vec!["clock".to_string()],
            None,
        )
        .expect("open");
        let mut evaluator = Evaluator::with_state_seed_open(
            State::new(),
            42,
            open,
            "contract_abort_open.ddn".to_string(),
            None,
        );
        let frame = evaluator.begin_contract_frame(span).expect("frame");
        evaluator
            .open
            .open_clock("contract_abort_open_site", span)
            .expect("clock");
        let written_len = std::fs::metadata(&log_path).expect("metadata").len();
        assert!(written_len > 0);
        evaluator
            .rollback_contract_frame(frame.frame_id, span)
            .expect("rollback");
        let rolled_back_len = std::fs::metadata(&log_path).expect("metadata").len();
        let _ = std::fs::remove_file(&log_path);
        assert_eq!(rolled_back_len, 0);
    }

    #[test]
    fn contract_frame_rollback_restores_open_replay_queue() {
        let span = crate::lang::span::Span::new(1, 1, 1, 1);
        let mut log_path = std::env::temp_dir();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        log_path.push(format!("teul_contract_abort_open_replay_{nonce}.jsonl"));
        let mut record = OpenRuntime::new(
            OpenMode::Record,
            Some(log_path.clone()),
            vec!["input".to_string()],
            None,
        )
        .expect("record");
        let first = OpenInputFrame::new(0b0001, 0b0001, 0);
        let second = OpenInputFrame::new(0b0010, 0b0010, 0b0001);
        record
            .open_input("contract_abort_input_site", 0, Some(first), span)
            .expect("record first");
        record
            .open_input("contract_abort_input_site", 0, Some(second), span)
            .expect("record second");
        drop(record);

        let replay = OpenRuntime::new(
            OpenMode::Replay,
            Some(log_path.clone()),
            vec!["input".to_string()],
            None,
        )
        .expect("replay");
        let mut evaluator = Evaluator::with_state_seed_open(
            State::new(),
            42,
            replay,
            "contract_abort_input.ddn".to_string(),
            None,
        );
        let frame = evaluator.begin_contract_frame(span).expect("frame");
        let consumed = evaluator
            .open
            .open_input("contract_abort_input_site", 0, None, span)
            .expect("consume first");
        assert_eq!(consumed, first);
        evaluator
            .rollback_contract_frame(frame.frame_id, span)
            .expect("rollback");
        let replayed_first = evaluator
            .open
            .open_input("contract_abort_input_site", 0, None, span)
            .expect("replay first");
        let replayed_second = evaluator
            .open
            .open_input("contract_abort_input_site", 0, None, span)
            .expect("replay second");
        let _ = std::fs::remove_file(&log_path);
        assert_eq!(replayed_first, first);
        assert_eq!(replayed_second, second);
    }

    #[test]
    fn diagnostic_failure_and_bogae_request_do_not_change_state_hash() {
        let plain = run_source_once(
            r#"
값 <- 1.
"#,
        )
        .expect("plain run");
        let with_diag_and_bogae = run_source_once(
            r#"
값 <- 1.
#진단(값 = 2, 이름="diag_only")
보개로 그려.
"#,
        )
        .expect("diag+bogae run");
        assert_eq!(
            crate::core::hash::state_hash(&plain.state),
            crate::core::hash::state_hash(&with_diag_and_bogae.state)
        );
        assert_eq!(
            state_num(&plain, "값"),
            state_num(&with_diag_and_bogae, "값")
        );
        assert_eq!(with_diag_and_bogae.diagnostics.len(), 1);
        assert_eq!(with_diag_and_bogae.diagnostic_failures.len(), 1);
        assert!(with_diag_and_bogae.bogae_requested);
    }

    #[test]
    fn beat_block_defers_reserved_assignments_until_commit() {
        let source = r#"
x <- 1.
덩이 {
  x 보여주기.
  x <- 10 미루기.
  x 보여주기.
  y <- (x + 1) 미루기.
}.
x 보여주기.
y 보여주기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "x"), fixed("10"));
        assert_eq!(state_num(&output, "y"), fixed("2"));
        assert_eq!(
            trace_logs(&output),
            vec![
                "1".to_string(),
                "1".to_string(),
                "10".to_string(),
                "2".to_string(),
            ]
        );
    }

    #[test]
    fn legacy_beat_keyword_is_rejected() {
        let source = r#"
x <- 1.
박자 {
  x 보여주기.
  x <- 10 미루기.
  x 보여주기.
  y <- (x + 1) 미루기.
}.
x 보여주기.
y 보여주기.
"#;
        let tokens = Lexer::tokenize(source).expect("lex");
        let default_root = Parser::default_root_for_source(source);
        let err = Parser::parse_with_default_root(tokens, default_root).expect_err("must fail");
        assert_eq!(err.code(), "E_PARSE_UNEXPECTED_TOKEN");
    }

    #[test]
    fn beat_block_contract_abort_rolls_back_whole_block() {
        let source = r#"
x <- 1.
덩이 {
  x <- 10 미루기.
  y <- 99.
  { 거짓 }인것 바탕으로(물림) 아니면 {
  }.
}.
끝 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "x"), fixed("1"));
        assert!(output.state.get(&Key::new("y".to_string())).is_none());
        assert!(output.state.get(&Key::new("끝".to_string())).is_none());
        assert_eq!(output.contract_diags.len(), 1);
        assert!(matches!(output.contract_diags[0].mode, ContractMode::Abort));
    }

    #[test]
    fn beat_block_contract_abort_keeps_pre_block_state_hash() {
        let baseline = run_source_once(
            r#"
x <- 1.
"#,
        )
        .expect("baseline");
        let aborted = run_source_once(
            r#"
x <- 1.
덩이 {
  x <- 10 미루기.
  y <- 99.
  { 거짓 }인것 바탕으로(물림) 아니면 {
  }.
}.
z <- 7.
"#,
        )
        .expect("aborted");
        assert_eq!(
            crate::core::hash::state_hash(&baseline.state),
            crate::core::hash::state_hash(&aborted.state)
        );
        assert_eq!(state_num(&aborted, "x"), fixed("1"));
        assert!(aborted.state.get(&Key::new("y".to_string())).is_none());
        assert!(aborted.state.get(&Key::new("z".to_string())).is_none());
    }
}
