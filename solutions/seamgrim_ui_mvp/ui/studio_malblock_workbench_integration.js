function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
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

function normalizePaletteSummary(paletteSummary = {}) {
  const row = asObject(paletteSummary);
  return {
    category_count: asCount(row.category_count ?? row.categoryCount),
    category_ids: asArray(row.category_ids ?? row.categoryIds).map((item) => String(item ?? "")).filter(Boolean),
    block_count: asCount(row.block_count ?? row.blockCount),
  };
}

function normalizeCanvasSummary(canvasSummary = {}) {
  const row = asObject(canvasSummary);
  return {
    block_count: asCount(row.block_count ?? row.blockCount),
    block_kinds: asArray(row.block_kinds ?? row.blockKinds).map((item) => String(item ?? "")).filter(Boolean),
    error_count: asCount(row.error_count ?? row.errorCount),
  };
}

function normalizeCallbacks(callbacks = {}) {
  const row = asObject(callbacks);
  return {
    text_mode_count: asCount(row.text_mode_count ?? row.textModeCount),
    run_count: asCount(row.run_count ?? row.runCount),
    last_text_mode_ddn: String(row.last_text_mode_ddn ?? row.lastTextModeDdn ?? ""),
    last_run_ddn: String(row.last_run_ddn ?? row.lastRunDdn ?? ""),
  };
}

function normalizeSaveState(saveState = {}) {
  const row = asObject(saveState);
  return {
    local_save_available: asBool(row.local_save_available ?? row.localSaveAvailable ?? true),
    filename: asText(row.filename ?? row.download, "lesson.ddn"),
    remote_save_claim: asBool(row.remote_save_claim ?? row.remoteSaveClaim),
  };
}

function normalizeRunRequest(runRequest = {}) {
  const row = asObject(runRequest);
  return {
    launch_kind: asText(row.launch_kind ?? row.launchKind, "block_editor_run"),
    source_type: asText(row.source_type ?? row.sourceType, "ddn"),
  };
}

function normalizeDecodeState(decodeState = {}) {
  const row = asObject(decodeState);
  return {
    error_count: asCount(row.error_count ?? row.errorCount),
    raw_fallback_count: asCount(row.raw_fallback_count ?? row.rawFallbackCount),
    error_text: String(row.error_text ?? row.errorText ?? ""),
  };
}

function normalizeWorkbenchContext(workbenchContext = {}) {
  const row = asObject(workbenchContext);
  return {
    source_mode: asText(row.source_mode ?? row.sourceMode, "malblock"),
    target_mode: asText(row.target_mode ?? row.targetMode, "studio_run"),
    workbench_shell_reused: asBool(row.workbench_shell_reused ?? row.workbenchShellReused ?? true),
    lesson_schema_change: asBool(row.lesson_schema_change ?? row.lessonSchemaChange),
    active_allowlist_mutation: asBool(row.active_allowlist_mutation ?? row.activeAllowlistMutation),
    parser_frontdoor_change: asBool(row.parser_frontdoor_change ?? row.parserFrontdoorChange),
    runtime_claim: asBool(row.runtime_claim ?? row.runtimeClaim),
  };
}

