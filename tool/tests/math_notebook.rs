use ddonirang_tool::math_notebook::observe_uniform_motion;
const SOURCE: &str = include_str!("fixtures/v3_uniform_motion/등속.ddn");
#[test]
fn source_motion_exact_dimensions_and_time() {
    let text = observe_uniform_motion(SOURCE, "등속.ddn").unwrap();
    let report: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        report["rows"][2]["position"]["coefficient"]["numerator"],
        "7"
    );
    assert_eq!(report["solution"]["time"]["coefficient"]["numerator"], "4");
    assert_eq!(report["solution"]["substitution"]["equal"], true);
}

#[test]
fn motion_refusals_and_solver_outcomes() {
    for (source, kind) in [
        (
            SOURCE
                .replace("3 / 2", "0")
                .replace("목표:나눔수 := 7", "목표:나눔수 := 1"),
            "non_unique",
        ),
        (SOURCE.replace("3 / 2", "0"), "no_solution"),
        (
            SOURCE.replace("목표:나눔수 := 7", "목표:나눔수 := -2"),
            "outside_domain",
        ),
    ] {
        let r: serde_json::Value =
            serde_json::from_str(&observe_uniform_motion(&source, "등속.ddn").unwrap()).unwrap();
        assert_eq!(r["solution"]["kind"], kind);
        if kind == "outside_domain" {
            assert_eq!(r["solution"]["substitution"]["equal"], true);
            assert!(r["solution"].get("time").is_none());
        } else {
            assert!(r["solution"].get("substitution").is_none());
        }
    }
    for (source, code) in [
        (
            SOURCE.replace("길이기준 <- 1@m", "길이기준 <- 1@s"),
            "E_NOTEBOOK_DIMENSION",
        ),
        (
            SOURCE.replace("속도기준 <- 1@m/s", "속도기준 <- 1@m"),
            "E_NOTEBOOK_DIMENSION",
        ),
        (
            SOURCE.replace("길이기준 <- 1@m", "길이기준 <- 2@m"),
            "E_NOTEBOOK_UNIT_BASIS",
        ),
        (
            SOURCE.replace("영:나눔수 := 0", "영:나눔수 := -1"),
            "E_NOTEBOOK_TIME_DOMAIN",
        ),
    ] {
        assert!(observe_uniform_motion(&source, "등속.ddn")
            .unwrap_err()
            .contains(code));
    }
}

#[test]
fn motion_fraction_and_changed_source_are_exact() {
    let raw = SOURCE
        .replace("3 / 2", "3")
        .replace("목표:나눔수 := 7", "목표:나눔수 := 2");
    let r: serde_json::Value =
        serde_json::from_str(&observe_uniform_motion(&raw, "등속.ddn").unwrap()).unwrap();
    assert_eq!(
        r["solution"]["time"]["coefficient"],
        serde_json::json!({"numerator":"1","denominator":"3"})
    );
    assert_eq!(r["solution"]["substitution"]["equal"], true);
    let changed = SOURCE.replace("3 / 2", "2");
    let r: serde_json::Value =
        serde_json::from_str(&observe_uniform_motion(&changed, "등속.ddn").unwrap()).unwrap();
    assert_eq!(r["rows"][2]["position"]["coefficient"]["numerator"], "9");
}

#[test]
fn whole_authored_source_and_rejection_boundaries() {
    let request = serde_json::json!({"source_profile":"v1-core-v25","module_name":"",
        "source_snapshot":{"root_path":"root.ddn","entry_path":"main.ddn","files":[
            {"path":"root.ddn","text":include_str!("fixtures/v3_uniform_motion/root.ddn")},
            {"path":"main.ddn","text":include_str!("fixtures/v3_uniform_motion/main.ddn")}]}});
    let artifact =
        ddonirang_tool::authored_project::operate("create", &request.to_string()).unwrap();
    let text = ddonirang_tool::math_notebook::observe_uniform_motion_project(&artifact).unwrap();
    let report: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(report["solution"]["time"]["coefficient"]["numerator"], "4");
    assert!(report["authored_source"]["content_sha256"].is_string());
    // Changing saved source without common artifact update cannot reuse custody.
    let tampered = artifact.replace("3 / 2", "5 / 2");
    assert_ne!(tampered, artifact);
    assert!(ddonirang_tool::math_notebook::observe_uniform_motion_project(&tampered).is_err());
    for raw in [
        SOURCE.replace("3 / 2", "3 / 0"),
        
    ] {
        assert!(observe_uniform_motion(&raw, "거부.ddn").is_err());
    }
    let fresh = observe_uniform_motion(&SOURCE.replace("3 / 2", "2"), "등속.ddn").unwrap();
    let fresh: serde_json::Value = serde_json::from_str(&fresh).unwrap();
    let previous: serde_json::Value =
        serde_json::from_str(&observe_uniform_motion(SOURCE, "등속.ddn").unwrap()).unwrap();
    assert_ne!(fresh["source_sha256"], previous["source_sha256"]);
    assert_ne!(fresh["rows"], previous["rows"]);
}

#[test]
fn actual_unit_overflow_is_not_a_parse_failure() {
    let raw=include_str!("fixtures/v3_uniform_motion/단위넘침.ddn");
    let error=observe_uniform_motion(&raw,"넘침.ddn").unwrap_err();
    assert!(error.contains("Fixed64 계산 범위를 벗어났습니다"),"{error}");
}

#[test]
fn negative_algebraic_time_is_checked_but_not_domain_admitted() {
    let raw = SOURCE.replace("처음:나눔수 := 1", "처음:나눔수 := 5")
        .replace("3 / 2", "2").replace("목표:나눔수 := 7", "목표:나눔수 := 4");
    let r: serde_json::Value = serde_json::from_str(&observe_uniform_motion(&raw, "음수검산.ddn").unwrap()).unwrap();
    let s = &r["solution"];
    assert_eq!(s["kind"], "outside_domain");
    assert_eq!(s["algebraic_time"]["coefficient"], serde_json::json!({"numerator":"-1","denominator":"2"}));
    assert_eq!(s["substitution"]["left"]["coefficient"], serde_json::json!({"numerator":"4","denominator":"1"}));
    assert_eq!(s["substitution"]["left"], s["substitution"]["right"]);
    assert_eq!(s["substitution"]["equal"], true);
    assert_eq!(r["proof"], false);
    assert_eq!(r["time_domain"], "t>=0");
    assert!(s.get("time").is_none());
}
