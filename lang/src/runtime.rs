use crate::ast::{Assertion, Expr, Formula, RegexLiteral, StateMachine, Template, TypeRef};
use ddonirang_core::{
    is_key_just_pressed, is_key_pressed, unit_spec_from_symbol, Fixed64, ResourceHandle, UnitDim,
    UnitValue,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[cfg(test)]
mod canonical_decimal_tests {
    use super::*;

    #[test]
    fn vc13_tuple_category_order_clone_and_key_are_independent() {
        let number=Value::Fixed64(Fixed64::from_i64(1));
        let tuple=Value::Tuple(vec![number.clone(),Value::String("글".into())]);
        assert_eq!(tuple,tuple.clone());
        assert_ne!(tuple,Value::Tuple(vec![Value::String("글".into()),number.clone()]));
        assert_ne!(tuple,Value::List(vec![number,Value::String("글".into())]));
        assert_eq!(map_key_canon(&tuple),"(1, \"글\")");
        assert_eq!(map_key_canon(&Value::Tuple(vec![])),"()");
        assert_eq!(map_key_canon(&Value::Tuple(vec![Value::Bool(true)])),"(참,)");
    }
    #[test]
    fn runtime_collection_keys_preserve_text_and_support_deep_lookup() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let pack = Value::Pack([("a".into(), Value::Bool(true)), ("b".into(), Value::String("한글\n".into()))].into());
            assert_eq!(map_key_canon(&pack), "묶음{a=참, b=\"한글\\n\"}");
            let set = Value::Set([("a".into(), Value::Bool(true)), ("b".into(), Value::Bool(false))].into());
            assert_eq!(map_key_canon(&set), "모음{참, 거짓}");
            let map = Value::Map([("a".into(), MapEntry { key: Value::Bool(true), value: pack.clone() }),
                ("b".into(), MapEntry { key: Value::Bool(false), value: set.clone() })].into());
            assert_eq!(map_key_canon(&map), "짝맞춤{참=>묶음{a=참, b=\"한글\\n\"}, 거짓=>모음{참, 거짓}}");
            let fields = [("값".into(), map.clone())].into();
            let record = Value::Record { field_types: BTreeMap::new(), fields };
            let mut identity = Vec::new();
            ddonirang_core::platform::RecordFieldType::Record { fields: BTreeMap::new() }.encode_canon(&mut identity);
            let hex: String = identity.iter().map(|byte| format!("{byte:02x}")).collect();
            assert_eq!(map_key_canon(&record), format!("record-v1:{hex}:묶음{{값=짝맞춤{{참=>묶음{{a=참, b=\"한글\\n\"}}, 거짓=>모음{{참, 거짓}}}}}}"));
            let mut key = Value::Bool(true);
            for _ in 0..20_000 { key = Value::List(vec![key]); }
            let expected = format!("{}참{}", "차림[".repeat(20_000), "]".repeat(20_000));
            assert_eq!(map_key_canon(&key), expected);
            let entries = [(expected, MapEntry { key: key.clone(), value: Value::String("찾음".into()) })].into();
            assert!(matches!(&map_get(&entries, &key), Value::String(text) if text == "찾음"));
            drop(entries); drop(key); drop(record); drop(map); drop(set); drop(pack);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn runtime_clone_preserves_deep_values_and_real_list_lookup_on_small_stack() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            fn same(left: &Value, right: &Value) {
                let mut pending = vec![(left, right)];
                while let Some((left, right)) = pending.pop() {
                    match (left, right) {
                        (Value::List(a), Value::List(b)) => { assert_eq!(a.len(), b.len()); pending.extend(a.iter().zip(b)); }
                        (Value::Set(a), Value::Set(b)) | (Value::Pack(a), Value::Pack(b)) => {
                            assert!(a.keys().eq(b.keys())); pending.extend(a.values().zip(b.values()));
                        }
                        (Value::Map(a), Value::Map(b)) => {
                            assert!(a.keys().eq(b.keys()));
                            for (a, b) in a.values().zip(b.values()) { pending.push((&a.key, &b.key)); pending.push((&a.value, &b.value)); }
                        }
                        (Value::Record { field_types: a, fields: av }, Value::Record { field_types: b, fields: bv }) => {
                            assert_eq!(a, b); assert!(av.keys().eq(bv.keys())); pending.extend(av.values().zip(bv.values()));
                        }
                        (Value::String(a), Value::String(b)) => assert_eq!(a, b),
                        (Value::Bool(a), Value::Bool(b)) => assert_eq!(a, b),
                        _ => panic!("cloned saved variant changed"),
                    }
                }
            }
            let mut value = Value::String("끝".into());
            for depth in 0..20_000 {
                value = match depth % 5 {
                    0 => Value::List(vec![value, Value::Bool(true)]),
                    1 => Value::Pack([("보존된색인".into(), value)].into()),
                    2 => Value::Set([("보존된색인".into(), value)].into()),
                    3 => Value::Map([("보존된색인".into(), MapEntry { key: value, value: Value::Bool(false) })].into()),
                    _ => Value::Record { field_types: BTreeMap::new(), fields: [("값".into(), value)].into() },
                };
            }
            let copy = value.clone(); same(&value, &copy); assert!(value == copy); drop(copy);
            let key_text = map_key_canon(&value);
            let key_map = [(key_text.clone(), MapEntry { key: value.clone(), value: Value::Bool(true) })].into();
            assert!(matches!(map_get(&key_map, &value), Value::Bool(true)));
            assert_eq!(map_key_canon(&value.clone()), key_text);
            drop(key_map);
            let mut different = value.clone();
            let mut leaf = &mut different;
            loop {
                match leaf {
                    Value::List(items) => leaf = &mut items[0],
                    Value::Set(items) | Value::Pack(items) => leaf = items.values_mut().next().unwrap(),
                    Value::Map(items) => leaf = &mut items.values_mut().next().unwrap().key,
                    Value::Record { fields, .. } => leaf = fields.values_mut().next().unwrap(),
                    Value::String(text) => { *text = "다른끝".into(); break; }
                    _ => panic!("unexpected deep leaf"),
                }
            }
            assert!(value != different); drop(different);
            let list = Value::List(vec![value]);
            let copied = list_nth(&list, &Value::Fixed64(Fixed64::ZERO)).unwrap();
            let Value::List(items) = &list else { unreachable!() };
            same(&items[0], &copied);
            drop(copied); drop(list);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn runtime_clone_preserves_deep_captures_and_magnifier_identity() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let recipe = SealedViewRecipe { source_identity: "source".into(), definition_name: "recipe".into(),
                definition_span: [3, 7], parameter_type: "글".into(), seal: "seal".into(), validation_visits: 9 };
            let mut value = Value::String("같은물체".into());
            for _ in 0..20_000 {
                value = Value::BogaeComposition(BogaeCompositionValue { relations: vec![
                    BogaeRelationValue::Primary { view: ViewValue::ResultTable, source_location: Some([1,2,3,4]), time_override: Some(BogaeTimePolicy::Common) },
                    BogaeRelationValue::Magnifier { target: Box::new(value), recipe: recipe.clone(), view: ViewValue::ResultTable,
                        source_location: Some([5,6,7,8]), time_override: Some(BogaeTimePolicy::Relative(-1)) },
                ] });
            }
            let lambda = Value::Lambda(LambdaValue { id: 123, param: "물체".into(),
                body: Expr::new(0, crate::ast::Span::new(0, 0), crate::ast::ExprKind::Literal(crate::ast::Literal::None)),
                captured: [("물체".into(), value)].into() });
            let cloned = lambda.clone();
            let (Value::Lambda(a), Value::Lambda(b)) = (&lambda, &cloned) else { unreachable!() };
            assert_eq!(a.id, b.id); assert_eq!(a.param, b.param);
            assert!(matches!(b.body.kind, crate::ast::ExprKind::Literal(crate::ast::Literal::None)));
            assert!(a.captured["물체"] == b.captured["물체"]);
            let mut target = &b.captured["물체"];
            for _ in 0..20_000 {
                let Value::BogaeComposition(composition) = target else { panic!("composition changed") };
                assert_eq!(composition.relations.len(), 2);
                assert_eq!(composition.relations[0].source_location(), Some([1,2,3,4]));
                let BogaeRelationValue::Magnifier { target: next, recipe: cloned_recipe, source_location, time_override, .. } = &composition.relations[1] else { panic!("magnifier changed") };
                assert_eq!(cloned_recipe, &recipe); assert_eq!(*source_location, Some([5,6,7,8]));
                assert_eq!(*time_override, Some(BogaeTimePolicy::Relative(-1))); target = next;
            }
            assert!(matches!(target, Value::String(text) if text == "같은물체"));
            drop(cloned); drop(lambda);
        }).unwrap().join().unwrap();
    }

    #[test]
    fn runtime_equality_keeps_lambda_identity_indexes_and_relation_metadata() {
        let lambda = |id, param: &str| Value::Lambda(LambdaValue { id, param: param.into(),
            body: Expr::new(0, crate::ast::Span::new(0, 0), crate::ast::ExprKind::Literal(crate::ast::Literal::None)),
            captured: [(param.into(), Value::Bool(id == 1))].into() });
        assert!(lambda(1, "a") == lambda(1, "b"));
        assert!(lambda(1, "a") != lambda(2, "a"));
        let pack = |key: &str| Value::Pack([(key.into(), Value::None)].into());
        assert!(pack("stored-a") != pack("stored-b"));
        assert!(Value::Pack(BTreeMap::new()) != Value::Set(BTreeMap::new()));
        let side = |placement, location, policy| Value::BogaeRelation(BogaeRelationValue::Side {
            view: ViewValue::ResultTable, placement, source_location: location, time_override: policy });
        let original = side(SidePlacement::Left, Some([1,2,3,4]), Some(BogaeTimePolicy::Common));
        assert!(original == original.clone());
        assert!(original != side(SidePlacement::Right, Some([1,2,3,4]), Some(BogaeTimePolicy::Common)));
        assert!(original != side(SidePlacement::Left, None, Some(BogaeTimePolicy::Common)));
        assert!(original != side(SidePlacement::Left, Some([1,2,3,4]), Some(BogaeTimePolicy::Fixed(0))));
        let overlay = |order| Value::BogaeRelation(BogaeRelationValue::Overlay {
            view: ViewValue::ResultTable, basis_location: Some([1,2,3,4]), order,
            source_location: None, time_override: None });
        assert!(overlay(1) != overlay(2));
    }
    #[test]
    fn automatic_value_drop_covers_containers_captures_and_magnifier_targets() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            fn deep() -> Value {
                let mut value = Value::None;
                for _ in 0..20_000 { value = Value::List(vec![value]); }
                value
            }
            drop(Value::Map([("key".into(), MapEntry { key: deep(), value: deep() })].into()));
            drop(Value::Set([("key".into(), deep())].into()));
            drop(Value::Pack([("key".into(), deep())].into()));
            drop(Value::Record { field_types: BTreeMap::new(), fields: [("key".into(), deep())].into() });
            drop(Value::Lambda(LambdaValue {
                id: 1, param: "x".into(),
                body: Expr::new(0, crate::ast::Span::new(0, 0), crate::ast::ExprKind::Literal(crate::ast::Literal::None)),
                captured: [("key".into(), deep())].into(),
            }));
            let magnifier = || BogaeRelationValue::Magnifier {
                target: Box::new(deep()), view: ViewValue::ResultTable,
                recipe: SealedViewRecipe { source_identity: "test".into(), definition_name: "test".into(), definition_span: [0,0], parameter_type: "test".into(), seal: "test".into(), validation_visits: 0 },
                source_location: None, time_override: None,
            };
            drop(Value::BogaeRelation(magnifier()));
            drop(Value::BogaeComposition(BogaeCompositionValue::from_relations(vec![
                BogaeRelationValue::Primary { view: ViewValue::ResultTable, source_location: None, time_override: None }, magnifier(),
            ]).unwrap()));
            let mut locals: HashMap<String, Value> = [("key".into(), deep())].into();
            locals.insert("key".into(), Value::None);
            assert!(std::panic::catch_unwind(|| { let _pending = deep(); panic!("cleanup test"); }).is_err());
        }).unwrap().join().unwrap();
    }
    #[test]
    fn saved_record_tree_walk_is_iterative_and_keeps_first_error() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut value = Value::None;
            for _ in 0..20_000 { value = Value::List(vec![value]); }
            let result = validate_record_tree(&value);
            drop(value);
            assert!(result.is_ok());
        }).unwrap().join().unwrap();
        let wrong = Value::Record {
            field_types: [("a".into(), ddonirang_core::platform::RecordFieldType::Named { name: "셈수".into() })].into(),
            fields: [("a".into(), Value::Bool(true))].into(),
        };
        let expected = validate_record_value(&wrong);
        assert_eq!(validate_record_tree(&Value::List(vec![wrong, Value::None])), expected);
    }
    #[test]
    fn view_type_does_not_admit_a_plain_pack_or_string() {
        let type_ref = TypeRef::Named("보기값".to_string());
        assert!(check_value_type(&Value::View(ViewValue::ResultTable), &type_ref).is_ok());
        assert!(check_value_type(&Value::Pack(BTreeMap::new()), &type_ref).is_err());
        assert!(check_value_type(&Value::String("결과표".to_string()), &type_ref).is_err());
    }
    #[test]
    fn composition_type_is_not_a_plain_list_or_relation() {
        assert_eq!(BogaeCompositionValue::from_relations(vec![]).unwrap_err(),
            "E_BOGAE_PRIMARY_COUNT: 보개짜임에는 으뜸 관계가 정확히 하나 있어야 합니다");
        let relation = BogaeRelationValue::Primary { view: ViewValue::ResultTable, source_location: None, time_override: None };
        let carrier = BogaeCompositionValue::from_relations(vec![relation.clone()]).unwrap();
        assert_eq!(carrier.relations(), &[relation.clone()]);
        let ty = TypeRef::Named("보개짜임값".into());
        assert!(check_value_type(&Value::BogaeComposition(carrier), &ty).is_ok());
        assert!(check_value_type(&Value::List(vec![Value::BogaeRelation(relation)]), &ty).is_err());
        assert!(check_value_type(&Value::String("짜임".into()), &ty).is_err());
    }
    #[test]
    fn adjacent_fixed64_keys_are_exact_for_map_set_and_nested_units() {
        let a = Value::Fixed64(Fixed64::from_raw_i64(-1));
        let b = Value::Fixed64(Fixed64::ZERO);
        let ka = map_key_canon(&a);
        let kb = map_key_canon(&b);
        assert_eq!(ka, "-0.00000000023283064365386962890625");
        assert_eq!(kb, "0");
        let set = Value::Set(BTreeMap::from([(ka.clone(), a.clone()), (kb.clone(), b.clone())]));
        assert!(matches!(set, Value::Set(ref values) if values.len() == 2));
        let map = BTreeMap::from([
            (ka, MapEntry { key: a.clone(), value: Value::Bool(true) }),
            (kb, MapEntry { key: b, value: Value::Bool(false) }),
        ]);
        assert!(matches!(map_get(&map, &a), Value::Bool(true)));
        assert!(matches!(map_get(&map, &Value::Fixed64(Fixed64::ZERO)), Value::Bool(false)));
        let unit = Value::Unit(UnitValue::new(Fixed64::from_raw_i64(-1), ddonirang_core::Unit::Second));
        assert_eq!(map_key_canon(&Value::List(vec![unit])),
            "차림[-0.00000000023283064365386962890625@s]");
    }
}

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

