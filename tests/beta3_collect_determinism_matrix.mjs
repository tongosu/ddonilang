// Evidence index only: never turns a narrower result into whole-row PASS.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import {createHash} from 'node:crypto';
const [output,...entries]=process.argv.slice(2);assert(output&&entries.length&&!fs.existsSync(output));
assert(entries[0].startsWith('target='),'current collection requires the exact product manifest first');
const groups=['numeric','subject_atom','derived','logical_tetris','double_pendulum','multifile'];
const routes=['ddonilang','ddonirang','또니랑','WASM','Workbench'],methods=['continuous','cold_resume','repeat'];
const matrix={schema:'ddn.beta3.determinism_evidence_matrix.work.v1',claim:'F1_PROGRESS_ONLY_NOT_BETA3_COMPLETE_OR_ROW_PASS',
 baseline_product:'6b2e640f8333a03f5cbd76eead0654d44abb00d7',
 cells:groups.flatMap(group=>routes.flatMap(route=>methods.map(method=>({group,route,method,status:'NOT_RUN'})))),
 evidence:[],unclosed:['pacing and wall-clock independence','view toggles','same-PESI canonical result identity audit',
   'causal/durable receipt producer and replay','remaining Workbench cells','independent whole-row review'],verdict:'INCOMPLETE'};
const sha=x=>createHash('sha256').update(x).digest('hex');
function put(group,route,method,details){const c=matrix.cells.find(x=>x.group===group&&x.route===route&&x.method===method);assert(c);
 if(c.status!=='NOT_RUN'){(c.supporting_evidence??=[]).push(details);return;}
 Object.assign(c,{...details,status:'PASS_TARGET_SCOPE'});}
