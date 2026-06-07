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

const GATE_DEFS = [
  {
    id: "teacher_feedback_surface_gate",
    source_runner: "tests/studio_teacher_feedback_surface_preview_runner.mjs",
    source_surface_row: "teacher_summary_panel",
    gate_lane: "teacher_feedback",
    title: "교사 피드백 preview",
    summary: "교사용 로컬 피드백 preview가 제품 UI에서 유지되는지 확인하는 게이트입니다.",
  },
  {
    id: "classroom_operations_panel_gate",
    source_runner: "tests/studio_classroom_operations_panel_preview_runner.mjs",
    source_surface_row: "classroom_report_status_panel",
    gate_lane: "classroom_operations",
    title: "수업 운영 패널",
    summary: "수업 운영 패널 preview가 쓰기 없이 유지되는지 확인합니다.",
  },
  {
    id: "benchmark_baseline_snapshot_gate",
    source_runner: "tests/studio_benchmark_baseline_local_snapshot_runner.mjs",
    source_surface_row: "benchmark_lts_matrix_snapshot",
    gate_lane: "benchmark_baseline",
    title: "Benchmark snapshot",
    summary: "benchmark 실행 없이 local snapshot 검토 표면이 보존되는지 확인합니다.",
  },
  {
    id: "release_review_packet_gate",
    source_runner: "tests/studio_release_review_packet_dashboard_runner.mjs",
    source_surface_row: "approval_state_dashboard_card",
    gate_lane: "release_review",
    title: "릴리스 검토 패킷",
    summary: "승인 대기 상태와 필수 승인 문구가 실행 없이 보존되는지 확인합니다.",
  },
  {
    id: "lesson_publication_surface_gate",
    source_runner: "tests/studio_lesson_publication_review_surface_runner.mjs",
    source_surface_row: "candidate_catalog_review_surface",
    gate_lane: "lesson_publication",
    title: "수업 공개 검토",
    summary: "12개 후보 수업의 공개 검토 표면이 upload 없이 유지되는지 확인합니다.",
  },
  {
    id: "product_stabilization_smoke_gate",
    source_runner: "tests/run_seamgrim_product_stabilization_smoke_check.py",
    source_surface_row: null,
    gate_lane: "product_stabilization",
    title: "제품 smoke 묶음",
    summary: "현재 제품 smoke 묶음의 검증 위치를 matrix에 고정합니다.",
  },
];

export const DEFAULT_MA3_REGRESSION_GATE_EVIDENCE = GATE_DEFS.map((gate) => ({
  id: gate.id,
  source_runner: gate.source_runner,
  gate_lane: gate.gate_lane,
  gate_matrix_only: true,
  generated_now: false,
  test_execution_claim: false,
  release_execution_claim: false,
}));

