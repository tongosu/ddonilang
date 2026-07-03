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

function asBool(value) {
  return value === true || value === "true" || value === "참";
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

const REHEARSAL_DEFS = [
  {
    id: "approval_recheck_anchor",
    source_anchor: "pack/studio_public_release_approval_recheck_v1/public_release_approval_recheck.detjson",
    rehearsal_lane: "approval_recheck",
    title: "approval recheck",
    summary: "정확한 승인 문구가 없으므로 공개 릴리스 실행 상태로 넘어가지 않습니다.",
  },
  {
    id: "pre_execution_dry_run_anchor",
    source_anchor: "pack/studio_release_pre_execution_dry_run_v1/dry_run.detjson",
    rehearsal_lane: "pre_execution_dry_run",
    title: "pre-execution dry run",
    summary: "release preflight는 dry-run only로 유지되고 실제 archive나 upload를 만들지 않습니다.",
  },
  {
    id: "asset_plan_anchor",
    source_anchor: "pack/studio_public_release_asset_plan_v1/release_assets.detjson",
    rehearsal_lane: "asset_plan",
    title: "asset plan",
    summary: "4개 planned asset은 local rehearsal에서 generated_now=false 상태를 유지합니다.",
  },
  {
    id: "approval_continuity_anchor",
    source_anchor: "pack/studio_release_approval_packet_continuity_v1/continuity.detjson",
    rehearsal_lane: "approval_continuity",
    title: "approval continuity",
    summary: "다음 상태는 계속 AWAIT_EXPLICIT_RELEASE_APPROVAL이며 release execution claim은 없습니다.",
  },
  {
    id: "publication_artifact_handoff",
    source_anchor: "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
    rehearsal_lane: "next_handoff",
    title: "publication artifact",
    summary: "다음 항목은 실제 배포 산출물이 아니라 publication artifact dry-run manifest입니다.",
  },
];

export const DEFAULT_LOCAL_RELEASE_REHEARSAL_ROWS = REHEARSAL_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  rehearsal_lane: row.rehearsal_lane,
  local_rehearsal_only: true,
  dry_run_only: true,
  generated_now: false,
  product_ui_change: true,
  release_approval_claim: false,
  release_execution_claim: false,
}));

function normalizeRows(rows = DEFAULT_LOCAL_RELEASE_REHEARSAL_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return REHEARSAL_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      rehearsal_lane: asText(source.rehearsal_lane, def.rehearsal_lane),
      rehearsal_surface: "local_studio_release_rehearsal_check",
      local_rehearsal_only: true,
      dry_run_only: true,
      generated_now: asBool(source.generated_now),
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      github_release_claim: false,
      archive_generation_claim: false,
      publication_checksum_generation_claim: false,
      artifact_signing_claim: false,
      install_enablement_claim: false,
      publication_snapshot_emit_claim: false,
      cloud_sync_claim: false,
      runtime_claim: false,
      replay_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      parser_frontdoor_change: false,
      solver_implementation_change: false,
      benchmark_execution_claim: false,
      lts_certification_claim: false,
      product_ui_change: true,
    };
  });
}

export function buildLocalReleaseRehearsalCheck({
  rehearsalRows = DEFAULT_LOCAL_RELEASE_REHEARSAL_ROWS,
  activeRehearsalId = "asset_plan_anchor",
} = {}) {
  const rows = normalizeRows(rehearsalRows);
  const active = rows.some((row) => row.id === activeRehearsalId)
    ? activeRehearsalId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.local_rehearsal_only === true &&
    row.dry_run_only === true &&
    row.generated_now === false &&
    row.release_approval_claim === false &&
    row.release_execution_claim === false &&
    row.public_release_claim === false &&
    row.archive_generation_claim === false &&
    row.public_upload_claim === false &&
    row.runtime_claim === false
  ));
  const plannedAssetCount = 4;
  const stages = [
    ["rehearsal_row_alignment", rows.length === REHEARSAL_DEFS.length],
    ["approval_recheck_linked", rows.some((row) => row.id === "approval_recheck_anchor")],
    ["dry_run_linked", rows.some((row) => row.id === "pre_execution_dry_run_anchor")],
    ["asset_plan_linked", rows.some((row) => row.id === "asset_plan_anchor")],
    ["approval_continuity_linked", rows.some((row) => row.id === "approval_continuity_anchor")],
    ["release_boundary_blocked", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_local_release_rehearsal_check",
    schema: "ddn.studio.local_release_rehearsal_check.v1",
    work_item: "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
    based_on: "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
    workflow_claim: "local_release_rehearsal_check",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    next_state: "AWAIT_EXPLICIT_RELEASE_APPROVAL",
    product_ui_change: true,
    product_code_change: true,
    local_rehearsal_check_claim: true,
    local_rehearsal_only: true,
    dry_run_only: true,
    runtime_claim: false,
    replay_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    archive_generation_claim: false,
    publication_checksum_generation_claim: false,
    artifact_signing_claim: false,
    install_enablement_claim: false,
    publication_snapshot_emit_claim: false,
    cloud_sync_claim: false,
    benchmark_execution_claim: false,
    lts_certification_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    solver_implementation_change: false,
    planned_asset_count: plannedAssetCount,
    all_planned_assets_generated_now: false,
    status: ready_stage_count === stages.length ? "local_release_rehearsal_ready" : "local_release_rehearsal_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    rehearsal_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_rehearsal_id: active,
    rehearsal_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 8,
      super_long_total: 18,
      super_long_percent: 44,
      studio_local_super_long_closed: 8,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 44,
      current_stage_closed: 4,
      current_stage_total: 4,
      current_stage_percent: 100,
      roadmap_v2_behavior_closed: 6,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 7,
      roadmap_v2_matrix_behavior_closed: 6,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 7,
      roadmap_v2_pack_evidence_reference_closed: 25,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 28,
    },
    next_item: "MA5_SEAMGRIM_CURRICULUM_5_LTS_PACK_CLOSURE_V1",
  };
}

