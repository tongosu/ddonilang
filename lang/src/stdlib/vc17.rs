//! RD-675: the existing standard-library owner's REQUIRED-only physical exports.
//! Surface spelling ends at admission. Runtime dispatch consumes Definition, never a name.
use crate::ast::NodeId;
use crate::runtime::Value;

/// RD-542 S08 finite call tail; not a general alias or stem rule.
pub const SUJARO_CALL_TAIL: &str = "숫자로 바꾸기";
pub const PROVIDER: &str = "stdlib.vc17.ddn-pure-minimal-v1";
pub const VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Definition {
    Geulro,
    Giri,
    Dadumgi,
    Deureoitna,
    Majimak,
    TextReplace,
    CollectionTransform,
    TextJoin,
    SequenceConcat,
    Sujaro,
    Jareugi,
    Jego,
    Chajabogi,
    Cheotbeonjje,
    Chuga,
    Tomaknaegi,
    Pohamhana,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Any,
    Text,
    Sequence,
    TextSequence,
    TextOrSequence,
    Index,
    Callable,
}

/// RD-685 schemes are typed metadata, not display-string dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scheme { T0, T1, Text, Natural, Number, Bool, List(Box<Scheme>), Optional(Box<Scheme>), Callable(Box<Scheme>, Box<Scheme>), TextOrList }
pub fn signature(definition: Definition) -> (Vec<Scheme>, Scheme, bool) {
    use Scheme::*;
    let list = |t| List(Box::new(t));
    let optional = |t| Optional(Box::new(t));
    match definition {
        Definition::Geulro => (vec![T0], Text, true),
        Definition::Giri => (vec![TextOrList], Natural, true),
        Definition::Dadumgi => (vec![Text], Text, true),
        Definition::Deureoitna => (vec![list(T0), T0], Bool, false),
        Definition::Majimak | Definition::Cheotbeonjje => (vec![list(T0)], optional(T0), true),
        Definition::TextReplace => (vec![Text, Text, Text], Text, false),
        Definition::CollectionTransform => (vec![list(T0), Callable(Box::new(T0), Box::new(T1))], list(T1), false),
        Definition::TextJoin => (vec![list(Text), Text], Text, false),
        Definition::SequenceConcat => (vec![list(T0), list(T0)], list(T0), false),
        Definition::Sujaro => (vec![Text], optional(Number), true),
        Definition::Jareugi => (vec![Text, Text], list(Text), false),
        Definition::Jego => (vec![list(T0), Natural], list(T0), false),
        Definition::Chajabogi => (vec![list(T0), T0], optional(Natural), false),
        Definition::Chuga => (vec![list(T0), T0], list(T0), false),
        Definition::Tomaknaegi => (vec![list(T0), Natural, Natural], list(T0), false),
        Definition::Pohamhana => (vec![Text, Text], Bool, false),
    }
}

#[derive(Debug, Clone)]
pub struct NamedPureBinding {
    definition: Definition,
    request: super::vc17_callable_ref::ConcreteType,
    result: super::vc17_callable_ref::ConcreteType,
    carrier: super::vc17_callable_ref::VerifiedCallableRef,
}
impl NamedPureBinding {
    pub fn definition(&self) -> Definition { self.definition }
    pub fn request(&self) -> &super::vc17_callable_ref::ConcreteType { &self.request }
    pub fn result(&self) -> &super::vc17_callable_ref::ConcreteType { &self.result }
    pub fn carrier(&self) -> &super::vc17_callable_ref::VerifiedCallableRef { &self.carrier }
    pub fn reverify(&self) -> Result<(), AdmissionError> {
        let verified = bind_named_pure(self.definition, self.request.clone())?;
        if verified.carrier != self.carrier || verified.result != self.result { return Err(AdmissionError::Type); }
        Ok(())
    }
}

