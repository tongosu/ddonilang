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

const SWEEP_ROWS = [
  { id: "baseline", coefficient: 2, start_value: 1, ghost: false },
  { id: "low_lever", coefficient: 1, start_value: 1, ghost: true },
  { id: "high_lever", coefficient: 3, start_value: 1, ghost: true },
];

export const DEFAULT_FREE_LAB_UI_ROWS = SWEEP_ROWS.map((row) => ({
  id: row.id,
  coefficient: row.coefficient,
  start_value: row.start_value,
  ghost: row.ghost,
  ready: true,
}));

function buildSeries({ coefficient, start_value }) {
  return [0, 1, 2, 3].map((frame) => ({
    frame,
    value: start_value + coefficient * frame,
  }));
}

function normalizeRows(rows = DEFAULT_FREE_LAB_UI_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return SWEEP_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    const coefficient = asNumber(source.coefficient, row.coefficient);
    const startValue = asNumber(source.start_value, row.start_value);
    return {
      id: row.id,
      title: row.id === "baseline" ? "기준" : row.id === "low_lever" ? "낮은 레버" : "높은 레버",
      coefficient,
      start_value: startValue,
      ghost: source.ghost === undefined ? row.ghost : source.ghost === true,
      ready: source.ready !== false,
      series: buildSeries({ coefficient, start_value: startValue }),
      product_ui_behavior: true,
      runtime_claim: false,
      share_claim: false,
      registry_publish_claim: false,
      cloud_sync_claim: false,
    };
  });
}

