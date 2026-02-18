use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

const DIALECT_TABLE_TSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../docs/context/notes/dialect/DDONIRANG_dialect_keywords_full_v8_20260215.tsv"
));

const LEGACY_ALIASES: &[(&str, &str)] = &[
    ("반복", "되풀이"),
    ("반환", "되돌림"),
    ("돌려줘", "되돌림"),
    ("보장하고", "다짐하고"),
    ("전제하에", "바탕으로"),
    ("아니라면", "아니면"),
];

const TAG_COLUMNS_EXCLUDED: &[&str] = &["kind", "ko_canon", "ko_alias", "notes", "sym3"];

#[derive(Clone, Debug)]
pub struct DialectConfig {
    #[allow(dead_code)]
    active_tag: Option<String>,
    keyword_map: HashMap<String, String>,
    symbol_map: HashMap<String, String>,
    symbol_tokens: Vec<String>,
}

impl DialectConfig {
    pub fn from_source(source: &str) -> Self {
        let lexicon = DialectLexicon::get();
        let active_tag = detect_active_tag(source, &lexicon.header_tokens, &lexicon.tags);
        let keyword_map = build_active_keyword_map(&lexicon.by_lang, active_tag.as_deref());
        Self {
            active_tag,
            keyword_map,
            symbol_map: lexicon.symbol_map.clone(),
            symbol_tokens: lexicon.symbol_tokens.clone(),
        }
    }

    pub fn canonicalize_keyword<'a>(&'a self, token: &str) -> Option<&'a str> {
        self.keyword_map.get(token).map(|value| value.as_str())
    }

    pub fn canonicalize<'a>(&'a self, token: &str) -> Option<&'a str> {
        self.canonicalize_keyword(token)
    }

    pub fn canonicalize_symbol<'a>(&'a self, token: &str) -> Option<&'a str> {
        self.symbol_map.get(token).map(|value| value.as_str())
    }

    pub fn canonicalize_josa<'a>(&'a self, token: &str) -> Option<&'a str> {
        DialectLexicon::get()
            .josa_role_map
            .get(token)
            .map(|value| value.as_str())
    }

    pub fn symbol_tokens(&self) -> &[String] {
        &self.symbol_tokens
    }

    pub fn sym3_tokens() -> &'static [String] {
        &DialectLexicon::get().symbol_tokens
    }

    pub fn is_inactive_keyword(&self, token: &str) -> bool {
        let lexicon = DialectLexicon::get();
        if !lexicon.all_keywords.contains_key(token) {
            return false;
        }
        !self.keyword_map.contains_key(token)
    }
}

struct DialectLexicon {
    header_tokens: Vec<String>,
    tags: HashSet<String>,
    by_lang: HashMap<String, HashMap<String, String>>,
    all_keywords: HashMap<String, String>,
    josa_role_map: HashMap<String, String>,
    symbol_map: HashMap<String, String>,
    symbol_tokens: Vec<String>,
}

