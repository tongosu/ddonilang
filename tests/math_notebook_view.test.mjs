import test from 'node:test';
import assert from 'node:assert/strict';
import { createNotebookModel, validateNotebookView } from '../solutions/seamgrim_ui_mvp/ui/workbench/notebook/model.js';
import { notebookResultHtml, comparisonRanges, notebookViewport } from '../solutions/seamgrim_ui_mvp/ui/workbench/notebook/view.js';
import { buildGraphPreviewHtml } from '../solutions/seamgrim_ui_mvp/ui/components/graph_preview.js';
import { projectUniformMotion, createUniformMotionProvider } from '../solutions/seamgrim_ui_mvp/ui/workbench/notebook/uniform_motion.js';
import { readFileSync } from 'node:fs';

// Explicit view fixtures only, never DDN execution or producer evidence.
function view(values = [1, 4, 7]) {
  return { columns: [{ label: '시각', unit: 's' }, { label: '위치', unit: 'm' }],
    rows: values.map((x, i) => [String(i * 2), String(x)]),
    graph: { series: [{ id: '위치', points: values.map((y, i) => ({ x: i * 2, y })) }] },
    axisLabels: { x: '시각 (s)', y: '위치 (m)' }, positionAxis: '위치 (m)',
    positions: values.map((x, i) => ({ label: `${i * 2} s`, coordinate: x, exact: `${x} m` })) };
}

test('provider absence never produces data', async () => {
  const model = createNotebookModel({ readSource: () => 'raw' });
  await assert.rejects(model.calculate(), /연결되지/);
  assert.equal(model.snapshot().current, null);
});
test('source edit rejects a late response', async () => {
  let source = 'a', release;
  const model = createNotebookModel({ readSource: () => source,
    inspect: () => new Promise(resolve => { release = resolve; }), project: x => x });
  const pending = model.calculate();
  source = 'b'; model.invalidate(); release(view());
  await assert.rejects(pending, /원문이 바뀌어/);
  assert.equal(model.snapshot().current, null);
});
test('latest request keeps busy after an older response completes', async () => {
  let source = 'a'; const releases = [];
  const model = createNotebookModel({ readSource: () => source,
    inspect: () => new Promise(resolve => releases.push(resolve)), project: x => x });
  const old = model.calculate(); source = 'b'; model.invalidate(); const current = model.calculate();
  releases[0](view()); await assert.rejects(old);
  assert.equal(model.snapshot().busy, true);
  releases[1](view([1, 5, 9])); await current;
  assert.equal(model.snapshot().current.source, 'b');
});
test('two independent sources and results remain immutable in comparisons', async () => {
  let source = 'a'; const original = view();
  const model = createNotebookModel({ readSource: () => source, inspect: async () => original, project: x => x });
  await model.calculate(); model.pin();
  original.rows[0][1] = 'FORGED'; source = 'b'; model.invalidate();
  assert.equal(model.snapshot().comparisons[0].view.rows[0][1], '1');
  await model.calculate(); model.pin();
  assert.deepEqual(model.snapshot().comparisons.map(x => x.source), ['a', 'b']);
  assert.throws(() => model.pin(), /이미/);
  const snapshot = model.snapshot(); snapshot.comparisons[0].source = 'changed';
  assert.equal(model.snapshot().comparisons[0].source, 'a');
});
test('missing typed labels and numeric projections are rejected rather than guessed', () => {
  const noUnit = view(); delete noUnit.columns[0].unit;
  assert.throws(() => validateNotebookView(noUnit));
  const noAxis = view(); delete noAxis.axisLabels;
  assert.throws(() => validateNotebookView(noAxis));
  const invalid = view(); invalid.graph.series[0].points[0].x = null;
  assert.throws(() => validateNotebookView(invalid));
});
test('table graph position are rendering only and escape supplied labels', () => {
  const projection = view(); projection.columns[0].label = '<script>alert(1)</script>';
  const before = structuredClone(projection);
  assert.match(notebookResultHtml(projection), /&lt;script&gt;/);
  assert.match(notebookResultHtml(projection, 'graph'), /시각 \(s\)/);
  assert.match(notebookResultHtml(projection, 'position'), /7 m/);
  assert.deepEqual(projection, before);
});
test('reopen clears session observations and does not calculate automatically', async () => {
  let source = 'A', calls = 0;
  const model = createNotebookModel({ readSource: () => source,
    inspect: async () => { calls++; return {}; }, project: () => view() });
  await model.calculate(); model.pin();
  source = 'B'; await model.calculate(); model.pin();
  assert.equal(model.snapshot().comparisons.length, 2);
  model.reopened();
  assert.equal(model.snapshot().current, null);
  assert.deepEqual(model.snapshot().comparisons, []);
  assert.equal(model.snapshot().busy, false);
  assert.equal(calls, 2);
  await model.calculate(); assert.equal(calls, 3);
  assert.equal(model.snapshot().comparisons.length, 0);
});

test('two slopes use a common numeric scale, not separately fitted diagonals', () => {
  const a = view(), b = view([1, 5, 9]); const scale = comparisonRanges([a, b]);
  assert.deepEqual(scale.axis, { x_min: 0, x_max: 4, y_min: 1, y_max: 9 });
  assert.notEqual(notebookResultHtml(a, 'graph', scale), notebookResultHtml(b, 'graph', scale));
  assert.equal(scale.position.high, 9);
  assert.match(notebookResultHtml(a, 'graph', scale), />9<\/text>/);
});
test('mismatched axis labels never silently become a common-unit comparison', () => {
  const a = view(), b = view(); b.axisLabels.y = '위치 (km)';
  assert.match(notebookResultHtml(a, 'graph', comparisonRanges([a, b])), /표로 비교/);
});
test('graph renderer default path remains opt-in and source points unchanged', () => {
  const graph = view().graph, before = structuredClone(graph);
  assert.doesNotMatch(buildGraphPreviewHtml(graph), /notebook-axis-labels/);
  assert.match(buildGraphPreviewHtml(graph, { axisLabels: { x: 's', y: 'm' } }), /notebook-axis-labels/);
  assert.deepEqual(graph, before);
});

