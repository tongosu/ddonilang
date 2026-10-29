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
    let resolved=if prepared.has_coordinate_contract() {
        let candidate=crate::numeric_constraint_plan::coordinate_proof_candidate(&prepared)
            .map_err(|(message,span)|ddonirang_lang::ParseError {span:span.unwrap_or(ddonirang_lang::Span::new(0,source_len)),message})?;
        prepared.resolve_with_numeric_candidate(&candidate)?
    }else{prepared.resolve()?};
    resolved
        .iter().map(ddonirang_proof::check_assumption_transfer).collect()
}


/// Checked definitions from one common source frontdoor; no JSON import path.
#[derive(Clone,Debug)]
pub enum CheckedSourceDefinition {
    Assumption(ddonirang_proof::CheckedAssumptionTransfer),
    Algebra(ddonirang_proof::algebra::CheckedAlgebraDefinition),
}
impl CheckedSourceDefinition {
    pub fn validate_source(&self,source:&str,path:&str)->Result<(),ddonirang_lang::ParseError> {
        match self {Self::Assumption(c)=>c.validate_source(source,path),Self::Algebra(c)=>c.validate_source(source,path)}
    }
}
pub fn verify_conditional_source(source:&str,path:&str)->Result<Vec<CheckedSourceDefinition>,ddonirang_lang::ParseError> {
    let prepared=ddonirang_lang::proof_authoring::prepare_source(source,path)?;
    if prepared.has_algebra_contract() {
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
    if input.extension().and_then(|ext| ext.to_str()) == Some("ddn") {
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
    let prepared=match ddonirang_lang::proof_authoring::prepare_source(source,path){Ok(p)=>p,Err(e)=>{report["status"]=json!("rejected");report["diagnostic"]=diagnostic(source,path,&e);return report;}};
    let resolved=match prepared.resolve_algebra(){Ok(d)=>d,Err(e)=>{report["status"]=json!("rejected");report["diagnostic"]=diagnostic(source,path,&e);return report;}};
    report["phase"]=json!("checker");
    let mut stopped=false;let mut definitions=Vec::new();
    for context in &resolved {
        let inspection=if stopped{None}else{Some(ddonirang_proof::algebra::inspect_algebra_definition(context))};
        let mut steps=Vec::new();
        for (i,step) in context.steps().iter().enumerate() {
            let (operation,prior,operand,operand_span)=match &step.rule {
                AlgebraRule::SubtractSame{prior,term:t}=>("양변빼기",*prior,term(t,context),step.span),
                AlgebraRule::RewriteSides{prior,goal}=>("등식정리",*prior,equality(goal,context),step.span),
                AlgebraRule::DivideSameNonzero{prior,coefficient,coefficient_span}=>("양변나누기",*prior,term(coefficient,context),*coefficient_span),
            };
            let checked=inspection.as_ref().and_then(|v|v.checked_steps.get(i));
            let failed=inspection.as_ref().is_some_and(|v|v.error.is_some()&&v.failed_step==Some(i));
            let reciprocal=inspection.as_ref().and_then(|v|v.reciprocals.get(i)).and_then(|r|r.as_ref());
            steps.push(json!({"index":i,"operation":operation,"prior":reference(prior),"operand":operand,"operand_span":span(operand_span),
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
