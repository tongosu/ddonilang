//! RD-679: one shared saved-project supply owner, before adapter discovery/I/O.
//! Projection preserves already supplied identity bytes; it never issues identity.

use crate::local_project::{
    self, NormalizedProjectManifest, VirtualProjectFileSource, VirtualProjectSource,
    LOCAL_PROJECT_TREE_KIND, MAX_FILE_BYTES, MAX_PROJECT_FILES, MAX_PROJECT_TOTAL_UTF8_BYTES,
    VIRTUAL_PROJECT_SOURCE_SCHEMA,
};
use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;

pub(crate) const MAX_IDENTITY_BYTES: usize = 4096;
const MAX_REVISION_ENVELOPE_BYTES: usize = 1_000_000;

/// Borrowed complete saved bytes, never a path to reread or a digest substitute.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SavedSourceBlobV1<'a> {
    pub(crate) path: &'a str,
    pub(crate) role: &'a str,
    pub(crate) bytes: &'a [u8],
}

/// Only the source/manifest part of supply admission. This is NOT a PESI,
/// dependency, semantic DefinitionIdentity, or durable-commit admission proof.
#[derive(Debug)]
pub(crate) struct VerifiedSavedSourceV1 {
    source: VirtualProjectSource,
    manifest: NormalizedProjectManifest,
    official_entry_definition: Option<CanonicalIdentityBytesV1>,
}

/// Forward v25 source supply. Each logical DDN source is bound to exactly
/// the saved raw bytes, not a reread, selected tab or adapter-created digest.
/// Still NOT DefinitionIdentity/dependency/PESI/durable commit admission.
#[derive(Debug)]
pub(crate) struct VerifiedSavedV25Sources {
    saved: VerifiedSavedSourceV1,
    captures: Vec<(String, crate::source_snapshot_admission::SourceCapture)>,
    preparation: local_project::AuthoredPreparation,
    entry_declaration: SavedEntryDeclaration,
    atom_value_occurrences: Vec<ResolvedSavedAtomOccurrence>,
}
#[derive(Debug)]
pub(crate) struct ResolvedSavedAtomOccurrence {
    path: String,
    occurrence: ddonirang_lang::AtomValueOccurrence,
}
impl ResolvedSavedAtomOccurrence {
    pub(crate) fn path(&self) -> &str { &self.path }
    pub(crate) fn occurrence(&self) -> &ddonirang_lang::AtomValueOccurrence { &self.occurrence }
}
#[derive(Debug)]
pub(crate) struct SavedEntryDeclaration { name: String, path: String, span: ddonirang_lang::Span }
impl SavedEntryDeclaration {
    pub(crate) fn name(&self) -> &str { &self.name }
    pub(crate) fn path(&self) -> &str { &self.path }
    pub(crate) fn span(&self) -> ddonirang_lang::Span { self.span }
}
impl VerifiedSavedV25Sources {
    pub(crate) fn atom_value_occurrences(&self) -> &[ResolvedSavedAtomOccurrence] { &self.atom_value_occurrences }
    pub(crate) fn recorded_capture_catalog_json(&self) -> Result<String,String> {
        let records=self.captures.iter().map(|(path,capture)|Ok(SavedCaptureRecordV1 {
            path:path.clone(),record_json_utf8:capture.recorded_json()?,
            snapshot_identity_hex:hex::encode(capture.identity()),
        })).collect::<Result<Vec<_>,String>>()?;
        serde_json::to_string(&SavedCaptureCatalogV1 {
            schema:"ddn.saved_source_capture_catalog.v1".into(),records,
        }).map_err(|_|"E_PROJECT_SOURCE_CAPTURE_CATALOG_ENCODING: 저장 봉인 목록을 만들 수 없습니다".into())
    }
    pub(crate) fn saved(&self) -> &VerifiedSavedSourceV1 { &self.saved }
    pub(crate) fn captures(&self) -> &[(String, crate::source_snapshot_admission::SourceCapture)] { &self.captures }
    pub(crate) fn preparation(&self) -> &local_project::AuthoredPreparation { &self.preparation }
    pub(crate) fn entry_declaration(&self) -> &SavedEntryDeclaration { &self.entry_declaration }
    pub(crate) fn definition_capture(&self) -> &crate::source_snapshot_admission::SourceCapture {
        &self.captures.iter().find(|(path,_)|path == &self.entry_declaration.path)
            .expect("start declaration belongs to the admitted source closure").1
    }
    pub(crate) fn entry_capture(&self) -> &crate::source_snapshot_admission::SourceCapture {
        &self.captures.iter().find(|(path,_)|path == &self.saved.manifest.entry_path)
            .expect("saved entry was admitted before source sealing").1
    }
}

/// Explicit project/module ownership admitted by the existing authoring owner.
/// This does not turn caller-supplied DefinitionIdentity bytes into evidence.
/// Source ownership before canon. There is deliberately no final Start,
/// definition context, PESI, registration admission, or world capability here.
/// Only the shared source/module/capture verifier constructs this stage.
#[derive(Debug,Clone)]
pub(crate) struct PreCanonAuthoredSourceV1 {
    sources: std::sync::Arc<VerifiedSavedV25Sources>,
    project: crate::authored_project::ProjectArtifact,
    entry_module_index: usize,
}
impl PreCanonAuthoredSourceV1 {
    pub(crate) fn sources(&self)->&std::sync::Arc<VerifiedSavedV25Sources> {&self.sources}
    pub(crate) fn project(&self)->&crate::authored_project::ProjectArtifact {&self.project}
    pub(crate) fn entry_module(&self)->&crate::authored_project::AuthoredModule {
        &self.project.authored_identity.modules[self.entry_module_index]
    }
    /// Paths select an existing owned row, never issue a module identity.
    /// A metadata root outside every module remains None, not a fake module.
    pub(crate) fn module_for_source(&self,path:&str)->Result<Option<&crate::authored_project::AuthoredModule>,String> {
        let mut owners=self.project.authored_identity.modules.iter().filter(|module|module.sources.iter().any(|source|source==path));
        let owner=owners.next();
        if owners.next().is_some(){return Err("E_PROJECT_SAVED_AUTHORED_OWNER: 원문의 소유 모듈이 둘 이상입니다".into());}
        Ok(owner)
    }
}

#[derive(Debug,Clone)]
pub(crate) struct VerifiedSavedAuthoredProjectV1 {
    sources: std::sync::Arc<VerifiedSavedV25Sources>,
    project: crate::authored_project::ProjectArtifact,
    entry_module_index: usize,
    definition_identity: crate::saved_definition_identity::SavedDefinitionIdentity,
    registration:Option<std::sync::Arc<crate::authored_project::registration::RegistrationDraft>>,
}
impl VerifiedSavedAuthoredProjectV1 {
    pub(crate) fn precanon_view(&self)->PreCanonAuthoredSourceV1 {
        PreCanonAuthoredSourceV1{sources:self.sources.clone(),project:self.project.clone(),entry_module_index:self.entry_module_index}
    }
    pub(crate) fn registration(&self)->Option<&crate::authored_project::registration::RegistrationDraft> {self.registration.as_deref()}
    /// Candidate binding only. Durable lineage/readback still precedes Run.
    pub(crate) fn bind_registration_candidate(mut self,draft:crate::authored_project::registration::RegistrationDraft)->Result<Self,String> {
        draft.verify_source_binding(&self)?;
        self.registration=Some(std::sync::Arc::new(draft));Ok(self)
    }
    pub(crate) fn resolved_revision_supply(&self) -> VerifiedSavedSourceV1 {
        let saved=self.sources.saved();
        VerifiedSavedSourceV1 {
            source:saved.source.clone(), manifest:saved.manifest.clone(),
            official_entry_definition:Some(CanonicalIdentityBytesV1(self.definition_identity.as_bytes().to_vec())),
        }
    }
    pub(crate) fn definition_identity(&self) -> &crate::saved_definition_identity::SavedDefinitionIdentity {
        &self.definition_identity
    }
    pub(crate) fn sources(&self) -> &std::sync::Arc<VerifiedSavedV25Sources> { &self.sources }
    pub(crate) fn project(&self) -> &crate::authored_project::ProjectArtifact { &self.project }
    pub(crate) fn entry_module(&self) -> &crate::authored_project::AuthoredModule {
        &self.project.authored_identity.modules[self.entry_module_index]
    }
}

