import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';

// WO-06 observation only. Acceptance never establishes semantic authority.
for (const name of ['DDN_NATIVE_DIR', 'DDN_WASM_DIR']) {
  if (!process.env[name]) throw new Error(`${name} is required`);
}
const wasmDir = path.resolve(process.env.DDN_WASM_DIR);
const bytes = fs.readFileSync(path.join(wasmDir, 'ddonirang_tool_bg.wasm'));
const wasm = await import(pathToFileURL(path.join(wasmDir, 'ddonirang_tool.js')).href);
await wasm.default({module_or_path:bytes});
const cases = [
  ['top-level-value', '값 := 1. 값 <- 2. 값 보여주기.'],
  ['local-value', '검산:움직씨 := { 값 := 1. 값 <- 2. 값 보여주기. }. () 검산.'],
  ['madi-value', '(매마디)마다 { 값 := 1. 값 <- 2. 값 보여주기. }.'],
  ['callable-name', '(값:수) 더함:수:셈씨 := { 더함 <- 값 + 1. }. 더함 <- 3. 더함 보여주기.'],
  ['parameter-binding', '(값:수) 더함:수:셈씨 := { 값 <- 2. 더함 <- 값. }. (값=1) 더함 보여주기.'],
  ['iteration-binding', '목록 <- 차림[1, 2]. 목록마다 { 목록 <- 9. 목록 보여주기. }.'],
  ['callable-result-place', '(값:수) 더함:수:셈씨 := { 더함 <- 값 + 1. 더함 <- 값 + 2. }. (값=1) 더함 보여주기.'],
  // Literal source projection from SSOT_LANG RD-346~357, not an invented type grammar.
  ['product-type-name', '(가로: 셈수, 세로: 셈수) 좌표:묶음씨 := {}. 좌표 <- 3.'],
  ['sum-type-name', '계산결과:고름씨 := { $성공(값: 셈수). $실패(이유: 글). }. 계산결과 <- 3.'],
];
const folder = fs.mkdtempSync(path.join(os.tmpdir(), 'ddn-wo06-observation-'));
for (const [id, source] of cases) {
  const file = path.join(folder, `${id}.ddn`);
  fs.writeFileSync(file, source, 'utf8');
  let summary, failure;
  try { summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, file, 1, 'v1-core-v25')); }
  catch (error) { failure = String(error); }
  const native = ['ddonirang', 'ddonilang', '또니랑'].map(alias => {
    const result = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
      ['실행', file, '--source-profile', 'v1-core-v25', '--마디수', '1', '--json'],
      {encoding:'utf8', windowsHide:true, timeout:20000});
    if (result.error) throw result.error;
    if (result.status !== 0) return {alias, accepted:false, diagnostic:result.stdout + result.stderr};
    const value = JSON.parse(result.stdout);
    return {alias, accepted:true, output:(value.resources.value_json.output_log ?? []).map(row => row.text), state_hash:value.state_hash};
  });
  console.log(JSON.stringify({id, source, source_sha256:createHash('sha256').update(source).digest('hex'),
    wasm:{accepted:summary !== undefined, output:summary?.output_log_texts, state_hash:summary?.state_hash, diagnostic:failure}, native,
    authority:'OBSERVATION_ONLY_NOT_SEMANTIC_APPROVAL'}));
}
const modules = '모듬=차림[(이름=앱,머리="main.ddn",원문="main.ddn"),(이름=힘,머리="head.ddn",원문="head.ddn")]';
for (const [id, identity] of [['module-export-local-project', ''], ['public-gaji-project', '가지="검산.가지",판본="0.1.0",내놓음=차림[힘],']]) {
  const source = {schema:'ddn.virtual_project_source.v1', kind:'seamgrim.local_project_tree.v0',
    root_path:'chosen-root.ddn', entry_path:'main.ddn', files:[
      {path:'chosen-root.ddn',role:'root',utf8_text:`(${identity}글판="v25",${modules},시작=(모듈=앱,정의=실행)) 얽음.`},
      {path:'main.ddn',role:'ddn_source',utf8_text:'(힘=힘) 쓰임. 실행:움직씨 := { 힘.중력 보여주기. }.'},
      {path:'head.ddn',role:'ddn_source',utf8_text:'(중력=중력) 드러냄. 중력:수 := 1. 중력 <- 2.'},
    ]};
  const text = JSON.stringify(source);
  const file = path.join(folder, `${id}.json`);
  fs.writeFileSync(file, text, 'utf8');
  let wasmResult;
  try { wasmResult = {accepted:true, summary:JSON.parse(wasm.wasm_project_run_summary_json(text, '', false, false))}; }
  catch (error) { wasmResult = {accepted:false, diagnostic:String(error)}; }
  const native = ['ddonirang', 'ddonilang', '또니랑'].map(alias => {
    const result = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
      ['project-run', '--virtual', file],
      {encoding:'utf8', windowsHide:true, timeout:20000});
    if (result.error) throw result.error;
    return {alias, accepted:result.status === 0, stdout:result.stdout, stderr:result.stderr};
  });
  console.log(JSON.stringify({id, source_sha256:createHash('sha256').update(text).digest('hex'),
    source, wasm:wasmResult, native, authority:'OBSERVATION_ONLY_NOT_SEMANTIC_APPROVAL'}));
}
console.log(JSON.stringify({standalone_cases:cases.length, project_cases:2, wasm_sha256:createHash('sha256').update(bytes).digest('hex'),
  source_directory:folder, result:'WO06_OBSERVATIONS_RECORDED_NOT_A_CLOSURE_VERDICT'}));
