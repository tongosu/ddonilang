import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const wasmDir=process.env.DDN_WASM_DIR, native=process.env.DDN_AUTHORED_NATIVE;
assert.ok(wasmDir && native,'fresh WASM and Native paths required');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-authored-project-'));
const request=JSON.stringify({source_profile:'v1-core-v25',module_name:'앱',source_snapshot:{entry_path:'main.ddn',files:[{path:'main.ddn',text:'값 := 1. 값 보여주기.'}]}});
function cli(operation,text,name) {
 const file=path.join(evidence,name);fs.writeFileSync(file,text);
 const result=spawnSync(native,['작품자료',operation,file],{encoding:'utf8',timeout:30000});
 assert.equal(result.status,0,result.stderr);return result.stdout.trim();
}
const fromWasm=wasm.authored_project_operation('create',request);
assert.equal(cli('열기',fromWasm,'wasm-project.json'),fromWasm);
const fromNative=cli('만들기',request,'request.json');
assert.equal(wasm.authored_project_operation('open',fromNative),fromNative);
const original=JSON.parse(fromWasm), clone=JSON.parse(cli('복제',fromWasm,'original.json'));
const updatedSnapshot=structuredClone(original.source_snapshot);
updatedSnapshot.files[0].text='값 := 2. 값 보여주기.';
const updateRequest=JSON.stringify({original_json:fromWasm,source_snapshot:updatedSnapshot});
const updatedText=wasm.authored_project_operation('update',updateRequest);
assert.equal(cli('갱신',updateRequest,'update-request.json'),updatedText);
const updated=JSON.parse(updatedText);
assert.deepEqual(updated.authored_identity,original.authored_identity);
assert.deepEqual(updated.source_snapshot,updatedSnapshot);
assert.notEqual(updated.content_sha256,original.content_sha256);
assert.equal(wasm.authored_project_operation('open',updatedText),updatedText);
const multiRequest=JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:{
  root_path:'selected.ddn',entry_path:'app/main.ddn',files:[
    {path:'selected.ddn',text:'(글판="v25", 모듬=차림[(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn"),(이름=힘, 머리="lib/힘.ddn", 원문="lib/힘.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음.'},
    {path:'app/main.ddn',text:'실행:움직씨 := { 1 보여주기. }.'},
    {path:'lib/힘.ddn',text:'힘 := 2.'},
  ],
}});
const multiText=wasm.authored_project_operation('create',multiRequest),multi=JSON.parse(multiText);
assert.equal(cli('열기',multiText,'multi.json'),multiText);
assert.equal(multi.authored_identity.modules.length,2);
assert.ok(multi.authored_identity.modules.every(module=>!module.sources.includes('selected.ddn')));
const multiEdited=structuredClone(multi.source_snapshot);multiEdited.files[2].text='힘 := 3.';
const multiUpdate=JSON.stringify({original_json:multiText,source_snapshot:multiEdited});
const multiUpdatedText=cli('갱신',multiUpdate,'multi-update.json');
assert.equal(wasm.authored_project_operation('update',multiUpdate),multiUpdatedText);
assert.deepEqual(JSON.parse(multiUpdatedText).authored_identity,multi.authored_identity);
fs.writeFileSync(path.join(evidence,'multi-request.json'),multiRequest);
const rootlessSnapshot=structuredClone(multi.source_snapshot);
delete rootlessSnapshot.root_path;
rootlessSnapshot.files=rootlessSnapshot.files.filter(file=>file.path!=='selected.ddn');
const rootlessUpdate=JSON.stringify({original_json:multiText,source_snapshot:rootlessSnapshot});
const rootlessFile=path.join(evidence,'rootless-update.json');fs.writeFileSync(rootlessFile,rootlessUpdate);
const rootlessResult=spawnSync(native,['작품자료','갱신',rootlessFile],{encoding:'utf8',timeout:30000});
assert.notEqual(rootlessResult.status,0,'removing the root must not bypass module resolution');
assert.equal(rootlessResult.stdout,'');
assert.throws(()=>wasm.authored_project_operation('update',rootlessUpdate));
assert.notEqual(clone.authored_identity.logical_dependency_id,original.authored_identity.logical_dependency_id);
assert.notEqual(clone.authored_identity.modules[0].id,original.authored_identity.modules[0].id);
assert.equal(clone.authored_identity.clone_origin,original.authored_identity.logical_dependency_id);
assert.deepEqual(clone.source_snapshot,original.source_snapshot);
assert.equal(wasm.authored_project_operation('open',JSON.stringify(clone)),JSON.stringify(clone));
// A valid checksum is not authority to admit a source with no module owner.
function canonical(value) {
 if(Array.isArray(value)) return value.map(canonical);
 if(value && typeof value==='object') return Object.fromEntries(Object.keys(value).sort().map(key=>[key,canonical(value[key])]));
 return value;
}
const unowned=JSON.parse(fromWasm);
unowned.source_snapshot.files.push({path:'extra.ddn',text:'2 보여주기.'});
delete unowned.content_sha256;
unowned.content_sha256=createHash('sha256').update(JSON.stringify(canonical(unowned))).digest('hex');
const rootless=JSON.parse(multiText);rootless.source_snapshot=rootlessSnapshot;
delete rootless.content_sha256;
rootless.content_sha256=createHash('sha256').update(JSON.stringify(canonical(rootless))).digest('hex');
for(const [index,text] of [fromWasm.replace('값 := 1.','값 := 2.'),`{"schema":"bad",${fromWasm.slice(1)}`,`{"unknown":1,${fromWasm.slice(1)}`,'\ufeff'+fromWasm,JSON.stringify(unowned),JSON.stringify(rootless)].entries()) {
 assert.throws(()=>wasm.authored_project_operation('open',text));
 const file=path.join(evidence,`bad${index}.json`);fs.writeFileSync(file,text);
 const result=spawnSync(native,['작품자료','열기',file],{encoding:'utf8',timeout:30000});
 assert.notEqual(result.status,0);assert.equal(result.stdout,'');
}
assert.equal(fs.readFileSync(path.join(evidence,'original.json'),'utf8'),fromWasm);
console.log(JSON.stringify({result:'AUTHORED_PROJECT_FILE_PARITY_PASS',native_aliases:1,actual_wasm:true,evidence,
 scope:'explicit creation, edit save, multi-module ownership, exact file reopen, cloning, corruption rejection; Workbench UI and view activation NOT_RUN'}));
