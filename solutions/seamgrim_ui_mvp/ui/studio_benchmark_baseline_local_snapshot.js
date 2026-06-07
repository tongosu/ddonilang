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

const SNAPSHOT_DEFS = [
  {
    id: "benchmark_lts_matrix_snapshot",
    source_planned_input: "benchmark_lts_matrix_input",
    source_panel_row: null,
    snapshot_lane: "benchmark_lts_matrix",
    title: "Benchmark/LTS matrix",
    summary: "LTS 후보 행렬을 실행 전 로컬 기준점으로 고정합니다.",
  },
  {
    id: "classroom_operations_triage_snapshot",
    source_planned_input: "classroom_operations_triage_input",
    source_panel_row: null,
    snapshot_lane: "classroom_operations_triage",
    title: "수업 운영 triage",
    summary: "수업 운영 triage 입력을 baseline 후보로만 보존합니다.",
  },
  {
    id: "browser_smoke_matrix_snapshot",
    source_planned_input: "browser_smoke_matrix_input",
    source_panel_row: null,
    snapshot_lane: "browser_smoke_matrix",
    title: "Browser smoke matrix",
    summary: "브라우저 smoke 범위를 성능 측정 없이 검토합니다.",
  },
  {
    id: "local_packaging_snapshot",
    source_planned_input: "local_packaging_input",
    source_panel_row: null,
    snapshot_lane: "local_packaging",
    title: "Local packaging",
    summary: "로컬 패키징 경계를 공개 업로드 전 상태로 묶습니다.",
  },
  {
    id: "approval_continuity_snapshot",
    source_planned_input: "approval_continuity_input",
    source_panel_row: null,
    snapshot_lane: "approval_continuity",
    title: "Approval continuity",
    summary: "승인 연속성 상태를 실행 승인과 분리해 보여줍니다.",
  },
  {
    id: "classroom_operations_panel_snapshot",
    source_planned_input: null,
    source_panel_row: "classroom_report_status_panel",
    snapshot_lane: "classroom_operations_panel",
    title: "수업 운영 패널",
    summary: "직전 로컬 운영 패널의 anchor를 benchmark snapshot에 연결합니다.",
  },
];

export const DEFAULT_BENCHMARK_BASELINE_INPUTS = SNAPSHOT_DEFS.map((row) => ({
  id: row.source_planned_input,
  baseline_lane: row.snapshot_lane,
  prep_only: true,
  generated_now: false,
  benchmark_execution_claim: false,
  performance_baseline_generation_claim: false,
  performance_baseline_publication_claim: false,
})).filter((row) => row.id);

export const DEFAULT_BENCHMARK_CLASSROOM_PANEL_ROWS = [{
  id: "classroom_report_status_panel",
  panel_preview_only: true,
  generated_now: false,
  write_claim: false,
}];

