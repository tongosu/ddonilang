import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wasmDir = process.env.DDN_WASM_DIR ? path.resolve(process.env.DDN_WASM_DIR)
  : path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm");
const alphaDir = path.join(root, "solutions", "seamgrim_ui_mvp", "alpha1");
const profile = "v1-core-v25";
// In-process delivered receipts only: this test adds no file replay ingress.
const programs = [
  ["01_요금_계산기.ddn", ["2800", "2100"]],
  ["02_조건과_반복.ddn", ["작은 제곱", "작은 제곱", "큰 제곱", "큰 제곱", "30"]],
  ["03_자동_함수표.ddn", ["0", "0", "2", "1", "1", "4", "2", "4", "10", "3", "9", "20"]],
  ["04_시간단계_시뮬레이션.ddn", ["1", "2", "3", "2", "5", "4", "3", "9", "5", "4", "14", "6"]],
];
const invalidPrograms = [
  ["01_요금_타입오류.ddn", "E_RUNTIME_TYPE_MISMATCH"],
  ["02_반복_원소타입오류.ddn", "E_RUNTIME_TYPE_MISMATCH"],
  ["03_함수표_인자누락.ddn", "E_CALL_REQUIRED_PIN_MISSING"],
  ["04_시뮬레이션_인자중복.ddn", "핀 '위치'에 인자가 중복되었습니다"],
  ["05_조사_모호오류.ddn", "조사 '을'가 모호합니다"],
];
const forbiddenFailureKeys = new Set([
  "output_log_texts",
  "output_rows",
  "resources",
  "patch",
  "state_hash",
  "public_observation_trace_hash",
]);

function parseThrown(error) {
  const raw = String(error);
  const start = raw.indexOf("{");
  if (start >= 0) {
    try {
      return JSON.parse(raw.slice(start));
    } catch {
      // Fall through to the exact text carrier.
    }
  }
  return { raw };
}

function deepForbiddenKeys(value, found = new Set()) {
  if (Array.isArray(value)) {
    for (const child of value) deepForbiddenKeys(child, found);
  } else if (value && typeof value === "object") {
    for (const [key, child] of Object.entries(value)) {
      if (forbiddenFailureKeys.has(key)) found.add(key);
      deepForbiddenKeys(child, found);
    }
  }
  return found;
}

