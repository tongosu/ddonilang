import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';

// Actual product source ingress; no lowering or replacement execution.
assert.ok(process.env.DDN_NATIVE_DIR);
assert.ok(process.env.DDN_WASM_DIR);
const wasmDir = path.resolve(process.env.DDN_WASM_DIR);
const bytes = fs.readFileSync(path.join(wasmDir, 'ddonirang_tool_bg.wasm'));
const wasm = await import(pathToFileURL(path.join(wasmDir, 'ddonirang_tool.js')).href);
await wasm.default({module_or_path: bytes});
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'ddn-alpha2-diagnostics-'));
const primary = '기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸.\n';
const cases = [
  ['primary-count', primary+'짜임 := (관계들=차림[기본,기본]) 표준보개.짜임.', 'E_BOGAE_PRIMARY_COUNT'],
  ['overlay-basis', '곁 := (보기=(() 표준보기.결과표), 자리=#오른쪽) 표준보개.곁.\n겹 := (보기=(() 표준보기.결과표), 기준=곁, 순서=1) 표준보개.겹.', 'E_BOGAE_OVERLAY_BASIS_INVALID'],
  ['wrong-relation', primary+'짜임 := (관계들=차림["으뜸"]) 표준보개.짜임.', 'E_BOGAE_UNSUPPORTED'],
  ['radius', '원 := (중심=차림[0,0], 반지름=0) 표준장면.원.', 'E_SCENE_RADIUS'],
  ['coordinate', '원 := (중심=차림[0,0,0], 반지름=2) 표준장면.원.', 'E_SCENE_COORDINATE'],
  ['bounds', '원 := (중심=차림[0,0], 반지름=2) 표준장면.원.\n장면 := (요소들=차림[원], 가로범위=차림[5,-5], 세로범위=차림[-5,5]) 표준장면.평면.', 'E_SCENE_BOUNDS'],
  ['recipe-result', '(대상:수) 확대:수:셈씨 := { 확대 <- 대상. }.\n자세히 := (대상=2, 보기법=확대) 표준보개.돋.', 'E_BOGAE_RECIPE_TYPE'],
  ['recipe-effect', '(대상:수) 확대:보기값:셈씨 := { 대상 보여주기. 확대 <- () 표준보기.결과표. }.\n자세히 := (대상=2, 보기법=확대) 표준보개.돋.', 'E_BOGAE_RECIPE_UNSEALED'],
  ['recipe-not-callable', '자세히 := (대상=2, 보기법=(() 표준보기.결과표)) 표준보개.돋.', 'E_BOGAE_RECIPE_UNSEALED'],
];
const rows=[];
const unsupportedRows=[];
for (const [id,call] of [
  ['fixed-positional','(2) 표준시간.고정마디'],
  ['relative-positional','(-1) 표준시간.상대마디'],
  ['fixed-named','(마디=2) 표준시간.고정마디'],
  ['relative-named','(차이=-1) 표준시간.상대마디'],
]) {
  const source='\n\n시간 := '+call+'.';
  const file=path.join(directory,id+'.ddn');
  fs.writeFileSync(file,source,'utf8');
  let failure;
  assert.throws(()=>wasm.wasm_run_summary_with_source_profile_json(source,file,1,'v1-core-v25'),
    error=>{failure=String(error);return failure.includes('E_BOGAE_TIME_POLICY_UNSUPPORTED')&&failure.includes('알파3');});
  assert.match(failure,/3:\d+/);
  assert.equal(JSON.parse(failure).code,'E_BOGAE_TIME_POLICY_UNSUPPORTED');
  const analysis=JSON.parse(wasm.wasm_source_analysis_snapshot_json(JSON.stringify({
    source_text:source,path:file,analysis_generation:0,source_profile_identity:'v1-core-v25'})));
  assert.equal(analysis.outcome,'REJECTED');
  assert.equal(analysis.diagnostics[0].code,'E_BOGAE_TIME_POLICY_UNSUPPORTED');
  assert.ok(Buffer.from(source).subarray(analysis.diagnostics[0].span.start).toString('utf8').startsWith('표준시간'));
  const native=[];
  for(const alias of ['ddonilang','ddonirang','또니랑']) {
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,alias+'.exe'),
      ['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],
      {encoding:'utf8',windowsHide:true,timeout:30000});
    assert.ifError(run.error);assert.notEqual(run.status,0);
    assert.match(run.stdout+'\n'+run.stderr,/E_BOGAE_TIME_POLICY_UNSUPPORTED/);
    assert.ok((run.stdout+'\n'+run.stderr).includes('알파3'));
    const snapshot=JSON.parse(run.stderr);
    assert.equal(snapshot.outcome,'REJECTED');
    assert.equal(snapshot.diagnostics[0].code,'E_BOGAE_TIME_POLICY_UNSUPPORTED');
    assert.ok(Buffer.from(source).subarray(snapshot.diagnostics[0].span.start).toString('utf8').startsWith('표준시간'));
    assert.deepEqual(snapshot,analysis,`${alias}/${id}: actual source-analysis snapshot parity`);
    native.push({alias,status:run.status});
  }
  unsupportedRows.push({id,native,wasm_message:failure});
}
for (const [id, body, code] of cases) {
  const source='\n\n'+body;
  const file=path.join(directory, id+'.ddn');
  fs.writeFileSync(file, source, 'utf8');
  let failure;
  assert.throws(() => wasm.wasm_run_summary_with_source_profile_json(source,file,1,'v1-core-v25'), e => {
    failure=String(e); return failure.includes(code);
  }, id);
  const location=failure.match(/\b(\d+):(\d+)\b/);
  assert.ok(location, `${id}: missing WASM source position: ${failure}`);
  const expectedLine=['radius','coordinate','recipe-not-callable'].includes(id)?3:4;
  assert.equal(Number(location[1]),expectedLine,`${id}: authored line independently counted`);
  const structured=JSON.parse(failure);
  assert.equal(structured.source_path,file);
  assert.ok(structured.source_span && structured.source_span.end>structured.source_span.start);
  const call=Buffer.from(source).subarray(structured.source_span.start,structured.source_span.end).toString('utf8');
  assert.match(call,/표준(?:보개|장면)\./,`${id}: span must select the actual authored constructor`);
  const native=[];
  for (const alias of ['ddonilang','ddonirang','또니랑']) {
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,alias+'.exe'),
      ['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],
      {encoding:'utf8',windowsHide:true,timeout:30000});
    assert.ifError(run.error); assert.notEqual(run.status,0,id);
    const diagnostic=run.stdout+'\n'+run.stderr;
    assert.ok(diagnostic.includes(code), diagnostic);
    assert.ok(diagnostic.includes(location[0]), `${alias}/${id}: location differs: ${diagnostic}`);
    native.push({alias,status:run.status});
  }
  rows.push({id,source_sha256:createHash('sha256').update(source).digest('hex'),code,location:location[0],wasm_message:failure,native});
}
const report={result:'ALPHA2_DIAGNOSTIC_PARITY_PASS',cases:rows.length,native_aliases:3,
  wasm_sha256:createHash('sha256').update(bytes).digest('hex'),rows,unsupported_time_policies:unsupportedRows};
fs.writeFileSync(path.join(directory,'report.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify({...report,evidence_directory:directory}));
