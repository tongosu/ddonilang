// WO14 VC05 successor evidence, not the historical P1 suite or SSOT promotion.
// Real Workbench/Worker/WASM; no source lowering or host-calculated result.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_NUMERIC_EVIDENCE_DIR,'Native3/WASM numeric report required');
const input=path.resolve(process.env.DDN_NUMERIC_EVIDENCE_DIR);
const report=JSON.parse(fs.readFileSync(path.join(input,'report.json'),'utf8'));
assert.equal(report.status,'PASS');assert.equal(report.native_aliases,3);assert.equal(report.wasm,'PASS');
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-numeric-workbench-'));
const rows=[];
for(const row of report.rows){
  const source=path.join(input,row.id+'.ddn');
  const bytes=fs.readFileSync(source);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),row.source_sha256);
  const run=spawnSync(process.execPath,['tests/alpha3_numeric_display_browser_runner.mjs'],{
    env:{...process.env,DDN_OPEN_SOURCE_FILE:'1',DDN_THREE_MADI_FLOW:row.id==='successive-madis'?'1':'0',DDN_TENSOR_PHYSICAL_CASE:'',DDN_EXPECTED_ERROR_CODE:'',DDN_DISPLAY_SOURCE_FILE:source,DDN_EXPECTED_DISPLAY:JSON.stringify(row.display),DDN_DISPLAY_DECISION:'A3-P1/A3-P3 PROVISIONAL'},
    encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:8*1024*1024});
  fs.writeFileSync(path.join(evidence,row.id+'.log'),run.stdout+'\n'+run.stderr);
  assert.ifError(run.error);assert.equal(run.status,0,run.stderr);
  const child=JSON.parse(run.stdout.trim().split('\n').at(-1));
  assert.equal(child.status,'PASS');assert.equal(child.result.state_hash,row.state_hash,row.id+'/Native3-WASM-Workbench world');
  rows.push({id:row.id,source_sha256:row.source_sha256,state_hash:row.state_hash,wasm_sha256:child.wasm_sha256,browser_evidence:child.evidence_directory});
}
assert.equal(rows.length,report.rows.length);
const rejectionRows=[];
for(const row of report.rejection_rows){
  const source=path.join(input,row.id+'.ddn');
  assert.equal(createHash('sha256').update(fs.readFileSync(source)).digest('hex'),row.source_sha256);
  const run=spawnSync(process.execPath,['tests/alpha3_numeric_display_browser_runner.mjs'],{
    env:{...process.env,DDN_OPEN_SOURCE_FILE:'1',DDN_THREE_MADI_FLOW:'0',DDN_TENSOR_PHYSICAL_CASE:'',DDN_DISPLAY_SOURCE_FILE:source,DDN_EXPECTED_ERROR_CODE:row.code,DDN_DISPLAY_DECISION:'A3-P1/A3-P3 PROVISIONAL'},
    encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:8*1024*1024});
  fs.writeFileSync(path.join(evidence,row.id+'.log'),run.stdout+'\n'+run.stderr);
  assert.ifError(run.error);assert.equal(run.status,0,run.stderr);
  const child=JSON.parse(run.stdout.trim().split('\n').at(-1));
  assert.equal(child.status,'PASS');assert.equal(child.result.code,row.code);
  rejectionRows.push({id:row.id,code:row.code,frame_preserved:child.result.frame_preserved,browser_evidence:child.evidence_directory});
}
const result={status:'PASS',scope:'9 raw-file positives including Run/Pause/Step three-madi progression, and 6 typed atomic rejects',native_wasm_report:input,rows,rejection_rows:rejectionRows};
fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify(result,null,2));
console.log(JSON.stringify({...result,evidence_directory:evidence}));
