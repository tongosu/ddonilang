use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{Map, Value as JsonValue};

use crate::cli::detjson::write_text;
use crate::cli::paths;
use crate::core::fixed64::Fixed64;
use crate::core::state::{Key, State};
use crate::core::unit::UnitDim;
use crate::core::value::{Quantity, Value};
use crate::lang::lexer::Lexer;
use crate::lang::parser::Parser;
use crate::runtime::{Evaluator, OpenRuntime};

#[derive(Deserialize)]
struct MacroMicroRunnerInput {
    schema: Option<String>,
    #[serde(default)]
    seed: u64,
    ticks: u64,
    shock: Option<RunnerShock>,
    models: RunnerModels,
    #[serde(default)]
    diagnostics: Vec<RunnerDiagnostic>,
    report_path: Option<String>,
}

#[derive(Deserialize)]
struct RunnerShock {
    #[serde(rename = "type")]
    kind: Option<String>,
    target: Option<String>,
    delta: Option<JsonValue>,
    #[serde(rename = "at_tick")]
    at_tick: Option<u64>,
    scope: Option<String>,
}

#[derive(Deserialize)]
struct RunnerModels {
    #[serde(rename = "거시")]
    macro_model: String,
    #[serde(rename = "미시")]
    micro_model: String,
}

#[derive(Deserialize)]
struct RunnerDiagnostic {
    name: String,
    lhs: String,
    rhs: String,
    threshold: Option<JsonValue>,
    #[serde(rename = "허용오차")]
    tolerance: Option<JsonValue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShockScope {
    Both,
    MacroOnly,
    MicroOnly,
}

impl ShockScope {
    fn applies_macro(self) -> bool {
        matches!(self, ShockScope::Both | ShockScope::MacroOnly)
    }

    fn applies_micro(self) -> bool {
        matches!(self, ShockScope::Both | ShockScope::MicroOnly)
    }

    fn label(self) -> &'static str {
        match self {
            ShockScope::Both => "양쪽",
            ShockScope::MacroOnly => "거시",
            ShockScope::MicroOnly => "미시",
        }
    }
}

#[derive(Clone, Debug)]
struct ShockSpec {
    kind: Option<String>,
    target: String,
    delta: Fixed64,
    at_tick: u64,
    scope: ShockScope,
}

