//! RD-662: recorded registered-atom contracts, shared by all execution surfaces.
use sha2::{Digest, Sha256};

pub(crate) mod source_binding;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContractFailure {
    ByteLengthMismatch,
    ShaMismatch,
    SchemaInvalid,
    OwnerMismatch,
    SnapshotMismatch,
    NotAdmitted,
}

impl ContractFailure {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::ByteLengthMismatch => "REGISTERED_ATOM_CONTRACT_BYTE_LENGTH_MISMATCH",
            Self::ShaMismatch => "REGISTERED_ATOM_CONTRACT_SHA_MISMATCH",
            Self::SchemaInvalid => "REGISTERED_ATOM_CONTRACT_SCHEMA_INVALID",
            Self::SnapshotMismatch => "REGISTERED_ATOM_CONTRACT_SNAPSHOT_MISMATCH",
            Self::OwnerMismatch | Self::NotAdmitted => "REGISTERED_ATOM_NOT_ADMITTED",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ContractCustody {
    pub byte_length: u64,
    pub sha256: [u8; 32],
    pub snapshot_identity: [u8; 32],
}

#[derive(Clone, Debug)]
pub(crate) struct RecordedAtomContract {
    owner: Vec<u8>,
    atoms: Vec<Vec<u8>>,
    custody: ContractCustody,
    // Preserve the exact admitted artifact, including its terminal LF. A
    // later capture must not regenerate this document from current values.
    recorded_bytes: Vec<u8>,
}

impl RecordedAtomContract {
    /// Records already-issued identities; never derives identities from names.
    pub(crate) fn record(
        owner: &[u8],
        atoms: &[Vec<u8>],
    ) -> Result<(Vec<u8>, Self), ContractFailure> {
        if owner.is_empty()
            || owner.len() > 4096
            || atoms.len() > 65536
            || atoms.iter().any(|atom| atom.len() > 4096)
            || atoms.iter().any(Vec::is_empty)
            || atoms.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(ContractFailure::SchemaInvalid);
        }
        // RD-668: determine the whole artifact budget before allocating or
        // hex-expanding it. Per-identity bounds alone can still allow 268 MB.
        let mut encoded_length = b"{\"atoms\":[".len()
            + b"],\"owner_registry_identity_hex\":\"".len()
            + owner.len() * 2
            + b"\",\"schema\":\"ddn.registered-atom-contract.v1\",\"version\":1}\n".len();
        for (index, atom) in atoms.iter().enumerate() {
            encoded_length = encoded_length.checked_add(
                b"{\"registered_atom_identity_hex\":\"".len()
                + atom.len() * 2 + b"\",\"status\":\"ADMITTED\"}".len()
                + usize::from(index != 0),
            ).ok_or(ContractFailure::SchemaInvalid)?;
            if encoded_length > 1_000_000 {
                return Err(ContractFailure::SchemaInvalid);
            }
        }
        fn append_hex(out: &mut Vec<u8>, bytes: &[u8]) {
            const HEX: &[u8; 16] = b"0123456789abcdef";
            for byte in bytes {
                out.push(HEX[(byte >> 4) as usize]);
                out.push(HEX[(byte & 15) as usize]);
            }
        }
        let mut raw = Vec::new();
        raw.try_reserve_exact(encoded_length).map_err(|_| ContractFailure::SchemaInvalid)?;
        raw.extend_from_slice(b"{\"atoms\":[");
        for (index, atom) in atoms.iter().enumerate() {
            if index != 0 {
                raw.push(b',');
            }
            raw.extend_from_slice(b"{\"registered_atom_identity_hex\":\"");
            append_hex(&mut raw, atom);
            raw.extend_from_slice(b"\",\"status\":\"ADMITTED\"}");
        }
        raw.extend_from_slice(b"],\"owner_registry_identity_hex\":\"");
        append_hex(&mut raw, owner);
        raw.extend_from_slice(
            b"\",\"schema\":\"ddn.registered-atom-contract.v1\",\"version\":1}\n",
        );
        let byte_length =
            u64::try_from(raw.len()).map_err(|_| ContractFailure::ByteLengthMismatch)?;
        let mut snapshot = Sha256::new();
        snapshot.update(b"DDN-REGISTERED-ATOM-CONTRACT-SNAPSHOT-V1\0");
        snapshot.update(1u16.to_le_bytes());
        snapshot.update(byte_length.to_le_bytes());
        snapshot.update(&raw);
        let custody = ContractCustody {
            byte_length,
            sha256: Sha256::digest(&raw).into(),
            snapshot_identity: snapshot.finalize().into(),
        };
        let contract = Self::restore(&raw, owner, custody)?;
        Ok((raw, contract))
    }