const wasm = await import(pathToFileURL(path.join(wasmDir, "ddonirang_tool.js")).href);
await wasm.default({ module_or_path: fs.readFileSync(path.join(wasmDir, "ddonirang_tool_bg.wasm")) });
for (const [source, code] of [
  ['\uFEFF값 <- 1.', 'E_SOURCE_UTF8_BOM_FORBIDDEN'],
  ['만약 참 이라면 { 1 보여주기. }.', 'E_V25_IF_SURFACE'],
]) {
  assert.throws(() => wasm.wasm_run_summary_with_source_profile_json(
    source, 'c1-review.ddn', 1, profile,
  ), error => {
    const diagnostic = JSON.parse(String(error));
    return diagnostic.code === code && diagnostic.message.includes(code);
  });
}
const missingPinSource = fs.readFileSync(path.join(alphaDir, '잘못된입력', '03_함수표_인자누락.ddn'), 'utf8');
assert.throws(() => wasm.wasm_run_summary_with_source_profile_json(
  missingPinSource, 'missing-pin.ddn', 1, profile,
), error => {
  const diagnostic = JSON.parse(String(error));
  return diagnostic.code === 'E_CALL_REQUIRED_PIN_MISSING' && diagnostic.message.includes('(4:1)');
});
console.log('C1_REVIEW_DIAGNOSTICS_PASS bom=1 legacy_if=1 missing_pin_location=1');
if (process.env.DDN_PREVIOUS_WASM_DIR && process.env.DDN_DT_ARTIFACT_DIR) {
  const previousDir = path.resolve(process.env.DDN_PREVIOUS_WASM_DIR);
  const previous = await import(pathToFileURL(path.join(previousDir, "ddonirang_tool.js")).href);
  await previous.default({ module_or_path: fs.readFileSync(path.join(previousDir, "ddonirang_tool_bg.wasm")) });
  const cases = programs.map(([name]) => [name, fs.readFileSync(path.join(alphaDir, name), "utf8"), ""]);
  for (const name of ["포물선 운동", "진자 보기 코어", "격자 상태와 낙하 조각", "일차함수 기울기 작업", "새 물리 프로젝트", "새 한 파일", "새 빈 프로젝트"]) {
    const snapshot = JSON.parse(fs.readFileSync(path.join(process.env.DDN_DT_ARTIFACT_DIR, `${name}.json`), "utf8")).project_snapshot;
    const payload = snapshot.local_package_payload;
    let source = snapshot.ddn_text;
    let entry = "";
    if (payload?.files?.some(file => file.role === "root")) {
      // Same virtual-tree envelope as the product; source bytes are unchanged.
      const tree = { schema: "ddn.virtual_project_source.v1", kind: payload.kind,
        root_path: payload.root_path, entry_path: payload.entry_path,
        files: payload.files.map(({path, role, utf8_text}) => ({path, role, utf8_text})) };
      const prepared = JSON.parse(wasm.wasm_project_prepare_link_json(JSON.stringify(tree), "", false, false));
      source = prepared.linked_execution_source;
      entry = prepared.target;
    }
    cases.push([name, source, entry]);
  }
  function calculate(module, source, entry, high, low) {
    const vm = new module.DdnWasmVm("1 보여주기.");
    try {
      vm.update_logic_with_source_profile(source, profile);
      if (entry) vm.configure_project_entry(entry);
      return Array.from({length: 3}, () => {
        const frame = JSON.parse(vm.step_one_with_input_fixed64_v1(0, "", 0, 0, high, low));
        return {state_hash: frame.state_hash, output_rows: frame.output_rows, resources: frame.resources};
      });
    } finally { vm.free(); }
  }
  for (const [name, source, entry] of cases) {
    const oldOne = calculate(previous, source, entry, 1, 0);
    const old64 = calculate(previous, source, entry, 0, 67108864);
    const fixed = calculate(wasm, source, entry, 1, 0);
    assert.deepEqual(old64, oldOne, `${name}: prior host dt changed world/output`);
    assert.deepEqual(fixed, oldOne, `${name}: D12 changed world/output`);
    console.log(`D12_WORK_COMPARISON_PASS ${JSON.stringify({name, source_bytes: Buffer.byteLength(source), world_hashes: fixed.map(frame => frame.state_hash)})}`);
  }
  console.log("D12_ALL_WORKS_PASS works=11 frames_each=3 previous_dt=1,1/64 current_dt=1 helper_product_ingress=0");
}
assert.equal(typeof wasm.wasm_run_summary_with_source_profile_json, "function");
assert.equal(typeof wasm.wasm_canon_ddn_with_source_profile, "function");

// Real Rust decimal admission, including String(number)'s exponent spelling.
// Tiny nonzero values currently round to zero in the shared Fixed64 parser;
// recording that fact is not an assertion that the control UX is complete.
{
  const vm = new wasm.DdnWasmVm("1 보여주기.");
  try {
    const admitted = (text) => JSON.parse(vm.fixed64_from_decimal_v1(text));
    const small = admitted(String(1e-7));
    assert.deepEqual(small, admitted("0.0000001"));
    assert.deepEqual(small.value, { high_i32: 0, low_u32: 429 });
    assert.deepEqual(admitted(String(-1e-7)).value, { high_i32: -1, low_u32: 4294966867 });
    assert.throws(() => admitted(String(1e21)), /E_FIXED64_UNREPRESENTABLE_CONVERSION/);
    assert.deepEqual(admitted("1e-30"), admitted("0.000000000000000000000000000001"));
    assert.deepEqual(admitted("1e-30").value, { high_i32: 0, low_u32: 0 });
  } finally { vm.free(); }
}
console.log("CONTROL_DECIMAL_ACTUAL_WASM_PASS exponent_small=429 exponent_large=rejected tiny_rounds_zero=observed");

