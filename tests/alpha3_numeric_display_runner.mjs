// A3-P1: typed presentation is not a world mutation or a string heuristic.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

const native=process.env.DDN_NATIVE_DIR, baseline=process.env.DDN_BASELINE_NATIVE_DIR;
assert.ok(native && baseline,'actual new and fixed baseline Native3 required');
const wasmDir=process.env.DDN_WASM_DIR;
const wasm=wasmDir?await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href):null;
if(wasm)await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-display-'));
const corpusDir=path.resolve('pack/alpha3_numeric_complete_row_r1');
const corpusBytes=fs.readFileSync(path.join(corpusDir,'manifest.json'));
const corpus=JSON.parse(corpusBytes);
for(const fixture of corpus.files){
  assert.equal(createHash('sha256').update(fs.readFileSync(path.join(corpusDir,fixture.path))).digest('hex'),fixture.sha256,'fixed corpus bytes: '+fixture.path);
}
const corpusSource=c=>{
  const source=fs.readFileSync(path.join(corpusDir,c.id+'.ddn'),'utf8');
  assert.equal(source,c.source,c.id+'/predeclared source matches fixed corpus');
  return source;
};
const cases=[
  // VC05 evidence: independently calculated exact results, not JS Number or
  // freshly captured product output. These also exercise snapshot restoration.
  {id:'exact-rational-normalization',source:'a <- (1, 3) 나눔수. b <- (1, 6) 나눔수. (a + b) 보여주기. (a * b) 보여주기. (a / b) 보여주기.',ticks:1,
   // Division retains the rational family; normalized numerator/denominator
   // are 2 and 1, rather than silently changing the result to integer family.
   expected:['1/2','1/18','2/1']},
  {id:'exact-bigint-above-i128',source:'a <- ("340282366920938463463374607431768211456") 큰바른수. b <- ("1") 큰바른수. (a + b) 보여주기. (a * ("3") 큰바른수) 보여주기.',ticks:1,
   expected:['340282366920938463463374607431768211457','1020847100762815390390123822295304634368']},
  {id:'exact-family-no-float-coercion',source:'("9007199254740993") 큰바른수 보여주기. (6, 9) 나눔수 보여주기. (72) 곱수 보여주기.',ticks:1,
   expected:['9007199254740993','2/3','2^3 * 3^2']},
  {id:'exact-mixed-family-promotion',source:'큰 <- ("100") 큰바른수. 분 <- (1, 4) 나눔수. (큰 + 분) 보여주기. (큰 * 분) 보여주기. (큰 + 분 > 큰 * 분) 보여주기.',ticks:1,
   // Rational promotion retains its family even for integral results (RD-083).
   expected:['401/4','25/1','참']},
  {id:'exact-signed-rational-modulo',source:'a <- ("170141183460469231731687303715884105728") 큰바른수. b <- ("2") 큰바른수. (a + b) 보여주기. (a * b) 보여주기. 분수 <- ("6", "-9") 나눔수. 분수 보여주기. (분수 + ("5", "3") 나눔수) 보여주기. (a * b % ("7") 큰바른수) 보여주기.',ticks:1,
   expected:['170141183460469231731687303715884105730','340282366920938463463374607431768211456','-2/3','1/1','4']},
  {id:'typed-exact-initialization',source:'채비 { r:나눔수 := (6,9) 나눔수. f:곱수 := (72) 곱수. }. r 보여주기. f 보여주기.',ticks:1,
   // A3-P3 regression: the fixed predecessor rejected top-level setup names
   // before execution. There is no predecessor world hash to compare here.
   expectedBaselineRejection:'E_CHECK_UNKNOWN_NAME',expected:['2/3','2^3 * 3^2']},
  {id:'typed-and-literal',source:fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha3_numeric_display.ddn','utf8'),ticks:1,
   expected:['19.02','-19.02','19.019999999','차림[19.02, -19.02]']},
  {id:'ties-and-negative-zero',source:'(1 / 128) 보여주기. (3 / 128) 보여주기. (-3 / 128) 보여주기. (-0.0000000002) 보여주기.',ticks:1,
   expected:['0.007812','0.023438','-0.023438','0']},
  {id:'successive-madis',source:'(매마디)마다 { (20 - 9.8 * 0.1) 보여주기. "19.019999999" 보여주기. }.',ticks:3,
   expected:['19.02','19.019999999','19.02','19.019999999','19.02','19.019999999']},
];
const run=(bin,file,ticks,json=true)=>{
  const args=['실행',file,'--source-profile','v1-core-v25','--마디수',String(ticks),...(json?['--json']:[])];
  const r=spawnSync(bin,args,{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:32*1024*1024});
  assert.ifError(r.error);assert.equal(r.status,0,r.stderr);return json?JSON.parse(r.stdout):r.stdout;
};
const rows=[];
const json=value=>typeof value==='string'?JSON.parse(value):value;
// VC05 rejection evidence: zero denominator and invalid exact integer input
// must fail in both frontdoors and leave an already running VM untouched.
const rejectionRows=[];
for(const c of [
  {id:'zero-rational-denominator',source:'(1, 0) 나눔수 보여주기.'},
  {id:'invalid-bigint-text',source:'("12.5") 큰바른수 보여주기.'},
  {id:'exact-rational-zero-divisor',source:'a <- (1, 3) 나눔수. b <- (0, 1) 나눔수. (a / b) 보여주기.'},
  {id:'typed-exact-family-mismatch',source:'채비 { 값:나눔수 := (12) 곱수. }.',top_level:true},
  {id:'integer-rejects-fraction',source:'(1.5) 바른수 보여주기.'},
  {id:'invalid-grouped-bigint-text',source:'("12,34") 큰바른수 보여주기.'},
]) {
  c.source=corpusSource(c);
  const file=path.join(dir,c.id+'.ddn');fs.writeFileSync(file,c.source,'utf8');
  let expectedDiagnostic;
  for(const alias of ['ddonirang','ddonilang','또니랑']) {
    const rejected=spawnSync(path.join(native,alias+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:32*1024*1024});
    assert.ifError(rejected.error);assert.notEqual(rejected.status,0,c.id+'/must reject');
    const diagnostic=JSON.parse((rejected.stderr||rejected.stdout).trim());
    assert.ok(diagnostic.code||diagnostic.diagnostics?.[0]?.code,c.id+'/typed diagnostic');
    if(expectedDiagnostic)assert.deepEqual(diagnostic,expectedDiagnostic,c.id+'/Native3 diagnostic');
    expectedDiagnostic=diagnostic;
  }
  if(wasm) {
    let diagnostic;
    try { wasm.wasm_run_summary_with_source_profile_json(c.source,file,1,'v1-core-v25');assert.fail(c.id+'/must reject'); }
    catch(error) { diagnostic=json(error); }
    if(expectedDiagnostic.schema==='ddn.source_analysis_snapshot.candidate.v1'){
      const analysis=json(wasm.wasm_source_analysis_snapshot_json(JSON.stringify({source_text:c.source,path:file,analysis_generation:0,source_profile_identity:'v1-core-v25'})));
      assert.deepEqual(analysis,expectedDiagnostic,c.id+'/CLI Workbench WASM analysis exact');
      assert.ok(diagnostic.code,c.id+'/lower-level summary explicitly rejects');
    }else assert.deepEqual(diagnostic,expectedDiagnostic,c.id+'/CLI WASM runtime rejection');
    const vm=new wasm.DdnWasmVm('');
    try {
      vm.update_logic_with_source_profile('(매마디)마다 { 살아있음 <- 7. }.','v1-core-v25');vm.reset(false);vm.step_one();
      if(expectedDiagnostic.schema==='ddn.source_analysis_snapshot.candidate.v1'){
        const before=vm.get_state_json(),hash=vm.get_state_hash();
        assert.throws(()=>vm.update_logic_with_source_profile(c.source,'v1-core-v25'));
        assert.equal(vm.get_state_hash(),hash,c.id+'/failed compile preserves world');
        assert.deepEqual(json(vm.get_state_json()),json(before),c.id+'/failed compile preserves complete state');
      }else{
      vm.update_logic_with_source_profile(c.top_level?c.source:`(매마디)마다 { ${c.source} }.`,'v1-core-v25');
      const before=vm.get_state_json(), hash=vm.get_state_hash();
      assert.throws(()=>vm.step_one(),undefined,c.id+'/live rejection');
      assert.equal(vm.get_state_hash(),hash,c.id+'/old world preserved');
      assert.deepEqual(json(vm.get_state_json()),json(before),c.id+'/whole state preserved');
      }
    }finally{vm.free();}
  }
  fs.writeFileSync(path.join(dir,c.id+'.diagnostic.json'),JSON.stringify(expectedDiagnostic,null,2));
  assert.equal(expectedDiagnostic.code||expectedDiagnostic.diagnostics[0].code,corpus.negative.find(row=>row.id===c.id).code,c.id+'/fixed rejection code');
  rejectionRows.push({id:c.id,code:expectedDiagnostic.code||expectedDiagnostic.diagnostics[0].code,diagnostic_frontdoor:expectedDiagnostic.schema?'common-source-analysis':'common-runtime',source_sha256:createHash('sha256').update(c.source).digest('hex')});
}
for(const c of cases){
  c.source=corpusSource(c);
  assert.deepEqual(c.expected,corpus.positive.find(row=>row.id===c.id).display,c.id+'/independent expectation binding');
  const file=path.join(dir,c.id+'.ddn');fs.writeFileSync(file,c.source,'utf8');
  let canonicalSource;
  if(wasm){
    // VC05 canon evidence: no Number-based rewriting or fixture lowering.
    canonicalSource=wasm.wasm_canon_ddn_with_source_profile(c.source,'v1-core-v25');
    assert.equal(canonicalSource,fs.readFileSync(path.join(corpusDir,c.id+'.canonical.ddn'),'utf8'),c.id+'/fixed canon fixture');
    assert.equal(wasm.wasm_canon_ddn_with_source_profile(canonicalSource,'v1-core-v25'),canonicalSource,c.id+'/canon idempotence');
    fs.writeFileSync(path.join(dir,c.id+'.canonical.ddn'),canonicalSource);
  }
  let final;
  for(const alias of ['ddonirang','ddonilang','또니랑']){
    const fresh=run(path.join(native,alias+'.exe'),file,c.ticks);
    if(c.expectedBaselineRejection){
      const old=spawnSync(path.join(baseline,alias+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수',String(c.ticks),'--json'],{encoding:'utf8',windowsHide:true,timeout:60000});
      assert.ifError(old.error);assert.notEqual(old.status,0,c.id+'/actual predecessor RED');
      assert.equal(JSON.parse(old.stderr||old.stdout).diagnostics[0].code,c.expectedBaselineRejection);
    }else{
      const old=run(path.join(baseline,alias+'.exe'),file,c.ticks);
      assert.equal(fresh.state_hash,old.state_hash,c.id+'/world hash unchanged');
      assert.deepEqual(fresh.resources,old.resources,c.id+'/world resources unchanged');
      assert.deepEqual(fresh.output_log_texts,old.output_log_texts,c.id+'/canonical text unchanged');
    }
    assert.deepEqual(fresh.output_display.texts,c.expected,c.id+'/typed display');
    assert.equal(fresh.output_display.world_hash_participation,false);
    if(final)assert.deepEqual(fresh,final,c.id+'/Native3 exact');final=fresh;
    if(canonicalSource){
      const canon=spawnSync(path.join(native,alias+'.exe'),['개발','정본',file,'--source-profile','v1-core-v25'],{encoding:'utf8',windowsHide:true,timeout:60000});
      assert.ifError(canon.error);assert.equal(canon.status,0,canon.stderr);
      assert.equal(canon.stdout.trimEnd(),canonicalSource.trimEnd(),c.id+'/CLI WASM canon');
    }
    const human=run(path.join(native,alias+'.exe'),file,c.ticks,false);
    for(const expected of c.expected)assert.ok(human.includes(expected),c.id+'/human stdout '+expected);
  }
  if(wasm){
    const w=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(c.source,file,c.ticks,'v1-core-v25'));
    assert.equal(w.state_hash,final.state_hash,c.id+'/WASM hash');
    assert.deepEqual(w.resources,final.resources,c.id+'/WASM raw');
    assert.deepEqual(w.output_display,final.output_display,c.id+'/common Rust display');
    const canonicalResult=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(canonicalSource,file,c.ticks,'v1-core-v25'));
    assert.equal(canonicalResult.state_hash,w.state_hash,c.id+'/canon preserves exact world');
    assert.deepEqual(canonicalResult.resources,w.resources,c.id+'/canon preserves exact numeric resources');
    const vm=new wasm.DdnWasmVm('');
    try{
      vm.update_logic_with_source_profile(c.source,'v1-core-v25');vm.reset(false);
      for(let madi=0;madi<c.ticks;madi++)vm.step_one();
      const saved=json(vm.get_state_json());
      const savedHash=vm.get_state_hash();
      assert.deepEqual(saved.output_display.texts,c.expected,c.id+'/live VM display');
      vm.reset(false);vm.restore_state(JSON.stringify(saved));
      const restored=json(vm.get_state_json());
      assert.equal(vm.get_state_hash(),savedHash,c.id+'/restore world unchanged');
      assert.deepEqual(restored.output_display,saved.output_display,c.id+'/restore display');
      const corrupt=structuredClone(saved);corrupt.output_display.world_state_hash='foreign-world';
      assert.throws(()=>vm.restore_state(JSON.stringify(corrupt)),/E_OUTPUT_DISPLAY_SNAPSHOT/);
      assert.equal(vm.get_state_hash(),savedHash,c.id+'/rejected restore atomic');
      assert.deepEqual(json(vm.get_state_json()).output_display,saved.output_display);
    }finally{vm.free();}
  }
  rows.push({id:c.id,state_hash:final.state_hash,display:c.expected,predecessor:c.expectedBaselineRejection?{status:'REJECTED',code:c.expectedBaselineRejection}:'WORLD_AND_CANON_UNCHANGED',source_sha256:createHash('sha256').update(c.source).digest('hex'),canon_product_parity:canonicalSource?'PASS':'NOT_RUN',canonical_source_sha256:canonicalSource?createHash('sha256').update(canonicalSource).digest('hex'):null});
}
const report={status:'PASS',decision:'A3-P1 PROVISIONAL',corpus_manifest_sha256:createHash('sha256').update(corpusBytes).digest('hex'),corpus_files_verified:corpus.files.length,corpus_source_consumption:'ACTUAL_RAW_BYTES',native_aliases:3,wasm:wasm?'PASS':'NOT_RUN',rows,rejection_rows:rejectionRows};
fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify({...report,evidence_directory:dir}));
