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
    id: "teacher_feedback_surface_preview",
    work_item: "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
    source_anchor: "tests/studio_teacher_feedback_surface_preview_runner.mjs",
    coordinate: "하-3",
    closure_lane: "teacher_feedback",
    title: "교사 피드백 표면",
    summary: "교사 요약 패널과 local-only 피드백 표면을 닫힘-동작 근거로 묶습니다.",
  },
  {
    id: "classroom_operations_panel_preview",
    work_item: "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
    source_anchor: "tests/studio_classroom_operations_panel_preview_runner.mjs",
    coordinate: "하-3",
    closure_lane: "classroom_operations",
    title: "교실 운영 패널",
    summary: "교실 보고 상태 패널을 운영 preview stage의 제품 UI 근거로 유지합니다.",
  },
  {
    id: "benchmark_baseline_local_snapshot",
    work_item: "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
    source_anchor: "tests/studio_benchmark_baseline_local_snapshot_runner.mjs",
    coordinate: "타-3",
    closure_lane: "benchmark_baseline",
    title: "벤치마크 baseline snapshot",
    summary: "실행 없이 로컬 baseline snapshot 표시 경계를 닫습니다.",
  },
  {
    id: "release_review_packet_dashboard",
    work_item: "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
    source_anchor: "tests/studio_release_review_packet_dashboard_runner.mjs",
    coordinate: "마-3",
    closure_lane: "release_review",
    title: "릴리스 검토 dashboard",
    summary: "승인 대기 상태를 보여주되 승인과 실행은 하지 않는 dashboard를 묶습니다.",
  },
  {
    id: "lesson_publication_review_surface",
    work_item: "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
    source_anchor: "tests/studio_lesson_publication_review_surface_runner.mjs",
    coordinate: "마-3",
    closure_lane: "lesson_publication",
    title: "수업 공개 검토 표면",
    summary: "공개 후보 검토 표면을 public upload 없이 stage 근거로 고정합니다.",
  },
  {
    id: "ma3_regression_gate_matrix",
    work_item: "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
    source_anchor: "tests/studio_ma3_regression_gate_matrix_runner.mjs",
    coordinate: "타-3",
    closure_lane: "regression_gate",
    title: "MA3 회귀 게이트",
    summary: "제품 smoke와 UI runner 묶음을 stage closure 선행 gate로 연결합니다.",
  },
  {
    id: "ma3_next_queue_coordinate_lock",
    work_item: "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
    source_anchor: "tests/studio_ma3_next_queue_coordinate_lock_runner.mjs",
    coordinate: "마-3",
    closure_lane: "coordinate_lock",
    title: "MA3 다음 좌표 잠금",
    summary: "다음 기본 좌표를 마-3으로 잠그고 자동 큐 생성을 막습니다.",
  },
  {
    id: "operations_preview_stage_closure",
    work_item: "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
    source_anchor: "solutions/seamgrim_ui_mvp/ui/studio_operations_preview_stage_closure.js",
    coordinate: "마-3",
    closure_lane: "stage_closure",
    title: "운영 preview stage closure",
    summary: "8개 stage row를 제품 UI에서 닫힘-동작 상태로 확정합니다.",
  },
];

export const DEFAULT_OPERATIONS_PREVIEW_STAGE_CLOSURE_ROWS = CLOSURE_DEFS.map((row) => ({
  id: row.id,
  work_item: row.work_item,
  source_anchor: row.source_anchor,
  coordinate: row.coordinate,
  closure_lane: row.closure_lane,
  behavior_closed: true,
  docs_closed: true,
  generated_now: false,
  product_ui_change: true,
  release_execution_claim: false,
  public_upload_claim: false,
}));

