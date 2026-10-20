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

const POLICY_ROWS = [
  { id: "baseline_run", title: "Baseline run", ghost_kind: "reduced_form_baseline_trace", local_uri: "social-world://policy-ghost/baseline" },
  { id: "policy_variant", title: "Policy variant", ghost_kind: "policy_variant_trace", local_uri: "social-world://policy-ghost/variant" },
  { id: "ghost_overlay", title: "Ghost overlay", ghost_kind: "ghost_overlay_preview", local_uri: "social-world://policy-ghost/overlay" },
  { id: "classroom_compare", title: "Classroom compare", ghost_kind: "classroom_compare_packet", local_uri: "social-world://policy-ghost/classroom" },
];

const DEFAULT_ARTIFACTS = [
  { name: "policy.baseline.trace.detjson", kind: "baseline_run", bytes: 704 },
  { name: "policy.variant.trace.detjson", kind: "policy_variant", bytes: 736 },
  { name: "policy.ghost.overlay.detjson", kind: "ghost_overlay", bytes: 818 },
  { name: "policy.classroom.compare.detjson", kind: "classroom_compare", bytes: 624 },
];

export const DEFAULT_SOCIAL_WORLD_POLICY_GHOST_ROWS = POLICY_ROWS.map((row) => ({
  id: row.id,
  ghost_kind: row.ghost_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_SOCIAL_WORLD_POLICY_GHOST_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return POLICY_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      ghost_kind: asText(source.ghost_kind, row.ghost_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_preview_claim: true,
      state_hash_participation: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `policy_ghost_${index + 1}.detjson`),
      kind: asText(source.kind, "policy_ghost_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildGhostText(rows, artifacts) {
  return [
    "social_world_policy_ghost:social_world_econ_3_v1",
    "coordinate:파-3",
    "real_world_prediction:false",
    "policy_advice:false",
    "agent_simulation_execution:false",
    "live_policy_deployment:false",
    "state_hash_participation:false",
    ...rows.map((row) => `${row.id}\t${row.ghost_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildSocialWorldPolicyGhostUi({
  rows = DEFAULT_SOCIAL_WORLD_POLICY_GHOST_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "baseline_run",
} = {}) {
  const ghostRows = normalizeRows(rows);
  const ghostArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(ghostArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = ghostRows.filter((row) => row.ready).length;
  const readyArtifactCount = ghostArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["baseline_run", "policy_variant", "ghost_overlay", "classroom_compare"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === ghostRows.length &&
    readyArtifactCount === ghostArtifacts.length &&
    hasAllArtifacts;
  const active = ghostRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : ghostRows[0]?.id ?? "";
  return {
    __종류: "social_world_policy_ghost_ui",
    schema: "ddn.social_world.policy_ghost_ui.v1",
    work_item: "PA3_POLICY_GHOST_UI_V1",
    primary_coordinate: "파-3",
    depends_on_coordinate: ["파-2", "파-1", "타-2"],
    pack: "social_world_econ_3_v1",
    status: ready ? "social_world_policy_ghost_ready" : "social_world_policy_ghost_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    policy_ghost_ui_claim: ready,
    compare_run_claim: artifactKinds.has("baseline_run") && artifactKinds.has("policy_variant"),
    ghost_overlay_claim: artifactKinds.has("ghost_overlay"),
    classroom_compare_claim: artifactKinds.has("classroom_compare"),
    real_world_prediction_claim: false,
    policy_advice_claim: false,
    agent_simulation_execution_claim: false,
    live_policy_deployment_claim: false,
    state_hash_participation_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: ghostRows,
    artifacts: ghostArtifacts,
    active_row_id: active,
    ghost_text: buildGhostText(ghostRows, ghostArtifacts),
    artifact_size_bytes: ghostArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 26,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 29,
      roadmap_v2_pack_evidence_reference_closed: 46,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 51,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "PA4_SOCIAL_TEMPLATE_REGISTRY_V1",
  };
}

export function formatSocialWorldPolicyGhostUiText(policyGhost = {}) {
  const payload = asObject(policyGhost);
  if (payload.schema !== "ddn.social_world.policy_ghost_ui.v1") {
    throw new Error("social_world_expected_policy_ghost_ui");
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
    `policy_ghost_ui_claim\t${payload.policy_ghost_ui_claim === true ? "true" : "false"}`,
    `compare_run_claim\t${payload.compare_run_claim === true ? "true" : "false"}`,
    `ghost_overlay_claim\t${payload.ghost_overlay_claim === true ? "true" : "false"}`,
    `classroom_compare_claim\t${payload.classroom_compare_claim === true ? "true" : "false"}`,
    `real_world_prediction_claim\t${payload.real_world_prediction_claim === true ? "true" : "false"}`,
    `policy_advice_claim\t${payload.policy_advice_claim === true ? "true" : "false"}`,
    `agent_simulation_execution_claim\t${payload.agent_simulation_execution_claim === true ? "true" : "false"}`,
    `live_policy_deployment_claim\t${payload.live_policy_deployment_claim === true ? "true" : "false"}`,
    "",
    "row_id\tghost_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.ghost_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderSocialWorldPolicyGhostUi(root, policyGhost = {}) {
  if (!root) return null;
  const payload = asObject(policyGhost);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.socialWorldPolicyGhostStatus = asText(payload.status, "social_world_policy_ghost_incomplete");
  root.innerHTML = `
    <div class="social-policy-head">
      <div>
        <div class="social-policy-kicker">Policy Ghost</div>
        <h2>Social policy ghost UI</h2>
      </div>
      <div class="social-policy-progress" data-social-policy-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="social-policy-summary" data-social-policy-summary>
      baseline/variant compare와 ghost overlay preview를 local UI로 고정하고 real-world prediction, policy advice, live policy deployment는 후속으로 둡니다.
    </div>
    <div class="social-policy-body">
      <div class="social-policy-list">
        ${rows.map((row) => `
          <button type="button" class="social-policy-btn${row.id === activeId ? " active" : ""}" data-social-policy-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.ghost_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="social-policy-detail">
        <div class="social-policy-title" data-social-policy-active-title>${escapeHtml(active.title)}</div>
        <p data-social-policy-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="social-policy-artifacts">
          ${artifacts.map((artifact) => `
            <span data-social-policy-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="social-policy-preview" data-social-policy-preview>${escapeHtml(payload.ghost_text ?? "")}</pre>
        <button type="button" class="ghost" data-social-policy-copy>policy ghost 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderSocialWorldPolicyGhostUi(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-social-policy-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-social-policy-row") || ""));
  });
  root.querySelector("[data-social-policy-copy]")?.addEventListener("click", async () => {
    root.dataset.socialWorldPolicyGhostCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatSocialWorldPolicyGhostUiText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