    pub(crate) fn restore(
        raw: &[u8],
        expected_owner: &[u8],
        custody: ContractCustody,
    ) -> Result<Self, ContractFailure> {
        Self::restore_inner(raw,Some(expected_owner),custody)
    }

    /// The complete-state owner supplies custody, while the exact artifact
    /// itself supplies its registry identity. This never consults a live registry.
    pub(crate) fn restore_recorded(raw:&[u8],custody:ContractCustody)->Result<Self,ContractFailure> {
        Self::restore_inner(raw,None,custody)
    }

    fn restore_inner(raw:&[u8],expected_owner:Option<&[u8]>,custody:ContractCustody)->Result<Self,ContractFailure> {
        if raw.len() > 1_000_000 || expected_owner.is_some_and(|owner|owner.len()>4096) {
            return Err(ContractFailure::SchemaInvalid);
        }
        if u64::try_from(raw.len()).ok() != Some(custody.byte_length) {
            return Err(ContractFailure::ByteLengthMismatch);
        }
        if <[u8; 32]>::from(Sha256::digest(raw)) != custody.sha256 {
            return Err(ContractFailure::ShaMismatch);
        }
        let mut cursor = Cursor(raw);
        cursor.take(b"{\"atoms\":[")?;
        let mut atoms: Vec<Vec<u8>> = Vec::new();
        if !cursor.0.starts_with(b"]") {
            loop {
                if atoms.len() == 65536 {
                    return Err(ContractFailure::SchemaInvalid);
                }
                cursor.take(b"{\"registered_atom_identity_hex\":\"")?;
                let atom = cursor.hex()?;
                cursor.take(b"\",\"status\":\"ADMITTED\"}")?;
                if atoms.last().is_some_and(|previous| previous >= &atom) {
                    return Err(ContractFailure::SchemaInvalid);
                }
                atoms.push(atom);
                if !cursor.0.starts_with(b",") {
                    break;
                }
                cursor.take(b",")?;
            }
        }
        cursor.take(b"],\"owner_registry_identity_hex\":\"")?;
        let owner = cursor.hex()?;
        cursor.take(b"\",\"schema\":\"ddn.registered-atom-contract.v1\",\"version\":1}\n")?;
        if !cursor.0.is_empty() {
            return Err(ContractFailure::SchemaInvalid);
        }
        if expected_owner.is_some_and(|expected|owner!=expected) {
            return Err(ContractFailure::OwnerMismatch);
        }
        let mut snapshot = Sha256::new();
        snapshot.update(b"DDN-REGISTERED-ATOM-CONTRACT-SNAPSHOT-V1\0");
        snapshot.update(1u16.to_le_bytes());
        snapshot.update(custody.byte_length.to_le_bytes());
        snapshot.update(raw);
        if <[u8; 32]>::from(snapshot.finalize()) != custody.snapshot_identity {
            return Err(ContractFailure::SnapshotMismatch);
        }
        Ok(Self {
            owner,
            atoms,
            custody,
            recorded_bytes: raw.to_vec(),
        })
    }

    pub(crate) fn admit(&self, owner: &[u8], atom: &[u8]) -> Result<(), ContractFailure> {
        if owner != self.owner {
            return Err(ContractFailure::OwnerMismatch);
        }
        self.atoms
            .binary_search_by(|candidate| candidate.as_slice().cmp(atom))
            .map(|_| ())
            .map_err(|_| ContractFailure::NotAdmitted)
    }

