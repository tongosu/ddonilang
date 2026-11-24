//! World-free original nominal source plus checked retained registration.
//! This carrier is neither a Native grant, Start/PESI nor a saved world.
use sha2::{Digest,Sha256};
use crate::authored_project::registration::nominal_source::RegisteredNominalSource;
pub(crate) const SCHEMA:&str="ddn.nominal_source_only_artifact.v1";
const MAX:usize=8_000_000;
#[derive(Clone,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NominalSource {
 schema:String,pub(crate) project_hex:String,pub(crate) parent_hex:String,pub(crate) revision_hex:String,
 pub(crate) profile:String,pub(crate) authored_project_json_utf8:String,pub(crate) registration_material_json_utf8:String,integrity_hex:String,
}
pub(crate) fn checked_project(raw:&str)->Result<Option<crate::authored_project::ProjectArtifact>,String>{RegisteredNominalSource::candidate_project(raw)}
impl NominalSource {
 pub(crate) fn prepare(raw:&str,previous:Option<&Self>,parent:&str,revision:&str)->Result<Self,String>{
  let p=checked_project(raw)?.ok_or("E_NOMINAL_SOURCE_REQUIRED")?;
  if previous.is_some_and(|old|old.project_hex!=p.authored_identity.logical_dependency_id.as_str()||old.revision_hex!=parent||old.profile!=p.source_profile){return Err("E_NOMINAL_SOURCE_PARENT".into());}
  if previous.is_none()&&!parent.is_empty(){return Err("E_NOMINAL_SOURCE_PARENT_REQUIRED".into());}
  let old=previous.map(Self::fresh_context).transpose()?;
  let registered=RegisteredNominalSource::prepare(raw,old.as_ref(),&mut |id|getrandom::fill(id).map_err(|_|"E_NOMINAL_SOURCE_ENTROPY".into()))?;
  Self::from_parts(p,raw,&registered.material()?,parent,revision)
 }
 fn from_parts(p:crate::authored_project::ProjectArtifact,raw:&str,material:&str,parent:&str,revision:&str)->Result<Self,String>{
  let id=|s:&str|s.len()==32&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b));
  if !id(revision)||(!parent.is_empty()&&!id(parent))||parent==revision{return Err("E_NOMINAL_SOURCE_REVISION".into());}
  RegisteredNominalSource::check_material_custody(raw,material)?;
  let mut result=Self{schema:SCHEMA.into(),project_hex:p.authored_identity.logical_dependency_id.as_str().into(),parent_hex:parent.into(),revision_hex:revision.into(),profile:p.source_profile,
   authored_project_json_utf8:raw.into(),registration_material_json_utf8:material.into(),integrity_hex:String::new()};
  result.integrity_hex=hex::encode(Sha256::digest(result.encode()?.as_bytes()));if result.encode()?.len()>MAX{return Err("E_NOMINAL_SOURCE_LIMIT".into());}Ok(result)
 }
 pub(crate) fn encode(&self)->Result<String,String>{serde_json::to_string(self).map_err(|_|"E_NOMINAL_SOURCE_ENCODING".into())}
 pub(crate) fn decode(raw:&str)->Result<Self,String>{
  if raw.len()>MAX||raw.starts_with('\u{feff}'){return Err("E_NOMINAL_SOURCE_LIMIT".into());}
  let r:Self=serde_json::from_str(raw).map_err(|_|"E_NOMINAL_SOURCE_FIELDS")?;
  if r.schema!=SCHEMA||r.encode()?!=raw{return Err("E_NOMINAL_SOURCE_CANONICAL".into());}
  let p=super::code_source::custody_project(&r.authored_project_json_utf8)?.ok_or("E_NOMINAL_SOURCE_REQUIRED")?;
  let checked=Self::from_parts(p,&r.authored_project_json_utf8,&r.registration_material_json_utf8,&r.parent_hex,&r.revision_hex)?;
  if checked.encode()?!=raw{return Err("E_NOMINAL_SOURCE_INTEGRITY".into());}Ok(r)
 }
 pub(crate) fn fresh_context(&self)->Result<RegisteredNominalSource,String>{RegisteredNominalSource::restore(&self.authored_project_json_utf8,&self.registration_material_json_utf8)}
 pub(crate) fn anchor(&self)->Result<Anchor,String>{Ok(Anchor{revision:hex::decode(&self.revision_hex).map_err(|_|"E_NOMINAL_SOURCE_REVISION")?.try_into().map_err(|_|"E_NOMINAL_SOURCE_REVISION")?,integrity:hex::decode(&self.integrity_hex).map_err(|_|"E_NOMINAL_SOURCE_INTEGRITY")?.try_into().map_err(|_|"E_NOMINAL_SOURCE_INTEGRITY")?})}
 pub(crate) fn project(&self)->Result<[u8;32],String>{hex::decode(&self.project_hex).map_err(|_|"E_NOMINAL_SOURCE_PROJECT")?.try_into().map_err(|_|"E_NOMINAL_SOURCE_PROJECT".into())}
}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Anchor{pub(crate) revision:[u8;16],pub(crate) integrity:[u8;32]}
#[cfg(test)]pub(crate) mod tests{
 use super::*;
 pub(crate) const SOURCE:&str=include_str!("../../../tests/fixtures/명목원문_고름씨_입문17.또니");
 pub(crate) fn artifact(text:&str)->String{crate::authored_project::operate("create",&serde_json::json!({"source_profile":"v1-core-v25","module_name":"앱","source_snapshot":{"root_path":null,"entry_path":"작품.또니","files":[{"path":"작품.또니","text":text}]}}).to_string()).unwrap()}
 pub(crate) fn edited(raw:&str,text:&str)->String{crate::authored_project::operate("update",&serde_json::json!({"original_json":raw,"source_snapshot":{"root_path":null,"entry_path":"작품.또니","files":[{"path":"작품.또니","text":text}]}}).to_string()).unwrap()}
 #[test]fn nominal_source_carrier_retains_checked_registration_and_refuses_old_carriers(){
  let a=artifact(SOURCE);let first=NominalSource::prepare(&a,None,"",&"01".repeat(16)).unwrap();let encoded=first.encode().unwrap();let read=NominalSource::decode(&encoded).unwrap();assert_eq!(read.encode().unwrap(),encoded);
  assert!(super::super::code_source::CodeSource::decode(&encoded).is_err());assert!(super::super::model_source::ModelSource::decode(&encoded).is_err());assert!(super::super::source_only::PreparedSourceOnlyRevision::from_artifact(&encoded).is_err());
  let b=edited(&a,&SOURCE.replace("접수 <- 거짓으로 구매.","접수 <- 참으로 구매."));let second=NominalSource::prepare(&b,Some(&read),&first.revision_hex,&"02".repeat(16)).unwrap();assert_ne!(second.registration_material_json_utf8,first.registration_material_json_utf8);second.fresh_context().unwrap();assert!(first.fresh_context().is_ok());
  let mut bad:serde_json::Value=serde_json::from_str(&second.encode().unwrap()).unwrap();bad["registration_material_json_utf8"]=serde_json::json!(first.registration_material_json_utf8);assert!(NominalSource::decode(&serde_json::to_string(&bad).unwrap()).is_err());
  assert!(NominalSource::prepare(&b,Some(&read),"",&"03".repeat(16)).is_err());
 }
}
