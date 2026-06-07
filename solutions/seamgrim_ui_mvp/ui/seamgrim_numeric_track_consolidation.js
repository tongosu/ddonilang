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

const CONSOLIDATION_DEFS = [
  {
    id: "productization_rebase_anchor",
    source_anchor: "pack/studio_productization_stage_rebase_v1/productization_stage_rebase.detjson",
    consolidation_lane: "stage_handoff",
    title: "제품화 rebase 인계",
    summary: "직전 제품화 stage rebase의 다음 후보 선택을 numeric track consolidation으로 연결합니다.",
  },
  {
    id: "numeric_report_workflow_gate",
    source_anchor: "tests/studio_numeric_report_workflow_consolidation_runner.mjs",
    consolidation_lane: "report_workflow",
    title: "report workflow gate",
    summary: "긴 history/report/table/status/badge 체인을 짧은 report workflow gate로 묶습니다.",
  },
  {
    id: "numeric_result_report_gate",
    source_anchor: "tests/studio_numeric_result_report_consolidation_runner.mjs",
    consolidation_lane: "result_report",
    title: "result report gate",
    summary: "run result, history, summary, timeline, compare, report workflow를 하나의 gate로 묶습니다.",
  },
  {
    id: "legacy_runner_audit",
    source_anchor: "tests/seamgrim_numeric_track*_runner.mjs",
    consolidation_lane: "micro_slice_audit",
    title: "legacy runner audit",
    summary: "28개 legacy runner 중 16개가 60자 초과, 2개가 100자 초과임을 제품 UI에 표시합니다.",
  },
  {
    id: "browse_detail_dataset_guard",
    source_anchor: "solutions/seamgrim_ui_mvp/ui/screens/browse.js",
    consolidation_lane: "baseline_guard",
    title: "Browse detail guard",
    summary: "dataset 없는 detail panel에서도 numeric track 상태 기록이 실패하지 않게 유지합니다.",
  },
];

export const DEFAULT_NUMERIC_TRACK_CONSOLIDATION_ROWS = CONSOLIDATION_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  consolidation_lane: row.consolidation_lane,
  consolidation_only: true,
  generated_now: false,
  product_ui_change: true,
  runtime_claim: false,
  replay_claim: false,
}));

function normalizeRows(rows = DEFAULT_NUMERIC_TRACK_CONSOLIDATION_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return CONSOLIDATION_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      consolidation_lane: asText(source.consolidation_lane, def.consolidation_lane),
      consolidation_surface: "local_seamgrim_numeric_track_consolidation",
      consolidation_only: true,
      generated_now: asBool(source.generated_now),
      new_export_wrapper_claim: false,
      new_long_runner_claim: false,
      runtime_claim: false,
      replay_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      parser_frontdoor_change: false,
      product_ui_change: true,
    };
  });
}

export function buildSeamgrimNumericTrackConsolidation({
  consolidationRows = DEFAULT_NUMERIC_TRACK_CONSOLIDATION_ROWS,
  activeConsolidationId = "legacy_runner_audit",
} = {}) {
  const rows = normalizeRows(consolidationRows);
  const active = rows.some((row) => row.id === activeConsolidationId)
    ? activeConsolidationId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.consolidation_only === true &&
    row.generated_now === false &&
    row.runtime_claim === false &&
    row.replay_claim === false &&
    row.new_long_runner_claim === false
  ));
  const stages = [
    ["consolidation_row_alignment", rows.length === CONSOLIDATION_DEFS.length],
    ["report_workflow_gate_present", rows.some((row) => row.id === "numeric_report_workflow_gate")],
    ["result_report_gate_present", rows.some((row) => row.id === "numeric_result_report_gate")],
    ["legacy_runner_audit_present", rows.some((row) => row.id === "legacy_runner_audit")],
    ["browse_detail_guard_present", rows.some((row) => row.id === "browse_detail_dataset_guard")],
    ["local_only_boundary", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "seamgrim_numeric_track_consolidation",
    schema: "seamgrim.numeric_track_consolidation.v1",
    work_item: "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
    based_on: "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
    track_id: "studio_numeric_curriculum_track_v1",
    primary_coordinate: "마-3",
    workflow_claim: "numeric_track_micro_slice_consolidation",
    preferred_gates: [
      "tests/studio_numeric_report_workflow_consolidation_runner.mjs",
      "tests/studio_numeric_result_report_consolidation_runner.mjs",
    ],
    legacy_numeric_runner_count: 28,
    legacy_numeric_runner_over_60: 16,
    legacy_numeric_runner_over_100: 2,
    generated_locally: true,
    product_ui_change: true,
    consolidation_only: true,
    runtime_claim: false,
    replay_claim: false,
    new_export_wrapper_claim: false,
    new_long_runner_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    status: ready_stage_count === stages.length ? "numeric_track_consolidated" : "numeric_track_consolidation_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    consolidation_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_consolidation_id: active,
    consolidation_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 2,
      current_stage_total: 5,
      current_stage_percent: 40,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1",
  };
}

