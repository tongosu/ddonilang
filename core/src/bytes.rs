//! RD-656 byte values: one immutable octet carrier and exact V1 storage owner.
//! Text conversion is explicit; source-file BOM admission is a different boundary.
use sha2::{Digest, Sha256};
use std::sync::Arc;
use unicode_normalization::UnicodeNormalization;

const MAGIC: &[u8; 10] = b"DDN-BYTES\0";
const HEADER_LENGTH: usize = 20;
pub const UNICODE_VERSION: (u8, u8, u8) = (17, 0, 0);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteSequence(Arc<[u8]>);

/// Product-local provenance, deliberately not a new serialized String format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NfcDecodeProvenance {
    pub unicode_version: (u8, u8, u8),
    pub source_bytes_storage_sha256: [u8; 32],
    pub decoded_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BytesError {
    Magic,
    Version,
    Length,
    Hash,
    Utf8,
    UnicodeVersion,
    Index,
}

impl std::fmt::Display for BytesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Magic => "E_BYTES_STORAGE_MAGIC: 바이트열 저장 표지가 올바르지 않습니다",
            Self::Version => "E_BYTES_STORAGE_VERSION: 지원하지 않는 바이트열 저장 판입니다",
            Self::Length => "E_BYTES_STORAGE_LENGTH: 바이트열 저장 길이가 일치하지 않습니다",
            Self::Hash => "E_BYTES_STORAGE_HASH: 바이트열 저장 해시가 일치하지 않습니다",
            Self::Utf8 => "E_BYTES_UTF8_INVALID: 완전한 UTF-8 바이트열이 아닙니다",
            Self::UnicodeVersion => {
                "E_UNICODE_DATA_VERSION_MISMATCH: Unicode 17.0.0 자료가 필요합니다"
            }
            Self::Index => "E_COLLECTION_INDEX_OUT_OF_RANGE: 바이트열 범위를 벗어났습니다",
        })
    }
}

impl std::error::Error for BytesError {}

impl ByteSequence {
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self(bytes.into())
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn encode_utf8_exact(text: &str) -> Self {
        Self::new(text.as_bytes())
    }

    pub fn decode_utf8_nfc(&self) -> Result<String, BytesError> {
        Ok(self.decode_utf8_nfc_with_provenance()?.decoded_text)
    }

    pub fn decode_utf8_nfc_with_provenance(&self) -> Result<NfcDecodeProvenance, BytesError> {
        if unicode_normalization::UNICODE_VERSION != UNICODE_VERSION {
            return Err(BytesError::UnicodeVersion);
        }
        let text = std::str::from_utf8(&self.0).map_err(|_| BytesError::Utf8)?;
        Ok(NfcDecodeProvenance {
            unicode_version: UNICODE_VERSION,
            source_bytes_storage_sha256: self.storage_sha256(),
            decoded_text: text.nfc().collect(),
        })
    }

    pub fn byte_at(&self, index: usize) -> Result<Self, BytesError> {
        let byte = *self.0.get(index).ok_or(BytesError::Index)?;
        Ok(Self::new(vec![byte]))
    }

    pub fn slice(&self, start: usize, end: usize) -> Result<Self, BytesError> {
        self.0
            .get(start..end)
            .map(Self::new)
            .ok_or(BytesError::Index)
    }

