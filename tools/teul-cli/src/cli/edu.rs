use std::fs;
use std::path::{Path, PathBuf};

use ddonirang_core::Fixed64;
use serde::Deserialize;
use serde_json::{Map, Value as JsonValue};

use super::detjson::write_text;
use super::paths;

const DEFAULT_EPS: &str = "0.000000001";

#[derive(Deserialize)]
struct EduScenarioInput {
    schema: Option<String>,
    scenario_id: String,
    cases: Vec<EduCaseInput>,
    abs_tol: Option<String>,
    rel_tol: Option<String>,
    eps: Option<String>,
}

#[derive(Deserialize)]
struct EduCaseInput {
    id: String,
    formula: Vec<JsonValue>,
    sim: Vec<JsonValue>,
    abs_tol: Option<String>,
    rel_tol: Option<String>,
    eps: Option<String>,
}

#[derive(Clone)]
struct CaseResult {
    id: String,
    total_count: u64,
    pass_count: u64,
    match_ratio: Fixed64,
}

pub fn run_accuracy(input: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(input)
        .map_err(|e| format!("E_EDU_INPUT_READ {} {}", input.display(), e))?;
    let scenario: EduScenarioInput =
        serde_json::from_str(&text).map_err(|e| format!("E_EDU_INPUT_PARSE {}", e))?;
    if let Some(schema) = scenario.schema.as_deref() {
        if schema != "edu.scenario.v1" {
            return Err(format!("E_EDU_SCHEMA {}", schema));
        }
    }

    let abs_tol_default = parse_fixed64_string(scenario.abs_tol.as_deref().unwrap_or("0"))
        .map_err(|e| format!("E_EDU_ABS_TOL {}", e))?;
    let rel_tol_default = parse_fixed64_string(scenario.rel_tol.as_deref().unwrap_or("0"))
        .map_err(|e| format!("E_EDU_REL_TOL {}", e))?;
    let eps_default = parse_fixed64_string(scenario.eps.as_deref().unwrap_or(DEFAULT_EPS))
        .map_err(|e| format!("E_EDU_EPS {}", e))?;

    let mut total_count = 0u64;
    let mut pass_count = 0u64;
    let mut case_results = Vec::with_capacity(scenario.cases.len());

    for case in scenario.cases {
        let abs_tol = if let Some(value) = case.abs_tol.as_deref() {
            parse_fixed64_string(value).map_err(|e| format!("E_EDU_ABS_TOL {}", e))?
        } else {
            abs_tol_default
        };
        let rel_tol = if let Some(value) = case.rel_tol.as_deref() {
            parse_fixed64_string(value).map_err(|e| format!("E_EDU_REL_TOL {}", e))?
        } else {
            rel_tol_default
        };
        let eps = if let Some(value) = case.eps.as_deref() {
            parse_fixed64_string(value).map_err(|e| format!("E_EDU_EPS {}", e))?
        } else {
            eps_default
        };

        if case.formula.len() != case.sim.len() {
            return Err(format!(
                "E_EDU_CASE_LEN {} formula/sim 길이가 다릅니다",
                case.id
            ));
        }

        let mut case_total = 0u64;
        let mut case_pass = 0u64;
        for (formula_val, sim_val) in case.formula.iter().zip(case.sim.iter()) {
            let formula = parse_fixed64_value(formula_val)
                .map_err(|e| format!("E_EDU_FORMULA {} {}", case.id, e))?;
            let sim =
                parse_fixed64_value(sim_val).map_err(|e| format!("E_EDU_SIM {} {}", case.id, e))?;
            let abs_err = abs_fixed64(sim - formula);
            let denom = max_fixed64(abs_fixed64(formula), eps);
            let rel_err = abs_err
                .try_div(denom)
                .map_err(|_| "E_EDU_DIV_ZERO rel_err 분모가 0입니다".to_string())?;

            let pass = abs_err <= abs_tol || rel_err <= rel_tol;
            case_total += 1;
            if pass {
                case_pass += 1;
            }
        }

        total_count += case_total;
        pass_count += case_pass;
        let match_ratio = fixed_ratio(case_pass, case_total)?;
        case_results.push(CaseResult {
            id: case.id,
            total_count: case_total,
            pass_count: case_pass,
            match_ratio,
        });
    }

    if total_count == 0 {
        return Err("E_EDU_EMPTY 케이스가 비었습니다".to_string());
    }

    let report = build_report(
        &scenario.scenario_id,
        total_count,
        pass_count,
        &case_results,
    )?;

    let out_dir = resolve_out_dir(out_dir);
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    write_text(&out_dir.join("edu.report.detjson"), &report)?;

    println!("{}", report);
    Ok(())
}

fn resolve_out_dir(out_dir: Option<&Path>) -> PathBuf {
    match out_dir {
        Some(path) => path.to_path_buf(),
        None => paths::build_dir().join("edu"),
    }
}

