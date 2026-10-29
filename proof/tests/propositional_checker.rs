use ddonirang_lang::proof_authoring::resolve_assumption_source;
use ddonirang_proof::{check_propositional_evidence as check, PropositionalEvidence as E, PropositionalGoal as G};

const SOURCE: &str = "(P:세움, Q:세움, R:세움) 두고 (첫=첫:(P)보임~을, 둘=둘:(Q)보임~로, 셋=셋:(R)보임~에) 전달하다:(P)보임:셈씨 := { 전달하다 <- 첫. }.";

const PROJECT: &str = "(P:세움, Q:세움) 두고 (근거=근거:(P 이고 Q)보임~을) 첫근거쓰다:(P)보임:셈씨 := { 첫근거쓰다 <- 근거[0]. }.";

const CONSTRUCT: &str = "(P:세움, Q:세움) 두고 (왼=왼:(P)보임~을, 오른=오른:(Q)보임~로) 모으다:(P 이고 Q)보임:셈씨 := { 모으다 <- (왼, 오른). }.";

#[test]
fn authored_construction_projection_and_consumption_check_the_complete_tree() {
    let project = "(A:세움, B:세움) 두고 (첫=첫:(A)보임~을, 둘=둘:(B)보임~로) 꺼내다:(A)보임:셈씨 := { 꺼내다 <- ((첫, 둘) 모으다)[0]. }.";
    let next = project.replace("꺼내다", "이어쓰다").replace("((첫, 둘) 모으다)[0]", "(첫, 둘) 꺼내다");
    let source = format!("{CONSTRUCT}\n{project}\n{next}");
    let facts = resolve_assumption_source(&source, "생성소비.ddn").unwrap();
    assert_eq!(facts.len(), 3);
    for fact in &facts { ddonirang_proof::check_assumption_transfer(fact).unwrap(); }
    assert!(facts[0].returned().is_none());
    let parsed = ddonirang_lang::parse_v25_with_mode(&source,"생성소비.ddn",ddonirang_lang::ParseMode::Strict).unwrap();
    let formatted = ddonirang_lang::normalize(&parsed,ddonirang_lang::NormalizationLevel::N1);
    for fact in resolve_assumption_source(&formatted,"왕복.ddn").unwrap() {
        ddonirang_proof::check_assumption_transfer(&fact).unwrap();
    }
    for bad in [source.replace("(왼, 오른).", "(왼, 왼)."), source.replace("(왼, 오른).", "(오른, 왼)."),
        source.replace("(왼, 오른).", "(왼, 참)."), source.replace("(왼, 오른).", "(왼, 외부).") ] {
        assert!(resolve_assumption_source(&bad,"둘째항거부.ddn").is_err());
    }
}

#[test]
fn authored_construction_preserves_nary_nested_and_repeated_goals() {
    let repeat = CONSTRUCT.replace("P 이고 Q", "P 이고 P").replace("(왼, 오른).", "(왼, 왼).");
    let nested = CONSTRUCT.replace("P 이고 Q", "P 이고 (Q 이고 P)")
        .replace("(왼, 오른).", "(왼, (오른, 왼)).");
    let flat = CONSTRUCT.replace("P 이고 Q", "P 이고 Q 이고 P").replace("(왼, 오른).", "(왼, 오른, 왼).");
    for source in [&repeat,&nested,&flat] {
        let facts = resolve_assumption_source(source,"생성구조.ddn").unwrap();
        ddonirang_proof::check_assumption_transfer(&facts[0]).unwrap();
    }
    assert!(resolve_assumption_source(&nested.replace("(왼, (오른, 왼)).", "(왼, 오른, 왼)."),"구조거부.ddn").is_err());
    assert!(resolve_assumption_source(&CONSTRUCT.replace("(P 이고 Q)보임:셈씨", "(P)보임:셈씨"),"목표없음.ddn").is_err());
}

#[test]
fn authored_conjunction_projection_and_call_use_the_product_resolver() {
    let caller = "(A:세움, B:세움) 두고 (증거=증거:(A 이고 B)보임~을) 이어쓰다:(A)보임:셈씨 := { 이어쓰다 <- (증거) 첫근거쓰다. }.";
    let facts = resolve_assumption_source(&format!("{PROJECT}\n{caller}"), "논리곱.ddn").unwrap();
    for fact in &facts { ddonirang_proof::check_assumption_transfer(fact).unwrap(); }
    assert_eq!(facts[1].projection(), &[0]);
    assert_eq!(facts[1].assumptions()[0].goal().conjunction_parts().unwrap().len(), 2);
    let nested = PROJECT.replace("Q:세움", "Q:세움, R:세움")
        .replace("P 이고 Q", "P 이고 (Q 이고 R)").replace("첫근거쓰다:(P)", "첫근거쓰다:(Q)")
        .replace("근거[0]", "근거[1][0]");
    let facts = resolve_assumption_source(&nested, "중첩.ddn").unwrap();
    ddonirang_proof::check_assumption_transfer(&facts[0]).unwrap();
    let parsed = ddonirang_lang::parse_v25_with_mode(&nested, "중첩.ddn", ddonirang_lang::ParseMode::Strict).unwrap();
    let formatted = ddonirang_lang::normalize(&parsed, ddonirang_lang::NormalizationLevel::N1);
    let roundtrip = resolve_assumption_source(&formatted, "정규화.ddn").unwrap();
    ddonirang_proof::check_assumption_transfer(&roundtrip[0]).unwrap();
    assert_eq!(roundtrip[0].projection(), &[1,0]);
}

