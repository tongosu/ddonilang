//! 작품 원문의 기존 편집 계획. 처리 본문·공개 호출 경로는 그대로 유지한다.
// F5: rename/delete는 요청 경로와 정확히 같은 raw files 항목의 존재를 먼저
// 검사한다. rename의 path 변경과 delete의 항목 제거는 해당 lookup 뒤에 한다.
// build_set_entry_plan은 현재 dispatcher가 호출하지 않는 보존 후보이며,
// 그 본문을 이 설명만으로 현재 공개 입구로 간주하거나 삭제하지 않는다.
use super::*;

pub fn build_project_edit_plan(
    request: ProjectEditPlanRequest,
) -> Result<ProjectEditPlan, LocalProjectError> {
    let before_hash = virtual_project_source_sha256(&request.source)?;
    if request.expected_source_sha256 != before_hash {
        return Err(LocalProjectError::new(
            "E_PROJECT_EDIT_SOURCE_CONFLICT",
            "현재 project source hash가 계획 요청 시점과 달라 atomic apply를 거부했습니다",
        ));
    }
    match request.operation {
        ProjectEditOperation::RenameFile { from_path, to_path } => {
            build_rename_file_plan(request.source, &before_hash, &from_path, &to_path)
        }
        ProjectEditOperation::AddFile {
            path,
            utf8_text,
            role,
        } => build_add_file_plan(
            request.source,
            &before_hash,
            &path,
            &utf8_text,
            &role,
            "add_file",
        ),
        ProjectEditOperation::AddModule { path, module_name } => {
            build_add_module_plan(request.source, &before_hash, &path, &module_name)
        }
        ProjectEditOperation::DuplicateFile { from_path, to_path } => {
            build_duplicate_file_plan(request.source, &before_hash, &from_path, &to_path)
        }
        ProjectEditOperation::DeleteFile { path } => {
            build_delete_file_plan(request.source, &before_hash, &path)
        }
        ProjectEditOperation::SetEntry { path } => {
            Err(LocalProjectError::with_path("E_PROJECT_START_REFERENCE_REQUIRED", path,
                "시작은 파일 경로가 아니라 (모듈=이름,정의=이름)으로 지정해야 합니다"))
        }
        ProjectEditOperation::BindSealedDataset {
            binding_name,
            dataset_handle,
        } => build_bind_sealed_dataset_plan(
            request.source,
            &before_hash,
            &binding_name,
            &dataset_handle,
        ),
    }
}

