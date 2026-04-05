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

export async function createWasmCanon({
  wasmUrl = "../wasm/ddonirang_tool.js",
  cacheBust = Date.now(),
  initInput = undefined,
} = {}) {
  const modulePath = normalizeWasmModulePath(wasmUrl);
  let wasmModule = null;
  let buildInfo = "";

  async function ensureModule() {
    if (wasmModule) return wasmModule;
    wasmModule = await import(withCacheBust(modulePath, cacheBust));
    if (typeof wasmModule.default === "function") {
      if (initInput === undefined || initInput === null) {
        await wasmModule.default();
      } else {
        await wasmModule.default({ module_or_path: initInput });
      }
    }
    if (typeof wasmModule.wasm_build_info === "function") {
      buildInfo = String(wasmModule.wasm_build_info() ?? "");
    }
    return wasmModule;
  }

  async function canonFlatJson(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_canon_flat_json !== "function") {
      throw new Error("wasm_canon_flat_json export 누락");
    }
    return parseJsonText(mod.wasm_canon_flat_json(String(sourceText ?? "")), "flat");
  }

  async function canonMaegimPlan(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_canon_maegim_plan !== "function") {
      throw new Error("wasm_canon_maegim_plan export 누락");
    }
    return parseJsonText(mod.wasm_canon_maegim_plan(String(sourceText ?? "")), "maegim");
  }

  async function canonAlrimPlan(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_canon_alrim_plan !== "function") {
      throw new Error("wasm_canon_alrim_plan export 누락");
    }
    return parseJsonText(mod.wasm_canon_alrim_plan(String(sourceText ?? "")), "alrim");
  }

  async function canonBlockEditorPlan(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_canon_block_editor_plan !== "function") {
      throw new Error("wasm_canon_block_editor_plan export 누락");
    }
    return parseJsonText(mod.wasm_canon_block_editor_plan(String(sourceText ?? "")), "block_editor");
  }

  async function preprocessSource(sourceText) {
    const mod = await ensureModule();
    if (typeof mod.wasm_preprocess_source !== "function") {
      return String(sourceText ?? "");
    }
    return String(mod.wasm_preprocess_source(String(sourceText ?? "")) ?? "");
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
  };
}
