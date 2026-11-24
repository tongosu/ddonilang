//! Explicit workbench B adapter. The equation and proof remain actual source;
//! no second language parser, proof issuance, or persistence format.
use ddonirang_lang::Span;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ByteSpan {
    start: usize,
    end: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    source_sha256: String,
    source_path: String,
    definition_span: ByteSpan,
    left_span: ByteSpan,
    right_span: ByteSpan,
}
fn source_hash(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))
}

pub fn targets(source: &str, path: &str) -> Value {
    if source.len() > 1_000_000 {
        return error(
            source,
            path,
            "E_PROOF_BUDGET",
            "B 입력 자원 한도를 넘었습니다",
            Span::new(0, 0),
        );
    }
    crate::proof_workflow::observe_linear_targets(source, path)
}
fn error(source: &str, path: &str, code: &str, message: &str, span: Span) -> Value {
    let e = ddonirang_lang::ParseError {
        span,
        message: format!("{code}: {message}"),
    };
    json!({"observation_only":true,"unconditional_proof_issued":false,"source_path":path,
        "source_sha256":source_hash(source),"source_bytes":source.len(),"status":"rejected","phase":"analysis",
        "definitions":[],"diagnostic":{"code":code,"message":e.message,"span":{"start":span.start,"end":span.end},
            "formatted":crate::proof_workflow::format_source_error(source,path,&e)}})
}
fn valid_span(source: &str, s: &ByteSpan) -> bool {
    s.start < s.end
        && s.end <= source.len()
        && source.is_char_boundary(s.start)
        && source.is_char_boundary(s.end)
}
/// Same existing observation report. Informational calculation text is kept in
/// the existing diagnostic slot; definition/step verdicts remain the checker’s.
pub fn observe(source: &str, path: &str, target_json: &str) -> Value {
    let zero = Span::new(0, 0);
    if source.len() > 1_000_000 || target_json.len() > 16384 {
        return error(
            source,
            path,
            "E_PROOF_BUDGET",
            "B 입력 자원 한도를 넘었습니다",
            zero,
        );
    }
    let target: Target = match serde_json::from_str(target_json) {
        Ok(v) => v,
        Err(_) => {
            return error(
                source,
                path,
                "E_B_LINEAR_TARGET",
                "정확한 정의·초기 가정 위치와 원문 판본이 필요합니다",
                zero,
            )
        }
    };
    if target.source_path != path || target.source_sha256 != source_hash(source) {
        return error(
            source,
            path,
            "E_B_LINEAR_STALE",
            "대상 파일 또는 원문 판본이 바뀌었습니다",
            zero,
        );
    }
    if !valid_span(source, &target.definition_span)
        || !valid_span(source, &target.left_span)
        || !valid_span(source, &target.right_span)
        || target.left_span.start < target.definition_span.start
        || target.right_span.end > target.definition_span.end
        || target.left_span.end > target.right_span.start
    {
        return error(
            source,
            path,
            "E_B_LINEAR_TARGET",
            "UTF-8 byte 범위 또는 대상 정의 결속이 다릅니다",
            zero,
        );
    }
    let d = Span::new(target.definition_span.start, target.definition_span.end);
    let resolved = crate::proof_workflow::selected_linear_problem_terms(
        source,
        path,
        &target.source_sha256,
        d,
        Span::new(target.left_span.start, target.left_span.end),
        Span::new(target.right_span.start, target.right_span.end),
    );
    let (definition, left, right) = match resolved {
        Ok(v) => v,
        Err(e) => {
            let code = e
                .message
                .split(':')
                .next()
                .filter(|s| s.starts_with("E_"))
                .unwrap_or("E_B_LINEAR_TARGET");
            return error(source, path, code, &e.message, e.span);
        }
    };
    use ddonirang_symbolic::{LinearProblemAdmission, RelationSolveOutcome};
    match ddonirang_symbolic::admit_linear_problem(&left, &right) {
        Ok(LinearProblemAdmission::Admitted) => {}
        Ok(LinearProblemAdmission::OutsideFamily(reason)) => {
            return error(
                source,
                path,
                "E_B_LINEAR_OUTSIDE_FAMILY",
                &format!("이 일차 학습 문제군 밖입니다: {reason}"),
                d,
            )
        }
        Err(message) => return error(source, path, "E_B_LINEAR_INPUT", &message, d),
    }
    let solved = match ddonirang_symbolic::solve_relation_equation(&left, &right) {
        Ok(v) => v,
        Err(e) => return error(source, path, "E_B_LINEAR_CALCULATION", &e, d),
    };
    let calculation = match solved {
        RelationSolveOutcome::Solution(bindings) => {
            let checked = match ddonirang_symbolic::relation_holds(&left, &right, &bindings) {
                Ok(v) => v,
                Err(e) => return error(source, path, "E_B_LINEAR_CALCULATION", &e, d),
            };
            let values = bindings
                .iter()
                .map(|(name, b)| {
                    let display = name
                        .strip_prefix('p')
                        .and_then(|s| s.parse::<usize>().ok())
                        .and_then(|i| definition.parameters().get(i))
                        .map(|p| p.pin_name.as_str())
                        .unwrap_or(name);
                    format!("{display} = {}/{}", b.numerator, b.denominator)
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "선택 등식의 답: {values}\n원래 선택 등식에 정확 유리수 대입 검산: {}",
                if checked { "참" } else { "거짓" }
            )
        }
        RelationSolveOutcome::NoSolution => "해 없음: 선택한 등식을 만족하는 값이 없습니다.\n검산: 해당 없음 · 정규화한 모순 관계".into(),
        RelationSolveOutcome::NonUnique => {
            "항등 · 해가 유일하지 않음: 선택한 등식은 항등이므로 값이 하나로 정해지지 않습니다.\n검산: 정규화한 항등 관계".into()
        }
    };
    let selected = Span::new(target.left_span.start, target.right_span.end);
    let equation = &source[selected.start..selected.end];
    let name = &definition.declaration().canonical_name;
    let info = format!("B 일차 문제군: 수용\n선택한 초기 등식의 계산 결과\n정의: {name}\n원문: {path} · UTF-8 {}..{}\n선택 등식: {equation}\n{calculation}\n이 계산은 선택 등식에 한정합니다. 모든 가정을 함께 만족하는 해나 가정의 일관성을 검사한 결과가 아닙니다.\n단계 검사는 주어진 가정 아래의 규칙을 확인하며, 가정의 참·해 존재·무가정 증명 발급을 보장하지 않습니다.", selected.start, selected.end);
    let mut report = crate::proof_workflow::observe_selected_algebra(source, path, definition);
    if report["diagnostic"].is_null() {
        report["diagnostic"] = json!({"code":"I_B_LINEAR_RESULT","message":info,"span":{"start":selected.start,"end":selected.end},"formatted":info});
    } else {
        let prior = report["diagnostic"]["formatted"]
            .as_str()
            .unwrap_or("")
            .to_string();
        report["diagnostic"]["formatted"] = json!(format!("{info}\n\n단계 검사 거부:\n{prior}"));
    }
    report
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn wasm_observe_linear_targets_json(source: &str, source_path: &str) -> String {
    targets(source, source_path).to_string()
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn wasm_observe_linear_problem_json(
    source: &str,
    source_path: &str,
    target_json: &str,
) -> String {
    observe(source, source_path, target_json).to_string()
}
