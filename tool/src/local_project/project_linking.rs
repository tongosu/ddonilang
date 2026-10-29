//! 작품 모듈을 연결하는 기존 처리. 구조체 정체와 오류 처리는 부모와 동일하다.
// F5: 이 linker가 소유한 graph의 module_paths/module_sources는 함께 삽입한다.
// closure 경로는 같은 정규 원문 map에 존재하고 directives도 같은 모듈 키로 만든다.
// DFS는 없는 import/순환을 거부한 뒤 순서를 확정하므로 후속 키 접근은 그 집합 안이다.
// 시작 모듈은 checked get으로 확인하며 matches[0]은 len()!=1 단락 평가 뒤에만 읽는다.
// 원래 raw source.files에서 머리를 찾는 별도 조회는 이 map 근거로 보장되지 않는다.
// 보존한 legacy linker는 현재 호출이 없는 후보이며 새 기능/입구로 수용하지 않는다.
use super::*;

pub(super) fn build_linked_project(
    source: VirtualProjectSource,
    target_path_override: Option<&str>,
) -> Result<LinkedProject, LocalProjectError> {
    let project_asset_registry = project_asset_registry_from_source(&source)?;
    build_linked_project_with_options(source, target_path_override, true, project_asset_registry)
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
        imported_definitions:BTreeSet::new()})
}

pub(super) fn build_linked_project_with_options(
    source: VirtualProjectSource,
    target_path_override: Option<&str>,
    run_for_symbol_values: bool,
    project_asset_registry: Option<Arc<ProjectAssetRegistry>>,
) -> Result<LinkedProject, LocalProjectError> {
    let graph = build_project_graph(source.clone())?;
    if target_path_override.is_some_and(|path| path != graph.entry_path) {
        return Err(LocalProjectError::new("E_PROJECT_START_OVERRIDE", "호출자가 얽음의 시작을 덮을 수 없습니다"));
    }
    let texts = source.files.iter().map(|file| {
        Ok((normalize_project_path(&file.path)?, file.utf8_text.clone()))
    }).collect::<Result<BTreeMap<_, _>, LocalProjectError>>()?;
    let mut definitions = BTreeMap::<String, String>::new();
    let mut programs = BTreeMap::new();
    let mut directives = BTreeMap::new();
    for (module, paths) in &graph.module_sources {
        let mut names = BTreeSet::new();
        for path in paths {
            let masked = mask_module_metadata_blocks(&texts[path])?;
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
            let program = parse_unlinked_work_module(&masked, path)
                .map_err(|error| LocalProjectError::with_path_span(
                    if error.code() == "E_PARSE" { "E_PROJECT_MODULE_PARSE" } else { error.code() }, path,
                    error.message, (error.span.start, error.span.end)))?;
            for item in &program.items {
                let TopLevelItem::SeedDef(seed) = item;
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
                } else {
                    return Err(LocalProjectError::with_path("E_PROJECT_TOP_LEVEL_EXECUTION", path,
                        "정식 모듈의 실행 문장은 시작에 지정한 움직씨 본문에 작성해 주세요"));
                }
            }
            programs.insert(path.clone(), program);
        }
        for name in names {
            if let Some(owner) = definitions.insert(name.clone(), module.clone()) {
                return Err(LocalProjectError::new("E_PROJECT_NAMESPACE_LOWERING_UNSUPPORTED",
                    format!("서로 다른 모듈의 동명 정의는 아직 실행 결속되지 않았습니다: {owner}, {module}, {name}")));
            }
        }
        directives.insert(module.clone(), resolve_work_project_module_directives(&source, &graph.module_paths[module])?);
    }
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
    let mut source_text = String::new();
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
                rewrites.insert(format!("{}.{}", import.alias, export.external_name), export.internal_name.clone());
            }
            imports.push(ResolvedImport { importer_path: head.clone(), alias: import.alias.clone(),
                source: import.source.clone(), resolved_path: imported_head.clone() });
        }
        for export in &directives[module].exports {
            exports.push(ResolvedExport { module_path: head.clone(), external_name: export.external_name.clone(),
                internal_name: export.internal_name.clone() });
        }
        for path in paths {
            for request in collect_alias_symbol_requests_from_tokens(&texts[path])? {
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
                        format!("{}{}", export.internal_name, suffix));
                    resolved_symbols.push(ResolvedSymbol {
                        alias: request.alias, symbol: request.symbol,
                        module_path: graph.module_paths[&import.source].clone(),
                        internal_name: export.internal_name.clone(), value: String::new(),
                    });
                }
            }
            let mapped = strip_module_metadata_blocks_mapped(&texts[path])?;
            let rewritten = rewrite_alias_refs_in_mapped_source(mapped, &rewrites)?;
            let start = source_text.len();
            source_text.push_str(&rewritten.text);
            source_map.push(LinkedSourceSegment { generated_start: start, generated_end: source_text.len(),
                source_path: path.clone(), source_boundaries: rewritten.source_boundaries });
            source_text.push('\n');
        }
    }
    if let Some(start_name) = &graph.start_definition {
        let module = graph.module_paths.iter().find(|(_, head)| **head == graph.entry_path).map(|(name, _)| name).unwrap();
        let matches = graph.module_sources[module].iter().filter_map(|path| programs.get(path)).flat_map(|program| program.items.iter())
            .filter_map(|item| { let TopLevelItem::SeedDef(seed) = item; (seed.canonical_name == *start_name).then_some(seed) })
            .collect::<Vec<_>>();
        if matches.len() != 1 || matches[0].seed_kind != ddonirang_lang::SeedKind::Umjikssi
            || !matches[0].params.is_empty() || matches[0].body.is_none() {
            return Err(LocalProjectError::new("E_PROJECT_START_DEFINITION", "시작은 같은 모듈의 인자 없는 움직씨 정의 하나를 가리켜야 합니다"));
        }
    }
    let output_rows = if run_for_symbol_values && graph.start_definition.is_some() {
        let target = graph.start_definition.as_deref().ok_or_else(|| LocalProjectError::new(
            "E_PROJECT_START_REQUIRED", "시작 없는 라이브러리는 실행할 수 없습니다"))?;
        let result = runtime_surface::run_summary_from_supported_source_profile_with_update_name_diagnostic(
            &source_text, &graph.entry_path, Some(1), Some(project_source_profile(&source, None)?), target, project_asset_registry)
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
        start_definition:graph.start_definition })
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

