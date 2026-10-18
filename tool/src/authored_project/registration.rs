//! VC04 common authoring producer. These are private, frozen candidates,
//! not execution admission or durable receipts. The transaction owner must
//! retain the same candidate through CAS/readback; restoring never calls this
//! issuer. No JSON `new` flag, spelling hash or AST ordinal issues an ID.
use crate::project_revision_envelope_supply::VerifiedSavedAuthoredProjectV1 as Project;
use crate::registered_atom_contract::{RecordedAtomContract,source_binding as wire};
use std::collections::{BTreeMap,BTreeSet};
use sha2::{Digest,Sha256};
pub(crate) mod temporary;
mod resolved_supply;

// Borrowed source facts, not a saved Project/PESI or an admission constructor.
// Saved and temporary producers retain their distinct private authority.
trait RegistrationSourceView {
    fn scope(&self)->&str;
    fn captures(&self)->&[(String,crate::source_snapshot_admission::SourceCapture)];
    fn module(&self,path:&str)->Result<Option<[u8;32]>,String>;
    fn occurrences(&self)->Vec<(&str,&ddonirang_lang::ast::AtomValueOccurrence)>;
    fn declarations(&self,path:&str,capture:&crate::source_snapshot_admission::SourceCapture)->Result<Vec<ddonirang_lang::SeedDef>,String>{
        Ok(ddonirang_lang::parse_v25_with_mode(capture.text(),path,ddonirang_lang::ParseMode::Strict)
            .map_err(|e|fail(&e.to_string()))?.items.into_iter().map(|ddonirang_lang::TopLevelItem::SeedDef(seed)|seed).collect())
    }
    fn original_span(&self,span:ddonirang_lang::Span)->Result<(String,ddonirang_lang::Span),String>;
    fn verify(&self,draft:&RegistrationDraft)->Result<(),String>;
    fn conflicts(&self,path:&str)->bool {self.captures().iter().any(|(old,_)|old.eq_ignore_ascii_case(path))}
}
struct SavedRegistrationSource<'a> {
    project:&'a crate::authored_project::ProjectArtifact,
    sources:&'a crate::project_revision_envelope_supply::VerifiedSavedV25Sources,
}
impl<'a> SavedRegistrationSource<'a> {
    fn finalized(owned:&'a Project)->Self {Self{project:owned.project(),sources:owned.sources()}}
    fn precanon(owned:&'a crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1)->Self {
        Self{project:owned.project(),sources:owned.sources()}
    }
}
impl RegistrationSourceView for SavedRegistrationSource<'_> {
    fn scope(&self)->&str {self.project.authored_identity.logical_dependency_id.as_str()}
    fn captures(&self)->&[(String,crate::source_snapshot_admission::SourceCapture)] {self.sources.captures()}
    fn module(&self,path:&str)->Result<Option<[u8;32]>,String>{
        let rows:Vec<_>=self.project.authored_identity.modules.iter().filter(|m|m.sources.iter().any(|p|p==path)).collect();
        if rows.is_empty(){return Ok(None)}
        if rows.len()!=1{return Err(fail("원문의 소유 모듈이 유일하지 않습니다"))}
        Ok(Some(hex::decode(rows[0].id.as_str()).map_err(|_|fail("모듈 정체 표기 오류"))?.try_into().map_err(|_|fail("모듈 정체 길이 오류"))?))
    }
    fn occurrences(&self)->Vec<(&str,&ddonirang_lang::ast::AtomValueOccurrence)>{self.sources.atom_value_occurrences().iter().map(|o|(o.path(),o.occurrence())).collect()}
    fn original_span(&self,span:ddonirang_lang::Span)->Result<(String,ddonirang_lang::Span),String>{let (path,span)=self.sources.preparation().original_span(span)?;Ok((path.into(),span))}
    fn verify(&self,draft:&RegistrationDraft)->Result<(),String>{draft.verify_saved_source_parts(self.project,self.sources)}
    fn conflicts(&self,path:&str)->bool{self.project.source_snapshot.files.iter().any(|file|file.path.eq_ignore_ascii_case(path))}
}

