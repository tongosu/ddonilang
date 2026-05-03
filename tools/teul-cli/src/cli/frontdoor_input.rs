use crate::file_meta::split_file_meta;

#[derive(Debug, Clone)]
pub struct PreparedFrontdoorCanonInput {
    pub prepared: String,
}

#[allow(dead_code)]
pub fn find_legacy_header(source: &str) -> Option<(usize, &'static str)> {
    ddonirang_lang::find_legacy_header(source)
}

pub fn validate_no_legacy_header(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_header(source)
}

pub fn validate_no_legacy_boim_surface(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_boim_surface(source)
}

pub fn validate_no_legacy_root_hide_directive(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_root_hide_directive(source)
}

pub fn validate_no_legacy_root_surface(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_root_surface(source)
}

pub fn validate_no_legacy_range_comment(source: &str) -> Result<(), String> {
    ddonirang_lang::validate_no_legacy_range_comment(source)
}

pub fn validate_no_legacy_frontdoor_surface(source: &str) -> Result<(), String> {
    validate_no_legacy_header(source)?;
    validate_no_legacy_boim_surface(source)?;
    validate_no_legacy_root_hide_directive(source)?;
    validate_no_legacy_root_surface(source)?;
    validate_no_legacy_range_comment(source)?;
    Ok(())
}

pub fn prepare_frontdoor_runtime_source(source: &str) -> String {
    ddonirang_lang::preprocess_frontdoor_source(source)
}

pub fn prepare_frontdoor_canon_input(source: &str) -> PreparedFrontdoorCanonInput {
    let meta_parse = split_file_meta(source);
    let stripped = meta_parse.stripped;
    // canon frontdoor는 정본 출력 의미를 보존해야 하므로, seed 자동 래핑을 포함한
    // runtime 전용 preprocess_bridge 경로를 사용하지 않는다.
    let prepared = ddonirang_lang::preprocess_frontdoor_source(&stripped);
    PreparedFrontdoorCanonInput { prepared }
}

#[cfg(test)]
mod tests {
    use super::{find_legacy_header, validate_no_legacy_frontdoor_surface};

    #[test]
    fn legacy_header_is_detected_via_lang_single_source() {
        let source = "#이름: 금지\n(매마디)마다 { n <- 1. }.";
        let found = find_legacy_header(source).expect("must detect");
        assert_eq!(found.0, 1);
        assert_eq!(found.1, "이름");
    }

    #[test]
    fn boim_is_accepted_via_frontdoor_guard() {
        let source = "보임 { x: 1. }.";
        validate_no_legacy_frontdoor_surface(source).expect("must pass");
    }

    #[test]
    fn legacy_root_prefix_is_rejected_via_frontdoor_guard() {
        let source = "살림.x <- 1.";
        let err = validate_no_legacy_frontdoor_surface(source).expect_err("must reject");
        assert!(err.contains("E_SALIM_REMOVED"));
    }
}
