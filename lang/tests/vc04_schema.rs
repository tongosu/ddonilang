use ddonirang_lang::{parse_v25_with_mode, ParseMode, TopLevelItem, SeedKind};
use ddonirang_lang::ast::SchemaCategory;
use ddonirang_lang::normalizer::{Normalizer, NormalizationLevel};

#[test]
fn vc04_slot_schema_does_not_capture_existing_reaction_owner(){
    let raw="(값:수) 알림:알림씨 := { 없음. }. 관제탑:임자 := { (받은:알림) 반응:움직씨 {알림을 받으면} := { 값 <- 7. }. }. 병사:임자 := { 이름:글. }.";
    let program=parse_v25_with_mode(raw,"reaction-owner.ddn",ParseMode::Strict).unwrap();
    let owners:Vec<_>=program.items.iter().map(|item|{let TopLevelItem::SeedDef(seed)=item;seed}).collect();
    assert!(owners.iter().any(|seed|seed.canonical_name=="관제탑"&&matches!(&seed.seed_kind,SeedKind::Named(name) if name=="임자")));
    assert!(owners.iter().any(|seed|seed.canonical_name=="병사"&&matches!(seed.seed_kind,SeedKind::Schema(SchemaCategory::Subject))));
    for bad in ["병사:임자 := { (1 + 2) 보여주기. }.","명찰:이름씨 := { (받은:알림) 반응:움직씨 {알림을 받으면} := { 값 <- 7. }. }."]{
        assert!(parse_v25_with_mode(bad,"bad-schema.ddn",ParseMode::Strict).is_err());
    }
}

#[test]
fn vc04_atom_role_is_owned_by_standard_callable_pin_not_spelling(){
    use ddonirang_lang::ast::AtomValueRole;
    let source="색 <- #오른쪽. 곁 := (보기=(() 표준보기.결과표),자리=#오른쪽) 표준보개.곁. \"#오른쪽\" 보여주기.";
    let program=parse_v25_with_mode(source,"roles.ddn",ParseMode::Strict).unwrap();
    assert_eq!(program.origin.atom_value_occurrences.len(),2);
    assert_eq!(program.origin.atom_value_occurrences.values().filter(|value|matches!(value.role,AtomValueRole::LocalValue)).count(),1);
    assert_eq!(program.origin.atom_value_occurrences.values().filter(|value|matches!(value.role,AtomValueRole::StandardBogaeSide{selector:"오른쪽"})).count(),1);
    for (place,code) in [("#빨강","E_BOGAE_UNSUPPORTED"),("\"오른쪽\"","E_ATOM_POLICY_ROLE_REQUIRED"),("색","E_ATOM_POLICY_ROLE_REQUIRED")] {
        let source=format!("색 <- #오른쪽. 곁 := (보기=(() 표준보기.결과표),자리={place}) 표준보개.곁.");
        let error=parse_v25_with_mode(&source,"bad-policy.ddn",ParseMode::Strict).unwrap_err();
        assert!(error.message.contains(code),"{}",error.message);
    }
}

#[test]
fn vc04_acquire_postfix_receiver_binding_and_pure_capability_are_explicit(){
    let source="병사:임자 := { 이름:글. }. (철수:병사) 획득:움직씨 := { 철수 가리키기 보여주기. }.";
    let program=parse_v25_with_mode(source,"acquire.ddn",ParseMode::Strict).unwrap();
    assert_eq!(program.origin.subject_reference_calls.len(),1);
    let binding=program.origin.subject_reference_calls.values().next().unwrap();
    assert_eq!(binding.definition,"VC04.DEF.GARIKIGI.R1");
    assert_eq!(binding.receiver_pin,"VC04.PIN.GARIKIGI.TARGET.R1");
    for source in [
        "병사:임자 := { 이름:글. }. (철수:병사) 획득:(병사)가리킴:셈씨 := { 획득 <- 철수 가리키기. }.",
        "병사:임자 := { 이름:글. }. (철수:병사) 도우미:움직씨 := { 철수 가리키기 보여주기. }. (철수:병사) 획득:바른수:셈씨 := { (철수) 도우미. 획득 <- 1. }.",
        "실행:움직씨 := { (대상=1) 가리키기. }.",
    ] {
        let failure=parse_v25_with_mode(source,"bad-acquire.ddn",ParseMode::Strict).unwrap_err();
        assert!(failure.message.contains("E_VC04_PURE_SUBJECT_READ")||failure.message.contains("E_VC04_REFERENCE_RECEIVER"),"{failure:?}");
    }
}

