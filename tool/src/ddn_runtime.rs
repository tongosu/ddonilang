use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::cell::Cell;

use crate::gate0_registry;
use crate::preprocess::{
    preprocess_source_for_parse, split_file_meta, validate_no_legacy_boim_surface,
    validate_no_legacy_header, FileMeta,
};
use ddonirang_core::platform::{
    EntityId, NuriWorld, Origin, Patch, PatchOp, ResourceMapEntry, ResourceValue,
};
use ddonirang_core::signals::DiagEvent;
use ddonirang_core::{
    unit_spec_from_symbol, ArithmeticFaultKind, ExprTrace, FaultContext, Fixed64, InputSnapshot,
    ResourceHandle, Signal, SourceSpan, UnitDim, UnitError, UnitValue, KEY_A, KEY_D, KEY_S, KEY_W,
};
use ddonirang_lang::runtime::{
    input_just_pressed, input_pressed, list_add, list_len, list_nth, list_remove, list_set,
    map_get, map_key_canon, string_concat, string_contains, string_ends, string_join, string_len,
    string_split, string_starts, string_to_number, InputState, LambdaValue, MapEntry, RuntimeError,
    Value,
};
use ddonirang_lang::{
    age_not_available_error, canonicalize, parse_with_mode, AgeTarget, Assertion, AtSuffix, Body,
    CanonProgram, Expr, ExprKind, Formula, FormulaDialect, Literal, ParamPin, ParseError,
    ParseMode, RegexLiteral, SeedDef, SeedKind, StateMachine, StateTransition, Stmt,
    TemplateFormat, TemplatePart, TopLevelItem, TypeRef,
};
use libm;
use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};
use regex::RegexBuilder;

static LAMBDA_SEQ: AtomicU64 = AtomicU64::new(1);
static DEFAULT_PARSE_MODE: AtomicU64 = AtomicU64::new(1);
static DEFAULT_AGE_TARGET: AtomicU64 = AtomicU64::new(3);
const STATE_MACHINE_RESOURCE_KIND: &str = "ddn.state_machine.v1";
const ASSERTION_RESOURCE_KIND: &str = "ddn.assertion.v1";
const INPUT_KEY_SPACE: u64 = 1 << 4;
const INPUT_KEY_ENTER: u64 = 1 << 5;
const INPUT_KEY_ESCAPE: u64 = 1 << 6;
const INPUT_KEY_Z: u64 = 1 << 7;
const INPUT_KEY_X: u64 = 1 << 8;
const NUMERIC_PACK_KIND_KEY: &str = "__수타입";
const NUMERIC_PACK_APPROX_KEY: &str = "__근사";
const NUMERIC_KIND_BIG_INT: &str = "큰바른수";
const NUMERIC_KIND_RATIONAL: &str = "나눔수";
const NUMERIC_KIND_FACTOR: &str = "곱수";
const NUMERIC_DIAG_RULE_ID_FACTOR_DECOMP_DEFERRED: &str = "L1-NUMERIC-01";
const NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED: &str = "NUMERIC_FACTOR_DECOMP_DEFERRED";
const NUMERIC_DIAG_TAG_FACTOR_DECOMP_DEFERRED: &str = "numeric:factor:deferred";
const NUMERIC_DIAG_RULE_ID_FACTOR_ROUTE_SUMMARY: &str = "L1-NUMERIC-02";
const NUMERIC_DIAG_REASON_FACTOR_ROUTE_SUMMARY: &str = "NUMERIC_FACTOR_ROUTE_SUMMARY";
const NUMERIC_DIAG_TAG_FACTOR_ROUTE_SUMMARY: &str = "numeric:factor:routes";
const NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY: &str = "수진단.곱수경로집계";
const NUMERIC_FACTOR_ROUTE_TOTAL_RESOURCE_KEY: &str = "수진단.곱수호출수";
const NUMERIC_FACTOR_BITS_MIN_RESOURCE_KEY: &str = "수진단.곱수비트최소";
const NUMERIC_FACTOR_BITS_MAX_RESOURCE_KEY: &str = "수진단.곱수비트최대";
const NUMERIC_FACTOR_BITS_SUM_RESOURCE_KEY: &str = "수진단.곱수비트합";
const NUMERIC_FACTOR_POLICY_RESOURCE_KEY: &str = "수진단.곱수정책";
const FACTOR_DECOMP_STATUS_KEY: &str = "분해상태";
const FACTOR_DECOMP_ROUTE_KEY: &str = "분해경로";
const FACTOR_DECOMP_BITS_KEY: &str = "분해비트수";
const FACTOR_DECOMP_STATUS_DONE: &str = "완료";
const FACTOR_DECOMP_STATUS_DEFERRED: &str = "지연";
const FACTOR_DECOMP_DEFERRED_REASON_KEY: &str = "지연사유";
const FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT: &str = "비트한계초과";
const FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED: &str = "분해실패";
const FACTOR_DECOMP_ROUTE_ZERO: &str = "zero";
const FACTOR_DECOMP_ROUTE_I64: &str = "i64";
const FACTOR_DECOMP_ROUTE_BIGINT: &str = "bigint";
const FACTOR_DECOMP_ROUTE_BIGINT_SMALL_PRIME: &str = "bigint:smallprime";
const FACTOR_DECOMP_ROUTE_BIGINT_POLLARD: &str = "bigint:pollard";
const FACTOR_DECOMP_ROUTE_BIGINT_FALLBACK: &str = "bigint:fallback";
const FACTOR_DECOMP_ROUTE_BIGINT_MIXED: &str = "bigint:mixed";
const FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT: &str = "deferred:bitlimit";
const FACTOR_DECOMP_ROUTE_DEFERRED_FACTOR_FAILED: &str = "deferred:factorfailed";
const FACTOR_BIGINT_FACTOR_BITS_LIMIT: usize = 512;
const FACTOR_POLLARD_MAX_ITERS: usize = 200_000;
const FACTOR_POLLARD_C_SEED_LIMIT: u64 = 64;
const FACTOR_POLLARD_X0_SEEDS: [u64; 6] = [2, 3, 5, 7, 11, 13];
const FACTOR_TRIAL_FALLBACK_LIMIT: u64 = 1_000_000;
const FACTOR_SMALL_PRIMES: [u64; 25] = [
    3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89,
    97, 101,
];
#[cfg(test)]
thread_local! {
    static FACTOR_POLLARD_MAX_ITERS_OVERRIDE: Cell<Option<usize>> = const { Cell::new(None) };
    static FACTOR_TRIAL_FALLBACK_LIMIT_OVERRIDE: Cell<Option<u64>> = const { Cell::new(None) };
}
const INPUT_KEY_BINDINGS: [(&str, u64, &[&str]); 9] = [
    ("ArrowLeft", KEY_A, &["왼쪽화살표", "왼쪽", "좌"]),
    ("ArrowRight", KEY_D, &["오른쪽화살표", "오른쪽", "우"]),
    (
        "ArrowDown",
        KEY_S,
        &["아래쪽화살표", "아래쪽", "아래", "하"],
    ),
    ("ArrowUp", KEY_W, &["위쪽화살표", "위쪽", "위", "상"]),
    (
        "Space",
        INPUT_KEY_SPACE,
        &["스페이스", "스페이스바", "공백"],
    ),
    ("Enter", INPUT_KEY_ENTER, &["엔터", "엔터키"]),
    ("Escape", INPUT_KEY_ESCAPE, &["이스케이프", "이스케이프키"]),
    ("KeyZ", INPUT_KEY_Z, &["Z키", "지키"]),
    ("KeyX", INPUT_KEY_X, &["X키", "엑스키"]),
];

fn value_from_flag_number(value: i64) -> Value {
    Value::Fixed64(Fixed64::from_i64(value))
}

fn seed_keyboard_flag_for_key(
    resources: &mut HashMap<String, Value>,
    key: &str,
    held_on: i64,
    pressed_on: i64,
    released_on: i64,
) {
    resources.insert(
        format!("샘.키보드.누르고있음.{}", key),
        value_from_flag_number(held_on),
    );
    resources.insert(
        format!("샘.키보드.눌림.{}", key),
        value_from_flag_number(pressed_on),
    );
    resources.insert(
        format!("샘.키보드.뗌.{}", key),
        value_from_flag_number(released_on),
    );
    resources.insert(
        format!("입력상태.키_누르고있음.{}", key),
        value_from_flag_number(held_on),
    );
    resources.insert(
        format!("입력상태.키_눌림.{}", key),
        value_from_flag_number(pressed_on),
    );
    resources.insert(
        format!("입력상태.키_뗌.{}", key),
        value_from_flag_number(released_on),
    );
}

fn seed_keyboard_state_resources(
    resources: &mut HashMap<String, Value>,
    prev_mask: u64,
    next_mask: u64,
) {
    for (key, bit, aliases) in INPUT_KEY_BINDINGS {
        let held_on = if next_mask & bit != 0 { 1 } else { 0 };
        let pressed_on = if (next_mask & bit != 0) && (prev_mask & bit == 0) {
            1
        } else {
            0
        };
        let released_on = if (next_mask & bit == 0) && (prev_mask & bit != 0) {
            1
        } else {
            0
        };
        seed_keyboard_flag_for_key(resources, key, held_on, pressed_on, released_on);
        for alias in aliases {
            seed_keyboard_flag_for_key(resources, alias, held_on, pressed_on, released_on);
        }
    }
}

#[derive(Clone, Debug)]
pub struct DdnParseWarning {
    pub code: String,
    pub message: String,
    pub span_start: usize,
    pub span_end: usize,
}

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

fn encode_age_target(age: AgeTarget) -> u64 {
    match age {
        AgeTarget::Age0 => 0,
        AgeTarget::Age1 => 1,
        AgeTarget::Age2 => 2,
        AgeTarget::Age3 => 3,
        AgeTarget::Age4 => 4,
        AgeTarget::Age5 => 5,
        AgeTarget::Age6 => 6,
        AgeTarget::Age7 => 7,
    }
}

fn decode_age_target(value: u64) -> AgeTarget {
    match value {
        0 => AgeTarget::Age0,
        1 => AgeTarget::Age1,
        2 => AgeTarget::Age2,
        3 => AgeTarget::Age3,
        4 => AgeTarget::Age4,
        5 => AgeTarget::Age5,
        6 => AgeTarget::Age6,
        7 => AgeTarget::Age7,
        _ => AgeTarget::Age3,
    }
}

pub fn default_parse_mode() -> ParseMode {
    decode_parse_mode(DEFAULT_PARSE_MODE.load(Ordering::Relaxed))
}

pub fn set_default_parse_mode(mode: ParseMode) {
    DEFAULT_PARSE_MODE.store(encode_parse_mode(mode), Ordering::Relaxed);
}

pub fn default_age_target() -> AgeTarget {
    decode_age_target(DEFAULT_AGE_TARGET.load(Ordering::Relaxed))
}

pub fn set_default_age_target(age: AgeTarget) {
    DEFAULT_AGE_TARGET.store(encode_age_target(age), Ordering::Relaxed);
}

#[derive(Clone)]
pub struct DdnProgram {
    #[allow(dead_code)]
    program: CanonProgram,
    functions: HashMap<String, SeedDef>,
    #[allow(dead_code)]
    file_meta: FileMeta,
    parse_warnings: Vec<DdnParseWarning>,
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
        validate_no_legacy_header(source)?;
        validate_no_legacy_boim_surface(source)?;
        let meta_parse = split_file_meta(source);
        let cleaned = preprocess_source_for_parse(&meta_parse.stripped)?;
        let prepared = ddonirang_lang::preprocess_frontdoor_source(&cleaned);
        let mut program = parse_with_mode(&prepared, file_path, mode)
            .map_err(|e| format_parse_error(&prepared, &e))?;
        let report = canonicalize(&mut program).map_err(|e| format_parse_error(&prepared, &e))?;
        let parse_warnings = report
            .warnings
            .into_iter()
            .map(|warning| DdnParseWarning {
                code: warning.code.to_string(),
                message: warning.message,
                span_start: warning.span.start,
                span_end: warning.span.end,
            })
            .collect();
        enforce_regex_age_gate(&program, default_age_target())?;
        enforce_assertion_age_gate(&program, default_age_target())?;
        enforce_state_machine_age_gate(&program, default_age_target())?;
        enforce_quantifier_age_gate(&program, default_age_target())?;
        let mut functions = HashMap::new();
        let tails = ["기", "고", "면", "면서"];
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
            parse_warnings,
        })
    }

    pub fn parse_warnings(&self) -> &[DdnParseWarning] {
        &self.parse_warnings
    }
}

fn enforce_regex_age_gate(program: &CanonProgram, age_target: AgeTarget) -> Result<(), String> {
    if age_target >= AgeTarget::Age3 {
        return Ok(());
    }
    if let Some(feature) = detect_regex_feature(program) {
        return Err(age_not_available_error(
            feature,
            AgeTarget::Age3,
            age_target,
        ));
    }
    Ok(())
}

fn enforce_assertion_age_gate(program: &CanonProgram, age_target: AgeTarget) -> Result<(), String> {
    if age_target >= AgeTarget::Age1 {
        return Ok(());
    }
    if let Some(feature) = detect_assertion_feature(program) {
        return Err(age_not_available_error(
            feature,
            AgeTarget::Age1,
            age_target,
        ));
    }
    Ok(())
}

fn enforce_state_machine_age_gate(
    program: &CanonProgram,
    age_target: AgeTarget,
) -> Result<(), String> {
    if age_target >= AgeTarget::Age1 {
        return Ok(());
    }
    if let Some(feature) = detect_state_machine_feature(program) {
        return Err(age_not_available_error(
            feature,
            AgeTarget::Age1,
            age_target,
        ));
    }
    Ok(())
}

fn enforce_quantifier_age_gate(
    program: &CanonProgram,
    age_target: AgeTarget,
) -> Result<(), String> {
    if age_target >= AgeTarget::Age4 {
        return Ok(());
    }
    if let Some(feature) = detect_quantifier_feature(program) {
        return Err(age_not_available_error(
            feature,
            AgeTarget::Age4,
            age_target,
        ));
    }
    Ok(())
}

fn detect_regex_feature(program: &CanonProgram) -> Option<&'static str> {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            if let Some(feature) = body_regex_feature(body) {
                return Some(feature);
            }
        }
    }
    None
}

fn detect_assertion_feature(program: &CanonProgram) -> Option<&'static str> {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            if let Some(feature) = body_assertion_feature(body) {
                return Some(feature);
            }
        }
    }
    None
}

fn detect_state_machine_feature(program: &CanonProgram) -> Option<&'static str> {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            if let Some(feature) = body_state_machine_feature(body) {
                return Some(feature);
            }
        }
    }
    None
}

fn detect_quantifier_feature(program: &CanonProgram) -> Option<&'static str> {
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if let Some(body) = &seed.body {
            if let Some(feature) = body_quantifier_feature(body) {
                return Some(feature);
            }
        }
    }
    None
}

fn body_regex_feature(body: &Body) -> Option<&'static str> {
    for stmt in &body.stmts {
        if let Some(feature) = stmt_regex_feature(stmt) {
            return Some(feature);
        }
    }
    None
}

fn body_assertion_feature(body: &Body) -> Option<&'static str> {
    for stmt in &body.stmts {
        if let Some(feature) = stmt_assertion_feature(stmt) {
            return Some(feature);
        }
    }
    None
}

fn body_state_machine_feature(body: &Body) -> Option<&'static str> {
    for stmt in &body.stmts {
        if let Some(feature) = stmt_state_machine_feature(stmt) {
            return Some(feature);
        }
    }
    None
}

fn body_quantifier_feature(body: &Body) -> Option<&'static str> {
    for stmt in &body.stmts {
        if let Some(feature) = stmt_quantifier_feature(stmt) {
            return Some(feature);
        }
    }
    None
}

fn stmt_regex_feature(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    if let Some(feature) = expr_regex_feature(value) {
                        return Some(feature);
                    }
                }
            }
            None
        }
        Stmt::Mutate { target, value, .. } => {
            expr_regex_feature(target).or_else(|| expr_regex_feature(value))
        }
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            expr_regex_feature(expr)
        }
        Stmt::Return { value, .. } => expr_regex_feature(value),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => expr_regex_feature(condition)
            .or_else(|| body_regex_feature(then_body))
            .or_else(|| else_body.as_ref().and_then(body_regex_feature)),
        Stmt::Try { action, body, .. } => {
            expr_regex_feature(action).or_else(|| body_regex_feature(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if let Some(feature) = expr_regex_feature(&branch.condition) {
                    return Some(feature);
                }
                if let Some(feature) = body_regex_feature(&branch.body) {
                    return Some(feature);
                }
            }
            body_regex_feature(else_body)
        }
        Stmt::Repeat { body, .. } => body_regex_feature(body),
        Stmt::While {
            condition, body, ..
        } => expr_regex_feature(condition).or_else(|| body_regex_feature(body)),
        Stmt::ForEach { iterable, body, .. } => {
            expr_regex_feature(iterable).or_else(|| body_regex_feature(body))
        }
        Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => body_regex_feature(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_regex_feature(condition).or_else(|| body_regex_feature(body)),
        Stmt::Quantifier { body, .. } => body_regex_feature(body),
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => expr_regex_feature(condition)
            .or_else(|| then_body.as_ref().and_then(body_regex_feature))
            .or_else(|| body_regex_feature(else_body)),
        Stmt::Guard {
            condition, body, ..
        } => expr_regex_feature(condition).or_else(|| body_regex_feature(body)),
        Stmt::MetaBlock { .. } | Stmt::Pragma { .. } | Stmt::Break { .. } => None,
    }
}

fn stmt_assertion_feature(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    if let Some(feature) = expr_assertion_feature(value) {
                        return Some(feature);
                    }
                }
            }
            None
        }
        Stmt::Mutate { target, value, .. } => {
            expr_assertion_feature(target).or_else(|| expr_assertion_feature(value))
        }
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            expr_assertion_feature(expr)
        }
        Stmt::Return { value, .. } => expr_assertion_feature(value),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => expr_assertion_feature(condition)
            .or_else(|| body_assertion_feature(then_body))
            .or_else(|| else_body.as_ref().and_then(body_assertion_feature)),
        Stmt::Try { action, body, .. } => {
            expr_assertion_feature(action).or_else(|| body_assertion_feature(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if let Some(feature) = expr_assertion_feature(&branch.condition) {
                    return Some(feature);
                }
                if let Some(feature) = body_assertion_feature(&branch.body) {
                    return Some(feature);
                }
            }
            body_assertion_feature(else_body)
        }
        Stmt::Repeat { body, .. } => body_assertion_feature(body),
        Stmt::While {
            condition, body, ..
        } => expr_assertion_feature(condition).or_else(|| body_assertion_feature(body)),
        Stmt::ForEach { iterable, body, .. } => {
            expr_assertion_feature(iterable).or_else(|| body_assertion_feature(body))
        }
        Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => body_assertion_feature(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_assertion_feature(condition).or_else(|| body_assertion_feature(body)),
        Stmt::Quantifier { body, .. } => body_assertion_feature(body),
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => expr_assertion_feature(condition)
            .or_else(|| then_body.as_ref().and_then(body_assertion_feature))
            .or_else(|| body_assertion_feature(else_body)),
        Stmt::Guard {
            condition, body, ..
        } => expr_assertion_feature(condition).or_else(|| body_assertion_feature(body)),
        Stmt::MetaBlock { .. } | Stmt::Pragma { .. } | Stmt::Break { .. } => None,
    }
}

fn stmt_state_machine_feature(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    if let Some(feature) = expr_state_machine_feature(value) {
                        return Some(feature);
                    }
                }
            }
            None
        }
        Stmt::Mutate { target, value, .. } => {
            expr_state_machine_feature(target).or_else(|| expr_state_machine_feature(value))
        }
        Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
            expr_state_machine_feature(expr)
        }
        Stmt::Return { value, .. } => expr_state_machine_feature(value),
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => expr_state_machine_feature(condition)
            .or_else(|| body_state_machine_feature(then_body))
            .or_else(|| else_body.as_ref().and_then(body_state_machine_feature)),
        Stmt::Try { action, body, .. } => {
            expr_state_machine_feature(action).or_else(|| body_state_machine_feature(body))
        }
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if let Some(feature) = expr_state_machine_feature(&branch.condition) {
                    return Some(feature);
                }
                if let Some(feature) = body_state_machine_feature(&branch.body) {
                    return Some(feature);
                }
            }
            body_state_machine_feature(else_body)
        }
        Stmt::Repeat { body, .. } => body_state_machine_feature(body),
        Stmt::While {
            condition, body, ..
        } => expr_state_machine_feature(condition).or_else(|| body_state_machine_feature(body)),
        Stmt::ForEach { iterable, body, .. } => {
            expr_state_machine_feature(iterable).or_else(|| body_state_machine_feature(body))
        }
        Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => body_state_machine_feature(body),
        Stmt::HookWhenBecomes {
            condition, body, ..
        }
        | Stmt::HookWhile {
            condition, body, ..
        } => expr_state_machine_feature(condition).or_else(|| body_state_machine_feature(body)),
        Stmt::Quantifier { body, .. } => body_state_machine_feature(body),
        Stmt::Contract {
            condition,
            then_body,
            else_body,
            ..
        } => expr_state_machine_feature(condition)
            .or_else(|| then_body.as_ref().and_then(body_state_machine_feature))
            .or_else(|| body_state_machine_feature(else_body)),
        Stmt::Guard {
            condition, body, ..
        } => expr_state_machine_feature(condition).or_else(|| body_state_machine_feature(body)),
        Stmt::MetaBlock { .. } | Stmt::Pragma { .. } | Stmt::Break { .. } => None,
    }
}

fn stmt_quantifier_feature(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::DeclBlock { items, .. } => {
            for item in items {
                if let Some(value) = &item.value {
                    let _ = value;
                }
            }
            None
        }
        Stmt::Mutate { .. }
        | Stmt::Expr { .. }
        | Stmt::Show { .. }
        | Stmt::Inspect { .. }
        | Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Return { .. }
        | Stmt::Break { .. } => None,
        Stmt::If {
            then_body,
            else_body,
            ..
        } => body_quantifier_feature(then_body)
            .or_else(|| else_body.as_ref().and_then(body_quantifier_feature)),
        Stmt::Try { body, .. } => body_quantifier_feature(body),
        Stmt::Choose {
            branches,
            else_body,
            ..
        } => {
            for branch in branches {
                if let Some(feature) = body_quantifier_feature(&branch.body) {
                    return Some(feature);
                }
            }
            body_quantifier_feature(else_body)
        }
        Stmt::Repeat { body, .. } => body_quantifier_feature(body),
        Stmt::While { body, .. } => body_quantifier_feature(body),
        Stmt::ForEach { body, .. } => body_quantifier_feature(body),
        Stmt::BeatBlock { body, .. } | Stmt::Hook { body, .. } => body_quantifier_feature(body),
        Stmt::HookWhenBecomes { body, .. } | Stmt::HookWhile { body, .. } => {
            body_quantifier_feature(body)
        }
        Stmt::Quantifier { .. } => Some("logic_quantifier"),
        Stmt::Contract {
            then_body,
            else_body,
            ..
        } => then_body
            .as_ref()
            .and_then(body_quantifier_feature)
            .or_else(|| body_quantifier_feature(else_body)),
        Stmt::Guard { body, .. } => body_quantifier_feature(body),
    }
}

fn expr_regex_feature(expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::Literal(Literal::Regex(_)) => Some("regex_literal"),
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_)
        | ExprKind::StateMachine(_) => None,
        ExprKind::FieldAccess { target, .. } => expr_regex_feature(target),
        ExprKind::SeedLiteral { body, .. } => expr_regex_feature(body),
        ExprKind::Call { args, func } => {
            if matches!(
                func.as_str(),
                "정규맞추기"
                    | "정규찾기"
                    | "정규캡처하기"
                    | "정규이름캡처하기"
                    | "정규바꾸기"
                    | "정규나누기"
            ) {
                return Some("regex_api");
            }
            for arg in args {
                if let Some(feature) = expr_regex_feature(&arg.expr) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Infix { left, right, .. } => {
            expr_regex_feature(left).or_else(|| expr_regex_feature(right))
        }
        ExprKind::Suffix { value, .. } => expr_regex_feature(value),
        ExprKind::Thunk(body) => body_regex_feature(body),
        ExprKind::Eval { thunk, .. } => expr_regex_feature(thunk),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                if let Some(feature) = expr_regex_feature(stage) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                if let Some(feature) = expr_regex_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                if let Some(feature) = expr_regex_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Nuance { expr, .. } => expr_regex_feature(expr),
    }
}

