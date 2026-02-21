use crate::core::fixed64::Fixed64;
use crate::core::unit::{eval_unit_expr, UnitDim, UnitExpr, UnitFactor};
use crate::core::value::{Quantity, Value};
use crate::lang::span::Span;
use crate::runtime::error::RuntimeError;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
struct ParsedTemplate {
    parts: Vec<TemplatePart>,
    root_keys: BTreeSet<String>,
}

#[derive(Clone, Debug)]
enum TemplatePart {
    Text(String),
    Slot {
        path: Vec<String>,
        format: Option<TemplateFormat>,
    },
}

#[derive(Clone, Debug)]
enum TemplateFormat {
    Width {
        width: usize,
        zero_pad: bool,
    },
    Fixed {
        decimals: u8,
        unit: Option<UnitFormat>,
    },
}

#[derive(Clone, Debug)]
struct UnitFormat {
    text: String,
    expr: UnitExpr,
}

pub fn render_template(
    body: &str,
    bindings: &BTreeMap<String, Value>,
    span: Span,
) -> Result<String, RuntimeError> {
    let parsed = parse_template(body, span)?;
    ensure_bindings_match(&parsed.root_keys, bindings, span)?;

    let mut out = String::new();
    for part in parsed.parts {
        match part {
            TemplatePart::Text(text) => out.push_str(&text),
            TemplatePart::Slot { path, format } => {
                let value = resolve_path(bindings, &path, span)?;
                let rendered = render_value(value, format.as_ref(), span)?;
                out.push_str(&rendered);
            }
        }
    }
    Ok(out)
}

pub fn match_template(
    body: &str,
    target: &str,
    span: Span,
) -> Result<Option<BTreeMap<String, Value>>, RuntimeError> {
    let parsed = parse_template(body, span)?;
    let positions = char_positions(target);
    let mut captures = BTreeMap::new();
    if match_parts(&parsed.parts, target, &positions, 0, 0, &mut captures, span)? {
        let mut out = BTreeMap::new();
        for (name, value) in captures {
            out.insert(name, Value::Str(value));
        }
        Ok(Some(out))
    } else {
        Ok(None)
    }
}

fn parse_template(body: &str, span: Span) -> Result<ParsedTemplate, RuntimeError> {
    let chars: Vec<char> = body.chars().collect();
    let mut parts = Vec::new();
    let mut root_keys = BTreeSet::new();
    let mut buf = String::new();
    let mut idx = 0;

    while idx < chars.len() {
        let ch = chars[idx];
        if ch == '{' {
            if idx + 1 < chars.len() && chars[idx + 1] == '{' {
                buf.push('{');
                idx += 2;
                continue;
            }
            if !buf.is_empty() {
                parts.push(TemplatePart::Text(std::mem::take(&mut buf)));
            }
            idx += 1;
            let mut inner = String::new();
            while idx < chars.len() && chars[idx] != '}' {
                inner.push(chars[idx]);
                idx += 1;
            }
            if idx >= chars.len() {
                return Err(RuntimeError::Template {
                    message: "글무늬 자리표시자가 닫히지 않았습니다".to_string(),
                    span,
                });
            }
            idx += 1;
            let (path, format) = parse_placeholder(&inner, span)?;
            if let Some(root) = path.first() {
                root_keys.insert(root.clone());
            }
            parts.push(TemplatePart::Slot { path, format });
            continue;
        }
        if ch == '}' {
            if idx + 1 < chars.len() && chars[idx + 1] == '}' {
                buf.push('}');
                idx += 2;
                continue;
            }
            return Err(RuntimeError::Template {
                message: "글무늬에서 닫는 중괄호가 초과되었습니다".to_string(),
                span,
            });
        }
        buf.push(ch);
        idx += 1;
    }

    if !buf.is_empty() {
        parts.push(TemplatePart::Text(buf));
    }

    Ok(ParsedTemplate { parts, root_keys })
}

