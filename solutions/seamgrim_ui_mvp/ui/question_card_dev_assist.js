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

const DEV_ASSIST_ROWS = [
  { id: "codex_work_item", title: "Codex work item", assist_kind: "codex_work_item_draft", local_uri: "question-card://dev-assist/codex-work-item" },
  { id: "lesson_draft", title: "Lesson draft", assist_kind: "lesson_draft_preview", local_uri: "question-card://dev-assist/lesson-draft" },
  { id: "report_draft", title: "Report draft", assist_kind: "report_draft_preview", local_uri: "question-card://dev-assist/report-draft" },
  { id: "review_queue", title: "Review queue", assist_kind: "review_queue_packet", local_uri: "question-card://dev-assist/review-queue" },
  { id: "handoff_receipt", title: "Handoff receipt", assist_kind: "dev_assist_handoff_receipt", local_uri: "question-card://dev-assist/handoff-receipt" },
];

const DEFAULT_ARTIFACTS = [
  { name: "question_card.dev_assist.codex_work_item.detjson", kind: "codex_work_item_draft", bytes: 792 },
  { name: "question_card.dev_assist.lesson_draft.detjson", kind: "lesson_draft_preview", bytes: 818 },
  { name: "question_card.dev_assist.report_draft.detjson", kind: "report_draft_preview", bytes: 806 },
  { name: "question_card.dev_assist.review_queue.detjson", kind: "review_queue_packet", bytes: 844 },
  { name: "question_card.dev_assist.handoff_receipt.detjson", kind: "dev_assist_handoff_receipt", bytes: 734 },
];

