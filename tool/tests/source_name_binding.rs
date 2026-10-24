//! Name binding A1 on the measured rename counterexamples (R1~R11) and the
//! cases the A1 scope must refuse. Exact results are checked by applying the
//! edits and running the same program; unsupported results name the reason.
use ddonirang_tool::name_binding::{
    bind_document, BindingOutcome, ByteRange, DefinitionKind, NameBindingReport, NameDefinition,
    OccurrenceRole, Resolution, UnsupportedReason,
};
use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;
use ddonirang_tool::source_analysis::{analyze_source, SourceAnalysisRequest};

const PATH: &str = "name-binding.ddn";

fn callable<'a>(report: &'a NameBindingReport, name: &str) -> &'a NameDefinition {
    report
        .definitions
        .iter()
        .find(|d| d.kind == DefinitionKind::TopLevelCallable && d.name == name)
        .unwrap_or_else(|| panic!("no callable {name}: {report:#?}"))
}

fn exact(report: &NameBindingReport, name: &str) -> Vec<(OccurrenceRole, ByteRange)> {
    match &callable(report, name).resolution {
        Resolution::Exact(occurrences) => occurrences.iter().map(|o| (o.role, o.name_range)).collect(),
        other => panic!("{name} should be exact: {other:#?}"),
    }
}

fn reasons(report: &NameBindingReport, name: &str) -> Vec<UnsupportedReason> {
    match &callable(report, name).resolution {
        Resolution::Unsupported(sites) => sites.iter().map(|s| s.reason.clone()).collect(),
        other => panic!("{name} should be unsupported: {other:#?}"),
    }
}

fn at(source: &str, needle: &str, nth: usize) -> ByteRange {
    let start = source.match_indices(needle).nth(nth).expect("needle").0;
    ByteRange { start, end: start + needle.len() }
}

fn renamed(source: &str, report: &NameBindingReport, name: &str, new_name: &str) -> String {
    let Resolution::Exact(occurrences) = &callable(report, name).resolution else {
        panic!("not exact");
    };
    let mut text = source.to_string();
    let mut ranges: Vec<ByteRange> = occurrences.iter().map(|o| o.name_range).collect();
    ranges.sort();
    for r in ranges.iter().rev() {
        assert_eq!(&source[r.start..r.end], name);
        text.replace_range(r.start..r.end, new_name);
    }
    text
}

fn accepted(source: &str) -> bool {
    analyze_source(SourceAnalysisRequest {
        source_text: source.to_string(),
        path: PATH.to_string(),
        analysis_generation: 0,
        source_profile_identity: "v1-core-v25".to_string(),
    })
    .outcome
        == "ACCEPTED"
}

fn output(source: &str) -> serde_json::Value {
    run_summary_from_supported_source_profile(source, PATH, Some(1), Some("v1-core-v25"))
        .unwrap_or_else(|error| panic!("{source}\n{error}"))["output_log_texts"]
        .clone()
}

/// The rename edits keep the program accepted and its output unchanged.
fn assert_rename_preserves(source: &str, name: &str, new_name: &str) {
    let report = bind_document(source, PATH);
    let after = renamed(source, &report, name, new_name);
    assert!(accepted(&after), "{after}");
    assert_eq!(output(&after), output(source), "{after}");
    let again = bind_document(&after, PATH);
    assert_eq!(exact(&again, new_name).len(), exact(&report, name).len());
}

const E1: &str = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n결과 := 21을 두배하기. 결과 보여주기.\n";
const E2: &str = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n(값:수~을) 세배:수:셈씨 := { 두배:수 := 값 * 3. 세배 <- 두배. }.\n결과 := 7을 세배하기. 결과 보여주기.\n";
const E3: &str = "(두배:수~을) 세배:수:셈씨 := { 세배 <- 두배 * 3. }.\n(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n결과 := 7을 세배하기. 결과 보여주기.\n";

#[test]
fn r9_call_tail_and_result_place_are_one_exact_set() {
    let report = bind_document(E1, PATH);
    assert_eq!(report.outcome, BindingOutcome::Analyzed);
    assert_eq!(
        exact(&report, "두배"),
        vec![
            (OccurrenceRole::Declaration, at(E1, "두배", 0)),
            (OccurrenceRole::ResultWrite, at(E1, "두배", 1)),
            (OccurrenceRole::CallStem, at(E1, "두배", 2)),
        ]
    );
    // The call stem covers only `두배` of the token `두배하기`.
    assert_eq!(&E1[at(E1, "두배", 2).start..at(E1, "두배하기", 0).end], "두배하기");
    assert_eq!(output(E1), serde_json::json!(["42"]));
    assert_rename_preserves(E1, "두배", "곱절");
    // Parameters are listed, not resolved, in A1.
    let parameter = report
        .definitions
        .iter()
        .find(|d| d.kind == DefinitionKind::Parameter && d.name == "값")
        .unwrap();
    assert!(matches!(&parameter.resolution, Resolution::Unsupported(s) if s[0].reason == UnsupportedReason::OutOfA1Scope));
    // Position lookup returns the same exact binding.
    let (definition, occurrence) = report.occurrence_at(at(E1, "두배", 2).start + 1).unwrap();
    assert_eq!((definition.name.as_str(), occurrence.role), ("두배", OccurrenceRole::CallStem));
}

