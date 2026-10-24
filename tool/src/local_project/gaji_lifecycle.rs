//! 가지 추출·설치·곳간의 기존 구현. 공개 경로와 자료형 정체는 부모가 유지한다.
// F5: 현재 출하 Native 기본/WASM 프로필은 beta-project-features를 켜지 않으며,
// lifecycle 입구의 require_beta_project_feature가 graph 접근 전에 거부한다.
// 해당 feature를 켠 별도 빌드까지 안전하다는 뜻은 아니다.
// artifact manifest의 gaji.toml find는 같은 불변 files의 존재 검사를 먼저 통과한다.
use super::*;

pub fn build_gaji_extraction_plan_json(input_json: &str) -> Result<String, LocalProjectError> {
    let request: GajiExtractionPlanRequest = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_GAJI_EXTRACTION_REQUEST_JSON",
            format!("가지 추출 요청 JSON이 올바르지 않습니다: {err}"),
        )
    })?;
    let plan = build_gaji_extraction_plan(request)?;
    Ok(to_canonical_json_line(&plan))
}

pub fn build_local_gaji_artifact_json(input_json: &str) -> Result<String, LocalProjectError> {
    let request: GajiExtractionPlanRequest = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_GAJI_ARTIFACT_REQUEST_JSON",
            format!("가지 artifact 요청 JSON이 올바르지 않습니다: {err}"),
        )
    })?;
    let artifact = build_local_gaji_artifact(request)?;
    Ok(to_canonical_json_line(&artifact))
}

pub fn verify_local_gaji_artifact_json(input_json: &str) -> Result<String, LocalProjectError> {
    let artifact: LocalGajiArtifact = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_GAJI_ARTIFACT_JSON",
            format!("가지 artifact JSON을 읽을 수 없습니다: {err}"),
        )
    })?;
    verify_local_gaji_artifact(&artifact)?;
    Ok(to_canonical_json_line(&json!({
        "schema": "ddn.local_gaji_artifact_verification.v1",
        "status": "PASS",
        "gaji_id": artifact.gaji_id,
        "version": artifact.version,
        "content_sha256": artifact.content_sha256,
        "package_sha256": artifact.package_sha256,
    })))
}

pub fn publish_local_gotgan_json(input_json: &str) -> Result<String, LocalProjectError> {
    let request: LocalGotganPublishRequest = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_GOTGAN_PUBLISH_JSON",
            format!("내 가지곳간 발행 요청을 읽을 수 없습니다: {err}"),
        )
    })?;
    if request.schema != "ddn.local_gotgan_publish_request.v1" {
        return Err(LocalProjectError::new(
            "E_GOTGAN_PUBLISH_SCHEMA",
            "지원하지 않는 가지곳간 발행 schema입니다",
        ));
    }
    let store = publish_local_gotgan(request.store, request.artifact)?;
    Ok(to_canonical_json_line(&store))
}

pub fn search_local_gotgan_json(input_json: &str) -> Result<String, LocalProjectError> {
    let request: LocalGotganSearchRequest = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_GOTGAN_SEARCH_JSON",
            format!("가지 선반 검색 요청을 읽을 수 없습니다: {err}"),
        )
    })?;
    if request.schema != "ddn.local_gotgan_search_request.v1" {
        return Err(LocalProjectError::new(
            "E_GOTGAN_SEARCH_SCHEMA",
            "지원하지 않는 가지 선반 검색 schema입니다",
        ));
    }
    let rows = search_local_gotgan(&request.store, &request.query, &request.profiles)?;
    Ok(to_canonical_json_line(&json!({
        "schema": "ddn.local_gotgan_search_result.v1",
        "query": request.query,
        "profiles": request.profiles,
        "rows": rows,
    })))
}

pub fn build_gaji_lifecycle_plan_json(input_json: &str) -> Result<String, LocalProjectError> {
    let request: GajiLifecyclePlanRequest = serde_json::from_str(input_json).map_err(|err| {
        LocalProjectError::new(
            "E_GAJI_LIFECYCLE_JSON",
            format!("가지 lifecycle 요청을 읽을 수 없습니다: {err}"),
        )
    })?;
    let plan = build_gaji_lifecycle_plan(request)?;
    Ok(to_canonical_json_line(&plan))
}

