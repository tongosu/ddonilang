// Product-private CLI live finish; the unchanged C01 runner supplies its raw
// source/requests. This harness never encodes a tape or lowers source text.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
const [stage,c01Report,out]=process.argv.slice(2);assert(stage&&c01Report&&out&&!fs.existsSync(out));
fs.mkdirSync(out,{recursive:true});const sha=b=>createHash('sha256').update(b).digest('hex');
const c01=JSON.parse(fs.readFileSync(c01Report,'utf8'));assert.equal(c01.status,'PASS_ACTUAL_CLI3_WASM_ORDER_SEED_PACING');
const wasmDir=path.join(stage,'solutions/seamgrim_ui_mvp/ui/wasm');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const report={claim:'PRIVATE_CLI_LIVE_FINISH_CONTENT_ONLY_NOT_NATIVE_DURABILITY_OR_WHOLE_BETA3',
 wasm_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))),c01_report_sha256:sha(fs.readFileSync(c01Report)),cases:[]};
const cases=c01.rows.map(row=>({key:row.key,source:path.join(c01.out,'source.ddn'),request:JSON.parse(fs.readFileSync(path.join(c01.out,row.key+'.json'),'utf8')),authored:false}));
// A live finish does not synthesize Release just to make held state all false.
cases.push({...cases[0],key:'held-true',request:{...cases[0].request,snapshots:cases[0].request.snapshots.slice(0,1)}});
const sample=path.join(stage,'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris');
const first=JSON.parse(fs.readFileSync(path.join(sample,'initial-input.json'),'utf8'));
const next=JSON.parse(fs.readFileSync(path.join(sample,'next-input.json'),'utf8'));assert.deepEqual(first.binding,next.binding);
cases.push({key:'authored-tetris-seven',source:path.join(sample,'authored-project.json'),authored:true,
 request:{binding:first.binding,snapshots:[...first.snapshots,...next.snapshots]}});
for(const item of cases){
 const source=fs.readFileSync(item.source,'utf8');
 const expected=JSON.parse(wasm[item.authored?'authored_logical_input_record_v3':'logical_input_record'](source,JSON.stringify(item.request)));
 const requestFile=path.join(out,item.key+'.json');fs.writeFileSync(requestFile,JSON.stringify(item.request));
 const bindingFile=path.join(out,item.key+'-binding.json');fs.writeFileSync(bindingFile,JSON.stringify(item.request.binding));
 const row={key:item.key,source_sha256:sha(fs.readFileSync(item.source)),aliases:[]};
 for(const name of ['ddonilang','ddonirang','또니랑']){
  const invoke=args=>spawnSync(path.join(stage,'bin',name+'.exe'),args,{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:32*1024*1024});
  const output=path.join(out,item.key+'-'+name+'.bin'),flags=item.authored?['--authored']:[];
  const result=invoke(['logical-record',item.source,requestFile,output,...flags,'--finish-live-private']);
  assert.ifError(result.error);assert.equal(result.status,0,result.stderr);assert.deepEqual(JSON.parse(result.stdout),expected.summary);
  assert.deepEqual(fs.readFileSync(output),Buffer.from(expected.tape_bytes));
  const replay=invoke(['logical-replay',item.source,output,bindingFile,...flags]);
  assert.ifError(replay.error);assert.equal(replay.status,0,replay.stderr);assert.deepEqual(JSON.parse(replay.stdout),expected.summary);
  const before=fs.readFileSync(output);
  const badFile=path.join(out,item.key+'-'+name+'-bad.json'),badOutput=path.join(out,item.key+'-'+name+'-bad.bin');
  fs.writeFileSync(badFile,JSON.stringify({...item.request,snapshots:[...item.request.snapshots,item.request.snapshots[0]]}));
  const failed=invoke(['logical-record',item.source,badFile,badOutput,...flags,'--finish-live-private']);
  assert.ifError(failed.error);assert.notEqual(failed.status,0);assert.equal(failed.stdout,'');assert.equal(fs.existsSync(badOutput),false);
  const denied=invoke(['logical-record',item.source,badFile,output,...flags,'--finish-live-private']);
  assert.ifError(denied.error);assert.notEqual(denied.status,0);assert.equal(denied.stdout,'');assert.deepEqual(fs.readFileSync(output),before);
  row.aliases.push({name,bytes_summary_replay_equal:true,existing_artifact_preserved:true,failed_input_no_artifact_or_stdout:true});
 }
 report.cases.push(row);
}
report.verdict='PASS_PRIVATE_LIVE_FINISH_CLI3_WASM';fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({verdict:report.verdict,cases:report.cases.length,out}));
