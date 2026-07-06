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
const CALL_TAILS: &[&str] = &["하면서", "면서", "하기", "기", "하고", "고", "하면", "면"];
const BOGAE_SHOW_LINES_TAG: &str = "보개_출력_줄들";
const BOGAE_GRAPH_POINTS_F_TAG: &str = "보개_그래프_점목록_f";
const EXACT_NUMERIC_KIND_FIELD: &str = "__정확수종류";
const EXACT_NUMERIC_BIGINT_FIELD: &str = "값";
const EXACT_NUMERIC_RATIONAL_NUM_FIELD: &str = "분자";
const EXACT_NUMERIC_RATIONAL_DEN_FIELD: &str = "분모";
const EXACT_NUMERIC_FACTOR_VALUE_FIELD: &str = "값";
const NUMERIC_KIND_BIG_INT: &str = "큰바른수";
const NUMERIC_KIND_RATIONAL: &str = "나눔수";
const NUMERIC_KIND_FACTOR: &str = "곱수";
const RELATION_KIND_FIELD: &str = "__관계종류";
const RELATION_KIND_EQUATION: &str = "방정식";
const RELATION_LEFT_FIELD: &str = "왼쪽";
const RELATION_RIGHT_FIELD: &str = "오른쪽";
const RELATION_SOLVE_RESULT_KIND_FIELD: &str = "__풀이결과종류";
const RELATION_SOLVE_RESULT_SUCCESS: &str = "성공";
const RELATION_SOLVE_RESULT_FAILURE: &str = "실패";
const RELATION_SOLVE_VAR_FIELD: &str = "미지수";
const RELATION_SOLVE_VALUE_FIELD: &str = "값";
const RELATION_SOLVE_BINDINGS_FIELD: &str = "해";
const RELATION_SOLVE_REASON_FIELD: &str = "사유";
const STD_GRID_KIND: &str = "표준.격자";
const STD_INPUT_MAP_KIND: &str = "표준.입력사상";
const STD_BLOCK_PIECE_KIND: &str = "std_block_piece";
const STD_BLOCK_PIECE_KIND_FIELD: &str = "__종류";
const STD_BLOCK_PIECE_CELLS_FIELD: &str = "칸들";
const STD_RANDOM_BAG_KIND: &str = "std_random_bag";
const STD_RANDOM_BAG_DRAW_KIND: &str = "std_random_bag_draw";
const STD_RANDOM_BAG_KIND_FIELD: &str = "__종류";
const STD_RANDOM_BAG_SEED_FIELD: &str = "시앗";
const STD_RANDOM_BAG_STATE_FIELD: &str = "상태";
const STD_RANDOM_BAG_ORIGINAL_FIELD: &str = "원본";
const STD_RANDOM_BAG_REMAINING_FIELD: &str = "남은것";
const STD_RANDOM_BAG_DRAWS_FIELD: &str = "뽑은수";
const STD_RANDOM_BAG_VALUE_FIELD: &str = "값";
const STD_RANDOM_BAG_BAG_FIELD: &str = "가방";
const STD_GRID_GAME_STATE_KIND: &str = "std_grid_game_state";
const STD_GRID_GAME_STATE_KIND_FIELD: &str = "__종류";
const STD_GRID_GAME_STATE_STATE_FIELD: &str = "상태";
const STD_GRID_GAME_STATE_PREV_FIELD: &str = "이전상태";
const STD_GRID_GAME_STATE_TICK_FIELD: &str = "틱";
const STD_GRID_LINE_CLEAR_KIND: &str = "std_grid_line_clear";
const STD_FALLING_PIECE_KIND: &str = "std_falling_piece";
const STD_GRID_GAME_SCORE_KIND: &str = "std_grid_game_score";
const STD_GRID_GAME_SESSION_KIND: &str = "std_grid_game_session";
const STD_GRID_GAME_TICK_KIND: &str = "std_grid_game_tick";
const STD_GRID_GAME_NEXT_PIECE_KIND: &str = "std_grid_game_next_piece";
const STD_GRID_GAME_VIEW_SUMMARY_KIND: &str = "std_grid_game_view_summary";
const STD_GRID_GAME_HOLD_KIND: &str = "std_grid_game_hold_queue";
const STD_GRID_GAME_HOLD_SWAP_KIND: &str = "std_grid_game_hold_swap";
const STD_GRID_GAME_ROTATION_TRY_KIND: &str = "std_grid_game_rotation_try";

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
    const_scopes: Vec<BTreeSet<Key>>,
    pending_signals: VecDeque<PendingSignal>,
    processing_signal_queue: bool,
    deferred_assign_frames: Vec<Vec<DeferredAssign>>,
    flow_assigns: Vec<FlowAssignRecord>,
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
    ContinueLoop(crate::lang::span::Span),
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
struct FlowAssignRecord {
    key: Key,
    value: Expr,
    target_span: crate::lang::span::Span,
    span: crate::lang::span::Span,
}

#[derive(Clone)]
struct ContractFrame {
    frame_id: u64,
    persistent_snapshot: State,
    const_scopes: Vec<BTreeSet<Key>>,
    pending_signals_len: usize,
    deferred_assign_frames: Vec<Vec<DeferredAssign>>,
    flow_assigns: Vec<FlowAssignRecord>,
    open_checkpoint: OpenCheckpoint,
    depth: u32,
}

#[derive(Clone)]
struct NuriResetSnapshot {
    state: State,
    const_scopes: Vec<BTreeSet<Key>>,
    pending_signals: VecDeque<PendingSignal>,
    deferred_assign_frames: Vec<Vec<DeferredAssign>>,
    flow_assigns: Vec<FlowAssignRecord>,
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
        FlowControl::ContinueLoop(span) => Err(RuntimeError::ContinueOutsideForeach { span }),
        FlowControl::Break(span) => Err(RuntimeError::BreakOutsideLoop { span }),
        FlowControl::Return(_, span) => Err(RuntimeError::ReturnOutsideSeed { span }),
    }
}

fn is_live_control_key(name: &str) -> bool {
    name.starts_with("샘.") || name.starts_with("입력상태.")
}

fn capture_live_control_values(state: &State) -> Vec<(Key, Value)> {
    state
        .resources
        .iter()
        .filter_map(|(key, value)| {
            is_live_control_key(key.as_str()).then(|| (key.clone(), value.clone()))
        })
        .collect()
}

fn clear_live_control_values(state: &mut State) {
    let keys = state
        .resources
        .keys()
        .filter(|key| is_live_control_key(key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    for key in keys {
        state.resources.remove(&key);
    }
}

fn restore_live_control_values(state: &mut State, entries: &[(Key, Value)]) {
    for (key, value) in entries {
        state.resources.insert(key.clone(), value.clone());
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
            const_scopes: vec![BTreeSet::new()],
            pending_signals: VecDeque::new(),
            processing_signal_queue: false,
            deferred_assign_frames: Vec::new(),
            flow_assigns: Vec::new(),
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
        if let Err(error) = self.apply_flow_fixed_point() {
            return Err(self.into_failure(error));
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
        if let Err(error) = self.apply_flow_fixed_point() {
            return Err(self.into_failure(error));
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
            if let Err(error) = self.apply_flow_fixed_point() {
                return Err(self.into_failure(error));
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
                    let key = self.scoped_decl_key(&item.name);
                    self.state.set(key.clone(), value);
                    if matches!(item.kind, DeclKind::Butbak) {
                        self.declare_const(key);
                    }
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
                if self.is_const(&key) {
                    return Err(RuntimeError::Pack {
                        message: format!("붙박이는 재대입할 수 없습니다: {}", key.as_str()),
                        span: target.span,
                    });
                }
                if *deferred {
                    self.record_deferred_assign(key, val);
                } else {
                    self.state.set(key, val);
                }
                Ok(FlowControl::Continue)
            }
            Stmt::FlowAssign {
                target,
                value,
                span,
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
                let key = self.path_to_key(target)?;
                if self.is_const(&key) {
                    return Err(RuntimeError::Pack {
                        message: format!("붙박이는 재대입할 수 없습니다: {}", key.as_str()),
                        span: target.span,
                    });
                }
                self.flow_assigns.push(FlowAssignRecord {
                    key,
                    value: value.clone(),
                    target_span: target.span,
                    span: *span,
                });
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
            Stmt::Boim { entries, .. } => {
                self.eval_boim(entries)?;
                Ok(FlowControl::Continue)
            }
            Stmt::Hook { .. }
            | Stmt::HookWhenBecomes { .. }
            | Stmt::HookWhile { .. }
            | Stmt::LifecycleBlock { .. } => Ok(FlowControl::Continue),
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
                        FlowControl::ContinueLoop(span) => {
                            return Err(RuntimeError::ContinueOutsideForeach { span });
                        }
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
                        FlowControl::ContinueLoop(span) => {
                            return Err(RuntimeError::ContinueOutsideForeach { span });
                        }
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
                        path: item.clone(),
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
                        FlowControl::ContinueLoop(_) => continue,
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
            Stmt::ContinueLoop { span } => Ok(FlowControl::ContinueLoop(*span)),
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
                                    FlowControl::ContinueLoop(span) => {
                                        self.commit_contract_frame(frame.frame_id);
                                        return Ok(FlowControl::ContinueLoop(span));
                                    }
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
                                FlowControl::ContinueLoop(span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::ContinueLoop(span));
                                }
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
                                FlowControl::ContinueLoop(span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::ContinueLoop(span));
                                }
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
                                FlowControl::ContinueLoop(span) => {
                                    self.commit_contract_frame(frame.frame_id);
                                    return Ok(FlowControl::ContinueLoop(span));
                                }
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
            "수" | "셈수" => match value {
                Value::Num(qty) if qty.dim.is_dimensionless() => Ok(()),
                Value::Num(_) => Err(RuntimeError::UnitMismatch { span }),
                _ => Err(type_mismatch_detail(
                    if canonical == "셈수" {
                        "셈수"
                    } else {
                        "수"
                    },
                    value,
                    span,
                )),
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

    fn eval_boim(&mut self, entries: &[Binding]) -> Result<(), RuntimeError> {
        let mut evaluated = Vec::with_capacity(entries.len());
        for entry in entries {
            evaluated.push((entry.name.clone(), self.eval_expr(&entry.value)?));
        }
        if evaluated.is_empty() {
            return Ok(());
        }

        let mut tokens = match self.state.get(&Key::new(BOGAE_SHOW_LINES_TAG)).cloned() {
            Some(Value::List(list)) => list.items,
            _ => Vec::new(),
        };
        for (key, value) in &evaluated {
            tokens.push(Value::Str("table.row".to_string()));
            tokens.push(Value::Str(key.clone()));
            tokens.push(Value::Str(value.display()));
        }
        self.state.set(
            Key::new(BOGAE_SHOW_LINES_TAG),
            Value::List(ListValue { items: tokens }),
        );

        let Some(x) = pick_boim_axis_value(&evaluated, &["x축", "t", "시간", "tick"]) else {
            return Ok(());
        };
        let Some(y) = pick_boim_y_value(&evaluated) else {
            return Ok(());
        };
        let mut points = match self.state.get(&Key::new(BOGAE_GRAPH_POINTS_F_TAG)).cloned() {
            Some(Value::List(list)) => list.items,
            _ => Vec::new(),
        };
        let mut point = BTreeMap::new();
        insert_value_map_entry(&mut point, "x", Value::Num(quantity_plain(x)));
        insert_value_map_entry(&mut point, "y", Value::Num(quantity_plain(y)));
        points.push(Value::Map(MapValue { entries: point }));
        self.state.set(
            Key::new(BOGAE_GRAPH_POINTS_F_TAG),
            Value::List(ListValue { items: points }),
        );
        Ok(())
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
                FlowControl::ContinueLoop(span) => return Ok(FlowControl::ContinueLoop(span)),
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
            const_scopes: self.const_scopes.clone(),
            pending_signals: self.pending_signals.clone(),
            deferred_assign_frames: self.deferred_assign_frames.clone(),
            flow_assigns: self.flow_assigns.clone(),
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

    fn apply_nuri_world_reset(&mut self, keep_live_control_values: bool) {
        let Some(snapshot) = self.nuri_reset_snapshot.clone() else {
            return;
        };
        let live_controls = if keep_live_control_values {
            Some(capture_live_control_values(&self.state))
        } else {
            None
        };
        self.state = snapshot.state;
        self.const_scopes = snapshot.const_scopes;
        if let Some(entries) = live_controls {
            clear_live_control_values(&mut self.state);
            restore_live_control_values(&mut self.state, &entries);
        }
        self.pending_signals = snapshot.pending_signals;
        self.deferred_assign_frames = snapshot.deferred_assign_frames;
        self.flow_assigns = snapshot.flow_assigns;
        self.current_entity_stack = snapshot.current_entity_stack;
        self.aborted = snapshot.aborted;
        self.lifecycle_pan_name_to_index = snapshot.lifecycle_pan_name_to_index;
        self.lifecycle_madang_name_to_index = snapshot.lifecycle_madang_name_to_index;
        self.lifecycle_active_pan = snapshot.lifecycle_active_pan;
        self.lifecycle_active_madang = snapshot.lifecycle_active_madang;
    }

    fn apply_nuri_view_reset(&mut self) {
        let Some(snapshot) = self.nuri_reset_snapshot.clone() else {
            return;
        };
        self.bogae_requested = snapshot.bogae_requested;
        self.bogae_requested_tick = snapshot.bogae_requested_tick;
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
                let pan_named_index = self.lifecycle_pan_name_to_index.get(target_name).copied();
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
                let next_index = explicit_target_index.unwrap_or_else(|| {
                    match self.lifecycle_active_index(family) {
                        Some(active) if active + 1 < units_len => active + 1,
                        Some(_) | None => 0,
                    }
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
            const_scopes: self.const_scopes.clone(),
            pending_signals_len: self.pending_signals.len(),
            deferred_assign_frames: self.deferred_assign_frames.clone(),
            flow_assigns: self.flow_assigns.clone(),
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
        self.const_scopes = frame.const_scopes;
        self.pending_signals.truncate(frame.pending_signals_len);
        self.deferred_assign_frames = frame.deferred_assign_frames;
        self.flow_assigns = frame.flow_assigns;
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

    fn apply_flow_fixed_point(&mut self) -> Result<(), RuntimeError> {
        let records = std::mem::take(&mut self.flow_assigns);
        if records.is_empty() {
            return Ok(());
        }

        let mut by_key: BTreeMap<Key, usize> = BTreeMap::new();
        for (index, record) in records.iter().enumerate() {
            if by_key.insert(record.key.clone(), index).is_some() {
                return Err(RuntimeError::FlowMultipleSourceConflict {
                    target: record.key.as_str().to_string(),
                    span: record.target_span,
                });
            }
        }

        let flow_keys: BTreeSet<Key> = by_key.keys().cloned().collect();
        let mut dependencies: Vec<BTreeSet<Key>> = Vec::with_capacity(records.len());
        for record in &records {
            let mut deps = BTreeSet::new();
            self.collect_flow_expr_dependencies(&record.value, &mut deps)?;
            deps.retain(|key| flow_keys.contains(key) && key != &record.key);
            dependencies.push(deps);
        }

        let mut marks = vec![0u8; records.len()];
        let mut order = Vec::with_capacity(records.len());
        for index in 0..records.len() {
            Self::visit_flow_node(
                index,
                &records,
                &dependencies,
                &by_key,
                &mut marks,
                &mut order,
            )?;
        }

        let mut working_state = self.state.clone();
        let mut computed = Vec::with_capacity(order.len());
        for index in order {
            let record = &records[index];
            let saved_state = std::mem::replace(&mut self.state, working_state.clone());
            let eval_result = self.eval_expr(&record.value);
            self.state = saved_state;
            let value = eval_result?;
            working_state.set(record.key.clone(), value.clone());
            computed.push((record.key.clone(), value));
        }
        for (key, value) in computed {
            self.state.set(key, value);
        }
        Ok(())
    }

    fn visit_flow_node(
        index: usize,
        records: &[FlowAssignRecord],
        dependencies: &[BTreeSet<Key>],
        by_key: &BTreeMap<Key, usize>,
        marks: &mut [u8],
        order: &mut Vec<usize>,
    ) -> Result<(), RuntimeError> {
        match marks[index] {
            2 => return Ok(()),
            1 => {
                let record = &records[index];
                return Err(RuntimeError::FlowCircularReference {
                    target: record.key.as_str().to_string(),
                    span: record.span,
                });
            }
            _ => {}
        }
        marks[index] = 1;
        for dep in &dependencies[index] {
            if let Some(dep_index) = by_key.get(dep) {
                Self::visit_flow_node(*dep_index, records, dependencies, by_key, marks, order)?;
            }
        }
        marks[index] = 2;
        order.push(index);
        Ok(())
    }

    fn collect_flow_expr_dependencies(
        &self,
        expr: &Expr,
        out: &mut BTreeSet<Key>,
    ) -> Result<(), RuntimeError> {
        match expr {
            Expr::Path(path) => {
                out.insert(self.path_to_key(path)?);
            }
            Expr::FieldAccess { target, .. } => {
                self.collect_flow_expr_dependencies(target, out)?;
            }
            Expr::Unary { expr, .. } | Expr::SeedLiteral { body: expr, .. } => {
                self.collect_flow_expr_dependencies(expr, out)?;
            }
            Expr::Binary { left, right, .. } => {
                self.collect_flow_expr_dependencies(left, out)?;
                self.collect_flow_expr_dependencies(right, out)?;
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.collect_flow_expr_dependencies(&arg.expr, out)?;
                }
            }
            Expr::FormulaEval { bindings, .. } | Expr::Pack { bindings, .. } => {
                for binding in bindings {
                    self.collect_flow_expr_dependencies(&binding.value, out)?;
                }
            }
            Expr::TemplateFill {
                template, bindings, ..
            } => {
                self.collect_flow_expr_dependencies(template, out)?;
                for binding in bindings {
                    self.collect_flow_expr_dependencies(&binding.value, out)?;
                }
            }
            Expr::FormulaFill {
                formula, bindings, ..
            } => {
                self.collect_flow_expr_dependencies(formula, out)?;
                for binding in bindings {
                    self.collect_flow_expr_dependencies(&binding.value, out)?;
                }
            }
            Expr::Literal(_, _)
            | Expr::Atom { .. }
            | Expr::Formula { .. }
            | Expr::Assertion { .. }
            | Expr::Template { .. } => {}
        }
        Ok(())
    }

    fn enter_const_scope(&mut self) {
        self.const_scopes.push(BTreeSet::new());
    }

    fn exit_const_scope(&mut self) {
        self.const_scopes.pop();
    }

    fn declare_const(&mut self, key: Key) {
        if let Some(scope) = self.const_scopes.last_mut() {
            scope.insert(key);
        }
    }

    fn is_const(&self, key: &Key) -> bool {
        self.const_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(key))
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
                captured: self.lambda_capture_snapshot(),
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
            BinaryOp::RelationEq => self.eval_relation_eq(left, right, span),
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
        // Try stripping call tails (short and long forms), but never guess on ambiguity.
        let mut tail_candidates = Vec::new();
        for tail in CALL_TAILS {
            if let Some(stem) = resolved_name.strip_suffix(tail) {
                if self.user_seeds.contains_key(stem) {
                    tail_candidates.push(stem.to_string());
                }
            }
        }
        if tail_candidates.len() == 1 {
            let stem = &tail_candidates[0];
            let seed = self.user_seeds.get(stem).cloned().expect("candidate seed");
            let bound = self.bind_seed_args(&seed, args, &values, span)?;
            return self.eval_user_seed(stem, &seed, &bound, span);
        }
        if tail_candidates.len() > 1 {
            return Err(RuntimeError::CallTailAmbiguous {
                name: resolved_name,
                candidates: tail_candidates,
                span,
            });
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
                self.apply_nuri_world_reset(true);
                Ok(Value::None)
            }
            "판다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                self.apply_nuri_world_reset(true);
                Ok(Value::None)
            }
            "누리다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                self.apply_nuri_world_reset(true);
                Ok(Value::None)
            }
            "보개다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                self.apply_nuri_view_reset();
                Ok(Value::None)
            }
            "모두다시" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args",
                        span,
                    });
                }
                self.apply_nuri_world_reset(false);
                self.apply_nuri_view_reset();
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
            "수" | "셈수" => {
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
                        let raw =
                            Fixed64::parse_literal(text).ok_or(RuntimeError::TypeMismatch {
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
                Ok(make_exact_numeric_value(
                    NUMERIC_KIND_BIG_INT,
                    &[(EXACT_NUMERIC_BIGINT_FIELD, raw)],
                ))
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
                let normalized = ddonirang_numeric::normalize_rational_text(
                    &numerator,
                    &denominator,
                    Some(NUMERIC_KIND_RATIONAL.to_string()),
                )
                .map_err(|_| RuntimeError::MathDivZero { span })?;
                Ok(make_exact_numeric_value(
                    NUMERIC_KIND_RATIONAL,
                    &[
                        (EXACT_NUMERIC_RATIONAL_NUM_FIELD, normalized.num),
                        (EXACT_NUMERIC_RATIONAL_DEN_FIELD, normalized.den),
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
                let canon = factor_canon_text(&raw).unwrap_or_else(|| raw.clone());
                Ok(make_exact_numeric_value(
                    NUMERIC_KIND_FACTOR,
                    &[(EXACT_NUMERIC_FACTOR_VALUE_FIELD, raw), ("정본", canon)],
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
                if values.len() == 2 && exact_numeric_kind(&values[0]) == Some(NUMERIC_KIND_FACTOR)
                {
                    let Some(base) = exact_text_from_value(&values[0], span)? else {
                        return Err(type_mismatch_detail("곱수", &values[0], span));
                    };
                    let exp = parse_integer_like_value(&values[1], span)?;
                    if exp < 0 {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "non-negative integer exponent",
                            span,
                        });
                    }
                    let mut value = ddonirang_numeric::ExactText {
                        num: "1".to_string(),
                        den: "1".to_string(),
                        kind: Some(NUMERIC_KIND_FACTOR.to_string()),
                    };
                    for _ in 0..exp {
                        value = ddonirang_numeric::exact_binary_text("*", &value, &base).map_err(
                            |_| RuntimeError::TypeMismatch {
                                expected: "exact number",
                                span,
                            },
                        )?;
                    }
                    return Ok(exact_text_to_value(value, "*", span)?);
                }
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
            "적분.속도베를레" => {
                if values.len() != 5 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "position, velocity, acceleration, next_acceleration, dt",
                        span,
                    });
                }
                let position = expect_quantity_value(&values[0], span)?;
                let velocity = expect_quantity_value(&values[1], span)?;
                let acceleration = expect_quantity_value(&values[2], span)?;
                let next_acceleration = expect_quantity_value(&values[3], span)?;
                let dt = expect_quantity_value(&values[4], span)?;
                ensure_same_dim(&acceleration, &next_acceleration, span)?;

                let dt_half =
                    dt.raw
                        .checked_div(Fixed64::from_int(2))
                        .ok_or(RuntimeError::MathDivZero { span })?;
                let delta_v_half = Quantity::new(
                    acceleration.raw.saturating_mul(dt_half),
                    acceleration.dim.add(dt.dim),
                );
                ensure_same_dim(&velocity, &delta_v_half, span)?;
                let v_half =
                    Quantity::new(velocity.raw.saturating_add(delta_v_half.raw), velocity.dim);

                let delta_x = Quantity::new(
                    v_half.raw.saturating_mul(dt.raw),
                    v_half.dim.add(dt.dim),
                );
                ensure_same_dim(&position, &delta_x, span)?;
                let next_position =
                    Quantity::new(position.raw.saturating_add(delta_x.raw), position.dim);

                let delta_v_next = Quantity::new(
                    next_acceleration.raw.saturating_mul(dt_half),
                    next_acceleration.dim.add(dt.dim),
                );
                ensure_same_dim(&v_half, &delta_v_next, span)?;
                let next_velocity =
                    Quantity::new(v_half.raw.saturating_add(delta_v_next.raw), velocity.dim);

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
            "수치해.이분법" => {
                let (math, var_name, lower, upper, iterations) =
                    expect_numeric_bisection_args(values, span, "수치해.이분법")?;
                let prepared = prepare_numeric_formula(&math, &var_name, span, "수치해.이분법")?;
                let (root, residual, used_iterations) =
                    numeric_bisection_root(&prepared, lower, upper, iterations, span)?;
                Ok(Value::List(ListValue {
                    items: vec![
                        Value::Num(root),
                        Value::Num(residual),
                        Value::Num(Quantity::new(
                            Fixed64::from_int(used_iterations as i64),
                            UnitDim::zero(),
                        )),
                        Value::Str("이분법".to_string()),
                    ],
                }))
            }
            "선형부등식.풀기" => eval_linear_inequality_solve(values, span),
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
            "격자.만들기" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "width, height, default",
                        span,
                    });
                }
                let width = quantity_to_int(&expect_quantity_value(&values[0], span)?, span)?;
                let height = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                make_std_grid(width, height, values[2].clone(), span)
            }
            "격자.너비" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid",
                        span,
                    });
                }
                let fields = std_grid_fields(&values[0], span)?;
                let (width, _) = std_grid_dims(fields, span)?;
                Ok(fixed_value(width))
            }
            "격자.높이" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid",
                        span,
                    });
                }
                let fields = std_grid_fields(&values[0], span)?;
                let (_, height) = std_grid_dims(fields, span)?;
                Ok(fixed_value(height))
            }
            "격자.값" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid, x, y",
                        span,
                    });
                }
                let fields = std_grid_fields(&values[0], span)?;
                let x = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                let y = quantity_to_int(&expect_quantity_value(&values[2], span)?, span)?;
                let idx = std_grid_index(fields, x, y, span)?;
                Ok(std_grid_cells(fields, span)?
                    .get(idx)
                    .cloned()
                    .unwrap_or(Value::None))
            }
            "격자.바꾼값" => {
                if values.len() != 4 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid, x, y, value",
                        span,
                    });
                }
                let fields = std_grid_fields(&values[0], span)?;
                let x = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                let y = quantity_to_int(&expect_quantity_value(&values[2], span)?, span)?;
                let idx = std_grid_index(fields, x, y, span)?;
                let mut next_fields = fields.clone();
                let mut cells = std_grid_cells(fields, span)?.clone();
                if idx >= cells.len() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid cells matching size",
                        span,
                    });
                }
                cells[idx] = values[3].clone();
                next_fields.insert("칸들".to_string(), Value::List(ListValue { items: cells }));
                Ok(Value::Pack(PackValue {
                    fields: next_fields,
                }))
            }
            "격자.안인가" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid, x, y",
                        span,
                    });
                }
                let x = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                let y = quantity_to_int(&expect_quantity_value(&values[2], span)?, span)?;
                Ok(Value::Bool(std_grid_inside(&values[0], x, y, span)?))
            }
            "격자.막혔나" => {
                if values.len() != 4 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid, x, y, blocked values",
                        span,
                    });
                }
                let x = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                let y = quantity_to_int(&expect_quantity_value(&values[2], span)?, span)?;
                if !std_grid_inside(&values[0], x, y, span)? {
                    return Ok(Value::Bool(true));
                }
                let fields = std_grid_fields(&values[0], span)?;
                let idx = std_grid_index(fields, x, y, span)?;
                let cell = std_grid_cells(fields, span)?
                    .get(idx)
                    .cloned()
                    .unwrap_or(Value::None);
                let blocked = match &values[3] {
                    Value::List(list) => list.items.iter().any(|item| item == &cell),
                    other => other == &cell,
                };
                Ok(Value::Bool(blocked))
            }
            "격자.길찾기" => {
                if values.len() != 6 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid, start x, start y, goal x, goal y, blocked values",
                        span,
                    });
                }
                let start_x = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                let start_y = quantity_to_int(&expect_quantity_value(&values[2], span)?, span)?;
                let goal_x = quantity_to_int(&expect_quantity_value(&values[3], span)?, span)?;
                let goal_y = quantity_to_int(&expect_quantity_value(&values[4], span)?, span)?;
                std_grid_pathfind(
                    &values[0], start_x, start_y, goal_x, goal_y, &values[5], span,
                )
            }
            "블록조각.만들기" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "piece cells",
                        span,
                    });
                }
                let cells = std_block_piece_cells_from_value(&values[0], span)?;
                Ok(make_std_block_piece(cells))
            }
            "블록조각.칸목록" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "block piece",
                        span,
                    });
                }
                let cells = std_block_piece_cells(&values[0], span)?;
                Ok(std_block_piece_cells_value(&cells))
            }
            "블록조각.이동" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "block piece, dx, dy",
                        span,
                    });
                }
                let cells = std_block_piece_cells(&values[0], span)?;
                let dx = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                let dy = quantity_to_int(&expect_quantity_value(&values[2], span)?, span)?;
                Ok(make_std_block_piece(
                    cells.into_iter().map(|(x, y)| (x + dx, y + dy)).collect(),
                ))
            }
            "블록조각.회전" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "block piece, direction",
                        span,
                    });
                }
                let cells = std_block_piece_cells(&values[0], span)?;
                let Value::Str(direction) = &values[1] else {
                    return Err(type_mismatch_detail("string", &values[1], span));
                };
                let rotated = cells
                    .into_iter()
                    .map(|(x, y)| match direction.as_str() {
                        "오른쪽" => Ok((-y, x)),
                        "왼쪽" => Ok((y, -x)),
                        "뒤집기" => Ok((-x, -y)),
                        _ => Err(RuntimeError::Pack {
                            message: format!("블록조각 회전 방향을 지원하지 않습니다: {direction}"),
                            span,
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(make_std_block_piece(rotated))
            }
            "블록조각.충돌?" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "block piece, grid, blocked values",
                        span,
                    });
                }
                Ok(Value::Bool(std_block_piece_collides(
                    &values[0], &values[1], &values[2], span,
                )?))
            }
            "블록조각.고정" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "block piece, grid, value",
                        span,
                    });
                }
                std_block_piece_lock(&values[0], &values[1], values[2].clone(), span)
            }
            "물리1d.위치갱신" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "position, velocity, dt",
                        span,
                    });
                }
                let position = expect_quantity_value(&values[0], span)?;
                let velocity = expect_quantity_value(&values[1], span)?;
                let dt = expect_quantity_value(&values[2], span)?;
                ensure_dimensionless(&dt, span)?;
                ensure_same_dim(&position, &velocity, span)?;
                Ok(Value::Num(Quantity::new(
                    position
                        .raw
                        .saturating_add(velocity.raw.saturating_mul(dt.raw)),
                    position.dim,
                )))
            }
            "물리1d.속도갱신" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "velocity, acceleration, dt",
                        span,
                    });
                }
                let velocity = expect_quantity_value(&values[0], span)?;
                let acceleration = expect_quantity_value(&values[1], span)?;
                let dt = expect_quantity_value(&values[2], span)?;
                ensure_dimensionless(&dt, span)?;
                ensure_same_dim(&velocity, &acceleration, span)?;
                Ok(Value::Num(Quantity::new(
                    velocity
                        .raw
                        .saturating_add(acceleration.raw.saturating_mul(dt.raw)),
                    velocity.dim,
                )))
            }
            "물리1d.탄성충돌1d" => {
                if values.len() != 4 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "m1, v1, m2, v2",
                        span,
                    });
                }
                let m1 = expect_quantity_value(&values[0], span)?;
                let v1 = expect_quantity_value(&values[1], span)?;
                let m2 = expect_quantity_value(&values[2], span)?;
                let v2 = expect_quantity_value(&values[3], span)?;
                ensure_dimensionless(&m1, span)?;
                ensure_dimensionless(&m2, span)?;
                ensure_same_dim(&v1, &v2, span)?;
                let denom = m1.raw.saturating_add(m2.raw);
                if denom.raw() == 0 {
                    return Err(RuntimeError::MathDivZero { span });
                }
                let two = Fixed64::from_int(2);
                let next_v1_num = m1
                    .raw
                    .saturating_sub(m2.raw)
                    .saturating_mul(v1.raw)
                    .saturating_add(two.saturating_mul(m2.raw).saturating_mul(v2.raw));
                let next_v2_num = two
                    .saturating_mul(m1.raw)
                    .saturating_mul(v1.raw)
                    .saturating_add(m2.raw.saturating_sub(m1.raw).saturating_mul(v2.raw));
                let next_v1 = next_v1_num
                    .checked_div(denom)
                    .ok_or(RuntimeError::MathDivZero { span })?;
                let next_v2 = next_v2_num
                    .checked_div(denom)
                    .ok_or(RuntimeError::MathDivZero { span })?;
                Ok(Value::List(ListValue {
                    items: vec![
                        Value::Num(Quantity::new(next_v1, v1.dim)),
                        Value::Num(Quantity::new(next_v2, v1.dim)),
                    ],
                }))
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
            "입력사상.만들기" => {
                if values.is_empty() {
                    return Ok(make_std_input_map(BTreeMap::new()));
                }
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no args or pack",
                        span,
                    });
                }
                let Value::Pack(pack) = &values[0] else {
                    return Err(type_mismatch_detail("pack", &values[0], span));
                };
                Ok(make_std_input_map(pack.fields.clone()))
            }
            "입력사상.방향" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "input map",
                        span,
                    });
                }
                let map = std_input_map_fields(&values[0], span)?;
                let left = input_map_action_pressed(&self.state, &map, "왼쪽") as i64;
                let right = input_map_action_pressed(&self.state, &map, "오른쪽") as i64;
                let up = input_map_action_pressed(&self.state, &map, "위") as i64;
                let down = input_map_action_pressed(&self.state, &map, "아래") as i64;
                Ok(Value::List(ListValue {
                    items: vec![fixed_value(right - left), fixed_value(down - up)],
                }))
            }
            "입력사상.동작" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "input map, action",
                        span,
                    });
                }
                let map = std_input_map_fields(&values[0], span)?;
                let action = match &values[1] {
                    Value::Str(text) => text.clone(),
                    other => return Err(type_mismatch_detail("string", other, span)),
                };
                Ok(Value::Bool(input_map_action_pressed(
                    &self.state,
                    &map,
                    &action,
                )))
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
            "무작위가방.만들기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "seed, candidates",
                        span,
                    });
                }
                let seed = value_to_seed_u64(&values[0], span)?;
                let Value::List(candidates) = &values[1] else {
                    return Err(type_mismatch_detail("list", &values[1], span));
                };
                if candidates.items.is_empty() {
                    return Err(RuntimeError::Pack {
                        message: "무작위가방 후보들은 비어 있을 수 없습니다".to_string(),
                        span,
                    });
                }
                Ok(make_std_random_bag(
                    seed,
                    seed,
                    candidates.items.clone(),
                    candidates.items.clone(),
                    0,
                ))
            }
            "무작위가방.꺼내기" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "random bag",
                        span,
                    });
                }
                let (value, bag) = std_random_bag_draw_once(&values[0], span)?;
                let mut fields = BTreeMap::new();
                fields.insert(
                    STD_RANDOM_BAG_KIND_FIELD.to_string(),
                    Value::Str(STD_RANDOM_BAG_DRAW_KIND.to_string()),
                );
                fields.insert(STD_RANDOM_BAG_VALUE_FIELD.to_string(), value);
                fields.insert(STD_RANDOM_BAG_BAG_FIELD.to_string(), bag);
                Ok(Value::Pack(PackValue { fields }))
            }
            "무작위가방.미리보기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "random bag, count",
                        span,
                    });
                }
                let count = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                if count < 0 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "non-negative preview count",
                        span,
                    });
                }
                let mut bag = values[0].clone();
                let mut preview = Vec::new();
                for _ in 0..count {
                    let (value, next_bag) = std_random_bag_draw_once(&bag, span)?;
                    preview.push(value);
                    bag = next_bag;
                }
                Ok(Value::List(ListValue { items: preview }))
            }
            "무작위가방.남은것" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "random bag",
                        span,
                    });
                }
                let fields = std_random_bag_fields(&values[0], span)?;
                Ok(Value::List(ListValue {
                    items: std_random_bag_remaining(fields, span)?,
                }))
            }
            "무작위가방.비었나" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "random bag",
                        span,
                    });
                }
                let fields = std_random_bag_fields(&values[0], span)?;
                Ok(Value::Bool(
                    std_random_bag_remaining(fields, span)?.is_empty(),
                ))
            }
            "격자게임상태.초기화" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                Ok(make_std_grid_game_state("준비", Value::None, 0))
            }
            "격자게임상태.만들기" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "state",
                        span,
                    });
                }
                let state = expect_grid_game_state_name(&values[0], span)?;
                Ok(make_std_grid_game_state(&state, Value::None, 0))
            }
            "격자게임상태.상태" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game state",
                        span,
                    });
                }
                Ok(Value::Str(std_grid_game_state_state(
                    std_grid_game_state_fields(&values[0], span)?,
                    span,
                )?))
            }
            "격자게임상태.틱" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game state",
                        span,
                    });
                }
                Ok(fixed_value(std_grid_game_state_tick(
                    std_grid_game_state_fields(&values[0], span)?,
                    span,
                )?))
            }
            "격자게임상태.상태인가" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game state, state",
                        span,
                    });
                }
                let current =
                    std_grid_game_state_state(std_grid_game_state_fields(&values[0], span)?, span)?;
                let expected = expect_grid_game_state_name(&values[1], span)?;
                Ok(Value::Bool(current == expected))
            }
            "격자게임상태.바꾸기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game state, state",
                        span,
                    });
                }
                let fields = std_grid_game_state_fields(&values[0], span)?;
                let state = expect_grid_game_state_name(&values[1], span)?;
                let tick = std_grid_game_state_tick(fields, span)?.saturating_add(1);
                Ok(make_std_grid_game_state(
                    &state,
                    Value::Str(std_grid_game_state_state(fields, span)?),
                    tick,
                ))
            }
            "격자게임상태.멈춤" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game state",
                        span,
                    });
                }
                let fields = std_grid_game_state_fields(&values[0], span)?;
                let state = std_grid_game_state_state(fields, span)?;
                let previous = if state == "멈춤" {
                    std_grid_game_state_previous(fields)
                } else {
                    Value::Str(state)
                };
                Ok(make_std_grid_game_state(
                    "멈춤",
                    previous,
                    std_grid_game_state_tick(fields, span)?.saturating_add(1),
                ))
            }
            "격자게임상태.재개" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game state",
                        span,
                    });
                }
                let fields = std_grid_game_state_fields(&values[0], span)?;
                let state = std_grid_game_state_state(fields, span)?;
                if state != "멈춤" {
                    return Ok(values[0].clone());
                }
                let previous = match std_grid_game_state_previous(fields) {
                    Value::Str(candidate)
                        if is_allowed_grid_game_state(&candidate) && candidate != "멈춤" =>
                    {
                        candidate
                    }
                    _ => "진행".to_string(),
                };
                Ok(make_std_grid_game_state(
                    &previous,
                    Value::None,
                    std_grid_game_state_tick(fields, span)?.saturating_add(1),
                ))
            }
            "테트로미노.이름목록" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                Ok(Value::List(ListValue {
                    items: tetromino_names()
                        .into_iter()
                        .map(|name| Value::Str(name.to_string()))
                        .collect(),
                }))
            }
            "테트로미노.만들기" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "tetromino name",
                        span,
                    });
                }
                let name = value_to_tetromino_name(&values[0], span)?;
                Ok(make_std_block_piece(tetromino_cells(&name, span)?))
            }
            "테트로미노.목록" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                Ok(Value::List(ListValue {
                    items: tetromino_names()
                        .into_iter()
                        .map(|name| {
                            make_std_block_piece(
                                tetromino_cells(name, span).expect("known tetromino"),
                            )
                        })
                        .collect(),
                }))
            }
            "격자줄.찬줄목록" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid, empty value",
                        span,
                    });
                }
                Ok(Value::List(ListValue {
                    items: std_grid_line_full_rows(&values[0], &values[1], span)?
                        .into_iter()
                        .map(fixed_value)
                        .collect(),
                }))
            }
            "격자줄.지우기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid, empty value",
                        span,
                    });
                }
                std_grid_line_clear(&values[0], values[1].clone(), span)
            }
            "낙하조각.만들기" | "격자게임.스폰" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "piece, x, y",
                        span,
                    });
                }
                let _ = std_block_piece_cells(&values[0], span)?;
                let x = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                let y = quantity_to_int(&expect_quantity_value(&values[2], span)?, span)?;
                Ok(make_std_falling_piece(values[0].clone(), x, y))
            }
            "낙하조각.조각" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece",
                        span,
                    });
                }
                let (piece, _, _) = std_falling_piece_parts(&values[0], span)?;
                Ok(piece)
            }
            "낙하조각.위치" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece",
                        span,
                    });
                }
                let (_, x, y) = std_falling_piece_parts(&values[0], span)?;
                Ok(Value::List(ListValue {
                    items: vec![fixed_value(x), fixed_value(y)],
                }))
            }
            "낙하조각.배치" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece",
                        span,
                    });
                }
                Ok(std_block_piece_cells_value(&std_falling_piece_cells(
                    &values[0], span,
                )?))
            }
            "낙하조각.이동" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece, dx, dy",
                        span,
                    });
                }
                let (piece, x, y) = std_falling_piece_parts(&values[0], span)?;
                let dx = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                let dy = quantity_to_int(&expect_quantity_value(&values[2], span)?, span)?;
                Ok(make_std_falling_piece(piece, x + dx, y + dy))
            }
            "낙하조각.회전" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece, direction",
                        span,
                    });
                }
                let Value::Str(direction) = &values[1] else {
                    return Err(type_mismatch_detail("string", &values[1], span));
                };
                std_falling_piece_rotate(&values[0], direction, span)
            }
            "격자게임.놓을수있나" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece, grid, blocked values",
                        span,
                    });
                }
                Ok(Value::Bool(std_grid_game_placeable(
                    &values[0], &values[1], &values[2], span,
                )?))
            }
            "격자게임.입력적용" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece, input map",
                        span,
                    });
                }
                std_grid_game_apply_input(&values[0], &values[1], &self.state, span)
            }
            "격자게임.중력틱" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece",
                        span,
                    });
                }
                let (piece, x, y) = std_falling_piece_parts(&values[0], span)?;
                Ok(make_std_falling_piece(piece, x, y + 1))
            }
            "격자게임.고정" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece, grid, value",
                        span,
                    });
                }
                std_block_piece_lock(
                    &std_falling_piece_block(&values[0], span)?,
                    &values[1],
                    values[2].clone(),
                    span,
                )
            }
            "격자게임.다음조각" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "random bag",
                        span,
                    });
                }
                std_grid_game_next_piece(&values[0], span)
            }
            "격자게임.한틱" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session, input map",
                        span,
                    });
                }
                std_grid_game_tick(&values[0], &values[1], &self.state, span)
            }
            "격자게임.회전시도" => {
                if values.len() != 4 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "falling piece, grid, blocked values, direction",
                        span,
                    });
                }
                let Value::Str(direction) = &values[3] else {
                    return Err(type_mismatch_detail("string", &values[3], span));
                };
                std_grid_game_rotation_try(&values[0], &values[1], &values[2], direction, span)
            }
            "격자게임홀드.초기화" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                Ok(make_std_grid_game_hold(Value::None, false))
            }
            "격자게임홀드.칸" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game hold",
                        span,
                    });
                }
                std_grid_game_hold_piece(std_grid_game_hold_fields(&values[0], span)?, span)
            }
            "격자게임홀드.썼나" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game hold",
                        span,
                    });
                }
                Ok(Value::Bool(std_grid_game_hold_used(
                    std_grid_game_hold_fields(&values[0], span)?,
                    span,
                )?))
            }
            "격자게임홀드.교체" => {
                if values.len() != 5 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game hold, falling piece, random bag, x, y",
                        span,
                    });
                }
                let x = quantity_to_int(&expect_quantity_value(&values[3], span)?, span)?;
                let y = quantity_to_int(&expect_quantity_value(&values[4], span)?, span)?;
                std_grid_game_hold_swap(&values[0], &values[1], &values[2], x, y, span)
            }
            "격자게임홀드.초기화턴" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game hold",
                        span,
                    });
                }
                let piece =
                    std_grid_game_hold_piece(std_grid_game_hold_fields(&values[0], span)?, span)?;
                Ok(make_std_grid_game_hold(piece, false))
            }
            "격자게임점수.초기화" => {
                if !values.is_empty() {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "no arguments",
                        span,
                    });
                }
                Ok(make_std_grid_game_score(0, 0))
            }
            "격자게임점수.더하기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "score, cleared lines",
                        span,
                    });
                }
                let cleared = quantity_to_int(&expect_quantity_value(&values[1], span)?, span)?;
                std_grid_game_score_add(&values[0], cleared, span)
            }
            "격자게임점수.점수" | "격자게임점수.줄수" | "격자게임점수.레벨" =>
            {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "score",
                        span,
                    });
                }
                let field = match name {
                    "격자게임점수.점수" => "점수",
                    "격자게임점수.줄수" => "줄수",
                    _ => "레벨",
                };
                Ok(fixed_value(std_score_int_field(
                    std_grid_game_score_fields(&values[0], span)?,
                    field,
                    span,
                )?))
            }
            "격자게임세션.만들기" => {
                if values.len() != 5 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid, bag, state, score, falling piece",
                        span,
                    });
                }
                Ok(make_std_grid_game_session(
                    values[0].clone(),
                    values[1].clone(),
                    values[2].clone(),
                    values[3].clone(),
                    values[4].clone(),
                ))
            }
            "격자게임세션.격자"
            | "격자게임세션.가방"
            | "격자게임세션.상태"
            | "격자게임세션.점수"
            | "격자게임세션.낙하조각" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session",
                        span,
                    });
                }
                let field = match name {
                    "격자게임세션.격자" => "격자",
                    "격자게임세션.가방" => "가방",
                    "격자게임세션.상태" => "상태",
                    "격자게임세션.점수" => "점수",
                    _ => "낙하조각",
                };
                std_session_field(std_grid_game_session_fields(&values[0], span)?, field, span)
            }
            "격자게임세션.바꾸기" => {
                if values.len() != 6 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "session, grid, bag, state, score, falling piece",
                        span,
                    });
                }
                let _ = std_grid_game_session_fields(&values[0], span)?;
                Ok(make_std_grid_game_session(
                    values[1].clone(),
                    values[2].clone(),
                    values[3].clone(),
                    values[4].clone(),
                    values[5].clone(),
                ))
            }
            "격자게임보기.칸목록" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session, empty value, falling value",
                        span,
                    });
                }
                std_grid_game_view_cells(&values[0], &values[1], values[2].clone(), span)
            }
            "격자게임보기.문자판" => {
                if values.len() != 3 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session, empty value, falling value",
                        span,
                    });
                }
                std_grid_game_view_text(&values[0], &values[1], values[2].clone(), span)
            }
            "격자게임보기.상태요약" => {
                if values.len() != 1 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session",
                        span,
                    });
                }
                std_grid_game_view_summary(&values[0], span)
            }
            "격자게임보기.유령조각" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session, blocked values",
                        span,
                    });
                }
                std_grid_game_ghost_piece(&values[0], &values[1], span)
            }
            "격자게임보기.유령보개목록" => {
                if values.len() != 6 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session, empty value, falling value, ghost value, cell size, blocked values",
                        span,
                    });
                }
                std_grid_game_ghost_bogae_drawlist(
                    &values[0],
                    &values[1],
                    values[2].clone(),
                    values[3].clone(),
                    &values[4],
                    &values[5],
                    span,
                )
            }
            "격자게임보기.보개목록" => {
                if values.len() != 4 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session, empty value, falling value, cell size",
                        span,
                    });
                }
                std_grid_game_bogae_drawlist(
                    &values[0],
                    &values[1],
                    values[2].clone(),
                    &values[3],
                    span,
                )
            }
            "격자게임보기.보개크기" => {
                if values.len() != 2 {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "grid game session, cell size",
                        span,
                    });
                }
                std_grid_game_bogae_size(&values[0], &values[1], span)
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
                let transformed = transform_symbolic_formula_value(math, options, "diff", span)?;
                Ok(Value::Math(transformed))
            }
            "적분하기" => {
                let (math, options) = expect_formula_transform(values, span, "적분하기")?;
                let transformed = transform_symbolic_formula_value(math, options, "int", span)?;
                Ok(Value::Math(transformed))
            }
            "정리하기" => {
                let math = expect_single_formula(values, span, "정리하기")?;
                let transformed = transform_symbolic_formula_value(
                    math,
                    FormulaTransformOptions::default(),
                    "simplify",
                    span,
                )?;
                Ok(Value::Math(transformed))
            }
            "전개하기" => {
                let math = expect_single_formula(values, span, "전개하기")?;
                let transformed = transform_symbolic_formula_value(
                    math,
                    FormulaTransformOptions::default(),
                    "expand",
                    span,
                )?;
                Ok(Value::Math(transformed))
            }
            "인수분해하기" => {
                let math = expect_single_formula(values, span, "인수분해하기")?;
                let transformed = transform_symbolic_formula_value(
                    math,
                    FormulaTransformOptions::default(),
                    "factor",
                    span,
                )?;
                Ok(Value::Math(transformed))
            }
            "동치인가" => {
                let (left, right) = expect_two_formulas(values, span, "동치인가")?;
                Ok(Value::Bool(symbolic_formulas_equivalent(
                    &left, &right, span,
                )?))
            }
            "잇기" => {
                let (left, right) = expect_two_formulas(values, span, "잇기")?;
                Ok(make_relation_pack(left, right))
            }
            "이음관계.관계목록" => eval_endpoint_relation_list(values, span),
            "이음관계.정규화" => eval_endpoint_relation_normalize(values, span),
            "이음관계.방정식목록" => eval_endpoint_formula_relation_list(values, span),
            "이음관계.방정식화" => eval_endpoint_formula_relation_set(values, span),
            "이음관계.값관계목록" => {
                eval_endpoint_boundary_value_relation_list(values, span)
            }
            "이음관계.값주입" => eval_endpoint_boundary_value_injection(values, span),
            "이음관계.풀기" => eval_endpoint_explicit_solve(values, span),
            "이음관계.풀이값목록" => eval_endpoint_solve_result_value_list(values, span),
            "이음관계.풀이원복" => eval_endpoint_solve_result_remap(values, span),
            "이음관계.범위위반목록" => {
                eval_endpoint_boundary_range_violation_list(values, span)
            }
            "이음관계.범위검사" => eval_endpoint_boundary_range_check(values, span),
            "이음관계.풀고범위위반목록" => {
                eval_endpoint_explicit_solve_range_violation_list(values, span)
            }
            "이음관계.풀고범위검사" => {
                eval_endpoint_explicit_solve_range_check(values, span)
            }
            "이음관계.풀고범위행목록" => {
                eval_endpoint_solve_range_report_rows(values, span)
            }
            "이음관계.풀고범위보고서" => eval_endpoint_solve_range_report(values, span),
            "이음관계.보고서문자표" => {
                eval_endpoint_solve_range_text_report(values, span)
            }
            "이음관계.풀고범위문자표" => {
                eval_endpoint_explicit_solve_range_text_report(values, span)
            }
            "이음관계.풀고범위케이스" => eval_endpoint_solve_range_case(values, span),
            "이음관계.풀고범위스위트" => {
                eval_endpoint_solve_range_case_suite(values, span)
            }
            "이음관계.풀고범위스위트문자표" => {
                eval_endpoint_solve_range_case_suite_text(values, span)
            }
            "이음관계.풀고범위스위트상세문자표" => {
                eval_endpoint_solve_range_case_suite_detail_text(values, span)
            }
            "이음관계.풀고범위실행상세문자표" => {
                eval_endpoint_solve_range_case_suite_run_detail_text(values, span)
            }
            "이음관계.풀고범위스위트요약" => {
                eval_endpoint_solve_range_case_suite_summary(values, span)
            }
            "이음관계.풀고범위실행요약" => {
                eval_endpoint_solve_range_case_suite_run_summary(values, span)
            }
            "이음관계.풀고범위스위트판정" => {
                eval_endpoint_solve_range_case_suite_check(values, span)
            }
            "이음관계.풀고범위실행판정" => {
                eval_endpoint_solve_range_case_suite_run_check(values, span)
            }
            "증명하기" => {
                let proof = eval_symbolic_proof_tactic(values, span)?;
                Ok(Value::Pack(proof))
            }
            "방정식풀기" => {
                let relations = expect_equation_relations(values, span)?;
                eval_relation_solve_result(&relations, span)
            }
            "다항식.풀기" => eval_polynomial_solve_result(values, span),
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
            "흐름.만들기" => eval_stream_new(values, span),
            "흐름.밀어넣기" => eval_stream_push(values, span),
            "흐름.차림" => eval_stream_items(values, span),
            "흐름.최근값" => eval_stream_latest(values, span),
            "흐름.길이" => eval_stream_len(values, span),
            "흐름.용량" => eval_stream_capacity(values, span),
            "흐름.비우기" => eval_stream_clear(values, span),
            "흐름.잘라보기" => eval_stream_tail(values, span),
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
                | "격자.만들기"
                | "격자.너비"
                | "격자.높이"
                | "격자.값"
                | "격자.바꾼값"
                | "격자.안인가"
                | "격자.막혔나"
                | "격자.길찾기"
                | "테트로미노.이름목록"
                | "테트로미노.만들기"
                | "테트로미노.목록"
                | "격자줄.찬줄목록"
                | "격자줄.지우기"
                | "블록조각.만들기"
                | "블록조각.칸목록"
                | "블록조각.이동"
                | "블록조각.회전"
                | "블록조각.충돌?"
                | "블록조각.고정"
                | "낙하조각.만들기"
                | "낙하조각.조각"
                | "낙하조각.위치"
                | "낙하조각.배치"
                | "낙하조각.이동"
                | "낙하조각.회전"
                | "격자게임.놓을수있나"
                | "격자게임.입력적용"
                | "격자게임.중력틱"
                | "격자게임.고정"
                | "격자게임.스폰"
                | "격자게임.다음조각"
                | "격자게임.한틱"
                | "격자게임.회전시도"
                | "격자게임홀드.초기화"
                | "격자게임홀드.칸"
                | "격자게임홀드.썼나"
                | "격자게임홀드.교체"
                | "격자게임홀드.초기화턴"
                | "격자게임점수.초기화"
                | "격자게임점수.더하기"
                | "격자게임점수.점수"
                | "격자게임점수.줄수"
                | "격자게임점수.레벨"
                | "격자게임세션.만들기"
                | "격자게임세션.격자"
                | "격자게임세션.가방"
                | "격자게임세션.상태"
                | "격자게임세션.점수"
                | "격자게임세션.낙하조각"
                | "격자게임세션.바꾸기"
                | "격자게임보기.칸목록"
                | "격자게임보기.문자판"
                | "격자게임보기.상태요약"
                | "격자게임보기.유령조각"
                | "격자게임보기.유령보개목록"
                | "격자게임보기.보개목록"
                | "격자게임보기.보개크기"
                | "물리1d.위치갱신"
                | "물리1d.속도갱신"
                | "물리1d.탄성충돌1d"
                | "무작위가방.만들기"
                | "무작위가방.꺼내기"
                | "무작위가방.미리보기"
                | "무작위가방.남은것"
                | "무작위가방.비었나"
                | "격자게임상태.초기화"
                | "격자게임상태.만들기"
                | "격자게임상태.상태"
                | "격자게임상태.틱"
                | "격자게임상태.상태인가"
                | "격자게임상태.바꾸기"
                | "격자게임상태.멈춤"
                | "격자게임상태.재개"
                | "기억하기"
                | "길이"
                | "끝나나"
                | "이음관계.관계목록"
                | "이음관계.정규화"
                | "이음관계.방정식목록"
                | "이음관계.방정식화"
                | "이음관계.값관계목록"
                | "이음관계.값주입"
                | "이음관계.풀기"
                | "이음관계.풀이값목록"
                | "이음관계.풀이원복"
                | "이음관계.범위위반목록"
                | "이음관계.범위검사"
                | "이음관계.풀고범위위반목록"
                | "이음관계.풀고범위검사"
                | "이음관계.풀고범위행목록"
                | "이음관계.풀고범위보고서"
                | "이음관계.보고서문자표"
                | "이음관계.풀고범위문자표"
                | "이음관계.풀고범위케이스"
                | "이음관계.풀고범위스위트"
                | "이음관계.풀고범위스위트문자표"
                | "이음관계.풀고범위스위트상세문자표"
                | "이음관계.풀고범위실행상세문자표"
                | "이음관계.풀고범위스위트요약"
                | "이음관계.풀고범위실행요약"
                | "이음관계.풀고범위스위트판정"
                | "이음관계.풀고범위실행판정"
                | "입력키"
                | "입력키?"
                | "입력키!"
                | "입력사상.만들기"
                | "입력사상.방향"
                | "입력사상.동작"
                | "눌렸나"
                | "다듬기"
                | "대문자로바꾸기"
                | "되풀이하기"
                | "뒤집기"
                | "들어있나"
                | "다음으로"
                | "또는"
                | "수"
                | "셈수"
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
                | "동치인가"
                | "잇기"
                | "인수분해하기"
                | "미분.중앙차분"
                | "수치해.이분법"
                | "다항식.풀기"
                | "선형부등식.풀기"
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
                | "적분.속도베를레"
                | "적분.사다리꼴"
                | "적분하기"
                | "전개하기"
                | "증명하기"
                | "정렬"
                | "정리하기"
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
                | "방정식풀기"
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
            if dialect == FormulaDialect::Ascii1 && !Self::is_ascii1_formula_ident(&binding.name) {
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

    fn is_ascii1_formula_ident(name: &str) -> bool {
        let mut chars = name.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        first.is_ascii_alphabetic() && chars.all(|ch| ch.is_ascii_digit())
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

    fn eval_relation_eq(
        &self,
        left: Value,
        right: Value,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let Value::Math(left_math) = left else {
            return Err(RuntimeError::TypeMismatch {
                expected: "formula",
                span,
            });
        };
        let Value::Math(right_math) = right else {
            return Err(RuntimeError::TypeMismatch {
                expected: "formula",
                span,
            });
        };
        Ok(make_relation_pack(left_math, right_math))
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
        if let Some(passed) = eval_symbolic_assertion_bridge(assertion, values, span)? {
            return self.record_assertion_check(assertion, values.fields.len(), passed, span);
        }

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

        self.record_assertion_check(assertion, values.fields.len(), passed, span)
    }

    fn record_assertion_check(
        &mut self,
        assertion: &AssertionValue,
        binding_count: usize,
        passed: bool,
        span: crate::lang::span::Span,
    ) -> Result<Value, RuntimeError> {
        let binding_count_value = Fixed64::from_int(binding_count as i64);
        let failure_code = (!passed).then(|| "E_PROOF_CHECK_FAILED".to_string());
        self.diagnostics.push(DiagnosticRecord {
            tick: self.current_madi.get(),
            name: format!("살피기 {}", assertion.canon),
            lhs: if passed { "1" } else { "0" }.to_string(),
            rhs: "1".to_string(),
            delta: if passed { "0" } else { "1" }.to_string(),
            threshold: binding_count_value.format(),
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
                threshold: binding_count_value.format(),
                span,
            });
        }
        self.proof_runtime.push(ProofRuntimeEvent::ProofCheck {
            tick: self.current_madi.get(),
            target: assertion.canon.clone(),
            binding_count: binding_count as u64,
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
        if let Some(value) = eval_exact_binary("+", &left, &right, span)? {
            return Ok(value);
        }
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
        if let Some(value) = eval_exact_binary("-", &left, &right, span)? {
            return Ok(value);
        }
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
        if let Some(value) = eval_exact_binary("*", &left, &right, span)? {
            return Ok(value);
        }
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
        if let Some(value) = eval_exact_binary("/", &left, &right, span)? {
            return Ok(value);
        }
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
        if let Some(value) = eval_exact_binary("%", &left, &right, span)? {
            return Ok(value);
        }
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
        let params = lambda_param_names(&lambda.param);
        if args.len() != params.len() {
            return Err(RuntimeError::TypeMismatch {
                expected: "lambda argument",
                span,
            });
        }
        let mut state = State {
            resources: lambda
                .captured
                .iter()
                .map(|(key, value)| (Key::new(key.clone()), value.clone()))
                .collect(),
        };
        for (param, arg) in params.iter().zip(args.iter()) {
            state.set(Key::new(param.clone()), arg.clone());
        }
        let mut evaluator = Evaluator::with_state_and_seed(state, self.rng_state.get());
        evaluator.user_seeds = self.user_seeds.clone();
        evaluator.import_aliases = self.import_aliases.clone();
        evaluator.current_madi.set(self.current_madi.get());
        let result = evaluator.eval_expr(&lambda.body)?;
        self.rng_state.set(evaluator.rng_state.get());
        Ok(result)
    }

    fn lambda_capture_snapshot(&self) -> BTreeMap<String, Value> {
        self.state
            .resources
            .iter()
            .filter_map(|(key, value)| match value {
                Value::Lambda(_) => None,
                _ => Some((key.as_str().to_string(), value.clone())),
            })
            .collect()
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
        self.enter_const_scope();
        let flow = self.eval_block(&seed.body);
        self.exit_const_scope();
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
            Ok(FlowControl::ContinueLoop(span)) => {
                if is_immediate_proof {
                    self.record_immediate_proof_diag(
                        seed_name,
                        "실패",
                        Some("E_RUNTIME_CONTINUE_OUTSIDE_FOREACH".to_string()),
                    );
                }
                Err(RuntimeError::ContinueOutsideForeach { span })
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
        if let Some(result) = eval_exact_compare(op, &left, &right, span)? {
            return Ok(Value::Bool(result));
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

fn fixed_value(value: i64) -> Value {
    Value::Num(Quantity::new(Fixed64::from_int(value), UnitDim::zero()))
}

const STREAM_V1_SCHEMA: &str = "ddn.stream.v1";
const STREAM_CAPACITY_MAX: usize = 1_000_000;

#[derive(Clone)]
struct RuntimeStream {
    capacity: usize,
    head: usize,
    len: usize,
    buffer: Vec<Value>,
}

fn stream_key(key: &str) -> Value {
    Value::Str(key.to_string())
}

fn stream_insert(entries: &mut BTreeMap<String, MapEntry>, key: &str, value: Value) {
    let key_value = stream_key(key);
    entries.insert(
        key_value.canon(),
        MapEntry {
            key: key_value,
            value,
        },
    );
}

fn stream_get(map: &MapValue, key: &str) -> Value {
    map.map_get(&stream_key(key))
}

fn stream_parse_usize(value: &Value, span: crate::lang::span::Span) -> Result<usize, RuntimeError> {
    let raw = expect_int(value, span)?;
    if raw < 0 {
        return Err(RuntimeError::TypeMismatch {
            expected: "non-negative integer",
            span,
        });
    }
    usize::try_from(raw).map_err(|_| RuntimeError::TypeMismatch {
        expected: "integer range",
        span,
    })
}

fn stream_new(capacity: usize, span: crate::lang::span::Span) -> Result<RuntimeStream, RuntimeError> {
    if capacity == 0 || capacity > STREAM_CAPACITY_MAX {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream capacity in 1..=1000000",
            span,
        });
    }
    Ok(RuntimeStream {
        capacity,
        head: 0,
        len: 0,
        buffer: vec![Value::None; capacity],
    })
}

fn stream_from_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<RuntimeStream, RuntimeError> {
    let Value::Map(map) = value else {
        return Err(type_mismatch_detail("stream", value, span));
    };
    match stream_get(map, "__schema") {
        Value::Str(schema) if schema == STREAM_V1_SCHEMA => {}
        other => return Err(type_mismatch_detail("ddn.stream.v1", &other, span)),
    }
    let capacity = match stream_get(map, "capacity") {
        Value::None => 0,
        value => stream_parse_usize(&value, span)?,
    };
    let head = match stream_get(map, "head") {
        Value::None => None,
        value => Some(stream_parse_usize(&value, span)?),
    };
    let buffer_raw = match stream_get(map, "buffer") {
        Value::List(list) => list.items,
        other => return Err(type_mismatch_detail("stream buffer", &other, span)),
    };
    let len_raw = match stream_get(map, "len") {
        Value::None => buffer_raw.len(),
        value => stream_parse_usize(&value, span)?,
    };
    let capacity = capacity.max(buffer_raw.len());
    if capacity == 0 || capacity > STREAM_CAPACITY_MAX {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream capacity in 1..=1000000",
            span,
        });
    }
    let mut buffer = buffer_raw;
    if buffer.len() < capacity {
        buffer.resize(capacity, Value::None);
    } else if buffer.len() > capacity {
        buffer.truncate(capacity);
    }
    let len = len_raw.min(capacity);
    let head = if len == 0 {
        0
    } else {
        head.unwrap_or_else(|| len.saturating_sub(1))
            .min(capacity.saturating_sub(1))
    };
    Ok(RuntimeStream {
        capacity,
        head,
        len,
        buffer,
    })
}

fn stream_to_value(stream: &RuntimeStream) -> Value {
    let mut entries = BTreeMap::new();
    stream_insert(
        &mut entries,
        "__schema",
        Value::Str(STREAM_V1_SCHEMA.to_string()),
    );
    stream_insert(
        &mut entries,
        "capacity",
        fixed_value(stream.capacity.min(i64::MAX as usize) as i64),
    );
    stream_insert(
        &mut entries,
        "head",
        fixed_value(stream.head.min(i64::MAX as usize) as i64),
    );
    stream_insert(
        &mut entries,
        "len",
        fixed_value(stream.len.min(i64::MAX as usize) as i64),
    );
    stream_insert(
        &mut entries,
        "buffer",
        Value::List(ListValue {
            items: stream.buffer.clone(),
        }),
    );
    Value::Map(MapValue { entries })
}

fn stream_push(mut stream: RuntimeStream, value: Value) -> RuntimeStream {
    let next_head = if stream.len == 0 {
        0
    } else {
        (stream.head + 1) % stream.capacity
    };
    if stream.buffer.len() < stream.capacity {
        stream.buffer.resize(stream.capacity, Value::None);
    }
    stream.buffer[next_head] = value;
    stream.head = next_head;
    stream.len = stream.len.saturating_add(1).min(stream.capacity);
    stream
}

fn stream_clear(mut stream: RuntimeStream) -> RuntimeStream {
    if stream.buffer.len() < stream.capacity {
        stream.buffer.resize(stream.capacity, Value::None);
    }
    for slot in &mut stream.buffer {
        *slot = Value::None;
    }
    stream.head = 0;
    stream.len = 0;
    stream
}

fn stream_oldest_to_newest(stream: &RuntimeStream) -> Vec<Value> {
    if stream.len == 0 {
        return Vec::new();
    }
    let oldest = (stream.head + stream.capacity + 1 - stream.len) % stream.capacity;
    let mut out = Vec::with_capacity(stream.len);
    for offset in 0..stream.len {
        let idx = (oldest + offset) % stream.capacity;
        out.push(stream.buffer.get(idx).cloned().unwrap_or(Value::None));
    }
    out
}

fn eval_stream_new(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 && values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "capacity[, initial_list]",
            span,
        });
    }
    let capacity = stream_parse_usize(&values[0], span)?;
    let mut stream = stream_new(capacity, span)?;
    if values.len() == 2 {
        let Value::List(initial) = &values[1] else {
            return Err(type_mismatch_detail("list", &values[1], span));
        };
        for item in &initial.items {
            stream = stream_push(stream, item.clone());
        }
    }
    Ok(stream_to_value(&stream))
}