pub fn bind_named_pure(definition: Definition, request: super::vc17_callable_ref::ConcreteType) -> Result<NamedPureBinding, AdmissionError> {
    use super::vc17_callable_ref::{self, ConcreteType as T, RegisteredTarget};
    let (_, _, eligible) = signature(definition);
    if !eligible { return Err(AdmissionError::Type); }
    let result = match (definition, &request) {
        (Definition::Geulro, _) => T::Text,
        (Definition::Giri, T::Text | T::List(_)) => T::Natural,
        (Definition::Dadumgi, T::Text) => T::Text,
        (Definition::Sujaro, T::Text) => T::Optional(Box::new(T::Number)),
        (Definition::Cheotbeonjje | Definition::Majimak, T::List(t)) => T::Optional(t.clone()),
        _ => return Err(AdmissionError::Type),
    };
    let export = EXPORTS.iter().find(|e| e.definition == definition).ok_or(AdmissionError::Definition)?;
    let pins = export.pins.iter().map(|p| p.identity).collect::<Vec<_>>();
    let request_bytes = request.canonical_bytes().map_err(|_| AdmissionError::Type)?;
    let result_bytes = result.canonical_bytes().map_err(|_| AdmissionError::Type)?;
    if T::from_canonical(&request_bytes).map_err(|_| AdmissionError::Type)? != request
        || T::from_canonical(&result_bytes).map_err(|_| AdmissionError::Type)? != result { return Err(AdmissionError::Type); }
    let row = RegisteredTarget { provider: PROVIDER, version: VERSION, definition: export.identity,
        canonical_pins: &pins, request_type: &request_bytes, result_type: &result_bytes,
        required_inputs: 1, optional_inputs: 0, default_inputs: 0, pure: true, capture_count: 0 };
    let carrier = vc17_callable_ref::issue(&row, std::slice::from_ref(&row), &request_bytes)
        .map_err(|_| AdmissionError::Type)?;
    Ok(NamedPureBinding { definition, request, result, carrier })
}

/// Authored identifier resolution only, before identity-bound invocation. Never Value::String.
pub fn resolve_named_pure(surface: &str, request: super::vc17_callable_ref::ConcreteType) -> Result<NamedPureBinding, AdmissionError> {
    let mut matches = EXPORTS.iter().filter(|e| e.surface == surface);
    let export = matches.next().ok_or(AdmissionError::NoCandidate)?;
    if matches.next().is_some() { return Err(AdmissionError::Ambiguous); }
    bind_named_pure(export.definition, request)
}

#[derive(Debug)]
pub struct Pin {
    pub name: &'static str,
    pub identity: &'static str,
    pub input: InputType,
}

#[derive(Debug)]
pub struct Export {
    pub definition: Definition,
    pub identity: &'static str,
    pub surface: &'static str,
    pub pins: &'static [Pin],
}

