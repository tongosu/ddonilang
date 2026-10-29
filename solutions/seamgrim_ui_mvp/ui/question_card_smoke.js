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

const QUESTION_CARD_ROWS = [
  { id: "proposal_draft", title: "Proposal draft", artifact_kind: "local_proposal_draft", local_uri: "question-card://smoke/proposal-draft" },
  { id: "patch_preview", title: "Patch preview", artifact_kind: "non_applying_patch_preview", local_uri: "question-card://smoke/patch-preview" },
  { id: "fix_it_hint", title: "Fix-it hint", artifact_kind: "review_only_fix_it_hint", local_uri: "question-card://smoke/fix-it-hint" },
  { id: "review_packet", title: "Review packet", artifact_kind: "question_card_review_packet", local_uri: "question-card://smoke/review-packet" },
  { id: "boundary_receipt", title: "Boundary receipt", artifact_kind: "no_auto_apply_boundary_receipt", local_uri: "question-card://smoke/boundary-receipt" },
];

const DEFAULT_ARTIFACTS = [
  { name: "question_card.proposal_draft.detjson", kind: "local_proposal_draft", bytes: 704 },
  { name: "question_card.patch_preview.detjson", kind: "non_applying_patch_preview", bytes: 812 },
  { name: "question_card.fix_it_hint.detjson", kind: "review_only_fix_it_hint", bytes: 668 },
  { name: "question_card.review_packet.detjson", kind: "question_card_review_packet", bytes: 886 },
  { name: "question_card.boundary_receipt.detjson", kind: "no_auto_apply_boundary_receipt", bytes: 734 },
];

