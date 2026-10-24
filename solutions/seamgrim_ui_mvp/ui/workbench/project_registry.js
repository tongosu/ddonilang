// Existing one-file, portable and native project registry model; original templates and state references unchanged.
export function createWorkbenchProjectRegistry({ currentProject, FIRST_STEPS_PROJECTS, localProjectEntryPath, ONE_FILE_DDN_DRAFT_KEY, ONE_FILE_DDN_PROJECT_ID, PORTABLE_PROJECT_CATALOG_KEY, PROJECTS, pushDiagnostic, rememberMyWorkProject, renderAll, selectProject, selectUnversionedDocumentProfile, SOURCE_ANALYSIS_PROFILE_IDENTITY, stableCanonicalJson, stableHash, state, workbenchProjectRegistryState, writeNativeProjectCatalog, readPortableProjectCatalog }) {
function defaultOneFileDdnSource(title = "새 한 파일 DDN") {
  // The user-facing one-file template is current V25 source. Its title is
  // Workbench presentation metadata, not hidden source syntax.
  void title;
  return `값 := 1.
값 보여주기.
`;
}

function readOneFileDraft() {
  try {
    const raw = globalThis.localStorage?.getItem?.(ONE_FILE_DDN_DRAFT_KEY) ?? "";
    if (!raw) return "";
    const parsed = JSON.parse(raw);
    return String(parsed.source ?? "");
  } catch (_) {
    return "";
  }
}

function writeOneFileDraft(source) {
  try {
    globalThis.localStorage?.setItem?.(ONE_FILE_DDN_DRAFT_KEY, stableCanonicalJson({
      schema: "seamgrim.workbench.one_file_ddn_draft.v1",
      project_id: ONE_FILE_DDN_PROJECT_ID,
      source: String(source ?? ""),
    }));
  } catch (_) {
    // Local recovery is best-effort in restricted browser contexts.
  }
}

function writePortableProjectCatalog(rows = []) {
  try {
    globalThis.localStorage?.setItem?.(PORTABLE_PROJECT_CATALOG_KEY, JSON.stringify(rows.slice(0, 30)));
  } catch (_) {
    // Local catalog is best-effort; import preview still works without it.
  }
}

function normalizePortableTextArray(value) {
  return Array.isArray(value)
    ? value.map((item) => String(item ?? "").trim()).filter(Boolean).slice(0, 12)
    : [];
}

function localProjectTreePayloadFromSnapshot(snapshot = {}) {
  const payload = snapshot?.local_package_payload;
  if (
    payload &&
    typeof payload === "object" &&
    payload.kind === "seamgrim.local_project_tree.v0" &&
    Array.isArray(payload.files) &&
    payload.files.length > 0
  ) {
    return payload;
  }
  return null;
}

function projectRunLineageFromPayload(payload = {}) {
  const source = payload?.run_lineage && typeof payload.run_lineage === "object" ? payload.run_lineage : {};
  const request = payload?.execution_preparation?.runtime_request && typeof payload.execution_preparation.runtime_request === "object"
    ? payload.execution_preparation.runtime_request
    : {
      frontdoor: String(payload?.frontdoor ?? "wasm_project_run_summary_json"),
      requested_ticks: Number(payload?.runtime_execution_envelope?.requested_ticks ?? 1),
      target: String(payload?.target ?? ""),
      target_path: String(payload?.target_path ?? payload?.entry_path ?? "app/main.ddn"),
    };
  const runRequestHash = String(source.run_request_hash ?? "").trim()
    || `candidate:fnv1a32:project_run_request:${stableHash(stableCanonicalJson(request))}`;
  const lineage = {
    schema: "seamgrim.local_project_run_lineage.v0",
    project_graph_hash: String(source.project_graph_hash ?? payload?.project_graph_sha256 ?? ""),
    lock_hash: String(source.lock_hash ?? payload?.lock_sha256 ?? ""),
    linked_source_hash: String(
      source.linked_source_hash
        ?? payload?.linked_execution_source_sha256
        ?? payload?.execution_preparation?.linked_source?.source_sha256
        ?? "",
    ),
    run_request_hash: runRequestHash,
    parent_run_id: String(source.parent_run_id ?? ""),
    branch_madi: source.branch_madi === null || source.branch_madi === undefined ? null : Number(source.branch_madi),
    branch_state_hash: String(source.branch_state_hash ?? ""),
    final_state_hash: String(source.final_state_hash ?? payload?.runtime_state_hash ?? ""),
    view_artifact_hash: String(source.view_artifact_hash ?? ""),
    created_revision: String(source.created_revision ?? payload?.summary_sha256 ?? payload?.bounded_state_sha256 ?? ""),
  };
  return {
    ...lineage,
    lineage_hash: String(source.lineage_hash ?? "").trim()
      || `candidate:fnv1a32:project_run_lineage:${stableHash(stableCanonicalJson(lineage))}`,
  };
}

function localProjectRunEvidenceFromProject(project = currentProject()) {
  if (state.projectExecution?.current?.evidence) {
    return state.projectExecution.current.evidence;
  }
  void project;
  return null;
}

function importedProjectRunEvidenceFromProject(project = currentProject()) {
  const payload = localProjectTreePayloadFromSnapshot(project.portableSnapshot ?? {});
  if (!payload) return null;
  const trace = payload.project_observation_trace && typeof payload.project_observation_trace === "object"
    ? payload.project_observation_trace
    : null;
  const traceRows = Array.isArray(trace?.rows) ? trace.rows : [];
  const outputRows = Array.isArray(payload.output_rows) ? payload.output_rows.map((row) => String(row ?? "")) : [];
  if (!traceRows.length && !outputRows.length && !String(payload.bounded_state_sha256 ?? "").trim()) return null;
  return {
    schema: "seamgrim.approved_workbench.local_project_run_evidence.v1",
    frontdoor: String(payload.frontdoor ?? "wasm_project_run_summary_json"),
    bounded_state_sha256: String(payload.bounded_state_sha256 ?? ""),
    runtime_state_hash: String(payload.runtime_state_hash ?? trace?.determinism?.runtime_state_hash ?? ""),
    summary_sha256: String(payload.summary_sha256 ?? ""),
    trace_sha256: String(trace?.trace_sha256 ?? ""),
    run_lineage: projectRunLineageFromPayload(payload),
    output_rows: outputRows,
    observation_rows: traceRows.map((row, index) => ({
      index: Number(row?.index ?? index),
      observation_id: String(row?.observation_id ?? `project.obs.${index + 1}`),
      name: String(row?.name ?? `output_${index + 1}`),
      value_text: String(row?.value_text ?? outputRows[index] ?? ""),
      unit: row?.unit === null || row?.unit === undefined ? "" : String(row.unit),
      state_hash_participation: Boolean(row?.state_hash_participation),
    })),
  };
}

function gajiIdsFromLocalProjectPayload(payload = {}) {
  const ids = new Set();
  for (const file of Array.isArray(payload.files) ? payload.files : []) {
    const pathValue = String(file?.path ?? "");
    const match = pathValue.match(/^vendor\/([^/]+)\/gaji\.toml$/u);
    if (match?.[1]) ids.add(match[1]);
  }
  return Array.from(ids).sort();
}

function filesFromLocalProjectPayload(payload = {}, snapshot = {}) {
  const entryPath = String(payload.entry_path ?? "").trim();
  const rows = (Array.isArray(payload.files) ? payload.files : [])
    .map((file) => ({
      path: String(file?.path ?? "").trim(),
      role: String(file?.role ?? "ddn_source").trim() || "ddn_source",
      fallback: String(file?.utf8_text ?? file?.text ?? ""),
    }))
    .filter((file) => file.path);
  rows.sort((left, right) => {
    if (left.path === entryPath) return -1;
    if (right.path === entryPath) return 1;
    return left.path.localeCompare(right.path, "ko");
  });
  if (!rows.length) {
    return [{
      path: "lesson.ddn",
      role: "main",
      fallback: String(snapshot.ddn_text ?? ""),
    }];
  }
  return rows;
}

function portableProjectFromArtifact(artifact = {}, { catalogId = "", importedAt = "" } = {}) {
  const snapshot = artifact.project_snapshot ?? {};
  const localProjectPayload = localProjectTreePayloadFromSnapshot(snapshot);
  const payloadGaji = localProjectPayload ? gajiIdsFromLocalProjectPayload(localProjectPayload) : [];
  const metadata = {
    ...(snapshot.metadata ?? {}),
    required_gaji: Array.isArray(snapshot.metadata?.required_gaji)
      ? snapshot.metadata.required_gaji
      : payloadGaji,
  };
  const title = String(metadata.title || snapshot.project_name || "받은 셈그림 프로젝트");
  const summary = String(metadata.summary || "검증된 local file에서 가져온 프로젝트입니다.");
  const tags = Array.from(new Set(["받은 프로젝트", ...normalizePortableTextArray(metadata.tags)]));
  const sourceLessonId = String(snapshot.source_lesson_id || "portable");
  const known = PROJECTS.find((project) => project.id === sourceLessonId);
  const hashPrefix = String(artifact.content_sha256 || "portable").slice(0, 12) || "portable";
  const id = catalogId || `portable_${hashPrefix}_${String(Date.now()).slice(-6)}`;
  const files = localProjectPayload
    ? filesFromLocalProjectPayload(localProjectPayload, snapshot)
    : [
      {
        path: "lesson.ddn",
        role: "main",
        fallback: String(snapshot.ddn_text ?? ""),
      },
    ];
  return {
    id,
    title,
    kind: localProjectPayload ? "받은 로컬 프로젝트" : "받은 프로젝트",
    summary,
    tags,
    primaryView: known?.primaryView ?? "line",
    viewGrade: "portable-verified",
    viewPolicy: known?.viewPolicy ?? {
      primary: "가져온 DDN 보개",
      side: "가져온 관찰값",
      overlay: "가져온 보기 후보",
      magnifier: "가져온 보기 확대",
    },
    portableMetadata: {
      ...metadata,
      title,
      summary,
      tags,
      origin_hash_prefix: hashPrefix,
      received_at: importedAt || new Date().toISOString(),
      official_claim_ignored: true,
      trust_badge: "integrity_verified_only",
    },
    portableSnapshot: snapshot,
    portableArtifact: artifact,
    files,
  };
}

function loadPortableProjectCatalog() {
  workbenchProjectRegistryState.receivedPortableProjects = readPortableProjectCatalog()
    .map((row) => {
      try {
        const project = portableProjectFromArtifact(row.artifact, {
          catalogId: String(row.project_id || ""),
          importedAt: String(row.imported_at || ""),
        });
        if (row.selected_source_profile_identity) selectUnversionedDocumentProfile(project, row.selected_source_profile_identity);
        return project;
      } catch (_) {
        return null;
      }
    })
    .filter(Boolean);
}

function rememberPortableProject(project) {
  const artifact = project.portableArtifact;
  if (!artifact?.content_sha256) return;
  const rows = readPortableProjectCatalog();
  const next = {
    schema: "seamgrim.workbench.portable_project_entry.v1",
    project_id: project.id,
    title: project.title,
    content_sha256: artifact.content_sha256,
    imported_at: project.portableMetadata?.received_at || new Date().toISOString(),
    artifact,
    selected_source_profile_identity: project.selectedImportSourceProfileIdentity || "",
  };
  writePortableProjectCatalog([next, ...rows]);
}

function buildOneFileProject(source = "") {
  return {
    sourceProfileIdentity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
    id: ONE_FILE_DDN_PROJECT_ID,
    title: "새 한 파일 DDN",
    kind: "로컬 한 파일 DDN",
    summary: "예제 복제가 아니라 사용자가 직접 편집하는 최소 한 파일 프로젝트입니다.",
    tags: ["새 DDN", "로컬", "한 파일"],
    primaryView: "line",
    viewGrade: "source-only",
    viewPolicy: {
      primary: "실행 출력 보개",
      side: "값 관찰",
      overlay: "후보 원문 표시",
      magnifier: "현재 출력 확대",
    },
    files: [
      {
        path: "lesson.ddn",
        role: "main",
        fallback: source || defaultOneFileDdnSource(),
      },
    ],
  };
}

function ddnQuotedText(value) {
  return String(value ?? "").replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/[\r\n]+/g, " ").trim();
}

