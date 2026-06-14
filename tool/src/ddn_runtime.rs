#[cfg(test)]
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

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
const FORMULA_RESOURCE_KIND: &str = "ddn.formula.v1";
const BOGAE_SHOW_LINES_TAG: &str = "보개_출력_줄들";
const BOGAE_GRAPH_POINTS_F_TAG: &str = "보개_그래프_점목록_f";
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
    3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101,
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

fn fixed_value(value: i64) -> Value {
    Value::Fixed64(Fixed64::from_i64(value))
}

fn std_grid_error(message: &str) -> EvalError {
    EvalError::from(message.to_string())
}

fn std_block_piece_error(message: &str) -> EvalError {
    EvalError::from(message.to_string())
}

fn std_random_bag_error(message: &str) -> EvalError {
    EvalError::from(message.to_string())
}

fn std_grid_game_state_error(message: &str) -> EvalError {
    EvalError::from(message.to_string())
}

fn make_std_grid(width: i64, height: i64, default_value: Value) -> Result<Value, EvalError> {
    if width <= 0 || height <= 0 {
        return Err(std_grid_error("격자 크기는 1 이상이어야 합니다"));
    }
    let cell_count = width
        .checked_mul(height)
        .ok_or_else(|| std_grid_error("격자 크기가 너무 큽니다"))?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__kind".to_string(),
        Value::String(STD_GRID_KIND.to_string()),
    );
    fields.insert("너비".to_string(), fixed_value(width));
    fields.insert("높이".to_string(), fixed_value(height));
    fields.insert(
        "칸들".to_string(),
        Value::List(vec![default_value; cell_count as usize]),
    );
    Ok(Value::Pack(fields))
}

fn std_grid_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_error("격자 인자가 필요합니다"));
    };
    match fields.get("__kind") {
        Some(Value::String(kind)) if kind == STD_GRID_KIND => Ok(fields),
        _ => Err(std_grid_error("격자 인자가 필요합니다")),
    }
}

fn std_grid_dims(fields: &BTreeMap<String, Value>) -> Result<(i64, i64), EvalError> {
    let width = fields
        .get("너비")
        .ok_or_else(|| std_grid_error("격자 너비가 없습니다"))
        .and_then(value_to_i64)?;
    let height = fields
        .get("높이")
        .ok_or_else(|| std_grid_error("격자 높이가 없습니다"))
        .and_then(value_to_i64)?;
    if width <= 0 || height <= 0 {
        return Err(std_grid_error("격자 크기는 1 이상이어야 합니다"));
    }
    Ok((width, height))
}

fn std_grid_cells(fields: &BTreeMap<String, Value>) -> Result<&Vec<Value>, EvalError> {
    let Some(Value::List(cells)) = fields.get("칸들") else {
        return Err(std_grid_error("격자 칸 목록이 없습니다"));
    };
    Ok(cells)
}

fn std_grid_index(fields: &BTreeMap<String, Value>, x: i64, y: i64) -> Result<usize, EvalError> {
    let (width, height) = std_grid_dims(fields)?;
    if x < 0 || y < 0 || x >= width || y >= height {
        return Err(std_grid_error("격자 좌표가 범위를 벗어났습니다"));
    }
    Ok((y * width + x) as usize)
}

fn std_grid_inside(value: &Value, x: i64, y: i64) -> Result<bool, EvalError> {
    let fields = std_grid_fields(value)?;
    let (width, height) = std_grid_dims(fields)?;
    Ok(x >= 0 && y >= 0 && x < width && y < height)
}

fn std_block_piece_cell_from_value(value: &Value) -> Result<(i64, i64), EvalError> {
    let Value::List(items) = value else {
        return Err(std_block_piece_error("블록조각 칸은 차림[x, y]여야 합니다"));
    };
    if items.len() != 2 {
        return Err(std_block_piece_error(
            "블록조각 칸은 좌표 2개를 가져야 합니다",
        ));
    }
    Ok((value_to_i64(&items[0])?, value_to_i64(&items[1])?))
}

fn sort_std_block_piece_cells(cells: &mut Vec<(i64, i64)>) {
    cells.sort_by_key(|(x, y)| (*y, *x));
}

fn std_block_piece_cells_value(cells: &[(i64, i64)]) -> Value {
    Value::List(
        cells
            .iter()
            .map(|(x, y)| Value::List(vec![fixed_value(*x), fixed_value(*y)]))
            .collect(),
    )
}

fn std_block_piece_cells_from_value(value: &Value) -> Result<Vec<(i64, i64)>, EvalError> {
    let Value::List(items) = value else {
        return Err(std_block_piece_error(
            "블록조각 칸 목록은 차림이어야 합니다",
        ));
    };
    let mut cells = items
        .iter()
        .map(std_block_piece_cell_from_value)
        .collect::<Result<Vec<_>, _>>()?;
    sort_std_block_piece_cells(&mut cells);
    Ok(cells)
}

fn make_std_block_piece(mut cells: Vec<(i64, i64)>) -> Value {
    sort_std_block_piece_cells(&mut cells);
    let mut fields = BTreeMap::new();
    fields.insert(
        STD_BLOCK_PIECE_KIND_FIELD.to_string(),
        Value::String(STD_BLOCK_PIECE_KIND.to_string()),
    );
    fields.insert(
        STD_BLOCK_PIECE_CELLS_FIELD.to_string(),
        std_block_piece_cells_value(&cells),
    );
    Value::Pack(fields)
}

fn std_block_piece_cells(value: &Value) -> Result<Vec<(i64, i64)>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_block_piece_error("블록조각 인자가 필요합니다"));
    };
    match fields.get(STD_BLOCK_PIECE_KIND_FIELD) {
        Some(Value::String(kind)) if kind == STD_BLOCK_PIECE_KIND => {}
        _ => return Err(std_block_piece_error("블록조각 인자가 필요합니다")),
    }
    let cells = fields
        .get(STD_BLOCK_PIECE_CELLS_FIELD)
        .ok_or_else(|| std_block_piece_error("블록조각 칸 목록이 없습니다"))?;
    std_block_piece_cells_from_value(cells)
}

fn std_block_piece_collides(
    piece: &Value,
    grid: &Value,
    blocked_values: &Value,
) -> Result<bool, EvalError> {
    let cells = std_block_piece_cells(piece)?;
    let fields = std_grid_fields(grid)?;
    let grid_cells = std_grid_cells(fields)?;
    for (x, y) in cells {
        if !std_grid_inside(grid, x, y)? {
            return Ok(true);
        }
        let idx = std_grid_index(fields, x, y)?;
        let cell = grid_cells.get(idx).cloned().unwrap_or(Value::None);
        if value_matches_any(&cell, blocked_values) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn std_block_piece_lock(piece: &Value, grid: &Value, value: Value) -> Result<Value, EvalError> {
    let cells = std_block_piece_cells(piece)?;
    let fields = std_grid_fields(grid)?;
    let mut next = fields.clone();
    let mut grid_cells = std_grid_cells(fields)?.clone();
    for (x, y) in cells {
        let idx = std_grid_index(fields, x, y)?;
        if idx >= grid_cells.len() {
            return Err(std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"));
        }
        grid_cells[idx] = value.clone();
    }
    next.insert("칸들".to_string(), Value::List(grid_cells));
    Ok(Value::Pack(next))
}

fn splitmix64_next(state: u64) -> (u64, u64) {
    let next_state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = next_state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    (next_state, z ^ (z >> 31))
}

fn u64_state_text(value: u64) -> String {
    format!("0x{value:016x}")
}

fn parse_u64_state_text(text: &str) -> Result<u64, EvalError> {
    if let Some(hex) = text.strip_prefix("0x") {
        return u64::from_str_radix(hex, 16)
            .map_err(|_| std_random_bag_error("무작위가방 상태가 올바르지 않습니다"));
    }
    text.parse::<u64>()
        .map_err(|_| std_random_bag_error("무작위가방 상태가 올바르지 않습니다"))
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
        Value::String(STD_RANDOM_BAG_KIND.to_string()),
    );
    fields.insert(
        STD_RANDOM_BAG_SEED_FIELD.to_string(),
        fixed_value(seed as i64),
    );
    fields.insert(
        STD_RANDOM_BAG_STATE_FIELD.to_string(),
        Value::String(u64_state_text(state)),
    );
    fields.insert(
        STD_RANDOM_BAG_ORIGINAL_FIELD.to_string(),
        Value::List(original),
    );
    fields.insert(
        STD_RANDOM_BAG_REMAINING_FIELD.to_string(),
        Value::List(remaining),
    );
    fields.insert(STD_RANDOM_BAG_DRAWS_FIELD.to_string(), fixed_value(draws));
    Value::Pack(fields)
}

fn std_random_bag_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_random_bag_error("무작위가방 인자가 필요합니다"));
    };
    match fields.get(STD_RANDOM_BAG_KIND_FIELD) {
        Some(Value::String(kind)) if kind == STD_RANDOM_BAG_KIND => Ok(fields),
        _ => Err(std_random_bag_error("무작위가방 인자가 필요합니다")),
    }
}

fn std_random_bag_list_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<Vec<Value>, EvalError> {
    let Some(Value::List(items)) = fields.get(field) else {
        return Err(std_random_bag_error("무작위가방 차림 필드가 없습니다"));
    };
    Ok(items.clone())
}

fn std_random_bag_state(fields: &BTreeMap<String, Value>) -> Result<u64, EvalError> {
    match fields.get(STD_RANDOM_BAG_STATE_FIELD) {
        Some(Value::String(text)) => parse_u64_state_text(text),
        Some(value) => Ok(value_to_i64(value)? as u64),
        None => Err(std_random_bag_error("무작위가방 상태가 없습니다")),
    }
}

fn std_random_bag_seed(fields: &BTreeMap<String, Value>) -> Result<u64, EvalError> {
    fields
        .get(STD_RANDOM_BAG_SEED_FIELD)
        .ok_or_else(|| std_random_bag_error("무작위가방 시앗이 없습니다"))
        .and_then(value_to_i64)
        .map(|seed| seed as u64)
}

fn std_random_bag_draws(fields: &BTreeMap<String, Value>) -> Result<i64, EvalError> {
    fields
        .get(STD_RANDOM_BAG_DRAWS_FIELD)
        .ok_or_else(|| std_random_bag_error("무작위가방 뽑은수가 없습니다"))
        .and_then(value_to_i64)
}

fn std_random_bag_draw_once(bag: &Value) -> Result<(Value, Value), EvalError> {
    let fields = std_random_bag_fields(bag)?;
    let seed = std_random_bag_seed(fields)?;
    let original = std_random_bag_list_field(fields, STD_RANDOM_BAG_ORIGINAL_FIELD)?;
    if original.is_empty() {
        return Err(std_random_bag_error(
            "무작위가방 원본 후보들은 비어 있을 수 없습니다",
        ));
    }
    let mut remaining = std_random_bag_list_field(fields, STD_RANDOM_BAG_REMAINING_FIELD)?;
    if remaining.is_empty() {
        remaining = original.clone();
    }
    let (next_state, sample) = splitmix64_next(std_random_bag_state(fields)?);
    let idx = (sample % remaining.len() as u64) as usize;
    let value = remaining.remove(idx);
    let next_bag = make_std_random_bag(
        seed,
        next_state,
        original,
        remaining,
        std_random_bag_draws(fields)?.saturating_add(1),
    );
    Ok((value, next_bag))
}

fn is_allowed_grid_game_state(state: &str) -> bool {
    matches!(state, "준비" | "진행" | "잠금지연" | "정리" | "끝" | "멈춤")
}

fn value_to_grid_game_state_name(value: &Value) -> Result<String, EvalError> {
    let Value::String(state) = value else {
        return Err(std_grid_game_state_error(
            "격자게임상태 이름은 글이어야 합니다",
        ));
    };
    if !is_allowed_grid_game_state(state) {
        return Err(std_grid_game_state_error(&format!(
            "격자게임상태를 지원하지 않습니다: {state}"
        )));
    }
    Ok(state.clone())
}

fn make_std_grid_game_state(state: &str, previous: Value, tick: i64) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        STD_GRID_GAME_STATE_KIND_FIELD.to_string(),
        Value::String(STD_GRID_GAME_STATE_KIND.to_string()),
    );
    fields.insert(
        STD_GRID_GAME_STATE_STATE_FIELD.to_string(),
        Value::String(state.to_string()),
    );
    fields.insert(STD_GRID_GAME_STATE_PREV_FIELD.to_string(), previous);
    fields.insert(
        STD_GRID_GAME_STATE_TICK_FIELD.to_string(),
        fixed_value(tick),
    );
    Value::Pack(fields)
}

fn std_grid_game_state_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_state_error("격자게임상태 인자가 필요합니다"));
    };
    match fields.get(STD_GRID_GAME_STATE_KIND_FIELD) {
        Some(Value::String(kind)) if kind == STD_GRID_GAME_STATE_KIND => Ok(fields),
        _ => Err(std_grid_game_state_error("격자게임상태 인자가 필요합니다")),
    }
}

fn std_grid_game_state_state(fields: &BTreeMap<String, Value>) -> Result<String, EvalError> {
    let Some(Value::String(state)) = fields.get(STD_GRID_GAME_STATE_STATE_FIELD) else {
        return Err(std_grid_game_state_error("격자게임상태 상태가 없습니다"));
    };
    if !is_allowed_grid_game_state(state) {
        return Err(std_grid_game_state_error(&format!(
            "격자게임상태를 지원하지 않습니다: {state}"
        )));
    }
    Ok(state.clone())
}

fn std_grid_game_state_previous(fields: &BTreeMap<String, Value>) -> Value {
    fields
        .get(STD_GRID_GAME_STATE_PREV_FIELD)
        .cloned()
        .unwrap_or(Value::None)
}

fn std_grid_game_state_tick(fields: &BTreeMap<String, Value>) -> Result<i64, EvalError> {
    fields
        .get(STD_GRID_GAME_STATE_TICK_FIELD)
        .ok_or_else(|| std_grid_game_state_error("격자게임상태 틱이 없습니다"))
        .and_then(value_to_i64)
}

fn std_grid_game_error(message: &str) -> EvalError {
    EvalError::from(message.to_string())
}

fn tetromino_names() -> [&'static str; 7] {
    ["I", "O", "T", "S", "Z", "J", "L"]
}

fn tetromino_cells(name: &str) -> Result<Vec<(i64, i64)>, EvalError> {
    match name {
        "I" => Ok(vec![(-1, 0), (0, 0), (1, 0), (2, 0)]),
        "O" => Ok(vec![(0, 0), (1, 0), (0, 1), (1, 1)]),
        "T" => Ok(vec![(-1, 0), (0, 0), (1, 0), (0, 1)]),
        "S" => Ok(vec![(0, 0), (1, 0), (-1, 1), (0, 1)]),
        "Z" => Ok(vec![(-1, 0), (0, 0), (0, 1), (1, 1)]),
        "J" => Ok(vec![(-1, 0), (-1, 1), (0, 1), (1, 1)]),
        "L" => Ok(vec![(1, 0), (-1, 1), (0, 1), (1, 1)]),
        _ => Err(std_grid_game_error(&format!(
            "테트로미노를 지원하지 않습니다: {name}"
        ))),
    }
}

fn value_to_tetromino_name(value: &Value) -> Result<String, EvalError> {
    let Value::String(name) = value else {
        return Err(std_grid_game_error("테트로미노 이름은 글이어야 합니다"));
    };
    let _ = tetromino_cells(name)?;
    Ok(name.clone())
}

fn std_grid_line_full_rows(grid: &Value, empty_value: &Value) -> Result<Vec<i64>, EvalError> {
    let fields = std_grid_fields(grid)?;
    let (width, height) = std_grid_dims(fields)?;
    let cells = std_grid_cells(fields)?;
    let mut rows = Vec::new();
    for y in 0..height {
        let mut full = true;
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let cell = cells
                .get(idx)
                .ok_or_else(|| std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"))?;
            if values_equal(cell, empty_value) {
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

fn std_grid_line_clear(grid: &Value, empty_value: Value) -> Result<Value, EvalError> {
    let fields = std_grid_fields(grid)?;
    let (width, height) = std_grid_dims(fields)?;
    let cells = std_grid_cells(fields)?;
    let full_rows = std_grid_line_full_rows(grid, &empty_value)?;
    let full_set: BTreeSet<i64> = full_rows.iter().copied().collect();
    let mut kept_rows = Vec::new();
    for y in 0..height {
        if full_set.contains(&y) {
            continue;
        }
        let start = (y * width) as usize;
        let end = start + width as usize;
        kept_rows.extend_from_slice(
            cells
                .get(start..end)
                .ok_or_else(|| std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"))?,
        );
    }
    let mut next_cells = vec![empty_value.clone(); full_rows.len() * width as usize];
    next_cells.extend(kept_rows);
    let mut next_fields = fields.clone();
    next_fields.insert("칸들".to_string(), Value::List(next_cells));
    let mut result = BTreeMap::new();
    result.insert(
        "__종류".to_string(),
        Value::String(STD_GRID_LINE_CLEAR_KIND.to_string()),
    );
    result.insert("격자".to_string(), Value::Pack(next_fields));
    result.insert("지운줄수".to_string(), fixed_value(full_rows.len() as i64));
    result.insert(
        "지운줄목록".to_string(),
        Value::List(full_rows.into_iter().map(fixed_value).collect()),
    );
    Ok(Value::Pack(result))
}

fn make_std_falling_piece(piece: Value, x: i64, y: i64) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::String(STD_FALLING_PIECE_KIND.to_string()),
    );
    fields.insert("조각".to_string(), piece);
    fields.insert("x".to_string(), fixed_value(x));
    fields.insert("y".to_string(), fixed_value(y));
    Value::Pack(fields)
}

fn std_falling_piece_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_error("낙하조각 인자가 필요합니다"));
    };
    match fields.get("__종류") {
        Some(Value::String(kind)) if kind == STD_FALLING_PIECE_KIND => Ok(fields),
        _ => Err(std_grid_game_error("낙하조각 인자가 필요합니다")),
    }
}

fn std_falling_piece_parts(value: &Value) -> Result<(Value, i64, i64), EvalError> {
    let fields = std_falling_piece_fields(value)?;
    let piece = fields
        .get("조각")
        .cloned()
        .ok_or_else(|| std_grid_game_error("낙하조각 조각이 없습니다"))?;
    let x = fields
        .get("x")
        .ok_or_else(|| std_grid_game_error("낙하조각 x가 없습니다"))
        .and_then(value_to_i64)?;
    let y = fields
        .get("y")
        .ok_or_else(|| std_grid_game_error("낙하조각 y가 없습니다"))
        .and_then(value_to_i64)?;
    Ok((piece, x, y))
}

fn std_falling_piece_cells(value: &Value) -> Result<Vec<(i64, i64)>, EvalError> {
    let (piece, x, y) = std_falling_piece_parts(value)?;
    Ok(std_block_piece_cells(&piece)?
        .into_iter()
        .map(|(cx, cy)| (cx + x, cy + y))
        .collect())
}

fn std_falling_piece_block(value: &Value) -> Result<Value, EvalError> {
    Ok(make_std_block_piece(std_falling_piece_cells(value)?))
}

fn std_falling_piece_rotate(value: &Value, direction: &str) -> Result<Value, EvalError> {
    let (piece, x, y) = std_falling_piece_parts(value)?;
    let rotated_cells = std_block_piece_cells(&piece)?
        .into_iter()
        .map(|(cx, cy)| match direction {
            "오른쪽" => Ok((-cy, cx)),
            "왼쪽" => Ok((cy, -cx)),
            "뒤집기" => Ok((-cx, -cy)),
            _ => Err(std_grid_game_error(&format!(
                "낙하조각 회전 방향을 지원하지 않습니다: {direction}"
            ))),
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
        Value::String(STD_GRID_GAME_SCORE_KIND.to_string()),
    );
    fields.insert("점수".to_string(), fixed_value(score));
    fields.insert("줄수".to_string(), fixed_value(lines));
    fields.insert("레벨".to_string(), fixed_value(1 + lines.div_euclid(10)));
    Value::Pack(fields)
}

fn std_grid_game_score_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_error("격자게임점수 인자가 필요합니다"));
    };
    match fields.get("__종류") {
        Some(Value::String(kind)) if kind == STD_GRID_GAME_SCORE_KIND => Ok(fields),
        _ => Err(std_grid_game_error("격자게임점수 인자가 필요합니다")),
    }
}

fn std_score_int_field(fields: &BTreeMap<String, Value>, field: &str) -> Result<i64, EvalError> {
    fields
        .get(field)
        .ok_or_else(|| std_grid_game_error(&format!("격자게임점수 {field}가 없습니다")))
        .and_then(value_to_i64)
}

fn std_grid_game_score_add(score: &Value, cleared: i64) -> Result<Value, EvalError> {
    if !(0..=4).contains(&cleared) {
        return Err(std_grid_game_error("지운줄수는 0..4 범위여야 합니다"));
    }
    let fields = std_grid_game_score_fields(score)?;
    let current_score = std_score_int_field(fields, "점수")?;
    let current_lines = std_score_int_field(fields, "줄수")?;
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
        Value::String(STD_GRID_GAME_SESSION_KIND.to_string()),
    );
    fields.insert("격자".to_string(), grid);
    fields.insert("가방".to_string(), bag);
    fields.insert("상태".to_string(), state);
    fields.insert("점수".to_string(), score);
    fields.insert("낙하조각".to_string(), falling);
    Value::Pack(fields)
}

fn std_grid_game_session_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_error("격자게임세션 인자가 필요합니다"));
    };
    match fields.get("__종류") {
        Some(Value::String(kind)) if kind == STD_GRID_GAME_SESSION_KIND => Ok(fields),
        _ => Err(std_grid_game_error("격자게임세션 인자가 필요합니다")),
    }
}

fn std_session_field(fields: &BTreeMap<String, Value>, field: &str) -> Result<Value, EvalError> {
    fields
        .get(field)
        .cloned()
        .ok_or_else(|| std_grid_game_error(&format!("격자게임세션 {field}가 없습니다")))
}

fn std_grid_game_next_piece(bag: &Value) -> Result<Value, EvalError> {
    let (name_value, next_bag) = std_random_bag_draw_once(bag)?;
    let name = value_to_tetromino_name(&name_value)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::String(STD_GRID_GAME_NEXT_PIECE_KIND.to_string()),
    );
    fields.insert("이름".to_string(), Value::String(name.clone()));
    fields.insert(
        "조각".to_string(),
        make_std_block_piece(tetromino_cells(&name)?),
    );
    fields.insert("가방".to_string(), next_bag);
    Ok(Value::Pack(fields))
}

fn make_std_grid_game_hold(piece: Value, used: bool) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::String(STD_GRID_GAME_HOLD_KIND.to_string()),
    );
    fields.insert("조각".to_string(), piece);
    fields.insert("썼나".to_string(), Value::Bool(used));
    Value::Pack(fields)
}

fn std_grid_game_hold_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_error("격자게임홀드 인자가 필요합니다"));
    };
    match fields.get("__종류") {
        Some(Value::String(kind)) if kind == STD_GRID_GAME_HOLD_KIND => Ok(fields),
        _ => Err(std_grid_game_error("격자게임홀드 인자가 필요합니다")),
    }
}

fn std_grid_game_hold_piece(fields: &BTreeMap<String, Value>) -> Result<Value, EvalError> {
    let piece = fields.get("조각").cloned().unwrap_or(Value::None);
    if !matches!(piece, Value::None) {
        let _ = std_block_piece_cells(&piece)?;
    }
    Ok(piece)
}

fn std_grid_game_hold_used(fields: &BTreeMap<String, Value>) -> Result<bool, EvalError> {
    match fields.get("썼나") {
        Some(Value::Bool(flag)) => Ok(*flag),
        _ => Err(std_grid_game_error("격자게임홀드 썼나 값이 없습니다")),
    }
}

fn make_std_grid_game_hold_swap(hold: Value, falling: Value, bag: Value) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::String(STD_GRID_GAME_HOLD_SWAP_KIND.to_string()),
    );
    fields.insert("홀드".to_string(), hold);
    fields.insert("낙하조각".to_string(), falling);
    fields.insert("가방".to_string(), bag);
    Value::Pack(fields)
}

