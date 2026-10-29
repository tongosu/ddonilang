import {test} from 'node:test';
import assert from 'node:assert/strict';
import {installMultilangHost} from '../solutions/seamgrim_ui_mvp/ui/workbench/multilang/host.js';
// DOM/codec/storage doubles test adapter refusal and custody only, not Native/DDL authority.
class Element {
  constructor(tag,doc){this.tag=tag;this.ownerDocument=doc;this.children=[];this.dataset={};this.events={};this.textContent='';this.value='';}
  append(...nodes){for(const node of nodes){node.parent=this;this.children.push(node);}}
  replaceChildren(...nodes){this.children=[];this.append(...nodes);}
  setAttribute(){} remove(){if(this.parent)this.parent.children=this.parent.children.filter(n=>n!==this);}
  addEventListener(name,fn){this.events[name]=fn;} removeEventListener(name){delete this.events[name];}
  emit(name){this.events[name]?.();}
}
const walk=root=>[root,...root.children.flatMap(walk)];
const settle=async()=>{for(let i=0;i<12;i++)await new Promise(setImmediate);};
function fixture({multi=false,native=async()=>true,storage:override}={}){
  const document={createElement:tag=>new Element(tag,document)},root=new Element('root',document);
  const artifact={source_profile:'profile:test',authored_identity:{logical_dependency_id:'test-only-project'},
    source_snapshot:{entry_path:'main.ddn',files:[{path:'main.ddn',text:'old'},...(multi?[{path:'other.ddn',text:'preserve'}]:[])],...(multi?{root_path:'root.ddn'}:{})}};
  let raw={projectId:'ui-only',path:'main.ddn',profile:'profile:test',artifactText:JSON.stringify(artifact),
    sourceSnapshot:structuredClone(artifact.source_snapshot),locator:JSON.stringify({native_source_project_id:'test-only-project',authoring_selection:{testOnly:true}}),running:false};
  const calls=[];let saved='';
  const bridge={authoredProjectOperation:async({operation,text})=>{
    if(operation==='open')return text;
    const request=JSON.parse(text);return JSON.stringify({...JSON.parse(request.original_json),source_snapshot:request.source_snapshot});
  },sourceAnalysis:async()=>{calls.push('analysis');throw Error('not used in refusal tests');}};
  const storage=override??{save:async(text,check)=>{check();saved=text;calls.push(JSON.parse(text));
    return {locator:JSON.stringify({native_source_project_id:'test-only-project',authored_project_json_utf8:text})};},
    reopen:async()=>{calls.push('reopen');return saved;}};
  const host=installMultilangHost({root,runtimeBridge:bridge,readContext:()=>raw,storage,nativeAvailable:native});
  return {root,host,calls,get raw(){return raw;},set raw(value){raw=value;},
    node:label=>walk(root).find(n=>n.textContent===label),
    edit:text=>{const editor=walk(root).find(n=>n.tag==='textarea');editor.value=text;editor.emit('input');}};
}
test('전체 artifact와 root/다른 파일을 보존하며 선택 원문만 common update에 전달한다',async()=>{
  const f=fixture({multi:true});await f.host.loadCurrent();f.edit('new');f.node('원문 저장').emit('click');await settle();
  const saved=f.calls.find(value=>typeof value==='object');
  assert.equal(saved.source_snapshot.root_path,'root.ddn');assert.equal(saved.source_snapshot.files.length,2);
  assert.equal(saved.source_snapshot.files[0].text,'new');assert.equal(saved.source_snapshot.files[1].text,'preserve');
  assert.equal(f.raw.sourceSnapshot.files[0].text,'old');f.host.dispose();
});
test('Native 미가용은 브라우저 저장으로 성공시키지 않는다',async()=>{
  const f=fixture({native:async()=>false});await f.host.loadCurrent();f.node('원문 저장').emit('click');await settle();
  assert.equal(f.calls.length,0);assert(walk(f.root).some(n=>n.textContent.includes('E_MULTILANG_NATIVE_REQUIRED')));f.host.dispose();
});
test('artifact 없는 원문에서 정체/root/profile을 만들지 않는다',async()=>{
  const f=fixture();f.raw={...f.raw,artifactText:''};await f.host.loadCurrent();
  assert.equal(f.node('원문 저장').disabled,true);assert(walk(f.root).some(n=>n.textContent.includes('E_MULTILANG_PROJECT_REQUIRED')));f.host.dispose();
});
test('Native 응답이 아닌 저장 locator는 성공 receipt로 발행하지 않는다',async()=>{
  const f=fixture({storage:{save:async()=>({locator:JSON.stringify({source_project_id:'test-only-project'})})}});
  await f.host.loadCurrent();f.node('원문 저장').emit('click');await settle();
  assert(walk(f.root).some(n=>n.textContent.includes('E_MULTILANG_NATIVE_READBACK')));
  assert.equal(f.node('저장본 열고 재검사').disabled,true);f.host.dispose();
});
test('여러 파일을 raw 단일 분석으로 평탄화하지 않는다',async()=>{
  const f=fixture({multi:true});await f.host.loadCurrent();f.node('검사').emit('click');await settle();
  assert.equal(f.calls.length,0);assert(walk(f.root).some(n=>n.textContent.includes('E_MULTILANG_BUNDLE_ANALYSIS_UNSUPPORTED')));f.host.dispose();
});
test('async 준비 중 작품 변경은 Native 쓰기 전에 거부한다',async()=>{
  let resolve;const f=fixture({native:()=>new Promise(r=>{resolve=r;})});await f.host.loadCurrent();
  f.node('원문 저장').emit('click');await settle();f.raw={...f.raw,projectId:'other'};resolve(true);await settle();
  assert.equal(f.calls.length,0);assert(walk(f.root).some(n=>n.textContent.includes('E_MULTILANG_CONTEXT_CHANGED')));f.host.dispose();
});
test('main 작품이 바뀌면 이전 panel 저장본을 다른 작품에 재열기하지 않는다',async()=>{
  const f=fixture();await f.host.loadCurrent();f.node('원문 저장').emit('click');await settle();
  f.raw={...f.raw,projectId:'other'};f.node('저장본 열고 재검사').emit('click');await settle();
  assert(!f.calls.includes('reopen'));assert(walk(f.root).some(n=>n.textContent.includes('E_MULTILANG_CONTEXT_CHANGED')));f.host.dispose();
});
test('판본 선택이 없는 수정 저장은 Native 쓰기 전에 명시 거부한다',async()=>{
  const f=fixture();f.raw={...f.raw,locator:''};await f.host.loadCurrent();f.edit('new');
  f.node('원문 저장').emit('click');await settle();
  assert.equal(f.calls.length,0);assert(walk(f.root).some(n=>n.textContent.includes('E_MULTILANG_NATIVE_SELECTION_REQUIRED')));f.host.dispose();
});
