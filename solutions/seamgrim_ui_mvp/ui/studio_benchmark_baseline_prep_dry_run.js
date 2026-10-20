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

const INPUT_DEFS = [
  {
    id: "benchmark_lts_matrix_input",
    source_anchor: "benchmark_lts_matrix",
    baseline_lane: "benchmark_lts_matrix",
    planned_path: "build/studio_benchmark/baseline/benchmark_lts_matrix.detjson",
    title: "benchmark/LTS matrix",
    summary: "benchmark/LTS matrix evidence를 baseline 입력 후보로만 고정합니다.",
  },
  {
    id: "classroom_operations_triage_input",
    source_anchor: "classroom_operations_triage",
    baseline_lane: "classroom_operations",
    planned_path: "build/studio_benchmark/baseline/classroom_operations_triage.detjson",
    title: "classroom triage",
    summary: "classroom operations triage evidence를 baseline 준비 입력으로만 사용합니다.",
  },
  {
    id: "browser_smoke_matrix_input",
    source_anchor: "benchmark_lts_matrix",
    baseline_lane: "browser_smoke_matrix",
    planned_path: "build/studio_benchmark/baseline/browser_smoke_matrix.detjson",
    title: "browser smoke",
    summary: "브라우저 smoke matrix는 실행 결과 생성 없이 planned input으로만 남깁니다.",
  },
  {
    id: "local_packaging_input",
    source_anchor: "benchmark_lts_matrix",
    baseline_lane: "local_packaging",
    planned_path: "build/studio_benchmark/baseline/local_packaging.detjson",
    title: "local packaging",
    summary: "로컬 packaging 입력 후보는 artifact 생성이나 signing 없이 유지합니다.",
  },
  {
    id: "approval_continuity_input",
    source_anchor: "benchmark_lts_matrix",
    baseline_lane: "approval_continuity",
    planned_path: "build/studio_benchmark/baseline/approval_continuity.detjson",
    title: "approval continuity",
    summary: "approval continuity 입력은 release approval/execution 없이 연결만 확인합니다.",
  },
];

export const DEFAULT_BENCHMARK_BASELINE_PREP_INPUT_ROWS = INPUT_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  baseline_lane: row.baseline_lane,
  planned_path: row.planned_path,
  prep_only: true,
  generated_now: false,
  benchmark_execution_claim: false,
  product_ui_change: false,
}));

function normalizeRows(rows = DEFAULT_BENCHMARK_BASELINE_PREP_INPUT_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return INPUT_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      baseline_lane: asText(source.baseline_lane, def.baseline_lane),
      planned_path: asText(source.planned_path, def.planned_path),
      prep_surface: "local_studio_benchmark_baseline_prep_dry_run",
      prep_only: true,
      generated_now: asBool(source.generated_now),
      benchmark_execution_claim: false,
      performance_baseline_generation_claim: false,
      performance_baseline_publication_claim: false,
      lts_certification_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_upload_claim: false,
      archive_generation_claim: false,
      publication_checksum_generation_claim: false,
      artifact_signing_claim: false,
      cloud_sync_claim: false,
      account_setup_claim: false,
      permission_system_claim: false,
      runtime_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      parser_frontdoor_change: false,
      product_ui_change: false,
    };
  });
}

