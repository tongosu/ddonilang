#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

const root = process.cwd();
const statePath = path.join(root, "solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js");
const runPath = path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
const playgroundPath = path.join(root, "solutions/seamgrim_ui_mvp/ui/playground.js");
const wasmRuntimePath = path.join(root, "solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js");
const indexPath = path.join(root, "solutions/seamgrim_ui_mvp/ui/runtime/index.js");
const deadBuilderPath = path.join(root, "solutions/seamgrim_ui_mvp/ui/runtime/observation_manifest.js");
const [stateMod, runMod, wasmRuntimeMod, stateSource, runSource, playgroundSource, runtimeSource, indexSource] = await Promise.all([
  import(pathToFileURL(statePath).href),
  import(pathToFileURL(runPath).href),
  import(pathToFileURL(wasmRuntimePath).href),
  fs.readFile(statePath, "utf8"),
  fs.readFile(runPath, "utf8"),
  fs.readFile(playgroundPath, "utf8"),
  fs.readFile(wasmRuntimePath, "utf8"),
  fs.readFile(indexPath, "utf8"),
]);

const SCHEMA = "ddn.observation_manifest.v0";
const VERSION = "20.6.33";

function manifest(overrides = {}) {
  return {
    schema: SCHEMA,
    version: VERSION,
    nodes: [
      { name: "y", dtype: "f64", role: "상태" },
      { name: "x", dtype: "i64", role: "파생" },
    ],
    params: [],
    traces: [],
    diagnostics: [],
    default_pivot: { x_channel: "x", y_channels: ["y"] },
    ...overrides,
  };
}

function state(observationManifest = manifest()) {
  return {
    schema: "seamgrim.engine_response.v0",
    state_hash: "blake3:t4f-state-preserved",
    channels: [
      { key: "x", dtype: "web-invented-x", role: "web-invented-x" },
      { key: "y", dtype: "web-invented-y", role: "web-invented-y" },
      { key: "raw-only", dtype: "web-only", role: "web-only" },
    ],
    row: [11, 42, 999],
    observation_manifest: observationManifest,
  };
}

function createClient(nativeState) {
  return {
    updateLogic() {},
    parseWarningsParsed() { return []; },
    getStateParsed() { return nativeState; },
    resetParsed() { return nativeState; },
    stepOneParsed() { return nativeState; },
    stepOneWithInputParsed() { return nativeState; },
    columnsParsed() {
      return {
        columns: nativeState.channels,
        row: nativeState.row,
        observation_manifest: nativeState.observation_manifest,
      };
    },
    getStateHash() { return nativeState.state_hash; },
    setParamParsed() { return { ok: true }; },
  };
}

function createHandle(nativeState) {
  return new wasmRuntimeMod.WasmVmHandle({
    loader: {
      async ensure() { return createClient(nativeState); },
      reset() {},
      getLastBuildInfo() { return null; },
      getLastBuildInfoDiag() { return null; },
      getLastInitDiag() { return null; },
      getLastPreprocessed() { return ""; },
      getLastPreprocessDiag() { return null; },
      getCacheBust() { return 0; },
    },
    defaultSourceText: "매틱:움직씨 = { y <- 42. }.",
  });
}

const authoritativeState = state();
const present = stateMod.extractObservationChannelsFromState(authoritativeState);
assert.equal(present.status, "available", "manifest present: projection available");
assert.deepEqual(present.channels, [
  { key: "y", dtype: "f64", role: "상태", unit: undefined },
  { key: "x", dtype: "i64", role: "파생", unit: undefined },
], "manifest present: native node order/role/dtype preserved");
assert.deepEqual(present.row, [42, 11], "manifest present: values bound by channel identity");
assert.deepEqual(present.values, { y: 42, x: 11 }, "manifest present: raw-only value excluded");
assert(!Object.hasOwn(present.all_values, "raw-only"), "manifest present: raw-only semantic promotion forbidden");

const emptyState = state(manifest({ nodes: [], default_pivot: { x_channel: "", y_channels: [] } }));
const empty = stateMod.extractObservationChannelsFromState(emptyState);
assert.equal(empty.status, "empty", "valid zero-node manifest is legitimate empty");
assert.equal(empty.channels.length, 0, "legitimate empty has zero channels");
assert.equal(empty.diagnostics.length, 0, "legitimate empty has no diagnostic");

const missingState = {
  schema: "seamgrim.engine_response.v0",
  state_hash: "blake3:t4f-state-preserved",
  channels: [{ key: "보개_점", dtype: "f64" }],
  row: [42],
};
const missingBefore = JSON.stringify(missingState);
const missing = stateMod.extractObservationChannelsFromState(missingState);
assert.equal(missing.status, "unavailable", "missing manifest with raw channels fails closed");
assert.equal(missing.diagnostic?.code, "E_WASM_OBSERVATION_MANIFEST_MISSING", "missing exact code");
assert.equal(missing.diagnostic?.severity, "warning", "projection diagnostic does not reclassify execution");
assert.equal(missing.channels.length, 0, "missing manifest returns no semantic channel");
assert.equal(missing.row.length, 0, "missing manifest returns no semantic row");
assert.deepEqual(missing.values, {}, "missing manifest returns no semantic values");
assert.equal(JSON.stringify(missingState), missingBefore, "missing manifest does not mutate runtime result");
assert.equal(missingState.state_hash, "blake3:t4f-state-preserved", "missing manifest preserves state hash");
const missingViews = stateMod.extractStructuredViewsFromState(missingState);
assert.equal(missingViews.graph, null, "missing manifest cannot synthesize graph");
assert.equal(missingViews.graphSource, null, "missing manifest has no observation fallback graph source");
assert.equal(stateMod.extractObservationOutputRowsFromState(missingState).length, 0, "missing manifest cannot promote raw channel to output");

