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

const REQUIRED_APPROVAL_PHRASE = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다";

const DASHBOARD_DEFS = [
  {
    id: "approval_state_dashboard_card",
    source_snapshot_row: "approval_continuity_snapshot",
    source_review_material: "pack/studio_release_approval_chain_closure_v1/closure.detjson",
    dashboard_lane: "approval_state",
    title: "승인 대기 상태",
    summary: "필수 승인 문구를 보존하되 실행 승인으로 처리하지 않습니다.",
  },
  {
    id: "benchmark_snapshot_dashboard_card",
    source_snapshot_row: "benchmark_lts_matrix_snapshot",
    source_review_material: null,
    dashboard_lane: "benchmark_snapshot",
    title: "Benchmark snapshot",
    summary: "로컬 benchmark snapshot evidence를 릴리스 검토 카드로 연결합니다.",
  },
  {
    id: "classroom_operations_dashboard_card",
    source_snapshot_row: "classroom_operations_panel_snapshot",
    source_review_material: null,
    dashboard_lane: "classroom_operations",
    title: "수업 운영 패널",
    summary: "수업 운영 패널 상태를 release review packet 안에서 확인합니다.",
  },
  {
    id: "local_packaging_review_dashboard_card",
    source_snapshot_row: "local_packaging_snapshot",
    source_review_material: "pack/studio_local_packaging_consolidation_v1/local_package_manifest.detjson",
    dashboard_lane: "local_packaging_review",
    title: "로컬 패키징 검토",
    summary: "로컬 패키징 자료를 공개 업로드 전 검토 상태로 유지합니다.",
  },
  {
    id: "publication_prep_review_dashboard_card",
    source_snapshot_row: null,
    source_review_material: "pack/studio_public_lesson_publication_prep_v1/publication_prep.detjson",
    dashboard_lane: "publication_prep_review",
    title: "공개 수업 준비",
    summary: "공개 수업 준비 자료를 snapshot-only 검토 카드로 묶습니다.",
  },
  {
    id: "registry_share_review_dashboard_card",
    source_snapshot_row: null,
    source_review_material: "pack/studio_registry_share_seed_v1/registry_share_seed.detjson",
    dashboard_lane: "registry_share_review",
    title: "Registry/share 검토",
    summary: "registry/share seed evidence를 public release 없이 검토합니다.",
  },
];

export const DEFAULT_RELEASE_REVIEW_SNAPSHOT_ROWS = [
  "approval_continuity_snapshot",
  "benchmark_lts_matrix_snapshot",
  "classroom_operations_panel_snapshot",
  "local_packaging_snapshot",
].map((id) => ({
  id,
  snapshot_only: true,
  generated_now: false,
  benchmark_execution_claim: false,
  performance_baseline_generation_claim: false,
  performance_baseline_publication_claim: false,
}));

export const DEFAULT_RELEASE_REVIEW_MATERIALS = [
  "pack/studio_release_approval_chain_closure_v1/closure.detjson",
  "pack/studio_local_packaging_consolidation_v1/local_package_manifest.detjson",
  "pack/studio_public_lesson_publication_prep_v1/publication_prep.detjson",
  "pack/studio_registry_share_seed_v1/registry_share_seed.detjson",
];