fn eval_stream_push(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream, value",
            span,
        });
    }
    let stream = stream_from_value(&values[0], span)?;
    Ok(stream_to_value(&stream_push(stream, values[1].clone())))
}

fn eval_stream_items(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream",
            span,
        });
    }
    let stream = stream_from_value(&values[0], span)?;
    Ok(Value::List(ListValue {
        items: stream_oldest_to_newest(&stream),
    }))
}

fn eval_stream_latest(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream",
            span,
        });
    }
    let stream = stream_from_value(&values[0], span)?;
    if stream.len == 0 {
        Ok(Value::None)
    } else {
        Ok(stream.buffer.get(stream.head).cloned().unwrap_or(Value::None))
    }
}

fn eval_stream_len(values: &[Value], span: crate::lang::span::Span) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream",
            span,
        });
    }
    let stream = stream_from_value(&values[0], span)?;
    Ok(fixed_value(stream.len.min(i64::MAX as usize) as i64))
}

fn eval_stream_capacity(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream",
            span,
        });
    }
    let stream = stream_from_value(&values[0], span)?;
    Ok(fixed_value(stream.capacity.min(i64::MAX as usize) as i64))
}

fn eval_stream_clear(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream",
            span,
        });
    }
    let stream = stream_from_value(&values[0], span)?;
    Ok(stream_to_value(&stream_clear(stream)))
}

fn eval_stream_tail(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "stream, count",
            span,
        });
    }
    let stream = stream_from_value(&values[0], span)?;
    let count = stream_parse_usize(&values[1], span)?;
    let ordered = stream_oldest_to_newest(&stream);
    let take = count.min(ordered.len());
    let start = ordered.len().saturating_sub(take);
    Ok(Value::List(ListValue {
        items: ordered[start..].to_vec(),
    }))
}

fn make_std_grid(
    width: i64,
    height: i64,
    default_value: Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if width <= 0 || height <= 0 {
        return Err(RuntimeError::TypeMismatch {
            expected: "positive grid size",
            span,
        });
    }
    let cell_count = width
        .checked_mul(height)
        .ok_or(RuntimeError::TypeMismatch {
            expected: "grid size within range",
            span,
        })?;
    let mut fields = BTreeMap::new();
    fields.insert("__kind".to_string(), Value::Str(STD_GRID_KIND.to_string()));
    fields.insert("너비".to_string(), fixed_value(width));
    fields.insert("높이".to_string(), fixed_value(height));
    fields.insert(
        "칸들".to_string(),
        Value::List(ListValue {
            items: vec![default_value; cell_count as usize],
        }),
    );
    Ok(Value::Pack(PackValue { fields }))
}

fn std_grid_fields<'a>(
    value: &'a Value,
    span: crate::lang::span::Span,
) -> Result<&'a BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("격자", value, span));
    };
    match pack.fields.get("__kind") {
        Some(Value::Str(kind)) if kind == STD_GRID_KIND => Ok(&pack.fields),
        _ => Err(type_mismatch_detail("격자", value, span)),
    }
}

fn std_grid_dims(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<(i64, i64), RuntimeError> {
    let width = fields
        .get("너비")
        .ok_or(RuntimeError::TypeMismatch {
            expected: "grid width",
            span,
        })
        .and_then(|value| quantity_to_int(&expect_quantity_value(value, span)?, span))?;
    let height = fields
        .get("높이")
        .ok_or(RuntimeError::TypeMismatch {
            expected: "grid height",
            span,
        })
        .and_then(|value| quantity_to_int(&expect_quantity_value(value, span)?, span))?;
    if width <= 0 || height <= 0 {
        return Err(RuntimeError::TypeMismatch {
            expected: "positive grid size",
            span,
        });
    }
    Ok((width, height))
}

fn std_grid_cells<'a>(
    fields: &'a BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<&'a Vec<Value>, RuntimeError> {
    let Some(Value::List(cells)) = fields.get("칸들") else {
        return Err(RuntimeError::TypeMismatch {
            expected: "grid cells",
            span,
        });
    };
    Ok(&cells.items)
}

fn std_grid_index(
    fields: &BTreeMap<String, Value>,
    x: i64,
    y: i64,
    span: crate::lang::span::Span,
) -> Result<usize, RuntimeError> {
    let (width, height) = std_grid_dims(fields, span)?;
    if x < 0 || y < 0 || x >= width || y >= height {
        return Err(RuntimeError::Pack {
            message: "격자 좌표가 범위를 벗어났습니다".to_string(),
            span,
        });
    }
    Ok((y * width + x) as usize)
}

fn std_grid_inside(
    value: &Value,
    x: i64,
    y: i64,
    span: crate::lang::span::Span,
) -> Result<bool, RuntimeError> {
    let fields = std_grid_fields(value, span)?;
    let (width, height) = std_grid_dims(fields, span)?;
    Ok(x >= 0 && y >= 0 && x < width && y < height)
}

fn std_block_piece_cell_from_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<(i64, i64), RuntimeError> {
    let Value::List(list) = value else {
        return Err(type_mismatch_detail("block piece cell", value, span));
    };
    if list.items.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "cell [x, y]",
            span,
        });
    }
    let x = quantity_to_int(&expect_quantity_value(&list.items[0], span)?, span)?;
    let y = quantity_to_int(&expect_quantity_value(&list.items[1], span)?, span)?;
    Ok((x, y))
}

fn sort_std_block_piece_cells(cells: &mut Vec<(i64, i64)>) {
    cells.sort_by_key(|(x, y)| (*y, *x));
}

fn std_block_piece_cells_value(cells: &[(i64, i64)]) -> Value {
    Value::List(ListValue {
        items: cells
            .iter()
            .map(|(x, y)| {
                Value::List(ListValue {
                    items: vec![fixed_value(*x), fixed_value(*y)],
                })
            })
            .collect(),
    })
}

fn std_block_piece_cells_from_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Vec<(i64, i64)>, RuntimeError> {
    let Value::List(list) = value else {
        return Err(type_mismatch_detail("block piece cells", value, span));
    };
    let mut cells = list
        .items
        .iter()
        .map(|item| std_block_piece_cell_from_value(item, span))
        .collect::<Result<Vec<_>, _>>()?;
    sort_std_block_piece_cells(&mut cells);
    Ok(cells)
}

fn make_std_block_piece(mut cells: Vec<(i64, i64)>) -> Value {
    sort_std_block_piece_cells(&mut cells);
    let mut fields = BTreeMap::new();
    fields.insert(
        STD_BLOCK_PIECE_KIND_FIELD.to_string(),
        Value::Str(STD_BLOCK_PIECE_KIND.to_string()),
    );
    fields.insert(
        STD_BLOCK_PIECE_CELLS_FIELD.to_string(),
        std_block_piece_cells_value(&cells),
    );
    Value::Pack(PackValue { fields })
}

fn std_block_piece_cells(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Vec<(i64, i64)>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("block piece", value, span));
    };
    match pack.fields.get(STD_BLOCK_PIECE_KIND_FIELD) {
        Some(Value::Str(kind)) if kind == STD_BLOCK_PIECE_KIND => {}
        _ => return Err(type_mismatch_detail("block piece", value, span)),
    }
    let cells = pack
        .fields
        .get(STD_BLOCK_PIECE_CELLS_FIELD)
        .ok_or(RuntimeError::TypeMismatch {
            expected: "block piece cells",
            span,
        })?;
    std_block_piece_cells_from_value(cells, span)
}

fn std_block_piece_collides(
    piece: &Value,
    grid: &Value,
    blocked_values: &Value,
    span: crate::lang::span::Span,
) -> Result<bool, RuntimeError> {
    let cells = std_block_piece_cells(piece, span)?;
    for (x, y) in cells {
        if !std_grid_inside(grid, x, y, span)? {
            return Ok(true);
        }
        let fields = std_grid_fields(grid, span)?;
        let idx = std_grid_index(fields, x, y, span)?;
        let cell = std_grid_cells(fields, span)?
            .get(idx)
            .cloned()
            .unwrap_or(Value::None);
        let blocked = match blocked_values {
            Value::List(list) => list.items.iter().any(|item| item == &cell),
            other => other == &cell,
        };
        if blocked {
            return Ok(true);
        }
    }
    Ok(false)
}

fn std_block_piece_lock(
    piece: &Value,
    grid: &Value,
    value: Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let cells = std_block_piece_cells(piece, span)?;
    let fields = std_grid_fields(grid, span)?;
    let mut next_fields = fields.clone();
    let mut grid_cells = std_grid_cells(fields, span)?.clone();
    for (x, y) in cells {
        let idx = std_grid_index(fields, x, y, span)?;
        if idx >= grid_cells.len() {
            return Err(RuntimeError::TypeMismatch {
                expected: "grid cells matching size",
                span,
            });
        }
        grid_cells[idx] = value.clone();
    }
    next_fields.insert(
        "칸들".to_string(),
        Value::List(ListValue { items: grid_cells }),
    );
    Ok(Value::Pack(PackValue {
        fields: next_fields,
    }))
}

fn value_to_seed_u64(value: &Value, span: crate::lang::span::Span) -> Result<u64, RuntimeError> {
    let seed = quantity_to_int(&expect_quantity_value(value, span)?, span)?;
    Ok(seed as u64)
}

fn u64_state_text(value: u64) -> String {
    format!("0x{value:016x}")
}

fn parse_u64_state_text(text: &str, span: crate::lang::span::Span) -> Result<u64, RuntimeError> {
    if let Some(hex) = text.strip_prefix("0x") {
        return u64::from_str_radix(hex, 16).map_err(|_| RuntimeError::TypeMismatch {
            expected: "u64 state",
            span,
        });
    }
    text.parse::<u64>().map_err(|_| RuntimeError::TypeMismatch {
        expected: "u64 state",
        span,
    })
}

fn make_std_random_bag(
    seed: u64,
    state: u64,
    original: Vec<Value>,
    remaining: Vec<Value>,
    draws: i64,
) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        STD_RANDOM_BAG_KIND_FIELD.to_string(),
        Value::Str(STD_RANDOM_BAG_KIND.to_string()),
    );
    fields.insert(
        STD_RANDOM_BAG_SEED_FIELD.to_string(),
        fixed_value(seed as i64),
    );
    fields.insert(
        STD_RANDOM_BAG_STATE_FIELD.to_string(),
        Value::Str(u64_state_text(state)),
    );
    fields.insert(
        STD_RANDOM_BAG_ORIGINAL_FIELD.to_string(),
        Value::List(ListValue { items: original }),
    );
    fields.insert(
        STD_RANDOM_BAG_REMAINING_FIELD.to_string(),
        Value::List(ListValue { items: remaining }),
    );
    fields.insert(STD_RANDOM_BAG_DRAWS_FIELD.to_string(), fixed_value(draws));
    Value::Pack(PackValue { fields })
}

fn std_random_bag_fields<'a>(
    value: &'a Value,
    span: crate::lang::span::Span,
) -> Result<&'a BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("random bag", value, span));
    };
    match pack.fields.get(STD_RANDOM_BAG_KIND_FIELD) {
        Some(Value::Str(kind)) if kind == STD_RANDOM_BAG_KIND => Ok(&pack.fields),
        _ => Err(type_mismatch_detail("random bag", value, span)),
    }
}

fn std_random_bag_list_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
    span: crate::lang::span::Span,
) -> Result<Vec<Value>, RuntimeError> {
    let Some(Value::List(list)) = fields.get(field) else {
        return Err(RuntimeError::TypeMismatch {
            expected: "random bag list field",
            span,
        });
    };
    Ok(list.items.clone())
}

