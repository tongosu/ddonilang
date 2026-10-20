import { preprocessDdnText } from "./ddn_preprocess.js";

function normalizeWasmModulePath(wasmUrl) {
  if (typeof wasmUrl !== "string" || !wasmUrl.trim()) {
    return "../wasm/ddonirang_tool.js";
  }
  if (wasmUrl.endsWith(".js")) return wasmUrl;
  return "../wasm/ddonirang_tool.js";
}

function withCacheBust(modulePath, cacheBust) {
  const sep = modulePath.includes("?") ? "&" : "?";
  return `${modulePath}${sep}v=${cacheBust}`;
}

function parseJsonText(text, label) {
  try {
    return JSON.parse(String(text ?? ""));
  } catch (err) {
    throw new Error(`${label} JSON 파싱 실패: ${String(err?.message ?? err)}`);
  }
}

function buildCanonDiag(code, message, detail = "") {
  return {
    code,
    message,
    detail: String(detail ?? ""),
  };
}

function prepareCanonSourceForUi(sourceText) {
  const source = String(sourceText ?? "");
  if (!source.trim()) return source;
  const normalized = source.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  const needsUiPreprocess = /(^|\n)\s*보임\s*\{/u.test(normalized);
  if (!needsUiPreprocess) {
    return source;
  }
  const pre = preprocessDdnText(source);
  const body = String(pre?.bodyText ?? "");
  return body.trim() ? body : source;
}

export async function createWasmCanon({
  wasmUrl = "../wasm/ddonirang_tool.js",
  cacheBust = Date.now(),
  initInput = undefined,
  moduleFactory = undefined,
} = {}) {
  const modulePath = normalizeWasmModulePath(wasmUrl);
  let wasmModule = null;
  let buildInfo = "";
  let lastInitDiag = null;
  let lastBuildInfoDiag = null;
  let lastPreprocessDiag = null;
  let lastCanonDiag = null;

  async function ensureModule() {
    if (wasmModule) return wasmModule;
    let loadedModule = null;
    try {
      if (typeof moduleFactory === "function") {
        loadedModule = await moduleFactory({
          modulePath,
          cacheBust,
          initInput,
        });
      } else {
        loadedModule = await import(withCacheBust(modulePath, cacheBust));
      }
    } catch (err) {
      lastInitDiag = buildCanonDiag(
        "E_WASM_CANON_MODULE_LOAD_FAILED",
        "wasm canonical module 로드에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
    if (typeof loadedModule?.default === "function") {
      try {
        if (initInput === undefined || initInput === null) {
          await loadedModule.default();
        } else {
          await loadedModule.default({ module_or_path: initInput });
        }
      } catch (err) {
        wasmModule = null;
        lastInitDiag = buildCanonDiag(
          "E_WASM_CANON_MODULE_INIT_FAILED",
          "wasm canonical module 초기화에 실패했습니다.",
          err?.message ?? String(err ?? ""),
        );
        throw err;
      }
    }
    wasmModule = loadedModule;
    lastInitDiag = null;
    if (typeof wasmModule.wasm_build_info === "function") {
      try {
        buildInfo = String(wasmModule.wasm_build_info() ?? "");
        lastBuildInfoDiag = null;
      } catch (err) {
        buildInfo = "";
        lastBuildInfoDiag = buildCanonDiag(
          "E_WASM_CANON_BUILD_INFO_CALL_FAILED",
          "wasm_canon build_info 호출에 실패했습니다.",
          err?.message ?? String(err ?? ""),
        );
      }
    }
    return wasmModule;
  }

  async function canonFlatJson(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_canon_flat_json !== "function") {
      lastCanonDiag = buildCanonDiag(
        "E_WASM_CANON_EXPORT_MISSING",
        "wasm_canon_flat_json export 누락",
        "flat",
      );
      throw new Error("wasm_canon_flat_json export 누락");
    }
    try {
      const normalizedSource = prepareCanonSourceForUi(sourceText);
      const parsed = parseJsonText(mod.wasm_canon_flat_json(normalizedSource), "flat");
      lastCanonDiag = null;
      return parsed;
    } catch (err) {
      lastCanonDiag = buildCanonDiag(
        "E_WASM_CANON_JSON_PARSE_FAILED",
        "flat canonical JSON 파싱에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async function canonMaegimPlan(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_canon_maegim_plan !== "function") {
      lastCanonDiag = buildCanonDiag(
        "E_WASM_CANON_EXPORT_MISSING",
        "wasm_canon_maegim_plan export 누락",
        "maegim",
      );
      throw new Error("wasm_canon_maegim_plan export 누락");
    }
    try {
      const normalizedSource = prepareCanonSourceForUi(sourceText);
      const parsed = parseJsonText(mod.wasm_canon_maegim_plan(normalizedSource), "maegim");
      lastCanonDiag = null;
      return parsed;
    } catch (err) {
      lastCanonDiag = buildCanonDiag(
        "E_WASM_CANON_JSON_PARSE_FAILED",
        "maegim canonical JSON 파싱에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async function canonAlrimPlan(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_canon_alrim_plan !== "function") {
      lastCanonDiag = buildCanonDiag(
        "E_WASM_CANON_EXPORT_MISSING",
        "wasm_canon_alrim_plan export 누락",
        "alrim",
      );
      throw new Error("wasm_canon_alrim_plan export 누락");
    }
    try {
      const normalizedSource = prepareCanonSourceForUi(sourceText);
      const parsed = parseJsonText(mod.wasm_canon_alrim_plan(normalizedSource), "alrim");
      lastCanonDiag = null;
      return parsed;
    } catch (err) {
      lastCanonDiag = buildCanonDiag(
        "E_WASM_CANON_JSON_PARSE_FAILED",
        "alrim canonical JSON 파싱에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async function canonBlockEditorPlan(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_canon_block_editor_plan !== "function") {
      lastCanonDiag = buildCanonDiag(
        "E_WASM_CANON_EXPORT_MISSING",
        "wasm_canon_block_editor_plan export 누락",
        "block_editor",
      );
      throw new Error("wasm_canon_block_editor_plan export 누락");
    }
    try {
      const normalizedSource = prepareCanonSourceForUi(sourceText);
      const parsed = parseJsonText(
        mod.wasm_canon_block_editor_plan(normalizedSource),
        "block_editor",
      );
      lastCanonDiag = null;
      return parsed;
    } catch (err) {
      lastCanonDiag = buildCanonDiag(
        "E_WASM_CANON_JSON_PARSE_FAILED",
        "block_editor canonical JSON 파싱에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  async function preprocessSource(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_preprocess_source !== "function") {
      lastPreprocessDiag = buildCanonDiag(
        "E_WASM_CANON_PREPROCESS_API_MISSING",
        "wasm_preprocess_source API가 없습니다.",
      );
      return String(sourceText ?? "");
    }
    try {
      const text = String(mod.wasm_preprocess_source(String(sourceText ?? "")) ?? "");
      lastPreprocessDiag = null;
      return text;
    } catch (err) {
      lastPreprocessDiag = buildCanonDiag(
        "E_WASM_CANON_PREPROCESS_CALL_FAILED",
        "wasm_preprocess_source 호출에 실패했습니다.",
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  return {
    ensureModule,
    canonFlatJson,
    canonMaegimPlan,
    canonAlrimPlan,
    canonBlockEditorPlan,
    preprocessSource,
    getBuildInfo() {
      return buildInfo;
    },
    getLastInitDiag() {
      return lastInitDiag ? { ...lastInitDiag } : null;
    },
    getLastBuildInfoDiag() {
      return lastBuildInfoDiag ? { ...lastBuildInfoDiag } : null;
    },
    getLastPreprocessDiag() {
      return lastPreprocessDiag ? { ...lastPreprocessDiag } : null;
    },
    getLastCanonDiag() {
      return lastCanonDiag ? { ...lastCanonDiag } : null;
    },
  };
}
