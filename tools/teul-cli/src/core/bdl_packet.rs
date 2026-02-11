use blake3;

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
                format!("지원하지 않는 packet 버전: {}", version)
            }
            BdlPacketError::UnsupportedCodec { codec } => {
                format!("지원하지 않는 codec: {}", String::from_utf8_lossy(codec))
            }
            BdlPacketError::LengthMismatch { expected, actual } => {
                format!("payload 길이 불일치 expected={} actual={}", expected, actual)
            }
            BdlPacketError::HashMismatch => "payload_hash 불일치".to_string(),
            BdlPacketError::Truncated => "packet 길이가 부족함".to_string(),
            BdlPacketError::PayloadTooLarge { len } => {
                format!("payload 길이가 u32 범위를 초과: {}", len)
            }
        }
    }
}

pub struct BdlPacketInfo {
    pub payload_hash: [u8; 32],
}

pub fn encode_bdl1_packet(payload: &[u8]) -> Result<Vec<u8>, BdlPacketError> {
    let len = u32::try_from(payload.len()).map_err(|_| BdlPacketError::PayloadTooLarge {
        len: payload.len(),
    })?;
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
            actual: (payload.len()) as u32,
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
    let hash = blake3::Hash::from_bytes(*hash);
    format!("blake3:{}", hash.to_hex())
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
