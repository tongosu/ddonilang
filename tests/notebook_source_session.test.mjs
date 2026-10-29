import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createNotebookSourceSession} from '../solutions/seamgrim_ui_mvp/ui/workbench/persistence/notebook_source.js';
const raw=readFileSync(new URL('../solutions/seamgrim_ui_mvp/first_steps_rc1/v2_structure_identity_r1/모듈/canonical-작품자료.json',import.meta.url),'utf8');
const id=JSON.parse(raw).authored_identity.logical_dependency_id;
const receipt=JSON.stringify({native_source_project_id:id,authored_project_json_utf8:raw});
function fakeStore(){let stored='',fail=false;return {get stored(){return stored;},set fail(v){fail=v;},async save(text,guard){guard();if(fail)throw Error('실제 저장 실패 자리');stored=text;return {locator:receipt};},async reopen(projectId,guard){guard();assert.equal(projectId,id);return stored;}};}
const session=store=>createNotebookSourceSession({sourceStorage:store,nativeAvailable:async()=>true});
// These tests isolate controller publication/ownership. They do not substitute
// a fake storage result for actual Native or mathematical admission evidence.
test('raw source is preserved; reopen drops reports and requires explicit recheck',async()=>{
 const store=fakeStore(),a=session(store);a.setSourceProject(raw);let checks=0;
 await a.recheck(async text=>{checks++;assert.equal(text,raw);return {sourceReport:'observation only'};});
 assert.equal(checks,1);assert.equal(a.snapshot().phase,'inspected');const saved=await a.save();assert.equal(store.stored,raw);
 const b=session(store);await b.open(saved);assert.equal(b.snapshot().sourceProjectText,raw);assert.equal(b.snapshot().needsRecheck,true);assert.equal(b.snapshot().report,null);assert.equal(checks,1);
 await b.recheck(async text=>{checks++;return {bytes:text.length};});assert.equal(checks,2);
});
test('failed save keeps the last good source and does not publish success',async()=>{
 const store=fakeStore(),a=session(store);a.setSourceProject(raw);const saved=await a.save();store.fail=true;
 await assert.rejects(a.save(),/실제 저장 실패/);assert.equal(a.snapshot().phase,'save-failed');assert.equal(a.snapshot().hasSavedSource,true);assert.equal(store.stored,raw);
 const b=session(store);await b.open(saved);assert.equal(b.snapshot().sourceProjectText,raw);
});
test('edit during pending save cannot publish an old response for new source',async()=>{
 let finish;const a=session({save:()=>new Promise(r=>finish=r)});a.setSourceProject(raw);
 const pending=a.save();await new Promise(r=>setTimeout(r,0));a.setSourceProject(raw+'\n');finish({locator:receipt});
 await assert.rejects(pending,/편집판/);assert.equal(a.snapshot().phase,'save-failed');assert.equal(a.snapshot().hasSavedSource,false);assert.equal(a.snapshot().needsRecheck,true);
});
test('late inspection is not attached to an edited source',async()=>{
 const a=session(fakeStore());a.setSourceProject(raw);let finish;
 const pending=a.recheck(()=>new Promise(r=>finish=r));a.setSourceProject(raw+'\n');finish({success:true});
 await assert.rejects(pending,/편집판/);assert.equal(a.snapshot().report,null);assert.equal(a.snapshot().needsRecheck,true);
});
test('absence of Native support never falls back to another store',async()=>{
 let called=0;const a=createNotebookSourceSession({sourceStorage:{async save(){called++;}},nativeAvailable:async()=>false});a.setSourceProject(raw);
 await assert.rejects(a.save(),/E_NATIVE_SOURCE_HOST_REQUIRED/);assert.equal(called,0);assert.equal(a.snapshot().hasSavedSource,false);
});
test('different project and overlapping operations are refused',async()=>{
 const a=session(fakeStore());a.setSourceProject(raw);
 await assert.rejects(a.open(JSON.stringify({native_source_project_id:'another'})),/다른 노트/);
 let finish;const pending=a.recheck(()=>new Promise(r=>finish=r));await assert.rejects(a.save(),/진행 중/);finish({viewOnly:true});await pending;
 const changed=JSON.parse(raw);changed.authored_identity.logical_dependency_id='another';assert.throws(()=>a.setSourceProject(JSON.stringify(changed)),/다른 노트/);
});

