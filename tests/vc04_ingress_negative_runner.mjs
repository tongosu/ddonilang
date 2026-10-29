// Actual raw-source frontdoors, never a language lowering. Development builds
// only: this is not a fixed release, complete-row, or durable-first-run PASS.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const [nativeDir,wasmDir,outputRoot]=process.argv.slice(2);assert(nativeDir&&wasmDir&&outputRoot);
const out=fs.mkdtempSync(path.join(outputRoot,'VC04_원문진단_'));
const sha=b=>createHash('sha256').update(b).digest('hex');
const binaryHashes=Object.fromEntries(['ddonilang','ddonirang','또니랑'].map(name=>[name,sha(fs.readFileSync(path.join(nativeDir,name+'.exe')))]));
const bytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);await wasm.default({module_or_path:bytes});
const head='병사:임자 := { 이름:글. }. ';
const cases=[
 ['직접생성',head+'검사:병사:셈씨 := { 검사 <- (이름="철수")인 병사. }.','E_VC04_PURE_WORLD_WRITE'],
 ['간접생성',head+'생성:움직씨 := { 새값 <- (이름="철수")인 병사. }. 검사:바른수:셈씨 := { 생성. 검사 <- 1. }.','E_VC04_PURE_WORLD_WRITE'],
 ['거짓분기',head+'생성:움직씨 := { 새값 <- (이름="철수")인 병사. }. 검사:바른수:셈씨 := { { 거짓 }이면 { 생성. }. 검사 <- 1. }.','E_VC04_PURE_WORLD_WRITE'],
 ['명령결과사용',head+'(표적:(병사)가리킴) 제거:움직씨 := { 값 <- (대상=표적) 폐기하기. }.','E_VC04_COMMAND_VALUE'],
 ['받는자리오류',head+'(표적:(병사)가리킴) 제거:움직씨 := { (다른=표적) 폐기하기. }.','E_VC04_RETIRE_PIN'],
 ['대상종류오류',head+'제거:움직씨 := { (대상=1) 폐기하기. }.','E_VC04_RETIRE_TARGET'],
 // Reviewed temporary-source exact R1 §6: ordinary raw execution prepares
 // registration before running. Independent invocation identity is not a
 // shared saved-world hash; compare observations, not erased identities.
 ['명목값자동등록','명찰:이름씨 := { 이름:글. }. 값 <- (이름="철수")인 명찰. 값.이름 보여주기.',null,['철수'],true],
 ['기존묶음씨','(가로:바른수) 점:묶음씨 := {}. 값 <- (가로=2) 점. (값.가로) 보여주기.',null,['2']],
];
const report={scope:'WIP_DEVELOPMENT_RAW_SOURCE_CLI3_NODE_WASM_DIAGNOSTICS_NOT_COMPLETE_VC04',binaryHashes,wasm_sha256:sha(bytes),cases:[]};
try{
 for(const [id,source,errorCode,texts,independentIdentity] of cases){
  const file=path.join(out,id+'.ddn');fs.writeFileSync(file,source,{flag:'wx'});
  let result,error;try{result=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source,file,1,'v1-core-v25'));}catch(e){error=String(e);}
  if(errorCode){assert(error?.includes(errorCode),`${id}: ${error}`);assert.equal(result,undefined);}else{assert.equal(error,undefined,error);assert.deepEqual(result.output_display.texts,texts);}
  const native=[];
  for(const name of Object.keys(binaryHashes)){
   const run=spawnSync(path.join(nativeDir,name+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:16e6});
   assert.ifError(run.error);fs.writeFileSync(path.join(out,`${id}-${name}.stdout`),run.stdout,{flag:'wx'});fs.writeFileSync(path.join(out,`${id}-${name}.stderr`),run.stderr,{flag:'wx'});
   if(errorCode){assert.notEqual(run.status,0);assert.equal(run.stdout,'');assert(run.stderr.includes(errorCode),`${id}/${name}: ${run.stderr}`);}else{
    assert.equal(run.status,0,run.stderr);const data=JSON.parse(run.stdout);assert.deepEqual(data.output_display.texts,texts);if(!independentIdentity)assert.equal(data.state_hash,result.state_hash);
   }
   native.push({name,exit:run.status});
  }
  report.cases.push({id,source_sha256:sha(Buffer.from(source)),expected_error:errorCode,wasm_error:error??null,state_hash:result?.state_hash??null,independent_temporary_identity:Boolean(independentIdentity),native});
 }
 report.verdict='PASS_DIAGNOSTICS_AND_LEGACY_PRODUCT_NOT_REGISTERED_EXECUTION';
}catch(e){report.verdict='FAIL';report.error=String(e.stack??e);process.exitCode=1;}
finally{fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({out,verdict:report.verdict,error:report.error}));}
