import fs from "fs";
import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const SCHEMA = "ddn.observation_manifest.v0";
const VERSION = "20.6.33";

function nativeManifest(overrides = {}) {
  return {
    schema: SCHEMA,
    version: VERSION,
    nodes: [{ name: "위치", dtype: "f64", role: "state" }],
    params: [],
    traces: [],
    diagnostics: [],
    default_pivot: { x_channel: "", y_channels: ["위치"] },
    ...overrides,
  };
}

function nativeState(manifest = nativeManifest()) {
  return {
    schema: "seamgrim.state.v0",
    tick_id: 7,
    state_hash: "blake3:native-state",
    channels: [{ name: "JS가_해석하면_안_되는_표현용_열", role: "presentation-only" }],
    row: [7],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
    observation_manifest: manifest,
  };
}

function createClient({ state, columnsPayload = null }) {
  return {
    updateLogic() {},
    parseWarningsParsed() { return []; },
    getStateParsed() { return state; },
    resetParsed() { return state; },
    stepOneParsed() { return state; },
    stepOneWithInputParsed() { return state; },
    columnsParsed() {
      return columnsPayload ?? {
        columns: state?.channels ?? [],
        row: state?.row ?? [],
        observation_manifest: state?.observation_manifest,
      };
    },
    getStateHash() { return String(state?.state_hash ?? ""); },
    setParamParsed() { return { ok: true }; },
  };
}

function createHandle(WasmVmHandle, client) {
  return new WasmVmHandle({
    loader: {
      async ensure() { return client; },
      reset() {},
      getLastBuildInfo() { return null; },
      getLastBuildInfoDiag() { return null; },
      getLastInitDiag() { return null; },
      getLastPreprocessed() { return ""; },
      getLastPreprocessDiag() { return null; },
      getCacheBust() { return 0; },
    },
    defaultSourceText: "매틱:움직씨 = { 위치 <- 7. }.",
  });
}

async function expectContractFailure(operation, expectedCode) {
  let first = null;
  let second = null;
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      await operation();
      throw new Error(`expected ${expectedCode}`);
    } catch (error) {
      assert(error?.code === expectedCode, `${expectedCode}: exact structured code`);
      assert(error?.phase === "observation_manifest", `${expectedCode}: phase`);
      assert(typeof error?.detail === "string" && error.detail.includes('"expected_schema"'), `${expectedCode}: detail`);
      if (attempt === 0) first = { message: error.message, detail: error.detail };
      else second = { message: error.message, detail: error.detail };
    }
  }
  assert(JSON.stringify(first) === JSON.stringify(second), `${expectedCode}: deterministic repeats`);
}

async function main() {
  const root = process.cwd();
  const runtimePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js");
  const workbenchPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js");
  const runtimeSource = fs.readFileSync(runtimePath, "utf8");
  const workbenchSource = fs.readFileSync(workbenchPath, "utf8");

  const { WasmVmHandle } = await import(pathToFileURL(runtimePath).href);
  const manifest = nativeManifest();
  const state = nativeState(manifest);
  const successHandle = createHandle(WasmVmHandle, createClient({ state }));
  const updated = await successHandle.updateLogic("#보개: 표.\n매틱:움직씨 = { 위치 <- 7. }.");
  assert(updated === state, "positive: native payload object is not rewritten");
  assert(updated.observation_manifest === manifest, "positive: native manifest identity is preserved");
  assert(updated.state_hash === "blake3:native-state", "positive: state identity is preserved");
  const reset = await successHandle.reset();
  const stepped = await successHandle.step({ dtMs: 16 });
  const columns = await successHandle.columns();
  assert(reset.observation_manifest === manifest, "reset: native manifest preserved");
  assert(stepped.observation_manifest === manifest, "step: native manifest preserved");
  assert(columns.observation_manifest === manifest, "columns: native manifest preserved");
  assert(successHandle.getDebugInfo().runtimeDiags.length === 0, "positive: no synthetic warning or fallback diagnostic");

  const missingState = nativeState();
  delete missingState.observation_manifest;
  const missingHandle = createHandle(WasmVmHandle, createClient({ state: missingState }));
  await expectContractFailure(
    () => missingHandle.updateLogic("매틱:움직씨 = { 위치 <- 7. }."),
    "E_WASM_OBSERVATION_MANIFEST_MISSING",
  );
  assert(
    missingHandle.getDebugInfo().runtimeDiags.at(-1)?.code === "E_WASM_OBSERVATION_MANIFEST_MISSING",
    "missing: runtime diagnostic preserves exact code",
  );
  assert(!("observation_manifest" in missingState), "missing: JS does not synthesize a manifest");

  const malformedState = nativeState(nativeManifest({ nodes: "not-an-array" }));
  const malformedHandle = createHandle(WasmVmHandle, createClient({ state: malformedState }));
  await expectContractFailure(
    () => malformedHandle.reset(),
    "E_WASM_OBSERVATION_MANIFEST_INVALID",
  );

  const unsupportedState = nativeState(nativeManifest({ version: "999.0.0" }));
  const unsupportedHandle = createHandle(WasmVmHandle, createClient({ state: unsupportedState }));
  await expectContractFailure(
    () => unsupportedHandle.step({ dtMs: 16 }),
    "E_WASM_OBSERVATION_MANIFEST_UNSUPPORTED",
  );

  const missingColumnsHandle = createHandle(WasmVmHandle, createClient({
    state,
    columnsPayload: { columns: [], row: [] },
  }));
  await expectContractFailure(
    () => missingColumnsHandle.columns(),
    "E_WASM_OBSERVATION_MANIFEST_MISSING",
  );

  const presentationVariant = nativeState(manifest);
  presentationVariant.channels = [{ name: "완전히_다른_표현", role: "view-only" }];
  presentationVariant.row = [999];
  const presentationHandle = createHandle(WasmVmHandle, createClient({ state: presentationVariant }));
  const presentationResult = await presentationHandle.updateLogic("#보개: 선.\n매틱:움직씨 = { 위치 <- 999. }.");
  assert(presentationResult.observation_manifest === manifest, "presentation independence: manifest comes from native payload only");
  assert(presentationResult.row[0] === 999, "presentation independence: payload is not normalized by JS");

  assert(!runtimeSource.includes('from "./observation_manifest.js"'), "F06: JS observation-manifest builder import removed");
  assert(!runtimeSource.includes("buildObservationManifest("), "F06: JS semantic synthesis call removed");
  assert(!runtimeSource.includes("attachObservationManifest("), "F06: legacy attach/synthesis path removed");
  assert(runtimeSource.includes("requireNativeObservationManifest("), "F06: native contract consumer exists");
  assert(workbenchSource.includes("err?.code ?? match?.[1]"), "Workbench preserves structured product code");
  assert(!workbenchSource.includes('pushDiagnostic("E_WASM_RUN_START_FAILED"'), "Workbench does not mask start failure code");
  assert(!workbenchSource.includes('pushDiagnostic("E_WASM_STEP_FAILED"'), "Workbench does not mask step failure code");

  console.log("seamgrim workbench observation manifest t4d ok (positive=1 missing=2 malformed=1 unsupported=1 deterministic=3 synthesis=0)");
}

main().catch((error) => {
  console.error(error?.stack ?? String(error));
  process.exit(1);
});
