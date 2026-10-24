//! RD-682 bounded physical projection of the resolved canonical type AST.
//! Alias resolution belongs to the existing canonicalizer, never this codec.
use ddonirang_lang::ast::TypeRef;

const DOMAIN: &[u8] = b"DDN-BASIC-TENSOR-ELEMENT-TYPE-IDENTITY-V1\0";
const LENGTH: usize = 51;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ElementTypeError {
    UnsupportedIdentity,
    MalformedProfile,
    Noncanonical,
    ResolutionMismatch,
    LimitExceeded,
}

/// Only an already resolved canonical AST identity can issue this profile.
pub(crate) fn issue(resolved: &TypeRef) -> Result<[u8; LENGTH], ElementTypeError> {
    if !matches!(resolved, TypeRef::Named(name) if name == "셈수") {
        return Err(ElementTypeError::UnsupportedIdentity);
    }
    let mut bytes = [0; LENGTH];
    bytes[..DOMAIN.len()].copy_from_slice(DOMAIN);
    bytes[DOMAIN.len()..].copy_from_slice(&[0, 1, 1, 1, 0, 0, 0, 1, 1]);
    Ok(bytes)
}

/// Allocation-free validation for callers admitting an entire compound value.
pub(crate) fn validate_bytes(bytes: &[u8]) -> Result<(), ElementTypeError> {
    // Fixed bounds precede allocation and AST construction.
    if bytes.len() != LENGTH {
        return Err(ElementTypeError::LimitExceeded);
    }
    if &bytes[..DOMAIN.len()] != DOMAIN {
        return Err(ElementTypeError::MalformedProfile);
    }
    if &bytes[DOMAIN.len()..] != [0, 1, 1, 1, 0, 0, 0, 1, 1] {
        return Err(ElementTypeError::Noncanonical);
    }
    Ok(())
}

pub(crate) fn decode(bytes: &[u8]) -> Result<TypeRef, ElementTypeError> {
    validate_bytes(bytes)?;
    let resolved = TypeRef::Named("셈수".to_owned());
    if issue(&resolved)?.as_slice() != bytes {
        return Err(ElementTypeError::Noncanonical);
    }
    Ok(resolved)
}

pub(crate) fn verify(bytes: &[u8], resolved: &TypeRef) -> Result<(), ElementTypeError> {
    validate_bytes(bytes)?;
    if !matches!(resolved, TypeRef::Named(name) if name == "셈수") {
        return Err(ElementTypeError::ResolutionMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn rd682_exact_vector_and_resolved_owner_roundtrip() {
        let kind = TypeRef::Named("셈수".into());
        let bytes = issue(&kind).unwrap();
        assert_eq!(bytes.len(), 51);
        let digest = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(digest, "f334b6f3e3530ac476fbf00af39d40e1a4cec8af95e4f1126161e91900680b05");
        assert_eq!(decode(&bytes), Ok(kind.clone()));
        assert_eq!(verify(&bytes, &kind), Ok(()));
        assert_eq!(issue(&decode(&bytes).unwrap()).unwrap(), bytes);
    }

    #[test]
    fn unresolved_aliases_and_other_families_cannot_issue() {
        for kind in [
            TypeRef::Named("fixed64".into()), TypeRef::Named("TYPE-I64".into()),
            TypeRef::Named("바른수".into()), TypeRef::Named("글".into()),
            TypeRef::Named("".into()), TypeRef::Infer,
            TypeRef::Applied { name: "셈수".into(), args: vec![] },
            TypeRef::Function { params: vec![], result: Box::new(TypeRef::Named("셈수".into())) },
        ] {
            assert_eq!(issue(&kind), Err(ElementTypeError::UnsupportedIdentity));
        }
    }

    #[test]
    fn every_byte_and_exact_exhaustion_are_authoritative() {
        let fixed = issue(&TypeRef::Named("셈수".into())).unwrap();
        for index in 0..fixed.len() {
            let mut changed = fixed;
            changed[index] ^= 1;
            assert!(decode(&changed).is_err(), "byte {index}");
        }
        for length in 0..fixed.len() {
            assert!(decode(&fixed[..length]).is_err());
        }
        let mut suffix = fixed.to_vec();
        suffix.push(0);
        assert!(decode(&suffix).is_err());
    }

    #[test]
    fn foreign_resolved_identity_fails_without_alias_repair() {
        let bytes = issue(&TypeRef::Named("셈수".into())).unwrap();
        for name in ["바른수", "fixed64", "TYPE-I64"] {
            assert_eq!(verify(&bytes, &TypeRef::Named(name.into())), Err(ElementTypeError::ResolutionMismatch));
        }
    }
}
