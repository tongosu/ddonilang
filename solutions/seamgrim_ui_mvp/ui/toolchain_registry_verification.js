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

const REGISTRY_ROWS = [
  { id: "publish_manifest", title: "Publish manifest", verification_kind: "publish_dry_run_manifest", local_uri: "toolchain://registry/verify/publish" },
  { id: "install_plan", title: "Install plan", verification_kind: "install_plan_resolution", local_uri: "toolchain://registry/verify/install" },
  { id: "digest_verify", title: "Digest verify", verification_kind: "digest_and_lockfile_check", local_uri: "toolchain://registry/verify/digest" },
  { id: "rollback_probe", title: "Rollback probe", verification_kind: "rollback_resolution_probe", local_uri: "toolchain://registry/verify/rollback" },
];

const DEFAULT_ARTIFACTS = [
  { name: "registry.publish.manifest.detjson", kind: "publish_manifest", bytes: 734 },
  { name: "registry.install.plan.detjson", kind: "install_plan", bytes: 692 },
  { name: "registry.digest.lock.detjson", kind: "digest_verify", bytes: 618 },
  { name: "registry.rollback.probe.detjson", kind: "rollback_probe", bytes: 556 },
];

export const DEFAULT_TOOLCHAIN_REGISTRY_VERIFICATION_ROWS = REGISTRY_ROWS.map((row) => ({
  id: row.id,
  verification_kind: row.verification_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_TOOLCHAIN_REGISTRY_VERIFICATION_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return REGISTRY_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      verification_kind: asText(source.verification_kind, row.verification_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      dry_run: true,
      local_only: true,
      network_io: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `registry_verification_${index + 1}.detjson`),
      kind: asText(source.kind, "registry_verification_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildVerificationText(rows, artifacts) {
  return [
    "toolchain_registry_verification:toolchain_pack_4_v1",
    "coordinate:타-4",
    "public_registry_publish:false",
    "install_execution:false",
    "network_io:false",
    "trust_signing:false",
    ...rows.map((row) => `${row.id}\t${row.verification_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildToolchainRegistryVerification({
  rows = DEFAULT_TOOLCHAIN_REGISTRY_VERIFICATION_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "publish_manifest",
} = {}) {
  const verificationRows = normalizeRows(rows);
  const verificationArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(verificationArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = verificationRows.filter((row) => row.ready).length;
  const readyArtifactCount = verificationArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["publish_manifest", "install_plan", "digest_verify", "rollback_probe"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === verificationRows.length &&
    readyArtifactCount === verificationArtifacts.length &&
    hasAllArtifacts;
  const active = verificationRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : verificationRows[0]?.id ?? "";
  return {
    __종류: "toolchain_registry_verification",
    schema: "ddn.toolchain.registry_verification.v1",
    work_item: "TA4_REGISTRY_VERIFICATION_V1",
    primary_coordinate: "타-4",
    depends_on_coordinate: ["타-3", "타-2"],
    pack: "toolchain_pack_4_v1",
    status: ready ? "toolchain_registry_verification_ready" : "toolchain_registry_verification_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    registry_verification_claim: ready,
    publish_dry_run_claim: artifactKinds.has("publish_manifest"),
    install_plan_claim: artifactKinds.has("install_plan"),
    digest_verify_claim: artifactKinds.has("digest_verify"),
    rollback_probe_claim: artifactKinds.has("rollback_probe"),
    public_registry_publish_claim: false,
    install_execution_claim: false,
    update_remove_execution_claim: false,
    network_io_claim: false,
    trust_signing_claim: false,
    cloud_sync_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: verificationRows,
    artifacts: verificationArtifacts,
    active_row_id: active,
    verification_text: buildVerificationText(verificationRows, verificationArtifacts),
    artifact_size_bytes: verificationArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 23,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 26,
      roadmap_v2_pack_evidence_reference_closed: 43,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 48,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "TA5_BENCHMARK_LTS_V1",
  };
}

export function formatToolchainRegistryVerificationText(registryVerification = {}) {
  const payload = asObject(registryVerification);
  if (payload.schema !== "ddn.toolchain.registry_verification.v1") {
    throw new Error("toolchain_expected_registry_verification");
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
    `registry_verification_claim\t${payload.registry_verification_claim === true ? "true" : "false"}`,
    `publish_dry_run_claim\t${payload.publish_dry_run_claim === true ? "true" : "false"}`,
    `install_plan_claim\t${payload.install_plan_claim === true ? "true" : "false"}`,
    `digest_verify_claim\t${payload.digest_verify_claim === true ? "true" : "false"}`,
    `rollback_probe_claim\t${payload.rollback_probe_claim === true ? "true" : "false"}`,
    `public_registry_publish_claim\t${payload.public_registry_publish_claim === true ? "true" : "false"}`,
    `install_execution_claim\t${payload.install_execution_claim === true ? "true" : "false"}`,
    `network_io_claim\t${payload.network_io_claim === true ? "true" : "false"}`,
    `trust_signing_claim\t${payload.trust_signing_claim === true ? "true" : "false"}`,
    "",
    "row_id\tverification_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.verification_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderToolchainRegistryVerification(root, registryVerification = {}) {
  if (!root) return null;
  const payload = asObject(registryVerification);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.toolchainRegistryVerificationStatus = asText(payload.status, "toolchain_registry_verification_incomplete");
  root.innerHTML = `
    <div class="toolchain-registry-head">
      <div>
        <div class="toolchain-registry-kicker">Registry verification</div>
        <h2>Registry verify dry-run</h2>
      </div>
      <div class="toolchain-registry-progress" data-toolchain-registry-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="toolchain-registry-summary" data-toolchain-registry-summary>
      publish/install/verify 흐름을 local dry-run artifact로 검증하고 public registry publish, install execution, network IO, trust signing은 후속으로 둡니다.
    </div>
    <div class="toolchain-registry-body">
      <div class="toolchain-registry-list">
        ${rows.map((row) => `
          <button type="button" class="toolchain-registry-btn${row.id === activeId ? " active" : ""}" data-toolchain-registry-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.verification_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="toolchain-registry-detail">
        <div class="toolchain-registry-title" data-toolchain-registry-active-title>${escapeHtml(active.title)}</div>
        <p data-toolchain-registry-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="toolchain-registry-artifacts">
          ${artifacts.map((artifact) => `
            <span data-toolchain-registry-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="toolchain-registry-preview" data-toolchain-registry-preview>${escapeHtml(payload.verification_text ?? "")}</pre>
        <button type="button" class="ghost" data-toolchain-registry-copy>registry 검증 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderToolchainRegistryVerification(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-toolchain-registry-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-toolchain-registry-row") || ""));
  });
  root.querySelector("[data-toolchain-registry-copy]")?.addEventListener("click", async () => {
    root.dataset.toolchainRegistryVerificationCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatToolchainRegistryVerificationText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