pub static EXPORTS: &[Export] = &[
    Export {
        definition: Definition::Geulro,
        identity: "VC17.DEF.GEULRO.R1",
        surface: "글로",
        pins: &[Pin {
            name: "값",
            identity: "VC17.PIN.GEULRO.VALUE.R1",
            input: InputType::Any,
        }],
    },
    Export {
        definition: Definition::Giri,
        identity: "VC17.DEF.GIRI.R1",
        surface: "길이",
        pins: &[Pin {
            name: "대상",
            identity: "VC17.PIN.GIRI.TARGET.R1",
            input: InputType::TextOrSequence,
        }],
    },
    Export {
        definition: Definition::Dadumgi,
        identity: "VC17.DEF.DADUMGI.R1",
        surface: "다듬기",
        pins: &[Pin {
            name: "글",
            identity: "VC17.PIN.DADUMGI.TEXT.R1",
            input: InputType::Text,
        }],
    },
    Export {
        definition: Definition::Deureoitna,
        identity: "VC17.DEF.DEUREOITNA.R1",
        surface: "들어있나",
        pins: &[
            Pin {
                name: "차림",
                identity: "VC17.PIN.DEUREOITNA.SEQUENCE.R1",
                input: InputType::Sequence,
            },
            Pin {
                name: "값",
                identity: "VC17.PIN.DEUREOITNA.VALUE.R1",
                input: InputType::Any,
            },
        ],
    },
    Export {
        definition: Definition::Majimak,
        identity: "VC17.DEF.MAJIMAK.R1",
        surface: "마지막",
        pins: &[Pin {
            name: "차림",
            identity: "VC17.PIN.MAJIMAK.SEQUENCE.R1",
            input: InputType::Sequence,
        }],
    },
    Export {
        definition: Definition::TextReplace,
        identity: "VC17.DEF.BAKKUGI.TEXT_REPLACE.R1",
        surface: "바꾸기",
        pins: &[
            Pin {
                name: "글",
                identity: "VC17.PIN.BAKKUGI.TEXT_REPLACE.SOURCE_TEXT.R1",
                input: InputType::Text,
            },
            Pin {
                name: "찾을글",
                identity: "VC17.PIN.BAKKUGI.TEXT_REPLACE.OLD_FRAGMENT.R1",
                input: InputType::Text,
            },
            Pin {
                name: "바꿀글",
                identity: "VC17.PIN.BAKKUGI.TEXT_REPLACE.NEW_FRAGMENT.R1",
                input: InputType::Text,
            },
        ],
    },
    Export {
        definition: Definition::CollectionTransform,
        identity: "VC17.DEF.BAKKUGI.COLLECTION_TRANSFORM.R1",
        surface: "바꾸기",
        pins: &[
            Pin {
                name: "차림",
                identity: "VC17.PIN.BAKKUGI.COLLECTION_TRANSFORM.SEQUENCE.R1",
                input: InputType::Sequence,
            },
            Pin {
                name: "바꿀셈씨",
                identity: "VC17.PIN.BAKKUGI.COLLECTION_TRANSFORM.TRANSFORM.R1",
                input: InputType::Callable,
            },
        ],
    },
    Export {
        definition: Definition::TextJoin,
        identity: "VC17.DEF.BUTIGI.TEXT_JOIN.R1",
        surface: "붙이기",
        pins: &[
            Pin {
                name: "글차림",
                identity: "VC17.PIN.BUTIGI.TEXT_JOIN.TEXTS.R1",
                input: InputType::TextSequence,
            },
            Pin {
                name: "사이글",
                identity: "VC17.PIN.BUTIGI.TEXT_JOIN.JOINER.R1",
                input: InputType::Text,
            },
        ],
    },
    Export {
        definition: Definition::SequenceConcat,
        identity: "VC17.DEF.BUTIGI.SEQUENCE_CONCAT.R1",
        surface: "붙이기",
        pins: &[
            Pin {
                name: "앞차림",
                identity: "VC17.PIN.BUTIGI.SEQUENCE_CONCAT.LEFT_SEQUENCE.R1",
                input: InputType::Sequence,
            },
            Pin {
                name: "뒤차림",
                identity: "VC17.PIN.BUTIGI.SEQUENCE_CONCAT.RIGHT_SEQUENCE.R1",
                input: InputType::Sequence,
            },
        ],
    },
    Export {
        definition: Definition::Sujaro,
        identity: "VC17.DEF.SUJARO.R1",
        surface: "숫자로",
        pins: &[Pin {
            name: "글",
            identity: "VC17.PIN.SUJARO.TEXT.R1",
            input: InputType::Text,
        }],
    },
    Export {
        definition: Definition::Jareugi,
        identity: "VC17.DEF.JAREUGI.R1",
        surface: "자르기",
        pins: &[
            Pin {
                name: "글",
                identity: "VC17.PIN.JAREUGI.TEXT.R1",
                input: InputType::Text,
            },
            Pin {
                name: "구분",
                identity: "VC17.PIN.JAREUGI.DELIMITER.R1",
                input: InputType::Text,
            },
        ],
    },
    Export {
        definition: Definition::Jego,
        identity: "VC17.DEF.JEGO.R1",
        surface: "제거",
        pins: &[
            Pin {
                name: "차림",
                identity: "VC17.PIN.JEGO.SEQUENCE.R1",
                input: InputType::Sequence,
            },
            Pin {
                name: "위치",
                identity: "VC17.PIN.JEGO.POSITION.R1",
                input: InputType::Index,
            },
        ],
    },
    Export {
        definition: Definition::Chajabogi,
        identity: "VC17.DEF.CHAJABOGI.R1",
        surface: "찾아보기",
        pins: &[
            Pin {
                name: "차림",
                identity: "VC17.PIN.CHAJABOGI.SEQUENCE.R1",
                input: InputType::Sequence,
            },
            Pin {
                name: "값",
                identity: "VC17.PIN.CHAJABOGI.VALUE.R1",
                input: InputType::Any,
            },
        ],
    },
    Export {
        definition: Definition::Cheotbeonjje,
        identity: "VC17.DEF.CHEOTBEONJJE.R1",
        surface: "첫번째",
        pins: &[Pin {
            name: "차림",
            identity: "VC17.PIN.CHEOTBEONJJE.SEQUENCE.R1",
            input: InputType::Sequence,
        }],
    },
    Export {
        definition: Definition::Chuga,
        identity: "VC17.DEF.CHUGA.R1",
        surface: "추가",
        pins: &[
            Pin {
                name: "차림",
                identity: "VC17.PIN.CHUGA.SEQUENCE.R1",
                input: InputType::Sequence,
            },
            Pin {
                name: "값",
                identity: "VC17.PIN.CHUGA.VALUE.R1",
                input: InputType::Any,
            },
        ],
    },
    Export {
        definition: Definition::Tomaknaegi,
        identity: "VC17.DEF.TOMAKNAEGI.R1",
        surface: "토막내기",
        pins: &[
            Pin {
                name: "차림",
                identity: "VC17.PIN.TOMAKNAEGI.SEQUENCE.R1",
                input: InputType::Sequence,
            },
            Pin {
                name: "시작",
                identity: "VC17.PIN.TOMAKNAEGI.START.R1",
                input: InputType::Index,
            },
            Pin {
                name: "끝",
                identity: "VC17.PIN.TOMAKNAEGI.END.R1",
                input: InputType::Index,
            },
        ],
    },
    Export {
        definition: Definition::Pohamhana,
        identity: "VC17.DEF.POHAMHANA.R1",
        surface: "포함하나",
        pins: &[
            Pin {
                name: "글",
                identity: "VC17.PIN.POHAMHANA.TEXT.R1",
                input: InputType::Text,
            },
            Pin {
                name: "조각",
                identity: "VC17.PIN.POHAMHANA.FRAGMENT.R1",
                input: InputType::Text,
            },
        ],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    Provider,
    Version,
    Definition,
    PinSet,
    Type,
    NoCandidate,
    Ambiguous,
    Provenance,
    Bound,
    EmptyPattern,
}

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Provider => "표준 라이브러리 공급자 identity가 일치하지 않습니다",
            Self::Version => "표준 라이브러리 공급자 판본이 일치하지 않습니다",
            Self::Definition => "등록된 표준 라이브러리 정의 identity가 아닙니다",
            Self::PinSet => "필수 핀이 누락되었거나 중복·알 수 없는 핀·다른 정의의 핀입니다",
            Self::Type => "표준 라이브러리 핀의 인자 타입이 일치하지 않습니다",
            Self::NoCandidate => "표면·핀·타입에 맞는 표준 라이브러리 정의가 없습니다",
            Self::Ambiguous => "표면·핀·타입에 맞는 표준 라이브러리 정의가 둘 이상입니다",
            Self::Provenance => "표준 라이브러리 호출의 원문 결속이 없습니다",
            Self::Bound => "표준 라이브러리 입력 또는 결과가 허용된 크기를 넘습니다",
            Self::EmptyPattern => "빈 구분 또는 빈 찾을글은 허용되지 않습니다",
        };
        write!(f, "E_VC17_ADMISSION: {message}")
    }
}
impl std::error::Error for AdmissionError {}