{
  const vm = new wasm.DdnWasmVm("1 보여주기.");
  try {
    const before = JSON.parse(vm.get_state_json());
    const fraction = "0.016666666604578495025634765625";
    const parts = JSON.parse(vm.fixed64_from_decimal_v1(fraction)).value;
    assert.deepEqual(parts, { high_i32: 0, low_u32: 71582788 });
    assert.throws(() => vm.set_dt_fixed64_v1(parts.high_i32, parts.low_u32), /E_ALPHA_HOST_DT_UNSUPPORTED/);
    assert.throws(() => vm.set_input_fixed64_v1(4, "ArrowLeft", 2, 3, 0, parts.low_u32), /E_ALPHA_HOST_DT_UNSUPPORTED/);
    assert.throws(() => vm.step_one_with_input_fixed64_v1(4, "ArrowLeft", 2, 3, 0, parts.low_u32), /E_ALPHA_HOST_DT_UNSUPPORTED/);
    assert.deepEqual(JSON.parse(vm.get_state_json()), before, "rejected host input changed the installed VM");
    const old = structuredClone(before);
    old.state.input.dt = fraction;
    assert.throws(() => vm.restore_state(JSON.stringify(old)), /E_ALPHA_HOST_DT_UNSUPPORTED/);
    assert.deepEqual(JSON.parse(vm.get_state_json()), before, "rejected restore changed the installed VM");
    vm.set_dt_fixed64_v1(1, 0);
    const oldReceipt = JSON.parse(vm.step_one()).input_receipt;
    oldReceipt.input_snapshot.input.dt = fraction;
    // Test-only mutation: preserve the receipt identity so D12, not a damaged
    // hash, is what rejects the old host interval. No file or product ingress.
    const ordered = value => Array.isArray(value) ? value.map(ordered)
      : value !== null && typeof value === "object"
        ? Object.fromEntries(Object.keys(value).sort().map(key => [key, ordered(value[key])])) : value;
    oldReceipt.input_snapshot_sha256 = createHash("sha256")
      .update(JSON.stringify(ordered(oldReceipt.input_snapshot))).digest("hex");
    assert.throws(() => wasm.wasm_run_summary_with_source_profile_input_receipts_json(
      "1 보여주기.", "d12.ddn", 1, profile, JSON.stringify([oldReceipt])), /E_ALPHA_HOST_DT_UNSUPPORTED/);
    vm.reset();
    assert.equal(JSON.parse(vm.get_state_json()).state.input.alpha_host_dt, "1");
  } finally { vm.free(); }
}
console.log("D12_ACTUAL_WASM_HOST_DT_PASS setters=3 restore_rejected=1 receipt_rejected=1 atomic_rejection=4 reset=1 fraction_raw_roundtrip=71582788");