pub fn run_macro_micro(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(input)
        .map_err(|e| format!("E_ECO_RUNNER_READ {} {}", input.display(), e))?;
    let spec: MacroMicroRunnerInput =
        serde_json::from_str(&text).map_err(|e| format!("E_ECO_RUNNER_PARSE {}", e))?;
    if spec.schema.as_deref() != Some("ddn.macro_micro_runner.v0") {
        return Err(
            "E_ECO_RUNNER_SCHEMA schema=ddn.macro_micro_runner.v0 이어야 합니다".to_string(),
        );
    }
    if spec.ticks == 0 {
        return Err("E_ECO_RUNNER_TICKS ticks는 1 이상이어야 합니다".to_string());
    }
    if spec.diagnostics.is_empty() {
        return Err("E_ECO_RUNNER_DIAG diagnostics는 최소 1개 이상이어야 합니다".to_string());
    }
    let shock_spec = parse_shock(spec.shock.as_ref(), spec.ticks)?;

    let base_dir = input.parent().unwrap_or_else(|| Path::new("."));
    let macro_path = resolve_model_path(base_dir, &spec.models.macro_model);
    let micro_path = resolve_model_path(base_dir, &spec.models.micro_model);
    let macro_source = fs::read_to_string(&macro_path)
        .map_err(|e| format!("E_ECO_RUNNER_MODEL_READ {} {}", macro_path.display(), e))?;
    let micro_source = fs::read_to_string(&micro_path)
        .map_err(|e| format!("E_ECO_RUNNER_MODEL_READ {} {}", micro_path.display(), e))?;

    let macro_shock = shock_spec
        .as_ref()
        .filter(|shock| shock.scope.applies_macro());
    let micro_shock = shock_spec
        .as_ref()
        .filter(|shock| shock.scope.applies_micro());
    let macro_states = run_model_series(&macro_source, spec.seed, spec.ticks, macro_shock)?;
    let micro_states = run_model_series(&micro_source, spec.seed, spec.ticks, micro_shock)?;
    let shock_tick = shock_spec.as_ref().map(|shock| shock.at_tick);

    let mut results = Vec::with_capacity(spec.diagnostics.len());
    for diagnostic in &spec.diagnostics {
        let threshold = parse_threshold(diagnostic)?;
        let mut divergence_tick: Option<u64> = None;
        let mut max_delta = Fixed64::zero();
        let mut before_converged_all = true;
        let mut after_converged_all = true;
        let mut before_seen = false;
        let mut after_seen = false;

        for tick in 1..=spec.ticks {
            let macro_state = &macro_states[(tick - 1) as usize];
            let micro_state = &micro_states[(tick - 1) as usize];
            let lhs = resolve_reference(&diagnostic.lhs, macro_state, micro_state)?;
            let rhs = resolve_reference(&diagnostic.rhs, macro_state, micro_state)?;
            let delta = fixed64_abs(lhs.saturating_sub(rhs));
            if delta.raw() > max_delta.raw() {
                max_delta = delta;
            }
            let converged = delta.raw() <= threshold.raw();
            if !converged && divergence_tick.is_none() {
                divergence_tick = Some(tick);
            }
            if let Some(shock_tick) = shock_tick {
                if tick < shock_tick {
                    before_seen = true;
                    before_converged_all &= converged;
                } else {
                    after_seen = true;
                    after_converged_all &= converged;
                }
            } else {
                before_seen = true;
                before_converged_all &= converged;
                after_seen = true;
                after_converged_all &= converged;
            }
        }

        if !before_seen {
            before_converged_all = true;
        }
        if !after_seen {
            after_converged_all = true;
        }

        let mut entry = Map::new();
        entry.insert(
            "name".to_string(),
            JsonValue::String(diagnostic.name.clone()),
        );
        entry.insert(
            "convergence_before_shock".to_string(),
            JsonValue::Bool(before_converged_all),
        );
        entry.insert(
            "convergence_after_shock".to_string(),
            JsonValue::Bool(after_converged_all),
        );
        match divergence_tick {
            Some(value) => {
                entry.insert(
                    "divergence_tick".to_string(),
                    JsonValue::Number(value.into()),
                );
                entry.insert(
                    "max_delta".to_string(),
                    JsonValue::String(max_delta.format()),
                );
                entry.insert(
                    "error_code".to_string(),
                    JsonValue::String(infer_error_code(diagnostic).to_string()),
                );
            }
            None => {
                entry.insert("divergence_tick".to_string(), JsonValue::Null);
            }
        }
        results.push(JsonValue::Object(entry));
    }

    let mut report = BTreeMap::new();
    report.insert(
        "schema".to_string(),
        JsonValue::String("ddn.runner_report.v0".to_string()),
    );
    report.insert(
        "seed".to_string(),
        JsonValue::Number(serde_json::Number::from(spec.seed)),
    );
    report.insert(
        "ticks".to_string(),
        JsonValue::Number(serde_json::Number::from(spec.ticks)),
    );
    report.insert(
        "shock_tick".to_string(),
        match shock_tick {
            Some(value) => JsonValue::Number(serde_json::Number::from(value)),
            None => JsonValue::Null,
        },
    );
    if let Some(shock) = shock_spec.as_ref() {
        report.insert(
            "shock_target".to_string(),
            JsonValue::String(shock.target.clone()),
        );
        report.insert(
            "shock_delta".to_string(),
            JsonValue::String(shock.delta.format()),
        );
        report.insert(
            "shock_scope".to_string(),
            JsonValue::String(shock.scope.label().to_string()),
        );
        if let Some(kind) = shock.kind.as_ref() {
            report.insert("shock_type".to_string(), JsonValue::String(kind.clone()));
        }
    }
    report.insert("results".to_string(), JsonValue::Array(results));
    let report_json = JsonValue::Object(report.into_iter().collect::<Map<String, JsonValue>>());
    let report_text = serde_json::to_string(&report_json)
        .map_err(|e| format!("E_ECO_RUNNER_REPORT_JSON {}", e))?;

    let out_path = resolve_report_path(input, spec.report_path.as_deref(), out);
    ensure_parent_dir(&out_path, "E_ECO_RUNNER_DIR")?;
    write_text(&out_path, &(report_text + "\n"))?;
    println!("eco_runner_report={}", out_path.display());
    Ok(())
}

