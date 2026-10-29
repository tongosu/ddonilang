import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cliPath = process.env.DDN_W1B_CLI;
const wasmDir = process.env.DDN_W1B_WASM_DIR;
const trackedWasmDir = process.env.DDN_W1B_TRACKED_WASM_DIR;
assert(cliPath, "DDN_W1B_CLI must point to a source-built teul-cli executable");
assert(wasmDir, "DDN_W1B_WASM_DIR must point to a source-built nodejs wasm-pack directory");
assert(trackedWasmDir, "DDN_W1B_TRACKED_WASM_DIR must point to the tracked Workbench WASM directory");

const require = createRequire(import.meta.url);
const wasm = require(path.join(wasmDir, "ddonirang_tool.js"));
const trackedWasm = await import(
  pathToFileURL(path.join(trackedWasmDir, "ddonirang_tool.js")).href
);
trackedWasm.initSync({
  module: fs.readFileSync(path.join(trackedWasmDir, "ddonirang_tool_bg.wasm")),
});
const packRoot = "pack/future_grammar_atlas_owner_closed_delta_r2";

function casePath(caseName) {
  return `${packRoot}/${caseName}/input.ddn`;
}

function source(caseName) {
  return fs.readFileSync(path.join(repoRoot, casePath(caseName)), "utf8");
}

function runCli(args) {
  const result = spawnSync(cliPath, args, {
    cwd: repoRoot,
    encoding: "utf8",
    windowsHide: true,
  });
  return {
    status: result.status,
    stdout: result.stdout.replaceAll("\r\n", "\n"),
    stderr: result.stderr.replaceAll("\r\n", "\n"),
  };
}

function nativeCanonPresentation(canonicalBody) {
  return canonicalBody.endsWith("\n") ? canonicalBody : `${canonicalBody}\n`;
}

for (const caseName of ["c11_async_canon_roundtrip", "c12_rendezvous_canon_roundtrip"]) {
  const native = runCli(["canon", casePath(caseName)]);
  assert.equal(native.status, 0, `${caseName}: native canon failed: ${native.stderr}`);
  const sourceCanonBody = wasm.wasm_canon_ddn(source(caseName));
  assert.equal(
    native.stdout,
    nativeCanonPresentation(sourceCanonBody),
    `${caseName}: source WASM canon with native presentation LF`,
  );
  const trackedCanonBody = trackedWasm.wasm_canon_ddn(source(caseName));
  assert.equal(trackedCanonBody, sourceCanonBody, `${caseName}: source/tracked WASM canon body`);
  assert.equal(
    native.stdout,
    nativeCanonPresentation(trackedCanonBody),
    `${caseName}: tracked WASM canon with native presentation LF`,
  );
}

for (const caseName of ["c08_historical_migration_explicit", "c10_historical_migration_span"]) {
  const sourcePath = casePath(caseName);
  const native = runCli(["historical-async-migration-evidence", sourcePath]);
  assert.equal(native.status, 0, `${caseName}: native migration failed: ${native.stderr}`);
  const nativeArtifact = native.stdout.trimEnd();
  const wasmArtifact = wasm.wasm_historical_async_migration_evidence_json(
    source(caseName),
    sourcePath,
  );
  assert.equal(wasmArtifact, nativeArtifact, `${caseName}: migration artifact parity`);
  assert.equal(
    trackedWasm.wasm_historical_async_migration_evidence_json(source(caseName), sourcePath),
    nativeArtifact,
    `${caseName}: tracked Workbench migration artifact parity`,
  );
  const parsed = JSON.parse(wasmArtifact);
  assert.equal(parsed.parser_mode, "HistoricalMigrationEvidence");
  assert.equal(parsed.product_execution_authorized, false);
  for (const artifact of parsed.artifacts) {
    assert.equal(artifact.historical_identity, "MessageTransfer::HistoricalAsync");
    assert.equal(artifact.diagnostic_code, "E_HISTORICAL_ASYNC_MIGRATION_REQUIRED");
    assert.ok(Number.isInteger(artifact.source_span.start));
    assert.ok(artifact.source_span.end > artifact.source_span.start);
    assert.match(artifact.successor_replacement, /~>/);
  }
}

const boundaryCase = "c13_native_wasm_rendezvous_boundary";
const boundarySource = source(boundaryCase);
const nativeBoundary = runCli(["run", casePath(boundaryCase)]);
assert.equal(nativeBoundary.status, 1, "native rendezvous must fail closed");
assert.match(nativeBoundary.stderr, /E_RENDEZVOUS_PRODUCT_OPEN/);

const vm = new wasm.DdnWasmVm(boundarySource);
const beforeState = JSON.stringify(vm.get_state_json());
const beforeHash = vm.get_state_hash();
let wasmBoundaryError = "";
try {
  vm.step_one();
} catch (error) {
  wasmBoundaryError = String(error);
}
assert.match(wasmBoundaryError, /E_RENDEZVOUS_PRODUCT_OPEN/);
assert.equal(JSON.stringify(vm.get_state_json()), beforeState, "WASM failure must preserve state");
assert.equal(vm.get_state_hash(), beforeHash, "WASM failure must preserve state hash");
vm.free();

const trackedVm = new trackedWasm.DdnWasmVm(boundarySource);
const trackedBeforeState = JSON.stringify(trackedVm.get_state_json());
const trackedBeforeHash = trackedVm.get_state_hash();
let trackedBoundaryError = "";
try {
  trackedVm.step_one();
} catch (error) {
  trackedBoundaryError = String(error);
}
assert.match(trackedBoundaryError, /E_RENDEZVOUS_PRODUCT_OPEN/);
assert.equal(JSON.stringify(trackedVm.get_state_json()), trackedBeforeState);
assert.equal(trackedVm.get_state_hash(), trackedBeforeHash);
trackedVm.free();

assert.throws(
  () => wasm.DdnWasmVm.new_with_mode(boundarySource, "historical-migration-evidence"),
  /지원하지 않는 lang-mode/,
  "historical migration mode must not enter the WASM product VM",
);
assert.throws(
  () => trackedWasm.DdnWasmVm.new_with_mode(boundarySource, "historical-migration-evidence"),
  /지원하지 않는 lang-mode/,
  "historical migration mode must not enter the tracked Workbench VM",
);

console.log(
  JSON.stringify({
    schema: "ddn.execution_concurrency.w1b.native_wasm_parity.v1",
    canon_cases: 2,
    migration_cases: 2,
    rendezvous_fail_closed_cases: 1,
    diagnostic: "E_RENDEZVOUS_PRODUCT_OPEN",
    failure_atomicity: true,
    tracked_workbench_wasm: true,
  }),
);
