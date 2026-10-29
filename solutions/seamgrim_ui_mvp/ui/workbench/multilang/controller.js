// 다국어 소비자. parser/registry/type/identity/저장 wire를 정의하지 않는다.
// runtimeBridge는 기존 RuntimeBridge 메서드를 그대로 사용한다.
// persistence는 dev가 기존 원문 저장/열기에 결속해야 하는 소비자 콜백이다.
const SCHEMA = 'ddn.source_analysis_snapshot.candidate.v1';
const clone = value => structuredClone(value);
const validDocument = doc => doc && typeof doc.sourceText === 'string'
  && typeof doc.path === 'string' && doc.path.length > 0
  && (doc.sourceProfileIdentity === null || typeof doc.sourceProfileIdentity === 'string');

export function createMultilangController({runtimeBridge, persistence = null} = {}) {
  const slots = {ko: slot(), ja: slot()};
  const listeners = new Set();
  let locale = 'ko';
  let disposed = false;
  function slot() { return {generation:0, documentEpoch:0, status:'idle', document:null,
    analysis:null, result:null, savedReceipt:null, error:null, busy:false}; }
  function current(language) {
    if (disposed) throw new Error('닫힌 다국어 패널입니다.');
    if (!Object.hasOwn(slots, language)) throw new TypeError('원문 말씨는 ko/ja만 지원합니다.');
    return slots[language];
  }
  function snapshot() {
    const view = clone({locale, slots, comparison:'supplier_required'});
    for (const item of Object.values(view.slots)) {
      // receipt 호환 읽기는 재열기 위치만 뜻한다. 현재 원문/분석/저장 성공을 나타내지 않는다.
      item.receipt = item.savedReceipt?.receipt ?? null;
      const saved = item.savedReceipt?.document;
      item.savedContentMatches = Boolean(saved && item.document
        && saved.sourceText === item.document.sourceText && saved.path === item.document.path
        && saved.sourceProfileIdentity === item.document.sourceProfileIdentity);
    }
    return view;
  }
  function notify() { for (const listener of listeners) listener(snapshot()); }
  function setDocument(language, document, {recheckOnly = false, replaceDocument = false} = {}) {
    const item = current(language);
    if (!validDocument(document)) throw new TypeError('원문·경로·명시 프로필 자료가 필요합니다.');
    if (typeof recheckOnly !== 'boolean') throw new TypeError('재검사 선택은 참/거짓이어야 합니다.');
    if (typeof replaceDocument !== 'boolean' || (replaceDocument && recheckOnly)) {
      throw new TypeError('새 문서 교체는 참/거짓이며 재검사와 함께 지정할 수 없습니다.');
    }
    if (recheckOnly && (!item.document || document.sourceText !== item.document.sourceText
        || document.path !== item.document.path
        || document.sourceProfileIdentity !== item.document.sourceProfileIdentity)) {
      throw new Error('공급 변경 재검사는 같은 원문·경로·프로필에서만 가능합니다.');
    }
    // 진행 중 save/open 콜백의 실제 부작용은 취소할 수 없다. 중복 요청은 계속 잠근다.
    // 외부 profile/DDL/의존 변경은 host가 명시 통지한다. receipt는 재열기 위치일 뿐 분석 유효성 아님.
    // 같은 칸의 편집은 저장본을 보존한다. host의 작품 전환은 같은 경로라도 replaceDocument:true다.
    const replacement = replaceDocument || !item.document || document.path !== item.document.path
      || document.sourceProfileIdentity !== item.document.sourceProfileIdentity;
    Object.assign(item, {generation:item.generation + 1, document:clone(document),
      documentEpoch:item.documentEpoch + (replacement ? 1 : 0),
      analysis:null, result:null, savedReceipt:replacement ? null : item.savedReceipt,
      error:null, status:recheckOnly ? 'recheck_required' : 'edited'});
    notify();
  }
  async function perform(language, pending, operation, {allowEmpty = false} = {}) {
    const item = current(language);
    if (item.busy) return {status:'busy'};
    if (!item.document || (!allowEmpty && item.document.sourceText.length === 0)) return {status:'empty'};
    const generation = item.generation;
    const documentEpoch = item.documentEpoch;
    const document = clone(item.document);
    item.busy = true; item.status = pending; item.error = null; notify();
    const fresh = () => !disposed && item.generation === generation;
    try {
      const answer = await operation(document, generation, item);
      // 저장만 과거 세대의 성공을 보관한다. 현재 원문·분석은 건드리지 않고 문서 교체면 버린다.
      const retained = !disposed && item.documentEpoch === documentEpoch && typeof answer.retainSaved === 'function';
      if (retained) answer.retainSaved(item);
      if (!fresh()) return {status:retained ? 'saved_previous' : 'stale'};
      if (answer.commit) answer.commit(item);
      item.status = answer.status;
      return {status:answer.status};
    } catch (error) {
      if (!fresh()) return {status:'stale'};
      item.status = ['unsupported','supply_missing'].includes(error?.presentationStatus) ? error.presentationStatus : 'failed';
      item.error = String(error?.message ?? error);
      return {status:item.status};
    } finally {
      item.busy = false;
      if (!disposed) notify();
    }
  }
  function analyze(language) {
    const item = current(language);
    if (!item.busy) { item.analysis = null; item.result = null; }
    return perform(language, 'checking', async (document, generation) => {
      if (!document.sourceProfileIdentity || typeof runtimeBridge?.sourceAnalysis !== 'function') {
        return {status:'supply_missing'};
      }
      const answer = clone(await runtimeBridge.sourceAnalysis({source_text:document.sourceText,
        path:document.path, source_profile_identity:document.sourceProfileIdentity,
        analysis_generation:generation}));
      // 분석 결과의 identity는 생산자 값만 보존한다. 한·일 identity 일치 요구 없음.
      if (answer.schema !== SCHEMA || answer.analysis_generation !== generation
          || answer.source_profile_identity !== document.sourceProfileIdentity
          || typeof answer.source_revision_identity !== 'string' || !answer.source_revision_identity
          || !['ACCEPTED','REJECTED'].includes(answer.outcome) || !Array.isArray(answer.diagnostics)
          || (answer.outcome === 'ACCEPTED' && answer.diagnostics.some(d => d.severity === 'error'))) {
        throw new Error('공통 분석 응답의 판본·세대·프로필 또는 상태가 맞지 않습니다.');
      }
      return {status:answer.outcome === 'ACCEPTED' ? 'accepted' : 'rejected',
        commit:item => { item.analysis = answer; item.result = null; }};
    });
  }
  function run(language) {
    const item = current(language);
    if (item.analysis?.outcome !== 'ACCEPTED') return Promise.resolve({status:'analysis_required'});
    return perform(language, 'running', async document => {
      if (typeof runtimeBridge?.runSummaryWithSourceProfile !== 'function') return {status:'supply_missing'};
      const result = clone(await runtimeBridge.runSummaryWithSourceProfile({sourceText:document.sourceText,
        sourcePath:document.path, requestedTicks:1, sourceProfileIdentity:document.sourceProfileIdentity}));
      // completed는 응답 수신 표시다. runtime 결과나 typed 실패값을 성공으로 재분류하지 않는다.
      return {status:'completed', commit:target => { target.result = result; }};
    });
  }
  function save(language) {
    return perform(language, 'saving', async (document, generation, item) => {
      const documentEpoch = item.documentEpoch;
      if (typeof persistence?.saveDocument !== 'function') return {status:'supply_missing'};
      const receipt = clone(await persistence.saveDocument(clone(document)));
      if (receipt == null) throw new Error('원문 저장 응답이 없습니다.');
      return {status:'saved', retainSaved:target => {
        target.savedReceipt = {receipt, generation, documentEpoch, document:clone(document)};
      }};
    });
  }
  async function reopen(language) {
    const item = current(language);
    if (item.savedReceipt === null) return {status:'save_required'};
    const receipt = clone(item.savedReceipt.receipt);
    let reopened = null;
    let openedGeneration = null;
    const answer = await perform(language, 'reopening', async () => {
      if (typeof persistence?.openDocument !== 'function') return {status:'supply_missing'};
      reopened = clone(await persistence.openDocument(receipt));
      if (!validDocument(reopened)) throw new Error('저장본 원문·경로·프로필 공급이 불완전합니다.');
      return {status:'edited', commit:target => {
        target.document = reopened; target.generation += 1;
        openedGeneration = target.generation;
        target.analysis = null; target.result = null; target.error = null;
      }};
    }, {allowEmpty:true});
    // 재열기만으로 실행하지 않는다. 이전 분석은 폐기하고 공통 생산자에게 새로 검사한다.
    if (answer.status === 'edited' && !disposed && item.generation === openedGeneration) return analyze(language);
    return answer;
  }
  return Object.freeze({snapshot, setDocument, analyze, run, save, reopen,
    setLocale(value) {
      if (!['ko','ja'].includes(value)) throw new TypeError('표시 언어는 ko/ja만 지원합니다.');
      locale = value; notify();
    },
    subscribe(listener) { listeners.add(listener); listener(snapshot()); return () => listeners.delete(listener); },
    dispose() { disposed = true; listeners.clear(); },
  });
}
