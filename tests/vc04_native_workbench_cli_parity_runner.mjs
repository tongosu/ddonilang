// Actual Browser saved roots -> actual CLI3 cold consumers. Cache JSON is
// only input data; every success still requires the protected Native owner.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const [browserReport,nativeDir,outRoot]=process.argv.slice(2);
const sha=b=>createHash('sha256').update(b).digest('hex');
const browser=JSON.parse(fs.readFileSync(browserReport,'utf8'));
assert.equal(browser.verdict,'PASS_NATIVE_WASM_WARM_SAVE_PROCESS_EXIT_COLD_RESUME_AND_RESAVE');
const out=fs.mkdtempSync(path.join(outRoot,'VC04_Chrome저장_CLI3_'));
const report={scope:'ACTUAL_BROWSER_ROOT_CLI3_COLD_PARITY_NOT_HUMAN_OR_COMPLETE_ROW',browser_report_sha256:sha(fs.readFileSync(browserReport)),checks:[]};
let sequence=0;
function call(name,args){
 const run=spawnSync(path.join(nativeDir,name+'.exe'),['작품자료','원문이어실행',...args],{encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:32e6});
 fs.writeFileSync(path.join(out,`${++sequence}_${name}.stdout`),run.stdout??'');
 fs.writeFileSync(path.join(out,`${sequence}_${name}.stderr`),run.stderr??'');
 assert.ifError(run.error);return run;
}
try{
 for(const [index,testCase] of browser.checks.entries()){
  assert.match(testCase.projectId,/^[0-9a-f]{64}$/);
  const root=path.join(process.env.LOCALAPPDATA,'Ddonilang','WorkbenchSource',testCase.projectId);
  const locator=JSON.parse(fs.readFileSync(path.join(root,'source.locator.json'),'utf8'));
  const caches=fs.readdirSync(root).filter(name=>/^world-[0-9a-f]{64}\.json$/.test(name));assert(caches.length&&caches.length<=16);
  const candidates=[];
  for(const [n,name] of caches.entries()){
   const cache=JSON.parse(fs.readFileSync(path.join(root,name),'utf8'));
   const file=path.join(out,`${index}_${n}_actual-artifact.json`);fs.writeFileSync(file,cache.saved_artifact_json_utf8,{flag:'wx'});
   const args=[file,'--부모',cache.parent_hex,'--판',cache.revision_hex,'--저장소',root,'--허가',locator.grant_locator,'--마디수','1'];
   const run=call('ddonilang',args);
   if(run.status===0)candidates.push({cache,file,args,summary:JSON.parse(run.stdout)});
   else assert(run.stderr.includes('E_SOURCE_WORLD_COLD_ADMISSION'),run.stderr);
  }
  assert.equal(candidates.length,1,'only the protected current Witness may admit a cache');
  const selected=candidates[0],frame=testCase.second_cold_frame.wasm_state;
  const processed=frame.input_receipt?.input_snapshot?.madi;assert(Number.isSafeInteger(processed));
  const next=processed+1,suffix=next-(selected.summary.next_tick_id-1);assert(Number.isSafeInteger(suffix)&&suffix>=1);
  const args=[...selected.args];args[args.length-1]=String(suffix);
  const rows=[];let baseline;
  for(const name of ['ddonilang','ddonirang','또니랑']){
   const run=call(name,args);assert.equal(run.status,0,run.stderr);const summary=JSON.parse(run.stdout);
   assert.equal(summary.state_hash,frame.state_hash);assert.equal(summary.next_tick_id,next);
   assert.deepEqual(summary.output_display?.texts,frame.output_display?.texts);
   // These are different existing transport schemas, not different worlds.
   // Compare every shared resource channel, with a closed key census. Native
   // value_det is re-admitted by the actual codec/Native path and world hash;
   // the UI has no such field. These sources have no UI component projection.
   assert.deepEqual(Object.keys(summary.resources).sort(),['fixed64','handle','json','schema','unit_wire_schema','value','value_det','value_json'].sort());
   assert.deepEqual(Object.keys(frame.resources).sort(),['fixed64','handle','json','value','value_json','component'].sort());
   const {schema,unit_wire_schema}=summary.resources;
   assert.equal(schema,'ddn.resource_snapshot.v2');assert.equal(unit_wire_schema,'ddn.unit_value_wire.v2');
   assert(summary.resources.value_det&&typeof summary.resources.value_det==='object');assert.deepEqual(frame.resources.component,{});
   for(const channel of ['fixed64','handle','json','value','value_json'])assert.deepEqual(summary.resources[channel],frame.resources[channel]);
   if(baseline)assert.deepEqual(summary,baseline);else baseline=summary;
   rows.push({name,binary_sha256:sha(fs.readFileSync(path.join(nativeDir,name+'.exe'))),state_hash:summary.state_hash,next_tick_id:summary.next_tick_id,
    native_output:summary.output_display?.texts,wasm_output:frame.output_display?.texts,
    resource_comparison:'ALL_FIVE_SHARED_CHANNELS_EXACT_AND_WORLD_HASH; DIFFERENT_TRANSPORT_METADATA_VALIDATED_NOT_BYTE_EQUAL'});
  }
  report.checks.push({projectId:testCase.projectId,raw_source_sha256:testCase.raw_source_sha256,selected_revision:selected.cache.revision_hex,suffix,rows});
 }
 report.verdict='PASS_CLI3_CURRENT_SOURCE_ROOT_MATCHES_ACTUAL_WASM_WORLD';
}catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({out,verdict:report.verdict,error:report.error}));