export function buildBenchmarkBaselinePrepDryRun({
  inputRows = DEFAULT_BENCHMARK_BASELINE_PREP_INPUT_ROWS,
  activeInputId = "benchmark_lts_matrix_input",
} = {}) {
  const rows = normalizeRows(inputRows);
  const active = rows.some((row) => row.id === activeInputId)
    ? activeInputId
    : rows[0]?.id ?? "";
  const prepOnly = rows.every((row) => (
    row.prep_only === true &&
    row.generated_now === false &&
    row.benchmark_execution_claim === false &&
    row.performance_baseline_generation_claim === false &&
    row.performance_baseline_publication_claim === false &&
    row.lts_certification_claim === false &&
    row.release_execution_claim === false &&
    row.public_upload_claim === false
  ));
  const stages = [
    ["input_row_alignment", rows.length === INPUT_DEFS.length],
    ["benchmark_lts_matrix_linked", rows.some((row) => row.id === "benchmark_lts_matrix_input")],
    ["classroom_operations_linked", rows.some((row) => row.id === "classroom_operations_triage_input")],
    ["browser_smoke_planned", rows.some((row) => row.id === "browser_smoke_matrix_input")],
    ["approval_continuity_planned", rows.some((row) => row.id === "approval_continuity_input")],
    ["benchmark_execution_blocked", prepOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_benchmark_baseline_prep_dry_run",
    schema: "ddn.studio.benchmark_baseline_prep_dry_run.v1",
    work_item: "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
    based_on: "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
    workflow_claim: "benchmark_baseline_prep_dry_run",
    primary_coordinate: "타-3",
    support_coordinate: "마-3",
    product_ui_change: false,
    product_code_change: false,
    benchmark_baseline_prep_dry_run_claim: true,
    runtime_claim: false,
    benchmark_execution_claim: false,
    performance_baseline_generation_claim: false,
    performance_baseline_publication_claim: false,
    lts_certification_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_upload_claim: false,
    archive_generation_claim: false,
    publication_checksum_generation_claim: false,
    artifact_signing_claim: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    result_replay_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    planned_baseline_input_count: rows.length,
    all_inputs_prep_only: true,
    all_inputs_generated_now: false,
    all_inputs_benchmark_execution_claim: false,
    status: ready_stage_count === stages.length ? "benchmark_baseline_prep_ready" : "benchmark_baseline_prep_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_input_id: active,
    planned_baseline_inputs: rows,
    stages,
    progress: {
      super_long_behavior_closed: 5,
      super_long_total: 18,
      super_long_percent: 28,
      current_stage_closed: 7,
      current_stage_total: 8,
      current_stage_percent: 88,
      roadmap_v2_behavior_closed: 21,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 23,
    },
    next_item: "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
  };
}

export function formatBenchmarkBaselinePrepDryRunText(prep = {}) {
  const payload = asObject(prep);
  if (payload.schema !== "ddn.studio.benchmark_baseline_prep_dry_run.v1") {
    throw new Error("seamgrim_expected_benchmark_baseline_prep_dry_run");
  }
  const rows = asArray(payload.planned_baseline_inputs);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `planned_baseline_input_count\t${payload.planned_baseline_input_count ?? rows.length}`,
    `all_inputs_prep_only\t${payload.all_inputs_prep_only === true ? "true" : "false"}`,
    `all_inputs_generated_now\t${payload.all_inputs_generated_now === true ? "true" : "false"}`,
    `all_inputs_benchmark_execution_claim\t${payload.all_inputs_benchmark_execution_claim === true ? "true" : "false"}`,
    `benchmark_execution_claim\t${payload.benchmark_execution_claim === true ? "true" : "false"}`,
    `performance_baseline_generation_claim\t${payload.performance_baseline_generation_claim === true ? "true" : "false"}`,
    `followup\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `followup_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    "",
    "input_id\tlane\tsource_anchor\tgenerated_now",
    ...rows.map((row) => [
      row.id,
      row.baseline_lane,
      row.source_anchor,
      row.generated_now === true ? "true" : "false",
    ].join("\t")),
  ].join("\n");
}

export function renderBenchmarkBaselinePrepDryRun(root, prep = {}) {
  if (!root) return null;
  const payload = asObject(prep);
  const rows = asArray(payload.planned_baseline_inputs);
  const activeId = asText(payload.active_input_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.benchmarkBaselinePrepDryRunStatus = asText(payload.status, "benchmark_baseline_prep_incomplete");
  root.innerHTML = `
    <div class="benchmark-prep-head">
      <div>
        <div class="benchmark-prep-kicker">Benchmark baseline</div>
        <h2>Prep dry-run inputs</h2>
      </div>
      <div class="benchmark-prep-progress" data-benchmark-prep-progress>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} follow-up</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>prep-only</span>
      </div>
    </div>
    <div class="benchmark-prep-summary" data-benchmark-prep-summary>
      ${escapeHtml(String(payload.planned_baseline_input_count ?? 0))} inputs · generated_now=false · benchmark_execution=false
    </div>
    <div class="benchmark-prep-body">
      <div class="benchmark-prep-list" data-benchmark-prep-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="benchmark-prep-btn${row.id === activeId ? " active" : ""}"
            data-benchmark-prep="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.baseline_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="benchmark-prep-detail" data-benchmark-prep-detail>
        <div class="benchmark-prep-title" data-benchmark-prep-active-title>${escapeHtml(active.title)}</div>
        <p data-benchmark-prep-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-benchmark-prep-active-lane>${escapeHtml(active.baseline_lane)}</dd></div>
          <div><dt>path</dt><dd data-benchmark-prep-active-path>${escapeHtml(active.planned_path)}</dd></div>
          <div><dt>boundary</dt><dd>prep_only=true</dd></div>
        </dl>
        <button type="button" class="ghost" data-benchmark-prep-copy>baseline prep 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderBenchmarkBaselinePrepDryRun(root, {
      ...payload,
      active_input_id: rowId,
    });
  };
  root.querySelectorAll("[data-benchmark-prep]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-benchmark-prep") || "");
    });
  });
  root.querySelector("[data-benchmark-prep-copy]")?.addEventListener("click", async () => {
    const text = formatBenchmarkBaselinePrepDryRunText(payload);
    root.dataset.benchmarkBaselinePrepDryRunCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
