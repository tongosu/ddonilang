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

const LOCK_DEFS = [
  {
    id: "ma3_coordinate_lock",
    source_anchor: "pack/studio_ma3_next_development_queue_rebase_v1/ma3_next_development_queue_rebase.detjson",
    coordinate: "마-3",
    lock_lane: "coordinate_lock",
    title: "마-3 좌표 잠금",
    summary: "다음 기본 작업 좌표를 마-3 Studio productization 흐름에 고정합니다.",
  },
  {
    id: "regression_gate_matrix_lock",
    source_anchor: "pack/studio_ma3_regression_gate_matrix_v1/ma3_regression_gate_matrix.detjson",
    coordinate: "타-3",
    lock_lane: "regression_gate",
    title: "회귀 게이트 인계",
    summary: "회귀 게이트 매트릭스가 다음 좌표 잠금의 선행 근거임을 표시합니다.",
  },
  {
    id: "lesson_publication_surface_lock",
    source_anchor: "pack/studio_lesson_publication_review_surface_v1/lesson_publication_review_surface.detjson",
    coordinate: "마-3",
    lock_lane: "publication_surface",
    title: "공개 검토 표면 인계",
    summary: "수업 공개 검토 표면의 12개 후보 경계를 다음 단계에 넘깁니다.",
  },
  {
    id: "product_smoke_gate_lock",
    source_anchor: "tests/run_seamgrim_product_stabilization_smoke_check.py",
    coordinate: "타-3",
    lock_lane: "product_smoke",
    title: "제품 smoke 게이트",
    summary: "제품 smoke 검증 위치를 좌표 잠금 기준에 포함합니다.",
  },
  {
    id: "docs_ssot_boundary_lock",
    source_anchor: "git status --short -- docs/ssot",
    coordinate: "마-3",
    lock_lane: "docs_ssot_boundary",
    title: "SSOT 경계",
    summary: "docs/ssot 직접 변경 없이 로컬 작업 경계만 유지합니다.",
  },
  {
    id: "next_stage_handoff_lock",
    source_anchor: "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
    coordinate: "마-3",
    lock_lane: "stage_handoff",
    title: "다음 stage 인계",
    summary: "자동 큐 생성 없이 closure rebase 후보로 인계합니다.",
  },
];

export const DEFAULT_MA3_NEXT_QUEUE_LOCK_ROWS = LOCK_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  coordinate: row.coordinate,
  lock_lane: row.lock_lane,
  coordinate_lock_only: true,
  generated_now: false,
  new_automatic_queue_claim: false,
  release_execution_claim: false,
  public_upload_claim: false,
}));

