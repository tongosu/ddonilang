//! Source-bound read-only model execution state used by the ordinary VM API.
//! The existing shared model producer owns all parsing, solving and projection.
//! No world capability, source rewriting, evaluator or saved kind is created.
use serde_json::Value;
use crate::ddn_runtime::{SupportedSourceProfile,V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY};
pub(crate) struct ModelSourceSession {
    source:String,path:String,canonical:String,prepared:Value,state:Value,executed:bool,
}
impl ModelSourceSession {
    pub(crate) fn prepare(source:&str,path:&str,profile:SupportedSourceProfile)->Result<Self,String> {
        if profile!=SupportedSourceProfile::V1CoreV25 {return Err("E_MODEL_SOURCE_PROFILE: 모형 원문의 지원 글판을 명시하세요".into());}
        if !ddonirang_lang::has_model_surface(source) {return Err("E_MODEL_SOURCE_REQUIRED: 실제 모형 원문이 필요합니다".into());}
        let parsed=ddonirang_lang::parse_model_program_with_source_file(source,path)
            .map_err(|e|format!("E_MODEL_FRONTDOOR: {e}"))?;
        let prepared=crate::runtime_surface::model_prepared_state_from_source(source,path)?;
        Ok(Self{source:source.into(),path:path.into(),canonical:parsed.canonical_ddn,
            state:prepared.clone(),prepared,executed:false})
    }
    pub(crate) fn canonical(&self)->&str {&self.canonical}
    pub(crate) fn state(&self)->&Value {&self.state}
    pub(crate) fn reset(&mut self) {self.state=self.prepared.clone();self.executed=false;}
    pub(crate) fn step(&mut self)->Result<(),String> {
        if self.executed {return Err("E_MODEL_ONESHOT_COMPLETED: 모형 실험은 완료됐습니다. 새 실험은 명시적으로 다시 시작하세요".into());}
        let summary=crate::runtime_surface::run_summary_from_supported_source_profile_diagnostic(
            &self.source,&self.path,Some(1),Some(V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY))
            .map_err(|e|e.to_json_value().to_string())?;
        let state=summary.get("engine_state").filter(|s|s.is_object())
            .ok_or("E_MODEL_STATE_PROJECTION: 공통 모형 실행 상태가 없습니다")?.clone();
        self.commit_candidate(state)
    }

    fn commit_candidate(&mut self, state: Value) -> Result<(), String> {
        let execution = &state["model_execution"];
        if !execution["time_execution"].is_null() {
            let verified = super::verify_execution_report(execution);
            if verified["ok"] != true || verified["post_state_hash"] != state["state_hash"] {
                return Err(format!("E_MODEL_TIME_COMMIT_REJECTED: {verified}"));
            }
        }
        self.state = state;
        self.executed = true;
        Ok(())
    }
}
#[cfg(test)]mod tests {
    use super::*;
    const SOURCE:&str=include_str!("../../tests/fixtures/model_vm_source_r1/용수철_관계.또니");
    #[test]fn prepare_and_reset_do_not_execute_model(){
        let mut s=ModelSourceSession::prepare(SOURCE,"용수철.또니",SupportedSourceProfile::V1CoreV25).unwrap();
        assert!(s.state()["model_execution"].is_null());assert_eq!(s.state()["tick_id"],0);
        s.reset();assert!(s.state()["output_rows"].as_array().unwrap().is_empty());
    }
    #[test]fn explicit_step_equals_same_shared_summary(){
        let mut s=ModelSourceSession::prepare(SOURCE,"용수철.또니",SupportedSourceProfile::V1CoreV25).unwrap();
        let expected=crate::runtime_surface::run_summary_from_supported_source_profile_diagnostic(SOURCE,"용수철.또니",Some(1),Some(V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY)).unwrap();
        s.step().unwrap();assert_eq!(s.state(),&expected["engine_state"]);
        let rows=s.state()["public_observation_table_projection"]["tables"][0]["rows"].as_array().unwrap();assert_eq!(rows.len(),1);
        let values=rows[0]["cells"].as_array().unwrap().iter().map(|c|c["value_canon"].as_str().unwrap()).collect::<Vec<_>>();
        assert_eq!(values,vec!["-1@m","4@kg/s^2","1@kg","4@kg*m/s^2","4@m/s^2"]);
    }
    #[test]fn completed_session_needs_explicit_restart(){
        let mut s=ModelSourceSession::prepare(SOURCE,"용수철.또니",SupportedSourceProfile::V1CoreV25).unwrap();s.step().unwrap();let prior=s.state().clone();
        assert!(s.step().unwrap_err().contains("E_MODEL_ONESHOT_COMPLETED"));assert_eq!(s.state(),&prior);
        s.reset();assert_eq!(s.state()["tick_id"],0);s.step().unwrap();assert_eq!(s.state(),&prior);
    }
    #[test]fn failed_model_never_commits_a_result(){
        let zero=SOURCE.replace("질량: 1@kg","질량: 0@kg");assert_ne!(zero,SOURCE);
        let mut s=ModelSourceSession::prepare(&zero,"영질량.또니",SupportedSourceProfile::V1CoreV25).unwrap();let prior=s.state().clone();
        assert!(s.step().is_err());assert_eq!(s.state(),&prior);
    }
    #[test]
    fn one_spring_step_publishes_state_and_samples_together() {
        let source = include_str!("../../tests/fixtures/model_time_r1/용수철_시간_선언.또니");
        let source = source
            .replace("0.015625@s", "0.25@s")
            .replace("0.125@s", "0.25@s")
            .replace("끝=4@s", "끝=0.25@s");
        let mut session = ModelSourceSession::prepare(
            &source, "spring.ddn", SupportedSourceProfile::V1CoreV25
        ).unwrap();
        let before = session.state().clone();
        let mut committed = crate::runtime_surface::run_summary_from_supported_source_profile_diagnostic(
            &source, "spring.ddn", Some(1), Some(V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY)
        ).unwrap()["engine_state"].clone();
        assert_eq!(committed["model_execution"]["time_execution"]["steps"], "1");
        let samples = committed["model_execution"]["committed_state"]["temporal_samples"].as_array_mut().unwrap();
        assert_eq!(samples.len(), 2);
        samples.pop();
        assert!(session.commit_candidate(committed).unwrap_err().contains("E_MODEL_TIME_COMMIT_REJECTED"));
        assert_eq!(session.state(), &before);
        assert!(!session.executed);

        session.step().unwrap();
        let state = session.state();
        let execution = &state["model_execution"];
        assert_ne!(state["state_hash"], before["state_hash"]);
        assert_eq!(state["state_hash"], execution["post_state_hash"]);
        assert_eq!(execution["committed_state"]["temporal_samples"].as_array().unwrap().len(), 2);
    }
    #[test]fn profile_and_invalid_source_are_not_rewritten(){
        assert!(ModelSourceSession::prepare(SOURCE,"용수철.또니",SupportedSourceProfile::V1CoreV25JaBaseR1).is_err());
        assert!(ModelSourceSession::prepare("1 보여주기.","값.또니",SupportedSourceProfile::V1CoreV25).is_err());
        assert!(ModelSourceSession::prepare(&SOURCE.replace("= 모형",":= 모형"),"변형.또니",SupportedSourceProfile::V1CoreV25).is_err());
    }
}