pub(super) fn build_bind_sealed_dataset_plan(
    mut source: VirtualProjectSource,
    before_hash: &str,
    binding_name: &str,
    dataset_handle: &str,
) -> Result<ProjectEditPlan, LocalProjectError> {
    let binding_name = binding_name.trim().nfc().collect::<String>();
    require_beta_project_feature("E_ALPHA_DATASET_UNSUPPORTED", "봉인 자료 결속은 알파에서 지원하지 않습니다. 베타에서 지원할 예정입니다")?;
    if binding_name.is_empty() {
        return Err(LocalProjectError::new(
            "E_PROJECT_DATASET_BINDING_NAME_REQUIRED",
            "DDN에서 사용할 자료 이름을 입력해야 합니다",
        ));
    }
    validate_dataset_binding_name(&binding_name)?;
    let handle_text = dataset_handle.trim();
    if handle_text.len() != 16
        || !handle_text
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(LocalProjectError::new(
            "E_PROJECT_DATASET_HANDLE_INVALID",
            "봉인 자료 자원핸들은 소문자 16자리 hex여야 합니다",
        ));
    }
    let handle_raw = u64::from_str_radix(handle_text, 16).map_err(|_| {
        LocalProjectError::new(
            "E_PROJECT_DATASET_HANDLE_INVALID",
            "봉인 자료 자원핸들을 읽을 수 없습니다",
        )
    })?;
    let registry = project_asset_registry_from_source(&source)?.ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_ASSET_REGISTRY_MISSING",
            "ddn.asset.json",
            "봉인 자료를 연결하려면 검증된 project asset registry가 필요합니다",
        )
    })?;
    let envelope_path = registry
        .sealed_dataset_path_for_handle(ResourceHandle::from_raw(handle_raw))
        .map_err(|err| {
            LocalProjectError::with_path(
                "E_PROJECT_DATASET_HANDLE_UNVERIFIED",
                "ddn.asset.json",
                err,
            )
        })?;
    let graph = build_project_graph(source.clone())?;
    let target_path = graph.targets.get("앱").cloned().ok_or_else(|| {
        LocalProjectError::with_path(
            "E_PROJECT_TARGET_MISSING",
            graph.root_path.clone(),
            "봉인 자료를 연결할 프로젝트 목표 앱을 찾을 수 없습니다",
        )
    })?;
    let entry = source
        .files
        .iter_mut()
        .find(|file| file.path == target_path)
        .ok_or_else(|| {
            LocalProjectError::with_path(
                "E_PROJECT_ENTRY_MISSING",
                target_path.clone(),
                "봉인 자료를 연결할 DDN 파일이 프로젝트에 없습니다",
            )
        })?;
    if entry.role != "ddn_source" || !entry.path.ends_with(".ddn") {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ENTRY_ROLE",
            entry.path.clone(),
            "봉인 자료 연결 대상은 DDN source여야 합니다",
        ));
    }
    if entry_declares_name(&entry.utf8_text, &entry.path, &binding_name)? {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_DATASET_BINDING_NAME_CONFLICT",
            entry.path.clone(),
            format!("앱 본문에 {binding_name} 이름이 이미 있습니다"),
        ));
    }
    let exact_resource_call = format!("(@\"{envelope_path}\") 자료.열기");
    if entry.utf8_text.contains(&exact_resource_call) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_DATASET_ALREADY_BOUND",
            entry.path.clone(),
            "같은 봉인 자료가 이미 앱 DDN에 연결되어 있습니다",
        ));
    }
    let target_body = find_top_level_target_body(&entry.utf8_text, "앱")
        .or_else(|| find_canonical_seed_body(&entry.utf8_text, &entry.path));
    let body_start = target_body.map(|span| span.0).unwrap_or(0);
    let newline = if entry.utf8_text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let (insert_at, prefix, indent) = if target_body.is_none() {
        (0, "", "")
    } else if entry.utf8_text[body_start..].starts_with("\r\n") {
        (body_start + 2, "", "  ")
    } else if entry.utf8_text[body_start..].starts_with('\n') {
        (body_start + 1, "", "  ")
    } else {
        (body_start, newline, "  ")
    };
    let statement =
        format!("{prefix}{indent}{binding_name}:묶음 := {exact_resource_call}.{newline}");
    entry.utf8_text.insert_str(insert_at, &statement);
    let modified_path = entry.path.clone();
    let modified_text = entry.utf8_text.clone();
    validate_linked_entry_source(&modified_text, &modified_path, &binding_name)?;
    build_project_prepare_link(source.clone(), "앱", true, false)?;
    let after_hash = virtual_project_source_sha256(&source)?;
    Ok(ProjectEditPlan {
        schema: PROJECT_EDIT_PLAN_SCHEMA,
        operation: "bind_sealed_dataset",
        before_source_sha256: before_hash.to_string(),
        after_source_sha256: after_hash,
        affected_files: vec![modified_path.clone()],
        affected_symbols: vec![binding_name],
        affected_references: vec![format!("{}:{}", modified_path, envelope_path)],
        patches: vec![ProjectEditPatch {
            path: modified_path,
            start: insert_at,
            end: insert_at,
            before: String::new(),
            after: statement,
            reason: "verified_sealed_dataset_binding",
        }],
        diagnostics: Vec::new(),
        project_source: source,
        atomic_apply: true,
        lock_stale: true,
    })
}