pub const TEXT_BYTES_MAX: u64 = 1_000_000;
pub const SEQUENCE_ELEMENTS_MAX: u64 = 65_536;

pub fn check_operand(value: &Value) -> Result<(), AdmissionError> {
    match value {
        Value::String(text) if text.len() as u64 > TEXT_BYTES_MAX => Err(AdmissionError::Bound),
        Value::List(items) if items.len() as u64 > SEQUENCE_ELEMENTS_MAX => Err(AdmissionError::Bound),
        _ => Ok(()),
    }
}

/// RD-684: finish all result-size arithmetic before constructing the output.
pub fn replace_text(text: &str, pattern: &str, replacement: &str) -> Result<String, AdmissionError> {
    if pattern.is_empty() { return Err(AdmissionError::EmptyPattern); }
    if [text.len(), pattern.len(), replacement.len()].iter().any(|n| *n as u64 > TEXT_BYTES_MAX) {
        return Err(AdmissionError::Bound);
    }
    let matches = text.match_indices(pattern).count() as u64;
    let removed = matches.checked_mul(pattern.len() as u64).ok_or(AdmissionError::Bound)?;
    let inserted = matches.checked_mul(replacement.len() as u64).ok_or(AdmissionError::Bound)?;
    let size = (text.len() as u64).checked_sub(removed).and_then(|n| n.checked_add(inserted))
        .ok_or(AdmissionError::Bound)?;
    if size > TEXT_BYTES_MAX { return Err(AdmissionError::Bound); }
    let mut output = String::with_capacity(size as usize);
    let mut end = 0;
    for (start, matched) in text.match_indices(pattern) {
        output.push_str(&text[end..start]);
        output.push_str(replacement);
        end = start + matched.len();
    }
    output.push_str(&text[end..]);
    Ok(output)
}

pub fn split_text(text: &str, delimiter: &str) -> Result<Vec<Value>, AdmissionError> {
    if delimiter.is_empty() { return Err(AdmissionError::EmptyPattern); }
    if text.len() as u64 > TEXT_BYTES_MAX || delimiter.len() as u64 > TEXT_BYTES_MAX {
        return Err(AdmissionError::Bound);
    }
    let count = (text.match_indices(delimiter).count() as u64).checked_add(1)
        .ok_or(AdmissionError::Bound)?;
    if count > SEQUENCE_ELEMENTS_MAX { return Err(AdmissionError::Bound); }
    Ok(text.split(delimiter).map(|part| Value::String(part.to_owned())).collect())
}

