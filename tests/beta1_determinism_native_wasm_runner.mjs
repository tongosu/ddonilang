// Same raw authored program enters shipped CLI3 and the committed WASM.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
const [root,commit]=process.argv.slice(2);assert(root&&commit);
const out=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-beta1-determinism-'));
const sha=b=>createHash('sha256').update(b).digest('hex');
const dir=path.join(root,'solutions/seamgrim_ui_mvp/ui/wasm');
const wasmBytes=fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(dir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:wasmBytes});
// Reuse the common owner's ordered query example; (2+1)*2=6 vs 2*2+1=5.
const source=`(매마디)마다 {
 계산값 <- 2. 사건들 <- () 입력사상.사건들.
 사건들마다 {
  { 사건들.채널 == "더함" }이면 { 계산값 <- 계산값 + 1. }
  아니면 { 계산값 <- 계산값 * 2. }.
 }.
 같은사건들 <- () 입력사상.사건들.
 조작맵 <- () 입력사상.만들기.
 눌림값 <- (조작맵, "더함") 입력사상.동작.
}.`;
const template=JSON.parse(fs.readFileSync(path.join(root,'solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json')));
Object.assign(template,{channels:{더함:'Control',곱함:'Control'},initial_held:{더함:false,곱함:false}});
const bound=JSON.parse(wasm.DdnLogicalInputVm.bind_source_json(source,JSON.stringify(template)));
const sourceFile=path.join(out,'source.ddn');fs.writeFileSync(sourceFile,source);
const rows=[];
for(const reverse of [false,true]){
 const names=reverse?['곱함','더함']:['더함','곱함'];
 const snapshots=[0,1,2,3].map(madi=>({madi,held:{더함:madi%2===0,곱함:madi%2===0},
  events:names.map((channel,ordinal)=>({ordinal,channel,event:{kind:madi%2===0?'Press':'Release'}}))}));
 for(const seed of [7,8]){
  const binding={...bound,initial_rng:seed},request={binding,snapshots};
  const key=(reverse?'reverse':'forward')+'-seed'+seed;
  const requestFile=path.join(out,key+'.json'),bindingFile=path.join(out,key+'-binding.json');
  fs.writeFileSync(requestFile,JSON.stringify(request));fs.writeFileSync(bindingFile,JSON.stringify(binding));
  const batch=JSON.parse(wasm.logical_input_record(source,JSON.stringify(request)));
  assert.equal(batch.summary.resources.fixed64.계산값,reverse?'5':'6');
  const live=[];
  for(const delay of [0,3]){
   const vm=new wasm.DdnLogicalInputVm(source,JSON.stringify(binding));
   try{
    for(const snapshot of snapshots){if(delay)await new Promise(resolve=>setTimeout(resolve,delay));vm.step_json(JSON.stringify(snapshot));}
    const sealed=JSON.parse(vm.seal_json());
    assert.deepEqual(sealed,batch,'host pacing/live-vs-batch must not change accepted receipts or bytes');
    live.push({host_delay_ms:delay,exact_batch_bytes_and_summary:true});
   }finally{vm.free();}
  }
  const bytes=Buffer.from(batch.tape_bytes),tape=path.join(out,key+'.tape');fs.writeFileSync(tape,bytes);
  assert.deepEqual(JSON.parse(wasm.logical_input_replay(source,bytes,JSON.stringify(binding))),batch.summary);
  const damaged=Buffer.from(bytes);damaged[damaged.length-1]^=1;
  const bad=path.join(out,key+'-damaged.tape');fs.writeFileSync(bad,damaged);
  assert.throws(()=>wasm.logical_input_replay(source,damaged,JSON.stringify(binding)));
  const aliases=[];
  for(const name of ['ddonilang','ddonirang','또니랑']){
   const exe=path.join(root,'bin',name+'.exe'),recorded=path.join(out,key+'-'+name+'.tape');
   const invoke=args=>spawnSync(exe,args,{encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024});
   const record=invoke(['논리입력기록',sourceFile,requestFile,recorded]);assert.ifError(record.error);assert.equal(record.status,0,record.stderr);
   assert.deepEqual(JSON.parse(record.stdout),batch.summary);assert.deepEqual(fs.readFileSync(recorded),bytes);
   const reject=invoke(['논리입력재생',sourceFile,bad,bindingFile]);assert.ifError(reject.error);assert.notEqual(reject.status,0);assert.equal(reject.stdout,'');
   const replay=invoke(['논리입력재생',sourceFile,tape,bindingFile]);assert.ifError(replay.error);assert.equal(replay.status,0,replay.stderr);assert.deepEqual(JSON.parse(replay.stdout),batch.summary);
   aliases.push({name,sha256:sha(fs.readFileSync(exe)),record_replay_receipts_bytes_equal:true,damaged_rejected_no_stdout:true});
  }
  rows.push({key,seed,reverse,source_sha256:sha(Buffer.from(source)),tape_sha256:sha(bytes),state_hash:batch.summary.state_hash,hand_calculated:reverse?5:6,live,aliases});
 }
}
assert.notEqual(rows[0].state_hash,rows[2].state_hash,'reversing accepted events changes the world');
assert.notEqual(rows[0].tape_sha256,rows[1].tape_sha256,'seed must remain bound even when this example does not draw randomness');
const report={status:'PASS_ACTUAL_CLI3_WASM_ORDER_SEED_PACING',commit,root,out,wasm_sha256:sha(wasmBytes),rows,
 scope:'logical single-file actual record/replay, host pacing and tamper rejection; ordinary XOR-tick/Local Project cold paths use separately bound evidence',
 not_claimed:['whole_VE01','RD669_full','official_ledger','Human','independent_review']};
fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({status:report.status,out,cases:rows.length}));
