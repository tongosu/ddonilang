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

const SEED_DEFS = [
  {
    id: "teacher_summary_note",
    source_anchor: "classroom_report_workflow",
    feedback_surface: "teacher_notes.md",
    title: "teacher summary",
    summary: "교사용 요약 노트 후보만 고정하고 feedback write는 수행하지 않습니다.",
  },
  {
    id: "student_next_step_note",
    source_anchor: "classroom_report_workflow",
    feedback_surface: "student_sheet.md",
    title: "student next step",
    summary: "학생 다음 단계 문장 seed만 남기고 student data collection은 열지 않습니다.",
  },
  {
    id: "misconception_marker",
    source_anchor: "classroom_report_workflow",
    feedback_surface: "teacher_notes.md",
    title: "misconception marker",
    summary: "오개념 표시 후보를 teacher_notes 표면에 묶되 원격 저장은 하지 않습니다.",
  },
  {
    id: "retry_prompt",
    source_anchor: "classroom_report_workflow",
    feedback_surface: "student_sheet.md",
    title: "retry prompt",
    summary: "재시도 안내 후보만 기록하고 result replay나 자동 patch 적용은 막습니다.",
  },
  {
    id: "publication_candidate_feedback",
    source_anchor: "publication_artifact_dry_run",
    feedback_surface: "teacher_notes.md",
    title: "publication candidate",
    summary: "공개 후보 피드백 seed는 publication dry-run evidence만 참조합니다.",
  },
  {
    id: "approval_safe_handoff_note",
    source_anchor: "publication_artifact_dry_run",
    feedback_surface: "teacher_notes.md",
    title: "approval handoff",
    summary: "승인 안전 인수 노트는 release approval이나 execution claim 없이 유지됩니다.",
  },
];

export const DEFAULT_TEACHER_FEEDBACK_LOOP_SEED_ROWS = SEED_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  feedback_surface: row.feedback_surface,
  intended_artifact: row.id,
  seed_only: true,
  generated_now: false,
  write_claim: false,
  product_ui_change: true,
  teacher_feedback_runtime_claim: false,
  student_data_collection_claim: false,
}));

