#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use ddonirang_core::{KEY_A, KEY_D, KEY_S, KEY_W};

const MAGIC_V1: &[u8] = b"DDN_INPUT_TAPE_V1\n";
const MAGIC_V2: &[u8] = b"DDN_INPUT_TAPE_V2\n";
const VERSION_V1: u32 = 1;
const VERSION_V2: u32 = 2;
pub const CONTROL_REGISTRY_ID: &str = "KEY_REGISTRY_V2_CONTROL";
pub const CONTROL_REGISTRY_KEYS: [&str; 8] = ["ArrowLeft", "ArrowRight", "ArrowDown", "ArrowUp", "Space", "Enter", "KeyZ", "KeyX"];

pub const KEY_REGISTRY_ID: &str = "KEY_REGISTRY_V1_MIN";
pub const KEY_REGISTRY_KEYS: [&str; 9] = [
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputRecord {
    pub madi: u32,
    pub held_mask: Vec<u8>,
    pub events: Vec<InputEvent>,
}

impl InputRecord {
    pub(crate) fn runtime_edges(&self) -> (u64, u64) {
        let mut pressed = 0;
        let mut released = 0;
        for event in &self.events {
            match event.kind {
                InputEventKind::Press => pressed |= event.key_bit,
                InputEventKind::Release => released |= event.key_bit,
            }
        }
        (tape_mask_to_runtime_keys(pressed), tape_mask_to_runtime_keys(released))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventKind {
    Press,
    Release,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub sequence: u64,
    pub key_bit: u16,
    pub kind: InputEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputTape {
    /// V1 remains snapshot-only provenance; it is never implicitly upgraded.
    pub version: u32,
    pub madi_hz: u32,
    pub records: Vec<InputRecord>,
}

pub fn key_registry_string() -> String {
    KEY_REGISTRY_KEYS.join("\n")
}

pub fn key_registry_hash() -> [u8; 32] {
    *blake3::hash(key_registry_string().as_bytes()).as_bytes()
}

pub fn expected_mask_len() -> usize {
    (KEY_REGISTRY_KEYS.len() + 7) / 8
}

pub fn key_index(token: &str) -> Option<usize> {
    if token.eq_ignore_ascii_case("ArrowLeft")
        || token.eq_ignore_ascii_case("Left")
        || token == "왼쪽화살표"
        || token == "왼쪽"
        || token == "좌"
    {
        return Some(0);
    }
    if token.eq_ignore_ascii_case("ArrowRight")
        || token.eq_ignore_ascii_case("Right")
        || token == "오른쪽화살표"
        || token == "오른쪽"
        || token == "우"
    {
        return Some(1);
    }
    if token.eq_ignore_ascii_case("ArrowDown")
        || token.eq_ignore_ascii_case("Down")
        || token == "아래쪽화살표"
        || token == "아래쪽"
        || token == "아래"
        || token == "하"
    {
        return Some(2);
    }
    if token.eq_ignore_ascii_case("ArrowUp")
        || token.eq_ignore_ascii_case("Up")
        || token == "위쪽화살표"
        || token == "위쪽"
        || token == "위"
        || token == "상"
    {
        return Some(3);
    }
    if token.eq_ignore_ascii_case("Space")
        || token.eq_ignore_ascii_case("Spacebar")
        || token == "스페이스"
        || token == "스페이스바"
        || token == "공백"
    {
        return Some(4);
    }
    if token.eq_ignore_ascii_case("Enter") || token == "엔터" || token == "엔터키" {
        return Some(5);
    }
    if token.eq_ignore_ascii_case("Escape")
        || token.eq_ignore_ascii_case("Esc")
        || token == "이스케이프"
        || token == "이스케이프키"
    {
        return Some(6);
    }
    if token.eq_ignore_ascii_case("KeyZ")
        || token.eq_ignore_ascii_case("Z")
        || token.eq_ignore_ascii_case("ZKey")
        || token == "Z키"
        || token == "지키"
    {
        return Some(7);
    }
    if token.eq_ignore_ascii_case("KeyX")
        || token.eq_ignore_ascii_case("X")
        || token.eq_ignore_ascii_case("XKey")
        || token == "X키"
        || token == "엑스키"
    {
        return Some(8);
    }
    None
}

pub fn parse_held_mask(line: &str) -> Result<u16, String> {
    let mut mask: u16 = 0;
    for raw in line.split(|ch: char| ch.is_whitespace() || ch == ',') {
        let token = raw.trim();
        if token.is_empty() {
            continue;
        }
        let idx = key_index(token).ok_or_else(|| format!("알 수 없는 키 이름입니다: {token}"))?;
        mask |= 1u16 << idx;
    }
    Ok(mask)
}

pub fn mask_to_bytes(mask: u16) -> Vec<u8> {
    let bytes = mask.to_le_bytes();
    bytes[..expected_mask_len()].to_vec()
}

pub fn mask_from_bytes(bytes: &[u8]) -> Result<u16, String> {
    let needed = expected_mask_len();
    if bytes.len() != needed {
        return Err(format!(
            "held_mask 길이가 다릅니다: 기대 {needed}, 실제 {}",
            bytes.len()
        ));
    }
    let mut raw = [0u8; 2];
    raw[..needed].copy_from_slice(bytes);
    let mask = u16::from_le_bytes(raw);
    if mask & !((1u16 << KEY_REGISTRY_KEYS.len()) - 1) != 0 {
        return Err("E_SAM_TAPE_FOOTER_INVALID 등록되지 않은 키 비트입니다".into());
    }
    Ok(mask)
}

/// Builds the versioned input tape consumed by every product runtime frontdoor.
///
/// Interactive hosts may collect key masks themselves, but record ordering,
/// key-registry bounds and the tape identity stay in this shared authority.
pub fn input_tape_from_masks(madi_hz: u32, masks: &[u16]) -> Result<InputTape, String> {
    if madi_hz == 0 {
        return Err("madi_hz는 0보다 커야 합니다".to_string());
    }
    if masks.len() > u32::MAX as usize {
        return Err("입력 테이프 기록 수가 u32 한계를 넘었습니다".to_string());
    }
    let allowed_mask = (1u16 << KEY_REGISTRY_KEYS.len()) - 1;
    let mut records = Vec::with_capacity(masks.len());
    for (index, mask) in masks.iter().copied().enumerate() {
        if mask & !allowed_mask != 0 {
            return Err(format!(
                "입력 mask에 등록되지 않은 키 비트가 있습니다: {:#x}",
                mask & !allowed_mask
            ));
        }
        records.push(InputRecord {
            madi: index as u32,
            held_mask: mask_to_bytes(mask),
            events: Vec::new(),
        });
    }
    Ok(InputTape { version: VERSION_V1, madi_hz, records })
}

/// Converts the versioned input-tape key registry into the runtime key mask.
///
/// The tape registry is ordered by public key identity (left, right, down,
/// up), while the core runtime preserves the historical W/A/S/D bit layout.
/// Keeping this conversion in the shared tool authority prevents CLI and
/// Workbench consumers from rebuilding the mapping privately.
pub fn tape_mask_to_runtime_keys(mask: u16) -> u64 {
    let mut runtime = 0u64;
    if mask & (1 << 0) != 0 {
        runtime |= KEY_A;
    }
    if mask & (1 << 1) != 0 {
        runtime |= KEY_D;
    }
    if mask & (1 << 2) != 0 {
        runtime |= KEY_S;
    }
    if mask & (1 << 3) != 0 {
        runtime |= KEY_W;
    }
    for bit in 4..KEY_REGISTRY_KEYS.len() {
        if mask & (1 << bit) != 0 {
            runtime |= 1u64 << bit;
        }
    }
    runtime
}

/// Converts the shared runtime key mask back into the ordered input-tape
/// registry used by snapshot state projection.
pub fn runtime_keys_to_tape_mask(runtime: u64) -> u16 {
    let mut mask = 0u16;
    if runtime & KEY_A != 0 {
        mask |= 1 << 0;
    }
    if runtime & KEY_D != 0 {
        mask |= 1 << 1;
    }
    if runtime & KEY_S != 0 {
        mask |= 1 << 2;
    }
    if runtime & KEY_W != 0 {
        mask |= 1 << 3;
    }
    for bit in 4..KEY_REGISTRY_KEYS.len() {
        if runtime & (1u64 << bit) != 0 {
            mask |= 1u16 << bit;
        }
    }
    mask
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_input_tape(path: &Path, tape: &InputTape) -> Result<(), String> {
    let bytes = encode_input_tape(tape)?;
    crate::artifact_output::write_binary_artifact_atomic(path, &bytes)
}

pub fn encode_input_tape(tape: &InputTape) -> Result<Vec<u8>, String> {
    if !matches!(tape.version, VERSION_V1 | VERSION_V2) {
        return Err("E_SAM_TAPE_VERSION_UNSUPPORTED 지원하지 않는 테이프 버전입니다".into());
    }
    if tape.madi_hz == 0 || tape.records.len() > u32::MAX as usize {
        return Err("E_SAM_TAPE_FOOTER_INVALID 마디 속도 또는 기록 개수가 잘못되었습니다".into());
    }
    let mut previous_sequence = None;
    let mut held = 0u16;
    for (index, record) in tape.records.iter().enumerate() {
        let mask = mask_from_bytes(&record.held_mask)?;
        if record.madi != index as u32 {
            return Err("E_SAM_RECORD_ORDER_MISMATCH 기록의 마디 순서가 다릅니다".into());
        }
        if tape.version == VERSION_V2 && mask & (1 << 6) != 0 {
            return Err("E_SAM_TAPE_HOST_KEY_FORBIDDEN Escape는 호스트 종료키입니다".into());
        }
        if record.events.len() > 1024 {
            return Err("E_SAM_LIVE_QUEUE_OVERFLOW 마디별 사건 수가 1024개를 넘었습니다".into());
        }
        if tape.version == VERSION_V1 && !record.events.is_empty() {
            return Err("E_SAM_TAPE_VERSION_UNSUPPORTED V1에는 사건 순번을 기록할 수 없습니다".into());
        }
        for event in &record.events {
            validate_event(event, previous_sequence)?;
            previous_sequence = Some(event.sequence);
            match event.kind {
                InputEventKind::Press => held |= event.key_bit,
                InputEventKind::Release => held &= !event.key_bit,
            }
        }
        if tape.version == VERSION_V2 && held != mask_from_bytes(&record.held_mask)? {
            return Err("E_SAM_TAPE_FOOTER_INVALID 사건 순서와 최종 held mask가 다릅니다".into());
        }
    }
    let mut out = Vec::new();
    out.extend_from_slice(if tape.version == VERSION_V2 { MAGIC_V2 } else { MAGIC_V1 });
    out.extend_from_slice(&tape.version.to_le_bytes());
    let registry_id = if tape.version == VERSION_V2 { CONTROL_REGISTRY_ID } else { KEY_REGISTRY_ID }.as_bytes();
    out.extend_from_slice(&(registry_id.len() as u32).to_le_bytes());
    out.extend_from_slice(registry_id);
    let registry_hash = if tape.version == VERSION_V2 { *blake3::hash(CONTROL_REGISTRY_KEYS.join("\n").as_bytes()).as_bytes() } else { key_registry_hash() };
    out.extend_from_slice(&registry_hash);
    out.extend_from_slice(&tape.madi_hz.to_le_bytes());
    out.extend_from_slice(&(tape.records.len() as u32).to_le_bytes());
    for record in &tape.records {
        out.extend_from_slice(&record.madi.to_le_bytes());
        if tape.version == VERSION_V2 {
            let mask = mask_from_bytes(&record.held_mask)?;
            out.extend_from_slice(&1u32.to_le_bytes());
            out.push(((mask & 0x3f) | ((mask & 0x180) >> 1)) as u8);
        } else {
            out.extend_from_slice(&(record.held_mask.len() as u32).to_le_bytes());
            out.extend_from_slice(&record.held_mask);
        }
        if tape.version == VERSION_V1 { continue; }
        out.extend_from_slice(&(record.events.len() as u32).to_le_bytes());
        for event in &record.events {
            out.extend_from_slice(&event.sequence.to_le_bytes());
            let internal_index = event.key_bit.trailing_zeros() as u16;
            let key_index = if internal_index > 6 { internal_index - 1 } else { internal_index };
            out.extend_from_slice(&key_index.to_le_bytes());
            out.push(match event.kind {
                InputEventKind::Press => 0,
                InputEventKind::Release => 1,
            });
        }
    }
    if tape.version == VERSION_V2 {
        let digest = blake3::hash(&out);
        out.extend_from_slice(digest.as_bytes());
    }
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_input_tape(path: &Path) -> Result<InputTape, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    decode_input_tape(&bytes)
}

pub fn decode_input_tape(bytes: &[u8]) -> Result<InputTape, String> {
    let mut idx = 0usize;
    let magic = take_bytes(bytes, &mut idx, MAGIC_V1.len())?;
    if magic != MAGIC_V1 && magic != MAGIC_V2 {
        return Err("입력 테이프 형식 식별자가 잘못되었습니다".to_string());
    }
    let version = read_u32(bytes, &mut idx)?;
    if (magic == MAGIC_V1 && version != VERSION_V1) || (magic == MAGIC_V2 && version != VERSION_V2)
    {
        return Err(format!("E_SAM_TAPE_VERSION_UNSUPPORTED 지원하지 않는 테이프 버전입니다: {version}"));
    }
    let registry_len = read_u32(bytes, &mut idx)? as usize;
    let registry_id = if version == VERSION_V2 { CONTROL_REGISTRY_ID } else { KEY_REGISTRY_ID };
    if take_bytes(bytes, &mut idx, registry_len)? != registry_id.as_bytes() {
        return Err("키 레지스트리 식별자가 다릅니다 (key registry id mismatch)".to_string());
    }
    let registry_hash = if version == VERSION_V2 { *blake3::hash(CONTROL_REGISTRY_KEYS.join("\n").as_bytes()).as_bytes() } else { key_registry_hash() };
    if take_bytes(bytes, &mut idx, 32)? != registry_hash {
        return Err("키 레지스트리 해시가 다릅니다 (key registry hash mismatch)".to_string());
    }
    let madi_hz = read_u32(bytes, &mut idx)?;
    let record_count = read_u32(bytes, &mut idx)? as usize;
    if version == VERSION_V2 {
        verify_control_envelope(bytes, idx, record_count)?;
    }
    let mut records = Vec::with_capacity(record_count.min(4096));
    let mut previous_sequence = None;
    for _ in 0..record_count {
        let madi = read_u32(bytes, &mut idx)?;
        let mask_len = read_u32(bytes, &mut idx)? as usize;
        let held_mask = if version == VERSION_V2 {
            if mask_len != 1 { return Err("E_SAM_TAPE_FOOTER_INVALID 제어키 mask 길이가 다릅니다".into()); }
            let raw = u16::from(take_bytes(bytes, &mut idx, 1)?[0]);
            mask_to_bytes((raw & 0x3f) | ((raw & 0xc0) << 1))
        } else { take_bytes(bytes, &mut idx, mask_len)?.to_vec() };
        mask_from_bytes(&held_mask)?;
        let events = if version == VERSION_V2 {
            let event_count = read_u32(bytes, &mut idx)? as usize;
            if event_count > 1024 { return Err("E_SAM_LIVE_QUEUE_OVERFLOW 마디별 사건 수가 1024개를 넘었습니다".into()); }
            if event_count > (bytes.len().saturating_sub(idx) / 11) {
                return Err("E_SAM_TAPE_TRUNCATED 입력 사건 기록이 잘렸습니다".to_string());
            }
            let mut events = Vec::with_capacity(event_count);
            for _ in 0..event_count {
                let sequence = read_u64(bytes, &mut idx)?;
                let key_index = read_u16(bytes, &mut idx)?;
                if usize::from(key_index) >= CONTROL_REGISTRY_KEYS.len() {
                    return Err("E_SAM_TAPE_HOST_KEY_FORBIDDEN 허용되지 않은 제어키 식별자입니다".into());
                }
                let key_bit = 1u16 << if key_index >= 6 { key_index + 1 } else { key_index };
                let kind = match take_bytes(bytes, &mut idx, 1)?[0] {
                    0 => InputEventKind::Press,
                    1 => InputEventKind::Release,
                    _ => return Err("E_SAM_TAPE_FOOTER_INVALID 입력 사건 종류가 잘못되었습니다".to_string()),
                };
                let event = InputEvent {
                    sequence,
                    key_bit,
                    kind,
                };
                validate_event(&event, previous_sequence)?;
                previous_sequence = Some(sequence);
                events.push(event);
            }
            events
        } else {
            Vec::new()
        };
        records.push(InputRecord {
            madi,
            held_mask,
            events,
        });
    }
    if version == VERSION_V2 {
        take_bytes(bytes, &mut idx, 32)?;
    }
    if idx != bytes.len() {
        return Err("E_SAM_TAPE_TRAILING_BYTES 테이프 끝에 불필요한 바이트가 있습니다".to_string());
    }
    let tape = InputTape { version, madi_hz, records };
    encode_input_tape(&tape)?;
    Ok(tape)
}

/// Inspect bounded framing and verify all canonical bytes before allocating events.
fn verify_control_envelope(bytes: &[u8], mut index: usize, records: usize) -> Result<(), String> {
    if records > bytes.len().saturating_sub(index) / 13 {
        return Err("E_SAM_TAPE_TRUNCATED 마디 기록이 잘렸습니다".into());
    }
    for _ in 0..records {
        read_u32(bytes, &mut index)?;
        let mask_len = read_u32(bytes, &mut index)? as usize;
        if mask_len != 1 {
            return Err("E_SAM_TAPE_FOOTER_INVALID 제어키 mask 길이가 다릅니다".into());
        }
        take_bytes(bytes, &mut index, 1)?;
        let events = read_u32(bytes, &mut index)? as usize;
        if events > 1024 {
            return Err("E_SAM_LIVE_QUEUE_OVERFLOW 마디별 사건 수가 1024개를 넘었습니다".into());
        }
        take_bytes(bytes, &mut index, events * 11)?;
    }
    let body_end = index;
    let digest = take_bytes(bytes, &mut index, 32)?;
    if index != bytes.len() {
        return Err("E_SAM_TAPE_TRAILING_BYTES footer 뒤에 불필요한 바이트가 있습니다".into());
    }
    if digest != blake3::hash(&bytes[..body_end]).as_bytes() {
        return Err("E_SAM_TAPE_DIGEST_MISMATCH 입력 테이프가 손상되었습니다. 원본 기록을 확인하세요".into());
    }
    Ok(())
}

fn take_bytes<'a>(bytes: &'a [u8], idx: &mut usize, len: usize) -> Result<&'a [u8], String> {
    let end = idx.saturating_add(len);
    if end > bytes.len() {
        return Err("E_SAM_TAPE_TRUNCATED 입력 테이프가 잘렸습니다. 원본 기록을 확인하세요".to_string());
    }
    let out = &bytes[*idx..end];
    *idx = end;
    Ok(out)
}

fn read_u32(bytes: &[u8], idx: &mut usize) -> Result<u32, String> {
    let raw = take_bytes(bytes, idx, 4)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_u16(bytes: &[u8], idx: &mut usize) -> Result<u16, String> {
    let raw = take_bytes(bytes, idx, 2)?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}
fn read_u64(bytes: &[u8], idx: &mut usize) -> Result<u64, String> {
    let raw = take_bytes(bytes, idx, 8)?;
    Ok(u64::from_le_bytes([
        raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
    ]))
}

fn validate_event(event: &InputEvent, previous_sequence: Option<u64>) -> Result<(), String> {
    if previous_sequence.is_some_and(|previous| event.sequence <= previous) {
        return Err("E_SAM_TAPE_FOOTER_INVALID 입력 사건 순번이 중복되거나 역전되었습니다".to_string());
    }
    if event.key_bit == 0
        || event.key_bit.count_ones() != 1
        || event.key_bit.trailing_zeros() as usize >= KEY_REGISTRY_KEYS.len()
    {
        return Err("E_SAM_TAPE_FOOTER_INVALID 입력 키 비트가 잘못되었습니다".to_string());
    }
    if event.key_bit == (1u16 << 6) {
        return Err("E_SAM_TAPE_HOST_KEY_FORBIDDEN Escape는 호스트 종료키이므로 기록할 수 없습니다".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        decode_input_tape, encode_input_tape, input_tape_from_masks, runtime_keys_to_tape_mask,
        tape_mask_to_runtime_keys, InputEvent, InputEventKind, InputRecord, InputTape,
    };
    use ddonirang_core::{KEY_A, KEY_D, KEY_S, KEY_W};

    fn sample() -> InputTape {
        input_tape_from_masks(60, &[0b10, 0]).expect("shared input tape")
    }

    fn v1_bytes(tape: &InputTape) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(super::MAGIC_V1);
        bytes.extend_from_slice(&super::VERSION_V1.to_le_bytes());
        bytes.extend_from_slice(&(super::KEY_REGISTRY_ID.len() as u32).to_le_bytes());
        bytes.extend_from_slice(super::KEY_REGISTRY_ID.as_bytes());
        bytes.extend_from_slice(&super::key_registry_hash());
        bytes.extend_from_slice(&tape.madi_hz.to_le_bytes());
        bytes.extend_from_slice(&(tape.records.len() as u32).to_le_bytes());
        for record in &tape.records {
            bytes.extend_from_slice(&record.madi.to_le_bytes());
            bytes.extend_from_slice(&(record.held_mask.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&record.held_mask);
        }
        bytes
    }

    #[test]
    fn shared_tape_roundtrip_is_byte_deterministic() {
        let tape = sample();
        assert_eq!(
            tape.records
                .iter()
                .map(|record| record.madi)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        let first = encode_input_tape(&tape).expect("encode");
        let second = encode_input_tape(&tape).expect("repeat encode");
        assert_eq!(first, second);
        assert_eq!(decode_input_tape(&first).expect("decode"), tape);
    }

    #[test]
    fn v1_is_an_explicit_read_compatibility_format_without_v2_event_claims() {
        let tape = sample();
        let decoded = decode_input_tape(&v1_bytes(&tape)).expect("read v1");
        assert_eq!(decoded.madi_hz, tape.madi_hz);
        assert_eq!(decoded.records.len(), tape.records.len());
        assert!(decoded
            .records
            .iter()
            .all(|record| record.events.is_empty()));
        assert!(encode_input_tape(&decoded)
            .expect("explicit V1 preservation")
            .starts_with(super::MAGIC_V1));
    }

    #[test]
    fn registry_hash_mutation_fails_closed() {
        let mut bytes = encode_input_tape(&sample()).expect("encode");
        let offset = b"DDN_INPUT_TAPE_V1\n".len()
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<u32>()
            + super::KEY_REGISTRY_ID.len();
        bytes[offset] ^= 1;
        let error = decode_input_tape(&bytes).expect_err("mutated registry must fail");
        assert_eq!(error, "키 레지스트리 해시가 다릅니다 (key registry hash mismatch)");
    }

    #[test]
    fn invalid_record_is_rejected_before_publication_bytes_exist() {
        let tape = InputTape {
            version: 1,
            madi_hz: 60,
            records: vec![InputRecord {
                madi: 0,
                held_mask: vec![0],
                events: Vec::new(),
            }],
        };
        let error = encode_input_tape(&tape).expect_err("invalid mask must fail");
        assert!(error.contains("held_mask 길이가 다릅니다"), "{error}");
        assert!(input_tape_from_masks(0, &[0]).is_err());
        assert!(input_tape_from_masks(60, &[1 << 9]).is_err());
    }

    #[test]
    fn tape_registry_mask_maps_to_shared_runtime_keys() {
        assert_eq!(tape_mask_to_runtime_keys(1 << 0), KEY_A);
        assert_eq!(tape_mask_to_runtime_keys(1 << 1), KEY_D);
        assert_eq!(tape_mask_to_runtime_keys(1 << 2), KEY_S);
        assert_eq!(tape_mask_to_runtime_keys(1 << 3), KEY_W);
        assert_eq!(
            tape_mask_to_runtime_keys((1 << 0) | (1 << 4)),
            KEY_A | (1 << 4)
        );
        for mask in [0, 1, 0b1010, 0b1_1111_1111] {
            assert_eq!(
                runtime_keys_to_tape_mask(tape_mask_to_runtime_keys(mask)),
                mask
            );
        }
    }

    #[test]
    fn v2_preserves_press_release_sequence_and_rejects_digest_tamper() {
        let tape = InputTape {
            version: 2,
            madi_hz: 60,
            records: vec![InputRecord {
                madi: 0,
                held_mask: super::mask_to_bytes(0),
                events: vec![
                    InputEvent {
                        sequence: 4,
                        key_bit: 1,
                        kind: InputEventKind::Press,
                    },
                    InputEvent {
                        sequence: 5,
                        key_bit: 1,
                        kind: InputEventKind::Release,
                    },
                ],
            }],
        };
        let bytes = encode_input_tape(&tape).expect("encode v2");
        assert_eq!(decode_input_tape(&bytes).expect("decode v2"), tape);
        let mut tampered = bytes.clone();
        let body_byte = tampered.len() - 35;
        tampered[body_byte] ^= 1;
        assert!(decode_input_tape(&tampered)
            .expect_err("body tamper")
            .contains("DIGEST_MISMATCH"));
        let mut footer_tampered = bytes;
        let last = footer_tampered.len() - 1;
        footer_tampered[last] ^= 1;
        assert!(decode_input_tape(&footer_tampered)
            .expect_err("footer tamper")
            .contains("DIGEST_MISMATCH"));
    }

    #[test]
    fn v2_rejects_host_stop_and_trailing_bytes() {
        let tape = InputTape {
            version: 2,
            madi_hz: 60,
            records: vec![InputRecord {
                madi: 0,
                held_mask: super::mask_to_bytes(0),
                events: vec![InputEvent {
                    sequence: 0,
                    key_bit: 1 << 6,
                    kind: InputEventKind::Press,
                }],
            }],
        };
        assert!(encode_input_tape(&tape)
            .expect_err("host key")
            .contains("HOST_KEY_FORBIDDEN"));
        let valid = InputTape { version: 2, madi_hz: 60, records: vec![InputRecord {
            madi: 0, held_mask: super::mask_to_bytes(0), events: Vec::new(),
        }] };
        let mut good = encode_input_tape(&valid).expect("encode");
        good.push(0);
        assert!(decode_input_tape(&good)
            .expect_err("trailing bytes")
            .contains("TRAILING_BYTES"));

        let mut truncated = encode_input_tape(&valid).expect("encode");
        truncated.truncate(truncated.len() - 1);
        assert!(decode_input_tape(&truncated)
            .expect_err("truncated")
            .contains("TRUNCATED"));
    }

    #[test]
    fn v2_footer_digest_corruption_is_rejected() {
        let tape = InputTape { version: 2, madi_hz: 60, records: Vec::new() };
        let mut bytes = encode_input_tape(&tape).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        assert!(decode_input_tape(&bytes).expect_err("corrupt footer cannot yield a tape").contains("DIGEST_MISMATCH"));
    }

    #[test]
    fn v2_control_registry_empty_tail_and_invalid_sequences() {
        let mut tape = InputTape { version: 2, madi_hz: 60, records: Vec::new() };
        let empty = encode_input_tape(&tape).unwrap();
        assert_eq!(decode_input_tape(&empty).unwrap(), tape);
        assert!(empty.windows(super::CONTROL_REGISTRY_ID.len()).any(|part| part == super::CONTROL_REGISTRY_ID.as_bytes()));
        assert_eq!(&empty[empty.len()-32..], blake3::hash(&empty[..empty.len()-32]).as_bytes());
        tape.records = vec![InputRecord { madi: 0, held_mask: super::mask_to_bytes(1 << 7), events: vec![InputEvent {
            sequence: 0, key_bit: 1 << 7, kind: InputEventKind::Press,
        }] }, InputRecord { madi: 1, held_mask: super::mask_to_bytes(1 << 7), events: Vec::new() }];
        let bytes = encode_input_tape(&tape).unwrap();
        assert_eq!(decode_input_tape(&bytes).unwrap(), tape);
        let mut unsupported = bytes.clone();
        let offset = super::MAGIC_V2.len();
        unsupported[offset..offset+4].copy_from_slice(&3u32.to_le_bytes());
        assert!(decode_input_tape(&unsupported).unwrap_err().contains("VERSION_UNSUPPORTED"));
        tape.records[1].events.push(InputEvent { sequence: 0, key_bit: 1 << 7, kind: InputEventKind::Press });
        assert!(encode_input_tape(&tape).unwrap_err().contains("입력 사건 순번이 중복되거나 역전되었습니다"));
        tape.records[1].events[0].sequence = 1;
        tape.records[1].events[0].kind = InputEventKind::Release;
        assert!(encode_input_tape(&tape).unwrap_err().contains("held mask"));
    }
}