function nativeProjectFiles(name, template = "physics") {
  const safeName = ddnQuotedText(name) || "새 DDN 프로젝트";
  if (template === "single") {
    return [
      {
        path: "lesson.ddn",
        role: "main",
        utf8_text: defaultOneFileDdnSource(safeName),
      },
    ];
  }
  const modules = template === "empty"
    ? '(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn")'
    : '(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn"), (이름=힘, 머리="lib/힘.ddn", 원문="lib/힘.ddn")';
  const root = `(글판="v25", 모듬=차림[${modules}], 시작=(모듈=앱, 정의=실행)) 얽음.`;
  if (template === "empty") {
    return [
      { path: "뿌리.ddn", role: "root", utf8_text: root },
      {
        path: "app/main.ddn",
        role: "ddn_source",
        utf8_text: `실행:움직씨 := {
  값:수 := 0.
  값 보여주기.
}.
`,
      },
    ];
  }
  return [
    { path: "뿌리.ddn", role: "root", utf8_text: root },
    {
      path: "app/main.ddn",
      role: "ddn_source",
      utf8_text: `(힘=힘) 쓰임.

실행:움직씨 := {
초기속도x:수 := 0.
초기속도y:수 := 20.
시간간격:수 := 0.1.
중력가속도:수 := 힘.중력.
속도:수 := 초기속도y - 중력가속도 * 시간간격.
위치:수 := 초기속도x + 속도 * 시간간격.
위치 보여주기.
속도 보여주기.
힘.중력 보여주기.
}.
`,
    },
    {
      path: "lib/힘.ddn",
      role: "ddn_source",
      utf8_text: `(중력=중력) 드러냄.

중력:수 := 9.8.
`,
    },
  ];
}

