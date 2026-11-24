import {observationCellDisplay, observationRowsForDisplay} from '../authored_bogae_renderer.js';

const schema = 'ddn.public_observation_table_projection.v2';
const cellStates = new Set(['present', 'gap', 'missing', 'pruned']);
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const rawText = value => value === undefined ? '미공급' : JSON.stringify(value, null, 2);

// Renderability only. Registration, observation ownership and sealing belong
// to the producer. In particular, the global full table is never a fallback.
function inspect(state) {
  const projection = state?.bogae_projection;
  if (!projection) return {kind: 'not_produced', message: '표 관측 공급 대기 · NOT_PRODUCED'};
  if (projection.status !== 'ready' || projection.diagnostic != null) {
    const rejected = projection.diagnostic != null || ['rejected', 'invalid', 'failed', 'error'].includes(projection.status);
    return {kind: rejected ? 'rejected' : 'unsupported',
      message: `공급 상태: ${projection.status ?? '미공급'} · ${rejected ? '거부된 관측' : '표시 미지원'}`,
      diagnostic: projection.diagnostic};
  }
  if (!Array.isArray(projection.candidates) || projection.candidates.length !== 1)
    return {kind: 'unsupported', message: '하나의 보기 후보가 필요합니다. · 표시 미지원'};
  const candidate = projection.candidates[0], artifact = candidate?.artifact;
  const views = [{scope: 'primary', label: '기본 보기', view: artifact?.view, resolvedMadi: artifact?.resolved_madi,
    timePolicy: artifact?.time_policy, ...inspectView(artifact?.view)}];
  if (artifact?.relations != null && !Array.isArray(artifact.relations))
    views.push({scope: 'relation', label: '관계 자료', kind: 'unsupported', message: '관계 자료 구조를 표시할 수 없습니다.'});
  else for (const [index, relation] of (artifact?.relations ?? []).entries()) {
    // Preserve each supplied owner, including absent and unsupported views.
    // A valid primary table must not hide partial relation delivery.
    const outcome = !object(relation)
      ? {kind: 'unsupported', message: '관계 자료 구조를 표시할 수 없습니다.'}
      : relation.view == null
        ? {kind: 'not_produced', message: '이 관계의 보기가 아직 공급되지 않았습니다. · NOT_PRODUCED', diagnostic: relation.diagnostic}
        : inspectView(relation.view);
    views.push({scope: 'relation', label: `관계 보기: ${relation?.authored_name ?? relation?.kind ?? '이름 미공급'}`,
      index, relation, view: relation?.view, resolvedMadi: relation?.resolved_madi, timePolicy: relation?.time_policy,
      ...outcome});
  }
  // Each supplied view keeps its own projection, run and outcome. Never sort,
  // merge or borrow one view's table for another (even a primary relation).
  const kinds = new Set(views.map(view => view.kind));
  return {kind: kinds.size === 1 ? views[0].kind : 'mixed', candidate, views,
    message: views.length === 1 ? views[0].message : '보기별 공급 상태와 표를 확인하세요.'};
}

function suppliedTimeLabel(policy) {
  if (policy?.kind === 'fixed' && Number.isSafeInteger(policy.madi) && policy.madi >= 0)
    return '고정 ' + policy.madi + '마디';
  if (policy?.kind === 'relative' && Number.isSafeInteger(policy.offset)) {
    if (policy.offset === 0) return '현재 마디 기준';
    return '현재 마디 기준 ' + (policy.offset > 0 ? '+' : '') + policy.offset;
  }
  return policy?.kind === 'common' ? '공통 시간' : '';
}

function inspectView(view) {
  if (view == null)
    return {kind: 'not_produced', message: '이 보기가 아직 공급되지 않았습니다. · NOT_PRODUCED'};
  if (view?.kind !== 'result_table' && !(view?.kind === 'scene' && view.table_projection != null))
    return {kind: 'unsupported', message: `보기 종류: ${view?.kind ?? '미공급'} · 표 표시 미지원`};
  const table = view.table_projection;
  if (!table) return {kind: 'not_produced', message: '이 보기의 표가 아직 공급되지 않았습니다. · NOT_PRODUCED'};
  if (table.schema !== schema)
    return {kind: 'unsupported', message: `표 형식: ${table.schema ?? '미공급'} · 표시 미지원`};
  if (table.presentation_state === 'no_opportunity')
    return {kind: 'no_opportunity', message: '공개 관측 기회가 없었습니다. · no_opportunity'};
  if (table.presentation_state === 'run_failed')
    return {kind: 'run_failed', message: '실행 실패가 공급됐습니다. · run_failed', diagnostic: table.diagnostic};
  if (table.presentation_state !== 'ready')
    return {kind: ['rejected', 'invalid', 'failed', 'error'].includes(table.presentation_state) ? 'rejected' : 'unsupported',
      message: `표 공급 상태: ${table.presentation_state ?? '미공급'} · 공급 원문 확인`,
      diagnostic: table.diagnostic};
  const malformed = !Array.isArray(table.tables) || table.tables.some(t => !object(t)
    || !Array.isArray(t.columns) || !t.columns.every(object) || !Array.isArray(t.rows)
    || t.rows.some(row => !object(row) || !Array.isArray(row.cells)
      || row.cells.length !== t.columns.length || !row.cells.every(object)
      || row.cells.some(cell => cell.status === 'present'
        && (!Object.hasOwn(cell, 'value_text') || !Object.hasOwn(cell, 'value_canon')))));
  if (malformed) return {kind: 'unsupported', message: '공급된 표의 열·행·셀 구조를 표시할 수 없습니다.'};
  if (table.tables.some(t => t.rows.some(row => row.cells.some(cell => !cellStates.has(cell.status)))))
    return {kind: 'unsupported', message: '지원하지 않는 관측 셀 상태가 있습니다. 원문을 확인하세요.'};
  return {kind: 'ready', message: '공급된 표 관측을 읽었습니다.',
    tables: table.tables};
}