export function formatSeamgrimNumericTrackConsolidationText(consolidation = {}) {
  const payload = asObject(consolidation);
  if (payload.schema !== "seamgrim.numeric_track_consolidation.v1") {
    throw new Error("seamgrim_expected_numeric_track_consolidation");
  }
  const rows = asArray(payload.consolidation_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `legacy_numeric_runner_count\t${payload.legacy_numeric_runner_count ?? 0}`,
    `legacy_numeric_runner_over_60\t${payload.legacy_numeric_runner_over_60 ?? 0}`,
    `legacy_numeric_runner_over_100\t${payload.legacy_numeric_runner_over_100 ?? 0}`,
    `new_export_wrapper_claim\t${payload.new_export_wrapper_claim === true ? "true" : "false"}`,
    `new_long_runner_claim\t${payload.new_long_runner_claim === true ? "true" : "false"}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `consolidation_row_count\t${payload.consolidation_row_count ?? rows.length}`,
    "",
    "consolidation_id\tlane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.consolidation_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderSeamgrimNumericTrackConsolidation(root, consolidation = {}) {
  if (!root) return null;
  const payload = asObject(consolidation);
  const rows = asArray(payload.consolidation_rows);
  const activeId = asText(payload.active_consolidation_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.numericTrackConsolidationStatus = asText(payload.status, "numeric_track_consolidation_incomplete");
  root.innerHTML = `
    <div class="numeric-consolidation-head">
      <div>
        <div class="numeric-consolidation-kicker">Numeric track</div>
        <h2>Micro-slice consolidation</h2>
      </div>
      <div class="numeric-consolidation-progress" data-numeric-consolidation-progress>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(String(payload.legacy_numeric_runner_over_100 ?? 0))} over 100 chars</span>
      </div>
    </div>
    <div class="numeric-consolidation-body">
      <div class="numeric-consolidation-list" data-numeric-consolidation-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="numeric-consolidation-btn${row.id === activeId ? " active" : ""}"
            data-numeric-consolidation="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.consolidation_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="numeric-consolidation-detail" data-numeric-consolidation-detail>
        <div class="numeric-consolidation-title" data-numeric-consolidation-active-title>${escapeHtml(active.title)}</div>
        <p data-numeric-consolidation-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-numeric-consolidation-active-lane>${escapeHtml(active.consolidation_lane)}</dd></div>
          <div><dt>source</dt><dd data-numeric-consolidation-active-source>${escapeHtml(active.source_anchor)}</dd></div>
          <div><dt>boundary</dt><dd>no new long runner</dd></div>
        </dl>
        <button type="button" class="ghost" data-numeric-consolidation-copy>consolidation 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderSeamgrimNumericTrackConsolidation(root, {
      ...payload,
      active_consolidation_id: rowId,
    });
  };
  root.querySelectorAll("[data-numeric-consolidation]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-numeric-consolidation") || "");
    });
  });
  root.querySelector("[data-numeric-consolidation-copy]")?.addEventListener("click", async () => {
    const text = formatSeamgrimNumericTrackConsolidationText(payload);
    root.dataset.numericTrackConsolidationCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
