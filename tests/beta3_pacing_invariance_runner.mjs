// Host delay is measured only by this verifier, never sent to the world.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
const [stage,out,...fixtures]=process.argv.slice(2);assert(stage&&out&&fixtures.length&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const sha=b=>createHash('sha256').update(b).digest('hex');
const wasmDir=path.join(stage,'solutions/seamgrim_ui_mvp/ui/wasm'),wasmBytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);await wasm.default({module_or_path:wasmBytes});
const report={scope:'SAME_ACTUAL_NATIVE_SAVED_IDENTITY_WASM_PACING_AND_WALLCLOCK_ONLY_NOT_RECEIPT_OR_BETA3_COMPLETE',
 wasm_sha256:sha(wasmBytes),rows:[],verdict:'NOT_RUN'};
report.target=JSON.parse(fs.readFileSync(path.join(stage,'BETA3_DEVELOPMENT_MANIFEST.json')));
const parse=x=>typeof x==='string'?JSON.parse(x):x;let vm;
try{
 for(const fixture of fixtures){
  const bytes=fs.readFileSync(path.join(fixture,'saved3.json')),sourceHash=sha(bytes);
  const run=async delays=>{
   vm=wasm.DdnWasmVm.from_saved_project_artifact_json(bytes.toString('utf8'),'01','02');
   const frames=[],start=process.hrtime.bigint();
   for(const ms of delays){if(ms)await new Promise(r=>setTimeout(r,ms));frames.push(parse(vm.step_one()));}
   const state=parse(vm.get_state_json());vm.free();vm=null;
   return {frames,state,elapsed_ns:String(process.hrtime.bigint()-start)};
  };
  const fast=await run([0,0,0,0]),slow=await run([100,100,100,100]),varied=await run([1,37,5,80]);
  assert.deepEqual(slow.frames,fast.frames,'all committed worlds, input receipts, output and scenes independent of host delay');
  assert.deepEqual(varied.frames,fast.frames);assert.deepEqual(slow.state,fast.state);assert.deepEqual(varied.state,fast.state);
  assert.equal(sha(fs.readFileSync(path.join(fixture,'saved3.json'))),sourceHash);
  const index=report.rows.length;
  fs.writeFileSync(path.join(out,`${index}-committed-frames.json`),JSON.stringify(fast.frames));
  report.rows.push({fixture,saved_sha256:sourceHash,delays_ms:[[0,0,0,0],[100,100,100,100],[1,37,5,80]],
   elapsed_ns:[fast.elapsed_ns,slow.elapsed_ns,varied.elapsed_ns],full_frames_and_state_equal:true,
   state_hash:fast.frames.at(-1).state_hash,input_receipts_equal:true,source_bytes_preserved:true});
 }
 report.verdict='PASS_TARGET_SCOPE';
}catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
finally{vm?.free();fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({out,rows:report.rows.length,verdict:report.verdict,error:report.error}));}
