//! 작품 모듈을 연결하는 기존 처리. 구조체 정체와 오류 처리는 부모와 동일하다.
// F5: 이 linker가 소유한 graph의 module_paths/module_sources는 함께 삽입한다.
// closure 경로는 같은 정규 원문 map에 존재하고 directives도 같은 모듈 키로 만든다.
// DFS는 없는 import/순환을 거부한 뒤 순서를 확정하므로 후속 키 접근은 그 집합 안이다.
// 시작 모듈은 checked get으로 확인하며 matches[0]은 len()!=1 단락 평가 뒤에만 읽는다.
// 원래 raw source.files에서 머리를 찾는 별도 조회는 이 map 근거로 보장되지 않는다.
// 보존한 legacy linker는 현재 호출이 없는 후보이며 새 기능/입구로 수용하지 않는다.
use super::*;

pub(super) fn prepare_logic_callable_authored_execution(source:VirtualProjectSource,syntax:ddonirang_lang::ddl::CheckedJ1Profile,
    context:Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>,
)->Result<AuthoredPreparation,LocalProjectError>{
    let activation=context.activation().ok_or_else(||LocalProjectError::new("E_DDL_ACTIVATION_REQUIRED","명시 내장 모듈 결속이 없습니다"))?;
    if activation.target!="logic"||!context.profile().is_logic_builtin(){return Err(LocalProjectError::new("E_LOGIC_PROVIDER_REQUIRED","등록된 논리 builtin 공급이 필요합니다"));}
    let graph=build_project_graph_selected(source.clone(),Some(syntax))?;
    if graph.module_paths.contains_key(&activation.target){return Err(LocalProjectError::new("E_LOGIC_PROVIDER_COLLISION","같은 이름의 사용자 모듈과 builtin을 추측 선택하지 않습니다"));}
    if graph.module_sources.len()!=1||graph.module_sources.values().next().map(Vec::len)!=Some(1){
        return Err(LocalProjectError::new("E_LOGIC_SINGLE_ENTRY_SCOPE","첫 builtin 실행은 실제 단일 소비 모듈을 사용합니다"));
    }
    let path=&graph.entry_path;
    let file=source.files.iter().find(|f|&f.path==path).ok_or_else(||LocalProjectError::new("E_LOGIC_SOURCE_REQUIRED","소비 원문이 없습니다"))?;
    let program=ddonirang_lang::Parser::parse_with_logic_callable_context(&file.utf8_text,path,syntax,context)
        .map_err(|e|{
            let mut error=LocalProjectError::with_path_span(e.j1_source_code(),path,e.message,(e.span.start,e.span.end));
            error.phase=Some("lang_parse");error
        })?;
    let candidates=program.items.iter().filter_map(|TopLevelItem::SeedDef(seed)|(Some(&seed.canonical_name)==graph.start_definition.as_ref()).then_some(seed)).collect::<Vec<_>>();
    if !program.root_stmts.is_empty()||candidates.len()!=1||candidates[0].seed_kind!=ddonirang_lang::SeedKind::Umjikssi
        ||!candidates[0].params.is_empty()||candidates[0].body.is_none(){
        return Err(LocalProjectError::new("E_PROJECT_START_DEFINITION","시작은 같은 모듈의 인자 없는 움직씨 하나여야 합니다"));
    }
    Ok(AuthoredPreparation{source:file.utf8_text.clone(),target_path:path.clone(),start_definition:graph.start_definition,
        source_map:vec![LinkedSourceSegment{generated_start:0,generated_end:file.utf8_text.len(),source_path:path.clone(),source_boundaries:(0..=file.utf8_text.len()).collect()}],
        // Builtin identities are not imported authored-source declarations.
        imported_definitions:BTreeSet::new(),original_declarations:Vec::new(),requires_registered_nominal_owner:false})
}

pub(super) fn build_linked_project(
    source: VirtualProjectSource,
    target_path_override: Option<&str>,
) -> Result<LinkedProject, LocalProjectError> {
    let project_asset_registry = project_asset_registry_from_source(&source)?;
    build_linked_project_with_options(source, target_path_override, true, project_asset_registry)
}

pub(super) fn prepare_public_callable_authored_execution(
    source:VirtualProjectSource,artifact:&crate::authored_project::PublicCallableProjectArtifactV4,
)->Result<AuthoredPreparation,LocalProjectError>{
    let context=artifact.restore_context().map_err(|e|LocalProjectError::new("E_DDL_V4_SOURCE_OWNER",e))?;
    let snapshot=&artifact.source_snapshot;
    if snapshot.root_path.as_deref()!=Some(source.root_path.as_str())||snapshot.entry_path!=source.entry_path
        ||snapshot.files.len()!=source.files.len()||snapshot.files.iter().any(|file|!source.files.iter().any(|raw|raw.path==file.path&&raw.utf8_text.as_bytes()==file.text.as_bytes())){
        return Err(LocalProjectError::new("E_DDL_V4_SOURCE_OWNER","실제 소비 작품의 원자료와 공급이 다릅니다"));
    }
    let graph=build_project_graph_selected(source.clone(),context.compiler_context().consumer_j1())?;
    let path=artifact.consumer_path();
    if graph.entry_path!=path||graph.module_sources.len()!=1||graph.module_sources.values().next().map(Vec::len)!=Some(1){
        return Err(LocalProjectError::new("E_DDL_NATIVE_SINGLE_ENTRY_SCOPE","소비 시작 원문은 실제 단일 모듈 원문을 사용합니다; 원본의 전체 공개 분모는 축소하지 않습니다"));
    }
    let file=source.files.iter().find(|file|file.path==path).ok_or_else(||LocalProjectError::new("E_DDL_CONSUMER_SOURCE_REQUIRED","소비 원문이 없습니다"))?;
    context.verify_runtime_activation(&file.utf8_text,path).map_err(|e|LocalProjectError::new("E_DDL_V4_SOURCE_OWNER",e))?;
    let program=ddonirang_lang::parser::Parser::parse_with_public_callable_context(&file.utf8_text,path,context.compiler_context().clone())
        .map_err(|e|LocalProjectError::with_path("E_DDL_NATIVE_SOURCE_PARSE",path,e.to_string()))?;
    if !program.root_stmts.is_empty(){return Err(LocalProjectError::with_path("E_PROJECT_TOP_LEVEL_EXECUTION",path,"실행 문장은 실제 시작 움직씨 본문에 작성해 주세요"));}
    let matches=program.items.iter().filter_map(|TopLevelItem::SeedDef(seed)|(Some(&seed.canonical_name)==graph.start_definition.as_ref()).then_some(seed)).collect::<Vec<_>>();
    if matches.len()!=1||matches[0].seed_kind!=ddonirang_lang::SeedKind::Umjikssi||!matches[0].params.is_empty()||matches[0].body.is_none(){
        return Err(LocalProjectError::new("E_PROJECT_START_DEFINITION","시작은 같은 모듈의 인자 없는 움직씨 정의 하나를 가리켜야 합니다"));
    }
    Ok(AuthoredPreparation{source:file.utf8_text.clone(),target_path:path.into(),start_definition:graph.start_definition,
        source_map:vec![LinkedSourceSegment{generated_start:0,generated_end:file.utf8_text.len(),source_path:path.into(),source_boundaries:(0..=file.utf8_text.len()).collect()}],
        imported_definitions:BTreeSet::new(),original_declarations:Vec::new(),requires_registered_nominal_owner:false})
}

