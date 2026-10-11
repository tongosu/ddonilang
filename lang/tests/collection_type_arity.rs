use std::collections::BTreeMap;

use ddonirang_core::Fixed64;
use ddonirang_lang::runtime::{check_value_type, MapEntry, Value};
use ddonirang_lang::TypeRef;

fn applied(name: &str, count: usize) -> TypeRef {
    TypeRef::Applied {
        name: name.to_string(),
        args: vec![TypeRef::Named("수".to_string()); count],
    }
}

#[test]
fn collection_type_arguments_cannot_silently_disable_element_checks() {
    let number = Value::Fixed64(Fixed64::from_i64(1));
    for (name, value, arity) in [
        ("차림", Value::List(vec![number.clone()]), 1),
        (
            "모음",
            Value::Set(BTreeMap::from([("1".to_string(), number.clone())])),
            1,
        ),
        (
            "짝맞춤",
            Value::Map(BTreeMap::from([(
                "1".to_string(),
                MapEntry {
                    key: number.clone(),
                    value: number.clone(),
                },
            )])),
            2,
        ),
        ("차림", Value::List(vec![]), 1),
        ("모음", Value::Set(BTreeMap::new()), 1),
        ("짝맞춤", Value::Map(BTreeMap::new()), 2),
    ] {
        for count in 0..=3 {
            assert_eq!(
                check_value_type(&value, &applied(name, count)).is_ok(),
                count == arity,
                "{name} with {count} arguments and value {value:?}"
            );
        }
        // A named collection without explicit arguments retains its existing contract.
        assert!(check_value_type(&value, &TypeRef::Named(name.to_string())).is_ok());
    }
}

#[test]
fn valid_collection_arity_still_checks_nested_element_types() {
    let mixed = Value::List(vec![Value::String("글".to_string())]);
    assert!(check_value_type(&mixed, &applied("차림", 1)).is_err());
    let nested = TypeRef::Applied {
        name: "차림".to_string(),
        args: vec![applied("차림", 2)],
    };
    assert!(check_value_type(&Value::List(vec![Value::List(vec![])]), &nested).is_err());
}
