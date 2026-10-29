import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {execFileSync} from 'node:child_process';

const root = process.cwd();
const modulePath = 'solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js';
const binaryPath = 'solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm';
const file = path.join(root, 'tests/wo09_fixed64_display.ddn');
const source = fs.readFileSync(file, 'utf8');
const current = await import(pathToFileURL(path.join(root,modulePath)).href+'?wo09=current');
const baseline = await import(pathToFileURL(path.join(root,modulePath)).href+'?wo09=baseline');
await current.default({module_or_path:fs.readFileSync(path.join(root,binaryPath))});
await baseline.default({module_or_path:execFileSync('git',['show',`d0162ea5:${binaryPath}`],{maxBuffer:64*1024*1024})});
const args = [source,file,1,'v1-core-v25'];
const before = JSON.parse(baseline.wasm_run_summary_with_source_profile_json(...args));
const after = JSON.parse(current.wasm_run_summary_with_source_profile_json(...args));
const cli = process.env.DDN_CLI;
assert(cli,'DDN_CLI must identify the freshly built official CLI');
const native = JSON.parse(execFileSync(cli,['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],{encoding:'utf8'}));
for (const [key,value] of Object.entries({a:'-0.08',b:'-1.5',c:'-2',d:'0.25'})) {
  assert.equal(after.resources.fixed64[key],value);
  assert.equal(native.resources.fixed64[key],value);
}
assert.equal(before.resources.fixed64.a,'-1.919999');
assert.equal(before.state_hash,after.state_hash,'display must not change world identity');
assert.equal(native.state_hash,after.state_hash);
assert.deepEqual(before.final_row,after.final_row);
console.log(JSON.stringify({checks:4,cli_wasm_equal:true,baseline_world_hash_unchanged:true,state_hash:after.state_hash}));
