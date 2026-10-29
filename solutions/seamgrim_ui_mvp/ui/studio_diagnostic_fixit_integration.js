import {
  buildDiagnosticFixitPreview,
  formatDiagnosticFixitPreviewText,
} from "./studio_diagnostic_fixit_preview.js";

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
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

function normalizeBoundary(boundary = {}) {
  const row = asObject(boundary);
  return {
    auto_apply: asBool(row.auto_apply ?? row.autoApply),
    file_write: asBool(row.file_write ?? row.fileWrite),
    lsp_protocol_change: asBool(row.lsp_protocol_change ?? row.lspProtocolChange),
    parser_frontdoor_change: asBool(row.parser_frontdoor_change ?? row.parserFrontdoorChange),
    runtime_claim: asBool(row.runtime_claim ?? row.runtimeClaim),
    lesson_schema_change: asBool(row.lesson_schema_change ?? row.lessonSchemaChange),
    active_allowlist_mutation: asBool(row.active_allowlist_mutation ?? row.activeAllowlistMutation),
  };
}

export function buildDiagnosticFixitIntegration({
  sourceText = "",
  diagnostics = [],
  boundary = {},
} = {}) {
  const preview = buildDiagnosticFixitPreview({ sourceText, diagnostics });
  const previewText = String(preview.preview_text ?? "");
  const diffText = String(preview.diff_text ?? "");
  const formattedText = formatDiagnosticFixitPreviewText(preview);
  const normalizedBoundary = normalizeBoundary(boundary);
  const stages = [
    ["diagnostic_preview", preview.__종류 === "studio_diagnostic_fixit_preview" && preview.diagnostic_count > 0],
    ["patch_candidates", asCount(preview.fixit_count) > 0],
    ["preview_text", previewText.length > 0 && previewText !== String(sourceText ?? "")],
    ["diff_text", diffText.includes("--- original") && diffText.includes("+++ preview")],
    ["unsupported_rows", asCount(preview.unsupported_count) > 0],
    ["formatter_text", formattedText.includes("진단\t") && formattedText.includes("수정후보\t")],
    ["no_auto_apply_boundary", preview.auto_apply === false && normalizedBoundary.auto_apply === false],
    ["no_file_write_boundary", normalizedBoundary.file_write === false],
    [
      "no_surface_expansion",
      normalizedBoundary.lsp_protocol_change === false
        && normalizedBoundary.parser_frontdoor_change === false
        && normalizedBoundary.runtime_claim === false
        && normalizedBoundary.lesson_schema_change === false
        && normalizedBoundary.active_allowlist_mutation === false,
    ],
  ].map(([stageId, ready]) => ({
    stage_id: stageId,
    ready: ready === true,
  }));
  const readyStageCount = stages.filter((stage) => stage.ready).length;
  const ready = readyStageCount === stages.length;
  return {
    schema: "seamgrim.diagnostic_fixit_integration.v1",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    workflow_claim: "diagnostic_fixit_integration",
    preview_only: true,
    auto_apply: false,
    file_write: false,
    lsp_protocol_change: false,
    parser_frontdoor_change: false,
    runtime_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    replay_claim: false,
    status: ready ? "diagnostic_fixit_ready" : "diagnostic_fixit_incomplete",
    tone: ready ? "success" : "warning",
    stage_count: stages.length,
    ready_stage_count: readyStageCount,
    missing_stage_count: stages.length - readyStageCount,
    diagnostic_count: asCount(preview.diagnostic_count),
    fixit_count: asCount(preview.fixit_count),
    unsupported_count: asCount(preview.unsupported_count),
    preview_text_line_count: countLines(previewText),
    diff_text_line_count: countLines(diffText),
    formatter_text_line_count: countLines(formattedText),
    stages,
    preview,
    formatted_text: formattedText,
    boundary: normalizedBoundary,
  };
}

export function formatDiagnosticFixitIntegrationText(workflow = {}) {
  const row = asObject(workflow);
  if (row.schema !== "seamgrim.diagnostic_fixit_integration.v1") {
    throw new Error("seamgrim_expected_diagnostic_fixit_integration");
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
    `preview_only\t${row.preview_only === true ? "true" : "false"}`,
    `auto_apply\t${row.auto_apply === true ? "true" : "false"}`,
    `file_write\t${row.file_write === true ? "true" : "false"}`,
    `lsp_protocol_change\t${row.lsp_protocol_change === true ? "true" : "false"}`,
    `parser_frontdoor_change\t${row.parser_frontdoor_change === true ? "true" : "false"}`,
    `runtime_claim\t${row.runtime_claim === true ? "true" : "false"}`,
    `lesson_schema_change\t${row.lesson_schema_change === true ? "true" : "false"}`,
    `active_allowlist_mutation\t${row.active_allowlist_mutation === true ? "true" : "false"}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status ?? ""}`,
    `stage_count\t${row.stage_count ?? stages.length}`,
    `ready_stage_count\t${row.ready_stage_count ?? 0}`,
    `missing_stage_count\t${row.missing_stage_count ?? 0}`,
    `diagnostic_count\t${row.diagnostic_count ?? 0}`,
    `fixit_count\t${row.fixit_count ?? 0}`,
    `unsupported_count\t${row.unsupported_count ?? 0}`,
    `preview_text_line_count\t${row.preview_text_line_count ?? 0}`,
    `diff_text_line_count\t${row.diff_text_line_count ?? 0}`,
    `formatter_text_line_count\t${row.formatter_text_line_count ?? 0}`,
    "",
    "stage_id\tready",
    ...stageLines,
  ].join("\n");
}
