import assert from 'node:assert/strict';
import {createNativeLogicalFinishBarrier} from '../solutions/seamgrim_ui_mvp/ui/runtime/native_recording_finish.js';
const id={result_identity:'sha256:'+'a'.repeat(64)};
const packet={producer_generation:'browser',first_unprocessed:'1',observed_until:'1',events:[]};
const record={summary:{madi_count:1,state_hash:'not-an-authority',frames:[{input:{madi:0,events:[]}}]},tape_bytes:[1,2]};
function fixture({stale=false,admissionError=null,badPreview=false,badPosition=false}={}){
 const calls=[];let begins=0,identities=0;
 const barrier=createNativeLogicalFinishBarrier({authored:'stub-only-not-source-evidence',binding:{},unchanged(){},
  available:async()=>true,checkpointMatches:()=>true,request:async(operation,authored,fields)=>{
   calls.push({operation,fields});
   if(operation==='logical_finish_begin'){
    if(admissionError)throw Error(admissionError);
    begins++;return {session_handle:'native-'+begins,registration:{producer_generation:'native',accepted_event_count:badPosition?'0':'1'}};
   }
   if(operation==='logical_recording_result_identity'){
    identities++;return stale&&identities===1?{result_identity:'sha256:'+'b'.repeat(64)}:id;
   }
   if(operation==='logical_save')return {kind:'native_logical_world',summary:record.summary};
   if(operation==='logical_finish_prepare')return badPreview?{...record,tape_bytes:[9]}:record;
   if(operation==='logical_finish_commit')return record;
   if(operation==='logical_finish_notice')return {excluded_input_count:'0'};
   if(operation==='logical_finish_close')return {closed:true};
   throw Error('unexpected '+operation);
  }});
 return {barrier,calls};
}
const good=fixture();assert.deepEqual(await good.barrier(packet,record,id),record);
assert(!good.calls.some(x=>x.operation==='logical_save'));
assert.equal(good.calls.filter(x=>x.operation==='logical_recording_result_identity').length,2);
assert(good.calls.findIndex(x=>x.operation==='logical_finish_prepare')<good.calls.findIndex(x=>x.operation==='logical_finish_commit'));
await good.barrier.close();
const stale=fixture({stale:true});assert.deepEqual(await stale.barrier(packet,record,id),record);
assert.equal(stale.calls.filter(x=>x.operation==='logical_save').length,1);
assert.equal(stale.calls.filter(x=>x.operation==='logical_finish_begin').length,2);
assert(stale.calls.findIndex(x=>x.operation==='logical_finish_close')<stale.calls.findIndex(x=>x.operation==='logical_save'));
await stale.barrier.close();
for(const admissionError of ['E_SOURCE_WORLD_CURRENT: RecoveryRequired','E_NATIVE_SOURCE_SOURCE_CHANGED']){
 const rejected=fixture({admissionError});await assert.rejects(rejected.barrier(packet,record,id),new RegExp(admissionError));
 assert(!rejected.calls.some(x=>['logical_save','logical_finish_commit'].includes(x.operation)));
}
const wrong=fixture({badPreview:true});await assert.rejects(wrong.barrier(packet,record,id),/BEFORE_PENDING/);
assert(!wrong.calls.some(x=>x.operation==='logical_finish_commit'));assert.equal(wrong.calls.at(-1).operation,'logical_finish_close');
const position=fixture({badPosition:true});await assert.rejects(position.barrier(packet,record,id),/BEFORE_PENDING/);
assert(!position.calls.some(x=>x.operation==='logical_finish_prepare'||x.operation==='logical_finish_commit'));
console.log(JSON.stringify({status:'PASS',scope:'HOST_TRANSPORT_ONLY_NOT_NATIVE_AUTHORITY',cases:6,
 checks:['actual-native-identity-required','full-candidate-before-commit','stale-hint-ordinary-checked-save',
 'custody-error-no-fallback','source-error-no-fallback','preview-and-position-reject-before-publication']}));