/// File names are checked as locators only. Permanent identity is supplied by
/// an explicitly authored project, never minted while reopening saved bytes.
pub(crate) fn admit_saved_authored_project(
    sources: std::sync::Arc<VerifiedSavedV25Sources>,
    artifact_bytes: &[u8],
) -> Result<VerifiedSavedAuthoredProjectV1,String> {
    finalize_legacy_authored_source(admit_precanon_authored_source(sources,artifact_bytes)?)
}

/// Bootstrap input for the R3 registration owner. Reuses every original
/// source ownership check, but never calls the final canon/Start producer.
pub(crate) fn admit_precanon_authored_source(
    sources:std::sync::Arc<VerifiedSavedV25Sources>,artifact_bytes:&[u8],
)->Result<PreCanonAuthoredSourceV1,String> {
    let project=crate::authored_project::open(artifact_bytes)?;
    let snapshot=&project.source_snapshot;
    let manifest=sources.saved().manifest();
    if snapshot.root_path.as_deref()!=Some(manifest.root_path.as_str())
        || snapshot.entry_path!=manifest.entry_path
        || snapshot.files.len()!=sources.captures().len() {
        return Err("E_PROJECT_SAVED_AUTHORED_SOURCE: 작품 정체의 원문 목록과 저장 공급이 다릅니다".into());
    }
    for (path,capture) in sources.captures() {
        let file=snapshot.files.iter().find(|file|file.path==*path)
            .ok_or("E_PROJECT_SAVED_AUTHORED_SOURCE: 작품 정체의 원문이 공급 목록에 없습니다")?;
        if file.text.as_bytes()!=capture.bytes() {
            return Err("E_PROJECT_SAVED_AUTHORED_SOURCE: 작품 정체의 원문 바이트와 봉인이 다릅니다".into());
        }
    }
    // Authored open already validates module source ownership against 얽음.
    // Resolve the actual Start declaration's owner, not the selected tab/head.
    let declaration_path=sources.entry_declaration().path();
    let mut owners=project.authored_identity.modules.iter().enumerate()
        .filter(|(_,module)|module.sources.iter().any(|path|path==declaration_path));
    let (entry_module_index,_)=owners.next()
        .ok_or("E_PROJECT_SAVED_AUTHORED_OWNER: 시작 선언의 명시 모듈 정체가 없습니다")?;
    if owners.next().is_some() {
        return Err("E_PROJECT_SAVED_AUTHORED_OWNER: 시작 선언의 소유 모듈이 둘 이상입니다".into());
    }
    Ok(PreCanonAuthoredSourceV1 {sources,project,entry_module_index})
}

// Existing compiler path, with all prior admission checks preserved. The R3
// catalog/raw-canon successor must be inserted before its final Start stage;
// this function does not claim to implement that successor or invent a Start.
fn finalize_legacy_authored_source(source:PreCanonAuthoredSourceV1)->Result<VerifiedSavedAuthoredProjectV1,String> {
    let definition_identity=crate::saved_definition_identity::issue_saved_start(
        &source.sources,&source.project,source.entry_module_index,
    )?;
    Ok(VerifiedSavedAuthoredProjectV1 {sources:source.sources,project:source.project,
        entry_module_index:source.entry_module_index,definition_identity,registration:None})
}

/// Source-save bootstrap: verify the actual candidate before final Start,
/// then recheck the same candidate against the finalized source. Neither
/// stage grants world activation or durable transaction success.
pub(crate) fn finalize_registered_authored_source(
    source:PreCanonAuthoredSourceV1,
    draft:crate::authored_project::registration::RegistrationDraft,
)->Result<VerifiedSavedAuthoredProjectV1,String> {
    draft.verify_precanon_binding(&source)?;
    let definition_identity=crate::saved_definition_identity::issue_registered_saved_start(&source,&draft,source.entry_module_index)?;
    VerifiedSavedAuthoredProjectV1{sources:source.sources,project:source.project,entry_module_index:source.entry_module_index,
        definition_identity,registration:None}.bind_registration_candidate(draft)
}

pub(crate) fn seal_saved_v25_sources(saved: VerifiedSavedSourceV1) -> Result<VerifiedSavedV25Sources,String> {
    prepare_saved_v25_captures(saved,|_,bytes|crate::source_snapshot_admission::capture_declared_v25(bytes))
}

#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedCaptureCatalogV1 { schema:String,records:Vec<SavedCaptureRecordV1> }
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedCaptureRecordV1 { path:String,record_json_utf8:String,snapshot_identity_hex:String }

/// Restore the existing capture catalog. The resulting typed closure still
/// requires the complete revision/PESI comparisons before VM publication.
pub(crate) fn restore_saved_v25_sources(
    saved:VerifiedSavedSourceV1,catalog_json:&str,
) -> Result<VerifiedSavedV25Sources,String> {
    if catalog_json.len()>1_000_000 {
        return Err("E_PROJECT_SOURCE_CAPTURE_CATALOG_LIMIT: 저장 봉인 목록 한도를 넘었습니다".into());
    }
    let strict=crate::unit_wire::parse_snapshot_json(catalog_json)?;
    let count=strict.get("records").and_then(serde_json::Value::as_array).map(Vec::len)
        .ok_or("E_PROJECT_SOURCE_CAPTURE_CATALOG_FIELDS: 저장 봉인 목록이 없습니다")?;
    if count>MAX_PROJECT_FILES || count!=saved.source.files.len() {
        return Err("E_PROJECT_SOURCE_CAPTURE_CATALOG_COUNT: 원문과 저장 봉인 목록의 개수가 다릅니다".into());
    }
    let catalog=<SavedCaptureCatalogV1 as serde::Deserialize>::deserialize(&*strict)
        .map_err(|_|"E_PROJECT_SOURCE_CAPTURE_CATALOG_FIELDS: 저장 봉인 목록 필드가 다릅니다")?;
    if catalog.schema!="ddn.saved_source_capture_catalog.v1"
        || serde_json::to_string(&catalog).map_err(|_|"E_PROJECT_SOURCE_CAPTURE_CATALOG_ENCODING")?!=catalog_json {
        return Err("E_PROJECT_SOURCE_CAPTURE_CATALOG_CANONICAL: 저장 봉인 목록이 정본이 아닙니다".into());
    }
    // Record order equals the already normalized, complete raw-source closure.
    // Duplicated/reordered/missing/foreign entries are not silently repaired.
    if catalog.records.iter().zip(&saved.source.files).any(|(record,file)|record.path!=file.path
        || !matches!(file.role.as_str(),"root"|"ddn_source")) {
        return Err("E_PROJECT_SOURCE_CAPTURE_CATALOG_PATH: 저장 봉인 목록의 원문 경로가 다릅니다".into());
    }
    prepare_saved_v25_captures(saved,|path,bytes| {
        let record=catalog.records.iter().find(|record|record.path==path)
            .ok_or("E_PROJECT_SOURCE_CAPTURE_CATALOG_MISSING: 저장 봉인이 없습니다")?;
        if record.snapshot_identity_hex.len()!=64
            || !record.snapshot_identity_hex.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)) {
            return Err("E_PROJECT_SOURCE_CAPTURE_CATALOG_IDENTITY: 저장 원문 정체 표기가 다릅니다".into());
        }
        let expected:[u8;32]=hex::decode(&record.snapshot_identity_hex)
            .map_err(|_|"E_PROJECT_SOURCE_CAPTURE_CATALOG_IDENTITY")?.try_into()
            .map_err(|_|"E_PROJECT_SOURCE_CAPTURE_CATALOG_IDENTITY")?;
        crate::source_snapshot_admission::restore_recorded_v25_capture(&record.record_json_utf8,bytes,&expected)
    })
}

