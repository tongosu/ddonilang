use ddonirang_lang::{ast::*, parse_v25_with_mode, ParseMode};

#[test]
fn calculation_chain_retains_actual_definition_pins_and_call_span() {
    let base=include_str!("structure_law_ko_surface_r1.ddn");
    let tail="\n(값:바른수~을, 근거:(항등구조.법칙)보임~로) 이어계산:바른수:셈씨 := { 이어계산 <- (값=값, 근거=근거) 법칙필수계산. }.";
    let raw=format!("// 한글 😀\r\n{base}{tail}");
    let facts=ddonirang_lang::proof_authoring::resolve_assumption_source(&raw,"연쇄.ddn").unwrap();
    let application=facts[2].calculation_application().unwrap();
    assert!(application.bindings_match());
    assert!(facts[2].structure_consumption().is_none());
    assert_eq!(application.definition().scope(),facts[1].scope());
    assert_eq!(&raw[application.call().span.start..application.call().span.end],"(값=값, 근거=근거) 법칙필수계산");
    assert_eq!(application.argument(),&facts[2].assumptions()[0]);
}

#[test]
fn ko_surface_preserves_authored_static_arguments_and_origin() {
    let source = include_str!("structure_law_ko_surface_r1.ddn");
    let program = parse_v25_with_mode(source, "한글.ddn", ParseMode::Strict).unwrap();
    assert_eq!(program.root_stmts.len(), 1);
    let Stmt::Definition { value, .. } = &program.root_stmts[0] else { panic!("정적 정의 필요") };
    let ExprKind::Call { func, args } = &value.kind else { panic!("공통 호출 AST 필요") };
    assert_eq!(func, "항등법칙요구짜임");
    assert_eq!(args.len(), 2);
    for (arg, (pin, name)) in args.iter().zip([("바탕씨", "바른수"), ("연산", "바른수그대로")]) {
        assert_eq!(arg.resolved_pin.as_deref(), Some(pin));
        assert!(matches!(&arg.expr.kind, ExprKind::Var(n) if n == name));
        assert_eq!(&source[arg.expr.span.start..arg.expr.span.end], name);
    }
}

#[test]
fn ko_surface_shadow_preserves_actual_authored_definition_candidate() {
    let source = format!("(바탕씨:바른수~을, 연산:바른수~로) 항등법칙요구짜임:바른수:셈씨 := {{ 항등법칙요구짜임 <- 1. }}.\n{}", include_str!("structure_law_ko_surface_r1.ddn"));
    let program = parse_v25_with_mode(&source, "가림.ddn", ParseMode::Strict).unwrap();
    let Stmt::Definition { value, .. } = &program.root_stmts[0] else { panic!("정적 정의 필요") };
    let TopLevelItem::SeedDef(shadow) = &program.items[0];
    assert_eq!(shadow.canonical_name, "항등법칙요구짜임");
    assert_eq!(program.origin.authored_call_candidates.get(&value.id), Some(&vec![shadow.id]));
}

#[test]
fn law_selector_does_not_consume_ordinary_declaration_terminators() {
    let source="병사:임자 := { 이름:글. 나이:바른수. }.";
    parse_v25_with_mode(source,"ordinary.ddn",ParseMode::Strict).unwrap();
    let outside="(값:항등구조.법칙~을) 전달:바른수:셈씨 := { 전달 <- 1. }.";
    assert!(parse_v25_with_mode(outside,"ordinary.ddn",ParseMode::Strict).is_err());
}

