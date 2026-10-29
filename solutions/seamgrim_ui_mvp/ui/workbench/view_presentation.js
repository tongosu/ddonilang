// Existing view controls, observation and diagnostic presentation; original state references and bodies retained.
import { appendDiagnosticPresentation, diagnosticSummary } from './diagnostic_display.js';
import {runStatusPresentation,persistencePresentation,displayMessage} from './display_locale.js';
import {extractObservationOutputLogFromState} from '../seamgrim_runtime_state.js';
export function createViewPresentation({ activeFrame, activeRun, activeRuntimeSourceText, applyExplorationPreset, applySourceReadinessFixIt, authoredCompositionHost, buildCommittedCanonicalViewHistory, buildCommittedFrameCanonicalViewPayloads, buildCommittedSemanticViewPayload, currentLogicalPresentation, currentProject, currentVirtualProjectSourceHash, describePersistenceStatus, els, escapeHtml, EXECUTION_STATES, executionStatusLabel, extractObservationOutputRowsFromState, formatNumber, getSourceReadinessCandidate, loadLocalFallbackSnapshot, localProjectRunEvidenceFromProject, overlayViewInstances, primaryViewInstance, pushDiagnostic, reconcileViewWorkspace, renderAll, renderCommittedSemanticView, renderCurrentMadi, resizeCanvasToDisplaySize, selectCommittedOutputView, selectViewTarget, sideViewInstances, sourceReadinessViewOnly, stableHash, state, viewRoleCapabilities, workbenchHostExecutionState }) {
let lastRunStatus = null;
let lastPersistenceFeedback = null;
function paintPersistenceFeedback() {
  if (!lastPersistenceFeedback) return;
  const locale = document.documentElement.dataset.displayLocale ?? 'ko';
  const shown = persistencePresentation(lastPersistenceFeedback,locale);
  const saveButton = document.querySelector("[data-testid='project-save']");
  if (saveButton) saveButton.textContent = shown.buttonText;
  els.saveStatus.textContent = shown.text;
  els.saveStatus.title = shown.title;
  if (shown.fallback) els.saveStatus.dataset.displayFallback = 'ko';
  else delete els.saveStatus.dataset.displayFallback;
}
function paintRunStatus() {
  if (!els.runStatus || !lastRunStatus || els.root?.dataset.executionOwner === 'logical_input') return;
  const locale = document.documentElement.dataset.displayLocale ?? 'ko';
  const shown = runStatusPresentation(lastRunStatus, locale);
  els.runStatus.textContent = shown.text;
  if (shown.fallback) els.runStatus.dataset.displayFallback = 'ko';
  else delete els.runStatus.dataset.displayFallback;
}
function rememberRunStatus(kind, code = undefined) {
  lastRunStatus = Object.freeze({kind, code,
    lifecycle:Object.freeze({...state.executionLifecycle}),
    stepInFlight:Boolean(workbenchHostExecutionState.hostStepInFlight)});
  // The normal render order may subsequently display the logical-input owner.
  const locale = document.documentElement.dataset.displayLocale ?? 'ko';
  const shown = runStatusPresentation(lastRunStatus, locale);
  els.runStatus.textContent = shown.text;
  if (shown.fallback) els.runStatus.dataset.displayFallback = 'ko';
  else delete els.runStatus.dataset.displayFallback;
}
function renderMode() {
  els.layout.dataset.mode = state.mode;
  document.querySelector('.app-frame').dataset.experimentMode = String(state.mode === 'bogae-full');
  document.querySelectorAll("button[data-mode]").forEach((button) => {
    button.classList.toggle("active", button.dataset.mode === state.mode);
    button.setAttribute("aria-selected", button.dataset.mode === state.mode ? "true" : "false");
  });
  const sourceWidth = Number.parseFloat(els.layout.style.getPropertyValue("--source-width"))
    || (state.mode === "bogae-main" ? 40 : 60);
  els.ratioLabel.textContent = state.mode === "bogae-full" ? "보개 전체 보기"
    : `원문 ${Math.round(sourceWidth)} / 보개 ${Math.round(100 - sourceWidth)}`;
}

function buildViewInstanceSemanticPayload(project, instance) {
  const overlays = instance.role === "primary" ? overlayViewInstances(state.viewWorkspace) : [];
  const run = activeRun();
  const frame = activeFrame();
  return buildCommittedSemanticViewPayload({
    project,
    run,
    frame,
    viewConfig: {
      primary_view: instance.family ?? project.primaryView,
      role: instance.role,
      mode: state.mode,
      view_instance_id: instance.view_instance_id,
      target_view_instance_id: instance.target_view_instance_id,
      overlay_instance_ids: overlays.map((overlay) => overlay.view_instance_id),
      overlay_enabled: overlays.length > 0,
      focus_target: instance.target,
    },
    canonicalViewPayloads: buildCommittedFrameCanonicalViewPayloads(run, frame),
    canonicalViewHistory: buildCommittedCanonicalViewHistory(run, frame, instance.family ?? project.primaryView),
  });
}

function setViewActionState(button, { visible, enabled, reason = "", active = false, pressed = null }) {
  if (!button) return;
  if (!button.dataset.enabledTitle) button.dataset.enabledTitle = button.title || "";
  button.hidden = !visible;
  button.disabled = !enabled;
  button.dataset.disabledReason = enabled ? "" : reason;
  button.title = enabled ? button.dataset.enabledTitle : reason;
  button.classList.toggle("active", active);
  if (pressed !== null) button.setAttribute("aria-pressed", pressed ? "true" : "false");
}

function renderSideViewInstances(project, instances) {
  if (!els.sideViewList) return;
  els.sideViewList.innerHTML = instances.map((instance, index) => {
    const roleLabel = instance.role === "magnifier" ? "돋보기" : `곁보기 ${index + 1}`;
    const targetLabel = instance.target?.target_id ? `관찰 대상 ${instance.target.target_id}` : "확정된 관찰값";
    return `<section class="side-view-instance" data-view-instance-id="${escapeHtml(instance.view_instance_id)}" data-view-instance-role="${escapeHtml(instance.role)}" data-temporary="${instance.temporary ? "true" : "false"}">
      <div class="side-view-instance-header"><strong>${escapeHtml(roleLabel)}</strong><button type="button" class="panel-action" data-view-instance-close="${escapeHtml(instance.view_instance_id)}" title="이 보기 닫기" aria-label="${escapeHtml(roleLabel)} 닫기">닫기</button></div>
      <canvas width="360" height="220" data-view-instance-canvas="${escapeHtml(instance.view_instance_id)}" aria-label="${escapeHtml(roleLabel)} canvas"></canvas>
      <span class="side-view-instance-meta">${escapeHtml(targetLabel)} · ${escapeHtml(instance.view_instance_id)}</span>
    </section>`;
  }).join("");
  instances.forEach((instance) => {
    const canvas = els.sideViewList.querySelector(`[data-view-instance-canvas='${instance.view_instance_id}']`);
    if (!canvas) return;
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const { width, height } = resizeCanvasToDisplaySize(canvas);
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "#f8fbf7";
    ctx.fillRect(0, 0, width, height);
    const payload = buildViewInstanceSemanticPayload(project, instance);
    const model = renderCommittedSemanticView(ctx, width, height, payload);
    canvas.dataset.viewInstanceId = instance.view_instance_id;
    canvas.dataset.viewInstanceRole = instance.role;
    canvas.dataset.targetViewInstanceId = instance.target_view_instance_id;
    canvas.dataset.committedStateHash = payload.committed_frame?.state_hash ?? "";
    canvas.dataset.semanticPayloadHash = payload.semantic_payload_hash;
    canvas.dataset.rendererStatus = model.status;
    canvas.dataset.sideGraphSamples = String(model.side_graph_samples ?? 0);
    canvas.dataset.sideGraphPanels = String(model.side_graph_panels ?? 0);
  });
}

// Initial-condition authoring only: dragging edits a visible literal. It does
// not mutate the committed world or synthesize an execution input snapshot.
let initialConditionDrag = null;
function initialDragPoint(event, view) {
  const rect=els.canvas.getBoundingClientRect(), camera=authoredCompositionHost.camera;
  const width=els.canvas.width,height=els.canvas.height,zoom=camera?.zoom ?? 1;
  const px=((event.clientX-rect.left)*width/rect.width-width/2-(camera?.x ?? 0))/zoom+width/2;
  const py=((event.clientY-rect.top)*height/rect.height-height/2-(camera?.y ?? 0))/zoom+height/2;
  const [xmin,xmax]=view.horizontal.map(raw=>Number(BigInt(raw))/4294967296);
  const [ymin,ymax]=view.vertical.map(raw=>Number(BigInt(raw))/4294967296);
  const scale=Math.min((width-36)/(xmax-xmin),(height-36)/(ymax-ymin));
  return [xmin+(px-(width-(xmax-xmin)*scale)/2)/scale,
    ymin+((height+(ymax-ymin)*scale)/2-py)/scale];
}
els.canvas.addEventListener('pointerdown',event=>{
  if(event.button!==0)return; // Middle-button navigation never edits initial conditions.
  const lab=currentProject().course?.lab_id ?? currentProject().id;
  if(!['alpha3_double_pendulum_lab','alpha4_lrl_lab'].includes(lab) || els.sourceEditor.readOnly) return;
  const view=activeFrame()?.wasm_state?.bogae_projection?.candidates?.[0]?.artifact?.view;
  if(view?.kind!=='scene') return;
  const point=initialDragPoint(event,view);
  if(lab==='alpha4_lrl_lab') {
    const center=view.elements[0]?.center?.map(raw=>Number(BigInt(raw))/4294967296);
    if(!center || Math.hypot(point[0]-center[0],point[1]-center[1])>=0.18) return;
    initialConditionDrag={pointer:event.pointerId,view,lab,anchor:center};
    els.canvas.setPointerCapture(event.pointerId);return;
  }
  const centers=[3,4].map(index=>view.elements[index]?.center?.map(raw=>Number(BigInt(raw))/4294967296));
  const index=centers.findIndex(center=>center && Math.hypot(point[0]-center[0],point[1]-center[1])<0.18);
  if(index<0) return;
  initialConditionDrag={pointer:event.pointerId,view,index,anchor:index===0?[0,0]:centers[0]};
  els.canvas.setPointerCapture(event.pointerId);
});
els.canvas.addEventListener('pointerup',event=>{
  const drag=initialConditionDrag;
  if(!drag || drag.pointer!==event.pointerId) return;
  initialConditionDrag=null;
  const point=initialDragPoint(event,drag.view);
  if(drag.lab==='alpha4_lrl_lab') {
    try {
      const source=applyExplorationPreset(els.sourceEditor.value,
        {발사설정:{min:0,max:1,integer:true},발사가로속도:{min:-3,max:3},발사세로속도:{min:0.2,max:3}},
        {발사설정:1,발사가로속도:Number((point[0]-drag.anchor[0]).toFixed(6)),발사세로속도:Number((point[1]-drag.anchor[1]).toFixed(6))});
      els.sourceEditor.value=source;els.sourceEditor.dispatchEvent(new Event('input',{bubbles:true}));
      pushDiagnostic('W_EXPLORATION_SOURCE_EDIT','발사 속도를 원문에 반영했습니다. ▶로 다시 계산하세요.');
    } catch(error) {pushDiagnostic('E_EXPLORATION_PRESET',String(error.message),'error');renderAll();}
    return;
  }
  const angle=Math.atan2(point[0]-drag.anchor[0],-(point[1]-drag.anchor[1]));
  if(!Number.isFinite(angle)) return;
  try {
    const source=applyExplorationPreset(els.sourceEditor.value,currentProject().exploration.parameters,
      {[drag.index===0?'초기각1':'초기각2']:Number(angle.toFixed(6))});
    els.sourceEditor.value=source;els.sourceEditor.dispatchEvent(new Event('input',{bubbles:true}));
    pushDiagnostic('W_EXPLORATION_SOURCE_EDIT','추의 초기각을 원문에 반영했습니다. ▶로 다시 계산하세요.');
  } catch(error) {
    pushDiagnostic('E_EXPLORATION_PRESET',String(error.message),'error');renderAll();
  }
});
els.canvas.addEventListener('pointercancel',()=>{initialConditionDrag=null;});

function renderViewPolicy(project) {
  // Preparing/rejected requests retain the last committed presentation, including
  // side instances and their local hidden state. Never carry it across projects.
  const frozenDisplay = workbenchHostExecutionState.pendingBogaeDisplay ?? workbenchHostExecutionState.rejectedBogaeDisplay;
  const logical = currentLogicalPresentation();
  const presentationFrame = logical ? {wasm_state:logical.value} : frozenDisplay?.projectId === state.projectId
    ? frozenDisplay.frame : activeFrame();
  const presentationRun=frozenDisplay?.projectId===state.projectId ? frozenDisplay.run : activeRun();
  const authored = authoredCompositionHost.render({projectId:logical ? `${state.projectId}:logical:${logical.generation}` : `${state.projectId}:${presentationRun?.run_id ?? "none"}`,
    wasmState:presentationFrame?.wasm_state, list:els.sideViewList, stage:els.bogaeStage,
    toolbar:document.querySelector('[data-testid="authored-view-controls"]'),canvas:els.canvas});
  document.querySelector('[data-testid="authored-close-sides"]').hidden=!authored;
  els.viewRoleSummary.hidden=Boolean(authored);
  const entry=document.querySelector('[data-testid="bogae-entry"]');
  const entryDetails=entry.closest('details');
  const entryWasHidden=entryDetails.hidden;
  const requiresEntry=Boolean(presentationFrame?.wasm_state?.bogae_projection?.candidates?.length>1
    || state.diagnostics.some(row=>row.code==="E_BOGAE_SELECTION_REQUIRED"));
  entryDetails.hidden=!(requiresEntry || entry.value.trim());
  if (requiresEntry && entryWasHidden) entryDetails.open=true;
  els.bogaeSourceAccess.hidden=Boolean(authored);
  document.querySelector('[data-testid="side-view-close"]').hidden=Boolean(authored);
  const outputView = !authored && presentationFrame
    ? selectCommittedOutputView(buildCommittedFrameCanonicalViewPayloads(presentationRun,presentationFrame),
        presentationFrame,presentationRun?.source_hash) : null;
  const textOnly = outputView?.family === 'console';
  els.viewPrimary.closest('.view-roles').hidden=Boolean(textOnly);
  if (textOnly) {
    els.viewRoleSummary.hidden=true;
    els.bogaeSourceAccess.hidden=true;
    document.querySelector('[data-testid="side-view-close"]').hidden=true;
    els.bogaeStage.dataset.sideOpen='false';
    els.sideViewList.innerHTML='';
    document.querySelector('[data-testid="example-guide"]').textContent='';
    return;
  }
  if (authored) {
    for (const button of [els.viewSide,els.viewOverlay,els.viewMagnifier]) button.hidden=true;
    els.viewPrimary.hidden=true;
    els.bogaeSourceAccess.hidden=true;
    document.querySelector('[data-testid="side-view-close"]').hidden=true;
    els.sideViewTitle.textContent=`열린 곁보기 ${authored.sideCount}`;
    els.sideViewBody.textContent="원문이 선언한 곁보기입니다. 닫아도 관계와 화면 상태를 보존합니다.";
    els.viewRoleSummary.textContent=`원문 관계 ${authored.relationCount}`;
    document.querySelector('[data-testid="example-guide"]').textContent="원문 보개짜임 선언대로 표시합니다.";
    return;
  }
  if (logical) {
    for (const button of [els.viewPrimary,els.viewSide,els.viewOverlay,els.viewMagnifier]) button.hidden=true;
    els.viewRoleSummary.hidden=true;
    els.bogaeSourceAccess.hidden=true;
    document.querySelector('[data-testid="side-view-close"]').hidden=true;
    document.querySelector('[data-testid="spatial-trail"]').hidden=true;
    document.querySelector('[data-testid="example-guide"]').textContent='';
    return;
  }
  const guides = {
    line: "정적인 직선은 한 번 계산합니다. a와 b를 고쳐 다시 실행하고 기울기와 절편을 비교해 보세요.",
    grid: "가로 10칸·세로 20칸입니다. 0.5초 재생으로 아래로 떨어져 바닥에 고정되는 모습을 관찰하세요. 일시정지 후 한 마디씩 진행하고 방향키로 움직일 수 있습니다. 완성된 테트리스 제품은 아닙니다.",
    pendulum: "일시정지하고 겹을 켜면 수직 기준선과 현재 각도가 나타납니다. 곁에서는 같은 마디의 각도·각속도·에너지 기록을 비교합니다.",
    projectile: "지나온 자리는 실행이 확정된 위치를 화면에서 연결한 기록이며 예측 궤도가 아닙니다. 겹은 현재 속도 벡터, 곁은 위치와 속도 기록입니다.",
  };
  // The fallback renderer kind is not an authored lesson identity. Imported
  // and newly created projects must not inherit the built-in linear guide.
  document.querySelector('[data-testid="example-guide"]').textContent = project.viewGrade === "output-only"
    ? "" : (guides[project.primaryView] ?? "");
  document.querySelector('[data-testid="spatial-trail"]').hidden = project.primaryView !== "projectile";
  const committedFrame = activeFrame();
  state.viewWorkspace = reconcileViewWorkspace(state.viewWorkspace, project,
    committedFrame ? { ...committedFrame, run_id: activeRun()?.run_id } : null);
  const policy = project.viewPolicy ?? {};
  const capabilities = viewRoleCapabilities(project, state.semanticViewPayload, state.semanticRenderModel);
  const sideInstances = sideViewInstances(state.viewWorkspace);
  const overlays = overlayViewInstances(state.viewWorkspace);
  const selectedTarget = state.viewWorkspace.selected_target;
  const primary = primaryViewInstance(state.viewWorkspace);
  setViewActionState(els.viewPrimary, {
    visible: true,
    enabled: true,
    active: state.viewWorkspace.active_view_instance_id === primary?.view_instance_id,
  });
  els.viewPrimary.textContent = "으뜸 1";
  els.viewPrimary.title = String(policy.primary ?? "으뜸보기");
  setViewActionState(els.viewSide, {
    visible: capabilities.side.supported || sideInstances.some((instance) => instance.role === "side"),
    enabled: capabilities.side.supported && capabilities.side.enabled,
    reason: capabilities.side.reason,
  });
  els.viewSide.textContent = sideInstances.some((instance) => instance.role === "side") ? "곁보기 열림" : "곁보기 열기";
  if (capabilities.side.supported) els.viewSide.title = String(policy.side ?? "확정된 관찰값");
  setViewActionState(els.viewOverlay, {
    visible: capabilities.overlay.supported || overlays.length > 0,
    enabled: overlays.length > 0 || (capabilities.overlay.supported && capabilities.overlay.enabled),
    reason: capabilities.overlay.reason,
    active: overlays.length > 0,
    pressed: overlays.length > 0,
  });
  els.viewOverlay.textContent = overlays.length ? "겹 끄기" : "겹 켜기";
  if (capabilities.overlay.supported) els.viewOverlay.title = String(policy.overlay ?? "");
  setViewActionState(els.viewMagnifier, {
    visible: capabilities.magnifier.supported || sideInstances.some((instance) => instance.role === "magnifier"),
    enabled: capabilities.magnifier.supported && capabilities.magnifier.enabled && Boolean(selectedTarget),
    reason: selectedTarget ? capabilities.magnifier.reason : "으뜸보기에서 움직이는 대상을 먼저 선택하세요.",
  });
  els.bogaeStage.dataset.sideOpen = sideInstances.length ? "true" : "false";
  els.bogaeStage.dataset.viewInstanceCount = String(state.viewWorkspace.instances.length);
  els.bogaeStage.dataset.sideInstanceCount = String(sideInstances.length);
  els.bogaeStage.dataset.overlayInstanceCount = String(overlays.length);
  els.bogaeStage.dataset.selectedTarget = selectedTarget?.target_id ?? "";
  els.sideViewTitle.textContent = `열린 곁보기 ${sideInstances.length}`;
  els.sideViewBody.textContent = "";
  const layoutText = state.mode === "ddn-main"
    ? "원문 중심"
    : state.mode === "bogae-main"
      ? "보개 중심"
      : "보개 전체";
  els.layoutPolicy.textContent = layoutText;
  els.viewRoleSummary.textContent = sideInstances.length || overlays.length
    ? `곁보기 ${sideInstances.filter((instance) => instance.role === "side").length} · 겹보기 ${overlays.length} · 돋보기 ${sideInstances.filter((instance) => instance.role === "magnifier").length}` : "";
  renderSideViewInstances(project, sideInstances);
}

function selectPrimaryCanvasTarget(event) {
  if (els.canvas?.dataset.authoredBogaeStatus==='ready') return false;
  const frame = activeFrame();
  const model = state.semanticRenderModel;
  const canvas = els.canvas;
  if (!frame || model?.status !== "ready" || !canvas) return false;
  const projection = model.canvas_projection ?? {};
  const targetPoint = projection.canvas_bob ?? projection.canvas_current ?? null;
  if (!targetPoint) {
    state.viewWorkspace = selectViewTarget(state.viewWorkspace, null);
    renderAll();
    return false;
  }
  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / Math.max(1, rect.width);
  const scaleY = canvas.height / Math.max(1, rect.height);
  const pointer = {
    x: (event.clientX - rect.left) * scaleX,
    y: (event.clientY - rect.top) * scaleY,
  };
  const distance = Math.hypot(pointer.x - targetPoint.x, pointer.y - targetPoint.y);
  const threshold = 56 * Math.max(scaleX, scaleY);
  if (distance > threshold) {
    state.viewWorkspace = selectViewTarget(state.viewWorkspace, null);
    renderAll();
    return false;
  }
  const targetKind = model.family === "pendulum" ? "bob" : "current_point";
  state.viewWorkspace = selectViewTarget(state.viewWorkspace, {
    target_id: `${model.family}:${targetKind}:${frame.state_hash}`,
    kind: "committed_renderer_current",
    family: model.family,
    run_id: activeRun()?.run_id ?? "",
    frame_id: frame.frame_id,
    madi: frame.madi,
    state_hash: frame.state_hash,
    canvas_point: { x: targetPoint.x, y: targetPoint.y },
  });
  renderAll();
  return true;
}

function appendCommittedExactOutput(frame) {
  const committed=frame?.wasm_state;
  // Opaque rational resources must not be decoded/coerced in JavaScript.
  // Show the existing committed Rust output text verbatim; this is display,
  // never a numeric type, mathematical participation, or observation issuer.
  const exact=Object.values(committed?.resources?.value_json ?? {}).some(value=>
    value?.type==='rational_v1'&&value.version===1&&value.resource_value_tag==='10');
  const display=committed?.output_display;
  // Ordinary output remains visible when no observations were declared.
  const noObservations=Number(els.semanticObservations.dataset.rowCount||0)===0;
  const outputRows=(exact||noObservations)&&display?.schema==='ddn.output-display.v1'
    &&display.world_state_hash===frame?.state_hash
    ?extractObservationOutputLogFromState(committed).filter(row=>row.kind==='output'&&row.tick===frame.madi).slice(-4):[];
  if (noObservations && outputRows.length) els.semanticObservations.innerHTML='';
  const label=displayMessage('output',document.documentElement.dataset.displayLocale ?? 'ko').text;
  els.semanticObservations.innerHTML+=outputRows.map(row=>`<span class="semantic-observation" data-committed-output="${escapeHtml(row.text)}" data-committed-madi="${frame.madi}" data-state-hash-participation="0"><strong data-committed-output-label>${escapeHtml(label)}</strong><span>${escapeHtml(row.text)}</span></span>`).join('');
  els.semanticObservations.dataset.rowCount=String(Number(els.semanticObservations.dataset.rowCount||0)+outputRows.length);
}

function renderSemanticObservations(project) {
  if (!els.semanticObservations) return;
  const logical=currentLogicalPresentation()?.value;
  if (logical) {
    const rows=Object.entries(logical.resources?.fixed64 ?? {});
    els.semanticObservations.dataset.projectRunBound='false';
    els.semanticObservations.dataset.traceSha256='';
    els.semanticObservations.dataset.semanticPayloadHash='';
    els.semanticObservations.dataset.rowCount=String(rows.length);
    els.semanticObservations.dataset.committedStateHash=logical.state_hash ?? logical.frame?.state_hash ?? '';
    els.semanticObservations.innerHTML=rows.map(([name,value])=>`<span class="semantic-observation" data-logical-resource-name="${escapeHtml(name)}"><strong>${escapeHtml(name)}</strong><span>${escapeHtml(formatNumber(value))}</span></span>`).join('');
    appendCommittedExactOutput(activeFrame());
    return;
  }
  const runEvidence = localProjectRunEvidenceFromProject(project);
  const activeRunValue = activeRun();
  const activeFrameValue = activeFrame();
  const semanticPayload = state.semanticViewPayload ?? buildCommittedSemanticViewPayload({
    project,
    run: activeRunValue,
    frame: activeFrameValue,
    viewConfig: { primary_view: project.primaryView, role: "primary", mode: state.mode },
    canonicalViewPayloads: buildCommittedFrameCanonicalViewPayloads(activeRunValue, activeFrameValue),
    canonicalViewHistory: buildCommittedCanonicalViewHistory(activeRunValue, activeFrameValue, project.primaryView),
  });
  if (semanticPayload.ordered_observations.length) {
    els.semanticObservations.dataset.projectRunBound = "true";
    els.semanticObservations.dataset.traceSha256 = runEvidence?.trace_sha256 ?? semanticPayload.semantic_payload_hash;
    els.semanticObservations.dataset.semanticPayloadHash = semanticPayload.semantic_payload_hash;
    els.semanticObservations.dataset.rowCount = String(semanticPayload.ordered_observations.length);
    els.semanticObservations.dataset.committedStateHash = semanticPayload.committed_frame?.state_hash ?? "";
    els.semanticObservations.innerHTML = semanticPayload.ordered_observations.map((row) => {
      const display = formatNumber(row.value ?? row.value_text);
      return `<span class="semantic-observation" title="${escapeHtml(row.value_text)}" data-project-observation-id="${escapeHtml(row.observation_id)}" data-project-observation-name="${escapeHtml(row.name)}" data-project-observation-value="${escapeHtml(row.value_text)}" data-state-hash-participation="${row.state_hash_participation ? "1" : "0"}"><strong>${escapeHtml(row.name)}</strong><span>${escapeHtml(display)} ${escapeHtml(row.unit)}</span></span>`;
    }).join("");
    appendCommittedExactOutput(activeFrameValue);
    if (project.lessonDisposition === "알파1 새 구현" && activeFrameValue?.wasm_state) {
      // Presentation of existing committed DDN output rows only. This does not
      // synthesize a canonical Table, world value, source lowering or library.
      const rows = extractObservationOutputRowsFromState(activeFrameValue.wasm_state);
      if (rows.length) {
        const table = document.createElement("table");
        table.setAttribute("data-testid", "lesson-output-table");
        table.setAttribute("aria-label", "실행 출력 관찰표");
        table.dataset.committedStateHash = activeFrameValue.state_hash;
        table.innerHTML = `<caption>실행 출력 관찰표</caption><thead><tr><th scope="col">관찰값</th><th scope="col">값</th></tr></thead><tbody>${rows.map(row => `<tr><th scope="row">${escapeHtml(row.key)}</th><td>${escapeHtml(formatNumber(row.value))}</td></tr>`).join("")}</tbody>`;
        els.semanticObservations.appendChild(table);
      }
    }
    return;
  }
  els.semanticObservations.dataset.projectRunBound = "false";
  els.semanticObservations.dataset.traceSha256 = "";
  els.semanticObservations.dataset.semanticPayloadHash = "";
  els.semanticObservations.dataset.rowCount = "0";
  els.semanticObservations.dataset.committedStateHash = "";
  els.semanticObservations.innerHTML = `<span class="semantic-observation" data-observation-status="awaiting-committed-frame"><strong>관찰값</strong><span>${activeFrameValue ? "이번 실행에는 관찰값이 없습니다." : "아직 실행 결과가 없습니다."}</span></span>`;
  appendCommittedExactOutput(activeFrameValue);
}

function renderRunMetadata(project) {
  const source = activeRuntimeSourceText(project);
  const run = activeRun();
  const frame = activeFrame();
  const runEvidence = localProjectRunEvidenceFromProject(project);
  const sourceHash = `candidate:fnv1a32:${stableHash(`${project.id}\n${source}`)}`;
  const activeSourceHash = currentVirtualProjectSourceHash(project) || `candidate:fnv1a32:${stableHash(source)}`;
  const sourceDirty = Boolean(run && run.source_hash !== activeSourceHash);
  // Display labels are derived from typed presentation inputs below; hashes stay unchanged.
  const frameHash = frame?.state_hash ?? "-";
  const executionState = runEvidence
    ? "current_success"
    : state.projectExecution?.failure
      ? "current_failure"
      : state.projectExecution?.previousSuccess
        ? "previous_success"
        : state.projectExecution?.importedRecord
          ? "imported_record"
          : "idle";
  els.root.dataset.sourceDirty = sourceDirty ? "true" : "false";
  els.root.dataset.projectRunBound = runEvidence ? "true" : "false";
  els.root.dataset.projectExecutionState = executionState;
  els.root.dataset.projectRunTraceSha256 = runEvidence?.trace_sha256 ?? "";
  els.root.dataset.projectRunBoundedStateSha256 = runEvidence?.bounded_state_sha256 ?? "";
  els.root.dataset.projectRunRuntimeStateHash = runEvidence?.runtime_state_hash ?? "";
  els.root.dataset.projectRunLineageHash = runEvidence?.run_lineage?.lineage_hash ?? "";
  els.root.dataset.projectRunRequestHash = runEvidence?.run_lineage?.run_request_hash ?? "";
  els.root.dataset.projectRunCreatedRevision = runEvidence?.run_lineage?.created_revision ?? "";
  els.root.dataset.projectRunOutputRows = runEvidence?.output_rows?.join("|") ?? "";
  els.root.dataset.previousProjectRunLineageHash = state.projectExecution?.previousSuccess?.evidence?.run_lineage?.lineage_hash ?? "";
  els.root.dataset.projectRunFailureSourceHash = state.projectExecution?.failure?.source_hash ?? "";
  els.root.dataset.importedProjectRecordSourceHash = state.projectExecution?.importedRecord?.source_hash ?? "";
  if (state.sourceReadinessRunBlocked) {
    rememberRunStatus('readiness_blocked');
    els.runHash.textContent = `source_hash: ${sourceHash}`;
  } else if (runEvidence) {
    rememberRunStatus('lifecycle');
    els.runHash.textContent = runEvidence.bounded_state_sha256
      ? `state_hash: ${runEvidence.runtime_state_hash || "-"} · bounded_state: ${runEvidence.bounded_state_sha256}`
      : `state_hash: ${runEvidence.runtime_state_hash || frameHash}`;
  } else if (state.projectExecution?.failure) {
    const failure = state.projectExecution.failure;
    rememberRunStatus('failure', failure.diagnostic?.code);
    els.runHash.textContent = `failed_source: ${failure.source_hash || "-"}`;
  } else if (state.projectExecution?.previousSuccess) {
    const previous = state.projectExecution.previousSuccess;
    rememberRunStatus('previous_success');
    els.runHash.textContent = `previous_state_hash: ${previous.evidence?.runtime_state_hash || "-"}`;
  } else if (state.projectExecution?.importedRecord) {
    rememberRunStatus('imported_record');
    els.runHash.textContent = `imported_source: ${state.projectExecution.importedRecord.source_hash || "-"}`;
  } else {
    rememberRunStatus(sourceDirty ? 'dirty_source' : 'lifecycle');
    els.runHash.textContent = frameHash === "-" ? `source_hash: ${sourceHash}` : `state_hash: ${frameHash}`;
  }
  const logicalCommitted = currentLogicalPresentation()?.value;
  renderCurrentMadi(logicalCommitted,frame?.madi);
  els.runLineage.textContent = runEvidence
    ? `project-run: ${runEvidence.frontdoor} · lineage ${String(runEvidence.run_lineage?.lineage_hash ?? "").slice(0, 24)} · revision ${runEvidence.run_lineage?.created_revision || "-"}`
    : run
      ? `run: ${run.run_id}${run.parent_run_id ? ` / parent ${run.parent_run_id}@${run.parent_madi}` : ""}`
      : state.projectExecution?.previousSuccess
        ? `previous-project-run: ${String(state.projectExecution.previousSuccess.evidence?.run_lineage?.lineage_hash ?? "").slice(0, 24)}`
        : "run: -";
  els.runArchiveStatus.textContent = run?.server_run_id
    ? `보관 기록: ${run.server_run_id} · 확인 지점 ${run.server_checkpoint_count ?? 0} · ${run.server_final_status ?? "실행 중"}`
    : runEvidence
      ? `보관 기록: 현재 실행 · 출력 ${runEvidence.output_rows.length}줄 · 요청 ${String(runEvidence.run_lineage?.run_request_hash ?? "").slice(0, 24)}`
      : `보관 기록: 서버 실행 ${state.runArchives.length}개`;
  els.viewGradeLabel.textContent = project.files.length === 1
    ? "원문 1개"
    : `프로젝트 파일 ${project.files.length}개`;
  els.viewStatus.textContent = "보기 변경은 계산 결과를 바꾸지 않습니다.";
  const capabilityRole = state.capabilitySession?.role || state.serverProject.role || "";
  const viewOnly = Boolean((state.serverProject.projectId || state.capabilitySession?.project_id) && capabilityRole === "view");
  els.sourceEditor.readOnly = viewOnly || Boolean(run && state.cursorIndex >= 0 && state.cursorIndex < run.frames.length - 1);
  if (!state.persistenceStatus) {
    try {
      const localSnapshot = loadLocalFallbackSnapshot(project.id);
      if (localSnapshot) {
        state.persistenceStatus = {
          schema: "seamgrim.workbench.persistence_status.v1",
          mode: "local_fallback",
          server_saved: false,
          local_saved: true,
          user_label: `이 기기에만 임시 저장 ${localSnapshot.canonical_hash}`,
          snapshot: localSnapshot,
        };
      }
    } catch (_) {
      state.persistenceStatus = null;
    }
  }
  renderPersistenceFeedback();
}

function renderPersistenceFeedback(presentation = null) {
  const saving = state.persistenceOperation?.phase === "saving";
  const saveButton = document.querySelector("[data-testid='project-save']");
  // Keep focus on the admitted input control: disabling it can emit focusout
  // and change the logical input snapshot while a save is frozen.
  if (saveButton) {
    saveButton.dataset.disabledReason = saving ? "persistence_in_flight" : saveButton.disabled ? "view_write_forbidden" : "";
    saveButton.setAttribute("aria-busy", String(saving));
    saveButton.setAttribute("aria-disabled", String(saving || saveButton.disabled));
  }
  lastPersistenceFeedback = Object.freeze({phase:presentation?.phase ?? state.persistenceOperation?.phase ?? '',
    outcome:state.persistenceOperation?.outcome ?? '',statusMode:state.persistenceStatus?.mode ?? '',
    original:saving?'':presentation?.original ?? describePersistenceStatus(state.persistenceStatus)});
  paintPersistenceFeedback();
}

function renderDiagnostics() {
  if (!els.diagnostics) return;
  const locale = document.documentElement?.dataset.displayLocale === 'ja' ? 'ja' : 'ko';
  const errorCount = state.diagnostics.filter((diag) => diag.severity === "error").length;
  const labels = diagnosticSummary(locale, state.diagnostics.length, errorCount);
  if (els.projectVerificationOpen) {
    els.projectVerificationOpen.textContent = labels.label;
    els.projectVerificationOpen.dataset.hasErrors = errorCount ? "true" : "false";
    els.projectVerificationOpen.setAttribute("aria-label", labels.aria);
  }
  els.diagnostics.innerHTML = "";
  const readinessDiagnostic = state.sourceReadiness?.diagnostics?.[0] ?? null;
  if (!readinessDiagnostic && !state.diagnostics.length) {
    const row = document.createElement("li");
    row.textContent = labels.empty;
    els.diagnostics.appendChild(row);
    return;
  }
  if (readinessDiagnostic) {
    const row = document.createElement("li");
    row.dataset.severity = readinessDiagnostic.severity;
    row.dataset.diagnosticCode = readinessDiagnostic.code;
    const message = document.createElement("span");
    appendDiagnosticPresentation(message, readinessDiagnostic,
      `${readinessDiagnostic.code}: ${readinessDiagnostic.message} `, locale);
    row.append(message);
    const candidate = getSourceReadinessCandidate();
    if (candidate && readinessDiagnostic.fix_it) {
      const fixButton = document.createElement("button");
      fixButton.type = "button";
      fixButton.className = "project-graph-diagnostic";
      fixButton.dataset.testid = "source-readiness-fixit";
      fixButton.textContent = "콜론 제거하기";
      fixButton.disabled = sourceReadinessViewOnly();
      fixButton.dataset.disabledReason = fixButton.disabled ? "view_only" : "";
      fixButton.addEventListener("click", () => applySourceReadinessFixIt(candidate));
      row.append(fixButton);
    }
    els.diagnostics.appendChild(row);
  }
  [...state.diagnostics].sort((a, b) => Number(b.severity === "error") - Number(a.severity === "error")).forEach((diag) => {
    const row = document.createElement("li");
    row.dataset.severity = diag.severity;
    const original = diag.message.includes(diag.code) || ["E_PROJECT_RUN_FAILED", "E_WASM_STEP_FAILED"].includes(diag.code)
      ? diag.message : `${diag.code}: ${diag.message}`;
    appendDiagnosticPresentation(row, diag, original, locale);
    els.diagnostics.appendChild(row);
  });
}

  document.addEventListener?.('ddn-display-locale-change', () => {
    renderDiagnostics();paintRunStatus();paintPersistenceFeedback();
    const label=displayMessage('output',document.documentElement.dataset.displayLocale ?? 'ko').text;
    els.semanticObservations?.querySelectorAll('[data-committed-output-label]').forEach(node=>{node.textContent=label;});
  });
  return { renderMode, buildViewInstanceSemanticPayload, setViewActionState, renderSideViewInstances, initialDragPoint, renderViewPolicy, selectPrimaryCanvasTarget, renderSemanticObservations, renderRunMetadata, renderPersistenceFeedback, renderDiagnostics };
}