pub(super) fn validate_dataset_binding_name(binding_name: &str) -> Result<(), LocalProjectError> {
    let probe = format!("앱:움직씨 := {{\n  {binding_name}:묶음 := 0.\n}}\n");
    let program = ddonirang_lang::parse_v25(&probe, "<dataset-binding-name>").map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_DATASET_BINDING_NAME_INVALID",
            format!("DDN 이름으로 사용할 수 없습니다: {}", err.message),
        )
    })?;
    let valid = program.items.iter().any(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        seed.body.as_ref().is_some_and(|body| {
            body.stmts.iter().any(|stmt| {
                matches!(
                    stmt,
                    Stmt::Definition { name, .. } if name == binding_name
                )
            })
        })
    });
    if !valid {
        return Err(LocalProjectError::new(
            "E_PROJECT_DATASET_BINDING_NAME_INVALID",
            "DDN에서 사용할 수 있는 한 이름이어야 합니다",
        ));
    }
    Ok(())
}

/// The project's root owns its epoch (RD-452/492); source tokens never do.
pub fn project_source_profile(
    source: &VirtualProjectSource,
    caller_profile: Option<&str>,
) -> Result<&'static str, LocalProjectError> {
    let root_path = normalize_project_path(&source.root_path)?;
    let root = source.files.iter().find(|file| {
        normalize_project_path(&file.path).ok().as_deref() == Some(root_path.as_str())
    }).ok_or_else(|| LocalProjectError::with_path(
        "E_PROJECT_ROOT_MISSING", &root_path, "프로젝트 뿌리 파일이 없습니다",
    ))?;
    reject_source_bom(&root.utf8_text, Some(&root_path))?;
    use ddonirang_lang::work_root::{parse_work_root, FieldValueKind, RootField};
    let directive = parse_work_root(&root.utf8_text).map_err(|error| {
        LocalProjectError::with_path_span("E_PROJECT_ROOT_PARSE", &root_path,
            error.message, (error.span.start, error.span.end))
    })?;
    let epoch = match directive.field(RootField::SourceEpoch).map(|field| &field.kind) {
        Some(FieldValueKind::Text(value)) => Some(value.as_str()),
        Some(_) => return Err(LocalProjectError::with_path("E_PROJECT_SOURCE_PROFILE_INVALID", &root_path,
            "얽음의 글판 값은 \"v25\"처럼 글로 지정해 주세요")),
        None => None,
    };
    match epoch {
        Some("v25") => {}
        Some(value) => return Err(LocalProjectError::with_path("E_PROJECT_SOURCE_PROFILE_UNSUPPORTED", &root_path,
            format!("지원하지 않는 프로젝트 판본입니다: {value}. 알파는 v25만 지원합니다"))),
        None => return Err(LocalProjectError::with_path("E_PROJECT_SOURCE_PROFILE_REQUIRED", &root_path,
            "정식 얽음 뿌리에 글판=\"v25\"를 지정해 주세요. 가지 판본은 문법 글판이 아닙니다")),
    }
    let profile = crate::ddn_runtime::V1_CORE_SUPPORTED_SOURCE_PROFILE_IDENTITY;
    if caller_profile.is_some_and(|caller| caller != profile) {
        return Err(LocalProjectError::with_path("E_PROJECT_SOURCE_PROFILE_CONFLICT", &root_path,
            "호출자가 지정한 판본과 프로젝트 뿌리의 판본이 다릅니다"));
    }
    Ok(profile)
}

pub(super) fn entry_declares_name(text: &str, path: &str, name: &str) -> Result<bool, LocalProjectError> {
    let prepared = runtime_surface::prepare_source_for_execution(text).map_err(|err| {
        LocalProjectError::with_path(
            "E_PROJECT_ENTRY_PARSE",
            path,
            format!("entry DDN 전처리 실패: {err}"),
        )
    })?;
    let program = ddonirang_lang::parse_v25_with_mode(&prepared.execution_source, path, ParseMode::Strict)
    .map_err(|err| {
        LocalProjectError::with_path(
            "E_PROJECT_ENTRY_PARSE",
            path,
            format!("entry DDN 파싱 실패: {}", err.message),
        )
    })?;
    Ok(program.items.iter().any(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        seed.body.as_ref().is_some_and(|body| {
            body.stmts.iter().any(|stmt| match stmt {
                Stmt::Mutate {
                    target:
                        Expr {
                            kind: ExprKind::Var(current),
                            ..
                        },
                    ..
                } => current == name,
                Stmt::DeclBlock { items, .. } => items.iter().any(|item| item.name == name),
                Stmt::Definition { name: current, .. } => current == name,
                _ => false,
            })
        })
    }))
}

