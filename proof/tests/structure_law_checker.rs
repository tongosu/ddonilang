use ddonirang_lang::proof_authoring::resolve_assumption_source;
use ddonirang_proof::check_assumption_transfer;
const RAW:&str=include_str!("../../lang/tests/structure_law_r3.ddn");
const KO:&str=include_str!("../../lang/tests/structure_law_ko_surface_r1.ddn");
const CALC_CHAIN:&str="\n(값:바른수~을, 근거:(항등구조.법칙)보임~로) 이어계산:바른수:셈씨 := { 이어계산 <- (값=값, 근거=근거) 법칙필수계산. }.";

#[test]
fn s4_initial_identity_issues_from_definition_and_consumes_same_structure() {
    let facts=resolve_assumption_source(KO,"s4.ddn").unwrap();
    let initial=ddonirang_proof::CheckedAssumptionTransfer::check_initial_identity(&facts[0]).unwrap();
    initial.validate_source(KO,"s4.ddn").unwrap();
    let admitted=initial.admit_direct_calculation(&facts[1]).unwrap();
    admitted.validate_source(KO,"s4.ddn").unwrap();
    assert!(admitted.definition().structure_consumption().is_some());
    assert!(initial.admit_direct_calculation(&facts[0]).is_err());
    let reparsed=resolve_assumption_source(KO,"s4.ddn").unwrap();
    assert!(initial.admit_direct_calculation(&reparsed[1]).is_err());
    assert!(initial.validate_source(KO,"other.ddn").is_err());
    assert!(admitted.validate_source(&format!("{KO}\n"),"s4.ddn").is_err());
}

#[test]
fn s4_constant_seven_is_not_identity_even_when_the_sample_would_be_seven() {
    for raw in [
        KO.replace("그대로 <- 값.","그대로 <- 7."),
        KO.replace("바른수그대로 <- (값=값) 그대로.","바른수그대로 <- 7."),
        KO.replace("바른수그대로 <- (값=값) 그대로.","바른수그대로 <- (값=7) 그대로."),
    ] {
        // The product source producer already rejects these invalid bodies;
        // no sample execution is used to confer an identity law.
        assert!(resolve_assumption_source(&raw,"s4-constant.ddn").is_err());
    }
}

#[test]
fn s4_initial_consumption_rejects_other_structure_and_extra_hop() {
    let other="\n다른구조 := (바탕씨=바른수, 연산=바른수그대로) 항등법칙요구짜임.\n(값:바른수~을, 근거:(다른구조.법칙)보임~로) 다른계산:바른수:셈씨 := { 다른계산 <- (값=값) 바른수그대로. }.";
    let raw=format!("{KO}{other}{CALC_CHAIN}");
    let facts=resolve_assumption_source(&raw,"s4-context.ddn").unwrap();
    let initial=ddonirang_proof::CheckedAssumptionTransfer::check_initial_identity(&facts[0]).unwrap();
    assert!(initial.admit_direct_calculation(&facts[1]).is_ok());
    assert!(initial.admit_direct_calculation(&facts[2]).is_err());
    assert!(initial.admit_direct_calculation(&facts[3]).is_err());
    let missing=KO.replace(", 근거:(항등구조.법칙)보임~로","");
    assert!(resolve_assumption_source(&missing,"s4-missing.ddn").is_err());
}

#[test]
fn calculation_chain_checks_numeric_application_without_issuing_proof_return() {
    for tail in [CALC_CHAIN.to_string(), CALC_CHAIN.replace("(값=값, 근거=근거)","(근거=근거, 값=값)")] {
        let raw=format!("{KO}{tail}");
        let facts=resolve_assumption_source(&raw,"연쇄.ddn").unwrap();
        assert_eq!(facts.len(),3);
        for fact in &facts {check_assumption_transfer(fact).unwrap();}
        let checked=check_assumption_transfer(&facts[1]).unwrap();
        checked.check_calculation_in(&facts[2]).unwrap();
        assert!(checked.apply_in(&facts[2],&[(facts[1].goal().clone(),facts[2].goal().clone())],facts[2].assumptions()).is_err());
        let chained=check_assumption_transfer(&facts[2]).unwrap();
        assert!(chained.apply_in(&facts[0],&[(facts[2].goal().clone(),facts[0].goal().clone())],facts[0].assumptions()).is_err());
        let changed=raw.replace("값=값, 근거=근거","값=1, 근거=근거");
        if changed!=raw {assert!(chained.validate_source(&changed,"연쇄.ddn").is_err());}
        let other=resolve_assumption_source(&raw,"연쇄.ddn").unwrap();
        assert!(checked.check_calculation_in(&other[2]).is_err());
        let revised=raw.replace("바른수그대로","새연산");
        assert!(chained.validate_source(&revised,"연쇄.ddn").is_err());
        let revised_facts=resolve_assumption_source(&revised,"연쇄.ddn").unwrap();
        assert!(checked.check_calculation_in(&revised_facts[2]).is_err());
        let proof_only=check_assumption_transfer(&facts[0]).unwrap();
        assert!(proof_only.check_calculation_in(&facts[2]).is_err());
    }
}

