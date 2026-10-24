//! 작품 원문 해소·실행 준비의 기존 처리. 정본·해시·공개 경로와 자료형 정체는 유지한다.
// F5: directive의 segment_start는 0 또는 앞선 Dot index+1이므로 다음 index 이하이다.
// mask의 bytes는 같은 원문 복사이고 길이를 바꾸지 않는다. span은 같은 lexer/parser가
// 공급한다. MappedSourceText는 비공개 생산자가 output byte 수+1개의 경계를 유지한다.
// rewrite는 start<cursor를 거부한다. removal 구간 중첩은 이 근거로 판정하지 않는다.
// 선택된 to_value 5곳의 자료형은 SealedDatasetAssetIdentity/EvidenceManifestIdentity
// 차림과 HostResourceTranscript/Receipt다. 전이 필드는 derive Serialize된 문자열·u64·
// Option·Vec·고정 enum/record로 닫혀 있으며 임의 Serialize나 외부 I/O writer가 없다.
// 일반 T:Serialize와 JSON 생성 매크로·할당/재귀의 자원 실패는 이 분류에서 제외한다.
use super::*;

pub(super) fn wrap_linked_execution_source_for_supported_project_run(linked: &mut LinkedProject) {
    // The supported V25 runtime dispatches the shared update callable named
    // `매마디`.  Keep the generated project wrapper on that same authority;
    // `매틱` is a historical producer spelling and would leave a valid V25
    // project with no runnable update callable.
    const PREFIX: &str = "매마디:움직씨 := {\n";
    const SUFFIX: &str = "}\n";
    let mut text =
        String::with_capacity(PREFIX.len() + linked.execution_source.len() + SUFFIX.len());
    text.push_str(PREFIX);
    text.push_str(&linked.execution_source);
    text.push_str(SUFFIX);
    for segment in &mut linked.execution_source_map {
        segment.generated_start += PREFIX.len();
        segment.generated_end += PREFIX.len();
    }
    linked.execution_source = text;
}

pub(super) fn build_execution_source(
    entry_path: &str,
    module_graph_order: &[String],
    modules: &BTreeMap<String, ModuleInfo>,
    text_by_path: &BTreeMap<String, String>,
    entry_rewrites: &BTreeMap<String, String>,
) -> Result<LinkedExecutionSource, LocalProjectError> {
    let mut out = String::new();
    let mut source_map = Vec::new();
    out.push_str("// linked by ddn.local_project.linker.v1\n");
    for path in module_graph_order {
        if path == entry_path {
            continue;
        }
        let text = text_by_path.get(path).ok_or_else(|| {
            LocalProjectError::with_path("E_MODULE_MISSING", path.clone(), "모듈 파일이 없습니다")
        })?;
        let module = modules.get(path).ok_or_else(|| {
            LocalProjectError::with_path(
                "E_MODULE_MISSING",
                path.clone(),
                "모듈 metadata가 없습니다",
            )
        })?;
        let rewrites = module_alias_rewrites(module, modules)?;
        let stripped = strip_module_metadata_blocks_mapped(text)?;
        let rewritten = rewrite_alias_refs_in_mapped_source(stripped, &rewrites)?;
        if !rewritten.text.trim().is_empty() {
            out.push_str("// module ");
            out.push_str(path);
            out.push('\n');
            let generated_start = out.len();
            out.push_str(&rewritten.text);
            source_map.push(LinkedSourceSegment {
                generated_start,
                generated_end: out.len(),
                source_path: path.clone(),
                source_boundaries: rewritten.source_boundaries,
            });
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
    }
    let entry_text = text_by_path.get(entry_path).ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_ENTRY_MISSING",
            entry_path,
            "entry 파일이 없습니다",
        )
    })?;
    let entry_body = strip_module_metadata_blocks_mapped(entry_text)?;
    let entry_rewritten = rewrite_alias_refs_in_mapped_source(entry_body, entry_rewrites)?;
    out.push_str("// entry ");
    out.push_str(entry_path);
    out.push('\n');
    let generated_start = out.len();
    out.push_str(&entry_rewritten.text);
    source_map.push(LinkedSourceSegment {
        generated_start,
        generated_end: out.len(),
        source_path: entry_path.to_string(),
        source_boundaries: entry_rewritten.source_boundaries,
    });
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(LinkedExecutionSource {
        text: out,
        source_map,
    })
}

pub(super) fn module_alias_rewrites(
    module: &ModuleInfo,
    modules: &BTreeMap<String, ModuleInfo>,
) -> Result<BTreeMap<String, String>, LocalProjectError> {
    let mut rewrites = BTreeMap::new();
    for import in &module.imports {
        let imported = modules.get(&import.resolved_path).ok_or_else(|| {
            LocalProjectError::with_path(
                "E_MODULE_MISSING",
                import.resolved_path.clone(),
                "imported module metadata가 없습니다",
            )
        })?;
        for external_name in imported.exports.keys() {
            let resolved = resolve_exported_symbol(
                modules,
                &import.resolved_path,
                external_name,
                &mut Vec::new(),
            )?;
            rewrites.insert(
                format!("{}.{}", import.alias, external_name),
                resolved.internal_name,
            );
        }
    }
    Ok(rewrites)
}

pub(super) fn strip_module_metadata_blocks_mapped(text: &str) -> Result<MappedSourceText, LocalProjectError> {
    let mut mapped = MappedSourceText {
        // Current parenthesized metadata is first blanked through the shared
        // typed parser.  Its byte-preserving mask keeps the source map exact;
        // historical brace custody continues through the existing removal
        // path below.
        text: mask_module_metadata_blocks(text)?,
        source_boundaries: (0..=text.len()).collect(),
    };
    for name in ["쓰임", "드러냄", "공개"] {
        mapped = remove_named_block_mapped(mapped, name)?;
    }
    Ok(mapped)
}

pub(super) fn remove_named_block_mapped(
    mapped: MappedSourceText,
    name: &str,
) -> Result<MappedSourceText, LocalProjectError> {
    let text = &mapped.text;
    let removals = legacy_named_brace_ranges(text, name)?;
    if removals.is_empty() {
        return Ok(mapped);
    }
    let mut out = String::with_capacity(text.len());
    let mut boundaries = vec![mapped.source_boundaries[0]];
    let mut kept_cursor = 0usize;
    for (start, end) in removals {
        out.push_str(&text[kept_cursor..start]);
        boundaries.extend_from_slice(&mapped.source_boundaries[(kept_cursor + 1)..=start]);
        *boundaries.last_mut().expect("mapped boundary") = mapped.source_boundaries[end];
        kept_cursor = end;
    }
    out.push_str(&text[kept_cursor..]);
    boundaries.extend_from_slice(&mapped.source_boundaries[(kept_cursor + 1)..=text.len()]);
    debug_assert_eq!(boundaries.len(), out.len() + 1);
    Ok(MappedSourceText {
        text: out,
        source_boundaries: boundaries,
    })
}

/// Historical brace metadata is recognized only from lexer tokens.  Raw text
/// search would incorrectly consume `쓰임 {` inside a string or comment and
/// could therefore turn non-semantic text into a project-link error.
pub(super) fn legacy_named_brace_ranges(
    text: &str,
    name: &str,
) -> Result<Vec<(usize, usize)>, LocalProjectError> {
    reject_source_bom(text, None)?;
    let tokens = Lexer::new(text).tokenize().map_err(|error| {
        LocalProjectError::new(
            "E_PROJECT_ENTRY_PARSE",
            format!("module metadata 토큰화 실패: {}", error.message),
        )
    })?;
    let mut ranges = Vec::new();
    for pair in tokens.windows(2) {
        let is_name = matches!(
            &pair[0].kind,
            TokenKind::Ident(value) | TokenKind::Josa(value) if value == name
        );
        if !is_name || !matches!(pair[1].kind, TokenKind::LBrace) {
            continue;
        }
        let end = find_block_end(text, pair[1].span.start).ok_or_else(|| {
            LocalProjectError::new(
                "E_PROJECT_METADATA_BLOCK",
                format!("{name} metadata 블록을 닫지 못했습니다"),
            )
        })?;
        ranges.push((pair[0].span.start, end));
    }
    Ok(ranges)
}

/// Module metadata is not parser input. Mask it without changing any original
/// byte offset so AST-backed authoring locations remain source-exact.
pub(crate) fn mask_module_metadata_blocks(text: &str) -> Result<String, LocalProjectError> {
    reject_source_bom(text, None)?;
    let mut bytes = text.as_bytes().to_vec();
    let tokens = Lexer::new(text).tokenize().map_err(|error| {
        LocalProjectError::new(
            "E_PROJECT_ENTRY_PARSE",
            format!("current module source 토큰화 실패: {}", error.message),
        )
    })?;
    if matches!(
        tokens.first().map(|token| &token.kind),
        Some(TokenKind::LParen)
    ) {
        let mut context = ModuleDirectiveContext::default();
        context.known_import_targets = tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::StringLit(value) | TokenKind::Ident(value) | TokenKind::Josa(value) => Some(value.clone()),
                _ => None,
            })
            .collect();
        context.public_definition_targets = tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Ident(value) | TokenKind::Josa(value) => Some(value.clone()),
                _ => None,
            })
            .collect();
        if let Ok(document) = parse_module_directive_prefix(text, "project.ddn", &context) {
            for directive in [document.imports, document.exports].into_iter().flatten() {
                for byte in &mut bytes[directive.span.start..directive.span.end] {
                    if !matches!(*byte, b'\n' | b'\r') {
                        *byte = b' ';
                    }
                }
            }
            return String::from_utf8(bytes).map_err(|_| {
                LocalProjectError::new(
                    "E_PROJECT_AUTHORING_MASK_UTF8",
                    "current directive offset 보존 mask가 UTF-8을 보존하지 못했습니다",
                )
            });
        }
    }
    for name in ["쓰임", "드러냄", "공개"] {
        for (start, end) in legacy_named_brace_ranges(text, name)? {
            for byte in &mut bytes[start..end] {
                if !matches!(*byte, b'\n' | b'\r') {
                    *byte = b' ';
                }
            }
        }
    }
    String::from_utf8(bytes).map_err(|_| {
        LocalProjectError::new(
            "E_PROJECT_AUTHORING_MASK_UTF8",
            "metadata offset 보존 mask가 UTF-8을 보존하지 못했습니다",
        )
    })
}