pub(super) fn validate_linked_entry_source(
    text: &str,
    path: &str,
    binding_name: &str,
) -> Result<(), LocalProjectError> {
    if !entry_declares_name(text, path, binding_name)? {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_DATASET_BINDING_NOT_LANDED",
            path,
            "자료 연결 뒤 Rust AST에서 DDN 이름을 찾지 못했습니다",
        ));
    }
    Ok(())
}

pub(super) fn find_top_level_target_body(text: &str, target: &str) -> Option<(usize, usize)> {
    let tokens = Lexer::new(text).tokenize().ok()?;
    let mut depth = 0usize;
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if depth == 0 && token.raw == target {
            let mut open_index = index + 1;
            while open_index < tokens.len()
                && !matches!(
                    tokens[open_index].kind,
                    TokenKind::LBrace | TokenKind::Dot | TokenKind::Eof
                )
            {
                open_index += 1;
            }
            if open_index < tokens.len() && matches!(tokens[open_index].kind, TokenKind::LBrace) {
                let mut body_depth = 1usize;
                let mut close_index = open_index + 1;
                while close_index < tokens.len() {
                    match tokens[close_index].kind {
                        TokenKind::LBrace => body_depth += 1,
                        TokenKind::RBrace => {
                            body_depth -= 1;
                            if body_depth == 0 {
                                return Some((
                                    tokens[open_index].span.end,
                                    tokens[close_index].span.start,
                                ));
                            }
                        }
                        _ => {}
                    }
                    close_index += 1;
                }
                return None;
            }
        }
        match token.kind {
            TokenKind::LBrace => depth += 1,
            TokenKind::RBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }
    None
}

pub(super) fn find_canonical_seed_body(text: &str, path: &str) -> Option<(usize, usize)> {
    let program = ddonirang_lang::parse_v25_with_mode(text, path, ParseMode::Strict).ok()?;
    program.items.into_iter().find_map(|item| {
        let TopLevelItem::SeedDef(seed) = item;
        seed.body.map(|body| {
            (
                body.span.start.saturating_add(1),
                body.span.end.saturating_sub(1),
            )
        })
    })
}

