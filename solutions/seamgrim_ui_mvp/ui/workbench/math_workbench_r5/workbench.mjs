import { mountLinearProblemPanel } from './linear_problem_panel.mjs';
import { renderGoalEvidence, renderAlgebraSourceNavigation, conditionalAlgebraHtml } from './goal_evidence.mjs';
// 사람이 읽는 학습 기록이다. 재입력·판정 복원·작품 저장 형식이 아니다.
export function learningRecordText(entries, baseline = null) {
  const lines = ['수학 탐구 기록', '학습자 해석은 미검증입니다. 원래 응답과 구별하세요. 이 파일은 실행·증명·작품 저장 또는 검사 결과 복원이 아닙니다.'];
  if (baseline && entries.includes(baseline)) lines.push(`비교 기준: 기록 ${entries.indexOf(baseline) + 1} · 학습자가 선택한 기준이며 정답 판정이 아닙니다.`);
  for (const [index, entry] of entries.entries()) {
    lines.push('', `===== 기록 ${index + 1} =====`, entry.description ?? '',
      '실행 전 예상 · 미검증', entry.prediction || '(작성하지 않음)',
      '실행 후 해석 · 미검증', entry.reflection || '(작성하지 않음)',
      '당시 요청과 원래 응답 · JSON 문자열 안의 원문/줄바꿈 보존',
      JSON.stringify(entry.state, null, 2));
  }
  return lines.join('\n');
}
// 두 실행 기록의 원문과 공급 응답만 읽는다. 현재 초안·수학 판정은 변경하지 않는다.
export function recordedComparisonText(baseline, selected) {
  if (!baseline || !selected) return '';
  const difference = compareSourceLines(baseline.state.request.sourceText, selected.state.request.sourceText);
  return [
    baseline === selected ? '기준과 같은 기록입니다. 비교할 다른 기록을 선택하세요.' : '두 과거 요청의 비교입니다. 현재 초안의 검사 결과가 아닙니다.',
    '원문이 같아도 동작·선택 대상·원문 문맥이 다르면 같은 검사라는 뜻은 아닙니다. 조건 하나만 바뀌었는지는 원문과 요청을 읽어 확인하세요. 수치 차이·동치·인과관계·전체 해를 여기서 판정하지 않습니다.',
    difference ? `원문 차이: ${difference.startLine}줄부터\n기준 원문 범위\n${difference.before}\n선택 원문 범위\n${difference.after}` : '요청 원문은 같습니다.',
    '기준 기록 · 당시 요청/선택 대상/원래 응답', JSON.stringify(baseline.state, null, 2),
    '선택 기록 · 당시 요청/선택 대상/원래 응답', JSON.stringify(selected.state, null, 2),
  ].join('\n\n');
}
// 원문 줄의 차이만 비교한다. 식 해소·가정 분류·수학적 동치 판정이 아니다.
export function compareSourceLines(before, after) {
  if (before === after) return null;
  const left = before.split('\n'), right = after.split('\n');
  let start = 0, tail = 0;
  while (start < Math.min(left.length, right.length) && left[start] === right[start]) start++;
  while (tail < Math.min(left.length, right.length) - start
      && left[left.length - 1 - tail] === right[right.length - 1 - tail]) tail++;
  return { startLine: start + 1,
    beforeEndLine: left.length - tail, afterEndLine: right.length - tail,
    before: left.slice(start, left.length - tail).join('\n'),
    after: right.slice(start, right.length - tail).join('\n') };
}
// 본체 기능은 주입한다. 이 모듈은 원문을 파싱하거나 수학 판정을 만들지 않는다.
// 고정 엔진의 보임 진단에 포함된 UTF-8 범위를 textarea의 UTF-16 선택으로 바꾼다.
export function proofErrorSelection(error, request) {
  if (typeof error !== 'string' || typeof request?.sourceText !== 'string'
      || typeof request.sourcePath !== 'string' || !request.sourcePath) return null;
  const prefix = request.sourcePath + ':';
  if (!error.startsWith(prefix)) return null;
  const match = /^(\d+)\.\.(\d+) E_[A-Z0-9_]+:/.exec(error.slice(prefix.length));
  if (!match) return null;
  const startByte = Number(match[1]), endByte = Number(match[2]);
  return byteSelection(request.sourceText, startByte, endByte);
}
export function diagnosticSelection(diagnostic, request) {
  if (!diagnostic || typeof request?.sourceText !== 'string' || !request.sourcePath
      || diagnostic.source_path !== request.sourcePath || !diagnostic.source_span) return null;
  return byteSelection(request.sourceText, diagnostic.source_span.start, diagnostic.source_span.end);
}
function byteSelection(source, startByte, endByte) {
  if (!Number.isSafeInteger(startByte) || !Number.isSafeInteger(endByte) || startByte > endByte) return null;
  const positions = new Map([[0, 0]]);
  let bytes = 0, units = 0;
  const encoder = new TextEncoder();
  for (const character of source) {
    bytes += encoder.encode(character).length; units += character.length;
    positions.set(bytes, units);
  }
  if (!positions.has(startByte) || !positions.has(endByte)) return null;
  // textarea는 CRLF와 단독 CR을 LF로 표시한다. 엔진에 보낸 원문은 바꾸지 않는다.
  const editorOffset = offset => source.slice(0, offset).replace(/\r\n?/g, '\n').length;
  // CRLF 두 바이트 사이를 별도 화면 위치로 가장하지 않는다.
  const insidePair = offset => source[offset - 1] === '\r' && source[offset] === '\n';
  const start = positions.get(startByte), end = positions.get(endByte);
  if (insidePair(start) || insidePair(end)) return null;
  return { start: editorOffset(start), end: editorOffset(end) };
}
function errorRecord(error) {
  const raw = error instanceof Error ? error.message : typeof error === 'string' ? error : JSON.stringify(error);
  let diagnostic = null;
  try { const value = JSON.parse(raw); if (value && typeof value === 'object' && !Array.isArray(value)) diagnostic = value; } catch {}
  return { error: raw ?? String(error), diagnostic };
}
async function checkObservationSource(result, request) {
  const bytes = new TextEncoder().encode(request.sourceText);
  const received = { source_path: result?.source_path, source_bytes: result?.source_bytes, source_sha256: result?.source_sha256 };
  const reject = () => { throw JSON.stringify({
    code: 'UI_OBSERVATION_SOURCE_MISMATCH',
    message: '목표·근거 응답의 원문이 이번 요청과 일치하지 않습니다. 현재 작품 원문을 확인하고 다시 명시 관찰하세요.',
    received_source: received,
  }); };
  if (received.source_path !== request.sourcePath || received.source_bytes !== bytes.length
      || typeof received.source_sha256 !== 'string' || !/^[0-9a-f]{64}$/.test(received.source_sha256)) reject();
  if (!globalThis.crypto?.subtle) throw new Error('원문 무결성 확인 기능을 사용할 수 없어 목표·근거 응답을 표시하지 않습니다.');
  const digest = await globalThis.crypto.subtle.digest('SHA-256', bytes);
  const hash = Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, '0')).join('');
  if (received.source_sha256 !== hash) reject();
}
export function createWorkbench({ runSummaryWithSourceProfile, verifyAssumptionSource, observeAssumptionSource, observeAlgebraSource }) {
  if (typeof runSummaryWithSourceProfile !== 'function') throw new TypeError('실행 기능이 필요합니다.');
  let generation = 0, disposed = false, request = null;
  let state = { status: 'empty', result: null, error: null };
  const listeners = new Set();
  const snapshot = () => structuredClone(state);
  function publish(next) {
    state = next;
    for (const listener of listeners) listener(snapshot());
  }
  return {
    snapshot,
    subscribe(listener) {
      if (disposed) throw new Error('종료된 작업대입니다.');
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    setRequest(next) {
      if (disposed) return;
      generation++;
      request = null;
      publish({ status: 'empty', result: null, error: null });
      if (next === null) return;
      if (typeof next?.sourceText !== 'string' || typeof next?.sourcePath !== 'string'
          || !next.sourcePath || typeof next?.sourceProfileIdentity !== 'string'
          || !next.sourceProfileIdentity) throw new TypeError('원문·경로·명시 프로필이 필요합니다.');
      request = structuredClone(next);
      publish({ status: 'edited', result: null, error: null });
    },
    async run(mode = 'execute') {
      if (disposed || request === null) return;
      const ticket = ++generation;
      const submitted = structuredClone(request);
      publish({ status: 'running', result: null, error: null });
      if (disposed || ticket !== generation) return;
      try {
        if (!['execute', 'proof', 'evidence', 'algebra'].includes(mode)) throw new Error('지원하지 않는 명시 동작입니다.');
        if (mode === 'evidence' && typeof observeAssumptionSource !== 'function') throw new Error('목표별 관찰 기능이 공급되지 않았습니다.');
        if (mode === 'proof' && typeof verifyAssumptionSource !== 'function') throw new Error('보임 검사 기능이 공급되지 않았습니다.');
        if (mode === 'algebra' && typeof observeAlgebraSource !== 'function') throw new Error('대수 단계 관찰 기능이 공급되지 않았습니다.');
        const supplied = mode === 'algebra' ? await observeAlgebraSource(submitted.sourceText, submitted.sourcePath) : mode === 'evidence' ? await observeAssumptionSource(submitted.sourceText, submitted.sourcePath) : mode === 'proof'
          ? await verifyAssumptionSource(submitted.sourceText, submitted.sourcePath)
          : await runSummaryWithSourceProfile(submitted);
        if (disposed || ticket !== generation) return;
        const result = structuredClone(supplied);
        if (mode === 'evidence' || mode === 'algebra') await checkObservationSource(result, submitted);
        if (disposed || ticket !== generation) return;
        publish({ status: 'returned', mode, request: submitted, result, error: null });
      } catch (error) {
        if (disposed || ticket !== generation) return;
        publish({ status: 'error', mode, request: submitted, result: null, ...errorRecord(error) });
      }
    },
    dispose() { disposed = true; generation++; request = null; state = { status: 'disposed', result: null, error: null }; listeners.clear(); },
  };
}

export function mountMathWorkbench(root, { runSummaryWithSourceProfile, verifyAssumptionSource, observeAssumptionSource, observeAlgebraSource, renderAlgebraObservation, linearProblemSupport, mountObservationView, initialRequest, readContext, applySource }) {
  const document = root.ownerDocument;
  const container = document.createElement('section');
  const label = document.createElement('label');
  label.textContent = '수학 원문';
  const editor = document.createElement('textarea');
  editor.rows = 14; editor.cols = 75;
  label.append(editor);
  const predictionLabel = document.createElement('label');
  predictionLabel.textContent = '실행 전 예상과 이유 — 선택 입력';
  const prediction = document.createElement('textarea'); prediction.rows = 3; prediction.maxLength = 4000;
  predictionLabel.append(prediction);
  const predictionHelp = document.createElement('p');
  predictionHelp.textContent = '학습자 메모이며 엔진에 전송하거나 채점하지 않습니다. 원문·동작 변경 시 비워집니다. 비교 기록에는 실행 당시 메모만 남고 작품에 저장되지 않습니다.';
  const submittedPrediction = document.createElement('p'); submittedPrediction.setAttribute('aria-label', '이번 요청의 실행 전 예상');
  let predictionAtRun = '';
  const previousPrediction = document.createElement('p'); previousPrediction.setAttribute('aria-label', '이전 요청의 실행 전 예상');
  const button = document.createElement('button');
  button.type = 'button'; button.textContent = '명시 실행';
  const loadSource = document.createElement('button'), applyDraft = document.createElement('button');
  loadSource.type = applyDraft.type = 'button';
  loadSource.textContent = '현재 작품 원문 읽기'; applyDraft.textContent = '편집기에 적용';
  loadSource.hidden = applyDraft.hidden = typeof readContext !== 'function' || typeof applySource !== 'function';
  const sourceStatus = document.createElement('p'); sourceStatus.setAttribute('role', 'status');
  const modeLabel = document.createElement('label'); modeLabel.textContent = '동작';
  const mode = document.createElement('select');
  for (const [value, text] of [['execute', '원문 실행'], ['proof', '가정 의존 보임 검사'], ['evidence', '편집 원문의 목표·근거 관찰'], ['algebra', '편집 원문의 대수 단계·재사용 관찰']]) {
    if (value === 'proof' && typeof verifyAssumptionSource !== 'function') continue;
    if (value === 'algebra' && (typeof observeAlgebraSource !== 'function' || typeof renderAlgebraObservation !== 'function')) continue;
    if (value === 'evidence' && typeof observeAssumptionSource !== 'function') continue;
    const option = document.createElement('option'); option.value = value; option.textContent = text; mode.append(option);
  }
  modeLabel.append(mode);
  const draftShelf=document.createElement('details');
  const draftTitle=document.createElement('summary');draftTitle.textContent='검사 전 초안 두 개를 오가며 작성하기';draftShelf.append(draftTitle);
  const draftHelp=document.createElement('p');draftHelp.textContent='함수 원문과 구간 식의 보임처럼 번갈아 수정할 원문을 임시로 보관합니다. 원문과 동작만 보관하며 검사 결과·선택 대상·증거는 보관하지 않습니다. 작품 원문 다시 읽기·파일/예제 전환·닫기 때 비워집니다. 필요한 원문은 각각 내려받으세요. 작품 저장이 아닙니다.';draftShelf.append(draftHelp);
  const draftSlots=[null,null], draftDisplays=[];let pendingDraft=null;
  const draftStatus=document.createElement('p');draftStatus.setAttribute('role','status');
  const draftConfirm=document.createElement('button'),draftCancel=document.createElement('button');
  draftConfirm.type=draftCancel.type='button';draftConfirm.textContent='현재 초안을 보관 원문으로 교체';draftCancel.textContent='보관 초안 가져오기 취소';draftConfirm.hidden=draftCancel.hidden=true;
  function cancelDraft(){pendingDraft=null;draftConfirm.hidden=draftCancel.hidden=true;}
  function clearDrafts(){cancelDraft();draftSlots.fill(null);draftDisplays.forEach(view=>{view.textContent='비어 있음';});draftStatus.textContent='';}
  function draftContextValid(){
    if(disposed||sourceStale||!context)return false;
    if(sourceContext){let current;try{current=readContext();}catch{}if(!sameSourceContext(sourceContext,current)){clearDrafts();draftStatus.textContent='작품 원문이 바뀌어 임시 초안을 폐기했습니다. 현재 원문을 다시 읽으세요.';return false;}}
    return true;
  }
  for(let index=0;index<2;index++){
    const box=document.createElement('section'),title=document.createElement('h4');title.textContent=`보관 초안 ${index+1}`;
    const capture=document.createElement('button'),restore=document.createElement('button'),remove=document.createElement('button'),view=document.createElement('pre');
    capture.type=restore.type=remove.type='button';capture.textContent=`현재 원문을 초안 ${index+1}에 보관·교체`;restore.textContent=`보관 초안 ${index+1} 가져오기`;remove.textContent=`보관 초안 ${index+1} 지우기`;view.textContent='비어 있음';draftDisplays.push(view);
    capture.onclick=()=>{if(!draftContextValid())return;cancelDraft();draftSlots[index]={request:structuredClone(context),mode:mode.value};view.textContent=`${context.sourcePath} · ${mode.selectedOptions[0]?.textContent??mode.value}\n${context.sourceText}`;draftStatus.textContent=`초안 ${index+1}의 원문과 동작을 보관했습니다. 실행·검사·저장하지 않았습니다.`;};
    restore.onclick=()=>{if(!draftContextValid()||!draftSlots[index])return;pendingDraft={index,entry:draftSlots[index],generation:draftGeneration};draftConfirm.hidden=draftCancel.hidden=false;draftStatus.textContent='현재 편집 원문을 바꿉니다. 필요한 수정본은 다른 초안이나 파일에 먼저 보관하세요. 검사 결과는 복원하지 않습니다.';};
    remove.onclick=()=>{cancelDraft();draftSlots[index]=null;view.textContent='비어 있음';draftStatus.textContent=`초안 ${index+1}을 지웠습니다. 현재 편집 원문은 그대로입니다.`;};
    box.append(title,capture,restore,remove,view);draftShelf.append(box);
  }
  draftCancel.onclick=()=>{cancelDraft();draftStatus.textContent='현재 초안을 유지했습니다.';};
  draftConfirm.onclick=()=>{
    const pending=pendingDraft;cancelDraft();if(!draftContextValid()||!pending)return;
    const entry=pending.entry;
    if(pending.generation!==draftGeneration||draftSlots[pending.index]!==entry||entry.request.sourcePath!==context.sourcePath||entry.request.sourceProfileIdentity!==context.sourceProfileIdentity){draftStatus.textContent='편집 문맥이 달라졌습니다. 보관 초안을 다시 선택하세요.';return;}
    context=structuredClone(entry.request);editor.value=context.sourceText;mode.value=entry.mode;changeMode();editor.focus();
    draftStatus.textContent=`보관 초안 ${pending.index+1}의 원문만 가져왔습니다. 명시 실행·검사가 필요합니다. B 대상은 다시 읽고 선택하세요.`;
  };
  draftShelf.append(draftStatus,draftConfirm,draftCancel);
  const status = document.createElement('p'); status.setAttribute('role', 'status');
  const resultContext = document.createElement('p');
  resultContext.setAttribute('aria-label', '결과의 원문과 동작');
  resultContext.hidden = true;
  const output = document.createElement('pre');
  const goalEvidence = document.createElement('section'); goalEvidence.setAttribute('aria-label', '편집 원문의 목표와 근거');
  const locate = document.createElement('button');
  locate.type = 'button'; locate.textContent = '오류 원문으로 이동'; locate.hidden = true;
  const note = document.createElement('p');
  note.textContent = '출력은 원문의 보여주기 순서입니다. 동치 비교의 참·거짓은 사용자 보임 발급이나 방정식의 해 판정이 아닙니다.';
  const details = document.createElement('details');
  const summary = document.createElement('summary'); summary.textContent = '요청과 원래 결과 확인';
  const raw = document.createElement('pre'); details.append(summary, raw);
  const keepResult = document.createElement('button');
  keepResult.type = 'button'; keepResult.textContent = '결과를 비교용으로 남기기'; keepResult.disabled = true;
  const previous = document.createElement('details'); previous.hidden = true;
  const previousTitle = document.createElement('summary'); previousTitle.textContent = '이전 요청 기록 — 현재 검사 결과가 아닙니다';
  const previousNote = document.createElement('p');
  previousNote.textContent = '최대 8건을 이 화면에서만 보관합니다. 오류도 원문과 함께 남깁니다. 작품 저장에 포함되지 않으며 다른 원문·방법의 결과를 대신하지 않습니다.';
  const history = [];
  let comparisonBaseline = null;
  const pinBaseline = document.createElement('button'); pinBaseline.type = 'button'; pinBaseline.textContent = '선택 기록을 비교 기준으로 고정';
  const unpinBaseline = document.createElement('button'); unpinBaseline.type = 'button'; unpinBaseline.textContent = '비교 기준 해제';
  const pairComparison = document.createElement('section'); pairComparison.setAttribute('aria-label', '고정 기준과 선택 기록 비교'); pairComparison.hidden = true;
  const pairTitle = document.createElement('h3'); pairTitle.textContent = '기준 실험과 변경 실험의 기록 비교';
  const pairStatus = document.createElement('p');
  const pairEvidence = document.createElement('section'); pairEvidence.setAttribute('aria-label', '두 기록의 판정과 원문 근거');
  let pairObservationViews = [];
  const pairOutput = document.createElement('pre'); pairOutput.style.whiteSpace = 'pre-wrap'; pairOutput.style.overflowWrap = 'anywhere';
  const pairRaw = document.createElement('details'), pairRawTitle = document.createElement('summary'), pairRawOutput = document.createElement('pre');
  pairRawTitle.textContent = '두 기록의 요청·선택 대상·원래 응답 확인';
  pairRawOutput.style.whiteSpace = 'pre-wrap'; pairRawOutput.style.overflowWrap = 'anywhere';
  pairRaw.append(pairRawTitle, pairRawOutput);
  pairComparison.append(pairTitle, pairStatus, pairOutput, pairEvidence, pairRaw, unpinBaseline);
  function showRecordedComparison() {
    for (const view of pairObservationViews) view.dispose();
    pairObservationViews = [];
    const selected = history[Number(historySelect.value)];
    pairComparison.hidden = !comparisonBaseline || !selected || disposed || sourceStale;
    pairOutput.textContent = pairStatus.textContent = pairRawOutput.textContent = ''; pairRaw.open = false; pairEvidence.replaceChildren();
    if (pairComparison.hidden) return;
    pairStatus.textContent = `기준 기록 ${history.indexOf(comparisonBaseline) + 1} → 선택 기록 ${history.indexOf(selected) + 1}`;
    const difference = compareSourceLines(comparisonBaseline.state.request.sourceText, selected.state.request.sourceText);
    pairOutput.textContent = [
      comparisonBaseline === selected ? '같은 기록입니다. 다른 기록을 선택해 비교하세요.' : '현재 초안과 독립적인 두 과거 응답입니다.',
      '기준 기록', comparisonBaseline.description, comparisonBaseline.display,
      '선택 기록', selected.description, selected.display,
      difference ? `바뀐 원문 · ${difference.startLine}줄부터\n기준:\n${difference.before}\n선택:\n${difference.after}` : '요청 원문은 같습니다. 동작과 선택 대상도 확인하세요.',
      '두 응답의 조건·방법·목표를 읽고 아래 해석에 비교 결과를 적으세요. 이 비교는 동치·인과관계·정답 판정이 아닙니다.',
    ].join('\n\n');
    pairRawOutput.textContent = recordedComparisonText(comparisonBaseline, selected);
    const baseline = comparisonBaseline;
    for (const [label, entry] of [['기준 기록', baseline], ['선택 기록', selected]]) {
      if (entry.state.mode === 'execute' && typeof mountObservationView === 'function') {
        const box = document.createElement('section'); box.setAttribute('aria-label', `${label}의 공급된 표`);
        const title = document.createElement('h4'); title.textContent = `${label} · 당시 실행의 표`;
        const context = document.createElement('p'); context.textContent = `${entry.state.request.sourcePath} · 다른 기록의 표와 합치거나 대응 행을 추정하지 않습니다.`;
        const body = document.createElement('section'); box.append(title, context, body); pairEvidence.append(box);
        if (entry.state.result) {
          const view = mountObservationView(body, {readState: () =>
            disposed || sourceStale || comparisonBaseline !== baseline || history[Number(historySelect.value)] !== selected
              ? null : entry.state.result});
          pairObservationViews.push(view);
        } else {
          body.textContent = '이 요청은 성공 결과표를 공급하지 않았습니다. 위의 실제 오류와 원래 응답을 확인하세요.';
        }
        continue;
      }
      if (!['evidence','algebra','linear'].includes(entry.state.mode) || !entry.state.result) continue;
      const box = document.createElement('details'), title = document.createElement('summary');
      const result = entry.state.result;
      const status = {accepted:'수용', rejected:'거부', not_checked:'미검사'}[result.status] ?? '상태 상세 확인';
      title.textContent = `${label}의 판정·목표·가정 — ${status}`; box.append(title);
      const body = document.createElement('section'); body.setAttribute('aria-label', `${label}의 공급된 근거`);
      const sourceLabel = document.createElement('label'); sourceLabel.textContent = `${label}의 당시 원문 — 읽기 전용`;
      const source = document.createElement('textarea'); source.readOnly = true; source.rows = 8; source.value = entry.state.request.sourceText;
      sourceLabel.append(source);
      const location = document.createElement('p'); location.setAttribute('role','status');
      const locate = span => {
        if (disposed || sourceStale || comparisonBaseline !== baseline || history[Number(historySelect.value)] !== selected) return;
        const range = diagnosticSelection({source_path: result.source_path, source_span:span}, entry.state.request);
        if (!range) return;
        source.setSelectionRange(range.start,range.end); source.focus();
        location.textContent = `${label}의 원문 위치입니다. 현재 초안·검사 결과는 바뀌지 않습니다.`;
      };
      if (entry.state.mode === 'evidence') renderGoalEvidence(body, result, locate, entry.state.request.sourceText);
      else showAlgebra(body, result, entry.state.request.sourceText, locate);
      box.append(body, sourceLabel, location); pairEvidence.append(box);
    }
  }
  pinBaseline.onclick = () => {
    if (disposed || sourceStale) return;
    comparisonBaseline = history[Number(historySelect.value)] ?? null;
    showRecordedComparison();
  };
  unpinBaseline.onclick = () => { comparisonBaseline = null; showRecordedComparison(); };
  const historyLabel = document.createElement('label'); historyLabel.textContent = '비교할 이전 요청';
  const historySelect = document.createElement('select'); historySelect.setAttribute('aria-label', '비교할 이전 요청'); historyLabel.append(historySelect);
  const historyStatus = document.createElement('p'); historyStatus.setAttribute('role', 'status');
  const reflectionLabel = document.createElement('label'); reflectionLabel.textContent = '선택 기록의 실행 후 해석 — 미검증 학습자 메모';
  const reflection = document.createElement('textarea'); reflection.rows = 5; reflection.maxLength = 4000;
  reflectionLabel.append(reflection);
  const reflectionHelp = document.createElement('p');
  reflectionHelp.textContent = '예상과 무엇이 달랐나요? 원문에서 바꾼 조건과 실제 응답을 근거로 적고, 아직 결론 내릴 수 없는 점과 다음 확인을 덧붙이세요. 구간 거부는 해 없음이 아니고, 근사값은 정확 해·모든 해·보임 증거가 아닙니다. 메모는 선택한 과거 기록에만 붙으며 검사 결과를 바꾸지 않습니다.';
  const downloadRecords = document.createElement('button'); downloadRecords.type = 'button'; downloadRecords.textContent = '원문·응답·학습자 메모 내려받기';
  downloadRecords.onclick = () => {
    if (disposed || sourceStale || !history.length) return;
    const url = URL.createObjectURL(new Blob([learningRecordText(history, comparisonBaseline)], { type: 'text/plain;charset=utf-8' }));
    const link = document.createElement('a'); link.href = url; link.download = '수학탐구_학습기록.txt';
    container.append(link); link.click(); link.remove(); setTimeout(() => URL.revokeObjectURL(url), 1000);
    historyStatus.textContent = '학습 기록 다운로드를 요청했습니다. 작품 저장·검사 인증·재열기 완료가 아닙니다. 기록을 지우기 전 내려받은 파일을 확인하세요.';
  };
  reflection.oninput = () => {
    if (disposed || sourceStale) return;
    const entry = history[Number(historySelect.value)];
    if (entry) entry.reflection = reflection.value;
  };
  const restoreDraft = document.createElement('button'); restoreDraft.type = 'button'; restoreDraft.textContent = '선택 기록의 원문을 초안으로 가져오기';
  const restoreConfirm = document.createElement('button'); restoreConfirm.type = 'button'; restoreConfirm.textContent = '현재 초안을 선택 기록으로 바꾸기'; restoreConfirm.hidden = true;
  const restoreCancel = document.createElement('button'); restoreCancel.type = 'button'; restoreCancel.textContent = '초안 바꾸기 취소'; restoreCancel.hidden = true;
  const removeRecord = document.createElement('button'); removeRecord.type = 'button'; removeRecord.textContent = '선택 기록 지우기';
  let pendingRestore = null;
  const previousContext = document.createElement('p'), previousOutput = document.createElement('pre');
  const previousSourceTitle = document.createElement('p'); previousSourceTitle.textContent = '당시 요청 원문';
  const previousSource = document.createElement('textarea'); previousSource.readOnly = true; previousSource.rows = 8; previousSource.setAttribute('aria-label', '당시 요청 원문 — 읽기 전용');
  const comparison = document.createElement('section'); comparison.setAttribute('aria-label', '이전 요청과 현재 초안의 원문 차이');
  const comparisonStatus = document.createElement('p');
  const beforeLabel = document.createElement('p'), afterLabel = document.createElement('p');
  const beforeChange = document.createElement('pre'), afterChange = document.createElement('pre');
  beforeChange.style.whiteSpace = afterChange.style.whiteSpace = 'pre-wrap';
  const comparisonHelp = document.createElement('p');
  comparisonHelp.textContent = '가정·목표·검사 단계 중 무엇을 바꿨는지 원문으로 확인하세요. 아래는 처음 달라진 줄부터 마지막 달라진 줄까지이며 중간의 같은 줄도 포함합니다. 줄바꿈 차이도 원문 차이입니다. 수용 결과끼리도 가정과 목표가 다르면 같은 주장을 확인한 것이 아닙니다. 현재 초안은 명시 재검사하고, 두 결과의 목표·가정·실제 단계를 각각 읽으세요.';
  comparison.append(comparisonStatus, beforeLabel, beforeChange, afterLabel, afterChange, comparisonHelp);
  const previousEvidence = document.createElement('section'); previousEvidence.setAttribute('aria-label', '이전 기록의 목표·근거');
  const previousTableRoot=document.createElement('section');previousTableRoot.setAttribute('aria-label','이전 실행의 공급된 보기별 표');
  let previousTableState=null,previousObservationView=null;
  const previousRawDetails = document.createElement('details'), previousRawTitle = document.createElement('summary'), previousRaw = document.createElement('pre');
  previousRawTitle.textContent = '이전 요청과 원래 결과 확인'; previousRawDetails.append(previousRawTitle, previousRaw);
  const forgetResult = document.createElement('button'); forgetResult.type = 'button'; forgetResult.textContent = '이전 요청 기록 지우기';
  previous.append(previousTitle, previousNote, historyLabel, historyStatus, pinBaseline, pairComparison, comparison, previousContext, previousPrediction, previousOutput, previousEvidence, previousTableRoot, previousSourceTitle, previousSource, previousRawDetails, reflectionLabel, reflectionHelp, downloadRecords, restoreDraft, restoreConfirm, restoreCancel, removeRecord, forgetResult);
  container.append(loadSource, applyDraft, sourceStatus, label, modeLabel, draftShelf, predictionLabel, predictionHelp, button, status, submittedPrediction, resultContext, note, output, goalEvidence, locate, details, keepResult, previous); root.append(container);
  const model = createWorkbench({
    runSummaryWithSourceProfile: request => withSourceContext(() => runSummaryWithSourceProfile(request)),
    verifyAssumptionSource: typeof verifyAssumptionSource === 'function'
      ? (source, path) => withSourceContext(() => verifyAssumptionSource(source, path)) : undefined,
    observeAssumptionSource: typeof observeAssumptionSource === 'function'
      ? (source, path) => withSourceContext(() => observeAssumptionSource(source, path)) : undefined,
    observeAlgebraSource: typeof observeAlgebraSource === 'function'
      ? (source, path) => withSourceContext(() => observeAlgebraSource(source, path)) : undefined,
  });
  function showAlgebra(target, result, source, onLocate) {
    try {
      if (result.observation_only !== true || result.unconditional_proof_issued !== false || !Array.isArray(result.definitions)) throw Error('지원하지 않는 대수 관찰 응답입니다.');
      target.innerHTML = conditionalAlgebraHtml(result, source, renderAlgebraObservation);
      renderAlgebraSourceNavigation(target, result, onLocate);
    } catch (error) { target.textContent = '대수 상세 표시 실패 · 원래 응답을 확인하세요: ' + String(error); }
  }
  let context = null, disposed = false, errorLocation = null, sourceContext = null, sourceEpoch = 0, sourceStale = false;
  async function withSourceContext(call) {
    const expected = sourceContext && structuredClone(sourceContext), ticket = sourceEpoch;
    function check() {
      if (!expected || disposed || ticket !== sourceEpoch) return;
      let current = null;
      try { current = readContext(); } catch {}
      if (!sameSourceContext(expected, current)) {
        clearDrafts(); sourceEpoch++; sourceContext = null; sourceStale = true; applyDraft.disabled = true;
        clearPrevious();
        model.setRequest(null);
        sourceStatus.textContent = '본체 원문이 바뀌어 검사 결과를 폐기했습니다. 편집 중 원문은 남겨 두었습니다. 현재 작품 원문을 다시 읽으세요.';
        button.disabled = true;
        throw new Error('원문 문맥이 바뀌었습니다.');
      }
    }
    check();
    try { return await call(); } finally { check(); }
  }
  let draftGeneration = 0;
  const linearRoot = document.createElement('section');
  const linearPanel = linearProblemSupport ? mountLinearProblemPanel(linearRoot, {
    ...linearProblemSupport,
    retainResult: (state, prediction = '') => {
      if (disposed || sourceStale || !context || state.context?.path!==context.sourcePath
          || state.context?.sourceText!==context.sourceText || state.context?.generation!==draftGeneration) throw Error('원문이 바뀌어 비교 기록에 넣지 않았습니다.');
      if (history.length>=8) throw Error('비교 기록이 8건입니다. 불필요한 기록을 지운 뒤 다시 남기세요.');
      const selected=state.targets[state.selectedTargetIndex];
      if (!selected || !state.report) return;
      history.push({state:{status:'returned',mode:'linear',request:structuredClone(context),result:structuredClone(state.report),target:structuredClone(selected)},
        prediction,description:'이전 선택 등식 검사 · '+selected.label+' · '+context.sourcePath,
        display:'이 결과는 당시 선택한 등식에 한정합니다. 가정 집합의 일관성이나 현재 원문의 검사 결과가 아닙니다.'});
      refreshHistory(history.length-1);previous.hidden=false;previous.open=true;
    },
    locateDiagnostic: (diagnosticText, expected, apply) => {
      if (disposed || sourceStale || !context || context.sourcePath!==expected.path || context.sourceText!==expected.sourceText || draftGeneration!==expected.generation) return false;
      const request={sourcePath:expected.path,sourceText:expected.sourceText};
      const selection=typeof diagnosticText==='string' ? proofErrorSelection(diagnosticText,request)
        : diagnosticSelection({source_path:expected.path,source_span:diagnosticText?.span},request);
      if (!selection) return false;
      if (apply) {editor.focus();editor.setSelectionRange(selection.start,selection.end);}
      return true;
    },
    readContext: () => context && !sourceStale && !disposed ? {projectId:'dedicated-editor-local',path:context.sourcePath,generation:draftGeneration,sourceText:context.sourceText} : null,
    inspect: (source,path,options) => withSourceContext(() => linearProblemSupport.inspect(source,path,options)),
  }) : null;
  if (linearPanel) container.append(linearRoot);
  const observationRoot=document.createElement('section');observationRoot.setAttribute('aria-label','이번 실행의 공급된 보기별 표');
  const observationView=typeof mountObservationView==='function'?mountObservationView(observationRoot,{
    readState:()=>{const state=model.snapshot();return !disposed&&!sourceStale&&state.status==='returned'&&state.mode==='execute'?state.result:null;},
    subscribeState:notify=>model.subscribe(()=>notify()),
  }):null;
  if(observationView)container.append(observationRoot);
  if(typeof mountObservationView==='function')previousObservationView=mountObservationView(previousTableRoot,{readState:()=>disposed||sourceStale?null:previousTableState});
  const unsubscribe = model.subscribe(state => {
    if (['empty','edited'].includes(state.status)) {draftGeneration++;linearPanel?.invalidate();}
    if (['empty', 'edited'].includes(state.status)) {
      prediction.value = ''; predictionAtRun = ''; submittedPrediction.textContent = '';
    }
    if (['running', 'returned', 'error'].includes(state.status)) {
      submittedPrediction.textContent = predictionAtRun ? '실행 당시 학습자 예상 · 미검증: ' + predictionAtRun : '실행 전 예상은 기록하지 않았습니다.';
    }
    cancelRestore();
    keepResult.disabled = !['returned', 'error'].includes(state.status) || history.length >= 8;
    status.textContent = ({ empty: '입력 대기', edited: '원문이 변경됐습니다. 다시 실행하세요.', running: '실행 중', returned: '응답 도착 — 원래 결과를 확인하세요.', error: '실행 응답 오류' })[state.status] ?? state.status;
    button.disabled = state.status === 'running' || context === null || sourceStale;
    output.textContent = '';
    goalEvidence.replaceChildren();
    raw.textContent = '';
    resultContext.textContent = '';
    resultContext.hidden = true;
    if (state.request && ['returned', 'error'].includes(state.status)) {
      const action = state.mode === 'algebra' ? '편집 원문의 대수 단계·재사용 관찰' : state.mode === 'evidence' ? '편집 원문의 목표·근거 관찰' : state.mode === 'proof' ? '가정 의존 보임 검사' : '원문 실행';
      resultContext.textContent = '검사한 원문: ' + state.request.sourcePath + ' · 동작: ' + action
        + ' · 명시 요청 당시의 원문에 대한 응답입니다.';
      resultContext.hidden = false;
    }
    errorLocation = null; locate.hidden = true;
    if (state.error !== null) {
      const diagnostic = state.diagnostic;
      output.textContent = diagnostic && typeof diagnostic.message === 'string'
        ? [diagnostic.code, diagnostic.message, diagnostic.source_path].filter(value => typeof value === 'string').join('\n')
        : state.error;
      raw.textContent = JSON.stringify({ mode: state.mode, request: state.request, error: state.error, diagnostic }, null, 2);
      const selection = diagnostic ? diagnosticSelection(diagnostic, state.request)
        : state.mode === 'proof' ? proofErrorSelection(state.error, state.request) : null;
      if (selection) { errorLocation = { ...selection, request: state.request }; locate.hidden = false; }
    }
    if (state.result !== null) {
      const display = state.result?.engine_state?.output_display;
      if (state.mode === 'algebra') {
        output.textContent = '명시 가정에 의존하는 대수 단계 관찰입니다. 선택 등식의 답 계산이나 가정 집합의 일관성 판정이 아닙니다.';
        showAlgebra(goalEvidence, state.result, state.request.sourceText, span => {
          if (disposed || sourceStale || model.snapshot().status !== 'returned' || editor.value !== state.request.sourceText.replace(/\r\n?/g,'\n')) return;
          const selection = diagnosticSelection({source_path:state.result.source_path,source_span:span},state.request);
          if (selection) {editor.focus();editor.setSelectionRange(selection.start,selection.end);}
        });
        const selection = diagnosticSelection(state.result.diagnostic, state.request);
        if (selection) { errorLocation = {...selection, request:state.request}; locate.hidden = false; }
      } else if (state.mode === 'evidence') {
        output.textContent = '편집 원문에 대한 읽기 전용 관찰입니다. 작품 적용·저장 결과가 아닙니다.';
        renderGoalEvidence(goalEvidence, state.result, span => {
          if (disposed || sourceStale || model.snapshot().status !== 'returned'
              || editor.value !== state.request.sourceText.replace(/\r\n?/g, '\n')) return;
          const selection = diagnosticSelection({source_path: state.result.source_path, source_span: span}, state.request);
          if (selection) { editor.focus(); editor.setSelectionRange(selection.start, selection.end); }
        }, state.request.sourceText);
      } else if (state.mode === 'proof') {
        const result = state.result;
        output.textContent = typeof result.message === 'string' ? result.message : '검사 요약이 공급되지 않았습니다.';
        if (Number.isInteger(result.conditional_definition_count)) output.textContent += '\n공급된 조건부 정의 수: ' + result.conditional_definition_count;
        if (result.assumption_dependent === true) output.textContent += '\n명시 가정에 의존하는 검사입니다.';
        if (result.unconditional_proof_issued === false) output.textContent += '\n무가정 증명을 발급하지 않았습니다.';
        output.textContent += '\n목표별 근거·투영 내역은 이 응답에 공급되지 않습니다.';
      } else if (display?.schema === 'ddn.output-display.v1' && Array.isArray(display.texts)
          && display.texts.every(text => typeof text === 'string')) output.textContent = display.texts.join('\n');
      else {
        const model = state.result?.engine_state?.model_execution;
        const fields = model?.result_pack?.ordered_fields;
        if (model?.status === 'completed' && model.result_pack?.schema === 'ddn.model_result_pack.candidate.v1'
            && Array.isArray(fields) && fields.length > 0
            && fields.every(field => typeof field?.name === 'string' && typeof field.value_canon === 'string'
              && typeof field.type === 'string' && typeof field.unit === 'string')) {
          output.textContent = '모형 관계 계산 · 공급된 결과 순서\n'
            + fields.map(field => `${field.name}: ${field.value_canon} · 타입 ${field.type} · 단위 ${field.unit || '명시 없음'}`).join('\n')
            + '\n시간 변화·일반 방정식·보임의 수용을 뜻하지 않습니다. 원문과 원래 응답은 아래에서 확인하세요.';
        } else output.textContent = '표시 가능한 출력이 공급되지 않았습니다. 원래 결과를 확인하세요.';
      }
      raw.textContent = JSON.stringify({ mode: state.mode, request: state.request, result: state.result }, null, 2);
    }
    showComparison(state);
  });
  function showComparison(state = model.snapshot()) {
    const entry = history[Number(historySelect.value)];
    comparison.hidden = !entry || !context || disposed || sourceStale;
    comparisonStatus.textContent = beforeLabel.textContent = afterLabel.textContent = '';
    beforeChange.textContent = afterChange.textContent = '';
    if (comparison.hidden) return;
    const difference = compareSourceLines(entry.state.request.sourceText, context.sourceText);
    const resultState = entry.state.mode === 'linear'
      ? '선택 등식 검사의 현재 대상·결과는 별도 검사 영역에서 확인하세요. 원문이 같아도 선택 대상이 같다는 뜻은 아닙니다.'
      : ['returned','error'].includes(state.status)
      ? '현재 원문의 응답은 위에 있습니다. 이전 기록의 응답과 구별하세요.'
      : state.status === 'running' ? '현재 원문은 응답 대기 중입니다.' : '현재 원문은 아직 재검사하지 않았습니다.';
    comparisonStatus.textContent = (difference ? '요청 원문과 현재 초안이 다릅니다. ' : '요청 원문과 현재 초안이 같습니다. ') + resultState;
    beforeLabel.hidden = beforeChange.hidden = afterLabel.hidden = afterChange.hidden = !difference;
    if (!difference) return;
    const range = end => end < difference.startLine ? `${difference.startLine}줄 앞 · 해당 줄 없음` : `${difference.startLine}–${end}줄`;
    beforeLabel.textContent = '이전 요청 · ' + range(difference.beforeEndLine);
    afterLabel.textContent = '현재 초안 · ' + range(difference.afterEndLine);
    beforeChange.textContent = difference.before || '(빈 내용)';
    afterChange.textContent = difference.after || '(빈 내용)';
  }
  function clearPrevious() {
    comparisonBaseline = null; showRecordedComparison();
    reflection.value = '';
    previousTableState=null;previousObservationView?.refresh();
    history.length = 0; historySelect.replaceChildren(); cancelRestore(); historyStatus.textContent = '';
    previous.hidden = true; previous.open = false;
    previousPrediction.textContent = previousContext.textContent = previousOutput.textContent = previousRaw.textContent = '';
    previousSource.value = ''; previousEvidence.replaceChildren();
    previousRawDetails.open = false;
    showComparison();
    keepResult.disabled = !['returned', 'error'].includes(model.snapshot().status);
  }
  const keepCurrentResult = () => {
    const state = model.snapshot();
    if (disposed || sourceStale || !['returned', 'error'].includes(state.status) || !state.request || history.length >= 8) return;
    history.push({ state, prediction: predictionAtRun, description: resultContext.textContent, display: output.textContent });
    refreshHistory(history.length - 1);
    previous.hidden = false; previous.open = true;
  };
  function cancelRestore() {
    pendingRestore = null; restoreConfirm.hidden = restoreCancel.hidden = true;
  }
  function showHistory() {
    showRecordedComparison();
    cancelRestore(); historyStatus.textContent = history.length === 8 ? '기록이 8건입니다. 새로 남기려면 불필요한 기록을 먼저 지우세요.' : '';
    const entry = history[Number(historySelect.value)]; if (!entry) return;
    reflection.value = entry.reflection ?? '';
    previousTableState=entry.state.status==='returned'&&entry.state.mode==='execute'?entry.state.result:null;
    previousObservationView?.refresh();
    previousPrediction.textContent = entry.prediction ? '당시 학습자 예상 · 미검증: ' + entry.prediction : '당시 예상 없음';
    previousContext.textContent = entry.description;
    previousOutput.textContent = entry.display;
    previousSource.value = entry.state.request.sourceText;
    previousEvidence.replaceChildren();
    if (['algebra','linear'].includes(entry.state.mode) && entry.state.result) showAlgebra(previousEvidence, entry.state.result, entry.state.request.sourceText, span => {
      if (disposed || sourceStale || history[Number(historySelect.value)] !== entry) return;
      const selection=diagnosticSelection({source_path:entry.state.result.source_path,source_span:span},entry.state.request);
      if (selection) {previousSource.focus();previousSource.setSelectionRange(selection.start,selection.end);historyStatus.textContent='이전 요청 원문의 위치입니다. 현재 초안은 바뀌지 않습니다.';}
    });
    if (entry.state.mode === 'evidence' && entry.state.result) {
      renderGoalEvidence(previousEvidence, entry.state.result, span => {
        if (disposed || sourceStale || history[Number(historySelect.value)] !== entry) return;
        const selection = diagnosticSelection({source_path: entry.state.result.source_path, source_span: span}, entry.state.request);
        if (selection) {
          previousSource.focus(); previousSource.setSelectionRange(selection.start, selection.end);
          historyStatus.textContent = '이전 요청 원문에서 위치를 표시했습니다. 현재 초안과 검사 결과는 바뀌지 않습니다.';
        }
      }, entry.state.request.sourceText);
    }
    previousRaw.textContent = JSON.stringify(entry.state, null, 2); previousRawDetails.open = false;
    showComparison();
  }
  function refreshHistory(selected = 0) {
    historySelect.replaceChildren();
    history.forEach((entry, index) => {
      const option = document.createElement('option'); option.value = String(index);
      option.textContent = `${index + 1}. ${entry.state.request.sourcePath} · ${entry.state.mode === 'execute' ? '원문 실행' : entry.state.mode === 'proof' ? '보임 검사' : entry.state.mode === 'linear' ? '선택 등식 검사 · '+entry.state.target.label : entry.state.mode === 'algebra' ? '대수 단계·재사용 관찰' : '목표·근거 관찰'} · ${entry.state.status === 'error' ? '호출 오류' : '응답 기록'}`;
      historySelect.append(option);
    });
    historySelect.value = String(selected); showHistory();
    keepResult.disabled = !['returned', 'error'].includes(model.snapshot().status) || history.length >= 8;
  }
  historySelect.onchange = showHistory;
  removeRecord.onclick = () => {
    if (disposed) return;
    if (comparisonBaseline === history[Number(historySelect.value)]) comparisonBaseline = null;
    history.splice(Number(historySelect.value), 1);
    if (history.length) refreshHistory(); else { clearPrevious(); keepResult.disabled = !['returned', 'error'].includes(model.snapshot().status); }
  };
  restoreDraft.onclick = () => {
    if (disposed || sourceStale || !context) return;
    pendingRestore = history[Number(historySelect.value)]; if (!pendingRestore) return;
    historyStatus.textContent = '현재 초안을 바꿉니다. 필요한 현재 원문은 먼저 보존하세요. 과거 결과는 복원하지 않으며 실행·적용·저장도 하지 않습니다.';
    restoreConfirm.hidden = restoreCancel.hidden = false;
  };
  restoreCancel.onclick = () => { cancelRestore(); historyStatus.textContent = '초안을 바꾸지 않았습니다.'; };
  restoreConfirm.onclick = () => {
    const entry = pendingRestore; cancelRestore();
    if (disposed || sourceStale || !context || !entry) return;
    if (sourceContext) {
      let current; try { current = readContext(); } catch {}
      if (!sameSourceContext(sourceContext, current)) {
        clearDrafts(); sourceEpoch++; sourceContext = null; sourceStale = true; applyDraft.disabled = true;
        clearPrevious(); model.setRequest(null); button.disabled = true;
        sourceStatus.textContent = '작품 원문이 바뀌었습니다. 현재 원문을 다시 읽으세요. 초안은 바꾸지 않았습니다.'; return;
      }
    }
    if (entry.state.request.sourcePath !== context.sourcePath || entry.state.request.sourceProfileIdentity !== context.sourceProfileIdentity) {
      historyStatus.textContent = '현재 원문 문맥과 다른 기록은 가져올 수 없습니다.'; return;
    }
    context = structuredClone(entry.state.request); editor.value = context.sourceText; mode.value = entry.state.mode==='linear' ? 'execute' : entry.state.mode;
    changeMode(); editor.focus(); historyStatus.textContent = entry.state.mode==='linear' ? '원문만 가져왔습니다. B 대상과 결과는 복원하지 않습니다. 가정 대상을 다시 읽고 명시 선택·검사하세요.' : '원문을 초안으로 가져왔습니다. 과거 결과는 현재 결과가 아닙니다. 명시 실행·검사를 하세요.';
  };
  keepResult.addEventListener('click', keepCurrentResult);
  forgetResult.addEventListener('click', clearPrevious);
  function setRequest(next, {mode: nextMode} = {}) {
    if (disposed) return;
    if (nextMode !== undefined && !Array.from(mode.options).some(option => option.value === nextMode)) {
      throw new Error('이 작업대에 공급되지 않은 동작입니다. 원문을 바꾸지 않았습니다.');
    }
    if (nextMode !== undefined) { mode.value = nextMode; updateModePresentation(); }
    clearDrafts();
    clearPrevious();
    sourceEpoch++; sourceContext = null; sourceStale = false; applyDraft.disabled = true; sourceStatus.textContent = '';
    context = null;
    model.setRequest(null);
    context = next === null ? null : structuredClone(next);
    editor.value = context?.sourceText ?? '';
    model.setRequest(context);
  }
  const loadCurrentSource = () => {
    if (disposed) return;
    try {
      const next = readContext();
      if (!next || !next.projectId || !next.path || !Number.isSafeInteger(next.generation)
          || typeof next.sourceText !== 'string' || !initialRequest?.sourceProfileIdentity) throw new Error('현재 작품 원문과 명시 프로필이 필요합니다.');
      setRequest({ ...initialRequest, sourceText: next.sourceText, sourcePath: next.path });
      sourceContext = structuredClone(next); applyDraft.disabled = false;
      sourceStatus.textContent = next.path + ' — 현재 작품 원문을 읽었습니다. 적용은 저장이 아닙니다.';
    } catch (error) {
      clearDrafts(); sourceEpoch++; sourceContext = null; sourceStale = true; applyDraft.disabled = true;
      clearPrevious(); model.setRequest(null); button.disabled = true;
      sourceStatus.textContent = String(error) + ' 편집 중 원문은 남겨 두었습니다. 현재 작품 원문을 다시 읽으세요.';
    }
  };
  const applyCurrentDraft = () => {
    if (disposed || !sourceContext || !context) return;
    const expected = structuredClone(sourceContext), ticket = sourceEpoch, draft = context.sourceText;
    try {
      const current = readContext();
      if (!sameSourceContext(expected, current)) throw new Error('작품·파일·원문 판본이 바뀌었습니다. 현재 원문을 다시 읽으세요.');
      // 기존 본체의 동기 compare-and-apply callback을 사용한다. 저장은 요청하지 않는다.
      const applied = applySource(expected, draft);
      if (disposed || ticket !== sourceEpoch) return;
      if (!applied || applied.projectId !== expected.projectId || applied.path !== expected.path
          || applied.sourceText !== draft || !Number.isSafeInteger(applied.generation)
          || !sameSourceContext(applied, readContext())) throw new Error('적용된 현재 원문을 확인하지 못했습니다. 다시 읽으세요.');
      setRequest({ ...context, sourceText: applied.sourceText, sourcePath: applied.path });
      sourceContext = structuredClone(applied); applyDraft.disabled = false;
      sourceStatus.textContent = '편집기에 적용했습니다. 저장·검사는 별도로 수행하세요.';
    } catch (error) {
      if (!disposed && ticket === sourceEpoch) {
        clearDrafts(); sourceEpoch++; sourceContext = null; sourceStale = true; applyDraft.disabled = true;
        clearPrevious(); model.setRequest(null); button.disabled = true;
        sourceStatus.textContent = String(error) + ' 편집 중 원문은 남겨 두었습니다. 현재 작품 원문을 다시 읽은 뒤 실행하세요.';
      }
    }
  };
  loadSource.addEventListener('click', loadCurrentSource); applyDraft.addEventListener('click', applyCurrentDraft);
  const edit = () => { cancelRestore(); if (context) { context = { ...context, sourceText: editor.value }; model.setRequest(sourceStale ? null : context); } };
  const run = () => { if (!sourceStale) { predictionAtRun = prediction.value; void model.run(mode.value); } };
  const locateError = () => {
    if (disposed || !errorLocation || context?.sourcePath !== errorLocation.request.sourcePath
        || editor.value !== errorLocation.request.sourceText.replace(/\r\n?/g, '\n')) return;
    editor.focus(); editor.setSelectionRange(errorLocation.start, errorLocation.end);
  };
  locate.addEventListener('click', locateError);
  const changeMode = () => { model.setRequest(context); updateModePresentation(); };
  function updateModePresentation() {
    button.textContent = mode.value === 'algebra' ? '명시 대수 단계 관찰' : mode.value === 'evidence' ? '명시 목표·근거 관찰' : mode.value === 'proof' ? '명시 보임 검사' : '명시 실행';
    note.textContent = mode.value === 'algebra' ? '기존 대수 검사기가 공급한 가정·주장·단계·호출 인자를 읽습니다. 가정 자체의 참이나 무가정 증명을 뜻하지 않습니다.' : mode.value === 'evidence' ? '편집 원문의 목표·선언 가정·근거 참조를 관찰합니다. 참조 정보는 수용 판정이나 가정 자체의 참을 뜻하지 않습니다.'
      : mode.value === 'proof' ? '전체 원문을 기존 checker에 전달합니다. 조건부 정의 수용은 가정 자체의 참이나 무가정 증명이 아닙니다.' : '출력은 원문의 보여주기 순서입니다. 동치 비교의 참·거짓은 사용자 보임 발급이나 방정식의 해 판정이 아닙니다.';
  }
  mode.addEventListener('change', changeMode);
  editor.addEventListener('input', edit); button.addEventListener('click', run);
  setRequest(initialRequest ?? null);
  return {
    setRequest,
    readDraft() { return !disposed && !sourceStale && context ? { request: structuredClone(context), generation: draftGeneration } : null; },
    invalidate() { setRequest(null); },
    dispose() { disposed = true; clearDrafts(); sourceEpoch++; sourceContext = null; context = null; errorLocation = null; unsubscribe(); linearPanel?.dispose(); observationView?.dispose(); previousObservationView?.dispose(); previousTableState=null; model.dispose(); editor.removeEventListener('input', edit); button.removeEventListener('click', run); locate.removeEventListener('click', locateError); loadSource.removeEventListener('click', loadCurrentSource); applyDraft.removeEventListener('click', applyCurrentDraft); mode.removeEventListener('change', changeMode); keepResult.removeEventListener('click', keepCurrentResult); forgetResult.removeEventListener('click', clearPrevious); clearPrevious(); container.remove(); },
  };
}
function sameSourceContext(left, right) {
  return Boolean(left && right && ['projectId', 'path', 'generation', 'sourceText'].every(key => left[key] === right[key]));
}