/// RD-534 forbids both a source BOM and stripping or masking it at ingress.
/// Keep the original bytes intact and report the exact three-byte prefix.
pub(super) fn reject_source_bom(text: &str, path: Option<&str>) -> Result<(), LocalProjectError> {
    if text.starts_with('\u{feff}') {
        let mut error = LocalProjectError::new(
            "E_SOURCE_UTF8_BOM_FORBIDDEN",
            "원문 앞의 UTF-8 BOM은 허용되지 않습니다. BOM 없는 UTF-8로 저장해 주세요.",
        );
        error.path = path.map(str::to_owned);
        error.source_span = Some((0, 3));
        error.phase = Some("source_ingress");
        return Err(error);
    }
    Ok(())
}

pub(super) fn find_block_end(text: &str, open_brace: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let mut end = open_brace;
    for (offset, ch) in text[open_brace..].char_indices() {
        let idx = open_brace + offset;
        end = idx + ch.len_utf8();
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                let mut tail = end;
                while let Some(next) = text[tail..].chars().next() {
                    if next.is_whitespace() {
                        tail += next.len_utf8();
                    } else {
                        break;
                    }
                }
                if text[tail..].starts_with('.') {
                    tail += 1;
                }
                if text[tail..].starts_with('\r') {
                    tail += 1;
                }
                if text[tail..].starts_with('\n') {
                    tail += 1;
                }
                return Some(tail);
            }
        }
    }
    Some(end).filter(|_| depth == 0)
}

pub(super) fn rewrite_alias_refs_in_mapped_source(
    mapped: MappedSourceText,
    rewrites: &BTreeMap<String, String>,
) -> Result<MappedSourceText, LocalProjectError> {
    let text = &mapped.text;
    if rewrites.is_empty() {
        return Ok(mapped);
    }
    let tokens = Lexer::new(text).tokenize().map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_LINKER_LEX",
            format!("프로젝트 alias rewrite 토큰화 실패: {}", err.message),
        )
    })?;
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    for window in tokens.windows(3) {
        let [left, dot, right] = window else {
            continue;
        };
        if !matches!(dot.kind, TokenKind::Dot) {
            continue;
        }
        let Some(alias) = token_name(left) else {
            continue;
        };
        let Some(symbol) = token_name(right) else {
            continue;
        };
        if !is_whitespace_only(text, left.span.end, dot.span.start)
            || !is_whitespace_only(text, dot.span.end, right.span.start)
        {
            continue;
        }
        let key = format!("{alias}.{symbol}");
        if let Some(to) = rewrites.get(&key) {
            replacements.push((left.span.start, right.span.end, to.clone()));
        } else {
            let prefix = format!("{alias}.");
            let known = rewrites.keys().filter_map(|name| name.strip_prefix(&prefix))
                .map(str::to_string).collect::<HashSet<_>>();
            if let Ok((stem, canonical_call)) = resolve_known_call_target(
                &symbol, &known, Span::new(right.span.start, right.span.end),
            ) {
                if let Some(to) = rewrites.get(&format!("{alias}.{stem}")) {
                    let suffix = canonical_call.strip_prefix(&stem).unwrap_or_default();
                    replacements.push((left.span.start, right.span.end, format!("{to}{suffix}")));
                }
            }
        }
    }
    if replacements.is_empty() {
        return Ok(mapped);
    }
    replacements.sort_by_key(|(start, _, _)| *start);
    let mut out = String::with_capacity(text.len());
    let mut source_boundaries = vec![mapped.source_boundaries[0]];
    let mut cursor = 0usize;
    for (start, end, to) in replacements {
        if start < cursor {
            return Err(LocalProjectError::new(
                "E_PROJECT_LINKER_REWRITE_OVERLAP",
                "프로젝트 alias rewrite 범위가 겹칩니다",
            ));
        }
        out.push_str(&text[cursor..start]);
        source_boundaries.extend_from_slice(&mapped.source_boundaries[(cursor + 1)..=start]);
        out.push_str(&to);
        if !to.is_empty() {
            let source_start = mapped.source_boundaries[start];
            let source_end = mapped.source_boundaries[end];
            source_boundaries.extend(std::iter::repeat_n(source_start, to.len() - 1));
            source_boundaries.push(source_end);
        }
        cursor = end;
    }
    out.push_str(&text[cursor..]);
    source_boundaries.extend_from_slice(&mapped.source_boundaries[(cursor + 1)..=text.len()]);
    debug_assert_eq!(source_boundaries.len(), out.len() + 1);
    Ok(MappedSourceText {
        text: out,
        source_boundaries,
    })
}

pub(super) fn token_name(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(name) | TokenKind::Josa(name) => Some(name.as_str()),
        _ => None,
    }
}

pub(super) fn is_whitespace_only(text: &str, start: usize, end: usize) -> bool {
    start <= end && text[start..end].chars().all(char::is_whitespace)
}

pub(super) fn is_identifier_char(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric() || ('가'..='힣').contains(&ch)
}

pub(super) fn json_string_array(value: Option<&JsonValue>) -> Vec<String> {
    value
        .and_then(JsonValue::as_array)
        .map(|items| items.iter().map(json_scalar_to_output_text).collect())
        .unwrap_or_default()
}