pub(super) fn prepare_first_callable_authored_execution(
    source:VirtualProjectSource,first:&crate::authored_project::FirstCallableAuthoredContext,
)->Result<AuthoredPreparation,LocalProjectError>{
    let (path,text)=first.checked_native_source(&source).map_err(|e|LocalProjectError::new("E_DDL_NATIVE_SOURCE_OWNER",e))?;
    let graph=build_project_graph(source.clone())?;
    if graph.entry_path!=path||graph.module_sources.len()!=1||graph.module_sources.values().next().map(Vec::len)!=Some(1){return Err(LocalProjectError::new("E_DDL_NATIVE_SINGLE_ENTRY_SCOPE","한정 일본어 호출은 실제 시작 원문 하나를 사용합니다"));}
    let program=first.parse_consumer_owned_program(text,path).map_err(|e|LocalProjectError::with_path("E_DDL_NATIVE_SOURCE_PARSE",path,e))?;
    if !program.root_stmts.is_empty(){return Err(LocalProjectError::with_path("E_PROJECT_TOP_LEVEL_EXECUTION",path,"실행 문장은 실제 시작 움직씨 본문에 작성해 주세요"));}
    let matches=program.items.iter().filter_map(|TopLevelItem::SeedDef(seed)|(Some(&seed.canonical_name)==graph.start_definition.as_ref()).then_some(seed)).collect::<Vec<_>>();
    if matches.len()!=1||matches[0].seed_kind!=ddonirang_lang::SeedKind::Umjikssi||!matches[0].params.is_empty()||matches[0].body.is_none(){return Err(LocalProjectError::new("E_PROJECT_START_DEFINITION","시작은 같은 모듈의 인자 없는 움직씨 정의 하나를 가리켜야 합니다"));}
    Ok(AuthoredPreparation{source:text.into(),target_path:path.into(),start_definition:graph.start_definition,
        source_map:vec![LinkedSourceSegment{generated_start:0,generated_end:text.len(),source_path:path.into(),source_boundaries:(0..=text.len()).collect()}],
        imported_definitions:BTreeSet::new(),original_declarations:Vec::new(),requires_registered_nominal_owner:false})
}

pub(super) fn build_linked_project_with_options(
    source: VirtualProjectSource,
    target_path_override: Option<&str>,
    run_for_symbol_values: bool,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
) -> Result<LinkedProject, LocalProjectError> {
    build_linked_project_selected(source,target_path_override,run_for_symbol_values,project_asset_registry,None,None,None)
}
pub(super) fn build_linked_project_with_j1_profile(source:VirtualProjectSource,target_path_override:Option<&str>,run_for_symbol_values:bool,project_asset_registry:Option<Arc<ProjectAssetRegistry>>,profile:ddonirang_lang::ddl::CheckedJ1Profile)->Result<LinkedProject,LocalProjectError>{
    build_linked_project_selected(source,target_path_override,run_for_symbol_values,project_asset_registry,Some(profile),None,None)
}
/// Link an explicitly checked B1 module through the ordinary project graph.
pub(super) fn link_checked_logic_module(
    source: VirtualProjectSource,
    path: &str,
    syntax: ddonirang_lang::ddl::CheckedJ1Profile,
    context: Arc<ddonirang_lang::parser::RegisteredLogicCallableContext>,
) -> Result<(AuthoredPreparation, ddonirang_lang::CanonProgram), LocalProjectError> {
    let file = source.files.iter().find(|file| file.path == path)
        .ok_or_else(|| LocalProjectError::with_path("E_LOGIC_SOURCE_REQUIRED", path, "소비 원문이 없습니다"))?;
    let original = ddonirang_lang::Parser::parse_with_logic_callable_context(&file.utf8_text, path, syntax, context)
        .map_err(|error| LocalProjectError::with_path_span(error.j1_source_code(), path, error.message, (error.span.start, error.span.end)))?;
    let linked = build_linked_project_selected(source, None, false, None, Some(syntax), Some((path, &original)), None)?;
    let preparation = AuthoredPreparation::from_linked_project(linked);
    let mut program = ddonirang_lang::Parser::parse_j1_v25_with_mode(
        &preparation.source, &preparation.target_path, syntax, ParseMode::Strict,
    ).map_err(|error| LocalProjectError::with_path_span(error.j1_source_code(), &preparation.target_path, error.message, (error.span.start, error.span.end)))?;
    carry_linked_logic(&preparation, &mut program, path, &original)?;
    Ok((preparation, program))
}

fn carry_linked_logic(
    preparation: &AuthoredPreparation, program: &mut ddonirang_lang::CanonProgram,
    path: &str, original: &ddonirang_lang::CanonProgram,
) -> Result<(), LocalProjectError> {
    let segment = preparation.source_map.iter().find(|segment| segment.source_path == path)
        .ok_or_else(|| LocalProjectError::with_path("E_LOGIC_SOURCE_REQUIRED", path, "링크한 소비 원문이 없습니다"))?;
    let range = Span::new(segment.generated_start, segment.generated_end);
    program.carry_logic_module_calls(original, range, |span| {
        let (owner, span) = preparation.original_span(span).ok()?;
        (owner == path).then_some(span)
    }).map_err(|error| LocalProjectError::with_path("E_LOGIC_LINKED_SOURCE_SITE_MISMATCH", path, error.message))
}

