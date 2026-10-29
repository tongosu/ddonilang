// Actual VC07 source ingress: the same fixture bytes enter Native three
// names and the generated WASM frontdoor. No test-only lowering.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import {createHash} from "node:crypto";
import {spawnSync} from "node:child_process";
import {pathToFileURL} from "node:url";
const hash=bytes=>createHash("sha256").update(bytes).digest("hex");
const bins=JSON.parse(process.env.DDN_CLI_BINS||"[]");
assert.equal(bins.length,3,"supply all three real CLI names");
assert.ok(process.env.DDN_WASM_DIR);
const dir=path.resolve(process.env.DDN_WASM_DIR);
const wasmBytes=fs.readFileSync(path.join(dir,"ddonirang_tool_bg.wasm"));
const wasm=await import(pathToFileURL(path.join(dir,"ddonirang_tool.js")).href);
await wasm.default({module_or_path:wasmBytes});
const pack="pack/v1_core_supported_ingress_w1";
const manifestBytes=fs.readFileSync(path.join(pack,"manifest.detjson"));
const caseField=process.env.DDN_CASE_FIELD||"vc07_cases";
assert.ok(["vc07_cases","vc06_bytes_cases"].includes(caseField));
const cases=JSON.parse(manifestBytes)[caseField];
const results=[];
for(const [name,expected,diagnostic] of cases){
  const file=path.join(pack,"fixtures",name), sourceBytes=fs.readFileSync(file);
  const source=sourceBytes.toString("utf8");
  let result,error;
  try{result=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source,file,1,"v1-core-v25"));}
  catch(e){error=JSON.parse(String(e));}
  if(expected===null){
    assert.ok(error,name);assert.equal(error.code,diagnostic,name);
    for(const key of ["state_hash","trace_hash","output_log_texts","work_cost_receipt"])assert.ok(!(key in error),name+":"+key);
  }else{
    assert.ok(result,name+":"+JSON.stringify(error));assert.deepEqual(result.output_log_texts,expected,name);
  }
  for(const bin of bins){
    const run=spawnSync(bin,["실행",file,"--source-profile","v1-core-v25","--마디수","1","--json"],{encoding:"utf8"});
    if(expected===null){
      assert.notEqual(run.status,0,name);
      const nativeError=JSON.parse(run.stderr.trim());
      if(nativeError.schema==="ddn.source_analysis_snapshot.candidate.v1"){
        assert.equal(nativeError.outcome,"REJECTED",name);
        assert.ok(nativeError.diagnostics.some(d=>d.severity==="error" && d.code===diagnostic),name+":"+run.stderr);
      }else assert.equal(nativeError.code,diagnostic,name+":"+run.stderr);
      for(const key of ["state_hash","trace_hash","output_log_texts","work_cost_receipt"])
        assert.ok(!(key in nativeError),name+":"+key);
    }else{
      assert.equal(run.status,0,name+":"+run.stderr);
      const native=JSON.parse(run.stdout);
      for(const key of ["state_hash","output_log_texts"])assert.deepEqual(native[key],result[key],name+":"+key);
    }
  }
  results.push({file,source_sha256:hash(sourceBytes),expected,diagnostic:diagnostic||null,
    state_hash:result?.state_hash||null,native_three_names:"PASS",wasm:"PASS"});
}
const report={scope:caseField+" actual source Native3/WASM only",wasm_sha256:hash(wasmBytes),
  manifest_sha256:hash(manifestBytes),cases:results.length,results};
const out=fs.mkdtempSync(path.join(os.tmpdir(),"ddn-vc07-record-parity-"));
fs.writeFileSync(path.join(out,"report.json"),JSON.stringify(report,null,2));
console.log(JSON.stringify({report:path.join(out,"report.json"),report_sha256:hash(fs.readFileSync(path.join(out,"report.json"))),
  cases:results.length,wasm_sha256:report.wasm_sha256,status:"PASS"}));