#[derive(Debug)]
pub struct FrozenSubjectViewV1 {
    table: std::sync::Arc<ddonirang_core::nominal::NominalSlotTableV1>,
    slots: BTreeMap<[u8;32],ddonirang_core::platform::ResourceValue>,
}
impl FrozenSubjectViewV1 {
    pub fn from_resolved_slots(table:std::sync::Arc<ddonirang_core::nominal::NominalSlotTableV1>,slots:BTreeMap<[u8;32],ddonirang_core::platform::ResourceValue>)->Self {Self{table,slots}}
    pub fn canonical_body(&self)->&[u8]{self.table.canonical_body()}
    pub fn name_type_identity(&self)->&[u8;32]{self.table.name_type_identity()}
    pub fn slot(&self,id:&[u8;32])->Option<&ddonirang_core::platform::ResourceValue>{self.slots.get(id)}
}
impl PartialEq for FrozenSubjectViewV1 {
    fn eq(&self,other:&Self)->bool{self.table==other.table}
}

#[derive(Debug)]
pub enum Value {
    /// Execution-only immutable Subject slots. This is neither the nominal
    /// value carrier11 nor a live handle12, and has no persistence encoding.
    FrozenSubjectViewV1(std::sync::Arc<FrozenSubjectViewV1>),
    NominalValueV1(std::sync::Arc<ddonirang_core::nominal::NominalSlotTableV1>),
    SubjectHandleV1(Box<ddonirang_core::nominal::SubjectHandleV1>),
    Rational(std::sync::Arc<ddonirang_core::saved_rational::NormalizedRationalValueV1>),
    IdentityReference(Box<ddonirang_core::platform::IdentityReferenceValueV1>),
    RegisteredAtom(Box<ddonirang_core::platform::RegisteredAtomValueV1>),
    Tensor(TensorValue),
    None,
    Bool(bool),
    Fixed64(Fixed64),
    Unit(UnitValue),
    String(String),
    Bytes(ddonirang_core::bytes::ByteSequence),
    ResourceHandle(ResourceHandle),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    SumBranch {
        owner: String,
        branch: String,
        payload: BTreeMap<String,Value>,
    },
    Set(BTreeMap<String, Value>),
    Map(BTreeMap<String, MapEntry>),
    Pack(BTreeMap<String, Value>),
    Record {
        field_types: BTreeMap<String, ddonirang_core::platform::RecordFieldType>,
        fields: BTreeMap<String, Value>,
    },
    Assertion(Assertion),
    StateMachine(StateMachine),
    Formula(Formula),
    Template(Template),
    Regex(RegexLiteral),
    Lambda(LambdaValue),
    View(ViewValue),
    TimePolicy(BogaeTimePolicy),
    SceneElement(PlaneElement),
    Scene(PlaneScene),
    BogaeRelation(BogaeRelationValue),
    BogaeComposition(BogaeCompositionValue),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        enum Pair<'a> { Value(&'a Value, &'a Value), Relation(&'a BogaeRelationValue, &'a BogaeRelationValue) }
        let mut pending = vec![Pair::Value(self, other)];
        while let Some(pair) = pending.pop() {
            match pair {
                Pair::Relation(left, right) => match (left, right) {
                    (BogaeRelationValue::Primary { view: a, source_location: al, time_override: at },
                     BogaeRelationValue::Primary { view: b, source_location: bl, time_override: bt }) => {
                        if a != b || al != bl || at != bt { return false; }
                    }
                    (BogaeRelationValue::Overlay { view: a, basis_location: ab, order: ao, source_location: al, time_override: at },
                     BogaeRelationValue::Overlay { view: b, basis_location: bb, order: bo, source_location: bl, time_override: bt }) => {
                        if a != b || ab != bb || ao != bo || al != bl || at != bt { return false; }
                    }
                    (BogaeRelationValue::Side { view: a, placement: ap, source_location: al, time_override: at },
                     BogaeRelationValue::Side { view: b, placement: bp, source_location: bl, time_override: bt }) => {
                        if a != b || ap != bp || al != bl || at != bt { return false; }
                    }
                    (BogaeRelationValue::Magnifier { target: a, recipe: ar, view: av, source_location: al, time_override: at },
                     BogaeRelationValue::Magnifier { target: b, recipe: br, view: bv, source_location: bl, time_override: bt }) => {
                        if ar != br || av != bv || al != bl || at != bt { return false; }
                        pending.push(Pair::Value(a, b));
                    }
                    _ => return false,
                },
                Pair::Value(left, right) => match (left, right) {
                    (Value::List(a), Value::List(b)) | (Value::Tuple(a), Value::Tuple(b)) => {
                        if a.len() != b.len() { return false; }
                        pending.extend(a.iter().zip(b).rev().map(|(a, b)| Pair::Value(a, b)));
                    }
                    (Value::Set(a), Value::Set(b)) | (Value::Pack(a), Value::Pack(b)) => {
                        if !a.keys().eq(b.keys()) { return false; }
                        pending.extend(a.values().zip(b.values()).rev().map(|(a, b)| Pair::Value(a, b)));
                    }
                    (Value::Map(a), Value::Map(b)) => {
                        if !a.keys().eq(b.keys()) { return false; }
                        for (a, b) in a.values().zip(b.values()).rev() {
                            pending.push(Pair::Value(&a.value, &b.value)); pending.push(Pair::Value(&a.key, &b.key));
                        }
                    }
                    (Value::Record { field_types: a, fields: av }, Value::Record { field_types: b, fields: bv }) => {
                        if a != b || !av.keys().eq(bv.keys()) { return false; }
                        pending.extend(av.values().zip(bv.values()).rev().map(|(a, b)| Pair::Value(a, b)));
                    }
                    (Value::BogaeRelation(a), Value::BogaeRelation(b)) => pending.push(Pair::Relation(a, b)),
                    (Value::SumBranch { owner:a, branch:ab, payload:av },
                     Value::SumBranch { owner:b, branch:bb, payload:bv }) => {
                        if a!=b || ab!=bb || !av.keys().eq(bv.keys()) { return false; }
                        pending.extend(av.values().zip(bv.values()).rev().map(|(a,b)|Pair::Value(a,b)));
                    }
                    (Value::BogaeComposition(a), Value::BogaeComposition(b)) => {
                        if a.relations.len() != b.relations.len() { return false; }
                        pending.extend(a.relations.iter().zip(&b.relations).rev().map(|(a, b)| Pair::Relation(a, b)));
                    }
                    // Lambda equality has always been identity-only, not an
                    // equality test of captured state or the authored body.
                    (Value::Lambda(a), Value::Lambda(b)) => if a.id != b.id { return false; },
                    (Value::IdentityReference(a), Value::IdentityReference(b)) => if a != b { return false; },
                    (Value::NominalValueV1(a), Value::NominalValueV1(b)) => if a != b { return false; },
                    (Value::FrozenSubjectViewV1(a), Value::FrozenSubjectViewV1(b)) => if a != b { return false; },
                    (Value::SubjectHandleV1(a), Value::SubjectHandleV1(b)) => if a != b { return false; },
                    // RD-660/662: 의미 정체와 저장 근거는 별개다.
                    // ResourceValue의 저장 비교·정본 bytes는 바꾸지 않는다.
                    (Value::RegisteredAtom(a), Value::RegisteredAtom(b)) => {
                        if a.owner_registry_identity != b.owner_registry_identity
                            || a.registered_atom_identity != b.registered_atom_identity {
                            return false;
                        }
                    },
                    (Value::Tensor(a), Value::Tensor(b)) => if a != b { return false; },
                    (Value::None, Value::None) => {},
                    (Value::Bool(a), Value::Bool(b)) => if a != b { return false; },
                    (Value::Fixed64(a), Value::Fixed64(b)) => if a != b { return false; },
                    (Value::Rational(a),Value::Rational(b)) => if a!=b {return false;},
                    (Value::Unit(a), Value::Unit(b)) => if a != b { return false; },
                    (Value::String(a), Value::String(b)) => if a != b { return false; },
                    (Value::Bytes(a), Value::Bytes(b)) => if a != b { return false; },
                    (Value::ResourceHandle(a), Value::ResourceHandle(b)) => if a != b { return false; },
                    (Value::Assertion(a), Value::Assertion(b)) => if a != b { return false; },
                    (Value::StateMachine(a), Value::StateMachine(b)) => if a != b { return false; },
                    (Value::Formula(a), Value::Formula(b)) => if a != b { return false; },
                    (Value::Template(a), Value::Template(b)) => if a != b { return false; },
                    (Value::Regex(a), Value::Regex(b)) => if a != b { return false; },
                    (Value::View(a), Value::View(b)) => if a != b { return false; },
                    (Value::TimePolicy(a), Value::TimePolicy(b)) => if a != b { return false; },
                    (Value::SceneElement(a), Value::SceneElement(b)) => if a != b { return false; },
                    (Value::Scene(a), Value::Scene(b)) => if a != b { return false; },
                    _ => return false,
                },
            }
        }
        true
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        enum Job<'a> {
            Value(&'a Value), List(usize), Tuple(usize),
            Sum(&'a str, &'a str, &'a BTreeMap<String,Value>),
            Indexed(&'a BTreeMap<String, Value>, bool),
            Map(&'a BTreeMap<String, MapEntry>),
            Record(&'a BTreeMap<String, ddonirang_core::platform::RecordFieldType>, &'a BTreeMap<String, Value>),
            Lambda(&'a LambdaValue, Vec<&'a str>),
            Relation(&'a BogaeRelationValue), Magnifier(&'a BogaeRelationValue),
            Composition(usize),
        }
        let mut jobs = vec![Job::Value(self)];
        // Automatic Value Drop also protects completed siblings during unwind.
        let mut built = Vec::<Value>::new();
        while let Some(job) = jobs.pop() {
            match job {
                Job::Value(value) => match value {
                    Value::List(values) => {
                        jobs.push(Job::List(values.len()));
                        jobs.extend(values.iter().rev().map(Job::Value));
                    }
                    Value::Tuple(values) => {
                        jobs.push(Job::Tuple(values.len()));
                        jobs.extend(values.iter().rev().map(Job::Value));
                    }
                    Value::SumBranch { owner, branch, payload } => {
                        jobs.push(Job::Sum(owner,branch,payload));
                        jobs.extend(payload.values().rev().map(Job::Value));
                    }
                    Value::Set(values) | Value::Pack(values) => {
                        jobs.push(Job::Indexed(values, matches!(value, Value::Set(_))));
                        jobs.extend(values.values().rev().map(Job::Value));
                    }
                    Value::Map(values) => {
                        jobs.push(Job::Map(values));
                        for entry in values.values().rev() { jobs.push(Job::Value(&entry.value)); jobs.push(Job::Value(&entry.key)); }
                    }
                    Value::Record { field_types, fields } => {
                        jobs.push(Job::Record(field_types, fields));
                        jobs.extend(fields.values().rev().map(Job::Value));
                    }
                    Value::Lambda(lambda) => {
                        let names: Vec<_> = lambda.captured.keys().map(String::as_str).collect();
                        jobs.push(Job::Lambda(lambda, names.clone()));
                        jobs.extend(names.into_iter().rev().map(|name| Job::Value(&lambda.captured[name])));
                    }
                    Value::BogaeRelation(relation) => jobs.push(Job::Relation(relation)),
                    Value::BogaeComposition(composition) => {
                        jobs.push(Job::Composition(composition.relations.len()));
                        jobs.extend(composition.relations.iter().rev().map(Job::Relation));
                    }
                    // These variants contain no structural Value children.
                    // Authored AST/View recursion remains its own contract.
                    Value::IdentityReference(value) => built.push(Value::IdentityReference(value.clone())),
                    Value::NominalValueV1(value) => built.push(Value::NominalValueV1(value.clone())),
                    Value::FrozenSubjectViewV1(value) => built.push(Value::FrozenSubjectViewV1(value.clone())),
                    Value::SubjectHandleV1(value) => built.push(Value::SubjectHandleV1(value.clone())),
                    Value::RegisteredAtom(value) => built.push(Value::RegisteredAtom(value.clone())),
                    Value::Tensor(value) => built.push(Value::Tensor(value.clone())),
                    Value::None => built.push(Value::None),
                    Value::Bool(value) => built.push(Value::Bool(*value)),
                    Value::Fixed64(value) => built.push(Value::Fixed64(*value)),
                    Value::Rational(value) => built.push(Value::Rational(value.clone())),
                    Value::Unit(value) => built.push(Value::Unit(*value)),
                    Value::String(value) => built.push(Value::String(value.clone())),
                    Value::Bytes(value) => built.push(Value::Bytes(value.clone())),
                    Value::ResourceHandle(value) => built.push(Value::ResourceHandle(*value)),
                    Value::Assertion(value) => built.push(Value::Assertion(value.clone())),
                    Value::StateMachine(value) => built.push(Value::StateMachine(value.clone())),
                    Value::Formula(value) => built.push(Value::Formula(value.clone())),
                    Value::Template(value) => built.push(Value::Template(value.clone())),
                    Value::Regex(value) => built.push(Value::Regex(value.clone())),
                    Value::View(value) => built.push(Value::View(value.clone())),
                    Value::TimePolicy(value) => built.push(Value::TimePolicy(*value)),
                    Value::SceneElement(value) => built.push(Value::SceneElement(value.clone())),
                    Value::Scene(value) => built.push(Value::Scene(value.clone())),
                },
                Job::List(count) => {
                    let values = built.split_off(built.len() - count);
                    built.push(Value::List(values));
                }
                Job::Tuple(count) => {
                    let values = built.split_off(built.len() - count);
                    built.push(Value::Tuple(values));
                }
                Job::Sum(owner,branch,source) => {
                    let values=built.split_off(built.len()-source.len());
                    let payload=source.keys().cloned().zip(values).collect();
                    built.push(Value::SumBranch {owner:owner.into(),branch:branch.into(),payload});
                }
                Job::Indexed(source, set) => {
                    let values = built.split_off(built.len() - source.len());
                    let values = source.keys().cloned().zip(values).collect();
                    built.push(if set { Value::Set(values) } else { Value::Pack(values) });
                }
                Job::Map(source) => {
                    let values = built.split_off(built.len() - source.len() * 2);
                    let mut values = values.into_iter();
                    let entries = source.keys().map(|key| (key.clone(), MapEntry {
                        key: values.next().expect("cloned map key"), value: values.next().expect("cloned map value"),
                    })).collect();
                    built.push(Value::Map(entries));
                }
                Job::Record(field_types, source) => {
                    let values = built.split_off(built.len() - source.len());
                    built.push(Value::Record { field_types: field_types.clone(), fields: source.keys().cloned().zip(values).collect() });
                }
                Job::Lambda(source, names) => {
                    let values = built.split_off(built.len() - names.len());
                    built.push(Value::Lambda(LambdaValue { id: source.id, param: source.param.clone(), body: source.body.clone(),
                        captured: names.into_iter().map(str::to_owned).zip(values).collect() }));
                }
                Job::Relation(source) => {
                    if let BogaeRelationValue::Magnifier { target, .. } = source {
                        jobs.push(Job::Magnifier(source)); jobs.push(Job::Value(target));
                    } else { built.push(Value::BogaeRelation(source.clone())); }
                }
                Job::Magnifier(source) => {
                    let BogaeRelationValue::Magnifier { recipe, view, source_location, time_override, .. } = source else { unreachable!() };
                    let target = built.pop().expect("cloned magnifier target");
                    built.push(Value::BogaeRelation(BogaeRelationValue::Magnifier { target: Box::new(target), recipe: recipe.clone(),
                        view: view.clone(), source_location: *source_location, time_override: *time_override }));
                }
                Job::Composition(count) => {
                    let values = built.split_off(built.len() - count);
                    let relations = values.into_iter().map(|mut value| {
                        let Value::BogaeRelation(relation) = &mut value else { unreachable!() };
                        std::mem::replace(relation, BogaeRelationValue::Primary {
                            view: ViewValue::ResultTable, source_location: None, time_override: None,
                        })
                    }).collect();
                    // Clone must preserve an existing carrier, not re-admit it
                    // or synthesize new relation identities/source locations.
                    built.push(Value::BogaeComposition(BogaeCompositionValue { relations }));
                }
            }
        }
        built.pop().expect("cloned runtime value")
    }
}

impl Value {
    /// Explicit staging cleanup, delegated to the automatic structural owner.
    /// Unrelated authored AST/view object traversal remains a separate contract.
    pub fn drop_saved_tree_stack_safe(self) {
        drop(self);
    }

    pub fn contains_transient_subject_view(&self) -> bool {
        let mut pending=vec![self];
        while let Some(value)=pending.pop(){match value {
            Value::FrozenSubjectViewV1(_)=>return true,
            Value::List(items)|Value::Tuple(items)=>pending.extend(items),
            Value::SumBranch{payload,..}|Value::Record{fields:payload,..}|Value::Pack(payload)|Value::Set(payload)=>pending.extend(payload.values()),
            Value::Map(items)=>for item in items.values(){pending.push(&item.key);pending.push(&item.value);},
            Value::Lambda(lambda)=>pending.extend(lambda.captured.values()),
            Value::BogaeRelation(BogaeRelationValue::Magnifier{target,..})=>pending.push(target),
            Value::BogaeComposition(composition)=>for relation in &composition.relations {
                if let BogaeRelationValue::Magnifier{target,..}=relation {pending.push(target);}
            },
            _=>{},
        }}
        false
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        fn detach_relation(relation: &mut BogaeRelationValue, pending: &mut Vec<Value>) {
            if let BogaeRelationValue::Magnifier { target, .. } = relation {
                pending.push(std::mem::replace(target.as_mut(), Value::None));
            }
        }
        fn detach(value: &mut Value, pending: &mut Vec<Value>) {
            match value {
                Value::List(items) | Value::Tuple(items) => pending.extend(std::mem::take(items)),
                Value::SumBranch { payload, .. } => pending.extend(std::mem::take(payload).into_values()),
                Value::Set(items) | Value::Pack(items) => pending.extend(std::mem::take(items).into_values()),
                Value::Map(items) => for entry in std::mem::take(items).into_values() {
                    pending.push(entry.key); pending.push(entry.value);
                },
                Value::Record { field_types, fields } => {
                    field_types.clear();
                    pending.extend(std::mem::take(fields).into_values());
                }
                Value::Lambda(lambda) => pending.extend(std::mem::take(&mut lambda.captured).into_values()),
                Value::BogaeRelation(relation) => detach_relation(relation, pending),
                Value::BogaeComposition(composition) => for relation in &mut composition.relations {
                    detach_relation(relation, pending);
                },
                _ => {}
            }
        }
        let mut pending = Vec::new();
        detach(self, &mut pending);
        while let Some(mut value) = pending.pop() { detach(&mut value, &mut pending); }
    }
}

/// Immutable admitted Tensor projection; the common saved-value owner issues bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TensorValue {
    element_identity: Vec<u8>,
    shape: Vec<u64>,
    elements: Vec<Fixed64>,
    saved: Vec<u8>,
}

impl TensorValue {
    #[doc(hidden)]
    pub fn from_admitted_parts(element_identity: Vec<u8>, shape: Vec<u64>, elements: Vec<Fixed64>, saved: Vec<u8>) -> Self {
        Self { element_identity, shape, elements, saved }
    }
    pub fn element_identity(&self) -> &[u8] { &self.element_identity }
    pub fn shape(&self) -> &[u64] { &self.shape }
    pub fn elements(&self) -> &[Fixed64] { &self.elements }
    pub fn saved_bytes(&self) -> &[u8] { &self.saved }
}

/// RD-123의 명시 결과표. 세계 자원이 아니라 보개가 해소할 typed 보기값이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewValue {
    ResultTable,
    Scene(PlaneScene),
    Timed { view: Box<ViewValue>, policy: BogaeTimePolicy },
}

