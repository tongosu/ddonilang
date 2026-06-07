function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function asBool(value) {
  return value === true || value === "true" || value === "참";
}

function asCount(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) return 0;
  return Math.max(0, Math.trunc(number));
}

function countLines(text) {
  const body = String(text ?? "");
  if (!body) return 0;
  return body.split("\n").length;
}

function normalizeDraft(draft = {}) {
  const row = asObject(draft);
  const ddnText = String(row.ddn_text ?? row.ddnText ?? row.source_text ?? row.sourceText ?? "");
  return {
    draft_id: asText(row.draft_id ?? row.id, "draft_local_001"),
    lesson_id: asText(row.lesson_id ?? row.lessonId, "local_authoring_lesson"),
    title: asText(row.title ?? row["제목"], "local authoring draft"),
    source_kind: asText(row.source_kind ?? row.sourceKind, "direct_ddn"),
    ddn_text: ddnText,
    ddn_line_count: countLines(ddnText),
    dirty: asBool(row.dirty ?? row.needs_save ?? row.needsSave),
  };
}

function normalizeRunRequest(runRequest = {}) {
  const row = asObject(runRequest);
  return {
    launch_kind: asText(row.launch_kind ?? row.launchKind, "editor_run"),
    source_type: asText(row.source_type ?? row.sourceType, "ddn"),
    created_at_ms: asCount(row.created_at_ms ?? row.createdAtMs),
    auto_execute: asBool(row.auto_execute ?? row.autoExecute),
  };
}

function normalizeSaveState(saveState = {}) {
  const row = asObject(saveState);
  return {
    local_save_available: asBool(row.local_save_available ?? row.localSaveAvailable ?? true),
    local_save_status: asText(row.local_save_status ?? row.localSaveStatus, "저장 대기"),
    filename: asText(row.filename ?? row.download, "lesson.ddn"),
    remote_save_claim: asBool(row.remote_save_claim ?? row.remoteSaveClaim),
  };
}

function normalizeLoaderContract(loaderContract = {}) {
  const row = asObject(loaderContract);
  return {
    lesson_loader_reused: asBool(row.lesson_loader_reused ?? row.lessonLoaderReused ?? true),
    lesson_schema_change: asBool(row.lesson_schema_change ?? row.lessonSchemaChange),
    active_allowlist_mutation: asBool(row.active_allowlist_mutation ?? row.activeAllowlistMutation),
  };
}

function normalizeRunPresetContext(runPresetContext = {}) {
  const row = asObject(runPresetContext);
  return {
    preset_schema: asText(row.preset_schema ?? row.schema, "seamgrim.run_preset_rail.v1"),
    onboarding_profile: asText(row.onboarding_profile ?? row.onboardingProfile, "student"),
    layout_mode: asText(row.layout_mode ?? row.layoutMode, "split"),
    required_view_count: asCount(row.required_view_count ?? row.requiredViewCount),
  };
}

