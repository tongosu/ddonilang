import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {spawn,spawnSync} from 'node:child_process';

// Real competing SQLite transaction in an isolated copy of an admitted store.
// No product failpoint, altered codec, or termination of a foreign process.
async function nativeLockedStorageGuard(executable,args,store,receipt,expected) {
  const copy=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-native-locked-save-'));
  fs.cpSync(store,copy,{recursive:true});
  const database=path.join(copy,'vc04_snapshot_v2.sqlite3');
  const inspect=`import sqlite3,sys,json,hashlib
db=sqlite3.connect(sys.argv[1]); rows={}
for table in ['vc04_prepared_payloads','vc04_project_revisions','vc04_project_heads']:
    rows[table]=[[v.hex() if isinstance(v,bytes) else v for v in row] for row in db.execute('SELECT * FROM '+table+' ORDER BY 1')]
print(hashlib.sha256(json.dumps(rows,sort_keys=True,separators=(',',':')).encode()).hexdigest()); db.close()`;
  const fingerprint=()=>{
    const run=spawnSync('python',['-c',inspect,database],{encoding:'utf8',timeout:30_000});
    assert.ifError(run.error);assert.equal(run.status,0,run.stderr);return run.stdout.trim();
  };
  const before=fingerprint();
  const lock=spawn('python',['-u','-c',`import sqlite3,sys
db=sqlite3.connect(sys.argv[1]); db.execute('BEGIN IMMEDIATE')
print('LOCK_READY',flush=True); sys.stdin.readline(); db.rollback(); db.close()`,database],{stdio:['pipe','pipe','pipe']});
  let output='',errors='';lock.stderr.on('data',chunk=>errors+=chunk.toString());
  const closed=new Promise((resolve,reject)=>{lock.once('error',reject);lock.once('close',resolve);});
  const copied=[...args];copied[10]=copy;
  const receiptFile=path.join(copy,'lock-test-previous-receipt.hex');fs.writeFileSync(receiptFile,receipt+'\n');
  let rejected;
  try {
    await new Promise((resolve,reject)=>{
      const timer=setTimeout(()=>reject(new Error('SQLite lock handshake timeout: '+errors)),10_000);
      lock.stdout.on('data',chunk=>{output+=chunk.toString();if(output.includes('LOCK_READY')){clearTimeout(timer);resolve();}});
      lock.once('error',error=>{clearTimeout(timer);reject(error);});
      lock.once('close',()=>{clearTimeout(timer);if(!output.includes('LOCK_READY'))reject(new Error(errors));});
    });
    rejected=spawnSync(executable,[...copied,'--저장-판','03','--이전-영수증',receiptFile],{encoding:'utf8',timeout:30_000});
    assert.ifError(rejected.error);assert.notEqual(rejected.status,0,'locked store must not publish success');
    assert.equal(rejected.stdout,'','failed checkpoint must not publish summary, artifact or receipt');
    assert.match(rejected.stderr,/E_PROJECT_RUNTIME_NATIVE/);
    assert.equal(fingerprint(),before,'busy failure must preserve all existing persisted rows');
  } finally {
    lock.stdin.end('\n');assert.equal(await closed,0,errors);
  }
  assert.equal(fingerprint(),before,'normal lock release must not change the checkpoint');
  const resumed=spawnSync(executable,copied,{encoding:'utf8',timeout:30_000});
  assert.ifError(resumed.error);assert.equal(resumed.status,0,resumed.stderr);
  assert.deepEqual(JSON.parse(resumed.stdout),expected,'original checkpoint must resume after lock release');
  const saved=spawnSync(executable,[...copied,'--저장-판','03','--이전-영수증',receiptFile],{encoding:'utf8',timeout:30_000});
  assert.ifError(saved.error);assert.equal(saved.status,0,saved.stderr);
  const checkpoint=JSON.parse(saved.stdout);assert.deepEqual(checkpoint.summary,expected);
  assert.match(checkpoint.committed_receipt_hex,/^[0-9a-f]+$/);
  return {store:copy,lock_process_pid:lock.pid,rejection_exit:rejected.status,stdout_bytes:0,
    persisted_rows_unchanged:true,release_exit:0,resume_equal:true,retry_checkpoint_committed:true,
    diagnostic:rejected.stderr.trim(),scope:'SQLITE_BUSY_FAILURE_NOT_POWER_LOSS_RECOVERY'};
}

