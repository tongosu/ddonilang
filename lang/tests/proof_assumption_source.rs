use ddonirang_lang::proof_authoring::resolve_assumption_source;
const RAW: &str = "(P:세움) 두고\n(증거=증거:(P)보임~을)\n가정그대로쓰다 : (P)보임 : 셈씨 := {\n    가정그대로쓰다 <- 증거.\n}.";

#[test]
fn opaque_jjaim_cannot_disappear_from_proof_admission() {
    let block = "짜임 { 미확정법칙 }.";
    for raw in [format!("// 한글 😀\r\n{block}\r\n{RAW}"), format!("{RAW}\r\n{block}")] {
        let program = ddonirang_lang::parse_v25(&raw, "짜임.ddn").unwrap();
        assert!(program.root_stmts.is_empty(), "일반 실행 문장으로 바꾸지 않는다");
        assert_eq!(program.origin.authored_jjaim_blocks.len(), 1);
        let error = resolve_assumption_source(&raw, "짜임.ddn").unwrap_err();
        assert!(error.message.starts_with("E_PROOF_ROOT_UNSUPPORTED:"));
        assert_eq!(&raw[error.span.start..error.span.end], block);
    }
    let commented = format!("// 짜임 {{ 미확정법칙 }}.\n{RAW}");
    assert!(resolve_assumption_source(&commented, "주석.ddn").is_ok());
    assert!(resolve_assumption_source(RAW, "정상.ddn").is_ok());
}

#[test]
fn actual_source_preserves_distinct_scopes_and_local_binding() {
    let first = resolve_assumption_source(RAW, "첫.ddn").unwrap();
    let second = resolve_assumption_source(RAW, "첫.ddn").unwrap();
    assert_ne!(first[0].scope(), second[0].scope());
    assert_ne!(first[0].goal(), second[0].goal());
    assert_eq!(first[0].assumptions(), &[first[0].returned().unwrap().clone()]);
    let renamed = RAW.replace("증거=증거", "공개증거=본문증거").replace("<- 증거", "<- 본문증거");
    resolve_assumption_source(&renamed, "별칭.ddn").unwrap();
}

#[test]
fn missing_evidence_and_wrong_result_are_source_errors() {
    for bad in [RAW.replace("<- 증거", "<- 없음"), RAW.replace("가정그대로쓰다 <- 증거.", ""),
        RAW.replace("<- 증거", "<- 가정그대로쓰다"), RAW.replace("<- 증거", "<- 참"),
        RAW.replace("<- 증거", "<- (P)보임 { 증거 }"),
        RAW.replace("증거=증거", "증거=_").replace("<- 증거", "<- _")] {
        assert!(resolve_assumption_source(&bad, "실패.ddn").is_err(), "{bad}");
    }
}

#[test]
fn korean_emoji_crlf_error_points_at_missing_evidence() {
    let raw = format!("// 한글 😀\r\n{}", RAW.replace('\n', "\r\n").replace("<- 증거", "<- 없음"));
    let error = resolve_assumption_source(&raw, "위치.ddn").unwrap_err();
    assert_eq!(&raw[error.span.start..error.span.end], "없음");
}

#[test]
fn shared_call_analysis_supplies_actual_owner_pins_and_utf8_range() {
    use ddonirang_lang::{parse_v25, ExprKind, Stmt, TopLevelItem};
    let source = format!("// 한글 😀\r\n{RAW}\r\n(Q:세움) 두고\r\n(근거=근거:(Q)보임~을)\r\n이어쓰다:(Q)보임:셈씨 := {{ 이어쓰다 <- (근거) 가정그대로쓰다. }}.");
    let program = parse_v25(&source, "호출.ddn").unwrap();
    let TopLevelItem::SeedDef(callee) = &program.items[0];
    let TopLevelItem::SeedDef(caller) = &program.items[1];
    let Stmt::Mutate { value, .. } = &caller.body.as_ref().unwrap().stmts[0] else { panic!("결과 기록") };
    let ExprKind::Call { args, .. } = &value.kind else { panic!("실제 호출") };
    assert_eq!(program.origin.authored_call_candidates.get(&value.id), Some(&vec![callee.id]));
    assert!(program.origin.registered_call_candidates.is_empty());
    assert_eq!(args[0].resolved_pin.as_deref(), Some("증거"));
    assert!(matches!(&args[0].expr.kind, ExprKind::Var(name) if name == "근거"));
    assert_eq!(&source[value.span.start..value.span.end], "(근거) 가정그대로쓰다");
    // Resolution retains the application; the proof crate must still check it.
    let resolved = resolve_assumption_source(&source, "호출.ddn").unwrap();
    assert!(resolved[1].application().is_some());
}
