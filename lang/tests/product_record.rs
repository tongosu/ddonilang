use ddonirang_lang::{parse_v25, canonicalize, normalize, NormalizationLevel, ExprKind, SeedKind, Stmt, TopLevelItem};

const SCHEMA: &str = "(가로:셈수, 세로:셈수) 좌표:묶음씨 := {}.\n";

#[test]
fn product_field_diagnostic_preserves_unicode_byte_boundaries() {
    let source = format!("{SCHEMA}점 <- (가로😀=10, 세로=20) 좌표.");
    let error = parse_v25(&source, "product.ddn").expect_err("emoji is not a field pin");
    let start = source.find('😀').unwrap();
    assert_eq!(error.span.start, start);
    assert_eq!(error.span.end, start + '😀'.len_utf8());
    assert_eq!(&source[error.span.start..error.span.end], "😀");
    assert_eq!(ddonirang_lang::utf16_position(&source, error.span.start),
        Some(ddonirang_lang::Utf16Position { line: 1, character: 8 }));
    assert_eq!(ddonirang_lang::utf16_position(&source, error.span.end),
        Some(ddonirang_lang::Utf16Position { line: 1, character: 10 }));
}

#[test]
fn product_typed_consumer_accepts_ordinary_definition_terminator() {
    let source = format!("{SCHEMA}(점:좌표) 점출력:움직씨 := {{ 점.가로 보여주기. }}.\n(점=(가로=10, 세로=20) 좌표) 점출력.");
    let mut program = parse_v25(&source, "product.ddn").expect("typed consumer after Product schema");
    canonicalize(&mut program).unwrap();
    let canonical = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&canonical, "product.ddn").expect(&canonical);
    canonicalize(&mut reparsed).unwrap();
    assert_eq!(canonical, normalize(&reparsed, NormalizationLevel::N1));
    assert!(parse_v25("(가로:셈수) 좌표:묶음씨 := {}", "product.ddn").is_err(), "Product schema still requires its own terminator");
}

#[test]
fn product_schema_construction_and_canon_are_typed() {
    let source = format!("{SCHEMA}점 <- (세로=20, 가로=10) 좌표.\n점.가로 보여주기.");
    let mut program = parse_v25(&source, "product.ddn").expect("product source");
    let TopLevelItem::SeedDef(schema) = &program.items[0];
    assert_eq!(schema.seed_kind, SeedKind::Product);
    assert!(schema.body.as_ref().unwrap().stmts.is_empty());
    let Stmt::Mutate { value, .. } = &program.root_stmts[0] else { panic!("root binding") };
    let ExprKind::ProductConstruction { owner, definition, fields, category } = &value.kind else { panic!("not a call") };
    assert!(category.is_none(),"structural Product remains distinct from nominal schemas");
    assert_eq!(owner, "좌표");
    assert_eq!(*definition, schema.id);
    assert_eq!(fields[0].resolved_pin.as_deref(), Some("세로"));
    canonicalize(&mut program).unwrap();
    let first = normalize(&program, NormalizationLevel::N1);
    let mut reparsed = parse_v25(&first, "product.ddn").expect(&first);
    canonicalize(&mut reparsed).unwrap();
    assert_eq!(first, normalize(&reparsed, NormalizationLevel::N1));
}