// Exercise the shipped CLI against corrupted copies of its own admitted SQLite
// store. These are test-only mutations, never a product codec or repair path.
function nativeCorruptionGuards(executable,args,store,expected) {
  const rows=[];
  for(const kind of ['snapshot_rehashed','envelope_corrupt','revision_missing','rng_resealed']) {
    const copy=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-native-corrupt-'+kind+'-'));
    fs.cpSync(store,copy,{recursive:true});
    const copiedArgs=[...args];copiedArgs[10]=copy;
    const pristine=spawnSync(executable,copiedArgs,{encoding:'utf8',timeout:30_000});
    assert.ifError(pristine.error);assert.equal(pristine.status,0,pristine.stderr);
    assert.deepEqual(JSON.parse(pristine.stdout),expected,'copied store must be admitted before corruption');
    const inspect=`import sqlite3,sys,json,hashlib
db=sqlite3.connect(sys.argv[1]); db.execute('PRAGMA foreign_keys=OFF')
kind=sys.argv[2]; digest=bytes.fromhex(sys.argv[3])
if kind=='snapshot_rehashed':
    value=db.execute('SELECT canonical_snapshot_v2_bytes FROM vc04_prepared_payloads WHERE transaction_digest=?',(digest,)).fetchone()[0]
    changed=bytearray(value); changed[-1]^=1; changed=bytes(changed)
    db.execute('UPDATE vc04_prepared_payloads SET canonical_snapshot_v2_bytes=?,snapshot_v2_sha256=? WHERE transaction_digest=?',(changed,hashlib.sha256(changed).digest(),digest))
elif kind=='envelope_corrupt':
    value=db.execute('SELECT revision_envelope_bytes FROM vc04_prepared_payloads WHERE transaction_digest=?',(digest,)).fetchone()[0]
    changed=bytes([value[0]^1])+value[1:]
    db.execute('UPDATE vc04_prepared_payloads SET revision_envelope_bytes=? WHERE transaction_digest=?',(changed,digest))
elif kind=='revision_missing':
    db.execute('DELETE FROM vc04_project_revisions WHERE transaction_digest=?',(digest,))
elif kind=='rng_resealed':
    row=db.execute('SELECT project_identity,expected_parent_history_revision_identity,new_history_revision_identity,execution_profile_identity,project_execution_snapshot_identity,canonical_snapshot_v2_bytes,revision_integrity_sha256 FROM vc04_prepared_payloads WHERE transaction_digest=?',(digest,)).fetchone()
    domain=b'DDN-SNAPSHOT-V2-COMPLETE-STATE-V2'+bytes([0]); value=row[5]
    assert value.startswith(domain+bytes([0,2,19]))
    offset=len(domain)+3; fields=[]
    for tag in range(1,20):
        assert value[offset]==tag
        length=int.from_bytes(value[offset+1:offset+9],'big'); offset+=9
        fields.append(value[offset:offset+length]); offset+=length
    assert offset==len(value) and fields[13][:3]==bytes([0,1,3])
    # Change the retained base seed but correctly recompute BOTH capsule and
    # SQLite transaction hashes. A checksum-only guard cannot catch this.
    rng=bytearray(fields[13]); rng[-25]^=1; fields[13]=bytes(rng); fields[18]=b''
    def frame():
        return domain+bytes([0,2,19])+b''.join(bytes([i+1])+len(v).to_bytes(8,'big')+v for i,v in enumerate(fields))
    unsigned=frame()
    fields[18]=hashlib.sha256(b'DDN-RESUMABLE-COMPLETE-STATE-INTEGRITY-V2'+bytes([0,0,2])+len(unsigned).to_bytes(8,'big')+unsigned).digest()
    changed=frame(); snapshot_hash=hashlib.sha256(changed).digest()
    binding=[row[0],row[1] or b'',row[2],row[3],row[4],snapshot_hash,len(changed).to_bytes(8,'big'),row[6]]
    preimage=b'DDN-VC04-SNAPSHOT-V2-DURABLE-TRANSACTION-V1'+bytes([0,0,1,8])+b''.join(bytes([i+1])+len(v).to_bytes(4,'big')+v for i,v in enumerate(binding))
    new_digest=hashlib.sha256(preimage).digest()
    db.execute('UPDATE vc04_prepared_payloads SET canonical_snapshot_v2_bytes=?,snapshot_v2_sha256=?,transaction_digest=? WHERE transaction_digest=?',(changed,snapshot_hash,new_digest,digest))
    for table in ['vc04_project_revisions','vc04_project_heads']:
        db.execute('UPDATE '+table+' SET transaction_digest=? WHERE transaction_digest=?',(new_digest,digest))
    digest=new_digest
db.commit()
rows={}
for table in ['vc04_prepared_payloads','vc04_project_revisions','vc04_project_heads']:
    rows[table]=[[v.hex() if isinstance(v,bytes) else v for v in row] for row in db.execute('SELECT * FROM '+table+' ORDER BY 1')]
print(json.dumps({'rows_sha256':hashlib.sha256(json.dumps(rows,sort_keys=True,separators=(',',':')).encode()).hexdigest(),'transaction':digest.hex()}))
db.close()
`;
    const database=path.join(copy,'vc04_snapshot_v2.sqlite3');
    const sql=(operation)=>spawnSync('python',['-c',inspect,database,operation,args[12]],{encoding:'utf8',timeout:30_000});
    const mutated=sql(kind);assert.ifError(mutated.error);assert.equal(mutated.status,0,mutated.stderr);
    const mutatedState=JSON.parse(mutated.stdout);copiedArgs[12]=mutatedState.transaction;
    const rejected=spawnSync(executable,copiedArgs,{encoding:'utf8',timeout:30_000});
    assert.ifError(rejected.error);assert.notEqual(rejected.status,0,kind+' must not resume');
    assert.equal(rejected.stdout,'',kind+' must not publish partial success');
    const after=sql('inspect');assert.ifError(after.error);assert.equal(after.status,0,after.stderr);
    assert.equal(JSON.parse(after.stdout).rows_sha256,mutatedState.rows_sha256,
      kind+' must not silently repair or replace the saved head');
    const intact=spawnSync(executable,args,{encoding:'utf8',timeout:30_000});
    assert.ifError(intact.error);assert.equal(intact.status,0,intact.stderr);
    assert.deepEqual(JSON.parse(intact.stdout),expected,'original normal checkpoint must remain runnable');
    rows.push({kind,store:copy,exit:rejected.status,stdout_bytes:0,
      persisted_rows_unchanged:true,original_checkpoint_green:true,diagnostic:rejected.stderr.trim()});
  }
  return rows;
}

