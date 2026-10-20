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

const API_ROWS = [
  {
    id: "publication_read",
    title: "Immutable publication",
    endpoint: "GET /api/v1/publications/{publication_id}",
    contract_kind: "revision_pinned_publication_read",
  },
  {
    id: "manifest_read",
    title: "Publication manifest",
    endpoint: "GET /api/v1/publications/{publication_id}/manifest",
    contract_kind: "manifest_v1_read",
  },
  {
    id: "package_metadata",
    title: "Package metadata",
    endpoint: "GET /api/v1/registry/packages/{scope}/{name}/{version}",
    contract_kind: "registry_metadata_read",
  },
  {
    id: "alias_redirect",
    title: "Alias redirect",
    endpoint: "GET /u/{owner}/{slug}",
    contract_kind: "redirect_only_alias",
  },
];

const DEFAULT_ARTIFACTS = [
  { name: "publication.api.contract.detjson", kind: "api_contract", bytes: 724 },
  { name: "publication.read.fixture.detjson", kind: "read_fixture", bytes: 618 },
  { name: "publication.manifest.v1.detjson", kind: "manifest_v1", bytes: 552 },
  { name: "registry.metadata.read.detjson", kind: "metadata_read", bytes: 486 },
];

export const DEFAULT_TTONIMARU_PUBLICATION_READ_API_ROWS = API_ROWS.map((row) => ({
  id: row.id,
  endpoint: row.endpoint,
  contract_kind: row.contract_kind,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_TTONIMARU_PUBLICATION_READ_API_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return API_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      endpoint: asText(source.endpoint, row.endpoint),
      contract_kind: asText(source.contract_kind, row.contract_kind),
      ready: source.ready !== false,
      read_only: true,
      immutable_publication: row.id === "publication_read",
      revision_pin_required: row.id === "publication_read" || row.id === "manifest_read",
      mutation_claim: false,
      install_update_remove_claim: false,
      trust_signing_claim: false,
      team_membership_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `publication_read_api_${index + 1}.detjson`),
      kind: asText(source.kind, "api_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildApiText(artifacts) {
  return [
    "ttonimaru_publication_read_api:ttonimaru_registry_2_v1",
    "coordinate:카-2",
    "public_registry_final:false",
    "mutation_api:false",
    "trust_signing:false",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildTtonimaruPublicationReadApi({
  rows = DEFAULT_TTONIMARU_PUBLICATION_READ_API_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeEndpointId = "publication_read",
} = {}) {
  const apiRows = normalizeRows(rows);
  const apiArtifacts = normalizeArtifacts(artifacts);
  const active = apiRows.some((row) => row.id === activeEndpointId)
    ? activeEndpointId
    : apiRows[0]?.id ?? "";
  const readyRowCount = apiRows.filter((row) => row.ready).length;
  const readyArtifactCount = apiArtifacts.filter((artifact) => artifact.ready).length;
  const artifactKinds = new Set(apiArtifacts.map((artifact) => artifact.kind));
  const hasApiContract = artifactKinds.has("api_contract");
  const hasReadFixture = artifactKinds.has("read_fixture");
  const hasManifest = artifactKinds.has("manifest_v1");
  const hasMetadata = artifactKinds.has("metadata_read");
  const apiReady = (
    readyRowCount === apiRows.length &&
    readyArtifactCount === apiArtifacts.length &&
    hasApiContract &&
    hasReadFixture &&
    hasManifest &&
    hasMetadata
  );
  return {
    __종류: "ttonimaru_publication_read_api",
    schema: "ddn.ttonimaru.publication_read_api.v1",
    work_item: "KA2_PUBLICATION_READ_API_CLOSURE_V1",
    primary_coordinate: "카-2",
    depends_on_coordinate: ["카-1"],
    pack: "ttonimaru_registry_2_v1",
    status: apiReady ? "ttonimaru_publication_read_api_ready" : "ttonimaru_publication_read_api_incomplete",
    matrix_closure_tier: apiReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    publication_read_api_claim: apiReady,
    immutable_publication_claim: hasReadFixture,
    read_only_api_claim: apiRows.every((row) => row.read_only === true),
    revision_pin_claim: hasManifest,
    package_metadata_read_claim: hasMetadata,
    public_registry_final_claim: false,
    mutation_api_claim: false,
    registry_publish_claim: false,
    install_update_remove_claim: false,
    trust_signing_claim: false,
    team_membership_claim: false,
    cloud_sync_claim: false,
    endpoint_count: apiRows.length,
    artifact_count: apiArtifacts.length,
    artifact_size_bytes: apiArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    current_stage_closed: apiReady ? 5 : Math.min(4, readyRowCount),
    current_stage_total: 5,
    current_stage_percent: apiReady ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
    progress: {
      current_stage_closed: apiReady ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: apiReady ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 18,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 20,
      roadmap_v2_pack_evidence_reference_closed: 38,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 42,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    endpoints: apiRows,
    artifacts: apiArtifacts,
    api_text: buildApiText(apiArtifacts),
    active_endpoint_id: active,
    next_item: "KA3_PROJECT_SHARE_UI_V1",
  };
}

export function formatTtonimaruPublicationReadApiText(publicationReadApi = {}) {
  const payload = asObject(publicationReadApi);
  if (payload.schema !== "ddn.ttonimaru.publication_read_api.v1") {
    throw new Error("ttonimaru_expected_publication_read_api");
  }
  const endpoints = asArray(payload.endpoints);
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
    `publication_read_api_claim\t${payload.publication_read_api_claim === true ? "true" : "false"}`,
    `immutable_publication_claim\t${payload.immutable_publication_claim === true ? "true" : "false"}`,
    `read_only_api_claim\t${payload.read_only_api_claim === true ? "true" : "false"}`,
    `revision_pin_claim\t${payload.revision_pin_claim === true ? "true" : "false"}`,
    `package_metadata_read_claim\t${payload.package_metadata_read_claim === true ? "true" : "false"}`,
    `public_registry_final_claim\t${payload.public_registry_final_claim === true ? "true" : "false"}`,
    `mutation_api_claim\t${payload.mutation_api_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `install_update_remove_claim\t${payload.install_update_remove_claim === true ? "true" : "false"}`,
    `trust_signing_claim\t${payload.trust_signing_claim === true ? "true" : "false"}`,
    `team_membership_claim\t${payload.team_membership_claim === true ? "true" : "false"}`,
    "",
    "endpoint_id\tendpoint\tread_only\tcontract_kind",
    ...endpoints.map((endpoint) => [
      endpoint.id,
      endpoint.endpoint,
      endpoint.read_only === true ? "true" : "false",
      endpoint.contract_kind,
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

export function renderTtonimaruPublicationReadApi(root, publicationReadApi = {}) {
  if (!root) return null;
  const payload = asObject(publicationReadApi);
  const endpoints = asArray(payload.endpoints);
  const activeId = asText(payload.active_endpoint_id, endpoints[0]?.id ?? "");
  const active = endpoints.find((row) => row.id === activeId) || endpoints[0] || {};
  const artifacts = asArray(payload.artifacts);
  root.dataset.ttonimaruPublicationReadApiStatus = asText(
    payload.status,
    "ttonimaru_publication_read_api_incomplete",
  );
  root.innerHTML = `
    <div class="ttonimaru-publication-head">
      <div>
        <div class="ttonimaru-publication-kicker">Publication/read API</div>
        <h2>Ttonimaru read API</h2>
      </div>
      <div class="ttonimaru-publication-progress" data-ttonimaru-publication-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="ttonimaru-publication-summary" data-ttonimaru-publication-summary>
      Immutable publication read, manifest read, package metadata read, redirect-only alias를 묶고 mutation, trust signing, install/update/remove는 후속으로 둡니다.
    </div>
    <div class="ttonimaru-publication-body">
      <div class="ttonimaru-publication-list" data-ttonimaru-publication-list>
        ${endpoints.map((row) => `
          <button
            type="button"
            class="ttonimaru-publication-btn${row.id === activeId ? " active" : ""}"
            data-ttonimaru-publication-endpoint="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.endpoint)}</small>
          </button>
        `).join("")}
      </div>
      <div class="ttonimaru-publication-detail" data-ttonimaru-publication-detail>
        <div class="ttonimaru-publication-title" data-ttonimaru-publication-active-title>${escapeHtml(active.title)}</div>
        <p data-ttonimaru-publication-active-endpoint>${escapeHtml(active.endpoint)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="ttonimaru-publication-artifacts" data-ttonimaru-publication-artifacts>
          ${artifacts.map((artifact) => `
            <span data-ttonimaru-publication-artifact="${escapeHtml(artifact.name)}">
              ${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b
            </span>
          `).join("")}
        </div>
        <pre class="ttonimaru-publication-preview" data-ttonimaru-publication-preview>${escapeHtml(payload.api_text || "")}</pre>
        <button type="button" class="ghost" data-ttonimaru-publication-copy>read API 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (endpointId) => {
    renderTtonimaruPublicationReadApi(root, {
      ...payload,
      active_endpoint_id: endpointId,
    });
  };
  root.querySelectorAll("[data-ttonimaru-publication-endpoint]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-ttonimaru-publication-endpoint") || "");
    });
  });
  root.querySelector("[data-ttonimaru-publication-copy]")?.addEventListener("click", async () => {
    const text = formatTtonimaruPublicationReadApiText(payload);
    root.dataset.ttonimaruPublicationReadApiCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