#[test]
fn product_schema_and_exact_fields_fail_closed() {
    for (source, code) in [
        // Product fields are schema fields, not callable public pins. Preserve
        // duplicate rejection through the current Product diagnostic owner.
        ("(가로:셈수, 가로:셈수) 좌표:묶음씨 := {}.".to_owned(), "E_PRODUCT_FIELD_DUPLICATE"),
        ("(가로:셈수) 좌표:묶음씨 := { 1 보여주기. }.".to_owned(), "E_PRODUCT_SCHEMA_BODY"),
        (format!("{SCHEMA}점 <- (가로=10) 좌표."), "E_PRODUCT_FIELD_MISSING"),
        (format!("{SCHEMA}점 <- (가로=10, 세로=20, 높이=30) 좌표."), "E_PRODUCT_FIELD_UNKNOWN"),
        (format!("{SCHEMA}점 <- (가로=10, 가로=20) 좌표."), "E_PRODUCT_FIELD_DUPLICATE"),
        (format!("{SCHEMA}점 <- (10, 20) 좌표."), "E_PRODUCT_NAMED_FIELD_REQUIRED"),
        (format!("{SCHEMA}점 <- (가로=\"한글😀\", 세로=20) 좌표."), "E_RUNTIME_TYPE_MISMATCH"),
        (format!("{SCHEMA}점 <- (가로=10, 세로=20) 좌표. 점.높이 보여주기."), "E_FIELD_MISSING"),
    ] {
        let error = parse_v25(&source, "product.ddn").expect_err(&source);
        assert!(error.message.contains(code), "{source}\n{error}");
    }
}

#[test]
fn product_schema_type_context_is_structural_and_exact() {
    use ddonirang_lang::{runtime::{check_value_type_with_records, RecordSchemas}, TypeRef, Value};
    use std::collections::BTreeMap;
    let number = TypeRef::Named("셈수".into());
    let fields = BTreeMap::from([("가로".into(), number.clone()), ("세로".into(), number.clone())]);
    let mut schemas = RecordSchemas::from([("좌표".into(), fields.clone())]);
    schemas.insert("다른이름".into(), fields.into_iter().rev().collect());
    assert_eq!(schemas["좌표"], schemas["다른이름"], "source order/name is not structural identity");
    let ddonirang_core::platform::RecordFieldType::Record { fields: ref field_types } =
        ddonirang_lang::runtime::record_type_identity(&TypeRef::Named("좌표".into()), &schemas).unwrap()
    else { panic!("record identity") };
    let values = BTreeMap::from([
        ("가로".into(), Value::Fixed64(ddonirang_core::Fixed64::from_i32(10))),
        ("세로".into(), Value::Fixed64(ddonirang_core::Fixed64::from_i32(20))),
    ]);
    let value = Value::Record { field_types: field_types.clone(), fields: values.clone() };
    assert!(check_value_type_with_records(&Value::Pack(values), &TypeRef::Named("좌표".into()), &schemas).is_err(), "shape is not record admission");
    for name in ["좌표", "다른이름"] {
        check_value_type_with_records(&value, &TypeRef::Named(name.into()), &schemas).unwrap();
    }
    schemas.insert("다른씨".into(), BTreeMap::from([("가로".into(), number), ("세로".into(), TypeRef::Named("글".into()))]));
    assert_ne!(schemas["좌표"], schemas["다른씨"]);
    assert!(check_value_type_with_records(&value, &TypeRef::Named("다른씨".into()), &schemas).is_err());
    assert!(check_value_type_with_records(&value, &TypeRef::Named("미등록이름씨".into()), &schemas).is_err());
}

#[test]
fn product_resolution_does_not_depend_on_declaration_position() {
    let source = format!("점 <- (가로=10, 세로=20) 좌표.\n{SCHEMA}");
    let program = parse_v25(&source, "product.ddn").unwrap();
    let Stmt::Mutate { value, .. } = &program.root_stmts[0] else { panic!("binding") };
    assert!(matches!(value.kind, ExprKind::ProductConstruction { .. }));
}

#[test]
fn product_statement_and_definition_owner_are_not_callable_aliases() {
    let source = "(가로:셈수) 걷기:묶음씨 := {}.\n(가로=10) 걷기.";
    let program = parse_v25(source, "product.ddn").unwrap();
    let Stmt::Expr { expr, .. } = &program.root_stmts[0] else { panic!("construction statement") };
    assert!(matches!(expr.kind, ExprKind::ProductConstruction { .. }));
    let duplicate = format!("{SCHEMA}{SCHEMA}");
    assert!(parse_v25(&duplicate, "product.ddn").unwrap_err().message.contains("E_PRODUCT_DEFINITION_DUPLICATE"));
}
