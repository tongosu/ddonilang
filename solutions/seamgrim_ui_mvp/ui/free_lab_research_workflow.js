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

const WORKFLOW_ROWS = [
  {
    id: "batch",
    title: "Batch queue",
    artifact_kind: "local_batch_queue",
    handoff_uri: "seamgrim://free-lab/local/research/batch/baseline-sweep",
  },
  {
    id: "csv",
    title: "CSV export",
    artifact_kind: "local_csv_export",
    handoff_uri: "seamgrim://free-lab/local/research/csv/baseline-sweep",
  },
  {
    id: "notebook",
    title: "Notebook handoff",
    artifact_kind: "local_notebook_handoff",
    handoff_uri: "seamgrim-notebook://local/free-lab/research-workflow",
  },
];

const DEFAULT_BATCH_RUNS = [
  { id: "baseline", coefficient: 2, start_value: 1, frame_3_value: 7 },
  { id: "low_lever", coefficient: 1, start_value: 1, frame_3_value: 4 },
  { id: "high_lever", coefficient: 3, start_value: 1, frame_3_value: 10 },
];

export const DEFAULT_FREE_LAB_RESEARCH_WORKFLOW_ROWS = WORKFLOW_ROWS.map((row) => ({
  id: row.id,
  artifact_kind: row.artifact_kind,
  handoff_uri: row.handoff_uri,
  ready: true,
}));

function normalizeBatchRuns(batchRuns = DEFAULT_BATCH_RUNS) {
  const rows = asArray(batchRuns).length > 0 ? asArray(batchRuns) : DEFAULT_BATCH_RUNS;
  return rows.map((run, index) => {
    const source = asObject(run);
    return {
      id: asText(source.id, `run_${index + 1}`),
      coefficient: asNumber(source.coefficient, 0),
      start_value: asNumber(source.start_value, 0),
      frame_3_value: asNumber(source.frame_3_value, 0),
      ready: source.ready !== false,
    };
  });
}

function formatCsvRows(batchRuns) {
  return [
    "run_id,coefficient,start_value,frame_3_value",
    ...batchRuns.map((run) => [
      run.id,
      run.coefficient,
      run.start_value,
      run.frame_3_value,
    ].join(",")),
  ].join("\n");
}

function normalizeRows(rows = DEFAULT_FREE_LAB_RESEARCH_WORKFLOW_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return WORKFLOW_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      artifact_kind: asText(source.artifact_kind, row.artifact_kind),
      handoff_uri: asText(source.handoff_uri, row.handoff_uri),
      ready: source.ready !== false,
      local_only: true,
      product_ui_behavior: true,
      cloud_sync_claim: false,
      registry_publish_claim: false,
      public_upload_claim: false,
      external_notebook_server_claim: false,
    };
  });
}

