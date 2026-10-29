use ddonirang_tool::name_binding::final_bindings::{
    bind_document, BindingState, DefinitionKind, FinalBindingOutcome,
};
const SOURCE:&str="(값:수~을) 두배:수:셈씨 := { 임시:수 := 값 + 1. 두배 <- 임시 * 2. }. 결과 := 21을 두배하기. 결과 보여주기.";
#[test]
fn editor_reads_common_parameter_and_local_facts_of_the_current_document() {
    let report = bind_document(SOURCE, "문서.ddn");
    assert_eq!(report.outcome(), FinalBindingOutcome::Available);
    let facts = report.facts().unwrap();
    for (needle, kind) in [
        ("값 +", DefinitionKind::Parameter),
        ("임시 *", DefinitionKind::Local),
        ("두배 <-", DefinitionKind::ResultPlace),
    ] {
        let offset = SOURCE.find(needle).unwrap();
        let u = report.use_in_source(SOURCE, "문서.ddn", offset).unwrap();
        let BindingState::Final(key) = u.state() else {
            panic!("{u:?}")
        };
        assert_eq!(facts.definition(*key).unwrap().kind(), kind);
    }
    assert!(report
        .use_in_source(
            &SOURCE.replace("21", "22"),
            "문서.ddn",
            SOURCE.find("임시 *").unwrap()
        )
        .is_none());
    assert!(report.use_in_source(SOURCE, "다른문서.ddn", 0).is_none());
}
#[test]
fn rejected_shadow_call_never_exports_a_successful_prefix() {
    let source="(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }. (값:수~을) 세배:수:셈씨 := { 두배:수 := 4. 세배 <- 1을 두배하기. }.";
    let report = bind_document(source, "shadow.ddn");
    assert_eq!(report.outcome(), FinalBindingOutcome::SourceRejected);
    assert!(report.facts().is_none());
}
#[test]
fn callable_candidates_do_not_grant_rename_and_a1_api_is_unchanged() {
    let report = bind_document(SOURCE, "문서.ddn");
    let call = report
        .use_in_source(SOURCE, "문서.ddn", SOURCE.find("두배하기").unwrap())
        .unwrap();
    assert!(matches!(call.state(),BindingState::Candidates(ids) if ids.len()==1));
    let old = ddonirang_tool::name_binding::bind_document(SOURCE, "문서.ddn");
    assert_eq!(
        old.outcome,
        ddonirang_tool::name_binding::BindingOutcome::Analyzed
    );
    assert!(old.definitions.iter().any(|d| d.name == "두배"
        && matches!(
            d.resolution,
            ddonirang_tool::name_binding::Resolution::Exact(_)
        )));
}
#[test]
fn common_name_ranges_match_the_existing_a2_1_supply() {
    use ddonirang_tool::name_binding::token_ranges::{analyze_name_tokens, DeclarationNameRange};
    let report = bind_document(SOURCE, "문서.ddn");
    let facts = report.facts().unwrap();
    let tokens = analyze_name_tokens(SOURCE, "문서.ddn");
    for def in facts
        .definitions()
        .iter()
        .filter(|d| matches!(d.kind(), DefinitionKind::Parameter | DefinitionKind::Local))
    {
        let expected = tokens
            .declarations()
            .iter()
            .find(|d| d.name == def.name() && d.syntax_range.start == def.syntax_span().start)
            .unwrap();
        let DeclarationNameRange::Located(range) = expected.name_range else {
            panic!("missing existing name token")
        };
        let actual = def.name_span().unwrap();
        assert_eq!((actual.start, actual.end), (range.start, range.end));
    }
}
#[test]
fn complete_local_census_supports_real_recheck_and_execution_without_touching_other_symbols() {
    use ddonirang_tool::name_binding::token_ranges::analyze_name_tokens;
    use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;
    let mut source = SOURCE.to_string();
    let run = |source: &str| {
        run_summary_from_supported_source_profile(source, "文書.ddn", Some(1), Some("v1-core-v25"))
            .unwrap()["output_log_texts"]
            .clone()
    };
    assert_eq!(run(&source), serde_json::json!(["44"]));
    for (old, new, needle) in [("값", "인수값", "값 +"), ("임시", "중간값", "임시 *")] {
        let report = bind_document(&source, "文書.ddn");
        let tokens = analyze_name_tokens(&source, "文書.ddn");
        let BindingState::Final(key) = report
            .use_in_source(&source, "文書.ddn", source.find(needle).unwrap())
            .unwrap()
            .state()
        else {
            panic!("not final")
        };
        let mut edits = report
            .exact_local_occurrences(&source, "文書.ddn", *key, &tokens)
            .unwrap();
        assert_eq!(edits.len(), 2);
        edits.sort_by_key(|s| std::cmp::Reverse(s.start));
        for span in edits {
            assert_eq!(&source[span.start..span.end], old);
            source.replace_range(span.start..span.end, new);
        }
        assert!(report.use_in_source(&source, "文書.ddn", 0).is_none());
        assert_eq!(run(&source), serde_json::json!(["44"]));
        assert_eq!(
            bind_document(&source, "文書.ddn").outcome(),
            FinalBindingOutcome::Available
        );
    }
    assert!(source.contains("두배하기"));
}
#[test]
fn unresolved_named_pin_is_refused_instead_of_issuing_partial_parameter_edits() {
    use ddonirang_tool::name_binding::final_bindings::LocalCoverageIssue;
    use ddonirang_tool::name_binding::token_ranges::analyze_name_tokens;
    let source = SOURCE.replace("21을 두배하기", "(값=21) 두배하기");
    let report = bind_document(&source, "pin.ddn");
    assert_eq!(report.outcome(), FinalBindingOutcome::Available);
    let BindingState::Final(key) = report
        .use_in_source(&source, "pin.ddn", source.find("값 +").unwrap())
        .unwrap()
        .state()
    else {
        panic!("not final")
    };
    assert_eq!(
        report
            .exact_local_occurrences(
                &source,
                "pin.ddn",
                *key,
                &analyze_name_tokens(&source, "pin.ddn")
            )
            .unwrap_err(),
        LocalCoverageIssue::UnresolvedOccurrence
    );
}
#[test]
fn original_r10_r11_raw_sources_keep_other_same_spelling_bindings_and_actual_output() {
    use ddonirang_tool::name_binding::token_ranges::analyze_name_tokens;
    use ddonirang_tool::runtime_surface::run_summary_from_supported_source_profile;
    for (raw, kind, new_name) in [
        (
            include_bytes!("fixtures/v2_name_binding/e2.ddn").as_slice(),
            DefinitionKind::Local,
            "지역값",
        ),
        (
            include_bytes!("fixtures/v2_name_binding/e3.ddn").as_slice(),
            DefinitionKind::Parameter,
            "인수값",
        ),
    ] {
        let source = std::str::from_utf8(raw).unwrap();
        let report = bind_document(source, "원문.ddn");
        assert_eq!(report.outcome(), FinalBindingOutcome::Available);
        let facts = report.facts().unwrap();
        let target = facts
            .definitions()
            .iter()
            .find(|d| d.kind() == kind && d.name() == "두배")
            .unwrap();
        let tokens = analyze_name_tokens(source, "원문.ddn");
        let mut spans = report
            .exact_local_occurrences(source, "원문.ddn", target.key(), &tokens)
            .unwrap();
        assert_eq!(spans.len(), 2);
        let before = run_summary_from_supported_source_profile(
            source,
            "원문.ddn",
            Some(1),
            Some("v1-core-v25"),
        )
        .unwrap();
        assert_eq!(before["output_log_texts"], serde_json::json!(["21"]));
        spans.sort_by_key(|s| std::cmp::Reverse(s.start));
        let mut after = source.to_string();
        for span in spans {
            assert_eq!(&after[span.start..span.end], "두배");
            after.replace_range(span.start..span.end, new_name);
        }
        let checked = bind_document(&after, "원문.ddn");
        assert_eq!(checked.outcome(), FinalBindingOutcome::Available);
        let result = run_summary_from_supported_source_profile(
            &after,
            "원문.ddn",
            Some(1),
            Some("v1-core-v25"),
        )
        .unwrap();
        assert_eq!(result["output_log_texts"], before["output_log_texts"]);
        let original = ddonirang_tool::name_binding::bind_document(source, "원문.ddn");
        let changed = ddonirang_tool::name_binding::bind_document(&after, "원문.ddn");
        let declaration = |r: ddonirang_tool::name_binding::NameBindingReport| {
            r.definitions
                .into_iter()
                .find(|d| {
                    d.kind == ddonirang_tool::name_binding::DefinitionKind::TopLevelCallable
                        && d.name == "두배"
                })
                .unwrap()
        };
        assert_eq!(
            declaration(original).name,
            declaration(changed).name,
            "global callable must not be renamed"
        );
        assert!(after.contains("두배 <- 값 * 2"));
    }
}