function buildNativeProject(name, template = "physics") {
  const title = String(name ?? "").trim() || "새 DDN 프로젝트";
  const files = nativeProjectFiles(title, template);
  const singleFile = template === "single";
  const payload = singleFile
    ? null
    : {
        schema: "seamgrim.local_project_payload.v0",
        kind: "seamgrim.local_project_tree.v0",
        root_path: "뿌리.ddn",
        entry_path: "app/main.ddn",
        files,
      };
  const id = `native_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 7)}`;
  const kind = template === "physics"
    ? "로컬 물리 DDN 프로젝트"
    : singleFile
      ? "로컬 한 파일 DDN 프로젝트"
      : "로컬 빈 DDN 프로젝트";
  const summary = singleFile
    ? "가져오기 없이 이 작업대에서 만든 네이티브 한 파일 프로젝트입니다."
    : "가져오기 없이 이 작업대에서 만든 네이티브 다중 파일 프로젝트입니다.";
  const templateTag = template === "physics" ? "물리" : singleFile ? "한 파일" : "빈 프로젝트";
  const portableSnapshot = {
    project_name: title,
    source_lesson_id: id,
    creation_template: template,
    metadata: { title, summary: "셈그림에서 새로 만든 DDN 프로젝트", tags: ["내 작업", "네이티브", templateTag], source_profile_identity: SOURCE_ANALYSIS_PROFILE_IDENTITY },
  };
  if (payload) portableSnapshot.local_package_payload = payload;
  const moduleDirectiveRegistry = template === "physics"
    ? {
        known_import_targets: ["./lib/힘"],
        public_definition_targets: ["중력"],
      }
    : null;
  return {
    id,
    title,
    kind,
    summary,
    creationTemplate: template,
    tags: ["내 작업", "로컬", "네이티브", templateTag],
    primaryView: template === "physics" ? "projectile" : "line",
    viewGrade: "source-authored",
    viewPolicy: {
      primary: template === "physics" ? "위치/속도 운동 보개" : "실행 출력 보개",
      side: "이름·값·단위 관찰",
      overlay: "현재 마디 상태",
      magnifier: "현재 관찰값 확대",
    },
    portableSnapshot,
    moduleDirectiveRegistry,
    files: files.map((file) => ({ path: file.path, role: file.role, fallback: file.utf8_text })),
  };
}

