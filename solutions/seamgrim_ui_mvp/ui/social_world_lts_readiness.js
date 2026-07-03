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

const LTS_ROWS = [
  { id: "education_stability", title: "Education stability", lts_kind: "education_model_stability", local_uri: "social-world://lts/education-stability" },
  { id: "policy_regression", title: "Policy regression", lts_kind: "policy_model_regression_matrix", local_uri: "social-world://lts/policy-regression" },
  { id: "history_fixture", title: "History fixture", lts_kind: "history_scenario_fixture", local_uri: "social-world://lts/history-fixture" },
  { id: "lts_gate", title: "LTS gate", lts_kind: "local_lts_gate_rehearsal", local_uri: "social-world://lts/gate" },
];

const DEFAULT_ARTIFACTS = [
  { name: "social.lts.education.stability.detjson", kind: "education_stability", bytes: 824 },
  { name: "social.lts.policy.regression.detjson", kind: "policy_regression", bytes: 868 },
  { name: "social.lts.history.fixture.detjson", kind: "history_fixture", bytes: 742 },
  { name: "social.lts.gate.detjson", kind: "lts_gate", bytes: 694 },
];

export const DEFAULT_SOCIAL_WORLD_LTS_ROWS = LTS_ROWS.map((row) => ({
  id: row.id,
  lts_kind: row.lts_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_SOCIAL_WORLD_LTS_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return LTS_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      lts_kind: asText(source.lts_kind, row.lts_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_lts_readiness_claim: true,
      remote_certification_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `social_lts_${index + 1}.detjson`),
      kind: asText(source.kind, "social_lts_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildLtsText(rows, artifacts) {
  return [
    "social_world_lts_readiness:social_world_econ_5_v1",
    "coordinate:파-5",
    "remote_lts_certification:false",
    "public_release_execution:false",
    "live_policy_deployment:false",
    "real_world_prediction:false",
    "policy_advice:false",
    "state_hash_participation:false",
    ...rows.map((row) => `${row.id}\t${row.lts_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildSocialWorldLtsReadiness({
  rows = DEFAULT_SOCIAL_WORLD_LTS_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "education_stability",
} = {}) {
  const ltsRows = normalizeRows(rows);
  const ltsArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(ltsArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = ltsRows.filter((row) => row.ready).length;
  const readyArtifactCount = ltsArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["education_stability", "policy_regression", "history_fixture", "lts_gate"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === ltsRows.length &&
    readyArtifactCount === ltsArtifacts.length &&
    hasAllArtifacts;
  const active = ltsRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : ltsRows[0]?.id ?? "";
  return {
    __종류: "social_world_lts_readiness",
    schema: "ddn.social_world.lts_readiness.v1",
    work_item: "PA5_SOCIAL_WORLD_LTS_V1",
    primary_coordinate: "파-5",
    depends_on_coordinate: ["파-4", "파-3", "타-2"],
    pack: "social_world_econ_5_v1",
    status: ready ? "social_world_lts_readiness_ready" : "social_world_lts_readiness_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    social_world_lts_claim: ready,
    education_stability_claim: artifactKinds.has("education_stability"),
    policy_regression_claim: artifactKinds.has("policy_regression"),
    history_fixture_claim: artifactKinds.has("history_fixture"),
    lts_gate_claim: artifactKinds.has("lts_gate"),
    remote_lts_certification_claim: false,
    public_release_execution_claim: false,
    live_policy_deployment_claim: false,
    real_world_prediction_claim: false,
    policy_advice_claim: false,
    state_hash_participation_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: ltsRows,
    artifacts: ltsArtifacts,
    active_row_id: active,
    lts_text: buildLtsText(ltsRows, ltsArtifacts),
    artifact_size_bytes: ltsArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 28,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 31,
      roadmap_v2_pack_evidence_reference_closed: 48,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 53,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "HA2_EDUCATION_ASSESSMENT_PACK_V1",
  };
}

export function formatSocialWorldLtsReadinessText(ltsReadiness = {}) {
  const payload = asObject(ltsReadiness);
  if (payload.schema !== "ddn.social_world.lts_readiness.v1") {
    throw new Error("social_world_expected_lts_readiness");
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
    `social_world_lts_claim\t${payload.social_world_lts_claim === true ? "true" : "false"}`,
    `education_stability_claim\t${payload.education_stability_claim === true ? "true" : "false"}`,
    `policy_regression_claim\t${payload.policy_regression_claim === true ? "true" : "false"}`,
    `history_fixture_claim\t${payload.history_fixture_claim === true ? "true" : "false"}`,
    `lts_gate_claim\t${payload.lts_gate_claim === true ? "true" : "false"}`,
    `remote_lts_certification_claim\t${payload.remote_lts_certification_claim === true ? "true" : "false"}`,
    `public_release_execution_claim\t${payload.public_release_execution_claim === true ? "true" : "false"}`,
    `live_policy_deployment_claim\t${payload.live_policy_deployment_claim === true ? "true" : "false"}`,
    `real_world_prediction_claim\t${payload.real_world_prediction_claim === true ? "true" : "false"}`,
    "",
    "row_id\tlts_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.lts_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderSocialWorldLtsReadiness(root, ltsReadiness = {}) {
  if (!root) return null;
  const payload = asObject(ltsReadiness);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.socialWorldLtsReadinessStatus = asText(payload.status, "social_world_lts_readiness_incomplete");
  root.innerHTML = `
    <div class="social-lts-head">
      <div>
        <div class="social-lts-kicker">Social World LTS</div>
        <h2>Social world LTS readiness</h2>
      </div>
      <div class="social-lts-progress" data-social-lts-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="social-lts-summary" data-social-lts-summary>
      education stability, policy regression, history fixture, local LTS gate를 readiness로 고정하고 remote LTS certification, public release execution, live policy deployment는 후속으로 둡니다.
    </div>
    <div class="social-lts-body">
      <div class="social-lts-list">
        ${rows.map((row) => `
          <button type="button" class="social-lts-btn${row.id === activeId ? " active" : ""}" data-social-lts-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.lts_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="social-lts-detail">
        <div class="social-lts-title" data-social-lts-active-title>${escapeHtml(active.title)}</div>
        <p data-social-lts-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="social-lts-artifacts">
          ${artifacts.map((artifact) => `
            <span data-social-lts-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="social-lts-preview" data-social-lts-preview>${escapeHtml(payload.lts_text ?? "")}</pre>
        <button type="button" class="ghost" data-social-lts-copy>LTS readiness 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderSocialWorldLtsReadiness(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-social-lts-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-social-lts-row") || ""));
  });
  root.querySelector("[data-social-lts-copy]")?.addEventListener("click", async () => {
    root.dataset.socialWorldLtsReadinessCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatSocialWorldLtsReadinessText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
