// Actual frontend queue class; fake runtime responses are HOST ONLY evidence.
import assert from 'node:assert/strict';
import {LogicalActionInput} from '../solutions/seamgrim_ui_mvp/ui/runtime/logical_action_input.js';
const contract={channels:{왼쪽:'Control'},initialHeld:{왼쪽:false},maxEvents:4,
  actionMap:{schema:'ddn.input_action_map.v1',actions:[{action_id:'왼쪽',input_snapshot:{last_key:'ArrowLeft'}}]}};
let releaseStep;
const gate=new Promise(resolve=>{releaseStep=resolve;});
const accepted=[],packets=[];let stale=false;
const completed={tape_bytes:[1,2,3],summary:{state_hash:'fake-not-a-runtime-proof'}};
const session={
  registerInputProducer:()=>({producer_generation:'registered-by-fake-runtime',accepted_event_count:'0'}),
  step:async input=>{accepted.push(structuredClone(input));await gate;return {frame:{input}};},
  abort:()=>{throw Error('no input loss expected');},
  finishPending:async packet=>{
    if(stale) throw Error('E_STALE_WORKER');
    packets.push(structuredClone(packet));return structuredClone(completed);
  },
  finishNotice:()=>({excluded_input_count:'1'}),
};
const host=new LogicalActionInput({...contract,session});
host.receive({key:'ArrowLeft'});
const step=host.step();
host.receive({key:'ArrowLeft'},true); // remains pending while prior Press commits
const finish=host.seal();
assert.throws(()=>host.receive({key:'ArrowLeft'}),/STOPPED/);
assert.equal(packets.length,0,'finish must wait for the actual older step');
releaseStep();await step;
assert.deepEqual(await finish,completed);
assert.equal(accepted.length,1,'pending Release must not make a hidden madi');
assert.equal(accepted[0].events[0].event.kind,'Press');
assert.deepEqual(packets[0],{producer_generation:'registered-by-fake-runtime',
  first_unprocessed:'1',observed_until:'2',events:[{ordinal:0,channel:'왼쪽',event:{kind:'Release'}}]});
assert.equal(host.events.length,0);assert.equal(host.finishNotice.excluded_input_count,'1');
assert.deepEqual(await host.seal(),completed);assert.deepEqual(packets[1],packets[0]);
stale=true;await assert.rejects(host.seal(),/STALE_WORKER/);

let fail=true;
const retrySession={...session,step:async input=>({frame:{input}}),
  finishPending:async()=>{if(fail)throw Error('E_INPUT_TAPE_V3: FinishNotReady');return structuredClone(completed);}};
const retry=new LogicalActionInput({...contract,session:retrySession});
retry.receive({key:'ArrowLeft'});const original=structuredClone(retry.events);
await assert.rejects(retry.seal(),/FinishNotReady/);
assert.deepEqual(retry.events,original);assert.equal(retry.stopped,false);assert.equal(retry.stopping,false);
fail=false;assert.deepEqual(await retry.seal(),completed);assert.equal(retry.events.length,0);

let ran=0;
const oldProgress=new LogicalActionInput({...contract,session:{...session,
  registerInputProducer:()=>{throw Error('MissingReplayProgress');},
  step:async input=>{ran++;return {frame:{input}};}}});
await oldProgress.step();assert.equal(ran,1,'old basic continuation stays usable without new proof');
await assert.rejects(oldProgress.seal(),/MissingReplayProgress/);
assert.equal(oldProgress.stopped,false);
let noticeLost=true;let originPacket;
const origin=new LogicalActionInput({...contract,session:{...session,
  registerInputProducer:()=>({producer_generation:'same-runtime-owner',accepted_event_count:'3'}),
  step:async input=>({frame:{input}}),
  finishPending:async packet=>{originPacket=structuredClone(packet);return structuredClone(completed);},
  finishNotice:()=>{if(noticeLost)throw Error('E_WORKER_LOST');return {excluded_input_count:'1'};}}});
origin.receive({key:'ArrowLeft'});await origin.step();origin.receive({key:'ArrowLeft'},true);
const pendingBefore=structuredClone(origin.events);
await assert.rejects(origin.seal(),/WORKER_LOST/);
assert.equal(origin.finishUncertain,true);assert.deepEqual(origin.events,pendingBefore);
assert.throws(()=>origin.receive({key:'ArrowLeft'}),/STOPPED/);
await assert.rejects(origin.step(),/STOPPED/);
noticeLost=false;await origin.seal();
assert.equal(originPacket.first_unprocessed,'4');assert.equal(originPacket.observed_until,'5');
assert.equal(origin.finishUncertain,false);assert.equal(origin.stopped,true);
console.log(JSON.stringify({result:'PASS',scope:'HOST_QUEUE_ONLY_NOT_RUST_WASM_BROWSER_OR_DURABLE_PROOF',cases:5}));
