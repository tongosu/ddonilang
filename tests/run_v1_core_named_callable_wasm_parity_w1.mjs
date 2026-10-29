import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";


const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packRoot = path.resolve(process.env.DDN_VC09_PACK_ROOT || path.join(root, "pack", "v1_core_named_callable_evidence_rebase_w1"));
const manifest = JSON.parse(fs.readFileSync(path.join(packRoot, "manifest.detjson"), "utf8"));
const wasmDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm");
const requiredIds = [
  "NC01_NUMERIC_ARGUMENTS_RESULT",
  "NC02_TEXT_ARGUMENT_RESULT",
  "NC03_UNKNOWN_CALLABLE",
  "NC04_ARITY_MISMATCH",
  "NC05_TYPE_MISMATCH",
  "NC06_RECURSION_ATTEMPT",
];
const forbiddenFailureKeys = new Set([
  "output_log_texts", "output_rows", "resources", "patch", "state_hash", "public_observation_trace_hash",
]);

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function deepKeys(value, found = new Set()) {
  if (Array.isArray(value)) {
    for (const child of value) deepKeys(child, found);
  } else if (value && typeof value === "object") {
    for (const [key, child] of Object.entries(value)) {
      if (forbiddenFailureKeys.has(key)) found.add(key);
      deepKeys(child, found);
    }
  }
  return found;
}

function parseThrown(error) {
  const raw = String(error);
  try {
    return JSON.parse(raw);
  } catch {
    const start = raw.indexOf("{");
    if (start >= 0) {
      try { return JSON.parse(raw.slice(start)); } catch { /* fail below */ }
    }
  }
  return { raw };
}

function receiptProjection(result) {
  if (result.ok === false) {
    const error = result.error;
    assert(error && typeof error === "object", "failure diagnostic missing");
    return {
      ok: false,
      diagnostic: {
        code: error.code,
        phase: error.phase,
        source_path: error.source_path,
        source_span: error.source_span ?? null,
        offending_field: error.offending_field ?? null,
      },
    };
  }
  const payload = result.payload;
  assert(payload && typeof payload === "object", "success payload missing");
  const envelope = payload.execution_envelope;
  const workCost = payload.work_cost_receipt;
  assert(envelope && typeof envelope === "object", "execution envelope missing");
  assert(workCost && typeof workCost === "object", "work cost receipt missing");
  return {
    ok: true,
    receipt: {
      schema: payload.schema,
      source_profile_identity: payload.source_profile_identity,
      source_path: payload.source_path,
      ticks_run: payload.ticks_run,
      output_log_texts: payload.output_log_texts,
      state_hash: payload.state_hash,
      public_observation_trace_hash: payload.public_observation_trace_hash,
      canonical_ddn_sha256: envelope.canonical_ddn_sha256,
      preprocessed_source_sha256: envelope.preprocessed_source_sha256,
      source_bytes_sha256: envelope.source_bytes_sha256,
      work_cost_schema: workCost.schema,
      work_cost_counters: workCost.counters,
    },
  };
}

assert.equal(manifest.schema, "ddn.v1_core_named_callable_evidence_rebase_w1.v1", "manifest schema");
assert.equal(manifest.source_profile_identity, "v1-core-v25", "profile identity");
assert.equal(manifest.repeat_count, 3, "repeat count");
assert.deepEqual(manifest.cases.map((item) => item.id), requiredIds, "case count/id/order");

for (const item of manifest.cases) {
  const fixtureBytes = fs.readFileSync(path.join(packRoot, item.path));
  const normalizedFixtureBytes = Buffer.from(fixtureBytes.toString("utf8").replace(/\r\n/gu, "\n"), "utf8");
  assert.equal(sha256(normalizedFixtureBytes), item.fixture_sha256, `${item.id} fixture hash`);
  const receiptBytes = fs.readFileSync(path.join(packRoot, item.receipt));
  const normalizedReceiptBytes = Buffer.from(receiptBytes.toString("utf8").replace(/\r\n/gu, "\n"), "utf8");
  assert.equal(sha256(normalizedReceiptBytes), item.receipt_sha256, `${item.id} receipt hash`);
}

const freshness = spawnSync(
  process.env.PYTHON || "python",
  ["tests/run_ddn_cli_wasm_freshness_check.py"],
  { cwd: root, encoding: "utf8", env: { ...process.env, PYTHONUTF8: "1" } },
);
assert.equal(
  freshness.status,
  0,
  `tracked WASM freshness failed\n${freshness.stdout}\n${freshness.stderr}`,
);
assert.match(freshness.stdout, /\[ddn-cli-wasm-freshness\] PASS/u, "freshness PASS marker missing");

const wasm = await import(pathToFileURL(path.join(wasmDir, "ddonirang_tool.js")).href);
assert.equal(typeof wasm.default, "function", "WASM initializer missing");
assert.equal(typeof wasm.wasm_run_summary_with_source_profile_json, "function", "supported-profile export missing");
await wasm.default({ module_or_path: fs.readFileSync(path.join(wasmDir, "ddonirang_tool_bg.wasm")) });

const nativeRun = spawnSync(
  process.env.PYTHON || "python",
  ["tests/run_v1_core_named_callable_evidence_rebase_w1.py", "--emit-native-json", "--skip-supplementary"],
  { cwd: root, encoding: "utf8", env: { ...process.env, PYTHONUTF8: "1", DDN_VC09_PACK_ROOT: packRoot } },
);
assert.equal(nativeRun.status, 0, `native runner failed\n${nativeRun.stdout}\n${nativeRun.stderr}`);
const nativeLines = nativeRun.stdout.split(/\r?\n/).filter((line) => line.trim());
assert.equal(nativeLines.length, 1, "native fixed projection must be one JSON line");
const native = JSON.parse(nativeLines[0]);
assert.equal(native.schema, "ddn.vc09.native.fixed.v1");
assert.deepEqual(native.cases.map((item) => item.id), requiredIds);

const wasmCases = [];
for (const [index, item] of manifest.cases.entries()) {
  const source = fs.readFileSync(path.join(packRoot, item.path), "utf8").replace(/\r\n/gu, "\n");
  const sourcePath = `pack/v1_core_named_callable_evidence_rebase_w1/${item.path}`;
  const runs = [];
  for (let repeat = 0; repeat < manifest.repeat_count; repeat += 1) {
    try {
      const payload = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(
        source,
        sourcePath,
        item.ticks,
        manifest.source_profile_identity,
      ));
      runs.push({ ok: true, payload });
    } catch (error) {
      runs.push({ ok: false, error: parseThrown(error) });
    }
  }
  assert.deepEqual(runs.slice(1), Array(manifest.repeat_count - 1).fill(runs[0]), `${item.id} deterministic repeats`);
  assert.equal(runs[0].ok, item.kind === "success", `${item.id} outcome`);
  if (item.kind === "failure") {
    assert.deepEqual([...deepKeys(runs[0].error)], [], `${item.id} partial publish`);
  }
  const expected = JSON.parse(fs.readFileSync(path.join(packRoot, item.receipt), "utf8"));
  assert.deepEqual(receiptProjection(runs[0]), expected, `${item.id} fixed receipt`);
  assert.deepEqual(runs[0], native.cases[index].result, `${item.id} native/WASM exact parity`);
  wasmCases.push({ id: item.id, result: runs[0] });
}

console.log(
  `VC09_WASM_PARITY_PASS cases=${wasmCases.length} native_wasm_exact=${wasmCases.length} ` +
  `deterministic_repeats=${manifest.repeat_count} partial_publish=0 freshness=PASS tracked_wasm_refresh=0`,
);
