use crate::ast::{CanonProgram, SeedDef, SeedKind, TopLevelItem, TypeRef};
use crate::stdlib::canonicalize_type_alias;
use std::collections::BTreeMap;

/// M-A: the closed builtin family is resolved after authored callable owners.
/// This is a compilation-local dispatch identity, not a saved identity/tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarMathBuiltin { Sin, Cos, Sqrt, Atan2 }

impl ScalarMathBuiltin {
    pub fn canonical_name(self) -> &'static str {
        match self { Self::Sin=>"sin", Self::Cos=>"cos", Self::Sqrt=>"sqrt", Self::Atan2=>"atan2" }
    }
    pub fn scalar_pin_type(self) -> Option<TypeRef> {
        // The atan2 successor assigns a conditional result only, no pin.
        (self!=Self::Atan2).then(||TypeRef::Named("셈수".into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarMathCallableOwner {
    Declared(Vec<crate::ast::NodeId>),
    Builtin(ScalarMathBuiltin),
}

pub fn resolve_scalar_math_callable(program:&CanonProgram, spelling:&str) -> Option<ScalarMathCallableOwner> {
    let builtin=match crate::stdlib::canonicalize_stdlib_alias(spelling) {
        "sin"=>ScalarMathBuiltin::Sin, "cos"=>ScalarMathBuiltin::Cos,
        "sqrt"=>ScalarMathBuiltin::Sqrt, "atan2"=>ScalarMathBuiltin::Atan2, _=>return None,
    };
    let declarations=program.items.iter().filter_map(|item| {
        let TopLevelItem::SeedDef(seed)=item;
        (is_v1_ordinary_callable(seed) && seed.callable_surface_names().iter().any(|name|name==spelling)).then_some(seed.id)
    }).collect::<Vec<_>>();
    Some(if declarations.is_empty() {ScalarMathCallableOwner::Builtin(builtin)}
        else {ScalarMathCallableOwner::Declared(declarations)})
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrdinaryCallablePairDisposition {
    ProvenDisjoint,
    KnownOverlap,
    Unknown,
    Unresolved,
}

impl OrdinaryCallablePairDisposition {
    pub fn label(self) -> &'static str {
        match self {
            Self::ProvenDisjoint => "PROVEN_DISJOINT",
            Self::KnownOverlap => "KNOWN_OVERLAP",
            Self::Unknown => "UNKNOWN",
            Self::Unresolved => "UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryCallableAdmissionEvidence {
    pub effective_set: String,
    pub left_signature: String,
    pub right_signature: String,
    pub disposition: OrdinaryCallablePairDisposition,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinaryCallableAdmissionFailure {
    pub evidence: OrdinaryCallableAdmissionEvidence,
    pub span_start: usize,
    pub span_end: usize,
}

impl OrdinaryCallableAdmissionFailure {
    pub fn diagnostic_message(&self) -> String {
        format!(
            "E_ORDINARY_CALLABLE_ADMISSION_NOT_DISJOINT: effective_set={} left={} right={} disposition={} reason={}",
            self.evidence.effective_set,
            self.evidence.left_signature,
            self.evidence.right_signature,
            self.evidence.disposition.label(),
            self.evidence.reason,
        )
    }
}

pub fn is_v1_ordinary_callable(seed: &SeedDef) -> bool {
    matches!(
        seed.seed_kind,
        SeedKind::Semssi | SeedKind::ValueFunc | SeedKind::Umjikssi
    )
}

pub fn validate_ordinary_callable_admission(
    program: &CanonProgram,
) -> Result<Vec<OrdinaryCallableAdmissionEvidence>, OrdinaryCallableAdmissionFailure> {
    let mut families = BTreeMap::<String, Vec<&SeedDef>>::new();
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        if is_v1_ordinary_callable(seed) {
            families
                .entry(seed.canonical_name.clone())
                .or_default()
                .push(seed);
        }
    }

    let mut evidence = Vec::new();
    for (name, seeds) in families {
        for left_index in 0..seeds.len() {
            for right_index in left_index + 1..seeds.len() {
                let left = seeds[left_index];
                let right = seeds[right_index];
                let (disposition, reason) = compare_input_signatures(left, right);
                let mut left_signature = ordinary_callable_input_signature(left);
                let mut right_signature = ordinary_callable_input_signature(right);
                if right_signature < left_signature {
                    std::mem::swap(&mut left_signature, &mut right_signature);
                }
                let row = OrdinaryCallableAdmissionEvidence {
                    effective_set: name.clone(),
                    left_signature,
                    right_signature,
                    disposition,
                    reason,
                };
                if disposition != OrdinaryCallablePairDisposition::ProvenDisjoint {
                    return Err(OrdinaryCallableAdmissionFailure {
                        evidence: row,
                        span_start: right.span.start,
                        span_end: right.span.end,
                    });
                }
                evidence.push(row);
            }
        }
    }
    evidence.sort_by(|left, right| {
        (
            &left.effective_set,
            &left.left_signature,
            &left.right_signature,
        )
            .cmp(&(
                &right.effective_set,
                &right.left_signature,
                &right.right_signature,
            ))
    });
    Ok(evidence)
}

fn compare_input_signatures(
    left: &SeedDef,
    right: &SeedDef,
) -> (OrdinaryCallablePairDisposition, &'static str) {
    if left
        .params
        .iter()
        .any(|pin| pin.optional || pin.default_value.is_some())
        || right
            .params
            .iter()
            .any(|pin| pin.optional || pin.default_value.is_some())
    {
        return (
            OrdinaryCallablePairDisposition::Unresolved,
            "FIXED_EXPLICIT_INPUT_SHAPE_REQUIRED",
        );
    }
    if left.params.len() != right.params.len()
        || !left.params.iter().zip(&right.params).all(|(left, right)| {
            left.pin_name == right.pin_name && left.josa_list == right.josa_list
        })
    {
        return (
            OrdinaryCallablePairDisposition::Unresolved,
            "STABLE_PIN_MAPPING_UNRESOLVED",
        );
    }
    let mut unknown = false;
    for (left, right) in left.params.iter().zip(&right.params) {
        match type_relation(&left.type_ref, &right.type_ref) {
            TypeRelation::Disjoint => {
                return (
                    OrdinaryCallablePairDisposition::ProvenDisjoint,
                    "DISJOINT_PARAMETER_TYPE",
                )
            }
            TypeRelation::Overlap => {}
            TypeRelation::Unknown => unknown = true,
        }
    }
    if unknown {
        (
            OrdinaryCallablePairDisposition::Unknown,
            "INPUT_TYPE_DISJOINTNESS_UNKNOWN",
        )
    } else {
        (
            OrdinaryCallablePairDisposition::KnownOverlap,
            "INPUT_DOMAINS_OVERLAP",
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeRelation {
    Disjoint,
    Overlap,
    Unknown,
}

fn type_relation(left: &TypeRef, right: &TypeRef) -> TypeRelation {
    if let (TypeRef::Tuple(a), TypeRef::Tuple(b)) = (left, right) {
        if a.len() != b.len() { return TypeRelation::Disjoint; }
        let mut unknown = false;
        for (a, b) in a.iter().zip(b) {
            match type_relation(a, b) {
                TypeRelation::Disjoint => return TypeRelation::Disjoint,
                TypeRelation::Unknown => unknown = true,
                TypeRelation::Overlap => {}
            }
        }
        return if unknown { TypeRelation::Unknown } else { TypeRelation::Overlap };
    }
    if let (TypeRef::Tensor { element: a, shape: sa }, TypeRef::Tensor { element: b, shape: sb }) = (left, right) {
        return if sa == sb && a == b { TypeRelation::Overlap } else { TypeRelation::Disjoint };
    }
    let left = runtime_shape(left);
    let right = runtime_shape(right);
    match (left, right) {
        (None, _) | (_, None) => TypeRelation::Unknown,
        (Some(left), Some(right)) if left == right => TypeRelation::Overlap,
        (Some("any"), _) | (_, Some("any")) => TypeRelation::Overlap,
        _ => TypeRelation::Disjoint,
    }
}

pub fn proven_same_input_domain(left: &TypeRef, right: &TypeRef) -> bool {
    matches!(type_relation(left, right), TypeRelation::Overlap)
        && runtime_shape(left) != Some("any")
        && runtime_shape(right) != Some("any")
}

fn runtime_shape(type_ref: &TypeRef) -> Option<&'static str> {
    let name = match type_ref {
        TypeRef::Tuple(_) => return Some("tuple"),
        TypeRef::Tensor { .. } => return Some("tensor"),
        TypeRef::Named(name) => canonicalize_type_alias(name),
        TypeRef::Applied { name, .. } => canonicalize_type_alias(name),
        TypeRef::Function { .. } | TypeRef::Infer => return None,
    };
    Some(match name {
        "값" => "any",
        "수" | "셈수" | "바른수" | "큰바른수" | "나눔수" | "곱수" => "number",
        "글" => "string",
        "바이트열" => "bytes",
        "참거짓" => "boolean",
        "없음" => "none",
        "세움값" => "assertion",
        "상태머신값" => "state-machine",
        "차림" => "list",
        "모음" => "set",
        "짝맞춤" => "map",
        "묶음" => "pack",
        "자원" | "자원핸들" => "resource",
        "수식값" => "formula",
        "글무늬값" => "template",
        "정규식" => "regex",
        _ => return None,
    })
}

pub fn ordinary_callable_input_signature(seed: &SeedDef) -> String {
    let pins = seed
        .params
        .iter()
        .map(|pin| format!("{}:{:?}", pin.pin_name, pin.type_ref))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}({pins})", seed.canonical_name)
}
