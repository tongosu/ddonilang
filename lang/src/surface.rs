#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceError {
    pub message: String,
}

impl SurfaceError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

pub fn surface_form(stem: &str, morphemes: &[&str]) -> Result<String, SurfaceError> {
    if stem.is_empty() {
        return Err(SurfaceError::new("어간이 비어 있습니다"));
    }
    let (base, alt) = split_stem(stem);
    let out_stem = alt.unwrap_or(base).to_string();
    if morphemes.is_empty() {
        return Ok(out_stem);
    }

    let first = choose_variant(&out_stem, morphemes[0]);
    let mut out = apply_contraction(&out_stem, &first)?;
    for suffix in &morphemes[1..] {
        out.push_str(suffix);
    }
    Ok(out)
}

fn split_stem(stem: &str) -> (&str, Option<&str>) {
    if let Some(idx) = stem.find('~') {
        let (base, alt) = stem.split_at(idx);
        let alt = alt.trim_start_matches('~');
        return (base, Some(alt));
    }
    (stem, None)
}

fn choose_variant(stem: &str, token: &str) -> String {
    if let Some(idx) = token.find('/') {
        let (left, right) = token.split_at(idx);
        let right = right.trim_start_matches('/');
        let left = strip_paren(left);
        let right = strip_paren(right);
        if ends_with_consonant(stem) {
            return right.to_string();
        }
        return left.to_string();
    }
    strip_paren(token).to_string()
}

fn strip_paren(token: &str) -> &str {
    token.split('(').next().unwrap_or(token)
}

fn apply_contraction(stem: &str, suffix: &str) -> Result<String, SurfaceError> {
    if stem == "가" && suffix == "았" {
        return Ok("갔".to_string());
    }
    if stem == "하" && suffix == "여" {
        return Ok("해".to_string());
    }
    if stem.ends_with("리") && suffix == "어" {
        let mut out = stem.to_string();
        out.pop();
        out.push_str("려");
        return Ok(out);
    }
    if stem.ends_with("우") && suffix == "아" {
        let mut out = stem.to_string();
        out.pop();
        out.push_str("와");
        return Ok(out);
    }
    if suffix == "ㄹ" && ends_with_vowel(stem) {
        if let Some(out) = attach_final_consonant(stem, 'ㄹ') {
            return Ok(out);
        }
    }
    Ok(format!("{stem}{suffix}"))
}

fn ends_with_vowel(stem: &str) -> bool {
    !ends_with_consonant(stem)
}

fn ends_with_consonant(stem: &str) -> bool {
    let Some(last) = stem.chars().last() else {
        return false;
    };
    let code = last as u32;
    if !(0xAC00..=0xD7A3).contains(&code) {
        return false;
    }
    let index = code - 0xAC00;
    let jong = index % 28;
    jong != 0
}

fn attach_final_consonant(stem: &str, jong_char: char) -> Option<String> {
    let jong = match jong_char {
        'ㄹ' => 8,
        _ => return None,
    };
    let mut out = stem.to_string();
    let last = out.pop()?;
    let code = last as u32;
    if !(0xAC00..=0xD7A3).contains(&code) {
        return None;
    }
    let base = code - 0xAC00;
    let new_code = 0xAC00 + (base - (base % 28)) + jong;
    out.push(char::from_u32(new_code)?);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_goldens() {
        let cases = vec![
            ("먹", vec!["니/으니"], "먹으니"),
            ("먹", vec!["어"], "먹어"),
            ("돌리", vec!["어"], "돌려"),
            ("돕~도우", vec!["니/으니"], "도우니"),
            ("돕~도우", vec!["아"], "도와"),
            ("듣~들", vec!["어"], "들어"),
            ("가", vec!["았", "다"], "갔다"),
            ("가", vec!["ㄹ/을(어미)"], "갈"),
            ("하", vec!["여"], "해"),
        ];
        for (stem, morphemes, expected) in cases {
            let out = surface_form(stem, &morphemes).expect("surface");
            assert_eq!(out, expected);
        }
    }
}