#[test]
fn structure_goal_retains_owner_and_is_not_an_ordinary_named_type() {
    let source = "(근거:(항등구조.법칙)보임~을)\n법칙전달:(항등구조.법칙)보임:셈씨 := { 법칙전달 <- 근거. }.";
    let program = parse_v25_with_mode(source, "structure.ddn", ParseMode::Strict).unwrap();
    let TopLevelItem::SeedDef(seed) = &program.items[0];
    let TypeRef::Applied { name, args } = seed.result_type.as_ref().unwrap() else { panic!("proof type") };
    assert_eq!(name, "보임");
    assert_eq!(args, &[TypeRef::StructureLaw { structure: "항등구조".into(), member: "법칙".into() }]);
    // Parsing a selector does not mint an admitted structure or a proof.
    assert!(ddonirang_lang::proof_authoring::resolve_assumption_source(source, "structure.ddn").is_err());
}

#[test]
fn unsupported_structure_goal_member_is_rejected_at_source() {
    let source = "(근거:(항등구조.성공)보임~을)\n법칙전달:(항등구조.성공)보임:셈씨 := { 법칙전달 <- 근거. }.";
    let error = parse_v25_with_mode(source, "structure.ddn", ParseMode::Strict).unwrap_err();
    assert!(error.message.contains("E_PROOF_GOAL"));
    assert!(error.span.end <= source.len());
}

#[test]
fn adopted_r3_raw_passes_shared_parser() {
    let source = include_str!("structure_law_r3.ddn");
    parse_v25_with_mode(source, "structure.ddn", ParseMode::Strict).unwrap();
}

#[test]
fn missing_identity_supply_is_not_an_accepted_proof() {
    let source = include_str!("structure_law_r3.ddn").replace("항등구조.법칙", "없는구조.법칙");
    let error = ddonirang_lang::proof_authoring::resolve_assumption_source(&source, "structure.ddn").unwrap_err();
    assert!(error.message.starts_with("E_PROOF_GOAL:"));
    let marked = &source[error.span.start..error.span.end];
    assert!(marked.contains("없는구조.법칙"));
    assert!(!marked.contains("명시단항차림새 :="));
}

#[test]
fn structure_source_resolves_two_conditional_definitions() {
    let source=include_str!("structure_law_r3.ddn");
    let facts=ddonirang_lang::proof_authoring::resolve_assumption_source(source,"structure.ddn").unwrap();
    assert_eq!(facts.len(),2);
    assert!(facts[0].goal().structure_law().is_some());
    assert!(facts[0].structure_consumption().is_none());
    assert!(facts[1].structure_consumption().unwrap().member_matches());
}

#[test]
fn ko_invalid_pin_diagnostic_marks_authored_label_in_utf8() {
    let base = format!("// 한글 😀\r\n{}", include_str!("structure_law_ko_surface_r1.ddn").replace('\n', "\r\n"));
    for (replacement, label) in [("다른자리=바른수그대로", "다른자리"),
        ("연산=바른수그대로, 연산=바른수그대로", "연산")] {
        let source = base.replace("연산=바른수그대로", replacement);
        let error = parse_v25_with_mode(&source, "위치.ddn", ParseMode::Strict).unwrap_err();
        let start = source.rfind(&format!("{label}=바른수그대로")).unwrap();
        assert!(error.message.starts_with("E_PROOF_STRUCTURE_DESCRIPTOR:"));
        assert_eq!((error.span.start, error.span.end), (start, start + label.len()));
    }
}

const COORDINATE_R12_SOURCE: &str = r###"(점:(바른수,바른수)~을) 직각회전하다{직각회전하기}:(바른수,바른수):셈씨 := {
  직각회전하다 <- (0 - 점[1], 점[0]).
}.

(점:(바른수,바른수)~을) 평행이동하다{평행이동하기}:(바른수,바른수):셈씨 := {
  평행이동하다 <- (점[0] + 10, 점[1] + 2).
}.

(점:(바른수,바른수)~을) 회전뒤이동하다{회전뒤이동하기}:(바른수,바른수):셈씨 := {
  회전뒤이동하다 <- (점=((점=점) 직각회전하기)) 평행이동하기.
}.