export function buildLessonAuthoringRunIntegration({
  draft = {},
  runRequest = {},
  saveState = {},
  loaderContract = {},
  runPresetContext = {},
} = {}) {
  const normalizedDraft = normalizeDraft(draft);
  const normalizedRunRequest = normalizeRunRequest(runRequest);
  const normalizedSaveState = normalizeSaveState(saveState);
  const normalizedLoaderContract = normalizeLoaderContract(loaderContract);
  const normalizedRunPresetContext = normalizeRunPresetContext(runPresetContext);
  const stages = [
    ["authoring_draft", normalizedDraft.ddn_text.trim().length > 0 && normalizedDraft.ddn_line_count > 0],
    ["draft_edit_state", normalizedDraft.dirty === true],
    ["run_request", normalizedRunRequest.launch_kind.length > 0 && normalizedRunRequest.source_type === "ddn"],
    ["local_save_path", normalizedSaveState.local_save_available === true && normalizedSaveState.filename.endsWith(".ddn")],
    ["lesson_loader_contract", normalizedLoaderContract.lesson_loader_reused === true],
    ["run_preset_context", normalizedRunPresetContext.preset_schema === "seamgrim.run_preset_rail.v1"],
    [
      "no_schema_boundary",
      normalizedLoaderContract.lesson_schema_change === false
        && normalizedLoaderContract.active_allowlist_mutation === false
        && normalizedSaveState.remote_save_claim === false,
    ],
  ].map(([stageId, ready]) => ({
    stage_id: stageId,
    ready: ready === true,
  }));
  const readyStageCount = stages.filter((stage) => stage.ready).length;
  const ready = readyStageCount === stages.length;
  return {
    schema: "seamgrim.lesson_authoring_run_integration.v1",
    primary_coordinate: "마-3",
    support_coordinate: "라-3",
    workflow_claim: "lesson_authoring_run_integration",
    generated_locally: true,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    runtime_claim: false,
    remote_save_claim: false,
    replay_claim: false,
    status: ready ? "authoring_run_ready" : "authoring_run_incomplete",
    tone: ready ? "success" : "warning",
    stage_count: stages.length,
    ready_stage_count: readyStageCount,
    missing_stage_count: stages.length - readyStageCount,
    draft_id: normalizedDraft.draft_id,
    lesson_id: normalizedDraft.lesson_id,
    title: normalizedDraft.title,
    source_kind: normalizedDraft.source_kind,
    ddn_line_count: normalizedDraft.ddn_line_count,
    launch_kind: normalizedRunRequest.launch_kind,
    source_type: normalizedRunRequest.source_type,
    local_save_status: normalizedSaveState.local_save_status,
    local_save_filename: normalizedSaveState.filename,
    onboarding_profile: normalizedRunPresetContext.onboarding_profile,
    layout_mode: normalizedRunPresetContext.layout_mode,
    required_view_count: normalizedRunPresetContext.required_view_count,
    stages,
    draft: normalizedDraft,
    run_request: normalizedRunRequest,
    save_state: normalizedSaveState,
    loader_contract: normalizedLoaderContract,
    run_preset_context: normalizedRunPresetContext,
  };
}

export function formatLessonAuthoringRunIntegrationText(workflow = {}) {
  const row = asObject(workflow);
  if (row.schema !== "seamgrim.lesson_authoring_run_integration.v1") {
    throw new Error("seamgrim_expected_lesson_authoring_run_integration");
  }
  const stages = Array.isArray(row.stages) ? row.stages : [];
  const stageLines = stages.map((stage) => [
    String(stage.stage_id ?? ""),
    stage.ready === true ? "true" : "false",
  ].join("\t"));
  return [
    `schema\t${row.schema ?? ""}`,
    `primary_coordinate\t${row.primary_coordinate ?? ""}`,
    `support_coordinate\t${row.support_coordinate ?? ""}`,
    `workflow_claim\t${row.workflow_claim ?? ""}`,
    `generated_locally\t${row.generated_locally === true ? "true" : "false"}`,
    `lesson_schema_change\t${row.lesson_schema_change === true ? "true" : "false"}`,
    `active_allowlist_mutation\t${row.active_allowlist_mutation === true ? "true" : "false"}`,
    `runtime_claim\t${row.runtime_claim === true ? "true" : "false"}`,
    `remote_save_claim\t${row.remote_save_claim === true ? "true" : "false"}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status ?? ""}`,
    `stage_count\t${row.stage_count ?? stages.length}`,
    `ready_stage_count\t${row.ready_stage_count ?? 0}`,
    `missing_stage_count\t${row.missing_stage_count ?? 0}`,
    `draft_id\t${row.draft_id ?? ""}`,
    `lesson_id\t${row.lesson_id ?? ""}`,
    `title\t${row.title ?? ""}`,
    `source_kind\t${row.source_kind ?? ""}`,
    `ddn_line_count\t${row.ddn_line_count ?? 0}`,
    `launch_kind\t${row.launch_kind ?? ""}`,
    `source_type\t${row.source_type ?? ""}`,
    `local_save_status\t${row.local_save_status ?? ""}`,
    `local_save_filename\t${row.local_save_filename ?? ""}`,
    `onboarding_profile\t${row.onboarding_profile ?? ""}`,
    `layout_mode\t${row.layout_mode ?? ""}`,
    `required_view_count\t${row.required_view_count ?? 0}`,
    "",
    "stage_id\tready",
    ...stageLines,
  ].join("\n");
}