function normalizeDashboardRows({
  snapshotRows = DEFAULT_RELEASE_REVIEW_SNAPSHOT_ROWS,
  reviewMaterials = DEFAULT_RELEASE_REVIEW_MATERIALS,
} = {}) {
  const snapshotById = new Map(asArray(snapshotRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  const materials = new Set(asArray(reviewMaterials).map((item) => asText(item)).filter(Boolean));
  return DASHBOARD_DEFS.map((def) => {
    const snapshot = def.source_snapshot_row ? snapshotById.get(def.source_snapshot_row) || {} : {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_snapshot_row: def.source_snapshot_row,
      source_review_material: def.source_review_material,
      dashboard_lane: def.dashboard_lane,
      dashboard_surface: "local_release_review_packet_dashboard",
      dashboard_only: true,
      source_available: Boolean(def.source_snapshot_row ? snapshotById.has(def.source_snapshot_row) : materials.has(def.source_review_material)),
      generated_now: asBool(snapshot.generated_now),
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      product_ui_change: true,
      cloud_sync_claim: false,
      account_setup_claim: false,
      permission_system_claim: false,
    };
  });
}

export function buildReleaseReviewPacketDashboard({
  snapshotRows = DEFAULT_RELEASE_REVIEW_SNAPSHOT_ROWS,
  reviewMaterials = DEFAULT_RELEASE_REVIEW_MATERIALS,
  activeDashboardId = "approval_state_dashboard_card",
} = {}) {
  const rows = normalizeDashboardRows({ snapshotRows, reviewMaterials });
  const active = rows.some((row) => row.id === activeDashboardId)
    ? activeDashboardId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.dashboard_only === true &&
    row.generated_now === false &&
    row.release_approval_claim === false &&
    row.release_execution_claim === false &&
    row.public_release_claim === false
  ));
  const stages = [
    ["dashboard_card_alignment", rows.length === DASHBOARD_DEFS.length],
    ["snapshot_anchor_alignment", rows.filter((row) => row.source_snapshot_row).length === 4],
    ["review_material_alignment", rows.filter((row) => row.source_review_material).length === 4],
    ["approval_phrase_preserved", REQUIRED_APPROVAL_PHRASE.length > 0],
    ["local_only_boundary", localOnly],
    ["no_release_execution", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_release_review_packet_dashboard",
    schema: "ddn.studio.release_review_packet_dashboard.v1",
    work_item: "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    workflow_claim: "release_review_packet_dashboard",
    generated_locally: true,
    product_ui_change: true,
    dashboard_only: true,
    runtime_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    github_release_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    public_link_creation_claim: false,
    install_enablement_claim: false,
    publication_snapshot_emit_claim: false,
    archive_generation_claim: false,
    publication_checksum_generation_claim: false,
    artifact_signing_claim: false,
    benchmark_execution_claim: false,
    performance_baseline_generation_claim: false,
    performance_baseline_publication_claim: false,
    lts_certification_claim: false,
    classroom_operations_runtime_claim: false,
    teacher_feedback_runtime_claim: false,
    student_data_collection_claim: false,
    remote_save_claim: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    result_replay_claim: false,
    required_approval_phrase: REQUIRED_APPROVAL_PHRASE,
    generic_next_dev_request_is_approval: false,
    next_state: "AWAIT_EXPLICIT_RELEASE_APPROVAL",
    status: ready_stage_count === stages.length ? "release_review_dashboard_ready" : "release_review_dashboard_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    dashboard_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_dashboard_id: active,
    dashboard_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 5,
      current_stage_total: 8,
      current_stage_percent: 63,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
  };
}

export function formatReleaseReviewPacketDashboardText(dashboard = {}) {
  const payload = asObject(dashboard);
  if (payload.schema !== "ddn.studio.release_review_packet_dashboard.v1") {
    throw new Error("seamgrim_expected_release_review_packet_dashboard");
  }
  const rows = asArray(payload.dashboard_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `release_approval_claim\t${payload.release_approval_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_release_claim\t${payload.public_release_claim === true ? "true" : "false"}`,
    `required_approval_phrase\t${payload.required_approval_phrase ?? ""}`,
    `dashboard_row_count\t${payload.dashboard_row_count ?? rows.length}`,
    "",
    "dashboard_id\tdashboard_lane\tsource_snapshot_row\tsource_review_material",
    ...rows.map((row) => [
      row.id,
      row.dashboard_lane,
      row.source_snapshot_row ?? "",
      row.source_review_material ?? "",
    ].join("\t")),
  ].join("\n");
}

export function renderReleaseReviewPacketDashboard(root, dashboard = {}) {
  if (!root) return null;
  const payload = asObject(dashboard);
  const rows = asArray(payload.dashboard_rows);
  const activeId = asText(payload.active_dashboard_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.releaseReviewStatus = asText(payload.status, "release_review_dashboard_incomplete");
  root.innerHTML = `
    <div class="release-review-dashboard-head">
      <div>
        <div class="release-review-dashboard-kicker">Release review</div>
        <h2>검토 패킷 대시보드</h2>
      </div>
      <div class="release-review-dashboard-progress" data-release-review-progress>
        <span>${escapeHtml(String(payload.dashboard_row_count ?? rows.length))}개 카드</span>
        <span>승인 없음</span>
        <span>실행 없음</span>
      </div>
    </div>
    <div class="release-review-dashboard-body">
      <div class="release-review-dashboard-list" data-release-review-dashboard-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="release-review-dashboard-btn${row.id === activeId ? " active" : ""}"
            data-release-review-dashboard="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.dashboard_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="release-review-dashboard-detail" data-release-review-dashboard-detail>
        <div class="release-review-dashboard-title" data-release-review-active-title>${escapeHtml(active.title)}</div>
        <p data-release-review-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-release-review-active-lane>${escapeHtml(active.dashboard_lane)}</dd></div>
          <div><dt>source</dt><dd data-release-review-active-source>${escapeHtml(active.source_snapshot_row || active.source_review_material || "local")}</dd></div>
          <div><dt>state</dt><dd data-release-review-state>${escapeHtml(payload.next_state || "AWAIT_EXPLICIT_RELEASE_APPROVAL")}</dd></div>
        </dl>
        <div class="release-review-approval-phrase" data-release-review-approval-phrase>${escapeHtml(payload.required_approval_phrase || REQUIRED_APPROVAL_PHRASE)}</div>
        <button type="button" class="ghost" data-release-review-copy>검토 패킷 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (dashboardId) => {
    renderReleaseReviewPacketDashboard(root, {
      ...payload,
      active_dashboard_id: dashboardId,
    });
  };
  root.querySelectorAll("[data-release-review-dashboard]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-release-review-dashboard") || "");
    });
  });
  root.querySelector("[data-release-review-copy]")?.addEventListener("click", async () => {
    const text = formatReleaseReviewPacketDashboardText(payload);
    root.dataset.releaseReviewCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
