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

const HARDENING_ROWS = [
  { id: "approval_gate", title: "Approval gate", hardening_kind: "manual_approval_gate", local_uri: "question-card://workflow-hardening/approval-gate" },
  { id: "replay_packet", title: "Replay packet", hardening_kind: "workflow_replay_packet", local_uri: "question-card://workflow-hardening/replay-packet" },
  { id: "audit_trail", title: "Audit trail", hardening_kind: "append_only_audit_preview", local_uri: "question-card://workflow-hardening/audit-trail" },
  { id: "rollback_plan", title: "Rollback plan", hardening_kind: "local_rollback_plan", local_uri: "question-card://workflow-hardening/rollback-plan" },
  { id: "lts_gate", title: "LTS gate", hardening_kind: "ai_workflow_lts_gate", local_uri: "question-card://workflow-hardening/lts-gate" },
];

const DEFAULT_ARTIFACTS = [
  { name: "question_card.workflow.approval_gate.detjson", kind: "manual_approval_gate", bytes: 884 },
  { name: "question_card.workflow.replay_packet.detjson", kind: "workflow_replay_packet", bytes: 926 },
  { name: "question_card.workflow.audit_trail.detjson", kind: "append_only_audit_preview", bytes: 876 },
  { name: "question_card.workflow.rollback_plan.detjson", kind: "local_rollback_plan", bytes: 802 },
  { name: "question_card.workflow.lts_gate.detjson", kind: "ai_workflow_lts_gate", bytes: 858 },
];

export const DEFAULT_QUESTION_CARD_WORKFLOW_HARDENING_ROWS = HARDENING_ROWS.map((row) => ({
  id: row.id,
  hardening_kind: row.hardening_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_QUESTION_CARD_WORKFLOW_HARDENING_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return HARDENING_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      hardening_kind: asText(source.hardening_kind, row.hardening_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      workflow_hardening_claim: true,
      auto_apply_claim: false,
      file_write_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `question_card_workflow_hardening_${index + 1}.detjson`),
      kind: asText(source.kind, "workflow_hardening_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildHardeningText(rows, artifacts) {
  return [
    "question_card_workflow_hardening:question_card_workflow_hardening_v1",
    "coordinate:거-5",
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
    "release_execution:false",
    "lts_certification:false",
    ...rows.map((row) => `${row.id}\t${row.hardening_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildQuestionCardWorkflowHardening({
  rows = DEFAULT_QUESTION_CARD_WORKFLOW_HARDENING_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "approval_gate",
} = {}) {
  const hardeningRows = normalizeRows(rows);
  const hardeningArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(hardeningArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = hardeningRows.filter((row) => row.ready).length;
  const readyArtifactCount = hardeningArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = [
    "manual_approval_gate",
    "workflow_replay_packet",
    "append_only_audit_preview",
    "local_rollback_plan",
    "ai_workflow_lts_gate",
  ].every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === hardeningRows.length &&
    readyArtifactCount === hardeningArtifacts.length &&
    hasAllArtifacts;
  const active = hardeningRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : hardeningRows[0]?.id ?? "";
  return {
    __종류: "question_card_workflow_hardening",
    schema: "ddn.geo.question_card_workflow_hardening.v1",
    work_item: "GEO5_AI_WORKFLOW_HARDENING_V1",
    primary_coordinate: "거-5",
    depends_on_coordinate: ["거-4", "거-3", "거-2", "거-1", "타-2"],
    pack: "question_card_workflow_hardening_v1",
    status: ready ? "question_card_workflow_hardening_ready" : "question_card_workflow_hardening_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    workflow_hardening_claim: ready,
    approval_gate_claim: artifactKinds.has("manual_approval_gate"),
    replay_packet_claim: artifactKinds.has("workflow_replay_packet"),
    audit_trail_claim: artifactKinds.has("append_only_audit_preview"),
    rollback_plan_claim: artifactKinds.has("local_rollback_plan"),
    lts_gate_claim: artifactKinds.has("ai_workflow_lts_gate"),
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
    release_execution_claim: false,
    lts_certification_claim: false,
    grammar_claim: false,
    rows: hardeningRows,
    artifacts: hardeningArtifacts,
    active_row_id: active,
    hardening_text: buildHardeningText(hardeningRows, hardeningArtifacts),
    artifact_size_bytes: hardeningArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 37,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 41,
      roadmap_v2_pack_evidence_reference_closed: 58,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 64,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "ROADMAP_V2_NEXT_FRONTIER_REBASE_V1",
  };
}

export function formatQuestionCardWorkflowHardeningText(hardening = {}) {
  const payload = asObject(hardening);
  if (payload.schema !== "ddn.geo.question_card_workflow_hardening.v1") {
    throw new Error("question_card_expected_workflow_hardening");
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
    `workflow_hardening_claim\t${payload.workflow_hardening_claim === true ? "true" : "false"}`,
    `approval_gate_claim\t${payload.approval_gate_claim === true ? "true" : "false"}`,
    `replay_packet_claim\t${payload.replay_packet_claim === true ? "true" : "false"}`,
    `audit_trail_claim\t${payload.audit_trail_claim === true ? "true" : "false"}`,
    `rollback_plan_claim\t${payload.rollback_plan_claim === true ? "true" : "false"}`,
    `lts_gate_claim\t${payload.lts_gate_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `lts_certification_claim\t${payload.lts_certification_claim === true ? "true" : "false"}`,
    "",
    "row_id\thardening_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.hardening_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderQuestionCardWorkflowHardening(root, hardening = {}) {
  if (!root) return null;
  const payload = asObject(hardening);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.questionCardWorkflowHardeningStatus = asText(payload.status, "question_card_workflow_hardening_incomplete");
  root.innerHTML = `
    <div class="question-hardening-head">
      <div>
        <div class="question-hardening-kicker">Seulgi Workflow Hardening</div>
        <h2>AI workflow hardening</h2>
      </div>
      <div class="question-hardening-progress" data-question-hardening-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="question-hardening-summary" data-question-hardening-summary>
      Manual approval gate, replay packet, audit trail, rollback plan, local LTS gate를 로컬 hardening UI로 고정합니다. Release execution, LTS certification, auto apply는 이번 범위가 아닙니다.
    </div>
    <div class="question-hardening-body">
      <div class="question-hardening-list">
        ${rows.map((row) => `
          <button type="button" class="question-hardening-btn${row.id === activeId ? " active" : ""}" data-question-hardening-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.hardening_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="question-hardening-detail">
        <div class="question-hardening-title" data-question-hardening-active-title>${escapeHtml(active.title)}</div>
        <p data-question-hardening-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="question-hardening-artifacts">
          ${artifacts.map((artifact) => `
            <span data-question-hardening-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="question-hardening-preview" data-question-hardening-preview>${escapeHtml(payload.hardening_text ?? "")}</pre>
        <button type="button" class="ghost" data-question-hardening-copy>Workflow hardening 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderQuestionCardWorkflowHardening(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-question-hardening-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-question-hardening-row") || ""));
  });
  root.querySelector("[data-question-hardening-copy]")?.addEventListener("click", async () => {
    root.dataset.questionCardWorkflowHardeningCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatQuestionCardWorkflowHardeningText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
