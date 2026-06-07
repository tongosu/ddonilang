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

const TRIAGE_DEFS = [
  {
    id: "classroom_report_ready",
    source_anchor: "classroom_report_workflow",
    operations_lane: "classroom_report",
    title: "classroom report",
    summary: "수업 보고 workflow evidence를 local operations packet에 연결합니다.",
  },
  {
    id: "teacher_feedback_seed_ready",
    source_anchor: "teacher_feedback_loop_seed",
    operations_lane: "teacher_feedback",
    title: "feedback seed",
    summary: "교사 피드백 seed row를 operations triage 입력으로만 사용합니다.",
  },
  {
    id: "student_next_step_queue",
    source_anchor: "teacher_feedback_loop_seed",
    operations_lane: "student_next_step",
    title: "student queue",
    summary: "학생 다음 단계 queue 후보만 남기고 student data collection은 열지 않습니다.",
  },
  {
    id: "misconception_review_queue",
    source_anchor: "teacher_feedback_loop_seed",
    operations_lane: "misconception_review",
    title: "misconception queue",
    summary: "오개념 검토 queue 후보만 고정하고 triage write는 수행하지 않습니다.",
  },
  {
    id: "publication_candidate_review",
    source_anchor: "teacher_feedback_loop_seed",
    operations_lane: "publication_candidate",
    title: "publication review",
    summary: "공개 후보 검토 queue는 local-only 상태로 유지합니다.",
  },
  {
    id: "approval_safe_handoff_queue",
    source_anchor: "teacher_feedback_loop_seed",
    operations_lane: "approval_safe_handoff",
    title: "approval handoff",
    summary: "승인 안전 인수 queue는 release approval이나 execution claim 없이 유지됩니다.",
  },
];

export const DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_UI_ROWS = TRIAGE_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  operations_lane: row.operations_lane,
  triage_surface: "local_operations_packet",
  triage_only: true,
  generated_now: false,
  write_claim: false,
  product_ui_change: true,
  classroom_operations_runtime_claim: false,
  teacher_feedback_runtime_claim: false,
  student_data_collection_claim: false,
}));

function normalizeRows(rows = DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_UI_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return TRIAGE_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      operations_lane: asText(source.operations_lane, def.operations_lane),
      triage_surface: asText(source.triage_surface, "local_operations_packet"),
      triage_only: true,
      generated_now: asBool(source.generated_now),
      write_claim: false,
      classroom_operations_runtime_claim: false,
      teacher_feedback_runtime_claim: false,
      student_data_collection_claim: false,
      triage_write_claim: false,
      feedback_write_claim: false,
      remote_save_claim: false,
      cloud_sync_claim: false,
      account_setup_claim: false,
      permission_system_claim: false,
      result_replay_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_upload_claim: false,
      runtime_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      parser_frontdoor_change: false,
      product_ui_change: true,
    };
  });
}

