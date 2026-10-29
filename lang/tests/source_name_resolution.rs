use ddonirang_lang::name_resolution::{BindingState, DefinitionKind, NameResolutionFacts};
use ddonirang_lang::{parse_v25_with_mode, ParseMode};
fn parse(source: &str, path: &str) -> ddonirang_lang::CanonProgram {
    parse_v25_with_mode(source, path, ParseMode::Strict).unwrap()
}
fn key(
    facts: &NameResolutionFacts,
    offset: usize,
) -> ddonirang_lang::name_resolution::DefinitionKey {
    match facts.use_at(offset).unwrap().state() {
        BindingState::Final(k) => *k,
        other => panic!("not final: {other:?} at {offset}"),
    }
}
const SOURCE: &str =
    "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n결과 := 21을 두배하기. 결과 보여주기.\n";
#[test]
fn parameter_and_result_place_have_distinct_source_bound_definitions() {
    let p = parse(SOURCE, "한글.ddn");
    let f = p.origin.name_resolution().unwrap();
    let param = key(f, SOURCE.find("값:수").unwrap());
    let read = key(f, SOURCE.find("값 *").unwrap());
    assert_eq!(param, read);
    assert_eq!(
        f.definition(param).unwrap().kind(),
        DefinitionKind::Parameter
    );
    let result = key(f, SOURCE.find("두배 <-").unwrap());
    let d = f.definition(result).unwrap();
    assert_eq!(d.kind(), DefinitionKind::ResultPlace);
    let callable = d.owner_callable().unwrap();
    assert_ne!(callable, result);
    assert_eq!(
        f.definition(callable).unwrap().kind(),
        DefinitionKind::Callable
    );
    assert_eq!(d.name_span(), f.definition(callable).unwrap().name_span());
    let span = d.name_span().expect("actual declaration token");
    assert_eq!(&SOURCE[span.start..span.end], "두배");
    assert_eq!(span.start, SOURCE.find("두배:수").unwrap());
    let call = f
        .uses()
        .iter()
        .find(|u| u.spelling() == "두배하기")
        .unwrap();
    assert!(
        matches!(call.state(),BindingState::Candidates(c) if c.len()==1),
        "one candidate is not a final overload decision"
    );
}
#[test]
fn initializer_uses_outer_binding_and_nested_scope_never_leaks() {
    let source="값 := 1. { 참 }이면 { 값:수 := 값 + 1. 값 보여주기. } 아니면 { 값 보여주기. }. 값 보여주기.";
    let p = parse(source, "scope.ddn");
    let f = p.origin.name_resolution().unwrap();
    let root = key(f, 0);
    let local = key(f, source.find("값:수").unwrap());
    assert_ne!(root, local);
    assert_eq!(key(f, source.find("값 +").unwrap()), root);
    let shows = source
        .match_indices("값 보여주기")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert_eq!(shows.len(), 3);
    assert_eq!(key(f, shows[0]), local);
    assert_eq!(key(f, shows[1]), root);
    assert_eq!(key(f, shows[2]), root);
}
#[test]
fn parameter_and_outermost_local_duplicate_are_rejected() {
    let source = "(값:수~을) 두배:수:셈씨 := { 값:수 := 3. 두배 <- 값. }.";
    let e = parse_v25_with_mode(source, "duplicate.ddn", ParseMode::Strict).unwrap_err();
    assert!(e.message.contains("중복"), "{e:?}");
}
#[test]
fn same_root_value_and_callable_conflict_in_either_source_order() {
    let function = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.";
    for source in [
        format!("두배 := 1. {function}"),
        format!("{function} 두배 := 1."),
    ] {
        let e = parse_v25_with_mode(&source, "conflict.ddn", ParseMode::Strict).unwrap_err();
        assert!(e.message.contains("충돌"), "{e:?}");
    }
}
#[test]
fn verified_authored_call_owner_cannot_fallback_through_non_callable_local() {
    let source="(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n(값:수~을) 세배:수:셈씨 := { 두배:수 := 9. 세배 <- 1을 두배하기. }.";
    let e = parse_v25_with_mode(source, "shadow.ddn", ParseMode::Strict).unwrap_err();
    assert!(e.message.contains("가려진 호출"), "{e:?}");
    assert_eq!(&source[e.span.start..e.span.end], "두배하기");
}
#[test]
fn unicode_crlf_and_revision_changes_do_not_reuse_definition_keys() {
    let source = format!("// 😀 값\r\n{}", SOURCE.replace('\n', "\r\n"));
    let p = parse(&source, "u.ddn");
    let f = p.origin.name_resolution().unwrap();
    let start = source.find("값:수").unwrap();
    assert!(f.use_at(start + 1).is_none());
    let k = key(f, start);
    assert!(f.is_for_source(&source, "u.ddn"));
    assert!(!f.is_for_source(&source.replace("21", "22"), "u.ddn"));
    assert!(!f.is_for_source(&source, "other.ddn"));
    let changed = parse(&source.replace("21", "22"), "u.ddn");
    assert!(changed
        .origin
        .name_resolution()
        .unwrap()
        .definition(k)
        .is_none());
    let moved = parse(&source, "other.ddn");
    assert!(moved
        .origin
        .name_resolution()
        .unwrap()
        .definition(k)
        .is_none());
    assert!(f
        .uses()
        .iter()
        .all(|u| u.name_span().start >= "// 😀 값\r\n".len()));
}
#[test]
fn unrelated_spelling_is_not_promoted_to_a_shadow_relation() {
    let source="(값:수~을) 두배하다{두배하기}:수:셈씨 := { 두배하다 <- 값 * 2. }. (값:수~을) 세배하다{세배하기}:수:셈씨 := { 두배:수 := 9. 세배하다 <- 1을 두배하기. }.";
    let p = parse(source, "relation.ddn");
    let facts = p.origin.name_resolution().unwrap();
    let at = source.rfind("두배하기").unwrap();
    assert!(matches!(
        facts.use_at(at).unwrap().state(),
        BindingState::Candidates(_)
    ));
}
#[test]
fn named_pin_and_registered_alias_calls_do_not_bypass_shadow_rejection() {
    for source in [
        "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }. (값:수~을) 세배:수:셈씨 := { 두배:수 := 9. 세배 <- (값=1) 두배하기. }.",
        "(값:수~을) 두배하다{두배하기}:수:셈씨 := { 두배하다 <- 값 * 2. }. (값:수~을) 세배하다{세배하기}:수:셈씨 := { 두배하다:수 := 9. 세배하다 <- (값=1) 두배하기. }.",
    ]{
        let e=parse_v25_with_mode(source,"named-shadow.ddn",ParseMode::Strict).unwrap_err();assert!(e.message.contains("가려진 호출"),"{e:?}");assert_eq!(&source[e.span.start..e.span.end],"두배하기");
    }
}
#[test]
fn unknown_local_callable_type_is_explicitly_unsupported_without_global_fallback() {
    let source="(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }. (값:수~을) 세배:수:셈씨 := { 두배 := 값 * 3. 세배 <- (값=1) 두배하기. }.";
    let error =
        parse_v25_with_mode(source, "unresolved-shadow.ddn", ParseMode::Strict).unwrap_err();
    assert!(error.message.contains("형식이 확인되지"), "{error:?}");
    assert_eq!(&source[error.span.start..error.span.end], "두배하기");
}