pub(super) fn json_scalar_to_output_text(value: &JsonValue) -> String {
    match value {
        JsonValue::String(text) => text.clone(),
        JsonValue::Number(number) => number.to_string(),
        JsonValue::Bool(value) => {
            if *value {
                "참".to_string()
            } else {
                "거짓".to_string()
            }
        }
        JsonValue::Null => "없음".to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

pub fn build_project_gaji_lock(
    source: VirtualProjectSource,
    require_existing_lock: bool,
) -> Result<LocalGajiLock, LocalProjectError> {
    let mut graph_source = source.clone();
    graph_source.files.retain(|file| {
        normalize_project_path(&file.path)
            .map(|path| path != "ddn.lock")
            .unwrap_or(true)
    });
    let graph = build_project_graph(graph_source)?;
    let mut text_by_path = BTreeMap::new();
    for file in source.files {
        let path = normalize_project_path(&file.path)?;
        text_by_path.insert(path, file.utf8_text);
    }

    let mut manifests = BTreeMap::new();
    for (alias, raw_path) in &graph.gaji_aliases {
        let path = normalize_project_reference(raw_path)?;
        let manifest = load_gaji_manifest_for_path(&path, &text_by_path)?;
        manifests.insert(manifest.id.clone(), manifest);
        let mut stack = vec![format!("project:{alias}")];
        collect_gaji_dependencies(&path, &text_by_path, &mut manifests, &mut stack)?;
    }

    let direct_ids = graph
        .gaji_aliases
        .values()
        .map(|path| normalize_project_reference(path))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|path| gaji_id_from_vendor_path(&path).map(|id| id.to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;

    let mut dependency_of = BTreeMap::<String, BTreeSet<String>>::new();
    for manifest in manifests.values() {
        for dep_id in manifest.dependencies.keys() {
            dependency_of
                .entry(dep_id.clone())
                .or_default()
                .insert(manifest.id.clone());
        }
    }

    let mut packages = Vec::new();
    for manifest in manifests.values() {
        packages.push(LocalGajiPin {
            id: manifest.id.clone(),
            version: manifest.version.clone(),
            path: gaji_vendor_path_for_id(&manifest.id),
            direct: direct_ids.contains(&manifest.id),
            dependency_of: dependency_of
                .remove(&manifest.id)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            dependencies: manifest.dependencies.clone(),
            manifest_path: manifest.manifest_path.clone(),
            manifest_sha256: manifest.manifest_sha256.clone(),
            package_sha256: manifest.package_sha256.clone(),
        });
    }
    packages.sort_by(|a, b| a.id.cmp(&b.id));

    let lock_hash_input = serde_json::to_string(&json!({
        "schema": "ddn.local_gaji_lock.hash_input.v1",
        "project_graph_sha256": graph.project_graph_sha256,
        "packages": packages,
    }))
    .expect("local gaji lock hash input serialization");
    let lock_sha256 = sha256_hex(lock_hash_input.as_bytes());
    let lock = LocalGajiLock {
        schema: LOCAL_GAJI_LOCK_SCHEMA,
        lock_format: "ddn.lock.v0-local",
        root_path: graph.root_path,
        entry_path: graph.entry_path,
        project_graph_sha256: graph.project_graph_sha256,
        packages,
        lock_sha256,
        diagnostics: Vec::new(),
    };

    let canonical = to_canonical_json_line(&lock);
    match text_by_path.get("ddn.lock") {
        Some(existing) => {
            if existing.trim_end() != canonical.trim_end() {
                return Err(LocalProjectError::with_path(
                    "E_LOCK_HASH_MISMATCH",
                    "ddn.lock",
                    "ddn.lock 내용이 현재 vendored 가지 해소 결과와 다릅니다",
                ));
            }
        }
        None if require_existing_lock => {
            return Err(LocalProjectError::with_path(
                "E_LOCK_MISSING",
                "ddn.lock",
                "frozen 검증에는 ddn.lock이 필요합니다",
            ));
        }
        None => {}
    }

    Ok(lock)
}

pub(crate) fn project_asset_registry_from_source(
    source: &VirtualProjectSource,
) -> Result<Option<Arc<ProjectAssetRegistry>>, LocalProjectError> {
    let Some(manifest) = source
        .files
        .iter()
        .find(|file| file.path == "ddn.asset.json")
    else {
        return Ok(None);
    };
    require_beta_project_feature("E_ALPHA_DATASET_UNSUPPORTED", "자료셋 연결은 알파에서 지원하지 않습니다. 베타에서 지원할 예정입니다")?;
    let files = source
        .files
        .iter()
        .map(|file| (file.path.clone(), file.utf8_text.as_bytes().to_vec()))
        .collect::<HashMap<_, _>>();
    build_project_asset_registry_from_manifest(manifest.utf8_text.as_bytes(), &files, true)
        .map(Arc::new)
        .map(Some)
        .map_err(|err| {
            LocalProjectError::with_path(
                "E_PROJECT_ASSET_REGISTRY",
                "ddn.asset.json",
                format!("project asset registry 검증 실패: {err}"),
            )
        })
}

pub(crate) fn project_asset_registry_from_json(
    input_json: &str,
) -> Result<Option<Arc<ProjectAssetRegistry>>, LocalProjectError> {
    let source: VirtualProjectSource = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_VIRTUAL_PROJECT_JSON_INVALID",
            format!("virtual project JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    project_asset_registry_from_source(&source)
}

pub fn build_project_prepare_link(
    source: VirtualProjectSource,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<LocalProjectPrepareLink, LocalProjectError> {
    let source_profile = project_source_profile(&source, None)?;
    let requested_target = target.trim();
    let project_asset_registry = project_asset_registry_from_source(&source)?;
    let domain_evidence_manifests = domain_evidence_manifest_identities_from_source(&source)?;
    let sealed_dataset_assets = project_asset_registry
        .as_deref()
        .map(ProjectAssetRegistry::sealed_dataset_identities)
        .transpose()
        .map_err(|err| {
            LocalProjectError::with_path(
                "E_PROJECT_DATASET_BINDING",
                "ddn.asset.json",
                format!("sealed dataset binding 검증 실패: {err}"),
            )
        })?
        .unwrap_or_default();
    let normalized_project = normalize_virtual_project_source(source.clone())?;
    let graph = build_project_graph(source.clone())?;
    let target = graph.start_definition.as_deref().ok_or_else(||
        LocalProjectError::new("E_PROJECT_START_REQUIRED", "시작 없는 라이브러리는 실행 준비할 수 없습니다"))?;
    if !requested_target.is_empty() && requested_target != target {
        return Err(LocalProjectError::new("E_PROJECT_START_OVERRIDE", "호출자가 얽음의 시작을 덮을 수 없습니다"));
    }
    let target_path = graph.targets.get(target).cloned().ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_TARGET_MISSING",
            graph.root_path.clone(),
            format!("프로젝트 목표 {target}을 찾을 수 없습니다"),
        )
    })?;
    let lock_required = !graph.gaji_aliases.is_empty();
    let verified_lock = if inspect_lock && lock_required {
        Some(build_project_gaji_lock(source.clone(), true)?)
    } else {
        None
    };
    let linked = build_linked_project_with_options(source, Some(&target_path), false, None)?;
    // The linker validated the authored start definition. Let the shared
    // execution frontdoor report call errors with their original source span.
    let linked_execution_source_sha256 = sha256_hex(linked.execution_source.as_bytes());
    let mut runtime_execution_envelope = runtime_surface::execution_envelope_from_source_with_profile(
        &linked.execution_source,
        &target_path,
        Some(1),
        Some(source_profile),
    )
    .map_err(|err| {
        LocalProjectError::from_execution_diagnostic(linked.remap_execution_diagnostic(err))
    })?;
    if let Some(envelope) = runtime_execution_envelope.as_object_mut() {
        envelope.insert(
            "sealed_dataset_assets".to_string(),
            serde_json::to_value(&sealed_dataset_assets)
                .expect("sealed dataset identities serialize"),
        );
        envelope.insert(
            "domain_evidence_manifests".to_string(),
            serde_json::to_value(&domain_evidence_manifests)
                .expect("domain evidence identities serialize"),
        );
    }
    let mut prepared = LocalProjectPrepareLink {
        source_profile_identity: source_profile,
        schema: "ddn.local_project.prepare_link.v1",
        frontdoor: "wasm_project_prepare_link_json",
        root_path: graph.root_path.clone(),
        entry_path: graph.entry_path.clone(),
        target: target.to_string(),
        target_path,
        file_count: graph.file_count,
        normalized_project,
        project_graph: graph,
        graph_inspected: inspect_graph,
        lock_required,
        lock_inspected: inspect_lock,
        verified_lock,
        linked_execution_source: linked.execution_source,
        linked_execution_source_sha256,
        runtime_execution_envelope,
        symbol_boundary: linked.symbol_boundary,
        sealed_dataset_assets,
        domain_evidence_manifests,
        diagnostics: Vec::new(),
        state_hash_formula_change: false,
        prepare_link_sha256: String::new(),
    };
    let hash_input = serde_json::to_string(&json!({
        "schema": "ddn.local_project.prepare_link.hash_input.v1",
        "source_profile_identity": prepared.source_profile_identity,
        "root_path": &prepared.root_path,
        "entry_path": &prepared.entry_path,
        "target": &prepared.target,
        "target_path": &prepared.target_path,
        "project_graph_sha256": &prepared.project_graph.project_graph_sha256,
        "lock_required": prepared.lock_required,
        "lock_sha256": prepared.verified_lock.as_ref().map(|lock| lock.lock_sha256.clone()),
        "linked_execution_source_sha256": &prepared.linked_execution_source_sha256,
        "runtime_execution_envelope": &prepared.runtime_execution_envelope,
        "symbol_boundary_sha256": &prepared.symbol_boundary.symbol_boundary_sha256,
        "sealed_dataset_assets": &prepared.sealed_dataset_assets,
        "domain_evidence_manifests": &prepared.domain_evidence_manifests,
        "state_hash_formula_change": prepared.state_hash_formula_change,
    }))
    .expect("local project prepare/link hash input serialization");
    prepared.prepare_link_sha256 = sha256_hex(hash_input.as_bytes());
    Ok(prepared)
}

pub fn build_project_run_summary(
    source: VirtualProjectSource,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<LocalProjectRunSummary, LocalProjectError> {
    build_project_run_summary_internal(source, target, inspect_graph, inspect_lock, false)
}

pub fn build_project_run_summary_with_supported_profile(
    source: VirtualProjectSource,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
) -> Result<LocalProjectRunSummary, LocalProjectError> {
    build_project_run_summary_internal(source, target, inspect_graph, inspect_lock, true)
}

pub fn build_project_run_summary_with_supported_profile_and_host_resources(
    source: VirtualProjectSource,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
    request: &HostResourceRequest,
) -> Result<LocalProjectRunSummary, LocalProjectError> {
    // This preflight is deliberately before the project runtime.  A malformed
    // declaration therefore cannot publish a project execution result.
    let prepared = host_resource::prepare_request(request)
        .map_err(|error| LocalProjectError::new("E_HOST_RESOURCE_PREFLIGHT", error))?;
    build_project_run_summary_with_supported_profile_and_prepared_host_resources(
        source,
        target,
        inspect_graph,
        inspect_lock,
        &prepared,
    )
}

pub fn build_project_run_summary_with_supported_profile_and_prepared_host_resources(
    source: VirtualProjectSource,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
    prepared: &PreparedHostResources,
) -> Result<LocalProjectRunSummary, LocalProjectError> {
    let mut summary =
        build_project_run_summary_internal(source, target, inspect_graph, inspect_lock, true)?;
    attach_host_resources_to_project_summary(&mut summary, prepared);
    Ok(summary)
}

pub(super) fn attach_host_resources_to_project_summary(
    summary: &mut LocalProjectRunSummary,
    prepared: &PreparedHostResources,
) {
    let transcript =
        serde_json::to_value(&prepared.transcript).expect("host resource transcript serializes");
    let receipt =
        serde_json::to_value(&prepared.receipt).expect("host resource receipt serializes");
    if let Some(envelope) = summary.runtime_execution_envelope.as_object_mut() {
        envelope.insert("host_resource_transcript".to_string(), transcript.clone());
        envelope.insert("host_resource_receipt".to_string(), receipt.clone());
    }
    if let Some(preparation) = summary.execution_preparation.as_object_mut() {
        preparation.insert("host_resource_transcript".to_string(), transcript.clone());
        preparation.insert("host_resource_receipt".to_string(), receipt.clone());
    }
    summary.host_resource_transcript = Some(transcript);
    summary.host_resource_receipt = Some(receipt);
    // A host declaration is part of the invocation, not a display-only
    // annotation.  Rebind this project-result identity after the ordinary
    // runtime result is known, without changing legacy project-run hashes.
    summary.summary_sha256 = sha256_hex(
        serde_json::to_string(&serde_json::json!({
            "schema": "ddn.local_project_run_summary.host_resource_binding.v1",
            "base_summary_sha256": summary.summary_sha256,
            "transcript_identity": prepared.transcript.transcript_identity,
            "receipt_identity": prepared.receipt.receipt_identity,
        }))
        .expect("host resource project summary hash input serialization")
        .as_bytes(),
    );
}

pub(super) fn build_project_run_summary_internal(
    source: VirtualProjectSource,
    target: &str,
    inspect_graph: bool,
    inspect_lock: bool,
    supported_profile: bool,
) -> Result<LocalProjectRunSummary, LocalProjectError> {
    let source_profile = project_source_profile(&source, supported_profile.then_some(
        crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY,
    ))?;
    let requested_target = target.trim();
    if supported_profile {
        validate_current_project_sources_before_link(&source)?;
    }
    let project_asset_registry = project_asset_registry_from_source(&source)?;
    let domain_evidence_manifests = domain_evidence_manifest_identities_from_source(&source)?;
    let sealed_dataset_assets = project_asset_registry
        .as_deref()
        .map(ProjectAssetRegistry::sealed_dataset_identities)
        .transpose()
        .map_err(|err| {
            LocalProjectError::with_path(
                "E_PROJECT_DATASET_BINDING",
                "ddn.asset.json",
                format!("sealed dataset binding 검증 실패: {err}"),
            )
        })?
        .unwrap_or_default();
    let graph = build_project_graph(source.clone())?;
    let target = graph.start_definition.as_deref().ok_or_else(||
        LocalProjectError::new("E_PROJECT_START_REQUIRED", "시작 없는 라이브러리는 실행할 수 없습니다"))?;
    if !requested_target.is_empty() && requested_target != target {
        return Err(LocalProjectError::new("E_PROJECT_START_OVERRIDE", "호출자가 얽음의 시작을 덮을 수 없습니다"));
    }
    let target_path = graph.targets.get(target).cloned().ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_TARGET_MISSING",
            graph.root_path.clone(),
            format!("프로젝트 목표 {target}을 찾을 수 없습니다"),
        )
    })?;
    if inspect_lock && !graph.gaji_aliases.is_empty() {
        build_project_gaji_lock(source.clone(), true)?;
    }
    let lock_sha256 = if graph.gaji_aliases.is_empty() {
        None
    } else {
        Some(build_project_gaji_lock(source.clone(), false)?.lock_sha256)
    };
    // project-run은 아래의 asset-aware 공통 runtime에서 실제 실행을 한 번 수행한다.
    // linker 내부의 asset-unaware 사전 실행을 중복 호출하면 검증된 registry를 잃는다.
    let linked = build_linked_project_with_options(source, Some(&target_path), false, None)?;
    // Current V25 project sources must take the same shared runtime profile as
    // the explicit supported-profile entrypoint.  Leaving this choice to the
    // legacy project-run default makes a valid `:=` module reach an old
    // execution path with no V25 update wrapper.
    let v25_named_target = graph.start_definition.is_some();
    let runtime = if v25_named_target {
        runtime_surface::run_summary_from_supported_source_profile_with_update_name_diagnostic(
            &linked.execution_source,
            &target_path,
            Some(1),
            Some(source_profile),
            target,
            project_asset_registry,
        )
    } else { match project_asset_registry {
        Some(registry) => runtime_surface::run_summary_from_supported_source_profile_with_project_assets_diagnostic(
                &linked.execution_source,
                &target_path,
                Some(1),
                Some(source_profile),
                registry,
            ),
        None => runtime_surface::run_summary_from_supported_source_profile_diagnostic(
                &linked.execution_source,
                &target_path,
                Some(1),
                Some(source_profile),
            ),
    }}
    .map_err(|err| {
        LocalProjectError::from_execution_diagnostic(linked.remap_execution_diagnostic(err))
    })?;
    let symbols = linked.symbol_boundary;
    let output_rows = runtime
        .get("output_rows")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default();
    let runtime_output_log_texts = json_string_array(runtime.get("output_log_texts"));
    let runtime_final_row = runtime
        .get("final_row")
        .cloned()
        .unwrap_or_else(|| JsonValue::Object(Default::default()));
    let runtime_state_hash = runtime
        .get("state_hash")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
    let runtime_engine_state = runtime
        .get("engine_state")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let work_cost_value = runtime.get("work_cost_receipt").ok_or_else(|| {
        LocalProjectError::with_path(
            E_WORK_COST_REGISTRY_MISSING_CANDIDATE,
            &target_path,
            "공통 runtime 결과에 WorkCostV1 영수증이 없습니다",
        )
    })?;
    let work_cost_counters: WorkCostCounters = serde_json::from_value(
        work_cost_value
            .get("counters")
            .cloned()
            .unwrap_or(JsonValue::Null),
    )
    .map_err(|error| {
        LocalProjectError::with_path(
            E_WORK_COST_REGISTRY_MISSING_CANDIDATE,
            &target_path,
            format!("WorkCostV1 counter 영수증을 읽을 수 없습니다: {error}"),
        )
    })?;
    let work_cost_limits: StandardResourceLimitsV1 = serde_json::from_value(
        work_cost_value
            .get("limits")
            .cloned()
            .unwrap_or(JsonValue::Null),
    )
    .map_err(|error| {
        LocalProjectError::with_path(
            E_WORK_COST_REGISTRY_MISSING_CANDIDATE,
            &target_path,
            format!("WorkCostV1 limit 영수증을 읽을 수 없습니다: {error}"),
        )
    })?;
    let mut work_cost = WorkCostMeter::from_receipt_parts(
        work_cost_counters,
        work_cost_limits,
        work_cost_value
            .get("rd206_resource_capability")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
    );
    let mut runtime_execution_envelope = runtime
        .get("execution_envelope")
        .cloned()
        .unwrap_or(JsonValue::Null);
    if let Some(envelope) = runtime_execution_envelope.as_object_mut() {
        envelope.insert(
            "domain_evidence_manifests".to_string(),
            serde_json::to_value(&domain_evidence_manifests)
                .expect("domain evidence identities serialize"),
        );
    }
    let canonical_ddn = runtime
        .get("canonical_ddn")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
    let linked_execution_source_sha256 = sha256_hex(linked.execution_source.as_bytes());
    let linked_execution_canonical_sha256 = sha256_hex(canonical_ddn.as_bytes());
    let project_observation_trace = runtime
        .get("public_observation_trace")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let project_observation_trace_hash = runtime
        .get("public_observation_trace_hash")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
    let project_observation_table_projection = runtime
        .get("public_observation_table_projection")
        .cloned()
        .unwrap_or(JsonValue::Null);
    let execution_preparation = json!({
        "schema": "ddn.local_project.execution_preparation.v1",
        "root_path": graph.root_path,
        "entry_path": graph.entry_path,
        "target": target,
        "target_path": target_path,
        "file_count": graph.file_count,
        "graph": {
            "project_graph_sha256": graph.project_graph_sha256,
            "module_graph_order": symbols.module_graph_order,
            "module_graph_sha256": symbols.module_graph_sha256,
        },
        "lock": {
            "inspected": inspect_lock,
            "required": !graph.gaji_aliases.is_empty(),
            "lock_sha256": lock_sha256,
        },
        "linked_source": {
            "source_sha256": linked_execution_source_sha256,
            "canonical_sha256": linked_execution_canonical_sha256,
        },
        "runtime_request": {
            "frontdoor": LOCAL_PROJECT_RUN_SEMANTIC_IDENTITY,
            "requested_ticks": 1,
            "target": target,
            "target_path": target_path,
        },
        "limits": {
            "max_project_files": MAX_PROJECT_FILES,
            "max_file_bytes": MAX_FILE_BYTES,
            "max_serialized_artifact_bytes": MAX_SERIALIZED_ARTIFACT_BYTES,
        },
        "provenance": {
            "imports": symbols.imports,
            "exports": symbols.exports,
            "resolved_symbols": symbols.resolved_symbols,
        },
        "sealed_dataset_assets": sealed_dataset_assets,
        "domain_evidence_manifests": domain_evidence_manifests,
        "diagnostics": [],
        "state_hash_formula_change": false,
    });
    let summary_hash_input = serde_json::to_string(&json!({
        "schema": "ddn.local_project_run_summary.hash_input.v1",
        "target": target,
        "target_path": target_path,
        "project_graph_sha256": graph.project_graph_sha256,
        "lock_sha256": lock_sha256,
        "symbol_boundary_sha256": symbols.symbol_boundary_sha256,
        "output_rows": output_rows,
        "runtime_output_log_texts": runtime_output_log_texts,
        "runtime_state_hash": runtime_state_hash,
        "runtime_engine_state": runtime_engine_state,
        "runtime_execution_envelope": runtime_execution_envelope,
        "execution_preparation": execution_preparation,
        "project_observation_trace": project_observation_trace,
        "project_observation_trace_hash": project_observation_trace_hash,
        "project_observation_table_projection": project_observation_table_projection,
        "sealed_dataset_assets": sealed_dataset_assets,
        "domain_evidence_manifests": domain_evidence_manifests,
        "linked_execution_source_sha256": linked_execution_source_sha256,
        "linked_execution_canonical_sha256": linked_execution_canonical_sha256,
    }))
    .expect("local project run summary hash input serialization");
    let bounded_state_sha256 = sha256_hex(
        serde_json::to_string(&json!({
            "schema": "ddn.local_project_run_summary.bounded_state.v1",
            "state_hash": runtime_state_hash,
            "output_rows": output_rows,
        }))
        .expect("bounded state hash input serialization")
        .as_bytes(),
    );
    let summary_sha256 = sha256_hex(summary_hash_input.as_bytes());
    let mut summary = LocalProjectRunSummary {
        schema: LOCAL_PROJECT_RUN_SUMMARY_SCHEMA,
        frontdoor: LOCAL_PROJECT_RUN_SEMANTIC_IDENTITY,
        root_path: graph.root_path,
        entry_path: graph.entry_path,
        target: target.to_string(),
        target_path,
        file_count: graph.file_count,
        project_graph_sha256: graph.project_graph_sha256,
        lock_sha256,
        symbol_boundary_sha256: symbols.symbol_boundary_sha256,
        output_rows,
        runtime_output_log_texts,
        runtime_final_row,
        runtime_state_hash,
        runtime_engine_state,
        runtime_execution_envelope,
        work_cost_receipt: work_cost.receipt(),
        execution_preparation,
        project_observation_trace,
        project_observation_trace_hash,
        project_observation_table_projection,
        sealed_dataset_assets,
        domain_evidence_manifests,
        host_resource_transcript: None,
        host_resource_receipt: None,
        linked_execution_source: linked.execution_source.clone(),
        linked_execution_canonical_ddn: canonical_ddn.clone(),
        linked_execution_source_sha256,
        linked_execution_canonical_sha256,
        bounded_state_sha256,
        graph_inspected: inspect_graph,
        lock_inspected: inspect_lock,
        summary_sha256,
        diagnostics: Vec::new(),
    };
    for _ in 0..16 {
        let bytes = serde_json::to_vec(&summary)
            .map_err(|error| {
                LocalProjectError::with_path(
                    "E_RESULT_ARTIFACT_SERIALIZE",
                    &summary.target_path,
                    error.to_string(),
                )
            })?
            .len();
        if summary.work_cost_receipt.counters.result_artifact_bytes == bytes as u64 {
            return Ok(summary);
        }
        work_cost
            .set_result_artifact_bytes(bytes)
            .map_err(|failure| {
                let receipt = serde_json::to_string(&failure.receipt)
                    .unwrap_or_else(|_| "{\"receipt_error\":true}".to_string());
                LocalProjectError::with_path(
                    failure.code,
                    &summary.target_path,
                    format!("{failure}; work_cost_receipt={receipt}"),
                )
            })?;
        summary.work_cost_receipt = work_cost.receipt();
    }
    Err(LocalProjectError::with_path(
        "E_RESULT_ARTIFACT_SIZE_FIXED_POINT",
        &summary.target_path,
        "project-run 결과 artifact byte 크기와 WorkCostV1 영수증이 수렴하지 않았습니다",
    ))
}

