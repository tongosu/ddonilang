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

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

const PROPOSAL_ROWS = [
  { id: "proposal", title: "슬기제안", kind: "advisor_proposal", decision: "review", uri: "seulgi://proposal/local-draft" },
  { id: "preview", title: "미리보기", kind: "non_applying_preview", decision: "review", uri: "seulgi://proposal/preview" },
  { id: "approve", title: "승인", kind: "approval_receipt", decision: "approved", uri: "seulgi://proposal/approval-receipt" },
  { id: "reject", title: "거절", kind: "rejection_receipt", decision: "rejected", uri: "seulgi://proposal/rejection-receipt" },
  { id: "audit", title: "감사 기록", kind: "append_only_audit_note", decision: "sealed", uri: "seulgi://proposal/audit-note" },
];

export const DEFAULT_SEULGI_PROPOSAL_UI_ROWS = PROPOSAL_ROWS.map((row) => ({
  id: row.id,
  kind: row.kind,
  decision: row.decision,
  uri: row.uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_SEULGI_PROPOSAL_UI_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return PROPOSAL_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      kind: asText(source.kind, row.kind),
      decision: asText(source.decision, row.decision),
      uri: asText(source.uri, row.uri),
      ready: source.ready !== false,
    };
  });
}

function buildProposalText(rows) {
  return [
    "seulgi_proposal_ui:seulgi_proposal_ui_v1",
    "coordinate:자-3",
    "ai_call:false",
    "auto_apply:false",
    "file_write:false",
    "runtime_ast_persisted:false",
    "state_hash_owner:false",
    "model_training:false",
    ...rows.map((row) => `${row.id}\t${row.kind}\t${row.decision}\t${row.ready ? "ready" : "missing"}`),
  ].join("\n");
}

export function buildSeulgiProposalUi({
  rows = DEFAULT_SEULGI_PROPOSAL_UI_ROWS,
  activeRowId = "proposal",
} = {}) {
  const proposalRows = normalizeRows(rows);
  const readyCount = proposalRows.filter((row) => row.ready).length;
  const ready = readyCount === proposalRows.length &&
    proposalRows.some((row) => row.decision === "approved") &&
    proposalRows.some((row) => row.decision === "rejected");
  const active = proposalRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : proposalRows[0]?.id ?? "";
  return {
    __kind: "seulgi_proposal_ui",
    schema: "ddn.ja.seulgi_proposal_ui.v1",
    work_item: "JA3_SEULGI_PROPOSAL_UI_V1",
    primary_coordinate: "자-3",
    depends_on_coordinate: ["자-0", "자-1", "자-2"],
    pack: "seulgi_proposal_ui_v1",
    status: ready ? "seulgi_proposal_ui_ready" : "seulgi_proposal_ui_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    proposal_ui_claim: ready,
    proposal_draft_claim: proposalRows.some((row) => row.id === "proposal" && row.ready),
    preview_claim: proposalRows.some((row) => row.id === "preview" && row.ready),
    approve_claim: proposalRows.some((row) => row.id === "approve" && row.ready),
    reject_claim: proposalRows.some((row) => row.id === "reject" && row.ready),
    audit_claim: proposalRows.some((row) => row.id === "audit" && row.ready),
    ai_call_claim: false,
    auto_apply_claim: false,
    file_write_claim: false,
    runtime_ast_persisted: false,
    state_hash_owner: false,
    model_training_claim: false,
    rows: proposalRows,
    active_row_id: active,
    proposal_text: buildProposalText(proposalRows),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 74,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 82,
      roadmap_v2_docs_closed: 5,
      roadmap_v2_docs_total: 90,
      roadmap_v2_docs_percent: 6,
      roadmap_v2_pack_evidence_reference_closed: 76,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 84,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "ROADMAP_V2_JA5_REPLAY_SAFE_AI_WORKFLOW_V1",
  };
}

export function formatSeulgiProposalUiText(proposal = {}) {
  const payload = asObject(proposal);
  if (payload.schema !== "ddn.ja.seulgi_proposal_ui.v1") {
    throw new Error("seulgi_proposal_ui_expected");
  }
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
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
    `docs_closed\t${progress.roadmap_v2_docs_closed ?? 0}/${progress.roadmap_v2_docs_total ?? 0}`,
    `pack_evidence_reference\t${progress.roadmap_v2_pack_evidence_reference_closed ?? 0}/${progress.roadmap_v2_pack_evidence_reference_total ?? 0}`,
    `studio_local_super_long\t${progress.studio_local_super_long_closed ?? 0}/${progress.studio_local_super_long_total ?? 0}`,
    `proposal_ui_claim\t${payload.proposal_ui_claim === true ? "true" : "false"}`,
    `ai_call_claim\t${payload.ai_call_claim === true ? "true" : "false"}`,
    `auto_apply_claim\t${payload.auto_apply_claim === true ? "true" : "false"}`,
    `file_write_claim\t${payload.file_write_claim === true ? "true" : "false"}`,
    `runtime_ast_persisted\t${payload.runtime_ast_persisted === true ? "true" : "false"}`,
    `state_hash_owner\t${payload.state_hash_owner === true ? "true" : "false"}`,
    "",
    "row_id\tkind\tdecision\tready",
    ...rows.map((row) => `${row.id}\t${row.kind}\t${row.decision}\t${row.ready === true ? "true" : "false"}`),
  ].join("\n");
}

export function renderSeulgiProposalUi(root, proposal = {}) {
  if (!root) return null;
  const payload = asObject(proposal);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.seulgiProposalUiStatus = asText(payload.status, "seulgi_proposal_ui_incomplete");
  root.innerHTML = `
    <div class="question-card-head">
      <div>
        <div class="question-card-kicker">Seulgi Proposal UI</div>
        <h2>AI 제안 UI</h2>
      </div>
      <div class="question-card-progress" data-seulgi-proposal-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="question-card-summary" data-seulgi-proposal-summary>
      슬기제안, 미리보기, 승인, 거절, 감사 기록을 로컬 UI로 묶습니다. AI call, auto-apply, file write, runtime AST persistence는 이번 범위가 아닙니다.
    </div>
    <div class="question-card-body">
      <div class="question-card-list">
        ${rows.map((row) => `
          <button type="button" class="question-card-btn${row.id === activeId ? " active" : ""}" data-seulgi-proposal-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.kind)} · ${escapeHtml(row.decision)}</small>
          </button>
        `).join("")}
      </div>
      <div class="question-card-detail">
        <div class="question-card-title" data-seulgi-proposal-active-title>${escapeHtml(active.title)}</div>
        <p data-seulgi-proposal-active-link>${escapeHtml(active.uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>decision</dt><dd>${escapeHtml(active.decision ?? "")}</dd></div>
        </dl>
        <div class="question-card-artifacts">
          ${rows.map((row) => `
            <span data-seulgi-proposal-artifact="${escapeHtml(row.id)}">${escapeHtml(row.kind)} · ${escapeHtml(row.decision)}</span>
          `).join("")}
        </div>
        <pre class="question-card-preview" data-seulgi-proposal-preview>${escapeHtml(payload.proposal_text ?? "")}</pre>
        <button type="button" class="ghost" data-seulgi-proposal-copy>AI 제안 UI 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderSeulgiProposalUi(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-seulgi-proposal-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-seulgi-proposal-row") || ""));
  });
  root.querySelector("[data-seulgi-proposal-copy]")?.addEventListener("click", async () => {
    root.dataset.seulgiProposalCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatSeulgiProposalUiText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