fn std_grid_game_hold_swap(
    hold: &Value,
    falling: &Value,
    bag: &Value,
    x: i64,
    y: i64,
) -> Result<Value, EvalError> {
    let hold_fields = std_grid_game_hold_fields(hold)?;
    if std_grid_game_hold_used(hold_fields)? {
        return Err(std_grid_game_error(
            "격자게임홀드는 이번 턴에 이미 썼습니다",
        ));
    }
    let (current_piece, _, _) = std_falling_piece_parts(falling)?;
    let held_piece = std_grid_game_hold_piece(hold_fields)?;
    if matches!(held_piece, Value::None) {
        let next = std_grid_game_next_piece(bag)?;
        let Value::Pack(next_fields) = &next else {
            unreachable!()
        };
        let next_piece = std_session_field(next_fields, "조각")?;
        let next_bag = std_session_field(next_fields, "가방")?;
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
) -> Result<bool, EvalError> {
    std_block_piece_collides(&std_falling_piece_block(falling)?, grid, blocked).map(|v| !v)
}

fn std_grid_game_rotation_try(
    falling: &Value,
    grid: &Value,
    blocked: &Value,
    direction: &str,
) -> Result<Value, EvalError> {
    let rotated = std_falling_piece_rotate(falling, direction)?;
    let (piece, x, y) = std_falling_piece_parts(&rotated)?;
    for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (-2, 0), (2, 0), (0, -1)] {
        let candidate = make_std_falling_piece(piece.clone(), x + dx, y + dy);
        if std_grid_game_placeable(&candidate, grid, blocked)? {
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
        Value::String(STD_GRID_GAME_ROTATION_TRY_KIND.to_string()),
    );
    fields.insert("낙하조각".to_string(), falling);
    fields.insert("성공".to_string(), Value::Bool(success));
    fields.insert(
        "오프셋".to_string(),
        Value::List(vec![fixed_value(dx), fixed_value(dy)]),
    );
    Value::Pack(fields)
}

fn std_grid_game_apply_input(
    falling: &Value,
    input_map: &Value,
    input: &InputState,
) -> Result<Value, EvalError> {
    let input_fields = std_input_map_fields(input_map)?;
    let mut candidate = falling.clone();
    if input_map_action_pressed(input, &input_fields, "왼쪽") {
        let (piece, x, y) = std_falling_piece_parts(&candidate)?;
        candidate = make_std_falling_piece(piece, x - 1, y);
    }
    if input_map_action_pressed(input, &input_fields, "오른쪽") {
        let (piece, x, y) = std_falling_piece_parts(&candidate)?;
        candidate = make_std_falling_piece(piece, x + 1, y);
    }
    if input_map_action_pressed(input, &input_fields, "아래") {
        let (piece, x, y) = std_falling_piece_parts(&candidate)?;
        candidate = make_std_falling_piece(piece, x, y + 1);
    }
    if input_map_action_pressed(input, &input_fields, "위")
        || input_map_action_pressed(input, &input_fields, "회전")
    {
        let (piece, x, y) = std_falling_piece_parts(&candidate)?;
        let rotated_cells = std_block_piece_cells(&piece)?
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
    input: &InputState,
) -> Result<Value, EvalError> {
    let fields = std_grid_game_session_fields(session)?;
    let grid = std_session_field(fields, "격자")?;
    let bag = std_session_field(fields, "가방")?;
    let state = std_session_field(fields, "상태")?;
    let score = std_session_field(fields, "점수")?;
    let falling = std_session_field(fields, "낙하조각")?;
    let state_name = std_grid_game_state_state(std_grid_game_state_fields(&state)?)?;
    if state_name == "멈춤" || state_name == "끝" {
        let mut tick_fields = BTreeMap::new();
        tick_fields.insert(
            "__종류".to_string(),
            Value::String(STD_GRID_GAME_TICK_KIND.to_string()),
        );
        tick_fields.insert("세션".to_string(), session.clone());
        tick_fields.insert("고정됐나".to_string(), Value::Bool(false));
        tick_fields.insert("지운줄수".to_string(), fixed_value(0));
        return Ok(Value::Pack(tick_fields));
    }

    let candidate = std_grid_game_apply_input(&falling, input_map, input)?;
    let blocked = Value::List(vec![Value::String("X".to_string())]);
    let active = if std_grid_game_placeable(&candidate, &grid, &blocked)? {
        candidate
    } else {
        falling
    };
    let (piece, x, y) = std_falling_piece_parts(&active)?;
    let gravity = make_std_falling_piece(piece, x, y + 1);
    let (next_session, locked, cleared) = if std_grid_game_placeable(&gravity, &grid, &blocked)? {
        (
            make_std_grid_game_session(grid, bag, state, score, gravity),
            false,
            0,
        )
    } else {
        let locked_grid = std_block_piece_lock(
            &std_falling_piece_block(&active)?,
            &grid,
            Value::String("X".to_string()),
        )?;
        let cleared_pack = std_grid_line_clear(&locked_grid, Value::String(".".to_string()))?;
        let Value::Pack(cleared_fields) = &cleared_pack else {
            unreachable!()
        };
        let next_grid = std_session_field(cleared_fields, "격자")?;
        let cleared_count = std_score_int_field(cleared_fields, "지운줄수")?;
        let next_score = std_grid_game_score_add(&score, cleared_count)?;
        let next_piece_pack = std_grid_game_next_piece(&bag)?;
        let Value::Pack(next_fields) = &next_piece_pack else {
            unreachable!()
        };
        let next_bag = std_session_field(next_fields, "가방")?;
        let next_piece = std_session_field(next_fields, "조각")?;
        let grid_fields = std_grid_fields(&next_grid)?;
        let (width, _) = std_grid_dims(grid_fields)?;
        let spawn = make_std_falling_piece(next_piece, width.div_euclid(2) - 1, 0);
        let next_state = if std_grid_game_placeable(&spawn, &next_grid, &blocked)? {
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
        Value::String(STD_GRID_GAME_TICK_KIND.to_string()),
    );
    tick_fields.insert("세션".to_string(), next_session);
    tick_fields.insert("고정됐나".to_string(), Value::Bool(locked));
    tick_fields.insert("지운줄수".to_string(), fixed_value(cleared));
    Ok(Value::Pack(tick_fields))
}

fn std_grid_game_view_cell(x: i64, y: i64, value: Value, source: &str) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("x".to_string(), fixed_value(x));
    fields.insert("y".to_string(), fixed_value(y));
    fields.insert("값".to_string(), value);
    fields.insert("원천".to_string(), Value::String(source.to_string()));
    Value::Pack(fields)
}

fn std_grid_game_view_project(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
) -> Result<(i64, i64, Vec<Value>), EvalError> {
    let session_fields = std_grid_game_session_fields(session)?;
    let grid = std_session_field(session_fields, "격자")?;
    let falling = std_session_field(session_fields, "낙하조각")?;
    let grid_fields = std_grid_fields(&grid)?;
    let (width, height) = std_grid_dims(grid_fields)?;
    let cells = std_grid_cells(grid_fields)?;
    let expected_len = width
        .checked_mul(height)
        .ok_or_else(|| std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"))?
        as usize;
    if cells.len() != expected_len {
        return Err(std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"));
    }

    let overlay = std_falling_piece_cells(&falling)?
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
                let source = if values_equal(&cell, empty_value) {
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
) -> Result<Value, EvalError> {
    let (_, _, cells) = std_grid_game_view_project(session, empty_value, falling_value)?;
    Ok(Value::List(cells))
}

fn std_grid_game_ghost_piece(session: &Value, blocked: &Value) -> Result<Value, EvalError> {
    let session_fields = std_grid_game_session_fields(session)?;
    let grid = std_session_field(session_fields, "격자")?;
    let falling = std_session_field(session_fields, "낙하조각")?;
    if !std_grid_game_placeable(&falling, &grid, blocked)? {
        return Ok(falling);
    }
    let mut current = falling;
    loop {
        let (piece, x, y) = std_falling_piece_parts(&current)?;
        let next = make_std_falling_piece(piece, x, y + 1);
        if std_grid_game_placeable(&next, &grid, blocked)? {
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
) -> Result<(i64, i64, Vec<Value>), EvalError> {
    let session_fields = std_grid_game_session_fields(session)?;
    let grid = std_session_field(session_fields, "격자")?;
    let falling = std_session_field(session_fields, "낙하조각")?;
    let ghost = std_grid_game_ghost_piece(session, blocked)?;
    let grid_fields = std_grid_fields(&grid)?;
    let (width, height) = std_grid_dims(grid_fields)?;
    let cells = std_grid_cells(grid_fields)?;
    let expected_len = width
        .checked_mul(height)
        .ok_or_else(|| std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"))?
        as usize;
    if cells.len() != expected_len {
        return Err(std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"));
    }

    let ghost_overlay = std_falling_piece_cells(&ghost)?
        .into_iter()
        .filter(|(x, y)| *x >= 0 && *y >= 0 && *x < width && *y < height)
        .collect::<BTreeSet<_>>();
    let falling_overlay = std_falling_piece_cells(&falling)?
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
                let source = if values_equal(&cell, empty_value) {
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
) -> Result<Value, EvalError> {
    let (width, height, cells) = std_grid_game_view_project(session, empty_value, falling_value)?;
    let mut rows = Vec::new();
    for y in 0..height {
        let mut row = String::new();
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let Value::Pack(fields) = &cells[idx] else {
                unreachable!()
            };
            if let Some(value) = fields.get("값") {
                row.push_str(&value_to_string(value));
            }
        }
        rows.push(row);
    }
    Ok(Value::String(rows.join("\n")))
}

fn std_grid_game_view_summary(session: &Value) -> Result<Value, EvalError> {
    let session_fields = std_grid_game_session_fields(session)?;
    let state = std_session_field(session_fields, "상태")?;
    let score = std_session_field(session_fields, "점수")?;
    let falling = std_session_field(session_fields, "낙하조각")?;
    let state_fields = std_grid_game_state_fields(&state)?;
    let score_fields = std_grid_game_score_fields(&score)?;
    let (_, x, y) = std_falling_piece_parts(&falling)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::String(STD_GRID_GAME_VIEW_SUMMARY_KIND.to_string()),
    );
    fields.insert(
        "상태".to_string(),
        Value::String(std_grid_game_state_state(state_fields)?),
    );
    fields.insert(
        "틱".to_string(),
        fixed_value(std_grid_game_state_tick(state_fields)?),
    );
    fields.insert(
        "점수".to_string(),
        fixed_value(std_score_int_field(score_fields, "점수")?),
    );
    fields.insert(
        "줄수".to_string(),
        fixed_value(std_score_int_field(score_fields, "줄수")?),
    );
    fields.insert(
        "레벨".to_string(),
        fixed_value(std_score_int_field(score_fields, "레벨")?),
    );
    fields.insert(
        "낙하조각위치".to_string(),
        Value::List(vec![fixed_value(x), fixed_value(y)]),
    );
    Ok(Value::Pack(fields))
}

fn std_grid_game_bogae_color(source: &str) -> &'static str {
    match source {
        "낙하" => "#ffcc00ff",
        "유령" => "#88ffffff",
        "고정" => "#4a90e2ff",
        _ => "#111111ff",
    }
}

fn std_grid_game_bogae_rects(cells: Vec<Value>, cell: i64) -> Result<Value, EvalError> {
    let mut items = Vec::with_capacity(cells.len());
    for value in cells {
        let Value::Pack(cell_fields) = value else {
            return Err(std_grid_error("격자게임보기 칸 묶음이 필요합니다"));
        };
        let x = cell_fields
            .get("x")
            .ok_or_else(|| std_grid_error("격자게임보기 칸 x가 없습니다"))
            .and_then(value_to_i64)?;
        let y = cell_fields
            .get("y")
            .ok_or_else(|| std_grid_error("격자게임보기 칸 y가 없습니다"))
            .and_then(value_to_i64)?;
        let source = match cell_fields.get("원천") {
            Some(Value::String(source)) => source.as_str(),
            _ => return Err(std_grid_error("격자게임보기 칸 원천이 없습니다")),
        };
        let mut fields = BTreeMap::new();
        fields.insert(
            "id".to_string(),
            Value::String(format!("격자게임셀_{y}_{x}")),
        );
        fields.insert("결".to_string(), Value::String("#보개/2D.Rect".to_string()));
        fields.insert("x".to_string(), fixed_value(x.saturating_mul(cell)));
        fields.insert("y".to_string(), fixed_value(y.saturating_mul(cell)));
        fields.insert("w".to_string(), fixed_value(cell));
        fields.insert("h".to_string(), fixed_value(cell));
        fields.insert(
            "채움색".to_string(),
            Value::String(std_grid_game_bogae_color(source).to_string()),
        );
        items.push(Value::Pack(fields));
    }
    Ok(Value::List(items))
}

fn std_grid_game_bogae_drawlist(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
    cell_size: &Value,
) -> Result<Value, EvalError> {
    let cell = value_to_i64(cell_size)?;
    if cell <= 0 {
        return Err(std_grid_error("보개 칸크기는 1 이상이어야 합니다"));
    }
    let (_, _, cells) = std_grid_game_view_project(session, empty_value, falling_value)?;
    std_grid_game_bogae_rects(cells, cell)
}

fn std_grid_game_ghost_bogae_drawlist(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
    ghost_value: Value,
    cell_size: &Value,
    blocked: &Value,
) -> Result<Value, EvalError> {
    let cell = value_to_i64(cell_size)?;
    if cell <= 0 {
        return Err(std_grid_error("보개 칸크기는 1 이상이어야 합니다"));
    }
    let (_, _, cells) = std_grid_game_ghost_view_project(
        session,
        empty_value,
        falling_value,
        ghost_value,
        blocked,
    )?;
    std_grid_game_bogae_rects(cells, cell)
}

fn std_grid_game_bogae_size(session: &Value, cell_size: &Value) -> Result<Value, EvalError> {
    let cell = value_to_i64(cell_size)?;
    if cell <= 0 {
        return Err(std_grid_error("보개 칸크기는 1 이상이어야 합니다"));
    }
    let session_fields = std_grid_game_session_fields(session)?;
    let grid = std_session_field(session_fields, "격자")?;
    let grid_fields = std_grid_fields(&grid)?;
    let (width, height) = std_grid_dims(grid_fields)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::String("std_grid_game_bogae_size".to_string()),
    );
    fields.insert("가로".to_string(), fixed_value(width.saturating_mul(cell)));
    fields.insert("세로".to_string(), fixed_value(height.saturating_mul(cell)));
    Ok(Value::Pack(fields))
}

fn value_matches_any(target: &Value, blocked: &Value) -> bool {
    match blocked {
        Value::List(items) => items.iter().any(|item| values_equal(target, item)),
        other => values_equal(target, other),
    }
}

fn std_grid_pathfind(
    grid: &Value,
    start_x: i64,
    start_y: i64,
    goal_x: i64,
    goal_y: i64,
    blocked_values: &Value,
) -> Result<Value, EvalError> {
    let fields = std_grid_fields(grid)?;
    let (width, height) = std_grid_dims(fields)?;
    let start = std_grid_index(fields, start_x, start_y)?;
    let goal = std_grid_index(fields, goal_x, goal_y)?;
    let cells = std_grid_cells(fields)?;
    if start >= cells.len() || goal >= cells.len() {
        return Err(std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"));
    }
    if value_matches_any(&cells[start], blocked_values)
        || value_matches_any(&cells[goal], blocked_values)
    {
        return Ok(Value::List(Vec::new()));
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
            if next >= cells.len()
                || visited[next]
                || value_matches_any(&cells[next], blocked_values)
            {
                continue;
            }
            visited[next] = true;
            prev[next] = Some(current);
            queue.push_back((nx, ny));
        }
    }

    if !visited[goal] {
        return Ok(Value::List(Vec::new()));
    }

    let mut route = Vec::new();
    let mut cursor = goal;
    loop {
        let x = (cursor as i64) % width;
        let y = (cursor as i64) / width;
        route.push(Value::List(vec![fixed_value(x), fixed_value(y)]));
        if cursor == start {
            break;
        }
        cursor =
            prev[cursor].ok_or_else(|| std_grid_error("격자 길찾기 경로를 복원할 수 없습니다"))?;
    }
    route.reverse();
    Ok(Value::List(route))
}

fn expect_unit_value(value: &Value) -> Result<UnitValue, EvalError> {
    match value {
        Value::Fixed64(raw) => Ok(UnitValue {
            value: *raw,
            dim: UnitDim::NONE,
        }),
        Value::Unit(unit) => Ok(*unit),
        _ => Err("수 값이 필요합니다".to_string().into()),
    }
}

fn unit_value_to_runtime_value(value: UnitValue) -> Value {
    if value.dim == UnitDim::NONE {
        Value::Fixed64(value.value)
    } else {
        Value::Unit(value)
    }
}

fn ensure_same_unit_dim(left: &UnitValue, right: &UnitValue) -> Result<(), EvalError> {
    if left.dim != right.dim {
        return Err("단위 차원이 맞지 않습니다".to_string().into());
    }
    Ok(())
}

fn ensure_dimensionless_unit(value: &UnitValue) -> Result<(), EvalError> {
    if value.dim != UnitDim::NONE {
        return Err("무차원 수가 필요합니다".to_string().into());
    }
    Ok(())
}

fn make_std_input_map(bindings: BTreeMap<String, Value>) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__kind".to_string(),
        Value::String(STD_INPUT_MAP_KIND.to_string()),
    );
    for (key, value) in bindings {
        fields.insert(key, value);
    }
    Value::Pack(fields)
}

fn std_input_map_fields(value: &Value) -> Result<BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(EvalError::from("입력사상 인자가 필요합니다".to_string()));
    };
    if matches!(fields.get("__kind"), Some(Value::String(kind)) if kind == STD_INPUT_MAP_KIND) {
        return Ok(fields.clone());
    }
    Ok(fields.clone())
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
            Value::String(text) => keys.push(text.clone()),
            Value::List(items) => {
                keys.extend(
                    items
                        .iter()
                        .map(value_to_string)
                        .filter(|key| !key.is_empty()),
                );
            }
            other => {
                let rendered = value_to_string(other);
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

fn input_map_action_pressed(
    input: &InputState,
    map: &BTreeMap<String, Value>,
    action: &str,
) -> bool {
    input_map_value_keys(map, action)
        .iter()
        .any(|key| matches!(input_pressed(input, key), Value::Bool(true)))
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
    top_level_decl_names: HashSet<String>,
    #[allow(dead_code)]
    file_meta: FileMeta,
    parse_warnings: Vec<DdnParseWarning>,
    configured_madi: Option<u64>,
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
        let top_level_decl_names = collect_top_level_decl_names(&meta_parse.stripped);
        let configured_madi = extract_setting_madi(&meta_parse.stripped)?;
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
            top_level_decl_names,
            file_meta: meta_parse.meta,
            parse_warnings,
            configured_madi,
        })
    }

    pub fn parse_warnings(&self) -> &[DdnParseWarning] {
        &self.parse_warnings
    }

    pub fn configured_madi(&self) -> Option<u64> {
        self.configured_madi
    }
}

fn collect_top_level_decl_names(source: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut depth = 0usize;
    let mut i = 0usize;
    let mut in_string = false;
    while i < source.len() {
        let rest = &source[i..];
        let Some(ch) = rest.chars().next() else {
            break;
        };
        if in_string {
            if ch == '"' && !source[..i].ends_with('\\') {
                in_string = false;
            }
            i += ch.len_utf8();
            continue;
        }
        if ch == '"' {
            in_string = true;
            i += ch.len_utf8();
            continue;
        }
        if depth == 0 && rest.starts_with("채비") && is_ident_boundary(source, i, "채비") {
            let mut cursor = i + "채비".len();
            cursor = skip_ws(source, cursor);
            if source[cursor..].starts_with(':') {
                cursor += ':'.len_utf8();
                cursor = skip_ws(source, cursor);
            }
            if source[cursor..].starts_with('{') {
                if let Some(close) = find_matching_brace(source, cursor) {
                    collect_decl_names_from_block(&source[cursor + 1..close], &mut names);
                    i = close + '}'.len_utf8();
                    continue;
                }
            }
        }
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        i += ch.len_utf8();
    }
    names
}

fn is_ident_boundary(source: &str, start: usize, keyword: &str) -> bool {
    let before = source[..start].chars().next_back();
    let after = source[start + keyword.len()..].chars().next();
    !before.is_some_and(is_ident_char) && !after.is_some_and(is_ident_char)
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

fn skip_ws(source: &str, mut idx: usize) -> usize {
    while idx < source.len() {
        let Some(ch) = source[idx..].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        idx += ch.len_utf8();
    }
    idx
}

fn find_matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut i = open;
    while i < source.len() {
        let ch = source[i..].chars().next()?;
        if in_string {
            if ch == '"' && !source[..i].ends_with('\\') {
                in_string = false;
            }
            i += ch.len_utf8();
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += ch.len_utf8();
    }
    None
}

fn collect_decl_names_from_block(block: &str, names: &mut HashSet<String>) {
    for line in block.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some(colon) = trimmed.find(':') else {
            continue;
        };
        let name = trimmed[..colon].trim();
        if !name.is_empty() && name.chars().all(is_ident_char) {
            names.insert(name.to_string());
        }
    }
}

pub fn extract_setting_madi(source: &str) -> Result<Option<u64>, String> {
    let mut search_start = 0;
    while let Some(rel_idx) = source[search_start..].find("설정") {
        let setting_start = search_start + rel_idx;
        let after_name = setting_start + "설정".len();
        let rest = &source[after_name..];
        let leading_ws = rest
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
            .map(|(idx, _)| idx)
            .unwrap_or(rest.len());
        let brace_idx = after_name + leading_ws;
        if source[brace_idx..].starts_with('{') {
            if let Some((body, end_idx)) = extract_braced_body(source, brace_idx) {
                if let Some(value) = parse_madi_from_setting_body(body)? {
                    return Ok(Some(value));
                }
                search_start = end_idx;
                continue;
            }
        }
        search_start = after_name;
    }
    Ok(None)
}

fn extract_braced_body(source: &str, open_brace_idx: usize) -> Option<(&str, usize)> {
    let mut depth = 0usize;
    let mut body_start = None;
    for (rel_idx, ch) in source[open_brace_idx..].char_indices() {
        let idx = open_brace_idx + rel_idx;
        match ch {
            '{' => {
                depth += 1;
                if body_start.is_none() {
                    body_start = Some(idx + ch.len_utf8());
                }
            }
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return body_start.map(|start| (&source[start..idx], idx + ch.len_utf8()));
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_madi_from_setting_body(body: &str) -> Result<Option<u64>, String> {
    let Some((_, after_key)) = find_setting_key(body, "마디수") else {
        return Ok(None);
    };
    let after_ws = after_key.trim_start();
    let after_colon = after_ws[1..].trim_start();
    let raw_value = after_colon
        .split(|ch| ch == '.' || ch == '\n' || ch == '\r' || ch == '}')
        .next()
        .unwrap_or("")
        .trim();
    if raw_value.is_empty()
        || raw_value.starts_with('-')
        || !raw_value.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(setting_madi_error());
    }
    let value = raw_value.parse::<u64>().map_err(|_| setting_madi_error())?;
    if value == 0 {
        return Err(setting_madi_error());
    }
    Ok(Some(value))
}

fn find_setting_key<'a>(body: &'a str, key: &str) -> Option<(usize, &'a str)> {
    let mut search_start = 0;
    while let Some(rel_idx) = body[search_start..].find(key) {
        let key_idx = search_start + rel_idx;
        let before = body[..key_idx].chars().rev().find(|ch| !ch.is_whitespace());
        let after_key = &body[key_idx + key.len()..];
        let after_ws = after_key.trim_start();
        let key_boundary = match before {
            Some(ch) => ch == '.' || ch == '{' || ch == '\n' || ch == '\r',
            None => true,
        };
        if key_boundary && after_ws.starts_with(':') {
            return Some((key_idx, after_ws));
        }
        search_start = key_idx + key.len();
    }
    None
}

fn setting_madi_error() -> String {
    "E_SETTING_MADI_BAD_VALUE 설정 마디수는 1 이상의 정수여야 합니다.".to_string()
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
        Stmt::Receive {
            condition, body, ..
        } => condition
            .as_ref()
            .and_then(expr_regex_feature)
            .or_else(|| body_regex_feature(body)),
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => sender
            .as_ref()
            .and_then(expr_regex_feature)
            .or_else(|| expr_regex_feature(payload))
            .or_else(|| expr_regex_feature(receiver)),
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
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => None,
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
        Stmt::Receive {
            condition, body, ..
        } => condition
            .as_ref()
            .and_then(expr_assertion_feature)
            .or_else(|| body_assertion_feature(body)),
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => sender
            .as_ref()
            .and_then(expr_assertion_feature)
            .or_else(|| expr_assertion_feature(payload))
            .or_else(|| expr_assertion_feature(receiver)),
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
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => None,
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
        Stmt::Receive {
            condition, body, ..
        } => condition
            .as_ref()
            .and_then(expr_state_machine_feature)
            .or_else(|| body_state_machine_feature(body)),
        Stmt::Send {
            sender,
            payload,
            receiver,
            ..
        } => sender
            .as_ref()
            .and_then(expr_state_machine_feature)
            .or_else(|| expr_state_machine_feature(payload))
            .or_else(|| expr_state_machine_feature(receiver)),
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
        Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => None,
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
        | Stmt::Send { .. }
        | Stmt::Expr { .. }
        | Stmt::Show { .. }
        | Stmt::Inspect { .. }
        | Stmt::MetaBlock { .. }
        | Stmt::Pragma { .. }
        | Stmt::Return { .. }
        | Stmt::Break { .. }
        | Stmt::ContinueLoop { .. } => None,
        Stmt::Receive { body, .. } => body_quantifier_feature(body),
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
        let update_name = self.update_name.clone();
        self.run_named_seed(world, input, defaults, &update_name, false)
    }

    pub fn run_finish(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
    ) -> Result<DdnRunOutput, String> {
        self.run_named_seed(world, input, defaults, "끝", true)
    }

    fn run_named_seed(
        &mut self,
        world: &NuriWorld,
        input: &InputSnapshot,
        defaults: &HashMap<String, Value>,
        seed_name: &str,
        missing_is_noop: bool,
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
        let update = if let Some(update) = ctx.program.functions.get(seed_name) {
            update
        } else if seed_name == "매마디" {
            ctx.program
                .functions
                .get("매틱")
                .ok_or_else(|| format!("업데이트 함수 '{}'를 찾을 수 없습니다", seed_name))?
        } else if missing_is_noop {
            return Ok(DdnRunOutput {
                patch: Patch {
                    ops: Vec::new(),
                    origin: Origin::system("ddn"),
                },
                resources: ctx.resources,
                state_transitions: Vec::new(),
            });
        } else {
            return Err(format!("업데이트 함수 '{}'를 찾을 수 없습니다", seed_name));
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
    pending_top_level_decl_names: HashSet<String>,
    factor_route_counts: BTreeMap<String, u64>,
    factor_bits_total: u128,
    factor_bits_min: Option<u64>,
    factor_bits_max: u64,
}

enum ThunkResult {
    Value(Value),
    Return(Value),
    ContinueLoop(ddonirang_lang::Span),
    Break(ddonirang_lang::Span),
}

enum FlowControl {
    Continue,
    Return(Value),
    ContinueLoop(ddonirang_lang::Span),
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
            pending_top_level_decl_names: program.top_level_decl_names.clone(),
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
                FlowControl::ContinueLoop(_) => {
                    return Err("건너뛰기는 반복 안에서만 사용할 수 있습니다"
                        .to_string()
                        .into())
                }
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
            ThunkResult::ContinueLoop(_) => Err("건너뛰기는 반복 안에서만 사용할 수 있습니다"
                .to_string()
                .into()),
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
                ThunkResult::ContinueLoop(span) => return Ok(ThunkResult::ContinueLoop(span)),
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
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Mutate { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Receive { .. } => Ok(ThunkResult::Value(Value::None)),
            Stmt::Send { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
                FlowControl::Break(span) => Ok(ThunkResult::Break(span)),
            },
            Stmt::Expr { expr, .. } | Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                Ok(ThunkResult::Value(self.eval_expr(locals, expr)?))
            }
            Stmt::MetaBlock { .. } => match self.eval_stmt(locals, stmt)? {
                FlowControl::Continue => Ok(ThunkResult::Value(Value::None)),
                FlowControl::Return(value) => Ok(ThunkResult::Return(value)),
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
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
                        ThunkResult::ContinueLoop(_) => continue,
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
                        ThunkResult::ContinueLoop(_) => continue,
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
                        ThunkResult::ContinueLoop(_) => continue,
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
            Stmt::ContinueLoop { span, .. } => Ok(ThunkResult::ContinueLoop(*span)),
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
                                FlowControl::ContinueLoop(span) => {
                                    return Ok(ThunkResult::ContinueLoop(span))
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
                                FlowControl::ContinueLoop(span) => {
                                    return Ok(ThunkResult::ContinueLoop(span))
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
                FlowControl::ContinueLoop(span) => Ok(ThunkResult::ContinueLoop(span)),
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
                    let is_top_level_decl =
                        self.pending_top_level_decl_names.remove(&item.name);
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
                    let value = if item.value.is_some()
                        && !is_top_level_decl
                        && self.resource_exists(&item.name)
                    {
                        self.get_resource(&item.name).unwrap_or(value)
                    } else {
                        value
                    };
                    locals.insert(item.name.clone(), value);
                    if item.value.is_some()
                        && (is_top_level_decl || !self.resource_exists(&item.name))
                    {
                        let initial = locals
                            .get(&item.name)
                            .cloned()
                            .unwrap_or(Value::None);
                        if !matches!(initial, Value::None) {
                            self.set_resource(&item.name, initial)?;
                        }
                    }
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
                            locals.insert(name.clone(), val.clone());
                            if self.resource_exists(name) {
                                self.set_resource(name, val)?;
                            }
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
            Stmt::Receive { .. } => Ok(FlowControl::Continue),
            Stmt::Send {
                sender,
                payload,
                receiver,
                ..
            } => {
                self.dispatch_signal_send(locals, sender.as_ref(), payload, receiver)?;
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
            Stmt::Show { expr, .. } | Stmt::Inspect { expr, .. } => {
                match self.eval_expr(locals, expr) {
                    Ok(value) => self.append_output_log(value)?,
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
                } else if matches!(kind, ddonirang_lang::MetaBlockKind::Boim) {
                    self.apply_boim_meta_block(locals, entries)?;
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
                        FlowControl::ContinueLoop(_) => continue,
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
                        FlowControl::ContinueLoop(_) => continue,
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
                        FlowControl::ContinueLoop(_) => continue,
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
            Stmt::ContinueLoop { span, .. } => Ok(FlowControl::ContinueLoop(*span)),
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

    fn dispatch_signal_send(
        &mut self,
        locals: &mut HashMap<String, Value>,
        sender_expr: Option<&Expr>,
        payload_expr: &Expr,
        receiver_expr: &Expr,
    ) -> Result<(), EvalError> {
        let receiver_name = self.resolve_entity_name(receiver_expr)?;
        let sender_name = sender_expr
            .map(|expr| self.resolve_entity_name(expr))
            .transpose()?
            .or_else(|| self.current_seed_name.clone())
            .unwrap_or_else(|| "누리".to_string());
        let (event_kind, event_payload) = self.build_signal_payload(locals, payload_expr)?;
        self.dispatch_signal_to_receiver(
            locals,
            &receiver_name,
            &event_kind,
            &sender_name,
            event_payload,
        )
    }

    fn resolve_entity_name(&self, expr: &Expr) -> Result<String, EvalError> {
        match &expr.kind {
            ExprKind::Var(name) if name == "제" => {
                self.current_seed_name.clone().ok_or_else(|| {
                    "E_RUNTIME_JE_OUTSIDE_IMJA: `제`는 임자 안에서만 사용할 수 있습니다"
                        .to_string()
                        .into()
                })
            }
            ExprKind::Var(name) => Ok(name.clone()),
            ExprKind::FieldAccess { target, field } => {
                let base = self.resolve_entity_name(target)?;
                Ok(format!("{base}.{field}"))
            }
            _ => Err("알림 대상은 임자 이름이어야 합니다".to_string().into()),
        }
    }

    fn build_signal_payload(
        &mut self,
        locals: &mut HashMap<String, Value>,
        payload_expr: &Expr,
    ) -> Result<(String, Value), EvalError> {
        let ExprKind::Call { func, args } = &payload_expr.kind else {
            let value = self.eval_expr(locals, payload_expr)?;
            return Ok(("알림".to_string(), value));
        };
        let Some(seed) = self.program.functions.get(func) else {
            let value = self.eval_expr(locals, payload_expr)?;
            return Ok((func.clone(), value));
        };
        if !matches!(&seed.seed_kind, SeedKind::Named(kind) if kind == "알림씨") {
            let value = self.eval_expr(locals, payload_expr)?;
            return Ok((func.clone(), value));
        }
        let mut fields = BTreeMap::new();
        for (index, arg) in args.iter().enumerate() {
            let name = arg
                .resolved_pin
                .clone()
                .or_else(|| seed.params.get(index).map(|param| param.pin_name.clone()))
                .unwrap_or_else(|| format!("값{index}"));
            fields.insert(name, self.eval_expr(locals, &arg.expr)?);
        }
        Ok((func.clone(), Value::Pack(fields)))
    }

    fn dispatch_signal_to_receiver(
        &mut self,
        locals: &mut HashMap<String, Value>,
        receiver_name: &str,
        event_kind: &str,
        sender_name: &str,
        event_payload: Value,
    ) -> Result<(), EvalError> {
        let Some(seed) = self.program.functions.get(receiver_name).cloned() else {
            return Err(format!("알림 대상 임자를 찾을 수 없습니다: {receiver_name}").into());
        };
        if !matches!(&seed.seed_kind, SeedKind::Named(kind) if kind == "임자") {
            return Err(format!("알림 대상은 임자여야 합니다: {receiver_name}").into());
        }
        let Some(body) = seed.body.as_ref() else {
            return Ok(());
        };
        let prev_seed = self.current_seed_name.clone();
        self.current_seed_name = Some(receiver_name.to_string());
        let result = (|| {
            for rank in 0..4 {
                for stmt in &body.stmts {
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
                    if !receive_stmt_matches_rank(
                        rank,
                        kind.as_deref(),
                        binding.as_ref(),
                        condition.as_ref(),
                        event_kind,
                    ) {
                        continue;
                    }
                    self.eval_receive_stmt(
                        locals,
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
            Ok(())
        })();
        self.current_seed_name = prev_seed;
        result
    }

    fn eval_receive_stmt(
        &mut self,
        locals: &mut HashMap<String, Value>,
        binding: Option<&str>,
        condition: Option<&Expr>,
        body: &Body,
        kind: Option<&str>,
        event_kind: &str,
        sender_name: &str,
        event_payload: &Value,
    ) -> Result<(), EvalError> {
        let bound_value = if kind.is_some() {
            event_payload.clone()
        } else {
            let mut fields = BTreeMap::new();
            fields.insert("이름".to_string(), Value::String(event_kind.to_string()));
            fields.insert("보낸이".to_string(), Value::String(sender_name.to_string()));
            fields.insert("정보".to_string(), event_payload.clone());
            Value::Pack(fields)
        };
        let restore = if let Some(name) = binding {
            Some((
                name.to_string(),
                locals.insert(name.to_string(), bound_value),
            ))
        } else {
            None
        };
        let should_run = condition
            .map(|expr| {
                self.eval_expr(locals, expr)
                    .and_then(|value| is_truthy(&value))
            })
            .transpose()?
            .unwrap_or(true);
        if should_run {
            let _ = self.eval_body(locals, body)?;
        }
        if let Some((name, previous)) = restore {
            if let Some(value) = previous {
                locals.insert(name, value);
            } else {
                locals.remove(&name);
            }
        }
        Ok(())
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

    fn apply_boim_meta_block(
        &mut self,
        locals: &mut HashMap<String, Value>,
        entries: &[String],
    ) -> Result<(), EvalError> {
        let mut evaluated = Vec::new();
        for entry in entries {
            let Some((raw_key, raw_expr)) = entry.split_once(':') else {
                return Err(format!(
                    "E_BOIM_BAD_ENTRY: 보임 항목은 `이름: 값.` 형태여야 합니다: {}",
                    entry
                )
                .into());
            };
            let key = raw_key.trim();
            let expr_text = raw_expr.trim();
            if key.is_empty() || expr_text.is_empty() {
                return Err(format!(
                    "E_BOIM_BAD_ENTRY: 보임 항목은 빈 이름/값을 허용하지 않습니다: {}",
                    entry
                )
                .into());
            }
            let expr = parse_boim_value_expr(expr_text)?;
            let value = self.eval_expr(locals, &expr)?;
            evaluated.push((key.to_string(), value));
        }
        if evaluated.is_empty() {
            return Ok(());
        }
        self.append_boim_output_rows(&evaluated)?;
        self.append_boim_graph_point(&evaluated)?;
        Ok(())
    }

    fn append_boim_output_rows(&mut self, rows: &[(String, Value)]) -> Result<(), EvalError> {
        let mut tokens = self.take_list_resource(BOGAE_SHOW_LINES_TAG);
        for (key, value) in rows {
            tokens.push(Value::String("table.row".to_string()));
            tokens.push(Value::String(key.clone()));
            tokens.push(Value::String(value_to_string(value)));
        }
        self.set_resource(BOGAE_SHOW_LINES_TAG, Value::List(tokens))
    }

    fn append_boim_graph_point(&mut self, rows: &[(String, Value)]) -> Result<(), EvalError> {
        let Some(x) = pick_boim_axis_value(rows, &["x축", "t", "시간", "tick"]) else {
            return Ok(());
        };
        let Some(y) = pick_boim_y_value(rows) else {
            return Ok(());
        };

        let mut points = self.take_list_resource(BOGAE_GRAPH_POINTS_F_TAG);
        let mut point = BTreeMap::new();
        insert_string_map_entry(&mut point, "x", Value::Fixed64(x));
        insert_string_map_entry(&mut point, "y", Value::Fixed64(y));
        points.push(Value::Map(point));
        self.set_resource(BOGAE_GRAPH_POINTS_F_TAG, Value::List(points))
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
        let params = lambda_param_names(&lambda.param);
        if args.len() != params.len() {
            return Err(format!("씨앗 인자는 {}개여야 합니다", params.len()).into());
        }
        let mut locals = lambda.captured.clone();
        for (param, arg) in params.iter().zip(args.iter()) {
            locals.insert(param.clone(), arg.clone());
        }
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
        let (state, value) = splitmix64_next(self.rng_state);
        self.rng_state = state;
        value
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
            "수" | "셈수" => {
                if args.len() != 1 {
                    return Err("셈수는 인자 1개를 받습니다".to_string().into());
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
                    return Err("나눔수는 인자 2개(분자, 분모)를 받습니다"
                        .to_string()
                        .into());
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
                    return Err(
                        "흐름.만들기는 인자 1개(용량) 또는 2개(용량, 초기값차림)를 받습니다"
                            .to_string()
                            .into(),
                    );
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
            "격자.만들기" => {
                if args.len() != 3 {
                    return Err("격자.만들기는 인자 3개를 받습니다".to_string().into());
                }
                let width = value_to_i64(&args[0])?;
                let height = value_to_i64(&args[1])?;
                make_std_grid(width, height, args[2].clone())
            }
            "격자.너비" => {
                if args.len() != 1 {
                    return Err("격자.너비는 인자 1개를 받습니다".to_string().into());
                }
                let fields = std_grid_fields(&args[0])?;
                let (width, _) = std_grid_dims(fields)?;
                Ok(fixed_value(width))
            }
            "격자.높이" => {
                if args.len() != 1 {
                    return Err("격자.높이는 인자 1개를 받습니다".to_string().into());
                }
                let fields = std_grid_fields(&args[0])?;
                let (_, height) = std_grid_dims(fields)?;
                Ok(fixed_value(height))
            }
            "격자.값" => {
                if args.len() != 3 {
                    return Err("격자.값은 인자 3개를 받습니다".to_string().into());
                }
                let fields = std_grid_fields(&args[0])?;
                let idx = std_grid_index(fields, value_to_i64(&args[1])?, value_to_i64(&args[2])?)?;
                let cells = std_grid_cells(fields)?;
                Ok(cells.get(idx).cloned().unwrap_or(Value::None))
            }
            "격자.바꾼값" => {
                if args.len() != 4 {
                    return Err("격자.바꾼값은 인자 4개를 받습니다".to_string().into());
                }
                let fields = std_grid_fields(&args[0])?;
                let idx = std_grid_index(fields, value_to_i64(&args[1])?, value_to_i64(&args[2])?)?;
                let mut next = fields.clone();
                let mut cells = std_grid_cells(fields)?.clone();
                if idx >= cells.len() {
                    return Err("격자 칸 목록이 크기와 맞지 않습니다".to_string().into());
                }
                cells[idx] = args[3].clone();
                next.insert("칸들".to_string(), Value::List(cells));
                Ok(Value::Pack(next))
            }
            "격자.안인가" => {
                if args.len() != 3 {
                    return Err("격자.안인가는 인자 3개를 받습니다".to_string().into());
                }
                Ok(Value::Bool(std_grid_inside(
                    &args[0],
                    value_to_i64(&args[1])?,
                    value_to_i64(&args[2])?,
                )?))
            }
            "격자.막혔나" => {
                if args.len() != 4 {
                    return Err("격자.막혔나는 인자 4개를 받습니다".to_string().into());
                }
                if !std_grid_inside(&args[0], value_to_i64(&args[1])?, value_to_i64(&args[2])?)? {
                    return Ok(Value::Bool(true));
                }
                let fields = std_grid_fields(&args[0])?;
                let idx = std_grid_index(fields, value_to_i64(&args[1])?, value_to_i64(&args[2])?)?;
                let cell = std_grid_cells(fields)?
                    .get(idx)
                    .cloned()
                    .unwrap_or(Value::None);
                Ok(Value::Bool(value_matches_any(&cell, &args[3])))
            }
            "격자.길찾기" => {
                if args.len() != 6 {
                    return Err("격자.길찾기는 인자 6개를 받습니다".to_string().into());
                }
                std_grid_pathfind(
                    &args[0],
                    value_to_i64(&args[1])?,
                    value_to_i64(&args[2])?,
                    value_to_i64(&args[3])?,
                    value_to_i64(&args[4])?,
                    &args[5],
                )
            }
            "블록조각.만들기" => {
                if args.len() != 1 {
                    return Err("블록조각.만들기는 인자 1개를 받습니다".to_string().into());
                }
                let cells = std_block_piece_cells_from_value(&args[0])?;
                Ok(make_std_block_piece(cells))
            }
            "블록조각.칸목록" => {
                if args.len() != 1 {
                    return Err("블록조각.칸목록은 인자 1개를 받습니다".to_string().into());
                }
                let cells = std_block_piece_cells(&args[0])?;
                Ok(std_block_piece_cells_value(&cells))
            }
            "블록조각.이동" => {
                if args.len() != 3 {
                    return Err("블록조각.이동은 인자 3개를 받습니다".to_string().into());
                }
                let cells = std_block_piece_cells(&args[0])?;
                let dx = value_to_i64(&args[1])?;
                let dy = value_to_i64(&args[2])?;
                Ok(make_std_block_piece(
                    cells.into_iter().map(|(x, y)| (x + dx, y + dy)).collect(),
                ))
            }
            "블록조각.회전" => {
                if args.len() != 2 {
                    return Err("블록조각.회전은 인자 2개를 받습니다".to_string().into());
                }
                let cells = std_block_piece_cells(&args[0])?;
                let Value::String(direction) = &args[1] else {
                    return Err("블록조각.회전은 글 방향을 받습니다".to_string().into());
                };
                let rotated = cells
                    .into_iter()
                    .map(|(x, y)| match direction.as_str() {
                        "오른쪽" => Ok((-y, x)),
                        "왼쪽" => Ok((y, -x)),
                        "뒤집기" => Ok((-x, -y)),
                        _ => Err(
                            format!("블록조각 회전 방향을 지원하지 않습니다: {direction}").into(),
                        ),
                    })
                    .collect::<Result<Vec<_>, EvalError>>()?;
                Ok(make_std_block_piece(rotated))
            }
            "블록조각.충돌?" => {
                if args.len() != 3 {
                    return Err("블록조각.충돌?은 인자 3개를 받습니다".to_string().into());
                }
                Ok(Value::Bool(std_block_piece_collides(
                    &args[0], &args[1], &args[2],
                )?))
            }
            "블록조각.고정" => {
                if args.len() != 3 {
                    return Err("블록조각.고정은 인자 3개를 받습니다".to_string().into());
                }
                std_block_piece_lock(&args[0], &args[1], args[2].clone())
            }
            "물리1d.위치갱신" => {
                if args.len() != 3 {
                    return Err("물리1d.위치갱신은 인자 3개를 받습니다".to_string().into());
                }
                let position = expect_unit_value(&args[0])?;
                let velocity = expect_unit_value(&args[1])?;
                let dt = expect_unit_value(&args[2])?;
                ensure_dimensionless_unit(&dt)?;
                ensure_same_unit_dim(&position, &velocity)?;
                Ok(unit_value_to_runtime_value(UnitValue {
                    value: position
                        .value
                        .saturating_add(velocity.value.saturating_mul(dt.value)),
                    dim: position.dim,
                }))
            }
            "물리1d.속도갱신" => {
                if args.len() != 3 {
                    return Err("물리1d.속도갱신은 인자 3개를 받습니다".to_string().into());
                }
                let velocity = expect_unit_value(&args[0])?;
                let acceleration = expect_unit_value(&args[1])?;
                let dt = expect_unit_value(&args[2])?;
                ensure_dimensionless_unit(&dt)?;
                ensure_same_unit_dim(&velocity, &acceleration)?;
                Ok(unit_value_to_runtime_value(UnitValue {
                    value: velocity
                        .value
                        .saturating_add(acceleration.value.saturating_mul(dt.value)),
                    dim: velocity.dim,
                }))
            }
            "물리1d.탄성충돌1d" => {
                if args.len() != 4 {
                    return Err("물리1d.탄성충돌1d는 인자 4개를 받습니다".to_string().into());
                }
                let m1 = expect_unit_value(&args[0])?;
                let v1 = expect_unit_value(&args[1])?;
                let m2 = expect_unit_value(&args[2])?;
                let v2 = expect_unit_value(&args[3])?;
                ensure_dimensionless_unit(&m1)?;
                ensure_dimensionless_unit(&m2)?;
                ensure_same_unit_dim(&v1, &v2)?;
                let denom = m1.value.saturating_add(m2.value);
                if denom.raw_i64() == 0 {
                    return Err("물리1d.탄성충돌1d 질량 합은 0이 될 수 없습니다"
                        .to_string()
                        .into());
                }
                let two = Fixed64::from_i64(2);
                let next_v1_num = m1
                    .value
                    .saturating_sub(m2.value)
                    .saturating_mul(v1.value)
                    .saturating_add(two.saturating_mul(m2.value).saturating_mul(v2.value));
                let next_v2_num = two
                    .saturating_mul(m1.value)
                    .saturating_mul(v1.value)
                    .saturating_add(m2.value.saturating_sub(m1.value).saturating_mul(v2.value));
                let next_v1 = next_v1_num
                    .try_div(denom)
                    .map_err(|_| EvalError::from("0으로 나눌 수 없습니다".to_string()))?;
                let next_v2 = next_v2_num
                    .try_div(denom)
                    .map_err(|_| EvalError::from("0으로 나눌 수 없습니다".to_string()))?;
                Ok(Value::List(vec![
                    unit_value_to_runtime_value(UnitValue {
                        value: next_v1,
                        dim: v1.dim,
                    }),
                    unit_value_to_runtime_value(UnitValue {
                        value: next_v2,
                        dim: v1.dim,
                    }),
                ]))
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
            "정리하기" => {
                let formula = expect_single_formula(&args, "정리하기")?;
                let transformed = transform_formula_value(
                    formula,
                    FormulaTransformOptions::default(),
                    "simplify",
                    "정리하기",
                )?;
                Ok(Value::Formula(transformed))
            }
            "전개하기" => {
                let formula = expect_single_formula(&args, "전개하기")?;
                let transformed = transform_formula_value(
                    formula,
                    FormulaTransformOptions::default(),
                    "expand",
                    "전개하기",
                )?;
                Ok(Value::Formula(transformed))
            }
            "인수분해하기" => {
                let formula = expect_single_formula(&args, "인수분해하기")?;
                let transformed = transform_formula_value(
                    formula,
                    FormulaTransformOptions::default(),
                    "factor",
                    "인수분해하기",
                )?;
                Ok(Value::Formula(transformed))
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
            "동치인가" => {
                let (left, right) = expect_two_formulas(&args, "동치인가")?;
                Ok(Value::Bool(symbolic_formulas_equivalent(&left, &right)?))
            }
            "잇기" => {
                let (left, right) = expect_two_formulas(&args, "잇기")?;
                Ok(make_relation_pack(left, right))
            }
            "이음관계.관계목록" => eval_endpoint_relation_list(&args),
            "이음관계.정규화" => eval_endpoint_relation_normalize(&args),
            "이음관계.방정식목록" => eval_endpoint_formula_relation_list(&args),
            "이음관계.방정식화" => eval_endpoint_formula_relation_set(&args),
            "이음관계.값관계목록" => eval_endpoint_boundary_value_relation_list(&args),
            "이음관계.값주입" => eval_endpoint_boundary_value_injection(&args),
            "이음관계.풀기" => eval_endpoint_explicit_solve(&args),
            "이음관계.풀이값목록" => eval_endpoint_solve_result_value_list(&args),
            "이음관계.풀이원복" => eval_endpoint_solve_result_remap(&args),
            "이음관계.범위위반목록" => eval_endpoint_boundary_range_violation_list(&args),
            "이음관계.범위검사" => eval_endpoint_boundary_range_check(&args),
            "이음관계.풀고범위위반목록" => {
                eval_endpoint_explicit_solve_range_violation_list(&args)
            }
            "이음관계.풀고범위검사" => eval_endpoint_explicit_solve_range_check(&args),
            "이음관계.풀고범위행목록" => eval_endpoint_solve_range_report_rows(&args),
            "이음관계.풀고범위보고서" => eval_endpoint_solve_range_report(&args),
            "이음관계.보고서문자표" => eval_endpoint_solve_range_text_report(&args),
            "이음관계.풀고범위문자표" => {
                eval_endpoint_explicit_solve_range_text_report(&args)
            }
            "이음관계.풀고범위케이스" => eval_endpoint_solve_range_case(&args),
            "이음관계.풀고범위스위트" => eval_endpoint_solve_range_case_suite(&args),
            "이음관계.풀고범위스위트문자표" => {
                eval_endpoint_solve_range_case_suite_text(&args)
            }
            "이음관계.풀고범위스위트상세문자표" => {
                eval_endpoint_solve_range_case_suite_detail_text(&args)
            }
            "이음관계.풀고범위실행상세문자표" => {
                eval_endpoint_solve_range_case_suite_run_detail_text(&args)
            }
            "이음관계.풀고범위스위트요약" => {
                eval_endpoint_solve_range_case_suite_summary(&args)
            }
            "이음관계.풀고범위실행요약" => {
                eval_endpoint_solve_range_case_suite_run_summary(&args)
            }
            "이음관계.풀고범위스위트판정" => {
                eval_endpoint_solve_range_case_suite_check(&args)
            }
            "이음관계.풀고범위실행판정" => {
                eval_endpoint_solve_range_case_suite_run_check(&args)
            }
            "방정식풀기" => {
                let relations = expect_equation_relations(&args)?;
                eval_relation_solve_result(&relations)
            }
            "다항식.풀기" => eval_polynomial_solve_result(&args),
            "증명하기" => {
                let proof = eval_symbolic_proof_tactic(&args)?;
                Ok(Value::Pack(proof))
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
                    let key =
                        self.eval_callable(locals, &func, vec![item.clone()], source_span.clone())?;
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
                    let verdict =
                        self.eval_callable(locals, &func, vec![item.clone()], source_span.clone())?;
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
                    acc =
                        self.eval_callable(locals, &func, vec![acc, item], source_span.clone())?;
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
            "수치해.이분법" => {
                let (formula, var_name, lower, upper, iterations) =
                    expect_numeric_bisection_args(&args, "수치해.이분법")?;
                let prepared = prepare_numeric_formula(&formula, &var_name, "수치해.이분법")?;
                let (root, residual, used_iterations) =
                    numeric_bisection_root(&prepared, lower, upper, iterations)?;
                Ok(Value::List(vec![
                    unit_value_to_value(root),
                    unit_value_to_value(residual),
                    unit_value_to_value(UnitValue {
                        value: Fixed64::from_i64(used_iterations as i64),
                        dim: UnitDim::NONE,
                    }),
                    Value::String("이분법".to_string()),
                ]))
            }
            "선형부등식.풀기" => eval_linear_inequality_solve(&args),
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
                if numeric_pack_kind(&args[0]) == Some(NUMERIC_KIND_FACTOR) {
                    let Some(base) = exact_numeric_from_value(&args[0])? else {
                        return Err("곱수 값이 필요합니다".to_string().into());
                    };
                    if !base.is_integer() {
                        return Err("곱수 거듭제곱은 정수 값만 지원합니다".to_string().into());
                    }
                    let exp = value_to_i64(&args[1])?;
                    if exp < 0 {
                        return Err("곱수 거듭제곱은 음수 지수를 지원하지 않습니다"
                            .to_string()
                            .into());
                    }
                    let raw = base.value.to_integer().pow(exp as u32);
                    let (value, status) = make_factor_value(&make_big_int_pack_from_bigint(&raw))?;
                    self.note_factor_route_from_value(&value);
                    if status == FACTOR_DECOMP_STATUS_DEFERRED {
                        self.emit_factor_decomposition_deferred_diag(&value, source_span);
                    }
                    return Ok(value);
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
            "입력사상.만들기" => {
                if args.is_empty() {
                    return Ok(make_std_input_map(BTreeMap::new()));
                }
                if args.len() != 1 {
                    return Err("입력사상.만들기는 인자 0개 또는 1개를 받습니다"
                        .to_string()
                        .into());
                }
                let Value::Pack(fields) = &args[0] else {
                    return Err("입력사상.만들기는 묶음 인자를 받습니다".to_string().into());
                };
                Ok(make_std_input_map(fields.clone()))
            }
            "입력사상.방향" => {
                if args.len() != 1 {
                    return Err("입력사상.방향은 인자 1개를 받습니다".to_string().into());
                }
                let map = std_input_map_fields(&args[0])?;
                let left = input_map_action_pressed(&self.input, &map, "왼쪽") as i64;
                let right = input_map_action_pressed(&self.input, &map, "오른쪽") as i64;
                let up = input_map_action_pressed(&self.input, &map, "위") as i64;
                let down = input_map_action_pressed(&self.input, &map, "아래") as i64;
                Ok(Value::List(vec![
                    fixed_value(right - left),
                    fixed_value(down - up),
                ]))
            }
            "입력사상.동작" => {
                if args.len() != 2 {
                    return Err("입력사상.동작은 인자 2개를 받습니다".to_string().into());
                }
                let map = std_input_map_fields(&args[0])?;
                let action = value_to_string(&args[1]);
                Ok(Value::Bool(input_map_action_pressed(
                    &self.input,
                    &map,
                    &action,
                )))
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
            "무작위가방.만들기" => {
                if args.len() != 2 {
                    return Err("무작위가방.만들기는 인자 2개를 받습니다".to_string().into());
                }
                let seed = value_to_i64(&args[0])? as u64;
                let Value::List(candidates) = &args[1] else {
                    return Err("무작위가방.만들기는 후보 차림을 받습니다"
                        .to_string()
                        .into());
                };
                if candidates.is_empty() {
                    return Err("무작위가방 후보들은 비어 있을 수 없습니다"
                        .to_string()
                        .into());
                }
                Ok(make_std_random_bag(
                    seed,
                    seed,
                    candidates.clone(),
                    candidates.clone(),
                    0,
                ))
            }
            "무작위가방.꺼내기" => {
                if args.len() != 1 {
                    return Err("무작위가방.꺼내기는 인자 1개를 받습니다".to_string().into());
                }
                let (value, bag) = std_random_bag_draw_once(&args[0])?;
                let mut fields = BTreeMap::new();
                fields.insert(
                    STD_RANDOM_BAG_KIND_FIELD.to_string(),
                    Value::String(STD_RANDOM_BAG_DRAW_KIND.to_string()),
                );
                fields.insert(STD_RANDOM_BAG_VALUE_FIELD.to_string(), value);
                fields.insert(STD_RANDOM_BAG_BAG_FIELD.to_string(), bag);
                Ok(Value::Pack(fields))
            }
            "무작위가방.미리보기" => {
                if args.len() != 2 {
                    return Err("무작위가방.미리보기는 인자 2개를 받습니다"
                        .to_string()
                        .into());
                }
                let count = value_to_i64(&args[1])?;
                if count < 0 {
                    return Err("무작위가방.미리보기 개수는 0 이상이어야 합니다"
                        .to_string()
                        .into());
                }
                let mut bag = args[0].clone();
                let mut values = Vec::new();
                for _ in 0..count {
                    let (value, next_bag) = std_random_bag_draw_once(&bag)?;
                    values.push(value);
                    bag = next_bag;
                }
                Ok(Value::List(values))
            }
            "무작위가방.남은것" => {
                if args.len() != 1 {
                    return Err("무작위가방.남은것은 인자 1개를 받습니다".to_string().into());
                }
                let fields = std_random_bag_fields(&args[0])?;
                Ok(Value::List(std_random_bag_list_field(
                    fields,
                    STD_RANDOM_BAG_REMAINING_FIELD,
                )?))
            }
            "무작위가방.비었나" => {
                if args.len() != 1 {
                    return Err("무작위가방.비었나는 인자 1개를 받습니다".to_string().into());
                }
                let fields = std_random_bag_fields(&args[0])?;
                Ok(Value::Bool(
                    std_random_bag_list_field(fields, STD_RANDOM_BAG_REMAINING_FIELD)?.is_empty(),
                ))
            }
            "격자게임상태.초기화" => {
                if !args.is_empty() {
                    return Err("격자게임상태.초기화는 인자 0개를 받습니다"
                        .to_string()
                        .into());
                }
                Ok(make_std_grid_game_state("준비", Value::None, 0))
            }
            "격자게임상태.만들기" => {
                if args.len() != 1 {
                    return Err("격자게임상태.만들기는 인자 1개를 받습니다"
                        .to_string()
                        .into());
                }
                let state = value_to_grid_game_state_name(&args[0])?;
                Ok(make_std_grid_game_state(&state, Value::None, 0))
            }
            "격자게임상태.상태" => {
                if args.len() != 1 {
                    return Err("격자게임상태.상태는 인자 1개를 받습니다".to_string().into());
                }
                Ok(Value::String(std_grid_game_state_state(
                    std_grid_game_state_fields(&args[0])?,
                )?))
            }
            "격자게임상태.틱" => {
                if args.len() != 1 {
                    return Err("격자게임상태.틱은 인자 1개를 받습니다".to_string().into());
                }
                Ok(fixed_value(std_grid_game_state_tick(
                    std_grid_game_state_fields(&args[0])?,
                )?))
            }
            "격자게임상태.상태인가" => {
                if args.len() != 2 {
                    return Err("격자게임상태.상태인가는 인자 2개를 받습니다"
                        .to_string()
                        .into());
                }
                let current = std_grid_game_state_state(std_grid_game_state_fields(&args[0])?)?;
                let expected = value_to_grid_game_state_name(&args[1])?;
                Ok(Value::Bool(current == expected))
            }
            "격자게임상태.바꾸기" => {
                if args.len() != 2 {
                    return Err("격자게임상태.바꾸기는 인자 2개를 받습니다"
                        .to_string()
                        .into());
                }
                let fields = std_grid_game_state_fields(&args[0])?;
                let state = value_to_grid_game_state_name(&args[1])?;
                Ok(make_std_grid_game_state(
                    &state,
                    Value::String(std_grid_game_state_state(fields)?),
                    std_grid_game_state_tick(fields)?.saturating_add(1),
                ))
            }
            "격자게임상태.멈춤" => {
                if args.len() != 1 {
                    return Err("격자게임상태.멈춤은 인자 1개를 받습니다".to_string().into());
                }
                let fields = std_grid_game_state_fields(&args[0])?;
                let state = std_grid_game_state_state(fields)?;
                let previous = if state == "멈춤" {
                    std_grid_game_state_previous(fields)
                } else {
                    Value::String(state)
                };
                Ok(make_std_grid_game_state(
                    "멈춤",
                    previous,
                    std_grid_game_state_tick(fields)?.saturating_add(1),
                ))
            }
            "격자게임상태.재개" => {
                if args.len() != 1 {
                    return Err("격자게임상태.재개는 인자 1개를 받습니다".to_string().into());
                }
                let fields = std_grid_game_state_fields(&args[0])?;
                let state = std_grid_game_state_state(fields)?;
                if state != "멈춤" {
                    return Ok(args[0].clone());
                }
                let previous = match std_grid_game_state_previous(fields) {
                    Value::String(candidate)
                        if is_allowed_grid_game_state(&candidate) && candidate != "멈춤" =>
                    {
                        candidate
                    }
                    _ => "진행".to_string(),
                };
                Ok(make_std_grid_game_state(
                    &previous,
                    Value::None,
                    std_grid_game_state_tick(fields)?.saturating_add(1),
                ))
            }
            "테트로미노.이름목록" => {
                if !args.is_empty() {
                    return Err("테트로미노.이름목록은 인자 0개를 받습니다"
                        .to_string()
                        .into());
                }
                Ok(Value::List(
                    tetromino_names()
                        .into_iter()
                        .map(|name| Value::String(name.to_string()))
                        .collect(),
                ))
            }
            "테트로미노.만들기" => {
                if args.len() != 1 {
                    return Err("테트로미노.만들기는 인자 1개를 받습니다".to_string().into());
                }
                let name = value_to_tetromino_name(&args[0])?;
                Ok(make_std_block_piece(tetromino_cells(&name)?))
            }
            "테트로미노.목록" => {
                if !args.is_empty() {
                    return Err("테트로미노.목록은 인자 0개를 받습니다".to_string().into());
                }
                Ok(Value::List(
                    tetromino_names()
                        .into_iter()
                        .map(|name| {
                            make_std_block_piece(tetromino_cells(name).expect("known tetromino"))
                        })
                        .collect(),
                ))
            }
            "격자줄.찬줄목록" => {
                if args.len() != 2 {
                    return Err("격자줄.찬줄목록은 인자 2개를 받습니다".to_string().into());
                }
                Ok(Value::List(
                    std_grid_line_full_rows(&args[0], &args[1])?
                        .into_iter()
                        .map(fixed_value)
                        .collect(),
                ))
            }
            "격자줄.지우기" => {
                if args.len() != 2 {
                    return Err("격자줄.지우기는 인자 2개를 받습니다".to_string().into());
                }
                std_grid_line_clear(&args[0], args[1].clone())
            }
            "낙하조각.만들기" | "격자게임.스폰" => {
                if args.len() != 3 {
                    return Err("낙하조각.만들기는 인자 3개를 받습니다".to_string().into());
                }
                let _ = std_block_piece_cells(&args[0])?;
                let x = value_to_i64(&args[1])?;
                let y = value_to_i64(&args[2])?;
                Ok(make_std_falling_piece(args[0].clone(), x, y))
            }
            "낙하조각.조각" => {
                if args.len() != 1 {
                    return Err("낙하조각.조각은 인자 1개를 받습니다".to_string().into());
                }
                let (piece, _, _) = std_falling_piece_parts(&args[0])?;
                Ok(piece)
            }
            "낙하조각.위치" => {
                if args.len() != 1 {
                    return Err("낙하조각.위치는 인자 1개를 받습니다".to_string().into());
                }
                let (_, x, y) = std_falling_piece_parts(&args[0])?;
                Ok(Value::List(vec![fixed_value(x), fixed_value(y)]))
            }
            "낙하조각.배치" => {
                if args.len() != 1 {
                    return Err("낙하조각.배치는 인자 1개를 받습니다".to_string().into());
                }
                Ok(std_block_piece_cells_value(&std_falling_piece_cells(
                    &args[0],
                )?))
            }
            "낙하조각.이동" => {
                if args.len() != 3 {
                    return Err("낙하조각.이동은 인자 3개를 받습니다".to_string().into());
                }
                let (piece, x, y) = std_falling_piece_parts(&args[0])?;
                Ok(make_std_falling_piece(
                    piece,
                    x + value_to_i64(&args[1])?,
                    y + value_to_i64(&args[2])?,
                ))
            }
            "낙하조각.회전" => {
                if args.len() != 2 {
                    return Err("낙하조각.회전은 인자 2개를 받습니다".to_string().into());
                }
                let Value::String(direction) = &args[1] else {
                    return Err("낙하조각 회전 방향은 글이어야 합니다".to_string().into());
                };
                std_falling_piece_rotate(&args[0], direction)
            }
            "격자게임.놓을수있나" => {
                if args.len() != 3 {
                    return Err("격자게임.놓을수있나는 인자 3개를 받습니다"
                        .to_string()
                        .into());
                }
                Ok(Value::Bool(std_grid_game_placeable(
                    &args[0], &args[1], &args[2],
                )?))
            }
            "격자게임.입력적용" => {
                if args.len() != 2 {
                    return Err("격자게임.입력적용은 인자 2개를 받습니다".to_string().into());
                }
                std_grid_game_apply_input(&args[0], &args[1], &self.input)
            }
            "격자게임.중력틱" => {
                if args.len() != 1 {
                    return Err("격자게임.중력틱은 인자 1개를 받습니다".to_string().into());
                }
                let (piece, x, y) = std_falling_piece_parts(&args[0])?;
                Ok(make_std_falling_piece(piece, x, y + 1))
            }
            "격자게임.고정" => {
                if args.len() != 3 {
                    return Err("격자게임.고정은 인자 3개를 받습니다".to_string().into());
                }
                std_block_piece_lock(
                    &std_falling_piece_block(&args[0])?,
                    &args[1],
                    args[2].clone(),
                )
            }
            "격자게임.다음조각" => {
                if args.len() != 1 {
                    return Err("격자게임.다음조각은 인자 1개를 받습니다".to_string().into());
                }
                std_grid_game_next_piece(&args[0])
            }
            "격자게임.한틱" => {
                if args.len() != 2 {
                    return Err("격자게임.한틱은 인자 2개를 받습니다".to_string().into());
                }
                std_grid_game_tick(&args[0], &args[1], &self.input)
            }
            "격자게임.회전시도" => {
                if args.len() != 4 {
                    return Err("격자게임.회전시도는 인자 4개를 받습니다".to_string().into());
                }
                let Value::String(direction) = &args[3] else {
                    return Err("격자게임.회전시도 방향은 글이어야 합니다"
                        .to_string()
                        .into());
                };
                std_grid_game_rotation_try(&args[0], &args[1], &args[2], direction)
            }
            "격자게임홀드.초기화" => {
                if !args.is_empty() {
                    return Err("격자게임홀드.초기화는 인자 0개를 받습니다"
                        .to_string()
                        .into());
                }
                Ok(make_std_grid_game_hold(Value::None, false))
            }
            "격자게임홀드.칸" => {
                if args.len() != 1 {
                    return Err("격자게임홀드.칸은 인자 1개를 받습니다".to_string().into());
                }
                std_grid_game_hold_piece(std_grid_game_hold_fields(&args[0])?)
            }
            "격자게임홀드.썼나" => {
                if args.len() != 1 {
                    return Err("격자게임홀드.썼나는 인자 1개를 받습니다".to_string().into());
                }
                Ok(Value::Bool(std_grid_game_hold_used(
                    std_grid_game_hold_fields(&args[0])?,
                )?))
            }
            "격자게임홀드.교체" => {
                if args.len() != 5 {
                    return Err("격자게임홀드.교체는 인자 5개를 받습니다".to_string().into());
                }
                std_grid_game_hold_swap(
                    &args[0],
                    &args[1],
                    &args[2],
                    value_to_i64(&args[3])?,
                    value_to_i64(&args[4])?,
                )
            }
            "격자게임홀드.초기화턴" => {
                if args.len() != 1 {
                    return Err("격자게임홀드.초기화턴은 인자 1개를 받습니다"
                        .to_string()
                        .into());
                }
                let piece = std_grid_game_hold_piece(std_grid_game_hold_fields(&args[0])?)?;
                Ok(make_std_grid_game_hold(piece, false))
            }
            "격자게임점수.초기화" => {
                if !args.is_empty() {
                    return Err("격자게임점수.초기화는 인자 0개를 받습니다"
                        .to_string()
                        .into());
                }
                Ok(make_std_grid_game_score(0, 0))
            }
            "격자게임점수.더하기" => {
                if args.len() != 2 {
                    return Err("격자게임점수.더하기는 인자 2개를 받습니다"
                        .to_string()
                        .into());
                }
                std_grid_game_score_add(&args[0], value_to_i64(&args[1])?)
            }
            "격자게임점수.점수" | "격자게임점수.줄수" | "격자게임점수.레벨" =>
            {
                if args.len() != 1 {
                    return Err("격자게임점수 접근자는 인자 1개를 받습니다"
                        .to_string()
                        .into());
                }
                let field = match func {
                    "격자게임점수.점수" => "점수",
                    "격자게임점수.줄수" => "줄수",
                    _ => "레벨",
                };
                Ok(fixed_value(std_score_int_field(
                    std_grid_game_score_fields(&args[0])?,
                    field,
                )?))
            }
            "격자게임세션.만들기" => {
                if args.len() != 5 {
                    return Err("격자게임세션.만들기는 인자 5개를 받습니다"
                        .to_string()
                        .into());
                }
                Ok(make_std_grid_game_session(
                    args[0].clone(),
                    args[1].clone(),
                    args[2].clone(),
                    args[3].clone(),
                    args[4].clone(),
                ))
            }
            "격자게임세션.격자"
            | "격자게임세션.가방"
            | "격자게임세션.상태"
            | "격자게임세션.점수"
            | "격자게임세션.낙하조각" => {
                if args.len() != 1 {
                    return Err("격자게임세션 접근자는 인자 1개를 받습니다"
                        .to_string()
                        .into());
                }
                let field = match func {
                    "격자게임세션.격자" => "격자",
                    "격자게임세션.가방" => "가방",
                    "격자게임세션.상태" => "상태",
                    "격자게임세션.점수" => "점수",
                    _ => "낙하조각",
                };
                std_session_field(std_grid_game_session_fields(&args[0])?, field)
            }
            "격자게임세션.바꾸기" => {
                if args.len() != 6 {
                    return Err("격자게임세션.바꾸기는 인자 6개를 받습니다"
                        .to_string()
                        .into());
                }
                let _ = std_grid_game_session_fields(&args[0])?;
                Ok(make_std_grid_game_session(
                    args[1].clone(),
                    args[2].clone(),
                    args[3].clone(),
                    args[4].clone(),
                    args[5].clone(),
                ))
            }
            "격자게임보기.칸목록" => {
                if args.len() != 3 {
                    return Err("격자게임보기.칸목록은 인자 3개를 받습니다"
                        .to_string()
                        .into());
                }
                std_grid_game_view_cells(&args[0], &args[1], args[2].clone())
            }
            "격자게임보기.문자판" => {
                if args.len() != 3 {
                    return Err("격자게임보기.문자판은 인자 3개를 받습니다"
                        .to_string()
                        .into());
                }
                std_grid_game_view_text(&args[0], &args[1], args[2].clone())
            }
            "격자게임보기.상태요약" => {
                if args.len() != 1 {
                    return Err("격자게임보기.상태요약은 인자 1개를 받습니다"
                        .to_string()
                        .into());
                }
                std_grid_game_view_summary(&args[0])
            }
            "격자게임보기.유령조각" => {
                if args.len() != 2 {
                    return Err("격자게임보기.유령조각은 인자 2개를 받습니다"
                        .to_string()
                        .into());
                }
                std_grid_game_ghost_piece(&args[0], &args[1])
            }
            "격자게임보기.유령보개목록" => {
                if args.len() != 6 {
                    return Err("격자게임보기.유령보개목록은 인자 6개를 받습니다"
                        .to_string()
                        .into());
                }
                std_grid_game_ghost_bogae_drawlist(
                    &args[0],
                    &args[1],
                    args[2].clone(),
                    args[3].clone(),
                    &args[4],
                    &args[5],
                )
            }
            "격자게임보기.보개목록" => {
                if args.len() != 4 {
                    return Err("격자게임보기.보개목록은 인자 4개를 받습니다"
                        .to_string()
                        .into());
                }
                std_grid_game_bogae_drawlist(&args[0], &args[1], args[2].clone(), &args[3])
            }
            "격자게임보기.보개크기" => {
                if args.len() != 2 {
                    return Err("격자게임보기.보개크기는 인자 2개를 받습니다"
                        .to_string()
                        .into());
                }
                std_grid_game_bogae_size(&args[0], &args[1])
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
            "relation_eq" | "=:=" => {
                let Value::Formula(left_formula) = left else {
                    return Err("방정식 관계의 왼쪽은 수식값이어야 합니다"
                        .to_string()
                        .into());
                };
                let Value::Formula(right_formula) = right else {
                    return Err("방정식 관계의 오른쪽은 수식값이어야 합니다"
                        .to_string()
                        .into());
                };
                Ok(make_relation_pack(left_formula, right_formula))
            }
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
                ddonirang_lang::ContractMode::Abort => "물림".to_string(),
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
                let reason =
                    fields
                        .get(FACTOR_DECOMP_DEFERRED_REASON_KEY)
                        .and_then(|raw| match raw {
                            Value::String(text) => Some(text.clone()),
                            _ => None,
                        });
                let route = fields
                    .get(FACTOR_DECOMP_ROUTE_KEY)
                    .and_then(|raw| match raw {
                        Value::String(text) => Some(text.clone()),
                        _ => None,
                    });
                let bits = fields
                    .get(FACTOR_DECOMP_BITS_KEY)
                    .and_then(|raw| match raw {
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
                let route = fields
                    .get(FACTOR_DECOMP_ROUTE_KEY)
                    .and_then(|raw| match raw {
                        Value::String(route) => Some(route.clone()),
                        _ => None,
                    });
                let bits = fields
                    .get(FACTOR_DECOMP_BITS_KEY)
                    .and_then(|raw| match raw {
                        Value::Fixed64(value) if value.int_part() >= 0 => {
                            Some(value.int_part() as u64)
                        }
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
        let Some((summary, total, bit_min, bit_max, bit_sum)) =
            self.factor_route_summary_components()
        else {
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
        let Some((summary, total, bit_min, bit_max, bit_sum)) =
            self.factor_route_summary_components()
        else {
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
                } if event.rule_id == "L0-CONTRACT-01" && event.mode.as_deref() == Some("물림") => {
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
            | Value::StateMachine(_)
            | Value::Formula(_) => {
                if parse_resource_unit_tag(name).is_some() {
                    return Err(
                        "단위 태그 자원에는 차림/모음/짝맞춤/묶음/세움값/상태머신값/수식값을 저장할 수 없습니다"
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
            Value::Template(_) => {
                return Err("글무늬는 자원에 대입할 수 없습니다".to_string().into())
            }
            Value::Regex(_) => return Err("정규식은 자원에 대입할 수 없습니다".to_string().into()),
            Value::Lambda(lambda) => {
                self.resources
                    .insert(name.to_string(), Value::Lambda(lambda));
            }
        }
        Ok(())
    }

    fn append_output_log(&mut self, value: Value) -> Result<(), EvalError> {
        let mut entries = self.take_list_resource("output_log");
        let line_no = entries.len().saturating_add(1).min(i64::MAX as usize) as i64;
        let mut row = BTreeMap::new();
        let insert = |row: &mut BTreeMap<String, MapEntry>, key: &str, value: Value| {
            let key_value = Value::String(key.to_string());
            row.insert(
                map_key_canon(&key_value),
                MapEntry {
                    key: key_value,
                    value,
                },
            );
        };
        insert(
            &mut row,
            "tick",
            Value::Fixed64(Fixed64::from_i64(self.tick_id.min(i64::MAX as u64) as i64)),
        );
        insert(
            &mut row,
            "line_no",
            Value::Fixed64(Fixed64::from_i64(line_no)),
        );
        insert(&mut row, "kind", Value::String("output".to_string()));
        insert(&mut row, "text", Value::String(value_to_string(&value)));
        entries.push(Value::Map(row));
        self.set_resource("output_log", Value::List(entries))
    }

    fn take_list_resource(&mut self, name: &str) -> Vec<Value> {
        if let Some(Value::List(items)) = self.resources.remove(name) {
            return items;
        }
        match self.get_resource(name) {
            Some(Value::List(items)) => {
                self.resources.remove(name);
                items
            }
            _ => {
                self.resources.remove(name);
                Vec::new()
            }
        }
    }
}

fn receive_stmt_matches_rank(
    rank: u8,
    kind: Option<&str>,
    binding: Option<&String>,
    condition: Option<&Expr>,
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

fn parse_boim_value_expr(source: &str) -> Result<Expr, EvalError> {
    let wrapped = format!("보임값:셈씨 = {{\n  ({}) 보여주기.\n}}\n", source);
    let program = parse_with_mode(&wrapped, "#boim", ParseMode::Strict)
        .map_err(|err| EvalError::Message(format!("E_BOIM_BAD_EXPR: {}", err.message)))?;
    let Some(TopLevelItem::SeedDef(seed)) = program.items.first() else {
        return Err("E_BOIM_BAD_EXPR: 보임 값을 해석할 수 없습니다"
            .to_string()
            .into());
    };
    let Some(body) = &seed.body else {
        return Err("E_BOIM_BAD_EXPR: 보임 값 본문이 비어 있습니다"
            .to_string()
            .into());
    };
    let Some(Stmt::Show { expr, .. }) = body.stmts.first() else {
        return Err("E_BOIM_BAD_EXPR: 보임 값은 식이어야 합니다"
            .to_string()
            .into());
    };
    Ok(expr.clone())
}

fn insert_string_map_entry(row: &mut BTreeMap<String, MapEntry>, key: &str, value: Value) {
    let key_value = Value::String(key.to_string());
    row.insert(
        map_key_canon(&key_value),
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
        Value::Fixed64(value) => Some(*value),
        Value::Unit(value) => Some(value.value),
        Value::Bool(value) => Some(Fixed64::from_i64(if *value { 1 } else { 0 })),
        Value::String(value) => value.parse::<f64>().ok().map(Fixed64::from_f64_lossy),
        _ => None,
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
            pending_top_level_decl_names: self.pending_top_level_decl_names.clone(),
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
            pending_top_level_decl_names: self.pending_top_level_decl_names.clone(),
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
        if let Some((left, right)) = parse_symbolic_equivalence_assertion(&assertion.body_source) {
            return verify_symbolic_assertion(&left, &right);
        }
        if let Some((left, right)) = parse_symbolic_relation_assertion(&assertion.body_source) {
            let solve_bindings =
                solve_bindings_from_hashmap(&bindings).map_err(|err| err.message())?;
            let left_text = relation_formula_text(&left).map_err(|err| err.message())?;
            let right_text = relation_formula_text(&right).map_err(|err| err.message())?;
            if ddonirang_symbolic::relation_holds(&left_text, &right_text, &solve_bindings)
                .map_err(|err| symbolic_formula_error("equiv", err).message())?
            {
                return Ok(());
            }
            return Err("세움 relation assertion failed".to_string());
        }
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
            pending_top_level_decl_names: self.pending_top_level_decl_names.clone(),
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

fn expect_numeric_bisection_args(
    args: &[Value],
    label: &'static str,
) -> Result<(Formula, String, UnitValue, UnitValue, usize), EvalError> {
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
    let lower = unit_value_from_value(&args[2])?;
    let upper = unit_value_from_value(&args[3])?;
    let iterations = unit_value_from_value(&args[4])?;
    if lower.dim != upper.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: lower.dim,
            right: upper.dim,
        }));
    }
    if iterations.dim != UnitDim::NONE {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: iterations.dim,
            right: UnitDim::NONE,
        }));
    }
    let iteration_count = fixed64_to_nonnegative_index(iterations.value)?;
    if iteration_count == 0 {
        return Err(
            "E_CALC_NUMERIC_BAD_ITERATION: 수치해.이분법 반복횟수는 1 이상이어야 합니다"
                .to_string()
                .into(),
        );
    }
    Ok((formula, var_name, lower, upper, iteration_count))
}

fn expect_polynomial_solve_args(
    args: &[Value],
    label: &'static str,
) -> Result<(Formula, String), EvalError> {
    if args.len() != 2 {
        return Err(format!("{label}는 인자 2개를 받습니다").into());
    }
    let formula = match &args[0] {
        Value::Formula(value) => value.clone(),
        _ => return Err(format!("{label} 첫 번째 인자는 수식값이어야 합니다").into()),
    };
    let var_name = match &args[1] {
        Value::String(value) => value.trim().trim_start_matches('#').to_string(),
        _ => return Err(format!("{label} 두 번째 인자는 변수 글이어야 합니다").into()),
    };
    if var_name.is_empty() {
        return Err(format!("E_CALC_NUMERIC_BAD_VAR: {label} 변수 이름이 비어 있습니다").into());
    }
    Ok((formula, var_name))
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

fn eval_polynomial_solve_result(args: &[Value]) -> Result<Value, EvalError> {
    let (formula, var_name) = expect_polynomial_solve_args(args, "다항식.풀기")?;
    let _prepared = prepare_numeric_formula(&formula, &var_name, "다항식.풀기")?;
    let zero = Formula {
        raw: "0".to_string(),
        dialect: formula.dialect.clone(),
        explicit_tag: formula.explicit_tag,
    };
    let Value::Pack(relation) = make_relation_pack(formula, zero) else {
        return Err("다항식.풀기 relation pack 생성 실패".to_string().into());
    };
    eval_relation_solve_result(&[relation])
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
    let zero = Fixed64::ZERO;
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
) -> Result<(), EvalError> {
    if slope.to_raw() == 0 {
        if !linear_zero_condition_satisfies(compare, rhs) {
            interval.empty = true;
        }
        return Ok(());
    }
    let bound = rhs
        .try_div(slope)
        .map_err(|_| "E_LINEAR_INEQUALITY_DIV_ZERO: 선형부등식 경계 계산 중 0으로 나눌 수 없습니다".to_string())?;
    let inclusive = matches!(compare, "<=" | ">=" | "이하" | "이상");
    let slope_positive = slope.to_raw() > 0;
    match (compare, slope_positive) {
        ("<=" | "<" | "이하" | "미만", true) | (">=" | ">" | "이상" | "초과", false) => {
            tighten_linear_upper(interval, bound, inclusive)
        }
        ("<=" | "<" | "이하" | "미만", false) | (">=" | ">" | "이상" | "초과", true) => {
            tighten_linear_lower(interval, bound, inclusive)
        }
        _ => {
            return Err(
                "E_LINEAR_INEQUALITY_BAD_COMPARE: 비교는 이하, 미만, 이상, 초과 중 하나여야 합니다"
                    .to_string()
                    .into(),
            )
        }
    }
    Ok(())
}

fn linear_inequality_pack_string_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<String, EvalError> {
    match fields.get(field) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(format!("선형부등식 조건 {field} 필드는 글이어야 합니다").into()),
        None => Err(format!("선형부등식 조건에 {field} 필드가 필요합니다").into()),
    }
}

fn linear_inequality_pack_formula_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<Formula, EvalError> {
    match fields.get(field) {
        Some(Value::Formula(value)) => Ok(value.clone()),
        Some(_) => Err(format!("선형부등식 조건 {field} 필드는 수식값이어야 합니다").into()),
        None => Err(format!("선형부등식 조건에 {field} 필드가 필요합니다").into()),
    }
}

fn linear_inequality_pack_unit_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<UnitValue, EvalError> {
    match fields.get(field) {
        Some(value) => unit_value_from_value(value),
        None => Err(format!("선형부등식 조건에 {field} 필드가 필요합니다").into()),
    }
}

fn eval_linear_inequality_solve(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err("선형부등식.풀기는 인자 2개를 받습니다".to_string().into());
    }
    let conditions = match &args[0] {
        Value::List(items) => items,
        _ => return Err("선형부등식.풀기 첫 번째 인자는 조건 차림이어야 합니다".to_string().into()),
    };
    let var_name = match &args[1] {
        Value::String(value) => value.trim().trim_start_matches('#').to_string(),
        _ => return Err("선형부등식.풀기 두 번째 인자는 변수 글이어야 합니다".to_string().into()),
    };
    if var_name.is_empty() {
        return Err("E_LINEAR_INEQUALITY_BAD_VAR: 변수 이름이 비어 있습니다"
            .to_string()
            .into());
    }

    let mut interval = LinearInequalityInterval::default();
    for condition in conditions {
        let Value::Pack(fields) = condition else {
            return Err("선형부등식 조건은 묶음이어야 합니다".to_string().into());
        };
        let formula = linear_inequality_pack_formula_field(fields, "식")?;
        let compare = linear_inequality_pack_string_field(fields, "비교")?;
        let boundary = linear_inequality_pack_unit_field(fields, "경계")?;
        let prepared = prepare_numeric_formula(&formula, &var_name, "선형부등식.풀기")?;
        let y0 = eval_numeric_formula_at(
            &prepared,
            UnitValue {
                value: Fixed64::from_i64(0),
                dim: UnitDim::NONE,
            },
            "선형부등식.풀기",
        )?;
        let y1 = eval_numeric_formula_at(
            &prepared,
            UnitValue {
                value: Fixed64::from_i64(1),
                dim: UnitDim::NONE,
            },
            "선형부등식.풀기",
        )?;
        let y2 = eval_numeric_formula_at(
            &prepared,
            UnitValue {
                value: Fixed64::from_i64(2),
                dim: UnitDim::NONE,
            },
            "선형부등식.풀기",
        )?;
        for rhs in [y1, y2, boundary] {
            if y0.dim != rhs.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: y0.dim,
                    right: rhs.dim,
                }));
            }
        }
        let slope01 = y1.value.saturating_sub(y0.value);
        let slope12 = y2.value.saturating_sub(y1.value);
        if slope01 != slope12 {
            return Err("E_LINEAR_INEQUALITY_NONLINEAR: V1은 1변수 선형 부등식만 지원합니다"
                .to_string()
                .into());
        }
        let rhs = boundary.value.saturating_sub(y0.value);
        apply_linear_inequality_constraint(&mut interval, slope01, rhs, compare.trim())?;
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
        Value::String("linear_inequality_solution".to_string()),
    );
    fields.insert("변수".to_string(), Value::String(var_name));
    fields.insert(
        "제약개수".to_string(),
        unit_value_to_value(UnitValue {
            value: Fixed64::from_i64(conditions.len() as i64),
            dim: UnitDim::NONE,
        }),
    );
    if interval.empty {
        fields.insert("상태".to_string(), Value::String("공집합".to_string()));
    } else {
        fields.insert(
            "상태".to_string(),
            Value::String(if interval.lower.is_none() && interval.upper.is_none() {
                "전체"
            } else {
                "구간"
            }
            .to_string()),
        );
        if let Some(lower) = interval.lower {
            fields.insert(
                "하한".to_string(),
                unit_value_to_value(UnitValue {
                    value: lower.value,
                    dim: UnitDim::NONE,
                }),
            );
            fields.insert("하한포함".to_string(), Value::Bool(lower.inclusive));
        }
        if let Some(upper) = interval.upper {
            fields.insert(
                "상한".to_string(),
                unit_value_to_value(UnitValue {
                    value: upper.value,
                    dim: UnitDim::NONE,
                }),
            );
            fields.insert("상한포함".to_string(), Value::Bool(upper.inclusive));
        }
    }
    Ok(Value::Pack(fields))
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

fn fixed64_sign(value: Fixed64) -> i8 {
    match value.raw_i64().cmp(&0) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn numeric_bisection_root(
    prepared: &NumericFormulaPrepared,
    mut lower: UnitValue,
    mut upper: UnitValue,
    iterations: usize,
) -> Result<(UnitValue, UnitValue, usize), EvalError> {
    if upper.value < lower.value {
        std::mem::swap(&mut lower, &mut upper);
    }

    let mut f_lower = eval_numeric_formula_at(prepared, lower, "수치해.이분법")?;
    let mut f_upper = eval_numeric_formula_at(prepared, upper, "수치해.이분법")?;
    if f_lower.dim != f_upper.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: f_lower.dim,
            right: f_upper.dim,
        }));
    }
    if f_lower.value.raw_i64() == 0 {
        return Ok((
            lower,
            UnitValue {
                value: Fixed64::ZERO,
                dim: f_lower.dim,
            },
            0,
        ));
    }
    if f_upper.value.raw_i64() == 0 {
        return Ok((
            upper,
            UnitValue {
                value: Fixed64::ZERO,
                dim: f_upper.dim,
            },
            0,
        ));
    }
    if fixed64_sign(f_lower.value) == fixed64_sign(f_upper.value) {
        return Err(
            "E_CALC_NUMERIC_BRACKET_SIGN: 수치해.이분법 하한/상한 함수값의 부호가 달라야 합니다"
                .to_string()
                .into(),
        );
    }

    let two = Fixed64::from_i64(2);
    let mut best_root = lower;
    let mut best_residual = f_lower;
    let mut used_iterations = 0_usize;
    for idx in 1..=iterations {
        let mid_value = lower
            .value
            .saturating_add(upper.value)
            .try_div(two)
            .map_err(|_| "수치해.이분법 계산 중 0으로 나눌 수 없습니다".to_string())?;
        let mid = UnitValue {
            value: mid_value,
            dim: lower.dim,
        };
        let f_mid = eval_numeric_formula_at(prepared, mid, "수치해.이분법")?;
        if f_lower.dim != f_mid.dim {
            return Err(unit_error(UnitError::DimensionMismatch {
                left: f_lower.dim,
                right: f_mid.dim,
            }));
        }
        best_root = mid;
        best_residual = UnitValue {
            value: fixed64_abs(f_mid.value),
            dim: f_mid.dim,
        };
        used_iterations = idx;

        if f_mid.value.raw_i64() == 0 {
            break;
        }
        if fixed64_sign(f_lower.value) != fixed64_sign(f_mid.value) {
            upper = mid;
            f_upper = f_mid;
        } else {
            lower = mid;
            f_lower = f_mid;
        }
        if f_lower.dim != f_upper.dim {
            return Err(unit_error(UnitError::DimensionMismatch {
                left: f_lower.dim,
                right: f_upper.dim,
            }));
        }
    }

    Ok((best_root, best_residual, used_iterations))
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
        return Err(format!("{context} 정수 문자열이 비었습니다")
            .to_string()
            .into());
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
        return Err(format!("{context} 정수 문자열 형식이 아닙니다")
            .to_string()
            .into());
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

fn factorize_bigint_full(
    n: BigInt,
) -> Result<(Vec<(BigInt, u32)>, FactorizationStats), &'static str> {
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
    if sign < 0 {
        format!("-{body}")
    } else {
        body
    }
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
    if sign < 0 {
        format!("-{body}")
    } else {
        body
    }
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
    let left_kind = numeric_pack_kind(left);
    let right_kind = numeric_pack_kind(right);
    let both_factor =
        left_kind == Some(NUMERIC_KIND_FACTOR) && right_kind == Some(NUMERIC_KIND_FACTOR);
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
    if both_factor && matches!(op, "*" | "/") && out.is_integer() {
        let integer = out.value.to_integer();
        let (value, _) = make_factor_value(&make_big_int_pack_from_bigint(&integer))?;
        return Ok(Some(value));
    }
    let prefer_rational = op == "/"
        || both_factor && matches!(op, "+" | "-")
        || !left_exact.is_integer()
        || !right_exact.is_integer();
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
        Value::Unit(unit) if unit.is_dimensionless() => {
            Ok(Some(ExactNumeric::from_fixed64(unit.value)))
        }
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
                Ok(Some(ExactNumeric::from_bigint(BigInt::from(
                    approx.int_part(),
                ))))
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

fn make_relation_pack(left: Formula, right: Formula) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_KIND_FIELD.to_string(),
        Value::String(RELATION_KIND_EQUATION.to_string()),
    );
    fields.insert(RELATION_LEFT_FIELD.to_string(), Value::Formula(left));
    fields.insert(RELATION_RIGHT_FIELD.to_string(), Value::Formula(right));
    Value::Pack(fields)
}

fn eval_endpoint_relation_list(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err("endpoint connect relation 1개가 필요합니다"
            .to_string()
            .into());
    }
    let mut items = Vec::new();
    flatten_endpoint_relation_value(&values[0], &mut items)?;
    Ok(Value::List(items))
}

fn eval_endpoint_relation_normalize(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err("endpoint connect relation 1개가 필요합니다"
            .to_string()
            .into());
    }
    let mut items = Vec::new();
    flatten_endpoint_relation_value(&values[0], &mut items)?;
    let count = items.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_relation_flat_set".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(count));
    fields.insert("관계들".to_string(), Value::List(items));
    Ok(Value::Pack(fields))
}

struct EndpointFormulaRelationBridge {
    relations: Vec<Value>,
    mappings: Vec<Value>,
}

fn eval_endpoint_formula_relation_list(values: &[Value]) -> Result<Value, EvalError> {
    let bridge = endpoint_formula_relation_bridge(values)?;
    Ok(Value::List(bridge.relations))
}

fn eval_endpoint_formula_relation_set(values: &[Value]) -> Result<Value, EvalError> {
    let bridge = endpoint_formula_relation_bridge(values)?;
    let relation_count = bridge.relations.len() as i64;
    let mapping_count = bridge.mappings.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_formula_relation_set".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(relation_count));
    fields.insert("변수개수".to_string(), fixed_value(mapping_count));
    fields.insert("관계들".to_string(), Value::List(bridge.relations));
    fields.insert("변수사상".to_string(), Value::List(bridge.mappings));
    Ok(Value::Pack(fields))
}

struct EndpointBoundaryValueInjection {
    relations: Vec<Value>,
    injected_relations: Vec<Value>,
    injected_values: Vec<Value>,
    mappings: Vec<Value>,
}

fn eval_endpoint_boundary_value_relation_list(values: &[Value]) -> Result<Value, EvalError> {
    let injected = endpoint_boundary_value_injection(values)?;
    Ok(Value::List(injected.injected_relations))
}

fn eval_endpoint_boundary_value_injection(values: &[Value]) -> Result<Value, EvalError> {
    let injected = endpoint_boundary_value_injection(values)?;
    let relation_count = injected.relations.len() as i64;
    let mapping_count = injected.mappings.len() as i64;
    let injected_count = injected.injected_values.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_formula_relation_set_with_values".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(relation_count));
    fields.insert("변수개수".to_string(), fixed_value(mapping_count));
    fields.insert("주입개수".to_string(), fixed_value(injected_count));
    fields.insert("관계들".to_string(), Value::List(injected.relations));
    fields.insert("변수사상".to_string(), Value::List(injected.mappings));
    fields.insert(
        "주입값들".to_string(),
        Value::List(injected.injected_values),
    );
    Ok(Value::Pack(fields))
}

fn eval_endpoint_explicit_solve(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 2 {
        return Err(
            "endpoint connect relation과 boundary value 차림이 필요합니다"
                .to_string()
                .into(),
        );
    }
    let formula_set = eval_endpoint_formula_relation_set(&[values[0].clone()])?;
    let solve_source = match &values[1] {
        Value::List(items) if items.is_empty() => formula_set,
        Value::List(_) => {
            eval_endpoint_boundary_value_injection(&[formula_set, values[1].clone()])?
        }
        _ => eval_endpoint_boundary_value_injection(&[formula_set, values[1].clone()])?,
    };
    let formula_fields = expect_endpoint_formula_relation_mapping_source(&solve_source)?;
    let relation_list = endpoint_relation_list_field(formula_fields, "관계들")?.to_vec();
    let relation_arg = Value::List(relation_list);
    let solve_result = match expect_equation_relations(&[relation_arg])
        .and_then(|relations| eval_relation_solve_result(&relations))
    {
        Ok(result) => result,
        Err(_) => make_relation_solve_failure("unsupported"),
    };
    eval_endpoint_solve_result_remap(&[solve_source, solve_result])
}

fn eval_endpoint_explicit_solve_range_violation_list(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 3 {
        return Err(
            "endpoint connect relation, boundary value 차림, range 차림이 필요합니다"
                .to_string()
                .into(),
        );
    }
    let solve_result = eval_endpoint_explicit_solve(&[values[0].clone(), values[1].clone()])?;
    eval_endpoint_boundary_range_violation_list(&[solve_result, values[2].clone()])
}

fn eval_endpoint_explicit_solve_range_check(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 3 {
        return Err(
            "endpoint connect relation, boundary value 차림, range 차림이 필요합니다"
                .to_string()
                .into(),
        );
    }
    let solve_result = eval_endpoint_explicit_solve(&[values[0].clone(), values[1].clone()])?;
    let range_check =
        eval_endpoint_boundary_range_check(&[solve_result.clone(), values[2].clone()])?;
    let Value::Pack(solve_pack) = &solve_result else {
        return Err("endpoint solve result 묶음이 필요합니다".to_string().into());
    };
    let Value::Pack(range_pack) = &range_check else {
        return Err("endpoint range check 묶음이 필요합니다".to_string().into());
    };
    let solve_kind = endpoint_relation_string_field(solve_pack, "풀이결과종류")?.to_string();
    let check_kind = endpoint_relation_string_field(range_pack, "검사결과")?.to_string();
    let violation_count = range_pack
        .get("위반개수")
        .cloned()
        .ok_or_else(|| EvalError::from("endpoint range check 위반개수가 필요합니다".to_string()))?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_check".to_string()),
    );
    fields.insert("풀이결과".to_string(), solve_result);
    fields.insert("범위검사".to_string(), range_check);
    fields.insert("풀이결과종류".to_string(), Value::String(solve_kind));
    fields.insert("검사결과".to_string(), Value::String(check_kind));
    fields.insert("위반개수".to_string(), violation_count);
    Ok(Value::Pack(fields))
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

fn eval_endpoint_solve_range_report_rows(values: &[Value]) -> Result<Value, EvalError> {
    let report = endpoint_solve_range_report(values)?;
    Ok(Value::List(report.rows))
}

fn eval_endpoint_solve_range_report(values: &[Value]) -> Result<Value, EvalError> {
    let report = endpoint_solve_range_report(values)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_report".to_string()),
    );
    fields.insert("검사".to_string(), report.check);
    fields.insert("풀이결과종류".to_string(), Value::String(report.solve_kind));
    fields.insert("검사결과".to_string(), Value::String(report.check_kind));
    fields.insert("행개수".to_string(), fixed_value(report.rows.len() as i64));
    fields.insert("값개수".to_string(), fixed_value(report.value_count as i64));
    fields.insert(
        "누락개수".to_string(),
        fixed_value(report.missing_count as i64),
    );
    fields.insert(
        "범위개수".to_string(),
        fixed_value(report.range_count as i64),
    );
    fields.insert(
        "위반개수".to_string(),
        fixed_value(report.violation_count as i64),
    );
    fields.insert("행들".to_string(), Value::List(report.rows));
    Ok(Value::Pack(fields))
}

fn eval_endpoint_solve_range_text_report(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
        ));
    }
    Ok(Value::String(endpoint_solve_range_text_table(&values[0])?))
}

fn eval_endpoint_explicit_solve_range_text_report(values: &[Value]) -> Result<Value, EvalError> {
    let report = eval_endpoint_solve_range_report(values)?;
    Ok(Value::String(endpoint_solve_range_text_table(&report)?))
}

fn eval_endpoint_solve_range_case(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    endpoint_solve_range_case_result(&values[0])
}

fn eval_endpoint_solve_range_case_suite(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    let Value::List(cases) = &values[0] else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    };
    let mut results = Vec::new();
    let mut passed = 0usize;
    for case in cases {
        let result = endpoint_solve_range_case_result(case)?;
        let Value::Pack(result_pack) = &result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_malformed_case",
            ));
        };
        let pass = match result_pack.get("통과여부") {
            Some(Value::Bool(pass)) => *pass,
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_malformed_case",
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

fn eval_endpoint_solve_range_case_suite_text(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
        ));
    }
    Ok(Value::String(endpoint_solve_range_case_suite_text(
        &values[0],
    )?))
}

fn eval_endpoint_solve_range_case_suite_detail_text(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
        ));
    }
    Ok(Value::String(endpoint_solve_range_case_suite_detail_text(
        &values[0],
    )?))
}

fn eval_endpoint_solve_range_case_suite_run_detail_text(
    values: &[Value],
) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    let suite = eval_endpoint_solve_range_case_suite(values)?;
    Ok(Value::String(endpoint_solve_range_case_suite_detail_text(
        &suite,
    )?))
}

fn eval_endpoint_solve_range_case_suite_summary(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
        ));
    }
    endpoint_solve_range_case_suite_summary(&values[0])
}

