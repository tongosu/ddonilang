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

const CANDIDATE_IDS = [
  "rep_econ_supply_demand_tax_v1",
  "rep_math_function_line_v1",
  "rep_phys_projectile_xy_v1",
  "rep_physics_velocity_history_v1",
  "rep_cs_linear_search_timeline_v1",
  "rep_science_phase_change_timeline_v1",
  "rep_grid_game_state_drop_v1",
  "rep_econ_growth_compound_v1",
  "rep_ddonirang_vol2_filter_v1",
  "rep_ddonirang_vol2_map_v1",
  "rep_ddonirang_vol2_pipeline_v1",
  "rep_ddonirang_vol4_event_dispatch_v1",
  "rep_ddonirang_vol4_state_transition_v1",
  "rep_ddonirang_vol4_resume_isolation_v1",
  "rep_ddonirang_vol4_multi_signal_priority_v1",
];

const SURFACE_DEFS = [
  {
    id: "candidate_catalog_review_surface",
    source_review_gate: "candidate_ids_present_in_lesson_index",
    source_dashboard_row: null,
    surface_lane: "candidate_catalog",
    title: "후보 수업 카탈로그",
    summary: "15개 후보 수업이 로컬 lesson index 기준으로 검토 가능한지 보여줍니다.",
  },
  {
    id: "active_allowlist_review_surface",
    source_review_gate: "candidate_ids_match_active_allowlist",
    source_dashboard_row: null,
    surface_lane: "active_allowlist_review",
    title: "Active allowlist 검토",
    summary: "allowlist를 변경하지 않고 후보 ID 정합성만 확인합니다.",
  },
  {
    id: "lesson_index_alignment_surface",
    source_review_gate: "candidate_ids_present_in_lesson_index",
    source_dashboard_row: null,
    surface_lane: "lesson_index_alignment",
    title: "Lesson index 정렬",
    summary: "공개 후보가 lesson index와 어긋나지 않는지 로컬 표면으로 묶습니다.",
  },
  {
    id: "local_packaging_review_surface",
    source_review_gate: "local_packaging_consolidation_checker_passes",
    source_dashboard_row: "local_packaging_review_dashboard_card",
    surface_lane: "local_packaging_review",
    title: "로컬 패키징 검토",
    summary: "로컬 패키징 evidence를 공개 업로드 전 검토 상태로 유지합니다.",
  },
  {
    id: "release_dashboard_publication_surface",
    source_review_gate: "docs_ssot_clean",
    source_dashboard_row: "publication_prep_review_dashboard_card",
    surface_lane: "release_dashboard_publication",
    title: "릴리스 대시보드 연결",
    summary: "release review dashboard와 publication prep evidence를 연결합니다.",
  },
  {
    id: "registry_share_handoff_surface",
    source_review_gate: "docs_ssot_clean",
    source_dashboard_row: "registry_share_review_dashboard_card",
    surface_lane: "registry_share_handoff",
    title: "Registry/share 인계",
    summary: "registry/share handoff를 public release 없이 검토합니다.",
  },
];

export const DEFAULT_LESSON_PUBLICATION_REVIEW_GATES = [
  "candidate_ids_present_in_lesson_index",
  "candidate_ids_match_active_allowlist",
  "local_packaging_consolidation_checker_passes",
  "docs_ssot_clean",
];

export const DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS = CANDIDATE_IDS;

export const DEFAULT_LESSON_PUBLICATION_DASHBOARD_ROWS = [
  "local_packaging_review_dashboard_card",
  "publication_prep_review_dashboard_card",
  "registry_share_review_dashboard_card",
].map((id) => ({
  id,
  dashboard_only: true,
  generated_now: false,
  release_approval_claim: false,
  release_execution_claim: false,
  public_release_claim: false,
}));