/// Link the local graph and bind both providers at their verified call sites.
/// The runtime still checks the admitted project captures and B1 proofs.
pub(super) fn link_checked_multilang_modules(
    project: &crate::authored_project::ProjectArtifact,
    public: &crate::authored_project::PublicCallableAuthoredContext,
    logic: &crate::authored_project::registration::logic_dependency::CheckedLogicDependency,
) -> Result<LinkedMultilangProgram, LocalProjectError> {
    let (public_path, prefix) = public.linked_module_prefix(project).map_err(|error|
        LocalProjectError::new("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED", error))?;
    let logic_path = logic.consumer_source_path();
    if public_path == logic_path {
        return Err(LocalProjectError::with_path("E_LOGIC_SOURCE_CONTEXT_CONFLICT", public_path,
            "한 모듈에 두 외부 등록을 결속할 수 없습니다"));
    }
    let syntax = public.compiler_context().consumer_j1().ok_or_else(|| LocalProjectError::new("E_J1_PROFILE_REQUIRED", "J1 판본이 필요합니다"))?;
    logic.verify_production_profile().map_err(|error| LocalProjectError::with_path(
        "E_LOGIC_DEPENDENCY_PAYLOAD", logic_path, error))?;
    let logic_context = logic.restore_context(project, &[]).map_err(|error| LocalProjectError::with_path(
        "E_LOGIC_DEPENDENCY_PAYLOAD", logic_path, error))?;
    let logic_file = project.source_snapshot.files.iter().find(|file| file.path == logic_path)
        .ok_or_else(|| LocalProjectError::with_path("E_LOGIC_SOURCE_REQUIRED", logic_path, "등록 원문이 없습니다"))?;
    let logic_program = ddonirang_lang::Parser::parse_with_logic_callable_context(
        &logic_file.text, logic_path, syntax, logic_context.clone(),
    ).map_err(|error| LocalProjectError::with_path_span(
        error.j1_source_code(), logic_path, error.message, (error.span.start, error.span.end),
    ))?;
    let source = crate::authored_project::snapshot_source_selected(&project.source_snapshot, Some(syntax))
        .map_err(|error| LocalProjectError::new("E_PROJECT_SOURCE_ADMISSION", error))?;
    let linked = build_linked_project_selected(
        source, None, false, None, Some(syntax),
        Some((logic_path, &logic_program)), Some((public_path, &prefix)),
    )?;
    let preparation = AuthoredPreparation::from_linked_project(linked);
    let mut program = ddonirang_lang::Parser::parse_j1_linked_public_consumer(
        &preparation.source, &preparation.target_path, syntax, public.compiler_context(),
    ).map_err(|error| linked_multilang_parse_error(&preparation, error))?;
    carry_linked_logic(&preparation, &mut program, logic_path, &logic_program)?;

    let public_file = project.source_snapshot.files.iter().find(|file| file.path == public_path)
        .ok_or_else(|| LocalProjectError::with_path("E_DDL_CONSUMER_SOURCE_REQUIRED", public_path, "등록 원문이 없습니다"))?;
    let original = ddonirang_lang::Parser::parse_unlinked_j1_module(&public_file.text, public_path, syntax)
        .map_err(|error| LocalProjectError::with_path_span(
            error.j1_source_code(), public_path, error.message, (error.span.start, error.span.end),
        ))?;
    let [TopLevelItem::SeedDef(original_seed)] = original.items.as_slice() else {
        return Err(LocalProjectError::with_path("E_MULTILANG_PUBLIC_CALLER_SCOPE", public_path,
            "사용자 DDL을 호출하는 로컬 함수 하나가 필요합니다"));
    };
    let mut linked_seeds = program.items.iter().filter_map(|TopLevelItem::SeedDef(seed)| {
        let (path, span) = preparation.original_span(seed.span).ok()?;
        (path == public_path && span == original_seed.span).then_some(seed)
    });
    let mismatch = || LocalProjectError::with_path_span(
        "E_MULTILANG_LINKED_SOURCE_SITE_MISMATCH", public_path,
        "링크 전후의 등록 호출 원문이 다릅니다", (original_seed.span.start, original_seed.span.end),
    );
    let linked_seed = linked_seeds.next().ok_or_else(&mismatch)?;
    if linked_seeds.next().is_some()
        || !public_consumer_seed_preserved(&preparation, public_file, original_seed, linked_seed)
    {
        return Err(mismatch());
    }
    let linked_span = linked_seed.span;
    ddonirang_lang::canonicalizer::bind_linked_public_callable_seed(
        &mut program, linked_span, public.compiler_context(),
    ).map_err(|error| linked_multilang_parse_error(&preparation, error))?;
    Ok(LinkedMultilangProgram { preparation, program, logic_context })
}

/// Only the existing linker may relocate a local callable. Compare the
/// executable result expression at its exact mapped range before rebinding
/// the external call, even when the local name/result slot was relocated.
fn public_consumer_seed_preserved(
    preparation: &AuthoredPreparation,
    file: &crate::authored_project::SourceFile,
    original: &ddonirang_lang::SeedDef,
    linked: &ddonirang_lang::SeedDef,
) -> bool {
    let same_bytes = |a: Span, b: Span| {
        preparation.original_span(b).is_ok_and(|(path, span)| path == file.path && span == a)
            && file.text.get(a.start..a.end).is_some_and(|raw|
                preparation.source.get(b.start..b.end) == Some(raw))
    };
    if same_bytes(original.span, linked.span) {
        return true;
    }

    fn result(seed: &ddonirang_lang::SeedDef) -> Option<(&ddonirang_lang::ast::Expr, Option<Span>)> {
        if seed.seed_kind != ddonirang_lang::SeedKind::Semssi || !seed.params.is_empty()
            || !seed.type_params.is_empty() || !seed.alternate_forms.is_empty()
            || !seed.modifiers.is_empty() || seed.proof_header.is_some()
        {
            return None;
        }
        match seed.body.as_ref()?.stmts.as_slice() {
            [Stmt::Return { value, .. }] => Some((value, None)),
            [Stmt::Mutate { target, value, deferred: false, .. }]
                if matches!(&target.kind, ddonirang_lang::ast::ExprKind::Var(name)
                    if name == &seed.canonical_name) => Some((value, Some(target.span))),
            _ => None,
        }
    }

    let (Some((before, before_target)), Some((after, after_target))) = (result(original), result(linked))
    else { return false; };
    if linked.canonical_name == original.canonical_name || original.result_type != linked.result_type
        || !same_bytes(before.span, after.span)
    {
        return false;
    }
    match (before_target, after_target) {
        (None, None) => true,
        (Some(a), Some(b)) => preparation.original_span(b)
            .is_ok_and(|(path, span)| path == file.path && span == a),
        _ => false,
    }
}

fn linked_multilang_parse_error(
    preparation: &AuthoredPreparation, error: ddonirang_lang::ParseError,
) -> LocalProjectError {
    let code = match error.message.as_str() {
        "E_MULTILANG_LINKED_DEFINITION_COLLISION" => "E_MULTILANG_LINKED_DEFINITION_COLLISION",
        "E_MULTILANG_LINKED_SOURCE_SITE_MISMATCH" => "E_MULTILANG_LINKED_SOURCE_SITE_MISMATCH",
        _ => error.j1_source_code(),
    };
    let mut diagnostic = match preparation.original_span(error.span) {
        Ok((path, span)) if span.start < span.end => {
            LocalProjectError::with_path_span(code, path, error.message, (span.start, span.end))
        }
        _ => LocalProjectError::new(code, error.message),
    };
    diagnostic.phase = Some("project_link_parse");
    diagnostic
}

fn external_provider_directives(prefix: &ddonirang_lang::ModuleDirectiveDocument) -> ProjectModuleDirectives {
    // Verified providers do not become local graph nodes. Only real local
    // declarations from the checked prefix enter the public symbol table.
    let exports = prefix.exports.iter().flat_map(|directive| &directive.entries)
        .map(|entry| ParsedExport {
            external_name: entry.local_name.clone(),
            internal_name: entry.target.clone(),
        }).collect();
    ProjectModuleDirectives { imports: Vec::new(), exports, typed_current: true }
}