// Actual shipped multi-file source, not test-only source rewriting/lowering.
if(process.env.DDN_NATIVE_TETRIS_PROJECT==='1') {
  const wasmDir=process.env.DDN_WASM_DIR,binDir=process.env.DDN_NATIVE_BIN_DIR;
  assert.ok(wasmDir && binDir);
  const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
  await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
  const sample=path.resolve('solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris');
  const authored=wasm.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',
    source_snapshot:{root_path:'root.ddn',entry_path:'main.ddn',files:['root.ddn','main.ddn'].map(name=>({path:name,text:fs.readFileSync(path.join(sample,name),'utf8')}))}}));
  const originalSource=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn','utf8');
  const template=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json','utf8');
  const binding=JSON.parse(wasm.DdnLogicalInputVm.bind_authored_project_json(authored,template));
  const oldBinding=JSON.parse(wasm.DdnLogicalInputVm.bind_source_json(originalSource,template));
  const vm=wasm.DdnLogicalInputVm.from_authored_project(authored,JSON.stringify(binding));
  const oldVm=new wasm.DdnLogicalInputVm(originalSource,JSON.stringify(oldBinding));
  let predecessorVm=null,predecessorModule=null;
  const predecessorDir=process.env.DDN_PREDECESSOR_WASM_DIR;
  if(predecessorDir) {
    const predecessor=await import(pathToFileURL(path.join(predecessorDir,'ddonirang_tool.js')).href);
    predecessorModule=predecessor;
    await predecessor.default({module_or_path:fs.readFileSync(path.join(predecessorDir,'ddonirang_tool_bg.wasm'))});
    predecessorVm=predecessor.DdnLogicalInputVm.from_authored_project(authored,JSON.stringify(binding));
  }
  const hashImpact=[];
  const snapshots=[],held={...binding.initial_held};
  const moves=[[],[['왼쪽','Press']],[['왼쪽','Release'],['회전','Press']],
    [['회전','Release'],['떨어뜨리기','Press']],[['떨어뜨리기','Release']],[['오른쪽','Press']],[['오른쪽','Release']]];
  for(let madi=0;madi<moves.length;madi++) {
    const events=moves[madi].map(([channel,kind],ordinal)=>{held[channel]=kind!=='Release';return {ordinal,channel,event:{kind}};});
    const snapshot={madi,events,held:{...held}};snapshots.push(snapshot);
    const actual=JSON.parse(vm.step_json(JSON.stringify(snapshot)));
    const previous=JSON.parse(oldVm.step_json(JSON.stringify(snapshot)));
    assert.equal(actual.frame.accepted_world_digest,previous.frame.accepted_world_digest,'named Start must preserve the existing Tetris world at madi '+madi);
    if(predecessorVm) {
      const old=JSON.parse(predecessorVm.step_json(JSON.stringify(snapshot)));
      const oldSummary=JSON.parse(predecessorVm.summary_json()),newSummary=JSON.parse(vm.summary_json());
      const oldValues={...oldSummary.resources.value_json},newValues={...newSummary.resources.value_json};
      if(Object.hasOwn(oldValues,'고정')) {
        assert.ok(oldValues['고정']==='참'||oldValues['고정']==='거짓','fixed predecessor must contain the actual legacy Bool spelling');
        assert.equal(newValues['고정'],oldValues['고정']==='참');
        delete oldValues['고정'];delete newValues['고정'];
      }
      assert.deepEqual(newValues,oldValues,'non-migrated Tetris game values must remain unchanged');
      hashImpact.push({madi,old_state_hash:oldSummary.state_hash,new_state_hash:newSummary.state_hash,
        old_accepted_world_digest:old.frame.accepted_world_digest,new_accepted_world_digest:actual.frame.accepted_world_digest,
        old_frame_sha256:createHash('sha256').update(JSON.stringify(old)).digest('hex'),
        new_frame_sha256:createHash('sha256').update(JSON.stringify(actual)).digest('hex'),game_values_equal:true,
        bool_projection_change:'legacy Korean text -> typed JSON Bool; impact comparison only, never a product restore converter'});
    }
  }
  const expected=JSON.parse(vm.summary_json());
  assert.deepEqual(expected.resources.json,{},'new typed execution must not retain legacy resources');
  assert.equal(expected.resources.value_det['고정'].type,'bool');
  assert.equal(expected.resources.value_det['행동'].type,'string');
  let legacyRestore='NOT_RUN';
  if(predecessorModule) {
    // A real old producer, not a manufactured typed capsule with JSON inside.
    const oldStateVm=new predecessorModule.DdnWasmVm('');
    oldStateVm.set_param('고정',false);oldStateVm.set_param('행동','왼쪽');
    const oldState=oldStateVm.get_state_json();oldStateVm.free();
    const newStateVm=new wasm.DdnWasmVm('');
    const before=newStateVm.get_state_json();
    const oldStateText=typeof oldState==='string'?oldState:JSON.stringify(oldState);
    assert.throws(()=>newStateVm.restore_state(oldStateText),/legacy resource_json|E_RESOURCE_SNAPSHOT_INVALID/);
    assert.deepEqual(newStateVm.get_state_json(),before,'legacy restore rejection must not change any VM state');
    newStateVm.free();legacyRestore='REAL_R17_LEGACY_SNAPSHOT_REJECTED_WITH_STATE_UNCHANGED';
  }
  const checkUnsupported=process.env.DDN_TETRIS_CHECKPOINT_REJECTION==='1';
  if(checkUnsupported) {
    assert.throws(()=>vm.export_saved_project(new Uint8Array(),new Uint8Array([2])),/E_PROJECT_RUNTIME_LEGACY_CHANNEL_UNSUPPORTED/);
    assert.deepEqual(JSON.parse(vm.summary_json()),expected,'rejected export must not change the live world');
  }
  let exported=null;
  if(!checkUnsupported) {
    exported=vm.export_saved_project(new Uint8Array(),new Uint8Array([2]));
    assert.equal(typeof exported,'string');
    assert.deepEqual(JSON.parse(vm.summary_json()),expected,'successful checkpoint export must not change the world');
  }
  vm.free();oldVm.free();predecessorVm?.free();
  const output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-native-tetris-project-'));
  const stageReport={scope:'AUTHORED_TETRIS_PRODUCT_PROGRESS',source_status:'WIP_NOT_FIXED',output,
    wasm_sha256:createHash('sha256').update(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))).digest('hex'),
    source_hashes:Object.fromEntries(['root.ddn','main.ddn'].map(name=>[name,createHash('sha256').update(fs.readFileSync(path.join(sample,name))).digest('hex')])),
    wasm_existing_world_parity:{passed:7,failed:0},native_checkpoint:'NOT_RUN',browser_cold_resume:'NOT_RUN',release_human:'NOT_CLAIMED'};
  fs.writeFileSync(path.join(output,'progress.json'),JSON.stringify(stageReport,null,2));
  fs.writeFileSync(path.join(output,'hash-impact.json'),JSON.stringify({scope:'R17_TO_TYPED_TEXT_BOOL_TETRIS',
    predecessor_wasm_sha256:predecessorDir?createHash('sha256').update(fs.readFileSync(path.join(predecessorDir,'ddonirang_tool_bg.wasm'))).digest('hex'):null,
    new_wasm_sha256:stageReport.wasm_sha256,rows:hashImpact},null,2));
  if(exported!==null)fs.writeFileSync(path.join(output,'wasm-exported-checkpoint.json'),exported);
  const authoredFile=path.join(output,'authored.json'),prefix=path.join(output,'prefix.json'),suffix=path.join(output,'suffix.json'),none=path.join(output,'none.json');
  fs.writeFileSync(authoredFile,authored);fs.writeFileSync(prefix,JSON.stringify({binding,snapshots:snapshots.slice(0,3)}));
  fs.writeFileSync(suffix,JSON.stringify({binding,snapshots:snapshots.slice(3)}));fs.writeFileSync(none,JSON.stringify({binding,snapshots:[]}));
  const rows=[];
  for(const name of ['ddonilang','ddonirang','또니랑']) {
    const executable=path.join(binDir,name+(process.platform==='win32'?'.exe':''));
    const store=fs.mkdtempSync(path.join(output,'store-'));
    const recorded=spawnSync(executable,['작품자료','논리기록',authoredFile,'--입력',prefix,'--부모','처음','--판','02','--저장소',store,'--저장소-초기화'],{encoding:'utf8',timeout:60_000});
    if(checkUnsupported) {
      assert.ifError(recorded.error);assert.notEqual(recorded.status,0);
      assert.equal(recorded.stdout,'','unsupported storage must not publish a partial success');
      assert.match(recorded.stderr,/E_PROJECT_RUNTIME_LEGACY_CHANNEL_UNSUPPORTED/);
      assert.match(recorded.stderr,/고정, 행동/);
      assert.deepEqual(fs.readdirSync(store),[],'rejection must happen before storage initialization');
      rows.push({name,unsupported_checkpoint_rejected:true,stdout_bytes:0,storage_unchanged:true});
      continue;
    }
    if(recorded.status!==0) {
      stageReport.native_checkpoint={name,status:recorded.status,stderr:recorded.stderr,stdout_bytes:Buffer.byteLength(recorded.stdout??'')};
      fs.writeFileSync(path.join(output,'progress.json'),JSON.stringify(stageReport,null,2));
      console.log('progress evidence: '+path.join(output,'progress.json'));
    }
    assert.ifError(recorded.error);assert.equal(recorded.status,0,recorded.stderr);
    const saved=JSON.parse(recorded.stdout),file=path.join(store,'saved.json'),receipt=path.join(store,'receipt.hex');
    fs.writeFileSync(file,saved.saved_artifact_json_utf8);fs.writeFileSync(receipt,saved.committed_receipt_hex);
    fs.writeFileSync(path.join(store,'record-response.json'),recorded.stdout);
    const resumeArgs=['작품자료','이어실행',file,'--부모','처음','--판','02','--마디수','4','--저장소',store,'--거래',saved.transaction_digest_sha256,'--입력',suffix];
    const lockedStorage=process.env.DDN_NATIVE_LOCKED_STORAGE==='1'
      ? await nativeLockedStorageGuard(executable,resumeArgs,store,saved.committed_receipt_hex,expected):'NOT_RUN';
    const resumed=spawnSync(executable,['작품자료','이어실행',file,'--부모','처음','--판','02','--마디수','4','--저장소',store,'--거래',saved.transaction_digest_sha256,'--입력',suffix,'--저장-판','03','--이전-영수증',receipt],{encoding:'utf8',timeout:60_000});
    assert.ifError(resumed.error);assert.equal(resumed.status,0,resumed.stderr);
    const continued=JSON.parse(resumed.stdout);assert.deepEqual(continued.summary,expected);
    fs.writeFileSync(file,continued.saved_artifact_json_utf8);
    const reopened=spawnSync(executable,['작품자료','이어실행',file,'--부모','02','--판','03','--마디수','0','--저장소',store,'--거래',continued.transaction_digest_sha256,'--입력',none],{encoding:'utf8',timeout:60_000});
    assert.ifError(reopened.error);assert.equal(reopened.status,0,reopened.stderr);assert.deepEqual(JSON.parse(reopened.stdout),expected);
    rows.push({name,store,record_pid:recorded.pid,resume_pid:resumed.pid,reopen_pid:reopened.pid,save_resume_resave_reopen_equal:true,locked_storage:lockedStorage});
  }
  const report={scope:checkUnsupported?'AUTHORED_TETRIS_CHECKPOINT_REJECTION_CLI3_WASM':'SHIPPED_AUTHORED_TETRIS_NATIVE_CLI3_WASM',source_status:'WIP_NOT_FIXED',output,rows,
    native_checkpoint:checkUnsupported?'UNSUPPORTED_EXPLICITLY_REJECTED':'VERIFIED',
    wasm_export:checkUnsupported?'UNSUPPORTED_EXPLICITLY_REJECTED':'EXPORTED_WITHOUT_WORLD_CHANGE',
    same_contract_single_file_worlds_preserved:true,legacy_restore:legacyRestore,
    hash_impact_file:'hash-impact.json',browser_cold_resume:'NOT_RUN',release_human:'NOT_CLAIMED'};
  fs.writeFileSync(path.join(output,'result.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));process.exit(0);
}

// Optional Native complete-state cold admission. The fixture directory is
// emitted by the common Rust product-encoder test, never JS lowering.
if(process.env.DDN_NATIVE_STATEFUL_FIXTURE==='1') {
  const wasmDir=process.env.DDN_WASM_DIR;assert.ok(wasmDir);
  const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
  await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
  const sample=path.resolve('solutions/seamgrim_ui_mvp/samples/beta1_logical_counter');
  const files=['root.ddn','app/main.ddn','lib/힘.ddn'].map(name=>({path:name,text:fs.readFileSync(path.join(sample,name),'utf8')}));
  const authored=wasm.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',
    source_snapshot:{root_path:'root.ddn',entry_path:'app/main.ddn',files}}));
  const template=fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.binding.json','utf8');
  const counterTemplate={...JSON.parse(template),input_schema:'beta1.counter.controls@1',
    channel_registry:'beta1.counter.channels@1',recording_policy:'beta1.counter.recording@1',channels:{},initial_held:{}};
  const binding=JSON.parse(wasm.DdnLogicalInputVm.bind_authored_project_json(authored,JSON.stringify(counterTemplate)));
  const vm=wasm.DdnLogicalInputVm.from_authored_project(authored,JSON.stringify(binding));
  const snapshots=[0,1].map(madi=>({madi,events:[],held:{...binding.initial_held}}));
  vm.step_json(JSON.stringify(snapshots[0]));
  const first=JSON.parse(vm.summary_json());
  assert.equal(first.resources.fixed64['누적'],'3','independent golden: 0 + 3');
  vm.step_json(JSON.stringify(snapshots[1]));
  const expected=JSON.parse(vm.summary_json());
  assert.equal(expected.resources.fixed64['누적'],'6','independent golden: 3 + 3');
  assert.notEqual(first.state_hash,expected.state_hash,'continuation must change the actual world');
  vm.free();
  const fixture=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-stateful-native-checkpoint-'));
  fs.writeFileSync(path.join(fixture,'authored-project.json'),authored);
  fs.writeFileSync(path.join(fixture,'initial-input.json'),JSON.stringify({binding,snapshots:snapshots.slice(0,1)}));
  fs.writeFileSync(path.join(fixture,'next-input.json'),JSON.stringify({binding,snapshots:snapshots.slice(1)}));
  fs.writeFileSync(path.join(fixture,'failing-next-input.json'),JSON.stringify({binding,snapshots:[{...snapshots[1],madi:99}]}));
  fs.writeFileSync(path.join(fixture,'continuous-expected.json'),JSON.stringify(expected));
  fs.writeFileSync(path.join(fixture,'cold-resume-identity.json'),JSON.stringify({parent:'처음',revision:'02',madi:1}));
  process.env.DDN_NATIVE_CHECKPOINT_DIR=fixture;
}

