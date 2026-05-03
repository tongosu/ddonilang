use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::core::fixed64::Fixed64;
use crate::core::unit::{format_dim, UnitDim};
use crate::lang::ast::Expr;
use ddonirang_core::ResourceHandle;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quantity {
    pub raw: Fixed64,
    pub dim: UnitDim,
}

impl Quantity {
    pub fn new(raw: Fixed64, dim: UnitDim) -> Self {
        Self { raw, dim }
    }

    pub fn display(&self) -> String {
        let mut out = self.raw.format();
        let unit = format_dim(self.dim);
        if !unit.is_empty() {
            out.push('@');
            out.push_str(&unit);
        }
        out
    }

    pub fn canon(&self) -> String {
        self.display()
    }
}

impl Ord for Quantity {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.dim.cmp(&other.dim) {
            Ordering::Equal => self.raw.raw().cmp(&other.raw.raw()),
            other => other,
        }
    }
}

impl PartialOrd for Quantity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MathValue {
    pub dialect: String,
    pub body: String,
}

impl MathValue {
    pub fn display(&self) -> String {
        format!("({}) 수식{{ {} }}", self.dialect, self.body)
    }

    pub fn canon(&self) -> String {
        self.display()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateValue {
    pub body: String,
}

impl TemplateValue {
    pub fn display(&self) -> String {
        format!("글무늬{{\"{}\"}}", escape_canon_string(&self.body))
    }

    pub fn canon(&self) -> String {
        self.display()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssertionValue {
    pub body_source: String,
    pub canon: String,
}

impl AssertionValue {
    pub fn display(&self) -> String {
        self.canon.clone()
    }

    pub fn canon(&self) -> String {
        self.canon.clone()
    }
}

#[derive(Clone, Debug)]
pub struct LambdaValue {
    pub id: u64,
    pub param: String,
    pub body: Expr,
    pub captured: BTreeMap<String, Value>,
}

impl PartialEq for LambdaValue {
    fn eq(&self, other: &Self) -> bool {
        self.canon() == other.canon()
    }
}

impl Eq for LambdaValue {}

impl LambdaValue {
    pub fn canon(&self) -> String {
        let mut base = String::new();
        base.push_str(&self.param);
        base.push('\n');
        base.push_str(&format!("{:?}", self.body));
        for (key, value) in &self.captured {
            base.push('\n');
            base.push_str(key);
            base.push('=');
            base.push_str(&value.canon());
        }
        let digest = Sha256::digest(base.as_bytes());
        let hash = digest.iter().map(|b| format!("{b:02x}")).collect::<String>();
        format!("<씨앗:{}>", hash)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiceValue {
    pub seed: u64,
    pub state: u64,
    pub draws: u64,
}

impl DiceValue {
    pub fn display(&self) -> String {
        format!(
            "주사위씨{{시앗=0x{seed:016x}, 상태=0x{state:016x}, 횟수={draws}}}",
            seed = self.seed,
            state = self.state,
            draws = self.draws
        )
    }

    pub fn canon(&self) -> String {
        self.display()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackValue {
    pub fields: BTreeMap<String, Value>,
}

impl PackValue {
    pub fn display(&self) -> String {
        if let Some(text) = format_exact_numeric_pack(&self.fields) {
            return text;
        }
        if let Some(text) = format_relation_pack(&self.fields, ValueFormat::Display) {
            return text;
        }
        if let Some(text) = format_relation_solve_result_pack(&self.fields, ValueFormat::Display) {
            return text;
        }
        format_pack(&self.fields, ValueFormat::Display)
    }

    pub fn canon(&self) -> String {
        if let Some(text) = format_exact_numeric_pack(&self.fields) {
            return text;
        }
        if let Some(text) = format_relation_pack(&self.fields, ValueFormat::Canon) {
            return text;
        }
        if let Some(text) = format_relation_solve_result_pack(&self.fields, ValueFormat::Canon) {
            return text;
        }
        format_pack(&self.fields, ValueFormat::Canon)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListValue {
    pub items: Vec<Value>,
}

impl ListValue {
    pub fn display(&self) -> String {
        format_list(&self.items, ValueFormat::Display)
    }

    pub fn canon(&self) -> String {
        format_list(&self.items, ValueFormat::Canon)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetValue {
    pub items: BTreeMap<String, Value>,
}

impl SetValue {
    pub fn display(&self) -> String {
        format_set(&self.items, ValueFormat::Display)
    }

    pub fn canon(&self) -> String {
        format_set(&self.items, ValueFormat::Canon)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapEntry {
    pub key: Value,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapValue {
    pub entries: BTreeMap<String, MapEntry>,
}

impl MapValue {
    pub fn display(&self) -> String {
        format_map(&self.entries, ValueFormat::Display)
    }

    pub fn canon(&self) -> String {
        format_map(&self.entries, ValueFormat::Canon)
    }

    pub fn map_get(&self, key: &Value) -> Value {
        self.entries
            .get(&key.canon())
            .map(|entry| entry.value.clone())
            .unwrap_or(Value::None)
    }

    pub fn map_set(&self, key: Value, value: Value) -> Self {
        let mut entries = self.entries.clone();
        entries.insert(key.canon(), MapEntry { key, value });
        Self { entries }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    None,
    Bool(bool),
    Num(Quantity),
    Str(String),
    ResourceHandle(ResourceHandle),
    Math(MathValue),
    Template(TemplateValue),
    Assertion(AssertionValue),
    Lambda(LambdaValue),
    Dice(DiceValue),
    #[allow(dead_code)]
    Pack(PackValue),
    List(ListValue),
    Set(SetValue),
    Map(MapValue),
}

impl Value {
    pub fn display(&self) -> String {
        match self {
            Value::None => "없음".to_string(),
            Value::Bool(true) => "참".to_string(),
            Value::Bool(false) => "거짓".to_string(),
            Value::Num(qty) => qty.display(),
            Value::Str(text) => text.clone(),
            Value::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
            Value::Math(math) => math.display(),
            Value::Template(template) => template.display(),
            Value::Assertion(assertion) => assertion.display(),
            Value::Lambda(lambda) => lambda.canon(),
            Value::Dice(dice) => dice.display(),
            Value::Pack(pack) => pack.display(),
            Value::List(list) => list.display(),
            Value::Set(set) => set.display(),
            Value::Map(map) => map.display(),
        }
    }

    pub fn canon(&self) -> String {
        match self {
            Value::None => "없음".to_string(),
            Value::Bool(true) => "참".to_string(),
            Value::Bool(false) => "거짓".to_string(),
            Value::Num(qty) => qty.canon(),
            Value::Str(text) => format!("\"{}\"", escape_canon_string(text)),
            Value::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
            Value::Math(math) => math.display(),
            Value::Template(template) => template.canon(),
            Value::Assertion(assertion) => assertion.canon(),
            Value::Lambda(lambda) => lambda.canon(),
            Value::Dice(dice) => dice.canon(),
            Value::Pack(pack) => pack.canon(),
            Value::List(list) => list.canon(),
            Value::Set(set) => set.canon(),
            Value::Map(map) => map.canon(),
        }
    }

    #[allow(dead_code)]
    pub fn tag(&self) -> u8 {
        match self {
            Value::None => 0x00,
            Value::Bool(_) => 0x01,
            Value::Num(_) => 0x02,
            Value::Str(_) => 0x03,
            Value::ResourceHandle(_) => 0x04,
            Value::Math(_) => 0x05,
            Value::Template(_) => 0x06,
            Value::Assertion(_) => 0x07,
            Value::Pack(_) => 0x08,
            Value::List(_) => 0x09,
            Value::Set(_) => 0x0A,
            Value::Map(_) => 0x0B,
            Value::Lambda(_) => 0x0C,
            Value::Dice(_) => 0x0D,
        }
    }
}

fn escape_canon_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::None, Value::None) => Ordering::Equal,
            (Value::None, _) => Ordering::Less,
            (_, Value::None) => Ordering::Greater,
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Bool(_), _) => Ordering::Less,
            (_, Value::Bool(_)) => Ordering::Greater,
            (Value::Num(a), Value::Num(b)) => a.cmp(b),
            (Value::Num(_), _) => Ordering::Less,
            (_, Value::Num(_)) => Ordering::Greater,
            (Value::Str(a), Value::Str(b)) => a.cmp(b),
            (Value::Str(_), _) => Ordering::Less,
            (_, Value::Str(_)) => Ordering::Greater,
            (Value::ResourceHandle(a), Value::ResourceHandle(b)) => a.cmp(b),
            (Value::ResourceHandle(_), _) => Ordering::Less,
            (_, Value::ResourceHandle(_)) => Ordering::Greater,
            (Value::Math(a), Value::Math(b)) => a.body.cmp(&b.body),
            (Value::Math(_), _) => Ordering::Less,
            (_, Value::Math(_)) => Ordering::Greater,
            (Value::Template(a), Value::Template(b)) => a.body.cmp(&b.body),
            (Value::Template(_), _) => Ordering::Less,
            (_, Value::Template(_)) => Ordering::Greater,
            (Value::Assertion(a), Value::Assertion(b)) => a.canon.cmp(&b.canon),
            (Value::Assertion(_), _) => Ordering::Less,
            (_, Value::Assertion(_)) => Ordering::Greater,
            (Value::Lambda(a), Value::Lambda(b)) => a.id.cmp(&b.id),
            (Value::Lambda(_), _) => Ordering::Less,
            (_, Value::Lambda(_)) => Ordering::Greater,
            (Value::Dice(a), Value::Dice(b)) => compare_dice(a, b),
            (Value::Dice(_), _) => Ordering::Less,
            (_, Value::Dice(_)) => Ordering::Greater,
            (Value::Pack(a), Value::Pack(b)) => compare_pack(a, b),
            (Value::Pack(_), _) => Ordering::Less,
            (_, Value::Pack(_)) => Ordering::Greater,
            (Value::List(a), Value::List(b)) => compare_list(a, b),
            (Value::List(_), _) => Ordering::Less,
            (_, Value::List(_)) => Ordering::Greater,
            (Value::Set(a), Value::Set(b)) => compare_set(a, b),
            (Value::Set(_), _) => Ordering::Less,
            (_, Value::Set(_)) => Ordering::Greater,
            (Value::Map(a), Value::Map(b)) => compare_map(a, b),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy)]
enum ValueFormat {
    Display,
    Canon,
}

fn format_pack(fields: &BTreeMap<String, Value>, mode: ValueFormat) -> String {
    let mut out = String::from("묶음{");
    let mut first = true;
    for (key, value) in fields {
        if !first {
            out.push_str(", ");
        }
        first = false;
        out.push_str(key);
        out.push('=');
        let rendered = match mode {
            ValueFormat::Display => value.display(),
            ValueFormat::Canon => value.canon(),
        };
        out.push_str(&rendered);
    }
    out.push('}');
    out
}

fn format_exact_numeric_pack(fields: &BTreeMap<String, Value>) -> Option<String> {
    let kind = match fields.get("__정확수종류")? {
        Value::Str(text) => text.as_str(),
        _ => return None,
    };
    match kind {
        "큰바른수" => match fields.get("값") {
            Some(Value::Str(text)) => Some(text.clone()),
            _ => Some("[큰바른수]".to_string()),
        },
        "나눔수" => {
            let num = match fields.get("분자") {
                Some(Value::Str(text)) => text.as_str(),
                _ => "?",
            };
            let den = match fields.get("분모") {
                Some(Value::Str(text)) => text.as_str(),
                _ => "?",
            };
            Some(format!("{num}/{den}"))
        }
        "곱수" => match fields.get("정본") {
            Some(Value::Str(text)) => Some(text.clone()),
            _ => match fields.get("값") {
                Some(Value::Str(text)) => Some(text.clone()),
                _ => Some("[곱수]".to_string()),
            },
        },
        _ => None,
    }
}

fn format_relation_pack(fields: &BTreeMap<String, Value>, mode: ValueFormat) -> Option<String> {
    let kind = match fields.get("__관계종류")? {
        Value::Str(text) => text.as_str(),
        _ => return None,
    };
    if kind != "방정식" {
        return None;
    }
    let lhs = match fields.get("왼쪽")? {
        Value::Math(value) => match mode {
            ValueFormat::Display => value.display(),
            ValueFormat::Canon => value.canon(),
        },
        _ => return None,
    };
    let rhs = match fields.get("오른쪽")? {
        Value::Math(value) => match mode {
            ValueFormat::Display => value.display(),
            ValueFormat::Canon => value.canon(),
        },
        _ => return None,
    };
    Some(format!("{lhs} =:= {rhs}"))
}

fn format_relation_solve_result_pack(
    fields: &BTreeMap<String, Value>,
    mode: ValueFormat,
) -> Option<String> {
    let kind = match fields.get("__풀이결과종류")? {
        Value::Str(text) => text.as_str(),
        _ => return None,
    };
    match kind {
        "성공" => {
            if let (Some(Value::Str(variable)), Some(value)) = (fields.get("미지수"), fields.get("값")) {
                let rendered = match mode {
                    ValueFormat::Display => value.display(),
                    ValueFormat::Canon => value.canon(),
                };
                return Some(format!("#성공(미지수=\"{variable}\", 값={rendered})"));
            }
            let Value::Pack(bindings) = fields.get("해")? else {
                return None;
            };
            let rendered = bindings
                .fields
                .iter()
                .map(|(key, value)| {
                    let text = match mode {
                        ValueFormat::Display => value.display(),
                        ValueFormat::Canon => value.canon(),
                    };
                    format!("{key}={text}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("#성공(해=({rendered}))"))
        }
        "실패" => {
            let reason = match fields.get("사유")? {
                Value::Str(text) => text.clone(),
                _ => return None,
            };
            Some(format!("#실패(사유=\"{reason}\")"))
        }
        _ => None,
    }
}

fn format_list(items: &[Value], mode: ValueFormat) -> String {
    let mut out = String::from("차림[");
    let mut first = true;
    for item in items {
        if !first {
            out.push_str(", ");
        }
        first = false;
        let rendered = match mode {
            ValueFormat::Display => item.display(),
            ValueFormat::Canon => item.canon(),
        };
        out.push_str(&rendered);
    }
    out.push(']');
    out
}

fn format_set(items: &BTreeMap<String, Value>, mode: ValueFormat) -> String {
    let mut out = String::from("모음{");
    let mut first = true;
    for (_canon_key, value) in items {
        if !first {
            out.push_str(", ");
        }
        first = false;
        let rendered = match mode {
            ValueFormat::Display => value.display(),
            ValueFormat::Canon => value.canon(),
        };
        out.push_str(&rendered);
    }
    out.push('}');
    out
}

fn format_map(entries: &BTreeMap<String, MapEntry>, mode: ValueFormat) -> String {
    let mut out = String::from("짝맞춤{");
    let mut first = true;
    for (_canon_key, entry) in entries {
        if !first {
            out.push_str(", ");
        }
        first = false;
        let key_rendered = match mode {
            ValueFormat::Display => entry.key.display(),
            ValueFormat::Canon => entry.key.canon(),
        };
        let value_rendered = match mode {
            ValueFormat::Display => entry.value.display(),
            ValueFormat::Canon => entry.value.canon(),
        };
        out.push_str(&key_rendered);
        out.push_str("=>");
        out.push_str(&value_rendered);
    }
    out.push('}');
    out
}

fn compare_pack(a: &PackValue, b: &PackValue) -> Ordering {
    let mut iter_a = a.fields.iter();
    let mut iter_b = b.fields.iter();
    loop {
        match (iter_a.next(), iter_b.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some((ka, va)), Some((kb, vb))) => match ka.cmp(kb) {
                Ordering::Equal => match va.cmp(vb) {
                    Ordering::Equal => continue,
                    other => return other,
                },
                other => return other,
            },
        }
    }
}

fn compare_dice(a: &DiceValue, b: &DiceValue) -> Ordering {
    match a.seed.cmp(&b.seed) {
        Ordering::Equal => match a.state.cmp(&b.state) {
            Ordering::Equal => a.draws.cmp(&b.draws),
            other => other,
        },
        other => other,
    }
}

fn compare_list(a: &ListValue, b: &ListValue) -> Ordering {
    let mut iter_a = a.items.iter();
    let mut iter_b = b.items.iter();
    loop {
        match (iter_a.next(), iter_b.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(va), Some(vb)) => match va.cmp(vb) {
                Ordering::Equal => continue,
                other => return other,
            },
        }
    }
}

fn compare_set(a: &SetValue, b: &SetValue) -> Ordering {
    let mut iter_a = a.items.iter();
    let mut iter_b = b.items.iter();
    loop {
        match (iter_a.next(), iter_b.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some((ka, va)), Some((kb, vb))) => match ka.cmp(kb) {
                Ordering::Equal => match va.cmp(vb) {
                    Ordering::Equal => continue,
                    other => return other,
                },
                other => return other,
            },
        }
    }
}

fn compare_map(a: &MapValue, b: &MapValue) -> Ordering {
    let mut iter_a = a.entries.iter();
    let mut iter_b = b.entries.iter();
    loop {
        match (iter_a.next(), iter_b.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some((ka, ea)), Some((kb, eb))) => match ka.cmp(kb) {
                Ordering::Equal => match ea.value.cmp(&eb.value) {
                    Ordering::Equal => continue,
                    other => return other,
                },
                other => return other,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_get_returns_value_or_none() {
        let key = Value::Str("name".to_string());
        let map = MapValue {
            entries: BTreeMap::from([(
                key.canon(),
                MapEntry {
                    key: key.clone(),
                    value: Value::Str("ddn".to_string()),
                },
            )]),
        };
        assert_eq!(map.map_get(&key), Value::Str("ddn".to_string()));
        assert_eq!(map.map_get(&Value::Str("missing".to_string())), Value::None);
    }

    #[test]
    fn map_set_overwrites_existing_key() {
        let key = Value::Str("k".to_string());
        let map = MapValue {
            entries: BTreeMap::new(),
        };
        let map = map.map_set(key.clone(), Value::Bool(true));
        let map = map.map_set(key.clone(), Value::Bool(false));
        assert_eq!(map.map_get(&key), Value::Bool(false));
    }
}
