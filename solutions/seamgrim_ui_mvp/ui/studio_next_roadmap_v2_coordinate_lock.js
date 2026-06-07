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

const DECISION_DEFS = [
  {
    id: "default_next_coordinate",
    decision: "마-3",
    lane: "coordinate",
    title: "기본 좌표",
    summary: "post-super-long follow-up 이후 기본 ROADMAP_V2 좌표를 마-3으로 고정합니다.",
  },
  {
    id: "studio_first_continuity",
    decision: "preserve_studio_first",
    lane: "continuity",
    title: "Studio 우선 연속성",
    summary: "닫힌 super-long evidence가 Studio productization 중심임을 유지합니다.",
  },
  {
    id: "post_followup_denominator_closed",
    decision: "8/8_closed",
    lane: "denominator",
    title: "후속 분모 마감",
    summary: "post-super-long follow-up 분모를 8/8로 봉인하고 암묵 확장을 막습니다.",
  },
  {
    id: "release_execution_still_approval_gated",
    decision: "approval_phrase_required",
    lane: "release_gate",
    title: "release 승인 게이트",
    summary: "release 실행은 명시 승인 문구 없이는 계속 차단합니다.",
  },
  {
    id: "next_queue_requires_explicit_selection",
    decision: "AWAIT_NEXT_DEVELOPMENT_SELECTION",
    lane: "next_state",
    title: "다음 선택 대기",
    summary: "새 구현 큐나 새 denominator는 다음 명시 선택 후에만 엽니다.",
  },
];

export const DEFAULT_NEXT_ROADMAP_V2_COORDINATE_LOCK_DECISIONS = DECISION_DEFS.map((row) => ({
  id: row.id,
  decision: row.decision,
  lane: row.lane,
  locked: true,
  opens_new_queue: false,
  runtime_claim: false,
}));

function normalizeDecisions(decisions = DEFAULT_NEXT_ROADMAP_V2_COORDINATE_LOCK_DECISIONS) {
  const byId = new Map(asArray(decisions).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return DECISION_DEFS.map((def) => {
    const source = byId.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      decision: asText(source.decision, def.decision),
      lane: asText(source.lane, def.lane),
      locked: true,
      opens_new_queue: false,
      runtime_claim: false,
      generated_now: asBool(source.generated_now),
      new_automatic_queue_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_upload_claim: false,
      benchmark_execution_claim: false,
    };
  });
}

export function buildNextRoadmapV2CoordinateLock({
  decisions = DEFAULT_NEXT_ROADMAP_V2_COORDINATE_LOCK_DECISIONS,
  activeDecisionId = "default_next_coordinate",
} = {}) {
  const rows = normalizeDecisions(decisions);
  const active = rows.some((row) => row.id === activeDecisionId)
    ? activeDecisionId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.locked === true &&
    row.opens_new_queue === false &&
    row.runtime_claim === false &&
    row.release_execution_claim === false &&
    row.public_upload_claim === false &&
    row.benchmark_execution_claim === false
  ));
  const stages = [
    ["decision_count", rows.length === DECISION_DEFS.length],
    ["default_coordinate_ma3", rows.some((row) => row.id === "default_next_coordinate" && row.decision === "마-3")],
    ["followup_denominator_closed", rows.some((row) => row.decision === "8/8_closed")],
    ["await_next_selection", rows.some((row) => row.decision === "AWAIT_NEXT_DEVELOPMENT_SELECTION")],
    ["local_only_boundary", localOnly],
    ["no_new_automatic_queue", true],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_next_roadmap_v2_coordinate_lock",
    schema: "ddn.studio.next_roadmap_v2_coordinate_lock.v1",
    work_item: "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
    based_on: "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    selected_default_coordinate: "마-3",
    next_state: "AWAIT_NEXT_DEVELOPMENT_SELECTION",
    workflow_claim: "next_roadmap_v2_coordinate_lock",
    generated_locally: true,
    product_code_change: true,
    product_ui_change: true,
    coordinate_lock_only: true,
    runtime_claim: false,
    new_automatic_queue_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    publication_snapshot_emit_claim: false,
    archive_generation_claim: false,
    publication_checksum_generation_claim: false,
    artifact_signing_claim: false,
    benchmark_execution_claim: false,
    performance_baseline_generation_claim: false,
    performance_baseline_publication_claim: false,
    lts_certification_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    cloud_sync_claim: false,
    account_setup_claim: false,
    permission_system_claim: false,
    result_replay_claim: false,
    status: ready_stage_count === stages.length ? "next_roadmap_v2_coordinate_lock_ready" : "next_roadmap_v2_coordinate_lock_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    decision_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_decision_id: active,
    coordinate_decisions: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 8,
      current_stage_total: 8,
      current_stage_percent: 100,
      roadmap_v2_behavior_closed: 88,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 98,
    },
    next_item: null,
  };
}

