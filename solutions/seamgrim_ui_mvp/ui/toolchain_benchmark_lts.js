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

function asNumber(value, fallback = 0) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

const BENCHMARK_ROWS = [
  { id: "perf_budget", title: "Perf budget", benchmark_kind: "local_perf_budget_snapshot", local_uri: "toolchain://benchmark/lts/perf-budget" },
  { id: "reference_band", title: "Reference band", benchmark_kind: "reference_band_snapshot", local_uri: "toolchain://benchmark/lts/reference-band" },
  { id: "migration_ledger", title: "Migration ledger", benchmark_kind: "migration_compat_ledger", local_uri: "toolchain://benchmark/lts/migration" },
  { id: "lts_gate", title: "LTS gate", benchmark_kind: "lts_gate_rehearsal", local_uri: "toolchain://benchmark/lts/gate" },
];

const DEFAULT_ARTIFACTS = [
  { name: "benchmark.perf.budget.detjson", kind: "perf_budget", bytes: 764 },
  { name: "benchmark.reference.band.detjson", kind: "reference_band", bytes: 812 },
  { name: "benchmark.migration.ledger.detjson", kind: "migration_ledger", bytes: 688 },
  { name: "benchmark.lts.gate.detjson", kind: "lts_gate", bytes: 642 },
];

