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

const QUEUE_DEFS = [
  {
    id: "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
    coordinate: "마-3",
    status: "closed",
    claim: "explicit next development denominator lock",
  },
  {
    id: "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
    coordinate: "하-3",
    status: "next",
    claim: "local teacher feedback preview surface only",
  },
  {
    id: "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
    coordinate: "하-3",
    status: "planned",
    claim: "local classroom operations panel preview only",
  },
  {
    id: "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
    coordinate: "타-3",
    status: "planned",
    claim: "local benchmark baseline snapshot only",
  },
  {
    id: "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
    coordinate: "마-3",
    status: "planned",
    claim: "approval-safe release review dashboard only",
  },
  {
    id: "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
    coordinate: "마-3",
    status: "planned",
    claim: "local lesson publication review surface only",
  },
  {
    id: "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
    coordinate: "타-3",
    status: "planned",
    claim: "local regression gate matrix only",
  },
  {
    id: "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
    coordinate: "마-3",
    status: "planned",
    claim: "next queue coordinate lock only",
  },
];

export const DEFAULT_MA3_NEXT_DEVELOPMENT_QUEUE_ROWS = QUEUE_DEFS.map((row) => ({
  id: row.id,
  coordinate: row.coordinate,
  status: row.status,
  claim: row.claim,
  queue_rebase_only: true,
  generated_now: false,
  runtime_claim: false,
  release_execution_claim: false,
  public_upload_claim: false,
}));

function normalizeQueueRows(queueRows = DEFAULT_MA3_NEXT_DEVELOPMENT_QUEUE_ROWS) {
  const byId = new Map(asArray(queueRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return QUEUE_DEFS.map((def, index) => {
    const source = byId.get(def.id) || {};
    return {
      id: def.id,
      coordinate: asText(source.coordinate, def.coordinate),
      status: asText(source.status, def.status),
      claim: asText(source.claim, def.claim),
      order: index + 1,
      queue_surface: "local_ma3_next_development_queue_rebase",
      queue_rebase_only: true,
      generated_now: asBool(source.generated_now),
      product_ui_change: true,
      runtime_claim: false,
      parser_frontdoor_change: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      new_automatic_queue_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      github_release_claim: false,
      benchmark_execution_claim: false,
      performance_baseline_generation_claim: false,
      performance_baseline_publication_claim: false,
      lts_certification_claim: false,
      cloud_sync_claim: false,
      account_setup_claim: false,
      permission_system_claim: false,
    };
  });
}

export function buildMa3NextDevelopmentQueueRebase({
  queueRows = DEFAULT_MA3_NEXT_DEVELOPMENT_QUEUE_ROWS,
  activeQueueItemId = "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
} = {}) {
  const rows = normalizeQueueRows(queueRows);
  const active = rows.some((row) => row.id === activeQueueItemId)
    ? activeQueueItemId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.queue_rebase_only === true &&
    row.generated_now === false &&
    row.runtime_claim === false &&
    row.release_execution_claim === false &&
    row.public_upload_claim === false &&
    row.benchmark_execution_claim === false
  ));
  const stages = [
    ["queue_row_alignment", rows.length === QUEUE_DEFS.length],
    ["explicit_selection_consumed", true],
    ["closed_rebase_item_present", rows.some((row) => row.id === "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1" && row.status === "closed")],
    ["next_teacher_feedback_selected", rows.some((row) => row.id === "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1" && row.status === "next")],
    ["ma3_coordinate_present", rows.some((row) => row.coordinate === "마-3")],
    ["local_only_boundary", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_ma3_next_development_queue_rebase",
    schema: "ddn.studio.ma3_next_development_queue_rebase.v1",
    work_item: "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
    based_on: "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
    workflow_claim: "ma3_next_development_queue_rebase",
    primary_coordinate: "마-3",
    support_coordinates: ["하-3", "타-3"],
    selected_default_coordinate: "마-3",
    generated_locally: true,
    product_code_change: true,
    product_ui_change: true,
    explicit_next_development_selection_claim: true,
    queue_rebase_only: true,
    runtime_claim: false,
    parser_frontdoor_change: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    new_automatic_queue_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    benchmark_execution_claim: false,
    performance_baseline_generation_claim: false,
    performance_baseline_publication_claim: false,
    lts_certification_claim: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    status: ready_stage_count === stages.length ? "ma3_next_development_queue_rebased" : "ma3_next_development_queue_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    queue_item_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_queue_item_id: active,
    queue_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 9,
      super_long_total: 18,
      super_long_percent: 50,
      previous_stage_closed: 8,
      previous_stage_total: 8,
      previous_stage_percent: 100,
      current_stage_closed: 1,
      current_stage_total: 8,
      current_stage_percent: 13,
      roadmap_v2_behavior_closed: 89,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 99,
    },
    next_item: "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
  };
}