pub fn build_gaji_lifecycle_plan(
    request: GajiLifecyclePlanRequest,
) -> Result<GajiLifecyclePlan, LocalProjectError> {
    require_beta_project_feature("E_ALPHA_GAJI_LIFECYCLE_UNSUPPORTED", "가지 설치·제거·갱신은 알파에서 지원하지 않습니다. 베타에서 지원할 예정입니다")?;
    if request.schema != GAJI_LIFECYCLE_PLAN_SCHEMA {
        return Err(LocalProjectError::new(
            "E_GAJI_LIFECYCLE_SCHEMA",
            "지원하지 않는 가지 lifecycle schema입니다",
        ));
    }
    let before_source_sha256 = virtual_project_source_sha256(&request.source)?;
    if before_source_sha256 != request.expected_source_sha256 {
        return Err(LocalProjectError::new(
            "E_GAJI_LIFECYCLE_SOURCE_STALE",
            "현재 project source hash가 계획 기준과 다릅니다",
        ));
    }
    let before_graph = build_project_graph(request.source.clone())?;
    let embedded_lock_sha256 = request
        .source
        .files
        .iter()
        .find(|file| file.path == "ddn.lock")
        .and_then(|file| serde_json::from_str::<serde_json::Value>(&file.utf8_text).ok())
        .and_then(|value| {
            value
                .get("lock_sha256")
                .and_then(|item| item.as_str())
                .map(str::to_string)
        });
    let mut unlocked_source = request.source.clone();
    unlocked_source.files.retain(|file| file.path != "ddn.lock");
    let before_lock = if before_graph.gaji_aliases.is_empty() {
        None
    } else {
        Some(build_project_gaji_lock(unlocked_source, false)?)
    };
    let before_lock_sha256 = before_lock.as_ref().map(|lock| lock.lock_sha256.clone());
    if request.expected_lock_sha256.is_some()
        && request.expected_lock_sha256 != embedded_lock_sha256
    {
        return Err(LocalProjectError::new(
            "E_GAJI_LIFECYCLE_LOCK_STALE",
            "현재 ddn.lock hash가 계획 기준과 다릅니다",
        ));
    }
    let mut source = request.source;
    source.files.retain(|file| file.path != "ddn.lock");
    let (operation, alias, affected_files, version_diff) = match request.operation {
        GajiLifecycleOperation::Install { alias, artifact } => {
            if before_graph.gaji_aliases.contains_key(&alias) {
                return Err(LocalProjectError::new(
                    "E_GAJI_ALIAS_EXISTS",
                    format!("가지 alias {alias}가 이미 설치되어 있습니다"),
                ));
            }
            let affected = install_artifact_into_source(&mut source, &alias, &artifact, false)?;
            ("install", alias, affected, None)
        }
        GajiLifecycleOperation::Update { alias, artifact } => {
            if !before_graph.gaji_aliases.contains_key(&alias) {
                return Err(LocalProjectError::new(
                    "E_GAJI_ALIAS_MISSING",
                    format!("갱신할 가지 alias {alias}가 없습니다"),
                ));
            }
            let old_path = before_graph.gaji_aliases.get(&alias).expect("alias");
            let old_id = gaji_id_from_vendor_path(old_path)?.to_string();
            let old_artifact = installed_artifact_from_source(&source, &old_id)?;
            let version_diff = diff_gaji_artifacts(Some(&old_artifact), Some(&artifact));
            let root_text = source
                .files
                .iter()
                .find(|file| file.path == source.root_path)
                .map(|file| file.utf8_text.clone());
            source
                .files
                .retain(|file| !file.path.starts_with(&format!("vendor/{old_id}/")));
            let affected = install_artifact_into_source(&mut source, &alias, &artifact, true)?;
            if old_id == artifact.gaji_id {
                if let (Some(original), Some(root)) = (
                    root_text,
                    source
                        .files
                        .iter_mut()
                        .find(|file| file.path == source.root_path),
                ) {
                    root.utf8_text = original;
                }
            }
            ("update", alias, affected, Some(version_diff))
        }
        GajiLifecycleOperation::Remove { alias } => {
            let old_path = before_graph.gaji_aliases.get(&alias).ok_or_else(|| {
                LocalProjectError::new(
                    "E_GAJI_ALIAS_MISSING",
                    format!("제거할 가지 alias {alias}가 없습니다"),
                )
            })?;
            let old_id = gaji_id_from_vendor_path(old_path)?.to_string();
            for file in &source.files {
                if file.path.starts_with("vendor/") || file.path == source.root_path {
                    continue;
                }
                if parse_imports(&file.utf8_text, &file.path)?
                    .iter()
                    .any(|item| {
                        item.alias == alias
                            || resolve_import_source_path(&file.path, &item.source)
                                .is_ok_and(|path| path.starts_with(&format!("vendor/{old_id}/")))
                    })
                {
                    return Err(LocalProjectError::with_path(
                        "E_GAJI_REMOVE_REFERENCED",
                        &file.path,
                        format!("쓰임 alias {alias}를 참조 중이므로 제거할 수 없습니다"),
                    ));
                }
            }
            let old_artifact = installed_artifact_from_source(&source, &old_id)?;
            source
                .files
                .retain(|file| !file.path.starts_with(&format!("vendor/{old_id}/")));
            set_project_gaji_alias(&mut source, &alias, None)?;
            (
                "remove",
                alias,
                vec![format!("vendor/{old_id}")],
                Some(diff_gaji_artifacts(Some(&old_artifact), None)),
            )
        }
    };
    let after_graph = build_project_graph(source.clone())?;
    let after_lock = if after_graph.gaji_aliases.is_empty() {
        None
    } else {
        Some(build_project_gaji_lock(source.clone(), false)?)
    };
    if let Some(lock) = &after_lock {
        source.files.push(VirtualProjectFileSource {
            path: "ddn.lock".to_string(),
            role: "lock".to_string(),
            utf8_text: to_canonical_json_line(lock),
        });
        source.files.sort_by(|a, b| a.path.cmp(&b.path));
    }
    normalize_virtual_project_source(source.clone())?;
    let after_source_sha256 = virtual_project_source_sha256(&source)?;
    Ok(GajiLifecyclePlan {
        schema: GAJI_LIFECYCLE_PLAN_SCHEMA,
        operation,
        alias,
        before_source_sha256,
        after_source_sha256,
        before_lock_sha256,
        after_lock_sha256: after_lock.map(|lock| lock.lock_sha256),
        affected_files,
        version_diff,
        diagnostics: Vec::new(),
        project_source: source,
        atomic_apply: true,
    })
}

