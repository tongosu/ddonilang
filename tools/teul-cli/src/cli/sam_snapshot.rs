use crate::cli::input_tape::KEY_REGISTRY_KEYS;
use crate::core::fixed64::Fixed64;
use crate::core::geoul::{InputSnapshotV1, NetEventV1};
use crate::core::state::Key;
use crate::core::unit::UnitDim;
use crate::core::value::{Quantity, Value};
use crate::core::State;

pub fn apply_snapshot(state: &mut State, snapshot: &InputSnapshotV1) {
    clear_sam_keys(state);
    apply_keyboard_mask(
        state,
        snapshot.held_mask,
        snapshot.pressed_mask,
        snapshot.released_mask,
    );
    apply_net_events(state, &snapshot.net_events);
}

pub fn snapshot_from_held_mask(
    madi: u64,
    seed: u64,
    held_mask: u16,
    last_mask: u16,
) -> InputSnapshotV1 {
    let pressed_mask = (!last_mask) & held_mask;
    let released_mask = last_mask & !held_mask;
    InputSnapshotV1 {
        madi,
        held_mask,
        pressed_mask,
        released_mask,
        rng_seed: seed,
        net_events: Vec::new(),
    }
}

fn clear_sam_keys(state: &mut State) {
    let keys = state
        .resources
        .keys()
        .filter(|key| {
            let name = key.as_str();
            name.starts_with("샘.") || name.starts_with("입력상태.")
        })
        .cloned()
        .collect::<Vec<_>>();
    for key in keys {
        state.resources.remove(&key);
    }
}

fn key_aliases(key: &str) -> &'static [&'static str] {
    match key {
        "ArrowLeft" => &["왼쪽화살표", "왼쪽", "좌"],
        "ArrowRight" => &["오른쪽화살표", "오른쪽", "우"],
        "ArrowDown" => &["아래쪽화살표", "아래쪽", "아래", "하"],
        "ArrowUp" => &["위쪽화살표", "위쪽", "위", "상"],
        "Space" => &["스페이스", "스페이스바", "공백"],
        "Enter" => &["엔터", "엔터키"],
        "Escape" => &["이스케이프", "이스케이프키"],
        "KeyZ" => &["Z키", "지키"],
        "KeyX" => &["X키", "엑스키"],
        _ => &[],
    }
}

fn apply_keyboard_mask(state: &mut State, held: u16, pressed: u16, released: u16) {
    for (idx, key) in KEY_REGISTRY_KEYS.iter().enumerate() {
        let bit = 1u16 << idx;
        let held_on = if held & bit != 0 { 1 } else { 0 };
        let pressed_on = if pressed & bit != 0 { 1 } else { 0 };
        let released_on = if released & bit != 0 { 1 } else { 0 };
        set_flag_number(
            state,
            format!("샘.키보드.누르고있음.{}", key),
            held_on,
        );
        set_flag_number(state, format!("샘.키보드.눌림.{}", key), pressed_on);
        set_flag_number(state, format!("샘.키보드.뗌.{}", key), released_on);

        set_flag_number(
            state,
            format!("입력상태.키_누르고있음.{}", key),
            held_on,
        );
        set_flag_number(
            state,
            format!("입력상태.키_눌림.{}", key),
            pressed_on,
        );
        set_flag_number(state, format!("입력상태.키_뗌.{}", key), released_on);

        for alias in key_aliases(key) {
            set_flag_number(
                state,
                format!("샘.키보드.누르고있음.{}", alias),
                held_on,
            );
            set_flag_number(state, format!("샘.키보드.눌림.{}", alias), pressed_on);
            set_flag_number(state, format!("샘.키보드.뗌.{}", alias), released_on);

            set_flag_number(
                state,
                format!("입력상태.키_누르고있음.{}", alias),
                held_on,
            );
            set_flag_number(
                state,
                format!("입력상태.키_눌림.{}", alias),
                pressed_on,
            );
            set_flag_number(state, format!("입력상태.키_뗌.{}", alias), released_on);
        }
    }
}

fn apply_net_events(state: &mut State, net_events: &[NetEventV1]) {
    if net_events.is_empty() {
        set_flag_number(state, "샘.네트워크.이벤트_개수".to_string(), 0);
        set_flag_text(state, "샘.네트워크.이벤트_요약".to_string(), String::new());
        return;
    }
    let mut summary = String::new();
    for (idx, event) in net_events.iter().enumerate() {
        if idx > 0 {
            summary.push('\n');
        }
        summary.push_str(&event.sender);
        summary.push('\t');
        summary.push_str(&event.seq.to_string());
        summary.push('\t');
        summary.push_str(&event.order_key);
        summary.push('\t');
        summary.push_str(&event.payload);
    }
    set_flag_number(
        state,
        "샘.네트워크.이벤트_개수".to_string(),
        net_events.len() as i64,
    );
    set_flag_text(
        state,
        "샘.네트워크.이벤트_요약".to_string(),
        summary,
    );
}

fn set_flag_number(state: &mut State, key: String, value: i64) {
    let qty = Quantity::new(Fixed64::from_int(value), UnitDim::zero());
    state.resources.insert(Key::new(key), Value::Num(qty));
}

fn set_flag_text(state: &mut State, key: String, value: String) {
    state.resources.insert(Key::new(key), Value::Str(value));
}
