//! Checked rootless nominal source owner. No saved-world/Start/PESI stand-in.
use super::*;
use crate::source_snapshot_admission::{SourceCapture,EpochSupply};
use ddonirang_lang::{CanonProgram,Span,SeedKind,SchemaCategory,TopLevelItem};
use std::sync::Arc;
#[derive(Debug)]
pub(super) struct RootlessNominalSource {
 project:crate::authored_project::ProjectArtifact,module:[u8;32],captures:Vec<(String,SourceCapture)>,
 occurrences:Vec<ddonirang_lang::ast::AtomValueOccurrence>,declarations:Vec<ddonirang_lang::SeedDef>,
 removed_header_bytes:usize,checked_program:Arc<CanonProgram>,
}
impl RootlessNominalSource {
 fn admit(raw:&str,record:Option<(&str,[u8;32])>)->Result<Self,String>{
  let project=crate::unit_wire::project_runtime_attachment::code_source::custody_project(raw)?.ok_or_else(||fail("명목 단독 원문 작품이 필요합니다"))?;
  if project.schema==crate::authored_project::SCHEMA_V3{return Err(fail("첫 명목 보관은 DDL 작품을 받지 않습니다"));}
  let file=&project.source_snapshot.files[0];
  let capture=match record{Some((record,id))=>crate::source_snapshot_admission::restore_recorded_v25_capture(record,file.text.as_bytes(),&id)?,None=>crate::source_snapshot_admission::capture_declared_v25(file.text.as_bytes())?};
  let program=crate::ddn_runtime::DdnProgram::from_rootless_nominal_capture(capture.clone(),&file.path)?;
  let canonical=program.canonical_program();
  if canonical.items.iter().any(|TopLevelItem::SeedDef(seed)|matches!(seed.seed_kind,SeedKind::Schema(SchemaCategory::Subject))||!seed.type_params.is_empty()){
   return Err(fail("첫 명목 보관은 임자·generic 선언을 받지 않습니다"));
  }
  let has_nominal=canonical.items.iter().any(|TopLevelItem::SeedDef(seed)|matches!(seed.seed_kind,SeedKind::Schema(SchemaCategory::NominalValue)))||super::requires_resolved_program(canonical);
  if !has_nominal{return Err(fail("지원하는 이름씨·고름씨 선언이 없습니다"));}
  let module=hex::decode(project.authored_identity.modules[0].id.as_str()).map_err(|_|fail("모듈 정체 오류"))?.try_into().map_err(|_|fail("모듈 정체 길이 오류"))?;
  let meta=crate::file_meta::split_file_meta(&file.text);let removed_header_bytes=file.text.len().checked_sub(meta.stripped.len()).ok_or_else(||fail("원문 머리말 범위 오류"))?;
  let occurrences=canonical.origin.atom_value_occurrences.values().map(|item|{
   let mut item=item.clone();item.span.start=item.span.start.checked_add(removed_header_bytes).ok_or_else(||fail("원문 위치 초과"))?;item.span.end=item.span.end.checked_add(removed_header_bytes).ok_or_else(||fail("원문 위치 초과"))?;Ok(item)
  }).collect::<Result<Vec<_>,String>>()?;
  let declarations=canonical.items.iter().map(|TopLevelItem::SeedDef(seed)|seed.clone()).collect();
  let path=file.path.clone();
  Ok(Self{project,module,captures:vec![(path,capture)],occurrences,declarations,removed_header_bytes,checked_program:Arc::new(canonical.clone())})
 }
 pub(super) fn checked_program(&self)->&CanonProgram{&self.checked_program}
 pub(super) fn module_id(&self)->[u8;32]{self.module}
}
impl RegistrationSourceView for RootlessNominalSource {
 fn scope(&self)->&str{self.project.authored_identity.logical_dependency_id.as_str()}
 fn captures(&self)->&[(String,SourceCapture)]{&self.captures}
 fn module(&self,path:&str)->Result<Option<[u8;32]>,String>{if path!=self.captures[0].0{return Err(fail("다른 원문 소유 모듈입니다"));}Ok(Some(self.module))}
 fn occurrences(&self)->Vec<(&str,&ddonirang_lang::ast::AtomValueOccurrence)>{self.occurrences.iter().map(|o|(self.captures[0].0.as_str(),o)).collect()}
 fn declarations(&self,path:&str,capture:&SourceCapture)->Result<Vec<ddonirang_lang::SeedDef>,String>{if path!=self.captures[0].0||capture.identity()!=self.captures[0].1.identity(){return Err(fail("다른 선언 원문 봉인입니다"));}Ok(self.declarations.clone())}
 fn original_span(&self,span:Span)->Result<(String,Span),String>{
  let start=span.start.checked_add(self.removed_header_bytes).ok_or_else(||fail("원문 범위 초과"))?;let end=span.end.checked_add(self.removed_header_bytes).ok_or_else(||fail("원문 범위 초과"))?;let text=self.captures[0].1.text();
  if start>end||!text.is_char_boundary(start)||!text.is_char_boundary(end){return Err(fail("원문 범위 오류"));}Ok((self.captures[0].0.clone(),Span{start,end}))
 }
 fn verify(&self,draft:&RegistrationDraft)->Result<(),String>{
  if draft.project!=self.scope(){return Err(fail("다른 작품의 명목 등록입니다"));}
  let epoch=EpochSupply::for_declared_epoch(ddonirang_lang::SourceEpoch::V25);for (_,c) in &self.captures{crate::source_snapshot_admission::replay(&epoch,c).map_err(|_|fail("원문 봉인이 다릅니다"))?;}
  let candidate=wire::Candidate::parse(&draft.descriptor).map_err(contract_error)?;candidate.verify_sources(&self.captures).map_err(contract_error)?;candidate.verify_contracts(&draft.contracts).map_err(contract_error)?;
  if candidate.bindings.len()!=self.occurrences.len(){return Err(fail("원자 대응 개수 오류"));}
  for (binding,occurrence) in candidate.bindings.iter().zip(&self.occurrences){if binding.source!=0||binding.start!=occurrence.span.start as u64||binding.end!=occurrence.span.end as u64||candidate.registries[binding.registry].entries[binding.entry].label!=occurrence.label{return Err(fail("원자 원문 대응 오류"));}}
  if draft.schemas.iter().any(|schema|schema.module!=self.module){return Err(fail("다른 모듈의 명목 선언입니다"));}
  for blob in &draft.artifacts{if blob.sha!=<[u8;32]>::from(Sha256::digest(&blob.bytes)){return Err(fail("명목 등록 원자료 hash가 다릅니다"));}}Ok(())
 }
}
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Material{schema:String,capture_identity:String,capture_record:String,records:Vec<RecordedRegistrationBlob>}
/// Source-owner-checked data, neither a Native grant nor a world capability.
pub(crate) struct RegisteredNominalSource {pub(super) activation:Arc<RootlessNominalSource>,pub(super) registration:RegistrationDraft}
impl RegisteredNominalSource {
 pub(crate) fn candidate_project(raw:&str)->Result<Option<crate::authored_project::ProjectArtifact>,String>{
  let Some(_) = crate::unit_wire::project_runtime_attachment::code_source::custody_project(raw)? else{return Ok(None)};
  if crate::authored_project::operate("source_probe",&serde_json::json!({"artifact_json_utf8":raw}).to_string())?!="true"{return Ok(None);}
  Ok(Some(RootlessNominalSource::admit(raw,None)?.project))
 }
 /// Custody data verification only. Current grammar/type/call admission remains
 /// in restore; old heads must not become unreadable through a new parser.
 pub(crate) fn check_material_custody(raw:&str,material:&str)->Result<(),String>{
  let p=crate::unit_wire::project_runtime_attachment::code_source::custody_project(raw)?.ok_or_else(||fail("명목 단독 원문 보관 구조가 아닙니다"))?;
  if p.schema==crate::authored_project::SCHEMA_V3||material.len()>6_000_000||material.starts_with('\u{feff}'){return Err(fail("명목 보관 원자료 범위 오류"));}
  let m:Material=serde_json::from_str(material).map_err(|_|fail("명목 보관 원자료 필드 오류"))?;
  if m.schema!="ddn.rootless_nominal_registration.v1"||serde_json::to_string(&m).map_err(|_|fail("명목 정본 오류"))?!=material{return Err(fail("명목 보관 원자료 판본·정본 오류"));}
  let identity:[u8;32]=hex::decode(&m.capture_identity).map_err(|_|fail("봉인 표기 오류"))?.try_into().map_err(|_|fail("봉인 길이 오류"))?;
  if hex::encode(identity)!=m.capture_identity{return Err(fail("봉인 정본 표기 오류"));}
  let file=&p.source_snapshot.files[0];let capture=crate::source_snapshot_admission::restore_recorded_v25_capture(&m.capture_record,file.text.as_bytes(),&identity)?;
  let draft=decode_recorded_candidate_parts(&p,m.records,false)?;
  let module:[u8;32]=hex::decode(p.authored_identity.modules[0].id.as_str()).map_err(|_|fail("모듈 표기 오류"))?.try_into().map_err(|_|fail("모듈 길이 오류"))?;
  if draft.schemas.iter().any(|s|s.module!=module||s.category!=2){return Err(fail("다른 소유자·종류의 명목 보관 자료입니다"));}
  let candidate=wire::Candidate::parse(&draft.descriptor).map_err(contract_error)?;candidate.verify_sources(&[(file.path.clone(),capture)]).map_err(contract_error)?;candidate.verify_contracts(&draft.contracts).map_err(contract_error)?;
  for blob in draft.artifacts.iter().filter(|b|b.role==Some(resolved_supply::RESOLVED_ROLE)){
   let value:serde_json::Value=serde_json::from_slice(blob.bytes()).map_err(|_|fail("고름씨 보관 원자료 오류"))?;
   if value["schema"]!="ddn.resolved-supply-catalog.r3.3"||value["mode"]!="nominal-source"||value["scope"]!=p.authored_identity.logical_dependency_id.as_str(){return Err(fail("다른 역할·작품의 고름씨 원자료입니다"));}
  }
  Ok(())
 }
 pub(crate) fn prepare(raw:&str,previous:Option<&Self>,entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<Self,String>{
  let activation=Arc::new(RootlessNominalSource::admit(raw,None)?);
  if previous.is_some_and(|old|old.activation.scope()!=activation.scope()||old.activation.module!=activation.module){return Err(fail("다른 작품·모듈의 등록 부모입니다"));}
  let mut registration=prepare_from_source_view(activation.as_ref(),previous.map(|old|&old.registration),entropy)?;
  let view=resolved_supply::Source::nominal(activation.clone());
  if resolved_supply::ResolvedCatalog::needed(&view)?{
   let resolved=match previous.and_then(|old|old.registration.resolved.as_deref()){
    Some(old)=>{let previous=previous.unwrap();let old_view=resolved_supply::Source::nominal(previous.activation.clone());
        resolved_supply::ResolvedCatalog::rebind_nominal_revision(&old_view,&previous.registration,&view,&registration,old.bytes())?},
    None=>resolved_supply::ResolvedCatalog::issue(&view,&registration,entropy)?,
   };Self::install_resolved(&mut registration,resolved)?;
  }
  let result=Self{activation,registration};result.verify_program()?;Ok(result)
 }
 fn install_resolved(draft:&mut RegistrationDraft,resolved:resolved_supply::ResolvedCatalog)->Result<(),String>{
  let bytes=resolved.bytes().to_vec();if draft.artifacts.iter().any(|b|b.path==resolved_supply::RESOLVED_PATH){return Err(fail("고름씨 등록 자료 중복"));}
  draft.artifacts.push(RegistrationBlob{path:resolved_supply::RESOLVED_PATH.into(),role:Some(resolved_supply::RESOLVED_ROLE),sha:Sha256::digest(&bytes).into(),bytes});draft.artifacts.sort_by(|a,b|a.path.cmp(&b.path));draft.resolved=Some(Arc::new(resolved));Ok(())
 }
 pub(crate) fn material(&self)->Result<String,String>{serde_json::to_string(&Material{schema:"ddn.rootless_nominal_registration.v1".into(),capture_identity:hex::encode(self.activation.captures[0].1.identity()),capture_record:self.activation.captures[0].1.recorded_json()?,records:self.registration.recorded_blobs()}).map_err(|_|fail("명목 등록 원자료 인코딩 오류"))}
 pub(crate) fn restore(raw:&str,material:&str)->Result<Self,String>{
  if material.len()>6_000_000||material.starts_with('\u{feff}'){return Err(fail("명목 등록 원자료 한도"));}
  let m:Material=serde_json::from_str(material).map_err(|_|fail("명목 등록 원자료 필드 오류"))?;
  if m.schema!="ddn.rootless_nominal_registration.v1"||serde_json::to_string(&m).map_err(|_|fail("명목 정본 오류"))?!=material{return Err(fail("명목 등록 원자료 판본·정본 오류"));}
  let identity=hex::decode(&m.capture_identity).map_err(|_|fail("원문 봉인 표기 오류"))?.try_into().map_err(|_|fail("원문 봉인 길이 오류"))?;
  let activation=Arc::new(RootlessNominalSource::admit(raw,Some((&m.capture_record,identity)))?);
  let mut registration=decode_recorded_candidate_parts(&activation.project,m.records,false)?;activation.verify(&registration)?;
  let view=resolved_supply::Source::nominal(activation.clone());let rows=registration.artifacts.iter().filter(|b|b.role==Some(resolved_supply::RESOLVED_ROLE)).collect::<Vec<_>>();
  if resolved_supply::ResolvedCatalog::needed(&view)?{if rows.len()!=1{return Err(fail("고름씨 등록 자료 누락"));}registration.resolved=Some(Arc::new(resolved_supply::ResolvedCatalog::restore(&view,&registration,rows[0].bytes())?));}
  else if !rows.is_empty(){return Err(fail("여분의 고름씨 등록 자료"));}
  let result=Self{activation,registration};result.verify_program()?;Ok(result)
 }
 fn verify_program(&self)->Result<(),String>{self.activation.verify(&self.registration)?;self.registration.bind_constructors_from(self.activation.as_ref(),self.activation.checked_program())?;Ok(())}
 pub(crate) fn source(&self)->&str{self.activation.captures[0].1.text()}
 pub(crate) fn path(&self)->&str{&self.activation.captures[0].0}
 pub(crate) fn capture(&self)->SourceCapture{self.activation.captures[0].1.clone()}
 pub(crate) fn bind_resolved_program(&self,program:&mut CanonProgram)->Result<(),String>{if let Some(c)=&self.registration.resolved{let source=resolved_supply::Source::nominal(self.activation.clone());let checked=resolved_supply::ResolvedCatalog::restore(&source,&self.registration,c.bytes())?;checked.bind_program(&source,program)?;}Ok(())}
 pub(crate) fn bind_constructors(&self,program:&CanonProgram)->Result<BTreeMap<ddonirang_lang::NodeId,ResolvedConstructorV1>,String>{self.registration.bind_constructors_from(self.activation.as_ref(),program)}
 pub(crate) fn bind_atoms(&self,program:&CanonProgram)->Result<BoundAtomValuesV1,String>{self.registration.bind_atom_values_from(self.activation.as_ref(),program)}
 pub(crate) fn catalog(&self,program:&CanonProgram)->Result<SavedNominalCatalogV1,String>{let mut schemas=BTreeMap::new();for schema in self.bind_constructors(program)?.into_values(){if schemas.insert(schema.name_type,schema).is_some(){return Err(fail("명목 정체 중복"));}}Ok(SavedNominalCatalogV1{schemas})}
 pub(crate) fn contracts(&self)->&[RecordedAtomContract]{&self.registration.contracts}
}
#[cfg(test)]mod tests{
 use super::*;
 fn artifact(source:&str)->String{crate::authored_project::operate("create",&serde_json::json!({"source_profile":"v1-core-v25","module_name":"앱","source_snapshot":{"root_path":null,"entry_path":"작품.또니","files":[{"path":"작품.또니","text":source}]}}).to_string()).unwrap()}
 #[test]fn nominal_rootless_actual_issuer_and_recorded_restore_keep_material(){
  for source in [include_str!("../../../tests/fixtures/명목원문_고름씨_입문17.또니"),include_str!("../../../tests/fixtures/명목원문_이름씨_입문20.또니")]{
   let raw=artifact(source);let mut issued=0;let first=RegisteredNominalSource::prepare(&raw,None,&mut |id|{issued+=1;getrandom::fill(id).map_err(|_|"entropy".into())}).unwrap();assert!(issued>0);let material=first.material().unwrap();let restored=RegisteredNominalSource::restore(&raw,&material).unwrap();assert_eq!(restored.material().unwrap(),material);assert!(restored.activation.project.source_snapshot.root_path.is_none());
   assert!(RegisteredNominalSource::restore(&raw,&material.replace("rootless_nominal_registration.v1","rootless_nominal_registration.v9")).is_err());
  }
 }
 #[test]fn nominal_rootless_revised_source_keeps_ids_and_executes_after_recorded_restore(){
  use ddonirang_core::{Fixed64,InputSnapshot,InputSource};use ddonirang_core::platform::NuriWorld;
  for (source,old_text,new_text,expected) in [(include_str!("../../../tests/fixtures/명목원문_고름씨_입문17.또니"),"접수 <- 거짓으로 구매.","접수 <- 참으로 구매.",vec!["3"]),(include_str!("../../../tests/fixtures/명목원문_이름씨_입문20.또니"),"점수=8","점수=9",vec!["연두","9"])]{
   let raw=artifact(source);let mut entropy=|id:&mut [u8;32]|getrandom::fill(id).map_err(|_|"entropy".into());let first=RegisteredNominalSource::prepare(&raw,None,&mut entropy).unwrap();
   let changed=source.replace(old_text,new_text);let next=crate::authored_project::operate("update",&serde_json::json!({"original_json":raw,"source_snapshot":{"root_path":null,"entry_path":"작품.또니","files":[{"path":"작품.또니","text":changed}]}}).to_string()).unwrap();
   let second=RegisteredNominalSource::prepare(&next,Some(&first),&mut |_|Err("existing declaration identities must not be reissued".into())).unwrap();second.registration.verify_retained_from(&first.registration).unwrap();
   let material=second.material().unwrap();let restored=Arc::new(RegisteredNominalSource::restore(&next,&material).unwrap());assert_eq!(restored.material().unwrap(),material);assert!(RegisteredNominalSource::restore(&raw,&material).is_err());
   let program=crate::ddn_runtime::DdnProgram::from_nominal_source_context(restored).unwrap();assert!(!program.has_temporary_registration());
   let input=InputSnapshot{tick_id:0,dt:Fixed64::from_i64(1),keys_pressed:0,last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],frame_source:InputSource::Person,rng_seed:0};
   let result=crate::ddn_runtime::DdnRunner::new(program,"매마디").run_update(&NuriWorld::new(),&input,&std::collections::HashMap::new()).unwrap();assert_eq!(result.display_outputs.iter().map(|r|r.text.as_str()).collect::<Vec<_>>(),expected);
  }
 }

}
