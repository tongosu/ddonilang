use std::fs;
use std::path::Path;

use blake3;

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

#[derive(Clone, Debug)]
pub struct InputRecord {
    pub madi: u32,
    pub held_mask: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct InputTape {
    pub madi_hz: u32,
    pub records: Vec<InputRecord>,
}

pub fn key_registry_string() -> String {
    KEY_REGISTRY_KEYS.join("\n")
}

pub fn key_registry_hash() -> [u8; 32] {
    let digest = blake3::hash(key_registry_string().as_bytes());
    *digest.as_bytes()
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
        let idx = key_index(token).ok_or_else(|| format!("unknown key token: {}", token))?;
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
            "held_mask length mismatch: expected {}, got {}",
            needed,
            bytes.len()
        ));
    }
    let mut raw = [0u8; 2];
    raw[..needed].copy_from_slice(bytes);
    Ok(u16::from_le_bytes(raw))
}

pub fn write_input_tape(path: &Path, tape: &InputTape) -> Result<(), String> {
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

    fs::write(path, out).map_err(|e| e.to_string())
}

pub fn read_input_tape(path: &Path) -> Result<InputTape, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let mut idx = 0usize;

    let magic = take_bytes(&bytes, &mut idx, MAGIC.len())?;
    if magic != MAGIC {
        return Err("invalid input tape magic".to_string());
    }
    let version = read_u32(&bytes, &mut idx)?;
    if version != VERSION {
        return Err(format!("unsupported input tape version: {}", version));
    }

    let registry_len = read_u32(&bytes, &mut idx)? as usize;
    let registry_id = take_bytes(&bytes, &mut idx, registry_len)?;
    if registry_id != KEY_REGISTRY_ID.as_bytes() {
        return Err("key registry id mismatch".to_string());
    }

    let registry_hash = take_bytes(&bytes, &mut idx, 32)?;
    if registry_hash != key_registry_hash() {
        return Err("key registry hash mismatch".to_string());
    }

    let madi_hz = read_u32(&bytes, &mut idx)?;
    let record_count = read_u32(&bytes, &mut idx)? as usize;

    let mut records = Vec::with_capacity(record_count);
    for _ in 0..record_count {
        let madi = read_u32(&bytes, &mut idx)?;
        let mask_len = read_u32(&bytes, &mut idx)? as usize;
        let mask = take_bytes(&bytes, &mut idx, mask_len)?.to_vec();
        if mask_len != expected_mask_len() {
            return Err(format!(
                "held_mask length mismatch: expected {}, got {}",
                expected_mask_len(),
                mask_len
            ));
        }
        records.push(InputRecord {
            madi,
            held_mask: mask,
        });
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
