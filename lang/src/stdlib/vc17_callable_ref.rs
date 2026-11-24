//! RD-684 invocation-scoped physical carrier. No public value, wire, or name fallback.
//! Registry rows must come from the admitted provider owner, never authored strings.
// Keep the recovered SHA-256 byte contract without a new Cargo dependency.
// ByteSequence owns the existing core SHA-256 implementation; these private
// registration messages are bounded by MAX_BYTES before carrier admission.
struct Sha256(Vec<u8>);
impl Sha256 {
    fn new() -> Self { Self(Vec::new()) }
    fn update(&mut self, bytes: impl AsRef<[u8]>) { self.0.extend_from_slice(bytes.as_ref()); }
    fn finalize(self) -> [u8;32] { Self::digest(self.0) }
    fn digest(bytes: impl AsRef<[u8]>) -> [u8;32] {
        ddonirang_core::bytes::ByteSequence::new(bytes.as_ref().to_vec()).content_sha256()
    }
}

const DOMAIN: &[u8] = b"DDN-TYPED-NAMED-PURE-CALLABLE-REF-V1\0";
const PIN_DOMAIN: &[u8] = b"DDN-PARAM-PIN-SET-IDENTITY-V1\0";
pub const MAX_BYTES: usize = 4096;
const TYPE_DOMAIN: &[u8] = b"DDN-VC17-RUNTIME-TYPE-IDENTITY-V1\0";

/// RD-685 closed registry. No display spelling is a byte-level identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcreteType {
    Text, Natural, Number, Bool,
    List(Box<ConcreteType>), Optional(Box<ConcreteType>),
    Callable(Box<ConcreteType>, Box<ConcreteType>),
}

