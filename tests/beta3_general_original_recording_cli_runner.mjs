// Real CLI processes and the shipped WASM frontdoor. No JS replay encoder,
// world/custody copying, source lowering or fallback from a failed store.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';import {createHash} from 'node:crypto';import {pathToFileURL} from 'node:url';
const [stage,out]=process.argv.slice(2);assert(stage&&out&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const numericOnly=process.argv[4]==='--numeric-only';
const nominalWorld=process.argv[4]==='--nominal-world';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const wasmDir=path.join(stage,'bin/workbench/ui/wasm');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
if(process.argv[4]==='--cross-alias-existing'){
  await compareExistingOriginalStores(process.argv[5]);process.exit(process.exitCode??0);
}
const source=nominalWorld
  ? '명찰:이름씨 := { 이름:글. }. 병사:임자 := { 이름:글. }. 채비 { 이름표:(나눔수,명찰) := (1 / 3,(이름="처음")인 명찰). 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름="철수")인 병사. 표지 <- #표지. 이름표 보여주기. }. }.\n'
  : '채비 { 횟수:바른수 := 0. 분수:나눔수 := 1 / 3. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 횟수 보여주기. 분수 보여주기. }. }.\n';
const files=[{path:'root.ddn',text:'(글판="v25", 모듬=차림[(이름=앱, 머리="main.ddn", 원문="main.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음.\n'},{path:'main.ddn',text:source}];
const artifact=wasm.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:{root_path:'root.ddn',entry_path:'main.ddn',files}}));
const artifactFile=path.join(out,'authored-project.json');fs.writeFileSync(artifactFile,artifact);fs.writeFileSync(path.join(out,'main.ddn'),source);
const report={claim:numericOnly?'GENERAL_ORIGINAL_RECORDING_CLI3_NUMERIC_WASM_PARITY_NOT_BROWSER_TYPED_RESULT_OR_NORMAL_END':
  'GENERAL_ORIGINAL_RECORDING_CLI3_WASM_TYPED_RESULT_NOT_BROWSER_OR_NORMAL_END',mode:numericOnly?'EXPLICIT_NUMERIC_ONLY':'TYPED_ORIGINAL_REPLAY_REQUIRED',
  fixture:nominalWorld?'NOMINAL_FRACTION_TUPLE_SUBJECT_AND_ATOM_WORLD_NOT_ATOM_OUTPUT':'NUMERIC_FRACTION_OUTPUT',
  source_sha256:sha(source),artifact_sha256:sha(artifact),checks:[],verdict:'NOT_RUN'};