pub fn run_network_flow(
    input: &Path,
    ticks: u64,
    seed: u64,
    threshold: Fixed64,
    out: Option<&Path>,
) -> Result<(), String> {
    if ticks == 0 {
        return Err("E_ECO_NETWORK_FLOW ticks는 1 이상이어야 합니다".to_string());
    }
    let source = fs::read_to_string(input)
        .map_err(|e| format!("E_ECO_NETWORK_FLOW_READ {} {}", input.display(), e))?;
    let state = run_model_snapshot(&source, seed, ticks, None)?;
    let lhs = resolve_network_income(&state)?;
    let rhs = resolve_network_spending(&state)?;
    let delta = fixed64_abs(lhs.saturating_sub(rhs));
    let converged = delta.raw() <= threshold.raw();

    let mut report = BTreeMap::new();
    report.insert(
        "schema".to_string(),
        JsonValue::String("ddn.eco.network_flow_report.v0".to_string()),
    );
    report.insert(
        "seed".to_string(),
        JsonValue::Number(serde_json::Number::from(seed)),
    );
    report.insert(
        "ticks".to_string(),
        JsonValue::Number(serde_json::Number::from(ticks)),
    );
    report.insert("lhs".to_string(), JsonValue::String(lhs.format()));
    report.insert("rhs".to_string(), JsonValue::String(rhs.format()));
    report.insert("delta".to_string(), JsonValue::String(delta.format()));
    report.insert(
        "threshold".to_string(),
        JsonValue::String(threshold.format()),
    );
    report.insert(
        "result".to_string(),
        JsonValue::String(if converged {
            "수렴".to_string()
        } else {
            "발산".to_string()
        }),
    );
    if !converged {
        report.insert(
            "error_code".to_string(),
            JsonValue::String("E_SFC_IDENTITY_VIOLATION".to_string()),
        );
    }
    let out_path = resolve_network_flow_report_path(out);
    ensure_parent_dir(&out_path, "E_ECO_NETWORK_FLOW_DIR")?;
    write_text(
        &out_path,
        &(serde_json::to_string(&JsonValue::Object(
            report.into_iter().collect::<Map<String, JsonValue>>(),
        ))
        .map_err(|e| format!("E_ECO_NETWORK_FLOW_JSON {}", e))?
            + "\n"),
    )?;
    println!("eco_network_flow_report={}", out_path.display());
    if converged {
        Ok(())
    } else {
        Err("E_SFC_IDENTITY_VIOLATION".to_string())
    }
}

pub fn run_abm_spatial(
    input: &Path,
    ticks: u64,
    seed: u64,
    out: Option<&Path>,
) -> Result<(), String> {
    if ticks == 0 {
        return Err("E_ECO_ABM_SPATIAL ticks는 1 이상이어야 합니다".to_string());
    }
    let source = fs::read_to_string(input)
        .map_err(|e| format!("E_ECO_ABM_SPATIAL_READ {} {}", input.display(), e))?;
    let state = run_model_snapshot(&source, seed, ticks, None)?;
    let wealth = extract_wealth(&state);

    let gini = fixed_from_state_key(&state, "지니계수")
        .or_else(|| wealth.as_ref().map(|list| eco_gini(list)))
        .unwrap_or_else(Fixed64::zero);
    let mean_wealth = fixed_from_state_key(&state, "평균부")
        .or_else(|| wealth.as_ref().map(|list| eco_mean(list)))
        .unwrap_or_else(Fixed64::zero);
    let max_wealth = fixed_from_state_key(&state, "최대부")
        .or_else(|| wealth.as_ref().map(|list| eco_max(list)))
        .unwrap_or_else(Fixed64::zero);
    let p90_wealth = fixed_from_state_key(&state, "분위90")
        .or_else(|| {
            wealth
                .as_ref()
                .map(|list| eco_quantile_linear(list, fixed_literal("0.9")))
        })
        .unwrap_or_else(Fixed64::zero);

    let mut report = BTreeMap::new();
    report.insert(
        "schema".to_string(),
        JsonValue::String("ddn.eco.abm_spatial_report.v0".to_string()),
    );
    report.insert(
        "seed".to_string(),
        JsonValue::Number(serde_json::Number::from(seed)),
    );
    report.insert(
        "ticks".to_string(),
        JsonValue::Number(serde_json::Number::from(ticks)),
    );
    report.insert("gini".to_string(), JsonValue::String(gini.format()));
    report.insert(
        "mean_wealth".to_string(),
        JsonValue::String(mean_wealth.format()),
    );
    report.insert(
        "max_wealth".to_string(),
        JsonValue::String(max_wealth.format()),
    );
    report.insert(
        "p90_wealth".to_string(),
        JsonValue::String(p90_wealth.format()),
    );
    if let Some(wealth) = wealth.as_ref() {
        report.insert(
            "agent_count".to_string(),
            JsonValue::Number(serde_json::Number::from(wealth.len() as u64)),
        );
    }
    let out_path = resolve_abm_spatial_report_path(out);
    ensure_parent_dir(&out_path, "E_ECO_ABM_SPATIAL_DIR")?;
    write_text(
        &out_path,
        &(serde_json::to_string(&JsonValue::Object(
            report.into_iter().collect::<Map<String, JsonValue>>(),
        ))
        .map_err(|e| format!("E_ECO_ABM_SPATIAL_JSON {}", e))?
            + "\n"),
    )?;
    println!("eco_abm_spatial_report={}", out_path.display());
    Ok(())
}

