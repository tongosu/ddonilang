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

const PANEL_DEFS = [
  {
    id: "classroom_report_status_panel",
    source_triage_row: "classroom_report_ready",
    source_preview_section: null,
    operations_lane: "classroom_report",
    title: "수업 보고 상태",
    summary: "수업 실행 결과와 보고 준비 상태를 로컬 운영 패널로 묶습니다.",
  },
  {
    id: "teacher_feedback_status_panel",
    source_triage_row: "teacher_feedback_seed_ready",
    source_preview_section: "teacher_summary_panel",
    operations_lane: "teacher_feedback",
    title: "교사 피드백 상태",
    summary: "교사용 피드백 preview가 준비됐는지 확인합니다.",
  },
  {
    id: "student_next_step_queue_panel",
    source_triage_row: "student_next_step_queue",
    source_preview_section: "student_next_step_panel",
    operations_lane: "student_next_step",
    title: "학생 다음 단계 대기열",
    summary: "학생에게 넘길 다음 활동 항목을 로컬 대기열로 보여줍니다.",
  },
  {
    id: "misconception_review_queue_panel",
    source_triage_row: "misconception_review_queue",
    source_preview_section: "misconception_review_panel",
    operations_lane: "misconception_review",
    title: "오개념 검토 대기열",
    summary: "검토가 필요한 실패 케이스와 오개념 후보를 분리합니다.",
  },
  {
    id: "publication_candidate_review_panel",
    source_triage_row: "publication_candidate_review",
    source_preview_section: "publication_candidate_panel",
    operations_lane: "publication_candidate",
    title: "공개 후보 검토",
    summary: "공개 후보 교과의 검토 항목을 승인 전 로컬 상태로 유지합니다.",
  },
  {
    id: "approval_safe_handoff_panel",
    source_triage_row: "approval_safe_handoff_queue",
    source_preview_section: "approval_safe_handoff_panel",
    operations_lane: "approval_safe_handoff",
    title: "승인 안전 인계",
    summary: "릴리스 승인과 실행을 분리한 인계 상태를 고정합니다.",
  },
];

export const DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_ROWS = PANEL_DEFS.map((panel) => ({
  id: panel.source_triage_row,
  operations_lane: panel.operations_lane,
  triage_surface: "local_operations_packet",
  triage_only: true,
  generated_now: false,
  write_claim: false,
}));

