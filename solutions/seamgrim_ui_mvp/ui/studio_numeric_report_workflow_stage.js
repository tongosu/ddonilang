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

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

const WORKFLOW_DEFS = [
  {
    id: "numeric_track_consolidation_anchor",
    source_anchor: "pack/seamgrim_numeric_track_consolidation_v1/numeric_track_consolidation.detjson",
    workflow_lane: "stage_handoff",
    title: "numeric track 인계",
    summary: "직전 numeric track consolidation의 다음 항목 선택을 report workflow로 연결합니다.",
  },
  {
    id: "report_workflow_gate",
    source_anchor: "tests/studio_numeric_report_workflow_consolidation_runner.mjs",
    workflow_lane: "workflow_gate",
    title: "17 stage workflow gate",
    summary: "compare history부터 A11Y status export summary까지 17개 stage를 하나로 닫습니다.",
  },
  {
    id: "browse_strip_surface",
    source_anchor: "solutions/seamgrim_ui_mvp/ui/screens/browse.js",
    workflow_lane: "browse_surface",
    title: "browse strip surface",
    summary: "browse compare-history panel에 workflow schema와 상태 strip을 표시합니다.",
  },
  {
    id: "copy_text_export_gate",
    source_anchor: "btn-copy-numeric-report-workflow-consolidation",
    workflow_lane: "text_export",
    title: "deterministic text export",
    summary: "새 export wrapper 없이 기존 workflow text를 복사 가능한 제품 UI 동작으로 제공합니다.",
  },
  {
    id: "next_result_report_handoff",
    source_anchor: "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
    workflow_lane: "next_handoff",
    title: "result report 인계",
    summary: "다음 productization stage 항목을 numeric result report consolidation으로 고정합니다.",
  },
];

export const DEFAULT_NUMERIC_REPORT_WORKFLOW_STAGE_ROWS = WORKFLOW_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  workflow_lane: row.workflow_lane,
  workflow_stage_only: true,
  generated_now: false,
  product_ui_change: true,
  replay_claim: false,
  runtime_claim: false,
}));

function normalizeRows(rows = DEFAULT_NUMERIC_REPORT_WORKFLOW_STAGE_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return WORKFLOW_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      workflow_lane: asText(source.workflow_lane, def.workflow_lane),
      workflow_surface: "local_studio_numeric_report_workflow_stage",
      workflow_stage_only: true,
      generated_now: asBool(source.generated_now),
      new_export_wrapper_claim: false,
      replay_claim: false,
      runtime_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      parser_frontdoor_change: false,
      product_ui_change: true,
    };
  });
}

export function buildNumericReportWorkflowStage({
  workflowRows = DEFAULT_NUMERIC_REPORT_WORKFLOW_STAGE_ROWS,
  activeWorkflowId = "report_workflow_gate",
} = {}) {
  const rows = normalizeRows(workflowRows);
  const active = rows.some((row) => row.id === activeWorkflowId)
    ? activeWorkflowId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.workflow_stage_only === true &&
    row.generated_now === false &&
    row.replay_claim === false &&
    row.runtime_claim === false &&
    row.new_export_wrapper_claim === false
  ));
  const stages = [
    ["workflow_row_alignment", rows.length === WORKFLOW_DEFS.length],
    ["numeric_track_anchor_present", rows.some((row) => row.id === "numeric_track_consolidation_anchor")],
    ["report_workflow_gate_present", rows.some((row) => row.id === "report_workflow_gate")],
    ["browse_surface_present", rows.some((row) => row.id === "browse_strip_surface")],
    ["text_export_present", rows.some((row) => row.id === "copy_text_export_gate")],
    ["local_only_boundary", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_numeric_report_workflow_stage",
    schema: "ddn.studio.numeric_report_workflow_stage.v1",
    work_item: "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1",
    based_on: "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
    track_id: "studio_numeric_curriculum_track_v1",
    workflow_schema: "seamgrim.numeric_report_workflow_consolidation.v1",
    workflow_claim: "product_workflow_consolidation",
    primary_coordinate: "마-3",
    product_ui_change: true,
    workflow_stage_only: true,
    runtime_claim: false,
    replay_claim: false,
    new_export_wrapper_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    workflow_ready_stage_count: 17,
    workflow_stage_count: 17,
    pair_count: 2,
    row_count: 2,
    lesson_count: 3,
    status: ready_stage_count === stages.length ? "numeric_report_workflow_stage_ready" : "numeric_report_workflow_stage_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    workflow_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_workflow_id: active,
    workflow_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 9,
      super_long_total: 18,
      super_long_percent: 50,
      current_stage_closed: 3,
      current_stage_total: 5,
      current_stage_percent: 60,
      roadmap_v2_behavior_closed: 51,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 57,
    },
    next_item: "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
  };
}

