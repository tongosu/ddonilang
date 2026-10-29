import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {performance} from 'node:perf_hooks';
const wasm=await import(pathToFileURL(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool_bg.wasm'))});
const oldSource=fs.readFileSync(process.env.DDN_PREDECESSOR_SOURCE,'utf8');
const source=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn','utf8');
const sha=text=>createHash('sha256').update(text).digest('hex');
assert.equal(sha(oldSource),'acff53cfae9cf633a28687bf5a6b61868c214c8dea5ba5340974cb6a9d9518e3');
const empty=new wasm.DdnWasmVm(''); const initialWorld=empty.get_state_hash();empty.free();
const channels=Object.fromEntries(['왼쪽','오른쪽','아래','회전','떨어뜨리기','다시시작'].map(name=>[name,'Control']));
const held=Object.fromEntries(Object.keys(channels).map(name=>[name,false]));
const inputs=[[],['왼쪽'],['오른쪽','회전'],['아래'],['떨어뜨리기'],['다시시작']].map((actions,madi)=>({
 madi,events:actions.flatMap((channel,n)=>[{ordinal:n*2,channel,event:{kind:'Press'}},
 {ordinal:n*2+1,channel,event:{kind:'Release'}}]),held}));
function run(text) {
 const binding={source_profile:'v1-core-v25-logical-alpha2',input_schema:'alpha2.tetris.controls@1',
 channel_registry:'alpha2.tetris.channels@1',source_revision:`sha256:${sha(text)}`,initial_world:initialWorld,initial_rng:7,
 recording_policy:'alpha2.tetris.recording@1',integrity_policy:'ddn.logical_input_tape.v1',madi_hz:60,dt_raw:4294967296,channels,initial_held:held};
 const vm=new wasm.DdnLogicalInputVm(text,JSON.stringify(binding)); const rows=[]; const times=[];
 for(const input of inputs) {
  const start=performance.now(); const row=JSON.parse(vm.step_json(JSON.stringify(input)));times.push(performance.now()-start);
  rows.push({row,view:JSON.parse(vm.view_json())});
 }
 vm.free();return {rows,times};
}
const before=run(oldSource),after=run(source);
const gameplay=['판','점수','지운줄','끝남값','마디계수','가방','조각번호','지역칸','가로','세로','현재칸','다음조각',
 '보개_그림판_목록','보개_그림판_가로','보개_그림판_세로'];
const removed=['색표','배경','행','열','색','칸그림'];
for(let n=0;n<inputs.length;n++) {
 for(const key of gameplay) assert.deepEqual(after.rows[n].row.resources.value_json[key],before.rows[n].row.resources.value_json[key],`madi${n} ${key}`);
 assert.deepEqual(after.rows[n].view,before.rows[n].view,`madi${n} authored drawing must remain identical`);
 for(const key of removed) assert.equal(Object.hasOwn(after.rows[n].row.resources.value_json,key),false,`${key} must stay local`);
}
console.log(JSON.stringify({result:'ALPHA2_TETRIS_DRAWING_LOCAL_PASS',actual_wasm:true,madis:inputs.length,
 predecessor_source_sha256:sha(oldSource),source_sha256:sha(source),world_hash_equivalence_claim:false,
 gameplay_and_view_equal:true,local_only:removed,before_step_ms:before.times,after_step_ms:after.times,
 scope:'Same input gameplay/drawing projection; timing is observational, not a throughput gate or whole-world hash equivalence'}));