for (const [name, body, key, reads] of [
  ["absent", "7 보여주기.", "", []],
  ["unread", "7 보여주기.", "ArrowLeft", []],
  ["read", "입력키 보여주기.", "ArrowLeft", ["last_key_name"]],
  ["short-circuit", '거짓 그리고 (입력키 == "ArrowLeft") 보여주기.', "ArrowLeft", []],
]) {
  const source = `(매마디)마다 { ${body} }.`;
  const vm = new wasm.DdnWasmVm("1 보여주기.");
  try {
    vm.update_logic_with_source_profile(source, profile);
    const frame = JSON.parse(vm.step_one_with_input_fixed64_v1(key ? 4 : 0, key, 0, 0, 1, 0));
    const receipt = frame.input_receipt;
    const summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_input_receipts_json(
      source, "i1.ddn", 1, profile, JSON.stringify([receipt])));
    assert.equal(frame.state_hash, summary.state_hash, `${name}: same delivered execution`);
    assert.equal(receipt.input_snapshot_sha256, summary.input_receipts[0].input_snapshot_sha256);
    assert.equal(receipt.input_snapshot.input.dt, "1");
    assert.deepEqual(receipt.input_reads, reads);
    assert.equal(receipt.tape_v1_recording_error === null, reads.length === 0);
    if (reads.length) assert.match(receipt.tape_v1_recording_error, /^E_INPUT_TAPE_V1_UNRECORDED_READ:/);
    if (reads.length) {
      assert.throws(() => wasm.wasm_export_input_tape_v1(JSON.stringify([receipt]), 60),
        /E_INPUT_TAPE_V1_UNRECORDED_READ/);
      if (process.env.DDN_NATIVE_DIR) {
        const directory = fs.mkdtempSync(path.join(os.tmpdir(), "ddn-tape-reject-"));
        try {
          const sourceFile = path.join(directory, "i1.ddn");
          fs.writeFileSync(sourceFile, source, "utf8");
          for (const alias of ["ddonirang", "ddonilang", "또니랑"]) {
            const tapeFile = path.join(directory, `${alias}.tape`);
            const cli = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
              ["실행", sourceFile, "--source-profile", profile, "--sam-live", "web",
                "--sam-live-port", "0", "--madi", "1", "--record-sam", tapeFile, "--json"],
              { encoding: "utf8", windowsHide: true, timeout: 10000 });
            assert.ifError(cli.error);
            assert.equal(cli.status, 0, cli.stderr);
            const recorded = fs.readFileSync(tapeFile);
            assert.equal(recorded.subarray(0, 18).toString("utf8"), "DDN_INPUT_TAPE_V2\n");
            assert.throws(() => wasm.wasm_run_summary_with_source_profile_tape_v1_json(
              source, "i1.ddn", profile, recorded), /E_SAM_TAPE_VERSION_UNSUPPORTED/);
          }
        } finally { fs.rmSync(directory, { recursive: true }); }
      }
    } else {
      const bytes = wasm.wasm_export_input_tape_v1(JSON.stringify([receipt]), 60);
      assert.equal(new TextDecoder().decode(bytes.slice(0, 18)), "DDN_INPUT_TAPE_V1\n");
      const replay = JSON.parse(wasm.wasm_run_summary_with_source_profile_tape_v1_json(
        source, "i1.ddn", profile, bytes));
      assert.equal(replay.state_hash, frame.state_hash, `${name}: Tape V1 compatible world`);
      if (process.env.DDN_NATIVE_DIR) {
        const directory = fs.mkdtempSync(path.join(os.tmpdir(), "ddn-tape-v1-"));
        try {
          const sourceFile = path.join(directory, "i1.ddn");
          const tapeFile = path.join(directory, "input.tape");
          fs.writeFileSync(sourceFile, source, "utf8");
          fs.writeFileSync(tapeFile, bytes);
          for (const alias of ["ddonirang", "ddonilang", "또니랑"]) {
            const cli = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
              ["실행", sourceFile, "--source-profile", profile, "--sam", tapeFile, "--json"],
              { encoding: "utf8", windowsHide: true, timeout: 10000 });
            assert.ifError(cli.error);
            assert.equal(cli.status, 0, cli.stderr);
            const result = JSON.parse(cli.stdout);
            assert.equal(result.state_hash, replay.state_hash, `${alias}/${name}: actual Tape file`);
            assert.equal(result.input_record_sha256, replay.input_record_sha256);
            const recordedFile = path.join(directory, `${alias}.tape`);
            const recorded = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
              ["실행", sourceFile, "--source-profile", profile, "--sam-live", "web",
                "--sam-live-port", "0", "--madi", "1", "--record-sam", recordedFile, "--json"],
              { encoding: "utf8", windowsHide: true, timeout: 10000 });
            assert.ifError(recorded.error);
            assert.equal(recorded.status, 0, recorded.stderr);
            const recordedBytes = fs.readFileSync(recordedFile);
            assert.equal(recordedBytes.subarray(0, 18).toString("utf8"), "DDN_INPUT_TAPE_V2\n");
            assert.throws(() => wasm.wasm_run_summary_with_source_profile_tape_v1_json(
              source, "i1.ddn", profile, recordedBytes), /E_SAM_TAPE_VERSION_UNSUPPORTED/);
          }
        } finally { fs.rmSync(directory, { recursive: true }); }
      }
      assert.throws(() => wasm.wasm_run_summary_with_source_profile_tape_v1_json(
        '(매마디)마다 { 입력키 보여주기. }.', "i1.ddn", profile, bytes),
        /E_INPUT_TAPE_V1_UNRECORDED_READ/);
    }
    const changed = structuredClone(receipt);
    changed.input_snapshot.input.pointer_x_i32 = 1;
    assert.throws(() => wasm.wasm_run_summary_with_source_profile_input_receipts_json(
      source, "i1.ddn", 1, profile, JSON.stringify([changed])), /E_INPUT_RECEIPT_IDENTITY_MISMATCH/);
  } finally { vm.free(); }
}
console.log(`I1_ACTUAL_WASM_RECEIPT_PASS cases=4 changed_input_rejected=4 dt=1 tape_v1_replay=3 unsupported_read_rejected=1 cli_v1_file_parity=${process.env.DDN_NATIVE_DIR ? 'PASS' : 'NOT_RUN'} cli_v2_record_wasm_rejection=${process.env.DDN_NATIVE_DIR ? 'PASS' : 'NOT_RUN'} full_input_file_replay=0`);

