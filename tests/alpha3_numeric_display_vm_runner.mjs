// A3-P1: real VM snapshot restore is atomic and presentation follows world rows.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
assert.ok(process.env.DDN_WASM_DIR,'actual source-built WASM required');
const dir=process.env.DDN_WASM_DIR;
const wasm=await import(pathToFileURL(path.join(dir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'))});
const json=value=>typeof value==='string'?JSON.parse(value):value;
const vm=new wasm.DdnWasmVm('');
try{
  const source='(매마디)마다 { (20 - 9.8 * 0.1) 보여주기. "19.019999999" 보여주기. }.';
  vm.update_logic_with_source_profile(source,'v1-core-v25');vm.reset(false);
  for(let madi=0;madi<3;madi++)vm.step_one();
  const saved=json(vm.get_state_json()), hash=vm.get_state_hash();
  assert.deepEqual(saved.output_display.texts,['19.02','19.019999999','19.02','19.019999999','19.02','19.019999999']);
  vm.reset(false);vm.restore_state(JSON.stringify(saved));
  assert.equal(vm.get_state_hash(),hash);
  assert.deepEqual(json(vm.get_state_json()).output_display,saved.output_display);
  const corrupt=structuredClone(saved);corrupt.output_display.world_state_hash='other-world';
  assert.throws(()=>vm.restore_state(JSON.stringify(corrupt)),/E_OUTPUT_DISPLAY_SNAPSHOT/);
  assert.equal(vm.get_state_hash(),hash);
  assert.deepEqual(json(vm.get_state_json()).output_display,saved.output_display);
  vm.step_one();
  assert.equal(json(vm.get_state_json()).output_display.texts.length,8);
  // The existing run_ticks finish path is separate from exact step_one.
  // Retain its world behavior, but every emitted typed value must be projected.
  vm.run_ticks(1);
  const completed=json(vm.get_state_json());
  assert.equal(completed.output_display.texts.length,completed.state.resources.value_json.output_log.length);
  for(let i=0;i<completed.output_display.texts.length;i+=2)
    assert.deepEqual(completed.output_display.texts.slice(i,i+2),['19.02','19.019999999']);
  vm.reset(false);
  assert.equal(json(vm.get_state_json()).output_display,undefined);
  console.log(JSON.stringify({status:'PASS',decision:'A3-P1 PROVISIONAL',snapshot_hash:hash,
    restore_projection:true,failed_restore_atomic:true,reset:true,resumed_madi:true}));
}finally{vm.free();}