pub fn discover_virtual_project_root(
    request: ProjectRootDiscoveryRequest,
) -> Result<ProjectRootDiscoveryResult, LocalProjectError> {
    if request.schema != PROJECT_ROOT_DISCOVERY_REQUEST_SCHEMA {
        return Err(LocalProjectError::new(
            "E_PROJECT_DISCOVERY_SCHEMA",
            format!("schema는 {PROJECT_ROOT_DISCOVERY_REQUEST_SCHEMA} 이어야 합니다"),
        ));
    }
    let start = normalize_project_path(&request.start_path)?;
    let explicit = request
        .explicit_project_path
        .as_deref()
        .map(normalize_project_path)
        .transpose()?;
    let mut files = BTreeSet::new();
    let mut file_texts = BTreeMap::new();
    for file in request.files {
        let path = normalize_project_path(&file.path)?;
        if let Some(text) = file.utf8_text {
            file_texts.insert(path.clone(), text);
        }
        files.insert(path);
    }
    let path = explicit.unwrap_or(start);
    let text = file_texts.get(&path).ok_or_else(|| LocalProjectError::with_path(
        "E_PROJECT_ROOT_EXPLICIT_REQUIRED", &path, "얽음 원문 파일을 명시해 주세요. 폴더나 파일명 관례로 찾지 않습니다"))?;
    reject_source_bom(text, Some(&path))?;
    ddonirang_lang::work_root::parse_work_root(text).map_err(|error|
        LocalProjectError::with_path("E_PROJECT_ROOT_PARSE", &path, error.message))?;
    Ok(ProjectRootDiscoveryResult { schema: PROJECT_ROOT_DISCOVERY_SCHEMA, root_kind: "explicit_work_root",
        root_marker_path: Some(path), project_json_path: None, used_fallback: false, diagnostics: Vec::new() })
}

