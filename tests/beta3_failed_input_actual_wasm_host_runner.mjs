// Actual WASM VM + actual frontend queue. Native-unavailable transport is
// deliberately a stub: this does not prove Native persistence or browser UI.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
import {LogicalActionInput} from '../solutions/seamgrim_ui_mvp/ui/runtime/logical_action_input.js';
import {createNativeLogicalFinishBarrier} from '../solutions/seamgrim_ui_mvp/ui/runtime/native_recording_finish.js';
const [wasmDir,out]=process.argv.slice(2);assert(wasmDir&&out&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const bytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const sha=data=>createHash('sha256').update(data).digest('hex');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);await wasm.default({module_or_path:bytes});
const empty=new wasm.DdnWasmVm('');const initialWorld=empty.get_state_hash();empty.free();
const source='(매마디)마다 { 값 <- 6.0. 사건들 <- () 입력사상.사건들. 사건들마다 { { 사건들.채널 == "곱함" }이면 { 값 <- 1.0 / 0.0. }. }. }. 짜임 := (관계들=차림[(보기=(() 표준보기.결과표)) 표준보개.으뜸]) 표준보개.짜임. (값:값) 보임.';
const channels={곱함:'Control',더함:'Control'},initialHeld={곱함:false,더함:false};
const binding={source_profile:'v1-core-v25-logical-alpha2',input_schema:'wo23.failed-input-test@1',channel_registry:'wo23.failed-channels-test@1',
  source_revision:'sha256:'+sha(source),initial_world:initialWorld,initial_rng:7,recording_policy:'test.record@1',integrity_policy:'test.seal@1',
  madi_hz:60,dt_raw:4294967296,channels,initial_held:initialHeld};
const report={claim:'ACTUAL_WASM_FAILED_ATTEMPT_HOST_PRESERVED_NOT_NATIVE_OR_BROWSER',wasm_sha256:sha(bytes),source_sha256:sha(source),verdict:'NOT_RUN'};
const vm=new wasm.DdnLogicalInputVm(source,JSON.stringify(binding));
try{
  let steps=0;const operations=[];
  const session={step:async input=>{steps++;return JSON.parse(vm.step_json(JSON.stringify(input)));},abort:async message=>vm.abort_input(message),
    summary:async()=>JSON.parse(vm.summary_json()),resultIdentity:async()=>JSON.parse(vm.result_identity_json()),
    stopRecording:async()=>JSON.parse(vm.stop_recording_v3_json()),registerInputProducer:()=>JSON.parse(vm.register_input_producer_json())};
  const barrier=createNativeLogicalFinishBarrier({authored:'transport-stub-not-native-authority',binding,unchanged(){},available:async()=>true,
    request:async operation=>{operations.push(operation);throw Error('E_NATIVE_FAILURE_ANCHOR_REQUIRED: deliberately unavailable transport');}});
  const host=new LogicalActionInput({session,channels,initialHeld,maxEvents:8,finishBarrier:barrier,
    actionMap:{schema:'ddn.input_action_map.v1',actions:[{action_id:'더함',input_snapshot:{last_key:'ArrowLeft'}},{action_id:'곱함',input_snapshot:{last_key:'ArrowRight'}}]}});
  host.receive({key:'ArrowLeft'});await host.step();const before=await session.summary();
  host.receive({key:'ArrowRight'});await assert.rejects(host.step());const after=await session.summary();
  for(const key of ['state_hash','resources','frames','input_receipts','madi_count','bogae_projection'])assert.deepEqual(after[key],before[key],key);
  assert.equal(after.outcome,'ABORTED');assert.equal(host.failedInput.madi,1);assert.equal(host.failedInput.events[0].channel,'곱함');
  const record=await host.stopRecording();assert.equal(record.outcome,'ABORTED');assert.deepEqual(record.summary,after);assert.equal(steps,2);
  assert.equal(host.nativeStopStored,false);assert.match(host.nativeStopFailure,/ANCHOR_REQUIRED/);
  assert.deepEqual(operations,['logical_finish_begin'],'failure must not save RAM as a normal world');
  fs.writeFileSync(path.join(out,'actual-aborted.bin'),Uint8Array.from(record.tape_bytes));
  report.accepted_count=after.madi_count;report.failed_madi=host.failedInput.madi;report.actual_failure=after.failure;
  report.tape_sha256=sha(Uint8Array.from(record.tape_bytes));report.native='EXPLICITLY_UNVERIFIED_TRANSPORT_STUB';report.verdict='PASS_TARGET_SCOPE';
}catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
finally{vm.free();fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));}
