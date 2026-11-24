// Presentation-only helpers. The host owns observation identity and lifecycle.
// No quantity calculation, nearest-point lookup, interpolation or persistence.
const escape = value => String(value).replace(/[&<>"']/g, c =>
  ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

function rowsOf(view) {
  if (!view || !Array.isArray(view.columns) || !view.columns.length
      || !view.columns.every(c => typeof c.label === 'string' && typeof c.unit === 'string')
      || !Array.isArray(view.rows) || !view.rows.length || view.rows.length > 2000
      || !view.rows.every(row => Array.isArray(row) && row.length === view.columns.length
        && row.every(cell => typeof cell === 'string'))) {
    throw new Error('정확한 표본과 열 이름·단위 공급이 필요합니다.');
  }
  return view.rows;
}

export function selectedSample(view, index) {
  const rows = rowsOf(view);
  if (index === null) return null;
  if (!Number.isSafeInteger(index) || index < 0 || index >= rows.length) {
    throw new Error('현재 관측에 없는 표본입니다.');
  }
  return { index, cells: view.columns.map((column, i) =>
    ({ label: column.label, unit: column.unit, exact: rows[index][i] })) };
}

export function sampleSelectionHtml(view, index = null) {
  if (!view) return '';
  const rows = rowsOf(view), selected = selectedSample(view, index);
  return `<label>표본 선택 <select data-notebook-sample aria-label="표본 선택">
    <option value=""${index === null ? ' selected' : ''}>선택 없음</option>
    ${rows.map((row, i) => `<option value="${i}"${i === index ? ' selected' : ''}>${i + 1}번 · ${escape(row.join(' / '))}</option>`).join('')}
    </select></label><div role="status" aria-live="polite" data-notebook-sample-readout>
    ${selected ? `<strong>선택 표본 ${index + 1}</strong><dl>${selected.cells.map(cell =>
      `<dt>${escape(cell.label)}</dt><dd>${escape(cell.exact)}${cell.unit ? ` ${escape(cell.unit)}` : ''}</dd>`).join('')}</dl>
      <p>위 값은 공급된 정확값입니다. 그래프와 눈금의 위치는 화면 근사입니다.</p>` : '표본을 선택하면 정확값과 단위를 읽을 수 있습니다.'}</div>`;
}

// Decorate existing rendered elements. Graph coordinates remain untouched.
// Only the current one-series, row-aligned projection is admitted here.
export function markSelectedSample(root, view, index, mode) {
  const selected = selectedSample(view, index);
  let elements;
  if (mode === 'table') {
    elements = [...root.querySelectorAll('tbody > tr')];
  } else if (mode === 'position') {
    if (!view.positions) return { status: 'unavailable' };
    if (!Array.isArray(view.positions) || view.positions.length !== view.rows.length) {
      throw new Error('눈금과 표의 표본 대응이 다릅니다.');
    }
    elements = [...root.querySelectorAll('.notebook-positions > li')];
  } else if (mode === 'graph') {
    if (!view.graph) return { status: 'unavailable' };
    const series = view.graph.series;
    if (!Array.isArray(series) || series.length !== 1 || !Array.isArray(series[0].points)
        || series[0].points.length !== view.rows.length || !series[0].points.every(p =>
          Number.isFinite(p.x) && Number.isFinite(p.y))) {
      throw new Error('그래프와 표의 표본 대응을 확인할 수 없습니다.');
    }
    elements = [...root.querySelectorAll('circle.runtime-graph-point')];
  } else {
    throw new Error('지원하지 않는 표본 보기입니다.');
  }
  if (elements.length !== view.rows.length) {
    throw new Error('화면과 관측의 표본 수가 다릅니다.');
  }
  elements.forEach((element, i) => {
    if (selected && i === selected.index) {
      element.setAttribute('data-notebook-sample-selected', 'true');
      element.setAttribute('aria-current', 'true');
    } else {
      element.removeAttribute('data-notebook-sample-selected');
      element.removeAttribute('aria-current');
    }
  });
  return { status: 'ready', index };
}
