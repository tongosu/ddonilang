use std::fs;
use std::path::Path;

use crate::artifact_output::write_binary_artifact_atomic;

const MAGIC: &[u8; 4] = b"BDLP";
const VERSION: u32 = 1;
const CODEC_BDL1: &[u8; 4] = b"BDL1";

#[derive(Debug)]
pub enum BdlPacketError {
    InvalidMagic,
    UnsupportedVersion { version: u32 },
    UnsupportedCodec { codec: [u8; 4] },
    LengthMismatch { expected: u32, actual: u32 },
    HashMismatch,
    Truncated,
    PayloadTooLarge { len: usize },
}

impl BdlPacketError {
    pub fn code(&self) -> &'static str {
        match self {
            BdlPacketError::PayloadTooLarge { .. } => "E_BDL1_PACKET_TOO_LARGE",
            _ => "E_BDL1_PACKET_INVALID",
        }
    }

    pub fn message(&self) -> String {
        match self {
            BdlPacketError::InvalidMagic => "packet magic이 BDLP가 아님".to_string(),
            BdlPacketError::UnsupportedVersion { version } => {
                format!("지원하지 않는 packet 버전: {version}")
            }
            BdlPacketError::UnsupportedCodec { codec } => {
                format!("지원하지 않는 codec: {}", String::from_utf8_lossy(codec))
            }
            BdlPacketError::LengthMismatch { expected, actual } => {
                format!("payload 길이 불일치 expected={expected} actual={actual}")
            }
            BdlPacketError::HashMismatch => "payload_hash 불일치".to_string(),
            BdlPacketError::Truncated => "packet 길이가 부족함".to_string(),
            BdlPacketError::PayloadTooLarge { len } => {
                format!("payload 길이가 u32 범위를 초과: {len}")
            }
        }
    }
}

pub struct BdlPacketInfo {
    pub payload_hash: [u8; 32],
}

