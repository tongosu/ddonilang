// Continuous vs split from one admitted checkpoint with real registered subjects.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';import {createHash} from 'node:crypto';
const [stage,prior,out]=process.argv.slice(2);assert(stage&&prior&&out&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const sha=b=>createHash('sha256').update(b).digest('hex');
const raw=fs.readFileSync(path.join(prior,'RESULT.json')),baseline=JSON.parse(raw);assert.equal(baseline.verdict,'PASS_TARGET_SCOPE');
const head=JSON.parse(fs.readFileSync(path.join(prior,'NATIVE_ADMITTED_SAVED_HEAD.json'),'utf8'));
const saved=path.join(prior,'saved-prefix.json'),savedBytes=fs.readFileSync(saved);assert.equal(savedBytes.toString(),head.saved_artifact_json_utf8);
const store=path.join(baseline.native_local,'Ddonilang/WorkbenchSource',baseline.project_identity);
const grant=JSON.parse(fs.readFileSync(path.join(store,'source.locator.json'),'utf8'));
const report={scope:'CLI3_CONTINUOUS_TWO_VS_COLD_ONE_PLUS_ONE_FROM_SAME_REGISTERED_NATIVE_CHECKPOINT_NOT_INITIAL_ZERO',
 prior_sha256:sha(raw),saved_sha256:sha(savedBytes),project_identity:baseline.project_identity,checks:[],verdict:'NOT_RUN'};
report.target=JSON.parse(fs.readFileSync(path.join(stage,'BETA3_DEVELOPMENT_MANIFEST.json')));
assert.equal(report.target.fixed_commit,baseline.target.fixed_commit);
report.native_binary_sha256=Object.fromEntries(['ddonilang','ddonirang','또니랑'].map(alias=>[alias,sha(fs.readFileSync(path.join(stage,'bin',alias+'.exe')))]));
function run(alias,label,args){const r=spawnSync(path.join(stage,'bin',alias+'.exe'),['작품자료',...args],{cwd:stage,windowsHide:true,
 env:{...process.env,LOCALAPPDATA:baseline.native_local},encoding:'utf8',timeout:180000,maxBuffer:32*1024*1024});
 fs.writeFileSync(path.join(out,label+'.stdout'),r.stdout??'');fs.writeFileSync(path.join(out,label+'.stderr'),r.stderr??'');
 assert.ifError(r.error);assert.equal(r.status,0,r.stderr);return JSON.parse(r.stdout);}
const resume=(file,parent,revision,count)=>['원문이어실행',file,'--부모',parent,'--판',revision,'--저장소',store,'--허가',grant.grant_locator,'--마디수',String(count)];
try{
 const aliases=['ddonilang','ddonirang','또니랑'];const uninterrupted=[];
 for(const alias of aliases)uninterrupted.push(run(alias,alias+'-continuous2',resume(saved,head.parent_hex,head.revision_hex,2)));
 for(const result of uninterrupted)assert.deepEqual(result,uninterrupted[0]);
 // Append one real transaction to this owned test store; do not copy custody,
 // forge a capability, modify the old artifact, or reuse a revision.
 const next=sha(Buffer.from('WO23 subject split '+path.resolve(out))).slice(0,32),nextFile=path.join(out,'saved-next.json');
 const first=run(aliases[0],'split-save1',[...resume(saved,head.parent_hex,head.revision_hex,1),'--저장판',next,'--저장파일',nextFile]);
 assert.equal(first.continued_state_saved,true);assert.equal(fs.readFileSync(nextFile,'utf8'),first.saved_artifact_json_utf8);
 for(const alias of aliases){const cold=run(alias,alias+'-cold-next1',resume(nextFile,head.revision_hex,next,1)),expected=uninterrupted[0];
   for(const key of ['state_hash','resources','output_display','bogae_projection'])assert.deepEqual(cold[key],expected[key],key);
   assert.deepEqual([...first.summary.frames,...cold.frames],expected.frames,'actual ordered input/committed frame sequence');
   report.checks.push({alias,state_hash:cold.state_hash,split:'same admitted checkpoint +2 = +1/save/process exit/+1',
     registered_identity_preserved:true,full_resources_output_bogae_and_ordered_frames_equal:true});
 }
 assert.deepEqual(fs.readFileSync(saved),savedBytes);report.new_revision=next;report.saved_next_sha256=sha(fs.readFileSync(nextFile));
 report.owned_store_appended_one_transaction=true;report.verdict='PASS_TARGET_SCOPE';
}catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
finally{fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));}
