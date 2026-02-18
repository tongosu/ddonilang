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
    return "./wasm/ddonirang_tool.js";
  }
  if (wasmUrl.endsWith(".js")) return wasmUrl;
  return "./wasm/ddonirang_tool.js";
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
    if (this.seedU64 !== null && typeof client?.setRngSeed === "function") {
      try {
        client.setRngSeed(this.seedU64);
      } catch (_) {
        // keep runtime usable even when seed API is not available
      }
      this.seedU64 = null;
    }
    return client;
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

  getDebugInfo() {
    return {
      buildInfo: this.loader.getLastBuildInfo?.() ?? "",
      preprocessed: this.loader.getLastPreprocessed?.() ?? this.lastBodyText ?? "",
      pragmas: this.lastPragmas,
      diags: this.lastDiags,
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
