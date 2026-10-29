import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {performance} from 'node:perf_hooks';
const source=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn','utf8');
const sources=[source, '// 위치 🧪\r\n(매마디)마다 {\n 값 <- 차림[1][2].\r\n}.',
 '(값:수) 계산:수:셈씨 := { 임시 <- 값. 임시 <- 임시 + 2. 임시 <- 임시 / 0. 계산 <- 임시. }. (매마디)마다 { 답 <- (1) 계산. }.',
 '(값:수) 계산:수:셈씨 := { 임시 <- 값. 임시 <- 임시 + 2. 계산 <- 임시. }. (매마디)마다 { 답 <- (1) 계산. }.'];
const channels=Object.fromEntries(['왼쪽','오른쪽','아래','회전','떨어뜨리기','다시시작'].map(name=>[name,'Control']));
const held=Object.fromEntries(Object.keys(channels).map(name=>[name,false]));
async function run(dir) {
 const wasm=await import(pathToFileURL(path.join(dir,'ddonirang_tool.js')).href);
 await wasm.default({module_or_path:fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'))});
 const empty=new wasm.DdnWasmVm(''); const initialWorld=empty.get_state_hash();empty.free();
 const outcomes=[];
 for(const text of sources) {
  const binding={source_profile:'v1-core-v25-logical-alpha2',input_schema:'alpha2.tetris.controls@1',
  channel_registry:'alpha2.tetris.channels@1',source_revision:`sha256:${createHash('sha256').update(text).digest('hex')}`,
  initial_world:initialWorld,initial_rng:7,recording_policy:'alpha2.tetris.recording@1',integrity_policy:'ddn.logical_input_tape.v1',
  madi_hz:60,dt_raw:4294967296,channels,initial_held:held};
  const vm=new wasm.DdnLogicalInputVm(text,JSON.stringify(binding)); const rows=[],times=[];let error=null;
  for(let madi=0;madi<12;madi++) {
   const actions=madi===2?['왼쪽']:madi===4?['회전']:madi===6?['떨어뜨리기']:madi===8?['다시시작']:[];
   const events=actions.flatMap((channel,n)=>[{ordinal:n*2,channel,event:{kind:'Press'}},{ordinal:n*2+1,channel,event:{kind:'Release'}}]);
   const start=performance.now();
   try { rows.push({row:JSON.parse(vm.step_json(JSON.stringify({madi,events,held}))),view:JSON.parse(vm.view_json())}); }
   catch(e) {error=String(e);console.error('RUNTIME_PARITY_DIAGNOSTIC',JSON.stringify({directory:dir,madi,error,stack:e?.stack}));break;}
   times.push(performance.now()-start);
  }
  const summary=JSON.parse(vm.summary_json());
  let seal=null;if(!error) seal=JSON.parse(vm.seal_json());
  vm.free();outcomes.push({rows,summary,seal,error,times});
 }
 return outcomes;
}
const repetitions=Number(process.env.DDN_RUNTIME_PARITY_RUNS ?? 1);
assert.ok(Number.isSafeInteger(repetitions) && repetitions>=1 && repetitions<=10);
const runs=[];
let baselineRows, baselineSeal;
for(let repetition=0;repetition<repetitions;repetition++) {
 const before=await run(process.env.DDN_PREDECESSOR_WASM_DIR),after=await run(process.env.DDN_WASM_DIR);
 for(const outcome of [before[0],after[0]]) {
  assert.equal(outcome.error,null,'normal Tetris must succeed, not merely fail equally');
  assert.equal(outcome.rows.length,12,'all requested madis must execute');
  assert.notEqual(outcome.seal,null,'normal run must produce an actual sealed tape');
 }
 for(let n=0;n<sources.length;n++) {
  assert.deepEqual(after[n].rows,before[n].rows,'same source/input must preserve complete world/input receipts and authored views');
  assert.deepEqual(after[n].summary,before[n].summary);assert.deepEqual(after[n].seal,before[n].seal);
  assert.equal(after[n].error,before[n].error,'Unicode/CRLF error position/message must remain exact');
 }
 assert.ok(after[1].error?.includes('CHARIM_INDEX_OUT_OF_RANGE'));
 assert.ok(after[2].error,'pure callable arithmetic failure must not become successful');
 assert.equal(after[2].rows.length,0,'failed madi must not publish a world row');
 assert.equal(after[3].error,null,'pure callable scalar assignment must execute');
 assert.equal(after[3].rows.length,12);
 if(repetition===0) {baselineRows=after[0].rows;baselineSeal=after[0].seal;}
 else {
  assert.deepEqual(after[0].rows,baselineRows,'same input must remain deterministic between repetitions');
  assert.deepEqual(after[0].seal,baselineSeal);
 }
 runs.push({before_step_ms:before[0].times,after_step_ms:after[0].times});
}
const wasmSha=dir=>createHash('sha256').update(fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'))).digest('hex');
console.log(JSON.stringify({result:'ALPHA2_RUNTIME_TETRIS_PARITY_PASS',actual_wasm:true,madis:runs[0].after_step_ms.length,
 whole_world_and_input_and_tape_equal:true,error_message_and_position_equal:true,
 before_wasm_sha256:wasmSha(process.env.DDN_PREDECESSOR_WASM_DIR),after_wasm_sha256:wasmSha(process.env.DDN_WASM_DIR),
 source_sha256:createHash('sha256').update(source).digest('hex'),repetitions,runs,
 scope:'Exact actual-engine equality; timing observational, full Alpha2/performance target not claimed'}));
