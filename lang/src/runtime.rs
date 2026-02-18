use std::collections::{BTreeMap, HashMap};
use ddonirang_core::{Fixed64, ResourceHandle, UnitDim, UnitValue, is_key_just_pressed, is_key_pressed};
use crate::ast::{Expr, Formula, Template};

#[derive(Debug, Clone, PartialEq)]
pub struct MapEntry {
    pub key: Value,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct LambdaValue {
    pub id: u64,
    pub param: String,
    pub body: Expr,
    pub captured: HashMap<String, Value>,
}

impl PartialEq for LambdaValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Fixed64(Fixed64),
    Unit(UnitValue),
    String(String),
    ResourceHandle(ResourceHandle),
    List(Vec<Value>),
    Set(BTreeMap<String, Value>),
    Map(BTreeMap<String, MapEntry>),
    Pack(BTreeMap<String, Value>),
    Formula(Formula),
    Template(Template),
    Lambda(LambdaValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    TypeMismatch { expected: &'static str },
    IndexOutOfRange,
}

#[derive(Debug, Clone, Copy)]
pub struct InputState {
    pub keys_pressed: u64,
    pub prev_keys_pressed: u64,
}

impl InputState {
    pub fn new(keys_pressed: u64, prev_keys_pressed: u64) -> Self {
        Self {
            keys_pressed,
            prev_keys_pressed,
        }
    }
}

pub fn input_pressed(state: &InputState, key: &str) -> Value {
    Value::Bool(is_key_pressed(state.keys_pressed, key))
}

pub fn input_just_pressed(state: &InputState, key: &str) -> Value {
    Value::Bool(is_key_just_pressed(
        state.prev_keys_pressed,
        state.keys_pressed,
        key,
    ))
}

pub fn list_new(values: Vec<Value>) -> Value {
    Value::List(values)
}

pub fn list_len(list: &Value) -> Result<Value, RuntimeError> {
    match list {
        Value::List(items) => Ok(Value::Fixed64(Fixed64::from_i64(items.len() as i64))),
        _ => Err(RuntimeError::TypeMismatch { expected: "차림" }),
    }
}

pub fn list_nth(list: &Value, index: &Value) -> Result<Value, RuntimeError> {
    let idx = parse_index(index)?;
    match list {
        Value::List(items) => Ok(items.get(idx).cloned().unwrap_or(Value::None)),
        _ => Err(RuntimeError::TypeMismatch { expected: "차림" }),
    }
}

pub fn list_add(list: &Value, value: Value) -> Result<Value, RuntimeError> {
    match list {
        Value::List(items) => {
            let mut out = items.clone();
            out.push(value);
            Ok(Value::List(out))
        }
        _ => Err(RuntimeError::TypeMismatch { expected: "차림" }),
    }
}

pub fn list_remove(list: &Value, index: &Value) -> Result<Value, RuntimeError> {
    let idx = parse_index(index)?;
    match list {
        Value::List(items) => {
            if idx >= items.len() {
                return Ok(Value::List(items.clone()));
            }
            let mut out = items.clone();
            out.remove(idx);
            Ok(Value::List(out))
        }
        _ => Err(RuntimeError::TypeMismatch { expected: "차림" }),
    }
}

pub fn list_set(list: &Value, index: &Value, value: Value) -> Result<Value, RuntimeError> {
    let idx = parse_index(index)?;
    match list {
        Value::List(items) => {
            if idx == usize::MAX || idx >= items.len() {
                return Err(RuntimeError::IndexOutOfRange);
            }
            let mut out = items.clone();
            out[idx] = value;
            Ok(Value::List(out))
        }
        _ => Err(RuntimeError::TypeMismatch { expected: "차림" }),
    }
}

pub fn map_get(map: &BTreeMap<String, MapEntry>, key: &Value) -> Value {
    map.get(&map_key_canon(key))
        .map(|entry| entry.value.clone())
        .unwrap_or(Value::None)
}

pub fn map_key_canon(value: &Value) -> String {
    match value {
        Value::None => "없음".to_string(),
        Value::Bool(true) => "참".to_string(),
        Value::Bool(false) => "거짓".to_string(),
        Value::Fixed64(n) => n.to_string(),
        Value::Unit(unit) => {
            let suffix = unit
                .display_symbol()
                .map(|s| s.to_string())
                .unwrap_or_else(|| unit.dim.format());
            format!("{}@{}", unit.value, suffix)
        }
        Value::String(s) => format!("\"{}\"", escape_canon_string(s)),
        Value::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
        Value::List(items) => {
            let mut out = String::from("차림[");
            let mut first = true;
            for item in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&map_key_canon(item));
            }
            out.push(']');
            out
        }
        Value::Set(items) => {
            let mut out = String::from("모음{");
            let mut first = true;
            for item in items.values() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&map_key_canon(item));
            }
            out.push('}');
            out
        }
        Value::Map(entries) => {
            let mut out = String::from("짝맞춤{");
            let mut first = true;
            for entry in entries.values() {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&map_key_canon(&entry.key));
                out.push_str("=>");
                out.push_str(&map_key_canon(&entry.value));
            }
            out.push('}');
            out
        }
        Value::Pack(items) => {
            let mut out = String::from("묶음{");
            let mut first = true;
            for (key, item) in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(key);
                out.push('=');
                out.push_str(&map_key_canon(item));
            }
            out.push('}');
            out
        }
        Value::Formula(formula) => formula.raw.clone(),
        Value::Template(template) => template.raw.clone(),
        Value::Lambda(lambda) => format!("<씨앗#{}>", lambda.id),
    }
}

