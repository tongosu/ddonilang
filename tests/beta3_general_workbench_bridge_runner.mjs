// Real current adapter and real shipped Rust/WASM; module loading alone is
// injected. Not a Native HTTP admission, actual worker or Chrome UI proof.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
const [stage,previous,out]=process.argv.slice(2);assert(stage&&previous&&out&&!fs.existsSync(out));
fs.mkdirSync(out,{recursive:true});
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const moduleFile=new URL('../solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js',import.meta.url);
const {WasmRuntimeAdapter}=await import(moduleFile.href);
const dir=path.join(stage,'solutions/seamgrim_ui_mvp/ui/wasm');
const wasm=await import(pathToFileURL(path.join(dir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'))});
const baseline=JSON.parse(fs.readFileSync(path.join(previous,'RESULT.json'),'utf8'));assert.equal(baseline.verdict,'PASS_TARGET_SCOPE');
const adapter=new WasmRuntimeAdapter();let moduleRequests=0;
adapter.ensureProjectModule=async()=>{moduleRequests++;return wasm;};
const checks=[];
for(const row of baseline.checks){
  const bytes=fs.readFileSync(path.join(previous,row.alias+'-original.bundle'));assert.equal(sha(bytes),row.original_bundle_sha256);
  const actual=JSON.parse(await adapter.compareGeneralOriginalRecording({bundleHex:bytes.toString('hex')}));
  assert.equal(actual.result_identity,row.readonly_result_identity);assert.equal(actual.status,'RUNNING_RESULT_COMPARED');
  assert.equal(actual.accepted_count_decimal,'5');checks.push({alias:row.alias,original_typed_result_equal:true});
  await assert.rejects(()=>adapter.compareGeneralOriginalRecording({bundleHex:bytes.subarray(0,bytes.length-1).toString('hex')}));
}
const before=moduleRequests;
await assert.rejects(()=>adapter.compareGeneralOriginalRecording({bundleHex:'zz'}),/E_GENERAL_RECORDING_BYTES/);
await assert.rejects(()=>adapter.compareGeneralOriginalRecording({bundleHex:'00'.repeat(8*1024*1024+1)}),/E_GENERAL_RECORDING_BYTES/);
assert.equal(moduleRequests,before,'format/size refusal must precede WASM materialization');
const report={verdict:'PASS_ADAPTER_TRANSPORT_ACTUAL_TYPED_WASM_ONLY',
  claim:'NOT_NATIVE_HTTP_ACTUAL_WORKER_CHROME_NORMAL_END_OR_WHOLE_PRODUCT_PASS',
  adapter_source_sha256:sha(fs.readFileSync(moduleFile)),wasm_sha256:sha(fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'))),
  checks,truncated_common_rust_refused:true,invalid_and_oversized_hex_refused_before_module:true};
fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
