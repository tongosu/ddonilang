//! Ordinary SourceProgram admission. Never a saved Project/Start/PESI/Grant.
use super::*;
use crate::source_snapshot_admission::{SourceCapture,EpochSupply};
use ddonirang_lang::{CanonProgram,NodeId,Span,SeedKind,TopLevelItem};
use std::sync::Arc;

pub(super) struct DirectSourceActivation {
    scope:String,module:[u8;32],captures:Vec<(String,SourceCapture)>,
    occurrences:Vec<ddonirang_lang::ast::AtomValueOccurrence>,removed_header_bytes:usize,
    declarations:Vec<ddonirang_lang::SeedDef>,
    checked_program:Arc<CanonProgram>,
}
impl DirectSourceActivation {pub(super) fn checked_program(&self)->&CanonProgram {&self.checked_program}}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(source:&str,ticks:u32)->Result<serde_json::Value,String>{
        crate::runtime_surface::run_summary_from_supported_source_profile(source,"temporary-test.ddn",Some(ticks),Some("v1-core-v25"))
    }
    #[test]
    fn r3_1_temporary_sum_keeps_private_scope_and_never_creates_saved_artifacts() {
        let source="선택:고름씨 := { $쌍(가로:바른수,세로:바른수). }. 선택.$쌍(세로=4,가로=3)에 따라 { 선택.$쌍(가로,세로)이면 { (가로+세로) 보여주기. }. }.";
        let profile=crate::ddn_runtime::resolve_supported_source_profile(Some("v1-core-v25")).unwrap();
        let mut program=crate::ddn_runtime::DdnProgram::from_supported_source_profile_with_mode(source,"sum-source.ddn",ddonirang_lang::ParseMode::Strict,profile).unwrap();
        let capture=*program.sealed_source_capture().unwrap().identity();
        let first=TemporaryRegisteredSource::prepare(&program).unwrap().unwrap();
        let second=TemporaryRegisteredSource::prepare(&program).unwrap().unwrap();
        assert_ne!(first.digest(),second.digest());
        let catalog=first.registration.resolved.as_ref().unwrap();
        let transport:serde_json::Value=serde_json::from_slice(catalog.bytes()).unwrap();
        assert_eq!(transport["mode"],"temporary");assert!(transport["scope"].as_str().unwrap().starts_with("temporary:"));
        assert!(first.registration.artifacts.iter().all(|blob|blob.role()!=Some(super::super::resolved_supply::RESOLVED_ROLE)),"temporary catalog is memory-only, not a saved resource");
        program.prepare_temporary_source_execution().unwrap();program.prepare_temporary_source_execution().unwrap();
        assert_eq!(*program.sealed_source_capture().unwrap().identity(),capture);assert!(program.sealed_project_sources().is_none());
        assert!(program.has_temporary_registration());
        assert_eq!(run(source,1).unwrap()["output_log_texts"],serde_json::json!(["7"]));
    }
    #[test]
    fn vc04_temporary_ordinary_nominal_and_atom_register_before_execution(){
        for (source,expected) in [
            ("명찰:이름씨 := { 이름:글. }. 표 <- (이름=\"철수\")인 명찰. 표.이름 보여주기.",serde_json::json!(["철수"])),
            ("이름표 <- #철수. (이름표 == #철수) 보여주기.",serde_json::json!(["참"])),
            ("# 제목: 임시 원문\n명찰:이름씨 := { 이름:글. }. 표 <- (이름=\"철수\")인 명찰. 표.이름 보여주기.",serde_json::json!(["철수"])),
        ] {
            let result=run(source,1).unwrap();assert_eq!(result["output_log_texts"],expected);
        }
    }
    #[test]
    fn vc04_temporary_registration_preserves_capture_and_separates_invocations(){
        let profile=crate::ddn_runtime::resolve_supported_source_profile(Some("v1-core-v25")).unwrap();
        let source="명찰:이름씨 := { 이름:글. }. 표찰:이름씨 := { 이름:글. }. 이름표 <- #철수.";
        let mut program=crate::ddn_runtime::DdnProgram::from_supported_source_profile_with_mode(source,"a.ddn",ddonirang_lang::ParseMode::Strict,profile).unwrap();
        let capture=*program.sealed_source_capture().unwrap().identity();
        let first=TemporaryRegisteredSource::prepare(&program).unwrap().unwrap();
        let second=TemporaryRegisteredSource::prepare(&program).unwrap().unwrap();
        assert_ne!(first.digest(),second.digest());
        let schemas=first.bind_constructors(program.canonical_program()).unwrap();
        let ids=schemas.values().map(|schema|schema.name_type).collect::<BTreeSet<_>>();assert_eq!(ids.len(),2);
        program.prepare_temporary_source_execution().unwrap();program.prepare_temporary_source_execution().unwrap();
        assert_eq!(*program.sealed_source_capture().unwrap().identity(),capture);
        assert!(program.sealed_project_sources().is_none());assert!(program.has_temporary_registration());
    }
    #[test]
    fn vc04_temporary_subject_keeps_lifecycle_without_saved_pesi(){
        let source="병사:임자 := { 이름:글. }. 표적 <- (이름=\"철수\")인 병사. 첫 <- 표적 가리키기. (첫 == (표적 가리키기)) 보여주기. (대상=첫) 폐기하기. 다음 <- (이름=\"영희\")인 병사. (표적 == 다음) 보여주기.";
        let result=run(source,1).unwrap();assert_eq!(result["output_log_texts"],serde_json::json!(["참","거짓"]));
    }
    #[test]
    fn vc04_temporary_pure_creation_remains_rejected(){
        for source in [
            "병사:임자 := { 이름:글. }. 검사:병사:셈씨 := { 검사 <- (이름=\"철수\")인 병사. }.",
            "병사:임자 := { 이름:글. }. 생성:움직씨 := { 새값 <- (이름=\"철수\")인 병사. }. 검사:바른수:셈씨 := { 생성. 검사 <- 1. }.",
        ]{assert!(run(source,1).unwrap_err().contains("E_VC04_PURE_WORLD_WRITE"));}
    }
}
impl RegistrationSourceView for DirectSourceActivation {
    fn scope(&self)->&str {&self.scope}
    fn captures(&self)->&[(String,SourceCapture)] {&self.captures}
    fn module(&self,path:&str)->Result<Option<[u8;32]>,String>{
        if path!=self.captures[0].0{return Err(fail("임시 모듈의 원문 경계가 다릅니다"))}Ok(Some(self.module))
    }
    fn occurrences(&self)->Vec<(&str,&ddonirang_lang::ast::AtomValueOccurrence)>{self.occurrences.iter().map(|o|(self.captures[0].0.as_str(),o)).collect()}
    fn declarations(&self,path:&str,capture:&SourceCapture)->Result<Vec<ddonirang_lang::SeedDef>,String>{
        if path!=self.captures[0].0||capture.identity()!=self.captures[0].1.identity(){return Err(fail("임시 선언의 원문 봉인이 다릅니다"))}
        Ok(self.declarations.clone())
    }
    fn original_span(&self,span:Span)->Result<(String,Span),String>{
        let start=span.start.checked_add(self.removed_header_bytes).ok_or_else(||fail("원문 범위 초과"))?;
        let end=span.end.checked_add(self.removed_header_bytes).ok_or_else(||fail("원문 범위 초과"))?;
        let text=self.captures[0].1.text();
        if start>end||!text.is_char_boundary(start)||!text.is_char_boundary(end){return Err(fail("임시 원문 범위가 다릅니다"))}
        Ok((self.captures[0].0.clone(),Span{start,end}))
    }
    fn verify(&self,draft:&RegistrationDraft)->Result<(),String>{
        if draft.project!=self.scope{return Err(fail("다른 임시 실행의 등록 자료입니다"))}
        let epoch=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);
        for (_,capture) in &self.captures {crate::source_snapshot_admission::replay(&epoch,capture).map_err(|_|fail("임시 원문 봉인이 다릅니다"))?;}
        let candidate=wire::Candidate::parse(&draft.descriptor).map_err(contract_error)?;
        candidate.verify_sources(&self.captures).map_err(contract_error)?;
        candidate.verify_contracts(&draft.contracts).map_err(contract_error)?;
        if candidate.bindings.len()!=self.occurrences.len(){return Err(fail("임시 원자 대응 개수가 다릅니다"))}
        for (binding,occurrence) in candidate.bindings.iter().zip(&self.occurrences){
            if binding.source!=0||binding.start!=occurrence.span.start as u64||binding.end!=occurrence.span.end as u64
                ||candidate.registries[binding.registry].entries[binding.entry].label!=occurrence.label {
                return Err(fail("임시 원자 원문 대응이 다릅니다"))
            }
        }
        if draft.schemas.iter().any(|schema|schema.module!=self.module){return Err(fail("다른 임시 모듈의 선언입니다"))}
        for blob in &draft.artifacts {if blob.sha!=<[u8;32]>::from(Sha256::digest(&blob.bytes)){return Err(fail("임시 등록 바이트가 다릅니다"))}}
        Ok(())
    }
}