pub fn string_len(value: &Value) -> Result<Value, RuntimeError> {
    match value {
        Value::String(s) => Ok(Value::Fixed64(Fixed64::from_i64(s.chars().count() as i64))),
        _ => Err(RuntimeError::TypeMismatch { expected: "글" }),
    }
}

pub fn string_concat(left: &Value, right: &Value) -> Result<Value, RuntimeError> {
    match (left, right) {
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
        _ => Err(RuntimeError::TypeMismatch { expected: "글" }),
    }
}

pub fn string_split(value: &Value, delim: &Value) -> Result<Value, RuntimeError> {
    match (value, delim) {
        (Value::String(s), Value::String(d)) => {
            let parts = if d.is_empty() {
                s.chars().map(|c| Value::String(c.to_string())).collect()
            } else {
                s.split(d).map(|p| Value::String(p.to_string())).collect()
            };
            Ok(Value::List(parts))
        }
        _ => Err(RuntimeError::TypeMismatch { expected: "글" }),
    }
}

pub fn string_join(list: &Value, delim: &Value) -> Result<Value, RuntimeError> {
    match (list, delim) {
        (Value::List(items), Value::String(d)) => {
            let mut out = String::new();
            for (i, item) in items.iter().enumerate() {
                let Value::String(s) = item else {
                    return Err(RuntimeError::TypeMismatch { expected: "차림<글>" });
                };
                if i > 0 {
                    out.push_str(d);
                }
                out.push_str(s);
            }
            Ok(Value::String(out))
        }
        _ => Err(RuntimeError::TypeMismatch { expected: "차림<글>" }),
    }
}

pub fn string_contains(value: &Value, pattern: &Value) -> Result<Value, RuntimeError> {
    match (value, pattern) {
        (Value::String(s), Value::String(pat)) => Ok(Value::Bool(s.contains(pat))),
        _ => Err(RuntimeError::TypeMismatch { expected: "글" }),
    }
}

pub fn string_starts(value: &Value, pattern: &Value) -> Result<Value, RuntimeError> {
    match (value, pattern) {
        (Value::String(s), Value::String(pat)) => Ok(Value::Bool(s.starts_with(pat))),
        _ => Err(RuntimeError::TypeMismatch { expected: "글" }),
    }
}