test('zoom and pan change display coordinates only', () => {
  const input = view(), original = structuredClone(input);
  const axis = comparisonRanges([input]).axis;
  const shifted = notebookViewport(axis, { zoom: 2, pan: 0.5 });
  assert.notDeepEqual(shifted, axis);
  assert.match(notebookResultHtml(input, 'graph', { zoom: 2, pan: 0.5 }), /clipPath/);
  assert.deepEqual(input, original);
  assert.equal(notebookViewport(axis, { zoom: NaN }), null);
});

test('actual supplied Rust/WASM observations project without evaluating the model',
  { skip: !process.env.DDN_NOTEBOOK_OBSERVATIONS }, () => {
    const supplied = JSON.parse(readFileSync(process.env.DDN_NOTEBOOK_OBSERVATIONS, 'utf8'));
    const reports = supplied.filter(row => row.report);
    assert.equal(reports.length, 6);
    for (const row of reports) {
      assert.equal(row.pass, true);
      const before = structuredClone(row.report);
      const output = projectUniformMotion(row.report);
      validateNotebookView(output);
      assert.deepEqual(row.report, before);
      if (row.name === '기본') {
        assert.deepEqual(output.rows, [['0', '1'], ['2', '4'], ['4', '7']]);
        assert.match(output.solutionMessage, /4 s.*7 m = 7 m/);
      }
      if (row.name === '속도2') assert.deepEqual(output.rows, [['0', '1'], ['2', '5'], ['4', '9']]);
      if (row.name === '정확분수') assert.match(output.solutionMessage, /1\/3 s/);
      const wrong = structuredClone(row.report); wrong.rows[0].position.dimension = [0];
      assert.throws(() => projectUniformMotion(wrong), /차원/);
    }
    const normal = projectUniformMotion(supplied.find(x => x.name === '기본').report);
    const changed = projectUniformMotion(supplied.find(x => x.name === '속도2').report);
    assert.deepEqual(normal.comparisonKey, changed.comparisonKey);
    assert.notDeepEqual(normal.comparedInput, changed.comparedInput);
  });

test('outside_domain shows actual substitution without admitting a nonnegative solution',
  { skip: !process.env.DDN_NOTEBOOK_OUTSIDE_REPORT }, () => {
    const report = JSON.parse(readFileSync(process.env.DDN_NOTEBOOK_OUTSIDE_REPORT, 'utf8'));
    const before = structuredClone(report);
    assert.equal(report.solution.kind, 'outside_domain');
    assert.equal(report.proof, false);
    assert.equal(report.solution.time, undefined);
    const view = projectUniformMotion(report);
    assert.match(view.solutionMessage, /-1\/2 s.*t>=0.*밖.*원식 검산 4 m = 4 m/);
    assert.doesNotMatch(view.solutionMessage, /도달 시간/);
    assert.deepEqual(report, before);
    const unequal = structuredClone(report); unequal.solution.substitution.equal = false;
    assert.match(projectUniformMotion(unequal).solutionMessage, /4 m ≠ 4 m/);
  });

test('outside_domain refuses missing or malformed substitution instead of inventing values',
  { skip: !process.env.DDN_NOTEBOOK_OUTSIDE_REPORT }, () => {
    const report = JSON.parse(readFileSync(process.env.DDN_NOTEBOOK_OUTSIDE_REPORT, 'utf8'));
    for (const part of ['left', 'right', 'equal']) {
      const missing = structuredClone(report); delete missing.solution.substitution[part];
      assert.throws(() => projectUniformMotion(missing));
    }
    const empty = structuredClone(report); delete empty.solution.substitution;
    assert.throws(() => projectUniformMotion(empty));
    const wrong = structuredClone(report); wrong.solution.substitution.left.dimension = [0];
    assert.throws(() => projectUniformMotion(wrong), /차원/);
    const fake = structuredClone(report); fake.solution.substitution.equal = 'true';
    assert.throws(() => projectUniformMotion(fake), /검산 결과 공급/);
  });

test('wrong root target and wrong source hash cannot publish an observation',
  { skip: !process.env.DDN_NOTEBOOK_ROOT_ARTIFACT || !process.env.DDN_NOTEBOOK_ROOT_RESULT }, async () => {
    const raw = readFileSync(process.env.DDN_NOTEBOOK_ROOT_ARTIFACT, 'utf8');
    const supplied = JSON.parse(readFileSync(process.env.DDN_NOTEBOOK_ROOT_RESULT, 'utf8'));
    let calls = 0, wrongTarget = true;
    const provider = createUniformMotionProvider({
      authoredProjectOperation: async () => '{}',
      projectGraph: async () => ({ ...supplied.graph, start_definition: wrongTarget ? '다른정의' : '매마디' }),
      observeMathNotebook: async () => { calls++; return { ...supplied.observation, source_sha256: '0'.repeat(64) }; },
    });
    await assert.rejects(provider.inspect(raw), /같은 매마디/);
    assert.equal(calls, 0);
    wrongTarget = false;
    await assert.rejects(provider.inspect(raw), /원문이 일치하지/);
    assert.equal(calls, 1);
  });
