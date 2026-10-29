import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const wasmDir=process.env.DDN_WASM_DIR;
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const source=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn','utf8');
const publishedBinding=JSON.parse(fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json','utf8'));
assert.equal(publishedBinding.source_revision,`sha256:${createHash('sha256').update(source).digest('hex')}`,'the shipped CLI binding must name the shipped original bytes');
const empty=new wasm.DdnWasmVm(''); const initialWorld=empty.get_state_hash(); empty.free();
const channels=Object.fromEntries(['왼쪽','오른쪽','아래','회전','떨어뜨리기','다시시작'].map(name=>[name,'Control']));
const binding={source_profile:'v1-core-v25-logical-alpha2',input_schema:'alpha2.tetris.controls@1',
 channel_registry:'alpha2.tetris.channels@1',source_revision:`sha256:${createHash('sha256').update(source).digest('hex')}`,
 initial_world:initialWorld,initial_rng:7,recording_policy:'alpha2.tetris.recording@1',integrity_policy:'ddn.logical_input_tape.v1',
 madi_hz:60,dt_raw:4294967296,channels,initial_held:Object.fromEntries(Object.keys(channels).map(name=>[name,false]))};
const vm=new wasm.DdnLogicalInputVm(source,JSON.stringify(binding));
const previousWasmDir=process.env.DDN_TETRIS_PREDECESSOR_WASM_DIR;
let previousRuntime=null;
const runtimeTiming=[];
if(previousWasmDir) {
 const previousModule=await import(pathToFileURL(path.join(previousWasmDir,'ddonirang_tool.js')).href);
 await previousModule.default({module_or_path:fs.readFileSync(path.join(previousWasmDir,'ddonirang_tool_bg.wasm'))});
 previousRuntime=new previousModule.DdnLogicalInputVm(source,JSON.stringify(binding));
}
// Optional fixed predecessor executes its own unmodified DDN source. This is
// an algorithm regression comparison, not source/receipt identity equivalence.
const predecessorCommit=process.env.DDN_TETRIS_PREDECESSOR_COMMIT;
let predecessor=null;
if(predecessorCommit) {
 const read=spawnSync('git',['--no-optional-locks','show',`${predecessorCommit}:solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn`],{encoding:'utf8'});
 assert.equal(read.status,0,read.stderr);
 predecessor=new wasm.DdnLogicalInputVm(read.stdout,JSON.stringify({...binding,
  source_revision:`sha256:${createHash('sha256').update(read.stdout).digest('hex')}`}));
}
let madi=0; const held={...binding.initial_held}; const frames=[]; const dropTiming=[]; const stepTiming=[];
function step(actions=[]) {
 if (process.env.DDN_TEST_VERBOSE) console.log(`TETRIS_STEP_BEGIN ${madi} ${actions.join(',')}`);
 const events=[];
 for(const channel of actions) {
   events.push({ordinal:events.length,channel,event:{kind:'Press'}}); held[channel]=true;
   events.push({ordinal:events.length,channel,event:{kind:'Release'}}); held[channel]=false;
 }
 let result;
 const started=performance.now();
 try {result=JSON.parse(vm.step_json(JSON.stringify({madi:madi++,events,held})));}
 catch(error) {throw new Error(`madi=${madi-1}, actions=${actions.join(',')}, last=${JSON.stringify(frames.at(-1)?.resources.fixed64)}: ${String(error)}`);}
 stepTiming.push({madi:madi-1,actions:[...actions],elapsed_ms:performance.now()-started});
 if(previousRuntime) {
  const previousStarted=performance.now();
  const previous=JSON.parse(previousRuntime.step_json(JSON.stringify({madi:madi-1,events,held})));
  runtimeTiming.push({madi:madi-1,current_ms:stepTiming.at(-1).elapsed_ms,previous_ms:performance.now()-previousStarted});
  assert.deepEqual(result,previous,`runtime output/receipt/projection mismatch at madi ${madi-1}`);
 }
 if(predecessor) {
  const currentMs=performance.now()-started;
  const previousStarted=performance.now();
  const previous=JSON.parse(predecessor.step_json(JSON.stringify({madi:madi-1,events,held})));
  const previousMs=performance.now()-previousStarted;
  assert.deepEqual(result.resources,previous.resources,`fixed predecessor resource mismatch at madi ${madi-1}`);
  if(actions.includes('떨어뜨리기')) dropTiming.push({madi:madi-1,current_ms:currentMs,predecessor_ms:previousMs});
 }
 frames.push(result); return result.resources;
}
const first=step();
const fixed=resources=>resources.fixed64;
assert.equal(fixed(first).가로,'3'); assert.equal(fixed(first).세로,'0');
assert.equal(fixed(first).점수,'0'); assert.equal(fixed(first).끝남값,'0');
assert.equal(first.value_json.판.length,20); assert.equal(first.value_json.판[0].length,10);
const initialProjection=frames[0].bogae_projection;
assert.equal(initialProjection.status,'ready');
const artifact=initialProjection.candidates[0].artifact;
assert.equal(artifact.view.elements.length,5,'board outline and four authored piece cells');
assert.deepEqual(artifact.relations.map(relation=>relation.kind),['primary','overlay','side','side']);
assert.equal(artifact.relations[2].view.kind,'result_table');
assert.equal(artifact.relations[3].view.elements.length,5,'next piece has its own authored scene');
const raw=value=>String(BigInt(value)*4294967296n);
assert.equal(artifact.relations.some(relation=>relation.kind==='magnifier'),false,'game omits magnifier; generic feature remains covered by menu3');
assert.deepEqual(artifact.view.elements[1].vertices,first.value_json.현재칸.slice(0,1).flatMap(([x,y])=>[
 [raw(x),raw(19-y)],[raw(x+1),raw(19-y)],[raw(x+1),raw(20-y)],[raw(x),raw(20-y)]
]),'screen coordinates come from the authored row-to-plane conversion');
const piece=fixed(first).조각번호;
assert.equal(fixed(step(['왼쪽'])).가로,'2');
assert.equal(fixed(step(['오른쪽','회전'])).가로,'3');
assert.notDeepEqual(frames.at(-1).resources.value_json.지역칸,first.value_json.지역칸,'rotation must alter the authored cell coordinates');
assert.equal(fixed(step(['아래'])).세로,'1');
for(let n=0;n<20;n++) step(['왼쪽']);
assert.equal(fixed(frames.at(-1).resources).가로,'-1','rotated T has min local x1, so its anchor stops at -1');
const beforeDrop=frames.at(-1).resources;
const dropBoard=beforeDrop.value_json.판.map(row=>[...row]);
const dropCells=beforeDrop.value_json.현재칸;
let expectedDistance=0;
while(dropCells.every(([x,y])=>x>=0 && x<10 && y+expectedDistance+1>=0 && y+expectedDistance+1<20 && dropBoard[y+expectedDistance+1][x]===0)) expectedDistance++;
for(const [x,y] of dropCells) dropBoard[y+expectedDistance][x]=Number(beforeDrop.fixed64.조각번호)+1;
const landed=step(['떨어뜨리기']);
assert.deepEqual(landed.value_json.판,dropBoard,'independently computed lowest noncolliding cells');
assert.equal(landed.value_json.판.flat().filter(cell=>cell!==0).length,4,'four actual fixed cells, not JS game state');
assert.equal(frames.at(-1).bogae_projection.candidates[0].artifact.view.elements.length,9);
assert.deepEqual(frames.at(-1).bogae_projection.candidates[0].artifact.identity,artifact.identity,'dynamic scene geometry does not replace authored identity');
assert.equal(fixed(landed).세로,'0','hard drop fixes old piece and spawns next');
assert.notEqual(fixed(landed).조각번호,piece,'7-bag must not repeat before exhaustion');
const restarted=step(['다시시작']);
assert.equal(fixed(restarted).점수,'0'); assert.equal(fixed(restarted).조각번호,piece);
assert.equal(fixed(restarted).가로,'3'); assert.equal(fixed(restarted).세로,'0');
for(let n=0;n<29;n++) step();
assert.equal(fixed(frames.at(-1).resources).세로,'1','30 deterministic madis, no wallclock');
const record=JSON.parse(vm.seal_json()); vm.free();
if(previousRuntime) {
 assert.deepEqual(record,JSON.parse(previousRuntime.seal_json()),'runtime optimization must preserve exact tape bytes and receipt');
 previousRuntime.free();
 console.log(JSON.stringify({comparison:'FIXED_RUNTIME_PREDECESSOR',full_output_madis:frames.length,exact_tape_and_receipt:true,timing_observation:runtimeTiming}));
}
if(predecessor) predecessor.free();
if(predecessorCommit) console.log(JSON.stringify({comparison:'AUTHORED_TETRIS_PREDECESSOR',predecessor:predecessorCommit,resource_madis:frames.length,drop_timing_observation:dropTiming,receipt_identity_claim:false}));
console.log('TETRIS_LIVE_PASS: replaying the actual sealed file');
assert.deepEqual(JSON.parse(wasm.logical_input_replay(source,Uint8Array.from(record.tape_bytes),JSON.stringify(binding))),record.summary);
const output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-tetris-'));
fs.writeFileSync(path.join(output,'source.ddn'),source);fs.writeFileSync(path.join(output,'binding.json'),JSON.stringify(binding));
fs.writeFileSync(path.join(output,'record.tape'),Uint8Array.from(record.tape_bytes));
for(const alias of ['ddonilang','ddonirang','또니랑']) {
 console.log(`TETRIS_NATIVE_REPLAY ${alias}`);
 const result=spawnSync(path.join(process.env.DDN_NATIVE_DIR,`${alias}.exe`),['논리입력재생',path.join(output,'source.ddn'),path.join(output,'record.tape'),path.join(output,'binding.json')],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});
 assert.equal(result.status,0,result.stderr); assert.deepEqual(JSON.parse(result.stdout),record.summary);
}
// An ordinary source-data edit sets seven fixed cells on the last row. With
// seed 0, the first T piece fills x=3,4,5 there: independently expected 1 row,
// score100, and its upper central cell becomes the new last-row cell.
const exerciseSource=source.replace('판 <- 빈판. 점수 <- 0.', '판 <- 빈판. 판[19] <- 차림[1,1,1,0,0,0,1,1,1,1]. 점수 <- 0.');
const exerciseBinding={...binding,source_revision:`sha256:${createHash('sha256').update(exerciseSource).digest('hex')}`};
const exercise=new wasm.DdnLogicalInputVm(exerciseSource,JSON.stringify(exerciseBinding));
exercise.step_json(JSON.stringify({madi:0,events:[],held}));
const clear=JSON.parse(exercise.step_json(JSON.stringify({madi:1,events:[
 {ordinal:0,channel:'떨어뜨리기',event:{kind:'Press'}},{ordinal:1,channel:'떨어뜨리기',event:{kind:'Release'}}],held})));
