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
  updateThrows = false,
  stepThrows = false,
  columnsThrows = false,
  setParamThrows = false,
  hashThrows = false,
  stateThrows = false,
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
      if (updateThrows) {
        throw new Error("update-fail");
      }
      this._updates.push(String(body ?? ""));
    },
    getStateParsed() {
      if (stateThrows) {
        throw new Error("state-fail");
      }
      return sharedState;
    },
    resetParsed() {
      return sharedState;
    },
    stepOneParsed() {
      if (stepThrows) {
        throw new Error("step-fail");
      }
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
      if (columnsThrows) {
        throw new Error("columns-fail");
      }
      return { columns: [], row: [] };
    },
    setParamParsed(key, value) {
      if (setParamThrows) {
        throw new Error("set-param-fail");
      }
      return { ok: true, key, value };
    },
    getStateHash() {
      if (hashThrows) {
        throw new Error("hash-fail");
      }
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
    {
      code: 77,
      message: null,
      detail: 42,
      span: { start: "x", end: undefined },
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
    getLastBuildInfoDiag() {
      return {
        code: "E_WASM_BUILD_INFO_CALL_FAILED",
        message: "mock build-info failed",
        detail: "mock",
      };
    },
    getLastInitDiag() {
      return {
        code: "E_WASM_LOADER_MODULE_INIT_FAILED",
        message: "mock module init failed",
        detail: "mock",
      };
    },
    getLastPreprocessed() {
      return "preprocessed-body";
    },
    getLastPreprocessDiag() {
      return {
        code: "E_WASM_PREPROCESS_CALL_FAILED",
        message: "mock preprocess failed",
        detail: "mock",
      };
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
  assert(handle.getParseWarnings().length === 2, "wasm vm handle: parse warnings retained");
  assert(
    handle.getDebugInfo().parseWarnings?.[0]?.code === "W_BLOCK_HEADER_COLON_DEPRECATED",
    "wasm vm handle: parse warnings in debug info",
  );
  assert(
    handle.getDebugInfo().parseWarnings?.[1]?.code === "77",
    "wasm vm handle: parse warnings code normalized to string",
  );
  assert(
    handle.getDebugInfo().parseWarnings?.[1]?.detail === "42",
    "wasm vm handle: parse warnings detail normalized to string",
  );
  assert(
    handle.getDebugInfo().parseWarnings?.[1]?.span?.start === 0,
    "wasm vm handle: parse warnings span.start normalized",
  );
  assert(
    handle.getDebugInfo().runtimeDiags?.[0]?.code === "E_WASM_SET_RNG_SEED_FAILED",
    "wasm vm handle: seed failure runtime diag",
  );
  assert(
    handle.getDebugInfo().buildInfoDiag?.code === "E_WASM_BUILD_INFO_CALL_FAILED",
    "wasm vm handle: build info diag passthrough",
  );
  assert(
    handle.getDebugInfo().initDiag?.code === "E_WASM_LOADER_MODULE_INIT_FAILED",
    "wasm vm handle: init diag passthrough",
  );
  assert(
    handle.getDebugInfo().preprocessDiag?.code === "E_WASM_PREPROCESS_CALL_FAILED",
    "wasm vm handle: preprocess diag passthrough",
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
      getLastBuildInfoDiag() {
        return null;
      },
      getLastInitDiag() {
        return null;
      },
      getLastPreprocessed() {
        return "preprocessed-body";
      },
      getLastPreprocessDiag() {
        return null;
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

  const failClient = createClient({
    warnings: [],
    updateThrows: true,
    stepThrows: true,
    columnsThrows: true,
    setParamThrows: true,
    hashThrows: true,
    stateThrows: true,
  });
  const failHandle = new WasmVmHandle({
    loader: {
      async ensure() {
        return failClient;
      },
      reset() {},
      getLastBuildInfo() {
        return "";
      },
      getLastBuildInfoDiag() {
        return null;
      },
      getLastInitDiag() {
        return null;
      },
      getLastPreprocessed() {
        return "";
      },
      getLastPreprocessDiag() {
        return null;
      },
      getCacheBust() {
        return 13;
      },
    },
    defaultSourceText: "매틱:움직씨 = { x <- 1. }.",
  });

  let updateFailed = false;
  try {
    await failHandle.updateLogic("매틱:움직씨 = { x <- 1. }.");
  } catch (_) {
    updateFailed = true;
  }
  assert(updateFailed, "wasm vm handle: update failure path");
  assert(
    failHandle.getDebugInfo().runtimeDiags?.some((row) => row?.code === "E_WASM_UPDATELOGIC_FAILED"),
    "wasm vm handle: update failure diag",
  );

  let stepFailed = false;
  try {
    await failHandle.step({ n: 1 });
  } catch (_) {
    stepFailed = true;
  }
  assert(stepFailed, "wasm vm handle: step failure path");
  assert(
    failHandle.getDebugInfo().runtimeDiags?.some((row) => row?.code === "E_WASM_STEP_FAILED"),
    "wasm vm handle: step failure diag",
  );

  let columnsFailed = false;
  try {
    await failHandle.columns();
  } catch (_) {
    columnsFailed = true;
  }
  assert(columnsFailed, "wasm vm handle: columns failure path");
  assert(
    failHandle.getDebugInfo().runtimeDiags?.some((row) => row?.code === "E_WASM_COLUMNS_FAILED"),
    "wasm vm handle: columns failure diag",
  );

  let setParamFailed = false;
  try {
    await failHandle.setParam({ key: "k", value: 1 });
  } catch (_) {
    setParamFailed = true;
  }
  assert(setParamFailed, "wasm vm handle: setParam failure path");
  assert(
    failHandle.getDebugInfo().runtimeDiags?.some((row) => row?.code === "E_WASM_SET_PARAM_FAILED"),
    "wasm vm handle: setParam failure diag",
  );

  let hashFailed = false;
  try {
    await failHandle.getStateHash();
  } catch (_) {
    hashFailed = true;
  }
  assert(hashFailed, "wasm vm handle: state hash failure path");
  assert(
    failHandle.getDebugInfo().runtimeDiags?.some((row) => row?.code === "E_WASM_STATE_HASH_FAILED"),
    "wasm vm handle: state hash failure diag",
  );

  let stateJsonFailed = false;
  try {
    await failHandle.getStateJson();
  } catch (_) {
    stateJsonFailed = true;
  }
  assert(stateJsonFailed, "wasm vm handle: state json failure path");
  assert(
    failHandle.getDebugInfo().runtimeDiags?.some((row) => row?.code === "E_WASM_GET_STATE_JSON_FAILED"),
    "wasm vm handle: state json failure diag",
  );

  console.log("seamgrim wasm vm runtime ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