fn eval_endpoint_solve_range_case_suite_run_summary(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    let suite = eval_endpoint_solve_range_case_suite(values)?;
    endpoint_solve_range_case_suite_summary(&suite)
}

fn eval_endpoint_solve_range_case_suite_check(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
        ));
    }
    endpoint_solve_range_case_suite_check(&values[0])
}

fn eval_endpoint_solve_range_case_suite_run_check(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    let summary = eval_endpoint_solve_range_case_suite_run_summary(values)?;
    endpoint_solve_range_case_suite_check(&summary)
}

fn endpoint_solve_range_case_result(case: &Value) -> Result<Value, EvalError> {
    let Value::Pack(case_pack) = case else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    };
    let name = endpoint_relation_string_field(case_pack, "이름")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_malformed_case"))?
        .to_string();
    let relation = case_pack
        .get("이음관계")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case"))?;
    let values = case_pack
        .get("값들")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case"))?;
    let ranges = case_pack
        .get("범위들")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case"))?;
    let expected = match case_pack.get("기대검사결과") {
        Some(Value::String(value)) if value == "통과" || value == "실패" => value.clone(),
        Some(_) => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_invalid_expected_result",
            ))
        }
        None => "통과".to_string(),
    };
    let report = eval_endpoint_solve_range_report(&[relation, values, ranges])?;
    let Value::Pack(report_pack) = &report else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    };
    let actual = endpoint_relation_string_field(report_pack, "검사결과")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_malformed_case"))?
        .to_string();
    let text = endpoint_solve_range_text_table(&report)?;
    let passed = expected == actual;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_case_result".to_string()),
    );
    fields.insert("이름".to_string(), Value::String(name));
    fields.insert("기대검사결과".to_string(), Value::String(expected));
    fields.insert("실제검사결과".to_string(), Value::String(actual));
    fields.insert("통과여부".to_string(), Value::Bool(passed));
    fields.insert("보고서".to_string(), report);
    fields.insert("문자표".to_string(), Value::String(text));
    Ok(Value::Pack(fields))
}

