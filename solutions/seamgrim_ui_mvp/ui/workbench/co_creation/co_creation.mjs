// 작성 도구 후보. 의미 검사는 주입된 공통 제품 검사만 사용한다.
export function createCoCreation({ read, compareAndSet, validate, provider = null, timeoutMs = 30000, readCompletedRuns = () => [], selectCompletedRunPair = () => { throw Error("완료 실행 공급이 없습니다."); } }) {
  let generation = 0, pending = null, candidate = null, undo = null;
  let baseline = null, restorePreview = null;
  let state = { status: "idle", message: "바꾸고 싶은 점을 적어 주세요." };
  const listeners = new Set();
  const same = (a, b) => a.projectId === b.projectId && a.documentId === b.documentId
    && a.revision === b.revision && a.source === b.source;
  function snapshot() {
    const s = read();
    if (!s || typeof s.source !== "string" || typeof s.revision !== "string"
        || !s.projectId || !s.documentId) throw Error("문서 판본을 확인할 수 없습니다.");
    return Object.freeze({ projectId: s.projectId, documentId: s.documentId, revision: s.revision, source: s.source });
  }
  const sameDocument = (a, b) => a.projectId === b.projectId && a.documentId === b.documentId;
  function reconcileBaseline() {
    const current = snapshot();
    if (baseline && !sameDocument(baseline, current)) baseline = null;
    if (!baseline || (restorePreview && !same(restorePreview.current, current))) restorePreview = null;
    return current;
  }
  const view = () => {
    return structuredClone({ ...state, candidate, canRestore: Boolean(undo),
      baselineDocument: baseline?.documentId ?? null, restorePreview });
  };
  function publish(status, message) {
    state = { status, message };
    for (const listener of listeners) listener(view());
    return view();
  }
  function cancel() {
    restorePreview = null;
    generation++;
    pending?.abort(); pending = null; candidate = null;
    return publish("cancelled", "요청을 취소했습니다. 원문은 그대로입니다.");
  }
  function invalidate(reason = "edit") {
    if (reason === "context") baseline = null;
    restorePreview = null;
    generation++; pending?.abort(); pending = null; candidate = null; undo = null;
    return publish("stale", "원문 또는 작품이 바뀌었습니다. 현재 원문으로 다시 요청하세요.");
  }
  function preview(intent) {
    const base = snapshot();
    if (!intent.trim() || intent.length > 1000 || base.source.length > 16000)
      throw Error("변경 의도는 1~1000자, 원문은 16000자 이내여야 합니다.");
    return Object.freeze({ base, intent, disclosure: "고정 제안 데모입니다. 현재 문서와 의도를 기기 안에서만 사용하며 AI에는 연결하지 않습니다." });
  }
  async function request(approved) {
    cancel();
    if (!provider) return publish("unavailable", "AI 연결이 없습니다. 일반 편집·실행·저장은 계속할 수 있습니다.");
    if (!approved || !same(approved.base, snapshot())) return publish("stale", "보낼 원문이 바뀌었습니다. 전송 내용을 다시 확인하세요.");
    const { base, intent } = preview(approved.intent);
    const token = generation, abort = new AbortController(); pending = abort;
    let timer;
    publish("requesting", "작은 수정 후보 하나를 요청했습니다. 취소할 수 있습니다.");
    try {
      const stopped = new Promise((_, reject) => {
        abort.signal.addEventListener("abort", () => reject(Error("cancelled")), { once: true });
        timer = setTimeout(() => { reject(Error("timeout")); abort.abort(); }, timeoutMs);
      });
      const response = await Promise.race([Promise.resolve().then(() => {
        if (abort.signal.aborted) throw Error("cancelled");
        return provider({ source: base.source, intent }, { signal: abort.signal });
      }), stopped]);
      if (token !== generation) return view();
      if (!same(base, snapshot())) return invalidate();
      const e = response?.edit;
      if (typeof response?.explanation !== "string" || !response.explanation.trim() || response.explanation.length > 1000
          || !e || !Number.isInteger(e.start) || !Number.isInteger(e.end)
          || e.start < 0 || e.end < e.start || e.end > base.source.length
          || typeof e.before !== "string" || typeof e.after !== "string"
          || e.before !== base.source.slice(e.start, e.end) || e.before === e.after
          || e.before.length > 120 || e.after.length > 120 || /[\r\n]/.test(e.before + e.after))
        return publish("invalid", "한 줄 안의 작은 변경 후보가 아닙니다. 의도를 좁혀 다시 요청하세요.");
      const source = base.source.slice(0, e.start) + e.after + base.source.slice(e.end);
      candidate = { base, source, edit: { ...e }, explanation: response.explanation, checked: false };
      return publish("candidate", "설명과 바뀔 부분을 확인한 뒤 공통 제품 검사를 실행하세요.");
    } catch (error) {
      if (token !== generation) return view();
      candidate = null;
      if (error?.code === "E_CO_CREATION_UNSUPPORTED_TARGET")
        return publish("unsupported", "이 원문은 고정 후보의 지원 대상이 아닙니다. 현재 후보는 배달 로봇의 속도 1 한 곳만 지원합니다. 원문은 그대로입니다. 직접 편집해 실험하세요.");
      if (error?.code === "E_CO_CREATION_AMBIGUOUS_TARGET")
        return publish("unsupported", "고정 후보의 속도 1 대상이 여러 곳이라 한 곳을 정할 수 없습니다. 원문은 그대로입니다. 변경할 곳을 확인하고 직접 편집하세요.");
      return publish("failed", "연결이 실패했거나 제한 시간을 넘었습니다. 원문은 그대로입니다. 다시 요청하거나 직접 편집하세요.");
    } finally { clearTimeout(timer); if (token === generation) pending = null; }
  }
  async function check() {
    if (!candidate || state.status === "checking") return view();
    const current = candidate, token = generation;
    if (!same(current.base, snapshot())) return invalidate();
    publish("checking", "공통 제품 경로에서 검사하고 있습니다.");
    try {
      const result = await validate({ ...current.base, source: current.source });
      if (token !== generation || candidate !== current) return view();
      if (!same(current.base, snapshot())) return invalidate();
      if (result?.ok !== true || result.source !== current.source || !result.validationId) {
        candidate = null;
        return publish("rejected", "제품 검사를 통과하지 못했습니다. 원문은 그대로입니다. 의도를 바꿔 다시 요청하세요.");
      }
      current.checked = true; current.validationId = result.validationId;
      return publish("checked", "검사를 통과했습니다. 적용 여부는 직접 선택하세요.");
    } catch {
      if (token !== generation || candidate !== current) return view();
      candidate = null;
      return publish("rejected", "제품 검사를 완료하지 못했습니다. 일반 편집으로 돌아가거나 다시 요청하세요.");
    }
  }
  function apply() {
    if (!candidate?.checked || state.status !== "checked") return view();
    const current = candidate;
    if (!same(current.base, snapshot())) return invalidate();
    // Host의 동기 원자 교체: 판본 비교와 편집은 한 동작이어야 한다. 저장/실행하지 않는다.
    const receipt = compareAndSet(current.base, current.source);
    if (!receipt?.ok) return invalidate();
    undo = { before: current.base.source, after: snapshot() }; candidate = null;
    return publish("applied", "원문에 적용했습니다. 기존 실행으로 차이를 보고, 다음 실험은 직접 정해 보세요.");
  }
  function restore() {
    if (!undo) return view();
    if (!same(undo.after, snapshot())) return invalidate();
    if (!compareAndSet(undo.after, undo.before)?.ok) return invalidate();
    undo = null;
    return publish("restored", "적용 전 원문을 복원했습니다. 실행 세계는 되감지 않았습니다. 필요하면 다시 실행하고 저장하세요.");
  }
  function captureBaseline() {
    const current = snapshot();
    cancel(); baseline = current;
    return publish("baselineCaptured", "현재 문서의 원문을 작업 중 기준으로 보관했습니다. 문서·작품을 바꾸거나 창을 닫으면 해제됩니다.");
  }
  function previewBaselineRestore() {
    const current = reconcileBaseline();
    cancel();
    if (!baseline) return publish("baselineMissing", "보관된 기준 원문이 없습니다. 현재 원문을 먼저 보관하세요.");
    if (current.source === baseline.source) return publish("baselineUnchanged", "현재 원문과 기준 원문이 같습니다.");
    restorePreview = { current, source: baseline.source };
    return publish("baselinePreview", "현재 원문을 아래 기준 원문으로 바꿉니다. 내용을 확인한 뒤 복원을 선택하세요.");
  }
  function restoreBaseline() {
    const confirmed = restorePreview;
    const current = reconcileBaseline();
    if (!confirmed || confirmed !== restorePreview || !same(confirmed.current, current))
      return publish("baselineStale", "복원 확인 뒤 원문이 바뀌었거나 확인 내용이 없습니다. 현재 원문을 다시 확인하세요.");
    // compareAndSet may synchronously notify invalidate through the editor input event.
    const source = confirmed.source;
    restorePreview = null;
    if (!compareAndSet(current, source)?.ok) return invalidate();
    generation++; pending?.abort(); pending = null; candidate = null; undo = null;
    return publish("baselineRestored", "기준 원문으로 복원했습니다. 실행 세계는 그대로입니다. 실행과 저장은 직접 선택하세요.");
  }
  function reject() { cancel(); return publish("rejected", "후보를 거부했습니다. 원문과 저장본은 그대로입니다."); }
  return { view, preview, request, cancel, invalidate, check, apply, restore, reject, readCompletedRuns, selectCompletedRunPair,
    captureBaseline, previewBaselineRestore, restoreBaseline,
    subscribe(listener) { listeners.add(listener); listener(view()); return () => listeners.delete(listener); } };
}
