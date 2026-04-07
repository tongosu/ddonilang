import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function createClient({
  warnings = [],
  state = null,
  supportsStepWithInput = true,
  seedThrows = false,
  supportsSeed = true,
} = {}) {
  const sharedState = state ?? {
    schema: "seamgrim.state.v0",
    tick_id: 0,
    channels: [],
    row: [],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
  };
  const client = {
    _updates: [],
    parseWarningsParsed() {
      return warnings;
    },
    updateLogic(body) {
      this._updates.push(String(body ?? ""));
    },
    getStateParsed() {
      return sharedState;
    },
    resetParsed() {
      return sharedState;
    },
    stepOneParsed() {
      return sharedState;
    },
    stepOneWithInputParsed(keys, lastKey, px, py, dt) {
      if (!supportsStepWithInput) {
        throw new Error("stepOneWithInputParsed unavailable");
      }
      return {
        ...sharedState,
        input: { keys, lastKey, px, py, dt },
      };
    },
    columnsParsed() {
      return { columns: [], row: [] };
    },
    setParamParsed(key, value) {
      return { ok: true, key, value };
    },
    getStateHash() {
      return "blake3:test";
    },
  };
  if (supportsSeed) {
    client.setRngSeed = function setRngSeed(seed) {
      if (seedThrows) {
        throw new Error(`seed-fail:${String(seed)}`);
      }
      this._seed = Number(seed);
    };
  }
  return client;
}

async function main() {
  const root = process.cwd();
  const runtimePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js");
  const runtimeMod = await import(pathToFileURL(runtimePath).href);
  const { WasmVmHandle } = runtimeMod;
  assert(typeof WasmVmHandle === "function", "runtime export: WasmVmHandle");

  const strictWarnings = [
    {
      code: "W_BLOCK_HEADER_COLON_DEPRECATED",
      message: "deprecated",
      span: { start: 0, end: 1 },
    },
  ];
  const strictClient = createClient({ warnings: strictWarnings, seedThrows: true });
  const compatClient = createClient({ warnings: [] });

  let ensureCount = 0;
  const loader = {
    async ensure() {
      ensureCount += 1;
      return ensureCount === 1 ? strictClient : compatClient;
    },
    reset() {},
    getLastBuildInfo() {
      return "build-info";
    },
    getLastPreprocessed() {
      return "preprocessed-body";
    },
    getCacheBust() {
      return 7;
    },
  };

  const handle = new WasmVmHandle({
    loader,
    defaultSourceText: "매틱:움직씨 = { x <- 1. }.",
    seedU64: 123,
  });

  const updatedState = await handle.updateLogic("#이름: 테스트\n매틱:움직씨 = { x <- 2. }.");
  assert(updatedState?.schema === "seamgrim.state.v0", "wasm vm handle: update state schema");
  assert(handle.getParseWarnings().length === 1, "wasm vm handle: parse warnings retained");
  assert(
    handle.getDebugInfo().parseWarnings?.[0]?.code === "W_BLOCK_HEADER_COLON_DEPRECATED",
    "wasm vm handle: parse warnings in debug info",
  );
  assert(
    handle.getDebugInfo().runtimeDiags?.[0]?.code === "E_WASM_SET_RNG_SEED_FAILED",
    "wasm vm handle: seed failure runtime diag",
  );

  await handle.invalidate();
  assert(handle.getParseWarnings().length === 0, "wasm vm handle: parse warnings reset on invalidate");

  await handle.updateLogic("매틱:움직씨 = { x <- 3. }.");
  assert(handle.getParseWarnings().length === 0, "wasm vm handle: compat empty warnings");
  assert(handle.getDebugInfo().runtimeDiags?.length === 0, "wasm vm handle: runtime diags reset after next ensure");

  const missingSeedApiHandle = new WasmVmHandle({
    loader: {
      async ensure() {
        return createClient({ warnings: [], supportsSeed: false });
      },
      reset() {},
      getLastBuildInfo() {
        return "build-info";
      },
      getLastPreprocessed() {
        return "preprocessed-body";
      },
      getCacheBust() {
        return 11;
      },
    },
    defaultSourceText: "매틱:움직씨 = { x <- 4. }.",
    seedU64: 77,
  });
  await missingSeedApiHandle.updateLogic("매틱:움직씨 = { x <- 4. }.");
  assert(
    missingSeedApiHandle.getDebugInfo().runtimeDiags?.[0]?.code === "E_WASM_SET_RNG_SEED_API_MISSING",
    "wasm vm handle: seed api missing runtime diag",
  );

  const missingApiWarnings = handle.readParseWarnings({});
  assert(
    missingApiWarnings?.[0]?.code === "E_WASM_PARSE_WARNINGS_API_MISSING",
    "wasm vm handle: parse warnings api missing diag",
  );
  const readFailedWarnings = handle.readParseWarnings({
    parseWarningsParsed() {
      throw new Error("parse-warn-read-fail");
    },
  });
  assert(
    readFailedWarnings?.[0]?.code === "E_WASM_PARSE_WARNINGS_READ_FAILED",
    "wasm vm handle: parse warnings read failed diag",
  );

  console.log("seamgrim wasm vm runtime ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