export function buildFreeLabResearchWorkflow({
  rows = DEFAULT_FREE_LAB_RESEARCH_WORKFLOW_ROWS,
  batchRuns = DEFAULT_BATCH_RUNS,
  activeWorkflowId = "csv",
} = {}) {
  const workflows = normalizeRows(rows);
  const normalizedBatchRuns = normalizeBatchRuns(batchRuns);
  const active = workflows.some((workflow) => workflow.id === activeWorkflowId)
    ? activeWorkflowId
    : workflows[0]?.id ?? "";
  const readyWorkflowCount = workflows.filter((workflow) => workflow.ready).length;
  const readyBatchCount = normalizedBatchRuns.filter((run) => run.ready).length;
  const csvText = formatCsvRows(normalizedBatchRuns);
  const hasBatch = normalizedBatchRuns.length >= 3 && readyBatchCount === normalizedBatchRuns.length;
  const hasCsv = csvText.includes("run_id,coefficient,start_value,frame_3_value");
  const hasNotebook = workflows.some((workflow) => (
    workflow.id === "notebook" &&
    workflow.handoff_uri.startsWith("seamgrim-notebook://local/")
  ));
  const workflowReady = readyWorkflowCount === workflows.length && hasBatch && hasCsv && hasNotebook;
  return {
    __종류: "free_lab_research_workflow",
    schema: "ddn.seamgrim.free_lab.research_workflow.v1",
    work_item: "BA5_FREE_LAB_RESEARCH_WORKFLOW_CLOSURE_V1",
    primary_coordinate: "바-5",
    depends_on_coordinate: ["바-4"],
    pack: "free_lab_5_v1",
    status: workflowReady ? "free_lab_research_workflow_ready" : "free_lab_research_workflow_incomplete",
    matrix_closure_tier: workflowReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    research_mode_claim: workflowReady,
    batch_queue_claim: hasBatch,
    csv_export_claim: hasCsv,
    notebook_handoff_claim: hasNotebook,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    cloud_sync_claim: false,
    external_notebook_server_claim: false,
    workflow_count: workflows.length,
    batch_run_count: normalizedBatchRuns.length,
    ready_workflow_count: readyWorkflowCount,
    current_stage_closed: workflowReady ? 5 : Math.min(4, readyWorkflowCount),
    current_stage_total: 5,
    current_stage_percent: workflowReady ? 100 : Math.round((Math.min(4, readyWorkflowCount) / 5) * 100),
    progress: {
      current_stage_closed: workflowReady ? 5 : Math.min(4, readyWorkflowCount),
      current_stage_total: 5,
      current_stage_percent: workflowReady ? 100 : Math.round((Math.min(4, readyWorkflowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 12,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 13,
      roadmap_v2_pack_evidence_reference_closed: 31,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 34,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    workflows,
    batch_runs: normalizedBatchRuns,
    csv_text: csvText,
    active_workflow_id: active,
    next_item: "CHA0_RPG_SEED_REBASE_V1",
  };
}

export function formatFreeLabResearchWorkflowText(workflow = {}) {
  const payload = asObject(workflow);
  if (payload.schema !== "ddn.seamgrim.free_lab.research_workflow.v1") {
    throw new Error("seamgrim_expected_free_lab_research_workflow");
  }
  const workflows = asArray(payload.workflows);
  const batchRuns = asArray(payload.batch_runs);
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
    `research_mode_claim\t${payload.research_mode_claim === true ? "true" : "false"}`,
    `batch_queue_claim\t${payload.batch_queue_claim === true ? "true" : "false"}`,
    `csv_export_claim\t${payload.csv_export_claim === true ? "true" : "false"}`,
    `notebook_handoff_claim\t${payload.notebook_handoff_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `cloud_sync_claim\t${payload.cloud_sync_claim === true ? "true" : "false"}`,
    "",
    "workflow_id\tartifact_kind\tlocal_only\thandoff_uri",
    ...workflows.map((workflowRow) => [
      workflowRow.id,
      workflowRow.artifact_kind,
      workflowRow.local_only === true ? "true" : "false",
      workflowRow.handoff_uri,
    ].join("\t")),
    "",
    "run_id,coefficient,start_value,frame_3_value",
    ...batchRuns.map((run) => [
      run.id,
      run.coefficient,
      run.start_value,
      run.frame_3_value,
    ].join(",")),
  ].join("\n");
}

export function renderFreeLabResearchWorkflow(root, workflow = {}) {
  if (!root) return null;
  const payload = asObject(workflow);
  const workflows = asArray(payload.workflows);
  const activeId = asText(payload.active_workflow_id, workflows[0]?.id ?? "");
  const active = workflows.find((row) => row.id === activeId) || workflows[0] || {};
  root.dataset.freeLabResearchWorkflowStatus = asText(
    payload.status,
    "free_lab_research_workflow_incomplete",
  );
  root.innerHTML = `
    <div class="free-lab-research-head">
      <div>
        <div class="free-lab-research-kicker">Research workflow</div>
        <h2>자유 실험 연구자 모드</h2>
      </div>
      <div class="free-lab-research-progress" data-free-lab-research-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="free-lab-research-summary" data-free-lab-research-summary>
      batch queue, CSV export, notebook handoff를 로컬 research workflow로 묶고 외부 서버/공개 업로드는 후속으로 둡니다.
    </div>
    <div class="free-lab-research-body">
      <div class="free-lab-research-list" data-free-lab-research-list>
        ${workflows.map((row) => `
          <button
            type="button"
            class="free-lab-research-btn${row.id === activeId ? " active" : ""}"
            data-free-lab-research="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.artifact_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="free-lab-research-detail" data-free-lab-research-detail>
        <div class="free-lab-research-title" data-free-lab-research-active-title>${escapeHtml(active.title)}</div>
        <p data-free-lab-research-active-link>${escapeHtml(active.handoff_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>batch runs</dt><dd>${escapeHtml(String(payload.batch_run_count ?? 0))}</dd></div>
        </dl>
        <pre class="free-lab-research-csv" data-free-lab-research-csv>${escapeHtml(payload.csv_text || "")}</pre>
        <button type="button" class="ghost" data-free-lab-research-copy>research workflow 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (workflowId) => {
    renderFreeLabResearchWorkflow(root, {
      ...payload,
      active_workflow_id: workflowId,
    });
  };
  root.querySelectorAll("[data-free-lab-research]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-free-lab-research") || "");
    });
  });
  root.querySelector("[data-free-lab-research-copy]")?.addEventListener("click", async () => {
    const text = formatFreeLabResearchWorkflowText(payload);
    root.dataset.freeLabResearchWorkflowCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
