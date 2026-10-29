import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const source=`(값:수) 지역계산:수:셈씨 := { 값 <- 값 + 1. 임시 <- 20. 임시 <- 임시 + 1. 지역계산 <- 값 + 임시. }.
(시작)할때 { 값 <- 100. 임시 <- 300. }.
(매마디)마다 { 결과 <- (값=2) 지역계산. }.`;
async function run(dir, sourceText=source) {
 const wasm=await import(pathToFileURL(path.join(dir,'ddonirang_tool.js')).href);
 await wasm.default({module_or_path:fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'))});
 const empty=new wasm.DdnWasmVm(''); const initialWorld=empty.get_state_hash(); empty.free();
 const binding={source_profile:'v1-core-v25-logical-alpha2',input_schema:'local-binding.test@1',channel_registry:'local-binding.channels@1',
   source_revision:`sha256:${createHash('sha256').update(sourceText).digest('hex')}`,initial_world:initialWorld,initial_rng:7,
   recording_policy:'local-binding.recording@1',integrity_policy:'ddn.logical_input_tape.v1',madi_hz:60,dt_raw:4294967296,
   channels:{실험:'Control'},initial_held:{실험:false}};
 const vm=new wasm.DdnLogicalInputVm(sourceText,JSON.stringify(binding)); const rows=[];
 for(let madi=0;madi<3;madi++) rows.push(JSON.parse(vm.step_json(JSON.stringify({madi,events:[],held:{실험:false}}))).resources);
 if (typeof vm.view_json==='function') {
   const before=vm.summary_json(); vm.view_json(); assert.equal(vm.summary_json(),before,'view projection must not mutate world/receipt');
 }
 vm.free(); return rows;
}
const old=await run(process.env.DDN_PREDECESSOR_WASM_DIR);
assert.equal(old[0].fixed64.값,'3'); assert.equal(old[0].fixed64.임시,'21','old actual product must reproduce the causal leak');
const fixed=await run(process.env.DDN_WASM_DIR);
for(const {fixed64:row} of fixed) {assert.equal(row.값,'100');assert.equal(row.임시,'300');assert.equal(row.결과,'24');}
const resultless=`(값:수) 지역계산:셈씨 := { 값 <- 값 + 1. 임시 <- 20. }.
(시작)할때 { 값 <- 100. 임시 <- 300. }.
(매마디)마다 { (값=2) 지역계산. }.`;
const resultFlow=`(판:(수)차림) 지역계산:수:셈씨 := { 7 -> 판[0]. 지역계산 <- 판[0]. }.
(시작)할때 { 판 <- 차림[100,300]. }.
(매마디)마다 { 결과 <- (판=차림[2,3]) 지역계산. }.`;
const oldVoid=await run(process.env.DDN_PREDECESSOR_WASM_DIR,resultless);
assert.equal(oldVoid[0].fixed64.값,'3'); assert.equal(oldVoid[0].fixed64.임시,'20');
const newVoid=await run(process.env.DDN_WASM_DIR,resultless);
for(const {fixed64:row} of newVoid) {assert.equal(row.값,'100');assert.equal(row.임시,'300');}
const oldFlow=await run(process.env.DDN_PREDECESSOR_WASM_DIR,resultFlow);
assert.deepEqual(oldFlow[0].value_json.판,[7,3]);
const newFlow=await run(process.env.DDN_WASM_DIR,resultFlow);
for(const row of newFlow) {assert.deepEqual(row.value_json.판,[100,300]);assert.equal(row.fixed64.결과,'7');}
console.log(JSON.stringify({result:'ALPHA2_PURE_LOCAL_BINDING_RED_GREEN',actual_wasm:true,original_resource_values:{값:100,임시:300},
  predecessor_values:{값:old[0].fixed64.값,임시:old[0].fixed64.임시},fixed_madis:fixed.length,
  resultless_and_result_flow_checked:true}));