function normalizeClosureRows(closureRows = DEFAULT_OPERATIONS_PREVIEW_STAGE_CLOSURE_ROWS) {
  const rowById = new Map(asArray(closureRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return CLOSURE_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      work_item: asText(source.work_item, def.work_item),
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      coordinate: asText(source.coordinate, def.coordinate),
      closure_lane: asText(source.closure_lane, def.closure_lane),
      closure_surface: "local_studio_operations_preview_stage_closure",
      behavior_closed: source.behavior_closed !== false,
      docs_closed: source.docs_closed !== false,
      stage_closure_only: true,
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

export function buildOperationsPreviewStageClosure({
  closureRows = DEFAULT_OPERATIONS_PREVIEW_STAGE_CLOSURE_ROWS,
  activeClosureId = "operations_preview_stage_closure",
} = {}) {
  const rows = normalizeClosureRows(closureRows);
  const active = rows.some((row) => row.id === activeClosureId)
    ? activeClosureId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.stage_closure_only === true &&
    row.behavior_closed === true &&
    row.generated_now === false &&
    row.release_execution_claim === false &&
    row.public_upload_claim === false
  ));
  const stages = [
    ["closure_row_alignment", rows.length === CLOSURE_DEFS.length],
    ["all_rows_behavior_closed", rows.every((row) => row.behavior_closed === true)],
    ["all_rows_docs_closed", rows.every((row) => row.docs_closed === true)],
    ["coordinate_lock_anchor", rows.some((row) => row.id === "ma3_next_queue_coordinate_lock")],
    ["local_only_boundary", localOnly],
    ["stage_percent_complete", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_operations_preview_stage_closure",
    schema: "ddn.studio.operations_preview_stage_closure.v1",
    work_item: "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
    based_on: "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    workflow_claim: "operations_preview_stage_closure",
    generated_locally: true,
    product_ui_change: true,
    stage_closure_only: true,
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
    status: ready_stage_count === stages.length ? "operations_preview_stage_closed" : "operations_preview_stage_incomplete",
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
      current_stage_closed: 8,
      current_stage_total: 8,
      current_stage_percent: 100,
      roadmap_v2_behavior_closed: 51,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 57,
    },
    next_item: "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
  };
}

export function formatOperationsPreviewStageClosureText(closure = {}) {
  const payload = asObject(closure);
  if (payload.schema !== "ddn.studio.operations_preview_stage_closure.v1") {
    throw new Error("seamgrim_expected_operations_preview_stage_closure");
  }
  const rows = asArray(payload.closure_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `closure_row_count\t${payload.closure_row_count ?? rows.length}`,
    "",
    "closure_id\twork_item\tcoordinate\tclosure_lane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.work_item,
      row.coordinate,
      row.closure_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderOperationsPreviewStageClosure(root, closure = {}) {
  if (!root) return null;
  const payload = asObject(closure);
  const rows = asArray(payload.closure_rows);
  const activeId = asText(payload.active_closure_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.operationsPreviewStageStatus = asText(payload.status, "operations_preview_stage_incomplete");
  root.innerHTML = `
    <div class="operations-stage-closure-head">
      <div>
        <div class="operations-stage-closure-kicker">Studio operations</div>
        <h2>Preview stage closure</h2>
      </div>
      <div class="operations-stage-closure-progress" data-operations-stage-closure-progress>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>닫힘-동작</span>
      </div>
    </div>
    <div class="operations-stage-closure-body">
      <div class="operations-stage-closure-list" data-operations-stage-closure-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="operations-stage-closure-btn${row.id === activeId ? " active" : ""}"
            data-operations-stage-closure="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.closure_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="operations-stage-closure-detail" data-operations-stage-closure-detail>
        <div class="operations-stage-closure-title" data-operations-stage-closure-active-title>${escapeHtml(active.title)}</div>
        <p data-operations-stage-closure-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>coordinate</dt><dd data-operations-stage-closure-active-coordinate>${escapeHtml(active.coordinate)}</dd></div>
          <div><dt>lane</dt><dd data-operations-stage-closure-active-lane>${escapeHtml(active.closure_lane)}</dd></div>
          <div><dt>work item</dt><dd data-operations-stage-closure-active-work-item>${escapeHtml(active.work_item)}</dd></div>
        </dl>
        <button type="button" class="ghost" data-operations-stage-closure-copy>stage closure 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (closureId) => {
    renderOperationsPreviewStageClosure(root, {
      ...payload,
      active_closure_id: closureId,
    });
  };
  root.querySelectorAll("[data-operations-stage-closure]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-operations-stage-closure") || "");
    });
  });
  root.querySelector("[data-operations-stage-closure-copy]")?.addEventListener("click", async () => {
    const text = formatOperationsPreviewStageClosureText(payload);
    root.dataset.operationsPreviewStageCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