#[test]
fn calculation_chain_rejects_wrong_bindings_result_kinds_and_extra_hops() {
    for tail in [
        CALC_CHAIN.replace("값=값, ",""),
        CALC_CHAIN.replace("값=값, ","값=값, 값=값, "),
        CALC_CHAIN.replace("값=값","값=\"글\""),
        CALC_CHAIN.replace("값=값","값=1"),
        CALC_CHAIN.replace("근거=근거","근거=참"),
        CALC_CHAIN.replace(", 근거:(항등구조.법칙)보임~로",""),
        CALC_CHAIN.replace("이어계산:바른수","이어계산:글"),
        CALC_CHAIN.replace("법칙필수계산.","이어계산."),
        CALC_CHAIN.replace("법칙필수계산.","법칙전달."),
        format!("{CALC_CHAIN}{}",CALC_CHAIN.replace("이어계산","또계산").replace("법칙필수계산.","이어계산.")),
    ] {
        let raw=format!("{KO}{tail}");
        assert!(resolve_assumption_source(&raw,"거부.ddn").is_err(),"{tail}");
    }
    let other="\n다른구조 := (바탕씨=바른수, 연산=바른수그대로) 항등법칙요구짜임.\n";
    let raw=format!("{KO}{other}{}",CALC_CHAIN.replace("항등구조.법칙","다른구조.법칙"));
    assert!(resolve_assumption_source(&raw,"다른구조.ddn").is_err());
    assert!(resolve_assumption_source(&format!("{CALC_CHAIN}{KO}"),"전방.ddn").is_err());
}

#[test]
fn ko_surface_checks_same_member_and_actual_proof_caller() {
    let caller="\n(근거:(항등구조.법칙)보임~을) 이어전달:(항등구조.법칙)보임:셈씨 := { 이어전달 <- (근거=근거) 법칙전달. }.";
    for raw in [KO.to_string(), KO.replace("바탕씨=바른수, 연산=바른수그대로", "연산=바른수그대로, 바탕씨=바른수"), format!("{KO}{caller}")] {
        let facts=resolve_assumption_source(&raw,"한글.ddn").unwrap();
        for fact in &facts {check_assumption_transfer(fact).unwrap();}
        assert!(facts[1].structure_consumption().unwrap().member_matches());
    }
}

#[test]
fn ko_surface_rejects_invalid_pins_static_references_and_fake_evidence() {
    for (from,to,code) in [
        ("바탕씨=바른수, ","","E_PROOF_STRUCTURE_DESCRIPTOR"),
        ("연산=바른수그대로","연산=바른수그대로, 연산=바른수그대로","E_PROOF_STRUCTURE_DESCRIPTOR"),
        ("연산=바른수그대로","다른자리=바른수그대로","E_PROOF_STRUCTURE_DESCRIPTOR"),
        ("바탕씨=바른수","바탕씨=글","E_PROOF_STRUCTURE_TYPE"),
        ("연산=바른수그대로","연산=\"바른수그대로\"","E_PROOF_STRUCTURE_MEMBER"),
        ("연산=바른수그대로","연산=없는연산","E_PROOF_STRUCTURE_MEMBER"),
        ("법칙전달 <- 근거.","법칙전달 <- 참.","E_PROOF_EVIDENCE"),
        (", 근거:(항등구조.법칙)보임~로","","E_PROOF_ASSUMPTION"),
        ("그대로 <- 값.","그대로 <- 7.","E_PROOF_STRUCTURE_POLICY"),
    ] {
        let raw=KO.replace(from,to);
        let error=resolve_assumption_source(&raw,"한글.ddn").unwrap_err();
        assert!(error.message.starts_with(code),"{}: {}",code,error.message);
        assert!(error.span.start<error.span.end && error.span.end<=raw.len());
    }
}

