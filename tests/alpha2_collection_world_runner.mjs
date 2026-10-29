import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

assert.ok(process.env.DDN_NATIVE_DIR);
assert.ok(process.env.DDN_WASM_DIR);
const sourcePath = path.resolve('solutions/seamgrim_ui_mvp/samples/alpha2_collection_world.ddn');
const source = fs.readFileSync(sourcePath, 'utf8');
assert.ok(!/격자\.|테트로미노\.|낙하조각\.|격자게임/.test(source));
const wasmDir = path.resolve(process.env.DDN_WASM_DIR);
const bytes = fs.readFileSync(path.join(wasmDir, 'ddonirang_tool_bg.wasm'));
const wasm = await import(pathToFileURL(path.join(wasmDir, 'ddonirang_tool.js')).href);
await wasm.default({module_or_path:bytes});
const summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, sourcePath, 1, 'v1-core-v25'));
// Independently derived: the square cannot cross x=0, moves right to x=1,
// and its top row reaches y=4 on a six-row board. Filling the two lower
// half-filled rows removes exactly two rows, leaving an entirely empty board.
const expected = ['참','1','4','5','3','2','0','0','0','참',
  '3','2','2','1','1','참','참','참','참','4',
  '200','2','1','200','50','1','참','참'];
const initial = [[0,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,7,7],[0,0,7,7]];
const square = [[0,0],[1,0],[0,1],[1,1]];
function check(result, label) {
  const values = result.resources.value_json;
  assert.deepEqual(values.output_log.map(row => row.text), expected, `${label}: independent outputs`);
  assert.deepEqual(values.시작판, initial, `${label}: original board unchanged`);
  assert.deepEqual(values.시작칸, square, `${label}: original piece unchanged`);
  assert.deepEqual(values.막힌이동, square, `${label}: collision does not mutate piece`);
  assert.deepEqual(values.떨어진칸, [[0,4],[1,4],[0,5],[1,5]], `${label}: downward convention`);
  assert.deepEqual(values.붙인판.slice(4), [[3,3,7,7],[3,3,7,7]], `${label}: attach before clearing`);
  assert.deepEqual(values.정리판, Array.from({length:6}, () => [0,0,0,0]), `${label}: two-row clear`);
  assert.deepEqual(values.다시시작판, initial, `${label}: initial-state restoration`);
  assert.deepEqual(values.허용회전, [[3,1],[2,1],[1,1],[1,2]], `${label}: allowed clockwise rotation`);
  assert.deepEqual(values.벽회전, [[2,1],[2,2],[2,3],[3,3]], `${label}: wall rotation rejected without movement`);
  assert.deepEqual(values.걸린회전, [[1,1],[1,2],[1,3],[2,3]], `${label}: occupied rotation rejected`);
  assert.deepEqual(values.빈판, Array.from({length:6}, () => [0,0,0,0]), `${label}: checks preserve empty board`);
  assert.equal(values.끝남, '참', `${label}: blocked spawn ends play`);
  assert.equal(values.계속가능, '참', `${label}: empty spawn permits play`);
  // This validates the seven-piece bag contract, not an invented RNG sequence.
  assert.deepEqual([...values.뽑은번호].sort((a,b) => a-b), [0,1,2,3,4,5,6], `${label}: exactly one of each candidate`);
  assert.deepEqual(values.미리볼번호, values.뽑은번호, `${label}: preview does not advance bag`);
  assert.equal(values.조각모음.length, 7, `${label}: seven authored coordinate groups`);
  for (const cells of values.조각모음) {
    assert.equal(cells.length, 4, `${label}: four cells per piece`);
    assert.equal(new Set(cells.map(cell => cell.join(','))).size, 4, `${label}: distinct cells`);
  }
  assert.deepEqual(values.첫조각, values.조각모음[values.뽑은번호[0]], `${label}: drawn ID selects authored piece`);
  const empty = Array.from({length:6}, () => [0,0,0,0]);
  assert.deepEqual(values.낙하뒤, [empty,square,[[200,0,2]]], `${label}: attach/clear/score/spawn transition`);
  assert.deepEqual(values.이동뒤, [empty,[[1,0],[2,0],[1,1],[2,1]],[[200,0,0]]], `${label}: next piece moves without resetting score`);
  assert.deepEqual(values.초기진행값, [initial,square,[[0,0,0]]], `${label}: previous state immutable`);
  const blockedBoard = empty.map(row => [...row]);
  blockedBoard[0][0] = 9;
  blockedBoard[4][2] = 3; blockedBoard[4][3] = 3;
  blockedBoard[5][2] = 3; blockedBoard[5][3] = 3;
  assert.deepEqual(values.종료뒤, [blockedBoard,square,[[50,1,0]]], `${label}: blocked next spawn ends without inventing cleared rows`);
  assert.deepEqual(values.종료유지, values.종료뒤, `${label}: finished game does not consume move`);
  assert.deepEqual(values.재시작뒤, values.초기진행값, `${label}: restart restores board/piece/score/end together`);
}
check(summary, 'actual WASM');
for (const alias of ['ddonilang','ddonirang','또니랑']) {
  const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
    ['실행',sourcePath,'--source-profile','v1-core-v25','--마디수','1','--json'],
    {encoding:'utf8',timeout:30000,maxBuffer:4*1024*1024});
  assert.equal(run.error, undefined, `${alias}: invocation`);
  assert.equal(run.status, 0, `${alias}: ${run.stderr}`);
  const native = JSON.parse(run.stdout);
  check(native, alias);
  assert.equal(native.state_hash, summary.state_hash, `${alias}: world hash`);
  assert.deepEqual(native.work_cost_receipt, summary.work_cost_receipt, `${alias}: work receipt`);
}
console.log(JSON.stringify({result:'ALPHA2_COLLECTION_WORLD_PASS',
  source_sha256:createHash('sha256').update(source).digest('hex'),
  wasm_sha256:createHash('sha256').update(bytes).digest('hex'),
  native_aliases:3, actual_wasm:1, output_checks:expected.length,
  state_hash:summary.state_hash, new_standard_functions:0,
  scope:'Authored move/drop/attach/clear/reset/rotation/bag/spawn-block and scored state transitions; interactive input, full playable Tetris and views remain incomplete'}));