    pub(crate) fn custody(&self) -> &ContractCustody {
        &self.custody
    }

    pub(crate) fn recorded_bytes(&self) -> &[u8] {
        &self.recorded_bytes
    }

    /// Construct only from membership verified against this recorded contract.
    pub(crate) fn admitted_value(
        &self,
        owner: &[u8],
        atom: &[u8],
    ) -> Result<ddonirang_core::platform::RegisteredAtomValueV1, ContractFailure> {
        self.admit(owner, atom)?;
        Ok(ddonirang_core::platform::RegisteredAtomValueV1 {
            profile_version: 1,
            owner_registry_identity: owner.to_vec(),
            registered_atom_identity: atom.to_vec(),
            contract_profile: "ddn.registered-atom-contract.v1".to_owned(),
            contract_version: 1,
            contract_snapshot_identity: self.custody.snapshot_identity.to_vec(),
            contract_sha256: self.custody.sha256,
        })
    }

    /// A structurally valid saved leaf is not evidence of admission.
    /// Never substitute today's contract for its recorded provenance.
    pub(crate) fn verify_saved_value(
        &self,
        value: &ddonirang_core::platform::RegisteredAtomValueV1,
    ) -> Result<(), ContractFailure> {
        if value.profile_version != 1
            || value.contract_profile != "ddn.registered-atom-contract.v1"
            || value.contract_version != 1
        {
            return Err(ContractFailure::SchemaInvalid);
        }
        if value.contract_snapshot_identity != self.custody.snapshot_identity {
            return Err(ContractFailure::SnapshotMismatch);
        }
        if value.contract_sha256 != self.custody.sha256 {
            return Err(ContractFailure::ShaMismatch);
        }
        self.admit(
            &value.owner_registry_identity,
            &value.registered_atom_identity,
        )
    }
}

/// Conflicting snapshots for one owner cannot be selected by iteration order.
pub(crate) fn verify_contract_set(contracts: &[RecordedAtomContract]) -> Result<(), ContractFailure> {
    if contracts.len() > 256 { return Err(ContractFailure::SchemaInvalid); }
    for (index, contract) in contracts.iter().enumerate() {
        if contracts[..index].iter().any(|previous| previous.owner == contract.owner) {
            return Err(ContractFailure::SnapshotMismatch);
        }
    }
    Ok(())
}

pub(crate) fn verify_recorded_atom(value: &ddonirang_core::platform::RegisteredAtomValueV1, contracts: &[RecordedAtomContract]) -> Result<(), ContractFailure> {
    let contract = contracts.iter().find(|contract| contract.owner == value.owner_registry_identity)
        .ok_or(ContractFailure::NotAdmitted)?;
    contract.verify_saved_value(value)
}