pub(super) fn install_artifact_into_source(
    source: &mut VirtualProjectSource,
    alias: &str,
    artifact: &LocalGajiArtifact,
    replace_alias: bool,
) -> Result<Vec<String>, LocalProjectError> {
    verify_local_gaji_artifact(artifact)?;
    let prefix = format!("vendor/{}", artifact.gaji_id);
    if source
        .files
        .iter()
        .any(|file| file.path == prefix || file.path.starts_with(&format!("{prefix}/")))
    {
        return Err(LocalProjectError::new(
            "E_GAJI_VENDOR_CONFLICT",
            "같은 가지 vendor 경로가 이미 존재합니다",
        ));
    }
    let mut affected = Vec::new();
    let metadata_path = format!("{prefix}/artifact.detjson");
    affected.push(metadata_path.clone());
    source.files.push(VirtualProjectFileSource {
        path: metadata_path,
        role: "metadata".to_string(),
        utf8_text: to_canonical_json_line(artifact),
    });
    for file in &artifact.files {
        if file.path == "artifact.detjson" {
            continue;
        }
        let path = format!("{prefix}/{}", file.path);
        affected.push(path.clone());
        source.files.push(VirtualProjectFileSource {
            path,
            role: if file.path == "gaji.toml" {
                "gaji_manifest".to_string()
            } else {
                "ddn_source".to_string()
            },
            utf8_text: file.utf8_text.clone(),
        });
    }
    if artifact.provenance.extraction_modules.is_empty() {
        return Err(LocalProjectError::new(
            "E_GAJI_ENTRY_MODULE_MISSING",
            "가지 계보에 추출 모듈이 없습니다",
        ));
    }
    set_project_gaji_alias(source, alias, Some(&prefix))?;
    if replace_alias {
        affected.push(format!("@alias:{alias}"));
    }
    Ok(affected)
}

pub(super) fn installed_artifact_from_source(
    source: &VirtualProjectSource,
    id: &str,
) -> Result<LocalGajiArtifact, LocalProjectError> {
    let path = format!("vendor/{id}/artifact.detjson");
    let file = source
        .files
        .iter()
        .find(|file| file.path == path)
        .ok_or_else(|| {
            LocalProjectError::with_path(
                "E_GAJI_INSTALLED_METADATA_MISSING",
                &path,
                "갱신·되돌리기에 필요한 설치 artifact metadata가 없습니다",
            )
        })?;
    let artifact: LocalGajiArtifact = serde_json::from_str(&file.utf8_text).map_err(|err| {
        LocalProjectError::with_path(
            "E_GAJI_INSTALLED_METADATA_INVALID",
            &path,
            format!("설치 artifact metadata가 올바르지 않습니다: {err}"),
        )
    })?;
    verify_local_gaji_artifact(&artifact)?;
    Ok(artifact)
}

