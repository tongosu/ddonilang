import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';

// The artifact and baseline are produced by the actual Native first-world
// cold-process test. No source rewriting, ID issuance or JSON authority here.
const [wasmDir, nativeRoot, reportPath, expected='green'] = process.argv.slice(2);
const api = await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')));
await api.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const artifact=fs.readFileSync(path.join(nativeRoot,'saved-artifact.json'),'utf8');
const baseline=JSON.parse(fs.readFileSync(path.join(nativeRoot,'cold-result.json'),'utf8'));
let vm;
try { vm=api.DdnWasmVm.from_saved_project_artifact_json(artifact,'01','02'); }
catch(error) {
  if(expected==='subject-json-only-red') {
    assert.equal(String(error),'E_VC04_CARRIER_ADMISSION: SubjectCustodyInvalid');
  } else {
    if(expected!=='red') throw error;
    assert.match(String(error),/E_VC04_REGISTERED_CONTEXT/);
  }
  fs.writeFileSync(reportPath,JSON.stringify({verdict:'EXPECTED_RED',error:String(error),wasmDir,nativeRoot},null,2));
  console.log('EXPECTED_RED',String(error)); process.exit(0);
}
assert.equal(expected,'green','prelink unexpectedly admitted the saved nominal');
const parse=value=>typeof value==='string'?JSON.parse(value):value;
const stepped=parse(vm.step_one());
assert.equal(stepped.state_hash,baseline.state_hash);
// Compare public summaries to public summaries (Fixed64 display text), and
// typed carrier DTOs to typed carrier DTOs. Snapshot Fixed64 uses raw_i64 and
// is deliberately not reinterpreted as public display text in this runner.
for(const channel of ['fixed64','handle','json','value','value_json']) {
  assert.deepEqual(stepped.resources[channel],baseline.resources[channel],channel);
}
assert.deepEqual(stepped.state.snapshot_v2.resources.value_det,baseline.resources.value_det);
assert.deepEqual(stepped.output_display,baseline.output_display);
const exported=vm.export_saved_project_artifact_json('02','03');
const before=parse(vm.get_state_json());
vm.free();
vm=api.DdnWasmVm.from_saved_project_artifact_json(exported,'02','03');
const reopened=parse(vm.get_state_json());
assert.equal(reopened.state_hash,before.state_hash);
assert.deepEqual(reopened.resources,before.resources);
const mutated=structuredClone(reopened);
const carrierType=mutated.state.snapshot_v2.resources.value_det['이름표'].type;
assert.ok(['vc04_canonical_carrier','tuple_v1'].includes(carrierType),'closed tested carrier kinds');
mutated.state.snapshot_v2.resources.value_det['이름표'].admitted=true;
let refusal='';
try {vm.restore_state(JSON.stringify(mutated));} catch(error) {refusal=String(error);}
// Extra keys are rejected by the existing ordered JSON transport gate before
// the contextual binary decoder. Keep that earlier, exact refusal visible.
assert.match(refusal,carrierType==='tuple_v1'?/위치형 저장 필드 순서가 다릅니다/:/E_VC04_CARRIER_FIELDS/);
assert.equal(parse(vm.get_state_json()).state_hash,reopened.state_hash);
vm.free();
fs.writeFileSync(reportPath,JSON.stringify({verdict:'PASS',scope:'actual public WASM VM; not Chrome or durable Native receipt',wasmDir,nativeRoot,state_hash:stepped.state_hash,resources_equal:true,reopen_equal:true,mutation_refusal:refusal},null,2));
console.log('PASS',stepped.state_hash);
