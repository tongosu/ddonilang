import assert from 'node:assert/strict';
import {createNativeLogicalFinishBarrier} from '../solutions/seamgrim_ui_mvp/ui/runtime/native_recording_finish.js';
import {LogicalActionInput} from '../solutions/seamgrim_ui_mvp/ui/runtime/logical_action_input.js';

const packet={producer_generation:'actual-browser-generation',first_unprocessed:'1',observed_until:'2',
  events:[{ordinal:0,channel:'left',event:'Release'}]};
const record={tape_bytes:[7,8,9],summary:{madi_count:1,state_hash:'test-only',frames:[{input:{madi:0}}]}};
const resultIdentity={result_identity:`sha256:${'a'.repeat(64)}`};
function fixture({mismatch=false,loseFirstCommit=false,aborted=null}={}){
  const calls=[],progress=[];let lost=false;
  const barrier=createNativeLogicalFinishBarrier({authored:'owned-test-artifact',binding:{source_revision:'test-only'},
    unchanged(){},onProgress:message=>progress.push(message),available:async()=>true,request:async(operation,authored,fields)=>{
      calls.push({operation,authored,fields:structuredClone(fields)});
      switch(operation){
        case 'logical_save':return {kind:'native_logical_world',summary:record.summary};
        case 'logical_finish_begin':return {session_handle:'private-handle',registration:{producer_generation:'actual-native-generation',accepted_event_count:'1'}};
        case 'logical_finish_prepare':return mismatch?{...record,tape_bytes:[0]}:structuredClone(record);
        case 'logical_recording_result_identity':return structuredClone(resultIdentity);
        case 'logical_finish_commit':
          if(loseFirstCommit&&!lost){lost=true;throw new Error('transport acknowledgement lost');}
          return structuredClone(record);
        case 'logical_finish_notice':return {excluded_input_count:'1'};
        case 'logical_finish_verify':return {status:'REPLAY_VERIFIED',summary:record.summary};
        case 'logical_recording_stop':return {outcome:'INCOMPLETE',tape_bytes:[4],summary:record.summary};
        case 'logical_recording_failed_input':return structuredClone(aborted);
        case 'logical_finish_close':return {closed:true};
        default:throw new Error(operation);
      }
    }});
  return {calls,barrier,progress};
}
const good=fixture();
assert.deepEqual(await good.barrier(structuredClone(packet),structuredClone(record),resultIdentity),record);
assert.equal(good.progress.length,3);assert(good.progress.every(message=>message.includes('중입니다')));
assert.deepEqual(good.calls.map(v=>v.operation),['logical_save','logical_finish_begin','logical_finish_prepare','logical_recording_result_identity','logical_finish_commit','logical_finish_notice']);
assert.equal(good.calls[0].fields.expected_result_identity,resultIdentity.result_identity);
const submitted=JSON.parse(good.calls.find(v=>v.operation==='logical_finish_commit').fields.pending_json_utf8);
assert.deepEqual(submitted,{...packet,producer_generation:'actual-native-generation'});
assert.equal(packet.producer_generation,'actual-browser-generation');
assert.deepEqual(await good.barrier(structuredClone(packet),structuredClone(record),resultIdentity),record);
assert.equal(good.calls.filter(v=>v.operation==='logical_save').length,1);
await assert.rejects(good.barrier.stop(record.summary,resultIdentity),/FINISHED/);
assert.equal((await good.barrier.verifyReceipt()).status,'REPLAY_VERIFIED');
assert.deepEqual(await good.barrier(structuredClone(packet),structuredClone(record),resultIdentity),record);
await assert.rejects(good.barrier({...packet,events:[]},record,resultIdentity),/PENDING_CHANGED/);
await good.barrier.close();
assert.equal(good.calls.at(-1).operation,'logical_finish_close');
const mismatch=fixture({mismatch:true});
await assert.rejects(mismatch.barrier(packet,record,resultIdentity),/RESULT_MISMATCH/);
assert.equal(mismatch.calls.filter(v=>v.operation==='logical_finish_commit').length,0);
await assert.rejects(mismatch.barrier(packet,record,resultIdentity),/RESULT_MISMATCH/);
assert.equal(mismatch.calls.filter(v=>v.operation==='logical_finish_commit').length,0);
const uncertain=fixture({loseFirstCommit:true});
await assert.rejects(uncertain.barrier(packet,record,resultIdentity),/acknowledgement lost/);
assert.deepEqual(await uncertain.barrier(packet,record,resultIdentity),record);
assert.equal(uncertain.calls.filter(v=>v.operation==='logical_finish_begin').length,1);
const stopped=fixture();
assert.equal((await stopped.barrier.stop(record.summary,resultIdentity)).outcome,'INCOMPLETE');
assert.equal((await stopped.barrier.stop(record.summary,resultIdentity)).outcome,'INCOMPLETE');
assert.equal(stopped.calls.filter(v=>v.operation==='logical_save').length,1);
await assert.rejects(stopped.barrier(packet,record,resultIdentity),/NO_END/);
const failedSummary={...record.summary,outcome:'ABORTED',failure:'test-only actual-runtime rejection'};
const failedRecord={outcome:'ABORTED',tape_bytes:[5],summary:failedSummary};
const badInput={madi:1,events:[],held:{left:true}};
const failed=fixture({aborted:failedRecord});
assert.deepEqual(await failed.barrier.stop(failedSummary,resultIdentity,badInput),failedRecord);
assert.deepEqual(failed.calls.map(row=>row.operation),['logical_finish_begin','logical_recording_failed_input']);
assert.deepEqual(failed.calls[1].fields.failed_input,badInput);
assert.deepEqual(failed.calls[1].fields.input_request.snapshots,record.summary.frames.map(row=>row.input));
assert.equal(failed.calls.some(row=>row.operation==='logical_save'),false,'failed RAM must not become a normal world save');
const loss=fixture();assert.equal(await loss.barrier.stop(failedSummary,null),null);
assert.equal(loss.calls.length,0,'an input-loss message cannot fabricate a Native attempted input');
const wrongFailure=fixture({aborted:{...failedRecord,outcome:'INCOMPLETE'}});
await assert.rejects(wrongFailure.barrier.stop(failedSummary,resultIdentity,badInput),/FAILURE_RESULT_MISMATCH/);
let attempted=null,failedSubmitCount=0;
const failedHost=new LogicalActionInput({channels:{left:'Control'},initialHeld:{left:false},maxEvents:4,
  actionMap:{schema:'ddn.input_action_map.v1',actions:[{action_id:'left',input_snapshot:{last_key:'ArrowLeft'}}]},
  session:{step:async()=>{failedSubmitCount++;throw Error('actual rejected step - fake runtime only');},abort:async()=>{},
    summary:async()=>failedSummary,resultIdentity:async()=>resultIdentity,stopRecording:async()=>failedRecord},
  finishBarrier:{stop:async(_summary,_result,input)=>{attempted=structuredClone(input);throw Error('E_NATIVE_FAILURE_ANCHOR_REQUIRED');},close:async()=>{}}});
