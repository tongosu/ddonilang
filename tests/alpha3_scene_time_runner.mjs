// A3-TIME-P1 PROVISIONAL: exact product source, Native3 and actual WASM.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_NATIVE_DIR && process.env.DDN_WASM_DIR);
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-scene-time-'));
const wasm=await import(pathToFileURL(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool_bg.wasm'))});
const json=value=>typeof value==='string'?JSON.parse(value):value;
const rows=[];
for(const [file,entry,kind] of [
  ['solutions/seamgrim_ui_mvp/samples/alpha3_scene_time.ddn','비교짜임','scene'],
  ['solutions/seamgrim_ui_mvp/samples/alpha3_table_time.ddn','짜임','table'],
  ['solutions/seamgrim_ui_mvp/samples/alpha3_scene_relative.ddn','비교짜임','relative'],
]) {
  const source=fs.readFileSync(file,'utf8');let native;
  const check=state=>{
    const relations=state.bogae_projection.candidates[0].artifact.relations;
    assert.equal(relations[1].resolved_madi,kind==='relative'?1:0);
    if(kind!=='table')assert.equal(relations[1].view.elements[0].center[0],kind==='relative'?'8589934592':'4294967296');
    else assert.equal(relations[1].view.table_projection.tables[0].rows[0].cells[0].value_text,'1');
  };
  for(const name of ['ddonirang','ddonilang','또니랑']) {
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,name+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수','3','--보개-시작',entry,'--json'],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:32*1024*1024});
    assert.ifError(run.error);assert.equal(run.status,0,run.stderr);
    const result=JSON.parse(run.stdout);check(result);
    if(native)assert.deepEqual(result,native);native=result;
  }
  const summary=json(wasm.wasm_run_summary_with_bogae_entry_json(source,file,3,'v1-core-v25',entry));
  check(summary);assert.equal(summary.state_hash,native.state_hash);
  assert.deepEqual(summary.bogae_projection,native.bogae_projection);
  const vm=new wasm.DdnWasmVm('');
  try {
    vm.update_logic_with_source_profile(source,'v1-core-v25');vm.configure_bogae_entry(entry);vm.reset(false);
    vm.step_one();const saved=JSON.stringify(json(vm.get_state_json()));
    vm.step_one();vm.step_one();const uninterrupted=json(vm.get_state_json());check(uninterrupted);
    const hash=vm.get_state_hash();assert.equal(hash,native.state_hash);
    vm.reset(false);vm.restore_state(saved);
    assert.deepEqual(json(vm.get_state_json()).bogae_projection,JSON.parse(saved).bogae_projection,
      'reopening must restore the selected picture immediately, without advancing the world');
    vm.step_one();vm.step_one();
    const restored=json(vm.get_state_json());check(restored);
    assert.equal(vm.get_state_hash(),hash);
    assert.deepEqual(restored.bogae_projection,uninterrupted.bogae_projection);
    const corrupt=JSON.parse(saved);corrupt.state.snapshot_v2.scene_history.sha256='bad';
    const before=json(vm.get_state_json());assert.throws(()=>vm.restore_state(JSON.stringify(corrupt)),/E_BOGAE_HISTORY_SNAPSHOT_INVALID/);
    assert.deepEqual(json(vm.get_state_json()),before);
  } finally {vm.free();}
  rows.push({kind,source_sha256:createHash('sha256').update(source).digest('hex'),state_hash:native.state_hash,save_restore:'PASS',corrupt_restore:'ATOMIC_REJECT'});
}
const invalidSource=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha3_scene_time.ddn','utf8').replace('번호=0','번호=9');
const invalidVm=new wasm.DdnWasmVm('');
try {
  invalidVm.update_logic_with_source_profile(invalidSource,'v1-core-v25');invalidVm.configure_bogae_entry('비교짜임');invalidVm.reset(false);
  const before=json(invalidVm.get_state_json());
  assert.throws(()=>invalidVm.step_one(),/E_BOGAE_TIME_OUT_OF_RANGE/);
  assert.deepEqual(json(invalidVm.get_state_json()),before,'missing scene cannot advance world, madi or presentation history');
}finally{invalidVm.free();}
const report={status:'PASS',decision:'A3-TIME-P1 PROVISIONAL',missing_madi:'ATOMIC_REJECT',native_aliases:3,native_build_mode:process.env.DDN_NATIVE_BUILD_MODE??'UNVERIFIED',
  actual_wasm:true,wasm_sha256:createHash('sha256').update(fs.readFileSync(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool_bg.wasm'))).digest('hex'),rows};
fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({...report,evidence_directory:dir}));