pub fn encode_bdl1_packet(payload: &[u8]) -> Result<Vec<u8>, BdlPacketError> {
    let len = u32::try_from(payload.len())
        .map_err(|_| BdlPacketError::PayloadTooLarge { len: payload.len() })?;
    let mut out = Vec::with_capacity(4 + 4 + 4 + 4 + 32 + payload.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(CODEC_BDL1);
    out.extend_from_slice(&len.to_le_bytes());
    let hash = blake3::hash(payload);
    out.extend_from_slice(hash.as_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

pub fn decode_bdl1_packet(bytes: &[u8]) -> Result<(Vec<u8>, BdlPacketInfo), BdlPacketError> {
    let mut idx = 0usize;
    let magic = take(bytes, &mut idx, 4)?;
    if magic != MAGIC {
        return Err(BdlPacketError::InvalidMagic);
    }
    let version = read_u32(bytes, &mut idx)?;
    if version != VERSION {
        return Err(BdlPacketError::UnsupportedVersion { version });
    }
    let codec = take(bytes, &mut idx, 4)?;
    if codec != CODEC_BDL1 {
        return Err(BdlPacketError::UnsupportedCodec {
            codec: [codec[0], codec[1], codec[2], codec[3]],
        });
    }
    let len = read_u32(bytes, &mut idx)?;
    let hash = take(bytes, &mut idx, 32)?;
    let payload = take(bytes, &mut idx, len as usize)?;
    if idx != bytes.len() {
        return Err(BdlPacketError::LengthMismatch {
            expected: len,
            actual: payload.len() as u32,
        });
    }
    let expected = blake3::hash(payload);
    if expected.as_bytes() != hash {
        return Err(BdlPacketError::HashMismatch);
    }
    let mut hash_out = [0u8; 32];
    hash_out.copy_from_slice(hash);
    Ok((
        payload.to_vec(),
        BdlPacketInfo {
            payload_hash: hash_out,
        },
    ))
}

pub fn payload_hash_string(hash: &[u8; 32]) -> String {
    format!("blake3:{}", blake3::Hash::from_bytes(*hash).to_hex())
}

pub fn wrap_packet(input: &Path, out: &Path) -> Result<(), String> {
    require_file_target(out)?;
    let payload =
        fs::read(input).map_err(|error| format_error(input, BdlPacketError::Truncated, error))?;
    if payload.len() < 4 || &payload[..4] != CODEC_BDL1 {
        return Err(format!(
            "{} {}:1:1 입력이 BDL1 detbin이 아닙니다.",
            BdlPacketError::InvalidMagic.code(),
            input.display()
        ));
    }
    let packet = encode_bdl1_packet(&payload).map_err(|error| format_error(input, error, ""))?;
    write_binary_artifact_atomic(out, &packet)?;
    println!(
        "payload_hash={}",
        payload_hash_string(blake3::hash(&payload).as_bytes())
    );
    Ok(())
}

pub fn unwrap_packet(input: &Path, out: &Path) -> Result<(), String> {
    require_file_target(out)?;
    let bytes =
        fs::read(input).map_err(|error| format_error(input, BdlPacketError::Truncated, error))?;
    let (payload, info) =
        decode_bdl1_packet(&bytes).map_err(|error| format_error(input, error, ""))?;
    write_binary_artifact_atomic(out, &payload)?;
    println!("payload_hash={}", payload_hash_string(&info.payload_hash));
    Ok(())
}

fn require_file_target(path: &Path) -> Result<(), String> {
    if path.exists() && !path.is_file() {
        return Err(format!(
            "E_BDL_PACKET_OUTPUT_TARGET_NOT_FILE {}",
            path.display()
        ));
    }
    Ok(())
}

fn format_error(path: &Path, error: BdlPacketError, suffix: impl std::fmt::Display) -> String {
    let suffix = suffix.to_string();
    if suffix.is_empty() {
        format!(
            "{} {}:1:1 {}",
            error.code(),
            path.display(),
            error.message()
        )
    } else {
        format!(
            "{} {}:1:1 {} {}",
            error.code(),
            path.display(),
            error.message(),
            suffix
        )
    }
}

fn take<'a>(bytes: &'a [u8], idx: &mut usize, len: usize) -> Result<&'a [u8], BdlPacketError> {
    let end = idx.saturating_add(len);
    if end > bytes.len() {
        return Err(BdlPacketError::Truncated);
    }
    let out = &bytes[*idx..end];
    *idx = end;
    Ok(out)
}

fn read_u32(bytes: &[u8], idx: &mut usize) -> Result<u32, BdlPacketError> {
    let raw = take(bytes, idx, 4)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

#[cfg(test)]
mod tests {
    use super::{decode_bdl1_packet, encode_bdl1_packet, unwrap_packet, wrap_packet};
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("ddn_bdl_packet_{label}_{}_{nonce}", process::id()))
    }

    #[test]
    fn codec_roundtrip_preserves_payload() {
        let payload = b"BDL1\x00\x01product";
        let packet = encode_bdl1_packet(payload).expect("encode packet");
        let (decoded, _) = decode_bdl1_packet(&packet).expect("decode packet");
        assert_eq!(decoded, payload);
    }

    #[test]
    fn file_roundtrip_is_atomic_and_rejects_directory_target() {
        let dir = test_dir("workflow");
        fs::create_dir_all(&dir).expect("create test dir");
        let input = dir.join("input.bdl1.detbin");
        let packet = dir.join("input.bdlp");
        let output = dir.join("output.bdl1.detbin");
        let blocked = dir.join("blocked.bdlp");
        fs::write(&input, b"BDL1\x00\x01product").expect("write input");
        fs::create_dir_all(&blocked).expect("create blocked target");

        wrap_packet(&input, &packet).expect("wrap packet");
        unwrap_packet(&packet, &output).expect("unwrap packet");
        assert_eq!(
            fs::read(&output).expect("read output"),
            fs::read(&input).expect("read input")
        );
        let error = wrap_packet(&input, &blocked).expect_err("directory target must fail closed");
        assert!(error.contains("E_BDL_PACKET_OUTPUT_TARGET_NOT_FILE"));
        assert_eq!(
            fs::read_dir(&blocked).expect("read blocked target").count(),
            0
        );

        fs::remove_dir_all(&dir).expect("remove test dir");
    }
}
