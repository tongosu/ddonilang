// Raw learner source parity against an explicit extracted candidate, not WIP
// runtime binaries or a test-only lowering of the authored language.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {fileURLToPath,pathToFileURL} from 'node:url';
const identityFile=process.argv[2], outputRoot=process.argv[3];
assert(identityFile&&outputRoot,'node tests/beta1_firststeps_runner.mjs <extracted-identity.json> <output-root>');
const identity=JSON.parse(fs.readFileSync(identityFile,'utf8'));
const captureBaseline=process.argv[4]==='capture-baseline';
const root=identity.extracted, sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const wasmPath=identity.wasm_directory??path.join(root,'solutions/seamgrim_ui_mvp/ui/wasm');
const nativeDirectory=identity.native_directory??path.join(root,'bin');
if(identity.native_directory)assert(['ddonilang','ddonirang','또니랑'].every(name=>/^[a-f0-9]{64}$/.test(identity.native_hashes?.[name]??'')),'explicit native variant requires all three expected binary hashes');
const wasmBytes=fs.readFileSync(path.join(wasmPath,'ddonirang_tool_bg.wasm'));
assert.equal(sha(wasmBytes),identity.wasm_sha256);
const wasm=await import(pathToFileURL(path.join(wasmPath,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:wasmBytes});
const sample=path.join(root,'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris');
const prefix=JSON.parse(fs.readFileSync(path.join(sample,'initial-input.json'),'utf8'));
const suffix=JSON.parse(fs.readFileSync(path.join(sample,'next-input.json'),'utf8'));
assert.deepEqual(prefix.binding,suffix.binding);
const binding=JSON.stringify(prefix.binding);
if(process.argv[4]==='resume-wasm') {
  const vm=wasm.DdnLogicalInputVm.from_saved_project_checkpoint(
    fs.readFileSync(path.join(outputRoot,'saved3.json'),'utf8'),fs.readFileSync(path.join(outputRoot,'checkpoint3.bin')),
    new Uint8Array(),new Uint8Array([2]),binding);
  for(const input of suffix.snapshots)vm.step_json(JSON.stringify(input));
  process.stdout.write(vm.summary_json());vm.free();process.exit(0);
}
const course=path.join(root,'solutions/seamgrim_ui_mvp/first_steps_beta1');
const manifest=JSON.parse(fs.readFileSync(path.join(course,'검증목록.json'),'utf8'));
const output=fs.mkdtempSync(path.join(outputRoot,'첫걸음_31-34_'));
const report={scope:'EXPLICIT_FIXED_PACKAGE_CLI3_NODE_WASM_LEARNER_SOURCES_NOT_BROWSER_OR_NEW_WIP_PRODUCT_PASS',identity,
  wasm_sha256:sha(wasmBytes),base_cases:[],cases:[],tetris:null};
function invoke(name,args,tag) {
  const binary=path.join(nativeDirectory,name+'.exe');
  if(identity.native_directory)assert.equal(sha(fs.readFileSync(binary)),identity.native_hashes[name]);
  const result=spawnSync(binary,args,{encoding:'utf8',windowsHide:true,timeout:90000,maxBuffer:32e6});
  fs.writeFileSync(path.join(output,tag+'.stdout'),result.stdout??'',{flag:'wx'});
  fs.writeFileSync(path.join(output,tag+'.stderr'),result.stderr??'',{flag:'wx'});
  assert.ifError(result.error);return {...result,binary_sha256:sha(fs.readFileSync(binary))};
}
try {
  const base=path.join(root,'solutions/seamgrim_ui_mvp/first_steps');
  const original=JSON.parse(fs.readFileSync(path.join(base,'검증목록.json'),'utf8'));
  const baseRows=[...original.cases,...original.corrections,...original.modifications,
    {id:'SUP06',file:'보충/06_마지막에배우는축약.ddn',expected:['14','14']},
    {id:'SUP09',file:'보충/09_축약대조.ddn',expected:['18','18']}];
  const files=fs.readdirSync(base,{recursive:true}).filter(name=>name.endsWith('.ddn')).map(name=>name.replaceAll('\\','/')).sort();
  assert.equal(baseRows.length,57);assert.deepEqual(baseRows.map(row=>row.file).sort(),files);
  assert.equal(baseRows.filter(row=>!Array.isArray(row.expected)).length,5);
  for(const row of baseRows){
    const file=path.join(base,row.file),source=fs.readFileSync(file,'utf8');let summary,error,wasmOutput;
    try{wasmOutput=wasm.wasm_run_summary_with_source_profile_json(source,file,1,original.profile);summary=JSON.parse(wasmOutput);}catch(cause){error=String(cause);}
    if(captureBaseline)fs.writeFileSync(path.join(output,'base-'+row.id+'.wasm-'+(error?'error.txt':'output.json')),wasmOutput??error,{flag:'wx'});
    let diagnostic;try{diagnostic=JSON.parse(error);}catch{}
    if(Array.isArray(row.expected)){assert.equal(error,undefined,error);assert(Array.isArray(summary.output_log_texts));assert.deepEqual(summary.output_log_texts,row.expected);}
    else{assert(error);assert.equal(summary,undefined);assert(String(diagnostic?.message??error).includes(row.expected.reject),error);}
    const native=[];
    for(const name of ['ddonilang','ddonirang','또니랑']){
      const run=invoke(name,['실행',file,'--source-profile',original.profile,'--마디수','1','--json'],'base-'+row.id+'-'+name);
      if(Array.isArray(row.expected)){
        assert.equal(run.status,0,run.stderr);const actual=JSON.parse(run.stdout);
        assert(Array.isArray(actual.output_log_texts));assert.deepEqual(actual.output_log_texts,row.expected);assert.equal(actual.state_hash,summary.state_hash);
      }else{assert.notEqual(run.status,0);assert.equal(run.stdout.trim(),'');assert(run.stderr.includes(row.expected.reject),run.stderr);}
      native.push({name,exit:run.status,binary_sha256:run.binary_sha256});
    }
    report.base_cases.push({id:row.id,source_sha256:sha(Buffer.from(source)),expected:row.expected,error,diagnostic_code:diagnostic?.code,native});
  }
  for(const row of manifest.cases.filter(row=>row.file)) {
    const file=path.join(course,row.file), source=fs.readFileSync(file,'utf8');
    let summary,error,wasmOutput;
    try{wasmOutput=wasm.wasm_run_summary_with_source_profile_json(source,file,1,'v1-core-v25');summary=JSON.parse(wasmOutput);}catch(e){error=String(e);}
    if(captureBaseline)fs.writeFileSync(path.join(output,row.id+'.wasm-'+(error?'error.txt':'output.json')),wasmOutput??error,{flag:'wx'});
    if(row.expected_error){assert(error?.includes(row.expected_error),error);assert.equal(summary,undefined);}
    else{assert.equal(error,undefined,error);assert.deepEqual(summary.output_display.texts,row.expected);}
    const native=[];
    for(const name of ['ddonilang','ddonirang','또니랑']) {
      const run=invoke(name,['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],row.id+'-'+name);
      if(row.expected_error){assert.notEqual(run.status,0);assert.equal(run.stdout.trim(),'');assert(run.stderr.includes(row.expected_error),run.stderr);}
      else{assert.equal(run.status,0,run.stderr);const actual=JSON.parse(run.stdout);assert.deepEqual(actual.output_display.texts,row.expected);assert.equal(actual.state_hash,summary.state_hash);}
      native.push({name,exit:run.status,binary_sha256:run.binary_sha256});
    }
    report.cases.push({id:row.id,file,source_sha256:sha(Buffer.from(source)),texts:summary?.output_display.texts,
      state_hash:summary?.state_hash,error,native});
  }
  const lesson=manifest.cases.find(row=>row.id==='34');
  assert.equal(sha(fs.readFileSync(path.join(sample,'main.ddn'))),lesson.source_sha256);
  const authored=fs.readFileSync(path.join(sample,'authored-project.json'),'utf8');
  for(const file of JSON.parse(authored).source_snapshot.files)assert.equal(file.text,fs.readFileSync(path.join(sample,file.path),'utf8'));
  let vm=wasm.DdnLogicalInputVm.from_authored_project(authored,binding);
  for(const input of prefix.snapshots)vm.step_json(JSON.stringify(input));
  fs.writeFileSync(path.join(output,'saved3.json'),vm.export_saved_project(new Uint8Array(),new Uint8Array([2])),{flag:'wx'});
  fs.writeFileSync(path.join(output,'checkpoint3.bin'),vm.export_project_checkpoint(new Uint8Array(),new Uint8Array([2])),{flag:'wx'});vm.free();
  const child=spawnSync(process.execPath,[fileURLToPath(import.meta.url),identityFile,output,'resume-wasm'],{encoding:'utf8',windowsHide:true,timeout:90000,maxBuffer:32e6});
  assert.ifError(child.error);assert.equal(child.status,0,child.stderr);const resumed=JSON.parse(child.stdout);
  fs.writeFileSync(path.join(output,'wasm-resumed7.json'),child.stdout,{flag:'wx'});
  vm=wasm.DdnLogicalInputVm.from_authored_project(authored,binding);
  for(const input of [...prefix.snapshots,...suffix.snapshots])vm.step_json(JSON.stringify(input));
  const continuous=JSON.parse(vm.summary_json());vm.free();
  const project=summary=>Object.fromEntries(['state_hash','resources','frames','input_receipts','madi_count'].map(key=>[key,summary[key]]));
  assert.deepEqual(project(resumed),project(continuous));assert.equal(resumed.state_hash,lesson.expected_state_hash);
  const seven=path.join(output,'seven-input.json');fs.writeFileSync(seven,JSON.stringify({binding:prefix.binding,snapshots:[...prefix.snapshots,...suffix.snapshots]}),{flag:'wx'});
  const rows=[];
  for(const name of ['ddonilang','ddonirang','또니랑']) {
    const store=path.join(output,name+'-store'),baselineStore=path.join(output,name+'-continuous');fs.mkdirSync(store);fs.mkdirSync(baselineStore);
    const start=invoke(name,['작품자료','논리기록',path.join(sample,'authored-project.json'),'--입력',path.join(sample,'initial-input.json'),'--부모','처음','--판','02','--저장소',store,'--저장소-초기화'],'34-save-'+name);
    assert.equal(start.status,0,start.stderr);const saved=JSON.parse(start.stdout),file=path.join(output,name+'-saved.json');fs.writeFileSync(file,saved.saved_artifact_json_utf8,{flag:'wx'});
    const cold=invoke(name,['작품자료','이어실행',file,'--부모','처음','--판','02','--마디수','4','--저장소',store,'--거래',saved.transaction_digest_sha256,'--입력',path.join(sample,'next-input.json')],'34-resume-'+name);
    const baseline=invoke(name,['작품자료','논리기록',path.join(sample,'authored-project.json'),'--입력',seven,'--부모','처음','--판','02','--저장소',baselineStore,'--저장소-초기화'],'34-continuous-'+name);
    assert.equal(cold.status,0,cold.stderr);assert.equal(baseline.status,0,baseline.stderr);
    assert.deepEqual(project(JSON.parse(cold.stdout)),project(JSON.parse(baseline.stdout).summary));assert.deepEqual(project(JSON.parse(cold.stdout)),project(resumed));
    rows.push({name,binary_sha256:cold.binary_sha256,separate_processes:3,state_hash:resumed.state_hash});
  }
  report.tetris={source_sha256:lesson.source_sha256,rows,wasm_separate_resume_processes:1,state_hash:resumed.state_hash};report.verdict='PASS';
}catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
fs.writeFileSync(path.join(output,'RESULT.json'),JSON.stringify(report,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({output,verdict:report.verdict,error:report.error}));
