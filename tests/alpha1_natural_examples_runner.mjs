import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
assert.ok(process.env.DDN_NATIVE_DIR && process.env.DDN_WASM_DIR);
const wasm = await import(pathToFileURL(path.join(process.env.DDN_WASM_DIR, 'ddonirang_tool.js')).href);
await wasm.default({module_or_path: fs.readFileSync(path.join(process.env.DDN_WASM_DIR, 'ddonirang_tool_bg.wasm'))});
const menu = fs.readFileSync(path.join(root, 'solutions/seamgrim_ui_mvp/ui/workbench/workbench.js'), 'utf8');
const source = menu.slice(menu.indexOf('id: "local_linear_slope_v1"')).match(/fallback: `([\s\S]*?)`/)[1];
assert.ok(!source.includes('매마디') && !source.includes('{ 참 }이면'), '정적 일차함수에 가짜 실행 틀 금지');
assert.ok(!source.includes('실행:움직씨') && !source.includes('실행하기'), '그림 우회용 실행 감싸기 금지');
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ddn-natural-examples-'));
const file = path.join(dir, 'linear.ddn');
fs.writeFileSync(file, source);
const summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, file, 1, 'v1-core-v25'));
assert.deepEqual(summary.output_log_texts, ['일차함수 값', '2', '5']);
const drawing = summary.resources.value_json['보개_그림판_목록'];
assert.equal(drawing.length, 4, '직선·축·점이 실제 WASM에서 게시됨');
assert.equal(drawing[2].y1, -9, '왼쪽 끝 y=2*(-5)+1');
assert.equal(drawing[2].y2, 11, '오른쪽 끝 y=2*5+1');
for (const name of ['ddonilang', 'ddonirang', '또니랑']) {
  const result = spawnSync(path.join(process.env.DDN_NATIVE_DIR, name + '.exe'),
    ['실행', file, '--source-profile', 'v1-core-v25', '--마디수', '1', '--json'],
    {encoding:'utf8', windowsHide:true, timeout:20000});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).state_hash, summary.state_hash, name + ': CLI/WASM 세계 해시');
  assert.deepEqual(JSON.parse(result.stdout).resources.value_json['보개_그림판_목록'], drawing, name + ': 도형도 같음');
}
console.log('ALPHA1_NATURAL_EXAMPLES_PASS linear=one-shot cli_aliases=3 actual_wasm=1 state_hash=' + summary.state_hash);
const lesson = fs.readFileSync(path.join(root, 'solutions/seamgrim_ui_mvp/first_steps/예제/02A_두배정의.ddn'), 'utf8');
const prose = fs.readFileSync(path.join(root, 'solutions/seamgrim_ui_mvp/first_steps/README.md'), 'utf8');
const excerpt = prose.slice(prose.indexOf('### 조사형과 이름 인자 비교')).match(/```ddn\n([\s\S]*?)```/)[1];
const comparison = lesson.slice(0, lesson.indexOf('7의 두배')) + excerpt;
const compareFile = path.join(dir, 'comparison.ddn');
fs.writeFileSync(compareFile, comparison);
const compareSummary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(comparison, compareFile, 1, 'v1-core-v25'));
assert.deepEqual(compareSummary.output_log_texts, ['14', '14'], '본문 비교 원문 그대로 실행');
for (const name of ['ddonilang', 'ddonirang', '또니랑']) {
  const result = spawnSync(path.join(process.env.DDN_NATIVE_DIR, name + '.exe'),
    ['실행', compareFile, '--source-profile', 'v1-core-v25', '--마디수', '1', '--json'],
    {encoding:'utf8', windowsHide:true, timeout:20000});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).state_hash, compareSummary.state_hash, '본문 비교 CLI/WASM');
}
console.log('COURSE_CALL_COMPARISON_PASS outputs=14,14 cli_aliases=3 actual_wasm=1');
