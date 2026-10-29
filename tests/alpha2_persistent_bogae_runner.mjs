import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

assert.ok(process.env.DDN_NATIVE_DIR && process.env.DDN_WASM_DIR, 'source-built products required');
const wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const bytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:bytes});
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-persistent-bogae-'));
const natives=['ddonirang.exe','ddonilang.exe','또니랑.exe'];
let counter=0;
function cli(executable,args,text) {
  const file=path.join(evidence,`input-${++counter}.json`);
  fs.writeFileSync(file,text,'utf8');
  const result=spawnSync(path.join(process.env.DDN_NATIVE_DIR,executable),['작품자료',...args.slice(0,1),file,...args.slice(1)],
    {encoding:'utf8',timeout:180000,maxBuffer:16*1024*1024});
  assert.equal(result.status,0,result.stderr || result.stdout);
  return JSON.parse(result.stdout);
}
function operation(operation,text) {return wasm.authored_project_operation(operation,text);}
const reports=[];
for(const [name,definition,ticks] of [['linear','일차함수짜임',1],['parabola','포물선짜임',17],['pendulum','진자짜임',17]]) {
  const source=fs.readFileSync(`solutions/seamgrim_ui_mvp/samples/alpha2_${name}_composition.ddn`,'utf8');
  const snapshot={entry_path:'main.ddn',files:[{path:'main.ddn',text:source}]};
  const v1=operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'앱',source_snapshot:snapshot}));
  assert.equal(JSON.parse(v1).schema,'ddn.authored_project_artifact.v1');
  assert.throws(()=>wasm.wasm_run_summary_with_authored_bogae_json(source,'main.ddn',ticks,'v1-core-v25',v1), /E_AUTHORED/);
  const artifactText=operation('register_bogae',JSON.stringify({original_json:v1,module_name:'앱',source_path:'main.ddn',composition_definition:definition,make_entry:true}));
  const stored=JSON.parse(artifactText);
  assert.equal(stored.schema,'ddn.authored_project_artifact.v2');
  assert.equal(operation('open',artifactText),artifactText);
  const summary=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(source,'main.ddn',ticks,'v1-core-v25',artifactText));
  const relocated=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(source,'renamed-locator.ddn',ticks,'v1-core-v25',artifactText));
  assert.deepEqual(relocated.bogae_projection,summary.bogae_projection,'a diagnostic locator is not authored identity');
  assert.equal(relocated.state_hash,summary.state_hash);
  assert.equal(summary.bogae_projection.status,'ready');
  assert.equal(summary.bogae_projection.persistent_identity,true);
  for(const executable of natives) {
    assert.deepEqual(cli(executable,['열기'],artifactText),stored);
    const native=cli(executable,['실행','--마디수',String(ticks)],artifactText);
    // File/path diagnostics may differ; identity, world and typed projection may not.
    assert.deepEqual(native.bogae_projection,summary.bogae_projection);
    assert.deepEqual(native.state_hash,summary.state_hash);
    assert.ok(summary.resources && summary.work_cost_receipt);
    assert.deepEqual(native.resources,summary.resources);
    assert.deepEqual(native.work_cost_receipt,summary.work_cost_receipt);
  }
  const one=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(source,'main.ddn',1,'v1-core-v25',artifactText));
  assert.deepEqual(one.bogae_projection.candidates[0].artifact.identity,summary.bogae_projection.candidates[0].artifact.identity);
  assert.equal(one.bogae_projection.candidates[0].authored_digest,summary.bogae_projection.candidates[0].authored_digest,
    'current geometry must not change authored identity/digest');
  const temporary=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source,'main.ddn',ticks,'v1-core-v25'));
  assert.equal(temporary.state_hash,summary.state_hash,'identity must not change world');
  const copyText=operation('clone',artifactText),copy=JSON.parse(copyText);
  const copied=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(source,'main.ddn',ticks,'v1-core-v25',copyText));
  assert.notEqual(copy.authored_identity.logical_dependency_id,stored.authored_identity.logical_dependency_id);
  assert.notDeepEqual(copied.bogae_projection.candidates[0].artifact.identity,summary.bogae_projection.candidates[0].artifact.identity);
  assert.equal(copied.state_hash,summary.state_hash);
  const newName=definition+'이름변경';
  const renamedSource=source.replaceAll(definition,newName);
  const renamedSnapshot={entry_path:'main.ddn',files:[{path:'main.ddn',text:renamedSource}]};
  const renameRequest=JSON.stringify({original_json:artifactText,owner_module_id:stored.authored_identity.modules[0].id,
    target_kind:'composition',target_id:stored.authored_identity.modules[0].compositions[0].id,new_name:newName,source_snapshot:renamedSnapshot});
  const renamedText=operation('rename',renameRequest),renamedArtifact=JSON.parse(renamedText);
  const moveRequest=JSON.stringify({original_json:renamedText,old_path:'main.ddn',new_path:'학습/main.ddn',
    source_snapshot:{entry_path:'학습/main.ddn',files:[{path:'학습/main.ddn',text:renamedSource}]}});
  const movedText=operation('relocate',moveRequest),movedArtifact=JSON.parse(movedText);
  const movedSummary=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(renamedSource,'학습/main.ddn',ticks,'v1-core-v25',movedText));
  assert.deepEqual(movedSummary.bogae_projection.candidates[0].artifact.identity,summary.bogae_projection.candidates[0].artifact.identity);
  assert.equal(movedSummary.bogae_projection.candidates[0].authored_digest,summary.bogae_projection.candidates[0].authored_digest);
  assert.equal(movedSummary.state_hash,summary.state_hash);
  for(const executable of natives) {
    assert.deepEqual(cli(executable,['이름변경'],renameRequest),renamedArtifact);
    assert.deepEqual(cli(executable,['경로변경'],moveRequest),movedArtifact);
    const movedNative=cli(executable,['실행','--마디수',String(ticks)],movedText);
    for(const key of ['state_hash','resources','bogae_projection','work_cost_receipt'])assert.deepEqual(movedNative[key],movedSummary[key],`${executable} explicit lifecycle ${key}`);
  }
  assert.throws(()=>wasm.wasm_run_summary_with_authored_bogae_json(source+'\n1 보여주기.','main.ddn',ticks,'v1-core-v25',artifactText), /E_AUTHORED/);
  const vm=new wasm.DdnWasmVm('');
  vm.update_logic_with_source_profile(source,'v1-core-v25');
  const before=vm.get_state_json();
  assert.throws(()=>vm.configure_authored_bogae(artifactText.replace('"content_sha256":"','"content_sha256":"f')), /E_AUTHORED/);
  assert.equal(vm.get_state_json(),before,'invalid admission must not change world/frame');
  vm.configure_authored_bogae(artifactText);
  vm.step_one();
  const committed=vm.get_state_json();
  assert.throws(()=>vm.configure_authored_bogae(copyText), /E_AUTHORED_BOGAE_LIFECYCLE/);
  assert.equal(vm.get_state_json(),committed);
  vm.free();
  reports.push({name,definition,ticks,source_sha256:createHash('sha256').update(source).digest('hex'),
    authored_digest:summary.bogae_projection.candidates[0].authored_digest,state_hash:summary.state_hash});
}
// Formal work uses the product linker and its exact original-source map. The
// fixture is authored DDN, never lowered or rewritten by this test harness.
const multiSnapshot={root_path:'selected.ddn',entry_path:'app/main.ddn',files:[
  {path:'selected.ddn',text:'(글판="v25",모듬=차림[(이름=앱,머리="app/main.ddn",원문="app/main.ddn"),(이름=힘,머리="lib/힘.ddn",원문="lib/힘.ddn")],시작=(모듈=앱,정의=실행)) 얽음.'},
  {path:'app/main.ddn',text:'(힘=힘) 쓰임.\n원 := (중심=차림[힘.값,0],반지름=0.2) 표준장면.원.\n장면 := (요소들=차림[원],가로범위=차림[0,4],세로범위=차림[-2,2]) 표준장면.평면.\n으뜸보기 := (보기=((장면=장면) 표준보기.장면)) 표준보개.으뜸.\n작품짜임 := (관계들=차림[으뜸보기]) 표준보개.짜임.\n실행:움직씨 := { 힘.값 보여주기. }.'},
  {path:'lib/힘.ddn',text:'(값=값) 드러냄.\n값 := 2.'},
]};
const multiV1=operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:multiSnapshot}));
const multiText=operation('register_bogae',JSON.stringify({original_json:multiV1,module_name:'앱',source_path:'app/main.ddn',composition_definition:'작품짜임',make_entry:true}));
const prepared=JSON.parse(wasm.wasm_project_prepare_link_json(JSON.stringify({schema:'ddn.virtual_project_source.v1',root_path:multiSnapshot.root_path,entry_path:multiSnapshot.entry_path,
  files:multiSnapshot.files.map(file=>({path:file.path,utf8_text:file.text,role:file.path===multiSnapshot.root_path?'root':'ddn_source'}))}),'실행',true,false));