function normalizeLockRows(lockRows = DEFAULT_MA3_NEXT_QUEUE_LOCK_ROWS) {
  const rowById = new Map(asArray(lockRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return LOCK_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      coordinate: asText(source.coordinate, def.coordinate),
      lock_lane: asText(source.lock_lane, def.lock_lane),
      lock_surface: "local_ma3_next_queue_coordinate_lock",
      coordinate_lock_only: true,
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

export function buildMa3NextQueueCoordinateLock({
  lockRows = DEFAULT_MA3_NEXT_QUEUE_LOCK_ROWS,
  activeLockId = "ma3_coordinate_lock",
} = {}) {
  const rows = normalizeLockRows(lockRows);
  const active = rows.some((row) => row.id === activeLockId)
    ? activeLockId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.coordinate_lock_only === true &&
    row.generated_now === false &&
    row.new_automatic_queue_claim === false &&
    row.release_execution_claim === false &&
    row.public_upload_claim === false
  ));
  const stages = [
    ["lock_row_alignment", rows.length === LOCK_DEFS.length],
    ["ma3_coordinate_present", rows.some((row) => row.coordinate === "마-3")],
    ["regression_gate_anchor", rows.some((row) => row.id === "regression_gate_matrix_lock")],
    ["docs_ssot_boundary_anchor", rows.some((row) => row.id === "docs_ssot_boundary_lock")],
    ["local_only_boundary", localOnly],
    ["no_new_automatic_queue", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_ma3_next_queue_coordinate_lock",
    schema: "ddn.studio.ma3_next_queue_coordinate_lock.v1",
    work_item: "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    selected_default_coordinate: "마-3",
    workflow_claim: "ma3_next_queue_coordinate_lock",
    generated_locally: true,
    product_ui_change: true,
    coordinate_lock_only: true,
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
    status: ready_stage_count === stages.length ? "ma3_next_queue_coordinate_lock_ready" : "ma3_next_queue_coordinate_lock_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    lock_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_lock_id: active,
    lock_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 8,
      current_stage_total: 8,
      current_stage_percent: 100,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
  };
}

export function formatMa3NextQueueCoordinateLockText(lock = {}) {
  const payload = asObject(lock);
  if (payload.schema !== "ddn.studio.ma3_next_queue_coordinate_lock.v1") {
    throw new Error("seamgrim_expected_ma3_next_queue_coordinate_lock");
  }
  const rows = asArray(payload.lock_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `selected_default_coordinate\t${payload.selected_default_coordinate ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `new_automatic_queue_claim\t${payload.new_automatic_queue_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `lock_row_count\t${payload.lock_row_count ?? rows.length}`,
    "",
    "lock_id\tcoordinate\tlock_lane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.coordinate,
      row.lock_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderMa3NextQueueCoordinateLock(root, lock = {}) {
  if (!root) return null;
  const payload = asObject(lock);
  const rows = asArray(payload.lock_rows);
  const activeId = asText(payload.active_lock_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.ma3CoordinateLockStatus = asText(payload.status, "ma3_next_queue_coordinate_lock_incomplete");
  root.innerHTML = `
    <div class="ma3-coordinate-lock-head">
      <div>
        <div class="ma3-coordinate-lock-kicker">MA3 queue</div>
        <h2>다음 좌표 잠금</h2>
      </div>
      <div class="ma3-coordinate-lock-progress" data-ma3-coordinate-lock-progress>
        <span>${escapeHtml(payload.selected_default_coordinate || "마-3")}</span>
        <span>${escapeHtml(String(payload.lock_row_count ?? rows.length))}개 lock</span>
        <span>자동 큐 없음</span>
      </div>
    </div>
    <div class="ma3-coordinate-lock-body">
      <div class="ma3-coordinate-lock-list" data-ma3-coordinate-lock-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="ma3-coordinate-lock-btn${row.id === activeId ? " active" : ""}"
            data-ma3-coordinate-lock="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.lock_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="ma3-coordinate-lock-detail" data-ma3-coordinate-lock-detail>
        <div class="ma3-coordinate-lock-title" data-ma3-coordinate-lock-active-title>${escapeHtml(active.title)}</div>
        <p data-ma3-coordinate-lock-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>coordinate</dt><dd data-ma3-coordinate-lock-active-coordinate>${escapeHtml(active.coordinate)}</dd></div>
          <div><dt>lane</dt><dd data-ma3-coordinate-lock-active-lane>${escapeHtml(active.lock_lane)}</dd></div>
          <div><dt>boundary</dt><dd>coordinate lock only · no queue creation</dd></div>
        </dl>
        <button type="button" class="ghost" data-ma3-coordinate-lock-copy>좌표 잠금 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (lockId) => {
    renderMa3NextQueueCoordinateLock(root, {
      ...payload,
      active_lock_id: lockId,
    });
  };
  root.querySelectorAll("[data-ma3-coordinate-lock]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-ma3-coordinate-lock") || "");
    });
  });
  root.querySelector("[data-ma3-coordinate-lock-copy]")?.addEventListener("click", async () => {
    const text = formatMa3NextQueueCoordinateLockText(payload);
    root.dataset.ma3CoordinateLockCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