for (const [name, expectedOutput] of programs) {
  const source = fs.readFileSync(path.join(alphaDir, name), "utf8").replace(/\r\n/gu, "\n");
  const sourcePath = `solutions/seamgrim_ui_mvp/alpha1/${name}`;
  const runs = [0, 1].map(() =>
    JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, sourcePath, 1, profile))
  );
  assert.deepEqual(runs[1], runs[0], `${name}: repeated WASM run drift`);
  assert.deepEqual(runs[0].output_log_texts, expectedOutput, `${name}: output`);
  const canonical = wasm.wasm_canon_ddn_with_source_profile(source, profile);
  const recanonical = wasm.wasm_canon_ddn_with_source_profile(canonical, profile);
  assert.equal(recanonical, canonical, `${name}: canonical roundtrip`);
}

const shortLiteralDefinition = "값 := 100.\n값 보여주기.\n";
const shortLiteralRun = JSON.parse(
  wasm.wasm_run_summary_with_source_profile_json(
    shortLiteralDefinition,
    "tests/fixtures/alpha1_short_literal_definition.ddn",
    1,
    profile,
  ),
);
assert.deepEqual(shortLiteralRun.output_log_texts, ["100"], "short literal definition output");
const shortLiteralCanon = wasm.wasm_canon_ddn_with_source_profile(shortLiteralDefinition, profile);
assert.equal(
  wasm.wasm_canon_ddn_with_source_profile(shortLiteralCanon, profile),
  shortLiteralCanon,
  "short literal definition canonical roundtrip",
);

for (const [name, expectedText] of invalidPrograms) {
  const source = fs.readFileSync(path.join(alphaDir, "잘못된입력", name), "utf8").replace(/\r\n/gu, "\n");
  const sourcePath = `solutions/seamgrim_ui_mvp/alpha1/잘못된입력/${name}`;
  const failures = [];
  for (let repeat = 0; repeat < 2; repeat += 1) {
    try {
      wasm.wasm_run_summary_with_source_profile_json(source, sourcePath, 1, profile);
      assert.fail(`${name}: invalid input unexpectedly succeeded`);
    } catch (error) {
      failures.push(parseThrown(error));
    }
  }
  assert.deepEqual(failures[1], failures[0], `${name}: repeated failure drift`);
  assert.match(JSON.stringify(failures[0]), new RegExp(expectedText.replace(/[.*+?^$\{\}()|[\]\\]/gu, "\\$&"), "u"));
  assert.deepEqual([...deepForbiddenKeys(failures[0])], [], `${name}: partial publish`);
}

