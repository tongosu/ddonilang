use ddonirang_lang::proof_authoring::resolve_assumption_source;
use ddonirang_proof::check_assumption_transfer;
const RAW: &str = "(P:세움) 두고\n(증거=증거:(P)보임~을)\n가정그대로쓰다 : (P)보임 : 셈씨 := { 가정그대로쓰다 <- 증거. }.";
const CALLER: &str = "(Q:세움) 두고 (근거=근거:(Q)보임~을) 이어쓰다:(Q)보임:셈씨 := { 이어쓰다 <- (근거) 가정그대로쓰다. }.";

#[test]
fn authored_application_and_next_consumer_are_actually_checked() {
    let third = CALLER.replace("Q", "R").replace("가정그대로쓰다", "이어쓰다")
        .replace("이어쓰다:(R)", "마지막:(R)").replace("이어쓰다 <-", "마지막 <-");
    let source = format!("{RAW}\n{CALLER}\n{third}");
    let facts = resolve_assumption_source(&source, "연결.ddn").unwrap();
    assert_eq!(facts.len(), 3);
    for fact in &facts { check_assumption_transfer(fact).unwrap(); }
    let application = facts[2].application().unwrap();
    assert!(application.definition().application().is_some());
    assert_eq!(facts[2].assumptions().len(), 1);
    assert_ne!(application.definition().goal(), facts[2].goal());
}

#[test]
fn source_application_rejects_conflict_unresolved_and_wrong_goal() {
    let missing = format!("{}\n{CALLER}", RAW.replace("P:세움", "P:세움, 숨김:세움"));
    assert!(resolve_assumption_source(&missing, "미결.ddn").unwrap_err().message.starts_with("E_PROOF_INFERENCE_UNRESOLVED"));
    let wrong = format!("{RAW}\n{}", CALLER.replace("Q:세움", "Q:세움, R:세움")
        .replace("이어쓰다:(Q)", "이어쓰다:(R)"));
    let facts = resolve_assumption_source(&wrong, "다른목표.ddn").unwrap();
    assert!(check_assumption_transfer(&facts[1]).is_err());
    let two = RAW.replace("증거:(P)보임~을)", "증거:(P)보임~을, 둘=둘:(P)보임~로)");
    let conflicting = CALLER.replace("Q:세움", "Q:세움, R:세움")
        .replace("근거:(Q)보임~을)", "근거:(Q)보임~을, 다른=다른:(R)보임~로)")
        .replace("(근거) 가정그대로쓰다", "(근거, 다른) 가정그대로쓰다");
    let source = format!("// 한글 😀\r\n{two}\r\n{conflicting}");
    let error = resolve_assumption_source(&source, "충돌.ddn").unwrap_err();
    assert!(error.message.starts_with("E_GENERIC_UNIFY_CONFLICT"));
    assert_eq!(&source[error.span.start..error.span.end], "다른");
    for bad in [CALLER.replace("(근거) 가정", "(참) 가정"),
        CALLER.replace("(근거) 가정", "((검증: 참)) 가정"),
        CALLER.replace("(근거) 가정", "() 가정")] {
        assert!(resolve_assumption_source(&format!("{RAW}\n{bad}"), "거부.ddn").is_err());
    }
}

#[test]
fn source_application_never_trusts_an_invalid_callee_or_unfinished_cycle() {
    let invalid = RAW.replace("P:세움", "P:세움, Q:세움")
        .replace("증거:(P)보임~을)", "증거:(P)보임~을, 둘=둘:(Q)보임~로)")
        .replace("가정그대로쓰다 : (P)", "가정그대로쓰다 : (Q)");
    let caller = CALLER.replace("Q:세움", "Q:세움, R:세움")
        .replace("근거:(Q)보임~을)", "근거:(Q)보임~을, 다른=다른:(R)보임~로)")
        .replace("(근거) 가정그대로쓰다", "(근거, 다른) 가정그대로쓰다");
    let facts = resolve_assumption_source(&format!("{invalid}\n{caller}"), "거짓정의.ddn").unwrap();
    // The caller's projected returned assumption matches Q, but the callee
    // falsely claims Q from P. Checking only the projection would accept it.
    assert_eq!(facts[1].returned().unwrap().goal(), facts[1].goal());
    assert!(check_assumption_transfer(&facts[1]).is_err());
    let recursive = RAW.replace("<- 증거", "<- (증거) 가정그대로쓰다");
    assert!(resolve_assumption_source(&recursive, "순환.ddn").unwrap_err().message.starts_with("E_PROOF_CALL_UNSUPPORTED"));
    let mut chain = RAW.replace("가정그대로쓰다", "전달0");
    for index in 1..=65 {
        chain.push_str(&CALLER.replace("가정그대로쓰다", &format!("전달{}", index - 1))
            .replace("이어쓰다", &format!("전달{index}")));
    }
    assert!(resolve_assumption_source(&chain, "한도.ddn").unwrap_err().message.starts_with("E_PROOF_BUDGET"));
}
#[test]
fn checker_retains_assumptions_and_rejects_stale_source() {
    let facts = resolve_assumption_source(RAW, "입력.ddn").unwrap();
    let checked = check_assumption_transfer(&facts[0]).unwrap();
    assert_eq!(checked.resolved().assumptions().len(), 1);
    checked.validate_source(RAW, "입력.ddn").unwrap();
    assert!(checked.validate_source(&RAW.replace("<- 증거", "<- 참"), "입력.ddn").is_err());
    assert!(checked.validate_source(RAW, "다른.ddn").is_err());
}
#[test]
fn actual_checker_rejects_another_proposition() {
    let raw = RAW.replace("P:세움", "P:세움, Q:세움").replace("증거:(P)보임", "증거:(Q)보임");
    let facts = resolve_assumption_source(&raw, "목표.ddn").unwrap();
    assert!(check_assumption_transfer(&facts[0]).is_err());
}

