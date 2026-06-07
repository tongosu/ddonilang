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

const REBASE_DEFS = [
  {
    id: "operations_preview_closure_anchor",
    source_anchor: "pack/studio_operations_preview_stage_closure_v1/operations_preview_stage_closure.detjson",
    coordinate: "마-3",
    rebase_lane: "stage_handoff",
    title: "Operations preview 인계",
    summary: "직전 Studio operations preview stage closure를 새 제품화 stage의 선행 근거로 고정합니다.",
  },
  {
    id: "micro_slice_consolidation_priority",
    source_anchor: "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
    coordinate: "마-3",
    rebase_lane: "micro_slice_consolidation",
    title: "micro-slice 통합 우선",
    summary: "긴 numeric runner/export 계열을 새 slice 추가보다 통합 대상으로 먼저 둡니다.",
  },
  {
    id: "productization_stage_denominator",
    source_anchor: "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
    coordinate: "마-3",
    rebase_lane: "stage_denominator",
    title: "제품화 stage 분모",
    summary: "현재 제품화 stage를 5개 항목 중 1개 닫힘으로 시작합니다.",
  },
  {
    id: "release_boundary_guard",
    source_anchor: "AWAIT_EXPLICIT_RELEASE_APPROVAL",
    coordinate: "타-3",
    rebase_lane: "release_boundary",
    title: "릴리스 경계",
    summary: "release approval, execution, public upload, registry publish를 계속 막습니다.",
  },
  {
    id: "next_item_selection",
    source_anchor: "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
    coordinate: "마-3",
    rebase_lane: "next_selection",
    title: "다음 구현 선택",
    summary: "다음 제품 동작 후보를 numeric track consolidation으로 선택합니다.",
  },
];

export const DEFAULT_PRODUCTIZATION_STAGE_REBASE_ROWS = REBASE_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  coordinate: row.coordinate,
  rebase_lane: row.rebase_lane,
  stage_rebase_only: true,
  generated_now: false,
  product_ui_change: true,
  release_execution_claim: false,
  public_upload_claim: false,
}));

function normalizeRebaseRows(rebaseRows = DEFAULT_PRODUCTIZATION_STAGE_REBASE_ROWS) {
  const rowById = new Map(asArray(rebaseRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return REBASE_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      coordinate: asText(source.coordinate, def.coordinate),
      rebase_lane: asText(source.rebase_lane, def.rebase_lane),
      rebase_surface: "local_studio_productization_stage_rebase",
      stage_rebase_only: true,
      generated_now: asBool(source.generated_now),
      new_automatic_queue_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      benchmark_execution_claim: false,
      performance_baseline_generation_claim: false,
      active_allowlist_mutation: false,
      lesson_schema_change: false,
      product_ui_change: true,
    };
  });
}

export function buildProductizationStageRebase({
  rebaseRows = DEFAULT_PRODUCTIZATION_STAGE_REBASE_ROWS,
  activeRebaseId = "next_item_selection",
} = {}) {
  const rows = normalizeRebaseRows(rebaseRows);
  const active = rows.some((row) => row.id === activeRebaseId)
    ? activeRebaseId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.stage_rebase_only === true &&
    row.generated_now === false &&
    row.release_execution_claim === false &&
    row.public_upload_claim === false
  ));
  const stages = [
    ["rebase_row_alignment", rows.length === REBASE_DEFS.length],
    ["operations_preview_anchor", rows.some((row) => row.id === "operations_preview_closure_anchor")],
    ["micro_slice_consolidation_selected", rows.some((row) => row.id === "micro_slice_consolidation_priority")],
    ["release_boundary_guarded", rows.some((row) => row.id === "release_boundary_guard")],
    ["local_only_boundary", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_productization_stage_rebase",
    schema: "ddn.studio.productization_stage_rebase.v1",
    work_item: "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
    based_on: "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    workflow_claim: "productization_stage_rebase",
    selected_next_item: "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
    generated_locally: true,
    product_ui_change: true,
    stage_rebase_only: true,
    runtime_claim: false,
    new_automatic_queue_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    publication_snapshot_emit_claim: false,
    archive_generation_claim: false,
    publication_checksum_generation_claim: false,
    artifact_signing_claim: false,
    benchmark_execution_claim: false,
    performance_baseline_generation_claim: false,
    performance_baseline_publication_claim: false,
    lts_certification_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    status: ready_stage_count === stages.length ? "productization_stage_rebased" : "productization_stage_rebase_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    rebase_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_rebase_id: active,
    rebase_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 1,
      current_stage_total: 5,
      current_stage_percent: 20,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
  };
}

export function formatProductizationStageRebaseText(rebase = {}) {
  const payload = asObject(rebase);
  if (payload.schema !== "ddn.studio.productization_stage_rebase.v1") {
    throw new Error("seamgrim_expected_productization_stage_rebase");
  }
  const rows = asArray(payload.rebase_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `selected_next_item\t${payload.selected_next_item ?? ""}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `rebase_row_count\t${payload.rebase_row_count ?? rows.length}`,
    "",
    "rebase_id\tcoordinate\trebase_lane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.coordinate,
      row.rebase_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderProductizationStageRebase(root, rebase = {}) {
  if (!root) return null;
  const payload = asObject(rebase);
  const rows = asArray(payload.rebase_rows);
  const activeId = asText(payload.active_rebase_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.productizationStageRebaseStatus = asText(payload.status, "productization_stage_rebase_incomplete");
  root.innerHTML = `
    <div class="productization-rebase-head">
      <div>
        <div class="productization-rebase-kicker">Studio productization</div>
        <h2>Stage rebase</h2>
      </div>
      <div class="productization-rebase-progress" data-productization-rebase-progress>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(payload.selected_next_item || "")}</span>
      </div>
    </div>
    <div class="productization-rebase-body">
      <div class="productization-rebase-list" data-productization-rebase-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="productization-rebase-btn${row.id === activeId ? " active" : ""}"
            data-productization-rebase="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.rebase_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="productization-rebase-detail" data-productization-rebase-detail>
        <div class="productization-rebase-title" data-productization-rebase-active-title>${escapeHtml(active.title)}</div>
        <p data-productization-rebase-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>coordinate</dt><dd data-productization-rebase-active-coordinate>${escapeHtml(active.coordinate)}</dd></div>
          <div><dt>lane</dt><dd data-productization-rebase-active-lane>${escapeHtml(active.rebase_lane)}</dd></div>
          <div><dt>source</dt><dd data-productization-rebase-active-source>${escapeHtml(active.source_anchor)}</dd></div>
        </dl>
        <button type="button" class="ghost" data-productization-rebase-copy>rebase 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rebaseId) => {
    renderProductizationStageRebase(root, {
      ...payload,
      active_rebase_id: rebaseId,
    });
  };
  root.querySelectorAll("[data-productization-rebase]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-productization-rebase") || "");
    });
  });
  root.querySelector("[data-productization-rebase-copy]")?.addEventListener("click", async () => {
    const text = formatProductizationStageRebaseText(payload);
    root.dataset.productizationStageRebaseCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
