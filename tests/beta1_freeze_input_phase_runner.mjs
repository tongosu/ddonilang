// Same raw source and original accepted events at the actual CLI3/WASM frontdoors.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const [root,nativeDir,wasmDir,out]=process.argv.slice(2);
assert(root&&nativeDir&&wasmDir&&out);fs.mkdirSync(out);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const binary=wasmDir==='-'?null:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=binary?await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href):null;
if(wasm)await wasm.default({module_or_path:binary});
const template=JSON.parse(fs.readFileSync(path.join(root,'solutions/seamgrim_ui_mvp/samples/alpha4_manipulation_point.binding.json'),'utf8'));
const events=[
 {ordinal:0,channel:'반지름',event:{kind:'Number',payload:{raw:'4294967296',phase:'Set'}}},
 ...['Begin','Move','End'].map((phase,index)=>({ordinal:index+1,channel:'위치',event:{kind:'Coordinate',payload:{raw:['0','0'],phase}}})),
];
const source='(매마디)마다 { 사건목록 <- () 입력사상.사건들. 사건목록마다 { (순서=사건목록.순서) 조작단계 보여주기. }. }.';
const cases=[
 {id:'ordered',source,events,expected:['설정','시작','이동','끝']},
 {id:'reordered',source,events:[events[1],events[0],events[2],events[3]].map((event,ordinal)=>({...event,ordinal})),expected:['시작','설정','이동','끝']},
 {id:'non-consuming',source:source.replace('(순서=사건목록.순서) 조작단계 보여주기.','(순서=사건목록.순서) 조작단계 보여주기. (순서=사건목록.순서) 조작단계 보여주기.'),events,expected:['설정','설정','시작','시작','이동','이동','끝','끝']},
 {id:'warning-advisory',source:'(매마디)마다 { ((순서=0) 조작단계 == "중간") 보여주기. }.',events:[events[0]],expected:['거짓'],warning:1},
 {id:'valid-comparison',source:'(매마디)마다 { ((순서=0) 조작단계 == "설정") 보여주기. }.',events:[events[0]],expected:['참'],warning:0},
 {id:'authored-homonym',source:'(순서:바른수) 조작단계:글:셈씨 := { 조작단계 <- "중간". }. (매마디)마다 { ((순서=0) 조작단계 == "중간") 보여주기. }.',events:[events[0]],expected:['참'],warning:0},
];
const report={scope:wasm?'ACTUAL_RELEASE_CLI3_AND_WASM_RAW_INPUT_PHASE_NOT_CHROME_OR_HUMAN':'ACTUAL_RELEASE_CLI3_ONLY_WASM_AND_CHROME_NOT_RUN',wasm_sha256:binary?sha(binary):null,cases:[]};
try {
 for(const item of cases){
  const sourceFile=path.join(out,item.id+'.ddn');fs.writeFileSync(sourceFile,item.source);
  const binding={...template,source_revision:'sha256:'+sha(item.source)};
  const request={binding,snapshots:[{madi:0,events:item.events,held:{}}]};
  const requestFile=path.join(out,item.id+'.json');fs.writeFileSync(requestFile,JSON.stringify(request));
  const bindingFile=path.join(out,item.id+'-binding.json');fs.writeFileSync(bindingFile,JSON.stringify(binding));
  let warnings=null,summary=null,tape=null,wasmAnalysis=null;
  const tapeFile=path.join(out,item.id+'.tape');
  if(wasm){
   wasmAnalysis=JSON.parse(wasm.wasm_source_analysis_snapshot_json(JSON.stringify({source_text:item.source,path:sourceFile,analysis_generation:0,source_profile_identity:'v1-core-v25'})));
   warnings=wasmAnalysis.diagnostics.filter(row=>row.code==='W_INPUT_PHASE_COMPARE_OUTSIDE_DOMAIN');
   assert.equal(warnings.length,item.warning??0,item.id+': warning provenance');
   const recorded=JSON.parse(wasm.manipulation_input_record_v3(item.source,JSON.stringify(request)));
   tape=Buffer.from(recorded.tape_bytes);summary=recorded.summary;
   assert.deepEqual(JSON.parse(wasm.manipulation_input_replay_v3(item.source,tape,JSON.stringify(binding))),summary);
   fs.writeFileSync(tapeFile,tape);
  }
  const native=[];
  for(const name of ['ddonilang','ddonirang','또니랑']){
   const exe=path.join(nativeDir,name+'.exe');
   const check=spawnSync(exe,['검사',sourceFile,'--source-profile','v1-core-v25'],{encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:32e6});assert.ifError(check.error);
   fs.writeFileSync(path.join(out,item.id+'-'+name+'-analysis.json'),check.stdout);
   assert.equal(check.status,0,check.stderr);const checked=JSON.parse(check.stdout);
   const nativeWarnings=checked.diagnostics.filter(row=>row.code==='W_INPUT_PHASE_COMPARE_OUTSIDE_DOMAIN');
   assert.equal(nativeWarnings.length,item.warning??0,item.id+': '+name+' advisory count');
   for(const warning of nativeWarnings){assert.equal(warning.severity,'warning');assert.equal(Buffer.from(item.source).subarray(warning.span.start,warning.span.end).toString(),'"중간"');}
   if(wasmAnalysis)assert.deepEqual(checked.diagnostics,wasmAnalysis.diagnostics,item.id+': shared diagnostics');
   if(warnings===null)warnings=nativeWarnings;
   for(const [operation,args] of [['record',['조작입력기록',sourceFile,requestFile,path.join(out,item.id+'-'+name+'.tape')]],['replay',['조작입력재생',sourceFile,tapeFile,bindingFile]]]){
    const result=spawnSync(exe,args,{encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:32e6});assert.ifError(result.error);
    fs.writeFileSync(path.join(out,item.id+'-'+name+'-'+operation+'.stdout'),result.stdout);fs.writeFileSync(path.join(out,item.id+'-'+name+'-'+operation+'.stderr'),result.stderr);
    assert.equal(result.status,0,result.stderr);const actual=JSON.parse(result.stdout);
    if(!summary){summary=actual;tape=fs.readFileSync(path.join(out,item.id+'-'+name+'.tape'));fs.writeFileSync(tapeFile,tape);}
    assert.deepEqual(actual,summary,item.id+': '+name+' '+operation);
   }
   native.push({name,sha256:sha(fs.readFileSync(exe))});
  }
  assert.equal(tape.subarray(0,18).toString(),'DDN_INPUT_TAPE_V3\n');assert.deepEqual(summary.output_display.texts,item.expected,item.id);
  report.cases.push({id:item.id,raw_source_sha256:sha(item.source),tape_sha256:sha(tape),state_hash:summary.state_hash,texts:item.expected,warnings,native});
 }
 report.verdict='PASS';
} catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
fs.writeFileSync(path.join(out,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({out,verdict:report.verdict,error:report.error}));
