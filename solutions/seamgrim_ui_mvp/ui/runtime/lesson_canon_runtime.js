import { createWasmCanon } from "./wasm_canon_runtime.js";

const DEFAULT_WASM_CANON_URL = "../wasm/ddonirang_tool.js";

function buildCanonDiag(code, message, detail = "") {
  return {
    code,
    message,
    detail: String(detail ?? ""),
  };
}

function pickRuntimeCanonDiag(runtime, fallbackError) {
  if (!runtime || typeof runtime !== "object") return null;
  const fromInit = typeof runtime.getLastInitDiag === "function" ? runtime.getLastInitDiag() : null;
  if (fromInit && typeof fromInit === "object") {
    return buildCanonDiag(
      String(fromInit.code ?? "E_WASM_CANON_RUNTIME_DIAG"),
      String(fromInit.message ?? "wasm init runtime 오류"),
      String(fromInit.detail ?? fallbackError ?? ""),
    );
  }
  const fromCanon = typeof runtime.getLastCanonDiag === "function" ? runtime.getLastCanonDiag() : null;
  if (fromCanon && typeof fromCanon === "object") {
    return buildCanonDiag(
      String(fromCanon.code ?? "E_WASM_CANON_RUNTIME_DIAG"),
      String(fromCanon.message ?? "wasm canon runtime 오류"),
      String(fromCanon.detail ?? fallbackError ?? ""),
    );
  }
  const fromPreprocess =
    typeof runtime.getLastPreprocessDiag === "function" ? runtime.getLastPreprocessDiag() : null;
  if (fromPreprocess && typeof fromPreprocess === "object") {
    return buildCanonDiag(
      String(fromPreprocess.code ?? "E_WASM_CANON_RUNTIME_DIAG"),
      String(fromPreprocess.message ?? "wasm preprocess runtime 오류"),
      String(fromPreprocess.detail ?? fallbackError ?? ""),
    );
  }
  const fromBuildInfo =
    typeof runtime.getLastBuildInfoDiag === "function" ? runtime.getLastBuildInfoDiag() : null;
  if (fromBuildInfo && typeof fromBuildInfo === "object") {
    return buildCanonDiag(
      String(fromBuildInfo.code ?? "E_WASM_CANON_RUNTIME_DIAG"),
      String(fromBuildInfo.message ?? "wasm build_info runtime 오류"),
      String(fromBuildInfo.detail ?? fallbackError ?? ""),
    );
  }
  return null;
}

function normalizeLessonPayload(lesson) {
  return lesson && typeof lesson === "object" ? { ...lesson } : {};
}

function normalizeFlatPlanInstance(row) {
  const name = String(row?.name ?? "").trim();
  const typeName = String(row?.type_name ?? "").trim();
  if (!name) return null;
  return {
    name,
    typeName: typeName || "-",
    label: typeName ? `${name}: ${typeName}` : name,
  };
}

function normalizeFlatPlanLink(row) {
  const srcInstance = String(row?.src_instance ?? "").trim();
  const srcPort = String(row?.src_port ?? "").trim();
  const dstInstance = String(row?.dst_instance ?? "").trim();
  const dstPort = String(row?.dst_port ?? "").trim();
  if (!srcInstance || !dstInstance) return null;
  const srcLabel = srcPort ? `${srcInstance}.${srcPort}` : srcInstance;
  const dstLabel = dstPort ? `${dstInstance}.${dstPort}` : dstInstance;
  return {
    srcInstance,
    srcPort,
    dstInstance,
    dstPort,
    label: `${srcLabel} -> ${dstLabel}`,
  };
}

export function summarizeFlatPlan(flatPlan) {
  if (!flatPlan || typeof flatPlan !== "object") return "구성: flat plan 없음";
  const schema = String(flatPlan.schema ?? "").trim();
  if (schema !== "ddn.guseong_flatten_plan.v1") {
    return "구성: flat plan 없음";
  }
  const instances = Array.isArray(flatPlan.instances) ? flatPlan.instances : [];
  const links = Array.isArray(flatPlan.links) ? flatPlan.links : [];
  const topo = Array.isArray(flatPlan.topo_order) ? flatPlan.topo_order : [];
  const topoText = topo.length > 0 ? topo.map((item) => String(item ?? "").trim()).filter(Boolean).join(" -> ") : "-";
  return `구성: instance ${instances.length}개 · link ${links.length}개 · topo ${topoText}`;
}

export function buildFlatPlanView(flatPlan) {
  if (!flatPlan || typeof flatPlan !== "object") {
    return {
      summaryText: "구성: flat plan 없음",
      topoOrder: [],
      instances: [],
      links: [],
    };
  }
  const schema = String(flatPlan.schema ?? "").trim();
  if (schema !== "ddn.guseong_flatten_plan.v1") {
    return {
      summaryText: "구성: flat plan 없음",
      topoOrder: [],
      instances: [],
      links: [],
    };
  }
  const topoOrder = Array.isArray(flatPlan.topo_order)
    ? flatPlan.topo_order.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  const instances = Array.isArray(flatPlan.instances)
    ? flatPlan.instances.map(normalizeFlatPlanInstance).filter(Boolean)
    : [];
  const links = Array.isArray(flatPlan.links)
    ? flatPlan.links.map(normalizeFlatPlanLink).filter(Boolean)
    : [];
  return {
    summaryText: summarizeFlatPlan(flatPlan),
    topoOrder,
    instances,
    links,
  };
}