pub fn discover_disk_project_root(
    start: &Path,
    explicit_project: Option<&Path>,
) -> Result<ProjectRootDiscoveryResult, LocalProjectError> {
    let path = explicit_project.unwrap_or(start);
    if !path.is_file() {
        return Err(LocalProjectError::new("E_PROJECT_ROOT_EXPLICIT_REQUIRED", "얽음 원문 파일을 명시해 주세요. 상위 폴더에서 찾지 않습니다"));
    }
    let text = fs::read_to_string(path).map_err(|error| LocalProjectError::new("E_PROJECT_FILE_READ", error.to_string()))?;
    reject_source_bom(&text, path.to_str())?;
    ddonirang_lang::work_root::parse_work_root(&text).map_err(|error|
        LocalProjectError::new("E_PROJECT_ROOT_PARSE", error.message))?;
    Ok(ProjectRootDiscoveryResult { schema: PROJECT_ROOT_DISCOVERY_SCHEMA, root_kind: "explicit_work_root",
        root_marker_path: Some(path.to_string_lossy().into_owned()), project_json_path: None,
        used_fallback: false, diagnostics: Vec::new() })
}

pub fn build_virtual_project_source_from_disk(
    start: &Path,
    explicit_project: Option<&Path>,
) -> Result<VirtualProjectSource, LocalProjectError> {
    let root_file = explicit_project.unwrap_or(start);
    if !root_file.is_file() {
        return Err(LocalProjectError::new("E_PROJECT_ROOT_EXPLICIT_REQUIRED", "--project는 폴더가 아닌 얽음 원문 파일이어야 합니다"));
    }
    let root_dir = root_file.parent().unwrap_or_else(|| Path::new("."));
    let root_path = root_file.file_name().and_then(|name| name.to_str()).ok_or_else(||
        LocalProjectError::new("E_PROJECT_PATH", "뿌리 파일 경로는 UTF-8이어야 합니다"))?.to_string();
    let root_text = fs::read_to_string(root_file).map_err(|error|
        LocalProjectError::with_path("E_PROJECT_FILE_READ", &root_path, error.to_string()))?;
    reject_source_bom(&root_text, Some(&root_path))?;
    let root = ddonirang_lang::work_root::parse_work_root(&root_text).map_err(|error|
        LocalProjectError::with_path("E_PROJECT_ROOT_PARSE", &root_path, error.message))?;
    let modules = root.typed_modules().map_err(|error|
        LocalProjectError::with_path("E_PROJECT_MODULE_DECLARATION", &root_path, error.message))?;
    let start = root.typed_start().map_err(|error|
        LocalProjectError::with_path("E_PROJECT_START_DECLARATION", &root_path, error.message))?;
    let entry_path = start.as_ref().map(|start| {
        modules.iter().find(|module| module.name == start.module)
            .ok_or_else(|| LocalProjectError::new("E_PROJECT_START_MODULE_MISSING", "시작 모듈이 없습니다"))
            .and_then(|module| normalize_project_reference(&module.head))
    }).transpose()?.unwrap_or_default();
    let mut paths = BTreeSet::new();
    let mut visited = 0usize;
    for module in modules { for raw in module.sources {
        let path = normalize_project_reference(raw.trim_end_matches('/'))?;
        collect_declared_disk_sources(root_dir, &root_dir.join(path), &mut paths, &mut visited)?;
    } }
    let mut files = vec![VirtualProjectFileSource { path: root_path.clone(), role: "root".into(), utf8_text: root_text }];
    for path in paths {
        let disk_path = root_dir.join(&path);
        ensure_no_symlink_escape(&disk_path, root_dir)?;
        let text = fs::read_to_string(&disk_path).map_err(|error|
            LocalProjectError::with_path("E_PROJECT_FILE_READ", &path, error.to_string()))?;
        reject_source_bom(&text, Some(&path))?;
        files.push(VirtualProjectFileSource { path, role: "ddn_source".into(), utf8_text: text });
    }
    let source = VirtualProjectSource {
        schema: VIRTUAL_PROJECT_SOURCE_SCHEMA.to_string(),
        kind: Some(LOCAL_PROJECT_TREE_KIND.to_string()),
        package_hash: None,
        root_path,
        entry_path,
        files,
    };
    build_project_graph(source.clone())?;
    Ok(source)
}

pub(super) fn collect_declared_disk_sources(root: &Path, path: &Path, paths: &mut BTreeSet<String>, visited: &mut usize)
    -> Result<(), LocalProjectError> {
    *visited += 1;
    if *visited > 4096 || paths.len() >= MAX_PROJECT_FILES {
        return Err(LocalProjectError::new("E_PROJECT_FILE_COUNT_LIMIT", "선언 원문 집합의 탐색 한도를 넘었습니다"));
    }
    ensure_no_symlink_escape(path, root)?;
    let metadata = fs::symlink_metadata(path).map_err(|error|
        LocalProjectError::new("E_PROJECT_FILE_READ", error.to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(LocalProjectError::new("E_PROJECT_SYMLINK_ESCAPE", "원문 집합에 심볼릭 링크를 넣을 수 없습니다"));
    }
    if metadata.is_dir() {
        let mut children = fs::read_dir(path).map_err(|error| LocalProjectError::new("E_PROJECT_DIR_READ", error.to_string()))?
            .collect::<Result<Vec<_>, _>>().map_err(|error| LocalProjectError::new("E_PROJECT_DIR_READ", error.to_string()))?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            let child_path = child.path();
            if child_path.is_dir() || child_path.extension().is_some_and(|extension| extension == "ddn") {
                collect_declared_disk_sources(root, &child_path, paths, visited)?;
            }
        }
    } else if metadata.is_file() {
        let logical = path.strip_prefix(root).map_err(|_| LocalProjectError::new("E_PROJECT_PATH_ESCAPE", "원문이 뿌리 밖에 있습니다"))?
            .to_str().ok_or_else(|| LocalProjectError::new("E_PROJECT_PATH", "원문 경로는 UTF-8이어야 합니다"))?.replace('\\', "/");
        paths.insert(normalize_project_path(&logical)?);
    }
    Ok(())
}

pub fn normalize_virtual_project_source(
    source: VirtualProjectSource,
) -> Result<NormalizedProjectManifest, LocalProjectError> {
    if source.files.iter().any(|file| file.path == "ddn.asset.json" || matches!(normalize_role(&file.role).as_str(),
        "asset_manifest" | "sealed_dataset_envelope" | "dataset_payload" | "transform_recipe" | "domain_evidence_manifest")) {
        require_beta_project_feature("E_ALPHA_DATASET_UNSUPPORTED", "자료셋·증거 자료 연결은 알파에서 지원하지 않습니다. 베타에서 선언 방식과 함께 지원할 예정입니다")?;
    }
    if source.schema != VIRTUAL_PROJECT_SOURCE_SCHEMA {
        return Err(LocalProjectError::new(
            "E_VIRTUAL_PROJECT_SCHEMA",
            format!("schema는 {VIRTUAL_PROJECT_SOURCE_SCHEMA} 이어야 합니다"),
        ));
    }
    if let Some(kind) = &source.kind {
        if kind != LOCAL_PROJECT_TREE_KIND {
            return Err(LocalProjectError::new(
                "E_VIRTUAL_PROJECT_KIND",
                format!("kind는 {LOCAL_PROJECT_TREE_KIND} 이어야 합니다"),
            ));
        }
    }
    if source.files.len() > MAX_PROJECT_FILES {
        return Err(LocalProjectError::new(
            "E_PROJECT_FILE_COUNT_LIMIT",
            format!("파일 수가 {MAX_PROJECT_FILES}개를 넘었습니다"),
        ));
    }
    validate_domain_evidence_manifest_files(&source)?;

    let root_path = normalize_project_path(&source.root_path)?;
    let entry_path = if source.entry_path.is_empty() { String::new() } else {
        normalize_project_path(&source.entry_path)?
    };
    let mut normalized_paths = BTreeSet::new();
    let mut case_fold_paths = BTreeMap::<String, String>::new();
    let mut files = Vec::with_capacity(source.files.len());
    let mut total_utf8_bytes = 0usize;
    for file in source.files {
        let path = normalize_project_path(&file.path)?;
        validate_role(&file.role, &path)?;
        if matches!(normalize_role(&file.role).as_str(), "root" | "ddn_source") {
            reject_source_bom(&file.utf8_text, Some(&path))?;
        }
        if !normalized_paths.insert(path.clone()) {
            return Err(LocalProjectError::with_path(
                "E_PROJECT_PATH_DUPLICATE",
                path,
                "같은 정규 경로가 두 번 들어왔습니다",
            ));
        }
        let folded = path.to_ascii_lowercase();
        if let Some(existing) = case_fold_paths.insert(folded, path.clone()) {
            return Err(LocalProjectError::with_path(
                "E_PROJECT_PATH_CASE_COLLISION",
                path,
                format!("Windows 대소문자 접기 기준으로 {existing} 경로와 충돌합니다"),
            ));
        }
        let byte_len = file.utf8_text.as_bytes().len();
        if byte_len > MAX_FILE_BYTES {
            return Err(LocalProjectError::with_path(
                "E_PROJECT_FILE_SIZE_LIMIT",
                path,
                format!("파일이 {MAX_FILE_BYTES} 바이트 제한을 넘었습니다"),
            ));
        }
        total_utf8_bytes = total_utf8_bytes.checked_add(byte_len).ok_or_else(|| {
            LocalProjectError::new("E_PROJECT_TOTAL_SIZE_LIMIT", "프로젝트 크기가 너무 큽니다")
        })?;
        if total_utf8_bytes > MAX_PROJECT_TOTAL_UTF8_BYTES {
            return Err(LocalProjectError::new(
                "E_PROJECT_TOTAL_SIZE_LIMIT",
                format!("프로젝트 UTF-8 본문 합계가 {MAX_PROJECT_TOTAL_UTF8_BYTES} 바이트 제한을 넘었습니다"),
            ));
        }
        let sha256 = sha256_hex(file.utf8_text.as_bytes());
        files.push(PendingFile {
            path,
            role: normalize_role(&file.role),
            byte_len,
            sha256,
        });
    }
    if !normalized_paths.contains(&root_path) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ROOT_MISSING",
            root_path,
            "virtual project source에 root 파일이 없습니다",
        ));
    }
    if !entry_path.is_empty() && !normalized_paths.contains(&entry_path) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ENTRY_MISSING",
            entry_path,
            "virtual project source에 entry 파일이 없습니다",
        ));
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    let graph_input = graph_hash_input(&root_path, &entry_path, &files);
    let project_graph_sha256 = sha256_hex(graph_input.as_bytes());
    let lock_sha256 = files
        .iter()
        .find(|file| file.path == "ddn.lock")
        .map(|file| file.sha256.clone());
    let mut manifest = NormalizedProjectManifest {
        schema: VIRTUAL_PROJECT_MANIFEST_SCHEMA,
        kind: LOCAL_PROJECT_TREE_KIND,
        root_path,
        entry_path,
        file_count: files.len(),
        total_utf8_bytes,
        files: files
            .iter()
            .map(|file| NormalizedProjectFile {
                path: file.path.clone(),
                role: file.role.clone(),
                byte_length: file.byte_len,
                sha256: file.sha256.clone(),
            })
            .collect(),
        project_graph_sha256,
        lock_sha256,
        canonical_manifest_sha256: String::new(),
        limits: ProjectLimits {
            max_files: MAX_PROJECT_FILES,
            max_file_bytes: MAX_FILE_BYTES,
            max_serialized_artifact_bytes: MAX_SERIALIZED_ARTIFACT_BYTES,
        },
    };
    let manifest_without_hash = to_canonical_json_line(&manifest);
    if manifest_without_hash.as_bytes().len() > MAX_SERIALIZED_ARTIFACT_BYTES {
        return Err(LocalProjectError::new(
            "E_PROJECT_SERIALIZED_SIZE_LIMIT",
            format!("정규 manifest가 {MAX_SERIALIZED_ARTIFACT_BYTES} 바이트 제한을 넘었습니다"),
        ));
    }
    manifest.canonical_manifest_sha256 = sha256_hex(manifest_without_hash.as_bytes());
    Ok(manifest)
}

