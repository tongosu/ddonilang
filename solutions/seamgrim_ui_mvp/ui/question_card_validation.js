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

const VALIDATION_ROWS = [
  { id: "ddn_lint", title: "DDN lint", validation_kind: "ai_generated_ddn_lint", local_uri: "question-card://validation/ddn-lint" },
  { id: "lesson_lint", title: "Lesson lint", validation_kind: "ai_generated_lesson_lint", local_uri: "question-card://validation/lesson-lint" },
  { id: "intent_alignment", title: "Intent alignment", validation_kind: "intent_alignment_check", local_uri: "question-card://validation/intent-alignment" },
  { id: "patch_safety", title: "Patch safety", validation_kind: "patch_safety_check", local_uri: "question-card://validation/patch-safety" },
  { id: "validation_receipt", title: "Validation receipt", validation_kind: "ai_output_validation_receipt", local_uri: "question-card://validation/receipt" },
];

const DEFAULT_ARTIFACTS = [
  { name: "question_card.validation.ddn_lint.detjson", kind: "ai_generated_ddn_lint", bytes: 742 },
  { name: "question_card.validation.lesson_lint.detjson", kind: "ai_generated_lesson_lint", bytes: 768 },
  { name: "question_card.validation.intent_alignment.detjson", kind: "intent_alignment_check", bytes: 806 },
  { name: "question_card.validation.patch_safety.detjson", kind: "patch_safety_check", bytes: 836 },
  { name: "question_card.validation.receipt.detjson", kind: "ai_output_validation_receipt", bytes: 724 },
];