#[derive(Clone,Debug,PartialEq,Eq)]
struct Slot { id:[u8;32],label:String,kind:u8,ty:Vec<u8> }
#[derive(Clone,Debug,PartialEq,Eq)]
struct Schema {module:[u8;32],definition:[u8;32],name_type:[u8;32],category:u8,label:String,slots:Vec<Slot>}
#[derive(Clone,Debug)]
struct Atom {id:[u8;32],label:String}
/// Only this producer can construct a draft. It deliberately cannot mint a
/// SourceBoundSavedRuntimeAttachment or an execution capability.
#[derive(Debug)]
pub(crate) struct RegistrationDraft {
    project:String,
    schemas:Vec<Schema>,
    registry:Option<[u8;32]>,
    atoms:Vec<Atom>,
    descriptor:Vec<u8>,
    contracts:Vec<RecordedAtomContract>,
    artifacts:Vec<RegistrationBlob>,
    resolved:Option<std::sync::Arc<resolved_supply::ResolvedCatalog>>,
}
#[derive(Debug)]
pub(crate) struct RegistrationBlob {path:String,role:Option<&'static str>,bytes:Vec<u8>,sha:[u8;32]}
#[derive(Clone,Debug)]
pub(crate) struct ResolvedConstructorV1 {
    pub(crate) category:u8,
    pub(crate) name_type:[u8;32],
    pub(crate) slots:Vec<ResolvedConstructorSlotV1>,
}
#[derive(Clone,Debug)]
pub(crate) struct ResolvedConstructorSlotV1 {
    pub(crate) identity:[u8;32],
    pub(crate) label:String,
    pub(crate) kind:ddonirang_core::platform::RecordFieldType,
}
/// Source-bound catalog data. Only the recorded registration owner builds it;
/// callers cannot deserialize a JSON map into saved-value admission.
#[derive(Debug)]
pub(crate) struct SavedNominalCatalogV1 {
    schemas:BTreeMap<[u8;32],ResolvedConstructorV1>,
}
impl SavedNominalCatalogV1 {
    pub(crate) fn schema(&self,id:&[u8;32])->Option<&ResolvedConstructorV1>{self.schemas.get(id)}
    pub(crate) fn schemas(&self)->impl Iterator<Item=&ResolvedConstructorV1>{self.schemas.values()}
}
/// Product payload transport, not a FRAME2 field or a nominal-value tag.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedRegistrationBlob {logical_path:String,role:Option<String>,byte_length:u64,sha256:String,raw_hex:String}
impl RegistrationBlob {
    pub(crate) fn path(&self)->&str {&self.path}
    pub(crate) fn role(&self)->Option<&str> {self.role}
    pub(crate) fn bytes(&self)->&[u8] {&self.bytes}
    pub(crate) fn sha(&self)->[u8;32] {self.sha}
}
impl RegistrationDraft {
    pub(crate) fn bind_resolved_program(&self,owned:&Project,program:&mut ddonirang_lang::CanonProgram)->Result<(),String> {
        let source=resolved_supply::Source::saved(&owned.precanon_view());
        if let Some(catalog)=self.resolved.as_deref() {
            let checked=resolved_supply::ResolvedCatalog::restore(&source,self,catalog.bytes())?;
            checked.bind_program(&source,program)?;
        } else if resolved_supply::requires_resolved_program(program) {
            return Err("E_RESOLVED_SUPPLY_CATALOG_REQUIRED: 실행 원문의 검증된 등록 자료가 없습니다".into());
        }
        Ok(())
    }
    fn attach_resolved(&mut self,source:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1,
        previous:Option<&RegistrationDraft>,entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<(),String> {
        let view=resolved_supply::Source::saved(source);
        if !resolved_supply::ResolvedCatalog::needed(&view)? {return Ok(());}
        let resolved=if let Some(old)=previous.and_then(|old|old.resolved.as_deref()) {
            // Unchanged source retains IDs. An unproved successor is refused,
            // never repaired by entropy or a spelling-based transfer.
            resolved_supply::ResolvedCatalog::restore(&view,self,old.bytes())?
        } else {resolved_supply::ResolvedCatalog::issue(&view,self,entropy)?};
        if self.artifacts.iter().any(|blob|blob.path==resolved_supply::RESOLVED_PATH)
            ||source.project().source_snapshot.files.iter().any(|file|file.path.eq_ignore_ascii_case(resolved_supply::RESOLVED_PATH)) {
            return Err(fail("고름씨 등록 자료 경로가 중복됩니다"));
        }
        let bytes=resolved.bytes().to_vec();let sha=Sha256::digest(&bytes).into();
        self.artifacts.push(RegistrationBlob{path:resolved_supply::RESOLVED_PATH.into(),role:Some(resolved_supply::RESOLVED_ROLE),bytes,sha});
        let total=self.artifacts.iter().try_fold(0usize,|size,row|size.checked_add(row.bytes.len())).ok_or_else(||fail("등록 자료 합계 한도"))?;
        if total>2_000_000{return Err(fail("등록 자료 합계 한도"));}
        self.artifacts.sort_by(|a,b|a.path.cmp(&b.path));self.resolved=Some(std::sync::Arc::new(resolved));Ok(())
    }
    fn restore_resolved(&mut self,source:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1)->Result<(),String> {
        let source=resolved_supply::Source::saved(source);
        let rows=self.artifacts.iter().filter(|blob|blob.role==Some(resolved_supply::RESOLVED_ROLE)).collect::<Vec<_>>();
        if resolved_supply::ResolvedCatalog::needed(&source)? {
            if rows.len()!=1||rows[0].path!=resolved_supply::RESOLVED_PATH {return Err("E_RESOLVED_SUPPLY_CATALOG_REQUIRED: 이 원문에는 R3.2 등록 자료가 필요합니다. 자동 발급하지 않습니다".into());}
            self.resolved=Some(std::sync::Arc::new(resolved_supply::ResolvedCatalog::restore(&source,self,rows[0].bytes())?));
        } else if !rows.is_empty(){return Err(fail("원문에 없는 여분의 고름씨 등록 자료입니다"));}
        Ok(())
    }
    pub(crate) fn resolved_start_material(&self,source:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1)->Result<Option<(String,Vec<u8>,Vec<u8>)>,String> {
        let source=resolved_supply::Source::saved(source);
        match self.resolved.as_deref() {
            Some(resolved)=>{
                let checked=resolved_supply::ResolvedCatalog::restore(&source,self,resolved.bytes())?;
                Ok(Some((checked.checked_canon(&source,self)?,checked.bytes().into(),resolved_supply::builtin_manifest_bytes()?)))
            },
            None if resolved_supply::ResolvedCatalog::needed(&source)?=>Err("E_RESOLVED_SUPPLY_CATALOG_REQUIRED: 원문 등록 자료가 없습니다".into()),
            None=>Ok(None),
        }
    }
    pub(crate) fn saved_nominal_catalog(&self,owned:&Project)->Result<SavedNominalCatalogV1,String> {
        let prepared=owned.sources().preparation();
        let program=ddonirang_lang::parse_v25_with_mode(&prepared.source,&prepared.target_path,ddonirang_lang::ParseMode::Strict)
            .map_err(|error|fail(&error.to_string()))?;
        let constructors=self.bind_constructors(owned,&program)?;
        let mut schemas=BTreeMap::new();
        for schema in constructors.into_values() {
            if schemas.insert(schema.name_type,schema).is_some(){return Err(fail("명목 씨 정체가 중복됩니다"));}
        }
        Ok(SavedNominalCatalogV1{schemas})
    }
    /// Resolve linked constructor nodes through the actual original source
    /// map and recorded declaration span, never through an ambient label map.
    pub(crate) fn bind_atom_values(&self,owned:&Project,program:&ddonirang_lang::CanonProgram)
        ->Result<BTreeMap<ddonirang_lang::NodeId,ddonirang_core::platform::RegisteredAtomValueV1>,String>{
        self.bind_atom_values_from(&SavedRegistrationSource::finalized(owned),program)
    }
    fn bind_atom_values_from(&self,source:&impl RegistrationSourceView,program:&ddonirang_lang::CanonProgram)
        ->Result<BTreeMap<ddonirang_lang::NodeId,ddonirang_core::platform::RegisteredAtomValueV1>,String>{
        source.verify(self)?;
        let candidate=wire::Candidate::parse(&self.descriptor).map_err(contract_error)?;
        let mut bindings=BTreeMap::new();
        for binding in &candidate.bindings {
            let key=(candidate.sources[binding.source].path.to_owned(),binding.start,binding.end);
            if bindings.insert(key,binding).is_some(){return Err(fail("한 원자 원문에 중복 대응이 있습니다"));}
        }
        let mut values=BTreeMap::new();
        for (node,occurrence) in &program.origin.atom_value_occurrences {
            if !matches!(occurrence.role,ddonirang_lang::ast::AtomValueRole::LocalValue){continue;}
            let (path,span)=source.original_span(occurrence.span)?;
            let binding=bindings.get(&(path.to_owned(),span.start as u64,span.end as u64))
                .ok_or_else(||fail("실제 원자 값 원문의 등록 결속이 없습니다"))?;
            let registry=&candidate.registries[binding.registry];let entry=&registry.entries[binding.entry];
            if entry.label!=occurrence.label{return Err(fail("등록 원자 원문 이름이 다릅니다"));}
            let contract=self.contracts.iter().find(|contract|contract.custody().snapshot_identity.as_slice()==registry.snapshot)
                .ok_or_else(||fail("기록된 원자 계약이 없습니다"))?;
            let atom=contract.admitted_value(registry.identity,entry.identity).map_err(contract_error)?;
            values.insert(*node,atom);
        }
        Ok(values)
    }
    pub(crate) fn bind_constructors(&self,owned:&Project,program:&ddonirang_lang::CanonProgram)
        ->Result<BTreeMap<ddonirang_lang::NodeId,ResolvedConstructorV1>,String> {
        self.bind_constructors_from(&SavedRegistrationSource::finalized(owned),program)
    }
    fn bind_constructors_from(&self,source:&impl RegistrationSourceView,program:&ddonirang_lang::CanonProgram)
        ->Result<BTreeMap<ddonirang_lang::NodeId,ResolvedConstructorV1>,String> {
        use ddonirang_lang::{TopLevelItem,SeedKind,SchemaCategory};
        use ddonirang_core::platform::RecordFieldType as T;
        source.verify(self)?;
        let candidate=wire::Candidate::parse(&self.descriptor).map_err(contract_error)?;
        let mut result=BTreeMap::new();
        for TopLevelItem::SeedDef(seed) in &program.items {
            let SeedKind::Schema(category)=&seed.seed_kind else {continue};
            let tail=candidate.nominal.as_ref().ok_or_else(||fail("명목 선언 catalog가 없습니다"))?;
            let (path,span)=source.original_span(seed.span)?;
            let mut matches=tail.declarations.iter().filter(|decl| {
                candidate.sources[decl.source].path==path && decl.start==span.start as u64 && decl.end==span.end as u64
            });
            let declaration=matches.next().ok_or_else(||fail("실제 원문 선언의 명목 정체를 찾을 수 없습니다"))?;
            if matches.next().is_some(){return Err(fail("원문 선언의 명목 정체가 중복됩니다"));}
            let schema=&tail.schemas[declaration.schema];
            if schema.category!=match category {SchemaCategory::Subject=>1,SchemaCategory::NominalValue=>2}
                || schema.slots.len()!=seed.params.len() {
                return Err(fail("원문 선언의 종류 또는 붙이 집합이 다릅니다"));
            }
            let mut slots=Vec::new();
            for slot in &schema.slots {
                if !seed.params.iter().any(|pin|pin.pin_name==slot.label){return Err(fail("원문 붙이 소유자가 다릅니다"));}
                let kind=if slot.kind==1 {
                    let target=tail.schemas.iter().find(|row|row.name_type==slot.resolved_type).ok_or_else(||fail("붙이의 명목 씨가 없습니다"))?;
                    T::ResolvedNominalV1{category:target.category,name_type_identity:target.name_type.try_into().map_err(|_|fail("씨 정체 길이 오류"))?}
                } else {crate::unit_wire::decode_recorded_field_type(slot.resolved_type).map_err(|_|fail("붙이의 해소된 씨가 다릅니다"))?};
                slots.push(ResolvedConstructorSlotV1{identity:slot.identity.try_into().map_err(|_|fail("붙이 정체 길이 오류"))?,label:slot.label.into(),kind});
            }
            result.insert(seed.id,ResolvedConstructorV1{category:schema.category,
                name_type:schema.name_type.try_into().map_err(|_|fail("명목 정체 길이 오류"))?,slots});
        }
        Ok(result)
    }
    /// Source successors retain all previously issued identities, including
    /// inactive schemas/atoms. New labels may extend, never replace, them.
    pub(crate) fn verify_retained_from(&self,old:&Self)->Result<(),String>{
        if self.project!=old.project||old.registry.is_some()&&self.registry!=old.registry
            ||old.schemas.iter().any(|prior|!self.schemas.iter().any(|next|next==prior))
            ||old.atoms.iter().any(|prior|!self.atoms.iter().any(|next|next.id==prior.id&&next.label==prior.label)){
            return Err(fail("새 원문 판에서 보유한 등록 정체가 없어지거나 바뀌었습니다"));
        }
        Ok(())
    }
    pub(crate) fn recorded_blobs(&self)->Vec<RecordedRegistrationBlob> {
        self.artifacts.iter().map(|blob|RecordedRegistrationBlob{logical_path:blob.path.clone(),role:blob.role.map(String::from),
            byte_length:blob.bytes.len() as u64,sha256:hex::encode(blob.sha),raw_hex:hex::encode(&blob.bytes)}).collect()
    }
    pub(crate) fn artifacts(&self)->&[RegistrationBlob] {&self.artifacts}
    pub(crate) fn verified_recorded_atom_contracts(&self,owned:&Project)->Result<&[RecordedAtomContract],String>{
        self.verify_source_binding(owned)?;Ok(&self.contracts)
    }
    pub(crate) fn descriptor(&self)->&[u8] {&self.descriptor}
    /// Actual raw captures/contracts are rechecked before a resource owner
    /// may consume this candidate. This check alone is NOT a committed lease.
    pub(crate) fn verify_source_binding(&self,owned:&Project)->Result<(),String> {
        self.verify_saved_source_parts(owned.project(),owned.sources())
    }
    pub(crate) fn verify_precanon_binding(&self,owned:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1)->Result<(),String> {
        self.verify_saved_source_parts(owned.project(),owned.sources())
    }
    fn verify_saved_source_parts(&self,project:&crate::authored_project::ProjectArtifact,
        sources:&crate::project_revision_envelope_supply::VerifiedSavedV25Sources)->Result<(),String> {
        if self.project!=project.authored_identity.logical_dependency_id.as_str(){return Err(fail("다른 작품의 등록 후보입니다"));}
        let parsed=wire::Candidate::parse(&self.descriptor).map_err(contract_error)?;
        parsed.verify_resolved_occurrences(sources).map_err(contract_error)?;
        parsed.verify_contracts(&self.contracts).map_err(contract_error)?;
        for blob in &self.artifacts {
            if blob.sha!=<[u8;32]>::from(Sha256::digest(&blob.bytes)){return Err(fail("등록 원문이 달라졌습니다"));}
        }
        Ok(())
    }
}

/// Rebuild a recorded candidate without entropy. External revision lineage
/// and the recomputed resource/PESI context must still admit it before Run.
pub(crate) fn restore_recorded_candidate(owned:&Project,records:Vec<RecordedRegistrationBlob>)->Result<RegistrationDraft,String> {
    let mut draft=restore_recorded_candidate_parts(owned.project(),owned.sources(),records)?;
    draft.restore_resolved(&owned.precanon_view())?;Ok(draft)
}
/// The recorded consumer never reaches an issuer or the CSPRNG. It checks
/// source correspondence before the final Start/canon stage is allowed.
pub(crate) fn restore_precanon_recorded_candidate(
    owned:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1,
    records:Vec<RecordedRegistrationBlob>,
)->Result<RegistrationDraft,String> {
    let mut draft=restore_recorded_candidate_parts(owned.project(),owned.sources(),records)?;
    draft.restore_resolved(owned)?;Ok(draft)
}
fn restore_recorded_candidate_parts(
    project:&crate::authored_project::ProjectArtifact,
    sources:&crate::project_revision_envelope_supply::VerifiedSavedV25Sources,
    records:Vec<RecordedRegistrationBlob>,
)->Result<RegistrationDraft,String> {
    if records.is_empty()||records.len()>257{return Err(fail("등록 자료 개수 오류"));}
    let mut artifacts=Vec::new();let mut total=0usize;let mut descriptor_count=0;
    for row in records {
        if row.byte_length==0||row.byte_length>1_000_000||row.raw_hex.len()!=row.byte_length as usize*2
            ||row.sha256.len()!=64||!row.raw_hex.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))
            ||!row.sha256.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {
            return Err(fail("등록 자료 길이·정본 표기 오류"));
        }
        total=total.checked_add(row.byte_length as usize).ok_or_else(||fail("등록 자료 한도"))?;
        if total>2_000_000||artifacts.last().is_some_and(|old:&RegistrationBlob|old.path>=row.logical_path){return Err(fail("등록 자료 정렬·크기 오류"));}
        let role=match row.role.as_deref(){
            Some("vc04_atom_source_binding")=>{descriptor_count+=1;Some("vc04_atom_source_binding")},
            Some(resolved_supply::RESOLVED_ROLE)=>Some(resolved_supply::RESOLVED_ROLE),
            None=>None, // Contract blobs are declared resources, not another input role.
            _=>return Err(fail("등록 자료 role 오류")),
        };
        let bytes=hex::decode(row.raw_hex).map_err(|_|fail("등록 자료 원문 오류"))?;
        let sha:[u8;32]=Sha256::digest(&bytes).into();if hex::encode(sha)!=row.sha256{return Err(fail("등록 자료 원문 hash 오류"));}
        artifacts.push(RegistrationBlob{path:row.logical_path,role,bytes,sha});
    }
    if descriptor_count!=1{return Err(fail("source binding role이 하나가 아닙니다"));}
    let descriptor_blob=artifacts.iter().find(|blob|blob.role==Some("vc04_atom_source_binding")).unwrap();
    let parsed=wire::Candidate::parse(&descriptor_blob.bytes).map_err(contract_error)?;
    let expected_path=format!(".ddn/registration/atom-source-binding.v{}.bin",parsed.version);
    if descriptor_blob.path!=expected_path{return Err(fail("source binding 경로·버전 불일치"));}
    // No external registry authority is synthesized by this local producer.
    if parsed.registries.len()>1||parsed.registries.iter().any(|row|row.origin!=1){return Err(fail("외부 등록 공급은 별도 검증이 필요합니다"));}
    let mut contracts=Vec::new();let mut registry=None;let mut atoms=Vec::new();
    for row in &parsed.registries {
        let id:[u8;32]=row.identity.try_into().map_err(|_|fail("local registry 길이 오류"))?;registry=Some(id);
        let path=format!(".ddn/registration/contracts/{}.v1.json",hex::encode(id));
        let blob=artifacts.iter().find(|blob|blob.role.is_none()&&blob.path==path).ok_or_else(||fail("원자 계약 원문이 없습니다"))?;
        let custody=crate::registered_atom_contract::ContractCustody{byte_length:row.contract_length,
            sha256:row.contract_sha.try_into().unwrap(),snapshot_identity:row.snapshot.try_into().unwrap()};
        contracts.push(RecordedAtomContract::restore(blob.bytes(),row.identity,custody).map_err(contract_error)?);
        atoms=row.entries.iter().map(|entry|Ok(Atom{id:entry.identity.try_into().map_err(|_|fail("local atom 길이 오류"))?,label:entry.label.into()})).collect::<Result<_,String>>()?;
    }
    let resolved_count=artifacts.iter().filter(|blob|blob.role==Some(resolved_supply::RESOLVED_ROLE)).count();
    if resolved_count>1||artifacts.len()!=1+contracts.len()+resolved_count{return Err(fail("여분의 등록 자료입니다"));}
    let schemas=parsed.nominal.as_ref().map(|tail|tail.schemas.iter().map(|row|Schema{module:row.module.try_into().unwrap(),definition:row.definition.try_into().unwrap(),
        name_type:row.name_type.try_into().unwrap(),category:row.category,label:row.label.into(),slots:row.slots.iter().map(|slot|Slot{id:slot.identity.try_into().unwrap(),label:slot.label.into(),kind:slot.kind,ty:slot.resolved_type.into()}).collect()}).collect()).unwrap_or_default();
    let draft=RegistrationDraft{project:project.authored_identity.logical_dependency_id.as_str().into(),schemas,registry,atoms,descriptor:descriptor_blob.bytes.clone(),contracts,artifacts,resolved:None};
    draft.verify_saved_source_parts(project,sources)?;verify_nominal_source_binding_parts(project,sources,&draft)?;
    Ok(draft)
}

