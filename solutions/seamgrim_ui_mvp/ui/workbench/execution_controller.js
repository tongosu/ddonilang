// Existing run, step, stop and branch controller; original state transitions and atomic failure bodies retained.
export function createLogicalPresentationPanel({ installLogicalInputPanel, activeSourceText, workbenchAuthoredPanelAccess, nativeSourceAvailable, nativeSourceRequest, renderExecutionControls, els, workbenchLogicalPresentationState, state, renderViewPolicy, currentProject, drawPreview, renderSemanticObservations, createNativeProject }) {
return installLogicalInputPanel({ parent: document.querySelector('[data-testid="logical-record-content"]'), readSource: () => activeSourceText(),
  readAuthoredProject: () => workbenchAuthoredPanelAccess.current.getRunProjectArtifact() || null,
  onNativeCheckpoint:selection=>workbenchAuthoredPanelAccess.current.rememberNativeSelection(selection),
  prepareNativeNewRecord:async(artifact,binding)=>{
    if(!workbenchAuthoredPanelAccess.current.hasNativeSourceContext()||!await nativeSourceAvailable())throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED');
    const options=workbenchAuthoredPanelAccess.current.nativeSourceRunOptions();
    const selection=JSON.parse(options.nativeSourceSelectionJson??'null');
    if(!selection||selection.kind!==1)throw new Error('E_NEW_RECORD_WORLD_REQUIRED: 먼저 세계를 저장하거나 저장한 세계를 여세요.');
    if(workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifact)throw new Error('E_NEW_RECORD_SOURCE_CHANGED');
    const result=await nativeSourceRequest('logical_new_record',artifact,{input_request:{binding,snapshots:[]},authoring_selection:selection});
    if(result.kind!=='native_logical_world'||result.authored_project_json_utf8!==artifact||workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifact)
      throw new Error('E_NEW_RECORD_SOURCE_CHANGED');
    workbenchAuthoredPanelAccess.current.rememberNativeSelection(result.authoring_selection);
    return {artifact_text:result.saved_artifact_json_utf8,parent_hex:result.parent_hex,revision_hex:result.revision_hex,native_logical_summary:result.summary};
  },
  prepareNativeNewRun:async(artifact,binding)=>{
    const options=workbenchAuthoredPanelAccess.current.nativeSourceRunOptions();
    if(!options.nativeSourceNewRun)return null;
    if(!await nativeSourceAvailable())throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED');
    if(workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifact)throw new Error('E_PROJECT_RUNTIME_SOURCE_CHANGED');
    const initial={binding,snapshots:[{madi:0,events:[],held:binding.initial_held}]};
    const result=await nativeSourceRequest('logical_new_run',artifact,{input_request:initial,
      authoring_selection:JSON.parse(options.nativeSourceSelectionJson)});
    if(result.kind!=='native_logical_world'||result.authored_project_json_utf8!==artifact||workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifact)
      throw new Error('E_NATIVE_SOURCE_NEW_LOGICAL_READBACK');
    workbenchAuthoredPanelAccess.current.rememberNativeSelection(result.authoring_selection);
    return result.summary;
  },
  valueControlsParent: document.querySelector('[data-testid="logical-manipulation-controls"]'),
  onStateChange: renderExecutionControls,
  inputRegion: els.canvas,
  viewControls: [els.bogaeStage,document.querySelector('[data-testid="project-save"]'),document.querySelector('[data-testid="authored-view-controls"]'),document.querySelector('[data-testid="logical-record-drawer"]'),document.querySelector('[data-testid="logical-record-open"]'),document.querySelector('[data-testid="project-more-menu"]'),document.querySelector('[data-testid="authored-project-drawer"]')],
  onPresentation: (value,generation) => {
    workbenchLogicalPresentationState.logicalPresentation = value ? {value,generation,projectId:state.projectId} : null;
    renderViewPolicy(currentProject()); drawPreview(); renderSemanticObservations(currentProject());
    // Host status notification precedes this committed response. Refresh the
    // counter after accepting the response, rather than showing the prior tick.
    renderExecutionControls();
  },
  playbackControls: ['lifecycle-run','lifecycle-step','lifecycle-stop'].map(id => document.querySelector(`[data-testid="${id}"]`)),
  openExample: sourceText => createNativeProject('테트리스 입력 작업', 'single', sourceText) });
}

