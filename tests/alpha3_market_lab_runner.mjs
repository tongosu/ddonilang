// A3-P5: fixed, explicitly stated linear model; not an empirical market claim.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
const native=process.env.DDN_NATIVE_DIR,wasmDir=process.env.DDN_WASM_DIR;
assert.ok(native&&wasmDir);
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const source=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha3_market_lab.ddn','utf8');
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-market-')),rows=[];
const hashes=new Map();
for(const mode of [0,1])for(const [tax,rate,years] of [[0,0,0],[8,0.05,5],[16,0.1,10]]) {
  const edited=source.replace('세금 := 8.',`세금 := ${tax}.`).replace('이자율 := 0.05.',`이자율 := ${rate}.`).replace('해수 := 5.',`해수 := ${years}.`).replace('시장해석 := 0.',`시장해석 := ${mode}.`);
  const file=path.join(dir,`${mode}-${tax}-${rate}-${years}.ddn`);fs.writeFileSync(file,edited);
  let ref;
  for(const name of ['ddonirang','ddonilang','또니랑']) {
    const r=spawnSync(path.join(native,name+'.exe'),['실행',file,'--source-profile','v1-core-v25','--json'],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:32*1024*1024});
    assert.ifError(r.error);assert.equal(r.status,0,r.stderr);const result=JSON.parse(r.stdout);if(ref)assert.deepEqual(result,ref);ref=result;
  }
  const actual=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(edited,file,1,'v1-core-v25'));
  assert.equal(actual.state_hash,ref.state_hash);assert.deepEqual(actual.resources,ref.resources);assert.deepEqual(actual.bogae_projection,ref.bogae_projection);
  const v=ref.resources.value_json;
  const key=`${tax}-${rate}-${years}`;
  if(mode===0)hashes.set(key,ref.state_hash);else assert.equal(ref.state_hash,hashes.get(key));
  // Independent solution of demand/supply plus Pc-Ps=t.
  assert.equal(v.소비가격,10+3*tax/8);assert.equal(v.생산가격,10-5*tax/8);
  assert.equal(v.거래수량,50-15*tax/8);assert.equal(v.세수,tax*v.거래수량);
  assert.equal(v.소비부담+v.생산부담,tax);
  // Fixed64 literal/product truncation remains < 1e-5 over ten periods.
  assert.ok(Math.abs(v.복리결과-1000*(1+rate)**years)<1e-5);
  assert.ok(Math.abs(v.단리결과-1000*(1+rate*years))<1e-5);
  if(mode===1) {
    const relations=ref.bogae_projection.candidates[0].artifact.relations;
    const compound=relations.find(r=>r.kind==='overlay').view.elements.filter(e=>e.kind==='circle');
    const simple=relations.find(r=>r.kind==='primary').view.elements.filter(e=>e.kind==='circle');
    assert.equal(compound.length,years+1);assert.equal(simple.length,years+1);
    const raw=x=>Number(BigInt(x))/2**32;
    for(let year=0;year<=years;year++) {
      assert.ok(Math.abs(raw(compound[year].center[0])*3-year)<0.00000001,'authored coordinate scale matches year labels');
      assert.ok(Math.abs(raw(compound[year].center[1])*1000-1000*(1+rate)**year)<0.00001);
      assert.ok(Math.abs(raw(simple[year].center[1])*1000-1000*(1+rate*year))<0.00001);
    }
  }
  rows.push({mode,tax,rate,years,state_hash:ref.state_hash,values:v});
}
fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify({status:'PASS',rows,native_aliases:3,wasm:'PASS'},null,2));
console.log(JSON.stringify({status:'PASS',rows,evidence_directory:dir}));
