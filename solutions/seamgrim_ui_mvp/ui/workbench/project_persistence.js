// Existing project save, reopen and feedback transitions; rollback, publication and recovery boundaries unchanged.
export function createProjectPersistence({ activeRun, appendServerRevision, applyExecutionEvent, applySetupRecipe, authoredCompositionHost, backOneMadi, branchFromCursor, buildProjectSnapshot, buildServerProjectArtifact, cancelLivePump, closeAllSideViews, closeViewInstance, createServerProject, currentProject, describePersistenceStatus, editRecoveryGuard, els, EXECUTION_EVENTS, finalizeRunRecord, hydrateProductViewWorkspace, loadLocalFallbackSnapshot, openProjectFromServerSession, primaryExecutionCommand, pushDiagnostic, readWorkbenchCapabilitySession, recordServerConflict, refreshServerRevisions, rememberMyWorkProject, renderAll, renderPersistenceFeedback, renderServerProject, saveCurrentRuntimeProject, saveLocalFallbackSnapshot, selectProjectInputContract, setNextAction, sourceStorageErrorMessage, state, stepForward, stopRun, updateServerStateFromPayload, workbenchAuthoredPanelAccess, workbenchLogicalPresentationState }) {
function beginPersistenceOperation() {
  const generation = Number(state.persistenceOperation?.generation ?? 0) + 1;
  editRecoveryGuard.beginSave(generation);
  state.persistenceOperation = {
    generation,
    completedGeneration: Number(state.persistenceOperation?.completedGeneration ?? 0),
    phase: "saving",
    outcome: "",
  };
  renderPersistenceFeedback();
  return generation;
}

function completePersistenceOperation(generation, phase, outcome) {
  state.persistenceOperation = {
    generation: Math.max(Number(state.persistenceOperation?.generation ?? 0), Number(generation ?? 0)),
    completedGeneration: Number(generation ?? 0),
    phase: String(phase || "saved"),
    outcome: String(outcome || ""),
  };
  if(phase==='saved')editRecoveryGuard.saved(generation);
  else editRecoveryGuard.failed(generation);
  if(phase==='saved'&&['source_only','native_world'].includes(outcome)&&workbenchAuthoredPanelAccess.current.hasNativeSourceContext()) {
    // This is only a UI locator. Reopening always asks the protected Native
    // source owner; a catalogue entry is never a world/permission producer.
    try{
      const nativeLocator=localStorage.getItem('ddn.authored_project.last.v1'),locator=JSON.parse(nativeLocator);
      rememberMyWorkProject({projectId:locator.native_source_project_id,title:currentProject().title,source:'native_source',
        nativeLocator,logicalInput:currentProject().logicalInput??null});
    }catch{pushDiagnostic('W_NATIVE_WORK_LIST_NOT_UPDATED','Native 저장은 완료했지만 내 작업 목록을 갱신하지 못했습니다. 작성 정체 자료에서 저장 자료를 다시 열 수 있습니다.','warning');}
  }
}

async function saveProjectSnapshot() {
  if (state.persistenceOperation?.phase === "saving") {
    throw new Error("E_PROJECT_SAVE_BUSY: 이미 저장 중입니다. 완료될 때까지 기다려 주세요.");
  }
  const persistenceGeneration = beginPersistenceOperation();
  const snapshot = buildProjectSnapshot();
  const sourceSession=state.capabilitySession??readWorkbenchCapabilitySession();
  if(!(state.serverProject.projectId&&sourceSession?.role!=="edit")) {
    try {
      if(currentProject().nativeSaveOnFirstUse&&!workbenchAuthoredPanelAccess.current.getRunProjectArtifact()) {
        if(state.runtimeBusy||state.livePump?.in_flight||state.runtimeStatus==='running')
          throw new Error('E_NATIVE_SOURCE_PAUSE_REQUIRED: 일시정지한 뒤 내 작업으로 저장하세요.');
        const project=currentProject();
        const saved=await workbenchAuthoredPanelAccess.current.createAndSaveSource({title:project.title,logicalInput:project.logicalInput});
        await selectProjectInputContract(currentProject());
        state.persistenceStatus={schema:'seamgrim.workbench.persistence_status.v1',mode:'source_only',server_saved:false,
          local_saved:true,user_label:saved.message};
        completePersistenceOperation(persistenceGeneration,'saved','source_only');renderAll();return saved;
      }
      if(workbenchAuthoredPanelAccess.current.hasSourceEdits()) {
        if(state.runtimeBusy||state.livePump?.in_flight||state.runtimeStatus==='running')
          throw new Error('E_NATIVE_SOURCE_PAUSE_REQUIRED: 일시정지한 뒤 수정 원문을 저장하세요.');
        const sourceSaved=await workbenchAuthoredPanelAccess.current.saveIfRegisteredSource();
        if(sourceSaved){
          state.persistenceStatus={schema:'seamgrim.workbench.persistence_status.v1',mode:'source_only',server_saved:false,
            local_saved:true,user_label:sourceSaved.message,snapshot};
          completePersistenceOperation(persistenceGeneration,'saved','source_only');renderAll();return sourceSaved;
        }
      }
      if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.status().started&&workbenchAuthoredPanelAccess.current.hasNativeSourceContext()){
        const message=await saveCurrentRuntimeProject(workbenchAuthoredPanelAccess.current.getRunProjectArtifact());
        state.persistenceStatus={schema:'seamgrim.workbench.persistence_status.v1',mode:'native_world',server_saved:false,
          local_saved:true,user_label:message,snapshot};
        completePersistenceOperation(persistenceGeneration,'saved','native_world');renderAll();return {kind:'native_logical_world_saved'};
      }
      if(state.wasm?.hasNativeSource?.()) {
        if(state.runtimeBusy||state.livePump?.in_flight||state.runtimeStatus==='running')
          throw new Error('E_NATIVE_SOURCE_PAUSE_REQUIRED: 일시정지한 뒤 저장하세요.');
        const authored=workbenchAuthoredPanelAccess.current.getRunProjectArtifact();
        const saved=await state.wasm.saveNativeCheckpoint(authored);
        if(saved?.kind!=='native_world_saved')throw new Error('E_NATIVE_SOURCE_SAVE_READBACK');
        workbenchAuthoredPanelAccess.current.rememberNativeSelection(saved.authoring_selection);
        state.persistenceStatus={schema:'seamgrim.workbench.persistence_status.v1',mode:'native_world',server_saved:false,
          local_saved:true,user_label:'현재 세계를 Native에 저장하고 다시 확인했습니다.',snapshot};
        completePersistenceOperation(persistenceGeneration,'saved','native_world');renderAll();return saved;
      }
      const sourceSaved=await workbenchAuthoredPanelAccess.current.saveIfRegisteredSource();
      if(sourceSaved) {
        state.persistenceStatus={schema:'seamgrim.workbench.persistence_status.v1',mode:'source_only',server_saved:false,
          local_saved:true,user_label:sourceSaved.message,snapshot};
        completePersistenceOperation(persistenceGeneration,'saved','source_only');renderAll();return sourceSaved;
      }
    }catch(error) {
      // Never turn a Rust/source/CAS failure into the old local fallback's
      // success. Existing source, VM, run records and saved head stay intact.
      state.persistenceStatus={...(state.persistenceStatus??{}),last_error:`원문 저장 실패: ${sourceStorageErrorMessage(error)}`};
      els.saveStatus.textContent=describePersistenceStatus(state.persistenceStatus);
      pushDiagnostic('E_SOURCE_SAVE_FAILED',sourceStorageErrorMessage(error),'error');
      completePersistenceOperation(persistenceGeneration,'error','source_only');throw error;
    }
  }
  let artifact = null;
  try {
    artifact = await buildServerProjectArtifact();
    const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
    if (state.serverProject.projectId && session?.role !== "edit") {
      pushDiagnostic("E_PROJECT_VIEW_WRITE_REJECTED", "보기 권한에서는 저장할 수 없습니다.", "error");
      els.saveStatus.textContent = "보기 권한으로는 저장할 수 없음";
      completePersistenceOperation(persistenceGeneration, "rejected", "view_write_forbidden");
      return {
        schema: "seamgrim.workbench.write_rejected.v1",
        code: "E_PROJECT_VIEW_WRITE_REJECTED",
      };
    }
    let payload;
    if (state.serverProject.projectId && session?.role === "edit" && session?.capability) {
      payload = await appendServerRevision({
        projectId: state.serverProject.projectId,
        capability: session.capability,
        artifact,
        expectedParentRevisionId: state.serverProject.headRevisionId,
      });
    } else {
      payload = await createServerProject({
        artifact,
        title: currentProject().title,
      });
    }
    updateServerStateFromPayload(payload);
    state.persistenceStatus = {
      schema: "seamgrim.workbench.persistence_status.v1",
      mode: "server_project",
      server_saved: true,
      local_saved: false,
      revision_id: state.serverProject.headRevisionId,
      user_label: `서버 저장 ${state.serverProject.headRevisionId}`,
      snapshot,
    };
    await refreshServerRevisions();
    completePersistenceOperation(persistenceGeneration, "saved", "server_project");
    renderAll();
    return payload;
  } catch (err) {
    if (Number(err?.status) === 409) {
      const conflict = await recordServerConflict(err, artifact);
      completePersistenceOperation(persistenceGeneration, "conflict", "server_conflict");
      return conflict;
    }
    if (err?.status && ![404, 405, 501, 503].includes(Number(err.status))) {
      pushDiagnostic(err?.payload?.code ?? "E_PROJECT_SERVER_SAVE", err?.message ?? "서버 저장 실패", "error");
      els.saveStatus.textContent = `서버 저장 실패 ${err?.payload?.code ?? err?.status ?? ""}`.trim();
      completePersistenceOperation(persistenceGeneration, "error", String(err?.payload?.code ?? err?.status ?? "server_save_failed"));
      throw err;
    }
    state.persistenceStatus = saveLocalFallbackSnapshot(snapshot.project_id, snapshot);
    completePersistenceOperation(persistenceGeneration, "saved", "local_fallback");
    els.saveStatus.textContent = describePersistenceStatus(state.persistenceStatus);
    renderServerProject();
    return state.persistenceStatus.snapshot;
  }
}