function normalizeTriageRows(triageRows = []) {
  const rowsById = new Map(asArray(triageRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return PANEL_DEFS.map((panel) => {
    const source = rowsById.get(panel.source_triage_row) || {};
    return {
      id: panel.id,
      title: panel.title,
      summary: panel.summary,
      source_triage_row: panel.source_triage_row,
      source_preview_section: panel.source_preview_section,
      operations_lane: asText(source.operations_lane, panel.operations_lane),
      source_anchor: asText(source.source_anchor, "local_operations_packet"),
      panel_surface: "local_classroom_operations_panel_preview",
      panel_preview_only: true,
      generated_now: asBool(source.generated_now),
      write_claim: asBool(source.write_claim),
      product_ui_change: true,
      classroom_operations_runtime_claim: false,
      teacher_feedback_runtime_claim: false,
      student_data_collection_claim: false,
      cloud_sync_claim: false,
      account_setup_claim: false,
      permission_system_claim: false,
      result_replay_claim: false,
    };
  });
}

export function buildClassroomOperationsPanelPreview({
  triageRows = DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_ROWS,
  activePanelId = "classroom_report_status_panel",
} = {}) {
  const panelRows = normalizeTriageRows(triageRows);
  const active = panelRows.some((panel) => panel.id === activePanelId)
    ? activePanelId
    : panelRows[0]?.id ?? "";
  const localOnly = panelRows.every((panel) => (
    panel.panel_preview_only === true &&
    panel.generated_now === false &&
    panel.write_claim === false
  ));
  const stages = [
    ["triage_alignment", panelRows.length === PANEL_DEFS.length],
    ["panel_render_model", active.length > 0],
    ["local_only_boundary", localOnly],
    ["no_student_collection", true],
    ["no_panel_write", true],
    ["no_remote_runtime", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_classroom_operations_panel_preview",
    schema: "ddn.studio.classroom_operations_panel_preview.v1",
    work_item: "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
    primary_coordinate: "하-3",
    support_coordinate: "마-3",
    workflow_claim: "classroom_operations_panel_preview",
    generated_locally: true,
    product_ui_change: true,
    panel_preview_only: true,
    runtime_claim: false,
    classroom_operations_runtime_claim: false,
    teacher_feedback_runtime_claim: false,
    student_data_collection_claim: false,
    panel_write_claim: false,
    triage_write_claim: false,
    feedback_write_claim: false,
    remote_save_claim: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    result_replay_claim: false,
    status: ready_stage_count === stages.length ? "classroom_operations_panel_ready" : "classroom_operations_panel_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    panel_row_count: panelRows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_panel_id: active,
    panel_rows: panelRows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 3,
      current_stage_total: 8,
      current_stage_percent: 38,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
  };
}

export function formatClassroomOperationsPanelPreviewText(panel = {}) {
  const payload = asObject(panel);
  if (payload.schema !== "ddn.studio.classroom_operations_panel_preview.v1") {
    throw new Error("seamgrim_expected_classroom_operations_panel_preview");
  }
  const rows = asArray(payload.panel_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `runtime_claim\t${payload.runtime_claim === true ? "true" : "false"}`,
    `student_data_collection_claim\t${payload.student_data_collection_claim === true ? "true" : "false"}`,
    `panel_write_claim\t${payload.panel_write_claim === true ? "true" : "false"}`,
    `panel_row_count\t${payload.panel_row_count ?? rows.length}`,
    "",
    "panel_id\toperations_lane\tsource_triage_row\twrite_claim",
    ...rows.map((row) => [
      row.id,
      row.operations_lane,
      row.source_triage_row,
      row.write_claim === true ? "true" : "false",
    ].join("\t")),
  ].join("\n");
}

export function renderClassroomOperationsPanelPreview(root, panel = {}) {
  if (!root) return null;
  const payload = asObject(panel);
  const rows = asArray(payload.panel_rows);
  const activeId = asText(payload.active_panel_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.classroomOperationsStatus = asText(payload.status, "classroom_operations_panel_incomplete");
  root.innerHTML = `
    <div class="classroom-operations-panel-head">
      <div>
        <div class="classroom-operations-panel-kicker">수업 운영</div>
        <h2>로컬 운영 패널</h2>
      </div>
      <div class="classroom-operations-panel-progress" data-classroom-operations-progress>
        <span>${escapeHtml(String(payload.panel_row_count ?? rows.length))}개 패널</span>
        <span>쓰기 없음</span>
        <span>런타임 없음</span>
      </div>
    </div>
    <div class="classroom-operations-panel-body">
      <div class="classroom-operations-panel-list" data-classroom-operations-panel-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="classroom-operations-panel-btn${row.id === activeId ? " active" : ""}"
            data-classroom-operations-panel="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.operations_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="classroom-operations-detail" data-classroom-operations-detail>
        <div class="classroom-operations-detail-title" data-classroom-operations-active-title>${escapeHtml(active.title)}</div>
        <p data-classroom-operations-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-classroom-operations-active-lane>${escapeHtml(active.operations_lane)}</dd></div>
          <div><dt>triage</dt><dd data-classroom-operations-active-triage>${escapeHtml(active.source_triage_row)}</dd></div>
          <div><dt>boundary</dt><dd>local panel only · no write</dd></div>
        </dl>
        <button type="button" class="ghost" data-classroom-operations-copy>운영 패널 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (panelId) => {
    renderClassroomOperationsPanelPreview(root, {
      ...payload,
      active_panel_id: panelId,
    });
  };
  root.querySelectorAll("[data-classroom-operations-panel]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-classroom-operations-panel") || "");
    });
  });
  root.querySelector("[data-classroom-operations-copy]")?.addEventListener("click", async () => {
    const text = formatClassroomOperationsPanelPreviewText(payload);
    root.dataset.classroomOperationsCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