export function formatMa3NextDevelopmentQueueRebaseText(rebase = {}) {
  const payload = asObject(rebase);
  if (payload.schema !== "ddn.studio.ma3_next_development_queue_rebase.v1") {
    throw new Error("seamgrim_expected_ma3_next_development_queue_rebase");
  }
  const rows = asArray(payload.queue_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `selected_default_coordinate\t${payload.selected_default_coordinate ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `runtime_claim\t${payload.runtime_claim === true ? "true" : "false"}`,
    `new_automatic_queue_claim\t${payload.new_automatic_queue_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `new_queue\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `new_queue_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    "",
    "queue_id\tcoordinate\tstatus\tclaim",
    ...rows.map((row) => [
      row.id,
      row.coordinate,
      row.status,
      row.claim,
    ].join("\t")),
  ].join("\n");
}

export function renderMa3NextDevelopmentQueueRebase(root, rebase = {}) {
  if (!root) return null;
  const payload = asObject(rebase);
  const rows = asArray(payload.queue_rows);
  const activeId = asText(payload.active_queue_item_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.ma3NextDevelopmentQueueStatus = asText(payload.status, "ma3_next_development_queue_incomplete");
  root.innerHTML = `
    <div class="ma3-dev-queue-head">
      <div>
        <div class="ma3-dev-queue-kicker">MA3 development</div>
        <h2>다음 개발 큐</h2>
      </div>
      <div class="ma3-dev-queue-progress" data-ma3-dev-queue-progress>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.super_long_percent ?? 0))}%</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} new queue</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(payload.next_item || "")}</span>
      </div>
    </div>
    <div class="ma3-dev-queue-summary">
      ${escapeHtml(String(payload.queue_item_count ?? rows.length))} items · explicit_selection=true · runtime=false · release_execution=false
    </div>
    <div class="ma3-dev-queue-body">
      <div class="ma3-dev-queue-list" data-ma3-dev-queue-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="ma3-dev-queue-btn${row.id === activeId ? " active" : ""}"
            data-ma3-dev-queue="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.id)}</span>
            <small>${escapeHtml(`${row.coordinate} · ${row.status}`)}</small>
          </button>
        `).join("")}
      </div>
      <div class="ma3-dev-queue-detail" data-ma3-dev-queue-detail>
        <div class="ma3-dev-queue-title" data-ma3-dev-queue-active-title>${escapeHtml(active.id)}</div>
        <p data-ma3-dev-queue-active-claim>${escapeHtml(active.claim)}</p>
        <dl>
          <div><dt>coordinate</dt><dd data-ma3-dev-queue-active-coordinate>${escapeHtml(active.coordinate)}</dd></div>
          <div><dt>status</dt><dd data-ma3-dev-queue-active-status>${escapeHtml(active.status)}</dd></div>
          <div><dt>boundary</dt><dd>queue lock only · no release/runtime</dd></div>
        </dl>
        <button type="button" class="ghost" data-ma3-dev-queue-copy>개발 큐 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderMa3NextDevelopmentQueueRebase(root, {
      ...payload,
      active_queue_item_id: rowId,
    });
  };
  root.querySelectorAll("[data-ma3-dev-queue]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-ma3-dev-queue") || "");
    });
  });
  root.querySelector("[data-ma3-dev-queue-copy]")?.addEventListener("click", async () => {
    const text = formatMa3NextDevelopmentQueueRebaseText(payload);
    root.dataset.ma3NextDevelopmentQueueCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
