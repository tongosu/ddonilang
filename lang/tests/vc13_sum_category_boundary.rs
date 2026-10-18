use ddonirang_lang::{canonicalize,normalize,parse_v25,ExprKind,NormalizationLevel,Stmt,TopLevelItem};
const PAYLOAD:&str=r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$수(값=3). 값 보여주기.
}
"#;
const EXHAUSTIVE:&str=r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$수(값=3). 값에 따라 { 선택.$예이면 { "예" 보여주기. }. 선택.$수(내용)이면 { 내용 보여주기. }. }.
}
"#;
const EXACT_NAME:&str=r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(내용:수). }.
값 <- 선택.$수(내용=3). 값에 따라 { 선택.$예이면 { "예" 보여주기. }. 선택.$수(내용)이면 { 내용 보여주기. }. }.
}
"#;
#[test]
fn vc13_sum_value_generation_and_current_category_canon_are_preserved(){
    assert_eq!(PAYLOAD.len(),118);assert_eq!(EXHAUSTIVE.len(),214);
    // RD-613 supersedes the historical free rename, not the saved old bytes.
    let error=parse_v25(EXHAUSTIVE,"historical-sum.ddn").expect_err("old payload rename is not current authority");
    assert!(error.message.contains("E_SUM_MATCH_BINDER_UNKNOWN"));
    for source in [PAYLOAD,EXACT_NAME]{
        let mut program=parse_v25(source,"sum-category.ddn").expect("current-compatible Sum source");
        let TopLevelItem::SeedDef(seed)=&program.items[0];
        let statements=&seed.body.as_ref().unwrap().stmts;
        assert!(matches!(statements[0],Stmt::SumDefinition{..}));
        let Stmt::Mutate{value,..}=&statements[1] else {panic!("owned branch binding")};
        assert!(matches!(value.kind,ExprKind::SumBranchValue{..}));
        if source==EXACT_NAME {assert!(matches!(statements[2],Stmt::SumMatch{..}));}
        canonicalize(&mut program).unwrap();let first=normalize(&program,NormalizationLevel::N1);
        assert!(first.contains("선택.$수("));
        let mut reparsed=parse_v25(&first,"sum-category.ddn").expect("canonical source");
        canonicalize(&mut reparsed).unwrap();assert_eq!(normalize(&reparsed,NormalizationLevel::N1),first);
    }
}
#[test]
fn vc13_sum_owner_category_rejects_original_invalid_sources(){
    for source in [r#"매마디:움직씨 := {
선택:고름씨 := { $예. $예. }.
}
"#,
r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$미지.
}
"#,
r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- $예.
}
"#,
r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$예. 값에 따라 { 선택.$예이면 { }. }.
}
"#,
r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$예. 값에 따라 { 선택.$예이면 { }. 아니면 { }. 아니면 { }. }.
}
"#,
r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$예. 값에 따라 { 아니면 { }. 선택.$예이면 { }. }.
}
"#,
r#"매마디:움직씨 := {
선택:고름씨 := { $예. $수(값:수). }.
값 <- 선택.$ 예.
}
"#]{
        assert!(parse_v25(source,"sum-category-invalid.ddn").is_err(),"must reject original invalid category: {source}");
    }
}

#[test]
fn vc13_current_sum_fixture_roles_and_canon_roundtrip(){
    let cases:serde_json::Value=serde_json::from_str(include_str!("../../tests/fixtures/vc13_sum_pattern_scope_cases.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let parsed=parse_v25(case["source"].as_str().unwrap(),"current-sum.ddn");
        if let Some(code)=case["code"].as_str().filter(|code|*code!="E_RUNTIME_TYPE_MISMATCH" && case["runtime_error"]!=true) {
            let error=parsed.expect_err("invalid pattern must reject at admission");
            assert!(error.message.contains(code),"{}: {}",case["id"],error.message);
        } else {
            let mut program=parsed.unwrap_or_else(|error|panic!("{}: {}",case["id"],error.message));
            canonicalize(&mut program).unwrap();
            let text=normalize(&program,NormalizationLevel::N1);
            let mut reparsed=parse_v25(&text,"current-sum.ddn").unwrap_or_else(|error|panic!("{}: {}\n{text}",case["id"],error.message));
            canonicalize(&mut reparsed).unwrap();
            assert_eq!(normalize(&reparsed,NormalizationLevel::N1),text,"{}",case["id"]);
        }
    }
}

#[test]
fn vc13_multi_pin_canon_candidate_keeps_binder_and_argument_roles_separate(){
    // PD-WO19-001 is a private candidate, not an assigned public ID scheme.
    for schema in ["가로:수,세로:수","세로:수,가로:수"] {
        for selected in ["가로,세로","세로,가로"] {
            let source=format!("매마디:움직씨 := {{ 선택:고름씨 := {{ $쌍({schema}). }}. 자료 <- 선택.$쌍(세로=4,가로=3). 자료에 따라 {{ 선택.$쌍({selected})이면 {{ (가로+세로) 보여주기. }}. }}. }}");
            let mut program=parse_v25(&source,"sum-pin-candidate.ddn").unwrap();
            let TopLevelItem::SeedDef(seed)=&program.items[0];
            let statements=&seed.body.as_ref().unwrap().stmts;
            let Stmt::SumDefinition{branches,..}=&statements[0] else {panic!("schema")};
            let Stmt::Mutate{value,..}=&statements[1] else {panic!("construction")};
            let ExprKind::SumBranchValue{payload,..}=&value.kind else {panic!("owned construction")};
            assert_eq!(payload.iter().map(|arg|arg.pin.as_str()).collect::<Vec<_>>(),["세로","가로"]);
            let Stmt::SumMatch{arms,..}=&statements[2] else {panic!("typed arm")};
            assert_ne!(arms[0].binder[0].id,arms[0].binder[1].id);
            for binder in &arms[0].binder {assert_ne!(binder.id,branches[0].id);}
            canonicalize(&mut program).unwrap();
            let text=normalize(&program,NormalizationLevel::N1);
            assert!(text.contains("선택.$쌍(가로, 세로)이면"),"candidate must ignore schema/source pin order: {text}");
            assert!(text.contains("선택.$쌍(세로=4, 가로=3)"),"argument evaluation order must survive canon: {text}");
            let mut reparsed=parse_v25(&text,"sum-pin-candidate.ddn").unwrap();
            canonicalize(&mut reparsed).unwrap();
            assert_eq!(normalize(&reparsed,NormalizationLevel::N1),text);
        }
    }
}