#[test]
fn r10_same_spelling_local_is_unsupported_and_others_stay_exact() {
    let report = bind_document(E2, PATH);
    assert!(reasons(&report, "두배").contains(&UnsupportedReason::SameSpellingLocalBinding));
    assert_eq!(
        exact(&report, "세배").iter().map(|(role, _)| *role).collect::<Vec<_>>(),
        vec![OccurrenceRole::Declaration, OccurrenceRole::ResultWrite, OccurrenceRole::CallStem]
    );
    assert_eq!(output(E2), serde_json::json!(["21"]));
    assert_rename_preserves(E2, "세배", "세곱");
}

#[test]
fn r11_same_spelling_parameter_is_unsupported() {
    let report = bind_document(E3, PATH);
    assert!(reasons(&report, "두배").contains(&UnsupportedReason::SameSpellingParameter));
    assert_rename_preserves(E3, "세배", "세곱");
}

#[test]
fn r1_r2_shadowing_cases_are_unsupported() {
    let local = "합:셈씨 := { 1 돌려줘. }\n둘:셈씨 := { 합:수 := 2. 합 돌려줘. }\n";
    assert!(reasons(&bind_document(local, PATH), "합").contains(&UnsupportedReason::SameSpellingLocalBinding));
    let parameter = "합:셈씨 := { 1 돌려줘. }\n(합:수) 둘:셈씨 := { 합 돌려줘. }\n";
    assert!(reasons(&bind_document(parameter, PATH), "합").contains(&UnsupportedReason::SameSpellingParameter));
}

#[test]
fn r3_r4_body_calls_and_strings() {
    for source in [
        "첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { 합:수 := 2. 첫함수하기. 합 돌려줘. }\n",
        "첫함수:셈씨 := { 1 돌려줘. }\n둘함수:셈씨 := { \"첫함수\" 보여주기. 첫함수하기. }\n",
    ] {
        let report = bind_document(source, PATH);
        let definition = callable(&report, "첫함수");
        match &definition.resolution {
            // Exact only if every use, including the body call, is accounted.
            Resolution::Exact(occurrences) => {
                assert!(occurrences.iter().any(|o| o.role == OccurrenceRole::CallStem), "{source}");
                let string = at(source, "\"첫함수\"", 0);
                assert!(occurrences.iter().all(|o| o.name_range.end <= string.start || o.name_range.start >= string.end));
            }
            // Otherwise the reason must say the body call was not resolved.
            Resolution::Unsupported(sites) => assert!(
                sites.iter().any(|s| matches!(
                    s.reason,
                    UnsupportedReason::CallResolutionMismatch
                        | UnsupportedReason::UnclassifiedOccurrence
                        | UnsupportedReason::CallTailParsedAsVariable
                )),
                "{source}: {sites:#?}"
            ),
        }
    }
}

#[test]
fn r5_r6_r7_scope_is_one_document() {
    let a = "첫함수:셈씨 := { 1 돌려줘. }\n";
    assert_eq!(exact(&bind_document(a, PATH), "첫함수").len(), 1);
    // A document that only calls a callable defined elsewhere binds nothing to it.
    let b = "둘함수:셈씨 := { 첫함수하기. }\n";
    let report = bind_document(b, PATH);
    assert!(report.definitions.iter().all(|d| d.name != "첫함수"));
}

#[test]
fn refused_cases_name_their_reason() {
    // Reading the result place inside the body is not resolved in A1.
    let read = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값. 두배 <- 두배 * 2. }.\n결과 := 3을 두배하기. 결과 보여주기.\n";
    if accepted(read) {
        assert!(reasons(&bind_document(read, PATH), "두배").contains(&UnsupportedReason::ResultPlaceRead));
    }
    // Text the shared check rejects is not analyzed at all.
    let report = bind_document("값 <- (", PATH);
    assert_eq!(report.outcome, BindingOutcome::SourceNotAccepted);
    assert!(report.definitions.is_empty());
    // A different name that merely starts with the callable name is left alone.
    let prefix = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n두배수 := 5.\n두배수 보여주기.\n결과 := 4를 두배하기. 결과 보여주기.\n";
    if accepted(prefix) {
        let occurrences = exact(&bind_document(prefix, PATH), "두배");
        assert_eq!(occurrences.len(), 3);
        assert!(occurrences.iter().all(|(_, r)| !prefix[r.start..].starts_with("두배수")));
    }
}

#[test]
fn identity_is_bound_to_the_source_revision() {
    let first = bind_document(E1, PATH);
    let second = bind_document(&E1.replace("21을", "22를"), PATH);
    assert_ne!(first.source_revision_identity, second.source_revision_identity);
    assert_eq!(first, bind_document(E1, PATH), "deterministic");
}
