import test from 'node:test';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {readFileSync} from 'node:fs';
import {RuntimeBridgeV1,WasmRuntimeAdapter} from '../solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js';
import {createUniformMotionProvider,uniformMotionSourceContext} from '../solutions/seamgrim_ui_mvp/ui/workbench/notebook/uniform_motion.js';
const require=createRequire(import.meta.url);
const actual=require(process.env.DDN_MATH_WASM);
const adapter=new WasmRuntimeAdapter();adapter.ensureProjectModule=async()=>actual;
const bridge=new RuntimeBridgeV1({adapter});
const raw=readFileSync(process.env.DDN_MATH_ARTIFACT,'utf8');
test('whole admitted project connects actual producer to exact display and source references',async()=>{
 const report=await createUniformMotionProvider(bridge).inspect(raw);
 const view=createUniformMotionProvider(bridge).project(report,raw);
 assert.deepEqual(view.rows,[['0','1'],['2','4'],['4','7']]);
 assert.match(view.solutionMessage,/4 s.*7 m = 7 m/);
 assert.equal(report.authored_source.project_identity,JSON.parse(raw).authored_identity.logical_dependency_id);
 assert.equal(uniformMotionSourceContext(raw).dependencies[0].path,'root.ddn');
});
test('common clone and update preserve model comparison across independent project identities',async()=>{
 const clone=await bridge.authoredProjectOperation({operation:'clone',text:raw});
 const source=structuredClone(JSON.parse(clone).source_snapshot);
 source.files.find(f=>f.path==='main.ddn').text=source.files.find(f=>f.path==='main.ddn').text.replace('3 / 2','2');
 const b=await bridge.authoredProjectOperation({operation:'update',text:JSON.stringify({original_json:clone,source_snapshot:source})});
 const provider=createUniformMotionProvider(bridge);
 const aView=provider.project(await provider.inspect(raw),raw),bView=provider.project(await provider.inspect(b),b);
 assert.notEqual(JSON.parse(raw).authored_identity.logical_dependency_id,JSON.parse(b).authored_identity.logical_dependency_id);
 assert.deepEqual(bView.rows,[['0','1'],['2','5'],['4','9']]);
 assert.deepEqual(aView.comparisonKey,bView.comparisonKey);assert.notDeepEqual(aView.comparedInput,bView.comparedInput);
});
test('returned observation from another artifact cannot become current-source evidence',async()=>{
 const provider=createUniformMotionProvider({
   authoredProjectOperation:x=>bridge.authoredProjectOperation(x),projectGraph:x=>bridge.projectGraph(x),
   observeMathNotebook:async x=>({...await bridge.observeMathNotebook(x),authored_source:{artifact_sha256:'0'.repeat(64)}})
 });
 await assert.rejects(provider.inspect(raw),/원문이 일치하지/);
});