export function buildFreeLabUiPack({
  rows = DEFAULT_FREE_LAB_UI_ROWS,
  activeRunId = "baseline",
} = {}) {
  const runs = normalizeRows(rows);
  const active = runs.some((run) => run.id === activeRunId)
    ? activeRunId
    : runs[0]?.id ?? "";
  const baseline = runs.find((run) => run.id === "baseline") || runs[0] || {};
  const ghostRuns = runs.filter((run) => run.ghost);
  const readyCount = runs.filter((run) => run.ready).length;
  const hasSweep = runs.length >= 3 && runs.every((run) => run.series.length === 4);
  const hasGhostCompare = ghostRuns.length >= 2 && Boolean(baseline.id);
  const uiReady = readyCount === runs.length && hasSweep && hasGhostCompare;
  return {
    __종류: "free_lab_ui_pack",
    schema: "ddn.seamgrim.free_lab.ui_pack.v1",
    work_item: "BA3_FREE_LAB_UI_PACK_CLOSURE_V1",
    primary_coordinate: "바-3",
    depends_on_coordinate: ["바-2"],
    pack: "free_lab_3_v1",
    status: uiReady ? "free_lab_ui_ready" : "free_lab_ui_incomplete",
    matrix_closure_tier: uiReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    parameter_sweep_claim: hasSweep,
    ghost_compare_claim: hasGhostCompare,
    share_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    cloud_sync_claim: false,
    run_count: runs.length,
    ghost_run_count: ghostRuns.length,
    ready_run_count: readyCount,
    current_stage_closed: uiReady ? 5 : readyCount,
    current_stage_total: 5,
    current_stage_percent: uiReady ? 100 : Math.round((readyCount / 5) * 100),
    progress: {
      current_stage_closed: uiReady ? 5 : readyCount,
      current_stage_total: 5,
      current_stage_percent: uiReady ? 100 : Math.round((readyCount / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 10,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 11,
      roadmap_v2_pack_evidence_reference_closed: 29,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 32,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    runs,
    active_run_id: active,
    baseline_run_id: baseline.id ?? "",
    next_item: "BA4_FREE_LAB_SHARE_PACK_CLOSURE_V1",
  };
}

export function formatFreeLabUiPackText(pack = {}) {
  const payload = asObject(pack);
  if (payload.schema !== "ddn.seamgrim.free_lab.ui_pack.v1") {
    throw new Error("seamgrim_expected_free_lab_ui_pack");
  }
  const runs = asArray(payload.runs);
  return [
    `schema\t${payload.schema}`,
    `work_item\t${payload.work_item ?? ""}`,
    `primary_coordinate\t${payload.primary_coordinate ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `matrix_closure_tier\t${payload.matrix_closure_tier ?? ""}`,
    `current_stage\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? 0}`,
    `roadmap_matrix\t${payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0}/${payload.progress?.roadmap_v2_matrix_behavior_total ?? 0}`,
    `roadmap_matrix_percent\t${payload.progress?.roadmap_v2_matrix_behavior_percent ?? 0}`,
    `pack_evidence_reference\t${payload.progress?.roadmap_v2_pack_evidence_reference_closed ?? 0}/${payload.progress?.roadmap_v2_pack_evidence_reference_total ?? 0}`,
    `pack_evidence_reference_percent\t${payload.progress?.roadmap_v2_pack_evidence_reference_percent ?? 0}`,
    `studio_local_super_long\t${payload.progress?.studio_local_super_long_closed ?? 0}/${payload.progress?.studio_local_super_long_total ?? 0}`,
    `studio_local_super_long_percent\t${payload.progress?.studio_local_super_long_percent ?? 0}`,
    `parameter_sweep_claim\t${payload.parameter_sweep_claim === true ? "true" : "false"}`,
    `ghost_compare_claim\t${payload.ghost_compare_claim === true ? "true" : "false"}`,
    `share_claim\t${payload.share_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    "",
    "run_id\tcoefficient\tstart_value\tghost\tseries",
    ...runs.map((run) => [
      run.id,
      run.coefficient,
      run.start_value,
      run.ghost === true ? "true" : "false",
      asArray(run.series).map((point) => `${point.frame}:${point.value}`).join(","),
    ].join("\t")),
  ].join("\n");
}

export function renderFreeLabUiPack(root, pack = {}) {
  if (!root) return null;
  const payload = asObject(pack);
  const runs = asArray(payload.runs);
  const activeId = asText(payload.active_run_id, runs[0]?.id ?? "");
  const active = runs.find((run) => run.id === activeId) || runs[0] || {};
  const baseline = runs.find((run) => run.id === payload.baseline_run_id) || runs[0] || {};
  root.dataset.freeLabUiPackStatus = asText(payload.status, "free_lab_ui_incomplete");
  root.innerHTML = `
    <div class="free-lab-ui-head">
      <div>
        <div class="free-lab-ui-kicker">Free lab UI</div>
        <h2>자유 실험 UI</h2>
      </div>
      <div class="free-lab-ui-progress" data-free-lab-ui-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="free-lab-ui-summary" data-free-lab-ui-summary>
      계수 sweep 세 가지를 같은 표면에서 비교하고, 기준 run과 ghost run의 차이를 로컬 UI에서 확인합니다.
    </div>
    <div class="free-lab-ui-body">
      <div class="free-lab-ui-list" data-free-lab-ui-runs>
        ${runs.map((run) => `
          <button
            type="button"
            class="free-lab-ui-btn${run.id === activeId ? " active" : ""}"
            data-free-lab-ui-run="${escapeHtml(run.id)}"
          >
            <span>${escapeHtml(run.title)}</span>
            <small>계수 ${escapeHtml(String(run.coefficient))} · ${run.ghost ? "ghost" : "baseline"}</small>
          </button>
        `).join("")}
      </div>
      <div class="free-lab-ui-detail" data-free-lab-ui-detail>
        <div class="free-lab-ui-title" data-free-lab-ui-active-title>${escapeHtml(active.title)}</div>
        <p data-free-lab-ui-active-summary>
          ${escapeHtml(active.id || "")}: 시작값 ${escapeHtml(String(active.start_value ?? ""))}, 계수 ${escapeHtml(String(active.coefficient ?? ""))}
        </p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>ghosts</dt><dd>${escapeHtml(String(payload.ghost_run_count ?? 0))}</dd></div>
        </dl>
        <div class="free-lab-ui-series" data-free-lab-ui-series>
          ${asArray(active.series).map((point) => `
            <span>${escapeHtml(String(point.frame))}: ${escapeHtml(String(point.value))}</span>
          `).join("")}
        </div>
        <div class="free-lab-ui-compare" data-free-lab-ui-compare>
          기준 ${escapeHtml(String(baseline.coefficient ?? ""))} 대비 ${escapeHtml(String(active.coefficient ?? ""))} · ${active.ghost ? "ghost 비교" : "기준 run"}
        </div>
        <button type="button" class="ghost" data-free-lab-ui-copy>UI pack 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (runId) => {
    renderFreeLabUiPack(root, {
      ...payload,
      active_run_id: runId,
    });
  };
  root.querySelectorAll("[data-free-lab-ui-run]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-free-lab-ui-run") || "");
    });
  });
  root.querySelector("[data-free-lab-ui-copy]")?.addEventListener("click", async () => {
    const text = formatFreeLabUiPackText(payload);
    root.dataset.freeLabUiPackCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