#[test]
fn authored_projection_rejects_wrong_goal_shape_missing_premise_and_constructor() {
    for raw in [PROJECT.replace("첫근거쓰다:(P)", "첫근거쓰다:(Q)"),
        PROJECT.replace("근거[0]", "근거[2]"), PROJECT.replace("근거[0]", "근거[0,1]"),
        PROJECT.replace("근거[0]", "근거[참]"), PROJECT.replace("P 이고 Q", "P 이고 미선언"),
        PROJECT.replace("근거[0]", "(근거,근거)")] {
        match resolve_assumption_source(&raw, "거부.ddn") {
            Err(e) => assert!(e.span.start < e.span.end),
            Ok(facts) => assert!(ddonirang_proof::check_assumption_transfer(&facts[0]).is_err()),
        }
    }
    let flat = PROJECT.replace("Q:세움", "Q:세움, R:세움").replace("P 이고 Q", "P 이고 Q 이고 R")
        .replace("첫근거쓰다:(P)", "첫근거쓰다:(Q 이고 R)").replace("근거[0]", "근거[1]");
    let facts = resolve_assumption_source(&flat, "펼침거부.ddn").unwrap();
    assert!(ddonirang_proof::check_assumption_transfer(&facts[0]).is_err());
}

#[test]
fn ordered_construction_projection_and_next_consumption_preserve_context() {
    let facts = resolve_assumption_source(SOURCE, "논리문맥.ddn").unwrap();
    let c = &facts[0];
    let a = c.assumptions();
    let p = G::Atom(a[0].goal().clone());
    let q = G::Atom(a[1].goal().clone());
    let pair = E::Conjunction(vec![E::Assumption(a[0].clone()), E::Assumption(a[1].clone())]);
    let checked = check(c, &G::Conjunction(vec![p.clone(), q.clone()]), &pair).unwrap();
    let projection = E::Projection { evidence: Box::new(checked.evidence().clone()), index: 0 };
    let projected = check(c, &p, &projection).unwrap();
    let next = E::Conjunction(vec![projected.evidence().clone(), E::Assumption(a[1].clone())]);
    check(c, &G::Conjunction(vec![p.clone(), q.clone()]), &next).unwrap();
    assert_eq!(projected.context().assumptions(), a); // unused R is retained
    assert!(check(c, &G::Conjunction(vec![q, p]), &pair).is_err());
    projected.validate_source(SOURCE, "논리문맥.ddn").unwrap();
    assert!(projected.validate_source(&(SOURCE.to_owned()+" "), "논리문맥.ddn").is_err());
    assert!(projected.validate_source(SOURCE, "다른문맥.ddn").is_err());
}

#[test]
fn nested_products_never_flatten_and_indices_are_checked() {
    let facts = resolve_assumption_source(SOURCE, "논리문맥.ddn").unwrap();
    let c = &facts[0];
    let terms: Vec<_> = c.assumptions().iter().cloned().map(E::Assumption).collect();
    let goals: Vec<_> = c.assumptions().iter().map(|a| G::Atom(a.goal().clone())).collect();
    let nested = E::Conjunction(vec![E::Conjunction(terms[..2].to_vec()), terms[2].clone()]);
    let target = G::Conjunction(vec![G::Conjunction(goals[..2].to_vec()), goals[2].clone()]);
    check(c, &target, &nested).unwrap();
    assert!(check(c, &G::Conjunction(goals.clone()), &nested).is_err());
    for index in [2, usize::MAX] {
        assert!(check(c, &goals[0], &E::Projection { evidence: Box::new(nested.clone()), index }).is_err());
    }
    assert!(check(c, &goals[0], &E::Projection { evidence: Box::new(terms[0].clone()), index: 0 }).is_err());
    let first_pair = E::Projection { evidence: Box::new(nested), index: 0 };
    check(c, &goals[1], &E::Projection { evidence: Box::new(first_pair), index: 1 }).unwrap();
}

#[test]
fn projection_checks_discarded_evidence_and_rejects_same_spelling_foreign_scope() {
    let one = resolve_assumption_source(SOURCE, "논리문맥.ddn").unwrap();
    let other = resolve_assumption_source(SOURCE, "논리문맥.ddn").unwrap();
    let c = &one[0];
    let p = G::Atom(c.assumptions()[0].goal().clone());
    let forged = E::Conjunction(vec![E::Assumption(c.assumptions()[0].clone()), E::Assumption(other[0].assumptions()[1].clone())]);
    let error = check(c, &p, &E::Projection { evidence: Box::new(forged), index: 0 }).unwrap_err();
    assert!(error.message.starts_with("E_PROOF_ASSUMPTION"));
    let foreign_goal = G::Atom(other[0].assumptions()[0].goal().clone());
    assert!(check(c, &foreign_goal, &E::Assumption(c.assumptions()[0].clone())).is_err());
}

#[test]
fn incomplete_products_and_budget_exhaustion_do_not_issue_evidence() {
    let facts = resolve_assumption_source(SOURCE, "논리문맥.ddn").unwrap();
    let c = &facts[0];
    let p = G::Atom(c.assumptions()[0].goal().clone());
    let atom = E::Assumption(c.assumptions()[0].clone());
    for items in [vec![], vec![atom.clone()]] {
        assert!(check(c, &p, &E::Conjunction(items)).unwrap_err().message.starts_with("E_PROOF_RULE_UNSUPPORTED"));
    }
    let mut deep = atom.clone();
    for _ in 0..65 { deep = E::Projection { evidence: Box::new(E::Conjunction(vec![deep, atom.clone()])), index: 0 }; }
    assert!(check(c, &p, &deep).unwrap_err().message.starts_with("E_PROOF_BUDGET"));
    let wide = E::Projection { evidence: Box::new(E::Conjunction(vec![atom; 4097])), index: 0 };
    assert!(check(c, &p, &wide).unwrap_err().message.starts_with("E_PROOF_BUDGET"));
}
