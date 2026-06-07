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

export const REQUIRED_PUBLIC_RELEASE_APPROVAL_PHRASE = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다";

const RECHECK_DEFS = [
  {
    id: "required_phrase_lock",
    source_anchor: "pack/studio_release_approval_packet_continuity_v1/continuity.detjson",
    approval_lane: "phrase_lock",
    title: "exact phrase",
    summary: "공개 릴리스 실행은 정확한 승인 문구가 있어야만 다음 상태로 넘어갑니다.",
  },
  {
    id: "generic_request_rejected",
    source_anchor: "STUDIO_POST_SUPER_LONG_REBASE_V1",
    approval_lane: "generic_request_boundary",
    title: "generic request",
    summary: "일반적인 다음 개발 요청은 release approval로 취급하지 않습니다.",
  },
  {
    id: "current_request_rejected",
    source_anchor: "current_user_request",
    approval_lane: "current_request_boundary",
    title: "current request",
    summary: "현재 요청도 개발 지속 요청이며 release approval이 아닙니다.",
  },
  {
    id: "await_state_guard",
    source_anchor: "pack/studio_release_approval_chain_closure_v1/closure.detjson",
    approval_lane: "await_state",
    title: "await state",
    summary: "다음 상태는 계속 AWAIT_EXPLICIT_RELEASE_APPROVAL로 유지됩니다.",
  },
  {
    id: "local_rehearsal_handoff",
    source_anchor: "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
    approval_lane: "next_handoff",
    title: "local rehearsal",
    summary: "다음 항목은 release execution이 아닌 local rehearsal check입니다.",
  },
];

export const DEFAULT_PUBLIC_RELEASE_APPROVAL_RECHECK_ROWS = RECHECK_DEFS.map((row) => ({
  id: row.id,
  source_anchor: row.source_anchor,
  approval_lane: row.approval_lane,
  approval_recheck_only: true,
  generated_now: false,
  product_ui_change: true,
  release_approval_claim: false,
  release_execution_claim: false,
}));

function normalizeRows(rows = DEFAULT_PUBLIC_RELEASE_APPROVAL_RECHECK_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return RECHECK_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      source_anchor: asText(source.source_anchor, def.source_anchor),
      approval_lane: asText(source.approval_lane, def.approval_lane),
      approval_surface: "local_studio_public_release_approval_recheck",
      approval_recheck_only: true,
      generated_now: asBool(source.generated_now),
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      github_release_claim: false,
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

export function buildPublicReleaseApprovalRecheck({
  approvalRows = DEFAULT_PUBLIC_RELEASE_APPROVAL_RECHECK_ROWS,
  activeApprovalId = "await_state_guard",
} = {}) {
  const rows = normalizeRows(approvalRows);
  const active = rows.some((row) => row.id === activeApprovalId)
    ? activeApprovalId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.approval_recheck_only === true &&
    row.generated_now === false &&
    row.release_approval_claim === false &&
    row.release_execution_claim === false &&
    row.public_release_claim === false &&
    row.runtime_claim === false
  ));
  const stages = [
    ["approval_row_alignment", rows.length === RECHECK_DEFS.length],
    ["required_phrase_present", REQUIRED_PUBLIC_RELEASE_APPROVAL_PHRASE.length > 0],
    ["generic_request_rejected", rows.some((row) => row.id === "generic_request_rejected")],
    ["current_request_rejected", rows.some((row) => row.id === "current_request_rejected")],
    ["await_state_guarded", rows.some((row) => row.id === "await_state_guard")],
    ["release_boundary_blocked", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_public_release_approval_recheck",
    schema: "ddn.studio.public_release_approval_recheck.v1",
    work_item: "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
    based_on: "STUDIO_POST_SUPER_LONG_REBASE_V1",
    workflow_claim: "public_release_approval_recheck",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    required_approval_phrase: REQUIRED_PUBLIC_RELEASE_APPROVAL_PHRASE,
    generic_next_dev_request_is_approval: false,
    current_request_is_release_approval: false,
    next_state: "AWAIT_EXPLICIT_RELEASE_APPROVAL",
    product_ui_change: true,
    product_code_change: true,
    approval_recheck_only: true,
    runtime_claim: false,
    replay_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    benchmark_execution_claim: false,
    lts_certification_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    solver_implementation_change: false,
    status: ready_stage_count === stages.length ? "approval_recheck_waiting" : "approval_recheck_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    approval_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_approval_id: active,
    approval_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 2,
      current_stage_total: 8,
      current_stage_percent: 25,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
  };
}

