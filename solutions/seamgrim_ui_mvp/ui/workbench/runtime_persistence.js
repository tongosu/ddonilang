// Existing runtime storage capture and restoration; producer, integrity and rejection paths unchanged.
export function createRuntimePersistence({ activeFrame, activeRun, activeRunSourceChanged, alpha2HostPacing, buildFrame, buildRunId, commitRuntimeProject, currentProject, currentVirtualProjectSource, drawPreview, EXECUTION_EVENTS, hostBridge, loadRuntimeProject, nativeSourceAvailable, nativeSourceRequest, newRuntimeRevisionHex, projectExecutionPlanFromPrepared, readStateHash, recipeHash, renderAll, runtimeBridge, runtimeStorageRecord, state, syncExecutionLifecycleCompatibility, transitionExecutionLifecycle, virtualProjectSourceHash, workbenchAuthoredPanelAccess, workbenchHostExecutionState, workbenchLogicalPresentationState, workbenchSourceProfile }) {
function captureRuntimeStorageContext(artifactText,{allowCold=false}={}) {
  // The ordinary VM can remain paused behind a logical presentation. Never
  // save/reopen that stale VM while claiming it is the visible logical run.
  // Logical durable continuation needs its own admitted input/checkpoint owner.
  if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.status().started)
    throw new Error('E_PROJECT_RUNTIME_LOGICAL_CHECKPOINT_REQUIRED: 논리 입력 실행의 저장·이어 실행 경로가 아직 연결되지 않았습니다. 일반 실행 상태로 대신 저장하거나 복원하지 않습니다.');
  const run=activeRun(),vm=state.wasm;
  const cold=allowCold && !run && !vm && state.executionLifecycle.status==='idle';
  if(!cold && (!run || !vm || !run.project_runtime_artifact_text || run.project_runtime_artifact_text!==artifactText))
    throw new Error('E_PROJECT_RUNTIME_OWNER_REQUIRED: 명시 뿌리 작품을 먼저 실행하세요.');
  if((!cold && !['paused','done'].includes(state.executionLifecycle.status)) || state.livePump.inFlight || workbenchHostExecutionState.hostStepInFlight)
    throw new Error('E_PROJECT_RUNTIME_PAUSE_REQUIRED: 한 마디가 끝난 뒤 일시정지하고 저장·재열기 하세요.');
  if(!cold && (state.cursorIndex!==run.frames.length-1 || activeRunSourceChanged()))
    throw new Error('E_PROJECT_RUNTIME_SOURCE_CHANGED: 최신 마디와 저장된 원문에서만 실행 상태를 다룹니다.');
  const projectId=JSON.parse(artifactText).authored_identity.logical_dependency_id;
  const generation=state.executionLifecycle.generation,frame=activeFrame();
  const token=JSON.stringify([state.projectId,run?.run_id,state.cursorIndex,run?.frames.length,frame?.madi,frame?.state_hash]);
  const unchanged=()=>{
    const current=activeRun(),currentFrame=activeFrame();
    if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.status().started || state.wasm!==vm || state.executionLifecycle.generation!==generation || !(cold?state.executionLifecycle.status==='idle':['paused','done'].includes(state.executionLifecycle.status))
      || state.livePump.inFlight || workbenchHostExecutionState.hostStepInFlight || workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifactText
      || JSON.stringify([state.projectId,current?.run_id,state.cursorIndex,current?.frames.length,currentFrame?.madi,currentFrame?.state_hash])!==token)
      throw new Error('E_PROJECT_RUNTIME_CHANGED: 저장·재열기 중 원문이나 실행이 바뀌었습니다. 결과를 게시하지 않습니다.');
  };
  return {run,vm,projectId,unchanged,cold};
}
async function restoredRuntimeCandidate(context,record) {
  // Storage lineage is supplied by the store, never read from the runtime file.
  const candidate=await runtimeBridge.createSession({ddnSourceText:context.run.execution_source,
    sourceProfileIdentity:workbenchSourceProfile(currentProject()),savedProjectArtifactText:record.artifact_text,
    expectedParentRevisionHex:record.parent_hex,historyRevisionHex:record.revision_hex});
  try{context.unchanged();return {candidate,state:await candidate.getStateJson()};}
  catch(error){await candidate.close();throw error;}
}
async function saveCurrentRuntimeProject(artifactText) {
  if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.status().started) {
    const context=await workbenchLogicalPresentationState.logicalPanel.logicalExecution.captureStorageContext();
    const projectId=JSON.parse(artifactText).authored_identity.logical_dependency_id;
    const unchanged=()=>{context.unchanged();if(workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifactText)
      throw new Error('E_PROJECT_RUNTIME_CHANGED: 저장 중 작품 원문이 바뀌었습니다.');};
    const native=await nativeSourceAvailable();unchanged();
    if(native){
      if(!workbenchAuthoredPanelAccess.current.hasNativeSourceContext())throw new Error('E_NATIVE_SOURCE_SAVE_REQUIRED: 먼저 작품 원문을 Native에 저장하세요.');
      const input=await context.inputRequest();unchanged();
      const current=await context.summary();unchanged();
      const result=await nativeSourceRequest('logical_save',artifactText,{input_request:input,state_hash:current.state_hash});unchanged();
      if(result.kind!=='native_logical_world'||result.summary?.state_hash!==current.state_hash||result.summary?.madi_count!==current.madi_count)
        throw new Error('E_NATIVE_SOURCE_LOGICAL_SAVE_MISMATCH: Native 저장 결과가 현재 세계와 다릅니다.');
      workbenchAuthoredPanelAccess.current.rememberNativeSelection(result.authoring_selection);
      return '논리 입력과 세계를 Native에 저장하고 검증했습니다. 종료 후 다시 열어 이어갈 수 있습니다.';
    }
    if(workbenchAuthoredPanelAccess.current.hasNativeSourceContext())throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED: 같은 Native 작업대에서 저장하세요.');
    unchanged();const prior=await loadRuntimeProject(projectId);unchanged();
    const parent=prior?.revision_hex??'',revision=newRuntimeRevisionHex();
    const bytes=hex=>Uint8Array.from(hex.match(/../g)??[],pair=>parseInt(pair,16));
    const exported=await context.export(bytes(parent),bytes(revision));unchanged();
    const record=runtimeStorageRecord(projectId,parent,revision,exported.artifact,exported.checkpoint);
    const checked=await workbenchLogicalPresentationState.logicalPanel.logicalExecution.prepareStoredCheckpoint(record);
    try {
      const current=await context.summary();unchanged();
      if(checked.summary.state_hash!==current.state_hash||checked.summary.madi_count!==current.madi_count)
        throw new Error('E_PROJECT_RUNTIME_SAVE_HASH: 논리 입력 복원 검산이 현재 실행과 다릅니다.');
      await commitRuntimeProject(record);unchanged();
      return '논리 입력 이력과 실행 상태를 함께 저장하고 다시 읽었습니다. Native 내구성 영수증은 아닙니다.';
    }finally{await checked.close();}
  }
  const context=captureRuntimeStorageContext(artifactText);
  const prior=await loadRuntimeProject(context.projectId);context.unchanged();
  const parent=prior?.revision_hex??'',revision=newRuntimeRevisionHex();
  const text=await context.vm.exportSavedProjectArtifact(parent,revision);context.unchanged();
  const record=runtimeStorageRecord(context.projectId,parent,revision,text);
  const checked=await restoredRuntimeCandidate(context,record);
  try{
    const current=await context.vm.getStateJson();context.unchanged();
    if(readStateHash(checked.state)!==readStateHash(current))throw new Error('E_PROJECT_RUNTIME_SAVE_HASH: 복원 검산 결과가 현재 실행과 다릅니다.');
    await commitRuntimeProject(record);context.unchanged();
    return '이 기기에 실행 상태를 저장하고 다시 읽어 검산했습니다. 원문 저장과는 별도입니다.';
  }finally{await checked.candidate.close();}
}
async function reopenCurrentRuntimeProject(artifactText) {
  if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.ownsCurrentSource()) {
    if(workbenchAuthoredPanelAccess.current.hasNativeSourceContext()){
      if(!await nativeSourceAvailable())throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED: 같은 Native 작업대에서 다시 여세요.');
      const binding=workbenchLogicalPresentationState.logicalPanel.logicalExecution.storageInputBinding();
      const native=await nativeSourceRequest('logical_load',artifactText,{input_request:{binding,snapshots:[]}});
      if(workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifactText||native.authored_project_json_utf8!==artifactText||native.kind!=='native_logical_world')
        throw new Error('E_PROJECT_RUNTIME_SOURCE_CHANGED: 다시 여는 중 작품이 바뀌었습니다.');
      const checked=await workbenchLogicalPresentationState.logicalPanel.logicalExecution.prepareStoredCheckpoint({native_logical_summary:native.summary});
      let published=false;
      try{const cleanup=checked.publish();published=true;await cleanup;
        return 'Native 저장본과 실제 입력을 검증하고 화면을 복원했습니다. ▶ 또는 한 마디로 이어갑니다.';
      }finally{if(!published)await checked.close();}
    }
    const projectId=JSON.parse(artifactText).authored_identity.logical_dependency_id;
    const record=await loadRuntimeProject(projectId);
    if(workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifactText)
      throw new Error('E_PROJECT_RUNTIME_CHANGED: 다시 여는 중 작품이 바뀌었습니다.');
    if(record?.checkpoint_bytes) {
      if(JSON.parse(record.artifact_text).project_snapshot?.authored_project_json_utf8!==artifactText)
        throw new Error('E_PROJECT_RUNTIME_SOURCE_CHANGED: 저장 실행의 원문과 현재 작품 자료가 다릅니다.');
      const checked=await workbenchLogicalPresentationState.logicalPanel.logicalExecution.prepareStoredCheckpoint(record);
      let published=false;
      try {
        if(workbenchAuthoredPanelAccess.current.getRunProjectArtifact()!==artifactText)
          throw new Error('E_PROJECT_RUNTIME_CHANGED: 다시 여는 중 작품이 바뀌었습니다.');
        const cleanup=checked.publish();published=true;await cleanup;
        return '논리 입력 실행을 복원했습니다. ▶ 또는 한 마디로 다음 마디를 이어갑니다.';
      }finally{if(!published)await checked.close();}
    }
  }
  const context=captureRuntimeStorageContext(artifactText,{allowCold:true});
  const record=await loadRuntimeProject(context.projectId);context.unchanged();
  if(!record)throw new Error('E_PROJECT_RUNTIME_NOT_SAVED: 이 작품의 저장된 실행 상태가 없습니다.');
  if(context.cold) {
    const source=currentVirtualProjectSource(currentProject());
    if(!source)throw new Error('E_PROJECT_RUNTIME_ROOT_REQUIRED: 명시 뿌리 원문을 먼저 여세요.');
    const prepared=await runtimeBridge.prepareProjectLink(source,'',true,false);context.unchanged();
    const sourceHash=virtualProjectSourceHash(source),plan=projectExecutionPlanFromPrepared(prepared,sourceHash);
    context.run={schema:'seamgrim.run_record.v1',project_id:state.projectId,source_hash:sourceHash,
      project_source_hash:sourceHash,source_kind:'virtual_project_source',execution_source:plan.linked_execution_source,
      project_execution_plan:plan,project_runtime_artifact_text:artifactText,
      authored_artifact_text:workbenchAuthoredPanelAccess.current.getRunArtifact(),bogae_entry:'',recipe_hash:recipeHash(),
      recipe_version:state.recipeVersion,setup:{},frames:[],immutable:true};
  }
  const checked=await restoredRuntimeCandidate(context,record);
  let published=false;
  try{
    const savedSource=JSON.parse(record.artifact_text).project_snapshot?.authored_project_json_utf8;
    if(savedSource!==artifactText)throw new Error('E_PROJECT_RUNTIME_SOURCE_CHANGED: 저장 실행의 원문과 현재 작품 자료가 다릅니다.');
    context.unchanged();
    const run={...context.run,run_id:buildRunId('restored'),parent_run_id:context.run.run_id,
      frames:[],restored_from_revision:record.revision_hex,restored_without_tape_history:true,
      committed_step_count:Number(checked.state.tick_id??0)};
    run.execution_mode_decision=checked.candidate.getExecutionModeDecision();
    run.configured_madi=checked.candidate.getConfiguredMadi();
    if(run.execution_mode_decision?.mode==='live')run.host_pacing=alpha2HostPacing(checked.state.input);
    const restoredLifecycle=transitionExecutionLifecycle(state.executionLifecycle,EXECUTION_EVENTS.RESTORED,
      {runId:run.run_id,mode:run.execution_mode_decision?.mode,completed:run.configured_madi>0 && run.committed_step_count>=run.configured_madi});
    if(!restoredLifecycle.accepted)throw new Error(restoredLifecycle.code);
    const nextMadi=Number(checked.state.tick_id);
    if(!Number.isSafeInteger(nextMadi)||nextMadi<1)
      throw new Error('E_PROJECT_RUNTIME_COMMITTED_MADI_REQUIRED: 완료한 마디가 있는 실행 상태만 다시 열 수 있습니다.');
    const frame=buildFrame(run,checked.state,null);
    // get_state_json describes the next executable tick; step_one describes
    // the tick just committed. Keep Rust state untouched and use the committed
    // number for this restored observation, never execute an extra tick.
    frame.madi=nextMadi-1;run.frames.push(frame);
    // No await inside publication; both VM and visible frame change together.
    state.runs.push(run);state.activeRunId=run.run_id;state.cursorIndex=0;state.wasm=checked.candidate;
    state.executionLifecycle=restoredLifecycle.lifecycle;syncExecutionLifecycleCompatibility();
    published=true;renderAll();drawPreview();
    await context.vm?.close();
    return '저장한 실행 상태를 복원했습니다. 연속 실행 작품은 다음 마디를 이어갈 수 있습니다.';
  }finally{if(!published)await checked.candidate.close();}
}
async function exportCurrentRuntimeProject(artifactText) {
  const context=captureRuntimeStorageContext(artifactText);
  const record=await loadRuntimeProject(context.projectId);context.unchanged();
  if(!record)throw new Error('E_PROJECT_RUNTIME_NOT_SAVED: 먼저 실행 상태를 저장하세요.');
  if(JSON.parse(record.artifact_text).project_snapshot?.authored_project_json_utf8!==artifactText)
    throw new Error('E_PROJECT_RUNTIME_SOURCE_CHANGED: 저장 실행의 원문과 현재 작품 자료가 다릅니다.');
  const checked=await restoredRuntimeCandidate(context,record);
  try {
    const reread=await loadRuntimeProject(context.projectId);context.unchanged();
    if(!reread || reread.revision_hex!==record.revision_hex || reread.artifact_text!==record.artifact_text)
      throw new Error('E_PROJECT_RUNTIME_CHANGED: 저장 자료가 바뀌었습니다. 다시 시도하세요.');
    // Existing RD-686 artifact bytes, not a new recording or a durable receipt.
    await hostBridge.downloadTextFile({text:record.artifact_text,filename:'작품-실행상태.json',mimeType:'application/json'});
    return `저장한 실행 상태를 내보냈습니다. 입력 기록·내구성 영수증은 아닙니다. CLI 이어 실행: ddonilang 작품자료 이어실행 작품-실행상태.json --부모 ${record.parent_hex||'처음'} --판 ${record.revision_hex} --마디수 1`;
  }finally{await checked.candidate.close();}
}
  return { captureRuntimeStorageContext, restoredRuntimeCandidate, saveCurrentRuntimeProject, reopenCurrentRuntimeProject, exportCurrentRuntimeProject };
}
