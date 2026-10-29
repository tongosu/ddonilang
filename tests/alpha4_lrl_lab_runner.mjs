// Product project linker/codec execute authored DDN. JS supplies independent numerical checks only.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_NATIVE_DIR&&process.env.DDN_WASM_DIR);
const wasmDir=path.resolve(process.env.DDN_WASM_DIR),wasmBytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:wasmBytes});
const rootPath='alpha4_lrl_lab.project.ddn',entryPath='alpha4_lrl_lab.ddn';
const originals=[rootPath,entryPath,'alpha4_drawing_common.ddn'].map(name=>({path:name,text:fs.readFileSync(path.join('solutions/seamgrim_ui_mvp/samples',name),'utf8')}));
const source=originals.find(file=>file.path===entryPath).text;
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha4-lrl-')),rows=[];
const sha=x=>createHash('sha256').update(x).digest('hex');
function derivative(y,epsilon){
  const r=Math.hypot(y[0],y[1]),radial=-(1+epsilon/r)/r**3;
  return [y[2],y[3],radial*y[0],radial*y[1]];
}
function step(y,h,epsilon){
  const shift=(k,t)=>y.map((value,index)=>value+t*k[index]);
  const a=derivative(y,epsilon),b=derivative(shift(a,h/2),epsilon),c=derivative(shift(b,h/2),epsilon),d=derivative(shift(c,h),epsilon);
  return y.map((value,index)=>value+h*(a[index]+2*b[index]+2*c[index]+d[index])/6);
}
const distance=(a,b)=>Math.max(...a.map((value,index)=>Math.abs(value-b[index])));
const cases=[{e:0.5,epsilon:0,speed:1,h:0.01,ticks:1},{e:0.5,epsilon:0,speed:1,h:0.01,ticks:60},
  {e:0,epsilon:0,speed:1,h:0.01,ticks:60},{e:0.5,epsilon:0.02,speed:1,h:0.01,ticks:60},
  {e:0.5,epsilon:0,speed:0.9,h:0.01,ticks:60}];
