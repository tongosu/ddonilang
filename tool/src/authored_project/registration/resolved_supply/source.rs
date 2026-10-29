//! Two existing, distinct source owners. Temporary admission never fabricates
//! a Project/Start/PESI or an authored permanent module to reuse this compiler.
use super::*;
use super::super::{RegistrationSourceView,RegistrationDraft};
use crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1;
use crate::source_snapshot_admission::SourceCapture;

pub(in crate::authored_project::registration) enum Source {
    Saved(std::sync::Arc<PreCanonAuthoredSourceV1>),
    Temporary(std::sync::Arc<super::super::temporary::DirectSourceActivation>),
}
pub(super) struct Preparation<'a> {pub source:&'a str,pub target_path:&'a str}
impl Source {
    pub(in crate::authored_project::registration) fn saved(source:&PreCanonAuthoredSourceV1)->Self {Self::Saved(std::sync::Arc::new(source.clone()))}
    pub(in crate::authored_project::registration) fn temporary(source:std::sync::Arc<super::super::temporary::DirectSourceActivation>)->Result<Self,String> {
        let captures=source.captures();
        if captures.len()!=1||source.checked_program().source_epoch!=ddonirang_lang::SourceEpoch::V25
            ||source.checked_program().origin.source!=crate::file_meta::split_file_meta(captures[0].1.text()).stripped {
            return Err(fail("임시 실행의 실제 원문·정적 검사 경계가 다릅니다"));
        }
        Ok(Self::Temporary(source))
    }
    pub(super) fn mode(&self)->&'static str {match self {Self::Saved(_)=>"saved",Self::Temporary(_)=>"temporary"}}
    pub(super) fn scope(&self)->&str {match self {Self::Saved(source)=>source.project().authored_identity.logical_dependency_id.as_str(),Self::Temporary(source)=>source.scope()}}
    pub(super) fn captures(&self)->&[(String,SourceCapture)] {match self {Self::Saved(source)=>source.sources().captures(),Self::Temporary(source)=>source.captures()}}
    pub(super) fn module(&self,path:&str)->Result<Option<[u8;32]>,String> {
        match self {
            Self::Saved(source)=>source.module_for_source(path)?.map(|module|hex::decode(module.id.as_str()).map_err(|_|fail("모듈 정체 표기 오류"))?.try_into().map_err(|_|fail("모듈 정체 길이 오류"))).transpose(),
            Self::Temporary(source)=>source.module(path),
        }
    }
    pub(super) fn module_ids(&self)->Result<Vec<[u8;32]>,String> {
        match self {
            Self::Saved(source)=>source.project().authored_identity.modules.iter().map(|module|hex::decode(module.id.as_str()).map_err(|_|fail("모듈 정체 표기 오류"))?.try_into().map_err(|_|fail("모듈 정체 길이 오류"))).collect(),
            Self::Temporary(source)=>Ok(vec![source.module(&source.captures()[0].0)?.ok_or_else(||fail("임시 모듈 정체가 없습니다"))?]),
        }
    }
    pub(super) fn preparation(&self)->Preparation<'_> {
        match self {
            Self::Saved(source)=>{let p=source.sources().preparation();Preparation{source:&p.source,target_path:&p.target_path}},
            Self::Temporary(source)=>{let p=source.checked_program();Preparation{source:&p.origin.source,target_path:&p.origin.file_path}},
        }
    }
    pub(super) fn original_span(&self,span:Span)->Result<(String,Span),String> {
        match self {Self::Saved(source)=>source.sources().preparation().original_span(span).map(|(path,span)|(path.into(),span)),Self::Temporary(source)=>source.original_span(span)}
    }
    pub(super) fn verify_nominals(&self,draft:&RegistrationDraft)->Result<(),String> {
        match self {Self::Saved(source)=>draft.verify_precanon_binding(source),Self::Temporary(source)=>source.verify(draft)}
    }
    pub(super) fn checked_program(&self)->Result<CanonProgram,String> {
        match self {
            Self::Saved(source)=>Ok(crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(source.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target())?.canonical_program().clone()),
            Self::Temporary(source)=>Ok(source.checked_program().clone()),
        }
    }
    #[cfg(test)]
    pub(super) fn saved_owner(&self)->&PreCanonAuthoredSourceV1 {match self {Self::Saved(source)=>source,Self::Temporary(_)=>panic!("saved test fixture required")}}
}