struct Cursor<'a>(&'a [u8]);
impl Cursor<'_> {
    fn take(&mut self, prefix: &[u8]) -> Result<(), ContractFailure> {
        self.0 = self
            .0
            .strip_prefix(prefix)
            .ok_or(ContractFailure::SchemaInvalid)?;
        Ok(())
    }

    fn hex(&mut self) -> Result<Vec<u8>, ContractFailure> {
        let end = self
            .0
            .iter()
            .position(|&b| b == b'"')
            .ok_or(ContractFailure::SchemaInvalid)?;
        let raw = &self.0[..end];
        if raw.is_empty() || raw.len() > 8192 || raw.len() % 2 != 0 {
            return Err(ContractFailure::SchemaInvalid);
        }
        fn nibble(b: u8) -> Result<u8, ContractFailure> {
            match b {
                b'0'..=b'9' => Ok(b - b'0'),
                b'a'..=b'f' => Ok(b - b'a' + 10),
                _ => Err(ContractFailure::SchemaInvalid),
            }
        }
        let bytes = raw
            .chunks_exact(2)
            .map(|p| Ok(nibble(p[0])? * 16 + nibble(p[1])?))
            .collect::<Result<Vec<_>, ContractFailure>>()?;
        self.0 = &self.0[end..];
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::platform::ResourceValue;

    #[test]
    fn beta1_atom_json_requires_recorded_contract_before_collection_merge() {
        let (_, contract)=RecordedAtomContract::record(b"owner", &[vec![1]]).unwrap();
        let atom=ResourceValue::RegisteredAtom(contract.admitted_value(b"owner", &[1]).unwrap().into());
        let wire=crate::unit_wire::resource_value_to_detjson(&atom);
        assert!(crate::unit_wire::resource_value_from_detjson(&wire).is_err());
        assert_eq!(crate::unit_wire::resource_value_from_detjson_with_contracts(&wire, &[contract.clone()]).unwrap(), atom);
        for kind in ["list","set"] {
            let valid=serde_json::json!({"type":kind,"items":[wire.clone()]});
            assert!(crate::unit_wire::resource_value_from_detjson_with_contracts(&valid,&[contract.clone()]).is_ok());
            let mut forged=wire.clone();forged["contract_sha256"]=serde_json::json!("00".repeat(32));
            // Same semantic key must not erase invalid provenance in either order.
            for items in [vec![wire.clone(),forged.clone()],vec![forged,wire.clone()]] {
                let bad=serde_json::json!({"type":kind,"items":items});
                assert!(crate::unit_wire::resource_value_from_detjson_with_contracts(&bad,&[contract.clone()]).is_err());
            }
        }
        let (_,successor)=RecordedAtomContract::record(b"owner", &[vec![1],vec![2]]).unwrap();
        assert!(crate::unit_wire::resource_value_from_detjson_with_contracts(&wire,&[successor.clone()]).is_err());
        assert!(crate::unit_wire::resource_value_from_detjson_with_contracts(&wire,&[contract,successor]).is_err());
    }

    #[test]
    fn beta1_atom_closed_wire_rejects_shape_bounds_and_raw_duplicates() {
        let (_, contract)=RecordedAtomContract::record(b"owner", &[vec![1]]).unwrap();
        let wire=crate::unit_wire::resource_value_to_detjson(&ResourceValue::RegisteredAtom(contract.admitted_value(b"owner", &[1]).unwrap().into()));
        for (key,value) in [
            ("profile_version",serde_json::json!(1.0)),
            ("contract_version",serde_json::json!(2)),
            ("owner_registry_identity_hex",serde_json::json!("AB")),
            ("registered_atom_identity_hex",serde_json::json!("00".repeat(4097))),
            ("contract_snapshot_identity_hex",serde_json::json!("00".repeat(31))),
            ("contract_snapshot_identity_hex",serde_json::json!("00".repeat(33))),
            ("contract_sha256",serde_json::json!("00".repeat(31))),
            ("contract_sha256",serde_json::json!("00".repeat(33))),
            ("unexpected",serde_json::json!(true)),
        ] {
            let mut bad=wire.clone();bad[key]=value;
            assert!(crate::unit_wire::resource_value_from_detjson_with_contracts(&bad,&[contract.clone()]).is_err(),"{key}");
        }
        for key in wire.as_object().unwrap().keys() {
            let mut bad=wire.clone();bad.as_object_mut().unwrap().remove(key);
            assert!(crate::unit_wire::resource_value_from_detjson_with_contracts(&bad,&[contract.clone()]).is_err(),"missing {key}");
        }
        let raw=serde_json::to_string(&wire).unwrap();
        let duplicate=raw.replacen("{","{\"profile_version\":1,",1);
        assert!(crate::unit_wire::parse_snapshot_json(&duplicate).is_err());
    }

    #[test]
    fn beta1_atom_runtime_carrier_and_record_keep_typed_identity() {
        let (_, contract)=RecordedAtomContract::record(b"owner", &[vec![1]]).unwrap();
        let atom=ResourceValue::RegisteredAtom(contract.admitted_value(b"owner", &[1]).unwrap().into());
        let runtime=crate::ddn_runtime::value_from_resource_value(&atom);
        assert!(matches!(runtime,ddonirang_lang::runtime::Value::RegisteredAtom(_)));
        let record=ResourceValue::Record {
            field_types:std::collections::BTreeMap::from([("값".into(),ddonirang_core::platform::RecordFieldType::Named{name:"값".into()})]),
            fields:std::collections::BTreeMap::from([("값".into(),atom)]),
        };
        let wire=crate::unit_wire::resource_value_to_detjson(&record);
        assert!(crate::unit_wire::resource_value_from_detjson(&wire).is_err());
        assert_eq!(crate::unit_wire::resource_value_from_detjson_with_contracts(&wire,&[contract]).unwrap(),record);
    }
    #[test]
    fn producer_checks_total_artifact_budget_before_hex_expansion() {
        let atoms: Vec<Vec<u8>> = (0u16..130).map(|ordinal| {
            let mut atom = vec![1; 4096];
            atom[..2].copy_from_slice(&ordinal.to_be_bytes());
            atom
        }).collect();
        assert_eq!(RecordedAtomContract::record(&[1], &atoms).unwrap_err(), ContractFailure::SchemaInvalid);
        let (raw, _) = RecordedAtomContract::record(&[1], &atoms[..100]).unwrap();
        assert!(raw.len() < 1_000_000);
    }
    #[test]
    fn saved_provenance_cannot_substitute_current_membership() {
        let (_, first) = RecordedAtomContract::record(&[1], &[vec![2]]).unwrap();
        let (_, successor) = RecordedAtomContract::record(&[1], &[vec![2], vec![3]]).unwrap();
        let leaf = first.admitted_value(&[1], &[2]).unwrap();
        assert_eq!(first.verify_saved_value(&leaf), Ok(()));
        assert_eq!(successor.verify_saved_value(&leaf), Err(ContractFailure::SnapshotMismatch));
        let mut forged = leaf;
        forged.contract_sha256[0] ^= 1;
        assert_eq!(first.verify_saved_value(&forged), Err(ContractFailure::ShaMismatch));
    }
    #[test]
    fn rd668_atom_identity_bounds_precede_materialization() {
        assert!(RecordedAtomContract::record(&vec![1; 4096], &[vec![2; 4096]]).is_ok());
        assert!(RecordedAtomContract::record(&vec![1; 4097], &[vec![2]]).is_err());
        assert!(RecordedAtomContract::record(&[1], &[vec![2; 4097]]).is_err());
        let oversized = vec![b'0'; 8194];
        let mut quoted = oversized;
        quoted.push(b'"');
        assert!(Cursor(&quoted).hex().is_err());
        let raw = vec![0; 1_000_001];
        assert!(RecordedAtomContract::restore(
            &raw,
            &[1],
            ContractCustody {
                byte_length: raw.len() as u64,
                sha256: Sha256::digest(&raw).into(),
                snapshot_identity: [0; 32],
            }
        )
        .is_err());
    }
    const ONE: &[u8] = b"{\"atoms\":[{\"registered_atom_identity_hex\":\"01\",\"status\":\"ADMITTED\"}],\"owner_registry_identity_hex\":\"61\",\"schema\":\"ddn.registered-atom-contract.v1\",\"version\":1}\n";

    fn digest(hex: &str) -> [u8; 32] {
        let quoted = format!("{hex}\"");
        Cursor(quoted.as_bytes()).hex().unwrap().try_into().unwrap()
    }

    fn custody() -> ContractCustody {
        ContractCustody {
            byte_length: 160,
            sha256: digest("712380168dce69f9cc853aa01666842862cb353f6bb72b54319942d88f586b1c"),
            snapshot_identity: digest(
                "5579eef061208f511d32b2350dea271eff7d2775f8e4f9df41a01dd676712b49",
            ),
        }
    }

    #[test]
    fn fixed_vector_preserves_membership_and_custody() {
        let contract = RecordedAtomContract::restore(ONE, b"a", custody()).unwrap();
        assert_eq!(contract.custody(), &custody());
        assert_eq!(contract.admit(b"a", &[1]), Ok(()));
        assert_eq!(
            contract.admit(b"a", &[2]),
            Err(ContractFailure::NotAdmitted)
        );
        assert_eq!(
            contract.admit(b"b", &[1]),
            Err(ContractFailure::OwnerMismatch)
        );
    }





    #[test]
    fn producer_matches_fixed_bytes_without_order_repair() {
        let (raw, contract) = RecordedAtomContract::record(b"a", &[vec![1]]).unwrap();
        assert_eq!(raw, ONE);
        assert_eq!(contract.custody(), &custody());
        assert!(RecordedAtomContract::record(b"a", &[vec![2], vec![1]]).is_err());
        assert!(RecordedAtomContract::record(b"a", &[vec![1], vec![1]]).is_err());
        assert!(RecordedAtomContract::record(b"", &[]).is_err());
    }

    #[test]
    fn alternate_json_bytes_are_not_repaired() {
        for raw in [
            String::from_utf8(ONE.to_vec())
                .unwrap()
                .replace("version\":1", "version\":1.0"),
            String::from_utf8(ONE.to_vec())
                .unwrap()
                .replace("{\"atoms", "{ \"atoms"),
            String::from_utf8(ONE.to_vec()).unwrap().replace("01", "AA"),
            String::from_utf8(ONE.to_vec())
                .unwrap()
                .replace('\n', "\r\n"),
        ] {
            let mut seal = custody();
            seal.byte_length = raw.len() as u64;
            seal.sha256 = Sha256::digest(raw.as_bytes()).into();
            assert_eq!(
                RecordedAtomContract::restore(raw.as_bytes(), b"a", seal).unwrap_err(),
                ContractFailure::SchemaInvalid
            );
        }
    }

    #[test]
    fn tampered_custody_fails_before_admission() {
        let mut seal = custody();
        seal.sha256[0] ^= 1;
        assert_eq!(
            RecordedAtomContract::restore(ONE, b"a", seal).unwrap_err(),
            ContractFailure::ShaMismatch
        );
        let mut seal = custody();
        seal.snapshot_identity[0] ^= 1;
        assert_eq!(
            RecordedAtomContract::restore(ONE, b"a", seal).unwrap_err(),
            ContractFailure::SnapshotMismatch
        );
    }

    #[test]
    fn empty_contract_vector_is_valid_but_admits_nothing() {
        let raw = b"{\"atoms\":[],\"owner_registry_identity_hex\":\"61\",\"schema\":\"ddn.registered-atom-contract.v1\",\"version\":1}\n";
        let seal = ContractCustody {
            byte_length: 103,
            sha256: digest("a6391da915339266708927e83b2225a78ba8f3824cdf376add22aebc2e92714c"),
            snapshot_identity: digest(
                "bb7016aac6cbd47ad7ee022b13297bcf6dbeaed96befddea7c996c0b318d43f9",
            ),
        };
        let contract = RecordedAtomContract::restore(raw, b"a", seal).unwrap();
        assert_eq!(
            contract.admit(b"a", &[1]),
            Err(ContractFailure::NotAdmitted)
        );
    }

    #[test]
    fn duplicate_and_descending_atoms_are_rejected_before_snapshot() {
        for atoms in [
            "{\"registered_atom_identity_hex\":\"01\",\"status\":\"ADMITTED\"},{\"registered_atom_identity_hex\":\"01\",\"status\":\"ADMITTED\"}",
            "{\"registered_atom_identity_hex\":\"02\",\"status\":\"ADMITTED\"},{\"registered_atom_identity_hex\":\"01\",\"status\":\"ADMITTED\"}",
        ] {
            let raw = format!("{{\"atoms\":[{atoms}],\"owner_registry_identity_hex\":\"61\",\"schema\":\"ddn.registered-atom-contract.v1\",\"version\":1}}\n");
            let mut seal = custody();
            seal.byte_length = raw.len() as u64;
            seal.sha256 = Sha256::digest(raw.as_bytes()).into();
            assert_eq!(RecordedAtomContract::restore(raw.as_bytes(), b"a", seal).unwrap_err(), ContractFailure::SchemaInvalid);
        }
    }
}