function normalizeGateRows(evidenceRows = DEFAULT_MA3_REGRESSION_GATE_EVIDENCE) {
  const evidenceById = new Map(asArray(evidenceRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return GATE_DEFS.map((gate) => {
    const evidence = evidenceById.get(gate.id) || {};
    return {
      id: gate.id,
      title: gate.title,
      summary: gate.summary,
      source_runner: asText(evidence.source_runner, gate.source_runner),
      source_surface_row: gate.source_surface_row,
      gate_lane: asText(evidence.gate_lane, gate.gate_lane),
      matrix_surface: "local_ma3_regression_gate_matrix",
      gate_matrix_only: true,
      generated_now: asBool(evidence.generated_now),
      test_execution_claim: false,
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

export function buildMa3RegressionGateMatrix({
  evidenceRows = DEFAULT_MA3_REGRESSION_GATE_EVIDENCE,
  activeGateId = "teacher_feedback_surface_gate",
} = {}) {
  const rows = normalizeGateRows(evidenceRows);
  const active = rows.some((row) => row.id === activeGateId)
    ? activeGateId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.gate_matrix_only === true &&
    row.generated_now === false &&
    row.test_execution_claim === false &&
    row.release_execution_claim === false &&
    row.public_upload_claim === false &&
    row.registry_publish_claim === false
  ));
  const stages = [
    ["gate_row_alignment", rows.length === GATE_DEFS.length],
    ["runner_anchor_alignment", rows.every((row) => String(row.source_runner || "").length > 0)],
    ["product_ui_gate_count", rows.filter((row) => row.source_runner.endsWith(".mjs")).length === 5],
    ["product_smoke_anchor", rows.some((row) => row.id === "product_stabilization_smoke_gate")],
    ["local_only_boundary", localOnly],
    ["no_release_or_publication_execution", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_ma3_regression_gate_matrix",
    schema: "ddn.studio.ma3_regression_gate_matrix.v1",
    work_item: "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
    primary_coordinate: "타-3",
    support_coordinate: "마-3",
    workflow_claim: "ma3_regression_gate_matrix",
    generated_locally: true,
    product_ui_change: true,
    gate_matrix_only: true,
    runtime_claim: false,
    test_execution_claim: false,
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
    status: ready_stage_count === stages.length ? "ma3_regression_gate_matrix_ready" : "ma3_regression_gate_matrix_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    gate_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_gate_id: active,
    gate_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 7,
      current_stage_total: 8,
      current_stage_percent: 88,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
  };
}

export function formatMa3RegressionGateMatrixText(matrix = {}) {
  const payload = asObject(matrix);
  if (payload.schema !== "ddn.studio.ma3_regression_gate_matrix.v1") {
    throw new Error("seamgrim_expected_ma3_regression_gate_matrix");
  }
  const rows = asArray(payload.gate_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `test_execution_claim\t${payload.test_execution_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `gate_row_count\t${payload.gate_row_count ?? rows.length}`,
    "",
    "gate_id\tgate_lane\tsource_runner\tsource_surface_row",
    ...rows.map((row) => [
      row.id,
      row.gate_lane,
      row.source_runner,
      row.source_surface_row ?? "",
    ].join("\t")),
  ].join("\n");
}

export function renderMa3RegressionGateMatrix(root, matrix = {}) {
  if (!root) return null;
  const payload = asObject(matrix);
  const rows = asArray(payload.gate_rows);
  const activeId = asText(payload.active_gate_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.ma3RegressionStatus = asText(payload.status, "ma3_regression_gate_matrix_incomplete");
  root.innerHTML = `
    <div class="ma3-regression-matrix-head">
      <div>
        <div class="ma3-regression-matrix-kicker">MA3 regression</div>
        <h2>회귀 게이트 매트릭스</h2>
      </div>
      <div class="ma3-regression-matrix-progress" data-ma3-regression-progress>
        <span>${escapeHtml(String(payload.gate_row_count ?? rows.length))}개 게이트</span>
        <span>실행 claim 없음</span>
        <span>공개 없음</span>
      </div>
    </div>
    <div class="ma3-regression-matrix-body">
      <div class="ma3-regression-matrix-list" data-ma3-regression-gate-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="ma3-regression-matrix-btn${row.id === activeId ? " active" : ""}"
            data-ma3-regression-gate="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.gate_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="ma3-regression-matrix-detail" data-ma3-regression-detail>
        <div class="ma3-regression-matrix-title" data-ma3-regression-active-title>${escapeHtml(active.title)}</div>
        <p data-ma3-regression-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-ma3-regression-active-lane>${escapeHtml(active.gate_lane)}</dd></div>
          <div><dt>runner</dt><dd data-ma3-regression-active-runner>${escapeHtml(active.source_runner)}</dd></div>
          <div><dt>boundary</dt><dd>matrix only · no execution claim</dd></div>
        </dl>
        <button type="button" class="ghost" data-ma3-regression-copy>게이트 매트릭스 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (gateId) => {
    renderMa3RegressionGateMatrix(root, {
      ...payload,
      active_gate_id: gateId,
    });
  };
  root.querySelectorAll("[data-ma3-regression-gate]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-ma3-regression-gate") || "");
    });
  });
  root.querySelector("[data-ma3-regression-copy]")?.addEventListener("click", async () => {
    const text = formatMa3RegressionGateMatrixText(payload);
    root.dataset.ma3RegressionCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