fn prepare_saved_v25_captures(
    saved: VerifiedSavedSourceV1,
    mut capture:impl FnMut(&str,&[u8])->Result<crate::source_snapshot_admission::SourceCapture,String>,
) -> Result<VerifiedSavedV25Sources,String> {
    // Read the explicitly bound 글판 from the admitted root. An old root or
    // omitted profile must never silently become a v25 source epoch.
    local_project::project_source_profile(saved.source(),None)
        .map_err(|error|format!("{}: {}",error.code,error.message))?;
    // Reuse the product linker: exact-one local, no-argument 움직씨 and
    // declared module ownership, not a text search or the selected tab.
    let preparation=local_project::prepare_authored_execution(saved.source.clone())
        .map_err(|error|format!("{}: {}",error.code,error.message))?;
    let name=preparation.start_definition.as_ref()
        .ok_or("E_PROJECT_START_REQUIRED: 시작 없는 라이브러리는 실행 상태를 복원할 수 없습니다")?;
    let linked=ddonirang_lang::parse_v25_with_mode(&preparation.source,&preparation.target_path,ddonirang_lang::ParseMode::Strict)
        .map_err(|error|format!("E_PROJECT_START_PARSE: {error}"))?;
    let definition=linked.items.iter().find_map(|item| {
        let ddonirang_lang::TopLevelItem::SeedDef(seed)=item;
        (seed.canonical_name==*name).then_some(seed)
    }).ok_or("E_PROJECT_START_DEFINITION: 저장된 시작 정의가 없습니다")?;
    let (path,span)=preparation.original_span(definition.span)?;
    let entry_declaration=SavedEntryDeclaration {name:name.clone(),path,span};
    if !saved.source.files.iter().any(|file|file.path==entry_declaration.path && file.role=="ddn_source") {
        return Err("E_PROJECT_START_SOURCE: 시작 정의의 소유 원문이 공급 목록에 없습니다".into());
    }
    let mut captures=Vec::new();
    captures.try_reserve_exact(saved.source.files.len())
        .map_err(|_|"E_PROJECT_SOURCE_ADMISSION_LIMIT: 원문 봉인 공간이 부족합니다")?;
    for file in &saved.source.files {
        if matches!(file.role.as_str(),"root"|"ddn_source") {
            let sealed=capture(&file.path,file.utf8_text.as_bytes())?;
            captures.push((file.path.clone(),sealed));
        }
    }
    if !captures.iter().any(|(path,_)|path == &saved.manifest.entry_path) {
        return Err("E_PROJECT_SOURCE_ADMISSION_ENTRY: 봉인된 시작 원문이 없습니다".into());
    }
    // Consume actual parser value nodes and the linker's checked source map.
    // No textual `#` scan and no path/selected-editor identity substitution.
    let mut atom_value_occurrences = Vec::new();
    for occurrence in linked.origin.atom_value_occurrences.values() {
        if !matches!(occurrence.role,ddonirang_lang::ast::AtomValueRole::LocalValue){continue;}
        let (path,span) = preparation.original_span(occurrence.span)?;
        let source = captures.iter().find(|(candidate,_)|candidate == &path)
            .ok_or("E_PROJECT_ATOM_SOURCE: 원자 값의 봉인 원문이 없습니다")?;
        let raw = source.1.text().get(span.start..span.end)
            .ok_or("E_PROJECT_ATOM_SOURCE: 원자 값의 원문 범위가 다릅니다")?;
        if raw.strip_prefix('#') != Some(occurrence.label.as_str()) {
            return Err("E_PROJECT_ATOM_SOURCE: 원자 값과 봉인 원문이 다릅니다".into());
        }
        atom_value_occurrences.push(ResolvedSavedAtomOccurrence {path,
            occurrence:ddonirang_lang::AtomValueOccurrence {span,label:occurrence.label.clone(),role:occurrence.role.clone()}});
    }
    atom_value_occurrences.sort_by(|a,b|(&a.path,a.occurrence.span.start,a.occurrence.span.end)
        .cmp(&(&b.path,b.occurrence.span.start,b.occurrence.span.end)));
    if atom_value_occurrences.windows(2).any(|pair|pair[0].path == pair[1].path
        && pair[0].occurrence.span.end > pair[1].occurrence.span.start) {
        return Err("E_PROJECT_ATOM_SOURCE: 원자 값의 원문 범위가 중복됩니다".into());
    }
    Ok(VerifiedSavedV25Sources {saved,captures,preparation,entry_declaration,atom_value_occurrences})
}

impl VerifiedSavedSourceV1 {
    pub(crate) fn source(&self) -> &VirtualProjectSource {
        &self.source
    }
    pub(crate) fn manifest(&self) -> &NormalizedProjectManifest {
        &self.manifest
    }
    pub(crate) fn official_entry_definition(&self) -> Result<&CanonicalIdentityBytesV1,Failure> {
        self.official_entry_definition.as_ref().ok_or(Failure::InvalidInput)
    }

    /// Compare the typed normalizer output, not its legacy display JSON. The
    /// RD-631 projection and its self-empty digest are a separate next gate.
    pub(crate) fn verify_manifest(
        &self,
        candidate: &NormalizedProjectManifest,
    ) -> Result<(), Failure> {
        let expected = &self.manifest;
        let header_matches = candidate.schema == expected.schema
            && candidate.kind == expected.kind
            && candidate.root_path == expected.root_path
            && candidate.entry_path == expected.entry_path
            && candidate.file_count == expected.file_count
            && candidate.total_utf8_bytes == expected.total_utf8_bytes
            && candidate.project_graph_sha256 == expected.project_graph_sha256
            && candidate.lock_sha256 == expected.lock_sha256
            && candidate.canonical_manifest_sha256 == expected.canonical_manifest_sha256
            && candidate.limits.max_files == expected.limits.max_files
            && candidate.limits.max_file_bytes == expected.limits.max_file_bytes
            && candidate.limits.max_serialized_artifact_bytes
                == expected.limits.max_serialized_artifact_bytes;
        let files_match = candidate.files.len() == expected.files.len()
            && candidate.files.iter().zip(&expected.files).all(|(a, b)| {
                a.path == b.path
                    && a.role == b.role
                    && a.byte_length == b.byte_length
                    && a.sha256 == b.sha256
            });
        if header_matches && files_match {
            Ok(())
        } else {
            Err(Failure::CustodyConflict)
        }
    }
}

