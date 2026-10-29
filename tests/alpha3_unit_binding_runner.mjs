// A3-P2 PROVISIONAL: existing typed UnitValue, no new unit/affine conversion.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

assert.ok(process.env.DDN_NATIVE_DIR && process.env.DDN_WASM_DIR);
const wasm=await import(pathToFileURL(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(process.env.DDN_WASM_DIR,'ddonirang_tool_bg.wasm'))});
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-unit-'));
const json=value=>typeof value==='string'?JSON.parse(value):value;
const cases=[
  {id:'scaled-length',source:'거리 <- 1@m. 거리 <- 100@cm. 거리 보여주기.',ticks:1,expected:['1@m']},
  {id:'derived-area',source:'검산:움직씨 := { 면적 <- 2@m * 3@m. 면적 <- 4@m * 5@m. 면적 보여주기. }.\n() 검산하기.',ticks:1,expected:['20@m^2']},
  {id:'chaebi-persistence',source:'채비 { 거리 := 1@m. }.\n(매마디)마다 { 거리 <- 거리 + 1@m. 거리 보여주기. }.',ticks:3,expected:['2@m','3@m','4@m']},
  {id:'chaebi-explicit-unit-type',source:'채비 { 거리:(m)수 := 1@m. }.\n(매마디)마다 { 거리 <- 거리 + 1@m. 거리 보여주기. }.',ticks:3,expected:['2@m','3@m','4@m']},
  {id:'root-dimension-mismatch',source:'거리 <- 1@m. 거리 <- 1@s.',ticks:1,error:true},
  {id:'local-derived-mismatch',source:'검산:움직씨 := { 면적 <- 2@m * 3@m. 면적 <- 1@s. }.\n() 검산하기.',ticks:1,error:true},
  {id:'17A',source:fs.readFileSync('solutions/seamgrim_ui_mvp/first_steps_alpha3/예제/17A_계산과표시.ddn','utf8'),ticks:1,expected:['19.02','-19.02','19.019999999']},
  {id:'17B',source:fs.readFileSync('solutions/seamgrim_ui_mvp/first_steps_alpha3/예제/17B_차원을유지하기.ddn','utf8'),ticks:1,expected:['1@m','20@m^2']},
  {id:'MOD17',source:fs.readFileSync('solutions/seamgrim_ui_mvp/first_steps_alpha3/수정실습/MOD17_음수와단위.ddn','utf8'),ticks:1,expected:['-0.08','2@m']},
];
const rows=[];
for(const c of cases){
  const file=path.join(dir,c.id+'.ddn');fs.writeFileSync(file,c.source,'utf8');
  let nativeResult,nativeError;
  for(const name of ['ddonirang','ddonilang','또니랑']){
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,name+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수',String(c.ticks),'--json'],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:16*1024*1024});
    assert.ifError(run.error);
    if(c.error){
      assert.notEqual(run.status,0);assert.match(run.stdout+run.stderr,/TYPE_MISMATCH|씨 불일치/);
      const diagnostic=JSON.parse((run.stderr||run.stdout).trim());
      if(nativeError)assert.deepEqual(diagnostic,nativeError,c.id+'/Native3 diagnostic');nativeError=diagnostic;continue;
    }
    assert.equal(run.status,0,run.stderr);const result=JSON.parse(run.stdout);
    assert.deepEqual(result.output_display.texts,c.expected,c.id);
    if(nativeResult)assert.deepEqual(result,nativeResult,c.id+'/Native3');nativeResult=result;
  }
  if(c.error){
    let wasmError;
    try{wasm.wasm_run_summary_with_source_profile_json(c.source,file,c.ticks,'v1-core-v25');assert.fail('rejection required');}catch(error){wasmError=json(error);}
    assert.deepEqual(wasmError,nativeError,c.id+'/CLI/WASM diagnostic exact');
    const vm=new wasm.DdnWasmVm('');
    try{vm.update_logic_with_source_profile(c.source,'v1-core-v25');vm.reset(false);const before=json(vm.get_state_json());assert.throws(()=>vm.step_one(),/TYPE_MISMATCH|씨 불일치/);assert.deepEqual(json(vm.get_state_json()),before,c.id+'/failed madi atomic');}finally{vm.free();}
  }else{
    const result=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(c.source,file,c.ticks,'v1-core-v25'));
    assert.equal(result.state_hash,nativeResult.state_hash,c.id+'/WASM world');
    assert.deepEqual(result.resources,nativeResult.resources,c.id+'/typed resources');
    assert.deepEqual(result.output_display,nativeResult.output_display,c.id+'/display');
    const vm=new wasm.DdnWasmVm('');
    try{vm.update_logic_with_source_profile(c.source,'v1-core-v25');vm.reset(false);for(let i=0;i<c.ticks;i++)vm.step_one();const saved=JSON.stringify(json(vm.get_state_json())),hash=vm.get_state_hash();vm.reset(false);vm.restore_state(saved);assert.equal(vm.get_state_hash(),hash,c.id+'/typed unit restore');}finally{vm.free();}
  }
  rows.push({id:c.id,result:c.error?'EXPLICIT_REJECT':'PASS',state_hash:nativeResult?.state_hash??null,diagnostic:nativeError??null,source_sha256:createHash('sha256').update(c.source).digest('hex')});
}
const report={status:'PASS',decision:'A3-P2 PROVISIONAL',native_aliases:3,actual_wasm:true,rows};
fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({...report,evidence_directory:dir}));