pub fn normalize_project_path(raw: &str) -> Result<String, LocalProjectError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(LocalProjectError::new(
            "E_PROJECT_PATH_EMPTY",
            "빈 경로는 사용할 수 없습니다",
        ));
    }
    if trimmed.contains('\\') {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_PATH_BACKSLASH",
            trimmed,
            "경로 구분자는 / 만 허용합니다",
        ));
    }
    if trimmed.contains('\0') || trimmed.chars().any(|ch| ch.is_control()) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_PATH_CONTROL",
            trimmed,
            "제어 문자가 들어간 경로는 사용할 수 없습니다",
        ));
    }
    if trimmed.starts_with('/')
        || trimmed.starts_with("//")
        || trimmed.contains("://")
        || looks_like_windows_drive(trimmed)
    {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_PATH_ABSOLUTE",
            trimmed,
            "프로젝트 경로는 상대 경로여야 합니다",
        ));
    }
    let nfc = trimmed.nfc().collect::<String>();
    let parts: Vec<&str> = nfc.split('/').collect();
    if parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_PATH_SEGMENT",
            trimmed,
            ". 또는 .. 또는 빈 경로 조각은 사용할 수 없습니다",
        ));
    }
    Ok(parts.join("/"))
}

pub(super) fn validate_role(raw: &str, path: &str) -> Result<(), LocalProjectError> {
    let role = normalize_role(raw);
    match role.as_str() {
        "root"
        | "ddn_source"
        | "project_json"
        | "lock"
        | "gaji_manifest"
        | "metadata"
        | "asset_manifest"
        | "sealed_dataset_envelope"
        | "dataset_payload"
        | "transform_recipe"
        | "domain_evidence_manifest" => Ok(()),
        _ => Err(LocalProjectError::with_path(
            "E_PROJECT_FILE_ROLE",
            path,
            format!("지원하지 않는 project file role입니다: {raw}"),
        )),
    }
}

pub(super) fn validate_domain_evidence_manifest_files(
    source: &VirtualProjectSource,
) -> Result<(), LocalProjectError> {
    domain_evidence_manifest_identities_from_source(source).map(|_| ())
}

pub(super) fn domain_evidence_manifest_identities_from_source(
    source: &VirtualProjectSource,
) -> Result<Vec<EvidenceManifestIdentity>, LocalProjectError> {
    let evidence_files = source
        .files
        .iter()
        .filter(|file| normalize_role(&file.role) == "domain_evidence_manifest")
        .collect::<Vec<_>>();
    if evidence_files.is_empty() {
        return Ok(Vec::new());
    }

    let mut datasets = Vec::new();
    for file in source
        .files
        .iter()
        .filter(|file| normalize_role(&file.role) == "sealed_dataset_envelope")
    {
        let dataset: SealedDomainDataset =
            serde_json::from_str(&file.utf8_text).map_err(|err| {
                LocalProjectError::with_path(
                    "E_PROJECT_EVIDENCE_DATASET_CONTEXT",
                    file.path.clone(),
                    format!("evidence 대상 dataset envelope를 읽을 수 없습니다: {err}"),
                )
            })?;
        if dataset.schema != SEALED_DOMAIN_DATASET_SCHEMA {
            return Err(LocalProjectError::with_path(
                "E_PROJECT_EVIDENCE_DATASET_CONTEXT",
                file.path.clone(),
                format!("dataset schema는 {SEALED_DOMAIN_DATASET_SCHEMA}여야 합니다"),
            ));
        }
        let project_revision_ref = match &dataset.source {
            DatasetSource::ProjectAuthored {
                project_revision_ref,
                ..
            } => Some(project_revision_ref.clone()),
            DatasetSource::ExternalSnapshot { .. } => None,
        };
        datasets.push(EvidenceDatasetTarget {
            dataset_content_sha256: dataset.content_sha256,
            project_revision_ref,
            column_names: dataset
                .ordered_columns
                .into_iter()
                .map(|column| column.name)
                .collect(),
            rows: dataset
                .ordered_rows
                .into_iter()
                .map(|row| EvidenceDatasetRowTarget {
                    source_ordinal: row.source_ordinal,
                    cell_statuses: row
                        .cells
                        .into_iter()
                        .map(|cell| match cell {
                            DatasetCell::Present { .. } => EvidenceCellStatus::Present,
                            DatasetCell::Gap => EvidenceCellStatus::Gap,
                            DatasetCell::Missing { reason } => EvidenceCellStatus::Missing {
                                reason: match reason {
                                    MissingReason::CalculationFailed => "#계산실패",
                                    MissingReason::SourceUnavailable => "#원천없음",
                                    MissingReason::RecordDamaged => "#기록손상",
                                }
                                .to_string(),
                            },
                            DatasetCell::Pruned => EvidenceCellStatus::Pruned,
                        })
                        .collect(),
                })
                .collect(),
        });
    }

    let mut identities = Vec::with_capacity(evidence_files.len());
    for file in evidence_files {
        let manifest = parse_and_validate_domain_evidence_manifest(&file.utf8_text, &datasets)
            .map_err(|err| {
                LocalProjectError::with_path(
                    err.code,
                    file.path.clone(),
                    format!(
                        "domain evidence manifest 검증 실패({}): {}",
                        err.path, err.message
                    ),
                )
            })?;
        identities.push(
            evidence_manifest_identity(&file.path, &manifest, &datasets).map_err(|err| {
                LocalProjectError::with_path(
                    err.code,
                    file.path.clone(),
                    format!(
                        "domain evidence identity 생성 실패({}): {}",
                        err.path, err.message
                    ),
                )
            })?,
        );
    }
    identities.sort_by(|left, right| left.source_ref.cmp(&right.source_ref));
    Ok(identities)
}

pub(super) fn normalize_role(raw: &str) -> String {
    match raw.trim() {
        "뿌리.ddn" => "root".to_string(),
        "DDN source" => "ddn_source".to_string(),
        other => other.to_string(),
    }
}