function normalizeSurfaceRows({
  reviewGates = DEFAULT_LESSON_PUBLICATION_REVIEW_GATES,
  dashboardRows = DEFAULT_LESSON_PUBLICATION_DASHBOARD_ROWS,
  candidateIds = DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS,
} = {}) {
  const gates = new Set(asArray(reviewGates).map((gate) => asText(gate)).filter(Boolean));
  const dashboardById = new Map(asArray(dashboardRows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  const candidateCount = asArray(candidateIds).length || CANDIDATE_IDS.length;
  return SURFACE_DEFS.map((def) => {
    const dashboard = def.source_dashboard_row ? dashboardById.get(def.source_dashboard_row) || {} : {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_review_gate: def.source_review_gate,
      source_dashboard_row: def.source_dashboard_row,
      surface_lane: def.surface_lane,
      surface_kind: "local_lesson_publication_review_surface",
      candidate_count: candidateCount,
      source_available: gates.has(def.source_review_gate) && (!def.source_dashboard_row || dashboardById.has(def.source_dashboard_row)),
      surface_only: true,
      generated_now: asBool(dashboard.generated_now),
      public_upload_claim: false,
      registry_publish_claim: false,
      publication_snapshot_emit_claim: false,
      active_allowlist_mutation: false,
      lesson_schema_change: false,
      product_ui_change: true,
    };
  });
}

export function buildLessonPublicationReviewSurface({
  reviewGates = DEFAULT_LESSON_PUBLICATION_REVIEW_GATES,
  dashboardRows = DEFAULT_LESSON_PUBLICATION_DASHBOARD_ROWS,
  candidateIds = DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS,
  activeSurfaceId = "candidate_catalog_review_surface",
} = {}) {
  const rows = normalizeSurfaceRows({ reviewGates, dashboardRows, candidateIds });
  const active = rows.some((row) => row.id === activeSurfaceId)
    ? activeSurfaceId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.surface_only === true &&
    row.generated_now === false &&
    row.public_upload_claim === false &&
    row.registry_publish_claim === false &&
    row.publication_snapshot_emit_claim === false &&
    row.active_allowlist_mutation === false &&
    row.lesson_schema_change === false
  ));
  const stages = [
    ["surface_row_alignment", rows.length === SURFACE_DEFS.length],
    ["candidate_count_alignment", asArray(candidateIds).length === 15],
    ["review_gate_alignment", DEFAULT_LESSON_PUBLICATION_REVIEW_GATES.every((gate) => asArray(reviewGates).includes(gate))],
    ["dashboard_anchor_alignment", rows.filter((row) => row.source_dashboard_row).length === 3],
    ["local_only_boundary", localOnly],
    ["no_publication_execution", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_lesson_publication_review_surface",
    schema: "ddn.studio.lesson_publication_review_surface.v1",
    work_item: "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    workflow_claim: "lesson_publication_review_surface",
    generated_locally: true,
    product_ui_change: true,
    surface_only: true,
    runtime_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    public_link_creation_claim: false,
    install_enablement_claim: false,
    publication_snapshot_emit_claim: false,
    archive_generation_claim: false,
    publication_checksum_generation_claim: false,
    artifact_signing_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    result_replay_claim: false,
    benchmark_execution_claim: false,
    performance_baseline_generation_claim: false,
    performance_baseline_publication_claim: false,
    lts_certification_claim: false,
    status: ready_stage_count === stages.length ? "lesson_publication_review_surface_ready" : "lesson_publication_review_surface_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    candidate_count: asArray(candidateIds).length,
    candidate_ids: asArray(candidateIds),
    surface_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_surface_id: active,
    surface_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 9,
      super_long_total: 18,
      super_long_percent: 50,
      current_stage_closed: 6,
      current_stage_total: 8,
      current_stage_percent: 75,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
  };
}

export function formatLessonPublicationReviewSurfaceText(surface = {}) {
  const payload = asObject(surface);
  if (payload.schema !== "ddn.studio.lesson_publication_review_surface.v1") {
    throw new Error("seamgrim_expected_lesson_publication_review_surface");
  }
  const rows = asArray(payload.surface_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `active_allowlist_mutation\t${payload.active_allowlist_mutation === true ? "true" : "false"}`,
    `candidate_count\t${payload.candidate_count ?? 0}`,
    `surface_row_count\t${payload.surface_row_count ?? rows.length}`,
    "",
    "surface_id\tsurface_lane\tsource_review_gate\tsource_dashboard_row",
    ...rows.map((row) => [
      row.id,
      row.surface_lane,
      row.source_review_gate,
      row.source_dashboard_row ?? "",
    ].join("\t")),
  ].join("\n");
}

export function renderLessonPublicationReviewSurface(root, surface = {}) {
  if (!root) return null;
  const payload = asObject(surface);
  const rows = asArray(payload.surface_rows);
  const activeId = asText(payload.active_surface_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.lessonPublicationStatus = asText(payload.status, "lesson_publication_review_surface_incomplete");
  root.innerHTML = `
    <div class="lesson-publication-surface-head">
      <div>
        <div class="lesson-publication-surface-kicker">Lesson publication</div>
        <h2>공개 검토 표면</h2>
      </div>
      <div class="lesson-publication-surface-progress" data-lesson-publication-progress>
        <span>${escapeHtml(String(payload.candidate_count ?? 0))}개 후보</span>
        <span>업로드 없음</span>
        <span>allowlist 변경 없음</span>
      </div>
    </div>
    <div class="lesson-publication-surface-body">
      <div class="lesson-publication-surface-list" data-lesson-publication-surface-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="lesson-publication-surface-btn${row.id === activeId ? " active" : ""}"
            data-lesson-publication-surface="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.surface_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="lesson-publication-surface-detail" data-lesson-publication-surface-detail>
        <div class="lesson-publication-surface-title" data-lesson-publication-active-title>${escapeHtml(active.title)}</div>
        <p data-lesson-publication-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-lesson-publication-active-lane>${escapeHtml(active.surface_lane)}</dd></div>
          <div><dt>gate</dt><dd data-lesson-publication-active-gate>${escapeHtml(active.source_review_gate)}</dd></div>
          <div><dt>candidate</dt><dd data-lesson-publication-candidate-count>${escapeHtml(String(payload.candidate_count ?? 0))}</dd></div>
        </dl>
        <button type="button" class="ghost" data-lesson-publication-copy>검토 표면 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (surfaceId) => {
    renderLessonPublicationReviewSurface(root, {
      ...payload,
      active_surface_id: surfaceId,
    });
  };
  root.querySelectorAll("[data-lesson-publication-surface]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-lesson-publication-surface") || "");
    });
  });
  root.querySelector("[data-lesson-publication-copy]")?.addEventListener("click", async () => {
    const text = formatLessonPublicationReviewSurfaceText(payload);
    root.dataset.lessonPublicationCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