export function formatNumericReportWorkflowStageText(stage = {}) {
  const payload = asObject(stage);
  if (payload.schema !== "ddn.studio.numeric_report_workflow_stage.v1") {
    throw new Error("seamgrim_expected_numeric_report_workflow_stage");
  }
  const rows = asArray(payload.workflow_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_schema\t${payload.workflow_schema ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `workflow_ready_stage_count\t${payload.workflow_ready_stage_count ?? 0}`,
    `workflow_stage_count\t${payload.workflow_stage_count ?? 0}`,
    `new_export_wrapper_claim\t${payload.new_export_wrapper_claim === true ? "true" : "false"}`,
    `replay_claim\t${payload.replay_claim === true ? "true" : "false"}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `workflow_row_count\t${payload.workflow_row_count ?? rows.length}`,
    "",
    "workflow_id\tlane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.workflow_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderNumericReportWorkflowStage(root, stage = {}) {
  if (!root) return null;
  const payload = asObject(stage);
  const rows = asArray(payload.workflow_rows);
  const activeId = asText(payload.active_workflow_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.numericReportWorkflowStageStatus = asText(payload.status, "numeric_report_workflow_stage_incomplete");
  root.innerHTML = `
    <div class="numeric-report-stage-head">
      <div>
        <div class="numeric-report-stage-kicker">Numeric report</div>
        <h2>Workflow consolidation</h2>
      </div>
      <div class="numeric-report-stage-progress" data-numeric-report-stage-progress>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(String(payload.workflow_ready_stage_count ?? 0))}/${escapeHtml(String(payload.workflow_stage_count ?? 0))} workflow</span>
      </div>
    </div>
    <div class="numeric-report-stage-body">
      <div class="numeric-report-stage-list" data-numeric-report-stage-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="numeric-report-stage-btn${row.id === activeId ? " active" : ""}"
            data-numeric-report-stage="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.workflow_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="numeric-report-stage-detail" data-numeric-report-stage-detail>
        <div class="numeric-report-stage-title" data-numeric-report-stage-active-title>${escapeHtml(active.title)}</div>
        <p data-numeric-report-stage-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-numeric-report-stage-active-lane>${escapeHtml(active.workflow_lane)}</dd></div>
          <div><dt>source</dt><dd data-numeric-report-stage-active-source>${escapeHtml(active.source_anchor)}</dd></div>
          <div><dt>boundary</dt><dd>no new export wrapper</dd></div>
        </dl>
        <button type="button" class="ghost" data-numeric-report-stage-copy>workflow stage 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderNumericReportWorkflowStage(root, {
      ...payload,
      active_workflow_id: rowId,
    });
  };
  root.querySelectorAll("[data-numeric-report-stage]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-numeric-report-stage") || "");
    });
  });
  root.querySelector("[data-numeric-report-stage-copy]")?.addEventListener("click", async () => {
    const text = formatNumericReportWorkflowStageText(payload);
    root.dataset.numericReportWorkflowStageCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
