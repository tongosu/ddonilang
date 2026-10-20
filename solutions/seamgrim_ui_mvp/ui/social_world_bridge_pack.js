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

const SOCIAL_BRIDGE_ROWS = [
  { id: "model_first", title: "Model-first", bridge_kind: "reduced_form_market_model", local_uri: "social-world://bridge/model-first" },
  { id: "agent_first", title: "Agent-first", bridge_kind: "agent_boundary_stub", local_uri: "social-world://bridge/agent-first" },
  { id: "bridge_report", title: "Bridge report", bridge_kind: "model_agent_bridge_report", local_uri: "social-world://bridge/report" },
  { id: "policy_handoff", title: "Policy handoff", bridge_kind: "classroom_policy_handoff", local_uri: "social-world://bridge/policy-handoff" },
];

const DEFAULT_ARTIFACTS = [
  { name: "social.model.first.detjson", kind: "model_first", bytes: 716 },
  { name: "social.agent.boundary.detjson", kind: "agent_first", bytes: 642 },
  { name: "social.bridge.report.detjson", kind: "bridge_report", bytes: 824 },
  { name: "social.policy.handoff.detjson", kind: "policy_handoff", bytes: 588 },
];

export const DEFAULT_SOCIAL_WORLD_BRIDGE_ROWS = SOCIAL_BRIDGE_ROWS.map((row) => ({
  id: row.id,
  bridge_kind: row.bridge_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_SOCIAL_WORLD_BRIDGE_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return SOCIAL_BRIDGE_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      bridge_kind: asText(source.bridge_kind, row.bridge_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      view_snapshot_claim: true,
      state_hash_participation: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `social_bridge_${index + 1}.detjson`),
      kind: asText(source.kind, "social_bridge_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildBridgeText(rows, artifacts) {
  return [
    "social_world_bridge:social_world_econ_2_v1",
    "coordinate:파-2",
    "real_world_prediction:false",
    "policy_advice:false",
    "agent_simulation_execution:false",
    "new_economic_theory:false",
    "state_hash_participation:false",
    ...rows.map((row) => `${row.id}\t${row.bridge_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildSocialWorldBridgePack({
  rows = DEFAULT_SOCIAL_WORLD_BRIDGE_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "model_first",
} = {}) {
  const bridgeRows = normalizeRows(rows);
  const bridgeArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(bridgeArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = bridgeRows.filter((row) => row.ready).length;
  const readyArtifactCount = bridgeArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["model_first", "agent_first", "bridge_report", "policy_handoff"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === bridgeRows.length &&
    readyArtifactCount === bridgeArtifacts.length &&
    hasAllArtifacts;
  const active = bridgeRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : bridgeRows[0]?.id ?? "";
  return {
    __종류: "social_world_bridge_pack",
    schema: "ddn.social_world.bridge_pack.v1",
    work_item: "PA2_SOCIAL_BRIDGE_PACK_V1",
    primary_coordinate: "파-2",
    depends_on_coordinate: ["파-1", "파-0", "타-2"],
    pack: "social_world_econ_2_v1",
    status: ready ? "social_world_bridge_pack_ready" : "social_world_bridge_pack_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    social_bridge_pack_claim: ready,
    model_first_claim: artifactKinds.has("model_first"),
    agent_first_claim: artifactKinds.has("agent_first"),
    bridge_report_claim: artifactKinds.has("bridge_report"),
    policy_handoff_claim: artifactKinds.has("policy_handoff"),
    real_world_prediction_claim: false,
    policy_advice_claim: false,
    agent_simulation_execution_claim: false,
    new_economic_theory_claim: false,
    state_hash_participation_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: bridgeRows,
    artifacts: bridgeArtifacts,
    active_row_id: active,
    bridge_text: buildBridgeText(bridgeRows, bridgeArtifacts),
    artifact_size_bytes: bridgeArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 25,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 28,
      roadmap_v2_pack_evidence_reference_closed: 45,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 50,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "PA3_POLICY_GHOST_UI_V1",
  };
}

export function formatSocialWorldBridgePackText(bridgePack = {}) {
  const payload = asObject(bridgePack);
  if (payload.schema !== "ddn.social_world.bridge_pack.v1") {
    throw new Error("social_world_expected_bridge_pack");
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
    `social_bridge_pack_claim\t${payload.social_bridge_pack_claim === true ? "true" : "false"}`,
    `model_first_claim\t${payload.model_first_claim === true ? "true" : "false"}`,
    `agent_first_claim\t${payload.agent_first_claim === true ? "true" : "false"}`,
    `bridge_report_claim\t${payload.bridge_report_claim === true ? "true" : "false"}`,
    `policy_handoff_claim\t${payload.policy_handoff_claim === true ? "true" : "false"}`,
    `real_world_prediction_claim\t${payload.real_world_prediction_claim === true ? "true" : "false"}`,
    `policy_advice_claim\t${payload.policy_advice_claim === true ? "true" : "false"}`,
    `agent_simulation_execution_claim\t${payload.agent_simulation_execution_claim === true ? "true" : "false"}`,
    `new_economic_theory_claim\t${payload.new_economic_theory_claim === true ? "true" : "false"}`,
    "",
    "row_id\tbridge_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.bridge_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderSocialWorldBridgePack(root, bridgePack = {}) {
  if (!root) return null;
  const payload = asObject(bridgePack);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.socialWorldBridgePackStatus = asText(payload.status, "social_world_bridge_pack_incomplete");
  root.innerHTML = `
    <div class="social-bridge-head">
      <div>
        <div class="social-bridge-kicker">Social World</div>
        <h2>Social bridge pack</h2>
      </div>
      <div class="social-bridge-progress" data-social-bridge-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="social-bridge-summary" data-social-bridge-summary>
      model-first, agent-first, bridge report, policy handoff를 local report pack으로 묶고 real-world prediction, policy advice, agent simulation execution은 후속으로 둡니다.
    </div>
    <div class="social-bridge-body">
      <div class="social-bridge-list">
        ${rows.map((row) => `
          <button type="button" class="social-bridge-btn${row.id === activeId ? " active" : ""}" data-social-bridge-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.bridge_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="social-bridge-detail">
        <div class="social-bridge-title" data-social-bridge-active-title>${escapeHtml(active.title)}</div>
        <p data-social-bridge-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="social-bridge-artifacts">
          ${artifacts.map((artifact) => `
            <span data-social-bridge-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="social-bridge-preview" data-social-bridge-preview>${escapeHtml(payload.bridge_text ?? "")}</pre>
        <button type="button" class="ghost" data-social-bridge-copy>bridge pack 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderSocialWorldBridgePack(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-social-bridge-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-social-bridge-row") || ""));
  });
  root.querySelector("[data-social-bridge-copy]")?.addEventListener("click", async () => {
    root.dataset.socialWorldBridgePackCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatSocialWorldBridgePackText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
