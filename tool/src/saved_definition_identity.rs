//! A3-P8: product-owned physical projection of a saved, resolved Start.
//! RD-325/326 declaration semantics are not source-capture or AST identities.
use ddonirang_lang::{SeedKind, TopLevelItem, TypeRef};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SavedDefinitionIdentity([u8; 32]);
impl SavedDefinitionIdentity {
    pub(crate) fn as_bytes(&self) -> &[u8; 32] { &self.0 }
}

fn type_material(t: &TypeRef) -> Value {
    match t {
        TypeRef::Tuple(items) => json!(["tuple", items.iter().map(type_material).collect::<Vec<_>>()]),
        TypeRef::Infer => json!(["infer"]),
        TypeRef::Named(name) => json!(["named", name]),
        TypeRef::Tensor { element, shape } => json!(["tensor", type_material(element), shape]),
        TypeRef::Applied { name, args } => json!(["applied", name, args.iter().map(type_material).collect::<Vec<_>>()]),
        TypeRef::Function { params, result } => json!(["function", params.iter().map(type_material).collect::<Vec<_>>(), type_material(result)]),
    }
}

fn kind_material(kind: &SeedKind) -> Value {
    match kind {
        SeedKind::Imeumssi => json!(["imeumssi"]),
        SeedKind::Umjikssi => json!(["umjikssi"]),
        SeedKind::ValueFunc => json!(["value_func"]),
        SeedKind::Gallaessi => json!(["gallaessi"]),
        SeedKind::Relationssi => json!(["relationssi"]),
        SeedKind::Sam => json!(["sam"]),
        SeedKind::Heureumssi => json!(["heureumssi"]),
        SeedKind::Ieumssi => json!(["ieumssi"]),
        SeedKind::Semssi => json!(["semssi"]),
        SeedKind::Product => json!(["product"]),
        SeedKind::Schema(ddonirang_lang::ast::SchemaCategory::Subject) => json!(["schema", "subject"]),
        SeedKind::Schema(ddonirang_lang::ast::SchemaCategory::NominalValue) => json!(["schema", "nominal_value"]),
        SeedKind::Named(name) => json!(["named", name]),
    }
}

