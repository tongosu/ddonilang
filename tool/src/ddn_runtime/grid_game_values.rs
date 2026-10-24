//! Grid, piece, path and random-bag value operations; no evaluator dispatch.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use ddonirang_lang::runtime::{InputState, Value};
use super::{EvalError, fixed_value, splitmix64_next, std_input_map_fields, input_map_action_pressed, values_equal, value_to_i64, value_to_string};

pub(super) const STD_GRID_KIND: &str = "표준.격자";
pub(super) const STD_BLOCK_PIECE_KIND: &str = "std_block_piece";
const STD_BLOCK_PIECE_KIND_FIELD: &str = "__종류";
const STD_BLOCK_PIECE_CELLS_FIELD: &str = "칸들";
pub(super) const STD_RANDOM_BAG_KIND: &str = "std_random_bag";
pub(super) const STD_RANDOM_BAG_DRAW_KIND: &str = "std_random_bag_draw";
pub(super) const STD_RANDOM_BAG_KIND_FIELD: &str = "__종류";
const STD_RANDOM_BAG_SEED_FIELD: &str = "시앗";
const STD_RANDOM_BAG_STATE_FIELD: &str = "상태";
const STD_RANDOM_BAG_ORIGINAL_FIELD: &str = "원본";
pub(super) const STD_RANDOM_BAG_REMAINING_FIELD: &str = "남은것";
const STD_RANDOM_BAG_DRAWS_FIELD: &str = "뽑은수";
pub(super) const STD_RANDOM_BAG_VALUE_FIELD: &str = "값";
pub(super) const STD_RANDOM_BAG_BAG_FIELD: &str = "가방";
pub(super) const STD_GRID_GAME_STATE_KIND: &str = "std_grid_game_state";
const STD_GRID_GAME_STATE_KIND_FIELD: &str = "__종류";
const STD_GRID_GAME_STATE_STATE_FIELD: &str = "상태";
const STD_GRID_GAME_STATE_PREV_FIELD: &str = "이전상태";
const STD_GRID_GAME_STATE_TICK_FIELD: &str = "틱";
pub(super) const STD_GRID_LINE_CLEAR_KIND: &str = "std_grid_line_clear";
pub(super) const STD_FALLING_PIECE_KIND: &str = "std_falling_piece";
pub(super) const STD_GRID_GAME_SCORE_KIND: &str = "std_grid_game_score";
pub(super) const STD_GRID_GAME_SESSION_KIND: &str = "std_grid_game_session";
pub(super) const STD_GRID_GAME_TICK_KIND: &str = "std_grid_game_tick";
pub(super) const STD_GRID_GAME_NEXT_PIECE_KIND: &str = "std_grid_game_next_piece";
pub(super) const STD_GRID_GAME_VIEW_SUMMARY_KIND: &str = "std_grid_game_view_summary";
pub(super) const STD_GRID_GAME_HOLD_KIND: &str = "std_grid_game_hold_queue";
pub(super) const STD_GRID_GAME_HOLD_SWAP_KIND: &str = "std_grid_game_hold_swap";
pub(super) const STD_GRID_GAME_ROTATION_TRY_KIND: &str = "std_grid_game_rotation_try";

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

pub(super) fn make_std_grid(width: i64, height: i64, default_value: Value) -> Result<Value, EvalError> {
    if width <= 0 || height <= 0 {
        return Err(std_grid_error("격자 크기는 1 이상이어야 합니다"));
    }
    let cell_count = width
        .checked_mul(height)
        .and_then(|count| usize::try_from(count).ok())
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
        Value::List(vec![default_value; cell_count]),
    );
    Ok(Value::Pack(fields))
}

pub(super) fn std_grid_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_error("격자 인자가 필요합니다"));
    };
    match fields.get("__kind") {
        Some(Value::String(kind)) if kind == STD_GRID_KIND => Ok(fields),
        _ => Err(std_grid_error("격자 인자가 필요합니다")),
    }
}

