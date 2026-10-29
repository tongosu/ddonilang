// Read actual retained Native SQLite bytes and pass them unchanged through
// the WASM product source-only frontdoor. SQLite extraction is not lowering.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const [wasmDir,database,outputRoot,priorWasmDir]=process.argv.slice(2);assert(wasmDir&&database&&outputRoot);
const out=fs.mkdtempSync(path.join(outputRoot,'VC04_원문재읽기_WASM_'));
const sha=b=>createHash('sha256').update(b).digest('hex');
const sql=String.raw`import sqlite3,sys,json,pathlib,hashlib
db=sqlite3.connect(pathlib.Path(sys.argv[1]).resolve().as_uri()+"?mode=ro",uri=True)
# Select the actual immutable initial source even when later source revisions
# exist. This test checks these exact saved bytes, not chain/first-use authority.
rows=db.execute("SELECT project_identity,expected_parent_history_revision_identity,new_history_revision_identity,execution_profile_identity,project_execution_snapshot_identity,revision_integrity_sha256,revision_envelope_bytes,project_payload_bytes FROM vc04_prepared_payloads WHERE payload_kind=0 AND expected_parent_history_revision_identity IS NULL LIMIT 2").fetchall()
assert len(rows)==1
row=rows[0]
print(json.dumps(dict(project=row[0].hex(),parent=(row[1] or b'').hex(),revision=row[2].hex(),profile=row[3].hex(),pesi=row[4].hex(),integrity=row[5].hex(),envelope=row[6].hex(),payload=row[7].decode('utf-8'),payload_sha256=hashlib.sha256(row[7]).hexdigest()),ensure_ascii=True))
db.close()`;
const extraction=spawnSync('python',['-c',sql,database],{encoding:'utf8',windowsHide:true,timeout:30000,maxBuffer:8e6});
assert.ifError(extraction.error);assert.equal(extraction.status,0,extraction.stderr);
const native=JSON.parse(extraction.stdout),bytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
assert.equal(sha(Buffer.from(native.payload)),native.payload_sha256,'SQL raw UTF-8 payload must survive process transport byte-identically');
fs.writeFileSync(path.join(out,'NATIVE_INPUT.json'),JSON.stringify(native));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);await wasm.default({module_or_path:bytes});
const raw=hex=>new Uint8Array(Buffer.from(hex,'hex'));
const call=(payload=native.payload,pesi=native.pesi)=>wasm.wasm_restore_recorded_source_revision(payload,raw(native.envelope),raw(native.project),raw(native.parent),raw(native.revision),raw(native.integrity),raw(native.profile),raw(pesi));
const report={scope:'NATIVE_ACTUAL_SAVED_SOURCE_BYTES_TO_ACTUAL_WASM_READBACK_NOT_CLI3_WORKBENCH_OR_FIRST_RUN',database,
 wasm_sha256:sha(bytes),payload_sha256:sha(Buffer.from(native.payload)),envelope_sha256:sha(Buffer.from(native.envelope,'hex')),checks:[]};
try {
 const expected=JSON.parse(native.payload).authored_project_json_utf8;
 if(priorWasmDir){
  const prior=await import(pathToFileURL(path.join(priorWasmDir,'ddonirang_tool.js')).href);
  const priorBytes=fs.readFileSync(path.join(priorWasmDir,'ddonirang_tool_bg.wasm'));await prior.default({module_or_path:priorBytes});
  let rejected=null;try{prior.wasm_restore_recorded_source_revision(native.payload,raw(native.envelope),raw(native.project),raw(native.parent),raw(native.revision),raw(native.integrity),raw(native.profile),raw(native.pesi));}catch(e){rejected=String(e);}
  assert(rejected?.includes('E_VC04_REGISTRATION'),rejected??'prior WASM unexpectedly accepted the new registration profile');
  report.checks.push({id:'prior_product_lacks_same_raw_registered_type',result:'EXPECTED_REJECTION',wasm_sha256:sha(priorBytes),error:rejected});
 }
 assert.equal(call(),expected);assert.equal(call(),expected);
 report.checks.push({id:'exact_recorded_source_and_module_identity',result:'PASS'});
 const changed=JSON.parse(native.payload);assert(changed.registration_resources.length);
 const record=changed.registration_resources[0];record.raw_hex=record.raw_hex.slice(0,-2)+(record.raw_hex.endsWith('00')?'01':'00');
 let rejected=null;try{call(JSON.stringify(changed));}catch(e){rejected=String(e);}
 assert(rejected?.includes('E_VC04_REGISTRATION'),rejected);
 report.checks.push({id:'changed_registration_blob',result:'REJECTED',error:rejected});
 const badPesi=Buffer.from(native.pesi,'hex');badPesi[0]^=1;
 rejected=null;try{call(native.payload,badPesi.toString('hex'));}catch(e){rejected=String(e);}
 assert(rejected?.includes('E_SOURCE_REVISION_READBACK'),rejected);
 report.checks.push({id:'different_expected_pesi',result:'REJECTED',error:rejected});
 assert.equal(call(),expected);report.checks.push({id:'original_exact_input_again',result:'PASS'});
 report.verdict='PASS_RECORDED_SOURCE_READBACK_AND_EXPLICIT_REJECTION_NO_WORLD_AUTHORITY';
}catch(e){report.verdict='FAIL';report.error=String(e.stack??e);process.exitCode=1;}
finally{fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({out,verdict:report.verdict,error:report.error}));}