fn verify_nominal_source_binding_parts(project:&crate::authored_project::ProjectArtifact,
    sources:&crate::project_revision_envelope_supply::VerifiedSavedV25Sources,draft:&RegistrationDraft)->Result<(),String>{
    use ddonirang_lang::{SeedKind,SchemaCategory,TopLevelItem};
    let parsed=wire::Candidate::parse(&draft.descriptor).map_err(contract_error)?;
    if draft.schemas.iter().any(|schema|!project.authored_identity.modules.iter().any(|module|hex::encode(schema.module)==module.id.as_str())){return Err(fail("다른 모듈의 명목 catalog입니다"));}
    let mut expected=Vec::new();
    for (source,(path,capture)) in sources.captures().iter().enumerate(){
        let modules:Vec<_>=project.authored_identity.modules.iter().filter(|module|module.sources.iter().any(|file|file==path)).collect();
        if modules.is_empty(){continue;}if modules.len()!=1{return Err(fail("모듈 owner 중복"));}
        let module:[u8;32]=hex::decode(modules[0].id.as_str()).unwrap().try_into().unwrap();
        let program=ddonirang_lang::parse_v25_with_mode(capture.text(),path,ddonirang_lang::ParseMode::Strict).map_err(|e|fail(&e.to_string()))?;
        for TopLevelItem::SeedDef(seed) in program.items {if let SeedKind::Schema(category)=seed.seed_kind {
            expected.push((source,seed.span,module,seed.canonical_name,match category{SchemaCategory::Subject=>1,SchemaCategory::NominalValue=>2},seed.params));
        }}
    }
    let Some(tail)=&parsed.nominal else {return if expected.is_empty(){Ok(())}else{Err(fail("명목 catalog가 없습니다"))}};
    if tail.declarations.len()!=expected.len(){return Err(fail("명목 선언 집합이 다릅니다"));}
    let active:BTreeSet<_>=expected.iter().map(|(_,_,module,label,_,_)|(*module,label.as_str())).collect();
    for (binding,(source,span,module,label,category,pins)) in tail.declarations.iter().zip(expected.iter()) {
        let row=&draft.schemas[binding.schema];
        if binding.source!=*source||binding.start!=span.start as u64||binding.end!=span.end as u64||row.module!=*module||row.label!=*label||row.category!=*category||row.slots.len()!=pins.len(){return Err(fail("명목 선언과 catalog 결속이 다릅니다"));}
        for pin in pins {
            let slot=row.slots.iter().find(|slot|slot.label==pin.pin_name).ok_or_else(||fail("붙이 누락"))?;
            let (kind,ty)=resolve_type(&pin.type_ref,*module,&draft.schemas,&active)?;
            if slot.kind!=kind||slot.ty!=ty{return Err(fail("붙이 씨 결속이 다릅니다"));}
        }
    }
    Ok(())
}

