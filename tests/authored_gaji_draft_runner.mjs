import assert from 'node:assert/strict';
import fs from 'node:fs';
import {pathToFileURL} from 'node:url';
import {webcrypto} from 'node:crypto';
import {prepareAuthoredGajiDraft} from '../solutions/seamgrim_ui_mvp/ui/workbench/authored_gaji_draft.js';
import {createGajiController} from '../solutions/seamgrim_ui_mvp/ui/workbench/gaji_controller.js';
const dir=process.argv[2];if(!dir)throw new Error('actual fixed A/B inputs and WASM root required');
const wasm=process.argv[3];globalThis.crypto??=webcrypto;
const api=await import(pathToFileURL(wasm+'/ddonirang_tool.js'));await api.default({module_or_path:fs.readFileSync(wasm+'/ddonirang_tool_bg.wasm')});
const read=name=>JSON.parse(fs.readFileSync(dir+'/'+name+'.json','utf8'));
const artifact=read('A1-artifact'),source=read('A2-source');let calls=[];
const bridge={authoredProjectOperation:async request=>{calls.push(request.operation);return api.authored_project_operation(request.operation,request.text);}};
const before=JSON.stringify(artifact);
const projected=await prepareAuthoredGajiDraft({artifact,source,bridge,assertCurrent(){}});
assert.deepEqual(projected,JSON.parse(api.authored_project_operation('source_project',JSON.stringify(read('A2-artifact')))));
assert.equal(JSON.stringify(artifact),before);assert.deepEqual(calls,['update','source_project']);
let checks=0;
await assert.rejects(prepareAuthoredGajiDraft({artifact,source,bridge,assertCurrent(){if(++checks===1)throw new Error('stale context');}}),/stale context/);
await assert.rejects(prepareAuthoredGajiDraft({artifact,source,assertCurrent(){},bridge:{authoredProjectOperation:async request=>{
 const value=JSON.parse(await bridge.authoredProjectOperation(request));if(request.operation==='source_project')value.files[0].utf8_text+='forged';return JSON.stringify(value);
}}}),/E_PROJECT_RUNTIME_SOURCE_SUPPLY/);
await assert.rejects(prepareAuthoredGajiDraft({artifact,source:{...source,root_path:'other.ddn'},bridge,assertCurrent(){}}),/E_PROJECT_EDIT_FAILED/);
// Exercise the actual controller with the actual common WASM plan owner.
let selected=read('A1-source');const state={projectId:'A',projectEditGeneration:0,serverProject:{role:''},projectGraph:{graph:{gaji_aliases:{힘:'vendor/운동'}}}};
const presentation={};let applied=0;let duringPlan=()=>{};
globalThis.document={querySelectorAll:()=>[]};
const els={gajiInstallAlias:{value:'힘'},gajiInstallPreview:{},gajiInstallStatus:{},gajiInstallApply:{},gajiInstallRemove:{},gajiRemixStart:{}};
const second=read('helper2'),row={package_sha256:second.package_sha256,public_symbols:['속도'],...second};
const controller=createGajiController({currentProject:()=>({authoredArtifact:artifact}),currentVirtualProjectSource:()=>selected,
 applyVirtualProjectSource:async value=>{assert.deepEqual(value,source);applied++;},state,els,workbenchGajiPresentationState:presentation,
 workbenchProjectRegistryState:{nativeProjects:[]},loadLocalGotganStore:()=>({artifacts:[second]}),pushDiagnostic(){},refreshProjectGraph:async()=>{},
 runtimeBridge:{projectSourceSha256:async value=>api.wasm_project_source_sha256_json(JSON.stringify(value)),planGajiLifecycle:async request=>{
  const result=JSON.parse(api.wasm_gaji_lifecycle_plan_json(JSON.stringify(request)));duringPlan();return result;
 }},});
const button={classList:{}};
await controller.previewShelfInstall(row,button);assert.equal(els.gajiInstallApply.disabled,false);assert.equal(presentation.pendingGajiInstall.plan.operation,'update');
await controller.applyShelfInstall();assert.equal(applied,1);
await controller.previewShelfInstall(row,button);state.projectId='B';await assert.rejects(controller.applyShelfInstall(),/E_PROJECT_EDIT_SOURCE_CONFLICT/);assert.equal(applied,1);
state.projectId='A';duringPlan=()=>{state.projectEditGeneration++;};await assert.rejects(controller.previewShelfInstall(row,button),/E_PROJECT_EDIT_SOURCE_CONFLICT/);assert.equal(applied,1);
console.log('PASS: actual common update/projection and authored shelf apply; source untouched on stale/foreign bytes/root change; Native save not fabricated');