#[test]
fn ko_surface_cannot_fallback_after_shadow_or_mix_structure_evidence() {
    for prefix in [
        "항등법칙요구짜임 := 1.\n",
        "(바탕씨:바른수~을, 연산:바른수~로) 항등법칙요구짜임:바른수:셈씨 := { 항등법칙요구짜임 <- 1. }.\n",
        "바른수 := 1.\n",
        "바른수그대로 := 1.\n",
    ] {assert!(resolve_assumption_source(&format!("{prefix}{KO}"),"한글.ddn").is_err());}
    let first=resolve_assumption_source(KO,"한글.ddn").unwrap();
    let second=resolve_assumption_source(&KO.replace("항등구조","다른구조"),"한글.ddn").unwrap();
    let checked=check_assumption_transfer(&first[0]).unwrap();
    assert!(checked.apply_in(&second[0],&[(first[0].goal().clone(),second[0].goal().clone())],second[0].assumptions()).is_err());
    let old=resolve_assumption_source(RAW,"원문.ddn").unwrap();
    assert!(checked.apply_in(&old[0],&[(first[0].goal().clone(),old[0].goal().clone())],old[0].assumptions()).is_err());
    assert!(resolve_assumption_source(&RAW.replace("  제공자판: 1,\n",""),"원문.ddn").is_err());
}

#[test]
fn ko_surface_cannot_reuse_checked_explicit_source_evidence() {
    let ko = include_str!("../../lang/tests/structure_law_ko_surface_r1.ddn");
    let facts = resolve_assumption_source(RAW, "structure.ddn").unwrap();
    for fact in &facts {
        let checked = check_assumption_transfer(fact).unwrap();
        checked.validate_source(RAW, "structure.ddn").unwrap();
        let error = checked.validate_source(ko, "structure.ddn").unwrap_err();
        assert!(error.message.starts_with("E_PROOF_STALE_SOURCE:"));
    }
}

#[test]
fn actual_caller_cannot_substitute_another_fixed_structure() {
    let caller="\n(근거:(항등구조.법칙)보임~을) 이어전달:(항등구조.법칙)보임:셈씨 := { 이어전달 <- (근거=근거) 법칙전달. }.";
    let raw=format!("{RAW}{caller}");
    let facts=resolve_assumption_source(&raw,"structure.ddn").unwrap();
    assert_eq!(facts.len(),3);
    for fact in &facts {check_assumption_transfer(fact).unwrap();}
    let other=RAW.replace("항등구조 := (차림새=명시단항차림새) 짜임.",
        "항등구조 := (차림새=명시단항차림새) 짜임.\n다른구조 := (차림새=명시단항차림새) 짜임.");
    let bad=format!("{other}{}",caller.replace("항등구조.법칙","다른구조.법칙"));
    assert!(resolve_assumption_source(&bad,"structure.ddn").is_err());
    // The public checker application API must enforce the same fixed goal.
    let other_facts=resolve_assumption_source(&RAW.replace("항등구조","다른구조"),"other.ddn").unwrap();
    let checked=check_assumption_transfer(&facts[0]).unwrap();
    assert!(checked.apply_in(&other_facts[0],&[(facts[0].goal().clone(),other_facts[0].goal().clone())],other_facts[0].assumptions()).is_err());
}

#[test]
fn explicit_law_and_same_member_consumption_are_checked() {
    let facts=resolve_assumption_source(RAW,"structure.ddn").unwrap();
    assert_eq!(facts.len(),2);
    for fact in &facts {check_assumption_transfer(fact).unwrap();}
    assert!(facts[1].structure_consumption().is_some());
    assert_eq!(facts[1].assumptions().len(),1);
    assert!(!facts[1].source_matches(&RAW.replace("차림새판: 1","차림새판: 2"),"structure.ddn"));
}

#[test]
fn missing_law_and_unsupported_operation_policy_are_rejected() {
    for raw in [RAW.replace(", 근거:(항등구조.법칙)보임~로",""),
        RAW.replace("그대로 <- 값.","그대로 <- 7."),
        RAW.replace("제공자판: 1","제공자판: 2"),
        RAW.replace("법칙전달 <- 근거.","법칙전달 <- 참.")] {
        assert!(resolve_assumption_source(&raw,"structure.ddn").is_err());
    }
}

#[test]
fn constructor_shadow_and_false_descriptor_type_do_not_gain_admission() {
    let shadow=RAW.replace("그대로:값씨", "짜임:값씨")
        .replace("(값=값) 그대로.", "(값=값) 짜임.");
    assert!(resolve_assumption_source(&shadow,"structure.ddn").is_err());
    let false_type=RAW.replace("명시단항차림새 :=", "명시단항차림새:글 :=");
    let error=resolve_assumption_source(&false_type,"structure.ddn").unwrap_err();
    assert!(error.message.starts_with("E_PROOF_STRUCTURE_TYPE:"));
}

