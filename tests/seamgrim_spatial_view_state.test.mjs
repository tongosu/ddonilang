import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createSpatialViewState, updateSpatialView, fitSpatialHistory, resetSpatialRange }
  from '../solutions/seamgrim_ui_mvp/ui/workbench/spatial_view_state.js';

const circle = (x, y = 0, r = 1) => ({ kind: 'circle', x, y, r });
function update(state, madi, drawlist, extra = {}) {
  return updateSpatialView(state, { runId: 'run1', sourceHash: 'source1',
    frame: { madi, state_hash: `h${madi}`, source_hash: 'source1' },
    space2d: { schema: 'seamgrim.space2d.v0', drawlist }, ...extra });
}

test('explicit camera remains authoritative until fit and is restored by reset', () => {
  const state = createSpatialViewState();
  const camera = {x_min: -20, x_max: 20, y_min: -10, y_max: 10};
  const expected = {xMin: -20, xMax: 20, yMin: -10, yMax: 10};
  const step = (madi, x) => update(state, madi, [], {space2d: {camera, drawlist: [circle(x)]}});
  assert.deepEqual(step(0, 0).viewState.range, expected);
  assert.deepEqual(step(1, 100).viewState.range, expected);
  assert.ok(fitSpatialHistory(state).viewState.range.xMax > 100);
  assert.deepEqual(resetSpatialRange(state).viewState.range, expected);
  assert.deepEqual(step(2, 200).viewState.range, expected);
  assert.equal(JSON.stringify(camera), '{"x_min":-20,"x_max":20,"y_min":-10,"y_max":10}');
});

test('incomplete, nonfinite and reversed cameras cannot replace valid automatic bounds', () => {
  for (const camera of [{x_min: 0}, {x_min: 0, x_max: Infinity, y_min: 0, y_max: 1},
    {x_min: 2, x_max: 1, y_min: 0, y_max: 1}, {x_min: 0, x_max: 1, y_min: null, y_max: 1}]) {
    const state = createSpatialViewState();
    const result = update(state, 0, [], {space2d: {camera, drawlist: [circle(10)]}});
    assert.ok(result.viewState.range.xMax > 11);
    assert.equal(state.mode, 'expand');
  }
});

test('first fit remains stable and expands only for bounds outside the view', () => {
  const state = createSpatialViewState();
  const first = update(state, 0, [circle(0)]).viewState.range;
  assert.deepEqual(update(state, 1, [circle(0.1)]).viewState.range, first);
  const expanded = update(state, 2, [circle(10)]).viewState.range;
  assert.ok(expanded.xMin <= first.xMin && expanded.xMax > 11);
  assert.deepEqual(update(state, 3, [circle(0)]).viewState.range, expanded);
  assert.deepEqual(resetSpatialRange(state).viewState.range, first);
  assert.deepEqual(update(state, 4, [circle(100)]).viewState.range, first);
  assert.ok(fitSpatialHistory(state).viewState.range.xMax > 101);
});

test('bounds use radius, line endpoints and rectangle extent, including negatives', () => {
  const state = createSpatialViewState();
  const range = update(state, 0, [circle(-5, 0, 3),
    { kind: 'line', x1: -20, y1: -30, x2: 4, y2: 2 },
    { kind: 'rect', x: 10, y: 10, w: 5, h: 6 }]).viewState.range;
  assert.ok(range.xMin < -20 && range.xMax > 15 && range.yMin < -30 && range.yMax > 16);
});

test('trails come only from consecutive committed positions, without source mutation', () => {
  const state = createSpatialViewState();
  const payload = { drawlist: [circle(0)], state_hash: 'unaltered' };
  const before = JSON.stringify(payload);
  updateSpatialView(state, { runId: 'run1', sourceHash: 'source1',
    frame: { madi: 0, state_hash: 'h0' }, space2d: payload });
  assert.equal(JSON.stringify(payload), before);
  assert.equal(update(state, 1, [circle(1)]).overlayDrawlist.length, 1);
  assert.equal(update(state, 1, [circle(1)]).overlayDrawlist.length, 1);
  assert.equal(update(state, 3, [circle(3)]).overlayDrawlist.length, 1);
  assert.equal(update(state, 4, [circle(4, 0, 2)]).overlayDrawlist.length, 1);
  assert.equal(update(state, 5, [circle(5, 0, 2), circle(9)]).overlayDrawlist.length, 1);
  assert.equal(update(state, 0, [circle(0)]).overlayDrawlist.length, 0);
});

test('run and source changes clear view cache; invalid identity cannot add motion', () => {
  const state = createSpatialViewState();
  update(state, 0, [circle(0)]);
  update(state, 1, [circle(1)]);
  update(state, 2, [circle(20)], { frame: { madi: 2, state_hash: 'h2', source_hash: 'other' } });
  assert.equal(state.frames.length, 2);
  assert.equal(update(state, 0, [circle(50)], { runId: 'run2' }).overlayDrawlist.length, 0);
  assert.equal(state.frames.length, 1);
  update(state, 0, [circle(80)], { sourceHash: 'source2', frame: { madi: 0, state_hash: 'new' } });
  assert.equal(state.frames.length, 1);
  assert.ok(state.range.xMin > 70);
});

test('history is bounded and explicit duplicate ids are not joined', () => {
  const state = createSpatialViewState({ maxFrames: 3 });
  for (let i = 0; i < 10; i++) update(state, i, [circle(i)]);
  assert.equal(state.frames.length, 3);
  assert.equal(update(state, 9, [circle(9)]).overlayDrawlist.length, 2);
  const duplicate = createSpatialViewState();
  update(duplicate, 0, [{ ...circle(0), id: 'a' }, { ...circle(10), id: 'a' }]);
  assert.equal(update(duplicate, 1, [{ ...circle(1), id: 'a' }]).overlayDrawlist.length, 0);
});

test('automatic expansion does not compound padding across a full curve', () => {
  const state = createSpatialViewState();
  for (let i = 0; i <= 1000; i++) {
    const frame = {madi:i, state_hash:`h${i}`, source_hash:'source1'};
    const space2d = {drawlist:[{kind:'point', x:i, y:1000-i}]};
    const before = JSON.stringify({frame, space2d});
    updateSpatialView(state, {runId:'run1', sourceHash:'source1', frame, space2d});
    assert.equal(JSON.stringify({frame, space2d}), before);
  }
  assert(state.range.xMin >= -100 && state.range.xMax <= 1100);
  assert(state.range.yMin >= -100 && state.range.yMax <= 1100);
  assert(state.range.xMax >= 1000 && state.range.yMin <= 0);
  assert.equal(state.frames.length, 512);
});
