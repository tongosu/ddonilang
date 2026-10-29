// Actual WASM snapshot ingress: malformed values cannot borrow legitimate
// false/empty defaults to pass the complete-world integrity comparison.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

const dir=process.env.DDN_WASM_DIR;
assert.ok(dir,'DDN_WASM_DIR must identify source-built product WASM');
const binary=fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(dir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:binary});
const source='';
const vm=new wasm.DdnWasmVm('');
const rows=[];
try {
  vm.update_logic_with_source_profile(source,'v1-core-v25');vm.reset(false);
  // These are valid inputs to the existing typed snapshot frontdoor, not
  // substituted DDN or a JS-computed world/hash. Rust admits the control,
  // computes its actual world hash and emits the snapshot we mutate below.
  const control=JSON.parse(vm.get_state_json());
  control.state.snapshot_v2.resources.value_det={
    진리:{type:'bool',value:false},글:{type:'string',value:''},목록:{type:'list',items:[]},
    모음:{type:'set',items:[]},짝:{type:'map',entries:[]},
  };
  vm.restore_state(JSON.stringify(control));
  const original=String(vm.get_state_json()),hash=vm.get_state_hash();
  const saved=JSON.parse(original);
  const values=saved.state.snapshot_v2.resources.value_det;
  assert.equal(values.진리.type,'bool');assert.equal(values.진리.value,false);
  assert.equal(values.글.type,'string');assert.equal(values.글.value,'');
  assert.equal(values.목록.type,'list');assert.deepEqual(values.목록.items,[]);
  for(const [name,field] of [['진리','value'],['글','value'],['목록','items'],['모음','items'],['짝','entries']]) {
    for(const mode of ['missing','wrong_type']) {
      const corrupt=structuredClone(saved);
      let touched=0;
      function mutate(node) {
        if(!node||typeof node!=='object')return;
        if(node.value_det?.[name]) {
          if(mode==='missing')delete node.value_det[name][field];
          else node.value_det[name][field]=null;
          touched++;
        }
        for(const child of Object.values(node))mutate(child);
      }
      mutate(corrupt);assert.ok(touched>0);
      assert.throws(()=>vm.restore_state(JSON.stringify(corrupt)),undefined,`${name}/${mode} must reject`);
      assert.equal(vm.get_state_hash(),hash,'failed restore world hash unchanged');
      assert.equal(String(vm.get_state_json()),original,'failed restore whole VM unchanged');
      rows.push({name,mode,status:'EXPLICIT_REJECT_OLD_COMPLETE'});
    }
  }
  vm.restore_state(original);assert.equal(vm.get_state_hash(),hash);
  console.log(JSON.stringify({status:'PASS',actual_wasm:true,wasm_sha256:createHash('sha256').update(binary).digest('hex'),source_sha256:createHash('sha256').update(source).digest('hex'),rows}));
} finally {vm.free();}