const run=(alias,label,args,allowFailure=false)=>{
  const env={...process.env};for(const name of ['RUST_MIN_STACK','DDN_B2_CRASH_STAGE','DDN_B2_CRASH_CAPTURE','DDN_B2_CRASH_MARKER'])delete env[name];
  const result=spawnSync(path.join(stage,'bin',alias+'.exe'),['작품자료',...args],{cwd:stage,env,windowsHide:true,encoding:'utf8',timeout:240000,maxBuffer:32*1024*1024});
  fs.writeFileSync(path.join(out,label+'.stdout'),result.stdout??'');fs.writeFileSync(path.join(out,label+'.stderr'),result.stderr??'');
  assert.ifError(result.error);if(allowFailure)return result;
  assert.equal(result.status,0,result.stderr);return JSON.parse(result.stdout);
};
try{
  const linked=JSON.parse(wasm.wasm_project_prepare_link_json(JSON.stringify({schema:'ddn.virtual_project_source.v1',root_path:'root.ddn',entry_path:'main.ddn',
    files:files.map(file=>({path:file.path,utf8_text:file.text,role:file.path==='root.ddn'?'root':'ddn_source'}))}),'실행',true,false));
  // The exact authored project supplies its explicit Start. The plain source
  // frontdoor would only define 실행 and leave the counter at its initial 0.
  // A fresh summary frontdoor is not a Subject creation grant. The nominal
  // fixture must use the actual original Native SEED/bundle consumer below.
  const expected=nominalWorld?null:JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(linked.linked_execution_source,linked.target_path,7,'v1-core-v25',artifact));
  if(expected)fs.writeFileSync(path.join(out,'wasm-seven.json'),JSON.stringify(expected,null,2));
  for(const alias of ['ddonilang','ddonirang','또니랑']){
    const store=path.join(out,alias+'-store');fs.mkdirSync(store);
    const saved=run(alias,alias+'-source',['원문저장',artifactFile,'--판','01','--저장소',store]);
    const locatorFile=path.join(out,alias+'-source.json');fs.writeFileSync(locatorFile,JSON.stringify(saved));
    const firstFile=path.join(out,alias+'-three.json');
    const first=run(alias,alias+'-first3',['첫실행',locatorFile,'--판','02','--저장소',store,'--마디수','3','--저장파일',firstFile,'--record-original-private']);
    assert.equal(fs.readFileSync(firstFile,'utf8'),first.saved_artifact_json_utf8);
    const before=fs.readFileSync(firstFile);
    const resume=(file,parent,revision,count)=>['원문이어실행',file,'--부모',parent,'--판',revision,'--저장소',store,'--허가',saved.grant_locator,'--마디수',String(count)];
    const uninterrupted=run(alias,alias+'-cold-plus4',resume(firstFile,'01','02',4));
    const secondFile=path.join(out,alias+'-five.json');
    const second=run(alias,alias+'-save-plus2',[...resume(firstFile,'01','02',2),'--저장판','03','--저장파일',secondFile]);
    assert.equal(second.continued_state_saved,true);assert.equal(fs.readFileSync(secondFile,'utf8'),second.saved_artifact_json_utf8);
    const cold=run(alias,alias+'-cold-five-plus2',resume(secondFile,'02','03',2));
    let typedEvidence={typed_wasm_original_replay_equal:false,typed_evidence_status:'NOT_RUN_EXPLICIT_NUMERIC_ONLY'};
    if(!numericOnly){
      const evidenceFile=path.join(out,alias+'-original.bundle');
      const nativeEvidence=run(alias,alias+'-evidence',[...resume(secondFile,'02','03',0),'--recording-evidence-private',evidenceFile]);
      const originalBundle=fs.readFileSync(evidenceFile);
      const wasmEvidence=JSON.parse(wasm.wasm_compare_general_original_recording_json(originalBundle));
      assert.deepEqual(wasmEvidence,nativeEvidence,'same original bundle and actual typed result via CLI and WASM');
      assert.equal(nativeEvidence.status,'RUNNING_RESULT_COMPARED');assert.equal(nativeEvidence.accepted_count_decimal,'5');
      typedEvidence={original_bundle_sha256:sha(originalBundle),readonly_result_identity:nativeEvidence.result_identity,typed_wasm_original_replay_equal:true};
    }
    for(const key of ['state_hash','resources','output_display']){
      assert.deepEqual(cold[key],uninterrupted[key],alias+'/'+key+'/split');
      // New unsaved subject worlds have separate first-use identity. Compare
      // that world through the actual original Native SEED/bundle above, not
      // through an unrelated fresh WASM world. Numeric/default coverage is
      // unchanged. Nominal typed outputs are compared through the original
      // bundle; no unsupported fresh-run grant or reference world is invented.
      if(expected)assert.deepEqual(cold[key],expected[key],alias+'/'+key+'/wasm');
    }
    assert.deepEqual([...second.summary.frames,...cold.frames],uninterrupted.frames,alias+'/ordered-actual-receipts');
    assert.deepEqual(fs.readFileSync(firstFile),before,'old artifact bytes must be preserved');
    const rejected=run(alias,alias+'-stale-old-head',resume(firstFile,'01','02',1),true);
    assert.notEqual(rejected.status,0);assert.equal(rejected.stdout.trim(),'','rejected stale history must not publish success');
    report.checks.push({alias,three_then_four_equals_three_then_two_save_cold_two:true,state_hash:cold.state_hash,
      saved_three_sha256:sha(before),saved_five_sha256:sha(fs.readFileSync(secondFile)),stale_head_rejected:true,
      fresh_unseeded_world_identity_comparison:nominalWorld?'NOT_APPLICABLE_DISTINCT_FIRST_USE_USE_SAME_ORIGINAL_BUNDLE':'PASS',
      ...typedEvidence});
  }
  report.verdict='PASS_TARGET_SCOPE';
}catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
finally{fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));}

