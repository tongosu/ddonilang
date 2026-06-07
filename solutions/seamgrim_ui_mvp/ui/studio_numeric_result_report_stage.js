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

const RESULT_STAGE_DEFS = [
  {
    id: "report_workflow_anchor",
    source_anchor: "pack/studio_numeric_report_workflow_consolidation_v1/numeric_report_workflow_stage.detjson",
    workflow_lane: "stage_handoff",
    title: "report workflow 인계",
    summary: "직전 numeric report workflow stage의 17/17 ready evidence를 result report stage로 연결합니다.",
  },
  {
    id: "result_report_gate",
    source_anchor: "tests/studio_numeric_result_report_consolidation_runner.mjs",
    workflow_lane: "result_report_gate",
    title: "result report gate",
    summary: "result link, history, timeline, compare, report workflow를 하나의 result report artifact로 닫습니다.",
  },
  {
    id: "evidence_pack_rollup",
    source_anchor: "pack/studio_numeric_result_report_consolidation_v1/contract.detjson",
    workflow_lane: "evidence_rollup",
    title: "evidence pack rollup",
    summary: "3개 numeric result와 3개 evidence pack을 solver 변경 없이 로컬 제품 증거로 묶습니다.",
  },
  {
    id: "copy_text_export_gate",
    source_anchor: "formatNumericResultReportConsolidationText",
    workflow_lane: "text_export",
    title: "deterministic text export",
    summary: "새 export wrapper 없이 기존 deterministic formatter를 제품 UI copy 동작으로 노출합니다.",
  },
  {
    id: "final_stage_handoff",
    source_anchor: "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
    workflow_lane: "next_handoff",
    title: "final stage 인계",
    summary: "현재 Studio productization rebase의 마지막 닫힘 항목을 stage closure로 고정합니다.",
  },
];

export const DEFAULT_NUMERIC_RESULT_REPORT_STAGE_ROWS = RESULT_STAGE_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  workflow_lane: row.workflow_lane,
  result_report_stage_only: true,
  generated_now: false,
  product_ui_change: true,
  replay_claim: false,
  runtime_claim: false,
}));

function normalizeRows(rows = DEFAULT_NUMERIC_RESULT_REPORT_STAGE_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return RESULT_STAGE_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      workflow_lane: asText(source.workflow_lane, def.workflow_lane),
      workflow_surface: "local_studio_numeric_result_report_stage",
      result_report_stage_only: true,
      generated_now: asBool(source.generated_now),
      new_export_wrapper_claim: false,
      replay_claim: false,
      runtime_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      parser_frontdoor_change: false,
      solver_implementation_change: false,
      product_ui_change: true,
    };
  });
}

export function buildNumericResultReportStage({
  resultRows = DEFAULT_NUMERIC_RESULT_REPORT_STAGE_ROWS,
  activeResultId = "result_report_gate",
} = {}) {
  const rows = normalizeRows(resultRows);
  const active = rows.some((row) => row.id === activeResultId)
    ? activeResultId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.result_report_stage_only === true &&
    row.generated_now === false &&
    row.replay_claim === false &&
    row.runtime_claim === false &&
    row.new_export_wrapper_claim === false &&
    row.solver_implementation_change === false
  ));
  const stages = [
    ["result_row_alignment", rows.length === RESULT_STAGE_DEFS.length],
    ["report_workflow_anchor_present", rows.some((row) => row.id === "report_workflow_anchor")],
    ["result_report_gate_present", rows.some((row) => row.id === "result_report_gate")],
    ["evidence_pack_rollup_present", rows.some((row) => row.id === "evidence_pack_rollup")],
    ["text_export_present", rows.some((row) => row.id === "copy_text_export_gate")],
    ["local_only_boundary", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_numeric_result_report_stage",
    schema: "ddn.studio.numeric_result_report_stage.v1",
    work_item: "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
    based_on: "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1",
    track_id: "studio_numeric_curriculum_track_v1",
    workflow_schema: "seamgrim.numeric_result_report_consolidation.v1",
    workflow_claim: "numeric_result_report_consolidation",
    primary_coordinate: "마-3",
    support_coordinate: "다-2",
    product_ui_change: true,
    result_report_stage_only: true,
    runtime_claim: false,
    replay_claim: false,
    new_export_wrapper_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    solver_implementation_change: false,
    result_count: 3,
    pair_count: 2,
    evidence_pack_count: 3,
    report_workflow_ready_stage_count: 17,
    report_workflow_stage_count: 17,
    result_report_stage_count: 10,
    result_report_ready_stage_count: 10,
    status: ready_stage_count === stages.length ? "numeric_result_report_stage_ready" : "numeric_result_report_stage_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    result_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_result_id: active,
    result_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 4,
      current_stage_total: 5,
      current_stage_percent: 80,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
  };
}

