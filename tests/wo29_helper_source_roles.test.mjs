// Focused tests of the actual authoredProjectPanel.apply source, not a DDN
// evaluator or a replacement for the real Rust/WASM/Chrome integration gate.
import fs from 'node:fs';
import vm from 'node:vm';
import path from 'node:path';
import assert from 'node:assert/strict';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const source=fs.readFileSync(path.join(root,'solutions/seamgrim_ui_mvp/ui/workbench/workbench.js'),'utf8');
const marker='  apply:async (artifact,projectContext={})=>{';
const start=source.indexOf(marker,source.indexOf('const authoredProjectPanel ='));
const end=source.indexOf('\n  },\n});',start);
assert.ok(start>=0&&end>start);
const body=source.slice(start+marker.length,end);
const artifact=JSON.parse(fs.readFileSync(path.join(root,'tool/tests/fixtures/v2_helper_source.json'),'utf8'));
const expectedRoles=['ddn_source','metadata','ddn_source','gaji_manifest','root','lock'];
function setup(change) {
  const state={projectId:'before',projectEditGeneration:1,sourceAnalysisGeneration:2,completionGeneration:3,activeFilePath:'before.ddn'};
  const registry={receivedPortableProjects:[]};let calls=0,selected=0,invalidated=0;
  const input={value:'old'};
  const context={state,workbenchProjectRegistryState:registry,
    readMyWorkIndex:()=>[],currentProject:()=>({sourceProfileIdentity:'v1-core-v25'}),
    workbenchSourceProfile:p=>p.sourceProfileIdentity,
    runtimeBridge:{async authoredProjectOperation(request){
      calls++;assert.equal(request.operation,'source_project');
      assert.deepEqual(JSON.parse(request.text),artifact);
      const projected={schema:'ddn.virtual_project_source.v1',root_path:artifact.source_snapshot.root_path,
        entry_path:artifact.source_snapshot.entry_path,files:artifact.source_snapshot.files.map((file,i)=>({path:file.path,utf8_text:file.text,role:expectedRoles[i]}))};
      if(change)await change({state,projected});
      return JSON.stringify(projected);
    }},
    selectProject:async id=>{selected++;state.projectId=id;state.projectEditGeneration++;},
    proofSourceInspection:{invalidate(){invalidated++;}},document:{querySelector:()=>input},
  };
  const apply=vm.runInNewContext('(async (artifact,projectContext={})=>{'+body+'\n})',context);
  return {apply,state,registry,input,stats:()=>({calls,selected,invalidated})};
}
test('whole helper projection preserves six raw files and the three metadata roles',async()=>{
  const h=setup();await h.apply(artifact);
  const p=h.registry.receivedPortableProjects[0];assert.equal(p.files.length,6);
  assert.deepEqual(Array.from(p.files,file=>file.role),expectedRoles);
  p.files.forEach((file,i)=>assert.equal(file.fallback,artifact.source_snapshot.files[i].text));
  assert.equal(p.portableSnapshot.local_package_payload.files.length,6);
  assert.deepEqual(h.stats(),{calls:1,selected:1,invalidated:1});
});
test('legacy single-file source does not call the new rooted projection',async()=>{
  const h=setup();const single=structuredClone(artifact);delete single.source_snapshot.root_path;
  single.source_snapshot.files=[{path:'main.ddn',text:'값 := 1.'}];single.source_snapshot.entry_path='main.ddn';
  await h.apply(single);assert.equal(h.registry.receivedPortableProjects[0].files[0].role,'main');
  assert.equal(h.stats().calls,0);
});
for(const field of ['projectId','projectEditGeneration','sourceAnalysisGeneration','completionGeneration','activeFilePath']) {
  test('stale '+field+' never selects or publishes the imported project',async()=>{
    const h=setup(({state})=>{state[field]=String(state[field])+'changed';});
    await assert.rejects(h.apply(artifact),/E_PROJECT_EDIT_SOURCE_CONFLICT/);
    assert.equal(h.registry.receivedPortableProjects.length,0);
    assert.deepEqual(h.stats(),{calls:1,selected:0,invalidated:0});
    assert.equal(h.input.value,'old');
  });
}
for(const kind of ['dropped_member','changed_bytes','changed_entry']) {
  test('invalid '+kind+' preserves the current UI instead of filtering source',async()=>{
    const h=setup(({projected})=>{
      if(kind==='dropped_member')projected.files.pop();
      if(kind==='changed_bytes')projected.files[0].utf8_text+=' ';
      if(kind==='changed_entry')projected.entry_path='other.ddn';
    });
    await assert.rejects(h.apply(artifact),/E_PROJECT_RUNTIME_SOURCE_SUPPLY/);
    assert.equal(h.registry.receivedPortableProjects.length,0);assert.equal(h.stats().selected,0);
  });
}
