import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';

const wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const output=path.resolve(process.env.DDN_PREFIX_OUTPUT);
assert(!fs.existsSync(output),'use a new evidence directory');fs.mkdirSync(output,{recursive:true});
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const binary=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:binary});
const empty=new wasm.DdnWasmVm('');const initialWorld=empty.get_state_hash();empty.free();
const aliases=['ddonilang','ddonirang','또니랑'];const rows=[];
const source='(매마디)마다 { 1 보여주기. }.';
for(const kind of ['logical','manipulation']) {
  const profile=kind==='logical'?'v1-core-v25-logical-alpha2':'v1-core-v25-manipulation-alpha4-provisional';
  const domain={unit:'1',min_raw:'-4294967296',max_raw:'4294967296',step_raw:'1'};
  const binding={source_profile:profile,input_schema:'beta2.prefix-test@1',channel_registry:'beta2.prefix-channels@1',
    source_revision:`sha256:${sha(source)}`,initial_world:initialWorld,initial_rng:7,
    recording_policy:'beta2.prefix-record@1',integrity_policy:'test.seal@1',madi_hz:60,dt_raw:4294967296,
    channels:kind==='logical'?{왼쪽:'Control'}:{수:{kind:'Number',target:'test.radius',domain}},
    initial_held:kind==='logical'?{왼쪽:false}:{}};
  const held=binding.initial_held;
  const snapshots=[0,1,2].map(madi=>({madi,events:[],held}));
  const VM=kind==='logical'?wasm.DdnLogicalInputVm:wasm.DdnManipulationVm;
  const session=new VM(source,JSON.stringify(binding));
  for(const frame of snapshots)session.step_json(JSON.stringify(frame));
  const before=JSON.parse(session.summary_json());
  // The producer is the actual live VM stop operation, never an edited finite tape.
  const record=JSON.parse(session.stop_recording_v3_json());session.free();
  const bytes=Uint8Array.from(record.tape_bytes),tapeHash=sha(bytes);
  assert.equal(record.outcome,'INCOMPLETE');
  const replay=wasm[`${kind}_input_replay_v3`];
  assert.throws(()=>replay(source,bytes,JSON.stringify(binding)),/미완/);
  assert.throws(()=>replay(source,bytes,JSON.stringify(binding),false),/미완/);
  const partial=JSON.parse(replay(source,bytes,JSON.stringify(binding),true));
  assert.equal(partial.state_hash,before.state_hash);
  assert.equal(partial.partial_replay,true);assert.equal(partial.source_record_termination,'Incomplete');
  assert.equal(partial.stopped_after_madi_count,3);assert.equal(partial.last_recorded_madi,2);
  assert.equal(partial.replay_notice,'중단된 기록, 3마디까지');assert.equal(sha(bytes),tapeHash);
  const bad=bytes.slice();bad[bad.length-1]^=1;
  assert.throws(()=>replay(source,bad,JSON.stringify(binding),true),/봉인/);
  assert.throws(()=>replay(source,bytes,JSON.stringify({...binding,initial_rng:8}),true));
  const failed=new VM(source,JSON.stringify(binding));failed.step_json(JSON.stringify(snapshots[0]));
  failed.abort_input('실제 입력 손실');const aborted=JSON.parse(failed.stop_recording_v3_json());failed.free();
  assert.equal(aborted.outcome,'ABORTED');
  assert.throws(()=>replay(source,Uint8Array.from(aborted.tape_bytes),JSON.stringify(binding),true),/실패 기록/);
  const complete=JSON.parse(wasm[`${kind}_input_record_v3`](source,JSON.stringify({binding,snapshots})));
  assert.deepEqual(JSON.parse(replay(source,Uint8Array.from(complete.tape_bytes),JSON.stringify(binding),true)),complete.summary);
  const base=path.join(output,kind);
  fs.writeFileSync(`${base}.ddn`,source);fs.writeFileSync(`${base}.binding.json`,JSON.stringify(binding));
  fs.writeFileSync(`${base}.tape`,bytes);fs.writeFileSync(`${base}.aborted.tape`,Uint8Array.from(aborted.tape_bytes));
  fs.writeFileSync(`${base}.summary.json`,JSON.stringify(partial,null,2));
  for(const alias of aliases) {
    const exe=path.join(process.env.DDN_NATIVE_DIR,`${alias}.exe`);
    const run=(file,flags)=>spawnSync(exe,[`${kind}-replay`,`${base}.ddn`,file,`${base}.binding.json`,...flags],{encoding:'utf8',timeout:120000,maxBuffer:16*1024*1024});
    const denied=run(`${base}.tape`,[]);assert.equal(denied.error,undefined);assert.notEqual(denied.status,0);assert.equal(denied.stdout,'');assert.match(denied.stderr,/미완/);
    const accepted=run(`${base}.tape`,['--부분재생']);assert.equal(accepted.error,undefined);assert.equal(accepted.status,0,accepted.stderr);
    assert.deepEqual(JSON.parse(accepted.stdout),partial);
    const rejected=run(`${base}.aborted.tape`,['--부분재생']);assert.equal(rejected.error,undefined);assert.notEqual(rejected.status,0);assert.equal(rejected.stdout,'');assert.match(rejected.stderr,/실패 기록/);
    rows.push({kind,alias,tape_sha256:tapeHash,state_hash:partial.state_hash,default_rejected:true,explicit_prefix_equal_wasm:true,aborted_rejected:true});
  }
}
fs.writeFileSync(path.join(output,'RESULT.json'),JSON.stringify({claim:'PREFIX_CLI3_WASM_ONLY_NOT_BROWSER_OR_BETA2_PASS',wasm_sha256:sha(binary),rows},null,2));
console.log(`PASS ${rows.length} CLI3/WASM prefix comparisons; default and Aborted rejected; bytes preserved`);