fn endpoint_solve_range_case_suite_pack(results: Vec<Value>, passed: usize) -> Value {
    let total = results.len();
    let failed = total.saturating_sub(passed);
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_case_suite".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(total as i64));
    fields.insert("통과개수".to_string(), fixed_value(passed as i64));
    fields.insert("실패개수".to_string(), fixed_value(failed as i64));
    fields.insert("전체통과".to_string(), Value::Bool(failed == 0));
    fields.insert("결과들".to_string(), Value::List(results));
    Value::Pack(fields)
}

fn endpoint_solve_range_case_suite_text(suite: &Value) -> Result<String, EvalError> {
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
        ));
    };
    if endpoint_relation_kind(suite_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
        ));
    }
    let results = endpoint_relation_list_field(suite_pack, "결과들")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?;
    let mut lines = vec!["이름\t기대\t실제\t통과".to_string()];
    for result in results {
        let Value::Pack(result_pack) = result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_text_expected_suite",
            ));
        };
        if endpoint_relation_kind(result_pack)
            .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?
            != "endpoint_solve_range_case_result"
        {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_text_expected_suite",
            ));
        }
        let name = endpoint_relation_string_field(result_pack, "이름")
            .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?;
        let expected = endpoint_relation_string_field(result_pack, "기대검사결과")
            .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?;
        let actual = endpoint_relation_string_field(result_pack, "실제검사결과")
            .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?;
        let passed = match result_pack.get("통과여부") {
            Some(Value::Bool(true)) => "참",
            Some(Value::Bool(false)) => "거짓",
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_text_expected_suite",
                ))
            }
        };
        lines.push(format!("{name}\t{expected}\t{actual}\t{passed}"));
    }
    Ok(lines.join("\n"))
}

fn endpoint_solve_range_case_suite_detail_text(suite: &Value) -> Result<String, EvalError> {
    let summary = endpoint_solve_range_case_suite_text(suite)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite"))?;
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
        ));
    };
    if endpoint_relation_kind(suite_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite"))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
        ));
    }
    let results = endpoint_relation_list_field(suite_pack, "결과들")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite"))?;
    let mut sections = vec![summary];
    for result in results {
        sections.push(endpoint_solve_range_case_detail_section(result)?);
    }
    Ok(sections.join("\n\n"))
}

fn endpoint_solve_range_case_detail_section(result: &Value) -> Result<String, EvalError> {
    let Value::Pack(result_pack) = result else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_malformed_case_result",
        ));
    };
    if endpoint_relation_kind(result_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result"))?
        != "endpoint_solve_range_case_result"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_malformed_case_result",
        ));
    }
    let name = endpoint_relation_string_field(result_pack, "이름").map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result")
    })?;
    let expected = endpoint_relation_string_field(result_pack, "기대검사결과").map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result")
    })?;
    let actual = endpoint_relation_string_field(result_pack, "실제검사결과").map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result")
    })?;
    let passed = match result_pack.get("통과여부") {
        Some(Value::Bool(true)) => "참",
        Some(Value::Bool(false)) => "거짓",
        _ => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_detail_malformed_case_result",
            ))
        }
    };
    let text = endpoint_relation_string_field(result_pack, "문자표").map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result")
    })?;
    Ok(format!(
        "## {name}\n기대\t{expected}\n실제\t{actual}\n통과\t{passed}\n{text}"
    ))
}

fn endpoint_solve_range_case_suite_summary(suite: &Value) -> Result<Value, EvalError> {
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
        ));
    };
    if endpoint_relation_kind(suite_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_summary_expected_suite"))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
        ));
    }
    let results = endpoint_relation_list_field(suite_pack, "결과들")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_summary_expected_suite"))?;

    let mut pass_names = Vec::new();
    let mut fail_names = Vec::new();
    let mut expected_fail_actual_pass = Vec::new();
    let mut expected_pass_actual_fail = Vec::new();

    for result in results {
        let Value::Pack(result_pack) = result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_summary_malformed_case_result",
            ));
        };
        if endpoint_relation_kind(result_pack).map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result")
        })? != "endpoint_solve_range_case_result"
        {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_summary_malformed_case_result",
            ));
        }
        let name = endpoint_relation_string_field(result_pack, "이름")
            .map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result")
            })?
            .to_string();
        let expected =
            endpoint_relation_string_field(result_pack, "기대검사결과").map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result")
            })?;
        let actual = endpoint_relation_string_field(result_pack, "실제검사결과").map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result")
        })?;
        let passed = match result_pack.get("통과여부") {
            Some(Value::Bool(pass)) => *pass,
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_summary_malformed_case_result",
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

    let total = results.len();
    let passed = pass_names.len();
    let failed = fail_names.len();
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_case_suite_summary".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(total as i64));
    fields.insert("통과개수".to_string(), fixed_value(passed as i64));
    fields.insert("실패개수".to_string(), fixed_value(failed as i64));
    fields.insert("전체통과".to_string(), Value::Bool(failed == 0));
    fields.insert(
        "통과케이스들".to_string(),
        Value::List(pass_names.into_iter().map(Value::String).collect()),
    );
    fields.insert(
        "실패케이스들".to_string(),
        Value::List(fail_names.into_iter().map(Value::String).collect()),
    );
    fields.insert(
        "기대실패통과케이스들".to_string(),
        Value::List(
            expected_fail_actual_pass
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
    fields.insert(
        "기대통과실패케이스들".to_string(),
        Value::List(
            expected_pass_actual_fail
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
    Ok(Value::Pack(fields))
}

fn endpoint_solve_range_case_suite_check(summary: &Value) -> Result<Value, EvalError> {
    let Value::Pack(summary_pack) = summary else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
        ));
    };
    if endpoint_relation_kind(summary_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_check_expected_summary"))?
        != "endpoint_solve_range_case_suite_summary"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
        ));
    }

    let overall = match summary_pack.get("전체통과") {
        Some(Value::Bool(value)) => *value,
        _ => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_check_malformed_summary",
            ))
        }
    };
    let count = endpoint_case_suite_check_count(summary_pack, "개수")?;
    let passed = endpoint_case_suite_check_count(summary_pack, "통과개수")?;
    let failed = endpoint_case_suite_check_count(summary_pack, "실패개수")?;
    let fail_cases = endpoint_case_suite_check_string_list(summary_pack, "실패케이스들")?;
    let expected_fail_actual_pass =
        endpoint_case_suite_check_string_list(summary_pack, "기대실패통과케이스들")?;
    let expected_pass_actual_fail =
        endpoint_case_suite_check_string_list(summary_pack, "기대통과실패케이스들")?;

    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_case_suite_check".to_string()),
    );
    fields.insert(
        "판정".to_string(),
        Value::String(if overall { "통과" } else { "실패" }.to_string()),
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
    Ok(Value::Pack(fields))
}

fn endpoint_case_suite_check_count(
    fields: &BTreeMap<String, Value>,
    name: &str,
) -> Result<Value, EvalError> {
    match fields.get(name) {
        Some(Value::Fixed64(_)) => Ok(fields.get(name).expect("count field").clone()),
        _ => Err(endpoint_case_suite_error(
            "connect_case_suite_check_malformed_summary",
        )),
    }
}

fn endpoint_case_suite_check_string_list(
    fields: &BTreeMap<String, Value>,
    name: &str,
) -> Result<Vec<Value>, EvalError> {
    let list = endpoint_relation_list_field(fields, name).map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_check_malformed_summary")
    })?;
    if list.iter().any(|item| !matches!(item, Value::String(_))) {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_malformed_summary",
        ));
    }
    Ok(list.to_vec())
}

fn endpoint_case_suite_error(marker: &'static str) -> EvalError {
    EvalError::from(marker.to_string())
}

fn endpoint_solve_range_text_table(report: &Value) -> Result<String, EvalError> {
    let Value::Pack(report_pack) = report else {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
        ));
    };
    if endpoint_relation_kind(report_pack).map_err(|_| {
        endpoint_report_text_error("connect_report_text_expected_solve_range_report")
    })? != "endpoint_solve_range_report"
    {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
        ));
    }
    let rows = endpoint_relation_list_field(report_pack, "행들")
        .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
    let mut lines = vec!["변수\t경로\t값상태\t값\t범위상태\t하한\t상한\t위반".to_string()];
    for row in rows {
        let Value::Pack(row_pack) = row else {
            return Err(endpoint_report_text_error(
                "connect_report_text_malformed_row",
            ));
        };
        if endpoint_relation_kind(row_pack)
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?
            != "endpoint_solve_range_report_row"
        {
            return Err(endpoint_report_text_error(
                "connect_report_text_malformed_row",
            ));
        }
        let variable = endpoint_relation_string_field(row_pack, "변수")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let path = endpoint_relation_string_field(row_pack, "경로")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let value_status = endpoint_relation_string_field(row_pack, "값상태")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let range_status = endpoint_relation_string_field(row_pack, "범위상태")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let value = row_pack.get("값").map(value_to_string).unwrap_or_default();
        let lower = row_pack
            .get("하한")
            .map(value_to_string)
            .unwrap_or_default();
        let upper = row_pack
            .get("상한")
            .map(value_to_string)
            .unwrap_or_default();
        let violations = endpoint_relation_list_field(row_pack, "위반들")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let mut reasons = Vec::new();
        for violation in violations {
            let Value::Pack(violation_pack) = violation else {
                return Err(endpoint_report_text_error(
                    "connect_report_text_malformed_row",
                ));
            };
            reasons.push(
                endpoint_relation_string_field(violation_pack, "이유")
                    .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?
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

fn endpoint_report_text_error(marker: &'static str) -> EvalError {
    EvalError::from(marker.to_string())
}

fn endpoint_solve_range_report(values: &[Value]) -> Result<EndpointSolveRangeReport, EvalError> {
    if values.len() != 3 {
        return Err(
            "endpoint connect relation, boundary value 차림, range 차림이 필요합니다"
                .to_string()
                .into(),
        );
    }
    let check = eval_endpoint_explicit_solve_range_check(values)?;
    let Value::Pack(check_pack) = &check else {
        return Err("endpoint solve range check 묶음이 필요합니다"
            .to_string()
            .into());
    };
    let solve_result = check_pack
        .get("풀이결과")
        .cloned()
        .ok_or_else(|| EvalError::from("endpoint solve result가 필요합니다".to_string()))?;
    let range_check = check_pack
        .get("범위검사")
        .cloned()
        .ok_or_else(|| EvalError::from("endpoint range check가 필요합니다".to_string()))?;
    let solve_pack = expect_endpoint_solve_result_value(&solve_result)?;
    let Value::Pack(range_pack) = &range_check else {
        return Err("endpoint range check 묶음이 필요합니다".to_string().into());
    };
    let solve_kind = endpoint_relation_string_field(solve_pack, "풀이결과종류")?.to_string();
    let check_kind = endpoint_relation_string_field(range_pack, "검사결과")?.to_string();
    let mappings = endpoint_relation_list_field(solve_pack, "변수사상")?;
    let value_by_path = endpoint_solve_result_values_by_path(solve_pack)?;
    let ranges = endpoint_range_bounds(&values[2])?;
    let mut range_by_path: BTreeMap<String, (Option<Value>, Option<Value>)> = BTreeMap::new();
    for range in &ranges {
        range_by_path.insert(range.path.clone(), (range.min.clone(), range.max.clone()));
    }
    let mut violations_by_path: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let violations = endpoint_relation_list_field(range_pack, "위반들")?;
    for violation in violations {
        let Value::Pack(violation_pack) = violation else {
            return Err("endpoint range violation 묶음이 필요합니다"
                .to_string()
                .into());
        };
        let path = endpoint_relation_string_field(violation_pack, "경로")?;
        violations_by_path
            .entry(path.to_string())
            .or_default()
            .push(violation.clone());
    }

    let mut rows = Vec::new();
    let mut value_count = 0usize;
    let mut missing_count = 0usize;
    for mapping in mappings {
        let Value::Pack(mapping_pack) = mapping else {
            return Err("endpoint variable mapping 묶음이 필요합니다"
                .to_string()
                .into());
        };
        let variable = endpoint_relation_string_field(mapping_pack, "변수")?;
        let path = endpoint_relation_string_field(mapping_pack, "경로")?;
        let row_violations = violations_by_path.get(path).cloned().unwrap_or_default();
        let mut fields = BTreeMap::new();
        fields.insert(
            "__이음관계종류".to_string(),
            Value::String("endpoint_solve_range_report_row".to_string()),
        );
        fields.insert("변수".to_string(), Value::String(variable.to_string()));
        fields.insert("경로".to_string(), Value::String(path.to_string()));
        if let Some(value) = value_by_path.get(path) {
            fields.insert("값상태".to_string(), Value::String("값있음".to_string()));
            fields.insert("값".to_string(), value.clone());
            value_count += 1;
        } else {
            fields.insert("값상태".to_string(), Value::String("누락".to_string()));
            missing_count += 1;
        }
        if let Some((min, max)) = range_by_path.get(path) {
            fields.insert(
                "범위상태".to_string(),
                Value::String(if row_violations.is_empty() {
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
            fields.insert(
                "범위상태".to_string(),
                Value::String("범위없음".to_string()),
            );
        }
        fields.insert(
            "위반개수".to_string(),
            fixed_value(row_violations.len() as i64),
        );
        fields.insert("위반들".to_string(), Value::List(row_violations));
        rows.push(Value::Pack(fields));
    }

    Ok(EndpointSolveRangeReport {
        check,
        rows,
        solve_kind,
        check_kind,
        value_count,
        missing_count,
        range_count: ranges.len(),
        violation_count: violations.len(),
    })
}

fn eval_endpoint_solve_result_value_list(values: &[Value]) -> Result<Value, EvalError> {
    let remapped = endpoint_solve_result_remap(values)?;
    Ok(Value::List(remapped.values))
}

fn eval_endpoint_solve_result_remap(values: &[Value]) -> Result<Value, EvalError> {
    let remapped = endpoint_solve_result_remap(values)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_result".to_string()),
    );
    fields.insert("풀이결과종류".to_string(), Value::String(remapped.kind));
    fields.insert("값들".to_string(), Value::List(remapped.values));
    fields.insert(
        "누락변수들".to_string(),
        Value::List(remapped.missing_variables),
    );
    fields.insert("변수사상".to_string(), Value::List(remapped.mappings));
    fields.insert("원래풀이".to_string(), remapped.original);
    Ok(Value::Pack(fields))
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

fn endpoint_solve_result_remap(values: &[Value]) -> Result<EndpointSolveResultRemap, EvalError> {
    if values.len() != 2 {
        return Err(
            "endpoint formula relation set과 relation solve result가 필요합니다"
                .to_string()
                .into(),
        );
    }
    let formula_set = expect_endpoint_formula_relation_mapping_source(&values[0])?;
    let mappings_list = endpoint_relation_list_field(formula_set, "변수사상")?;
    let mut mappings = Vec::new();
    let mut ordered_mapping = Vec::new();
    for item in mappings_list {
        let Value::Pack(mapping_pack) = item else {
            return Err("endpoint variable mapping 묶음이 필요합니다"
                .to_string()
                .into());
        };
        let variable = endpoint_relation_string_field(mapping_pack, "변수")?;
        let path = endpoint_relation_string_field(mapping_pack, "경로")?;
        if ordered_mapping
            .iter()
            .any(|(known, _): &(String, String)| known == variable)
        {
            return Err("endpoint variable mapping 변수는 유일해야 합니다"
                .to_string()
                .into());
        }
        ordered_mapping.push((variable.to_string(), path.to_string()));
        mappings.push(item.clone());
    }
    let unit_components = endpoint_unit_components(formula_set, &ordered_mapping)?;

    let solve_result = expect_relation_solve_result(&values[1])?;
    let original = values[1].clone();
    let result_kind = relation_solve_result_kind(solve_result)?;
    if result_kind != RELATION_SOLVE_RESULT_SUCCESS {
        return Ok(EndpointSolveResultRemap {
            kind: "실패".to_string(),
            values: Vec::new(),
            missing_variables: ordered_mapping
                .iter()
                .map(|(variable, _)| Value::String(variable.clone()))
                .collect(),
            mappings,
            original,
        });
    }

    let bindings = match solve_result.get(RELATION_SOLVE_BINDINGS_FIELD) {
        Some(Value::Pack(bindings)) => bindings,
        _ => {
            return Err("relation solve result 해 묶음이 필요합니다"
                .to_string()
                .into())
        }
    };
    for variable in bindings.keys() {
        if !ordered_mapping
            .iter()
            .any(|(mapped_variable, _)| mapped_variable == variable)
        {
            return Err("endpoint_variable_mapping relation solve binding이 endpoint 변수사상에 포함되어야 합니다"
                .to_string()
                .into());
        }
    }

    let mut remapped_values = Vec::new();
    let mut missing_variables = Vec::new();
    for (variable, path) in &ordered_mapping {
        if let Some(value) = bindings.get(variable) {
            let value = match unit_components.get(variable) {
                Some(seed) => {
                    let raw = endpoint_solve_numeric_value(value)?;
                    unit_value_to_value(UnitValue {
                        value: raw,
                        dim: seed.dim,
                    })
                }
                None => value.clone(),
            };
            let mut item = BTreeMap::new();
            item.insert("변수".to_string(), Value::String(variable.clone()));
            item.insert("경로".to_string(), Value::String(path.clone()));
            item.insert("값".to_string(), value);
            remapped_values.push(Value::Pack(item));
        } else {
            missing_variables.push(Value::String(variable.clone()));
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
) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err("endpoint_formula_relation_set 묶음이 필요합니다"
            .to_string()
            .into());
    };
    match fields.get("__이음관계종류") {
        Some(Value::String(kind)) if kind == "endpoint_formula_relation_set" => Ok(fields),
        _ => Err("endpoint_formula_relation_set 묶음이 필요합니다"
            .to_string()
            .into()),
    }
}

fn expect_endpoint_formula_relation_mapping_source(
    value: &Value,
) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err("endpoint_formula_relation_set 묶음이 필요합니다"
            .to_string()
            .into());
    };
    match fields.get("__이음관계종류") {
        Some(Value::String(kind))
            if kind == "endpoint_formula_relation_set"
                || kind == "endpoint_formula_relation_set_with_values" =>
        {
            Ok(fields)
        }
        _ => Err("endpoint_formula_relation_set 묶음이 필요합니다"
            .to_string()
            .into()),
    }
}

fn endpoint_boundary_value_injection(
    values: &[Value],
) -> Result<EndpointBoundaryValueInjection, EvalError> {
    if values.len() != 2 {
        return Err(endpoint_boundary_value_error(
            "connect_boundary_value_expected_formula_set",
        ));
    }
    let formula_set = expect_endpoint_formula_relation_set(&values[0]).map_err(|_| {
        endpoint_boundary_value_error("connect_boundary_value_expected_formula_set")
    })?;
    let original_relations = endpoint_relation_list_field(formula_set, "관계들")?;
    let mappings_list = endpoint_relation_list_field(formula_set, "변수사상")?;

    let mut path_to_variable = BTreeMap::new();
    let mut mappings = Vec::new();
    for item in mappings_list {
        let Value::Pack(mapping_pack) = item else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_malformed_item",
            ));
        };
        let variable = endpoint_relation_string_field(mapping_pack, "변수")?;
        let path = endpoint_relation_string_field(mapping_pack, "경로")?;
        path_to_variable.insert(path.to_string(), variable.to_string());
        mappings.push(item.clone());
    }

    let Value::List(value_items) = &values[1] else {
        return Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
        ));
    };
    let mut seen_paths = BTreeSet::new();
    let mut injected_relations = Vec::new();
    let mut injected_values = Vec::new();
    for item in value_items {
        let Value::Pack(value_pack) = item else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_malformed_item",
            ));
        };
        let path = endpoint_boundary_value_path(value_pack)?;
        if !seen_paths.insert(path.to_string()) {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_duplicate_path",
            ));
        }
        let Some(variable) = path_to_variable.get(path) else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_unknown_path",
            ));
        };
        let boundary = endpoint_boundary_value_number(value_pack)?;
        injected_relations.push(make_relation_pack(
            endpoint_formula_value(variable),
            endpoint_formula_value(&boundary.formula_text),
        ));
        let mut injected = BTreeMap::new();
        injected.insert("변수".to_string(), Value::String(variable.clone()));
        injected.insert("경로".to_string(), Value::String(path.to_string()));
        injected.insert("값".to_string(), boundary.value);
        if let Some(dim) = boundary.unit_dim {
            injected.insert("단위차원".to_string(), Value::String(dim.format()));
        }
        if let Some(symbol) = boundary.unit_symbol {
            injected.insert("단위기호".to_string(), Value::String(symbol));
        }
        injected_values.push(Value::Pack(injected));
    }

    let mut relations = original_relations.to_vec();
    relations.extend(injected_relations.clone());
    Ok(EndpointBoundaryValueInjection {
        relations,
        injected_relations,
        injected_values,
        mappings,
    })
}

