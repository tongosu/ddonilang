//! Read-only observation of the same source-owned checker. Not proof input.
use ddonirang_lang::{conditional_algebra::*,ParseError,Span};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
fn span(s:Span)->Value{json!({"start":s.start,"end":s.end})}
fn term(t:&BoundTerm,source:&str)->Value{
 let s=t.span();json!({"kind":"source_bound_expression","source_text":source.get(s.start..s.end),"display_text":t.source_text(),"span":span(s)})
}
fn equality(e:&Equality,source:&str)->Value{json!({"relation":"equality","left":term(&e.left,source),"right":term(&e.right,source),"left_span":span(e.left.span()),"right_span":span(e.right.span())})}
fn reference(r:EvidenceRef)->Value{match r{EvidenceRef::Input(i)=>json!({"kind":"assumption","index":i}),EvidenceRef::Step(i)=>json!({"kind":"step","index":i})}}
pub(super) fn observe(source:&str,path:&str,resolved:Result<Vec<ResolvedDefinition>,ParseError>)->Value{
 let mut report=json!({"observation_only":true,"unconditional_proof_issued":false,"source_path":path,"source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"source_bytes":source.len(),"status":"not_checked","phase":"analysis","definitions":[],"diagnostic":null});
 let diagnostic=|e:&ParseError|json!({"code":e.message.split(':').next().unwrap_or(""),"message":e.message,"span":span(e.span),"formatted":super::format_source_error(source,path,e)});
 let defs=match resolved{Ok(d)=>d,Err(e)=>{report["status"]=json!("rejected");report["diagnostic"]=diagnostic(&e);return report;}};
 report["phase"]=json!("checker");let mut stopped=false;let mut output=Vec::new();
 for d in defs{
  let checked=if stopped{None}else{Some(ddonirang_proof::conditional_algebra::check_definition(&d))};
  let status=match &checked{Some(Ok(_))=>"accepted",Some(Err(e))=>{stopped=true;report["diagnostic"]=diagnostic(e);"rejected"},None=>"not_checked"};
  let assumptions=d.assumptions().iter().enumerate().map(|(i,a)|match a{
   Assumption::Nonzero(t)=>json!({"index":i,"relation":"nonzero","expression":term(t,source),"span":span(t.span())}),
   Assumption::Equality(eq)=>{let mut v=equality(eq,source);v["index"]=json!(i);v}
  }).collect::<Vec<_>>();
  let success=checked.as_ref().and_then(|c|c.as_ref().ok());
  // A failed replay emits its actual diagnostic, never a fabricated checked prefix.
  let steps=if success.is_some(){d.steps().iter().enumerate().map(|(i,s)|{
   let(operation,prior,operand)=match &s.rule{
    Rule::Rewrite{prior}=>("등식정리",reference(*prior),equality(&s.goal,source)),
    Rule::ApplyDefinition{definition,values,evidence}=>("보임 정의 호출",Value::Null,json!({"kind":"definition_call","definition":definition.declaration().canonical_name,"definition_span":span(definition.declaration().span),"values":values.iter().map(|v|term(v,source)).collect::<Vec<_>>(),"evidence":evidence.iter().map(|r|reference(*r)).collect::<Vec<_>>()})),
   };
   json!({"index":i,"operation":operation,"prior":prior,"operand":operand,"operand_span":span(s.span),"span":span(s.span),"status":"checked","equality":equality(&s.goal,source),"reciprocal":null})
  }).collect::<Vec<_>>()}else{vec![]};
  output.push(json!({"name":d.declaration().canonical_name,"span":span(d.declaration().span),"definition_id":format!("{:?}",d.declaration().id),"conditional_definedness":true,"parameters":d.parameters().iter().enumerate().map(|(i,p)|json!({"slot":i,"kernel_symbol":format!("p{i}"),"name":p.pin_name,"pin_id":format!("{:?}",p.id),"span":span(p.span)})).collect::<Vec<_>>(),"assumptions":assumptions,"goal":equality(d.goal(),source),"result":reference(d.result()),"result_span":span(d.result_span()),"steps":steps,"status":status,"diagnostic":checked.as_ref().and_then(|c|c.as_ref().err()).map(diagnostic),"normalized_result":success.map(|c|json!({"left":c.left,"right":c.right})),"retained_conditions":success.map(|c|&c.retained_conditions),"used_assumptions":success.map(|c|&c.used_assumptions),"checked_steps":success.map(|c|c.checked_steps),"unconditional_proof_issued":false}));
 }
 report["status"]=json!(if stopped{"rejected"}else if output.is_empty(){"not_checked"}else{"accepted"});report["definitions"]=json!(output);report
}