pub fn check_join(items: &[Value], delimiter: &str) -> Result<(), AdmissionError> {
    if items.len() as u64 > SEQUENCE_ELEMENTS_MAX || delimiter.len() as u64 > TEXT_BYTES_MAX {
        return Err(AdmissionError::Bound);
    }
    let gaps = if items.is_empty() { 0 } else { items.len() as u64 - 1 };
    let mut size = gaps.checked_mul(delimiter.len() as u64).ok_or(AdmissionError::Bound)?;
    for item in items {
        let Value::String(text) = item else { return Err(AdmissionError::Type); };
        if text.len() as u64 > TEXT_BYTES_MAX { return Err(AdmissionError::Bound); }
        size = size.checked_add(text.len() as u64).ok_or(AdmissionError::Bound)?;
    }
    if size > TEXT_BYTES_MAX { return Err(AdmissionError::Bound); }
    Ok(())
}

pub fn check_concat(left: &[Value], right: &[Value]) -> Result<(), AdmissionError> {
    let count = (left.len() as u64).checked_add(right.len() as u64).ok_or(AdmissionError::Bound)?;
    if count > SEQUENCE_ELEMENTS_MAX { return Err(AdmissionError::Bound); }
    // The current Value::List has no type witness for two empty operands.
    if count == 0 { return Err(AdmissionError::Type); }
    Ok(())
}

/// In-process only: the shared AST call node is retained, not a new persisted identity.
#[derive(Debug, Clone)]
pub struct Invocation {
    definition: Definition,
    arguments: Vec<Value>,
    call_node: NodeId,
    transform: Option<NamedPureBinding>,
}
impl Invocation {
    pub fn definition(&self) -> Definition {
        self.definition
    }
    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }
    pub fn call_node(&self) -> NodeId {
        self.call_node
    }
    pub fn transform(&self) -> Option<&NamedPureBinding> { self.transform.as_ref() }
}

pub fn admit_transform(sequence: Value, binding: NamedPureBinding, call_node: NodeId) -> Result<Invocation, AdmissionError> {
    check_operand(&sequence)?;
    binding.reverify()?;
    let Value::List(items) = &sequence else { return Err(AdmissionError::Type); };
    if !items.iter().all(|item| binding.request.admits(item)) { return Err(AdmissionError::Type); }
    Ok(Invocation { definition: Definition::CollectionTransform, arguments: vec![sequence], call_node, transform: Some(binding) })
}

fn matches_type(input: InputType, value: &Value) -> bool {
    match input {
        InputType::Any => true,
        InputType::Text => matches!(value, Value::String(_)),
        InputType::Sequence => matches!(value, Value::List(_)),
        InputType::TextSequence => {
            matches!(value, Value::List(items) if items.iter().all(|v| matches!(v, Value::String(_))))
        }
        InputType::TextOrSequence => matches!(value, Value::String(_) | Value::List(_)),
        InputType::Index => {
            matches!(value, Value::Fixed64(n) if n.frac_part() == 0 && n.int_part() >= 0)
        }
        // General Value is not a function-reference carrier. The separate
        // admit_transform lane requires an immutable verified NamedPureBinding.
        InputType::Callable => false,
    }
}

/// Exact export/pin boundary. There is no display-name or Rust-symbol fallback.
pub fn admit(
    provider: &str,
    version: u16,
    definition: &str,
    arguments: &[(&str, Value)],
    call_node: Option<NodeId>,
) -> Result<Invocation, AdmissionError> {
    if provider != PROVIDER {
        return Err(AdmissionError::Provider);
    }
    if version != VERSION {
        return Err(AdmissionError::Version);
    }
    let export = EXPORTS
        .iter()
        .find(|e| e.identity == definition)
        .ok_or(AdmissionError::Definition)?;
    let call_node = call_node.ok_or(AdmissionError::Provenance)?;
    for (_, value) in arguments { check_operand(value)?; }
    if arguments.len() != export.pins.len() {
        return Err(AdmissionError::PinSet);
    }
    let mut ordered = Vec::with_capacity(export.pins.len());
    for pin in export.pins {
        let mut matching = arguments.iter().filter(|(id, _)| *id == pin.identity);
        let (_, value) = matching.next().ok_or(AdmissionError::PinSet)?;
        if matching.next().is_some() {
            return Err(AdmissionError::PinSet);
        }
        if !matches_type(pin.input, value) {
            return Err(AdmissionError::Type);
        }
        ordered.push(value.clone());
    }
    Ok(Invocation {
        definition: export.definition,
        arguments: ordered,
        call_node,
        transform: None,
    })
}

pub fn is_surface(surface: &str) -> bool {
    EXPORTS.iter().any(|e| e.surface == surface)
}