/// RD-136: exact scene-madi references, never wall-clock durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BogaeTimePolicy { Common, Fixed(u64), Relative(i64) }

impl BogaeTimePolicy {
    /// Distinct typed values must not collapse to the same canonical map key.
    pub fn canonical_text(self) -> String {
        match self {
            Self::Common => "<보개시간값:공통>".into(),
            Self::Fixed(madi) => format!("<보개시간값:고정마디:{madi}>"),
            Self::Relative(offset) => format!("<보개시간값:상대마디:{offset}>"),
        }
    }
}

impl ViewValue {
    pub fn time_policy(&self) -> BogaeTimePolicy {
        match self { Self::Timed {policy,..} => *policy, _ => BogaeTimePolicy::Common }
    }
    pub fn without_time(&self) -> &ViewValue {
        match self { Self::Timed {view,..} => view.without_time(), _ => self }
    }
}

/// RD-538 closed scene roles. Exact Fixed64 geometry is authored, not fitted by UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaneElement {
    Circle { center: [Fixed64; 2], radius: Fixed64 },
    Polygon { vertices: Vec<[Fixed64; 2]> },
    Text { position: [Fixed64; 2], content: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaneScene {
    pub elements: Vec<PlaneElement>,
    pub horizontal: [Fixed64; 2],
    pub vertical: [Fixed64; 2],
}

/// RD-520 작성 관계의 내용. 정체는 짜임 해소 시 별도로 결속한다.
#[derive(Debug, Clone, PartialEq)]
pub enum BogaeRelationValue {
    Primary { view: ViewValue, source_location: Option<[u32; 4]>, time_override: Option<BogaeTimePolicy> },
    Overlay { view: ViewValue, basis_location: Option<[u32; 4]>, order: i64,
        source_location: Option<[u32; 4]>, time_override: Option<BogaeTimePolicy> },
    Side { view: ViewValue, placement: SidePlacement, source_location: Option<[u32; 4]>, time_override: Option<BogaeTimePolicy> },
    Magnifier { target: Box<Value>, recipe: SealedViewRecipe, view: ViewValue,
        source_location: Option<[u32; 4]>, time_override: Option<BogaeTimePolicy> },
}

/// Product seal of an authored typed named callable, never a renderer callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedViewRecipe {
    pub source_identity: String,
    pub definition_name: String,
    pub definition_span: [usize; 2],
    pub parameter_type: String,
    pub seal: String,
    pub validation_visits: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidePlacement { Left, Right, Top, Bottom, Auto }

impl SidePlacement {
    pub fn from_authored(value: &str) -> Option<Self> {
        match value { "왼쪽" => Some(Self::Left), "오른쪽" => Some(Self::Right),
            "위" => Some(Self::Top), "아래" => Some(Self::Bottom), "자동" => Some(Self::Auto), _ => None }
    }
    pub fn authored_name(self) -> &'static str {
        match self { Self::Left => "왼쪽", Self::Right => "오른쪽", Self::Top => "위",
            Self::Bottom => "아래", Self::Auto => "자동" }
    }
}