fn match_parts(
    parts: &[TemplatePart],
    target: &str,
    positions: &[usize],
    part_idx: usize,
    pos_idx: usize,
    captures: &mut BTreeMap<String, String>,
    span: Span,
) -> Result<bool, RuntimeError> {
    if part_idx == parts.len() {
        return Ok(pos_idx == positions.len() - 1);
    }

    let pos = positions[pos_idx];
    match &parts[part_idx] {
        TemplatePart::Text(text) => {
            let Some(rest) = target.get(pos..) else {
                return Ok(false);
            };
            if !rest.starts_with(text) {
                return Ok(false);
            }
            let next_pos = pos + text.len();
            let next_idx = pos_index(positions, next_pos, span)?;
            match_parts(
                parts,
                target,
                positions,
                part_idx + 1,
                next_idx,
                captures,
                span,
            )
        }
        TemplatePart::Slot { path, format } => {
            if format.is_some() {
                return Err(RuntimeError::Template {
                    message: "맞추기에서는 글무늬 포맷을 사용할 수 없습니다".to_string(),
                    span,
                });
            }
            if path.len() != 1 {
                return Err(RuntimeError::Template {
                    message: "맞추기 자리표시자는 단일 이름이어야 합니다".to_string(),
                    span,
                });
            }
            let name = &path[0];
            if let Some(existing) = captures.get(name) {
                let Some(rest) = target.get(pos..) else {
                    return Ok(false);
                };
                if !rest.starts_with(existing) {
                    return Ok(false);
                }
                let next_pos = pos + existing.len();
                let next_idx = pos_index(positions, next_pos, span)?;
                return match_parts(
                    parts,
                    target,
                    positions,
                    part_idx + 1,
                    next_idx,
                    captures,
                    span,
                );
            }

            // 최소 매칭 규칙: 가능한 가장 짧은 캡처부터 시도한다.
            for end_idx in pos_idx..positions.len() {
                let end_pos = positions[end_idx];
                let Some(segment) = target.get(pos..end_pos) else {
                    continue;
                };
                captures.insert(name.clone(), segment.to_string());
                if match_parts(
                    parts,
                    target,
                    positions,
                    part_idx + 1,
                    end_idx,
                    captures,
                    span,
                )? {
                    return Ok(true);
                }
                captures.remove(name);
            }
            Ok(false)
        }
    }
}

fn char_positions(text: &str) -> Vec<usize> {
    let mut positions = text.char_indices().map(|(idx, _)| idx).collect::<Vec<_>>();
    positions.push(text.len());
    positions
}

fn pos_index(positions: &[usize], pos: usize, span: Span) -> Result<usize, RuntimeError> {
    positions
        .binary_search(&pos)
        .map_err(|_| RuntimeError::Template {
            message: "글무늬 매칭 위치 계산에 실패했습니다".to_string(),
            span,
        })
}

fn parse_placeholder(
    text: &str,
    span: Span,
) -> Result<(Vec<String>, Option<TemplateFormat>), RuntimeError> {
    if text.is_empty() {
        return Err(RuntimeError::Template {
            message: "글무늬 자리표시자 키가 비어 있습니다".to_string(),
            span,
        });
    }
    if text.chars().any(|ch| ch.is_whitespace()) {
        return Err(RuntimeError::Template {
            message: "글무늬 자리표시자에는 공백을 둘 수 없습니다".to_string(),
            span,
        });
    }

    let mut iter = text.split('|');
    let key_part = iter.next().unwrap_or("");
    let format_part = iter.next();
    if iter.next().is_some() {
        return Err(RuntimeError::Template {
            message: "글무늬 포맷 구분자는 하나만 사용할 수 있습니다".to_string(),
            span,
        });
    }

    let path = parse_key_path(key_part, span)?;
    let format = if let Some(format_text) = format_part {
        Some(parse_format_spec(format_text, span)?)
    } else {
        None
    };

    Ok((path, format))
}

