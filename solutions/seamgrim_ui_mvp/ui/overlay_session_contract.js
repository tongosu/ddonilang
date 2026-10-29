import { resolveGraphUpdateTick } from "./update_tick_contract.js";

function normalizeToken(value) {
  return String(value ?? "").trim().toLowerCase();
}

function pickMetaToken(meta, ...keys) {
  if (!meta || typeof meta !== "object") return "";
  for (const key of keys) {
    const token = normalizeToken(meta[key]);
    if (token) return token;
  }
  return "";
}

function readPrimarySeriesId(graph) {
  const series = Array.isArray(graph?.series) ? graph.series : [];
  for (const row of series) {
    const id = normalizeToken(row?.id);
    if (id) return id;
  }
  return "";
}

function toComparableGraph(raw) {
  const graph = raw && typeof raw === "object" ? raw.graph : null;
  if (!graph || typeof graph !== "object") return null;
  const meta = graph.meta && typeof graph.meta === "object" ? graph.meta : {};
  return {
    graphKind: pickMetaToken(meta, "graph_kind", "kind", "schema"),
    xKind: pickMetaToken(meta, "axis_x_kind", "axis_kind"),
    xUnit: pickMetaToken(meta, "axis_x_unit", "axis_unit"),
    yKind: pickMetaToken(meta, "axis_y_kind", "y_kind"),
    yUnit: pickMetaToken(meta, "axis_y_unit", "y_unit"),
    seriesId: readPrimarySeriesId(graph),
    schemaToken: normalizeToken(graph.schema),
  };
}

function failCompare(code, reason) {
  return {
    ok: false,
    code,
    reason,
  };
}

function canOverlayCompareRuns(baseline, variant) {
  const left = toComparableGraph(baseline);
  const right = toComparableGraph(variant);
  if (!left || !right) {
    return failCompare("graph_missing", "그래프 데이터가 없어 비교할 수 없습니다.");
  }

  const baselineGraphKind = left.graphKind || left.schemaToken;
  const variantGraphKind = right.graphKind || right.schemaToken;
  if (baselineGraphKind !== variantGraphKind) {
    return failCompare("mismatch_graphKind", "graph_kind 가 서로 다릅니다.");
  }
  if (left.xKind !== right.xKind) {
    return failCompare("mismatch_xKind", "x축 kind 가 서로 다릅니다.");
  }
  if (left.xUnit !== right.xUnit) {
    return failCompare("mismatch_xUnit", "x축 unit 이 서로 다릅니다.");
  }
  if (left.yKind !== right.yKind) {
    return failCompare("mismatch_yKind", "y축 kind 가 서로 다릅니다.");
  }
  if (left.yUnit !== right.yUnit) {
    return failCompare("mismatch_yUnit", "y축 unit 이 서로 다릅니다.");
  }

  if (!left.seriesId || !right.seriesId) {
    return failCompare("series_missing", "series_id 가 비어 있어 비교할 수 없습니다.");
  }
  if (left.seriesId !== right.seriesId) {
    return failCompare("series_mismatch", "series_id 가 서로 다릅니다.");
  }
  return {
    ok: true,
    code: "ok",
    reason: "",
  };
}

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

function toGroupId(value) {
  const text = String(value ?? "").trim();
  return text ? text : null;
}

function toRunRecord(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  const id = asTextId(row.id);
  const graph = row.graph && typeof row.graph === "object" ? row.graph : null;
  const updateTick = resolveGraphUpdateTick(graph);
  return {
    id,
    label: String(row.label ?? id ?? ""),
    visible: typeof row.visible === "boolean" ? row.visible : true,
    layerIndex: Number.isFinite(row.layerIndex)
      ? row.layerIndex
      : Number.isFinite(row.layer_index)
        ? row.layer_index
        : 0,
    groupId: toGroupId(row.groupId ?? row.group_id ?? row.group ?? row["그룹"] ?? row["묶음"]),
    compareRole: toRole(row.compareRole ?? row.compare_role),
    source: row.source && typeof row.source === "object" ? row.source : {},
    inputs: row.inputs && typeof row.inputs === "object" ? row.inputs : {},
    graph,
    space2d: row.space2d ?? null,
    textDoc: row.textDoc ?? row.text_doc ?? null,
    update: updateTick.update,
    tick: updateTick.tick,
    ticks: updateTick.ticks,
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
      group_id: row.groupId,
      compare_role: row.compareRole,
      source: row.source,
      inputs: row.inputs,
      graph: row.graph,
      space2d: row.space2d,
      text_doc: row.textDoc,
      update: row.update,
      tick: row.tick,
      ticks: row.ticks,
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
  const runs = (Array.isArray(row.runs) ? row.runs : [])
    .map((raw) => toRunRecord(raw))
    .filter((run) => Boolean(run.id));
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
  const screenMode = SCREEN_MODES.has(String(row.screen_mode ?? row.screenMode))
    ? String(row.screen_mode ?? row.screenMode)
    : "explore";
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