impl BogaeRelationValue {
    pub fn kind_name(&self) -> &'static str {
        match self { Self::Primary {..} => "으뜸보개관계값", Self::Overlay {..} => "겹보개관계값",
            Self::Side {..} => "곁보개관계값", Self::Magnifier {..} => "돋보개관계값" }
    }
    pub fn source_location(&self) -> Option<[u32; 4]> {
        match self { Self::Primary {source_location,..} | Self::Overlay {source_location,..}
            | Self::Side {source_location,..} | Self::Magnifier {source_location,..} => *source_location }
    }
    pub fn view(&self) -> &ViewValue {
        match self { Self::Primary {view,..} | Self::Overlay {view,..} | Self::Side {view,..}
            | Self::Magnifier {view,..} => view }
    }
    pub fn time_policy(&self) -> BogaeTimePolicy {
        let policy=match self { Self::Primary {time_override,..} | Self::Overlay {time_override,..}
            | Self::Side {time_override,..} | Self::Magnifier {time_override,..} => *time_override };
        policy.unwrap_or_else(||self.view().time_policy())
    }
    pub fn time_override(&self) -> Option<BogaeTimePolicy> {
        match self { Self::Primary {time_override,..} | Self::Overlay {time_override,..}
            | Self::Side {time_override,..} | Self::Magnifier {time_override,..} => *time_override }
    }
}

