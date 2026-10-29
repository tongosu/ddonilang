import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';

assert.ok(process.env.DDN_NATIVE_DIR && process.env.DDN_WASM_DIR);
const wasm = await import(pathToFileURL(path.join(process.env.DDN_WASM_DIR, 'ddonirang_tool.js')).href);
await wasm.default({module_or_path: fs.readFileSync(path.join(process.env.DDN_WASM_DIR, 'ddonirang_tool_bg.wasm'))});
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ddn-root-drawing-'));
const cases = [
  ['root', 'x <- 2. 모양 { 원(x, 0, r=1). }. x 보여주기.', 2],
  ['ordered', 'x <- 2. 모양 { 원(x, 0, r=1). }. x <- 3. x 보여주기.', 2],
  ['last-block', '모양 { 원(1, 0, r=1). }. 모양 { 원(3, 0, r=1). }.', 3],
  ['body', '실행:움직씨 := { 모양 { 원(2, 0, r=1). }. }. () 실행하기.', 2],
  ['bogae', '보개 { 원(2, 0, r=1). }.', 2],
  ['config', '설정 { 화면: "기본". }. 슬기 { 설명: "검사". }. 모양 { 원(2, 0, r=1). }.', 2],
];
for (const [id, source, x] of cases) {
  const file = path.join(dir, id + '.ddn');
  fs.writeFileSync(file, source);
  const summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, file, 1, 'v1-core-v25'));
  assert.ok(!summary.error, JSON.stringify(summary));
  const drawing = summary.resources.value_json['보개_그림판_목록'];
  assert.equal(drawing.length, 1, id);
  assert.equal(drawing[0].x, x, id + ': source order');
  for (const name of ['ddonilang', 'ddonirang', '또니랑']) {
    const result = spawnSync(path.join(process.env.DDN_NATIVE_DIR, name + '.exe'),
      ['실행', file, '--source-profile', 'v1-core-v25', '--마디수', '1', '--json'],
      {encoding: 'utf8', windowsHide: true, timeout: 30000});
    assert.ifError(result.error);
    assert.equal(result.status, 0, result.stderr);
    const native = JSON.parse(result.stdout);
    assert.equal(native.state_hash, summary.state_hash, id + '/' + name);
    assert.deepEqual(native.resources.value_json['보개_그림판_목록'], drawing);
  }
}
console.log('TOP_LEVEL_SHAPE_PASS cases=' + cases.length + ' cli_aliases=3 actual_wasm=1');