fn parse_key_path(text: &str, span: Span) -> Result<Vec<String>, RuntimeError> {
    let mut parts = Vec::new();
    for segment in text.split('.') {
        if segment.is_empty() {
            return Err(RuntimeError::Template {
                message: "글무늬 자리표시자 경로가 비었습니다".to_string(),
                span,
            });
        }
        let mut chars = segment.chars();
        let Some(first) = chars.next() else {
            return Err(RuntimeError::Template {
                message: "글무늬 자리표시자 경로가 비었습니다".to_string(),
                span,
            });
        };
        if !is_ident_start(first) {
            return Err(RuntimeError::Template {
                message: format!("글무늬 자리표시자 키가 올바르지 않습니다: {}", segment),
                span,
            });
        }
        if !chars.all(is_ident_continue) {
            return Err(RuntimeError::Template {
                message: format!("글무늬 자리표시자 키가 올바르지 않습니다: {}", segment),
                span,
            });
        }
        parts.push(segment.to_string());
    }
    Ok(parts)
}

fn parse_format_spec(text: &str, span: Span) -> Result<TemplateFormat, RuntimeError> {
    if text.is_empty() || text.chars().any(|ch| ch.is_whitespace()) {
        return Err(RuntimeError::Template {
            message: "글무늬 포맷이 비어 있습니다".to_string(),
            span,
        });
    }
    let Some(rest) = text.strip_prefix('@') else {
        return Err(RuntimeError::Template {
            message: "글무늬 포맷은 @로 시작해야 합니다".to_string(),
            span,
        });
    };

    if let Some(decimals_part) = rest.strip_prefix('.') {
        let digit_count = decimals_part
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .count();
        if digit_count == 0 {
            return Err(RuntimeError::Template {
                message: "글무늬 포맷 소수 자릿수가 필요합니다".to_string(),
                span,
            });
        }
        let digits: String = decimals_part.chars().take(digit_count).collect();
        let decimals: u8 = digits.parse().map_err(|_| RuntimeError::Template {
            message: "글무늬 포맷 소수 자릿수가 올바르지 않습니다".to_string(),
            span,
        })?;
        if decimals > 9 {
            return Err(RuntimeError::Template {
                message: "글무늬 포맷 소수 자릿수는 0..9 범위입니다".to_string(),
                span,
            });
        }
        let unit_text = &decimals_part[digit_count..];
        if unit_text.is_empty() {
            return Ok(TemplateFormat::Fixed {
                decimals,
                unit: None,
            });
        }
        let expr = parse_unit_expr(unit_text, span)?;
        return Ok(TemplateFormat::Fixed {
            decimals,
            unit: Some(UnitFormat {
                text: unit_text.to_string(),
                expr,
            }),
        });
    }

    let (zero_pad, digits) = if let Some(rest) = rest.strip_prefix('0') {
        (true, rest)
    } else {
        (false, rest)
    };
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(RuntimeError::Template {
            message: "글무늬 포맷 폭은 숫자로 지정해야 합니다".to_string(),
            span,
        });
    }
    let width: usize = digits.parse().map_err(|_| RuntimeError::Template {
        message: "글무늬 포맷 폭이 올바르지 않습니다".to_string(),
        span,
    })?;
    if !(1..=99).contains(&width) {
        return Err(RuntimeError::Template {
            message: "글무늬 포맷 폭은 1..99 범위입니다".to_string(),
            span,
        });
    }
    Ok(TemplateFormat::Width { width, zero_pad })
}

