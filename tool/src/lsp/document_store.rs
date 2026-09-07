use super::position::{range_to_byte_range, ByteRange, LspRange, PositionEncoding, PositionError};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceSnapshotIdentity(String);

impl SourceSnapshotIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn derive(uri: &str, version: i64, bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"ddonirang.lsp0.source-snapshot.v1\0");
        hasher.update((uri.len() as u64).to_le_bytes());
        hasher.update(uri.as_bytes());
        hasher.update(version.to_le_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
        Self(format!("sha256:{:x}", hasher.finalize()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSnapshot {
    pub uri: String,
    pub language_id: String,
    pub version: i64,
    pub bytes: Vec<u8>,
    pub identity: SourceSnapshotIdentity,
}

impl DocumentSnapshot {
    pub fn text(&self) -> Result<&str, DocumentStoreError> {
        std::str::from_utf8(&self.bytes).map_err(|_| DocumentStoreError::InvalidUtf8)
    }

    pub fn byte_range_is_valid(&self, range: ByteRange) -> bool {
        range.start <= range.end
            && range.end <= self.bytes.len()
            && self
                .text()
                .map(|text| text.is_char_boundary(range.start) && text.is_char_boundary(range.end))
                .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentChange {
    pub range: Option<LspRange>,
    pub text: String,
    pub range_snapshot: Option<SourceSnapshotIdentity>,
}

impl DocumentChange {
    pub fn full(text: impl Into<String>) -> Self {
        Self {
            range: None,
            text: text.into(),
            range_snapshot: None,
        }
    }

    pub fn ranged(range: LspRange, text: impl Into<String>) -> Self {
        Self {
            range: Some(range),
            text: text.into(),
            range_snapshot: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentStoreError {
    InvalidUri,
    InvalidUtf8,
    DuplicateOpen,
    NotOpen,
    VersionNotMonotonic,
    SnapshotMismatch,
    InvalidRange(PositionError),
}

impl fmt::Display for DocumentStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUri => f.write_str("URI가 비어 있습니다"),
            Self::InvalidUtf8 => f.write_str("문서 bytes가 UTF-8이 아닙니다"),
            Self::DuplicateOpen => f.write_str("문서가 이미 열려 있습니다"),
            Self::NotOpen => f.write_str("문서가 열려 있지 않습니다"),
            Self::VersionNotMonotonic => f.write_str("문서 version이 증가하지 않았습니다"),
            Self::SnapshotMismatch => f.write_str("문서 snapshot identity가 다릅니다"),
            Self::InvalidRange(error) => write!(f, "문서 변경 range가 잘못되었습니다: {error}"),
        }
    }
}

impl std::error::Error for DocumentStoreError {}

#[derive(Debug, Default, Clone)]
pub struct DocumentStore {
    documents: BTreeMap<String, DocumentSnapshot>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(
        &mut self,
        uri: impl Into<String>,
        language_id: impl Into<String>,
        version: i64,
        bytes: Vec<u8>,
    ) -> Result<SourceSnapshotIdentity, DocumentStoreError> {
        let uri = uri.into();
        if uri.is_empty() {
            return Err(DocumentStoreError::InvalidUri);
        }
        if self.documents.contains_key(&uri) {
            return Err(DocumentStoreError::DuplicateOpen);
        }
        std::str::from_utf8(&bytes).map_err(|_| DocumentStoreError::InvalidUtf8)?;
        let identity = SourceSnapshotIdentity::derive(&uri, version, &bytes);
        self.documents.insert(
            uri.clone(),
            DocumentSnapshot {
                uri,
                language_id: language_id.into(),
                version,
                bytes,
                identity: identity.clone(),
            },
        );
        Ok(identity)
    }

    pub fn apply_changes(
        &mut self,
        uri: &str,
        version: i64,
        changes: &[DocumentChange],
        encoding: PositionEncoding,
    ) -> Result<SourceSnapshotIdentity, DocumentStoreError> {
        let current = self.documents.get(uri).ok_or(DocumentStoreError::NotOpen)?;
        if version <= current.version {
            return Err(DocumentStoreError::VersionNotMonotonic);
        }
        let original_identity = current.identity.clone();
        let mut bytes = current.bytes.clone();
        for change in changes {
            if let Some(expected) = &change.range_snapshot {
                if expected != &original_identity {
                    return Err(DocumentStoreError::SnapshotMismatch);
                }
            }
            let range = match change.range {
                Some(range) => {
                    let text =
                        std::str::from_utf8(&bytes).map_err(|_| DocumentStoreError::InvalidUtf8)?;
                    range_to_byte_range(text, range, encoding)
                        .map_err(DocumentStoreError::InvalidRange)?
                }
                None => ByteRange {
                    start: 0,
                    end: bytes.len(),
                },
            };
            if range.start > range.end || range.end > bytes.len() {
                return Err(DocumentStoreError::InvalidRange(
                    PositionError::ByteOutOfRange,
                ));
            }
            if std::str::from_utf8(&bytes)
                .map_err(|_| DocumentStoreError::InvalidUtf8)?
                .is_char_boundary(range.start)
                == false
                || std::str::from_utf8(&bytes)
                    .map_err(|_| DocumentStoreError::InvalidUtf8)?
                    .is_char_boundary(range.end)
                    == false
            {
                return Err(DocumentStoreError::InvalidRange(
                    PositionError::ContinuationByte,
                ));
            }
            bytes.splice(
                range.start..range.end,
                change.text.as_bytes().iter().copied(),
            );
        }
        std::str::from_utf8(&bytes).map_err(|_| DocumentStoreError::InvalidUtf8)?;
        let identity = SourceSnapshotIdentity::derive(uri, version, &bytes);
        let language_id = current.language_id.clone();
        self.documents.insert(
            uri.to_string(),
            DocumentSnapshot {
                uri: uri.to_string(),
                language_id,
                version,
                bytes,
                identity: identity.clone(),
            },
        );
        Ok(identity)
    }

    pub fn close(&mut self, uri: &str) -> Result<DocumentSnapshot, DocumentStoreError> {
        self.documents
            .remove(uri)
            .ok_or(DocumentStoreError::NotOpen)
    }

    pub fn get(&self, uri: &str) -> Option<&DocumentSnapshot> {
        self.documents.get(uri)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &DocumentSnapshot)> {
        self.documents
            .iter()
            .map(|(uri, snapshot)| (uri.as_str(), snapshot))
    }

    pub fn is_open(&self, uri: &str) -> bool {
        self.documents.contains_key(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::position::{LspPosition, LspRange};

    #[test]
    fn lifecycle_and_identity_are_deterministic() {
        let mut store = DocumentStore::new();
        let first = store
            .open("file:///a.ddn", "ddn", 1, "가".as_bytes().to_vec())
            .unwrap();
        assert_eq!(
            first,
            SourceSnapshotIdentity::derive("file:///a.ddn", 1, "가".as_bytes())
        );
        assert!(matches!(
            store.open("file:///a.ddn", "ddn", 1, b"x".to_vec()),
            Err(DocumentStoreError::DuplicateOpen)
        ));
        let next = store
            .apply_changes(
                "file:///a.ddn",
                2,
                &[DocumentChange::ranged(
                    LspRange {
                        start: LspPosition {
                            line: 0,
                            character: 0,
                        },
                        end: LspPosition {
                            line: 0,
                            character: 3,
                        },
                    },
                    "나",
                )],
                PositionEncoding::Utf8,
            )
            .unwrap();
        assert_ne!(first, next);
        assert_eq!(store.get("file:///a.ddn").unwrap().text().unwrap(), "나");
    }

    #[test]
    fn failed_batch_does_not_partially_mutate_document() {
        let mut store = DocumentStore::new();
        store
            .open("file:///a.ddn", "ddn", 1, b"abc".to_vec())
            .unwrap();
        let result = store.apply_changes(
            "file:///a.ddn",
            2,
            &[
                DocumentChange::full("first"),
                DocumentChange {
                    range: Some(LspRange {
                        start: LspPosition {
                            line: 99,
                            character: 0,
                        },
                        end: LspPosition {
                            line: 99,
                            character: 1,
                        },
                    }),
                    text: "second".into(),
                    range_snapshot: None,
                },
            ],
            PositionEncoding::Utf8,
        );
        assert!(result.is_err());
        let snapshot = store.get("file:///a.ddn").unwrap();
        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.text().unwrap(), "abc");
    }
}
