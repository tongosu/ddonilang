import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';

// E0 evidence: authored DDN only, no harness lowering or new library callable.
assert.ok(process.env.DDN_NATIVE_DIR);
assert.ok(process.env.DDN_WASM_DIR);
const sourcePath = path.resolve('tests/fixtures/alpha2/collection_board_feasibility.ddn');
const source = fs.readFileSync(sourcePath, 'utf8');
assert.ok(!/격자\.|테트로미노\.|낙하조각\.|격자게임/.test(source));
const wasmDir = path.resolve(process.env.DDN_WASM_DIR);
const bytes = fs.readFileSync(path.join(wasmDir, 'ddonirang_tool_bg.wasm'));
const wasm = await import(pathToFileURL(path.join(wasmDir, 'ddonirang_tool.js')).href);
await wasm.default({module_or_path:bytes});
const summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, sourcePath, 1, 'v1-core-v25'));
// Hand derivation: translation +(2,1); clockwise turn (2-y,x);
// four turns return each original coordinate. Full rows 1 and 3 disappear,
// leaving two blank rows above [1,0,0,0] and [0,3,0,0].
const expected = ['2','3','2','0','1', '0','0','1','0','1','1','1','2',
  '거짓','참','참','5','0','참','0','0','1','3','4'];
const original = [[0,0],[1,0],[1,1],[1,2]];
const emptyBoard = Array.from({length:4}, () => [0,0,0,0]);
const clearedBoard = [[0,0,0,0],[0,0,0,0],[1,0,0,0],[0,3,0,0]];
function check(result, label) {
  const values = result.resources.value_json;
  assert.deepEqual(values.output_log.map(row => row.text), expected, `${label}: independent outputs`);
  assert.deepEqual(values.조각, original, `${label}: original piece unchanged`);
  assert.deepEqual(values.네번, original, `${label}: four rotations`);
  assert.deepEqual(values.판, emptyBoard, `${label}: persistent original board`);
  assert.deepEqual(values.정리판, clearedBoard, `${label}: stable row deletion and padding`);
}
check(summary, 'actual WASM');
for (const alias of ['ddonilang', 'ddonirang', '또니랑']) {
  const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
    ['실행',sourcePath,'--source-profile','v1-core-v25','--마디수','1','--json'],
    {encoding:'utf8',timeout:30000,maxBuffer:4*1024*1024});
  assert.equal(run.error, undefined, `${alias}: invocation`);
  assert.equal(run.status,0, `${alias}: ${run.stderr}`);
  const native = JSON.parse(run.stdout);
  check(native, alias);
  assert.equal(native.state_hash,summary.state_hash, `${alias}: world hash`);
  assert.deepEqual(native.work_cost_receipt,summary.work_cost_receipt, `${alias}: work receipt`);
}
console.log(JSON.stringify({result:'ALPHA2_COLLECTION_BOARD_FEASIBILITY_PASS',
  source_sha256:createHash('sha256').update(source).digest('hex'),
  wasm_sha256:createHash('sha256').update(bytes).digest('hex'),
  output_checks:expected.length, native_aliases:3, actual_wasm:1,
  state_hash:summary.state_hash, new_standard_functions:0,
  scope:'E0 collection substitution evidence; not interactive Tetris or Alpha2 completion'}));
