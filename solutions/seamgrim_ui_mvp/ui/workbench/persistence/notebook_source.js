import { createAuthoredSourceStorage } from './authored_source_save.js';
import { nativeSourceAvailable, sourceStorageErrorMessage } from '../../runtime/native_source_transport.js';

// Source-only notebook session. Inputs, units, assumptions and dependency pins
// must already be materialized in the common producer's complete source
// project. There is no extra notebook wire, metadata side-store or proof store.
export function createNotebookSourceSession({runtimeBridge, sourceStorage,
  nativeAvailable = nativeSourceAvailable} = {}) {
  const store = sourceStorage ?? createAuthoredSourceStorage(runtimeBridge);
  let document = null, epoch = 0, disposed = false, busy = false;
  let notebookContext = null;
  const pastReports = [];
  let savedText = '', locator = '', report = null, phase = 'empty', error = '', viewError = '';
  const listeners = new Set();

  function project(raw) {
    if (typeof raw !== 'string' || !raw) throw new TypeError('노트의 실제 작품 원문이 필요합니다.');
    const parsed = JSON.parse(raw);
    const id = parsed?.authored_identity?.logical_dependency_id;
    if (parsed?.schema !== 'ddn.authored_project_artifact.v1' || typeof id !== 'string' || !id
      || !Array.isArray(parsed?.source_snapshot?.files) || !parsed.source_snapshot.files.length) {
      throw new TypeError('공통 제품이 만든 전체 작품 원문을 사용하세요.');
    }
    // JSON is inspected only for UI ownership. Never reserialize the artifact,
    // repair its digest, parse DDN, infer units, or mint a project identity here.
    return {text: raw, id, profile: parsed.source_profile, source: parsed.source_snapshot};
  }
  function assertLive() {
    if (disposed) throw new Error('닫힌 노트입니다.');
  }
  function snapshot() {
    return {projectId: document?.id ?? '', sourceProjectText: document?.text ?? '',
      epoch, phase, busy, error, viewError, needsRecheck: report === null,
      report: report === null ? null : structuredClone(report),
      notebookContext: notebookContext === null ? null : structuredClone(notebookContext),
      contextNeedsBinding: notebookContext === null,
      pastReports: structuredClone(pastReports), historyIsPersisted: false,
      hasSavedSource: Boolean(locator), currentSourceSaved: Boolean(document && savedText === document.text)};
  }
  function notify() {
    // A broken view must not interrupt source custody or turn an acknowledged
    // Native save into a failed transaction. Surface it separately from I/O.
    let failed = '';
    for (const fn of listeners) {
      try { fn(snapshot()); } catch (cause) { failed = String(cause?.message ?? cause); }
    }
    viewError = failed;
  }
  function archiveReport(reason) {
    if (report === null) return;
    pastReports.push({label: '과거 결과', reason, usableAsCurrent: false,
      sourceProjectText: document.text, sourceProfile: document.profile,
      context: notebookContext === null ? null : structuredClone(notebookContext),
      report: structuredClone(report)});
  }
  function matchedContext(candidate, current = document) {
    if (!current || !candidate || typeof candidate !== 'object') throw new TypeError('현재 원문에 결속할 노트 항목이 필요합니다.');
    const files = new Map(current.source.files.map(file => [file.path, file.text]));
    const checked = {};
    for (const role of ['inputs', 'units', 'assumptions', 'dependencies']) {
      if (!Array.isArray(candidate[role]) || candidate[role].length === 0) throw new TypeError(`노트 ${role}의 실제 원문 위치가 필요합니다.`);
      checked[role] = candidate[role].map(ref => {
        if (!ref || typeof ref.path !== 'string' || typeof ref.text !== 'string'
          || !Number.isSafeInteger(ref.startByte) || !Number.isSafeInteger(ref.endByte)) throw new TypeError('원문 경로와 UTF-8 바이트 범위가 필요합니다.');
        const source = files.get(ref.path);
        if (typeof source !== 'string') throw new Error('노트 항목의 원문 파일이 현재 작품에 없습니다.');
        const bytes = new TextEncoder().encode(source);
        if (ref.startByte < 0 || ref.endByte <= ref.startByte || ref.endByte > bytes.length) throw new Error('노트 항목의 원문 범위가 올바르지 않습니다.');
        const text = new TextDecoder('utf-8', {fatal: true}).decode(bytes.slice(ref.startByte, ref.endByte));
        if (text !== ref.text) throw new Error('노트 항목의 값·단위·가정·의존 표기가 현재 원문과 다릅니다.');
        return {path: ref.path, startByte: ref.startByte, endByte: ref.endByte, text};
      });
    }
    // Lock bytes, not a display name/version, bind external dependencies. With
    // no lock, bind the explicit root/entry source instead. Rust still checks
    // the graph, package integrity and actual type/unit semantics.
    const dependencyPath = files.has('ddn.lock') ? 'ddn.lock' : current.source.root_path ?? current.source.entry_path;
    const full = new TextEncoder().encode(files.get(dependencyPath) ?? '').length;
    if (!checked.dependencies.some(ref => ref.path === dependencyPath && ref.startByte === 0 && ref.endByte === full))
      throw new Error('의존 판본은 실제 전체 lock 또는 뿌리 원문에 결속해야 합니다.');
    return {...checked, validation: 'source-text-matched-only', mathematicalUnitCheckIssued: false};
  }
  function assertIdle() {
    assertLive();
    if (busy) throw new Error('노트의 저장·재열기·검사가 진행 중입니다.');
  }
  function capture() {
    if (!document) throw new Error('먼저 노트 작품을 여세요.');
    const ticket = epoch, current = document;
    return {current, unchanged() {
      assertLive();
      if (epoch !== ticket || document !== current) throw new Error('노트 편집판이 바뀌었습니다. 이전 응답을 현재 결과로 게시하지 않습니다.');
    }};
  }
  async function requireNative(unchanged) {
    if (!await nativeAvailable()) throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED: 노트 원문은 Native 작업대에서 저장·재열기하세요.');
    unchanged();
  }
  async function saveSource() {
      assertIdle(); const {current, unchanged} = capture();
      busy = true; phase = 'saving'; error = ''; notify();
      try {
        await requireNative(unchanged);
        const saved = await store.save(current.text, unchanged,
          {previousArtifactText: savedText, retainedLocator: locator});
        unchanged();
        if (typeof saved?.locator !== 'string' || !saved.locator) throw new Error('E_SOURCE_SAVE_READBACK: 저장 확인 위치가 없습니다.');
        const owner = JSON.parse(saved.locator);
        if (owner.native_source_project_id !== current.id) throw new Error('E_NATIVE_SOURCE_READBACK_PROJECT');
        // Publish success only after the existing protected save/readback and
        // current-document guard. On error the previous good locator stays.
        locator = saved.locator; savedText = current.text; phase = 'saved';
        return locator;
      } catch (cause) {
        phase = 'save-failed'; error = sourceStorageErrorMessage(cause); throw cause;
      } finally { busy = false; notify(); }
    }
  return {
    snapshot,
    subscribe(fn) { assertLive(); listeners.add(fn); notify(); return () => listeners.delete(fn); },
    setSourceProject(artifactText) {
      assertLive();
      const next = project(artifactText);
      if (document && next.id !== document.id) throw new Error('다른 노트 ID는 새 세션으로 여세요. 기존 저장본을 교체하지 않습니다.');
      if (document?.text === next.text) return snapshot();
      archiveReport('source-or-dependency-changed');
      document = next; epoch++; report = null; notebookContext = null; phase = 'edited'; error = ''; notify();
      return snapshot();
    },
    save: saveSource,
    bindContext(candidate) {
      assertIdle(); notebookContext = matchedContext(candidate); notify(); return snapshot();
    },
    async saveNotebook() {
      assertIdle();
      if (notebookContext === null) throw new Error('입력·단위·가정·의존 원문을 먼저 결속하세요. 원문 전용 저장을 완성 노트 저장으로 표시하지 않습니다.');
      matchedContext(notebookContext);
      return saveSource();
    },
    async open(savedLocator) {
      assertIdle();
      if (typeof savedLocator !== 'string') throw new TypeError('기존 Native 저장 위치가 필요합니다.');
      const owner = JSON.parse(savedLocator), id = owner?.native_source_project_id;
      if (typeof id !== 'string' || !id) throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED: Native 원문 저장 위치가 아닙니다.');
      if (document && document.id !== id) throw new Error('다른 노트 ID는 새 세션으로 여세요.');
      const ticket = epoch;
      const unchanged = () => { assertLive(); if (epoch !== ticket) throw new Error('노트 편집판이 바뀌어 재열기 응답을 적용하지 않습니다.'); };
      busy = true; phase = 'opening'; error = ''; notify();
      try {
        await requireNative(unchanged);
        const raw = await store.reopen(id, unchanged, owner); unchanged();
        const reopened = project(raw);
        if (reopened.id !== id) throw new Error('E_NATIVE_SOURCE_READBACK_PROJECT');
        archiveReport('reopened-needs-recheck');
        document = reopened; epoch++; savedText = raw; notebookContext = null;
        // The common reader may update the actual Native authoring selection.
        // Only its existing opaque locator is retained; raw source is untouched.
        locator = JSON.stringify(owner); report = null; phase = 'needs-recheck';
        return snapshot();
      } catch (cause) {
        phase = 'open-failed'; error = sourceStorageErrorMessage(cause); throw cause;
      } finally { busy = false; notify(); }
    },
    async recheck(inspectSourceProject) {
      assertIdle();
      if (typeof inspectSourceProject !== 'function') throw new TypeError('공통 원문 재검사 공급자가 필요합니다.');
      const {current, unchanged} = capture();
      busy = true; report = null; phase = 'checking'; error = ''; notify();
      try {
        const checked = await inspectSourceProject(current.text, notebookContext === null ? null : structuredClone(notebookContext)); unchanged();
        if (!checked || typeof checked !== 'object') throw new Error('공통 검사 결과를 확인하지 못했습니다.');
        // View-only current-source report; never persisted as proof authority.
        if (checked.notebookContext !== undefined) notebookContext = matchedContext(checked.notebookContext, current);
        report = structuredClone(checked); phase = 'inspected'; return snapshot();
      } catch (cause) {
        phase = 'check-failed'; error = sourceStorageErrorMessage(cause); throw cause;
      } finally { busy = false; notify(); }
    },
    dispose() { disposed = true; epoch++; report = null; listeners.clear(); },
  };
}

