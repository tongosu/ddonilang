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
  {
    id: "seed_catalog",
    title: "Seed catalog",
    registry_kind: "curated_public_seed_catalog",
    local_uri: "ttonimaru://registry/seed/local/catalog",
  },
  {
    id: "lineage_record",
    title: "Lineage record",
    registry_kind: "revision_lineage_record",
    local_uri: "ttonimaru://registry/seed/local/lineage",
  },
  {
    id: "trust_badge",
    title: "Trust badge",
    registry_kind: "non_signing_trust_badge",
    local_uri: "ttonimaru://registry/seed/local/trust-badge",
  },
  {
    id: "seed_preview",
    title: "Seed preview",
    registry_kind: "read_only_registry_preview",
    local_uri: "ttonimaru://registry/seed/local/preview",
  },
];

const DEFAULT_REGISTRY_ARTIFACTS = [
  { name: "registry.seed.catalog.detjson", kind: "seed_catalog", bytes: 736 },
  { name: "registry.lineage.record.detjson", kind: "lineage_record", bytes: 584 },
  { name: "registry.trust.badge.detjson", kind: "trust_badge", bytes: 442 },
  { name: "registry.seed.preview.detjson", kind: "seed_preview", bytes: 508 },
];

export const DEFAULT_TTONIMARU_PUBLIC_REGISTRY_SEED_ROWS = REGISTRY_ROWS.map((row) => ({
  id: row.id,
  registry_kind: row.registry_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_TTONIMARU_PUBLIC_REGISTRY_SEED_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return REGISTRY_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      registry_kind: asText(source.registry_kind, row.registry_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_only: true,
      read_only: true,
      product_ui_behavior: true,
      registry_publish_claim: false,
      trust_signing_claim: false,
      cloud_sync_claim: false,
      account_permission_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_REGISTRY_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_REGISTRY_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `registry_seed_${index + 1}.detjson`),
      kind: asText(source.kind, "registry_seed_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildRegistryText(artifacts) {
  return [
    "ttonimaru_public_registry_seed:ttonimaru_registry_4_v1",
    "coordinate:카-4",
    "registry_publish:false",
    "trust_signing:false",
    "cloud_sync:false",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildTtonimaruPublicRegistrySeed({
  rows = DEFAULT_TTONIMARU_PUBLIC_REGISTRY_SEED_ROWS,
  artifacts = DEFAULT_REGISTRY_ARTIFACTS,
  activeRegistryId = "seed_catalog",
} = {}) {
  const registryRows = normalizeRows(rows);
  const registryArtifacts = normalizeArtifacts(artifacts);
  const active = registryRows.some((row) => row.id === activeRegistryId)
    ? activeRegistryId
    : registryRows[0]?.id ?? "";
  const readyRowCount = registryRows.filter((row) => row.ready).length;
  const readyArtifactCount = registryArtifacts.filter((artifact) => artifact.ready).length;
  const artifactKinds = new Set(registryArtifacts.map((artifact) => artifact.kind));
  const hasSeedCatalog = artifactKinds.has("seed_catalog");
  const hasLineage = artifactKinds.has("lineage_record");
  const hasTrustBadge = artifactKinds.has("trust_badge");
  const hasPreview = artifactKinds.has("seed_preview");
  const registryReady = (
    readyRowCount === registryRows.length &&
    readyArtifactCount === registryArtifacts.length &&
    hasSeedCatalog &&
    hasLineage &&
    hasTrustBadge &&
    hasPreview
  );
  return {
    __종류: "ttonimaru_public_registry_seed",
    schema: "ddn.ttonimaru.public_registry_seed.v1",
    work_item: "KA4_PUBLIC_REGISTRY_SEED_V1",
    primary_coordinate: "카-4",
    depends_on_coordinate: ["카-3"],
    pack: "ttonimaru_registry_4_v1",
    status: registryReady ? "ttonimaru_public_registry_seed_ready" : "ttonimaru_public_registry_seed_incomplete",
    matrix_closure_tier: registryReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    public_registry_seed_claim: registryReady,
    seed_catalog_claim: hasSeedCatalog,
    lineage_record_claim: hasLineage,
    trust_badge_claim: hasTrustBadge,
    registry_preview_claim: hasPreview,
    public_registry_final_claim: false,
    registry_publish_claim: false,
    install_update_remove_claim: false,
    trust_signing_claim: false,
    moderation_claim: false,
    team_membership_claim: false,
    account_permission_claim: false,
    cloud_sync_claim: false,
    platform_hardening_claim: false,
    registry_row_count: registryRows.length,
    artifact_count: registryArtifacts.length,
    artifact_size_bytes: registryArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    current_stage_closed: registryReady ? 5 : Math.min(4, readyRowCount),
    current_stage_total: 5,
    current_stage_percent: registryReady ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
    progress: {
      current_stage_closed: registryReady ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: registryReady ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 20,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 22,
      roadmap_v2_pack_evidence_reference_closed: 40,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 44,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    registry_rows: registryRows,
    artifacts: registryArtifacts,
    registry_text: buildRegistryText(registryArtifacts),
    active_registry_id: active,
    next_item: "KA5_PLATFORM_HARDENING_V1",
  };
}

export function formatTtonimaruPublicRegistrySeedText(registrySeed = {}) {
  const payload = asObject(registrySeed);
  if (payload.schema !== "ddn.ttonimaru.public_registry_seed.v1") {
    throw new Error("ttonimaru_expected_public_registry_seed");
  }
  const rows = asArray(payload.registry_rows);
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
    `public_registry_seed_claim\t${payload.public_registry_seed_claim === true ? "true" : "false"}`,
    `seed_catalog_claim\t${payload.seed_catalog_claim === true ? "true" : "false"}`,
    `lineage_record_claim\t${payload.lineage_record_claim === true ? "true" : "false"}`,
    `trust_badge_claim\t${payload.trust_badge_claim === true ? "true" : "false"}`,
    `registry_preview_claim\t${payload.registry_preview_claim === true ? "true" : "false"}`,
    `public_registry_final_claim\t${payload.public_registry_final_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `trust_signing_claim\t${payload.trust_signing_claim === true ? "true" : "false"}`,
    `moderation_claim\t${payload.moderation_claim === true ? "true" : "false"}`,
    `cloud_sync_claim\t${payload.cloud_sync_claim === true ? "true" : "false"}`,
    "",
    "registry_id\tregistry_kind\tread_only\tlocal_uri",
    ...rows.map((row) => [
      row.id,
      row.registry_kind,
      row.read_only === true ? "true" : "false",
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

export function renderTtonimaruPublicRegistrySeed(root, registrySeed = {}) {
  if (!root) return null;
  const payload = asObject(registrySeed);
  const rows = asArray(payload.registry_rows);
  const activeId = asText(payload.active_registry_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  const artifacts = asArray(payload.artifacts);
  root.dataset.ttonimaruPublicRegistrySeedStatus = asText(
    payload.status,
    "ttonimaru_public_registry_seed_incomplete",
  );
  root.innerHTML = `
    <div class="ttonimaru-registry-head">
      <div>
        <div class="ttonimaru-registry-kicker">Public registry seed</div>
        <h2>Ttonimaru registry seed</h2>
      </div>
      <div class="ttonimaru-registry-progress" data-ttonimaru-registry-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="ttonimaru-registry-summary" data-ttonimaru-registry-summary>
      Curated seed catalog, lineage record, non-signing trust badge, read-only preview를 묶고 registry publish, trust signing, moderation, cloud sync는 후속으로 둡니다.
    </div>
    <div class="ttonimaru-registry-body">
      <div class="ttonimaru-registry-list" data-ttonimaru-registry-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="ttonimaru-registry-btn${row.id === activeId ? " active" : ""}"
            data-ttonimaru-registry="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.registry_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="ttonimaru-registry-detail" data-ttonimaru-registry-detail>
        <div class="ttonimaru-registry-title" data-ttonimaru-registry-active-title>${escapeHtml(active.title)}</div>
        <p data-ttonimaru-registry-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="ttonimaru-registry-artifacts" data-ttonimaru-registry-artifacts>
          ${artifacts.map((artifact) => `
            <span data-ttonimaru-registry-artifact="${escapeHtml(artifact.name)}">
              ${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b
            </span>
          `).join("")}
        </div>
        <pre class="ttonimaru-registry-preview" data-ttonimaru-registry-preview>${escapeHtml(payload.registry_text || "")}</pre>
        <button type="button" class="ghost" data-ttonimaru-registry-copy>registry seed 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (registryId) => {
    renderTtonimaruPublicRegistrySeed(root, {
      ...payload,
      active_registry_id: registryId,
    });
  };
  root.querySelectorAll("[data-ttonimaru-registry]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-ttonimaru-registry") || "");
    });
  });
  root.querySelector("[data-ttonimaru-registry-copy]")?.addEventListener("click", async () => {
    const text = formatTtonimaruPublicRegistrySeedText(payload);
    root.dataset.ttonimaruPublicRegistrySeedCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
