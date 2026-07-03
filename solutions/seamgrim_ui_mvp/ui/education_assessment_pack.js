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

const ASSESSMENT_ROWS = [
  { id: "teacher_notes", title: "Teacher notes", assessment_kind: "teacher_notes_alignment", local_uri: "education://assessment/teacher-notes" },
  { id: "student_sheet", title: "Student sheet", assessment_kind: "student_sheet_alignment", local_uri: "education://assessment/student-sheet" },
  { id: "rubric_mapping", title: "Rubric mapping", assessment_kind: "rubric_criteria_mapping", local_uri: "education://assessment/rubric-mapping" },
  { id: "sample_submission", title: "Sample submission", assessment_kind: "local_sample_submission_fixture", local_uri: "education://assessment/sample-submission" },
  { id: "pack_handoff", title: "Pack handoff", assessment_kind: "lesson_pack_handoff", local_uri: "education://assessment/pack-handoff" },
];

const DEFAULT_ARTIFACTS = [
  { name: "education.teacher_notes.detjson", kind: "teacher_notes", bytes: 748 },
  { name: "education.student_sheet.detjson", kind: "student_sheet", bytes: 812 },
  { name: "education.rubric_mapping.detjson", kind: "rubric_mapping", bytes: 780 },
  { name: "education.sample_submission.detjson", kind: "sample_submission", bytes: 694 },
  { name: "education.pack_handoff.detjson", kind: "pack_handoff", bytes: 736 },
];

export const DEFAULT_EDUCATION_ASSESSMENT_ROWS = ASSESSMENT_ROWS.map((row) => ({
  id: row.id,
  assessment_kind: row.assessment_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_EDUCATION_ASSESSMENT_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return ASSESSMENT_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      assessment_kind: asText(source.assessment_kind, row.assessment_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_assessment_claim: true,
      remote_gradebook_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `education_assessment_${index + 1}.detjson`),
      kind: asText(source.kind, "education_assessment_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildAssessmentText(rows, artifacts) {
  return [
    "education_assessment_pack:education_curriculum_2_v1",
    "coordinate:하-2",
    "gradebook_write:false",
    "student_personal_data_collection:false",
    "remote_classroom_sync:false",
    "live_submission:false",
    "account_permission_change:false",
    "state_hash_participation:false",
    ...rows.map((row) => `${row.id}\t${row.assessment_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildEducationAssessmentPack({
  rows = DEFAULT_EDUCATION_ASSESSMENT_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "teacher_notes",
} = {}) {
  const assessmentRows = normalizeRows(rows);
  const assessmentArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(assessmentArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = assessmentRows.filter((row) => row.ready).length;
  const readyArtifactCount = assessmentArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["teacher_notes", "student_sheet", "rubric_mapping", "sample_submission", "pack_handoff"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === assessmentRows.length &&
    readyArtifactCount === assessmentArtifacts.length &&
    hasAllArtifacts;
  const active = assessmentRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : assessmentRows[0]?.id ?? "";
  return {
    __종류: "education_assessment_pack",
    schema: "ddn.education.assessment_pack.v1",
    work_item: "HA2_EDUCATION_ASSESSMENT_PACK_V1",
    primary_coordinate: "하-2",
    depends_on_coordinate: ["타-2", "하-1", "마-2"],
    pack: "education_curriculum_2_v1",
    status: ready ? "education_assessment_pack_ready" : "education_assessment_pack_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    education_assessment_pack_claim: ready,
    teacher_notes_claim: artifactKinds.has("teacher_notes"),
    student_sheet_claim: artifactKinds.has("student_sheet"),
    rubric_mapping_claim: artifactKinds.has("rubric_mapping"),
    sample_submission_claim: artifactKinds.has("sample_submission"),
    pack_handoff_claim: artifactKinds.has("pack_handoff"),
    gradebook_write_claim: false,
    student_personal_data_collection_claim: false,
    remote_classroom_sync_claim: false,
    live_submission_claim: false,
    account_permission_change_claim: false,
    state_hash_participation_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: assessmentRows,
    artifacts: assessmentArtifacts,
    active_row_id: active,
    assessment_text: buildAssessmentText(assessmentRows, assessmentArtifacts),
    artifact_size_bytes: assessmentArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 29,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 32,
      roadmap_v2_pack_evidence_reference_closed: 49,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 54,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "HA3_CLASSROOM_UI_PACK_V1",
  };
}

export function formatEducationAssessmentPackText(assessment = {}) {
  const payload = asObject(assessment);
  if (payload.schema !== "ddn.education.assessment_pack.v1") {
    throw new Error("education_expected_assessment_pack");
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
    `education_assessment_pack_claim\t${payload.education_assessment_pack_claim === true ? "true" : "false"}`,
    `teacher_notes_claim\t${payload.teacher_notes_claim === true ? "true" : "false"}`,
    `student_sheet_claim\t${payload.student_sheet_claim === true ? "true" : "false"}`,
    `rubric_mapping_claim\t${payload.rubric_mapping_claim === true ? "true" : "false"}`,
    `sample_submission_claim\t${payload.sample_submission_claim === true ? "true" : "false"}`,
    `pack_handoff_claim\t${payload.pack_handoff_claim === true ? "true" : "false"}`,
    `gradebook_write_claim\t${payload.gradebook_write_claim === true ? "true" : "false"}`,
    `student_personal_data_collection_claim\t${payload.student_personal_data_collection_claim === true ? "true" : "false"}`,
    `remote_classroom_sync_claim\t${payload.remote_classroom_sync_claim === true ? "true" : "false"}`,
    `live_submission_claim\t${payload.live_submission_claim === true ? "true" : "false"}`,
    "",
    "row_id\tassessment_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.assessment_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderEducationAssessmentPack(root, assessment = {}) {
  if (!root) return null;
  const payload = asObject(assessment);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.educationAssessmentPackStatus = asText(payload.status, "education_assessment_pack_incomplete");
  root.innerHTML = `
    <div class="education-assessment-head">
      <div>
        <div class="education-assessment-kicker">Education Assessment</div>
        <h2>Education assessment pack</h2>
      </div>
      <div class="education-assessment-progress" data-education-assessment-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="education-assessment-summary" data-education-assessment-summary>
      teacher notes, student sheet, rubric mapping, sample submission, pack handoff를 로컬 pack으로 고정하고 gradebook write, student personal data collection, remote classroom sync는 후속으로 둡니다.
    </div>
    <div class="education-assessment-body">
      <div class="education-assessment-list">
        ${rows.map((row) => `
          <button type="button" class="education-assessment-btn${row.id === activeId ? " active" : ""}" data-education-assessment-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.assessment_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="education-assessment-detail">
        <div class="education-assessment-title" data-education-assessment-active-title>${escapeHtml(active.title)}</div>
        <p data-education-assessment-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="education-assessment-artifacts">
          ${artifacts.map((artifact) => `
            <span data-education-assessment-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="education-assessment-preview" data-education-assessment-preview>${escapeHtml(payload.assessment_text ?? "")}</pre>
        <button type="button" class="ghost" data-education-assessment-copy>Assessment pack 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderEducationAssessmentPack(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-education-assessment-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-education-assessment-row") || ""));
  });
  root.querySelector("[data-education-assessment-copy]")?.addEventListener("click", async () => {
    root.dataset.educationAssessmentPackCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatEducationAssessmentPackText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