// M5 A: two independent source reproductions, not stored observations. The
// caller obtains distinct projects through the common create/clone producer.
// Locations are existing opaque Native receipts, not a new persistent wire.
export function createNotebookReproductionPair({runtimeBridge, nativeAvailable,
  sourceStorageFactory} = {}) {
  const make = slot => createNotebookSourceSession({runtimeBridge, nativeAvailable,
    sourceStorage: sourceStorageFactory?.(slot)});
  let sessions = {A: make('A'), B: make('B')};
  let locations = {A: '', B: ''}, published = {A: false, B: false};
  let epoch = 0, busy = false, disposed = false, phase = 'empty', error = '';
  let confirmedThisAttempt = {A: false, B: false};
  let pairSaveConfirmed = false;
  const slots = ['A', 'B'];
  function live() { if (disposed) throw new Error('닫힌 두 실험입니다.'); }
  function slotName(slot) { live(); if (!slots.includes(slot)) throw new TypeError('실험 A 또는 B가 필요합니다.'); }
  function idle() { live(); if (busy) throw new Error('두 실험의 저장·재열기·계산이 진행 중입니다.'); }
  function distinct(a, b) {
    if (!a || !b || a === b) throw new Error('A와 B는 공통 생성·복제로 만든 서로 다른 작품 정체여야 합니다. 같은 저장본을 덮지 않습니다.');
  }
  function snapshot() {
    const state = {};
    for (const slot of slots) {
      state[slot] = sessions[slot].snapshot();
      if (!published[slot]) { state[slot].report = null; state[slot].needsRecheck = true; }
      state[slot].resultStatus = published[slot] ? 'PRODUCER_RETURNED' : 'NOT_PRODUCED';
      state[slot].resultOrigin = published[slot] ? 'explicit-new-request' : null;
    }
    return {phase, busy, error, epoch, experiments: state,
      locations: {...locations}, confirmedThisAttempt: {...confirmedThisAttempt},
      bothSourcesSaved: pairSaveConfirmed && state.A.currentSourceSaved && state.B.currentSourceSaved,
      pastResultsPersisted: false, mathematicalComparison: 'COMMON_PRODUCER_REQUIRED'};
  }
  return {
    snapshot,
    setSourceProject(slot, raw) {
      slotName(slot);
      const id = JSON.parse(raw)?.authored_identity?.logical_dependency_id;
      const other = sessions[slot === 'A' ? 'B' : 'A'].snapshot().projectId;
      if (other && id === other) distinct(id, other);
      const previous = sessions[slot].snapshot().sourceProjectText;
      sessions[slot].setSourceProject(raw);
      if (previous !== raw) { epoch++; published[slot] = false; pairSaveConfirmed = false; phase = 'edited'; }
      return snapshot();
    },
    bindContext(slot, context) { slotName(slot); idle(); sessions[slot].bindContext(context); return snapshot(); },
    async save() {
      idle();
      const a = sessions.A.snapshot(), b = sessions.B.snapshot(); distinct(a.projectId, b.projectId);
      if (a.contextNeedsBinding || b.contextNeedsBinding) throw new Error('A와 B의 입력·단위·가정·의존 원문을 각각 결속하세요.');
      const ticket = epoch; busy = true; phase = 'saving'; error = '';
      pairSaveConfirmed = false;
      confirmedThisAttempt = {A: false, B: false};
      try {
        for (const slot of slots) {
          // Each Native acknowledgement belongs only to this project. If B
          // fails, preserve A's real acknowledgement without claiming a pair
          // transaction, rolling A back, or automatically retrying either side.
          locations[slot] = await sessions[slot].saveNotebook();
          confirmedThisAttempt[slot] = true;
          live(); if (epoch !== ticket) throw new Error('실험 판본이 바뀌었습니다. 두 실험 저장 완료로 게시하지 않습니다.');
        }
        phase = 'saved'; pairSaveConfirmed = true; return {...locations};
      } catch (cause) { phase = 'partial-or-failed-save'; error = sourceStorageErrorMessage(cause); throw cause; }
      finally { busy = false; }
    },
    async open(receipts) {
      idle();
      const ids = slots.map(slot => JSON.parse(receipts?.[slot] ?? '{}').native_source_project_id);
      distinct(...ids);
      const ticket = epoch, candidates = {A: make('A'), B: make('B')};
      busy = true; phase = 'opening'; error = ''; published = {A: false, B: false}; pairSaveConfirmed = false;
      try {
        for (const slot of slots) {
          await candidates[slot].open(receipts[slot]);
          live(); if (epoch !== ticket) throw new Error('실험 판본이 바뀌어 재열기 응답을 적용하지 않습니다.');
        }
        for (const slot of slots) sessions[slot].dispose();
        sessions = candidates; locations = {A: receipts.A, B: receipts.B}; epoch++;
        phase = 'needs-recheck'; pairSaveConfirmed = true; confirmedThisAttempt = {A: false, B: false};
        return snapshot();
      } catch (cause) {
        for (const slot of slots) candidates[slot].dispose();
        phase = 'open-failed'; error = sourceStorageErrorMessage(cause); throw cause;
      } finally { busy = false; }
    },
    async recompute(slot, commonProducer) {
      slotName(slot); idle();
      const selected = sessions[slot]; busy = true; published[slot] = false; error = '';
      try {
        // The producer performs actual recheck/calculation. Storage does not
        // interpret a returned object as mathematical success or a proof.
        await selected.recheck(commonProducer);
        live(); if (sessions[slot] !== selected) throw new Error('다른 실험의 이전 계산 응답입니다.');
        published[slot] = true; phase = 'producer-returned'; return snapshot();
      } catch (cause) { phase = 'calculation-failed'; error = sourceStorageErrorMessage(cause); throw cause; }
      finally { busy = false; }
    },
    dispose() { disposed = true; epoch++; for (const slot of slots) sessions[slot].dispose(); },
  };
}