fn std_random_bag_original(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<Vec<Value>, RuntimeError> {
    std_random_bag_list_field(fields, STD_RANDOM_BAG_ORIGINAL_FIELD, span)
}

fn std_random_bag_remaining(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<Vec<Value>, RuntimeError> {
    std_random_bag_list_field(fields, STD_RANDOM_BAG_REMAINING_FIELD, span)
}

fn std_random_bag_state(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<u64, RuntimeError> {
    match fields.get(STD_RANDOM_BAG_STATE_FIELD) {
        Some(Value::Str(state)) => parse_u64_state_text(state, span),
        Some(Value::Num(qty)) => Ok(quantity_to_int(qty, span)? as u64),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "random bag state",
            span,
        }),
    }
}

fn std_random_bag_seed(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<u64, RuntimeError> {
    match fields.get(STD_RANDOM_BAG_SEED_FIELD) {
        Some(value) => value_to_seed_u64(value, span),
        None => Err(RuntimeError::TypeMismatch {
            expected: "random bag seed",
            span,
        }),
    }
}

fn std_random_bag_draws(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<i64, RuntimeError> {
    match fields.get(STD_RANDOM_BAG_DRAWS_FIELD) {
        Some(Value::Num(qty)) => quantity_to_int(qty, span),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "random bag draw count",
            span,
        }),
    }
}

fn std_random_bag_draw_once(
    bag: &Value,
    span: crate::lang::span::Span,
) -> Result<(Value, Value), RuntimeError> {
    let fields = std_random_bag_fields(bag, span)?;
    let seed = std_random_bag_seed(fields, span)?;
    let original = std_random_bag_original(fields, span)?;
    if original.is_empty() {
        return Err(RuntimeError::Pack {
            message: "무작위가방 원본 후보들은 비어 있을 수 없습니다".to_string(),
            span,
        });
    }
    let mut remaining = std_random_bag_remaining(fields, span)?;
    if remaining.is_empty() {
        remaining = original.clone();
    }
    let (next_state, sample) = splitmix64_next(std_random_bag_state(fields, span)?);
    let idx = (sample % remaining.len() as u64) as usize;
    let value = remaining.remove(idx);
    let next_bag = make_std_random_bag(
        seed,
        next_state,
        original,
        remaining,
        std_random_bag_draws(fields, span)?.saturating_add(1),
    );
    Ok((value, next_bag))
}

fn is_allowed_grid_game_state(state: &str) -> bool {
    matches!(state, "준비" | "진행" | "잠금지연" | "정리" | "끝" | "멈춤")
}

fn expect_grid_game_state_name(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let Value::Str(state) = value else {
        return Err(type_mismatch_detail("grid game state name", value, span));
    };
    if !is_allowed_grid_game_state(state) {
        return Err(RuntimeError::Pack {
            message: format!("격자게임상태를 지원하지 않습니다: {state}"),
            span,
        });
    }
    Ok(state.clone())
}

fn make_std_grid_game_state(state: &str, previous: Value, tick: i64) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        STD_GRID_GAME_STATE_KIND_FIELD.to_string(),
        Value::Str(STD_GRID_GAME_STATE_KIND.to_string()),
    );
    fields.insert(
        STD_GRID_GAME_STATE_STATE_FIELD.to_string(),
        Value::Str(state.to_string()),
    );
    fields.insert(STD_GRID_GAME_STATE_PREV_FIELD.to_string(), previous);
    fields.insert(
        STD_GRID_GAME_STATE_TICK_FIELD.to_string(),
        fixed_value(tick),
    );
    Value::Pack(PackValue { fields })
}

fn std_grid_game_state_fields<'a>(
    value: &'a Value,
    span: crate::lang::span::Span,
) -> Result<&'a BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("grid game state", value, span));
    };
    match pack.fields.get(STD_GRID_GAME_STATE_KIND_FIELD) {
        Some(Value::Str(kind)) if kind == STD_GRID_GAME_STATE_KIND => Ok(&pack.fields),
        _ => Err(type_mismatch_detail("grid game state", value, span)),
    }
}

fn std_grid_game_state_state(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let Some(Value::Str(state)) = fields.get(STD_GRID_GAME_STATE_STATE_FIELD) else {
        return Err(RuntimeError::TypeMismatch {
            expected: "grid game state",
            span,
        });
    };
    if !is_allowed_grid_game_state(state) {
        return Err(RuntimeError::Pack {
            message: format!("격자게임상태를 지원하지 않습니다: {state}"),
            span,
        });
    }
    Ok(state.clone())
}

fn std_grid_game_state_previous(fields: &BTreeMap<String, Value>) -> Value {
    fields
        .get(STD_GRID_GAME_STATE_PREV_FIELD)
        .cloned()
        .unwrap_or(Value::None)
}

fn std_grid_game_state_tick(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<i64, RuntimeError> {
    let Some(Value::Num(qty)) = fields.get(STD_GRID_GAME_STATE_TICK_FIELD) else {
        return Err(RuntimeError::TypeMismatch {
            expected: "grid game state tick",
            span,
        });
    };
    quantity_to_int(qty, span)
}

fn std_grid_game_error(message: impl Into<String>, span: crate::lang::span::Span) -> RuntimeError {
    RuntimeError::Pack {
        message: message.into(),
        span,
    }
}

fn tetromino_names() -> [&'static str; 7] {
    ["I", "O", "T", "S", "Z", "J", "L"]
}

fn tetromino_cells(
    name: &str,
    span: crate::lang::span::Span,
) -> Result<Vec<(i64, i64)>, RuntimeError> {
    let cells = match name {
        "I" => vec![(-1, 0), (0, 0), (1, 0), (2, 0)],
        "O" => vec![(0, 0), (1, 0), (0, 1), (1, 1)],
        "T" => vec![(-1, 0), (0, 0), (1, 0), (0, 1)],
        "S" => vec![(0, 0), (1, 0), (-1, 1), (0, 1)],
        "Z" => vec![(-1, 0), (0, 0), (0, 1), (1, 1)],
        "J" => vec![(-1, 0), (-1, 1), (0, 1), (1, 1)],
        "L" => vec![(1, 0), (-1, 1), (0, 1), (1, 1)],
        _ => {
            return Err(std_grid_game_error(
                format!("테트로미노를 지원하지 않습니다: {name}"),
                span,
            ))
        }
    };
    Ok(cells)
}

fn value_to_tetromino_name(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let Value::Str(name) = value else {
        return Err(type_mismatch_detail("tetromino name", value, span));
    };
    let _ = tetromino_cells(name, span)?;
    Ok(name.clone())
}

fn std_grid_line_full_rows(
    grid: &Value,
    empty_value: &Value,
    span: crate::lang::span::Span,
) -> Result<Vec<i64>, RuntimeError> {
    let fields = std_grid_fields(grid, span)?;
    let (width, height) = std_grid_dims(fields, span)?;
    let cells = std_grid_cells(fields, span)?;
    let mut rows = Vec::new();
    for y in 0..height {
        let mut full = true;
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let cell = cells.get(idx).ok_or(RuntimeError::TypeMismatch {
                expected: "grid cells matching size",
                span,
            })?;
            if cell == empty_value {
                full = false;
                break;
            }
        }
        if full {
            rows.push(y);
        }
    }
    Ok(rows)
}

fn std_grid_line_clear(
    grid: &Value,
    empty_value: Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let fields = std_grid_fields(grid, span)?;
    let (width, height) = std_grid_dims(fields, span)?;
    let cells = std_grid_cells(fields, span)?;
    let full_rows = std_grid_line_full_rows(grid, &empty_value, span)?;
    let full_set: BTreeSet<i64> = full_rows.iter().copied().collect();
    let mut kept_rows = Vec::new();
    for y in 0..height {
        if full_set.contains(&y) {
            continue;
        }
        let start = (y * width) as usize;
        let end = start + width as usize;
        kept_rows.extend_from_slice(cells.get(start..end).ok_or(RuntimeError::TypeMismatch {
            expected: "grid cells matching size",
            span,
        })?);
    }
    let mut next_cells = vec![empty_value.clone(); full_rows.len() * width as usize];
    next_cells.extend(kept_rows);
    let mut next_fields = fields.clone();
    next_fields.insert(
        "칸들".to_string(),
        Value::List(ListValue { items: next_cells }),
    );
    let mut result = BTreeMap::new();
    result.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_LINE_CLEAR_KIND.to_string()),
    );
    result.insert(
        "격자".to_string(),
        Value::Pack(PackValue {
            fields: next_fields,
        }),
    );
    result.insert("지운줄수".to_string(), fixed_value(full_rows.len() as i64));
    result.insert(
        "지운줄목록".to_string(),
        Value::List(ListValue {
            items: full_rows.into_iter().map(fixed_value).collect(),
        }),
    );
    Ok(Value::Pack(PackValue { fields: result }))
}

fn make_std_falling_piece(piece: Value, x: i64, y: i64) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str(STD_FALLING_PIECE_KIND.to_string()),
    );
    fields.insert("조각".to_string(), piece);
    fields.insert("x".to_string(), fixed_value(x));
    fields.insert("y".to_string(), fixed_value(y));
    Value::Pack(PackValue { fields })
}

fn std_falling_piece_fields<'a>(
    value: &'a Value,
    span: crate::lang::span::Span,
) -> Result<&'a BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("falling piece", value, span));
    };
    match pack.fields.get("__종류") {
        Some(Value::Str(kind)) if kind == STD_FALLING_PIECE_KIND => Ok(&pack.fields),
        _ => Err(type_mismatch_detail("falling piece", value, span)),
    }
}

fn std_falling_piece_parts(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<(Value, i64, i64), RuntimeError> {
    let fields = std_falling_piece_fields(value, span)?;
    let piece = fields
        .get("조각")
        .cloned()
        .ok_or(RuntimeError::TypeMismatch {
            expected: "falling piece block",
            span,
        })?;
    let x = fields
        .get("x")
        .ok_or(RuntimeError::TypeMismatch {
            expected: "falling piece x",
            span,
        })
        .and_then(|value| quantity_to_int(&expect_quantity_value(value, span)?, span))?;
    let y = fields
        .get("y")
        .ok_or(RuntimeError::TypeMismatch {
            expected: "falling piece y",
            span,
        })
        .and_then(|value| quantity_to_int(&expect_quantity_value(value, span)?, span))?;
    Ok((piece, x, y))
}

fn std_falling_piece_cells(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Vec<(i64, i64)>, RuntimeError> {
    let (piece, x, y) = std_falling_piece_parts(value, span)?;
    Ok(std_block_piece_cells(&piece, span)?
        .into_iter()
        .map(|(cx, cy)| (cx + x, cy + y))
        .collect())
}

fn std_falling_piece_block(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    Ok(make_std_block_piece(std_falling_piece_cells(value, span)?))
}

fn std_falling_piece_rotate(
    value: &Value,
    direction: &str,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let (piece, x, y) = std_falling_piece_parts(value, span)?;
    let rotated_cells = std_block_piece_cells(&piece, span)?
        .into_iter()
        .map(|(cx, cy)| match direction {
            "오른쪽" => Ok((-cy, cx)),
            "왼쪽" => Ok((cy, -cx)),
            "뒤집기" => Ok((-cx, -cy)),
            _ => Err(std_grid_game_error(
                format!("낙하조각 회전 방향을 지원하지 않습니다: {direction}"),
                span,
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(make_std_falling_piece(
        make_std_block_piece(rotated_cells),
        x,
        y,
    ))
}

fn make_std_grid_game_score(score: i64, lines: i64) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_GAME_SCORE_KIND.to_string()),
    );
    fields.insert("점수".to_string(), fixed_value(score));
    fields.insert("줄수".to_string(), fixed_value(lines));
    fields.insert("레벨".to_string(), fixed_value(1 + lines.div_euclid(10)));
    Value::Pack(PackValue { fields })
}

fn std_grid_game_score_fields<'a>(
    value: &'a Value,
    span: crate::lang::span::Span,
) -> Result<&'a BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("grid game score", value, span));
    };
    match pack.fields.get("__종류") {
        Some(Value::Str(kind)) if kind == STD_GRID_GAME_SCORE_KIND => Ok(&pack.fields),
        _ => Err(type_mismatch_detail("grid game score", value, span)),
    }
}

fn std_score_int_field(
    fields: &BTreeMap<String, Value>,
    field: &'static str,
    span: crate::lang::span::Span,
) -> Result<i64, RuntimeError> {
    fields
        .get(field)
        .ok_or(RuntimeError::TypeMismatch {
            expected: field,
            span,
        })
        .and_then(|value| quantity_to_int(&expect_quantity_value(value, span)?, span))
}

fn std_grid_game_score_add(
    score: &Value,
    cleared: i64,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if !(0..=4).contains(&cleared) {
        return Err(std_grid_game_error("지운줄수는 0..4 범위여야 합니다", span));
    }
    let fields = std_grid_game_score_fields(score, span)?;
    let current_score = std_score_int_field(fields, "점수", span)?;
    let current_lines = std_score_int_field(fields, "줄수", span)?;
    let level = 1 + current_lines.div_euclid(10);
    let add = match cleared {
        0 => 0,
        1 => 100 * level,
        2 => 300 * level,
        3 => 500 * level,
        4 => 800 * level,
        _ => unreachable!(),
    };
    Ok(make_std_grid_game_score(
        current_score.saturating_add(add),
        current_lines.saturating_add(cleared),
    ))
}

fn make_std_grid_game_session(
    grid: Value,
    bag: Value,
    state: Value,
    score: Value,
    falling: Value,
) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_GAME_SESSION_KIND.to_string()),
    );
    fields.insert("격자".to_string(), grid);
    fields.insert("가방".to_string(), bag);
    fields.insert("상태".to_string(), state);
    fields.insert("점수".to_string(), score);
    fields.insert("낙하조각".to_string(), falling);
    Value::Pack(PackValue { fields })
}

fn std_grid_game_session_fields<'a>(
    value: &'a Value,
    span: crate::lang::span::Span,
) -> Result<&'a BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("grid game session", value, span));
    };
    match pack.fields.get("__종류") {
        Some(Value::Str(kind)) if kind == STD_GRID_GAME_SESSION_KIND => Ok(&pack.fields),
        _ => Err(type_mismatch_detail("grid game session", value, span)),
    }
}

fn std_session_field(
    fields: &BTreeMap<String, Value>,
    field: &'static str,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    fields
        .get(field)
        .cloned()
        .ok_or(RuntimeError::TypeMismatch {
            expected: field,
            span,
        })
}

fn std_grid_game_next_piece(
    bag: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let (name_value, next_bag) = std_random_bag_draw_once(bag, span)?;
    let name = value_to_tetromino_name(&name_value, span)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_GAME_NEXT_PIECE_KIND.to_string()),
    );
    fields.insert("이름".to_string(), Value::Str(name.clone()));
    fields.insert(
        "조각".to_string(),
        make_std_block_piece(tetromino_cells(&name, span)?),
    );
    fields.insert("가방".to_string(), next_bag);
    Ok(Value::Pack(PackValue { fields }))
}

fn make_std_grid_game_hold(piece: Value, used: bool) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_GAME_HOLD_KIND.to_string()),
    );
    fields.insert("조각".to_string(), piece);
    fields.insert("썼나".to_string(), Value::Bool(used));
    Value::Pack(PackValue { fields })
}

fn std_grid_game_hold_fields<'a>(
    value: &'a Value,
    span: crate::lang::span::Span,
) -> Result<&'a BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("grid game hold", value, span));
    };
    match pack.fields.get("__종류") {
        Some(Value::Str(kind)) if kind == STD_GRID_GAME_HOLD_KIND => Ok(&pack.fields),
        _ => Err(type_mismatch_detail("grid game hold", value, span)),
    }
}

fn std_grid_game_hold_piece(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let piece = fields.get("조각").cloned().unwrap_or(Value::None);
    if !matches!(piece, Value::None) {
        let _ = std_block_piece_cells(&piece, span)?;
    }
    Ok(piece)
}

fn std_grid_game_hold_used(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<bool, RuntimeError> {
    match fields.get("썼나") {
        Some(Value::Bool(flag)) => Ok(*flag),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "grid game hold used flag",
            span,
        }),
    }
}

fn make_std_grid_game_hold_swap(hold: Value, falling: Value, bag: Value) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_GAME_HOLD_SWAP_KIND.to_string()),
    );
    fields.insert("홀드".to_string(), hold);
    fields.insert("낙하조각".to_string(), falling);
    fields.insert("가방".to_string(), bag);
    Value::Pack(PackValue { fields })
}

fn std_grid_game_hold_swap(
    hold: &Value,
    falling: &Value,
    bag: &Value,
    x: i64,
    y: i64,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let hold_fields = std_grid_game_hold_fields(hold, span)?;
    if std_grid_game_hold_used(hold_fields, span)? {
        return Err(std_grid_game_error(
            "격자게임홀드는 이번 턴에 이미 썼습니다",
            span,
        ));
    }
    let (current_piece, _, _) = std_falling_piece_parts(falling, span)?;
    let held_piece = std_grid_game_hold_piece(hold_fields, span)?;
    if matches!(held_piece, Value::None) {
        let next = std_grid_game_next_piece(bag, span)?;
        let Value::Pack(next_pack) = &next else {
            unreachable!()
        };
        let next_piece = std_session_field(&next_pack.fields, "조각", span)?;
        let next_bag = std_session_field(&next_pack.fields, "가방", span)?;
        Ok(make_std_grid_game_hold_swap(
            make_std_grid_game_hold(current_piece, true),
            make_std_falling_piece(next_piece, x, y),
            next_bag,
        ))
    } else {
        Ok(make_std_grid_game_hold_swap(
            make_std_grid_game_hold(current_piece, true),
            make_std_falling_piece(held_piece, x, y),
            bag.clone(),
        ))
    }
}

fn std_grid_game_placeable(
    falling: &Value,
    grid: &Value,
    blocked: &Value,
    span: crate::lang::span::Span,
) -> Result<bool, RuntimeError> {
    std_block_piece_collides(
        &std_falling_piece_block(falling, span)?,
        grid,
        blocked,
        span,
    )
    .map(|v| !v)
}

fn std_grid_game_rotation_try(
    falling: &Value,
    grid: &Value,
    blocked: &Value,
    direction: &str,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let rotated = std_falling_piece_rotate(falling, direction, span)?;
    let (piece, x, y) = std_falling_piece_parts(&rotated, span)?;
    for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (-2, 0), (2, 0), (0, -1)] {
        let candidate = make_std_falling_piece(piece.clone(), x + dx, y + dy);
        if std_grid_game_placeable(&candidate, grid, blocked, span)? {
            return Ok(make_std_grid_game_rotation_try(candidate, true, dx, dy));
        }
    }
    Ok(make_std_grid_game_rotation_try(
        falling.clone(),
        false,
        0,
        0,
    ))
}

fn make_std_grid_game_rotation_try(falling: Value, success: bool, dx: i64, dy: i64) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_GAME_ROTATION_TRY_KIND.to_string()),
    );
    fields.insert("낙하조각".to_string(), falling);
    fields.insert("성공".to_string(), Value::Bool(success));
    fields.insert(
        "오프셋".to_string(),
        Value::List(ListValue {
            items: vec![fixed_value(dx), fixed_value(dy)],
        }),
    );
    Value::Pack(PackValue { fields })
}

fn std_grid_game_apply_input(
    falling: &Value,
    input_map: &Value,
    state_store: &State,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let input_fields = std_input_map_fields(input_map, span)?;
    let mut candidate = falling.clone();
    if input_map_action_pressed(state_store, &input_fields, "왼쪽") {
        let (piece, x, y) = std_falling_piece_parts(&candidate, span)?;
        candidate = make_std_falling_piece(piece, x - 1, y);
    }
    if input_map_action_pressed(state_store, &input_fields, "오른쪽") {
        let (piece, x, y) = std_falling_piece_parts(&candidate, span)?;
        candidate = make_std_falling_piece(piece, x + 1, y);
    }
    if input_map_action_pressed(state_store, &input_fields, "아래") {
        let (piece, x, y) = std_falling_piece_parts(&candidate, span)?;
        candidate = make_std_falling_piece(piece, x, y + 1);
    }
    if input_map_action_pressed(state_store, &input_fields, "위")
        || input_map_action_pressed(state_store, &input_fields, "회전")
    {
        let (piece, x, y) = std_falling_piece_parts(&candidate, span)?;
        let rotated_cells = std_block_piece_cells(&piece, span)?
            .into_iter()
            .map(|(cx, cy)| (-cy, cx))
            .collect();
        candidate = make_std_falling_piece(make_std_block_piece(rotated_cells), x, y);
    }
    Ok(candidate)
}

fn std_grid_game_tick(
    session: &Value,
    input_map: &Value,
    state_store: &State,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let fields = std_grid_game_session_fields(session, span)?;
    let grid = std_session_field(fields, "격자", span)?;
    let bag = std_session_field(fields, "가방", span)?;
    let state = std_session_field(fields, "상태", span)?;
    let score = std_session_field(fields, "점수", span)?;
    let falling = std_session_field(fields, "낙하조각", span)?;
    let state_name = std_grid_game_state_state(std_grid_game_state_fields(&state, span)?, span)?;
    if state_name == "멈춤" || state_name == "끝" {
        let mut tick_fields = BTreeMap::new();
        tick_fields.insert(
            "__종류".to_string(),
            Value::Str(STD_GRID_GAME_TICK_KIND.to_string()),
        );
        tick_fields.insert("세션".to_string(), session.clone());
        tick_fields.insert("고정됐나".to_string(), Value::Bool(false));
        tick_fields.insert("지운줄수".to_string(), fixed_value(0));
        return Ok(Value::Pack(PackValue {
            fields: tick_fields,
        }));
    }

    let candidate = std_grid_game_apply_input(&falling, input_map, state_store, span)?;
    let blocked = Value::List(ListValue {
        items: vec![Value::Str("X".to_string())],
    });
    let active = if std_grid_game_placeable(&candidate, &grid, &blocked, span)? {
        candidate
    } else {
        falling
    };
    let (piece, x, y) = std_falling_piece_parts(&active, span)?;
    let gravity = make_std_falling_piece(piece, x, y + 1);
    let (next_session, locked, cleared) =
        if std_grid_game_placeable(&gravity, &grid, &blocked, span)? {
            (
                make_std_grid_game_session(grid, bag, state, score, gravity),
                false,
                0,
            )
        } else {
            let locked_grid = std_block_piece_lock(
                &std_falling_piece_block(&active, span)?,
                &grid,
                Value::Str("X".to_string()),
                span,
            )?;
            let cleared_pack =
                std_grid_line_clear(&locked_grid, Value::Str(".".to_string()), span)?;
            let cleared_fields = match &cleared_pack {
                Value::Pack(pack) => &pack.fields,
                _ => unreachable!(),
            };
            let next_grid = std_session_field(cleared_fields, "격자", span)?;
            let cleared_count = std_score_int_field(cleared_fields, "지운줄수", span)?;
            let next_score = std_grid_game_score_add(&score, cleared_count, span)?;
            let next_piece_pack = std_grid_game_next_piece(&bag, span)?;
            let next_fields = match &next_piece_pack {
                Value::Pack(pack) => &pack.fields,
                _ => unreachable!(),
            };
            let next_bag = std_session_field(next_fields, "가방", span)?;
            let next_piece = std_session_field(next_fields, "조각", span)?;
            let grid_fields = std_grid_fields(&next_grid, span)?;
            let (width, _) = std_grid_dims(grid_fields, span)?;
            let spawn = make_std_falling_piece(next_piece, width.div_euclid(2) - 1, 0);
            let next_state = if std_grid_game_placeable(&spawn, &next_grid, &blocked, span)? {
                state
            } else {
                make_std_grid_game_state("끝", Value::None, 0)
            };
            (
                make_std_grid_game_session(next_grid, next_bag, next_state, next_score, spawn),
                true,
                cleared_count,
            )
        };
    let mut tick_fields = BTreeMap::new();
    tick_fields.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_GAME_TICK_KIND.to_string()),
    );
    tick_fields.insert("세션".to_string(), next_session);
    tick_fields.insert("고정됐나".to_string(), Value::Bool(locked));
    tick_fields.insert("지운줄수".to_string(), fixed_value(cleared));
    Ok(Value::Pack(PackValue {
        fields: tick_fields,
    }))
}

fn std_grid_game_view_cell(x: i64, y: i64, value: Value, source: &str) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("x".to_string(), fixed_value(x));
    fields.insert("y".to_string(), fixed_value(y));
    fields.insert("값".to_string(), value);
    fields.insert("원천".to_string(), Value::Str(source.to_string()));
    Value::Pack(PackValue { fields })
}

fn std_grid_game_view_project(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
    span: crate::lang::span::Span,
) -> Result<(i64, i64, Vec<Value>), RuntimeError> {
    let session_fields = std_grid_game_session_fields(session, span)?;
    let grid = std_session_field(session_fields, "격자", span)?;
    let falling = std_session_field(session_fields, "낙하조각", span)?;
    let grid_fields = std_grid_fields(&grid, span)?;
    let (width, height) = std_grid_dims(grid_fields, span)?;
    let cells = std_grid_cells(grid_fields, span)?;
    let expected_len = width
        .checked_mul(height)
        .ok_or(RuntimeError::TypeMismatch {
            expected: "grid cells matching size",
            span,
        })? as usize;
    if cells.len() != expected_len {
        return Err(RuntimeError::TypeMismatch {
            expected: "grid cells matching size",
            span,
        });
    }

    let overlay = std_falling_piece_cells(&falling, span)?
        .into_iter()
        .filter(|(x, y)| *x >= 0 && *y >= 0 && *x < width && *y < height)
        .collect::<BTreeSet<_>>();
    let mut out = Vec::with_capacity(expected_len);
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if overlay.contains(&(x, y)) {
                out.push(std_grid_game_view_cell(x, y, falling_value.clone(), "낙하"));
            } else {
                let cell = cells[idx].clone();
                let source = if &cell == empty_value {
                    "빈칸"
                } else {
                    "고정"
                };
                out.push(std_grid_game_view_cell(x, y, cell, source));
            }
        }
    }
    Ok((width, height, out))
}

fn std_grid_game_view_cells(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let (_, _, cells) = std_grid_game_view_project(session, empty_value, falling_value, span)?;
    Ok(Value::List(ListValue { items: cells }))
}

fn std_grid_game_ghost_piece(
    session: &Value,
    blocked: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let session_fields = std_grid_game_session_fields(session, span)?;
    let grid = std_session_field(session_fields, "격자", span)?;
    let falling = std_session_field(session_fields, "낙하조각", span)?;
    if !std_grid_game_placeable(&falling, &grid, blocked, span)? {
        return Ok(falling);
    }
    let mut current = falling;
    loop {
        let (piece, x, y) = std_falling_piece_parts(&current, span)?;
        let next = make_std_falling_piece(piece, x, y + 1);
        if std_grid_game_placeable(&next, &grid, blocked, span)? {
            current = next;
        } else {
            return Ok(current);
        }
    }
}

fn std_grid_game_ghost_view_project(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
    ghost_value: Value,
    blocked: &Value,
    span: crate::lang::span::Span,
) -> Result<(i64, i64, Vec<Value>), RuntimeError> {
    let session_fields = std_grid_game_session_fields(session, span)?;
    let grid = std_session_field(session_fields, "격자", span)?;
    let falling = std_session_field(session_fields, "낙하조각", span)?;
    let ghost = std_grid_game_ghost_piece(session, blocked, span)?;
    let grid_fields = std_grid_fields(&grid, span)?;
    let (width, height) = std_grid_dims(grid_fields, span)?;
    let cells = std_grid_cells(grid_fields, span)?;
    let expected_len = width
        .checked_mul(height)
        .ok_or(RuntimeError::TypeMismatch {
            expected: "grid cells matching size",
            span,
        })? as usize;
    if cells.len() != expected_len {
        return Err(RuntimeError::TypeMismatch {
            expected: "grid cells matching size",
            span,
        });
    }

    let ghost_overlay = std_falling_piece_cells(&ghost, span)?
        .into_iter()
        .filter(|(x, y)| *x >= 0 && *y >= 0 && *x < width && *y < height)
        .collect::<BTreeSet<_>>();
    let falling_overlay = std_falling_piece_cells(&falling, span)?
        .into_iter()
        .filter(|(x, y)| *x >= 0 && *y >= 0 && *x < width && *y < height)
        .collect::<BTreeSet<_>>();
    let mut out = Vec::with_capacity(expected_len);
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if falling_overlay.contains(&(x, y)) {
                out.push(std_grid_game_view_cell(x, y, falling_value.clone(), "낙하"));
            } else if ghost_overlay.contains(&(x, y)) {
                out.push(std_grid_game_view_cell(x, y, ghost_value.clone(), "유령"));
            } else {
                let cell = cells[idx].clone();
                let source = if &cell == empty_value {
                    "빈칸"
                } else {
                    "고정"
                };
                out.push(std_grid_game_view_cell(x, y, cell, source));
            }
        }
    }
    Ok((width, height, out))
}

fn std_grid_game_view_text(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let (width, height, cells) =
        std_grid_game_view_project(session, empty_value, falling_value, span)?;
    let mut rows = Vec::new();
    for y in 0..height {
        let mut row = String::new();
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let Value::Pack(pack) = &cells[idx] else {
                unreachable!()
            };
            if let Some(value) = pack.fields.get("값") {
                row.push_str(&value.display());
            }
        }
        rows.push(row);
    }
    Ok(Value::Str(rows.join("\n")))
}

fn std_grid_game_view_summary(
    session: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let session_fields = std_grid_game_session_fields(session, span)?;
    let state = std_session_field(session_fields, "상태", span)?;
    let score = std_session_field(session_fields, "점수", span)?;
    let falling = std_session_field(session_fields, "낙하조각", span)?;
    let state_fields = std_grid_game_state_fields(&state, span)?;
    let score_fields = std_grid_game_score_fields(&score, span)?;
    let (_, x, y) = std_falling_piece_parts(&falling, span)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str(STD_GRID_GAME_VIEW_SUMMARY_KIND.to_string()),
    );
    fields.insert(
        "상태".to_string(),
        Value::Str(std_grid_game_state_state(state_fields, span)?),
    );
    fields.insert(
        "틱".to_string(),
        fixed_value(std_grid_game_state_tick(state_fields, span)?),
    );
    fields.insert(
        "점수".to_string(),
        fixed_value(std_score_int_field(score_fields, "점수", span)?),
    );
    fields.insert(
        "줄수".to_string(),
        fixed_value(std_score_int_field(score_fields, "줄수", span)?),
    );
    fields.insert(
        "레벨".to_string(),
        fixed_value(std_score_int_field(score_fields, "레벨", span)?),
    );
    fields.insert(
        "낙하조각위치".to_string(),
        Value::List(ListValue {
            items: vec![fixed_value(x), fixed_value(y)],
        }),
    );
    Ok(Value::Pack(PackValue { fields }))
}

fn std_grid_game_bogae_color(source: &str) -> &'static str {
    match source {
        "낙하" => "#ffcc00ff",
        "유령" => "#88ffffff",
        "고정" => "#4a90e2ff",
        _ => "#111111ff",
    }
}

fn std_grid_game_bogae_rects(
    cells: Vec<Value>,
    cell: i64,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let mut items = Vec::with_capacity(cells.len());
    for value in cells {
        let Value::Pack(pack) = value else {
            return Err(RuntimeError::TypeMismatch {
                expected: "grid game view cell",
                span,
            });
        };
        let x = pack
            .fields
            .get("x")
            .ok_or(RuntimeError::TypeMismatch {
                expected: "grid game view cell x",
                span,
            })
            .and_then(|value| quantity_to_int(&expect_quantity_value(value, span)?, span))?;
        let y = pack
            .fields
            .get("y")
            .ok_or(RuntimeError::TypeMismatch {
                expected: "grid game view cell y",
                span,
            })
            .and_then(|value| quantity_to_int(&expect_quantity_value(value, span)?, span))?;
        let source = match pack.fields.get("원천") {
            Some(Value::Str(source)) => source.as_str(),
            _ => {
                return Err(RuntimeError::TypeMismatch {
                    expected: "grid game view cell source",
                    span,
                })
            }
        };
        let mut fields = BTreeMap::new();
        fields.insert("id".to_string(), Value::Str(format!("격자게임셀_{y}_{x}")));
        fields.insert("결".to_string(), Value::Str("#보개/2D.Rect".to_string()));
        fields.insert("x".to_string(), fixed_value(x.saturating_mul(cell)));
        fields.insert("y".to_string(), fixed_value(y.saturating_mul(cell)));
        fields.insert("w".to_string(), fixed_value(cell));
        fields.insert("h".to_string(), fixed_value(cell));
        fields.insert(
            "채움색".to_string(),
            Value::Str(std_grid_game_bogae_color(source).to_string()),
        );
        items.push(Value::Pack(PackValue { fields }));
    }
    Ok(Value::List(ListValue { items }))
}

fn std_grid_game_bogae_drawlist(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
    cell_size: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let cell = quantity_to_int(&expect_quantity_value(cell_size, span)?, span)?;
    if cell <= 0 {
        return Err(RuntimeError::TypeMismatch {
            expected: "positive cell size",
            span,
        });
    }
    let (_, _, cells) = std_grid_game_view_project(session, empty_value, falling_value, span)?;
    std_grid_game_bogae_rects(cells, cell, span)
}

fn std_grid_game_ghost_bogae_drawlist(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
    ghost_value: Value,
    cell_size: &Value,
    blocked: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let cell = quantity_to_int(&expect_quantity_value(cell_size, span)?, span)?;
    if cell <= 0 {
        return Err(RuntimeError::TypeMismatch {
            expected: "positive cell size",
            span,
        });
    }
    let (_, _, cells) = std_grid_game_ghost_view_project(
        session,
        empty_value,
        falling_value,
        ghost_value,
        blocked,
        span,
    )?;
    std_grid_game_bogae_rects(cells, cell, span)
}

fn std_grid_game_bogae_size(
    session: &Value,
    cell_size: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let cell = quantity_to_int(&expect_quantity_value(cell_size, span)?, span)?;
    if cell <= 0 {
        return Err(RuntimeError::TypeMismatch {
            expected: "positive cell size",
            span,
        });
    }
    let session_fields = std_grid_game_session_fields(session, span)?;
    let grid = std_session_field(session_fields, "격자", span)?;
    let grid_fields = std_grid_fields(&grid, span)?;
    let (width, height) = std_grid_dims(grid_fields, span)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str("std_grid_game_bogae_size".to_string()),
    );
    fields.insert("가로".to_string(), fixed_value(width.saturating_mul(cell)));
    fields.insert("세로".to_string(), fixed_value(height.saturating_mul(cell)));
    Ok(Value::Pack(PackValue { fields }))
}

fn std_grid_pathfind(
    grid: &Value,
    start_x: i64,
    start_y: i64,
    goal_x: i64,
    goal_y: i64,
    blocked_values: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let fields = std_grid_fields(grid, span)?;
    let (width, height) = std_grid_dims(fields, span)?;
    let start = std_grid_index(fields, start_x, start_y, span)?;
    let goal = std_grid_index(fields, goal_x, goal_y, span)?;
    let cells = std_grid_cells(fields, span)?;
    if start >= cells.len() || goal >= cells.len() {
        return Err(RuntimeError::TypeMismatch {
            expected: "grid cells matching size",
            span,
        });
    }
    let is_blocked = |cell: &Value| match blocked_values {
        Value::List(list) => list.items.iter().any(|item| item == cell),
        other => other == cell,
    };
    if is_blocked(&cells[start]) || is_blocked(&cells[goal]) {
        return Ok(Value::List(ListValue { items: Vec::new() }));
    }

    let cell_count = (width * height) as usize;
    let mut visited = vec![false; cell_count];
    let mut prev: Vec<Option<usize>> = vec![None; cell_count];
    let mut queue = VecDeque::new();
    visited[start] = true;
    queue.push_back((start_x, start_y));

    const DIRS: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    while let Some((x, y)) = queue.pop_front() {
        let current = (y * width + x) as usize;
        if current == goal {
            break;
        }
        for (dx, dy) in DIRS {
            let nx = x + dx;
            let ny = y + dy;
            if nx < 0 || ny < 0 || nx >= width || ny >= height {
                continue;
            }
            let next = (ny * width + nx) as usize;
            if next >= cells.len() || visited[next] || is_blocked(&cells[next]) {
                continue;
            }
            visited[next] = true;
            prev[next] = Some(current);
            queue.push_back((nx, ny));
        }
    }

    if !visited[goal] {
        return Ok(Value::List(ListValue { items: Vec::new() }));
    }

    let mut route = Vec::new();
    let mut cursor = goal;
    loop {
        let x = (cursor as i64) % width;
        let y = (cursor as i64) / width;
        route.push(Value::List(ListValue {
            items: vec![fixed_value(x), fixed_value(y)],
        }));
        if cursor == start {
            break;
        }
        cursor = prev[cursor].ok_or(RuntimeError::TypeMismatch {
            expected: "restorable grid path",
            span,
        })?;
    }
    route.reverse();
    Ok(Value::List(ListValue { items: route }))
}

fn make_std_input_map(fields: BTreeMap<String, Value>) -> Value {
    let mut next = BTreeMap::new();
    next.insert(
        "__kind".to_string(),
        Value::Str(STD_INPUT_MAP_KIND.to_string()),
    );
    next.extend(fields);
    Value::Pack(PackValue { fields: next })
}

fn std_input_map_fields(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("입력사상", value, span));
    };
    Ok(pack.fields.clone())
}

fn input_map_key_aliases(action: &str) -> &'static [&'static str] {
    match action {
        "왼쪽" => &["ArrowLeft", "left", "a", "j", "왼쪽", "왼쪽화살표", "좌"],
        "오른쪽" => &[
            "ArrowRight",
            "right",
            "d",
            "l",
            "오른쪽",
            "오른쪽화살표",
            "우",
        ],
        "위" => &["ArrowUp", "up", "w", "i", "위", "위쪽", "위쪽화살표", "상"],
        "아래" => &[
            "ArrowDown",
            "down",
            "s",
            "k",
            "아래",
            "아래쪽",
            "아래쪽화살표",
            "하",
        ],
        "확인" => &[
            "Space",
            "Enter",
            "space",
            "enter",
            "스페이스",
            "스페이스바",
            "엔터",
            "엔터키",
        ],
        "취소" => &["Escape", "escape", "이스케이프", "이스케이프키"],
        _ => &[],
    }
}

fn input_map_value_keys(map: &BTreeMap<String, Value>, action: &str) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(value) = map
        .get(action)
        .or_else(|| map.get(&format!("동작.{action}")))
        .or_else(|| map.get(&format!("키.{action}")))
    {
        match value {
            Value::Str(text) => keys.push(text.clone()),
            Value::List(items) => keys.extend(
                items
                    .items
                    .iter()
                    .map(Value::display)
                    .filter(|key| !key.is_empty()),
            ),
            other => {
                let rendered = other.display();
                if !rendered.is_empty() {
                    keys.push(rendered);
                }
            }
        }
    }
    if keys.is_empty() {
        keys.extend(
            input_map_key_aliases(action)
                .iter()
                .map(|key| key.to_string()),
        );
    }
    keys
}