export function buildMalblockWorkbenchIntegration({
  paletteSummary = {},
  canvasSummary = {},
  ddnText = "",
  callbacks = {},
  saveState = {},
  runRequest = {},
  decodeState = {},
  workbenchContext = {},
} = {}) {
  const palette = normalizePaletteSummary(paletteSummary);
  const canvas = normalizeCanvasSummary(canvasSummary);
  const callbackState = normalizeCallbacks(callbacks);
  const save = normalizeSaveState(saveState);
  const run = normalizeRunRequest(runRequest);
  const decode = normalizeDecodeState(decodeState);
  const context = normalizeWorkbenchContext(workbenchContext);
  const generatedDdn = String(ddnText ?? "");
  const stages = [
    ["palette_grouping", palette.category_count >= 4 && palette.category_ids.length >= 4],
    ["canvas_blocks", canvas.block_count > 0 && canvas.block_kinds.length > 0],
    ["ddn_generation", generatedDdn.trim().length > 0 && countLines(generatedDdn) > 0],
    ["text_mode_callback", callbackState.text_mode_count > 0 && callbackState.last_text_mode_ddn === generatedDdn],
    ["run_callback", callbackState.run_count > 0 && callbackState.last_run_ddn === generatedDdn],
    ["local_save_boundary", save.local_save_available === true && save.filename.endsWith(".ddn") && save.remote_save_claim === false],
    ["workbench_shell_boundary", context.workbench_shell_reused === true && context.source_mode === "malblock"],
    ["decode_error_boundary", decode.error_count >= 1 && decode.raw_fallback_count >= 1],
    [
      "no_surface_expansion",
      context.lesson_schema_change === false
        && context.active_allowlist_mutation === false
        && context.parser_frontdoor_change === false
        && context.runtime_claim === false,
    ],
  ].map(([stageId, ready]) => ({
    stage_id: stageId,
    ready: ready === true,
  }));
  const readyStageCount = stages.filter((stage) => stage.ready).length;
  const ready = readyStageCount === stages.length;
  return {
    schema: "seamgrim.malblock_workbench_integration.v1",
    primary_coordinate: "마-3",
    support_coordinate: "라-3",
    workflow_claim: "malblock_workbench_integration",
    generated_locally: true,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    runtime_claim: false,
    remote_save_claim: false,
    replay_claim: false,
    status: ready ? "malblock_workbench_ready" : "malblock_workbench_incomplete",
    tone: ready ? "success" : "warning",
    stage_count: stages.length,
    ready_stage_count: readyStageCount,
    missing_stage_count: stages.length - readyStageCount,
    palette_category_count: palette.category_count,
    palette_category_ids: palette.category_ids,
    canvas_block_count: canvas.block_count,
    canvas_block_kinds: canvas.block_kinds,
    ddn_line_count: countLines(generatedDdn),
    text_mode_count: callbackState.text_mode_count,
    run_count: callbackState.run_count,
    local_save_filename: save.filename,
    launch_kind: run.launch_kind,
    source_type: run.source_type,
    decode_error_count: decode.error_count,
    raw_fallback_count: decode.raw_fallback_count,
    source_mode: context.source_mode,
    target_mode: context.target_mode,
    stages,
    palette_summary: palette,
    canvas_summary: canvas,
    callbacks: callbackState,
    save_state: save,
    run_request: run,
    decode_state: decode,
    workbench_context: context,
  };
}

export function formatMalblockWorkbenchIntegrationText(workflow = {}) {
  const row = asObject(workflow);
  if (row.schema !== "seamgrim.malblock_workbench_integration.v1") {
    throw new Error("seamgrim_expected_malblock_workbench_integration");
  }
  const stages = asArray(row.stages);
  const stageLines = stages.map((stage) => [
    String(stage.stage_id ?? ""),
    stage.ready === true ? "true" : "false",
  ].join("\t"));
  const categoryIds = asArray(row.palette_category_ids);
  const blockKinds = asArray(row.canvas_block_kinds);
  return [
    `schema\t${row.schema ?? ""}`,
    `primary_coordinate\t${row.primary_coordinate ?? ""}`,
    `support_coordinate\t${row.support_coordinate ?? ""}`,
    `workflow_claim\t${row.workflow_claim ?? ""}`,
    `generated_locally\t${row.generated_locally === true ? "true" : "false"}`,
    `lesson_schema_change\t${row.lesson_schema_change === true ? "true" : "false"}`,
    `active_allowlist_mutation\t${row.active_allowlist_mutation === true ? "true" : "false"}`,
    `parser_frontdoor_change\t${row.parser_frontdoor_change === true ? "true" : "false"}`,
    `runtime_claim\t${row.runtime_claim === true ? "true" : "false"}`,
    `remote_save_claim\t${row.remote_save_claim === true ? "true" : "false"}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status ?? ""}`,
    `stage_count\t${row.stage_count ?? stages.length}`,
    `ready_stage_count\t${row.ready_stage_count ?? 0}`,
    `missing_stage_count\t${row.missing_stage_count ?? 0}`,
    `palette_category_count\t${row.palette_category_count ?? 0}`,
    `palette_category_ids\t${categoryIds.join("|")}`,
    `canvas_block_count\t${row.canvas_block_count ?? 0}`,
    `canvas_block_kinds\t${blockKinds.join("|")}`,
    `ddn_line_count\t${row.ddn_line_count ?? 0}`,
    `text_mode_count\t${row.text_mode_count ?? 0}`,
    `run_count\t${row.run_count ?? 0}`,
    `local_save_filename\t${row.local_save_filename ?? ""}`,
    `launch_kind\t${row.launch_kind ?? ""}`,
    `source_type\t${row.source_type ?? ""}`,
    `decode_error_count\t${row.decode_error_count ?? 0}`,
    `raw_fallback_count\t${row.raw_fallback_count ?? 0}`,
    `source_mode\t${row.source_mode ?? ""}`,
    `target_mode\t${row.target_mode ?? ""}`,
    "",
    "stage_id\tready",
    ...stageLines,
  ].join("\n");
}
