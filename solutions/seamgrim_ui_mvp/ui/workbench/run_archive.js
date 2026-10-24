// Existing input snapshots, frame construction and archive connections; unchanged authority and failure handling.
export function createWorkbenchRunArchive({ activeRun, activeRuntimeSourceText, appendRunCheckpoint, buildCheckpointPayload, buildRunArchiveDraft, claimRunFinalization, createRunArchive, currentProject, currentVirtualProjectSourceHash, els, finalizeRunArchive, getRunArchive, listRunArchives, localProjectEntryPath, primaryViewInstance, pushDiagnostic, readMadi, readStateHash, readWorkbenchCapabilitySession, stableCanonicalJson, stableHash, state, workbenchSourceProfile }) {
function inputSnapshotRequestBinding(project = currentProject(), source = activeRuntimeSourceText(project)) {
  const entryPath = localProjectEntryPath(project);
  const primaryView = primaryViewInstance(state.viewWorkspace);
  const sourceIdentity = currentVirtualProjectSourceHash(project)
    || `candidate:fnv1a32:${stableHash(String(source ?? ""))}`;
  return {
    workspace_identity: String(project?.id ?? ""),
    document_identity: `seamgrim://${encodeURIComponent(String(project?.id ?? ""))}/${encodeURIComponent(entryPath)}`,
    document_version: Number(state.completionDocumentVersion),
    source_profile_identity: workbenchSourceProfile(currentProject()),
    view_identity: String(primaryView?.view_instance_id ?? ""),
    bogae_identity: String(primaryView?.family ?? ""),
    provider_identity: "ddonirang_tool.wasm_runtime",
    request_source_identity: sourceIdentity,
  };
}

function inputSnapshotRequestHash(binding) {
  return `candidate:fnv1a32:input_snapshot_request:${stableHash(stableCanonicalJson(binding))}`;
}

function queuedInputBindingValid(input, binding) {
  if (!input || typeof input !== "object") return true;
  return String(input.workspace_identity ?? "") === binding.workspace_identity
    && String(input.document_identity ?? "") === binding.document_identity
    && Number(input.document_version) === binding.document_version
    && String(input.source_profile_identity ?? "") === binding.source_profile_identity
    && String(input.view_identity ?? "") === binding.view_identity
    && String(input.bogae_identity ?? "") === binding.bogae_identity
    && String(input.provider_identity ?? "") === binding.provider_identity
    && String(input.request_source_identity ?? "") === binding.request_source_identity
    && String(input.input_binding_hash ?? "") === inputSnapshotRequestHash(binding);
}

function makeSnapshot(intent = "none", input = state.nextInput, run = activeRun(), wasmState = null) {
  const receipt = wasmState?.input_receipt;
  const delivered = receipt?.input_snapshot?.input;
  if (!delivered || typeof delivered.dt !== "string"
      || !/^[0-9a-f]{64}$/u.test(receipt?.input_snapshot_sha256 ?? "")) {
    throw new Error("E_INPUT_RECEIPT_MISSING: 실제 계산에 전달된 입력 영수증이 없습니다.");
  }
  const project = currentProject();
  const requestBinding = inputSnapshotRequestBinding(project, run?.execution_source ?? activeRuntimeSourceText(project));
  const snapshot = {
    schema: "seamgrim.input_snapshot.ui.v1",
    madi: receipt.input_snapshot.madi,
    intent,
    input_id: String(input?.input_id ?? ""),
    action_id: String(input?.action_id ?? ""),
    action_map_hash: String(input?.action_map_hash ?? ""),
    keys: delivered.keys_pressed,
    lastKey: delivered.last_key_name,
    pointer: { x: delivered.pointer_x_i32, y: delivered.pointer_y_i32 },
    dtText: delivered.dt,
    ...requestBinding,
    source_identity: String(run?.shared_runtime_result?.source_identity ?? run?.source_hash ?? ""),
    world_state_identity: readStateHash(wasmState, ""),
  };
  return {
    ...snapshot,
    // This seals delivered Rust input, not the UI intent/request metadata above.
    snapshot_hash: receipt.input_snapshot_sha256,
    input_snapshot_sha256: receipt.input_snapshot_sha256,
  };
}

function buildFrame(run, wasmState, snapshot) {
  return {
    frame_id: run.frames.length,
    madi: readMadi(wasmState, run.frames.length),
    state_hash: readStateHash(wasmState, `candidate:missing:${run.frames.length}`),
    source_hash: run.source_hash,
    recipe_hash: run.recipe_hash,
    input_snapshot: snapshot,
    wasm_state: wasmState,
  };
}

function appendFrame(run, wasmState, snapshot) {
  const frame = buildFrame(run, wasmState, snapshot);
  run.frames.push(frame);
  state.cursorIndex = run.frames.length - 1;
  return frame;
}

function runtimeStateSignalsDone(wasmState) {
  return wasmState?.done === true
    || wasmState?.runtime_done === true
    || wasmState?.status === "done"
    || wasmState?.lifecycle === "done";
}

function runReachedDone(run, wasmState) {
  const configuredMadi = Math.max(0, Number(run?.configured_madi ?? 0));
  const committedSteps = Math.max(0, Number(run?.committed_step_count ?? 0));
  return runtimeStateSignalsDone(wasmState)
    || (configuredMadi > 0 && committedSteps >= configuredMadi);
}

function editableProjectSession() {
  const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
  if (!state.serverProject.projectId || !state.serverProject.headRevisionId) return null;
  if (!session?.capability || session.role !== "edit") return null;
  return session;
}

function renderRunArchiveHistory() {
  if (!els.runArchiveList || !els.runArchiveDetail) return;
  const selectedId = String(state.selectedRunArchive?.run?.server_run_id ?? "");
  els.runArchiveList.innerHTML = "";
  if (!state.runArchives.length) {
    const empty = document.createElement("span");
    empty.className = "run-hash";
    empty.textContent = "저장된 실행 없음";
    els.runArchiveList.appendChild(empty);
  } else {
    for (const archive of state.runArchives) {
      const runId = String(archive.server_run_id ?? "");
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.serverRunId = runId;
      button.dataset.testid = `run-archive-open-${runId}`;
      button.setAttribute("aria-current", selectedId === runId ? "true" : "false");
      const identity = document.createElement("span");
      identity.textContent = `${runId}${archive.parent_server_run_id ? ` · parent ${archive.parent_server_run_id}` : ""}`;
      const status = document.createElement("span");
      const checkpointCount = Math.max(0, Number(archive.latest_sequence ?? -1) + 1);
      status.textContent = `${archive.final_status ?? "running"} · cp ${checkpointCount}`;
      button.append(identity, status);
      els.runArchiveList.appendChild(button);
    }
  }

  const selected = state.selectedRunArchive?.run;
  els.runArchiveDetail.hidden = !selected;
  if (!selected) {
    if (els.runArchiveCheckpointList) els.runArchiveCheckpointList.innerHTML = "";
    return;
  }
  const checkpoints = Array.isArray(selected.checkpoints) ? selected.checkpoints : [];
  els.runArchiveDetailTitle.textContent = `${selected.server_run_id} · ${selected.final_status} · checkpoint ${checkpoints.length}개`;
  els.runArchiveCheckpointList.innerHTML = "";
  for (const checkpoint of checkpoints) {
    const row = document.createElement("li");
    const stateHash = String(checkpoint.artifact?.state_hashes?.[0] ?? "-");
    row.dataset.checkpointSequence = String(checkpoint.sequence ?? "");
    row.textContent = `seq ${checkpoint.sequence} · madi ${checkpoint.committed_madi} · state ${stateHash} · sha ${checkpoint.checkpoint_sha256}`;
    els.runArchiveCheckpointList.appendChild(row);
  }
}

async function openRunArchiveDetail(serverRunId) {
  const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
  const projectId = state.serverProject.projectId || session?.project_id || "";
  const capability = session?.capability || state.serverProject.viewCapability || state.serverProject.editCapability || "";
  const requestedRunId = String(serverRunId ?? "").trim();
  if (!projectId || !capability || !requestedRunId) return null;
  const payload = await getRunArchive({
    projectId,
    serverRunId: requestedRunId,
    capability,
  });
  if (state.serverProject.projectId !== projectId && state.capabilitySession?.project_id !== projectId) return null;
  state.selectedRunArchive = payload;
  renderRunArchiveHistory();
  return payload;
}

async function refreshRunArchives() {
  const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
  const projectId = state.serverProject.projectId || session?.project_id || "";
  const capability = session?.capability || state.serverProject.viewCapability || state.serverProject.editCapability || "";
  if (!projectId || !capability) {
    state.runArchives = [];
    state.selectedRunArchive = null;
    renderRunArchiveHistory();
    return [];
  }
  const payload = await listRunArchives({ projectId, capability });
  state.runArchives = Array.isArray(payload.runs) ? payload.runs : [];
  const selectedId = String(state.selectedRunArchive?.run?.server_run_id ?? "");
  if (selectedId && !state.runArchives.some((run) => run.server_run_id === selectedId)) {
    state.selectedRunArchive = null;
  }
  renderRunArchiveHistory();
  return state.runArchives;
}

async function createServerRunArchive(run, { branchFrom = null } = {}) {
  const session = editableProjectSession();
  if (!session) {
    run.server_archive_status = "local_only";
    return null;
  }
  run.server_project_id = state.serverProject.projectId;
  run.server_edit_capability = session.capability;
  const draft = buildRunArchiveDraft({
    projectId: state.serverProject.projectId,
    sourceRevisionId: state.serverProject.headRevisionId,
    expectedHeadRevisionId: state.serverProject.headRevisionId,
    sourceContentSha256: state.serverProject.contentSha256,
    recipeHash: run.recipe_hash,
    seed: `ui:${run.run_id}`,
    parentServerRunId: branchFrom?.server_run_id ?? "",
    branchMadi: branchFrom?.madi ?? null,
    branchStateHash: branchFrom?.state_hash ?? "",
  });
  const payload = await createRunArchive({
    projectId: state.serverProject.projectId,
    capability: session.capability,
    draft,
  });
  const serverRun = payload.run ?? {};
  run.server_run_id = String(serverRun.server_run_id ?? "");
  run.source_revision_id = String(serverRun.source_revision_id ?? state.serverProject.headRevisionId);
  run.source_content_sha256 = String(serverRun.source_content_sha256 ?? state.serverProject.contentSha256);
  run.server_archive_status = "running";
  run.server_checkpoint_count = 0;
  run.server_final_status = String(serverRun.final_status ?? "running");
  await refreshRunArchives();
  return serverRun;
}

async function persistRunCheckpoint(run, frame) {
  if (!run?.server_run_id || !frame) return null;
  const currentSession = editableProjectSession();
  const projectId = String(run.server_project_id || state.serverProject.projectId || "");
  const capability = String(run.server_edit_capability || currentSession?.capability || "");
  if (!projectId || !capability) return null;
  const previous = run.server_latest_checkpoint_sha256 ?? "";
  const checkpoint = buildCheckpointPayload({
    run,
    frame,
    sequence: Number(run.server_checkpoint_count ?? 0),
    previousCheckpointSha256: previous,
  });
  const payload = await appendRunCheckpoint({
    projectId,
    serverRunId: run.server_run_id,
    capability,
    checkpoint,
  });
  const checkpoints = payload.run?.checkpoints ?? [];
  const latest = checkpoints.at(-1) ?? {};
  run.server_latest_checkpoint_sha256 = String(latest.checkpoint_sha256 ?? "");
  run.server_checkpoint_count = checkpoints.length;
  run.server_archive_status = "checkpointed";
  if (state.serverProject.projectId === projectId) await refreshRunArchives();
  return latest;
}

async function finalizeRunRecord(run, finalStatus = "stopped") {
  if (!run) return null;
  if (run.local_finalized || run.server_finalized || run.server_finalization_started) return null;
  run.final_status = finalStatus;
  if (!run.server_run_id) {
    if (["local_only", "error"].includes(run.server_archive_status)) run.local_finalized = true;
    return null;
  }
  const currentSession = editableProjectSession();
  const projectId = String(run.server_project_id || (
    currentSession && state.serverProject.projectId === run.project_id ? state.serverProject.projectId : ""
  ));
  const capability = String(run.server_edit_capability || (
    currentSession && state.serverProject.projectId === run.project_id ? currentSession.capability : ""
  ));
  if (!projectId || !capability) return null;
  if (!claimRunFinalization(run, finalStatus).claimed) return null;
  const payload = await finalizeRunArchive({
    projectId,
    serverRunId: run.server_run_id,
    capability,
    finalStatus,
  });
  run.server_finalized = true;
  run.server_archive_status = "finalized";
  run.server_final_status = String(payload.run?.final_status ?? finalStatus);
  if (state.serverProject.projectId === projectId) await refreshRunArchives();
  return payload.run;
}

async function finalizeActiveRun(finalStatus = "stopped") {
  return finalizeRunRecord(activeRun(), finalStatus);
}

async function finalizeFailedRunArchive(run, diagnostic = {}) {
  if (!run) return null;
  run.server_failure_diagnostic = diagnostic;
  try {
    return await finalizeRunRecord(run, "failed");
  } catch (archiveErr) {
    run.server_archive_status = "failed-finalize-error";
    run.server_final_status = "failed";
    run.server_finalize_error = String(archiveErr?.message ?? archiveErr);
    pushDiagnostic(
      "E_RUN_ARCHIVE_FAILED_FINALIZE_FAILED",
      `실패한 실행 기록을 failed로 종료하지 못했습니다: ${run.server_finalize_error}`,
      "error",
    );
    return null;
  }
}

  return { inputSnapshotRequestBinding, inputSnapshotRequestHash, queuedInputBindingValid, makeSnapshot, buildFrame, appendFrame, runtimeStateSignalsDone, runReachedDone, editableProjectSession, renderRunArchiveHistory, openRunArchiveDetail, refreshRunArchives, createServerRunArchive, persistRunCheckpoint, finalizeRunRecord, finalizeActiveRun, finalizeFailedRunArchive };
}