export function createLessonCanonHydrator({
  wasmUrl = DEFAULT_WASM_CANON_URL,
  cacheBust = 0,
  initInput = undefined,
  createCanon = createWasmCanon,
} = {}) {
  let canonRuntimePromise = null;
  let lastCanonDiags = [];
  let lastRuntimeDiags = [];

  function setCanonDiag(code, message, detail = "") {
    lastCanonDiags = [buildCanonDiag(code, message, detail)];
  }

  function clearCanonDiags() {
    lastCanonDiags = [];
  }

  function getCanonDiags() {
    return Array.isArray(lastCanonDiags) ? [...lastCanonDiags] : [];
  }

  function updateRuntimeDiags(runtime) {
    if (!runtime || typeof runtime !== "object") return;
    const rows = [];
    if (typeof runtime.getLastInitDiag === "function") {
      const row = runtime.getLastInitDiag();
      if (row && typeof row === "object") rows.push(row);
    }
    if (typeof runtime.getLastBuildInfoDiag === "function") {
      const row = runtime.getLastBuildInfoDiag();
      if (row && typeof row === "object") rows.push(row);
    }
    if (typeof runtime.getLastPreprocessDiag === "function") {
      const row = runtime.getLastPreprocessDiag();
      if (row && typeof row === "object") rows.push(row);
    }
    if (typeof runtime.getLastCanonDiag === "function") {
      const row = runtime.getLastCanonDiag();
      if (row && typeof row === "object") rows.push(row);
    }
    lastRuntimeDiags = rows.map((row) =>
      buildCanonDiag(
        String(row.code ?? "E_WASM_CANON_RUNTIME_DIAG"),
        String(row.message ?? ""),
        String(row.detail ?? ""),
      ),
    );
  }

  function getRuntimeDiags() {
    return Array.isArray(lastRuntimeDiags) ? [...lastRuntimeDiags] : [];
  }

  async function getRuntime() {
    if (!canonRuntimePromise) {
      canonRuntimePromise = createCanon({
        wasmUrl,
        cacheBust,
        initInput,
      }).catch((error) => {
        lastRuntimeDiags = [
          buildCanonDiag(
            "E_WASM_CANON_RUNTIME_CREATE_FAILED",
            "wasm canon runtime 생성에 실패했습니다.",
            error?.message ?? String(error ?? ""),
          ),
        ];
        canonRuntimePromise = null;
        throw error;
      });
    }
    return canonRuntimePromise;
  }

  async function deriveMaegimControlJson(ddnText) {
    clearCanonDiags();
    const sourceText = String(ddnText ?? "").trim();
    if (!sourceText) return "";
    let runtime = null;
    try {
      runtime = await getRuntime();
      const plan = await runtime.canonMaegimPlan(sourceText);
      updateRuntimeDiags(runtime);
      if (!plan || typeof plan !== "object") return "";
      return JSON.stringify(plan, null, 2);
    } catch (error) {
      const errText = error?.message ?? String(error ?? "");
      updateRuntimeDiags(runtime);
      const runtimeDiag = pickRuntimeCanonDiag(runtime, errText);
      if (runtimeDiag) {
        setCanonDiag(runtimeDiag.code, runtimeDiag.message, runtimeDiag.detail);
      } else {
        setCanonDiag(
          "E_WASM_MAEGIM_PLAN_FALLBACK_FAILED",
          "wasm maegim plan canonicalization에 실패했습니다.",
          errText,
        );
      }
      console.warn(
        `[seamgrim] wasm maegim plan fallback failed: ${String(error?.message ?? error)}`,
      );
      return "";
    }
  }

  async function deriveFlatJson(ddnText, { quiet = false } = {}) {
    clearCanonDiags();
    const sourceText = String(ddnText ?? "").trim();
    if (!sourceText) return null;
    let runtime = null;
    try {
      runtime = await getRuntime();
      const flat = await runtime.canonFlatJson(sourceText);
      updateRuntimeDiags(runtime);
      return flat && typeof flat === "object" ? flat : null;
    } catch (error) {
      const errText = error?.message ?? String(error ?? "");
      updateRuntimeDiags(runtime);
      const runtimeDiag = pickRuntimeCanonDiag(runtime, errText);
      if (runtimeDiag) {
        setCanonDiag(runtimeDiag.code, runtimeDiag.message, runtimeDiag.detail);
      } else {
        setCanonDiag(
          "E_WASM_FLAT_PLAN_FALLBACK_FAILED",
          "wasm flat plan canonicalization에 실패했습니다.",
          errText,
        );
      }
      if (!quiet) {
        console.warn(
          `[seamgrim] wasm flat plan fallback failed: ${String(error?.message ?? error)}`,
        );
      }
      return null;
    }
  }

  async function hydrateLessonCanon(lesson) {
    const nextLesson = normalizeLessonPayload(lesson);
    const existing = String(nextLesson.maegimControlJson ?? "").trim();
    if (existing) return nextLesson;
    const maegimControlJson = await deriveMaegimControlJson(nextLesson.ddnText ?? "");
    if (!maegimControlJson) return nextLesson;
    return {
      ...nextLesson,
      maegimControlJson,
    };
  }

  return {
    getRuntime,
    getCanonDiags,
    getRuntimeDiags,
    deriveMaegimControlJson,
    deriveFlatJson,
    hydrateLessonCanon,
  };
}
