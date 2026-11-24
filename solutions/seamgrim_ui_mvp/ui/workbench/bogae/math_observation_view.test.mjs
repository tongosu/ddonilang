import test from 'node:test';
import assert from 'node:assert/strict';
import {mountMathObservationView,mountProjectMathObservationView,mountProjectMathInspector} from './math_observation_view.mjs';

// UI lifetime fixtures, not runtime/checker evidence. Real producer records are
// exercised separately in the browser through the existing validation callbacks.
function root(){
  const doc={createElement(tag){return {tag,ownerDocument:doc,children:[],dataset:{},attrs:{},textContent:'',innerHTML:'',
    classList:{add(){},remove(){}},setAttribute(k,v){this.attrs[k]=v;},append(...n){this.children.push(...n);},
    replaceChildren(...n){this.children=n;},addEventListener(){},removeEventListener(){}};}};
  return doc.createElement('main');
}
const observation=tag=>({source:tag,path:tag,report:{tag,definitions:[]}});
const tick=()=>new Promise(resolve=>setImmediate(resolve));

test('older validation cannot overwrite a newer observation',async()=>{
  const host=root();let current=observation('A'),release;
  const api=mountMathObservationView(host,{readObservation:()=>current,
    validateObservation:report=>report.tag==='A'?new Promise(resolve=>{release=resolve;}):Promise.resolve(),
    renderSteps:report=>report.tag,sourceSlice:()=>assert.fail('No definition source supplied')});
  current=observation('B');await api.refresh();
  const rendered=host.children[3].children[0];assert.equal(rendered.innerHTML,'B');
  release();await tick();assert.equal(host.children[3].children[0],rendered);api.dispose();
});

test('source change without refresh rejects a pending observation',async()=>{
  const host=root();let current=observation('A'),release;
  const api=mountMathObservationView(host,{readObservation:()=>current,
    validateObservation:()=>new Promise(resolve=>{release=resolve;}),
    renderSteps:()=>assert.fail('Stale observation rendered'),sourceSlice:()=>''});
  current=observation('B');release();await tick();
  assert.match(host.children[2].textContent,/원문이나 관측이 바뀌/);assert.equal(host.children[3].children.length,0);api.dispose();
});

test('unmount cancels pending display and stops future reads',async()=>{
  const host=root();let release,reads=0;
  const api=mountMathObservationView(host,{readObservation:()=>{reads++;return observation('A');},
    validateObservation:()=>new Promise(resolve=>{release=resolve;}),
    renderSteps:()=>assert.fail('Unmounted observation rendered'),sourceSlice:()=>''});
  api.dispose();release();await tick();assert.equal(host.children.length,0);
  assert.equal(await api.refresh(),null);assert.equal(reads,1);
});

test('observation notifications clear edited results and disconnect on disposal',async()=>{
  const host=root();let current=observation('A'),notify,disconnects=0,reads=0;
  const api=mountMathObservationView(host,{readObservation:()=>{reads++;return current;},
    validateObservation:async()=>{},renderSteps:report=>report.tag,
    subscribeObservation:listener=>{notify=listener;return ()=>{disconnects++;};}});
  await tick();assert.equal(host.children[3].children[0].innerHTML,'A');
  current=null;await notify();assert.equal(host.children[3].children.length,0);
  assert.match(host.children[2].textContent,/NOT_PRODUCED/);
  current=observation('B');await notify();assert.equal(host.children[3].children[0].innerHTML,'B');
  api.dispose();api.dispose();const previousReads=reads;await notify();
  assert.equal(disconnects,1);assert.equal(reads,previousReads);assert.equal(host.children.length,0);
});

test('subscription setup failure clears mounted content and cancels pending rendering',async()=>{
  const host=root();let release;
  assert.throws(()=>mountMathObservationView(host,{readObservation:()=>observation('A'),
    validateObservation:()=>new Promise(resolve=>{release=resolve;}),
    renderSteps:()=>assert.fail('Failed mount rendered'),
    subscribeObservation:()=>{throw new Error('Subscription unavailable');}}),/Subscription unavailable/);
  release();await tick();assert.equal(host.children.length,0);
});