/// Constructor is the actual raw-source owner, not serde or caller IDs.
pub(crate) struct TemporaryRegisteredSource {
    activation:Arc<DirectSourceActivation>,registration:RegistrationDraft,digest:[u8;32],
}
impl TemporaryRegisteredSource {
    pub(crate) fn prepare(program:&crate::ddn_runtime::DdnProgram)->Result<Option<Self>,String>{
        let canonical=program.canonical_program();
        let needed=canonical.items.iter().any(|TopLevelItem::SeedDef(seed)|matches!(seed.seed_kind,SeedKind::Schema(_)))
            ||canonical.origin.atom_value_occurrences.values().any(|o|matches!(o.role,ddonirang_lang::ast::AtomValueRole::LocalValue))
            ||super::resolved_supply::requires_resolved_program(canonical);
        if !needed{return Ok(None)}
        let capture=program.sealed_source_capture().ok_or_else(||fail("임시 등록에는 실제 원문 봉인이 필요합니다"))?.clone();
        let stripped=crate::file_meta::split_file_meta(capture.text()).stripped;
        if canonical.origin.source!=stripped{return Err(fail("임시 원문의 실제 해석 경계를 확인할 수 없습니다"))}
        let removed_header_bytes=capture.bytes().len().checked_sub(stripped.len()).ok_or_else(||fail("원문 머리 범위 오류"))?;
        let mut module=[0;32];getrandom::fill(&mut module).map_err(|_|fail("임시 모듈 정체를 준비하지 못했습니다"))?;
        if module==[0;32]{return Err(fail("임시 모듈 정체가 비어 있습니다. 재발급하지 않습니다"));}
        let mut occurrences=Vec::new();
        for original in canonical.origin.atom_value_occurrences.values().filter(|o|matches!(o.role,ddonirang_lang::ast::AtomValueRole::LocalValue)){
            let mut occurrence=original.clone();
            occurrence.span=Span{start:occurrence.span.start.checked_add(removed_header_bytes).ok_or_else(||fail("원문 범위 초과"))?,
                end:occurrence.span.end.checked_add(removed_header_bytes).ok_or_else(||fail("원문 범위 초과"))?};
            occurrences.push(occurrence);
        }
        occurrences.sort_by_key(|o|(o.span.start,o.span.end));
        // Reuse actual compiled declarations and map metadata spans back to
        // the retained raw capture. This is not source rewriting/reparsing.
        let declarations=canonical.items.iter().map(|TopLevelItem::SeedDef(seed)|{
            let mut seed=seed.clone();seed.span=Span{start:seed.span.start.checked_add(removed_header_bytes).ok_or_else(||fail("선언 범위 초과"))?,end:seed.span.end.checked_add(removed_header_bytes).ok_or_else(||fail("선언 범위 초과"))?};Ok(seed)
        }).collect::<Result<Vec<_>,String>>()?;
        let activation=Arc::new(DirectSourceActivation{scope:format!("temporary:{}",hex::encode(module)),module,
            captures:vec![("source.ddn".into(),capture)],occurrences,removed_header_bytes,declarations,checked_program:Arc::new(canonical.clone())});
        let mut registration=prepare_from_source_view(activation.as_ref(),None,&mut |id|{
            getrandom::fill(id).map_err(|_|fail("임시 등록 정체를 준비하지 못했습니다"))
        })?;
        let view=super::resolved_supply::Source::temporary(activation.clone())?;
        if super::resolved_supply::ResolvedCatalog::needed(&view)? {
            registration.resolved=Some(Arc::new(super::resolved_supply::ResolvedCatalog::issue(&view,&registration,&mut |id|{
                getrandom::fill(id).map_err(|_|fail("임시 고름씨 등록 정체를 준비하지 못했습니다"))
            })?));
        }
        let mut result=Self{activation,registration,digest:[0;32]};
        result.digest=result.context_digest()?;
        result.bind_constructors(canonical)?;result.bind_atoms(canonical)?;
        Ok(Some(result))
    }
    pub(crate) fn bind_constructors(&self,program:&CanonProgram)->Result<BTreeMap<NodeId,ResolvedConstructorV1>,String>{
        self.registration.bind_constructors_from(self.activation.as_ref(),program)
    }
    pub(crate) fn bind_atoms(&self,program:&CanonProgram)->Result<super::BoundAtomValuesV1,String>{
        self.registration.bind_atom_values_from(self.activation.as_ref(),program)
    }
    pub(crate) fn catalog(&self,program:&CanonProgram)->Result<SavedNominalCatalogV1,String>{
        let mut schemas=BTreeMap::new();
        for schema in self.bind_constructors(program)?.into_values(){if schemas.insert(schema.name_type,schema).is_some(){return Err(fail("임시 명목 정체 중복"))}}
        Ok(SavedNominalCatalogV1{schemas})
    }
    pub(crate) fn contracts(&self)->&[RecordedAtomContract]{&self.registration.contracts}
    pub(crate) fn bind_resolved_program(&self,program:&mut CanonProgram)->Result<(),String> {
        if let Some(catalog)=self.registration.resolved.as_deref() {
            let source=super::resolved_supply::Source::temporary(self.activation.clone())?;
            let checked=super::resolved_supply::ResolvedCatalog::restore(&source,&self.registration,catalog.bytes())?;
            checked.bind_program(&source,program)?;
        }
        Ok(())
    }
    pub(crate) fn digest(&self)->[u8;32]{self.digest}
    fn context_digest(&self)->Result<[u8;32],String>{
        let mut bytes=b"DDN-VC04-TEMPORARY-SOURCE-CONTEXT-V1\0".to_vec();bytes.extend(1u16.to_be_bytes());
        bytes.extend((self.activation.captures.len() as u32).to_be_bytes());
        let blob=|bytes:&mut Vec<u8>,value:&[u8]|->Result<(),String>{
            let n=u32::try_from(value.len()).map_err(|_|fail("임시 문맥 크기 한도"))?;bytes.extend(n.to_be_bytes());bytes.extend(value);Ok(())};
        for (path,capture) in &self.activation.captures {
            blob(&mut bytes,path.as_bytes())?;bytes.extend(capture.identity());bytes.extend((capture.bytes().len() as u64).to_be_bytes());
            bytes.extend(capture.content_digest());bytes.extend(self.activation.module);
        }
        bytes.extend((self.registration.descriptor.len() as u64).to_be_bytes());bytes.extend(Sha256::digest(&self.registration.descriptor));
        if let Some(catalog)=self.registration.resolved.as_deref() {
            blob(&mut bytes,b"resolved-supply-catalog.r3.3")?;
            blob(&mut bytes,catalog.bytes())?;
            blob(&mut bytes,&super::resolved_supply::builtin_manifest_bytes()?)?;
        }
        let candidate=wire::Candidate::parse(&self.registration.descriptor).map_err(contract_error)?;
        bytes.extend((candidate.registries.len() as u32).to_be_bytes());
        for registry in &candidate.registries {
            let contract=self.registration.contracts.iter().find(|c|c.custody().snapshot_identity.as_slice()==registry.snapshot).ok_or_else(||fail("임시 원자 계약 누락"))?;
            bytes.extend(registry.identity);bytes.extend(contract.custody().byte_length.to_be_bytes());bytes.extend(contract.custody().sha256);
        }
        blob(&mut bytes,b"v1-core-v25")?;Ok(Sha256::digest(bytes).into())
    }
}