export const DEFAULT_QUESTION_CARD_VALIDATION_ROWS = VALIDATION_ROWS.map((row) => ({
  id: row.id,
  validation_kind: row.validation_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_QUESTION_CARD_VALIDATION_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return VALIDATION_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      validation_kind: asText(source.validation_kind, row.validation_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      ai_output_validation_claim: true,
      ai_call_claim: false,
      patch_execution_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `question_card_validation_${index + 1}.detjson`),
      kind: asText(source.kind, "question_card_validation_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildValidationText(rows, artifacts) {
  return [
    "question_card_validation:question_card_validation_v1",
    "coordinate:거-2",
    "ai_call:false",
    "parser_preprocessor:false",
    "auto_apply:false",
    "patch_execution:false",
    "file_write:false",
    "network_call:false",
    "runtime_ast_persisted:false",
    "state_hash_owner:false",
    "model_training:false",
    "account_permission_change:false",
    ...rows.map((row) => `${row.id}\t${row.validation_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildQuestionCardValidation({
  rows = DEFAULT_QUESTION_CARD_VALIDATION_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "ddn_lint",
} = {}) {
  const validationRows = normalizeRows(rows);
  const validationArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(validationArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = validationRows.filter((row) => row.ready).length;
  const readyArtifactCount = validationArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = [
    "ai_generated_ddn_lint",
    "ai_generated_lesson_lint",
    "intent_alignment_check",
    "patch_safety_check",
    "ai_output_validation_receipt",
  ].every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === validationRows.length &&
    readyArtifactCount === validationArtifacts.length &&
    hasAllArtifacts;
  const active = validationRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : validationRows[0]?.id ?? "";
  return {
    __종류: "question_card_validation",
    schema: "ddn.geo.question_card_validation.v1",
    work_item: "GEO2_AI_OUTPUT_VALIDATION_PACK_V1",
    primary_coordinate: "거-2",
    depends_on_coordinate: ["거-1", "타-2", "자-1/자-2"],
    pack: "question_card_validation_v1",
    status: ready ? "question_card_validation_ready" : "question_card_validation_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    ai_output_validation_claim: ready,
    ddn_lint_claim: artifactKinds.has("ai_generated_ddn_lint"),
    lesson_lint_claim: artifactKinds.has("ai_generated_lesson_lint"),
    intent_alignment_claim: artifactKinds.has("intent_alignment_check"),
    patch_safety_claim: artifactKinds.has("patch_safety_check"),
    validation_receipt_claim: artifactKinds.has("ai_output_validation_receipt"),
    ai_call_claim: false,
    parser_preprocessor_claim: false,
    auto_apply_claim: false,
    patch_execution_claim: false,
    file_write_claim: false,
    network_call_claim: false,
    runtime_ast_persisted: false,
    state_hash_owner: false,
    model_training_claim: false,
    account_permission_change_claim: false,
    grammar_claim: false,
    rows: validationRows,
    artifacts: validationArtifacts,
    active_row_id: active,
    validation_text: buildValidationText(validationRows, validationArtifacts),
    artifact_size_bytes: validationArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 34,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 38,
      roadmap_v2_pack_evidence_reference_closed: 55,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 61,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "GEO3_DEV_ASSIST_UI_V1",
  };
}

export function formatQuestionCardValidationText(validation = {}) {
  const payload = asObject(validation);
  if (payload.schema !== "ddn.geo.question_card_validation.v1") {
    throw new Error("question_card_expected_validation");
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
    `ai_output_validation_claim\t${payload.ai_output_validation_claim === true ? "true" : "false"}`,
    `ddn_lint_claim\t${payload.ddn_lint_claim === true ? "true" : "false"}`,
    `lesson_lint_claim\t${payload.lesson_lint_claim === true ? "true" : "false"}`,
    `intent_alignment_claim\t${payload.intent_alignment_claim === true ? "true" : "false"}`,
    `patch_safety_claim\t${payload.patch_safety_claim === true ? "true" : "false"}`,
    `validation_receipt_claim\t${payload.validation_receipt_claim === true ? "true" : "false"}`,
    `ai_call_claim\t${payload.ai_call_claim === true ? "true" : "false"}`,
    `auto_apply_claim\t${payload.auto_apply_claim === true ? "true" : "false"}`,
    `patch_execution_claim\t${payload.patch_execution_claim === true ? "true" : "false"}`,
    `file_write_claim\t${payload.file_write_claim === true ? "true" : "false"}`,
    `network_call_claim\t${payload.network_call_claim === true ? "true" : "false"}`,
    `state_hash_owner\t${payload.state_hash_owner === true ? "true" : "false"}`,
    "",
    "row_id\tvalidation_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.validation_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderQuestionCardValidation(root, validation = {}) {
  if (!root) return null;
  const payload = asObject(validation);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.questionCardValidationStatus = asText(payload.status, "question_card_validation_incomplete");
  root.innerHTML = `
    <div class="question-validation-head">
      <div>
        <div class="question-validation-kicker">Seulgi Validation</div>
        <h2>AI output validation</h2>
      </div>
      <div class="question-validation-progress" data-question-validation-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="question-validation-summary" data-question-validation-summary>
      AI-generated DDN lint, lesson lint, intent alignment, patch safety, validation receipt를 로컬 validation pack으로 고정합니다. AI call, patch execution, file write, network call은 이번 범위가 아닙니다.
    </div>
    <div class="question-validation-body">
      <div class="question-validation-list">
        ${rows.map((row) => `
          <button type="button" class="question-validation-btn${row.id === activeId ? " active" : ""}" data-question-validation-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.validation_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="question-validation-detail">
        <div class="question-validation-title" data-question-validation-active-title>${escapeHtml(active.title)}</div>
        <p data-question-validation-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="question-validation-artifacts">
          ${artifacts.map((artifact) => `
            <span data-question-validation-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="question-validation-preview" data-question-validation-preview>${escapeHtml(payload.validation_text ?? "")}</pre>
        <button type="button" class="ghost" data-question-validation-copy>AI output validation 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderQuestionCardValidation(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-question-validation-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-question-validation-row") || ""));
  });
  root.querySelector("[data-question-validation-copy]")?.addEventListener("click", async () => {
    root.dataset.questionCardValidationCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatQuestionCardValidationText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
