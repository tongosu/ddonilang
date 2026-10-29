import assert from 'node:assert/strict';
import {diagnosticPresentation, appendDiagnosticPresentation, diagnosticSummary}
  from '../solutions/seamgrim_ui_mvp/ui/workbench/diagnostic_display.js';
import {createViewPresentation}
  from '../solutions/seamgrim_ui_mvp/ui/workbench/view_presentation.js';

class Node {
  constructor(doc) { this.ownerDocument = doc; this.children = []; this.dataset = {}; this.style = {}; this.attributes = {}; this.text = ''; }
  set textContent(value) { this.text = value; this.children = []; }
  get textContent() { return this.text + this.children.map(child => child.textContent).join(''); }
  set innerHTML(value) { assert.equal(value, ''); this.text = ''; this.children = []; }
  append(...children) { this.children.push(...children); }
  appendChild(child) { this.append(child); }
  setAttribute(key, value) { this.attributes[key] = value; }
  addEventListener() {}
}
const listeners = new Map();
const doc = {
  documentElement: {dataset: {displayLocale: 'ko'}},
  createElement() { return new Node(doc); },
  addEventListener(name, fn) { listeners.set(name, fn); },
};
globalThis.document = doc;
const original = 'E_DIV_ZERO: 0으로 나눌 수 없습니다.\r\n학습.ddn:2:9 <원문>';
const unknown = diagnosticPresentation('E_DIV_ZERO', original, 'ja');
assert.equal(unknown.original, original);
assert.equal(unknown.fallback, true);
assert.equal(diagnosticPresentation('__proto__', original, 'ja').fallback, true);
assert.match(diagnosticPresentation('E_PROJECT_RUNTIME_SOURCE_CHANGED', original, 'ja').guidance, /保存してから実行/);
assert.throws(() => diagnosticPresentation('E_DIV_ZERO', original, 'en'), RangeError);

const state = {diagnostics: [{code: 'E_PROJECT_RUN_FAILED', severity: 'error', message: original,
  range: {start: 21, end: 25}, at: 'fixed'}], sourceReadiness: null};
const before = JSON.stringify(state);
const els = {canvas: new Node(doc), diagnostics: new Node(doc), projectVerificationOpen: new Node(doc)};
const view = createViewPresentation({state, els, getSourceReadinessCandidate: () => null});
view.renderDiagnostics();
assert.equal(els.diagnostics.children[0].textContent, original);
assert.equal(els.projectVerificationOpen.textContent, '검증 (1)');
doc.documentElement.dataset.displayLocale = 'ja';
listeners.get('ddn-display-locale-change')();
const row = els.diagnostics.children[0];
assert.match(row.textContent, /作品を実行できませんでした/);
assert.equal(row.children.find(child => child.dataset.testid === 'diagnostic-original').textContent, original);
assert.equal(row.dataset.diagnosticCode, 'E_PROJECT_RUN_FAILED');
assert.equal(els.projectVerificationOpen.textContent, '検証 (1)');
assert.equal(JSON.stringify(state), before, 'UI locale must not rewrite original diagnostics or byte range');

const missingRow = new Node(doc);
appendDiagnosticPresentation(missingRow, {code: 'E_DIV_ZERO'}, original, 'ja');
assert.equal(missingRow.textContent, '[ko] ' + original);
assert.equal(missingRow.dataset.displayFallback, 'ko');
assert.equal(missingRow.children[1].children.length, 0, 'raw source remains text, never HTML');
doc.documentElement.dataset.displayLocale = 'ko';
listeners.get('ddn-display-locale-change')();
assert.equal(els.diagnostics.children[0].textContent, original);
assert.equal(JSON.stringify(state), before);

state.diagnostics = [];
doc.documentElement.dataset.displayLocale = 'ja';
view.renderDiagnostics();
assert.equal(els.diagnostics.children[0].textContent, diagnosticSummary('ja', 0, 0).empty);
assert.equal(els.projectVerificationOpen.attributes['aria-label'], 'エラーと診断を開く、診断なし');
console.log(JSON.stringify({status: 'PASS', cases: ['known-guidance-keeps-original', 'unknown-visible-ko-fallback',
  'prototype-and-unsupported-locale', 'actual-renderer-locale-event', 'state-and-range-unchanged',
  'ko-ja-ko-roundtrip', 'raw-markup-is-text', 'empty-and-count-labels']}));
