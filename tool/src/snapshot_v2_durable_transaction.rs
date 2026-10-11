//! RD-669: shared private transaction authority, independent of storage mechanics.
//! No callback, transport status, or visible head is a committed receipt.

use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub(crate) enum SnapshotV2DurabilityFailureV1 {
    InvalidInput = 1,
    LimitExceeded,
    SnapshotHashMismatch,
    AtomEvidenceInvalid,
    SubjectCustodyInvalid,
    PesiMismatch,
    ForeignPesi,
    StaleRevision,
    CustodyConflict,
    ConflictingRetry,
    UnsupportedDurability,
    WriteFailed,
    ShortWrite,
    SyncFailed,
    RereadFailed,
    RereadMismatch,
    AtomicCommitFailed,
    ContainerDurabilityFailed,
    RevisionCommitFailed,
    AdapterAborted,
    Interrupted,
    CorruptRecoveryRecord,
    RecoveryRequired,
    ArtifactMismatch,
    RevisionEnvelopeRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum SnapshotV2TransactionStateV1 {
    Prepared = 1,
    DurablyWritten,
    Verified,
    Committed,
    Aborted,
    RecoveryRequired,
}

impl SnapshotV2TransactionStateV1 {
    // Graph admission only. Evidence admission is performed separately by the owner.
    fn permits(self, next: Self) -> bool {
        use SnapshotV2TransactionStateV1::*;
        matches!(
            (self, next),
            (Prepared, DurablyWritten | Aborted | RecoveryRequired)
                | (DurablyWritten, Verified | Aborted | RecoveryRequired)
                | (Verified, Committed | Aborted | RecoveryRequired)
                | (Committed, Committed)
                | (Aborted, Aborted)
                | (RecoveryRequired, RecoveryRequired | Aborted | Committed)
        )
    }
}

/// Already supplied constituent bytes. This does not issue any semantic identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransactionBindingV1 {
    pub project: Vec<u8>,
    pub expected_parent: Vec<u8>,
    pub new_revision: Vec<u8>,
    pub execution_profile: Vec<u8>,
    pub pesi: [u8; 32],
    pub snapshot_sha256: [u8; 32],
    pub snapshot_length: u64,
    pub revision_integrity: [u8; 32],
}

impl TransactionBindingV1 {
    fn preimage(&self) -> Result<Vec<u8>, SnapshotV2DurabilityFailureV1> {
        use SnapshotV2DurabilityFailureV1::*;
        if self.project.is_empty()
            || self.new_revision.is_empty()
            || self.execution_profile.is_empty()
            || self.snapshot_length == 0
        {
            return Err(InvalidInput);
        }
        let length = self.snapshot_length.to_be_bytes();
        let fields: [&[u8]; 8] = [
            &self.project,
            &self.expected_parent,
            &self.new_revision,
            &self.execution_profile,
            &self.pesi,
            &self.snapshot_sha256,
            &length,
            &self.revision_integrity,
        ];
        let mut total = b"DDN-VC04-SNAPSHOT-V2-DURABLE-TRANSACTION-V1\0".len() + 3;
        for field in fields {
            u32::try_from(field.len()).map_err(|_| LimitExceeded)?;
            total = total
                .checked_add(5)
                .and_then(|n| n.checked_add(field.len()))
                .ok_or(LimitExceeded)?;
        }
        let mut out = Vec::new();
        out.try_reserve_exact(total).map_err(|_| LimitExceeded)?;
        out.extend_from_slice(b"DDN-VC04-SNAPSHOT-V2-DURABLE-TRANSACTION-V1\0");
        out.extend_from_slice(&1u16.to_be_bytes());
        out.push(8);
        for (index, field) in fields.into_iter().enumerate() {
            out.push(index as u8 + 1);
            out.extend_from_slice(&(field.len() as u32).to_be_bytes());
            out.extend_from_slice(field);
        }
        Ok(out)
    }

    pub(crate) fn transaction_digest(&self) -> Result<[u8; 32], SnapshotV2DurabilityFailureV1> {
        Ok(Sha256::digest(self.preimage()?).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex32(s: &str) -> [u8; 32] {
        std::array::from_fn(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
    }

    fn binding() -> TransactionBindingV1 {
        TransactionBindingV1 {
            project: (0x80..=0x9f).collect(),
            expected_parent: (0xa0..=0xbf).collect(),
            new_revision: (0xc0..=0xdf).collect(),
            execution_profile: b"ddn.execution.profile.test.v1".to_vec(),
            pesi: hex32("9497bcb802baeab1610d3de5502568f892625391141ac3f5265b5522d635fad4"),
            snapshot_sha256: hex32(
                "9438e0e11ead728b23a04328c64deecd16ebf14a6f5b9b63aae73f0f0f00303f",
            ),
            snapshot_length: 614,
            revision_integrity: std::array::from_fn(|i| 0xe0 + i as u8),
        }
    }

    #[test]
    fn rd669_exact_corrected_transaction_vector() {
        let b = binding();
        assert_eq!(b.preimage().unwrap().len(), 316);
        assert_eq!(
            b.transaction_digest().unwrap(),
            hex32("fb6c08f374f5594b78c86e174e0777f488f27ff2a7c9ddc5e75b0e8cab55aed6")
        );
        assert_eq!(b.transaction_digest(), b.transaction_digest());
    }

    #[test]
    fn rd669_revision_changes_transaction_but_not_pesi() {
        let b = binding();
        let mut next = b.clone();
        next.new_revision[0] ^= 1;
        assert_ne!(b.transaction_digest(), next.transaction_digest());
        assert_eq!(b.pesi, next.pesi);
        next.expected_parent.clear(); // Only INITIAL parent may be empty.
        assert!(next.transaction_digest().is_ok());
        next.project.clear();
        assert_eq!(
            next.transaction_digest(),
            Err(SnapshotV2DurabilityFailureV1::InvalidInput)
        );
    }

    #[test]
    fn rd669_exact_state_graph_has_no_recovery_to_verified() {
        use SnapshotV2TransactionStateV1::*;
        let states = [
            Prepared,
            DurablyWritten,
            Verified,
            Committed,
            Aborted,
            RecoveryRequired,
        ];
        let allowed = [
            vec![1, 4, 5],
            vec![2, 4, 5],
            vec![3, 4, 5],
            vec![3],
            vec![4],
            vec![3, 4, 5],
        ];
        for (i, from) in states.iter().enumerate() {
            for (j, to) in states.iter().enumerate() {
                assert_eq!(
                    from.permits(*to),
                    allowed[i].contains(&j),
                    "{from:?} -> {to:?}"
                );
            }
        }
        assert_eq!(SnapshotV2DurabilityFailureV1::ArtifactMismatch as u16, 24);
        assert_eq!(
            SnapshotV2DurabilityFailureV1::RevisionEnvelopeRequired as u16,
            25
        );
    }
}
