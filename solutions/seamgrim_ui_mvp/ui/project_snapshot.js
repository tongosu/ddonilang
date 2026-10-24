export const PROJECT_SNAPSHOT_SCHEMA = "seamgrim.project_snapshot.v0";
export const PROJECT_SNAPSHOT_WRAPPER_SCHEMA = "seamgrim.project_snapshot_artifact.v0";

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function asText(value, fallback = "") {
  const text = String(value ?? "").replace(/\r\n/g, "\n").trim();
  return text || fallback;
}

function cloneJson(value) {
  if (value === undefined || typeof value === "function") return null;
  return JSON.parse(JSON.stringify(value ?? null));
}

function sortDeep(value) {
  if (Array.isArray(value)) {
    return value.map((item) => sortDeep(item));
  }
  if (value && typeof value === "object") {
    const out = {};
    Object.keys(value)
      .filter((key) => value[key] !== undefined && typeof value[key] !== "function")
      .sort()
      .forEach((key) => {
        out[key] = sortDeep(value[key]);
      });
    return out;
  }
  return value;
}

export function stableProjectSnapshotJson(value) {
  return JSON.stringify(sortDeep(value));
}

async function sha256HexUtf8(text) {
  const bytes = new TextEncoder().encode(String(text ?? ""));
  const subtle = globalThis?.crypto?.subtle;
  if (!subtle || typeof subtle.digest !== "function") {
    throw new Error("project_snapshot_sha256_unavailable");
  }
  const digest = await subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function normalizeControlsSnapshot(value = null) {
  const source = asObject(value);
  const viewWorkspace = asObject(source.view_workspace ?? source.viewWorkspace);
  return {
    selected_view: asText(source.selected_view ?? source.selectedView, ""),
    run_mode: asText(source.run_mode ?? source.runMode, ""),
    controls: sortDeep(asArray(source.controls ?? source.maegim_controls ?? source.maegimControls)),
    view_authoring: sortDeep(asObject(source.view_authoring ?? source.viewAuthoring)),
    ...(Object.keys(viewWorkspace).length ? { view_workspace: sortDeep(viewWorkspace) } : {}),
  };
}

function normalizeLineage(value = null, { sourceLessonId = "" } = {}) {
  const source = asObject(value);
  return {
    source_lesson_id: asText(source.source_lesson_id ?? source.sourceLessonId, sourceLessonId),
    source_project_id: asText(source.source_project_id ?? source.sourceProjectId, ""),
    source_revision_id: asText(source.source_revision_id ?? source.sourceRevisionId, ""),
    source_package_hash: asText(source.source_package_hash ?? source.sourcePackageHash, ""),
    origin_content_sha256: asText(source.origin_content_sha256 ?? source.originContentSha256, ""),
    origin_kind: asText(source.origin_kind ?? source.originKind, "editor_snapshot"),
  };
}

function normalizeLocalPackagePayload(value = null) {
  const payload = asObject(value);
  if (!Object.keys(payload).length) return null;
  const manifest = asObject(payload.manifest);
  return sortDeep({
    ...cloneJson(payload),
    package_hash: asText(payload.package_hash ?? manifest.package_hash, ""),
  });
}

function normalizeCurriculumInquiryArtifact(value = null) {
  const artifact = asObject(value);
  if (!Object.keys(artifact).length) return null;
  return sortDeep(artifact);
}

function normalizeCurriculumArtifactBundle(value = null) {
  const bundle = asObject(value);
  if (!Object.keys(bundle).length) return null;
  return sortDeep(bundle);
}

function normalizeCurriculumModelComparison(value = null) {
  const model = asObject(value);
  if (!Object.keys(model).length) return null;
  return sortDeep(model);
}

function normalizeCurriculumSemanticScene(value = null) {
  const scene = asObject(value);
  if (!Object.keys(scene).length) return null;
  return sortDeep(scene);
}

export function buildProjectSnapshotV0({
  projectName = "",
  ddnText = "",
  lessonPlanText = "",
  sourceLessonId = "",
  controlsSnapshot = null,
  localPackagePayload = null,
  curriculumInquiryArtifact = null,
  curriculumArtifactBundle = null,
  curriculumModelComparison = null,
  curriculumSemanticScene = null,
  lineage = null,
  metadata = null,
} = {}) {
  const normalizedSourceLessonId = asText(sourceLessonId, "custom");
  const localPackage = normalizeLocalPackagePayload(localPackagePayload);
  const inquiryArtifact = normalizeCurriculumInquiryArtifact(curriculumInquiryArtifact);
  const artifactBundle = normalizeCurriculumArtifactBundle(curriculumArtifactBundle);
  const modelComparison = normalizeCurriculumModelComparison(curriculumModelComparison);
  const semanticScene = normalizeCurriculumSemanticScene(curriculumSemanticScene);
  return sortDeep({
    schema: PROJECT_SNAPSHOT_SCHEMA,
    project_name: asText(projectName, "이름 없는 셈그림"),
    source_lesson_id: normalizedSourceLessonId,
    ddn_text: String(ddnText ?? "").replace(/\r\n/g, "\n"),
    lesson_plan_text: String(lessonPlanText ?? "").replace(/\r\n/g, "\n"),
    controls_snapshot: normalizeControlsSnapshot(controlsSnapshot),
    local_package_payload: localPackage,
    package_hash_compat: asText(localPackage?.package_hash, ""),
    ...(inquiryArtifact ? { curriculum_inquiry_artifact: inquiryArtifact } : {}),
    ...(artifactBundle ? { curriculum_artifact_bundle: artifactBundle } : {}),
    ...(modelComparison ? { curriculum_model_comparison: modelComparison } : {}),
    ...(semanticScene ? { curriculum_semantic_scene: semanticScene } : {}),
    lineage: normalizeLineage(lineage, { sourceLessonId: normalizedSourceLessonId }),
    metadata: sortDeep({
      product: "seamgrim_ui_mvp",
      builder: PROJECT_SNAPSHOT_SCHEMA,
      ...asObject(metadata),
    }),
    boundaries: {
      account_required: false,
      public_listing: false,
      server_executes_ddn_for_save: false,
      state_hash_participation: false,
      candidate_package_hash_security_claim: false,
    },
  });
}

export async function buildProjectSnapshotArtifactV0(input = {}) {
  const snapshot = buildProjectSnapshotV0(input);
  const canonicalJson = stableProjectSnapshotJson(snapshot);
  const contentSha256 = await sha256HexUtf8(canonicalJson);
  return {
    schema: PROJECT_SNAPSHOT_WRAPPER_SCHEMA,
    project_snapshot: snapshot,
    canonical_json: canonicalJson,
    content_sha256: contentSha256,
    canonical_byte_length: new TextEncoder().encode(canonicalJson).length,
  };
}

export async function assertProjectSnapshotArtifactV0(artifact = {}) {
  const row = asObject(artifact);
  if (row.schema !== PROJECT_SNAPSHOT_WRAPPER_SCHEMA) {
    throw new Error("project_snapshot_artifact_schema_mismatch");
  }
  const snapshot = asObject(row.project_snapshot);
  if (snapshot.schema !== PROJECT_SNAPSHOT_SCHEMA) {
    throw new Error("project_snapshot_schema_mismatch");
  }
  const expectedJson = stableProjectSnapshotJson(snapshot);
  if (row.canonical_json !== expectedJson) {
    throw new Error("project_snapshot_canonical_json_mismatch");
  }
  const expectedSha = await sha256HexUtf8(expectedJson);
  if (row.content_sha256 !== expectedSha) {
    throw new Error("project_snapshot_sha256_mismatch");
  }
  return true;
}

export function buildProjectSnapshotInputFromEditorState({
  editorScreen = null,
  appState = null,
  projectName = "",
  localPackagePayload = null,
  controlsSnapshot = null,
  curriculumInquiryArtifact = null,
  curriculumArtifactBundle = null,
  curriculumModelComparison = null,
  curriculumSemanticScene = null,
  lineage = null,
} = {}) {
  const state = asObject(appState);
  const lesson = asObject(state.currentLesson);
  const ddnText = typeof editorScreen?.getDdn === "function"
    ? editorScreen.getDdn()
    : String(lesson.ddnText ?? "");
  const lessonPlanText = typeof editorScreen?.getLessonPlanText === "function"
    ? editorScreen.getLessonPlanText()
    : String(lesson.lessonPlanText ?? "");
  return {
    projectName: asText(projectName, lesson.title || "이름 없는 셈그림"),
    ddnText,
    lessonPlanText,
    sourceLessonId: asText(lesson.id, "custom"),
    controlsSnapshot: controlsSnapshot ?? {
      selected_view: String(state.run?.selectedView ?? state.studio?.selectedView ?? ""),
      run_mode: String(state.run?.mode ?? ""),
      controls: asArray(lesson.maegimControls ?? lesson.controls),
      view_authoring: asObject(state.studio?.viewAuthoring),
    },
    localPackagePayload,
    curriculumInquiryArtifact,
    curriculumArtifactBundle,
    curriculumModelComparison,
    curriculumSemanticScene,
    lineage: lineage ?? {
      source_lesson_id: asText(lesson.id, "custom"),
      source_package_hash: asText(lesson.localPackageHash, ""),
      origin_kind: lesson.localPackageId ? "local_package" : "editor_snapshot",
    },
  };
}