    pub fn storage_frame(&self) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.extend_from_slice(MAGIC);
        frame.extend_from_slice(&1u16.to_be_bytes());
        frame.extend_from_slice(&(self.len() as u64).to_be_bytes());
        frame.extend_from_slice(&self.0);
        frame
    }

    /// Exact size from the same frame owner, without allocating a frame.
    pub fn storage_frame_byte_length(&self) -> usize { HEADER_LENGTH + self.len() }

    pub fn from_storage_frame(frame: &[u8]) -> Result<Self, BytesError> {
        Ok(Self::new(Self::validated_payload(frame)?))
    }

    // Admission borrows the payload; no untrusted length drives an allocation.
    fn validated_payload(frame: &[u8]) -> Result<&[u8], BytesError> {
        if frame.get(..10) != Some(MAGIC.as_slice()) {
            return Err(BytesError::Magic);
        }
        let version = frame.get(10..12).ok_or(BytesError::Length)?;
        if version != 1u16.to_be_bytes() {
            return Err(BytesError::Version);
        }
        let length: [u8; 8] = frame
            .get(12..20)
            .ok_or(BytesError::Length)?
            .try_into()
            .map_err(|_| BytesError::Length)?;
        let length = usize::try_from(u64::from_be_bytes(length)).map_err(|_| BytesError::Length)?;
        let expected = HEADER_LENGTH
            .checked_add(length)
            .ok_or(BytesError::Length)?;
        if frame.len() != expected {
            return Err(BytesError::Length);
        }
        Ok(&frame[HEADER_LENGTH..])
    }

    pub fn content_sha256(&self) -> [u8; 32] {
        Sha256::digest(&self.0).into()
    }
    pub fn storage_sha256(&self) -> [u8; 32] {
        Sha256::digest(self.storage_frame()).into()
    }

    /// Mechanical RD-680 transport projection of the existing frame, not an
    /// alternate value/hash authority. Decoding belongs to the saved codec.
    pub fn storage_base64url(&self) -> String {
        const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let frame = self.storage_frame();
        let mut out = String::new();
        for chunk in frame.chunks(3) {
            let a = chunk[0];
            let b = chunk.get(1).copied().unwrap_or(0);
            let c = chunk.get(2).copied().unwrap_or(0);
            out.push(ALPHABET[(a >> 2) as usize] as char);
            out.push(ALPHABET[(((a & 3) << 4) | (b >> 4)) as usize] as char);
            if chunk.len() > 1 {
                out.push(ALPHABET[(((b & 15) << 2) | (c >> 6)) as usize] as char);
            }
            if chunk.len() > 2 {
                out.push(ALPHABET[(c & 63) as usize] as char);
            }
        }
        out
    }

    pub fn restore_verified(
        frame: &[u8],
        content: &[u8; 32],
        storage: &[u8; 32],
    ) -> Result<Self, BytesError> {
        let payload = Self::validated_payload(frame)?;
        let actual_content: [u8; 32] = Sha256::digest(payload).into();
        let actual_storage: [u8; 32] = Sha256::digest(frame).into();
        if &actual_content != content || &actual_storage != storage {
            return Err(BytesError::Hash);
        }
        // Materialize only after the complete frame and both domains agree.
        Ok(Self::new(payload))
    }

    pub fn canonical_literal(&self) -> String {
        let items: Vec<_> = self.0.iter().map(|byte| format!("0x{byte:02x}")).collect();
        format!("바이트열{{{}}}", items.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verified_restore_checks_both_hash_domains_before_materializing() {
        let value = ByteSequence::new(vec![0, 255]);
        let frame = value.storage_frame();
        let payload_hash = value.content_sha256();
        let frame_hash = value.storage_sha256();
        let payload = ByteSequence::validated_payload(&frame).unwrap();
        assert_eq!(payload.as_ptr(), frame[HEADER_LENGTH..].as_ptr());
        for (content, storage) in [
            ([0; 32], frame_hash),
            (payload_hash, [0; 32]),
            (frame_hash, payload_hash),
        ] {
            assert_eq!(ByteSequence::restore_verified(&frame, &content, &storage),
                Err(BytesError::Hash));
        }
        let mut unbounded = frame.clone();
        unbounded[12..20].copy_from_slice(&u64::MAX.to_be_bytes());
        assert_eq!(ByteSequence::restore_verified(&unbounded, &payload_hash, &frame_hash),
            Err(BytesError::Length));
        assert_eq!(ByteSequence::restore_verified(&frame, &payload_hash, &frame_hash),
            Ok(value));
    }

    #[test]
    fn byte_frame_exact_roundtrip_and_hash_domains() {
        let value = ByteSequence::new(vec![0, 255]);
        let mut expected = b"DDN-BYTES\0\0\x01\0\0\0\0\0\0\0\x02".to_vec();
        expected.extend([0, 255]);
        assert_eq!(value.storage_frame(), expected);
        assert_ne!(value.content_sha256(), value.storage_sha256());
        assert_eq!(
            ByteSequence::restore_verified(
                &expected,
                &value.content_sha256(),
                &value.storage_sha256()
            ),
            Ok(value.clone())
        );
        expected.push(0);
        assert_eq!(
            ByteSequence::from_storage_frame(&expected),
            Err(BytesError::Length)
        );
        for length in 0..value.storage_frame().len() {
            assert!(ByteSequence::from_storage_frame(&value.storage_frame()[..length]).is_err());
        }
        let mut wrong = value.storage_frame();
        wrong[11] = 2;
        assert_eq!(
            ByteSequence::from_storage_frame(&wrong),
            Err(BytesError::Version)
        );
        wrong = value.storage_frame();
        wrong[20] = 1;
        assert_eq!(
            ByteSequence::restore_verified(
                &wrong,
                &value.content_sha256(),
                &value.storage_sha256()
            ),
            Err(BytesError::Hash)
        );
    }

    #[test]
    fn utf8_is_explicit_strict_and_bom_is_value_data() {
        let nfd = "가";
        let bytes = ByteSequence::encode_utf8_exact(nfd);
        assert_eq!(bytes.as_slice(), nfd.as_bytes());
        assert_eq!(bytes.decode_utf8_nfc().unwrap(), "가");
        assert_eq!(
            ByteSequence::new(vec![0xef, 0xbb, 0xbf])
                .decode_utf8_nfc()
                .unwrap(),
            "\u{feff}"
        );
        for invalid in [
            vec![0xff],
            vec![0xc0, 0xaf],
            vec![0xed, 0xa0, 0x80],
            vec![0xe1, 0x84],
        ] {
            assert_eq!(
                ByteSequence::new(invalid).decode_utf8_nfc(),
                Err(BytesError::Utf8)
            );
        }
        let empty = ByteSequence::new(Vec::<u8>::new());
        assert_eq!(
            ByteSequence::from_storage_frame(&empty.storage_frame()),
            Ok(empty)
        );
    }

    #[test]
    fn bytes_ranges_are_octet_half_open_not_character_offsets() {
        let value = ByteSequence::encode_utf8_exact("A가😀");
        assert_eq!(value.len(), 8);
        assert_eq!(value.byte_at(7).unwrap().as_slice(), &[0x80]);
        assert_eq!(value.slice(1, 4).unwrap().as_slice(), "가".as_bytes());
        assert_eq!(value.slice(2, 3).unwrap().as_slice(), &[0xb0]);
        assert!(value.slice(8, 8).unwrap().is_empty());
        assert_eq!(value.byte_at(8), Err(BytesError::Index));
        assert_eq!(value.byte_at(usize::MAX), Err(BytesError::Index));
        assert_eq!(value.slice(4, 1), Err(BytesError::Index));
        assert_eq!(value.slice(0, 9), Err(BytesError::Index));
    }

    #[test]
    fn nfc_provenance_binds_exact_original_bytes_without_encode_normalization() {
        let decomposed = ByteSequence::encode_utf8_exact("가");
        let composed = ByteSequence::encode_utf8_exact("가");
        let left = decomposed.decode_utf8_nfc_with_provenance().unwrap();
        let right = composed.decode_utf8_nfc_with_provenance().unwrap();
        assert_eq!(left.decoded_text, right.decoded_text);
        assert_eq!(left.unicode_version, (17, 0, 0));
        assert_eq!(
            left.source_bytes_storage_sha256,
            decomposed.storage_sha256()
        );
        assert_eq!(right.source_bytes_storage_sha256, composed.storage_sha256());
        assert_ne!(
            left.source_bytes_storage_sha256,
            right.source_bytes_storage_sha256
        );
    }

    #[test]
    fn storage_length_overflow_and_both_hash_mismatches_are_rejected() {
        let value = ByteSequence::new(vec![0x00, 0xff]);
        let mut frame = value.storage_frame();
        frame[12..20].copy_from_slice(&u64::MAX.to_be_bytes());
        assert_eq!(
            ByteSequence::from_storage_frame(&frame),
            Err(BytesError::Length)
        );
        for (content, storage) in [
            ([0; 32], value.storage_sha256()),
            (value.content_sha256(), [0; 32]),
        ] {
            assert_eq!(
                ByteSequence::restore_verified(&value.storage_frame(), &content, &storage),
                Err(BytesError::Hash)
            );
        }
    }
}