fn run_model_series(
    source: &str,
    seed: u64,
    ticks: u64,
    shock: Option<&ShockSpec>,
) -> Result<Vec<State>, String> {
    let tokens =
        Lexer::tokenize(source).map_err(|e| format!("E_ECO_RUNNER_MODEL_LEX {}", e.code()))?;
    let default_root = Parser::default_root_for_source(source);
    let program = Parser::parse_with_default_root(tokens, default_root)
        .map_err(|e| format!("E_ECO_RUNNER_MODEL_PARSE {}", e.code()))?;
    let evaluator = Evaluator::with_state_seed_open(
        State::new(),
        seed,
        OpenRuntime::deny(),
        "<eco_runner>".to_string(),
        Some(source.to_string()),
    );
    let mut states = Vec::with_capacity(ticks as usize);
    evaluator
        .run_with_ticks_observe_and_inject(
            &program,
            ticks,
            |madi, state| {
                let tick = madi + 1;
                if let Some(shock) = shock {
                    if tick == shock.at_tick {
                        apply_shock(state, shock);
                    }
                }
                Ok(())
            },
            |_, state, _| {
                states.push(state.clone());
            },
        )
        .map_err(|e| format!("E_ECO_RUNNER_MODEL_EXEC {}", e.code()))?;
    Ok(states)
}

fn run_model_snapshot(
    source: &str,
    seed: u64,
    ticks: u64,
    shock: Option<&ShockSpec>,
) -> Result<State, String> {
    let states = run_model_series(source, seed, ticks, shock)?;
    states
        .into_iter()
        .last()
        .ok_or_else(|| "E_ECO_MODEL_EXEC 실행 결과가 비어 있습니다".to_string())
}

fn apply_shock(state: &mut State, shock: &ShockSpec) {
    let key = Key::new(shock.target.clone());
    let base = match state.get(&key) {
        Some(Value::Num(quantity)) => quantity.raw,
        _ => Fixed64::zero(),
    };
    let next = base.saturating_add(shock.delta);
    state.set(key, Value::Num(Quantity::new(next, UnitDim::zero())));
}

fn parse_threshold(diagnostic: &RunnerDiagnostic) -> Result<Fixed64, String> {
    let source = diagnostic
        .tolerance
        .as_ref()
        .or(diagnostic.threshold.as_ref());
    let Some(value) = source else {
        return Ok(Fixed64::zero());
    };
    match value {
        JsonValue::Number(num) => Fixed64::parse_literal(&num.to_string()).ok_or_else(|| {
            format!(
                "E_ECO_RUNNER_THRESHOLD {} threshold 파싱 실패",
                diagnostic.name
            )
        }),
        JsonValue::String(text) => Fixed64::parse_literal(text).ok_or_else(|| {
            format!(
                "E_ECO_RUNNER_THRESHOLD {} threshold 파싱 실패",
                diagnostic.name
            )
        }),
        _ => Err(format!(
            "E_ECO_RUNNER_THRESHOLD {} threshold는 숫자/문자열이어야 합니다",
            diagnostic.name
        )),
    }
}

fn resolve_reference(
    reference: &str,
    macro_state: &State,
    micro_state: &State,
) -> Result<Fixed64, String> {
    let trimmed = reference.trim();
    if let Some(literal) = Fixed64::parse_literal(trimmed) {
        return Ok(literal);
    }
    let (scope, key) = split_reference(trimmed)?;
    let state = match scope {
        ModelScope::Macro => macro_state,
        ModelScope::Micro => micro_state,
    };
    let value = state
        .get(&Key::new(key.clone()))
        .ok_or_else(|| format!("E_ECO_RUNNER_REF {} 키를 찾을 수 없습니다", trimmed))?;
    match value {
        Value::Num(quantity) => Ok(quantity.raw),
        _ => Err(format!("E_ECO_RUNNER_REF {} 숫자 값이 아닙니다", trimmed)),
    }
}

#[derive(Clone, Copy)]
enum ModelScope {
    Macro,
    Micro,
}

fn split_reference(text: &str) -> Result<(ModelScope, String), String> {
    let Some((scope, key)) = text.split_once('.') else {
        return Err(format!(
            "E_ECO_RUNNER_REF {} 참조는 <거시|미시>.<키> 형식이어야 합니다",
            text
        ));
    };
    let scope = match scope.trim() {
        "거시" | "macro" | "Macro" | "MACRO" => ModelScope::Macro,
        "미시" | "micro" | "Micro" | "MICRO" => ModelScope::Micro,
        _ => {
            return Err(format!(
                "E_ECO_RUNNER_REF {} scope는 거시/미시여야 합니다",
                text
            ))
        }
    };
    let key = key.trim();
    if key.is_empty() {
        return Err(format!("E_ECO_RUNNER_REF {} 키가 비었습니다", text));
    }
    Ok((scope, key.to_string()))
}

