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
  attachThrowsInState = false,
  columnsAttachThrows = false,
  resetThrows = false,
  updateThrows = false,
  stepThrows = false,
  columnsThrows = false,
  setParamThrows = false,
  hashThrows = false,
  stateThrows = false,
} = {}) {
  const sharedState =
    state ??
    (() => {
      const base = {
        schema: "seamgrim.state.v0",
        tick_id: 0,
        channels: [],
        row: [],
        resources: { json: {}, fixed64: {}, handle: {}, value: {} },
        observation_manifest: {
          schema: "ddn.observation_manifest.v0",
          version: "20.6.33",
          nodes: [],
          params: [],
          traces: [],
          diagnostics: [],
          default_pivot: { x_channel: "", y_channels: [] },
        },
      };
      if (attachThrowsInState) {
        Object.defineProperty(base, "observation_manifest", {
          get() {
            throw new Error("state-attach-fail");
          },
        });
      }
      return base;
    })();
  const client = {
    _updates: [],
    _inputSteps: [],
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
      if (resetThrows) {
        throw new Error("reset-fail");
      }
      return sharedState;
    },
    stepOneParsed() {
      if (stepThrows) {
        throw new Error("step-fail");
      }
      return sharedState;
    },
    columnsParsed() {
      if (columnsThrows) {
        throw new Error("columns-fail");
      }
      if (columnsAttachThrows) {
        const payload = { columns: [], row: [] };
        Object.defineProperty(payload, "observation_manifest", {
          get() {
            throw new Error("columns-attach-fail");
          },
        });
        return payload;
      }
      return {
        columns: [],
        row: [],
        observation_manifest: {
          schema: "ddn.observation_manifest.v0",
          version: "20.6.33",
          nodes: [],
          params: [],
          traces: [],
          diagnostics: [],
          default_pivot: { x_channel: "", y_channels: [] },
        },
      };
    },
    setParamParsed(key, value) {
      if (setParamThrows) {
        throw new Error("set-param-fail");
      }
      return { ok: true, key, value };
    },
    setParamFixed64StringParsed(key, value) {
      if (setParamThrows) throw new Error("set-param-fail");
      return { ok: true, key, raw_i64: value };
    },
    getStateHash() {
      if (hashThrows) {
        throw new Error("hash-fail");
      }
      return "blake3:test";
    },
  };
  if (supportsStepWithInput) {
    client.inputStateParsed = () => ({ alpha_host_dt: "1" });
    client.fixed64FromDecimalV1Parsed = function fixed64FromDecimalV1Parsed(dtText) {
      if (String(dtText) === "invalid") throw new Error("E_FIXED64_UNREPRESENTABLE_CONVERSION");
      assert(String(dtText) === "1", "runtime must parse the Rust host policy, not a JS dt default");
      return { ok: true, value: { high_i32: 1, low_u32: 0 } };
    };
    client.stepOneWithInputFixed64V1Parsed = function stepOneWithInputFixed64V1Parsed(keys, lastKey, px, py, high, low) {
      this._inputSteps.push({ keys, lastKey, px, py, high, low });
      return {
        ...sharedState,
        input: { keys, lastKey, px, py, high, low },
      };
    };
  }
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
  const numericHandle = new WasmVmHandle();
  const admissionCalls = [];
  numericHandle.client = {
    fixed64FromDecimalV1Parsed(text) {
      admissionCalls.push(text);
      return { ok: true, value: { high_i32: -13, low_u32: 3006477107 } };
    },
    setParamFixed64StringParsed(key, raw) { return { key, raw }; },
    setParamParsed() { throw new Error("numeric controls must never use the legacy JS-number setter"); },
  };
  const numericResult = await numericHandle.setParam({ key: "중력", value: -12.3 });
  assert(admissionCalls.join() === "-12.3", "numeric admission must be owned by Rust");
  assert(numericResult.raw === "-52828097741", "signed Q31.32 carrier must retain every returned bit");
  let rejectedInfinity = false;
  try { await numericHandle.setParam({ key: "중력", value: Infinity }); } catch { rejectedInfinity = true; }
  assert(rejectedInfinity, "non-finite control must fail before admission");

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
    resetThrows: true,
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

  let resetFailed = false;
  try {
    await failHandle.reset({ keepParams: false });
  } catch (_) {
    resetFailed = true;
  }
  assert(resetFailed, "wasm vm handle: reset failure path");
  assert(
    failHandle.getDebugInfo().runtimeDiags?.some((row) => row?.code === "E_WASM_RESET_FAILED"),
    "wasm vm handle: reset failure diag",
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

  const incompleteInputClient = createClient({ warnings: [] });
  const incompleteInputHandle = new WasmVmHandle({
    loader: {
      async ensure() {
        return incompleteInputClient;
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
        return 19;
      },
    },
    defaultSourceText: "매틱:움직씨 = { x <- 1. }.",
  });
  let incompleteInputRejected = false;
  try {
    await incompleteInputHandle.step({
      n: 1,
      input: { keys: 1, lastKey: "ArrowRight", px: 4 },
    });
  } catch (_) {
    incompleteInputRejected = true;
  }
  assert(incompleteInputRejected, "wasm vm handle: incomplete typed input rejected before dispatch");
  assert(
    incompleteInputClient._inputSteps.length === 0,
    "wasm vm handle: incomplete typed input has no frame publication",
  );
  assert(
    incompleteInputHandle.getDebugInfo().runtimeDiags?.some(
      (row) => row?.code === "E_WASM_STEP_FAILED",
    ),
    "wasm vm handle: incomplete typed input uses existing step failure",
  );
  const validInputState = await incompleteInputHandle.step({
    n: 1,
    input: { keys: 1, lastKey: "ArrowRight", px: 4, py: 8 },
  });
  assert(
    validInputState?.input?.high === 1 && validInputState?.input?.low === 0 && incompleteInputClient._inputSteps.length === 1,
    "wasm vm handle: complete typed input dispatches exactly once",
  );
  const frozenInput = Object.freeze({ keys: 2, lastKey: "ArrowLeft", px: 3, py: 5, dtText: "1" });
  const frozenInputBefore = JSON.stringify(frozenInput);
  await incompleteInputHandle.step({ n: 1, input: frozenInput });
  assert(
    JSON.stringify(frozenInput) === frozenInputBefore,
    "wasm vm handle: typed input payload remains immutable",
  );
  const beforeRejected = incompleteInputClient._inputSteps.length;
  for (const dtText of ["0.015625", "0.016666", "0.016666666666666666"]) {
    let rejected = false;
    try { await incompleteInputHandle.step({ n: 1, input: { ...frozenInput, dtText } }); }
    catch (error) { rejected = String(error).includes("E_ALPHA_HOST_DT_CONFLICT"); }
    assert(rejected, `non-Alpha host dt must be rejected: ${dtText}`);
  }
  assert(incompleteInputClient._inputSteps.length === beforeRejected, "conflicting dt must not publish a frame");
  delete incompleteInputClient.inputStateParsed;
  let policyRejected = false;
  try { await incompleteInputHandle.step({ n: 1, input: frozenInput }); }
  catch (error) { policyRejected = String(error).includes("E_ALPHA_HOST_DT_POLICY_MISSING"); }
  assert(policyRejected, "missing Rust host policy must fail without a JS default");
  incompleteInputClient.inputStateParsed = () => ({ alpha_host_dt: "1" });

  const noTypedInputApiClient = createClient({ warnings: [], supportsStepWithInput: false });
  const noTypedInputApiHandle = new WasmVmHandle({
    loader: {
      async ensure() {
        return noTypedInputApiClient;
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
        return 31;
      },
    },
    defaultSourceText: "매틱:움직씨 = { x <- 1. }.",
  });
  let noTypedInputApiRejected = false;
  try {
    await noTypedInputApiHandle.step({
      n: 1,
      input: { keys: 1, lastKey: "ArrowRight", px: 4, py: 8, dtText: "0.016" },
    });
  } catch (_) {
    noTypedInputApiRejected = true;
  }
  assert(noTypedInputApiRejected, "wasm vm handle: typed input API absence rejects before fallback");
  assert(
    noTypedInputApiClient._inputSteps.length === 0,
    "wasm vm handle: typed input API absence has no frame publication",
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

  const stepAttachFailHandle = new WasmVmHandle({
    loader: {
      async ensure() {
        return createClient({ warnings: [], attachThrowsInState: true });
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
        return 23;
      },
    },
    defaultSourceText: "매틱:움직씨 = { x <- 1. }.",
  });
  let stepAttachFailed = false;
  try {
    await stepAttachFailHandle.step({ n: 1 });
  } catch (_) {
    stepAttachFailed = true;
  }
  assert(stepAttachFailed, "wasm vm handle: step attach failure path");
  assert(
    stepAttachFailHandle.getDebugInfo().runtimeDiags?.some(
      (row) => row?.code === "E_WASM_STEP_STATE_ATTACH_FAILED",
    ),
    "wasm vm handle: step attach failure diag",
  );

  const columnsAttachFailHandle = new WasmVmHandle({
    loader: {
      async ensure() {
        return createClient({ warnings: [], columnsAttachThrows: true });
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
        return 29;
      },
    },
    defaultSourceText: "매틱:움직씨 = { x <- 1. }.",
  });
  let columnsAttachFailed = false;
  try {
    await columnsAttachFailHandle.columns();
  } catch (_) {
    columnsAttachFailed = true;
  }
  assert(columnsAttachFailed, "wasm vm handle: columns attach failure path");
  assert(
    columnsAttachFailHandle.getDebugInfo().runtimeDiags?.some(
      (row) => row?.code === "E_WASM_COLUMNS_ATTACH_FAILED",
    ),
    "wasm vm handle: columns attach failure diag",
  );

  const ensureFailHandle = new WasmVmHandle({
    loader: {
      async ensure() {
        throw new Error("ensure-fail");
      },
      reset() {},
      getLastBuildInfo() {
        return "";
      },
      getLastBuildInfoDiag() {
        return null;
      },
      getLastInitDiag() {
        return { code: "E_WASM_LOADER_MODULE_LOAD_FAILED", message: "load-fail", detail: "load-fail" };
      },
      getLastPreprocessed() {
        return "";
      },
      getLastPreprocessDiag() {
        return null;
      },
      getCacheBust() {
        return 19;
      },
    },
    defaultSourceText: "매틱:움직씨 = { x <- 1. }.",
  });

  let ensureFailed = false;
  try {
    await ensureFailHandle.updateLogic("매틱:움직씨 = { x <- 1. }.");
  } catch (_) {
    ensureFailed = true;
  }
  assert(ensureFailed, "wasm vm handle: ensure failure path");
  assert(
    ensureFailHandle.getDebugInfo().runtimeDiags?.some((row) => row?.code === "E_WASM_UPDATE_ENSURE_FAILED"),
    "wasm vm handle: ensure failure diag",
  );

  console.log("seamgrim wasm vm runtime ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