impl DialectLexicon {
    fn get() -> &'static Self {
        static INSTANCE: OnceLock<DialectLexicon> = OnceLock::new();
        INSTANCE.get_or_init(Self::load)
    }

    fn load() -> Self {
        let mut lines = DIALECT_TABLE_TSV.lines();
        let header_line = lines.next().unwrap_or_default().trim_start_matches('\u{feff}');
        let headers: Vec<&str> = header_line.split('\t').collect();
        let mut tags = HashSet::new();
        let mut by_lang: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut all_keywords: HashMap<String, String> = HashMap::new();
        let mut josa_role_map: HashMap<String, String> = HashMap::new();
        let mut symbol_map: HashMap<String, String> = HashMap::new();
        let mut header_tokens: Vec<String> = Vec::new();

        for header in &headers {
            if TAG_COLUMNS_EXCLUDED.contains(header) {
                continue;
            }
            tags.insert((*header).to_string());
        }

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let mut values: Vec<&str> = line.split('\t').collect();
            if values.len() < headers.len() {
                values.resize(headers.len(), "");
            }
            let mut row = HashMap::new();
            for (header, value) in headers.iter().zip(values.iter()) {
                row.insert(*header, (*value).trim());
            }
            let ko_canon = row.get("ko_canon").copied().unwrap_or_default();
            let ko_alias = row.get("ko_alias").copied().unwrap_or_default();
            let sym3_raw = row.get("sym3").copied().unwrap_or_default();
            let kind = row.get("kind").copied().unwrap_or_default();

            if kind == "josa" {
                let role = row.get("notes").copied().unwrap_or_default().trim();
                if !role.is_empty() {
                    for header in &headers {
                        if matches!(*header, "kind" | "notes") {
                            continue;
                        }
                        if let Some(raw) = row.get(*header).copied() {
                            insert_josa_tokens(&mut josa_role_map, raw, role);
                        }
                    }
                }
                continue;
            }

            if ko_canon.starts_with("#말씨") {
                let tokens = collect_header_tokens(&row, &headers);
                for token in tokens {
                    if !header_tokens.contains(&token) {
                        header_tokens.push(token);
                    }
                }
                continue;
            }

            let canon = match normalize_keyword_token(ko_canon) {
                Some(value) => value,
                None => continue,
            };

            let ko_map = by_lang.entry("ko".to_string()).or_default();
            insert_keyword(ko_map, &canon, &canon);
            insert_keyword(&mut all_keywords, &canon, &canon);

            if let Some(alias) = normalize_keyword_token(ko_alias) {
                insert_keyword(ko_map, &alias, &canon);
                insert_keyword(&mut all_keywords, &alias, &canon);
            }

            for tag in tags.iter() {
                if let Some(raw) = row.get(tag.as_str()).copied() {
                    if let Some(token) = normalize_keyword_token(raw) {
                        let map = by_lang.entry(tag.to_string()).or_default();
                        insert_keyword(map, &token, &canon);
                        insert_keyword(&mut all_keywords, &token, &canon);
                    }
                }
            }

            for token in split_symbol_tokens(sym3_raw) {
                insert_symbol(&mut symbol_map, &token, &canon);
            }
        }

        if let Some(ko_map) = by_lang.get_mut("ko") {
            for (alias, canon) in LEGACY_ALIASES {
                insert_keyword(ko_map, alias, canon);
                insert_keyword(&mut all_keywords, alias, canon);
            }
        }

        if header_tokens.is_empty() {
            header_tokens.push("#말씨:".to_string());
            header_tokens.push("#사투리:".to_string());
            header_tokens.push("#dialect:".to_string());
        }

        let mut symbol_tokens: Vec<String> = symbol_map.keys().cloned().collect();
        symbol_tokens.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));

        Self {
            header_tokens,
            tags,
            by_lang,
            all_keywords,
            josa_role_map,
            symbol_map,
            symbol_tokens,
        }
    }
}

fn collect_header_tokens(row: &HashMap<&str, &str>, headers: &[&str]) -> Vec<String> {
    let mut tokens = Vec::new();
    for header in headers {
        if let Some(raw) = row.get(header).copied() {
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }
            if raw.starts_with('#') {
                tokens.push(raw.to_string());
            }
        }
    }
    tokens
}

fn detect_active_tag(source: &str, headers: &[String], tags: &HashSet<String>) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim_start();
        for header in headers {
            if trimmed.starts_with(header) {
                let rest = trimmed[header.len()..].trim();
                if rest.is_empty() {
                    return None;
                }
                let tag = rest
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches(|ch: char| ch == '#' || ch == ':');
                if tag.is_empty() {
                    return None;
                }
                let normalized = normalize_tag(tag);
                return tags.contains(&normalized).then_some(normalized);
            }
        }
    }
    None
}