fn input_map_action_pressed(state: &State, map: &BTreeMap<String, Value>, action: &str) -> bool {
    input_map_value_keys(map, action).iter().any(|key| {
        read_state_flag(state, &format!("샘.키보드.누르고있음.{}", key))
            || read_state_flag(state, &format!("입력상태.키_누르고있음.{}", key))
    })
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
    input.trim().to_string()
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

fn quantity_plain(raw: Fixed64) -> Quantity {
    Quantity {
        raw,
        dim: UnitDim::zero(),
    }
}

fn insert_value_map_entry(entries: &mut BTreeMap<String, MapEntry>, key: &str, value: Value) {
    let key_value = Value::Str(key.to_string());
    entries.insert(
        key_value.canon(),
        MapEntry {
            key: key_value,
            value,
        },
    );
}

fn pick_boim_axis_value(rows: &[(String, Value)], keys: &[&str]) -> Option<Fixed64> {
    for wanted in keys {
        for (key, value) in rows {
            if key == wanted {
                if let Some(number) = boim_value_to_fixed64(value) {
                    return Some(number);
                }
            }
        }
    }
    None
}

fn pick_boim_y_value(rows: &[(String, Value)]) -> Option<Fixed64> {
    if let Some(value) = pick_boim_axis_value(rows, &["y축", "값"]) {
        return Some(value);
    }
    for (key, value) in rows {
        if matches!(key.as_str(), "x축" | "t" | "시간" | "tick") {
            continue;
        }
        if let Some(number) = boim_value_to_fixed64(value) {
            return Some(number);
        }
    }
    None
}

fn boim_value_to_fixed64(value: &Value) -> Option<Fixed64> {
    match value {
        Value::Num(qty) => Some(qty.raw),
        Value::Bool(value) => Some(Fixed64::from_int(if *value { 1 } else { 0 })),
        Value::Str(value) => Fixed64::parse_literal(value),
        _ => None,
    }
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
    let trimmed = text.trim().trim_start_matches('+').trim_start_matches('-');
    !trimmed.is_empty() && trimmed.chars().all(|ch| ch == '0')
}

fn factor_canon_text(raw: &str) -> Option<String> {
    let job = ddonirang_numeric::new_factor_job(raw).ok()?;
    let outcome = ddonirang_numeric::step_factor_job(job, 250_000).ok()?;
    outcome.result.canonical
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

fn eval_exact_binary(
    op: &str,
    left: &Value,
    right: &Value,
    span: crate::lang::span::Span,
) -> Result<Option<Value>, RuntimeError> {
    if exact_numeric_kind(left).is_none() && exact_numeric_kind(right).is_none() {
        return Ok(None);
    }
    let Some(left_exact) = exact_text_from_value(left, span)? else {
        return Ok(None);
    };
    let Some(right_exact) = exact_text_from_value(right, span)? else {
        return Ok(None);
    };
    let out =
        ddonirang_numeric::exact_binary_text(op, &left_exact, &right_exact).map_err(|err| {
            if err == "E_NUMERIC_DIV_ZERO" {
                RuntimeError::MathDivZero { span }
            } else {
                RuntimeError::TypeMismatch {
                    expected: "exact number",
                    span,
                }
            }
        })?;
    Ok(Some(exact_text_to_value(out, op, span)?))
}

fn eval_exact_compare(
    op: &BinaryOp,
    left: &Value,
    right: &Value,
    span: crate::lang::span::Span,
) -> Result<Option<bool>, RuntimeError> {
    if exact_numeric_kind(left).is_none() && exact_numeric_kind(right).is_none() {
        return Ok(None);
    }
    let Some(left_exact) = exact_text_from_value(left, span)? else {
        return Ok(None);
    };
    let Some(right_exact) = exact_text_from_value(right, span)? else {
        return Ok(None);
    };
    let op_text = match op {
        BinaryOp::Eq => "==",
        BinaryOp::NotEq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Lte => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Gte => ">=",
        _ => return Ok(None),
    };
    let result = ddonirang_numeric::exact_compare_text(op_text, &left_exact, &right_exact)
        .map_err(|_| RuntimeError::TypeMismatch {
            expected: "exact number",
            span,
        })?;
    Ok(Some(result))
}

fn exact_text_from_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Option<ddonirang_numeric::ExactText>, RuntimeError> {
    match value {
        Value::Num(qty) if qty.dim.is_dimensionless() => Ok(Some(ddonirang_numeric::ExactText {
            num: qty.raw.raw().to_string(),
            den: (1_i64 << 32).to_string(),
            kind: None,
        })),
        Value::Pack(pack) => match exact_numeric_kind(value) {
            Some(NUMERIC_KIND_BIG_INT) => {
                let raw = exact_pack_str(pack, EXACT_NUMERIC_BIGINT_FIELD, span)?;
                Ok(Some(ddonirang_numeric::ExactText {
                    num: raw,
                    den: "1".to_string(),
                    kind: Some(NUMERIC_KIND_BIG_INT.to_string()),
                }))
            }
            Some(NUMERIC_KIND_RATIONAL) => {
                let num = exact_pack_str(pack, EXACT_NUMERIC_RATIONAL_NUM_FIELD, span)?;
                let den = exact_pack_str(pack, EXACT_NUMERIC_RATIONAL_DEN_FIELD, span)?;
                Ok(Some(ddonirang_numeric::ExactText {
                    num,
                    den,
                    kind: Some(NUMERIC_KIND_RATIONAL.to_string()),
                }))
            }
            Some(NUMERIC_KIND_FACTOR) => {
                let raw = exact_pack_str(pack, EXACT_NUMERIC_FACTOR_VALUE_FIELD, span)?;
                Ok(Some(ddonirang_numeric::ExactText {
                    num: raw,
                    den: "1".to_string(),
                    kind: Some(NUMERIC_KIND_FACTOR.to_string()),
                }))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn exact_pack_str(
    pack: &PackValue,
    key: &str,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    match pack.fields.get(key) {
        Some(Value::Str(text)) => Ok(text.clone()),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "exact numeric field",
            span,
        }),
    }
}

fn exact_text_to_value(
    value: ddonirang_numeric::ExactText,
    op: &str,
    _span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if value.kind.as_deref() == Some(NUMERIC_KIND_FACTOR) && value.den == "1" {
        let raw = value.num;
        let canon = factor_canon_text(&raw).unwrap_or_else(|| raw.clone());
        return Ok(make_exact_numeric_value(
            NUMERIC_KIND_FACTOR,
            &[(EXACT_NUMERIC_FACTOR_VALUE_FIELD, raw), ("정본", canon)],
        ));
    }
    if op == "/" || value.den != "1" || value.kind.as_deref() == Some(NUMERIC_KIND_RATIONAL) {
        return Ok(make_exact_numeric_value(
            NUMERIC_KIND_RATIONAL,
            &[
                (EXACT_NUMERIC_RATIONAL_NUM_FIELD, value.num),
                (EXACT_NUMERIC_RATIONAL_DEN_FIELD, value.den),
            ],
        ));
    }
    Ok(make_exact_numeric_value(
        NUMERIC_KIND_BIG_INT,
        &[(EXACT_NUMERIC_BIGINT_FIELD, value.num)],
    ))
}

fn make_relation_pack(
    left: crate::core::value::MathValue,
    right: crate::core::value::MathValue,
) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_KIND_FIELD.to_string(),
        Value::Str(RELATION_KIND_EQUATION.to_string()),
    );
    fields.insert(RELATION_LEFT_FIELD.to_string(), Value::Math(left));
    fields.insert(RELATION_RIGHT_FIELD.to_string(), Value::Math(right));
    Value::Pack(PackValue { fields })
}

fn eval_endpoint_relation_list(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation",
            span,
        });
    }
    let mut items = Vec::new();
    flatten_endpoint_relation_value(&values[0], span, &mut items)?;
    Ok(Value::List(ListValue { items }))
}

fn eval_endpoint_relation_normalize(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation",
            span,
        });
    }
    let mut items = Vec::new();
    flatten_endpoint_relation_value(&values[0], span, &mut items)?;
    let count = items.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_relation_flat_set".to_string()),
    );
    fields.insert(
        "개수".to_string(),
        Value::Num(Quantity::new(Fixed64::from_int(count), UnitDim::zero())),
    );
    fields.insert("관계들".to_string(), Value::List(ListValue { items }));
    Ok(Value::Pack(PackValue { fields }))
}

struct EndpointFormulaRelationBridge {
    relations: Vec<Value>,
    mappings: Vec<Value>,
}

fn eval_endpoint_formula_relation_list(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let bridge = endpoint_formula_relation_bridge(values, span)?;
    Ok(Value::List(ListValue {
        items: bridge.relations,
    }))
}

fn eval_endpoint_formula_relation_set(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let bridge = endpoint_formula_relation_bridge(values, span)?;
    let relation_count = bridge.relations.len() as i64;
    let mapping_count = bridge.mappings.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_formula_relation_set".to_string()),
    );
    fields.insert(
        "개수".to_string(),
        Value::Num(Quantity::new(
            Fixed64::from_int(relation_count),
            UnitDim::zero(),
        )),
    );
    fields.insert(
        "변수개수".to_string(),
        Value::Num(Quantity::new(
            Fixed64::from_int(mapping_count),
            UnitDim::zero(),
        )),
    );
    fields.insert(
        "관계들".to_string(),
        Value::List(ListValue {
            items: bridge.relations,
        }),
    );
    fields.insert(
        "변수사상".to_string(),
        Value::List(ListValue {
            items: bridge.mappings,
        }),
    );
    Ok(Value::Pack(PackValue { fields }))
}

struct EndpointBoundaryValueInjection {
    relations: Vec<Value>,
    injected_relations: Vec<Value>,
    injected_values: Vec<Value>,
    mappings: Vec<Value>,
}

fn eval_endpoint_boundary_value_relation_list(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let injected = endpoint_boundary_value_injection(values, span)?;
    Ok(Value::List(ListValue {
        items: injected.injected_relations,
    }))
}

fn eval_endpoint_boundary_value_injection(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let injected = endpoint_boundary_value_injection(values, span)?;
    let relation_count = injected.relations.len() as i64;
    let mapping_count = injected.mappings.len() as i64;
    let injected_count = injected.injected_values.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_formula_relation_set_with_values".to_string()),
    );
    fields.insert(
        "개수".to_string(),
        Value::Num(Quantity::new(
            Fixed64::from_int(relation_count),
            UnitDim::zero(),
        )),
    );
    fields.insert(
        "변수개수".to_string(),
        Value::Num(Quantity::new(
            Fixed64::from_int(mapping_count),
            UnitDim::zero(),
        )),
    );
    fields.insert(
        "주입개수".to_string(),
        Value::Num(Quantity::new(
            Fixed64::from_int(injected_count),
            UnitDim::zero(),
        )),
    );
    fields.insert(
        "관계들".to_string(),
        Value::List(ListValue {
            items: injected.relations,
        }),
    );
    fields.insert(
        "변수사상".to_string(),
        Value::List(ListValue {
            items: injected.mappings,
        }),
    );
    fields.insert(
        "주입값들".to_string(),
        Value::List(ListValue {
            items: injected.injected_values,
        }),
    );
    Ok(Value::Pack(PackValue { fields }))
}

fn eval_endpoint_explicit_solve(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation and boundary value list",
            span,
        });
    }
    let formula_set = eval_endpoint_formula_relation_set(&[values[0].clone()], span)?;
    let solve_source = match &values[1] {
        Value::List(list) if list.items.is_empty() => formula_set,
        Value::List(_) => {
            eval_endpoint_boundary_value_injection(&[formula_set, values[1].clone()], span)?
        }
        _ => eval_endpoint_boundary_value_injection(&[formula_set, values[1].clone()], span)?,
    };
    let formula_pack = expect_endpoint_formula_relation_mapping_source(&solve_source, span)?;
    let relation_list = endpoint_relation_list_field(&formula_pack.fields, "관계들", span)?.clone();
    let relation_arg = Value::List(relation_list);
    let solve_result = match expect_equation_relations(&[relation_arg], span)
        .and_then(|relations| eval_relation_solve_result(&relations, span))
    {
        Ok(result) => result,
        Err(_) => make_relation_solve_failure("unsupported"),
    };
    eval_endpoint_solve_result_remap(&[solve_source, solve_result], span)
}

fn eval_endpoint_explicit_solve_range_violation_list(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation, boundary value list, and range list",
            span,
        });
    }
    let solve_result = eval_endpoint_explicit_solve(&[values[0].clone(), values[1].clone()], span)?;
    eval_endpoint_boundary_range_violation_list(&[solve_result, values[2].clone()], span)
}

fn eval_endpoint_explicit_solve_range_check(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation, boundary value list, and range list",
            span,
        });
    }
    let solve_result = eval_endpoint_explicit_solve(&[values[0].clone(), values[1].clone()], span)?;
    let range_check =
        eval_endpoint_boundary_range_check(&[solve_result.clone(), values[2].clone()], span)?;
    let Value::Pack(solve_pack) = &solve_result else {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint solve result",
            span,
        });
    };
    let Value::Pack(range_pack) = &range_check else {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint range check",
            span,
        });
    };
    let solve_kind =
        endpoint_relation_string_field(&solve_pack.fields, "풀이결과종류", span)?.to_string();
    let check_kind =
        endpoint_relation_string_field(&range_pack.fields, "검사결과", span)?.to_string();
    let violation_count =
        range_pack
            .fields
            .get("위반개수")
            .cloned()
            .ok_or(RuntimeError::TypeMismatch {
                expected: "endpoint range check violation count",
                span,
            })?;

    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_solve_range_check".to_string()),
    );
    fields.insert("풀이결과".to_string(), solve_result);
    fields.insert("범위검사".to_string(), range_check);
    fields.insert("풀이결과종류".to_string(), Value::Str(solve_kind));
    fields.insert("검사결과".to_string(), Value::Str(check_kind));
    fields.insert("위반개수".to_string(), violation_count);
    Ok(Value::Pack(PackValue { fields }))
}

struct EndpointSolveRangeReport {
    check: Value,
    rows: Vec<Value>,
    solve_kind: String,
    check_kind: String,
    value_count: usize,
    missing_count: usize,
    range_count: usize,
    violation_count: usize,
}

fn eval_endpoint_solve_range_report_rows(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let report = endpoint_solve_range_report(values, span)?;
    Ok(Value::List(ListValue { items: report.rows }))
}

fn eval_endpoint_solve_range_report(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let report = endpoint_solve_range_report(values, span)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_solve_range_report".to_string()),
    );
    fields.insert("검사".to_string(), report.check);
    fields.insert("풀이결과종류".to_string(), Value::Str(report.solve_kind));
    fields.insert("검사결과".to_string(), Value::Str(report.check_kind));
    fields.insert(
        "행개수".to_string(),
        endpoint_report_count(report.rows.len()),
    );
    fields.insert(
        "값개수".to_string(),
        endpoint_report_count(report.value_count),
    );
    fields.insert(
        "누락개수".to_string(),
        endpoint_report_count(report.missing_count),
    );
    fields.insert(
        "범위개수".to_string(),
        endpoint_report_count(report.range_count),
    );
    fields.insert(
        "위반개수".to_string(),
        endpoint_report_count(report.violation_count),
    );
    fields.insert(
        "행들".to_string(),
        Value::List(ListValue { items: report.rows }),
    );
    Ok(Value::Pack(PackValue { fields }))
}

fn eval_endpoint_solve_range_text_report(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
            span,
        ));
    }
    Ok(Value::Str(endpoint_solve_range_text_table(
        &values[0], span,
    )?))
}

fn eval_endpoint_explicit_solve_range_text_report(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let report = eval_endpoint_solve_range_report(values, span)?;
    Ok(Value::Str(endpoint_solve_range_text_table(&report, span)?))
}

fn eval_endpoint_solve_range_case(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
            span,
        ));
    }
    endpoint_solve_range_case_result(&values[0], span)
}

fn eval_endpoint_solve_range_case_suite(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
            span,
        ));
    }
    let Value::List(cases) = &values[0] else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
            span,
        ));
    };
    let mut results = Vec::new();
    let mut passed = 0usize;
    for case in &cases.items {
        let result = endpoint_solve_range_case_result(case, span)?;
        let Value::Pack(result_pack) = &result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_malformed_case",
                span,
            ));
        };
        let pass = match result_pack.fields.get("통과여부") {
            Some(Value::Bool(pass)) => *pass,
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_malformed_case",
                    span,
                ))
            }
        };
        if pass {
            passed += 1;
        }
        results.push(result);
    }
    Ok(endpoint_solve_range_case_suite_pack(results, passed))
}

fn eval_endpoint_solve_range_case_suite_text(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
            span,
        ));
    }
    Ok(Value::Str(endpoint_solve_range_case_suite_text(
        &values[0], span,
    )?))
}

fn eval_endpoint_solve_range_case_suite_detail_text(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
            span,
        ));
    }
    Ok(Value::Str(endpoint_solve_range_case_suite_detail_text(
        &values[0], span,
    )?))
}

fn eval_endpoint_solve_range_case_suite_run_detail_text(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
            span,
        ));
    }
    let suite = eval_endpoint_solve_range_case_suite(values, span)?;
    Ok(Value::Str(endpoint_solve_range_case_suite_detail_text(
        &suite, span,
    )?))
}

fn eval_endpoint_solve_range_case_suite_summary(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
            span,
        ));
    }
    endpoint_solve_range_case_suite_summary(&values[0], span)
}

fn eval_endpoint_solve_range_case_suite_run_summary(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
            span,
        ));
    }
    let suite = eval_endpoint_solve_range_case_suite(values, span)?;
    endpoint_solve_range_case_suite_summary(&suite, span)
}

fn eval_endpoint_solve_range_case_suite_check(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
            span,
        ));
    }
    endpoint_solve_range_case_suite_check(&values[0], span)
}

fn eval_endpoint_solve_range_case_suite_run_check(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
            span,
        ));
    }
    let summary = eval_endpoint_solve_range_case_suite_run_summary(values, span)?;
    endpoint_solve_range_case_suite_check(&summary, span)
}

fn endpoint_solve_range_case_result(
    case: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let Value::Pack(case_pack) = case else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
            span,
        ));
    };
    let name = endpoint_relation_string_field(&case_pack.fields, "이름", span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_malformed_case", span))?
        .to_string();
    let relation = case_pack
        .fields
        .get("이음관계")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case", span))?;
    let values = case_pack
        .fields
        .get("값들")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case", span))?;
    let ranges = case_pack
        .fields
        .get("범위들")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case", span))?;
    let expected = match case_pack.fields.get("기대검사결과") {
        Some(Value::Str(value)) if value == "통과" || value == "실패" => value.clone(),
        Some(_) => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_invalid_expected_result",
                span,
            ))
        }
        None => "통과".to_string(),
    };
    let report = eval_endpoint_solve_range_report(&[relation, values, ranges], span)?;
    let Value::Pack(report_pack) = &report else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
            span,
        ));
    };
    let actual = endpoint_relation_string_field(&report_pack.fields, "검사결과", span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_malformed_case", span))?
        .to_string();
    let text = endpoint_solve_range_text_table(&report, span)?;
    let passed = expected == actual;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_solve_range_case_result".to_string()),
    );
    fields.insert("이름".to_string(), Value::Str(name));
    fields.insert("기대검사결과".to_string(), Value::Str(expected));
    fields.insert("실제검사결과".to_string(), Value::Str(actual));
    fields.insert("통과여부".to_string(), Value::Bool(passed));
    fields.insert("보고서".to_string(), report);
    fields.insert("문자표".to_string(), Value::Str(text));
    Ok(Value::Pack(PackValue { fields }))
}

fn endpoint_solve_range_case_suite_pack(results: Vec<Value>, passed: usize) -> Value {
    let total = results.len();
    let failed = total.saturating_sub(passed);
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_solve_range_case_suite".to_string()),
    );
    fields.insert("개수".to_string(), endpoint_report_count(total));
    fields.insert("통과개수".to_string(), endpoint_report_count(passed));
    fields.insert("실패개수".to_string(), endpoint_report_count(failed));
    fields.insert("전체통과".to_string(), Value::Bool(failed == 0));
    fields.insert(
        "결과들".to_string(),
        Value::List(ListValue { items: results }),
    );
    Value::Pack(PackValue { fields })
}

fn endpoint_solve_range_case_suite_text(
    suite: &Value,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
            span,
        ));
    };
    if endpoint_relation_kind(&suite_pack.fields, span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite", span))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
            span,
        ));
    }
    let results = endpoint_relation_list_field(&suite_pack.fields, "결과들", span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite", span))?;
    let mut lines = vec!["이름\t기대\t실제\t통과".to_string()];
    for result in &results.items {
        let Value::Pack(result_pack) = result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_text_expected_suite",
                span,
            ));
        };
        if endpoint_relation_kind(&result_pack.fields, span).map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_text_expected_suite", span)
        })? != "endpoint_solve_range_case_result"
        {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_text_expected_suite",
                span,
            ));
        }
        let name =
            endpoint_relation_string_field(&result_pack.fields, "이름", span).map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_text_expected_suite", span)
            })?;
        let expected = endpoint_relation_string_field(&result_pack.fields, "기대검사결과", span)
            .map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_text_expected_suite", span)
            })?;
        let actual = endpoint_relation_string_field(&result_pack.fields, "실제검사결과", span)
            .map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_text_expected_suite", span)
            })?;
        let passed = match result_pack.fields.get("통과여부") {
            Some(Value::Bool(true)) => "참",
            Some(Value::Bool(false)) => "거짓",
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_text_expected_suite",
                    span,
                ))
            }
        };
        lines.push(format!("{name}\t{expected}\t{actual}\t{passed}"));
    }
    Ok(lines.join("\n"))
}

fn endpoint_solve_range_case_suite_detail_text(
    suite: &Value,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let summary = endpoint_solve_range_case_suite_text(suite, span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite", span))?;
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
            span,
        ));
    };
    if endpoint_relation_kind(&suite_pack.fields, span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite", span))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
            span,
        ));
    }
    let results = endpoint_relation_list_field(&suite_pack.fields, "결과들", span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite", span))?;
    let mut sections = vec![summary];
    for result in &results.items {
        sections.push(endpoint_solve_range_case_detail_section(result, span)?);
    }
    Ok(sections.join("\n\n"))
}

fn endpoint_solve_range_case_detail_section(
    result: &Value,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let Value::Pack(result_pack) = result else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_malformed_case_result",
            span,
        ));
    };
    if endpoint_relation_kind(&result_pack.fields, span).map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result", span)
    })? != "endpoint_solve_range_case_result"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_malformed_case_result",
            span,
        ));
    }
    let name = endpoint_relation_string_field(&result_pack.fields, "이름", span).map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result", span)
    })?;
    let expected = endpoint_relation_string_field(&result_pack.fields, "기대검사결과", span)
        .map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result", span)
        })?;
    let actual = endpoint_relation_string_field(&result_pack.fields, "실제검사결과", span)
        .map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result", span)
        })?;
    let passed = match result_pack.fields.get("통과여부") {
        Some(Value::Bool(true)) => "참",
        Some(Value::Bool(false)) => "거짓",
        _ => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_detail_malformed_case_result",
                span,
            ))
        }
    };
    let text =
        endpoint_relation_string_field(&result_pack.fields, "문자표", span).map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result", span)
        })?;
    Ok(format!(
        "## {name}\n기대\t{expected}\n실제\t{actual}\n통과\t{passed}\n{text}"
    ))
}

fn endpoint_solve_range_case_suite_summary(
    suite: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
            span,
        ));
    };
    if endpoint_relation_kind(&suite_pack.fields, span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_summary_expected_suite", span))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
            span,
        ));
    }
    let results =
        endpoint_relation_list_field(&suite_pack.fields, "결과들", span).map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_summary_expected_suite", span)
        })?;

    let mut pass_names = Vec::new();
    let mut fail_names = Vec::new();
    let mut expected_fail_actual_pass = Vec::new();
    let mut expected_pass_actual_fail = Vec::new();

    for result in &results.items {
        let Value::Pack(result_pack) = result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_summary_malformed_case_result",
                span,
            ));
        };
        if endpoint_relation_kind(&result_pack.fields, span).map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result", span)
        })? != "endpoint_solve_range_case_result"
        {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_summary_malformed_case_result",
                span,
            ));
        }
        let name = endpoint_relation_string_field(&result_pack.fields, "이름", span)
            .map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result", span)
            })?
            .to_string();
        let expected = endpoint_relation_string_field(&result_pack.fields, "기대검사결과", span)
            .map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result", span)
            })?;
        let actual = endpoint_relation_string_field(&result_pack.fields, "실제검사결과", span)
            .map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result", span)
            })?;
        let passed = match result_pack.fields.get("통과여부") {
            Some(Value::Bool(pass)) => *pass,
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_summary_malformed_case_result",
                    span,
                ))
            }
        };
        if passed {
            pass_names.push(name.clone());
        } else {
            fail_names.push(name.clone());
        }
        if expected == "실패" && actual == "통과" {
            expected_fail_actual_pass.push(name.clone());
        }
        if expected == "통과" && actual == "실패" {
            expected_pass_actual_fail.push(name);
        }
    }

    let total = results.items.len();
    let passed = pass_names.len();
    let failed = fail_names.len();
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_solve_range_case_suite_summary".to_string()),
    );
    fields.insert("개수".to_string(), endpoint_report_count(total));
    fields.insert("통과개수".to_string(), endpoint_report_count(passed));
    fields.insert("실패개수".to_string(), endpoint_report_count(failed));
    fields.insert("전체통과".to_string(), Value::Bool(failed == 0));
    fields.insert(
        "통과케이스들".to_string(),
        Value::List(ListValue {
            items: pass_names.into_iter().map(Value::Str).collect(),
        }),
    );
    fields.insert(
        "실패케이스들".to_string(),
        Value::List(ListValue {
            items: fail_names.into_iter().map(Value::Str).collect(),
        }),
    );
    fields.insert(
        "기대실패통과케이스들".to_string(),
        Value::List(ListValue {
            items: expected_fail_actual_pass
                .into_iter()
                .map(Value::Str)
                .collect(),
        }),
    );
    fields.insert(
        "기대통과실패케이스들".to_string(),
        Value::List(ListValue {
            items: expected_pass_actual_fail
                .into_iter()
                .map(Value::Str)
                .collect(),
        }),
    );
    Ok(Value::Pack(PackValue { fields }))
}

fn endpoint_solve_range_case_suite_check(
    summary: &Value,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let Value::Pack(summary_pack) = summary else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
            span,
        ));
    };
    if endpoint_relation_kind(&summary_pack.fields, span)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_check_expected_summary", span))?
        != "endpoint_solve_range_case_suite_summary"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
            span,
        ));
    }

    let overall = match summary_pack.fields.get("전체통과") {
        Some(Value::Bool(value)) => *value,
        _ => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_check_malformed_summary",
                span,
            ))
        }
    };
    let count = endpoint_case_suite_check_count(&summary_pack.fields, "개수", span)?;
    let passed = endpoint_case_suite_check_count(&summary_pack.fields, "통과개수", span)?;
    let failed = endpoint_case_suite_check_count(&summary_pack.fields, "실패개수", span)?;
    let fail_cases =
        endpoint_case_suite_check_string_list(&summary_pack.fields, "실패케이스들", span)?;
    let expected_fail_actual_pass = endpoint_case_suite_check_string_list(
        &summary_pack.fields,
        "기대실패통과케이스들",
        span,
    )?;
    let expected_pass_actual_fail = endpoint_case_suite_check_string_list(
        &summary_pack.fields,
        "기대통과실패케이스들",
        span,
    )?;

    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_solve_range_case_suite_check".to_string()),
    );
    fields.insert(
        "판정".to_string(),
        Value::Str(if overall { "통과" } else { "실패" }.to_string()),
    );
    fields.insert("전체통과".to_string(), Value::Bool(overall));
    fields.insert("개수".to_string(), count);
    fields.insert("통과개수".to_string(), passed);
    fields.insert("실패개수".to_string(), failed);
    fields.insert("실패케이스들".to_string(), Value::List(fail_cases));
    fields.insert(
        "기대실패통과케이스들".to_string(),
        Value::List(expected_fail_actual_pass),
    );
    fields.insert(
        "기대통과실패케이스들".to_string(),
        Value::List(expected_pass_actual_fail),
    );
    fields.insert("요약".to_string(), summary.clone());
    Ok(Value::Pack(PackValue { fields }))
}

fn endpoint_case_suite_check_count(
    fields: &BTreeMap<String, Value>,
    name: &str,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    match fields.get(name) {
        Some(Value::Num(_)) => Ok(fields.get(name).expect("count field").clone()),
        _ => Err(endpoint_case_suite_error(
            "connect_case_suite_check_malformed_summary",
            span,
        )),
    }
}

fn endpoint_case_suite_check_string_list(
    fields: &BTreeMap<String, Value>,
    name: &str,
    span: crate::lang::span::Span,
) -> Result<ListValue, RuntimeError> {
    let list = endpoint_relation_list_field(fields, name, span).map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_check_malformed_summary", span)
    })?;
    if list.items.iter().any(|item| !matches!(item, Value::Str(_))) {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_malformed_summary",
            span,
        ));
    }
    Ok(list.clone())
}

fn endpoint_case_suite_error(marker: &'static str, span: crate::lang::span::Span) -> RuntimeError {
    RuntimeError::Pack {
        message: marker.to_string(),
        span,
    }
}

fn endpoint_solve_range_text_table(
    report: &Value,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let Value::Pack(report_pack) = report else {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
            span,
        ));
    };
    if endpoint_relation_kind(&report_pack.fields, span).map_err(|_| {
        endpoint_report_text_error("connect_report_text_expected_solve_range_report", span)
    })? != "endpoint_solve_range_report"
    {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
            span,
        ));
    }
    let rows = endpoint_relation_list_field(&report_pack.fields, "행들", span)
        .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row", span))?;
    let mut lines = vec!["변수\t경로\t값상태\t값\t범위상태\t하한\t상한\t위반".to_string()];
    for row in &rows.items {
        let Value::Pack(row_pack) = row else {
            return Err(endpoint_report_text_error(
                "connect_report_text_malformed_row",
                span,
            ));
        };
        if endpoint_relation_kind(&row_pack.fields, span)
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row", span))?
            != "endpoint_solve_range_report_row"
        {
            return Err(endpoint_report_text_error(
                "connect_report_text_malformed_row",
                span,
            ));
        }
        let variable = endpoint_relation_string_field(&row_pack.fields, "변수", span)
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row", span))?;
        let path = endpoint_relation_string_field(&row_pack.fields, "경로", span)
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row", span))?;
        let value_status = endpoint_relation_string_field(&row_pack.fields, "값상태", span)
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row", span))?;
        let range_status = endpoint_relation_string_field(&row_pack.fields, "범위상태", span)
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row", span))?;
        let value = row_pack
            .fields
            .get("값")
            .map(Value::display)
            .unwrap_or_default();
        let lower = row_pack
            .fields
            .get("하한")
            .map(Value::display)
            .unwrap_or_default();
        let upper = row_pack
            .fields
            .get("상한")
            .map(Value::display)
            .unwrap_or_default();
        let violations = endpoint_relation_list_field(&row_pack.fields, "위반들", span)
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row", span))?;
        let mut reasons = Vec::new();
        for violation in &violations.items {
            let Value::Pack(violation_pack) = violation else {
                return Err(endpoint_report_text_error(
                    "connect_report_text_malformed_row",
                    span,
                ));
            };
            reasons.push(
                endpoint_relation_string_field(&violation_pack.fields, "이유", span)
                    .map_err(|_| {
                        endpoint_report_text_error("connect_report_text_malformed_row", span)
                    })?
                    .to_string(),
            );
        }
        lines.push(format!(
            "{variable}\t{path}\t{value_status}\t{value}\t{range_status}\t{lower}\t{upper}\t{}",
            reasons.join("|")
        ));
    }
    Ok(lines.join("\n"))
}

fn endpoint_report_text_error(marker: &'static str, span: crate::lang::span::Span) -> RuntimeError {
    RuntimeError::Pack {
        message: marker.to_string(),
        span,
    }
}

fn endpoint_solve_range_report(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<EndpointSolveRangeReport, RuntimeError> {
    if values.len() != 3 {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation, boundary value list, and range list",
            span,
        });
    }
    let check = eval_endpoint_explicit_solve_range_check(values, span)?;
    let Value::Pack(check_pack) = &check else {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint solve range check",
            span,
        });
    };
    let solve_result =
        check_pack
            .fields
            .get("풀이결과")
            .cloned()
            .ok_or(RuntimeError::TypeMismatch {
                expected: "endpoint solve range check solve result",
                span,
            })?;
    let range_check =
        check_pack
            .fields
            .get("범위검사")
            .cloned()
            .ok_or(RuntimeError::TypeMismatch {
                expected: "endpoint solve range check range check",
                span,
            })?;
    let solve_pack = expect_endpoint_solve_result_value(&solve_result, span)?;
    let Value::Pack(range_pack) = &range_check else {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint range check",
            span,
        });
    };
    let solve_kind =
        endpoint_relation_string_field(&solve_pack.fields, "풀이결과종류", span)?.to_string();
    let check_kind =
        endpoint_relation_string_field(&range_pack.fields, "검사결과", span)?.to_string();
    let mappings = endpoint_relation_list_field(&solve_pack.fields, "변수사상", span)?;
    let value_by_path = endpoint_solve_result_values_by_path(solve_pack, span)?;
    let ranges = endpoint_range_bounds(&values[2], span)?;
    let mut range_by_path: BTreeMap<String, (Option<Value>, Option<Value>)> = BTreeMap::new();
    for range in &ranges {
        range_by_path.insert(range.path.clone(), (range.min.clone(), range.max.clone()));
    }
    let mut violations_by_path: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let violations = endpoint_relation_list_field(&range_pack.fields, "위반들", span)?;
    for violation in &violations.items {
        let Value::Pack(violation_pack) = violation else {
            return Err(RuntimeError::TypeMismatch {
                expected: "endpoint range violation",
                span,
            });
        };
        let path = endpoint_relation_string_field(&violation_pack.fields, "경로", span)?;
        violations_by_path
            .entry(path.to_string())
            .or_default()
            .push(violation.clone());
    }

    let mut rows = Vec::new();
    let mut value_count = 0usize;
    let mut missing_count = 0usize;
    for mapping in &mappings.items {
        let Value::Pack(mapping_pack) = mapping else {
            return Err(RuntimeError::TypeMismatch {
                expected: "endpoint variable mapping",
                span,
            });
        };
        let variable = endpoint_relation_string_field(&mapping_pack.fields, "변수", span)?;
        let path = endpoint_relation_string_field(&mapping_pack.fields, "경로", span)?;
        let row_violations = violations_by_path.get(path).cloned().unwrap_or_default();
        let mut fields = BTreeMap::new();
        fields.insert(
            "__이음관계종류".to_string(),
            Value::Str("endpoint_solve_range_report_row".to_string()),
        );
        fields.insert("변수".to_string(), Value::Str(variable.to_string()));
        fields.insert("경로".to_string(), Value::Str(path.to_string()));
        if let Some(value) = value_by_path.get(path) {
            fields.insert("값상태".to_string(), Value::Str("값있음".to_string()));
            fields.insert("값".to_string(), value.clone());
            value_count += 1;
        } else {
            fields.insert("값상태".to_string(), Value::Str("누락".to_string()));
            missing_count += 1;
        }
        if let Some((min, max)) = range_by_path.get(path) {
            fields.insert(
                "범위상태".to_string(),
                Value::Str(if row_violations.is_empty() {
                    "통과".to_string()
                } else {
                    "실패".to_string()
                }),
            );
            if let Some(min) = min {
                fields.insert("하한".to_string(), min.clone());
            }
            if let Some(max) = max {
                fields.insert("상한".to_string(), max.clone());
            }
        } else {
            fields.insert("범위상태".to_string(), Value::Str("범위없음".to_string()));
        }
        fields.insert(
            "위반개수".to_string(),
            endpoint_report_count(row_violations.len()),
        );
        fields.insert(
            "위반들".to_string(),
            Value::List(ListValue {
                items: row_violations,
            }),
        );
        rows.push(Value::Pack(PackValue { fields }));
    }

    Ok(EndpointSolveRangeReport {
        check,
        rows,
        solve_kind,
        check_kind,
        value_count,
        missing_count,
        range_count: ranges.len(),
        violation_count: violations.items.len(),
    })
}

fn endpoint_report_count(value: usize) -> Value {
    Value::Num(Quantity::new(
        Fixed64::from_int(value as i64),
        UnitDim::zero(),
    ))
}

fn eval_endpoint_solve_result_value_list(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let remapped = endpoint_solve_result_remap(values, span)?;
    Ok(Value::List(ListValue {
        items: remapped.values,
    }))
}

fn eval_endpoint_solve_result_remap(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let remapped = endpoint_solve_result_remap(values, span)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_solve_result".to_string()),
    );
    fields.insert("풀이결과종류".to_string(), Value::Str(remapped.kind));
    fields.insert(
        "값들".to_string(),
        Value::List(ListValue {
            items: remapped.values,
        }),
    );
    fields.insert(
        "누락변수들".to_string(),
        Value::List(ListValue {
            items: remapped.missing_variables,
        }),
    );
    fields.insert(
        "변수사상".to_string(),
        Value::List(ListValue {
            items: remapped.mappings,
        }),
    );
    fields.insert("원래풀이".to_string(), remapped.original);
    Ok(Value::Pack(PackValue { fields }))
}

struct EndpointSolveResultRemap {
    kind: String,
    values: Vec<Value>,
    missing_variables: Vec<Value>,
    mappings: Vec<Value>,
    original: Value,
}

#[derive(Clone, Copy)]
struct EndpointUnitSeed {
    dim: UnitDim,
}

struct EndpointBoundaryNumber {
    value: Value,
    formula_text: String,
    unit_dim: Option<UnitDim>,
    unit_symbol: Option<String>,
}

