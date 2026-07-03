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

const OPERATION_ROWS = [
  { id: "submission_versioning", title: "Submission versioning", operation_kind: "local_submission_versioning", local_uri: "education://operations/submission-versioning" },
  { id: "assessment_archive", title: "Assessment archive", operation_kind: "local_assessment_archive", local_uri: "education://operations/assessment-archive" },
  { id: "curriculum_version_lock", title: "Curriculum version lock", operation_kind: "curriculum_version_lock", local_uri: "education://operations/curriculum-version-lock" },
  { id: "lts_gate", title: "LTS gate", operation_kind: "local_lts_gate", local_uri: "education://operations/lts-gate" },
  { id: "operations_handoff", title: "Operations handoff", operation_kind: "operations_handoff_packet", local_uri: "education://operations/handoff" },
];

const DEFAULT_ARTIFACTS = [
  { name: "education.operations.submission_versioning.detjson", kind: "submission_versioning", bytes: 826 },
  { name: "education.operations.assessment_archive.detjson", kind: "assessment_archive", bytes: 812 },
  { name: "education.operations.curriculum_version_lock.detjson", kind: "curriculum_version_lock", bytes: 846 },
  { name: "education.operations.lts_gate.detjson", kind: "lts_gate", bytes: 794 },
  { name: "education.operations.handoff.detjson", kind: "operations_handoff", bytes: 768 },
];

export const DEFAULT_EDUCATION_OPERATIONS_LTS_ROWS = OPERATION_ROWS.map((row) => ({
  id: row.id,
  operation_kind: row.operation_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_EDUCATION_OPERATIONS_LTS_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return OPERATION_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      operation_kind: asText(source.operation_kind, row.operation_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_operations_claim: true,
      remote_lts_certification_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `education_operations_${index + 1}.detjson`),
      kind: asText(source.kind, "education_operations_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildOperationsText(rows, artifacts) {
  return [
    "education_operations_lts:education_curriculum_5_v1",
    "coordinate:하-5",
    "remote_lts_certification:false",
    "live_submission:false",
    "gradebook_write:false",
    "student_personal_data_collection:false",
    "remote_classroom_sync:false",
    "release_execution:false",
    "registry_publish:false",
    "account_permission_change:false",
    "state_hash_participation:false",
    ...rows.map((row) => `${row.id}\t${row.operation_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildEducationOperationsLts({
  rows = DEFAULT_EDUCATION_OPERATIONS_LTS_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "submission_versioning",
} = {}) {
  const operationRows = normalizeRows(rows);
  const operationArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(operationArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = operationRows.filter((row) => row.ready).length;
  const readyArtifactCount = operationArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["submission_versioning", "assessment_archive", "curriculum_version_lock", "lts_gate", "operations_handoff"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === operationRows.length &&
    readyArtifactCount === operationArtifacts.length &&
    hasAllArtifacts;
  const active = operationRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : operationRows[0]?.id ?? "";
  return {
    __종류: "education_operations_lts",
    schema: "ddn.education.operations_lts.v1",
    work_item: "HA5_EDUCATION_OPERATIONS_LTS_V1",
    primary_coordinate: "하-5",
    depends_on_coordinate: ["하-4", "마-5", "타-5"],
    pack: "education_curriculum_5_v1",
    status: ready ? "education_operations_lts_ready" : "education_operations_lts_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    education_operations_lts_claim: ready,
    submission_versioning_claim: artifactKinds.has("submission_versioning"),
    assessment_archive_claim: artifactKinds.has("assessment_archive"),
    curriculum_version_lock_claim: artifactKinds.has("curriculum_version_lock"),
    lts_gate_claim: artifactKinds.has("lts_gate"),
    operations_handoff_claim: artifactKinds.has("operations_handoff"),
    remote_lts_certification_claim: false,
    live_submission_claim: false,
    gradebook_write_claim: false,
    student_personal_data_collection_claim: false,
    remote_classroom_sync_claim: false,
    release_execution_claim: false,
    registry_publish_claim: false,
    account_permission_change_claim: false,
    state_hash_participation_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: operationRows,
    artifacts: operationArtifacts,
    active_row_id: active,
    operations_text: buildOperationsText(operationRows, operationArtifacts),
    artifact_size_bytes: operationArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 32,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 36,
      roadmap_v2_pack_evidence_reference_closed: 52,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 58,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "GEO0_AI_QUESTION_CARD_SEED_V1",
  };
}

export function formatEducationOperationsLtsText(operations = {}) {
  const payload = asObject(operations);
  if (payload.schema !== "ddn.education.operations_lts.v1") {
    throw new Error("education_expected_operations_lts");
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
    `education_operations_lts_claim\t${payload.education_operations_lts_claim === true ? "true" : "false"}`,
    `submission_versioning_claim\t${payload.submission_versioning_claim === true ? "true" : "false"}`,
    `assessment_archive_claim\t${payload.assessment_archive_claim === true ? "true" : "false"}`,
    `curriculum_version_lock_claim\t${payload.curriculum_version_lock_claim === true ? "true" : "false"}`,
    `lts_gate_claim\t${payload.lts_gate_claim === true ? "true" : "false"}`,
    `operations_handoff_claim\t${payload.operations_handoff_claim === true ? "true" : "false"}`,
    `remote_lts_certification_claim\t${payload.remote_lts_certification_claim === true ? "true" : "false"}`,
    `live_submission_claim\t${payload.live_submission_claim === true ? "true" : "false"}`,
    `gradebook_write_claim\t${payload.gradebook_write_claim === true ? "true" : "false"}`,
    `remote_classroom_sync_claim\t${payload.remote_classroom_sync_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    "",
    "row_id\toperation_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.operation_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderEducationOperationsLts(root, operations = {}) {
  if (!root) return null;
  const payload = asObject(operations);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.educationOperationsLtsStatus = asText(payload.status, "education_operations_lts_incomplete");
  root.innerHTML = `
    <div class="education-operations-head">
      <div>
        <div class="education-operations-kicker">Education Operations</div>
        <h2>Education operations LTS</h2>
      </div>
      <div class="education-operations-progress" data-education-operations-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="education-operations-summary" data-education-operations-summary>
      Submission versioning, assessment archive, curriculum version lock, local LTS gate, operations handoff를 로컬 education operations LTS로 고정하고 remote LTS certification, live submission, gradebook write는 후속으로 둡니다.
    </div>
    <div class="education-operations-body">
      <div class="education-operations-list">
        ${rows.map((row) => `
          <button type="button" class="education-operations-btn${row.id === activeId ? " active" : ""}" data-education-operations-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.operation_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="education-operations-detail">
        <div class="education-operations-title" data-education-operations-active-title>${escapeHtml(active.title)}</div>
        <p data-education-operations-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="education-operations-artifacts">
          ${artifacts.map((artifact) => `
            <span data-education-operations-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="education-operations-preview" data-education-operations-preview>${escapeHtml(payload.operations_text ?? "")}</pre>
        <button type="button" class="ghost" data-education-operations-copy>Operations LTS 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderEducationOperationsLts(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-education-operations-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-education-operations-row") || ""));
  });
  root.querySelector("[data-education-operations-copy]")?.addEventListener("click", async () => {
    root.dataset.educationOperationsLtsCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatEducationOperationsLtsText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