fn endpoint_boundary_value_path(fields: &BTreeMap<String, Value>) -> Result<&str, EvalError> {
    match fields.get("경로") {
        Some(Value::String(path)) => Ok(path.as_str()),
        _ => Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
        )),
    }
}

fn endpoint_boundary_value_number(
    fields: &BTreeMap<String, Value>,
) -> Result<EndpointBoundaryNumber, EvalError> {
    match fields.get("값") {
        Some(Value::Fixed64(value)) => Ok(EndpointBoundaryNumber {
            value: Value::Fixed64(*value),
            formula_text: value.to_string(),
            unit_dim: None,
            unit_symbol: None,
        }),
        Some(Value::Unit(unit)) => Ok(EndpointBoundaryNumber {
            value: Value::Unit(*unit),
            formula_text: unit.value.to_string(),
            unit_dim: (!unit.is_dimensionless()).then_some(unit.dim),
            unit_symbol: (!unit.is_dimensionless()).then(|| {
                unit.display_symbol()
                    .map(str::to_string)
                    .unwrap_or_else(|| unit.dim.format())
            }),
        }),
        Some(_) => Err(endpoint_boundary_value_error(
            "connect_boundary_value_non_numeric",
        )),
        None => Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
        )),
    }
}

fn endpoint_unit_components(
    formula_set: &BTreeMap<String, Value>,
    ordered_mapping: &[(String, String)],
) -> Result<BTreeMap<String, EndpointUnitSeed>, EvalError> {
    let kind = match formula_set.get("__이음관계종류") {
        Some(Value::String(kind)) => kind.as_str(),
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

    if let Ok(relations) = endpoint_relation_list_field(formula_set, "관계들") {
        for relation in relations {
            let Value::Pack(relation_pack) = relation else {
                continue;
            };
            let mut vars = BTreeSet::new();
            if let Some(Value::Formula(left)) = relation_pack.get(RELATION_LEFT_FIELD) {
                for variable in endpoint_variables_in_formula(&relation_formula_text(left)?) {
                    if known_variables.contains(&variable) {
                        vars.insert(variable);
                    }
                }
            }
            if let Some(Value::Formula(right)) = relation_pack.get(RELATION_RIGHT_FIELD) {
                for variable in endpoint_variables_in_formula(&relation_formula_text(right)?) {
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
    if let Some(Value::List(injected)) = formula_set.get("주입값들") {
        for item in injected {
            let Value::Pack(pack) = item else {
                continue;
            };
            let variable = endpoint_relation_string_field(pack, "변수")?;
            let Some(Value::Unit(unit)) = pack.get("값") else {
                continue;
            };
            if unit.is_dimensionless() {
                continue;
            }
            let root = endpoint_find(&mut parent, variable);
            if let Some(existing) = component_units.get(&root).copied() {
                if existing != unit.dim {
                    let marker = if endpoint_currency_pair(existing, unit.dim) {
                        "connect_unit_boundary_incompatible_unit"
                    } else {
                        "connect_unit_boundary_dim_conflict"
                    };
                    return Err(endpoint_boundary_value_error(marker));
                }
            } else {
                component_units.insert(root, unit.dim);
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
    (left == UnitDim::KRW && right == UnitDim::USD)
        || (left == UnitDim::USD && right == UnitDim::KRW)
}

fn endpoint_solve_numeric_value(value: &Value) -> Result<Fixed64, EvalError> {
    match value {
        Value::Fixed64(raw) => Ok(*raw),
        Value::Unit(unit) if unit.is_dimensionless() => Ok(unit.value),
        _ => numeric_pack_approx(value)
            .ok_or_else(|| endpoint_boundary_value_error("connect_boundary_value_non_numeric")),
    }
}

fn endpoint_boundary_value_error(marker: &'static str) -> EvalError {
    marker.to_string().into()
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

fn eval_endpoint_boundary_range_violation_list(values: &[Value]) -> Result<Value, EvalError> {
    let checked = endpoint_boundary_range_check(values)?;
    Ok(Value::List(checked.violations))
}

fn eval_endpoint_boundary_range_check(values: &[Value]) -> Result<Value, EvalError> {
    let checked = endpoint_boundary_range_check(values)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_range_check".to_string()),
    );
    fields.insert(
        "검사결과".to_string(),
        Value::String(if checked.violations.is_empty() {
            "통과".to_string()
        } else {
            "실패".to_string()
        }),
    );
    fields.insert(
        "범위개수".to_string(),
        fixed_value(checked.range_count as i64),
    );
    fields.insert(
        "위반개수".to_string(),
        fixed_value(checked.violations.len() as i64),
    );
    fields.insert("위반들".to_string(), Value::List(checked.violations));
    Ok(Value::Pack(fields))
}

fn endpoint_boundary_range_check(
    values: &[Value],
) -> Result<EndpointBoundaryRangeCheck, EvalError> {
    if values.len() != 2 {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
        ));
    }
    let solve_result = expect_endpoint_solve_result_value(&values[0])?;
    let known_paths = endpoint_solve_result_mapping_paths(solve_result)?;
    let value_by_path = endpoint_solve_result_values_by_path(solve_result)?;
    let ranges = endpoint_range_bounds(&values[1])?;

    let mut violations = Vec::new();
    for range in &ranges {
        if !known_paths.contains(&range.path) {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_unknown_path",
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
            violations.push(Value::Pack(fields));
            continue;
        };
        if let Some(min) = &range.min {
            let ord = endpoint_compare_range_numbers(value, min)?;
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
                violations.push(Value::Pack(fields));
                continue;
            }
        }
        if let Some(max) = &range.max {
            let ord = endpoint_compare_range_numbers(value, max)?;
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
                violations.push(Value::Pack(fields));
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
) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
        ));
    };
    match fields.get("__이음관계종류") {
        Some(Value::String(kind)) if kind == "endpoint_solve_result" => Ok(fields),
        _ => Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
        )),
    }
}

fn endpoint_solve_result_mapping_paths(
    solve_result: &BTreeMap<String, Value>,
) -> Result<BTreeSet<String>, EvalError> {
    let mappings = endpoint_relation_list_field(solve_result, "변수사상")?;
    let mut out = BTreeSet::new();
    for item in mappings {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
            ));
        };
        out.insert(endpoint_relation_string_field(pack, "경로")?.to_string());
    }
    Ok(out)
}

fn endpoint_solve_result_values_by_path(
    solve_result: &BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Value>, EvalError> {
    let values = endpoint_relation_list_field(solve_result, "값들")?;
    let mut out = BTreeMap::new();
    for item in values {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
            ));
        };
        let path = endpoint_relation_string_field(pack, "경로")?;
        let Some(value) = pack.get("값") else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
            ));
        };
        out.insert(path.to_string(), value.clone());
    }
    Ok(out)
}

fn endpoint_range_bounds(value: &Value) -> Result<Vec<EndpointRangeBound>, EvalError> {
    let Value::List(items) = value else {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
        ));
    };
    let mut seen_paths = BTreeSet::new();
    let mut ranges = Vec::new();
    for item in items {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_malformed_item",
            ));
        };
        let path = endpoint_range_path(pack)?.to_string();
        if !seen_paths.insert(path.clone()) {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_duplicate_path",
            ));
        }
        let min = endpoint_optional_range_number(pack, "최소")?;
        let max = endpoint_optional_range_number(pack, "최대")?;
        if min.is_none() && max.is_none() {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_malformed_item",
            ));
        }
        let min_inclusive = endpoint_optional_bool(pack, "최소포함", true)?;
        let max_inclusive = endpoint_optional_bool(pack, "최대포함", true)?;
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

fn endpoint_range_path(fields: &BTreeMap<String, Value>) -> Result<&str, EvalError> {
    match fields.get("경로") {
        Some(Value::String(path)) => Ok(path.as_str()),
        _ => Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
        )),
    }
}

fn endpoint_optional_range_number(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<Option<Value>, EvalError> {
    match fields.get(field) {
        Some(Value::Fixed64(_)) | Some(Value::Unit(_)) => Ok(fields.get(field).cloned()),
        Some(_) => Err(endpoint_boundary_range_error(
            "connect_boundary_range_non_numeric",
        )),
        None => Ok(None),
    }
}

fn endpoint_optional_bool(
    fields: &BTreeMap<String, Value>,
    field: &str,
    default: bool,
) -> Result<bool, EvalError> {
    match fields.get(field) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
        )),
        None => Ok(default),
    }
}

fn endpoint_compare_range_numbers(
    left: &Value,
    right: &Value,
) -> Result<std::cmp::Ordering, EvalError> {
    let left_exact = exact_numeric_from_value(left)?;
    let right_exact = exact_numeric_from_value(right)?;
    if let (Some(left), Some(right)) = (left_exact, right_exact) {
        return Ok(exact_cmp(&left, &right));
    }
    let left = endpoint_range_numeric_unit(left)?;
    let right = endpoint_range_numeric_unit(right)?;
    if left.dim != right.dim {
        let marker = if endpoint_currency_pair(left.dim, right.dim) {
            "connect_boundary_range_incompatible_unit"
        } else {
            "connect_boundary_range_dim_conflict"
        };
        return Err(endpoint_boundary_range_error(marker));
    }
    Ok(left.value.cmp(&right.value))
}

fn endpoint_range_numeric_unit(value: &Value) -> Result<UnitValue, EvalError> {
    match value {
        Value::Fixed64(raw) => Ok(UnitValue {
            value: *raw,
            dim: UnitDim::NONE,
        }),
        Value::Unit(unit) => Ok(*unit),
        _ => Err(endpoint_boundary_range_error(
            "connect_boundary_range_non_numeric",
        )),
    }
}

fn endpoint_range_violation_base(path: &str, reason: &str) -> BTreeMap<String, Value> {
    let mut fields = BTreeMap::new();
    fields.insert("경로".to_string(), Value::String(path.to_string()));
    fields.insert("이유".to_string(), Value::String(reason.to_string()));
    fields
}

fn endpoint_boundary_range_error(marker: &'static str) -> EvalError {
    marker.to_string().into()
}

fn expect_relation_solve_result(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err("relation solve result 묶음이 필요합니다".to_string().into());
    };
    relation_solve_result_kind(fields)?;
    Ok(fields)
}

fn relation_solve_result_kind(fields: &BTreeMap<String, Value>) -> Result<&str, EvalError> {
    match fields.get(RELATION_SOLVE_RESULT_KIND_FIELD) {
        Some(Value::String(kind)) => Ok(kind.as_str()),
        _ => Err("relation solve result kind가 필요합니다".to_string().into()),
    }
}

fn endpoint_formula_relation_bridge(
    values: &[Value],
) -> Result<EndpointFormulaRelationBridge, EvalError> {
    if values.len() != 1 {
        return Err("endpoint connect relation 1개가 필요합니다"
            .to_string()
            .into());
    }
    let mut flat_relations = Vec::new();
    flatten_endpoint_relation_value(&values[0], &mut flat_relations)?;

    let mut variables: BTreeMap<String, String> = BTreeMap::new();
    let mut mappings = Vec::new();
    let mut relation_packs = Vec::new();
    for relation in flat_relations {
        let Value::Pack(fields) = relation else {
            return Err("endpoint connect relation 묶음이 필요합니다"
                .to_string()
                .into());
        };
        match endpoint_relation_kind(&fields)? {
            "endpoint_equality" => {
                let left = endpoint_formula_variable_for_field(
                    &fields,
                    "왼쪽",
                    &mut variables,
                    &mut mappings,
                )?;
                let right = endpoint_formula_variable_for_field(
                    &fields,
                    "오른쪽",
                    &mut variables,
                    &mut mappings,
                )?;
                relation_packs.push(make_relation_pack(
                    endpoint_formula_value(&left),
                    endpoint_formula_value(&right),
                ));
            }
            "endpoint_flow" => {
                let convention = endpoint_relation_string_field(&fields, "부호규약")?;
                if convention != "left_plus_right_zero" {
                    return Err(
                        "endpoint flow는 left_plus_right_zero 부호규약만 방정식화할 수 있습니다"
                            .to_string()
                            .into(),
                    );
                }
                let left = endpoint_formula_variable_for_field(
                    &fields,
                    "왼쪽",
                    &mut variables,
                    &mut mappings,
                )?;
                let right = endpoint_formula_variable_for_field(
                    &fields,
                    "오른쪽",
                    &mut variables,
                    &mut mappings,
                )?;
                relation_packs.push(make_relation_pack(
                    endpoint_formula_value(&format!("{left} + {right}")),
                    endpoint_formula_value("0"),
                ));
            }
            "endpoint_carried_property" => {
                return Err("endpoint_carried_property는 방정식화 대상이 아닙니다"
                    .to_string()
                    .into());
            }
            _ => {
                return Err("지원하지 않는 endpoint relation은 방정식화할 수 없습니다"
                    .to_string()
                    .into());
            }
        }
    }

    Ok(EndpointFormulaRelationBridge {
        relations: relation_packs,
        mappings,
    })
}

fn flatten_endpoint_relation_value(value: &Value, out: &mut Vec<Value>) -> Result<(), EvalError> {
    let Value::Pack(fields) = value else {
        return Err("endpoint connect relation 묶음이 필요합니다"
            .to_string()
            .into());
    };
    match endpoint_relation_kind(fields)? {
        "endpoint_equality" | "endpoint_flow" | "endpoint_carried_property" => {
            out.push(value.clone());
            Ok(())
        }
        "endpoint_relation_set" => {
            let relations = endpoint_relation_list_field(fields, "관계들")?;
            for item in relations {
                flatten_endpoint_relation_value(item, out)?;
            }
            Ok(())
        }
        "endpoint_relation_flat_set" => {
            let relations = endpoint_relation_list_field(fields, "관계들")?;
            for item in relations {
                flatten_endpoint_relation_value(item, out)?;
            }
            Ok(())
        }
        "endpoint_statement_set" => {
            let statements = endpoint_relation_list_field(fields, "이음들")?;
            for item in statements {
                flatten_endpoint_relation_value(item, out)?;
            }
            Ok(())
        }
        _ => Err("지원하지 않는 endpoint connect relation입니다"
            .to_string()
            .into()),
    }
}

fn endpoint_formula_variable_for_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
    variables: &mut BTreeMap<String, String>,
    mappings: &mut Vec<Value>,
) -> Result<String, EvalError> {
    let path = endpoint_relation_string_field(fields, field)?;
    if let Some(variable) = variables.get(path) {
        return Ok(variable.clone());
    }
    let variable = format!("ep_{:03}", variables.len() + 1);
    variables.insert(path.to_string(), variable.clone());
    let mut mapping = BTreeMap::new();
    mapping.insert("변수".to_string(), Value::String(variable.clone()));
    mapping.insert("경로".to_string(), Value::String(path.to_string()));
    mappings.push(Value::Pack(mapping));
    Ok(variable)
}

fn endpoint_relation_string_field<'a>(
    fields: &'a BTreeMap<String, Value>,
    field: &str,
) -> Result<&'a str, EvalError> {
    match fields.get(field) {
        Some(Value::String(value)) => Ok(value.as_str()),
        _ => Err(format!("endpoint connect relation field '{field}' 글이 필요합니다").into()),
    }
}

fn endpoint_formula_value(body: &str) -> Formula {
    Formula {
        raw: body.to_string(),
        dialect: FormulaDialect::Ascii,
        explicit_tag: true,
    }
}

fn endpoint_relation_kind(fields: &BTreeMap<String, Value>) -> Result<&str, EvalError> {
    match fields.get("__이음관계종류") {
        Some(Value::String(kind)) => Ok(kind.as_str()),
        _ => Err("endpoint connect relation kind가 필요합니다"
            .to_string()
            .into()),
    }
}

fn endpoint_relation_list_field<'a>(
    fields: &'a BTreeMap<String, Value>,
    field: &str,
) -> Result<&'a [Value], EvalError> {
    match fields.get(field) {
        Some(Value::List(items)) => Ok(items),
        _ => Err(format!("endpoint connect relation field '{field}' 차림이 필요합니다").into()),
    }
}

fn make_relation_solve_success_bindings(bindings: BTreeMap<String, Value>) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
        Value::String(RELATION_SOLVE_RESULT_SUCCESS.to_string()),
    );
    if bindings.len() == 1 {
        if let Some((variable, value)) = bindings.iter().next() {
            fields.insert(
                RELATION_SOLVE_VAR_FIELD.to_string(),
                Value::String(variable.clone()),
            );
            fields.insert(RELATION_SOLVE_VALUE_FIELD.to_string(), value.clone());
        }
    }
    fields.insert(
        RELATION_SOLVE_BINDINGS_FIELD.to_string(),
        Value::Pack(bindings),
    );
    Value::Pack(fields)
}

fn make_relation_solve_failure(reason: &str) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
        Value::String(RELATION_SOLVE_RESULT_FAILURE.to_string()),
    );
    fields.insert(
        RELATION_SOLVE_REASON_FIELD.to_string(),
        Value::String(reason.to_string()),
    );
    Value::Pack(fields)
}

fn expect_single_equation_relation(value: &Value) -> Result<BTreeMap<String, Value>, EvalError> {
    let Value::Pack(pack) = value else {
        return Err("방정식 관계가 필요합니다".to_string().into());
    };
    match pack.get(RELATION_KIND_FIELD) {
        Some(Value::String(kind)) if kind == RELATION_KIND_EQUATION => {}
        _ => return Err("방정식 관계가 필요합니다".to_string().into()),
    }
    match (
        pack.get(RELATION_LEFT_FIELD),
        pack.get(RELATION_RIGHT_FIELD),
    ) {
        (Some(Value::Formula(_)), Some(Value::Formula(_))) => Ok(pack.clone()),
        _ => Err("방정식 관계가 필요합니다".to_string().into()),
    }
}

fn expect_equation_relations(values: &[Value]) -> Result<Vec<BTreeMap<String, Value>>, EvalError> {
    if values.len() != 1 {
        return Err("방정식 관계 1개가 필요합니다".to_string().into());
    }
    match &values[0] {
        Value::Pack(_) => Ok(vec![expect_single_equation_relation(&values[0])?]),
        Value::List(items) => {
            if items.len() != 2 {
                return Err("방정식 관계 차림은 2개여야 합니다".to_string().into());
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(expect_single_equation_relation(item)?);
            }
            Ok(out)
        }
        _ => Err("방정식 관계가 필요합니다".to_string().into()),
    }
}

fn relation_formula_text(formula: &Formula) -> Result<String, EvalError> {
    if !matches!(
        formula.dialect,
        FormulaDialect::Ascii | FormulaDialect::Ascii1
    ) {
        return Err(EvalError::Message(
            "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE #ascii 수식만 지원합니다".to_string(),
        ));
    }
    let analysis = analyze_formula_for_transform(formula)?;
    Ok(match analysis.assign_name {
        Some(assign_name) => format!(
            "({assign_name}) - ({})",
            format_formula_expr(&analysis.expr, 0)
        ),
        None => format_formula_expr(&analysis.expr, 0),
    })
}

fn relation_binding_value_from_text(
    binding: &ddonirang_symbolic::SolveBinding,
) -> Result<Value, EvalError> {
    let num = parse_bigint_text(&binding.numerator, "relation solve numerator")?;
    let den = parse_bigint_text(&binding.denominator, "relation solve denominator")?;
    Ok(if den == BigInt::from(1u8) {
        make_big_int_pack_from_bigint(&num)
    } else {
        make_rational_pack_from_big_rational(&BigRational::new(num, den))
    })
}

