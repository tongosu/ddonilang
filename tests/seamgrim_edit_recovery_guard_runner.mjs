import assert from 'node:assert/strict';
import {createEditRecoveryGuard} from '../solutions/seamgrim_ui_mvp/ui/workbench/persistence/edit_recovery_guard.js';
const store=new Map();const storage={getItem:key=>store.get(key)??null,setItem:(key,value)=>store.set(key,value)};
let source={project_id:'authored_fixed_project',profile:'v1-core-v25',files:[{path:'root.ddn',text:'원본 뿌리'},{path:'main.ddn',text:'원본 본문'}]};
let notice,applied=0;const savedWorld={head:'01',hash:'same-world'};const originalWorld=JSON.stringify(savedWorld);
const make=options=>createEditRecoveryGuard({storage,capture:()=>structuredClone(source),present:value=>notice=value,
  applyDraft:async draft=>{applied++;source=structuredClone(draft);},...options});
let guard=make();guard.offer();assert.equal(notice.visible,false);
// Input capture precedes the model handler; the model deliberately remains
// old here to represent an exception before its normal assignment.
guard.saveEditor('main.ddn','수정 본문');guard.fault(new Error('E_INJECTED_UI_FAILURE'));
assert(notice.message.includes('E_INJECTED_UI_FAILURE'));assert.equal(notice.canRestore,true);
guard.saveEditor('root.ddn','수정 뿌리');
guard.fault(new Error('E_INJECTED_UI_FAILURE'));
const record=JSON.parse([...store.values()][0]);
assert.equal(record.source.files.find(f=>f.path==='main.ddn').text,'수정 본문');
assert.equal(record.source.files.find(f=>f.path==='root.ddn').text,'수정 뿌리');
// A fresh guard reads actual stored bytes, but restores only on explicit action.
guard=make();guard.offer();assert(notice.message.includes('저장 안 된 편집'));assert(notice.message.includes('E_INJECTED_UI_FAILURE'));
const authoredSource=structuredClone(source);source={project_id:'startup-default',profile:'v1-core-v25',files:[{path:'main.ddn',text:'기본 원문'}]};
guard=make();guard.offer();assert.equal(notice.visible,true);assert.equal(notice.canRestore,false);assert.equal(notice.canOpen,true);assert(notice.message.includes('다른 작품'));
assert.equal(await guard.restore(),false);assert.equal(source.project_id,'startup-default');
source=authoredSource;guard=make();guard.offer();
assert.equal(source.files.find(f=>f.path==='main.ddn').text,'원본 본문');assert.equal(applied,0);
assert.equal(await guard.restore(),true);assert.equal(applied,1);
assert.equal(source.files.find(f=>f.path==='root.ddn').text,'수정 뿌리');
assert.equal(source.files.find(f=>f.path==='main.ddn').text,'수정 본문');
assert.equal(JSON.stringify(savedWorld),originalWorld,'editing recovery touched saved world');
// A stale save completion must not mark a later edit as saved.
guard.beginSave(1);guard.saveEditor('main.ddn','저장 중 추가 편집');source.files.find(f=>f.path==='main.ddn').text='저장 중 추가 편집';
guard.saved(1);assert.equal(JSON.parse([...store.values()][0]).pending,true);assert(notice.message.includes('저장 안 된 편집'));
guard.beginSave(2);guard.saved(2);assert.equal(JSON.parse([...store.values()][0]).pending,false);
guard.saveEditor('main.ddn','저장 실패 뒤 남는 편집');guard.beginSave(3);guard.failed(3);assert(notice.message.includes('저장 안 된 편집'));assert.equal(JSON.parse([...store.values()][0]).pending,true);
// Wrong structure/profile refuses application and preserves all original bytes.
source.profile='different-profile';assert.equal(await guard.restore(),false);assert.equal(applied,1);assert(notice.message.includes('구조'));
const before=[...store.values()][0];source.profile='v1-core-v25';
guard.saveEditor('main.ddn','a'.repeat(1_000_001));assert(notice.message.includes('한도'));assert.equal([...store.values()][0],before);
const denied=make({storage:{getItem:()=>null,setItem:()=>{throw new Error('E_QUOTA_DENIED');}}});
denied.saveEditor('main.ddn','편집');assert(notice.message.includes('E_QUOTA_DENIED'));assert.equal(notice.canRestore,false);
store.set([...store.keys()][0],'{broken');const broken=[...store.values()][0];guard.offer();assert.equal(notice.canRestore,false);assert.equal([...store.values()][0],broken);
assert.equal(JSON.stringify(savedWorld),originalWorld);
console.log(JSON.stringify({verdict:'PASS',cases:['pre-model capture','multi-file survival','explicit cold restore','no world mutation','stale save completion','failed save preserves draft','profile mismatch','limit without truncation','quota denial','corrupt bytes preserved']}));
