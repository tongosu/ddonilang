import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createNotebookReproductionPair} from '../solutions/seamgrim_ui_mvp/ui/workbench/persistence/notebook_source.js';
const raw=readFileSync(new URL('../solutions/seamgrim_ui_mvp/first_steps_rc1/v2_structure_identity_r1/모듈/canonical-작품자료.json',import.meta.url),'utf8');
// Controller boundaries only. These specimens do not mint admitted project
// identities or mathematical results; Native tests use the real clone producer.
const b=JSON.parse(raw);b.authored_identity.logical_dependency_id='controller-specimen-B';const other=JSON.stringify(b);
function context(raw){const s=JSON.parse(raw).source_snapshot;const f=s.files.find(f=>f.path===s.entry_path);const dep=s.files.find(f=>f.path==='ddn.lock')??s.files.find(f=>f.path===(s.root_path??s.entry_path));const ref=f=>({path:f.path,startByte:0,endByte:Buffer.byteLength(f.text),text:f.text});return {inputs:[ref(f)],units:[ref(f)],assumptions:[ref(f)],dependencies:[ref(dep)]};}
function harness(){const saved=new Map();let failB=false;const options={nativeAvailable:async()=>true,sourceStorageFactory:slot=>({async save(text,guard){guard();if(slot==='B'&&failB)throw Error('RecoveryRequired: unknown commit');const id=JSON.parse(text).authored_identity.logical_dependency_id;saved.set(id,text);return {locator:JSON.stringify({native_source_project_id:id,authored_project_json_utf8:text})};},async reopen(id,guard){guard();if(!saved.has(id))throw Error('missing saved source');return saved.get(id);}})};return {saved,options,set failB(value){failB=value;}};}
function ready(h){const pair=createNotebookReproductionPair(h.options);for(const [slot,text] of [['A',raw],['B',other]]){pair.setSourceProject(slot,text);pair.bindContext(slot,context(text));}return pair;}
test('two distinct reproductions reopen without calculations and explicit B cannot overwrite A',async()=>{
 const h=harness(),pair=ready(h);const refs=await pair.save();assert.equal(h.saved.size,2);
 const cold=createNotebookReproductionPair(h.options);await cold.open(refs);
 for(const slot of ['A','B']){assert.equal(cold.snapshot().experiments[slot].resultStatus,'NOT_PRODUCED');assert.equal(cold.snapshot().experiments[slot].report,null);}
 await cold.recompute('A',async()=>({observation:'new A, not a mathematical certificate'}));const a=cold.snapshot().experiments.A.report;
 await cold.recompute('B',async()=>({observation:'new B, not a mathematical certificate'}));assert.deepEqual(cold.snapshot().experiments.A.report,a);assert.equal(h.saved.get(JSON.parse(raw).authored_identity.logical_dependency_id),raw);
});
test('B uncertainty preserves A acknowledgement and never announces pair success',async()=>{
 const h=harness(),pair=ready(h);h.failB=true;await assert.rejects(pair.save(),/RecoveryRequired/);
 const state=pair.snapshot();assert.equal(state.bothSourcesSaved,false);assert.deepEqual(state.confirmedThisAttempt,{A:true,B:false});assert.ok(state.locations.A);assert.equal(state.locations.B,'');assert.equal(h.saved.size,1);
});
test('same identity and incomplete B binding are rejected before any Native writes',async()=>{
 const h=harness(),pair=createNotebookReproductionPair(h.options);pair.setSourceProject('A',raw);assert.throws(()=>pair.setSourceProject('B',raw),/서로 다른/);pair.setSourceProject('B',other);pair.bindContext('A',context(raw));await assert.rejects(pair.save(),/각각 결속/);assert.equal(h.saved.size,0);
});
test('late calculation is not published after editing its source',async()=>{
 const h=harness(),pair=ready(h);let finish;const pending=pair.recompute('A',()=>new Promise(r=>finish=r));pair.setSourceProject('A',raw+'\n');finish({observation:'old'});await assert.rejects(pending,/편집판/);assert.equal(pair.snapshot().experiments.A.resultStatus,'NOT_PRODUCED');
});
test('failed staged open preserves previous locations without reviving old reports',async()=>{
 const h=harness(),pair=ready(h);const refs=await pair.save();await pair.recompute('A',async()=>({observation:'old'}));const bad={...refs,B:JSON.stringify({native_source_project_id:'missing'})};await assert.rejects(pair.open(bad),/missing/);assert.deepEqual(pair.snapshot().locations,refs);assert.equal(pair.snapshot().experiments.A.resultStatus,'NOT_PRODUCED');assert.equal(pair.snapshot().experiments.A.sourceProjectText,raw);assert.equal(pair.snapshot().bothSourcesSaved,false);
});
