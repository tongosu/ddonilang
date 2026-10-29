// View-only state. The producer and its explicit projection own all math,
// quantity labels and numerical samples; this module evaluates no DDN.
export function createNotebookModel({ readSource, inspect, project, changed = () => {} }) {
  let generation = 0, busy = false, current = null, error = '', disposed = false;
  let comparisons = [];
  const snapshot = () => structuredClone({ busy, current, comparisons, error });
  const notify = () => changed(snapshot());
  function live() { if (disposed) throw new Error('닫힌 노트입니다.'); }
  function invalidate() {
    generation++; busy = false; current = null; error = ''; notify();
  }
  return {
    snapshot,
    invalidate,
    reopened() {
      live(); comparisons = []; invalidate();
    },
    async calculate() {
      live();
      if (busy) throw new Error('계산 중입니다.');
      if (typeof inspect !== 'function' || typeof project !== 'function') {
        throw new Error('공통 계산 공급이 아직 연결되지 않았습니다.');
      }
      const source = readSource();
      if (typeof source !== 'string' || !source) throw new Error('현재 작품 원문을 먼저 저장하세요.');
      const ticket = ++generation;
      busy = true; current = null; error = ''; notify();
      try {
        const report = await inspect(source);
        if (disposed || generation !== ticket || readSource() !== source) {
          throw new Error('원문이 바뀌어 이전 계산 결과를 적용하지 않았습니다.');
        }
        // Projection is supplied with the real producer, never inferred from
        // names or result-shaped JSON by the view.
        const view = validateNotebookView(project(report, source));
        if (disposed || generation !== ticket || readSource() !== source) {
          throw new Error('원문이 바뀌어 이전 계산 결과를 적용하지 않았습니다.');
        }
        current = { source, view: structuredClone(view) };
        return snapshot();
      } catch (cause) {
        if (!disposed && generation === ticket) { current = null; error = String(cause.message ?? cause); }
        throw cause;
      } finally {
        if (!disposed && generation === ticket) { busy = false; notify(); }
      }
    },
    pin() {
      live();
      if (!current || readSource() !== current.source) throw new Error('현재 원문으로 계산한 결과가 필요합니다.');
      if (comparisons.length === 2) throw new Error('비교할 두 실험이 이미 있습니다. 비교를 비운 뒤 다시 선택하세요.');
      if (comparisons[0]?.view.comparisonKey) {
        if (JSON.stringify(comparisons[0].view.comparisonKey) !== JSON.stringify(current.view.comparisonKey)) {
          throw new Error('속도 외의 입력도 달라졌습니다. 같은 조건의 실험을 선택하세요.');
        }
        if (JSON.stringify(comparisons[0].view.comparedInput) === JSON.stringify(current.view.comparedInput)) {
          throw new Error('속도를 바꾼 별도 실험을 선택하세요.');
        }
      }
      comparisons.push(structuredClone(current)); notify();
    },
    clearComparisons() { live(); comparisons = []; notify(); },
    dispose() { disposed = true; generation++; current = null; comparisons = []; },
  };
}

export function sameEquationContext(a, b) {
  return Boolean(a && b && a.projectId === b.projectId && a.path === b.path
    && a.generation === b.generation && a.sourceText === b.sourceText);
}

// Text insertion only. Expressions are never parsed, simplified or evaluated.
export function equationStepText({ operation, reference, term, left, right, operationName, claimName, resultPlace = false }) {
  if (!['subtract', 'divide'].includes(operation)) throw new Error('조작을 선택하세요.');
  for (const value of [reference, term, left, right, operationName, claimName]) {
    if (typeof value !== 'string' || !value.trim()) throw new Error('등식·항·근거·단계 이름을 모두 적으세요.');
  }
  const name = operation === 'subtract' ? '양변빼기' : '양변나누기';
  const pin = operation === 'subtract' ? '항' : '계수';
  return `${operationName} := (근거=${reference}, ${pin}=${term}) ${name}.\n${claimName} ${resultPlace ? '<-' : ':='} (근거=${operationName}, 왼=${left}, 오른=${right}) 등식정리.`;
}