fn fixed64_abs(value: Fixed64) -> Fixed64 {
    if value.raw() < 0 {
        Fixed64::from_raw(value.raw().saturating_neg())
    } else {
        value
    }
}

fn infer_error_code(diagnostic: &RunnerDiagnostic) -> &'static str {
    if diagnostic.name.to_ascii_uppercase().contains("SFC")
        || ((diagnostic.lhs.contains("총수입") && diagnostic.rhs.contains("총지출"))
            || (diagnostic.lhs.contains("총지출") && diagnostic.rhs.contains("총수입")))
    {
        "E_SFC_IDENTITY_VIOLATION"
    } else {
        "E_ECO_DIVERGENCE_DETECTED"
    }
}

fn parse_shock(shock: Option<&RunnerShock>, ticks: u64) -> Result<Option<ShockSpec>, String> {
    let Some(shock) = shock else {
        return Ok(None);
    };
    let at_tick = shock
        .at_tick
        .ok_or_else(|| "E_ECO_RUNNER_SHOCK at_tick이 필요합니다".to_string())?;
    if at_tick == 0 || at_tick > ticks {
        return Err(format!(
            "E_ECO_RUNNER_SHOCK at_tick 범위 오류 at_tick={} ticks={}",
            at_tick, ticks
        ));
    }
    let target = shock
        .target
        .as_ref()
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| "E_ECO_RUNNER_SHOCK target이 필요합니다".to_string())?
        .to_string();
    let delta = parse_required_fixed64("E_ECO_RUNNER_SHOCK delta", shock.delta.as_ref())?;
    let scope = parse_shock_scope(shock.scope.as_deref())?;
    Ok(Some(ShockSpec {
        kind: shock.kind.clone(),
        target,
        delta,
        at_tick,
        scope,
    }))
}

fn parse_required_fixed64(prefix: &str, value: Option<&JsonValue>) -> Result<Fixed64, String> {
    let Some(value) = value else {
        return Err(format!("{prefix}가 필요합니다"));
    };
    parse_json_fixed64(prefix, value)
}

fn parse_json_fixed64(prefix: &str, value: &JsonValue) -> Result<Fixed64, String> {
    match value {
        JsonValue::Number(num) => {
            Fixed64::parse_literal(&num.to_string()).ok_or_else(|| format!("{prefix} 파싱 실패"))
        }
        JsonValue::String(text) => {
            Fixed64::parse_literal(text).ok_or_else(|| format!("{prefix} 파싱 실패"))
        }
        _ => Err(format!("{prefix}는 숫자/문자열이어야 합니다")),
    }
}

fn parse_shock_scope(scope: Option<&str>) -> Result<ShockScope, String> {
    let Some(scope) = scope else {
        return Ok(ShockScope::Both);
    };
    match scope.trim() {
        "" | "양쪽" | "both" | "BOTH" | "Both" => Ok(ShockScope::Both),
        "거시" | "macro" | "MACRO" | "Macro" => Ok(ShockScope::MacroOnly),
        "미시" | "micro" | "MICRO" | "Micro" => Ok(ShockScope::MicroOnly),
        other => Err(format!(
            "E_ECO_RUNNER_SHOCK scope 지원값이 아닙니다: {}",
            other
        )),
    }
}

fn fixed_from_state_key(state: &State, key: &str) -> Option<Fixed64> {
    let value = state.get(&Key::new(key))?;
    match value {
        Value::Num(quantity) => Some(quantity.raw),
        _ => None,
    }
}

fn resolve_network_income(state: &State) -> Result<Fixed64, String> {
    if let Some(value) = fixed_from_state_key(state, "총수입") {
        return Ok(value);
    }
    let wage = fixed_from_state_key(state, "임금")
        .ok_or_else(|| "E_ECO_NETWORK_FLOW 총수입 계산 키 누락(임금)".to_string())?;
    let transfer = fixed_from_state_key(state, "이전소득")
        .ok_or_else(|| "E_ECO_NETWORK_FLOW 총수입 계산 키 누락(이전소득)".to_string())?;
    Ok(wage.saturating_add(transfer))
}

fn resolve_network_spending(state: &State) -> Result<Fixed64, String> {
    if let Some(value) = fixed_from_state_key(state, "총지출") {
        return Ok(value);
    }
    let consume = fixed_from_state_key(state, "소비")
        .ok_or_else(|| "E_ECO_NETWORK_FLOW 총지출 계산 키 누락(소비)".to_string())?;
    let tax = fixed_from_state_key(state, "세금")
        .ok_or_else(|| "E_ECO_NETWORK_FLOW 총지출 계산 키 누락(세금)".to_string())?;
    Ok(consume.saturating_add(tax))
}

