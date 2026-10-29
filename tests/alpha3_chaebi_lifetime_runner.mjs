// A3-P3 PROVISIONAL: retained top-level setup is not recalculation.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_NATIVE_DIR && process.env.DDN_WASM_DIR);
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-chaebi-'));
const wasm=await import(pathToFileURL(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool_bg.wasm'))});
const json=value=>typeof value==='string'?JSON.parse(value):value;
const cases=[
  {id:'initializer-once',file:'solutions/seamgrim_ui_mvp/samples/alpha3_chaebi_once.ddn',expected:['1','1','1']},
  {id:'slot-initialized-at-start',file:'solutions/seamgrim_ui_mvp/samples/alpha3_chaebi_empty.ddn',expected:['4','5','6']},
];
const rows=[];
for(const c of cases){
  const source=fs.readFileSync(c.file,'utf8');let native;
  for(const name of ['ddonirang','ddonilang','또니랑']){
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,name+'.exe'),['실행',c.file,'--source-profile','v1-core-v25','--마디수','3','--json'],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:16*1024*1024});
    assert.ifError(run.error);assert.equal(run.status,0,run.stderr);const result=JSON.parse(run.stdout);
    assert.deepEqual(result.output_display.texts,c.expected);
    if(native)assert.deepEqual(result,native,c.id+'/Native3');native=result;
  }
  const result=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source,c.file,3,'v1-core-v25'));
  assert.equal(result.state_hash,native.state_hash,c.id+'/CLI-WASM summary');assert.deepEqual(result.resources,native.resources);
  const vm=new wasm.DdnWasmVm('');
  try{
    vm.update_logic_with_source_profile(source,'v1-core-v25');vm.reset(false);vm.step_one();
    const saved=JSON.stringify(json(vm.get_state_json()));
    vm.step_one();vm.step_one();
    const uninterrupted=json(vm.get_state_json()),hash=vm.get_state_hash();
    assert.deepEqual(uninterrupted.output_display.texts,c.expected);
    assert.equal(hash,native.state_hash,c.id+'/real VM');
    vm.reset(false);vm.restore_state(saved);vm.step_one();vm.step_one();
    assert.equal(vm.get_state_hash(),hash,c.id+'/restored initializer not evaluated again');
    assert.deepEqual(json(vm.get_state_json()).output_display,uninterrupted.output_display);
    vm.reset(false);for(let n=0;n<3;n++)vm.step_one();
    assert.equal(vm.get_state_hash(),hash,c.id+'/reset initializes a new run');
  }finally{vm.free();}
  rows.push({id:c.id,source_sha256:createHash('sha256').update(source).digest('hex'),state_hash:native.state_hash,expected:c.expected});
}
const vm=new wasm.DdnWasmVm('');
try{
  vm.update_logic_with_source_profile('채비 { 분모:수 := 0. 값:수 := 1 / 분모. }.','v1-core-v25');vm.reset(false);
  const before=json(vm.get_state_json());assert.throws(()=>vm.step_one(),/0으로 나눌/);
  assert.deepEqual(json(vm.get_state_json()),before,'failed initial setup cannot publish partial slots');
}finally{vm.free();}
const report={status:'PASS',decision:'A3-P3 PROVISIONAL',native_aliases:3,
  native_directory:path.resolve(process.env.DDN_NATIVE_DIR),
  native_build_mode:process.env.DDN_NATIVE_BUILD_MODE??'UNVERIFIED',
  wasm_sha256:createHash('sha256').update(fs.readFileSync(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool_bg.wasm'))).digest('hex'),
  actual_wasm:true,rows,failed_initialization:'ATOMIC_REJECT'};
fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({...report,evidence_directory:dir}));