const linked=JSON.parse(wasm.wasm_run_summary_with_authored_bogae_json(prepared.linked_execution_source,prepared.target_path,1,'v1-core-v25',multiText));
assert.equal(linked.bogae_projection.persistent_identity,true);
assert.deepEqual(linked.output_log_texts,['2']);
for(const executable of natives) {
  const native=cli(executable,['실행','--마디수','1'],multiText);
  for(const key of ['state_hash','resources','bogae_projection','output_log_texts','work_cost_receipt']) assert.deepEqual(native[key],linked[key],`${executable} linked ${key}`);
}
const linkedVM=new wasm.DdnWasmVm('');
linkedVM.update_logic_with_source_profile(prepared.linked_execution_source,'v1-core-v25');
linkedVM.configure_authored_bogae(multiText); // Stored graph start, not a host default.
linkedVM.step_one();
assert.equal(linkedVM.get_state_hash(),linked.state_hash);
assert.deepEqual(JSON.parse(linkedVM.get_state_json()).bogae_projection,linked.bogae_projection);
linkedVM.free();
assert.throws(()=>wasm.wasm_run_summary_with_authored_bogae_json(prepared.linked_execution_source+'\n1 보여주기.',prepared.target_path,1,'v1-core-v25',multiText), /E_AUTHORED/);
fs.writeFileSync(path.join(evidence,'linked-project.json'),multiText);
reports.push({name:'formal_multi_module',state_hash:linked.state_hash,authored_digest:linked.bogae_projection.candidates[0].authored_digest});
fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify({wasm_sha256:createHash('sha256').update(bytes).digest('hex'),reports},null,2));
console.log(JSON.stringify({status:'PASS',cases:reports.length,native_names:natives,evidence}));
