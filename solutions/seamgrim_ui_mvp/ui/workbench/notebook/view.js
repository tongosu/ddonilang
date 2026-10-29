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
    <button type="button" data-equation="check" disabled>검사</button></div>
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
  if (term.kind === 'integer' && typeof term.value === 'string') return term.value;
  if (term.kind === 'parameter' && typeof term.name === 'string') return term.name;
  if (term.kind === 'divide_by_integer' && typeof term.denominator === 'string') {
    return `(${equationTermText(term.left, depth + 1)} / ${term.denominator})`;
  }
  const symbol = { add: '+', subtract: '−', multiply: '×' }[term.kind];
  if (!symbol) throw new Error('지원하지 않는 검사 항 표시입니다.');
  return `(${equationTermText(term.left, depth + 1)} ${symbol} ${equationTermText(term.right, depth + 1)})`;
}

export function equationObservationHtml(report, source) {
  if (!report) return '';
  const diagnostic = item => item ? `<pre>${escape(item.formatted || item.message || item.code)}</pre>` : '';
  const status = { checked: '검사됨', failed: '첫 거부', not_checked: '미검사' };
  return `<p>${report.phase === 'analysis' ? '원문 분석' : '조건부 단계 검사'} · ${report.status === 'accepted' ? '이번 가정 아래 수용 · 무가정 증명 발급 아님' : '거부'}</p>
    ${diagnostic(report.diagnostic)}${report.definitions.map(definition => `<section><h4>${escape(definition.name)} · ${escape(definition.status)}</h4>
      ${diagnostic(definition.diagnostic)}<ol>${definition.steps.map(step => {
        const prior = step.prior && ['assumption', 'step'].includes(step.prior.kind)
          ? `${step.prior.kind === 'assumption' ? '가정' : '원문 단계'} ${step.prior.index + 1}` : '근거 미공급';
        const equality = step.status === 'checked' ? `${equationTermText(step.equality.left)} = ${equationTermText(step.equality.right)}` : '검사된 양변 없음';
        return `<li><strong>${escape(step.operation)} · ${escape(status[step.status])}</strong> · ${escape(prior)}
          <p>${escape(equality)}</p>${step.reciprocal ? `<p>검사된 역수 ${escape(equationTermText(step.reciprocal))}</p>` : ''}
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

export function notebookResultHtml(view, mode = 'table', ranges = {}) {
  if (!view) return '';
  if (mode === 'graph') {
    if (!view.graph) return '<p>그래프 자료 없음</p>';
    if (ranges.incompatible) return '<p>축 표시가 달라 표로 비교하세요.</p>';
    const basis = ranges.axis ?? comparisonRanges([view]).axis;
    const axis = notebookViewport(basis, ranges);
    if (!axis) return '<p>좌표 표시 범위를 넘었습니다. 표의 정확값을 확인하세요.</p>';
    return `<figure>${buildGraphPreviewHtml({ ...view.graph, axis },
      { width: 640, height: 320, axisLabels: view.axisLabels, clipId: ranges.clipId ?? 'notebook-current-plot' })}
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
  return { x_min: centerX - scaledX / 2, x_max: centerX + scaledX / 2,
    y_min: centerY - scaledY / 2, y_max: centerY + scaledY / 2 };
}