export function buildClassroomOperationsTriage({
  triageRows = DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_UI_ROWS,
  activeTriageId = "student_next_step_queue",
} = {}) {
  const rows = normalizeRows(triageRows);
  const active = rows.some((row) => row.id === activeTriageId)
    ? activeTriageId
    : rows[0]?.id ?? "";
  const triageOnly = rows.every((row) => (
    row.triage_only === true &&
    row.generated_now === false &&
    row.write_claim === false &&
    row.classroom_operations_runtime_claim === false &&
    row.teacher_feedback_runtime_claim === false &&
    row.student_data_collection_claim === false &&
    row.remote_save_claim === false &&
    row.cloud_sync_claim === false &&
    row.permission_system_claim === false &&
    row.release_execution_claim === false
  ));
  const stages = [
    ["triage_row_alignment", rows.length === TRIAGE_DEFS.length],
    ["classroom_report_linked", rows.some((row) => row.id === "classroom_report_ready")],
    ["teacher_feedback_seed_linked", rows.some((row) => row.id === "teacher_feedback_seed_ready")],
    ["student_next_step_queued", rows.some((row) => row.id === "student_next_step_queue")],
    ["approval_safe_handoff_queued", rows.some((row) => row.id === "approval_safe_handoff_queue")],
    ["write_boundary_blocked", triageOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_classroom_operations_triage",
    schema: "ddn.studio.classroom_operations_triage.v1",
    work_item: "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
    based_on: "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
    workflow_claim: "classroom_operations_triage",
    primary_coordinate: "하-3",
    support_coordinate: "마-3",
    product_ui_change: true,
    product_code_change: true,
    classroom_operations_triage_claim: true,
    runtime_claim: false,
    classroom_operations_runtime_claim: false,
    teacher_feedback_runtime_claim: false,
    student_data_collection_claim: false,
    triage_write_claim: false,
    feedback_write_claim: false,
    remote_save_claim: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    result_replay_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_upload_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    triage_row_count: rows.length,
    all_triage_rows_triage_only: true,
    all_triage_rows_generated_now: false,
    all_triage_rows_write_claim: false,
    status: ready_stage_count === stages.length ? "classroom_operations_triage_ready" : "classroom_operations_triage_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_triage_id: active,
    triage_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 6,
      current_stage_total: 8,
      current_stage_percent: 75,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
  };
}

export function formatClassroomOperationsTriageText(triage = {}) {
  const payload = asObject(triage);
  if (payload.schema !== "ddn.studio.classroom_operations_triage.v1") {
    throw new Error("seamgrim_expected_classroom_operations_triage");
  }
  const rows = asArray(payload.triage_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `triage_row_count\t${payload.triage_row_count ?? rows.length}`,
    `all_triage_rows_triage_only\t${payload.all_triage_rows_triage_only === true ? "true" : "false"}`,
    `all_triage_rows_generated_now\t${payload.all_triage_rows_generated_now === true ? "true" : "false"}`,
    `all_triage_rows_write_claim\t${payload.all_triage_rows_write_claim === true ? "true" : "false"}`,
    `classroom_operations_runtime_claim\t${payload.classroom_operations_runtime_claim === true ? "true" : "false"}`,
    `student_data_collection_claim\t${payload.student_data_collection_claim === true ? "true" : "false"}`,
    `followup\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `followup_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    "",
    "triage_id\tlane\tsource_anchor\twrite_claim",
    ...rows.map((row) => [
      row.id,
      row.operations_lane,
      row.source_anchor,
      row.write_claim === true ? "true" : "false",
    ].join("\t")),
  ].join("\n");
}

export function renderClassroomOperationsTriage(root, triage = {}) {
  if (!root) return null;
  const payload = asObject(triage);
  const rows = asArray(payload.triage_rows);
  const activeId = asText(payload.active_triage_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.classroomOperationsTriageStatus = asText(payload.status, "classroom_operations_triage_incomplete");
  root.innerHTML = `
    <div class="classroom-triage-head">
      <div>
        <div class="classroom-triage-kicker">Classroom operations</div>
        <h2>Triage packet</h2>
      </div>
      <div class="classroom-triage-progress" data-classroom-triage-progress>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} follow-up</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>triage-only</span>
      </div>
    </div>
    <div class="classroom-triage-summary" data-classroom-triage-summary>
      ${escapeHtml(String(payload.triage_row_count ?? 0))} triage rows · generated_now=false · write_claim=false
    </div>
    <div class="classroom-triage-body">
      <div class="classroom-triage-list" data-classroom-triage-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="classroom-triage-btn${row.id === activeId ? " active" : ""}"
            data-classroom-triage="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.operations_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="classroom-triage-detail" data-classroom-triage-detail>
        <div class="classroom-triage-title" data-classroom-triage-active-title>${escapeHtml(active.title)}</div>
        <p data-classroom-triage-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-classroom-triage-active-lane>${escapeHtml(active.operations_lane)}</dd></div>
          <div><dt>source</dt><dd data-classroom-triage-active-source>${escapeHtml(active.source_anchor)}</dd></div>
          <div><dt>boundary</dt><dd>triage_only=true</dd></div>
        </dl>
        <button type="button" class="ghost" data-classroom-triage-copy>triage 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderClassroomOperationsTriage(root, {
      ...payload,
      active_triage_id: rowId,
    });
  };
  root.querySelectorAll("[data-classroom-triage]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-classroom-triage") || "");
    });
  });
  root.querySelector("[data-classroom-triage-copy]")?.addEventListener("click", async () => {
    const text = formatClassroomOperationsTriageText(payload);
    root.dataset.classroomOperationsTriageCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