fn endpoint_solve_result_remap(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<EndpointSolveResultRemap, RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint formula relation set and relation solve result",
            span,
        });
    }
    let formula_set = expect_endpoint_formula_relation_mapping_source(&values[0], span)?;
    let mappings_list = endpoint_relation_list_field(&formula_set.fields, "변수사상", span)?;
    let mut mappings = Vec::new();
    let mut ordered_mapping = Vec::new();
    for item in &mappings_list.items {
        let Value::Pack(mapping_pack) = item else {
            return Err(RuntimeError::TypeMismatch {
                expected: "endpoint variable mapping",
                span,
            });
        };
        let variable = endpoint_relation_string_field(&mapping_pack.fields, "변수", span)?;
        let path = endpoint_relation_string_field(&mapping_pack.fields, "경로", span)?;
        if ordered_mapping
            .iter()
            .any(|(known, _): &(String, String)| known == variable)
        {
            return Err(RuntimeError::TypeMismatch {
                expected: "unique endpoint variable mapping",
                span,
            });
        }
        ordered_mapping.push((variable.to_string(), path.to_string()));
        mappings.push(item.clone());
    }
    let unit_components = endpoint_unit_components(formula_set, &ordered_mapping, span)?;

    let solve_result = expect_relation_solve_result(&values[1], span)?;
    let original = values[1].clone();
    let result_kind = relation_solve_result_kind(&solve_result.fields, span)?;
    if result_kind != RELATION_SOLVE_RESULT_SUCCESS {
        return Ok(EndpointSolveResultRemap {
            kind: "실패".to_string(),
            values: Vec::new(),
            missing_variables: ordered_mapping
                .iter()
                .map(|(variable, _)| Value::Str(variable.clone()))
                .collect(),
            mappings,
            original,
        });
    }

    let bindings = match solve_result.fields.get(RELATION_SOLVE_BINDINGS_FIELD) {
        Some(Value::Pack(bindings)) => &bindings.fields,
        _ => {
            return Err(RuntimeError::TypeMismatch {
                expected: "relation solve result bindings",
                span,
            })
        }
    };
    for variable in bindings.keys() {
        if !ordered_mapping
            .iter()
            .any(|(mapped_variable, _)| mapped_variable == variable)
        {
            return Err(RuntimeError::TypeMismatch {
                expected: "endpoint_variable_mapping relation solve bindings covered by endpoint variable mapping",
                span,
            });
        }
    }

    let mut remapped_values = Vec::new();
    let mut missing_variables = Vec::new();
    for (variable, path) in &ordered_mapping {
        if let Some(value) = bindings.get(variable) {
            let value = match unit_components.get(variable) {
                Some(seed) => {
                    let raw = endpoint_solve_numeric_value(value, span)?;
                    Value::Num(Quantity::new(raw, seed.dim))
                }
                None => value.clone(),
            };
            let mut item = BTreeMap::new();
            item.insert("변수".to_string(), Value::Str(variable.clone()));
            item.insert("경로".to_string(), Value::Str(path.clone()));
            item.insert("값".to_string(), value);
            remapped_values.push(Value::Pack(PackValue { fields: item }));
        } else {
            missing_variables.push(Value::Str(variable.clone()));
        }
    }
    let kind = if missing_variables.is_empty() {
        "성공"
    } else {
        "부분성공"
    };
    Ok(EndpointSolveResultRemap {
        kind: kind.to_string(),
        values: remapped_values,
        missing_variables,
        mappings,
        original,
    })
}

fn expect_endpoint_formula_relation_set(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<&PackValue, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint_formula_relation_set",
            span,
        });
    };
    match pack.fields.get("__이음관계종류") {
        Some(Value::Str(kind)) if kind == "endpoint_formula_relation_set" => Ok(pack),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "endpoint_formula_relation_set",
            span,
        }),
    }
}

fn expect_endpoint_formula_relation_mapping_source(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<&PackValue, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint_formula_relation_set",
            span,
        });
    };
    match pack.fields.get("__이음관계종류") {
        Some(Value::Str(kind))
            if kind == "endpoint_formula_relation_set"
                || kind == "endpoint_formula_relation_set_with_values" =>
        {
            Ok(pack)
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "endpoint_formula_relation_set",
            span,
        }),
    }
}

fn endpoint_boundary_value_injection(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<EndpointBoundaryValueInjection, RuntimeError> {
    if values.len() != 2 {
        return Err(endpoint_boundary_value_error(
            "connect_boundary_value_expected_formula_set",
            span,
        ));
    }
    let formula_set = expect_endpoint_formula_relation_set(&values[0], span).map_err(|_| {
        endpoint_boundary_value_error("connect_boundary_value_expected_formula_set", span)
    })?;
    let original_relations = endpoint_relation_list_field(&formula_set.fields, "관계들", span)?;
    let mappings_list = endpoint_relation_list_field(&formula_set.fields, "변수사상", span)?;

    let mut path_to_variable = BTreeMap::new();
    let mut mappings = Vec::new();
    for item in &mappings_list.items {
        let Value::Pack(mapping_pack) = item else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_malformed_item",
                span,
            ));
        };
        let variable = endpoint_relation_string_field(&mapping_pack.fields, "변수", span)?;
        let path = endpoint_relation_string_field(&mapping_pack.fields, "경로", span)?;
        path_to_variable.insert(path.to_string(), variable.to_string());
        mappings.push(item.clone());
    }

    let Value::List(value_items) = &values[1] else {
        return Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
            span,
        ));
    };
    let mut seen_paths = BTreeSet::new();
    let mut injected_relations = Vec::new();
    let mut injected_values = Vec::new();
    for item in &value_items.items {
        let Value::Pack(value_pack) = item else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_malformed_item",
                span,
            ));
        };
        let path = endpoint_boundary_value_path(&value_pack.fields, span)?;
        if !seen_paths.insert(path.to_string()) {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_duplicate_path",
                span,
            ));
        }
        let Some(variable) = path_to_variable.get(path) else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_unknown_path",
                span,
            ));
        };
        let boundary = endpoint_boundary_value_number(&value_pack.fields, span)?;
        injected_relations.push(make_relation_pack(
            endpoint_formula_math(variable),
            endpoint_formula_math(&boundary.formula_text),
        ));
        let mut injected = BTreeMap::new();
        injected.insert("변수".to_string(), Value::Str(variable.clone()));
        injected.insert("경로".to_string(), Value::Str(path.to_string()));
        injected.insert("값".to_string(), boundary.value);
        if let Some(dim) = boundary.unit_dim {
            injected.insert("단위차원".to_string(), Value::Str(format_dim(dim)));
        }
        if let Some(symbol) = boundary.unit_symbol {
            injected.insert("단위기호".to_string(), Value::Str(symbol));
        }
        injected_values.push(Value::Pack(PackValue { fields: injected }));
    }

    let mut relations = original_relations.items.clone();
    relations.extend(injected_relations.clone());
    Ok(EndpointBoundaryValueInjection {
        relations,
        injected_relations,
        injected_values,
        mappings,
    })
}

fn endpoint_boundary_value_path<'a>(
    fields: &'a BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<&'a str, RuntimeError> {
    match fields.get("경로") {
        Some(Value::Str(path)) => Ok(path.as_str()),
        _ => Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
            span,
        )),
    }
}

fn endpoint_boundary_value_number(
    fields: &BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<EndpointBoundaryNumber, RuntimeError> {
    match fields.get("값") {
        Some(Value::Num(quantity)) => Ok(EndpointBoundaryNumber {
            value: Value::Num(quantity.clone()),
            formula_text: quantity.raw.format(),
            unit_dim: (!quantity.dim.is_dimensionless()).then_some(quantity.dim),
            unit_symbol: (!quantity.dim.is_dimensionless()).then(|| format_dim(quantity.dim)),
        }),
        Some(_) => Err(endpoint_boundary_value_error(
            "connect_boundary_value_non_numeric",
            span,
        )),
        None => Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
            span,
        )),
    }
}

fn endpoint_unit_components(
    formula_set: &PackValue,
    ordered_mapping: &[(String, String)],
    span: crate::lang::span::Span,
) -> Result<BTreeMap<String, EndpointUnitSeed>, RuntimeError> {
    let kind = match formula_set.fields.get("__이음관계종류") {
        Some(Value::Str(kind)) => kind.as_str(),
        _ => "",
    };
    if kind != "endpoint_formula_relation_set_with_values" {
        return Ok(BTreeMap::new());
    }

    let known_variables: BTreeSet<String> = ordered_mapping
        .iter()
        .map(|(variable, _)| variable.clone())
        .collect();
    let mut parent: BTreeMap<String, String> = known_variables
        .iter()
        .map(|variable| (variable.clone(), variable.clone()))
        .collect();

    if let Ok(relations) = endpoint_relation_list_field(&formula_set.fields, "관계들", span) {
        for relation in &relations.items {
            let Value::Pack(relation_pack) = relation else {
                continue;
            };
            let mut vars = BTreeSet::new();
            if let Some(Value::Math(left)) = relation_pack.fields.get(RELATION_LEFT_FIELD) {
                for variable in endpoint_variables_in_formula(&relation_formula_text(left, span)?) {
                    if known_variables.contains(&variable) {
                        vars.insert(variable);
                    }
                }
            }
            if let Some(Value::Math(right)) = relation_pack.fields.get(RELATION_RIGHT_FIELD) {
                for variable in endpoint_variables_in_formula(&relation_formula_text(right, span)?)
                {
                    if known_variables.contains(&variable) {
                        vars.insert(variable);
                    }
                }
            }
            let mut vars = vars.into_iter();
            if let Some(first) = vars.next() {
                for variable in vars {
                    endpoint_union(&mut parent, &first, &variable);
                }
            }
        }
    }

    let mut component_units: BTreeMap<String, UnitDim> = BTreeMap::new();
    if let Some(Value::List(injected)) = formula_set.fields.get("주입값들") {
        for item in &injected.items {
            let Value::Pack(pack) = item else {
                continue;
            };
            let variable = endpoint_relation_string_field(&pack.fields, "변수", span)?;
            let Some(Value::Num(quantity)) = pack.fields.get("값") else {
                continue;
            };
            if quantity.dim.is_dimensionless() {
                continue;
            }
            let root = endpoint_find(&mut parent, variable);
            if let Some(existing) = component_units.get(&root).copied() {
                if existing != quantity.dim {
                    let marker = if endpoint_currency_pair(existing, quantity.dim) {
                        "connect_unit_boundary_incompatible_unit"
                    } else {
                        "connect_unit_boundary_dim_conflict"
                    };
                    return Err(endpoint_boundary_value_error(marker, span));
                }
            } else {
                component_units.insert(root, quantity.dim);
            }
        }
    }

    let mut by_variable = BTreeMap::new();
    for variable in known_variables {
        let root = endpoint_find(&mut parent, &variable);
        if let Some(dim) = component_units.get(&root).copied() {
            by_variable.insert(variable, EndpointUnitSeed { dim });
        }
    }
    Ok(by_variable)
}

fn endpoint_variables_in_formula(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 6 <= bytes.len() {
        if &bytes[i..i + 3] == b"ep_"
            && bytes[i + 3].is_ascii_digit()
            && bytes[i + 4].is_ascii_digit()
            && bytes[i + 5].is_ascii_digit()
        {
            out.push(text[i..i + 6].to_string());
            i += 6;
        } else {
            i += 1;
        }
    }
    out
}

fn endpoint_find(parent: &mut BTreeMap<String, String>, variable: &str) -> String {
    let current = parent
        .get(variable)
        .cloned()
        .unwrap_or_else(|| variable.to_string());
    if current == variable {
        return current;
    }
    let root = endpoint_find(parent, &current);
    parent.insert(variable.to_string(), root.clone());
    root
}

fn endpoint_union(parent: &mut BTreeMap<String, String>, left: &str, right: &str) {
    let left_root = endpoint_find(parent, left);
    let right_root = endpoint_find(parent, right);
    if left_root != right_root {
        parent.insert(right_root, left_root);
    }
}

fn endpoint_currency_pair(left: UnitDim, right: UnitDim) -> bool {
    let krw = UnitDim {
        length: 0,
        time: 0,
        mass: 0,
        angle: 0,
        pixel: 0,
        temperature: 0,
        krw: 1,
        usd: 0,
    };
    let usd = UnitDim {
        length: 0,
        time: 0,
        mass: 0,
        angle: 0,
        pixel: 0,
        temperature: 0,
        krw: 0,
        usd: 1,
    };
    (left == krw && right == usd) || (left == usd && right == krw)
}

fn endpoint_solve_numeric_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Fixed64, RuntimeError> {
    if let Value::Num(quantity) = value {
        if quantity.dim.is_dimensionless() {
            return Ok(quantity.raw);
        }
    }
    let Some(exact) = exact_text_from_value(value, span)? else {
        return Err(endpoint_boundary_value_error(
            "connect_boundary_value_non_numeric",
            span,
        ));
    };
    let num = exact
        .num
        .parse::<i64>()
        .map_err(|_| endpoint_boundary_value_error("connect_boundary_value_non_numeric", span))?;
    let den = exact
        .den
        .parse::<i64>()
        .map_err(|_| endpoint_boundary_value_error("connect_boundary_value_non_numeric", span))?;
    Ok(Fixed64::from_ratio(num, den))
}

fn endpoint_boundary_value_error(
    marker: &'static str,
    span: crate::lang::span::Span,
) -> RuntimeError {
    RuntimeError::Pack {
        message: marker.to_string(),
        span,
    }
}

struct EndpointRangeBound {
    path: String,
    min: Option<Value>,
    max: Option<Value>,
    min_inclusive: bool,
    max_inclusive: bool,
}

struct EndpointBoundaryRangeCheck {
    violations: Vec<Value>,
    range_count: usize,
}

fn eval_endpoint_boundary_range_violation_list(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let checked = endpoint_boundary_range_check(values, span)?;
    Ok(Value::List(ListValue {
        items: checked.violations,
    }))
}

fn eval_endpoint_boundary_range_check(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let checked = endpoint_boundary_range_check(values, span)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::Str("endpoint_range_check".to_string()),
    );
    fields.insert(
        "검사결과".to_string(),
        Value::Str(if checked.violations.is_empty() {
            "통과".to_string()
        } else {
            "실패".to_string()
        }),
    );
    fields.insert(
        "범위개수".to_string(),
        Value::Num(Quantity::new(
            Fixed64::from_int(checked.range_count as i64),
            UnitDim::zero(),
        )),
    );
    fields.insert(
        "위반개수".to_string(),
        Value::Num(Quantity::new(
            Fixed64::from_int(checked.violations.len() as i64),
            UnitDim::zero(),
        )),
    );
    fields.insert(
        "위반들".to_string(),
        Value::List(ListValue {
            items: checked.violations,
        }),
    );
    Ok(Value::Pack(PackValue { fields }))
}

fn endpoint_boundary_range_check(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<EndpointBoundaryRangeCheck, RuntimeError> {
    if values.len() != 2 {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
            span,
        ));
    }
    let solve_result = expect_endpoint_solve_result_value(&values[0], span)?;
    let known_paths = endpoint_solve_result_mapping_paths(solve_result, span)?;
    let value_by_path = endpoint_solve_result_values_by_path(solve_result, span)?;
    let ranges = endpoint_range_bounds(&values[1], span)?;

    let mut violations = Vec::new();
    for range in &ranges {
        if !known_paths.contains(&range.path) {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_unknown_path",
                span,
            ));
        }
        let Some(value) = value_by_path.get(&range.path) else {
            let mut fields = endpoint_range_violation_base(&range.path, "missing_value");
            if let Some(min) = &range.min {
                fields.insert("하한".to_string(), min.clone());
            }
            if let Some(max) = &range.max {
                fields.insert("상한".to_string(), max.clone());
            }
            violations.push(Value::Pack(PackValue { fields }));
            continue;
        };
        if let Some(min) = &range.min {
            let ord = endpoint_compare_range_numbers(value, min, span)?;
            let below = if range.min_inclusive {
                ord == std::cmp::Ordering::Less
            } else {
                ord != std::cmp::Ordering::Greater
            };
            if below {
                let mut fields = endpoint_range_violation_base(&range.path, "below_min");
                fields.insert("값".to_string(), value.clone());
                fields.insert("하한".to_string(), min.clone());
                if let Some(max) = &range.max {
                    fields.insert("상한".to_string(), max.clone());
                }
                violations.push(Value::Pack(PackValue { fields }));
                continue;
            }
        }
        if let Some(max) = &range.max {
            let ord = endpoint_compare_range_numbers(value, max, span)?;
            let above = if range.max_inclusive {
                ord == std::cmp::Ordering::Greater
            } else {
                ord != std::cmp::Ordering::Less
            };
            if above {
                let mut fields = endpoint_range_violation_base(&range.path, "above_max");
                fields.insert("값".to_string(), value.clone());
                if let Some(min) = &range.min {
                    fields.insert("하한".to_string(), min.clone());
                }
                fields.insert("상한".to_string(), max.clone());
                violations.push(Value::Pack(PackValue { fields }));
            }
        }
    }
    Ok(EndpointBoundaryRangeCheck {
        violations,
        range_count: ranges.len(),
    })
}

fn expect_endpoint_solve_result_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<&PackValue, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
            span,
        ));
    };
    match pack.fields.get("__이음관계종류") {
        Some(Value::Str(kind)) if kind == "endpoint_solve_result" => Ok(pack),
        _ => Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
            span,
        )),
    }
}

fn endpoint_solve_result_mapping_paths(
    solve_result: &PackValue,
    span: crate::lang::span::Span,
) -> Result<BTreeSet<String>, RuntimeError> {
    let mappings = endpoint_relation_list_field(&solve_result.fields, "변수사상", span)?;
    let mut out = BTreeSet::new();
    for item in &mappings.items {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
                span,
            ));
        };
        out.insert(endpoint_relation_string_field(&pack.fields, "경로", span)?.to_string());
    }
    Ok(out)
}

fn endpoint_solve_result_values_by_path(
    solve_result: &PackValue,
    span: crate::lang::span::Span,
) -> Result<BTreeMap<String, Value>, RuntimeError> {
    let values = endpoint_relation_list_field(&solve_result.fields, "값들", span)?;
    let mut out = BTreeMap::new();
    for item in &values.items {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
                span,
            ));
        };
        let path = endpoint_relation_string_field(&pack.fields, "경로", span)?;
        let Some(value) = pack.fields.get("값") else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
                span,
            ));
        };
        out.insert(path.to_string(), value.clone());
    }
    Ok(out)
}

fn endpoint_range_bounds(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Vec<EndpointRangeBound>, RuntimeError> {
    let Value::List(items) = value else {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
            span,
        ));
    };
    let mut seen_paths = BTreeSet::new();
    let mut ranges = Vec::new();
    for item in &items.items {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_malformed_item",
                span,
            ));
        };
        let path = endpoint_range_path(&pack.fields, span)?.to_string();
        if !seen_paths.insert(path.clone()) {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_duplicate_path",
                span,
            ));
        }
        let min = endpoint_optional_range_number(&pack.fields, "최소", span)?;
        let max = endpoint_optional_range_number(&pack.fields, "최대", span)?;
        if min.is_none() && max.is_none() {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_malformed_item",
                span,
            ));
        }
        let min_inclusive = endpoint_optional_bool(&pack.fields, "최소포함", true, span)?;
        let max_inclusive = endpoint_optional_bool(&pack.fields, "최대포함", true, span)?;
        ranges.push(EndpointRangeBound {
            path,
            min,
            max,
            min_inclusive,
            max_inclusive,
        });
    }
    Ok(ranges)
}

fn endpoint_range_path<'a>(
    fields: &'a BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<&'a str, RuntimeError> {
    match fields.get("경로") {
        Some(Value::Str(path)) => Ok(path.as_str()),
        _ => Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
            span,
        )),
    }
}

fn endpoint_optional_range_number(
    fields: &BTreeMap<String, Value>,
    field: &str,
    span: crate::lang::span::Span,
) -> Result<Option<Value>, RuntimeError> {
    match fields.get(field) {
        Some(Value::Num(_)) => Ok(fields.get(field).cloned()),
        Some(_) => Err(endpoint_boundary_range_error(
            "connect_boundary_range_non_numeric",
            span,
        )),
        None => Ok(None),
    }
}

fn endpoint_optional_bool(
    fields: &BTreeMap<String, Value>,
    field: &str,
    default: bool,
    span: crate::lang::span::Span,
) -> Result<bool, RuntimeError> {
    match fields.get(field) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
            span,
        )),
        None => Ok(default),
    }
}

fn endpoint_compare_range_numbers(
    left: &Value,
    right: &Value,
    span: crate::lang::span::Span,
) -> Result<std::cmp::Ordering, RuntimeError> {
    let left = endpoint_range_numeric_quantity(left, span)?;
    let right = endpoint_range_numeric_quantity(right, span)?;
    if left.dim != right.dim {
        let marker = if endpoint_currency_pair(left.dim, right.dim) {
            "connect_boundary_range_incompatible_unit"
        } else {
            "connect_boundary_range_dim_conflict"
        };
        return Err(endpoint_boundary_range_error(marker, span));
    }
    Ok(left.raw.cmp(&right.raw))
}

fn endpoint_range_numeric_quantity(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Quantity, RuntimeError> {
    match value {
        Value::Num(quantity) => Ok(quantity.clone()),
        _ => {
            let Some(exact) = exact_text_from_value(value, span)? else {
                return Err(endpoint_boundary_range_error(
                    "connect_boundary_range_non_numeric",
                    span,
                ));
            };
            let num = exact.num.parse::<i64>().map_err(|_| {
                endpoint_boundary_range_error("connect_boundary_range_non_numeric", span)
            })?;
            let den = exact.den.parse::<i64>().map_err(|_| {
                endpoint_boundary_range_error("connect_boundary_range_non_numeric", span)
            })?;
            Ok(Quantity::new(
                Fixed64::from_ratio(num, den),
                UnitDim::zero(),
            ))
        }
    }
}

fn endpoint_range_violation_base(path: &str, reason: &str) -> BTreeMap<String, Value> {
    let mut fields = BTreeMap::new();
    fields.insert("경로".to_string(), Value::Str(path.to_string()));
    fields.insert("이유".to_string(), Value::Str(reason.to_string()));
    fields
}

fn endpoint_boundary_range_error(
    marker: &'static str,
    span: crate::lang::span::Span,
) -> RuntimeError {
    RuntimeError::Pack {
        message: marker.to_string(),
        span,
    }
}

fn expect_relation_solve_result(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<&PackValue, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(RuntimeError::TypeMismatch {
            expected: "relation solve result",
            span,
        });
    };
    relation_solve_result_kind(&pack.fields, span)?;
    Ok(pack)
}

fn relation_solve_result_kind<'a>(
    fields: &'a BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<&'a str, RuntimeError> {
    match fields.get(RELATION_SOLVE_RESULT_KIND_FIELD) {
        Some(Value::Str(kind)) => Ok(kind.as_str()),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "relation solve result kind",
            span,
        }),
    }
}

fn endpoint_formula_relation_bridge(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<EndpointFormulaRelationBridge, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation",
            span,
        });
    }
    let mut flat_relations = Vec::new();
    flatten_endpoint_relation_value(&values[0], span, &mut flat_relations)?;

    let mut variables: BTreeMap<String, String> = BTreeMap::new();
    let mut mappings = Vec::new();
    let mut relation_packs = Vec::new();
    for relation in flat_relations {
        let Value::Pack(pack) = relation else {
            return Err(RuntimeError::TypeMismatch {
                expected: "endpoint connect relation",
                span,
            });
        };
        match endpoint_relation_kind(&pack.fields, span)? {
            "endpoint_equality" => {
                let left = endpoint_formula_variable_for_field(
                    &pack.fields,
                    "왼쪽",
                    span,
                    &mut variables,
                    &mut mappings,
                )?;
                let right = endpoint_formula_variable_for_field(
                    &pack.fields,
                    "오른쪽",
                    span,
                    &mut variables,
                    &mut mappings,
                )?;
                relation_packs.push(make_relation_pack(
                    endpoint_formula_math(&left),
                    endpoint_formula_math(&right),
                ));
            }
            "endpoint_flow" => {
                let convention = endpoint_relation_string_field(&pack.fields, "부호규약", span)?;
                if convention != "left_plus_right_zero" {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "endpoint flow left_plus_right_zero convention",
                        span,
                    });
                }
                let left = endpoint_formula_variable_for_field(
                    &pack.fields,
                    "왼쪽",
                    span,
                    &mut variables,
                    &mut mappings,
                )?;
                let right = endpoint_formula_variable_for_field(
                    &pack.fields,
                    "오른쪽",
                    span,
                    &mut variables,
                    &mut mappings,
                )?;
                relation_packs.push(make_relation_pack(
                    endpoint_formula_math(&format!("{left} + {right}")),
                    endpoint_formula_math("0"),
                ));
            }
            "endpoint_carried_property" => {
                return Err(RuntimeError::TypeMismatch {
                    expected: "solver-compatible endpoint equality or flow relation (endpoint_carried_property unsupported)",
                    span,
                });
            }
            _ => {
                return Err(RuntimeError::TypeMismatch {
                    expected: "solver-compatible endpoint relation",
                    span,
                });
            }
        }
    }

    Ok(EndpointFormulaRelationBridge {
        relations: relation_packs,
        mappings,
    })
}

fn flatten_endpoint_relation_value(
    value: &Value,
    span: crate::lang::span::Span,
    out: &mut Vec<Value>,
) -> Result<(), RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation",
            span,
        });
    };
    match endpoint_relation_kind(&pack.fields, span)? {
        "endpoint_equality" | "endpoint_flow" | "endpoint_carried_property" => {
            out.push(value.clone());
            Ok(())
        }
        "endpoint_relation_set" => {
            let relations = endpoint_relation_list_field(&pack.fields, "관계들", span)?;
            for item in &relations.items {
                flatten_endpoint_relation_value(item, span, out)?;
            }
            Ok(())
        }
        "endpoint_relation_flat_set" => {
            let relations = endpoint_relation_list_field(&pack.fields, "관계들", span)?;
            for item in &relations.items {
                flatten_endpoint_relation_value(item, span, out)?;
            }
            Ok(())
        }
        "endpoint_statement_set" => {
            let statements = endpoint_relation_list_field(&pack.fields, "이음들", span)?;
            for item in &statements.items {
                flatten_endpoint_relation_value(item, span, out)?;
            }
            Ok(())
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "supported endpoint connect relation",
            span,
        }),
    }
}

fn endpoint_formula_variable_for_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
    span: crate::lang::span::Span,
    variables: &mut BTreeMap<String, String>,
    mappings: &mut Vec<Value>,
) -> Result<String, RuntimeError> {
    let path = endpoint_relation_string_field(fields, field, span)?;
    if let Some(variable) = variables.get(path) {
        return Ok(variable.clone());
    }
    let variable = format!("ep_{:03}", variables.len() + 1);
    variables.insert(path.to_string(), variable.clone());
    let mut mapping = BTreeMap::new();
    mapping.insert("변수".to_string(), Value::Str(variable.clone()));
    mapping.insert("경로".to_string(), Value::Str(path.to_string()));
    mappings.push(Value::Pack(PackValue { fields: mapping }));
    Ok(variable)
}

fn endpoint_relation_string_field<'a>(
    fields: &'a BTreeMap<String, Value>,
    field: &str,
    span: crate::lang::span::Span,
) -> Result<&'a str, RuntimeError> {
    match fields.get(field) {
        Some(Value::Str(value)) => Ok(value.as_str()),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation string field",
            span,
        }),
    }
}

fn endpoint_formula_math(body: &str) -> crate::core::value::MathValue {
    crate::core::value::MathValue {
        dialect: FormulaDialect::Ascii.tag().to_string(),
        body: body.to_string(),
    }
}

fn endpoint_relation_kind<'a>(
    fields: &'a BTreeMap<String, Value>,
    span: crate::lang::span::Span,
) -> Result<&'a str, RuntimeError> {
    match fields.get("__이음관계종류") {
        Some(Value::Str(kind)) => Ok(kind.as_str()),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation",
            span,
        }),
    }
}

fn endpoint_relation_list_field<'a>(
    fields: &'a BTreeMap<String, Value>,
    field: &str,
    span: crate::lang::span::Span,
) -> Result<&'a ListValue, RuntimeError> {
    match fields.get(field) {
        Some(Value::List(list)) => Ok(list),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "endpoint connect relation list",
            span,
        }),
    }
}

fn make_relation_solve_success_bindings(bindings: BTreeMap<String, Value>) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
        Value::Str(RELATION_SOLVE_RESULT_SUCCESS.to_string()),
    );
    if bindings.len() == 1 {
        if let Some((variable, value)) = bindings.iter().next() {
            fields.insert(
                RELATION_SOLVE_VAR_FIELD.to_string(),
                Value::Str(variable.clone()),
            );
            fields.insert(RELATION_SOLVE_VALUE_FIELD.to_string(), value.clone());
        }
    }
    fields.insert(
        RELATION_SOLVE_BINDINGS_FIELD.to_string(),
        Value::Pack(PackValue { fields: bindings }),
    );
    Value::Pack(PackValue { fields })
}

fn make_relation_solve_failure(reason: &str) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
        Value::Str(RELATION_SOLVE_RESULT_FAILURE.to_string()),
    );
    fields.insert(
        RELATION_SOLVE_REASON_FIELD.to_string(),
        Value::Str(reason.to_string()),
    );
    Value::Pack(PackValue { fields })
}

fn expect_single_equation_relation(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<PackValue, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(RuntimeError::TypeMismatch {
            expected: "equation relation",
            span,
        });
    };
    match pack.fields.get(RELATION_KIND_FIELD) {
        Some(Value::Str(kind)) if kind == RELATION_KIND_EQUATION => {}
        _ => {
            return Err(RuntimeError::TypeMismatch {
                expected: "equation relation",
                span,
            });
        }
    }
    match (
        pack.fields.get(RELATION_LEFT_FIELD),
        pack.fields.get(RELATION_RIGHT_FIELD),
    ) {
        (Some(Value::Math(_)), Some(Value::Math(_))) => Ok(pack.clone()),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "equation relation",
            span,
        }),
    }
}

fn expect_equation_relations(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Vec<PackValue>, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "equation relation",
            span,
        });
    }
    match &values[0] {
        Value::Pack(_) => Ok(vec![expect_single_equation_relation(&values[0], span)?]),
        Value::List(list) => {
            if list.items.len() != 2 {
                return Err(RuntimeError::TypeMismatch {
                    expected: "2-equation list",
                    span,
                });
            }
            let mut out = Vec::with_capacity(list.items.len());
            for item in &list.items {
                out.push(expect_single_equation_relation(item, span)?);
            }
            Ok(out)
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "equation relation",
            span,
        }),
    }
}

fn relation_formula_text(
    math: &crate::core::value::MathValue,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    let dialect = FormulaDialect::from_tag(&math.dialect).ok_or(RuntimeError::TypeMismatch {
        expected: "formula",
        span,
    })?;
    if !matches!(dialect, FormulaDialect::Ascii | FormulaDialect::Ascii1) {
        return Err(RuntimeError::Pack {
            message: "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE #ascii 수식만 지원합니다".to_string(),
            span,
        });
    }
    let analysis =
        analyze_formula(&math.body, dialect).map_err(|err| map_formula_error(err, span))?;
    Ok(match analysis.assign_name {
        Some(assign_name) => format!("({assign_name}) - ({})", analysis.expr_text),
        None => analysis.expr_text,
    })
}

fn relation_binding_value_from_text(
    binding: &ddonirang_symbolic::SolveBinding,
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let exact = ddonirang_numeric::ExactText {
        num: binding.numerator.clone(),
        den: binding.denominator.clone(),
        kind: Some(if binding.denominator == "1" {
            NUMERIC_KIND_BIG_INT.to_string()
        } else {
            NUMERIC_KIND_RATIONAL.to_string()
        }),
    };
    exact_text_to_value(exact, "=", span)
}

fn eval_relation_solve_result(
    relations: &[PackValue],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    // `=:= relation` is the relation input surface; `방정식풀기` remains a bounded solve line.
    // Current dispatch only covers single-equation solve and 2식 2미지수 exact system solve.
    let outcome = match relations.len() {
        1 => {
            let left = match relations[0].fields.get(RELATION_LEFT_FIELD) {
                Some(Value::Math(value)) => value,
                _ => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "equation relation",
                        span,
                    })
                }
            };
            let right = match relations[0].fields.get(RELATION_RIGHT_FIELD) {
                Some(Value::Math(value)) => value,
                _ => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "equation relation",
                        span,
                    })
                }
            };
            let left_text = relation_formula_text(left, span)?;
            let right_text = relation_formula_text(right, span)?;
            ddonirang_symbolic::solve_relation_equation(&left_text, &right_text)
        }
        2 => {
            let pairs = relations
                .iter()
                .map(|relation| {
                    let left = match relation.fields.get(RELATION_LEFT_FIELD) {
                        Some(Value::Math(value)) => value,
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "equation relation",
                                span,
                            })
                        }
                    };
                    let right = match relation.fields.get(RELATION_RIGHT_FIELD) {
                        Some(Value::Math(value)) => value,
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "equation relation",
                                span,
                            })
                        }
                    };
                    Ok((
                        relation_formula_text(left, span)?,
                        relation_formula_text(right, span)?,
                    ))
                })
                .collect::<Result<Vec<_>, RuntimeError>>()?;
            ddonirang_symbolic::solve_relation_system(&pairs)
        }
        _ => Err("E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE relation solve arity".to_string()),
    };
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(err) if err.starts_with("E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE") => {
            return Ok(make_relation_solve_failure("unsupported"));
        }
        Err(err) => return Err(RuntimeError::Pack { message: err, span }),
    };

    Ok(match outcome {
        ddonirang_symbolic::RelationSolveOutcome::Solution(solution) => {
            let mut bindings = BTreeMap::new();
            for (variable, binding) in solution {
                bindings.insert(variable, relation_binding_value_from_text(&binding, span)?);
            }
            make_relation_solve_success_bindings(bindings)
        }
        ddonirang_symbolic::RelationSolveOutcome::NoSolution => {
            make_relation_solve_failure("no_solution")
        }
        ddonirang_symbolic::RelationSolveOutcome::NonUnique => {
            make_relation_solve_failure("non_unique")
        }
    })
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

fn lambda_param_names(param: &str) -> Vec<String> {
    let names: Vec<String> = param
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect();
    if names.is_empty() {
        vec![param.trim().to_string()]
    } else {
        names
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

fn eval_symbolic_assertion_bridge(
    assertion: &AssertionValue,
    values: &PackValue,
    span: crate::lang::span::Span,
) -> Result<Option<bool>, RuntimeError> {
    if let Some((lhs, rhs)) = parse_symbolic_equivalence_assertion(&assertion.body_source) {
        let cert = ddonirang_symbolic::prove_equivalent(&lhs, &rhs).map_err(|err| {
            RuntimeError::FormulaParse {
                message: format!("E_SYMBOLIC_PROOF_UNSUPPORTED: {err}"),
                span,
            }
        })?;
        let cert_value = serde_json::to_value(&cert).map_err(|err| RuntimeError::Pack {
            message: format!("proof certificate serialize failed: {err}"),
            span,
        })?;
        let report =
            ddonirang_proof::verify_value(&cert_value).map_err(|err| RuntimeError::Pack {
                message: format!("proof verify failed: {err}"),
                span,
            })?;
        return Ok(Some(report.valid && cert.equivalent));
    }
    let Some((lhs, rhs)) = parse_symbolic_relation_assertion(&assertion.body_source) else {
        return Ok(None);
    };
    let bindings = solve_bindings_from_pack(values, span)?;
    Ok(Some(
        ddonirang_symbolic::relation_holds(&lhs, &rhs, &bindings).map_err(|err| {
            RuntimeError::FormulaParse {
                message: format!("E_SYMBOLIC_PROOF_UNSUPPORTED: {err}"),
                span,
            }
        })?,
    ))
}

fn parse_symbolic_equivalence_assertion(raw: &str) -> Option<(String, String)> {
    let (lhs, rest) = parse_ascii_formula_prefix(raw)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let (rhs, rest) = parse_ascii_formula_prefix(rest)?;
    let rest = rest.trim();
    if !rest.is_empty() && rest != "." {
        return None;
    }
    Some((lhs, rhs))
}

fn parse_symbolic_relation_assertion(raw: &str) -> Option<(String, String)> {
    let (lhs, rest) = parse_ascii_formula_prefix(raw)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix("=:=")?.trim_start();
    let (rhs, rest) = parse_ascii_formula_prefix(rest)?;
    let rest = rest.trim();
    if !rest.is_empty() && rest != "." {
        return None;
    }
    Some((lhs, rhs))
}

fn solve_binding_from_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<ddonirang_symbolic::SolveBinding, RuntimeError> {
    let Some(exact) = exact_text_from_value(value, span)? else {
        return Err(type_mismatch_detail("exact numeric", value, span));
    };
    Ok(ddonirang_symbolic::SolveBinding {
        numerator: exact.num,
        denominator: exact.den,
    })
}

fn solve_bindings_from_pack(
    pack: &PackValue,
    span: crate::lang::span::Span,
) -> Result<BTreeMap<String, ddonirang_symbolic::SolveBinding>, RuntimeError> {
    let mut out = BTreeMap::new();
    for (name, value) in &pack.fields {
        out.insert(name.clone(), solve_binding_from_value(value, span)?);
    }
    Ok(out)
}

fn extract_relation_texts_from_value(
    value: &Value,
    span: crate::lang::span::Span,
) -> Result<Vec<(String, String)>, RuntimeError> {
    match value {
        Value::Pack(_) => {
            let relation = expect_single_equation_relation(value, span)?;
            Ok(vec![(
                relation_formula_text(
                    match relation.fields.get(RELATION_LEFT_FIELD) {
                        Some(Value::Math(value)) => value,
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "equation relation",
                                span,
                            })
                        }
                    },
                    span,
                )?,
                relation_formula_text(
                    match relation.fields.get(RELATION_RIGHT_FIELD) {
                        Some(Value::Math(value)) => value,
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "equation relation",
                                span,
                            })
                        }
                    },
                    span,
                )?,
            )])
        }
        Value::List(list) => {
            let mut out = Vec::with_capacity(list.items.len());
            for item in &list.items {
                out.extend(extract_relation_texts_from_value(item, span)?);
            }
            Ok(out)
        }
        other => Err(type_mismatch_detail("equation relation", other, span)),
    }
}

fn parse_ascii_formula_prefix(raw: &str) -> Option<(String, &str)> {
    let mut rest = raw.trim_start();
    rest = rest.strip_prefix('(')?.trim_start();
    rest = rest.strip_prefix("#ascii")?.trim_start();
    rest = rest.strip_prefix(')')?.trim_start();
    rest = rest.strip_prefix("수식")?.trim_start();
    rest = rest.strip_prefix('{')?;
    let mut depth = 1usize;
    for (idx, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let body = rest[..idx].trim().to_string();
                    let tail = &rest[idx + ch.len_utf8()..];
                    return Some((body, tail));
                }
            }
            _ => {}
        }
    }
    None
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

fn expect_single_formula(
    values: &[Value],
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<crate::core::value::MathValue, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: label,
            span,
        });
    }
    match &values[0] {
        Value::Math(value) => Ok(value.clone()),
        value => Err(type_mismatch_detail("formula", value, span)),
    }
}

fn expect_two_formulas(
    values: &[Value],
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<(crate::core::value::MathValue, crate::core::value::MathValue), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: label,
            span,
        });
    }
    let left = match &values[0] {
        Value::Math(value) => value.clone(),
        value => return Err(type_mismatch_detail("formula", value, span)),
    };
    let right = match &values[1] {
        Value::Math(value) => value.clone(),
        value => return Err(type_mismatch_detail("formula", value, span)),
    };
    Ok((left, right))
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