failedHost.receive({key:'ArrowLeft'});await assert.rejects(failedHost.step());
assert.deepEqual(await failedHost.stopRecording(),failedRecord);assert.equal(failedSubmitCount,1);
assert.equal(attempted.madi,0);assert.equal(attempted.events.length,1);
assert.equal(failedHost.nativeStopStored,false);assert.match(failedHost.nativeStopFailure,/ANCHOR_REQUIRED/);
for(const [reason,blocked] of [['E_NATIVE_FINISH_BEFORE_PENDING: injected rejection',false],
  ['E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: injected uncertainty',true]]){
  let published=0;
  const session={step:async input=>({frame:{input}}),abort:async()=>{},
    registerInputProducer:async()=>({producer_generation:'host-only',accepted_event_count:'0'}),
    previewFinishPending:async()=>({tape_bytes:[0],summary:{madi_count:0}}),
    resultIdentity:async()=>structuredClone(resultIdentity),
    finishPending:async()=>{published++;return record;},finishNotice:async()=>({excluded_input_count:'1'})};
  const live=new LogicalActionInput({session,channels:{left:'Control'},initialHeld:{left:false},maxEvents:4,
    actionMap:{schema:'ddn.input_action_map.v1',actions:[{action_id:'left',input_snapshot:{last_key:'ArrowLeft'}}]},
    finishBarrier:async()=>{throw new Error(reason);}});
  live.receive({key:'ArrowLeft',repeat:false});
  await assert.rejects(live.seal(),error=>error.message===reason);
  assert.equal(published,0);assert.equal(live.stopped,false);assert.equal(live.events.length,1);
  assert.equal(live.finishUncertain,blocked);
}
console.log(JSON.stringify({result:'PASS',scope:'HOST_TRANSPORT_ONLY_NOT_NATIVE_OR_WASM_EVIDENCE',
  checks:['queue-preserved','native-candidate-before-commit','changed-queue-rejected','mismatch-never-committed','uncertain-ack-same-owner-retry',
    'failed-attempt-not-status','no-normal-save-of-failed-RAM','input-loss-not-native-proof','failed-file-retained-with-explicit-unverified-native']}));