/// Only the admitted explicit project owner can supply permanent module IDs.
/// No path/name/content-derived module ID, random issuance or ordinal here.
pub(crate) fn issue_saved_start(
    sources: &crate::project_revision_envelope_supply::VerifiedSavedV25Sources,
    project: &crate::authored_project::ProjectArtifact,
    entry_module_index: usize,
) -> Result<SavedDefinitionIdentity, String> {
    issue_saved_start_impl(sources,project,entry_module_index,None)
}
pub(crate) fn issue_registered_saved_start(
    source:&crate::project_revision_envelope_supply::PreCanonAuthoredSourceV1,
    draft:&crate::authored_project::registration::RegistrationDraft,
    entry_module_index:usize,
)->Result<SavedDefinitionIdentity,String> {
    let binding=draft.resolved_start_material(source)?;
    issue_saved_start_impl(source.sources(),source.project(),entry_module_index,binding.as_ref())
}
fn issue_saved_start_impl(
    sources:&crate::project_revision_envelope_supply::VerifiedSavedV25Sources,
    project:&crate::authored_project::ProjectArtifact,entry_module_index:usize,
    binding:Option<&(String,Vec<u8>,Vec<u8>)>,
)->Result<SavedDefinitionIdentity,String> {
    let preparation = sources.preparation();
    let mut program = ddonirang_lang::parse_v25_with_mode(
        &preparation.source, &preparation.target_path, ddonirang_lang::ParseMode::Strict,
    ).map_err(|e| format!("E_SAVED_DEFINITION_PARSE: {e}"))?;
    ddonirang_lang::canonicalize(&mut program)
        .map_err(|e| format!("E_SAVED_DEFINITION_CANON: {e}"))?;
    let name = sources.entry_declaration().name();
    let mut matching = program.items.iter().filter_map(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        (seed.canonical_name == name).then_some(seed)
    });
    let start = matching.next().ok_or("E_SAVED_DEFINITION_START: 시작 정의가 없습니다")?;
    if matching.next().is_some() || start.seed_kind != SeedKind::Umjikssi || !start.params.is_empty() {
        return Err("E_SAVED_DEFINITION_START: 시작은 유일한 받는자리 없는 움직씨여야 합니다".into());
    }
    let entry_module = project.authored_identity.modules.get(entry_module_index)
        .ok_or("E_SAVED_DEFINITION_OWNER: 시작 모듈 정체가 없습니다")?;
    let mut heads = Vec::with_capacity(program.items.len());
    for item in &program.items {
        let TopLevelItem::SeedDef(seed) = item;
        // Modifier currently has no semantic representation beyond provenance.
        // Never silently omit it from an identity if a future parser admits it.
        if !seed.modifiers.is_empty() {
            return Err("E_SAVED_DEFINITION_MODIFIER_UNSUPPORTED: 정의 매김의 저장 정체를 해소할 수 없습니다".into());
        }
        let (path, _) = preparation.original_span(seed.span)?;
        let mut owners = project.authored_identity.modules.iter()
            .filter(|module| module.sources.iter().any(|source| source == &path));
        let owner = owners.next().ok_or("E_SAVED_DEFINITION_OWNER: 정의의 소유 모듈이 없습니다")?;
        if owners.next().is_some() {
            return Err("E_SAVED_DEFINITION_OWNER: 정의 소유 모듈이 유일하지 않습니다".into());
        }
        heads.push(json!({
            "module": owner.id.as_str(), "name": seed.canonical_name,
            "alternates": seed.alternate_forms, "kind": kind_material(&seed.seed_kind),
            "result": seed.result_type.as_ref().map(type_material),
            "pins": seed.params.iter().map(|pin| json!({
                "name": pin.pin_name, "type": type_material(&pin.type_ref),
                "roles": pin.josa_list, "optional": pin.optional,
                "default": pin.default_value.as_ref().map(ddonirang_lang::normalizer::normalize_expression_identity),
            })).collect::<Vec<_>>(),
        }));
    }
    // N1 owns bodies and lexical resolution. Explicit heads also retain the
    // full pin/result/alternate material (including ValueFunc alternates).
    // Binding root statements remain context; they never mint definitions.
    let mut context=json!({
        "source_epoch": "v25", "compiler_profile": "v1-core-v25",
        "entry_module": entry_module.id.as_str(), "entry_definition": name,
        "declarations": heads,
        "resolved_program": binding.map(|value|value.0.clone()).unwrap_or_else(||ddonirang_lang::normalize(&program, ddonirang_lang::NormalizationLevel::N1)),
    });
    if let Some((_,catalog,manifest))=binding {
        context["compiler_source_admission"]=json!("resolved-supply-catalog.r3.3");
        context["resolved_catalog_sha256"]=json!(hex::encode(Sha256::digest(catalog)));
        context["resolved_builtin_manifest"]=json!(hex::encode(manifest));
    }
    let material = serde_json::to_vec(&context).map_err(|_| "E_SAVED_DEFINITION_ENCODING: 정의 정체를 직렬화할 수 없습니다")?;
    if material.len() > 32_000_000 {
        return Err("E_SAVED_DEFINITION_LIMIT: 정의 정체 해소 한도를 넘었습니다".into());
    }
    let mut digest = Sha256::new();
    digest.update(b"ddn.saved_definition_identity.v1\0");
    digest.update(if binding.is_some(){3u16}else{1u16}.to_be_bytes());
    digest.update((material.len() as u32).to_be_bytes());
    digest.update(material);
    Ok(SavedDefinitionIdentity(digest.finalize().into()))
}