/// The parser checks the authored pin set without guessing unresolved types.
/// Child definitions with different pin sets can be canonically ordered here;
/// positional typed overloads remain unresolved until typed admission.
pub fn bind_source_pins(
    surface: &str,
    arguments: &mut Vec<crate::ast::ArgBinding>,
) -> Result<(), AdmissionError> {
    let named = arguments.iter().any(|a| a.resolved_pin.is_some());
    if arguments.iter().any(|a| a.josa.is_some())
        || (named && arguments.iter().any(|a| a.resolved_pin.is_none()))
    {
        return Err(AdmissionError::PinSet);
    }
    let candidates = EXPORTS
        .iter()
        .filter(|e| e.surface == surface && e.pins.len() == arguments.len())
        .filter(|e| {
            !named
                || e.pins.iter().all(|p| {
                    arguments
                        .iter()
                        .filter(|a| a.resolved_pin.as_deref() == Some(p.name))
                        .count()
                        == 1
                })
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Err(AdmissionError::PinSet);
    }
    if named {
        let [export] = candidates.as_slice() else {
            return Err(AdmissionError::Ambiguous);
        };
        let ordered = export
            .pins
            .iter()
            .map(|p| {
                arguments
                    .iter()
                    .find(|a| a.resolved_pin.as_deref() == Some(p.name))
                    .unwrap()
                    .clone()
            })
            .collect();
        *arguments = ordered;
    }
    Ok(())
}

/// Resolve source presentation, then immediately bind definition-scoped physical pins.
/// Positional inputs use the definition's declared pin order, never registry order.
/// Explicit pins may be reordered; mixing named and positional input is not repaired.
pub fn resolve(
    surface: &str,
    arguments: &[(Option<&str>, Value)],
    call_node: NodeId,
) -> Result<Invocation, AdmissionError> {
    resolve_in(EXPORTS.iter(), surface, arguments, call_node)
}

fn resolve_in<'a>(
    exports: impl Iterator<Item = &'a Export>,
    surface: &str,
    arguments: &[(Option<&str>, Value)],
    call_node: NodeId,
) -> Result<Invocation, AdmissionError> {
    let named = arguments.iter().any(|(pin, _)| pin.is_some());
    for (_, value) in arguments { check_operand(value)?; }
    if named && arguments.iter().any(|(pin, _)| pin.is_none()) {
        return Err(AdmissionError::PinSet);
    }
    let mut selected = None;
    for export in exports.filter(|e| e.surface == surface) {
        if export.pins.len() != arguments.len() {
            continue;
        }
        let mut bound = Vec::with_capacity(export.pins.len());
        let mut eligible = true;
        for (index, pin) in export.pins.iter().enumerate() {
            let value = if named {
                let mut found = arguments.iter().filter(|(name, _)| *name == Some(pin.name));
                match (found.next(), found.next()) {
                    (Some((_, value)), None) => value,
                    _ => {
                        eligible = false;
                        break;
                    }
                }
            } else {
                &arguments[index].1
            };
            if !matches_type(pin.input, value) {
                eligible = false;
                break;
            }
            bound.push((pin.identity, value.clone()));
        }
        if !eligible {
            continue;
        }
        let invocation = admit(PROVIDER, VERSION, export.identity, &bound, Some(call_node))?;
        if selected.replace(invocation).is_some() {
            return Err(AdmissionError::Ambiguous);
        }
    }
    selected.ok_or(AdmissionError::NoCandidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::Fixed64;
    #[test]
    fn rd685_exact_registry_and_six_unary_specializations() {
        use super::super::vc17_callable_ref::ConcreteType as T;
        assert_eq!(EXPORTS.iter().filter(|e| signature(e.definition).2).count(), 6);
        for export in EXPORTS {
            let (pins, _, eligible) = signature(export.definition);
            assert_eq!(pins.len(), export.pins.len());
            if !eligible { assert!(bind_named_pure(export.definition, T::Text).is_err()); }
        }
        for (d, request, result) in [
            (Definition::Geulro, T::Bool, T::Text),
            (Definition::Giri, T::Text, T::Natural),
            (Definition::Giri, T::List(Box::new(T::Text)), T::Natural),
            (Definition::Dadumgi, T::Text, T::Text),
            (Definition::Sujaro, T::Text, T::Optional(Box::new(T::Number))),
            (Definition::Cheotbeonjje, T::List(Box::new(T::Number)), T::Optional(Box::new(T::Number))),
            (Definition::Majimak, T::List(Box::new(T::Bool)), T::Optional(Box::new(T::Bool))),
        ] {
            let binding = bind_named_pure(d, request).unwrap();
            assert_eq!(*binding.result(), result);
            binding.reverify().unwrap();
        }
        assert!(bind_named_pure(Definition::Giri, T::Number).is_err());
        assert!(resolve_named_pure("길이세기", T::Text).is_err());
        assert!(resolve_named_pure("fixture", T::Text).is_err());
        let mut binding = bind_named_pure(Definition::Dadumgi, T::Text).unwrap();
        binding.result = T::Number;
        assert_eq!(binding.reverify(), Err(AdmissionError::Type));
    }
    #[test]
    fn rd684_text_laws_and_preallocation_bounds() {
        assert_eq!(replace_text("aaaaa", "aa", "b").unwrap(), "bba");
        assert_eq!(replace_text("한😀한😀", "한😀", "").unwrap(), "");
        assert_eq!(replace_text("", "a", "b").unwrap(), "");
        assert_eq!(replace_text("", "", "b"), Err(AdmissionError::EmptyPattern));
        assert_eq!(split_text("", "/").unwrap(), vec![Value::String("".into())]);
        assert_eq!(split_text("/가//", "/").unwrap(), ["", "가", "", ""].map(|s| Value::String(s.into())));
        assert_eq!(split_text("abc", ""), Err(AdmissionError::EmptyPattern));
        let exact = "x".repeat(TEXT_BYTES_MAX as usize);
        assert_eq!(replace_text(&exact, "x", "x").unwrap().len(), TEXT_BYTES_MAX as usize);
        assert_eq!(replace_text(&exact, "x", "xx"), Err(AdmissionError::Bound));
        let fragments = "/".repeat(SEQUENCE_ELEMENTS_MAX as usize - 1);
        assert_eq!(split_text(&fragments, "/").unwrap().len(), SEQUENCE_ELEMENTS_MAX as usize);
        assert_eq!(split_text(&(fragments + "/"), "/"), Err(AdmissionError::Bound));
        assert!(check_join(&[], "/").is_ok());
        assert!(check_join(&[Value::String(exact.clone())], "/").is_ok());
        assert_eq!(check_join(&[Value::String(exact), Value::String("".into())], "/"), Err(AdmissionError::Bound));
        assert_eq!(check_concat(&[], &[]), Err(AdmissionError::Type));
        let elements = vec![Value::Bool(true); SEQUENCE_ELEMENTS_MAX as usize];
        assert!(check_concat(&elements, &[]).is_ok());
        assert_eq!(check_concat(&elements, &[Value::Bool(true)]), Err(AdmissionError::Bound));
    }
    #[test]
    fn exact_required_export_and_pin_denominators() {
        use std::collections::HashSet;
        assert_eq!(EXPORTS.len(), 17);
        assert_eq!(
            EXPORTS
                .iter()
                .map(|e| e.surface)
                .collect::<HashSet<_>>()
                .len(),
            15
        );
        assert_eq!(
            EXPORTS
                .iter()
                .map(|e| e.identity)
                .collect::<HashSet<_>>()
                .len(),
            17
        );
        let pins = EXPORTS
            .iter()
            .flat_map(|e| e.pins.iter().map(|p| p.identity))
            .collect::<HashSet<_>>();
        assert_eq!(pins.len(), 30);
        assert_eq!(EXPORTS.iter().map(|e| e.pins.len()).sum::<usize>(), 30);
    }
    #[test]
    fn physical_admission_rejects_unknown_owner_version_pin_and_provenance() {
        let values = [(
            "VC17.PIN.GEULRO.VALUE.R1",
            Value::Fixed64(Fixed64::from_i64(1)),
        )];
        assert_eq!(
            admit("foreign", 1, "VC17.DEF.GEULRO.R1", &values, Some(7)).unwrap_err(),
            AdmissionError::Provider
        );
        assert_eq!(
            admit(PROVIDER, 2, "VC17.DEF.GEULRO.R1", &values, Some(7)).unwrap_err(),
            AdmissionError::Version
        );
        assert_eq!(
            admit(PROVIDER, 1, "글로", &values, Some(7)).unwrap_err(),
            AdmissionError::Definition
        );
        assert_eq!(
            admit(PROVIDER, 1, "VC17.DEF.GIRI.R1", &values, Some(7)).unwrap_err(),
            AdmissionError::PinSet
        );
        assert_eq!(
            admit(PROVIDER, 1, "VC17.DEF.GEULRO.R1", &values, None).unwrap_err(),
            AdmissionError::Provenance
        );
        assert_eq!(
            admit(PROVIDER, 1, "VC17.DEF.GEULRO.R1", &[], Some(7)).unwrap_err(),
            AdmissionError::PinSet
        );
        assert!(admit(PROVIDER, 1, "VC17.DEF.GEULRO.R1", &values, Some(7)).is_ok());
    }
    #[test]
    fn child_resolution_is_typed_and_registry_order_independent() {
        let args = [
            (Some("뒤차림"), Value::List(vec![])),
            (Some("앞차림"), Value::List(vec![])),
        ];
        let a = resolve("붙이기", &args, 9).unwrap();
        let b = resolve_in(EXPORTS.iter().rev(), "붙이기", &args, 9).unwrap();
        assert_eq!(a.definition(), Definition::SequenceConcat);
        assert_eq!(a.definition(), b.definition());
        assert_eq!(a.arguments(), b.arguments());
        assert_eq!(a.call_node(), 9);
        assert_eq!(
            resolve_in(EXPORTS.iter().chain(EXPORTS.iter()), "붙이기", &args, 9).unwrap_err(),
            AdmissionError::Ambiguous
        );
        assert_eq!(
            resolve("이어붙이기", &args, 9).unwrap_err(),
            AdmissionError::NoCandidate
        );
    }
    #[test]
    fn pins_and_fractional_indices_fail_closed() {
        let seq = Value::List(vec![]);
        for args in [
            vec![(Some("차림"), seq.clone()), (Some("차림"), seq.clone())],
            vec![
                (Some("차림"), seq.clone()),
                (
                    Some("위치"),
                    Value::Fixed64(Fixed64::parse_decimal("0.5").unwrap()),
                ),
            ],
            vec![
                (Some("차림"), seq.clone()),
                (None, Value::Fixed64(Fixed64::ZERO)),
            ],
        ] {
            assert!(resolve("제거", &args, 1).is_err());
        }
        let slice = [
            (Some("차림"), seq),
            (Some("처음"), Value::Fixed64(Fixed64::ZERO)),
            (Some("끝"), Value::Fixed64(Fixed64::ZERO)),
        ];
        assert_eq!(
            resolve("토막내기", &slice, 1).unwrap_err(),
            AdmissionError::NoCandidate
        );
    }

    #[test]
    fn removal_outside_collection_is_failure_not_original_value() {
        let list = Value::List(vec![Value::Fixed64(Fixed64::from_i64(10))]);
        for index in [1, 2] {
            assert!(matches!(
                crate::runtime::list_remove(&list, &Value::Fixed64(Fixed64::from_i64(index))),
                Err(crate::runtime::RuntimeError::IndexOutOfRange)
            ));
        }
        assert_eq!(
            crate::runtime::list_remove(&list, &Value::Fixed64(Fixed64::ZERO)).unwrap(),
            Value::List(vec![])
        );
    }

    #[test]
    fn unbound_callable_bridge_does_not_promote_lambda_or_name() {
        let lambda = Value::Lambda(crate::runtime::LambdaValue {
            id: 42,
            param: "값".into(),
            body: crate::ast::Expr::new(
                43,
                crate::ast::Span::new(0, 3),
                crate::ast::ExprKind::Var("값".into()),
            ),
            captured: Default::default(),
        });
        for callable in [lambda, Value::String("증가".into())] {
            let arguments = [
                (
                    "VC17.PIN.BAKKUGI.COLLECTION_TRANSFORM.SEQUENCE.R1",
                    Value::List(vec![]),
                ),
                (
                    "VC17.PIN.BAKKUGI.COLLECTION_TRANSFORM.TRANSFORM.R1",
                    callable,
                ),
            ];
            assert_eq!(
                admit(
                    PROVIDER,
                    VERSION,
                    "VC17.DEF.BAKKUGI.COLLECTION_TRANSFORM.R1",
                    &arguments,
                    Some(1)
                )
                .unwrap_err(),
                AdmissionError::Type
            );
        }
    }

    #[test]
    fn explicit_pins_survive_collection_canonical_roundtrip() {
        let canon = |source: &str| {
            let mut program = crate::parse_v25(source, "vc17.ddn").unwrap();
            crate::canonicalize(&mut program).unwrap();
            crate::normalize(&program, crate::NormalizationLevel::N1)
        };
        for (left, right) in [
            (
                "(차림=차림[10, 20], 위치=0) 제거 보여주기.",
                "(위치=0, 차림=차림[10, 20]) 제거 보여주기.",
            ),
            (
                "(차림=차림[10, 20], 시작=1, 끝=2) 토막내기 보여주기.",
                "(끝=2, 차림=차림[10, 20], 시작=1) 토막내기 보여주기.",
            ),
        ] {
            let canonical = canon(left);
            assert_eq!(canonical, canon(right));
            assert_eq!(canonical, canon(&canonical));
        }
        assert!(crate::parse_v25(
            "(차림=차림[10], 처음=0, 끝=1) 토막내기 보여주기.",
            "vc17.ddn"
        )
        .is_err());
    }
}