/// RD-129/520 authored carrier. Semantic identities are resolved separately;
/// this value is neither a world resource nor an activated UI composition.
#[derive(Debug, Clone, PartialEq)]
pub struct BogaeCompositionValue {
    relations: Vec<BogaeRelationValue>,
}

impl BogaeCompositionValue {
    pub fn from_relations(relations: Vec<BogaeRelationValue>) -> Result<Self, &'static str> {
        let primary: Vec<_> = relations.iter().filter(|r| matches!(r, BogaeRelationValue::Primary {..})).collect();
        if primary.len() != 1 {
            return Err("E_BOGAE_PRIMARY_COUNT: 보개짜임에는 으뜸 관계가 정확히 하나 있어야 합니다");
        }
        let mut locations = BTreeSet::new();
        let mut orders = BTreeSet::new();
        for relation in &relations {
            if let Some(location) = relation.source_location() {
                if !locations.insert(location) {
                    return Err("E_BOGAE_RELATION_DUPLICATE: 같은 작성 관계를 중복할 수 없습니다");
                }
            }
            if let BogaeRelationValue::Overlay {basis_location, order, ..} = relation {
                if basis_location.is_none() || *basis_location != primary[0].source_location() {
                    return Err("E_BOGAE_OVERLAY_BASIS_INVALID: 겹 기준은 같은 짜임의 으뜸이어야 합니다");
                }
                if !orders.insert(*order) {
                    return Err("E_BOGAE_OVERLAY_ORDER_DUPLICATE: 겹 순서는 중복할 수 없습니다");
                }
            }
        }
        Ok(Self { relations })
    }

    pub fn relations(&self) -> &[BogaeRelationValue] {
        &self.relations
    }
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
    enum Job<'a> { Value(&'a Value), Text(&'a str), Pack(&'a BTreeMap<String, Value>) }
    let mut pending = vec![Job::Value(value)];
    let mut out = String::new();
    while let Some(job) = pending.pop() {
        match job {
            Job::Text(text) => out.push_str(text),
            Job::Pack(items) => {
                out.push_str("묶음{");
                pending.push(Job::Text("}"));
                for (index, (key, item)) in items.iter().enumerate().rev() {
                    pending.push(Job::Value(item));
                    pending.push(Job::Text("="));
                    pending.push(Job::Text(key));
                    if index != 0 { pending.push(Job::Text(", ")); }
                }
            }
            Job::Value(Value::List(items)) => {
                out.push_str("차림[");
                pending.push(Job::Text("]"));
                for (index, item) in items.iter().enumerate().rev() {
                    pending.push(Job::Value(item));
                    if index != 0 { pending.push(Job::Text(", ")); }
                }
            }
            Job::Value(Value::Tuple(items)) => {
                out.push('(');
                pending.push(Job::Text(")"));
                if items.len()==1 {pending.push(Job::Text(","));}
                for (index,item) in items.iter().enumerate().rev() {
                    pending.push(Job::Value(item));
                    if index!=0 {pending.push(Job::Text(", "));}
                }
            }
            Job::Value(Value::SumBranch {owner,branch,payload}) => {
                out.push_str(owner);out.push_str(".$");out.push_str(branch);
                if !payload.is_empty() {
                    out.push('(');
                    pending.push(Job::Text(")"));
                    for (index,(pin,value)) in payload.iter().enumerate().rev() {
                        pending.push(Job::Value(value));pending.push(Job::Text("="));pending.push(Job::Text(pin));
                        if index!=0 {pending.push(Job::Text(", "));}
                    }
                }
            }
            Job::Value(Value::Set(items)) => {
                out.push_str("모음{");
                pending.push(Job::Text("}"));
                for (index, item) in items.values().enumerate().rev() {
                    pending.push(Job::Value(item));
                    if index != 0 { pending.push(Job::Text(", ")); }
                }
            }
            Job::Value(Value::Map(items)) => {
                out.push_str("짝맞춤{");
                pending.push(Job::Text("}"));
                for (index, entry) in items.values().enumerate().rev() {
                    pending.push(Job::Value(&entry.value));
                    pending.push(Job::Text("=>"));
                    pending.push(Job::Value(&entry.key));
                    if index != 0 { pending.push(Job::Text(", ")); }
                }
            }
            Job::Value(Value::Pack(items)) => pending.push(Job::Pack(items)),
            Job::Value(Value::Record { field_types, fields }) => {
                let mut identity = Vec::new();
                ddonirang_core::platform::RecordFieldType::Record { fields: field_types.clone() }
                    .encode_canon(&mut identity);
                out.push_str("record-v1:");
                for byte in identity { use std::fmt::Write; write!(&mut out, "{byte:02x}").expect("String writer"); }
                out.push(':');
                pending.push(Job::Pack(fields));
            }
            Job::Value(leaf) => out.push_str(&map_key_leaf_canon(leaf)),
        }
    }
    out
}

