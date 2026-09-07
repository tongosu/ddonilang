use ddonirang_core::InputSource;

use crate::input_tape::KEY_REGISTRY_KEYS;

const SNAPSHOT_MAGIC: &[u8; 11] = b"DDN_SAM_V1\n";
const SNAPSHOT_SOURCE_EXT_MAGIC: &[u8; 4] = b"ISRC";
const SNAPSHOT_SOURCE_EXT_VERSION: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetEventV1 {
    pub sender: String,
    pub seq: u64,
    pub order_key: String,
    pub payload: String,
    pub source: InputSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputSnapshotV1 {
    pub madi: u64,
    pub held_mask: u16,
    pub pressed_mask: u16,
    pub released_mask: u16,
    pub rng_seed: u64,
    pub frame_source: InputSource,
    pub net_events: Vec<NetEventV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotStateValue {
    Number(i64),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotStateEntry {
    pub key: String,
    pub value: SnapshotStateValue,
}

pub fn snapshot_from_held_mask(
    madi: u64,
    seed: u64,
    held_mask: u16,
    last_mask: u16,
) -> InputSnapshotV1 {
    InputSnapshotV1 {
        madi,
        held_mask,
        pressed_mask: (!last_mask) & held_mask,
        released_mask: last_mask & !held_mask,
        rng_seed: seed,
        frame_source: InputSource::Relay,
        net_events: Vec::new(),
    }
}

pub fn project_snapshot_state(snapshot: &InputSnapshotV1) -> Vec<SnapshotStateEntry> {
    let mut entries = project_keyboard_state(
        snapshot.held_mask,
        snapshot.pressed_mask,
        snapshot.released_mask,
    );
    entries.extend(project_net_event_state(&snapshot.net_events));
    entries
}

pub fn project_keyboard_state(held: u16, pressed: u16, released: u16) -> Vec<SnapshotStateEntry> {
    let mut entries = Vec::new();
    for (idx, key) in KEY_REGISTRY_KEYS.iter().enumerate() {
        let bit = 1u16 << idx;
        let values = [
            ("누르고있음", i64::from(held & bit != 0)),
            ("눌림", i64::from(pressed & bit != 0)),
            ("뗌", i64::from(released & bit != 0)),
        ];
        for name in std::iter::once(*key).chain(key_aliases(key).iter().copied()) {
            for (suffix, value) in values {
                push_number(&mut entries, format!("샘.키보드.{suffix}.{name}"), value);
            }
            push_number(
                &mut entries,
                format!("입력상태.키_누르고있음.{name}"),
                values[0].1,
            );
            push_number(
                &mut entries,
                format!("입력상태.키_눌림.{name}"),
                values[1].1,
            );
            push_number(&mut entries, format!("입력상태.키_뗌.{name}"), values[2].1);
        }
    }
    entries
}

pub fn project_net_event_state(net_events: &[NetEventV1]) -> Vec<SnapshotStateEntry> {
    let mut entries = Vec::new();
    let summary = net_events
        .iter()
        .map(|event| {
            format!(
                "{}\t{}\t{}\t{}",
                event.sender, event.seq, event.order_key, event.payload
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let source_summary = net_events
        .iter()
        .map(|event| event.source.label())
        .collect::<Vec<_>>()
        .join("\n");
    push_number(
        &mut entries,
        "샘.네트워크.이벤트_개수".to_string(),
        net_events.len() as i64,
    );
    push_text(&mut entries, "샘.네트워크.이벤트_요약".to_string(), summary);
    push_text(
        &mut entries,
        "샘.네트워크.이벤트_원천요약".to_string(),
        source_summary,
    );
    entries
}

fn push_number(entries: &mut Vec<SnapshotStateEntry>, key: String, value: i64) {
    entries.push(SnapshotStateEntry {
        key,
        value: SnapshotStateValue::Number(value),
    });
}

fn push_text(entries: &mut Vec<SnapshotStateEntry>, key: String, value: String) {
    entries.push(SnapshotStateEntry {
        key,
        value: SnapshotStateValue::Text(value),
    });
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

/// Canonical default key bindings for the standard input-map actions.
///
/// Native, WASM and retained CLI runtimes consume this registry so action
/// defaults cannot drift between execution frontdoors.
pub fn input_action_key_aliases(action: &str) -> &'static [&'static str] {
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

pub fn encode_input_snapshot(snapshot: &InputSnapshotV1) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(SNAPSHOT_MAGIC);
    out.extend_from_slice(&snapshot.madi.to_le_bytes());
    out.extend_from_slice(&snapshot.held_mask.to_le_bytes());
    out.extend_from_slice(&snapshot.pressed_mask.to_le_bytes());
    out.extend_from_slice(&snapshot.released_mask.to_le_bytes());
    out.extend_from_slice(&snapshot.rng_seed.to_le_bytes());
    out.extend_from_slice(&(snapshot.net_events.len() as u32).to_le_bytes());
    for event in &snapshot.net_events {
        push_str(&mut out, &event.sender);
        out.extend_from_slice(&event.seq.to_le_bytes());
        push_str(&mut out, &event.order_key);
        push_str(&mut out, &event.payload);
    }
    out.extend_from_slice(SNAPSHOT_SOURCE_EXT_MAGIC);
    out.push(SNAPSHOT_SOURCE_EXT_VERSION);
    out.push(snapshot.frame_source.code_u8());
    out.extend_from_slice(&(snapshot.net_events.len() as u32).to_le_bytes());
    for event in &snapshot.net_events {
        out.push(event.source.code_u8());
    }
    out
}

pub fn decode_input_snapshot(bytes: &[u8]) -> Result<InputSnapshotV1, String> {
    let mut idx = 0usize;
    if bytes.len() < SNAPSHOT_MAGIC.len() {
        return Err("snapshot detbin 길이가 너무 짧습니다".to_string());
    }
    let magic = &bytes[..SNAPSHOT_MAGIC.len()];
    if magic != SNAPSHOT_MAGIC {
        return Err("snapshot detbin magic 불일치".to_string());
    }
    idx += SNAPSHOT_MAGIC.len();
    let madi = read_u64_slice(bytes, &mut idx)?;
    let held_mask = read_u16_slice(bytes, &mut idx)?;
    let pressed_mask = read_u16_slice(bytes, &mut idx)?;
    let released_mask = read_u16_slice(bytes, &mut idx)?;
    let rng_seed = read_u64_slice(bytes, &mut idx)?;
    let event_count = read_u32_slice(bytes, &mut idx)? as usize;
    let mut net_events = Vec::with_capacity(event_count);
    for _ in 0..event_count {
        let sender = read_str_slice(bytes, &mut idx)?;
        let seq = read_u64_slice(bytes, &mut idx)?;
        let order_key = read_str_slice(bytes, &mut idx)?;
        let payload = read_str_slice(bytes, &mut idx)?;
        net_events.push(NetEventV1 {
            sender,
            seq,
            order_key,
            payload,
            source: InputSource::Person,
        });
    }
    let mut frame_source = InputSource::Person;
    if idx < bytes.len() {
        if idx.saturating_add(SNAPSHOT_SOURCE_EXT_MAGIC.len() + 1 + 1 + 4) > bytes.len() {
            return Err("snapshot detbin source extension EOF".to_string());
        }
        if &bytes[idx..idx + SNAPSHOT_SOURCE_EXT_MAGIC.len()] != SNAPSHOT_SOURCE_EXT_MAGIC {
            return Err("snapshot detbin source extension magic 불일치".to_string());
        }
        idx += SNAPSHOT_SOURCE_EXT_MAGIC.len();
        let version = read_u8_slice(bytes, &mut idx)?;
        if version != SNAPSHOT_SOURCE_EXT_VERSION {
            return Err(format!(
                "snapshot detbin source extension version 불일치: {version}"
            ));
        }
        frame_source = decode_input_source(read_u8_slice(bytes, &mut idx)?)?;
        let source_count = read_u32_slice(bytes, &mut idx)? as usize;
        if source_count != net_events.len() {
            return Err("snapshot detbin source extension event count 불일치".to_string());
        }
        for event in &mut net_events {
            event.source = decode_input_source(read_u8_slice(bytes, &mut idx)?)?;
        }
    }
    if idx != bytes.len() {
        return Err("snapshot detbin에 여분 바이트가 있습니다".to_string());
    }
    Ok(InputSnapshotV1 {
        madi,
        held_mask,
        pressed_mask,
        released_mask,
        rng_seed,
        frame_source,
        net_events,
    })
}

fn push_str(out: &mut Vec<u8>, text: &str) {
    let bytes = text.as_bytes();
    let len = bytes.len() as u64;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
}

fn read_u8_slice(bytes: &[u8], idx: &mut usize) -> Result<u8, String> {
    if *idx >= bytes.len() {
        return Err("snapshot detbin EOF".to_string());
    }
    let out = bytes[*idx];
    *idx += 1;
    Ok(out)
}

fn read_u16_slice(bytes: &[u8], idx: &mut usize) -> Result<u16, String> {
    let end = idx.saturating_add(2);
    if end > bytes.len() {
        return Err("snapshot detbin EOF".to_string());
    }
    let out = u16::from_le_bytes([bytes[*idx], bytes[*idx + 1]]);
    *idx = end;
    Ok(out)
}

fn read_u32_slice(bytes: &[u8], idx: &mut usize) -> Result<u32, String> {
    let end = idx.saturating_add(4);
    if end > bytes.len() {
        return Err("snapshot detbin EOF".to_string());
    }
    let out = u32::from_le_bytes([
        bytes[*idx],
        bytes[*idx + 1],
        bytes[*idx + 2],
        bytes[*idx + 3],
    ]);
    *idx = end;
    Ok(out)
}

fn read_u64_slice(bytes: &[u8], idx: &mut usize) -> Result<u64, String> {
    let end = idx.saturating_add(8);
    if end > bytes.len() {
        return Err("snapshot detbin EOF".to_string());
    }
    let out = u64::from_le_bytes([
        bytes[*idx],
        bytes[*idx + 1],
        bytes[*idx + 2],
        bytes[*idx + 3],
        bytes[*idx + 4],
        bytes[*idx + 5],
        bytes[*idx + 6],
        bytes[*idx + 7],
    ]);
    *idx = end;
    Ok(out)
}

fn read_str_slice(bytes: &[u8], idx: &mut usize) -> Result<String, String> {
    let len = read_u64_slice(bytes, idx)? as usize;
    let end = idx.saturating_add(len);
    if end > bytes.len() {
        return Err("snapshot detbin 문자열 EOF".to_string());
    }
    let text = std::str::from_utf8(&bytes[*idx..end])
        .map_err(|_| "snapshot detbin UTF-8 오류".to_string())?;
    *idx = end;
    Ok(text.to_string())
}

fn decode_input_source(value: u8) -> Result<InputSource, String> {
    InputSource::from_code_u8(value)
        .ok_or_else(|| format!("snapshot detbin input source 코드 오류: {value}"))
}

#[cfg(test)]
mod tests {
    use super::{
        decode_input_snapshot, encode_input_snapshot, input_action_key_aliases,
        project_snapshot_state, push_str,
        snapshot_from_held_mask, InputSnapshotV1, NetEventV1, SnapshotStateEntry,
        SnapshotStateValue, SNAPSHOT_MAGIC, SNAPSHOT_SOURCE_EXT_MAGIC,
    };
    use ddonirang_core::InputSource;

    fn snapshot() -> InputSnapshotV1 {
        InputSnapshotV1 {
            madi: 7,
            held_mask: 1,
            pressed_mask: 1,
            released_mask: 0,
            rng_seed: 99,
            frame_source: InputSource::Relay,
            net_events: vec![NetEventV1 {
                sender: "peer".to_string(),
                seq: 3,
                order_key: "peer#3".to_string(),
                payload: "{\"kind\":\"k\"}".to_string(),
                source: InputSource::ExternalTask,
            }],
        }
    }

    #[test]
    fn source_extension_roundtrips() {
        let expected = snapshot();
        let decoded = decode_input_snapshot(&encode_input_snapshot(&expected)).expect("decode");
        assert_eq!(decoded, expected);
    }

    #[test]
    fn legacy_without_source_defaults_to_person() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(SNAPSHOT_MAGIC);
        bytes.extend_from_slice(&1u64.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&42u64.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        push_str(&mut bytes, "peer");
        bytes.extend_from_slice(&1u64.to_le_bytes());
        push_str(&mut bytes, "peer#1");
        push_str(&mut bytes, "{\"kind\":\"k\"}");

        let decoded = decode_input_snapshot(&bytes).expect("decode legacy");
        assert_eq!(decoded.frame_source, InputSource::Person);
        assert_eq!(decoded.net_events[0].source, InputSource::Person);
    }

    #[test]
    fn source_extension_event_count_mismatch_fails_closed() {
        let mut bytes = encode_input_snapshot(&snapshot());
        let extension = bytes
            .windows(SNAPSHOT_SOURCE_EXT_MAGIC.len())
            .position(|window| window == SNAPSHOT_SOURCE_EXT_MAGIC)
            .expect("source extension");
        let source_count = extension + SNAPSHOT_SOURCE_EXT_MAGIC.len() + 1 + 1;
        bytes[source_count..source_count + 4].copy_from_slice(&0u32.to_le_bytes());

        let error = decode_input_snapshot(&bytes).expect_err("count mismatch must fail closed");
        assert_eq!(error, "snapshot detbin source extension event count 불일치");
    }

    #[test]
    fn snapshot_projection_owns_keyboard_alias_and_network_state() {
        let entries = project_snapshot_state(&snapshot());
        assert!(entries.contains(&SnapshotStateEntry {
            key: "샘.키보드.누르고있음.ArrowLeft".to_string(),
            value: SnapshotStateValue::Number(1),
        }));
        assert!(entries.contains(&SnapshotStateEntry {
            key: "입력상태.키_누르고있음.왼쪽".to_string(),
            value: SnapshotStateValue::Number(1),
        }));
        assert!(entries.contains(&SnapshotStateEntry {
            key: "샘.네트워크.이벤트_요약".to_string(),
            value: SnapshotStateValue::Text("peer\t3\tpeer#3\t{\"kind\":\"k\"}".to_string()),
        }));
        assert!(entries.contains(&SnapshotStateEntry {
            key: "샘.네트워크.이벤트_원천요약".to_string(),
            value: SnapshotStateValue::Text(InputSource::ExternalTask.label().to_string()),
        }));
    }

    #[test]
    fn input_action_defaults_keep_canonical_keyboard_entries() {
        assert_eq!(input_action_key_aliases("오른쪽").first(), Some(&"ArrowRight"));
        assert!(input_action_key_aliases("확인").contains(&"Enter"));
        assert!(input_action_key_aliases("없는동작").is_empty());
    }

    #[test]
    fn held_mask_transition_is_shared_and_deterministic() {
        let snapshot = snapshot_from_held_mask(4, 9, 0b0101, 0b0011);
        assert_eq!(snapshot.held_mask, 0b0101);
        assert_eq!(snapshot.pressed_mask, 0b0100);
        assert_eq!(snapshot.released_mask, 0b0010);
        assert_eq!(snapshot.frame_source, InputSource::Relay);
    }
}