#[cfg(feature = "v2-shared-modules")]
fn actual_helper_editor_source() -> String {
    ddonirang_tool::authored_project::operate(
        "source_project", include_str!("fixtures/v2_helper_source.json"),
    ).unwrap()
}

#[test]
#[cfg(feature = "v2-shared-modules")]
fn project_owner_supplies_imported_final_and_exact_original_name_token() {
    use ddonirang_tool::name_binding::final_bindings::{bind_project, ProjectBindingIssue};
    let raw = actual_helper_editor_source();
    let source: ddonirang_tool::local_project::VirtualProjectSource = serde_json::from_str(&raw).unwrap();
    let app = &source.files.iter().find(|f| f.path == "app/main.ddn").unwrap().utf8_text;
    let report = bind_project(&raw, "file:///project-one/", 8).unwrap();
    let alias = app.find("힘.속도").unwrap();
    let offset = alias + "힘.".len();
    let target = report.binding_at(&raw, "file:///project-one/", 8, "app/main.ddn", offset).unwrap();
    assert_eq!(target.usage().span().end - target.usage().span().start, "속도".len());
    assert_eq!(target.usage().span().start, offset);
    assert_eq!(target.definition().path(), "vendor/운동/ddn/lib/운동.ddn");
    let library = &source.files.iter().find(|f| f.path == target.definition().path()).unwrap().utf8_text;
    assert_eq!(&library[target.definition().span().start..target.definition().span().end], "속도");
    assert!(library[target.definition().span().start..].starts_with("속도:수"));
    assert!(matches!(report.binding_at(&raw, "file:///project-one/", 8, "app/main.ddn", alias),
        Err(ProjectBindingIssue::NonNamePosition)));
    // The exported alias surface has its own owner: a successful jump must
    // not silently issue a partial project rename or write into a dependency.
    assert_eq!(report.exact_occurrences(&raw, "file:///project-one/", 8, &target).unwrap_err(),
        ProjectBindingIssue::IncompleteOccurrences);
}