fn map_key_leaf_canon(value: &Value) -> String {
    match value {
        Value::Tensor(value) => format!("<Tensor:{:02x?}>", value.saved_bytes()),
        Value::None => "없음".to_string(),
        Value::Bool(true) => "참".to_string(),
        Value::Bool(false) => "거짓".to_string(),
        Value::Fixed64(n) => n.canonical_decimal_text(),
        Value::Rational(value) => ddonirang_core::platform::ResourceValue::RationalV1(value.clone()).canon_key(),
        Value::Unit(unit) => {
            let suffix = unit
                .display_symbol()
                .map(|s| s.to_string())
                .unwrap_or_else(|| unit.dim.format());
            format!("{}@{}", unit.value.canonical_decimal_text(), suffix)
        }
        Value::String(s) => format!("\"{}\"", escape_canon_string(s)),
        Value::RegisteredAtom(atom) => ddonirang_core::platform::ResourceValue::RegisteredAtom(atom.clone()).canon_key(),
        Value::IdentityReference(value) => ddonirang_core::platform::ResourceValue::IdentityReference(value.clone()).canon_key(),
        Value::NominalValueV1(value) => ddonirang_core::platform::ResourceValue::NominalValueV1(value.clone()).canon_key(),
        Value::FrozenSubjectViewV1(value) => format!("transient-subject-view:{:?}",value.canonical_body()),
        Value::SubjectHandleV1(value) => ddonirang_core::platform::ResourceValue::SubjectHandleV1(value.clone()).canon_key(),
        Value::Bytes(bytes) => bytes.canonical_literal(),
        Value::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
        Value::List(_) | Value::Tuple(_) | Value::SumBranch { .. } | Value::Set(_) | Value::Map(_) | Value::Record { .. } | Value::Pack(_) =>
            unreachable!("structural map keys are handled by the heap walker"),
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
        Value::View(view) => match view.without_time() { ViewValue::ResultTable => "<보기값:결과표>", _ => "<보기값:장면>" }.to_string(),
        Value::TimePolicy(policy) => policy.canonical_text(),
        Value::SceneElement(_) => "<장면요소값>".to_string(),
        Value::Scene(_) => "<장면값>".to_string(),
        Value::BogaeRelation(relation) => format!("<{}>", relation.kind_name()),
        Value::BogaeComposition(_) => "<보개짜임값>".to_string(),
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

    fn dispose_type(kind: TypeRef) {
        let mut pending = vec![kind];
        while let Some(kind) = pending.pop() {
            match kind {
                TypeRef::Applied { args, .. } => pending.extend(args),
                TypeRef::Function { params, result } => {
                    pending.extend(params); pending.push(*result);
                }
                TypeRef::Tensor { element, .. } => pending.push(*element),
                _ => {}
            }
        }
    }

    fn dispose_saved_value(value: Value) {
        value.drop_saved_tree_stack_safe();
    }

    #[test]
    fn persisted_record_projection_check_and_rejection_cleanup_are_iterative() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            use ddonirang_core::platform::RecordFieldType as R;
            let mut kind = R::Named { name: "셈수".into() };
            let mut item = Value::Fixed64(Fixed64::ZERO);
            for _ in 0..20_000 {
                kind = R::Applied { name: "차림".into(), args: vec![kind] };
                item = Value::List(vec![item]);
            }
            let mut value = Value::Record {
                field_types: BTreeMap::from([("a".into(), kind)]),
                fields: BTreeMap::from([("a".into(), item)]),
            };
            let accepted = validate_record_value(&value);
            let Value::Record { fields, .. } = &mut value else { unreachable!() };
            let mut leaf = fields.get_mut("a").unwrap();
            for _ in 0..20_000 {
                let Value::List(items) = leaf else { unreachable!() };
                leaf = &mut items[0];
            }
            *leaf = Value::Bool(true);
            let rejected = validate_record_value(&value);
            dispose_saved_value(value);
            assert!(accepted.is_ok());
            let detail = rejected.unwrap_err();
            assert_eq!(detail.expected, format!("<persisted-record-0>.a: {}셈수{}", "(".repeat(20_000), ")차림".repeat(20_000)));
            assert_eq!(detail.actual, format!("{}참거짓{}", "차림(요소: ".repeat(20_000), ")".repeat(20_000)));
        }).unwrap().join().unwrap();
    }

    #[test]
    fn iterative_collection_checks_keep_legacy_first_error_and_messages() {
        let samples = vec![
            Value::None, Value::List(vec![]),
            Value::List(vec![Value::Bool(true), Value::String("next".into())]),
            Value::List(vec![Value::List(vec![Value::Bool(true)])]),
            Value::Set(BTreeMap::from([("a".into(), Value::Bool(true))])),
            Value::Map(BTreeMap::from([("a".into(), MapEntry {
                key: Value::Bool(true), value: Value::String("bad".into()),
            })])),
            Value::Map(BTreeMap::from([("a".into(), MapEntry {
                key: Value::Fixed64(Fixed64::ZERO), value: Value::Bool(true),
            })])),
        ];
        for name in ["차림", "모음", "짝맞춤", "수", "unknown"] {
            for args in [vec![], vec![TypeRef::Named("셈수".into())],
                vec![TypeRef::Named("셈수".into()), TypeRef::Named("글".into())],
                vec![TypeRef::Applied { name: "차림".into(), args: vec![TypeRef::Named("셈수".into())] }]] {
                let kind = TypeRef::Applied { name: name.into(), args };
                let TypeRef::Applied { args, .. } = &kind else { unreachable!() };
                for sample in &samples {
                    assert_eq!(check_value_type_with_records(sample, &kind, &RecordSchemas::new()),
                        check_applied_type(sample, name, args, &RecordSchemas::new()));
                }
            }
        }
    }

    #[test]
    fn persisted_record_projection_keeps_postorder_names_and_error_context() {
        use ddonirang_core::platform::RecordFieldType as R;
        let child_types = BTreeMap::from([("b".into(), R::Named { name: "셈수".into() })]);
        let child = Value::Record {
            field_types: child_types.clone(), fields: BTreeMap::from([("b".into(), Value::Bool(true))]),
        };
        let direct = Value::Record {
            field_types: BTreeMap::from([("a".into(), R::Record { fields: child_types.clone() })]),
            fields: BTreeMap::from([("a".into(), child.clone())]),
        };
        assert_eq!(validate_record_value(&direct).unwrap_err(), TypeMismatchDetail {
            expected: "<persisted-record-1>.a: <persisted-record-0>.b: 셈수".into(),
            actual: "참거짓".into(),
        });
        let collection = Value::Record {
            field_types: BTreeMap::from([("a".into(), R::Applied {
                name: "차림".into(), args: vec![R::Record { fields: child_types }],
            })]),
            fields: BTreeMap::from([("a".into(), Value::List(vec![child]))]),
        };
        assert_eq!(validate_record_value(&collection).unwrap_err(), TypeMismatchDetail {
            expected: "<persisted-record-1>.a: (<persisted-record-0>)차림".into(),
            actual: "차림(요소: 참거짓)".into(),
        });
    }

    #[test]
    fn type_identity_and_diagnostics_use_heap_traversal() {
        std::thread::Builder::new().stack_size(128 * 1024).spawn(|| {
            let mut kind = TypeRef::Named("셈수".into());
            for _ in 0..20_000 {
                kind = TypeRef::Applied { name: "차림".into(), args: vec![kind] };
            }
            let identity = record_type_identity(&kind, &RecordSchemas::new()).unwrap();
            let rendered = format_type_ref(&kind);
            identity.drop_stack_safe();
            // A completed deep sibling must be cleaned up if the next type fails.
            let invalid = TypeRef::Applied {
                name: "짝맞춤".into(), args: vec![kind, TypeRef::Infer],
            };
            let rejected = record_type_identity(&invalid, &RecordSchemas::new());
            dispose_type(invalid);
            assert_eq!(rendered, format!("{}셈수{}", "(".repeat(20_000), ")차림".repeat(20_000)));
            assert_eq!(rejected.err().as_deref(), Some("E_PRODUCT_TYPE_IDENTITY: 저장할 수 없는 필드 타입입니다"));
        }).unwrap().join().unwrap();
    }

    #[test]
    fn type_identity_keeps_field_order_cycle_guard_and_diagnostic_surface() {
        let kind = TypeRef::Function {
            params: vec![TypeRef::Infer, TypeRef::Applied {
                name: "수".into(), args: vec![TypeRef::Named("m".into())],
            }],
            result: Box::new(TypeRef::Tensor {
                element: Box::new(TypeRef::Named("셈수".into())), shape: vec![3],
            }),
        };
        assert_eq!(format_type_ref(&kind), "(_, 수@m) --> (원소씨=셈수, 모양=(3,)) 텐서씨");
        let mut records = RecordSchemas::new();
        records.insert("점".into(), BTreeMap::from([
            ("y".into(), TypeRef::Named("글".into())),
            ("x".into(), TypeRef::Named("셈수".into())),
        ]));
        let identity = record_type_identity(&TypeRef::Named("점".into()), &records).unwrap();
        let ddonirang_core::platform::RecordFieldType::Record { fields } = &identity else { panic!("record") };
        assert_eq!(fields.keys().map(String::as_str).collect::<Vec<_>>(), vec!["x", "y"]);
        records.get_mut("점").unwrap().insert("self".into(), TypeRef::Named("점".into()));
        assert_eq!(record_type_identity(&TypeRef::Named("점".into()), &records).err().as_deref(),
            Some("E_PRODUCT_TYPE_IDENTITY: 유한한 묶음 타입이 필요합니다"));
    }

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
        let Value::List(items) = &removed else {
            panic!("expected list")
        };
        assert_eq!(items, &vec![Value::String("b".to_string())]);
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
        let Value::List(items) = &parts else {
            panic!("expected list")
        };
        assert!(!items.is_empty());
        let rejoin =
            string_join(&parts, &Value::String("a".to_string())).expect("join");
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
        assert_eq!(
            string_to_number(&Value::String("2147483648".to_string())),
            Ok(Value::None),
            "current textual ingress must reject Fixed64 overflow rather than saturate"
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
    check_value_type_with_records(value, type_ref, &BTreeMap::new())
}

/// Declaration-owned structural schemas. Authored order stays in the AST;
/// lookup and admission use the complete field-name/type structure (RD-350).
/// This is an internal type-checking context, not a value or wire envelope.
pub type RecordSchemas = BTreeMap<String, BTreeMap<String, TypeRef>>;

pub fn record_type_identity(
    kind: &TypeRef,
    records: &RecordSchemas,
) -> Result<ddonirang_core::platform::RecordFieldType, String> {
    use ddonirang_core::platform::RecordFieldType as R;
    enum Job<'a> {
        Type(&'a TypeRef),
        Tuple(usize),
        Applied(String, usize),
        Record(Vec<String>),
    }
    // An error may leave completed deep siblings in the staging stack.
    struct Built(Vec<R>);
    impl Drop for Built {
        fn drop(&mut self) {
            for kind in self.0.drain(..) { kind.drop_stack_safe(); }
        }
    }
    let mut pending = vec![Job::Type(kind)];
    let mut built = Built(Vec::new());
    let mut active = Vec::new();
    while let Some(job) = pending.pop() {
        let kind = match job {
            Job::Tuple(count) => {
                let items = built.0.split_off(built.0.len() - count);
                built.0.push(R::Tuple { items });
                continue;
            }
            Job::Applied(name, count) => {
                let args = built.0.split_off(built.0.len() - count);
                built.0.push(R::Applied { name, args });
                continue;
            }
            Job::Record(keys) => {
                let values = built.0.split_off(built.0.len() - keys.len());
                built.0.push(R::Record { fields: keys.into_iter().zip(values).collect() });
                active.pop();
                continue;
            }
            Job::Type(kind) => kind,
        };
        match kind {
            TypeRef::Tuple(items) => {
                pending.push(Job::Tuple(items.len()));
                pending.extend(items.iter().rev().map(Job::Type));
            }
            TypeRef::Named(name) if records.contains_key(name) => {
                if active.contains(name) || active.len() >= 64 {
                    return Err("E_PRODUCT_TYPE_IDENTITY: 유한한 묶음 타입이 필요합니다".into());
                }
                active.push(name.clone());
                let fields = &records[name];
                pending.push(Job::Record(fields.keys().cloned().collect()));
                pending.extend(fields.values().rev().map(Job::Type));
            }
            TypeRef::Named(name) => built.0.push(R::Named { name: canonical_type_name(name) }),
            TypeRef::Applied { name, args } => {
                pending.push(Job::Applied(canonical_type_name(name), args.len()));
                pending.extend(args.iter().rev().map(Job::Type));
            }
            _ => return Err("E_PRODUCT_TYPE_IDENTITY: 저장할 수 없는 필드 타입입니다".into()),
        }
    }
    Ok(built.0.pop().expect("type identity root"))
}