function normalizeSnapshotRows({
  plannedInputs = DEFAULT_BENCHMARK_BASELINE_INPUTS,
  panelRows = DEFAULT_BENCHMARK_CLASSROOM_PANEL_ROWS,
} = {}) {
  const inputById = new Map(asArray(plannedInputs).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  const panelById = new Map(asArray(panelRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return SNAPSHOT_DEFS.map((def) => {
    const source = def.source_planned_input
      ? inputById.get(def.source_planned_input) || {}
      : panelById.get(def.source_panel_row) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_planned_input: def.source_planned_input,
      source_panel_row: def.source_panel_row,
      snapshot_lane: asText(source.baseline_lane, def.snapshot_lane),
      snapshot_surface: "local_benchmark_baseline_snapshot",
      snapshot_only: true,
      generated_now: asBool(source.generated_now),
      benchmark_execution_claim: asBool(source.benchmark_execution_claim),
      performance_baseline_generation_claim: asBool(source.performance_baseline_generation_claim),
      performance_baseline_publication_claim: asBool(source.performance_baseline_publication_claim),
      lts_certification_claim: false,
      public_release_claim: false,
    };
  });
}

export function buildBenchmarkBaselineLocalSnapshot({
  plannedInputs = DEFAULT_BENCHMARK_BASELINE_INPUTS,
  panelRows = DEFAULT_BENCHMARK_CLASSROOM_PANEL_ROWS,
  activeSnapshotId = "benchmark_lts_matrix_snapshot",
} = {}) {
  const rows = normalizeSnapshotRows({ plannedInputs, panelRows });
  const active = rows.some((row) => row.id === activeSnapshotId)
    ? activeSnapshotId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.snapshot_only === true &&
    row.generated_now === false &&
    row.benchmark_execution_claim === false &&
    row.performance_baseline_generation_claim === false &&
    row.performance_baseline_publication_claim === false
  ));
  const stages = [
    ["prep_alignment", rows.filter((row) => row.source_planned_input).length === 5],
    ["panel_anchor_alignment", rows.filter((row) => row.source_panel_row).length === 1],
    ["snapshot_render_model", active.length > 0],
    ["local_only_boundary", localOnly],
    ["no_benchmark_execution", true],
    ["no_performance_publication", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_benchmark_baseline_local_snapshot",
    schema: "ddn.studio.benchmark_baseline_local_snapshot.v1",
    work_item: "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
    primary_coordinate: "타-3",
    support_coordinate: "마-3",
    workflow_claim: "benchmark_baseline_local_snapshot",
    generated_locally: true,
    product_ui_change: true,
    snapshot_only: true,
    runtime_claim: false,
    benchmark_execution_claim: false,
    performance_baseline_generation_claim: false,
    performance_baseline_publication_claim: false,
    lts_certification_claim: false,
    classroom_operations_runtime_claim: false,
    teacher_feedback_runtime_claim: false,
    student_data_collection_claim: false,
    panel_write_claim: false,
    feedback_write_claim: false,
    remote_save_claim: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    result_replay_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    status: ready_stage_count === stages.length ? "benchmark_baseline_snapshot_ready" : "benchmark_baseline_snapshot_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    snapshot_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_snapshot_id: active,
    snapshot_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 4,
      current_stage_total: 8,
      current_stage_percent: 50,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
  };
}

export function formatBenchmarkBaselineLocalSnapshotText(snapshot = {}) {
  const payload = asObject(snapshot);
  if (payload.schema !== "ddn.studio.benchmark_baseline_local_snapshot.v1") {
    throw new Error("seamgrim_expected_benchmark_baseline_local_snapshot");
  }
  const rows = asArray(payload.snapshot_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `runtime_claim\t${payload.runtime_claim === true ? "true" : "false"}`,
    `benchmark_execution_claim\t${payload.benchmark_execution_claim === true ? "true" : "false"}`,
    `performance_baseline_generation_claim\t${payload.performance_baseline_generation_claim === true ? "true" : "false"}`,
    `snapshot_row_count\t${payload.snapshot_row_count ?? rows.length}`,
    "",
    "snapshot_id\tsnapshot_lane\tsource_planned_input\tsource_panel_row",
    ...rows.map((row) => [
      row.id,
      row.snapshot_lane,
      row.source_planned_input ?? "",
      row.source_panel_row ?? "",
    ].join("\t")),
  ].join("\n");
}

export function renderBenchmarkBaselineLocalSnapshot(root, snapshot = {}) {
  if (!root) return null;
  const payload = asObject(snapshot);
  const rows = asArray(payload.snapshot_rows);
  const activeId = asText(payload.active_snapshot_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.benchmarkBaselineStatus = asText(payload.status, "benchmark_baseline_snapshot_incomplete");
  root.innerHTML = `
    <div class="benchmark-baseline-snapshot-head">
      <div>
        <div class="benchmark-baseline-snapshot-kicker">Benchmark baseline</div>
        <h2>로컬 스냅샷</h2>
      </div>
      <div class="benchmark-baseline-snapshot-progress" data-benchmark-baseline-progress>
        <span>${escapeHtml(String(payload.snapshot_row_count ?? rows.length))}개 스냅샷</span>
        <span>실행 없음</span>
        <span>공개 없음</span>
      </div>
    </div>
    <div class="benchmark-baseline-snapshot-body">
      <div class="benchmark-baseline-snapshot-list" data-benchmark-baseline-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="benchmark-baseline-snapshot-btn${row.id === activeId ? " active" : ""}"
            data-benchmark-baseline-snapshot="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.snapshot_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="benchmark-baseline-snapshot-detail" data-benchmark-baseline-detail>
        <div class="benchmark-baseline-snapshot-title" data-benchmark-baseline-active-title>${escapeHtml(active.title)}</div>
        <p data-benchmark-baseline-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-benchmark-baseline-active-lane>${escapeHtml(active.snapshot_lane)}</dd></div>
          <div><dt>source</dt><dd data-benchmark-baseline-active-source>${escapeHtml(active.source_planned_input || active.source_panel_row || "local")}</dd></div>
          <div><dt>boundary</dt><dd>snapshot only · no benchmark run</dd></div>
        </dl>
        <button type="button" class="ghost" data-benchmark-baseline-copy>스냅샷 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (snapshotId) => {
    renderBenchmarkBaselineLocalSnapshot(root, {
      ...payload,
      active_snapshot_id: snapshotId,
    });
  };
  root.querySelectorAll("[data-benchmark-baseline-snapshot]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-benchmark-baseline-snapshot") || "");
    });
  });
  root.querySelector("[data-benchmark-baseline-copy]")?.addEventListener("click", async () => {
    const text = formatBenchmarkBaselineLocalSnapshotText(payload);
    root.dataset.benchmarkBaselineCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