impl ConcreteType {
    pub fn from_resolved_type(ty: &crate::ast::TypeRef) -> Result<Self, RefError> {
        use crate::ast::TypeRef;
        match ty {
            TypeRef::Named(name) => match crate::stdlib::canonicalize_type_alias(name) {
                "글" => Ok(Self::Text), "바른수" => Ok(Self::Natural), "셈수" => Ok(Self::Number),
                "참거짓" => Ok(Self::Bool), _ => Err(RefError::Type),
            },
            TypeRef::Applied { name, args } if args.len() == 1 => {
                let inner = Box::new(Self::from_resolved_type(&args[0])?);
                match name.as_str() { "차림" => Ok(Self::List(inner)), "있없" => Ok(Self::Optional(inner)), _ => Err(RefError::Type) }
            }
            _ => Err(RefError::Type),
        }
    }
    fn node(&self, depth: usize) -> Result<Vec<u8>, RefError> {
        if depth > 16 { return Err(RefError::Bound); }
        let (code, children): (u8, Vec<&Self>) = match self {
            Self::Text => return Ok(vec![1, 1]), Self::Natural => return Ok(vec![1, 2]),
            Self::Number => return Ok(vec![1, 3]), Self::Bool => return Ok(vec![1, 4]),
            Self::List(t) => (1, vec![t]), Self::Optional(t) => (2, vec![t]),
            Self::Callable(a, b) => (3, vec![a, b]),
        };
        let mut out = vec![2, code, children.len() as u8];
        for child in children {
            let bytes = child.node(depth + 1)?;
            let size = (out.len() as u64).checked_add(4 + bytes.len() as u64).ok_or(RefError::Bound)?;
            if size > 1024 { return Err(RefError::Bound); }
            out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
            out.extend_from_slice(&bytes);
        }
        Ok(out)
    }
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RefError> {
        let node = self.node(1)?;
        if TYPE_DOMAIN.len() + 2 + node.len() > 1024 { return Err(RefError::Bound); }
        let mut out = TYPE_DOMAIN.to_vec();
        out.extend_from_slice(&1u16.to_be_bytes()); out.extend_from_slice(&node); Ok(out)
    }
    pub fn from_canonical(bytes: &[u8]) -> Result<Self, RefError> {
        if bytes.len() > 1024 { return Err(RefError::Bound); }
        if !bytes.starts_with(TYPE_DOMAIN) || bytes.get(TYPE_DOMAIN.len()..TYPE_DOMAIN.len()+2) != Some(&[0,1][..]) {
            return Err(RefError::Type);
        }
        fn parse_node(bytes: &[u8], depth: usize, materialize: bool) -> Result<Option<ConcreteType>, RefError> {
            if depth > 16 { return Err(RefError::Bound); }
            if bytes.len() == 2 && bytes[0] == 1 {
                let ty = match bytes[1] { 1 => ConcreteType::Text, 2 => ConcreteType::Natural,
                    3 => ConcreteType::Number, 4 => ConcreteType::Bool, _ => return Err(RefError::Type) };
                return Ok(materialize.then_some(ty));
            }
            if bytes.len() < 3 || bytes[0] != 2 { return Err(RefError::Type); }
            let count = match bytes[1] { 1 | 2 => 1, 3 => 2, _ => return Err(RefError::Type) };
            if bytes[2] != count { return Err(RefError::Type); }
            let mut at = 3usize;
            let mut children = Vec::new();
            for _ in 0..count {
                let header = bytes.get(at..at+4).ok_or(RefError::Type)?;
                let len = u32::from_be_bytes(header.try_into().unwrap()) as u64;
                let end = (at as u64).checked_add(4).and_then(|n| n.checked_add(len)).ok_or(RefError::Bound)?;
                if end > bytes.len() as u64 { return Err(RefError::Type); }
                if let Some(child) = parse_node(&bytes[at+4..end as usize], depth+1, materialize)? { children.push(child); }
                at = end as usize;
            }
            if at != bytes.len() { return Err(RefError::Type); }
            if !materialize { return Ok(None); }
            let mut it = children.into_iter();
            let a = Box::new(it.next().unwrap());
            Ok(Some(match bytes[1] { 1 => ConcreteType::List(a), 2 => ConcreteType::Optional(a),
                3 => ConcreteType::Callable(a, Box::new(it.next().unwrap())), _ => unreachable!() }))
        }
        let node = &bytes[TYPE_DOMAIN.len()+2..];
        parse_node(node, 1, false)?; // complete bounded preflight before materialization
        let ty = parse_node(node, 1, true)?.unwrap();
        if ty.canonical_bytes()? != bytes { return Err(RefError::Type); }
        Ok(ty)
    }
    pub fn admits(&self, value: &crate::runtime::Value) -> bool {
        use crate::runtime::Value;
        match (self, value) {
            (Self::Text, Value::String(_)) | (Self::Number, Value::Fixed64(_)) | (Self::Bool, Value::Bool(_)) => true,
            (Self::Natural, Value::Fixed64(n)) => n.frac_part() == 0 && n.int_part() >= 0,
            (Self::List(t), Value::List(items)) => items.iter().all(|v| t.admits(v)),
            (Self::Optional(_), Value::None) => true,
            (Self::Optional(t), value) => t.admits(value),
            _ => false,
        }
    }
    pub fn of_value(value: &crate::runtime::Value) -> Result<Self, RefError> {
        use crate::runtime::Value;
        match value {
            Value::String(_) => Ok(Self::Text), Value::Fixed64(_) => Ok(Self::Number), Value::Bool(_) => Ok(Self::Bool),
            Value::List(items) => {
                let t = Self::of_value(items.first().ok_or(RefError::Type)?)?;
                for item in items { if Self::of_value(item)? != t { return Err(RefError::Type); } }
                Ok(Self::List(Box::new(t)))
            }
            _ => Err(RefError::Type),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefError { Structure, Bound, Identity, PinSet, Type, Ambiguous, Effect }
impl std::fmt::Display for RefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E_VC17_ADMISSION: 이름 있는 PURE 셈씨 참조 검증 실패 ({self:?})")
    }
}
impl std::error::Error for RefError {}

/// Exact metadata projected by the existing registry owner. Canonical types are
/// opaque here: equality against admitted bytes, never display-name encoding.
#[derive(Debug)]
pub struct RegisteredTarget<'a> {
    pub provider: &'a str,
    pub version: u16,
    pub definition: &'a str,
    pub canonical_pins: &'a [&'a str],
    pub request_type: &'a [u8],
    pub result_type: &'a [u8],
    pub required_inputs: u16,
    pub optional_inputs: u16,
    pub default_inputs: u16,
    pub pure: bool,
    pub capture_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedCallableRef {
    bytes: Vec<u8>,
    identity: [u8; 32],
}
impl VerifiedCallableRef {
    pub fn canonical_bytes(&self) -> &[u8] { &self.bytes }
    pub fn identity(&self) -> &[u8; 32] { &self.identity }
}

