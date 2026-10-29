import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

assert.ok(process.env.DDN_NATIVE_DIR);
assert.ok(process.env.DDN_WASM_DIR);
const wasmDir = path.resolve(process.env.DDN_WASM_DIR);
const binary = fs.readFileSync(path.join(wasmDir, 'ddonirang_tool_bg.wasm'));
const wasm = await import(pathToFileURL(path.join(wasmDir, 'ddonirang_tool.js')).href);
await wasm.default({module_or_path: binary});
// The existing VM exposes its unstepped initial world. No harness world emulator.
const empty = new wasm.DdnWasmVm('');
const initialWorld = empty.get_state_hash();
empty.free();
const output = fs.mkdtempSync(path.join(os.tmpdir(), 'ddn-alpha2-logical-'));
let checks = 0;
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const source = `(매마디)마다 {
    계산값 <- 2.
    사건들 <- () 입력사상.사건들.
    사건들마다 {
        { 사건들.채널 == "더함" }이면 { 계산값 <- 계산값 + 1. }
        아니면 { 계산값 <- 계산값 * 2. }.
    }.
    같은사건들 <- () 입력사상.사건들.
    조작맵 <- () 입력사상.만들기.
    눌림값 <- (조작맵, "더함") 입력사상.동작.
}.`;
function requestFor(text, channels = {곱함: 'Control', 더함: 'Control'}) {
  return {binding: {
    source_profile: 'v1-core-v25-logical-alpha2',
    input_schema: 'alpha2.logical-test-schema@1',
    channel_registry: 'alpha2.logical-test-channels@1',
    source_revision: `sha256:${sha(text)}`, initial_world: initialWorld,
    initial_rng: 7, recording_policy: 'alpha2.logical-test-recording@1',
    integrity_policy: 'ddn.logical_input_tape.v1', madi_hz: 60,
    dt_raw: 4294967296, channels,
    initial_held: Object.fromEntries(Object.keys(channels).map(name => [name, false])),
  }, snapshots: []};
}
function native(alias, args, ok = true, code = '') {
  const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`), args,
    {encoding: 'utf8', timeout: 30000, maxBuffer: 8 * 1024 * 1024});
  assert.equal(run.error, undefined);
  if (ok) {
    assert.equal(run.status, 0, `${alias}: ${run.stderr}`);
    checks++;
    return JSON.parse(run.stdout);
  }
  assert.notEqual(run.status, 0);
  assert.ok(run.stderr.includes(code), run.stderr);
  assert.equal(run.stdout.trim(), '', 'rejected execution must not publish a summary');
  checks++;
}
function compare(actual, expected) {
  assert.deepEqual(actual, expected, 'same complete execution and input receipt');
  checks++;
}
const scenarios = [
  {name: 'no-input', text: '(매마디)마다 { 값 <- 2. }.', inputs: [{madi:0, events:[], held:{곱함:false, 더함:false}}]},
  {name: 'input-unread', text: '(매마디)마다 { 값 <- 2. }.', inputs: [{madi:0, events:[{ordinal:0,channel:'더함',event:{kind:'Press'}}],held:{곱함:false,더함:true}}]},
  {name: 'input-read', text: source, order:['더함','곱함'], expected:'6'},
  {name: 'input-reversed', text: source, order:['곱함','더함'], expected:'5'},
  {name: 'repeat-release', text: source, order:['더함','곱함'], expected:'6', next:true},
  // A3-P4: actual source frontdoor and unchanged logical record bytes.
  ...[false,true].map(reverse=>({name:`authored-reaction-${reverse?'reverse':'forward'}`,
    text:fs.readFileSync('samples/alpha3_logical_reaction.ddn','utf8'),
    order:reverse?['곱함','더함']:['더함','곱함'],reactionExpected:reverse?'5':'6'})),
  // A3-TIME-P1: use the actual authored source; preserve the existing tape.
  {name: 'historical-table', text: fs.readFileSync('solutions/seamgrim_ui_mvp/samples/alpha3_table_time.ddn','utf8'),
    inputs: Array.from({length:3},(_,madi)=>({madi,events:[],held:{곱함:false,더함:false}})), historicalTable:true},
  ...['scene_time','scene_relative'].map(kind=>({name:`historical-${kind}`,
    text:fs.readFileSync(`solutions/seamgrim_ui_mvp/samples/alpha3_${kind}.ddn`,'utf8'),
    inputs:Array.from({length:3},(_,madi)=>({madi,events:[],held:{곱함:false,더함:false}})),
    historicalScene:kind==='scene_relative'?1:0})),
];
for (const scenario of scenarios) {
  const request = requestFor(scenario.text);
  request.snapshots = scenario.inputs ?? [{madi:0,
    events:scenario.order.map((channel,ordinal) => ({ordinal,channel,event:{kind:'Press'}})),
    held:{곱함:true,더함:true}}];
  if (scenario.next) request.snapshots.push({madi:1, events:[
    {ordinal:0,channel:'더함',event:{kind:'Repeat'}},
    {ordinal:1,channel:'곱함',event:{kind:'Release'}},
  ],held:{곱함:false,더함:true}});
  const stem = path.join(output, scenario.name);
  fs.writeFileSync(`${stem}.ddn`, scenario.text);
  fs.writeFileSync(`${stem}.request.json`, JSON.stringify(request));
  fs.writeFileSync(`${stem}.binding.json`, JSON.stringify(request.binding));
  // Actual WASM-produced bytes are the same file consumed by all three CLIs.
  const recorded = JSON.parse(wasm.logical_input_record(scenario.text, JSON.stringify(request)));
  if(scenario.reactionExpected) {
    assert.equal(recorded.summary.resources.fixed64.계산값,scenario.reactionExpected);
    assert.equal(recorded.summary.resources.fixed64.마디결과,scenario.reactionExpected);
    assert.equal(recorded.summary.reaction_support,true);
    checks+=3;
  }
  if (scenario.historicalTable) {
    const relation=recorded.summary.bogae_projection.candidates[0].artifact.relations[1];
    assert.equal(relation.resolved_madi,0);
    const rows=relation.view.table_projection.tables[0].rows;
    assert.equal(rows.length,1,'fixed scene must not show current three-madi table');
    assert.equal(rows[0].cells[0].value_text,'1');
    checks+=3;
  }
  if (scenario.historicalScene!==undefined) {
    assert.equal(recorded.summary.bogae_projection.candidates[0].artifact.relations[1].resolved_madi,scenario.historicalScene);
    checks++;
  }
  const tape = Buffer.from(recorded.tape_bytes);
  const live = new wasm.DdnLogicalInputVm(scenario.text, JSON.stringify(request.binding));
  for (const input of request.snapshots) {
    const step = JSON.parse(live.step_json(JSON.stringify(input)));
    assert.deepEqual(step.frame, recorded.summary.frames[input.madi]);
    assert.deepEqual(step.input_receipt, recorded.summary.input_receipts[input.madi]);
    checks++;
  }
  compare(JSON.parse(live.summary_json()), recorded.summary);
  const liveRecord = JSON.parse(live.seal_json());
  assert.deepEqual(Buffer.from(liveRecord.tape_bytes), tape, 'incremental and batch records are identical');
  checks++;
  live.free();
  fs.writeFileSync(`${stem}.wasm.tape`, tape);
  if (scenario.expected) {
    assert.equal(recorded.summary.resources.fixed64.계산값, scenario.expected);
    assert.deepEqual(recorded.summary.resources.value_json.사건들,
      recorded.summary.resources.value_json.같은사건들, 'queries do not consume events');
    checks += 2;
  }
  compare(JSON.parse(wasm.logical_input_replay(scenario.text, tape, JSON.stringify(request.binding))), recorded.summary);
  for (const alias of ['ddonilang','ddonirang','또니랑']) {
    compare(native(alias, ['논리입력재생', `${stem}.ddn`, `${stem}.wasm.tape`, `${stem}.binding.json`]), recorded.summary);
    const cliSummary = native(alias, ['논리입력기록', `${stem}.ddn`, `${stem}.request.json`, `${stem}.${alias}.tape`]);
    const cliTape = fs.readFileSync(`${stem}.${alias}.tape`);
    assert.deepEqual(cliTape, tape, 'one shared canonical byte encoding');
    compare(JSON.parse(wasm.logical_input_replay(scenario.text, cliTape, JSON.stringify(request.binding))), cliSummary);
  }
}
// Explicit rejection, never physical-key/text/default-input fallback.
for (const [name,text,mutate,code] of [
  ['physical', '(매마디)마다 { 값 <- 입력키. }.', () => {}, 'E_LOGICAL_INPUT_PHYSICAL_UNSUPPORTED'],
  ['text', source, r => {r.binding.channels.글 = 'Text';}, 'E_LOGICAL_INPUT_TEXT_UNSUPPORTED'],
  ['wrong-dt', source, r => {r.binding.dt_raw = 67108864;}, 'E_LOGICAL_INPUT_PROFILE'],
  ['arithmetic', '(매마디)마다 { 값 <- 6. 값 <- 1 / 0. }.', () => {}, ''],
]) {
  const request = requestFor(text);
  request.snapshots = [{madi:0,events:[],held:{곱함:false,더함:false}}];
  mutate(request);
  assert.throws(() => wasm.logical_input_record(text, JSON.stringify(request)), error => String(error).includes(code));
  checks++;
  const stem = path.join(output, name);
  fs.writeFileSync(`${stem}.ddn`, text);
  fs.writeFileSync(`${stem}.json`, JSON.stringify(request));
  native('ddonilang', ['논리입력기록', `${stem}.ddn`, `${stem}.json`, `${stem}.tape`], false, code);
  assert.equal(fs.existsSync(`${stem}.tape`), false, 'failed madi must not create success tape');
}
const binding = JSON.parse(fs.readFileSync(path.join(output,'input-read.binding.json')));
const good = fs.readFileSync(path.join(output,'input-read.wasm.tape'));
const corrupt = Buffer.from(good); corrupt[40] ^= 1;
assert.throws(() => wasm.logical_input_replay(source, corrupt, JSON.stringify(binding)), /E_LOGICAL_INPUT_SEAL/);
checks++;
binding.initial_rng++;
assert.throws(() => wasm.logical_input_replay(source, good, JSON.stringify(binding)), /E_LOGICAL_INPUT_BINDING_MISMATCH/);
checks++;
const failSource = `(매마디)마다 {
  값 <- 6. 사건들 <- () 입력사상.사건들.
  사건들마다 { { 사건들.채널 == "곱함" }이면 { 값 <- 1 / 0. }. }.
}.`;
const liveRequest = requestFor(failSource);
const live = new wasm.DdnLogicalInputVm(failSource, JSON.stringify(liveRequest.binding));
live.step_json(JSON.stringify({madi:0,events:[{ordinal:0,channel:'더함',event:{kind:'Press'}}],held:{곱함:false,더함:true}}));
const before = JSON.parse(live.summary_json());
assert.throws(() => live.step_json(JSON.stringify({madi:1,events:[{ordinal:0,channel:'곱함',event:{kind:'Press'}}],held:{곱함:true,더함:true}})));
const after = JSON.parse(live.summary_json());
for (const key of ['state_hash','resources','frames','input_receipts','madi_count']) {
  assert.deepEqual(after[key], before[key], `failed live madi changed ${key}`);
  checks++;
}
assert.equal(after.outcome,'ABORTED');
assert.throws(() => live.seal_json(), /SESSION_ABORTED/);
assert.throws(() => live.step_json('{}'));
live.free();
checks += 3;
const malformed = new wasm.DdnLogicalInputVm(source, JSON.stringify(requestFor(source).binding));
assert.throws(() => malformed.step_json('{"madi":0}'), /E_LOGICAL_INPUT_SNAPSHOT/);
assert.throws(() => malformed.seal_json(), /SESSION_ABORTED/);
assert.equal(JSON.parse(malformed.summary_json()).madi_count,0);
malformed.free();
checks += 3;
console.log(JSON.stringify({result:'ALPHA2_LOGICAL_INPUT_PASS',checks,native_aliases:3,
  actual_wasm:1,wasm_sha256:sha(binary),evidence_directory:output,
  scope:'Shared query, A3-P4 authored reaction ordering and sealed file replay; UI input and whole Alpha3 closure are not claimed'}));
