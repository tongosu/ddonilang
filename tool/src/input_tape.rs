#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use ddonirang_core::{KEY_A, KEY_D, KEY_S, KEY_W};

const MAGIC: &[u8] = b"DDN_INPUT_TAPE_V1\n";
const VERSION: u32 = 1;

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputTape {
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
        let idx = key_index(token).ok_or_else(|| format!("unknown key token: {token}"))?;
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
            "held_mask length mismatch: expected {needed}, got {}",
            bytes.len()
        ));
    }
    let mut raw = [0u8; 2];
    raw[..needed].copy_from_slice(bytes);
    Ok(u16::from_le_bytes(raw))
}

/// Builds the versioned input tape consumed by every product runtime frontdoor.
///
/// Interactive hosts may collect key masks themselves, but record ordering,
/// key-registry bounds and the tape identity stay in this shared authority.
pub fn input_tape_from_masks(madi_hz: u32, masks: &[u16]) -> Result<InputTape, String> {
    if madi_hz == 0 {
        return Err("madi_hz must be greater than zero".to_string());
    }
    if masks.len() > u32::MAX as usize {
        return Err("input tape record count exceeds u32".to_string());
    }
    let allowed_mask = (1u16 << KEY_REGISTRY_KEYS.len()) - 1;
    let mut records = Vec::with_capacity(masks.len());
    for (index, mask) in masks.iter().copied().enumerate() {
        if mask & !allowed_mask != 0 {
            return Err(format!(
                "input mask contains unregistered key bits: {:#x}",
                mask & !allowed_mask
            ));
        }
        records.push(InputRecord {
            madi: index as u32,
            held_mask: mask_to_bytes(mask),
        });
    }
    Ok(InputTape { madi_hz, records })
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
    for record in &tape.records {
        mask_from_bytes(&record.held_mask)?;
    }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    let registry_id = KEY_REGISTRY_ID.as_bytes();
    out.extend_from_slice(&(registry_id.len() as u32).to_le_bytes());
    out.extend_from_slice(registry_id);
    out.extend_from_slice(&key_registry_hash());
    out.extend_from_slice(&tape.madi_hz.to_le_bytes());
    out.extend_from_slice(&(tape.records.len() as u32).to_le_bytes());
    for record in &tape.records {
        out.extend_from_slice(&record.madi.to_le_bytes());
        out.extend_from_slice(&(record.held_mask.len() as u32).to_le_bytes());
        out.extend_from_slice(&record.held_mask);
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
    if take_bytes(bytes, &mut idx, MAGIC.len())? != MAGIC {
        return Err("invalid input tape magic".to_string());
    }
    let version = read_u32(bytes, &mut idx)?;
    if version != VERSION {
        return Err(format!("unsupported input tape version: {version}"));
    }
    let registry_len = read_u32(bytes, &mut idx)? as usize;
    if take_bytes(bytes, &mut idx, registry_len)? != KEY_REGISTRY_ID.as_bytes() {
        return Err("key registry id mismatch".to_string());
    }
    if take_bytes(bytes, &mut idx, 32)? != key_registry_hash() {
        return Err("key registry hash mismatch".to_string());
    }
    let madi_hz = read_u32(bytes, &mut idx)?;
    let record_count = read_u32(bytes, &mut idx)? as usize;
    let mut records = Vec::with_capacity(record_count);
    for _ in 0..record_count {
        let madi = read_u32(bytes, &mut idx)?;
        let mask_len = read_u32(bytes, &mut idx)? as usize;
        let held_mask = take_bytes(bytes, &mut idx, mask_len)?.to_vec();
        mask_from_bytes(&held_mask)?;
        records.push(InputRecord { madi, held_mask });
    }
    if idx != bytes.len() {
        return Err("extra bytes at end of input tape".to_string());
    }
    Ok(InputTape { madi_hz, records })
}

fn take_bytes<'a>(bytes: &'a [u8], idx: &mut usize, len: usize) -> Result<&'a [u8], String> {
    let end = idx.saturating_add(len);
    if end > bytes.len() {
        return Err("unexpected EOF while reading input tape".to_string());
    }
    let out = &bytes[*idx..end];
    *idx = end;
    Ok(out)
}

fn read_u32(bytes: &[u8], idx: &mut usize) -> Result<u32, String> {
    let raw = take_bytes(bytes, idx, 4)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

#[cfg(test)]
mod tests {
    use super::{
        decode_input_tape, encode_input_tape, input_tape_from_masks, runtime_keys_to_tape_mask,
        tape_mask_to_runtime_keys, InputRecord, InputTape,
    };
    use ddonirang_core::{KEY_A, KEY_D, KEY_S, KEY_W};

    fn sample() -> InputTape {
        input_tape_from_masks(60, &[0b10, 0]).expect("shared input tape")
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
    fn registry_hash_mutation_fails_closed() {
        let mut bytes = encode_input_tape(&sample()).expect("encode");
        let offset = b"DDN_INPUT_TAPE_V1\n".len()
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<u32>()
            + super::KEY_REGISTRY_ID.len();
        bytes[offset] ^= 1;
        let error = decode_input_tape(&bytes).expect_err("mutated registry must fail");
        assert_eq!(error, "key registry hash mismatch");
    }

    #[test]
    fn invalid_record_is_rejected_before_publication_bytes_exist() {
        let tape = InputTape {
            madi_hz: 60,
            records: vec![InputRecord {
                madi: 0,
                held_mask: vec![0],
            }],
        };
        let error = encode_input_tape(&tape).expect_err("invalid mask must fail");
        assert!(error.contains("held_mask length mismatch"), "{error}");
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
}