for(const entry of entries){
 const at=entry.indexOf('=');assert(at>0);const name=entry.slice(0,at),file=path.resolve(entry.slice(at+1));
 const bytes=fs.readFileSync(file),report=JSON.parse(bytes),evidence={name,path:file,bytes:bytes.length,sha256:sha(bytes)};
 matrix.evidence.push(evidence);
 if(name==='target'){
   assert.match(report.fixed_commit??'',/^[a-f0-9]{40}$/);matrix.baseline_product=report.fixed_commit;matrix.target=report;continue;
 }
 assert(matrix.target,'target identity must precede observed evidence');
 if(report.target)assert.equal(report.target.fixed_commit,matrix.target.fixed_commit,'no cross-target PASS inheritance');
 if(report.stage?.fixed_commit)assert.equal(report.stage.fixed_commit,matrix.target.fixed_commit,'repeat must use current product');
 if(report.wasm_sha256)assert.equal(report.wasm_sha256,matrix.target.wasm_sha256,'actual WASM identity');
 if(report.native_binary_sha256)for(const [alias,hash] of Object.entries(report.native_binary_sha256))
   assert.equal(hash,matrix.target.native.files.find(row=>row.file===alias+'.exe')?.sha256,'actual Native binary identity');
 if(groups.includes(name)){
   assert.equal(report.verdict,'PARTIAL_PASS');
   for(const c of report.cells.filter(c=>c.group===name&&c.status==='PASS'))put(c.group,c.route,c.method,{evidence:matrix.evidence.length-1,...c});
 }else if(name.endsWith('.browser_repeat')&&groups.includes(name.slice(0,-15))){
   const group=name.slice(0,-15);
   assert.equal(report.verdict,'PASS_TARGET_SCOPE');assert.equal(report.cli3.length,3);
   assert.match(report.reused_warm_report_sha256,/^[a-f0-9]{64}$/);
   assert.match(report.saved_sha256,/^[a-f0-9]{64}$/);assert.equal(report.page_errors.length,0);
   for(const route of ['WASM','Workbench'])put(group,route,'repeat',{evidence:matrix.evidence.length-1,
     saved_sha256:report.saved_sha256,scope:'reopened the same actual Native saved identity in Chrome; all published engine channels; no fresh identity or standalone JSON authority'});
 }else if(name.endsWith('.cli_repeat')&&groups.includes(name.slice(0,-11))){
   const group=name.slice(0,-11);assert.equal(report.verdict,'PASS_TARGET_SCOPE');
   assert.equal(report.checks.length,6);
   for(const alias of routes.slice(0,3)){
     const checks=report.checks.filter(c=>c.alias===alias);assert.equal(checks.length,2);
     assert(checks.every(c=>c.full_summary_equal&&c.new_os_process));
     assert.equal(checks[0].state_hash,checks[1].state_hash);
     put(group,alias,'repeat',{evidence:matrix.evidence.length-1,state_hash:checks[0].state_hash,
       scope:'two new processes resumed the same saved Native head; not a fresh initial continuous run'});
   }
 }else if(name==='subject_atom.registered_continuous_split'){
   assert.equal(report.verdict,'PASS_TARGET_SCOPE');assert.equal(report.checks.length,3);
   assert.equal(report.owned_store_appended_one_transaction,true);
   for(const alias of routes.slice(0,3)){
     const row=report.checks.find(c=>c.alias===alias);assert(row?.registered_identity_preserved&&row.full_resources_output_bogae_and_ordered_frames_equal);
     put('subject_atom',alias,'continuous',{evidence:matrix.evidence.length-1,state_hash:row.state_hash,
       saved_sha256:report.saved_sha256,scope:'same admitted registered checkpoint: uninterrupted +2 vs +1/save/process exit/+1; not zero-madi registration or bare JSON authority'});
   }
 }else if(name==='logical_tetris.browser_cold_repeat'){
   assert.equal(report.verdict,'PASS_TARGET_SCOPE');assert.equal(report.checks.length,2);
   assert(report.checks.every(c=>c.full_summary_equal&&c.saved_head_unchanged));
   assert.equal(report.checks[0].state_hash,report.checks[1].state_hash);
   assert.equal(report.page_errors.length,0);assert.equal(report.host_exits.length,2);
   assert(report.host_exits.every(e=>e.code===0));
   put('logical_tetris','Workbench','repeat',{evidence:matrix.evidence.length-1,
     scope:'two new Native hosts and Chrome profiles replayed the same saved logical input; every Rust summary field and separate view matched, head unchanged'});
 }else if(name==='logical_tetris.browser'){
   assert.equal(report.verdict,'PASS_NATIVE_LOGICAL_CHROME_SAVE_CLOSE_COLD_CONTINUE_RESAVE_MATCHES_CONTINUOUS_WASM');
   assert.equal(report.page_errors.length,0);assert.equal(report.http_errors.length,0);
   assert.equal(report.stage.commit,matrix.target.fixed_commit);
   assert(report.resaved_madi>report.saved_madi);
   for(const method of ['continuous','cold_resume'])put('logical_tetris','Workbench',method,{evidence:matrix.evidence.length-1,
     scope:'actual extracted target Native Chrome save/host shutdown/cold restore/resave; accepted inputs and full continuous WASM summary compared; not repeat or a new receipt authority'});
 }else if(name.endsWith('.browser')&&groups.includes(name.slice(0,-8))){
   const group=name.slice(0,-8);
   assert.equal(report.verdict,'PASS_TARGET_SCOPE');assert.equal(report.cli3.length,3);
   for(const c of report.cli3)put(group,c.alias,'cold_resume',{evidence:matrix.evidence.length-1,state_hash:c.state_hash,scope:'Native cold vs actual same saved identity Chrome warm'});
   for(const route of ['WASM','Workbench'])for(const method of ['continuous','cold_resume'])
     put(group,route,method,{evidence:matrix.evidence.length-1,scope:'actual Chrome Native-bound WASM; all public engine channels; not a standalone JSON admission claim'});
 }else if(name==='logical_tetris.native'){
   assert.equal(report.results.length,3);
   for(const row of report.results)assert.equal(row.exe_sha256,matrix.target.native.files.find(file=>file.file===row.exe+'.exe')?.sha256);
   for(const row of report.results){assert(row.save3_cold4_equals7&&row.v3_same_authored_record_replay&&row.resave_cold_reopen);
     for(const method of methods)put('logical_tetris',row.exe,method,{evidence:matrix.evidence.length-1,state_hash:row.state_hash,scope:'actual original C02 assertions unchanged'});}
 }else if(name==='logical_tetris.wasm'){
   assert.equal(report.verdict,'PASS_TARGET_SCOPE');assert(report.new_node_process&&report.continuous_repeat_cold_full_summary_equal);
   for(const method of methods)put('logical_tetris','WASM',method,{evidence:matrix.evidence.length-1,state_hash:report.state_hash,
     scope:'same local WASM checkpoint producer; Native summary/tape equality; not Native durable artifact admission or receipt equality'});
 }else if(name==='invariance.pacing'){
   assert.equal(report.verdict,'PASS_TARGET_SCOPE');assert.equal(report.rows.length,4);
   assert(report.rows.every(r=>r.full_frames_and_state_equal&&r.input_receipts_equal&&r.source_bytes_preserved));
   (matrix.invariance??=[]).push({kind:'wall_clock_pacing',evidence:matrix.evidence.length-1,
     scope:report.scope,fixtures:report.rows.map(r=>{assert.equal(typeof r.fixture,'string');return r.fixture;}),coverage:'four original non-logical groups only; not all platforms or new durable receipt owner'});
 }else if(name==='invariance.view'){
   assert.equal(report.verdict,'PASS_TARGET_SCOPE');assert.equal(report.errors.length,0);
   assert(report.checks.length>0&&report.checks.every(r=>r.world_and_processed_input_unchanged));
   (matrix.invariance??=[]).push({kind:'view_controls',evidence:matrix.evidence.length-1,
     operations:report.checks.map(r=>r.kind),scope:report.scope,coverage:'actual saved scene and tested controls; not all scenes'});
 }else throw Error('unrecognized evidence group '+name);
}
// Preserve every explicit gap. These cells do not establish the F2/F3 identity
// and receipt claim even when their observed world/output channels are equal.
matrix.observed_pass_cells=matrix.cells.filter(c=>c.status==='PASS_TARGET_SCOPE'||c.status==='PASS').length;
matrix.not_run_cells=matrix.cells.filter(c=>c.status==='NOT_RUN').length;
if(matrix.not_run_cells===0)matrix.unclosed=matrix.unclosed.map(item=>item==='remaining Workbench cells'?'broader Workbench boundaries beyond these observed route cells':item);
fs.mkdirSync(path.dirname(path.resolve(output)),{recursive:true});fs.writeFileSync(output,JSON.stringify(matrix,null,2)+'\n');
console.log(JSON.stringify({output,observed_pass_cells:matrix.observed_pass_cells,not_run_cells:matrix.not_run_cells,verdict:matrix.verdict}));