fn expr_assertion_feature(expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::Assertion(_) => Some("assertion_literal"),
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Formula(_)
        | ExprKind::Template(_)
        | ExprKind::StateMachine(_) => None,
        ExprKind::FieldAccess { target, .. } => expr_assertion_feature(target),
        ExprKind::SeedLiteral { body, .. } => expr_assertion_feature(body),
        ExprKind::Call { args, func } => {
            if func == "살피기" {
                return Some("assertion_api");
            }
            for arg in args {
                if let Some(feature) = expr_assertion_feature(&arg.expr) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Infix { left, right, .. } => {
            expr_assertion_feature(left).or_else(|| expr_assertion_feature(right))
        }
        ExprKind::Suffix { value, .. } => expr_assertion_feature(value),
        ExprKind::Thunk(body) => body_assertion_feature(body),
        ExprKind::Eval { thunk, .. } => expr_assertion_feature(thunk),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                if let Some(feature) = expr_assertion_feature(stage) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                if let Some(feature) = expr_assertion_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                if let Some(feature) = expr_assertion_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Nuance { expr, .. } => expr_assertion_feature(expr),
    }
}

fn expr_state_machine_feature(expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::StateMachine(_) => Some("state_machine_literal"),
        ExprKind::Literal(_)
        | ExprKind::Var(_)
        | ExprKind::FlowValue
        | ExprKind::Assertion(_)
        | ExprKind::Formula(_)
        | ExprKind::Template(_) => None,
        ExprKind::FieldAccess { target, .. } => expr_state_machine_feature(target),
        ExprKind::SeedLiteral { body, .. } => expr_state_machine_feature(body),
        ExprKind::Call { args, func } => {
            if matches!(func.as_str(), "처음으로" | "지금상태" | "다음으로") {
                return Some("state_machine_api");
            }
            for arg in args {
                if let Some(feature) = expr_state_machine_feature(&arg.expr) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Infix { left, right, .. } => {
            expr_state_machine_feature(left).or_else(|| expr_state_machine_feature(right))
        }
        ExprKind::Suffix { value, .. } => expr_state_machine_feature(value),
        ExprKind::Thunk(body) => body_state_machine_feature(body),
        ExprKind::Eval { thunk, .. } => expr_state_machine_feature(thunk),
        ExprKind::Pipe { stages } => {
            for stage in stages {
                if let Some(feature) = expr_state_machine_feature(stage) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Pack { fields } => {
            for (_, value) in fields {
                if let Some(feature) = expr_state_machine_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::TemplateRender { inject, .. } | ExprKind::FormulaEval { inject, .. } => {
            for (_, value) in inject {
                if let Some(feature) = expr_state_machine_feature(value) {
                    return Some(feature);
                }
            }
            None
        }
        ExprKind::Nuance { expr, .. } => expr_state_machine_feature(expr),
    }
}

pub struct DdnRunner {
    program: DdnProgram,
    update_name: String,
    prev_keys_pressed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransitionRecord {
    pub madi: u64,
    pub origin: String,
    pub from: String,
    pub to: String,
    pub message: String,
    pub guard_name: Option<String>,
    pub action_name: Option<String>,
}

pub struct DdnRunOutput {
    pub patch: Patch,
    pub resources: HashMap<String, Value>,
    #[allow(dead_code)]
    pub state_transitions: Vec<StateTransitionRecord>,
}

fn parse_state_transition_tag(tag: &str) -> Option<(String, String)> {
    let body = tag.strip_prefix("state_transition:")?;
    let (from, to) = body.split_once("->")?;
    Some((from.to_string(), to.to_string()))
}

fn parse_state_transition_text(text: Option<&str>) -> (Option<String>, Option<String>) {
    let mut guard_name = None;
    let mut action_name = None;
    let Some(text) = text else {
        return (None, None);
    };
    for part in text.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("guard=") {
            let value = value.trim();
            if !value.is_empty() && value != "-" {
                guard_name = Some(value.to_string());
            }
        } else if let Some(value) = part.strip_prefix("action=") {
            let value = value.trim();
            if !value.is_empty() && value != "-" {
                action_name = Some(value.to_string());
            }
        }
    }
    (guard_name, action_name)
}

fn collect_state_transition_records(ops: &[PatchOp]) -> Vec<StateTransitionRecord> {
    ops.iter()
        .filter_map(|op| match op {
            PatchOp::EmitSignal {
                signal: Signal::Diag { event },
                ..
            } if event.rule_id == "L1-STATE-01" && event.reason == "STATE_TRANSITION" => {
                let expr = event.expr.as_ref()?;
                let (from, to) = parse_state_transition_tag(&expr.tag)?;
                let (guard_name, action_name) = parse_state_transition_text(expr.text.as_deref());
                Some(StateTransitionRecord {
                    madi: event.madi,
                    origin: event.origin.clone(),
                    from,
                    to,
                    message: event.message.clone().unwrap_or_default(),
                    guard_name,
                    action_name,
                })
            }
            _ => None,
        })
        .collect()
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
        let prev_keys_pressed = self.prev_keys_pressed;
        let input_state = InputState::new(input.keys_pressed, prev_keys_pressed);
        self.prev_keys_pressed = input.keys_pressed;

        let mut ctx = EvalContext::new(
            &self.program,
            world,
            defaults,
            input_state,
            input.rng_seed,
            input.tick_id,
        );
        ctx.resources.insert(
            "입력키".to_string(),
            Value::String(input.last_key_name.clone()),
        );
        seed_keyboard_state_resources(&mut ctx.resources, prev_keys_pressed, input.keys_pressed);
        let update = if let Some(update) = ctx.program.functions.get(&self.update_name) {
            update
        } else if self.update_name == "매마디" {
            ctx.program
                .functions
                .get("매틱")
                .ok_or_else(|| format!("업데이트 함수 '{}'를 찾을 수 없습니다", self.update_name))?
        } else {
            return Err(format!(
                "업데이트 함수 '{}'를 찾을 수 없습니다",
                self.update_name
            ));
        };
        ctx.eval_seed(update, Vec::new())
            .map_err(|err| err.to_string())?;
        ctx.flush_factor_route_metrics_resource();
        ctx.emit_factor_route_summary_diag();
        let patch = Patch {
            ops: ctx.patch_ops,
            origin: Origin::system("ddn"),
        };
        let state_transitions = collect_state_transition_records(&patch.ops);
        Ok(DdnRunOutput {
            patch,
            resources: ctx.resources,
            state_transitions,
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
    factor_route_counts: BTreeMap<String, u64>,
    factor_bits_total: u128,
    factor_bits_min: Option<u64>,
    factor_bits_max: u64,
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

#[derive(Clone)]
struct EvalFrameSnapshot {
    locals: HashMap<String, Value>,
    resources: HashMap<String, Value>,
    patch_ops: Vec<PatchOp>,
    guard_rejected: bool,
    const_scopes: Vec<HashSet<String>>,
    factor_route_counts: BTreeMap<String, u64>,
    factor_bits_total: u128,
    factor_bits_min: Option<u64>,
    factor_bits_max: u64,
}

#[derive(Debug)]
enum EvalError {
    Message(String),
    UnitMismatch { left: UnitDim, right: UnitDim },
    DivisionByZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BogaeShapeKind {
    Line,
    Circle,
    Point,
    Rect,
    Text,
    Polyline,
    Polygon,
}

#[derive(Debug, Clone)]
struct BogaeShapeCall {
    kind: BogaeShapeKind,
    positional: Vec<String>,
    named: BTreeMap<String, String>,
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
                format!(
                    "단위 차원이 다릅니다: {} vs {}",
                    left.format(),
                    right.format()
                )
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
            factor_route_counts: BTreeMap::new(),
            factor_bits_total: 0,
            factor_bits_min: None,
            factor_bits_max: 0,
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
                    return Err("멈추기는 반복 안에서만 사용할 수 있습니다"
                        .to_string()
                        .into())
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
                Err(type_mismatch_error(
                    &param.pin_name,
                    &expected,
                    &detail.actual,
                ))
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
            let snapshot = self.capture_frame_snapshot(locals);
            match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => {}
                other => return Ok(other),
            }
            if self.aborted {
                self.restore_frame_snapshot_preserving_abort_contract_diag(locals, &snapshot);
                return Ok(FlowControl::Continue);
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
            ThunkResult::Break(_) => Err("멈추기는 반복 안에서만 사용할 수 있습니다"
                .to_string()
                .into()),
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
            let snapshot = self.capture_frame_snapshot(locals);
            match self.eval_stmt_for_value(locals, stmt)? {
                ThunkResult::Return(v) => return Ok(ThunkResult::Return(v)),
                ThunkResult::Break(span) => return Ok(ThunkResult::Break(span)),
                ThunkResult::Value(v) => last = v,
            }
            if self.aborted {
                self.restore_frame_snapshot_preserving_abort_contract_diag(locals, &snapshot);
                return Ok(ThunkResult::Value(Value::None));
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
            Stmt::DeclBlock { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Mutate { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                Ok(ThunkResult::Value(self.eval_expr(locals, expr)?))
            }
            Stmt::MetaBlock { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Pragma { .. } => Ok(ThunkResult::Value(Value::None)),
            Stmt::Return { value, .. } => Ok(ThunkResult::Return(self.eval_expr(locals, value)?)),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
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
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
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
            Stmt::While {
                condition, body, ..
            } => {
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
            Stmt::ForEach {
                item,
                iterable,
                body,
                ..
            } => {
                let iter_value = self.eval_expr(locals, iterable)?;
                let items = match iter_value {
                    Value::List(items) => items,
                    Value::Set(items) => items.into_values().collect(),
                    Value::Map(entries) => entries
                        .into_values()
                        .map(|entry| Value::List(vec![entry.key, entry.value]))
                        .collect(),
                    _ => {
                        return Err("순회 대상은 차림/모음/짝맞춤이어야 합니다"
                            .to_string()
                            .into())
                    }
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
            Stmt::BeatBlock { .. }
            | Stmt::Hook { .. }
            | Stmt::HookWhenBecomes { .. }
            | Stmt::HookWhile { .. } => Err(
                "E_RUNTIME_UNSUPPORTED_STMT: 현재 ddonirang-tool 실행기에서는 박자/훅 문장을 지원하지 않습니다"
                    .to_string()
                    .into(),
            ),
            Stmt::Quantifier { .. } => Ok(ThunkResult::Value(Value::None)),
            Stmt::Break { span, .. } => Ok(ThunkResult::Break(*span)),
            Stmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
                ..
            } => {
                let snapshot = self.capture_frame_snapshot(locals);
                let cond_value = self.eval_expr(locals, condition)?;
                let ok = is_truthy(&cond_value)?;
                match kind {
                    ddonirang_lang::ContractKind::Pre => {
                        if ok {
                            if let Some(body) = then_body {
                                let out = self.eval_body_for_value_inner(locals, body)?;
                                if self.aborted {
                                    self.restore_frame_snapshot_preserving_abort_contract_diag(
                                        locals, &snapshot,
                                    );
                                    return Ok(ThunkResult::Value(Value::None));
                                }
                                Ok(out)
                            } else {
                                Ok(ThunkResult::Value(Value::None))
                            }
                        } else {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                FlowControl::Return(value) => {
                                    return Ok(ThunkResult::Return(value))
                                }
                                FlowControl::Break(span) => return Ok(ThunkResult::Break(span)),
                            }
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(ThunkResult::Value(Value::None));
                            }
                            if matches!(mode, ddonirang_lang::ContractMode::Abort) {
                                self.restore_frame_snapshot(locals, &snapshot);
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
                                FlowControl::Return(value) => {
                                    return Ok(ThunkResult::Return(value))
                                }
                                FlowControl::Break(span) => return Ok(ThunkResult::Break(span)),
                            }
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(ThunkResult::Value(Value::None));
                            }
                            let cond_value = self.eval_expr(locals, condition)?;
                            if !is_truthy(&cond_value)? {
                                if matches!(mode, ddonirang_lang::ContractMode::Abort) {
                                    self.restore_frame_snapshot(locals, &snapshot);
                                }
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
                            let out = self.eval_body_for_value_inner(locals, body)?;
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(ThunkResult::Value(Value::None));
                            }
                            Ok(out)
                        } else {
                            Ok(ThunkResult::Value(Value::None))
                        }
                    }
                }
            }
            Stmt::Guard { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
        }
    }

    fn eval_stmt(
        &mut self,
        locals: &mut HashMap<String, Value>,
        stmt: &Stmt,
    ) -> Result<FlowControl, EvalError> {
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
                        return Err(format!(
                            "채비에서 '=' 항목은 초기값이 필요합니다: {}",
                            item.name
                        )
                        .into());
                    } else {
                        Value::None
                    };
                    if item.value.is_some() {
                        if let Err(detail) = check_value_type(&value, &item.type_ref) {
                            return Err(type_mismatch_error(&item.name, &detail.expected, &detail.actual));
                        }
                    }
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
                        if locals.contains_key(name) {
                            locals.insert(name.clone(), val);
                        } else {
                            match self.set_resource(name, val) {
                                Ok(()) => {}
                                Err(EvalError::UnitMismatch { left, right }) => {
                                    let trace =
                                        Some(self.arith_trace(value, "arith:UNIT_MISMATCH"));
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
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
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
            Stmt::MetaBlock { kind, entries, .. } => {
                if matches!(kind, ddonirang_lang::MetaBlockKind::Bogae) {
                    self.apply_bogae_meta_block(locals, entries)?;
                }
                Ok(FlowControl::Continue)
            }
            Stmt::Pragma { .. } => Ok(FlowControl::Continue),
            Stmt::Return { value, .. } => {
                let val = self.eval_expr(locals, value)?;
                Ok(FlowControl::Return(val))
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
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
            Stmt::Choose {
                branches,
                else_body,
                ..
            } => {
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
            Stmt::While {
                condition, body, ..
            } => {
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
            Stmt::ForEach {
                item,
                iterable,
                body,
                ..
            } => {
                let iter_value = self.eval_expr(locals, iterable)?;
                let items = match iter_value {
                    Value::List(items) => items,
                    Value::Set(items) => items.into_values().collect(),
                    Value::Map(entries) => entries
                        .into_values()
                        .map(|entry| Value::List(vec![entry.key, entry.value]))
                        .collect(),
                    _ => {
                        return Err("순회 대상은 차림/모음/짝맞춤이어야 합니다"
                            .to_string()
                            .into())
                    }
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
            Stmt::BeatBlock { .. }
            | Stmt::Hook { .. }
            | Stmt::HookWhenBecomes { .. }
            | Stmt::HookWhile { .. } => Err(
                "E_RUNTIME_UNSUPPORTED_STMT: 현재 ddonirang-tool 실행기에서는 박자/훅 문장을 지원하지 않습니다"
                    .to_string()
                    .into(),
            ),
            Stmt::Quantifier { .. } => Ok(FlowControl::Continue),
            Stmt::Break { span, .. } => Ok(FlowControl::Break(*span)),
            Stmt::Contract {
                kind,
                mode,
                condition,
                then_body,
                else_body,
                ..
            } => {
                let snapshot = self.capture_frame_snapshot(locals);
                let cond_value = self.eval_expr(locals, condition)?;
                let ok = is_truthy(&cond_value)?;
                match kind {
                    ddonirang_lang::ContractKind::Pre => {
                        if ok {
                            if let Some(body) = then_body {
                                let out = self.eval_body(locals, body)?;
                                if self.aborted {
                                    self.restore_frame_snapshot_preserving_abort_contract_diag(
                                        locals, &snapshot,
                                    );
                                    return Ok(FlowControl::Continue);
                                }
                                return Ok(out);
                            }
                            Ok(FlowControl::Continue)
                        } else {
                            match self.eval_body(locals, else_body)? {
                                FlowControl::Continue => {}
                                other => return Ok(other),
                            }
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(FlowControl::Continue);
                            }
                            if matches!(mode, ddonirang_lang::ContractMode::Abort) {
                                self.restore_frame_snapshot(locals, &snapshot);
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
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(FlowControl::Continue);
                            }
                            let cond_value = self.eval_expr(locals, condition)?;
                            if !is_truthy(&cond_value)? {
                                if matches!(mode, ddonirang_lang::ContractMode::Abort) {
                                    self.restore_frame_snapshot(locals, &snapshot);
                                }
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
                            let out = self.eval_body(locals, body)?;
                            if self.aborted {
                                self.restore_frame_snapshot_preserving_abort_contract_diag(
                                    locals, &snapshot,
                                );
                                return Ok(FlowControl::Continue);
                            }
                            return Ok(out);
                        }
                        Ok(FlowControl::Continue)
                    }
                }
            }
            Stmt::Guard {
                condition, body, ..
            } => {
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

    fn eval_expr(
        &mut self,
        locals: &mut HashMap<String, Value>,
        expr: &Expr,
    ) -> Result<Value, EvalError> {
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
                self.eval_call(locals, func, values, self.source_span_for_expr(expr))
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
                    return Err("평가 표지는 안은문장에만 붙일 수 있습니다"
                        .to_string()
                        .into());
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
            ExprKind::Assertion(assertion) => Ok(Value::Assertion(assertion.clone())),
            ExprKind::StateMachine(machine) => Ok(Value::StateMachine(machine.clone())),
            ExprKind::Formula(formula) => Ok(Value::Formula(formula.clone())),
            ExprKind::Template(template) => Ok(Value::Template(template.clone())),
            ExprKind::TemplateRender { template, inject } => {
                let pack = self.eval_inject_fields(locals, inject)?;
                self.eval_call(
                    locals,
                    "채우기",
                    vec![Value::Template(template.clone()), pack],
                    self.source_span_for_expr(expr),
                )
            }
            ExprKind::FormulaEval { formula, inject } => {
                let pack = self.eval_inject_fields(locals, inject)?;
                self.eval_call(
                    locals,
                    "풀기",
                    vec![Value::Formula(formula.clone()), pack],
                    self.source_span_for_expr(expr),
                )
            }
            ExprKind::Nuance { expr, .. } => self.eval_expr(locals, expr),
        }
    }

    fn apply_bogae_meta_block(
        &mut self,
        locals: &mut HashMap<String, Value>,
        entries: &[String],
    ) -> Result<(), EvalError> {
        let mut draw_items = Vec::new();
        for entry in entries {
            if let Some(draw_item) = self.bogae_entry_to_draw_item(locals, entry) {
                draw_items.push(draw_item);
            }
        }
        if draw_items.is_empty() {
            return Ok(());
        }
        self.set_resource("보개_그림판_목록", Value::List(draw_items))
    }

    fn bogae_entry_to_draw_item(
        &mut self,
        locals: &mut HashMap<String, Value>,
        entry: &str,
    ) -> Option<Value> {
        let shape = parse_bogae_shape_call(entry)?;
        let mut map = BTreeMap::new();
        let mut put = |key: &str, value: Value| {
            let key_value = Value::String(key.to_string());
            map.insert(
                map_key_canon(&key_value),
                MapEntry {
                    key: key_value,
                    value,
                },
            );
        };

        match shape.kind {
            BogaeShapeKind::Line => {
                put("kind", Value::String("line".to_string()));
                put(
                    "x1",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x1"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y1",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y1"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "x2",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x2"],
                        2,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y2",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y2"],
                        3,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "선색", "stroke", "color"],
                        usize::MAX,
                        Value::String("#9ca3af".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.02)),
                    ),
                );
            }
            BogaeShapeKind::Circle => {
                put("kind", Value::String("circle".to_string()));
                put(
                    "x",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x", "cx"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y", "cy"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "r",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["r", "반지름"],
                        2,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.08)),
                    ),
                );
                put(
                    "fill",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "fill"],
                        usize::MAX,
                        Value::String("#38bdf8".to_string()),
                    ),
                );
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["선색", "stroke", "color"],
                        usize::MAX,
                        Value::String("#0ea5e9".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.02)),
                    ),
                );
            }
            BogaeShapeKind::Point => {
                put("kind", Value::String("point".to_string()));
                put(
                    "x",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x", "cx"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y", "cy"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "size",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["크기", "size", "r"],
                        2,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.045)),
                    ),
                );
                put(
                    "color",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "color"],
                        usize::MAX,
                        Value::String("#f59e0b".to_string()),
                    ),
                );
            }
            BogaeShapeKind::Rect => {
                put("kind", Value::String("rect".to_string()));
                put(
                    "x",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "w",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["w", "너비"],
                        2,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.2)),
                    ),
                );
                put(
                    "h",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["h", "높이"],
                        3,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.12)),
                    ),
                );
                put(
                    "fill",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["채움색", "채움", "fill", "색", "color"],
                        usize::MAX,
                        Value::String("#38bdf8".to_string()),
                    ),
                );
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["선색", "stroke"],
                        usize::MAX,
                        Value::String("#0ea5e9".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.02)),
                    ),
                );
            }
            BogaeShapeKind::Text => {
                put("kind", Value::String("text".to_string()));
                put(
                    "x",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["x"],
                        0,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "y",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["y"],
                        1,
                        Value::Fixed64(Fixed64::ZERO),
                    ),
                );
                put(
                    "text",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["글", "내용", "text"],
                        2,
                        Value::String(String::new()),
                    ),
                );
                put(
                    "size",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["크기", "size"],
                        usize::MAX,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.08)),
                    ),
                );
                put(
                    "color",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "글색", "color"],
                        usize::MAX,
                        Value::String("#f8fafc".to_string()),
                    ),
                );
            }
            BogaeShapeKind::Polyline => {
                let points = bogae_pick_arg(&shape, &["점들", "좌표들", "points"], usize::MAX)
                    .and_then(|text| self.parse_bogae_value_token(locals, text))
                    .or_else(|| bogae_points_from_positional(&shape, locals, self))?;
                put("kind", Value::String("polyline".to_string()));
                put("points", points);
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["색", "선색", "stroke", "color"],
                        usize::MAX,
                        Value::String("#22c55e".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.02)),
                    ),
                );
            }
            BogaeShapeKind::Polygon => {
                let points = bogae_pick_arg(&shape, &["점들", "좌표들", "points"], usize::MAX)
                    .and_then(|text| self.parse_bogae_value_token(locals, text))
                    .or_else(|| bogae_points_from_positional(&shape, locals, self))?;
                put("kind", Value::String("polygon".to_string()));
                put("points", points);
                put(
                    "fill",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["채움색", "채움", "fill", "색", "color"],
                        usize::MAX,
                        Value::String("#38bdf844".to_string()),
                    ),
                );
                put(
                    "stroke",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["선색", "stroke"],
                        usize::MAX,
                        Value::String("#0f172a".to_string()),
                    ),
                );
                put(
                    "width",
                    bogae_read_arg(
                        self,
                        locals,
                        &shape,
                        &["굵기", "width"],
                        usize::MAX,
                        Value::Fixed64(Fixed64::from_f64_lossy(0.02)),
                    ),
                );
            }
        }
        if let Some(layer_index) = bogae_pick_arg(
            &shape,
            &["층", "레이어", "layer_index", "layer", "z", "z_index"],
            usize::MAX,
        )
        .and_then(|text| self.parse_bogae_value_token(locals, text))
        {
            put("layer_index", layer_index);
        }
        if let Some(group_id) = bogae_pick_arg(
            &shape,
            &["그룹", "묶음", "group_id", "group", "groupId"],
            usize::MAX,
        )
        .and_then(|text| self.parse_bogae_value_token(locals, text))
        {
            put("group_id", group_id);
        }
        Some(Value::Map(map))
    }

    fn parse_bogae_value_token(
        &mut self,
        locals: &mut HashMap<String, Value>,
        token: &str,
    ) -> Option<Value> {
        let text = token.trim();
        if text.is_empty() {
            return None;
        }
        if let Some(unquoted) = strip_wrapping_quotes(text) {
            return Some(Value::String(unquoted));
        }
        if text == "참" {
            return Some(Value::Bool(true));
        }
        if text == "거짓" {
            return Some(Value::Bool(false));
        }
        let compact_numeric: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
        if let Ok(num) = compact_numeric.parse::<f64>() {
            if num.is_finite() {
                return Some(Value::Fixed64(Fixed64::from_f64_lossy(num)));
            }
        }
        if text.starts_with('#') {
            return Some(Value::String(text.to_string()));
        }
        if let Some(value) = locals.get(text) {
            return Some(value.clone());
        }
        self.get_resource(text)
    }

    fn eval_callable(
        &mut self,
        locals: &HashMap<String, Value>,
        callable: &Value,
        args: Vec<Value>,
        source_span: Option<SourceSpan>,
    ) -> Result<Value, EvalError> {
        match callable {
            Value::Lambda(lambda) => self.eval_lambda(lambda, &args),
            Value::String(name) => {
                let func = name.trim_start_matches('#');
                self.eval_call(locals, func, args, source_span)
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

    fn eval_pipe(
        &mut self,
        locals: &mut HashMap<String, Value>,
        stages: &[Expr],
    ) -> Result<Value, EvalError> {
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

    fn eval_call(
        &mut self,
        locals: &HashMap<String, Value>,
        func: &str,
        args: Vec<Value>,
        source_span: Option<SourceSpan>,
    ) -> Result<Value, EvalError> {
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
            "수" => {
                if args.len() != 1 {
                    return Err("수는 인자 1개를 받습니다".to_string().into());
                }
                let value = value_to_fixed64_coerce(&args[0])?;
                Ok(Value::Fixed64(value))
            }
            "바른수" => {
                if args.len() != 1 {
                    return Err("바른수는 인자 1개를 받습니다".to_string().into());
                }
                let value = value_to_i64_strict(&args[0])?;
                Ok(Value::Fixed64(Fixed64::from_i64(value)))
            }
            "큰바른수" => {
                if args.len() != 1 {
                    return Err("큰바른수는 인자 1개를 받습니다".to_string().into());
                }
                Ok(make_big_int_value(&args[0])?)
            }
            "나눔수" => {
                if args.len() != 2 {
                    return Err("나눔수는 인자 2개(분자, 분모)를 받습니다".to_string().into());
                }
                Ok(make_rational_value(&args[0], &args[1])?)
            }
            "곱수" => {
                if args.len() != 1 {
                    return Err("곱수는 인자 1개를 받습니다".to_string().into());
                }
                let (value, status) = make_factor_value(&args[0])?;
                self.note_factor_route_from_value(&value);
                if status == FACTOR_DECOMP_STATUS_DEFERRED {
                    self.emit_factor_decomposition_deferred_diag(&value, source_span);
                }
                Ok(value)
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
                    return Err("짝맞춤.바꾼값은 짝맞춤 인자가 필요합니다"
                        .to_string()
                        .into());
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
            "흐름.만들기" => {
                if args.len() != 1 && args.len() != 2 {
                    return Err("흐름.만들기는 인자 1개(용량) 또는 2개(용량, 초기값차림)를 받습니다"
                        .to_string()
                        .into());
                }
                let capacity = value_to_i64(&args[0])?;
                if capacity <= 0 {
                    return Err("흐름 용량은 1 이상이어야 합니다".to_string().into());
                }
                let mut stream = stream_new(capacity as usize)?;
                if args.len() == 2 {
                    let Value::List(initial) = &args[1] else {
                        return Err("흐름.만들기의 두 번째 인자는 차림이어야 합니다"
                            .to_string()
                            .into());
                    };
                    for item in initial {
                        stream = stream_push(stream, item.clone());
                    }
                }
                Ok(stream_to_value(&stream))
            }
            "흐름.밀어넣기" => {
                if args.len() != 2 {
                    return Err("흐름.밀어넣기는 인자 2개(흐름, 값)를 받습니다"
                        .to_string()
                        .into());
                }
                let stream = stream_from_value(&args[0])?;
                let stream = stream_push(stream, args[1].clone());
                Ok(stream_to_value(&stream))
            }
            "흐름.차림" => {
                if args.len() != 1 {
                    return Err("흐름.차림은 인자 1개(흐름)를 받습니다".to_string().into());
                }
                let stream = stream_from_value(&args[0])?;
                Ok(Value::List(stream_oldest_to_newest(&stream)))
            }
            "흐름.최근값" => {
                if args.len() != 1 {
                    return Err("흐름.최근값은 인자 1개(흐름)를 받습니다".to_string().into());
                }
                let stream = stream_from_value(&args[0])?;
                if stream.len == 0 || stream.capacity == 0 {
                    Ok(Value::None)
                } else {
                    Ok(stream.buffer[stream.head].clone())
                }
            }
            "흐름.길이" => {
                if args.len() != 1 {
                    return Err("흐름.길이는 인자 1개(흐름)를 받습니다".to_string().into());
                }
                let stream = stream_from_value(&args[0])?;
                Ok(Value::Fixed64(Fixed64::from_i64(stream.len as i64)))
            }
            "흐름.용량" => {
                if args.len() != 1 {
                    return Err("흐름.용량은 인자 1개(흐름)를 받습니다".to_string().into());
                }
                let stream = stream_from_value(&args[0])?;
                Ok(Value::Fixed64(Fixed64::from_i64(stream.capacity as i64)))
            }
            "흐름.비우기" => {
                if args.len() != 1 {
                    return Err("흐름.비우기는 인자 1개(흐름)를 받습니다".to_string().into());
                }
                let stream = stream_from_value(&args[0])?;
                Ok(stream_to_value(&stream_clear(stream)))
            }
            "흐름.잘라보기" => {
                if args.len() != 2 {
                    return Err("흐름.잘라보기는 인자 2개(흐름, 개수)를 받습니다"
                        .to_string()
                        .into());
                }
                let stream = stream_from_value(&args[0])?;
                let count = value_to_i64(&args[1])?;
                if count < 0 {
                    return Err("흐름.잘라보기 개수는 0 이상이어야 합니다"
                        .to_string()
                        .into());
                }
                let ordered = stream_oldest_to_newest(&stream);
                let take = (count as usize).min(ordered.len());
                let start = ordered.len().saturating_sub(take);
                Ok(Value::List(ordered[start..].to_vec()))
            }
            "키목록" => {
                if args.len() != 1 {
                    return Err("키목록은 인자 1개를 받습니다".to_string().into());
                }
                let Value::Pack(pack) = &args[0] else {
                    return Err("키목록은 묶음 인자를 받습니다".to_string().into());
                };
                let keys = pack.keys().cloned().map(Value::String).collect::<Vec<_>>();
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
                if matches!(
                    formula.dialect,
                    FormulaDialect::Latex | FormulaDialect::Other(_)
                ) {
                    return Err("FATAL:FORMULA_DIALECT_UNSUPPORTED".to_string().into());
                }
                let parsed = parse_formula_value(formula)?;
                let required = &parsed.vars;
                let provided: BTreeSet<String> = pack.keys().cloned().collect();
                let missing: Vec<String> = required.difference(&provided).cloned().collect();
                if !missing.is_empty() {
                    return Err(
                        format!("풀기: 주입 키가 누락되었습니다: {}", missing.join(", ")).into(),
                    );
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
            "살피기" => {
                if args.len() != 2 {
                    return Err("살피기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::Assertion(assertion) = &args[0] else {
                    return Err("살피기는 세움값 인자가 필요합니다".to_string().into());
                };
                let bindings = assertion_bindings_from_value(&args[1])?;
                self.eval_assertion(assertion, bindings)
            }
            "처음으로" => {
                if args.len() != 1 {
                    return Err("처음으로는 인자 1개를 받습니다".to_string().into());
                }
                let Value::StateMachine(machine) = &args[0] else {
                    return Err("처음으로는 상태머신값 인자가 필요합니다".to_string().into());
                };
                Ok(Value::String(machine.initial.clone()))
            }
            "지금상태" => {
                if args.len() != 2 {
                    return Err("지금상태는 인자 2개를 받습니다".to_string().into());
                }
                let Value::StateMachine(machine) = &args[0] else {
                    return Err("지금상태는 상태머신값 인자가 필요합니다".to_string().into());
                };
                let Value::String(state) = &args[1] else {
                    return Err("지금상태는 글 상태 이름 인자가 필요합니다"
                        .to_string()
                        .into());
                };
                ensure_declared_state(machine, state)?;
                Ok(Value::String(state.clone()))
            }
            "다음으로" => {
                if args.len() != 2 {
                    return Err("다음으로는 인자 2개를 받습니다".to_string().into());
                }
                let Value::StateMachine(machine) = &args[0] else {
                    return Err("다음으로는 상태머신값 인자가 필요합니다".to_string().into());
                };
                let Value::String(state) = &args[1] else {
                    return Err("다음으로는 글 상태 이름 인자가 필요합니다"
                        .to_string()
                        .into());
                };
                ensure_declared_state(machine, state)?;
                let candidates: Vec<&StateTransition> = machine
                    .transitions
                    .iter()
                    .filter(|t| t.from == *state)
                    .collect();
                if candidates.is_empty() {
                    return Err(format!("E_STATE_TRANSITION_MISSING: {}", state).into());
                }
                let mut selected: Option<&StateTransition> = None;
                let mut guard_rejected = false;
                for transition in candidates {
                    let bindings =
                        build_state_machine_transition_bindings(&transition.from, &transition.to);
                    if let Some(guard_name) = &transition.guard_name {
                        if !self.eval_state_machine_guard(locals, guard_name, &bindings)? {
                            guard_rejected = true;
                            continue;
                        }
                    }
                    selected = Some(transition);
                    break;
                }
                let Some(next) = selected else {
                    if guard_rejected {
                        return Err(format!("E_STATE_TRANSITION_GUARD_REJECTED: {}", state).into());
                    }
                    return Err(format!("E_STATE_TRANSITION_MISSING: {}", state).into());
                };
                self.apply_state_machine_transition(locals, machine, next)?;
                self.emit_state_machine_transition(
                    state,
                    &next.to,
                    next.guard_name.as_deref(),
                    next.action_name.as_deref(),
                );
                Ok(Value::String(next.to.clone()))
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
                        let step = UnitValue {
                            value: Fixed64::from_i64(1),
                            dim: start.dim,
                        };
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
                        items.push(unit_value_to_value(UnitValue {
                            value: current,
                            dim: start.dim,
                        }));
                        current = current.saturating_add(step_value);
                    }
                } else {
                    while current.raw_i64() >= end.value.raw_i64() {
                        items.push(unit_value_to_value(UnitValue {
                            value: current,
                            dim: start.dim,
                        }));
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
                let step = UnitValue {
                    value: Fixed64::from_i64(1),
                    dim: start.dim,
                };
                let mut items = Vec::new();
                let mut current = start.value;
                if start.value.raw_i64() <= end.value.raw_i64() {
                    if include_end {
                        while current.raw_i64() <= end.value.raw_i64() {
                            items.push(unit_value_to_value(UnitValue {
                                value: current,
                                dim: start.dim,
                            }));
                            current = current.saturating_add(step.value);
                        }
                    } else {
                        while current.raw_i64() < end.value.raw_i64() {
                            items.push(unit_value_to_value(UnitValue {
                                value: current,
                                dim: start.dim,
                            }));
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
                let idx = text
                    .find(pattern)
                    .map(|byte_idx| text[..byte_idx].chars().count() as i64);
                Ok(Value::Fixed64(Fixed64::from_i64(idx.unwrap_or(-1))))
            }
            "찾기?" => {
                if args.len() != 2 {
                    return Err("찾기?는 인자 2개를 받습니다".to_string().into());
                }
                let Value::Map(map) = &args[0] else {
                    return Err("찾기?는 짝맞춤 인자를 받습니다".to_string().into());
                };
                Ok(map_get(map, &args[1]))
            }
            "정규맞추기" => {
                if args.len() != 2 {
                    return Err("정규맞추기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("정규맞추기 첫 인자는 글이어야 합니다".to_string().into());
                };
                let Value::Regex(regex) = &args[1] else {
                    return Err("정규맞추기 둘째 인자는 정규식이어야 합니다"
                        .to_string()
                        .into());
                };
                Ok(Value::Bool(regex_is_full_match(text, regex)?))
            }
            "정규찾기" => {
                if args.len() != 2 {
                    return Err("정규찾기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("정규찾기 첫 인자는 글이어야 합니다".to_string().into());
                };
                let Value::Regex(regex) = &args[1] else {
                    return Err("정규찾기 둘째 인자는 정규식이어야 합니다"
                        .to_string()
                        .into());
                };
                let matched = regex_find_first(text, regex)?;
                Ok(matched.map(Value::String).unwrap_or(Value::None))
            }
            "정규캡처하기" => {
                if args.len() != 2 {
                    return Err("정규캡처하기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("정규캡처하기 첫 인자는 글이어야 합니다".to_string().into());
                };
                let Value::Regex(regex) = &args[1] else {
                    return Err("정규캡처하기 둘째 인자는 정규식이어야 합니다"
                        .to_string()
                        .into());
                };
                Ok(Value::List(
                    regex_capture_first(text, regex)?
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ))
            }
            "정규이름캡처하기" => {
                if args.len() != 2 {
                    return Err("정규이름캡처하기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("정규이름캡처하기 첫 인자는 글이어야 합니다"
                        .to_string()
                        .into());
                };
                let Value::Regex(regex) = &args[1] else {
                    return Err("정규이름캡처하기 둘째 인자는 정규식이어야 합니다"
                        .to_string()
                        .into());
                };
                Ok(Value::Map(regex_named_capture_first(text, regex)?))
            }
            "정규바꾸기" => {
                if args.len() != 3 {
                    return Err("정규바꾸기는 인자 3개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("정규바꾸기 첫 인자는 글이어야 합니다".to_string().into());
                };
                let Value::Regex(regex) = &args[1] else {
                    return Err("정규바꾸기 둘째 인자는 정규식이어야 합니다"
                        .to_string()
                        .into());
                };
                let Value::String(replacement) = &args[2] else {
                    return Err("정규바꾸기 셋째 인자는 글이어야 합니다".to_string().into());
                };
                Ok(Value::String(regex_replace_all(text, regex, replacement)?))
            }
            "정규나누기" => {
                if args.len() != 2 {
                    return Err("정규나누기는 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(text) = &args[0] else {
                    return Err("정규나누기 첫 인자는 글이어야 합니다".to_string().into());
                };
                let Value::Regex(regex) = &args[1] else {
                    return Err("정규나누기 둘째 인자는 정규식이어야 합니다"
                        .to_string()
                        .into());
                };
                Ok(Value::List(
                    regex_split(text, regex)?
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ))
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
                    let key = self.eval_callable(
                        locals,
                        &func,
                        vec![item.clone()],
                        source_span.clone(),
                    )?;
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
                    let verdict = self.eval_callable(
                        locals,
                        &func,
                        vec![item.clone()],
                        source_span.clone(),
                    )?;
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
                    let mapped =
                        self.eval_callable(locals, &func, vec![item], source_span.clone())?;
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
                    let mapped =
                        self.eval_callable(locals, &func, vec![item], source_span.clone())?;
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
                    let _ = self.eval_callable(locals, &func, vec![item], source_span.clone())?;
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
                    acc = self.eval_callable(locals, &func, vec![acc, item], source_span.clone())?;
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
                    _ => Err("붙이기는 차림/글 조합 또는 차림/차림 조합만 지원합니다"
                        .to_string()
                        .into()),
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
                Ok(unit_value_to_value(UnitValue {
                    value,
                    dim: qty.dim,
                }))
            }
            "천장" => {
                if args.len() != 1 {
                    return Err("천장은 인자 1개를 받습니다".to_string().into());
                }
                let qty = unit_value_from_value(&args[0])?;
                let value = fixed64_ceil(qty.value);
                Ok(unit_value_to_value(UnitValue {
                    value,
                    dim: qty.dim,
                }))
            }
            "반올림" => {
                if args.len() != 1 {
                    return Err("반올림은 인자 1개를 받습니다".to_string().into());
                }
                let qty = unit_value_from_value(&args[0])?;
                let value = fixed64_round_even(qty.value);
                Ok(unit_value_to_value(UnitValue {
                    value,
                    dim: qty.dim,
                }))
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
            "지니" | "지니계수" => {
                let values = expect_numeric_list_arg(&args, "지니")?;
                if values.is_empty() {
                    return Ok(Value::None);
                }
                let mut total = Fixed64::ZERO;
                for value in &values {
                    if value.value.raw_i64() < 0 {
                        return Err("지니 입력은 음수가 될 수 없습니다".to_string().into());
                    }
                    total = total.saturating_add(value.value);
                }
                if total.raw_i64() == 0 {
                    return Ok(Value::Fixed64(Fixed64::ZERO));
                }
                let mut pair_sum = Fixed64::ZERO;
                for i in 0..values.len() {
                    for j in (i + 1)..values.len() {
                        let diff = values[i].value.saturating_sub(values[j].value);
                        pair_sum = pair_sum.saturating_add(fixed64_abs(diff));
                    }
                }
                let count = Fixed64::from_i64(values.len() as i64);
                let denom = count.saturating_mul(total);
                let gini = pair_sum
                    .try_div(denom)
                    .map_err(|_| "지니 계산 중 0으로 나눌 수 없습니다".to_string())?;
                Ok(Value::Fixed64(gini))
            }
            "분위수" | "백분위수" => {
                let (mut values, p, mode) = expect_quantile_args(&args)?;
                if values.is_empty() {
                    return Ok(Value::None);
                }
                if values.len() == 1 {
                    return Ok(unit_value_to_value(values[0]));
                }
                values.sort_by(|a, b| a.value.raw_i64().cmp(&b.value.raw_i64()));
                match mode {
                    PercentileMode::Linear => {
                        let max_index = Fixed64::from_i64((values.len() - 1) as i64);
                        let pos = p.saturating_mul(max_index);
                        let low = fixed64_floor(pos);
                        let high = fixed64_ceil(pos);
                        let low_idx = fixed64_to_nonnegative_index(low)?;
                        let high_idx = fixed64_to_nonnegative_index(high)?;
                        if low_idx >= values.len() || high_idx >= values.len() {
                            return Err("분위수 인덱스가 범위를 벗어났습니다".to_string().into());
                        }
                        if low_idx == high_idx {
                            return Ok(unit_value_to_value(values[low_idx]));
                        }
                        let frac = pos.saturating_sub(low);
                        let base = values[low_idx];
                        let next = values[high_idx];
                        let delta = next.value.saturating_sub(base.value);
                        let step = delta.saturating_mul(frac);
                        let interpolated = UnitValue {
                            value: base.value.saturating_add(step),
                            dim: base.dim,
                        };
                        Ok(unit_value_to_value(interpolated))
                    }
                    PercentileMode::NearestRank => {
                        let idx = nearest_rank_index(p, values.len())?;
                        Ok(unit_value_to_value(values[idx]))
                    }
                }
            }
            "적분.오일러" => {
                if args.len() != 3 {
                    return Err("적분.오일러는 인자 3개를 받습니다".to_string().into());
                }
                let value = unit_value_from_value(&args[0])?;
                let rate = unit_value_from_value(&args[1])?;
                let dt = unit_value_from_value(&args[2])?;
                let delta = rate.mul(dt);
                let out = value.add(delta).map_err(unit_error)?;
                Ok(unit_value_to_value(out))
            }
            "적분.반암시적오일러" => {
                if args.len() != 4 {
                    return Err("적분.반암시적오일러는 인자 4개를 받습니다"
                        .to_string()
                        .into());
                }
                let position = unit_value_from_value(&args[0])?;
                let velocity = unit_value_from_value(&args[1])?;
                let acceleration = unit_value_from_value(&args[2])?;
                let dt = unit_value_from_value(&args[3])?;
                let next_velocity = velocity.add(acceleration.mul(dt)).map_err(unit_error)?;
                let next_position = position.add(next_velocity.mul(dt)).map_err(unit_error)?;
                Ok(Value::List(vec![
                    unit_value_to_value(next_position),
                    unit_value_to_value(next_velocity),
                ]))
            }
            "보간.선형" => {
                if args.len() != 3 {
                    return Err("보간.선형은 인자 3개를 받습니다".to_string().into());
                }
                let start = unit_value_from_value(&args[0])?;
                let end = unit_value_from_value(&args[1])?;
                let t = unit_value_from_value(&args[2])?;
                if !t.is_dimensionless() {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: t.dim,
                        right: UnitDim::NONE,
                    }));
                }
                let delta = end.sub(start).map_err(unit_error)?;
                let out = start.add(delta.mul(t)).map_err(unit_error)?;
                Ok(unit_value_to_value(out))
            }
            "보간.계단" => {
                if args.len() != 4 {
                    return Err("보간.계단은 인자 4개를 받습니다".to_string().into());
                }
                let start = unit_value_from_value(&args[0])?;
                let end = unit_value_from_value(&args[1])?;
                if start.dim != end.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: start.dim,
                        right: end.dim,
                    }));
                }
                let t = unit_value_from_value(&args[2])?;
                if !t.is_dimensionless() {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: t.dim,
                        right: UnitDim::NONE,
                    }));
                }
                let threshold = unit_value_from_value(&args[3])?;
                if !threshold.is_dimensionless() {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: threshold.dim,
                        right: UnitDim::NONE,
                    }));
                }
                if t.value < threshold.value {
                    Ok(unit_value_to_value(start))
                } else {
                    Ok(unit_value_to_value(end))
                }
            }
            "필터.이동평균" => {
                if args.len() != 2 {
                    return Err("필터.이동평균은 인자 2개를 받습니다".to_string().into());
                }
                let Value::List(window) = &args[0] else {
                    return Err("필터.이동평균 첫 인자는 차림이어야 합니다"
                        .to_string()
                        .into());
                };
                let next = unit_value_from_value(&args[1])?;
                let mut next_window = Vec::with_capacity(window.len() + 1);
                let mut total = next.value;
                for item in window {
                    let value = unit_value_from_value(item)?;
                    if value.dim != next.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: next.dim,
                            right: value.dim,
                        }));
                    }
                    total = total.saturating_add(value.value);
                    next_window.push(unit_value_to_value(value));
                }
                next_window.push(unit_value_to_value(next));
                let count = Fixed64::from_i64(next_window.len() as i64);
                let avg = total
                    .try_div(count)
                    .map_err(|_| "필터.이동평균 계산 중 0으로 나눌 수 없습니다".to_string())?;
                Ok(Value::List(vec![
                    Value::List(next_window),
                    unit_value_to_value(UnitValue {
                        value: avg,
                        dim: next.dim,
                    }),
                ]))
            }
            "필터.지수평활" => {
                if args.len() != 3 {
                    return Err("필터.지수평활은 인자 3개를 받습니다".to_string().into());
                }
                let prev = unit_value_from_value(&args[0])?;
                let next = unit_value_from_value(&args[1])?;
                if prev.dim != next.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: prev.dim,
                        right: next.dim,
                    }));
                }
                let alpha = unit_value_from_value(&args[2])?;
                if !alpha.is_dimensionless() {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: alpha.dim,
                        right: UnitDim::NONE,
                    }));
                }
                if alpha.value.raw_i64() < 0 || alpha.value.raw_i64() > Fixed64::ONE.raw_i64() {
                    return Err("필터.지수평활 alpha는 0..1 범위여야 합니다"
                        .to_string()
                        .into());
                }
                let one = Fixed64::from_i64(1);
                let keep = one.saturating_sub(alpha.value);
                let out = prev
                    .value
                    .saturating_mul(keep)
                    .saturating_add(next.value.saturating_mul(alpha.value));
                Ok(unit_value_to_value(UnitValue {
                    value: out,
                    dim: prev.dim,
                }))
            }
            "미분.중앙차분" => {
                let (formula, var_name, point, step) =
                    expect_numeric_derivative_args(&args, "미분.중앙차분")?;
                let prepared = prepare_numeric_formula(&formula, &var_name, "미분.중앙차분")?;
                let two = Fixed64::from_i64(2);

                let x_plus = UnitValue {
                    value: point.value.saturating_add(step.value),
                    dim: point.dim,
                };
                let x_minus = UnitValue {
                    value: point.value.saturating_sub(step.value),
                    dim: point.dim,
                };
                let y_plus = eval_numeric_formula_at(&prepared, x_plus, "미분.중앙차분")?;
                let y_minus = eval_numeric_formula_at(&prepared, x_minus, "미분.중앙차분")?;
                if y_plus.dim != y_minus.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: y_plus.dim,
                        right: y_minus.dim,
                    }));
                }
                let denom = step.value.saturating_mul(two);
                let d_h = y_plus
                    .value
                    .saturating_sub(y_minus.value)
                    .try_div(denom)
                    .map_err(|_| "미분.중앙차분 계산 중 0으로 나눌 수 없습니다".to_string())?;

                let step2 = step.value.saturating_mul(two);
                let x_plus2 = UnitValue {
                    value: point.value.saturating_add(step2),
                    dim: point.dim,
                };
                let x_minus2 = UnitValue {
                    value: point.value.saturating_sub(step2),
                    dim: point.dim,
                };
                let y_plus2 = eval_numeric_formula_at(&prepared, x_plus2, "미분.중앙차분")?;
                let y_minus2 = eval_numeric_formula_at(&prepared, x_minus2, "미분.중앙차분")?;
                if y_plus2.dim != y_minus2.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: y_plus2.dim,
                        right: y_minus2.dim,
                    }));
                }
                let denom2 = step2.saturating_mul(two);
                let d_2h = y_plus2
                    .value
                    .saturating_sub(y_minus2.value)
                    .try_div(denom2)
                    .map_err(|_| "미분.중앙차분 계산 중 0으로 나눌 수 없습니다".to_string())?;
                let three = Fixed64::from_i64(3);
                let richardson_delta = d_h.saturating_sub(d_2h);
                let richardson_correction = richardson_delta
                    .try_div(three)
                    .map_err(|_| "미분.중앙차분 계산 중 0으로 나눌 수 없습니다".to_string())?;
                let richardson_value = d_h.saturating_add(richardson_correction);
                let richardson_error = fixed64_abs(richardson_delta)
                    .try_div(three)
                    .map_err(|_| "미분.중앙차분 계산 중 0으로 나눌 수 없습니다".to_string())?;
                let diff_dim = y_plus.dim.sub(step.dim);
                Ok(Value::List(vec![
                    unit_value_to_value(UnitValue {
                        value: richardson_value,
                        dim: diff_dim,
                    }),
                    unit_value_to_value(UnitValue {
                        value: richardson_error,
                        dim: diff_dim,
                    }),
                    Value::String("중앙차분".to_string()),
                ]))
            }
            "적분.사다리꼴" => {
                let (formula, var_name, start, end, step) =
                    expect_numeric_integral_args(&args, "적분.사다리꼴")?;
                let prepared = prepare_numeric_formula(&formula, &var_name, "적분.사다리꼴")?;
                let (lower, upper, sign) = if end.value < start.value {
                    (end, start, -1_i64)
                } else {
                    (start, end, 1_i64)
                };
                let interval = upper.value.saturating_sub(lower.value);
                let n_est = interval
                    .try_div(step.value)
                    .map_err(|_| "적분.사다리꼴 계산 중 0으로 나눌 수 없습니다".to_string())?;
                let mut n = fixed64_to_nonnegative_index(fixed64_ceil(n_est))?;
                if n == 0 {
                    n = 1;
                }
                let coarse = numeric_trapezoid_integral(&prepared, lower, upper, n)?;
                let refined = numeric_trapezoid_integral(
                    &prepared,
                    lower,
                    upper,
                    n.saturating_mul(2).max(2),
                )?;
                if coarse.dim != refined.dim {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: coarse.dim,
                        right: refined.dim,
                    }));
                }
                let three = Fixed64::from_i64(3);
                let richardson_delta = refined.value.saturating_sub(coarse.value);
                let richardson_correction = richardson_delta
                    .try_div(three)
                    .map_err(|_| "적분.사다리꼴 계산 중 0으로 나눌 수 없습니다".to_string())?;
                let richardson_value = refined.value.saturating_add(richardson_correction);
                let richardson_error = fixed64_abs(richardson_delta)
                    .try_div(three)
                    .map_err(|_| "적분.사다리꼴 계산 중 0으로 나눌 수 없습니다".to_string())?;
                let mut approx = refined;
                approx.value = richardson_value;
                if sign < 0 {
                    approx.value = Fixed64::ZERO.saturating_sub(approx.value);
                }
                let err = UnitValue {
                    value: richardson_error,
                    dim: coarse.dim,
                };
                Ok(Value::List(vec![
                    unit_value_to_value(approx),
                    unit_value_to_value(err),
                    Value::String("사다리꼴".to_string()),
                ]))
            }
            "abs" => {
                if args.len() != 1 {
                    return Err("abs는 인자 1개를 받습니다".to_string().into());
                }
                let qty = unit_value_from_value(&args[0])?;
                let value = fixed64_abs(qty.value);
                Ok(unit_value_to_value(UnitValue {
                    value,
                    dim: qty.dim,
                }))
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
                let out = if left.value <= right.value {
                    left
                } else {
                    right
                };
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
                let out = if left.value >= right.value {
                    left
                } else {
                    right
                };
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
            "sin" | "cos" | "tan" => {
                if args.len() != 1 {
                    return Err(format!("{}는 인자 1개를 받습니다", func).into());
                }
                let qty = unit_value_from_value(&args[0])?;
                if !(qty.is_dimensionless() || qty.dim == UnitDim::ANGLE) {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: qty.dim,
                        right: UnitDim::ANGLE,
                    }));
                }
                let angle = fixed64_to_f64(qty.value);
                let out = match func {
                    "sin" => libm::sin(angle),
                    "cos" => libm::cos(angle),
                    _ => libm::tan(angle),
                };
                Ok(Value::Fixed64(Fixed64::from_f64_lossy(out)))
            }
            "asin" | "acos" | "atan" => {
                if args.len() != 1 {
                    return Err(format!("{}는 인자 1개를 받습니다", func).into());
                }
                let qty = unit_value_from_value(&args[0])?;
                if !qty.is_dimensionless() {
                    return Err(unit_error(UnitError::DimensionMismatch {
                        left: qty.dim,
                        right: UnitDim::NONE,
                    }));
                }
                let val = fixed64_to_f64(qty.value);
                let out = match func {
                    "asin" => libm::asin(val),
                    "acos" => libm::acos(val),
                    _ => libm::atan(val),
                };
                Ok(Value::Fixed64(Fixed64::from_f64_lossy(out)))
            }
            "atan2" => {
                if args.len() != 2 {
                    return Err("atan2는 인자 2개를 받습니다".to_string().into());
                }
                let y = unit_value_from_value(&args[0])?;
                let x = unit_value_from_value(&args[1])?;
                let out = libm::atan2(fixed64_to_f64(y.value), fixed64_to_f64(x.value));
                Ok(Value::Fixed64(Fixed64::from_f64_lossy(out)))
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
                    Value::String(path) => {
                        Ok(Value::ResourceHandle(ResourceHandle::from_path(path)))
                    }
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
                    return Err("무작위정수의 최소/최대가 올바르지 않습니다"
                        .to_string()
                        .into());
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
                if let Some(exact) = eval_exact_numeric_infix(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                let sum = l.add(r).map_err(unit_error)?;
                Ok(unit_value_to_value(sum))
            }
            "-" => {
                if let Some(exact) = eval_exact_numeric_infix(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                let diff = l.sub(r).map_err(unit_error)?;
                Ok(unit_value_to_value(diff))
            }
            "*" => {
                if let Some(exact) = eval_exact_numeric_infix(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                Ok(unit_value_to_value(l.mul(r)))
            }
            "/" => {
                if let Some(exact) = eval_exact_numeric_infix(op, &left, &right)? {
                    return Ok(exact);
                }
                let l = unit_value_from_value(&left)?;
                let r = unit_value_from_value(&right)?;
                let div = l.div(r).map_err(unit_error)?;
                Ok(unit_value_to_value(div))
            }
            "%" => {
                if let Some(exact) = eval_exact_numeric_infix(op, &left, &right)? {
                    return Ok(exact);
                }
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
                if let Some(eq) = eval_exact_numeric_compare_infix(op, &left, &right)? {
                    return Ok(Value::Bool(eq));
                }
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
                if let Some(cmp) = eval_exact_numeric_compare_infix(op, &left, &right)? {
                    return Ok(Value::Bool(cmp));
                }
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
            ddonirang_lang::ContractKind::Pre => {
                ("CONTRACT_PRE", "PRE_VIOLATION", "pre", "contract:pre")
            }
            ddonirang_lang::ContractKind::Post => {
                ("CONTRACT_POST", "POST_VIOLATION", "post", "contract:post")
            }
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

    fn emit_factor_decomposition_deferred_diag(
        &mut self,
        value: &Value,
        source_span: Option<SourceSpan>,
    ) {
        let (deferred_value, deferred_reason, deferred_route, deferred_bits) = match value {
            Value::Pack(fields) => {
                let raw = fields.get("값").and_then(|raw| match raw {
                    Value::String(text) => Some(text.clone()),
                    _ => None,
                });
                let reason = fields
                    .get(FACTOR_DECOMP_DEFERRED_REASON_KEY)
                    .and_then(|raw| match raw {
                        Value::String(text) => Some(text.clone()),
                        _ => None,
                    });
                let route = fields.get(FACTOR_DECOMP_ROUTE_KEY).and_then(|raw| match raw {
                    Value::String(text) => Some(text.clone()),
                    _ => None,
                });
                let bits = fields.get(FACTOR_DECOMP_BITS_KEY).and_then(|raw| match raw {
                    Value::Fixed64(value) => Some(value.int_part()),
                    _ => None,
                });
                (raw, reason, route, bits)
            }
            _ => (None, None, None, None),
        };
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let message = match (deferred_value, deferred_reason.clone()) {
            (Some(raw), Some(reason)) => Some(format!("곱수 분해를 지연했습니다({reason}): {raw}")),
            (Some(raw), None) => Some(format!("곱수 분해를 지연했습니다: {raw}")),
            _ => None,
        };
        let agg_suffix = self
            .factor_route_summary_components()
            .map(|(routes, total, _, _, _)| format!(";agg_routes={routes};agg_total={total}"))
            .unwrap_or_default();
        let trace_text = match (deferred_route, deferred_bits) {
            (Some(route), Some(bits)) => Some(format!("route={route};bits={bits}{agg_suffix}")),
            (Some(route), None) => Some(format!("route={route}{agg_suffix}")),
            (None, Some(bits)) => Some(format!("bits={bits}{agg_suffix}")),
            (None, None) => None,
        };
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED.to_string(),
            rule_id: NUMERIC_DIAG_RULE_ID_FACTOR_DECOMP_DEFERRED.to_string(),
            reason: NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED.to_string(),
            sub_reason: Some(
                deferred_reason.unwrap_or_else(|| FACTOR_DECOMP_STATUS_DEFERRED.to_string()),
            ),
            mode: Some("알림".to_string()),
            contract_kind: None,
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span,
            expr: Some(ExprTrace {
                tag: NUMERIC_DIAG_TAG_FACTOR_DECOMP_DEFERRED.to_string(),
                text: trace_text,
            }),
            message,
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
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

    fn note_factor_route_from_value(&mut self, value: &Value) {
        let (route, bits) = match value {
            Value::Pack(fields) => {
                let route = fields.get(FACTOR_DECOMP_ROUTE_KEY).and_then(|raw| match raw {
                    Value::String(route) => Some(route.clone()),
                    _ => None,
                });
                let bits = fields.get(FACTOR_DECOMP_BITS_KEY).and_then(|raw| match raw {
                    Value::Fixed64(value) if value.int_part() >= 0 => Some(value.int_part() as u64),
                    _ => None,
                });
                (route, bits)
            }
            _ => (None, None),
        };
        if let Some(route) = route {
            *self.factor_route_counts.entry(route).or_insert(0) += 1;
        }
        if let Some(bits) = bits {
            self.factor_bits_total = self.factor_bits_total.saturating_add(bits as u128);
            self.factor_bits_min = Some(match self.factor_bits_min {
                Some(prev) => prev.min(bits),
                None => bits,
            });
            self.factor_bits_max = self.factor_bits_max.max(bits);
        }
    }

    fn flush_factor_route_metrics_resource(&mut self) {
        let Some((summary, total, bit_min, bit_max, bit_sum)) = self.factor_route_summary_components() else {
            return;
        };
        let policy_summary = factor_policy_summary_text();
        let total_i64 = if total > i64::MAX as u64 {
            i64::MAX
        } else {
            total as i64
        };
        let bit_min_i64 = if bit_min > i64::MAX as u64 {
            i64::MAX
        } else {
            bit_min as i64
        };
        let bit_max_i64 = if bit_max > i64::MAX as u64 {
            i64::MAX
        } else {
            bit_max as i64
        };
        let bit_sum_i64 = if bit_sum > i64::MAX as u128 {
            i64::MAX
        } else {
            bit_sum as i64
        };
        self.resources.insert(
            NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY.to_string(),
            Value::String(summary),
        );
        self.resources.insert(
            NUMERIC_FACTOR_ROUTE_TOTAL_RESOURCE_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(total_i64)),
        );
        self.resources.insert(
            NUMERIC_FACTOR_BITS_MIN_RESOURCE_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(bit_min_i64)),
        );
        self.resources.insert(
            NUMERIC_FACTOR_BITS_MAX_RESOURCE_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(bit_max_i64)),
        );
        self.resources.insert(
            NUMERIC_FACTOR_BITS_SUM_RESOURCE_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(bit_sum_i64)),
        );
        self.resources.insert(
            NUMERIC_FACTOR_POLICY_RESOURCE_KEY.to_string(),
            Value::String(policy_summary),
        );
    }

    fn emit_factor_route_summary_diag(&mut self) {
        let Some((summary, total, bit_min, bit_max, bit_sum)) = self.factor_route_summary_components() else {
            return;
        };
        let policy_summary = factor_policy_summary_text();
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let trace_text = format!(
            "routes={summary};total={total};bit_min={bit_min};bit_max={bit_max};bit_sum={bit_sum};policy={policy_summary}"
        );
        let message = format!(
            "곱수 분해경로 집계: {summary} (총 {total}, 비트 최소={bit_min}, 최대={bit_max}, 합={bit_sum}, 정책={policy_summary})"
        );
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: NUMERIC_DIAG_REASON_FACTOR_ROUTE_SUMMARY.to_string(),
            rule_id: NUMERIC_DIAG_RULE_ID_FACTOR_ROUTE_SUMMARY.to_string(),
            reason: NUMERIC_DIAG_REASON_FACTOR_ROUTE_SUMMARY.to_string(),
            sub_reason: None,
            mode: Some("알림".to_string()),
            contract_kind: None,
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span: None,
            expr: Some(ExprTrace {
                tag: NUMERIC_DIAG_TAG_FACTOR_ROUTE_SUMMARY.to_string(),
                text: Some(trace_text),
            }),
            message: Some(message),
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
    }

    fn factor_route_summary_components(&self) -> Option<(String, u64, u64, u64, u128)> {
        if self.factor_route_counts.is_empty() {
            return None;
        }
        let summary = self
            .factor_route_counts
            .iter()
            .map(|(route, count)| format!("{route}={count}"))
            .collect::<Vec<_>>()
            .join(",");
        let total = self
            .factor_route_counts
            .values()
            .fold(0u64, |acc, v| acc.saturating_add(*v));
        let bit_min = self.factor_bits_min.unwrap_or(0);
        let bit_max = self.factor_bits_max;
        let bit_sum = self.factor_bits_total;
        Some((summary, total, bit_min, bit_max, bit_sum))
    }

    fn capture_frame_snapshot(&self, locals: &HashMap<String, Value>) -> EvalFrameSnapshot {
        EvalFrameSnapshot {
            locals: locals.clone(),
            resources: self.resources.clone(),
            patch_ops: self.patch_ops.clone(),
            guard_rejected: self.guard_rejected,
            const_scopes: self.const_scopes.clone(),
            factor_route_counts: self.factor_route_counts.clone(),
            factor_bits_total: self.factor_bits_total,
            factor_bits_min: self.factor_bits_min,
            factor_bits_max: self.factor_bits_max,
        }
    }

    fn restore_frame_snapshot(
        &mut self,
        locals: &mut HashMap<String, Value>,
        snapshot: &EvalFrameSnapshot,
    ) {
        *locals = snapshot.locals.clone();
        self.resources = snapshot.resources.clone();
        self.patch_ops = snapshot.patch_ops.clone();
        self.guard_rejected = snapshot.guard_rejected;
        self.const_scopes = snapshot.const_scopes.clone();
        self.factor_route_counts = snapshot.factor_route_counts.clone();
        self.factor_bits_total = snapshot.factor_bits_total;
        self.factor_bits_min = snapshot.factor_bits_min;
        self.factor_bits_max = snapshot.factor_bits_max;
    }

    fn restore_frame_snapshot_preserving_abort_contract_diag(
        &mut self,
        locals: &mut HashMap<String, Value>,
        snapshot: &EvalFrameSnapshot,
    ) {
        let preserved = self.abort_contract_diag_ops_since(snapshot.patch_ops.len());
        self.restore_frame_snapshot(locals, snapshot);
        self.patch_ops.extend(preserved);
    }

    fn abort_contract_diag_ops_since(&self, base_len: usize) -> Vec<PatchOp> {
        self.patch_ops
            .iter()
            .skip(base_len)
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    targets,
                } if event.rule_id == "L0-CONTRACT-01" && event.mode.as_deref() == Some("중단") => {
                    Some(PatchOp::EmitSignal {
                        signal: Signal::Diag {
                            event: event.clone(),
                        },
                        targets: targets.clone(),
                    })
                }
                _ => None,
            })
            .collect()
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

    #[allow(dead_code)]
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
                    self.resources
                        .insert(name.to_string(), Value::Unit(unit_value));
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
                    self.resources
                        .insert(name.to_string(), Value::Fixed64(unit.value));
                } else {
                    return Err("단위가 있는 값은 단위 태그 자원에만 저장할 수 있습니다"
                        .to_string()
                        .into());
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
            Value::List(_)
            | Value::Set(_)
            | Value::Map(_)
            | Value::Pack(_)
            | Value::Assertion(_)
            | Value::StateMachine(_) => {
                if parse_resource_unit_tag(name).is_some() {
                    return Err(
                        "단위 태그 자원에는 차림/모음/짝맞춤/묶음/세움값/상태머신값을 저장할 수 없습니다"
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
            Value::Template(_) => {
                return Err("글무늬는 자원에 대입할 수 없습니다".to_string().into())
            }
            Value::Regex(_) => return Err("정규식은 자원에 대입할 수 없습니다".to_string().into()),
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
        Literal::Regex(regex) => Value::Regex(regex.clone()),
        Literal::Resource(path) => Value::ResourceHandle(ResourceHandle::from_path(path)),
        Literal::None => Value::None,
    }
}

fn build_regex(regex: &RegexLiteral) -> Result<regex::Regex, EvalError> {
    validate_regex_flags(&regex.flags)?;
    let mut builder = RegexBuilder::new(&regex.pattern);
    for ch in regex.flags.chars() {
        match ch {
            'i' => builder.case_insensitive(true),
            'm' => builder.multi_line(true),
            's' => builder.dot_matches_new_line(true),
            _ => {
                return Err(format!(
                    "E_REGEX_FLAGS_INVALID: 지원하지 않는 깃발 '{}'",
                    regex.flags
                )
                .into())
            }
        };
    }
    builder
        .build()
        .map_err(|e| format!("E_REGEX_PATTERN_INVALID: {}", e).into())
}

fn ensure_declared_state(machine: &StateMachine, state: &str) -> Result<(), EvalError> {
    if machine.states.iter().any(|name| name == state) {
        Ok(())
    } else {
        Err(format!("E_STATE_UNDECLARED: {}", state).into())
    }
}

fn build_state_machine_transition_bindings(from: &str, to: &str) -> HashMap<String, Value> {
    let from_value = Value::String(from.to_string());
    let to_value = Value::String(to.to_string());
    HashMap::from([
        ("현재".to_string(), from_value.clone()),
        ("다음".to_string(), to_value.clone()),
        ("현재상태".to_string(), from_value.clone()),
        ("다음상태".to_string(), to_value.clone()),
        ("출발상태".to_string(), from_value),
        ("도착상태".to_string(), to_value),
    ])
}

fn state_machine_transition_action_args(
    seed: &SeedDef,
    bindings: &HashMap<String, Value>,
) -> Result<Vec<Value>, EvalError> {
    let mut args = Vec::with_capacity(seed.params.len());
    for param in &seed.params {
        let value = bindings.get(&param.pin_name).cloned().ok_or_else(|| {
            EvalError::from(format!(
                "E_STATE_TRANSITION_ACTION_ARG_UNRESOLVED: {}:{}",
                seed.canonical_name, param.pin_name
            ))
        })?;
        args.push(value);
    }
    Ok(args)
}

fn assertion_bindings_from_value(value: &Value) -> Result<HashMap<String, Value>, EvalError> {
    match value {
        Value::Pack(pack) => Ok(pack
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()),
        Value::Map(entries) => {
            let mut out = HashMap::new();
            for entry in entries.values() {
                let Value::String(key) = &entry.key else {
                    return Err("살피기 값들은 글 키를 가진 묶음/짝맞춤이어야 합니다"
                        .to_string()
                        .into());
                };
                out.insert(key.clone(), entry.value.clone());
            }
            Ok(out)
        }
        _ => Err("살피기 값들은 묶음 또는 짝맞춤이어야 합니다"
            .to_string()
            .into()),
    }
}

fn parse_assertion_body(assertion: &Assertion) -> Result<Body, EvalError> {
    let wrapped = format!("검사:셈씨 = {{{}}}", assertion.body_source);
    let program = parse_with_mode(&wrapped, "#assertion", ParseMode::Strict).map_err(|err| {
        EvalError::Message(format!(
            "세움 본문 파싱 실패: {}",
            format_parse_error(&wrapped, &err)
        ))
    })?;
    let Some(TopLevelItem::SeedDef(seed)) = program.items.first() else {
        return Err("세움 본문을 찾을 수 없습니다".to_string().into());
    };
    let Some(body) = &seed.body else {
        return Err("세움 본문이 비어 있습니다".to_string().into());
    };
    Ok(body.clone())
}

fn extract_assertion_body_source(canon: &str) -> Option<String> {
    let start = canon.find('{')?;
    let end = canon.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(canon[start + 1..end].to_string())
}

impl<'a> EvalContext<'a> {
    fn resolve_named_assertion(
        &self,
        locals: &HashMap<String, Value>,
        name: &str,
    ) -> Option<Assertion> {
        if let Some(Value::Assertion(assertion)) = locals.get(name) {
            return Some(assertion.clone());
        }
        if let Some(Value::Assertion(assertion)) = self.resources.get(name) {
            return Some(assertion.clone());
        }
        if let Some(Value::Assertion(assertion)) = self.defaults.get(name) {
            return Some(assertion.clone());
        }
        None
    }

    fn eval_state_machine_transition_checks(
        &mut self,
        locals: &HashMap<String, Value>,
        machine: &StateMachine,
        from: &str,
        to: &str,
    ) -> Result<(), EvalError> {
        if machine.on_transition_checks.is_empty() {
            return Ok(());
        }
        let bindings = build_state_machine_transition_bindings(from, to);
        for check_name in &machine.on_transition_checks {
            let assertion = self
                .resolve_named_assertion(locals, check_name)
                .ok_or_else(|| {
                    EvalError::from(format!(
                        "E_STATE_TRANSITION_CHECK_UNRESOLVED: {}",
                        check_name
                    ))
                })?;
            let passed = self.eval_assertion(&assertion, bindings.clone())?;
            match passed {
                Value::Bool(true) => {}
                _ => {
                    return Err(format!("E_STATE_TRANSITION_CHECK_FAILED: {}", check_name).into());
                }
            }
        }
        Ok(())
    }

    fn eval_state_machine_guard(
        &mut self,
        locals: &HashMap<String, Value>,
        guard_name: &str,
        bindings: &HashMap<String, Value>,
    ) -> Result<bool, EvalError> {
        let assertion = self
            .resolve_named_assertion(locals, guard_name)
            .ok_or_else(|| {
                EvalError::from(format!(
                    "E_STATE_TRANSITION_GUARD_UNRESOLVED: {}",
                    guard_name
                ))
            })?;
        Ok(self
            .eval_assertion_result(&assertion, bindings.clone())
            .is_ok())
    }

    fn eval_state_machine_transition_action(
        &mut self,
        action_name: &str,
        bindings: &HashMap<String, Value>,
    ) -> Result<(), EvalError> {
        let seed = self
            .program
            .functions
            .get(action_name)
            .ok_or_else(|| {
                EvalError::from(format!(
                    "E_STATE_TRANSITION_ACTION_UNRESOLVED: {}",
                    action_name
                ))
            })?
            .clone();
        let args = state_machine_transition_action_args(&seed, bindings)?;
        let mut child = EvalContext {
            program: self.program,
            world: self.world,
            defaults: self.defaults,
            resources: self.resources.clone(),
            input: self.input,
            patch_ops: self.patch_ops.clone(),
            guard_rejected: self.guard_rejected,
            aborted: false,
            current_seed_name: self.current_seed_name.clone(),
            rng_state: self.rng_state,
            flow_stack: self.flow_stack.clone(),
            tick_id: self.tick_id,
            const_scopes: self.const_scopes.clone(),
            factor_route_counts: self.factor_route_counts.clone(),
            factor_bits_total: self.factor_bits_total,
            factor_bits_min: self.factor_bits_min,
            factor_bits_max: self.factor_bits_max,
        };
        child.eval_seed(&seed, args)?;
        if child.aborted {
            return Err(EvalError::from(format!(
                "E_STATE_TRANSITION_ACTION_ABORTED: {}",
                action_name
            )));
        }
        self.resources = child.resources;
        self.patch_ops = child.patch_ops;
        self.guard_rejected = child.guard_rejected;
        self.rng_state = child.rng_state;
        self.const_scopes = child.const_scopes;
        self.factor_route_counts = child.factor_route_counts;
        self.factor_bits_total = child.factor_bits_total;
        self.factor_bits_min = child.factor_bits_min;
        self.factor_bits_max = child.factor_bits_max;
        Ok(())
    }

    fn apply_state_machine_transition(
        &mut self,
        locals: &HashMap<String, Value>,
        machine: &StateMachine,
        transition: &StateTransition,
    ) -> Result<(), EvalError> {
        let bindings = build_state_machine_transition_bindings(&transition.from, &transition.to);
        let mut child = EvalContext {
            program: self.program,
            world: self.world,
            defaults: self.defaults,
            resources: self.resources.clone(),
            input: self.input,
            patch_ops: self.patch_ops.clone(),
            guard_rejected: self.guard_rejected,
            aborted: false,
            current_seed_name: self.current_seed_name.clone(),
            rng_state: self.rng_state,
            flow_stack: self.flow_stack.clone(),
            tick_id: self.tick_id,
            const_scopes: self.const_scopes.clone(),
            factor_route_counts: self.factor_route_counts.clone(),
            factor_bits_total: self.factor_bits_total,
            factor_bits_min: self.factor_bits_min,
            factor_bits_max: self.factor_bits_max,
        };
        if let Some(action_name) = &transition.action_name {
            child.eval_state_machine_transition_action(action_name, &bindings)?;
        }
        child.eval_state_machine_transition_checks(
            locals,
            machine,
            &transition.from,
            &transition.to,
        )?;
        self.resources = child.resources;
        self.patch_ops = child.patch_ops;
        self.guard_rejected = child.guard_rejected;
        self.rng_state = child.rng_state;
        self.const_scopes = child.const_scopes;
        self.factor_route_counts = child.factor_route_counts;
        self.factor_bits_total = child.factor_bits_total;
        self.factor_bits_min = child.factor_bits_min;
        self.factor_bits_max = child.factor_bits_max;
        Ok(())
    }

    fn emit_state_machine_transition(
        &mut self,
        from: &str,
        to: &str,
        guard_name: Option<&str>,
        action_name: Option<&str>,
    ) {
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: "STATE_TRANSITION".to_string(),
            rule_id: "L1-STATE-01".to_string(),
            reason: "STATE_TRANSITION".to_string(),
            sub_reason: Some("NEXT".to_string()),
            mode: None,
            contract_kind: None,
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span: None,
            expr: Some(ExprTrace {
                tag: format!("state_transition:{}->{}", from, to),
                text: Some(format!(
                    "guard={};action={}",
                    guard_name.unwrap_or("-"),
                    action_name.unwrap_or("-")
                )),
            }),
            message: Some(format!("상태 전이: {} -> {}", from, to)),
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
    }

    fn eval_assertion_result(
        &mut self,
        assertion: &Assertion,
        bindings: HashMap<String, Value>,
    ) -> Result<(), String> {
        let body = parse_assertion_body(assertion).map_err(|err| err.message())?;
        let mut child = EvalContext {
            program: self.program,
            world: self.world,
            defaults: self.defaults,
            resources: self.resources.clone(),
            input: self.input,
            patch_ops: Vec::new(),
            guard_rejected: false,
            aborted: false,
            current_seed_name: self.current_seed_name.clone(),
            rng_state: self.rng_state,
            flow_stack: self.flow_stack.clone(),
            tick_id: self.tick_id,
            const_scopes: self.const_scopes.clone(),
            factor_route_counts: self.factor_route_counts.clone(),
            factor_bits_total: self.factor_bits_total,
            factor_bits_min: self.factor_bits_min,
            factor_bits_max: self.factor_bits_max,
        };
        let mut locals = bindings;
        match child.eval_body(&mut locals, &body) {
            Ok(_) if !child.aborted => Ok(()),
            Ok(_) => Err("세움 살피기가 실패했습니다".to_string()),
            Err(err) => Err(err.message()),
        }
    }

    fn eval_assertion(
        &mut self,
        assertion: &Assertion,
        bindings: HashMap<String, Value>,
    ) -> Result<Value, EvalError> {
        match self.eval_assertion_result(assertion, bindings) {
            Ok(()) => Ok(Value::Bool(true)),
            Err(message) => {
                self.emit_assertion_failure(assertion, message);
                Ok(Value::Bool(false))
            }
        }
    }

    fn emit_assertion_failure(&mut self, assertion: &Assertion, message: String) {
        let origin = self
            .current_seed_name
            .as_ref()
            .map(|name| format!("seed:{}", name))
            .unwrap_or_else(|| "#system:ddn".to_string());
        let event = DiagEvent {
            madi: self.tick_id,
            seq: 0,
            fault_id: "ASSERTION_FAILED".to_string(),
            rule_id: "L1-ASSERT-01".to_string(),
            reason: "ASSERTION_FAILED".to_string(),
            sub_reason: Some("CHECK_FAILED".to_string()),
            mode: None,
            contract_kind: None,
            origin: origin.clone(),
            targets: vec![origin],
            sam_hash: None,
            source_span: None,
            expr: Some(ExprTrace {
                tag: "assertion:check".to_string(),
                text: Some(assertion.canon.clone()),
            }),
            message: Some(message),
        };
        self.patch_ops.push(PatchOp::EmitSignal {
            signal: Signal::Diag { event },
            targets: Vec::new(),
        });
    }
}

fn build_regex_full_match(regex: &RegexLiteral) -> Result<regex::Regex, EvalError> {
    validate_regex_flags(&regex.flags)?;
    let wrapped = format!(r"\A(?:{})\z", regex.pattern);
    let mut builder = RegexBuilder::new(&wrapped);
    for ch in regex.flags.chars() {
        match ch {
            'i' => builder.case_insensitive(true),
            'm' => builder.multi_line(true),
            's' => builder.dot_matches_new_line(true),
            _ => {
                return Err(format!(
                    "E_REGEX_FLAGS_INVALID: 지원하지 않는 깃발 '{}'",
                    regex.flags
                )
                .into())
            }
        };
    }
    builder
        .build()
        .map_err(|e| format!("E_REGEX_PATTERN_INVALID: {}", e).into())
}

fn validate_regex_flags(flags: &str) -> Result<(), EvalError> {
    let mut seen_i = false;
    let mut seen_m = false;
    let mut seen_s = false;
    for ch in flags.chars() {
        match ch {
            'i' if !seen_i => seen_i = true,
            'm' if !seen_m => seen_m = true,
            's' if !seen_s => seen_s = true,
            _ => {
                return Err(format!("E_REGEX_FLAGS_INVALID: 지원하지 않는 깃발 '{}'", flags).into())
            }
        }
    }
    Ok(())
}

fn regex_is_full_match(text: &str, regex: &RegexLiteral) -> Result<bool, EvalError> {
    let re = build_regex_full_match(regex)?;
    Ok(re.is_match(text))
}

fn regex_find_first(text: &str, regex: &RegexLiteral) -> Result<Option<String>, EvalError> {
    let re = build_regex(regex)?;
    Ok(re.find(text).map(|m| m.as_str().to_string()))
}

fn regex_capture_first(text: &str, regex: &RegexLiteral) -> Result<Vec<String>, EvalError> {
    let re = build_regex(regex)?;
    let Some(captures) = re.captures(text) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::with_capacity(captures.len());
    for idx in 0..captures.len() {
        if let Some(hit) = captures.get(idx) {
            out.push(hit.as_str().to_string());
        } else {
            out.push(String::new());
        }
    }
    Ok(out)
}

fn regex_named_capture_first(
    text: &str,
    regex: &RegexLiteral,
) -> Result<BTreeMap<String, MapEntry>, EvalError> {
    let re = build_regex(regex)?;
    let Some(captures) = re.captures(text) else {
        return Ok(BTreeMap::new());
    };
    let mut out = BTreeMap::new();
    for name in re.capture_names().flatten() {
        let key = Value::String(name.to_string());
        let value = Value::String(
            captures
                .name(name)
                .map(|item| item.as_str().to_string())
                .unwrap_or_default(),
        );
        out.insert(map_key_canon(&key), MapEntry { key, value });
    }
    Ok(out)
}

fn regex_replacement_ref_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn validate_regex_replacement(re: &regex::Regex, replacement: &str) -> Result<(), EvalError> {
    let names: BTreeSet<&str> = re.capture_names().flatten().collect();
    let chars: Vec<char> = replacement.chars().collect();
    let mut idx = 0usize;
    while idx < chars.len() {
        if chars[idx] != '$' {
            idx += 1;
            continue;
        }
        if idx + 1 >= chars.len() {
            return Err(
                "E_REGEX_REPLACEMENT_INVALID: 치환 참조가 '$'에서 끝났습니다"
                    .to_string()
                    .into(),
            );
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
                return Err(
                    "E_REGEX_REPLACEMENT_INVALID: 치환 참조의 닫는 '}'가 없습니다"
                        .to_string()
                        .into(),
                );
            }
            let token: String = chars[idx + 2..end].iter().collect();
            if token.is_empty() {
                return Err(
                    "E_REGEX_REPLACEMENT_INVALID: 빈 치환 참조는 사용할 수 없습니다"
                        .to_string()
                        .into(),
                );
            }
            if token.chars().all(|ch| ch.is_ascii_digit()) {
                let capture_index = token.parse::<usize>().unwrap_or(usize::MAX);
                if capture_index >= re.captures_len() {
                    return Err(format!(
                        "E_REGEX_REPLACEMENT_INVALID: 알 수 없는 캡처 번호입니다: {}",
                        token
                    )
                    .into());
                }
            } else if !names.contains(token.as_str()) {
                return Err(format!(
                    "E_REGEX_REPLACEMENT_INVALID: 알 수 없는 이름 캡처입니다: {}",
                    token
                )
                .into());
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
            if capture_index >= re.captures_len() {
                return Err(format!(
                    "E_REGEX_REPLACEMENT_INVALID: 알 수 없는 캡처 번호입니다: {}",
                    token
                )
                .into());
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
                return Err(format!(
                    "E_REGEX_REPLACEMENT_INVALID: 알 수 없는 이름 캡처입니다: {}",
                    token
                )
                .into());
            }
            idx = end;
            continue;
        }
        return Err(format!(
            "E_REGEX_REPLACEMENT_INVALID: 지원하지 않는 치환 참조 시작입니다: ${}",
            next
        )
        .into());
    }
    Ok(())
}

fn regex_replace_all(
    text: &str,
    regex: &RegexLiteral,
    replacement: &str,
) -> Result<String, EvalError> {
    let re = build_regex(regex)?;
    validate_regex_replacement(&re, replacement)?;
    Ok(re.replace_all(text, replacement).to_string())
}

fn regex_split(text: &str, regex: &RegexLiteral) -> Result<Vec<String>, EvalError> {
    let re = build_regex(regex)?;
    Ok(re.split(text).map(ToString::to_string).collect())
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Fixed64(a), Value::Fixed64(b)) => a.raw_i64() == b.raw_i64(),
        (Value::Fixed64(a), Value::Unit(b)) if b.is_dimensionless() => {
            a.raw_i64() == b.value.raw_i64()
        }
        (Value::Unit(a), Value::Fixed64(b)) if a.is_dimensionless() => {
            a.value.raw_i64() == b.raw_i64()
        }
        (Value::Unit(a), Value::Unit(b)) => {
            a.dim == b.dim && a.value.raw_i64() == b.value.raw_i64()
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::ResourceHandle(a), Value::ResourceHandle(b)) => a == b,
        (Value::None, Value::None) => true,
        (Value::Pack(a), Value::Pack(b)) => a == b,
        (Value::Assertion(a), Value::Assertion(b)) => a.canon == b.canon,
        (Value::StateMachine(a), Value::StateMachine(b)) => a == b,
        (Value::Formula(a), Value::Formula(b)) => a == b,
        (Value::Template(a), Value::Template(b)) => a.raw == b.raw && a.tag == b.tag,
        (Value::Regex(a), Value::Regex(b)) => a == b,
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
        Value::Regex(_) => 7,
        Value::Assertion(_) => 8,
        Value::StateMachine(_) => 9,
        Value::Lambda(_) => 10,
        Value::Pack(_) => {
            if numeric_pack_kind(value).is_some() {
                2
            } else {
                11
            }
        }
        Value::List(_) => 12,
        Value::Set(_) => 13,
        Value::Map(_) => 14,
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
        (Value::Fixed64(a), Value::Unit(b)) if b.is_dimensionless() => {
            a.raw_i64().cmp(&b.value.raw_i64())
        }
        (Value::Unit(a), Value::Fixed64(b)) if a.is_dimensionless() => {
            a.value.raw_i64().cmp(&b.raw_i64())
        }
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::ResourceHandle(a), Value::ResourceHandle(b)) => a.cmp(b),
        (Value::Formula(a), Value::Formula(b)) => a.raw.cmp(&b.raw),
        (Value::Template(a), Value::Template(b)) => a.raw.cmp(&b.raw),
        (Value::Regex(a), Value::Regex(b)) => a
            .pattern
            .cmp(&b.pattern)
            .then_with(|| a.flags.cmp(&b.flags)),
        (Value::Assertion(a), Value::Assertion(b)) => a.canon.cmp(&b.canon),
        (Value::StateMachine(a), Value::StateMachine(b)) => {
            state_machine_canon(a).cmp(&state_machine_canon(b))
        }
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
        Value::Fixed64(v) => Ok(UnitValue {
            value: *v,
            dim: UnitDim::NONE,
        }),
        Value::Unit(unit) => Ok(*unit),
        Value::Pack(_) => {
            let Some(v) = numeric_pack_approx(value) else {
                return Err("수치가 필요합니다".to_string().into());
            };
            Ok(UnitValue {
                value: v,
                dim: UnitDim::NONE,
            })
        }
        _ => Err("수치가 필요합니다".to_string().into()),
    }
}