#[test]
fn vc04_read_snapshot_static_qualifier_rejects_hidden_persistence(){
    let prefix="병사:임자 := { 이름:글. }. 실행:움직씨 := { 철수:병사 := (이름=\"철수\")인 병사. 표적:(병사)가리킴 := 철수 가리키기. 읽은것:(병사,가리킴안됨)맺음 := 표적 읽기.";
    for tail in [
        "저장 <- 읽은것.",
        "저장 <- (1,읽은것).",
        "저장 <- 차림[읽은것].",
        "읽은것에 따라 { (병사,가리킴안됨)맺음.$잘됨(값)이면 { 저장 <- 값. }. (병사,가리킴안됨)맺음.$안됨(이유)이면 { }. }.",
        "읽은것에 따라 { (병사,가리킴안됨)맺음.$잘됨(값)이면 { 값.이름 <- \"금지\". }. (병사,가리킴안됨)맺음.$안됨(이유)이면 { }. }.",
        "읽은것에 따라 { (병사,가리킴안됨)맺음.$잘됨(값)이면 { 값 가리키기 보여주기. }. (병사,가리킴안됨)맺음.$안됨(이유)이면 { }. }.",
    ] {
        let source=format!("{prefix}{tail} }}.");
        let failure=parse_v25_with_mode(&source,"snapshot-escape.ddn",ParseMode::Strict).unwrap_err();
        assert!(failure.message.contains("E_VC04_READ_SNAPSHOT_ESCAPE")||failure.message.contains("E_VC04_READ_ONLY_PLACE"),"{failure:?}");
    }
}

#[test]
fn vc04_schema_category_and_slots_survive_canon_roundtrip() {
    let source = "병사:임자 := { 이름:글. 나이:바른수. }. 명찰:이름씨 := { 이름:글. }.";
    let parsed = parse_v25_with_mode(source, "schema.ddn", ParseMode::Strict).unwrap();
    assert!(parsed.root_stmts.is_empty());
    let TopLevelItem::SeedDef(subject) = &parsed.items[0];
    assert_eq!(subject.seed_kind, SeedKind::Schema(SchemaCategory::Subject));
    assert_eq!(subject.params.iter().map(|slot|slot.pin_name.as_str()).collect::<Vec<_>>(), ["이름", "나이"]);
    assert!(subject.body.is_none());
    assert!(subject.callable_surface_names().is_empty());
    let TopLevelItem::SeedDef(nominal) = &parsed.items[1];
    assert_eq!(nominal.seed_kind, SeedKind::Schema(SchemaCategory::NominalValue));
    let canon = Normalizer::new(NormalizationLevel::N1).normalize_program(&parsed);
    let restored = parse_v25_with_mode(&canon, "schema.ddn", ParseMode::Strict).unwrap();
    let TopLevelItem::SeedDef(restored_subject) = &restored.items[0];
    assert_eq!(restored_subject.seed_kind, subject.seed_kind);
    assert_eq!(restored_subject.params.len(), 2);
}

#[test]
fn vc04_schema_rejects_duplicate_or_executable_slots_and_definitions() {
    for source in [
        "병사:임자 := { 이름:글. 이름:글. }.",
        "병사:임자 := { 이름:글 := \"철수\". }.",
        "병사:임자 := { \"실행\" 보여주기. }.",
        "병사:임자 := {}. 병사:이름씨 := {}.",
        "병사:임자 := {}. 병사:임자 := {}.",
    ] {
        assert!(parse_v25_with_mode(source, "schema.ddn", ParseMode::Strict).is_err(), "{source}");
    }
}