fn transform_symbolic_formula_value(
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
                symbolic_formula_call_label(call_name)
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
    let needs_var = matches!(call_name, "diff" | "int");
    let var_name = if needs_var {
        let var_name = match options.var_name {
            Some(name) => {
                if !vars.contains(&name) {
                    return Err(RuntimeError::FormulaParse {
                        message: format!(
                            "E_CALC_FREEVAR_NOT_FOUND: {} 변수 '{}'가 수식에 없습니다",
                            symbolic_formula_call_label(call_name),
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
        var_name
    } else {
        String::new()
    };

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

    let transformed = match call_name {
        "simplify" => ddonirang_symbolic::simplify(&analysis.expr_text),
        "expand" => ddonirang_symbolic::expand(&analysis.expr_text),
        "factor" => ddonirang_symbolic::factor(&analysis.expr_text),
        "diff" => {
            let mut out = analysis.expr_text.clone();
            let order = options.order.unwrap_or(1);
            for _ in 0..order {
                out = ddonirang_symbolic::diff(&out, &var_name)
                    .map_err(|err| symbolic_formula_error(call_name, err, span))?;
            }
            Ok(out)
        }
        "int" => ddonirang_symbolic::integrate(&analysis.expr_text, &var_name),
        _ => Err(format!("E_SYMBOLIC_UNKNOWN_TRANSFORM {call_name}")),
    };
    let mut body = transformed.map_err(|err| symbolic_formula_error(call_name, err, span))?;
    if call_name == "int" && options.include_const.unwrap_or(false) {
        body = format!("{body} + C");
    }
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

fn symbolic_formula_call_label(call_name: &str) -> &'static str {
    match call_name {
        "simplify" => "정리하기",
        "expand" => "전개하기",
        "factor" => "인수분해하기",
        "diff" => "미분하기",
        "int" => "적분하기",
        _ => "수식변환",
    }
}

fn symbolic_formula_error(
    call_name: &str,
    message: String,
    span: crate::lang::span::Span,
) -> RuntimeError {
    RuntimeError::FormulaParse {
        message: format!(
            "E_SYMBOLIC_FORMULA_UNSUPPORTED: {} ({})",
            symbolic_formula_call_label(call_name),
            message
        ),
        span,
    }
}

fn symbolic_formulas_equivalent(
    left: &crate::core::value::MathValue,
    right: &crate::core::value::MathValue,
    span: crate::lang::span::Span,
) -> Result<bool, RuntimeError> {
    let Some(left_dialect) = FormulaDialect::from_tag(&left.dialect) else {
        return Err(RuntimeError::FormulaParse {
            message: format!("알 수 없는 수식 방언: {}", left.dialect),
            span,
        });
    };
    let Some(right_dialect) = FormulaDialect::from_tag(&right.dialect) else {
        return Err(RuntimeError::FormulaParse {
            message: format!("알 수 없는 수식 방언: {}", right.dialect),
            span,
        });
    };
    if left_dialect != FormulaDialect::Ascii || right_dialect != FormulaDialect::Ascii {
        return Err(RuntimeError::FormulaParse {
            message: "동치인가는 #ascii 수식만 지원합니다".to_string(),
            span,
        });
    }
    let left_analysis =
        analyze_formula(&left.body, left_dialect).map_err(|err| map_formula_error(err, span))?;
    let right_analysis =
        analyze_formula(&right.body, right_dialect).map_err(|err| map_formula_error(err, span))?;
    ddonirang_symbolic::equivalent(&left_analysis.expr_text, &right_analysis.expr_text)
        .map_err(|err| symbolic_formula_error("equiv", err, span))
}

fn eval_symbolic_proof_tactic(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<PackValue, RuntimeError> {
    if values.len() != 1 {
        return Err(RuntimeError::TypeMismatch {
            expected: "proof tactic pack",
            span,
        });
    }
    let Value::Pack(pack) = &values[0] else {
        return Err(type_mismatch_detail("pack", &values[0], span));
    };
    if let Some(relation_value) = find_pack_value(pack, &["관계", "relation", "식", "equation"])
    {
        let relations = extract_relation_texts_from_value(relation_value, span)?;
        if let Some(bindings_value) = find_pack_value(pack, &["해", "bindings", "값들"]) {
            let Value::Pack(bindings_pack) = bindings_value else {
                return Err(type_mismatch_detail("묶음값", bindings_value, span));
            };
            let bindings = solve_bindings_from_pack(bindings_pack, span)?;
            let verified = ddonirang_symbolic::relation_system_holds(&relations, &bindings)
                .map_err(|err| symbolic_formula_error("equiv", err, span))?;
            let mut fields = BTreeMap::new();
            fields.insert("검증".to_string(), Value::Bool(verified));
            fields.insert(
                "종류".to_string(),
                Value::Str("relation_solve_consistency".to_string()),
            );
            fields.insert(
                "관계수".to_string(),
                Value::Num(Quantity::new(
                    Fixed64::from_int(relations.len() as i64),
                    UnitDim::zero(),
                )),
            );
            return Ok(PackValue { fields });
        }
        if relations.len() != 1 {
            return Err(RuntimeError::Pack {
                message: "증명하기 relation proof는 단일 관계만 지원합니다".to_string(),
                span,
            });
        }
        let cert = ddonirang_symbolic::prove_equivalent(&relations[0].0, &relations[0].1)
            .map_err(|err| symbolic_formula_error("equiv", err, span))?;
        let cert_value = serde_json::to_value(&cert).map_err(|err| RuntimeError::Pack {
            message: format!("proof certificate serialize failed: {err}"),
            span,
        })?;
        let report =
            ddonirang_proof::verify_value(&cert_value).map_err(|err| RuntimeError::Pack {
                message: format!("proof verify failed: {err}"),
                span,
            })?;
        let mut fields = BTreeMap::new();
        fields.insert(
            "검증".to_string(),
            Value::Bool(report.valid && cert.equivalent),
        );
        fields.insert(
            "종류".to_string(),
            Value::Str("relation_equivalence".to_string()),
        );
        fields.insert("증거".to_string(), Value::Str(cert.certificate_hash));
        fields.insert("왼쪽정본".to_string(), Value::Str(cert.lhs_canonical));
        fields.insert("오른쪽정본".to_string(), Value::Str(cert.rhs_canonical));
        return Ok(PackValue { fields });
    }
    let left = match find_pack_value(pack, &["왼쪽", "left", "lhs"]) {
        Some(Value::Math(value)) => value.clone(),
        Some(other) => return Err(type_mismatch_detail("formula", other, span)),
        None => {
            return Err(RuntimeError::Pack {
                message: "증명하기 왼쪽 수식이 없습니다".to_string(),
                span,
            })
        }
    };
    let right = match find_pack_value(pack, &["오른쪽", "right", "rhs"]) {
        Some(Value::Math(value)) => value.clone(),
        Some(other) => return Err(type_mismatch_detail("formula", other, span)),
        None => {
            return Err(RuntimeError::Pack {
                message: "증명하기 오른쪽 수식이 없습니다".to_string(),
                span,
            })
        }
    };
    let Some(left_dialect) = FormulaDialect::from_tag(&left.dialect) else {
        return Err(RuntimeError::FormulaParse {
            message: format!("알 수 없는 수식 방언: {}", left.dialect),
            span,
        });
    };
    let Some(right_dialect) = FormulaDialect::from_tag(&right.dialect) else {
        return Err(RuntimeError::FormulaParse {
            message: format!("알 수 없는 수식 방언: {}", right.dialect),
            span,
        });
    };
    if left_dialect != FormulaDialect::Ascii || right_dialect != FormulaDialect::Ascii {
        return Err(RuntimeError::FormulaParse {
            message: "증명하기는 #ascii 수식만 지원합니다".to_string(),
            span,
        });
    }
    let left_analysis =
        analyze_formula(&left.body, left_dialect).map_err(|err| map_formula_error(err, span))?;
    let right_analysis =
        analyze_formula(&right.body, right_dialect).map_err(|err| map_formula_error(err, span))?;
    let cert =
        ddonirang_symbolic::prove_equivalent(&left_analysis.expr_text, &right_analysis.expr_text)
            .map_err(|err| symbolic_formula_error("equiv", err, span))?;
    let cert_value = serde_json::to_value(&cert).map_err(|err| RuntimeError::Pack {
        message: format!("proof certificate serialize failed: {err}"),
        span,
    })?;
    let report = ddonirang_proof::verify_value(&cert_value).map_err(|err| RuntimeError::Pack {
        message: format!("proof verify failed: {err}"),
        span,
    })?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "검증".to_string(),
        Value::Bool(report.valid && cert.equivalent),
    );
    fields.insert(
        "종류".to_string(),
        Value::Str("symbolic_equivalence".to_string()),
    );
    fields.insert("증거".to_string(), Value::Str(cert.certificate_hash));
    fields.insert("왼쪽정본".to_string(), Value::Str(cert.lhs_canonical));
    fields.insert("오른쪽정본".to_string(), Value::Str(cert.rhs_canonical));
    Ok(PackValue { fields })
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

fn expect_numeric_bisection_args(
    values: &[Value],
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<
    (
        crate::core::value::MathValue,
        String,
        Quantity,
        Quantity,
        usize,
    ),
    RuntimeError,
> {
    if values.len() != 5 {
        return Err(RuntimeError::TypeMismatch {
            expected: "formula, var, lower, upper, iterations",
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
    let lower = expect_quantity_value(&values[2], span)?;
    let upper = expect_quantity_value(&values[3], span)?;
    let iterations = expect_quantity_value(&values[4], span)?;
    ensure_same_dim(&lower, &upper, span)?;
    ensure_dimensionless(&iterations, span)?;
    let iteration_count = fixed64_to_nonnegative_index(iterations.raw, span)?;
    if iteration_count == 0 {
        return Err(RuntimeError::MathDomain {
            message: "E_CALC_NUMERIC_BAD_ITERATION: 수치해.이분법 반복횟수는 1 이상이어야 합니다",
            span,
        });
    }
    Ok((math, var_name, lower, upper, iteration_count))
}

fn expect_polynomial_solve_args(
    values: &[Value],
    span: crate::lang::span::Span,
    label: &'static str,
) -> Result<(crate::core::value::MathValue, String), RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "formula, var",
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
    Ok((math, var_name))
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

fn eval_polynomial_solve_result(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    let (math, var_name) = expect_polynomial_solve_args(values, span, "다항식.풀기")?;
    let _prepared = prepare_numeric_formula(&math, &var_name, span, "다항식.풀기")?;
    let zero = crate::core::value::MathValue {
        dialect: math.dialect.clone(),
        body: "0".to_string(),
    };
    let relation = make_relation_pack(math, zero);
    let Value::Pack(pack) = relation else {
        return Err(RuntimeError::TypeMismatch {
            expected: "equation relation",
            span,
        });
    };
    eval_relation_solve_result(&[pack], span)
}

#[derive(Clone, Copy)]
struct LinearInequalityBound {
    value: Fixed64,
    inclusive: bool,
}

#[derive(Default)]
struct LinearInequalityInterval {
    lower: Option<LinearInequalityBound>,
    upper: Option<LinearInequalityBound>,
    empty: bool,
}

fn linear_inequality_error(message: &'static str, span: crate::lang::span::Span) -> RuntimeError {
    RuntimeError::MathDomain { message, span }
}

fn tighten_linear_lower(interval: &mut LinearInequalityInterval, value: Fixed64, inclusive: bool) {
    let next = LinearInequalityBound { value, inclusive };
    match interval.lower {
        None => interval.lower = Some(next),
        Some(current)
            if value > current.value || (value == current.value && !inclusive && current.inclusive) =>
        {
            interval.lower = Some(next);
        }
        _ => {}
    }
}

fn tighten_linear_upper(interval: &mut LinearInequalityInterval, value: Fixed64, inclusive: bool) {
    let next = LinearInequalityBound { value, inclusive };
    match interval.upper {
        None => interval.upper = Some(next),
        Some(current)
            if value < current.value || (value == current.value && !inclusive && current.inclusive) =>
        {
            interval.upper = Some(next);
        }
        _ => {}
    }
}

fn linear_zero_condition_satisfies(compare: &str, rhs: Fixed64) -> bool {
    let zero = Fixed64::zero();
    match compare {
        "<=" | "이하" => zero <= rhs,
        "<" | "미만" => zero < rhs,
        ">=" | "이상" => zero >= rhs,
        ">" | "초과" => zero > rhs,
        _ => false,
    }
}

fn apply_linear_inequality_constraint(
    interval: &mut LinearInequalityInterval,
    slope: Fixed64,
    rhs: Fixed64,
    compare: &str,
    span: crate::lang::span::Span,
) -> Result<(), RuntimeError> {
    if slope.raw() == 0 {
        if !linear_zero_condition_satisfies(compare, rhs) {
            interval.empty = true;
        }
        return Ok(());
    }
    let bound = rhs.checked_div(slope).ok_or_else(|| {
        linear_inequality_error(
            "E_LINEAR_INEQUALITY_DIV_ZERO: 선형부등식 경계 계산 중 0으로 나눌 수 없습니다",
            span,
        )
    })?;
    let inclusive = matches!(compare, "<=" | ">=" | "이하" | "이상");
    let slope_positive = slope.raw() > 0;
    match (compare, slope_positive) {
        ("<=" | "<" | "이하" | "미만", true) | (">=" | ">" | "이상" | "초과", false) => {
            tighten_linear_upper(interval, bound, inclusive)
        }
        ("<=" | "<" | "이하" | "미만", false) | (">=" | ">" | "이상" | "초과", true) => {
            tighten_linear_lower(interval, bound, inclusive)
        }
        _ => {
            return Err(linear_inequality_error(
                "E_LINEAR_INEQUALITY_BAD_COMPARE: 비교는 이하, 미만, 이상, 초과 중 하나여야 합니다",
                span,
            ))
        }
    }
    Ok(())
}

fn expect_linear_inequality_condition<'a>(
    value: &'a Value,
    span: crate::lang::span::Span,
) -> Result<&'a BTreeMap<String, Value>, RuntimeError> {
    let Value::Pack(pack) = value else {
        return Err(type_mismatch_detail("linear inequality condition pack", value, span));
    };
    Ok(&pack.fields)
}

fn linear_inequality_pack_string_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
    span: crate::lang::span::Span,
) -> Result<String, RuntimeError> {
    match fields.get(field) {
        Some(Value::Str(value)) => Ok(value.clone()),
        Some(other) => Err(type_mismatch_detail("string", other, span)),
        None => Err(RuntimeError::Pack {
            message: format!("선형부등식 조건에 {field} 필드가 필요합니다"),
            span,
        }),
    }
}

fn linear_inequality_pack_math_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
    span: crate::lang::span::Span,
) -> Result<crate::core::value::MathValue, RuntimeError> {
    match fields.get(field) {
        Some(Value::Math(value)) => Ok(value.clone()),
        Some(other) => Err(type_mismatch_detail("formula", other, span)),
        None => Err(RuntimeError::Pack {
            message: format!("선형부등식 조건에 {field} 필드가 필요합니다"),
            span,
        }),
    }
}

fn linear_inequality_pack_quantity_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
    span: crate::lang::span::Span,
) -> Result<Quantity, RuntimeError> {
    match fields.get(field) {
        Some(value) => expect_quantity_value(value, span),
        None => Err(RuntimeError::Pack {
            message: format!("선형부등식 조건에 {field} 필드가 필요합니다"),
            span,
        }),
    }
}

fn eval_linear_inequality_solve(
    values: &[Value],
    span: crate::lang::span::Span,
) -> Result<Value, RuntimeError> {
    if values.len() != 2 {
        return Err(RuntimeError::TypeMismatch {
            expected: "conditions, var",
            span,
        });
    }
    let conditions = match &values[0] {
        Value::List(list) => list,
        other => return Err(type_mismatch_detail("condition list", other, span)),
    };
    let var_name = match &values[1] {
        Value::Str(value) => value.trim().trim_start_matches('#').to_string(),
        other => return Err(type_mismatch_detail("string var", other, span)),
    };
    if var_name.is_empty() {
        return Err(RuntimeError::FormulaParse {
            message: "E_LINEAR_INEQUALITY_BAD_VAR: 변수 이름이 비어 있습니다".to_string(),
            span,
        });
    }

    let mut interval = LinearInequalityInterval::default();
    for condition in &conditions.items {
        let fields = expect_linear_inequality_condition(condition, span)?;
        let math = linear_inequality_pack_math_field(fields, "식", span)?;
        let compare = linear_inequality_pack_string_field(fields, "비교", span)?;
        let boundary = linear_inequality_pack_quantity_field(fields, "경계", span)?;
        let prepared = prepare_numeric_formula(&math, &var_name, span, "선형부등식.풀기")?;
        let x0 = Quantity::new(Fixed64::from_int(0), UnitDim::zero());
        let x1 = Quantity::new(Fixed64::from_int(1), UnitDim::zero());
        let x2 = Quantity::new(Fixed64::from_int(2), UnitDim::zero());
        let y0 = eval_numeric_formula_at(&prepared, x0, span, "선형부등식.풀기")?;
        let y1 = eval_numeric_formula_at(&prepared, x1, span, "선형부등식.풀기")?;
        let y2 = eval_numeric_formula_at(&prepared, x2, span, "선형부등식.풀기")?;
        ensure_same_dim(&y0, &y1, span)?;
        ensure_same_dim(&y0, &y2, span)?;
        ensure_same_dim(&y0, &boundary, span)?;
        let slope01 = y1.raw.saturating_sub(y0.raw);
        let slope12 = y2.raw.saturating_sub(y1.raw);
        if slope01 != slope12 {
            return Err(linear_inequality_error(
                "E_LINEAR_INEQUALITY_NONLINEAR: V1은 1변수 선형 부등식만 지원합니다",
                span,
            ));
        }
        let rhs = boundary.raw.saturating_sub(y0.raw);
        apply_linear_inequality_constraint(&mut interval, slope01, rhs, compare.trim(), span)?;
    }

    if let (Some(lower), Some(upper)) = (interval.lower, interval.upper) {
        if lower.value > upper.value
            || (lower.value == upper.value && (!lower.inclusive || !upper.inclusive))
        {
            interval.empty = true;
        }
    }

    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::Str("linear_inequality_solution".to_string()),
    );
    fields.insert("변수".to_string(), Value::Str(var_name));
    fields.insert(
        "제약개수".to_string(),
        Value::Num(Quantity::new(
            Fixed64::from_int(conditions.items.len() as i64),
            UnitDim::zero(),
        )),
    );
    if interval.empty {
        fields.insert("상태".to_string(), Value::Str("공집합".to_string()));
    } else {
        fields.insert(
            "상태".to_string(),
            Value::Str(if interval.lower.is_none() && interval.upper.is_none() {
                "전체"
            } else {
                "구간"
            }
            .to_string()),
        );
        if let Some(lower) = interval.lower {
            fields.insert(
                "하한".to_string(),
                Value::Num(Quantity::new(lower.value, UnitDim::zero())),
            );
            fields.insert("하한포함".to_string(), Value::Bool(lower.inclusive));
        }
        if let Some(upper) = interval.upper {
            fields.insert(
                "상한".to_string(),
                Value::Num(Quantity::new(upper.value, UnitDim::zero())),
            );
            fields.insert("상한포함".to_string(), Value::Bool(upper.inclusive));
        }
    }
    Ok(Value::Pack(PackValue { fields }))
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

fn fixed64_sign(value: Fixed64) -> i8 {
    match value.raw().cmp(&0) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn numeric_bisection_root(
    prepared: &NumericFormulaPrepared,
    mut lower: Quantity,
    mut upper: Quantity,
    iterations: usize,
    span: crate::lang::span::Span,
) -> Result<(Quantity, Quantity, usize), RuntimeError> {
    if upper.raw.raw() < lower.raw.raw() {
        std::mem::swap(&mut lower, &mut upper);
    }

    let mut f_lower = eval_numeric_formula_at(prepared, lower.clone(), span, "수치해.이분법")?;
    let mut f_upper = eval_numeric_formula_at(prepared, upper.clone(), span, "수치해.이분법")?;
    ensure_same_dim(&f_lower, &f_upper, span)?;
    if f_lower.raw.raw() == 0 {
        let dim = f_lower.dim;
        return Ok((lower, Quantity::new(Fixed64::zero(), dim), 0));
    }
    if f_upper.raw.raw() == 0 {
        let dim = f_upper.dim;
        return Ok((upper, Quantity::new(Fixed64::zero(), dim), 0));
    }
    if fixed64_sign(f_lower.raw) == fixed64_sign(f_upper.raw) {
        return Err(RuntimeError::MathDomain {
            message: "E_CALC_NUMERIC_BRACKET_SIGN: 수치해.이분법 하한/상한 함수값의 부호가 달라야 합니다",
            span,
        });
    }

    let two = Fixed64::from_int(2);
    let mut best_root = lower.clone();
    let mut best_residual = f_lower.clone();
    let mut used_iterations = 0_usize;
    for idx in 1..=iterations {
        let mid_raw = lower
            .raw
            .saturating_add(upper.raw)
            .checked_div(two)
            .ok_or(RuntimeError::MathDivZero { span })?;
        let mid = Quantity::new(mid_raw, lower.dim);
        let f_mid = eval_numeric_formula_at(prepared, mid.clone(), span, "수치해.이분법")?;
        ensure_same_dim(&f_lower, &f_mid, span)?;
        best_root = mid.clone();
        best_residual = Quantity::new(fixed64_abs(f_mid.raw), f_mid.dim);
        used_iterations = idx;

        if f_mid.raw.raw() == 0 {
            break;
        }
        if fixed64_sign(f_lower.raw) != fixed64_sign(f_mid.raw) {
            upper = mid;
            f_upper = f_mid;
        } else {
            lower = mid;
            f_lower = f_mid;
        }
        ensure_same_dim(&f_lower, &f_upper, span)?;
    }

    Ok((best_root, best_residual, used_iterations))
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
    if dim.length != 0
        || dim.time != 0
        || dim.mass != 0
        || dim.pixel != 0
        || dim.temperature != 0
        || dim.krw != 0
        || dim.usd != 0
    {
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
        || dim.krw % 2 != 0
        || dim.usd % 2 != 0
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
        krw: dim.krw / 2,
        usd: dim.usd / 2,
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

    fn run_frontdoor_source_once(source: &str) -> Result<EvalOutput, RuntimeError> {
        let prepared = ddonirang_lang::preprocess_frontdoor_source(source);
        run_source_once(&prepared)
    }

    fn frontdoor_source_parses(source: &str) -> bool {
        let prepared = ddonirang_lang::preprocess_frontdoor_source(source);
        let tokens = Lexer::tokenize(&prepared).expect("lex");
        let default_root = Parser::default_root_for_source(&prepared);
        Parser::parse_with_default_root(tokens, default_root).is_ok()
    }

    fn run_source_ticks(source: &str, ticks: u64) -> Result<EvalOutput, RuntimeError> {
        let tokens = Lexer::tokenize(source).expect("lex");
        let default_root = Parser::default_root_for_source(source);
        let program = Parser::parse_with_default_root(tokens, default_root).expect("parse");
        let evaluator = Evaluator::with_state_and_seed(State::new(), 42);
        evaluator.run_with_ticks(&program, ticks)
    }

    fn parse_program(source: &str) -> Program {
        let tokens = Lexer::tokenize(source).expect("lex");
        let default_root = Parser::default_root_for_source(source);
        Parser::parse_with_default_root(tokens, default_root).expect("parse")
    }

    fn dimensionless_num(value: i64) -> Value {
        Value::Num(Quantity::new(Fixed64::from_int(value), UnitDim::zero()))
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
    fn when_hook_does_not_refire_in_same_madi_after_body_mutation() {
        let source = r#"
값 <- 0.
횟수 <- 0.
(시작)할때 {
  값 <- 1.
}.
(값 > 0)이 될때 {
  횟수 <- 횟수 + 1.
  값 <- 0.
  값 <- 1.
}.
"#;
        let output = run_source_ticks(source, 3).expect("run");
        assert_eq!(state_num(&output, "횟수"), Fixed64::from_int(1));
    }

    #[test]
    fn while_hook_runs_once_per_madi_not_intra_tick_loop() {
        let source = r#"
연료 <- 2.
횟수 <- 0.
(연료 > 0)인 동안 {
  횟수 <- 횟수 + 1.
  연료 <- 연료 - 1.
}.
"#;
        let output = run_source_ticks(source, 4).expect("run");
        assert_eq!(state_num(&output, "횟수"), Fixed64::from_int(2));
        assert_eq!(state_num(&output, "연료"), Fixed64::from_int(0));
    }

    #[test]
    fn flow_hook_runs_before_tail_becomes_hook() {
        let source = r#"
채비 {
  입력: 수 <- 0.
  흐름값: 수 <- 0.
  훅실행: 수 <- 0.
  결과: 수 <- 0.
  t: 수 <- 0.
}.

(시작)할때 {
  입력 <- 5.
}.

(매마디)마다 {
  흐름값 <<- 입력 * 2.
  t <- t + 1.
}.

(흐름값 > 8)일때 {
  훅실행 <- 1.
  결과 <- 흐름값.
}.
"#;
        let output = run_source_ticks(source, 1).expect("run");
        assert_eq!(state_num(&output, "입력"), Fixed64::from_int(5));
        assert_eq!(state_num(&output, "흐름값"), Fixed64::from_int(10));
        assert_eq!(state_num(&output, "훅실행"), Fixed64::from_int(1));
        assert_eq!(state_num(&output, "결과"), Fixed64::from_int(10));
    }

    #[test]
    fn flow_hook_body_does_not_refire_flow_same_madi() {
        let source = r#"
채비 {
  값: 수 <- 0.
  카운트: 수 <- 0.
  t: 수 <- 0.
}.

(시작)할때 {
  값 <- 3.
}.

(매마디)마다 {
  값 <<- 값 + 1.
  t <- t + 1.
}.

(값 > 5)일때 {
  카운트 <- 카운트 + 1.
  값 <- 값 + 10.
}.
"#;
        let after_tick_0 = run_source_ticks(source, 1).expect("run");
        assert_eq!(state_num(&after_tick_0, "값"), Fixed64::from_int(4));
        assert_eq!(state_num(&after_tick_0, "카운트"), Fixed64::from_int(0));

        let after_tick_2 = run_source_ticks(source, 3).expect("run");
        assert_eq!(state_num(&after_tick_2, "값"), Fixed64::from_int(16));
        assert_eq!(state_num(&after_tick_2, "카운트"), Fixed64::from_int(1));
    }

    #[test]
    fn flow_hook_multiple_source_conflict_errors() {
        let source = r#"
채비 {
  입력: 수 <- 0.
  흐름값: 수 <- 0.
}.

(시작)할때 {
  입력 <- 1.
}.

(매마디)마다 {
  흐름값 <<- 입력 + 1.
  흐름값 <<- 입력 + 2.
}.
"#;
        let err = match run_source_ticks(source, 1) {
            Ok(_) => panic!("conflict must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_FLOW_MULTIPLE_SOURCE_CONFLICT");
    }

    #[test]
    fn flow_hook_cycle_errors() {
        let source = r#"
채비 {
  a: 수 <- 0.
  b: 수 <- 0.
}.

(매마디)마다 {
  a <<- b + 1.
  b <<- a + 1.
}.
"#;
        let err = match run_source_ticks(source, 1) {
            Ok(_) => panic!("cycle must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_FLOW_CIRCULAR_REFERENCE");
    }

    #[test]
    fn continue_loop_skips_current_foreach_item() {
        let source = r#"
합 <- 0.
목록 <- [1, 2, 3].
(x) 목록에 대해 {
  x == 2 일때 {
    건너뛰기.
  }.
  합 <- 합 + x.
}.
"#;
        let output = run_source_ticks(source, 1).expect("run");
        assert_eq!(state_num(&output, "합"), Fixed64::from_int(4));
    }

    #[test]
    fn continue_loop_outside_foreach_is_runtime_error() {
        let source = r#"
건너뛰기.
"#;
        let err = match run_source_ticks(source, 1) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_RUNTIME_CONTINUE_OUTSIDE_FOREACH");
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
    fn nuri_reset_keeps_live_input_values() {
        let source = r#"
값 <- 0.
(매마디)마다 {
  값 <- 값 + 1.
  누리다시.
}.
"#;
        let program = parse_program(source);
        let evaluator = Evaluator::with_state_and_seed(State::new(), 42);
        let output = evaluator
            .run_with_ticks_observe_and_inject(
                &program,
                2,
                |madi, state| {
                    let pressed = if madi == 0 { 0 } else { 1 };
                    state.set(
                        Key::new("샘.키보드.누르고있음.ArrowRight"),
                        dimensionless_num(pressed),
                    );
                    Ok(())
                },
                |_, _, _| {},
            )
            .expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(0));
        assert_eq!(
            state_num(&output, "샘.키보드.누르고있음.ArrowRight"),
            Fixed64::from_int(1)
        );
    }

    #[test]
    fn all_reset_restores_live_input_values_to_snapshot() {
        let source = r#"
값 <- 0.
(매마디)마다 {
  값 <- 값 + 1.
  모두다시.
}.
"#;
        let mut initial_state = State::new();
        initial_state.set(
            Key::new("샘.키보드.누르고있음.ArrowRight"),
            dimensionless_num(1),
        );
        let program = parse_program(source);
        let evaluator = Evaluator::with_state_and_seed(initial_state, 42);
        let output = evaluator
            .run_with_ticks_observe_and_inject(
                &program,
                2,
                |_, state| {
                    state.set(
                        Key::new("샘.키보드.누르고있음.ArrowRight"),
                        dimensionless_num(0),
                    );
                    Ok(())
                },
                |_, _, _| {},
            )
            .expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(0));
        assert_eq!(
            state_num(&output, "샘.키보드.누르고있음.ArrowRight"),
            Fixed64::from_int(1)
        );
    }

    #[test]
    fn nuri_reset_drops_snapshot_live_input_when_current_tick_has_none() {
        let source = r#"
값 <- 0.
(매마디)마다 {
  값 <- 값 + 1.
  누리다시.
}.
"#;
        let mut initial_state = State::new();
        initial_state.set(
            Key::new("샘.키보드.누르고있음.ArrowRight"),
            dimensionless_num(1),
        );
        let program = parse_program(source);
        let evaluator = Evaluator::with_state_and_seed(initial_state, 42);
        let output = evaluator
            .run_with_ticks_observe_and_inject(
                &program,
                1,
                |_, state| {
                    state
                        .resources
                        .remove(&Key::new("샘.키보드.누르고있음.ArrowRight"));
                    Ok(())
                },
                |_, _, _| {},
            )
            .expect("run");
        assert_eq!(state_num(&output, "값"), Fixed64::from_int(0));
        assert!(output
            .state
            .get(&Key::new("샘.키보드.누르고있음.ArrowRight"))
            .is_none());
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
    fn std_input_map_closure_runtime_maps_explicit_and_default_keys() {
        let source = r#"
사상 <- (왼쪽="ArrowLeft", 오른쪽="ArrowRight", 위="ArrowUp", 아래="ArrowDown", 확인="Space") 입력사상.만들기.
기본 <- () 입력사상.만들기.
방향 <- (사상) 입력사상.방향.
오른쪽 <- (기본, "오른쪽") 입력사상.동작.
확인 <- (사상, "확인") 입력사상.동작.
"#;
        let mut initial_state = State::new();
        initial_state.set(
            Key::new("샘.키보드.누르고있음.ArrowRight"),
            dimensionless_num(1),
        );
        let program = parse_program(source);
        let evaluator = Evaluator::with_state_and_seed(initial_state, 42);
        let output = evaluator.run_with_ticks(&program, 1).expect("run");
        assert_eq!(state_display(&output, "방향"), "차림[1, 0]");
        assert!(state_bool(&output, "오른쪽"));
        assert!(!state_bool(&output, "확인"));
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
    fn choose_plain_condition_runs_first_matching_branch() {
        let source = r#"
고르기:
1 < 2: {
  살림.x <- 1.
}
1 == 1: {
  살림.x <- 2.
}
아니면: {
  살림.x <- 7.
}.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "x"), fixed("1"));
    }

    #[test]
    fn choose_plain_condition_runs_second_and_else_branches() {
        let second_source = r#"
고르기:
1 > 2: {
  살림.x <- 1.
}
1 == 1: {
  살림.x <- 2.
}
아니면: {
  살림.x <- 7.
}.
"#;
        let second_output = run_source_once(second_source).expect("run");
        assert_eq!(state_num(&second_output, "x"), fixed("2"));

        let else_source = r#"
고르기:
1 > 2: {
  살림.x <- 1.
}
1 == 2: {
  살림.x <- 2.
}
아니면: {
  살림.x <- 7.
}.
"#;
        let else_output = run_source_once(else_source).expect("run");
        assert_eq!(state_num(&else_output, "x"), fixed("7"));
    }

    #[test]
    fn choose_plain_condition_canonical_case_runs_fallback_body() {
        let source = r#"
고르기:
1 > 2 인 경우 {
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
    fn seumssi_v1b_surface_alias_runs_as_canonical_seum() {
        let source = r#"
검사 <- 세움씨{
  { 거리 >= 0 }인것 바탕으로(물림) 아니면 {
    없음.
  }.
}.
판정 <- (거리=3)인 검사 살피기.
"#;
        let output = run_source_once(source).expect("run");
        assert!(state_bool(&output, "판정"));
        assert_eq!(state_display(&output, "검사").starts_with("세움{"), true);
        assert_eq!(state_display(&output, "검사").starts_with("세움씨{"), false);
        assert_eq!(output.diagnostics.len(), 1);
        assert!(output.diagnostics[0].name.starts_with("살피기 세움{"));
        assert!(output.diagnostics[0].error_code.is_none());
    }

    #[test]
    fn seumssi_v1b_relation_bridge_runtime_matches_seum_canon() {
        let source = r#"
검사 <- 세움씨{
  (#ascii) 수식{x + y} =:= (#ascii) 수식{5}
}.
판정 <- (세움=검사, 값들=(x=("3") 큰바른수, y=("2") 큰바른수)) 살피기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(
            state_display(&output, "검사"),
            "세움{수식관계: x + y =:= 5}"
        );
        assert!(state_bool(&output, "판정"));
        assert!(output.diagnostics[0]
            .name
            .starts_with("살피기 세움{수식관계: x + y =:= 5}"));
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
    fn std_grid_closure_runtime_pathfind_and_physics_smoke() {
        let source = r##"
길 <- ".".
벽 <- "#".
격0 <- (4, 3, 길) 격자.만들기.
격1 <- (격0, 1, 0, 벽) 격자.바꾼값.
격2 <- (격1, 1, 1, 벽) 격자.바꾼값.
격3 <- (격2, 3, 1, 벽) 격자.바꾼값.
경로 <- (격3, 0, 0, 3, 2, (벽) 차림) 격자.길찾기.
경로길이 <- (경로) 길이.
다음위치 <- (10, 2, 3) 물리1d.위치갱신.
"##;
        let output = run_source_once(source).expect("run");
        assert_eq!(
            state_display(&output, "경로"),
            "차림[차림[0, 0], 차림[0, 1], 차림[0, 2], 차림[1, 2], 차림[2, 2], 차림[3, 2]]"
        );
        assert_eq!(state_num(&output, "경로길이"), fixed("6"));
        assert_eq!(state_num(&output, "다음위치"), fixed("16"));
    }

    #[test]
    fn std_block_piece_geometry_moves_and_rotates() {
        let source = r#"
조각 <- (((0, 0) 차림, (1, 0) 차림, (0, 1) 차림) 차림) 블록조각.만들기.
원본 <- (조각) 블록조각.칸목록.
이동 <- ((조각, 2, 1) 블록조각.이동) 블록조각.칸목록.
오른쪽 <- ((조각, "오른쪽") 블록조각.회전) 블록조각.칸목록.
왼쪽 <- ((조각, "왼쪽") 블록조각.회전) 블록조각.칸목록.
뒤집기 <- ((조각, "뒤집기") 블록조각.회전) 블록조각.칸목록.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(
            state_display(&output, "원본"),
            "차림[차림[0, 0], 차림[1, 0], 차림[0, 1]]"
        );
        assert_eq!(
            state_display(&output, "이동"),
            "차림[차림[2, 1], 차림[3, 1], 차림[2, 2]]"
        );
        assert_eq!(
            state_display(&output, "오른쪽"),
            "차림[차림[-1, 0], 차림[0, 0], 차림[0, 1]]"
        );
        assert_eq!(
            state_display(&output, "왼쪽"),
            "차림[차림[0, -1], 차림[0, 0], 차림[1, 0]]"
        );
        assert_eq!(
            state_display(&output, "뒤집기"),
            "차림[차림[0, -1], 차림[-1, 0], 차림[0, 0]]"
        );
    }

    #[test]
    fn std_block_piece_grid_bridge_collides_and_locks() {
        let source = r##"
빈칸 <- ".".
벽 <- "#".
격0 <- (4, 3, 빈칸) 격자.만들기.
격1 <- (격0, 3, 1, 벽) 격자.바꾼값.
조각 <- (((0, 0) 차림, (1, 0) 차림, (0, 1) 차림) 차림) 블록조각.만들기.
안쪽 <- (조각, 1, 1) 블록조각.이동.
벽쪽 <- (조각, 2, 1) 블록조각.이동.
바깥 <- (조각, -1, 0) 블록조각.이동.
안쪽충돌 <- (안쪽, 격1, (벽) 차림) 블록조각.충돌?.
벽충돌 <- (벽쪽, 격1, (벽) 차림) 블록조각.충돌?.
밖충돌 <- (바깥, 격1, (벽) 차림) 블록조각.충돌?.
고정격 <- (안쪽, 격1, "X") 블록조각.고정.
고정값 <- (고정격, 1, 1) 격자.값.
원래벽 <- (고정격, 3, 1) 격자.값.
"##;
        let output = run_source_once(source).expect("run");
        assert!(!state_bool(&output, "안쪽충돌"));
        assert!(state_bool(&output, "벽충돌"));
        assert!(state_bool(&output, "밖충돌"));
        assert_eq!(state_str(&output, "고정값"), "X");
        assert_eq!(state_str(&output, "원래벽"), "#");
    }

    #[test]
    fn std_random_bag_draws_refills_and_previews() {
        let source = r#"
가방0 <- (0, ("I", "O", "T") 차림) 무작위가방.만들기.
미리 <- (가방0, 5) 무작위가방.미리보기.
뽑1 <- (가방0) 무작위가방.꺼내기.
값1 <- 뽑1.값.
가방1 <- 뽑1.가방.
뽑2 <- (가방1) 무작위가방.꺼내기.
값2 <- 뽑2.값.
가방2 <- 뽑2.가방.
뽑3 <- (가방2) 무작위가방.꺼내기.
값3 <- 뽑3.값.
가방3 <- 뽑3.가방.
빈가 <- (가방3) 무작위가방.비었나.
남은3 <- (가방3) 무작위가방.남은것.
뽑4 <- (가방3) 무작위가방.꺼내기.
값4 <- 뽑4.값.
남은4 <- (뽑4.가방) 무작위가방.남은것.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_display(&output, "미리"), "차림[O, I, T, O, T]");
        assert_eq!(state_str(&output, "값1"), "O");
        assert_eq!(state_str(&output, "값2"), "I");
        assert_eq!(state_str(&output, "값3"), "T");
        assert!(state_bool(&output, "빈가"));
        assert_eq!(state_display(&output, "남은3"), "차림[]");
        assert_eq!(state_str(&output, "값4"), "O");
        assert_eq!(state_display(&output, "남은4"), "차림[I, T]");
    }

    #[test]
    fn std_grid_game_state_lifecycle_pause_resume() {
        let source = r#"
초기 <- () 격자게임상태.초기화.
상태0 <- (초기) 격자게임상태.상태.
틱0 <- (초기) 격자게임상태.틱.
진행 <- (초기, "진행") 격자게임상태.바꾸기.
진행인가 <- (진행, "진행") 격자게임상태.상태인가.
멈춤 <- (진행) 격자게임상태.멈춤.
멈춤상태 <- (멈춤) 격자게임상태.상태.
재개 <- (멈춤) 격자게임상태.재개.
재개상태 <- (재개) 격자게임상태.상태.
재개틱 <- (재개) 격자게임상태.틱.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "상태0"), "준비");
        assert_eq!(state_num(&output, "틱0"), fixed("0"));
        assert!(state_bool(&output, "진행인가"));
        assert_eq!(state_str(&output, "멈춤상태"), "멈춤");
        assert_eq!(state_str(&output, "재개상태"), "진행");
        assert_eq!(state_num(&output, "재개틱"), fixed("3"));
    }

    #[test]
    fn std_tetromino_catalog_returns_fixed_cells() {
        let source = r#"
이름들 <- () 테트로미노.이름목록.
아이 <- ("I") 테트로미노.만들기.
아이칸 <- (아이) 블록조각.칸목록.
조각들 <- () 테트로미노.목록.
조각수 <- (조각들) 길이.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(
            state_display(&output, "이름들"),
            "차림[I, O, T, S, Z, J, L]"
        );
        assert_eq!(
            state_display(&output, "아이칸"),
            "차림[차림[-1, 0], 차림[0, 0], 차림[1, 0], 차림[2, 0]]"
        );
        assert_eq!(state_num(&output, "조각수"), fixed("7"));
    }

    #[test]
    fn std_grid_line_clear_removes_full_rows() {
        let source = r#"
빈 <- ".".
격0 <- (4, 3, 빈) 격자.만들기.
격1 <- (격0, 1, 1, "P") 격자.바꾼값.
격2 <- (격1, 0, 2, "X") 격자.바꾼값.
격3 <- (격2, 1, 2, "X") 격자.바꾼값.
격4 <- (격3, 2, 2, "X") 격자.바꾼값.
격5 <- (격4, 3, 2, "X") 격자.바꾼값.
찬줄 <- (격5, 빈) 격자줄.찬줄목록.
지움 <- (격5, 빈) 격자줄.지우기.
지운수 <- 지움.지운줄수.
격후 <- 지움.격자.
아래중 <- (격후, 1, 2) 격자.값.
위빈 <- (격후, 0, 0) 격자.값.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_display(&output, "찬줄"), "차림[2]");
        assert_eq!(state_num(&output, "지운수"), fixed("1"));
        assert_eq!(state_str(&output, "아래중"), "P");
        assert_eq!(state_str(&output, "위빈"), ".");
    }

    #[test]
    fn std_falling_piece_state_places_and_moves() {
        let source = r#"
오 <- ("O") 테트로미노.만들기.
낙0 <- (오, 2, 3) 낙하조각.만들기.
위치0 <- (낙0) 낙하조각.위치.
배치0 <- (낙0) 낙하조각.배치.
낙1 <- (낙0, -1, 2) 낙하조각.이동.
위치1 <- (낙1) 낙하조각.위치.
돌림 <- (낙0, "오른쪽") 낙하조각.회전.
돌림배치 <- (돌림) 낙하조각.배치.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_display(&output, "위치0"), "차림[2, 3]");
        assert_eq!(
            state_display(&output, "배치0"),
            "차림[차림[2, 3], 차림[3, 3], 차림[2, 4], 차림[3, 4]]"
        );
        assert_eq!(state_display(&output, "위치1"), "차림[1, 5]");
        assert_eq!(
            state_display(&output, "돌림배치"),
            "차림[차림[1, 3], 차림[2, 3], 차림[1, 4], 차림[2, 4]]"
        );
    }

    #[test]
    fn std_grid_game_playable_scores_sessions_and_ticks() {
        let source = r#"
점0 <- () 격자게임점수.초기화.
점1 <- (점0, 4) 격자게임점수.더하기.
점2 <- (점1, 4) 격자게임점수.더하기.
점3 <- (점2, 2) 격자게임점수.더하기.
점4 <- (점3, 1) 격자게임점수.더하기.
점수4 <- (점4) 격자게임점수.점수.
줄4 <- (점4) 격자게임점수.줄수.
레벨4 <- (점4) 격자게임점수.레벨.
빈 <- ".".
격 <- (4, 4, 빈) 격자.만들기.
가방 <- (0, ("O", "I") 차림) 무작위가방.만들기.
상태0 <- () 격자게임상태.초기화.
진행상태 <- (상태0, "진행") 격자게임상태.바꾸기.
오 <- ("O") 테트로미노.만들기.
낙0 <- (오, 1, 0) 낙하조각.만들기.
세션0 <- (격, 가방, 진행상태, 점0, 낙0) 격자게임세션.만들기.
조작맵 <- () 입력사상.만들기.
틱0 <- (세션0, 조작맵) 격자게임.한틱.
고정0 <- 틱0.고정됐나.
세션1 <- 틱0.세션.
낙1 <- (세션1) 격자게임세션.낙하조각.
위치1 <- (낙1) 낙하조각.위치.
낙잠금 <- (오, 1, 2) 낙하조각.만들기.
세션잠금 <- (격, 가방, 진행상태, 점0, 낙잠금) 격자게임세션.만들기.
틱잠금 <- (세션잠금, 조작맵) 격자게임.한틱.
고정잠금 <- 틱잠금.고정됐나.
세션잠금2 <- 틱잠금.세션.
격잠금 <- (세션잠금2) 격자게임세션.격자.
잠긴값 <- (격잠금, 1, 2) 격자.값.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "점수4"), fixed("2100"));
        assert_eq!(state_num(&output, "줄4"), fixed("11"));
        assert_eq!(state_num(&output, "레벨4"), fixed("2"));
        assert!(!state_bool(&output, "고정0"));
        assert_eq!(state_display(&output, "위치1"), "차림[1, 1]");
        assert!(state_bool(&output, "고정잠금"));
        assert_eq!(state_str(&output, "잠긴값"), "X");
    }

    #[test]
    fn std_grid_game_view_projects_text_cells_and_summary() {
        let source = r#"
빈 <- ".".
격0 <- (4, 4, 빈) 격자.만들기.
격1 <- (격0, 0, 3, "X") 격자.바꾼값.
가방 <- (0, ("O", "I") 차림) 무작위가방.만들기.
상태0 <- () 격자게임상태.초기화.
진행 <- (상태0, "진행") 격자게임상태.바꾸기.
점 <- () 격자게임점수.초기화.
오 <- ("O") 테트로미노.만들기.
낙 <- (오, 1, 0) 낙하조각.만들기.
세션 <- (격1, 가방, 진행, 점, 낙) 격자게임세션.만들기.
칸목록 <- (세션, 빈, "O") 격자게임보기.칸목록.
칸수 <- (칸목록) 길이.
칸0 <- (칸목록, 0) 차림.값.
칸1 <- (칸목록, 1) 차림.값.
칸12 <- (칸목록, 12) 차림.값.
원천0 <- 칸0.원천.
원천1 <- 칸1.원천.
원천12 <- 칸12.원천.
값1 <- 칸1.값.
문자판 <- (세션, 빈, "O") 격자게임보기.문자판.
요약 <- (세션) 격자게임보기.상태요약.
요약종류 <- 요약.__종류.
요약틱 <- 요약.틱.
요약점수 <- 요약.점수.
요약줄수 <- 요약.줄수.
요약레벨 <- 요약.레벨.
요약위치 <- 요약.낙하조각위치.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "칸수"), fixed("16"));
        assert_eq!(state_str(&output, "원천0"), "빈칸");
        assert_eq!(state_str(&output, "원천1"), "낙하");
        assert_eq!(state_str(&output, "원천12"), "고정");
        assert_eq!(state_str(&output, "값1"), "O");
        assert_eq!(state_str(&output, "문자판"), ".OO.\n.OO.\n....\nX...");
        assert_eq!(state_str(&output, "요약종류"), "std_grid_game_view_summary");
        assert_eq!(state_num(&output, "요약틱"), fixed("1"));
        assert_eq!(state_num(&output, "요약점수"), fixed("0"));
        assert_eq!(state_num(&output, "요약줄수"), fixed("0"));
        assert_eq!(state_num(&output, "요약레벨"), fixed("1"));
        assert_eq!(state_display(&output, "요약위치"), "차림[1, 0]");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("요약".to_string())) else {
            panic!("요약 must be pack");
        };
        assert_eq!(
            pack.fields.get("상태"),
            Some(&Value::Str("진행".to_string()))
        );
    }

    #[test]
    fn std_grid_game_bogae_projects_rect_drawlist_and_size() {
        let source = r##"
빈 <- ".".
격0 <- (4, 4, 빈) 격자.만들기.
격1 <- (격0, 0, 3, "X") 격자.바꾼값.
가방 <- (0, ("O", "I") 차림) 무작위가방.만들기.
상태0 <- () 격자게임상태.초기화.
진행 <- (상태0, "진행") 격자게임상태.바꾸기.
점 <- () 격자게임점수.초기화.
오 <- ("O") 테트로미노.만들기.
낙 <- (오, 1, 0) 낙하조각.만들기.
세션 <- (격1, 가방, 진행, 점, 낙) 격자게임세션.만들기.
목록 <- (세션, 빈, "O", 8) 격자게임보기.보개목록.
크기 <- (세션, 8) 격자게임보기.보개크기.
칸수 <- (목록) 길이.
첫 <- (목록, 0) 차림.값.
둘 <- (목록, 1) 차림.값.
고정 <- (목록, 12) 차림.값.
첫id <- 첫.id.
첫결 <- 첫.결.
첫x <- 첫.x.
첫색 <- 첫.채움색.
둘id <- 둘.id.
둘x <- 둘.x.
둘색 <- 둘.채움색.
고정id <- 고정.id.
고정y <- 고정.y.
고정색 <- 고정.채움색.
가로 <- 크기.가로.
세로 <- 크기.세로.
"##;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "칸수"), fixed("16"));
        assert_eq!(state_str(&output, "첫id"), "격자게임셀_0_0");
        assert_eq!(state_str(&output, "첫결"), "#보개/2D.Rect");
        assert_eq!(state_num(&output, "첫x"), fixed("0"));
        assert_eq!(state_str(&output, "첫색"), "#111111ff");
        assert_eq!(state_str(&output, "둘id"), "격자게임셀_0_1");
        assert_eq!(state_num(&output, "둘x"), fixed("8"));
        assert_eq!(state_str(&output, "둘색"), "#ffcc00ff");
        assert_eq!(state_str(&output, "고정id"), "격자게임셀_3_0");
        assert_eq!(state_num(&output, "고정y"), fixed("24"));
        assert_eq!(state_str(&output, "고정색"), "#4a90e2ff");
        assert_eq!(state_num(&output, "가로"), fixed("32"));
        assert_eq!(state_num(&output, "세로"), fixed("32"));
    }

    #[test]
    fn std_grid_game_rules_hold_ghost_and_wall_kick_helpers() {
        let source = r##"
빈 <- ".".
막힘 <- ("X") 차림.
격 <- (4, 6, 빈) 격자.만들기.
가방0 <- (0, ("I", "O") 차림) 무작위가방.만들기.
상태0 <- () 격자게임상태.초기화.
진행 <- (상태0, "진행") 격자게임상태.바꾸기.
점 <- () 격자게임점수.초기화.
티 <- ("T") 테트로미노.만들기.
오 <- ("O") 테트로미노.만들기.
낙T <- (티, 0, 1) 낙하조각.만들기.
낙O <- (오, 1, 0) 낙하조각.만들기.
홀드0 <- () 격자게임홀드.초기화.
빈홀드 <- (홀드0) 격자게임홀드.칸.
안씀 <- (홀드0) 격자게임홀드.썼나.
교체1 <- (홀드0, 낙T, 가방0, 2, 0) 격자게임홀드.교체.
홀드1 <- 교체1.홀드.
홀드1칸 <- (홀드1) 격자게임홀드.칸.
홀드1씀 <- (홀드1) 격자게임홀드.썼나.
낙1 <- 교체1.낙하조각.
낙1위치 <- (낙1) 낙하조각.위치.
홀드2준비 <- (홀드1) 격자게임홀드.초기화턴.
홀드2씀 <- (홀드2준비) 격자게임홀드.썼나.
세션 <- (격, 가방0, 진행, 점, 낙O) 격자게임세션.만들기.
유령 <- (세션, 막힘) 격자게임보기.유령조각.
유령위치 <- (유령) 낙하조각.위치.
유령보개 <- (세션, 빈, "O", "G", 4, 막힘) 격자게임보기.유령보개목록.
낙하칸 <- (유령보개, 1) 차림.값.
유령칸 <- (유령보개, 17) 차림.값.
낙하색 <- 낙하칸.채움색.
유령색 <- 유령칸.채움색.
회전 <- (낙T, 격, 막힘, "오른쪽") 격자게임.회전시도.
회전성공 <- 회전.성공.
회전오프셋 <- 회전.오프셋.
회전낙 <- 회전.낙하조각.
회전위치 <- (회전낙) 낙하조각.위치.
"##;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_display(&output, "빈홀드"), "없음");
        assert!(!state_bool(&output, "안씀"));
        assert_eq!(
            state_display(&output, "홀드1칸"),
            "묶음{__종류=std_block_piece, 칸들=차림[차림[-1, 0], 차림[0, 0], 차림[1, 0], 차림[0, 1]]}"
        );
        assert!(state_bool(&output, "홀드1씀"));
        assert_eq!(state_display(&output, "낙1위치"), "차림[2, 0]");
        assert!(!state_bool(&output, "홀드2씀"));
        assert_eq!(state_display(&output, "유령위치"), "차림[1, 4]");
        assert_eq!(state_str(&output, "낙하색"), "#ffcc00ff");
        assert_eq!(state_str(&output, "유령색"), "#88ffffff");
        assert!(state_bool(&output, "회전성공"));
        assert_eq!(state_display(&output, "회전오프셋"), "차림[1, 0]");
        assert_eq!(state_display(&output, "회전위치"), "차림[1, 1]");
    }

    #[test]
    fn std_unit_closure_temperature_smoke() {
        let source = r#"
온도동치 <- 25@C == 77@F.
온도차이 <- 30@C - 20@C.
"#;
        let output = run_source_once(source).expect("run");
        assert!(state_bool(&output, "온도동치"));
        let diff = state_quantity(&output, "온도차이");
        assert_eq!(diff.display(), "10@K");
        assert_fixed_close(diff.raw, fixed("10"), 1);
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
베를레 <- (1, 0, -1, -0.96875, 0.25) 적분.속도베를레.
x3 <- (베를레, 0) 차림.값.
v3 <- (베를레, 1) 차림.값.
보간1 <- (0, 10, 0.25) 보간.선형.
보간2 <- (0, 10, 0.25, 0.5) 보간.계단.
보간3 <- (0, 10, 0.75, 0.5) 보간.계단.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_num(&out, "값1"), fixed("2"));
        assert_eq!(state_num(&out, "x2"), fixed("0.9375"));
        assert_eq!(state_num(&out, "v2"), fixed("-0.25"));
        assert_eq!(state_num(&out, "x3"), fixed("0.96875"));
        assert_eq!(state_num(&out, "v3"), fixed("-0.24609375"));
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
    fn butbak_decl_reassignment_fails_in_runtime() {
        let source = r#"
채비 { 상수:수 = 1. }.
상수 <- 2.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("butbak reassignment must fail"),
            Err(err) => err,
        };
        match err {
            RuntimeError::Pack { message, .. } => {
                assert!(message.contains("붙박이는 재대입할 수 없습니다"));
            }
            other => panic!("expected pack error, got {other:?}"),
        }
    }

    #[test]
    fn non_butbak_decl_reassignment_still_runs_in_runtime() {
        let source = r#"
채비 { 점수:수 <- 0. }.
점수 <- 1.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "점수"), fixed("1"));
    }

    #[test]
    fn relation_eq_infix_runtime_stores_relation_pack() {
        let source = r#"
관계 <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
"#;
        let output = run_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("관계".to_string())) else {
            panic!("관계 must be relation pack");
        };
        assert_eq!(
            pack.fields.get(RELATION_KIND_FIELD),
            Some(&Value::Str(RELATION_KIND_EQUATION.to_string()))
        );
        assert!(matches!(
            pack.fields.get(RELATION_LEFT_FIELD),
            Some(Value::Math(_))
        ));
        assert!(matches!(
            pack.fields.get(RELATION_RIGHT_FIELD),
            Some(Value::Math(_))
        ));
    }

    #[test]
    fn connect_jitgi_runtime_matches_relation_eq_pack() {
        let source = r#"
잇기관계 <- ((#ascii) 수식{2*x + 3}, (#ascii) 수식{7}) 잇기.
직접관계 <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
"#;
        let output = run_source_once(source).expect("run");
        let Some(Value::Pack(jitgi)) = output.state.get(&Key::new("잇기관계".to_string()))
        else {
            panic!("잇기관계 must be relation pack");
        };
        let Some(Value::Pack(direct)) = output.state.get(&Key::new("직접관계".to_string()))
        else {
            panic!("직접관계 must be relation pack");
        };
        assert_eq!(jitgi, direct);
        assert_eq!(
            jitgi.fields.get(RELATION_KIND_FIELD),
            Some(&Value::Str(RELATION_KIND_EQUATION.to_string()))
        );
    }

    #[test]
    fn connect_endpoint_equal_runtime_stores_endpoint_relation_pack() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint relation pack");
        };
        assert_eq!(
            pack.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_equality".to_string()))
        );
        assert_eq!(
            pack.fields.get("왼쪽"),
            Some(&Value::Str("전지.양극.전압".to_string()))
        );
        assert_eq!(
            pack.fields.get("오른쪽"),
            Some(&Value::Str("전구.왼핀.전압".to_string()))
        );
        assert_eq!(
            pack.fields.get("규칙"),
            Some(&Value::Str("같게".to_string()))
        );
        assert_eq!(
            pack.fields.get("채널"),
            Some(&Value::Str("전압".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_flow_runtime_stores_endpoint_flow_pack() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint flow pack");
        };
        assert_eq!(
            pack.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_flow".to_string()))
        );
        assert_eq!(
            pack.fields.get("왼쪽"),
            Some(&Value::Str("전지.양극.전류".to_string()))
        );
        assert_eq!(
            pack.fields.get("오른쪽"),
            Some(&Value::Str("전구.왼핀.전류".to_string()))
        );
        assert_eq!(
            pack.fields.get("규칙"),
            Some(&Value::Str("흐르게".to_string()))
        );
        assert_eq!(
            pack.fields.get("채널"),
            Some(&Value::Str("전류".to_string()))
        );
        assert_eq!(
            pack.fields.get("부호규약"),
            Some(&Value::Str("left_plus_right_zero".to_string()))
        );
        assert_eq!(
            pack.fields.get("방향"),
            Some(&Value::Str("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_reverse_flow_runtime_stores_endpoint_flow_pack() {
        let source = r#"
이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 거슬러 흐르게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint reverse flow pack");
        };
        assert_eq!(
            pack.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_flow".to_string()))
        );
        assert_eq!(
            pack.fields.get("왼쪽"),
            Some(&Value::Str("가계1.구매끝.돈".to_string()))
        );
        assert_eq!(
            pack.fields.get("오른쪽"),
            Some(&Value::Str("장터.소매끝.돈".to_string()))
        );
        assert_eq!(
            pack.fields.get("규칙"),
            Some(&Value::Str("거슬러 흐르게".to_string()))
        );
        assert_eq!(pack.fields.get("채널"), Some(&Value::Str("돈".to_string())));
        assert_eq!(
            pack.fields.get("부호규약"),
            Some(&Value::Str("left_plus_right_zero".to_string()))
        );
        assert_eq!(
            pack.fields.get("방향"),
            Some(&Value::Str("오른쪽에서왼쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_rejects_carried_property_runtime_surface() {
        assert!(!frontdoor_source_parses(
            r#"이음관계 <- 전지.양극과 전구.왼핀을 (재화가 돈에 실리게) 잇기."#
        ));
    }

    #[test]
    fn connect_endpoint_carried_property_forward_runtime_stores_relation_set_pack() {
        let source = r#"
이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        assert_eq!(
            pack.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_relation_set".to_string()))
        );
        let Some(Value::List(relations)) = pack.fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.items.len(), 2);
        let Value::Pack(carried) = &relations.items[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_carried_property".to_string()))
        );
        assert_eq!(
            carried.fields.get("왼쪽운반자"),
            Some(&Value::Str("은행.대출창구.대출금".to_string()))
        );
        assert_eq!(
            carried.fields.get("오른쪽운반자"),
            Some(&Value::Str("기업1.차입끝.대출금".to_string()))
        );
        assert_eq!(
            carried.fields.get("속성"),
            Some(&Value::Str("위험".to_string()))
        );
        assert_eq!(
            carried.fields.get("운반채널"),
            Some(&Value::Str("대출금".to_string()))
        );
        assert_eq!(
            carried.fields.get("운반규칙"),
            Some(&Value::Str("흐르게".to_string()))
        );
        assert_eq!(
            carried.fields.get("운반방향"),
            Some(&Value::Str("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_carried_property_reverse_runtime_stores_relation_set_pack() {
        let source = r#"
이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 거슬러 흐르게, 재화가 돈에 실리게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        let Some(Value::List(relations)) = pack.fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.items.len(), 2);
        let Value::Pack(carried) = &relations.items[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_carried_property".to_string()))
        );
        assert_eq!(
            carried.fields.get("속성"),
            Some(&Value::Str("재화".to_string()))
        );
        assert_eq!(
            carried.fields.get("운반채널"),
            Some(&Value::Str("돈".to_string()))
        );
        assert_eq!(
            carried.fields.get("운반규칙"),
            Some(&Value::Str("거슬러 흐르게".to_string()))
        );
        assert_eq!(
            carried.fields.get("운반방향"),
            Some(&Value::Str("오른쪽에서왼쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_multi_inner_runtime_stores_relation_set_pack() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        assert_eq!(
            pack.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_relation_set".to_string()))
        );
        assert_eq!(
            pack.fields.get("왼쪽끝"),
            Some(&Value::Str("전지.양극".to_string()))
        );
        assert_eq!(
            pack.fields.get("오른쪽끝"),
            Some(&Value::Str("전구.왼핀".to_string()))
        );
        let Some(Value::List(relations)) = pack.fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.items.len(), 2);
        let Value::Pack(first) = &relations.items[0] else {
            panic!("first relation must be a pack");
        };
        let Value::Pack(second) = &relations.items[1] else {
            panic!("second relation must be a pack");
        };
        assert_eq!(
            first.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_equality".to_string()))
        );
        assert_eq!(
            first.fields.get("채널"),
            Some(&Value::Str("전압".to_string()))
        );
        assert_eq!(
            second.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_flow".to_string()))
        );
        assert_eq!(
            second.fields.get("채널"),
            Some(&Value::Str("전류".to_string()))
        );
        assert_eq!(
            second.fields.get("방향"),
            Some(&Value::Str("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_multi_inner_econ_runtime_preserves_source_order() {
        let source = r#"
이음관계 <- 가계1.구매끝과 장터.소매끝을 (체결값은 같게, 재화는 흐르게, 돈은 거슬러 흐르게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        let Some(Value::List(relations)) = pack.fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.items.len(), 3);
        let channels = relations
            .items
            .iter()
            .map(|item| {
                let Value::Pack(pack) = item else {
                    panic!("relation item must be a pack");
                };
                pack.fields.get("채널").cloned()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            channels,
            vec![
                Some(Value::Str("체결값".to_string())),
                Some(Value::Str("재화".to_string())),
                Some(Value::Str("돈".to_string()))
            ]
        );
    }

    #[test]
    fn connect_endpoint_statement_append_same_pair_runtime_stores_statement_set_pack() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint statement set pack");
        };
        assert_eq!(
            pack.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_statement_set".to_string()))
        );
        assert_eq!(
            pack.fields.get("대상"),
            Some(&Value::Str("이음관계".to_string()))
        );
        assert_eq!(
            pack.fields.get("개수").map(Value::display).as_deref(),
            Some("2")
        );
        let Some(Value::List(items)) = pack.fields.get("이음들") else {
            panic!("이음들 must be a list");
        };
        assert_eq!(items.items.len(), 2);
        let Value::Pack(first) = &items.items[0] else {
            panic!("first statement must be a pack");
        };
        let Value::Pack(second) = &items.items[1] else {
            panic!("second statement must be a pack");
        };
        assert_eq!(
            first.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_equality".to_string()))
        );
        assert_eq!(
            second.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_flow".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_statement_append_mixed_pair_runtime_stores_statement_set_pack() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(pack)) = output.state.get(&Key::new("이음관계".to_string()))
        else {
            panic!("이음관계 must be endpoint statement set pack");
        };
        let Some(Value::List(items)) = pack.fields.get("이음들") else {
            panic!("이음들 must be a list");
        };
        assert_eq!(items.items.len(), 2);
        let Value::Pack(second) = &items.items[1] else {
            panic!("second statement must be a pack");
        };
        assert_eq!(
            second.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_relation_set".to_string()))
        );
        let Some(Value::List(relations)) = second.fields.get("관계들") else {
            panic!("mixed second statement 관계들 must be a list");
        };
        assert_eq!(relations.items.len(), 2);
        let Value::Pack(carried) = &relations.items[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_carried_property".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_rejects_statement_append_boundary_runtime_breaks_block() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
dt <- 1.
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
"#;
        let prepared = ddonirang_lang::preprocess_frontdoor_source(source);
        assert!(!prepared.contains("__이음관계종류: \"endpoint_statement_set\""));
    }

    #[test]
    fn connect_endpoint_normalize_single_runtime_returns_one_relation() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
목록 <- (이음관계) 이음관계.관계목록.
정규 <- (이음관계) 이음관계.정규화.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::List(list)) = output.state.get(&Key::new("목록".to_string())) else {
            panic!("목록 must be a list");
        };
        assert_eq!(list.items.len(), 1);
        let Value::Pack(first) = &list.items[0] else {
            panic!("first relation must be a pack");
        };
        assert_eq!(
            first.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_equality".to_string()))
        );
        let Some(Value::Pack(normalized)) = output.state.get(&Key::new("정규".to_string()))
        else {
            panic!("정규 must be a pack");
        };
        assert_eq!(
            normalized.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_relation_flat_set".to_string()))
        );
        assert_eq!(
            normalized.fields.get("개수").map(Value::display).as_deref(),
            Some("1")
        );
    }

    #[test]
    fn connect_endpoint_normalize_statement_set_flattens_nested_relation_set() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
목록 <- (이음관계) 이음관계.관계목록.
정규 <- (이음관계) 이음관계.정규화.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::List(list)) = output.state.get(&Key::new("목록".to_string())) else {
            panic!("목록 must be a list");
        };
        assert_eq!(list.items.len(), 3);
        let kinds = list
            .items
            .iter()
            .map(|item| {
                let Value::Pack(pack) = item else {
                    panic!("relation item must be a pack");
                };
                pack.fields.get("__이음관계종류").cloned()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                Some(Value::Str("endpoint_equality".to_string())),
                Some(Value::Str("endpoint_flow".to_string())),
                Some(Value::Str("endpoint_carried_property".to_string())),
            ]
        );
        let Some(Value::Pack(normalized)) = output.state.get(&Key::new("정규".to_string()))
        else {
            panic!("정규 must be a pack");
        };
        assert_eq!(
            normalized.fields.get("개수").map(Value::display).as_deref(),
            Some("3")
        );
        let Some(Value::List(relations)) = normalized.fields.get("관계들") else {
            panic!("normalized 관계들 must be a list");
        };
        assert_eq!(relations.items.len(), 3);
    }

    #[test]
    fn connect_endpoint_formula_relation_bridge_maps_endpoint_paths_to_ascii_vars() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
관계들 <- (이음관계) 이음관계.방정식목록.
방정식묶음 <- (이음관계) 이음관계.방정식화.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::List(relations)) = output.state.get(&Key::new("관계들".to_string()))
        else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.items.len(), 2);
        let Value::Pack(first_relation) = &relations.items[0] else {
            panic!("first formula relation must be a pack");
        };
        assert_eq!(
            first_relation.fields.get("__관계종류"),
            Some(&Value::Str("방정식".to_string()))
        );
        let Some(Value::Math(left)) = first_relation.fields.get("왼쪽") else {
            panic!("first left must be formula");
        };
        let Some(Value::Math(right)) = first_relation.fields.get("오른쪽") else {
            panic!("first right must be formula");
        };
        assert_eq!(left.body, "ep_001");
        assert_eq!(right.body, "ep_002");

        let Some(Value::Pack(set)) = output.state.get(&Key::new("방정식묶음".to_string()))
        else {
            panic!("방정식묶음 must be a pack");
        };
        assert_eq!(
            set.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_formula_relation_set".to_string()))
        );
        let Some(Value::List(mappings)) = set.fields.get("변수사상") else {
            panic!("변수사상 must be a list");
        };
        assert_eq!(mappings.items.len(), 2);
        let Value::Pack(first_mapping) = &mappings.items[0] else {
            panic!("first mapping must be a pack");
        };
        assert_eq!(
            first_mapping.fields.get("변수"),
            Some(&Value::Str("ep_001".to_string()))
        );
        assert_eq!(
            first_mapping.fields.get("경로"),
            Some(&Value::Str("전지.양극.전압".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_formula_relation_solve_uses_explicit_relation_list() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
관계들 <- 방정식묶음.관계들.
결과 <- (관계들) 방정식풀기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(result)) = output.state.get(&Key::new("결과".to_string())) else {
            panic!("결과 must be a pack");
        };
        assert_eq!(
            result.fields.get("__풀이결과종류"),
            Some(&Value::Str("성공".to_string()))
        );
        let Some(Value::Pack(bindings)) = result.fields.get("해") else {
            panic!("해 must be a pack");
        };
        assert!(bindings.fields.contains_key("ep_001"));
        assert!(bindings.fields.contains_key("ep_002"));
    }

    #[test]
    fn connect_endpoint_formula_relation_rejects_carried_property_metadata() {
        let source = r#"
이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
"#;
        assert!(run_frontdoor_source_once(source).is_err());
    }

    #[test]
    fn connect_endpoint_boundary_value_injection_appends_relation_packs() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
값관계들 <- (방정식묶음, 값들) 이음관계.값관계목록.
주입묶음 <- (방정식묶음, 값들) 이음관계.값주입.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::List(injected_relations)) =
            output.state.get(&Key::new("값관계들".to_string()))
        else {
            panic!("값관계들 must be a list");
        };
        assert_eq!(injected_relations.items.len(), 1);
        let Value::Pack(injected_relation) = &injected_relations.items[0] else {
            panic!("injected relation must be a pack");
        };
        assert_eq!(
            injected_relation.fields.get("__관계종류"),
            Some(&Value::Str("방정식".to_string()))
        );
        let Some(Value::Math(left)) = injected_relation.fields.get("왼쪽") else {
            panic!("injected left must be formula");
        };
        let Some(Value::Math(right)) = injected_relation.fields.get("오른쪽") else {
            panic!("injected right must be formula");
        };
        assert_eq!(left.body, "ep_001");
        assert_eq!(right.body, "5");

        let Some(Value::Pack(set)) = output.state.get(&Key::new("주입묶음".to_string())) else {
            panic!("주입묶음 must be a pack");
        };
        assert_eq!(
            set.fields.get("__이음관계종류"),
            Some(&Value::Str(
                "endpoint_formula_relation_set_with_values".to_string()
            ))
        );
        assert_eq!(
            set.fields.get("개수").map(Value::display).as_deref(),
            Some("2")
        );
        assert_eq!(
            set.fields.get("주입개수").map(Value::display).as_deref(),
            Some("1")
        );
        let Some(Value::List(all_relations)) = set.fields.get("관계들") else {
            panic!("combined 관계들 must be a list");
        };
        assert_eq!(all_relations.items.len(), 2);
    }

    #[test]
    fn connect_endpoint_boundary_value_solve_remap_accepts_injected_formula_set() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
주입묶음 <- (방정식묶음, 값들) 이음관계.값주입.
풀이 <- (주입묶음.관계들) 방정식풀기.
원복 <- (주입묶음, 풀이) 이음관계.풀이원복.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(remap)) = output.state.get(&Key::new("원복".to_string())) else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.fields.get("풀이결과종류"),
            Some(&Value::Str("성공".to_string()))
        );
        let Some(Value::List(values)) = remap.fields.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.items.len(), 2);
        let Value::Pack(first) = &values.items[0] else {
            panic!("first value must be a pack");
        };
        assert_eq!(
            first.fields.get("경로"),
            Some(&Value::Str("전지.양극.전압".to_string()))
        );
        assert_eq!(
            first.fields.get("값").map(Value::display).as_deref(),
            Some("5")
        );
    }

    #[test]
    fn connect_endpoint_boundary_value_rejects_unsupported_inputs() {
        let duplicate = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
값항목1 <- (경로="전지.양극.전압", 값=5).
값항목2 <- (경로="전지.양극.전압", 값=6).
값들 <- [값항목1, 값항목2].
주입묶음 <- (방정식묶음, 값들) 이음관계.값주입.
"#;
        let err = match run_frontdoor_source_once(duplicate) {
            Ok(_) => panic!("duplicate path must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_boundary_value_duplicate_path"),
            "{err:?}"
        );

        let unknown = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
값항목 <- (경로="없는.끝.전압", 값=5).
값들 <- [값항목].
주입묶음 <- (방정식묶음, 값들) 이음관계.값주입.
"#;
        let err = match run_frontdoor_source_once(unknown) {
            Ok(_) => panic!("unknown path must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_boundary_value_unknown_path"),
            "{err:?}"
        );

        let non_numeric = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
값항목 <- (경로="전지.양극.전압", 값="5").
값들 <- [값항목].
주입묶음 <- (방정식묶음, 값들) 이음관계.값주입.
"#;
        let err = match run_frontdoor_source_once(non_numeric) {
            Ok(_) => panic!("non numeric must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_boundary_value_non_numeric"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_equal_value_returns_endpoint_values() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(remap)) = output.state.get(&Key::new("원복".to_string())) else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_solve_result".to_string()))
        );
        assert_eq!(
            remap.fields.get("풀이결과종류"),
            Some(&Value::Str("성공".to_string()))
        );
        let Some(Value::List(values)) = remap.fields.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.items.len(), 2);
        let Value::Pack(first) = &values.items[0] else {
            panic!("first value must be a pack");
        };
        let Value::Pack(second) = &values.items[1] else {
            panic!("second value must be a pack");
        };
        assert_eq!(
            first.fields.get("값").map(Value::display).as_deref(),
            Some("5")
        );
        assert_eq!(
            second.fields.get("값").map(Value::display).as_deref(),
            Some("5")
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_flow_value_returns_opposite_endpoint_value() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
값항목 <- (경로="전지.양극.전류", 값=5).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(remap)) = output.state.get(&Key::new("원복".to_string())) else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.fields.get("풀이결과종류"),
            Some(&Value::Str("성공".to_string()))
        );
        let Some(Value::List(values)) = remap.fields.get("값들") else {
            panic!("값들 must be a list");
        };
        let Value::Pack(left) = &values.items[0] else {
            panic!("left value must be a pack");
        };
        let Value::Pack(right) = &values.items[1] else {
            panic!("right value must be a pack");
        };
        assert_eq!(
            left.fields.get("값").map(Value::display).as_deref(),
            Some("5")
        );
        assert_eq!(
            right.fields.get("값").map(Value::display).as_deref(),
            Some("-5")
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_flat_set_skips_public_normalize_step() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
정규 <- (이음관계) 이음관계.정규화.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
원복 <- (정규, 값들) 이음관계.풀기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(remap)) = output.state.get(&Key::new("원복".to_string())) else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.fields.get("풀이결과종류"),
            Some(&Value::Str("성공".to_string()))
        );
        let Some(Value::List(values)) = remap.fields.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.items.len(), 2);
    }

    #[test]
    fn connect_endpoint_explicit_solve_rejects_unsupported_and_boundary_errors() {
        let carried = r#"
이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
값들 <- [].
원복 <- (이음관계, 값들) 이음관계.풀기.
"#;
        assert!(run_frontdoor_source_once(carried).is_err());

        let duplicate = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목1 <- (경로="전지.양극.전압", 값=5).
값항목2 <- (경로="전지.양극.전압", 값=6).
값들 <- [값항목1, 값항목2].
원복 <- (이음관계, 값들) 이음관계.풀기.
"#;
        let err = match run_frontdoor_source_once(duplicate) {
            Ok(_) => panic!("duplicate path must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_boundary_value_duplicate_path"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_injection_records_unit_metadata() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
값항목 <- (경로="전지.양극.전압", 값=5@KRW).
값들 <- [값항목].
값관계들 <- (방정식묶음, 값들) 이음관계.값관계목록.
주입묶음 <- (방정식묶음, 값들) 이음관계.값주입.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::List(injected_relations)) =
            output.state.get(&Key::new("값관계들".to_string()))
        else {
            panic!("값관계들 must be a list");
        };
        let Value::Pack(relation) = &injected_relations.items[0] else {
            panic!("injected relation must be a pack");
        };
        let Some(Value::Math(right)) = relation.fields.get("오른쪽") else {
            panic!("injected right must be formula");
        };
        assert_eq!(right.body, "5");

        let Some(Value::Pack(set)) = output.state.get(&Key::new("주입묶음".to_string())) else {
            panic!("주입묶음 must be a pack");
        };
        let Some(Value::List(injected)) = set.fields.get("주입값들") else {
            panic!("주입값들 must be a list");
        };
        let Value::Pack(first) = &injected.items[0] else {
            panic!("injected value must be a pack");
        };
        assert_eq!(
            first.fields.get("값").map(Value::display).as_deref(),
            Some("5@KRW")
        );
        assert_eq!(
            first.fields.get("단위차원"),
            Some(&Value::Str("KRW".to_string()))
        );
        assert_eq!(
            first.fields.get("단위기호"),
            Some(&Value::Str("KRW".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_solve_remap_restores_unit_values() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
값항목 <- (경로="전지.양극.전류", 값=5@KRW).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(remap)) = output.state.get(&Key::new("원복".to_string())) else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.fields.get("풀이결과종류"),
            Some(&Value::Str("성공".to_string()))
        );
        let Some(Value::List(values)) = remap.fields.get("값들") else {
            panic!("값들 must be a list");
        };
        let Value::Pack(left) = &values.items[0] else {
            panic!("left value must be a pack");
        };
        let Value::Pack(right) = &values.items[1] else {
            panic!("right value must be a pack");
        };
        assert_eq!(
            left.fields.get("값").map(Value::display).as_deref(),
            Some("5@KRW")
        );
        assert_eq!(
            right.fields.get("값").map(Value::display).as_deref(),
            Some("-5@KRW")
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_rejects_dim_and_currency_conflicts() {
        let dim_conflict = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
값항목1 <- (경로="전지.양극.전압", 값=1@m).
값항목2 <- (경로="전구.왼핀.전압", 값=1@N).
값들 <- [값항목1, 값항목2].
주입묶음 <- (방정식묶음, 값들) 이음관계.값주입.
"#;
        let output = run_frontdoor_source_once(dim_conflict).expect("inject dim conflict");
        let injected = output
            .state
            .get(&Key::new("주입묶음".to_string()))
            .expect("주입묶음")
            .clone();
        let err = eval_endpoint_solve_result_remap(
            &[injected, endpoint_fake_solve_result()],
            crate::lang::span::Span::new(1, 1, 1, 1),
        )
        .expect_err("dimension conflict must fail");
        assert!(
            format!("{err:?}").contains("connect_unit_boundary_dim_conflict"),
            "{err:?}"
        );

        let incompatible = r#"
이음관계 <- 은행.왼끝과 기업.오른끝을 (돈은 같게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
값항목1 <- (경로="은행.왼끝.돈", 값=1@KRW).
값항목2 <- (경로="기업.오른끝.돈", 값=1@USD).
값들 <- [값항목1, 값항목2].
주입묶음 <- (방정식묶음, 값들) 이음관계.값주입.
"#;
        let output = run_frontdoor_source_once(incompatible).expect("inject incompatible unit");
        let injected = output
            .state
            .get(&Key::new("주입묶음".to_string()))
            .expect("주입묶음")
            .clone();
        let err = eval_endpoint_solve_result_remap(
            &[injected, endpoint_fake_solve_result()],
            crate::lang::span::Span::new(1, 1, 1, 1),
        )
        .expect_err("incompatible currency must fail");
        assert!(
            format!("{err:?}").contains("connect_unit_boundary_incompatible_unit"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_check_pass_and_violation() {
        let pass = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
범위 <- (경로="전지.양극.전압", 최소=0, 최대=10).
범위들 <- [범위].
검사 <- (원복, 범위들) 이음관계.범위검사.
위반들 <- (원복, 범위들) 이음관계.범위위반목록.
"#;
        let output = run_frontdoor_source_once(pass).expect("run");
        let Some(Value::Pack(check)) = output.state.get(&Key::new("검사".to_string())) else {
            panic!("검사 must be a pack");
        };
        assert_eq!(
            check.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_range_check".to_string()))
        );
        assert_eq!(
            check.fields.get("검사결과"),
            Some(&Value::Str("통과".to_string()))
        );
        assert_eq!(
            check.fields.get("위반개수").map(Value::display).as_deref(),
            Some("0")
        );
        let Some(Value::List(violations)) = output.state.get(&Key::new("위반들".to_string()))
        else {
            panic!("위반들 must be a list");
        };
        assert!(violations.items.is_empty());

        let fail = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
범위 <- (경로="전지.양극.전압", 최소=0, 최대=4).
범위들 <- [범위].
검사 <- (원복, 범위들) 이음관계.범위검사.
"#;
        let output = run_frontdoor_source_once(fail).expect("run");
        let Some(Value::Pack(check)) = output.state.get(&Key::new("검사".to_string())) else {
            panic!("검사 must be a pack");
        };
        assert_eq!(
            check.fields.get("검사결과"),
            Some(&Value::Str("실패".to_string()))
        );
        let Some(Value::List(violations)) = check.fields.get("위반들") else {
            panic!("위반들 must be a list");
        };
        assert_eq!(violations.items.len(), 1);
        let Value::Pack(first) = &violations.items[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.fields.get("이유"),
            Some(&Value::Str("above_max".to_string()))
        );
        assert_eq!(
            first.fields.get("값").map(Value::display).as_deref(),
            Some("5")
        );
        assert_eq!(
            first.fields.get("상한").map(Value::display).as_deref(),
            Some("4")
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_range_pass_and_fail() {
        let pass = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
범위들 <- [범위].
검사 <- (이음관계, 값들, 범위들) 이음관계.풀고범위검사.
위반들 <- (이음관계, 값들, 범위들) 이음관계.풀고범위위반목록.
"#;
        let output = run_frontdoor_source_once(pass).expect("run");
        let Some(Value::Pack(check)) = output.state.get(&Key::new("검사".to_string())) else {
            panic!("검사 must be a pack");
        };
        assert_eq!(
            check.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_solve_range_check".to_string()))
        );
        assert_eq!(
            check.fields.get("풀이결과종류"),
            Some(&Value::Str("성공".to_string()))
        );
        assert_eq!(
            check.fields.get("검사결과"),
            Some(&Value::Str("통과".to_string()))
        );
        assert_eq!(
            check.fields.get("위반개수").map(Value::display).as_deref(),
            Some("0")
        );
        let Some(Value::List(violations)) = output.state.get(&Key::new("위반들".to_string()))
        else {
            panic!("위반들 must be a list");
        };
        assert!(violations.items.is_empty());

        let fail = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
값항목 <- (경로="전지.양극.전류", 값=5).
값들 <- [값항목].
범위 <- (경로="전구.왼핀.전류", 최소=0, 최대=10).
범위들 <- [범위].
검사 <- (이음관계, 값들, 범위들) 이음관계.풀고범위검사.
"#;
        let output = run_frontdoor_source_once(fail).expect("run");
        let Some(Value::Pack(check)) = output.state.get(&Key::new("검사".to_string())) else {
            panic!("검사 must be a pack");
        };
        assert_eq!(
            check.fields.get("검사결과"),
            Some(&Value::Str("실패".to_string()))
        );
        let Some(Value::Pack(range_check)) = check.fields.get("범위검사") else {
            panic!("범위검사 must be a pack");
        };
        let Some(Value::List(violations)) = range_check.fields.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let Value::Pack(first) = &violations.items[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.fields.get("이유"),
            Some(&Value::Str("below_min".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_range_missing_and_error_propagation() {
        let missing = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값들 <- [].
범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
범위들 <- [범위].
검사 <- (이음관계, 값들, 범위들) 이음관계.풀고범위검사.
"#;
        let output = run_frontdoor_source_once(missing).expect("run");
        let Some(Value::Pack(check)) = output.state.get(&Key::new("검사".to_string())) else {
            panic!("검사 must be a pack");
        };
        assert_eq!(
            check.fields.get("검사결과"),
            Some(&Value::Str("실패".to_string()))
        );
        let Some(Value::Pack(range_check)) = check.fields.get("범위검사") else {
            panic!("범위검사 must be a pack");
        };
        let Some(Value::List(violations)) = range_check.fields.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let Value::Pack(first) = &violations.items[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.fields.get("이유"),
            Some(&Value::Str("missing_value".to_string()))
        );
        assert!(!first.fields.contains_key("값"));

        let duplicate_range = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
범위1 <- (경로="전구.왼핀.전압", 최소=0).
범위2 <- (경로="전구.왼핀.전압", 최대=10).
범위들 <- [범위1, 범위2].
검사 <- (이음관계, 값들, 범위들) 이음관계.풀고범위검사.
"#;
        let err = match run_frontdoor_source_once(duplicate_range) {
            Ok(_) => panic!("duplicate range path must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_boundary_range_duplicate_path"),
            "{err:?}"
        );

        let duplicate_value = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목1 <- (경로="전지.양극.전압", 값=5).
값항목2 <- (경로="전지.양극.전압", 값=6).
값들 <- [값항목1, 값항목2].
범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
범위들 <- [범위].
검사 <- (이음관계, 값들, 범위들) 이음관계.풀고범위검사.
"#;
        let err = match run_frontdoor_source_once(duplicate_value) {
            Ok(_) => panic!("duplicate boundary path must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_boundary_value_duplicate_path"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_report_pass_rows_are_endpoint_ordered() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
범위들 <- [범위].
보고서 <- (이음관계, 값들, 범위들) 이음관계.풀고범위보고서.
행목록 <- (이음관계, 값들, 범위들) 이음관계.풀고범위행목록.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(report)) = output.state.get(&Key::new("보고서".to_string())) else {
            panic!("보고서 must be a pack");
        };
        assert_eq!(
            report.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_solve_range_report".to_string()))
        );
        assert_eq!(
            report.fields.get("검사결과"),
            Some(&Value::Str("통과".to_string()))
        );
        assert_eq!(
            report.fields.get("행개수").map(Value::display).as_deref(),
            Some("2")
        );
        assert_eq!(
            report.fields.get("값개수").map(Value::display).as_deref(),
            Some("2")
        );
        assert_eq!(
            report.fields.get("누락개수").map(Value::display).as_deref(),
            Some("0")
        );
        let Some(Value::List(rows)) = report.fields.get("행들") else {
            panic!("행들 must be a list");
        };
        let Some(Value::List(row_list)) = output.state.get(&Key::new("행목록".to_string()))
        else {
            panic!("행목록 must be a list");
        };
        assert_eq!(rows.items.len(), 2);
        assert_eq!(row_list.items.len(), 2);
        let Value::Pack(left) = &rows.items[0] else {
            panic!("left row must be a pack");
        };
        let Value::Pack(right) = &rows.items[1] else {
            panic!("right row must be a pack");
        };
        assert_eq!(
            left.fields.get("범위상태"),
            Some(&Value::Str("범위없음".to_string()))
        );
        assert_eq!(
            right.fields.get("범위상태"),
            Some(&Value::Str("통과".to_string()))
        );
        assert_eq!(
            right.fields.get("하한").map(Value::display).as_deref(),
            Some("0")
        );
        assert_eq!(
            right.fields.get("상한").map(Value::display).as_deref(),
            Some("10")
        );
    }

    #[test]
    fn connect_endpoint_solve_range_report_missing_rows_distinguish_range_presence() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값들 <- [].
범위 <- (경로="전지.양극.전압", 최소=0, 최대=10).
범위들 <- [범위].
보고서 <- (이음관계, 값들, 범위들) 이음관계.풀고범위보고서.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(report)) = output.state.get(&Key::new("보고서".to_string())) else {
            panic!("보고서 must be a pack");
        };
        assert_eq!(
            report.fields.get("풀이결과종류"),
            Some(&Value::Str("실패".to_string()))
        );
        assert_eq!(
            report.fields.get("검사결과"),
            Some(&Value::Str("실패".to_string()))
        );
        assert_eq!(
            report.fields.get("누락개수").map(Value::display).as_deref(),
            Some("2")
        );
        let Some(Value::List(rows)) = report.fields.get("행들") else {
            panic!("행들 must be a list");
        };
        let Value::Pack(left) = &rows.items[0] else {
            panic!("left row must be a pack");
        };
        let Value::Pack(right) = &rows.items[1] else {
            panic!("right row must be a pack");
        };
        assert_eq!(
            left.fields.get("값상태"),
            Some(&Value::Str("누락".to_string()))
        );
        assert_eq!(
            left.fields.get("범위상태"),
            Some(&Value::Str("실패".to_string()))
        );
        let Some(Value::List(violations)) = left.fields.get("위반들") else {
            panic!("left violations must be a list");
        };
        let Value::Pack(first) = &violations.items[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.fields.get("이유"),
            Some(&Value::Str("missing_value".to_string()))
        );
        assert!(!first.fields.contains_key("값"));
        assert_eq!(
            right.fields.get("값상태"),
            Some(&Value::Str("누락".to_string()))
        );
        assert_eq!(
            right.fields.get("범위상태"),
            Some(&Value::Str("범위없음".to_string()))
        );
        assert_eq!(
            right.fields.get("위반개수").map(Value::display).as_deref(),
            Some("0")
        );
    }

    #[test]
    fn connect_endpoint_solve_range_text_report_formats_tsv_rows() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
범위들 <- [범위].
보고서 <- (이음관계, 값들, 범위들) 이음관계.풀고범위보고서.
문자표 <- (보고서) 이음관계.보고서문자표.
직접문자표 <- (이음관계, 값들, 범위들) 이음관계.풀고범위문자표.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let text = state_str(&output, "문자표");
        assert_eq!(text, state_str(&output, "직접문자표"));
        assert!(text.starts_with("변수\t경로\t값상태\t값\t범위상태\t하한\t상한\t위반"));
        assert!(text.contains("ep_001\t전지.양극.전압\t값있음\t5\t범위없음\t\t\t"));
        assert!(text.contains("ep_002\t전구.왼핀.전압\t값있음\t5\t통과\t0\t10\t"));
        assert!(!text.ends_with('\n'));
    }

    #[test]
    fn connect_endpoint_solve_range_text_report_rejects_non_report_input() {
        let source = r#"
보고서 <- (아무것=1).
문자표 <- (보고서) 이음관계.보고서문자표.
"#;
        let err = match run_frontdoor_source_once(source) {
            Ok(_) => panic!("non report must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_report_text_expected_solve_range_report"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_records_expectation_matrix() {
        let source = r#"
통과이음 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
통과값 <- (경로="전지.양극.전압", 값=5).
통과범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
통과케이스 <- (이름="pass-default", 이음관계=통과이음, 값들=[통과값], 범위들=[통과범위]).

실패이음 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
실패값 <- (경로="전지.양극.전류", 값=5).
실패범위 <- (경로="전구.왼핀.전류", 최소=0, 최대=10).
예상실패케이스 <- (이름="expected-fail", 이음관계=실패이음, 값들=[실패값], 범위들=[실패범위], 기대검사결과="실패").
예상통과실패케이스 <- (이름="unexpected-fail", 이음관계=실패이음, 값들=[실패값], 범위들=[실패범위]).
예상실패통과케이스 <- (이름="unexpected-success", 이음관계=통과이음, 값들=[통과값], 범위들=[통과범위], 기대검사결과="실패").

스위트 <- ([통과케이스, 예상실패케이스, 예상통과실패케이스, 예상실패통과케이스]) 이음관계.풀고범위스위트.
요약 <- (스위트) 이음관계.풀고범위스위트문자표.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(suite)) = output.state.get(&Key::new("스위트".to_string())) else {
            panic!("스위트 must be a pack");
        };
        assert_eq!(
            suite.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_solve_range_case_suite".to_string()))
        );
        assert_eq!(suite.fields.get("전체통과"), Some(&Value::Bool(false)));
        assert_eq!(
            suite.fields.get("통과개수").map(Value::display).as_deref(),
            Some("2")
        );
        assert_eq!(
            suite.fields.get("실패개수").map(Value::display).as_deref(),
            Some("2")
        );
        let summary = state_str(&output, "요약");
        assert!(summary.starts_with("이름\t기대\t실제\t통과"));
        assert!(summary.contains("pass-default\t통과\t통과\t참"));
        assert!(summary.contains("expected-fail\t실패\t실패\t참"));
        assert!(summary.contains("unexpected-fail\t통과\t실패\t거짓"));
        assert!(summary.contains("unexpected-success\t실패\t통과\t거짓"));
        assert!(!summary.ends_with('\n'));
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_rejects_bad_inputs() {
        let bad_expected = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
케이스 <- (이름="bad", 이음관계=이음관계, 값들=[값항목], 범위들=[범위], 기대검사결과="모름").
결과 <- (케이스) 이음관계.풀고범위케이스.
"#;
        let err = match run_frontdoor_source_once(bad_expected) {
            Ok(_) => panic!("bad expected result must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_case_suite_invalid_expected_result"),
            "{err:?}"
        );

        let bad_text = r#"
스위트 <- (아무것=1).
요약 <- (스위트) 이음관계.풀고범위스위트문자표.
"#;
        let err = match run_frontdoor_source_once(bad_text) {
            Ok(_) => panic!("non suite must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_case_suite_text_expected_suite"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_detail_formats_sections() {
        let source = r#"
전압이음 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
전압값 <- (경로="전지.양극.전압", 값=5).
전압범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
전압케이스 <- (이름="voltage-pass", 이음관계=전압이음, 값들=[전압값], 범위들=[전압범위]).

전류이음 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
전류값 <- (경로="전지.양극.전류", 값=5).
전류범위 <- (경로="전구.왼핀.전류", 최소=-10, 최대=0).
전류케이스 <- (이름="flow-pass", 이음관계=전류이음, 값들=[전류값], 범위들=[전류범위]).

케이스들 <- [전압케이스, 전류케이스].
스위트 <- (케이스들) 이음관계.풀고범위스위트.
상세 <- (스위트) 이음관계.풀고범위스위트상세문자표.
직접상세 <- (케이스들) 이음관계.풀고범위실행상세문자표.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let detail = state_str(&output, "상세");
        let direct = state_str(&output, "직접상세");
        assert_eq!(detail, direct);
        assert!(detail.starts_with("이름\t기대\t실제\t통과"));
        assert!(
            detail.contains("\n\n## voltage-pass\n기대\t통과\n실제\t통과\n통과\t참\n변수\t경로")
        );
        assert!(detail.contains("\n\n## flow-pass\n기대\t통과\n실제\t통과\n통과\t참\n변수\t경로"));
        assert!(detail.contains("ep_002\t전구.왼핀.전류\t값있음\t-5\t통과\t-10\t0\t"));
        assert!(!detail.ends_with('\n'));
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_detail_rejects_bad_inputs() {
        let bad_suite = r#"
스위트 <- (아무것=1).
상세 <- (스위트) 이음관계.풀고범위스위트상세문자표.
"#;
        let err = match run_frontdoor_source_once(bad_suite) {
            Ok(_) => panic!("non suite must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_case_suite_detail_expected_suite"),
            "{err:?}"
        );

        let malformed = r#"
나쁜결과 <- (__이음관계종류="endpoint_solve_range_case_result", 이름="bad", 기대검사결과="통과", 실제검사결과="통과", 통과여부=참).
스위트 <- (__이음관계종류="endpoint_solve_range_case_suite", 결과들=[나쁜결과]).
상세 <- (스위트) 이음관계.풀고범위스위트상세문자표.
"#;
        let err = match run_frontdoor_source_once(malformed) {
            Ok(_) => panic!("malformed case result must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_case_suite_detail_malformed_case_result"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_summary_records_mismatch_lists() {
        let source = r#"
전압이음 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
전압값 <- (경로="전지.양극.전압", 값=5).
전압범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
통과케이스 <- (이름="expected-pass-actual-pass", 이음관계=전압이음, 값들=[전압값], 범위들=[전압범위]).
예상실패실제통과 <- (이름="expected-fail-actual-pass", 이음관계=전압이음, 값들=[전압값], 범위들=[전압범위], 기대검사결과="실패").

전류이음 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
전류값 <- (경로="전지.양극.전류", 값=5).
실패범위 <- (경로="전구.왼핀.전류", 최소=0, 최대=10).
예상통과실제실패 <- (이름="expected-pass-actual-fail", 이음관계=전류이음, 값들=[전류값], 범위들=[실패범위]).
예상실패실제실패 <- (이름="expected-fail-actual-fail", 이음관계=전류이음, 값들=[전류값], 범위들=[실패범위], 기대검사결과="실패").

케이스들 <- [통과케이스, 예상실패실제실패, 예상통과실제실패, 예상실패실제통과].
스위트 <- (케이스들) 이음관계.풀고범위스위트.
요약 <- (스위트) 이음관계.풀고범위스위트요약.
직접요약 <- (케이스들) 이음관계.풀고범위실행요약.
요약종류 <- 요약.__이음관계종류.
개수 <- 요약.개수.
통과개수 <- 요약.통과개수.
실패개수 <- 요약.실패개수.
전체통과 <- 요약.전체통과.
통과들 <- 요약.통과케이스들.
실패들 <- 요약.실패케이스들.
기대실패통과들 <- 요약.기대실패통과케이스들.
기대통과실패들 <- 요약.기대통과실패케이스들.
직접실패들 <- 직접요약.실패케이스들.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        assert_eq!(
            state_str(&output, "요약종류"),
            "endpoint_solve_range_case_suite_summary"
        );
        assert_eq!(state_num(&output, "개수"), fixed("4"));
        assert_eq!(state_num(&output, "통과개수"), fixed("2"));
        assert_eq!(state_num(&output, "실패개수"), fixed("2"));
        assert!(!state_bool(&output, "전체통과"));
        assert_eq!(
            state_list_strings(&output, "통과들"),
            vec!["expected-pass-actual-pass", "expected-fail-actual-fail"]
        );
        assert_eq!(
            state_list_strings(&output, "실패들"),
            vec!["expected-pass-actual-fail", "expected-fail-actual-pass"]
        );
        assert_eq!(
            state_list_strings(&output, "기대실패통과들"),
            vec!["expected-fail-actual-pass"]
        );
        assert_eq!(
            state_list_strings(&output, "기대통과실패들"),
            vec!["expected-pass-actual-fail"]
        );
        assert_eq!(
            state_list_strings(&output, "직접실패들"),
            state_list_strings(&output, "실패들")
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_summary_rejects_bad_inputs() {
        let bad_suite = r#"
스위트 <- (아무것=1).
요약 <- (스위트) 이음관계.풀고범위스위트요약.
"#;
        let err = match run_frontdoor_source_once(bad_suite) {
            Ok(_) => panic!("non suite must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_case_suite_summary_expected_suite"),
            "{err:?}"
        );

        let malformed = r#"
나쁜결과 <- (__이음관계종류="endpoint_solve_range_case_result", 이름="bad", 기대검사결과="통과", 실제검사결과="통과").
스위트 <- (__이음관계종류="endpoint_solve_range_case_suite", 결과들=[나쁜결과]).
요약 <- (스위트) 이음관계.풀고범위스위트요약.
"#;
        let err = match run_frontdoor_source_once(malformed) {
            Ok(_) => panic!("malformed case result must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_case_suite_summary_malformed_case_result"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_check_records_pass_fail() {
        let source = r#"
전압이음 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
전압값 <- (경로="전지.양극.전압", 값=5).
전압범위 <- (경로="전구.왼핀.전압", 최소=0, 최대=10).
통과케이스 <- (이름="expected-pass-actual-pass", 이음관계=전압이음, 값들=[전압값], 범위들=[전압범위]).
예상실패실제통과 <- (이름="expected-fail-actual-pass", 이음관계=전압이음, 값들=[전압값], 범위들=[전압범위], 기대검사결과="실패").

전류이음 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
전류값 <- (경로="전지.양극.전류", 값=5).
실패범위 <- (경로="전구.왼핀.전류", 최소=0, 최대=10).
예상통과실제실패 <- (이름="expected-pass-actual-fail", 이음관계=전류이음, 값들=[전류값], 범위들=[실패범위]).
예상실패실제실패 <- (이름="expected-fail-actual-fail", 이음관계=전류이음, 값들=[전류값], 범위들=[실패범위], 기대검사결과="실패").

케이스들 <- [통과케이스, 예상실패실제실패, 예상통과실제실패, 예상실패실제통과].
스위트 <- (케이스들) 이음관계.풀고범위스위트.
요약 <- (스위트) 이음관계.풀고범위스위트요약.
판정 <- (요약) 이음관계.풀고범위스위트판정.
직접판정 <- (케이스들) 이음관계.풀고범위실행판정.
판정종류 <- 판정.__이음관계종류.
판정값 <- 판정.판정.
개수 <- 판정.개수.
통과개수 <- 판정.통과개수.
실패개수 <- 판정.실패개수.
전체통과 <- 판정.전체통과.
실패들 <- 판정.실패케이스들.
기대실패통과들 <- 판정.기대실패통과케이스들.
기대통과실패들 <- 판정.기대통과실패케이스들.
직접실패들 <- 직접판정.실패케이스들.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        assert_eq!(
            state_str(&output, "판정종류"),
            "endpoint_solve_range_case_suite_check"
        );
        assert_eq!(state_str(&output, "판정값"), "실패");
        assert_eq!(state_num(&output, "개수"), fixed("4"));
        assert_eq!(state_num(&output, "통과개수"), fixed("2"));
        assert_eq!(state_num(&output, "실패개수"), fixed("2"));
        assert!(!state_bool(&output, "전체통과"));
        assert_eq!(
            state_list_strings(&output, "실패들"),
            vec!["expected-pass-actual-fail", "expected-fail-actual-pass"]
        );
        assert_eq!(
            state_list_strings(&output, "기대실패통과들"),
            vec!["expected-fail-actual-pass"]
        );
        assert_eq!(
            state_list_strings(&output, "기대통과실패들"),
            vec!["expected-pass-actual-fail"]
        );
        assert_eq!(
            state_list_strings(&output, "직접실패들"),
            state_list_strings(&output, "실패들")
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_check_rejects_bad_inputs() {
        let bad_summary = r#"
요약 <- (아무것=1).
판정 <- (요약) 이음관계.풀고범위스위트판정.
"#;
        let err = match run_frontdoor_source_once(bad_summary) {
            Ok(_) => panic!("non summary must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_case_suite_check_expected_summary"),
            "{err:?}"
        );

        let malformed = r#"
요약 <- (__이음관계종류="endpoint_solve_range_case_suite_summary", 전체통과=참).
판정 <- (요약) 이음관계.풀고범위스위트판정.
"#;
        let err = match run_frontdoor_source_once(malformed) {
            Ok(_) => panic!("malformed summary must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_case_suite_check_malformed_summary"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_missing_value_is_soft_violation() {
        let remap = endpoint_partial_range_solve_result();
        let range = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(Value::Num(Quantity::new(
                Fixed64::from_int(0),
                UnitDim::zero(),
            ))),
            Some(Value::Num(Quantity::new(
                Fixed64::from_int(10),
                UnitDim::zero(),
            ))),
        )]);
        let checked = eval_endpoint_boundary_range_check(
            &[remap, range],
            crate::lang::span::Span::new(1, 1, 1, 1),
        )
        .expect("range check");
        let Value::Pack(check) = checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.fields.get("검사결과"),
            Some(&Value::Str("실패".to_string()))
        );
        let Some(Value::List(violations)) = check.fields.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let Value::Pack(first) = &violations.items[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.fields.get("이유"),
            Some(&Value::Str("missing_value".to_string()))
        );
        assert!(!first.fields.contains_key("값"));
        assert_eq!(
            first.fields.get("하한").map(Value::display).as_deref(),
            Some("0")
        );
        assert_eq!(
            first.fields.get("상한").map(Value::display).as_deref(),
            Some("10")
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_unit_policy_and_errors() {
        let pass = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
값항목 <- (경로="전지.양극.전류", 값=5@KRW).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
범위 <- (경로="전구.왼핀.전류", 최소=-10@KRW, 최대=0@KRW).
범위들 <- [범위].
검사 <- (원복, 범위들) 이음관계.범위검사.
"#;
        let output = run_frontdoor_source_once(pass).expect("run");
        let Some(Value::Pack(check)) = output.state.get(&Key::new("검사".to_string())) else {
            panic!("검사 must be a pack");
        };
        assert_eq!(
            check.fields.get("검사결과"),
            Some(&Value::Str("통과".to_string()))
        );

        let incompatible = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
값항목 <- (경로="전지.양극.전류", 값=5@KRW).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
범위 <- (경로="전지.양극.전류", 최대=10@USD).
범위들 <- [범위].
검사 <- (원복, 범위들) 이음관계.범위검사.
"#;
        let err = match run_frontdoor_source_once(incompatible) {
            Ok(_) => panic!("incompatible unit must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_boundary_range_incompatible_unit"),
            "{err:?}"
        );

        let dim_conflict = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
값항목 <- (경로="전지.양극.전류", 값=5@KRW).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
범위 <- (경로="전지.양극.전류", 최대=10@m).
범위들 <- [범위].
검사 <- (원복, 범위들) 이음관계.범위검사.
"#;
        let err = match run_frontdoor_source_once(dim_conflict) {
            Ok(_) => panic!("dimension conflict must fail"),
            Err(err) => err,
        };
        assert!(
            format!("{err:?}").contains("connect_boundary_range_dim_conflict"),
            "{err:?}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_rejects_unsupported_inputs() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
값항목 <- (경로="전지.양극.전압", 값=5).
값들 <- [값항목].
원복 <- (이음관계, 값들) 이음관계.풀기.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let remap = output
            .state
            .get(&Key::new("원복".to_string()))
            .expect("원복")
            .clone();

        let duplicate = endpoint_range_test_values_raw(vec![
            (
                "전지.양극.전압",
                Some(Value::Num(Quantity::new(
                    Fixed64::from_int(0),
                    UnitDim::zero(),
                ))),
                None,
            ),
            (
                "전지.양극.전압",
                None,
                Some(Value::Num(Quantity::new(
                    Fixed64::from_int(10),
                    UnitDim::zero(),
                ))),
            ),
        ]);
        let err = eval_endpoint_boundary_range_check(
            &[remap.clone(), duplicate],
            crate::lang::span::Span::new(1, 1, 1, 1),
        )
        .expect_err("duplicate path must fail");
        assert!(
            format!("{err:?}").contains("connect_boundary_range_duplicate_path"),
            "{err:?}"
        );

        let unknown = endpoint_range_test_values_raw(vec![(
            "없는.끝.전압",
            Some(Value::Num(Quantity::new(
                Fixed64::from_int(0),
                UnitDim::zero(),
            ))),
            None,
        )]);
        let err = eval_endpoint_boundary_range_check(
            &[remap.clone(), unknown],
            crate::lang::span::Span::new(1, 1, 1, 1),
        )
        .expect_err("unknown path must fail");
        assert!(
            format!("{err:?}").contains("connect_boundary_range_unknown_path"),
            "{err:?}"
        );

        let mut bad = BTreeMap::new();
        bad.insert("경로".to_string(), Value::Str("전지.양극.전압".to_string()));
        bad.insert("최소".to_string(), Value::Str("0".to_string()));
        let err = eval_endpoint_boundary_range_check(
            &[
                remap,
                Value::List(ListValue {
                    items: vec![Value::Pack(PackValue { fields: bad })],
                }),
            ],
            crate::lang::span::Span::new(1, 1, 1, 1),
        )
        .expect_err("non numeric must fail");
        assert!(
            format!("{err:?}").contains("connect_boundary_range_non_numeric"),
            "{err:?}"
        );
    }

    fn endpoint_fake_solve_result() -> Value {
        let mut bindings = BTreeMap::new();
        bindings.insert("ep_001".to_string(), fixed_value(1));
        bindings.insert("ep_002".to_string(), fixed_value(1));
        let mut result = BTreeMap::new();
        result.insert(
            RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
            Value::Str(RELATION_SOLVE_RESULT_SUCCESS.to_string()),
        );
        result.insert(
            RELATION_SOLVE_BINDINGS_FIELD.to_string(),
            Value::Pack(PackValue { fields: bindings }),
        );
        Value::Pack(PackValue { fields: result })
    }

    fn endpoint_partial_range_solve_result() -> Value {
        let mut first_mapping = BTreeMap::new();
        first_mapping.insert("변수".to_string(), Value::Str("ep_001".to_string()));
        first_mapping.insert("경로".to_string(), Value::Str("전지.양극.전압".to_string()));
        let mut second_mapping = BTreeMap::new();
        second_mapping.insert("변수".to_string(), Value::Str("ep_002".to_string()));
        second_mapping.insert("경로".to_string(), Value::Str("전구.왼핀.전압".to_string()));

        let mut first_value = BTreeMap::new();
        first_value.insert("변수".to_string(), Value::Str("ep_001".to_string()));
        first_value.insert("경로".to_string(), Value::Str("전지.양극.전압".to_string()));
        first_value.insert(
            "값".to_string(),
            Value::Num(Quantity::new(Fixed64::from_int(5), UnitDim::zero())),
        );

        let mut fields = BTreeMap::new();
        fields.insert(
            "__이음관계종류".to_string(),
            Value::Str("endpoint_solve_result".to_string()),
        );
        fields.insert(
            "풀이결과종류".to_string(),
            Value::Str("부분성공".to_string()),
        );
        fields.insert(
            "값들".to_string(),
            Value::List(ListValue {
                items: vec![Value::Pack(PackValue {
                    fields: first_value,
                })],
            }),
        );
        fields.insert(
            "누락변수들".to_string(),
            Value::List(ListValue {
                items: vec![Value::Str("ep_002".to_string())],
            }),
        );
        fields.insert(
            "변수사상".to_string(),
            Value::List(ListValue {
                items: vec![
                    Value::Pack(PackValue {
                        fields: first_mapping,
                    }),
                    Value::Pack(PackValue {
                        fields: second_mapping,
                    }),
                ],
            }),
        );
        fields.insert("원래풀이".to_string(), endpoint_fake_solve_result());
        Value::Pack(PackValue { fields })
    }

    fn endpoint_range_test_values_raw(items: Vec<(&str, Option<Value>, Option<Value>)>) -> Value {
        Value::List(ListValue {
            items: items
                .into_iter()
                .map(|(path, min, max)| {
                    let mut fields = BTreeMap::new();
                    fields.insert("경로".to_string(), Value::Str(path.to_string()));
                    if let Some(min) = min {
                        fields.insert("최소".to_string(), min);
                    }
                    if let Some(max) = max {
                        fields.insert("최대".to_string(), max);
                    }
                    Value::Pack(PackValue { fields })
                })
                .collect(),
        })
    }

    #[test]
    fn connect_endpoint_solve_result_remap_success_returns_endpoint_values() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
관계들 <- 방정식묶음.관계들.
풀이 <- (관계들) 방정식풀기.
원복 <- (방정식묶음, 풀이) 이음관계.풀이원복.
값들 <- (방정식묶음, 풀이) 이음관계.풀이값목록.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(remap)) = output.state.get(&Key::new("원복".to_string())) else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.fields.get("__이음관계종류"),
            Some(&Value::Str("endpoint_solve_result".to_string()))
        );
        assert_eq!(
            remap.fields.get("풀이결과종류"),
            Some(&Value::Str("성공".to_string()))
        );
        let Some(Value::List(values)) = remap.fields.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.items.len(), 2);
        let Value::Pack(first) = &values.items[0] else {
            panic!("first value must be a pack");
        };
        assert_eq!(
            first.fields.get("변수"),
            Some(&Value::Str("ep_001".to_string()))
        );
        assert_eq!(
            first.fields.get("경로"),
            Some(&Value::Str("전지.양극.전압".to_string()))
        );
        let Some(Value::List(value_list)) = output.state.get(&Key::new("값들".to_string()))
        else {
            panic!("풀이값목록 must be a list");
        };
        assert_eq!(value_list.items.len(), 2);
    }

    #[test]
    fn connect_endpoint_solve_result_remap_partial_records_missing_variables() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
부분관계 <- ((#ascii) 수식{ep_001}) =:= ((#ascii) 수식{0}).
부분풀이 <- (부분관계) 방정식풀기.
원복 <- (방정식묶음, 부분풀이) 이음관계.풀이원복.
"#;
        let output = run_frontdoor_source_once(source).expect("run");
        let Some(Value::Pack(remap)) = output.state.get(&Key::new("원복".to_string())) else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.fields.get("풀이결과종류"),
            Some(&Value::Str("부분성공".to_string()))
        );
        let Some(Value::List(values)) = remap.fields.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.items.len(), 1);
        let Some(Value::List(missing)) = remap.fields.get("누락변수들") else {
            panic!("누락변수들 must be a list");
        };
        assert_eq!(missing.items, vec![Value::Str("ep_002".to_string())]);
    }

    #[test]
    fn connect_endpoint_solve_result_remap_rejects_unknown_solver_binding() {
        let source = r#"
이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
방정식묶음 <- (이음관계) 이음관계.방정식화.
나쁜관계 <- ((#ascii) 수식{ep_999}) =:= ((#ascii) 수식{0}).
나쁜풀이 <- (나쁜관계) 방정식풀기.
원복 <- (방정식묶음, 나쁜풀이) 이음관계.풀이원복.
"#;
        assert!(run_frontdoor_source_once(source).is_err());
    }

    #[test]
    fn connect_endpoint_rejects_unsupported_multi_inner_sentence_runtime_surface() {
        assert!(!frontdoor_source_parses(
            r#"이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 재화가 돈에 실리게) 잇기."#
        ));
    }

    #[test]
    fn connect_endpoint_rejects_duplicate_carrier_flow_runtime_surface() {
        assert!(!frontdoor_source_parses(
            r#"이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 흐르게, 돈은 거슬러 흐르게, 재화가 돈에 실리게) 잇기."#
        ));
    }

    #[test]
    fn connect_endpoint_rejects_empty_multi_inner_sentence_runtime_surface() {
        assert!(!frontdoor_source_parses(
            r#"이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, ) 잇기."#
        ));
    }

    #[test]
    fn connect_endpoint_rejects_whole_object_shorthand_runtime_surface() {
        assert!(!frontdoor_source_parses(
            r#"이음관계 <- 전구와 전지를 (전압은 같게) 잇기."#
        ));
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
    fn call_tail_bare_statement_dispatches_to_seed_alias() {
        let source = r#"
돕~도우:움직씨 = {
살림.결과 <- "도움".
}.

돕기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_str(&output, "결과"), "도움");
    }

    #[test]
    fn call_tail_short_and_long_forms_dispatch_to_same_seed() {
        let source = r#"
회복:움직씨 = {
살림.횟수 <- 살림.횟수 + 1.
}.

살림.횟수 <- 0.
회복기.
회복하기.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(state_num(&output, "횟수"), fixed("2"));
    }

    #[test]
    fn call_tail_ambiguous_seed_stems_fail_without_guessing() {
        let source = r#"
계산:셈씨 = {
1 돌려줘.
}.

계산하:셈씨 = {
2 돌려줘.
}.

계산하기.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("ambiguous call tail must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_CALL_TAIL_AMBIGUOUS");
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
    fn numeric_kernel_big_exact_arithmetic_is_not_i128_limited() {
        let source = r#"
a <- ("170141183460469231731687303715884105728") 큰바른수.
b <- ("2") 큰바른수.
합 <- a + b.
곱 <- a * b.
"#;
        let output = run_source_once(source).expect("run");
        assert_eq!(
            output
                .state
                .get(&Key::new("합".to_string()))
                .expect("sum")
                .display(),
            "170141183460469231731687303715884105730"
        );
        assert_eq!(
            output
                .state
                .get(&Key::new("곱".to_string()))
                .expect("product")
                .display(),
            "340282366920938463463374607431768211456"
        );
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
    fn numeric_root_finding_v1_bisection_returns_root_residual_iteration_method() {
        let source = r#"
f <- (#ascii) 수식{x - 2}.
r <- (f, "x", 0, 4, 8) 수치해.이분법.
근 <- (r, 0) 차림.값.
잔차 <- (r, 1) 차림.값.
반복값 <- (r, 2) 차림.값.
방법 <- (r, 3) 차림.값.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_num(&out, "근"), fixed("2"));
        assert_eq!(state_num(&out, "잔차"), fixed("0"));
        assert_eq!(state_num(&out, "반복값"), fixed("1"));
        assert_eq!(state_str(&out, "방법"), "이분법");
    }

    #[test]
    fn numeric_root_finding_v1_rejects_unbracketed_root() {
        let source = r#"
f <- (#ascii) 수식{x^2 + 1}.
r <- (f, "x", -1, 1, 8) 수치해.이분법.
"#;
        let err = match run_source_once(source) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), "E_MATH_DOMAIN");
    }

    #[test]
    fn polynomial_solve_minimum_v1_reuses_exact_quadratic_relation_solver() {
        let source = r#"
f <- (#ascii) 수식{x^2 - 4*x + 4}.
결과 <- (f, "x") 다항식.풀기.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_display(&out, "결과"), "#성공(미지수=\"x\", 값=2)");
    }

    #[test]
    fn polynomial_solve_minimum_v1_preserves_non_unique_boundary() {
        let source = r#"
f <- (#ascii) 수식{x^2 - 5*x + 6}.
결과 <- (f, "x") 다항식.풀기.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_display(&out, "결과"), "#실패(사유=\"non_unique\")");
    }

    #[test]
    fn linear_inequality_solve_minimum_v1_returns_bounded_interval() {
        let source = r#"
f1 <- (#ascii) 수식{2*x + 1}.
f2 <- (#ascii) 수식{x - 1}.
조건들 <- [(식=f1, 비교="이하", 경계=9), (식=f2, 비교="이상", 경계=2)].
해 <- (조건들, "x") 선형부등식.풀기.
상태 <- 해.상태.
하한 <- 해.하한.
상한 <- 해.상한.
하한포함 <- 해.하한포함.
상한포함 <- 해.상한포함.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_str(&out, "상태"), "구간");
        assert_eq!(state_num(&out, "하한"), fixed("3"));
        assert_eq!(state_num(&out, "상한"), fixed("4"));
        assert!(state_bool(&out, "하한포함"));
        assert!(state_bool(&out, "상한포함"));
    }

    #[test]
    fn linear_inequality_solve_minimum_v1_detects_empty_interval() {
        let source = r#"
f1 <- (#ascii) 수식{x}.
조건들 <- [(식=f1, 비교="미만", 경계=1), (식=f1, 비교="초과", 경계=1)].
해 <- (조건들, "x") 선형부등식.풀기.
상태 <- 해.상태.
"#;
        let out = run_source_once(source).expect("run");
        assert_eq!(state_str(&out, "상태"), "공집합");
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