fn numeric_value(value: &Value) -> Option<UnitValue> {
    match value {
        Value::Fixed64(v) => Some(UnitValue {
            value: *v,
            dim: UnitDim::NONE,
        }),
        Value::Unit(unit) => Some(*unit),
        Value::Pack(_) => numeric_pack_approx(value).map(|v| UnitValue {
            value: v,
            dim: UnitDim::NONE,
        }),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PercentileMode {
    Linear,
    NearestRank,
}

fn expect_numeric_list_arg(args: &[Value], func_name: &str) -> Result<Vec<UnitValue>, EvalError> {
    if args.len() != 1 {
        return Err(format!("{func_name}는 인자 1개를 받습니다")
            .to_string()
            .into());
    }
    let Value::List(items) = &args[0] else {
        return Err(format!("{func_name}는 차림 인자를 받습니다")
            .to_string()
            .into());
    };
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        values.push(unit_value_from_value(item)?);
    }
    if let Some(head) = values.first() {
        for item in values.iter().skip(1) {
            if item.dim != head.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: head.dim,
                    right: item.dim,
                }));
            }
        }
    }
    Ok(values)
}

fn expect_quantile_args(
    args: &[Value],
) -> Result<(Vec<UnitValue>, Fixed64, PercentileMode), EvalError> {
    if args.len() != 2 && args.len() != 3 {
        return Err("분위수는 인자 2개 또는 3개를 받습니다".to_string().into());
    }
    let values = expect_numeric_list_arg(&args[..1], "분위수")?;
    let percentile = unit_value_from_value(&args[1])?;
    if !percentile.is_dimensionless() {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: percentile.dim,
            right: UnitDim::NONE,
        }));
    }
    if percentile.value.raw_i64() < 0 || percentile.value.raw_i64() > Fixed64::ONE.raw_i64() {
        return Err("분위수 p는 0..1 범위여야 합니다".to_string().into());
    }
    let mode = if args.len() == 3 {
        parse_percentile_mode(&args[2])?
    } else {
        PercentileMode::Linear
    };
    Ok((values, percentile.value, mode))
}

