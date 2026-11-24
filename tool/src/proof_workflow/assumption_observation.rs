//! Internal observation candidate, never a proof input or public wire.
//! All references below are display-local slots, not persistent identities.
use ddonirang_lang::{proof_authoring::{Assumption,Proposition,ResolvedEvidence,ResolvedTransfer},Span};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
fn span(s:Span)->Value {json!({"start":s.start,"end":s.end})}
fn goal(p:&Proposition)->Value {
    if let Some(parts)=p.conjunction_parts() {
        return json!({"kind":"conjunction","parts":parts.iter().map(goal).collect::<Vec<_>>()});
    }
    if let Some(d)=p.declaration() {
        return json!({"kind":"declared_proposition","label":d.label(),"span":span(d.span())});
    }
    // Existing law/coordinate handles stay opaque to this first observer.
    json!({"kind":"opaque_goal","declaration_metadata":null})
}
fn slot(d:&ResolvedTransfer,a:&Assumption)->Option<usize> {d.assumptions().iter().position(|owned|owned==a)}
fn evidence(d:&ResolvedTransfer,e:&ResolvedEvidence)->Value {
    match e {
        ResolvedEvidence::Assumption(a)=>json!({"kind":"assumption","assumption_slot":slot(d,a)}),
        ResolvedEvidence::Conjunction{goal:p,parts}=>json!({"kind":"conjunction","goal":goal(p),"parts":parts.iter().map(|e|evidence(d,e)).collect::<Vec<_>>() }),
        ResolvedEvidence::Projection{evidence:e,index}=>json!({"kind":"projection","index":index,"evidence":evidence(d,e)}),
    }
}
fn used_slots(d:&ResolvedTransfer,e:&ResolvedEvidence,used:&mut Vec<usize>) {
    match e {
        ResolvedEvidence::Assumption(a)=>{if let Some(i)=slot(d,a){if !used.contains(&i){used.push(i)}}},
        ResolvedEvidence::Conjunction{parts,..}=>for part in parts {used_slots(d,part,used)},
        ResolvedEvidence::Projection{evidence,..}=>used_slots(d,evidence,used),
    }
}
fn diagnostic(source:&str,path:&str,e:&ddonirang_lang::ParseError)->Value {
    json!({"code":e.message.split(':').next().unwrap_or(""),"message":e.message,"span":span(e.span),
        "formatted":super::format_source_error(source,path,e)})
}
/// Same parser/resolver/checker attempt. Nothing is accepted from JSON.
/// The default verifier and its existing summary do not call this candidate.
pub(crate) fn observe_assumption_source(source:&str,path:&str)->Value {
    let mut report=json!({"observation_only":true,"unconditional_proof_issued":false,
        "contract_status":"internal_candidate_not_public","source_path":path,
        "source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"source_bytes":source.len(),
        "status":"not_checked","phase":"analysis","definitions":[],"diagnostic":null});
    let resolved=ddonirang_lang::proof_authoring::prepare_source(source,path)
        .and_then(|p|super::resolve_prepared_assumption(p,source.len()));
    let definitions=match resolved {Ok(d)=>d,Err(e)=>{report["status"]=json!("rejected");report["diagnostic"]=diagnostic(source,path,&e);return report}};
    report["phase"]=json!("checker");let mut stopped=false;let mut rows=Vec::new();
    for (index,d) in definitions.iter().enumerate() {
        let checked=if stopped {None}else{Some(ddonirang_proof::check_assumption_transfer(d))};
        let error=checked.as_ref().and_then(|r|r.as_ref().err());
        // Emit from the exact checked owner where available; rejection details
        // remain visibly candidates and do not become checked observations.
        let d=checked.as_ref().and_then(|r|r.as_ref().ok()).map(|c|c.resolved()).unwrap_or(d);
        let mut used=Vec::new();used_slots(d,d.evidence(),&mut used);
        let application=d.application().map(|a|json!({"definition_name":a.definition().definition_name(),
            "definition_span":span(a.definition().definition_span()),
            "substitution":a.substitution().iter().map(|(f,t)|json!({"formal":goal(f),"actual":goal(t)})).collect::<Vec<_>>(),
            "argument_assumption_slots":a.arguments().iter().map(|a|slot(d,a)).collect::<Vec<_>>() }));
        rows.push(json!({"definition_index":index,"name":d.definition_name(),"span":span(d.definition_span()),
            "goal":goal(d.goal()),"assumptions":d.assumptions().iter().enumerate().map(|(i,a)|json!({
                "assumption_slot":i,"public_pin":a.public_pin(),"local_name":a.declaration().label(),
                "span":span(a.declaration().span()),"goal":goal(a.goal())})).collect::<Vec<_>>(),
            "result_span":span(d.result_span()),"evidence":evidence(d,d.evidence()),
            "used_assumption_slots":used,"application":application,
            "status":if checked.is_none(){"not_checked"}else if error.is_some(){"rejected"}else{"accepted"},
            "diagnostic":error.map(|e|diagnostic(source,path,e))}));
        if let Some(e)=error {if !stopped{report["diagnostic"]=diagnostic(source,path,e)}stopped=true;}
    }
    report["status"]=json!(if stopped{"rejected"}else{"accepted"});report["definitions"]=json!(rows);report
}
#[cfg(test)]
mod tests {
    use super::*;
    const SIMPLE:&str="(P:세움) 두고\n(공개증거=본문증거:(P)보임~을)\n가정쓰다:(P)보임:셈씨 := { 가정쓰다 <- 본문증거. }.";
    #[test]
    fn same_checked_owner_exposes_binder_goal_evidence_and_original_utf8_spans() {
        let source=format!("// 한글 😀\r\n{}",SIMPLE.replace('\n',"\r\n"));
        let report=observe_assumption_source(&source,"관찰.ddn");assert_eq!(report["status"],"accepted");
        let d=&report["definitions"][0];assert_eq!(d["name"],"가정쓰다");assert_eq!(d["goal"]["label"],"P");
        assert_eq!(d["assumptions"][0]["public_pin"],"공개증거");assert_eq!(d["assumptions"][0]["local_name"],"본문증거");
        assert_eq!(d["evidence"]["assumption_slot"],0);assert_eq!(d["used_assumption_slots"],json!([0]));
        let s=&d["result_span"];assert_eq!(&source[s["start"].as_u64().unwrap() as usize..s["end"].as_u64().unwrap() as usize],"본문증거");
        assert_eq!(super::super::verify_conditional_source(&source,"관찰.ddn").unwrap().len(),1);
        assert_eq!(report["unconditional_proof_issued"],false);
        if let Ok(out)=std::env::var("V4_GOAL_OBSERVATION_OUTPUT") {
            let p=std::path::Path::new(&out);std::fs::create_dir_all(p).unwrap();
            std::fs::write(p.join("accepted.ddn"),&source).unwrap();std::fs::write(p.join("accepted.json"),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
    }
    #[test]
    fn unknown_binder_is_not_turned_into_partial_checker_success() {
        let source=SIMPLE.replace("<- 본문증거","<- 없는근거");assert_ne!(source,SIMPLE);let r=observe_assumption_source(&source,"거부.ddn");
        assert_eq!(r["status"],"rejected");assert_eq!(r["phase"],"analysis");assert!(r["definitions"].as_array().unwrap().is_empty());
        assert_eq!(r["diagnostic"]["code"],"E_PROOF_ASSUMPTION");
    }
    #[test]
    fn actual_projection_conjunction_and_call_keep_their_own_source_binders() {
        let project="(P:세움, Q:세움) 두고 (근거=근거:(P 이고 Q)보임~을) 첫근거쓰다:(P)보임:셈씨 := { 첫근거쓰다 <- 근거[0]. }.";
        let construct="(P:세움, Q:세움) 두고 (왼=왼:(P)보임~을, 오른=오른:(Q)보임~로) 모으다:(P 이고 Q)보임:셈씨 := { 모으다 <- (왼, 오른). }.";
        let caller="(Q:세움) 두고 (근거=근거:(Q)보임~을) 이어쓰다:(Q)보임:셈씨 := { 이어쓰다 <- (근거) 가정쓰다. }.";
        let source=format!("{SIMPLE}\n{project}\n{construct}\n{caller}");
        let r=observe_assumption_source(&source,"결합과호출.ddn");assert_eq!(r["status"],"accepted");
        let defs=r["definitions"].as_array().unwrap();assert_eq!(defs.len(),4);
        assert_eq!(defs[1]["evidence"]["kind"],"projection");assert_eq!(defs[1]["evidence"]["index"],0);
        assert_eq!(defs[1]["used_assumption_slots"],json!([0]));
        assert_eq!(defs[2]["goal"]["kind"],"conjunction");assert_eq!(defs[2]["used_assumption_slots"],json!([0,1]));
        let a=&defs[3]["application"];assert_eq!(a["definition_name"],"가정쓰다");assert_eq!(a["argument_assumption_slots"],json!([0]));
        assert_eq!(a["substitution"][0]["formal"]["label"],"P");assert_eq!(a["substitution"][0]["actual"]["label"],"Q");
        assert_ne!(a["substitution"][0]["formal"]["span"],a["substitution"][0]["actual"]["span"]);
        if let Ok(out)=std::env::var("V4_GOAL_OBSERVATION_OUTPUT") {
            let p=std::path::Path::new(&out);std::fs::create_dir_all(p).unwrap();
            std::fs::write(p.join("projection-conjunction-call.ddn"),&source).unwrap();std::fs::write(p.join("projection-conjunction-call.json"),serde_json::to_vec_pretty(&r).unwrap()).unwrap();
        }
    }
    #[test]
    fn first_checker_rejection_stops_later_definitions_without_claiming_their_proofs() {
        let wrong=SIMPLE.replace("P:세움","P:세움, Q:세움").replace("본문증거:(P)보임","본문증거:(Q)보임").replace("가정쓰다","다른쓰다");
        let source=format!("{SIMPLE}\n{wrong}\n{}",SIMPLE.replace("가정쓰다","끝쓰다"));
        let r=observe_assumption_source(&source,"실패순서.ddn");assert_eq!(r["phase"],"checker");assert_eq!(r["status"],"rejected");
        let statuses=r["definitions"].as_array().unwrap().iter().map(|d|d["status"].as_str().unwrap()).collect::<Vec<_>>();
        assert_eq!(statuses,["accepted","rejected","not_checked"]);assert_eq!(r["diagnostic"]["code"],"E_PROOF_CONTEXT_MISMATCH");
        assert!(super::super::verify_conditional_source(&source,"실패순서.ddn").is_err());
        if let Ok(out)=std::env::var("V4_GOAL_OBSERVATION_OUTPUT") {
            let p=std::path::Path::new(&out);std::fs::create_dir_all(p).unwrap();
            std::fs::write(p.join("first-rejection.ddn"),&source).unwrap();std::fs::write(p.join("first-rejection.json"),serde_json::to_vec_pretty(&r).unwrap()).unwrap();
        }
    }

}