fn extract_wealth(state: &State) -> Option<Vec<Fixed64>> {
    let value = state.get(&Key::new("부목록"))?;
    let Value::List(list) = value else {
        return None;
    };
    let mut out = Vec::with_capacity(list.items.len());
    for item in &list.items {
        if let Some(number) = extract_wealth_item(item) {
            out.push(number);
        } else {
            return None;
        }
    }
    Some(out)
}

fn extract_wealth_item(value: &Value) -> Option<Fixed64> {
    match value {
        Value::Num(quantity) => Some(quantity.raw),
        Value::Pack(pack) => match pack.fields.get("부") {
            Some(Value::Num(quantity)) => Some(quantity.raw),
            _ => None,
        },
        _ => None,
    }
}

fn eco_mean(values: &[Fixed64]) -> Fixed64 {
    if values.is_empty() {
        return Fixed64::zero();
    }
    let mut sum: i128 = 0;
    for value in values {
        sum += value.raw() as i128;
    }
    let raw = sum / values.len() as i128;
    Fixed64::from_raw(saturating_i128_to_i64(raw))
}

fn eco_max(values: &[Fixed64]) -> Fixed64 {
    values
        .iter()
        .copied()
        .max_by_key(|value| value.raw())
        .unwrap_or_else(Fixed64::zero)
}

