pub(crate) mod assumption_observation;
mod conditional_algebra;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use crate::artifact_output::write_text_artifact_atomic;

/// Shared source path for the CLI and other product consumers. Acceptance is
/// a conditional definition check, never an unconditional proof of its goal.
pub fn verify_assumption_source(
    source: &str,
    path: &str,
) -> Result<Vec<ddonirang_proof::CheckedAssumptionTransfer>, ddonirang_lang::ParseError> {
    let prepared=ddonirang_lang::proof_authoring::prepare_source(source,path)?;
    verify_prepared_assumption(prepared, source.len())
}

fn verify_prepared_assumption(prepared:ddonirang_lang::proof_authoring::PreparedProofSource, source_len:usize)
    -> Result<Vec<ddonirang_proof::CheckedAssumptionTransfer>,ddonirang_lang::ParseError> {
    resolve_prepared_assumption(prepared, source_len)?
        .iter().map(ddonirang_proof::check_assumption_transfer).collect()
}

fn resolve_prepared_assumption(prepared:ddonirang_lang::proof_authoring::PreparedProofSource, source_len:usize)
    -> Result<Vec<ddonirang_lang::proof_authoring::ResolvedTransfer>,ddonirang_lang::ParseError> {
    let resolved=if prepared.has_coordinate_contract() {
        let candidate=crate::numeric_constraint_plan::coordinate_proof_candidate(&prepared)
            .map_err(|(message,span)|ddonirang_lang::ParseError {span:span.unwrap_or(ddonirang_lang::Span::new(0,source_len)),message})?;
        prepared.resolve_with_numeric_candidate(&candidate)?
    }else{prepared.resolve()?};
    Ok(resolved)
}


/// Checked definitions from one common source frontdoor; no JSON import path.
#[derive(Clone,Debug)]
pub enum CheckedSourceDefinition {
    Assumption(ddonirang_proof::CheckedAssumptionTransfer),
    Algebra(ddonirang_proof::algebra::CheckedAlgebraDefinition),
    ConditionalAlgebra(ddonirang_proof::conditional_algebra::CheckedDefinition),
}
impl CheckedSourceDefinition {
    pub fn validate_source(&self,source:&str,path:&str)->Result<(),ddonirang_lang::ParseError> {
        match self {Self::Assumption(c)=>c.validate_source(source,path),Self::Algebra(c)=>c.validate_source(source,path),Self::ConditionalAlgebra(c)=>c.validate_source(source,path)}
    }
}
pub fn verify_conditional_source(source:&str,path:&str)->Result<Vec<CheckedSourceDefinition>,ddonirang_lang::ParseError> {
    let prepared=ddonirang_lang::proof_authoring::prepare_source(source,path)?;
    if prepared.has_conditional_algebra_contract() {
        let mut checked:Vec<_>=prepared.resolve_conditional_algebra()?.iter().map(|d|ddonirang_proof::conditional_algebra::check_definition(d).map(CheckedSourceDefinition::ConditionalAlgebra)).collect::<Result<_,_>>()?;
        checked.extend(prepared.resolve_other_project_algebra()?.iter().map(|d|ddonirang_proof::algebra::check_algebra_definition(d).map(CheckedSourceDefinition::Algebra)).collect::<Result<Vec<_>,_>>()?);
        Ok(checked)
    } else if prepared.has_algebra_contract() {
        prepared.resolve_algebra()?.iter().map(|d|ddonirang_proof::algebra::check_algebra_definition(d).map(CheckedSourceDefinition::Algebra)).collect()
    } else {
        verify_prepared_assumption(prepared, source.len()).map(|items|items.into_iter().map(CheckedSourceDefinition::Assumption).collect())
    }
}