export function formatLocalReleaseRehearsalCheckText(rehearsal = {}) {
  const payload = asObject(rehearsal);
  if (payload.schema !== "ddn.studio.local_release_rehearsal_check.v1") {
    throw new Error("seamgrim_expected_local_release_rehearsal_check");
  }
  const rows = asArray(payload.rehearsal_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `next_state\t${payload.next_state ?? ""}`,
    `dry_run_only\t${payload.dry_run_only === true ? "true" : "false"}`,
    `planned_asset_count\t${payload.planned_asset_count ?? ""}`,
    `all_planned_assets_generated_now\t${payload.all_planned_assets_generated_now === true ? "true" : "false"}`,
    `release_approval_claim\t${payload.release_approval_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `followup\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `followup_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `rehearsal_row_count\t${payload.rehearsal_row_count ?? rows.length}`,
    "",
    "rehearsal_id\tlane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.rehearsal_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderLocalReleaseRehearsalCheck(root, rehearsal = {}) {
  if (!root) return null;
  const payload = asObject(rehearsal);
  const rows = asArray(payload.rehearsal_rows);
  const activeId = asText(payload.active_rehearsal_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.localReleaseRehearsalCheckStatus = asText(payload.status, "local_release_rehearsal_incomplete");
  root.innerHTML = `
    <div class="rehearsal-check-head">
      <div>
        <div class="rehearsal-check-kicker">Local rehearsal</div>
        <h2>Release rehearsal check</h2>
      </div>
      <div class="rehearsal-check-progress" data-rehearsal-check-progress>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} follow-up</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(payload.dry_run_only === true ? "dry-run only" : "review needed")}</span>
      </div>
    </div>
    <div class="rehearsal-check-assets" data-rehearsal-check-assets>
      ${escapeHtml(String(payload.planned_asset_count ?? 0))} assets · generated_now=false · ${escapeHtml(payload.next_state || "")}
    </div>
    <div class="rehearsal-check-body">
      <div class="rehearsal-check-list" data-rehearsal-check-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="rehearsal-check-btn${row.id === activeId ? " active" : ""}"
            data-rehearsal-check="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.rehearsal_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="rehearsal-check-detail" data-rehearsal-check-detail>
        <div class="rehearsal-check-title" data-rehearsal-check-active-title>${escapeHtml(active.title)}</div>
        <p data-rehearsal-check-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-rehearsal-check-active-lane>${escapeHtml(active.rehearsal_lane)}</dd></div>
          <div><dt>source</dt><dd data-rehearsal-check-active-source>${escapeHtml(active.source_anchor)}</dd></div>
          <div><dt>boundary</dt><dd>local dry-run only</dd></div>
        </dl>
        <button type="button" class="ghost" data-rehearsal-check-copy>rehearsal 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderLocalReleaseRehearsalCheck(root, {
      ...payload,
      active_rehearsal_id: rowId,
    });
  };
  root.querySelectorAll("[data-rehearsal-check]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-rehearsal-check") || "");
    });
  });
  root.querySelector("[data-rehearsal-check-copy]")?.addEventListener("click", async () => {
    const text = formatLocalReleaseRehearsalCheckText(payload);
    root.dataset.localReleaseRehearsalCheckCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
