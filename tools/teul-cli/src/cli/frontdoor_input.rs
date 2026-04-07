use crate::file_meta::split_file_meta;

#[derive(Debug, Clone)]
pub struct PreparedFrontdoorCanonInput {
    pub prepared: String,
}

const LEGACY_HEADER_KEYS: &[&str] = &[
    "이름",
    "설명",
    "말씨",
    "사투리",
    "그래프",
    "필수보기",
    "required_views",
    "필수보개",
    "조종",
    "조절",
    "control",
];

pub fn find_legacy_header(source: &str) -> Option<(usize, &'static str)> {
    for (line_no, raw) in source.lines().enumerate() {
        let line = raw.trim_start();
        if !line.starts_with('#') {
            continue;
        }
        let rest = line[1..].trim_start();
        for key in LEGACY_HEADER_KEYS {
            if header_key_matches(rest, key) {
                return Some((line_no + 1, *key));
            }
        }
    }
    None
}

pub fn validate_no_legacy_header(source: &str) -> Result<(), String> {
    if let Some((line, key)) = find_legacy_header(source) {
        return Err(format!(
            "E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN line={line} key={key} use=설정{{}}/매김{{}}/설정.보개"
        ));
    }
    Ok(())
}

fn header_key_matches(rest: &str, key: &str) -> bool {
    if key.is_ascii() {
        let lower = rest.to_ascii_lowercase();
        let key_lower = key.to_ascii_lowercase();
        if !lower.starts_with(&key_lower) {
            return false;
        }
        return lower[key_lower.len()..].trim_start().starts_with(':');
    }
    if !rest.starts_with(key) {
        return false;
    }
    rest[key.len()..].trim_start().starts_with(':')
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
    PreparedFrontdoorCanonInput {
        prepared,
    }
}
