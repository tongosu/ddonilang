import { buildGraphPreviewHtml } from '../../components/graph_preview.js';
import { equationSourceSlice } from './model.js';

const escape = value => String(value).replace(/[&<>"']/g, c =>
  ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

export function equationAuthoringHtml() {
  return `<h3>풀이 원문</h3>
    <div class="equation-actions"><button type="button" data-equation="load">현재 원문 읽기</button>
    <button type="button" data-equation="apply">편집기에 적용</button>
    <button type="button" data-equation="save">원문 저장</button>
    <button type="button" data-equation="open">저장 원문 열기</button>
    <button type="button" data-equation="check" disabled>일반 검사</button>
    <button type="button" data-equation="targets" disabled>가정 대상 읽기</button>
    <button type="button" data-equation="linear-check" disabled>일차 문제군 검사</button></div>
    <label class="equation-source-label">초기 가정 선택 <select data-equation="target" style="width:100%;min-width:0;max-width:100%" disabled><option value="">대상을 읽고 선택하세요</option></select></label>
    <p>Native 저장·재열기는 ‘새 프로젝트 → 빈 DDN 프로젝트’의 풀이 원문에서 시작하세요. 단일 파일 열기의 로컬 보관과는 구별됩니다. B 검사는 선택한 정의만 검사하며 작품 실행 검사를 대신하지 않습니다.</p>
    <p role="status" data-equation="status">NOT_PRODUCED · 단계 검사 연결 대기</p>
    <fieldset class="equation-fields"><legend>선택 범위에 넣을 조작과 등식</legend>
      <label>조작 <select data-equation="operation"><option value="subtract">양변에서 같은 항 빼기</option><option value="divide">양변을 같은 계수로 나누기</option></select></label>
      <label>근거 <input data-equation="reference" placeholder="근거 또는 앞 단계 이름"></label>
      <label>항·계수 <input data-equation="term" placeholder="직접 입력"></label>
      <label>왼쪽 식 <input data-equation="left" placeholder="직접 입력"></label>
      <label>오른쪽 식 <input data-equation="right" placeholder="직접 입력"></label>
      <label>조작 단계 이름 <input data-equation="operationName" placeholder="예: 빼기"></label>
      <label>정리 단계 이름 <input data-equation="claimName" placeholder="예: 정리"></label>
      <label>명명 결과자리 <input type="checkbox" data-equation="resultPlace"></label>
      <button type="button" data-equation="insert">선택 범위에 두 줄 넣기</button>
    </fieldset>
    <label class="equation-source-label">전체 DDN 원문 <textarea data-equation="source" spellcheck="false" rows="12"></textarea></label>
    <p>조작과 등식정리를 모두 확인한 뒤 적용하세요.</p>
    <section data-equation="result" aria-label="풀이 단계 검사 결과"></section>`;
}

function equationTermText(term, depth = 0) {
  if (!term || depth > 64) throw new Error('표시할 검사 항이 올바르지 않습니다.');
  if (term.kind === 'source_bound_expression' && typeof term.display_text === 'string') return term.display_text;
  if (term.kind === 'integer' && typeof term.value === 'string') return term.value;
  if (term.kind === 'parameter' && typeof term.name === 'string') return term.name;
  if (term.kind === 'divide_by_integer' && typeof term.denominator === 'string') {
    return `(${equationTermText(term.left, depth + 1)} / ${term.denominator})`;
  }
  const symbol = { add: '+', subtract: '−', multiply: '×' }[term.kind];
  if (!symbol) throw new Error('지원하지 않는 검사 항 표시입니다.');
  return `(${equationTermText(term.left, depth + 1)} ${symbol} ${equationTermText(term.right, depth + 1)})`;
}

function equationRelationText(relation) {
  if (relation?.relation === 'nonzero') return `${equationTermText(relation.expression)} != 0`;
  return `${equationTermText(relation.left)} = ${equationTermText(relation.right)}`;
}
function conditionalDomainHtml(definition) {
  if (definition.conditional_definedness !== true) return '';
  const used=Array.isArray(definition.used_assumptions)
    ? definition.used_assumptions.map(i=>Number.isSafeInteger(i)&&i>=0&&i<(definition.assumptions??[]).length?`가정 ${i+1}`:'자료 미공급').join(', ')||'없음' : '검사 수용 근거 미공급';
  const conditions=Array.isArray(definition.retained_conditions)?definition.retained_conditions:null;
  const legend=(definition.parameters??[]).map(p=>`${p.kernel_symbol}: ${p.name}`).join(', ');
  return `<section class="equation-conditional-domain"><p>명시 가정 아래의 조건부 대수 검사 · 가정 자체의 참이나 무가정 증명 아님</p>
    <p>사용 근거: ${escape(used)}</p><p>원래 분모의 비영 조건(공급 표기): ${conditions?escape(conditions.join(', ')||'없음'):'검사 수용 조건 미공급'}</p>
    <p>공급 기호와 현재 정의의 값핀: ${escape(legend||'자료 미공급')}</p>
    <p>약분해도 원래 제외점은 유지됩니다. 결과식의 값으로 제외점의 원식을 대신하지 않습니다.</p></section>`;
}

export function equationObservationHtml(report, source) {
  if (!report) return '';
  const diagnostic = item => item ? `<pre>${escape(item.formatted || item.message || item.code)}</pre>` : '';
  const status = { checked: '검사됨', failed: '첫 거부', not_checked: '미검사' };
  const referenceText = reference => reference && ['assumption', 'step'].includes(reference.kind)
    && Number.isSafeInteger(reference.index) && reference.index >= 0
    ? `${reference.kind === 'assumption' ? '가정' : '원문 단계'} ${reference.index + 1}` : null;
  function authoredClaimHtml(definition) {
    const goal = definition.goal;
    const claim = goal?.left && goal?.right
      ? `${equationTermText(goal.left)} = ${equationTermText(goal.right)}` : '주장 구조 자료 미공급';
    const hasAssumptions = Array.isArray(definition.assumptions);
    const assumptions = hasAssumptions ? definition.assumptions : [];
    return `<section class="equation-authored-claim"><p><strong>작성자 주장 · 검사 결과와 구분</strong></p>
      <p>${escape(claim)}</p>${assumptions.length ? `<ul>${assumptions.map((item,index) =>
        `<li>가정 ${index + 1}: ${escape(equationRelationText(item))}</li>`).join('')}</ul>`
        : `<p>${hasAssumptions ? '공급된 가정 없음' : '가정 구조 자료 미공급'}</p>`}
      <details><summary>주장과 가정의 실제 원문</summary><pre>${escape(equationSourceSlice(source,definition.span))}</pre></details></section>`;
  }
  function callHtml(operand) {
    if (operand?.kind !== 'definition_call') return '';
    const callee = typeof operand.definition === 'string' && operand.definition.length
      ? operand.definition : '호출 정의 자료 미공급';
    const values = Array.isArray(operand.values)
      ? operand.values.length ? operand.values.map(value => equationTermText(value)).join(', ') : '공급된 값 인수 없음'
      : '값 인수 자료 미공급';
    return `<p>호출 정의: ${escape(callee)}</p><p>값 인수: ${escape(values)}</p>`;
  }
  function evidenceText(step) {
    if (step.operand?.kind !== 'definition_call') return referenceText(step.prior) ?? '근거 미공급';
    const evidence = step.operand.evidence;
    if (!Array.isArray(evidence)) return '호출 근거 자료 미공급';
    if (!evidence.length) return '공급된 호출 근거 없음';
    const references = evidence.map(referenceText);
    return references.every(Boolean) ? `호출 근거: ${references.join(', ')}` : '호출 근거 자료 미공급';
  }
  return `<p>${report.phase === 'analysis' ? '원문 분석' : '조건부 단계 검사'} · ${({accepted:'이번 가정 아래 수용 · 무가정 증명 발급 아님',rejected:'거부',not_checked:'미검사'}[report.status]??'상태 미확인')}</p>
    ${diagnostic(report.diagnostic)}${report.definitions.map(definition => `<section><h4>${escape(definition.name)} · ${escape(definition.status)}</h4>
      ${authoredClaimHtml(definition)}${conditionalDomainHtml(definition)}${diagnostic(definition.diagnostic)}<ol>${definition.steps.map(step => {
        const equality = step.status !== 'checked' ? '검사된 양변 없음'
          : step.equality?.left && step.equality?.right
            ? `검사된 양변: ${equationTermText(step.equality.left)} = ${equationTermText(step.equality.right)}` : '검사된 양변 자료 미공급';
        return `<li><strong>${escape(step.operation)} · ${escape(status[step.status])}</strong> · ${escape(evidenceText(step))}
          ${callHtml(step.operand)}<p>${escape(equality)}</p>${step.status === 'checked' && step.reciprocal ? `<p>검사된 역수 ${escape(equationTermText(step.reciprocal))}</p>` : ''}
          <details><summary>실제 단계 원문</summary><pre>${escape(equationSourceSlice(source, step.span))}</pre></details></li>`;
      }).join('')}</ol></section>`).join('')}`;
}

export function comparisonRanges(views) {
  const graphs = views.filter(v => v?.graph);
  if (graphs.length > 1 && graphs.some(v => v.axisLabels.x !== graphs[0].axisLabels.x
      || v.axisLabels.y !== graphs[0].axisLabels.y)) return { incompatible: true };
  const points = graphs.flatMap(v => v.graph.series.flatMap(s => s.points));
  const positions = views.flatMap(v => v?.positions ?? []).map(p => p.coordinate);
  const positionViews = views.filter(v => v.positions?.length);
  const positionIncompatible = positionViews.some(v => v.positionAxis !== positionViews[0]?.positionAxis);
  return {
    positionIncompatible,
    axis: points.length ? { x_min: Math.min(...points.map(p => p.x)), x_max: Math.max(...points.map(p => p.x)),
      y_min: Math.min(...points.map(p => p.y)), y_max: Math.max(...points.map(p => p.y)) } : null,
    position: positions.length ? { low: Math.min(...positions), high: Math.max(...positions) } : null,
  };
}

export function notebookExperimentHtml(view, mode = 'table', ranges = {}) {
  if (!view) return '';
  return `${view.inputSummary ? `<p>${escape(view.inputSummary)}</p>` : ''}${notebookResultHtml(view, mode, ranges)}${view.solutionMessage ? `<p class="notebook-solution">${escape(view.solutionMessage)}</p>` : ''}`;
}

export function notebookErrorHtml(message) {
  if (!message) return '';
  const text = String(message);
  const code = text.match(/\bE_[A-Z_]+\b/u)?.[0];
  const descriptions = {
    E_PROJECT_MODULE_PARSE: ['원문 분석에서 문법 또는 단위 조건을 통과하지 못했습니다.', '진단에 나온 식과 단위를 확인하고 마지막 정상 원문과 비교하세요.'],
    E_NOTEBOOK_DIMENSION: ['입력과 단위 기준의 차원이 맞지 않습니다.', '이번 모형에 맞는 양·변화량·시간 단위를 확인하세요.'],
    E_NOTEBOOK_UNIT_BASIS: ['현재 모형에서 지원하지 않는 단위 기준입니다.', '지원되는 기준 단위를 사용하세요. 화면의 단위 글자만 바꾸지 마세요.'],
    E_NOTEBOOK_INPUT: ['계산에 필요한 입력 구성이 맞지 않습니다.', '원문의 탐구계수와 탐구시각을 수업 예제와 비교하세요.'],
    E_NOTEBOOK_TIME_DOMAIN: ['표본 시각이 이번 모형의 시간 범위 밖입니다.', '표본에는 0초 이상의 시각을 넣으세요. 목표의 음수 대수해와는 다른 입력 문제입니다.'],
    E_NOTEBOOK_BUDGET: ['원문이나 표본이 계산 한도를 넘었습니다.', '진단에서 지목한 크기·표본 수를 줄인 뒤 다시 계산하세요.'],
    E_NOTEBOOK_EXACT_REQUIRED: ['이 입력은 정확한 유리수 계수가 필요합니다.', '예제의 나눔수 정의와 입력 값을 확인하세요.'],
  };
  const [reason, correction] = descriptions[code] ?? ['현재 원문으로 계산을 완료하지 못했습니다.', '아래 진단과 현재 원문을 확인한 뒤 명시적으로 다시 계산하세요.'];
  return `<section class="notebook-error" role="alert"><dl><dt>확인할 곳</dt><dd>현재 계산 원문 · 정확한 위치는 공급된 진단을 확인하세요.</dd><dt>이유</dt><dd>${escape(reason)}</dd><dt>고치는 방법</dt><dd>${escape(correction)}</dd></dl><details><summary>진단 원문${code ? ` · ${escape(code)}` : ''}</summary><pre>${escape(text)}</pre></details></section>`;
}

export function notebookResultHtml(view, mode = 'table', ranges = {}) {
  if (!view) return '';
  if (mode === 'graph') {
    if (!view.graph) return '<p>그래프 자료 없음</p>';
    if (ranges.incompatible) return '<p>축 표시가 달라 표로 비교하세요.</p>';
    const basis = ranges.axis ?? comparisonRanges([view]).axis;
    const axis = notebookViewport(basis, ranges);
    if (!axis) return '<p>좌표 표시 범위를 넘었습니다. 표의 정확값을 확인하세요.</p>';
    return `<figure>${buildGraphPreviewHtml({ ...view.graph, axis },
      { width: 360, height: 240, axisLabels: view.axisLabels, clipId: ranges.clipId ?? 'notebook-current-plot' })}
      <figcaption>${escape(view.axisLabels.x)} / ${escape(view.axisLabels.y)}</figcaption></figure>`;
  }
  if (mode === 'position') {
    if (!view.positions?.length) return '<p>위치 자료 없음</p>';
    if (ranges.incompatible || ranges.positionIncompatible) return '<p>축 표시가 달라 표로 비교하세요.</p>';
    const coordinates = view.positions.map(p => p.coordinate);
    const low = ranges.position?.low ?? Math.min(...coordinates), high = ranges.position?.high ?? Math.max(...coordinates);
    // Pixel mapping only. Exact values are displayed verbatim alongside it.
    return `<p>${escape(view.positionAxis)} · ${escape(low)} ~ ${escape(high)}</p><ol class="notebook-positions">${view.positions.map(p => {
      const offset = high === low ? 50 : 5 + 90 * (p.coordinate - low) / (high - low);
      return `<li><span>${escape(p.label)}</span><span class="notebook-position-track"><span class="notebook-position-dot" style="left:${offset}%">●</span></span><output>${escape(p.exact)}</output></li>`;
    }).join('')}</ol>`;
  }
  return `<div class="notebook-table-scroll"><table><thead><tr>${view.columns.map(c =>
    `<th scope="col">${escape(c.label)}${c.unit ? ` (${escape(c.unit)})` : ''}</th>`).join('')}</tr></thead>
    <tbody>${view.rows.map(row => `<tr>${row.map(cell => `<td>${escape(cell)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;
}

export function notebookViewport(axis, { zoom = 1, pan = 0 } = {}) {
  if (!axis || !Number.isFinite(zoom) || zoom < 1 || zoom > 4 || !Number.isFinite(pan) || Math.abs(pan) > 1) return null;
  const xSpan = axis.x_max - axis.x_min, ySpan = axis.y_max - axis.y_min;
  if (!Number.isFinite(xSpan) || !Number.isFinite(ySpan)) return null;
  const scaledX = (xSpan || 1) / zoom, scaledY = (ySpan || 1) / zoom;
  const centerX = axis.x_min + xSpan / 2 + pan * xSpan / 2;
  const centerY = axis.y_min + ySpan / 2;
  // These are display bounds only, not exact quantities or comparison inputs.
  const displayBound = value => Number(value.toPrecision(12));
  return { x_min: displayBound(centerX - scaledX / 2), x_max: displayBound(centerX + scaledX / 2),
    y_min: displayBound(centerY - scaledY / 2), y_max: displayBound(centerY + scaledY / 2) };
}
