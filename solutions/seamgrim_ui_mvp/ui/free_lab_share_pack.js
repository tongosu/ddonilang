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

const SHARE_ROWS = [
  {
    id: "snapshot",
    title: "스냅샷",
    link_kind: "local_snapshot",
    share_link: "seamgrim://free-lab/local/snapshot/baseline",
  },
  {
    id: "remix",
    title: "리믹스",
    link_kind: "local_remix",
    share_link: "seamgrim://free-lab/local/remix/high-lever",
  },
  {
    id: "handoff",
    title: "인수인계",
    link_kind: "local_handoff",
    share_link: "seamgrim://free-lab/local/handoff/report-ui-pack",
  },
];

export const DEFAULT_FREE_LAB_SHARE_ROWS = SHARE_ROWS.map((row) => ({
  id: row.id,
  link_kind: row.link_kind,
  share_link: row.share_link,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_FREE_LAB_SHARE_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return SHARE_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      link_kind: asText(source.link_kind, row.link_kind),
      share_link: asText(source.share_link, row.share_link),
      ready: source.ready !== false,
      local_only: true,
      remix_enabled: row.id === "remix",
      product_ui_behavior: true,
      public_upload_claim: false,
      registry_publish_claim: false,
      cloud_sync_claim: false,
    };
  });
}

export function buildFreeLabSharePack({
  rows = DEFAULT_FREE_LAB_SHARE_ROWS,
  activeShareId = "remix",
} = {}) {
  const shares = normalizeRows(rows);
  const active = shares.some((share) => share.id === activeShareId)
    ? activeShareId
    : shares[0]?.id ?? "";
  const readyCount = shares.filter((share) => share.ready).length;
  const hasRemix = shares.some((share) => share.remix_enabled && share.share_link.includes("/remix/"));
  const hasLocalLinks = shares.every((share) => share.share_link.startsWith("seamgrim://free-lab/local/"));
  const shareReady = readyCount === shares.length && hasRemix && hasLocalLinks;
  return {
    __종류: "free_lab_share_pack",
    schema: "ddn.seamgrim.free_lab.share_pack.v1",
    work_item: "BA4_FREE_LAB_SHARE_PACK_CLOSURE_V1",
    primary_coordinate: "바-4",
    depends_on_coordinate: ["바-3"],
    pack: "free_lab_4_v1",
    status: shareReady ? "free_lab_share_ready" : "free_lab_share_incomplete",
    matrix_closure_tier: shareReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    local_share_claim: shareReady,
    remix_link_claim: hasRemix,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    cloud_sync_claim: false,
    research_mode_claim: false,
    share_count: shares.length,
    ready_share_count: readyCount,
    current_stage_closed: shareReady ? 5 : readyCount,
    current_stage_total: 5,
    current_stage_percent: shareReady ? 100 : Math.round((readyCount / 5) * 100),
    progress: {
      current_stage_closed: shareReady ? 5 : readyCount,
      current_stage_total: 5,
      current_stage_percent: shareReady ? 100 : Math.round((readyCount / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 11,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 12,
      roadmap_v2_pack_evidence_reference_closed: 30,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 33,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    shares,
    active_share_id: active,
    next_item: "BA5_FREE_LAB_RESEARCH_WORKFLOW_CLOSURE_V1",
  };
}

export function formatFreeLabSharePackText(pack = {}) {
  const payload = asObject(pack);
  if (payload.schema !== "ddn.seamgrim.free_lab.share_pack.v1") {
    throw new Error("seamgrim_expected_free_lab_share_pack");
  }
  const shares = asArray(payload.shares);
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
    `local_share_claim\t${payload.local_share_claim === true ? "true" : "false"}`,
    `remix_link_claim\t${payload.remix_link_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    "",
    "share_id\tlink_kind\tlocal_only\tshare_link",
    ...shares.map((share) => [
      share.id,
      share.link_kind,
      share.local_only === true ? "true" : "false",
      share.share_link,
    ].join("\t")),
  ].join("\n");
}

export function renderFreeLabSharePack(root, pack = {}) {
  if (!root) return null;
  const payload = asObject(pack);
  const shares = asArray(payload.shares);
  const activeId = asText(payload.active_share_id, shares[0]?.id ?? "");
  const active = shares.find((share) => share.id === activeId) || shares[0] || {};
  root.dataset.freeLabSharePackStatus = asText(payload.status, "free_lab_share_incomplete");
  root.innerHTML = `
    <div class="free-lab-share-head">
      <div>
        <div class="free-lab-share-kicker">Share pack</div>
        <h2>자유 실험 공유</h2>
      </div>
      <div class="free-lab-share-progress" data-free-lab-share-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="free-lab-share-summary" data-free-lab-share-summary>
      local remix/share link를 만들고 복사하지만, 공개 업로드와 registry publish는 후속으로 유지합니다.
    </div>
    <div class="free-lab-share-body">
      <div class="free-lab-share-list" data-free-lab-share-list>
        ${shares.map((share) => `
          <button
            type="button"
            class="free-lab-share-btn${share.id === activeId ? " active" : ""}"
            data-free-lab-share="${escapeHtml(share.id)}"
          >
            <span>${escapeHtml(share.title)}</span>
            <small>${escapeHtml(share.link_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="free-lab-share-detail" data-free-lab-share-detail>
        <div class="free-lab-share-title" data-free-lab-share-active-title>${escapeHtml(active.title)}</div>
        <p data-free-lab-share-active-link>${escapeHtml(active.share_link)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>boundary</dt><dd>local share only</dd></div>
        </dl>
        <button type="button" class="ghost" data-free-lab-share-copy>share pack 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (shareId) => {
    renderFreeLabSharePack(root, {
      ...payload,
      active_share_id: shareId,
    });
  };
  root.querySelectorAll("[data-free-lab-share]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-free-lab-share") || "");
    });
  });
  root.querySelector("[data-free-lab-share-copy]")?.addEventListener("click", async () => {
    const text = formatFreeLabSharePackText(payload);
    root.dataset.freeLabSharePackCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