function normalizeRows(rows = DEFAULT_TEACHER_FEEDBACK_LOOP_SEED_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return SEED_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      feedback_surface: asText(source.feedback_surface, def.feedback_surface),
      intended_artifact: asText(source.intended_artifact, def.id),
      seed_surface: "local_studio_teacher_feedback_loop_seed",
      seed_only: true,
      generated_now: asBool(source.generated_now),
      write_claim: false,
      teacher_feedback_runtime_claim: false,
      student_data_collection_claim: false,
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

export function buildTeacherFeedbackLoopSeed({
  seedRows = DEFAULT_TEACHER_FEEDBACK_LOOP_SEED_ROWS,
  activeSeedId = "teacher_summary_note",
} = {}) {
  const rows = normalizeRows(seedRows);
  const active = rows.some((row) => row.id === activeSeedId)
    ? activeSeedId
    : rows[0]?.id ?? "";
  const seedOnly = rows.every((row) => (
    row.seed_only === true &&
    row.generated_now === false &&
    row.write_claim === false &&
    row.teacher_feedback_runtime_claim === false &&
    row.student_data_collection_claim === false &&
    row.remote_save_claim === false &&
    row.cloud_sync_claim === false &&
    row.permission_system_claim === false &&
    row.release_execution_claim === false
  ));
  const stages = [
    ["seed_row_alignment", rows.length === SEED_DEFS.length],
    ["teacher_summary_seeded", rows.some((row) => row.id === "teacher_summary_note")],
    ["student_next_step_seeded", rows.some((row) => row.id === "student_next_step_note")],
    ["misconception_marker_seeded", rows.some((row) => row.id === "misconception_marker")],
    ["publication_handoff_seeded", rows.some((row) => row.id === "approval_safe_handoff_note")],
    ["write_boundary_blocked", seedOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_teacher_feedback_loop_seed",
    schema: "ddn.studio.teacher_feedback_loop_seed.v1",
    work_item: "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
    based_on: "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
    workflow_claim: "teacher_feedback_loop_seed",
    primary_coordinate: "하-3",
    support_coordinate: "마-3",
    product_ui_change: true,
    product_code_change: true,
    teacher_feedback_loop_seed_claim: true,
    runtime_claim: false,
    teacher_feedback_runtime_claim: false,
    student_data_collection_claim: false,
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
    seed_row_count: rows.length,
    all_seed_rows_seed_only: true,
    all_seed_rows_generated_now: false,
    all_seed_rows_write_claim: false,
    status: ready_stage_count === stages.length ? "teacher_feedback_loop_seed_ready" : "teacher_feedback_loop_seed_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_seed_id: active,
    seed_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 5,
      current_stage_total: 8,
      current_stage_percent: 63,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
  };
}

export function formatTeacherFeedbackLoopSeedText(seed = {}) {
  const payload = asObject(seed);
  if (payload.schema !== "ddn.studio.teacher_feedback_loop_seed.v1") {
    throw new Error("seamgrim_expected_teacher_feedback_loop_seed");
  }
  const rows = asArray(payload.seed_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `seed_row_count\t${payload.seed_row_count ?? rows.length}`,
    `all_seed_rows_seed_only\t${payload.all_seed_rows_seed_only === true ? "true" : "false"}`,
    `all_seed_rows_generated_now\t${payload.all_seed_rows_generated_now === true ? "true" : "false"}`,
    `all_seed_rows_write_claim\t${payload.all_seed_rows_write_claim === true ? "true" : "false"}`,
    `teacher_feedback_runtime_claim\t${payload.teacher_feedback_runtime_claim === true ? "true" : "false"}`,
    `student_data_collection_claim\t${payload.student_data_collection_claim === true ? "true" : "false"}`,
    `followup\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `followup_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    "",
    "seed_id\tsurface\tsource_anchor\twrite_claim",
    ...rows.map((row) => [
      row.id,
      row.feedback_surface,
      row.source_anchor,
      row.write_claim === true ? "true" : "false",
    ].join("\t")),
  ].join("\n");
}

export function renderTeacherFeedbackLoopSeed(root, seed = {}) {
  if (!root) return null;
  const payload = asObject(seed);
  const rows = asArray(payload.seed_rows);
  const activeId = asText(payload.active_seed_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.teacherFeedbackLoopSeedStatus = asText(payload.status, "teacher_feedback_loop_seed_incomplete");
  root.innerHTML = `
    <div class="teacher-loop-seed-head">
      <div>
        <div class="teacher-loop-seed-kicker">Teacher feedback</div>
        <h2>Loop seed rows</h2>
      </div>
      <div class="teacher-loop-seed-progress" data-teacher-loop-seed-progress>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} follow-up</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>seed-only</span>
      </div>
    </div>
    <div class="teacher-loop-seed-summary" data-teacher-loop-seed-summary>
      ${escapeHtml(String(payload.seed_row_count ?? 0))} seed rows · generated_now=false · write_claim=false
    </div>
    <div class="teacher-loop-seed-body">
      <div class="teacher-loop-seed-list" data-teacher-loop-seed-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="teacher-loop-seed-btn${row.id === activeId ? " active" : ""}"
            data-teacher-loop-seed="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.feedback_surface)}</small>
          </button>
        `).join("")}
      </div>
      <div class="teacher-loop-seed-detail" data-teacher-loop-seed-detail>
        <div class="teacher-loop-seed-title" data-teacher-loop-seed-active-title>${escapeHtml(active.title)}</div>
        <p data-teacher-loop-seed-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>surface</dt><dd data-teacher-loop-seed-active-surface>${escapeHtml(active.feedback_surface)}</dd></div>
          <div><dt>source</dt><dd data-teacher-loop-seed-active-source>${escapeHtml(active.source_anchor)}</dd></div>
          <div><dt>boundary</dt><dd>write_claim=false</dd></div>
        </dl>
        <button type="button" class="ghost" data-teacher-loop-seed-copy>feedback seed 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderTeacherFeedbackLoopSeed(root, {
      ...payload,
      active_seed_id: rowId,
    });
  };
  root.querySelectorAll("[data-teacher-loop-seed]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-teacher-loop-seed") || "");
    });
  });
  root.querySelector("[data-teacher-loop-seed-copy]")?.addEventListener("click", async () => {
    const text = formatTeacherFeedbackLoopSeedText(payload);
    root.dataset.teacherFeedbackLoopSeedCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
