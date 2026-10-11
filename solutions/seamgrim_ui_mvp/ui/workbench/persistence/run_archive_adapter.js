import {
  requestProjectApi,
} from "../../project_service_client.js";

export const WORKBENCH_RUN_ARCHIVE_SCHEMA = "seamgrim.workbench.run_archive_adapter.v1";
export const RUN_ARCHIVE_API_SUFFIX = "runs";

function asNonEmptyText(value = "") {
  return String(value ?? "").trim();
}

function buildRunArchivePrerequisiteError(code, message) {
  const err = new Error(message);
  err.name = "RunArchivePrerequisiteError";
  err.code = code;
  err.status = 0;
  err.adapterCode = "invalid_request";
  return err;
}

function requireRunArchiveProjectBinding({ projectId = "", capability = "", serverRunId = "", operation = "run_archive", requireServerRun = false } = {}) {
  const normalizedProjectId = asNonEmptyText(projectId);
  const normalizedCapability = asNonEmptyText(capability);
  const normalizedServerRunId = asNonEmptyText(serverRunId);
  if (!normalizedProjectId) {
    throw buildRunArchivePrerequisiteError(
      "E_RUN_ARCHIVE_PROJECT_ID_EMPTY",
      `${operation}: 서버 실행 기록에는 저장된 project_id가 필요합니다.`,
    );
  }
  if (!normalizedCapability) {
    throw buildRunArchivePrerequisiteError(
      "E_RUN_ARCHIVE_CAPABILITY_EMPTY",
      `${operation}: 서버 실행 기록에는 편집 또는 보기 권한이 필요합니다.`,
    );
  }
  if (requireServerRun && !normalizedServerRunId) {
    throw buildRunArchivePrerequisiteError(
      "E_RUN_ARCHIVE_SERVER_RUN_ID_EMPTY",
      `${operation}: 서버 실행 기록을 갱신하려면 server_run_id가 필요합니다.`,
    );
  }
  return {
    projectId: normalizedProjectId,
    capability: normalizedCapability,
    serverRunId: normalizedServerRunId,
  };
}

export function buildRunArchiveDraft({
  projectId = "",
  sourceRevisionId = "",
  expectedHeadRevisionId = "",
  sourceContentSha256 = "",
  recipeHash = "",
  seed = "ui-default",
  parentServerRunId = "",
  branchMadi = null,
  branchStateHash = "",
} = {}) {
  return {
    schema: WORKBENCH_RUN_ARCHIVE_SCHEMA,
    project_id: String(projectId || ""),
    source_revision_id: String(sourceRevisionId || ""),
    expected_head_revision_id: String(expectedHeadRevisionId || ""),
    source_content_sha256: String(sourceContentSha256 || ""),
    recipe_hash: String(recipeHash || ""),
    seed: String(seed || "ui-default"),
    parent_server_run_id: String(parentServerRunId || ""),
    branch_madi: branchMadi === null || branchMadi === undefined ? null : Number(branchMadi),
    branch_state_hash: String(branchStateHash || ""),
    persistence_status: "adapter_ready_server_bound",
  };
}

export function buildCheckpointPayload({ run, frame, sequence = 0, previousCheckpointSha256 = "" } = {}) {
  const normalizedSequence = Number(sequence);
  const runtimeMadi = Number(frame?.madi ?? normalizedSequence);
  return {
    sequence: normalizedSequence,
    committed_madi: Math.max(Number.isFinite(runtimeMadi) ? runtimeMadi : normalizedSequence, normalizedSequence),
    previous_checkpoint_sha256: String(previousCheckpointSha256 || ""),
    input_snapshots: frame?.input_snapshot ? [frame.input_snapshot] : [],
    state_hashes: frame?.state_hash ? [String(frame.state_hash)] : [],
    frame_summary: {
      local_run_id: String(run?.run_id || ""),
      frame_count: Array.isArray(run?.frames) ? run.frames.length : 0,
      latest_frame_id: Number(frame?.frame_id ?? sequence),
      renderer_pixels_stored: false,
    },
  };
}

export async function createRunArchive({
  projectId = "",
  capability = "",
  draft = {},
  fetchImpl = globalThis.fetch,
} = {}) {
  const binding = requireRunArchiveProjectBinding({ projectId, capability, operation: "createRunArchive" });
  return requestProjectApi(`${binding.projectId}/${RUN_ARCHIVE_API_SUFFIX}`, {
    method: "POST",
    capability: binding.capability,
    body: draft,
    fetchImpl,
  });
}

export async function appendRunCheckpoint({
  projectId = "",
  serverRunId = "",
  capability = "",
  checkpoint = {},
  fetchImpl = globalThis.fetch,
} = {}) {
  const binding = requireRunArchiveProjectBinding({
    projectId,
    capability,
    serverRunId,
    operation: "appendRunCheckpoint",
    requireServerRun: true,
  });
  return requestProjectApi(`${binding.projectId}/${RUN_ARCHIVE_API_SUFFIX}/${binding.serverRunId}/checkpoints`, {
    method: "POST",
    capability: binding.capability,
    body: { checkpoint },
    fetchImpl,
  });
}

export async function finalizeRunArchive({
  projectId = "",
  serverRunId = "",
  capability = "",
  finalStatus = "stopped",
  fetchImpl = globalThis.fetch,
} = {}) {
  const binding = requireRunArchiveProjectBinding({
    projectId,
    capability,
    serverRunId,
    operation: "finalizeRunArchive",
    requireServerRun: true,
  });
  return requestProjectApi(`${binding.projectId}/${RUN_ARCHIVE_API_SUFFIX}/${binding.serverRunId}/finalize`, {
    method: "POST",
    capability: binding.capability,
    body: { final_status: finalStatus },
    fetchImpl,
  });
}

export async function listRunArchives({
  projectId = "",
  capability = "",
  fetchImpl = globalThis.fetch,
} = {}) {
  const binding = requireRunArchiveProjectBinding({ projectId, capability, operation: "listRunArchives" });
  return requestProjectApi(`${binding.projectId}/${RUN_ARCHIVE_API_SUFFIX}`, { capability: binding.capability, fetchImpl });
}

export async function getRunArchive({
  projectId = "",
  serverRunId = "",
  capability = "",
  fetchImpl = globalThis.fetch,
} = {}) {
  const binding = requireRunArchiveProjectBinding({
    projectId,
    capability,
    serverRunId,
    operation: "getRunArchive",
    requireServerRun: true,
  });
  return requestProjectApi(`${binding.projectId}/${RUN_ARCHIVE_API_SUFFIX}/${binding.serverRunId}`, { capability: binding.capability, fetchImpl });
}

export function summarizeRunArchiveBoundary() {
  return {
    schema: WORKBENCH_RUN_ARCHIVE_SCHEMA,
    append_only: true,
    project_revision_separate_from_run_lineage: true,
    renderer_pixels_stored: false,
    state_hash_formula_change: false,
    server_binding_gate: "G5",
  };
}