pub(super) fn diff_gaji_artifacts(
    before: Option<&LocalGajiArtifact>,
    after: Option<&LocalGajiArtifact>,
) -> GajiVersionDiff {
    let before_files = before
        .map(|item| {
            item.files
                .iter()
                .map(|file| (file.path.clone(), file.sha256.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let after_files = after
        .map(|item| {
            item.files
                .iter()
                .map(|file| (file.path.clone(), file.sha256.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let before_symbols = before
        .map(|item| {
            item.public_symbols
                .iter()
                .map(|symbol| format!("{}:{}", symbol.module_path, symbol.external_name))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let after_symbols = after
        .map(|item| {
            item.public_symbols
                .iter()
                .map(|symbol| format!("{}:{}", symbol.module_path, symbol.external_name))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let added_files = after_files
        .keys()
        .filter(|path| !before_files.contains_key(*path))
        .cloned()
        .collect();
    let removed_files = before_files
        .keys()
        .filter(|path| !after_files.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let changed_files = after_files
        .iter()
        .filter(|(path, hash)| before_files.get(*path).is_some_and(|old| old != *hash))
        .map(|(path, _)| path.clone())
        .collect();
    let added_public_symbols = after_symbols.difference(&before_symbols).cloned().collect();
    let removed_public_symbols = before_symbols
        .difference(&after_symbols)
        .cloned()
        .collect::<Vec<_>>();
    let dependency_changed =
        before.map(|item| &item.dependencies) != after.map(|item| &item.dependencies);
    let risky =
        !removed_files.is_empty() || !removed_public_symbols.is_empty() || dependency_changed;
    GajiVersionDiff {
        from_version: before.map(|item| item.version.clone()),
        to_version: after.map(|item| item.version.clone()),
        added_files,
        removed_files,
        changed_files,
        added_public_symbols,
        removed_public_symbols,
        dependency_changed,
        risky,
    }
}

pub(super) fn set_project_gaji_alias(
    source: &mut VirtualProjectSource,
    alias: &str,
    path: Option<&str>,
) -> Result<(), LocalProjectError> {
    if alias.trim().is_empty() || alias.contains('.') {
        return Err(LocalProjectError::new(
            "E_GAJI_ALIAS_INVALID",
            "가지 alias가 올바르지 않습니다",
        ));
    }
    let root = source
        .files
        .iter_mut()
        .find(|file| file.path == source.root_path)
        .ok_or_else(|| LocalProjectError::new("E_PROJECT_ROOT_MISSING", "뿌리.ddn이 없습니다"))?;
    let mut pairs = extract_project_block(&root.utf8_text)
        .map(|block| parse_string_pairs_from_named_block(block, "가지"))
        .transpose()?
        .unwrap_or_default();
    match path {
        Some(value) => {
            pairs.insert(alias.to_string(), value.to_string());
        }
        None => {
            pairs.remove(alias);
        }
    }
    let body = pairs
        .iter()
        .map(|(key, value)| format!("\n  {key}: \"./{value}\"."))
        .collect::<String>()
        + "\n ";
    if let Some((start, end)) = extract_named_block_range(&root.utf8_text, "가지") {
        root.utf8_text.replace_range(start..end, &body);
    } else if let Some(project_pos) = root.utf8_text.find("프로젝트") {
        let open = project_pos
            + root.utf8_text[project_pos..].find('{').ok_or_else(|| {
                LocalProjectError::new("E_PROJECT_BLOCK_MISSING", "프로젝트 블록이 없습니다")
            })?;
        let close = find_matching_brace(&root.utf8_text, open).ok_or_else(|| {
            LocalProjectError::new(
                "E_PROJECT_BLOCK_MISSING",
                "프로젝트 블록이 닫히지 않았습니다",
            )
        })?;
        root.utf8_text
            .insert_str(close, &format!(" 가지 {{{body} }}"));
    }
    Ok(())
}

pub fn publish_local_gotgan(
    mut store: LocalGotganStore,
    artifact: LocalGajiArtifact,
) -> Result<LocalGotganStore, LocalProjectError> {
    if store.schema != LOCAL_GOTGAN_STORE_SCHEMA {
        return Err(LocalProjectError::new(
            "E_GOTGAN_STORE_SCHEMA",
            "지원하지 않는 local gotgan schema입니다",
        ));
    }
    verify_local_gaji_artifact(&artifact)?;
    for current in &store.artifacts {
        verify_local_gaji_artifact(current)?;
        if current.profile == artifact.profile
            && current.gaji_id == artifact.gaji_id
            && current.version == artifact.version
        {
            if current.package_sha256 == artifact.package_sha256 {
                return Ok(store);
            }
            return Err(LocalProjectError::new(
                "E_GOTGAN_VERSION_IMMUTABLE",
                "같은 profile/id/version에 다른 bytes를 덮어쓸 수 없습니다",
            ));
        }
    }
    store.artifacts.push(artifact);
    store.artifacts.sort_by(|a, b| {
        (&a.profile, &a.gaji_id, &a.version, &a.package_sha256).cmp(&(
            &b.profile,
            &b.gaji_id,
            &b.version,
            &b.package_sha256,
        ))
    });
    Ok(store)
}

pub fn search_local_gotgan(
    store: &LocalGotganStore,
    query: &str,
    profiles: &[String],
) -> Result<Vec<LocalGotganSearchRow>, LocalProjectError> {
    if store.schema != LOCAL_GOTGAN_STORE_SCHEMA {
        return Err(LocalProjectError::new(
            "E_GOTGAN_STORE_SCHEMA",
            "지원하지 않는 local gotgan schema입니다",
        ));
    }
    let needle = query.trim().to_lowercase();
    let profile_set = profiles.iter().cloned().collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for artifact in &store.artifacts {
        verify_local_gaji_artifact(artifact)?;
        if !profile_set.is_empty() && !profile_set.contains(&artifact.profile) {
            continue;
        }
        let symbols = artifact
            .public_symbols
            .iter()
            .map(|item| item.external_name.clone())
            .collect::<Vec<_>>();
        let haystack = format!(
            "{} {} {} {}",
            artifact.gaji_id,
            artifact.description,
            artifact.tags.join(" "),
            symbols.join(" ")
        )
        .to_lowercase();
        if !needle.is_empty() && !haystack.contains(&needle) {
            continue;
        }
        rows.push(LocalGotganSearchRow {
            profile: artifact.profile.clone(),
            gaji_id: artifact.gaji_id.clone(),
            version: artifact.version.clone(),
            description: artifact.description.clone(),
            tags: artifact.tags.clone(),
            public_symbols: symbols,
            content_sha256: artifact.content_sha256.clone(),
            package_sha256: artifact.package_sha256.clone(),
            origin_project_ref: artifact.provenance.origin_project_ref.clone(),
            relation: artifact.provenance.relation.clone(),
        });
    }
    rows.sort_by(|a, b| {
        (&a.gaji_id, &a.profile, &a.version).cmp(&(&b.gaji_id, &b.profile, &b.version))
    });
    Ok(rows)
}

pub fn build_local_gaji_artifact(
    request: GajiExtractionPlanRequest,
) -> Result<LocalGajiArtifact, LocalProjectError> {
    if !matches!(request.relation.as_str(), "extracted" | "remix") {
        return Err(LocalProjectError::new(
            "E_GAJI_LINEAGE_RELATION",
            "가지 relation은 extracted 또는 remix여야 합니다",
        ));
    }
    if request.relation == "remix" && request.origin_gaji.as_deref().is_none_or(str::is_empty) {
        return Err(LocalProjectError::new(
            "E_GAJI_REMIX_ORIGIN_REQUIRED",
            "리믹스에는 원본 가지 id@version 계보가 필요합니다",
        ));
    }
    let source = request.source.clone();
    let description = request.description.clone();
    let tags = request.tags.clone();
    let origin_gaji = request.origin_gaji.clone();
    let relation = request.relation.clone();
    let plan = build_gaji_extraction_plan(request)?;
    let text_by_path = source
        .files
        .into_iter()
        .map(|file| (file.path, file.utf8_text))
        .collect::<BTreeMap<_, _>>();
    let mut manifest = format!(
        "id = \"{}\"\nversion = \"{}\"\ncontent_sha256 = \"{}\"\n",
        plan.gaji_id, plan.version, plan.content_sha256
    );
    manifest.push_str("\n[requires]\ndet_tier = \"D-STRICT\"\nopenness = \"closed\"\n");
    let mut files = vec![local_gaji_artifact_file("gaji.toml", manifest)];
    for path in &plan.included_modules {
        let text = text_by_path.get(path).ok_or_else(|| {
            LocalProjectError::with_path(
                "E_GAJI_ARTIFACT_SOURCE_MISSING",
                path,
                "추출 source 파일이 없습니다",
            )
        })?;
        files.push(local_gaji_artifact_file(
            &format!("ddn/{path}"),
            text.clone(),
        ));
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let package_sha256 = local_gaji_artifact_package_hash(&files);
    let artifact = LocalGajiArtifact {
        schema: LOCAL_GAJI_ARTIFACT_SCHEMA.to_string(),
        gaji_id: plan.gaji_id,
        version: plan.version,
        profile: plan.profile,
        description,
        tags,
        manifest_path: "gaji.toml".to_string(),
        content_sha256: plan.content_sha256,
        package_sha256,
        public_symbols: plan.public_symbols,
        dependencies: plan.external_gaji_dependencies,
        provenance: LocalGajiArtifactProvenance {
            origin_project_ref: plan.origin_project_ref,
            origin_gaji,
            relation,
            extraction_modules: plan.selected_modules,
        },
        files,
    };
    verify_local_gaji_artifact(&artifact)?;
    Ok(artifact)
}

pub fn verify_local_gaji_artifact(artifact: &LocalGajiArtifact) -> Result<(), LocalProjectError> {
    if artifact.schema != LOCAL_GAJI_ARTIFACT_SCHEMA {
        return Err(LocalProjectError::new(
            "E_GAJI_ARTIFACT_SCHEMA",
            "지원하지 않는 가지 artifact schema입니다",
        ));
    }
    if artifact.manifest_path != "gaji.toml"
        || artifact.files.iter().all(|file| file.path != "gaji.toml")
    {
        return Err(LocalProjectError::new(
            "E_GAJI_MANIFEST_MISSING",
            "가지 artifact에 gaji.toml이 필요합니다",
        ));
    }
    if artifact
        .files
        .iter()
        .all(|file| file.path != "ddn" && !file.path.starts_with("ddn/"))
    {
        return Err(LocalProjectError::new(
            "E_GAJI_DDN_MISSING",
            "가지 artifact에 ddn/ source가 필요합니다",
        ));
    }
    if artifact.files.len() > GAJI_ARTIFACT_MAX_FILES {
        return Err(LocalProjectError::new(
            "E_GAJI_ARTIFACT_FILE_COUNT_BOUND",
            format!("가지 파일은 최대 {GAJI_ARTIFACT_MAX_FILES}개입니다"),
        ));
    }
    let total_bytes = artifact
        .files
        .iter()
        .try_fold(0usize, |total, file| total.checked_add(file.byte_length))
        .ok_or_else(|| {
            LocalProjectError::new(
                "E_GAJI_ARTIFACT_TOTAL_BOUND",
                "가지 전체 byte 합이 범위를 넘었습니다",
            )
        })?;
    if total_bytes > GAJI_ARTIFACT_MAX_TOTAL_BYTES {
        return Err(LocalProjectError::new(
            "E_GAJI_ARTIFACT_TOTAL_BOUND",
            format!("가지 전체 크기는 최대 {GAJI_ARTIFACT_MAX_TOTAL_BYTES} bytes입니다"),
        ));
    }
    let mut seen = BTreeSet::new();
    let mut folded_seen = BTreeSet::new();
    for file in &artifact.files {
        let path = normalize_project_path(&file.path)?;
        if path != file.path
            || !seen.insert(path.clone())
            || !folded_seen.insert(path.to_lowercase())
        {
            return Err(LocalProjectError::with_path(
                "E_GAJI_ARTIFACT_PATH_DUPLICATE",
                path,
                "정규화 뒤 중복 artifact 경로입니다",
            ));
        }
        if file.byte_length > GAJI_ARTIFACT_MAX_SINGLE_FILE_BYTES {
            return Err(LocalProjectError::with_path(
                "E_GAJI_ARTIFACT_FILE_BOUND",
                path,
                format!("가지 단일 파일은 최대 {GAJI_ARTIFACT_MAX_SINGLE_FILE_BYTES} bytes입니다"),
            ));
        }
        if file.path == "gaji.toml" && file.byte_length > GAJI_ARTIFACT_MAX_MANIFEST_BYTES {
            return Err(LocalProjectError::with_path(
                "E_GAJI_MANIFEST_BOUND",
                path,
                format!("gaji.toml은 최대 {GAJI_ARTIFACT_MAX_MANIFEST_BYTES} bytes입니다"),
            ));
        }
        if file.byte_length != file.utf8_text.len()
            || file.sha256 != sha256_hex(file.utf8_text.as_bytes())
        {
            return Err(LocalProjectError::with_path(
                "E_GAJI_ARTIFACT_FILE_HASH",
                path,
                "가지 파일 길이 또는 hash가 일치하지 않습니다",
            ));
        }
    }
    if artifact.package_sha256 != local_gaji_artifact_package_hash(&artifact.files) {
        return Err(LocalProjectError::new(
            "E_GAJI_ARTIFACT_PACKAGE_HASH",
            "가지 package hash가 일치하지 않습니다",
        ));
    }
    let manifest = artifact
        .files
        .iter()
        .find(|file| file.path == "gaji.toml")
        .expect("checked manifest");
    let expected_content_row = format!("content_sha256 = \"{}\"", artifact.content_sha256);
    if !manifest
        .utf8_text
        .lines()
        .any(|line| line.trim() == expected_content_row)
    {
        return Err(LocalProjectError::new(
            "E_GAJI_MANIFEST_CONTENT_HASH",
            "manifest content hash가 artifact와 일치하지 않습니다",
        ));
    }
    Ok(())
}

pub(super) fn local_gaji_artifact_file(path: &str, utf8_text: String) -> LocalGajiArtifactFile {
    LocalGajiArtifactFile {
        path: path.to_string(),
        byte_length: utf8_text.len(),
        sha256: sha256_hex(utf8_text.as_bytes()),
        utf8_text,
    }
}

pub(super) fn local_gaji_artifact_package_hash(files: &[LocalGajiArtifactFile]) -> String {
    let mut rows = files
        .iter()
        .map(|file| (&file.path, &file.sha256, file.byte_length))
        .collect::<Vec<_>>();
    rows.sort();
    sha256_hex(
        serde_json::to_string(&rows)
            .expect("artifact package hash rows")
            .as_bytes(),
    )
}

pub fn build_gaji_extraction_plan(
    request: GajiExtractionPlanRequest,
) -> Result<GajiExtractionPlan, LocalProjectError> {
    if request.schema != GAJI_EXTRACTION_PLAN_SCHEMA {
        return Err(LocalProjectError::new(
            "E_GAJI_EXTRACTION_SCHEMA",
            format!("schema는 {GAJI_EXTRACTION_PLAN_SCHEMA} 이어야 합니다"),
        ));
    }
    if request.gaji_id.trim().is_empty()
        || request.gaji_id.contains(char::is_whitespace)
        || request.gaji_id.contains('/')
        || request.gaji_id.contains('\\')
    {
        return Err(LocalProjectError::new(
            "E_GAJI_ID_INVALID",
            "가지 id는 비어 있거나 공백·경로 구분자를 포함할 수 없습니다",
        ));
    }
    if request.version.trim().is_empty() {
        return Err(LocalProjectError::new(
            "E_GAJI_VERSION_INVALID",
            "가지 version이 필요합니다",
        ));
    }
    if !matches!(request.profile.as_str(), "표준" | "나눔" | "내") {
        return Err(LocalProjectError::new(
            "E_GOTGAN_PROFILE_INVALID",
            "profile은 표준/나눔/내 중 하나여야 합니다",
        ));
    }
    if request.selected_modules.is_empty() {
        return Err(LocalProjectError::new(
            "E_GAJI_MODULE_SELECTION_EMPTY",
            "가지로 묶을 모듈을 하나 이상 선택하세요",
        ));
    }
    let graph = build_project_graph(request.source.clone())?;
    let source_project_sha256 = virtual_project_source_sha256(&request.source)?;
    let text_by_path = request
        .source
        .files
        .iter()
        .map(|file| Ok((normalize_project_path(&file.path)?, file.utf8_text.clone())))
        .collect::<Result<BTreeMap<_, _>, LocalProjectError>>()?;
    let mut selected_modules = request
        .selected_modules
        .iter()
        .map(|path| normalize_project_reference(path))
        .collect::<Result<Vec<_>, _>>()?;
    selected_modules.sort();
    selected_modules.dedup();
    let mut pending = Vec::new();
    for path in &selected_modules {
        if path == &graph.root_path || path == &graph.entry_path {
            return Err(LocalProjectError::with_path(
                "E_GAJI_MODULE_SELECTION_INVALID",
                path,
                "뿌리 또는 진입 파일은 재사용 모듈 추출 대상으로 선택할 수 없습니다",
            ));
        }
        let module = graph.module_sources.iter().find(|(_, paths)| paths.contains(path))
            .map(|(name, _)| name.clone()).ok_or_else(|| LocalProjectError::with_path(
                "E_GAJI_MODULE_SELECTION_INVALID", path, "선언된 모듈 원문을 선택해 주세요"))?;
        pending.push(module);
    }
    let mut modules = BTreeMap::new();
    while let Some(name) = pending.pop() {
        if modules.contains_key(&name) { continue; }
        let head = &graph.module_paths[&name];
        let directives = resolve_work_project_module_directives(&request.source, head)?;
        pending.extend(directives.imports.iter().map(|import| import.source.clone()));
        modules.insert(name, directives);
    }
    let mut remaining = modules.keys().cloned().collect::<BTreeSet<_>>();
    while !remaining.is_empty() {
        let ready = remaining.iter().filter(|name|
            modules[*name].imports.iter().all(|import| !remaining.contains(&import.source)))
            .cloned().collect::<Vec<_>>();
        if ready.is_empty() {
            return Err(LocalProjectError::new("E_IMPORT_CYCLE", "추출할 모듈에 순환 쓰임이 있습니다"));
        }
        for name in ready { remaining.remove(&name); }
    }
    let included_modules = modules.keys()
        .flat_map(|name| graph.module_sources[name].iter())
        .filter(|path| !path.starts_with("vendor/"))
        .cloned()
        .collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    let mut external_gaji_dependencies = BTreeMap::new();
    for (alias, vendor_path) in &graph.gaji_aliases {
        if included_modules.iter()
            .any(|path| path == vendor_path || path.starts_with(&format!("{vendor_path}/")))
        {
            external_gaji_dependencies.insert(alias.clone(), vendor_path.clone());
        }
    }
    let mut public_symbols = Vec::new();
    for (name, info) in &modules {
        let path = &graph.module_paths[name];
        if path.starts_with("vendor/") { continue; }
        for export in &info.exports {
            public_symbols.push(GajiPublicSymbol {
                module_path: path.clone(),
                external_name: export.external_name.clone(),
                internal_name: export.internal_name.clone(),
            });
        }
    }
    public_symbols.sort_by(|a, b| {
        (&a.module_path, &a.external_name, &a.internal_name).cmp(&(
            &b.module_path,
            &b.external_name,
            &b.internal_name,
        ))
    });
    if public_symbols.is_empty() {
        return Err(LocalProjectError::new(
            "E_GAJI_PUBLIC_API_EMPTY",
            "선택한 모듈 dependency closure에 드러냄 공개 심볼이 없습니다",
        ));
    }
    let selected_set = selected_modules.iter().cloned().collect::<BTreeSet<_>>();
    let mut files = Vec::new();
    for path in &included_modules {
        let text = text_by_path.get(path).expect("included source text");
        files.push(GajiExtractionFile {
            path: path.clone(),
            byte_length: text.len(),
            sha256: sha256_hex(text.as_bytes()),
            selected: selected_set.contains(path),
        });
    }
    let included_set = included_modules.iter().cloned().collect::<BTreeSet<_>>();
    let excluded_files = text_by_path
        .keys()
        .filter(|path| !included_set.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    let content_sha256 = sha256_hex(
        serde_json::to_string(&json!({
            "schema": "ddn.gaji_extraction_content_hash_input.v1",
            "gaji_id": request.gaji_id,
            "version": request.version,
            "files": files,
            "public_symbols": public_symbols,
            "external_gaji_dependencies": external_gaji_dependencies,
        }))
        .expect("gaji extraction hash input")
        .as_bytes(),
    );
    Ok(GajiExtractionPlan {
        schema: GAJI_EXTRACTION_PLAN_SCHEMA,
        gaji_id: request.gaji_id,
        version: request.version,
        profile: request.profile,
        origin_project_ref: request.origin_project_ref,
        source_project_sha256,
        selected_modules,
        included_modules,
        excluded_files,
        public_symbols,
        external_gaji_dependencies,
        files,
        content_sha256,
        diagnostics: Vec::new(),
        publishable: true,
    })
}