pub(super) fn build_rename_file_plan(
    mut source: VirtualProjectSource,
    before_hash: &str,
    from_path: &str,
    to_path: &str,
) -> Result<ProjectEditPlan, LocalProjectError> {
    let from_path = normalize_project_path(from_path)?;
    let to_path = normalize_project_path(to_path)?;
    if from_path == source.root_path {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ROOT_RENAME_FORBIDDEN",
            from_path,
            "지정 뿌리 파일 변경은 호출 경로와 함께 별도로 처리해야 합니다",
        ));
    }
    if source.files.iter().any(|file| file.path == to_path) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_PATH_DUPLICATE",
            to_path,
            "바꿀 경로가 이미 존재합니다",
        ));
    }
    if !source.files.iter().any(|file| file.path == from_path) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_EDIT_FILE_MISSING",
            from_path,
            "이름을 바꿀 파일이 없습니다",
        ));
    }

    let mut patches = Vec::new();
    let mut affected_files = BTreeSet::new();
    let mut affected_references = Vec::new();
    for file in &mut source.files {
        if file.path.ends_with(".ddn") {
            let block_names: &[&str] = if file.path == source.root_path { &["얽음"] } else { &[] };
            for block_name in block_names {
                let edits = work_root_path_edits(&file.utf8_text, &source.root_path, &from_path, &to_path)?;
                let _ = block_name;
                if !edits.is_empty() {
                    let original_path = file.path.clone();
                    apply_text_edits(&mut file.utf8_text, &edits);
                    affected_files.insert(original_path.clone());
                    for (start, end, before, after) in edits {
                        affected_references.push(format!("{original_path}:{start}-{end}"));
                        patches.push(ProjectEditPatch {
                            path: original_path.clone(),
                            start,
                            end,
                            before,
                            after,
                            reason: "structured_project_reference",
                        });
                    }
                }
            }
        }
    }
    let renamed = source
        .files
        .iter_mut()
        .find(|file| file.path == from_path)
        .expect("checked");
    renamed.path = to_path.clone();
    patches.push(ProjectEditPatch {
        path: from_path.clone(),
        start: 0,
        end: 0,
        before: from_path.clone(),
        after: to_path.clone(),
        reason: "file_path",
    });
    affected_files.insert(from_path.clone());
    affected_files.insert(to_path.clone());
    if source.entry_path == from_path {
        source.entry_path = to_path.clone();
    }
    build_project_graph(source.clone())?;
    let after_hash = virtual_project_source_sha256(&source)?;
    Ok(ProjectEditPlan {
        schema: PROJECT_EDIT_PLAN_SCHEMA,
        operation: "rename_file",
        before_source_sha256: before_hash.to_string(),
        after_source_sha256: after_hash,
        affected_files: affected_files.into_iter().collect(),
        affected_symbols: Vec::new(),
        affected_references,
        patches,
        diagnostics: Vec::new(),
        project_source: source,
        atomic_apply: true,
        lock_stale: true,
    })
}

pub(super) fn build_add_module_plan(
    mut source: VirtualProjectSource,
    before_hash: &str,
    path: &str,
    module_name: &str,
) -> Result<ProjectEditPlan, LocalProjectError> {
    use ddonirang_lang::work_root::{parse_work_root, RootField};
    let path = normalize_project_path(path)?;
    if !path.ends_with(".ddn") {
        return Err(LocalProjectError::with_path("E_PROJECT_DDN_EXTENSION", path,
            "모듈 머리 경로는 .ddn으로 끝나야 합니다"));
    }
    if source.files.iter().any(|file| file.path == path) {
        return Err(LocalProjectError::with_path("E_PROJECT_PATH_DUPLICATE", path,
            "추가할 경로가 이미 존재합니다"));
    }
    let base = parent_dir(&source.root_path);
    let relative = if base.is_empty() { path.clone() } else {
        path.strip_prefix(&format!("{base}/")).map(str::to_string)
            .ok_or_else(|| LocalProjectError::new("E_PROJECT_PATH_ESCAPE",
                "뿌리의 원문 경계 밖에 모듈을 추가할 수 없습니다"))?
    };
    let root_file = source.files.iter_mut().find(|file| file.path == source.root_path)
        .ok_or_else(|| LocalProjectError::new("E_PROJECT_EDIT_FILE_MISSING", "작품 뿌리가 없습니다"))?;
    let root = parse_work_root(&root_file.utf8_text)
        .map_err(|error| LocalProjectError::new("E_PROJECT_ROOT_PARSE", error.message))?;
    let existing_modules = root.typed_modules()
        .map_err(|error| LocalProjectError::new("E_PROJECT_ROOT_PARSE", error.message))?;
    let field = root.field(RootField::Modules)
        .ok_or_else(|| LocalProjectError::new("E_PROJECT_ROOT_PARSE", "모듬 선언이 없습니다"))?;
    let start = field.span.start;
    let end = field.span.end;
    let before = root_file.utf8_text[start..end].to_string();
    let closing = before.rfind(']').ok_or_else(||
        LocalProjectError::new("E_PROJECT_ROOT_PARSE", "모듬은 차림으로 선언해야 합니다"))?;
    let quoted_path = serde_json::to_string(&relative).expect("path JSON encoding");
    let mut after = before.clone();
    after.insert_str(closing, &format!(", (이름={module_name}, 머리={quoted_path}, 원문={quoted_path})"));
    let mut updated_root = root_file.utf8_text.clone();
    updated_root.replace_range(start..end, &after);
    // Parse the authored name before changing either file. This also rejects
    // duplicate names and delimiter injection without inventing an alias.
    let updated_modules = parse_work_root(&updated_root)
        .and_then(|root| root.typed_modules())
        .map_err(|error| LocalProjectError::new("E_PROJECT_ROOT_PARSE", error.message))?;
    if updated_modules.len() != existing_modules.len() + 1
        || updated_modules.last().map(|module| module.name.as_str()) != Some(module_name)
    {
        return Err(LocalProjectError::new("E_PROJECT_MODULE_NAME_INVALID",
            "모듈 이름은 따옴표 없는 이름 하나여야 합니다"));
    }
    root_file.utf8_text = updated_root;
    let text = "(값=값) 드러냄.\n값:수 := 0.\n".to_string();
    source.files.push(VirtualProjectFileSource {
        path: path.clone(), role: "ddn_source".to_string(), utf8_text: text.clone(),
    });
    let root_path = source.root_path.clone();
    finish_simple_edit_plan(source, before_hash, "add_module",
        vec![root_path.clone(), path.clone()],
        vec![ProjectEditPatch { path: root_path, start, end, before, after,
            reason: "module_declaration_add" },
            ProjectEditPatch { path, start: 0, end: 0, before: String::new(), after: text,
                reason: "file_add" }], Vec::new())
}

