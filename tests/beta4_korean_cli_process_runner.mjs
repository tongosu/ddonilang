// Actual process/UTF-8 argv checks, not a parser mock or a language lowering.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const [binaries,out,sourceMetadata]=process.argv.slice(2);
assert(binaries&&out&&sourceMetadata&&!fs.existsSync(out));
fs.mkdirSync(out,{recursive:true});
const source=JSON.parse(fs.readFileSync(sourceMetadata,'utf8'));
assert.match(source.commit,/^[a-f0-9]{40}$/u);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const report={claim:'ACTUAL_DEBUG_NATIVE_CLI3_ONLY_NOT_RELEASE_WASM_CHROME_OR_RC1_PASS',
  source_commit:source.commit,runner_sha256:sha(fs.readFileSync(new URL(import.meta.url))),cases:[]};
try{
 for(const name of ['ddonilang','ddonirang','또니랑']){
  const binary=path.resolve(binaries,name+'.exe'),binarySha=sha(fs.readFileSync(binary));
  const fixture=path.join(out,name);fs.mkdirSync(fixture);
  // The name is intentionally an option spelling, not an absolute path that
  // accidentally avoids the normalization bug. --out must remain absent.
  fs.writeFileSync(path.join(fixture,'--출력'),'{"operation":"not-a-native-operation"}',{flag:'wx'});
  fs.writeFileSync(path.join(fixture,'입력.ddn'),'값:바른수 := 1. 값 보여주기.\n',{flag:'wx'});
  assert(!fs.existsSync(path.join(fixture,'--out')));
  let serial=0;
  const invoke=args=>{
   const result=spawnSync(binary,args,{cwd:fixture,encoding:'utf8',windowsHide:true,timeout:90000,maxBuffer:8*1024*1024});
   assert.ifError(result.error);const key=String(serial++).padStart(2,'0');
   fs.writeFileSync(path.join(fixture,key+'.stdout'),result.stdout??'',{flag:'wx'});
   fs.writeFileSync(path.join(fixture,key+'.stderr'),result.stderr??'',{flag:'wx'});
   return {code:result.status,stdout:result.stdout,stderr:result.stderr};
  };
  const help=invoke(['도움']);assert.equal(help.code,0,help.stderr);
  assert.deepEqual(invoke(['help']),help);
  assert.deepEqual(invoke(['작업대','--도움']),help);
  assert.deepEqual(invoke(['workbench','--help']),help);
  for(const spelling of ['정본','다듬기','작품정리','작품찾기','작품관계','작품이름','작품잠금','작품실행','한줄실행','작업대','ddonilang 조작입력기록 <파일.ddn>','ddonilang 조작입력재생 <파일.ddn>','--글판','--브라우저없이'])assert(help.stdout.includes(spelling),spelling);
  for(const privateFlag of ['--finish-live-private','--record-original-private'])assert(!help.stdout.includes(privateFlag));
  assert.deepEqual(invoke(['판본']),invoke(['version']));
  const run=invoke(['실행','입력.ddn','--글판','v1-core-v25']);assert.equal(run.code,0,run.stderr);assert.equal(run.stdout.trim(),'1');
  assert.deepEqual(invoke(['run','입력.ddn','--source-profile','v1-core-v25']),run);
  const check=invoke(['검사','입력.ddn','--글판','v1-core-v25']);assert.equal(check.code,0,check.stderr);
  assert.deepEqual(invoke(['check','입력.ddn','--source-profile','v1-core-v25']),check);
  const originalPath=invoke(['작품자료','원문전이','--출력']);
  assert.notEqual(originalPath.code,0);assert.equal(originalPath.stdout,'');
  assert.match(originalPath.stderr,/E_NATIVE_SOURCE_REQUEST_FIELDS/u);
  assert.deepEqual(invoke(['authored-project','원문전이','--출력']),originalPath);
  assert(!fs.existsSync(path.join(fixture,'--out')));
  assert.equal(sha(fs.readFileSync(binary)),binarySha);
  report.cases.push({name,binary_sha256:binarySha,actual_processes:serial,korean_help_and_english_compatibility:true,
    raw_source_run_and_check_equal:true,opaque_native_request_path_preserved:true,invalid_request_no_success_stdout:true});
 }
 report.verdict='PASS_ACTUAL_DEBUG_CLI3_KOREAN_HELP_RAW_SOURCE_AND_NATIVE_OPAQUE_PATH';
}catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
finally{fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n',{flag:'wx'});console.log(JSON.stringify({verdict:report.verdict,out,error:report.error}));}
