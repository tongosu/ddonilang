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

const WORKFLOW_ROWS = [
  { id: "snapshot", title: "입력 스냅샷", kind: "sealed_input_snapshot", replay: "source_fixed", uri: "seulgi://replay/input-snapshot" },
  { id: "approval", title: "승인 영수증", kind: "approval_receipt", replay: "decision_fixed", uri: "seulgi://replay/approval" },
  { id: "artifact", title: "아티팩트 봉인", kind: "artifact_hash_line", replay: "artifact_fixed", uri: "seulgi://replay/artifact" },
  { id: "no_recall", title: "AI 재호출 없음", kind: "no_ai_recall_check", replay: "no_recall", uri: "seulgi://replay/no-ai-recall" },
  { id: "lts_gate", title: "LTS 게이트", kind: "replay_lts_gate", replay: "gate_ready", uri: "seulgi://replay/lts-gate" },
];

export const DEFAULT_SEULGI_REPLAY_SAFE_ROWS = WORKFLOW_ROWS.map((row) => ({
  id: row.id,
  kind: row.kind,
  replay: row.replay,
  uri: row.uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_SEULGI_REPLAY_SAFE_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return WORKFLOW_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      kind: asText(source.kind, row.kind),
      replay: asText(source.replay, row.replay),
      uri: asText(source.uri, row.uri),
      ready: source.ready !== false,
    };
  });
}

function buildReplayText(rows) {
  return [
    "seulgi_replay_safe_workflow:seulgi_replay_safe_workflow_v1",
    "coordinate:자-5",
    "ai_recall:false",
    "auto_apply:false",
    "file_write:false",
    "runtime_ast_persisted:false",
    "state_hash_owner:false",
    "model_training:false",
    ...rows.map((row) => `${row.id}\t${row.kind}\t${row.replay}\t${row.ready ? "ready" : "missing"}`),
  ].join("\n");
}

export function buildSeulgiReplaySafeWorkflow({
  rows = DEFAULT_SEULGI_REPLAY_SAFE_ROWS,
  activeRowId = "snapshot",
} = {}) {
  const workflowRows = normalizeRows(rows);
  const readyCount = workflowRows.filter((row) => row.ready).length;
  const ready = readyCount === workflowRows.length &&
    workflowRows.some((row) => row.replay === "no_recall") &&
    workflowRows.some((row) => row.replay === "gate_ready");
  const active = workflowRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : workflowRows[0]?.id ?? "";
  return {
    __kind: "seulgi_replay_safe_workflow",
    schema: "ddn.ja.seulgi_replay_safe_workflow.v1",
    work_item: "JA5_REPLAY_SAFE_AI_WORKFLOW_V1",
    primary_coordinate: "자-5",
    depends_on_coordinate: ["자-2", "자-3", "자-4"],
    pack: "seulgi_replay_safe_workflow_v1",
    status: ready ? "seulgi_replay_safe_workflow_ready" : "seulgi_replay_safe_workflow_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    replay_safe_workflow_claim: ready,
    input_snapshot_claim: workflowRows.some((row) => row.id === "snapshot" && row.ready),
    approval_receipt_claim: workflowRows.some((row) => row.id === "approval" && row.ready),
    artifact_hash_claim: workflowRows.some((row) => row.id === "artifact" && row.ready),
    no_ai_recall_claim: workflowRows.some((row) => row.id === "no_recall" && row.ready),
    lts_gate_claim: workflowRows.some((row) => row.id === "lts_gate" && row.ready),
    ai_recall_claim: false,
    auto_apply_claim: false,
    file_write_claim: false,
    runtime_ast_persisted: false,
    state_hash_owner: false,
    model_training_claim: false,
    rows: workflowRows,
    active_row_id: active,
    replay_text: buildReplayText(workflowRows),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 75,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 83,
      roadmap_v2_docs_closed: 5,
      roadmap_v2_docs_total: 90,
      roadmap_v2_docs_percent: 6,
      roadmap_v2_pack_evidence_reference_closed: 77,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 86,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "ROADMAP_V2_A3_NURIGYM_PYTHON_WEB_PARITY_V1",
  };
}

export function formatSeulgiReplaySafeWorkflowText(workflow = {}) {
  const payload = asObject(workflow);
  if (payload.schema !== "ddn.ja.seulgi_replay_safe_workflow.v1") {
    throw new Error("seulgi_replay_safe_workflow_expected");
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
    `replay_safe_workflow_claim\t${payload.replay_safe_workflow_claim === true ? "true" : "false"}`,
    `ai_recall_claim\t${payload.ai_recall_claim === true ? "true" : "false"}`,
    `auto_apply_claim\t${payload.auto_apply_claim === true ? "true" : "false"}`,
    `file_write_claim\t${payload.file_write_claim === true ? "true" : "false"}`,
    `runtime_ast_persisted\t${payload.runtime_ast_persisted === true ? "true" : "false"}`,
    `state_hash_owner\t${payload.state_hash_owner === true ? "true" : "false"}`,
    "",
    "row_id\tkind\treplay\tready",
    ...rows.map((row) => `${row.id}\t${row.kind}\t${row.replay}\t${row.ready === true ? "true" : "false"}`),
  ].join("\n");
}

export function renderSeulgiReplaySafeWorkflow(root, workflow = {}) {
  if (!root) return null;
  const payload = asObject(workflow);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.seulgiReplaySafeStatus = asText(payload.status, "seulgi_replay_safe_workflow_incomplete");
  root.innerHTML = `
    <div class="question-card-head">
      <div>
        <div class="question-card-kicker">Seulgi Replay Safe</div>
        <h2>Replay-safe AI workflow</h2>
      </div>
      <div class="question-card-progress" data-seulgi-replay-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="question-card-summary" data-seulgi-replay-summary>
      입력 스냅샷, 승인 영수증, 아티팩트 봉인, AI 재호출 없음, LTS 게이트를 로컬 replay workflow로 묶습니다. AI recall, auto-apply, file write는 이번 범위가 아닙니다.
    </div>
    <div class="question-card-body">
      <div class="question-card-list">
        ${rows.map((row) => `
          <button type="button" class="question-card-btn${row.id === activeId ? " active" : ""}" data-seulgi-replay-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.kind)} · ${escapeHtml(row.replay)}</small>
          </button>
        `).join("")}
      </div>
      <div class="question-card-detail">
        <div class="question-card-title" data-seulgi-replay-active-title>${escapeHtml(active.title)}</div>
        <p data-seulgi-replay-active-link>${escapeHtml(active.uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>replay</dt><dd>${escapeHtml(active.replay ?? "")}</dd></div>
        </dl>
        <div class="question-card-artifacts">
          ${rows.map((row) => `
            <span data-seulgi-replay-artifact="${escapeHtml(row.id)}">${escapeHtml(row.kind)} · ${escapeHtml(row.replay)}</span>
          `).join("")}
        </div>
        <pre class="question-card-preview" data-seulgi-replay-preview>${escapeHtml(payload.replay_text ?? "")}</pre>
        <button type="button" class="ghost" data-seulgi-replay-copy>Replay-safe workflow 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderSeulgiReplaySafeWorkflow(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-seulgi-replay-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-seulgi-replay-row") || ""));
  });
  root.querySelector("[data-seulgi-replay-copy]")?.addEventListener("click", async () => {
    root.dataset.seulgiReplayCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatSeulgiReplaySafeWorkflowText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