fn eval_relation_solve_result(relations: &[BTreeMap<String, Value>]) -> Result<Value, EvalError> {
    let outcome = match relations.len() {
        1 => {
            let left = match relations[0].get(RELATION_LEFT_FIELD) {
                Some(Value::Formula(formula)) => formula,
                _ => return Err("방정식 관계가 필요합니다".to_string().into()),
            };
            let right = match relations[0].get(RELATION_RIGHT_FIELD) {
                Some(Value::Formula(formula)) => formula,
                _ => return Err("방정식 관계가 필요합니다".to_string().into()),
            };
            let left_text = relation_formula_text(left)?;
            let right_text = relation_formula_text(right)?;
            ddonirang_symbolic::solve_relation_equation(&left_text, &right_text)
        }
        2 => {
            let pairs = relations
                .iter()
                .map(|relation| {
                    let left = match relation.get(RELATION_LEFT_FIELD) {
                        Some(Value::Formula(formula)) => formula,
                        _ => return Err("방정식 관계가 필요합니다".to_string().into()),
                    };
                    let right = match relation.get(RELATION_RIGHT_FIELD) {
                        Some(Value::Formula(formula)) => formula,
                        _ => return Err("방정식 관계가 필요합니다".to_string().into()),
                    };
                    Ok((relation_formula_text(left)?, relation_formula_text(right)?))
                })
                .collect::<Result<Vec<_>, EvalError>>()?;
            ddonirang_symbolic::solve_relation_system(&pairs)
        }
        _ => Err("E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE relation solve arity".to_string()),
    };
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(err) if err.starts_with("E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE") => {
            return Ok(make_relation_solve_failure("unsupported"));
        }
        Err(err) => return Err(EvalError::Message(err)),
    };

    Ok(match outcome {
        ddonirang_symbolic::RelationSolveOutcome::Solution(solution) => {
            let mut bindings = BTreeMap::new();
            for (variable, binding) in solution {
                bindings.insert(variable, relation_binding_value_from_text(&binding)?);
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
        return Err(format!(
            "지원하지 않는 흐름 schema: {} (기대값: {})",
            schema, STREAM_V1_SCHEMA
        )
        .into());
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
        Value::Fixed64(Fixed64::from_i64(
            stream.capacity.min(i64::MAX as usize) as i64
        )),
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
        Value::Fixed64(n) => format_fixed64_for_display(*n),
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
        Value::List(items) => {
            let rendered = items
                .iter()
                .map(value_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("차림[{rendered}]")
        }
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
            if let Some(Value::String(kind)) = items.get(RELATION_KIND_FIELD) {
                if kind == RELATION_KIND_EQUATION {
                    let lhs = items
                        .get(RELATION_LEFT_FIELD)
                        .map(value_to_string)
                        .unwrap_or_else(|| "[수식]".to_string());
                    let rhs = items
                        .get(RELATION_RIGHT_FIELD)
                        .map(value_to_string)
                        .unwrap_or_else(|| "[수식]".to_string());
                    return format!("{lhs} =:= {rhs}");
                }
            }
            if let Some(Value::String(kind)) = items.get(RELATION_SOLVE_RESULT_KIND_FIELD) {
                if kind == RELATION_SOLVE_RESULT_SUCCESS {
                    if let (Some(Value::String(variable)), Some(value)) = (
                        items.get(RELATION_SOLVE_VAR_FIELD),
                        items.get(RELATION_SOLVE_VALUE_FIELD),
                    ) {
                        return format!(
                            "#성공(미지수=\"{variable}\", 값={})",
                            value_to_string(value)
                        );
                    }
                    if let Some(Value::Pack(bindings)) = items.get(RELATION_SOLVE_BINDINGS_FIELD) {
                        let rendered = bindings
                            .iter()
                            .map(|(key, value)| format!("{key}={}", value_to_string(value)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        return format!("#성공(해=({rendered}))");
                    }
                }
                if kind == RELATION_SOLVE_RESULT_FAILURE {
                    let reason = items
                        .get(RELATION_SOLVE_REASON_FIELD)
                        .and_then(|v| match v {
                            Value::String(text) => Some(text.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "?".to_string());
                    return format!("#실패(사유=\"{reason}\")");
                }
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
        Value::Formula(formula) => formula_display_string(formula),
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

fn format_fixed64_for_display(value: Fixed64) -> String {
    let raw = value.raw_i64();
    if raw == 0 {
        return "0".to_string();
    }
    let negative = raw < 0;
    let abs = if raw == i64::MIN {
        (i64::MAX as i128 + 1) as i128
    } else {
        raw.abs() as i128
    };
    let int_part = abs >> Fixed64::FRAC_BITS;
    let mut frac = abs & ((1_i128 << Fixed64::FRAC_BITS) - 1);
    let mut out = int_part.to_string();
    if frac != 0 {
        let mut digits = String::new();
        for _ in 0..10 {
            frac *= 10;
            let digit = (frac >> Fixed64::FRAC_BITS) as u8;
            frac &= (1_i128 << Fixed64::FRAC_BITS) - 1;
            digits.push((b'0' + digit) as char);
        }
        while digits.ends_with('0') {
            digits.pop();
        }
        if !digits.is_empty() {
            out.push('.');
            out.push_str(&digits);
        }
    }
    if negative {
        out.insert(0, '-');
    }
    out
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
        out.push_str("바뀔때마다 ");
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

fn formula_resource_value(formula: &Formula) -> ResourceValue {
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
        ResourceValue::String(FORMULA_RESOURCE_KIND.to_string()),
    );
    insert(
        &mut converted,
        "raw",
        ResourceValue::String(formula.raw.clone()),
    );
    let dialect = match &formula.dialect {
        FormulaDialect::Ascii => "ascii".to_string(),
        FormulaDialect::Ascii1 => "ascii1".to_string(),
        FormulaDialect::Latex => "latex".to_string(),
        FormulaDialect::Other(value) => format!("other:{value}"),
    };
    insert(&mut converted, "dialect", ResourceValue::String(dialect));
    insert(
        &mut converted,
        "explicit_tag",
        ResourceValue::Bool(formula.explicit_tag),
    );
    ResourceValue::Map(converted)
}

pub(crate) fn formula_from_resource_value(value: &ResourceValue) -> Option<Formula> {
    match value {
        ResourceValue::Map(entries) => resource_map_to_formula(entries),
        _ => None,
    }
}

pub(crate) fn formula_summary_parts(formula: &Formula) -> (String, String) {
    let raw = format_formula_body(&formula.raw, &formula.dialect)
        .unwrap_or_else(|_| formula.raw.trim().to_string());
    (formula_dialect_tag(&formula.dialect), raw)
}

pub(crate) fn formula_display_string(formula: &Formula) -> String {
    let (tag, raw) = formula_summary_parts(formula);
    if formula.explicit_tag {
        format!("({tag}) 수식{{ {raw} }}")
    } else {
        format!("수식{{ {raw} }}")
    }
}

fn formula_dialect_tag(dialect: &FormulaDialect) -> String {
    match dialect {
        FormulaDialect::Ascii => "#ascii".to_string(),
        FormulaDialect::Ascii1 => "#ascii1".to_string(),
        FormulaDialect::Latex => "#latex".to_string(),
        FormulaDialect::Other(value) => format!("#{value}"),
    }
}

fn resource_map_to_formula(entries: &BTreeMap<String, ResourceMapEntry>) -> Option<Formula> {
    let kind = match resource_map_lookup(entries, "__ddn_kind")? {
        ResourceValue::String(value) => value,
        _ => return None,
    };
    if kind != FORMULA_RESOURCE_KIND {
        return None;
    }
    let raw = match resource_map_lookup(entries, "raw")? {
        ResourceValue::String(value) => value.clone(),
        _ => return None,
    };
    let dialect = match resource_map_lookup(entries, "dialect")? {
        ResourceValue::String(value) if value == "ascii" => FormulaDialect::Ascii,
        ResourceValue::String(value) if value == "ascii1" => FormulaDialect::Ascii1,
        ResourceValue::String(value) if value == "latex" => FormulaDialect::Latex,
        ResourceValue::String(value) => FormulaDialect::Other(
            value
                .strip_prefix("other:")
                .unwrap_or(value.as_str())
                .to_string(),
        ),
        _ => return None,
    };
    let explicit_tag = match resource_map_lookup(entries, "explicit_tag") {
        Some(ResourceValue::Bool(value)) => *value,
        Some(_) => return None,
        None => false,
    };
    Some(Formula {
        raw,
        dialect,
        explicit_tag,
    })
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
        Value::Formula(formula) => Ok(formula_resource_value(formula)),
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
            if let Some(formula) = resource_map_to_formula(entries) {
                return Value::Formula(formula);
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

fn expect_single_formula(values: &[Value], label: &'static str) -> Result<Formula, EvalError> {
    if values.len() != 1 {
        return Err(EvalError::Message(format!(
            "{label}는 수식값 인자 1개를 받습니다"
        )));
    }
    match &values[0] {
        Value::Formula(value) => Ok(value.clone()),
        _ => Err(EvalError::Message(format!(
            "{label}는 수식값 인자가 필요합니다"
        ))),
    }
}

fn expect_two_formulas(
    values: &[Value],
    label: &'static str,
) -> Result<(Formula, Formula), EvalError> {
    if values.len() != 2 {
        return Err(EvalError::Message(format!(
            "{label}는 수식값 인자 2개를 받습니다"
        )));
    }
    let left = match &values[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(EvalError::Message(format!(
                "{label} 왼쪽은 수식값이어야 합니다"
            )))
        }
    };
    let right = match &values[1] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(EvalError::Message(format!(
                "{label} 오른쪽은 수식값이어야 합니다"
            )))
        }
    };
    Ok((left, right))
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
    let needs_var = matches!(call_name, "diff" | "int");
    let var_name = if needs_var {
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
        var_name
    } else {
        String::new()
    };

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

    let expr_text = format_formula_expr(&analysis.expr, 0);
    let expr_text = match call_name {
        "simplify" => ddonirang_symbolic::simplify(&expr_text),
        "expand" => ddonirang_symbolic::expand(&expr_text),
        "factor" => ddonirang_symbolic::factor(&expr_text),
        "diff" => {
            let mut out = expr_text;
            let order = options.order.unwrap_or(1);
            for _ in 0..order {
                out = ddonirang_symbolic::diff(&out, &var_name)
                    .map_err(|err| symbolic_formula_error(call_name, err))?;
            }
            Ok(out)
        }
        "int" => ddonirang_symbolic::integrate(&expr_text, &var_name),
        _ => Err(format!("E_SYMBOLIC_UNKNOWN_TRANSFORM {call_name}")),
    };
    let mut expr_text = expr_text.map_err(|err| symbolic_formula_error(call_name, err))?;
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

fn symbolic_formula_error(call_name: &str, message: String) -> EvalError {
    let label = match call_name {
        "simplify" => "정리하기",
        "expand" => "전개하기",
        "factor" => "인수분해하기",
        "diff" => "미분하기",
        "int" => "적분하기",
        "equiv" => "동치인가",
        _ => "수식변환",
    };
    EvalError::Message(format!(
        "E_SYMBOLIC_FORMULA_UNSUPPORTED: {label} ({message})"
    ))
}

fn symbolic_formulas_equivalent(left: &Formula, right: &Formula) -> Result<bool, EvalError> {
    if !matches!(left.dialect, FormulaDialect::Ascii)
        || !matches!(right.dialect, FormulaDialect::Ascii)
    {
        return Err(EvalError::Message(
            "동치인가는 #ascii 수식만 지원합니다".to_string(),
        ));
    }
    let left_analysis = analyze_formula_for_transform(left)?;
    let right_analysis = analyze_formula_for_transform(right)?;
    let left_text = format_formula_expr(&left_analysis.expr, 0);
    let right_text = format_formula_expr(&right_analysis.expr, 0);
    ddonirang_symbolic::equivalent(&left_text, &right_text)
        .map_err(|err| symbolic_formula_error("equiv", err))
}

fn verify_symbolic_assertion(left: &Formula, right: &Formula) -> Result<(), String> {
    if !matches!(left.dialect, FormulaDialect::Ascii)
        || !matches!(right.dialect, FormulaDialect::Ascii)
    {
        return Err("세움 symbolic bridge는 #ascii 수식만 지원합니다".to_string());
    }
    let left_analysis = analyze_formula_for_transform(left).map_err(|err| err.message())?;
    let right_analysis = analyze_formula_for_transform(right).map_err(|err| err.message())?;
    let left_text = format_formula_expr(&left_analysis.expr, 0);
    let right_text = format_formula_expr(&right_analysis.expr, 0);
    let cert = ddonirang_symbolic::prove_equivalent(&left_text, &right_text)
        .map_err(|err| symbolic_formula_error("equiv", err).message())?;
    let cert_value = serde_json::to_value(&cert)
        .map_err(|err| format!("proof certificate serialize failed: {err}"))?;
    let report = ddonirang_proof::verify_value(&cert_value)
        .map_err(|err| format!("proof verify failed: {err}"))?;
    if report.valid && cert.equivalent {
        Ok(())
    } else {
        Err("세움 symbolic equivalence proof failed".to_string())
    }
}

fn parse_symbolic_equivalence_assertion(raw: &str) -> Option<(Formula, Formula)> {
    let body = trim_seum_wrapper(raw.trim());
    let eq_idx = find_top_level_equivalence_eq(body)?;
    let left = parse_formula_literal_text(body[..eq_idx].trim())?;
    let right = parse_formula_literal_text(body[eq_idx + 1..].trim())?;
    Some((left, right))
}

fn parse_symbolic_relation_assertion(raw: &str) -> Option<(Formula, Formula)> {
    let body = trim_seum_wrapper(raw.trim());
    let eq_idx = body.find("=:=")?;
    let left = parse_formula_literal_text(body[..eq_idx].trim())?;
    let right = parse_formula_literal_text(body[eq_idx + 3..].trim())?;
    Some((left, right))
}

fn trim_seum_wrapper(raw: &str) -> &str {
    let trimmed = raw.trim();
    let Some(rest) = trimmed.strip_prefix("세움") else {
        return trimmed;
    };
    let rest = rest.trim_start();
    if !rest.starts_with('{') || !rest.ends_with('}') {
        return trimmed;
    }
    rest[1..rest.len().saturating_sub(1)].trim()
}

fn find_top_level_equivalence_eq(raw: &str) -> Option<usize> {
    let mut brace_depth = 0i32;
    let mut paren_depth = 0i32;
    for (idx, ch) in raw.char_indices() {
        match ch {
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '=' if brace_depth == 0 && paren_depth == 0 => return Some(idx),
            _ => {}
        }
    }
    None
}

fn parse_formula_literal_text(raw: &str) -> Option<Formula> {
    let text = raw.trim().trim_end_matches('.').trim();
    let dialect = if text.contains("#ascii1") {
        FormulaDialect::Ascii1
    } else if text.contains("#ascii") {
        FormulaDialect::Ascii
    } else if text.contains("#latex") {
        FormulaDialect::Latex
    } else {
        return None;
    };
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(Formula {
        raw: text[start + 1..end].trim().to_string(),
        dialect,
        explicit_tag: true,
    })
}

fn solve_binding_from_value(value: &Value) -> Result<ddonirang_symbolic::SolveBinding, EvalError> {
    let Some(exact) = exact_numeric_from_value(value)? else {
        return Err("세움 relation binding은 exact numeric이어야 합니다"
            .to_string()
            .into());
    };
    Ok(ddonirang_symbolic::SolveBinding {
        numerator: exact.value.numer().to_string(),
        denominator: exact.value.denom().to_string(),
    })
}

fn solve_bindings_from_hashmap(
    bindings: &HashMap<String, Value>,
) -> Result<BTreeMap<String, ddonirang_symbolic::SolveBinding>, EvalError> {
    let mut out = BTreeMap::new();
    for (name, value) in bindings {
        out.insert(name.clone(), solve_binding_from_value(value)?);
    }
    Ok(out)
}

fn eval_symbolic_proof_tactic(values: &[Value]) -> Result<BTreeMap<String, Value>, EvalError> {
    if values.len() == 1 {
        let Value::Pack(pack) = &values[0] else {
            return Err(EvalError::Message(
                "증명하기는 묶음 인자가 필요합니다".to_string(),
            ));
        };
        if let Some(relation_value) = find_pack_value(pack, &["관계", "relation", "식", "equation"])
        {
            let relations = extract_relation_texts_from_value(relation_value)?;
            if let Some(bindings_value) = find_pack_value(pack, &["해", "bindings", "값들"]) {
                let Value::Pack(bindings_pack) = bindings_value else {
                    return Err("증명하기 해는 묶음이어야 합니다".to_string().into());
                };
                let bindings = solve_bindings_from_pack(bindings_pack)?;
                let verified = ddonirang_symbolic::relation_system_holds(&relations, &bindings)
                    .map_err(|err| symbolic_formula_error("equiv", err))?;
                let mut fields = BTreeMap::new();
                fields.insert("검증".to_string(), Value::Bool(verified));
                fields.insert(
                    "종류".to_string(),
                    Value::String("relation_solve_consistency".to_string()),
                );
                fields.insert(
                    "관계수".to_string(),
                    Value::Fixed64(Fixed64::from_i64(relations.len() as i64)),
                );
                return Ok(fields);
            }
            if relations.len() != 1 {
                return Err("증명하기 relation proof는 단일 관계만 지원합니다"
                    .to_string()
                    .into());
            }
            let cert = ddonirang_symbolic::prove_equivalent(&relations[0].0, &relations[0].1)
                .map_err(|err| symbolic_formula_error("equiv", err))?;
            let cert_value = serde_json::to_value(&cert).map_err(|err| {
                EvalError::Message(format!("proof certificate serialize failed: {err}"))
            })?;
            let report = ddonirang_proof::verify_value(&cert_value)
                .map_err(|err| EvalError::Message(format!("proof verify failed: {err}")))?;
            let mut fields = BTreeMap::new();
            fields.insert(
                "검증".to_string(),
                Value::Bool(report.valid && cert.equivalent),
            );
            fields.insert(
                "종류".to_string(),
                Value::String("relation_equivalence".to_string()),
            );
            fields.insert("증거".to_string(), Value::String(cert.certificate_hash));
            fields.insert("왼쪽정본".to_string(), Value::String(cert.lhs_canonical));
            fields.insert("오른쪽정본".to_string(), Value::String(cert.rhs_canonical));
            return Ok(fields);
        }
    }

    let (left, right) = if values.len() == 2 {
        expect_two_formulas(values, "증명하기")?
    } else {
        if values.len() != 1 {
            return Err(EvalError::Message(
                "증명하기는 증명요청 묶음 1개를 받습니다".to_string(),
            ));
        }
        let Value::Pack(pack) = &values[0] else {
            return Err(EvalError::Message(
                "증명하기는 묶음 인자가 필요합니다".to_string(),
            ));
        };
        let left = find_pack_formula(pack, &["왼쪽", "left", "lhs"])
            .ok_or_else(|| EvalError::Message("증명하기 왼쪽 수식이 없습니다".to_string()))?;
        let right = find_pack_formula(pack, &["오른쪽", "right", "rhs"])
            .ok_or_else(|| EvalError::Message("증명하기 오른쪽 수식이 없습니다".to_string()))?;
        (left, right)
    };
    if !matches!(left.dialect, FormulaDialect::Ascii)
        || !matches!(right.dialect, FormulaDialect::Ascii)
    {
        return Err(EvalError::Message(
            "증명하기는 #ascii 수식만 지원합니다".to_string(),
        ));
    }
    let left_analysis = analyze_formula_for_transform(&left)?;
    let right_analysis = analyze_formula_for_transform(&right)?;
    let left_text = format_formula_expr(&left_analysis.expr, 0);
    let right_text = format_formula_expr(&right_analysis.expr, 0);
    let cert = ddonirang_symbolic::prove_equivalent(&left_text, &right_text)
        .map_err(|err| symbolic_formula_error("equiv", err))?;
    let cert_value = serde_json::to_value(&cert)
        .map_err(|err| EvalError::Message(format!("proof certificate serialize failed: {err}")))?;
    let report = ddonirang_proof::verify_value(&cert_value)
        .map_err(|err| EvalError::Message(format!("proof verify failed: {err}")))?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "검증".to_string(),
        Value::Bool(report.valid && cert.equivalent),
    );
    fields.insert(
        "종류".to_string(),
        Value::String("symbolic_equivalence".to_string()),
    );
    fields.insert("증거".to_string(), Value::String(cert.certificate_hash));
    fields.insert("왼쪽정본".to_string(), Value::String(cert.lhs_canonical));
    fields.insert("오른쪽정본".to_string(), Value::String(cert.rhs_canonical));
    Ok(fields)
}

fn find_pack_value<'a>(pack: &'a BTreeMap<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(value) = pack.get(*key) {
            return Some(value);
        }
    }
    None
}

fn find_pack_formula(pack: &BTreeMap<String, Value>, keys: &[&str]) -> Option<Formula> {
    keys.iter().find_map(|key| match pack.get(*key) {
        Some(Value::Formula(formula)) => Some(formula.clone()),
        _ => None,
    })
}

fn solve_bindings_from_pack(
    pack: &BTreeMap<String, Value>,
) -> Result<BTreeMap<String, ddonirang_symbolic::SolveBinding>, EvalError> {
    let mut out = BTreeMap::new();
    for (name, value) in pack {
        out.insert(name.clone(), solve_binding_from_value(value)?);
    }
    Ok(out)
}

fn extract_relation_texts_from_value(value: &Value) -> Result<Vec<(String, String)>, EvalError> {
    match value {
        Value::Pack(_) => {
            let relation = expect_single_equation_relation(value)?;
            let left = match relation.get(RELATION_LEFT_FIELD) {
                Some(Value::Formula(formula)) => relation_formula_text(formula)?,
                _ => return Err("equation relation이 필요합니다".to_string().into()),
            };
            let right = match relation.get(RELATION_RIGHT_FIELD) {
                Some(Value::Formula(formula)) => relation_formula_text(formula)?,
                _ => return Err("equation relation이 필요합니다".to_string().into()),
            };
            Ok(vec![(left, right)])
        }
        Value::List(list) => {
            let mut out = Vec::new();
            for item in list {
                out.extend(extract_relation_texts_from_value(item)?);
            }
            Ok(out)
        }
        _ => Err("equation relation이 필요합니다".to_string().into()),
    }
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
                if matches!(dialect, FormulaDialect::Ascii1) {
                    tokens.extend(
                        split_ascii1_formula_ident(&ident)
                            .into_iter()
                            .map(FormulaToken::Ident),
                    );
                } else {
                    tokens.push(FormulaToken::Ident(ident));
                }
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

fn split_ascii1_formula_ident(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = name.chars().peekable();
    while let Some(first) = chars.next() {
        let mut ident = String::new();
        ident.push(first);
        while matches!(chars.peek(), Some(ch) if ch.is_ascii_digit()) {
            if let Some(ch) = chars.next() {
                ident.push(ch);
            }
        }
        out.push(ident);
    }
    out
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
        "수" | "셈수" => match value {
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
    use std::sync::{Mutex, MutexGuard};

    static AGE_TARGET_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct AgeTargetTestGuard {
        _lock: MutexGuard<'static, ()>,
        previous: AgeTarget,
    }

    impl AgeTargetTestGuard {
        fn new(age: AgeTarget) -> Self {
            let lock = AGE_TARGET_TEST_LOCK
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            let previous = default_age_target();
            set_default_age_target(age);
            Self {
                _lock: lock,
                previous,
            }
        }
    }

    impl Drop for AgeTargetTestGuard {
        fn drop(&mut self) {
            set_default_age_target(self.previous);
        }
    }

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

    fn linear_inequality_test_condition(body: &str, compare: &str, boundary: i64) -> RuntimeValue {
        let mut fields = BTreeMap::new();
        fields.insert(
            "식".to_string(),
            RuntimeValue::Formula(Formula {
                raw: body.to_string(),
                dialect: FormulaDialect::Ascii,
                explicit_tag: true,
            }),
        );
        fields.insert("비교".to_string(), RuntimeValue::String(compare.to_string()));
        fields.insert(
            "경계".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(boundary)),
        );
        RuntimeValue::Pack(fields)
    }

    #[test]
    fn imja_signal_send_dispatches_receive_handler() {
        let script = r#"
채비 {
  받은:글 <- "".
}.

(값:수) 첫알림:알림씨 = {
  없음.
}

관제탑:임자 = {
  첫알림을 받으면 {
    받은 <- "ok".
  }.
}

매틱:움직씨 = {
  ((값=1) 첫알림) ~~> 관제탑.
}
"#;
        let program = DdnProgram::from_source(script, "imja_signal.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("받은".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("받은"),
            Some(&RuntimeValue::String("ok".to_string()))
        );
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
    fn from_source_accepts_boim_surface_frontdoor() {
        let source = "보임 {\n  x: 1.\n}.\n";
        DdnProgram::from_source(source, "boim.ddn").expect("boim must parse");
    }

    #[test]
    fn setting_madi_is_extracted_from_program() {
        let source = r#"
설정 {
  제목: 마디수_시험.
  마디수: 3.
}.

채비 {
  t: 수 <- 0.
}.
"#;
        let program = DdnProgram::from_source(source, "<test>").expect("program");
        assert_eq!(program.configured_madi(), Some(3));
    }

    #[test]
    fn setting_madi_rejects_non_positive_value() {
        let source = r#"
설정 {
  마디수: 0.
}.

채비 {
  t: 수 <- 0.
}.
"#;
        let err = match DdnProgram::from_source(source, "<test>") {
            Ok(_) => panic!("bad madi must fail"),
            Err(err) => err,
        };
        assert!(err.contains("E_SETTING_MADI_BAD_VALUE"), "{err}");
    }

    #[test]
    fn from_source_accepts_decl_item_maegim_suffix_frontdoor() {
        let source = r#"
매틱:움직씨 = {
  채비 {
    데이터길이:수 <- (12) 매김 {
      범위: 4..40.
      간격: 1.
    }.
  }.
}.
"#;
        let _program = DdnProgram::from_source(source, "decl_item_maegim.ddn")
            .expect("decl item maegim must parse");
    }

    #[test]
    fn from_source_accepts_legacy_bogae_draw_shorthand_frontdoor() {
        let source = r#"
채비 {
  데이터길이:수 <- (12) 매김 {
    범위: 4..40.
    간격: 1.
  }.
}.

(시작)할때 {
  t <- 0.
}.

(매마디)마다 {
  t <- t + 1.
}.

보개로 그려.
"#;
        let _program = DdnProgram::from_source(source, "legacy_bogae_draw_frontdoor.ddn")
            .expect("legacy bogae draw shorthand must parse");
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
    fn numeric_root_finding_v1_bisection_returns_root_residual_iteration_method() {
        let script = r#"
매틱:움직씨 = {
    r <- (((#ascii) 수식{x - 2}), "x", 0, 4, 8) 수치해.이분법.
    근 <- (r, 0) 차림.값.
    잔차 <- (r, 1) 차림.값.
    반복값 <- (r, 2) 차림.값.
    방법 <- (r, 3) 차림.값.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_root_finding_v1.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("근".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("잔차".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("반복값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        defaults.insert("방법".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        assert_eq!(extract_fixed(&output.resources, "근"), Fixed64::from_i64(2));
        assert_eq!(extract_fixed(&output.resources, "잔차"), Fixed64::ZERO);
        assert_eq!(extract_fixed(&output.resources, "반복값"), Fixed64::from_i64(1));
        match output.resources.get("방법") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "이분법"),
            other => panic!("방법 must be string, got {:?}", other),
        }
    }

    #[test]
    fn numeric_root_finding_v1_rejects_unbracketed_root() {
        let script = r#"
매틱:움직씨 = {
    r <- (((#ascii) 수식{x^2 + 1}), "x", -1, 1, 8) 수치해.이분법.
}
"#;
        let program = DdnProgram::from_source(script, "numeric_root_finding_v1_bad.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("must fail"),
            Err(err) => err,
        };
        assert!(
            err.contains("E_CALC_NUMERIC_BRACKET_SIGN"),
            "expected bracket sign guard, got {err}"
        );
    }

    #[test]
    fn polynomial_solve_minimum_v1_reuses_exact_quadratic_relation_solver() {
        let script = r#"
매틱:움직씨 = {
    f <- (#ascii) 수식{x^2 - 4*x + 4}.
    결과 <- (f, "x") 다항식.풀기.
    미지수 <- 결과.미지수.
    값 <- 결과.값.
}
"#;
        let program =
            DdnProgram::from_source(script, "polynomial_solve_minimum_v1.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("미지수".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값".to_string(), RuntimeValue::Fixed64(Fixed64::ZERO));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("미지수") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "x"),
            other => panic!("미지수 must be string, got {:?}", other),
        }
        let Some(RuntimeValue::Pack(value_pack)) = output.resources.get("값") else {
            panic!("값 must be exact numeric pack");
        };
        assert_eq!(
            value_pack.get("__수타입"),
            Some(&RuntimeValue::String("큰바른수".to_string()))
        );
        assert_eq!(
            value_pack.get("값"),
            Some(&RuntimeValue::String("2".to_string()))
        );
    }

    #[test]
    fn polynomial_solve_minimum_v1_preserves_non_unique_boundary() {
        let script = r#"
매틱:움직씨 = {
    f <- (#ascii) 수식{x^2 - 5*x + 6}.
    결과 <- (f, "x") 다항식.풀기.
    사유 <- 결과.사유.
}
"#;
        let program = DdnProgram::from_source(
            script,
            "polynomial_solve_minimum_v1_non_unique.ddn",
        )
        .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("사유".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");

        match output.resources.get("사유") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "non_unique"),
            other => panic!("사유 must be string, got {:?}", other),
        }
    }

    #[test]
    fn linear_inequality_solve_minimum_v1_returns_bounded_interval() {
        let conditions = Value::List(vec![
            linear_inequality_test_condition("2*x + 1", "이하", 9),
            linear_inequality_test_condition("x - 1", "이상", 2),
        ]);
        let solved = eval_linear_inequality_solve(&[conditions, Value::String("x".to_string())])
            .expect("solve");
        let Value::Pack(output) = solved else {
            panic!("solution must be pack");
        };
        match output.get("상태") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "구간"),
            other => panic!("상태 must be string, got {:?}", other),
        }
        assert_eq!(output.get("하한"), Some(&unit_value_to_value(UnitValue {
            value: Fixed64::from_i64(3),
            dim: UnitDim::NONE,
        })));
        assert_eq!(output.get("상한"), Some(&unit_value_to_value(UnitValue {
            value: Fixed64::from_i64(4),
            dim: UnitDim::NONE,
        })));
        assert_eq!(output.get("하한포함"), Some(&RuntimeValue::Bool(true)));
        assert_eq!(output.get("상한포함"), Some(&RuntimeValue::Bool(true)));
    }

    #[test]
    fn linear_inequality_solve_minimum_v1_detects_empty_interval() {
        let conditions = Value::List(vec![
            linear_inequality_test_condition("x", "미만", 1),
            linear_inequality_test_condition("x", "초과", 1),
        ]);
        let solved = eval_linear_inequality_solve(&[conditions, Value::String("x".to_string())])
            .expect("solve");
        let Value::Pack(output) = solved else {
            panic!("solution must be pack");
        };
        match output.get("상태") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "공집합"),
            other => panic!("상태 must be string, got {:?}", other),
        }
    }

    #[test]
    fn contract_pre_abort_restores_state_before_else_body() {
        let script = r#"
매틱:움직씨 = {
    x <- 5.
    { x > 10 }인것 바탕으로(물림) 아니면 {
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
        assert_eq!(events[0].mode.as_deref(), Some("물림"));
        assert_eq!(events[0].contract_kind.as_deref(), Some("pre"));
    }

    #[test]
    fn contract_post_abort_restores_state_before_repair_body() {
        let script = r#"
매틱:움직씨 = {
    y <- 5.
    { y < 5 }인것 다짐하고(물림) 아니면 {
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
        assert_eq!(events[0].mode.as_deref(), Some("물림"));
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    전이_안전 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 노랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        초록 에서 노랑 으로.
        노랑 에서 빨강 으로.
        바뀔때마다 전이_안전 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
    확인 <- (기계, 다음) 지금상태.
}
"#;
        let program = DdnProgram::from_source(script, "state_machine.ddn").expect("parse");
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    전이_실패 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        바뀔때마다 전이_실패 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program = DdnProgram::from_source(script, "state_machine_fail.ddn").expect("parse");
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    기계 <- 상태머신{
        빨강, 초록 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로.
        바뀔때마다 전이_안전 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_unresolved.ddn").expect("parse");
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
(현재상태:글, 다음상태:글) 기록:움직씨 = {
    횟수 <- 횟수 + 1.
    마지막 <- 다음상태.
}

매틱:움직씨 = {
    거짓_거름 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    참_거름 <- 세움{
        { 현재상태 != 다음상태 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    기계 <- 상태머신{
        빨강, 초록, 파랑 으로 이뤄짐.
        빨강 으로 시작.
        빨강 에서 초록 으로 걸러서 거짓_거름 하고 기록.
        빨강 에서 파랑 으로 걸러서 참_거름 하고 기록.
        바뀔때마다 참_거름 살피기.
    }.
    현재 <- (기계) 처음으로.
    다음 <- (기계, 현재) 다음으로.
}
"#;
        let program =
            DdnProgram::from_source(script, "state_machine_guard_action.ddn").expect("parse");
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    거짓_거름 <- 세움{
        { 현재상태 == 다음상태 }인것 바탕으로(물림) 아니면 {
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age0);
        let err = match DdnProgram::from_source(script, "state_machine_age0.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let program = DdnProgram::from_source(script, "assertion_ok.ddn").expect("parse");
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    x <- 1.
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=0)인 검사 살피기.
}
"#;
        let program = DdnProgram::from_source(script, "assertion_fail.ddn").expect("parse");
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
        let script = r#"
매틱:움직씨 = {
    검사 <- 세움{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age0);
        let err = match DdnProgram::from_source(script, "assertion_age0.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
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
    fn seumssi_v1b_surface_alias_runs_as_canonical_seum() {
        let _age = AgeTargetTestGuard::new(AgeTarget::Age1);
        let script = r#"
매틱:움직씨 = {
    검사 <- 세움씨{
        { 거리 > 0 }인것 바탕으로(물림) 아니면 {
            없음.
        }.
    }.
    결과 <- (거리=3)인 검사 살피기.
}
"#;
        let program = DdnProgram::from_source(script, "seumssi_v1b.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        match output.resources.get("검사") {
            Some(RuntimeValue::Assertion(assertion)) => {
                assert!(assertion.canon.starts_with("세움{"));
                assert!(!assertion.canon.starts_with("세움씨{"));
            }
            other => panic!("검사 must be assertion, got {:?}", other),
        }
        match output.resources.get("결과") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("결과 must be bool, got {:?}", other),
        }
    }

    #[test]
    fn regex_builtins_run_deterministically() {
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
        let program = DdnProgram::from_source(script, "regex.ddn").expect("parse");
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
                assert_eq!(
                    rendered,
                    vec!["b".to_string(), "c".to_string(), "d".to_string()]
                );
            }
            other => panic!("값들 must be list, got {:?}", other),
        }
        match output.resources.get("최근") {
            Some(RuntimeValue::String(value)) => assert_eq!(value, "d"),
            other => panic!("최근 must be string, got {:?}", other),
        }
        assert_eq!(
            extract_fixed(&output.resources, "길이"),
            Fixed64::from_i64(3)
        );
        assert_eq!(
            extract_fixed(&output.resources, "용량"),
            Fixed64::from_i64(3)
        );
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
        assert_eq!(
            extract_fixed(&output.resources, "최근"),
            Fixed64::from_i64(3)
        );
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
        let program =
            DdnProgram::from_source(script, "stream_stdlib_clear_slice.ddn").expect("parse");
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
        let program =
            DdnProgram::from_source(script, "stream_stdlib_label_error.ddn").expect("parse");
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
        let program =
            DdnProgram::from_source(script, "regex_replacement_invalid.ddn").expect("parse");
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
        let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
        let program = DdnProgram::from_source(script, "regex_replacement_numeric_invalid.ddn")
            .expect("parse");
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
            let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
            let program = DdnProgram::from_source(script, file_name).expect("parse");
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
    fn std_grid_closure_runtime_pathfind_and_physics_smoke() {
        let script = r##"
매틱:움직씨 = {
    길 <- ".".
    벽 <- "#".
    격0 <- (4, 3, 길) 격자.만들기.
    격1 <- (격0, 1, 0, 벽) 격자.바꾼값.
    격2 <- (격1, 1, 1, 벽) 격자.바꾼값.
    격3 <- (격2, 3, 1, 벽) 격자.바꾼값.
    경로 <- (격3, 0, 0, 3, 2, (벽) 차림) 격자.길찾기.
    경로길이 <- (경로) 길이.
    다음위치 <- (10, 2, 3) 물리1d.위치갱신.
}
"##;
        let program = DdnProgram::from_source(script, "std_grid_closure.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "경로길이".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        defaults.insert(
            "다음위치".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(0)),
        );
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            extract_fixed(&output.resources, "경로길이"),
            Fixed64::from_i64(6)
        );
        assert_eq!(
            extract_fixed(&output.resources, "다음위치"),
            Fixed64::from_i64(16)
        );
    }

    #[test]
    fn std_block_piece_geometry_moves_and_rotates() {
        let script = r#"
매틱:움직씨 = {
    조각 <- (((0, 0) 차림, (1, 0) 차림, (0, 1) 차림) 차림) 블록조각.만들기.
    원본 <- (조각) 블록조각.칸목록.
    이동 <- ((조각, 2, 1) 블록조각.이동) 블록조각.칸목록.
    오른쪽 <- ((조각, "오른쪽") 블록조각.회전) 블록조각.칸목록.
    왼쪽 <- ((조각, "왼쪽") 블록조각.회전) 블록조각.칸목록.
    뒤집기 <- ((조각, "뒤집기") 블록조각.회전) 블록조각.칸목록.
}
"#;
        let program =
            DdnProgram::from_source(script, "std_block_piece_geometry.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["원본", "이동", "오른쪽", "왼쪽", "뒤집기"] {
            defaults.insert(key.to_string(), RuntimeValue::List(Vec::new()));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("원본").map(value_to_string),
            Some("차림[차림[0, 0], 차림[1, 0], 차림[0, 1]]".to_string())
        );
        assert_eq!(
            output.resources.get("이동").map(value_to_string),
            Some("차림[차림[2, 1], 차림[3, 1], 차림[2, 2]]".to_string())
        );
        assert_eq!(
            output.resources.get("오른쪽").map(value_to_string),
            Some("차림[차림[-1, 0], 차림[0, 0], 차림[0, 1]]".to_string())
        );
        assert_eq!(
            output.resources.get("왼쪽").map(value_to_string),
            Some("차림[차림[0, -1], 차림[0, 0], 차림[1, 0]]".to_string())
        );
        assert_eq!(
            output.resources.get("뒤집기").map(value_to_string),
            Some("차림[차림[0, -1], 차림[-1, 0], 차림[0, 0]]".to_string())
        );
    }

    #[test]
    fn std_block_piece_grid_bridge_collides_and_locks() {
        let script = r##"
매틱:움직씨 = {
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
}
"##;
        let program =
            DdnProgram::from_source(script, "std_block_piece_grid_bridge.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["안쪽충돌", "벽충돌", "밖충돌"] {
            defaults.insert(key.to_string(), RuntimeValue::Bool(false));
        }
        defaults.insert("고정값".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("원래벽".to_string(), RuntimeValue::String(String::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("안쪽충돌"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            output.resources.get("벽충돌"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("밖충돌"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("고정값"),
            Some(&RuntimeValue::String("X".to_string()))
        );
        assert_eq!(
            output.resources.get("원래벽"),
            Some(&RuntimeValue::String("#".to_string()))
        );
    }

    #[test]
    fn std_random_bag_draws_refills_and_previews() {
        let script = r#"
매틱:움직씨 = {
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
}
"#;
        let program = DdnProgram::from_source(script, "std_random_bag.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("미리".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("값1".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값2".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값3".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("값4".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("빈가".to_string(), RuntimeValue::Bool(false));
        defaults.insert("남은3".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("남은4".to_string(), RuntimeValue::List(Vec::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("미리").map(value_to_string),
            Some("차림[O, I, T, O, T]".to_string())
        );
        assert_eq!(
            output.resources.get("값1"),
            Some(&RuntimeValue::String("O".to_string()))
        );
        assert_eq!(
            output.resources.get("값2"),
            Some(&RuntimeValue::String("I".to_string()))
        );
        assert_eq!(
            output.resources.get("값3"),
            Some(&RuntimeValue::String("T".to_string()))
        );
        assert_eq!(
            output.resources.get("빈가"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("남은3").map(value_to_string),
            Some("차림[]".to_string())
        );
        assert_eq!(
            output.resources.get("값4"),
            Some(&RuntimeValue::String("O".to_string()))
        );
        assert_eq!(
            output.resources.get("남은4").map(value_to_string),
            Some("차림[I, T]".to_string())
        );
    }

    #[test]
    fn std_grid_game_state_lifecycle_pause_resume() {
        let script = r#"
매틱:움직씨 = {
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
}
"#;
        let program = DdnProgram::from_source(script, "std_grid_game_state.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("상태0".to_string(), RuntimeValue::String(String::new()));
        defaults.insert(
            "틱0".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
        );
        defaults.insert("진행인가".to_string(), RuntimeValue::Bool(false));
        defaults.insert("멈춤상태".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("재개상태".to_string(), RuntimeValue::String(String::new()));
        defaults.insert(
            "재개틱".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
        );
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("상태0"),
            Some(&RuntimeValue::String("준비".to_string()))
        );
        assert_eq!(
            output.resources.get("틱0"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(0)))
        );
        assert_eq!(
            output.resources.get("진행인가"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("멈춤상태"),
            Some(&RuntimeValue::String("멈춤".to_string()))
        );
        assert_eq!(
            output.resources.get("재개상태"),
            Some(&RuntimeValue::String("진행".to_string()))
        );
        assert_eq!(
            output.resources.get("재개틱"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(3)))
        );
    }

    #[test]
    fn std_grid_game_playable_catalog_lines_falling_score_and_tick() {
        let script = r#"
매틱:움직씨 = {
    이름들 <- () 테트로미노.이름목록.
    아이 <- ("I") 테트로미노.만들기.
    아이칸 <- (아이) 블록조각.칸목록.
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
    오 <- ("O") 테트로미노.만들기.
    낙0 <- (오, 2, 3) 낙하조각.만들기.
    배치0 <- (낙0) 낙하조각.배치.
    점0 <- () 격자게임점수.초기화.
    점1 <- (점0, 4) 격자게임점수.더하기.
    점2 <- (점1, 4) 격자게임점수.더하기.
    점3 <- (점2, 2) 격자게임점수.더하기.
    점4 <- (점3, 1) 격자게임점수.더하기.
    점수4 <- (점4) 격자게임점수.점수.
    줄4 <- (점4) 격자게임점수.줄수.
    레벨4 <- (점4) 격자게임점수.레벨.
    격 <- (4, 4, 빈) 격자.만들기.
    가방 <- (0, ("O", "I") 차림) 무작위가방.만들기.
    상태0 <- () 격자게임상태.초기화.
    진행상태 <- (상태0, "진행") 격자게임상태.바꾸기.
    낙 <- (오, 1, 0) 낙하조각.만들기.
    세션 <- (격, 가방, 진행상태, 점0, 낙) 격자게임세션.만들기.
    조작맵 <- () 입력사상.만들기.
    틱 <- (세션, 조작맵) 격자게임.한틱.
    고정 <- 틱.고정됐나.
    다음세션 <- 틱.세션.
    다음낙 <- (다음세션) 격자게임세션.낙하조각.
    위치 <- (다음낙) 낙하조각.위치.
}
"#;
        let program = DdnProgram::from_source(script, "std_grid_game_playable.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["이름들", "아이칸", "찬줄", "배치0", "위치"] {
            defaults.insert(key.to_string(), RuntimeValue::List(Vec::new()));
        }
        for key in ["지운수", "점수4", "줄4", "레벨4"] {
            defaults.insert(
                key.to_string(),
                RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
            );
        }
        defaults.insert("아래중".to_string(), RuntimeValue::String(String::new()));
        defaults.insert("고정".to_string(), RuntimeValue::Bool(true));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("이름들").map(value_to_string),
            Some("차림[I, O, T, S, Z, J, L]".to_string())
        );
        assert_eq!(
            output.resources.get("아이칸").map(value_to_string),
            Some("차림[차림[-1, 0], 차림[0, 0], 차림[1, 0], 차림[2, 0]]".to_string())
        );
        assert_eq!(
            output.resources.get("찬줄").map(value_to_string),
            Some("차림[2]".to_string())
        );
        assert_eq!(
            output.resources.get("지운수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        assert_eq!(
            output.resources.get("아래중"),
            Some(&RuntimeValue::String("P".to_string()))
        );
        assert_eq!(
            output.resources.get("배치0").map(value_to_string),
            Some("차림[차림[2, 3], 차림[3, 3], 차림[2, 4], 차림[3, 4]]".to_string())
        );
        assert_eq!(
            output.resources.get("점수4"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(2100)))
        );
        assert_eq!(
            output.resources.get("줄4"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(11)))
        );
        assert_eq!(
            output.resources.get("레벨4"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(2)))
        );
        assert_eq!(
            output.resources.get("고정"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            output.resources.get("위치").map(value_to_string),
            Some("차림[1, 1]".to_string())
        );
    }

    #[test]
    fn std_grid_game_view_projects_text_cells_and_summary() {
        let script = r#"
매틱:움직씨 = {
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
}
"#;
        let program = DdnProgram::from_source(script, "std_grid_game_view.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "칸수".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
        );
        for key in ["원천0", "원천1", "원천12", "값1", "문자판", "요약종류"] {
            defaults.insert(key.to_string(), RuntimeValue::String(String::new()));
        }
        for key in ["요약틱", "요약점수", "요약줄수", "요약레벨"] {
            defaults.insert(
                key.to_string(),
                RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
            );
        }
        defaults.insert("요약위치".to_string(), RuntimeValue::List(Vec::new()));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("칸수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(16)))
        );
        assert_eq!(
            output.resources.get("원천0"),
            Some(&RuntimeValue::String("빈칸".to_string()))
        );
        assert_eq!(
            output.resources.get("원천1"),
            Some(&RuntimeValue::String("낙하".to_string()))
        );
        assert_eq!(
            output.resources.get("원천12"),
            Some(&RuntimeValue::String("고정".to_string()))
        );
        assert_eq!(
            output.resources.get("값1"),
            Some(&RuntimeValue::String("O".to_string()))
        );
        assert_eq!(
            output.resources.get("문자판"),
            Some(&RuntimeValue::String(".OO.\n.OO.\n....\nX...".to_string()))
        );
        assert_eq!(
            output.resources.get("요약종류"),
            Some(&RuntimeValue::String(
                "std_grid_game_view_summary".to_string()
            ))
        );
        assert_eq!(
            output.resources.get("요약틱"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        assert_eq!(
            output.resources.get("요약점수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(0)))
        );
        assert_eq!(
            output.resources.get("요약줄수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(0)))
        );
        assert_eq!(
            output.resources.get("요약레벨"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        assert_eq!(
            output.resources.get("요약위치").map(value_to_string),
            Some("차림[1, 0]".to_string())
        );
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("요약") else {
            panic!("요약 must be pack");
        };
        assert_eq!(
            fields.get("상태"),
            Some(&RuntimeValue::String("진행".to_string()))
        );
    }

    #[test]
    fn std_grid_game_bogae_projects_rect_drawlist_and_size() {
        let script = r##"
매틱:움직씨 = {
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
}
"##;
        let program = DdnProgram::from_source(script, "std_grid_game_bogae.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert(
            "칸수".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
        );
        for key in ["첫id", "첫결", "첫색", "둘id", "둘색", "고정id", "고정색"] {
            defaults.insert(key.to_string(), RuntimeValue::String(String::new()));
        }
        for key in ["첫x", "둘x", "고정y", "가로", "세로"] {
            defaults.insert(
                key.to_string(),
                RuntimeValue::Fixed64(Fixed64::from_i64(-1)),
            );
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("칸수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(16)))
        );
        assert_eq!(
            output.resources.get("첫id"),
            Some(&RuntimeValue::String("격자게임셀_0_0".to_string()))
        );
        assert_eq!(
            output.resources.get("첫결"),
            Some(&RuntimeValue::String("#보개/2D.Rect".to_string()))
        );
        assert_eq!(
            output.resources.get("첫x"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(0)))
        );
        assert_eq!(
            output.resources.get("첫색"),
            Some(&RuntimeValue::String("#111111ff".to_string()))
        );
        assert_eq!(
            output.resources.get("둘id"),
            Some(&RuntimeValue::String("격자게임셀_0_1".to_string()))
        );
        assert_eq!(
            output.resources.get("둘x"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(8)))
        );
        assert_eq!(
            output.resources.get("둘색"),
            Some(&RuntimeValue::String("#ffcc00ff".to_string()))
        );
        assert_eq!(
            output.resources.get("고정id"),
            Some(&RuntimeValue::String("격자게임셀_3_0".to_string()))
        );
        assert_eq!(
            output.resources.get("고정y"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(24)))
        );
        assert_eq!(
            output.resources.get("고정색"),
            Some(&RuntimeValue::String("#4a90e2ff".to_string()))
        );
        assert_eq!(
            output.resources.get("가로"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(32)))
        );
        assert_eq!(
            output.resources.get("세로"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(32)))
        );
    }

    #[test]
    fn std_grid_game_rules_hold_ghost_and_wall_kick_helpers() {
        let script = r##"
매틱:움직씨 = {
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
}
"##;
        let program = DdnProgram::from_source(script, "std_grid_game_rules.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["홀드1칸", "낙1위치", "유령위치", "회전오프셋", "회전위치"] {
            defaults.insert(key.to_string(), RuntimeValue::None);
        }
        for key in ["안씀", "홀드1씀", "홀드2씀", "회전성공"] {
            defaults.insert(key.to_string(), RuntimeValue::Bool(false));
        }
        for key in ["낙하색", "유령색"] {
            defaults.insert(key.to_string(), RuntimeValue::String(String::new()));
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("안씀"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            output.resources.get("홀드1칸").map(value_to_string),
            Some("(__종류: std_block_piece, 칸들: 차림[차림[-1, 0], 차림[0, 0], 차림[1, 0], 차림[0, 1]])".to_string())
        );
        assert_eq!(
            output.resources.get("홀드1씀"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("낙1위치").map(value_to_string),
            Some("차림[2, 0]".to_string())
        );
        assert_eq!(
            output.resources.get("홀드2씀"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            output.resources.get("유령위치").map(value_to_string),
            Some("차림[1, 4]".to_string())
        );
        assert_eq!(
            output.resources.get("낙하색"),
            Some(&RuntimeValue::String("#ffcc00ff".to_string()))
        );
        assert_eq!(
            output.resources.get("유령색"),
            Some(&RuntimeValue::String("#88ffffff".to_string()))
        );
        assert_eq!(
            output.resources.get("회전성공"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("회전오프셋").map(value_to_string),
            Some("차림[1, 0]".to_string())
        );
        assert_eq!(
            output.resources.get("회전위치").map(value_to_string),
            Some("차림[1, 1]".to_string())
        );
    }

    #[test]
    fn std_unit_closure_temperature_smoke() {
        let script = r#"
매틱:움직씨 = {
    온도동치 <- 25@C == 77@F.
    온도차이확인 <- (30@C - 20@C) == 10@K.
}
"#;
        let program = DdnProgram::from_source(script, "std_unit_closure.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("온도동치".to_string(), RuntimeValue::Bool(false));
        defaults.insert("온도차이확인".to_string(), RuntimeValue::Bool(false));
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        for key in ["온도동치", "온도차이확인"] {
            match output.resources.get(key) {
                Some(RuntimeValue::Bool(value)) => assert!(*value, "{key}"),
                other => panic!("{key} must be bool, got {:?}", other),
            }
        }
    }

    #[test]
    fn std_input_map_closure_runtime_maps_explicit_and_default_keys() {
        let explicit_map = make_std_input_map(BTreeMap::from([
            (
                "왼쪽".to_string(),
                RuntimeValue::String("ArrowLeft".to_string()),
            ),
            (
                "오른쪽".to_string(),
                RuntimeValue::String("ArrowRight".to_string()),
            ),
            (
                "위".to_string(),
                RuntimeValue::String("ArrowUp".to_string()),
            ),
            (
                "아래".to_string(),
                RuntimeValue::String("ArrowDown".to_string()),
            ),
            (
                "확인".to_string(),
                RuntimeValue::String("Space".to_string()),
            ),
        ]));
        let explicit_fields = std_input_map_fields(&explicit_map).expect("input map fields");
        let input_state = InputState::new(KEY_D, 0);
        assert!(input_map_action_pressed(
            &input_state,
            &explicit_fields,
            "오른쪽"
        ));
        assert!(!input_map_action_pressed(
            &input_state,
            &explicit_fields,
            "확인"
        ));

        let script = r#"
매틱:움직씨 = {
    기본 <- () 입력사상.만들기.
    방향 <- (기본) 입력사상.방향.
    오른쪽 <- (기본, "오른쪽") 입력사상.동작.
    확인 <- (기본, "확인") 입력사상.동작.
}
"#;
        let program = DdnProgram::from_source(script, "std_input_map_closure.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut input = empty_input();
        input.keys_pressed = KEY_D;
        input.last_key_name = "ArrowRight".to_string();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("방향".to_string(), RuntimeValue::List(Vec::new()));
        defaults.insert("오른쪽".to_string(), RuntimeValue::Bool(false));
        defaults.insert("확인".to_string(), RuntimeValue::Bool(true));
        let output = runner
            .run_update(&world, &input, &defaults)
            .expect("run update");
        assert_eq!(
            output.resources.get("방향").map(value_to_string),
            Some("차림[1, 0]".to_string())
        );
        assert_eq!(
            output.resources.get("오른쪽"),
            Some(&RuntimeValue::Bool(true))
        );
        assert_eq!(
            output.resources.get("확인"),
            Some(&RuntimeValue::Bool(false))
        );
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
        let script = r#"
매틱:움직씨 = {
    결과 <- ("a1", 정규식{"[0-9]+"}) 정규찾기.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age2);
        let err = match DdnProgram::from_source(script, "regex_age2.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
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
        let script = r#"
매틱:움직씨 = {
    n 이 자연수 낱낱에 대해 {
        없음.
    }.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age3);
        let err = match DdnProgram::from_source(script, "quantifier_age3.ddn") {
            Ok(_) => panic!("age gate"),
            Err(err) => err,
        };
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
        let script = r#"
매틱:움직씨 = {
    결과 <- 3.
    n 이 자연수 낱낱에 대해 {
        없음.
    }.
}
"#;
        let _age = AgeTargetTestGuard::new(AgeTarget::Age4);
        let program = DdnProgram::from_source(script, "quantifier_age4.ddn").expect("parse");
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
                assert_eq!(text, "123456789012345678901234567890|1/2|2^2 * 3 * 7");
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
    fn relation_solve_surface_parses_and_runs() {
        let script = r#"
매틱:움직씨 = {
    rel <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
    결과 <- (rel) 방정식풀기.
}
"#;
        let program = DdnProgram::from_source(script, "relation_solve.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("rel".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("결과") else {
            panic!("결과 must be relation solve pack");
        };
        assert_eq!(
            fields.get(RELATION_SOLVE_RESULT_KIND_FIELD),
            Some(&RuntimeValue::String(
                RELATION_SOLVE_RESULT_SUCCESS.to_string()
            ))
        );
        assert_eq!(
            fields.get(RELATION_SOLVE_VAR_FIELD),
            Some(&RuntimeValue::String("x".to_string()))
        );
    }

    #[test]
    fn relation_eq_infix_runtime_stores_relation_pack() {
        let script = r#"
매틱:움직씨 = {
    관계 <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
}
"#;
        let program = DdnProgram::from_source(script, "relation_eq_pack.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("관계") else {
            panic!("관계 must be relation pack");
        };
        assert_eq!(
            fields.get(RELATION_KIND_FIELD),
            Some(&RuntimeValue::String(RELATION_KIND_EQUATION.to_string()))
        );
        assert!(matches!(
            fields.get(RELATION_LEFT_FIELD),
            Some(RuntimeValue::Formula(_))
        ));
        assert!(matches!(
            fields.get(RELATION_RIGHT_FIELD),
            Some(RuntimeValue::Formula(_))
        ));
    }

    #[test]
    fn connect_jitgi_runtime_matches_relation_eq_pack() {
        let script = r#"
매틱:움직씨 = {
    잇기관계 <- ((#ascii) 수식{2*x + 3}, (#ascii) 수식{7}) 잇기.
    직접관계 <- ((#ascii) 수식{2*x + 3}) =:= ((#ascii) 수식{7}).
}
"#;
        let program = DdnProgram::from_source(script, "connect_jitgi_pack.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("잇기관계".to_string(), RuntimeValue::None);
        defaults.insert("직접관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(jitgi)) = output.resources.get("잇기관계") else {
            panic!("잇기관계 must be relation pack");
        };
        let Some(RuntimeValue::Pack(direct)) = output.resources.get("직접관계") else {
            panic!("직접관계 must be relation pack");
        };
        assert_eq!(jitgi, direct);
        assert_eq!(
            jitgi.get(RELATION_KIND_FIELD),
            Some(&RuntimeValue::String(RELATION_KIND_EQUATION.to_string()))
        );
    }

    #[test]
    fn connect_endpoint_equal_runtime_stores_endpoint_relation_pack() {
        let script = r#"
매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}
"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_equal.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_equality".to_string()))
        );
        assert_eq!(
            fields.get("왼쪽"),
            Some(&RuntimeValue::String("전지.양극.전압".to_string()))
        );
        assert_eq!(
            fields.get("오른쪽"),
            Some(&RuntimeValue::String("전구.왼핀.전압".to_string()))
        );
        assert_eq!(
            fields.get("규칙"),
            Some(&RuntimeValue::String("같게".to_string()))
        );
        assert_eq!(
            fields.get("채널"),
            Some(&RuntimeValue::String("전압".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_flow_runtime_stores_endpoint_flow_pack() {
        let script = r#"
매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}
"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_flow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint flow pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_flow".to_string()))
        );
        assert_eq!(
            fields.get("왼쪽"),
            Some(&RuntimeValue::String("전지.양극.전류".to_string()))
        );
        assert_eq!(
            fields.get("오른쪽"),
            Some(&RuntimeValue::String("전구.왼핀.전류".to_string()))
        );
        assert_eq!(
            fields.get("규칙"),
            Some(&RuntimeValue::String("흐르게".to_string()))
        );
        assert_eq!(
            fields.get("채널"),
            Some(&RuntimeValue::String("전류".to_string()))
        );
        assert_eq!(
            fields.get("부호규약"),
            Some(&RuntimeValue::String("left_plus_right_zero".to_string()))
        );
        assert_eq!(
            fields.get("방향"),
            Some(&RuntimeValue::String("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_reverse_flow_runtime_stores_endpoint_flow_pack() {
        let script = r#"
매틱:움직씨 = {
    이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 거슬러 흐르게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_reverse_flow.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint reverse flow pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_flow".to_string()))
        );
        assert_eq!(
            fields.get("왼쪽"),
            Some(&RuntimeValue::String("가계1.구매끝.돈".to_string()))
        );
        assert_eq!(
            fields.get("오른쪽"),
            Some(&RuntimeValue::String("장터.소매끝.돈".to_string()))
        );
        assert_eq!(
            fields.get("규칙"),
            Some(&RuntimeValue::String("거슬러 흐르게".to_string()))
        );
        assert_eq!(
            fields.get("채널"),
            Some(&RuntimeValue::String("돈".to_string()))
        );
        assert_eq!(
            fields.get("부호규약"),
            Some(&RuntimeValue::String("left_plus_right_zero".to_string()))
        );
        assert_eq!(
            fields.get("방향"),
            Some(&RuntimeValue::String("오른쪽에서왼쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_rejects_carried_property_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (재화가 돈에 실리게) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_carried.ddn").is_err());
    }

    #[test]
    fn connect_endpoint_carried_property_forward_runtime_stores_relation_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_carried_forward.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_relation_set".to_string()))
        );
        let Some(RuntimeValue::List(relations)) = fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(carried) = &relations[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_carried_property".to_string()
            ))
        );
        assert_eq!(
            carried.get("왼쪽운반자"),
            Some(&RuntimeValue::String("은행.대출창구.대출금".to_string()))
        );
        assert_eq!(
            carried.get("오른쪽운반자"),
            Some(&RuntimeValue::String("기업1.차입끝.대출금".to_string()))
        );
        assert_eq!(
            carried.get("속성"),
            Some(&RuntimeValue::String("위험".to_string()))
        );
        assert_eq!(
            carried.get("운반채널"),
            Some(&RuntimeValue::String("대출금".to_string()))
        );
        assert_eq!(
            carried.get("운반규칙"),
            Some(&RuntimeValue::String("흐르게".to_string()))
        );
        assert_eq!(
            carried.get("운반방향"),
            Some(&RuntimeValue::String("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_carried_property_reverse_runtime_stores_relation_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 거슬러 흐르게, 재화가 돈에 실리게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_carried_reverse.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        let Some(RuntimeValue::List(relations)) = fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(carried) = &relations[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_carried_property".to_string()
            ))
        );
        assert_eq!(
            carried.get("속성"),
            Some(&RuntimeValue::String("재화".to_string()))
        );
        assert_eq!(
            carried.get("운반채널"),
            Some(&RuntimeValue::String("돈".to_string()))
        );
        assert_eq!(
            carried.get("운반규칙"),
            Some(&RuntimeValue::String("거슬러 흐르게".to_string()))
        );
        assert_eq!(
            carried.get("운반방향"),
            Some(&RuntimeValue::String("오른쪽에서왼쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_multi_inner_runtime_stores_relation_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전류는 흐르게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_multi.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_relation_set".to_string()))
        );
        assert_eq!(
            fields.get("왼쪽끝"),
            Some(&RuntimeValue::String("전지.양극".to_string()))
        );
        assert_eq!(
            fields.get("오른쪽끝"),
            Some(&RuntimeValue::String("전구.왼핀".to_string()))
        );
        let Some(RuntimeValue::List(relations)) = fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(first) = &relations[0] else {
            panic!("first relation must be a pack");
        };
        let RuntimeValue::Pack(second) = &relations[1] else {
            panic!("second relation must be a pack");
        };
        assert_eq!(
            first.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_equality".to_string()))
        );
        assert_eq!(
            first.get("채널"),
            Some(&RuntimeValue::String("전압".to_string()))
        );
        assert_eq!(
            second.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_flow".to_string()))
        );
        assert_eq!(
            second.get("채널"),
            Some(&RuntimeValue::String("전류".to_string()))
        );
        assert_eq!(
            second.get("방향"),
            Some(&RuntimeValue::String("왼쪽에서오른쪽".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_multi_inner_econ_runtime_preserves_source_order() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 가계1.구매끝과 장터.소매끝을 (체결값은 같게, 재화는 흐르게, 돈은 거슬러 흐르게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_multi_econ.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint relation set pack");
        };
        let Some(RuntimeValue::List(relations)) = fields.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 3);
        let channels = relations
            .iter()
            .map(|item| {
                let RuntimeValue::Pack(fields) = item else {
                    panic!("relation item must be a pack");
                };
                fields.get("채널").cloned()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            channels,
            vec![
                Some(RuntimeValue::String("체결값".to_string())),
                Some(RuntimeValue::String("재화".to_string())),
                Some(RuntimeValue::String("돈".to_string()))
            ]
        );
    }

    #[test]
    fn connect_endpoint_statement_append_same_pair_runtime_stores_statement_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_statement_append_same.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint statement set pack");
        };
        assert_eq!(
            fields.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_statement_set".to_string()))
        );
        assert_eq!(
            fields.get("대상"),
            Some(&RuntimeValue::String("이음관계".to_string()))
        );
        let Some(RuntimeValue::List(items)) = fields.get("이음들") else {
            panic!("이음들 must be a list");
        };
        assert_eq!(items.len(), 2);
        let RuntimeValue::Pack(first) = &items[0] else {
            panic!("first statement must be a pack");
        };
        let RuntimeValue::Pack(second) = &items[1] else {
            panic!("second statement must be a pack");
        };
        assert_eq!(
            first.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_equality".to_string()))
        );
        assert_eq!(
            second.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_flow".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_statement_append_mixed_pair_runtime_stores_statement_set_pack() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_statement_append_mixed.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("이음관계") else {
            panic!("이음관계 must be endpoint statement set pack");
        };
        let Some(RuntimeValue::List(items)) = fields.get("이음들") else {
            panic!("이음들 must be a list");
        };
        assert_eq!(items.len(), 2);
        let RuntimeValue::Pack(second) = &items[1] else {
            panic!("second statement must be a pack");
        };
        assert_eq!(
            second.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_relation_set".to_string()))
        );
        let Some(RuntimeValue::List(relations)) = second.get("관계들") else {
            panic!("mixed second statement 관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(carried) = &relations[1] else {
            panic!("carried relation must be a pack");
        };
        assert_eq!(
            carried.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_carried_property".to_string()
            ))
        );
    }

    #[test]
    fn connect_endpoint_normalize_single_runtime_returns_one_relation() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    목록 <- (이음관계) 이음관계.관계목록.
    정규 <- (이음관계) 이음관계.정규화.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_normalize_single.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("목록".to_string(), RuntimeValue::None);
        defaults.insert("정규".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::List(list)) = output.resources.get("목록") else {
            panic!("목록 must be a list");
        };
        assert_eq!(list.len(), 1);
        let RuntimeValue::Pack(first) = &list[0] else {
            panic!("first relation must be a pack");
        };
        assert_eq!(
            first.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_equality".to_string()))
        );
        let Some(RuntimeValue::Pack(normalized)) = output.resources.get("정규") else {
            panic!("정규 must be a pack");
        };
        assert_eq!(
            normalized.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_relation_flat_set".to_string()
            ))
        );
        assert!(matches!(
            normalized.get("개수"),
            Some(RuntimeValue::Fixed64(value)) if value.int_part() == 1
        ));
    }

    #[test]
    fn connect_endpoint_normalize_statement_set_flattens_nested_relation_set() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
    목록 <- (이음관계) 이음관계.관계목록.
    정규 <- (이음관계) 이음관계.정규화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_normalize_statement_set.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("목록".to_string(), RuntimeValue::None);
        defaults.insert("정규".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::List(list)) = output.resources.get("목록") else {
            panic!("목록 must be a list");
        };
        assert_eq!(list.len(), 3);
        let kinds = list
            .iter()
            .map(|item| {
                let RuntimeValue::Pack(fields) = item else {
                    panic!("relation item must be a pack");
                };
                fields.get("__이음관계종류").cloned()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                Some(RuntimeValue::String("endpoint_equality".to_string())),
                Some(RuntimeValue::String("endpoint_flow".to_string())),
                Some(RuntimeValue::String(
                    "endpoint_carried_property".to_string()
                )),
            ]
        );
        let Some(RuntimeValue::Pack(normalized)) = output.resources.get("정규") else {
            panic!("정규 must be a pack");
        };
        assert!(matches!(
            normalized.get("개수"),
            Some(RuntimeValue::Fixed64(value)) if value.int_part() == 3
        ));
        let Some(RuntimeValue::List(relations)) = normalized.get("관계들") else {
            panic!("normalized 관계들 must be a list");
        };
        assert_eq!(relations.len(), 3);
    }

    #[test]
    fn connect_endpoint_formula_relation_bridge_maps_endpoint_paths_to_ascii_vars() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    관계들 <- (이음관계) 이음관계.방정식목록.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_formula_relation_bridge.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("관계들".to_string(), RuntimeValue::None);
        defaults.insert("방정식묶음".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::List(relations)) = output.resources.get("관계들") else {
            panic!("관계들 must be a list");
        };
        assert_eq!(relations.len(), 2);
        let RuntimeValue::Pack(first_relation) = &relations[0] else {
            panic!("first formula relation must be a pack");
        };
        assert_eq!(
            first_relation.get("__관계종류"),
            Some(&RuntimeValue::String("방정식".to_string()))
        );
        let Some(RuntimeValue::Formula(left)) = first_relation.get("왼쪽") else {
            panic!("first left must be formula");
        };
        let Some(RuntimeValue::Formula(right)) = first_relation.get("오른쪽") else {
            panic!("first right must be formula");
        };
        assert_eq!(left.raw, "ep_001");
        assert_eq!(right.raw, "ep_002");

        let Some(RuntimeValue::Pack(set)) = output.resources.get("방정식묶음") else {
            panic!("방정식묶음 must be a pack");
        };
        assert_eq!(
            set.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_formula_relation_set".to_string()
            ))
        );
        let Some(RuntimeValue::List(mappings)) = set.get("변수사상") else {
            panic!("변수사상 must be a list");
        };
        assert_eq!(mappings.len(), 2);
        let RuntimeValue::Pack(first_mapping) = &mappings[0] else {
            panic!("first mapping must be a pack");
        };
        assert_eq!(
            first_mapping.get("변수"),
            Some(&RuntimeValue::String("ep_001".to_string()))
        );
        assert_eq!(
            first_mapping.get("경로"),
            Some(&RuntimeValue::String("전지.양극.전압".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_formula_relation_solve_uses_explicit_relation_list() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
    관계들 <- 방정식묶음.관계들.
    결과 <- (관계들) 방정식풀기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_formula_relation_solve.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("방정식묶음".to_string(), RuntimeValue::None);
        defaults.insert("관계들".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(result)) = output.resources.get("결과") else {
            panic!("결과 must be a pack");
        };
        assert_eq!(
            result.get("__풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::Pack(bindings)) = result.get("해") else {
            panic!("해 must be a pack");
        };
        assert!(bindings.contains_key("ep_001"));
        assert!(bindings.contains_key("ep_002"));
    }

    #[test]
    fn connect_endpoint_formula_relation_rejects_carried_property_metadata() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_formula_relation_unsupported.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        defaults.insert("방정식묶음".to_string(), RuntimeValue::None);
        assert!(runner
            .run_update(&world, &empty_input(), &defaults)
            .is_err());
    }

    #[test]
    fn connect_endpoint_boundary_value_injection_appends_relation_packs() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_boundary_value_injection.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();
        let value_list = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let injected_relation_list =
            eval_endpoint_boundary_value_relation_list(&[formula_set.clone(), value_list.clone()])
                .expect("value relation list");
        let RuntimeValue::List(injected_relations) = injected_relation_list else {
            panic!("값관계들 must be a list");
        };
        assert_eq!(injected_relations.len(), 1);
        let RuntimeValue::Pack(injected_relation) = &injected_relations[0] else {
            panic!("injected relation must be a pack");
        };
        assert_eq!(
            injected_relation.get("__관계종류"),
            Some(&RuntimeValue::String("방정식".to_string()))
        );
        let Some(RuntimeValue::Formula(left)) = injected_relation.get("왼쪽") else {
            panic!("injected left must be formula");
        };
        let Some(RuntimeValue::Formula(right)) = injected_relation.get("오른쪽") else {
            panic!("injected right must be formula");
        };
        assert_eq!(left.raw, "ep_001");
        assert_eq!(right.raw, "5");

        let injected_set =
            eval_endpoint_boundary_value_injection(&[formula_set, value_list]).expect("inject");
        let RuntimeValue::Pack(set) = injected_set else {
            panic!("주입묶음 must be a pack");
        };
        assert_eq!(
            set.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_formula_relation_set_with_values".to_string()
            ))
        );
        assert_eq!(
            set.get("개수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(2)))
        );
        assert_eq!(
            set.get("주입개수"),
            Some(&RuntimeValue::Fixed64(Fixed64::from_i64(1)))
        );
        let Some(RuntimeValue::List(all_relations)) = set.get("관계들") else {
            panic!("combined 관계들 must be a list");
        };
        assert_eq!(all_relations.len(), 2);
    }

    #[test]
    fn connect_endpoint_boundary_value_solve_remap_accepts_injected_formula_set() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_boundary_value_solve.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();
        let value_list = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let injected_set =
            eval_endpoint_boundary_value_injection(&[formula_set.clone(), value_list])
                .expect("inject");
        let RuntimeValue::Pack(injected_fields) = &injected_set else {
            panic!("주입묶음 must be a pack");
        };
        let relations_value = injected_fields.get("관계들").expect("관계들").clone();
        let relations = expect_equation_relations(&[relations_value]).expect("relations");
        let solve = eval_relation_solve_result(&relations).expect("solve");
        let remapped = eval_endpoint_solve_result_remap(&[injected_set, solve]).expect("remap");
        let RuntimeValue::Pack(remap) = remapped else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 2);
        let RuntimeValue::Pack(first) = &values[0] else {
            panic!("first value must be a pack");
        };
        assert_eq!(
            first.get("경로"),
            Some(&RuntimeValue::String("전지.양극.전압".to_string()))
        );
        assert_eq!(first.get("값").map(value_to_string).as_deref(), Some("5"));
    }

    #[test]
    fn connect_endpoint_boundary_value_rejects_unsupported_inputs() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_boundary_value_bad_base.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for key in ["이음관계", "방정식묶음"] {
            defaults.insert(key.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();

        let duplicate =
            endpoint_boundary_test_values(&[("전지.양극.전압", 5), ("전지.양극.전압", 6)]);
        let err = eval_endpoint_boundary_value_injection(&[formula_set.clone(), duplicate])
            .expect_err("duplicate must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_value_duplicate_path"),
            "{err}"
        );

        let unknown = endpoint_boundary_test_values(&[("없는.끝.전압", 5)]);
        let err = eval_endpoint_boundary_value_injection(&[formula_set.clone(), unknown])
            .expect_err("unknown must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_value_unknown_path"),
            "{err}"
        );

        let bad_value = endpoint_boundary_test_values_raw(vec![(
            "전지.양극.전압",
            RuntimeValue::String("5".to_string()),
        )]);
        let err = eval_endpoint_boundary_value_injection(&[formula_set, bad_value])
            .expect_err("non numeric must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_value_non_numeric"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_equal_value_returns_endpoint_values() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_explicit_solve_equal.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let remapped = eval_endpoint_explicit_solve(&[relation, values]).expect("explicit solve");
        let RuntimeValue::Pack(remap) = remapped else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_solve_result".to_string()))
        );
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 2);
        let RuntimeValue::Pack(first) = &values[0] else {
            panic!("first value must be a pack");
        };
        let RuntimeValue::Pack(second) = &values[1] else {
            panic!("second value must be a pack");
        };
        assert_eq!(first.get("값").map(value_to_string).as_deref(), Some("5"));
        assert_eq!(second.get("값").map(value_to_string).as_deref(), Some("5"));
    }

    #[test]
    fn connect_endpoint_explicit_solve_flow_value_returns_opposite_endpoint_value() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_explicit_solve_flow.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전류", 5)]);
        let remapped = eval_endpoint_explicit_solve(&[relation, values]).expect("explicit solve");
        let RuntimeValue::Pack(remap) = remapped else {
            panic!("원복 must be a pack");
        };
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        let RuntimeValue::Pack(left) = &values[0] else {
            panic!("left value must be a pack");
        };
        let RuntimeValue::Pack(right) = &values[1] else {
            panic!("right value must be a pack");
        };
        assert_eq!(left.get("값").map(value_to_string).as_deref(), Some("5"));
        assert_eq!(right.get("값").map(value_to_string).as_deref(), Some("-5"));
    }

    #[test]
    fn connect_endpoint_explicit_solve_flat_set_skips_public_normalize_step() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    정규 <- (이음관계) 이음관계.정규화.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_explicit_solve_flat.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "정규"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let flat = output.resources.get("정규").expect("정규").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let remapped = eval_endpoint_explicit_solve(&[flat, values]).expect("explicit solve");
        let RuntimeValue::Pack(remap) = remapped else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn connect_endpoint_explicit_solve_rejects_unsupported_and_boundary_errors() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 은행.대출창구와 기업1.차입끝을 (대출금은 흐르게, 위험이 대출금에 실리게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_explicit_solve_unsupported.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        assert!(eval_endpoint_explicit_solve(&[relation, RuntimeValue::List(vec![])]).is_err());

        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_explicit_solve_duplicate.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let duplicate =
            endpoint_boundary_test_values(&[("전지.양극.전압", 5), ("전지.양극.전압", 6)]);
        let err =
            eval_endpoint_explicit_solve(&[relation, duplicate]).expect_err("duplicate must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_value_duplicate_path"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_injection_records_unit_metadata() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_unit_boundary_injection.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();
        let value_list = endpoint_boundary_test_values_unit(&[("전지.양극.전압", 5, UnitDim::KRW)]);
        let injected_relation_list =
            eval_endpoint_boundary_value_relation_list(&[formula_set.clone(), value_list.clone()])
                .expect("value relation list");
        let RuntimeValue::List(injected_relations) = injected_relation_list else {
            panic!("값관계들 must be a list");
        };
        let RuntimeValue::Pack(relation) = &injected_relations[0] else {
            panic!("injected relation must be a pack");
        };
        let Some(RuntimeValue::Formula(right)) = relation.get("오른쪽") else {
            panic!("injected right must be formula");
        };
        assert_eq!(right.raw, "5");

        let injected_set =
            eval_endpoint_boundary_value_injection(&[formula_set, value_list]).expect("inject");
        let RuntimeValue::Pack(set) = injected_set else {
            panic!("주입묶음 must be a pack");
        };
        let Some(RuntimeValue::List(injected)) = set.get("주입값들") else {
            panic!("주입값들 must be a list");
        };
        let RuntimeValue::Pack(first) = &injected[0] else {
            panic!("injected value must be a pack");
        };
        assert_eq!(
            first.get("값").map(value_to_string).as_deref(),
            Some("5@KRW")
        );
        assert_eq!(
            first.get("단위차원"),
            Some(&RuntimeValue::String("KRW".to_string()))
        );
        assert_eq!(
            first.get("단위기호"),
            Some(&RuntimeValue::String("KRW".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_solve_remap_restores_unit_values() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let program = DdnProgram::from_source(script, "connect_endpoint_unit_boundary_solve.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values_unit(&[("전지.양극.전류", 5, UnitDim::KRW)]);
        let remapped = eval_endpoint_explicit_solve(&[relation, values]).expect("explicit solve");
        let RuntimeValue::Pack(remap) = remapped else {
            panic!("원복 must be a pack");
        };
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        let RuntimeValue::Pack(left) = &values[0] else {
            panic!("left value must be a pack");
        };
        let RuntimeValue::Pack(right) = &values[1] else {
            panic!("right value must be a pack");
        };
        assert_eq!(
            left.get("값").map(value_to_string).as_deref(),
            Some("5@KRW")
        );
        assert_eq!(
            right.get("값").map(value_to_string).as_deref(),
            Some("-5@KRW")
        );
    }

    #[test]
    fn connect_endpoint_unit_boundary_rejects_dim_and_currency_conflicts() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_unit_boundary_conflict.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let formula_set = output
            .resources
            .get("방정식묶음")
            .expect("방정식묶음")
            .clone();

        let dim_values = endpoint_boundary_test_values_unit(&[
            ("전지.양극.전압", 1, UnitDim::LENGTH),
            ("전구.왼핀.전압", 1, UnitDim::FORCE),
        ]);
        let injected = eval_endpoint_boundary_value_injection(&[formula_set.clone(), dim_values])
            .expect("inject dim conflict");
        let err = eval_endpoint_solve_result_remap(&[injected, endpoint_fake_solve_result()])
            .expect_err("dimension conflict must fail");
        assert!(
            err.to_string()
                .contains("connect_unit_boundary_dim_conflict"),
            "{err}"
        );

        let currency_values = endpoint_boundary_test_values_unit(&[
            ("전지.양극.전압", 1, UnitDim::KRW),
            ("전구.왼핀.전압", 1, UnitDim::USD),
        ]);
        let injected = eval_endpoint_boundary_value_injection(&[formula_set, currency_values])
            .expect("inject currency conflict");
        let err = eval_endpoint_solve_result_remap(&[injected, endpoint_fake_solve_result()])
            .expect_err("currency conflict must fail");
        assert!(
            err.to_string()
                .contains("connect_unit_boundary_incompatible_unit"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_check_pass_and_violation() {
        let remap = endpoint_range_solved_result("전압", 5, None);
        let pass = endpoint_range_test_values_raw(vec![(
            "전지.양극.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let checked =
            eval_endpoint_boundary_range_check(&[remap.clone(), pass]).expect("range check");
        let RuntimeValue::Pack(check) = checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_range_check".to_string()))
        );
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("통과".to_string()))
        );
        assert_eq!(check.get("위반개수"), Some(&fixed_value(0)));

        let fail = endpoint_range_test_values_raw(vec![(
            "전지.양극.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(4))),
        )]);
        let checked = eval_endpoint_boundary_range_check(&[remap, fail]).expect("range check");
        let RuntimeValue::Pack(check) = checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::List(violations)) = check.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("above_max".to_string()))
        );
        assert_eq!(first.get("값").map(value_to_string).as_deref(), Some("5"));
    }

    #[test]
    fn connect_endpoint_explicit_solve_range_pass_and_fail() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_explicit_solve_range_pass.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let checked =
            eval_endpoint_explicit_solve_range_check(&[relation.clone(), values.clone(), ranges])
                .expect("explicit solve range");
        let RuntimeValue::Pack(check) = checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_check".to_string()
            ))
        );
        assert_eq!(
            check.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("통과".to_string()))
        );
        assert_eq!(check.get("위반개수"), Some(&fixed_value(0)));
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(4))),
        )]);
        let checked =
            eval_endpoint_explicit_solve_range_check(&[relation, values, ranges]).expect("fail");
        let RuntimeValue::Pack(check) = checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::Pack(range_check)) = check.get("범위검사") else {
            panic!("범위검사 must be a pack");
        };
        let Some(RuntimeValue::List(violations)) = range_check.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("above_max".to_string()))
        );
    }

    #[test]
    fn connect_endpoint_explicit_solve_range_missing_and_error_propagation() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_explicit_solve_range_missing.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let checked = eval_endpoint_explicit_solve_range_check(&[
            relation.clone(),
            RuntimeValue::List(vec![]),
            ranges,
        ])
        .expect("missing value check");
        let RuntimeValue::Pack(check) = checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::Pack(range_check)) = check.get("범위검사") else {
            panic!("범위검사 must be a pack");
        };
        let Some(RuntimeValue::List(violations)) = range_check.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("missing_value".to_string()))
        );
        assert!(!first.contains_key("값"));

        let duplicate_range = endpoint_range_test_values_raw(vec![
            (
                "전구.왼핀.전압",
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
                None,
            ),
            (
                "전구.왼핀.전압",
                None,
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
            ),
        ]);
        let err = eval_endpoint_explicit_solve_range_check(&[
            relation,
            endpoint_boundary_test_values(&[("전지.양극.전압", 5)]),
            duplicate_range,
        ])
        .expect_err("duplicate range path must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_duplicate_path"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_report_pass_rows_are_endpoint_ordered() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_range_report_pass.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let report =
            eval_endpoint_solve_range_report(&[relation.clone(), values.clone(), ranges.clone()])
                .expect("report");
        let rows =
            eval_endpoint_solve_range_report_rows(&[relation, values, ranges]).expect("rows");
        let RuntimeValue::Pack(report) = report else {
            panic!("report must be a pack");
        };
        assert_eq!(
            report.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_report".to_string()
            ))
        );
        assert_eq!(
            report.get("검사결과"),
            Some(&RuntimeValue::String("통과".to_string()))
        );
        assert_eq!(report.get("행개수"), Some(&fixed_value(2)));
        assert_eq!(report.get("값개수"), Some(&fixed_value(2)));
        assert_eq!(report.get("누락개수"), Some(&fixed_value(0)));
        let Some(RuntimeValue::List(report_rows)) = report.get("행들") else {
            panic!("행들 must be a list");
        };
        let RuntimeValue::List(row_list) = rows else {
            panic!("rows must be a list");
        };
        assert_eq!(report_rows.len(), 2);
        assert_eq!(row_list.len(), 2);
        let RuntimeValue::Pack(left) = &report_rows[0] else {
            panic!("left row must be a pack");
        };
        let RuntimeValue::Pack(right) = &report_rows[1] else {
            panic!("right row must be a pack");
        };
        assert_eq!(
            left.get("범위상태"),
            Some(&RuntimeValue::String("범위없음".to_string()))
        );
        assert_eq!(
            right.get("범위상태"),
            Some(&RuntimeValue::String("통과".to_string()))
        );
        assert_eq!(right.get("하한").map(value_to_string).as_deref(), Some("0"));
        assert_eq!(
            right.get("상한").map(value_to_string).as_deref(),
            Some("10")
        );
    }

    #[test]
    fn connect_endpoint_solve_range_report_missing_rows_distinguish_range_presence() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_range_report_missing.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let ranges = endpoint_range_test_values_raw(vec![(
            "전지.양극.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let report =
            eval_endpoint_solve_range_report(&[relation, RuntimeValue::List(vec![]), ranges])
                .expect("report");
        let RuntimeValue::Pack(report) = report else {
            panic!("report must be a pack");
        };
        assert_eq!(
            report.get("풀이결과종류"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        assert_eq!(
            report.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        assert_eq!(report.get("누락개수"), Some(&fixed_value(2)));
        let Some(RuntimeValue::List(rows)) = report.get("행들") else {
            panic!("행들 must be a list");
        };
        let RuntimeValue::Pack(left) = &rows[0] else {
            panic!("left row must be a pack");
        };
        let RuntimeValue::Pack(right) = &rows[1] else {
            panic!("right row must be a pack");
        };
        assert_eq!(
            left.get("값상태"),
            Some(&RuntimeValue::String("누락".to_string()))
        );
        assert_eq!(
            left.get("범위상태"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::List(violations)) = left.get("위반들") else {
            panic!("violations must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("missing_value".to_string()))
        );
        assert!(!first.contains_key("값"));
        assert_eq!(
            right.get("값상태"),
            Some(&RuntimeValue::String("누락".to_string()))
        );
        assert_eq!(
            right.get("범위상태"),
            Some(&RuntimeValue::String("범위없음".to_string()))
        );
        assert_eq!(right.get("위반개수"), Some(&fixed_value(0)));
    }

    #[test]
    fn connect_endpoint_solve_range_text_report_formats_tsv_rows() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_range_text_report.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let relation = output.resources.get("이음관계").expect("이음관계").clone();
        let values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let report =
            eval_endpoint_solve_range_report(&[relation.clone(), values.clone(), ranges.clone()])
                .expect("report");
        let text = eval_endpoint_solve_range_text_report(&[report]).expect("text");
        let direct = eval_endpoint_explicit_solve_range_text_report(&[relation, values, ranges])
            .expect("direct text");
        let RuntimeValue::String(text) = text else {
            panic!("text report must be a string");
        };
        let RuntimeValue::String(direct) = direct else {
            panic!("direct text report must be a string");
        };
        assert_eq!(text, direct);
        assert!(text.starts_with("변수\t경로\t값상태\t값\t범위상태\t하한\t상한\t위반"));
        assert!(text.contains("ep_001\t전지.양극.전압\t값있음\t5\t범위없음\t\t\t"));
        assert!(text.contains("ep_002\t전구.왼핀.전압\t값있음\t5\t통과\t0\t10\t"));
        assert!(!text.ends_with('\n'));
    }

    #[test]
    fn connect_endpoint_solve_range_text_report_rejects_non_report_input() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err = eval_endpoint_solve_range_text_report(&[RuntimeValue::Pack(fields)])
            .expect_err("non report must fail");
        assert!(
            err.to_string()
                .contains("connect_report_text_expected_solve_range_report"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_records_expectation_matrix() {
        let pass_script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let pass_program =
            DdnProgram::from_source(pass_script, "connect_endpoint_case_suite_pass.ddn")
                .expect("parse");
        let mut pass_runner = DdnRunner::new(pass_program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let pass_output = pass_runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let pass_relation = pass_output
            .resources
            .get("이음관계")
            .expect("이음관계")
            .clone();

        let fail_script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let fail_program =
            DdnProgram::from_source(fail_script, "connect_endpoint_case_suite_fail.ddn")
                .expect("parse");
        let mut fail_runner = DdnRunner::new(fail_program, "매틱");
        let fail_output = fail_runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let fail_relation = fail_output
            .resources
            .get("이음관계")
            .expect("이음관계")
            .clone();

        let pass_values = endpoint_boundary_test_values(&[("전지.양극.전압", 5)]);
        let pass_ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let fail_values = endpoint_boundary_test_values(&[("전지.양극.전류", 5)]);
        let fail_ranges = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전류",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);

        fn case(
            name: &str,
            relation: RuntimeValue,
            values: RuntimeValue,
            ranges: RuntimeValue,
            expected: Option<&str>,
        ) -> RuntimeValue {
            let mut fields = BTreeMap::new();
            fields.insert("이름".to_string(), RuntimeValue::String(name.to_string()));
            fields.insert("이음관계".to_string(), relation);
            fields.insert("값들".to_string(), values);
            fields.insert("범위들".to_string(), ranges);
            if let Some(expected) = expected {
                fields.insert(
                    "기대검사결과".to_string(),
                    RuntimeValue::String(expected.to_string()),
                );
            }
            RuntimeValue::Pack(fields)
        }

        let suite = eval_endpoint_solve_range_case_suite(&[RuntimeValue::List(vec![
            case(
                "pass-default",
                pass_relation.clone(),
                pass_values.clone(),
                pass_ranges.clone(),
                None,
            ),
            case(
                "expected-fail",
                fail_relation.clone(),
                fail_values.clone(),
                fail_ranges.clone(),
                Some("실패"),
            ),
            case(
                "unexpected-fail",
                fail_relation,
                fail_values,
                fail_ranges,
                None,
            ),
            case(
                "unexpected-success",
                pass_relation,
                pass_values,
                pass_ranges,
                Some("실패"),
            ),
        ])])
        .expect("suite");
        let RuntimeValue::Pack(suite_pack) = &suite else {
            panic!("suite must be a pack");
        };
        assert_eq!(
            suite_pack.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_case_suite".to_string()
            ))
        );
        assert_eq!(suite_pack.get("전체통과"), Some(&RuntimeValue::Bool(false)));
        assert_eq!(suite_pack.get("통과개수"), Some(&fixed_value(2)));
        assert_eq!(suite_pack.get("실패개수"), Some(&fixed_value(2)));
        let text = eval_endpoint_solve_range_case_suite_text(&[suite]).expect("suite text");
        let RuntimeValue::String(text) = text else {
            panic!("suite text must be a string");
        };
        assert!(text.starts_with("이름\t기대\t실제\t통과"));
        assert!(text.contains("pass-default\t통과\t통과\t참"));
        assert!(text.contains("expected-fail\t실패\t실패\t참"));
        assert!(text.contains("unexpected-fail\t통과\t실패\t거짓"));
        assert!(text.contains("unexpected-success\t실패\t통과\t거짓"));
        assert!(!text.ends_with('\n'));
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_rejects_bad_inputs() {
        let mut bad_expected = BTreeMap::new();
        bad_expected.insert("이름".to_string(), RuntimeValue::String("bad".to_string()));
        bad_expected.insert("이음관계".to_string(), RuntimeValue::None);
        bad_expected.insert("값들".to_string(), RuntimeValue::List(Vec::new()));
        bad_expected.insert("범위들".to_string(), RuntimeValue::List(Vec::new()));
        bad_expected.insert(
            "기대검사결과".to_string(),
            RuntimeValue::String("모름".to_string()),
        );
        let err = eval_endpoint_solve_range_case(&[RuntimeValue::Pack(bad_expected)])
            .expect_err("bad expected result must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_invalid_expected_result"),
            "{err}"
        );

        let mut not_suite = BTreeMap::new();
        not_suite.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err = eval_endpoint_solve_range_case_suite_text(&[RuntimeValue::Pack(not_suite)])
            .expect_err("non suite must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_text_expected_suite"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_detail_formats_sections() {
        let pass_script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게) 잇기.
}"#;
        let pass_program =
            DdnProgram::from_source(pass_script, "connect_endpoint_case_suite_detail_pass.ddn")
                .expect("parse");
        let mut pass_runner = DdnRunner::new(pass_program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("이음관계".to_string(), RuntimeValue::None);
        let pass_output = pass_runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let pass_relation = pass_output
            .resources
            .get("이음관계")
            .expect("이음관계")
            .clone();

        let flow_script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전류는 흐르게) 잇기.
}"#;
        let flow_program =
            DdnProgram::from_source(flow_script, "connect_endpoint_case_suite_detail_flow.ddn")
                .expect("parse");
        let mut flow_runner = DdnRunner::new(flow_program, "매틱");
        let flow_output = flow_runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let flow_relation = flow_output
            .resources
            .get("이음관계")
            .expect("이음관계")
            .clone();

        fn case(
            name: &str,
            relation: RuntimeValue,
            values: RuntimeValue,
            ranges: RuntimeValue,
        ) -> RuntimeValue {
            let mut fields = BTreeMap::new();
            fields.insert("이름".to_string(), RuntimeValue::String(name.to_string()));
            fields.insert("이음관계".to_string(), relation);
            fields.insert("값들".to_string(), values);
            fields.insert("범위들".to_string(), ranges);
            RuntimeValue::Pack(fields)
        }

        let cases = RuntimeValue::List(vec![
            case(
                "voltage-pass",
                pass_relation,
                endpoint_boundary_test_values(&[("전지.양극.전압", 5)]),
                endpoint_range_test_values_raw(vec![(
                    "전구.왼핀.전압",
                    Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
                    Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
                )]),
            ),
            case(
                "flow-pass",
                flow_relation,
                endpoint_boundary_test_values(&[("전지.양극.전류", 5)]),
                endpoint_range_test_values_raw(vec![(
                    "전구.왼핀.전류",
                    Some(RuntimeValue::Fixed64(Fixed64::from_i64(-10))),
                    Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
                )]),
            ),
        ]);
        let suite = eval_endpoint_solve_range_case_suite(&[cases.clone()]).expect("suite");
        let detail =
            eval_endpoint_solve_range_case_suite_detail_text(&[suite]).expect("detail text");
        let direct =
            eval_endpoint_solve_range_case_suite_run_detail_text(&[cases]).expect("direct detail");
        let RuntimeValue::String(detail) = detail else {
            panic!("detail must be a string");
        };
        let RuntimeValue::String(direct) = direct else {
            panic!("direct detail must be a string");
        };
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
        let mut not_suite = BTreeMap::new();
        not_suite.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err =
            eval_endpoint_solve_range_case_suite_detail_text(&[RuntimeValue::Pack(not_suite)])
                .expect_err("non suite must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_detail_expected_suite"),
            "{err}"
        );

        let mut bad_result = BTreeMap::new();
        bad_result.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_result".to_string()),
        );
        bad_result.insert("이름".to_string(), RuntimeValue::String("bad".to_string()));
        bad_result.insert(
            "기대검사결과".to_string(),
            RuntimeValue::String("통과".to_string()),
        );
        bad_result.insert(
            "실제검사결과".to_string(),
            RuntimeValue::String("통과".to_string()),
        );
        bad_result.insert("통과여부".to_string(), RuntimeValue::Bool(true));
        let mut suite = BTreeMap::new();
        suite.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_suite".to_string()),
        );
        suite.insert(
            "결과들".to_string(),
            RuntimeValue::List(vec![RuntimeValue::Pack(bad_result)]),
        );
        let err = eval_endpoint_solve_range_case_suite_detail_text(&[RuntimeValue::Pack(suite)])
            .expect_err("malformed case result must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_detail_malformed_case_result"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_summary_records_mismatch_lists() {
        fn case_result(name: &str, expected: &str, actual: &str, passed: bool) -> RuntimeValue {
            let mut fields = BTreeMap::new();
            fields.insert(
                "__이음관계종류".to_string(),
                RuntimeValue::String("endpoint_solve_range_case_result".to_string()),
            );
            fields.insert("이름".to_string(), RuntimeValue::String(name.to_string()));
            fields.insert(
                "기대검사결과".to_string(),
                RuntimeValue::String(expected.to_string()),
            );
            fields.insert(
                "실제검사결과".to_string(),
                RuntimeValue::String(actual.to_string()),
            );
            fields.insert("통과여부".to_string(), RuntimeValue::Bool(passed));
            RuntimeValue::Pack(fields)
        }

        let suite = endpoint_solve_range_case_suite_pack(
            vec![
                case_result("expected-pass-actual-pass", "통과", "통과", true),
                case_result("expected-fail-actual-fail", "실패", "실패", true),
                case_result("expected-pass-actual-fail", "통과", "실패", false),
                case_result("expected-fail-actual-pass", "실패", "통과", false),
            ],
            2,
        );
        let summary =
            eval_endpoint_solve_range_case_suite_summary(&[suite]).expect("suite summary");
        let RuntimeValue::Pack(summary_pack) = summary else {
            panic!("summary must be a pack");
        };
        assert_eq!(
            summary_pack.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_case_suite_summary".to_string()
            ))
        );
        assert_eq!(summary_pack.get("개수"), Some(&fixed_value(4)));
        assert_eq!(summary_pack.get("통과개수"), Some(&fixed_value(2)));
        assert_eq!(summary_pack.get("실패개수"), Some(&fixed_value(2)));
        assert_eq!(
            summary_pack.get("전체통과"),
            Some(&RuntimeValue::Bool(false))
        );
        assert_eq!(
            summary_pack.get("통과케이스들"),
            Some(&RuntimeValue::List(vec![
                RuntimeValue::String("expected-pass-actual-pass".to_string()),
                RuntimeValue::String("expected-fail-actual-fail".to_string()),
            ]))
        );
        assert_eq!(
            summary_pack.get("실패케이스들"),
            Some(&RuntimeValue::List(vec![
                RuntimeValue::String("expected-pass-actual-fail".to_string()),
                RuntimeValue::String("expected-fail-actual-pass".to_string()),
            ]))
        );
        assert_eq!(
            summary_pack.get("기대실패통과케이스들"),
            Some(&RuntimeValue::List(vec![RuntimeValue::String(
                "expected-fail-actual-pass".to_string()
            )]))
        );
        assert_eq!(
            summary_pack.get("기대통과실패케이스들"),
            Some(&RuntimeValue::List(vec![RuntimeValue::String(
                "expected-pass-actual-fail".to_string()
            )]))
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_summary_rejects_bad_inputs() {
        let mut not_suite = BTreeMap::new();
        not_suite.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err = eval_endpoint_solve_range_case_suite_summary(&[RuntimeValue::Pack(not_suite)])
            .expect_err("non suite must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_summary_expected_suite"),
            "{err}"
        );

        let mut bad_result = BTreeMap::new();
        bad_result.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_result".to_string()),
        );
        bad_result.insert("이름".to_string(), RuntimeValue::String("bad".to_string()));
        bad_result.insert(
            "기대검사결과".to_string(),
            RuntimeValue::String("통과".to_string()),
        );
        bad_result.insert(
            "실제검사결과".to_string(),
            RuntimeValue::String("통과".to_string()),
        );
        let mut suite = BTreeMap::new();
        suite.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_suite".to_string()),
        );
        suite.insert(
            "결과들".to_string(),
            RuntimeValue::List(vec![RuntimeValue::Pack(bad_result)]),
        );
        let err = eval_endpoint_solve_range_case_suite_summary(&[RuntimeValue::Pack(suite)])
            .expect_err("malformed result must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_summary_malformed_case_result"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_check_records_pass_fail() {
        let mut summary = BTreeMap::new();
        summary.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_suite_summary".to_string()),
        );
        summary.insert("개수".to_string(), fixed_value(4));
        summary.insert("통과개수".to_string(), fixed_value(2));
        summary.insert("실패개수".to_string(), fixed_value(2));
        summary.insert("전체통과".to_string(), RuntimeValue::Bool(false));
        summary.insert(
            "실패케이스들".to_string(),
            RuntimeValue::List(vec![
                RuntimeValue::String("expected-pass-actual-fail".to_string()),
                RuntimeValue::String("expected-fail-actual-pass".to_string()),
            ]),
        );
        summary.insert(
            "기대실패통과케이스들".to_string(),
            RuntimeValue::List(vec![RuntimeValue::String(
                "expected-fail-actual-pass".to_string(),
            )]),
        );
        summary.insert(
            "기대통과실패케이스들".to_string(),
            RuntimeValue::List(vec![RuntimeValue::String(
                "expected-pass-actual-fail".to_string(),
            )]),
        );

        let check = eval_endpoint_solve_range_case_suite_check(&[RuntimeValue::Pack(summary)])
            .expect("suite check");
        let RuntimeValue::Pack(check_pack) = check else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check_pack.get("__이음관계종류"),
            Some(&RuntimeValue::String(
                "endpoint_solve_range_case_suite_check".to_string()
            ))
        );
        assert_eq!(
            check_pack.get("판정"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        assert_eq!(check_pack.get("개수"), Some(&fixed_value(4)));
        assert_eq!(check_pack.get("통과개수"), Some(&fixed_value(2)));
        assert_eq!(check_pack.get("실패개수"), Some(&fixed_value(2)));
        assert_eq!(check_pack.get("전체통과"), Some(&RuntimeValue::Bool(false)));
        assert_eq!(
            check_pack.get("기대실패통과케이스들"),
            Some(&RuntimeValue::List(vec![RuntimeValue::String(
                "expected-fail-actual-pass".to_string()
            )]))
        );
        assert_eq!(
            check_pack.get("기대통과실패케이스들"),
            Some(&RuntimeValue::List(vec![RuntimeValue::String(
                "expected-pass-actual-fail".to_string()
            )]))
        );
    }

    #[test]
    fn connect_endpoint_solve_range_case_suite_check_rejects_bad_inputs() {
        let mut not_summary = BTreeMap::new();
        not_summary.insert(
            "아무것".to_string(),
            RuntimeValue::Fixed64(Fixed64::from_i64(1)),
        );
        let err =
            eval_endpoint_solve_range_case_suite_check(&[RuntimeValue::Pack(not_summary)])
                .expect_err("non summary must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_check_expected_summary"),
            "{err}"
        );

        let mut malformed = BTreeMap::new();
        malformed.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_range_case_suite_summary".to_string()),
        );
        malformed.insert("전체통과".to_string(), RuntimeValue::Bool(true));
        let err = eval_endpoint_solve_range_case_suite_check(&[RuntimeValue::Pack(malformed)])
            .expect_err("malformed summary must fail");
        assert!(
            err.to_string()
                .contains("connect_case_suite_check_malformed_summary"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_missing_value_is_soft_violation() {
        let remap = endpoint_partial_range_solve_result();
        let range = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
        )]);
        let checked = eval_endpoint_boundary_range_check(&[remap, range]).expect("range check");
        let RuntimeValue::Pack(check) = checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("실패".to_string()))
        );
        let Some(RuntimeValue::List(violations)) = check.get("위반들") else {
            panic!("위반들 must be a list");
        };
        let RuntimeValue::Pack(first) = &violations[0] else {
            panic!("violation must be a pack");
        };
        assert_eq!(
            first.get("이유"),
            Some(&RuntimeValue::String("missing_value".to_string()))
        );
        assert!(!first.contains_key("값"));
        assert_eq!(first.get("하한").map(value_to_string).as_deref(), Some("0"));
        assert_eq!(
            first.get("상한").map(value_to_string).as_deref(),
            Some("10")
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_unit_policy_and_errors() {
        let remap = endpoint_range_solved_result("전류", 5, Some(UnitDim::KRW));
        let pass = endpoint_range_test_values_raw(vec![(
            "전구.왼핀.전류",
            Some(RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(-10),
                dim: UnitDim::KRW,
            })),
            Some(RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(0),
                dim: UnitDim::KRW,
            })),
        )]);
        let checked =
            eval_endpoint_boundary_range_check(&[remap.clone(), pass]).expect("range check");
        let RuntimeValue::Pack(check) = checked else {
            panic!("check must be a pack");
        };
        assert_eq!(
            check.get("검사결과"),
            Some(&RuntimeValue::String("통과".to_string()))
        );

        let incompatible = endpoint_range_test_values_raw(vec![(
            "전지.양극.전류",
            None,
            Some(RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(10),
                dim: UnitDim::USD,
            })),
        )]);
        let err = eval_endpoint_boundary_range_check(&[remap.clone(), incompatible])
            .expect_err("incompatible unit must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_incompatible_unit"),
            "{err}"
        );

        let dim_conflict = endpoint_range_test_values_raw(vec![(
            "전지.양극.전류",
            None,
            Some(RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(10),
                dim: UnitDim::LENGTH,
            })),
        )]);
        let err = eval_endpoint_boundary_range_check(&[remap, dim_conflict])
            .expect_err("dimension conflict must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_dim_conflict"),
            "{err}"
        );
    }

    #[test]
    fn connect_endpoint_boundary_range_rejects_unsupported_inputs() {
        let remap = endpoint_range_solved_result("전압", 5, None);
        let duplicate = endpoint_range_test_values_raw(vec![
            (
                "전지.양극.전압",
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
                None,
            ),
            (
                "전지.양극.전압",
                None,
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(10))),
            ),
        ]);
        let err = eval_endpoint_boundary_range_check(&[remap.clone(), duplicate])
            .expect_err("duplicate path must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_duplicate_path"),
            "{err}"
        );

        let unknown = endpoint_range_test_values_raw(vec![(
            "없는.끝.전압",
            Some(RuntimeValue::Fixed64(Fixed64::from_i64(0))),
            None,
        )]);
        let err = eval_endpoint_boundary_range_check(&[remap.clone(), unknown])
            .expect_err("unknown path must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_unknown_path"),
            "{err}"
        );

        let mut bad = BTreeMap::new();
        bad.insert(
            "경로".to_string(),
            RuntimeValue::String("전지.양극.전압".to_string()),
        );
        bad.insert("최소".to_string(), RuntimeValue::String("0".to_string()));
        let err = eval_endpoint_boundary_range_check(&[
            remap,
            RuntimeValue::List(vec![RuntimeValue::Pack(bad)]),
        ])
        .expect_err("non numeric must fail");
        assert!(
            err.to_string()
                .contains("connect_boundary_range_non_numeric"),
            "{err}"
        );
    }

    fn endpoint_boundary_test_values(items: &[(&str, i64)]) -> RuntimeValue {
        endpoint_boundary_test_values_raw(
            items
                .iter()
                .map(|(path, value)| (*path, RuntimeValue::Fixed64(Fixed64::from_i64(*value))))
                .collect(),
        )
    }

    fn endpoint_range_solved_result(
        channel: &str,
        left_value: i64,
        unit: Option<UnitDim>,
    ) -> RuntimeValue {
        let left_path = format!("전지.양극.{channel}");
        let right_path = format!("전구.왼핀.{channel}");
        let left = match unit {
            Some(dim) => RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(left_value),
                dim,
            }),
            None => RuntimeValue::Fixed64(Fixed64::from_i64(left_value)),
        };
        let right = match unit {
            Some(dim) => RuntimeValue::Unit(UnitValue {
                value: Fixed64::from_i64(-left_value),
                dim,
            }),
            None => RuntimeValue::Fixed64(Fixed64::from_i64(left_value)),
        };
        endpoint_range_solve_result_from_values(vec![
            ("ep_001", &left_path, Some(left)),
            ("ep_002", &right_path, Some(right)),
        ])
    }

    fn endpoint_partial_range_solve_result() -> RuntimeValue {
        endpoint_range_solve_result_from_values(vec![
            (
                "ep_001",
                "전지.양극.전압",
                Some(RuntimeValue::Fixed64(Fixed64::from_i64(5))),
            ),
            ("ep_002", "전구.왼핀.전압", None),
        ])
    }

    fn endpoint_range_solve_result_from_values(
        entries: Vec<(&str, &str, Option<RuntimeValue>)>,
    ) -> RuntimeValue {
        let mut mappings = Vec::new();
        let mut values = Vec::new();
        let mut missing = Vec::new();
        for (variable, path, value) in entries {
            let mut mapping = BTreeMap::new();
            mapping.insert(
                "변수".to_string(),
                RuntimeValue::String(variable.to_string()),
            );
            mapping.insert("경로".to_string(), RuntimeValue::String(path.to_string()));
            mappings.push(RuntimeValue::Pack(mapping));
            if let Some(value) = value {
                let mut item = BTreeMap::new();
                item.insert(
                    "변수".to_string(),
                    RuntimeValue::String(variable.to_string()),
                );
                item.insert("경로".to_string(), RuntimeValue::String(path.to_string()));
                item.insert("값".to_string(), value);
                values.push(RuntimeValue::Pack(item));
            } else {
                missing.push(RuntimeValue::String(variable.to_string()));
            }
        }
        let mut fields = BTreeMap::new();
        fields.insert(
            "__이음관계종류".to_string(),
            RuntimeValue::String("endpoint_solve_result".to_string()),
        );
        fields.insert(
            "풀이결과종류".to_string(),
            RuntimeValue::String(if missing.is_empty() {
                "성공".to_string()
            } else {
                "부분성공".to_string()
            }),
        );
        fields.insert("값들".to_string(), RuntimeValue::List(values));
        fields.insert("누락변수들".to_string(), RuntimeValue::List(missing));
        fields.insert("변수사상".to_string(), RuntimeValue::List(mappings));
        fields.insert("원래풀이".to_string(), endpoint_fake_solve_result());
        RuntimeValue::Pack(fields)
    }

    fn endpoint_range_test_values_raw(
        items: Vec<(&str, Option<RuntimeValue>, Option<RuntimeValue>)>,
    ) -> RuntimeValue {
        RuntimeValue::List(
            items
                .into_iter()
                .map(|(path, min, max)| {
                    let mut fields = BTreeMap::new();
                    fields.insert("경로".to_string(), RuntimeValue::String(path.to_string()));
                    if let Some(min) = min {
                        fields.insert("최소".to_string(), min);
                    }
                    if let Some(max) = max {
                        fields.insert("최대".to_string(), max);
                    }
                    RuntimeValue::Pack(fields)
                })
                .collect(),
        )
    }

    fn endpoint_fake_solve_result() -> RuntimeValue {
        let mut bindings = BTreeMap::new();
        bindings.insert("ep_001".to_string(), fixed_value(1));
        bindings.insert("ep_002".to_string(), fixed_value(1));
        let mut result = BTreeMap::new();
        result.insert(
            RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
            RuntimeValue::String(RELATION_SOLVE_RESULT_SUCCESS.to_string()),
        );
        result.insert(
            RELATION_SOLVE_BINDINGS_FIELD.to_string(),
            RuntimeValue::Pack(bindings),
        );
        RuntimeValue::Pack(result)
    }

    fn endpoint_boundary_test_values_unit(items: &[(&str, i64, UnitDim)]) -> RuntimeValue {
        endpoint_boundary_test_values_raw(
            items
                .iter()
                .map(|(path, value, dim)| {
                    (
                        *path,
                        RuntimeValue::Unit(UnitValue {
                            value: Fixed64::from_i64(*value),
                            dim: *dim,
                        }),
                    )
                })
                .collect(),
        )
    }

    fn endpoint_boundary_test_values_raw(items: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
        RuntimeValue::List(
            items
                .into_iter()
                .map(|(path, value)| {
                    let mut fields = BTreeMap::new();
                    fields.insert("경로".to_string(), RuntimeValue::String(path.to_string()));
                    fields.insert("값".to_string(), value);
                    RuntimeValue::Pack(fields)
                })
                .collect(),
        )
    }

    #[test]
    fn connect_endpoint_solve_result_remap_success_returns_endpoint_values() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
    관계들 <- 방정식묶음.관계들.
    풀이 <- (관계들) 방정식풀기.
    원복 <- (방정식묶음, 풀이) 이음관계.풀이원복.
    값들 <- (방정식묶음, 풀이) 이음관계.풀이값목록.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_result_remap_success.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음", "관계들", "풀이", "원복", "값들"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(remap)) = output.resources.get("원복") else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("__이음관계종류"),
            Some(&RuntimeValue::String("endpoint_solve_result".to_string()))
        );
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 2);
        let RuntimeValue::Pack(first) = &values[0] else {
            panic!("first value must be a pack");
        };
        assert_eq!(
            first.get("변수"),
            Some(&RuntimeValue::String("ep_001".to_string()))
        );
        assert_eq!(
            first.get("경로"),
            Some(&RuntimeValue::String("전지.양극.전압".to_string()))
        );
        let Some(RuntimeValue::List(value_list)) = output.resources.get("값들") else {
            panic!("풀이값목록 must be a list");
        };
        assert_eq!(value_list.len(), 2);
    }

    #[test]
    fn connect_endpoint_solve_result_remap_partial_records_missing_variables() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
    부분관계 <- ((#ascii) 수식{ep_001}) =:= ((#ascii) 수식{0}).
    부분풀이 <- (부분관계) 방정식풀기.
    원복 <- (방정식묶음, 부분풀이) 이음관계.풀이원복.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_result_remap_partial.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음", "부분관계", "부분풀이", "원복"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(remap)) = output.resources.get("원복") else {
            panic!("원복 must be a pack");
        };
        assert_eq!(
            remap.get("풀이결과종류"),
            Some(&RuntimeValue::String("부분성공".to_string()))
        );
        let Some(RuntimeValue::List(values)) = remap.get("값들") else {
            panic!("값들 must be a list");
        };
        assert_eq!(values.len(), 1);
        let Some(RuntimeValue::List(missing)) = remap.get("누락변수들") else {
            panic!("누락변수들 must be a list");
        };
        assert_eq!(missing, &vec![RuntimeValue::String("ep_002".to_string())]);
    }

    #[test]
    fn connect_endpoint_solve_result_remap_rejects_unknown_solver_binding() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 전압은 흐르게) 잇기.
    방정식묶음 <- (이음관계) 이음관계.방정식화.
    나쁜관계 <- ((#ascii) 수식{ep_999}) =:= ((#ascii) 수식{0}).
    나쁜풀이 <- (나쁜관계) 방정식풀기.
    원복 <- (방정식묶음, 나쁜풀이) 이음관계.풀이원복.
}"#;
        let program =
            DdnProgram::from_source(script, "connect_endpoint_solve_result_remap_bad.ddn")
                .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        for name in ["이음관계", "방정식묶음", "나쁜관계", "나쁜풀이", "원복"] {
            defaults.insert(name.to_string(), RuntimeValue::None);
        }
        assert!(runner
            .run_update(&world, &empty_input(), &defaults)
            .is_err());
    }

    #[test]
    fn connect_endpoint_rejects_unsupported_multi_inner_sentence_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, 재화가 돈에 실리게) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_multi_unsupported.ddn").is_err());
    }

    #[test]
    fn connect_endpoint_rejects_duplicate_carrier_flow_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 가계1.구매끝과 장터.소매끝을 (돈은 흐르게, 돈은 거슬러 흐르게, 재화가 돈에 실리게) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_duplicate_carrier.ddn").is_err());
    }

    #[test]
    fn connect_endpoint_rejects_empty_multi_inner_sentence_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전지.양극과 전구.왼핀을 (전압은 같게, ) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_multi_empty.ddn").is_err());
    }

    #[test]
    fn connect_endpoint_rejects_whole_object_shorthand_runtime_surface() {
        let script = r#"매틱:움직씨 = {
    이음관계 <- 전구와 전지를 (전압은 같게) 잇기.
}"#;
        assert!(DdnProgram::from_source(script, "connect_endpoint_unsupported.ddn").is_err());
    }

    #[test]
    fn relation_solve_surface_handles_quadratic_single_root() {
        let script = r#"
매틱:움직씨 = {
    rel <- ((#ascii) 수식{x^2 - 4*x + 4}) =:= ((#ascii) 수식{0}).
    결과 <- (rel) 방정식풀기.
}
"#;
        let program =
            DdnProgram::from_source(script, "relation_solve_quadratic.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("rel".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("결과") else {
            panic!("결과 must be relation solve pack");
        };
        assert_eq!(
            fields.get(RELATION_SOLVE_RESULT_KIND_FIELD),
            Some(&RuntimeValue::String(
                RELATION_SOLVE_RESULT_SUCCESS.to_string()
            ))
        );
        assert_eq!(
            fields.get(RELATION_SOLVE_VALUE_FIELD),
            Some(&make_big_int_pack_from_bigint(&BigInt::from(2)))
        );
    }

    #[test]
    fn relation_solve_surface_handles_linear_system_2x2() {
        let script = r#"
매틱:움직씨 = {
    rel1 <- ((#ascii) 수식{x + y}) =:= ((#ascii) 수식{5}).
    rel2 <- ((#ascii) 수식{x - y}) =:= ((#ascii) 수식{1}).
    rels <- (rel1, rel2) 차림.
    결과 <- (rels) 방정식풀기.
}
"#;
        let program = DdnProgram::from_source(script, "relation_solve_system.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let mut defaults: HashMap<String, RuntimeValue> = HashMap::new();
        defaults.insert("rel1".to_string(), RuntimeValue::None);
        defaults.insert("rel2".to_string(), RuntimeValue::None);
        defaults.insert("rels".to_string(), RuntimeValue::None);
        defaults.insert("결과".to_string(), RuntimeValue::None);
        let output = runner
            .run_update(&world, &empty_input(), &defaults)
            .expect("run update");
        let Some(RuntimeValue::Pack(fields)) = output.resources.get("결과") else {
            panic!("결과 must be relation solve pack");
        };
        assert_eq!(
            fields.get(RELATION_SOLVE_RESULT_KIND_FIELD),
            Some(&RuntimeValue::String(
                RELATION_SOLVE_RESULT_SUCCESS.to_string()
            ))
        );
        let Some(RuntimeValue::Pack(bindings)) = fields.get(RELATION_SOLVE_BINDINGS_FIELD) else {
            panic!("결과.해 must be pack");
        };
        assert_eq!(
            bindings.get("x"),
            Some(&make_big_int_pack_from_bigint(&BigInt::from(3)))
        );
        assert_eq!(
            bindings.get("y"),
            Some(&make_big_int_pack_from_bigint(&BigInt::from(2)))
        );
    }

    #[test]
    fn butbak_decl_reassignment_fails_in_runtime() {
        let script = r#"
채비 {
    상수:수 = 1.
}.

상수 <- 2.
"#;
        let program = DdnProgram::from_source(script, "butbak_reassign_fail.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let err = match runner.run_update(&world, &empty_input(), &HashMap::new()) {
            Ok(_) => panic!("butbak reassignment must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("붙박이는 재대입할 수 없습니다"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn non_butbak_decl_reassignment_still_runs_in_runtime() {
        let script = r#"
채비 {
    점수:수 <- 0.
}.

점수 <- 1.
"#;
        let program =
            DdnProgram::from_source(script, "non_butbak_reassign_runs.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        assert_eq!(
            extract_fixed(&output.resources, "점수"),
            Fixed64::from_i64(1)
        );
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
        let program = DdnProgram::from_source(script, "numeric_family_sized_variant_alias.ddn")
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
        let program = DdnProgram::from_source(script, "numeric_family_sized_variant_type_pin.ddn")
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
        let program = DdnProgram::from_source(script, "numeric_family_english_alias_type_pin.ddn")
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
        let program = DdnProgram::from_source(script, "numeric_family_alias_diag_fixed64.ddn")
            .expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let defaults: HashMap<String, RuntimeValue> = HashMap::new();
        let err = match runner.run_update(&world, &empty_input(), &defaults) {
            Ok(_) => panic!("must fail"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("기대=셈수"),
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
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_int64.ddn").expect("parse");
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
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_bigint.ddn").expect("parse");
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
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_ratio.ddn").expect("parse");
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
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_frac.ddn").expect("parse");
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
        let program =
            DdnProgram::from_source(script, "numeric_family_alias_diag_factor.ddn").expect("parse");
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
        let program =
            DdnProgram::from_source(script, "type_alias_diag_boolean.ddn").expect("parse");
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
        let program =
            DdnProgram::from_source(script, "type_alias_diag_geurimpyo.ddn").expect("parse");
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
        let program =
            DdnProgram::from_source(script, "type_alias_diag_valuepack.ddn").expect("parse");
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
        let program = DdnProgram::from_source(script, "numeric_family_exact_rational_add.ddn")
            .expect("parse");
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
        let program =
            DdnProgram::from_source(script, "numeric_family_exact_bigint_ops.ddn").expect("parse");
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
        let program = DdnProgram::from_source(script, "numeric_family_factor_from_bigint_ok.ddn")
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
        let program = DdnProgram::from_source(&script, "numeric_family_factor_route_metrics.ddn")
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

        let summary = match output
            .resources
            .get(NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY)
        {
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

        let summary = match output
            .resources
            .get(NUMERIC_FACTOR_ROUTE_SUMMARY_RESOURCE_KEY)
        {
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
        assert_eq!(
            events[0].message.as_deref(),
            Some(expected_message.as_str())
        );
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
        assert_eq!(events[0].reason, NUMERIC_DIAG_REASON_FACTOR_DECOMP_DEFERRED);
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
                assert_eq!(
                    text,
                    "지연|분해실패|deferred:factorfailed|9223372036854775809"
                )
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
        assert_eq!(
            span.file,
            "numeric_family_factor_deferred_eval_callable_span.ddn"
        );
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
        let program = DdnProgram::from_source(script, "numeric_family_grouped_integer_strings.ddn")
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
        let program = DdnProgram::from_source(script, "numeric_family_grouped_integer_invalid.ddn")
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
        assert!(
            message.contains("기대=나눔수"),
            "unexpected error: {message}"
        );
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
        let program = DdnProgram::from_source(script, "decl_type_alias_mismatch_rational.ddn")
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
            message.contains("E_RUNTIME_TYPE_MISMATCH"),
            "unexpected error: {message}"
        );
        assert!(
            message.contains("기대=나눔수"),
            "unexpected error: {message}"
        );
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
        let program =
            DdnProgram::from_source(script, "decl_type_alias_non_mismatch.ddn").expect("parse");
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
    fn numeric_kernel_factor_complete_large_power_shared_crate() {
        let input: BigInt = (BigInt::from(1u8) << 520) * BigInt::from(9u8);
        let outcome =
            ddonirang_numeric::complete_factor(&input.to_string()).expect("numeric kernel factor");
        assert_eq!(outcome.result.status, "done");
        assert_eq!(outcome.result.canonical.as_deref(), Some("2^520 * 3^2"));
        assert!(outcome.result.certificate.product_matches_input);
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

    #[test]
    fn parse_warnings_include_chaebi_reassign_warning() {
        let script = r#"
매틱:움직씨 = {
    채비 { 값:수 <- 0. }.
    값 <- 1.
}
"#;
        let program = DdnProgram::from_source(script, "warn_chaebi_reassign.ddn").expect("parse");
        assert!(program
            .parse_warnings()
            .iter()
            .any(|w| w.code == "W_CHAEBI_REDUNDANT_TOP_REASSIGN"));
    }

    #[test]
    fn parse_warnings_skip_chaebi_derived_reassign_warning() {
        let script = r#"
매틱:움직씨 = {
    채비 {
        점수:수 <- 72.
        통과함:참거짓 <- 거짓.
    }.
    통과함 <- 점수 >= 70.
    통과함 보여주기.
}
"#;
        let program = DdnProgram::from_source(script, "warn_chaebi_derived.ddn").expect("parse");
        assert!(!program
            .parse_warnings()
            .iter()
            .any(|w| w.code == "W_CHAEBI_REDUNDANT_TOP_REASSIGN"));
    }

    #[test]
    fn top_level_chaebi_assignment_updates_resource_and_show_log() {
        let script = r#"
채비 {
  점수: 수 <- 72.
  통과함: 참거짓 <- 거짓.
}.

통과함 <- 점수 >= 70.

통과함 보여주기.
"#;
        let program = DdnProgram::from_source(script, "top_level_show.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        match output.resources.get("통과함") {
            Some(RuntimeValue::Bool(value)) => assert!(*value),
            other => panic!("통과함 must be true, got {:?}", other),
        }
        let Some(RuntimeValue::List(entries)) = output.resources.get("output_log") else {
            panic!("output_log must be list");
        };
        let first = entries.first().expect("first output log entry");
        let RuntimeValue::Map(map) = first else {
            panic!("output log entry must be map");
        };
        let text = map
            .values()
            .find(|entry| matches!(&entry.key, RuntimeValue::String(key) if key == "text"))
            .and_then(|entry| match &entry.value {
                RuntimeValue::String(value) => Some(value.as_str()),
                _ => None,
            });
        assert_eq!(text, Some("참"));
    }

    #[test]
    fn boim_emits_view_rows_and_graph_points_without_output_log() {
        let script = r#"
채비 {
  t: 수 <- 1.
  위치: 수 <- 7.
}.

보임 {
  x축: t.
  값: 위치.
}.
"#;
        let program = DdnProgram::from_source(script, "boim_output.ddn").expect("parse");
        let mut runner = DdnRunner::new(program, "매틱");
        let world = NuriWorld::new();
        let output = runner
            .run_update(&world, &empty_input(), &HashMap::new())
            .expect("run update");
        assert!(!output.resources.contains_key("output_log"));
        let Some(RuntimeValue::List(lines)) = output.resources.get(BOGAE_SHOW_LINES_TAG) else {
            panic!("보개_출력_줄들 must be list");
        };
        let texts: Vec<String> = lines
            .iter()
            .filter_map(|value| match value {
                RuntimeValue::String(text) => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, vec!["table.row", "x축", "1", "table.row", "값", "7"]);
        let Some(RuntimeValue::List(points)) = output.resources.get(BOGAE_GRAPH_POINTS_F_TAG)
        else {
            panic!("보개_그래프_점목록_f must be list");
        };
        assert_eq!(points.len(), 1);
    }
}