fn build_linked_project_selected(source:VirtualProjectSource,target_path_override:Option<&str>,run_for_symbol_values:bool,project_asset_registry:Option<Arc<ProjectAssetRegistry>>,j1:Option<ddonirang_lang::ddl::CheckedJ1Profile>,
    logic: Option<(&str, &ddonirang_lang::CanonProgram)>,
    public: Option<(&str, &ddonirang_lang::ModuleDirectiveDocument)>,
)->Result<LinkedProject,LocalProjectError>{
    let graph = build_project_graph_selected(source.clone(),j1)?;
    if let Some((path, _)) = public {
        if !graph.module_paths.values().any(|head| head == path) {
            return Err(LocalProjectError::with_path("E_DDL_CONSUMER_SOURCE_OWNER_REQUIRED", path,
                "사용자 DDL은 선언된 모듈의 머리 원문에 결속해야 합니다"));
        }
    }
    if let Some((path, _)) = logic {
        if graph.module_paths.contains_key("logic") {
            return Err(LocalProjectError::new("E_LOGIC_PROVIDER_COLLISION", "같은 이름의 사용자 모듈과 builtin을 추측 선택하지 않습니다"));
        }
        if !graph.module_paths.values().any(|head| head == path) {
            return Err(LocalProjectError::with_path("E_LOGIC_SOURCE_MODULE_REQUIRED", path, "B1은 선언된 모듈의 머리 원문에 결속해야 합니다"));
        }
    }
    if target_path_override.is_some_and(|path| path != graph.entry_path) {
        return Err(LocalProjectError::new("E_PROJECT_START_OVERRIDE", "호출자가 얽음의 시작을 덮을 수 없습니다"));
    }
    let texts = source.files.iter().map(|file| {
        Ok((normalize_project_path(&file.path)?, file.utf8_text.clone()))
    }).collect::<Result<BTreeMap<_, _>, LocalProjectError>>()?;
    let mut module_definitions = BTreeMap::<String,BTreeSet<String>>::new();
    let mut programs = BTreeMap::new();
    let mut directives = BTreeMap::new();
    for (module, paths) in &graph.module_sources {
        let module_schema_headers=module_schema_headers(&source,&graph,module,paths,j1)?;
        let mut names = BTreeSet::new();
        for path in paths {
            let masked = if j1.is_some(){texts[path].clone()}else{mask_module_metadata_blocks(&texts[path])?};
            if masked.trim().is_empty() { continue; }
            if ddonirang_lang::has_model_surface(&masked) {
                let model = ddonirang_lang::parse_model_program_with_source_file(&masked, path)
                    .map_err(|error| LocalProjectError::with_path_span(error.candidate_code, path,
                        error.message, (error.span.start, error.span.end)))?;
                names.extend(model.ir.definitions.into_iter().map(|definition| definition.name));
                continue;
            }
            if is_model_recipe_dependency(&source, &graph, module)? {
                let recipes = ddonirang_lang::parse_model_recipe_library_with_source_file(&masked, path)
                    .map_err(|error| LocalProjectError::with_path_span(error.candidate_code, path,
                        error.message, (error.span.start, error.span.end)))?;
                names.extend(recipes.into_iter().map(|recipe| recipe.name));
                continue;
            }
            let program = match logic {
                Some((owner, checked)) if path == owner => checked.clone(),
                _ => parse_unlinked_work_module_with_headers(&masked,path,j1,&module_schema_headers)
                    .map_err(|error| {
                        let mut failure = LocalProjectError::with_path_span(
                            if error.code() == "E_PARSE" { "E_PROJECT_MODULE_PARSE" } else { error.code() },
                            path, &error.message, (error.span.start, error.span.end),
                        );
                        if j1.is_some() && matches!(error.j1_source_code(), "E_J1_ROLE_AMBIGUOUS" | "E_FIXED64_LITERAL_RANGE") {
                            // Keep the legacy project error and carry the actual J1 parser failure.
                            let mut cause = failure.clone().into_execution_diagnostic();
                            cause.code = error.j1_source_code();
                            cause.phase = "lang_parse";
                            failure.cause = Some(Box::new(cause));
                        }
                        failure
                    })?,
            };
            for item in &program.items {
                let TopLevelItem::SeedDef(seed) = item;
                let setup=seed.body.as_ref().is_some_and(|body|seed.span==body.span&&body.stmts.iter().all(|s|matches!(s,Stmt::DeclBlock{..})));
                if setup {continue;}
                if !names.insert(seed.canonical_name.clone()) {
                    return Err(LocalProjectError::with_path("E_PROJECT_DEFINITION_DUPLICATE", path,
                        format!("모듈 {module}의 정의가 중복됩니다: {}", seed.canonical_name)));
                }
            }
            for statement in &program.root_stmts {
                // RD-346~349: a sum schema is a Definition, not an executable
                // root statement. Keep it in the same module namespace checks.
                if let Stmt::Definition { name, .. } | Stmt::SumDefinition { name, .. } = statement {
                    if !names.insert(name.clone()) {
                        return Err(LocalProjectError::with_path("E_PROJECT_DEFINITION_DUPLICATE", path, name));
                    }
                } else if let Stmt::DerivedBinding {name,..}=statement {
                    // A world relation owns a destination, not a callable or
                    // exported DefinitionIdentity. It may refine a typed slot.
                    names.insert(name.clone());
                } else if let Stmt::DeclBlock { items, .. } = statement {
                    names.extend(items.iter().map(|item| item.name.clone()));
                } else if matches!(statement, Stmt::PublicObservation { .. })
                    && graph.module_paths[module] == graph.entry_path {
                    // The existing read-only observation declaration belongs
                    // to this executable module. It is evaluated by the runner
                    // after settlement, not as an extra module-start action.
                    // Imported/nested observation ownership is not inferred.
                } else {
                    return Err(LocalProjectError::with_path("E_PROJECT_TOP_LEVEL_EXECUTION", path,
                        "정식 모듈의 실행 문장은 시작에 지정한 움직씨 본문에 작성해 주세요"));
                }
            }
            programs.insert(path.clone(), program);
        }
        module_definitions.insert(module.clone(),names);
        let head = &graph.module_paths[module];
        let module_directives = match (public,logic,j1) {
            (Some((owner,prefix)),_,_) if head == owner => external_provider_directives(prefix),
            (_,Some((owner,_)),Some(syntax)) if head == owner => {
                let prefix = ddonirang_lang::Parser::logic_callable_module_prefix(&texts[head], head, syntax)
                    .map_err(|error| LocalProjectError::with_path_span(error.j1_source_code(), head, error.message, (error.span.start, error.span.end)))?;
                external_provider_directives(&prefix)
            }
            _ => resolve_work_project_module_directives_selected(&source,head,j1)?,
        };
        directives.insert(module.clone(), module_directives);
    }
    // Actual per-module declaration owners remain distinct. Only ambiguous
    // flat-runtime spellings need compiler-private link addresses.
    let mut forms_by_module=BTreeMap::<String,BTreeMap<String,String>>::new();
    let mut ambiguous_form_modules=BTreeSet::new();
    let mut facts_by_path=BTreeMap::new();
    for (module,paths) in &graph.module_sources {
        let mut forms=BTreeMap::new();
        for program in paths.iter().filter_map(|path|programs.get(path)) {for item in &program.items {
            let TopLevelItem::SeedDef(seed)=item;
            for surface in seed.callable_surface_names().into_iter().chain(seed.rd714_first_call_forms()) {
                if let Some(prior)=forms.insert(surface.clone(),seed.canonical_name.clone()) {if prior!=seed.canonical_name {
                    ambiguous_form_modules.insert(module.clone());
                }}
            }
        }}
        forms_by_module.insert(module.clone(),forms);
    }
    for (module,paths) in &graph.module_sources {for path in paths {
        if let Some(program)=programs.get(path) {
            let facts=crate::project_authoring::module_link_facts(program,&texts[path],path,&forms_by_module[module],j1)?;
            module_definitions.get_mut(module).unwrap().extend(facts.globals.iter().cloned());facts_by_path.insert(path.clone(),facts);
        }
    }}
    // A distinct canonical name can still generate an occupied call form.
    // Index the actual owner of each finite spelling before flattening modules;
    // same-module ambiguity remains with the existing parser/owner guard.
    let mut owners=BTreeMap::<String,BTreeSet<(String,String)>>::new();
    for (module,names) in &module_definitions {for name in names {
        owners.entry(name.clone()).or_default().insert((module.clone(),name.clone()));
    }}
    for (module,forms) in &forms_by_module {for (surface,canonical) in forms {
        owners.entry(surface.clone()).or_default().insert((module.clone(),canonical.clone()));
    }}
    let mut colliding=owners.values().filter(|owners|
        owners.iter().map(|(module,_)|module).collect::<BTreeSet<_>>().len()>1)
        .flat_map(|owners|owners.iter().map(|(_,canonical)|canonical.clone())).collect::<BTreeSet<_>>();
    // Every type in a real multi-module source has its own namespace, even
    // when its spelling is unique. Otherwise an unqualified foreign private
    // type could become visible solely because the files were flattened.
    // Single-module raw source keeps its existing temporary execution scope.
    if graph.module_sources.len()>1{
        for program in programs.values(){for TopLevelItem::SeedDef(seed)in &program.items{
            if matches!(seed.seed_kind,ddonirang_lang::SeedKind::Schema(_)){colliding.insert(seed.canonical_name.clone());}
        }}
    }
    // Original AST declarations, retained before private executable names are
    // assigned. The registration producer consumes these exact source owners;
    // private names never become canonical labels or physical type identities.
    let original_declarations=programs.iter().flat_map(|(path,program)|program.items.iter().map(move |item|{
        let TopLevelItem::SeedDef(seed)=item;(path.clone(),seed.clone())
    })).collect::<Vec<_>>();
    let module_types=graph.module_sources.iter().map(|(module,paths)|{
        let names=paths.iter().filter_map(|path|programs.get(path)).flat_map(|program|program.items.iter()).filter_map(|item|{
            let TopLevelItem::SeedDef(seed)=item;
            matches!(seed.seed_kind,ddonirang_lang::SeedKind::Schema(_)).then_some(seed.canonical_name.clone())
        }).collect::<BTreeSet<_>>();(module.clone(),names)
    }).collect::<BTreeMap<_,_>>();
    let requires_registered_nominal_owner=module_types.values().any(|names|names.iter().any(|name|colliding.contains(name)));
    for(module,paths)in &graph.module_sources{for path in paths{if let Some(facts)=facts_by_path.get(path){
        if facts.unmapped_type_names.iter().any(|name|colliding.contains(name)&&module_types[module].contains(name)){
            return Err(LocalProjectError::with_path("E_PROJECT_NOMINAL_TYPE_REFERENCE_UNSUPPORTED",path,
                "해당 타입 참조의 정확한 문법 범위와 소유를 확인하지 못했습니다"));
        }
    }}}
    // Compiler addresses must remain one identifier in the selected lexer.
    // J1 original Japanese words have their own closed token admission; do
    // not broaden it merely to append an ASCII namespace prefix.
    let private_name=|prefix:&str,surface:&str| {
        if j1.is_none(){return format!("{prefix}{surface}");}
        let mut encoded=String::from(prefix);
        for ch in surface.chars(){
            if ch.is_ascii()||matches!(ch,'가'..='힣'|'ㄱ'..='ㅎ'|'ㅏ'..='ㅣ'){encoded.push(ch);}
            else {encoded.push_str(&format!("u{:x}_",ch as u32));}
        }
        encoded
    };
    let mut prefixes=BTreeMap::<String,String>::new();
    if !colliding.is_empty() {
        let mut tokens_in_source=BTreeSet::new();
        for text in texts.values() {
            let mut lexer=match j1 {Some(profile)=>Lexer::with_j1_profile(text,profile),None=>Ok(Lexer::new(text))}
                .map_err(|e|LocalProjectError::new("E_PROJECT_NAMESPACE_LEX",e.message))?;
            for token in lexer.tokenize().map_err(|e|LocalProjectError::new("E_PROJECT_NAMESPACE_LEX",e.message))? {if let Some(name)=token_name(&token){tokens_in_source.insert(name.to_string());}}}
        for (module,names) in &module_definitions {
            if !names.iter().any(|n|colliding.contains(n)){continue;}
            if ambiguous_form_modules.contains(module){return Err(LocalProjectError::new("E_PROJECT_NAMESPACE_CALL_COLLISION","모듈 호출 꼴의 정의 소유가 모호합니다"));}
            // This is a private executable address, not ModuleLogicalIdentity,
            // generated source semantics, a registration ID or a public alias.
            let prefix=format!("__ddn_mod_{}_",&sha256_hex(module.as_bytes())[..24]);
            for name in names.iter().filter(|n|colliding.contains(*n)) {
                let forms=&forms_by_module[module];
                if std::iter::once(name).chain(forms.iter().filter_map(|(surface,canonical)|(canonical==name).then_some(surface)))
                    .any(|surface|tokens_in_source.contains(&private_name(&prefix,surface))){return Err(LocalProjectError::new("E_PROJECT_NAMESPACE_INTERNAL_COLLISION","내부 실행 주소와 작성 이름이 충돌합니다"));}
                let declaration=graph.module_sources[module].iter().filter_map(|p|facts_by_path.get(p)).any(|facts|facts.globals.contains(name));
                if !declaration{return Err(LocalProjectError::new("E_PROJECT_NAMESPACE_LOWERING_UNSUPPORTED","해당 선언 종류의 실제 바인딩 범위가 아직 공급되지 않았습니다"));}
                for p in &graph.module_sources[module] {if let Some(program)=programs.get(p){for item in &program.items {let TopLevelItem::SeedDef(seed)=item;
                    if seed.canonical_name==*name&&!matches!(seed.seed_kind,ddonirang_lang::SeedKind::Umjikssi|ddonirang_lang::SeedKind::Semssi|ddonirang_lang::SeedKind::ValueFunc|ddonirang_lang::SeedKind::Schema(_)){return Err(LocalProjectError::new("E_PROJECT_NAMESPACE_LOWERING_UNSUPPORTED","명목·구조 씨의 동명 타입 결속은 기존 typed 소유 공급을 유지해야 합니다"));}
                }}}
            }
            // Escape-looking authored names cannot collide with another
            // definition's private address. Same-owner finite forms coalesce.
            let mut address_owners=BTreeMap::new();
            for name in names.iter().filter(|n|colliding.contains(*n)) {
                for surface in std::iter::once(name).chain(forms_by_module[module].iter().filter_map(|(s,c)|(c==name).then_some(s))){
                    if let Some(prior)=address_owners.insert(private_name(&prefix,surface),name){
                        if prior!=name{return Err(LocalProjectError::new("E_PROJECT_NAMESPACE_INTERNAL_COLLISION","서로 다른 정의의 내부 실행 주소가 충돌합니다"));}
                    }
                }
            }
            prefixes.insert(module.clone(),prefix);
        }
    }
    let linked_name=|module:&str,name:&str| if colliding.contains(name) {private_name(prefixes.get(module).map(String::as_str).unwrap_or(""),name)}else{name.into()};
    fn visit_module(name: &str, directives: &BTreeMap<String, ProjectModuleDirectives>,
        active: &mut BTreeSet<String>, done: &mut BTreeSet<String>, order: &mut Vec<String>) -> Result<(), LocalProjectError> {
        if done.contains(name) { return Ok(()); }
        if !active.insert(name.to_string()) {
            return Err(LocalProjectError::new("E_IMPORT_CYCLE", format!("모듈 순환입니다: {name}")));
        }
        let module = directives.get(name).ok_or_else(|| LocalProjectError::new("E_MODULE_MISSING", name))?;
        for import in &module.imports { visit_module(&import.source, directives, active, done, order)?; }
        active.remove(name);
        done.insert(name.to_string());
        order.push(name.to_string());
        Ok(())
    }
    let mut module_order = Vec::new();
    let mut done = BTreeSet::new();
    for module in graph.module_sources.keys() {
        visit_module(module, &directives, &mut BTreeSet::new(), &mut done, &mut module_order)?;
    }
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut resolved_symbols = Vec::new();
    let mut source_text = if j1.is_some(){"#!日本語\n".to_string()}else{String::new()};
    let mut source_map = Vec::new();
    for module in &module_order {
        let paths = &graph.module_sources[module];
        let head = &graph.module_paths[module];
        let mut rewrites = BTreeMap::new();
        for import in &directives[module].imports {
            let imported_head = graph.module_paths.get(&import.source).ok_or_else(||
                LocalProjectError::with_path("E_PROJECT_IMPORT_MODULE_UNKNOWN", head, &import.source))?;
            let imported = &directives[&import.source];
            for export in &imported.exports {
                rewrites.insert(format!("{}.{}", import.alias, export.external_name), linked_name(&import.source,&export.internal_name));
            }
            imports.push(ResolvedImport { importer_path: head.clone(), alias: import.alias.clone(),
                source: import.source.clone(), resolved_path: imported_head.clone() });
        }
        for export in &directives[module].exports {
            exports.push(ResolvedExport { module_path: head.clone(), external_name: export.external_name.clone(),
                internal_name: export.internal_name.clone() });
        }
        for path in paths {
            for request in collect_alias_symbol_requests_selected(&texts[path],j1)? {
                if let Some(import) = directives[module].imports.iter().find(|row| row.alias == request.alias) {
                    let known_exports = directives[&import.source].exports.iter()
                        .map(|row| row.external_name.clone()).collect::<HashSet<_>>();
                    let (span_start, span_end) = request.source_span.unwrap_or((0, request.symbol.len()));
                    let (external_stem, canonical_call) = resolve_known_call_target(
                        &request.symbol, &known_exports, Span::new(span_start, span_end))
                        .unwrap_or_else(|_| (request.symbol.clone(), request.symbol.clone()));
                    let export = directives[&import.source].exports.iter()
                        .find(|row| row.external_name == external_stem)
                        .ok_or_else(|| LocalProjectError::with_path_span("E_SYMBOL_NOT_EXPORTED", path,
                            format!("공개하지 않은 이름입니다: {}.{}", request.alias, request.symbol),
                            request.source_span.unwrap_or((0, 0))))?;
                    let suffix = canonical_call.strip_prefix(&external_stem).unwrap_or_default();
                    rewrites.insert(format!("{}.{}", import.alias, request.symbol),
                        format!("{}{}", linked_name(&import.source,&export.internal_name), suffix));
                    resolved_symbols.push(ResolvedSymbol {
                        alias: request.alias, symbol: request.symbol,
                        module_path: graph.module_paths[&import.source].clone(),
                        internal_name: export.internal_name.clone(), value: String::new(),
                    });
                }
            }
            let mut mapped=match j1{Some(profile)=>source_resolution::strip_j1_module_metadata_mapped(&texts[path],profile)?,None=>strip_module_metadata_blocks_mapped(&texts[path])?};
            if let Some(prefix)=prefixes.get(module) {
                let facts=facts_by_path.get(path).ok_or_else(||LocalProjectError::new("E_PROJECT_NAMESPACE_BINDINGS_REQUIRED","원문 바인딩이 없습니다"))?;
                if facts.unsupported{return Err(LocalProjectError::with_path("E_PROJECT_NAMESPACE_LOWERING_UNSUPPORTED",path,"내부 원문을 가진 식/분기 바인더의 결속은 추측하지 않습니다"));}
                let mut edits=Vec::new();
                for occurrence in facts.occurrences.iter().filter(|o|colliding.contains(&o.name)&&module_definitions[module].contains(&o.name)) {
                    let start=mapped.source_boundaries.binary_search(&occurrence.start).map_err(|_|LocalProjectError::new("E_PROJECT_NAMESPACE_SPAN","시작 범위가 다릅니다"))?;
                    let end=mapped.source_boundaries.binary_search(&occurrence.end).map_err(|_|LocalProjectError::new("E_PROJECT_NAMESPACE_SPAN","끝 범위가 다릅니다"))?;
                    let value=private_name(prefix,&occurrence.surface);
                    edits.push((start,end,occurrence.label.as_ref().map_or(value.clone(),|label|format!("{label}:{value}"))));
                }
                // Type references use their separate grammar/AST sites and
                // actual module schema namespace. A value binder or field
                // spelling must never stand in for a type Definition owner.
                for(name,span)in facts.type_occurrences.iter().filter(|(name,_)|colliding.contains(name)&&module_types[module].contains(name)){
                    let start=mapped.source_boundaries.binary_search(&span.start).map_err(|_|LocalProjectError::new("E_PROJECT_NAMESPACE_SPAN","타입 시작 범위가 다릅니다"))?;
                    let end=mapped.source_boundaries.binary_search(&span.end).map_err(|_|LocalProjectError::new("E_PROJECT_NAMESPACE_SPAN","타입 끝 범위가 다릅니다"))?;
                    edits.push((start,end,private_name(prefix,name)));
                }
                edits.sort();edits.dedup();mapped=source_resolution::rewrite_mapped_source_ranges(mapped,edits)?;
            }
            let rewritten = source_resolution::rewrite_alias_refs_in_mapped_source_selected(mapped,&rewrites,j1)?;
            let start = source_text.len();
            source_text.push_str(&rewritten.text);
            source_map.push(LinkedSourceSegment { generated_start: start, generated_end: source_text.len(),
                source_path: path.clone(), source_boundaries: rewritten.source_boundaries });
            source_text.push('\n');
        }
    }
    let mut resolved_start_definition = graph.start_definition.clone();
    if let Some(start_name) = &graph.start_definition {
        // Root names retain authored bytes. Reuse the lexical normalization
        // of the actual module definition; never add generated call aliases.
        let dialect = ddonirang_lang::dialect::DialectConfig::from_source(&texts[&graph.entry_path]);
        let canonical_start = if j1.is_none() { dialect.canonicalize_keyword(start_name).unwrap_or(start_name) } else { start_name };
        let module = graph.module_paths.iter().find(|(_, head)| **head == graph.entry_path).map(|(name, _)| name).unwrap();
        let matches = graph.module_sources[module].iter().filter_map(|path| programs.get(path)).flat_map(|program| program.items.iter())
            .filter_map(|item| { let TopLevelItem::SeedDef(seed) = item; (seed.canonical_name == canonical_start).then_some(seed) })
            .collect::<Vec<_>>();
        if matches.len() != 1 || matches[0].seed_kind != ddonirang_lang::SeedKind::Umjikssi
            || !matches[0].params.is_empty() || matches[0].body.is_none() {
            return Err(LocalProjectError::new("E_PROJECT_START_DEFINITION", "시작은 같은 모듈의 인자 없는 움직씨 정의 하나를 가리켜야 합니다"));
        }
        resolved_start_definition = Some(linked_name(module,&matches[0].canonical_name));
    }
    if run_for_symbol_values && requires_registered_nominal_owner {
        return Err(LocalProjectError::new("E_PROJECT_REGISTERED_NOMINAL_OWNER_REQUIRED",
            "동명 모듈 이름씨는 원문과 실제 등록자료를 Native에 명시 보관한 뒤 실행해 주세요"));
    }
    let output_rows = if run_for_symbol_values && resolved_start_definition.is_some() {
        let target = resolved_start_definition.as_deref().ok_or_else(|| LocalProjectError::new(
            "E_PROJECT_START_REQUIRED", "시작 없는 라이브러리는 실행할 수 없습니다"))?;
        let result = runtime_surface::run_summary_from_supported_source_profile_with_update_name_diagnostic(
            &source_text, &graph.entry_path, Some(1), Some(match j1{Some(profile)=>profile.identity(),None=>project_source_profile(&source,None)?}), target, project_asset_registry)
            .map_err(|error| LocalProjectError::from_execution_diagnostic(remap_execution_diagnostic(&source_map, error)))?;
        json_string_array(result.get("output_log_texts"))
    } else { Vec::new() };
    let order = graph.module_sources.values().flatten().cloned().collect::<Vec<_>>();
    let hash = sha256_hex(serde_json::to_string(&json!({"modules": graph.module_sources,
        "imports": imports, "exports": exports, "source": source_text})).unwrap().as_bytes());
    Ok(LinkedProject { symbol_boundary: LocalProjectSymbolBoundary {
        schema: PROJECT_SYMBOL_BOUNDARY_SCHEMA, entry_path: graph.entry_path,
        module_graph_order: order, module_graph_sha256: hash.clone(), imports, exports,
        resolved_symbols, output_rows, symbol_boundary_sha256: hash,
        diagnostics: Vec::new(),
    }, execution_source: source_text, execution_source_map: source_map,
        start_definition:resolved_start_definition,original_declarations,requires_registered_nominal_owner })
}

