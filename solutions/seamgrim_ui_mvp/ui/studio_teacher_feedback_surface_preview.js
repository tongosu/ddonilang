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

const SECTION_DEFS = [
  {
    id: "teacher_summary_panel",
    source_seed_row: "teacher_summary_note",
    surface_lane: "teacher_summary",
    title: "교사용 요약",
    summary: "실행 결과와 수업 관찰 지점을 교사용 메모로 묶습니다.",
    feedback_surface: "teacher_notes.md",
  },
  {
    id: "student_next_step_panel",
    source_seed_row: "student_next_step_note",
    surface_lane: "student_next_step",
    title: "학생 다음 단계",
    summary: "학생에게 바로 제시할 다음 활동 문장을 분리합니다.",
    feedback_surface: "student_sheet.md",
  },
  {
    id: "misconception_review_panel",
    source_seed_row: "misconception_marker",
    surface_lane: "misconception_review",
    title: "오개념 검토",
    summary: "실패 케이스와 불일치 신호를 교사용 검토 항목으로 정리합니다.",
    feedback_surface: "teacher_notes.md",
  },
  {
    id: "retry_prompt_panel",
    source_seed_row: "retry_prompt",
    surface_lane: "retry_prompt",
    title: "재시도 프롬프트",
    summary: "학생이 다시 실행해 볼 조건과 입력 조절 방향을 제안합니다.",
    feedback_surface: "student_sheet.md",
  },
  {
    id: "publication_candidate_panel",
    source_seed_row: "publication_candidate_feedback",
    surface_lane: "publication_candidate",
    title: "공개 후보 피드백",
    summary: "공개 후보 교과의 검토 메모를 로컬 preview로만 유지합니다.",
    feedback_surface: "teacher_notes.md",
  },
  {
    id: "approval_safe_handoff_panel",
    source_seed_row: "approval_safe_handoff_note",
    surface_lane: "approval_safe_handoff",
    title: "승인 안전 인계",
    summary: "릴리스 승인과 실행을 분리한 인계 문구를 고정합니다.",
    feedback_surface: "teacher_notes.md",
  },
];

export const DEFAULT_TEACHER_FEEDBACK_SEED_ROWS = SECTION_DEFS.map((section) => ({
  id: section.source_seed_row,
  feedback_surface: section.feedback_surface,
  intended_artifact: section.source_seed_row,
  seed_only: true,
  generated_now: false,
  write_claim: false,
}));