pub(super) fn build_add_file_plan(
    mut source: VirtualProjectSource,
    before_hash: &str,
    path: &str,
    utf8_text: &str,
    role: &str,
    operation: &'static str,
) -> Result<ProjectEditPlan, LocalProjectError> {
    let path = normalize_project_path(path)?;
    if source.files.iter().any(|file| file.path == path) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_PATH_DUPLICATE",
            path,
            "추가할 경로가 이미 존재합니다",
        ));
    }
    if role == "ddn_source" && !path.ends_with(".ddn") {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_DDN_EXTENSION",
            path,
            "DDN source 파일 경로는 .ddn으로 끝나야 합니다",
        ));
    }
    source.files.push(VirtualProjectFileSource {
        path: path.clone(),
        role: role.to_string(),
        utf8_text: utf8_text.to_string(),
    });
    finish_simple_edit_plan(
        source,
        before_hash,
        operation,
        vec![path.clone()],
        vec![ProjectEditPatch {
            path,
            start: 0,
            end: 0,
            before: String::new(),
            after: utf8_text.to_string(),
            reason: "file_add",
        }],
        Vec::new(),
    )
}

pub(super) fn build_duplicate_file_plan(
    source: VirtualProjectSource,
    before_hash: &str,
    from_path: &str,
    to_path: &str,
) -> Result<ProjectEditPlan, LocalProjectError> {
    let from_path = normalize_project_path(from_path)?;
    if from_path == source.root_path {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ROOT_DUPLICATE_FORBIDDEN",
            from_path,
            "작업 머리 파일은 복제할 수 없습니다",
        ));
    }
    let original = source
        .files
        .iter()
        .find(|file| file.path == from_path)
        .ok_or_else(|| {
            LocalProjectError::with_path(
                "E_PROJECT_EDIT_FILE_MISSING",
                from_path,
                "복제할 파일이 없습니다",
            )
        })?;
    let text = original.utf8_text.clone();
    let role = original.role.clone();
    build_add_file_plan(source, before_hash, to_path, &text, &role, "duplicate_file")
}

