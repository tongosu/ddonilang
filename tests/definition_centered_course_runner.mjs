import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {spawnSync} from 'node:child_process';

const root = path.resolve(process.env.DDN_TEST_ROOT || path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'));
const dir = path.resolve(process.env.DDN_COURSE_DIR || path.join(root, 'solutions/seamgrim_ui_mvp/first_steps'));
const wasmDir = path.resolve(process.env.DDN_TEST_WASM_DIR || process.env.DDN_WASM_DIR || path.join(root, 'solutions/seamgrim_ui_mvp/ui/wasm'));
const nativeDir = process.env.DDN_TEST_NATIVE_DIR || process.env.DDN_NATIVE_DIR;
assert.ok(nativeDir, 'DDN_NATIVE_DIR or DDN_TEST_NATIVE_DIR must identify actual native binaries');
const sha=value=>createHash('sha256').update(value).digest('hex');
const results=[];
const manifest = JSON.parse(fs.readFileSync(path.join(dir, '검증목록.json'), 'utf8'));
assert.equal(manifest.cases.length, 40);
assert.equal(new Set(manifest.cases.map(row => row.file)).size, 40);
let normal = 0, rejected = 0, corrected = 0, modified = 0, supplements = 0;
const supplementCases = [
  {id:'SUP06',file:'보충/06_마지막에배우는축약.ddn',expected:['14','14']},
  {id:'SUP09',file:'보충/09_축약대조.ddn',expected:['18','18']},
];
const allCases = [...manifest.cases, ...manifest.corrections, ...manifest.modifications, ...supplementCases];
const shippedSources = fs.readdirSync(dir, {recursive:true}).filter(file => file.endsWith('.ddn')).map(file => file.replaceAll('\\', '/')).sort();
assert.deepEqual(allCases.map(row => row.file).sort(), shippedSources, '원문57개 모두 검사하며 누락·추가 없음');
assert.equal(manifest.corrections.length, 5);
assert.equal(manifest.modifications.length, 10);
// An explicit separate gate while a new WASM is pending. It cannot produce
// the CLI/WASM verdict below or silently fall back to an older WASM file.
if(process.env.DDN_NATIVE_ONLY==='1') {
  assert(process.env.DDN_NATIVE_COURSE_REPORT,'a fresh native-only evidence path is required');
  const rows=[];
  for(const item of allCases){
    const file=path.join(dir,item.file),sourceBytes=fs.readFileSync(file),reject=!Array.isArray(item.expected);
    let baseline=null;const aliases=[];
    for(const name of ['ddonilang','ddonirang','또니랑']){
      const exe=path.join(nativeDir,name+'.exe');
      const result=spawnSync(exe,['실행',file,'--source-profile',manifest.profile,'--마디수','1','--json'],{encoding:'utf8',windowsHide:true,timeout:20000,maxBuffer:32e6});
      assert.ifError(result.error);
      if(reject){assert.notEqual(result.status,0,item.id);assert((result.stderr+result.stdout).includes(item.expected.reject),item.id+': '+result.stderr);}
      else {
        assert.equal(result.status,0,item.id+': '+result.stderr);const actual=JSON.parse(result.stdout);
        assert.deepEqual(actual.output_log_texts,item.expected,item.id);
        if(baseline)for(const key of ['state_hash','resources','bogae_projection','work_cost_receipt','output_log_texts'])assert.deepEqual(actual[key],baseline[key],item.id+': '+key);
        else baseline=actual;
      }
      aliases.push({name,exit:result.status,exe_sha256:sha(fs.readFileSync(exe)),diagnostic:reject?result.stderr:null});
    }
    rows.push({id:item.id,raw_source_sha256:sha(sourceBytes),expected:item.expected,state_hash:baseline?.state_hash??null,aliases});
  }
  const report={scope:'ACTUAL_RELEASE_CLI3_ONLY_WASM_AND_BROWSER_NOT_RUN',cases:rows.length,normal:rows.filter(row=>Array.isArray(row.expected)).length,rejected:rows.filter(row=>!Array.isArray(row.expected)).length,rows};
  fs.writeFileSync(process.env.DDN_NATIVE_COURSE_REPORT,JSON.stringify(report,null,2),{flag:'wx'});
  console.log(JSON.stringify({scope:report.scope,cases:report.cases,normal:report.normal,rejected:report.rejected,report:process.env.DDN_NATIVE_COURSE_REPORT}));
  process.exit(0);
}
const wasm = await import(pathToFileURL(path.join(wasmDir, 'ddonirang_tool.js')).href);
const wasmBytes = fs.readFileSync(path.join(wasmDir, 'ddonirang_tool_bg.wasm'));
await wasm.default({module_or_path: wasmBytes});
for (const c of allCases) {
  const sourceFile = path.join(dir, c.file);
  const bytes = fs.readFileSync(sourceFile);
  assert.notEqual(bytes.subarray(0, 3).toString('hex'), 'efbbbf', `${c.id}: BOM`);
  const source = bytes.toString('utf8');
  assert.ok(!/(두배|삼각형넓이|큰값|합산|차림합계|요금계산|할인적용|피해|남은체력|문판정|상품금액|값보탬)하기/.test(source), c.id + ': 셈씨 결과 이름에 억지 하기 금지');
  assert.ok(!/(?:을|를|과|와)\s+(?:두배|차림합계|누적합|삼각형넓이|상품금액|구매진행하기)(?=[ .])/.test(source), c.id + ': 역할과 맞지 않는 조사 회귀 금지');
  assert.ok(!/\([^()\n=]+\) (삼각형넓이|큰값|피해|남은체력|요금계산|할인적용)(?=[ .])/.test(source),
    c.id + ': 역할이 다른 인자는 이름 핀으로 명시');
  if (!c.id.startsWith('SUP')) assert.ok(!source.includes('-->'), `${c.id}: shortcut belongs in supplement only`);
  if (Number(c.id.slice(0, 2)) >= 3) {
    assert.ok(source.includes('실행:움직씨 :=') && source.includes('() 실행하기.'), `${c.id}: missing authored entry action`);
    assert.ok(!source.includes('매마디'), `${c.id}: static course must not require frame hook`);
  }
  let summary, failure;
  try { summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, sourceFile, 1, manifest.profile)); }
  catch (error) { failure = String(error); }
  const reject = !Array.isArray(c.expected);
  const aliases=[];
  if (reject) {
    assert.ok(failure?.includes(c.expected.reject), `${c.id}: expected rejection ${c.expected.reject}, got ${failure}`);
    assert.equal(summary, undefined, `${c.id}: failure published summary`);
    rejected++;
  } else {
    assert.equal(failure, undefined, `${c.id}: ${failure}`);
    assert.deepEqual(summary.output_log_texts, c.expected, `${c.id}: independent expectation`);
    if (c.id.startsWith('SUP')) supplements++;
    else if (c.id.startsWith('MOD')) modified++;
    else if (c.id.startsWith('FIX')) corrected++;
    else normal++;
  }
  for (const alias of ['ddonilang', 'ddonirang', '또니랑']) {
    const run = spawnSync(path.join(nativeDir, `${alias}.exe`),
      ['실행', sourceFile, '--source-profile', manifest.profile, '--마디수', '1', '--json'],
      {encoding:'utf8', windowsHide:true, timeout:20000});
    assert.ifError(run.error);
    if (reject) {
      assert.notEqual(run.status, 0, `${alias}/${c.id}: invalid example accepted`);
      assert.ok((run.stderr + run.stdout).includes(c.expected.reject), `${alias}/${c.id}: ${run.stderr}`);
      aliases.push({alias,status:'REJECTED',diagnostic:run.stderr+run.stdout});
    } else {
      assert.equal(run.status, 0, `${alias}/${c.id}: ${run.stderr}`);
      const native = JSON.parse(run.stdout);
      const log = native.resources.value_json.output_log;
      if (c.expected.length === 0) assert.ok(log === undefined || (Array.isArray(log) && log.length === 0));
      else {
        assert.ok(Array.isArray(log), `${alias}/${c.id}: missing output`);
        assert.deepEqual(log.map(row => row.text), c.expected, `${alias}/${c.id}`);
      }
      assert.equal(native.state_hash, summary.state_hash, `${alias}/${c.id}: world hash`);
      for(const key of ['resources','bogae_projection','work_cost_receipt','output_log_texts'])assert.deepEqual(native[key],summary[key],`${alias}/${c.id}: ${key}`);
      aliases.push({alias,status:'PASS',state_hash:native.state_hash,resources_sha256:sha(JSON.stringify(native.resources)),work_cost_receipt_sha256:sha(JSON.stringify(native.work_cost_receipt??null))});
    }
  }
  const result={id:c.id,file:c.file,source_sha256:sha(bytes),expected:c.expected,reason:c.reason??'기존 보충 원문 기대값',state_hash:summary?.state_hash??null,wasm_diagnostic:failure??null,native_aliases:aliases,result:'PASS'};
  results.push(result);
  console.log(JSON.stringify({id:result.id,source_sha256:result.source_sha256,expected:result.expected,state_hash:result.state_hash,result:result.result}));
}
assert.equal(normal, 35);
assert.equal(rejected, 5);
assert.equal(corrected, 5);
assert.equal(modified, 10);
assert.equal(supplements, 2);
console.log(`DEFINITION_CENTERED_COURSE_PASS total=57 lessons=10 activities=40 normal=${normal} rejected=${rejected} corrections=${corrected} modifications=${modified} supplements=${supplements} cli_aliases=3 actual_wasm=1 world_hashes=52 wasm_sha256=${createHash('sha256').update(wasmBytes).digest('hex')}`);
const evidence=process.env.DDN_TEST_EVIDENCE_DIR?path.resolve(process.env.DDN_TEST_EVIDENCE_DIR):fs.mkdtempSync(path.join(os.tmpdir(),'ddn-first-steps-native-wasm-'));
fs.mkdirSync(evidence,{recursive:true});
const report={status:'PASS',target_root:root,course_directory:dir,profile:manifest.profile,manifest_sha256:sha(fs.readFileSync(path.join(dir,'검증목록.json'))),
  verifier_sha256:sha(fs.readFileSync(fileURLToPath(import.meta.url))),wasm_sha256:sha(wasmBytes),native_directory:path.resolve(nativeDir),
  native_binary_sha256:Object.fromEntries(['ddonilang','ddonirang','또니랑'].map(alias=>[alias,sha(fs.readFileSync(path.join(nativeDir,alias+'.exe')))])),
  counts:{total:57,normal:52,rejected,activities:40,corrections:corrected,modifications:modified,supplements},results};
const reportPath=path.join(evidence,'report.json'),reportText=JSON.stringify(report,null,2)+'\n';fs.writeFileSync(reportPath,reportText,'utf8');
console.log(JSON.stringify({status:'PASS',report:reportPath,report_sha256:sha(reportText),verifier_sha256:report.verifier_sha256,wasm_sha256:report.wasm_sha256,counts:report.counts}));