assert.equal(clear.resources.fixed64.지운줄,'1'); assert.equal(clear.resources.fixed64.점수,'100');
assert.equal(clear.resources.value_json.판[19].filter(cell=>cell!==0).length,1);
exercise.free();
const ending=new wasm.DdnLogicalInputVm(source,JSON.stringify(binding));
let ended=null;
for(let n=0;n<25;n++) {
 ended=JSON.parse(ending.step_json(JSON.stringify({madi:n,events:[
  {ordinal:0,channel:'떨어뜨리기',event:{kind:'Press'}},{ordinal:1,channel:'떨어뜨리기',event:{kind:'Release'}}],held})));
 if(ended.resources.fixed64.끝남값==='1') break;
}
assert.equal(ended.resources.fixed64.끝남값,'1','stacking at the spawn position eventually ends the game');
ending.free();
console.log(JSON.stringify({result:'ALPHA2_LOGICAL_TETRIS_PASS',madi_count:madi,native_aliases:3,actual_wasm:true,evidence_directory:output,
 wasm_sha256:createHash('sha256').update(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))).digest('hex'),
 step_timing_observation:stepTiming,performance_claim:false,
 line_clear_score:100,game_over_checked:true,
 scope:'authored movement/rotation/drop/collision/7-bag/restart/gravity and file replay; views/reaction/full Alpha2 incomplete'}));