#[test]
fn vc04_atom_occurrences_are_value_nodes_not_textual_hash_matches() {
    let source = "색 <- #빨강. 복사 <- #빨강. \"#가짜\" 보여주기. 식 <- (#ascii) 수식{x}.";
    let parsed = parse_v25_with_mode(source, "atoms.ddn", ParseMode::Strict).unwrap();
    let mut occurrences = parsed.origin.atom_value_occurrences.values().collect::<Vec<_>>();
    occurrences.sort_by_key(|occurrence|occurrence.span.start);
    assert_eq!(occurrences.len(), 2);
    for occurrence in occurrences {
        assert_eq!(occurrence.label, "빨강");
        assert_eq!(&source[occurrence.span.start..occurrence.span.end], "#빨강");
    }
}

#[test]
fn vc04_nominal_construction_keeps_real_category_and_in_marker_in_canon() {
    use ddonirang_lang::{Stmt,ExprKind};
    let source="병사:임자 := { 이름:글. }. 명찰:이름씨 := { 이름:글. }. 실행:움직씨 := { 새병사 <- (이름=\"철수\")인 병사. 새명찰 <- (이름=\"철수\")인 명찰. }.";
    let program=parse_v25_with_mode(source,"구성.ddn",ParseMode::Strict).unwrap();
    let TopLevelItem::SeedDef(seed)=&program.items[2];
    for (stmt,category) in seed.body.as_ref().unwrap().stmts.iter().zip([SchemaCategory::Subject,SchemaCategory::NominalValue]) {
        let Stmt::Mutate{value,..}=stmt else {panic!("binding")};
        assert!(matches!(value.kind,ExprKind::ProductConstruction{category:Some(actual),..} if actual==category));
    }
    let canon=Normalizer::new(NormalizationLevel::N1).normalize_program(&program);
    assert!(canon.contains(")인 병사"));assert!(canon.contains(")인 명찰"));
    let reopened=parse_v25_with_mode(&canon,"구성.ddn",ParseMode::Strict).unwrap();
    assert_eq!(canon,Normalizer::new(NormalizationLevel::N1).normalize_program(&reopened));
    for bad in ["병사:임자 := { 이름:글. }. 실행:움직씨 := { (이름=\"철수\") 병사. }.",
        "병사:임자 := { 이름:글. }. 실행:움직씨 := { (다른=\"철수\")인 병사. }.",
        "병사:임자 := { 이름:글. }. 실행:움직씨 := { ()인 병사. }."] {
        assert!(parse_v25_with_mode(bad,"오류.ddn",ParseMode::Strict).is_err(),"{bad}");
    }
}

#[test]
fn vc04_subject_world_write_is_rejected_from_pure_direct_indirect_and_dead_branch() {
    let head="병사:임자 := { 이름:글. }.";
    for body in [
        "검사:병사:셈씨 := { 검사 <- (이름=\"철수\")인 병사. }.",
        "생성:움직씨 := { 새값 <- (이름=\"철수\")인 병사. }. 검사:바른수:셈씨 := { 생성. 검사 <- 1. }.",
        "생성:움직씨 := { 새값 <- (이름=\"철수\")인 병사. }. 중계:움직씨 := { 생성. }. 검사:바른수:셈씨 := { { 거짓 }이면 { 중계. }. 검사 <- 1. }.",
    ] {
        let source=format!("{head}{body}");let error=parse_v25_with_mode(&source,"순수오류.ddn",ParseMode::Strict).unwrap_err();
        assert!(error.message.starts_with("E_VC04_PURE_WORLD_WRITE"),"{}",error.message);
        assert!(source.get(error.span.start..error.span.end).is_some());
    }
    let pure="명찰:이름씨 := { 이름:글. }. (이름:글) 이름표:명찰:셈씨 := { 이름표 <- (이름=이름)인 명찰. }.";
    parse_v25_with_mode(pure,"순수값.ddn",ParseMode::Strict).expect("nominal value construction is pure");
    let local=format!("{head}생성:움직씨 := {{ 새값 <- (이름=\"철수\")인 병사. }}. 검사:바른수:셈씨 := {{ 생성 <- 1. 검사 <- 생성. }}.");
    parse_v25_with_mode(&local,"지역값.ddn",ParseMode::Strict).expect("a local value is not the global niladic generator");
}