pub(crate) fn collect_saved_source_supply(
    root_path: &str,
    official_entry_path: &str,
    official_entry_definition: &CanonicalIdentityBytesV1,
    blobs: &[SavedSourceBlobV1<'_>],
) -> Result<VerifiedSavedSourceV1, Failure> {
    collect_source_supply(root_path,official_entry_path,Some(official_entry_definition),blobs)
}

fn collect_source_supply(
    root_path:&str,official_entry_path:&str,
    official_entry_definition:Option<&CanonicalIdentityBytesV1>,blobs:&[SavedSourceBlobV1<'_>],
) -> Result<VerifiedSavedSourceV1,Failure> {
    // Preflight all cardinalities and raw lengths before path normalization,
    // UTF-8 decoding, any owned copy, and calls into the existing normalizer.
    if blobs.len() > MAX_PROJECT_FILES {
        return Err(Failure::LimitExceeded);
    }
    let mut total = 0usize;
    let mut metadata_minimum = root_path
        .len()
        .checked_add(official_entry_path.len())
        .ok_or(Failure::LimitExceeded)?;
    for blob in blobs {
        if blob.bytes.len() > MAX_FILE_BYTES {
            return Err(Failure::LimitExceeded);
        }
        total = total
            .checked_add(blob.bytes.len())
            .ok_or(Failure::LimitExceeded)?;
        metadata_minimum = metadata_minimum
            .checked_add(blob.path.len())
            .and_then(|n| n.checked_add(blob.role.len()))
            .ok_or(Failure::LimitExceeded)?;
    }
    // These strings must occur in the existing bounded envelope. This is only
    // its proven lower bound, not an invented per-path limit or wire format.
    if total > MAX_PROJECT_TOTAL_UTF8_BYTES || metadata_minimum > MAX_REVISION_ENVELOPE_BYTES {
        return Err(Failure::LimitExceeded);
    }
    if let Some(identity)=official_entry_definition { validate_identity_length(identity.as_bytes().len())?; }
    verify_exact_project_path(root_path)?;
    verify_exact_project_path(official_entry_path)?;
    for (index, blob) in blobs.iter().enumerate() {
        verify_exact_project_path(blob.path)?;
        if blobs[..index]
            .iter()
            .any(|prior| prior.path.eq_ignore_ascii_case(blob.path))
        {
            return Err(Failure::CustodyConflict);
        }
    }
    for blob in blobs {
        if blob.bytes.starts_with(&[0xef, 0xbb, 0xbf]) || std::str::from_utf8(blob.bytes).is_err() {
            return Err(Failure::InvalidInput);
        }
    }
    if blobs
        .iter()
        .filter(|b| b.path == official_entry_path && b.role == "ddn_source")
        .count()
        != 1
        || blobs
            .iter()
            .filter(|b| b.path == root_path && b.role == "root")
            .count()
            != 1
    {
        return Err(Failure::CustodyConflict);
    }
    let mut files = Vec::new();
    files
        .try_reserve_exact(blobs.len())
        .map_err(|_| Failure::LimitExceeded)?;
    for blob in blobs {
        files.push(VirtualProjectFileSource {
            path: blob.path.to_owned(),
            role: blob.role.to_owned(),
            // Already checked above. No BOM stripping, EOL or Unicode repair.
            utf8_text: std::str::from_utf8(blob.bytes)
                .map_err(|_| Failure::InvalidInput)?
                .to_owned(),
        });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let source = VirtualProjectSource {
        schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_owned(),
        kind: Some(LOCAL_PROJECT_TREE_KIND.to_owned()),
        package_hash: None,
        root_path: root_path.to_owned(),
        entry_path: official_entry_path.to_owned(),
        files,
    };
    let manifest = local_project::normalize_virtual_project_source(source.clone()).map_err(
        |error| match error.code {
            "E_PROJECT_FILE_COUNT_LIMIT"
            | "E_PROJECT_FILE_SIZE_LIMIT"
            | "E_PROJECT_TOTAL_SIZE_LIMIT"
            | "E_PROJECT_SERIALIZED_SIZE_LIMIT" => Failure::LimitExceeded,
            _ => Failure::InvalidInput,
        },
    )?;
    // Saved material already has canonical roles. Do not silently take the
    // local-project display-role aliases as saved-envelope authority.
    if source
        .files
        .iter()
        .zip(&manifest.files)
        .any(|(raw, normalized)| raw.path != normalized.path || raw.role != normalized.role)
    {
        return Err(Failure::CustodyConflict);
    }
    Ok(VerifiedSavedSourceV1 {
        source,
        manifest,
        official_entry_definition: official_entry_definition.cloned(),
    })
}

/// Fresh explicit project execution admits exact authored source bytes. Start
/// identity remains absent during structural collection, then is independently
/// issued by the resolved declaration owner. No invented sentinel/hash is used.
pub(crate) fn admit_fresh_authored_project(
    artifact_bytes:&[u8],
) -> Result<VerifiedSavedAuthoredProjectV1,String> {
    finalize_legacy_authored_source(admit_fresh_precanon_authored_source(artifact_bytes)?)
}

pub(crate) fn admit_fresh_precanon_authored_source(
    artifact_bytes:&[u8],
)->Result<PreCanonAuthoredSourceV1,String> {
    let project=crate::authored_project::open(artifact_bytes)?;
    let snapshot=&project.source_snapshot;
    let root=snapshot.root_path.as_deref()
        .ok_or("E_PROJECT_RUNTIME_ROOT_REQUIRED: 실행 저장에는 명시 얽음 뿌리가 필요합니다")?;
    let blobs:Vec<_>=snapshot.files.iter().map(|file|SavedSourceBlobV1 {
        path:&file.path,role:if file.path==root {"root"}else{"ddn_source"},bytes:file.text.as_bytes(),
    }).collect();
    let saved=collect_source_supply(root,&snapshot.entry_path,None,&blobs)
        .map_err(|_|"E_PROJECT_RUNTIME_SOURCE_SUPPLY: 명시 작품의 원문 공급이 다릅니다")?;
    let sources=seal_saved_v25_sources(saved)?;
    admit_precanon_authored_source(std::sync::Arc::new(sources),artifact_bytes)
}

fn verify_exact_project_path(path: &str) -> Result<(), Failure> {
    if local_project::normalize_project_path(path).map_err(|_| Failure::InvalidInput)? != path {
        return Err(Failure::InvalidInput);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IdentityFamily {
    Project,
    HistoryRevision,
    SourceEpoch,
    ProjectSemanticConfiguration,
    Definition,
    DependencyResolution,
}

impl IdentityFamily {
    fn prefix(self) -> &'static str {
        match self {
            Self::Project => "ProjectIdentity:",
            Self::HistoryRevision => "HistoryRevisionIdentity:",
            Self::SourceEpoch => "SourceEpochIdentity:",
            Self::ProjectSemanticConfiguration => "ProjectSemanticConfigurationIdentity:",
            Self::Definition => "DefinitionIdentity:",
            Self::DependencyResolution => "DependencyResolutionIdentity:",
        }
    }
}

/// Validated physical carrier, not evidence that an upstream semantic owner exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalIdentityBytesV1(Vec<u8>);

impl CanonicalIdentityBytesV1 {
    pub(crate) fn from_owner_bytes(bytes: &[u8]) -> Result<Self, Failure> {
        validate_identity_length(bytes.len())?;
        let mut copied = Vec::new();
        copied
            .try_reserve_exact(bytes.len())
            .map_err(|_| Failure::LimitExceeded)?;
        copied.extend_from_slice(bytes);
        Ok(Self(copied))
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

fn validate_identity_length(length: usize) -> Result<(), Failure> {
    if length == 0 {
        Err(Failure::InvalidInput)
    } else if length > MAX_IDENTITY_BYTES {
        Err(Failure::LimitExceeded)
    } else {
        Ok(())
    }
}

pub(crate) fn issue_rd631_identity_projection(
    family: IdentityFamily,
    identity: &CanonicalIdentityBytesV1,
) -> Result<String, Failure> {
    validate_identity_length(identity.0.len())?;
    let length = identity
        .0
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(family.prefix().len()))
        .ok_or(Failure::LimitExceeded)?;
    let mut text = String::new();
    text.try_reserve_exact(length)
        .map_err(|_| Failure::LimitExceeded)?;
    text.push_str(family.prefix());
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in identity.as_bytes() {
        text.push(HEX[(byte >> 4) as usize] as char);
        text.push(HEX[(byte & 15) as usize] as char);
    }
    Ok(text)
}

pub(crate) fn verify_rd631_identity_projection(
    family: IdentityFamily,
    text: &str,
) -> Result<CanonicalIdentityBytesV1, Failure> {
    let maximum = family.prefix().len() + MAX_IDENTITY_BYTES * 2;
    if text.len() > maximum {
        return Err(Failure::LimitExceeded);
    }
    let hex = text
        .strip_prefix(family.prefix())
        .ok_or(Failure::InvalidInput)?;
    if hex.len() % 2 != 0 {
        return Err(Failure::InvalidInput);
    }
    validate_identity_length(hex.len() / 2)?;
    // Scan all digits before materializing, without normalization or repair.
    if !hex
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Failure::InvalidInput);
    }
    fn digit(byte: u8) -> u8 {
        if byte <= b'9' {
            byte - b'0'
        } else {
            byte - b'a' + 10
        }
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(hex.len() / 2)
        .map_err(|_| Failure::LimitExceeded)?;
    for pair in hex.as_bytes().chunks_exact(2) {
        bytes.push(digit(pair[0]) * 16 + digit(pair[1]));
    }
    Ok(CanonicalIdentityBytesV1(bytes))
}

/// Checks the projections that have matching request fields. This does not
/// replace semantic issuer admission or the source/configuration/PESI gates.
pub(crate) fn verify_revision_identity_crosschecks(
    envelope: &serde_json::Value,
    project: &[u8],
    expected_parent: &[u8],
    new_revision: &[u8],
) -> Result<(), Failure> {
    let read = |key: &str, family| {
        let text = envelope[key].as_str().ok_or(Failure::InvalidInput)?;
        verify_rd631_identity_projection(family, text)
    };
    let projected_project = read("project_identity", IdentityFamily::Project)?;
    let projected_revision = read("history_revision_identity", IdentityFamily::HistoryRevision)?;
    if projected_project.as_bytes() != project {
        return Err(Failure::CustodyConflict);
    }
    if projected_revision.as_bytes() != new_revision {
        return Err(Failure::StaleRevision);
    }
    match envelope.get("expected_parent_revision_identity") {
        Some(serde_json::Value::Null) if expected_parent.is_empty() => {}
        Some(serde_json::Value::Null) => return Err(Failure::StaleRevision),
        Some(serde_json::Value::String(_)) => {
            if read(
                "expected_parent_revision_identity",
                IdentityFamily::HistoryRevision,
            )?
            .as_bytes()
                != expected_parent
            {
                return Err(Failure::StaleRevision);
            }
        }
        _ => return Err(Failure::InvalidInput),
    }
    read("source_epoch_identity", IdentityFamily::SourceEpoch)?;
    read(
        "project_semantic_configuration_identity",
        IdentityFamily::ProjectSemanticConfiguration,
    )?;
    read(
        "official_entry_definition_identity",
        IdentityFamily::Definition,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddonirang_core::Nuri;

    #[test]
    fn r3_precanon_ownership_precedes_final_start_without_a_sentinel() {
        let raw=include_bytes!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/authored-project.json");
        let source=admit_fresh_precanon_authored_source(raw).unwrap();
        assert!(source.sources().saved().official_entry_definition().is_err(),"fresh ownership must not contain an invented final Start");
        assert!(source.module_for_source("root.ddn").unwrap().is_none());
        assert_eq!(source.module_for_source("main.ddn").unwrap().unwrap().id,source.entry_module().id);
        let capture_ids=source.sources().captures().iter().map(|(path,capture)|(path.clone(),capture.identity().to_vec())).collect::<Vec<_>>();
        let project_id=source.project().authored_identity.logical_dependency_id.clone();
        let verified=finalize_legacy_authored_source(source).unwrap();
        assert_eq!(verified.project().authored_identity.logical_dependency_id,project_id);
        assert_eq!(verified.sources().captures().iter().map(|(path,capture)|(path.clone(),capture.identity().to_vec())).collect::<Vec<_>>(),capture_ids);
        assert!(verified.definition_identity().as_bytes().iter().any(|byte|*byte!=0));
    }
    #[test]
    fn r3_precanon_rejects_different_raw_capture_before_finalization() {
        let raw=include_bytes!("../../solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/authored-project.json");
        let source=admit_fresh_precanon_authored_source(raw).unwrap();
        let mut changed=source.project().clone();
        changed.source_snapshot.files.iter_mut().find(|file|file.path=="main.ddn").unwrap().text.push_str("\n// 다른 원문 판\n");
        let changed=changed.save().unwrap();
        assert!(admit_precanon_authored_source(source.sources().clone(),changed.as_bytes()).is_err());
        assert!(admit_precanon_authored_source(source.sources().clone(),raw).is_ok());
    }

    #[test]
    fn saved_v25_sealing_binds_root_and_entry_exact_bytes_without_legacy_fallback() {
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let entry="실행:움직씨 := { 1 보여주기. }.\r\n";
        let blobs=[SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:root.as_bytes()},SavedSourceBlobV1 {path:"app/main.ddn",role:"ddn_source",bytes:entry.as_bytes()}];
        let definition=CanonicalIdentityBytesV1::from_owner_bytes(&[4;32]).unwrap();
        let saved=collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&blobs).unwrap();
        let sealed=seal_saved_v25_sources(saved).unwrap();
        assert_eq!(sealed.captures().len(),2);
        assert_eq!(sealed.entry_capture().bytes(),entry.as_bytes());
        assert_eq!(sealed.saved().manifest().entry_path,"app/main.ddn");
        assert_eq!(sealed.entry_declaration().name(),"실행");
        assert_eq!(sealed.entry_declaration().path(),"app/main.ddn");
        let span=sealed.entry_declaration().span();
        assert!(std::str::from_utf8(&sealed.definition_capture().bytes()[span.start..span.end]).unwrap().contains("실행"));
        let tree=crate::project_snapshot_identity::current::issue_saved_source_tree(&sealed).unwrap();
        assert!(tree.envelope.starts_with(b"ddn.source_logical_tree_content_identity.v1\0"));
        assert_ne!(tree.identity.as_raw(),sealed.entry_capture().identity());
        // A3-P10: reopening consumes recorded captures, never fresh ordinals.
        let catalog=sealed.recorded_capture_catalog_json().unwrap();
        let saved_again=collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&blobs).unwrap();
        let reopened=restore_saved_v25_sources(saved_again,&catalog).unwrap();
        assert_eq!(reopened.recorded_capture_catalog_json().unwrap(),catalog);
        let supply = crate::unit_wire::resumable_complete_state::encode_source_capture_supply(&sealed).unwrap();
        let restored_supply = crate::unit_wire::resumable_complete_state::encode_source_capture_supply(&reopened).unwrap();
        assert_eq!(supply, restored_supply, "reopening must not mint new captures");
        let check = |bytes: &[u8]| crate::unit_wire::resumable_complete_state::check_source_capture_supply(bytes, &reopened);
        check(&supply).unwrap();
        let mut corrupt = supply.clone(); *corrupt.last_mut().unwrap() ^= 1;
        assert!(check(&corrupt).is_err());
        assert!(check(&supply[..supply.len()-1]).is_err());
        let mut suffix = supply.clone(); suffix.push(0);
        assert!(check(&suffix).is_err());
        for (old,new) in sealed.captures().iter().zip(reopened.captures()) {
            assert_eq!(old.1.identity(),new.1.identity());
            assert_eq!(old.1.ordinal(),new.1.ordinal());
            assert_eq!(old.1.bytes(),new.1.bytes());
        }
        assert_eq!(crate::project_snapshot_identity::current::issue_saved_source_tree(&reopened).unwrap().identity.as_raw(),tree.identity.as_raw());
        for bad in [format!(" {catalog}"),catalog.replace("app/main.ddn","app/other.ddn"),catalog.replacen("{","{\"unknown\":true,",1)] {
            let saved_again=collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&blobs).unwrap();
            assert!(restore_saved_v25_sources(saved_again,&bad).is_err());
        }
        let restored=crate::ddn_runtime::DdnProgram::from_sealed_v25_source_with_mode(
            sealed.entry_capture().clone(),"app/main.ddn",ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target(),
        ).unwrap();
        assert_eq!(restored.sealed_source_capture().unwrap().identity(),sealed.entry_capture().identity());
        let legacy=collect_saved_source_supply("뿌리.ddn",ENTRY_PATH,&definition,&fixture()).unwrap();
        assert!(seal_saved_v25_sources(legacy).is_err());
    }
    #[test]
    fn saved_v25_start_is_resolved_in_its_owned_source_not_assumed_to_be_the_head() {
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=차림[\"app/main.ddn\",\"app/run.ddn\"])], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="실행:움직씨 := { 1 보여주기. }.\n";
        let definition=CanonicalIdentityBytesV1::from_owner_bytes(&[5;32]).unwrap();
        let blobs=[SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:root.as_bytes()},SavedSourceBlobV1 {path:"app/main.ddn",role:"ddn_source",bytes:b""},SavedSourceBlobV1 {path:"app/run.ddn",role:"ddn_source",bytes:body.as_bytes()}];
        let saved=collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&blobs).unwrap();
        let sealed=seal_saved_v25_sources(saved).unwrap();
        assert_eq!(sealed.entry_capture().bytes(),b"");
        assert_eq!(sealed.entry_declaration().path(),"app/run.ddn");
        assert_eq!(sealed.definition_capture().bytes(),body.as_bytes());
        assert!(sealed.preparation().source.contains("실행"));
        // Product compilation/execution of the saved closure, including an
        // empty module head. Never recapture generated linked code as source.
        let sealed=std::sync::Arc::new(sealed);
        let ids:Vec<_>=sealed.captures().iter().map(|(_,capture)|*capture.identity()).collect();
        let program=crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
            sealed.clone(),ddonirang_lang::ParseMode::Strict,crate::ddn_runtime::default_age_target(),
        ).unwrap();
        assert!(program.sealed_source_capture().is_none(),"linked text is not an original source capture");
        assert!(std::ptr::eq(program.sealed_project_sources().unwrap(),sealed.as_ref()));
        assert_eq!(program.sealed_project_sources().unwrap().captures().iter()
            .map(|(_,capture)|*capture.identity()).collect::<Vec<_>>(),ids);
        let cloned=program.clone();
        assert!(std::ptr::eq(cloned.sealed_project_sources().unwrap(),sealed.as_ref()));
        let mut runner=crate::ddn_runtime::DdnRunner::new(program,"매마디");
        runner.configure_project_entry(sealed.entry_declaration().name()).unwrap();
        let input=ddonirang_core::InputSnapshot {
            tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,
            last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,
            ai_injections:Vec::new(),net_events:Vec::new(),
            frame_source:ddonirang_core::InputSource::Person,rng_seed:0,
        };
        let world=ddonirang_core::platform::DetNuri::new();
        let output=runner.run_update(world.world(),&input,&std::collections::HashMap::new()).unwrap();
        assert_eq!(output.display_outputs.iter().map(|row|row.text.as_str()).collect::<Vec<_>>(),vec!["1"]);
        let absent=body.replace("실행","다른이름");
        let broken=[blobs[0],blobs[1],SavedSourceBlobV1 {bytes:absent.as_bytes(),..blobs[2]}];
        let saved=collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&broken).unwrap();
        assert!(seal_saved_v25_sources(saved).unwrap_err().starts_with("E_PROJECT_START_DEFINITION:"));
    }

    #[test]
    fn saved_v25_explicit_project_owner_binds_exact_source_and_start_module() {
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=차림[\"app/main.ddn\",\"app/run.ddn\"])], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="실행:움직씨 := { 1 보여주기. }.\r\n";
        let definition=CanonicalIdentityBytesV1::from_owner_bytes(&[9;32]).unwrap();
        let blobs=[SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:root.as_bytes()},
            SavedSourceBlobV1 {path:"app/main.ddn",role:"ddn_source",bytes:b""},
            SavedSourceBlobV1 {path:"app/run.ddn",role:"ddn_source",bytes:body.as_bytes()}];
        let sources=std::sync::Arc::new(seal_saved_v25_sources(
            collect_saved_source_supply("root.ddn","app/main.ddn",&definition,&blobs).unwrap(),
        ).unwrap());
        // Existing explicit project creation owns UUID issuance; reopening does not.
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:blobs.iter().map(|blob|SourceFile {path:blob.path.into(),
                text:std::str::from_utf8(blob.bytes).unwrap().into()}).collect(),
        },String::new()).unwrap();
        let bytes=project.save().unwrap();
        let first=admit_saved_authored_project(sources.clone(),bytes.as_bytes()).unwrap();
        let second=admit_saved_authored_project(sources.clone(),bytes.as_bytes()).unwrap();
        assert_eq!(first.entry_module().id,project.authored_identity.modules[0].id);
        assert_eq!(first.entry_module().id,second.entry_module().id);
        assert_eq!(first.definition_identity(),second.definition_identity());
        let first_execution=crate::project_snapshot_identity::current::issue_saved_local_project(&first).unwrap();
        let second_execution=crate::project_snapshot_identity::current::issue_saved_local_project(&second).unwrap();
        let program = crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
            sources.clone(), ddonirang_lang::ParseMode::Strict, crate::ddn_runtime::default_age_target(),
        ).unwrap();
        let runner = crate::ddn_runtime::DdnRunner::new(program, "매마디");
        let contract = crate::unit_wire::resumable_complete_state::encode_execution_contract_supply(&first, &runner).unwrap();
        let check_contract = |bytes: &[u8]| crate::unit_wire::resumable_complete_state::check_execution_contract_supply(bytes, &second, &runner);
        check_contract(&contract).unwrap();
        let mut forged_capability = contract.clone(); *forged_capability.last_mut().unwrap() = 1;
        assert!(check_contract(&forged_capability).is_err());
        assert!(check_contract(&contract[..contract.len()-1]).is_err());
        let mut suffix = contract.clone(); suffix.push(0);
        assert!(check_contract(&suffix).is_err());
        let plain = crate::ddn_runtime::DdnRunner::new(
            crate::ddn_runtime::DdnProgram::from_v25_source("1 보여주기.", "plain.ddn").unwrap(), "매마디",
        );
        assert!(crate::unit_wire::resumable_complete_state::encode_execution_contract_supply(&first, &plain).is_err());
        // Field12 reconstructs the real named Start in a staging runner; it
        // does not publish or claim admission of these structural fixtures.
        let empty = [0;8];
        let initial = crate::unit_wire::CompleteStateFrameV1 {
            pesi:first_execution.pesi().as_raw(),resources:&empty,parameters:&empty,
            next_tick:0,subjects:&empty,atom_evidence:&[],subject_custody:&[],
        }.encode_canonical().unwrap();
        let descriptor = |age:u8,strict:u8,n1:&[u8],entry:&[u8],strategy:u8,instance:&[u8],phase:u8,count:u64| {
            let mut out=vec![0,2];
            let add=|out:&mut Vec<u8>,bytes:&[u8]| {out.extend((bytes.len() as u32).to_be_bytes());out.extend(bytes);};
            add(&mut out,b"ddn.resume.source-rebuild-descriptor.v1");add(&mut out,b"v1-core-v25");
            out.extend([age,strict]);add(&mut out,n1);
            out.push(1);out.extend([0;24]);add(&mut out,entry);add(&mut out,entry);out.push(strategy);
            add(&mut out,bytes.as_bytes());add(&mut out,&initial);out.extend(0u64.to_be_bytes());out.extend(7u64.to_be_bytes());
            out.push(phase);out.extend(count.to_be_bytes());add(&mut out,instance);out.extend(0u32.to_be_bytes());out
        };
        let canonical=crate::ddn_runtime::DdnProgram::from_saved_v25_project_with_mode(
            sources.clone(),ddonirang_lang::ParseMode::Strict,ddonirang_lang::AgeTarget::Age3,
        ).unwrap().canonical_ddn();
        let entry=first.definition_identity().as_bytes();
        let instance=runner.execution_instance_owner_bytes();
        let good=descriptor(3,1,canonical.as_bytes(),entry,1,instance,0,0);
        let rebuild=|raw:&[u8]| crate::unit_wire::resumable_complete_state::rebuild_source_descriptor_candidate(raw,&first).is_ok();
        assert!(rebuild(&good));
        let mut candidate=crate::unit_wire::resumable_complete_state::rebuild_source_descriptor_candidate(&good,&first).unwrap();
        assert_eq!(candidate.initial_tick,0);assert_eq!(candidate.initial_world.next_tick,0);
        assert_eq!(candidate.base_rng_seed,7);assert_eq!(candidate.phase,0);assert_eq!(candidate.successful_madi_count,0);
        assert!(candidate.prior_durable_receipts.is_empty());
        assert_eq!(candidate.runner.execution_instance_owner_bytes(),instance);
        assert_eq!(candidate.runner.work_cost_receipt().counters.committed_madi,0,
            "rebuilding must not execute or publish the named entry");
        let input=ddonirang_core::InputSnapshot {
            tick_id:0,dt:ddonirang_core::Fixed64::ONE,keys_pressed:0,last_key_name:String::new(),
            pointer_x_i32:0,pointer_y_i32:0,ai_injections:Vec::new(),net_events:Vec::new(),
            frame_source:ddonirang_core::InputSource::Person,rng_seed:7,
        };
        let world=ddonirang_core::platform::DetNuri::new();
        let output=candidate.runner.run_update(world.world(),&input,&std::collections::HashMap::new()).unwrap();
        assert_eq!(output.display_outputs.iter().map(|row|row.text.as_str()).collect::<Vec<_>>(),vec!["1"]);
        assert_eq!(sources.captures().iter().map(|(_,c)|c.identity()).collect::<Vec<_>>(),
            candidate.runner.saved_program_sources().unwrap().captures().iter().map(|(_,c)|c.identity()).collect::<Vec<_>>());
        for bad in [descriptor(8,1,canonical.as_bytes(),entry,1,instance,0,0),
            descriptor(3,0,canonical.as_bytes(),entry,1,instance,0,0),
            descriptor(3,1,b"different",entry,1,instance,0,0),
            descriptor(3,1,canonical.as_bytes(),&[1;32],1,instance,0,0),
            descriptor(3,1,canonical.as_bytes(),entry,0,instance,0,0),
            descriptor(3,1,canonical.as_bytes(),entry,1,b"foreign",0,0),
            descriptor(3,1,canonical.as_bytes(),entry,1,instance,0,1),
            descriptor(3,1,canonical.as_bytes(),entry,1,instance,1,0)] {assert!(!rebuild(&bad));}
        assert!(!rebuild(&good[..good.len()-1]));
        let mut suffixed=good.clone();suffixed.push(0);assert!(!rebuild(&suffixed));
        let mut inflated=good.clone();let n=inflated.len();inflated[n-4..].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(!rebuild(&inflated));
        assert_eq!(first_execution.pesi(),second_execution.pesi());
        assert_eq!(first_execution.constituents(),second_execution.constituents());
        let constituent_supply = crate::unit_wire::resumable_complete_state::encode_constituent_supply(&first).unwrap();
        assert_eq!(constituent_supply, crate::unit_wire::resumable_complete_state::encode_constituent_supply(&second).unwrap());
        let check = |bytes: &[u8], pesi: &[u8;32]| crate::unit_wire::resumable_complete_state::check_constituent_supply(bytes, pesi, &second);
        check(&constituent_supply, first_execution.pesi().as_raw()).unwrap();
        let mut corrupt = constituent_supply.clone(); *corrupt.last_mut().unwrap() ^= 1;
        assert!(check(&corrupt, first_execution.pesi().as_raw()).is_err());
        assert!(check(&constituent_supply, &[0;32]).is_err());
        assert!(check(&constituent_supply[..constituent_supply.len()-1], first_execution.pesi().as_raw()).is_err());
        let source_tree=crate::project_snapshot_identity::current::issue_saved_source_tree(first.sources()).unwrap();
        assert_eq!(first_execution.constituents()[0],source_tree.identity);
        assert_eq!(first.project().authored_identity.logical_dependency_id,
            second.project().authored_identity.logical_dependency_id);
        assert!(std::sync::Arc::ptr_eq(first.sources(),&sources));
        assert_eq!(first.sources().definition_capture().bytes(),body.as_bytes());
        let copied=project.duplicate().unwrap();
        let duplicate=admit_saved_authored_project(sources.clone(),copied.save().unwrap().as_bytes()).unwrap();
        assert_ne!(duplicate.project().authored_identity.logical_dependency_id,
            first.project().authored_identity.logical_dependency_id);
        assert_ne!(duplicate.entry_module().id,first.entry_module().id);
        assert_ne!(duplicate.definition_identity(),first.definition_identity());
        assert_ne!(crate::project_snapshot_identity::current::issue_saved_local_project(&duplicate).unwrap().pesi(),
            first_execution.pesi());
        assert!(crate::unit_wire::resumable_complete_state::check_constituent_supply(
            &constituent_supply, first_execution.pesi().as_raw(), &duplicate,
        ).is_err(), "explicit project duplication cannot reuse the old constituent supply");
        let mut changed=project.clone();
        changed.source_snapshot.files.iter_mut().find(|file|file.path=="app/run.ddn").unwrap()
            .text=body.replace("1 보여주기","2 보여주기");
        assert!(admit_saved_authored_project(sources.clone(),changed.save().unwrap().as_bytes())
            .unwrap_err().starts_with("E_PROJECT_SAVED_AUTHORED_SOURCE:"));
        let mut repaired=project.clone();
        repaired.source_snapshot.files.iter_mut().find(|file|file.path=="app/run.ddn").unwrap()
            .text=body.replace("\r\n","\n");
        assert!(admit_saved_authored_project(sources,repaired.save().unwrap().as_bytes())
            .unwrap_err().starts_with("E_PROJECT_SAVED_AUTHORED_SOURCE:"));
    }

    #[test]
    fn saved_definition_identity_binds_declaration_not_capture_path_or_alternate_order() {
        // A3-P8 physical projection; RD-325/326 invariant, not Owner approval.
        use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"app/main.ddn\", 원문=\"app/main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="(값:수) 돕다{도우~도와}:수:셈씨 := { 돕다 <- 값 + 1. }.\n실행:움직씨 := { 1 보여주기. }.\n";
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"app/main.ddn".into(),
            files:vec![SourceFile {path:"root.ddn".into(),text:root.into()},
                SourceFile {path:"app/main.ddn".into(),text:body.into()}],
        },String::new()).unwrap();
        let admit=|text:&str,owner:&ProjectArtifact| {
            let blobs=[SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:root.as_bytes()},
                SavedSourceBlobV1 {path:"app/main.ddn",role:"ddn_source",bytes:text.as_bytes()}];
            // Legacy raw supply bytes remain NOT the typed identity owner.
            let opaque=CanonicalIdentityBytesV1::from_owner_bytes(&[17;32]).unwrap();
            let sources=std::sync::Arc::new(seal_saved_v25_sources(
                collect_saved_source_supply("root.ddn","app/main.ddn",&opaque,&blobs).unwrap(),
            ).unwrap());
            let mut owner=owner.clone();
            owner.source_snapshot.files.iter_mut().find(|f|f.path=="app/main.ddn").unwrap().text=text.into();
            admit_saved_authored_project(sources,owner.save().unwrap().as_bytes()).unwrap()
        };
        let first=admit(body,&project);
        let reordered=admit(&body.replace("도우~도와","도와~도우"),&project);
        let spaced=admit(&body.replace("1 보여주기", "  1   보여주기"),&project);
        assert_ne!(first.sources().definition_capture().identity(),reordered.sources().definition_capture().identity());
        assert_eq!(first.definition_identity(),reordered.definition_identity());
        assert_eq!(first.definition_identity(),spaced.definition_identity());
        let altered=admit(&body.replace("1 보여주기","2 보여주기"),&project);
        assert_ne!(first.definition_identity(),altered.definition_identity());
        let pin_changed=admit(&body.replace("값:수", "다른값:수").replace("값 +", "다른값 +"),&project);
        assert_ne!(first.definition_identity(),pin_changed.definition_identity());
        assert_ne!(first.definition_identity().as_bytes().as_slice(), &[17;32]);
        let moved_root=root.replace("app/main.ddn","moved/start.ddn");
        let mut moved=project.clone();
        moved.source_snapshot.entry_path="moved/start.ddn".into();
        moved.source_snapshot.files=vec![SourceFile {path:"root.ddn".into(),text:moved_root.clone()},
            SourceFile {path:"moved/start.ddn".into(),text:body.into()}];
        moved.authored_identity.modules[0].head="moved/start.ddn".into();
        moved.authored_identity.modules[0].sources=vec!["moved/start.ddn".into()];
        let opaque=CanonicalIdentityBytesV1::from_owner_bytes(&[17;32]).unwrap();
        let moved_blobs=[SavedSourceBlobV1 {path:"root.ddn",role:"root",bytes:moved_root.as_bytes()},
            SavedSourceBlobV1 {path:"moved/start.ddn",role:"ddn_source",bytes:body.as_bytes()}];
        let moved_sources=std::sync::Arc::new(seal_saved_v25_sources(
            collect_saved_source_supply("root.ddn","moved/start.ddn",&opaque,&moved_blobs).unwrap(),
        ).unwrap());
        let relocated=admit_saved_authored_project(moved_sources,moved.save().unwrap().as_bytes()).unwrap();
        assert_eq!(first.definition_identity(),relocated.definition_identity(),"paths are locators, not definition identity");
        let duplicate=project.duplicate().unwrap();
        assert_ne!(first.definition_identity(),admit(body,&duplicate).definition_identity());
    }

    const ROOT: &[u8] =
        "프로젝트 { 이름: \"저장 검증\". 첫글: \"./app/한글😀.ddn\". }.\n".as_bytes();
    const ENTRY: &[u8] = "값:수 := 1.\r\n값 보여주기.\r\n".as_bytes();
    const ENTRY_PATH: &str = "app/한글😀.ddn";

    fn fixture() -> [SavedSourceBlobV1<'static>; 2] {
        [
            SavedSourceBlobV1 {
                path: "뿌리.ddn",
                role: "root",
                bytes: ROOT,
            },
            SavedSourceBlobV1 {
                path: ENTRY_PATH,
                role: "ddn_source",
                bytes: ENTRY,
            },
        ]
    }

    fn collect(blobs: &[SavedSourceBlobV1<'_>]) -> Result<VerifiedSavedSourceV1, Failure> {
        let definition = CanonicalIdentityBytesV1::from_owner_bytes(&[0x44, 0, 0xfe]).unwrap();
        collect_saved_source_supply("뿌리.ddn", ENTRY_PATH, &definition, blobs)
    }

    #[test]
    fn rd679_saved_source_binds_exact_bytes_and_is_order_independent() {
        use sha2::{Digest, Sha256};
        let mut blobs = fixture();
        let first = collect(&blobs).unwrap();
        blobs.reverse();
        let second = collect(&blobs).unwrap();
        second.verify_manifest(first.manifest()).unwrap();
        assert_eq!(
            first.official_entry_definition().unwrap().as_bytes(),
            [0x44, 0, 0xfe]
        );
        let file = first
            .source()
            .files
            .iter()
            .find(|f| f.path == ENTRY_PATH)
            .unwrap();
        assert_eq!(file.utf8_text.as_bytes(), ENTRY);
        let entry = first
            .manifest()
            .files
            .iter()
            .find(|f| f.path == ENTRY_PATH)
            .unwrap();
        assert_eq!(entry.byte_length, ENTRY.len());
        assert_eq!(entry.sha256, hex::encode(Sha256::digest(ENTRY)));
        assert_eq!(first.manifest().file_count, 2);
        assert_eq!(first.manifest().total_utf8_bytes, ROOT.len() + ENTRY.len());
        assert_eq!(first.source().files[0].path, ENTRY_PATH);
    }

    #[test]
    fn rd679_saved_source_rejects_missing_duplicate_and_case_colliding_entry() {
        let blobs = fixture();
        assert_eq!(collect(&blobs[..1]).unwrap_err(), Failure::CustodyConflict);
        assert_eq!(
            collect(&[blobs[0], blobs[1], blobs[1]]).unwrap_err(),
            Failure::CustodyConflict
        );
        let collision = SavedSourceBlobV1 {
            path: "APP/한글😀.ddn",
            ..blobs[1]
        };
        assert_eq!(
            collect(&[blobs[0], blobs[1], collision]).unwrap_err(),
            Failure::CustodyConflict
        );
        let wrong_role = SavedSourceBlobV1 {
            role: "metadata",
            ..blobs[1]
        };
        assert_eq!(
            collect(&[blobs[0], wrong_role]).unwrap_err(),
            Failure::CustodyConflict
        );
    }

    #[test]
    fn rd679_saved_source_rejects_path_repair_instead_of_normalizing() {
        for path in [
            "",
            "/app/a.ddn",
            "C:/a.ddn",
            "file://a.ddn",
            "./a.ddn",
            "a/../b.ddn",
            "a//b.ddn",
            "a\\b.ddn",
            " app/a.ddn",
            "app/a.ddn ",
            "app/가.ddn",
        ] {
            let blobs = fixture();
            let bad = SavedSourceBlobV1 { path, ..blobs[1] };
            assert_eq!(
                collect(&[blobs[0], bad]).unwrap_err(),
                Failure::InvalidInput,
                "{path:?}"
            );
        }
    }

    #[test]
    fn rd679_saved_source_rejects_bom_and_malformed_utf8_without_repair() {
        for bytes in [
            &[0xef, 0xbb, 0xbf, b'x'][..],
            &[0xff],
            &[0xc0, 0xaf],
            &[0xf0, 0x9f],
        ] {
            let blobs = fixture();
            let bad = SavedSourceBlobV1 { bytes, ..blobs[1] };
            assert_eq!(
                collect(&[blobs[0], bad]).unwrap_err(),
                Failure::InvalidInput
            );
        }
    }

    #[test]
    fn rd679_saved_source_checks_bounds_before_decode_or_missing_entry() {
        let blobs = fixture();
        assert_eq!(
            collect(&vec![blobs[0]; MAX_PROJECT_FILES + 1]).unwrap_err(),
            Failure::LimitExceeded
        );
        let oversized = vec![0xff; MAX_FILE_BYTES + 1];
        let bad = SavedSourceBlobV1 {
            bytes: &oversized,
            ..blobs[1]
        };
        assert_eq!(
            collect(&[blobs[0], bad]).unwrap_err(),
            Failure::LimitExceeded
        );
        let full = vec![0xff; MAX_FILE_BYTES];
        let bad = SavedSourceBlobV1 {
            bytes: &full,
            ..blobs[0]
        };
        assert_eq!(collect(&[bad; 9]).unwrap_err(), Failure::LimitExceeded);
        let long_path = "a".repeat(MAX_REVISION_ENVELOPE_BYTES + 1);
        let bad = SavedSourceBlobV1 {
            path: &long_path,
            ..blobs[0]
        };
        assert_eq!(collect(&[bad]).unwrap_err(), Failure::LimitExceeded);
    }

    #[test]
    fn rd679_manifest_cannot_substitute_changed_bytes_or_metadata() {
        let blobs = fixture();
        let saved = collect(&blobs).unwrap();
        let changed = SavedSourceBlobV1 {
            bytes: "값:수 := 2.\r\n값 보여주기.\r\n".as_bytes(),
            ..blobs[1]
        };
        let other = collect(&[blobs[0], changed]).unwrap();
        assert_eq!(
            other.verify_manifest(saved.manifest()),
            Err(Failure::CustodyConflict)
        );
        let mut candidate = saved.manifest().clone();
        candidate.files[0].byte_length += 1;
        assert_eq!(
            saved.verify_manifest(&candidate),
            Err(Failure::CustodyConflict)
        );
        let mut candidate = saved.manifest().clone();
        candidate.files.reverse();
        assert_eq!(
            saved.verify_manifest(&candidate),
            Err(Failure::CustodyConflict)
        );
        let mut candidate = saved.manifest().clone();
        candidate.canonical_manifest_sha256 = "00".repeat(32);
        assert_eq!(
            saved.verify_manifest(&candidate),
            Err(Failure::CustodyConflict)
        );
        saved.verify_manifest(saved.manifest()).unwrap();
        assert_eq!(saved.source().files[0].utf8_text.as_bytes(), ENTRY);
    }

    #[test]
    fn rd679_saved_role_aliases_do_not_become_canonical_authority() {
        let blobs = fixture();
        let alias = SavedSourceBlobV1 {
            path: "meta.txt",
            role: " metadata ",
            bytes: b"ok",
        };
        assert_eq!(
            collect(&[blobs[0], blobs[1], alias]).unwrap_err(),
            Failure::CustodyConflict
        );
    }

    #[test]
    fn rd679_identity_projection_preserves_owner_bytes_exactly() {
        let raw = [0, 1, 127, 128, 255];
        let owner = CanonicalIdentityBytesV1::from_owner_bytes(&raw).unwrap();
        for family in [
            IdentityFamily::Project,
            IdentityFamily::HistoryRevision,
            IdentityFamily::SourceEpoch,
            IdentityFamily::ProjectSemanticConfiguration,
            IdentityFamily::Definition,
            IdentityFamily::DependencyResolution,
        ] {
            let text = issue_rd631_identity_projection(family, &owner).unwrap();
            assert_eq!(text, format!("{}00017f80ff", family.prefix()));
            assert_eq!(
                verify_rd631_identity_projection(family, &text)
                    .unwrap()
                    .as_bytes(),
                raw
            );
            assert_eq!(
                issue_rd631_identity_projection(family, &owner).unwrap(),
                text
            );
        }
    }

    #[test]
    fn rd679_identity_projection_rejects_alias_case_whitespace_and_partial_hex() {
        for text in [
            "ProjectIdentity:",
            "ProjectIdentity:0",
            "ProjectIdentity:FF",
            "ProjectIdentity:gg",
            "projectidentity:00",
            "DefinitionIdentity:00",
            " ProjectIdentity:00",
            "ProjectIdentity:00 ",
            "ProjectIdentity:０１",
        ] {
            assert_eq!(
                verify_rd631_identity_projection(IdentityFamily::Project, text),
                Err(Failure::InvalidInput),
                "{text}"
            );
        }
    }

    #[test]
    fn rd679_identity_bounds_precede_projection_or_decode_allocation() {
        assert_eq!(
            CanonicalIdentityBytesV1::from_owner_bytes(&[]),
            Err(Failure::InvalidInput)
        );
        assert_eq!(
            CanonicalIdentityBytesV1::from_owner_bytes(&[0; MAX_IDENTITY_BYTES + 1]),
            Err(Failure::LimitExceeded)
        );
        let owner = CanonicalIdentityBytesV1::from_owner_bytes(&[255; MAX_IDENTITY_BYTES]).unwrap();
        let text = issue_rd631_identity_projection(IdentityFamily::Project, &owner).unwrap();
        assert_eq!(
            verify_rd631_identity_projection(IdentityFamily::Project, &text).unwrap(),
            owner
        );
        assert_eq!(
            verify_rd631_identity_projection(IdentityFamily::Project, &(text + "00")),
            Err(Failure::LimitExceeded)
        );
    }
}
