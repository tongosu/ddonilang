// WO14 exploration evidence: analytic oracle, not copied product output.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {applyExplorationPreset} from '../solutions/seamgrim_ui_mvp/ui/workbench/exploration_presets.js';
const native=process.env.DDN_NATIVE_DIR, wasmDir=process.env.DDN_WASM_DIR;
assert.ok(native&&wasmDir,'Native3 and actual WASM required');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-function-lab-'));
const original=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha3_function_lab.ddn','utf8');
const parameters={이분횟수:{min:1,max:16,integer:true},나눈수:{min:1,max:32,integer:true}};
assert.ok(applyExplorationPreset(original,parameters,{이분횟수:6,나눈수:4}).includes('나눈수 := 4.'));
assert.throws(()=>applyExplorationPreset(original,parameters,{나눈수:0}),/E_EXPLORATION_PRESET/);
assert.throws(()=>applyExplorationPreset(original,parameters,{나눈수:1.5}),/E_EXPLORATION_PRESET/);
assert.throws(()=>applyExplorationPreset(original+'\n나눈수 := 9.',parameters,{나눈수:4}),/초기값 줄/);
assert.throws(()=>applyExplorationPreset(original.replace('나눈수 := 8.','나눈수 := 4 + 4.'),parameters,{이분횟수:6,나눈수:4}),/초기값 줄/);
assert.ok(original.includes('이분횟수 := 12.'),'a rejected multi-edit leaves caller source unchanged');
const rows=[],worldHashes=new Map();
for(const mode of [0,1])for(const [iterations,n] of [[1,1],[6,4],[12,8],[16,16],[16,32]]){
  // Ordinary author parameter edits; no harness-only lowering.
  const source=original.replace('이분횟수 := 12.',`이분횟수 := ${iterations}.`).replace('나눈수 := 8.',`나눈수 := ${n}.`).replace('해석 := 0.',`해석 := ${mode}.`);
  const file=path.join(dir,`${mode}-${iterations}-${n}.ddn`);fs.writeFileSync(file,source);
  let ref;
  for(const alias of ['ddonirang','ddonilang','또니랑']){
    const r=spawnSync(path.join(native,alias+'.exe'),['실행',file,'--source-profile','v1-core-v25','--json'],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:32*1024*1024});
    assert.ifError(r.error);assert.equal(r.status,0,r.stderr);const result=JSON.parse(r.stdout);
    if(ref)assert.deepEqual(result,ref);ref=result;
  }
  const result=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source,file,1,'v1-core-v25'));
  assert.equal(result.state_hash,ref.state_hash);assert.deepEqual(result.resources,ref.resources);
  assert.deepEqual(result.bogae_projection,ref.bogae_projection);
  const values=ref.resources.value_json;
  const key=`${iterations}-${n}`;
  if(mode===0)worldHashes.set(key,ref.state_hash);else assert.equal(ref.state_hash,worldHashes.get(key),'view-only definition cannot alter world hash');
  // Bisection midpoint error <= half the final bracket width.
  assert.ok(Math.abs(values.근-Math.sqrt(2))<=2**(-iterations-1)+1e-8);
  // Composite trapezoid for x² over [0,1] = 1/3 + 1/(6n²).
  assert.ok(Math.abs(values.거친넓이-(1/3+1/(6*n*n)))<1e-8);
  assert.ok(Math.abs(values.촘촘한넓이-(1/3+1/(24*n*n)))<1e-8);
  assert.ok(Math.abs(values.거친오차-4*values.촘촘한오차)<1e-8);
  const relations=ref.bogae_projection.candidates[0].artifact.relations;
  assert.deepEqual(relations.map(r=>r.kind).sort(),['overlay','primary','side']);
  const overlay=relations.find(r=>r.kind==='overlay').view;
  assert.equal(overlay.elements.length,mode===0?1:n+1);
  assert.equal(overlay.elements.filter(e=>e.kind==='polygon').length,mode===0?0:n);
  if(mode===1) {
    // Geometry is the same n trapezoids used in the sum, not an invented UI area.
    const raw=x=>Number(BigInt(x))/2**32;
    const area=overlay.elements.filter(e=>e.kind==='polygon').reduce((sum,e)=>{
      const vertices=e.vertices.map(p=>p.map(raw));
      const twice=vertices.reduce((total,[x,y],i)=>{const [u,v]=vertices[(i+1)%vertices.length];return total+x*v-y*u;},0);
      return sum+Math.abs(twice)/2;
    },0);
    assert.ok(Math.abs(area-values.거친넓이)<1e-8);
  }
  rows.push({mode,iterations,n,state_hash:ref.state_hash,values});
}
fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify({status:'PASS',native_aliases:3,wasm:'PASS',rows},null,2));
console.log(JSON.stringify({status:'PASS',rows,evidence_directory:dir}));