좌표가정 := (회전=직각회전하다, 이동=평행이동하다, 합성=회전뒤이동하다, 입력=(1,0)) 좌표합성가정짜임.

(
  우회전근거:(좌표가정.비교회전성공)보임~로,
  우가로근거:(좌표가정.비교가로성공)보임~로,
  우세로근거:(좌표가정.비교세로성공)보임~로,
  좌회전근거:(좌표가정.합성회전성공)보임~로,
  좌가로근거:(좌표가정.합성가로성공)보임~로,
  좌세로근거:(좌표가정.합성세로성공)보임~로,
  반환근거:(좌표가정.합성반환성공)보임~로,
  법칙근거:(좌표가정.적용합성법칙)보임~로
) 법칙전달하다{법칙전달하기}:(좌표가정.적용합성법칙)보임:셈씨 := {
  법칙전달하다 <- 법칙근거.
}.

(
  우회전근거:(좌표가정.비교회전성공)보임~로,
  우가로근거:(좌표가정.비교가로성공)보임~로,
  우세로근거:(좌표가정.비교세로성공)보임~로,
  좌회전근거:(좌표가정.합성회전성공)보임~로,
  좌가로근거:(좌표가정.합성가로성공)보임~로,
  좌세로근거:(좌표가정.합성세로성공)보임~로,
  반환근거:(좌표가정.합성반환성공)보임~로,
  법칙근거:(좌표가정.적용합성법칙)보임~로
) 근거로계산하다{근거로계산하기}:(바른수,바른수):셈씨 := {
  근거로계산하다 <- (점=(1,0)) 회전뒤이동하기.
}.
"###;

#[test]
fn coordinate_requires_actual_numeric_supply_and_same_parse_owner() {
    use ddonirang_lang::proof_authoring::{prepare_source,NumericSourceCandidate};
    let prepared=prepare_source(COORDINATE_R12_SOURCE,"좌표.ddn").unwrap();
    assert!(prepared.has_coordinate_contract());
    assert!(prepared.resolve().unwrap_err().message.contains("E_PROOF_NUMERIC_SUPPLY_REQUIRED"));
    let other=prepare_source(COORDINATE_R12_SOURCE,"좌표.ddn").unwrap();
    let foreign=NumericSourceCandidate{program:other.program(),expressions:vec![],operations:vec![]};
    assert!(prepared.resolve_with_numeric_candidate(&foreign).unwrap_err().message.contains("E_PROOF_NUMERIC_CONTEXT"));
    let empty=NumericSourceCandidate{program:prepared.program(),expressions:vec![],operations:vec![]};
    assert!(prepared.resolve_with_numeric_candidate(&empty).unwrap_err().message.contains("E_PROOF_NUMERIC_TYPE"));
}

#[test]
fn coordinate_input_label_does_not_relax_other_identifiers() {
    let cases = [
        "결과 := (입력=1) 다른호출.",
        "결과 := (입력=1) 좌표합성가정짜임.다른호출.",
        "입력 := 1.",
        "결과 := (입력=((입력=1) 다른호출)) 좌표합성가정짜임.",
        "결과 := (입력=(1,0)) 다른호출.",
        "결과 := (다른핀=입력) 좌표합성가정짜임.",
    ];
    for raw in cases {
        let source = format!("// 한글 😀\r\n{raw}");
        let error = parse_v25_with_mode(&source, "금지어.ddn", ParseMode::Strict).unwrap_err();
        assert!(error.message.contains("TERM-LINT-01"), "{raw}: {error:?}");
        assert_eq!(&source[error.span.start..error.span.end], "입력");
        if raw.contains("입력=((입력") {
            assert_eq!(error.span.start, source.rfind("입력").unwrap());
        }
    }
    let shadow = format!("좌표합성가정짜임 := 1.\n{COORDINATE_R12_SOURCE}");
    assert!(parse_v25_with_mode(&shadow, "가림.ddn", ParseMode::Strict).is_err());
}