/// Presentation only; preserve the diagnostic identity and common coordinates.
pub(crate) fn format_source_error(source: &str, path: &str, error: &ddonirang_lang::ParseError) -> String {
    let mut message = format!("{}:{}..{} {}", path, error.span.start, error.span.end, error.message);
    let range = crate::lsp::ByteRange { start: error.span.start, end: error.span.end };
    let Ok(location) = crate::lsp::byte_range_to_lsp_range(source, range, crate::lsp::PositionEncoding::Utf32) else { return message };
    let Some(fragment) = source.get(error.span.start..error.span.end) else { return message };
    let mut chars = fragment.chars();
    let excerpt: String = chars.by_ref().take(120).flat_map(char::escape_debug).collect();
    let suffix = if chars.next().is_some() { "…" } else { "" };
    let excerpt = if fragment.is_empty() { "(빈 범위)" } else { &excerpt };
    message.push_str(&format!("\n위치: {}줄 {}열부터 (1부터 세는 유니코드 문자 기준)\n해당 원문: {}{}", location.start.line + 1, location.start.character + 1, excerpt, suffix));
    message
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_verify(input: &Path, out: Option<&Path>) -> Result<(), String> {
    if input.is_dir() {
        if out.is_some(){return Err("E_PROOF_SOURCE_OUTPUT_UNSUPPORTED: 원문 보임 검사의 --out은 아직 지원하지 않습니다".into());}
        let source=crate::local_project::build_virtual_project_source_from_disk(&input.join("root.ddn"),None)
            .map_err(|e|format!("{}: {}",e.code,e.message))?;
        let count=verify_project_source(source)?;
        println!("가정 의존 정의 검사 수용: {}개; 무가정 증명 발급 아님",count);
        return Ok(());
    }

    if matches!(input.extension().and_then(|ext| ext.to_str()),Some("ddn"|"또니")) {
        if out.is_some() {
            return Err("E_PROOF_SOURCE_OUTPUT_UNSUPPORTED: 원문 보임 검사의 --out은 아직 지원하지 않습니다".into());
        }
        let source = fs::read_to_string(input).map_err(|e| e.to_string())?;
        let path = input.to_string_lossy();
        let checked = verify_conditional_source(&source, &path)
            .map_err(|error| format_source_error(&source, &path, &error))?;
        println!("가정 의존 정의 검사 수용: {}개; 무가정 증명 발급 아님", checked.len());
        return Ok(());
    }
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let report = ddonirang_proof::verify_json_text(&text)?;
    write_report(&report, out)?;
    println!("proof_valid={} kind={}", report.valid, report.kind);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_replay(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let report = ddonirang_proof::verify_json_text(&text)?;
    write_report(&report, out)?;
    println!("proof_replay_valid={} kind={}", report.valid, report.kind);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_prove_symbolic_eq(lhs: &str, rhs: &str, out: Option<&Path>) -> Result<(), String> {
    let cert = ddonirang_symbolic::prove_equivalent(lhs, rhs)?;
    if !cert.equivalent {
        return Err("E_PROOF_EQUIVALENCE_NOT_PROVEN_INTERNAL".to_string());
    }
    let value = serde_json::to_value(&cert).map_err(|e| e.to_string())?;
    let report = ddonirang_proof::verify_value(&value)?;
    if !report.valid {
        return Err("E_PROOF_CHECK_FAILED_INTERNAL".to_string());
    }
    if let Some(path) = out {
        write_text_artifact_atomic(path, &ddonirang_symbolic::to_detjson(&cert)?)?;
    }
    println!(
        "proof_tactic_ok=true kind={} equivalent=true",
        report.kind
    );
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_rewrite_chain(from: &str, to: &str, out: Option<&Path>) -> Result<(), String> {
    let proof = serde_json::json!({
        "schema": "ddn.proof.symbolic_rewrite.v1",
        "steps": [{"from": from, "to": to}]
    });
    let report = ddonirang_proof::verify_value(&proof)?;
    if !report.valid {
        return Err("E_PROOF_REWRITE_NOT_PROVEN_INTERNAL".to_string());
    }
    if let Some(path) = out {
        write_text_artifact_atomic(
            path,
            &serde_json::to_string_pretty(&proof).map_err(|e| e.to_string())?,
        )?;
    }
    println!("proof_rewrite_ok=true kind={}", report.kind);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_prove_relation_eq(
    first_lhs: &str,
    first_rhs: &str,
    second_lhs: &str,
    second_rhs: &str,
    out: Option<&Path>,
) -> Result<(), String> {
    let cert = ddonirang_symbolic::prove_relation_equivalent(
        first_lhs, first_rhs, second_lhs, second_rhs,
    )?;
    if !cert.equivalent {
        return Err("E_PROOF_RELATION_EQUIVALENCE_NOT_PROVEN_INTERNAL".to_string());
    }
    let value = serde_json::to_value(&cert).map_err(|e| e.to_string())?;
    let report = ddonirang_proof::verify_value(&value)?;
    if !report.valid {
        return Err("E_PROOF_CHECK_FAILED_INTERNAL".to_string());
    }
    if let Some(path) = out {
        write_text_artifact_atomic(path, &ddonirang_symbolic::to_detjson(&cert)?)?;
    }
    println!(
        "proof_relation_ok=true kind={} equivalent=true",
        report.kind
    );
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn write_report(report: &ddonirang_proof::VerifyReport, out: Option<&Path>) -> Result<(), String> {
    if let Some(path) = out {
        write_text_artifact_atomic(path, &ddonirang_proof::to_detjson(report)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod coordinate_source_tests {
const SOURCE:&str=r###"(점:(바른수,바른수)~을) 직각회전하다{직각회전하기}:(바른수,바른수):셈씨 := {
  직각회전하다 <- (0 - 점[1], 점[0]).
}.

(점:(바른수,바른수)~을) 평행이동하다{평행이동하기}:(바른수,바른수):셈씨 := {
  평행이동하다 <- (점[0] + 10, 점[1] + 2).
}.

(점:(바른수,바른수)~을) 회전뒤이동하다{회전뒤이동하기}:(바른수,바른수):셈씨 := {
  회전뒤이동하다 <- (점=((점=점) 직각회전하기)) 평행이동하기.
}.

좌표가정 := (회전=직각회전하다, 이동=평행이동하다, 합성=회전뒤이동하다, 입력=(1,0)) 좌표합성가정짜임.

(
  우회전근거:(좌표가정.비교회전성공)보임~로,
  우가로근거:(좌표가정.비교가로성공)보임~로,
  우세로근거:(좌표가정.비교세로성공)보임~로,
  좌회전근거:(좌표가정.합성회전성공)보임~로,
  좌가로근거:(좌표가정.합성가로성공)보임~로,
  좌세로근거:(좌표가정.합성세로성공)보임~로,
  반환근거:(좌표가정.합성반환성공)보임~로,
  법칙근거:(좌표가정.적용합성법칙)보임~로
) 법칙전달하다{법칙전달하기}:(좌표가정.적용합성법칙)보임:셈씨 := {
  법칙전달하다 <- 법칙근거.
}.

(
  우회전근거:(좌표가정.비교회전성공)보임~로,
  우가로근거:(좌표가정.비교가로성공)보임~로,
  우세로근거:(좌표가정.비교세로성공)보임~로,
  좌회전근거:(좌표가정.합성회전성공)보임~로,
  좌가로근거:(좌표가정.합성가로성공)보임~로,
  좌세로근거:(좌표가정.합성세로성공)보임~로,
  반환근거:(좌표가정.합성반환성공)보임~로,
  법칙근거:(좌표가정.적용합성법칙)보임~로
) 근거로계산하다{근거로계산하기}:(바른수,바른수):셈씨 := {
  근거로계산하다 <- (점=(1,0)) 회전뒤이동하기.
}.
"###;

#[test]
fn coordinate_original_frontdoor() {
    let checked=super::verify_assumption_source(SOURCE,"좌표.ddn").unwrap();
    assert_eq!(checked.len(),2);
    for c in &checked {c.validate_source(SOURCE,"좌표.ddn").unwrap();}
    assert!(checked[1].resolved().coordinate_calculation().is_some());
    let changed=SOURCE.replace("점[0] + 10", "점[0] + 11");
    for c in &checked {
        assert!(c.validate_source(&changed,"좌표.ddn").is_err());
        assert!(c.validate_source(SOURCE,"다른좌표.ddn").is_err());
    }
}

#[test]
fn coordinate_numeric_candidate_is_not_a_success_receipt() {
    use ddonirang_lang::proof_authoring::prepare_source;
    use ddonirang_symbolic::typed_math::NumericRole;
    let prepared=prepare_source(SOURCE,"좌표.ddn").unwrap();
    let mut candidate=crate::numeric_constraint_plan::coordinate_proof_candidate(&prepared).unwrap();
    let original=candidate.operations[0];
    candidate.operations[0].1=NumericRole::CheckedDimensionlessIntegerAdd;
    assert!(prepared.resolve_with_numeric_candidate(&candidate).unwrap_err().message.contains("E_PROOF_NUMERIC_CONTRACT"));
    candidate.operations[0]=original;
    candidate.operations.push(original);
    assert!(prepared.resolve_with_numeric_candidate(&candidate).unwrap_err().message.contains("E_PROOF_NUMERIC_CONTRACT"));
}
}

#[cfg(test)]
mod proof_error_presentation_tests {
    use super::*;
    use ddonirang_lang::{ParseError, Span};
    #[test]
    fn unicode_crlf_location_keeps_original_diagnostic() {
        let source="// 🍎\r\n한글 🍎 오른=5.\r\n";
        let start=source.find('5').unwrap();
        let error=ParseError{span:Span::new(start,start+1),message:"E_PROOF_ALGEBRA_REWRITE: 오른쪽 오류".into()};
        let shown=format_source_error(source,"학습.ddn",&error);
        assert!(shown.starts_with(&format!("학습.ddn:{start}..{} E_PROOF_ALGEBRA_REWRITE: 오른쪽 오류",start+1)));
        assert!(shown.contains("위치: 2줄 9열부터"),"{shown}");
        assert!(shown.ends_with("해당 원문: 5"));
    }
    #[test]
    fn multiline_control_and_long_spans_are_bounded() {
        let source="가\n\t\u{1b}".to_string()+&"나".repeat(300);
        let error=ParseError{span:Span::new(0,source.len()),message:"E_PROOF_RESULT: 오류".into()};
        let shown=format_source_error(&source,"학습.ddn",&error);
        assert!(shown.contains(r"가\n\t\u{1b}"));
        assert!(shown.ends_with('…'));
        assert_eq!(shown.lines().count(),3);
    }
    #[test]
    fn invalid_utf8_range_is_not_relocated_and_eof_is_explicit() {
        let source="한\n";
        for span in [Span::new(1,2),Span::new(4,5),Span::new(3,0)] {
            let error=ParseError{span,message:"E_PARSE: 오류".into()};
            let shown=format_source_error(source,"학습.ddn",&error);
            assert!(!shown.contains("위치:"));
        }
        let error=ParseError{span:Span::new(4,4),message:"E_PARSE: 끝".into()};
        let shown=format_source_error(source,"학습.ddn",&error);
        assert!(shown.contains("위치: 2줄 1열부터"));
        assert!(shown.ends_with("해당 원문: (빈 범위)"));
    }
}


/// Opt-in, non-authoritative projection of the same source resolver and replay.
/// Never accepted as an input to any proof or saved proof constructor.
pub fn observe_algebra_source(source:&str,path:&str)->serde_json::Value {
    let prepared=ddonirang_lang::proof_authoring::prepare_source(source,path);
    if prepared.as_ref().is_ok_and(|p|p.has_conditional_algebra_contract()) {
        return conditional_algebra::observe(source,path,prepared.and_then(|p|p.resolve_conditional_algebra()));
    }
    let resolved=prepared.and_then(|p|p.resolve_algebra());
    observe_resolved_algebra(source,path,resolved)
}

/// B-only selection boundary. Transport is intentionally not exposed yet.
pub(crate) fn admit_selected_linear_problem(
    source: &str, path: &str, expected_sha256: &str,
    definition_span: ddonirang_lang::Span,
    left_span: ddonirang_lang::Span, right_span: ddonirang_lang::Span,
) -> Result<ddonirang_symbolic::LinearProblemAdmission, ddonirang_lang::ParseError> {
    let (_,left,right)=selected_linear_problem_terms(source,path,expected_sha256,definition_span,left_span,right_span)?;
    ddonirang_symbolic::admit_linear_problem(&left,&right).map_err(|message|ddonirang_lang::ParseError {span:definition_span,message})
}

pub(crate) fn selected_linear_problem_terms(
    source:&str,path:&str,expected_sha256:&str,definition_span:ddonirang_lang::Span,
    left_span:ddonirang_lang::Span,right_span:ddonirang_lang::Span,
)->Result<(ddonirang_lang::algebra_authoring::ResolvedAlgebraDefinition,String,String),ddonirang_lang::ParseError> {
    use ddonirang_symbolic::algebra_term::QTerm;
    use sha2::{Digest, Sha256};
    let error = |message: &str| ddonirang_lang::ParseError {
        span: definition_span, message: message.to_string(),
    };
    if format!("{:x}", Sha256::digest(source.as_bytes())) != expected_sha256 {
        return Err(error("B 문제 선택의 원문 판본이 바뀌었습니다"));
    }
    let definitions = ddonirang_lang::proof_authoring::prepare_source(source,path)?.resolve_project_algebra()?;
    let same_span = |a: ddonirang_lang::Span, b: ddonirang_lang::Span| a.start == b.start && a.end == b.end;
    let selected: Vec<_> = definitions.iter().filter(|d| same_span(d.declaration().span,definition_span)
        && d.source_matches(source,path)).collect();
    if selected.len() != 1 { return Err(error("B 문제의 실제 정의 소유자를 유일하게 확인할 수 없습니다")); }
    let definition = selected[0];
    let assumptions: Vec<_> = definition.assumptions().iter().filter(|a|
        same_span(a.left_span,left_span) && same_span(a.right_span,right_span)).collect();
    if assumptions.len() != 1 { return Err(error("B 문제 선택은 해당 정의의 초기 가정 등식이어야 합니다")); }
    // Slots are supplied by the resolved definition's binder, not source spelling.
    fn expression(term: &QTerm, count: usize) -> Option<String> {
        Some(match term {
            QTerm::Integer(n) => n.to_string(),
            QTerm::Parameter(slot) => { if *slot >= count { return None; } format!("p{slot}") },
            QTerm::Add(a,b) => format!("({}+{})",expression(a,count)?,expression(b,count)?),
            QTerm::Subtract(a,b) => format!("({}-{})",expression(a,count)?,expression(b,count)?),
            QTerm::Multiply(a,b) => format!("({}*{})",expression(a,count)?,expression(b,count)?),
            QTerm::DivideByInteger(a,n) => format!("({}/{n})",expression(a,count)?),
        })
    }
    let assumption = assumptions[0];
    let left = expression(&assumption.left,definition.parameters().len()).ok_or_else(||error("해소된 미지수 소유가 일치하지 않습니다"))?;
    let right = expression(&assumption.right,definition.parameters().len()).ok_or_else(||error("해소된 미지수 소유가 일치하지 않습니다"))?;
    Ok((definition.clone(),left,right))
}

pub(crate) fn observe_selected_algebra(source:&str,path:&str,
    definition:ddonirang_lang::algebra_authoring::ResolvedAlgebraDefinition)->serde_json::Value {
    observe_resolved_algebra(source,path,Ok(vec![definition]))
}

pub(crate) fn observe_linear_targets(source:&str,path:&str)->serde_json::Value {
    // Existing actual-program algebra selection retains source/binder ownership
    // and skips ordinary runtime definitions. It does not validate or execute them.
    let resolved=ddonirang_lang::proof_authoring::prepare_source(source,path)
        .and_then(|p|p.resolve_project_algebra());
    observe_resolved_algebra(source,path,resolved)
}

#[cfg(test)]
mod linear_problem_selection_tests {
    use super::*;
    use sha2::{Digest,Sha256};
    #[test]
    fn selected_assumption_is_source_bound_and_not_a_proof_verdict() {
        let source = "(값:나눔수~을, 근거:(값 * 값 + 값 =:= 값 * 값 + 1)보임~로)\n결론보이다{결론보이기}:(값 =:= 2)보임:셈씨 := { 결론보이다 <- 근거. }.";
        let definitions = ddonirang_lang::proof_authoring::prepare_source(source,"선택.ddn").unwrap().resolve_algebra().unwrap();
        let d = &definitions[0]; let a = &d.assumptions()[0];
        let hash = format!("{:x}",Sha256::digest(source.as_bytes()));
        assert_eq!(admit_selected_linear_problem(source,"선택.ddn",&hash,d.declaration().span,a.left_span,a.right_span).unwrap(),ddonirang_symbolic::LinearProblemAdmission::Admitted);
        assert_eq!(observe_algebra_source(source,"선택.ddn")["status"],"rejected");
        assert!(admit_selected_linear_problem(source,"선택.ddn","old",d.declaration().span,a.left_span,a.right_span).is_err());
        assert!(admit_selected_linear_problem(source,"선택.ddn",&hash,d.declaration().span,a.right_span,a.left_span).is_err());
    }
}
fn observe_resolved_algebra(source:&str,path:&str,resolved:Result<Vec<ddonirang_lang::algebra_authoring::ResolvedAlgebraDefinition>,ddonirang_lang::ParseError>)->serde_json::Value {
    use ddonirang_lang::{algebra_authoring::*,Span};
    use ddonirang_symbolic::algebra_term::QTerm;
    use sha2::{Digest,Sha256};
    use serde_json::{json,Value};
    fn span(s:Span)->Value {json!({"start":s.start,"end":s.end})}
    fn diagnostic(source:&str,path:&str,e:&ddonirang_lang::ParseError)->Value {
        json!({"code":e.message.split(':').next().unwrap_or(""),"message":e.message,"span":span(e.span),"formatted":format_source_error(source,path,e)})
    }
    fn term(t:&QTerm,context:&ResolvedAlgebraDefinition)->Value {
        match t {
            QTerm::Integer(n)=>json!({"kind":"integer","value":n.to_string()}),
            QTerm::Parameter(i)=>json!({"kind":"parameter","slot":i,"name":context.parameters().get(*i).map(|p|&p.pin_name),"pin_id":context.parameters().get(*i).map(|p|format!("{:?}",p.id))}),
            QTerm::Add(a,b)|QTerm::Subtract(a,b)|QTerm::Multiply(a,b)=>json!({"kind":match t{QTerm::Add(..)=>"add",QTerm::Subtract(..)=>"subtract",_=>"multiply"},"left":term(a,context),"right":term(b,context)}),
            QTerm::DivideByInteger(a,n)=>json!({"kind":"divide_by_integer","left":term(a,context),"denominator":n.to_string()}),
        }
    }
    fn equality(eq:&AlgebraEquality,c:&ResolvedAlgebraDefinition)->Value {
        json!({"left":term(&eq.left,c),"right":term(&eq.right,c),"left_span":span(eq.left_span),"right_span":span(eq.right_span)})
    }
    fn reference(r:AlgebraEvidenceRef)->Value {
        match r {AlgebraEvidenceRef::Assumption(i)=>json!({"kind":"assumption","index":i}),AlgebraEvidenceRef::Step(i)=>json!({"kind":"step","index":i})}
    }
    let mut report=json!({"observation_only":true,"unconditional_proof_issued":false,"source_path":path,
        "source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"source_bytes":source.len(),
        "status":"not_checked","phase":"analysis","definitions":[],"diagnostic":null});
    let resolved=match resolved{Ok(d)=>d,Err(e)=>{report["status"]=json!("rejected");report["diagnostic"]=diagnostic(source,path,&e);return report;}};
    report["phase"]=json!("checker");
    let mut stopped=false;let mut definitions=Vec::new();
    for context in &resolved {
        let inspection=if stopped{None}else{Some(ddonirang_proof::algebra::inspect_algebra_definition(context))};
        let mut steps=Vec::new();
        for (i,step) in context.steps().iter().enumerate() {
            let (operation,prior,operand,operand_span)=match &step.rule {
                AlgebraRule::ApplyDefinition{definition,values,evidence}=>{
                    ("보임 정의 호출",None,json!({"kind":"definition_call","definition":definition.declaration().canonical_name,
                        "definition_span":span(definition.declaration().span),"values":values.iter().map(|v|term(v,context)).collect::<Vec<_>>(),
                        "evidence":evidence.iter().map(|r|reference(*r)).collect::<Vec<_>>()}),step.span)
                },
                AlgebraRule::SubtractSame{prior,term:t}=>("양변빼기",Some(*prior),term(t,context),step.span),
                AlgebraRule::RewriteSides{prior,goal}=>("등식정리",Some(*prior),equality(goal,context),step.span),
                AlgebraRule::DivideSameNonzero{prior,coefficient,coefficient_span}=>("양변나누기",Some(*prior),term(coefficient,context),*coefficient_span),
            };
            let checked=inspection.as_ref().and_then(|v|v.checked_steps.get(i));
            let failed=inspection.as_ref().is_some_and(|v|v.error.is_some()&&v.failed_step==Some(i));
            let reciprocal=inspection.as_ref().and_then(|v|v.reciprocals.get(i)).and_then(|r|r.as_ref());
            steps.push(json!({"index":i,"operation":operation,"prior":prior.map(reference),"operand":operand,"operand_span":span(operand_span),
                "span":span(step.span),"status":if checked.is_some(){"checked"}else if failed{"failed"}else{"not_checked"},
                "equality":checked.map(|e|equality(e,context)),"reciprocal":reciprocal.map(|r|term(r,context))}));
        }
        let error=inspection.as_ref().and_then(|v|v.error.as_ref());
        let status=if inspection.is_none(){"not_checked"}else if error.is_some(){"rejected"}else{"accepted"};
        definitions.push(json!({"name":context.declaration().canonical_name,"span":span(context.declaration().span),
            "parameters":context.parameters().iter().enumerate().map(|(i,p)|json!({"slot":i,"name":p.pin_name,"pin_id":format!("{:?}",p.id),"span":span(p.span)})).collect::<Vec<_>>(),
            "assumptions":context.assumptions().iter().map(|e|equality(e,context)).collect::<Vec<_>>(),
            "goal":equality(context.goal(),context),"result_span":span(context.result_span()),"result":reference(context.result()),
            "steps":steps,"status":status,"diagnostic":error.map(|e|diagnostic(source,path,e))}));
        if let Some(e)=error {if !stopped{report["diagnostic"]=diagnostic(source,path,e);}stopped=true;}
    }
    report["definitions"]=json!(definitions);report["status"]=json!(if stopped{"rejected"}else{"accepted"});report
}


fn project_source_from_artifact(artifact:&str)->Result<crate::local_project::VirtualProjectSource,String> {
    use crate::local_project::{self,VirtualProjectSource,VirtualProjectFileSource};
    // open verifies the existing artifact identity and original full source.
    let project=crate::authored_project::open(artifact.as_bytes())?;
    let snapshot=project.source_snapshot;
    let root=snapshot.root_path.ok_or("E_PROOF_PROJECT: 명시 작품 뿌리가 없습니다")?;
    let root_text=snapshot.files.iter().find(|f|f.path==root).ok_or("E_PROOF_PROJECT: 작품 뿌리 원문이 없습니다")?;
    let roles=local_project::v25_package_metadata_roles(&root_text.text).map_err(|e|format!("{}: {}",e.code,e.message))?;
    Ok(VirtualProjectSource{schema:local_project::VIRTUAL_PROJECT_SOURCE_SCHEMA.into(),kind:None,package_hash:None,
        root_path:root.clone(),entry_path:snapshot.entry_path,
        files:snapshot.files.into_iter().map(|f|VirtualProjectFileSource{role:if f.path==root{"root".into()}else{roles.get(&f.path).cloned().unwrap_or_else(||"ddn_source".into())},path:f.path,utf8_text:f.text}).collect()})
}

// Linking alone does not authorize unqualified cross-module calls. Check the
// resolved callee against the actual module graph and admitted import relation.
fn validate_project_algebra_imports(
    definitions: &[ddonirang_lang::algebra_authoring::ResolvedAlgebraDefinition],
    linked: &crate::local_project::AuthoredPreparation,
    source: &crate::local_project::VirtualProjectSource,
) -> Result<(), ddonirang_lang::ParseError> {
    use ddonirang_lang::{ParseError, Span};
    let fail = |span, message: String| ParseError { span, message };
    let graph = crate::local_project::build_project_graph(source.clone())
        .map_err(|e| fail(Span::new(0, 0), format!("{}: {}", e.code, e.message)))?;
    let owner = |span| -> Result<&str, ParseError> {
        let (path, _) = linked.original_span(span).map_err(|e| fail(span, e))?;
        let mut owners = graph.module_sources.iter().filter(|(_, files)| files.contains(&path));
        let (name, _) = owners.next().ok_or_else(|| fail(span,
            "E_PROOF_PROJECT_IMPORT: 보임 정의의 원문 모듈 소유자가 없습니다".into()))?;
        if owners.next().is_some() {
            return Err(fail(span, "E_PROOF_PROJECT_IMPORT: 보임 정의의 원문 모듈 소유자가 유일하지 않습니다".into()));
        }
        graph.module_paths.get(name).map(String::as_str).ok_or_else(|| fail(span,
            "E_PROOF_PROJECT_IMPORT: 보임 정의의 모듈 머리가 없습니다".into()))
    };
    for caller in definitions {
        let caller_owner = owner(caller.declaration().span)?;
        for step in caller.steps() {
            if let ddonirang_lang::algebra_authoring::AlgebraRule::ApplyDefinition { definition, .. } = &step.rule {
                let callee_owner = owner(definition.declaration().span)?;
                if caller_owner != callee_owner && !linked.permits_import(
                    caller_owner, callee_owner, &definition.declaration().canonical_name) {
                    return Err(fail(step.span, format!(
                        "E_PROOF_PROJECT_IMPORT: 다른 모듈 보임의 명시 쓰임·드러냄 해소가 없습니다: {}",
                        definition.declaration().canonical_name)));
                }
            }
        }
    }
    Ok(())
}

fn observe_project_source(source:crate::local_project::VirtualProjectSource)->Result<serde_json::Value,String> {
    use sha2::{Digest,Sha256};
    use serde_json::{json,Value};
    let full_source=serde_json::to_string(&source).map_err(|e|e.to_string())?;
    let linked=crate::local_project::prepare_authored_execution(source.clone()).map_err(|e|format!("{}: {}",e.code,e.message))?;
    let prepared=ddonirang_lang::proof_authoring::prepare_source(&linked.source,&linked.target_path);
    let mut report=if prepared.as_ref().is_ok_and(|p|p.has_conditional_algebra_contract()) {
        let p=prepared.unwrap();
        let mut report=conditional_algebra::observe(&linked.source,&linked.target_path,p.resolve_project_conditional_algebra());
        let other=p.resolve_other_project_algebra().and_then(|d|{validate_project_algebra_imports(&d,&linked,&source)?;Ok(d)});
        let rest=observe_resolved_algebra(&linked.source,&linked.target_path,other);
        if rest["status"]=="rejected"{report["status"]=json!("rejected");report["diagnostic"]=rest["diagnostic"].clone();}
        if let (Some(target),Some(items))=(report["definitions"].as_array_mut(),rest["definitions"].as_array()){target.extend(items.iter().cloned());}
        report
    }else{
        let resolved=prepared.and_then(|p|p.resolve_project_algebra()).and_then(|d|{validate_project_algebra_imports(&d,&linked,&source)?;Ok(d)});
        observe_resolved_algebra(&linked.source,&linked.target_path,resolved)
    };
    // Keep linked coordinates as such and add original ownership. Never pretend
    // generated offsets are offsets in the importing source.
    fn map(value:&mut Value,linked:&crate::local_project::AuthoredPreparation,source:&crate::local_project::VirtualProjectSource) {
        match value {
            Value::Array(items)=>for item in items {map(item,linked,source)},
            Value::Object(fields)=>{
                for child in fields.values_mut(){map(child,linked,source);}
                if let Some(span)=fields.get("span") {
                    if let (Some(start),Some(end))=(span["start"].as_u64(),span["end"].as_u64()) {
                        if let Ok((path,original))=linked.original_span(ddonirang_lang::Span::new(start as usize,end as usize)) {
                            fields.insert("original_source".into(),json!({"path":path,"start":original.start,"end":original.end}));
                            if let Some(message)=fields.get("message").and_then(Value::as_str).map(str::to_owned) {
                                if let Some(file)=source.files.iter().find(|f|f.path==path) {
                                    let e=ddonirang_lang::ParseError{span:original,message};
                                    fields.insert("formatted".into(),json!(format_source_error(&file.utf8_text,&path,&e)));
                                }
                            }
                        }
                    }
                }
            },_=>{}
        }
    }
    map(&mut report,&linked,&source);
    report["project_source_sha256"]=json!(format!("{:x}",Sha256::digest(full_source.as_bytes())));
    report["checked_scope"]=json!("conditional_algebra_definitions_only");
    report["execution_checked"]=json!(false);
    report["coordinate_space"]=json!("linked_source_with_original_source_mapping");
    Ok(report)
}

/// Observation input is a verified authored source artifact, never a proof JSON.
pub fn observe_algebra_project(artifact:&str)->serde_json::Value {
    use serde_json::json;
    match project_source_from_artifact(artifact).and_then(observe_project_source) {
        Ok(mut report)=>{use sha2::{Digest,Sha256};report["artifact_sha256"]=json!(format!("{:x}",Sha256::digest(artifact.as_bytes())));report},
        Err(message)=>json!({"observation_only":true,"unconditional_proof_issued":false,"status":"rejected","phase":"project_analysis","definitions":[],
            "diagnostic":{"code":message.split(':').next().unwrap_or(""),"message":message,"formatted":message}})
    }
}

/// Process-local result. No public constructor, serialization or restored proof.
#[derive(Debug)]
pub struct CheckedAlgebraProject { artifact:String, definitions:usize }
impl CheckedAlgebraProject {
    pub fn definition_count(&self)->usize {self.definitions}
    pub fn validate_artifact(&self,artifact:&str)->Result<(),String> {
        if self.artifact!=artifact {return Err("E_PROOF_PROJECT_STALE: 작품 원문이나 의존 판본이 달라졌습니다".into());}
        verify_algebra_project(artifact).map(|_|())
    }
}
pub fn verify_algebra_project(artifact:&str)->Result<CheckedAlgebraProject,String> {
    let definitions=verify_project_source(project_source_from_artifact(artifact)?)?;
    Ok(CheckedAlgebraProject{artifact:artifact.into(),definitions})
}

fn verify_project_source(source:crate::local_project::VirtualProjectSource)->Result<usize,String> {
    let linked=crate::local_project::prepare_authored_execution(source.clone()).map_err(|e|format!("{}: {}",e.code,e.message))?;
    let diagnostic=|e:ddonirang_lang::ParseError| {
        if let Ok((path,span))=linked.original_span(e.span) {
            if let Some(file)=source.files.iter().find(|f|f.path==path) {
                return format_source_error(&file.utf8_text,&path,&ddonirang_lang::ParseError{span,message:e.message});
            }
        }
        format_source_error(&linked.source,&linked.target_path,&e)
    };
    let prepared=ddonirang_lang::proof_authoring::prepare_source(&linked.source,&linked.target_path).map_err(&diagnostic)?;
    let conditional=if prepared.has_conditional_algebra_contract(){prepared.resolve_project_conditional_algebra().map_err(&diagnostic)?}else{vec![]};
    for definition in &conditional{ddonirang_proof::conditional_algebra::check_definition(definition).map_err(&diagnostic)?;}
    let definitions=if prepared.has_conditional_algebra_contract(){prepared.resolve_other_project_algebra()}else{prepared.resolve_project_algebra()}.map_err(&diagnostic)?;
    validate_project_algebra_imports(&definitions,&linked,&source).map_err(&diagnostic)?;
    for definition in &definitions {ddonirang_proof::algebra::check_algebra_definition(definition).map_err(&diagnostic)?;}
    Ok(definitions.len()+conditional.len())
}


/// Read-only source provenance for the narrowly admitted two-branch goal.
/// Resolution carries no checked equality, proof, or refutation authority.
pub(crate) fn observe_card_goal(source: &str, path: &str) -> Result<Option<String>, String> {
    // A cheap dispatch guard; only the strict parser and resolver admit a goal.
    if !source.contains("고름씨") || !source.contains("보임") { return Ok(None); }
    use sha2::{Digest, Sha256};
    use serde_json::json;
    let diagnostic = |e: ddonirang_lang::ParseError| format_source_error(source, path, &e);
    let prepared = ddonirang_lang::proof_authoring::prepare_source(source, path).map_err(diagnostic)?;
    let Some(goal) = prepared.resolve_card_goal().map_err(diagnostic)? else { return Ok(None) };
    let locate = |id: u64, span: ddonirang_lang::Span| json!({
        "ast_id": id, "start": span.start, "end": span.end
    });
    let definition = |seed: &ddonirang_lang::SeedDef| json!({
        "name": seed.canonical_name,
        "declaration": locate(seed.id, seed.span),
        "definition_sha256": hex::encode(Sha256::digest(source[seed.span.start..seed.span.end].as_bytes())),
    });
    let operand = |call: &ddonirang_lang::proof_authoring::ResolvedCardCall<'_>| json!({
        "definition": definition(call.definition),
        "pin": locate(call.parameter.id, call.parameter.span),
        "argument_owner": locate(goal.argument.id, goal.argument.span),
        "call_span": [call.syntax.span.start, call.syntax.span.end],
        "pin_span": [call.syntax.pin_span.start, call.syntax.pin_span.end],
        "argument_span": [call.syntax.argument_span.start, call.syntax.argument_span.end],
    });
    let reduced = goal.reduce_reflexivity().map_err(diagnostic)?;
    let branches = reduced.iter().map(|branch| {
        let same_branch = branch.left.id == branch.right.id;
        json!({
            "input": locate(branch.input.id, branch.input.span),
            "input_name": branch.input.name,
            "left": locate(branch.left.id, branch.left.span),
            "left_name": branch.left.name,
            "right": locate(branch.right.id, branch.right.span),
            "right_name": branch.right.name,
            "status": if same_branch { "accepted" } else { "rejected" },
            "diagnostic": (!same_branch).then(|| json!({
                "code": "E_PROOF_CARD_REFLEXIVITY_MISMATCH",
                "source_path": path,
                "start": branch.evidence_span.start,
                "end": branch.evidence_span.end,
                "message": "같은 갈래에서 해소한 양변이 서로 다른 카드 값입니다",
            })),
        })
    }).collect::<Vec<_>>();
    let accepted = reduced.iter().all(|branch| branch.left.id == branch.right.id);
    Ok(Some(json!({
        "source_path": path,
        "source_sha256": hex::encode(Sha256::digest(source.as_bytes())),
        "goal_status": "resolved",
        "check_status": if accepted { "accepted" } else { "rejected" },
        "reflexivity_branches": branches,
        "unconditional_proof_issued": false,
        "checked_refutation_issued": false,
        "definition": definition(goal.definition),
        "card": locate(goal.card_id, goal.card_span),
        "branches": goal.branches.iter().map(|branch| json!({
            "name": branch.name, "declaration": locate(branch.id, branch.span),
        })).collect::<Vec<_>>(),
        "argument": locate(goal.argument.id, goal.argument.span),
        "left": operand(&goal.left),
        "right": operand(&goal.right),
    }).to_string()))
}
