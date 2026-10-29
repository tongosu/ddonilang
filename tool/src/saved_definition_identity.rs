//! A3-P8: product-owned physical projection of a saved, resolved Start.
//! RD-325/326 declaration semantics are not source-capture or AST identities.
use ddonirang_lang::{SeedKind, TopLevelItem, TypeRef};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
mod proof_source;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SavedDefinitionIdentity([u8; 32]);
impl SavedDefinitionIdentity {
    pub(crate) fn as_bytes(&self) -> &[u8; 32] { &self.0 }
}

fn type_material(t: &TypeRef) -> Result<Value, String> {
    Ok(match t {
        TypeRef::AlgebraEquality { .. } => return Err("E_SAVED_DEFINITION_TYPE_UNSUPPORTED: 대수 목표의 저장 정체는 지원하지 않습니다".into()),
        TypeRef::StructureLaw { .. } => return Err("E_SAVED_DEFINITION_TYPE_UNSUPPORTED: 구조 법칙 목표의 저장 정체는 지원하지 않습니다".into()),
        // Preserve the material of every previously supported type exactly.
        TypeRef::PropositionConjunction(items) => json!(["proposition_conjunction", items.iter().map(type_material).collect::<Result<Vec<_>,_>>()?]),
        TypeRef::Tuple(items) => json!(["tuple", items.iter().map(type_material).collect::<Result<Vec<_>,_>>()?]),
        TypeRef::Infer => json!(["infer"]),
        TypeRef::Named(name) => json!(["named", name]),
        TypeRef::Tensor { element, shape } => json!(["tensor", type_material(element)?, shape]),
        TypeRef::Applied { name, args } => json!(["applied", name, args.iter().map(type_material).collect::<Result<Vec<_>,_>>()?]),
        TypeRef::Function { params, result } => json!(["function", params.iter().map(type_material).collect::<Result<Vec<_>,_>>()?, type_material(result)?]),
    })
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

#[cfg(test)]
mod structure_goal_storage_tests {
    use super::*;

    #[test]
    fn nested_proof_goal_never_becomes_saved_type_material() {
        let goal = TypeRef::StructureLaw { structure: "구조".into(), member: "법칙".into() };
        for kind in [goal.clone(), TypeRef::Tuple(vec![goal.clone()]),
            TypeRef::Applied { name: "보임".into(), args: vec![goal.clone()] },
            TypeRef::Function { params: vec![], result: Box::new(goal.clone()) },
            TypeRef::Function { params: vec![goal.clone()], result: Box::new(TypeRef::Infer) },
            TypeRef::Tensor { element: Box::new(goal), shape: vec![1] }] {
            assert!(type_material(&kind).unwrap_err().starts_with("E_SAVED_DEFINITION_TYPE_UNSUPPORTED:"));
        }
    }

    #[test]
    fn existing_material_bytes_are_unchanged() {
        let kind = TypeRef::Function {
            params: vec![TypeRef::Tuple(vec![TypeRef::Named("바른수".into()), TypeRef::Infer])],
            result: Box::new(TypeRef::Applied { name: "차림".into(), args: vec![TypeRef::Named("글".into())] }),
        };
        assert_eq!(serde_json::to_string(&type_material(&kind).unwrap()).unwrap(),
            r#"["function",[["tuple",[["named","바른수"],["infer"]]]],["applied","차림",[["named","글"]]]]"#);
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
    // All source admission and current owner material are still checked by
    // this call. Only exact syntax parsing/canonicalization may be reused.
    let program=if let Some(first)=sources.first_callable_context(){
        let checked=crate::ddn_runtime::DdnProgram::from_first_callable_authored_source(&preparation.source,&preparation.target_path,first.clone())?;
        let mut program=checked.canonical_program().clone();
        let original=first.compiler_context().original_public().declaration_program.as_ref().ok_or("E_DDL_ORIGINAL_DECLARATION_REQUIRED")?;
        let TopLevelItem::SeedDef(imported)=&original.items[0];
        program.items.retain(|TopLevelItem::SeedDef(seed)|seed.canonical_name!=imported.canonical_name);
        program
    }else{crate::ddn_runtime::canonical_definition_program(&preparation.source,&preparation.target_path)?};
    require_supported_mathematical_bodies(&program,&preparation.source,&preparation.target_path)?;
    let source_types=proof_source::SourceTypes::for_program(&program,&preparation.source,&preparation.target_path)?;
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
        let actual=source_types.as_ref().map(|types|types.original_definition(seed)).transpose()?;
        let material=|kind:&TypeRef,raw:&TypeRef|->Result<Value,String>{
            match (&source_types,actual) {
                (Some(types),Some(actual))=>types.material(kind,raw,actual),
                _=>type_material(kind),
            }
        };
        if actual.is_some_and(|raw|raw.params.len()!=seed.params.len()){
            return Err("E_SAVED_DEFINITION_TYPE_UNSUPPORTED: 입력 발생 위치가 달라졌습니다".into());
        }
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
            "result": seed.result_type.as_ref().map(|kind|material(kind,actual.and_then(|raw|raw.result_type.as_ref()).unwrap_or(kind))).transpose()?,
            "pins": seed.params.iter().enumerate().map(|(i,pin)| -> Result<Value, String> { Ok(json!({
                "name": pin.pin_name, "type": material(&pin.type_ref,actual.map(|raw|&raw.params[i].type_ref).unwrap_or(&pin.type_ref))?,
                "roles": pin.josa_list, "optional": pin.optional,
                "default": pin.default_value.as_ref().map(ddonirang_lang::normalizer::normalize_expression_identity),
            })) }).collect::<Result<Vec<_>,_>>()?,
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
    if let Some(first)=sources.first_callable_context(){
        // Keep external original ownership distinct from the consumer's heads.
        // Existing v1/v2 material and family/domain bytes remain unchanged.
        context["first_callable_dependency"]=first.saved_dependency_material()?;
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

// Do not issue a saved definition identity from headers while its domain,
// selected operations and checked participation are not supplied. Source-only
// syntax is neither an ordinary empty callable nor a verified function value.
fn require_supported_mathematical_bodies(program:&ddonirang_lang::CanonProgram,source:&str,path:&str)->Result<(),String>{
    crate::ddn_runtime::verify_piecewise_saved_definition(program,source,path)
}

#[cfg(test)]
mod wo30_piecewise_tests {
    use super::*;
    #[test]
    fn wo30_piecewise_saved_identity_rechecks_mathematical_body_admission(){
        let source="요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: 0 <= 사용량 <= 6. 10 * 사용량 : 0 <= 사용량 <= 6. }.";
        let p=ddonirang_lang::parse_v25_with_mode(source,"요금.ddn",ddonirang_lang::ParseMode::Strict).unwrap();
        assert!(require_supported_mathematical_bodies(&p,source,"요금.ddn").is_ok());
        let changed=source.replace("10 * 사용량","20 * 사용량");
        assert!(require_supported_mathematical_bodies(&p,&changed,"요금.ddn").is_err());
        let q=ddonirang_lang::parse_v25_with_mode(&changed,"요금.ddn",ddonirang_lang::ParseMode::Strict).unwrap();
        assert_ne!(ddonirang_lang::normalize(&p,ddonirang_lang::NormalizationLevel::N1),
            ddonirang_lang::normalize(&q,ddonirang_lang::NormalizationLevel::N1));
    }
}

#[cfg(test)]
mod syntax_cache_owner_tests {
    use super::*;
    use crate::authored_project::{ProjectArtifact,SourceSnapshot,SourceFile};

    #[test]
    fn cached_syntax_never_reuses_a_saved_definition_owner() {
        let root="(글판=\"v25\", 모듬=차림[(이름=앱, 머리=\"main.ddn\", 원문=\"main.ddn\")], 시작=(모듈=앱, 정의=실행)) 얽음.\n";
        let body="실행:움직씨 := { 1 보여주기. }.\n";
        let project=ProjectArtifact::create("v1-core-v25".into(),SourceSnapshot {
            root_path:Some("root.ddn".into()),entry_path:"main.ddn".into(),
            files:vec![SourceFile{path:"root.ddn".into(),text:root.into()},
                SourceFile{path:"main.ddn".into(),text:body.into()}],
        },String::new()).unwrap();
        let raw=project.save().unwrap();
        let admit=|raw:&str|crate::project_revision_envelope_supply::admit_fresh_authored_project(raw.as_bytes()).unwrap();
        let original=admit(&raw).definition_identity().clone();
        let _scope=crate::ddn_runtime::SavedProgramRebuildScope::begin();
        assert_eq!(original,*admit(&raw).definition_identity());
        // Identical source syntax does not carry the project's permanent module ID.
        let duplicate=project.duplicate().unwrap();
        assert_ne!(original,*admit(&duplicate.save().unwrap()).definition_identity());
        assert_eq!(original,*admit(&raw).definition_identity());
        let mut changed=project.clone();
        changed.source_snapshot.files.iter_mut().find(|f|f.path=="main.ddn").unwrap().text=body.replace("1 보여주기","2 보여주기");
        assert_ne!(original,*admit(&changed.save().unwrap()).definition_identity());
        assert_eq!(original,*admit(&raw).definition_identity());
    }
}
