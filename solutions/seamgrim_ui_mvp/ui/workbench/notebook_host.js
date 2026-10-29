import { createNotebookModel, createEquationAuthoringModel } from './notebook/model.js';
import { notebookResultHtml, comparisonRanges, equationAuthoringHtml, equationObservationHtml } from './notebook/view.js';
import { uniformMotionSourceContext } from './notebook/uniform_motion.js';
import { createNotebookReproductionPair } from './persistence/notebook_source.js';

export function installNotebookHost({ root, opener, runtimeBridge, readSource,
  readLocator, rememberLocator, applyReopenedLocator, readSavedSources = () => [], rememberSavedSource,
  readEquationContext, applyEquationSource, saveEquationSource }) {
  if (!root || !opener) return { invalidate() {}, dispose() {} };
  let provider = null, model = null, session = null, sessionId = '', mode = 'table', zoom = 1, pan = 0;
  let lastFocus = null, disposed = false, ioBusy = false, operationNotice = '';
  let pair = null, pairViews = {A:null,B:null};
  root.innerHTML = `<header class="notebook-toolbar"><h2>탐구 노트</h2>
    <button type="button" data-notebook="calculate">계산</button>
    <button type="button" data-notebook="pin">비교에 담기</button>
    <button type="button" data-notebook="save">원문 저장</button>
    <button type="button" data-notebook="open">저장 원문 열기</button>
    <button type="button" data-notebook="close" aria-label="탐구 노트 닫기" title="닫기">×</button></header>
    <p class="notebook-status" data-notebook="status" role="status"></p>
    <nav class="notebook-modes" aria-label="결과 보기"><button type="button" data-notebook-mode="table" aria-pressed="true">표</button>
    <button type="button" data-notebook-mode="graph" aria-pressed="false">그래프</button>
    <button type="button" data-notebook-mode="position" aria-pressed="false">위치</button></nav>
    <div class="notebook-viewport" data-notebook="viewport" hidden>
      <label>확대 <input type="range" data-notebook="zoom" min="1" max="4" step="0.25" value="1"></label>
      <label>가로 이동 <input type="range" data-notebook="pan" min="-1" max="1" step="0.1" value="0"></label>
      <button type="button" data-notebook="reset" title="그래프 범위 초기화" aria-label="그래프 범위 초기화">↺</button></div>
    <section data-notebook="result" aria-label="현재 계산 결과"></section>
    <section class="notebook-comparisons" data-notebook="comparisons" aria-label="두 실험 비교"></section>
    <button type="button" data-notebook="clear">비교 비우기</button>
    <button type="button" data-notebook="equation-toggle" aria-expanded="false">풀이 작성</button>
    <section data-notebook="equation" hidden></section>
    <fieldset><legend>두 실험의 재현 자료 · 과거 결과는 저장하지 않음</legend>
      <button type="button" data-notebook="pair-save">비교한 두 원문 저장</button>
      <label>실험 A <select data-notebook="saved-A"><option value="">내 작업에서 선택</option></select></label>
      <label>실험 B <select data-notebook="saved-B"><option value="">내 작업에서 선택</option></select></label>
      <button type="button" data-notebook="pair-open">두 저장 원문 열기</button>
      <button type="button" data-notebook="recompute-A">A 명시 재계산</button>
      <button type="button" data-notebook="recompute-B">B 명시 재계산</button>
    </fieldset>`;
  const element = name => root.querySelector(`[data-notebook="${name}"]`);
  const equationRoot = element('equation');
  equationRoot.innerHTML = equationAuthoringHtml();
  const eq = name => equationRoot.querySelector(`[data-equation="${name}"]`);
  let equationNotice = '', equationBusy = false;
  const equation = createEquationAuthoringModel({ readContext: readEquationContext,
    applySource: applyEquationSource,
    inspect: (source, path, options) => runtimeBridge.verifyAssumptionSource(source, path, options),
    changed: renderEquation });
  function renderEquation() {
    const state = equation.snapshot();
    if (eq('source').value !== state.draft) eq('source').value = state.draft;
    eq('status').textContent = state.error || equationNotice || (state.busy ? '단계 검사 중' : state.report
      ? state.report.status === 'accepted' ? '이번 원문의 조건부 단계 수용' : '이번 원문의 검사 거부' : 'NOT_PRODUCED · 명시 검사 전');
    eq('apply').disabled = !state.valid || !state.dirty || equationBusy || state.busy;
    eq('insert').disabled = !state.valid || equationBusy || state.busy;
    eq('save').disabled = !state.valid || state.dirty || equationBusy || state.busy || typeof saveEquationSource !== 'function';
    eq('open').disabled = equationBusy || !readLocator?.();
    eq('load').disabled = equationBusy || typeof readEquationContext !== 'function';
    eq('check').disabled = !state.valid || state.dirty || equationBusy || state.busy;
    try { eq('result').innerHTML = equationObservationHtml(state.report, state.context?.sourceText ?? ''); }
    catch (error) { eq('result').textContent = `표시 실패: ${error.message}`; }
  }
  function equationAction(fn) {
    try { equationNotice = ''; fn(); renderEquation(); }
    catch (error) { equationNotice = String(error.message ?? error); renderEquation(); }
  }
  element('equation-toggle').addEventListener('click', () => {
    equationRoot.hidden = !equationRoot.hidden;
    element('equation-toggle').setAttribute('aria-expanded', String(!equationRoot.hidden));
    renderEquation();
  });
  eq('load').addEventListener('click', () => equationAction(() => {
    if (equation.snapshot().dirty && !window.confirm('적용하지 않은 작성 원문을 현재 편집기 원문으로 바꿀까요?')) return;
    equation.load();
  }));
  eq('source').addEventListener('input', () => equationAction(() => equation.edit(eq('source').value)));
  eq('insert').addEventListener('click', () => equationAction(() => {
    const fields = Object.fromEntries(['operation', 'reference', 'term', 'left', 'right', 'operationName', 'claimName']
      .map(name => [name, eq(name).value]));
    fields.resultPlace = eq('resultPlace').checked;
    equation.insert(eq('source').selectionStart, eq('source').selectionEnd, fields);
  }));
  eq('apply').addEventListener('click', () => equationAction(() => equation.apply()));
  eq('check').addEventListener('click', async () => {
    equationNotice = '';
    try { await equation.check(); } catch { renderEquation(); }
  });
  async function equationIO(fn, notice) {
    if (equationBusy) return;
    equationBusy = true; equationNotice = ''; renderEquation();
    try { await fn(); equationNotice = notice; }
    catch (error) { equationNotice = String(error.message ?? error); }
    finally { equationBusy = false; renderEquation(); }
  }
  eq('save').addEventListener('click', () => equationIO(async () => {
    const state = equation.snapshot();
    if (!state.valid || state.dirty) throw new Error('작성 원문을 먼저 편집기에 적용하세요.');
    await saveEquationSource(state.context);
    equation.load();
  }, '원문 저장 확인 · NOT_PRODUCED'));
  eq('open').addEventListener('click', () => equationIO(async () => {
    if (equation.snapshot().dirty && !window.confirm('작성 중인 원문 대신 저장 원문을 열까요?')) throw new Error('재열기를 취소했습니다.');
    const locator = readLocator?.();
    if (!locator) throw new Error('저장 위치가 없습니다.');
    await applyReopenedLocator(locator);
    equation.load();
  }, '재열기 완료 · NOT_PRODUCED'));
  renderEquation();
  function render() {
    if (disposed) return;
    const state = model?.snapshot() ?? { current: null, comparisons: [], busy: false, error: '' };
    let ready = false;
    try { ready = Boolean(readSource()); } catch { /* Unsaved source is not a current canonical artifact. */ }
    element('calculate').disabled = !provider || !ready || state.busy || ioBusy;
    element('pin').disabled = !state.current || state.busy || ioBusy || state.comparisons.length >= 2;
    element('save').disabled = !ready || ioBusy || state.busy;
    element('open').disabled = ioBusy || state.busy || !readLocator?.();
    element('clear').disabled = !state.comparisons.length;
    element('pair-save').disabled = state.comparisons.length !== 2 || ioBusy || state.busy;
    element('pair-open').disabled = ioBusy || !element('saved-A').value || !element('saved-B').value;
    for (const slot of ['A','B']) element(`recompute-${slot}`).disabled = !provider || !pair?.snapshot().experiments[slot].projectId || ioBusy;
    element('status').textContent = state.error || operationNotice || (ioBusy ? '원문 처리 중' : !ready ? '현재 원문 저장 필요' : !provider
      ? '계산 기능 연결 준비 중' : state.busy ? '계산 중' : state.current ? '현재 원문 결과' : '재계산 전');
    const view = state.current?.view;
    element('result').replaceChildren();
    if (view?.inputSummary) { const p = document.createElement('p'); p.textContent = view.inputSummary; element('result').append(p); }
    element('viewport').hidden = mode !== 'graph';
    const visualization = document.createElement('div'); visualization.innerHTML = notebookResultHtml(view, mode, { zoom, pan }); element('result').append(visualization);
    if (view?.solutionMessage) { const p = document.createElement('p'); p.textContent = view.solutionMessage; element('result').append(p); }
    const ranges = comparisonRanges(state.comparisons.map(x => x.view));
    element('comparisons').innerHTML = state.comparisons.map((item, i) =>
      `<section aria-label="실험 ${i === 0 ? 'A' : 'B'}"><h3>실험 ${i === 0 ? 'A' : 'B'} · 이번 세션 결과</h3>${notebookResultHtml(item.view, mode, { ...ranges, zoom, pan, clipId: `notebook-compare-${i}` })}</section>`).join('');
    if (pair) {
      const ranges = comparisonRanges(Object.values(pairViews).filter(Boolean));
      element('comparisons').innerHTML = ['A','B'].map(slot => `<section aria-label="저장 실험 ${slot}"><h3>실험 ${slot} · ${pairViews[slot] ? '명시 재계산한 새 결과' : 'NOT_PRODUCED'}</h3>${notebookResultHtml(pairViews[slot], mode, {...ranges,zoom,pan,clipId:`saved-${slot}`})}</section>`).join('');
    }
    root.querySelectorAll('[data-notebook-mode]').forEach(button =>
      button.setAttribute('aria-pressed', String(button.dataset.notebookMode === mode)));
  }
  function buildModel() {
    model?.dispose();
    model = createNotebookModel({ readSource, inspect: provider?.inspect, project: provider?.project, changed: render });
  }
  async function sourceSession() {
    const raw = readSource();
    if (!raw) throw new Error('현재 작품 원문을 먼저 저장하세요.');
    const id = JSON.parse(raw)?.authored_identity?.logical_dependency_id;
    if (!id) throw new Error('실제 작품 정체가 필요합니다.');
    if (!session || sessionId !== id) {
      session?.dispose();
      const { createNotebookSourceSession } = await import('./persistence/notebook_source.js');
      if (disposed || readSource() !== raw) throw new Error('원문이 바뀌어 노트 작업을 취소했습니다.');
      session = createNotebookSourceSession({ runtimeBridge }); sessionId = id;
    }
    session.setSourceProject(raw);
    return session;
  }
  const reportError = error => { element('status').textContent = String(error?.message ?? error); };
  async function action(fn, successText) {
    if (ioBusy) return;
    operationNotice = ''; ioBusy = true; render();
    let failure = null;
    try { await fn(); } catch (error) { failure = error; }
    finally { ioBusy = false; operationNotice = failure ? String(failure.message ?? failure) : successText; render(); }
  }
  function savedChoices() {
    for (const slot of ['A','B']) {
      const select=element(`saved-${slot}`), previous=select.value;
      select.replaceChildren(new Option('내 작업에서 선택',''));
      for (const row of readSavedSources()) select.add(new Option(row.title || row.project_id,row.project_id));
      select.value=previous;
    }
  }
  function publishAcknowledgedSources() {
    const state=pair.snapshot();
    for (const slot of ['A','B']) if (state.confirmedThisAttempt[slot])
      rememberSavedSource?.(slot,state.locations[slot]);
    savedChoices();
  }
  element('pair-save').addEventListener('click',()=>action(async()=>{
    const compared=model.snapshot().comparisons;
    if(compared.length!==2)throw new Error('실제 계산한 두 원문을 먼저 비교에 담으세요.');
    const candidate=createNotebookReproductionPair({runtimeBridge});
    for(const [i,slot] of ['A','B'].entries()) {
      const raw=await runtimeBridge.authoredProjectOperation({operation:'clone',text:compared[i].source});
      candidate.setSourceProject(slot,raw);candidate.bindContext(slot,uniformMotionSourceContext(raw));
    }
    pair?.dispose();pair=candidate;pairViews={A:null,B:null};
    try {await pair.save();} finally {publishAcknowledgedSources();}
  },'두 재현 원문 저장 완료 · 과거 결과는 저장하지 않음'));
  for(const slot of ['A','B'])element(`saved-${slot}`).addEventListener('change',render);
  element('pair-open').addEventListener('click',()=>action(async()=>{
    const rows=readSavedSources(),receipts={};
    for(const slot of ['A','B']) {
      const row=rows.find(row=>row.project_id===element(`saved-${slot}`).value);
      if(!row?.native_source_locator)throw new Error('실제 Native 저장 위치를 선택하세요.');
      receipts[slot]=row.native_source_locator;
    }
    const candidate=createNotebookReproductionPair({runtimeBridge});
    try {await candidate.open(receipts);} catch(error) {candidate.dispose();throw error;}
    pair?.dispose();pair=candidate;pairViews={A:null,B:null};model.reopened();
  },'두 원문 재열기 완료 · 각각 명시 재계산 필요'));
  for(const slot of ['A','B'])element(`recompute-${slot}`).addEventListener('click',()=>action(async()=>{
    pairViews[slot]=null;
    const raw=pair.snapshot().experiments[slot].sourceProjectText;
    pair.bindContext(slot,uniformMotionSourceContext(raw));
    const state=await pair.recompute(slot,provider.inspectReproduction),view=provider.project(state.experiments[slot].report,raw);
    const other=pairViews[slot==='A'?'B':'A'];
    if(other && (JSON.stringify(other.comparisonKey)!==JSON.stringify(view.comparisonKey)
      || JSON.stringify(other.comparedInput)===JSON.stringify(view.comparedInput)))throw new Error('같은 모형·의존·시각·목표에서 속도만 바꾼 두 실험이 아닙니다.');
    pairViews[slot]=view;
  },`${slot} 명시 재계산 완료 · 새 결과`));
  element('calculate').addEventListener('click', () => { operationNotice = ''; model.calculate().catch(reportError); });
  element('pin').addEventListener('click', () => { try { model.pin(); } catch (e) { reportError(e); } });
  element('clear').addEventListener('click', () => model.clearComparisons());
  element('zoom').addEventListener('input', event => { zoom = Number(event.target.value); render(); });
  element('pan').addEventListener('input', event => { pan = Number(event.target.value); render(); });
  element('reset').addEventListener('click', () => {
    zoom = 1; pan = 0; element('zoom').value = '1'; element('pan').value = '0'; render();
  });
  element('save').addEventListener('click', () => action(async () => {
    const active = await sourceSession();
    active.bindContext(uniformMotionSourceContext(readSource()));
    const locator = await active.saveNotebook();
    rememberLocator(locator);
  }, '원문 저장 완료'));
  element('open').addEventListener('click', () => action(async () => {
    const locator = readLocator();
    if (!locator) throw new Error('저장 위치가 없습니다.');
    // Existing Workbench admission applies source to the shared editor.
    // Nothing here executes it or treats old observations as current.
    await applyReopenedLocator(locator);
    const active = await sourceSession();
    await active.open(locator);
    if (readSource() !== active.snapshot().sourceProjectText) throw new Error('재열기 원문이 일치하지 않습니다.');
    model.reopened();
  }, '재열기 완료 · 재계산 전'));
  root.querySelectorAll('[data-notebook-mode]').forEach(button => button.addEventListener('click', () => {
    mode = button.dataset.notebookMode; render();
  }));
  opener.addEventListener('click', () => { lastFocus = document.activeElement; savedChoices(); if (!root.open) root.showModal(); render(); });
  element('close').addEventListener('click', () => root.close());
  root.addEventListener('close', () => lastFocus?.focus());
  buildModel(); render();
  const api = {
    // Bind only the actual common producer and its explicit display projection.
    // No bridge method name or result wire is guessed before that supply.
    connectProducer(value) {
      if (typeof value?.inspect !== 'function' || typeof value?.project !== 'function') throw new TypeError('실제 계산·표시 공급자가 필요합니다.');
      provider = { ...value, inspectReproduction: value.inspect, inspect: async raw => {
        const active = await sourceSession();
        if (readSource() !== raw) throw new Error('원문이 바뀌었습니다.');
        return (await active.recheck(value.inspect)).report;
      } };
      buildModel(); render();
    },
    invalidate() {
      equation.invalidate();
      operationNotice = ''; model.invalidate();
      session?.dispose(); session = null; sessionId = '';
      pair?.dispose(); pair = null; pairViews = {A:null,B:null}; render();
    },
    dispose() { disposed = true; model.dispose(); equation.dispose(); session?.dispose(); pair?.dispose(); },
  };
  // The integrator explicitly connects a fixed common producer after supply.
  // Until then, no math report is produced and calculation remains disabled.
  return api;
}