export const DEFAULT_QUESTION_CARD_DEV_ASSIST_ROWS = DEV_ASSIST_ROWS.map((row) => ({
  id: row.id,
  assist_kind: row.assist_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_QUESTION_CARD_DEV_ASSIST_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return DEV_ASSIST_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      assist_kind: asText(source.assist_kind, row.assist_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      dev_assist_ui_claim: true,
      ai_call_claim: false,
      file_write_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `question_card_dev_assist_${index + 1}.detjson`),
      kind: asText(source.kind, "question_card_dev_assist_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildDevAssistText(rows, artifacts) {
  return [
    "question_card_dev_assist:question_card_dev_assist_v1",
    "coordinate:거-3",
    "ai_call:false",
    "parser_preprocessor:false",
    "auto_apply:false",
    "patch_execution:false",
    "file_write:false",
    "network_call:false",
    "runtime_ast_persisted:false",
    "state_hash_owner:false",
    "registry_publish:false",
    "account_permission_change:false",
    ...rows.map((row) => `${row.id}\t${row.assist_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildQuestionCardDevAssist({
  rows = DEFAULT_QUESTION_CARD_DEV_ASSIST_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "codex_work_item",
} = {}) {
  const assistRows = normalizeRows(rows);
  const assistArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(assistArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = assistRows.filter((row) => row.ready).length;
  const readyArtifactCount = assistArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = [
    "codex_work_item_draft",
    "lesson_draft_preview",
    "report_draft_preview",
    "review_queue_packet",
    "dev_assist_handoff_receipt",
  ].every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === assistRows.length &&
    readyArtifactCount === assistArtifacts.length &&
    hasAllArtifacts;
  const active = assistRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : assistRows[0]?.id ?? "";
  return {
    __종류: "question_card_dev_assist",
    schema: "ddn.geo.question_card_dev_assist.v1",
    work_item: "GEO3_DEV_ASSIST_UI_V1",
    primary_coordinate: "거-3",
    depends_on_coordinate: ["거-2", "거-1", "타-2"],
    pack: "question_card_dev_assist_v1",
    status: ready ? "question_card_dev_assist_ready" : "question_card_dev_assist_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    dev_assist_ui_claim: ready,
    codex_work_item_claim: artifactKinds.has("codex_work_item_draft"),
    lesson_draft_claim: artifactKinds.has("lesson_draft_preview"),
    report_draft_claim: artifactKinds.has("report_draft_preview"),
    review_queue_claim: artifactKinds.has("review_queue_packet"),
    handoff_receipt_claim: artifactKinds.has("dev_assist_handoff_receipt"),
    ai_call_claim: false,
    parser_preprocessor_claim: false,
    auto_apply_claim: false,
    patch_execution_claim: false,
    file_write_claim: false,
    network_call_claim: false,
    runtime_ast_persisted: false,
    state_hash_owner: false,
    registry_publish_claim: false,
    account_permission_change_claim: false,
    grammar_claim: false,
    rows: assistRows,
    artifacts: assistArtifacts,
    active_row_id: active,
    assist_text: buildDevAssistText(assistRows, assistArtifacts),
    artifact_size_bytes: assistArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 35,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 39,
      roadmap_v2_pack_evidence_reference_closed: 56,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 62,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "GEO4_AUTHOR_TOOL_SHARE_V1",
  };
}

export function formatQuestionCardDevAssistText(assist = {}) {
  const payload = asObject(assist);
  if (payload.schema !== "ddn.geo.question_card_dev_assist.v1") {
    throw new Error("question_card_expected_dev_assist");
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
    `dev_assist_ui_claim\t${payload.dev_assist_ui_claim === true ? "true" : "false"}`,
    `codex_work_item_claim\t${payload.codex_work_item_claim === true ? "true" : "false"}`,
    `lesson_draft_claim\t${payload.lesson_draft_claim === true ? "true" : "false"}`,
    `report_draft_claim\t${payload.report_draft_claim === true ? "true" : "false"}`,
    `review_queue_claim\t${payload.review_queue_claim === true ? "true" : "false"}`,
    `handoff_receipt_claim\t${payload.handoff_receipt_claim === true ? "true" : "false"}`,
    `ai_call_claim\t${payload.ai_call_claim === true ? "true" : "false"}`,
    `file_write_claim\t${payload.file_write_claim === true ? "true" : "false"}`,
    `network_call_claim\t${payload.network_call_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `state_hash_owner\t${payload.state_hash_owner === true ? "true" : "false"}`,
    "",
    "row_id\tassist_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.assist_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderQuestionCardDevAssist(root, assist = {}) {
  if (!root) return null;
  const payload = asObject(assist);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.questionCardDevAssistStatus = asText(payload.status, "question_card_dev_assist_incomplete");
  root.innerHTML = `
    <div class="question-assist-head">
      <div>
        <div class="question-assist-kicker">Seulgi Dev Assist</div>
        <h2>Development assist UI</h2>
      </div>
      <div class="question-assist-progress" data-question-assist-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="question-assist-summary" data-question-assist-summary>
      Codex work item, lesson draft, report draft, review queue, handoff receipt를 로컬 개발보조 UI로 고정합니다. AI call, file write, network call, registry publish는 이번 범위가 아닙니다.
    </div>
    <div class="question-assist-body">
      <div class="question-assist-list">
        ${rows.map((row) => `
          <button type="button" class="question-assist-btn${row.id === activeId ? " active" : ""}" data-question-assist-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.assist_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="question-assist-detail">
        <div class="question-assist-title" data-question-assist-active-title>${escapeHtml(active.title)}</div>
        <p data-question-assist-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="question-assist-artifacts">
          ${artifacts.map((artifact) => `
            <span data-question-assist-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="question-assist-preview" data-question-assist-preview>${escapeHtml(payload.assist_text ?? "")}</pre>
        <button type="button" class="ghost" data-question-assist-copy>Development assist 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderQuestionCardDevAssist(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-question-assist-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-question-assist-row") || ""));
  });
  root.querySelector("[data-question-assist-copy]")?.addEventListener("click", async () => {
    root.dataset.questionCardDevAssistCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatQuestionCardDevAssistText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
