import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
const [bins,wasmDir,out,caseSet='base']=process.argv.slice(2);assert(bins&&wasmDir&&out);assert(['base','f7'].includes(caseSet));assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const sha=b=>createHash('sha256').update(b).digest('hex');
const wasmBytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.resolve(wasmDir,'ddonirang_tool.js')).href);await wasm.default({module_or_path:wasmBytes});
const report={claim:'BARE_ROOT_NAME_RUNTIME_CLI3_WASM_AND_UNCHANGED_TETRIS_PREFLIGHT_ONLY',wasm_sha256:sha(wasmBytes),rows:[]};
if(caseSet==='f7')report.claim='F7_RAW_ADMISSION_CLI3_WASM_PARITY_NOT_BETA2_COMPLETE';
let currentCase='startup';process.on('uncaughtExceptionMonitor',error=>fs.writeFileSync(path.join(out,'FAILURE.json'),JSON.stringify({currentCase,error:String(error.stack??error)},null,2)));
const cases=[
 ['bare-batang','바탕 <- 3. 그림들 <- 바탕. 그림들 보여주기.',null],
 ['bare-salim','살림 <- 3. 값 <- 살림. 값 보여주기.',null],
 ['removed-batang','바탕.x <- 1.','E_BATANG_PREFIX_REMOVED'],
 ['removed-salim','살림.x <- 1.','E_SALIM_REMOVED'],
];
if(caseSet==='f7')cases.push(
 ['f7-local-bare-collection',fs.readFileSync('pack/v1_core_supported_ingress_w1/fixtures/language_core_collection.ddn','utf8'),null,'2'],
 ['f7-world-bare-collection','채비 { 값들:차림 := 차림[1,2,3]. }. 둘째:바른수 := 값들[1]. 둘째 보여주기.',null,'2'],
 ['f7-result-place-read',fs.readFileSync('tool/tests/fixtures/alpha_callable_read_forbidden.ddn','utf8'),'E_RESULTPLACE_READ_FORBIDDEN'],
 ['f7-closed-bounds',fs.readFileSync('pack/v1_core_supported_ingress_w1/fixtures/strict_selection_out_of_range.ddn','utf8'),'E_SELECTION_BOUNDS'],
 ['f7-mixed-number-compare','몫:나눔수 := 1 / 2. 근사:셈수 := 0.5. (몫 == 근사) 보여주기.','E_NUMERIC_COMPARISON_TYPE_MISMATCH']);
for(const [id,source,code,lastText] of cases){
 currentCase=id;const file=path.join(out,id+'.ddn');fs.writeFileSync(file,source);let state,error;
 try{state=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source,file,1,'v1-core-v25'));}catch(e){error=String(e);}
 fs.writeFileSync(path.join(out,id+'-wasm.json'),JSON.stringify(state??{error},null,2));
 const removedPrefix=id.startsWith('removed-');const profileCode=removedPrefix?'E_CHECK_UNKNOWN_NAME':code;
 if(removedPrefix){
  // The legacy preprocess API and explicit V25 profile execution have
  // different existing diagnostics. Check each actual contract, never
  // require the profile's static unknown-name error to impersonate legacy.
  assert.throws(()=>wasm.wasm_preprocess_source(source),e=>String(e).includes(code));
  assert(error?.includes('E_CHECK_UNKNOWN_NAME'),error);assert.equal(state,undefined);
 }else if(code){assert(error?.includes(code),error);assert.equal(state,undefined);}
 else{assert.equal(error,undefined);if(lastText){assert.equal(state.output_display.texts.at(-1),lastText);}else assert.deepEqual(state.output_display.texts,['3']);}
 const rows=[];
 for(const alias of ['ddonilang','ddonirang','또니랑']){
  const binary=path.join(bins,alias+'.exe');const run=spawnSync(binary,['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],{encoding:'utf8',timeout:120000});
  fs.writeFileSync(path.join(out,id+'-'+alias+'.stdout'),run.stdout??'');fs.writeFileSync(path.join(out,id+'-'+alias+'.stderr'),run.stderr??'');assert.equal(run.error,undefined);
  if(code){assert.notEqual(run.status,0);assert.equal(run.stdout,'');assert(run.stderr.includes(profileCode),run.stderr);}else{assert.equal(run.status,0,run.stderr);const actual=JSON.parse(run.stdout);assert.deepEqual(actual.output_display,state.output_display);assert.equal(actual.state_hash,state.state_hash);
   if(id==='f7-world-bare-collection')assert.deepEqual(actual.engine_state.state.snapshot_v2.resources,state.engine_state.state.snapshot_v2.resources,'same complete typed resource snapshot');}
  rows.push({alias,binary_sha256:sha(fs.readFileSync(binary)),exit:run.status});
 }
 report.rows.push({id,raw_sha256:sha(Buffer.from(source)),code,profile_code:profileCode,state_hash:state?.state_hash??null,native:rows});
}
currentCase='unchanged-tetris-preflight';const file=path.resolve('solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris/authored-project.json');
const artifact=fs.readFileSync(file);const source=JSON.parse(artifact).source_snapshot.files.find(f=>f.path==='main.ddn').text;
assert(source.includes('그림들 <- 바탕.'));
// An authored V25 project is not a legacy one-file preprocess request. Use
// the actual source-profile-bound project admission, without stepping an
// input-dependent world under a fabricated empty logical input profile.
const vm=wasm.DdnWasmVm.from_authored_project_json(artifact.toString('utf8'));vm.free();
report.tetris={artifact_sha256:sha(artifact),raw_main_sha256:sha(Buffer.from(source)),shared_profiled_project_admission:true,source_rewritten:false,actual_chrome:'SEPARATE_GATE',logical_input_execution:'SEPARATE_GATE'};
report.verdict='PASS';fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2));console.log(`PASS: ${cases.length} raw cases × CLI3/WASM; unchanged Tetris admission preflight`);