test('a failing renderer cannot block or relabel a Native source save',async()=>{
 const store=fakeStore(),a=session(store);a.subscribe(()=>{throw Error('화면 표시 실패');});
 a.setSourceProject(raw);const saved=await a.save();assert.equal(saved,receipt);assert.equal(store.stored,raw);
 assert.equal(a.snapshot().phase,'saved');assert.equal(a.snapshot().currentSourceSaved,true);assert.match(a.snapshot().viewError,/화면 표시 실패/);
});

// Controller reference validation only. Mathematical units/admission remain
// the common producer's responsibility; these tests mint no such evidence.
function sourceContext(text=raw){
 const source=JSON.parse(text).source_snapshot;
 const file=source.files.find(f=>f.path===source.entry_path)??source.files[0];
 const ref=f=>({path:f.path,startByte:0,endByte:Buffer.byteLength(f.text),text:f.text});
 const dep=source.files.find(f=>f.path==='ddn.lock')??source.files.find(f=>f.path===(source.root_path??source.entry_path));
 return {inputs:[ref(file)],units:[ref(file)],assumptions:[ref(file)],dependencies:[ref(dep)]};
}
test('notebook save requires exact source-bound context and whole dependency source',async()=>{
 const store=fakeStore(),a=session(store);a.setSourceProject(raw);
 await assert.rejects(a.saveNotebook(),/먼저 결속/);assert.equal(store.stored,'');
 const context=sourceContext();a.bindContext(context);await a.saveNotebook();assert.equal(store.stored,raw);
 assert.equal(a.snapshot().notebookContext.mathematicalUnitCheckIssued,false);
 const bad=structuredClone(context);bad.inputs[0].text+='다른 값';
 assert.throws(()=>a.bindContext(bad),/현재 원문과 다릅니다/);
 const partial=structuredClone(context);partial.dependencies[0].endByte--;
 partial.dependencies[0].text=Buffer.from(partial.dependencies[0].text).subarray(0,-1).toString();
 assert.throws(()=>a.bindContext(partial),/전체 lock|범위|encoded data/);
});
test('dependency edit archives the old report, clears context and demands recheck',async()=>{
 const a=session(fakeStore());a.setSourceProject(raw);a.bindContext(sourceContext());
 await a.recheck(async()=>({observation:'old'}));
 const next=JSON.parse(raw),dep=sourceContext().dependencies[0].path;
 next.source_snapshot.files.find(f=>f.path===dep).text+='\n';
 // Controller guard specimen only; no claim that this is an admitted artifact.
 a.setSourceProject(JSON.stringify(next));const state=a.snapshot();
 assert.equal(state.needsRecheck,true);assert.equal(state.contextNeedsBinding,true);
 assert.equal(state.report,null);assert.equal(state.pastReports.length,1);
 assert.equal(state.pastReports[0].label,'과거 결과');assert.equal(state.pastReports[0].usableAsCurrent,false);
 assert.equal(state.pastReports[0].sourceProjectText,raw);assert.equal(state.historyIsPersisted,false);
});
test('cold source open never restores a historical report as current authority',async()=>{
 const store=fakeStore(),a=session(store);a.setSourceProject(raw);a.bindContext(sourceContext());
 await a.recheck(async()=>({observation:true}));const saved=await a.saveNotebook();
 const b=session(store);await b.open(saved);
 assert.equal(b.snapshot().contextNeedsBinding,true);assert.equal(b.snapshot().needsRecheck,true);
 assert.equal(b.snapshot().pastReports.length,0);
 await b.recheck(async()=>({notebookContext:sourceContext(),observation:'fresh'}));
 assert.equal(b.snapshot().contextNeedsBinding,false);assert.equal(b.snapshot().report.observation,'fresh');
});
test('UTF-8 split references and stale supplied contexts are refused',async()=>{
 const a=session(fakeStore());a.setSourceProject(raw);const c=sourceContext();
 const f=JSON.parse(raw).source_snapshot.files.find(f=>/[^\x00-\x7f]/.test(f.text));
 const before=f.text.search(/[^\x00-\x7f]/),start=Buffer.byteLength(f.text.slice(0,before));
 c.inputs=[{path:f.path,startByte:start+1,endByte:start+2,text:'x'}];
 assert.throws(()=>a.bindContext(c));
 let finish;const pending=a.recheck(()=>new Promise(r=>finish=r));
 a.setSourceProject(raw+'\n');finish({notebookContext:sourceContext(),observation:'late'});
 await assert.rejects(pending,/편집판/);assert.equal(a.snapshot().report,null);
 assert.equal(a.snapshot().notebookContext,null);
});
