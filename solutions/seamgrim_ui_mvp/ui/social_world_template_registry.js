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

const TEMPLATE_ROWS = [
  { id: "template_catalog", title: "Template catalog", template_kind: "local_template_catalog", local_uri: "social-world://template-registry/catalog" },
  { id: "share_snapshot", title: "Share snapshot", template_kind: "share_snapshot_packet", local_uri: "social-world://template-registry/share" },
  { id: "remix_contract", title: "Remix contract", template_kind: "template_remix_contract", local_uri: "social-world://template-registry/remix" },
  { id: "classroom_registry", title: "Classroom registry", template_kind: "classroom_registry_preview", local_uri: "social-world://template-registry/classroom" },
];

const DEFAULT_ARTIFACTS = [
  { name: "social.template.catalog.detjson", kind: "template_catalog", bytes: 792 },
  { name: "social.template.share.snapshot.detjson", kind: "share_snapshot", bytes: 688 },
  { name: "social.template.remix.contract.detjson", kind: "remix_contract", bytes: 744 },
  { name: "social.template.classroom.registry.detjson", kind: "classroom_registry", bytes: 636 },
];

export const DEFAULT_SOCIAL_WORLD_TEMPLATE_REGISTRY_ROWS = TEMPLATE_ROWS.map((row) => ({
  id: row.id,
  template_kind: row.template_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_SOCIAL_WORLD_TEMPLATE_REGISTRY_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return TEMPLATE_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      template_kind: asText(source.template_kind, row.template_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_registry_claim: true,
      public_publish_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `social_template_${index + 1}.detjson`),
      kind: asText(source.kind, "social_template_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildTemplateText(rows, artifacts) {
  return [
    "social_world_template_registry:social_world_econ_4_v1",
    "coordinate:파-4",
    "public_template_publish:false",
    "network_registry_sync:false",
    "account_permission_change:false",
    "policy_advice:false",
    "state_hash_participation:false",
    ...rows.map((row) => `${row.id}\t${row.template_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildSocialWorldTemplateRegistry({
  rows = DEFAULT_SOCIAL_WORLD_TEMPLATE_REGISTRY_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "template_catalog",
} = {}) {
  const templateRows = normalizeRows(rows);
  const templateArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(templateArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = templateRows.filter((row) => row.ready).length;
  const readyArtifactCount = templateArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["template_catalog", "share_snapshot", "remix_contract", "classroom_registry"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === templateRows.length &&
    readyArtifactCount === templateArtifacts.length &&
    hasAllArtifacts;
  const active = templateRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : templateRows[0]?.id ?? "";
  return {
    __종류: "social_world_template_registry",
    schema: "ddn.social_world.template_registry.v1",
    work_item: "PA4_SOCIAL_TEMPLATE_REGISTRY_V1",
    primary_coordinate: "파-4",
    depends_on_coordinate: ["파-3", "파-2", "타-2"],
    pack: "social_world_econ_4_v1",
    status: ready ? "social_world_template_registry_ready" : "social_world_template_registry_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    social_template_registry_claim: ready,
    template_catalog_claim: artifactKinds.has("template_catalog"),
    share_snapshot_claim: artifactKinds.has("share_snapshot"),
    remix_contract_claim: artifactKinds.has("remix_contract"),
    classroom_registry_claim: artifactKinds.has("classroom_registry"),
    public_template_publish_claim: false,
    network_registry_sync_claim: false,
    account_permission_change_claim: false,
    policy_advice_claim: false,
    state_hash_participation_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: templateRows,
    artifacts: templateArtifacts,
    active_row_id: active,
    template_text: buildTemplateText(templateRows, templateArtifacts),
    artifact_size_bytes: templateArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 27,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 30,
      roadmap_v2_pack_evidence_reference_closed: 47,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 52,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "PA5_SOCIAL_WORLD_LTS_V1",
  };
}

export function formatSocialWorldTemplateRegistryText(templateRegistry = {}) {
  const payload = asObject(templateRegistry);
  if (payload.schema !== "ddn.social_world.template_registry.v1") {
    throw new Error("social_world_expected_template_registry");
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
    `social_template_registry_claim\t${payload.social_template_registry_claim === true ? "true" : "false"}`,
    `template_catalog_claim\t${payload.template_catalog_claim === true ? "true" : "false"}`,
    `share_snapshot_claim\t${payload.share_snapshot_claim === true ? "true" : "false"}`,
    `remix_contract_claim\t${payload.remix_contract_claim === true ? "true" : "false"}`,
    `classroom_registry_claim\t${payload.classroom_registry_claim === true ? "true" : "false"}`,
    `public_template_publish_claim\t${payload.public_template_publish_claim === true ? "true" : "false"}`,
    `network_registry_sync_claim\t${payload.network_registry_sync_claim === true ? "true" : "false"}`,
    `account_permission_change_claim\t${payload.account_permission_change_claim === true ? "true" : "false"}`,
    `policy_advice_claim\t${payload.policy_advice_claim === true ? "true" : "false"}`,
    "",
    "row_id\ttemplate_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.template_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderSocialWorldTemplateRegistry(root, templateRegistry = {}) {
  if (!root) return null;
  const payload = asObject(templateRegistry);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.socialWorldTemplateRegistryStatus = asText(payload.status, "social_world_template_registry_incomplete");
  root.innerHTML = `
    <div class="social-template-head">
      <div>
        <div class="social-template-kicker">Template Registry</div>
        <h2>Social template registry</h2>
      </div>
      <div class="social-template-progress" data-social-template-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="social-template-summary" data-social-template-summary>
      template catalog, share snapshot, remix contract, classroom registry preview를 local UI로 고정하고 public template publish, network registry sync, account permission change는 후속으로 둡니다.
    </div>
    <div class="social-template-body">
      <div class="social-template-list">
        ${rows.map((row) => `
          <button type="button" class="social-template-btn${row.id === activeId ? " active" : ""}" data-social-template-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.template_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="social-template-detail">
        <div class="social-template-title" data-social-template-active-title>${escapeHtml(active.title)}</div>
        <p data-social-template-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="social-template-artifacts">
          ${artifacts.map((artifact) => `
            <span data-social-template-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="social-template-preview" data-social-template-preview>${escapeHtml(payload.template_text ?? "")}</pre>
        <button type="button" class="ghost" data-social-template-copy>template registry 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderSocialWorldTemplateRegistry(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-social-template-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-social-template-row") || ""));
  });
  root.querySelector("[data-social-template-copy]")?.addEventListener("click", async () => {
    root.dataset.socialWorldTemplateRegistryCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatSocialWorldTemplateRegistryText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
