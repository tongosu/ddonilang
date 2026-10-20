import { createRuntimeBridgeV1 } from '../runtime/runtime_bridge.js';
import { LogicalActionInput } from '../runtime/logical_action_input.js';
import { createNativeLogicalFinishBarrier } from '../runtime/native_recording_finish.js';
import { nativeSourceAvailable,nativeSourceRequest,sourceStorageErrorMessage } from '../runtime/native_source_transport.js';
import { renderSpace2dCanvas2d } from '../wasm_page_common.js';
import { createAuthoredCompositionHost, renderAuthoredBogae } from './authored_bogae_renderer.js';

// The committed logical response can carry an explicit draw list without a
// Bogae composition. Never reconstruct shapes from world resources.
export function committedLogicalSpace2d(value) {
  const meta=value?.view_meta;
  if (!Array.isArray(meta?.draw_list) || !meta.draw_list.length) return null;
  const width=Number(meta.canvas_width),height=Number(meta.canvas_height);
  if (!Number.isFinite(width)||!Number.isFinite(height)||width<=0||height<=0) return null;
  return {drawlist:meta.draw_list,camera:{x_min:0,x_max:width,y_min:0,y_max:height}};
}

// Explicit, run-local Alpha2 contract intake. No source sniffing, project
// metadata defaults, new persisted schema, or Alpha1 execution replacement.
export function installLogicalInputPanel({ parent, readSource, readAuthoredProject = () => null, prepareNativeNewRun = async () => null, prepareNativeNewRecord = null, onNativeCheckpoint = () => {}, openExample, onStateChange = () => {}, playbackControls = [], inputRegion = null, viewControls = [], onPresentation = null, valueControlsParent = null }) {
  const panel = document.createElement('details');
  panel.id = 'logical-input-panel';
  panel.style.cssText = 'margin:12px;padding:12px;border:1px solid #888;border-radius:8px';
  panel.innerHTML = `<summary>입력 기록</summary>
    <div data-field="region" tabindex="0" aria-label="논리 행동 입력 영역">
      <button data-action="start">현재 원문으로 새로 시작</button>
      <button data-action="step">한 마디</button>
      <button data-action="play">60마디/초 재생</button>
      <button data-action="pause">일시정지</button>
      <button data-action="seal">기록 마치기·저장</button>
      <button data-action="stop">실행 닫기</button>
      <p>이 영역 안에서 지정한 키를 누르세요. 영역 밖으로 초점을 옮기면 입력 손실로 중단됩니다.</p>
      <div class="authored-view-toolbar" data-field="view-toolbar"></div>
      <div class="bogae-stage" data-field="view-stage">
        <div class="canvas-wrap"><canvas data-field="canvas" width="240" height="480" tabindex="0" hidden aria-label="원문이 만든 그림과 논리 입력 영역"></canvas></div>
      </div>
      <div data-field="view-list" hidden></div>
    </div>
    <label>기록 불러오기 <input data-field="replay" type="file"></label>
    <label><input data-field="partial-replay" type="checkbox"> 중단된 기록 끝까지만 재생</label>
    <p data-field="feedback" role="status"></p>
    <div data-field="value-controls" hidden></div>
    <details data-field="advanced"><summary>고급 설정</summary>
      <p>입력 계약·키 배치·실행 증거를 확인하거나 바꿉니다. 변경하면 현재 실행을 닫습니다.
      통합 V3 후보로 저장합니다. 기록 마치기는 성공한 마디까지만 정상 기록으로 남깁니다. 별도 중단은 미완 기록이며 정상 재생은 할 수 없습니다. 옛 형식의 자동 변환과 글자 입력은 지원하지 않습니다.</p>
      <button data-action="stop-recording">미완 기록으로 중단·저장</button>
      <button data-action="verify-receipt" hidden>저장한 완료 기록 재현 확인</button>
      <button data-action="new-record" hidden>저장한 시점에서 새 기록</button>
      <label>미리 정한 입력 요청으로 기록 <input data-field="finite-request" type="file" accept=".json,application/json"></label>
      <p>현재 원문·입력 계약에 맞는 요청 JSON의 모든 마디를 실행해 정상 V3 기록을 저장합니다. 실시간 중단 기록을 완료로 바꾸는 기능은 아닙니다.</p>
      <button data-action="open-tetris" ${typeof openExample==='function'?'':'hidden'}>새 테트리스 작업</button>
      <button data-action="tetris-contract">테트리스 입력 계약</button>
      <button data-action="bind-source">현재 원문에 다시 결속</button>
      <p data-field="contract-summary" style="overflow-wrap:anywhere">입력 계약을 선택하지 않았습니다.</p>
      <details data-field="contract-editor"><summary>입력 계약 편집</summary>
        <label>입력 계약(JSON) <textarea data-field="binding" rows="4"></textarea></label>
        <label>키 배치(action_map.v1) <textarea data-field="actions" rows="4"></textarea></label>
        <label>한 마디 사건 용량 <input data-field="capacity" type="number" min="1" step="1"></label>
      </details>
      <details><summary>실행 결과·영수증</summary><pre data-field="result"></pre></details>
    </details>`;
  panel.querySelectorAll('textarea').forEach(node => { node.style.cssText = 'display:block;width:100%;box-sizing:border-box'; });
  parent.append(panel);
  // Live controls belong next to the scene, not inside the record/settings drawer.
  const valueControls = panel.querySelector('[data-field="value-controls"]');
  if (valueControlsParent) valueControlsParent.append(valueControls);
  const field = name => name === 'value-controls' ? valueControls : panel.querySelector(`[data-field="${name}"]`);
  if (onPresentation) {
    for (const action of ['open-tetris','tetris-contract']) panel.querySelector(`[data-action="${action}"]`).remove();
    for (const name of ['view-stage','view-list','view-toolbar']) field(name).style.display='none';
    field('region').querySelector('p').hidden=true; // The official project presents its own key guide.
  }
  const bridge = createRuntimeBridgeV1();
  let host = null, session = null, source = '', bindingJson = '', actionsJson = '', capacityText = '';
  let timer = null, running = false, busy = false, settling = false, generation = 0, inFlightPump = null;
  let boundSource = null;
  let authoredSource = null, boundAuthoredSource = null;
  const typedBinding = json => JSON.parse(json).source_profile==='v1-core-v25-manipulation-alpha4-provisional';
  let valueIntake = Promise.resolve();
  function renderValueControls(binding) {
    panel.querySelector('[data-action="verify-receipt"]').hidden=!boundAuthoredSource||typedBinding(JSON.stringify(binding));
    panel.querySelector('[data-action="new-record"]').hidden=!boundAuthoredSource||typedBinding(JSON.stringify(binding))||typeof prepareNativeNewRecord!=='function';
    const controls=field('value-controls');controls.replaceChildren();
    controls.hidden=binding.source_profile!=='v1-core-v25-manipulation-alpha4-provisional';
    if(controls.hidden)return;
    const decimal=raw=>Number(BigInt(raw))/4294967296;
    const admit=(channel,phase,texts)=>{
      const token=generation;
      valueIntake=valueIntake.then(async()=>{
        if(token!==generation || !host)throw new Error('E_MANIPULATION_SESSION: ▶로 먼저 실행하세요.');
        const raw=await Promise.all(texts.map(text=>bridge.manipulationValueRaw(text)));
        if(token!==generation)throw new Error('E_MANIPULATION_SESSION: 실행이 바뀌었습니다.');
        host.receiveValue(channel,{phase,raw:raw.length===1?raw[0]:raw});
      }).catch(error=>{
        // An old conversion completing after explicit close belongs to the old
        // run; it must never abort or enqueue into its successor.
        if(token!==generation)return;
        host?.loseInput(String(error));field('feedback').textContent=String(error);
      });
    };
    for(const [channel,contract] of Object.entries(binding.channels)) {
      if(contract.kind==='Number') {
        const label=document.createElement('label');label.textContent=channel;
        const slider=document.createElement('input');slider.type='range';slider.dataset.manipulationChannel=channel;
        slider.min=String(decimal(contract.domain.min_raw));slider.max=String(decimal(contract.domain.max_raw));
        slider.step='any';slider.value=slider.min;
        slider.addEventListener('input',()=>admit(channel,'Set',[slider.value]));label.append(slider);controls.append(label);
      } else if(contract.kind==='Coordinate') {
        const pad=document.createElement('canvas');pad.width=240;pad.height=140;pad.tabIndex=0;pad.dataset.manipulationChannel=channel;
        pad.style.cssText='display:block;max-width:100%;border:1px solid #738577;touch-action:none';
        pad.setAttribute('aria-label',`${channel} 좌표 조작`);
        const context=pad.getContext('2d');context.fillStyle='#215b8b';context.fillText(`${channel} · 누르고 끌기`,10,20);
        let pointer=null;
        const send=(event,phase)=>{
          const rect=pad.getBoundingClientRect();
          const ratios=[Math.max(0,Math.min(1,(event.clientX-rect.left)/rect.width)),Math.max(0,Math.min(1,1-(event.clientY-rect.top)/rect.height))];
          const texts=contract.axes.map((axis,index)=>(decimal(axis.min_raw)+(decimal(axis.max_raw)-decimal(axis.min_raw))*ratios[index]).toFixed(6));
          admit(channel,phase,texts);
          context.clearRect(0,0,pad.width,pad.height);context.fillText(`${channel} (${texts.join(', ')})`,10,20);
          context.beginPath();context.arc(ratios[0]*pad.width,(1-ratios[1])*pad.height,5,0,Math.PI*2);context.fill();
        };
        pad.addEventListener('pointerdown',event=>{if(pointer!==null)return;pointer=event.pointerId;pad.setPointerCapture(pointer);send(event,'Begin');});
        pad.addEventListener('pointermove',event=>{if(pointer===event.pointerId)send(event,'Move');});
        pad.addEventListener('pointerup',event=>{if(pointer!==event.pointerId)return;send(event,'End');pointer=null;});
        pad.addEventListener('pointercancel',()=>{pointer=null;host?.loseInput('끌기 입력이 취소됐습니다.');});
        controls.append(pad);
      }
    }
  }
  let committedView = null;
  const compositionHost = createAuthoredCompositionHost(() => renderView(committedView));
  function renderView(value) {
    if (onPresentation) { onPresentation(value, generation); return; }
    const canvas = field('canvas');
    if (value?.bogae_projection) {
      canvas.hidden = false;
      compositionHost.render({projectId:'logical-input-session',wasmState:value,
        list:field('view-list'),stage:field('view-stage'),toolbar:field('view-toolbar'),canvas});
      const ctx = canvas.getContext('2d');
      ctx.clearRect(0,0,canvas.width,canvas.height);
      const model = renderAuthoredBogae(ctx,canvas.width,canvas.height,value,compositionHost);
      canvas.dataset.renderSource = model?.data_source ?? '';
      canvas.dataset.committedStateHash = value.state_hash ?? value.frame?.state_hash ?? '';
      return;
    }
    const space2d = committedLogicalSpace2d(value);
    canvas.hidden = !space2d;
    canvas.dataset.renderSource = space2d ? 'rust_legacy_drawlist' : '';
    if (space2d) renderSpace2dCanvas2d({canvas,space2d,yAxisDown:true,pad:0,showViewMetrics:false});
  }
  const show = value => {
    field('result').textContent = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
    field('feedback').textContent = typeof value === 'string' ? sourceStorageErrorMessage(value)
      : value?.replay_notice ? value.replay_notice
      : value?.frame ? `마디 ${value.frame.input.madi} 계산을 마쳤습니다.`
      : `실행 상태: ${value?.outcome ?? '준비'} · 계산한 마디 ${value?.madi_count ?? 0}`;
    const acceptedSummary = value?.madi_count > 0 && value?.outcome !== 'ABORTED'
      && (value?.bogae_projection || committedLogicalSpace2d(value));
    if (value?.frame || acceptedSummary) { committedView = value; renderView(value); }
    else if(committedView&&Number.isSafeInteger(value?.madi_count)&&value.madi_count>0
      &&Number.isSafeInteger(committedView.frame?.input?.madi)&&value.madi_count===committedView.frame.input.madi+1
      &&typeof value.state_hash==='string'&&value.state_hash===(committedView.state_hash??committedView.frame?.state_hash)) {
      // A finish summary has no drawing projection. Its matching accepted
      // count/hash confirms the already displayed frame; do not erase it or
      // invent a replacement world just to show the recording receipt.
      renderView(committedView);
    }
    else if (committedView && (typeof value === 'string' || value?.outcome === 'ABORTED')) {
      // A diagnostic is not a replacement committed frame. Keep the last
      // successful world projection while reporting the abort above. Explicit
      // close/start/project changes reset committedView in close(), so this
      // never carries another execution's picture into its successor.
    }
    else {
      compositionHost.reset(); committedView = null;
      onPresentation?.(null, generation);
      field('canvas').hidden = true;
      field('canvas').dataset.renderSource = '';
    }
  };
  function pause() { running = false; clearTimeout(timer); timer = null; onStateChange(); }
  async function pauseAndSettle() {
    settling = true;
    try { pause(); if(inFlightPump) await inFlightPump; }
    finally { settling = false; onStateChange(); }
  }
  function unchanged() {
    if (readSource() !== source || (readAuthoredProject() || null) !== authoredSource || field('binding').value !== bindingJson
      || field('actions').value !== actionsJson || field('capacity').value !== capacityText) {
      host?.loseInput('원문·입력 계약·키 사상 또는 사건 용량이 바뀌었습니다. 새로 시작하세요.');
      throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 원문·입력 계약·키 사상 또는 사건 용량이 바뀌었습니다.');
    }
  }
  async function close() {
    pause(); generation++;
    valueControls.replaceChildren(); valueControls.hidden = true;
    compositionHost.reset(); committedView = null;
    onPresentation?.(null, generation);
    await host?.finishBarrier?.close?.();
    host?.detach?.(); host = null;
    const old = session; session = null;
    if (old) await old.close();
  }
  async function step() {
    if (!host) throw new Error('E_LOGICAL_INPUT_SESSION: 먼저 새로 시작하세요.');
    unchanged();
    const token = generation;
    const result = await host.step();
    if (token !== generation) return null;
    unchanged();
    show(result); // Rust result/receipt only; never fabricate a world in JS.
    return result;
  }
  async function pump(token) {
    if (!running || token !== generation) return;
    const started = performance.now(); // Host scheduling only, never snapshot dt.
    const pending = step();
    inFlightPump = pending;
    try { await pending; }
    catch (error) { pause(); show(String(error)); return; }
    finally { if(inFlightPump===pending) inFlightPump=null; }
    if (running && token === generation) timer = setTimeout(() => pump(token), Math.max(0, 1000 / 60 - (performance.now() - started)));
  }
  async function execute(action) {
    if (action === 'pause') { await pauseAndSettle(); return; }
    if (busy) throw new Error('E_LOGICAL_INPUT_STEP_BUSY: 앞 작업이 끝난 뒤 다시 누르세요.');
    busy = true;
    onStateChange(); // Disable execution before the asynchronous contract bind.
    try {
      if (action === 'tetris-contract' || action === 'bind-source' || action === 'open-tetris') {
        const readAsset = async name => {
          const response = await fetch(new URL(`../../samples/${name}`, import.meta.url));
          if (!response.ok) throw new Error(`E_LOGICAL_INPUT_CONTRACT_LOAD: 입력 계약 파일을 읽지 못했습니다 (${response.status}).`);
          return response.text();
        };
        const exampleSource = action==='open-tetris' ? await readAsset('alpha2_logical_tetris.ddn') : null;
        await close();
        if(exampleSource!==null) {
          if(typeof openExample!=='function') throw new Error('E_LOGICAL_INPUT_EXAMPLE: 새 작업을 여는 입구가 없습니다.');
          await openExample(exampleSource);
        }
        const chosenSource = readSource();
        const chosenAuthoredSource = readAuthoredProject() || null;
        const originalBinding = field('binding').value;
        const originalActions = field('actions').value, originalCapacity = field('capacity').value;
        let chosenBinding = originalBinding, chosenActions = null;
        if (action === 'tetris-contract' || action === 'open-tetris') {
          [chosenBinding, chosenActions] = await Promise.all([
            readAsset('alpha2_logical_tetris.binding.json'), readAsset('alpha2_logical_tetris.action_map.json'),
          ]);
          JSON.parse(chosenActions);
        }
        if(chosenAuthoredSource && typedBinding(chosenBinding))throw new Error('E_MANIPULATION_PROJECT_UNSUPPORTED: 여러 파일 조작 입력은 아직 지원하지 않습니다.');
        const prepared = await bridge[typedBinding(chosenBinding)?'bindManipulationSource':'bindLogicalSource']({sourceText:chosenSource,bindingJson:chosenBinding,authoredProjectJson:chosenAuthoredSource});
        if (readSource() !== chosenSource || (readAuthoredProject() || null) !== chosenAuthoredSource || field('binding').value !== originalBinding
          || field('actions').value !== originalActions || field('capacity').value !== originalCapacity) {
          throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 결속 도중 원문 또는 계약이 바뀌었습니다. 다시 선택하세요.');
        }
        const bound = JSON.parse(prepared);
        field('binding').value = JSON.stringify(bound, null, 2);
        boundSource = chosenSource;
        boundAuthoredSource = chosenAuthoredSource;
        if (chosenActions !== null) {field('actions').value=chosenActions;field('capacity').value='128';}
        field('contract-summary').textContent = `${bound.source_profile} · 화면 ${bound.madi_hz}마디/초 · 세계 dt=1 · 원문 ${bound.source_revision}`;
        field('result').textContent='현재 원문에 결속했습니다. 상단 ▶로 실행하세요. 아직 세계 마디는 실행하지 않았습니다.';
        field('feedback').textContent='';
        renderValueControls(bound);
      } else if (action === 'start') {
        await close();
        source = readSource(); bindingJson = field('binding').value;
        authoredSource = readAuthoredProject() || null;
        actionsJson = field('actions').value; capacityText = field('capacity').value;
        const binding = JSON.parse(bindingJson);
        const actionMap = JSON.parse(actionsJson);
        const maxEvents = Number(capacityText);
        const typed=typedBinding(bindingJson);
        if(authoredSource!==boundAuthoredSource)throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 작품 원문을 다시 결속하세요.');
        if(typed && authoredSource)throw new Error('E_MANIPULATION_PROJECT_UNSUPPORTED: 여러 파일 조작 입력은 아직 지원하지 않습니다.');
        // Native consumes the protected once-permit before a fresh WASM VM.
        // It records the explicit initial empty-input tick, not an old run.
        const nativeSummary=!typed&&authoredSource ? await prepareNativeNewRun(authoredSource,binding) : null;
        unchanged();
        session = await bridge[typed?'createManipulationSession':'createLogicalSession']({ sourceText:source, bindingJson, authoredProjectJson:authoredSource });
        try {
          unchanged();
          if(nativeSummary){
            if(nativeSummary.outcome==='ABORTED'||nativeSummary.madi_count!==1||nativeSummary.frames?.length!==1)
              throw new Error('E_NATIVE_SOURCE_NEW_LOGICAL_INPUTS');
            await session.step(nativeSummary.frames[0].input);unchanged();
            const stable=value=>JSON.stringify(value,(_key,part)=>part&&typeof part==='object'&&!Array.isArray(part)
              ?Object.fromEntries(Object.entries(part).sort(([a],[b])=>a<b?-1:a>b?1:0)):part);
            if(stable(await session.summary())!==stable(nativeSummary))throw new Error('E_NATIVE_SOURCE_LOGICAL_REPLAY_MISMATCH');
          }
          const channels=typed?Object.fromEntries(Object.entries(binding.channels).filter(([,value])=>value.kind==='Control').map(([name])=>[name,'Control'])):binding.channels;
          const finishGeneration=generation,finishSession=session;
          const finishUnchanged=()=>{unchanged();if(generation!==finishGeneration||session!==finishSession)
            throw new Error('E_NATIVE_FINISH_SESSION_CHANGED: 기록 생산자가 바뀌었습니다.');};
          host = new LogicalActionInput({ session, channels, initialHeld:binding.initial_held, actionMap, maxEvents, valueChannels:typed?binding.channels:null,nextMadi:nativeSummary?1:0,
            finishBarrier:!typed&&authoredSource?createNativeLogicalFinishBarrier({authored:authoredSource,binding,unchanged:finishUnchanged,onCheckpoint:onNativeCheckpoint,
              onProgress:message=>{finishUnchanged();field('feedback').textContent=message;}}):null });
          renderValueControls(binding);
          host.attach(inputRegion ?? field('region'), { controls: [...playbackControls,...viewControls,
            panel.querySelector('summary'), field('advanced').querySelector(':scope > summary'), panel.querySelector('[data-action="seal"]'),
            panel.querySelector('[data-action="verify-receipt"]'),
            panel.querySelector('[data-action="new-record"]'),
            panel.querySelector('[data-action="stop-recording"]'),
            field('value-controls'),
            ...(inputRegion ? [field('region')] : [])] });
        } catch (error) { await close(); throw error; }
        const summary = await session.summary();
        unchanged();
        show(summary);
      } else if (action === 'step') { await pauseAndSettle(); await step(); }
      else if (action === 'play') {
        if (!host) throw new Error('E_LOGICAL_INPUT_SESSION: 먼저 새로 시작하세요.');
        unchanged(); pause(); running = true; void pump(generation);
      } else if (action === 'seal') {
        await pauseAndSettle(); await valueIntake; unchanged();
        if (!host) throw new Error('E_LOGICAL_INPUT_SESSION: 먼저 새로 시작하세요.');
        const token = generation;
        const record = await host.seal();
        if (token !== generation) return;
        unchanged();
        const url = URL.createObjectURL(new Blob([Uint8Array.from(record.tape_bytes)], {type:'application/octet-stream'}));
        const link = document.createElement('a'); link.href = url; link.download = 'input-v3-complete.bin'; link.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000); show(record.summary);
        const count=host.finishNotice?.excluded_input_count;
        field('feedback').textContent=count && count!=='0'
          ?`기록을 마쳤습니다. 처리되지 않은 입력 ${count}개는 기록에 넣지 않았습니다.`
          :'성공한 마디까지 기록을 마쳤습니다. 작품 저장은 별도입니다.';
      } else if (action === 'new-record') {
        await pauseAndSettle();await valueIntake;
        // A cold source/contract selection has no live producer yet. Validate
        // that selection, not the empty prior run's binding. An existing run
        // still has to pass its own unchanged check before it can be closed.
        if(host||session)unchanged();
        if(host&&!host.stopped)throw new Error('E_NEW_RECORD_CLOSE_REQUIRED: 먼저 기록을 마치거나 실행을 닫은 뒤 선택하세요.');
        if(typeof prepareNativeNewRecord!=='function')throw new Error('E_NEW_RECORD_UNSUPPORTED');
        const selected=readAuthoredProject(),selectedSource=readSource();
        const selectedBinding=field('binding').value,selectedActions=field('actions').value,selectedCapacity=field('capacity').value;
        const binding=JSON.parse(selectedBinding);
        if(!selected||typedBinding(selectedBinding))throw new Error('E_NEW_RECORD_UNSUPPORTED');
        const requireSelection=()=>{
          if(readSource()!==selectedSource||readAuthoredProject()!==selected
            ||boundSource!==selectedSource||boundAuthoredSource!==selected
            ||field('binding').value!==selectedBinding||field('actions').value!==selectedActions||field('capacity').value!==selectedCapacity)
            throw new Error('E_NEW_RECORD_SOURCE_CHANGED: 선택한 원문이나 입력 계약이 바뀌었습니다.');
        };
        requireSelection();
        await close();const token=generation;
        const record=await prepareNativeNewRecord(selected,binding);
        requireSelection();if(generation!==token)throw new Error('E_NEW_RECORD_SOURCE_CHANGED');
        const prepared=await panel.logicalExecution.prepareStoredCheckpoint(record);
        try{await prepared.publish();}catch(error){await prepared.close();throw error;}
        field('feedback').textContent='끝난 기록을 보존하고 실제 저장 시점에서 새 기록을 시작했습니다. ▶로 계속하세요.';
      } else if (action === 'verify-receipt') {
        await pauseAndSettle();await valueIntake;
        const selected=readAuthoredProject(),selectedSource=readSource(),token=generation,originalHost=host;
        const selectedBinding=field('binding').value,selectedActions=field('actions').value,selectedCapacity=field('capacity').value;
        const requireSelection=()=>{
          // A cold reader has no live session's source/action snapshots yet.
          // Use its explicit current binding, never synthesize a started VM.
          if(originalHost)unchanged();
          if(token!==generation||host!==originalHost||readAuthoredProject()!==selected||readSource()!==selectedSource
              ||boundAuthoredSource!==selected||boundSource!==selectedSource||field('binding').value!==selectedBinding
              ||field('actions').value!==selectedActions||field('capacity').value!==selectedCapacity)
            throw new Error('E_NATIVE_RECEIPT_SOURCE_CHANGED: 선택한 원문이나 입력 계약이 바뀌었습니다.');
        };
        requireSelection();
        if(!selected||typedBinding(selectedBinding)||!await nativeSourceAvailable())
          throw new Error('E_NATIVE_RECEIPT_UNSUPPORTED: Native에 저장한 논리 입력 완료 기록만 확인할 수 있습니다.');
        requireSelection();
        const verified=originalHost?.finishBarrier?.verifyReceipt
          ?await originalHost.finishBarrier.verifyReceipt()
          :await nativeSourceRequest('logical_verify_receipt',selected,
            {input_request:{binding:JSON.parse(selectedBinding),snapshots:[]}});
        requireSelection();
        if(verified.status!=='REPLAY_VERIFIED'||typeof verified.result_identity!=='string'||!/^sha256:[0-9a-f]{64}$/u.test(verified.result_identity))
          throw new Error('E_NATIVE_RECEIPT_REJECTED: 완료 기록의 재현을 확인하지 못했습니다.');
        field('feedback').textContent='재현 확인: 저장한 완료 기록과 원래 결과가 같습니다. 현재 실행·세계·입력은 바꾸지 않았습니다.';
        field('result').textContent=JSON.stringify({status:verified.status,result_identity:verified.result_identity},null,2);
      } else if (action === 'stop-recording') {
        await pauseAndSettle(); await valueIntake; unchanged();
        if (!host) throw new Error('E_LOGICAL_INPUT_SESSION: 먼저 새로 시작하세요.');
        const token = generation;
        const record = await host.stopRecording();
        if (token !== generation) return;
        unchanged();
        const url = URL.createObjectURL(new Blob([Uint8Array.from(record.tape_bytes)], {type:'application/octet-stream'}));
        const link = document.createElement('a'); link.href = url; link.download = `input-v3-${record.outcome.toLowerCase()}.bin`; link.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000); show(record.summary);
        field('feedback').textContent=record.outcome==='ABORTED'
          ?host.nativeStopStored
            ?'실패 전 입력과 Native의 실제 실패 기록을 보존했습니다. 성공 기록이 아니며 정상 재생할 수 없습니다.'
            :'실패 전까지 확정된 입력을 파일에 보존했습니다. 성공 기록이 아니며 정상 재생할 수 없습니다. Native 실패 기록은 확인되지 않았습니다.'+(host.nativeStopFailure?' '+host.nativeStopFailure:'')
          :host.nativeStopStored
            ?'미완 기록과 Native 저장 경계를 보존했습니다. 정상 완료나 정상 재생 성공은 아닙니다.'
            :'기록을 명시적으로 중단해 미완 기록 파일로 저장했습니다. 정상 재생할 수 없습니다. 작품 저장은 별도입니다.';
      } else if (action === 'stop') { await close(); show('실행을 닫았습니다.'); }
    } finally { busy = false; onStateChange(); }
  }
  panel.querySelectorAll('[data-action]').forEach(button => button.addEventListener('click', () => {
    execute(button.dataset.action).catch(error => { pause(); show(String(error)); });
  }));
  field('replay').addEventListener('change', async () => {
    if (busy || running) { field('replay').value = ''; show('E_LOGICAL_INPUT_STEP_BUSY: 실행을 닫고 파일을 여세요.'); return; }
    busy = true;
    const allowPartial=field('partial-replay').checked;
    field('partial-replay').disabled=true;
    try {
      const file = field('replay').files[0]; if (!file) return;
      if (file.size > 67108864) throw new Error('E_LOGICAL_INPUT_SIZE: 입력 기록의 크기 한도를 넘었습니다.');
      const replaySource = readSource(), replayBinding = field('binding').value, token = generation;
      const replayAuthored=readAuthoredProject()||null;
      const requireCurrent = () => {
        if (generation !== token || readSource() !== replaySource || field('binding').value !== replayBinding || (readAuthoredProject()||null)!==replayAuthored || field('partial-replay').checked!==allowPartial) {
          throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 재생 도중 원문 또는 입력 계약이 바뀌었습니다. 다시 파일을 여세요.');
        }
      };
      const tapeBytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      requireCurrent();
      if(typedBinding(replayBinding)&&replayAuthored)throw new Error('E_MANIPULATION_PROJECT_UNSUPPORTED: 여러 파일 조작 입력은 지원하지 않습니다.');
      const summary = await bridge[typedBinding(replayBinding)?'replayManipulationInput':'replayLogicalInput']({sourceText:replaySource, bindingJson:replayBinding, tapeBytes,authoredProjectJson:replayAuthored,allowPartial});
      requireCurrent();
      // Admit in a private Rust candidate first. Invalid/unchecked Incomplete/Aborted
      // files must not close the user's current VM or clear its scene.
      const committedGeneration=generation+1;
      await close();
      if(generation!==committedGeneration || readSource()!==replaySource || field('binding').value!==replayBinding || (readAuthoredProject()||null)!==replayAuthored || field('partial-replay').checked!==allowPartial) {
        throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 재생 게시 전에 작품이 바뀌었습니다.');
      }
      show(summary);
    } catch (error) { show(String(error)); }
    finally { field('replay').value = ''; field('partial-replay').disabled=false; busy = false; onStateChange(); }
  });
  field('finite-request').addEventListener('change',async()=>{
    if(busy||running){field('finite-request').value='';show('E_LOGICAL_INPUT_STEP_BUSY: 일시정지한 뒤 입력 요청을 여세요.');return;}
    busy=true;
    try{
      const file=field('finite-request').files[0];if(!file)return;
      if(file.size>67108864)throw new Error('E_INPUT_TAPE_V3: 입력 요청 크기 한도를 넘었습니다.');
      const originalSource=readSource(),expectedBinding=field('binding').value,originalProject=readAuthoredProject()||null,token=generation;
      const current=()=>{
        if(generation!==token||readSource()!==originalSource||field('binding').value!==expectedBinding||(readAuthoredProject()||null)!==originalProject)
          throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 기록 도중 원문 또는 입력 계약이 바뀌었습니다.');
      };
      const requestJson=await file.text();current();
      const record=await bridge.recordFiniteInput({sourceText:originalSource,requestJson,bindingJson:expectedBinding,
        authoredProjectJson:originalProject,manipulation:typedBinding(expectedBinding)});
      current();
      // No candidate, download or VM replacement escapes an admission error.
      const url=URL.createObjectURL(new Blob([Uint8Array.from(record.tape_bytes)],{type:'application/octet-stream'}));
      const link=document.createElement('a');link.href=url;link.download='input-v3-complete.bin';link.click();
      setTimeout(()=>URL.revokeObjectURL(url),1000);
      field('feedback').textContent='미리 정한 입력 요청의 모든 마디를 기록했습니다. 현재 세계에는 반영하지 않았습니다.';
    }catch(error){field('feedback').textContent=String(error);}
    finally{field('finite-request').value='';busy=false;onStateChange();}
  });
  panel.addEventListener('toggle', () => {
    if (!onPresentation && !panel.open && host) { pause(); host.loseInput('입력 영역을 접었습니다. 새로 시작하세요.'); show('입력 영역을 접어 실행이 중단됐습니다.'); }
  });
  panel.logicalExecution = {
    closeViewDialog(dialog) {
      // Closing an admitted view drawer is a deliberate local focus transfer,
      // not loss of world input. Detach only across this synchronous UI move;
      // real blur/pagehide/visibility loss still aborts the logical run.
      const retained=host,target=inputRegion??field('region');
      if(!retained){dialog.close();return;}
      retained.detach?.();
      try{dialog.close();target.focus();}
      finally{retained.attach(target,{controls:[...playbackControls,...viewControls,
        panel.querySelector('summary'),panel.querySelector('[data-action="seal"]'),field('value-controls'),
        ...(inputRegion?[field('region')]:[])]});}
    },
    async captureStorageContext() {
      await pauseAndSettle();await valueIntake;
      if(!session||!authoredSource||!host||host.failure||typedBinding(bindingJson))
        throw new Error('E_PROJECT_RUNTIME_LOGICAL: 정상 논리 입력 프로젝트 실행이 필요합니다.');
      unchanged();
      if(host.events.length||host.inFlight)throw new Error('E_LOGICAL_INPUT_PENDING: 전달하지 않은 입력을 먼저 한 마디 처리하세요.');
      const retained=session,token=generation,position=host.madi;
      const held=JSON.stringify(host.held);
      const requireCurrent=()=>{
        unchanged();
        if(token!==generation||session!==retained||running||host?.failure||host?.madi!==position
          ||host?.events.length||host?.inFlight||JSON.stringify(host?.held)!==held)
          throw new Error('E_PROJECT_RUNTIME_CHANGED: 저장 중 입력이나 실행이 바뀌었습니다.');
      };
      return {unchanged:requireCurrent,async export(parent,revision){
        requireCurrent();const artifact=await retained.exportSavedProjectArtifact(parent,revision);requireCurrent();
        const checkpoint=await retained.exportProjectCheckpoint(parent,revision);requireCurrent();
        return {artifact,checkpoint};
      },summary:()=>retained.summary(),async inputRequest(){
        requireCurrent();const summary=await retained.summary();requireCurrent();
        if(!Array.isArray(summary.frames)||summary.outcome==='ABORTED'||summary.madi_count!==summary.frames.length)
          throw new Error('E_NATIVE_SOURCE_LOGICAL_INPUTS: 정상 처리한 논리 입력만 저장할 수 있습니다.');
        return {binding:JSON.parse(bindingJson),snapshots:structuredClone(summary.frames.map(frame=>frame.input))};
      }};
    },
    storageInputBinding(){
      if(!boundAuthoredSource||readAuthoredProject()!==boundAuthoredSource||readSource()!==boundSource||typedBinding(field('binding').value))
        throw new Error('E_PROJECT_RUNTIME_LOGICAL: 현재 작품의 논리 입력 계약을 먼저 선택하세요.');
      return JSON.parse(field('binding').value);
    },
    async prepareStoredCheckpoint(record) {
      // Expected binding belongs to the selected project, never to the capsule.
      const selectedSource=readSource(),selectedAuthored=readAuthoredProject()||null;
      const selectedBinding=field('binding').value,selectedActions=field('actions').value,selectedCapacity=field('capacity').value;
      const token=generation;
      const previousSession=session,previousHost=host,previousMadi=host?.madi;
      const previousHeld=JSON.stringify(host?.held);
      if(!selectedAuthored||!selectedBinding||typedBinding(selectedBinding)
        ||selectedSource!==boundSource||selectedAuthored!==boundAuthoredSource)
        throw new Error('E_PROJECT_RUNTIME_LOGICAL: 현재 작품의 논리 입력 계약을 먼저 선택하세요.');
      const requireCurrent=()=>{
        if(token!==generation||session!==previousSession||host!==previousHost||host?.madi!==previousMadi
          ||running||host?.inFlight||host?.events.length||host?.failure||JSON.stringify(host?.held)!==previousHeld
          ||readSource()!==selectedSource||(readAuthoredProject()||null)!==selectedAuthored
          ||field('binding').value!==selectedBinding||field('actions').value!==selectedActions||field('capacity').value!==selectedCapacity)
          throw new Error('E_PROJECT_RUNTIME_CHANGED: 복원 중 작품이나 입력 계약이 바뀌었습니다.');
      };
      const bytes=hex=>Uint8Array.from(hex.match(/../g)??[],pair=>parseInt(pair,16));
      const nativeSummary=record.native_logical_summary;
      const candidate=await bridge.createLogicalSession({sourceText:selectedSource,bindingJson:selectedBinding,authoredProjectJson:selectedAuthored,
        ...(nativeSummary?{}:{savedCheckpoint:{artifact_text:record.artifact_text,checkpoint_bytes:record.checkpoint_bytes,
          parent_bytes:bytes(record.parent_hex),revision_bytes:bytes(record.revision_hex)}})});
      try {
        if(nativeSummary){
          if(!Array.isArray(nativeSummary.frames)||nativeSummary.outcome==='ABORTED'||nativeSummary.frames.length!==nativeSummary.madi_count)
            throw new Error('E_NATIVE_SOURCE_LOGICAL_INPUTS: 검증된 Native 입력 이력이 필요합니다.');
          // This is a fresh WASM presentation replay, never a conversion of
          // Native receipt history into a browser checkpoint/Native permit.
          for(const frame of nativeSummary.frames){requireCurrent();await candidate.step(frame.input);}
        }
        requireCurrent();const summary=await candidate.summary();requireCurrent();
        if(nativeSummary){
          const stable=value=>JSON.stringify(value,(_key,part)=>part&&typeof part==='object'&&!Array.isArray(part)
            ?Object.fromEntries(Object.entries(part).sort(([a],[b])=>a<b?-1:a>b?1:0)):part);
          if(stable(summary)!==stable(nativeSummary))throw new Error('E_NATIVE_SOURCE_LOGICAL_REPLAY_MISMATCH: Native와 화면의 입력·세계·출력이 다릅니다. 기존 화면을 보존했습니다.');
        }
        const displaySummary=await candidate.summary({includeView:true});requireCurrent();
        const binding=JSON.parse(selectedBinding),last=summary.frames?.at(-1)?.input;
        if(!last||summary.outcome==='ABORTED'||last.madi+1!==summary.madi_count)
          throw new Error('E_PROJECT_RUNTIME_LOGICAL: 완료된 마디와 실제 입력이 필요합니다.');
        const nextHost=new LogicalActionInput({session:candidate,channels:binding.channels,initialHeld:last.held,
          actionMap:JSON.parse(selectedActions),maxEvents:Number(selectedCapacity),nextMadi:summary.madi_count,
          finishBarrier:createNativeLogicalFinishBarrier({authored:selectedAuthored,binding,onCheckpoint:onNativeCheckpoint,
            onProgress:message=>{if(session===candidate&&host===nextHost)field('feedback').textContent=message;},unchanged:()=>{
            unchanged();if(session!==candidate||host!==nextHost)
              throw new Error('E_NATIVE_FINISH_SESSION_CHANGED: 다시 연 기록 생산자가 바뀌었습니다.');
          }})});
        return {summary,close:()=>candidate.close(),publish:()=>{
          requireCurrent();
          if(running||host?.inFlight||host?.events.length)
            throw new Error('E_LOGICAL_INPUT_PENDING: 현재 실행을 일시정지하고 입력 처리를 마치세요.');
          nextHost.attach(inputRegion??field('region'),{controls:[...playbackControls,...viewControls,
            panel.querySelector('summary'),field('advanced').querySelector(':scope > summary'),panel.querySelector('[data-action="seal"]'),field('value-controls'),
            panel.querySelector('[data-action="verify-receipt"]'),
            panel.querySelector('[data-action="new-record"]'),
            ...(inputRegion?[field('region')]:[])]});
          const old=session;host?.detach?.();pause();generation++;
          source=selectedSource;authoredSource=selectedAuthored;bindingJson=selectedBinding;
          actionsJson=selectedActions;capacityText=selectedCapacity;session=candidate;host=nextHost;
          renderValueControls(binding);show(displaySummary);onStateChange();
          return old?.close();
        }};
      }catch(error){await candidate.close();throw error;}
    },
    async exportSavedProjectArtifact(parent,revision) {
      await pauseAndSettle();await valueIntake;
      if(!session || !authoredSource || host?.failure)throw new Error('E_PROJECT_RUNTIME_LOGICAL: 정상 프로젝트 실행을 먼저 시작하세요.');
      unchanged();const token=generation;
      const artifact=await session.exportSavedProjectArtifact(parent,revision);
      unchanged();
      if(token!==generation)throw new Error('E_PROJECT_RUNTIME_CHANGED: 저장 중 실행이 바뀌었습니다.');
      return artifact;
    },
    async resetForProject() {
      await close();
      boundSource = null; source = ''; bindingJson = ''; actionsJson = ''; capacityText = '';
      authoredSource = null; boundAuthoredSource = null;
      panel.querySelector('[data-action="verify-receipt"]').hidden=true;
      panel.querySelector('[data-action="new-record"]').hidden=true;
      for (const name of ['binding','actions','capacity','replay','finite-request']) field(name).value = '';
      field('partial-replay').checked=false;
      field('partial-replay').disabled=false;
      field('contract-summary').textContent = '입력 계약을 선택하지 않았습니다.';
      field('result').textContent = ''; field('feedback').textContent = '';
      panel.open = false;
      panel.closest('dialog')?.close();
      onStateChange();
    },
    receiveValue(channel,payload) {
      if(!host)throw new Error('E_MANIPULATION_SESSION: ▶로 조작 실행을 먼저 시작하세요.');
      host.receiveValue(channel,payload);
    },
    async selectContract({bindingJson,actionsJson,capacity,reveal=true}) {
      await close();
      field('binding').value=bindingJson;
      field('actions').value=actionsJson;
      field('capacity').value=String(capacity);
      panel.open=reveal;
      await execute('bind-source');
    },
    ownsCurrentSource: () => boundSource !== null && readSource() === boundSource,
    status: () => ({ running, busy:busy || settling, started: Boolean(host), failed: Boolean(host?.failure) }),
    async command(action) {
      if (!onPresentation) panel.open = true;
      try {
        // Move focus within the admitted region before disabling a clicked
        // playback button; disabling a focused node may emit focusout(null).
        if (host) (inputRegion ?? field('region')).focus();
        if (action === 'primary') {
          if (host?.stopped) throw new Error('E_INPUT_RECORDING_STOPPED: 기록을 중단했습니다. 새로 시작하거나 저장된 작품을 다시 여세요.');
          if (running) return await execute('pause');
          if (!host || host.failure) await execute('start');
          if (inputRegion) inputRegion.focus();
          else field('canvas').hidden ? field('region').focus() : field('canvas').focus();
          await execute('play');
        } else await execute(action);
      } catch (error) { pause(); show(String(error)); }
      finally { onStateChange(); }
    },
  };
  if (playbackControls.length) {
    for (const action of ['start','play','pause','step','stop']) panel.querySelector(`[data-action="${action}"]`).hidden = true;
  }
  return panel;
}