#[test]
fn checked_application_preserves_both_contexts_and_source_dependencies() {
    let callee = resolve_assumption_source(RAW, "정의.ddn").unwrap().remove(0);
    let caller_source = RAW.replace("P", "Q").replace("가정그대로쓰다", "이어쓰다");
    let caller = resolve_assumption_source(&caller_source, "호출.ddn").unwrap().remove(0);
    let checked = check_assumption_transfer(&callee).unwrap();
    let substitution = [(callee.goal().clone(), caller.goal().clone())];
    let application = checked.apply_in(&caller, &substitution, caller.assumptions()).unwrap();
    assert_eq!(application.goal(), caller.goal());
    assert_eq!(application.definition().resolved().scope(), callee.scope());
    assert_eq!(application.caller().assumptions(), caller.assumptions());
    assert_eq!(application.arguments(), caller.assumptions());
    assert_eq!(application.substitution(), &substitution);
    application.validate_sources(RAW, "정의.ddn", &caller_source, "호출.ddn").unwrap();
    assert!(application.validate_sources(RAW, "다른.ddn", &caller_source, "호출.ddn").is_err());
    assert!(application.validate_sources(RAW, "정의.ddn", RAW, "호출.ddn").is_err());
}

#[test]
fn application_rejects_missing_extra_and_foreign_evidence() {
    let callee = resolve_assumption_source(RAW, "정의.ddn").unwrap().remove(0);
    let caller = resolve_assumption_source(RAW, "호출.ddn").unwrap().remove(0);
    // Identical bytes, path and spelling still do not identify this declaration.
    let foreign = resolve_assumption_source(RAW, "호출.ddn").unwrap().remove(0);
    let checked = check_assumption_transfer(&callee).unwrap();
    let substitution = [(callee.goal().clone(), caller.goal().clone())];
    for arguments in [vec![], vec![caller.returned().unwrap().clone(), caller.returned().unwrap().clone()]] {
        assert!(checked.apply_in(&caller, &substitution, &arguments).unwrap_err()
            .message.starts_with("E_PROOF_ARGUMENT_COUNT"));
    }
    let error = checked.apply_in(&caller, &substitution, foreign.assumptions()).unwrap_err();
    assert!(error.message.starts_with("E_PROOF_ASSUMPTION"));
    assert_eq!(error.span, caller.result_span());
    assert!(checked.apply_in(&caller, &[], caller.assumptions()).is_err());
    assert!(checked.apply_in(&caller, &[substitution[0].clone(), substitution[0].clone()],
        caller.assumptions()).is_err());
    assert!(checked.apply_in(&caller, &[(callee.goal().clone(), foreign.goal().clone())],
        caller.assumptions()).is_err());
}

#[test]
fn application_checks_unused_assumptions_and_each_distinct_proposition() {
    let raw = "(P:세움, Q:세움) 두고\n(증거=증거:(P)보임~을, 곁증거=곁증거:(Q)보임~로)\n가정그대로쓰다 : (P)보임 : 셈씨 := { 가정그대로쓰다 <- 증거. }.";
    let callee = resolve_assumption_source(raw, "정의.ddn").unwrap().remove(0);
    let caller = resolve_assumption_source(raw, "호출.ddn").unwrap().remove(0);
    let checked = check_assumption_transfer(&callee).unwrap();
    let substitution: Vec<_> = callee.assumptions().iter().zip(caller.assumptions())
        .map(|(a, b)| (a.goal().clone(), b.goal().clone())).collect();
    checked.apply_in(&caller, &substitution, caller.assumptions()).unwrap();
    let swapped = [caller.assumptions()[1].clone(), caller.assumptions()[0].clone()];
    assert!(checked.apply_in(&caller, &substitution, &swapped).is_err());
    assert!(checked.apply_in(&caller, &substitution[..1], caller.assumptions()).is_err());
    assert!(checked.apply_in(&caller, &substitution, &caller.assumptions()[..1]).is_err());
    let same_goal_source = raw.replace("(Q)보임", "(P)보임");
    let same_goal = resolve_assumption_source(&same_goal_source, "같은목표.ddn").unwrap().remove(0);
    let same_checked = check_assumption_transfer(&same_goal).unwrap();
    assert!(same_checked.apply_in(&caller,
        &[(same_goal.goal().clone(), caller.goal().clone())], caller.assumptions()).is_err());
}
