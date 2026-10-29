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

const CLOSURE_DEFS = [
  {
    id: "stage_rebase_anchor",
    source_anchor: "pack/studio_productization_stage_rebase_v1/productization_stage_rebase.detjson",
    closure_lane: "stage_anchor",
    title: "stage rebase",
    summary: "Studio productization rebase의 1/5 시작점을 마지막 closure 근거로 묶습니다.",
  },
  {
    id: "numeric_track_anchor",
    source_anchor: "pack/seamgrim_numeric_track_consolidation_v1/numeric_track_consolidation.detjson",
    closure_lane: "numeric_track",
    title: "numeric track",
    summary: "micro-slice 확장을 멈춘 numeric track consolidation을 제품화 stage chain에 고정합니다.",
  },
  {
    id: "report_workflow_anchor",
    source_anchor: "pack/studio_numeric_report_workflow_consolidation_v1/numeric_report_workflow_stage.detjson",
    closure_lane: "report_workflow",
    title: "report workflow",
    summary: "17/17 report workflow ready evidence를 current stage closure에 포함합니다.",
  },
  {
    id: "result_report_anchor",
    source_anchor: "pack/studio_numeric_result_report_consolidation_v1/numeric_result_report_stage.detjson",
    closure_lane: "result_report",
    title: "result report",
    summary: "10/10 result report stage와 5/5 result row evidence를 마지막 선행 근거로 사용합니다.",
  },
  {
    id: "post_super_long_handoff",
    source_anchor: "STUDIO_POST_SUPER_LONG_REBASE_V1",
    closure_lane: "next_handoff",
    title: "post super-long",
    summary: "V6.1 기준 전체 초장기 계획 9/18 상태를 유지한 채 후속 장기 계획 리베이스로 넘깁니다.",
  },
];

export const DEFAULT_PRODUCTIZATION_STAGE_CLOSURE_ROWS = CLOSURE_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  closure_lane: row.closure_lane,
  closure_stage_only: true,
  generated_now: false,
  product_ui_change: true,
  release_execution_claim: false,
  public_release_claim: false,
}));

function normalizeRows(rows = DEFAULT_PRODUCTIZATION_STAGE_CLOSURE_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return CLOSURE_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      closure_lane: asText(source.closure_lane, def.closure_lane),
      closure_surface: "local_studio_productization_stage_closure",
      closure_stage_only: true,
      generated_now: asBool(source.generated_now),
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      github_release_claim: false,
      runtime_claim: false,
      replay_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      parser_frontdoor_change: false,
      solver_implementation_change: false,
      benchmark_execution_claim: false,
      product_ui_change: true,
    };
  });
}

export function buildProductizationStageClosure({
  closureRows = DEFAULT_PRODUCTIZATION_STAGE_CLOSURE_ROWS,
  activeClosureId = "post_super_long_handoff",
} = {}) {
  const rows = normalizeRows(closureRows);
  const active = rows.some((row) => row.id === activeClosureId)
    ? activeClosureId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.closure_stage_only === true &&
    row.generated_now === false &&
    row.release_execution_claim === false &&
    row.public_release_claim === false &&
    row.runtime_claim === false &&
    row.solver_implementation_change === false
  ));
  const stages = [
    ["closure_row_alignment", rows.length === CLOSURE_DEFS.length],
    ["stage_rebase_anchor_present", rows.some((row) => row.id === "stage_rebase_anchor")],
    ["numeric_track_anchor_present", rows.some((row) => row.id === "numeric_track_anchor")],
    ["report_workflow_anchor_present", rows.some((row) => row.id === "report_workflow_anchor")],
    ["result_report_anchor_present", rows.some((row) => row.id === "result_report_anchor")],
    ["local_only_boundary", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_productization_stage_closure",
    schema: "ddn.studio.productization_stage_closure.v1",
    work_item: "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
    based_on: "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
    workflow_claim: "productization_stage_closure",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    product_ui_change: true,
    closure_stage_only: true,
    runtime_claim: false,
    replay_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    publication_snapshot_emit_claim: false,
    archive_generation_claim: false,
    publication_checksum_generation_claim: false,
    benchmark_execution_claim: false,
    performance_baseline_generation_claim: false,
    lts_certification_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    solver_implementation_change: false,
    stage_chain_closed: 5,
    stage_chain_total: 5,
    status: ready_stage_count === stages.length ? "productization_stage_closed" : "productization_stage_closure_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    closure_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_closure_id: active,
    closure_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 9,
      super_long_total: 18,
      super_long_percent: 50,
      current_stage_closed: 5,
      current_stage_total: 5,
      current_stage_percent: 100,
      roadmap_v2_behavior_closed: 51,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 57,
    },
    next_item: "STUDIO_POST_SUPER_LONG_REBASE_V1",
  };
}

export function formatProductizationStageClosureText(closure = {}) {
  const payload = asObject(closure);
  if (payload.schema !== "ddn.studio.productization_stage_closure.v1") {
    throw new Error("seamgrim_expected_productization_stage_closure");
  }
  const rows = asArray(payload.closure_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `stage_chain\t${payload.stage_chain_closed ?? 0}/${payload.stage_chain_total ?? 0}`,
    `super_long_percent\t${payload.progress?.super_long_percent ?? ""}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_release_claim\t${payload.public_release_claim === true ? "true" : "false"}`,
    `runtime_claim\t${payload.runtime_claim === true ? "true" : "false"}`,
    `closure_row_count\t${payload.closure_row_count ?? rows.length}`,
    "",
    "closure_id\tlane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.closure_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderProductizationStageClosure(root, closure = {}) {
  if (!root) return null;
  const payload = asObject(closure);
  const rows = asArray(payload.closure_rows);
  const activeId = asText(payload.active_closure_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.productizationStageClosureStatus = asText(payload.status, "productization_stage_closure_incomplete");
  root.innerHTML = `
    <div class="productization-closure-head">
      <div>
        <div class="productization-closure-kicker">Studio productization</div>
        <h2>Stage closure</h2>
      </div>
      <div class="productization-closure-progress" data-productization-closure-progress>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.super_long_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="productization-closure-body">
      <div class="productization-closure-list" data-productization-closure-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="productization-closure-btn${row.id === activeId ? " active" : ""}"
            data-productization-closure="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.closure_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="productization-closure-detail" data-productization-closure-detail>
        <div class="productization-closure-title" data-productization-closure-active-title>${escapeHtml(active.title)}</div>
        <p data-productization-closure-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-productization-closure-active-lane>${escapeHtml(active.closure_lane)}</dd></div>
          <div><dt>source</dt><dd data-productization-closure-active-source>${escapeHtml(active.source_anchor)}</dd></div>
          <div><dt>boundary</dt><dd>no release/runtime</dd></div>
        </dl>
        <button type="button" class="ghost" data-productization-closure-copy>closure 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (closureId) => {
    renderProductizationStageClosure(root, {
      ...payload,
      active_closure_id: closureId,
    });
  };
  root.querySelectorAll("[data-productization-closure]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-productization-closure") || "");
    });
  });
  root.querySelector("[data-productization-closure-copy]")?.addEventListener("click", async () => {
    const text = formatProductizationStageClosureText(payload);
    root.dataset.productizationStageClosureCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