export const DEFAULT_QUESTION_CARD_SMOKE_ROWS = QUESTION_CARD_ROWS.map((row) => ({
  id: row.id,
  artifact_kind: row.artifact_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_QUESTION_CARD_SMOKE_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return QUESTION_CARD_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      artifact_kind: asText(source.artifact_kind, row.artifact_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      question_card_smoke_claim: true,
      ai_call_claim: false,
      auto_apply_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `question_card_smoke_${index + 1}.detjson`),
      kind: asText(source.kind, "question_card_smoke_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildQuestionCardSmokeText(rows, artifacts) {
  return [
    "question_card_smoke:question_card_smoke_v1",
    "coordinate:거-1",
    "ai_call:false",
    "parser_preprocessor:false",
    "auto_apply:false",
    "file_write:false",
    "runtime_ast_persisted:false",
    "state_hash_owner:false",
    "replay_ai_recall:false",
    "model_training:false",
    "account_permission_change:false",
    ...rows.map((row) => `${row.id}\t${row.artifact_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildQuestionCardSmoke({
  rows = DEFAULT_QUESTION_CARD_SMOKE_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "proposal_draft",
} = {}) {
  const smokeRows = normalizeRows(rows);
  const smokeArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(smokeArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = smokeRows.filter((row) => row.ready).length;
  const readyArtifactCount = smokeArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = [
    "local_proposal_draft",
    "non_applying_patch_preview",
    "review_only_fix_it_hint",
    "question_card_review_packet",
    "no_auto_apply_boundary_receipt",
  ].every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === smokeRows.length &&
    readyArtifactCount === smokeArtifacts.length &&
    hasAllArtifacts;
  const active = smokeRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : smokeRows[0]?.id ?? "";
  return {
    __종류: "question_card_smoke",
    schema: "ddn.geo.question_card_smoke.v1",
    work_item: "GEO1_QUESTION_CARD_SMOKE_V1",
    primary_coordinate: "거-1",
    depends_on_coordinate: ["거-0", "타-1", "자-0"],
    pack: "question_card_smoke_v1",
    status: ready ? "question_card_smoke_ready" : "question_card_smoke_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    question_card_smoke_claim: ready,
    proposal_draft_claim: artifactKinds.has("local_proposal_draft"),
    patch_preview_claim: artifactKinds.has("non_applying_patch_preview"),
    fix_it_hint_claim: artifactKinds.has("review_only_fix_it_hint"),
    review_packet_claim: artifactKinds.has("question_card_review_packet"),
    boundary_receipt_claim: artifactKinds.has("no_auto_apply_boundary_receipt"),
    ai_call_claim: false,
    parser_preprocessor_claim: false,
    auto_apply_claim: false,
    file_write_claim: false,
    runtime_ast_persisted: false,
    state_hash_owner: false,
    replay_ai_recall_claim: false,
    model_training_claim: false,
    account_permission_change_claim: false,
    grammar_claim: false,
    rows: smokeRows,
    artifacts: smokeArtifacts,
    active_row_id: active,
    smoke_text: buildQuestionCardSmokeText(smokeRows, smokeArtifacts),
    artifact_size_bytes: smokeArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 33,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 37,
      roadmap_v2_pack_evidence_reference_closed: 54,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 60,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "GEO2_AI_OUTPUT_VALIDATION_PACK_V1",
  };
}

export function formatQuestionCardSmokeText(smoke = {}) {
  const payload = asObject(smoke);
  if (payload.schema !== "ddn.geo.question_card_smoke.v1") {
    throw new Error("question_card_expected_smoke");
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
    `question_card_smoke_claim\t${payload.question_card_smoke_claim === true ? "true" : "false"}`,
    `proposal_draft_claim\t${payload.proposal_draft_claim === true ? "true" : "false"}`,
    `patch_preview_claim\t${payload.patch_preview_claim === true ? "true" : "false"}`,
    `fix_it_hint_claim\t${payload.fix_it_hint_claim === true ? "true" : "false"}`,
    `review_packet_claim\t${payload.review_packet_claim === true ? "true" : "false"}`,
    `boundary_receipt_claim\t${payload.boundary_receipt_claim === true ? "true" : "false"}`,
    `ai_call_claim\t${payload.ai_call_claim === true ? "true" : "false"}`,
    `parser_preprocessor_claim\t${payload.parser_preprocessor_claim === true ? "true" : "false"}`,
    `auto_apply_claim\t${payload.auto_apply_claim === true ? "true" : "false"}`,
    `file_write_claim\t${payload.file_write_claim === true ? "true" : "false"}`,
    `runtime_ast_persisted\t${payload.runtime_ast_persisted === true ? "true" : "false"}`,
    `state_hash_owner\t${payload.state_hash_owner === true ? "true" : "false"}`,
    "",
    "row_id\tartifact_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.artifact_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderQuestionCardSmoke(root, smoke = {}) {
  if (!root) return null;
  const payload = asObject(smoke);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.questionCardSmokeStatus = asText(payload.status, "question_card_smoke_incomplete");
  root.innerHTML = `
    <div class="question-card-head">
      <div>
        <div class="question-card-kicker">Seulgi Question Card</div>
        <h2>Question card smoke</h2>
      </div>
      <div class="question-card-progress" data-question-card-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="question-card-summary" data-question-card-summary>
      Proposal draft, patch preview, fix-it hint, review packet, boundary receipt를 로컬 smoke UI로 고정합니다. AI call, auto-apply, file write, runtime AST persistence는 이번 범위가 아닙니다.
    </div>
    <div class="question-card-body">
      <div class="question-card-list">
        ${rows.map((row) => `
          <button type="button" class="question-card-btn${row.id === activeId ? " active" : ""}" data-question-card-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.artifact_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="question-card-detail">
        <div class="question-card-title" data-question-card-active-title>${escapeHtml(active.title)}</div>
        <p data-question-card-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="question-card-artifacts">
          ${artifacts.map((artifact) => `
            <span data-question-card-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="question-card-preview" data-question-card-preview>${escapeHtml(payload.smoke_text ?? "")}</pre>
        <button type="button" class="ghost" data-question-card-copy>Question card smoke 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderQuestionCardSmoke(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-question-card-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-question-card-row") || ""));
  });
  root.querySelector("[data-question-card-copy]")?.addEventListener("click", async () => {
    root.dataset.questionCardSmokeCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatQuestionCardSmokeText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