const rootCases = [
  ['(사용량:수~을~를, 기본요금:수~에, 단가:수~로~으로) 요금계산:수:셈씨 := { 요금계산 <- 기본요금 + 사용량 * 단가. }.',
    '결과 := 120을 1000에 15로 요금계산하기. 결과 보여주기.', ['2800']],
  ['', '입력들:(수)차림 := 차림[1, 2, 3]. 입력들마다 { 입력들 보여주기. }.', ['1', '2', '3']],
];
const basicAuthoringCases = [
  ["정의", "값 := 1. 값 보여주기.", ["1"]],
  ["결속", "값 <- 1. 값 <- 2. 값 보여주기.", ["2"]],
  ["호출", `${rootCases[0][0]} ${rootCases[0][1]}`, ["2800"]],
  ["조건", "{ 참 }이면 { 1 보여주기. } 아니면 { 2 보여주기. }.", ["1"]],
  ["반복", "값 <- 0. { 값 < 2 }인 동안 { 값 <- 값 + 1. }. 값 보여주기.", ["2"]],
  ["되풀이", "되풀이 { 1 보여주기. 멈추기. }.", ["1"]],
  ["순회", rootCases[1][1], ["1", "2", "3"]],
  ["출력", '"가" 보여주기.', ["가"]],
  ["찾기", "((차림[1, 2], 0) 찾기) 있는가 보여주기.", ["참"]],
  ["논리식", "(거짓 그리고 ((1 / 0) == 0)) 보여주기.", ["거짓"]],
];
const basicDirectory = process.env.DDN_NATIVE_DIR ? fs.mkdtempSync(path.join(os.tmpdir(), "ddn-basic-authoring-")) : null;
const basicAuthoringRejections = [
  ["정의", '값:수 := "가".'],
  ["결속", '값 <- 1. 값 <- "가".'],
  ["호출", `${rootCases[0][0]} 결과 := 120을 요금계산하기.`],
  ["조건", "{ 1 }이면 { 1 보여주기. }."],
  ["반복", "{ 없음 }인 동안 { 1 보여주기. }."],
  ["되풀이", "되풀이 { (1 / 0) 보여주기. 멈추기. }."],
  ["순회", "입력들:수 := 1. 입력들마다 { 입력들 보여주기. }."],
  ["출력", "미정 보여주기."],
  ["찾기", 'm <- ("a", 없음) 짝맞춤. ((m, "a") 찾기) 있는가 보여주기.'],
  ["논리식", "(거짓 그리고 1) 보여주기."],
];
try {
  for (const [name, source, expected] of basicAuthoringCases) {
    const summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, "기본.ddn", 1, profile));
    assert.deepEqual(summary.output_log_texts, expected, name);
    if (basicDirectory) {
      const input = path.join(basicDirectory, "기본.ddn");
      fs.writeFileSync(input, source, "utf8");
      for (const alias of ["ddonirang", "ddonilang", "또니랑"]) {
        const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
          ["실행", input, "--source-profile", profile, "--마디수", "1", "--json"],
          {encoding: "utf8", timeout: 10000, windowsHide: true});
        assert.ifError(run.error);
        assert.equal(run.status, 0, `${name}/${alias}: ${run.stderr}`);
        assert.deepEqual(JSON.parse(run.stdout).resources.value_json.output_log.map(row => row.text), expected);
      }
    }
  }
  for (const [name, source] of basicAuthoringRejections) {
    let diagnostic;
    try { wasm.wasm_run_summary_with_source_profile_json(source, "거부.ddn", 1, profile); }
    catch (error) { diagnostic = String(error); }
    assert.ok(diagnostic, `${name}: invalid source accepted by WASM`);
    if (basicDirectory) {
      const input = path.join(basicDirectory, "거부.ddn");
      fs.writeFileSync(input, source, "utf8");
      for (const alias of ["ddonirang", "ddonilang", "또니랑"]) {
        const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
          ["실행", input, "--source-profile", profile, "--마디수", "1", "--json"],
          {encoding: "utf8", timeout: 10000, windowsHide: true});
        assert.ifError(run.error);
        assert.notEqual(run.status, 0, `${name}/${alias}: invalid source accepted`);
      }
    }
    console.log(`BASIC_AUTHORING_REJECT ${JSON.stringify({name, source, diagnostic})}`);
  }
} finally { if (basicDirectory) fs.rmSync(basicDirectory, {recursive: true}); }
console.log(`BASIC_AUTHORING_ACTUAL_PASS normal=10 rejected=10 wasm=PASS cli3=${basicDirectory ? "PASS" : "NOT_RUN"}`);
for (const [declaration, body, expected] of rootCases) {
  for (const source of [`${declaration}\n${body}`, `${declaration}\n(매마디)마다 { ${body} }.`]) {
    const summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, '원문.ddn', 1, profile));
    assert.deepEqual(summary.output_log_texts, expected, source);
  }
}
for (const body of ['되풀이 { { 거짓 }인것 바탕으로(물림) 아니면 { }. }.',
  '{ 참 }인 동안 { { 거짓 }인것 바탕으로(물림) 아니면 { }. }.']) {
  assert.throws(() => wasm.wasm_run_summary_with_source_profile_json(body, '중단.ddn', 1, profile), /E_ROOT_PROGRAM_ABORTED/);
}
assert.throws(() => wasm.wasm_run_summary_with_source_profile_json(
  '값:(수,글)차림 := 차림[1, 2]. 값 보여주기.', '씨인자.ddn', 1, profile), /E_RUNTIME_TYPE_MISMATCH/);