if(process.env.DDN_NATIVE_CHECKPOINT_DIR) {
  const root=path.resolve(process.env.DDN_NATIVE_CHECKPOINT_DIR);
  const binDir=process.env.DDN_NATIVE_BIN_DIR;
  assert.ok(binDir,'DDN_NATIVE_BIN_DIR must identify the tested Native build');
  const identity=JSON.parse(fs.readFileSync(path.join(root,'cold-resume-identity.json'),'utf8'));
  const expected=JSON.parse(fs.readFileSync(path.join(root,'continuous-expected.json'),'utf8'));
  const originalArgs=['작품자료','이어실행',path.join(root,'saved-project.json'),'--부모',identity.parent,
    '--판',identity.revision,'--마디수',String(identity.madi),'--저장소',root,
    '--거래',identity.transaction,'--입력',path.join(root,'next-input.json')];
  const rows=[];
  for(const name of ['ddonilang','ddonirang','또니랑']) {
    const executable=path.join(binDir,name+(process.platform==='win32'?'.exe':''));
    let args=[...originalArgs];let recordPid=null;let recordedContext=null;
    if(process.env.DDN_NATIVE_RECORD==='1') {
      const store=fs.mkdtempSync(path.join(root,`cli-${name}-`));
      const recording=['작품자료','논리기록',path.join(root,'authored-project.json'),
        '--입력',path.join(root,'initial-input.json'),'--부모','처음','--판','02','--저장소',store,'--저장소-초기화'];
      const recorded=spawnSync(executable,recording,{encoding:'utf8',timeout:30_000});
      assert.ifError(recorded.error);assert.equal(recorded.status,0,recorded.stderr);
      const saved=JSON.parse(recorded.stdout);
      assert.deepEqual(Object.keys(saved).sort(),['summary','saved_artifact_json_utf8','committed_receipt_hex','transaction_digest_sha256'].sort());
      assert.equal(saved.summary.madi_count,1);assert.match(saved.committed_receipt_hex,/^[0-9a-f]+$/);
      assert.match(saved.transaction_digest_sha256,/^[0-9a-f]{64}$/);
      const file=path.join(store,'saved-project.json');
      fs.writeFileSync(file,saved.saved_artifact_json_utf8,'utf8'); // exact Rust export, no source/state rewriting
      args=['작품자료','이어실행',file,'--부모','처음','--판','02','--마디수','1',
        '--저장소',store,'--거래',saved.transaction_digest_sha256,'--입력',path.join(root,'next-input.json')];
      recordPid=recorded.pid;
      recordedContext={store,recording,saved};
      const duplicate=spawnSync(executable,recording,{encoding:'utf8',timeout:30_000});
      assert.ifError(duplicate.error);assert.notEqual(duplicate.status,0);assert.equal(duplicate.stdout,'');
    }
    const run=spawnSync(executable,args,{encoding:'utf8',timeout:30_000});
    assert.ifError(run.error);assert.equal(run.status,0,run.stderr);
    if(recordPid!==null)assert.notEqual(recordPid,run.pid,'restore must use a new OS process');
    const actual=JSON.parse(run.stdout);assert.deepEqual(actual,expected,name+'/continuous execution');
    const mismatch=[...args];mismatch[8]=String(identity.madi+1);
    const rejected=spawnSync(executable,mismatch,{encoding:'utf8',timeout:30_000});
    assert.ifError(rejected.error);assert.notEqual(rejected.status,0);
    assert.equal(rejected.stdout,'',name+'/no partial publication');
    assert.match(rejected.stderr,/E_LOGICAL_INPUT_REQUEST/);
    const failedSuffix=[...args];failedSuffix[8]='2';
    failedSuffix[14]=path.join(root,'failed-next-input.json');
    const failed=spawnSync(executable,failedSuffix,{encoding:'utf8',timeout:30_000});
    assert.ifError(failed.error);assert.notEqual(failed.status,0);
    assert.equal(failed.stdout,'',name+'/no publication after first successful suffix madi');
    assert.match(failed.stderr,/E_LOGICAL_/);
    const retry=spawnSync(executable,args,{encoding:'utf8',timeout:30_000});
    assert.ifError(retry.error);assert.equal(retry.status,0,retry.stderr);
    assert.deepEqual(JSON.parse(retry.stdout),expected,name+'/stored checkpoint preserved after failed suffix');
    const corruptionCases=process.env.DDN_NATIVE_STORAGE_MUTATIONS==='1'
      ? nativeCorruptionGuards(executable,args,recordedContext?.store??root,expected):'NOT_RUN';
    let freshProjectCas=false;
    let continuedCheckpoint=false;
    let lockedStorage='NOT_RUN';
    if(recordedContext) {
      const {store,recording,saved}=recordedContext;
      if(process.env.DDN_NATIVE_LOCKED_STORAGE==='1')
        lockedStorage=await nativeLockedStorageGuard(executable,args,store,saved.committed_receipt_hex,expected);
      const receiptFile=path.join(store,'previous-receipt.hex');fs.writeFileSync(receiptFile,saved.committed_receipt_hex+'\n');
      let currentRevision='02',nextRevision='03';
      if(process.env.DDN_NATIVE_RESAVE==='1') {
        const continued=spawnSync(executable,[...args,'--저장-판','03','--이전-영수증',receiptFile],{encoding:'utf8',timeout:30_000});
        assert.ifError(continued.error);assert.equal(continued.status,0,continued.stderr);
        const checkpoint=JSON.parse(continued.stdout);assert.deepEqual(checkpoint.summary,expected);
        const file=path.join(store,'continued-revision3.json');fs.writeFileSync(file,checkpoint.saved_artifact_json_utf8);
        const noInput=path.join(store,'no-input.json');
        fs.writeFileSync(noInput,JSON.stringify({binding:JSON.parse(fs.readFileSync(path.join(root,'next-input.json'),'utf8')).binding,snapshots:[]}));
        const restore=[...args];restore[2]=file;restore[4]='02';restore[6]='03';restore[8]='0';
        restore[12]=checkpoint.transaction_digest_sha256;restore[14]=noInput;
        const staleSave=spawnSync(executable,[...restore,'--저장-판','04','--이전-영수증',receiptFile],{encoding:'utf8',timeout:30_000});
        assert.ifError(staleSave.error);assert.notEqual(staleSave.status,0);assert.equal(staleSave.stdout,'');
        const cold=spawnSync(executable,restore,{encoding:'utf8',timeout:30_000});
        assert.ifError(cold.error);assert.equal(cold.status,0,cold.stderr);assert.deepEqual(JSON.parse(cold.stdout),expected);
        fs.writeFileSync(receiptFile,checkpoint.committed_receipt_hex+'\n');
        currentRevision='03';nextRevision='04';continuedCheckpoint=true;
      }
      const nextRecord=recording.slice(0,-1);nextRecord[6]=currentRevision;nextRecord[8]=nextRevision;
      nextRecord.push('--이전-영수증',receiptFile);
      const nextSaved=spawnSync(executable,nextRecord,{encoding:'utf8',timeout:30_000});
      assert.ifError(nextSaved.error);assert.equal(nextSaved.status,0,nextSaved.stderr);
      const revision=JSON.parse(nextSaved.stdout);assert.deepEqual(revision.summary,saved.summary);
      const nextFile=path.join(store,'saved-revision3.json');fs.writeFileSync(nextFile,revision.saved_artifact_json_utf8);
      const nextResume=[...args];nextResume[2]=nextFile;nextResume[4]=currentRevision;nextResume[6]=nextRevision;nextResume[12]=revision.transaction_digest_sha256;
      const stale=spawnSync(executable,nextRecord,{encoding:'utf8',timeout:30_000});
      assert.ifError(stale.error);assert.notEqual(stale.status,0);assert.equal(stale.stdout,'');
      const preserved=spawnSync(executable,nextResume,{encoding:'utf8',timeout:30_000});
      assert.ifError(preserved.error);assert.equal(preserved.status,0,preserved.stderr);
      assert.deepEqual(JSON.parse(preserved.stdout),expected,name+'/fresh project revision CAS and stale-parent preservation');
      freshProjectCas=true;
    }
    rows.push({name,record_process_pid:recordPid,restore_process_pid:run.pid,exit:run.status,continuous_equal:true,
      summary_sha256:createHash('sha256').update(JSON.stringify(actual)).digest('hex'),
      input_count_rejection_exit:rejected.status,suffix_failure_exit:failed.status,
      checkpoint_preserved:true,continued_checkpoint:continuedCheckpoint,fresh_project_cas:freshProjectCas,
      corruption_cases:corruptionCases,locked_storage:lockedStorage,partial_stdout:false});
  }
  const report={scope:process.env.DDN_NATIVE_RECORD==='1'?'NATIVE_CLI_RECORD_EXIT_REOPEN_RESUME':'NATIVE_LOGICAL_COMPLETE_STATE_COLD_RESUME',source_status:'WIP_NOT_FIXED',
    fixture:root,rows,wasm:'NOT_RUN',workbench:'NOT_RUN',release_human:'NOT_CLAIMED'};
  if(process.env.DDN_NATIVE_REPORT)fs.writeFileSync(process.env.DDN_NATIVE_REPORT,JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify(report,null,2));
  process.exit(0);
}

