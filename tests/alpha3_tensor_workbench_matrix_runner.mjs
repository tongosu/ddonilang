// WO14 A3-P6; real raw-file Workbench product entry, not fixture lowering.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_TENSOR_EVIDENCE_DIR);
const input=path.resolve(process.env.DDN_TENSOR_EVIDENCE_DIR);
const report=JSON.parse(fs.readFileSync(path.join(input,'report.json'),'utf8'));
assert.equal(report.status,'PASS');assert.equal(report.native_aliases,3);assert.equal(report.actual_wasm,true);
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-tensor-workbench-'));
const rows=[];
for(const row of report.rows){
  const source=path.join(input,row.id+'.ddn');
  assert.equal(createHash('sha256').update(fs.readFileSync(source)).digest('hex'),row.source_sha256);
  const positive=row.result==='PASS';
  const diagnosticCode=positive?'':row.diagnostic.code??row.diagnostic.diagnostics?.[0]?.code;
  assert.ok(positive||diagnosticCode,'Native compile/runtime diagnostic code is required');
  const run=spawnSync(process.execPath,['tests/alpha3_numeric_display_browser_runner.mjs'],{
    env:{...process.env,DDN_OPEN_SOURCE_FILE:'1',DDN_DISPLAY_SOURCE_FILE:source,
      DDN_EXPECTED_DISPLAY:'[]',DDN_EXPECTED_ERROR_CODE:diagnosticCode,
      DDN_TENSOR_PHYSICAL_CASE:positive?row.id:'',DDN_DISPLAY_DECISION:'A3-P6 PROVISIONAL'},
    encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:16*1024*1024});
  fs.writeFileSync(path.join(evidence,row.id+'.log'),run.stdout+'\n'+run.stderr);
  assert.ifError(run.error);assert.equal(run.status,0,run.stderr);
  const child=JSON.parse(run.stdout.trim().split('\n').at(-1));assert.equal(child.status,'PASS');
  if(positive){
    assert.equal(child.result.state_hash,row.state_hash);
    assert.equal(child.result.physical_oracle.pack_sha256,row.physical_oracle.pack_sha256);
    assert.deepEqual(child.result.physical_oracle.rows,row.physical_oracle.rows);
    // VM restore mutations are separate upstream evidence, not this UI flow.
  }
  else assert.equal(child.result.frame_preserved,true);
  rows.push({id:row.id,result:row.result,browser_evidence:child.evidence_directory,wasm_sha256:child.wasm_sha256});
}
assert.equal(rows.length,15);
const result={status:'PASS',scope:'raw source file / actual Chrome Worker WASM / 3 physical positives and 12 explicit atomic rejects; shared epoch issuer not covered',rows,evidence_directory:evidence};
fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify(result,null,2));
console.log(JSON.stringify(result));