/// Validate persisted evidence against the same admission owner used by calls.
/// Temporary names below are private checker slots, not serialized identities.
pub fn validate_record_value(value: &Value) -> Result<(), TypeMismatchDetail> {
    use ddonirang_core::platform::RecordFieldType as R;
    enum Job<'a> {
        Type(&'a R), Fields(&'a BTreeMap<String, R>),
        Applied(&'a str, usize), Record(Vec<String>), Tuple(usize),
    }
    struct Projection { values: Vec<TypeRef>, records: RecordSchemas }
    impl Drop for Projection {
        fn drop(&mut self) {
            let mut pending = std::mem::take(&mut self.values);
            for fields in std::mem::take(&mut self.records).into_values() {
                pending.extend(fields.into_values());
            }
            while let Some(kind) = pending.pop() {
                match kind {
                    TypeRef::Applied { args, .. } | TypeRef::Tuple(args) => pending.extend(args),
                    TypeRef::Tensor { element, .. } => pending.push(*element),
                    TypeRef::Function { params, result } => {
                        pending.extend(params); pending.push(*result);
                    }
                    _ => {}
                }
            }
        }
    }
    let Value::Record { field_types, .. } = value else {
        return Err(type_mismatch_detail("묶음씨", value));
    };
    let mut projected = Projection { values: Vec::new(), records: RecordSchemas::new() };
    let mut pending = vec![Job::Fields(field_types)];
    while let Some(job) = pending.pop() {
        match job {
            Job::Type(R::Tuple { items }) => {
                pending.push(Job::Tuple(items.len()));
                pending.extend(items.iter().rev().map(Job::Type));
            }
            Job::Type(R::ResolvedNominalV1 {..})=>return Err(type_mismatch_detail("등록 정본에 결속된 명목 필드씨",value)),
            Job::Type(R::RegisteredAtomV1)=>return Err(type_mismatch_detail("출력 전용 원자씨는 저장 필드씨가 아닙니다",value)),
            Job::Type(R::Named { name }) => projected.values.push(TypeRef::Named(name.clone())),
            Job::Type(R::Applied { name, args }) => {
                pending.push(Job::Applied(name, args.len()));
                pending.extend(args.iter().rev().map(Job::Type));
            }
            Job::Type(R::Record { fields }) | Job::Fields(fields) => {
                pending.push(Job::Record(fields.keys().cloned().collect()));
                pending.extend(fields.values().rev().map(Job::Type));
            }
            Job::Applied(name, count) => {
                let args = projected.values.split_off(projected.values.len() - count);
                projected.values.push(TypeRef::Applied { name: name.into(), args });
            }
            Job::Tuple(count) => {
                let items = projected.values.split_off(projected.values.len() - count);
                projected.values.push(TypeRef::Tuple(items));
            }
            Job::Record(keys) => {
                let values = projected.values.split_off(projected.values.len() - keys.len());
                let name = format!("<persisted-record-{}>", projected.records.len());
                projected.records.insert(name.clone(), keys.into_iter().zip(values).collect());
                projected.values.push(TypeRef::Named(name));
            }
        }
    }
    check_value_type_with_records(value, &projected.values[0], &projected.records)
}

pub fn validate_record_tree(value: &Value) -> Result<(), TypeMismatchDetail> {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
        Value::Record { fields, .. } => {
            validate_record_value(value)?;
            pending.extend(fields.values().rev());
        }
        Value::Pack(fields) | Value::Set(fields) => {
            pending.extend(fields.values().rev());
        }
        Value::List(values) | Value::Tuple(values) => {
            pending.extend(values.iter().rev());
        }
        Value::Map(entries) => {
            for entry in entries.values().rev() {
                pending.push(&entry.value); pending.push(&entry.key);
            }
        }
        _ => {}
        }
    }
    Ok(())
}