async function createNativeProject(name, template, initialSource = null) {
  const project = buildNativeProject(name, template);
  if(initialSource!==null) {
    if(template!=='single' || typeof initialSource!=='string') throw new Error('한 파일 시작 원문을 명시하세요.');
    project.files[0].fallback=initialSource;
  }
  workbenchProjectRegistryState.nativeProjects = [project, ...workbenchProjectRegistryState.nativeProjects];
  writeNativeProjectCatalog();
  await selectProject(project.id);
  state.activeFilePath = localProjectEntryPath(project);
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "local_recovery",
    server_saved: false,
    local_saved: true,
    user_label: `새 네이티브 ${project.files.length === 1 ? "한 파일 " : ""}프로젝트 local recovery 준비`,
  };
  rememberMyWorkProject({
    projectId: project.id,
    title: project.title,
    source: "local_only",
    role: "edit",
    updatedAt: new Date().toISOString(),
  });
  pushDiagnostic("W_NATIVE_PROJECT_CREATED", `${project.title}: 실제 DDN 파일 ${project.files.length}개를 만들었습니다.`);
  renderAll();
}

function allProjects() {
  return [
    ...workbenchProjectRegistryState.nativeProjects,
    ...(workbenchProjectRegistryState.customOneFileProject ? [workbenchProjectRegistryState.customOneFileProject] : []),
    ...workbenchProjectRegistryState.receivedPortableProjects,
    ...FIRST_STEPS_PROJECTS,
    ...PROJECTS,
  ];
}

  return { defaultOneFileDdnSource, readOneFileDraft, writeOneFileDraft, writePortableProjectCatalog, normalizePortableTextArray, localProjectTreePayloadFromSnapshot, projectRunLineageFromPayload, localProjectRunEvidenceFromProject, importedProjectRunEvidenceFromProject, gajiIdsFromLocalProjectPayload, filesFromLocalProjectPayload, portableProjectFromArtifact, loadPortableProjectCatalog, rememberPortableProject, buildOneFileProject, ddnQuotedText, nativeProjectFiles, buildNativeProject, createNativeProject, allProjects };
}