if(process.env.DDN_LONG_LRL==='1')cases.push({e:0.5,epsilon:0,speed:1,h:0.01,ticks:600},{e:0.5,epsilon:0.02,speed:1,h:0.01,ticks:600});
cases.push({e:0.5,epsilon:0,speed:1,h:0.01,ticks:60,launch:[0.2,1.2]});
for(const [index,{e,epsilon,speed,h,ticks,launch}] of cases.entries()){
  // Only explicit authoring parameters change. No generated DDN, import concatenation or world/geometry injection.
  let edited=source.replace('이심률 := 0.5.',`이심률 := ${e}.`).replace('섭동 := 0.',`섭동 := ${epsilon}.`)
    .replace('속도배율 := 1.',`속도배율 := ${speed}.`).replace('간격 := 0.01.',`간격 := ${h}.`);
  if(launch)edited=edited.replace('발사설정 := 0.','발사설정 := 1.').replace('발사가로속도 := 0.',`발사가로속도 := ${launch[0]}.`).replace('발사세로속도 := 1.2.',`발사세로속도 := ${launch[1]}.`);
  const files=originals.map(file=>file.path===entryPath?{...file,text:edited}:file);
  const v1=wasm.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:{root_path:rootPath,entry_path:entryPath,files}}));
  const artifact=wasm.authored_project_operation('register_bogae',JSON.stringify({original_json:v1,module_name:'앱',source_path:entryPath,composition_definition:'LRL실험짜임',make_entry:true}));
  const artifactPath=path.join(evidence,index+'.project.json');fs.writeFileSync(artifactPath,artifact);
  const prepared=JSON.parse(wasm.wasm_project_prepare_link_json(JSON.stringify({schema:'ddn.virtual_project_source.v1',root_path:rootPath,entry_path:entryPath,
    files:files.map(file=>({path:file.path,utf8_text:file.text,role:file.path===rootPath?'root':'ddn_source'}))}),'실행',true,false));
  const actual=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(prepared.linked_execution_source,prepared.target_path,ticks,'v1-core-v25',artifact));
  for(const alias of ['ddonilang','ddonirang','또니랑']){
    const result=spawnSync(path.join(process.env.DDN_NATIVE_DIR,alias+'.exe'),['작품자료','실행',artifactPath,'--마디수',String(ticks)],
      {encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:64*1024*1024});
    assert.ifError(result.error);assert.equal(result.status,0,result.stderr||result.stdout);
    const native=JSON.parse(result.stdout);
    for(const key of ['state_hash','resources','bogae_projection','output_log_texts','work_cost_receipt'])assert.deepEqual(native[key],actual[key],alias+'/'+key);
  }
  const vm=wasm.DdnWasmVm.from_authored_project_json(artifact);let restored;
  try{
    vm.step_one();const saved=vm.export_saved_project_artifact_json('','04');
    for(let madi=1;madi<ticks;madi++)vm.step_one();
    assert.equal(vm.get_state_hash(),actual.state_hash,'actual project VM');
    assert.deepEqual(JSON.parse(vm.get_state_json()).bogae_projection,actual.bogae_projection);
    if(ticks<=60){
      restored=wasm.DdnWasmVm.from_saved_project_artifact_json(saved,'','04');
      for(let madi=1;madi<ticks;madi++)restored.step_one();
      assert.equal(restored.get_state_hash(),actual.state_hash,'saved real project resumes without reinitializing');
    }
  }finally{restored?.free();vm.free();}
  const r0=1-e,v0=launch?.[1]??speed*Math.sqrt((1+e)/r0),vx0=launch?.[0]??0,L0=r0*v0,E0=(v0*v0+vx0*vx0)/2-1/r0-epsilon/(2*r0*r0),A0=[r0*v0*v0-1,-vx0*L0];
  const values=actual.resources.value_json,initial=values.처음량,current=values.현재량,y=values.궤도값;
  assert.ok(distance(initial,[r0,L0,E0,...A0])<2e-8,'independent initial hand calculation');
  let same=[r0,0,vx0,v0],reference=[...same];
  for(let madi=0;madi<ticks;madi++)same=step(same,h,epsilon);
  for(let madi=0;madi<ticks*16;madi++)reference=step(reference,h/16,epsilon);
  const implementationError=distance(y,same),coarseError=distance(y,reference),fineError=distance(values.고운값,reference);
  assert.ok(implementationError<1e-5);assert.ok(coarseError<5e-5);
  assert.ok(Math.abs(current[1]-L0)<2e-5);assert.ok(Math.abs(current[2]-E0)<2e-5);
  const ADrift=Math.hypot(current[3]-A0[0],current[4]-A0[1]);let hodographResidual=null;
  assert.ok(Math.abs(values.A제곱-(current[3]**2+current[4]**2))<2e-8);
  if(epsilon===0)assert.ok(Math.abs(values.A검산오차)<2e-8);
  if(values.방향있음)assert.ok(Math.abs(values.방향변화-Math.atan2(A0[0]*current[4]-A0[1]*current[3],A0[0]*current[3]+A0[1]*current[4]))<2e-6);
  if(epsilon===0){
    assert.ok(ADrift<1e-5,'Kepler LRL conservation');
    hodographResidual=Math.abs(Math.hypot(y[2]+A0[1]/L0,y[3]-A0[0]/L0)-1/L0);
    assert.ok(hodographResidual<1e-5,'independent velocity-circle identity');
  }else assert.ok(ADrift>0.001,'perturbed central force must change the Kepler LRL vector');
  assert.ok(Math.abs(values.시간-h*ticks)<1e-6);
  assert.equal(actual.bogae_projection.persistent_identity,true);
  const relations=actual.bogae_projection.candidates[0].artifact.relations;
  assert.deepEqual(relations.map(relation=>relation.kind),['primary','overlay','side','side']);
  const polygons=relations[1].view.elements.filter(element=>element.kind==='polygon');
  assert.equal(polygons.length,Math.hypot(current[3],current[4])>0.0001?3:2,'zero LRL vector must not create a degenerate arrow');
  for(const [view,expectedPoint] of [[relations[0].view,[y[0],y[1]]],[relations[2].view,[y[2],y[3]]]]){
    const point=view.elements.find(element=>element.kind==='circle').center.map(value=>Number(value)/2**32);
    assert.deepEqual(point,expectedPoint,'scene must observe this committed world, not the previous madi');
  }
  rows.push({e,epsilon,speed,h,ticks,state_hash:actual.state_hash,project_files:files.map(file=>({path:file.path,sha256:sha(file.text)})),artifact_sha256:sha(artifact),implementation_error:implementationError,
    coarse_error:coarseError,half_step_error:fineError,energy_drift:current[2]-E0,L_drift:current[1]-L0,A_drift:ADrift,hodograph_residual:hodographResidual});
}
const report={status:'PASS',scope:'authored common-library project path; Fixed64 floor means half-step improvement is not unconditional',wasm_sha256:sha(wasmBytes),native_directory:path.resolve(process.env.DDN_NATIVE_DIR),rows};
fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify({...report,evidence_directory:evidence}));
