pub fn build_diag(code: &str, detail: &str, hint: Option<String>, fix: Option<String>) -> String {
    let mut out = format!("{} {}", code, detail);
    if let Some(h) = hint {
        if !h.is_empty() {
            out.push_str(&format!(" hint={}", h));
        }
    }
    if let Some(f) = fix {
        if !f.is_empty() {
            out.push_str(&format!(" fix={}", f));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::build_diag;

    #[test]
    fn build_diag_with_all_fields() {
        let text = build_diag(
            "E_CODE",
            "detail",
            Some("h1".to_string()),
            Some("f1".to_string()),
        );
        assert_eq!(text, "E_CODE detail hint=h1 fix=f1");
    }

    #[test]
    fn build_diag_omits_empty_fields() {
        let text = build_diag("E_CODE", "detail", Some(String::new()), Some(String::new()));
        assert_eq!(text, "E_CODE detail");
    }
}