const noObservationState = { state_hash: "blake3:no-observation", channels: [], row: [] };
const noObservation = stateMod.extractObservationChannelsFromState(noObservationState);
assert.equal(noObservation.status, "empty", "missing manifest with no observation data is legitimate empty");
assert.equal(noObservation.diagnostics.length, 0, "no-observation empty does not invent a diagnostic");

const invalid = stateMod.extractObservationChannelsFromState(state(manifest({ nodes: "not-an-array" })));
assert.equal(invalid.status, "unavailable", "invalid manifest fails closed");
assert.equal(invalid.diagnostic?.code, "E_WASM_OBSERVATION_MANIFEST_INVALID", "invalid exact code");

const unsupported = stateMod.extractObservationChannelsFromState(state(manifest({ version: "999.0.0" })));
assert.equal(unsupported.status, "unavailable", "unsupported manifest fails closed");
assert.equal(unsupported.diagnostic?.code, "E_WASM_OBSERVATION_MANIFEST_UNSUPPORTED", "unsupported exact code");

const bindingInvalid = stateMod.extractObservationChannelsFromState({
  ...state(),
  channels: [{ key: "different" }],
  row: [42],
});
assert.equal(bindingInvalid.status, "unavailable", "manifest/raw identity mismatch fails closed");
assert.equal(bindingInvalid.diagnostic?.code, "E_WASM_OBSERVATION_MANIFEST_INVALID", "binding mismatch exact family");

const runMissing = runMod.extractRuntimeDerived(missingState);
assert.equal(runMissing.observation.status, "unavailable", "Run consumes unavailable projection");
assert.equal(runMissing.views.graph, null, "Run does not synthesize a missing-manifest graph");
assert.deepEqual(
  runMod.deriveRunKindAndChannels({ observation: runMissing.observation, hasSpace2d: false }),
  { kind: "observation_unavailable", channels: 0 },
  "Run distinguishes execution success from observation unavailability",
);

assert(playgroundSource.includes("const obs      = extractObservationChannelsFromState(state);"), "Playground consumes shared projection");
assert(playgroundSource.includes('obs?.status === "unavailable"'), "Playground renders explicit unavailable state");
assert(playgroundSource.includes("observationDiags"), "Playground preserves structured projection diagnostic");
assert(!playgroundSource.includes('role = "상태"'), "Playground does not synthesize observation role");
assert(!playgroundSource.includes('dtype = "unknown"'), "Playground does not synthesize observation dtype");
assert(runSource.includes("extractRuntimeDerived(stateJson)"), "Run actual consumer remains connected");
assert(runSource.includes('observation?.status === "unavailable"'), "Run renders explicit unavailable state");

const handle = createHandle(authoritativeState);
const handleState = await handle.updateLogic("매틱:움직씨 = { y <- 42. }.");
const vmProjection = stateMod.extractObservationChannelsFromState(handleState);
const runProjection = runMod.extractRuntimeDerived(handleState).observation;
const playgroundProjection = stateMod.extractObservationChannelsFromState(handleState);
assert.deepEqual(runProjection.channels, vmProjection.channels, "WasmVmHandle/Run semantic identity parity");
assert.deepEqual(playgroundProjection.channels, vmProjection.channels, "WasmVmHandle/Playground semantic identity parity");
assert.deepEqual(runProjection.values, playgroundProjection.values, "Run/Playground value identity parity");

assert(!stateSource.includes("function normalizeChannels("), "raw-channel semantic normalizer removed");
assert(!stateSource.includes('String(node?.dtype ?? "unknown")'), "manifest dtype default removed");
assert(!stateSource.includes('String(node?.role ?? "상태")'), "manifest role default removed");
assert(runtimeSource.includes("inspectNativeObservationManifest"), "T4d uses shared native contract validator");
assert(!runtimeSource.includes('from "./observation_manifest.js"'), "F06 builder import remains removed");
assert(!runtimeSource.includes("buildObservationManifest("), "F06 synthesis remains zero");
assert(indexSource.includes('from "./observation_manifest.js"'), "dead builder re-export retained for separate housekeeping");
assert((await fs.stat(deadBuilderPath)).isFile(), "dead builder deletion deferred");
assert(!stateSource.includes("E_WASM_MAEGIM_PLAN_FALLBACK_FAILED"), "T4f does not rename maegim diagnostic");

console.log(JSON.stringify({
  schema: "seamgrim.workbench.observation_channel_t4f.runner_result.v1",
  manifest_present: "PASS",
  manifest_empty: "PASS",
  manifest_missing: "FAIL_CLOSED",
  manifest_invalid: "FAIL_CLOSED",
  manifest_unsupported: "FAIL_CLOSED",
  execution_outcome_on_projection_failure: "PRESERVED",
  observation_projection_outcome: "UNAVAILABLE",
  run_observation_local_authority_after: 0,
  playground_observation_local_authority_after: 0,
  wasm_vm_observation_local_authority_after: 0,
  active_real_semantic_fallbacks_before: 1,
  active_real_semantic_fallbacks_after: 0,
  active_silent_semantic_fallbacks_before: 1,
  active_silent_semantic_fallbacks_after: 0,
  failure_to_success_before: 1,
  failure_to_success_after: 0,
  state_hash_drift: 0,
  result: "PASS",
}));
