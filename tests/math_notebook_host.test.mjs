import test from 'node:test';
import assert from 'node:assert/strict';
import { createEquationAuthoringModel, equationStepText, validateEquationObservation, equationSourceSlice } from '../solutions/seamgrim_ui_mvp/ui/workbench/notebook/model.js';
import { equationAuthoringHtml, equationObservationHtml } from '../solutions/seamgrim_ui_mvp/ui/workbench/notebook/view.js';
import { readFileSync } from 'node:fs';

const context = () => ({ projectId: 'project-a', path: 'main.ddn', generation: 1, sourceText: 'original source' });
const fields = () => ({ operation: 'subtract', reference: '근거', term: '1', left: '(3 / 2) * 미지수', right: '5', operationName: '빼기', claimName: '정리' });

test('authoring inserts exactly the typed claim without correcting its mathematics', () => {
  const text = equationStepText(fields());
  assert.equal(text, '빼기 := (근거=근거, 항=1) 양변빼기.\n정리 := (근거=빼기, 왼=(3 / 2) * 미지수, 오른=5) 등식정리.');
  assert.match(equationStepText({ ...fields(), operation: 'divide', term: '0' }), /계수=0\) 양변나누기/);
  assert.match(equationStepText({ ...fields(), resultPlace: true, claimName: '일차결론보이다' }), /일차결론보이다 <-/);
  assert.throws(() => equationStepText({ ...fields(), reference: '' }));
});

test('load edit and one guarded editor application never produce checker results', () => {
  let current = context(), edits = 0;
  const model = createEquationAuthoringModel({ readContext: () => current, applySource(expected, sourceText) {
    assert.deepEqual(expected, current); edits++;
    model.invalidate(); current = { ...current, generation: 2, sourceText }; return current;
  } });
  model.load(); model.edit('changed source');
  assert.equal(edits, 0);
  model.apply(); assert.equal(edits, 1);
  assert.equal(model.snapshot().resultStatus, 'NOT_PRODUCED');
  assert.equal(model.snapshot().dirty, false);
});

test('project path generation and exact source each guard against stale application', () => {
  for (const [key, value] of Object.entries({ projectId: 'other', path: 'other.ddn', generation: 2, sourceText: 'foreign source' })) {
    let current = context(), calls = 0;
    const model = createEquationAuthoringModel({ readContext: () => current, applySource() { calls++; } });
    model.load(); model.edit('candidate'); current = { ...current, [key]: value };
    assert.throws(() => model.apply(), /판본/); assert.equal(calls, 0);
    assert.equal(model.snapshot().draft, 'candidate');
  }
});

test('invalidation preserves uncommitted text but requires explicit reload', () => {
  const model = createEquationAuthoringModel({ readContext: context });
  model.load(); model.edit('uncommitted'); model.invalidate();
  assert.equal(model.snapshot().draft, 'uncommitted');
  assert.equal(model.snapshot().valid, false);
  assert.equal(model.snapshot().resultStatus, 'NOT_PRODUCED');
  assert.throws(() => model.insert(0, 0, fields()));
  model.load(); assert.equal(model.snapshot().draft, 'original source');
  assert.equal(model.snapshot().resultStatus, 'NOT_PRODUCED');
});

test('selection insertion is literal text and never invents a source span', () => {
  const model = createEquationAuthoringModel({ readContext: context });
  model.load(); model.insert(0, 8, fields());
  assert.equal(model.snapshot().draft, equationStepText(fields()) + ' source');
  assert.throws(() => model.insert(-1, 0, fields()));
  assert.equal(model.snapshot().resultStatus, 'NOT_PRODUCED');
});

test('pending producer screen has a disabled checker and no invented report', () => {
  const html = equationAuthoringHtml();
  assert.match(html, /data-equation="check" disabled/);
  assert.match(html, /NOT_PRODUCED/);
  assert.match(html, /전체 DDN 원문/);
  assert.doesNotMatch(html, /검사 성공|proof_handle|schema:/);
});

const supplied = process.env.DDN_V4_FIXED_SUPPLY;
function observed(name) {
  return {
    source: readFileSync(`${supplied}/parity/${name}.ddn`, 'utf8'),
    report: JSON.parse(readFileSync(`${supplied}/parity/${name}.json`, 'utf8')).observation,
  };
}

test('fixed producer normal and first-failure observations retain exact source and statuses', { skip: !supplied }, async () => {
  for (const name of ['normal', 'intermediate', 'zero', 'final_goal', 'crlf']) {
    const { source, report } = observed(name);
    const before = structuredClone(report);
    await validateEquationObservation(report, source, report.source_path);
    const html = equationObservationHtml(report, source);
    assert.deepEqual(report, before);
    if (name === 'normal') assert.match(html, /무가정 증명 발급 아님/);
    if (name === 'intermediate') {
      assert.deepEqual(report.definitions[0].steps.map(step => step.status), ['checked', 'failed', 'not_checked', 'not_checked']);
      assert.match(html, /첫 거부/); assert.match(html, /미검사/);
      assert.match(html, /오른쪽 항/);
    }
    if (name === 'final_goal') {
      assert.equal(report.status, 'rejected');
      assert.ok(report.definitions[0].steps.every(step => step.status === 'checked'));
      assert.match(html, /거부/); assert.doesNotMatch(html, /이번 가정 아래 수용/);
    }
  }
});

test('observation source path byte count SHA and invalid UTF8 span are fail-closed', { skip: !supplied }, async () => {
  const { source, report } = observed('normal');
  for (const patch of [{ source_path: 'other.ddn' }, { source_bytes: 0 }, { source_sha256: '0'.repeat(64) }]) {
    await assert.rejects(validateEquationObservation({ ...report, ...patch }, source, report.source_path));
  }
  assert.throws(() => equationSourceSlice('가', { start: 1, end: 3 }));
  assert.throws(() => equationSourceSlice(source, { start: 0, end: 999999 }));
  const forged = observed('intermediate');
  forged.report.definitions[0].steps[2].equality = report.definitions[0].steps[2].equality;
  await assert.rejects(validateEquationObservation(forged.report, forged.source, forged.report.source_path));
});

test('real opt-in call is explicit; edits and reload remove reports; late replies cannot publish', { skip: !supplied }, async () => {
  const { source, report } = observed('normal');
  let current = { ...context(), path: report.source_path, sourceText: source }, calls = 0, release;
  const model = createEquationAuthoringModel({ readContext: () => current, inspect: async (text, path, options) => {
    calls++; assert.equal(text, source); assert.equal(path, report.source_path); assert.deepEqual(options, { includeSteps: true });
    return structuredClone(report);
  } });
  model.load(); assert.equal(calls, 0); await model.check();
  assert.equal(model.snapshot().report.status, 'accepted');
  model.edit(source + '\n'); assert.equal(model.snapshot().report, null);
  model.load(); assert.equal(model.snapshot().resultStatus, 'NOT_PRODUCED'); assert.equal(calls, 1);
  const late = createEquationAuthoringModel({ readContext: () => current,
    inspect: () => new Promise(resolve => { release = resolve; }) });
  late.load(); const pending = late.check();
  current = { ...current, generation: 2 }; late.invalidate(); release(report);
  await assert.rejects(pending, /판본/);
  assert.equal(late.snapshot().report, null); assert.equal(late.snapshot().busy, false);
});