fn build_active_keyword_map(
    by_lang: &HashMap<String, HashMap<String, String>>,
    active_tag: Option<&str>,
) -> HashMap<String, String> {
    let mut keyword_map = HashMap::new();
    if let Some(ko_map) = by_lang.get("ko") {
        keyword_map.extend(ko_map.clone());
    }
    if let Some(tag) = active_tag {
        if tag != "ko" {
            if let Some(map) = by_lang.get(tag) {
                keyword_map.extend(map.clone());
            }
        }
    }
    keyword_map
}

fn normalize_tag(tag: &str) -> String {
    let mut out = String::new();
    for ch in tag.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_alphabetic() {
            out.push(ch);
        } else {
            break;
        }
    }
    out
}

fn insert_keyword(map: &mut HashMap<String, String>, token: &str, canon: &str) {
    map.entry(token.to_string())
        .or_insert_with(|| canon.to_string());
}

fn insert_josa_tokens(map: &mut HashMap<String, String>, raw: &str, role: &str) {
    for token in split_josa_tokens(raw) {
        if !token.starts_with('~') {
            continue;
        }
        map.entry(token)
            .or_insert_with(|| role.to_string());
    }
}

fn insert_symbol(map: &mut HashMap<String, String>, token: &str, canon: &str) {
    if token.is_empty() {
        return;
    }
    map.entry(token.to_string())
        .or_insert_with(|| canon.to_string());
}

fn split_josa_tokens(raw: &str) -> Vec<String> {
    raw.split(|ch: char| ch.is_whitespace() || ch == '/')
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "∅")
        .map(|value| value.to_string())
        .collect()
}

fn normalize_keyword_token(token: &str) -> Option<String> {
    let mut text = token.trim().to_string();
    if text.is_empty() {
        return None;
    }
    if text.ends_with(':') {
        text.pop();
    }
    if text.is_empty() {
        return None;
    }
    if text.contains('#') {
        return None;
    }
    if text.contains(' ') || text.contains('\t') {
        return None;
    }
    if !is_ident_like(&text) {
        return None;
    }
    Some(text)
}

fn split_symbol_tokens(raw: &str) -> Vec<String> {
    raw.split(|ch: char| ch.is_whitespace() || ch == '/')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect()
}

fn is_ident_like(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_ident_start(first) {
        return false;
    }
    for ch in chars {
        if !is_ident_continue(ch) {
            return false;
        }
    }
    true
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_numeric()
}

#[cfg(test)]
mod tests {
    use super::DialectConfig;

    #[test]
    fn active_tag_enables_selected_language_only() {
        let ko = DialectConfig::from_source("일때 참.\n");
        assert_eq!(ko.canonicalize_keyword("if"), None);
        assert_eq!(ko.canonicalize_keyword("もし"), None);
        assert_eq!(ko.canonicalize_keyword("일때"), Some("일때"));

        let en = DialectConfig::from_source("#말씨: en\nif 참.\n");
        assert_eq!(en.canonicalize_keyword("if"), Some("일때"));
        assert_eq!(en.canonicalize_keyword("もし"), None);
    }

    #[test]
    fn sym3_symbol_is_always_available() {
        let ko = DialectConfig::from_source("일때 참.\n");
        let en = DialectConfig::from_source("#말씨: en\nif 참.\n");
        assert_eq!(ko.canonicalize_symbol("?=>"), Some("일때"));
        assert_eq!(en.canonicalize_symbol("?=>"), Some("일때"));
    }

    #[test]
    fn unsupported_tag_does_not_activate_other_language_tokens() {
        let cfg = DialectConfig::from_source("#말씨: xx\nif 참.\n");
        assert_eq!(cfg.canonicalize_keyword("if"), None);
        assert!(cfg.is_inactive_keyword("if"));
    }

    #[test]
    fn josa_tokens_are_canonicalized_to_roles() {
        let cfg = DialectConfig::from_source("#말씨: ko\n값~ob 보여주기.\n");
        assert_eq!(cfg.canonicalize_josa("~ob"), Some("object"));
        assert_eq!(cfg.canonicalize_josa("~sb"), Some("subject"));
    }
}