fn build_report(
    scenario_id: &str,
    total_count: u64,
    pass_count: u64,
    cases: &[CaseResult],
) -> Result<String, String> {
    let match_ratio = fixed_ratio(pass_count, total_count)?;

    let mut map = Map::new();
    map.insert(
        "schema".to_string(),
        JsonValue::String("edu.report.v1".to_string()),
    );
    map.insert(
        "scenario_id".to_string(),
        JsonValue::String(scenario_id.to_string()),
    );
    map.insert(
        "total_count".to_string(),
        JsonValue::Number(total_count.into()),
    );
    map.insert(
        "pass_count".to_string(),
        JsonValue::Number(pass_count.into()),
    );
    map.insert(
        "match_ratio".to_string(),
        JsonValue::String(match_ratio.to_string()),
    );

    let mut case_list = Vec::with_capacity(cases.len());
    for case in cases {
        let mut case_map = Map::new();
        case_map.insert("id".to_string(), JsonValue::String(case.id.clone()));
        case_map.insert(
            "total_count".to_string(),
            JsonValue::Number(case.total_count.into()),
        );
        case_map.insert(
            "pass_count".to_string(),
            JsonValue::Number(case.pass_count.into()),
        );
        case_map.insert(
            "match_ratio".to_string(),
            JsonValue::String(case.match_ratio.to_string()),
        );
        case_list.push(JsonValue::Object(case_map));
    }
    map.insert("cases".to_string(), JsonValue::Array(case_list));

    Ok(JsonValue::Object(map).to_string())
}

fn fixed_ratio(pass_count: u64, total_count: u64) -> Result<Fixed64, String> {
    if total_count == 0 {
        return Err("E_EDU_DIV_ZERO total_count=0".to_string());
    }
    let pass = Fixed64::from_i64(pass_count as i64);
    let total = Fixed64::from_i64(total_count as i64);
    pass.try_div(total)
        .map_err(|_| "E_EDU_DIV_ZERO total_count=0".to_string())
}

fn abs_fixed64(value: Fixed64) -> Fixed64 {
    if value.raw_i64() < 0 {
        Fixed64::from_raw_i64(value.raw_i64().saturating_neg())
    } else {
        value
    }
}

fn max_fixed64(a: Fixed64, b: Fixed64) -> Fixed64 {
    if a.raw_i64() >= b.raw_i64() {
        a
    } else {
        b
    }
}

fn parse_fixed64_value(value: &JsonValue) -> Result<Fixed64, String> {
    match value {
        JsonValue::String(text) => parse_fixed64_string(text),
        JsonValue::Number(num) => parse_fixed64_string(&num.to_string()),
        _ => Err("수치는 문자열 또는 숫자여야 합니다".to_string()),
    }
}

fn parse_fixed64_string(input: &str) -> Result<Fixed64, String> {
    let trimmed = input.trim();
    if let Some(raw) = trimmed.strip_prefix("raw:") {
        let raw_value = raw
            .trim()
            .parse::<i64>()
            .map_err(|_| format!("Fixed64 raw 변환 실패: {}", input))?;
        return Ok(Fixed64::from_raw_i64(raw_value));
    }
    parse_fixed64_decimal(trimmed)
}

fn parse_fixed64_decimal(input: &str) -> Result<Fixed64, String> {
    let text = input.trim();
    if text.is_empty() {
        return Err("Fixed64 문자열이 비었습니다".to_string());
    }
    let mut sign = 1i128;
    let mut raw_text = text;
    if let Some(rest) = raw_text.strip_prefix('-') {
        sign = -1;
        raw_text = rest;
    } else if let Some(rest) = raw_text.strip_prefix('+') {
        raw_text = rest;
    }

    let parts: Vec<&str> = raw_text.split('.').collect();
    if parts.len() > 2 {
        return Err(format!("Fixed64 문자열이 잘못되었습니다: {}", input));
    }
    let int_part = parts[0];
    if int_part.is_empty() {
        return Err(format!("Fixed64 정수부가 비었습니다: {}", input));
    }
    let int_value = int_part
        .parse::<i128>()
        .map_err(|_| format!("Fixed64 정수부 변환 실패: {}", input))?;

    let frac_value = if parts.len() == 2 {
        let frac_part = parts[1];
        if frac_part.is_empty() {
            0i128
        } else if frac_part.len() > 12 {
            return Err(format!("Fixed64 소수부 자리수가 너무 깁니다: {}", input));
        } else {
            let frac_digits = frac_part
                .parse::<i128>()
                .map_err(|_| format!("Fixed64 소수부 변환 실패: {}", input))?;
            let scale = 10i128.pow(frac_part.len() as u32);
            let frac_scaled = (frac_digits << 32) / scale;
            frac_scaled
        }
    } else {
        0i128
    };

    let raw = ((int_value << 32) + frac_value) * sign;
    let raw_i64 = if raw > i64::MAX as i128 {
        i64::MAX
    } else if raw < i64::MIN as i128 {
        i64::MIN
    } else {
        raw as i64
    };
    Ok(Fixed64::from_raw_i64(raw_i64))
}