pub fn check_value_type_with_records(
    value: &Value,
    type_ref: &TypeRef,
    records: &RecordSchemas,
) -> Result<(), TypeMismatchDetail> {
    enum Context<'a> { Field(&'a str, &'a str), Element(&'a TypeRef, &'static str) }
    enum Job<'a> { Check(&'a Value, &'a TypeRef), Enter(Context<'a>), Exit }
    let mut pending = vec![Job::Check(value, type_ref)];
    let mut contexts = Vec::new();
    while let Some(job) = pending.pop() {
        let (value, type_ref) = match job {
            Job::Enter(context) => { contexts.push(context); continue; }
            Job::Exit => { contexts.pop(); continue; }
            Job::Check(value, kind) => (value, kind),
        };
        let result = (|| { match type_ref {
        TypeRef::Tuple(kinds) => {
            let Value::Tuple(items) = value else {
                return Err(type_mismatch_detail(&format_type_ref(type_ref), value));
            };
            if items.len() != kinds.len() {
                return Err(type_mismatch_detail(&format_type_ref(type_ref), value));
            }
            for (item, kind) in items.iter().zip(kinds).rev() {
                pending.push(Job::Exit);
                pending.push(Job::Check(item, kind));
                pending.push(Job::Enter(Context::Element(type_ref, "위치형(요소: ")));
            }
            Ok(())
        }
        TypeRef::Tensor { element, shape } => match value {
            Value::Tensor(tensor) if tensor.shape() == shape.as_slice()
                && matches!(element.as_ref(), TypeRef::Named(name) if name == "셈수") => Ok(()),
            _ => Err(type_mismatch_detail(&format_type_ref(type_ref), value)),
        },
        TypeRef::Infer => Ok(()),
        TypeRef::Named(name) if records.contains_key(name) => {
            let fields = &records[name];
            let Value::Record { field_types, fields: values } = value else {
                return Err(type_mismatch_detail(name, value));
            };
            let expected_identity = record_type_identity(type_ref, records)
                .map_err(|_| type_mismatch_detail(name, value))?;
            let actual_identity = ddonirang_core::platform::RecordFieldType::Record { fields: field_types.clone() };
            let identical = expected_identity == actual_identity;
            expected_identity.drop_stack_safe();
            actual_identity.drop_stack_safe();
            if !identical {
                return Err(type_mismatch_detail(name, value));
            }
            if fields.len() != values.len() || !fields.keys().eq(values.keys()) {
                return Err(type_mismatch_detail(name, value));
            }
            for (field, expected) in fields.iter().rev() {
                pending.push(Job::Exit);
                pending.push(Job::Check(&values[field], expected));
                pending.push(Job::Enter(Context::Field(name, field)));
            }
            Ok(())
        }
        TypeRef::Named(name) => check_named_type(value, name),
        TypeRef::Applied { name, args } => {
            let canonical = canonical_type_name(name);
            match canonical.as_str() {
                "있없" => {
                    if args.len()!=1 {return Err(type_mismatch_detail(&format_type_ref(type_ref),value));}
                    if !matches!(value,Value::None) {pending.push(Job::Check(value,&args[0]));}
                    Ok(())
                }
                "맺음" => {
                    let resolved=crate::sum_owner::resolve_applied_sum_owner(type_ref,crate::ast::Span{start:0,end:0})
                        .map_err(|_|type_mismatch_detail(&format_type_ref(type_ref),value))?;
                    let Value::SumBranch{owner,branch,payload}=value else {return Err(type_mismatch_detail(&resolved.key,value));};
                    if owner!=&resolved.key {return Err(type_mismatch_detail(&resolved.key,value));}
                    let (pin,kind)=match branch.as_str() {
                        "잘됨"=>("값",args.first()),
                        "안됨"=>("이유",args.get(1)),
                        _=>return Err(type_mismatch_detail(&resolved.key,value)),
                    };
                    if payload.len()!=1 {return Err(type_mismatch_detail(&resolved.key,value));}
                    let child=payload.get(pin).ok_or_else(||type_mismatch_detail(&resolved.key,value))?;
                    // Preserve the common heap traversal. The omitted trailing
                    // failure type is the existing scalar 글 default only.
                    if let Some(kind)=kind {pending.push(Job::Check(child,kind));}
                    else {check_named_type(child,"글")?;}
                    Ok(())
                }
                "수" if args.len() == 1 => {
                    if let TypeRef::Named(unit) = &args[0] { check_unit_type(value, "수", unit) }
                    else { Err(type_mismatch_detail("수", value)) }
                }
                "차림" | "모음" | "짝맞춤" => {
                    let required = if canonical == "짝맞춤" { 2 } else { 1 };
                    let matches_container = matches!((canonical.as_str(), value),
                        ("차림", Value::List(_)) | ("모음", Value::Set(_)) | ("짝맞춤", Value::Map(_)));
                    if !matches_container { return Err(type_mismatch_detail(&canonical, value)); }
                    if args.len() != required { return Err(type_mismatch_detail(&format_type_ref(type_ref), value)); }
                    let mut child = |value, kind, label| {
                        pending.push(Job::Exit);
                        pending.push(Job::Check(value, kind));
                        pending.push(Job::Enter(Context::Element(type_ref, label)));
                    };
                    match value {
                        Value::List(items) => for item in items.iter().rev() { child(item, &args[0], "차림(요소: "); },
                        Value::Set(items) => for item in items.values().rev() { child(item, &args[0], "모음(요소: "); },
                        Value::Map(items) => for item in items.values().rev() {
                            child(&item.value, &args[1], "짝맞춤(값: ");
                            child(&item.key, &args[0], "짝맞춤(열쇠: ");
                        },
                        _ => unreachable!("checked container"),
                    }
                    Ok(())
                }
                _ => Err(type_mismatch_detail(&canonical, value)),
            }
        }
        TypeRef::Function { .. } | TypeRef::PropositionConjunction(_) | TypeRef::StructureLaw { .. } | TypeRef::AlgebraEquality { .. } => Err(type_mismatch_detail(&format_type_ref(type_ref), value)),
        } })();
        if let Err(mut detail) = result {
            // The outermost container overwrites inner expected types, whereas
            // all container labels wrap actual. Render each output once.
            let mut expected = String::new();
            let mut actual = String::new();
            let mut replaced_expected = false;
            let mut closings = 0;
            for context in &contexts {
                match context {
                    Context::Field(name, field) if !replaced_expected => {
                        expected.push_str(&format!("{name}.{field}: "));
                    }
                    Context::Element(kind, prefix) => {
                        if !replaced_expected {
                            expected.push_str(&format_type_ref(kind));
                            replaced_expected = true;
                        }
                        actual.push_str(prefix);
                        closings += 1;
                    }
                    _ => {}
                }
            }
            if !replaced_expected { expected.push_str(&detail.expected); }
            actual.push_str(&detail.actual);
            for _ in 0..closings { actual.push(')'); }
            detail.expected = expected;
            detail.actual = actual;
            return Err(detail);
        }
    }
    Ok(())
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
        "바이트열" => match value {
            Value::Bytes(_) => Ok(()),
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
            Value::Pack(_) | Value::Record { .. } => Ok(()),
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
        "보기값" => match value {
            Value::View(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "보개시간값" => match value { Value::TimePolicy(_) => Ok(()), _ => Err(type_mismatch_detail(&canonical,value)) },
        "장면요소값" => match value { Value::SceneElement(_) => Ok(()), _ => Err(type_mismatch_detail(&canonical, value)) },
        "장면값" => match value { Value::Scene(_) => Ok(()), _ => Err(type_mismatch_detail(&canonical, value)) },
        "으뜸보개관계값" | "겹보개관계값" | "곁보개관계값" | "돋보개관계값" | "보개관계값" => match value {
            Value::BogaeRelation(relation) if canonical == "보개관계값" || relation.kind_name() == canonical => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        "보개짜임값" => match value {
            Value::BogaeComposition(_) => Ok(()),
            _ => Err(type_mismatch_detail(&canonical, value)),
        },
        _ if matches!(value,Value::SumBranch {owner,..} if owner==&canonical) => Ok(()),
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

#[cfg(test)]
// Shallow predecessor oracle for diagnostic parity; never a product frontdoor.
fn check_applied_type(
    value: &Value,
    name: &str,
    args: &[TypeRef],
    records: &RecordSchemas,
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
                return Err(type_mismatch_detail(&format_type_ref(&TypeRef::Applied {
                    name: canonical.clone(),
                    args: args.to_vec(),
                }), value));
            }
            for item in items {
                if let Err(detail) = check_value_type_with_records(item, &args[0], records) {
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
                return Err(type_mismatch_detail(&format_type_ref(&TypeRef::Applied {
                    name: canonical.clone(),
                    args: args.to_vec(),
                }), value));
            }
            for item in items.values() {
                if let Err(detail) = check_value_type_with_records(item, &args[0], records) {
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
                return Err(type_mismatch_detail(&format_type_ref(&TypeRef::Applied {
                    name: canonical.clone(),
                    args: args.to_vec(),
                }), value));
            }
            for entry in entries.values() {
                if let Err(detail) = check_value_type_with_records(&entry.key, &args[0], records) {
                    return Err(TypeMismatchDetail {
                        expected: format_type_ref(&TypeRef::Applied {
                            name: "짝맞춤".to_string(),
                            args: args.to_vec(),
                        }),
                        actual: format!("짝맞춤(열쇠: {})", detail.actual),
                    });
                }
                if let Err(detail) = check_value_type_with_records(&entry.value, &args[1], records) {
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
    match value {
        // A3-P2: inferred compound dimensions are produced by UnitDim::format,
        // not necessarily a registered input unit (for example m^2). Preserve
        // that exact existing dimension; do not admit a new scale or conversion.
        Value::Unit(unit_value)
            if unit_spec_from_symbol(unit).map(|spec| spec.dim == unit_value.dim)
                .unwrap_or_else(|| unit_value.dim.format() == unit) => Ok(()),
        _ => Err(type_mismatch_detail(&format!("수@{}", unit), value)),
    }
}

pub fn canonical_type_name(name: &str) -> String {
    crate::stdlib::canonicalize_type_alias(name).to_string()
}

pub(crate) fn format_type_ref(type_ref: &TypeRef) -> String {
    enum Job<'a> { Type(&'a TypeRef), Text(String) }
    let mut pending = vec![Job::Type(type_ref)];
    let mut output = String::new();
    while let Some(job) = pending.pop() {
        let type_ref = match job {
            Job::Text(text) => { output.push_str(&text); continue; }
            Job::Type(kind) => kind,
        };
        match type_ref {
        TypeRef::Tuple(items) => {
            output.push('(');
            pending.push(Job::Text(")".into()));
            for (index, item) in items.iter().enumerate().rev() {
                pending.push(Job::Type(item));
                if index > 0 { pending.push(Job::Text(", ".into())); }
            }
        }
        TypeRef::Tensor { element, shape } => {
            let mut axes = shape.iter().map(u64::to_string).collect::<Vec<_>>().join(", ");
            if shape.len() == 1 { axes.push(','); }
            output.push_str("(원소씨=");
            pending.push(Job::Text(format!(", 모양=({axes})) 텐서씨")));
            pending.push(Job::Type(element));
        }
        TypeRef::Named(name) => output.push_str(&canonical_type_name(name)),
        TypeRef::Applied { name, args } => {
            let name = canonical_type_name(name);
            if name == "수" && args.len() == 1 {
                if let TypeRef::Named(unit) = &args[0] {
                    output.push_str(&format!("수@{}", unit));
                    continue;
                }
            }
            output.push('(');
            pending.push(Job::Text(format!("){name}")));
            for (index, arg) in args.iter().enumerate().rev() {
                pending.push(Job::Type(arg));
                if index > 0 { pending.push(Job::Text(", ".into())); }
            }
        }
        TypeRef::Function { params, result } => {
            output.push('(');
            pending.push(Job::Type(result));
            pending.push(Job::Text(") --> ".into()));
            for (index, param) in params.iter().enumerate().rev() {
                pending.push(Job::Type(param));
                if index > 0 { pending.push(Job::Text(", ".into())); }
            }
        }
        TypeRef::AlgebraEquality {left,right} => output.push_str(&format!("{} =:= {}",left.source_text(),right.source_text())),
        TypeRef::StructureLaw { structure, member } => output.push_str(&format!("{structure}.{member}")),
        TypeRef::PropositionConjunction(items) => {
            output.push('(');
            pending.push(Job::Text(")".into()));
            for (index, item) in items.iter().enumerate().rev() {
                pending.push(Job::Type(item));
                if index > 0 { pending.push(Job::Text(" 이고 ".into())); }
            }
        }
        TypeRef::Infer => output.push('_'),
        }
    }
    output
}

fn type_mismatch_detail(expected: &str, actual: &Value) -> TypeMismatchDetail {
    TypeMismatchDetail {
        expected: expected.to_string(),
        actual: value_type_name(actual),
    }
}

fn numeric_pack_kind(value: &Value) -> Option<&str> {
    if matches!(value,Value::Rational(_)) {return Some(NUMERIC_KIND_RATIONAL);}
    let Value::Pack(fields) = value else {
        return None;
    };
    let Value::String(kind) = fields.get(NUMERIC_PACK_KIND_KEY)? else {
        return None;
    };
    // A forged/legacy Pack cannot certify the new exact numeric brand.
    (kind!=NUMERIC_KIND_RATIONAL).then_some(kind.as_str())
}

pub fn value_type_name(value: &Value) -> String {
    match value {
        Value::SumBranch {owner,..} => owner.clone(),
        Value::Tensor(_) => "텐서씨".to_string(),
        Value::Rational(_) => "나눔수".to_string(),
        Value::Tuple(_) => "위치형 값꾸러미".to_string(),
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
        Value::RegisteredAtom(_) => "등록씨".to_string(),
        Value::IdentityReference(_) => "참조".to_string(),
        Value::NominalValueV1(_) => "이름씨 값".to_string(),
        Value::FrozenSubjectViewV1(_) => "임자 읽기값(읽기 전용)".to_string(),
        Value::SubjectHandleV1(_) => "임자 값".to_string(),
        Value::Bytes(_) => "바이트열".to_string(),
        Value::ResourceHandle(_) => "자원핸들".to_string(),
        Value::List(_) => "차림".to_string(),
        Value::Set(_) => "모음".to_string(),
        Value::Map(_) => "짝맞춤".to_string(),
        Value::Record { .. } => "묶음값".to_string(),
        Value::Pack(_) => numeric_pack_kind(value)
            .map(str::to_string)
            .unwrap_or_else(|| "묶음".to_string()),
        Value::Assertion(_) => "세움값".to_string(),
        Value::StateMachine(_) => "상태머신값".to_string(),
        Value::Formula(_) => "수식값".to_string(),
        Value::Template(_) => "글무늬값".to_string(),
        Value::Regex(_) => "정규식".to_string(),
        Value::Lambda(_) => "씨앗".to_string(),
        Value::View(_) => "보기값".to_string(),
        Value::TimePolicy(_) => "보개시간값".to_string(),
        Value::SceneElement(_) => "장면요소값".to_string(),
        Value::Scene(_) => "장면값".to_string(),
        Value::BogaeRelation(relation) => relation.kind_name().to_string(),
        Value::BogaeComposition(_) => "보개짜임값".to_string(),
    }
}
