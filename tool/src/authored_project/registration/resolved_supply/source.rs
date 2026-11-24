//! Two existing, distinct source owners. Temporary admission never fabricates
//! a Project/Start/PESI or an authored permanent module to reuse this compiler.
use super::*;
use super::super::{RegistrationSourceView,RegistrationDraft};
use crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1;
use crate::source_snapshot_admission::SourceCapture;

pub(in crate::authored_project::registration) enum Source {
    Saved(std::sync::Arc<PreCanonAuthoredSourceV1>),
    Temporary(std::sync::Arc<super::super::temporary::DirectSourceActivation>),
    Nominal(std::sync::Arc<super::super::nominal_source::RootlessNominalSource>),
}
pub(super) struct Preparation<'a> {pub source:&'a str,pub target_path:&'a str}
impl Source {
    pub(in crate::authored_project::registration) fn nominal(source:std::sync::Arc<super::super::nominal_source::RootlessNominalSource>)->Self{Self::Nominal(source)}
    pub(in crate::authored_project::registration) fn saved(source:&PreCanonAuthoredSourceV1)->Self {Self::Saved(std::sync::Arc::new(source.clone()))}
    pub(in crate::authored_project::registration) fn temporary(source:std::sync::Arc<super::super::temporary::DirectSourceActivation>)->Result<Self,String> {
        let captures=source.captures();
        if captures.len()!=1||source.checked_program().source_epoch!=ddonirang_lang::SourceEpoch::V25
            ||source.checked_program().origin.source!=crate::file_meta::split_file_meta(captures[0].1.text()).stripped {
            return Err(fail("임시 실행의 실제 원문·정적 검사 경계가 다릅니다"));
        }
        Ok(Self::Temporary(source))
    }
    pub(super) fn mode(&self)->&'static str {match self {Self::Saved(_)=>"saved",Self::Temporary(_)=>"temporary",Self::Nominal(_)=>"nominal-source"}}
    pub(super) fn scope(&self)->&str {match self {Self::Saved(source)=>source.project().authored_identity.logical_dependency_id.as_str(),Self::Temporary(source)=>source.scope(),Self::Nominal(source)=>source.scope()}}
    pub(super) fn captures(&self)->&[(String,SourceCapture)] {match self {Self::Saved(source)=>source.sources().captures(),Self::Temporary(source)=>source.captures(),Self::Nominal(source)=>source.captures()}}
    pub(super) fn module(&self,path:&str)->Result<Option<[u8;32]>,String> {
        match self {
            Self::Saved(source)=>source.module_for_source(path)?.map(|module|hex::decode(module.id.as_str()).map_err(|_|fail("모듈 정체 표기 오류"))?.try_into().map_err(|_|fail("모듈 정체 길이 오류"))).transpose(),
            Self::Temporary(source)=>source.module(path),Self::Nominal(source)=>source.module(path),
        }
    }
    pub(super) fn module_ids(&self)->Result<Vec<[u8;32]>,String> {
        match self {
            Self::Saved(source)=>source.project().authored_identity.modules.iter().map(|module|hex::decode(module.id.as_str()).map_err(|_|fail("모듈 정체 표기 오류"))?.try_into().map_err(|_|fail("모듈 정체 길이 오류"))).collect(),
            Self::Temporary(source)=>Ok(vec![source.module(&source.captures()[0].0)?.ok_or_else(||fail("임시 모듈 정체가 없습니다"))?]),
            Self::Nominal(source)=>Ok(vec![source.module_id()]),
        }
    }
    pub(super) fn preparation(&self)->Preparation<'_> {
        match self {
            Self::Saved(source)=>{let p=source.sources().preparation();Preparation{source:&p.source,target_path:&p.target_path}},
            Self::Temporary(source)=>{let p=source.checked_program();Preparation{source:&p.origin.source,target_path:&p.origin.file_path}},
            Self::Nominal(source)=>{let p=source.checked_program();Preparation{source:&p.origin.source,target_path:&p.origin.file_path}},
        }
    }
    pub(super) fn original_span(&self,span:Span)->Result<(String,Span),String> {
        match self {Self::Saved(source)=>source.sources().preparation().original_span(span).map(|(path,span)|(path.into(),span)),Self::Temporary(source)=>source.original_span(span),Self::Nominal(source)=>source.original_span(span)}
    }
    pub(super) fn verify_nominals(&self,draft:&RegistrationDraft)->Result<(),String> {
        match self {Self::Saved(source)=>draft.verify_precanon_binding(source),Self::Temporary(source)=>source.verify(draft),Self::Nominal(source)=>source.verify(draft)}
    }
    fn j1_profile(&self)->Option<ddonirang_lang::ddl::CheckedJ1Profile>{match self{Self::Saved(source)=>source.sources().saved().j1_profile(),Self::Temporary(_)|Self::Nominal(_)=>None}}
    pub(super) fn parse_capture(&self,text:&str,path:&str)->Result<CanonProgram,ddonirang_lang::ParseError>{
        if let Self::Saved(owned)=self{if let Some(context)=owned.sources().logic_callable_context(){
            let prepared=owned.sources().preparation();
            let linked_owner=owned.sources().linked_logic_path();
            if path==linked_owner.unwrap_or(&prepared.target_path){
                let source_matches=if linked_owner.is_some(){
                    owned.sources().captures().iter().any(|(owner,capture)|owner==path&&capture.text()==text)
                }else{text==prepared.source};
                if !source_matches{return Err(ddonirang_lang::ParseError{span:Span::new(0,0),message:"E_LOGIC_SOURCE_OWNER_MISMATCH".into()});}
                let syntax=self.j1_profile().ok_or_else(||ddonirang_lang::ParseError{span:Span::new(0,0),message:"E_LOGIC_SOURCE_PROFILE_REQUIRED".into()})?;
                return ddonirang_lang::Parser::parse_with_logic_callable_context(text,path,syntax,context.clone());
            }
        }}
        match self.j1_profile(){Some(profile)=>ddonirang_lang::Parser::parse_unlinked_j1_module(text,path,profile),None=>ddonirang_lang::parse_v25_with_mode(text,path,ddonirang_lang::ParseMode::Strict)}
    }
    pub(super) fn parse_prepared(&self)->Result<CanonProgram,ddonirang_lang::ParseError>{
        if let Self::Saved(owned)=self{if let Some(context)=owned.sources().logic_callable_context(){
            if let Some(program)=owned.sources().linked_logic_program(){return Ok(program.clone());}
            let p=owned.sources().preparation();
            let syntax=self.j1_profile().ok_or_else(||ddonirang_lang::ParseError{span:Span::new(0,0),message:"E_LOGIC_SOURCE_PROFILE_REQUIRED".into()})?;
            return ddonirang_lang::Parser::parse_with_logic_callable_context(&p.source,&p.target_path,syntax,context.clone());
        }}
        let p=self.preparation();match self.j1_profile(){Some(profile)=>ddonirang_lang::Parser::parse_j1_v25_with_mode(p.source,p.target_path,profile,ddonirang_lang::ParseMode::Strict),None=>ddonirang_lang::parse_v25_with_mode(p.source,p.target_path,ddonirang_lang::ParseMode::Strict)}
    }
    pub(super) fn checked_program(&self)->Result<CanonProgram,String> {
        match self {
            Self::Saved(source) if source.sources().saved().j1_profile().is_some()=>{
                let mut program=self.parse_prepared().map_err(|e|fail(&e.to_string()))?;
                ddonirang_lang::canonicalize(&mut program).map_err(|e|fail(&e.to_string()))?;Ok(program)
            },
            Self::Saved(source)=>Ok(crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(source.sources().clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target())?.canonical_program().clone()),
            Self::Temporary(source)=>Ok(source.checked_program().clone()),Self::Nominal(source)=>Ok(source.checked_program().clone()),
        }
    }
    /// Borrow source-owned public facts without issuing identities or changing
    /// the default registration admission of old saved projects.
    pub(super) fn public_callable_facts(&self,module:&str)
        ->Result<crate::authored_project::public_surface::SourceOwnedPublicFacts,String> {
        let Self::Saved(owned)=self else {
            return Err(fail("공개 함수 선택은 실제 저장 원본 owner가 필요합니다"));
        };
        let facts=owned.project().source_owned_public_facts(module)?;
        for row in facts.declarations() {
            let (_,capture)=self.captures().iter().find(|(path,_)|path==row.source_path())
                .ok_or_else(||fail("공개 선언의 실제 원문 capture가 없습니다"))?;
            if hex::encode(<sha2::Sha256 as sha2::Digest>::digest(capture.bytes()))!=row.source_sha256()
                ||self.module(row.source_path())?!=Some(hex::decode(facts.module().as_str())
                    .map_err(|_|fail("공개 모듈 정체 오류"))?.try_into().map_err(|_|fail("공개 모듈 정체 길이 오류"))?) {
                return Err(fail("공개 선언의 원문·소유 모듈이 다릅니다"));
            }
        }
        Ok(facts)
    }
    #[cfg(test)]
    pub(super) fn saved_owner(&self)->&PreCanonAuthoredSourceV1 {match self {Self::Saved(source)=>source,Self::Temporary(_)|Self::Nominal(_)=>panic!("saved test fixture required")}}
}
