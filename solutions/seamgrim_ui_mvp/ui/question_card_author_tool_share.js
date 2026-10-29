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

const SHARE_ROWS = [
  { id: "template_registry", title: "Template registry", share_kind: "question_card_template_registry", local_uri: "question-card://author-share/template-registry" },
  { id: "tool_manifest", title: "Tool manifest", share_kind: "author_tool_manifest", local_uri: "question-card://author-share/tool-manifest" },
  { id: "lesson_template", title: "Lesson template", share_kind: "lesson_author_template", local_uri: "question-card://author-share/lesson-template" },
  { id: "review_template", title: "Review template", share_kind: "review_queue_template", local_uri: "question-card://author-share/review-template" },
  { id: "handoff_bundle", title: "Handoff bundle", share_kind: "author_share_handoff_bundle", local_uri: "question-card://author-share/handoff-bundle" },
];

const DEFAULT_ARTIFACTS = [
  { name: "question_card.author_share.template_registry.detjson", kind: "question_card_template_registry", bytes: 902 },
  { name: "question_card.author_share.tool_manifest.detjson", kind: "author_tool_manifest", bytes: 774 },
  { name: "question_card.author_share.lesson_template.detjson", kind: "lesson_author_template", bytes: 838 },
  { name: "question_card.author_share.review_template.detjson", kind: "review_queue_template", bytes: 812 },
  { name: "question_card.author_share.handoff_bundle.detjson", kind: "author_share_handoff_bundle", bytes: 798 },
];

export const DEFAULT_QUESTION_CARD_AUTHOR_TOOL_SHARE_ROWS = SHARE_ROWS.map((row) => ({
  id: row.id,
  share_kind: row.share_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_QUESTION_CARD_AUTHOR_TOOL_SHARE_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return SHARE_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      share_kind: asText(source.share_kind, row.share_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      author_tool_share_claim: true,
      registry_publish_claim: false,
      account_permission_change_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `question_card_author_tool_share_${index + 1}.detjson`),
      kind: asText(source.kind, "author_tool_share_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildShareText(rows, artifacts) {
  return [
    "question_card_author_tool_share:question_card_author_tool_share_v1",
    "coordinate:거-4",
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
    "cloud_sync:false",
    ...rows.map((row) => `${row.id}\t${row.share_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildQuestionCardAuthorToolShare({
  rows = DEFAULT_QUESTION_CARD_AUTHOR_TOOL_SHARE_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "template_registry",
} = {}) {
  const shareRows = normalizeRows(rows);
  const shareArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(shareArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = shareRows.filter((row) => row.ready).length;
  const readyArtifactCount = shareArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = [
    "question_card_template_registry",
    "author_tool_manifest",
    "lesson_author_template",
    "review_queue_template",
    "author_share_handoff_bundle",
  ].every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === shareRows.length &&
    readyArtifactCount === shareArtifacts.length &&
    hasAllArtifacts;
  const active = shareRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : shareRows[0]?.id ?? "";
  return {
    __종류: "question_card_author_tool_share",
    schema: "ddn.geo.question_card_author_tool_share.v1",
    work_item: "GEO4_AUTHOR_TOOL_SHARE_V1",
    primary_coordinate: "거-4",
    depends_on_coordinate: ["거-3", "거-2", "거-1", "타-2"],
    pack: "question_card_author_tool_share_v1",
    status: ready ? "question_card_author_tool_share_ready" : "question_card_author_tool_share_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    author_tool_share_claim: ready,
    template_registry_claim: artifactKinds.has("question_card_template_registry"),
    tool_manifest_claim: artifactKinds.has("author_tool_manifest"),
    lesson_template_claim: artifactKinds.has("lesson_author_template"),
    review_template_claim: artifactKinds.has("review_queue_template"),
    handoff_bundle_claim: artifactKinds.has("author_share_handoff_bundle"),
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
    cloud_sync_claim: false,
    grammar_claim: false,
    rows: shareRows,
    artifacts: shareArtifacts,
    active_row_id: active,
    share_text: buildShareText(shareRows, shareArtifacts),
    artifact_size_bytes: shareArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 36,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 40,
      roadmap_v2_pack_evidence_reference_closed: 57,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 63,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "GEO5_AI_WORKFLOW_HARDENING_V1",
  };
}

export function formatQuestionCardAuthorToolShareText(share = {}) {
  const payload = asObject(share);
  if (payload.schema !== "ddn.geo.question_card_author_tool_share.v1") {
    throw new Error("question_card_expected_author_tool_share");
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
    `author_tool_share_claim\t${payload.author_tool_share_claim === true ? "true" : "false"}`,
    `template_registry_claim\t${payload.template_registry_claim === true ? "true" : "false"}`,
    `tool_manifest_claim\t${payload.tool_manifest_claim === true ? "true" : "false"}`,
    `lesson_template_claim\t${payload.lesson_template_claim === true ? "true" : "false"}`,
    `review_template_claim\t${payload.review_template_claim === true ? "true" : "false"}`,
    `handoff_bundle_claim\t${payload.handoff_bundle_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `account_permission_change_claim\t${payload.account_permission_change_claim === true ? "true" : "false"}`,
    `cloud_sync_claim\t${payload.cloud_sync_claim === true ? "true" : "false"}`,
    "",
    "row_id\tshare_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.share_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderQuestionCardAuthorToolShare(root, share = {}) {
  if (!root) return null;
  const payload = asObject(share);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.questionCardAuthorToolShareStatus = asText(payload.status, "question_card_author_tool_share_incomplete");
  root.innerHTML = `
    <div class="question-share-head">
      <div>
        <div class="question-share-kicker">Seulgi Author Share</div>
        <h2>Author tool share</h2>
      </div>
      <div class="question-share-progress" data-question-share-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="question-share-summary" data-question-share-summary>
      Question card template registry, tool manifest, lesson template, review template, handoff bundle을 로컬 공유 UI로 고정합니다. Registry publish, account permission change, cloud sync는 이번 범위가 아닙니다.
    </div>
    <div class="question-share-body">
      <div class="question-share-list">
        ${rows.map((row) => `
          <button type="button" class="question-share-btn${row.id === activeId ? " active" : ""}" data-question-share-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.share_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="question-share-detail">
        <div class="question-share-title" data-question-share-active-title>${escapeHtml(active.title)}</div>
        <p data-question-share-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="question-share-artifacts">
          ${artifacts.map((artifact) => `
            <span data-question-share-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="question-share-preview" data-question-share-preview>${escapeHtml(payload.share_text ?? "")}</pre>
        <button type="button" class="ghost" data-question-share-copy>Author share 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderQuestionCardAuthorToolShare(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-question-share-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-question-share-row") || ""));
  });
  root.querySelector("[data-question-share-copy]")?.addEventListener("click", async () => {
    root.dataset.questionCardAuthorToolShareCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatQuestionCardAuthorToolShareText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