function normalizeSeedRows(seedRows = []) {
  const rowsById = new Map(asArray(seedRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return SECTION_DEFS.map((section) => {
    const source = rowsById.get(section.source_seed_row) || {};
    return {
      id: section.id,
      title: section.title,
      summary: section.summary,
      source_seed_row: section.source_seed_row,
      source_anchor: asText(source.source_anchor, "local_preview"),
      surface_lane: section.surface_lane,
      feedback_surface: asText(source.feedback_surface, section.feedback_surface),
      intended_artifact: asText(source.intended_artifact, section.source_seed_row),
      preview_surface: "local_teacher_feedback_preview",
      preview_only: true,
      generated_now: asBool(source.generated_now),
      write_claim: asBool(source.write_claim),
      product_ui_change: true,
      teacher_feedback_runtime_claim: false,
      student_data_collection_claim: false,
      cloud_sync_claim: false,
      account_setup_claim: false,
      permission_system_claim: false,
      result_replay_claim: false,
    };
  });
}

export function buildTeacherFeedbackSurfacePreview({
  seedRows = DEFAULT_TEACHER_FEEDBACK_SEED_ROWS,
  activeSectionId = "teacher_summary_panel",
} = {}) {
  const previewSections = normalizeSeedRows(seedRows);
  const active = previewSections.some((section) => section.id === activeSectionId)
    ? activeSectionId
    : previewSections[0]?.id ?? "";
  const localOnly = previewSections.every((section) => (
    section.preview_only === true &&
    section.generated_now === false &&
    section.write_claim === false
  ));
  const stages = [
    ["seed_alignment", previewSections.length === SECTION_DEFS.length],
    ["surface_render_model", active.length > 0],
    ["local_only_boundary", localOnly],
    ["no_student_collection", true],
    ["no_feedback_write", true],
    ["no_remote_runtime", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_teacher_feedback_surface_preview",
    schema: "ddn.studio.teacher_feedback_surface_preview.v1",
    work_item: "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
    primary_coordinate: "하-3",
    support_coordinate: "마-3",
    workflow_claim: "teacher_feedback_surface_preview",
    generated_locally: true,
    product_ui_change: true,
    preview_only: true,
    runtime_claim: false,
    teacher_feedback_runtime_claim: false,
    student_data_collection_claim: false,
    feedback_write_claim: false,
    remote_save_claim: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    result_replay_claim: false,
    status: ready_stage_count === stages.length ? "teacher_feedback_preview_ready" : "teacher_feedback_preview_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    preview_section_count: previewSections.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_section_id: active,
    preview_sections: previewSections,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 2,
      current_stage_total: 8,
      current_stage_percent: 25,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
  };
}

export function formatTeacherFeedbackSurfacePreviewText(preview = {}) {
  const payload = asObject(preview);
  if (payload.schema !== "ddn.studio.teacher_feedback_surface_preview.v1") {
    throw new Error("seamgrim_expected_teacher_feedback_surface_preview");
  }
  const sections = asArray(payload.preview_sections);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `runtime_claim\t${payload.runtime_claim === true ? "true" : "false"}`,
    `student_data_collection_claim\t${payload.student_data_collection_claim === true ? "true" : "false"}`,
    `feedback_write_claim\t${payload.feedback_write_claim === true ? "true" : "false"}`,
    `preview_section_count\t${payload.preview_section_count ?? sections.length}`,
    "",
    "section_id\tsurface_lane\tfeedback_surface\twrite_claim",
    ...sections.map((section) => [
      section.id,
      section.surface_lane,
      section.feedback_surface,
      section.write_claim === true ? "true" : "false",
    ].join("\t")),
  ].join("\n");
}

export function renderTeacherFeedbackSurfacePreview(root, preview = {}) {
  if (!root) return null;
  const payload = asObject(preview);
  const sections = asArray(payload.preview_sections);
  const activeId = asText(payload.active_section_id, sections[0]?.id ?? "");
  const active = sections.find((section) => section.id === activeId) || sections[0] || {};
  root.dataset.teacherFeedbackStatus = asText(payload.status, "teacher_feedback_preview_incomplete");
  root.innerHTML = `
    <div class="teacher-feedback-preview-head">
      <div>
        <div class="teacher-feedback-preview-kicker">교사 피드백</div>
        <h2>로컬 피드백 미리보기</h2>
      </div>
      <div class="teacher-feedback-preview-progress" data-teacher-feedback-progress>
        <span>${escapeHtml(String(payload.preview_section_count ?? sections.length))}개 섹션</span>
        <span>쓰기 없음</span>
        <span>계정 없음</span>
      </div>
    </div>
    <div class="teacher-feedback-preview-body">
      <div class="teacher-feedback-section-list" data-teacher-feedback-section-list>
        ${sections.map((section) => `
          <button
            type="button"
            class="teacher-feedback-section-btn${section.id === activeId ? " active" : ""}"
            data-teacher-feedback-section="${escapeHtml(section.id)}"
          >
            <span>${escapeHtml(section.title)}</span>
            <small>${escapeHtml(section.feedback_surface)}</small>
          </button>
        `).join("")}
      </div>
      <div class="teacher-feedback-detail" data-teacher-feedback-detail>
        <div class="teacher-feedback-detail-title" data-teacher-feedback-active-title>${escapeHtml(active.title)}</div>
        <p data-teacher-feedback-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-teacher-feedback-active-lane>${escapeHtml(active.surface_lane)}</dd></div>
          <div><dt>seed</dt><dd data-teacher-feedback-active-seed>${escapeHtml(active.source_seed_row)}</dd></div>
          <div><dt>boundary</dt><dd>local preview only · no write</dd></div>
        </dl>
        <button type="button" class="ghost" data-teacher-feedback-copy>미리보기 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (sectionId) => {
    const next = {
      ...payload,
      active_section_id: sectionId,
    };
    renderTeacherFeedbackSurfacePreview(root, next);
  };
  root.querySelectorAll("[data-teacher-feedback-section]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-teacher-feedback-section") || "");
    });
  });
  root.querySelector("[data-teacher-feedback-copy]")?.addEventListener("click", async () => {
    const text = formatTeacherFeedbackSurfacePreviewText(payload);
    root.dataset.teacherFeedbackCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