#[test]
fn same_operation_in_another_structure_does_not_replace_goal() {
    let raw=RAW.replace("항등구조 := (차림새=명시단항차림새) 짜임.",
        "항등구조 := (차림새=명시단항차림새) 짜임.\n다른구조 := (차림새=명시단항차림새) 짜임.")
        .replace("법칙전달:(항등구조.법칙)보임", "법칙전달:(다른구조.법칙)보임");
    assert!(resolve_assumption_source(&raw,"structure.ddn").is_err());
}
#[test]
fn typed_math_candidates_reject_bad_selection_and_application_types() {
    use ddonirang_symbolic::typed_math::{Term,TermType};
    let signatures=[(TermType::IntegerPair,TermType::IntegerPair);3];
    let p=Term::Pair(Box::new(Term::Integer(1)),Box::new(Term::Integer(0)));
    let rotation=Term::Apply{definition:0,argument:Box::new(p.clone())};
    assert_eq!(rotation.term_type(&signatures),Some(TermType::IntegerPair));
    assert_eq!(Term::Select{target:Box::new(p.clone()),index:2}.term_type(&signatures),None);
    assert_eq!(Term::Apply{definition:0,argument:Box::new(Term::Integer(1))}.term_type(&signatures),None);
    assert_eq!(Term::Apply{definition:3,argument:Box::new(p)}.term_type(&signatures),None);
    assert_eq!(Term::Add(Box::new(rotation),Box::new(Term::Integer(1))).term_type(&signatures),None);
}

#[test]
fn typed_math_candidates_do_not_promote_unfinished_or_overdeep_terms() {
    use ddonirang_symbolic::typed_math::{Term,TermType};
    let signatures=[(TermType::IntegerPair,TermType::IntegerPair);3];
    assert_eq!(Term::Parameter{definition:0,pin:1}.term_type(&signatures),None);
    let mut term=Term::Integer(1);
    for _ in 0..34 {term=Term::Add(Box::new(term),Box::new(Term::Integer(0)));}
    assert_eq!(term.term_type(&signatures),None);
}


#[test]
fn canonical_structure_sources_preserve_checker_admission_and_reject_stale_receipts() {
    use ddonirang_lang::{parse_v25_with_mode, ParseMode, normalizer::{Normalizer,NormalizationLevel}};
    for raw in [RAW.to_string(), KO.to_string(), format!("{KO}{CALC_CHAIN}")] {
        let original=resolve_assumption_source(&raw,"정본화.ddn").unwrap();
        let before:Vec<_>=original.iter().map(|d|check_assumption_transfer(d).unwrap()).collect();
        let program=parse_v25_with_mode(&raw,"정본화.ddn",ParseMode::Strict).unwrap();
        let canonical=Normalizer::new(NormalizationLevel::N1).normalize_program(&program);
        let reparsed=parse_v25_with_mode(&canonical,"정본화.ddn",ParseMode::Strict).unwrap();
        assert_eq!(canonical,Normalizer::new(NormalizationLevel::N1).normalize_program(&reparsed));
        let after=resolve_assumption_source(&canonical,"정본화.ddn").unwrap();
        assert_eq!(before.len(),after.len());
        for (old,new) in before.iter().zip(&after) {
            check_assumption_transfer(new).unwrap();
            if raw!=canonical {assert!(old.validate_source(&canonical,"정본화.ddn").is_err());}
        }
    }
}

#[test]
fn canonicalization_does_not_hoist_a_late_structure_into_scope_or_repair_false_evidence() {
    use ddonirang_lang::{parse_v25_with_mode, ParseMode, normalizer::{Normalizer,NormalizationLevel}};
    let declaration="항등구조 := (바탕씨=바른수, 연산=바른수그대로) 항등법칙요구짜임.";
    let late=format!("{}\n{declaration}",KO.replace(declaration,""));
    let false_evidence=KO.replace("법칙전달 <- 근거", "법칙전달 <- 참");
    for raw in [late,false_evidence] {
        let accepts=|text:&str|resolve_assumption_source(text,"거부.ddn").is_ok_and(|items|items.iter().all(|d|check_assumption_transfer(d).is_ok()));
        assert!(!accepts(&raw));
        let program=parse_v25_with_mode(&raw,"거부.ddn",ParseMode::Strict).unwrap();
        let canonical=Normalizer::new(NormalizationLevel::N1).normalize_program(&program);
        assert!(!accepts(&canonical),"{canonical}");
    }
}
