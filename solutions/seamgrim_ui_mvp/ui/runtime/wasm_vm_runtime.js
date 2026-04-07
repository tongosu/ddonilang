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
          this.lastRuntimeDiags = [
            ...this.lastRuntimeDiags,
            buildRuntimeDiag(
              "E_WASM_SET_RNG_SEED_FAILED",
              "setRngSeed 호출에 실패했습니다.",
              err?.message ?? String(err ?? ""),
            ),
          ];
        }
      } else {
        this.lastRuntimeDiags = [
          ...this.lastRuntimeDiags,
          buildRuntimeDiag(
            "E_WASM_SET_RNG_SEED_API_MISSING",
            "setRngSeed API가 없어 시드를 적용할 수 없습니다.",
          ),
        ];
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
      return warnings;
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
    const client = await this.ensureClient(ddnSourceText);
    client.updateLogic(this.lastBodyText);
    this.lastParseWarnings = this.readParseWarnings(client);
    return this.attachObservationManifest(client.getStateParsed());
  }

  async reset({ keepParams = false } = {}) {
    const client = this.client ?? (await this.ensureClient(this.defaultSourceText));
    return this.attachObservationManifest(client.resetParsed(Boolean(keepParams)));
  }

  async step({ n = 1, input = null, sourceText = null } = {}) {
    const client = await this.ensureClient(sourceText ?? this.lastSourceText ?? this.defaultSourceText);
    const countRaw = Number(n);
    const count = Number.isFinite(countRaw) && countRaw > 0 ? Math.floor(countRaw) : 1;
    let state = null;
    for (let i = 0; i < count; i += 1) {
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
    }
    return this.attachObservationManifest(state);
  }

  async columns() {
    const client = this.client ?? (await this.ensureClient(this.defaultSourceText));
    const payload = client.columnsParsed();
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
  }

  async setParam({ key, value } = {}) {
    const client = this.client ?? (await this.ensureClient(this.defaultSourceText));
    return client.setParamParsed(String(key ?? ""), value);
  }

  async getStateHash() {
    const client = this.client ?? (await this.ensureClient(this.defaultSourceText));
    return client.getStateHash();
  }

  async getStateJson() {
    const client = this.client ?? (await this.ensureClient(this.defaultSourceText));
    return this.attachObservationManifest(client.getStateParsed());
  }

  getParseWarnings() {
    return Array.isArray(this.lastParseWarnings) ? [...this.lastParseWarnings] : [];
  }

  getDebugInfo() {
    return {
      buildInfo: this.loader.getLastBuildInfo?.() ?? "",
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