// A3-P11 actual product save/restore, not a JS-made replacement snapshot.
// Storage commit durability and browser publication are separate gates.
const wasmDir=process.env.DDN_WASM_DIR;
assert.ok(wasmDir,'DDN_WASM_DIR must identify the newly built product WASM');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
const binary=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
await wasm.default({module_or_path:binary});
if(process.env.DDN_AUTHORED_LOGICAL_FIXTURE) {
  const fixture=path.resolve(process.env.DDN_AUTHORED_LOGICAL_FIXTURE);
  const authoredText=fs.readFileSync(path.join(fixture,'authored-project.json'),'utf8');
  const first=JSON.parse(fs.readFileSync(path.join(fixture,'initial-input.json'),'utf8'));
  const next=JSON.parse(fs.readFileSync(path.join(fixture,'next-input.json'),'utf8'));
  const expected=JSON.parse(fs.readFileSync(path.join(fixture,'continuous-expected.json'),'utf8'));
  const logical=wasm.DdnLogicalInputVm.from_authored_project(authoredText,JSON.stringify(first.binding));
  for(const snapshot of first.snapshots)logical.step_json(JSON.stringify(snapshot));
  const before=logical.summary_json();
  const carrier=logical.export_saved_project(new Uint8Array(),new Uint8Array([2]));
  assert.equal(JSON.parse(carrier).schema,'seamgrim.project_snapshot_artifact.v1');
  assert.equal(logical.summary_json(),before,'export must not advance the world');
  for(const snapshot of next.snapshots)logical.step_json(JSON.stringify(snapshot));
  assert.deepEqual(JSON.parse(logical.summary_json()),expected,'authored WASM and Native continuous run must agree');
  assert.throws(()=>wasm.DdnLogicalInputVm.from_authored_project('{}',JSON.stringify(first.binding)));
  assert.deepEqual(JSON.parse(logical.summary_json()),expected,'rejected admission must not mutate the existing run');
  logical.free();
  const {WasmRuntimeAdapter}=await import('../solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js');
  const adapter=new WasmRuntimeAdapter();
  adapter.projectModulePromise=Promise.resolve(wasm);
  const session=await adapter.createLogicalSession({authoredProjectJson:authoredText,bindingJson:JSON.stringify(first.binding)});
  for(const snapshot of first.snapshots)session.step(snapshot);
  const exported=session.exportSavedProjectArtifact(new Uint8Array(),new Uint8Array([2]));
  assert.equal(JSON.parse(exported).schema,JSON.parse(carrier).schema);
  assert.equal(session.exportSavedProjectArtifact(new Uint8Array(),new Uint8Array([2])),exported,
    'repeated export of the same run must preserve the exact Rust carrier');
  for(const snapshot of next.snapshots)session.step(snapshot);
  assert.deepEqual(session.summary(),expected);
  session.close();
  console.log('authored logical WASM live execution/export: PASS (durable resume NOT_RUN)');
}
assert.equal(typeof wasm.DdnWasmVm.from_saved_project_artifact_json,'function');
const live=new wasm.DdnWasmVm('1 보여주기.');
const before=String(live.get_state_json());
const beforeHash=live.get_state_hash();
const cases=[
  ['{}','','02','E_PROJECT_RUNTIME_ARTIFACT_FIELD_COUNT'],
  ['{}','','','E_PROJECT_RUNTIME_STORAGE_LINEAGE'],
  ['{}','GG','02','E_PROJECT_RUNTIME_STORAGE_LINEAGE'],
  ['{}','','0','E_PROJECT_RUNTIME_STORAGE_LINEAGE'],
  ['{"schema":"a","schema":"b"}','','02','E_PROJECT_RUNTIME_ARTIFACT_JSON'],
];
for(const [artifact,parent,revision,code] of cases) {
  assert.throws(()=>wasm.DdnWasmVm.from_saved_project_artifact_json(artifact,parent,revision),
    error=>String(error).includes(code),code);
  assert.equal(live.get_state_hash(),beforeHash,'failed candidate must not mutate the live world');
  assert.equal(String(live.get_state_json()),before,'failed candidate must not change tick, input or observations');
}
live.free();
const root='(글판="v25", 모듬=차림[(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음.\n';
const body='실행:움직씨 := { 판 <- (차림=차림[차림[1,2],차림[3,4]],원소씨=셈수,모양=(2,2)) 텐서로. 검산 <- 판[1,1]. }.\r\n';
const authored=wasm.authored_project_operation('create',JSON.stringify({
  source_profile:'v1-core-v25',module_name:'',source_snapshot:{root_path:'root.ddn',entry_path:'app/main.ddn',
    files:[{path:'root.ddn',text:root},{path:'app/main.ddn',text:body}]},
}));
const vm=wasm.DdnWasmVm.from_authored_project_json(authored);
vm.set_rng_seed(9n);
vm.step_one();
const original=JSON.parse(vm.get_state_json());
assert.equal(original.state.snapshot_v2.resources.value_det?.판?.type,'tensor_v1','the saved world must actually contain a Tensor, not just a local temporary');
const file=vm.export_saved_project_artifact_json('','02');
const saved=JSON.parse(file);
assert.equal(saved.schema,'seamgrim.project_snapshot_artifact.v1');
// A3-P11 / RD-686: inspect the actual product-issued integrated envelope,
// not a handcrafted substitute for a source/runtime attachment.
const topKeys=['schema','project_snapshot','runtime_attachment','canonical_json','content_sha256','canonical_byte_length'];
const attachmentKeys=['schema','source_profile_identity','project_execution_snapshot_identity_hex','revision_envelope_base64url','revision_envelope_byte_length','revision_envelope_sha256','runtime_state_json_utf8','runtime_state_byte_length','runtime_state_sha256','resource_snapshot_v2_json_utf8','resource_snapshot_v2_byte_length','resource_snapshot_v2_sha256'];
assert.deepEqual(Object.keys(saved),topKeys);
assert.deepEqual(Object.keys(saved.runtime_attachment),attachmentKeys);
assert.equal(saved.runtime_attachment.schema,'ddn.workbench.project_runtime_attachment.v1');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const attachment=saved.runtime_attachment;
const envelopeBytes=Buffer.from(attachment.revision_envelope_base64url,'base64url');
assert.equal(envelopeBytes.length,attachment.revision_envelope_byte_length);
assert.equal(sha(envelopeBytes),attachment.revision_envelope_sha256);
const envelope=JSON.parse(envelopeBytes.toString('utf8'));
assert.deepEqual(Object.keys(envelope).sort(),['schema','project_identity','history_revision_identity','expected_parent_revision_identity','source_epoch_identity','project_semantic_configuration_identity','official_entry_path','official_entry_definition_identity','source_manifest','dependency_resolution_identity','declared_resource_members','revision_integrity_sha256'].sort());
assert.equal(envelope.schema,'ddn.workbench.project_snapshot_revision.v1');
assert.equal(envelope.official_entry_path,'app/main.ddn');
for(const name of ['project_identity','history_revision_identity','source_epoch_identity','project_semantic_configuration_identity','official_entry_definition_identity'])assert.ok(envelope[name],name+'/actual owner binding');
for(const [text,length,hash] of [
  ['runtime_state_json_utf8','runtime_state_byte_length','runtime_state_sha256'],
  ['resource_snapshot_v2_json_utf8','resource_snapshot_v2_byte_length','resource_snapshot_v2_sha256'],
]){
  assert.equal(Buffer.byteLength(attachment[text]),attachment[length]);
  assert.equal(sha(attachment[text]),attachment[hash]);
}
const storedResources=JSON.parse(attachment.resource_snapshot_v2_json_utf8);
assert.equal(storedResources.value_det?.판?.type??storedResources.values?.판?.type??storedResources.typed_values?.판?.type,'tensor_v1','RD-683 carrier remains in the integrated attachment');
const canonical=JSON.stringify({project_snapshot:saved.project_snapshot,runtime_attachment:attachment});
assert.equal(saved.canonical_json,canonical);
assert.equal(saved.canonical_byte_length,Buffer.byteLength(canonical));
assert.equal(saved.content_sha256,sha(canonical));
const attachmentRejects=[];
const rejectCandidate=(text,label)=>{
  const unchanged=vm.get_state_json(),hash=vm.get_state_hash();
  let diagnostic='';
  try{wasm.DdnWasmVm.from_saved_project_artifact_json(text,'','02');assert.fail(label+'/must reject');}
  catch(error){diagnostic=String(error);assert.match(diagnostic,/E_PROJECT_RUNTIME_|E_REVISION_/);}
  assert.equal(vm.get_state_json(),unchanged,label+'/no partial live publication');
  assert.equal(vm.get_state_hash(),hash,label+'/world unchanged');
  attachmentRejects.push({label,diagnostic,live_state_preserved:true});
};
rejectCandidate(file.replace('"runtime_attachment":{','"runtime_attachment":{"schema":"duplicate",'),'duplicate-attachment-key');
const missingRevision=structuredClone(saved);
const missingEnvelope=structuredClone(envelope);delete missingEnvelope.official_entry_definition_identity;
missingEnvelope.revision_integrity_sha256='';
missingEnvelope.revision_integrity_sha256=sha(JSON.stringify(missingEnvelope)+'\n');
const missingBytes=Buffer.from(JSON.stringify(missingEnvelope)+'\n');
Object.assign(missingRevision.runtime_attachment,{revision_envelope_base64url:missingBytes.toString('base64url'),revision_envelope_byte_length:missingBytes.length,revision_envelope_sha256:sha(missingBytes)});
missingRevision.canonical_json=JSON.stringify({project_snapshot:missingRevision.project_snapshot,runtime_attachment:missingRevision.runtime_attachment});
missingRevision.canonical_byte_length=Buffer.byteLength(missingRevision.canonical_json);
missingRevision.content_sha256=sha(missingRevision.canonical_json);
rejectCandidate(JSON.stringify(missingRevision),'missing-revision-definition-with-valid-digests');
assert.equal(saved.project_snapshot.schema,'ddn.saved_runtime_project.v1');
assert.equal(JSON.parse(saved.project_snapshot.authored_project_json_utf8).source_snapshot.files.find(f=>f.path==='app/main.ddn').text,body);
// RD-643: a second actual WASM instance models a producer restart. Reopening
// recorded captures cannot reset the fresh admission counter beneath them.
const coldUrl=pathToFileURL(path.join(wasmDir,'ddonirang_tool.js'));
coldUrl.searchParams.set('instance','cold-reopen-high-water');
const coldWasm=await import(coldUrl.href);
await coldWasm.default({module_or_path:binary});
const coldReopened=coldWasm.DdnWasmVm.from_saved_project_artifact_json(file,'','02');
const recordedCatalog=JSON.parse(saved.project_snapshot.source_capture_catalog_json_utf8);
const greatestRecorded=recordedCatalog.records.reduce((max,row)=>{
  const value=BigInt(JSON.parse(row.record_json_utf8).admission_ordinal_decimal);
  return value>max?value:max;
},0n);
const nextFresh=coldWasm.DdnWasmVm.from_authored_project_json(authored);
nextFresh.step_one();
const nextSaved=JSON.parse(nextFresh.export_saved_project_artifact_json('','06'));
const freshCatalog=JSON.parse(nextSaved.project_snapshot.source_capture_catalog_json_utf8);
for(const row of freshCatalog.records) {
  assert.ok(BigInt(JSON.parse(row.record_json_utf8).admission_ordinal_decimal)>greatestRecorded,
    'fresh admission after cold reopen must not reuse a recorded ordinal');
}
assert.equal(coldReopened.get_state_hash(),vm.get_state_hash(),'retaining the admission floor does not change the restored world');
coldReopened.free();nextFresh.free();
const reopened=wasm.DdnWasmVm.from_saved_project_artifact_json(file,'','02');
const recovered=JSON.parse(reopened.get_state_json());
assert.equal(reopened.get_state_hash(),vm.get_state_hash(),'restored world hash');
assert.deepEqual(recovered.state.snapshot_v2,original.state.snapshot_v2,'tensor/resource/params/history exact restoration');
assert.deepEqual(recovered.state.input,original.state.input,'actual input restoration');
const resaved=JSON.parse(reopened.export_saved_project_artifact_json('02','03'));
assert.equal(resaved.project_snapshot.source_capture_catalog_json_utf8,saved.project_snapshot.source_capture_catalog_json_utf8,'saving reopened VM never mints capture ordinals');
assert.equal(resaved.runtime_attachment.project_execution_snapshot_identity_hex,saved.runtime_attachment.project_execution_snapshot_identity_hex,'same authored source/configuration PESI');
vm.step_one();reopened.step_one();
assert.equal(reopened.get_state_hash(),vm.get_state_hash(),'the next actual committed tick agrees after restoration');
for(const [parent,revision] of [['09','02'],['','03']]) {
  assert.throws(()=>wasm.DdnWasmVm.from_saved_project_artifact_json(file,parent,revision));
  assert.equal(reopened.get_state_hash(),vm.get_state_hash());
}
reopened.update_logic_with_source_profile('검산 <- 8.','v1-core-v25');
assert.throws(()=>reopened.export_saved_project_artifact_json('03','04'),e=>String(e).includes('E_PROJECT_RUNTIME_OWNER_REQUIRED'),'editing source invalidates prior source owner');
vm.free();reopened.free();
const inputArtifact=wasm.authored_project_operation('create',JSON.stringify({
  source_profile:'v1-core-v25',module_name:'',source_snapshot:{root_path:'root.ddn',entry_path:'app/main.ddn',
    files:[{path:'root.ddn',text:root},{path:'app/main.ddn',text:'실행:움직씨 := { 새눌림 <- ("left") 막눌렸나. }.'}]},
}));
const inputVm=wasm.DdnWasmVm.from_authored_project_json(inputArtifact);
const initialInputState=inputVm.get_state_json();
assert.throws(()=>inputVm.configure_project_entry('다른시작'),error=>String(error).includes('E_PROJECT_RUNTIME_ENTRY_CHANGED'));
assert.throws(()=>inputVm.configure_authored_bogae('{}'),error=>String(error).includes('E_PROJECT_RUNTIME_OWNER_CHANGED'));
assert.equal(inputVm.get_state_json(),initialInputState,'entry and owner override refusals leave the complete VM unchanged');
inputVm.inject_ai_action('검산','1');
assert.throws(()=>inputVm.export_saved_project_artifact_json('','05'),error=>String(error).includes('E_PROJECT_RUNTIME_PENDING_INPUT'));
assert.throws(()=>inputVm.export_saved_project_artifact_json('','05'),error=>String(error).includes('E_PROJECT_RUNTIME_PENDING_INPUT'),'a rejected export does not silently discard pending input');
inputVm.clear_ai_injections();
assert.equal(inputVm.get_state_json(),initialInputState,'refused export neither commits nor loses pending input');
inputVm.step_one_with_input_fixed64_v1(2,'',0,0,1,0);
assert.equal(JSON.parse(inputVm.get_state_json()).resources.value_json?.새눌림,'참','first tick must actually exercise the newly-pressed query');
// Current host input can change after a commit. It is not the last input that
// the runner consumed, so restoration must retain both independently.
inputVm.set_input_fixed64_v1(0,'',0,0,1,0);
const inputFile=inputVm.export_saved_project_artifact_json('','05');
const inputState=JSON.parse(inputVm.get_state_json());
for (const history of [null,{schema:'ddn.runner_input_history.v1',consumed_keys_decimal:'02'},
  {schema:'ddn.runner_input_history.v1',consumed_keys_decimal:'18446744073709551616'},
  {schema:'ddn.runner_input_history.v1',consumed_keys_decimal:'2',extra:true}]) {
  const damaged=structuredClone(inputState);
  if(history===null) delete damaged.runner_input_history;
  else damaged.runner_input_history=history;
  assert.throws(()=>inputVm.restore_state(JSON.stringify(damaged)));
  assert.equal(inputVm.get_state_json(),JSON.stringify(inputState),'invalid history must not publish partial restoration');
}
const inputRestored=wasm.DdnWasmVm.from_saved_project_artifact_json(inputFile,'','05');
inputVm.step_one_with_input_fixed64_v1(2,'',0,0,1,0);
inputRestored.step_one_with_input_fixed64_v1(2,'',0,0,1,0);
assert.equal(JSON.parse(inputVm.get_state_json()).resources.value_json?.새눌림,'거짓','held input is not a fresh press in the original run');
assert.equal(JSON.parse(inputRestored.get_state_json()).resources.value_json?.새눌림,'거짓','restoration preserves the consumed input history');
assert.equal(inputRestored.get_state_hash(),inputVm.get_state_hash(),'held key is not a second newly pressed edge after reopening');
inputVm.free();inputRestored.free();
const evidenceDirectory=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-project-attachment-'));
fs.writeFileSync(path.join(evidenceDirectory,'actual_saved_artifact.json'),file);
fs.writeFileSync(path.join(evidenceDirectory,'revision_envelope.json'),envelopeBytes);
// Optional actual Native process cross-check. No DDN transformation, synthetic
// replacement snapshot or copied CLI golden: both consumers reopen the SAME
// artifact emitted above by the product WASM source/runtime owner.
const cliParity=[];
if(process.env.DDN_CLI_DIR) {
  const artifactPath=path.join(evidenceDirectory,'actual_saved_artifact.json');
  const artifactHash=sha(fs.readFileSync(artifactPath));
  const expectedVm=wasm.DdnWasmVm.from_saved_project_artifact_json(file,'','02');
  try {
    const output=JSON.parse(String(expectedVm.step_one()));
    const expectedState=JSON.parse(String(expectedVm.get_state_json()));
    const expectedInput=expectedState.state.snapshot_v2.public_observation_frames.at(-1).input_snapshot_sha256;
    for(const name of ['ddonilang','ddonirang','또니랑']) {
      const executable=path.join(process.env.DDN_CLI_DIR,name+(process.platform==='win32'?'.exe':''));
      assert.ok(fs.existsSync(executable),name+'/actual executable missing');
      const command=['작품자료','이어실행',artifactPath,'--부모','처음','--판','02','--마디수','1'];
      const run=spawnSync(executable,command,{encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024});
      assert.ifError(run.error);
      assert.equal(run.status,0,name+'/resume: '+run.stderr);
      const actual=JSON.parse(run.stdout);
      assert.equal(actual.ok,true,name+'/actual resume');
      assert.equal(actual.next_tick_id,expectedState.state.snapshot_v2.next_tick_id,name+'/next tick');
      assert.equal(actual.state_hash,expectedVm.get_state_hash(),name+'/world');
      // Native response includes the typed storage carrier; step_one exposes
      // the display projection. Compare each to its matching actual WASM
      // product channel, not two different DTO envelopes to one another.
      assert.equal(actual.resources.schema,'ddn.resource_snapshot.v2',name+'/resource schema');
      assert.equal(actual.resources.unit_wire_schema,'ddn.unit_value_wire.v2',name+'/unit wire schema');
      for(const channel of ['fixed64','handle','json','value','value_json']) {
        assert.deepEqual(actual.resources[channel],output.resources[channel],name+'/'+channel);
      }
      assert.deepEqual(actual.resources.value_det,expectedState.state.snapshot_v2.resources.value_det,
        name+'/exact typed resource carriers');
      assert.equal(actual.frames.length,1,name+'/exact requested continuation');
      assert.equal(actual.frames[0].input_receipt.input_snapshot_sha256,expectedInput,name+'/full input');
      assert.equal(actual.durable_receipt,null,name+'/VM parity is not a durable receipt');
      assert.equal(actual.replay_history,'NOT_PRODUCED',name+'/do not invent a replay history');
      const badCommand=[...command];badCommand[4]='09';
      const rejected=spawnSync(executable,badCommand,{encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024});
      assert.ifError(rejected.error);assert.notEqual(rejected.status,0,name+'/foreign parent');
      assert.match(rejected.stdout+rejected.stderr,/E_PROJECT_RUNTIME_|E_REVISION_/);
      assert.equal(sha(fs.readFileSync(artifactPath)),artifactHash,name+'/failed resume must not write artifact');
      assert.equal(expectedVm.get_state_hash(),actual.state_hash,name+'/comparison must not mutate WASM');
      fs.writeFileSync(path.join(evidenceDirectory,name+'-resume.json'),run.stdout);
      cliParity.push({name,executable,executable_sha256:sha(fs.readFileSync(executable)),command,
        status:'PASS_SOURCE_BOUND_RESUME',next_tick:actual.next_tick_id,state_hash:actual.state_hash,
        input_snapshot_sha256:actual.frames[0].input_receipt.input_snapshot_sha256,
        foreign_parent_rejected:true,artifact_sha256:artifactHash,durable_storage_commit:'NOT_RUN'});
    }
  } finally { expectedVm.free(); }
}
const deepRoundtrip=[];
if(process.env.DDN_SAVED_DEEP_TREE==='1') {
  // User source creates the tree. No test-only lowering or manufactured state.
  // Fresh local definitions preserve each inferred type; rebinding a numeric
  // name to a list would be a different, correctly rejected language action.
  const deepDefinitions=['가지0 := 1.'];
  for(let depth=1;depth<=512;depth++)deepDefinitions.push(`가지${depth} := 차림[가지${depth-1}].`);
  const deepBody=`실행:움직씨 := { ${deepDefinitions.join(' ')} 나무 <- 가지512. }.\n`;
  const deepAuthored=wasm.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',
    source_snapshot:{root_path:'root.ddn',entry_path:'app/main.ddn',files:[
      {path:'root.ddn',text:root},{path:'app/main.ddn',text:deepBody}]}}));
  const deepVm=wasm.DdnWasmVm.from_authored_project_json(deepAuthored);
  deepVm.step_one();
  const deepState=JSON.parse(deepVm.get_state_json());
  let leaf=deepState.state.snapshot_v2.resources.value_det['나무'];
  for(let depth=0;depth<512;depth++) {assert.equal(leaf.type,'list');assert.equal(leaf.items.length,1);leaf=leaf.items[0];}
  assert.equal(leaf.type,'fixed64','512 nested lists must retain the numeric leaf');
  const deepSaved=deepVm.export_saved_project_artifact_json('','06');
  const deepFile=path.join(evidenceDirectory,'deep-source-saved-artifact.json');
  fs.writeFileSync(deepFile,deepSaved);
  const cold=wasm.DdnWasmVm.from_saved_project_artifact_json(deepSaved,'','06');
  assert.equal(cold.get_state_hash(),deepVm.get_state_hash(),'deep world cold restore');
  assert.deepEqual(JSON.parse(cold.get_state_json()).state.snapshot_v2,deepState.state.snapshot_v2);
  const bad=JSON.parse(cold.get_state_json());
  bad.state.snapshot_v2.resources.value_det.z={type:'bool',value:1};
  const before=cold.get_state_json();
  assert.throws(()=>cold.restore_state(JSON.stringify(bad)));
  assert.equal(cold.get_state_json(),before,'deep first row followed by bad row must not partially publish');
  deepVm.step_one();cold.step_one();
  assert.equal(cold.get_state_hash(),deepVm.get_state_hash(),'deep continuation');
  deepRoundtrip.push({surface:'WASM',source_sha256:sha(Buffer.from(deepBody)),depth:512,
    artifact_sha256:sha(Buffer.from(deepSaved)),state_hash:cold.get_state_hash(),status:'PASS'});
  if(process.env.DDN_CLI_DIR) for(const name of ['ddonilang','ddonirang','또니랑']) {
    const executable=path.join(process.env.DDN_CLI_DIR,name+(process.platform==='win32'?'.exe':''));
    const command=['작품자료','이어실행',deepFile,'--부모','처음','--판','06','--마디수','1'];
    const run=spawnSync(executable,command,{encoding:'utf8',timeout:60_000,maxBuffer:16*1024*1024});
    assert.ifError(run.error);assert.equal(run.status,0,name+'/deep cold resume: '+run.stderr);
    const result=JSON.parse(run.stdout);assert.equal(result.state_hash,cold.get_state_hash(),name+'/deep continuation world');
    deepRoundtrip.push({surface:name,executable_sha256:sha(fs.readFileSync(executable)),command,
      state_hash:result.state_hash,status:'PASS'});
  }
  cold.free();deepVm.free();
}
const report={status:'PASS_VM_ROUNDTRIP',rejection_cases:cases.length,evidence_directory:evidenceDirectory,
  deep_saved_tree:deepRoundtrip.length?deepRoundtrip:'NOT_RUN',
  rd686_integrated_attachment:{outer_keys:topKeys,attachment_keys:attachmentKeys,revision_envelope_sha256:sha(envelopeBytes),resource_snapshot_sha256:attachment.resource_snapshot_v2_sha256,raw_tensor_transport_preserved:true,atomic_rejections:attachmentRejects},
  wasm_sha256:createHash('sha256').update(binary).digest('hex'),
  successful_roundtrip:'PASS',cli_three_names:cliParity.length?cliParity:'NOT_RUN',
  browser_save_reopen:'NOT_RUN',durable_storage_commit:'NOT_RUN'};
fs.writeFileSync(path.join(evidenceDirectory,'report.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify(report));
