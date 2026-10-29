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

const REPORT_SECTIONS = [
  {
    id: "hypothesis",
    title: "가설",
    artifact_key: "hypothesis",
    prompt: "계수를 키우면 결과 증가폭이 커진다.",
  },
  {
    id: "lever",
    title: "레버",
    artifact_key: "lever",
    prompt: "계수=2, 시작값=1을 첫 실행 기준으로 둔다.",
  },
  {
    id: "metric",
    title: "지표",
    artifact_key: "metric",
    prompt: "프레임수와 결과의 쌍을 관찰 지표로 기록한다.",
  },
  {
    id: "conclusion",
    title: "결론",
    artifact_key: "conclusion",
    prompt: "결과는 시작값 + 계수 * 프레임수 형태로 증가한다.",
  },
];

export const DEFAULT_FREE_LAB_EXPERIMENT_REPORT_ROWS = REPORT_SECTIONS.map((section) => ({
  id: section.id,
  artifact_key: section.artifact_key,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_FREE_LAB_EXPERIMENT_REPORT_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return REPORT_SECTIONS.map((section) => {
    const row = byId.get(section.id) || {};
    return {
      id: section.id,
      title: section.title,
      artifact_key: asText(row.artifact_key, section.artifact_key),
      prompt: asText(row.prompt, section.prompt),
      ready: row.ready !== false,
      product_ui_behavior: true,
      runtime_claim: false,
      share_claim: false,
      registry_publish_claim: false,
      cloud_sync_claim: false,
    };
  });
}

export function buildFreeLabExperimentReport({
  rows = DEFAULT_FREE_LAB_EXPERIMENT_REPORT_ROWS,
  activeSectionId = "hypothesis",
} = {}) {
  const sections = normalizeRows(rows);
  const active = sections.some((section) => section.id === activeSectionId)
    ? activeSectionId
    : sections[0]?.id ?? "";
  const readyCount = sections.filter((section) => section.ready).length;
  const reportReady = readyCount === sections.length;
  return {
    __종류: "free_lab_experiment_report",
    schema: "ddn.seamgrim.free_lab.experiment_report.v1",
    work_item: "BA2_FREE_LAB_EXPERIMENT_REPORT_PACK_CLOSURE_V1",
    primary_coordinate: "바-2",
    depends_on_coordinate: ["바-1"],
    pack: "free_lab_2_v1",
    status: reportReady ? "experiment_report_ready" : "experiment_report_incomplete",
    matrix_closure_tier: reportReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    first_run_claim: false,
    experiment_report_claim: reportReady,
    full_free_lab_ui_claim: false,
    share_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    cloud_sync_claim: false,
    report_section_count: sections.length,
    ready_section_count: readyCount,
    current_stage_closed: reportReady ? 5 : readyCount,
    current_stage_total: 5,
    current_stage_percent: reportReady ? 100 : Math.round((readyCount / 5) * 100),
    progress: {
      current_stage_closed: reportReady ? 5 : readyCount,
      current_stage_total: 5,
      current_stage_percent: reportReady ? 100 : Math.round((readyCount / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 9,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 10,
      roadmap_v2_pack_evidence_reference_closed: 28,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 31,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    sections,
    active_section_id: active,
    next_item: "BA3_FREE_LAB_UI_PACK_CLOSURE_V1",
  };
}

export function formatFreeLabExperimentReportText(report = {}) {
  const payload = asObject(report);
  if (payload.schema !== "ddn.seamgrim.free_lab.experiment_report.v1") {
    throw new Error("seamgrim_expected_free_lab_experiment_report");
  }
  const sections = asArray(payload.sections);
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
    "section_id\tartifact_key\tready\tprompt",
    ...sections.map((section) => [
      section.id,
      section.artifact_key,
      section.ready === true ? "true" : "false",
      section.prompt,
    ].join("\t")),
  ].join("\n");
}

export function renderFreeLabExperimentReport(root, report = {}) {
  if (!root) return null;
  const payload = asObject(report);
  const sections = asArray(payload.sections);
  const activeId = asText(payload.active_section_id, sections[0]?.id ?? "");
  const active = sections.find((section) => section.id === activeId) || sections[0] || {};
  root.dataset.freeLabExperimentReportStatus = asText(payload.status, "experiment_report_incomplete");
  root.innerHTML = `
    <div class="free-lab-report-head">
      <div>
        <div class="free-lab-report-kicker">Experiment report</div>
        <h2>자유 실험 보고서</h2>
      </div>
      <div class="free-lab-report-progress" data-free-lab-report-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="free-lab-report-summary" data-free-lab-report-summary>
      가설, 레버, 지표, 결론을 하나의 로컬 artifact로 묶어 다음 자유 실험 UI가 재사용할 수 있게 합니다.
    </div>
    <div class="free-lab-report-body">
      <div class="free-lab-report-list" data-free-lab-report-sections>
        ${sections.map((section) => `
          <button
            type="button"
            class="free-lab-report-btn${section.id === activeId ? " active" : ""}"
            data-free-lab-report-section="${escapeHtml(section.id)}"
          >
            <span>${escapeHtml(section.title)}</span>
            <small>${escapeHtml(section.artifact_key)}</small>
          </button>
        `).join("")}
      </div>
      <div class="free-lab-report-detail" data-free-lab-report-detail>
        <div class="free-lab-report-title" data-free-lab-report-active-title>${escapeHtml(active.title)}</div>
        <p data-free-lab-report-active-prompt>${escapeHtml(active.prompt)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>boundary</dt><dd>local artifact</dd></div>
        </dl>
        <button type="button" class="ghost" data-free-lab-report-copy>보고서 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (sectionId) => {
    renderFreeLabExperimentReport(root, {
      ...payload,
      active_section_id: sectionId,
    });
  };
  root.querySelectorAll("[data-free-lab-report-section]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-free-lab-report-section") || "");
    });
  });
  root.querySelector("[data-free-lab-report-copy]")?.addEventListener("click", async () => {
    const text = formatFreeLabExperimentReportText(payload);
    root.dataset.freeLabExperimentReportCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
