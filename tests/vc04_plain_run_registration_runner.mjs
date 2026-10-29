// Raw ordinary-CLI ingress gate. No authored root, saved ID, source wrapper,
// registry injection, DDN lowering, or inferred temporary producer policy.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const [binDir,outRoot]=process.argv.slice(2);
assert(binDir&&outRoot,'node tests/vc04_plain_run_registration_runner.mjs <bin-dir> <output-root>');
const out=fs.mkdtempSync(path.join(outRoot,'VC04_일반실행등록_'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const cases=[
 {id:'기존글출력',source:'"철수" 보여주기.',output:'철수',required:true},
 {id:'이름씨최상위',source:'명찰:이름씨 := { 이름:글. }. 표 <- (이름="철수")인 명찰. 표.이름 보여주기.',output:'철수',required:true},
 {id:'원자최상위',source:'이름표 <- #철수. (이름표 == #철수) 보여주기.',output:'참',required:true},
 {id:'임자폐기재생성',source:'병사:임자 := { 이름:글. }. 표적 <- (이름="철수")인 병사. 첫 <- 표적 가리키기. (첫 == (표적 가리키기)) 보여주기. (대상=첫) 폐기하기. 다음 <- (이름="영희")인 병사. (표적 == 다음) 보여주기.',output:'참\n거짓',required:true},
 {id:'셈씨직접생성금지',source:'병사:임자 := { 이름:글. }. 검사:병사:셈씨 := { 검사 <- (이름="철수")인 병사. }.',error:'E_VC04_PURE_WORLD_WRITE'},
 {id:'셈씨간접생성금지',source:'병사:임자 := { 이름:글. }. 생성:움직씨 := { 새값 <- (이름="철수")인 병사. }. 검사:바른수:셈씨 := { 생성. 검사 <- 1. }.',error:'E_VC04_PURE_WORLD_WRITE'},
];
const report={scope:'RAW_ORDINARY_CLI_SOURCE_OUTPUT_GATE_ONLY_NOT_IDENTITY_CUSTODY_OR_COMPLETE_ROW',
 producer_contract:'NOT_SUPPLIED_BY_TEST',wasm_build:'NOT_RUN_NATIVE_ONLY_CONTACT_PREPARATION',checks:[],missing:[]};
try{
 for(const row of cases){
  const file=path.join(out,row.id+'.ddn');fs.writeFileSync(file,row.source,{flag:'wx'});
  for(const name of ['ddonilang','ddonirang','또니랑']){
   const binary=path.join(binDir,name+'.exe');const args=['run',file,'--source-profile','v1-core-v25'];
   const run=spawnSync(binary,args,{encoding:'utf8',windowsHide:true,timeout:30000,maxBuffer:4e6});assert.ifError(run.error);
   fs.writeFileSync(path.join(out,row.id+'-'+name+'.stdout'),run.stdout??'',{flag:'wx'});
   fs.writeFileSync(path.join(out,row.id+'-'+name+'.stderr'),run.stderr??'',{flag:'wx'});
   const accepted=run.status===0;
   if(row.required&&accepted){assert.equal(run.stdout.trim(),row.output);}
   else if(row.required){report.missing.push({id:row.id,name,exit:run.status,diagnostic:run.stderr});}
   else{assert.notEqual(run.status,0);assert.equal(run.stdout,'');assert(run.stderr.includes(row.error),run.stderr);}
   report.checks.push({id:row.id,name,args,binary_sha256:sha(fs.readFileSync(binary)),source_sha256:sha(Buffer.from(row.source)),exit:run.status,accepted,expected_output:row.output??null,expected_error:row.error??null});
  }
 }
 report.verdict=report.missing.length?'FAIL_REQUIRED_ORDINARY_RUN_REGISTRATION_GATE':'PASS_RAW_OUTPUT_AND_PURE_GUARDS_ONLY';
 if(report.missing.length)process.exitCode=1;
}catch(error){report.verdict='FAIL_VERIFIER_OR_OTHER_PRODUCT_BEHAVIOR';report.error=String(error.stack??error);process.exitCode=1;}
fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({out,verdict:report.verdict,missing:report.missing.length,error:report.error}));