fn identity(value: &str) -> Result<(), RefError> {
    if value.is_empty() || value.len() > 512 || value.as_bytes().contains(&0) {
        return Err(RefError::Identity);
    }
    Ok(())
}
fn type_bytes(value: &[u8]) -> Result<(), RefError> {
    if value.is_empty() || value.len() > 1024 { return Err(RefError::Type); }
    Ok(())
}

pub fn pin_set_identity(definition: &str, pins: &[&str]) -> Result<[u8; 32], RefError> {
    identity(definition)?;
    if pins.is_empty() || pins.len() > 64 { return Err(RefError::PinSet); }
    for (i, pin) in pins.iter().enumerate() {
        identity(pin)?;
        if pins[..i].contains(pin) { return Err(RefError::PinSet); }
    }
    // Hash incrementally; no attacker-sized preimage allocation.
    let mut h = Sha256::new();
    h.update(PIN_DOMAIN);
    h.update(1u16.to_be_bytes());
    h.update([1]);
    h.update((definition.len() as u32).to_be_bytes());
    h.update(definition.as_bytes());
    h.update((pins.len() as u16).to_be_bytes());
    for (ordinal, pin) in pins.iter().enumerate() {
        h.update((ordinal as u16).to_be_bytes());
        h.update((pin.len() as u32).to_be_bytes());
        h.update(pin.as_bytes());
    }
    Ok(h.finalize().into())
}

fn encode(row: &RegisteredTarget<'_>) -> Result<Vec<u8>, RefError> {
    identity(row.provider)?;
    identity(row.definition)?;
    type_bytes(row.request_type)?;
    type_bytes(row.result_type)?;
    let pins = pin_set_identity(row.definition, row.canonical_pins)?;
    let version = row.version.to_be_bytes();
    let fields = [row.provider.as_bytes(), &version, row.definition.as_bytes(),
        &pins, row.request_type, row.result_type];
    let mut size = (DOMAIN.len() + 3) as u64;
    for field in fields {
        size = size.checked_add(5).and_then(|n| n.checked_add(field.len() as u64))
            .ok_or(RefError::Bound)?;
    }
    if size > MAX_BYTES as u64 { return Err(RefError::Bound); }
    let mut out = Vec::with_capacity(size as usize);
    out.extend_from_slice(DOMAIN);
    out.extend_from_slice(&1u16.to_be_bytes());
    out.push(6);
    for (index, field) in fields.into_iter().enumerate() {
        out.push((index + 1) as u8);
        out.extend_from_slice(&(field.len() as u32).to_be_bytes());
        out.extend_from_slice(field);
    }
    Ok(out)
}

