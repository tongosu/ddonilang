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

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

export const FREE_LAB_FIRST_RUN_DDN_TEMPLATE = `설정 {
  제목: 자유_실험_첫실행.
  설명: "빈 캔버스에서 계수와 시작값을 바꾸며 결과를 관찰합니다.".
}.

채비 {
  계수:수 <- 2.
  시작값:수 <- 1.
  프레임수:수 <- 0.
  결과:수 <- 0.
}.

(매마디)마다 {
  결과 <- (시작값 + (계수 * 프레임수)).
  프레임수 보여주기.
  결과 보여주기.
  프레임수 <- (프레임수 + 1).
}.`;

const FIRST_RUN_LANES = [
  {
    id: "new_experiment",
    title: "새 실험",
    summary: "빈 캔버스에서 자유 실험 DDN 초안을 바로 작업실에 연다.",
    required_surface: "browse_to_studio_handoff",
  },
  {
    id: "parameter_setup",
    title: "매김",
    summary: "계수와 시작값을 채비 변수로 노출해 첫 실행 전에 조절 지점을 분리한다.",
    required_surface: "ddn_preset_parameters",
  },
  {
    id: "recording_boundary",
    title: "기록",
    summary: "첫 실행 결과는 로컬 저장/실행 기록 경계 안에서만 다루고 공유는 후속으로 남긴다.",
    required_surface: "local_run_record_boundary",
  },
];

export const DEFAULT_FREE_LAB_FIRST_RUN_ROWS = FIRST_RUN_LANES.map((lane) => ({
  id: lane.id,
  title: lane.title,
  required_surface: lane.required_surface,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_FREE_LAB_FIRST_RUN_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return FIRST_RUN_LANES.map((lane) => {
    const row = byId.get(lane.id) || {};
    return {
      id: lane.id,
      title: lane.title,
      summary: lane.summary,
      required_surface: asText(row.required_surface, lane.required_surface),
      ready: row.ready !== false,
      product_ui_behavior: true,
      runtime_claim: false,
      report_closure_claim: false,
      share_claim: false,
      registry_publish_claim: false,
      cloud_sync_claim: false,
    };
  });
}

export function buildFreeLabFirstRun({
  rows = DEFAULT_FREE_LAB_FIRST_RUN_ROWS,
  activeLaneId = "new_experiment",
  ddnTemplate = FREE_LAB_FIRST_RUN_DDN_TEMPLATE,
} = {}) {
  const lanes = normalizeRows(rows);
  const active = lanes.some((lane) => lane.id === activeLaneId)
    ? activeLaneId
    : lanes[0]?.id ?? "";
  const readyStageCount = lanes.filter((lane) => lane.ready).length;
  const firstRunReady = readyStageCount === lanes.length && String(ddnTemplate ?? "").includes("(매마디)마다");
  return {
    __종류: "free_lab_first_run",
    schema: "ddn.seamgrim.free_lab.first_run.v1",
    work_item: "BA1_FREE_LAB_FIRST_RUN_V1",
    primary_coordinate: "바-1",
    depends_on_coordinate: ["라-1", "사-1", "타-1"],
    pack: "free_lab_1_v1",
    status: firstRunReady ? "first_run_ready" : "first_run_incomplete",
    matrix_closure_tier: firstRunReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    report_closure_claim: false,
    share_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    cloud_sync_claim: false,
    first_run_claim: firstRunReady,
    ddn_template: String(ddnTemplate ?? ""),
    source_id: "ddn:free_lab:first_run",
    source_label: "자유 실험 첫실행",
    lane_count: lanes.length,
    ready_stage_count: readyStageCount,
    current_stage_closed: readyStageCount + (firstRunReady ? 2 : 0),
    current_stage_total: lanes.length + 2,
    current_stage_percent: firstRunReady ? 100 : Math.round((readyStageCount / (lanes.length + 2)) * 100),
    progress: {
      current_stage_closed: readyStageCount + (firstRunReady ? 2 : 0),
      current_stage_total: lanes.length + 2,
      current_stage_percent: firstRunReady ? 100 : Math.round((readyStageCount / (lanes.length + 2)) * 100),
      roadmap_v2_matrix_behavior_closed: 8,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 9,
      roadmap_v2_pack_evidence_reference_closed: 27,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 30,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    lanes,
    active_lane_id: active,
    next_item: "BA2_FREE_LAB_EXPERIMENT_REPORT_PACK_CLOSURE_V1",
  };
}

export function formatFreeLabFirstRunText(firstRun = {}) {
  const payload = asObject(firstRun);
  if (payload.schema !== "ddn.seamgrim.free_lab.first_run.v1") {
    throw new Error("seamgrim_expected_free_lab_first_run");
  }
  const lanes = asArray(payload.lanes);
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
    `share_claim\t${payload.share_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    "",
    "lane_id\trequired_surface\tready",
    ...lanes.map((lane) => [
      lane.id,
      lane.required_surface,
      lane.ready === true ? "true" : "false",
    ].join("\t")),
  ].join("\n");
}

export function renderFreeLabFirstRun(root, firstRun = {}, { onOpenFirstRun = null } = {}) {
  if (!root) return null;
  const payload = asObject(firstRun);
  const lanes = asArray(payload.lanes);
  const activeId = asText(payload.active_lane_id, lanes[0]?.id ?? "");
  const active = lanes.find((lane) => lane.id === activeId) || lanes[0] || {};
  root.dataset.freeLabFirstRunStatus = asText(payload.status, "first_run_incomplete");
  root.innerHTML = `
    <div class="free-lab-first-run-head">
      <div>
        <div class="free-lab-first-run-kicker">Free lab</div>
        <h2>자유 실험 첫실행</h2>
      </div>
      <div class="free-lab-first-run-progress" data-free-lab-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="free-lab-first-run-summary" data-free-lab-summary>
      빈 캔버스 실험을 작업실에 열고, 매김 변수와 로컬 기록 경계를 같은 첫 실행 경로에서 확인합니다.
    </div>
    <div class="free-lab-first-run-body">
      <div class="free-lab-first-run-list" data-free-lab-lanes>
        ${lanes.map((lane) => `
          <button
            type="button"
            class="free-lab-lane-btn${lane.id === activeId ? " active" : ""}"
            data-free-lab-lane="${escapeHtml(lane.id)}"
          >
            <span>${escapeHtml(lane.title)}</span>
            <small>${escapeHtml(lane.required_surface)}</small>
          </button>
        `).join("")}
      </div>
      <div class="free-lab-first-run-detail" data-free-lab-detail>
        <div class="free-lab-first-run-title" data-free-lab-active-title>${escapeHtml(active.title)}</div>
        <p data-free-lab-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>boundary</dt><dd>local first-run</dd></div>
        </dl>
        <button type="button" class="btn-primary" data-free-lab-open-first-run>새 자유 실험 열기</button>
        <button type="button" class="ghost" data-free-lab-copy>first-run 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (laneId) => {
    renderFreeLabFirstRun(root, {
      ...payload,
      active_lane_id: laneId,
    }, { onOpenFirstRun });
  };
  root.querySelectorAll("[data-free-lab-lane]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-free-lab-lane") || "");
    });
  });
  root.querySelector("[data-free-lab-open-first-run]")?.addEventListener("click", () => {
    root.dataset.freeLabOpened = "true";
    if (typeof onOpenFirstRun === "function") {
      onOpenFirstRun(payload);
    }
  });
  root.querySelector("[data-free-lab-copy]")?.addEventListener("click", async () => {
    const text = formatFreeLabFirstRunText(payload);
    root.dataset.freeLabCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