pub(super) fn looks_like_windows_drive(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

pub(super) fn discover_root_from_file_set(
    files: &BTreeSet<String>,
    file_texts: Option<&BTreeMap<String, String>>,
    start_path: &str,
    explicit_project_path: Option<&str>,
) -> Result<ProjectRootDiscoveryResult, LocalProjectError> {
    if let Some(explicit) = explicit_project_path {
        return discover_explicit_from_file_set(files, file_texts, explicit);
    }
    let dirs = virtual_parent_dirs(files, start_path);
    for dir in &dirs {
        if has_file(files, dir, PROJECT_ROOT_FILE) {
            if let Some(legacy) = legacy_in_dir(files, dir) {
                return Err(LocalProjectError::with_path(
                    "E_PROJECT_ROOT_AMBIGUOUS",
                    join_project_path(dir, legacy),
                    "현재 root 이름과 legacy root 이름이 같은 디렉터리에 있습니다",
                ));
            }
            ensure_no_project_json_entry_conflict_virtual(files, file_texts, dir)?;
            return Ok(root_result(join_project_path(dir, PROJECT_ROOT_FILE), None));
        }
    }
    for dir in &dirs {
        if let Some(legacy) = legacy_in_dir(files, dir) {
            return Err(LocalProjectError::with_path(
                "E_LEGACY_PROJECT_ROOT_NAME",
                join_project_path(dir, legacy),
                "legacy root 파일명은 current alias가 아닙니다. 뿌리.ddn을 사용하세요",
            ));
        }
    }
    for dir in &dirs {
        if has_file(files, dir, PROJECT_JSON_FILE) {
            return Ok(root_result(
                String::new(),
                Some(join_project_path(dir, PROJECT_JSON_FILE)),
            ));
        }
    }
    Err(LocalProjectError::new(
        "E_PROJECT_ROOT_MISSING",
        "뿌리.ddn 또는 ddn.project.json을 찾지 못했습니다",
    ))
}

pub(super) fn discover_explicit_from_file_set(
    files: &BTreeSet<String>,
    file_texts: Option<&BTreeMap<String, String>>,
    explicit: &str,
) -> Result<ProjectRootDiscoveryResult, LocalProjectError> {
    if path_file_name(explicit) == Some(PROJECT_ROOT_FILE) && files.contains(explicit) {
        ensure_no_project_json_entry_conflict_virtual(files, file_texts, &parent_dir(explicit))?;
        return Ok(root_result(explicit.to_string(), None));
    }
    if path_file_name(explicit) == Some(PROJECT_JSON_FILE) && files.contains(explicit) {
        return Ok(root_result(String::new(), Some(explicit.to_string())));
    }
    if LEGACY_ROOT_FILES
        .iter()
        .any(|legacy| path_file_name(explicit) == Some(*legacy) && files.contains(explicit))
    {
        return Err(LocalProjectError::with_path(
            "E_LEGACY_PROJECT_ROOT_NAME",
            explicit,
            "legacy root 파일명은 current alias가 아닙니다. 뿌리.ddn을 사용하세요",
        ));
    }
    let dir = if files.contains(explicit) {
        parent_dir(explicit)
    } else {
        explicit.to_string()
    };
    if has_file(files, &dir, PROJECT_ROOT_FILE) {
        if let Some(legacy) = legacy_in_dir(files, &dir) {
            return Err(LocalProjectError::with_path(
                "E_PROJECT_ROOT_AMBIGUOUS",
                join_project_path(&dir, legacy),
                "현재 root 이름과 legacy root 이름이 같은 디렉터리에 있습니다",
            ));
        }
        ensure_no_project_json_entry_conflict_virtual(files, file_texts, &dir)?;
        return Ok(root_result(
            join_project_path(&dir, PROJECT_ROOT_FILE),
            None,
        ));
    }
    if let Some(legacy) = legacy_in_dir(files, &dir) {
        return Err(LocalProjectError::with_path(
            "E_LEGACY_PROJECT_ROOT_NAME",
            join_project_path(&dir, legacy),
            "legacy root 파일명은 current alias가 아닙니다. 뿌리.ddn을 사용하세요",
        ));
    }
    if has_file(files, &dir, PROJECT_JSON_FILE) {
        return Ok(root_result(
            String::new(),
            Some(join_project_path(&dir, PROJECT_JSON_FILE)),
        ));
    }
    Err(LocalProjectError::with_path(
        "E_PROJECT_ROOT_MISSING",
        explicit,
        "명시한 project 위치에서 뿌리.ddn 또는 ddn.project.json을 찾지 못했습니다",
    ))
}

pub(super) fn discover_disk_upward(
    start_dir: &Path,
    start_abs: &Path,
) -> Result<ProjectRootDiscoveryResult, LocalProjectError> {
    let dirs = disk_ancestor_dirs(start_dir);
    for dir in &dirs {
        let root = dir.join(PROJECT_ROOT_FILE);
        if root.is_file() {
            if let Some(legacy) = disk_legacy_in_dir(dir) {
                return Err(LocalProjectError::with_path(
                    "E_PROJECT_ROOT_AMBIGUOUS",
                    legacy
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("legacy"),
                    "현재 root 이름과 legacy root 이름이 같은 디렉터리에 있습니다",
                ));
            }
            ensure_no_project_json_entry_conflict_disk(dir)?;
            ensure_no_symlink_escape(start_abs, dir)?;
            return Ok(root_result(PROJECT_ROOT_FILE.to_string(), None));
        }
    }
    for dir in &dirs {
        if let Some(legacy) = disk_legacy_in_dir(dir) {
            return Err(LocalProjectError::with_path(
                "E_LEGACY_PROJECT_ROOT_NAME",
                legacy
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("legacy"),
                "legacy root 파일명은 current alias가 아닙니다. 뿌리.ddn을 사용하세요",
            ));
        }
    }
    for dir in &dirs {
        if dir.join(PROJECT_JSON_FILE).is_file() {
            ensure_no_symlink_escape(start_abs, dir)?;
            return Ok(root_result(
                String::new(),
                Some(PROJECT_JSON_FILE.to_string()),
            ));
        }
    }
    Err(LocalProjectError::new(
        "E_PROJECT_ROOT_MISSING",
        "뿌리.ddn 또는 ddn.project.json을 찾지 못했습니다",
    ))
}

pub(super) fn discover_disk_project_root_dir(
    start: &Path,
    explicit_project: Option<&Path>,
) -> Result<PathBuf, LocalProjectError> {
    let start_abs = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| LocalProjectError::new("E_PROJECT_CWD", err.to_string()))?
            .join(start)
    };
    if let Some(explicit) = explicit_project {
        let explicit_abs = if explicit.is_absolute() {
            explicit.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|err| LocalProjectError::new("E_PROJECT_CWD", err.to_string()))?
                .join(explicit)
        };
        let dir = explicit_root_dir(&explicit_abs)?;
        let _ = discover_disk_explicit_project(&explicit_abs)?;
        ensure_no_symlink_escape(&start_abs, &dir)?;
        return Ok(dir);
    }
    let start_for_walk = if start_abs.is_file() {
        start_abs
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| start_abs.clone())
    } else {
        start_abs.clone()
    };
    let dirs = disk_ancestor_dirs(&start_for_walk);
    for dir in &dirs {
        if dir.join(PROJECT_ROOT_FILE).is_file() {
            let _ = discover_disk_upward(dir, &start_abs)?;
            return Ok(dir.clone());
        }
    }
    let _ = discover_disk_upward(&start_for_walk, &start_abs)?;
    Err(LocalProjectError::new(
        "E_PROJECT_ROOT_MISSING",
        "project root directory를 찾지 못했습니다",
    ))
}

pub(super) fn discover_disk_explicit_project(
    project: &Path,
) -> Result<ProjectRootDiscoveryResult, LocalProjectError> {
    let file_name = project
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if project.is_file() && file_name == PROJECT_ROOT_FILE {
        let dir = project.parent().unwrap_or(project);
        ensure_no_project_json_entry_conflict_disk(dir)?;
        return Ok(root_result(PROJECT_ROOT_FILE.to_string(), None));
    }
    if project.is_file() && file_name == PROJECT_JSON_FILE {
        return Ok(root_result(
            String::new(),
            Some(PROJECT_JSON_FILE.to_string()),
        ));
    }
    if project.is_file() && LEGACY_ROOT_FILES.contains(&file_name) {
        return Err(LocalProjectError::with_path(
            "E_LEGACY_PROJECT_ROOT_NAME",
            file_name,
            "legacy root 파일명은 current alias가 아닙니다. 뿌리.ddn을 사용하세요",
        ));
    }
    let dir = if project.is_file() {
        project.parent().unwrap_or(project)
    } else {
        project
    };
    if dir.join(PROJECT_ROOT_FILE).is_file() {
        if let Some(legacy) = disk_legacy_in_dir(dir) {
            return Err(LocalProjectError::with_path(
                "E_PROJECT_ROOT_AMBIGUOUS",
                legacy
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("legacy"),
                "현재 root 이름과 legacy root 이름이 같은 디렉터리에 있습니다",
            ));
        }
        ensure_no_project_json_entry_conflict_disk(dir)?;
        return Ok(root_result(PROJECT_ROOT_FILE.to_string(), None));
    }
    if let Some(legacy) = disk_legacy_in_dir(dir) {
        return Err(LocalProjectError::with_path(
            "E_LEGACY_PROJECT_ROOT_NAME",
            legacy
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("legacy"),
            "legacy root 파일명은 current alias가 아닙니다. 뿌리.ddn을 사용하세요",
        ));
    }
    if dir.join(PROJECT_JSON_FILE).is_file() {
        return Ok(root_result(
            String::new(),
            Some(PROJECT_JSON_FILE.to_string()),
        ));
    }
    Err(LocalProjectError::new(
        "E_PROJECT_ROOT_MISSING",
        "명시한 project 위치에서 뿌리.ddn 또는 ddn.project.json을 찾지 못했습니다",
    ))
}

pub(super) fn explicit_root_dir(project: &Path) -> Result<PathBuf, LocalProjectError> {
    let dir = if project.is_file() {
        project.parent().unwrap_or(project)
    } else {
        project
    };
    Ok(dir.to_path_buf())
}

pub(super) fn ensure_no_symlink_escape(start: &Path, root_dir: &Path) -> Result<(), LocalProjectError> {
    let Ok(root_real) = fs::canonicalize(root_dir) else {
        return Ok(());
    };
    let start_real = fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    if !start_real.starts_with(&root_real) {
        return Err(LocalProjectError::new(
            "E_PROJECT_SYMLINK_ESCAPE",
            "project root 밖으로 해소되는 symlink/junction 경로는 사용할 수 없습니다",
        ));
    }
    Ok(())
}

pub(super) fn disk_ancestor_dirs(start: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut current = Some(start);
    while let Some(dir) = current {
        dirs.push(dir.to_path_buf());
        current = dir.parent();
    }
    dirs
}

pub(super) fn disk_legacy_in_dir(dir: &Path) -> Option<PathBuf> {
    LEGACY_ROOT_FILES
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.is_file())
}

pub(super) fn collect_disk_project_files(
    root_dir: &Path,
    current: &Path,
    out: &mut Vec<VirtualProjectFileSource>,
) -> Result<(), LocalProjectError> {
    let mut entries = fs::read_dir(current)
        .map_err(|err| LocalProjectError::new("E_PROJECT_DIR_READ", err.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| LocalProjectError::new("E_PROJECT_DIR_READ", err.to_string()))?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| LocalProjectError::new("E_PROJECT_FILE_TYPE", err.to_string()))?;
        if file_type.is_dir() {
            if should_skip_project_dir(&path) {
                continue;
            }
            collect_disk_project_files(root_dir, &path, out)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let rel_path = path
            .strip_prefix(root_dir)
            .map_err(|_| LocalProjectError::new("E_PROJECT_PATH", "root 밖 파일입니다"))?
            .to_string_lossy()
            .replace('\\', "/");
        let normalized = normalize_project_path(&rel_path)?;
        let utf8_text = fs::read_to_string(&path).map_err(|err| {
            LocalProjectError::with_path(
                "E_PROJECT_FILE_READ",
                normalized.clone(),
                format!("프로젝트 파일을 UTF-8로 읽을 수 없습니다: {err}"),
            )
        })?;
        out.push(VirtualProjectFileSource {
            path: normalized.clone(),
            role: infer_project_file_role(&normalized),
            utf8_text: canonicalize_disk_text(utf8_text),
        });
    }
    Ok(())
}

pub(super) fn canonicalize_disk_text(text: String) -> String {
    if text.contains("\r\n") {
        text.replace("\r\n", "\n")
    } else {
        text
    }
}

pub(super) fn should_skip_project_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git" | "target" | "build" | "out" | "dist" | "node_modules" | ".cargo"
    )
}

pub(super) fn infer_project_file_role(path: &str) -> String {
    if path == PROJECT_ROOT_FILE {
        "root".to_string()
    } else if path == PROJECT_JSON_FILE {
        "project_json".to_string()
    } else if path == "ddn.lock" {
        "lock".to_string()
    } else if path == "ddn.asset.json" {
        "asset_manifest".to_string()
    } else if path.starts_with("evidence/") && path.ends_with(".manifest.json") {
        "domain_evidence_manifest".to_string()
    } else if path.starts_with("datasets/")
        && (path.ends_with("/envelope.json") || path.ends_with(".envelope.json"))
    {
        "sealed_dataset_envelope".to_string()
    } else if path.starts_with("datasets/") && path.ends_with("/recipe.json") {
        "transform_recipe".to_string()
    } else if path.starts_with("datasets/") && path.contains("/payload.") {
        "dataset_payload".to_string()
    } else if path.ends_with("/gaji.toml") {
        "gaji_manifest".to_string()
    } else if path.ends_with(".ddn") {
        "ddn_source".to_string()
    } else {
        "metadata".to_string()
    }
}

