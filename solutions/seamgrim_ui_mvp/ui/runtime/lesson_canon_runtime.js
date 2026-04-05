import { createWasmCanon } from "./wasm_canon_runtime.js";

const DEFAULT_WASM_CANON_URL = "../wasm/ddonirang_tool.js";

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

  async function getRuntime() {
    if (!canonRuntimePromise) {
      canonRuntimePromise = createCanon({
        wasmUrl,
        cacheBust,
        initInput,
      }).catch((error) => {
        canonRuntimePromise = null;
        throw error;
      });
    }
    return canonRuntimePromise;
  }

  async function deriveMaegimControlJson(ddnText) {
    const sourceText = String(ddnText ?? "").trim();
    if (!sourceText) return "";
    try {
      const runtime = await getRuntime();
      const plan = await runtime.canonMaegimPlan(sourceText);
      if (!plan || typeof plan !== "object") return "";
      return JSON.stringify(plan, null, 2);
    } catch (error) {
      console.warn(
        `[seamgrim] wasm maegim plan fallback failed: ${String(error?.message ?? error)}`,
      );
      return "";
    }
  }

  async function deriveFlatJson(ddnText, { quiet = false } = {}) {
    const sourceText = String(ddnText ?? "").trim();
    if (!sourceText) return null;
    try {
      const runtime = await getRuntime();
      const flat = await runtime.canonFlatJson(sourceText);
      return flat && typeof flat === "object" ? flat : null;
    } catch (error) {
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
    deriveMaegimControlJson,
    deriveFlatJson,
    hydrateLessonCanon,
  };
}