pub(super) fn std_grid_dims(fields: &BTreeMap<String, Value>) -> Result<(i64, i64), EvalError> {
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

pub(super) fn std_grid_cells(fields: &BTreeMap<String, Value>) -> Result<&Vec<Value>, EvalError> {
    let Some(Value::List(cells)) = fields.get("칸들") else {
        return Err(std_grid_error("격자 칸 목록이 없습니다"));
    };
    Ok(cells)
}

pub(super) fn std_grid_index(fields: &BTreeMap<String, Value>, x: i64, y: i64) -> Result<usize, EvalError> {
    let (width, height) = std_grid_dims(fields)?;
    if x < 0 || y < 0 || x >= width || y >= height {
        return Err(std_grid_error("격자 좌표가 범위를 벗어났습니다"));
    }
    y.checked_mul(width)
        .and_then(|base| base.checked_add(x))
        .and_then(|index| usize::try_from(index).ok())
        .ok_or_else(|| std_grid_error("격자 크기가 너무 큽니다"))
}

pub(super) fn std_grid_inside(value: &Value, x: i64, y: i64) -> Result<bool, EvalError> {
    let fields = std_grid_fields(value)?;
    let (width, height) = std_grid_dims(fields)?;
    Ok(x >= 0 && y >= 0 && x < width && y < height)
}

fn std_block_piece_cell_from_value(value: &Value) -> Result<(i64, i64), EvalError> {
    // F5-GA[GRID_PAIR]: Rejecting len!=2 precedes both immutable coordinate reads.
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

pub(super) fn std_block_piece_cells_value(cells: &[(i64, i64)]) -> Value {
    Value::List(
        cells
            .iter()
            .map(|(x, y)| Value::List(vec![fixed_value(*x), fixed_value(*y)]))
            .collect(),
    )
}

pub(super) fn std_block_piece_cells_from_value(value: &Value) -> Result<Vec<(i64, i64)>, EvalError> {
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

pub(super) fn make_std_block_piece(mut cells: Vec<(i64, i64)>) -> Value {
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

pub(super) fn std_block_piece_cells(value: &Value) -> Result<Vec<(i64, i64)>, EvalError> {
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

pub(super) fn std_block_piece_collides(
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

pub(super) fn std_block_piece_lock(piece: &Value, grid: &Value, value: Value) -> Result<Value, EvalError> {
    // F5-GA[GRID_LOCK]: Checked coordinate arithmetic supplies idx, and idx<len is tested immediately before the local cell write.
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

pub(super) fn make_std_random_bag(
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

pub(super) fn std_random_bag_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_random_bag_error("무작위가방 인자가 필요합니다"));
    };
    match fields.get(STD_RANDOM_BAG_KIND_FIELD) {
        Some(Value::String(kind)) if kind == STD_RANDOM_BAG_KIND => Ok(fields),
        _ => Err(std_random_bag_error("무작위가방 인자가 필요합니다")),
    }
}

pub(super) fn std_random_bag_list_field(
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

pub(super) fn std_random_bag_draw_once(bag: &Value) -> Result<(Value, Value), EvalError> {
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

pub(super) fn is_allowed_grid_game_state(state: &str) -> bool {
    matches!(state, "준비" | "진행" | "잠금지연" | "정리" | "끝" | "멈춤")
}

pub(super) fn value_to_grid_game_state_name(value: &Value) -> Result<String, EvalError> {
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

pub(super) fn make_std_grid_game_state(state: &str, previous: Value, tick: i64) -> Value {
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

pub(super) fn std_grid_game_state_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_state_error("격자게임상태 인자가 필요합니다"));
    };
    match fields.get(STD_GRID_GAME_STATE_KIND_FIELD) {
        Some(Value::String(kind)) if kind == STD_GRID_GAME_STATE_KIND => Ok(fields),
        _ => Err(std_grid_game_state_error("격자게임상태 인자가 필요합니다")),
    }
}

pub(super) fn std_grid_game_state_state(fields: &BTreeMap<String, Value>) -> Result<String, EvalError> {
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

pub(super) fn std_grid_game_state_previous(fields: &BTreeMap<String, Value>) -> Value {
    fields
        .get(STD_GRID_GAME_STATE_PREV_FIELD)
        .cloned()
        .unwrap_or(Value::None)
}

pub(super) fn std_grid_game_state_tick(fields: &BTreeMap<String, Value>) -> Result<i64, EvalError> {
    fields
        .get(STD_GRID_GAME_STATE_TICK_FIELD)
        .ok_or_else(|| std_grid_game_state_error("격자게임상태 틱이 없습니다"))
        .and_then(value_to_i64)
}

fn std_grid_game_error(message: &str) -> EvalError {
    EvalError::from(message.to_string())
}

pub(super) fn tetromino_names() -> [&'static str; 7] {
    ["I", "O", "T", "S", "Z", "J", "L"]
}

pub(super) fn tetromino_cells(name: &str) -> Result<Vec<(i64, i64)>, EvalError> {
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

pub(super) fn value_to_tetromino_name(value: &Value) -> Result<String, EvalError> {
    let Value::String(name) = value else {
        return Err(std_grid_game_error("테트로미노 이름은 글이어야 합니다"));
    };
    let _ = tetromino_cells(name)?;
    Ok(name.clone())
}

pub(super) fn std_grid_line_full_rows(grid: &Value, empty_value: &Value) -> Result<Vec<i64>, EvalError> {
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

pub(super) fn std_grid_line_clear(grid: &Value, empty_value: Value) -> Result<Value, EvalError> {
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

pub(super) fn make_std_falling_piece(piece: Value, x: i64, y: i64) -> Value {
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

pub(super) fn std_falling_piece_parts(value: &Value) -> Result<(Value, i64, i64), EvalError> {
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

pub(super) fn std_falling_piece_cells(value: &Value) -> Result<Vec<(i64, i64)>, EvalError> {
    let (piece, x, y) = std_falling_piece_parts(value)?;
    Ok(std_block_piece_cells(&piece)?
        .into_iter()
        .map(|(cx, cy)| (cx + x, cy + y))
        .collect())
}

pub(super) fn std_falling_piece_block(value: &Value) -> Result<Value, EvalError> {
    Ok(make_std_block_piece(std_falling_piece_cells(value)?))
}

pub(super) fn std_falling_piece_rotate(value: &Value, direction: &str) -> Result<Value, EvalError> {
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

pub(super) fn make_std_grid_game_score(score: i64, lines: i64) -> Value {
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

pub(super) fn std_grid_game_score_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_error("격자게임점수 인자가 필요합니다"));
    };
    match fields.get("__종류") {
        Some(Value::String(kind)) if kind == STD_GRID_GAME_SCORE_KIND => Ok(fields),
        _ => Err(std_grid_game_error("격자게임점수 인자가 필요합니다")),
    }
}

pub(super) fn std_score_int_field(fields: &BTreeMap<String, Value>, field: &str) -> Result<i64, EvalError> {
    fields
        .get(field)
        .ok_or_else(|| std_grid_game_error(&format!("격자게임점수 {field}가 없습니다")))
        .and_then(value_to_i64)
}

pub(super) fn std_grid_game_score_add(score: &Value, cleared: i64) -> Result<Value, EvalError> {
    if !(0..=4).contains(&cleared) {
        return Err(std_grid_game_error("지운줄수는 0..4 범위여야 합니다"));
    }
    let fields = std_grid_game_score_fields(score)?;
    let current_score = std_score_int_field(fields, "점수")?;
    let current_lines = std_score_int_field(fields, "줄수")?;
    let level = 1 + current_lines.div_euclid(10);
    let base: i64 = match cleared {
        0 => 0,
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => unreachable!(),
    };
    let add = base.checked_mul(level).ok_or_else(|| {
        std_grid_game_error("격자게임점수 중간 곱셈이 바른수 범위를 벗어났습니다")
    })?;
    Ok(make_std_grid_game_score(
        current_score.saturating_add(add),
        current_lines.saturating_add(cleared),
    ))
}

pub(super) fn make_std_grid_game_session(
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

pub(super) fn std_grid_game_session_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_error("격자게임세션 인자가 필요합니다"));
    };
    match fields.get("__종류") {
        Some(Value::String(kind)) if kind == STD_GRID_GAME_SESSION_KIND => Ok(fields),
        _ => Err(std_grid_game_error("격자게임세션 인자가 필요합니다")),
    }
}

pub(super) fn std_session_field(fields: &BTreeMap<String, Value>, field: &str) -> Result<Value, EvalError> {
    fields
        .get(field)
        .cloned()
        .ok_or_else(|| std_grid_game_error(&format!("격자게임세션 {field}가 없습니다")))
}

pub(super) fn std_grid_game_next_piece(bag: &Value) -> Result<Value, EvalError> {
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

pub(super) fn make_std_grid_game_hold(piece: Value, used: bool) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::String(STD_GRID_GAME_HOLD_KIND.to_string()),
    );
    fields.insert("조각".to_string(), piece);
    fields.insert("썼나".to_string(), Value::Bool(used));
    Value::Pack(fields)
}

pub(super) fn std_grid_game_hold_fields(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(std_grid_game_error("격자게임홀드 인자가 필요합니다"));
    };
    match fields.get("__종류") {
        Some(Value::String(kind)) if kind == STD_GRID_GAME_HOLD_KIND => Ok(fields),
        _ => Err(std_grid_game_error("격자게임홀드 인자가 필요합니다")),
    }
}

pub(super) fn std_grid_game_hold_piece(fields: &BTreeMap<String, Value>) -> Result<Value, EvalError> {
    let piece = fields.get("조각").cloned().unwrap_or(Value::None);
    if !matches!(piece, Value::None) {
        let _ = std_block_piece_cells(&piece)?;
    }
    Ok(piece)
}

pub(super) fn std_grid_game_hold_used(fields: &BTreeMap<String, Value>) -> Result<bool, EvalError> {
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

pub(super) fn std_grid_game_hold_swap(
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

pub(super) fn std_grid_game_placeable(
    falling: &Value,
    grid: &Value,
    blocked: &Value,
) -> Result<bool, EvalError> {
    std_block_piece_collides(&std_falling_piece_block(falling)?, grid, blocked).map(|v| !v)
}

pub(super) fn std_grid_game_rotation_try(
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

pub(super) fn std_grid_game_apply_input(
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

pub(super) fn std_grid_game_tick(
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
    // F5-GA[GRID_PROJECT]: Here and in ghost projection, checked positive width*height must fit usize and equal cells.len; text consumes exactly the constructed rectangular output.
    let session_fields = std_grid_game_session_fields(session)?;
    let grid = std_session_field(session_fields, "격자")?;
    let falling = std_session_field(session_fields, "낙하조각")?;
    let grid_fields = std_grid_fields(&grid)?;
    let (width, height) = std_grid_dims(grid_fields)?;
    let cells = std_grid_cells(grid_fields)?;
    let expected_len = width
        .checked_mul(height)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"))?;
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

pub(super) fn std_grid_game_view_cells(
    session: &Value,
    empty_value: &Value,
    falling_value: Value,
) -> Result<Value, EvalError> {
    let (_, _, cells) = std_grid_game_view_project(session, empty_value, falling_value)?;
    Ok(Value::List(cells))
}

pub(super) fn std_grid_game_ghost_piece(session: &Value, blocked: &Value) -> Result<Value, EvalError> {
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
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| std_grid_error("격자 칸 목록이 크기와 맞지 않습니다"))?;
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

pub(super) fn std_grid_game_view_text(
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

pub(super) fn std_grid_game_view_summary(session: &Value) -> Result<Value, EvalError> {
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
        let Value::Pack(cell_fields) = &value else {
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

pub(super) fn std_grid_game_bogae_drawlist(
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

pub(super) fn std_grid_game_ghost_bogae_drawlist(
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

pub(super) fn std_grid_game_bogae_size(session: &Value, cell_size: &Value) -> Result<Value, EvalError> {
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

pub(super) fn value_matches_any(target: &Value, blocked: &Value) -> bool {
    match blocked {
        Value::List(items) => items.iter().any(|item| values_equal(target, item)),
        other => values_equal(target, other),
    }
}

pub(super) fn std_grid_pathfind(
    grid: &Value,
    start_x: i64,
    start_y: i64,
    goal_x: i64,
    goal_y: i64,
    blocked_values: &Value,
) -> Result<Value, EvalError> {
    // F5-GA[GRID_PATH]: Checked index/count and immutable extents bound all queue coordinates and prev links; this does not bound feasible allocation size.
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

    let cell_count = width.checked_mul(height)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| std_grid_error("격자 크기가 너무 큽니다"))?;
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
