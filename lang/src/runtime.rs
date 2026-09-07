use crate::ast::{Assertion, Expr, Formula, RegexLiteral, StateMachine, Template, TypeRef};
use ddonirang_core::{
    is_key_just_pressed, is_key_pressed, unit_spec_from_symbol, Fixed64, ResourceHandle, UnitDim,
    UnitValue,
};
use std::collections::{BTreeMap, HashMap};

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
    Assertion(Assertion),
    StateMachine(StateMachine),
    Formula(Formula),
    Template(Template),
    Regex(RegexLiteral),
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
        Value::List(items) => items
            .get(idx)
            .cloned()
            .ok_or(RuntimeError::IndexOutOfRange),
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
        Value::Assertion(assertion) => assertion.canon.clone(),
        Value::StateMachine(machine) => {
            let mut out = String::from("상태머신{");
            out.push_str(&machine.states.join(", "));
            out.push_str(" 으로 이뤄짐.");
            out.push(' ');
            out.push_str(&machine.initial);
            out.push_str(" 으로 시작.");
            for transition in &machine.transitions {
                out.push(' ');
                out.push_str(&transition.from);
                out.push_str(" 에서 ");
                out.push_str(&transition.to);
                out.push_str(" 으로");
                if let Some(guard_name) = &transition.guard_name {
                    out.push_str(" 걸러서 ");
                    out.push_str(guard_name);
                }
                if let Some(action_name) = &transition.action_name {
                    out.push_str(" 하고 ");
                    out.push_str(action_name);
                }
                out.push('.');
            }
            for check in &machine.on_transition_checks {
                out.push(' ');
                out.push_str("바뀔때마다 ");
                out.push_str(check);
                out.push_str(" 살피기.");
            }
            out.push('}');
            out
        }
        Value::Formula(formula) => formula.raw.clone(),
        Value::Template(template) => template.raw.clone(),
        Value::Regex(regex) => {
            if regex.flags.is_empty() {
                format!("정규식{{\"{}\"}}", regex.pattern)
            } else {
                format!("정규식{{\"{}\", \"{}\"}}", regex.pattern, regex.flags)
            }
        }
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
                    return Err(RuntimeError::TypeMismatch {
                        expected: "차림<글>",
                    });
                };
                if i > 0 {
                    out.push_str(d);
                }
                out.push_str(s);
            }
            Ok(Value::String(out))
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "차림<글>",
        }),
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
    match Fixed64::parse_decimal(trimmed) {
        Some(parsed) => Ok(Value::Fixed64(parsed)),
        None => Ok(Value::None),
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
        let Value::List(items) = removed else {
            panic!("expected list")
        };
        assert_eq!(items, vec![Value::String("b".to_string())]);
    }

    #[test]
    fn runtime_list_nth_out_of_range_fails_closed() {
        let list = list_new(vec![Value::String("a".to_string())]);
        let error = list_nth(&list, &Value::Fixed64(Fixed64::from_i64(1)))
            .expect_err("strict selection must not return 없음");
        assert_eq!(error, RuntimeError::IndexOutOfRange);
    }

    #[test]
    fn runtime_string_ops_work() {
        let a = Value::String("ha".to_string());
        let b = Value::String("ha".to_string());
        let joined = string_concat(&a, &b).expect("concat");
        assert_eq!(joined, Value::String("haha".to_string()));
        let parts = string_split(&joined, &Value::String("a".to_string())).expect("split");
        let Value::List(items) = parts else {
            panic!("expected list")
        };
        assert!(!items.is_empty());
        let rejoin =
            string_join(&Value::List(items), &Value::String("a".to_string())).expect("join");
        assert_eq!(rejoin, joined);
    }

    #[test]
    fn runtime_string_to_number_keeps_q32_32_raw_value() {
        assert_eq!(
            string_to_number(&Value::String(" 3.14 ".to_string())),
            Ok(Value::Fixed64(Fixed64::from_raw_i64(13_486_197_309)))
        );
        assert_eq!(
            string_to_number(&Value::String("not-a-number".to_string())),
            Ok(Value::None)
        );
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
        assert_eq!(
            map_get(&map, &Value::String("없는키".to_string())),
            Value::None
        );
    }

    #[test]
    fn runtime_value_type_authority_checks_nested_collection_identity() {
        let value = Value::List(vec![
            Value::Fixed64(Fixed64::from_i64(1)),
            Value::String("둘".to_string()),
        ]);
        let expected = TypeRef::Applied {
            name: "차림".to_string(),
            args: vec![TypeRef::Named("수".to_string())],
        };
        let mismatch = check_value_type(&value, &expected).expect_err("mixed list must fail");
        assert_eq!(mismatch.expected, "(수)차림");
        assert_eq!(mismatch.actual, "차림(요소: 글)");
        assert_eq!(value_type_name(&value), "차림");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMismatchDetail {
    pub expected: String,
    pub actual: String,
}

const NUMERIC_PACK_KIND_KEY: &str = "__수타입";
const NUMERIC_KIND_BIG_INT: &str = "큰바른수";
const NUMERIC_KIND_RATIONAL: &str = "나눔수";
const NUMERIC_KIND_FACTOR: &str = "곱수";

pub fn check_value_type(value: &Value, type_ref: &TypeRef) -> Result<(), TypeMismatchDetail> {
    match type_ref {
        TypeRef::Infer => Ok(()),
        TypeRef::Named(name) => check_named_type(value, name),
        TypeRef::Applied { name, args } => check_applied_type(value, name, args),
        TypeRef::Function { .. } => Err(type_mismatch_detail(&format_type_ref(type_ref), value)),
    }
}

fn check_named_type(value: &Value, name: &str) -> Result<(), TypeMismatchDetail> {
    if let Some((base, unit)) = name.split_once('@') {
        return check_unit_type(value, base, unit);
    }
    let canonical = canonical_type_name(name);
    match canonical.as_str() {
        "값" => Ok(()),
        "수" | "셈수" => match value {
            Value::Fixed64(_) => Ok(()),
            Value::Unit(unit) if unit.is_dimensionless() => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "바른수" => match value {
            Value::Fixed64(n) if n.frac_part() == 0 => Ok(()),
            Value::Unit(unit) if unit.is_dimensionless() && unit.value.frac_part() == 0 => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "글" => match value {
            Value::String(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "참거짓" => match value {
            Value::Bool(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "없음" => match value {
            Value::None => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "세움값" => match value {
            Value::Assertion(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "상태머신값" => match value {
            Value::StateMachine(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "차림" => match value {
            Value::List(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "모음" => match value {
            Value::Set(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "짝맞춤" => match value {
            Value::Map(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "묶음" => match value {
            Value::Pack(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "큰바른수" => check_numeric_type(value, &canonical, NUMERIC_KIND_BIG_INT),
        "나눔수" => check_numeric_type(value, &canonical, NUMERIC_KIND_RATIONAL),
        "곱수" => check_numeric_type(value, &canonical, NUMERIC_KIND_FACTOR),
        "자원" | "자원핸들" => match value {
            Value::ResourceHandle(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "수식값" => match value {
            Value::Formula(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "글무늬값" => match value {
            Value::Template(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "정규식" => match value {
            Value::Regex(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        _ => Err(type_mismatch_detail(&canonical, value)),
    }
}

fn check_numeric_type(
    value: &Value,
    canonical: &str,
    numeric_kind: &str,
) -> Result<(), TypeMismatchDetail> {
    if numeric_pack_kind(value) == Some(numeric_kind) {
        return Ok(());
    }
    match value {
        Value::Fixed64(n) if n.frac_part() == 0 => Ok(()),
        Value::Unit(unit) if unit.is_dimensionless() && unit.value.frac_part() == 0 => Ok(()),
        _ => Err(type_mismatch_detail(canonical, value)),
    }
}

fn check_applied_type(
    value: &Value,
    name: &str,
    args: &[TypeRef],
) -> Result<(), TypeMismatchDetail> {
    let canonical = canonical_type_name(name);
    match canonical.as_str() {
        "수" if args.len() == 1 => {
            if let TypeRef::Named(unit) = &args[0] {
                return check_unit_type(value, "수", unit);
            }
            Err(type_mismatch_detail("수", value))
        }
        "차림" => {
            let Value::List(items) = value else {
                return Err(type_mismatch_detail(&canonical, value));
            };
            if args.len() != 1 {
                return Ok(());
            }
            for item in items {
                if let Err(detail) = check_value_type(item, &args[0]) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "차림".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("차림(요소: {})", detail.actual),
                    });
                }
            }
            Ok(())
        }
        "모음" => {
            let Value::Set(items) = value else {
                return Err(type_mismatch_detail(&canonical, value));
            };
            if args.len() != 1 {
                return Ok(());
            }
            for item in items.values() {
                if let Err(detail) = check_value_type(item, &args[0]) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "모음".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("모음(요소: {})", detail.actual),
                    });
                }
            }
            Ok(())
        }
        "짝맞춤" => {
            let Value::Map(entries) = value else {
                return Err(type_mismatch_detail(&canonical, value));
            };
            if args.len() != 2 {
                return Ok(());
            }
            for entry in entries.values() {
                if let Err(detail) = check_value_type(&entry.key, &args[0]) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "짝맞춤".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("짝맞춤(열쇠: {})", detail.actual),
                    });
                }
                if let Err(detail) = check_value_type(&entry.value, &args[1]) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "짝맞춤".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("짝맞춤(값: {})", detail.actual),
                    });
                }
            }
            Ok(())
        }
        _ => Err(type_mismatch_detail(&canonical, value)),
    }
}

fn check_unit_type(value: &Value, base: &str, unit: &str) -> Result<(), TypeMismatchDetail> {
    let canonical_base = canonical_type_name(base);
    if canonical_base != "수" {
        return Err(type_mismatch_detail(&canonical_base, value));
    }
    let Some(spec) = unit_spec_from_symbol(unit) else {
        return Err(type_mismatch_detail(&format!("수@{}", unit), value));
    };
    match value {
        Value::Unit(unit_value) if unit_value.dim == spec.dim => Ok(()),
        _ => Err(type_mismatch_detail(&format!("수@{}", unit), value)),
    }
}

fn canonical_type_name(name: &str) -> String {
    crate::stdlib::canonicalize_type_alias(name).to_string()
}

fn format_type_ref(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name) => canonical_type_name(name),
        TypeRef::Applied { name, args } => {
            let name = canonical_type_name(name);
            if name == "수" && args.len() == 1 {
                if let TypeRef::Named(unit) = &args[0] {
                    return format!("수@{}", unit);
                }
            }
            let args_out: Vec<String> = args.iter().map(format_type_ref).collect();
            format!("({}){}", args_out.join(", "), name)
        }
        TypeRef::Function { params, result } => {
            let params_out: Vec<String> = params.iter().map(format_type_ref).collect();
            format!("({}) --> {}", params_out.join(", "), format_type_ref(result))
        }
        TypeRef::Infer => "_".to_string(),
    }
}

fn type_mismatch_detail(expected: &str, actual: &Value) -> TypeMismatchDetail {
    TypeMismatchDetail {
        expected: expected.to_string(),
        actual: value_type_name(actual),
    }
}

fn numeric_pack_kind(value: &Value) -> Option<&str> {
    let Value::Pack(fields) = value else {
        return None;
    };
    let Value::String(kind) = fields.get(NUMERIC_PACK_KIND_KEY)? else {
        return None;
    };
    Some(kind.as_str())
}

pub fn value_type_name(value: &Value) -> String {
    match value {
        Value::None => "없음".to_string(),
        Value::Bool(_) => "참거짓".to_string(),
        Value::Fixed64(n) => {
            if n.frac_part() == 0 {
                "바른수".to_string()
            } else {
                "수".to_string()
            }
        }
        Value::Unit(unit) => {
            if unit.is_dimensionless() {
                if unit.value.frac_part() == 0 {
                    "바른수".to_string()
                } else {
                    "수".to_string()
                }
            } else {
                format!("수@{}", unit.dim.format())
            }
        }
        Value::String(_) => "글".to_string(),
        Value::ResourceHandle(_) => "자원핸들".to_string(),
        Value::List(_) => "차림".to_string(),
        Value::Set(_) => "모음".to_string(),
        Value::Map(_) => "짝맞춤".to_string(),
        Value::Pack(_) => numeric_pack_kind(value)
            .map(str::to_string)
            .unwrap_or_else(|| "묶음".to_string()),
        Value::Assertion(_) => "세움값".to_string(),
        Value::StateMachine(_) => "상태머신값".to_string(),
        Value::Formula(_) => "수식값".to_string(),
        Value::Template(_) => "글무늬값".to_string(),
        Value::Regex(_) => "정규식".to_string(),
        Value::Lambda(_) => "씨앗".to_string(),
    }
}