async function compareExistingOriginalStores(previous){
  assert(previous,'an exact previous PASS report/store is required');
  const previousReportBytes=fs.readFileSync(path.join(previous,'RESULT.json'));
  const base=JSON.parse(previousReportBytes);assert.equal(base.verdict,'PASS_TARGET_SCOPE');
  assert.equal(sha(fs.readFileSync(path.join(previous,'main.ddn'))),base.source_sha256);
  assert.equal(sha(fs.readFileSync(path.join(previous,'authored-project.json'))),base.artifact_sha256);
  const names=['ddonilang','ddonirang','또니랑'];
  assert.deepEqual(base.checks.map(row=>row.alias),names);
  const result={claim:'SAME_EXISTING_NATIVE_ROOT_CLI3_AND_ORIGINAL_WASM_NOT_FRESH_IDENTITY_OR_NORMAL_END',
    target:JSON.parse(fs.readFileSync(path.join(stage,'BETA3_DEVELOPMENT_MANIFEST.json'),'utf8')),
    previous_report_sha256:sha(previousReportBytes),source_sha256:base.source_sha256,
    wasm_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))),checks:[],verdict:'NOT_RUN'};
  try{
    for(const owner of base.checks){
      const savedFile=path.join(previous,owner.alias+'-five.json'),savedBytes=fs.readFileSync(savedFile);
      assert.equal(sha(savedBytes),owner.saved_five_sha256);
      const binding=JSON.parse(fs.readFileSync(path.join(previous,owner.alias+'-source.json'),'utf8'));
      const expected=JSON.parse(fs.readFileSync(path.join(previous,owner.alias+'-cold-five-plus2.stdout'),'utf8'));
      assert.equal(expected.state_hash,owner.state_hash);
      const original=fs.readFileSync(path.join(previous,owner.alias+'-original.bundle'));
      assert.equal(sha(original),owner.original_bundle_sha256);
      const originalWasm=JSON.parse(wasm.wasm_compare_general_original_recording_json(original));
      assert.equal(originalWasm.result_identity,owner.readonly_result_identity);
      for(const consumer of names.filter(name=>name!==owner.alias)){
        const prefix=owner.alias+'-by-'+consumer;
        const invoke=(label,extra)=>{
          const env={...process.env};for(const name of ['RUST_MIN_STACK','DDN_B2_CRASH_STAGE','DDN_B2_CRASH_CAPTURE','DDN_B2_CRASH_MARKER'])delete env[name];
          const run=spawnSync(path.join(stage,'bin',consumer+'.exe'),['작품자료','원문이어실행',savedFile,
            '--부모','02','--판','03','--저장소',path.join(previous,owner.alias+'-store'),'--허가',binding.grant_locator,...extra],
            {cwd:stage,env,windowsHide:true,encoding:'utf8',timeout:240000,maxBuffer:32*1024*1024});
          fs.writeFileSync(path.join(out,prefix+'-'+label+'.stdout'),run.stdout??'');
          fs.writeFileSync(path.join(out,prefix+'-'+label+'.stderr'),run.stderr??'');
          assert.ifError(run.error);assert.equal(run.status,0,run.stderr);return JSON.parse(run.stdout);
        };
        const continued=invoke('continue',['--마디수','2']);
        for(const key of ['state_hash','resources','output_display','frames'])assert.deepEqual(continued[key],expected[key],prefix+'/'+key);
        const evidenceFile=path.join(out,prefix+'.bundle');
        const actual=invoke('original',['--마디수','0','--recording-evidence-private',evidenceFile]);
        assert.deepEqual(actual,originalWasm);assert.deepEqual(fs.readFileSync(evidenceFile),original);
        assert.deepEqual(fs.readFileSync(savedFile),savedBytes);
        result.checks.push({saved_by:owner.alias,opened_by:consumer,same_original_world_and_outputs:true,
          same_protected_bundle_and_typed_wasm_result:true,state_hash:continued.state_hash,result_identity:actual.result_identity});
      }
    }
    assert.deepEqual(fs.readFileSync(path.join(previous,'RESULT.json')),previousReportBytes);
    result.verdict='PASS_SAME_ORIGINAL_ROOT_CROSS_CLI3_WASM';
  }catch(error){result.verdict='FAIL';result.error=String(error.stack??error);process.exitCode=1;}
  finally{fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));}
}
