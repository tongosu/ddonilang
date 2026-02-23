import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function buildRun(id, overrides = {}) {
  return {
    id,
    label: id,
    visible: true,
    layerIndex: 0,
    compareRole: null,
    source: { kind: "manual" },
    inputs: { L: "1.0" },
    graph: {
      schema: "seamgrim.graph.v0",
      meta: {
        graph_kind: "xy",
        axis_x_kind: "length",
        axis_x_unit: "m",
        axis_y_kind: "period",
        axis_y_unit: "s",
      },
      series: [{ id: "pendulum_curve", points: [{ x: 1.0, y: 2.0 }] }],
    },
    space2d: null,
    textDoc: null,
    ...overrides,
  };
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js");
  const contract = await import(pathToFileURL(modulePath).href);
  const {
    buildOverlaySessionRunsPayload,
    buildOverlayCompareSessionPayload,
    resolveOverlayCompareFromSession,
    buildSessionUiLayoutPayload,
    resolveSessionUiLayoutFromPayload,
    buildSessionViewComboPayload,
    resolveSessionViewComboFromPayload,
  } = contract;

  const baseline = buildRun("run-base", { compareRole: "baseline" });
  const variant = buildRun("run-var", {
    compareRole: "variant",
    inputs: { L: "2.0" },
    layerIndex: 1,
  });

  const comparePayload = buildOverlayCompareSessionPayload({
    enabled: true,
    baselineId: "run-base",
    variantId: "run-var",
  });
  assert(comparePayload.enabled === true, "compare payload: enabled");
  assert(comparePayload.baseline_id === "run-base", "compare payload: baseline_id");
  assert(comparePayload.variant_id === "run-var", "compare payload: variant_id");

  const sessionRuns = buildOverlaySessionRunsPayload([baseline, variant]);
  assert(Array.isArray(sessionRuns) && sessionRuns.length === 2, "runs payload: length");
  assert(sessionRuns[1].layer_index === 1, "runs payload: layer index");
  assert(sessionRuns[1].inputs?.L === "2.0", "runs payload: variant params");
  assert(sessionRuns[0].compare_role === "baseline", "runs payload: baseline role");

  const resolvedOk = resolveOverlayCompareFromSession({
    runs: [baseline, variant],
    compare: {
      enabled: true,
      baseline_id: "ignored-id",
      variant_id: "ignored-id-2",
    },
  });
  assert(resolvedOk.enabled === true, "resolve ok: enabled");
  assert(resolvedOk.baselineId === "run-base", "resolve ok: role precedence baseline");
  assert(resolvedOk.variantId === "run-var", "resolve ok: role precedence variant");
  assert(resolvedOk.blockReason === "", "resolve ok: no block reason");
  assert(resolvedOk.droppedVariant === false, "resolve ok: droppedVariant false");

  const mismatchVariant = buildRun("run-var-2", {
    compareRole: "variant",
    graph: {
      schema: "seamgrim.graph.v0",
      meta: {
        graph_kind: "xy",
        axis_x_kind: "length",
        axis_x_unit: "cm",
        axis_y_kind: "period",
        axis_y_unit: "s",
      },
      series: [{ id: "pendulum_curve_alt", points: [{ x: 100.0, y: 2.0 }] }],
    },
  });
  const resolvedMismatch = resolveOverlayCompareFromSession({
    runs: [baseline, mismatchVariant],
    compare: { enabled: true, baseline_id: "run-base", variant_id: "run-var-2" },
  });
  assert(resolvedMismatch.enabled === true, "resolve mismatch: enabled");
  assert(resolvedMismatch.baselineId === "run-base", "resolve mismatch: baseline kept");
  assert(resolvedMismatch.variantId === null, "resolve mismatch: variant dropped");
  assert(resolvedMismatch.droppedVariant === true, "resolve mismatch: droppedVariant true");
  assert(resolvedMismatch.dropCode === "mismatch_xUnit", "resolve mismatch: dropCode");
  assert(String(resolvedMismatch.blockReason).includes("x축 unit"), "resolve mismatch: block reason");

  const resolvedMissingBaseline = resolveOverlayCompareFromSession({
    runs: [variant],
    compare: { enabled: true, baseline_id: "run-base", variant_id: "run-var" },
  });
  assert(resolvedMissingBaseline.enabled === false, "resolve missing baseline: disabled");
  assert(resolvedMissingBaseline.dropCode === "baseline_missing", "resolve missing baseline: dropCode");
  assert(
    String(resolvedMissingBaseline.blockReason).includes("baseline 실행"),
    "resolve missing baseline: reason",
  );

  const resolvedDisabled = resolveOverlayCompareFromSession({
    runs: [baseline, variant],
    compare: { enabled: false, baseline_id: "run-base", variant_id: "run-var" },
  });
  assert(resolvedDisabled.enabled === false, "resolve disabled: disabled");
  assert(resolvedDisabled.baselineId === null, "resolve disabled: baseline null");
  assert(resolvedDisabled.variantId === null, "resolve disabled: variant null");

  const resolvedSameId = resolveOverlayCompareFromSession({
    runs: [baseline],
    compare: { enabled: true, baseline_id: "run-base", variant_id: "run-base" },
  });
  assert(resolvedSameId.enabled === true, "resolve same id: enabled");
  assert(resolvedSameId.baselineId === "run-base", "resolve same id: baseline");
  assert(resolvedSameId.variantId === null, "resolve same id: variant dropped");

  const layoutPayload = buildSessionUiLayoutPayload({
    screenMode: "run",
    workspaceMode: "advanced",
    mainTab: "tools-tab",
    activeView: "view-2d",
  });
  assert(layoutPayload.screen_mode === "run", "layout payload: screen_mode");
  assert(layoutPayload.workspace_mode === "advanced", "layout payload: workspace_mode");
  assert(layoutPayload.main_tab === "tools-tab", "layout payload: main_tab");
  assert(layoutPayload.active_view === "view-2d", "layout payload: active_view");

  const layoutFallback = resolveSessionUiLayoutFromPayload({
    screen_mode: "invalid",
    workspace_mode: "basic",
    main_tab: "tools-tab",
    active_view: "unknown",
  });
  assert(layoutFallback.screenMode === "explore", "layout fallback: screen");
  assert(layoutFallback.workspaceMode === "basic", "layout fallback: workspace");
  assert(layoutFallback.mainTab === "lesson-tab", "layout fallback: main_tab");
  assert(layoutFallback.activeView === "view-graph", "layout fallback: active_view");

  const layoutAdvanced = resolveSessionUiLayoutFromPayload({
    screen_mode: "run",
    workspace_mode: "advanced",
    main_tab: "ddn-tab",
    active_view: "view-table",
  });
  assert(layoutAdvanced.mainTab === "tools-tab", "layout advanced: tools tab enforced");

  const comboPayload = buildSessionViewComboPayload({
    enabled: true,
    layout: "overlay",
    overlayOrder: "space2d",
  });
  assert(comboPayload.enabled === true, "combo payload: enabled");
  assert(comboPayload.layout === "overlay", "combo payload: layout");
  assert(comboPayload.overlay_order === "space2d", "combo payload: overlay_order");

  const comboFallback = resolveSessionViewComboFromPayload({
    enabled: true,
    layout: "diagonal",
    overlay_order: "unknown",
  });
  assert(comboFallback.enabled === true, "combo fallback: enabled");
  assert(comboFallback.layout === "horizontal", "combo fallback: layout");
  assert(comboFallback.overlayOrder === "graph", "combo fallback: overlay_order");

  console.log("[overlay-session-contract] ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
