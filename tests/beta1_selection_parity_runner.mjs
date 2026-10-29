// Raw authored source enters both product frontdoors; no test lowering.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
assert.ok(process.env.DDN_NATIVE_DIR && process.env.DDN_WASM_DIR);
const wasmDir=process.env.DDN_WASM_DIR, directory=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-beta1-selection-'));
let currentCase=null;
process.on('uncaughtExceptionMonitor',error=>{
  fs.writeFileSync(path.join(directory,'failure.json'),JSON.stringify({status:'FAILED_VALIDATION_NOT_CLASSIFIED',current_case:currentCase,error:String(error.stack??error)},null,2));
  console.error('SELECTION_EVIDENCE',directory);
});
const bytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:bytes});
const cases=[];
for(const [id,value] of [['family','👨‍👩‍👧‍👦'],['skin','👍🏽'],['flag','🇰🇷'],['combining','e\u0301']]) {
  cases.push({id,source:`원본 <- "앞가끝". 글값 <- 원본. 글값[1] <- "${value}". 원본 보여주기. 글값 보여주기. (글값[1]) 보여주기.`,texts:['앞가끝',`앞${value}끝`,value]});
}
cases.push({id:'nested',source:'자료 <- 차림["앞가끝"]. 이전 <- 자료. 자료[0][1] <- "나". (이전[0]) 보여주기. (자료[0]) 보여주기.',texts:['앞가끝','앞나끝']});
for(const [id,source,code] of [
  ['delete','글값 <- "앞가끝". 글값[1] <- "".','E_SELECTION_TEXT_REPLACEMENT'],
  ['insert','글값 <- "앞가끝". 글값[1] <- "두글".','E_SELECTION_TEXT_REPLACEMENT'],
  ['bounds','글값 <- "앞가끝". 글값[3] <- "나".','E_SELECTION_TEXT_BOUNDS'],
  ['merge','글값 <- "ab". 글값[1] <- "\u0301".','E_SELECTION_TEXT_REPLACEMENT'],
  ['mixed-leading','값 <- (1, 이름: "가람"). 값 보여주기.','E_PACK_MIXED_SHAPE'],
  ['mixed-trailing','값 <- (이름: "가람", 1). 값 보여주기.','E_PACK_MIXED_SHAPE'],
  ['mixed-call','(1, 이름: "가람") 보여주기.','E_PACK_MIXED_SHAPE'],
  ['mixed-pin','값 <- (먼저=1, 이름: "가람").','E_PACK_MIXED_SHAPE'],
]) cases.push({id,source,code});
const family=process.env.DDN_SELECTION_FAMILY??'text-and-constructor';
if(family==='list-runtime-range') {
  // Reuse the nine existing raw range cases with explicitly supplied ZIP
  // executables, not the historical helper's hard-coded debug directory.
  cases.length=0;
  cases.push(
    {id:'runtime',source:'자료 <- 차림[10,20,30,40]. 처음 <- 1. 끝 <- 3. 이전 <- 자료. 조각 <- 자료[처음..끝]. 자료[처음..끝] <- 차림[7,8]. 조각 보여주기. 이전 보여주기. 자료 보여주기.',texts:['차림[20, 30]','차림[10, 20, 30, 40]','차림[10, 7, 8, 40]']},
    {id:'empty',source:'자료 <- 차림[10,20,30,40]. (자료[4..4]) 보여주기. 자료[4..4] <- 자료[0..0]. 자료 보여주기.',texts:['차림[]','차림[10, 20, 30, 40]']},
    {id:'nested',source:'자료 <- ("행",차림[10,20,30]) 짝맞춤. 이전 <- 자료. 처음 <- 1. 끝 <- 3. 자료["행"][처음..끝] <- 차림[7,8]. (이전["행"]) 보여주기. (자료["행"]) 보여주기.',texts:['차림[10, 20, 30]','차림[10, 7, 8]']},
    {id:'bounds',source:'자료 <- 차림[10,20,30]. (자료[1..4]) 보여주기.',code:'E_SELECTION_RANGE_BOUNDS'},
    {id:'reversed',source:'자료 <- 차림[10,20,30]. 처음 <- 2. 끝 <- 1. (자료[처음..끝]) 보여주기.',code:'E_SELECTION_RANGE_BOUNDS'},
    {id:'short-replacement',source:'자료 <- 차림[10,20,30]. 자료[1..3] <- 차림[7].',code:'E_SELECTION_RANGE_REPLACEMENT'},
    {id:'long-replacement',source:'자료 <- 차림[10,20,30]. 자료[1..3] <- 차림[7,8,9].',code:'E_SELECTION_RANGE_REPLACEMENT'},
    {id:'insertion',source:'자료 <- 차림[10,20,30]. 자료[3..3] <- 차림[7].',code:'E_SELECTION_RANGE_REPLACEMENT'},
    {id:'tensor-static',source:'판 <- (차림=차림[10,20,30],원소씨=셈수,모양=(3,)) 텐서로. 처음 <- 0. 끝 <- 2. (판[처음..끝]) 보여주기.',code:'E_TENSOR_STATIC_RANGE'},
  );
}
if(family==='map-tensor-field') {
  cases.length=0; // Existing text evidence is unchanged; run only the missing family.
  cases.push(
    {id:'map-nested-alias',source:'자료 <- ("행", 차림[2,3]) 짝맞춤. 이전 <- 자료. 자료["행"][1] <- 7. (이전["행"][1]) 보여주기. (자료["행"][1]) 보여주기.',texts:['3','7']},
    {id:'map-range-alias',source:'자료 <- ("행",차림[10,20,30]) 짝맞춤. 이전 <- 자료. 처음 <- 1. 끝 <- 3. 자료["행"][처음..끝] <- 차림[7,8]. (이전["행"]) 보여주기. (자료["행"]) 보여주기.',texts:['차림[10, 20, 30]','차림[10, 7, 8]']},
    {id:'map-missing-read',source:'자료 <- ("칸",2) 짝맞춤. (자료["없음"]) 보여주기.',code:'E_SELECTION_KEY_MISSING'},
    {id:'map-missing-write',source:'자료 <- ("칸",2) 짝맞춤. 자료["없음"] <- 7.',code:'E_SELECTION_KEY_MISSING'},
    {id:'map-typed-replacement',source:'자료:(글,수)짝맞춤 := ("칸",2) 짝맞춤. 자료["칸"] <- "다름".',reject:true},
    {id:'map-multiaxis',source:'자료 <- ("행",차림[2,3]) 짝맞춤. (자료["행",1]) 보여주기.',reject:true},
    {id:'list-multiaxis',source:'자료 <- 차림[차림[2,3]]. (자료[0,1]) 보여주기.',code:'E_TENSOR_SELECTION_TARGET'},
    {id:'record-named-field',source:'자료 <- (가로:3,세로:4). 자료.가로 보여주기. 자료.세로 보여주기.',texts:['3','4']},
    {id:'record-missing-field',source:'자료 <- (가로:3,세로:4). 자료.깊이 보여주기.',code:'E_FIELD_MISSING'},
    {id:'positional-not-named-field',source:'자료 <- (3,4). 자료.가로 보여주기.',reject:true},
    {id:'tensor-multiaxis-chained-persistent',source:'판 <- (차림=차림[차림[1,2,3],차림[4,5,6]],원소씨=셈수,모양=(2,3)) 텐서로. 이전 <- 판. (판[1,2]) 보여주기. (판[1][2]) 보여주기. 새열 <- (차림=차림[차림[8],차림[9]],원소씨=셈수,모양=(2,1)) 텐서로. 판[:,1..2] <- 새열. 판[1,2] <- 7. 판[0..1,:][0][0] <- 10. (판[0,0]) 보여주기. (판[0,1]) 보여주기. (판[1,1]) 보여주기. (판[1,2]) 보여주기. (이전[0,0]) 보여주기. (이전[1,2]) 보여주기.',texts:['6','6','10','8','9','7','1','6']},
    {id:'tensor-dynamic-range',source:'판 <- (차림=차림[10,20,30],원소씨=셈수,모양=(3,)) 텐서로. 처음 <- 0. 끝 <- 2. (판[처음..끝]) 보여주기.',code:'E_TENSOR_STATIC_RANGE'},
    {id:'tensor-shape-replacement',source:'판 <- (차림=차림[10,20,30],원소씨=셈수,모양=(3,)) 텐서로. 판[:] <- (차림=차림[7],원소씨=셈수,모양=(1,)) 텐서로.',reject:true},
    {id:'tensor-too-many-axes',source:'판 <- (차림=차림[10,20,30],원소씨=셈수,모양=(3,)) 텐서로. (판[0,0]) 보여주기.',reject:true},
    {id:'tensor-scalar-slice-category',source:'판 <- (차림=차림[차림[1,2],차림[3,4]],원소씨=셈수,모양=(2,2)) 텐서로. 행 <- 판[1]. 빈판 <- 판[2..2,:]. 기대행 <- (차림=차림[3,4],원소씨=셈수,모양=(2,)) 텐서로. 기대빈판 <- (차림=() 차림,원소씨=셈수,모양=(0,2)) 텐서로. (행 == 기대행) 보여주기. (빈판 == 기대빈판) 보여주기. ((판[1,1]) == 4) 보여주기.',texts:['참','참','참']},
    {id:'constructor-empty-without-element-type',source:'자료 <- 차림[].',code:'E_COLLECTION_CONSTRUCTOR_ELEMENT_TYPE_REQUIRED'},
  );
}
if(family==='vc08-owner-boundary'){
  cases.length=0;
  const commit='60d47bef1c5c0c1603ee3d92b1a8b0d34dea6aee',prefix='pack/v1_core_supported_ingress_w1/';
  const readGit=name=>{
    const result=spawnSync('git',['--no-optional-locks','show',`${commit}:${prefix}${name}`],{encoding:'utf8',windowsHide:true,maxBuffer:1024*1024});
    assert.equal(result.status,0,result.stderr);return result.stdout;
  };
  // Reuse the original primitive Sum contracts, including their exact errors.
  // Optional/Result are a different coverage family, not an inherited PASS.
  const required=new Set([
    'vc08_nullary.ddn','vc08_payload.ddn','vc08_exhaustive.ddn','vc08_default.ddn',
    'vc08_duplicate.ddn','vc08_unknown.ddn','vc08_match_duplicate.ddn',
    'vc08_nonexhaustive.ddn','vc08_duplicate_default.ddn','vc08_nontrailing.ddn',
    'vc08_nonsum.ddn','vc08_crossowner.ddn','vc08_payload_type.ddn',
    'vc08_bare.ddn','vc08_split.ddn',
  ]);
  for(const [file,texts,code] of JSON.parse(readGit('vc08_cases.json'))){
    if(required.has(file))cases.push({id:file.replace('.ddn',''),source:readGit('fixtures/'+file),texts:texts??undefined,code,origin_commit:commit,origin_path:prefix+'fixtures/'+file});
  }
  assert.equal(cases.length,required.size,'reuse exactly the original primitive owner/category fixtures');
}
if(family==='vc13-sum-pattern-scope') {
  cases.length=0;
  // The same raw authored fixtures enter the common Rust unit and CLI/WASM.
  cases.push(...JSON.parse(fs.readFileSync(new URL('./fixtures/vc13_sum_pattern_scope_cases.json',import.meta.url),'utf8')));
  assert.equal(cases.length,40);
}
if(family==='vc13-declared-call-types') {
  cases.length=0;
  const declarations='(원값:바른수) 정수전달:바른수:셈씨 := { 정수전달 <- 원값. }. (원값:셈수) 실수전달:셈수:셈씨 := { 실수전달 <- 원값. }. 정수:바른수 := 7. 실수:셈수 := 7. 글값:글 := "가". ';
  cases.push(
    {id:'declared-call-root',source:declarations+'묶음값 <- ((원값=정수) 정수전달,글값). 묶음값[0] 보여주기. 묶음값[1] 보여주기.',texts:['7','가']},
    {id:'declared-call-compose-select-replace',source:declarations+'묶음값 <- ((원값=정수) 정수전달,글값). 복사값 <- 묶음값[0..2]. 중첩값 <- (복사값,묶음값). 묶음값[0] <- (원값=정수) 정수전달. 복사값[0] 보여주기. 중첩값[1][1] 보여주기. 묶음값[0] 보여주기.',texts:['7','가','7']},
    {id:'declared-call-surface',source:declarations+'묶음값 <- ((원값=정수) 정수전달기,글값). 묶음값[0] 보여주기.',texts:['7']},
    {id:'declared-call-position-type-conflict',source:declarations+'묶음값 <- ((원값=정수) 정수전달,글값). 묶음값[0] <- (원값=실수) 실수전달.',reject:true},
    {id:'declared-call-result-type-conflict',source:'(원값:글) 잘못전달:바른수:셈씨 := { 잘못전달 <- 원값. }. 글값:글 := "가". 묶음값 <- ((원값=글값) 잘못전달,글값).',reject:true},
  );
}
if(family==='typed-numeric-comparison') {
  cases.length=0;
  for(const op of ['==','!=','<','<=','>','>='])for(const [a,b] of [['정확','근사'],['근사','정확'],['정수','근사']]) {
    cases.push({id:`compare-${cases.length}`,source:`채비 { 정확:나눔수 := 1.0. 근사:셈수 := 1.0. 정수:바른수 := 1. }. 9 보여주기. (${a} ${op} ${b}) 보여주기.`,code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'});
  }
  cases.push(
    {id:'same-rational',source:'왼값:나눔수 := 0.1. 오른값:나눔수 := 0.1. (왼값 == 오른값) 보여주기.',texts:['참']},
    {id:'same-approx',source:'왼값:셈수 := 1.0. 오른값:셈수 := 1.0. (왼값 == 오른값) 보여주기.',texts:['참']},
    {id:'pending-literals',source:'(1 == 1.0) 보여주기. ((1/2) == 0.5) 보여주기.',texts:['참','참']},
    {id:'typed-pins',source:'(왼값:나눔수,오른값:셈수) 비교:참거짓:셈씨 := { 비교 <- 왼값 == 오른값. }. 정확:나눔수 := 0.5. 근사:셈수 := 0.5. ((왼값=정확,오른값=근사) 비교) 보여주기.',code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'},
    {id:'typed-result',source:'(원값:나눔수) 정확값:나눔수:셈씨 := { 정확값 <- 원값. }. 정확:나눔수 := 0.5. 근사:셈수 := 0.5. (((원값=정확) 정확값) == 근사) 보여주기.',code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'},
    {id:'typed-selection',source:'정수:바른수 := 1. 근사:셈수 := 1.0. 묶음 <- (정수,근사). (묶음[0] == 묶음[1]) 보여주기.',code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'},
    {id:'numeric-constructor',source:'근사:셈수 := 0.5. (((1,2) 나눔수) == 근사) 보여주기.',code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'},
    {id:'different-exact',source:'(((2,1) 나눔수) == (("2") 큰바른수)) 보여주기.',code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'},
  );
}
if(family==='typed-numeric-negation') {
  cases.length=0;
  for(const op of ['==','!=','<','<=','>','>='])for(const [a,b] of [['(-정확)','(-근사)'],['정확','(-근사)'],['(-정확)','근사'],['(-(-정확))','근사']]) {
    cases.push({id:`negation-${cases.length}`,source:`정확:나눔수 := 0.5. 근사:셈수 := 0.5. 9 보여주기. (${a} ${op} ${b}) 보여주기.`,code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'});
  }
  cases.push(
    {id:'negative-approx',source:'왼값:셈수 := 0.5. 오른값:셈수 := 0.5. ((-왼값) == (-오른값)) 보여주기.',texts:['참']},
    {id:'negative-rational',source:'왼값:나눔수 := 0.5. 오른값:나눔수 := 0.5. ((-왼값) == (-오른값)) 보여주기.',texts:['참']},
    {id:'negative-rational-denominator-one',source:'원본:나눔수 := 1. 음수:나눔수 := -원본. 복사 := -원본. (음수 == 복사) 보여주기.',texts:['참']},
    {id:'negative-literal-compatibility',source:'((-1) == (-1.0)) 보여주기. ((-(1/2)) == (-0.5)) 보여주기.',texts:['참','참']},
    {id:'negative-definition-alias',source:'정확:나눔수 := 0.5. 근사:셈수 := 0.5. 복사 := 근사. (정확 == (-복사)) 보여주기.',code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'},
    {id:'negative-first-binding-alias',source:'정확:나눔수 := 0.5. 근사:셈수 := 0.5. 복사 <- 근사. (정확 == (-복사)) 보여주기.',code:'E_NUMERIC_COMPARISON_TYPE_MISMATCH'},
  );
}
if(family==='typed-numeric-original-direct') {
  cases.length=0;
  cases.push({id:'original-eq3-two-lines',source:'값:나눔수 := 0.1 + 0.2.\n(값 == (3/10)) 보여주기.\n',texts:['참']});
  for(const op of ['==','!=','<','<=','>','>='])for(const [a,b] of [['값','(3/10)'],['(3/10)','값'],['(-값)','(-(3/10))']]) {
    cases.push({id:`original-direct-${cases.length}`,source:`값:나눔수 := 0.1+0.2. 9 보여주기. (${a} ${op} ${b}) 보여주기.`,texts:['9',['==','<=','>='].includes(op)?'참':'거짓']});
  }
  for(const [id,body] of [
    ['quotient-alias','복사 <- 3/10. (값 == 복사) 보여주기.'],
    ['tuple-comparison','((값,참) == (3/10,참)) 보여주기.'],
    ['list-comparison','(차림[값] == 차림[3/10]) 보여주기.'],
    ['unresolved-constructor-owner','(((3,10) 나눔수) == (3/10)) 보여주기.'],
  ])cases.push({id,source:'값:나눔수 := 0.1+0.2. '+body,code:'E_NUMERIC_COMPARISON_TYPE_UNRESOLVED'});
  cases.push(
    {id:'computed-wrapper',source:'값:나눔수 := 0.1+0.2. (값 == ((3/10)+0)) 보여주기.',texts:['참']},
    {id:'decimal-tree-context',source:'값:나눔수 := 0.1+0.2. (값 == (0.1+0.2)) 보여주기.',texts:['참']},
    {id:'context-denominator-one',source:'값:나눔수 := 1. (값 == (2/2)) 보여주기.',texts:['참']},
    {id:'unknown-call-result',source:'(분자값:수,분모값:수) 몫:수:셈씨 := { 몫 <- 분자값/분모값. }. 값:나눔수 := 0.1+0.2. (값 == (분자값=3,분모값=10) 몫) 보여주기.',code:'E_NUMERIC_COMPARISON_TYPE_UNRESOLVED'},
    {id:'declared-quotient-rational',source:'값:나눔수 := 0.1+0.2. 비교값:나눔수 := 3/10. (값 == 비교값) 보여주기.',texts:['참']},
    {id:'constructed-rational',source:'값:나눔수 := 0.1+0.2. (값 == ((3,10) 나눔수)) 보여주기.',texts:['참']},
    {id:'bare-literal-unchanged',source:'값:나눔수 := 0.5. (값 == 0.5) 보여주기.',texts:['참']},
    {id:'pending-division-unchanged',source:'((3/10) == 0.3) 보여주기. (3/10) 보여주기.',texts:['참','0.3']},
  );
}
if(family==='vc13-b01-literal-defaults') {
  cases.length=0;
  cases.push(
    {id:'literal-defaults',source:'묶음값 <- (1,1.0,"글",참). 묶음값[0] 보여주기. 묶음값[1] 보여주기. 묶음값[2] 보여주기. 묶음값[3] 보여주기.',texts:['1','1','글','참']},
    {id:'literal-compose-replace',source:'묶음값 <- (1,"글",참). 복사 <- 묶음값[0..2]. 중첩 <- (복사,묶음값). 묶음값[0] <- 2. 중첩[1][0] 보여주기. 묶음값[0] 보여주기.',texts:['1','2']},
    {id:'literal-same-kind-arithmetic',source:'묶음값 <- (1+2,0.1+0.2). 묶음값[0] 보여주기. 묶음값[1] 보여주기.',texts:['3','0.3']},
    {id:'exact-decimal-context',source:'합:나눔수 := 0.1+0.2. 자리:나눔수 := 0.10000000001. (합 == (3,10) 나눔수) 보여주기. (자리 == ("10000000001","100000000000") 나눔수) 보여주기.',texts:['참','참']},
    {id:'exact-denominator-one',source:'몫:나눔수 := 4/2. (몫 == (2,1) 나눔수) 보여주기.',texts:['참']},
    {id:'exact-declared-pin-and-result',source:'(값:나눔수) 그대로:나눔수:셈씨 := { 그대로 <- 값. }. 합 <- (값=0.1+0.2) 그대로. (합 == (3,10) 나눔수) 보여주기.',texts:['참']},
    {id:'pending-mixed-compatibility',source:'실수:셈수 := 1. 실수 보여주기. 혼합 <- 1+0.2. (혼합 == 1.2) 보여주기.',texts:['1','참']},
    {id:'approx-binding-not-reliteralized',source:'근사:셈수 := 0.1. 정확:나눔수 := 근사.',reject:true},
    {id:'tuple-position-default-conflict',source:'묶음값 <- (1,"글"). 묶음값[0] <- 1.0.',reject:true},
  );
}
if(family==='beta1-freeze-contracts') {
  cases.length=0;
  cases.push(
    {id:'sum-root-multiple-pins',source:'선택:고름씨 := { $쌍(가로:바른수,세로:바른수). }. 선택.$쌍(세로=4,가로=3)에 따라 { 선택.$쌍(세로,가로)이면 { (가로+세로) 보여주기. }. }.',texts:['7']},
    {id:'sum-semssi-context',source:'선택:고름씨 := { $값(내용:바른수). }. (수치:바른수) 표본:바른수:셈씨 := { 선택.$값(내용=수치)에 따라 { 선택.$값(내용)이면 { 표본 <- 내용. }. }. }. (수치=3) 표본 보여주기.',texts:['3']},
    {id:'derived-reserved-root',source:'처음 <- 1. 결과 <<- 처음 + 1.',code:'E_PERSISTENT_DEPENDENCY_PRODUCT_OPEN'},
    {id:'derived-reserved-callable',source:'실행:움직씨 := { 처음 <- 1. 결과 <<- 처음 + 1. }.',code:'E_PERSISTENT_DEPENDENCY_PRODUCT_OPEN'},
    {id:'assignment-preserved',source:'실행:움직씨 := { 값 <- 1. 값 <- 값 + 1. 값 보여주기. }.',texts:['2']},
  );
}
const rows=[];
for(const item of cases) {
  currentCase=item;
  const file=path.join(directory,item.id+'.ddn');fs.writeFileSync(file,item.source);
  const analysis=JSON.parse(wasm.wasm_source_analysis_snapshot_json(JSON.stringify({source_text:item.source,path:file,analysis_generation:0,source_profile_identity:'v1-core-v25'})));
  let screen,error;
  try{screen=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(item.source,file,1,'v1-core-v25'));}
  catch(e){error=String(e);}
  if(item.code||item.reject) {
    assert.ok(error,`${item.id}: expected rejection`);
    if(item.code)assert.ok(error.includes(item.code),`${item.id}: ${error}`);
    assert.equal(screen,undefined,`${item.id}: rejected WASM request published a summary`);
    assert.ok(!error.includes('"state_hash"'),`${item.id}: rejected WASM request published a world hash`);
  }
  else {assert.equal(error,undefined);assert.deepEqual(screen.output_display.texts,item.texts);}
  const nativeRows=[];
  let canon=null;
  if(!error&&family!=='text-and-constructor'){
    canon=wasm.wasm_canon_ddn_with_source_profile(item.source,'v1-core-v25');
    assert.equal(wasm.wasm_canon_ddn_with_source_profile(canon,'v1-core-v25'),canon,`${item.id}: canonical roundtrip`);
  }
  for(const name of ['ddonilang','ddonirang','또니랑']) {
    const binary=path.join(process.env.DDN_NATIVE_DIR,name+'.exe');
    const options={encoding:'utf8',windowsHide:true,timeout:30000,maxBuffer:16*1024*1024};
    const run=spawnSync(binary,['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],options);
    assert.ifError(run.error);
    if(item.code||item.reject) {
      assert.notEqual(run.status,0);if(item.code)assert.ok((run.stdout+run.stderr).includes(item.code));
      assert.ok(!run.stdout.includes('"state_hash"'),`${item.id}: rejected CLI request published a success hash`);
      assert.equal(run.stdout.trim(),'','rejected CLI execution must not publish partial output');
      const diagnostic=JSON.parse(run.stderr.trim());
      // The CLI static preflight returns an analysis snapshot; the WASM run
      // request returns an execution-error envelope. Compare like contracts.
      assert.deepEqual(diagnostic,diagnostic.schema===analysis.schema?analysis:JSON.parse(error),`${item.id}: same shared diagnostic contract`);
    }
    else {
      assert.equal(run.status,0,run.stderr);const native=JSON.parse(run.stdout);assert.deepEqual(native.output_display.texts,item.texts);assert.equal(native.state_hash,screen.state_hash);
      if(canon!==null){
        const canonical=spawnSync(binary,['개발','정본',file,'--source-profile','v1-core-v25'],options);assert.ifError(canonical.error);assert.equal(canonical.status,0,canonical.stderr);
        assert.equal(canonical.stdout.trim(),canon.trim(),`${item.id}: same CLI/WASM canonical source`);
      }
    }
    nativeRows.push({name,exit:run.status,binary_sha256:createHash('sha256').update(fs.readFileSync(binary)).digest('hex'),same_complete_diagnostic:Boolean(error),same_canon:canon!==null});
  }
  rows.push({id:item.id,source:file,origin_commit:item.origin_commit??null,origin_path:item.origin_path??null,source_sha256:createHash('sha256').update(item.source).digest('hex'),expected_texts:item.texts??null,expected_rejection:Boolean(item.code||item.reject),analysis,wasm_error:error??null,canon,code:item.code ?? null,state_hash:screen?.state_hash ?? null,native:nativeRows});
}
const report={result:'PASS',family,cases:rows.length,native_aliases:3,wasm_sha256:createHash('sha256').update(bytes).digest('hex'),rows};
fs.writeFileSync(path.join(directory,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({...report,evidence:directory}));