export function formatNumericResultReportStageText(stage = {}) {
  const payload = asObject(stage);
  if (payload.schema !== "ddn.studio.numeric_result_report_stage.v1") {
    throw new Error("seamgrim_expected_numeric_result_report_stage");
  }
  const rows = asArray(payload.result_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_schema\t${payload.workflow_schema ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `result_count\t${payload.result_count ?? 0}`,
    `pair_count\t${payload.pair_count ?? 0}`,
    `evidence_pack_count\t${payload.evidence_pack_count ?? 0}`,
    `report_workflow_stage_count\t${payload.report_workflow_stage_count ?? 0}`,
    `result_report_stage_count\t${payload.result_report_stage_count ?? 0}`,
    `new_export_wrapper_claim\t${payload.new_export_wrapper_claim === true ? "true" : "false"}`,
    `replay_claim\t${payload.replay_claim === true ? "true" : "false"}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `result_row_count\t${payload.result_row_count ?? rows.length}`,
    "",
    "result_id\tlane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.workflow_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderNumericResultReportStage(root, stage = {}) {
  if (!root) return null;
  const payload = asObject(stage);
  const rows = asArray(payload.result_rows);
  const activeId = asText(payload.active_result_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.numericResultReportStageStatus = asText(payload.status, "numeric_result_report_stage_incomplete");
  root.innerHTML = `
    <div class="numeric-result-stage-head">
      <div>
        <div class="numeric-result-stage-kicker">Numeric result</div>
        <h2>Result report consolidation</h2>
      </div>
      <div class="numeric-result-stage-progress" data-numeric-result-stage-progress>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(String(payload.result_report_ready_stage_count ?? 0))}/${escapeHtml(String(payload.result_report_stage_count ?? 0))} result</span>
        <span>${escapeHtml(String(payload.report_workflow_ready_stage_count ?? 0))}/${escapeHtml(String(payload.report_workflow_stage_count ?? 0))} workflow</span>
      </div>
    </div>
    <div class="numeric-result-stage-body">
      <div class="numeric-result-stage-list" data-numeric-result-stage-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="numeric-result-stage-btn${row.id === activeId ? " active" : ""}"
            data-numeric-result-stage="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.workflow_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="numeric-result-stage-detail" data-numeric-result-stage-detail>
        <div class="numeric-result-stage-title" data-numeric-result-stage-active-title>${escapeHtml(active.title)}</div>
        <p data-numeric-result-stage-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-numeric-result-stage-active-lane>${escapeHtml(active.workflow_lane)}</dd></div>
          <div><dt>source</dt><dd data-numeric-result-stage-active-source>${escapeHtml(active.source_anchor)}</dd></div>
          <div><dt>boundary</dt><dd>no replay/runtime</dd></div>
        </dl>
        <button type="button" class="ghost" data-numeric-result-stage-copy>result stage 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderNumericResultReportStage(root, {
      ...payload,
      active_result_id: rowId,
    });
  };
  root.querySelectorAll("[data-numeric-result-stage]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-numeric-result-stage") || "");
    });
  });
  root.querySelector("[data-numeric-result-stage-copy]")?.addEventListener("click", async () => {
    const text = formatNumericResultReportStageText(payload);
    root.dataset.numericResultReportStageCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