pub fn string_ends(value: &Value, pattern: &Value) -> Result<Value, RuntimeError> {
    match (value, pattern) {
        (Value::String(s), Value::String(pat)) => Ok(Value::Bool(s.ends_with(pat))),
        _ => Err(RuntimeError::TypeMismatch { expected: "글" }),
    }
}

pub fn string_to_number(value: &Value) -> Result<Value, RuntimeError> {
    let Value::String(text) = value else {
        return Err(RuntimeError::TypeMismatch { expected: "글" });
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Value::None);
    }
    match trimmed.parse::<f64>() {
        Ok(parsed) => Ok(Value::Fixed64(Fixed64::from_f64_lossy(parsed))),
        Err(_) => Ok(Value::None),
    }
}

fn parse_index(index: &Value) -> Result<usize, RuntimeError> {
    match index {
        Value::Fixed64(n) => {
            let idx = n.int_part();
            if idx < 0 {
                Ok(usize::MAX)
            } else {
                Ok(idx as usize)
            }
        }
        Value::Unit(unit) if unit.dim == UnitDim::NONE => {
            let idx = unit.value.int_part();
            if idx < 0 {
                Ok(usize::MAX)
            } else {
                Ok(idx as usize)
            }
        }
        _ => Err(RuntimeError::TypeMismatch { expected: "정수" }),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::{KEY_A, KEY_W};

    #[test]
    fn runtime_input_pressed_reads_bits() {
        let state = InputState::new(KEY_W, 0);
        assert_eq!(input_pressed(&state, "w"), Value::Bool(true));
        assert_eq!(input_pressed(&state, "a"), Value::Bool(false));
    }

    #[test]
    fn runtime_input_just_pressed_uses_prev() {
        let state = InputState::new(KEY_W | KEY_A, KEY_W);
        assert_eq!(input_just_pressed(&state, "a"), Value::Bool(true));
        assert_eq!(input_just_pressed(&state, "w"), Value::Bool(false));
    }

    #[test]
    fn runtime_list_ops_work() {
        let list = list_new(vec![Value::String("a".to_string())]);
        let list = list_add(&list, Value::String("b".to_string())).expect("add");
        let len = list_len(&list).expect("len");
        assert_eq!(len, Value::Fixed64(Fixed64::from_i64(2)));
        let second = list_nth(&list, &Value::Fixed64(Fixed64::from_i64(1))).expect("nth");
        assert_eq!(second, Value::String("b".to_string()));
        let removed = list_remove(&list, &Value::Fixed64(Fixed64::from_i64(0))).expect("remove");
        let Value::List(items) = removed else { panic!("expected list") };
        assert_eq!(items, vec![Value::String("b".to_string())]);
    }

    #[test]
    fn runtime_string_ops_work() {
        let a = Value::String("ha".to_string());
        let b = Value::String("ha".to_string());
        let joined = string_concat(&a, &b).expect("concat");
        assert_eq!(joined, Value::String("haha".to_string()));
        let parts = string_split(&joined, &Value::String("a".to_string())).expect("split");
        let Value::List(items) = parts else { panic!("expected list") };
        assert!(!items.is_empty());
        let rejoin = string_join(&Value::List(items), &Value::String("a".to_string())).expect("join");
        assert_eq!(rejoin, joined);
    }

    #[test]
    fn runtime_map_get_uses_canonical_key_and_returns_none_when_missing() {
        let key = Value::String("이름".to_string());
        let mut map = BTreeMap::new();
        map.insert(
            map_key_canon(&key),
            MapEntry {
                key: key.clone(),
                value: Value::String("또니".to_string()),
            },
        );
        assert_eq!(map_get(&map, &key), Value::String("또니".to_string()));
        assert_eq!(map_get(&map, &Value::String("없는키".to_string())), Value::None);
    }
}