export const DEFAULT_TOOLCHAIN_BENCHMARK_LTS_ROWS = BENCHMARK_ROWS.map((row) => ({
  id: row.id,
  benchmark_kind: row.benchmark_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_TOOLCHAIN_BENCHMARK_LTS_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return BENCHMARK_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      benchmark_kind: asText(source.benchmark_kind, row.benchmark_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_snapshot: true,
      long_running_execution: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `benchmark_lts_${index + 1}.detjson`),
      kind: asText(source.kind, "benchmark_lts_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildBenchmarkText(rows, artifacts) {
  return [
    "toolchain_benchmark_lts:toolchain_pack_5_v1",
    "coordinate:타-5",
    "benchmark_execution:false",
    "lts_certification:false",
    "perf_regression_blocker:false",
    "release_gate_execution:false",
    ...rows.map((row) => `${row.id}\t${row.benchmark_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildToolchainBenchmarkLts({
  rows = DEFAULT_TOOLCHAIN_BENCHMARK_LTS_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "perf_budget",
} = {}) {
  const benchmarkRows = normalizeRows(rows);
  const benchmarkArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(benchmarkArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = benchmarkRows.filter((row) => row.ready).length;
  const readyArtifactCount = benchmarkArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["perf_budget", "reference_band", "migration_ledger", "lts_gate"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === benchmarkRows.length &&
    readyArtifactCount === benchmarkArtifacts.length &&
    hasAllArtifacts;
  const active = benchmarkRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : benchmarkRows[0]?.id ?? "";
  return {
    __종류: "toolchain_benchmark_lts",
    schema: "ddn.toolchain.benchmark_lts.v1",
    work_item: "TA5_BENCHMARK_LTS_V1",
    primary_coordinate: "타-5",
    depends_on_coordinate: ["타-4", "타-3", "타-2"],
    pack: "toolchain_pack_5_v1",
    status: ready ? "toolchain_benchmark_lts_ready" : "toolchain_benchmark_lts_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    benchmark_lts_claim: ready,
    perf_budget_claim: artifactKinds.has("perf_budget"),
    reference_band_claim: artifactKinds.has("reference_band"),
    migration_ledger_claim: artifactKinds.has("migration_ledger"),
    lts_gate_claim: artifactKinds.has("lts_gate"),
    benchmark_execution_claim: false,
    lts_certification_claim: false,
    perf_regression_blocker_claim: false,
    release_gate_execution_claim: false,
    public_release_claim: false,
    cloud_benchmark_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: benchmarkRows,
    artifacts: benchmarkArtifacts,
    active_row_id: active,
    benchmark_text: buildBenchmarkText(benchmarkRows, benchmarkArtifacts),
    artifact_size_bytes: benchmarkArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 24,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 27,
      roadmap_v2_pack_evidence_reference_closed: 44,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 49,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "PA2_SOCIAL_BRIDGE_PACK_V1",
  };
}

export function formatToolchainBenchmarkLtsText(benchmarkLts = {}) {
  const payload = asObject(benchmarkLts);
  if (payload.schema !== "ddn.toolchain.benchmark_lts.v1") {
    throw new Error("toolchain_expected_benchmark_lts");
  }
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  return [
    `schema\t${payload.schema ?? ""}`,
    `work_item\t${payload.work_item ?? ""}`,
    `primary_coordinate\t${payload.primary_coordinate ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `matrix_closure_tier\t${payload.matrix_closure_tier ?? ""}`,
    `current_stage\t${progress.current_stage_closed ?? 0}/${progress.current_stage_total ?? 0}`,
    `current_stage_percent\t${progress.current_stage_percent ?? 0}`,
    `roadmap_matrix\t${progress.roadmap_v2_matrix_behavior_closed ?? 0}/${progress.roadmap_v2_matrix_behavior_total ?? 0}`,
    `roadmap_matrix_percent\t${progress.roadmap_v2_matrix_behavior_percent ?? 0}`,
    `pack_evidence_reference\t${progress.roadmap_v2_pack_evidence_reference_closed ?? 0}/${progress.roadmap_v2_pack_evidence_reference_total ?? 0}`,
    `pack_evidence_reference_percent\t${progress.roadmap_v2_pack_evidence_reference_percent ?? 0}`,
    `studio_local_super_long\t${progress.studio_local_super_long_closed ?? 0}/${progress.studio_local_super_long_total ?? 0}`,
    `studio_local_super_long_percent\t${progress.studio_local_super_long_percent ?? 0}`,
    `benchmark_lts_claim\t${payload.benchmark_lts_claim === true ? "true" : "false"}`,
    `perf_budget_claim\t${payload.perf_budget_claim === true ? "true" : "false"}`,
    `reference_band_claim\t${payload.reference_band_claim === true ? "true" : "false"}`,
    `migration_ledger_claim\t${payload.migration_ledger_claim === true ? "true" : "false"}`,
    `lts_gate_claim\t${payload.lts_gate_claim === true ? "true" : "false"}`,
    `benchmark_execution_claim\t${payload.benchmark_execution_claim === true ? "true" : "false"}`,
    `lts_certification_claim\t${payload.lts_certification_claim === true ? "true" : "false"}`,
    `perf_regression_blocker_claim\t${payload.perf_regression_blocker_claim === true ? "true" : "false"}`,
    `release_gate_execution_claim\t${payload.release_gate_execution_claim === true ? "true" : "false"}`,
    "",
    "row_id\tbenchmark_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.benchmark_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderToolchainBenchmarkLts(root, benchmarkLts = {}) {
  if (!root) return null;
  const payload = asObject(benchmarkLts);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.toolchainBenchmarkLtsStatus = asText(payload.status, "toolchain_benchmark_lts_incomplete");
  root.innerHTML = `
    <div class="toolchain-benchmark-head">
      <div>
        <div class="toolchain-benchmark-kicker">Benchmark / LTS</div>
        <h2>Toolchain benchmark LTS</h2>
      </div>
      <div class="toolchain-benchmark-progress" data-toolchain-benchmark-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="toolchain-benchmark-summary" data-toolchain-benchmark-summary>
      perf/reference band/migration/LTS gate를 local snapshot으로 고정하고 benchmark execution, LTS certification, release gate execution은 후속으로 둡니다.
    </div>
    <div class="toolchain-benchmark-body">
      <div class="toolchain-benchmark-list">
        ${rows.map((row) => `
          <button type="button" class="toolchain-benchmark-btn${row.id === activeId ? " active" : ""}" data-toolchain-benchmark-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.benchmark_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="toolchain-benchmark-detail">
        <div class="toolchain-benchmark-title" data-toolchain-benchmark-active-title>${escapeHtml(active.title)}</div>
        <p data-toolchain-benchmark-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="toolchain-benchmark-artifacts">
          ${artifacts.map((artifact) => `
            <span data-toolchain-benchmark-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="toolchain-benchmark-preview" data-toolchain-benchmark-preview>${escapeHtml(payload.benchmark_text ?? "")}</pre>
        <button type="button" class="ghost" data-toolchain-benchmark-copy>benchmark LTS 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderToolchainBenchmarkLts(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-toolchain-benchmark-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-toolchain-benchmark-row") || ""));
  });
  root.querySelector("[data-toolchain-benchmark-copy]")?.addEventListener("click", async () => {
    root.dataset.toolchainBenchmarkLtsCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatToolchainBenchmarkLtsText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
