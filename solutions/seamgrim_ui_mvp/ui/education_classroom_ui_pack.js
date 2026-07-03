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

const CLASSROOM_ROWS = [
  { id: "teacher_mode", title: "Teacher mode", classroom_kind: "teacher_mode_preview", local_uri: "education://classroom/teacher-mode" },
  { id: "student_mode", title: "Student mode", classroom_kind: "student_mode_preview", local_uri: "education://classroom/student-mode" },
  { id: "assignment_panel", title: "Assignment panel", classroom_kind: "assignment_panel_preview", local_uri: "education://classroom/assignment-panel" },
  { id: "report_review", title: "Report review", classroom_kind: "report_review_preview", local_uri: "education://classroom/report-review" },
  { id: "classroom_handoff", title: "Classroom handoff", classroom_kind: "local_classroom_handoff", local_uri: "education://classroom/handoff" },
];

const DEFAULT_ARTIFACTS = [
  { name: "education.classroom.teacher_mode.detjson", kind: "teacher_mode", bytes: 812 },
  { name: "education.classroom.student_mode.detjson", kind: "student_mode", bytes: 798 },
  { name: "education.classroom.assignment_panel.detjson", kind: "assignment_panel", bytes: 856 },
  { name: "education.classroom.report_review.detjson", kind: "report_review", bytes: 772 },
  { name: "education.classroom.handoff.detjson", kind: "classroom_handoff", bytes: 704 },
];

export const DEFAULT_EDUCATION_CLASSROOM_UI_ROWS = CLASSROOM_ROWS.map((row) => ({
  id: row.id,
  classroom_kind: row.classroom_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_EDUCATION_CLASSROOM_UI_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return CLASSROOM_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      classroom_kind: asText(source.classroom_kind, row.classroom_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_classroom_ui_claim: true,
      remote_classroom_runtime_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `education_classroom_ui_${index + 1}.detjson`),
      kind: asText(source.kind, "education_classroom_ui_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildClassroomText(rows, artifacts) {
  return [
    "education_classroom_ui_pack:education_curriculum_3_v1",
    "coordinate:하-3",
    "gradebook_write:false",
    "student_personal_data_collection:false",
    "remote_classroom_sync:false",
    "live_submission:false",
    "real_time_collaboration:false",
    "account_permission_change:false",
    "state_hash_participation:false",
    ...rows.map((row) => `${row.id}\t${row.classroom_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildEducationClassroomUiPack({
  rows = DEFAULT_EDUCATION_CLASSROOM_UI_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "teacher_mode",
} = {}) {
  const classroomRows = normalizeRows(rows);
  const classroomArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(classroomArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = classroomRows.filter((row) => row.ready).length;
  const readyArtifactCount = classroomArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["teacher_mode", "student_mode", "assignment_panel", "report_review", "classroom_handoff"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === classroomRows.length &&
    readyArtifactCount === classroomArtifacts.length &&
    hasAllArtifacts;
  const active = classroomRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : classroomRows[0]?.id ?? "";
  return {
    __종류: "education_classroom_ui_pack",
    schema: "ddn.education.classroom_ui_pack.v1",
    work_item: "HA3_CLASSROOM_UI_PACK_V1",
    primary_coordinate: "하-3",
    depends_on_coordinate: ["하-2", "마-3", "타-3"],
    pack: "education_curriculum_3_v1",
    status: ready ? "education_classroom_ui_pack_ready" : "education_classroom_ui_pack_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    education_classroom_ui_pack_claim: ready,
    teacher_mode_claim: artifactKinds.has("teacher_mode"),
    student_mode_claim: artifactKinds.has("student_mode"),
    assignment_panel_claim: artifactKinds.has("assignment_panel"),
    report_review_claim: artifactKinds.has("report_review"),
    classroom_handoff_claim: artifactKinds.has("classroom_handoff"),
    gradebook_write_claim: false,
    student_personal_data_collection_claim: false,
    remote_classroom_sync_claim: false,
    live_submission_claim: false,
    real_time_collaboration_claim: false,
    account_permission_change_claim: false,
    state_hash_participation_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: classroomRows,
    artifacts: classroomArtifacts,
    active_row_id: active,
    classroom_text: buildClassroomText(classroomRows, classroomArtifacts),
    artifact_size_bytes: classroomArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 30,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 33,
      roadmap_v2_pack_evidence_reference_closed: 50,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 56,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1",
  };
}

export function formatEducationClassroomUiPackText(classroom = {}) {
  const payload = asObject(classroom);
  if (payload.schema !== "ddn.education.classroom_ui_pack.v1") {
    throw new Error("education_expected_classroom_ui_pack");
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
    `education_classroom_ui_pack_claim\t${payload.education_classroom_ui_pack_claim === true ? "true" : "false"}`,
    `teacher_mode_claim\t${payload.teacher_mode_claim === true ? "true" : "false"}`,
    `student_mode_claim\t${payload.student_mode_claim === true ? "true" : "false"}`,
    `assignment_panel_claim\t${payload.assignment_panel_claim === true ? "true" : "false"}`,
    `report_review_claim\t${payload.report_review_claim === true ? "true" : "false"}`,
    `classroom_handoff_claim\t${payload.classroom_handoff_claim === true ? "true" : "false"}`,
    `gradebook_write_claim\t${payload.gradebook_write_claim === true ? "true" : "false"}`,
    `student_personal_data_collection_claim\t${payload.student_personal_data_collection_claim === true ? "true" : "false"}`,
    `remote_classroom_sync_claim\t${payload.remote_classroom_sync_claim === true ? "true" : "false"}`,
    `real_time_collaboration_claim\t${payload.real_time_collaboration_claim === true ? "true" : "false"}`,
    "",
    "row_id\tclassroom_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.classroom_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderEducationClassroomUiPack(root, classroom = {}) {
  if (!root) return null;
  const payload = asObject(classroom);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.educationClassroomUiPackStatus = asText(payload.status, "education_classroom_ui_pack_incomplete");
  root.innerHTML = `
    <div class="education-classroom-head">
      <div>
        <div class="education-classroom-kicker">Education Classroom UI</div>
        <h2>Education classroom UI pack</h2>
      </div>
      <div class="education-classroom-progress" data-education-classroom-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="education-classroom-summary" data-education-classroom-summary>
      teacher mode, student mode, assignment panel, report review, classroom handoff를 로컬 UI pack으로 고정하고 gradebook write, student personal data collection, remote classroom sync, real-time collaboration은 후속으로 둡니다.
    </div>
    <div class="education-classroom-body">
      <div class="education-classroom-list">
        ${rows.map((row) => `
          <button type="button" class="education-classroom-btn${row.id === activeId ? " active" : ""}" data-education-classroom-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.classroom_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="education-classroom-detail">
        <div class="education-classroom-title" data-education-classroom-active-title>${escapeHtml(active.title)}</div>
        <p data-education-classroom-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="education-classroom-artifacts">
          ${artifacts.map((artifact) => `
            <span data-education-classroom-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="education-classroom-preview" data-education-classroom-preview>${escapeHtml(payload.classroom_text ?? "")}</pre>
        <button type="button" class="ghost" data-education-classroom-copy>Classroom UI pack 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderEducationClassroomUiPack(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-education-classroom-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-education-classroom-row") || ""));
  });
  root.querySelector("[data-education-classroom-copy]")?.addEventListener("click", async () => {
    root.dataset.educationClassroomUiPackCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatEducationClassroomUiPackText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