fn eco_gini(values: &[Fixed64]) -> Fixed64 {
    if values.is_empty() {
        return Fixed64::zero();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by_key(|value| value.raw());
    let n = sorted.len() as i128;
    let mut sum_raw: i128 = 0;
    let mut weighted_raw: i128 = 0;
    for (idx, value) in sorted.iter().enumerate() {
        let raw = value.raw() as i128;
        sum_raw += raw;
        weighted_raw += (idx as i128 + 1) * raw;
    }
    if sum_raw <= 0 {
        return Fixed64::zero();
    }
    let scale = Fixed64::SCALE as i128;
    let term1 = ((2_i128 * weighted_raw) << Fixed64::SCALE_BITS) / (n * sum_raw);
    let term2 = ((n + 1) * scale) / n;
    let mut out = term1 - term2;
    if out < 0 {
        out = 0;
    }
    if out > scale {
        out = scale;
    }
    Fixed64::from_raw(saturating_i128_to_i64(out))
}

fn eco_quantile_linear(values: &[Fixed64], p: Fixed64) -> Fixed64 {
    if values.is_empty() {
        return Fixed64::zero();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by_key(|value| value.raw());
    if p.raw() <= 0 {
        return sorted[0];
    }
    let one = Fixed64::one();
    if p.raw() >= one.raw() {
        return *sorted.last().unwrap_or(&sorted[0]);
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let n_minus_1 = (sorted.len() - 1) as i128;
    let pos_raw = (p.raw() as i128) * n_minus_1;
    let lower = (pos_raw >> Fixed64::SCALE_BITS) as usize;
    let frac_raw = pos_raw & ((Fixed64::SCALE as i128) - 1);
    if lower + 1 >= sorted.len() || frac_raw == 0 {
        return sorted[lower];
    }
    let lower_value = sorted[lower];
    let upper_value = sorted[lower + 1];
    let diff = upper_value.saturating_sub(lower_value);
    let weight = Fixed64::from_raw(saturating_i128_to_i64(frac_raw));
    lower_value.saturating_add(diff.saturating_mul(weight))
}

fn saturating_i128_to_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

fn fixed_literal(value: &str) -> Fixed64 {
    Fixed64::parse_literal(value).unwrap_or_else(Fixed64::zero)
}

fn ensure_parent_dir(path: &Path, code: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{code} {} {}", parent.display(), e))?;
    }
    Ok(())
}

fn resolve_model_path(base_dir: &Path, model: &str) -> PathBuf {
    let path = Path::new(model);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

fn resolve_report_path(input: &Path, report_path: Option<&str>, out: Option<&Path>) -> PathBuf {
    if let Some(path) = out {
        return path.to_path_buf();
    }
    if let Some(path) = report_path {
        let path_obj = Path::new(path);
        if path_obj.is_absolute() {
            return path_obj.to_path_buf();
        }
        let base_dir = input.parent().unwrap_or_else(|| Path::new("."));
        return base_dir.join(path_obj);
    }
    paths::build_dir()
        .join("eco")
        .join("macro_micro_runner.report.detjson")
}

fn resolve_network_flow_report_path(out: Option<&Path>) -> PathBuf {
    if let Some(path) = out {
        return path.to_path_buf();
    }
    paths::build_dir()
        .join("eco")
        .join("network_flow.report.detjson")
}

fn resolve_abm_spatial_report_path(out: Option<&Path>) -> PathBuf {
    if let Some(path) = out {
        return path.to_path_buf();
    }
    paths::build_dir()
        .join("eco")
        .join("abm_spatial.report.detjson")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::{Key, State};
    use crate::core::unit::UnitDim;
    use crate::core::value::{ListValue, PackValue, Quantity};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ddn_eco_runner_{}_{}", name, stamp));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn macro_micro_runner_report_is_deterministic() {
        let dir = temp_dir("deterministic");
        let macro_model = dir.join("macro.ddn");
        let micro_model = dir.join("micro.ddn");
        fs::write(
            &macro_model,
            "(시작)할때 { 세율 <- 0. }.\n(매마디)마다 { 균형가격 <- (100 + 세율 * 50). }.\n",
        )
        .expect("write macro");
        fs::write(
            &micro_model,
            "(시작)할때 { 세율 <- 0. }.\n(매마디)마다 { 평균가격 <- (100 + 세율 * 200). }.\n",
        )
        .expect("write micro");
        let input = dir.join("runner.json");
        let out = dir.join("runner.report.detjson");
        let spec = serde_json::json!({
            "schema": "ddn.macro_micro_runner.v0",
            "seed": 42,
            "ticks": 4,
            "shock": {
                "type": "세율_인상",
                "target": "세율",
                "delta": 0.1,
                "at_tick": 3,
                "scope": "양쪽"
            },
            "models": {
                "거시": macro_model.to_string_lossy(),
                "미시": micro_model.to_string_lossy()
            },
            "diagnostics": [
                {
                    "name": "거시↔미시 균형가격",
                    "lhs": "거시.균형가격",
                    "rhs": "미시.평균가격",
                    "threshold": 5.0
                }
            ]
        });
        fs::write(&input, serde_json::to_string(&spec).expect("spec")).expect("write input");

        run_macro_micro(&input, Some(&out)).expect("run 1");
        let first = fs::read_to_string(&out).expect("read 1");
        run_macro_micro(&input, Some(&out)).expect("run 2");
        let second = fs::read_to_string(&out).expect("read 2");
        assert_eq!(first, second, "report must be deterministic");

        let doc: JsonValue = serde_json::from_str(&first).expect("report json");
        assert_eq!(
            doc.get("schema").and_then(|v| v.as_str()),
            Some("ddn.runner_report.v0")
        );
        let results = doc
            .get("results")
            .and_then(|v| v.as_array())
            .expect("results");
        assert_eq!(results.len(), 1);
        let row = &results[0];
        assert_eq!(
            row.get("error_code").and_then(|v| v.as_str()),
            Some("E_ECO_DIVERGENCE_DETECTED")
        );
        assert_eq!(row.get("divergence_tick").and_then(|v| v.as_u64()), Some(3));
        assert_eq!(
            doc.get("shock_target").and_then(|v| v.as_str()),
            Some("세율")
        );
        assert_eq!(
            doc.get("shock_scope").and_then(|v| v.as_str()),
            Some("양쪽")
        );
    }

    #[test]
    fn macro_micro_runner_rejects_invalid_shock_input() {
        let shock = RunnerShock {
            kind: Some("세율_인상".to_string()),
            target: None,
            delta: Some(JsonValue::Number(1.into())),
            at_tick: Some(2),
            scope: Some("양쪽".to_string()),
        };
        let err = parse_shock(Some(&shock), 4).expect_err("must fail");
        assert!(err.contains("target이 필요합니다"));
    }

    #[test]
    fn macro_micro_runner_rejects_invalid_scope_value() {
        let shock = RunnerShock {
            kind: Some("세율_인상".to_string()),
            target: Some("세율".to_string()),
            delta: Some(JsonValue::Number(1.into())),
            at_tick: Some(2),
            scope: Some("invalid_scope".to_string()),
        };
        let err = parse_shock(Some(&shock), 4).expect_err("must fail");
        assert!(err.contains("E_ECO_RUNNER_SHOCK"));
    }

    #[test]
    fn network_flow_report_detects_violation() {
        let dir = temp_dir("network_violation");
        let input = dir.join("model.ddn");
        let out = dir.join("report.detjson");
        fs::write(
            &input,
            "(매마디)마다 {\n  임금 <- 70.\n  이전소득 <- 10.\n  소비 <- 65.\n  세금 <- 10.\n}.\n",
        )
        .expect("write model");
        let threshold = Fixed64::parse_literal("0.01").expect("threshold");
        let err = run_network_flow(&input, 1, 0, threshold, Some(&out)).expect_err("must diverge");
        assert_eq!(err, "E_SFC_IDENTITY_VIOLATION");
        let report: JsonValue =
            serde_json::from_str(&fs::read_to_string(&out).expect("read report")).expect("json");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.eco.network_flow_report.v0")
        );
        assert_eq!(report.get("result").and_then(|v| v.as_str()), Some("발산"));
        assert_eq!(
            report.get("error_code").and_then(|v| v.as_str()),
            Some("E_SFC_IDENTITY_VIOLATION")
        );
    }

    #[test]
    fn network_flow_report_passes_when_balanced() {
        let dir = temp_dir("network_balanced");
        let input = dir.join("model.ddn");
        let out = dir.join("report.detjson");
        fs::write(
            &input,
            "(매마디)마다 {\n  총수입 <- 100.\n  총지출 <- 100.\n}.\n",
        )
        .expect("write model");
        run_network_flow(
            &input,
            1,
            0,
            Fixed64::parse_literal("0").expect("zero"),
            Some(&out),
        )
        .expect("must converge");
        let report: JsonValue =
            serde_json::from_str(&fs::read_to_string(&out).expect("read report")).expect("json");
        assert_eq!(report.get("result").and_then(|v| v.as_str()), Some("수렴"));
        assert!(report.get("error_code").is_none());
    }

    #[test]
    fn abm_spatial_report_uses_existing_metrics() {
        let dir = temp_dir("abm_existing_metrics");
        let input = dir.join("model.ddn");
        let out = dir.join("report.detjson");
        fs::write(
            &input,
            "(매마디)마다 {\n  지니계수 <- 0.4.\n  평균부 <- 50.\n  최대부 <- 70.\n  분위90 <- 65.\n  부목록 <- [1, 1, 1].\n}.\n",
        )
        .expect("write model");
        run_abm_spatial(&input, 1, 0, Some(&out)).expect("abm run");
        let report: JsonValue =
            serde_json::from_str(&fs::read_to_string(&out).expect("read report")).expect("json");
        assert_eq!(
            report.get("schema").and_then(|v| v.as_str()),
            Some("ddn.eco.abm_spatial_report.v0")
        );
        let expected_gini = Fixed64::parse_literal("0.4").expect("literal").format();
        assert_eq!(
            report.get("gini").and_then(|v| v.as_str()),
            Some(expected_gini.as_str())
        );
        assert_eq!(
            report.get("mean_wealth").and_then(|v| v.as_str()),
            Some("50")
        );
        assert_eq!(
            report.get("max_wealth").and_then(|v| v.as_str()),
            Some("70")
        );
        assert_eq!(
            report.get("p90_wealth").and_then(|v| v.as_str()),
            Some("65")
        );
        assert_eq!(report.get("agent_count").and_then(|v| v.as_u64()), Some(3));
    }

    #[test]
    fn abm_spatial_report_computes_from_wealth_list() {
        let dir = temp_dir("abm_from_wealth");
        let input = dir.join("model.ddn");
        let out = dir.join("report.detjson");
        fs::write(&input, "(매마디)마다 { 부목록 <- [1, 1, 1, 1]. }.\n").expect("write model");
        run_abm_spatial(&input, 1, 0, Some(&out)).expect("abm run");
        let report: JsonValue =
            serde_json::from_str(&fs::read_to_string(&out).expect("read report")).expect("json");
        assert_eq!(report.get("gini").and_then(|v| v.as_str()), Some("0"));
        assert_eq!(
            report.get("mean_wealth").and_then(|v| v.as_str()),
            Some("1")
        );
        assert_eq!(report.get("max_wealth").and_then(|v| v.as_str()), Some("1"));
        assert_eq!(report.get("p90_wealth").and_then(|v| v.as_str()), Some("1"));
        assert_eq!(report.get("agent_count").and_then(|v| v.as_u64()), Some(4));
    }

    #[test]
    fn extract_wealth_supports_pack_items_with_bu_field() {
        let mut fields1 = BTreeMap::new();
        fields1.insert(
            "부".to_string(),
            Value::Num(Quantity::new(
                Fixed64::parse_literal("10").expect("num"),
                UnitDim::zero(),
            )),
        );
        let mut fields2 = BTreeMap::new();
        fields2.insert(
            "부".to_string(),
            Value::Num(Quantity::new(
                Fixed64::parse_literal("20").expect("num"),
                UnitDim::zero(),
            )),
        );
        let list = ListValue {
            items: vec![
                Value::Pack(PackValue { fields: fields1 }),
                Value::Pack(PackValue { fields: fields2 }),
            ],
        };
        let mut state = State::new();
        state.set(Key::new("부목록"), Value::List(list));

        let wealth = extract_wealth(&state).expect("wealth list");
        assert_eq!(wealth.len(), 2);
        assert_eq!(wealth[0], Fixed64::parse_literal("10").expect("num"));
        assert_eq!(wealth[1], Fixed64::parse_literal("20").expect("num"));
    }
}
