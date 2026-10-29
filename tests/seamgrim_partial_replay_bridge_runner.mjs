import assert from 'node:assert/strict';
import {WasmRuntimeAdapter} from '../solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js';

// These are transport tests, not a replacement for Rust replay or Chrome.
const adapter=new WasmRuntimeAdapter();
const calls=[];
adapter.ensureProjectModule=async()=>Object.fromEntries([
  'logical_input_replay_v3','authored_logical_input_replay_v3','manipulation_input_replay_v3'
].map(name=>[name,(...args)=>{calls.push({name,args});return JSON.stringify({replayed:true});}]));
const options={sourceText:'원문',tapeBytes:[1,2,3],bindingJson:'별도 계약'};
await adapter.replayLogicalInput(options);
assert.equal(calls.at(-1).args[3],false);
await adapter.replayLogicalInput({...options,allowPartial:true});
assert.equal(calls.at(-1).args[3],true);
assert.equal(calls.at(-1).args[0],'원문');
assert.deepEqual([...calls.at(-1).args[1]],[1,2,3]);
assert.equal(calls.at(-1).args[2],'별도 계약');
await adapter.replayLogicalInput({...options,authoredProjectJson:'작품 원문',allowPartial:true});
assert.equal(calls.at(-1).name,'authored_logical_input_replay_v3');
assert.equal(calls.at(-1).args[0],'작품 원문');
await adapter.replayManipulationInput(options);
assert.equal(calls.at(-1).args[3],false);
await adapter.replayManipulationInput({...options,allowPartial:true});
assert.equal(calls.at(-1).name,'manipulation_input_replay_v3');
assert.equal(calls.at(-1).args[3],true);
const count=calls.length;
for(const allowPartial of ['true',1,null,{}]) {
  await assert.rejects(()=>adapter.replayLogicalInput({...options,allowPartial}),/E_INPUT_REPLAY_MODE/);
  await assert.rejects(()=>adapter.replayManipulationInput({...options,allowPartial}),/E_INPUT_REPLAY_MODE/);
}
assert.equal(calls.length,count,'invalid options must not invoke the Rust frontdoor');
console.log('PASS: default complete-only, explicit prefix, authored/manipulation forwarding, invalid option rejection');