pub(super) fn ensure_no_project_json_entry_conflict_virtual(
    files: &BTreeSet<String>,
    file_texts: Option<&BTreeMap<String, String>>,
    dir: &str,
) -> Result<(), LocalProjectError> {
    if !has_file(files, dir, PROJECT_JSON_FILE) {
        return Ok(());
    }
    let Some(texts) = file_texts else {
        return Ok(());
    };
    let root_path = join_project_path(dir, PROJECT_ROOT_FILE);
    let json_path = join_project_path(dir, PROJECT_JSON_FILE);
    let Some(root_text) = texts.get(&root_path) else {
        return Ok(());
    };
    let Some(json_text) = texts.get(&json_path) else {
        return Ok(());
    };
    ensure_entry_fields_match(root_text, json_text, &json_path)
}

pub(super) fn ensure_project_json_entry_matches_texts(
    texts: &BTreeMap<String, String>,
    dir: &str,
    root_path: &str,
) -> Result<(), LocalProjectError> {
    let json_path = join_project_path(dir, PROJECT_JSON_FILE);
    let Some(root_text) = texts.get(root_path) else {
        return Ok(());
    };
    let Some(json_text) = texts.get(&json_path) else {
        return Ok(());
    };
    ensure_entry_fields_match(root_text, json_text, &json_path)
}

pub(super) fn ensure_no_project_json_entry_conflict_disk(dir: &Path) -> Result<(), LocalProjectError> {
    let root_path = dir.join(PROJECT_ROOT_FILE);
    let json_path = dir.join(PROJECT_JSON_FILE);
    if !root_path.is_file() || !json_path.is_file() {
        return Ok(());
    }
    let Ok(root_text) = fs::read_to_string(root_path) else {
        return Ok(());
    };
    let Ok(json_text) = fs::read_to_string(json_path) else {
        return Ok(());
    };
    ensure_entry_fields_match(&root_text, &json_text, PROJECT_JSON_FILE)
}

pub(super) fn ensure_entry_fields_match(
    root_text: &str,
    json_text: &str,
    json_path: &str,
) -> Result<(), LocalProjectError> {
    let Some(root_entry) = extract_ddn_first_entry(root_text) else {
        return Ok(());
    };
    let Some(json_entry) = extract_project_json_entry(json_text) else {
        return Ok(());
    };
    let root_norm = normalize_project_reference(&root_entry)?;
    let json_norm = normalize_project_reference(&json_entry)?;
    if root_norm != json_norm {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_BLOCK_VS_PROJECTJSON_CONFLICT",
            json_path,
            format!("뿌리.ddn 첫글({root_norm})과 ddn.project.json entry({json_norm})가 다릅니다"),
        ));
    }
    Ok(())
}

pub(super) fn extract_ddn_first_entry(text: &str) -> Option<String> {
    let key = text.find("첫글")?;
    let after_key = &text[key..];
    let colon = after_key.find(':')?;
    let after_colon = &after_key[colon + 1..];
    extract_first_quoted_string(after_colon)
}

pub(super) fn extract_project_json_entry(text: &str) -> Option<String> {
    let value: JsonValue = serde_json::from_str(text).ok()?;
    for key in ["entry_path", "entry", "first", "첫글"] {
        if let Some(entry) = value.get(key).and_then(|item| item.as_str()) {
            return Some(entry.to_string());
        }
    }
    value
        .get("targets")
        .and_then(|targets| targets.get("앱"))
        .and_then(|item| item.as_str())
        .map(ToString::to_string)
}

pub(super) fn extract_first_quoted_string(text: &str) -> Option<String> {
    let start = text.find('"')?;
    let rest = &text[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub(super) fn normalize_project_reference(raw: &str) -> Result<String, LocalProjectError> {
    let trimmed = raw.trim().trim_start_matches("./");
    normalize_project_path(trimmed)
}

pub(super) fn normalize_project_reference_allow_root(raw: &str) -> Result<String, LocalProjectError> {
    let trimmed = raw.trim();
    if trimmed == "." || trimmed == "./" {
        return Ok(String::new());
    }
    normalize_project_reference(trimmed)
}

pub(super) fn contains_project_block(text: &str) -> bool {
    Lexer::new(text)
        .tokenize()
        .map(|tokens| {
            tokens
                .windows(2)
                .any(|pair| pair[0].raw == "프로젝트" && matches!(pair[1].kind, TokenKind::LBrace))
        })
        .unwrap_or(false)
}

pub(super) fn extract_project_block(text: &str) -> Option<&str> {
    let project_pos = text.find("프로젝트")?;
    let open_rel = text[project_pos..].find('{')?;
    let open = project_pos + open_rel;
    let close = find_matching_brace(text, open)?;
    Some(&text[open + 1..close])
}

pub(super) fn extract_named_block<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let name_pos = text.find(name)?;
    let open_rel = text[name_pos..].find('{')?;
    let open = name_pos + open_rel;
    let close = find_matching_brace(text, open)?;
    Some(&text[open + 1..close])
}

pub(super) fn extract_named_block_range(text: &str, name: &str) -> Option<(usize, usize)> {
    let name_pos = text.find(name)?;
    let open_rel = text[name_pos..].find('{')?;
    let open = name_pos + open_rel;
    let close = find_matching_brace(text, open)?;
    Some((open + 1, close))
}

pub(super) fn find_named_block_key_span(text: &str, block_name: &str, key: &str) -> Option<(usize, usize)> {
    find_named_block_key_spans(text, block_name, key)
        .into_iter()
        .next()
}

pub(crate) fn find_named_block_key_spans(
    text: &str,
    block_name: &str,
    key: &str,
) -> Vec<(usize, usize)> {
    let Some((start, end)) = extract_named_block_range(text, block_name) else {
        return Vec::new();
    };
    let block = &text[start..end];
    let mut spans = Vec::new();
    for (line_offset, raw_line) in block.split_inclusive('\n').scan(0usize, |offset, line| {
        let current = *offset;
        *offset += line.len();
        Some((current, line))
    }) {
        let trimmed_start = raw_line.len() - raw_line.trim_start().len();
        let line = raw_line.trim().trim_end_matches('.');
        let raw_key = line
            .split_once(':')
            .map(|(raw_key, _)| raw_key)
            .unwrap_or(line);
        let key_start_in_line = trimmed_start + raw_key.find(raw_key.trim()).unwrap_or(0);
        let key_text = raw_key.trim();
        if key_text == key {
            let span_start = start + line_offset + key_start_in_line;
            spans.push((span_start, span_start + key_text.len()));
        }
    }
    spans
}

pub(super) fn find_matching_brace(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in text.char_indices().skip_while(|(idx, _)| *idx < open) {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

pub(super) fn extract_ddn_string_field(block: &str, name: &str) -> Option<String> {
    let name_pos = block.find(name)?;
    let after_name = &block[name_pos + name.len()..];
    let colon = after_name.find(':')?;
    extract_first_quoted_string(&after_name[colon + 1..])
}

pub(super) fn parse_string_pairs_from_named_block(
    project_block: &str,
    name: &str,
) -> Result<BTreeMap<String, String>, LocalProjectError> {
    let Some(block) = extract_named_block(project_block, name) else {
        return Ok(BTreeMap::new());
    };
    let mut out = BTreeMap::new();
    for raw_line in block.lines() {
        let line = raw_line.trim().trim_end_matches('.');
        if line.is_empty() {
            continue;
        }
        let Some((key, value_part)) = line.split_once(':') else {
            return Err(LocalProjectError::new(
                "E_PROJECT_BLOCK_UNSUPPORTED",
                format!("{name} 블록의 항목은 key: \"value\" 형식이어야 합니다"),
            ));
        };
        let Some(value) = extract_first_quoted_string(value_part) else {
            return Err(LocalProjectError::new(
                "E_PROJECT_BLOCK_UNSUPPORTED",
                format!("{name} 블록의 값은 문자열이어야 합니다"),
            ));
        };
        let key = key.trim().to_string();
        if out.insert(key.clone(), value).is_some() {
            return Err(LocalProjectError::new(
                "E_PROJECT_BLOCK_DUPLICATE_KEY",
                format!("{name} 블록에 중복 키가 있습니다: {key}"),
            ));
        }
    }
    Ok(out)
}

pub(super) fn parse_string_pair_list_from_named_block(
    project_block: &str,
    name: &str,
) -> Result<Vec<(String, String)>, LocalProjectError> {
    let Some(block) = extract_named_block(project_block, name) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for raw_line in block.lines() {
        let line = raw_line.trim().trim_end_matches('.');
        if line.is_empty() {
            continue;
        }
        let Some((key, value_part)) = line.split_once(':') else {
            return Err(LocalProjectError::new(
                "E_PROJECT_BLOCK_UNSUPPORTED",
                format!("{name} 블록의 항목은 key: \"value\" 형식이어야 합니다"),
            ));
        };
        let Some(value) = extract_first_quoted_string(value_part) else {
            return Err(LocalProjectError::new(
                "E_PROJECT_BLOCK_UNSUPPORTED",
                format!("{name} 블록의 값은 문자열이어야 합니다"),
            ));
        };
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(LocalProjectError::new(
                "E_PROJECT_BLOCK_UNSUPPORTED",
                format!("{name} 블록의 key는 비어 있을 수 없습니다"),
            ));
        }
        out.push((key, value));
    }
    Ok(out)
}

pub(super) fn load_gaji_manifest_for_path(
    path: &str,
    text_by_path: &BTreeMap<String, String>,
) -> Result<ParsedGajiManifest, LocalProjectError> {
    let id_from_path = gaji_id_from_vendor_path(path)?.to_string();
    let manifest_path = join_project_path(path, "gaji.toml");
    let text = text_by_path.get(&manifest_path).ok_or_else(|| {
        LocalProjectError::with_path(
            "E_GAJI_MANIFEST_MISSING",
            manifest_path.clone(),
            "vendored 가지의 gaji.toml이 없습니다",
        )
    })?;
    let mut manifest = parse_gaji_manifest_text(&manifest_path, text)?;
    if manifest.id != id_from_path {
        return Err(LocalProjectError::with_path(
            "E_GAJI_ID_PATH_MISMATCH",
            manifest_path,
            format!(
                "gaji id({})와 vendor path({id_from_path})가 다릅니다",
                manifest.id
            ),
        ));
    }
    manifest.package_sha256 = gaji_package_sha256(path, text_by_path);
    Ok(manifest)
}