const conditionBindingFailures = [
  // Shared static validation now rejects these before runtime execution.
  // Owner D6: canonical spelling; non-Bool rejection expectation unchanged.
  ['{ 1 }이면 { 100 보여주기. }.', 'E_CHECK_TYPE_MISMATCH'],
  // Eval(Thunk,Bool) inference now also checks while before any execution.
  ['{ 없음 }인 동안 { 100 보여주기. }.', 'E_CHECK_TYPE_MISMATCH'],
  // WO02: the existing CLI static rejection now also precedes WASM execution.
  ['값:수 := 1. 값 <- "가". 값 보여주기.', 'E_CHECK_TYPE_MISMATCH'],
  ['값:글 := "가". 값 <- 1. 값 보여주기.', 'E_CHECK_TYPE_MISMATCH'],
  ['판:(수)차림 := 차림[1, 2]. "가" -> 판[0]. 판 보여주기.', 'E_RUNTIME_TYPE_MISMATCH'],
];
for (const [body, code] of conditionBindingFailures) {
  for (const source of [body, `(매마디)마다 { ${body} }.`]) {
    let failure;
    try {
      wasm.wasm_run_summary_with_source_profile_json(source, '조건결속.ddn', 1, profile);
    } catch (error) {
      failure = parseThrown(error);
    }
    assert.ok(failure, `잘못된 원문이 성공했습니다: ${source}`);
    assert.ok(JSON.stringify(failure).includes(code), JSON.stringify(failure));
    assert.deepEqual([...deepForbiddenKeys(failure)], [], '거부된 실행의 부분 결과');
  }
}
const conditionBindingValid = [
  '값:수 := 1. { 참 }이면 { 값 <- 2. }. 값 보여주기.',
  '항목:수 := 7. 대상 := 차림["가", "나"]. (항목) 대상에 대해 { 항목 <- "다". }. 항목 <- 2. 항목 보여주기.',
  '계산:수:셈씨 := { 중간:수 := 1. 중간 <- 2. 계산 <- 중간. }. 중간:글 := "가". () 계산 보여주기. 중간 <- "나".',
];
for (const body of conditionBindingValid) {
  const summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(body, '정상결속.ddn', 1, profile));
  assert.deepEqual(summary.output_log_texts, ['2']);
}

