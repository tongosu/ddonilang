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

const FOLLOWUP_DEFS = [
  {
    id: "STUDIO_POST_SUPER_LONG_REBASE_V1",
    coordinate: "마-3",
    status: "closed",
    claim: "post-super-long denominator lock",
  },
  {
    id: "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
    coordinate: "마-3",
    status: "next",
    claim: "approval readiness recheck only",
  },
  {
    id: "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
    coordinate: "마-3",
    status: "planned",
    claim: "local rehearsal check only",
  },
  {
    id: "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
    coordinate: "타-3",
    status: "planned",
    claim: "dry-run artifact manifest only",
  },
  {
    id: "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
    coordinate: "하-3",
    status: "planned",
    claim: "local feedback seed only",
  },
  {
    id: "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
    coordinate: "하-3",
    status: "planned",
    claim: "local triage evidence only",
  },
  {
    id: "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
    coordinate: "타-3",
    status: "planned",
    claim: "baseline prep dry-run only",
  },
  {
    id: "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
    coordinate: "마-3",
    status: "planned",
    claim: "next ROADMAP_V2 coordinate lock",
  },
];

export const DEFAULT_POST_SUPER_LONG_REBASE_ROWS = FOLLOWUP_DEFS.map((row) => ({
  id: row.id,
  coordinate: row.coordinate,
  status: row.status,
  claim: row.claim,
  followup_rebase_only: true,
  generated_now: false,
  product_ui_change: true,
  release_execution_claim: false,
  public_release_claim: false,
}));

function normalizeRows(rows = DEFAULT_POST_SUPER_LONG_REBASE_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return FOLLOWUP_DEFS.map((def, index) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      coordinate: asText(source.coordinate, def.coordinate),
      status: asText(source.status, def.status),
      claim: asText(source.claim, def.claim),
      order: index + 1,
      rebase_surface: "local_studio_post_super_long_rebase",
      followup_rebase_only: true,
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

export function buildPostSuperLongRebase({
  followupRows = DEFAULT_POST_SUPER_LONG_REBASE_ROWS,
  activeFollowupId = "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
} = {}) {
  const rows = normalizeRows(followupRows);
  const active = rows.some((row) => row.id === activeFollowupId)
    ? activeFollowupId
    : rows[0]?.id ?? "";
  const localOnly = rows.every((row) => (
    row.followup_rebase_only === true &&
    row.generated_now === false &&
    row.release_approval_claim === false &&
    row.release_execution_claim === false &&
    row.public_release_claim === false &&
    row.runtime_claim === false
  ));
  const stages = [
    ["followup_row_alignment", rows.length === FOLLOWUP_DEFS.length],
    ["super_long_sealed", true],
    ["closed_item_present", rows.some((row) => row.id === "STUDIO_POST_SUPER_LONG_REBASE_V1" && row.status === "closed")],
    ["next_item_selected", rows.some((row) => row.id === "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1" && row.status === "next")],
    ["blocked_release_boundary", localOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_post_super_long_rebase",
    schema: "ddn.studio.post_super_long_rebase.v1",
    work_item: "STUDIO_POST_SUPER_LONG_REBASE_V1",
    based_on: "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
    workflow_claim: "post_super_long_rebase",
    primary_coordinate: "마-3",
    support_coordinate: "타-3",
    product_ui_change: true,
    product_code_change: true,
    followup_rebase_only: true,
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
    status: ready_stage_count === stages.length ? "post_super_long_rebased" : "post_super_long_rebase_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    super_long_plan_closed: false,
    followup_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_followup_id: active,
    followup_rows: rows,
    stages,
    progress: {
      super_long_behavior_closed: 9,
      super_long_total: 18,
      super_long_percent: 50,
      current_stage_closed: 1,
      current_stage_total: 8,
      current_stage_percent: 13,
      roadmap_v2_behavior_closed: 51,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 57,
    },
    next_item: "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
  };
}

export function formatPostSuperLongRebaseText(rebase = {}) {
  const payload = asObject(rebase);
  if (payload.schema !== "ddn.studio.post_super_long_rebase.v1") {
    throw new Error("seamgrim_expected_post_super_long_rebase");
  }
  const rows = asArray(payload.followup_rows);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `super_long\t${payload.progress?.super_long_behavior_closed ?? 0}/${payload.progress?.super_long_total ?? 0}`,
    `followup\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `followup_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    `public_release_claim\t${payload.public_release_claim === true ? "true" : "false"}`,
    `runtime_claim\t${payload.runtime_claim === true ? "true" : "false"}`,
    `followup_row_count\t${payload.followup_row_count ?? rows.length}`,
    "",
    "followup_id\tcoordinate\tstatus\tclaim",
    ...rows.map((row) => [
      row.id,
      row.coordinate,
      row.status,
      row.claim,
    ].join("\t")),
  ].join("\n");
}

export function renderPostSuperLongRebase(root, rebase = {}) {
  if (!root) return null;
  const payload = asObject(rebase);
  const rows = asArray(payload.followup_rows);
  const activeId = asText(payload.active_followup_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.postSuperLongRebaseStatus = asText(payload.status, "post_super_long_rebase_incomplete");
  root.innerHTML = `
    <div class="post-super-rebase-head">
      <div>
        <div class="post-super-rebase-kicker">Studio follow-up</div>
        <h2>Post-super-long rebase</h2>
      </div>
      <div class="post-super-rebase-progress" data-post-super-rebase-progress>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.super_long_percent ?? 0))}%</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} follow-up</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="post-super-rebase-body">
      <div class="post-super-rebase-list" data-post-super-rebase-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="post-super-rebase-btn${row.id === activeId ? " active" : ""}"
            data-post-super-rebase="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.id)}</span>
            <small>${escapeHtml(`${row.coordinate} · ${row.status}`)}</small>
          </button>
        `).join("")}
      </div>
      <div class="post-super-rebase-detail" data-post-super-rebase-detail>
        <div class="post-super-rebase-title" data-post-super-rebase-active-title>${escapeHtml(active.id)}</div>
        <p data-post-super-rebase-active-claim>${escapeHtml(active.claim)}</p>
        <dl>
          <div><dt>coordinate</dt><dd data-post-super-rebase-active-coordinate>${escapeHtml(active.coordinate)}</dd></div>
          <div><dt>status</dt><dd data-post-super-rebase-active-status>${escapeHtml(active.status)}</dd></div>
          <div><dt>boundary</dt><dd>no release/runtime</dd></div>
        </dl>
        <button type="button" class="ghost" data-post-super-rebase-copy>follow-up 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderPostSuperLongRebase(root, {
      ...payload,
      active_followup_id: rowId,
    });
  };
  root.querySelectorAll("[data-post-super-rebase]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-post-super-rebase") || "");
    });
  });
  root.querySelector("[data-post-super-rebase-copy]")?.addEventListener("click", async () => {
    const text = formatPostSuperLongRebaseText(payload);
    root.dataset.postSuperLongRebaseCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
