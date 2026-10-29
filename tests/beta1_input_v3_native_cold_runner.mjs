// Same authored artifact and original input frames at each real process.
// No source flattening, DDN rewriting, expected-world edits, or old-store reuse.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import assert from 'node:assert/strict';
const [repo,executables,out]=process.argv.slice(2);
assert(repo&&executables&&out);fs.mkdirSync(out);
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const write=(file,data)=>{fs.writeFileSync(file,data,{flag:'wx'});return file;};
const json=(file,data)=>write(file,JSON.stringify(data));
const sample=path.join(repo,'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris');
const authored=path.join(sample,'authored-project.json'),initial=path.join(sample,'initial-input.json'),next=path.join(sample,'next-input.json');
const first=JSON.parse(fs.readFileSync(initial,'utf8')),after=JSON.parse(fs.readFileSync(next,'utf8'));
assert.deepEqual(first.binding,after.binding);
const original=JSON.parse(fs.readFileSync(authored,'utf8'));
for(const file of original.source_snapshot.files)assert.equal(sha(Buffer.from(file.text)),sha(fs.readFileSync(path.join(sample,file.path))),'embedded and physical raw source must agree');
const full=json(path.join(out,'continuous-input.json'),{binding:first.binding,snapshots:[...first.snapshots,...after.snapshots]});
const empty=json(path.join(out,'empty-next.json'),{binding:first.binding,snapshots:[]});
const expected=json(path.join(out,'binding.json'),first.binding);
const invalid=structuredClone(after);invalid.snapshots[0].madi=2;
const bad=json(path.join(out,'wrong-next.json'),invalid);
const results=[];
process.on('uncaughtException',error=>{json(path.join(out,'failure.json'),{scope:'RAW_FAILURE_PRESERVED',results,error:String(error),stack:error.stack});console.error(error);process.exitCode=1;});
const storeHashes=root=>Object.fromEntries(fs.readdirSync(root,{withFileTypes:true}).filter(e=>e.isFile()).map(e=>[e.name,sha(fs.readFileSync(path.join(root,e.name)))]));
for(const name of ['ddonilang','ddonirang','또니랑']){
  const dir=path.join(out,name);fs.mkdirSync(dir);const exe=path.join(executables,name+'.exe');
  const invoke=(label,args,success=true)=>{
    const run=spawnSync(exe,args,{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024,timeout:180000});
    if(run.error)throw run.error;
    write(path.join(dir,label+'.stdout'),run.stdout);write(path.join(dir,label+'.stderr'),run.stderr);
    if(success)assert.equal(run.status,0,run.stderr);else assert.notEqual(run.status,0,'must reject without a success publication');
    return success?JSON.parse(run.stdout):run;
  };
  const store=path.join(dir,'store');fs.mkdirSync(store);
  const save=invoke('save3',['작품자료','논리기록',authored,'--입력',initial,'--부모','처음','--판','02','--저장소',store,'--저장소-초기화']);
  const saved=write(path.join(dir,'saved3.json'),save.saved_artifact_json_utf8),receipt=write(path.join(dir,'receipt3.hex'),save.committed_receipt_hex);
  const resumeArgs=input=>['작품자료','이어실행',saved,'--부모','처음','--판','02','--마디수','4','--저장소',store,'--거래',save.transaction_digest_sha256,'--입력',input];
  const before=storeHashes(store);invoke('reject-wrong-next',resumeArgs(bad),false);assert.deepEqual(storeHashes(store),before,'failure must preserve the existing store bytes');
  const resumed=invoke('cold-resume4',resumeArgs(next));
  const referenceStore=path.join(dir,'reference-store');fs.mkdirSync(referenceStore);
  const reference=invoke('continuous7',['작품자료','논리기록',authored,'--입력',full,'--부모','처음','--판','04','--저장소',referenceStore,'--저장소-초기화']).summary;
  assert.equal(reference.state_hash,'blake3:c7c9359477b58adc903fe72ad9b1c9105b5da8158db169842ef5a36d4468202d','preserve the unchanged sample documented hash');
  assert.equal(resumed.state_hash,reference.state_hash);assert.deepEqual(resumed.resources,reference.resources);
  assert.deepEqual(resumed.output_display?.texts,reference.output_display?.texts);
  const tape=path.join(dir,'finite-v3.bin');
  const recorded=invoke('v3-authored-record',['logical-record',authored,full,tape,'--format=v3','--authored']);
  const replayed=invoke('v3-authored-cold-replay',['logical-replay',authored,tape,expected,'--format=v3','--authored']);
  assert.deepEqual(replayed,recorded);assert.equal(recorded.state_hash,reference.state_hash);assert.deepEqual(recorded.resources,reference.resources);
  const rawTape=fs.readFileSync(tape),payload=JSON.parse(rawTape.subarray(26,rawTape.length-32).toString('utf8'));
  payload.frames[1].accepted_world_digest='blake3:wrong';
  const body=Buffer.from(JSON.stringify(payload)),prefix=Buffer.alloc(26);
  Buffer.from('DDN_INPUT_TAPE_V3\n').copy(prefix);prefix.writeBigUInt64LE(BigInt(body.length),18);
  const beforeFooter=Buffer.concat([prefix,body]);
  const footer=crypto.createHash('sha256').update(Buffer.from('ddn.input_tape.v3\0')).update(beforeFooter).digest();
  const mutated=write(path.join(dir,'tampered-resealed-v3.bin'),Buffer.concat([beforeFooter,footer]));
  const storeBeforeBadReplay=storeHashes(store);
  invoke('reject-resealed-world',['logical-replay',authored,mutated,expected,'--format=v3','--authored'],false);
  assert.deepEqual(storeHashes(store),storeBeforeBadReplay);
  const resaved=invoke('resave7',[...resumeArgs(next),'--저장-판','03','--이전-영수증',receipt]);
  assert.equal(resaved.summary.state_hash,reference.state_hash);
  const saved7=write(path.join(dir,'saved7.json'),resaved.saved_artifact_json_utf8);
  const reopened=invoke('cold-reopen7',['작품자료','이어실행',saved7,'--부모','02','--판','03','--마디수','0','--저장소',store,'--거래',resaved.transaction_digest_sha256,'--입력',empty]);
  assert.equal(reopened.state_hash,reference.state_hash);assert.deepEqual(reopened.resources,reference.resources);
  results.push({exe:name,exe_sha256:sha(fs.readFileSync(exe)),authored_sha256:sha(fs.readFileSync(authored)),
    physical_sources:original.source_snapshot.files.map(f=>({path:f.path,sha256:sha(fs.readFileSync(path.join(sample,f.path)))})),
    state_hash:reference.state_hash,v3_sha256:sha(fs.readFileSync(tape)),save3_cold4_equals7:true,wrong_input_store_preserved:true,
    v3_same_authored_record_replay:true,resave_cold_reopen:true});
}
json(path.join(out,'result.json'),{scope:'ACTUAL_DEBUG_NATIVE_CLI3_V3_AND_SQLITE_NOT_WASM_OR_HUMAN',results});
console.log(JSON.stringify({native_workflows:results.length,failures:0,report:path.join(out,'result.json')}));
