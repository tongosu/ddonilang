// Exercise the real Native CLI product; the harness never lowers DDN source.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const [binDir,wasmDir,outRoot,...references]=process.argv.slice(2);
assert(binDir&&wasmDir&&outRoot&&references.length);
const out=fs.mkdtempSync(path.join(outRoot,'VC04_CLI_저장종료재열기_'));
const sha=v=>createHash('sha256').update(v).digest('hex');
const names=['ddonilang','ddonirang','또니랑'];
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const report={scope:'ACTUAL_RELEASE_CLI_PROCESSES_NOT_CHROME_OR_COMPLETE_ROW',checks:[],executables:names.map(n=>({name:n,sha256:sha(fs.readFileSync(path.join(binDir,n+'.exe')))}))};
let sequence=0;
function call(name,args,ok=true){
 const result=spawnSync(path.join(binDir,name+'.exe'),['작품자료',...args],{encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:32e6});
 const stem=path.join(out,String(++sequence).padStart(2,'0')+'_'+name);
 fs.writeFileSync(stem+'.stdout',result.stdout??'');fs.writeFileSync(stem+'.stderr',result.stderr??'');
 assert.ifError(result.error);
 if(ok)assert.equal(result.status,0,result.stderr);else assert.notEqual(result.status,0,'must explicitly reject');
 return ok?JSON.parse(result.stdout):result.stderr;
}
const projection=x=>{
 assert.equal(typeof x.state_hash,'string');assert(x.resources&&x.output_display);
 assert.equal(typeof x.next_tick_id,'number');
 return {state_hash:x.state_hash,resources:x.resources,output:x.output_display.texts,next_tick:x.next_tick_id};
};
try{
 for(const [index,reference] of references.entries()){
  const raw=fs.readFileSync(path.join(reference,'saved-artifact.json'),'utf8');
  const original=JSON.parse(raw),authored=JSON.parse(original.project_snapshot.authored_project_json_utf8);
  const snapshot=authored.source_snapshot,sourceHash=sha(JSON.stringify(snapshot));
  // Explicit product create supplies a new project identity, unchanged source bytes.
  const created=wasm.authored_project_operation('create',JSON.stringify({source_profile:authored.source_profile??'v1-core-v25',module_name:'',source_snapshot:snapshot}));
  assert.equal(sha(JSON.stringify(JSON.parse(created).source_snapshot)),sourceHash);
  const projectFile=path.join(out,`project-${index}.json`);fs.writeFileSync(projectFile,created);
  const store=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-cli-source-'));
  const locator=call(names[0],['원문저장',projectFile,'--판','01','--저장소',store]);
  const locatorFile=path.join(out,`locator-${index}.json`);fs.writeFileSync(locatorFile,JSON.stringify(locator));
  const forged=path.join(out,`forged-${index}.json`);fs.writeFileSync(forged,JSON.stringify({...locator,permit:true}));
  assert.match(call(names[1],['첫실행',forged,'--판','02','--저장소',store,'--마디수','1'],false),/E_SOURCE_LOCATOR_FIELDS/);
  const first=call(names[1],['첫실행',locatorFile,'--판','02','--저장소',store,'--마디수','1']);
  const savedFile=path.join(out,`saved-${index}.json`);fs.writeFileSync(savedFile,first.saved_artifact_json_utf8);
  const savedHash=sha(fs.readFileSync(savedFile));
  const cold=names.map(name=>call(name,['원문이어실행',savedFile,'--부모','01','--판','02','--저장소',store,'--허가',locator.grant_locator,'--마디수','1']));
  cold.forEach(x=>assert.deepEqual(projection(x),projection(cold[0])));
  assert.equal(sha(fs.readFileSync(savedFile)),savedHash,'cold reads do not rewrite saved bytes');
  const reused=call(names[2],['첫실행',locatorFile,'--판','03','--저장소',store,'--마디수','1'],false);
  assert.match(reused,/E_SOURCE_|E_VC04_/);
  // Same-bootstrap uninterrupted reference was recorded by the real Native owner.
  const expected=JSON.parse(fs.readFileSync(path.join(reference,'cold-result.json'),'utf8'));
  const grant=fs.readFileSync(path.join(reference,'grant-locator.txt'),'utf8').trim();
  const baseline=names.map(name=>call(name,['원문이어실행',path.join(reference,'saved-artifact.json'),'--부모','01','--판','02','--저장소',reference,'--허가',grant,'--마디수','1']));
  baseline.forEach(x=>assert.deepEqual(projection(x),projection(expected)));
  assert.equal(fs.readFileSync(path.join(reference,'saved-artifact.json'),'utf8'),raw);
  report.checks.push({reference,store,source_snapshot_sha256:sourceHash,saved_sha256:savedHash,cli3_cold_equal:true,actual_same_bootstrap_reference_equal:true,new_project_state_hash:cold[0].state_hash,reference_state_hash:expected.state_hash,forged_report_rejected:true,first_use_reuse_rejected:reused});
 }
 report.verdict='PASS';
}catch(e){report.verdict='FAIL';report.error=String(e.stack??e);process.exitCode=1;}
finally{fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({out,verdict:report.verdict,error:report.error}));}