pub(super) fn build_legacy_linked_project_with_options(
    source: VirtualProjectSource,
    target_path_override: Option<&str>,
    run_for_symbol_values: bool,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
) -> Result<LinkedProject, LocalProjectError> {
    let source_profile = project_source_profile(&source, None)?;
    let graph = build_project_graph(source.clone())?;
    let entry_path = target_path_override
        .unwrap_or(&graph.entry_path)
        .to_string();
    let mut text_by_path = BTreeMap::new();
    for file in source.files {
        let path = normalize_project_path(&file.path)?;
        text_by_path.insert(path, file.utf8_text);
    }
    let entry_text = text_by_path.get(&entry_path).ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_ENTRY_MISSING",
            entry_path.clone(),
            "entry 파일이 없습니다",
        )
    })?;
    let mut modules = BTreeMap::new();
    collect_module_info(
        &entry_path,
        &entry_path,
        &text_by_path,
        &mut modules,
        &mut Vec::new(),
    )?;
    for path in modules.keys() {
        if let Some(text) = text_by_path.get(path) {
            validate_rd203_field_surface_before_link(text, path)?;
        }
    }

    let mut resolved_imports = Vec::new();
    let mut resolved_exports = Vec::new();
    for info in modules.values() {
        resolved_imports.extend(info.imports.iter().cloned());
        for (external_name, internal_name) in &info.exports {
            resolved_exports.push(ResolvedExport {
                module_path: info.path.clone(),
                external_name: external_name.clone(),
                internal_name: internal_name.clone(),
            });
        }
    }
    resolved_imports.sort_by(|a, b| {
        (&a.importer_path, &a.alias, &a.resolved_path).cmp(&(
            &b.importer_path,
            &b.alias,
            &b.resolved_path,
        ))
    });
    resolved_exports.sort_by(|a, b| {
        (&a.module_path, &a.external_name, &a.internal_name).cmp(&(
            &b.module_path,
            &b.external_name,
            &b.internal_name,
        ))
    });

    let mut show_requests = collect_alias_show_requests_from_ast(entry_text, &entry_path)?;
    let entry_info = modules.get(&entry_path).expect("entry info exists");
    for request in collect_alias_symbol_requests_from_tokens(entry_text)? {
        if entry_info
            .imports
            .iter()
            .any(|item| item.alias == request.alias)
        {
            if let Some(existing) = show_requests
                .iter_mut()
                .find(|item| item.alias == request.alias && item.symbol == request.symbol)
            {
                if existing.source_span.is_none() {
                    existing.source_span = request.source_span;
                }
            } else {
                show_requests.push(request);
            }
        }
    }
    let mut resolved_symbols = Vec::new();
    let mut entry_rewrites = BTreeMap::new();
    for request in show_requests {
        let import = entry_info
            .imports
            .iter()
            .find(|item| item.alias == request.alias)
            .ok_or_else(|| {
                LocalProjectError::with_path(
                    "E_ALIAS_UNKNOWN",
                    entry_path.clone(),
                    format!("알 수 없는 쓰임 alias입니다: {}", request.alias),
                )
            })?;
        let (resolved, rewritten_symbol) = resolve_imported_symbol_surface(
            &modules,
            &import.resolved_path,
            &request.symbol,
            request.source_span,
        )
        .map_err(|err| {
            if err.code == "E_SYMBOL_NOT_EXPORTED"
                && err.path.as_deref() == Some(import.resolved_path.as_str())
            {
                if let Some(span) = request.source_span {
                    return LocalProjectError::with_path_span(
                        err.code,
                        entry_path.clone(),
                        err.message,
                        span,
                    );
                }
            }
            err
        })?;
        entry_rewrites.insert(
            format!("{}.{}", request.alias, request.symbol),
            rewritten_symbol,
        );
        resolved_symbols.push(ResolvedSymbol {
            alias: request.alias,
            symbol: request.symbol,
            module_path: resolved.module_path,
            internal_name: resolved.internal_name,
            value: String::new(),
        });
    }

    let mut module_graph_order = modules.keys().cloned().collect::<Vec<_>>();
    module_graph_order.sort();
    let module_graph_sha256 = sha256_hex(
        serde_json::to_string(&json!({
            "schema": "ddn.local_project.module_graph.hash_input.v1",
            "order": module_graph_order.clone(),
            "imports": resolved_imports.clone(),
            "exports": resolved_exports.clone(),
        }))
        .expect("module graph hash input serialization")
        .as_bytes(),
    );

    let linked_execution_source = build_execution_source(
        &entry_path,
        &module_graph_order,
        &modules,
        &text_by_path,
        &entry_rewrites,
    )?;
    let output_rows = if run_for_symbol_values {
        let runtime = match project_asset_registry {
            Some(registry) => {
                runtime_surface::run_summary_from_supported_source_profile_with_project_assets_diagnostic(
                    &linked_execution_source.text,
                    &entry_path,
                    Some(1),
                    Some(source_profile),
                    registry,
                )
            }
            None => runtime_surface::run_summary_from_supported_source_profile_diagnostic(
                &linked_execution_source.text,
                &entry_path,
                Some(1),
                Some(source_profile),
            ),
        }
        .map_err(|err| {
            LocalProjectError::from_execution_diagnostic(remap_execution_diagnostic(
                &linked_execution_source.source_map,
                err,
            ))
        })?;
        let output_rows = json_string_array(runtime.get("output_log_texts"));
        let final_row = runtime
            .get("final_row")
            .and_then(JsonValue::as_object)
            .cloned()
            .unwrap_or_default();
        for symbol in &mut resolved_symbols {
            symbol.value = final_row
                .get(&symbol.internal_name)
                .map(json_scalar_to_output_text)
                .unwrap_or_default();
        }
        output_rows
    } else {
        Vec::new()
    };

    let hash_input = serde_json::to_string(&json!({
        "schema": "ddn.local_project_symbol_boundary.hash_input.v1",
        "entry_path": entry_path,
        "module_graph_order": module_graph_order.clone(),
        "module_graph_sha256": module_graph_sha256.clone(),
        "imports": resolved_imports.clone(),
        "exports": resolved_exports.clone(),
        "resolved_symbols": resolved_symbols.clone(),
        "output_rows": output_rows.clone(),
    }))
    .expect("symbol boundary hash input serialization");
    let symbol_boundary_sha256 = sha256_hex(hash_input.as_bytes());
    Ok(LinkedProject {
        symbol_boundary: LocalProjectSymbolBoundary {
            schema: PROJECT_SYMBOL_BOUNDARY_SCHEMA,
            entry_path,
            module_graph_order,
            module_graph_sha256,
            imports: resolved_imports,
            exports: resolved_exports,
            resolved_symbols,
            output_rows,
            symbol_boundary_sha256,
            diagnostics: Vec::new(),
        },
        execution_source: linked_execution_source.text,
        execution_source_map: linked_execution_source.source_map,
        start_definition: graph.start_definition,
        original_declarations:Vec::new(),requires_registered_nominal_owner:false,
    })
}