async function reopenProjectSnapshot() {
  try {
    const source=await workbenchAuthoredPanelAccess.current.reopenIfRegisteredSource(currentProject().authoredArtifact?.authored_identity?.logical_dependency_id??'',{allowLast:true});
    if(source){state.persistenceStatus={schema:'seamgrim.workbench.persistence_status.v1',mode:'source_only',server_saved:false,local_saved:true,user_label:source.message};els.saveStatus.textContent=source.message;return source;}
  }catch(error) {
    pushDiagnostic('E_SOURCE_REOPEN_FAILED',sourceStorageErrorMessage(error),'error');
    state.persistenceStatus={...(state.persistenceStatus??{}),last_error:`원문 재열기 실패: ${sourceStorageErrorMessage(error)}`};
    els.saveStatus.textContent=describePersistenceStatus(state.persistenceStatus);throw error;
  }
  if (state.capabilitySession?.project_id || state.serverProject.projectId) {
    try {
      const serverPayload = await openProjectFromServerSession();
      if (serverPayload) return serverPayload;
    } catch (err) {
      pushDiagnostic(err?.payload?.code ?? "E_PROJECT_SERVER_REOPEN", err?.message ?? "서버 프로젝트 재열기 실패", "error");
    }
  }
  const project = currentProject();
  let parsed = null;
  try {
    parsed = loadLocalFallbackSnapshot(project.id);
  } catch (err) {
    pushDiagnostic("E_PROJECT_REOPEN_CORRUPT", String(err?.message ?? err), "error");
    return null;
  }
  if (!parsed) {
    pushDiagnostic("W_PROJECT_REOPEN_EMPTY", "저장된 프로젝트 snapshot이 없습니다.");
    return null;
  }
  const supersededRun = activeRun();
  cancelLivePump("project_reopened");
  applyExecutionEvent(EXECUTION_EVENTS.REOPENED);
  state.runtimeBusy = false;
  state.runs = [];
  state.activeRunId = "";
  state.cursorIndex = -1;
  state.wasm = null;
  finalizeRunRecord(supersededRun, "stopped").catch((archiveErr) => {
    pushDiagnostic("E_RUN_ARCHIVE_FINALIZE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
  });
  state.mode = parsed.mode === "bogae-main" || parsed.mode === "bogae-full" ? parsed.mode : "ddn-main";
  state.viewWorkspace = hydrateProductViewWorkspace(
    project,
    parsed.view_workspace,
    parsed.view_role ?? parsed.selected_view ?? "",
    "local_snapshot",
  );
  state.activeFilePath = String(parsed.active_file_path ?? project.files[0].path);
  for (const file of Array.isArray(parsed.files) ? parsed.files : []) {
    state.fileTexts.set(String(file.path), String(file.text ?? ""));
  }
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "local_fallback",
    server_saved: false,
    local_saved: true,
    user_label: `이 기기에만 임시 저장 ${parsed.canonical_hash ?? ""}`,
    snapshot: parsed,
  };
  pushDiagnostic("W_PROJECT_REOPENED", `snapshot ${parsed.canonical_hash ?? ""} 재열기 완료`);
  renderAll();
  return parsed;
}