fn parse_percentile_mode(value: &Value) -> Result<PercentileMode, EvalError> {
    let Value::String(mode) = value else {
        return Err("분위수 mode는 글이어야 합니다".to_string().into());
    };
    match mode.trim() {
        "선형보간" => Ok(PercentileMode::Linear),
        "최근순위" => Ok(PercentileMode::NearestRank),
        _ => Err("분위수 mode는 선형보간 또는 최근순위여야 합니다"
            .to_string()
            .into()),
    }
}

fn fixed64_to_nonnegative_index(value: Fixed64) -> Result<usize, EvalError> {
    if value.raw_i64() < 0 || value.frac_part() != 0 {
        return Err("분위수 인덱스는 0 이상의 정수여야 합니다"
            .to_string()
            .into());
    }
    Ok(value.int_part() as usize)
}

fn nearest_rank_index(p: Fixed64, len: usize) -> Result<usize, EvalError> {
    if len == 0 {
        return Err("분위수 입력 목록이 비어 있습니다".to_string().into());
    }
    let p_raw = p.raw_i64() as i128;
    let n = len as i128;
    let scale = Fixed64::ONE_RAW as i128;
    let rank = if p_raw <= 0 {
        1
    } else {
        ((p_raw * n) + (scale - 1)) / scale
    };
    let clamped_rank = rank.clamp(1, n);
    Ok((clamped_rank - 1) as usize)
}

struct NumericFormulaPrepared {
    parsed: ParsedFormula,
    var_name: String,
}

fn expect_numeric_derivative_args(
    args: &[Value],
    label: &'static str,
) -> Result<(Formula, String, UnitValue, UnitValue), EvalError> {
    if args.len() != 4 {
        return Err(format!("{label}는 인자 4개를 받습니다").to_string().into());
    }
    let formula = match &args[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(format!("{label}는 수식값 인자가 필요합니다")
                .to_string()
                .into())
        }
    };
    let raw_var = match &args[1] {
        Value::String(text) => text.clone(),
        _ => {
            return Err(format!("{label} 변수 이름이 글이어야 합니다")
                .to_string()
                .into())
        }
    };
    let var_name = raw_var.trim().trim_start_matches('#').to_string();
    if var_name.is_empty() {
        return Err(
            format!("E_CALC_NUMERIC_BAD_VAR: {label} 변수 이름이 비어 있습니다")
                .to_string()
                .into(),
        );
    }
    let point = unit_value_from_value(&args[2])?;
    let step = unit_value_from_value(&args[3])?;
    if point.dim != step.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: point.dim,
            right: step.dim,
        }));
    }
    if step.value.raw_i64() == 0 {
        return Err(
            format!("E_CALC_NUMERIC_BAD_STEP: {label} 스텝은 0이 될 수 없습니다")
                .to_string()
                .into(),
        );
    }
    Ok((formula, var_name, point, step))
}

fn expect_numeric_integral_args(
    args: &[Value],
    label: &'static str,
) -> Result<(Formula, String, UnitValue, UnitValue, UnitValue), EvalError> {
    if args.len() != 5 {
        return Err(format!("{label}는 인자 5개를 받습니다").to_string().into());
    }
    let formula = match &args[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(format!("{label}는 수식값 인자가 필요합니다")
                .to_string()
                .into())
        }
    };
    let raw_var = match &args[1] {
        Value::String(text) => text.clone(),
        _ => {
            return Err(format!("{label} 변수 이름이 글이어야 합니다")
                .to_string()
                .into())
        }
    };
    let var_name = raw_var.trim().trim_start_matches('#').to_string();
    if var_name.is_empty() {
        return Err(
            format!("E_CALC_NUMERIC_BAD_VAR: {label} 변수 이름이 비어 있습니다")
                .to_string()
                .into(),
        );
    }
    let start = unit_value_from_value(&args[2])?;
    let end = unit_value_from_value(&args[3])?;
    let step = unit_value_from_value(&args[4])?;
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
    if step.value.raw_i64() <= 0 {
        return Err(
            format!("E_CALC_NUMERIC_BAD_STEP: {label} 스텝은 0보다 커야 합니다")
                .to_string()
                .into(),
        );
    }
    Ok((formula, var_name, start, end, step))
}

fn prepare_numeric_formula(
    formula: &Formula,
    var_name: &str,
    label: &'static str,
) -> Result<NumericFormulaPrepared, EvalError> {
    if !matches!(formula.dialect, FormulaDialect::Ascii) {
        return Err(format!(
            "E_CALC_NUMERIC_DIALECT_UNSUPPORTED: {label}는 #ascii 수식만 지원합니다"
        )
        .to_string()
        .into());
    }
    ensure_formula_ident(var_name, &formula.dialect, label)?;
    let parsed = parse_formula_value(formula)?;
    if !parsed.vars.contains(var_name) {
        return Err(format!(
            "E_CALC_NUMERIC_FREEVAR_NOT_FOUND: {label} 변수 '{}'가 수식에 없습니다",
            var_name
        )
        .to_string()
        .into());
    }
    if parsed.vars.len() != 1 {
        return Err(
            format!("E_CALC_NUMERIC_FREEVAR_AMBIGUOUS: {label} 변수 이름이 여러 개입니다")
                .to_string()
                .into(),
        );
    }
    Ok(NumericFormulaPrepared {
        parsed,
        var_name: var_name.to_string(),
    })
}

fn eval_numeric_formula_at(
    prepared: &NumericFormulaPrepared,
    x: UnitValue,
    label: &'static str,
) -> Result<UnitValue, EvalError> {
    let mut env = BTreeMap::new();
    env.insert(prepared.var_name.clone(), unit_value_to_value(x));
    eval_formula_expr(&prepared.parsed.expr, &env).map_err(|err| {
        EvalError::Message(format!(
            "E_CALC_NUMERIC_EVAL_FAIL: {label} 수식 평가 실패 ({err})"
        ))
    })
}

fn numeric_trapezoid_integral(
    prepared: &NumericFormulaPrepared,
    start: UnitValue,
    end: UnitValue,
    segments: usize,
) -> Result<UnitValue, EvalError> {
    if segments == 0 {
        return Err("E_CALC_NUMERIC_BAD_STEP: 적분.사다리꼴 구간 수가 0입니다"
            .to_string()
            .into());
    }
    let interval = end.value.saturating_sub(start.value);
    let seg = Fixed64::from_i64(segments as i64);
    let h = interval
        .try_div(seg)
        .map_err(|_| "적분.사다리꼴 계산 중 0으로 나눌 수 없습니다".to_string())?;

    let mut y_dim: Option<UnitDim> = None;
    let mut sum_raw = Fixed64::ZERO;
    let two = Fixed64::from_i64(2);
    for idx in 0..=segments {
        let idx_raw = Fixed64::from_i64(idx as i64);
        let x = UnitValue {
            value: start.value.saturating_add(h.saturating_mul(idx_raw)),
            dim: start.dim,
        };
        let y = eval_numeric_formula_at(prepared, x, "적분.사다리꼴")?;
        if let Some(dim) = y_dim {
            if y.dim != dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: dim,
                    right: y.dim,
                }));
            }
        } else {
            y_dim = Some(y.dim);
        }
        let weight = if idx == 0 || idx == segments {
            Fixed64::from_i64(1)
        } else {
            two
        };
        sum_raw = sum_raw.saturating_add(y.value.saturating_mul(weight));
    }
    let approx = sum_raw
        .saturating_mul(h)
        .try_div(Fixed64::from_i64(2))
        .map_err(|_| "적분.사다리꼴 계산 중 0으로 나눌 수 없습니다".to_string())?;
    Ok(UnitValue {
        value: approx,
        dim: y_dim.unwrap_or(UnitDim::NONE).add(start.dim),
    })
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

fn make_numeric_pack(kind: &str, mut fields: BTreeMap<String, Value>) -> Value {
    fields.insert(
        NUMERIC_PACK_KIND_KEY.to_string(),
        Value::String(kind.to_string()),
    );
    Value::Pack(fields)
}

fn numeric_pack_kind(value: &Value) -> Option<&str> {
    let Value::Pack(fields) = value else {
        return None;
    };
    let Value::String(kind) = fields.get(NUMERIC_PACK_KIND_KEY)? else {
        return None;
    };
    Some(kind.as_str())
}

fn numeric_pack_approx(value: &Value) -> Option<Fixed64> {
    let Value::Pack(fields) = value else {
        return None;
    };
    match fields.get(NUMERIC_PACK_APPROX_KEY) {
        Some(Value::Fixed64(v)) => Some(*v),
        Some(Value::Unit(unit)) if unit.is_dimensionless() => Some(unit.value),
        _ => None,
    }
}

fn value_to_fixed64_coerce(value: &Value) -> Result<Fixed64, EvalError> {
    match value {
        Value::Fixed64(v) => Ok(*v),
        Value::Unit(unit) if unit.is_dimensionless() => Ok(unit.value),
        Value::String(text) => text
            .trim()
            .parse::<f64>()
            .map(Fixed64::from_f64_lossy)
            .map_err(|_| "수치 값이 필요합니다".to_string().into()),
        _ => numeric_pack_approx(value)
            .ok_or_else(|| EvalError::from("수치 값이 필요합니다".to_string())),
    }
}

fn value_to_i64_strict(value: &Value) -> Result<i64, EvalError> {
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
        Value::String(text) => text
            .trim()
            .parse::<i64>()
            .map_err(|_| "정수 값이 필요합니다".to_string().into()),
        Value::Pack(_) => {
            let Some(exact) = exact_numeric_from_value(value)? else {
                return Err("정수 값이 필요합니다".to_string().into());
            };
            if !exact.is_integer() {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            let integer = exact.value.to_integer();
            integer
                .to_i64()
                .ok_or_else(|| EvalError::from("정수 값(i64 범위)이 필요합니다".to_string()))
        }
        _ => Err("정수 값이 필요합니다".to_string().into()),
    }
}

