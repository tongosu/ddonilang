// WO14 A3-P1/P2/P4/P5: actual education bytes through Native3 and WASM.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const dir=path.resolve('solutions/seamgrim_ui_mvp/first_steps_alpha3');
const manifest=JSON.parse(fs.readFileSync(path.join(dir,'검증목록.json'),'utf8'));
const wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const out=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-course-'));
const sha=x=>createHash('sha256').update(x).digest('hex');
function native(alias,args){const r=spawnSync(path.join(process.env.DDN_NATIVE_DIR,alias+'.exe'),args,
  {encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024,windowsHide:true});
  assert.ifError(r.error);assert.equal(r.status,0,r.stderr);return JSON.parse(r.stdout);}
const evidence=[];
// RD-293/RD-445: the alert must be typed before dispatch, even if unread.
for(const [pin,value,reject] of [['수','3',false],['수','"틀린씨"',true],['수','참',true],['글','"정상"',false],['글','3',true]]){
  const text=`(값:${pin}) 첫알림:알림씨 := { 없음. }.
관제탑:임자 := { (받은:첫알림) 첫반응:움직씨 {첫알림을 받으면} := { 성공 <- 7. }. }.
매틱:움직씨 := { (철수)의 ((값=${value}) 첫알림) ~> 관제탑. }.`;
  const file=path.join(out,`typed-alert-${evidence.length}.ddn`);fs.writeFileSync(file,text);
  let result,error;
  try{result=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(text,file,1,'v1-core-v25'));}
  catch(e){error=String(e);}
  if(reject){
    assert.match(error??JSON.stringify(result),/E_TYPE_MISMATCH/);
    assert.ok(!result?.state_hash&&!result?.resources,'failed alert cannot publish success evidence');
  }else{assert.equal(result.resources.fixed64.성공,'7');}
  for(const alias of ['ddonilang','ddonirang','또니랑']){
    const r=spawnSync(path.join(process.env.DDN_NATIVE_DIR,alias+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],
      {encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024,windowsHide:true});
    assert.ifError(r.error);
    if(reject){assert.notEqual(r.status,0);assert.match(r.stderr+r.stdout,/E_TYPE_MISMATCH/);}
    else{assert.equal(r.status,0,r.stderr);assert.equal(JSON.parse(r.stdout).state_hash,result.state_hash);}
  }
  evidence.push({id:'RD293_TYPED_ALERT',pin,value,rejected:reject,native_aliases:3});
}
for(const row of [...manifest.cases,...manifest.modifications]){
  const file=path.resolve(dir,row.file),source=fs.readFileSync(file,'utf8');
  const ticks=row.ticks??1;
  if(row.project_files){
    const files=row.project_files.map(item=>({path:item.path,text:fs.readFileSync(path.resolve(dir,item.file),'utf8')}));
    const artifact=wasm.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:{root_path:row.project_root,entry_path:row.project_entry,files}}));
    if(row.project_validation==='physics'){
      assert.equal(row.id,'18B','this branch is the bounded double-pendulum lesson, not the 27A lifetime oracle');
      assert.match(source,/초기각1 := 1\.2\./);assert.match(source,/초기각2 := 0\.8\./);assert.match(source,/간격 := 0\.02\./);
      const registered=wasm.authored_project_operation('register_bogae',JSON.stringify({original_json:artifact,module_name:'앱',source_path:row.project_entry,composition_definition:row.bogae_entry,make_entry:true}));
      const prepared=JSON.parse(wasm.wasm_project_prepare_link_json(JSON.stringify({schema:'ddn.virtual_project_source.v1',root_path:row.project_root,entry_path:row.project_entry,
        files:files.map(file=>({path:file.path,utf8_text:file.text,role:file.path===row.project_root?'root':'ddn_source'}))}),'실행',true,false));
      const w=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(prepared.linked_execution_source,prepared.target_path,ticks,'v1-core-v25',registered));
      const artifactFile=path.join(out,row.id+'.project.json');fs.writeFileSync(artifactFile,registered);
      for(const alias of ['ddonilang','ddonirang','또니랑']){
        const n=native(alias,['작품자료','실행',artifactFile,'--마디수',String(ticks)]);
        for(const key of ['state_hash','resources','bogae_projection','output_log_texts','work_cost_receipt'])assert.deepEqual(n[key],w[key],row.id+'/'+alias+'/'+key);
      }
      assert.equal(w.bogae_projection.persistent_identity,true);
      assert.deepEqual(w.bogae_projection.candidates[0].artifact.relations.map(relation=>relation.kind),['primary','overlay','side','side']);
      // Independent mass-matrix RK4, unchanged physical defaults; no geometry or world is supplied to DDN.
      const derivative=([a,b,u,v])=>{const s=Math.sin(a-b),c=Math.cos(a-b),r1=-19.6*Math.sin(a)-v*v*s,r2=-9.8*Math.sin(b)+u*u*s;return [u,v,(r1-c*r2)/(2-c*c),(2*r2-c*r1)/(2-c*c)];};
      const step=(y,h)=>{const shift=(k,t)=>y.map((v,i)=>v+t*k[i]);const a=derivative(y),b=derivative(shift(a,h/2)),c=derivative(shift(b,h/2)),d=derivative(shift(c,h));return y.map((v,i)=>v+h*(a[i]+2*b[i]+2*c[i]+d[i])/6);};
      let oracle=[1.2,0.8,0,0];for(let madi=0;madi<ticks;madi++)oracle=step(oracle,0.02);
      const error=Math.max(...w.resources.value_json.기준진자값.map((value,index)=>Math.abs(value-oracle[index])));
      assert.ok(error<0.000003,'18B independent physics');
      const vm=wasm.DdnWasmVm.from_authored_project_json(registered);let restored;
      try{
        vm.step_one();const saved=vm.export_saved_project_artifact_json('','18');
        for(let madi=1;madi<ticks;madi++)vm.step_one();assert.equal(vm.get_state_hash(),w.state_hash);
        assert.deepEqual(JSON.parse(vm.get_state_json()).bogae_projection,w.bogae_projection);
        restored=wasm.DdnWasmVm.from_saved_project_artifact_json(saved,'','18');
        for(let madi=1;madi<ticks;madi++)restored.step_one();assert.equal(restored.get_state_hash(),w.state_hash);
      }finally{restored?.free();vm.free();}
      evidence.push({id:row.id,file,source_sha256:sha(source),project_files:files.map(file=>({path:file.path,sha256:sha(file.text)})),state_hash:w.state_hash,native_aliases:3,project_ticks:ticks,physics_error:error,saved_project_restore:true});
      continue;
    }
    const vm=wasm.DdnWasmVm.from_authored_project_json(artifact);
    let restored;
    try{
      vm.step_one();
      const firstHash=vm.get_state_hash();
      for(const alias of ['ddonilang','ddonirang','또니랑']){
        const n=native(alias,['project-run',path.resolve(dir,row.project_files.find(f=>f.role==='root').file)]);
        assert.deepEqual(n.runtime_output_log_texts,['12']);assert.equal(n.runtime_state_hash,firstHash);
      }
      vm.step_one();const saved=vm.export_saved_project_artifact_json('','02');
      vm.step_one();assert.deepEqual(JSON.parse(vm.get_state_json()).output_display.texts,row.expected);
      restored=wasm.DdnWasmVm.from_saved_project_artifact_json(saved,'','02');restored.step_one();
      assert.equal(restored.get_state_hash(),vm.get_state_hash());
      evidence.push({id:row.id,file,source_sha256:sha(source),project_files:files.map(f=>({path:f.path,sha256:sha(f.text)})),state_hash:vm.get_state_hash(),native_aliases:3,native_project_ticks:1,wasm_ticks:3,saved_project_restore:true});
    }finally{restored?.free();vm.free();}
    continue;
  }
  if(row.expected?.reject){
    assert.throws(()=>wasm.wasm_run_summary_with_source_profile_json(source,file,ticks,'v1-core-v25'),new RegExp(row.expected.reject));
    for(const alias of ['ddonilang','ddonirang','또니랑']){
      const r=spawnSync(path.join(process.env.DDN_NATIVE_DIR,alias+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수',String(ticks),'--json'],{encoding:'utf8',windowsHide:true});
      assert.notEqual(r.status,0);assert.match(r.stdout+r.stderr,new RegExp(row.expected.reject));
    }
    const vm=new wasm.DdnWasmVm('');
    try{vm.update_logic_with_source_profile(source,'v1-core-v25');vm.reset(false);
      const before=vm.get_state_json();assert.throws(()=>vm.step_one(),new RegExp(row.expected.reject));
      assert.equal(vm.get_state_json(),before,row.id+'/whole failed madi unchanged');
    }finally{vm.free();}
    evidence.push({id:row.id,file,source_sha256:sha(source),reject:row.expected.reject,native_aliases:3,atomic:true});continue;
  }
  if(!row.expected&&!row.expected_resources&&!row.time_kind) {evidence.push({id:row.id,file,source_sha256:sha(source),execution:'covered by dedicated lab or logical tests, not inferred from asset existence'});continue;}
  const w=JSON.parse(row.bogae_entry?
    wasm.wasm_run_summary_with_bogae_entry_json(source,file,ticks,'v1-core-v25',row.bogae_entry):
    wasm.wasm_run_summary_with_source_profile_json(source,file,ticks,'v1-core-v25'));
  if(row.expected)assert.deepEqual(w.output_display.texts,row.expected,row.id+'/independent display');
  if(row.expected_resources)for(const [key,value] of Object.entries(row.expected_resources))assert.equal(w.resources.fixed64[key],value,row.id+'/'+key);
  if(row.time_kind){
    const relations=w.bogae_projection.candidates[0].artifact.relations;
    assert.equal(relations[1].resolved_madi,row.time_kind==='relative'?1:0);
    if(row.time_kind==='table')assert.equal(relations[1].view.table_projection.tables[0].rows[0].cells[0].value_text,'1');
    else assert.equal(relations[1].view.elements[0].center[0],row.time_kind==='relative'?'8589934592':'4294967296');
  }
  for(const alias of ['ddonilang','ddonirang','또니랑']){
    const n=native(alias,['실행',file,'--source-profile','v1-core-v25','--마디수',String(ticks),...(row.bogae_entry?['--보개-시작',row.bogae_entry]:[]),'--json']);
    if(row.expected)assert.deepEqual(n.output_display.texts,row.expected,row.id+'/'+alias);
    assert.equal(n.state_hash,w.state_hash);assert.deepEqual(n.resources,w.resources);
  }
  if(row.state_lifetime){
    const vm=new wasm.DdnWasmVm('');
    try{
      vm.update_logic_with_source_profile(source,'v1-core-v25');vm.reset(false);vm.step_one();vm.step_one();
      const saved=vm.get_state_json();vm.step_one();assert.equal(vm.get_state_hash(),w.state_hash);
      vm.reset(false);vm.restore_state(saved);vm.step_one();assert.equal(vm.get_state_hash(),w.state_hash);
      vm.reset(false);vm.step_one();assert.deepEqual(JSON.parse(vm.get_state_json()).output_display.texts,['12']);
    }finally{vm.free();}
  }
  evidence.push({id:row.id,file,source_sha256:sha(source),state_hash:w.state_hash,display:w.output_display?.texts??[],native_aliases:3,state_lifetime:row.state_lifetime??false,time_kind:row.time_kind??null});
}
for(const row of manifest.cases.filter(x=>x.binding_file)){
const file=path.resolve(dir,row.file),source=fs.readFileSync(file,'utf8');
const binding=JSON.parse(fs.readFileSync(path.join(dir,row.binding_file),'utf8'));
assert.equal(binding.source_revision,'sha256:'+sha(source));
for(const [order,expected] of [[['더함','곱함'],'6'],[['곱함','더함'],'5']]){
  const request={binding,snapshots:[{madi:0,events:order.map((channel,ordinal)=>({ordinal,channel,event:{kind:'Press'}})),held:{더함:true,곱함:true}}]};
  const w=JSON.parse(wasm.logical_input_record(source,JSON.stringify(request)));
  assert.equal(w.summary.resources.fixed64.계산값,expected);assert.equal(w.summary.resources.fixed64.마디결과,expected);
  const requestFile=path.join(out,expected+'.request.json'),tape=path.join(out,expected+'.tape');
  fs.writeFileSync(requestFile,JSON.stringify(request));
  for(const alias of ['ddonilang','ddonirang','또니랑']){
    const n=native(alias,['논리입력기록',file,requestFile,tape]);assert.deepEqual(n,w.summary);
    const replay=native(alias,['논리입력재생',file,tape,path.join(dir,row.binding_file)]);assert.deepEqual(replay,w.summary);
  }
  evidence.push({id:row.id,order,expected,state_hash:w.summary.state_hash,input_hash:w.summary.input_hash,native_aliases:3,actual_file_replay:true});
}
}
const report={status:'PASS',wasm_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))),native_directory:path.resolve(process.env.DDN_NATIVE_DIR),evidence};
fs.writeFileSync(path.join(out,'report.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify({...report,evidence_directory:out}));