document.querySelector("[data-testid='lifecycle-run']")?.addEventListener("click", () => primaryExecutionCommand());
document.querySelector("[data-testid='lifecycle-step']")?.addEventListener("click", () =>
  workbenchLogicalPresentationState.logicalPanel && (workbenchLogicalPresentationState.logicalPanel.logicalExecution.ownsCurrentSource() || workbenchLogicalPresentationState.logicalPanel.logicalExecution.status().started) ? workbenchLogicalPresentationState.logicalPanel.logicalExecution.command('step') : stepForward());
document.querySelector("[data-testid='lifecycle-stop']")?.addEventListener("click", () => {
  if (workbenchLogicalPresentationState.logicalPanel && (workbenchLogicalPresentationState.logicalPanel.logicalExecution.ownsCurrentSource() || workbenchLogicalPresentationState.logicalPanel.logicalExecution.status().started)) return workbenchLogicalPresentationState.logicalPanel.logicalExecution.command('stop');
  stopRun().catch((err) => {
    pushDiagnostic("E_RUN_STOP_FAILED", String(err?.message ?? err), "error");
    renderAll();
  });
});
document.querySelector("[data-testid='history-back']")?.addEventListener("click", () => backOneMadi());
document.querySelector("[data-testid='history-branch']")?.addEventListener("click", () => branchFromCursor());
document.querySelector("[data-testid='setup-apply']")?.addEventListener("click", () => applySetupRecipe());
els.setupControlList?.addEventListener("input", (event) => {
  const input = event.target?.closest?.("input[data-control-name]");
  if (!input || state.projectControls?.project_id !== state.projectId) return;
  const value = Number(input.value);
  if (Number.isFinite(value)) {
    state.projectControls.control_values = {
      ...state.projectControls.control_values,
      [String(input.dataset.controlName)]: value,
    };
  }
});
els.actionButtonList?.addEventListener("click", (event) => {
  const button = event.target?.closest?.("button[data-action-id]");
  if (button) setNextAction(String(button.dataset.actionId));
});
document.querySelector("[data-testid='side-view-close']")?.addEventListener("click", () => {
  if (authoredCompositionHost.closeAllSides()) return;
  state.viewWorkspace = closeAllSideViews(state.viewWorkspace);
  renderAll();
});
document.querySelector('[data-testid="authored-close-sides"]').addEventListener('click',()=>authoredCompositionHost.closeAllSides());
els.sideViewList?.addEventListener("click", (event) => {
  const button = event.target?.closest?.("[data-view-instance-close]");
  if (!button) return;
  state.viewWorkspace = closeViewInstance(state.viewWorkspace, String(button.dataset.viewInstanceClose ?? ""));
  renderAll();
});

  return { beginPersistenceOperation, completePersistenceOperation, saveProjectSnapshot, reopenProjectSnapshot };
}
