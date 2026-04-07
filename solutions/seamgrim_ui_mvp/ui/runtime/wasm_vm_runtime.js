import {
  createWasmLoader,
  stripMetaHeader,
} from "../wasm_page_common.js";
import { preprocessDdnText } from "./ddn_preprocess.js";
import { buildObservationManifest } from "./observation_manifest.js";

function normalizeStatusLines(lines) {
  if (Array.isArray(lines)) return lines;
  if (typeof lines === "string") return [lines];
  return [];
}

function normalizeWasmModulePath(wasmUrl) {
  if (typeof wasmUrl !== "string" || !wasmUrl.trim()) {
    return "../wasm/ddonirang_tool.js";
  }
  if (wasmUrl.endsWith(".js")) return wasmUrl;
  return "../wasm/ddonirang_tool.js";
}

function buildParseWarningsReadDiag(code, message, detail = "") {
  return [
    {
      code,
      message,
      detail: String(detail ?? ""),
      span: { start: 0, end: 0 },
    },
  ];
}

function normalizeParseWarningRow(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  const spanRaw = row.span && typeof row.span === "object" ? row.span : {};
  const startNum = Number(spanRaw.start);
  const endNum = Number(spanRaw.end);
  return {
    code: String(row.code ?? "W_WASM_PARSE_WARNING_UNKNOWN"),
    message: String(row.message ?? ""),
    detail: String(row.detail ?? ""),
    span: {
      start: Number.isFinite(startNum) ? startNum : 0,
      end: Number.isFinite(endNum) ? endNum : 0,
    },
  };
}

function buildRuntimeDiag(code, message, detail = "") {
  return {
    code,
    message,
    detail: String(detail ?? ""),
  };
}

export class WasmVmHandle {
  constructor({
    loader,
    defaultSourceText = "",
    seedU64 = undefined,
  } = {}) {
    this.loader = loader;
    this.defaultSourceText = String(defaultSourceText ?? "");
    this.seedU64 = Number.isFinite(Number(seedU64)) ? Number(seedU64) : null;
    this.client = null;
    this.lastSourceText = "";
    this.lastBodyText = "";
    this.lastPragmas = [];
    this.lastDiags = [];
    this.lastObservationManifest = null;
    this.lastParseWarnings = [];
    this.lastRuntimeDiags = [];
  }

  addRuntimeDiag(code, message, detail = "") {
    this.lastRuntimeDiags = [
      ...this.lastRuntimeDiags,
      buildRuntimeDiag(code, message, detail),
    ];
  }

  async ensureClientForOp(sourceText, code, message) {
    try {
      return await this.ensureClient(sourceText);
    } catch (err) {
      this.addRuntimeDiag(code, message, err?.message ?? String(err ?? ""));
      throw err;
    }
  }

  preprocess(sourceText) {
    const source = String(sourceText ?? this.lastSourceText ?? this.defaultSourceText);
    const pre = preprocessDdnText(source);
    const bodyText = stripMetaHeader(pre.bodyText ?? "");
    this.lastSourceText = source;
    this.lastBodyText = bodyText;
    this.lastPragmas = Array.isArray(pre.pragmas) ? pre.pragmas : [];
    this.lastDiags = Array.isArray(pre.diags) ? pre.diags : [];
    return {
      bodyText: this.lastBodyText,
      pragmas: this.lastPragmas,
      diags: this.lastDiags,
    };
  }

  async ensureClient(sourceText) {
    const pre = this.preprocess(sourceText);
    const client = await this.loader.ensure(pre.bodyText);
    this.client = client;
    this.lastRuntimeDiags = [];
    if (this.seedU64 !== null) {
      if (typeof client?.setRngSeed === "function") {
        try {
          client.setRngSeed(this.seedU64);
        } catch (err) {
          this.addRuntimeDiag(
            "E_WASM_SET_RNG_SEED_FAILED",
            "setRngSeed 호출에 실패했습니다.",
            err?.message ?? String(err ?? ""),
          );
        }
      } else {
        this.addRuntimeDiag(
          "E_WASM_SET_RNG_SEED_API_MISSING",
          "setRngSeed API가 없어 시드를 적용할 수 없습니다.",
        );
      }
      this.seedU64 = null;
    }
    this.lastParseWarnings = this.readParseWarnings(client);
    return client;
  }