const wo02Cases = [
  ['값 := 1. 값 <- "가".', null, 'E_CHECK_TYPE_MISMATCH'],
  ['값 <- 1. 값 <- "가".', null, 'E_CHECK_TYPE_MISMATCH'],
  ['값 := 1. 값 <- 2. 값 보여주기.', ['2']],
  // Definition-bearing source isolates D2 from the separately tracked WO03 F2
  // legacy source-epoch arithmetic behavior; no source rewriting is performed.
  ['검사 := (거짓 그리고 ((1 / 0) == 0)). 검사 보여주기.', ['거짓']],
  ['검사 := (참 또는 ((1 / 0) == 0)). 검사 보여주기.', ['참']],
  ['검사 := (참 그리고 ((1 / 0) == 0)). 검사 보여주기.', null, '0으로 나눌 수 없습니다'],
  ['(거짓 그리고 1) 보여주기.', null, 'E_CHECK_TYPE_MISMATCH'],
  ['m <- ("a", 없음) 짝맞춤. ((m, "a") 찾기) 있는가 보여주기.', null, 'E_RUNTIME_NESTED_OPTIONAL_UNSUPPORTED'],
  ['m <- ("a", 없음) 짝맞춤. ((m, "zz") 찾기) 있는가 보여주기.', ['거짓']],
  ['m <- ("a", 0, "b", 거짓) 짝맞춤. ((m, "a") 찾기) 있는가 보여주기. ((m, "b") 찾기) 있는가 보여주기.', ['참', '참']],
];
for (const [source, expected, code] of wo02Cases) {
  const run = () => wasm.wasm_run_summary_with_source_profile_json(source, 'wo02.ddn', 1, profile);
  if (expected) assert.deepEqual(JSON.parse(run()).output_log_texts, expected, source);
  else assert.throws(run, error => String(error).includes(code), source);
}
// Execute the wasm32-only unit test's assertions against the actual exported
// WASM VM. Native unit-test totals never stand in for this execution.
const validVmSource = '값 <- 1. 값 보여주기.';
const invalidVmSource = '값 <- 1. 값 <- "가".';
assert.throws(() => new wasm.DdnWasmVm(invalidVmSource), /E_CHECK_TYPE_MISMATCH/);
const wo02Vm = new wasm.DdnWasmVm(validVmSource);
const beforeIdentity = wo02Vm.get_debug_source_identity_json();
const beforeState = wo02Vm.get_state_json();
for (const update of [
  () => wo02Vm.update_logic(invalidVmSource),
  () => wo02Vm.update_logic_with_mode(invalidVmSource, 'strict'),
  () => wo02Vm.update_logic_with_source_profile(invalidVmSource, profile),
]) {
  assert.throws(update, /E_CHECK_TYPE_MISMATCH/);
  assert.equal(wo02Vm.get_debug_source_identity_json(), beforeIdentity);
  assert.equal(wo02Vm.get_state_json(), beforeState);
}
wo02Vm.free();
console.log('WO02_ACTUAL_WASM_PASS corpus=10 rejected_vm_updates=3');

if (process.env.DDN_NATIVE_DIR) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'ddn-r24-parity-'));
  try {
    for (const name of ['ddonirang', 'ddonilang', '또니랑']) {
      for (const [index, [source, expected, code]] of wo02Cases.entries()) {
        const input = path.join(temporary, `wo02-${index}.ddn`);
        fs.writeFileSync(input, source, 'utf8');
        const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${name}.exe`),
          ['실행', input, '--source-profile', profile, '--마디수', '1', '--json'],
          {encoding:'utf8',timeout:10000,windowsHide:true});
        assert.ifError(run.error);
        assert.equal(run.status === 0, expected !== null, `${name}: ${source}\n${run.stderr}`);
        if(expected) assert.deepEqual(JSON.parse(run.stdout).resources.value_json.output_log.map(row=>row.text), expected);
        else {
          assert.ok(`${run.stderr}\n${run.stdout}`.includes(code), `${name}: expected ${code}`);
          assert.equal(run.stdout.includes('"state_hash"'), false);
        }
      }
      for (const [index, [source, accepted]] of [
        ...conditionBindingFailures.map(([source]) => [source, false]),
        ...conditionBindingValid.map(source => [source, true]),
      ].entries()) {
        const input = path.join(temporary, `${index}.ddn`);
        fs.writeFileSync(input, source, 'utf8');
        const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${name}.exe`),
          ['실행', input, '--source-profile', profile, '--마디수', '1', '--json'],
          {encoding: 'utf8', timeout: 10000, windowsHide: true});
        assert.ifError(run.error);
        assert.equal(run.status === 0, accepted, `${name}: ${source}\n${run.stderr}\n${run.stdout}`);
        if (accepted) {
          const summary = JSON.parse(run.stdout);
          assert.deepEqual(summary.resources.value_json.output_log.map(row => row.text), ['2']);
        } else {
          assert.equal(run.stdout.includes('"state_hash"'), false, `${name}: 거부된 실행의 성공 결과`);
        }
      }
    }
    console.log('APPROVED_RULES_CLI_WASM_PASS aliases=3 cases_per_alias=8');
  } finally {
    fs.rmSync(temporary, {recursive: true});
  }
}

console.log(
  `ALPHA1_WASM_ACTUAL_PASS programs=${programs.length} short_literal_definition=1 failures=${invalidPrograms.length} repeats=2 canon_roundtrip=${programs.length + 1} partial_publish=0 root_body=4 abort=2 type_arity=1 condition_binding_reject=10 condition_binding_accept=3`,
);