fn parse_unit_expr(text: &str, span: Span) -> Result<UnitExpr, RuntimeError> {
    let chars: Vec<char> = text.chars().collect();
    let mut pos = 0;
    let mut factors = Vec::new();
    let mut sign = 1;

    while pos < chars.len() {
        let name = read_unit_name(&chars, &mut pos).ok_or(RuntimeError::Template {
            message: format!("글무늬 포맷 단위를 파싱할 수 없습니다: {}", text),
            span,
        })?;
        let mut exp = 1;
        if pos < chars.len() && chars[pos] == '^' {
            pos += 1;
            let start = pos;
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
            if start == pos {
                return Err(RuntimeError::Template {
                    message: "글무늬 포맷 단위 지수가 필요합니다".to_string(),
                    span,
                });
            }
            let digits: String = chars[start..pos].iter().collect();
            let value: i32 = digits.parse().map_err(|_| RuntimeError::Template {
                message: "글무늬 포맷 단위 지수가 올바르지 않습니다".to_string(),
                span,
            })?;
            exp = value;
        }
        factors.push(UnitFactor {
            name,
            exp: exp * sign,
        });

        if pos >= chars.len() {
            break;
        }
        match chars[pos] {
            '*' => {
                sign = 1;
                pos += 1;
            }
            '/' => {
                sign = -1;
                pos += 1;
            }
            _ => {
                return Err(RuntimeError::Template {
                    message: format!("글무늬 포맷 단위가 올바르지 않습니다: {}", text),
                    span,
                });
            }
        }
    }

    Ok(UnitExpr { factors })
}

fn read_unit_name(chars: &[char], pos: &mut usize) -> Option<String> {
    if *pos >= chars.len() {
        return None;
    }
    let ch = chars[*pos];
    if !ch.is_ascii_alphabetic() {
        return None;
    }
    let mut name = String::new();
    while *pos < chars.len() {
        let ch = chars[*pos];
        if ch.is_ascii_alphanumeric() {
            name.push(ch);
            *pos += 1;
        } else {
            break;
        }
    }
    Some(name)
}

fn ensure_bindings_match(
    root_keys: &BTreeSet<String>,
    bindings: &BTreeMap<String, Value>,
    span: Span,
) -> Result<(), RuntimeError> {
    let mut missing = Vec::new();
    for key in root_keys {
        if !bindings.contains_key(key) {
            missing.push(key.clone());
        }
    }
    if !missing.is_empty() {
        return Err(RuntimeError::Template {
            message: format!("주입 키 누락: {}", missing.join(", ")),
            span,
        });
    }

    let mut extra = Vec::new();
    for key in bindings.keys() {
        if !root_keys.contains(key) {
            extra.push(key.clone());
        }
    }
    if !extra.is_empty() {
        return Err(RuntimeError::Template {
            message: format!("주입 키 여분: {}", extra.join(", ")),
            span,
        });
    }
    Ok(())
}

fn resolve_path<'a>(
    bindings: &'a BTreeMap<String, Value>,
    path: &[String],
    span: Span,
) -> Result<&'a Value, RuntimeError> {
    let Some((root, rest)) = path.split_first() else {
        return Err(RuntimeError::Template {
            message: "글무늬 자리표시자 경로가 비었습니다".to_string(),
            span,
        });
    };
    let mut current = bindings.get(root).ok_or_else(|| RuntimeError::Template {
        message: format!("주입 키 없음: {}", root),
        span,
    })?;
    if matches!(current, Value::None) {
        return Err(RuntimeError::Template {
            message: format!("주입 키 값이 없음입니다: {}", root),
            span,
        });
    }
    for segment in rest {
        let Value::Pack(pack) = current else {
            return Err(RuntimeError::Template {
                message: format!("묶음이 아니어서 경로를 선택할 수 없습니다: {}", root),
                span,
            });
        };
        current = pack
            .fields
            .get(segment)
            .ok_or_else(|| RuntimeError::Template {
                message: format!("묶음 필드가 없습니다: {}", segment),
                span,
            })?;
        if matches!(current, Value::None) {
            return Err(RuntimeError::Template {
                message: format!("묶음 필드 값이 없음입니다: {}", segment),
                span,
            });
        }
    }
    Ok(current)
}

fn render_value(
    value: &Value,
    format: Option<&TemplateFormat>,
    span: Span,
) -> Result<String, RuntimeError> {
    if matches!(value, Value::None) {
        return Err(RuntimeError::Template {
            message: "없음은 글무늬에 사용할 수 없습니다".to_string(),
            span,
        });
    }
    match format {
        None => Ok(value.display()),
        Some(fmt) => match value {
            Value::Num(qty) => format_quantity(qty, fmt, span),
            _ => Err(RuntimeError::Template {
                message: "글무늬 포맷은 수 값에만 사용할 수 있습니다".to_string(),
                span,
            }),
        },
    }
}

