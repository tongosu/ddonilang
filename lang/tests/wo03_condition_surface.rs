use ddonirang_lang::{parse, parse_v25, parse_v25_and_normalize, NormalizationLevel, Stmt};

#[test]
fn current_if_preserves_korean_names_before_comparisons() {
    for operator in [">", ">=", "<", "<=", "==", "!="] {
        let source = format!("변길이 <- 2. 최대변길이 <- 1. {{ 변길이 {operator} 최대변길이 }}이면 {{ 변길이 <- 1. }}.");
        let canonical = parse_v25_and_normalize(&source, "비교.ddn", NormalizationLevel::N1).unwrap();
        assert_eq!(parse_v25_and_normalize(&canonical, "비교.ddn", NormalizationLevel::N1).unwrap(), canonical);
    }
}

#[test]
fn current_if_accepts_root_nested_and_roundtrips() {
    for source in [
        "{ 참 }이면 { 1 보여주기. }.",
        "{ 거짓 }이면 { 1 보여주기. } 아니면 { 2 보여주기. }.",
        "{ 참 }이면 { { 거짓 }이면 { 1 보여주기. } 아니면 { 2 보여주기. }. }.",
        "{ 거짓 }이면 {} 아니면 { { 참 }이면 { 3 보여주기. }. }.",
        "시험:움직씨 := { { 참 }이면 { 1 보여주기. }. }.",
    ] {
        let canonical = parse_v25_and_normalize(source, "조건.ddn", NormalizationLevel::N1).unwrap();
        let again = parse_v25_and_normalize(&canonical, "조건.ddn", NormalizationLevel::N1).unwrap();
        assert_eq!(canonical, again, "{source}");
        assert!(!canonical.contains("만약"));
        assert!(canonical.contains("이면"));
    }
    assert!(matches!(parse_v25("{ 참 }이면 {}.", "조건.ddn").unwrap().root_stmts[0], Stmt::If { .. }));
}

#[test]
fn current_if_rejects_legacy_and_direct_chain() {
    for source in [
        "만약 참 이라면 { 1 보여주기. }.",
        "{ 참 }인것 일때 { 1 보여주기. }.",
        "{ 참 }인것 맞으면 { 1 보여주기. }.",
        "{ 참 }이면: { 1 보여주기. }.",
        "{ 참 }이면 {} 아니라면 {}.",
        "{ 참 }이면 {}. 아니면 {}.",
        "{ 참 }이면 {} 아니면 { 거짓 }이면 {}.",
    ] {
        assert!(parse_v25(source, "옛조건.ddn").is_err(), "{source}");
    }
}

#[test]
fn explicit_legacy_epoch_keeps_legacy_condition() {
    parse("시험:움직씨 = { 만약 참 이라면 { 1 보여주기. }. }", "옛조건.ddn").unwrap();
    parse("시험:움직씨 = { { 참 }인것 일때 { 1 보여주기. }. }", "옛조건.ddn").unwrap();
}

#[test]
fn alpha_distribution_sources_parse_under_explicit_v25() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    for path in [
        "solutions/seamgrim_ui_mvp/lessons/rep_phys_projectile_xy_v1/lesson.ddn",
        "solutions/seamgrim_ui_mvp/lessons/rep_phys_pendulum_wave1_v1/lesson.ddn",
        "solutions/seamgrim_ui_mvp/lessons/rep_grid_game_state_drop_v1/lesson.ddn",
        "solutions/seamgrim_ui_mvp/alpha1/01_요금_계산기.ddn",
        "solutions/seamgrim_ui_mvp/alpha1/02_조건과_반복.ddn",
        "solutions/seamgrim_ui_mvp/alpha1/03_자동_함수표.ddn",
        "solutions/seamgrim_ui_mvp/alpha1/04_시간단계_시뮬레이션.ddn",
    ] {
        let source = std::fs::read_to_string(root.join(path)).unwrap();
        parse_v25(&source, path).unwrap_or_else(|error| panic!("{path}: {error}"));
    }
}
