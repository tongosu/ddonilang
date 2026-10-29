// Reuse the same actual Native store and saved head; no identity regeneration.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';import {createHash} from 'node:crypto';
const [stage,prior,out]=process.argv.slice(2);assert(stage&&prior&&out&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const reportBytes=fs.readFileSync(path.join(prior,'RESULT.json')),priorReport=JSON.parse(reportBytes);
assert.equal(priorReport.verdict,'PASS_TARGET_SCOPE');
const sha=b=>createHash('sha256').update(b).digest('hex');
const head=JSON.parse(fs.readFileSync(path.join(prior,'NATIVE_ADMITTED_SAVED_HEAD.json'),'utf8'));
const saved=path.join(prior,'saved-prefix.json');assert.equal(fs.readFileSync(saved,'utf8'),head.saved_artifact_json_utf8);
const store=path.join(priorReport.native_local,'Ddonilang/WorkbenchSource',priorReport.project_identity);
const grantBytes=fs.readFileSync(path.join(store,'source.locator.json')),grant=JSON.parse(grantBytes);
const result={scope:'CLI3_REPEATED_COLD_SAME_NATIVE_HEAD_FULL_SUMMARY_NOT_CONTINUOUS_OR_BROWSER',
 prior_report_sha256:sha(reportBytes),saved_sha256:sha(fs.readFileSync(saved)),checks:[],verdict:'NOT_RUN'};
result.target=JSON.parse(fs.readFileSync(path.join(stage,'BETA3_DEVELOPMENT_MANIFEST.json')));
assert.equal(result.target.fixed_commit,priorReport.target.fixed_commit);
result.native_binary_sha256=Object.fromEntries(['ddonilang','ddonirang','또니랑'].map(alias=>[alias,sha(fs.readFileSync(path.join(stage,'bin',alias+'.exe')))]));
try{
 for(const alias of ['ddonilang','ddonirang','또니랑']){
  const expected=JSON.parse(fs.readFileSync(path.join(prior,alias+'.stdout'),'utf8'));
  for(let attempt=1;attempt<=2;attempt++){
   const child=spawnSync(path.join(stage,'bin',alias+'.exe'),['작품자료','원문이어실행',saved,'--부모',head.parent_hex,'--판',head.revision_hex,
     '--저장소',store,'--허가',grant.grant_locator,'--마디수','2'],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:32*1024*1024,
       env:{...process.env,LOCALAPPDATA:priorReport.native_local}});
   fs.writeFileSync(path.join(out,`${alias}-${attempt}.stdout`),child.stdout??'');fs.writeFileSync(path.join(out,`${alias}-${attempt}.stderr`),child.stderr??'');
   assert.ifError(child.error);assert.equal(child.status,0,child.stderr);assert.deepEqual(JSON.parse(child.stdout),expected);
   result.checks.push({alias,attempt,state_hash:expected.state_hash,full_summary_equal:true,new_os_process:true});
  }
 }
 assert.equal(sha(fs.readFileSync(saved)),result.saved_sha256);assert.deepEqual(fs.readFileSync(path.join(store,'source.locator.json')),grantBytes);
 result.verdict='PASS_TARGET_SCOPE';
}catch(error){result.verdict='FAIL';result.error=String(error.stack??error);process.exitCode=1;}
finally{fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));}