fn format_quantity(
    qty: &Quantity,
    format: &TemplateFormat,
    span: Span,
) -> Result<String, RuntimeError> {
    match format {
        TemplateFormat::Width { width, zero_pad } => {
            let numeric = qty.raw.format();
            let padded = pad_width(&numeric, *width, *zero_pad);
            let mut out = padded;
            if qty.dim != UnitDim::zero() {
                out.push('@');
                out.push_str(&crate::core::unit::format_dim(qty.dim));
            }
            Ok(out)
        }
        TemplateFormat::Fixed { decimals, unit } => {
            let (value, unit_suffix) = if let Some(unit) = unit {
                let (dim, scale) = eval_unit_expr(&unit.expr).map_err(|err| {
                    let unit_name = match err {
                        crate::core::unit::UnitError::Unknown(name) => name,
                        crate::core::unit::UnitError::Overflow => "overflow".to_string(),
                    };
                    RuntimeError::UnitUnknown {
                        unit: unit_name,
                        span,
                    }
                })?;
                if dim != qty.dim {
                    return Err(RuntimeError::UnitMismatch { span });
                }
                (scale.apply_inverse(qty.raw), Some(unit.text.clone()))
            } else if qty.dim == UnitDim::zero() {
                (qty.raw, None)
            } else {
                (qty.raw, Some(crate::core::unit::format_dim(qty.dim)))
            };

            let numeric = format_fixed_decimals(value, *decimals);
            let mut out = numeric;
            if let Some(unit_text) = unit_suffix {
                out.push('@');
                out.push_str(&unit_text);
            }
            Ok(out)
        }
    }
}

fn pad_width(text: &str, width: usize, zero_pad: bool) -> String {
    let (sign, rest) = text
        .strip_prefix('-')
        .map_or(("", text), |rest| ("-", rest));
    let total_len = sign.len() + rest.len();
    if total_len >= width {
        return text.to_string();
    }
    let pad_len = width - total_len;
    let pad_char = if zero_pad { '0' } else { ' ' };
    let mut out = String::new();
    out.push_str(sign);
    for _ in 0..pad_len {
        out.push(pad_char);
    }
    out.push_str(rest);
    out
}

fn format_fixed_decimals(value: Fixed64, decimals: u8) -> String {
    let scaled = round_fixed_decimals(value.raw(), decimals);
    let mut negative = scaled < 0;
    let abs = scaled.abs();
    if abs == 0 {
        negative = false;
    }
    let factor = pow10(decimals as usize);
    let int_part = abs / factor;
    let frac_part = abs % factor;

    let mut out = int_part.to_string();
    if decimals > 0 {
        out.push('.');
        out.push_str(&format!("{:0width$}", frac_part, width = decimals as usize));
    }
    if negative {
        out.insert(0, '-');
    }
    out
}

fn round_fixed_decimals(raw: i64, decimals: u8) -> i128 {
    let scale = pow10(decimals as usize) as i128;
    let numer = (raw as i128).saturating_mul(scale);
    let denom = Fixed64::SCALE as i128;
    div_round_ties_even(numer, denom)
}

fn div_round_ties_even(numer: i128, denom: i128) -> i128 {
    if denom == 0 {
        return 0;
    }
    let sign = if numer < 0 { -1 } else { 1 };
    let abs = numer.abs();
    let q = abs / denom;
    let r = abs % denom;
    let half = denom / 2;
    let rounded = if r < half {
        q
    } else if r > half {
        q + 1
    } else if q % 2 == 0 {
        q
    } else {
        q + 1
    };
    rounded * sign
}

fn pow10(exp: usize) -> i128 {
    let mut value: i128 = 1;
    for _ in 0..exp {
        value = value.saturating_mul(10);
    }
    value
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic() || is_hangul(ch)
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

fn is_hangul(ch: char) -> bool {
    matches!(
        ch,
        '\u{AC00}'..='\u{D7AF}' | '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}'
    )
}
