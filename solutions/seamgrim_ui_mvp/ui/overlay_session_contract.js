import { canOverlayCompareRuns } from "./overlay_compare_contract.js";

function asTextId(value) {
  const text = String(value ?? "").trim();
  return text ? text : null;
}

function toBool(value) {
  return Boolean(value);
}

function toRole(value) {
  const token = String(value ?? "").trim().toLowerCase();
  return token === "baseline" || token === "variant" ? token : null;
}

function toRunRecord(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  const id = asTextId(row.id);
  return {
    id,
    label: String(row.label ?? id ?? ""),
    visible: typeof row.visible === "boolean" ? row.visible : true,
    layerIndex: Number.isFinite(row.layerIndex) ? row.layerIndex : Number.isFinite(row.layer_index) ? row.layer_index : 0,
    compareRole: toRole(row.compareRole ?? row.compare_role),
    source: row.source && typeof row.source === "object" ? row.source : {},
    inputs: row.inputs && typeof row.inputs === "object" ? row.inputs : {},
    graph: row.graph && typeof row.graph === "object" ? row.graph : null,
    space2d: row.space2d ?? null,
    textDoc: row.textDoc ?? row.text_doc ?? null,
  };
}

function findRunByRole(runs, role) {
  return runs.find((run) => run.compareRole === role) ?? null;
}

function findRunById(runs, id) {
  const targetId = asTextId(id);
  if (!targetId) return null;
  return runs.find((run) => run.id === targetId) ?? null;
}

function resolveCompareRunIds(runs, compare) {
  const compareRow = compare && typeof compare === "object" ? compare : {};
  const baselineRoleRun = findRunByRole(runs, "baseline");
  const variantRoleRun = findRunByRole(runs, "variant");
  const baselineById = findRunById(runs, compareRow.baseline_id ?? compareRow.baselineId);
  const variantById = findRunById(runs, compareRow.variant_id ?? compareRow.variantId);
  const baselineRun = baselineRoleRun ?? baselineById;
  const variantRun = variantRoleRun ?? variantById;
  return {
    baselineRun,
    variantRun,
  };
}

export function buildOverlaySessionRunsPayload(runs) {
  const source = Array.isArray(runs) ? runs : [];
  return source.map((raw) => {
    const row = toRunRecord(raw);
    return {
      id: row.id,
      label: row.label,
      visible: row.visible,
      layer_index: row.layerIndex,
      compare_role: row.compareRole,
      source: row.source,
      inputs: row.inputs,
      graph: row.graph,
      space2d: row.space2d,
      text_doc: row.textDoc,
    };
  });
}

export function buildOverlayCompareSessionPayload(compare) {
  const row = compare && typeof compare === "object" ? compare : {};
  return {
    enabled: toBool(row.enabled),
    baseline_id: asTextId(row.baselineId ?? row.baseline_id),
    variant_id: asTextId(row.variantId ?? row.variant_id),
  };
}

export function resolveOverlayCompareFromSession(sessionLike) {
  const row = sessionLike && typeof sessionLike === "object" ? sessionLike : {};
  const runs = (Array.isArray(row.runs) ? row.runs : []).map((raw) => toRunRecord(raw)).filter((run) => Boolean(run.id));
  const compare = row.compare && typeof row.compare === "object" ? row.compare : {};
  const compareEnabled = toBool(compare.enabled);
  if (!compareEnabled) {
    return {
      enabled: false,
      baselineId: null,
      variantId: null,
      droppedVariant: false,
      dropCode: "",
      blockReason: "",
    };
  }

  const { baselineRun, variantRun } = resolveCompareRunIds(runs, compare);
  if (!baselineRun) {
    return {
      enabled: false,
      baselineId: null,
      variantId: null,
      droppedVariant: false,
      dropCode: "baseline_missing",
      blockReason: "baseline 실행을 찾을 수 없어 비교를 비활성화했습니다.",
    };
  }

  if (!variantRun || variantRun.id === baselineRun.id) {
    return {
      enabled: true,
      baselineId: baselineRun.id,
      variantId: null,
      droppedVariant: false,
      dropCode: "",
      blockReason: "",
    };
  }

  const compareResult = canOverlayCompareRuns(baselineRun, variantRun);
  if (!compareResult.ok) {
    return {
      enabled: true,
      baselineId: baselineRun.id,
      variantId: null,
      droppedVariant: true,
      dropCode: String(compareResult.code ?? ""),
      blockReason: String(compareResult.reason ?? ""),
    };
  }

  return {
    enabled: true,
    baselineId: baselineRun.id,
    variantId: variantRun.id,
    droppedVariant: false,
    dropCode: "",
    blockReason: "",
  };
}

const SCREEN_MODES = new Set(["explore", "editor", "run"]);
const WORKSPACE_MODES = new Set(["basic", "advanced"]);
const RUN_ACTIVE_VIEWS = new Set(["view-graph", "view-table", "view-2d", "view-text"]);
const VIEW_COMBO_LAYOUTS = new Set(["horizontal", "overlay"]);
const VIEW_COMBO_ORDERS = new Set(["graph", "space2d"]);

export function resolveSessionUiLayoutFromPayload(payload) {
  const row = payload && typeof payload === "object" ? payload : {};
  const screenMode = SCREEN_MODES.has(String(row.screen_mode ?? row.screenMode)) ? String(row.screen_mode ?? row.screenMode) : "explore";
  const workspaceMode = WORKSPACE_MODES.has(String(row.workspace_mode ?? row.workspaceMode))
    ? String(row.workspace_mode ?? row.workspaceMode)
    : "basic";
  if (screenMode !== "run") {
    return {
      screenMode,
      workspaceMode,
      mainTab: "lesson-tab",
      activeView: "view-graph",
    };
  }

  const activeViewCandidate = String(row.active_view ?? row.activeView);
  const activeView = RUN_ACTIVE_VIEWS.has(activeViewCandidate) ? activeViewCandidate : "view-graph";
  const mainTabRaw = String(row.main_tab ?? row.mainTab);
  let mainTab = "lesson-tab";
  if (workspaceMode === "advanced") {
    mainTab = "tools-tab";
  } else if (mainTabRaw === "lesson-tab") {
    mainTab = "lesson-tab";
  }

  return {
    screenMode,
    workspaceMode,
    mainTab,
    activeView,
  };
}

export function buildSessionUiLayoutPayload(layout) {
  const row = resolveSessionUiLayoutFromPayload(layout);
  return {
    screen_mode: row.screenMode,
    workspace_mode: row.workspaceMode,
    main_tab: row.mainTab,
    active_view: row.activeView,
  };
}

export function resolveSessionViewComboFromPayload(payload) {
  const row = payload && typeof payload === "object" ? payload : {};
  const layoutRaw = String(row.layout);
  const overlayOrderRaw = String(row.overlay_order ?? row.overlayOrder);
  return {
    enabled: toBool(row.enabled),
    layout: VIEW_COMBO_LAYOUTS.has(layoutRaw) ? layoutRaw : "horizontal",
    overlayOrder: VIEW_COMBO_ORDERS.has(overlayOrderRaw) ? overlayOrderRaw : "graph",
  };
}

export function buildSessionViewComboPayload(combo) {
  const row = resolveSessionViewComboFromPayload(combo);
  return {
    enabled: row.enabled,
    layout: row.layout,
    overlay_order: row.overlayOrder,
  };
}