pub(super) fn validate_rd203_field_surface_before_link(
    source: &str,
    path: &str,
) -> Result<(), LocalProjectError> {
    const WRAPPER_PREFIX: &str = "매틱:움직씨 := {\n";
    const WRAPPER_SUFFIX: &str = "}\n";
    let masked = mask_module_metadata_blocks(source)?;
    match ddonirang_lang::parse_v25_with_mode(&masked, path, ddonirang_lang::ParseMode::Strict) {
        Ok(_) => return Ok(()),
        Err(err) if is_rd203_field_diagnostic(err.code()) => {
            let mut diagnostic = LocalProjectError::with_path_span(
                err.code(),
                path,
                err.message,
                (err.span.start, err.span.end.min(source.len())),
            );
            diagnostic.phase = Some("lang_parse");
            return Err(diagnostic);
        }
        Err(_) => {}
    }

    let mut wrapped =
        String::with_capacity(WRAPPER_PREFIX.len() + masked.len() + WRAPPER_SUFFIX.len() + 1);
    wrapped.push_str(WRAPPER_PREFIX);
    wrapped.push_str(&masked);
    if !masked.ends_with('\n') {
        wrapped.push('\n');
    }
    wrapped.push_str(WRAPPER_SUFFIX);
    if let Err(err) =
        ddonirang_lang::parse_v25_with_mode(&wrapped, path, ddonirang_lang::ParseMode::Strict)
    {
        if is_rd203_field_diagnostic(err.code()) {
            let span = if err.span.start >= WRAPPER_PREFIX.len()
                && err.span.end >= WRAPPER_PREFIX.len()
                && err.span.end - WRAPPER_PREFIX.len() <= source.len()
            {
                (
                    err.span.start - WRAPPER_PREFIX.len(),
                    err.span.end - WRAPPER_PREFIX.len(),
                )
            } else {
                (0, 0)
            };
            let mut diagnostic =
                LocalProjectError::with_path_span(err.code(), path, err.message, span);
            diagnostic.phase = Some("lang_parse");
            return Err(diagnostic);
        }
    }
    Ok(())
}

pub(super) fn is_rd203_field_diagnostic(code: &str) -> bool {
    matches!(
        code,
        "E_FIELD_MISSING" | "E_FIELD_TARGET_TYPE" | "E_FIELD_POSSESSIVE_SURFACE_FORBIDDEN"
    )
}

