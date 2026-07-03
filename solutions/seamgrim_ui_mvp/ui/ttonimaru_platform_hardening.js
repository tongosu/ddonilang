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
  { id: "auth_boundary", title: "Auth boundary", hardening_kind: "local_auth_boundary", local_uri: "ttonimaru://platform/hardening/local/auth" },
  { id: "rbac_matrix", title: "RBAC matrix", hardening_kind: "role_capability_matrix", local_uri: "ttonimaru://platform/hardening/local/rbac" },
  { id: "audit_log", title: "Audit log", hardening_kind: "append_only_audit_preview", local_uri: "ttonimaru://platform/hardening/local/audit" },
  { id: "backup_plan", title: "Backup plan", hardening_kind: "local_backup_restore_plan", local_uri: "ttonimaru://platform/hardening/local/backup" },
];

const DEFAULT_HARDENING_ARTIFACTS = [
  { name: "platform.auth.boundary.detjson", kind: "auth_boundary", bytes: 612 },
  { name: "platform.rbac.matrix.detjson", kind: "rbac_matrix", bytes: 548 },
  { name: "platform.audit.log.preview.detjson", kind: "audit_log", bytes: 684 },
  { name: "platform.backup.plan.detjson", kind: "backup_plan", bytes: 506 },
];

export const DEFAULT_TTONIMARU_PLATFORM_HARDENING_ROWS = HARDENING_ROWS.map((row) => ({
  id: row.id,
  hardening_kind: row.hardening_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_TTONIMARU_PLATFORM_HARDENING_ROWS) {
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
      local_only: true,
      product_ui_behavior: true,
      cloud_account_claim: false,
      cryptographic_signing_claim: false,
      production_backup_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_HARDENING_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_HARDENING_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `platform_hardening_${index + 1}.detjson`),
      kind: asText(source.kind, "platform_hardening_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildHardeningText(artifacts) {
  return [
    "ttonimaru_platform_hardening:ttonimaru_registry_5_v1",
    "coordinate:카-5",
    "production_deploy:false",
    "cloud_account:false",
    "cryptographic_signing:false",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildTtonimaruPlatformHardening({
  rows = DEFAULT_TTONIMARU_PLATFORM_HARDENING_ROWS,
  artifacts = DEFAULT_HARDENING_ARTIFACTS,
  activeHardeningId = "auth_boundary",
} = {}) {
  const hardeningRows = normalizeRows(rows);
  const hardeningArtifacts = normalizeArtifacts(artifacts);
  const active = hardeningRows.some((row) => row.id === activeHardeningId)
    ? activeHardeningId
    : hardeningRows[0]?.id ?? "";
  const readyRowCount = hardeningRows.filter((row) => row.ready).length;
  const readyArtifactCount = hardeningArtifacts.filter((artifact) => artifact.ready).length;
  const artifactKinds = new Set(hardeningArtifacts.map((artifact) => artifact.kind));
  const hasAuth = artifactKinds.has("auth_boundary");
  const hasRbac = artifactKinds.has("rbac_matrix");
  const hasAudit = artifactKinds.has("audit_log");
  const hasBackup = artifactKinds.has("backup_plan");
  const hardeningReady = readyRowCount === hardeningRows.length &&
    readyArtifactCount === hardeningArtifacts.length &&
    hasAuth && hasRbac && hasAudit && hasBackup;
  return {
    __종류: "ttonimaru_platform_hardening",
    schema: "ddn.ttonimaru.platform_hardening.v1",
    work_item: "KA5_PLATFORM_HARDENING_V1",
    primary_coordinate: "카-5",
    depends_on_coordinate: ["카-4"],
    pack: "ttonimaru_registry_5_v1",
    status: hardeningReady ? "ttonimaru_platform_hardening_ready" : "ttonimaru_platform_hardening_incomplete",
    matrix_closure_tier: hardeningReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    platform_hardening_claim: hardeningReady,
    auth_boundary_claim: hasAuth,
    rbac_matrix_claim: hasRbac,
    audit_log_claim: hasAudit,
    backup_plan_claim: hasBackup,
    public_registry_final_claim: false,
    registry_publish_claim: false,
    production_deploy_claim: false,
    cloud_account_claim: false,
    cryptographic_signing_claim: false,
    production_backup_claim: false,
    moderation_claim: false,
    team_membership_claim: false,
    hardening_row_count: hardeningRows.length,
    artifact_count: hardeningArtifacts.length,
    artifact_size_bytes: hardeningArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: hardeningReady ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: hardeningReady ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 21,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 23,
      roadmap_v2_pack_evidence_reference_closed: 41,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 46,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    hardening_rows: hardeningRows,
    artifacts: hardeningArtifacts,
    hardening_text: buildHardeningText(hardeningArtifacts),
    active_hardening_id: active,
    next_item: "TA3_DIAGNOSTIC_UI_LSP_V1",
  };
}

export function formatTtonimaruPlatformHardeningText(platformHardening = {}) {
  const payload = asObject(platformHardening);
  if (payload.schema !== "ddn.ttonimaru.platform_hardening.v1") {
    throw new Error("ttonimaru_expected_platform_hardening");
  }
  const rows = asArray(payload.hardening_rows);
  const artifacts = asArray(payload.artifacts);
  return [
    `schema\t${payload.schema}`,
    `work_item\t${payload.work_item ?? ""}`,
    `primary_coordinate\t${payload.primary_coordinate ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `matrix_closure_tier\t${payload.matrix_closure_tier ?? ""}`,
    `current_stage\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? 0}`,
    `roadmap_matrix\t${payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0}/${payload.progress?.roadmap_v2_matrix_behavior_total ?? 0}`,
    `roadmap_matrix_percent\t${payload.progress?.roadmap_v2_matrix_behavior_percent ?? 0}`,
    `pack_evidence_reference\t${payload.progress?.roadmap_v2_pack_evidence_reference_closed ?? 0}/${payload.progress?.roadmap_v2_pack_evidence_reference_total ?? 0}`,
    `pack_evidence_reference_percent\t${payload.progress?.roadmap_v2_pack_evidence_reference_percent ?? 0}`,
    `studio_local_super_long\t${payload.progress?.studio_local_super_long_closed ?? 0}/${payload.progress?.studio_local_super_long_total ?? 0}`,
    `studio_local_super_long_percent\t${payload.progress?.studio_local_super_long_percent ?? 0}`,
    `platform_hardening_claim\t${payload.platform_hardening_claim === true ? "true" : "false"}`,
    `auth_boundary_claim\t${payload.auth_boundary_claim === true ? "true" : "false"}`,
    `rbac_matrix_claim\t${payload.rbac_matrix_claim === true ? "true" : "false"}`,
    `audit_log_claim\t${payload.audit_log_claim === true ? "true" : "false"}`,
    `backup_plan_claim\t${payload.backup_plan_claim === true ? "true" : "false"}`,
    `production_deploy_claim\t${payload.production_deploy_claim === true ? "true" : "false"}`,
    `cloud_account_claim\t${payload.cloud_account_claim === true ? "true" : "false"}`,
    `cryptographic_signing_claim\t${payload.cryptographic_signing_claim === true ? "true" : "false"}`,
    `production_backup_claim\t${payload.production_backup_claim === true ? "true" : "false"}`,
    "",
    "hardening_id\thardening_kind\tlocal_only\tlocal_uri",
    ...rows.map((row) => [
      row.id,
      row.hardening_kind,
      row.local_only === true ? "true" : "false",
      row.local_uri,
    ].join("\t")),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => [
      artifact.name,
      artifact.kind,
      artifact.bytes,
    ].join("\t")),
  ].join("\n");
}

export function renderTtonimaruPlatformHardening(root, platformHardening = {}) {
  if (!root) return null;
  const payload = asObject(platformHardening);
  const rows = asArray(payload.hardening_rows);
  const activeId = asText(payload.active_hardening_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  const artifacts = asArray(payload.artifacts);
  root.dataset.ttonimaruPlatformHardeningStatus = asText(payload.status, "ttonimaru_platform_hardening_incomplete");
  root.innerHTML = `
    <div class="ttonimaru-hardening-head">
      <div>
        <div class="ttonimaru-hardening-kicker">Platform hardening</div>
        <h2>Ttonimaru hardening</h2>
      </div>
      <div class="ttonimaru-hardening-progress" data-ttonimaru-hardening-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="ttonimaru-hardening-summary" data-ttonimaru-hardening-summary>
      Auth boundary, RBAC matrix, append-only audit preview, local backup plan을 묶고 production deploy, cloud account, cryptographic signing, production backup은 후속으로 둡니다.
    </div>
    <div class="ttonimaru-hardening-body">
      <div class="ttonimaru-hardening-list" data-ttonimaru-hardening-list>
        ${rows.map((row) => `
          <button type="button" class="ttonimaru-hardening-btn${row.id === activeId ? " active" : ""}" data-ttonimaru-hardening="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.hardening_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="ttonimaru-hardening-detail" data-ttonimaru-hardening-detail>
        <div class="ttonimaru-hardening-title" data-ttonimaru-hardening-active-title>${escapeHtml(active.title)}</div>
        <p data-ttonimaru-hardening-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="ttonimaru-hardening-artifacts" data-ttonimaru-hardening-artifacts>
          ${artifacts.map((artifact) => `
            <span data-ttonimaru-hardening-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="ttonimaru-hardening-preview" data-ttonimaru-hardening-preview>${escapeHtml(payload.hardening_text || "")}</pre>
        <button type="button" class="ghost" data-ttonimaru-hardening-copy>hardening 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (hardeningId) => renderTtonimaruPlatformHardening(root, { ...payload, active_hardening_id: hardeningId });
  root.querySelectorAll("[data-ttonimaru-hardening]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-ttonimaru-hardening") || ""));
  });
  root.querySelector("[data-ttonimaru-hardening-copy]")?.addEventListener("click", async () => {
    root.dataset.ttonimaruPlatformHardeningCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatTtonimaruPlatformHardeningText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