#[test]
#[cfg(feature = "v2-shared-modules")]
fn proven_setup_and_hook_sites_support_complete_project_counter_edits_and_cli_recheck() {
    use ddonirang_tool::name_binding::final_bindings::{bind_project, DefinitionOwnerScope};
    let raw = actual_helper_editor_source();
    let mut project: ddonirang_tool::local_project::VirtualProjectSource = serde_json::from_str(&raw).unwrap();
    let app = &project.files.iter().find(|f| f.path == "app/main.ddn").unwrap().utf8_text;
    let report = bind_project(&raw, "file:///project-one/", 9).unwrap();
    let offset = app.find("횟수 +").unwrap();
    let target = report.binding_at(&raw, "file:///project-one/", 9, "app/main.ddn", offset).unwrap();
    assert_eq!(target.owner_scope(), DefinitionOwnerScope::RelocatedTopLevelSetup);
    let mut sites = report.exact_occurrences(&raw, "file:///project-one/", 9, &target).unwrap();
    assert_eq!(sites.len(), 4);
    assert!(sites.iter().all(|s| s.path() == "app/main.ddn"));
    let run = |raw: &str| -> serde_json::Value {
        let output = ddonirang_tool::local_project::build_project_run_summary_with_supported_profile_json(
            raw, "", false, false).unwrap();
        serde_json::from_str::<serde_json::Value>(&output).unwrap()["runtime_output_log_texts"].clone()
    };
    // Editing source is not a frozen-lock update. Use the existing explicit
    // product lock command, keeping every authored source byte unchanged.
    let lock_for_run = |raw: &str| -> String {
        let mut project: ddonirang_tool::local_project::VirtualProjectSource = serde_json::from_str(raw).unwrap();
        let mut unlocked = project.clone();
        unlocked.files.retain(|f| f.path != "ddn.lock");
        let lock = ddonirang_tool::local_project::build_project_gaji_lock_json(
            &serde_json::to_string(&unlocked).unwrap()).unwrap();
        project.files.iter_mut().find(|f| f.path == "ddn.lock").unwrap().utf8_text = lock;
        serde_json::to_string(&project).unwrap()
    };
    let before = run(&lock_for_run(&raw));
    sites.sort_by_key(|s| std::cmp::Reverse(s.span().start));
    let app = &mut project.files.iter_mut().find(|f| f.path == "app/main.ddn").unwrap().utf8_text;
    for site in &sites {
        assert_eq!(&app[site.span().start..site.span().end], "횟수");
        app.replace_range(site.span().start..site.span().end, "진행수");
    }
    assert!(app.contains("힘.속도"));
    let at = app.find("진행수 +").unwrap();
    let changed = serde_json::to_string(&project).unwrap();
    assert_eq!(run(&lock_for_run(&changed)), before);
    assert_eq!(before, serde_json::json!(["7", "1"]));
    let next = bind_project(&changed, "file:///project-one/", 10).unwrap();
    assert_eq!(next.binding_at(&changed, "file:///project-one/", 10, "app/main.ddn", at)
        .unwrap().owner_scope(), DefinitionOwnerScope::RelocatedTopLevelSetup);
    assert!(report.binding_at(&changed, "file:///project-one/", 9, "app/main.ddn", offset).is_err());
}

