import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const directory=path.resolve(process.env.DDN_WASM_DIR);
const wasm=await import(pathToFileURL(path.join(directory,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(directory,'ddonirang_tool_bg.wasm'))});
const invalid='값 <- 9. 첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표. (측정: 9) 보임.';
const vm=new wasm.DdnWasmVm('');
vm.update_logic_with_source_profile(invalid,'v1-core-v25');
try {
  const before=JSON.parse(vm.get_state_json());
  for(let attempt=0;attempt<2;attempt++) {
    assert.throws(()=>vm.step_one(),/E_BOGAE_SELECTION_REQUIRED/);
    const after=JSON.parse(vm.get_state_json());
    assert.deepEqual(after,before,'failed activation must expose no patch, tick, hash, observation or projection');
  }
} finally {vm.free();}
assert.throws(()=>wasm.wasm_run_summary_with_source_profile_json(invalid,'atomic.ddn',1,'v1-core-v25'),/E_BOGAE_SELECTION_REQUIRED/);
console.log(JSON.stringify({result:'PASS',checks:3,scope:'actual WASM activation atomicity; pure constructor tests retained'}));