  readParseWarnings(client) {
    if (!client || typeof client !== "object") {
      return buildParseWarningsReadDiag(
        "E_WASM_PARSE_WARNINGS_CLIENT_MISSING",
        "parse warnings를 읽을 client가 없습니다.",
      );
    }
    if (typeof client.parseWarningsParsed !== "function") {
      return buildParseWarningsReadDiag(
        "E_WASM_PARSE_WARNINGS_API_MISSING",
        "parseWarningsParsed API가 없습니다.",
      );
    }
    try {
      const warnings = client.parseWarningsParsed();
      if (!Array.isArray(warnings)) {
        return buildParseWarningsReadDiag(
          "E_WASM_PARSE_WARNINGS_PAYLOAD_INVALID",
          "parseWarningsParsed 결과가 배열이 아닙니다.",
          typeof warnings,
        );
      }
      return warnings.map(normalizeParseWarningRow);
    } catch (err) {
      return buildParseWarningsReadDiag(
        "E_WASM_PARSE_WARNINGS_READ_FAILED",
        "parseWarningsParsed 호출에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
    }
  }

  attachObservationManifest(state) {
    if (!state || typeof state !== "object") return state;
    const nativeManifest =
      state.observation_manifest && typeof state.observation_manifest === "object"
        ? state.observation_manifest
        : null;
    if (nativeManifest && String(nativeManifest.schema ?? "").trim()) {
      this.lastObservationManifest = nativeManifest;
      return state;
    }
    const channels = Array.isArray(state.channels) ? state.channels : [];
    const manifest = buildObservationManifest({
      channels,
      pragmas: this.lastPragmas,
    });
    this.lastObservationManifest = manifest;
    return {
      ...state,
      observation_manifest: manifest,
    };
  }

  async updateLogic(ddnSourceText) {
    const client = await this.ensureClientForOp(
      ddnSourceText,
      "E_WASM_UPDATE_ENSURE_FAILED",
      "updateLogic 진입을 위한 wasm client 준비에 실패했습니다.",
    );
    try {
      client.updateLogic(this.lastBodyText);
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_UPDATELOGIC_FAILED",
        "updateLogic 호출에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
    this.lastParseWarnings = this.readParseWarnings(client);
    try {
      return this.attachObservationManifest(client.getStateParsed());
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_GETSTATE_AFTER_UPDATE_FAILED",
        "updateLogic 이후 상태 조회에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async reset({ keepParams = false } = {}) {
    const client =
      this.client ??
      (await this.ensureClientForOp(
        this.defaultSourceText,
        "E_WASM_RESET_ENSURE_FAILED",
        "reset 진입을 위한 wasm client 준비에 실패했습니다.",
      ));
    try {
      return this.attachObservationManifest(client.resetParsed(Boolean(keepParams)));
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_RESET_FAILED",
        "resetParsed 호출에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async step({ n = 1, input = null, sourceText = null } = {}) {
    const client = await this.ensureClientForOp(
      sourceText ?? this.lastSourceText ?? this.defaultSourceText,
      "E_WASM_STEP_ENSURE_FAILED",
      "step 진입을 위한 wasm client 준비에 실패했습니다.",
    );
    const countRaw = Number(n);
    const count = Number.isFinite(countRaw) && countRaw > 0 ? Math.floor(countRaw) : 1;
    let state = null;
    for (let i = 0; i < count; i += 1) {
      try {
        if (
          input &&
          typeof input === "object" &&
          typeof client.stepOneWithInputParsed === "function"
        ) {
          state = client.stepOneWithInputParsed(
            Number(input.keys ?? 0),
            String(input.lastKey ?? ""),
            Number(input.px ?? 0),
            Number(input.py ?? 0),
            Number(input.dt ?? 0),
          );
        } else {
          state = client.stepOneParsed();
        }
      } catch (err) {
        this.addRuntimeDiag(
          "E_WASM_STEP_FAILED",
          "stepOne 호출에 실패했습니다.",
          `i=${i}; ${String(err?.message ?? err ?? "")}`,
        );
        throw err;
      }
    }
    try {
      return this.attachObservationManifest(state);
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_STEP_STATE_ATTACH_FAILED",
        "step 결과 상태 부착에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async columns() {
    const client =
      this.client ??
      (await this.ensureClientForOp(
        this.defaultSourceText,
        "E_WASM_COLUMNS_ENSURE_FAILED",
        "columns 진입을 위한 wasm client 준비에 실패했습니다.",
      ));
    let payload;
    try {
      payload = client.columnsParsed();
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_COLUMNS_FAILED",
        "columnsParsed 호출에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
    try {
      const nativeManifest =
        payload?.observation_manifest && typeof payload.observation_manifest === "object"
          ? payload.observation_manifest
          : null;
      if (nativeManifest && String(nativeManifest.schema ?? "").trim()) {
        this.lastObservationManifest = nativeManifest;
        return payload;
      }
      const columns = Array.isArray(payload?.columns) ? payload.columns : [];
      const manifest = buildObservationManifest({
        channels: columns,
        pragmas: this.lastPragmas,
      });
      this.lastObservationManifest = manifest;
      return {
        ...payload,
        observation_manifest: manifest,
      };
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_COLUMNS_ATTACH_FAILED",
        "columns 결과 상태 부착에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async setParam({ key, value } = {}) {
    const client =
      this.client ??
      (await this.ensureClientForOp(
        this.defaultSourceText,
        "E_WASM_SET_PARAM_ENSURE_FAILED",
        "setParam 진입을 위한 wasm client 준비에 실패했습니다.",
      ));
    try {
      return client.setParamParsed(String(key ?? ""), value);
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_SET_PARAM_FAILED",
        "setParamParsed 호출에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async getStateHash() {
    const client =
      this.client ??
      (await this.ensureClientForOp(
        this.defaultSourceText,
        "E_WASM_STATE_HASH_ENSURE_FAILED",
        "getStateHash 진입을 위한 wasm client 준비에 실패했습니다.",
      ));
    try {
      return client.getStateHash();
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_STATE_HASH_FAILED",
        "getStateHash 호출에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async getStateJson() {
    const client =
      this.client ??
      (await this.ensureClientForOp(
        this.defaultSourceText,
        "E_WASM_GET_STATE_JSON_ENSURE_FAILED",
        "getStateJson 진입을 위한 wasm client 준비에 실패했습니다.",
      ));
    try {
      return this.attachObservationManifest(client.getStateParsed());
    } catch (err) {
      this.addRuntimeDiag(
        "E_WASM_GET_STATE_JSON_FAILED",
        "getStateParsed 호출에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  getParseWarnings() {
    return Array.isArray(this.lastParseWarnings) ? [...this.lastParseWarnings] : [];
  }

  getDebugInfo() {
    return {
      buildInfo: this.loader.getLastBuildInfo?.() ?? "",
      buildInfoDiag: this.loader.getLastBuildInfoDiag?.() ?? null,
      initDiag: this.loader.getLastInitDiag?.() ?? null,
      preprocessed: this.loader.getLastPreprocessed?.() ?? this.lastBodyText ?? "",
      preprocessDiag: this.loader.getLastPreprocessDiag?.() ?? null,
      pragmas: this.lastPragmas,
      diags: this.lastDiags,
      parseWarnings: this.getParseWarnings(),
      runtimeDiags: Array.isArray(this.lastRuntimeDiags) ? [...this.lastRuntimeDiags] : [],
      observationManifest: this.lastObservationManifest,
      cacheBust: this.loader.getCacheBust?.() ?? null,
    };
  }

  invalidate() {
    this.client = null;
    this.lastSourceText = "";
    this.lastBodyText = "";
    this.lastPragmas = [];
    this.lastDiags = [];
    this.lastObservationManifest = null;
    this.lastParseWarnings = [];
    this.lastRuntimeDiags = [];
    this.loader.reset();
  }
}

export async function createWasmVm({
  wasmUrl = "./wasm/ddonirang_tool.js",
  ddnSourceText = "",
  seedU64 = undefined,
  cacheBust = Date.now(),
  setStatus = null,
  clearStatusError = null,
  missingExportMessage = "DdnWasmVm export missing",
  formatReadyStatus = null,
  formatFallbackStatus = null,
} = {}) {
  const modulePath = normalizeWasmModulePath(wasmUrl);
  const loader = createWasmLoader({
    cacheBust,
    modulePath,
    wrapperPath: "./wasm_ddn_wrapper.js",
    setStatus: (lines) => {
      if (typeof setStatus === "function") {
        setStatus(normalizeStatusLines(lines));
      }
    },
    clearStatusError: typeof clearStatusError === "function" ? clearStatusError : undefined,
    missingExportMessage,
    formatReadyStatus,
    formatFallbackStatus,
  });
  const handle = new WasmVmHandle({
    loader,
    defaultSourceText: ddnSourceText,
    seedU64,
  });
  if (String(ddnSourceText ?? "").trim()) {
    await handle.ensureClient(ddnSourceText);
  }
  return handle;
}