fn parse(bytes: &[u8]) -> Result<[&[u8]; 6], RefError> {
    if bytes.len() > MAX_BYTES { return Err(RefError::Bound); }
    if !bytes.starts_with(DOMAIN) { return Err(RefError::Structure); }
    let mut at = DOMAIN.len();
    if bytes.get(at..at + 3) != Some(&[0, 1, 6][..]) { return Err(RefError::Structure); }
    at += 3;
    let mut fields: [&[u8]; 6] = [&[]; 6];
    for (i, field) in fields.iter_mut().enumerate() {
        let head = bytes.get(at..at + 5).ok_or(RefError::Structure)?;
        if head[0] != (i + 1) as u8 { return Err(RefError::Structure); }
        let len = u32::from_be_bytes(head[1..5].try_into().unwrap()) as u64;
        let end = (at as u64).checked_add(5).and_then(|n| n.checked_add(len))
            .ok_or(RefError::Bound)?;
        if end > bytes.len() as u64 { return Err(RefError::Structure); }
        *field = &bytes[at + 5..end as usize];
        at = end as usize;
    }
    if at != bytes.len() { return Err(RefError::Structure); }
    identity(std::str::from_utf8(fields[0]).map_err(|_| RefError::Identity)?)?;
    identity(std::str::from_utf8(fields[2]).map_err(|_| RefError::Identity)?)?;
    if fields[1].len() != 2 || fields[3].len() != 32 { return Err(RefError::Structure); }
    type_bytes(fields[4])?;
    type_bytes(fields[5])?;
    Ok(fields)
}

