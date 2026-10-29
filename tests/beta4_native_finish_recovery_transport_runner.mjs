import assert from 'node:assert/strict';
import {recoverNativeLogicalFinish} from '../solutions/seamgrim_ui_mvp/ui/runtime/native_recording_finish.js';

// Transport/presentation tests only; mocks cannot establish Native recovery.
const binding={source_revision:'test-only'};
const raw='{"summary":{"madi_count":1},"tape_bytes":[7,8,9]}';
const original={original_output_json_utf8:raw,notice:{excluded_input_count:'2'},original_failure:'<original failure>'};
let calls=0,guards=0;
const recovered=await recoverNativeLogicalFinish({authored:'selected-source',binding,unchanged(){guards++;},
  available:async()=>true,request:async(operation,authored,fields)=>{
    calls++;assert.equal(operation,'logical_finish_recover');assert.equal(authored,'selected-source');
    assert.deepEqual(fields,{input_request:{binding,snapshots:[]}});return original;
  }});
assert.equal(recovered.original_output_json_utf8,raw);assert.equal(recovered.original_failure,'<original failure>');
assert.equal(calls,1);assert.equal(guards,3);
await assert.rejects(recoverNativeLogicalFinish({authored:'selected-source',binding,unchanged(){},
  available:async()=>false,request:async()=>{throw Error('must not request');}}),/UNSUPPORTED/);
let changed=false;
await assert.rejects(recoverNativeLogicalFinish({authored:'selected-source',binding,
  unchanged(){if(changed)throw Error('selection changed');},available:async()=>true,
  request:async()=>{changed=true;return original;}}),/selection changed/);
await assert.rejects(recoverNativeLogicalFinish({authored:'selected-source',binding,unchanged(){},available:async()=>true,
  request:async()=>({...original,original_output_json_utf8:'{"summary":{},"tape_bytes":[256]}'})}),/RECOVERY_RESPONSE/);
await assert.rejects(recoverNativeLogicalFinish({authored:'selected-source',binding,unchanged(){},available:async()=>true,
  request:async()=>{throw Error('E_REPLAY_EVIDENCE_RECOVERY_REQUIRED: missing protected cause');}}),/missing protected cause/);
console.log(JSON.stringify({status:'PASS',cases:5,scope:'transport only; Native/Chrome/process-exit NOT_RUN'}));