#[test]
fn vc04_retire_is_owned_command_without_result_or_pin_aliases() {
    let head="병사:임자 := { 이름:글. }.";
    let source=format!("{head}(표적:(병사)가리킴) 제거:움직씨 := {{ (대상=표적) 폐기하기. }}.");
    let program=parse_v25_with_mode(&source,"폐기.ddn",ParseMode::Strict).unwrap();
    assert_eq!(program.origin.retirement_commands.len(),1);
    let binding=program.origin.retirement_commands.values().next().unwrap();
    assert_eq!(binding.definition,"VC04.DEF.PYEGIHAGI.R1");assert_eq!(binding.target_pin,"VC04.PIN.PYEGIHAGI.TARGET.R1");
    for body in [
        "(표적:(병사)가리킴) 제거:움직씨 := { 복사 <- 표적. (대상=복사) 폐기하기. }.",
        "(표적:(병사)가리킴) 제거:움직씨 := { 복사 <- 차림[1]. 복사 <- 표적. (대상=복사) 폐기하기. }.",
        "(표적:(병사)가리킴) 제거:움직씨 := { 복사 <- 1. { 참 }이면 { 복사 <- 표적. }. (대상=복사) 폐기하기. }.",
    ] {
        parse_v25_with_mode(&format!("{head}{body}"),"폐기가리킴.ddn",ParseMode::Strict)
            .expect("reference aliases and replacement must not retain a stale non-reference fact");
    }
    for (body,code) in [
        ("(표적:(병사)가리킴) 제거:움직씨 := { 값 <- (대상=표적) 폐기하기. }.","E_VC04_COMMAND_VALUE"),
        ("(표적:(병사)가리킴) 제거:움직씨 := { ((대상=표적) 폐기하기) 보여주기. }.","E_VC04_COMMAND_VALUE"),
        ("(표적:(병사)가리킴) 제거:움직씨 := { (다른=표적) 폐기하기. }.","E_VC04_RETIRE_PIN"),
        ("(표적:(병사)가리킴) 제거:움직씨 := { (대상=표적,추가=1) 폐기하기. }.","E_VC04_RETIRE_PIN"),
        ("(표적:병사) 제거:움직씨 := { (대상=표적) 폐기하기. }.","E_VC04_RETIRE_TARGET"),
        ("제거:움직씨 := { (대상=1) 폐기하기. }.","E_VC04_RETIRE_TARGET"),
        ("제거:움직씨 := { 숫자 <- 1. (대상=숫자) 폐기하기. }.","E_VC04_RETIRE_TARGET"),
        ("제거:움직씨 := { 숫자 := 1. 복사 := 숫자. (대상=복사) 폐기하기. }.","E_VC04_RETIRE_TARGET"),
        ("제거:움직씨 := { 값 <- 차림[1]. (대상=값) 폐기하기. }.","E_VC04_RETIRE_TARGET"),
        ("제거:움직씨 := { 폐기하기. }.","E_VC04_RETIRE_PIN"),
        ("(표적:(병사)가리킴) 제거:움직씨 := { 폐기하기 <- 1. (대상=표적) 폐기하기. }.","E_VC04_RETIRE_AMBIGUOUS"),
        ("(대상:글) 폐기하:움직씨 := { 대상 보여주기. }. 제거:움직씨 := { (대상=1) 폐기하기. }.","E_VC04_RETIRE_AMBIGUOUS"),
        ("(표적:(병사)가리킴) 제거:바른수:셈씨 := { (대상=표적) 폐기하기. 제거 <- 1. }.","E_VC04_PURE_WORLD_WRITE"),
    ] {
        let error=parse_v25_with_mode(&format!("{head}{body}"),"폐기오류.ddn",ParseMode::Strict).unwrap_err();
        assert!(error.message.starts_with(code),"expected {code}, got {}",error.message);
        assert_eq!(error.code(),code);
    }
}