export function equationSourceSlice(source, span) {
  const bytes = new TextEncoder().encode(source);
  if (!span || !Number.isSafeInteger(span.start) || !Number.isSafeInteger(span.end)
      || span.start < 0 || span.end < span.start || span.end > bytes.length) throw new Error('원문 범위를 확인할 수 없습니다.');
  return new TextDecoder('utf-8', { fatal: true }).decode(bytes.slice(span.start, span.end));
}

export async function validateEquationObservation(report, source, path) {
  const bytes = new TextEncoder().encode(source);
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  const hash = Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, '0')).join('');
  if (report?.observation_only !== true || report.unconditional_proof_issued !== false
      || !['accepted', 'rejected'].includes(report.status) || !['analysis', 'checker'].includes(report.phase)
      || report.source_path !== path || report.source_sha256 !== hash || report.source_bytes !== bytes.length
      || !Array.isArray(report.definitions)) throw new Error('단계 관찰의 현재 원문 결속이 일치하지 않습니다.');
  if (report.phase === 'analysis' && report.definitions.length) throw new Error('분석 실패에 검사된 단계를 붙일 수 없습니다.');
  if (report.diagnostic?.span) equationSourceSlice(source, report.diagnostic.span);
  let stopped = false;
  for (const definition of report.definitions) {
    if (typeof definition.name !== 'string' || !['accepted', 'rejected', 'not_checked'].includes(definition.status)
        || !Array.isArray(definition.steps)) throw new Error('정의별 단계 관찰을 확인할 수 없습니다.');
    equationSourceSlice(source, definition.span);
    if (definition.diagnostic?.span) equationSourceSlice(source, definition.diagnostic.span);
    for (const step of definition.steps) {
      if (!Number.isSafeInteger(step.index) || !['checked', 'failed', 'not_checked'].includes(step.status)
          || typeof step.operation !== 'string') throw new Error('단계 상태 공급이 올바르지 않습니다.');
      equationSourceSlice(source, step.span);
      equationSourceSlice(source, step.operand_span);
      if (stopped && step.status !== 'not_checked') throw new Error('실패 뒤 단계를 검사 완료로 표시할 수 없습니다.');
      if (step.status !== 'checked') {
        if (step.equality !== null || step.reciprocal !== null) throw new Error('미수용 단계의 검사값을 표시할 수 없습니다.');
        stopped = true;
      } else {
        if (!step.equality) throw new Error('검사된 양쪽 항 공급이 없습니다.');
        equationSourceSlice(source, step.equality.left_span);
        equationSourceSlice(source, step.equality.right_span);
      }
    }
    if (definition.status !== 'accepted') stopped = true;
    if (definition.status === 'accepted' && definition.steps.some(step => step.status !== 'checked')) {
      throw new Error('정의 수용과 단계 상태가 일치하지 않습니다.');
    }
  }
  if (report.status === 'accepted' && (!report.definitions.length
      || report.definitions.some(definition => definition.status !== 'accepted'))) throw new Error('전체 수용 상태가 일치하지 않습니다.');
  return report;
}