/// Re-verification is required against the current sealed invocation registry.
/// A previously verified reference does not confer admission to a stale registry.
pub fn verify(bytes: &[u8], registry: &[RegisteredTarget<'_>], element_type: &[u8])
    -> Result<VerifiedCallableRef, RefError>
{
    let fields = parse(bytes)?; // All structural bounds before any lookup/allocation.
    let mut candidates = registry.iter().filter(|row| {
        row.provider.as_bytes() == fields[0] && row.version.to_be_bytes() == fields[1]
            && row.definition.as_bytes() == fields[2]
    });
    let row = candidates.next().ok_or(RefError::Identity)?;
    if candidates.next().is_some() { return Err(RefError::Ambiguous); }
    let pin_digest = pin_set_identity(row.definition, row.canonical_pins)?;
    if fields[3] != pin_digest { return Err(RefError::PinSet); }
    if fields[4] != row.request_type || fields[5] != row.result_type
        || fields[4] != element_type { return Err(RefError::Type); }
    if row.canonical_pins.len() != 1 || row.required_inputs != 1
        || row.optional_inputs != 0 || row.default_inputs != 0 { return Err(RefError::PinSet); }
    if !row.pure || row.capture_count != 0 { return Err(RefError::Effect); }
    let canonical = encode(row)?;
    if canonical != bytes { return Err(RefError::Structure); }
    let identity = Sha256::digest(&canonical).into();
    Ok(VerifiedCallableRef { bytes: canonical, identity })
}

pub fn issue(row: &RegisteredTarget<'_>, registry: &[RegisteredTarget<'_>], element_type: &[u8])
    -> Result<VerifiedCallableRef, RefError>
{
    verify(&encode(row)?, registry, element_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rd685_closed_type_roundtrip_and_firewalls() {
        use ConcreteType::*;
        for ty in [Text, Natural, Number, Bool, List(Box::new(Text)), Optional(Box::new(Number)), Callable(Box::new(Text), Box::new(Natural)), List(Box::new(List(Box::new(Number))))] {
            let bytes = ty.canonical_bytes().unwrap();
            assert_eq!(ConcreteType::from_canonical(&bytes).unwrap(), ty);
            let mut extra = bytes.clone(); extra.push(0);
            assert!(ConcreteType::from_canonical(&extra).is_err());
            for n in 0..bytes.len() { assert!(ConcreteType::from_canonical(&bytes[..n]).is_err()); }
        }
        for node in [&[1, 0][..], &[1, 5], &[3, 0], &[2, 1, 0], &[2, 3, 1], &[2, 4, 1]] {
            let mut bytes = TYPE_DOMAIN.to_vec(); bytes.extend_from_slice(&[0,1]); bytes.extend_from_slice(node);
            assert!(ConcreteType::from_canonical(&bytes).is_err());
        }
        let mut deep = Text;
        for _ in 0..16 { deep = List(Box::new(deep)); }
        assert_eq!(deep.canonical_bytes(), Err(RefError::Bound));
    }
    // Structural fixture only: these bytes never enter the product registry.
    fn row() -> RegisteredTarget<'static> {
        RegisteredTarget { provider: "fixture.pure", version: 1, definition: "fixture.def",
            canonical_pins: &["fixture.def.input"], request_type: b"fixture.type",
            result_type: b"fixture.type", required_inputs: 1, optional_inputs: 0,
            default_inputs: 0, pure: true, capture_count: 0 }
    }
    #[test]
    fn rd684_structural_vector_is_not_product_registry_authority() {
        let mut fixture = row();
        fixture.provider = "review.fixture.vc17.named-pure-v1";
        fixture.definition = "REVIEW.FIXTURE.DEF.ADD_ONE.R1";
        fixture.canonical_pins = &["REVIEW.FIXTURE.PIN.ADD_ONE.VALUE.R1"];
        let mut ty = b"DDN-BASIC-TENSOR-ELEMENT-TYPE-IDENTITY-V1\0".to_vec();
        ty.extend_from_slice(&[0, 1, 1, 1, 0, 0, 0, 1, 1]);
        fixture.request_type = &ty;
        fixture.result_type = &ty;
        let bytes = encode(&fixture).unwrap();
        assert_eq!(bytes.len(), 268);
        let hash = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect::<String>();
        assert_eq!(hash, "4b9085436da7fe2ac82839ab93e32fe75cfd26f381ed40a2251a90678283e456");
        assert_eq!(verify(&bytes, &[], &ty), Err(RefError::Identity));
    }
    #[test]
    fn exact_roundtrip_and_stale_registry() {
        let admitted = issue(&row(), &[row()], b"fixture.type").unwrap();
        assert_eq!(verify(admitted.canonical_bytes(), &[row()], b"fixture.type").unwrap(), admitted);
        assert_eq!(verify(admitted.canonical_bytes(), &[], b"fixture.type"), Err(RefError::Identity));
        assert_eq!(verify(admitted.canonical_bytes(), &[row(), row()], b"fixture.type"), Err(RefError::Ambiguous));
    }
    #[test]
    fn every_field_and_eof_are_bound() {
        let original = encode(&row()).unwrap();
        let mut at = DOMAIN.len() + 3;
        for _ in 0..6 {
            let len = u32::from_be_bytes(original[at+1..at+5].try_into().unwrap()) as usize;
            let mut corrupt = original.clone();
            corrupt[at+5] ^= 1;
            assert!(verify(&corrupt, &[row()], b"fixture.type").is_err());
            at += 5 + len;
        }
        for end in 0..original.len() { assert!(verify(&original[..end], &[row()], b"fixture.type").is_err()); }
        let mut extra = original; extra.push(0);
        assert_eq!(verify(&extra, &[row()], b"fixture.type"), Err(RefError::Structure));
        assert_eq!(verify(&vec![0; MAX_BYTES + 1], &[], b"fixture.type"), Err(RefError::Bound));
    }
    #[test]
    fn effect_capture_pin_and_element_firewalls() {
        let bytes = encode(&row()).unwrap();
        let mut bad = row(); bad.pure = false;
        assert_eq!(verify(&bytes, &[bad], b"fixture.type"), Err(RefError::Effect));
        let mut bad = row(); bad.capture_count = 1;
        assert_eq!(verify(&bytes, &[bad], b"fixture.type"), Err(RefError::Effect));
        let mut bad = row(); bad.optional_inputs = 1;
        assert_eq!(verify(&bytes, &[bad], b"fixture.type"), Err(RefError::PinSet));
        assert_eq!(verify(&bytes, &[row()], b"foreign.type"), Err(RefError::Type));
        assert_eq!(pin_set_identity("d", &["p", "p"]), Err(RefError::PinSet));
        assert_ne!(pin_set_identity("d", &["a", "b"]).unwrap(), pin_set_identity("d", &["b", "a"]).unwrap());
    }
}