fn parse_bigint_text(raw: &str, context: &str) -> Result<BigInt, EvalError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("{context} 정수 문자열이 비었습니다").to_string().into());
    }
    let (negative, digits) = if let Some(rest) = trimmed.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        (false, rest)
    } else {
        (false, trimmed)
    };
    let normalized = normalize_grouped_integer_digits(digits)
        .ok_or_else(|| EvalError::from(format!("{context} 정수 문자열 형식이 아닙니다")))?;
    if normalized.is_empty() {
        return Err(format!("{context} 정수 문자열 형식이 아닙니다").to_string().into());
    }
    let parsed = BigInt::parse_bytes(normalized.as_bytes(), 10)
        .ok_or_else(|| EvalError::from(format!("{context} 정수 문자열 형식이 아닙니다")))?;
    if negative {
        Ok(-parsed)
    } else {
        Ok(parsed)
    }
}

fn normalize_grouped_integer_digits(digits: &str) -> Option<String> {
    if digits.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(digits.len());
    let mut prev_was_digit = false;
    let mut prev_was_sep = false;
    for ch in digits.chars() {
        if ch.is_ascii_digit() {
            out.push(ch);
            prev_was_digit = true;
            prev_was_sep = false;
            continue;
        }
        if ch == '_' {
            if !prev_was_digit || prev_was_sep {
                return None;
            }
            prev_was_digit = false;
            prev_was_sep = true;
            continue;
        }
        return None;
    }
    if prev_was_sep {
        return None;
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

fn extract_bigint_string(value: &Value) -> Option<String> {
    let Value::Pack(fields) = value else {
        return None;
    };
    if numeric_pack_kind(value) != Some(NUMERIC_KIND_BIG_INT) {
        return None;
    }
    fields.get("값").and_then(|v| match v {
        Value::String(text) => Some(text.clone()),
        _ => None,
    })
}

fn bigint_from_value(value: &Value) -> Result<BigInt, EvalError> {
    match value {
        Value::Fixed64(n) => {
            if n.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(BigInt::from(n.int_part()))
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            if unit.value.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(BigInt::from(unit.value.int_part()))
        }
        Value::String(text) => parse_bigint_text(text, "정수"),
        Value::Pack(_) => {
            if let Some(raw) = extract_bigint_string(value) {
                return parse_bigint_text(&raw, "큰바른수");
            }
            if numeric_pack_kind(value) == Some(NUMERIC_KIND_RATIONAL) {
                let ratio = exact_numeric_from_value(value)?
                    .ok_or_else(|| EvalError::from("정수 값이 필요합니다".to_string()))?;
                if !ratio.value.is_integer() {
                    return Err("정수 값이 필요합니다".to_string().into());
                }
                return Ok(ratio.value.to_integer());
            }
            if let Some(v) = numeric_pack_approx(value) {
                if v.frac_part() == 0 {
                    return Ok(BigInt::from(v.int_part()));
                }
            }
            Err("정수 값이 필요합니다".to_string().into())
        }
        _ => Err("정수 값이 필요합니다".to_string().into()),
    }
}

fn factorize_i64(mut n: i64) -> Vec<(i64, i32)> {
    let mut out = Vec::new();
    if n < 0 {
        n = -n;
    }
    if n < 2 {
        return out;
    }
    let mut p = 2i64;
    while p.saturating_mul(p) <= n {
        let mut exp = 0i32;
        while n % p == 0 {
            n /= p;
            exp += 1;
        }
        if exp > 0 {
            out.push((p, exp));
        }
        p = if p == 2 { 3 } else { p + 2 };
    }
    if n > 1 {
        out.push((n, 1));
    }
    out
}

fn bigint_bit_len(value: &BigInt) -> usize {
    if value.is_zero() {
        0
    } else {
        value.to_str_radix(2).trim_start_matches('-').len()
    }
}

fn is_probable_prime_bigint(n: &BigInt) -> bool {
    let one = BigInt::from(1u8);
    let two = BigInt::from(2u8);
    let three = BigInt::from(3u8);

    if n < &two {
        return false;
    }
    if n == &two || n == &three {
        return true;
    }
    if (n % &two).is_zero() {
        return false;
    }

    for prime in FACTOR_SMALL_PRIMES {
        let p = BigInt::from(prime);
        if n == &p {
            return true;
        }
        if (n % &p).is_zero() {
            return false;
        }
    }

    let n_minus_one = n - &one;
    let mut d = n_minus_one.clone();
    let mut s = 0u32;
    while (&d % &two).is_zero() {
        d /= &two;
        s += 1;
    }

    let bases = [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    'witness: for base in bases {
        let a = BigInt::from(base);
        if &a >= n {
            continue;
        }
        let mut x = a.modpow(&d, n);
        if x == one || x == n_minus_one {
            continue;
        }
        for _ in 1..s {
            x = (&x * &x) % n;
            if x == n_minus_one {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

fn pollard_rho_step(x: &BigInt, c: &BigInt, modulus: &BigInt) -> BigInt {
    ((x * x) + c) % modulus
}

fn factor_pollard_max_iters() -> usize {
    #[cfg(test)]
    {
        let override_iters = FACTOR_POLLARD_MAX_ITERS_OVERRIDE.with(|cell| cell.get());
        if let Some(value) = override_iters {
            return value;
        }
    }
    FACTOR_POLLARD_MAX_ITERS
}

#[cfg(test)]
fn swap_factor_pollard_max_iters_for_test(value: usize) -> usize {
    FACTOR_POLLARD_MAX_ITERS_OVERRIDE.with(|cell| {
        let previous = cell.get().unwrap_or(usize::MAX);
        if value == usize::MAX {
            cell.set(None);
        } else {
            cell.set(Some(value));
        }
        previous
    })
}

fn factor_trial_fallback_limit() -> u64 {
    #[cfg(test)]
    {
        let override_limit = FACTOR_TRIAL_FALLBACK_LIMIT_OVERRIDE.with(|cell| cell.get());
        if let Some(value) = override_limit {
            return value;
        }
    }
    FACTOR_TRIAL_FALLBACK_LIMIT
}

fn factor_small_prime_max() -> u64 {
    FACTOR_SMALL_PRIMES.last().copied().unwrap_or(0)
}

fn factor_policy_summary_text() -> String {
    format!(
        "bit_limit={};pollard_iters={};pollard_c_seeds={};pollard_x0_seeds={};fallback_limit={};small_prime_max={}",
        FACTOR_BIGINT_FACTOR_BITS_LIMIT,
        factor_pollard_max_iters(),
        FACTOR_POLLARD_C_SEED_LIMIT,
        FACTOR_POLLARD_X0_SEEDS.len(),
        factor_trial_fallback_limit(),
        factor_small_prime_max()
    )
}

#[cfg(test)]
fn swap_factor_trial_fallback_limit_for_test(value: u64) -> u64 {
    FACTOR_TRIAL_FALLBACK_LIMIT_OVERRIDE.with(|cell| {
        let previous = cell.get().unwrap_or(u64::MAX);
        if value == u64::MAX {
            cell.set(None);
        } else {
            cell.set(Some(value));
        }
        previous
    })
}

fn fallback_trial_division_factor(n: &BigInt) -> Option<BigInt> {
    let mut candidate = 3u64;
    let limit = factor_trial_fallback_limit();
    while candidate <= limit {
        let factor = BigInt::from(candidate);
        if (&factor * &factor) > *n {
            break;
        }
        if (n % &factor).is_zero() {
            return Some(factor);
        }
        candidate += 2;
    }
    None
}

fn pollard_rho_factor(n: &BigInt) -> Option<BigInt> {
    let one = BigInt::from(1u8);
    let two = BigInt::from(2u8);

    if (n % &two).is_zero() {
        return Some(two);
    }
    if is_probable_prime_bigint(n) {
        return Some(n.clone());
    }

    for x0_seed in FACTOR_POLLARD_X0_SEEDS {
        for c_seed in 1u64..=FACTOR_POLLARD_C_SEED_LIMIT {
            let c = BigInt::from(c_seed);
            let mut x = BigInt::from(x0_seed);
            let mut y = BigInt::from(x0_seed);

            for _ in 0..factor_pollard_max_iters() {
                x = pollard_rho_step(&x, &c, n);
                y = pollard_rho_step(&y, &c, n);
                y = pollard_rho_step(&y, &c, n);
                let delta = if x >= y { &x - &y } else { &y - &x };
                let d = delta.gcd(n);
                if d == one {
                    continue;
                }
                if &d == n {
                    break;
                }
                return Some(d);
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug, Default)]
struct FactorizationStats {
    used_small_prime: bool,
    used_pollard: bool,
    used_fallback: bool,
}

fn factor_route_for_bigint_done(stats: &FactorizationStats) -> &'static str {
    if stats.used_fallback && (stats.used_pollard || stats.used_small_prime) {
        return FACTOR_DECOMP_ROUTE_BIGINT_MIXED;
    }
    if stats.used_pollard && stats.used_small_prime {
        return FACTOR_DECOMP_ROUTE_BIGINT_MIXED;
    }
    if stats.used_fallback {
        return FACTOR_DECOMP_ROUTE_BIGINT_FALLBACK;
    }
    if stats.used_pollard {
        return FACTOR_DECOMP_ROUTE_BIGINT_POLLARD;
    }
    if stats.used_small_prime {
        return FACTOR_DECOMP_ROUTE_BIGINT_SMALL_PRIME;
    }
    FACTOR_DECOMP_ROUTE_BIGINT
}

fn factor_route_for_deferred(reason: &str) -> &'static str {
    match reason {
        FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT => FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT,
        FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED => FACTOR_DECOMP_ROUTE_DEFERRED_FACTOR_FAILED,
        _ => FACTOR_DECOMP_ROUTE_DEFERRED_FACTOR_FAILED,
    }
}

fn factorize_bigint_collect(
    n: BigInt,
    out: &mut Vec<BigInt>,
    stats: &mut FactorizationStats,
) -> bool {
    let one = BigInt::from(1u8);
    let two = BigInt::from(2u8);

    if n == one {
        return true;
    }
    if is_probable_prime_bigint(&n) {
        out.push(n);
        return true;
    }
    if (&n % &two).is_zero() {
        out.push(two.clone());
        return factorize_bigint_collect(n / &two, out, stats);
    }
    for prime in FACTOR_SMALL_PRIMES {
        let p = BigInt::from(prime);
        if (&n % &p).is_zero() {
            stats.used_small_prime = true;
            out.push(p.clone());
            return factorize_bigint_collect(n / &p, out, stats);
        }
    }

    let divisor = match pollard_rho_factor(&n) {
        Some(found) => {
            stats.used_pollard = true;
            found
        }
        None => match fallback_trial_division_factor(&n) {
            Some(found) => {
                stats.used_fallback = true;
                found
            }
            None => return false,
        },
    };
    if divisor == one || divisor == n {
        return false;
    }

    let quotient = &n / &divisor;
    factorize_bigint_collect(divisor, out, stats) && factorize_bigint_collect(quotient, out, stats)
}

fn factorize_bigint_full(n: BigInt) -> Result<(Vec<(BigInt, u32)>, FactorizationStats), &'static str> {
    let two = BigInt::from(2u8);
    if n < two {
        return Ok((Vec::new(), FactorizationStats::default()));
    }
    if bigint_bit_len(&n) > FACTOR_BIGINT_FACTOR_BITS_LIMIT {
        return Err(FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT);
    }

    let mut primes = Vec::new();
    let mut stats = FactorizationStats::default();
    if !factorize_bigint_collect(n, &mut primes, &mut stats) {
        return Err(FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED);
    }
    primes.sort();

    let mut out: Vec<(BigInt, u32)> = Vec::new();
    for prime in primes {
        if let Some((last_prime, exp)) = out.last_mut() {
            if *last_prime == prime {
                *exp += 1;
                continue;
            }
        }
        out.push((prime, 1));
    }
    Ok((out, stats))
}

fn factor_canon(sign: i8, factors: &[(i64, i32)]) -> String {
    if factors.is_empty() {
        return if sign < 0 { "-1".to_string() } else { "1".to_string() };
    }
    let body = factors
        .iter()
        .map(|(prime, exp)| {
            if *exp == 1 {
                prime.to_string()
            } else {
                format!("{prime}^{exp}")
            }
        })
        .collect::<Vec<_>>()
        .join(" * ");
    if sign < 0 { format!("-{body}") } else { body }
}

fn factor_canon_bigint(sign: i8, factors: &[(BigInt, u32)]) -> String {
    if factors.is_empty() {
        return if sign < 0 {
            "-1".to_string()
        } else {
            "1".to_string()
        };
    }
    let body = factors
        .iter()
        .map(|(prime, exp)| {
            if *exp == 1 {
                prime.to_string()
            } else {
                format!("{prime}^{exp}")
            }
        })
        .collect::<Vec<_>>()
        .join(" * ");
    if sign < 0 { format!("-{body}") } else { body }
}

fn make_big_int_value(value: &Value) -> Result<Value, EvalError> {
    let parsed = bigint_from_value(value)?;
    let text = parsed.to_string();
    let mut fields = BTreeMap::new();
    fields.insert("값".to_string(), Value::String(text));
    if let Some(v) = parsed.to_i64() {
        fields.insert(
            NUMERIC_PACK_APPROX_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(v)),
        );
    }
    Ok(make_numeric_pack(NUMERIC_KIND_BIG_INT, fields))
}

fn make_rational_value(numerator: &Value, denominator: &Value) -> Result<Value, EvalError> {
    let n = bigint_from_value(numerator)?;
    let d = bigint_from_value(denominator)?;
    if d.is_zero() {
        return Err("나눔수의 분모는 0일 수 없습니다".to_string().into());
    }
    let ratio = BigRational::new(n, d);
    Ok(make_rational_pack_from_big_rational(&ratio))
}

fn make_factor_value(value: &Value) -> Result<(Value, &'static str), EvalError> {
    let parsed = bigint_from_value(value)?;
    let sign: i8 = match parsed.sign() {
        Sign::Minus => -1,
        Sign::NoSign => 0,
        Sign::Plus => 1,
    };
    let (canon, status, deferred_reason, route, bit_len) = if parsed.is_zero() {
        (
            "0".to_string(),
            FACTOR_DECOMP_STATUS_DONE,
            None,
            FACTOR_DECOMP_ROUTE_ZERO,
            0usize,
        )
    } else if let Some(integer) = parsed.to_i64() {
        if integer == i64::MIN {
            let abs = -parsed.clone();
            let bit_len = bigint_bit_len(&abs);
            match factorize_bigint_full(abs) {
                Ok((factors, stats)) => (
                    factor_canon_bigint(sign, &factors),
                    FACTOR_DECOMP_STATUS_DONE,
                    None,
                    factor_route_for_bigint_done(&stats),
                    bit_len,
                ),
                Err(reason) => (
                    parsed.to_string(),
                    FACTOR_DECOMP_STATUS_DEFERRED,
                    Some(reason),
                    factor_route_for_deferred(reason),
                    bit_len,
                ),
            }
        } else {
            let factors = factorize_i64(integer);
            let abs = integer.unsigned_abs();
            let bit_len = if abs == 0 {
                0usize
            } else {
                (u64::BITS - abs.leading_zeros()) as usize
            };
            (
                factor_canon(sign, &factors),
                FACTOR_DECOMP_STATUS_DONE,
                None,
                FACTOR_DECOMP_ROUTE_I64,
                bit_len,
            )
        }
    } else {
        let abs = if sign < 0 {
            -parsed.clone()
        } else {
            parsed.clone()
        };
        let bit_len = bigint_bit_len(&abs);
        match factorize_bigint_full(abs) {
            Ok((factors, stats)) => (
                factor_canon_bigint(sign, &factors),
                FACTOR_DECOMP_STATUS_DONE,
                None,
                factor_route_for_bigint_done(&stats),
                bit_len,
            ),
            // bit 제한 또는 분해 실패 시에는 표현 안정성을 위해 지연으로 둔다.
            Err(reason) => (
                parsed.to_string(),
                FACTOR_DECOMP_STATUS_DEFERRED,
                Some(reason),
                factor_route_for_deferred(reason),
                bit_len,
            ),
        }
    };
    let mut fields = BTreeMap::new();
    fields.insert("값".to_string(), Value::String(parsed.to_string()));
    fields.insert("표기".to_string(), Value::String(canon));
    fields.insert(
        FACTOR_DECOMP_STATUS_KEY.to_string(),
        Value::String(status.to_string()),
    );
    fields.insert(
        "부호".to_string(),
        Value::Fixed64(Fixed64::from_i64(sign as i64)),
    );
    fields.insert(
        FACTOR_DECOMP_ROUTE_KEY.to_string(),
        Value::String(route.to_string()),
    );
    fields.insert(
        FACTOR_DECOMP_BITS_KEY.to_string(),
        Value::Fixed64(Fixed64::from_i64(bit_len as i64)),
    );
    if let Some(reason) = deferred_reason {
        fields.insert(
            FACTOR_DECOMP_DEFERRED_REASON_KEY.to_string(),
            Value::String(reason.to_string()),
        );
    }
    if let Some(v) = parsed.to_i64() {
        fields.insert(
            NUMERIC_PACK_APPROX_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(v)),
        );
    }
    Ok((make_numeric_pack(NUMERIC_KIND_FACTOR, fields), status))
}

#[derive(Clone, Debug)]
struct ExactNumeric {
    value: BigRational,
}

impl ExactNumeric {
    fn from_bigint(value: BigInt) -> Self {
        Self {
            value: BigRational::from_integer(value),
        }
    }

    fn from_fixed64(value: Fixed64) -> Self {
        let num = BigInt::from(value.raw_i64());
        let den = BigInt::from(1_i64) << Fixed64::FRAC_BITS;
        Self {
            value: BigRational::new(num, den),
        }
    }

    fn is_integer(&self) -> bool {
        self.value.is_integer()
    }
}

fn eval_exact_numeric_infix(
    op: &str,
    left: &Value,
    right: &Value,
) -> Result<Option<Value>, EvalError> {
    if !is_exact_numeric_family_value(left) && !is_exact_numeric_family_value(right) {
        return Ok(None);
    }
    let Some(left_exact) = exact_numeric_from_value(left)? else {
        return Ok(None);
    };
    let Some(right_exact) = exact_numeric_from_value(right)? else {
        return Ok(None);
    };

    let out = match op {
        "+" => exact_add(&left_exact, &right_exact),
        "-" => exact_sub(&left_exact, &right_exact),
        "*" => exact_mul(&left_exact, &right_exact),
        "/" => exact_div(&left_exact, &right_exact)?,
        "%" => exact_rem(&left_exact, &right_exact)?,
        _ => return Ok(None),
    };
    let prefer_rational = op == "/" || !left_exact.is_integer() || !right_exact.is_integer();
    Ok(Some(exact_numeric_to_value(out, prefer_rational)))
}

fn eval_exact_numeric_compare_infix(
    op: &str,
    left: &Value,
    right: &Value,
) -> Result<Option<bool>, EvalError> {
    if !is_exact_numeric_family_value(left) && !is_exact_numeric_family_value(right) {
        return Ok(None);
    }
    let Some(left_exact) = exact_numeric_from_value(left)? else {
        return Ok(None);
    };
    let Some(right_exact) = exact_numeric_from_value(right)? else {
        return Ok(None);
    };
    let ord = exact_cmp(&left_exact, &right_exact);
    let out = match op {
        "==" => ord == std::cmp::Ordering::Equal,
        "!=" => ord != std::cmp::Ordering::Equal,
        "<" => ord == std::cmp::Ordering::Less,
        "<=" => ord != std::cmp::Ordering::Greater,
        ">" => ord == std::cmp::Ordering::Greater,
        ">=" => ord != std::cmp::Ordering::Less,
        _ => return Ok(None),
    };
    Ok(Some(out))
}

fn is_exact_numeric_family_value(value: &Value) -> bool {
    matches!(
        numeric_pack_kind(value),
        Some(NUMERIC_KIND_BIG_INT | NUMERIC_KIND_RATIONAL | NUMERIC_KIND_FACTOR)
    )
}

fn exact_numeric_from_value(value: &Value) -> Result<Option<ExactNumeric>, EvalError> {
    match value {
        Value::Fixed64(n) => Ok(Some(ExactNumeric::from_fixed64(*n))),
        Value::Unit(unit) if unit.is_dimensionless() => Ok(Some(ExactNumeric::from_fixed64(unit.value))),
        Value::Pack(fields) => match numeric_pack_kind(value) {
            Some(NUMERIC_KIND_BIG_INT) => {
                let raw = fields
                    .get("값")
                    .and_then(|v| match v {
                        Value::String(text) => Some(text.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| EvalError::from("큰바른수.값이 없습니다".to_string()))?;
                let parsed = parse_bigint_text(raw, "큰바른수")?;
                Ok(Some(ExactNumeric::from_bigint(parsed)))
            }
            Some(NUMERIC_KIND_RATIONAL) => {
                let num_raw = fields
                    .get("분자")
                    .and_then(|v| match v {
                        Value::String(text) => Some(text.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| EvalError::from("나눔수.분자가 없습니다".to_string()))?;
                let den_raw = fields
                    .get("분모")
                    .and_then(|v| match v {
                        Value::String(text) => Some(text.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| EvalError::from("나눔수.분모가 없습니다".to_string()))?;
                let num = parse_bigint_text(num_raw, "나눔수 분자")?;
                let den = parse_bigint_text(den_raw, "나눔수 분모")?;
                if den.is_zero() {
                    return Err("나눔수의 분모는 0일 수 없습니다".to_string().into());
                }
                Ok(Some(ExactNumeric {
                    value: BigRational::new(num, den),
                }))
            }
            Some(NUMERIC_KIND_FACTOR) => {
                if let Some(raw) = fields.get("값").and_then(|v| match v {
                    Value::String(text) => Some(text.as_str()),
                    _ => None,
                }) {
                    let parsed = parse_bigint_text(raw, "곱수 값")?;
                    return Ok(Some(ExactNumeric::from_bigint(parsed)));
                }
                let approx = numeric_pack_approx(value)
                    .ok_or_else(|| EvalError::from("곱수 근사값이 없습니다".to_string()))?;
                if approx.frac_part() != 0 {
                    return Err("곱수 근사값은 정수여야 합니다".to_string().into());
                }
                Ok(Some(ExactNumeric::from_bigint(BigInt::from(approx.int_part()))))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn exact_add(left: &ExactNumeric, right: &ExactNumeric) -> ExactNumeric {
    ExactNumeric {
        value: left.value.clone() + right.value.clone(),
    }
}

fn exact_sub(left: &ExactNumeric, right: &ExactNumeric) -> ExactNumeric {
    ExactNumeric {
        value: left.value.clone() - right.value.clone(),
    }
}

fn exact_mul(left: &ExactNumeric, right: &ExactNumeric) -> ExactNumeric {
    ExactNumeric {
        value: left.value.clone() * right.value.clone(),
    }
}

fn exact_div(left: &ExactNumeric, right: &ExactNumeric) -> Result<ExactNumeric, EvalError> {
    if right.value.is_zero() {
        return Err(unit_error(UnitError::DivisionByZero));
    }
    Ok(ExactNumeric {
        value: left.value.clone() / right.value.clone(),
    })
}

fn exact_rem(left: &ExactNumeric, right: &ExactNumeric) -> Result<ExactNumeric, EvalError> {
    if !left.is_integer() || !right.is_integer() {
        return Err("나머지 연산은 정수형(바른수/큰바른수/곱수)만 지원합니다"
            .to_string()
            .into());
    }
    let l = left.value.to_integer();
    let r = right.value.to_integer();
    if r.is_zero() {
        return Err(unit_error(UnitError::DivisionByZero));
    }
    Ok(ExactNumeric::from_bigint(l % r))
}

fn exact_cmp(left: &ExactNumeric, right: &ExactNumeric) -> std::cmp::Ordering {
    left.value.cmp(&right.value)
}

fn make_big_int_pack_from_bigint(value: &BigInt) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("값".to_string(), Value::String(value.to_string()));
    if let Some(v) = value.to_i64() {
        fields.insert(
            NUMERIC_PACK_APPROX_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(v)),
        );
    }
    make_numeric_pack(NUMERIC_KIND_BIG_INT, fields)
}

fn make_rational_pack_from_big_rational(value: &BigRational) -> Value {
    let num = value.numer();
    let den = value.denom();
    let mut fields = BTreeMap::new();
    fields.insert("분자".to_string(), Value::String(num.to_string()));
    fields.insert("분모".to_string(), Value::String(den.to_string()));
    if let (Some(n), Some(d)) = (num.to_i64(), den.to_i64()) {
        fields.insert(
            NUMERIC_PACK_APPROX_KEY.to_string(),
            Value::Fixed64(Fixed64::from_i64(n) / Fixed64::from_i64(d)),
        );
    }
    make_numeric_pack(NUMERIC_KIND_RATIONAL, fields)
}

fn exact_numeric_to_value(value: ExactNumeric, prefer_rational: bool) -> Value {
    if prefer_rational || !value.is_integer() {
        make_rational_pack_from_big_rational(&value.value)
    } else {
        let integer = value.value.to_integer();
        make_big_int_pack_from_bigint(&integer)
    }
}

fn value_to_i64(value: &Value) -> Result<i64, EvalError> {
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
        Value::Pack(_) => {
            let Some(exact) = exact_numeric_from_value(value)? else {
                return Err("정수 값이 필요합니다".to_string().into());
            };
            if !exact.is_integer() {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            let integer = exact.value.to_integer();
            integer
                .to_i64()
                .ok_or_else(|| EvalError::from("정수 값(i64 범위)이 필요합니다".to_string()))
        }
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

const STREAM_V1_SCHEMA: &str = "ddn.stream.v1";
const STREAM_CAPACITY_MAX: usize = 1_000_000;
const STREAM_VALUE_LABEL: &str = "흐름(흐름.만들기 결과)";

#[derive(Clone)]
struct RuntimeStream {
    capacity: usize,
    head: usize,
    len: usize,
    buffer: Vec<Value>,
}

fn stream_map_lookup<'a>(entries: &'a BTreeMap<String, MapEntry>, key: &str) -> Option<&'a Value> {
    let key_canon = map_key_canon(&Value::String(key.to_string()));
    entries.get(&key_canon).map(|entry| &entry.value)
}

fn stream_parse_u64(value: &Value, field: &str) -> Result<u64, EvalError> {
    match value {
        Value::Fixed64(n) => {
            if n.frac_part() != 0 {
                return Err(format!("흐름.{}는 정수여야 합니다", field).into());
            }
            let v = n.int_part();
            if v < 0 {
                return Err(format!("흐름.{}는 0 이상이어야 합니다", field).into());
            }
            Ok(v as u64)
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            if unit.value.frac_part() != 0 {
                return Err(format!("흐름.{}는 정수여야 합니다", field).into());
            }
            let v = unit.value.int_part();
            if v < 0 {
                return Err(format!("흐름.{}는 0 이상이어야 합니다", field).into());
            }
            Ok(v as u64)
        }
        Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
        Value::String(v) => v
            .parse::<u64>()
            .map_err(|_| format!("흐름.{}는 정수 문자열이어야 합니다", field).into()),
        _ => Err(format!("흐름.{}는 정수 값이어야 합니다", field).into()),
    }
}

fn stream_from_value(value: &Value) -> Result<RuntimeStream, EvalError> {
    let Value::Map(entries) = value else {
        return Err(format!("{} 인자가 필요합니다", STREAM_VALUE_LABEL).into());
    };
    let Some(Value::String(schema)) = stream_map_lookup(entries, "__schema") else {
        return Err(format!("{}에 __schema가 없습니다", STREAM_VALUE_LABEL).into());
    };
    if schema != STREAM_V1_SCHEMA {
        return Err(
            format!(
                "지원하지 않는 흐름 schema: {} (기대값: {})",
                schema, STREAM_V1_SCHEMA
            )
            .into(),
        );
    }
    let capacity_raw = match stream_map_lookup(entries, "capacity") {
        Some(value) => stream_parse_u64(value, "용량")?,
        None => 0,
    };
    let head_raw = match stream_map_lookup(entries, "head") {
        Some(value) => Some(stream_parse_u64(value, "머리")?),
        None => None,
    };
    let Some(Value::List(buffer_raw)) = stream_map_lookup(entries, "buffer") else {
        return Err(format!("{}에 buffer 차림이 없습니다", STREAM_VALUE_LABEL).into());
    };
    let len_raw = match stream_map_lookup(entries, "len") {
        Some(value) => stream_parse_u64(value, "길이")?,
        None => buffer_raw.len() as u64,
    };
    let mut capacity_u64 = capacity_raw.max(buffer_raw.len() as u64);
    if capacity_u64 > STREAM_CAPACITY_MAX as u64 {
        return Err(format!(
            "흐름 용량이 너무 큽니다 (최대 {}): {}",
            STREAM_CAPACITY_MAX, capacity_u64
        )
        .into());
    }
    let capacity = capacity_u64 as usize;
    let mut buffer = buffer_raw.clone();
    if buffer.len() < capacity {
        buffer.resize(capacity, Value::None);
    } else if buffer.len() > capacity {
        buffer.truncate(capacity);
        capacity_u64 = capacity as u64;
    }
    let len = len_raw.min(capacity_u64) as usize;
    let head = if capacity == 0 {
        0
    } else {
        let default_head = len.saturating_sub(1) as u64;
        head_raw
            .unwrap_or(default_head)
            .min(capacity_u64.saturating_sub(1)) as usize
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
    let insert = |entries: &mut BTreeMap<String, MapEntry>, key: &str, value: Value| {
        let key_value = Value::String(key.to_string());
        entries.insert(
            map_key_canon(&key_value),
            MapEntry {
                key: key_value,
                value,
            },
        );
    };
    insert(
        &mut entries,
        "__schema",
        Value::String(STREAM_V1_SCHEMA.to_string()),
    );
    insert(
        &mut entries,
        "capacity",
        Value::Fixed64(Fixed64::from_i64(stream.capacity.min(i64::MAX as usize) as i64)),
    );
    insert(
        &mut entries,
        "head",
        Value::Fixed64(Fixed64::from_i64(stream.head.min(i64::MAX as usize) as i64)),
    );
    insert(
        &mut entries,
        "len",
        Value::Fixed64(Fixed64::from_i64(stream.len.min(i64::MAX as usize) as i64)),
    );
    insert(&mut entries, "buffer", Value::List(stream.buffer.clone()));
    Value::Map(entries)
}

fn stream_new(capacity: usize) -> Result<RuntimeStream, EvalError> {
    if capacity == 0 {
        return Err("흐름 용량은 1 이상이어야 합니다".to_string().into());
    }
    if capacity > STREAM_CAPACITY_MAX {
        return Err(format!(
            "흐름 용량이 너무 큽니다 (최대 {}): {}",
            STREAM_CAPACITY_MAX, capacity
        )
        .into());
    }
    Ok(RuntimeStream {
        capacity,
        head: 0,
        len: 0,
        buffer: vec![Value::None; capacity],
    })
}

fn stream_push(mut stream: RuntimeStream, value: Value) -> RuntimeStream {
    if stream.capacity == 0 {
        return stream;
    }
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
    if stream.capacity == 0 || stream.len == 0 {
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
            if let Some(kind) = numeric_pack_kind(value) {
                return match kind {
                    NUMERIC_KIND_BIG_INT => match items.get("값") {
                        Some(Value::String(text)) => text.clone(),
                        _ => "[큰바른수]".to_string(),
                    },
                    NUMERIC_KIND_RATIONAL => {
                        let num = items
                            .get("분자")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        let den = items
                            .get("분모")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!("{num}/{den}")
                    }
                    NUMERIC_KIND_FACTOR => items
                        .get("표기")
                        .and_then(|v| match v {
                            Value::String(text) => Some(text.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "[곱수]".to_string()),
                    _ => "[수형식]".to_string(),
                };
            }
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
        Value::Assertion(assertion) => assertion.canon.clone(),
        Value::StateMachine(machine) => state_machine_canon(machine),
        Value::Formula(formula) => formula.raw.clone(),
        Value::Template(template) => template.raw.clone(),
        Value::Regex(regex) => {
            if regex.flags.is_empty() {
                format!("정규식{{\"{}\"}}", regex.pattern)
            } else {
                format!("정규식{{\"{}\", \"{}\"}}", regex.pattern, regex.flags)
            }
        }
        Value::Lambda(lambda) => format!("<씨앗#{}>", lambda.id),
    }
}

fn state_machine_canon(machine: &StateMachine) -> String {
    let mut out = String::from("상태머신{ ");
    out.push_str(&machine.states.join(", "));
    out.push_str(" 으로 이뤄짐. ");
    out.push_str(&machine.initial);
    out.push_str(" 으로 시작.");
    for transition in &machine.transitions {
        out.push(' ');
        out.push_str(&transition.from);
        out.push_str(" 에서 ");
        out.push_str(&transition.to);
        out.push_str(" 으로");
        if let Some(guard_name) = &transition.guard_name {
            out.push_str(" 걸러서 ");
            out.push_str(guard_name);
        }
        if let Some(action_name) = &transition.action_name {
            out.push_str(" 하고 ");
            out.push_str(action_name);
        }
        out.push('.');
    }
    for check in &machine.on_transition_checks {
        out.push(' ');
        out.push_str("전이마다 ");
        out.push_str(check);
        out.push_str(" 살피기.");
    }
    out.push_str(" }");
    out
}

fn state_machine_resource_value(machine: &StateMachine) -> ResourceValue {
    let mut converted = BTreeMap::new();
    let insert = |map: &mut BTreeMap<String, ResourceMapEntry>, key: &str, value: ResourceValue| {
        let entry_key = ResourceValue::String(key.to_string());
        map.insert(
            entry_key.canon_key(),
            ResourceMapEntry {
                key: entry_key,
                value,
            },
        );
    };
    insert(
        &mut converted,
        "__ddn_kind",
        ResourceValue::String(STATE_MACHINE_RESOURCE_KIND.to_string()),
    );
    insert(
        &mut converted,
        "states",
        ResourceValue::List(
            machine
                .states
                .iter()
                .map(|state| ResourceValue::String(state.clone()))
                .collect(),
        ),
    );
    insert(
        &mut converted,
        "initial",
        ResourceValue::String(machine.initial.clone()),
    );
    insert(
        &mut converted,
        "checks",
        ResourceValue::List(
            machine
                .on_transition_checks
                .iter()
                .map(|check| ResourceValue::String(check.clone()))
                .collect(),
        ),
    );
    insert(
        &mut converted,
        "transitions",
        ResourceValue::List(
            machine
                .transitions
                .iter()
                .map(|transition| {
                    let mut transition_map = BTreeMap::new();
                    let from_key = ResourceValue::String("from".to_string());
                    transition_map.insert(
                        from_key.canon_key(),
                        ResourceMapEntry {
                            key: from_key,
                            value: ResourceValue::String(transition.from.clone()),
                        },
                    );
                    let to_key = ResourceValue::String("to".to_string());
                    transition_map.insert(
                        to_key.canon_key(),
                        ResourceMapEntry {
                            key: to_key,
                            value: ResourceValue::String(transition.to.clone()),
                        },
                    );
                    if let Some(guard_name) = &transition.guard_name {
                        let guard_key = ResourceValue::String("guard".to_string());
                        transition_map.insert(
                            guard_key.canon_key(),
                            ResourceMapEntry {
                                key: guard_key,
                                value: ResourceValue::String(guard_name.clone()),
                            },
                        );
                    }
                    if let Some(action_name) = &transition.action_name {
                        let action_key = ResourceValue::String("action".to_string());
                        transition_map.insert(
                            action_key.canon_key(),
                            ResourceMapEntry {
                                key: action_key,
                                value: ResourceValue::String(action_name.clone()),
                            },
                        );
                    }
                    ResourceValue::Map(transition_map)
                })
                .collect(),
        ),
    );
    ResourceValue::Map(converted)
}

fn assertion_resource_value(assertion: &Assertion) -> ResourceValue {
    let mut converted = BTreeMap::new();
    let insert = |map: &mut BTreeMap<String, ResourceMapEntry>, key: &str, value: ResourceValue| {
        let entry_key = ResourceValue::String(key.to_string());
        map.insert(
            entry_key.canon_key(),
            ResourceMapEntry {
                key: entry_key,
                value,
            },
        );
    };
    insert(
        &mut converted,
        "__ddn_kind",
        ResourceValue::String(ASSERTION_RESOURCE_KIND.to_string()),
    );
    insert(
        &mut converted,
        "canon",
        ResourceValue::String(assertion.canon.clone()),
    );
    ResourceValue::Map(converted)
}

fn resource_map_lookup<'a>(
    entries: &'a BTreeMap<String, ResourceMapEntry>,
    key: &str,
) -> Option<&'a ResourceValue> {
    entries.values().find_map(|entry| match &entry.key {
        ResourceValue::String(value) if value == key => Some(&entry.value),
        _ => None,
    })
}

fn resource_map_to_state_machine(
    entries: &BTreeMap<String, ResourceMapEntry>,
) -> Option<StateMachine> {
    let kind = match resource_map_lookup(entries, "__ddn_kind")? {
        ResourceValue::String(value) => value,
        _ => return None,
    };
    if kind != STATE_MACHINE_RESOURCE_KIND {
        return None;
    }
    let states = match resource_map_lookup(entries, "states")?.clone() {
        ResourceValue::List(items) => items
            .into_iter()
            .map(|item| match item {
                ResourceValue::String(value) => Some(value),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?,
        _ => return None,
    };
    let initial = match resource_map_lookup(entries, "initial")? {
        ResourceValue::String(value) => value.clone(),
        _ => return None,
    };
    let checks = match resource_map_lookup(entries, "checks") {
        Some(value) => match value.clone() {
            ResourceValue::List(items) => items
                .into_iter()
                .map(|item| match item {
                    ResourceValue::String(value) => Some(value),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?,
            _ => return None,
        },
        None => Vec::new(),
    };
    let transitions = match resource_map_lookup(entries, "transitions")?.clone() {
        ResourceValue::List(items) => items
            .into_iter()
            .map(|item| match item {
                ResourceValue::Map(map) => {
                    let from = match resource_map_lookup(&map, "from")? {
                        ResourceValue::String(value) => value.clone(),
                        _ => return None,
                    };
                    let to = match resource_map_lookup(&map, "to")? {
                        ResourceValue::String(value) => value.clone(),
                        _ => return None,
                    };
                    let guard_name = match resource_map_lookup(&map, "guard") {
                        Some(ResourceValue::String(value)) => Some(value.clone()),
                        Some(_) => return None,
                        None => None,
                    };
                    let action_name = match resource_map_lookup(&map, "action") {
                        Some(ResourceValue::String(value)) => Some(value.clone()),
                        Some(_) => return None,
                        None => None,
                    };
                    Some(StateTransition {
                        from,
                        to,
                        guard_name,
                        action_name,
                    })
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?,
        _ => return None,
    };
    Some(StateMachine {
        states,
        initial,
        transitions,
        on_transition_checks: checks,
    })
}

fn resource_map_to_assertion(entries: &BTreeMap<String, ResourceMapEntry>) -> Option<Assertion> {
    let kind = match resource_map_lookup(entries, "__ddn_kind")? {
        ResourceValue::String(value) => value,
        _ => return None,
    };
    if kind != ASSERTION_RESOURCE_KIND {
        return None;
    }
    let canon = match resource_map_lookup(entries, "canon")? {
        ResourceValue::String(value) => value.clone(),
        _ => return None,
    };
    let body_source = extract_assertion_body_source(&canon)?;
    Some(Assertion { body_source, canon })
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
        Value::Assertion(assertion) => Ok(assertion_resource_value(assertion)),
        Value::StateMachine(machine) => Ok(state_machine_resource_value(machine)),
        Value::Formula(_) => Err("수식은 자원에 대입할 수 없습니다".to_string().into()),
        Value::Template(_) => Err("글무늬는 자원에 대입할 수 없습니다".to_string().into()),
        Value::Regex(_) => Err("정규식은 자원에 대입할 수 없습니다".to_string().into()),
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
            if let Some(assertion) = resource_map_to_assertion(entries) {
                return Value::Assertion(assertion);
            }
            if let Some(machine) = resource_map_to_state_machine(entries) {
                return Value::StateMachine(machine);
            }
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
            if let Some(kind) = numeric_pack_kind(value) {
                return match kind {
                    NUMERIC_KIND_BIG_INT => {
                        let body = items
                            .get("값")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!("큰바른수{{{body}}}")
                    }
                    NUMERIC_KIND_RATIONAL => {
                        let num = items
                            .get("분자")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        let den = items
                            .get("분모")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!("나눔수{{{num}/{den}}}")
                    }
                    NUMERIC_KIND_FACTOR => {
                        let body = items
                            .get("표기")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!("곱수{{{body}}}")
                    }
                    _ => "수형식{}".to_string(),
                };
            }
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
        Value::Assertion(assertion) => assertion.canon.clone(),
        Value::StateMachine(machine) => state_machine_canon(machine),
        Value::Formula(formula) => formula.raw.clone(),
        Value::Template(template) => template.raw.clone(),
        Value::Regex(regex) => {
            if regex.flags.is_empty() {
                format!("정규식{{\"{}\"}}", regex.pattern)
            } else {
                format!("정규식{{\"{}\", \"{}\"}}", regex.pattern, regex.flags)
            }
        }
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

fn format_template_value(
    value: &Value,
    format: Option<&TemplateFormat>,
) -> Result<String, EvalError> {
    let Some(format) = format else {
        if matches!(value, Value::None) {
            return Err("채우기: 자리표시자 값이 없습니다".to_string().into());
        }
        return Ok(value_to_string(value));
    };
    let (number, suffix) = match value {
        Value::Fixed64(v) => {
            if format.unit.is_some() {
                return Err("글무늬 포맷 단위는 단위 값에만 적용됩니다"
                    .to_string()
                    .into());
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
    Func {
        name: String,
        args: Vec<FormulaExpr>,
    },
    Unary {
        op: FormulaOp,
        expr: Box<FormulaExpr>,
    },
    Binary {
        op: FormulaOp,
        left: Box<FormulaExpr>,
        right: Box<FormulaExpr>,
    },
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
        FormulaExpr::Unary {
            op: FormulaOp::Sub,
            expr,
        } => {
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
        _ => Err(EvalError::Message(
            "string variable or pack options".to_string(),
        )),
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
            "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: 적분하기는 차수를 지원하지 않습니다".to_string(),
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

fn diff_formula_expr(
    expr: &FormulaExpr,
    var: &str,
    label: &'static str,
) -> Result<FormulaExpr, EvalError> {
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
        Binary {
            op: Add,
            left,
            right,
        } => add(
            diff_formula_expr(left, var, label)?,
            diff_formula_expr(right, var, label)?,
        ),
        Binary {
            op: Sub,
            left,
            right,
        } => sub(
            diff_formula_expr(left, var, label)?,
            diff_formula_expr(right, var, label)?,
        ),
        Binary {
            op: Mul,
            left,
            right,
        } => {
            let dl = diff_formula_expr(left, var, label)?;
            let dr = diff_formula_expr(right, var, label)?;
            add(
                mul(dl, right.as_ref().clone()),
                mul(left.as_ref().clone(), dr),
            )
        }
        Binary {
            op: Div,
            left,
            right,
        } => {
            let dl = diff_formula_expr(left, var, label)?;
            let dr = diff_formula_expr(right, var, label)?;
            let num = sub(
                mul(dl, right.as_ref().clone()),
                mul(left.as_ref().clone(), dr),
            );
            let denom = pow(right.as_ref().clone(), num_i(2));
            div(num, denom)
        }
        Binary {
            op: Pow,
            left,
            right,
        } => {
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

fn integrate_formula_expr(
    expr: &FormulaExpr,
    var: &str,
    label: &'static str,
) -> Result<FormulaExpr, EvalError> {
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
        Binary {
            op: Add,
            left,
            right,
        } => add(
            integrate_formula_expr(left, var, label)?,
            integrate_formula_expr(right, var, label)?,
        ),
        Binary {
            op: Sub,
            left,
            right,
        } => sub(
            integrate_formula_expr(left, var, label)?,
            integrate_formula_expr(right, var, label)?,
        ),
        Binary {
            op: Mul,
            left,
            right,
        } => {
            if !expr_contains_var(left, var) {
                mul(
                    left.as_ref().clone(),
                    integrate_formula_expr(right, var, label)?,
                )
            } else if !expr_contains_var(right, var) {
                mul(
                    right.as_ref().clone(),
                    integrate_formula_expr(left, var, label)?,
                )
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
        Binary {
            op: Pow,
            left,
            right,
        } => {
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
        FormulaExpr::Binary { left, right, .. } => {
            expr_contains_var(left, var) || expr_contains_var(right, var)
        }
    }
}

fn detect_var_power(expr: &FormulaExpr, var: &str) -> Option<i64> {
    match expr {
        FormulaExpr::Var(name) if name == var => Some(1),
        FormulaExpr::Binary {
            op: FormulaOp::Pow,
            left,
            right,
        } => {
            if let FormulaExpr::Var(name) = &**left {
                if name == var {
                    return number_as_int(right);
                }
            }
            None
        }
        FormulaExpr::Binary {
            op: FormulaOp::Mul,
            left,
            right,
        } => {
            if let (Some(a), Some(b)) = (detect_var_power(left, var), detect_var_power(right, var))
            {
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
    FormulaExpr::Number(UnitValue {
        value: Fixed64::from_i64(0),
        dim: UnitDim::NONE,
    })
}

fn num_one() -> FormulaExpr {
    FormulaExpr::Number(UnitValue {
        value: Fixed64::from_i64(1),
        dim: UnitDim::NONE,
    })
}

fn num_i(value: i64) -> FormulaExpr {
    FormulaExpr::Number(UnitValue {
        value: Fixed64::from_i64(value),
        dim: UnitDim::NONE,
    })
}

fn num_ln10() -> FormulaExpr {
    FormulaExpr::Number(UnitValue {
        value: Fixed64::from_f64_lossy(libm::log(10.0)),
        dim: UnitDim::NONE,
    })
}

fn num_ln2() -> FormulaExpr {
    FormulaExpr::Number(UnitValue {
        value: Fixed64::from_f64_lossy(libm::log(2.0)),
        dim: UnitDim::NONE,
    })
}

fn func(name: &str, args: Vec<FormulaExpr>) -> FormulaExpr {
    FormulaExpr::Func {
        name: name.to_string(),
        args,
    }
}

fn unary_sub(expr: FormulaExpr) -> FormulaExpr {
    FormulaExpr::Unary {
        op: FormulaOp::Sub,
        expr: Box::new(expr),
    }
}

fn add(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) {
        return right;
    }
    if is_zero(&right) {
        return left;
    }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.add(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary {
        op: FormulaOp::Add,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn sub(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&right) {
        return left;
    }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.sub(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary {
        op: FormulaOp::Sub,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn mul(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) || is_zero(&right) {
        return num_zero();
    }
    if is_one(&left) {
        return right;
    }
    if is_one(&right) {
        return left;
    }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        return FormulaExpr::Number(l.mul(r));
    }
    FormulaExpr::Binary {
        op: FormulaOp::Mul,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn div(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) {
        return num_zero();
    }
    if is_one(&right) {
        return left;
    }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.div(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary {
        op: FormulaOp::Div,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn pow(base: FormulaExpr, exp: FormulaExpr) -> FormulaExpr {
    if let Some(e) = number_as_int(&exp) {
        if e == 0 {
            return num_one();
        }
        if e == 1 {
            return base;
        }
    }
    FormulaExpr::Binary {
        op: FormulaOp::Pow,
        left: Box::new(base),
        right: Box::new(exp),
    }
}

fn as_number(expr: &FormulaExpr) -> Option<UnitValue> {
    if let FormulaExpr::Number(value) = expr {
        Some(*value)
    } else {
        None
    }
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
        return Err(EvalError::Message(
            "FATAL:FORMULA_TOKEN_INVALID".to_string(),
        ));
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
                return Err(EvalError::Message("FATAL:FORMULA_ASCII1_VAR".to_string()));
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
            Err(EvalError::Message(format!(
                "풀기: 키 '{}'가 없습니다",
                name
            )))
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
            let sign = if raw > 0 {
                1
            } else if raw < 0 {
                -1
            } else {
                0
            };
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
                if a.value <= b.value {
                    a
                } else {
                    b
                }
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
        return Err(EvalError::Message("FATAL:FORMULA_POW_INVALID".to_string()));
    }
    if exponent.value.frac_part() != 0 {
        return Err(EvalError::Message("FATAL:FORMULA_POW_INVALID".to_string()));
    }
    let exp = exponent.value.int_part();
    if exp < 0 {
        return Err(EvalError::Message("FATAL:FORMULA_POW_INVALID".to_string()));
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
        Value::Assertion(_) => Err("세움값은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::StateMachine(_) => Err("상태머신값은 조건식에 사용할 수 없습니다"
            .to_string()
            .into()),
        Value::Formula(_) => Err("수식은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Template(_) => Err("글무늬는 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Regex(_) => Err("정규식은 조건식에 사용할 수 없습니다".to_string().into()),
        Value::Lambda(_) => Err("씨앗은 조건식에 사용할 수 없습니다".to_string().into()),
    }
}

fn parse_bogae_shape_call(entry: &str) -> Option<BogaeShapeCall> {
    let raw = entry.trim().trim_end_matches('.').trim();
    if raw.is_empty() {
        return None;
    }
    let open = raw.find('(')?;
    let close = raw.rfind(')')?;
    if close <= open || !raw[close + 1..].trim().is_empty() {
        return None;
    }
    let name = raw[..open].trim();
    let kind = match name {
        "선" | "line" => BogaeShapeKind::Line,
        "원" | "circle" => BogaeShapeKind::Circle,
        "점" | "point" => BogaeShapeKind::Point,
        "사각형" | "rect" => BogaeShapeKind::Rect,
        "글" | "text" => BogaeShapeKind::Text,
        "꺾은선" | "polyline" => BogaeShapeKind::Polyline,
        "다각형" | "polygon" => BogaeShapeKind::Polygon,
        _ => return None,
    };

    let mut positional = Vec::new();
    let mut named = BTreeMap::new();
    for token in split_top_level_args(&raw[open + 1..close]) {
        if let Some((key, value)) = split_named_arg(&token) {
            let lower = key.to_lowercase();
            named.insert(key, value.clone());
            if !lower.is_empty() {
                named.insert(lower, value);
            }
        } else {
            positional.push(token);
        }
    }

    Some(BogaeShapeCall {
        kind,
        positional,
        named,
    })
}

fn bogae_pick_arg<'a>(shape: &'a BogaeShapeCall, keys: &[&str], idx: usize) -> Option<&'a str> {
    for key in keys {
        if let Some(value) = shape.named.get(*key) {
            return Some(value.as_str());
        }
        let lower = key.to_lowercase();
        if let Some(value) = shape.named.get(&lower) {
            return Some(value.as_str());
        }
    }
    if idx == usize::MAX {
        return None;
    }
    shape.positional.get(idx).map(|value| value.as_str())
}

fn split_top_level_args(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if let Some(active) = quote {
            if ch == active && (i == 0 || chars[i - 1] != '\\') {
                quote = None;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                let token: String = chars[start..i].iter().collect();
                let trimmed = token.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let token: String = chars[start..].iter().collect();
    let trimmed = token.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
    out
}

fn split_named_arg(token: &str) -> Option<(String, String)> {
    let chars: Vec<char> = token.chars().collect();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    for (i, ch) in chars.iter().enumerate() {
        if let Some(active) = quote {
            if *ch == active && (i == 0 || chars[i - 1] != '\\') {
                quote = None;
            }
            continue;
        }
        match *ch {
            '"' | '\'' => quote = Some(*ch),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '=' if depth == 0 => {
                let key: String = chars[..i].iter().collect();
                let value: String = chars[i + 1..].iter().collect();
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                if key.is_empty() || value.is_empty() {
                    return None;
                }
                return Some((key, value));
            }
            _ => {}
        }
    }
    None
}

fn strip_wrapping_quotes(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.len() < 2 {
        return None;
    }
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return Some(trimmed[1..trimmed.len() - 1].to_string());
    }
    None
}

fn bogae_point_value(x: Value, y: Value) -> Value {
    let mut map = BTreeMap::new();
    let x_key = Value::String("x".to_string());
    map.insert(
        map_key_canon(&x_key),
        MapEntry {
            key: x_key,
            value: x,
        },
    );
    let y_key = Value::String("y".to_string());
    map.insert(
        map_key_canon(&y_key),
        MapEntry {
            key: y_key,
            value: y,
        },
    );
    Value::Map(map)
}

fn bogae_points_from_positional(
    shape: &BogaeShapeCall,
    locals: &mut HashMap<String, Value>,
    ctx: &mut EvalContext<'_>,
) -> Option<Value> {
    if shape.positional.len() < 4 || shape.positional.len() % 2 != 0 {
        return None;
    }
    let mut points = Vec::new();
    let mut idx = 0usize;
    while idx + 1 < shape.positional.len() {
        let x = ctx.parse_bogae_value_token(locals, &shape.positional[idx])?;
        let y = ctx.parse_bogae_value_token(locals, &shape.positional[idx + 1])?;
        points.push(bogae_point_value(x, y));
        idx += 2;
    }
    Some(Value::List(points))
}

fn bogae_read_arg(
    ctx: &mut EvalContext<'_>,
    locals: &mut HashMap<String, Value>,
    shape: &BogaeShapeCall,
    keys: &[&str],
    idx: usize,
    default: Value,
) -> Value {
    bogae_pick_arg(shape, keys, idx)
        .and_then(|text| ctx.parse_bogae_value_token(locals, text))
        .unwrap_or(default)
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
    ddonirang_lang::stdlib::canonicalize_type_alias(name).to_string()
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
        "바른수" => match value {
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
        "세움값" => match value {
            Value::Assertion(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "상태머신값" => match value {
            Value::StateMachine(_) => Ok(()),
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
        "큰바른수" => {
            if numeric_pack_kind(value) == Some(NUMERIC_KIND_BIG_INT) {
                return Ok(());
            }
            match value {
                Value::Fixed64(n) if n.frac_part() == 0 => Ok(()),
                Value::Unit(unit) if unit.is_dimensionless() && unit.value.frac_part() == 0 => {
                    Ok(())
                }
                _ => Err(type_mismatch_detail(&canonical, value)),
            }
        }
        "나눔수" => {
            if numeric_pack_kind(value) == Some(NUMERIC_KIND_RATIONAL) {
                return Ok(());
            }
            match value {
                Value::Fixed64(n) if n.frac_part() == 0 => Ok(()),
                Value::Unit(unit) if unit.is_dimensionless() && unit.value.frac_part() == 0 => {
                    Ok(())
                }
                _ => Err(type_mismatch_detail(&canonical, value)),
            }
        }
        "곱수" => {
            if numeric_pack_kind(value) == Some(NUMERIC_KIND_FACTOR) {
                return Ok(());
            }
            match value {
                Value::Fixed64(n) if n.frac_part() == 0 => Ok(()),
                Value::Unit(unit) if unit.is_dimensionless() && unit.value.frac_part() == 0 => {
                    Ok(())
                }
                _ => Err(type_mismatch_detail(&canonical, value)),
            }
        }
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
        "정규식" => match value {
            Value::Regex(_) => Ok(()),
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

fn check_unit_type(value: &Value, base: &str, unit: &str) -> Result<(), TypeMismatchDetail> {
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
                "바른수".to_string()
            } else {
                "수".to_string()
            }
        }
        Value::Unit(unit) => {
            if unit.is_dimensionless() {
                if unit.value.frac_part() == 0 {
                    "바른수".to_string()
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
        Value::Pack(_) => numeric_pack_kind(value)
            .map(|kind| kind.to_string())
            .unwrap_or_else(|| "묶음".to_string()),
        Value::Assertion(_) => "세움값".to_string(),
        Value::StateMachine(_) => "상태머신값".to_string(),
        Value::Formula(_) => "수식값".to_string(),
        Value::Template(_) => "글무늬값".to_string(),
        Value::Regex(_) => "정규식".to_string(),
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
    format!(
        "파싱 실패: {} ({}:{})\n{}\n{}",
        err.message, line, col, line_text, caret
    )
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
    use ddonirang_lang::runtime::Value as RuntimeValue;
    use std::sync::Mutex;

    static AGE_TARGET_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct FactorizationTestOverrideGuard {
        previous_iters: usize,
        previous_trial_limit: u64,
    }

    impl FactorizationTestOverrideGuard {
        fn new(iter_limit: usize, trial_limit: u64) -> Self {
            Self {
                previous_iters: swap_factor_pollard_max_iters_for_test(iter_limit),
                previous_trial_limit: swap_factor_trial_fallback_limit_for_test(trial_limit),
            }
        }
    }

    impl Drop for FactorizationTestOverrideGuard {
        fn drop(&mut self) {
            swap_factor_pollard_max_iters_for_test(self.previous_iters);
            swap_factor_trial_fallback_limit_for_test(self.previous_trial_limit);
        }
    }

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

    fn extract_fixed(resources: &HashMap<String, RuntimeValue>, key: &str) -> Fixed64 {
        match resources.get(key) {
            Some(RuntimeValue::Fixed64(value)) => *value,
            Some(other) => panic!("{} must be Fixed64, got {:?}", key, other),
            None => panic!("{} missing", key),
        }
    }

    fn contract_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == "L0-CONTRACT-01" => Some(event),
                _ => None,
            })
            .collect()
    }

    fn assertion_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == "L1-ASSERT-01" => Some(event),
                _ => None,
            })
            .collect()
    }

    fn state_transition_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == "L1-STATE-01" => Some(event),
                _ => None,
            })
            .collect()
    }

    fn numeric_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == NUMERIC_DIAG_RULE_ID_FACTOR_DECOMP_DEFERRED => Some(event),
                _ => None,
            })
            .collect()
    }

    fn numeric_factor_route_summary_diag_events(output: &DdnRunOutput) -> Vec<&DiagEvent> {
        output
            .patch
            .ops
            .iter()
            .filter_map(|op| match op {
                PatchOp::EmitSignal {
                    signal: Signal::Diag { event },
                    ..
                } if event.rule_id == NUMERIC_DIAG_RULE_ID_FACTOR_ROUTE_SUMMARY => Some(event),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn from_source_rejects_legacy_header_frontdoor() {
        let source = "#이름: legacy\nx <- 1.\n";
        let err = match DdnProgram::from_source(source, "legacy_header.ddn") {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn from_source_rejects_legacy_boim_surface_frontdoor() {
        let source = "보임 {\n  x: 1.\n}.\n";
        let err = match DdnProgram::from_source(source, "legacy_boim.ddn") {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_CANON_LEGACY_BOIM_FORBIDDEN"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn keyboard_alias_resources_track_hold_press_release() {
        let script = r#"
매틱:움직씨 = {
}.
"#;
        let program = DdnProgram::from_source(script, "keyboard_alias_flags.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults = HashMap::new();

        let mut first = empty_input();
        first.keys_pressed = KEY_A;
        first.last_key_name = "ArrowLeft".to_string();
        let first_out = runner
            .run_update(&world, &first, &defaults)
            .expect("first run update");
        assert_eq!(
            extract_fixed(&first_out.resources, "샘.키보드.누르고있음.왼쪽화살표"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&first_out.resources, "샘.키보드.눌림.왼쪽화살표"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&first_out.resources, "샘.키보드.뗌.왼쪽화살표"),
            Fixed64::from_i64(0)
        );
        assert_eq!(
            extract_fixed(&first_out.resources, "입력상태.키_눌림.왼쪽화살표"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&first_out.resources, "샘.키보드.눌림.Z키"),
            Fixed64::from_i64(0)
        );

        let mut second = empty_input();
        second.keys_pressed = 0;
        let second_out = runner
            .run_update(&world, &second, &defaults)
            .expect("second run update");
        assert_eq!(
            extract_fixed(&second_out.resources, "샘.키보드.누르고있음.왼쪽화살표"),
            Fixed64::from_i64(0)
        );
        assert_eq!(
            extract_fixed(&second_out.resources, "샘.키보드.눌림.왼쪽화살표"),
            Fixed64::from_i64(0)
        );
        assert_eq!(
            extract_fixed(&second_out.resources, "샘.키보드.뗌.왼쪽화살표"),
            Fixed64::from_i64(1)
        );

        let mut third = empty_input();
        third.keys_pressed = INPUT_KEY_Z;
        third.last_key_name = "KeyZ".to_string();
        let third_out = runner
            .run_update(&world, &third, &defaults)
            .expect("third run update");
        assert_eq!(
            extract_fixed(&third_out.resources, "샘.키보드.눌림.Z키"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&third_out.resources, "입력상태.키_눌림.Z키"),
            Fixed64::from_i64(1)
        );
    }

    #[test]
    fn bogae_shape_block_emits_drawlist() {
        let script = r##"
매틱:움직씨 = {
    bob_x <- 0.5.
    bob_y <- -0.8.
    모양: {
        선(0, 0, bob_x, bob_y, 색="#9ca3af", 굵기=0.02).
        polyline(0, 0, 0.25, 0.2, 0.4, -0.1, 색="#22c55e", 굵기=0.03, 층=2, 그룹="pendulum.path").
        polygon(-0.1, -0.05, 0.1, -0.05, 0.0, 0.16, 채움색="#f59e0b", 선색="#7c2d12", 굵기=0.01, 레이어=5, group="pendulum.body").
        사각형(-0.2, -0.1, 0.4, 0.2, 채움색="#1e293b", 선색="#38bdf8", 굵기=0.01).
        글(-0.1, 0.25, "추", 크기=0.09, 색="#f8fafc").
        원(bob_x, bob_y, r=0.08, 색="#38bdf8", 선색="#0ea5e9", 굵기=0.02).
        점(0, 0, 크기=0.045, 색="#f59e0b").
    }.
}
"##;
        let program = DdnProgram::from_source(script, "bogae_shape.ddn").expect("parse");
        let seed = program.functions.get("매틱").expect("매틱 seed");
        let body = seed.body.as_ref().expect("매틱 body");
        let Stmt::MetaBlock { entries, .. } = &body.stmts[2] else {
            panic!("모양 meta block expected");
        };
        assert_eq!(entries.len(), 7, "entries={entries:?}");
        let world = NuriWorld::new();
        let preview_defaults = HashMap::new();
        let mut preview_ctx = EvalContext::new(
            &program,
            &world,
            &preview_defaults,
            InputState::new(0, 0),
            0,
            0,
        );
        let mut preview_locals = HashMap::new();
        preview_locals.insert(
            "bob_x".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_f64_lossy(0.5)),
        );
        preview_locals.insert(
            "bob_y".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_f64_lossy(-0.8)),
        );
        assert!(
            preview_ctx
                .bogae_entry_to_draw_item(&mut preview_locals, &entries[1])
                .is_some(),
            "polyline entry must render: {}",
            entries[1]
        );
        assert!(
            preview_ctx
                .bogae_entry_to_draw_item(&mut preview_locals, &entries[2])
                .is_some(),
            "polygon entry must render: {}",
            entries[2]
        );
        let mut runner = DdnRunner::new(program, "매틱");
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        let drawlist = output.resources.get("보개_그림판_목록");
        let Some(RuntimeValue::List(items)) = drawlist else {
            panic!("보개_그림판_목록 must be list, got {:?}", drawlist);
        };
        assert_eq!(items.len(), 7);

        let kinds: Vec<String> = items
            .iter()
            .map(|item| {
                let RuntimeValue::Map(map) = item else {
                    panic!("도형은 map 이어야 합니다");
                };
                map.values()
                    .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "kind"))
                    .and_then(|entry| match &entry.value {
                        RuntimeValue::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .expect("kind string")
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["line", "polyline", "polygon", "rect", "text", "circle", "point"]
        );

        let RuntimeValue::Map(polyline) = &items[1] else {
            panic!("꺾은선은 map 이어야 합니다");
        };
        let polyline_points = polyline
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "points"))
            .map(|entry| entry.value.clone());
        let Some(RuntimeValue::List(poly_points)) = polyline_points else {
            panic!("꺾은선 points must be list");
        };
        assert_eq!(poly_points.len(), 3);
        let polyline_layer = polyline
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "layer_index"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polyline_layer,
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(2)))
        );
        let polyline_group = polyline
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "group_id"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polyline_group,
            Some(RuntimeValue::String("pendulum.path".to_string()))
        );

        let RuntimeValue::Map(polygon) = &items[2] else {
            panic!("다각형은 map 이어야 합니다");
        };
        let polygon_fill = polygon
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "fill"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polygon_fill,
            Some(RuntimeValue::String("#f59e0b".to_string()))
        );
        let polygon_layer = polygon
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "layer_index"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polygon_layer,
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(5)))
        );
        let polygon_group = polygon
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "group_id"))
            .map(|entry| entry.value.clone());
        assert_eq!(
            polygon_group,
            Some(RuntimeValue::String("pendulum.body".to_string()))
        );

        let RuntimeValue::Map(rect) = &items[3] else {
            panic!("사각형은 map 이어야 합니다");
        };
        let rect_fill = rect
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "fill"))
            .map(|entry| entry.value.clone());
        assert_eq!(rect_fill, Some(RuntimeValue::String("#1e293b".to_string())));

        let RuntimeValue::Map(text) = &items[4] else {
            panic!("글은 map 이어야 합니다");
        };
        let text_value = text
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(k) if k == "text"))
            .map(|entry| entry.value.clone());
        assert_eq!(text_value, Some(RuntimeValue::String("추".to_string())));
    }

    #[test]
    fn bogae_shape_helpers_parse_polyline_and_polygon() {
        let polyline = parse_bogae_shape_call(
            r##"polyline(0, 0, 0.25, 0.2, 0.4, -0.1, 색="#22c55e", 굵기=0.03)"##,
        )
        .expect("polyline parse");
        assert_eq!(polyline.positional.len(), 6);
        assert!(polyline.named.contains_key("색"));

        let polygon = parse_bogae_shape_call(
            r##"polygon(-0.1, -0.05, 0.1, -0.05, 0.0, 0.16, 채움색="#f59e0b", 선색="#7c2d12", 굵기=0.01)"##,
        )
        .expect("polygon parse");
        assert_eq!(polygon.positional.len(), 6);
        assert!(polygon.named.contains_key("채움색"));
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

    #[test]
    fn map_optional_lookup_returns_none_when_missing() {
        let script = r#"
매틱:움직씨 = {
    m <- ("a", 1) 짝맞춤.
    있음 <- (m, "a") 찾기?.
    없음인가 <- ((m, "b") 찾기?) 아니다.
}
"#;
        let program = DdnProgram::from_source(script, "test.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("있음".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("없음인가".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            extract_fixed(&output.resources, "있음"),
            Fixed64::from_i64(1)
        );
        match output.resources.get("없음인가") {
            Some(RuntimeValue::Bool(true)) => {}
            other => panic!("없음인가 must be true, got {:?}", other),
        }
    }

    #[test]
    fn gini_and_quantile_builtins_support_aliases() {
        let script = r#"
매틱:움직씨 = {
    값목록 <- (1, 2, 3, 4) 차림.
    지니1 <- (값목록) 지니.
    지니2 <- (값목록) 지니계수.
    분위1 <- (값목록, 0.75, "선형보간") 분위수.
    분위2 <- (값목록, 0.75, "최근순위") 백분위수.
}
"#;
        let program = DdnProgram::from_source(script, "stats.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["지니1", "지니2", "분위1", "분위2"] {
            defaults.insert(key.to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(
            extract_fixed(&output.resources, "지니1"),
            Fixed64::from_f64_lossy(0.25)
        );
        assert_eq!(
            extract_fixed(&output.resources, "지니2"),
            Fixed64::from_f64_lossy(0.25)
        );
        assert_eq!(
            extract_fixed(&output.resources, "분위1"),
            Fixed64::from_f64_lossy(3.25)
        );
        assert_eq!(
            extract_fixed(&output.resources, "분위2"),
            Fixed64::from_i64(3)
        );
    }

    #[test]
    fn quantile_rejects_unknown_mode() {
        let script = r#"
매틱:움직씨 = {
    값목록 <- (1, 2, 3, 4) 차림.
    분위 <- (값목록, 0.5, "linear") 분위수.
}
"#;
        let program = DdnProgram::from_source(script, "stats_bad_mode.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("unknown mode must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("분위수 mode는 선형보간 또는 최근순위여야 합니다"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn stdlib_l1_integrator_and_interpolation_builtins() {
        let script = r#"
매틱:움직씨 = {
    값1 <- (1, 2, 0.5) 적분.오일러.
    쌍값 <- (1, 0, -1, 0.25) 적분.반암시적오일러.
    x2 <- (쌍값, 0) 차림.값.
    v2 <- (쌍값, 1) 차림.값.
    보간1 <- (0, 10, 0.25) 보간.선형.
    보간2 <- (0, 10, 0.25, 0.5) 보간.계단.
    보간3 <- (0, 10, 0.75, 0.5) 보간.계단.
}
"#;
        let program = DdnProgram::from_source(script, "stdlib_l1_integrators.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["값1", "x2", "v2", "보간1", "보간2", "보간3"] {
            defaults.insert(key.to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(
            extract_fixed(&output.resources, "값1"),
            Fixed64::from_i64(2)
        );
        assert_eq!(
            extract_fixed(&output.resources, "x2"),
            Fixed64::from_f64_lossy(0.9375)
        );
        assert_eq!(
            extract_fixed(&output.resources, "v2"),
            Fixed64::from_f64_lossy(-0.25)
        );
        assert_eq!(
            extract_fixed(&output.resources, "보간1"),
            Fixed64::from_f64_lossy(2.5)
        );
        assert_eq!(extract_fixed(&output.resources, "보간2"), Fixed64::ZERO);
        assert_eq!(
            extract_fixed(&output.resources, "보간3"),
            Fixed64::from_i64(10)
        );
    }

    #[test]
    fn stdlib_l1_filter_builtins() {
        let script = r#"
매틱:움직씨 = {
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
}
"#;
        let program = DdnProgram::from_source(script, "stdlib_l1_filters.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["평균1", "평균2", "평균3", "평균4", "y1", "y2"] {
            defaults.insert(key.to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(
            extract_fixed(&output.resources, "평균1"),
            Fixed64::from_i64(1)
        );
        assert_eq!(
            extract_fixed(&output.resources, "평균2"),
            Fixed64::from_f64_lossy(1.5)
        );
        assert_eq!(
            extract_fixed(&output.resources, "평균3"),
            Fixed64::from_i64(2)
        );
        assert_eq!(
            extract_fixed(&output.resources, "평균4"),
            Fixed64::from_f64_lossy(2.5)
        );
        assert_eq!(
            extract_fixed(&output.resources, "y1"),
            Fixed64::from_f64_lossy(0.5)
        );
        assert_eq!(
            extract_fixed(&output.resources, "y2"),
            Fixed64::from_f64_lossy(0.75)
        );
    }

    #[test]
    fn numeric_calculus_v1_builtins_return_value_error_method() {
        let script = r#"
매틱:움직씨 = {
    미분묶음 <- (((#ascii) 수식{x^3}), "x", 1, 0.5) 미분.중앙차분.
    미분값 <- (미분묶음, 0) 차림.값.
    미분오차 <- (미분묶음, 1) 차림.값.
    미분방법 <- (미분묶음, 2) 차림.값.

    적분묶음 <- (((#ascii) 수식{x^2}), "x", 0, 1, 0.5) 적분.사다리꼴.
    적분값 <- (적분묶음, 0) 차림.값.
    적분오차 <- (적분묶음, 1) 차림.값.
    적분방법 <- (적분묶음, 2) 차림.값.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_calculus_v1.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("미분값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("미분오차".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("미분방법".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("적분값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("적분오차".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("적분방법".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(
            extract_fixed(&output.resources, "미분값"),
            Fixed64::from_f64_lossy(3.0)
        );
        assert_eq!(
            extract_fixed(&output.resources, "미분오차"),
            Fixed64::from_f64_lossy(0.25)
        );
        match output.resources.get("미분방법") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "중앙차분"),
            other => panic!("미분방법 must be string, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "적분값"),
            Fixed64::from_raw_i64(1_431_655_766)
        );
        assert_eq!(
            extract_fixed(&output.resources, "적분오차"),
            Fixed64::from_raw_i64(44_739_242)
        );
        match output.resources.get("적분방법") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "사다리꼴"),
            other => panic!("적분방법 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_calculus_v1_rejects_zero_step() {
        let script = r#"
매틱:움직씨 = {
    d <- (((#ascii) 수식{x^2}), "x", 1, 0) 미분.중앙차분.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_calculus_v1_bad_step.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_CALC_NUMERIC_BAD_STEP"),
            "expected numeric step guard, got {err}"
        );
    }

    #[test]
    fn contract_pre_abort_restores_state_before_else_body() {
        let script = r#"
매틱:움직씨 = {
    x <- 5.
    { x > 10 }인것 바탕으로(중단) 아니면 {
        x <- x + 1.
    }
    y <- 1.
}
"#;
        let program = DdnProgram::from_source(script, "contract_pre_abort.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("x".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("y".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(extract_fixed(&output.resources, "x"), Fixed64::from_i64(5));
        assert!(!output.resources.contains_key("y"));
        let events = contract_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].mode.as_deref(), Some("중단"));
        assert_eq!(events[0].contract_kind.as_deref(), Some("pre"));
    }

    #[test]
    fn contract_post_abort_restores_state_before_repair_body() {
        let script = r#"
매틱:움직씨 = {
    y <- 5.
    { y < 5 }인것 다짐하고(중단) 아니면 {
        y <- y + 1.
    }
    z <- 1.
}
"#;
        let program = DdnProgram::from_source(script, "contract_post_abort.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("y".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("z".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(extract_fixed(&output.resources, "y"), Fixed64::from_i64(5));
        assert!(!output.resources.contains_key("z"));
        let events = contract_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].mode.as_deref(), Some("중단"));
        assert_eq!(events[0].contract_kind.as_deref(), Some("post"));
    }

    #[test]
    fn contract_alert_keeps_else_body_state_changes() {
        let script = r#"
매틱:움직씨 = {
    z <- 5.
    { z > 10 }인것 바탕으로(알림) 아니면 {
        z <- z + 1.
    }
    확인 <- z.
}
"#;
        let program = DdnProgram::from_source(script, "contract_alert.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("z".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("확인".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(extract_fixed(&output.resources, "z"), Fixed64::from_i64(6));
        assert_eq!(
            extract_fixed(&output.resources, "확인"),
            Fixed64::from_i64(6)
        );
        let events = contract_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].mode.as_deref(), Some("알림"));
        assert_eq!(events[0].contract_kind.as_deref(), Some("pre"));
    }

    #[test]
    fn tailed_call_alias_registers_myeonseo_variant() {
        let script = r#"
(왼:수~을~를, 오른:수~에) 더하:셈씨 = {
    왼 + 오른 돌려줘.
}

매틱:움직씨 = {
    합 <- (3을, 1에) 더하면서.
}
"#;
        let program = DdnProgram::from_source(script, "tail_alias_myeonseo.ddn").expect("parse");
        assert!(program.functions.contains_key("더하면서"));

        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("합".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(extract_fixed(&output.resources, "합"), Fixed64::from_i64(4));
    }

    #[test]
    fn state_machine_builtins_progress_and_store_value() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    전이_안전 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(중단) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 노랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        초록 에서 노랑 으로.
        노랑 에서 빨강 으로.
        전이마다 전이_안전 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
    확인 <- (기계, 다음) 지금상태.
}
"#;
        let program = DdnProgram::from_source(script, "state_machine.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("현재".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("다음".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("확인".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("기계") {
            Some(RuntimeValue::StateMachine(machine)) => {
                assert_eq!(machine.initial, "빨강");
                assert_eq!(machine.transitions.len(), 3);
            }
            other => panic!("기계 must be state machine, got {:?}", other),
        }
        match output.resources.get("현재") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "빨강"),
            other => panic!("현재 must be string, got {:?}", other),
        }
        match output.resources.get("다음") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "초록"),
            other => panic!("다음 must be string, got {:?}", other),
        }
        match output.resources.get("확인") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "초록"),
            other => panic!("확인 must be string, got {:?}", other),
        }
        let events = state_transition_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, "STATE_TRANSITION");
        assert_eq!(events[0].sub_reason.as_deref(), Some("NEXT"));
        assert_eq!(
            events[0].message.as_deref(),
            Some("상태 전이: 빨강 -> 초록")
        );
        assert_eq!(output.state_transitions.len(), 1);
        assert_eq!(output.state_transitions[0].madi, 0);
        assert_eq!(output.state_transitions[0].from, "빨강");
        assert_eq!(output.state_transitions[0].to, "초록");
        assert_eq!(
            output.state_transitions[0].message,
            "상태 전이: 빨강 -> 초록"
        );
    }

    #[test]
    fn state_machine_transition_check_failure_aborts_next_state() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    전이_실패 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(중단) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        전이마다 전이_실패 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program = DdnProgram::from_source(script, "state_machine_fail.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("transition check must fail"),
            Err(err) => err,
        };
        let text = err.to_string();
        assert!(
            text.contains("E_STATE_TRANSITION_CHECK_FAILED: 전이_실패"),
            "unexpected error: {text}"
        );
    }

    #[test]
    fn state_machine_transition_check_requires_resolved_assertion() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    기계 <- 상태머신{
        빨강, 초록 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        전이마다 전이_안전 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_unresolved.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("missing transition assertion must fail"),
            Err(err) => err,
        };
        let text = err.to_string();
        assert!(
            text.contains("E_STATE_TRANSITION_CHECK_UNRESOLVED: 전이_안전"),
            "unexpected error: {text}"
        );
    }

    #[test]
    fn state_machine_guard_selects_first_passing_transition_and_runs_action() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age1);
        let script = r#"
(현재상태:글, 다음상태:글) 기록:움직씨 = {
    횟수 <- 횟수 + 1.
    마지막 <- 다음상태.
}

매틱:움직씨 = {
    거짓_거름 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(중단) 아니면 {
            없음.
        }.
    }.
    참_거름 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(중단) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 거짓_거름 하고 기록.
        빨강 에서 파랑 으로 걸러서 참_거름 하고 기록.
        전이마다 참_거름 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_guard_action.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("현재".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("다음".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("마지막".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("횟수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("다음") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "파랑"),
            other => panic!("다음 must be string, got {:?}", other),
        }
        match output.resources.get("마지막") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "파랑"),
            other => panic!("마지막 must be string, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "횟수"),
            Fixed64::from_i64(1)
        );
        assert_eq!(output.state_transitions.len(), 1);
        assert_eq!(output.state_transitions[0].from, "빨강");
        assert_eq!(output.state_transitions[0].to, "파랑");
        assert_eq!(
            output.state_transitions[0].guard_name.as_deref(),
            Some("참_거름")
        );
        assert_eq!(
            output.state_transitions[0].action_name.as_deref(),
            Some("기록")
        );
    }

    #[test]
    fn state_machine_guard_rejected_when_all_candidates_fail() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    거짓_거름 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(중단) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 거짓_거름.
        빨강 에서 파랑 으로 걸러서 거짓_거름.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_guard_rejected.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("guard reject expected"),
            Err(err) => err,
        };
        let text = err.to_string();
        assert!(
            text.contains("E_STATE_TRANSITION_GUARD_REJECTED: 빨강"),
            "unexpected error: {text}"
        );
    }

    #[test]
    fn state_machine_is_age1_feature_gated() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let script = r#"
매틱:움직씨 = {
    기계 <- 상태머신{
        빨강, 초록, 노랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        초록 에서 노랑 으로.
        노랑 에서 빨강 으로.
    }.
}
"#;
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age0);
        let err = match DdnProgram::from_source(script, "state_machine_age0.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
        set_default_age_target(old_age);
        assert!(
            err.contains("E_AGE_NOT_AVAILABLE"),
            "expected age gate, got {err}"
        );
        assert!(
            err.contains("state_machine_literal"),
            "expected state machine feature, got {err}"
        );
    }

    #[test]
    fn assertion_check_returns_true_and_stores_value() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(중단) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let program = DdnProgram::from_source(script, "assertion_ok.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("결과".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("검사") {
            Some(RuntimeValue::Assertion(assertion)) => {
                assert!(assertion.canon.starts_with("세움{"));
            }
            other => panic!("검사 must be assertion, got {:?}", other),
        }
        match output.resources.get("결과") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("결과 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn assertion_check_failure_emits_diag_and_keeps_state_clean() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    x <- 1.
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(중단) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=0)인 검사 살피기.
}
"#;
        let program = DdnProgram::from_source(script, "assertion_fail.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("x".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("결과".to_string(), RuntimeValue::Bool(true));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(extract_fixed(&output.resources, "x"), Fixed64::from_i64(1));
        match output.resources.get("결과") {
            Some(RuntimeValue::Bool(value)) => assert!(!*value),
            other => panic!("결과 must be bool, got {:?}", other),
        }
        let events = assertion_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, "ASSERTION_FAILED");
    }

    #[test]
    fn assertion_is_age1_feature_gated() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let script = r#"
매틱:움직씨 = {
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(중단) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age0);
        let err = match DdnProgram::from_source(script, "assertion_age0.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
        set_default_age_target(old_age);
        assert!(
            err.contains("E_AGE_NOT_AVAILABLE"),
            "expected age gate, got {err}"
        );
        assert!(
            err.contains("assertion_literal") || err.contains("assertion_api"),
            "expected assertion feature, got {err}"
        );
    }

    #[test]
    fn regex_builtins_run_deterministically() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let script = r#"
매틱:움직씨 = {
    맞음 <- ("ab12", 정규식{"^[A-Z]{2}[0-9]+$", "i"}) 정규맞추기.
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
    줄매치 <- ("x\nAB\ny", 정규식{"^ab$", "mi"}) 정규찾기.
    점맞음 <- ("A\nb", 정규식{"a.b", "si"}) 정규맞추기.
    조각수 <- (조각) 길이.
}
"#;
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age3);
        let program = DdnProgram::from_source(script, "regex.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("맞음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("첫매치".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("캡처".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("이름캡처".to_string(), RuntimeValue::Map(BTreeMap::new()));
        defaults.insert(
            "이름선택캡처".to_string(),
            RuntimeValue::Map(BTreeMap::new()),
        );
        defaults.insert(
            "이름캡처없음".to_string(),
            RuntimeValue::Map(BTreeMap::new()),
        );
        defaults.insert("바꿈".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("재바꿈".to_string(), RuntimeValue::String(String::new()));
        defaults.insert(
            "이름재바꿈".to_string(),
            RuntimeValue::String(String::new()),
        );
        defaults.insert(
            "이름짧은재바꿈".to_string(),
            RuntimeValue::String(String::new()),
        );
        defaults.insert("캡처조각".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("줄매치".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("점맞음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("조각수".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("맞음") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("맞음 must be bool, got {:?}", other),
        }
        match output.resources.get("첫매치") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "12"),
            other => panic!("첫매치 must be string, got {:?}", other),
        }
        match output.resources.get("캡처") {
            Some(RuntimeValue::List(items)) => {
                let strings: Vec<String> = items
                    .iter()
                    .map(|item| match item {
                        RuntimeValue::String(value) => value.clone(),
                        other => panic!("캡처 item must be string, got {:?}", other),
                    })
                    .collect();
                assert_eq!(strings, vec!["ab-12", "ab", "12"]);
            }
            other => panic!("캡처 must be list, got {:?}", other),
        }
        match output.resources.get("이름캡처") {
            Some(RuntimeValue::Map(entries)) => {
                assert_eq!(entries.len(), 2);
                let word = entries.get("\"word\"").expect("word capture");
                let num = entries.get("\"num\"").expect("num capture");
                assert_eq!(value_canon(&word.value), "\"ab\"");
                assert_eq!(value_canon(&num.value), "\"12\"");
            }
            other => panic!("이름캡처 must be map, got {:?}", other),
        }
        match output.resources.get("이름선택캡처") {
            Some(RuntimeValue::Map(entries)) => {
                assert_eq!(entries.len(), 2);
                let word = entries.get("\"word\"").expect("word optional capture");
                let num = entries.get("\"num\"").expect("num optional capture");
                assert_eq!(value_canon(&word.value), "\"ab\"");
                assert_eq!(value_canon(&num.value), "\"\"");
            }
            other => panic!("이름선택캡처 must be map, got {:?}", other),
        }
        match output.resources.get("이름캡처없음") {
            Some(RuntimeValue::Map(entries)) => assert!(entries.is_empty()),
            other => panic!("이름캡처없음 must be map, got {:?}", other),
        }
        match output.resources.get("바꿈") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "a_b_"),
            other => panic!("바꿈 must be string, got {:?}", other),
        }
        match output.resources.get("재바꿈") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "12:ab"),
            other => panic!("재바꿈 must be string, got {:?}", other),
        }
        match output.resources.get("이름재바꿈") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "12:ab"),
            other => panic!("이름재바꿈 must be string, got {:?}", other),
        }
        match output.resources.get("이름짧은재바꿈") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "12:ab"),
            other => panic!("이름짧은재바꿈 must be string, got {:?}", other),
        }
        match output.resources.get("캡처조각") {
            Some(RuntimeValue::List(items)) => {
                let strings: Vec<String> = items
                    .iter()
                    .map(|item| match item {
                        RuntimeValue::String(value) => value.clone(),
                        other => panic!("캡처조각 item must be string, got {:?}", other),
                    })
                    .collect();
                assert_eq!(strings, vec!["ab".to_string(), "cd".to_string()]);
            }
            other => panic!("캡처조각 must be list, got {:?}", other),
        }
        match output.resources.get("줄매치") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "AB"),
            other => panic!("줄매치 must be string, got {:?}", other),
        }
        match output.resources.get("점맞음") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("점맞음 must be bool, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "조각수"),
            Fixed64::from_i64(3)
        );
    }

    #[test]
    fn stream_stdlib_push_and_read_order() {
        let script = r#"
매틱:움직씨 = {
    흐름 <- (3) 흐름.만들기.
    흐름 <- (흐름, "a") 흐름.밀어넣기.
    흐름 <- (흐름, "b") 흐름.밀어넣기.
    흐름 <- (흐름, "c") 흐름.밀어넣기.
    흐름 <- (흐름, "d") 흐름.밀어넣기.
    값들 <- (흐름) 흐름.차림.
    최근 <- (흐름) 흐름.최근값.
    길이 <- (흐름) 흐름.길이.
    용량 <- (흐름) 흐름.용량.
}
"#;
        let program = DdnProgram::from_source(script, "stream_stdlib.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("흐름".to_string(), RuntimeValue::Map(BTreeMap::new()));
        defaults.insert("값들".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("최근".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("길이".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("용량".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("값들") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_string(value))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec!["b".to_string(), "c".to_string(), "d".to_string()]);
            }
            other => panic!("값들 must be list, got {:?}", other),
        }
        match output.resources.get("최근") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "d"),
            other => panic!("최근 must be string, got {:?}", other),
        }
        assert_eq!(extract_fixed(&output.resources, "길이"), Fixed64::from_i64(3));
        assert_eq!(extract_fixed(&output.resources, "용량"), Fixed64::from_i64(3));
        match output.resources.get("흐름") {
            Some(RuntimeValue::Map(entries)) => {
                let schema = entries.get("\"__schema\"").expect("__schema");
                assert_eq!(value_canon(&schema.value), "\"ddn.stream.v1\"");
            }
            other => panic!("흐름 must be map, got {:?}", other),
        }
    }

    #[test]
    fn stream_stdlib_aliases_are_supported() {
        let script = r#"
매틱:움직씨 = {
    흐름 <- (2) 흐름만들기.
    흐름 <- (흐름, 1) 흐름추가.
    흐름 <- (흐름, 2) 흐름넣기.
    흐름 <- (흐름, 3) 흐름추가.
    값들 <- (흐름) 흐름값들.
    최근 <- (흐름) 흐름최근.
    최근하나 <- (흐름, 1) 흐름잘라보기.
    최근둘 <- (흐름, 2) 흐름최근N.
    흐름 <- (흐름) 흐름비우기.
    비운길이 <- (흐름) 흐름길이.
}
"#;
        let program = DdnProgram::from_source(script, "stream_stdlib_alias.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("흐름".to_string(), RuntimeValue::Map(BTreeMap::new()));
        defaults.insert("값들".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("최근".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("최근하나".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("최근둘".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("비운길이".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("값들") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_i64(value).expect("integer"))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec![2, 3]);
            }
            other => panic!("값들 must be list, got {:?}", other),
        }
        assert_eq!(extract_fixed(&output.resources, "최근"), Fixed64::from_i64(3));
        match output.resources.get("최근하나") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_i64(value).expect("integer"))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec![3]);
            }
            other => panic!("최근하나 must be list, got {:?}", other),
        }
        match output.resources.get("최근둘") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_i64(value).expect("integer"))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec![2, 3]);
            }
            other => panic!("최근둘 must be list, got {:?}", other),
        }
        assert_eq!(extract_fixed(&output.resources, "비운길이"), Fixed64::ZERO);
    }

    #[test]
    fn stream_stdlib_clear_and_slice_work() {
        let script = r#"
매틱:움직씨 = {
    흐름 <- (3) 흐름.만들기.
    흐름 <- (흐름, 1) 흐름.밀어넣기.
    흐름 <- (흐름, 2) 흐름.밀어넣기.
    흐름 <- (흐름, 3) 흐름.밀어넣기.
    흐름 <- (흐름, 4) 흐름.밀어넣기.
    최근둘 <- (흐름, 2) 흐름.잘라보기.
    흐름 <- (흐름) 흐름.비우기.
    비운길이 <- (흐름) 흐름.길이.
    비운값들 <- (흐름) 흐름.차림.
}
"#;
        let program = DdnProgram::from_source(script, "stream_stdlib_clear_slice.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("흐름".to_string(), RuntimeValue::Map(BTreeMap::new()));
        defaults.insert("최근둘".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("비운길이".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("비운값들".to_string(), RuntimeValue::List(Vec::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("최근둘") {
            Some(RuntimeValue::List(items)) => {
                let rendered = items
                    .iter()
                    .map(|value| value_to_i64(value).expect("integer"))
                    .collect::<Vec<_>>();
                assert_eq!(rendered, vec![3, 4]);
            }
            other => panic!("최근둘 must be list, got {:?}", other),
        }
        assert_eq!(extract_fixed(&output.resources, "비운길이"), Fixed64::ZERO);
        match output.resources.get("비운값들") {
            Some(RuntimeValue::List(items)) => assert!(items.is_empty()),
            other => panic!("비운값들 must be list, got {:?}", other),
        }
    }

    #[test]
    fn stream_stdlib_reports_consistent_stream_value_label() {
        let script = r#"
매틱:움직씨 = {
    길이 <- (흐름) 흐름.길이.
}
"#;
        let program = DdnProgram::from_source(script, "stream_stdlib_label_error.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("흐름".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("길이".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("흐름(흐름.만들기 결과) 인자가 필요합니다"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn regex_replacement_invalid_returns_dedicated_error() {
        let script = r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "${missing}") 정규바꾸기.
}
"#;
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age3);
        let program =
            DdnProgram::from_source(script, "regex_replacement_invalid.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("바꿈".to_string(), RuntimeValue::String(String::new()));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("E_REGEX_REPLACEMENT_INVALID"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn regex_replacement_numeric_backref_is_greedy_and_invalid_when_missing() {
        let script = r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}, "$10") 정규바꾸기.
}
"#;
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age3);
        let program = DdnProgram::from_source(script, "regex_replacement_numeric_invalid.ddn")
            .expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("바꿈".to_string(), RuntimeValue::String(String::new()));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("E_REGEX_REPLACEMENT_INVALID"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn regex_replacement_invalid_subcases_return_dedicated_error() {
        let cases = [
            (
                "regex_replacement_empty_ref_invalid.ddn",
                r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "${}") 정규바꾸기.
}
"#,
            ),
            (
                "regex_replacement_dangling_dollar_invalid.ddn",
                r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"(?P<word>[a-z]+)-(?P<num>[0-9]+)", "i"}, "$") 정규바꾸기.
}
"#,
            ),
            (
                "regex_replacement_numeric_overflow_invalid.ddn",
                r#"
매틱:움직씨 = {
    바꿈 <- ("ab-12", 정규식{"([a-z]+)-([0-9]+)", "i"}, "$999999999999999999999999") 정규바꾸기.
}
"#,
            ),
        ];
        for (file_name, script) in cases {
            let old_age = default_age_target();
            set_default_age_target(AgeTarget::Age3);
            let program = DdnProgram::from_source(script, file_name).expect("parse");
            set_default_age_target(old_age);
            let mut runner = DdnRunner::new(program, "매틱");
            let world = NuriWorld::new();
            let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
            defaults.insert("바꿈".to_string(), RuntimeValue::String(String::new()));
            let err = match runner.run_update(&world, &empty_input(), &defaults) {
                Ok(_) => panic!("must fail: {file_name}"),
                Err(err) => err,
            };
            assert!(
                err.to_string().contains("E_REGEX_REPLACEMENT_INVALID"),
                "unexpected error for {file_name}: {err}"
            );
        }
    }

    #[test]
    fn temperature_literals_compare_after_kelvin_normalization() {
        let script = r#"
매틱:움직씨 = {
    같음 <- 25@C == 77@F.
}
"#;
        let program = DdnProgram::from_source(script, "temperature_compare.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("같음".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("같음") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("같음 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn temperature_literals_subtract_to_kelvin_difference() {
        let script = r#"
매틱:움직씨 = {
    같음 <- (30@C - 20@C) == 10@K.
}
"#;
        let program = DdnProgram::from_source(script, "temperature_subtract.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("같음".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("같음") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("같음 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn template_format_can_render_temperature_in_celsius_and_fahrenheit() {
        let script = r#"
매틱:움직씨 = {
    섭씨텍스트 <- (t=298.15@K) 글무늬{"섭씨={t|@.1C}"}.
    화씨텍스트 <- (t=298.15@K) 글무늬{"화씨={t|@.1F}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "temperature_template_unit.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "섭씨텍스트".to_string(),
            RuntimeValue::String(String::new()),
        );
        defaults.insert(
            "화씨텍스트".to_string(),
            RuntimeValue::String(String::new()),
        );
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("섭씨텍스트") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "섭씨=25.0@C"),
            other => panic!("섭씨텍스트 must be string, got {:?}", other),
        }
        match output.resources.get("화씨텍스트") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "화씨=77.0@F"),
            other => panic!("화씨텍스트 must be string, got {:?}", other),
        }
    }

    #[test]
    fn template_format_defaults_temperature_display_to_kelvin() {
        let script = r#"
매틱:움직씨 = {
    기본텍스트 <- (t=25@C) 글무늬{"온도={t}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "temperature_template_default.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "기본텍스트".to_string(),
            RuntimeValue::String(String::new()),
        );
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("기본텍스트") {
            Some(RuntimeValue::String(value)) => {
                assert!(value.ends_with("@K"), "unexpected value: {}", value)
            }
            other => panic!("기본텍스트 must be string, got {:?}", other),
        }
    }

    #[test]
    fn regex_is_age3_feature_gated() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let script = r#"
매틱:움직씨 = {
    결과 <- ("a1", 정규식{"[0-9]+"}) 정규찾기.
}
"#;
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age2);
        let err = match DdnProgram::from_source(script, "regex_age2.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
        set_default_age_target(old_age);
        assert!(
            err.contains("E_AGE_NOT_AVAILABLE"),
            "expected age gate code, got {err}"
        );
        assert!(
            err.contains("regex_api"),
            "expected regex feature, got {err}"
        );
    }

    #[test]
    fn quantifier_is_age4_feature_gated() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let script = r#"
매틱:움직씨 = {
    n 이 자연수 낱낱에 대해 {
        없음.
    }.
}
"#;
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age3);
        let err = match DdnProgram::from_source(script, "quantifier_age3.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
        set_default_age_target(old_age);
        assert!(
            err.contains("E_AGE_NOT_AVAILABLE"),
            "expected age gate code, got {err}"
        );
        assert!(
            err.contains("logic_quantifier"),
            "expected quantifier feature, got {err}"
        );
    }

    #[test]
    fn quantifier_statement_runs_as_proof_only_noop() {
        let _age_lock = AGE_TARGET_TEST_LOCK.lock().expect("age test lock");
        let script = r#"
매틱:움직씨 = {
    결과 <- 3.
    n 이 자연수 낱낱에 대해 {
        없음.
    }.
}
"#;
        let old_age = default_age_target();
        set_default_age_target(AgeTarget::Age4);
        let program = DdnProgram::from_source(script, "quantifier_age4.ddn").expect("parse");
        set_default_age_target(old_age);
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("결과".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            extract_fixed(&output.resources, "결과"),
            Fixed64::from_i64(3)
        );
    }

    #[test]
    fn numeric_family_constructors_render_and_normalize() {
        let script = r#"
매틱:움직씨 = {
    큰 <- ("123456789012345678901234567890") 큰바른수.
    나눔 <- (2, 4) 나눔수.
    곱 <- (84) 곱수.
    요약 <- (큰=큰, 나눔=나눔, 곱=곱) 글무늬{"{큰}|{나눔}|{곱}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_ctor.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("큰".to_string(), RuntimeValue::None);
        defaults.insert("나눔".to_string(), RuntimeValue::None);
        defaults.insert("곱".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(
                    text,
                    "123456789012345678901234567890|1/2|2^2 * 3 * 7"
                );
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_alias_constructors_are_supported() {
        let script = r#"
매틱:움직씨 = {
    수값 <- ("3.25") fixed64.
    정 <- ("3") int.
    큰 <- ("9007199254740993") bigint.
    나눔 <- (6, 9) rational.
    곱 <- (72) factor.
    요약 <- (수값=수값, 정=정, 큰=큰, 나눔=나눔, 곱=곱) 글무늬{"{수값}|{정}|{큰}|{나눔}|{곱}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_ctor.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("수값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("정".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("큰".to_string(), RuntimeValue::None);
        defaults.insert("나눔".to_string(), RuntimeValue::None);
        defaults.insert("곱".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "3.25|3|9007199254740993|2/3|2^3 * 3^2");
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_sized_variant_aliases_are_supported() {
        let script = r#"
매틱:움직씨 = {
    a <- ("1.5") 셈수4.
    b <- ("7") 바른수8.
    c <- (a) 셈수2.
    d <- (b) 바른수4.
    요약 <- (c=c, d=d) 글무늬{"{c}|{d}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_sized_variant_alias.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("b".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("c".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("d".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "1.5|7");
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_sized_variant_type_pins_in_decl_block_are_supported() {
        let script = r#"
채비 {
    값:셈수2 <- 1.5.
    정:바른수4 <- 7.
}.

매틱:움직씨 = {
    요약 <- (값=값, 정=정) 글무늬{"{값}|{정}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_sized_variant_type_pin.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("정".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "1.5|7"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_english_alias_type_pins_in_decl_block_are_supported() {
        let script = r#"
채비 {
    실수형:fixed64 <- 1.5.
    정수형:int64 <- 7.
    큰:bigint <- ("9007199254740993") 큰바른수.
    유리:rational <- (6, 9) 나눔수.
    인수:factorized <- (72) 곱수.
}.

매틱:움직씨 = {
    요약 <- (실수형=실수형, 정수형=정수형, 큰=큰, 유리=유리, 인수=인수)
        글무늬{"{실수형}|{정수형}|{큰}|{유리}|{인수}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_english_alias_type_pin.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("실수형".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("정수형".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("큰".to_string(), RuntimeValue::None);
        defaults.insert("유리".to_string(), RuntimeValue::None);
        defaults.insert("인수".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert!(text.starts_with("1.5|7|9007199254740993|2/3|"));
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_fixed64_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:fixed64) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_fixed64.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("fixed64"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_int64_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:int64) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1.5:x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_int64.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=바른수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("int64"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_bigint_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:bigint) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_bigint.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=큰바른수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("bigint"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_rational_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:rational) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_rational.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=나눔수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("rational"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_factorized_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:factorized) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_factorized.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=곱수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("factorized"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_big_int_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:big_int) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_big_int.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=큰바른수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("big_int"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_ratio_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:ratio) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_ratio.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=나눔수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("ratio"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_frac_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:frac) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_frac.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=나눔수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("frac"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_factor_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:factor) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_factor.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=곱수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("factor"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_primepow_alias_type_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:primepow) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_primepow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=곱수"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("primepow"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_boolean_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:boolean) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_boolean.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=참거짓"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("boolean"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_list_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:list) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_list.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=차림"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("list"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_set_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:set) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_set.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=모음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("set"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_map_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:map) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_map.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=짝맞춤"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("map"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_pack_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:pack) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_pack.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=묶음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("pack"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_string_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:string) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_string.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=글"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("string"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_none_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:none) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_none.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=없음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("none"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_non_keyword_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:non) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program =
            DdnProgram::from_source(script, "type_alias_diag_non_keyword.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=없음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("non"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_non_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:논) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    1:x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_non.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=참거짓"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("논"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_mokrok_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:목록) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_mokrok.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=차림"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("목록"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_modum_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:모둠) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_modum.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=모음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("모둠"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_geurimpyo_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:그림표) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_geurimpyo.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=짝맞춤"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("그림표"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn type_alias_valuepack_mismatch_uses_canonical_expected_name() {
        let script = r#"
(x:값꾸러미) 통과:셈씨 = {
    x 돌려줘.
}

매틱:움직씨 = {
    "글":x 통과하기.
}
"#;
        let program = DdnProgram::from_source(script, "type_alias_diag_valuepack.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=묶음"),
            "expected canonical type name in error, got: {err}"
        );
        assert!(
            !err.contains("값꾸러미"),
            "alias name should not leak in canonical mismatch message: {err}"
        );
    }

    #[test]
    fn numeric_family_exact_infix_rational_addition() {
        let script = r#"
매틱:움직씨 = {
    a <- (1, 3) 나눔수.
    b <- (1, 6) 나눔수.
    c <- a + b.
    요약 <- (c=c) 글무늬{"{c}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_exact_rational_add.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("c".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "1/2"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_exact_infix_bigint_multiplication_and_division() {
        let script = r#"
매틱:움직씨 = {
    a <- ("10000000000000000000") 큰바른수.
    b <- ("3") 큰바른수.
    mul <- a * b.
    rat <- a / b.
    요약 <- (mul=mul, rat=rat) 글무늬{"{mul}|{rat}"}.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_family_exact_bigint_ops.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("mul".to_string(), RuntimeValue::None);
        defaults.insert("rat".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "30000000000000000000|10000000000000000000/3")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_exact_infix_bigint_over_i128_boundary() {
        let script = r#"
매틱:움직씨 = {
    a <- ("340282366920938463463374607431768211456") 큰바른수.
    b <- ("1") 큰바른수.
    c <- a + b.
    요약 <- (c=c) 글무늬{"{c}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_bigint_over_i128.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("c".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "340282366920938463463374607431768211457")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_rational_accepts_bigint_operands() {
        let script = r#"
매틱:움직씨 = {
    n <- ("680564733841876926926749214863536422912", "2") 나눔수.
    기대 <- ("340282366920938463463374607431768211456") 큰바른수.
    같음 <- n == 기대.
    요약 <- (같음=같음) 글무늬{"{같음}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_rational_bigint_operands.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("기대".to_string(), RuntimeValue::None);
        defaults.insert("같음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "참"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_factor_constructor_accepts_bigint_within_i64() {
        let script = r#"
매틱:움직씨 = {
    n <- ("84") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 경로=경로, f=f) 글무늬{"{표식}|{경로}|{f}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_factor_from_bigint_ok.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "완료|i64|2^2 * 3 * 7"),
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_route_metrics_resource_is_emitted() {
        let huge = format!("1{}", "0".repeat(160));
        let script = format!(
            r#"
매틱:움직씨 = {{
    a <- ("84") 큰바른수.
    fa <- (a) 곱수.
    b <- ("{huge}") 큰바른수.
    fb <- (b) 곱수.
    c <- ("16000000064000000063") 큰바른수.
    fc <- (c) 곱수.
}}
"#
        );
        let program =
            DdnProgram::from_source(&script, "numeric_family_factor_route_metrics.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("c".to_string(), RuntimeValue::None);
        defaults.insert("fa".to_string(), RuntimeValue::None);
        defaults.insert("fb".to_string(), RuntimeValue::None);
        defaults.insert("fc".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        let summary = match output.resources.get(NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY) {
            Some(RuntimeValue::String(text)) => text.clone(),
            other => panic!("route summary must be string, got {:?}", other),
        };
        assert!(
            summary.contains("i64=1"),
            "summary must include i64 route: {summary}"
        );
        assert!(
            summary.contains("deferred:bitlimit=1"),
            "summary must include deferred bitlimit route: {summary}"
        );
        assert!(
            summary.contains("bigint:pollard=1"),
            "summary must include pollard route: {summary}"
        );
        let total = extract_fixed(&output.resources, NUMERIC_FACTOR_ROUTE_TOTAL_RESOURCE_KEY);
        assert_eq!(total.int_part(), 3);
        let bits_min = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_MIN_RESOURCE_KEY);
        let bits_max = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_MAX_RESOURCE_KEY);
        let bits_sum = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_SUM_RESOURCE_KEY);
        let policy = match output.resources.get(NUMERIC_FACTOR_POLICY_RESOURCE_KEY) {
            Some(RuntimeValue::String(text)) => text.clone(),
            other => panic!("factor policy must be string, got {:?}", other),
        };
        println!("numeric_factor_policy={policy}");
        assert!(
            policy.contains("bit_limit=512"),
            "policy must expose bit_limit: {policy}"
        );
        assert!(
            policy.contains("pollard_iters=200000"),
            "policy must expose pollard iters: {policy}"
        );
        assert!(
            policy.contains("fallback_limit=1000000"),
            "policy must expose fallback limit: {policy}"
        );
        assert!(
            policy.contains("small_prime_max=101"),
            "policy must expose small-prime max: {policy}"
        );
        assert_eq!(bits_min.int_part(), 7);
        assert!(bits_max.int_part() > FACTOR_BIGINT_FACTOR_BITS_LIMIT as i64);
        assert!(bits_sum.int_part() >= bits_min.int_part() * 3);
    }

    #[test]
    fn numeric_family_factor_route_metrics_diag_event_is_emitted() {
        let script = r#"
매틱:움직씨 = {
    a <- ("84") 큰바른수.
    fa <- (a) 곱수.
    c <- ("16000000064000000063") 큰바른수.
    fc <- (c) 곱수.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_factor_route_metrics_diag.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("c".to_string(), RuntimeValue::None);
        defaults.insert("fa".to_string(), RuntimeValue::None);
        defaults.insert("fc".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        let summary = match output.resources.get(NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY) {
            Some(RuntimeValue::String(text)) => text.clone(),
            other => panic!("route summary must be string, got {:?}", other),
        };
        let total = extract_fixed(&output.resources, NUMERIC_FACTOR_ROUTE_TOTAL_RESOURCE_KEY);
        let total_int = total.int_part();
        let policy = match output.resources.get(NUMERIC_FACTOR_POLICY_RESOURCE_KEY) {
            Some(RuntimeValue::String(text)) => text.clone(),
            other => panic!("factor policy must be string, got {:?}", other),
        };

        let events = numeric_factor_route_summary_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, NUMERIC_DIAG_REASON_FACTOR_ROUTE_SUMMARY);
        assert_eq!(
            events[0].expr.as_ref().map(|trace| trace.tag.as_str()),
            Some(NUMERIC_DIAG_TAG_FACTOR_ROUTE_SUMMARY)
        );
        let bits_min = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_MIN_RESOURCE_KEY);
        let bits_max = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_MAX_RESOURCE_KEY);
        let bits_sum = extract_fixed(&output.resources, NUMERIC_FACTOR_BITS_SUM_RESOURCE_KEY);
        let expected_trace = format!(
            "routes={summary};total={total_int};bit_min={};bit_max={};bit_sum={};policy={policy}",
            bits_min.int_part(),
            bits_max.int_part(),
            bits_sum.int_part()
        );
        assert_eq!(
            events[0]
                .expr
                .as_ref()
                .and_then(|trace| trace.text.as_deref()),
            Some(expected_trace.as_str())
        );
        let expected_message = format!(
            "곱수 분해경로 집계: {summary} (총 {total_int}, 비트 최소={}, 최대={}, 합={}, 정책={policy})",
            bits_min.int_part(),
            bits_max.int_part(),
            bits_sum.int_part()
        );
        assert_eq!(events[0].message.as_deref(), Some(expected_message.as_str()));
    }

    #[test]
    fn numeric_family_factor_constructor_deferred_for_bigint_out_of_i64() {
        let huge = format!("1{}", "0".repeat(160));
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{huge}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    원인 <- f.지연사유.
    경로 <- f.분해경로.
    비트 <- f.분해비트수.
    요약 <- (표식=표식, 원인=원인, 경로=경로, f=f) 글무늬{{"{{표식}}|{{원인}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program =
            DdnProgram::from_source(&script, "numeric_family_factor_from_bigint_overflow.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("원인".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("비트".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let expected_summary = format!("지연|비트한계초과|deferred:bitlimit|{huge}");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, expected_summary.as_str()),
            other => panic!("요약 must be string, got {:?}", other),
        }
        let bits = extract_fixed(&output.resources, "비트");
        assert!(bits.int_part() > FACTOR_BIGINT_FACTOR_BITS_LIMIT as i64);
        let events = numeric_diag_events(&output);
        assert_eq!(events.len(), 1);
        let expected_agg_routes = format!("{}=1", FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT);
        let expected_trace = format!(
            "route={};bits={};agg_routes={};agg_total=1",
            FACTOR_DECOMP_ROUTE_DEFERRED_BIT_LIMIT,
            bits.int_part(),
            expected_agg_routes
        );
        assert_eq!(
            events[0]
                .expr
                .as_ref()
                .and_then(|trace| trace.text.as_deref()),
            Some(expected_trace.as_str())
        );
        assert_eq!(
            events[0].reason,
            NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED
        );
        assert_eq!(
            events[0].sub_reason.as_deref(),
            Some(FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT)
        );
        let expected_message = format!("곱수 분해를 지연했습니다(비트한계초과): {huge}");
        assert_eq!(
            events[0].message.as_deref(),
            Some(expected_message.as_str())
        );
        assert_eq!(
            events[0].expr.as_ref().map(|trace| trace.tag.as_str()),
            Some(NUMERIC_DIAG_TAG_FACTOR_DECOMP_DEFERRED)
        );
        let span = events[0]
            .source_span
            .as_ref()
            .expect("deferred factor diag must carry source_span");
        assert_eq!(span.file, "numeric_family_factor_from_bigint_overflow.ddn");
        assert!(span.start_line >= 1);
    }

    #[test]
    fn numeric_family_factor_constructor_deferred_for_factor_failure_reason() {
        let _guard = FactorizationTestOverrideGuard::new(0, 1);
        let n = "9223372036854775809";
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{n}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    원인 <- f.지연사유.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 원인=원인, 경로=경로, f=f) 글무늬{{"{{표식}}|{{원인}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_from_bigint_factor_failed.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("원인".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "지연|분해실패|deferred:factorfailed|9223372036854775809")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
        let events = numeric_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0]
                .expr
                .as_ref()
                .and_then(|trace| trace.text.as_deref()),
            Some("route=deferred:factorfailed;bits=64;agg_routes=deferred:factorfailed=1;agg_total=1")
        );
        assert_eq!(
            events[0].sub_reason.as_deref(),
            Some(FACTOR_DECOMP_DEFERRED_REASON_FACTOR_FAILED)
        );
        assert_eq!(
            events[0].message.as_deref(),
            Some("곱수 분해를 지연했습니다(분해실패): 9223372036854775809")
        );
    }

    #[test]
    fn numeric_family_factor_constructor_uses_trial_fallback_when_pollard_disabled() {
        let _guard = FactorizationTestOverrideGuard::new(0, 1_000_000);
        let n = "9223372036854775809";
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{n}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 경로=경로, f=f) 글무늬{{"{{표식}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_from_bigint_trial_fallback.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert!(
                text.starts_with("완료|bigint:mixed|"),
                "unexpected summary: {text}"
            ),
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_constructor_strips_small_prime_without_pollard_or_fallback() {
        let _guard = FactorizationTestOverrideGuard::new(0, 1);
        let n = "510423550381407695195061911147652317181";
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{n}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 경로=경로, f=f) 글무늬{{"{{표식}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_from_bigint_small_prime_strip.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(
                text,
                "완료|bigint:smallprime|3 * 170141183460469231731687303715884105727"
            ),
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_constructor_uses_pollard_route_for_large_semiprime() {
        let n = "16000000064000000063";
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{n}") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    경로 <- f.분해경로.
    요약 <- (표식=표식, 경로=경로, f=f) 글무늬{{"{{표식}}|{{경로}}|{{f}}"}}.
}}
"#
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_from_bigint_pollard_route.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("경로".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "완료|bigint:pollard|4000000007 * 4000000009")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_deferred_diag_keeps_source_span_via_eval_callable() {
        let huge = format!("1{}", "0".repeat(160));
        let script = format!(
            r##"
매틱:움직씨 = {{
    함수 <- "#곱수".
    값들 <- ("{huge}") 차림.
    결과 <- (값들, 함수) 변환.
    첫 <- (결과) 첫번째.
    표식 <- 첫.분해상태.
    요약 <- (표식=표식) 글무늬{{"{{표식}}"}}.
}}
"##
        );
        let program = DdnProgram::from_source(
            &script,
            "numeric_family_factor_deferred_eval_callable_span.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("함수".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값들".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        defaults.insert("첫".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "지연"),
            other => panic!("요약 must be string, got {:?}", other),
        }
        let events = numeric_diag_events(&output);
        assert_eq!(events.len(), 1);
        assert!(
            events[0]
                .expr
                .as_ref()
                .and_then(|trace| trace.text.as_deref())
                .map(|text| {
                    text.starts_with("route=deferred:bitlimit;bits=")
                        && text.contains(";agg_routes=deferred:bitlimit=1;agg_total=1")
                })
                .unwrap_or(false),
            "deferred diag must expose route/bits/agg in expr.text"
        );
        assert_eq!(
            events[0].sub_reason.as_deref(),
            Some(FACTOR_DECOMP_DEFERRED_REASON_BIT_LIMIT)
        );
        assert_eq!(
            events[0].expr.as_ref().map(|trace| trace.tag.as_str()),
            Some(NUMERIC_DIAG_TAG_FACTOR_DECOMP_DEFERRED)
        );
        let span = events[0]
            .source_span
            .as_ref()
            .expect("eval_callable path must keep source_span");
        assert_eq!(span.file, "numeric_family_factor_deferred_eval_callable_span.ddn");
        assert!(span.start_line >= 1);
    }

    #[test]
    fn numeric_family_i64_path_accepts_bigint_in_range() {
        let script = r#"
매틱:움직씨 = {
    최소 <- ("1") 큰바른수.
    최대 <- ("3") 큰바른수.
    값 <- (최소, 최대) 무작위정수.
    유효 <- (값 >= 최소) 그리고 (값 <= 최대).
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_i64_path_in_range.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("최소".to_string(), RuntimeValue::None);
        defaults.insert("최대".to_string(), RuntimeValue::None);
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("유효".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("유효") {
            Some(RuntimeValue::Bool(flag)) => assert!(*flag),
            other => panic!("유효 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_factor_deferred_value_participates_exact_infix() {
        let huge = format!("1{}", "0".repeat(160));
        let script = format!(
            r#"
매틱:움직씨 = {{
    n <- ("{huge}") 큰바른수.
    f <- (n) 곱수.
    next <- f + ("1") 큰바른수.
    요약 <- (next=next) 글무늬{{"{{next}}"}}.
}}
"#
        );
        let program =
            DdnProgram::from_source(&script, "numeric_family_factor_deferred_exact_infix.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("next".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let expected_next = format!("1{}1", "0".repeat(159));
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, expected_next.as_str()),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_factor_constructor_handles_bigint_over_i128_when_prime() {
        let script = r#"
매틱:움직씨 = {
    n <- ("170141183460469231731687303715884105727") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    요약 <- (표식=표식, f=f) 글무늬{"{표식}|{f}"}.
}
"#;
        let program = DdnProgram::from_source(
            script,
            "numeric_family_factor_from_bigint_over_i128_prime.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "완료|170141183460469231731687303715884105727")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_factor_constructor_handles_bigint_over_i128_when_factorable() {
        let script = r#"
매틱:움직씨 = {
    n <- ("1361129467683753853853498429727072845824") 큰바른수.
    f <- (n) 곱수.
    표식 <- f.분해상태.
    요약 <- (표식=표식, f=f) 글무늬{"{표식}|{f}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_factor_from_bigint_over_i128.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("n".to_string(), RuntimeValue::None);
        defaults.insert("f".to_string(), RuntimeValue::None);
        defaults.insert("표식".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "완료|2^130"),
            other => panic!("요약 must be string, got {:?}", other),
        }
        assert!(numeric_diag_events(&output).is_empty());
    }

    #[test]
    fn numeric_family_i64_path_rejects_bigint_out_of_range() {
        let script = r#"
매틱:움직씨 = {
    최소 <- ("9223372036854775808") 큰바른수.
    최대 <- ("9223372036854775810") 큰바른수.
    값 <- (최소, 최대) 무작위정수.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_i64_path_overflow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("최소".to_string(), RuntimeValue::None);
        defaults.insert("최대".to_string(), RuntimeValue::None);
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("정수 값(i64 범위)이 필요합니다"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn numeric_family_exact_infix_comparison_and_equality() {
        let script = r#"
매틱:움직씨 = {
    a <- ("10000000000000000000") 큰바른수.
    b <- ("9999999999999999999") 큰바른수.
    r1 <- (1, 3) 나눔수.
    r2 <- (2, 5) 나눔수.
    같음 <- (2, 1) 나눔수 == ("2") 큰바른수.
    큼 <- a > b.
    작음 <- r1 < r2.
    요약 <- (같음=같음, 큼=큼, 작음=작음) 글무늬{"{같음}|{큼}|{작음}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_exact_compare.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("r1".to_string(), RuntimeValue::None);
        defaults.insert("r2".to_string(), RuntimeValue::None);
        defaults.insert("같음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("큼".to_string(), RuntimeValue::Bool(false));
        defaults.insert("작음".to_string(), RuntimeValue::Bool(false));
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "참|참|참"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_exact_infix_modulo_for_integer_family() {
        let script = r#"
매틱:움직씨 = {
    a <- ("10000000000000000003") 큰바른수.
    b <- ("7") 큰바른수.
    m <- a % b.
    요약 <- (m=m) 글무늬{"{m}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_exact_modulo.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("a".to_string(), RuntimeValue::None);
        defaults.insert("b".to_string(), RuntimeValue::None);
        defaults.insert("m".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => assert_eq!(text, "6"),
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_integer_types_reject_fractional_value() {
        let script = r#"
매틱:움직씨 = {
    정 <- (1.5) 바른수.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_int_reject.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("정".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("정수 값이 필요합니다"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn numeric_family_grouped_integer_strings_are_supported() {
        let script = r#"
매틱:움직씨 = {
    큰 <- ("9_223_372_036_854_775_808") 큰바른수.
    다음 <- 큰 + ("1") 큰바른수.
    나눔 <- ("1_000", "4") 나눔수.
    곱 <- ("84_000") 곱수.
    요약 <- (다음=다음, 나눔=나눔, 곱=곱) 글무늬{"{다음}|{나눔}|{곱}"}.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_grouped_integer_strings.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("큰".to_string(), RuntimeValue::None);
        defaults.insert("다음".to_string(), RuntimeValue::None);
        defaults.insert("나눔".to_string(), RuntimeValue::None);
        defaults.insert("곱".to_string(), RuntimeValue::None);
        defaults.insert("요약".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        match output.resources.get("요약") {
            Some(RuntimeValue::String(text)) => {
                assert_eq!(text, "9223372036854775809|250/1|2^5 * 3 * 5^3 * 7")
            }
            other => panic!("요약 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_family_grouped_integer_string_rejects_invalid_format() {
        let script = r#"
매틱:움직씨 = {
    큰 <- ("1__000") 큰바른수.
}
"#;
        let program =
            DdnProgram::from_source(script, "numeric_family_grouped_integer_invalid.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("큰".to_string(), RuntimeValue::None);
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("정수 문자열 형식이 아닙니다"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn decl_block_typecheck_rejects_mismatched_exact_numeric_initializer() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:나눔수 <- (12) 곱수.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_mismatch_exact_numeric.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("E_RUNTIME_TYPE_MISMATCH"),
            "unexpected error: {message}"
        );
        assert!(message.contains("핀=값"), "unexpected error: {message}");
        assert!(message.contains("기대=나눔수"), "unexpected error: {message}");
        assert!(message.contains("실제=곱수"), "unexpected error: {message}");
    }

    #[test]
    fn decl_block_type_alias_mismatch_uses_canonical_expected_name() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:rational <- (12) 곱수.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_alias_mismatch_rational.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("E_RUNTIME_TYPE_MISMATCH"),
            "unexpected error: {message}"
        );
        assert!(message.contains("기대=나눔수"), "unexpected error: {message}");
        assert!(
            !message.contains("기대=rational"),
            "expected canonical type name, got: {message}"
        );
    }

    #[test]
    fn decl_block_type_alias_accepts_collection_and_string_none_aliases() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        글값:string <- "또니".
        없음값:none <- 없음.
        비움값:non <- 없음.
        목록값:목록 <- (1, 2) 차림.
        모둠값:모둠 <- (1, 2, 2) 모음.
        그림표값:그림표 <- ("x", 1) 짝맞춤.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_alias_accepts_collection_string_none.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run should succeed");
    }

    #[test]
    fn decl_block_type_alias_none_mismatch_uses_canonical_expected_name() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:none <- 1.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_alias_none_mismatch.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("기대=없음"),
            "expected canonical type name, got: {message}"
        );
        assert!(
            !message.contains("기대=none"),
            "alias name should not leak in canonical mismatch message: {message}"
        );
    }

    #[test]
    fn decl_block_type_alias_non_keyword_mismatch_uses_canonical_expected_name() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:non <- 1.
    }.
}
"#;
        let program = DdnProgram::from_source(script, "decl_type_alias_non_mismatch.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        let message = err.to_string();
        assert!(
            message.contains("기대=없음"),
            "expected canonical type name, got: {message}"
        );
        assert!(
            !message.contains("기대=non"),
            "alias name should not leak in canonical mismatch message: {message}"
        );
    }

    #[test]
    fn decl_block_infer_type_skips_runtime_typecheck() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        값:_ <- (12) 곱수.
    }.
}
"#;
        let program =
            DdnProgram::from_source(script, "decl_type_infer_skips_typecheck.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run should succeed");
    }

    #[test]
    fn parse_warnings_include_block_header_colon_deprecation() {
        let script = r#"
매틱:움직씨 = {
    채비: { 값:수 <- 0. }.
}
"#;
        let program = DdnProgram::from_source(script, "warn_block_header.ddn").expect("parse");
        assert!(program
            .parse_warnings()
            .iter()
            .any(|w| w.code == "W_BLOCK_HEADER_COLON_DEPRECATED"));
    }
}