export function createEquationAuthoringModel({ readContext, applySource, inspect, changed = () => {} }) {
  let context = null, draft = '', valid = false, disposed = false, report = null, busy = false, epoch = 0, error = '';
  const snapshot = () => ({ context: context && { ...context }, draft, valid,
    dirty: Boolean(context && draft !== context.sourceText), busy, error,
    report: report && structuredClone(report), resultStatus: report ? 'PRODUCED' : 'NOT_PRODUCED' });
  function live() { if (disposed) throw new Error('닫힌 풀이 원문입니다.'); }
  const notify = () => changed(snapshot());
  function clearReport() { epoch++; report = null; busy = false; error = ''; }
  return {
    snapshot,
    load() {
      live(); const current = readContext?.();
      if (!current || typeof current.sourceText !== 'string' || !current.projectId || !current.path
          || !Number.isSafeInteger(current.generation)) throw new Error('현재 편집 원문을 읽을 수 없습니다.');
      clearReport(); context = { ...current }; draft = current.sourceText; valid = true; notify(); return snapshot();
    },
    edit(text) { live(); if (typeof text !== 'string') throw new TypeError('원문이 필요합니다.'); clearReport(); draft = text; notify(); },
    insert(start, end, fields) {
      live();
      if (!context || !valid) throw new Error('현재 원문을 먼저 읽으세요.');
      if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 0 || end < start || end > draft.length) {
        throw new Error('전체 원문에서 바꿀 범위를 직접 선택하세요.');
      }
      const text = equationStepText(fields);
      clearReport(); draft = draft.slice(0, start) + text + draft.slice(end); notify(); return snapshot();
    },
    apply() {
      live();
      if (!valid || !sameEquationContext(context, readContext?.())) throw new Error('작품·파일·원문 판본이 바뀌었습니다. 현재 원문을 다시 읽으세요.');
      if (typeof applySource !== 'function') throw new Error('기존 편집 연결이 없습니다.');
      const expected = { ...context }, next = draft;
      const applied = applySource(expected, next);
      if (!applied || applied.projectId !== expected.projectId || applied.path !== expected.path || applied.sourceText !== next) {
        valid = false; notify(); throw new Error('적용된 원문을 확인하지 못했습니다.');
      }
      context = { ...applied }; valid = true; notify(); return snapshot();
    },
    async check() {
      live();
      if (busy) throw new Error('검사 중입니다.');
      if (!valid || draft !== context?.sourceText || !sameEquationContext(context, readContext?.())) throw new Error('현재 원문을 편집기에 적용한 뒤 검사하세요.');
      if (typeof inspect !== 'function') throw new Error('고정 단계 관찰 공급이 필요합니다.');
      const expected = { ...context }, ticket = ++epoch;
      busy = true; report = null; error = ''; notify();
      try {
        const observed = await inspect(expected.sourceText, expected.path, { includeSteps: true });
        if (disposed || ticket !== epoch || !sameEquationContext(expected, readContext?.())) throw new Error('원문 판본이 바뀌어 이전 검사를 적용하지 않았습니다.');
        await validateEquationObservation(observed, expected.sourceText, expected.path);
        if (disposed || ticket !== epoch || !sameEquationContext(expected, readContext?.())) throw new Error('원문 판본이 바뀌어 이전 검사를 적용하지 않았습니다.');
        report = structuredClone(observed); return snapshot();
      } catch (cause) {
        if (!disposed && ticket === epoch) { report = null; error = String(cause.message ?? cause); }
        throw cause;
      } finally { if (!disposed && ticket === epoch) { busy = false; notify(); } }
    },
    invalidate() { if (!disposed) { clearReport(); valid = false; notify(); } },
    dispose() { disposed = true; epoch++; report = null; valid = false; },
  };
}

// Internal rendering arguments, not a producer wire or persistence schema.
export function validateNotebookView(view) {
  if (!view || typeof view !== 'object' || !Array.isArray(view.columns) || !view.columns.length
      || !Array.isArray(view.rows) || !view.rows.length || view.rows.length > 2000) {
    throw new Error('표시할 공통 계산 결과가 없습니다.');
  }
  if (!view.columns.every(column => typeof column.label === 'string' && column.label
      && typeof column.unit === 'string')) throw new Error('열 이름과 단위 공급이 필요합니다.');
  if (!view.rows.every(row => Array.isArray(row) && row.length === view.columns.length
      && row.every(cell => typeof cell === 'string'))) throw new Error('정확값 표시가 공급되지 않았습니다.');
  if (view.graph !== null && view.graph !== undefined) {
    if (!Array.isArray(view.graph.series) || !view.graph.series.length
        || !view.graph.series.every(series => Array.isArray(series.points) && series.points.length
          && series.points.every(p => typeof p.x === 'number' && Number.isFinite(p.x)
            && typeof p.y === 'number' && Number.isFinite(p.y)))
        || !view.axisLabels || typeof view.axisLabels.x !== 'string' || !view.axisLabels.x
        || typeof view.axisLabels.y !== 'string' || !view.axisLabels.y) {
      throw new Error('그래프 좌표와 축 이름의 명시적 공급이 필요합니다.');
    }
  }
  if (view.positions !== null && view.positions !== undefined) {
    if (typeof view.positionAxis !== 'string' || !view.positionAxis
      || !Array.isArray(view.positions) || !view.positions.length || !view.positions.every(p =>
      typeof p.label === 'string' && typeof p.exact === 'string'
      && typeof p.coordinate === 'number' && Number.isFinite(p.coordinate))) {
      throw new Error('위치 그림의 좌표와 정확값 공급이 필요합니다.');
    }
  }
  return view;
}
