import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';

const wasmDir=process.env.DDN_WASM_DIR;
const nativeDir=process.env.DDN_NATIVE_DIR;
assert.ok(wasmDir && nativeDir,'exact WASM and Native artifact directories required');
const predecessorCommit=process.env.DDN_TETRIS_PREDECESSOR_COMMIT;
assert.ok(predecessorCommit,'fixed predecessor required');
const read=spawnSync('git',['--no-optional-locks','show',`${predecessorCommit}:solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn`],{encoding:'utf8'});
assert.equal(read.status,0,read.stderr);
const current=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn','utf8');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const empty=new wasm.DdnWasmVm('');const initialWorld=empty.get_state_hash();empty.free();
const sha=text=>`sha256:${createHash('sha256').update(text).digest('hex')}`;
const channels=Object.fromEntries(['왼쪽','오른쪽','아래','회전','떨어뜨리기','다시시작'].map(name=>[name,'Control']));
const held=Object.fromEntries(Object.keys(channels).map(name=>[name,false]));
const frame={madi:0,events:[],held};
const bindingFor=source=>({source_profile:'v1-core-v25-logical-alpha2',input_schema:'alpha2.tetris.controls@1',
 channel_registry:'alpha2.tetris.channels@1',source_revision:sha(source),initial_world:initialWorld,initial_rng:7,
 recording_policy:'alpha2.tetris.recording@1',integrity_policy:'ddn.logical_input_tape.v1',madi_hz:60,dt_raw:4294967296,channels,initial_held:held});
const ddn=value=>Array.isArray(value)?`차림[${value.map(ddn).join(',')}]`:String(value);
const zeroBoard=()=>Array.from({length:20},()=>Array(10).fill(0));
const shapes=[[[3,0],[4,0],[3,1],[4,1]],[[3,0],[3,1],[3,2],[3,3]],
 [[3,0],[4,0],[4,1],[5,1]],[[4,0],[3,1],[4,1],[5,1]]];
const cases=[];
for(const [shapeIndex,cells] of shapes.entries()) {
 for(let scene=0;scene<3;scene++) {
  const board=zeroBoard();
  if(scene===1) {board[15][4]=7;board[18][3]=2;}
  if(scene===2) {board[3][3]=1;board[3][4]=1;board[3][5]=1;}
  cases.push({name:`shape${shapeIndex}-scene${scene}`,board,cells,width:10,height:20});
 }
}
// The fallback must preserve old behavior/errors, not silently repair input.
cases.push({name:'ragged-short-row',board:[[0,0],[0]],cells:[[1,0]],width:2,height:2});
cases.push({name:'short-cell',board:[[0,0],[0,0]],cells:[[0]],width:2,height:2});
cases.push({name:'negative-x',board:[[0,0],[0,0]],cells:[[-1,0]],width:2,height:2});
cases.push({name:'fractional-x',board:[[0,0],[0,0]],cells:[[0.5,0]],width:2,height:2});
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-drop-distance-'));
const results=[];
for(const test of cases) {
 // This is an ordinary DDN fixture calling the unchanged public helper. No
 // AST lowering or host-side game implementation replaces the authored code.
 const call=`\n(매마디)마다 { 착지시험 <- (판=${ddn(test.board)}, 칸들=${ddn(test.cells)}, 폭=${test.width}, 높이=${test.height}) 바닥까지. }.\n`;
 const outputs=[];
 for(const [index,base] of [read.stdout,current].entries()) {
  const source=base+call;const binding=bindingFor(source);
  const vm=new wasm.DdnLogicalInputVm(source,JSON.stringify(binding));
  let row=null,error=null;
  try {row=JSON.parse(vm.step_json(JSON.stringify(frame)));} catch(e) {error=String(e);}
  if(error) {
   assert.equal(JSON.parse(vm.summary_json()).madi_count,0,'failed fixture must not commit a madi');
  } else if(index===1) {
   const record=JSON.parse(vm.seal_json());
   const folder=path.join(evidence,test.name);fs.mkdirSync(folder);
   fs.writeFileSync(path.join(folder,'source.ddn'),source);
   fs.writeFileSync(path.join(folder,'binding.json'),JSON.stringify(binding));
   fs.writeFileSync(path.join(folder,'record.tape'),Uint8Array.from(record.tape_bytes));
   for(const alias of ['ddonilang','ddonirang','또니랑']) {
    const native=spawnSync(path.join(nativeDir,`${alias}.exe`),['논리입력재생',path.join(folder,'source.ddn'),path.join(folder,'record.tape'),path.join(folder,'binding.json')],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});
    assert.equal(native.status,0,native.stderr);assert.deepEqual(JSON.parse(native.stdout),record.summary);
   }
  }
  outputs.push({value:row?.resources.value_json.착지시험,error});vm.free();
 }
 assert.deepEqual(outputs[1],outputs[0],`${test.name}: fixed predecessor result/error`);
 if(test.name.startsWith('shape')) {
  assert.equal(outputs[1].error,null);
  // Independent geometric oracle: test every whole-piece candidate against
  // the board until the first occupied/out-of-bounds position.
  let distance=0;
  while(test.cells.every(([x,y])=>y+distance+1<test.height && test.board[y+distance+1][x]===0)) distance++;
  assert.deepEqual(outputs[1].value,test.cells.map(([x,y])=>[x,y+distance]),`${test.name}: geometric oracle`);
 }
 results.push({name:test.name,...outputs[1]});
}
fs.writeFileSync(path.join(evidence,'results.json'),JSON.stringify(results,null,2));
console.log(JSON.stringify({result:'ALPHA2_DROP_DISTANCE_PASS',cases:cases.length,native_aliases:3,
 actual_wasm:true,predecessor:predecessorCommit,evidence_directory:evidence,
 source_sha256:sha(current),scope:'Authored helper geometry/fallback and actual sealed-file replay; source/work-cost/receipt identity equality not claimed'}));
