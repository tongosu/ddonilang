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

const SHARE_ROWS = [
  {
    id: "project_snapshot",
    title: "Project snapshot",
    share_kind: "local_project_snapshot",
    local_uri: "ttonimaru://project/share/local/project-snapshot",
  },
  {
    id: "revision_pin",
    title: "Revision pin",
    share_kind: "revision_pinned_share",
    local_uri: "ttonimaru://project/share/local/revision-pin",
  },
  {
    id: "share_link",
    title: "Share link",
    share_kind: "copyable_local_share_link",
    local_uri: "ttonimaru://project/share/local/link",
  },
  {
    id: "remix_handoff",
    title: "Remix handoff",
    share_kind: "local_remix_handoff",
    local_uri: "ttonimaru://project/share/local/remix",
  },
];

const DEFAULT_SHARE_ARTIFACTS = [
  { name: "project.snapshot.detjson", kind: "project_snapshot", bytes: 702 },
  { name: "revision.pin.detjson", kind: "revision_pin", bytes: 512 },
  { name: "share.link.detjson", kind: "share_link", bytes: 426 },
  { name: "remix.handoff.detjson", kind: "remix_handoff", bytes: 468 },
];

export const DEFAULT_TTONIMARU_PROJECT_SHARE_ROWS = SHARE_ROWS.map((row) => ({
  id: row.id,
  share_kind: row.share_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_TTONIMARU_PROJECT_SHARE_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return SHARE_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      share_kind: asText(source.share_kind, row.share_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_only: true,
      revision_pinned: row.id === "revision_pin" || row.id === "share_link",
      product_ui_behavior: true,
      public_registry_seed_claim: false,
      registry_publish_claim: false,
      cloud_sync_claim: false,
      account_permission_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_SHARE_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_SHARE_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `project_share_${index + 1}.detjson`),
      kind: asText(source.kind, "share_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildShareText(artifacts) {
  return [
    "ttonimaru_project_share_ui:ttonimaru_registry_3_v1",
    "coordinate:카-3",
    "public_registry_seed:false",
    "registry_publish:false",
    "cloud_sync:false",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildTtonimaruProjectShareUi({
  rows = DEFAULT_TTONIMARU_PROJECT_SHARE_ROWS,
  artifacts = DEFAULT_SHARE_ARTIFACTS,
  activeShareId = "project_snapshot",
} = {}) {
  const shareRows = normalizeRows(rows);
  const shareArtifacts = normalizeArtifacts(artifacts);
  const active = shareRows.some((row) => row.id === activeShareId)
    ? activeShareId
    : shareRows[0]?.id ?? "";
  const readyRowCount = shareRows.filter((row) => row.ready).length;
  const readyArtifactCount = shareArtifacts.filter((artifact) => artifact.ready).length;
  const artifactKinds = new Set(shareArtifacts.map((artifact) => artifact.kind));
  const hasSnapshot = artifactKinds.has("project_snapshot");
  const hasRevisionPin = artifactKinds.has("revision_pin");
  const hasShareLink = artifactKinds.has("share_link");
  const hasRemix = artifactKinds.has("remix_handoff");
  const shareReady = (
    readyRowCount === shareRows.length &&
    readyArtifactCount === shareArtifacts.length &&
    hasSnapshot &&
    hasRevisionPin &&
    hasShareLink &&
    hasRemix
  );
  return {
    __종류: "ttonimaru_project_share_ui",
    schema: "ddn.ttonimaru.project_share_ui.v1",
    work_item: "KA3_PROJECT_SHARE_UI_V1",
    primary_coordinate: "카-3",
    depends_on_coordinate: ["카-2"],
    pack: "ttonimaru_registry_3_v1",
    status: shareReady ? "ttonimaru_project_share_ui_ready" : "ttonimaru_project_share_ui_incomplete",
    matrix_closure_tier: shareReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    project_share_ui_claim: shareReady,
    project_snapshot_claim: hasSnapshot,
    revision_pin_claim: hasRevisionPin,
    share_link_claim: hasShareLink,
    remix_handoff_claim: hasRemix,
    public_registry_seed_claim: false,
    public_registry_final_claim: false,
    registry_publish_claim: false,
    install_update_remove_claim: false,
    trust_signing_claim: false,
    team_membership_claim: false,
    account_permission_claim: false,
    cloud_sync_claim: false,
    share_row_count: shareRows.length,
    artifact_count: shareArtifacts.length,
    artifact_size_bytes: shareArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    current_stage_closed: shareReady ? 5 : Math.min(4, readyRowCount),
    current_stage_total: 5,
    current_stage_percent: shareReady ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
    progress: {
      current_stage_closed: shareReady ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: shareReady ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 19,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 21,
      roadmap_v2_pack_evidence_reference_closed: 39,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 43,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    share_rows: shareRows,
    artifacts: shareArtifacts,
    share_text: buildShareText(shareArtifacts),
    active_share_id: active,
    next_item: "KA4_PUBLIC_REGISTRY_SEED_V1",
  };
}

export function formatTtonimaruProjectShareUiText(projectShare = {}) {
  const payload = asObject(projectShare);
  if (payload.schema !== "ddn.ttonimaru.project_share_ui.v1") {
    throw new Error("ttonimaru_expected_project_share_ui");
  }
  const rows = asArray(payload.share_rows);
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
    `project_share_ui_claim\t${payload.project_share_ui_claim === true ? "true" : "false"}`,
    `project_snapshot_claim\t${payload.project_snapshot_claim === true ? "true" : "false"}`,
    `revision_pin_claim\t${payload.revision_pin_claim === true ? "true" : "false"}`,
    `share_link_claim\t${payload.share_link_claim === true ? "true" : "false"}`,
    `remix_handoff_claim\t${payload.remix_handoff_claim === true ? "true" : "false"}`,
    `public_registry_seed_claim\t${payload.public_registry_seed_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `account_permission_claim\t${payload.account_permission_claim === true ? "true" : "false"}`,
    `cloud_sync_claim\t${payload.cloud_sync_claim === true ? "true" : "false"}`,
    "",
    "share_id\tshare_kind\trevision_pinned\tlocal_uri",
    ...rows.map((row) => [
      row.id,
      row.share_kind,
      row.revision_pinned === true ? "true" : "false",
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

export function renderTtonimaruProjectShareUi(root, projectShare = {}) {
  if (!root) return null;
  const payload = asObject(projectShare);
  const rows = asArray(payload.share_rows);
  const activeId = asText(payload.active_share_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  const artifacts = asArray(payload.artifacts);
  root.dataset.ttonimaruProjectShareUiStatus = asText(
    payload.status,
    "ttonimaru_project_share_ui_incomplete",
  );
  root.innerHTML = `
    <div class="ttonimaru-share-head">
      <div>
        <div class="ttonimaru-share-kicker">Project/share UI</div>
        <h2>Ttonimaru project share</h2>
      </div>
      <div class="ttonimaru-share-progress" data-ttonimaru-share-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="ttonimaru-share-summary" data-ttonimaru-share-summary>
      Project snapshot, revision pin, local share link, remix handoff를 묶고 public registry seed, registry publish, cloud sync, account permission은 후속으로 둡니다.
    </div>
    <div class="ttonimaru-share-body">
      <div class="ttonimaru-share-list" data-ttonimaru-share-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="ttonimaru-share-btn${row.id === activeId ? " active" : ""}"
            data-ttonimaru-share="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.share_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="ttonimaru-share-detail" data-ttonimaru-share-detail>
        <div class="ttonimaru-share-title" data-ttonimaru-share-active-title>${escapeHtml(active.title)}</div>
        <p data-ttonimaru-share-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="ttonimaru-share-artifacts" data-ttonimaru-share-artifacts>
          ${artifacts.map((artifact) => `
            <span data-ttonimaru-share-artifact="${escapeHtml(artifact.name)}">
              ${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b
            </span>
          `).join("")}
        </div>
        <pre class="ttonimaru-share-preview" data-ttonimaru-share-preview>${escapeHtml(payload.share_text || "")}</pre>
        <button type="button" class="ghost" data-ttonimaru-share-copy>share UI 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (shareId) => {
    renderTtonimaruProjectShareUi(root, {
      ...payload,
      active_share_id: shareId,
    });
  };
  root.querySelectorAll("[data-ttonimaru-share]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-ttonimaru-share") || "");
    });
  });
  root.querySelector("[data-ttonimaru-share-copy]")?.addEventListener("click", async () => {
    const text = formatTtonimaruProjectShareUiText(payload);
    root.dataset.ttonimaruProjectShareUiCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
