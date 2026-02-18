use ddonirang_lang::DialectConfig as SharedDialectConfig;

#[derive(Clone, Debug)]
pub struct DialectConfig {
    inner: SharedDialectConfig,
}

impl DialectConfig {
    pub fn from_source(source: &str) -> Self {
        Self {
            inner: SharedDialectConfig::from_source(source),
        }
    }

    pub fn canonicalize<'a>(&'a self, token: &str) -> Option<&'a str> {
        self.inner.canonicalize(token)
    }

    pub fn canonicalize_symbol<'a>(&'a self, token: &str) -> Option<&'a str> {
        self.inner.canonicalize_symbol(token)
    }

    pub fn canonicalize_josa<'a>(&'a self, token: &str) -> Option<&'a str> {
        self.inner.canonicalize_josa(token)
    }

    pub fn sym3_tokens() -> &'static [String] {
        SharedDialectConfig::sym3_tokens()
    }

    pub fn is_inactive_keyword(&self, token: &str) -> bool {
        self.inner.is_inactive_keyword(token)
    }
}

#[cfg(test)]
mod tests {
    use super::DialectConfig;

    #[test]
    fn active_tag_enables_selected_language_only() {
        let ko = DialectConfig::from_source("일때 참.\n");
        assert_eq!(ko.canonicalize("if"), None);
        assert_eq!(ko.canonicalize("もし"), None);
        assert_eq!(ko.canonicalize("일때"), Some("일때"));

        let en = DialectConfig::from_source("#말씨: en\nif 참.\n");
        assert_eq!(en.canonicalize("if"), Some("일때"));
        assert_eq!(en.canonicalize("もし"), None);
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
        assert_eq!(cfg.canonicalize("if"), None);
        assert!(cfg.is_inactive_keyword("if"));
    }
}