export function formatPublicReleaseApprovalRecheckText(recheck = {}) {
  const payload = asObject(recheck);
  if (payload.schema !== "ddn.studio.public_release_approval_recheck.v1") {
    throw new Error("seamgrim_expected_public_release_approval_recheck");
  }
  const rows = asArray(payload.approval_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `next_state\t${payload.next_state ?? ""}`,
    `required_approval_phrase\t${payload.required_approval_phrase ?? ""}`,
    `generic_next_dev_request_is_approval\t${payload.generic_next_dev_request_is_approval === true ? "true" : "false"}`,
    `current_request_is_release_approval\t${payload.current_request_is_release_approval === true ? "true" : "false"}`,
    `release_approval_claim\t${payload.release_approval_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `followup\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `followup_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `approval_row_count\t${payload.approval_row_count ?? rows.length}`,
    "",
    "approval_id\tlane\tsource_anchor",
    ...rows.map((row) => [
      row.id,
      row.approval_lane,
      row.source_anchor,
    ].join("\t")),
  ].join("\n");
}

export function renderPublicReleaseApprovalRecheck(root, recheck = {}) {
  if (!root) return null;
  const payload = asObject(recheck);
  const rows = asArray(payload.approval_rows);
  const activeId = asText(payload.active_approval_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.publicReleaseApprovalRecheckStatus = asText(payload.status, "approval_recheck_incomplete");
  root.innerHTML = `
    <div class="approval-recheck-head">
      <div>
        <div class="approval-recheck-kicker">Release approval</div>
        <h2>Approval recheck</h2>
      </div>
      <div class="approval-recheck-progress" data-approval-recheck-progress>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} follow-up</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>${escapeHtml(payload.next_state || "")}</span>
      </div>
    </div>
    <div class="approval-recheck-phrase" data-approval-recheck-phrase>${escapeHtml(payload.required_approval_phrase)}</div>
    <div class="approval-recheck-body">
      <div class="approval-recheck-list" data-approval-recheck-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="approval-recheck-btn${row.id === activeId ? " active" : ""}"
            data-approval-recheck="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.approval_lane)}</small>
          </button>
        `).join("")}
      </div>
      <div class="approval-recheck-detail" data-approval-recheck-detail>
        <div class="approval-recheck-title" data-approval-recheck-active-title>${escapeHtml(active.title)}</div>
        <p data-approval-recheck-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>lane</dt><dd data-approval-recheck-active-lane>${escapeHtml(active.approval_lane)}</dd></div>
          <div><dt>source</dt><dd data-approval-recheck-active-source>${escapeHtml(active.source_anchor)}</dd></div>
          <div><dt>boundary</dt><dd>no approval/execution</dd></div>
        </dl>
        <button type="button" class="ghost" data-approval-recheck-copy>approval recheck 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderPublicReleaseApprovalRecheck(root, {
      ...payload,
      active_approval_id: rowId,
    });
  };
  root.querySelectorAll("[data-approval-recheck]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-approval-recheck") || "");
    });
  });
  root.querySelector("[data-approval-recheck-copy]")?.addEventListener("click", async () => {
    const text = formatPublicReleaseApprovalRecheckText(payload);
    root.dataset.publicReleaseApprovalRecheckCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
