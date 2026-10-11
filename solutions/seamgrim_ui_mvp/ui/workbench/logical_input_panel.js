import { createRuntimeBridgeV1 } from '../runtime/runtime_bridge.js';
import { LogicalActionInput } from '../runtime/logical_action_input.js';
import { renderSpace2dCanvas2d } from '../wasm_page_common.js';
import { createAuthoredCompositionHost, renderAuthoredBogae } from './authored_bogae_renderer.js';

// Explicit, run-local Alpha2 contract intake. No source sniffing, project
// metadata defaults, new persisted schema, or Alpha1 execution replacement.
export function installLogicalInputPanel({ parent, readSource, openExample }) {
  const panel = document.createElement('details');
  panel.id = 'logical-input-panel';
  panel.style.cssText = 'margin:12px;padding:12px;border:1px solid #888;border-radius:8px';
  panel.innerHTML = `<summary>키보드로 실행 · 알파2 논리 입력</summary>
    <p>현재 편집 원문을 별도로 실행합니다. 승인된 논리 입력 계약과 행동 사상을 명시하세요.
    일반 실행·알파1 기록과는 별개입니다. 글자 입력과 반응 훅은 아직 지원하지 않습니다.</p>
    <button data-action="open-tetris" ${typeof openExample==='function'?'':'hidden'}>실행을 닫고 새 테트리스 작업 열기</button>
    <button data-action="tetris-contract">실행을 닫고 테트리스 입력 계약 선택</button>
    <button data-action="bind-source">실행을 닫고 현재 원문에 다시 결속</button>
    <p>테트리스: ←·→ 이동, ↓ 내리기, ↑ 회전, Space 떨어뜨리기, r 다시 시작.
    원문은 자동으로 바꾸지 않습니다. <a href="../../samples/alpha2_logical_tetris.ddn" target="_blank" rel="noopener">테트리스 원문 보기</a></p>
    <p data-field="contract-summary" style="overflow-wrap:anywhere">입력 계약을 명시적으로 선택하세요.</p>
    <details data-field="contract-editor"><summary>입력 계약 직접 편집</summary>
      <label>입력 계약(JSON) <textarea data-field="binding" rows="4"></textarea></label>
      <label>키→행동 사상(action_map.v1) <textarea data-field="actions" rows="4"></textarea></label>
      <label>한 마디 사건 용량 <input data-field="capacity" type="number" min="1" step="1"></label>
    </details>
    <div data-field="region" tabindex="0" aria-label="논리 행동 입력 영역">
      <button data-action="start">현재 원문으로 새로 시작</button>
      <button data-action="step">한 마디</button>
      <button data-action="play">60마디/초 재생</button>
      <button data-action="pause">일시정지</button>
      <button data-action="seal">논리 입력 기록 저장</button>
      <button data-action="stop">실행 닫기</button>
      <p>이 영역 안에서 지정한 키를 누르세요. 영역 밖으로 초점을 옮기면 입력 손실로 중단됩니다.</p>
      <div class="authored-view-toolbar" data-field="view-toolbar"></div>
      <div class="bogae-stage" data-field="view-stage">
        <div class="canvas-wrap"><canvas data-field="canvas" width="240" height="480" tabindex="0" hidden aria-label="원문이 만든 그림과 논리 입력 영역"></canvas></div>
      </div>
      <div data-field="view-list" hidden></div>
    </div>
    <label>봉인 논리 입력 파일 재생 <input data-field="replay" type="file"></label>
    <p data-field="feedback" role="status"></p>
    <details><summary>실행 결과와 영수증 자세히 보기</summary><pre data-field="result"></pre></details>`;
  panel.querySelectorAll('textarea').forEach(node => { node.style.cssText = 'display:block;width:100%;box-sizing:border-box'; });
  parent.append(panel);
  const field = name => panel.querySelector(`[data-field="${name}"]`);
  const bridge = createRuntimeBridgeV1();
  let host = null, session = null, source = '', bindingJson = '', actionsJson = '', capacityText = '';
  let timer = null, running = false, busy = false, generation = 0, inFlightPump = null;
  let committedView = null;
  const compositionHost = createAuthoredCompositionHost(() => renderView(committedView));
  function renderView(value) {
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
    const space2d = value?.view_meta?.space2d;
    canvas.hidden = !space2d;
    canvas.dataset.renderSource = space2d ? 'rust_legacy_drawlist' : '';
    if (space2d) renderSpace2dCanvas2d({canvas,space2d,yAxisDown:true,pad:0,showViewMetrics:false});
  }
  const show = value => {
    field('result').textContent = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
    field('feedback').textContent = typeof value === 'string' ? value
      : value?.frame ? `마디 ${value.frame.input.madi} 계산을 마쳤습니다.`
      : `실행 상태: ${value?.outcome ?? '준비'} · 계산한 마디 ${value?.madi_count ?? 0}`;
    const acceptedSummary = value?.madi_count > 0 && value?.outcome !== 'ABORTED' && value?.bogae_projection;
    if (value?.frame || acceptedSummary) { committedView = value; renderView(value); }
    else {
      compositionHost.reset(); committedView = null;
      field('canvas').hidden = true;
      field('canvas').dataset.renderSource = '';
    }
  };
  function pause() { running = false; clearTimeout(timer); timer = null; }
  async function pauseAndSettle() {
    pause();
    if(inFlightPump) await inFlightPump;
  }
  function unchanged() {
    if (readSource() !== source || field('binding').value !== bindingJson
      || field('actions').value !== actionsJson || field('capacity').value !== capacityText) {
      host?.loseInput('원문·입력 계약·키 사상 또는 사건 용량이 바뀌었습니다. 새로 시작하세요.');
      throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 원문·입력 계약·키 사상 또는 사건 용량이 바뀌었습니다.');
    }
  }
  async function close() {
    pause(); generation++;
    compositionHost.reset(); committedView = null;
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
        const originalBinding = field('binding').value;
        const originalActions = field('actions').value, originalCapacity = field('capacity').value;
        let chosenBinding = originalBinding, chosenActions = null;
        if (action === 'tetris-contract' || action === 'open-tetris') {
          [chosenBinding, chosenActions] = await Promise.all([
            readAsset('alpha2_logical_tetris.binding.json'), readAsset('alpha2_logical_tetris.action_map.json'),
          ]);
          JSON.parse(chosenActions);
        }
        const prepared = await bridge.bindLogicalSource({sourceText:chosenSource,bindingJson:chosenBinding});
        if (readSource() !== chosenSource || field('binding').value !== originalBinding
          || field('actions').value !== originalActions || field('capacity').value !== originalCapacity) {
          throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 결속 도중 원문 또는 계약이 바뀌었습니다. 다시 선택하세요.');
        }
        const bound = JSON.parse(prepared);
        field('binding').value = JSON.stringify(bound, null, 2);
        if (chosenActions !== null) {field('actions').value=chosenActions;field('capacity').value='128';}
        field('contract-summary').textContent = `${bound.source_profile} · 화면 ${bound.madi_hz}마디/초 · 세계 dt=1 · 원문 ${bound.source_revision}`;
        show('현재 원문에 결속했습니다. 입력 영역의 새로 시작을 누르세요. 아직 세계 마디는 실행하지 않았습니다.');
      } else if (action === 'start') {
        await close();
        source = readSource(); bindingJson = field('binding').value;
        actionsJson = field('actions').value; capacityText = field('capacity').value;
        const binding = JSON.parse(bindingJson);
        const actionMap = JSON.parse(actionsJson);
        const maxEvents = Number(capacityText);
        session = await bridge.createLogicalSession({ sourceText:source, bindingJson });
        try {
          unchanged();
          host = new LogicalActionInput({ session, channels:binding.channels, initialHeld:binding.initial_held, actionMap, maxEvents });
          host.attach(field('region'));
        } catch (error) { await close(); throw error; }
        const summary = await session.summary();
        unchanged();
        show(summary);
      } else if (action === 'step') { await pauseAndSettle(); await step(); }
      else if (action === 'play') {
        if (!host) throw new Error('E_LOGICAL_INPUT_SESSION: 먼저 새로 시작하세요.');
        unchanged(); pause(); running = true; void pump(generation);
      } else if (action === 'seal') {
        await pauseAndSettle(); unchanged();
        if (!host) throw new Error('E_LOGICAL_INPUT_SESSION: 먼저 새로 시작하세요.');
        const token = generation;
        const record = await host.seal();
        if (token !== generation) return;
        unchanged();
        const url = URL.createObjectURL(new Blob([Uint8Array.from(record.tape_bytes)], {type:'application/octet-stream'}));
        const link = document.createElement('a'); link.href = url; link.download = 'logical-input.tape'; link.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000); show(record.summary);
      } else if (action === 'stop') { await close(); show('실행을 닫았습니다.'); }
    } finally { busy = false; }
  }
  panel.querySelectorAll('[data-action]').forEach(button => button.addEventListener('click', () => {
    execute(button.dataset.action).catch(error => { pause(); show(String(error)); });
  }));
  field('replay').addEventListener('change', async () => {
    if (busy || running) { field('replay').value = ''; show('E_LOGICAL_INPUT_STEP_BUSY: 실행을 닫고 파일을 여세요.'); return; }
    busy = true;
    try {
      await close();
      const file = field('replay').files[0]; if (!file) return;
      if (file.size > 67108928) throw new Error('E_LOGICAL_INPUT_SIZE: 입력 기록의 크기 한도를 넘었습니다.');
      const replaySource = readSource(), replayBinding = field('binding').value, token = generation;
      const requireCurrent = () => {
        if (generation !== token || readSource() !== replaySource || field('binding').value !== replayBinding) {
          throw new Error('E_LOGICAL_INPUT_BINDING_CHANGED: 재생 도중 원문 또는 입력 계약이 바뀌었습니다. 다시 파일을 여세요.');
        }
      };
      const tapeBytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      requireCurrent();
      const summary = await bridge.replayLogicalInput({sourceText:replaySource, bindingJson:replayBinding, tapeBytes});
      requireCurrent();
      show(summary);
    } catch (error) { show(String(error)); }
    finally { field('replay').value = ''; busy = false; }
  });
  panel.addEventListener('toggle', () => {
    if (!panel.open && host) { pause(); host.loseInput('입력 영역을 접었습니다. 새로 시작하세요.'); show('입력 영역을 접어 실행이 중단됐습니다.'); }
  });
  return panel;
}