export function mountSelectedTableView(root, {readState, subscribeState} = {}) {
  if (!root?.ownerDocument || typeof readState !== 'function'
    || (subscribeState !== undefined && typeof subscribeState !== 'function'))
    throw new TypeError('표 관측을 읽을 표시 영역과 상태 연결이 필요합니다.');
  const document = root.ownerDocument;
  function node(parent, tag, text) {
    const element = document.createElement(tag);
    if (text !== undefined) element.textContent = text;
    parent.append(element);
    return element;
  }
  const container = node(root, 'section');
  container.dataset.selectedTableView = 'true';
  container.style.minWidth = '0';
  container.style.maxWidth = '100%';
  node(container, 'h3', '공급된 표 관측');
  const button = node(container, 'button', '표 관측 다시 읽기');
  button.type = 'button';
  button.style.minHeight = '44px';
  const status = node(container, 'p');
  status.setAttribute('role', 'status');
  status.setAttribute('aria-live', 'polite');
  const content = node(container, 'div');
  let generation = 0, disposed = false, unsubscribe = null;
  const tableObservers = [];
  const sourceDisclosures = [];
  function retireSourceDisclosures() {
    for (const retire of sourceDisclosures) retire();
    sourceDisclosures.length = 0;
  }
  function stopTableObservers() {
    for (const observer of tableObservers) observer.disconnect();
    tableObservers.length = 0;
  }

  function details(parent, label, value, deferred = false) {
    const disclosure = node(parent, 'details');
    node(disclosure, 'summary', label).style.minHeight = '32px';
    const pre = node(disclosure, 'pre', deferred ? '' : rawText(value));
    pre.style.whiteSpace = 'pre-wrap';
    pre.style.overflowWrap = 'anywhere';
    if (deferred) {
      const request = generation;
      disclosure.dataset.deferredSource = 'true';
      const onToggle = () => {
        let owner = disclosure;
        while (owner && owner !== content) owner = owner.parentNode;
        if (disposed || request !== generation || owner !== content || !disclosure.open) {
          pre.textContent = '';
          return;
        }
        pre.textContent = rawText(value);
      };
      disclosure.addEventListener('toggle', onToggle);
      sourceDisclosures.push(() => {
        disclosure.removeEventListener('toggle', onToggle);
        pre.textContent = '';
        value = undefined;
      });
    }
    return disclosure;
  }
  function renderTable(parent, table, index) {
    const controls = node(parent, 'div');
    controls.hidden = true;
    node(controls, 'p', '표를 가로로 밀거나 아래 버튼으로 처음·마지막 열을 보세요.');
    const first = node(controls, 'button', '표 처음 열');
    const last = node(controls, 'button', '표 마지막 열');
    const scroll = node(parent, 'div');
    for (const button of [first, last]) {
      button.type = 'button';
      button.style.minHeight = '44px';
    }
    first.onclick = () => { scroll.scrollLeft = 0; };
    last.onclick = () => { scroll.scrollLeft = scroll.scrollWidth - scroll.clientWidth; };
    scroll.style.overflow = 'auto';
    scroll.tabIndex = 0;
    scroll.setAttribute('role', 'region');
    scroll.setAttribute('aria-label', `표 ${index + 1} · 가로 스크롤`);
    const element = node(scroll, 'table');
    element.style.width = '100%';
    element.style.borderCollapse = 'collapse';
    const rows = observationRowsForDisplay(table, 'history');
    const caption = node(element, 'caption', `표 ${index + 1} · 공급 순서 · ${rows.length}행`);
    caption.style.textAlign = 'left';
    const head = node(node(element, 'thead'), 'tr');
    const rowHeading = node(head, 'th', '관측 행');
    rowHeading.setAttribute('scope', 'col');
    rowHeading.style.minWidth = '9rem';
    for (const column of table.columns) {
      const th = node(head, 'th');
      th.setAttribute('scope', 'col');
      th.style.minWidth = '9rem';
      th.style.textAlign = 'left';
      th.style.padding = '0.5rem';
      node(th, 'span', column.label ?? column.identity?.name ?? '이름 미공급');
      node(th, 'p', `타입: ${column.dtype ?? '미공급'}`);
      node(th, 'p', column.unit === '' ? '단위: 미표기'
        : `단위: ${column.unit === undefined ? '미공급' : rawText(column.unit)}`);
      details(th, '공급된 열 정보', column);
    }
    const body = node(element, 'tbody');
    for (const row of rows) {
      const tr = node(body, 'tr'), rowHead = node(tr, 'th');
      rowHead.setAttribute('scope', 'row');
      rowHead.style.verticalAlign = 'top';
      rowHead.style.textAlign = 'left';
      rowHead.style.padding = '0.5rem';
      node(rowHead, 'span', `마디: ${row.committed_madi ?? '미공급'} · 표시 사건: ${row.display_event_index ?? '미공급'}`);
      const {cells, ...metadata} = row;
      details(rowHead, '공급된 행 출처', metadata);
      for (const [i, cell] of cells.entries()) {
        const td = node(tr, 'td');
        td.style.verticalAlign = 'top';
        td.style.padding = '0.5rem';
        td.style.borderTop = '1px solid currentColor';
        td.dataset.observationStatus = cell.status;
        node(td, 'p', cell.status === 'present' ? '화면 표시' : `관측 상태: ${cell.status}`);
        const displayed = observationCellDisplay(cell, table.columns[i]);
        node(td, 'span', typeof displayed === 'string' ? displayed : rawText(displayed));
        details(td, '공급된 셀 원문', cell);
      }
    }
    const ResizeObserver = document.defaultView?.ResizeObserver;
    if (ResizeObserver) {
      const observer = new ResizeObserver(() => {
        controls.hidden = scroll.scrollWidth <= scroll.clientWidth;
      });
      observer.observe(scroll);
      observer.observe(element);
      tableObservers.push(observer);
    } else {
      controls.hidden = false;
    }
    if (!rows.length) node(parent, 'p', '공급된 표에 표시할 행이 없습니다.');
  }
  async function refresh() {
    if (disposed) return null;
    const request = ++generation;
    stopTableObservers();
    retireSourceDisclosures();
    content.replaceChildren();
    container.dataset.selectedTableState = 'loading';
    container.setAttribute('aria-busy', 'true');
    status.textContent = '현재 표 관측을 읽는 중입니다. 이전 표는 폐기했습니다.';
    try {
      const state = await readState();
      if (disposed || request !== generation) return null;
      const result = inspect(state);
      const fragment = document.createElement('div');
      if (result.diagnostic !== undefined) details(fragment, '공급된 진단', result.diagnostic);
      if (result.candidate !== undefined) details(fragment, '보기·표 원자료와 출처', result.candidate, true);
      for (const view of result.views ?? []) {
        const scope = node(fragment, 'section');
        scope.dataset.selectedTableScope = view.scope;
        scope.dataset.selectedTableState = view.kind;
        if (view.index !== undefined) scope.dataset.suppliedRelationIndex = String(view.index);
        if (result.views.length > 1) node(scope, 'h4', view.label);
        node(scope, 'p', view.message);
        const run = view.view?.table_projection?.run;
        node(scope, 'p', `실행 공급 상태: ${run?.status ?? '미공급'}`);
        if (run?.failure != null) details(scope, '공급된 실행 실패', run.failure);
        if (view.relation !== undefined) details(scope, '공급된 관계 원문과 출처', view.relation, true);
        if (view.diagnostic !== undefined) details(scope, '공급된 진단', view.diagnostic);
        if (view.kind === 'ready') {
          if (Number.isSafeInteger(view.resolvedMadi) && view.resolvedMadi >= 0) {
            const policy = suppliedTimeLabel(view.timePolicy);
            const time = node(scope, 'p', '표 보기 마디 ' + view.resolvedMadi + (policy ? ' · ' + policy : ''));
            time.dataset.tableViewMadi = String(view.resolvedMadi);
          }
          view.tables.forEach((table, i) => renderTable(scope, table, i));
        }
      }
      if (disposed || request !== generation) return null;
      content.replaceChildren(fragment);
      container.dataset.selectedTableState = result.kind;
      status.textContent = result.message;
      return result.kind;
    } catch (error) {
      if (disposed || request !== generation) return null;
      stopTableObservers();
      retireSourceDisclosures();
      content.replaceChildren();
      container.dataset.selectedTableState = 'read_error';
      status.textContent = `표 관측을 읽거나 표시하지 못했습니다. ${error?.message ?? String(error)}`;
      return 'read_error';
    } finally {
      if (!disposed && request === generation) container.setAttribute('aria-busy', 'false');
    }
  }
  function dispose() {
    if (disposed) return;
    disposed = true;
    ++generation;
    stopTableObservers();
    retireSourceDisclosures();
    button.removeEventListener('click', onRead);
    try { unsubscribe?.(); } finally { container.remove(); }
  }
  function onRead() { void refresh(); }
  button.addEventListener('click', onRead);
  try {
    if (subscribeState) {
      unsubscribe = subscribeState(refresh);
      if (unsubscribe !== undefined && typeof unsubscribe !== 'function') {
        unsubscribe = null;
        throw new TypeError('표 관측 구독 해제 함수가 필요합니다.');
      }
    }
    void refresh();
  } catch (error) { dispose(); throw error; }
  return {refresh, dispose};
}