export function formatNextRoadmapV2CoordinateLockText(lock = {}) {
  const payload = asObject(lock);
  if (payload.schema !== "ddn.studio.next_roadmap_v2_coordinate_lock.v1") {
    throw new Error("seamgrim_expected_next_roadmap_v2_coordinate_lock");
  }
  const rows = asArray(payload.coordinate_decisions);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `selected_default_coordinate\t${payload.selected_default_coordinate ?? ""}`,
    `next_state\t${payload.next_state ?? ""}`,
    `product_ui_change\t${payload.product_ui_change === true ? "true" : "false"}`,
    `new_automatic_queue_claim\t${payload.new_automatic_queue_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `followup\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `followup_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    "",
    "decision_id\tdecision\tlane\tlocked",
    ...rows.map((row) => [
      row.id,
      row.decision,
      row.lane,
      row.locked === true ? "true" : "false",
    ].join("\t")),
  ].join("\n");
}

export function renderNextRoadmapV2CoordinateLock(root, lock = {}) {
  if (!root) return null;
  const payload = asObject(lock);
  const rows = asArray(payload.coordinate_decisions);
  const activeId = asText(payload.active_decision_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.nextRoadmapV2CoordinateLockStatus = asText(payload.status, "next_roadmap_v2_coordinate_lock_incomplete");
  root.innerHTML = `
    <div class="next-roadmap-lock-head">
      <div>
        <div class="next-roadmap-lock-kicker">ROADMAP_V2</div>
        <h2>다음 좌표 잠금</h2>
      </div>
      <div class="next-roadmap-lock-progress" data-next-roadmap-lock-progress>
        <span>${escapeHtml(payload.selected_default_coordinate || "마-3")}</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} follow-up</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(payload.next_state || "AWAIT_NEXT_DEVELOPMENT_SELECTION")}</span>
      </div>
    </div>
    <div class="next-roadmap-lock-summary">
      ${escapeHtml(String(payload.decision_count ?? rows.length))} decisions · new_queue=false · runtime=false · release_execution=false
    </div>
    <div class="next-roadmap-lock-body">
      <div class="next-roadmap-lock-list" data-next-roadmap-lock-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="next-roadmap-lock-btn${row.id === activeId ? " active" : ""}"
            data-next-roadmap-lock="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="next-roadmap-lock-detail" data-next-roadmap-lock-detail>
        <div class="next-roadmap-lock-title" data-next-roadmap-lock-active-title>${escapeHtml(active.title)}</div>
        <p data-next-roadmap-lock-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>decision</dt><dd data-next-roadmap-lock-active-decision>${escapeHtml(active.decision)}</dd></div>
          <div><dt>lane</dt><dd data-next-roadmap-lock-active-lane>${escapeHtml(active.lane)}</dd></div>
          <div><dt>boundary</dt><dd>no automatic queue</dd></div>
        </dl>
        <button type="button" class="ghost" data-next-roadmap-lock-copy>좌표 잠금 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (decisionId) => {
    renderNextRoadmapV2CoordinateLock(root, {
      ...payload,
      active_decision_id: decisionId,
    });
  };
  root.querySelectorAll("[data-next-roadmap-lock]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-next-roadmap-lock") || "");
    });
  });
  root.querySelector("[data-next-roadmap-lock-copy]")?.addEventListener("click", async () => {
    const text = formatNextRoadmapV2CoordinateLockText(payload);
    root.dataset.nextRoadmapV2CoordinateLockCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
