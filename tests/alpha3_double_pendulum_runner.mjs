// A3-P5: DDN runs the physics. JS is an independent fine-step oracle only.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const native=process.env.DDN_NATIVE_DIR,wasmDir=process.env.DDN_WASM_DIR;
assert.ok(native&&wasmDir);
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const source=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha3_double_pendulum_lab.ddn','utf8');
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-double-pendulum-')),rows=[];
const rootPath='alpha3_double_pendulum_lab.project.ddn',entryPath='alpha3_double_pendulum_lab.ddn';
const fileNames=[rootPath,entryPath,'alpha4_drawing_common.ddn'];
const originalFiles=fileNames.map(name=>({path:name,text:fs.readFileSync(path.join('solutions/seamgrim_ui_mvp/samples',name),'utf8')}));
const sha=x=>createHash('sha256').update(x).digest('hex');
function project(edited){
  const files=originalFiles.map(file=>file.path===entryPath?{...file,text:edited}:file);
  const snapshot={root_path:rootPath,entry_path:entryPath,files};
  const original=wasm.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:snapshot}));
  const artifact=wasm.authored_project_operation('register_bogae',JSON.stringify({original_json:original,module_name:'앱',source_path:entryPath,composition_definition:'진자실험짜임',make_entry:true}));
  const prepared=JSON.parse(wasm.wasm_project_prepare_link_json(JSON.stringify({schema:'ddn.virtual_project_source.v1',root_path:rootPath,entry_path:entryPath,
    files:files.map(file=>({path:file.path,utf8_text:file.text,role:file.path===rootPath?'root':'ddn_source'}))}),'실행',true,false));
  return {artifact,prepared,files};
}
// Independently solve the 2x2 mass matrix rather than copying DDN's expanded formula.
function derivative([a,b,u,v]) {
  const s=Math.sin(a-b),c=Math.cos(a-b);
  const r1=-19.6*Math.sin(a)-v*v*s,r2=-9.8*Math.sin(b)+u*u*s;
  return [u,v,(r1-c*r2)/(2-c*c),(2*r2-c*r1)/(2-c*c)];
}
function step(y,h) {
  const shift=(k,t)=>y.map((v,i)=>v+t*k[i]);
  const k1=derivative(y),k2=derivative(shift(k1,h/2)),k3=derivative(shift(k2,h/2)),k4=derivative(shift(k3,h));
  return y.map((v,i)=>v+h*(k1[i]+2*k2[i]+2*k3[i]+k4[i])/6);
}
function energy([a,b,u,v]) {return u*u+v*v/2+u*v*Math.cos(a-b)-9.8*(2*Math.cos(a)+Math.cos(b));}
const cases=[[0,0,0.02,20],[1.2,0.8,0.02,60],[1.2,0.8,0.04,30]];
if(process.env.DDN_LONG_PENDULUM==='1')cases.push([1.2,0.8,0.02,600],[2,1.8,0.02,600]);
for(const [a,b,h,ticks] of cases) {
  const edited=source.replace('초기각1 := 1.2.',`초기각1 := ${a}.`).replace('초기각2 := 0.8.',`초기각2 := ${b}.`).replace('간격 := 0.02.',`간격 := ${h}.`);
  const {artifact,prepared,files}=project(edited);
  const file=path.join(dir,`${a}-${b}-${h}.project.json`);fs.writeFileSync(file,artifact);
  const actual=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(prepared.linked_execution_source,prepared.target_path,ticks,'v1-core-v25',artifact));
  let ref;
  for(const name of ['ddonirang','ddonilang','또니랑']) {
    const r=spawnSync(path.join(native,name+'.exe'),['작품자료','실행',file,'--마디수',String(ticks)],{encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:64*1024*1024});
    assert.ifError(r.error);assert.equal(r.status,0,r.stderr);
    const result=JSON.parse(r.stdout);if(ref)assert.deepEqual(result,ref);ref=result;
  }
  for(const key of ['state_hash','resources','bogae_projection','output_log_texts','work_cost_receipt'])assert.deepEqual(actual[key],ref[key],key);
  assert.equal(actual.bogae_projection.persistent_identity,true);
  let maximumLocalResidual=ticks>60?0:null;
  const vm=wasm.DdnWasmVm.from_authored_project_json(artifact);
  try{
    let previous=[a,b,0,0];
    for(let madi=0;madi<ticks;madi++){
      vm.step_one();
      if(ticks>60){
        const expected=step(previous,h);
        const current=JSON.parse(vm.get_state_json()).resources.value_json.기준진자값;
        const residual=Math.max(...current.map((v,i)=>Math.abs(v-expected[i])));
        maximumLocalResidual=Math.max(maximumLocalResidual,residual);
        assert.ok(residual<0.000003,JSON.stringify({a,b,madi,residual}));previous=current;
      }
    }
    assert.equal(vm.get_state_hash(),ref.state_hash,'actual project VM');
    const state=JSON.parse(vm.get_state_json());
    // Incremental get_state_json is the legacy projection, while batch uses the
    // typed v2 resource snapshot. Compare every legacy channel, not absent wire metadata.
    for(const key of ['fixed64','handle','json','value','value_json'])assert.deepEqual(state.resources[key],ref.resources[key],key);
    assert.deepEqual(state.bogae_projection,ref.bogae_projection);
  }finally{vm.free();}
  const values=ref.resources.value_json,coarse=values.기준진자값,fine=values.고운진자값;
  let oracle=[a,b,0,0],sameStep=[a,b,0,0];
  for(let i=0;i<ticks*16;i++)oracle=step(oracle,h/16);
  for(let i=0;i<ticks;i++)sameStep=step(sameStep,h);
  const distance=y=>Math.max(...y.map((v,i)=>Math.abs(v-oracle[i])));
  // Separate Fixed64 implementation error from RK4's finite-step modelling error.
  const implementationError=Math.max(...coarse.map((v,i)=>Math.abs(v-sameStep[i])));
  if(ticks<=60)assert.ok(implementationError<0.000003,JSON.stringify({a,b,ticks,implementationError}));
  if(ticks<=60){
    assert.ok(distance(coarse)<500*h**4+0.000003,JSON.stringify({h,coarse,oracle,error:distance(coarse)}));
    assert.ok(distance(fine)<500*(h/2)**4+0.000003,JSON.stringify({h,fine,oracle,error:distance(fine)}));
    if(a||b)assert.ok(distance(fine)<distance(coarse)/4,'halving the step must actually improve the result');
    assert.ok(Math.abs(energy(coarse)-energy([a,b,0,0]))<0.001);
  }else{
    // A3-P5 long observation is a distinct, measured numerical scope;
    // do not silently stretch the short-interval error bound into a theorem.
    assert.ok(distance(fine)<distance(coarse),'fine-step convergence on the explicit long scenarios');
    assert.ok(Math.abs(energy(coarse)-energy([a,b,0,0]))<0.02,'long-scenario energy drift');
  }
  assert.ok(Math.abs(values.시간-h*ticks)<(ticks<=60?1e-7:1e-6));
  assert.deepEqual(ref.bogae_projection.candidates[0].artifact.relations.map(r=>r.kind),['primary','overlay','side','side']);
  const circles=ref.bogae_projection.candidates[0].artifact.relations[0].view.elements.filter(element=>element.kind==='circle');
  const bob1=circles.at(-2).center.map(value=>Number(value)/2**32),bob2=circles.at(-1).center.map(value=>Number(value)/2**32);
  const x1=Math.sin(coarse[0]),y1=-Math.cos(coarse[0]);
  for(const [actualPoint,expectedPoint] of [[bob1,[x1,y1]],[bob2,[x1+Math.sin(coarse[1]),y1-Math.cos(coarse[1])]]])
    assert.ok(Math.max(...actualPoint.map((value,index)=>Math.abs(value-expectedPoint[index])))<1e-7,'projection must observe the committed current world, not the previous madi');
  const dots=ref.bogae_projection.candidates[0].artifact.relations[1].view.elements.filter(element=>element.kind==='circle');
  const compared=values.비교진자값,cx1=Math.sin(compared[0]),cy1=-Math.cos(compared[0]),cx2=cx1+Math.sin(compared[1]),cy2=cy1-Math.cos(compared[1]);
  assert.equal(dots.length,17,'fixed point plus two eight-dot segments from the common DDN library');
  for(let number=1;number<=8;number++){
    const fraction=number/8;
    for(const [dot,expectedPoint] of [[dots[number*2-1],[cx1*fraction,cy1*fraction]],[dots[number*2],[cx1+(cx2-cx1)*fraction,cy1+(cy2-cy1)*fraction]]]){
      const point=dot.center.map(value=>Number(value)/2**32);
      assert.ok(Math.max(...point.map((value,index)=>Math.abs(value-expectedPoint[index])))<1e-7,'common dot positions preserve the authored segment and the current comparison world');
    }
  }
  rows.push({a,b,h,ticks,state_hash:ref.state_hash,project_files:files.map(file=>({path:file.path,sha256:sha(file.text)})),artifact_sha256:sha(artifact),float_trajectory_difference:implementationError,maximum_local_residual:maximumLocalResidual,coarse_error:distance(coarse),fine_error:distance(fine),energy_error:energy(coarse)-energy([a,b,0,0]),values});
}
fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify({status:'PASS',wasm_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))),rows},null,2));
console.log(JSON.stringify({status:'PASS',rows,evidence_directory:dir}));