/// The mutable borrow serializes this candidate within an adapter; the
/// adapter's existing exclusive OS root lease serializes other adapters.
/// Retrying persistence uses this object's frozen draft, not this constructor.
#[cfg(not(target_arch="wasm32"))]
pub(crate) struct NativeRegistrationPreparation<'a> {
    adapter:&'a mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    draft:RegistrationDraft,
    expected_head:Option<crate::snapshot_v2_durable_transaction::native::StoredHeadV1>,
}
#[cfg(not(target_arch="wasm32"))]
impl NativeRegistrationPreparation<'_> {
    pub(crate) fn draft(&self)->&RegistrationDraft {&self.draft}
    pub(crate) fn finish(mut self)->Result<RegistrationDraft,String> {
        self.check_current_head()?;Ok(self.draft)
    }
    pub(crate) fn check_current_head(&mut self)->Result<(),String> {
        let project=hex::decode(&self.draft.project).map_err(|_|fail("작품 정체 표기 오류"))?;
        self.adapter.verify_registration_expected_head(&project,self.expected_head.as_ref()).map_err(|e|fail(&format!("저장 머리가 달라졌습니다: {e:?}")))
    }
}
#[cfg(not(target_arch="wasm32"))]
pub(crate) fn prepare_native_authoring<'a>(
    adapter:&'a mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    owned:&Project,previous:Option<&RegistrationDraft>,
    expected:Option<&crate::snapshot_v2_durable_transaction::native::StoredHeadV1>,
)->Result<NativeRegistrationPreparation<'a>,String> {
    prepare_native_from_view(adapter,&SavedRegistrationSource::finalized(owned),previous,expected)
}
#[cfg(not(target_arch="wasm32"))]
pub(crate) fn prepare_native_precanon_authoring<'a>(
    adapter:&'a mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    owned:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1,
    previous:Option<&RegistrationDraft>,
    expected:Option<&crate::snapshot_v2_durable_transaction::native::StoredHeadV1>,
)->Result<NativeRegistrationPreparation<'a>,String> {
    let mut prepared=prepare_native_from_view(adapter,&SavedRegistrationSource::precanon(owned),previous,expected)?;
    prepared.draft.attach_resolved(owned,previous,&mut |id|getrandom::fill(id).map_err(|_|fail("등록 정체 생성 실패")))?;
    Ok(prepared)
}
#[cfg(not(target_arch="wasm32"))]
fn prepare_native_from_view<'a>(
    adapter:&'a mut crate::snapshot_v2_durable_transaction::native::SqliteAdapterV1,
    source:&impl RegistrationSourceView,previous:Option<&RegistrationDraft>,
    expected:Option<&crate::snapshot_v2_durable_transaction::native::StoredHeadV1>,
)->Result<NativeRegistrationPreparation<'a>,String> {
    let project=hex::decode(source.scope()).map_err(|_|fail("작품 정체 표기 오류"))?;
    adapter.verify_registration_parent_resources(&project,expected,previous).map_err(|e|fail(&format!("실제 저장 잠금/머리·등록 이력 확인 실패: {e:?}")))?;
    let mut entropy=|id:&mut [u8;32]|getrandom::fill(id).map_err(|_|fail("정체 생성에 실패했습니다"));
    let draft=prepare_from_source_view(source,previous,&mut entropy)?;
    Ok(NativeRegistrationPreparation{adapter,draft,expected_head:expected.cloned()})
}
fn fail(message:&str)->String {format!("E_VC04_REGISTRATION: {message}")}
fn contract_error(error:crate::registered_atom_contract::ContractFailure)->String {format!("E_VC04_REGISTRATION: {}",error.code())}
fn fresh(used:&mut BTreeSet<[u8;32]>,entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<[u8;32],String> {
    let mut id=[0;32];entropy(&mut id)?;
    if !used.insert(id){return Err(fail("정체가 충돌했습니다. 재추첨하지 않습니다"));}
    Ok(id)
}

/// Called only by the project-save candidate owner under its ExpectedHead
/// lease, never by expression evaluation or recorded restore. Keep this
/// inside the authoring module; no expression/runtime caller can issue IDs.
pub(super) fn prepare_under_candidate_owner(owned:&Project,previous:Option<&RegistrationDraft>,
    entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>,
)->Result<RegistrationDraft,String> {
    prepare_from_source_view(&SavedRegistrationSource::finalized(owned),previous,entropy)
}
/// Same nominal/atom issuer under the original authoring transaction, but
/// before final Start/canon. This does not issue the R3 resolved-role catalog.
pub(super) fn prepare_precanon_under_candidate_owner(
    owned:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1,
    previous:Option<&RegistrationDraft>,entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>,
)->Result<RegistrationDraft,String> {
    let mut draft=prepare_from_source_view(&SavedRegistrationSource::precanon(owned),previous,entropy)?;
    draft.attach_resolved(owned,previous,entropy)?;Ok(draft)
}
pub(super) fn requires_resolved_program(program:&ddonirang_lang::CanonProgram)->bool {
    resolved_supply::requires_resolved_program(program)
}
fn prepare_from_source_view(view:&impl RegistrationSourceView,previous:Option<&RegistrationDraft>,
    entropy:&mut impl FnMut(&mut [u8;32])->Result<(),String>)->Result<RegistrationDraft,String>{
    use ddonirang_lang::{SeedKind,SchemaCategory,TopLevelItem};
    let project=view.scope();
    if previous.is_some_and(|old|old.project!=project){return Err(fail("다른 작품의 이전 등록 목록입니다"));}
    let mut schemas=previous.map(|old|old.schemas.clone()).unwrap_or_default();
    let mut used=BTreeSet::new();
    for schema in &schemas {for id in [schema.definition,schema.name_type].into_iter().chain(schema.slots.iter().map(|slot|slot.id)) {
        if !used.insert(id){return Err(fail("보유 정체가 중복됩니다"));}
    }}
    let captures=view.captures();
    let mut declarations=Vec::new();
    let mut total_slots=schemas.iter().try_fold(0usize,|n,row|n.checked_add(row.slots.len())).ok_or_else(||fail("붙이 개수 한도입니다"))?;
    for (source,(path,capture)) in captures.iter().enumerate() {
        let Some(module)=view.module(path)? else{continue};
        for seed in view.declarations(path,capture)? {
            let SeedKind::Schema(category)=seed.seed_kind else {continue};
            let category=match category {SchemaCategory::Subject=>1,SchemaCategory::NominalValue=>2};
            if declarations.iter().any(|(_,_,_,old_module,old_name,_)|old_module==&module&&old_name==&seed.canonical_name){return Err(fail("명목 선언이 중복됩니다"));}
            if !schemas.iter().any(|old|old.module==module&&old.label==seed.canonical_name){
                if schemas.len()>=65536 {return Err(fail("명목 선언 개수 한도입니다"));}
                total_slots=total_slots.checked_add(seed.params.len()).ok_or_else(||fail("붙이 개수 한도입니다"))?;
                if total_slots>65536{return Err(fail("붙이 개수 한도입니다"));}
                let definition=fresh(&mut used,entropy)?;let name_type=fresh(&mut used,entropy)?;
                let slots=seed.params.iter().map(|pin|Ok(Slot{id:fresh(&mut used,entropy)?,label:pin.pin_name.clone(),kind:0,ty:Vec::new()})).collect::<Result<Vec<_>,String>>()?;
                schemas.push(Schema{module,definition,name_type,category,label:seed.canonical_name.clone(),slots});
            }
            declarations.push((source,seed.span.start,seed.span.end,module,seed.canonical_name,seed.params.into_iter().map(|pin|(pin.pin_name,pin.type_ref)).collect::<Vec<_>>()));
            let index=schemas.iter().position(|old|old.module==module&&old.label==declarations.last().unwrap().4).unwrap();
            if schemas[index].category!=category{return Err(fail("기존 명목 종류를 바꿀 수 없습니다"));}
        }
    }
    let active:BTreeSet<_>=declarations.iter().map(|(_,_,_,module,label,_)|(*module,label.as_str())).collect();
    let mut resolved=BTreeMap::new();
    for (_,_,_,module,label,pins) in &declarations {
        let schema=schemas.iter().find(|schema|schema.module==*module&&schema.label==*label).unwrap();
        if pins.len()!=schema.slots.len(){return Err(fail("기존 붙이 집합과 다릅니다"));}
        let mut slots=Vec::new();
        for (label,ty) in pins {
            let old=schema.slots.iter().find(|slot|slot.label==*label).ok_or_else(||fail("기존 붙이 이름과 다릅니다"))?;
            let (kind,ty)=resolve_type(ty,*module,&schemas,&active)?;
            if !old.ty.is_empty()&&(old.kind!=kind||old.ty!=ty){return Err(fail("기존 붙이 씨와 다릅니다"));}
            slots.push(Slot{id:old.id,label:label.clone(),kind,ty});
        }
        slots.sort_by_key(|slot|slot.id);resolved.insert((*module,label.clone()),slots);
    }
    for schema in &mut schemas {if let Some(slots)=resolved.remove(&(schema.module,schema.label.clone())){schema.slots=slots;}}
    schemas.sort_by_key(|schema|(schema.module,schema.definition));
    let mut registry=previous.and_then(|old|old.registry);
    let mut atoms=previous.map(|old|old.atoms.clone()).unwrap_or_default();
    let mut atom_used:BTreeSet<_>=atoms.iter().map(|atom|atom.id).collect();
    if let Some(id)=registry{atom_used.insert(id);}
    let occurrences=view.occurrences();
    for (_,occurrence) in &occurrences{
        if atoms.iter().any(|atom|atom.label==occurrence.label){continue;}
        if atoms.len()>=65536{return Err(fail("원자 개수 한도입니다"));}
        if registry.is_none(){registry=Some(fresh(&mut atom_used,entropy)?);}
        let id=fresh(&mut atom_used,entropy)?;atoms.push(Atom{id,label:occurrence.label.clone()});
    }
    atoms.sort_by_key(|atom|atom.id);
    let contracts=if let Some(registry)=registry {
        vec![RecordedAtomContract::record(&registry,&atoms.iter().map(|atom|atom.id.to_vec()).collect::<Vec<_>>()).map_err(contract_error)?.1]
    } else {vec![]};
    let sources=captures.iter().map(|(path,capture)|wire::Source{path,snapshot:capture.identity(),length:capture.bytes().len() as u64,sha:capture.content_digest()}).collect();
    let registries=contracts.iter().map(|contract|wire::Registry{identity:registry.as_ref().unwrap(),origin:1,
        contract_length:contract.custody().byte_length,contract_sha:&contract.custody().sha256,snapshot:&contract.custody().snapshot_identity,
        entries:atoms.iter().map(|atom|wire::Entry{identity:&atom.id,label:&atom.label}).collect()}).collect();
    let bindings=occurrences.iter().map(|(path,occurrence)|wire::Binding{
        source:captures.iter().position(|(p,_)|p==path).unwrap(),
        start:occurrence.span.start as u64,end:occurrence.span.end as u64,
        registry:0,entry:atoms.iter().position(|atom|atom.label==occurrence.label).unwrap()}).collect();
    let nominal=wire::nominal::Tail{schemas:schemas.iter().map(|schema|wire::nominal::Schema{
        module:&schema.module,definition:&schema.definition,name_type:&schema.name_type,category:schema.category,label:&schema.label,
        slots:schema.slots.iter().map(|slot|wire::nominal::Slot{identity:&slot.id,label:&slot.label,kind:slot.kind,resolved_type:&slot.ty}).collect()}).collect(),
        declarations:declarations.iter().map(|(source,start,end,module,label,_)|wire::nominal::Declaration{source:*source,start:*start as u64,end:*end as u64,
            schema:schemas.iter().position(|schema|schema.module==*module&&schema.label==*label).unwrap()}).collect()};
    let descriptor=wire::Candidate{version:3,sources,registries,bindings,nominal:Some(nominal)}.encode().map_err(contract_error)?;
    let blob=|path:String,role,bytes:Vec<u8>|{let sha=Sha256::digest(&bytes).into();RegistrationBlob{path,role,bytes,sha}};
    let mut artifacts=vec![blob(".ddn/registration/atom-source-binding.v3.bin".into(),Some("vc04_atom_source_binding"),descriptor.clone())];
    if let Some(registry)=registry{artifacts.push(blob(format!(".ddn/registration/contracts/{}.v1.json",hex::encode(registry)),None,contracts[0].recorded_bytes().to_vec()));}
    artifacts.sort_by(|a,b|a.path.cmp(&b.path));
    let total=artifacts.iter().try_fold(0usize,|n,blob|n.checked_add(blob.bytes.len())).ok_or_else(||fail("등록 자료 크기 한도입니다"))?;
    if captures.iter().try_fold(total,|n,(_,capture)|n.checked_add(capture.bytes().len())).is_none_or(|n|n>2_000_000){return Err(fail("전체 자료 크기 한도입니다"));}
    if artifacts.iter().any(|blob|view.conflicts(&blob.path)){return Err(fail("등록 자료 경로가 기존 원문과 충돌합니다"));}
    let candidate=RegistrationDraft{project:project.into(),schemas,registry,atoms,descriptor,contracts,artifacts,resolved:None};
    view.verify(&candidate)?;Ok(candidate)
}
fn resolve_type(ty:&ddonirang_lang::TypeRef,module:[u8;32],schemas:&[Schema],active:&BTreeSet<([u8;32],&str)>)->Result<(u8,Vec<u8>),String> {
    use ddonirang_lang::TypeRef;
    use ddonirang_core::platform::RecordFieldType as T;
    if let TypeRef::Named(name)=ty {
        if let Some(schema)=schemas.iter().find(|schema|schema.module==module&&schema.label==*name){
            if !active.contains(&(module,name.as_str())){return Err(fail("현재 선언되지 않은 명목 씨입니다"));}
            return Ok((1,schema.name_type.to_vec()));
        }
    }
    enum Job<'a>{Type(&'a TypeRef),Applied(&'a str,usize),Tuple(usize)}
    let mut jobs=vec![Job::Type(ty)];let mut built=Vec::new();let mut nodes=0usize;let mut has_nominal=false;
    while let Some(job)=jobs.pop(){match job {
        Job::Type(ty)=>{
            nodes=nodes.checked_add(1).ok_or_else(||fail("붙이 씨 노드 한도입니다"))?;
            if nodes>4096{return Err(fail("붙이 씨 노드 한도입니다"));}
            match ty {
                TypeRef::Tuple(items)=>{
                    jobs.push(Job::Tuple(items.len()));jobs.extend(items.iter().rev().map(Job::Type));
                },
                TypeRef::Named(name)=>{
                    if let Some(schema)=schemas.iter().find(|schema|schema.module==module&&schema.label==*name){
                        if !active.contains(&(module,name.as_str())){return Err(fail("현재 선언되지 않은 명목 씨입니다"));}
                        has_nominal=true;built.push(T::ResolvedNominalV1{category:schema.category,name_type_identity:schema.name_type});
                    }else if matches!(name.as_str(),"바른수"|"셈수"|"나눔수"|"참거짓"|"글"){
                        built.push(T::Named{name:name.clone()});
                    }else{return Err(fail("검증된 붙이 씨 공급이 없습니다"));}
                },
                TypeRef::Applied{name,args} if matches!((name.as_str(),args.len()),("차림"|"모음",1)|("짝맞춤",2))=>{
                    jobs.push(Job::Applied(name,args.len()));jobs.extend(args.iter().rev().map(Job::Type));
                },
                _=>return Err(fail("검증된 붙이 씨 공급이 없습니다")),
            }
        },
        Job::Applied(name,n)=>{let args=built.split_off(built.len().checked_sub(n).ok_or_else(||fail("붙이 씨 구성 오류"))?);built.push(T::Applied{name:name.into(),args});},
        Job::Tuple(n)=>{let items=built.split_off(built.len().checked_sub(n).ok_or_else(||fail("붙이 씨 구성 오류"))?);built.push(T::Tuple{items});},
    }}
    let value=built.pop().ok_or_else(||fail("붙이 씨가 없습니다"))?;
    let mut raw=Vec::new();value.encode_canon(&mut raw);value.drop_stack_safe();
    if raw.len()>4096{return Err(fail("붙이 씨 바이트 한도입니다"));}Ok((if has_nominal{2}else{0},raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r3_precanon_nominal_registration_matches_existing_owner_without_final_start() {
        let finalized=create(SOURCE);
        let source=crate::project_revision_envelope_supply::admit_precanon_authored_source(
            finalized.sources().clone(),finalized.project().save().unwrap().as_bytes(),
        ).unwrap();
        let mut old_count=0;let mut new_count=0;
        let legacy=prepare_under_candidate_owner(&finalized,None,&mut |id|next(&mut old_count,id)).unwrap();
        let draft=prepare_precanon_under_candidate_owner(&source,None,&mut |id|next(&mut new_count,id)).unwrap();
        assert_eq!(old_count,new_count);
        assert_eq!(legacy.descriptor,draft.descriptor);
        assert_eq!(legacy.schemas,draft.schemas);
        assert_eq!(serde_json::to_vec(&legacy.recorded_blobs()).unwrap(),serde_json::to_vec(&draft.recorded_blobs()).unwrap());
        let admitted=crate::project_revision_envelope_supply::finalize_registered_authored_source(source,draft).unwrap();
        assert_eq!(finalized.definition_identity(),admitted.definition_identity());
        let restored=restore_recorded_candidate(&admitted,admitted.registration().unwrap().recorded_blobs()).unwrap();
        assert_eq!(restored.descriptor,legacy.descriptor);
    }
    #[test]
    fn r3_precanon_registration_refuses_foreign_source_before_final_start() {
        let first=create(SOURCE);let foreign=create(SOURCE);
        let mut count=0;
        let draft=prepare_under_candidate_owner(&first,None,&mut |id|next(&mut count,id)).unwrap();
        let source=crate::project_revision_envelope_supply::admit_precanon_authored_source(
            foreign.sources().clone(),foreign.project().save().unwrap().as_bytes(),
        ).unwrap();
        assert!(draft.verify_precanon_binding(&source).unwrap_err().contains("다른 작품"));
        assert!(crate::project_revision_envelope_supply::finalize_registered_authored_source(source,draft).unwrap_err().contains("다른 작품"));
        // Original source remains admissible with the same recorded IDs.
        assert!(restore_recorded_candidate(&first,prepare_under_candidate_owner(&first,None,&mut |id|next(&mut count,id)).unwrap().recorded_blobs()).is_ok());
    }
    #[test]
    fn r3_precanon_recorded_restore_preserves_bytes_and_rejects_resealed_foreign_capture() {
        let first=create(SOURCE);
        let source=crate::project_revision_envelope_supply::admit_precanon_authored_source(
            first.sources().clone(),first.project().save().unwrap().as_bytes(),
        ).unwrap();
        let mut count=0;
        let original=prepare_precanon_under_candidate_owner(&source,None,&mut |id|next(&mut count,id)).unwrap();
        let raw=serde_json::to_vec(&original.recorded_blobs()).unwrap();
        let restored=restore_precanon_recorded_candidate(&source,original.recorded_blobs()).unwrap();
        assert_eq!(serde_json::to_vec(&restored.recorded_blobs()).unwrap(),raw);
        // Recompute the transport checksum after changing an actual capture.
        // A checksum is not source-correspondence evidence and cannot repair it.
        let mut records=original.recorded_blobs();
        let row=records.iter_mut().find(|row|row.role.as_deref()==Some("vc04_atom_source_binding")).unwrap();
        let mut bytes=hex::decode(&row.raw_hex).unwrap();
        let at=bytes.windows(32).position(|window|window==source.sources().captures()[0].1.identity()).unwrap();
        bytes[at]^=1;
        row.raw_hex=hex::encode(&bytes);row.sha256=hex::encode(Sha256::digest(&bytes));
        assert!(restore_precanon_recorded_candidate(&source,records).is_err());
        let checked=restore_precanon_recorded_candidate(&source,original.recorded_blobs()).unwrap();
        let finalized=crate::project_revision_envelope_supply::finalize_registered_authored_source(source,checked).unwrap();
        assert_eq!(finalized.definition_identity(),first.definition_identity());
        assert_eq!(serde_json::to_vec(&finalized.registration().unwrap().recorded_blobs()).unwrap(),raw);
    }
    #[test]
    fn vc04_tuple_nominal_field_keeps_type03_positions_and_type04_identity(){
        use ddonirang_core::platform::RecordFieldType as T;
        let project=create("명찰:이름씨 := { 이름:글. }. 쌍명찰:이름씨 := { 짝:(나눔수,명찰). }. 실행:움직씨 := { }.");
        let mut count=0;
        let draft=prepare_under_candidate_owner(&project,None,&mut |id|next(&mut count,id)).unwrap();
        let nominal=draft.schemas.iter().find(|s|s.label=="명찰").unwrap();
        let pair=draft.schemas.iter().find(|s|s.label=="쌍명찰").unwrap();
        let slot=&pair.slots[0];assert_eq!(slot.kind,2);
        let kind=crate::unit_wire::decode_recorded_field_type(&slot.ty).unwrap();
        assert_eq!(kind,T::Tuple{items:vec![T::Named{name:"나눔수".into()},
            T::ResolvedNominalV1{category:2,name_type_identity:nominal.name_type}]});
        let restored=restore_recorded_candidate(&project,draft.recorded_blobs()).unwrap();
        assert_eq!(restored.schemas,draft.schemas);
        assert_eq!(restored.descriptor,draft.descriptor);
    }
    const SOURCE:&str="갑:이름씨 := { 이름:글. }. 을값:이름씨 := { 이름:글. }. 병사:임자 := { 왼쪽:갑. 오른쪽:갑. }. 실행:움직씨 := { 색 <- #빨강. }.";
    #[test]
    fn vc04_r5_nested_nominal_type_uses_v3_kind2_and_recorded_ids(){
        let project=create("명찰:이름씨 := { 이름:글. }. 보관함:이름씨 := { 값:(명찰)차림. 비율:나눔수. }. 실행:움직씨 := { }.");
        let mut count=0;let draft=prepare_under_candidate_owner(&project,None,&mut |id|next(&mut count,id)).unwrap();
        let nominal=draft.schemas.iter().find(|s|s.label=="명찰").unwrap();
        let container=draft.schemas.iter().find(|s|s.label=="보관함").unwrap();
        let slot=container.slots.iter().find(|s|s.label=="값").unwrap();assert_eq!(slot.kind,2);
        let kind=crate::unit_wire::decode_recorded_field_type(&slot.ty).unwrap();
        assert_eq!(kind,ddonirang_core::platform::RecordFieldType::Applied{name:"차림".into(),args:vec![
            ddonirang_core::platform::RecordFieldType::ResolvedNominalV1{category:2,name_type_identity:nominal.name_type} ]});
        let restored=restore_recorded_candidate(&project,draft.recorded_blobs()).unwrap();
        assert_eq!(restored.schemas,draft.schemas);assert_eq!(restored.descriptor,draft.descriptor);
        let mut old=wire::Candidate::parse(draft.descriptor()).unwrap();old.version=2;assert!(old.encode().is_err());
        let mut wrong=wire::Candidate::parse(draft.descriptor()).unwrap();
        let raw=kind;let mut mismatch=raw.clone();
        let ddonirang_core::platform::RecordFieldType::Applied{args,..}=&mut mismatch else{panic!()};
        let ddonirang_core::platform::RecordFieldType::ResolvedNominalV1{category,..}=&mut args[0] else{panic!()};*category=1;
        let mut bytes=Vec::new();mismatch.encode_canon(&mut bytes);
        let tail=wrong.nominal.as_mut().unwrap();let row=tail.schemas.iter_mut().find(|s|s.label=="보관함").unwrap();
        row.slots.iter_mut().find(|s|s.label=="값").unwrap().resolved_type=&bytes;
        assert!(wrong.encode().is_err());
        #[cfg(windows)]
        {
            let mut nonce=[0;32];getrandom::fill(&mut nonce).unwrap();
            let root=std::env::temp_dir().join(format!("ddn-r5-type04-{}",hex::encode(nonce)));std::fs::create_dir(&root).unwrap();
            let saved=crate::runtime_surface::save_initial_authored_source_native(&project.project().save().unwrap(),&[1],&root).unwrap();
            assert!(saved.project_payload_utf8.contains("atom-source-binding.v3.bin"),"actual Native source save must retain the v3 producer, not an old or test-only descriptor");
            let cold=crate::runtime_surface::read_authored_source_native(&root,&saved.project_identity,saved.grant_locator.as_deref().unwrap()).unwrap();
            assert_eq!(saved.project_payload_utf8,cold.project_payload_utf8);assert_eq!(saved.pesi,cold.pesi);
            assert_eq!(saved.revision_integrity,cold.revision_integrity);
            println!("VC04_TYPE04_NATIVE_SOURCE_PASS root={}",root.display());
        }
    }
    fn create(source:&str)->Project {
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.";
        let project=super::super::ProjectArtifact::create("v1-core-v25".into(),super::super::SourceSnapshot{
            root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),files:vec![
                super::super::SourceFile{path:"root.ddn".into(),text:root.into()},
                super::super::SourceFile{path:"main.ddn".into(),text:source.into()},
            ]},String::new()).unwrap();
        crate::project_revision_envelope_supply::admit_fresh_authored_project(project.save().unwrap().as_bytes()).unwrap()
    }
    fn edited(old:&Project,source:&str)->Project {
        let mut project=old.project().clone();
        project.source_snapshot.files.iter_mut().find(|file|file.path=="main.ddn").unwrap().text=source.into();
        crate::project_revision_envelope_supply::admit_fresh_authored_project(project.save().unwrap().as_bytes()).unwrap()
    }
    fn next(counter:&mut u8,id:&mut [u8;32])->Result<(),String> {*counter+=1;*id=[*counter;32];Ok(())}
    #[test]
    fn vc04_candidate_issuer_binds_real_modules_captures_resources_and_pesi() {
        let project=create(SOURCE);let mut count=0;
        let draft=prepare_under_candidate_owner(&project,None,&mut |id|next(&mut count,id)).unwrap();
        assert_eq!(count,12); // 3*(definition+type),4 slots,registry+atom.
        assert_eq!(draft.schemas.len(),3);assert_eq!(draft.atoms.len(),1);assert_eq!(draft.artifacts.len(),2);
        let soldier=draft.schemas.iter().find(|schema|schema.label=="병사").unwrap();
        assert_ne!(soldier.slots[0].id,soldier.slots[1].id);
        assert_eq!(soldier.slots[0].ty,soldier.slots[1].ty);
        let plain=crate::project_snapshot_identity::current::issue_saved_local_project(&project).unwrap();
        let registered=crate::project_snapshot_identity::current::issue_registration_candidate(&project,&draft).unwrap();
        assert_eq!(plain.constituents()[0],registered.constituents()[0]);
        assert_eq!(plain.constituents()[2],registered.constituents()[2]);
        assert_ne!(plain.constituents()[1],registered.constituents()[1]);
        assert_ne!(plain.constituents()[3],registered.constituents()[3]);assert_ne!(plain.pesi(),registered.pesi());
        assert!(draft.verify_source_binding(&create(SOURCE)).is_err(),"same source is not the same authored project");
        for blob in draft.artifacts(){assert_eq!(blob.sha(),<[u8;32]>::from(Sha256::digest(blob.bytes())));}
    }
    #[test]
    fn vc04_candidate_reauthor_keeps_ids_and_inactive_dependencies_without_entropy() {
        let project=create(SOURCE);let mut count=0;
        let old=prepare_under_candidate_owner(&project,None,&mut |id|next(&mut count,id)).unwrap();
        let no_entropy=&mut |_:&mut [u8;32]|->Result<(),String>{panic!("retained identity must not be reissued")};
        let reversed=edited(&project,"병사:임자 := { 오른쪽:갑. 왼쪽:갑. }. 을값:이름씨 := { 이름:글. }. 갑:이름씨 := { 이름:글. }. 실행:움직씨 := { 색 <- #빨강. }.");
        let reordered=prepare_under_candidate_owner(&reversed,Some(&old),no_entropy).unwrap();
        assert_eq!(old.schemas,reordered.schemas);assert_ne!(old.descriptor,reordered.descriptor);
        let removed=edited(&project,"을값:이름씨 := { 이름:글. }. 실행:움직씨 := { }.");
        let inactive=prepare_under_candidate_owner(&removed,Some(&old),no_entropy).unwrap();
        assert_eq!(old.schemas,inactive.schemas);assert_eq!(old.registry,inactive.registry);
        assert_eq!(old.contracts[0].recorded_bytes(),inactive.contracts[0].recorded_bytes());
        let readded=prepare_under_candidate_owner(&project,Some(&inactive),no_entropy).unwrap();assert_eq!(old.descriptor,readded.descriptor);
        let bad=edited(&project,"을값:이름씨 := { 이름:글. }. 병사:임자 := { 왼쪽:갑. 오른쪽:갑. }. 실행:움직씨 := { }.");
        assert!(prepare_under_candidate_owner(&bad,Some(&old),no_entropy).is_err());
        let changed=edited(&project,&SOURCE.replace("이름:글","이름:바른수"));
        assert!(prepare_under_candidate_owner(&changed,Some(&old),no_entropy).is_err());
        old.verify_source_binding(&project).unwrap();
    }
    #[test]
    fn vc04_candidate_entropy_failure_or_collision_never_retries_or_mutates_old() {
        let project=create(SOURCE);let mut calls=0;
        assert!(prepare_under_candidate_owner(&project,None,&mut |id|{calls+=1;*id=[9;32];Ok(())}).is_err());
        assert_eq!(calls,2);
        let mut calls=0;
        assert!(prepare_under_candidate_owner(&project,None,&mut |_|{calls+=1;Err("entropy unavailable".into())}).is_err());assert_eq!(calls,1);
    }
    #[test]
    fn vc04_registration_blob_payload_roundtrip_keeps_pesi_and_rejects_modified_artifacts() {
        use crate::unit_wire::project_runtime_attachment as attachment;
        let project=create(SOURCE);let mut count=0;
        let draft=prepare_under_candidate_owner(&project,None,&mut |id|next(&mut count,id)).unwrap();
        let mut trailer=draft.recorded_blobs();
        let row=trailer.iter_mut().find(|row|row.role.as_deref()==Some("vc04_atom_source_binding")).unwrap();
        row.raw_hex.push_str("00");row.byte_length+=1;row.sha256=hex::encode(Sha256::digest(hex::decode(&row.raw_hex).unwrap()));
        assert!(restore_recorded_candidate(&project,trailer).is_err());
        let mut contract=draft.recorded_blobs();
        let row=contract.iter_mut().find(|row|row.role.is_none()).unwrap();
        let mut bytes=hex::decode(&row.raw_hex).unwrap();bytes[0]^=1;row.raw_hex=hex::encode(&bytes);row.sha256=hex::encode(Sha256::digest(bytes));
        assert!(restore_recorded_candidate(&project,contract).is_err());
        let records=draft.recorded_blobs();let restored=restore_recorded_candidate(&project,records).unwrap();
        assert_eq!(restored.descriptor(),draft.descriptor());assert_eq!(restored.schemas,draft.schemas);
        let bound=project.bind_registration_candidate(restored).unwrap();
        // Transport fixture only: no VM, Subject or durable world is fabricated.
        let resources=r#"{"schema":"ddn.resource_snapshot.v2","unit_wire_schema":"ddn.unit_value_wire.v2","components":[],"next_entity":0,"json":{},"handle":{},"value_det":{},"fixed64":{}}"#;
        let runtime=serde_json::json!({"state":{"snapshot_v2":{"schema":"ddn.wasm_state_snapshot.v2","resource_snapshot_schema":"ddn.resource_snapshot.v2","resources":serde_json::from_str::<serde_json::Value>(resources).unwrap()}}}).to_string();
        let artifact=attachment::encode_saved_runtime_artifact(&bound,&runtime,resources,&[],&[2]).unwrap();
        let checked=attachment::check_bytes(artifact.as_bytes()).unwrap();
        let reopened=attachment::restore_saved_runtime_payload(&checked).unwrap();
        let before=crate::project_snapshot_identity::current::issue_saved_local_project(&bound).unwrap();
        let after=attachment::check_saved_revision_context(&checked,&reopened,&[],&[2]).unwrap();
        assert_eq!(before.pesi(),after.pesi());
        assert_eq!(reopened.registration().unwrap().descriptor(),draft.descriptor());
        assert!(attachment::check_saved_revision_context(&checked,&create(SOURCE),&[],&[2]).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn vc04_native_registration_prepares_under_actual_exclusive_root_and_head() {
        use crate::snapshot_v2_durable_transaction::native::{SqliteAdapterV1,StoredHeadV1};
        let project=create(SOURCE);
        let dir=std::env::temp_dir().join(format!("ddn-vc04-registration-{}",project.project().authored_identity.logical_dependency_id.as_str()));
        std::fs::create_dir(&dir).unwrap();
        let mut adapter=SqliteAdapterV1::provision(&dir).unwrap();
        let mut prepared=prepare_native_authoring(&mut adapter,&project,None,None).unwrap();
        prepared.check_current_head().unwrap();
        assert!(SqliteAdapterV1::open_admitted(&dir).is_err(),"other adapter cannot mint a concurrent first candidate");
        let raw=prepared.draft().descriptor().to_vec();prepared.check_current_head().unwrap();assert_eq!(raw,prepared.draft().descriptor());
        let stale=StoredHeadV1{project:hex::decode(project.project().authored_identity.logical_dependency_id.as_str()).unwrap(),revision:vec![1],integrity:[2;32],digest:[3;32]};
        prepared.draft().verify_source_binding(&project).unwrap();
        drop(prepared);
        assert!(prepare_native_authoring(&mut adapter,&project,None,Some(&stale)).is_err());
        // No durable commit or VM publication has occurred in this test.
    }
}