pub(super) fn build_delete_file_plan(
    mut source: VirtualProjectSource,
    before_hash: &str,
    path: &str,
) -> Result<ProjectEditPlan, LocalProjectError> {
    let path = normalize_project_path(path)?;
    if path == source.root_path {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ROOT_DELETE_FORBIDDEN",
            path,
            "뿌리.ddn은 삭제할 수 없습니다",
        ));
    }
    if path == source.entry_path {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ENTRY_DELETE_FORBIDDEN",
            path,
            "현재 진입 파일은 삭제할 수 없습니다. 먼저 진입 파일을 바꾸세요",
        ));
    }
    if !source.files.iter().any(|file| file.path == path) {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_EDIT_FILE_MISSING",
            path,
            "삭제할 파일이 없습니다",
        ));
    }
    // D9 ownership is declared by the root, not by old path-shaped imports.
    // Reject removing a declared source, including a member of a directory.
    let root = source.files.iter().find(|file| file.path == source.root_path)
        .ok_or_else(|| LocalProjectError::new("E_PROJECT_ROOT_MISSING", "작품 머리가 없습니다"))?;
    let directive = ddonirang_lang::work_root::parse_work_root(&root.utf8_text)
        .map_err(|err| LocalProjectError::new("E_PROJECT_ROOT_PARSE", err.message))?;
    let modules = directive.typed_modules()
        .map_err(|err| LocalProjectError::new("E_PROJECT_ROOT_PARSE", err.message))?;
    let mut references = Vec::new();
    for module in modules {
        for raw in std::iter::once(&module.head).chain(module.sources.iter()) {
            let relative = normalize_project_reference(raw.trim_end_matches('/'))?;
            let base = parent_dir(&source.root_path);
            let declared = normalize_project_path(&if base.is_empty() { relative } else { format!("{base}/{relative}") })?;
            if path == declared || path.starts_with(&format!("{declared}/")) {
                references.push(format!("{}:모듬.{}", source.root_path, module.name));
            }
        }
    }
    if !references.is_empty() {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_FILE_REFERENCED",
            path,
            format!(
                "참조 중인 모듈은 삭제할 수 없습니다: {}",
                references.join(", ")
            ),
        ));
    }
    let removed = source
        .files
        .iter()
        .find(|file| file.path == path)
        .expect("checked")
        .utf8_text
        .clone();
    source.files.retain(|file| file.path != path);
    finish_simple_edit_plan(
        source,
        before_hash,
        "delete_file",
        vec![path.clone()],
        vec![ProjectEditPatch {
            path,
            start: 0,
            end: removed.len(),
            before: removed,
            after: String::new(),
            reason: "file_delete",
        }],
        Vec::new(),
    )
}

pub(super) fn build_set_entry_plan(
    mut source: VirtualProjectSource,
    before_hash: &str,
    path: &str,
) -> Result<ProjectEditPlan, LocalProjectError> {
    let path = normalize_project_path(path)?;
    let candidate = source
        .files
        .iter()
        .find(|file| file.path == path)
        .ok_or_else(|| {
            LocalProjectError::with_path(
                "E_PROJECT_ENTRY_MISSING",
                path.clone(),
                "진입 파일이 프로젝트에 없습니다",
            )
        })?;
    if candidate.role != "ddn_source" || !path.ends_with(".ddn") {
        return Err(LocalProjectError::with_path(
            "E_PROJECT_ENTRY_ROLE",
            path,
            "진입 파일은 DDN source여야 합니다",
        ));
    }
    let old_entry = source.entry_path.clone();
    let mut patches = Vec::new();
    let root = source
        .files
        .iter_mut()
        .find(|file| file.path == source.root_path)
        .expect("normalized root");
    let edits = structural_path_edits(&root.utf8_text, "프로젝트", &old_entry, &path)?;
    apply_text_edits(&mut root.utf8_text, &edits);
    for (start, end, before, after) in edits {
        patches.push(ProjectEditPatch {
            path: root.path.clone(),
            start,
            end,
            before,
            after,
            reason: "entry_reference",
        });
    }
    source.entry_path = path.clone();
    patches.push(ProjectEditPatch {
        path: "@project.entry_path".to_string(),
        start: 0,
        end: old_entry.len(),
        before: old_entry,
        after: path.clone(),
        reason: "entry_path",
    });
    finish_simple_edit_plan(
        source,
        before_hash,
        "set_entry",
        vec![path],
        patches,
        Vec::new(),
    )
}