export function createExecutionController({ activeFrame, activeRun, activeRuntimeSourceText, activeSourceText, alpha2HostPacing, appendFrame, applyExecutionEvent, applyRunRecipeToWasm, assertExecutionGeneration, beginCommittedLiveStep, buildCurrentProjectRunEvidence, buildFrame, buildRunId, cancelLivePump, clearDiagnostics, createServerRunArchive, currentProject, currentVirtualProjectSource, els, EXECUTION_EVENTS, EXECUTION_STATES, executionControlPolicy, executionGenerationIsCurrent, executionLifecycleContext, finalizeActiveRun, finalizeFailedRunArchive, finalizeRunRecord, inputSnapshotRequestBinding, livePumpToken, livePumpTokenIsCurrent, localProjectEntryPath, logicalInputRequired, makeSnapshot, markCurrentProjectExecutionStale, openProgressiveDialog, persistRunCheckpoint, prepareCurrentProjectLink, projectDiagnosticFromError, projectExecutionPlanFromPrepared, pushDiagnostic, queuedInputBindingValid, readStateHash, recipeHash, recordVerifiedProjectLockTrust, refreshAuthoritativeSourceReadiness, refreshGeneratedProjectLockForRun, renderAll, requireParsedExecutionModeDecision, resetLivePumpMetricsForRun, runReachedDone, runtimeBridge, scheduleLivePump, sha256HexUtf8, sharedRunSummaryBindingValid, SOURCE_READINESS_DIAGNOSTIC_CODE, sourceReadinessRunOutcome, stableHash, state, stepWithHostStop, virtualProjectSourceHash, workbenchAuthoredPanelAccess, workbenchHostExecutionState, workbenchSourceProfile }) {
function clearFailedExecutionOutput(reason) {
  cancelLivePump(reason);
  state.activeRunId = "";
  state.cursorIndex = -1;
  state.wasm = null;
  markCurrentProjectExecutionStale(reason);
}

async function startRun({ branchFrom = null } = {}) {
  if (state.projectLoadError || state.runtimeBusy || state.sourceAnalysisInFlight) return false;
  const previousDisplay = {projectId: state.projectId, run: activeRun(), frame: activeFrame(),
    activeRunId: state.activeRunId, cursorIndex: state.cursorIndex,
    viewWorkspace: structuredClone(state.viewWorkspace),
    width: els.canvas.width, height: els.canvas.height,
    pixels: els.canvas.getContext("2d").getImageData(0,0,els.canvas.width,els.canvas.height),
    dataset: {...els.canvas.dataset}, renderModel: state.semanticRenderModel,
    semanticPayload: state.semanticViewPayload};
  workbenchHostExecutionState.hostStopNotice = "";
  runtimeBridge.cancelExecution("새 원문으로 실행합니다.");
  state.sourceAnalysisInFlight = true;
  const projectAtRequest = currentProject();
  let sourceReadiness;
  try {
    sourceReadiness = await refreshAuthoritativeSourceReadiness(projectAtRequest);
  } finally {
    state.sourceAnalysisInFlight = false;
  }
  if (sourceReadiness.superseded || currentProject().id !== projectAtRequest.id) return false;
  if (!sourceReadiness.ready) {
    clearFailedExecutionOutput("source_blocked");
    applyExecutionEvent(EXECUTION_EVENTS.START);
    applyExecutionEvent(EXECUTION_EVENTS.PREPARE_FAILED, { reason: "source_blocked" });
    state.runtimeStatus = "source-blocked";
    state.sourceReadinessRunBlocked = true;
    sourceReadinessRunOutcome("run_blocked", {
      code: sourceReadiness.diagnostics?.[0]?.code ?? SOURCE_READINESS_DIAGNOSTIC_CODE,
      source_hash: sourceReadiness.source_hash,
    });
    const diagnostic = sourceReadiness.diagnostics?.[0] ?? null;
    pushDiagnostic(
      String(diagnostic?.code ?? "E_SOURCE_RUN_BLOCKED"),
      String(diagnostic?.message ?? "원문 분석 또는 제품 준비 진단을 먼저 해결해야 실행할 수 있습니다."),
      "error",
    );
    renderAll();
    if (!logicalInputRequired(diagnostic?.code, diagnostic?.message)) openProgressiveDialog(els.projectVerificationDrawer);
    return false;
  }
  state.sourceReadinessRunBlocked = false;
  cancelLivePump("new_run");
  resetLivePumpMetricsForRun();
  const started = applyExecutionEvent(EXECUTION_EVENTS.START, { branch: Boolean(branchFrom) });
  if (!started.accepted) return false;
  workbenchHostExecutionState.pendingBogaeDisplay = previousDisplay;
  workbenchHostExecutionState.rejectedBogaeDisplay = null;
  state.runtimeBusy = true;
  clearDiagnostics();
  const executionGeneration = state.executionLifecycle.generation;
  const projectIdAtStart = state.projectId;
  let run = null;
  let projectSource = null;
  let projectSourceHash = "";
  let projectExecutionPlan = null;
  renderAll();
  try {
    const project = currentProject();
    projectSource = currentVirtualProjectSource(project);
    if (projectSource) {
      projectSource = await refreshGeneratedProjectLockForRun(projectSource);
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
    }
    projectSourceHash = projectSource ? virtualProjectSourceHash(projectSource) : "";
    const source = projectSource ? activeRuntimeSourceText(project) : activeSourceText();
    if (!source.trim()) {
      applyExecutionEvent(EXECUTION_EVENTS.PREPARE_FAILED, { reason: "source_empty" });
      pushDiagnostic("E_SOURCE_EMPTY", "DDN 원문이 비어 있습니다. 실행할 원문을 입력해 주세요.", "error");
      return false;
    }
    if (state.nextInput && !queuedInputBindingValid(state.nextInput, inputSnapshotRequestBinding(project, source))) {
      throw new Error("E_INPUT_SNAPSHOT_BINDING_REQUIRED");
    }
    if (projectSource) {
      markCurrentProjectExecutionStale("new_project_run");
      state.projectExecution.failure = null;
      const prepared = await prepareCurrentProjectLink(projectSource);
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      projectExecutionPlan = projectExecutionPlanFromPrepared(prepared, projectSourceHash);
      recordVerifiedProjectLockTrust(projectSourceHash, projectExecutionPlan);
    }
    const runRecipe = state.pendingRecipe?.project_id === project.id
      && state.pendingRecipe?.authoring_plan_hash === state.projectControls?.authoring_plan_hash
      ? structuredClone(state.pendingRecipe)
      : null;
    run = {
      schema: "seamgrim.run_record.v1",
      run_id: buildRunId(branchFrom ? "branch" : "run"),
      project_id: project.id,
      source_hash: projectSourceHash || `candidate:fnv1a32:${stableHash(source)}`,
      project_source_hash: projectSourceHash,
      source_kind: projectSource ? "virtual_project_source" : "single_file_source",
      bogae_entry: document.querySelector('[data-testid="bogae-entry"]').value.trim(),
      authored_artifact_text: workbenchAuthoredPanelAccess.current.getRunArtifact(),
      project_runtime_artifact_text: workbenchAuthoredPanelAccess.current.getRunProjectArtifact(),
      recipe_hash: recipeHash(),
      recipe_version: state.recipeVersion,
      setup: runRecipe ?? {},
      parent_run_id: branchFrom?.run_id ?? "",
      parent_server_run_id: branchFrom?.server_run_id ?? "",
      parent_madi: branchFrom?.madi ?? null,
      parent_state_hash: branchFrom?.state_hash ?? "",
      frames: [],
      immutable: true,
      execution_mode_decision: null,
      execution_source: projectExecutionPlan?.linked_execution_source ?? source,
      configured_madi: 0,
      committed_step_count: 0,
    };
    if (run.authored_artifact_text && run.bogae_entry) {
      throw new Error("E_AUTHORED_BOGAE_ENTRY: 저장된 보개 시작을 사용합니다. 임시 시작 정의 칸은 비워 주세요.");
    }
    if (projectExecutionPlan) {
      run.project_execution_plan = projectExecutionPlan;
      run.project_prepare_link_hash = projectExecutionPlan.prepare_link_sha256;
      run.project_graph_hash = String(projectExecutionPlan.project_graph?.project_graph_sha256 ?? "");
      run.linked_source_hash = projectExecutionPlan.linked_execution_source_sha256;
      run.execution_mode_decision = projectExecutionPlan.execution_mode_decision;
    }
    state.runs.push(run);
    state.activeRunId = run.run_id;
    state.cursorIndex = -1;
    state.wasm = null;
    try {
      await createServerRunArchive(run, { branchFrom });
    } catch (archiveErr) {
      run.server_archive_status = "error";
      pushDiagnostic("E_RUN_ARCHIVE_CREATE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
    }
    assertExecutionGeneration(executionGeneration, projectIdAtStart);
    let firstFrame = null;
    if (projectSource) {
      state.wasmVmEntryCount += 1;
      if (!String(projectExecutionPlan.target ?? "").trim()) {
        throw new Error("E_PROJECT_START_REQUIRED: 얽음에 실행할 시작 정의가 없습니다.");
      }
      const projectWasm = await runtimeBridge.createSession({
        ddnSourceText: projectExecutionPlan.linked_execution_source,
        projectRuntimeArtifactText: run.project_runtime_artifact_text,
        nativeSourceRequested: workbenchAuthoredPanelAccess.current.hasNativeSourceContext(),
        ...workbenchAuthoredPanelAccess.current.nativeSourceRunOptions(),
        sourceProfileIdentity: workbenchSourceProfile(project),
        projectEntryName: projectExecutionPlan.target,
        bogaeEntryName: run.bogae_entry,
        authoredArtifactText: run.authored_artifact_text,
        cacheBust: "approved-workbench-linked-project-v1",
        setStatus: () => {},
        clearStatusError: () => {},
      });
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      state.wasm = projectWasm;
      await projectWasm.updateLogic(projectExecutionPlan.linked_execution_source);
      workbenchAuthoredPanelAccess.current.rememberNativeSelection(projectWasm.nativeSourceSelection());
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      const vmExecutionModeDecision = requireParsedExecutionModeDecision(
        projectWasm.getExecutionModeDecision(),
      );
      if (vmExecutionModeDecision.mode !== run.execution_mode_decision.mode) {
        throw new Error("E_EXECUTION_MODE_DECISION_DIVERGED: linked envelope와 VM 판정이 다릅니다");
      }
      run.configured_madi = projectWasm.getConfiguredMadi();
      await projectWasm.reset({ keepParams: false });
      await applyRunRecipeToWasm(projectWasm, runRecipe);
      if (run.execution_mode_decision.mode === "live") {
        run.host_pacing = alpha2HostPacing((await projectWasm.getStateJson())?.input);
      }
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      const queuedInput = state.nextInput ? { ...state.nextInput } : null;
      const executedState = await stepWithHostStop(projectWasm, {
        n: 1,
        input: queuedInput
          ? {
              keys: queuedInput.keys,
              lastKey: queuedInput.lastKey,
              px: queuedInput.px,
              py: queuedInput.py,
              dt: queuedInput.dt,
              dtText: queuedInput.dtText,
            }
          : null,
        sourceText: projectExecutionPlan.linked_execution_source,
      });
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      firstFrame = appendFrame(
        run,
        executedState,
        makeSnapshot(branchFrom ? "project_branch_start" : "project_run_start", queuedInput, run, executedState),
      );
      run.committed_step_count = 1;
      if (queuedInput?.input_id && state.nextInput?.input_id === queuedInput.input_id) {
        state.nextInput = null;
      }
      const evidence = await buildCurrentProjectRunEvidence({
        prepared: projectExecutionPlan,
        run,
        frame: firstFrame,
        wasmState: executedState,
      });
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      state.projectExecution.current = {
        schema: "seamgrim.workbench.current_project_execution.v1",
        state: "current_success",
        source_hash: projectSourceHash,
        prepare_link_sha256: projectExecutionPlan.prepare_link_sha256,
        project_graph_hash: String(projectExecutionPlan.project_graph?.project_graph_sha256 ?? ""),
        linked_source_hash: projectExecutionPlan.linked_execution_source_sha256,
        run_id: run.run_id,
        evidence,
      };
      state.projectExecution.failure = null;
    } else {
      state.wasmVmEntryCount += 1;
      const wasm = await runtimeBridge.createSession({
        ddnSourceText: source,
        sourceProfileIdentity: workbenchSourceProfile(project),
        bogaeEntryName: run.bogae_entry,
        authoredArtifactText: run.authored_artifact_text,
        cacheBust: "approved-workbench-v2",
        setStatus: () => {},
        clearStatusError: () => {},
      });
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      state.wasm = wasm;
      await wasm.updateLogic(source);
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      run.execution_mode_decision = requireParsedExecutionModeDecision(
        wasm.getExecutionModeDecision(),
      );
      run.configured_madi = wasm.getConfiguredMadi();
      await wasm.reset({ keepParams: false });
      await applyRunRecipeToWasm(wasm, runRecipe);
      const resetState = await wasm.getStateJson();
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      if (run.execution_mode_decision.mode === "live") {
        run.host_pacing = alpha2HostPacing(resetState?.input);
      }
      const isOneshot = run.execution_mode_decision.mode === "oneshot";
      const queuedInput = state.nextInput ? { ...state.nextInput } : null;
      // A live run also needs one committed frame before its receipt can be
      // bound.  Comparing reset-state (zero ticks) with a run-summary at the
      // configured duration made every live project fail the exact-state gate.
      const committedState = await stepWithHostStop(wasm, {
        n: 1,
        input: queuedInput
          ? {
              keys: queuedInput.keys,
              lastKey: queuedInput.lastKey,
              px: queuedInput.px,
              py: queuedInput.py,
              dt: queuedInput.dt,
              dtText: queuedInput.dtText,
            }
          : null,
        // The bound receipt is issued for the editor's exact source bytes;
        // use those same bytes for this first committed VM frame.
        sourceText: source,
      });
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      if (!committedState?.input_receipt) {
        throw new Error("E_INPUT_RECEIPT_REQUIRED: 실제 계산의 입력 영수증이 필요합니다");
      }
      // A freshly issued temporary source identity is not shared with a
      // second independent VM. Consume Rust's same-invocation receipt, with
      // the unchanged source/input/state binding checks below.
      const temporarySummary = committedState.temporary_committed_summary;
      if (temporarySummary && temporarySummary.binding_scope !== "temporary_same_vm_first_committed_frame") {
        throw new Error("E_WORKBENCH_RUN_RECEIPT_BINDING_REQUIRED");
      }
      const sharedRunSummary = temporarySummary ?? await runtimeBridge.runSummaryWithSourceProfile({
        sourceText: source,
        sourcePath: localProjectEntryPath(project),
        requestedTicks: 1,
        sourceProfileIdentity: workbenchSourceProfile(project),
        inputReceipts: [committedState.input_receipt],
        paramOverrides: committedState.param_overrides,
        bogaeEntryName: run.bogae_entry,
        authoredArtifactText: run.authored_artifact_text,
      });
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      const committedStateHash = readStateHash(committedState, "");
      if (sharedRunSummary.input_receipts?.[0]?.input_snapshot_sha256
          !== committedState.input_receipt.input_snapshot_sha256) {
        throw new Error("E_INPUT_RECEIPT_IDENTITY_MISMATCH: 실제 계산과 영수증의 입력 기록이 다릅니다");
      }
      const sourceBytesSha256 = await sha256HexUtf8(source);
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
      if (!sharedRunSummaryBindingValid(sharedRunSummary, {
        sourceProfileIdentity: workbenchSourceProfile(project),
        sourceBytesSha256,
        stateHash: committedStateHash,
      })) {
        throw new Error("E_WORKBENCH_RUN_RECEIPT_BINDING_REQUIRED");
      }
      run.shared_runtime_result = {
        input_record_sha256: sharedRunSummary.input_record_sha256,
        input_receipts: sharedRunSummary.input_receipts,
        schema: String(sharedRunSummary.schema ?? ""),
        source_profile_identity: String(sharedRunSummary.source_profile_identity ?? ""),
        source_identity: String(sharedRunSummary.execution_envelope?.source_bytes_sha256 ?? ""),
        state_hash: String(sharedRunSummary.state_hash ?? ""),
        output_log_texts: Array.isArray(sharedRunSummary.output_log_texts)
          ? [...sharedRunSummary.output_log_texts]
          : [],
      };
      run.work_cost_receipt = structuredClone(sharedRunSummary.work_cost_receipt);
      firstFrame = appendFrame(
        run,
        committedState,
        makeSnapshot(branchFrom ? "branch_start" : "run_start", queuedInput, run, committedState),
      );
      run.committed_step_count = 1;
      if (queuedInput?.input_id && state.nextInput?.input_id === queuedInput.input_id) {
        state.nextInput = null;
      }
    }
    try {
      await persistRunCheckpoint(run, firstFrame);
    } catch (archiveErr) {
      run.server_archive_status = "checkpoint-error";
      pushDiagnostic("E_RUN_ARCHIVE_CHECKPOINT_FAILED", String(archiveErr?.message ?? archiveErr), "error");
    }
    assertExecutionGeneration(executionGeneration, projectIdAtStart);
    if (run.execution_mode_decision.mode === "oneshot") {
      try {
        await finalizeRunRecord(run, "completed");
      } catch (archiveErr) {
        pushDiagnostic("E_RUN_ARCHIVE_FINALIZE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
      }
      assertExecutionGeneration(executionGeneration, projectIdAtStart);
    }
    if (run.execution_mode_decision.mode === "live") {
      state.livePump.pacing = run.host_pacing;
      state.livePump.previousStartMs = performance.now();
    }
    applyExecutionEvent(EXECUTION_EVENTS.PREPARED, {
      mode: run.execution_mode_decision.mode,
      runId: run.run_id,
    });
    if (run.execution_mode_decision.mode === "live") {
      if (runReachedDone(run, firstFrame?.wasm_state)) {
        applyExecutionEvent(EXECUTION_EVENTS.RUNTIME_DONE, { reason: "configured_madi_reached" });
        try {
          await finalizeRunRecord(run, "completed");
        } catch (archiveErr) {
          pushDiagnostic("E_RUN_ARCHIVE_FINALIZE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
        }
      } else {
        scheduleLivePump();
      }
    }
    sourceReadinessRunOutcome("run_started", {
      source_hash: sourceReadiness.source_hash,
    });
    state.pendingRecipe = null;
    if (state.projectControls?.project_id === project.id) {
      state.projectControls.control_values = Object.fromEntries(
        (state.projectControls.controls ?? []).map((control) => [control.name, control.value]),
      );
    }
    return true;
  } catch (err) {
    const superseded = err?.code === "E_EXECUTION_GENERATION_SUPERSEDED"
      || !executionGenerationIsCurrent(executionGeneration, projectIdAtStart);
    if (superseded) {
      try {
        await finalizeRunRecord(run, "stopped");
      } catch (_) {
        // The current lifecycle owns diagnostics after a generation change.
      }
      return false;
    }
    const diagnostic = projectDiagnosticFromError(err);
    applyExecutionEvent(EXECUTION_EVENTS.PREPARE_FAILED, { reason: "prepare_failed" });
    if (diagnostic.code === "E_BOGAE_SELECTION_REQUIRED") {
      // RD-123/131/139: keep the last committed picture and cursor. The failed
      // VM is not reusable; its request produced no committed frame.
      state.activeRunId = previousDisplay.activeRunId;
      state.cursorIndex = previousDisplay.cursorIndex;
      state.viewWorkspace = previousDisplay.viewWorkspace;
      state.wasm = null;
      workbenchHostExecutionState.rejectedBogaeDisplay = previousDisplay;
    } else {
      clearFailedExecutionOutput("prepare_failed");
    }
    if (projectSource) {
      state.projectExecution.current = null;
      state.projectExecution.failure = {
        schema: "seamgrim.workbench.current_project_execution_failure.v1",
        state: "current_failure",
        source_hash: projectSourceHash,
        diagnostic,
        previous_success_lineage_hash: String(state.projectExecution?.previousSuccess?.evidence?.run_lineage?.lineage_hash ?? ""),
      };
    }
    if (run) {
      run.final_status = "failed";
      run.failure = diagnostic;
      await finalizeFailedRunArchive(run, diagnostic);
    }
    pushDiagnostic(diagnostic.code, diagnostic.message, "error");
    if (!logicalInputRequired(diagnostic.code, diagnostic.message)) openProgressiveDialog(els.projectVerificationDrawer);
    return false;
  } finally {
    if (workbenchHostExecutionState.pendingBogaeDisplay === previousDisplay) workbenchHostExecutionState.pendingBogaeDisplay = null;
    if (executionGenerationIsCurrent(executionGeneration, projectIdAtStart)) {
      state.runtimeBusy = false;
      renderAll();
    }
  }
}

async function commitNextLiveStep({ origin = "auto", token = livePumpToken() } = {}) {
  const allowPaused = origin === "manual";
  const run = activeRun();
  if (!run || run.execution_mode_decision?.mode !== "live") return false;
  if (!livePumpTokenIsCurrent(token, { allowPaused, allowPausing: origin === "auto" })) return false;
  const queuedInput = state.nextInput ? { ...state.nextInput } : null;
  const inputPayload = queuedInput
    ? {
        keys: queuedInput.keys,
        lastKey: queuedInput.lastKey,
        px: queuedInput.px,
        py: queuedInput.py,
        dt: queuedInput.dt,
        dtText: queuedInput.dtText,
      }
    : null;
  try {
    const plan = run.source_kind === "virtual_project_source"
      && run.project_execution_plan
      && typeof run.project_execution_plan === "object"
      ? run.project_execution_plan
      : null;
    const executionSource = plan
      ? String(plan.linked_execution_source ?? "")
      : String(run.execution_source ?? "");
    if (!executionSource.trim()) {
      throw new Error("E_RUN_EXECUTION_SOURCE_MISSING: 봉인된 실행 원문이 없습니다");
    }
    if (queuedInput && !queuedInputBindingValid(
      queuedInput,
      inputSnapshotRequestBinding(currentProject(), executionSource),
    )) {
      throw new Error("E_INPUT_SNAPSHOT_BINDING_REQUIRED");
    }
    const wasm = state.wasm;
    if (!wasm) throw new Error("E_RUN_VM_MISSING: 활성 실행의 봉인 VM이 없습니다");
    const wasmState = await stepWithHostStop(wasm, {
      n: 1,
      input: inputPayload,
      sourceText: executionSource,
    });
    if (!livePumpTokenIsCurrent(token, { allowPaused, allowPausing: origin === "auto" })) return false;

    const snapshot = makeSnapshot(
      queuedInput?.intent ?? (origin === "manual" ? "step" : "auto_step"),
      queuedInput,
      run,
      wasmState,
    );
    const frame = buildFrame(run, wasmState, snapshot);
    let evidence = null;
    if (plan) {
      evidence = await buildCurrentProjectRunEvidence({
        prepared: plan,
        run,
        frame,
        wasmState,
        vmLifecycle: "serial_step(1)",
      });
      if (!livePumpTokenIsCurrent(token, { allowPaused, allowPausing: origin === "auto" })) return false;
    }

    run.frames.push(frame);
    run.committed_step_count = Number(run.committed_step_count ?? 0) + 1;
    state.cursorIndex = run.frames.length - 1;
    state.livePump.stepCommitCount += 1;
    if (queuedInput?.input_id && state.nextInput?.input_id === queuedInput.input_id) {
      state.nextInput = null;
    }
    if (plan && evidence) {
      state.projectExecution.current = {
        schema: "seamgrim.workbench.current_project_execution.v1",
        state: "current_success",
        source_hash: run.project_source_hash,
        prepare_link_sha256: String(plan.prepare_link_sha256 ?? ""),
        project_graph_hash: String(plan.project_graph?.project_graph_sha256 ?? ""),
        linked_source_hash: String(plan.linked_execution_source_sha256 ?? ""),
        run_id: run.run_id,
        evidence,
      };
      state.projectExecution.failure = null;
    }
    try {
      await persistRunCheckpoint(run, frame);
    } catch (archiveErr) {
      run.server_archive_status = "checkpoint-error";
      pushDiagnostic("E_RUN_ARCHIVE_CHECKPOINT_FAILED", String(archiveErr?.message ?? archiveErr), "error");
    }

    if (runReachedDone(run, wasmState)
      && livePumpTokenIsCurrent(token, { allowPaused: true, allowPausing: true })) {
      applyExecutionEvent(EXECUTION_EVENTS.RUNTIME_DONE, { reason: "configured_madi_reached" });
      cancelLivePump("runtime_done");
      try {
        await finalizeRunRecord(run, "completed");
      } catch (archiveErr) {
        pushDiagnostic("E_RUN_ARCHIVE_FINALIZE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
      }
    }
    return true;
  } catch (err) {
    if (!livePumpTokenIsCurrent(token, { allowPaused: true, allowPausing: true })) return false;
    applyExecutionEvent(EXECUTION_EVENTS.RUNTIME_FAILED, { reason: "step_failed" });
    clearFailedExecutionOutput("step_failed");
    cancelLivePump("runtime_failed");
    run.failure = projectDiagnosticFromError(err, "E_WASM_STEP_FAILED");
    await finalizeFailedRunArchive(run, run.failure);
    pushDiagnostic(run.failure.code, run.failure.message, "error");
    renderAll();
    openProgressiveDialog(els.projectVerificationDrawer);
    return false;
  }
}

async function stepForward() {
  const run = activeRun();
  if (run && state.cursorIndex >= 0 && state.cursorIndex < run.frames.length - 1) {
    pushDiagnostic("E_RUN_HISTORY_READ_ONLY", "과거 마디는 읽기 전용입니다. branch를 만들어 계속하세요.", "error");
    cancelLivePump("history_cursor");
    applyExecutionEvent(EXECUTION_EVENTS.HISTORY_OPENED);
    renderAll();
    return false;
  }
  if (!run) {
    pushDiagnostic("E_RUN_STEP_WITHOUT_ACTIVE_RUN", "한 마디 진행할 활성 실행이 없습니다.", "error");
    renderAll();
    return false;
  }
  const requested = applyExecutionEvent(EXECUTION_EVENTS.STEP_REQUESTED, {
    atLatestCursor: state.cursorIndex === run.frames.length - 1,
  });
  if (!requested.accepted) return false;
  const committed = await beginCommittedLiveStep("manual");
  if (committed && state.executionLifecycle.status === EXECUTION_STATES.PAUSED) {
    applyExecutionEvent(EXECUTION_EVENTS.STEP_COMMITTED, { atLatestCursor: true });
    renderAll();
  }
  return committed;
}

function pauseRun() {
  const requested = applyExecutionEvent(EXECUTION_EVENTS.PAUSE_REQUESTED);
  if (!requested.accepted) return false;
  cancelLivePump("pause_requested", { invalidateInFlight: false });
  renderAll();
  if (!state.livePump.inFlight) {
    queueMicrotask(() => {
      const committed = applyExecutionEvent(EXECUTION_EVENTS.PAUSE_COMMITTED);
      if (committed.accepted) renderAll();
    });
  }
  return true;
}

function resumeRun() {
  const run = activeRun();
  if (!run) return false;
  if (state.cursorIndex < run.frames.length - 1) {
    pushDiagnostic("E_RUN_HISTORY_RESUME_REJECTED", "과거 마디에서 같은 run을 재개할 수 없습니다.", "error");
    applyExecutionEvent(EXECUTION_EVENTS.HISTORY_OPENED);
    renderAll();
    return false;
  }
  const resumed = applyExecutionEvent(EXECUTION_EVENTS.RESUME, { atLatestCursor: true });
  if (resumed.accepted) scheduleLivePump();
  renderAll();
  return resumed.accepted;
}

async function stopRun() {
  const requested = applyExecutionEvent(EXECUTION_EVENTS.STOP_REQUESTED);
  if (!requested.accepted) return false;
  const inFlightStep = state.livePump.inFlightPromise;
  cancelLivePump("stop_requested");
  runtimeBridge.cancelExecution("사용자가 실행을 중지했습니다.");
  const stopGeneration = state.executionLifecycle.generation;
  const projectIdAtStop = state.projectId;
  state.runtimeBusy = true;
  renderAll();
  try {
    if (inFlightStep) await inFlightStep.catch(() => false);
    await finalizeActiveRun("stopped");
  } catch (archiveErr) {
    pushDiagnostic("E_RUN_ARCHIVE_FINALIZE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
  } finally {
    if (executionGenerationIsCurrent(stopGeneration, projectIdAtStop)) {
      applyExecutionEvent(EXECUTION_EVENTS.STOP_COMMITTED, { finalStatus: "stopped" });
      state.runtimeBusy = false;
      renderAll();
    }
  }
  return true;
}

function backOneMadi() {
  const run = activeRun();
  const policy = executionControlPolicy(state.executionLifecycle, executionLifecycleContext());
  if (!run || !policy.canBack) return false;
  cancelLivePump("history_cursor");
  state.cursorIndex = Math.max(0, state.cursorIndex - 1);
  applyExecutionEvent(EXECUTION_EVENTS.HISTORY_OPENED);
  pushDiagnostic("W_RUN_HISTORY_READ_ONLY", "한 마디 뒤로 이동했습니다. 현재 run은 읽기 전용입니다.");
  renderAll();
  return true;
}

async function branchFromCursor() {
  const run = activeRun();
  const frame = activeFrame();
  const policy = executionControlPolicy(state.executionLifecycle, executionLifecycleContext());
  if (!run || !frame || !policy.canBranch) return false;
  try {
    await finalizeActiveRun("stopped");
  } catch (archiveErr) {
    pushDiagnostic("E_RUN_ARCHIVE_FINALIZE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
  }
  return startRun({
    branchFrom: {
      run_id: run.run_id,
      server_run_id: run.server_run_id ?? "",
      madi: frame.madi,
      state_hash: frame.state_hash,
    },
  });
}

  return { clearFailedExecutionOutput, startRun, commitNextLiveStep, stepForward, pauseRun, resumeRun, stopRun, backOneMadi, branchFromCursor };
}
