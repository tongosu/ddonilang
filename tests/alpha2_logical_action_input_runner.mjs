import assert from 'node:assert/strict';
import {LogicalActionInput} from '../solutions/seamgrim_ui_mvp/ui/runtime/logical_action_input.js';

const contract = {
  channels:{왼쪽:'Control', 회전:'Control'}, initialHeld:{왼쪽:false,회전:false}, maxEvents:4,
  actionMap:{schema:'ddn.input_action_map.v1', actions:[
    {action_id:'왼쪽', input_snapshot:{last_key:'ArrowLeft'}},
    {action_id:'회전', input_snapshot:{last_key:'ArrowUp'}},
  ]},
};
let accepted=[], aborted=[], pending=null;
const session={
  step:async input => {accepted.push(structuredClone(input)); if (pending) await pending; return {frame:{input}};},
  abort:async message => {aborted.push(message);}, seal:async () => ({accepted}),
};
const host=new LogicalActionInput({...contract,session});
assert.throws(()=>new LogicalActionInput({...contract,session,nextMadi:NaN}),/POSITION/);
const restoredHost=new LogicalActionInput({...contract,session,nextMadi:3,initialHeld:{왼쪽:true,회전:false}});
restoredHost.receive({key:'ArrowLeft'},true);
await restoredHost.step();
assert.equal(accepted.at(-1).madi,3);
assert.deepEqual(accepted.at(-1).events,[{ordinal:0,channel:'왼쪽',event:{kind:'Release'}}]);
accepted=[];
assert.equal(host.receive({key:'Unmapped'}),false);
host.receive({key:'ArrowLeft',repeat:false});
host.receive({key:'ArrowLeft',repeat:true});
host.receive({key:'ArrowLeft'},true);
await host.step();
assert.deepEqual(accepted[0],{madi:0,events:[
  {ordinal:0,channel:'왼쪽',event:{kind:'Press'}},
  {ordinal:1,channel:'왼쪽',event:{kind:'Repeat'}},
  {ordinal:2,channel:'왼쪽',event:{kind:'Release'}},
],held:{왼쪽:false,회전:false}});
host.receive({key:'ArrowUp',repeat:false});
await assert.rejects(host.seal(),/PENDING/);
let release;
pending=new Promise(resolve=>{release=resolve;});
const inFlight=host.step();
await assert.rejects(host.step(),/STEP_BUSY/);
// Arriving while the previous madi executes belongs to the following madi.
host.receive({key:'ArrowUp'},true);
release(); await inFlight; pending=null;
await host.step();
assert.deepEqual(accepted[1].held,{왼쪽:false,회전:true});
assert.equal(accepted[2].events[0].event.kind,'Release');
assert.equal(accepted[2].madi,2);
await host.seal();
await host.loseInput('초점 상실');
await assert.rejects(host.step(),/INPUT_LOST/);
await assert.rejects(host.seal(),/INPUT_LOST/);
assert.equal(aborted.length,1);
assert.equal(accepted.length,3,'focus loss must not invent a release frame');
const overflow=new LogicalActionInput({...contract,maxEvents:1,session});
overflow.receive({key:'ArrowLeft',repeat:false});
assert.throws(()=>overflow.receive({key:'ArrowLeft',repeat:true}),/INPUT_LOST/);
await assert.rejects(overflow.seal(),/INPUT_LOST/);
const lost=new LogicalActionInput({...contract,session});
assert.throws(()=>lost.receive({key:'ArrowLeft',repeat:true}),/INPUT_LOST/);
const duplicate=structuredClone(contract);
duplicate.actionMap.actions[1].input_snapshot.last_key='ArrowLeft';
assert.throws(()=>new LogicalActionInput({...duplicate,session}),/ACTION_MAP_INVALID/);
assert.throws(()=>new LogicalActionInput({...contract,maxEvents:undefined,session}),/CAPACITY/);
for (const change of ['event','loss','step']) {
  let finishSeal, finishStep;
  const delayed={...session,
    seal:()=>new Promise(resolve=>{finishSeal=resolve;}),
    step:input=>new Promise(resolve=>{finishStep=()=>resolve({frame:{input}});}),
  };
  const sealing=new LogicalActionInput({...contract,session:delayed});
  const result=sealing.seal();
  let stepResult;
  if(change==='event') sealing.receive({key:'ArrowLeft',repeat:false});
  else if(change==='loss') await sealing.loseInput('저장 응답 대기 중 초점 상실');
  else stepResult=sealing.step();
  finishSeal({tape_bytes:[1]});
  await assert.rejects(result,change==='loss'?/INPUT_LOST/:/PENDING/);
  if(stepResult) { finishStep(); await stepResult; }
}
// WO14-A2-UI: main playback controls are explicitly part of the input region;
// editors are not. No synthetic release or world transition on focus changes.
class InputRegion {
  constructor() { this.listeners={}; }
  addEventListener(kind,fn) { this.listeners[kind]=fn; }
  removeEventListener(kind) { delete this.listeners[kind]; }
  contains(node) { return node===this; }
}
const region=new InputRegion(), control=new InputRegion(), editor=new InputRegion();
const composite=new LogicalActionInput({...contract,session});
composite.attach(region,{controls:[control]});
region.listeners.focusout({relatedTarget:control});
assert.equal(composite.failure,null,'moving to explicit playback control is not input loss');
control.listeners.keydown({key:'ArrowLeft',repeat:false,preventDefault(){}});
control.listeners.keyup({key:'ArrowLeft',repeat:false,preventDefault(){}});
assert.deepEqual(composite.events.map(row=>row.event.kind),['Press','Release']);
control.listeners.focusout({relatedTarget:editor});
await assert.rejects(composite.step(),/INPUT_LOST/);
composite.detach();
assert.deepEqual(Object.keys(control.listeners),[],'detach includes playback controls');
const outer=new InputRegion(),inner=new InputRegion();
outer.contains=node=>node===outer || node===inner;
const nested=new LogicalActionInput({...contract,session});
nested.attach(inner,{controls:[outer,outer]});
assert.deepEqual(Object.keys(inner.listeners),[],'nested primary has no duplicate bubbling listeners');
outer.listeners.keydown({key:'ArrowLeft',repeat:false,preventDefault(){}});
assert.equal(nested.events.length,1);
nested.detach();
// Alpha4 provisional scalar/point payloads are queued without converting raw
// integers through JS Number or encoding values in action names.
const typedAccepted=[];
const typed=new LogicalActionInput({session:{...session,step:async snapshot=>{typedAccepted.push(structuredClone(snapshot));return {frame:{input:snapshot}};}},
  channels:{},initialHeld:{},actionMap:{schema:'ddn.input_action_map.v1',actions:[]},maxEvents:4,
  valueChannels:{값:{kind:'Number'},점:{kind:'Coordinate'}}});
typed.receiveValue('값',{raw:'2147483648',phase:'Set'});
typed.receiveValue('점',{raw:['12884901888','4294967296'],phase:'Begin'});
typed.receiveValue('점',{raw:['17179869184','4294967296'],phase:'End'});
await typed.step();
assert.deepEqual(typedAccepted[0].events.map(row=>row.ordinal),[0,1,2]);
assert.equal(typedAccepted[0].events[1].event.payload.raw[0],'12884901888');
assert.deepEqual(typedAccepted[0].held,{});
assert.throws(()=>typed.receiveValue('값',{raw:'1.0',phase:'Set'}),/MANIPULATION/);
await assert.rejects(typed.seal(),/MANIPULATION/);
console.log(JSON.stringify({result:'ALPHA2_LOGICAL_HOST_MAPPING_PASS',
  scope:'host-only mapping/ordered queue/single-flight/loss admission; fake session is not runtime evidence'}));