pub(super) fn finish_simple_edit_plan(
    source: VirtualProjectSource,
    before_hash: &str,
    operation: &'static str,
    affected_files: Vec<String>,
    patches: Vec<ProjectEditPatch>,
    affected_references: Vec<String>,
) -> Result<ProjectEditPlan, LocalProjectError> {
    // File edits cannot silently add a source outside the declared closure or
    // remove a declared head. The root remains the only membership authority.
    build_project_graph(source.clone())?;
    let after_hash = virtual_project_source_sha256(&source)?;
    Ok(ProjectEditPlan {
        schema: PROJECT_EDIT_PLAN_SCHEMA,
        operation,
        before_source_sha256: before_hash.to_string(),
        after_source_sha256: after_hash,
        affected_files,
        affected_symbols: Vec::new(),
        affected_references,
        patches,
        diagnostics: Vec::new(),
        project_source: source,
        atomic_apply: true,
        lock_stale: true,
    })
}

pub(super) fn structural_path_edits(
    text: &str,
    block_name: &str,
    from_path: &str,
    to_path: &str,
) -> Result<Vec<(usize, usize, String, String)>, LocalProjectError> {
    let Some((block_start, block_end)) = extract_named_block_range(text, block_name) else {
        return Ok(Vec::new());
    };
    let block = &text[block_start..block_end];
    let tokens = Lexer::new(block).tokenize().map_err(|err| {
        LocalProjectError::new(
            "E_PROJECT_EDIT_LEX",
            format!("구조 참조 분석에 실패했습니다: {err}"),
        )
    })?;
    let mut edits = Vec::new();
    for token in tokens {
        let TokenKind::StringLit(value) = token.kind else {
            continue;
        };
        let Ok(resolved) = normalize_project_reference(&value) else {
            continue;
        };
        let resolved = if resolved.ends_with(".ddn") {
            resolved
        } else {
            format!("{resolved}.ddn")
        };
        if resolved != from_path {
            continue;
        }
        let replacement_value = if value.ends_with(".ddn") {
            to_path.to_string()
        } else {
            to_path.trim_end_matches(".ddn").to_string()
        };
        let replacement_value = if value.starts_with("./") {
            format!("./{replacement_value}")
        } else {
            replacement_value
        };
        let start = block_start + token.span.start;
        let end = block_start + token.span.end;
        edits.push((start, end, token.raw, format!("\"{replacement_value}\"")));
    }
    Ok(edits)
}

pub(super) fn work_root_path_edits(text: &str, root_path: &str, from: &str, to: &str)
    -> Result<Vec<(usize, usize, String, String)>, LocalProjectError> {
    use ddonirang_lang::work_root::{parse_work_root, RootField};
    let root = parse_work_root(text).map_err(|error| LocalProjectError::new("E_PROJECT_ROOT_PARSE", error.message))?;
    let field = root.field(RootField::Modules).expect("validated modules");
    let base = parent_dir(root_path);
    let relative = |path: &str| -> Result<String, LocalProjectError> {
        if base.is_empty() { return Ok(path.to_string()); }
        path.strip_prefix(&format!("{base}/")).map(str::to_string)
            .ok_or_else(|| LocalProjectError::new("E_PROJECT_PATH_ESCAPE", "뿌리의 원문 경계 밖으로 이동할 수 없습니다"))
    };
    let from = relative(from)?;
    let to = relative(to)?;
    let mut edits = Vec::new();
    for token in &field.tokens {
        if let TokenKind::StringLit(value) = &token.kind {
            if normalize_project_reference(value)? == from {
                edits.push((token.span.start, token.span.end,
                    text[token.span.start..token.span.end].to_string(), serde_json::to_string(&to).unwrap()));
            }
        }
    }
    Ok(edits)
}

pub(super) fn apply_text_edits(text: &mut String, edits: &[(usize, usize, String, String)]) {
    for (start, end, _, after) in edits.iter().rev() {
        text.replace_range(*start..*end, after);
    }
}

