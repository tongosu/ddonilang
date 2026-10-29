// Same packaged raw Tetris and input frames as the Native CLI3 cold suite.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {pathToFileURL,fileURLToPath} from 'node:url';import {createHash} from 'node:crypto';import {spawnSync} from 'node:child_process';
const [stage,nativeOut,out,mode='produce']=process.argv.slice(2);
assert(stage&&nativeOut&&out);
const sha=x=>createHash('sha256').update(x).digest('hex');
const wasmDir=path.join(stage,'solutions/seamgrim_ui_mvp/ui/wasm');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const sample=path.join(stage,'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris');
const authored=fs.readFileSync(path.join(sample,'authored-project.json'),'utf8');
const first=JSON.parse(fs.readFileSync(path.join(sample,'initial-input.json'),'utf8'));
const after=JSON.parse(fs.readFileSync(path.join(sample,'next-input.json'),'utf8'));
const binding=JSON.stringify(first.binding);
let vm;
if(mode==='cold'||mode==='cold-paced'){
 // Pair the actual local checkpoint with its own admitted source/PESI.
 // A Native durable snapshot is not this local producer's attachment.
 const saved=fs.readFileSync(path.join(out,'WASM_SAVED3.json'),'utf8');
 vm=wasm.DdnLogicalInputVm.from_saved_project_checkpoint(saved,fs.readFileSync(path.join(out,'checkpoint.bin')),new Uint8Array(),new Uint8Array([2]),binding);
 for(const input of after.snapshots){if(mode==='cold-paced')await new Promise(r=>setTimeout(r,100));vm.step_json(JSON.stringify(input));}
 const summary=JSON.parse(vm.summary_json());vm.free();
 const expected=JSON.parse(fs.readFileSync(path.join(nativeOut,'ddonilang/cold-resume4.stdout'),'utf8'));
 assert.deepEqual(summary,expected,'actual cold WASM/Native complete logical summary');
 fs.writeFileSync(path.join(out,mode==='cold-paced'?'WASM_COLD_PACED.json':'WASM_COLD.json'),JSON.stringify(summary));
 console.log(JSON.stringify({verdict:'PASS_COLD_NODE_PROCESS',state_hash:summary.state_hash}));
}else{
 assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
 const report={scope:'PACKAGED_TETRIS_RAW_CLI3_NATIVE_AND_ACTUAL_WASM_FRESH_PROCESS_NOT_BROWSER_OR_ROW',verdict:'NOT_RUN'};
 try{
   const snapshots=[...first.snapshots,...after.snapshots];
   const full=JSON.stringify({binding:first.binding,snapshots});
   const recorded=JSON.parse(wasm.authored_logical_input_record_v3(authored,full));
   const nativeSummary=JSON.parse(fs.readFileSync(path.join(nativeOut,'ddonilang/continuous7.stdout'),'utf8')).summary;
   assert.deepEqual(recorded.summary,nativeSummary);
   assert.deepEqual(Buffer.from(recorded.tape_bytes),fs.readFileSync(path.join(nativeOut,'ddonilang/finite-v3.bin')));
   const replay=JSON.parse(wasm.authored_logical_input_replay_v3(authored,Uint8Array.from(recorded.tape_bytes),binding));
   assert.deepEqual(replay,recorded.summary);
   vm=wasm.DdnLogicalInputVm.from_authored_project(authored,binding);
   for(const input of first.snapshots)vm.step_json(JSON.stringify(input));
   const artifact=vm.export_saved_project(new Uint8Array(),new Uint8Array([2]));
   const checkpoint=vm.export_project_checkpoint(new Uint8Array(),new Uint8Array([2]));
   fs.writeFileSync(path.join(out,'WASM_SAVED3.json'),artifact);fs.writeFileSync(path.join(out,'checkpoint.bin'),checkpoint);
   for(const input of after.snapshots)vm.step_json(JSON.stringify(input));
   assert.deepEqual(JSON.parse(vm.summary_json()),recorded.summary);vm.free();vm=null;
   const child=spawnSync(process.execPath,[fileURLToPath(import.meta.url),stage,nativeOut,out,'cold'],
     {encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:16*1024*1024});
   fs.writeFileSync(path.join(out,'cold.stdout'),child.stdout??'');fs.writeFileSync(path.join(out,'cold.stderr'),child.stderr??'');
   assert.ifError(child.error);assert.equal(child.status,0,child.stderr);
   const paced=spawnSync(process.execPath,[fileURLToPath(import.meta.url),stage,nativeOut,out,'cold-paced'],
     {encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:16*1024*1024});
   fs.writeFileSync(path.join(out,'cold-paced.stdout'),paced.stdout??'');fs.writeFileSync(path.join(out,'cold-paced.stderr'),paced.stderr??'');
   assert.ifError(paced.error);assert.equal(paced.status,0,paced.stderr);
   assert.deepEqual(JSON.parse(fs.readFileSync(path.join(out,'WASM_COLD_PACED.json'),'utf8')),
     JSON.parse(fs.readFileSync(path.join(out,'WASM_COLD.json'),'utf8')));
   Object.assign(report,{verdict:'PASS_TARGET_SCOPE',state_hash:recorded.summary.state_hash,
     raw_artifact_sha256:sha(authored),tape_sha256:sha(Buffer.from(recorded.tape_bytes)),
     continuous_repeat_cold_full_summary_equal:true,new_node_process:true,
     snapshot_producer:'WASM_LOCAL_CHECKPOINT',native_durable_bytes_restored_in_wasm:false,
     native_durable_receipt_parity:'NOT_RUN',paced_cold_full_summary_equal:true,host_delay_per_madi_ms:100});
 }catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
 finally{vm?.free();fs.writeFileSync(path.join(out,'RESULT.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));}
}