test('project display uses owning UTF-8 source offsets without changing linked report',async()=>{
  const host=root(),source='가나다',raw=JSON.stringify({schema:'ddn.authored_project_artifact.v1',source_snapshot:{files:[{path:'정리.ddn',text:source}]}});
  const hash=Buffer.from(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(raw))).toString('hex');
  const report={coordinate_space:'linked_source_with_original_source_mapping',artifact_sha256:hash,
    observation_only:true,unconditional_proof_issued:false,status:'accepted',phase:'checker',definitions:[{
      name:'정의',status:'accepted',span:{start:100,end:109},original_source:{path:'정리.ddn',start:0,end:9},steps:[]}]};
  const before=JSON.stringify(report);let renderedSource;
  const api=mountProjectMathObservationView(host,{readObservation:()=>({sourceProjectText:raw,report}),renderSteps:(display,text)=>{
    renderedSource=text;assert.deepEqual(display.definitions[0].span,{start:0,end:9});return '기존 표시';}});
  await api.refresh();assert.equal(renderedSource,source);assert.equal(JSON.stringify(report),before);
  assert.match(host.children[3].children[0].innerHTML,/정리.ddn/);api.dispose();
});

test('project refuses dependency edits even when selected entry and report remain unchanged',async()=>{
  const host=root();let raw=JSON.stringify({schema:'ddn.authored_project_artifact.v1',source_snapshot:{files:[{path:'dependency.ddn',text:'before'}]}});
  const hash=Buffer.from(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(raw))).toString('hex');
  const report={coordinate_space:'linked_source_with_original_source_mapping',artifact_sha256:hash,
    observation_only:true,unconditional_proof_issued:false,status:'rejected',phase:'analysis',definitions:[]};
  let renders=0;const api=mountProjectMathObservationView(host,{readObservation:()=>({sourceProjectText:raw,report}),renderSteps:()=>{renders++;return '거부';}});
  await api.refresh();const previousRenders=renders;raw=raw.replace('before','after');await api.refresh();
  assert.equal(renders,previousRenders);assert.equal(host.children[3].children.length,0);
  assert.match(host.children[2].textContent,/전체 작품 원문 결속/);api.dispose();
});

test('explicit inspector never auto-runs and discards a response after a dependency edit',async()=>{
  const host=root();let source='original',notify,release,calls=0;
  const api=mountProjectMathInspector(host,{readSourceProject:()=>source,
    inspectObservation:()=>{calls++;return new Promise(resolve=>{release=resolve;});},
    renderSteps:()=>assert.fail('Stale inspection rendered'),
    subscribeSource:listener=>{notify=listener;}});
  assert.equal(calls,0);const pending=api.inspect();assert.equal(calls,1);
  source='dependency edited';notify();assert.equal(calls,1);
  release({status:'accepted'});assert.equal(await pending,null);
  assert.match(host.children[1].textContent,/원문이 바뀌/);assert.equal(host.children[0].disabled,false);
  api.dispose();assert.equal(await api.inspect(),null);assert.equal(calls,1);
});

test('explicit inspector exposes product errors and allows an explicit retry',async()=>{
  const host=root();let calls=0;
  const api=mountProjectMathInspector(host,{readSourceProject:()=>'exact artifact',
    inspectObservation:async()=>{calls++;throw new Error('E_PRODUCT_EXPORT_REQUIRED');},renderSteps:()=>assert.fail()});
  await api.inspect();assert.match(host.children[1].textContent,/E_PRODUCT_EXPORT_REQUIRED/);
  assert.equal(host.children[0].disabled,false);await api.inspect();assert.equal(calls,2);api.dispose();
});

test('project analysis refusal preserves supplied diagnostics without admitting step observations',async()=>{
  const host=root();const report={phase:'project_analysis',status:'rejected',definitions:[],
    observation_only:true,unconditional_proof_issued:false,diagnostic:{formatted:'E_AUTHORED_PROJECT: 저장 내용 해시가 다릅니다'}};
  const api=mountProjectMathInspector(host,{readSourceProject:()=>'unaltered request',inspectObservation:async()=>report,
    renderSteps:()=>assert.fail('Unbound project rejection rendered as step observation')});
  await api.inspect();assert.match(host.children[1].textContent,/E_AUTHORED_PROJECT: 저장 내용 해시가 다릅니다/);
  assert.equal(host.children[2].children[3].children.length,0);api.dispose();
});
