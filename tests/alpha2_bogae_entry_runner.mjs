import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
const dir=process.env.DDN_WASM_DIR;
const wasm=await import(pathToFileURL(path.join(dir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'))});
const source='첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표. 수값 := 2.';
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-bogae-entry-'));
const file=path.join(evidence,'original.ddn');fs.writeFileSync(file,source);
let checks=0;
for(const alias of ['ddonilang','ddonirang','또니랑']) {
 for(const entry of ['첫째','둘째','없는이름','수값',null]) {
  const args=['실행',file,'--source-profile','v1-core-v25','--json'];
  if(entry!==null) args.push('--보개-시작',entry);
  const native=spawnSync(path.join(process.env.DDN_NATIVE_DIR,`${alias}.exe`),args,{encoding:'utf8',timeout:30000});
  if(entry==='첫째'||entry==='둘째') {
   assert.equal(native.status,0,native.stderr);
   const projected=JSON.parse(native.stdout);
   const summary=JSON.parse(wasm.wasm_run_summary_with_bogae_entry_json(source,file,-1,'v1-core-v25',entry));
   assert.deepEqual(projected,summary,'actual common Native/WASM frontdoors');
   assert.equal(summary.bogae_projection.candidates[0].label,entry);
   assert.equal(summary.bogae_projection.candidates.length,1);
   assert.equal(summary.bogae_projection.persistent_identity,false);
  } else {
   const code=entry===null?'E_BOGAE_SELECTION_REQUIRED':'E_BOGAE_ENTRY_UNRESOLVED';
   assert.notEqual(native.status,0);assert.ok(native.stderr.includes(code),native.stderr);
   assert.equal(native.stdout,'','failure must not publish summary');
   assert.throws(()=>entry===null
    ? wasm.wasm_run_summary_with_source_profile_json(source,file,1,'v1-core-v25')
    : wasm.wasm_run_summary_with_bogae_entry_json(source,file,1,'v1-core-v25',entry),error=>String(error).includes(code));
  }
  checks++;
 }
}
console.log(JSON.stringify({result:'ALPHA2_BOGAE_EXPLICIT_ENTRY_PASS',checks,native_aliases:3,actual_wasm:true,evidence,
 scope:'Explicit temporary selection only; persistent composition identity and full F1 not claimed'}));