#[test]
#[cfg(feature = "v2-shared-modules")]
fn project_facts_refuse_stale_foreign_context_unregistered_alias_and_partial_utf8() {
    use ddonirang_tool::name_binding::final_bindings::{bind_project, ProjectBindingIssue};
    let raw = actual_helper_editor_source();
    let project: ddonirang_tool::local_project::VirtualProjectSource = serde_json::from_str(&raw).unwrap();
    let app = &project.files.iter().find(|f| f.path == "app/main.ddn").unwrap().utf8_text;
    let offset = app.find("횟수 +").unwrap();
    let report = bind_project(&raw, "file:///project-one/", 11).unwrap();
    let target = report.binding_at(&raw, "file:///project-one/", 11, "app/main.ddn", offset).unwrap();
    for (uri, generation) in [("file:///project-two/", 11), ("file:///project-one/", 12)] {
        assert_eq!(report.binding_at(&raw, uri, generation, "app/main.ddn", offset).unwrap_err(),
            ProjectBindingIssue::StaleProject);
        assert_eq!(report.exact_occurrences(&raw, uri, generation, &target).unwrap_err(),
            ProjectBindingIssue::StaleProject);
    }
    assert!(report.binding_at(&raw, "file:///project-one/", 11, "app/main.ddn", offset+1).is_err());
    let mut wrong = project.clone();
    let text = &mut wrong.files.iter_mut().find(|f| f.path == "app/main.ddn").unwrap().utf8_text;
    *text = text.replace("힘.속도", "다른힘.속도");
    let bad_offset = text.find("다른힘.속도").unwrap()+"다른힘.".len();
    let wrong = serde_json::to_string(&wrong).unwrap();
    if let Ok(bad) = bind_project(&wrong, "file:///project-one/", 12) {
        assert!(bad.binding_at(&wrong, "file:///project-one/", 12, "app/main.ddn", bad_offset).is_err());
    }
    // One document's ordinary edit API must not mistake setup-owner facts
    // for an ordinary local binding just because their coarse kind is Local.
    let single = "채비 { 횟수:바른수 := 0. }. 앱:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. }. }.";
    let one = bind_document(single, "single.ddn");
    assert_eq!(one.outcome(), FinalBindingOutcome::Available);
    let BindingState::Final(key) = one.use_in_source(single, "single.ddn", single.find("횟수 +").unwrap())
        .unwrap().state() else { panic!("actual setup owner required") };
    assert_eq!(one.exact_local_occurrences(single, "single.ddn", *key,
        &ddonirang_tool::name_binding::token_ranges::analyze_name_tokens(single,"single.ddn")).unwrap_err(),
        ddonirang_tool::name_binding::final_bindings::LocalCoverageIssue::NotLocal);
}

#[test]
#[cfg(feature = "v2-shared-modules")]
fn project_setup_owner_does_not_make_an_uncovered_hook_scope_or_partial_edits_final() {
    use ddonirang_tool::name_binding::final_bindings::{bind_project, ProjectBindingIssue};
    let raw = actual_helper_editor_source();
    let mut project: ddonirang_tool::local_project::VirtualProjectSource = serde_json::from_str(&raw).unwrap();
    let app = &mut project.files.iter_mut().find(|f| f.path == "app/main.ddn").unwrap().utf8_text;
    *app = app.replace("(매마디)마다 {", "(매마디)마다 { 중간값:바른수 := 10.");
    let declaration = app.find("횟수:바른수").unwrap();
    let use_offset = app.find("횟수 +").unwrap();
    let raw = serde_json::to_string(&project).unwrap();
    let report = bind_project(&raw, "file:///project-one/", 12).unwrap();
    assert_eq!(report.binding_at(&raw, "file:///project-one/", 12, "app/main.ddn", use_offset)
        .unwrap_err(), ProjectBindingIssue::NotFinal);
    let target = report.binding_at(&raw, "file:///project-one/", 12, "app/main.ddn", declaration).unwrap();
    assert_eq!(report.exact_occurrences(&raw, "file:///project-one/", 12, &target).unwrap_err(),
        ProjectBindingIssue::IncompleteOccurrences);
}


#[test]
#[cfg(not(feature = "v2-shared-modules"))]
fn project_feature_off_refuses_real_helper_source_projection() {
    let error = ddonirang_tool::authored_project::operate(
        "source_project", include_str!("fixtures/v2_helper_source.json")).unwrap_err();
    assert!(error.contains("E_PROJECT_UNDECLARED_SOURCE"), "{error}");
}
