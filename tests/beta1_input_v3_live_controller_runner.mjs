// Host/Worker adapter protocol tests only. Not an actual WASM/browser PASS.
import assert from 'node:assert/strict';
import {LogicalActionInput} from '../solutions/seamgrim_ui_mvp/ui/runtime/logical_action_input.js';
const contract={channels:{'왼쪽':'Control'},initialHeld:{'왼쪽':false},maxEvents:8,
  actionMap:{schema:'ddn.input_action_map.v1',actions:[{action_id:'왼쪽',input_snapshot:{last_key:'ArrowLeft'}}]}};
let steps=0,stops=0,aborts=0;
const session={step:async input=>{steps++;return {frame:{input}};},abort:async()=>{aborts++;},
  seal:async()=>{throw new Error('legacy completion must not be used');},
  stopRecording:async()=>{stops++;return {outcome:'INCOMPLETE',tape_bytes:[1],summary:{madi_count:steps}};}};
const host=new LogicalActionInput({...contract,session});
host.receive({key:'ArrowLeft',repeat:false});
await assert.rejects(host.stopRecording(),/PENDING/);assert.equal(stops,0);assert.equal(host.events.length,1);
await host.step();const record=await host.stopRecording();
assert.equal(record.outcome,'INCOMPLETE');assert.equal(record.summary.madi_count,1);
assert.equal(steps,1);assert.equal(stops,1);assert.equal(aborts,0);
assert.throws(()=>host.receive({key:'ArrowLeft',repeat:true}),/STOPPED/);
await assert.rejects(host.step(),/STOPPED/);await assert.rejects(host.seal(),/STOPPED/);await assert.rejects(host.stopRecording(),/STOPPED/);
await host.loseInput('중단 뒤 초점 변경');assert.equal(aborts,0);
let finish;
const racing=new LogicalActionInput({...contract,session:{...session,stopRecording:()=>new Promise(resolve=>{finish=resolve;})}});
const pending=racing.stopRecording();
assert.throws(()=>racing.receive({key:'ArrowLeft',repeat:false}),/STOPPED/);
await assert.rejects(racing.step(),/STOPPED/);await racing.loseInput('중단 응답 대기 중');
finish({outcome:'INCOMPLETE',tape_bytes:[2],summary:{}});assert.equal((await pending).outcome,'INCOMPLETE');assert.equal(aborts,0);
const falseComplete=new LogicalActionInput({...contract,session:{...session,stopRecording:async()=>({outcome:'COMPLETE',tape_bytes:[3]})}});
await assert.rejects(falseComplete.stopRecording(),/PROOF/);assert.equal(falseComplete.stopped,true);
const failed=new LogicalActionInput({...contract,session:{...session,step:async()=>{throw new Error('actual frame failure');},
  stopRecording:async()=>({outcome:'ABORTED',tape_bytes:[4],summary:{outcome:'ABORTED'}})}});
await assert.rejects(failed.step(),/actual frame failure/);assert.equal((await failed.stopRecording()).outcome,'ABORTED');
const oldRuntime=new LogicalActionInput({...contract,session:{step:session.step,abort:session.abort}});
await assert.rejects(oldRuntime.stopRecording(),/V3_UNSUPPORTED/);assert.equal(oldRuntime.stopped,false);
const lostReply=new LogicalActionInput({...contract,session:{...session,stopRecording:async()=>{throw new Error('worker reply lost');}}});
await assert.rejects(lostReply.stopRecording(),/worker reply lost/);await assert.rejects(lostReply.step(),/STOPPED/);
console.log(JSON.stringify({scope:'HOST_PROTOCOL_ONLY',checks:['pending-preserved','explicit-stop','closed-intake','no-extra-madi','no-complete-reseal','reply-race','false-complete-rejected','failed-prefix','old-runtime-explicit','lost-reply-no-resume'],failures:0}));
