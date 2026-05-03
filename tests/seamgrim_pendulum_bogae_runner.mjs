import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

if (typeof globalThis.requestAnimationFrame !== "function") {
  globalThis.requestAnimationFrame = (callback) => {
    if (typeof callback === "function") {
      callback();
    }
    return 1;
  };
}

function collectGroupIds(space2d) {
  const shapes = Array.isArray(space2d?.shapes) ? space2d.shapes : [];
  return shapes.map((shape) => String(shape?.group_id ?? "").trim()).filter(Boolean);
}

async function main() {
  const root = process.cwd();
  const runPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const observeActionContractPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_observe_action_contract.js");
  const runMod = await import(pathToFileURL(runPath).href);
  const observeActionMod = await import(pathToFileURL(observeActionContractPath).href);
  const { OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT } = observeActionMod;
  const {
    RunScreen,
    normalizeRuntimeTableView,
    resolveRunDockPanelOrder,
    resolveRunDockPanelVisibility,
    resolveRunLayoutProfile,
    resolveStudioLayoutBounds,
    resolveRuntimeTableCellMaxChars,
    summarizeRuntimeTableView,
    buildRuntimeStructurePreviewHtml,
    buildRuntimeGraphPreviewHtml,
    summarizeRuntimeGraphMarkdown,
    summarizeRuntimeStructureMarkdown,
    renderRuntimeTable,
    normalizeObserveOutputRows,
    summarizeObserveOutputRows,
    buildObserveOutputRowsPreview,
    resolveRunMainVisualMode,
    resolveRunPostExecuteTab,
    mergeRuntimeViewsWithObservationOutputFallback,
    synthesizePendulumSpace2dFromObservation,
    synthesizePointSpace2dFromObservation,
    synthesizeSpace2dFromGraph,
    synthesizeSpace2dFromObservation,
  } = runMod;

  assert(typeof synthesizeSpace2dFromObservation === "function", "run export: synthesizeSpace2dFromObservation");
  assert(typeof synthesizePendulumSpace2dFromObservation === "function", "run export: synthesizePendulumSpace2dFromObservation");
  assert(typeof synthesizePointSpace2dFromObservation === "function", "run export: synthesizePointSpace2dFromObservation");
  assert(typeof synthesizeSpace2dFromGraph === "function", "run export: synthesizeSpace2dFromGraph");
  assert(typeof normalizeRuntimeTableView === "function", "run export: normalizeRuntimeTableView");
  assert(typeof resolveRunDockPanelOrder === "function", "run export: resolveRunDockPanelOrder");
  assert(typeof resolveRunDockPanelVisibility === "function", "run export: resolveRunDockPanelVisibility");
  assert(typeof resolveRunLayoutProfile === "function", "run export: resolveRunLayoutProfile");
  assert(typeof resolveStudioLayoutBounds === "function", "run export: resolveStudioLayoutBounds");
  assert(typeof resolveRuntimeTableCellMaxChars === "function", "run export: resolveRuntimeTableCellMaxChars");
  assert(typeof buildRuntimeStructurePreviewHtml === "function", "run export: buildRuntimeStructurePreviewHtml");
  assert(typeof buildRuntimeGraphPreviewHtml === "function", "run export: buildRuntimeGraphPreviewHtml");
  assert(typeof summarizeRuntimeTableView === "function", "run export: summarizeRuntimeTableView");
  assert(typeof summarizeRuntimeGraphMarkdown === "function", "run export: summarizeRuntimeGraphMarkdown");
  assert(typeof summarizeRuntimeStructureMarkdown === "function", "run export: summarizeRuntimeStructureMarkdown");
  assert(typeof renderRuntimeTable === "function", "run export: renderRuntimeTable");
  assert(typeof normalizeObserveOutputRows === "function", "run export: normalizeObserveOutputRows");
  assert(typeof summarizeObserveOutputRows === "function", "run export: summarizeObserveOutputRows");
  assert(typeof buildObserveOutputRowsPreview === "function", "run export: buildObserveOutputRowsPreview");
  assert(typeof resolveRunMainVisualMode === "function", "run export: resolveRunMainVisualMode");
  assert(typeof mergeRuntimeViewsWithObservationOutputFallback === "function", "run export: mergeRuntimeViewsWithObservationOutputFallback");
  assert(typeof RunScreen === "function", "run export: RunScreen");

  const normalizedOutputRows = normalizeObserveOutputRows([
    { key: "속도", value: "1.23", source: "table.row", syntheticKey: false },
    { key: " ", value: "skip" },
    { key: "높이", value: "3.45", source: "table.row", syntheticKey: false },
  ]);
  assert(normalizedOutputRows.length === 2, "observe output rows normalize: count");
  assert(normalizedOutputRows[0].key === "속도", "observe output rows normalize: first key");
  assert(normalizedOutputRows[1].key === "높이", "observe output rows normalize: second key");
  assert(normalizedOutputRows[0].source === "table.row", "observe output rows normalize: source keep");
  assert(normalizedOutputRows[0].syntheticKey === false, "observe output rows normalize: synthetic flag keep");
  const outputRowsSummary = summarizeObserveOutputRows(normalizedOutputRows);
  assert(outputRowsSummary.includes("2행"), "observe output rows summary: row count");
  assert(outputRowsSummary.includes("최근 높이=3.45"), "observe output rows summary: recent row");
  const outputRowsPreview = buildObserveOutputRowsPreview(normalizedOutputRows, { maxRows: 2 });
  assert(outputRowsPreview.includes("속도=1.23"), "observe output rows preview: first row");
  assert(outputRowsPreview.includes("높이=3.45"), "observe output rows preview: second row");

  const mainVisualGraph = resolveRunMainVisualMode({
    views: {
      graph: {
        axis: { x_min: 0, x_max: 1, y_min: 0, y_max: 2 },
        series: [{ id: "y", points: [{ x: 0, y: 0 }, { x: 1, y: 2 }] }],
      },
    },
    allowShapeFallback: true,
  });
  assert(mainVisualGraph.mode === "debug-fallback", "main visual mode: graph-only view maps to space2d fallback");
  assert(mainVisualGraph.space2d?.meta?.title === "graph-point-fallback", "main visual mode: graph-only fallback title");

  const mainVisualObservationFallbackConsole = resolveRunMainVisualMode({
    views: {
      graph: {
        axis: { x_min: -1, x_max: 1, y_min: 14, y_max: 24 },
        series: [{ id: "합", points: [{ x: 0, y: 23 }] }],
      },
      graphSource: "observation-fallback",
    },
    outputRows: [
      { key: "합", value: "23", source: "observation" },
      { key: "x", value: "15", source: "observation" },
      { key: "y", value: "8", source: "observation" },
    ],
    allowShapeFallback: true,
  });
  assert(mainVisualObservationFallbackConsole.mode === "console-grid", "main visual mode: fallback graph yields console-grid for scalar output");
  assert(
    JSON.stringify(mainVisualObservationFallbackConsole.consoleLinesForGrid ?? []).includes('"23"'),
    "main visual mode: fallback graph console-grid keeps scalar output value",
  );

  const mainVisualObservationFallbackRequiredGraph = resolveRunMainVisualMode({
    views: {
      graph: {
        axis: { x_min: -1, x_max: 1, y_min: 14, y_max: 24 },
        series: [{ id: "합", points: [{ x: 0, y: 23 }] }],
      },
      graphSource: "observation-fallback",
    },
    outputRows: [
      { key: "합", value: "23", source: "observation" },
      { key: "x", value: "15", source: "observation" },
      { key: "y", value: "8", source: "observation" },
    ],
    lessonRequiredViews: ["graph"],
    allowShapeFallback: true,
  });
  assert(mainVisualObservationFallbackRequiredGraph.mode === "console-grid", "main visual mode: lesson-required graph still keeps console-grid for scalar output");

  const mainVisualConsole = resolveRunMainVisualMode({
    views: { text: { markdown: "## 안내" } },
    outputRows: [{ key: "t", value: "0" }],
    warnings: [{ code: "E_WASM_CANON_JSON_PARSE_FAILED", message: "구성 해석에 실패했습니다. 실행은 계속할 수 있습니다." }],
    allowShapeFallback: true,
  });
  assert(mainVisualConsole.mode === "console-grid", "main visual mode: console-grid fallback after graph absent");
  assert(Array.isArray(mainVisualConsole.consoleLinesForGrid), "main visual mode: console-grid exposes lines array");

  const mainVisualConsoleFallback = resolveRunMainVisualMode({
    views: null,
    outputLines: ["0", "2", "4"],
    outputRows: [
      { key: "출력1", value: "0", source: "fallback-line", syntheticKey: true },
      { key: "출력2", value: "1", source: "fallback-line", syntheticKey: true },
    ],
    warnings: [],
    allowShapeFallback: true,
  });
  assert(mainVisualConsoleFallback.mode === "console-grid", "main visual mode: synthetic rows still prefer console-grid");
  assert(
    JSON.stringify(mainVisualConsoleFallback.consoleLinesForGrid ?? []).includes("0"),
    "main visual mode: synthetic rows expose line payload",
  );
  assert(
    JSON.stringify(mainVisualConsoleFallback.consoleLinesForGrid ?? []).includes("1"),
    "main visual mode: synthetic rows preserve numeric payload",
  );

  const mainVisualSpace2d = resolveRunMainVisualMode({
    views: {
      space2d: {
        schema: "seamgrim.space2d.v0",
        camera: { x_min: -1, x_max: 1, y_min: -1, y_max: 1 },
        points: [{ x: 0, y: 0 }],
      },
      graph: {
        axis: { x_min: 0, x_max: 1, y_min: 0, y_max: 1 },
        series: [{ id: "y", points: [{ x: 0, y: 0 }, { x: 1, y: 1 }] }],
      },
    },
    allowShapeFallback: true,
  });
  assert(mainVisualSpace2d.mode === "space2d", "main visual mode: space2d priority");

  const outputLinesSpace2dState = {
    schema: "seamgrim.state.v0",
    channels: [{ key: "보개_출력_줄들", dtype: "text", role: "state" }],
    row: ['차림["space2d","space2d.shape","line","x1","0","y1","0","x2","1","y2","1","space2d.shape","circle","x","1","y","1","r","0.08","fill","#38bdf8"]'],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
  };
  const mergedFallbackViews = mergeRuntimeViewsWithObservationOutputFallback(outputLinesSpace2dState, {
    graph: null,
    graphRaw: null,
    graphSource: null,
    space2d: null,
    space2dRaw: null,
    table: null,
    tableRaw: null,
    text: null,
    textRaw: null,
    structure: null,
    structureRaw: null,
  });
  assert(mergedFallbackViews?.space2d && Array.isArray(mergedFallbackViews.space2d.shapes), "runtime merge: output-lines space2d restored");
  assert(mergedFallbackViews.space2d.shapes.length === 2, "runtime merge: output-lines shape count");
  const mainVisualMergedSpace2d = resolveRunMainVisualMode({
    views: mergedFallbackViews,
    allowShapeFallback: true,
  });
  assert(mainVisualMergedSpace2d.mode === "space2d", "main visual mode: merged output-lines space2d wins over console");
  assert(resolveRunPostExecuteTab({ outputRows: [{ key: "x", value: "1" }] }) === "output", "post execute tab: output rows => output");
  assert(resolveRunPostExecuteTab({ views: { graph: { series: [{ id: "v", points: [{ x: 0, y: 1 }] }] } } }) === "graph", "post execute tab: graph view => graph");
  assert(resolveRunPostExecuteTab({ views: { text: { markdown: "## 안내" } } }) === "graph", "post execute tab: text view => graph");
  assert(resolveRunPostExecuteTab({}) === "console", "post execute tab: empty => console");
  assert(
    resolveRunPostExecuteTab({
      views: { graph: { series: [{ id: "v", points: [{ x: 0, y: 1 }] }] } },
      outputRows: [{ key: "x", value: "1" }],
    }) === "graph",
    "post execute tab: graph payload wins over output rows",
  );

  const pendulumFromValues = synthesizeSpace2dFromObservation({
    channels: [{ key: "theta" }, { key: "L" }],
    row: [0.5, 1.0],
    values: { theta: 0.5, L: 1.0 },
  });
  assert(pendulumFromValues && Array.isArray(pendulumFromValues.shapes), "pendulum from values: shapes");
  assert(pendulumFromValues.meta?.title === "pendulum-observation-fallback", "pendulum from values: title");
  assert(
    collectGroupIds(pendulumFromValues).join(",") === "pendulum.rod,pendulum.bob,pendulum.pivot",
    "pendulum from values: group_id",
  );

  const pendulumFromRow = synthesizeSpace2dFromObservation({
    channels: ["t", "theta"],
    row: [0.02, 0.42],
  });
  assert(pendulumFromRow && Array.isArray(pendulumFromRow.shapes), "pendulum from row: shapes");
  assert(pendulumFromRow.meta?.title === "pendulum-observation-fallback", "pendulum from row: title");
  assert(
    collectGroupIds(pendulumFromRow).join(",") === "pendulum.rod,pendulum.bob,pendulum.pivot",
    "pendulum from row: group_id",
  );

  const pointFromRow = synthesizeSpace2dFromObservation({
    channels: ["x", "y"],
    row: [1.2, -0.4],
  });
  assert(pointFromRow && Array.isArray(pointFromRow.shapes), "point fallback from row: shapes");
  assert(pointFromRow.meta?.title === "xy-observation-fallback", "point fallback from row: title");
  assert(
    collectGroupIds(pointFromRow).join(",") === "graph.axis.x,graph.axis.y,graph.point.focus",
    "point fallback from row: group_id",
  );

  const none = synthesizeSpace2dFromObservation({ channels: ["t"], row: [0.1] });
  assert(none === null, "fallback none: null result");

  const pendulumFromGraph = synthesizeSpace2dFromGraph({
    series: [
      {
        id: "theta",
        points: [
          { x: 0, y: 0.5 },
          { x: 0.1, y: 0.3 },
        ],
      },
    ],
  });
  assert(pendulumFromGraph && Array.isArray(pendulumFromGraph.shapes), "pendulum from graph: shapes");
  assert(pendulumFromGraph.meta?.title === "pendulum-graph-fallback", "pendulum from graph: title");
  assert(
    collectGroupIds(pendulumFromGraph).join(",") === "pendulum.rod,pendulum.bob,pendulum.pivot",
    "pendulum from graph: group_id",
  );

  const pointFromGraph = synthesizeSpace2dFromGraph({
    axis: { x_min: -2, x_max: 2, y_min: -1, y_max: 1 },
    series: [{ id: "y", points: [{ x: 1.25, y: -0.5 }] }],
  });
  assert(pointFromGraph && Array.isArray(pointFromGraph.shapes), "point from graph: shapes");
  assert(pointFromGraph.meta?.title === "graph-point-fallback", "point from graph: title");
  const pointFromGraphGroups = collectGroupIds(pointFromGraph);
  assert(pointFromGraphGroups.includes("graph.axis.x"), "point from graph: has x axis");
  assert(pointFromGraphGroups.includes("graph.axis.y"), "point from graph: has y axis");
  assert(pointFromGraphGroups.includes("graph.point.focus"), "point from graph: has focus point");

  const graphOnlyLayout = resolveRunLayoutProfile(["graph", "text"]);
  assert(graphOnlyLayout.mode === "split", "run layout profile: graph/text => split");
  assert(graphOnlyLayout.hasSpatial === false, "run layout profile: graph/text spatial false");
  const splitLayout = resolveRunLayoutProfile(["space2d", "graph"]);
  assert(splitLayout.mode === "split", "run layout profile: space2d+graph => split");
  assert(splitLayout.hasSpatial === true, "run layout profile: space2d+graph spatial true");
  const spaceOnlyLayout = resolveRunLayoutProfile(["space2d"]);
  assert(spaceOnlyLayout.mode === "space_primary", "run layout profile: space2d => space_primary");
  const studioBounds = resolveStudioLayoutBounds({
    layoutWidth: 1280,
    layoutHeight: 820,
    splitterWidth: 6,
    toolbarHeight: 40,
    errorBannerHeight: 0,
    minVisualWidth: 420,
    subpanelMinHeight: 300,
    bogaeAspectRatio: 16 / 9,
  });
  assert(studioBounds.editorRatioMin >= 0, "studio layout bounds: min ratio");
  assert(studioBounds.editorRatioMax <= 0.85, "studio layout bounds: max ratio");
  assert(studioBounds.editorRatioMax >= studioBounds.editorRatioMin, "studio layout bounds: ratio order");
  assert(
    Math.abs(studioBounds.bogaeFrameMaxWidthPx - (studioBounds.bogaeFrameMaxHeightPx * (16 / 9))) < 1e-9,
    "studio layout bounds: aspect ratio width",
  );
  assert(studioBounds.hasConstraintOverflow === false, "studio layout bounds: no overflow");
  const studioBoundsHeightOverflow = resolveStudioLayoutBounds({
    layoutWidth: 920,
    layoutHeight: 300,
    splitterWidth: 6,
    toolbarHeight: 40,
    errorBannerHeight: 24,
    minVisualWidth: 420,
    subpanelMinHeight: 300,
    bogaeAspectRatio: 16 / 9,
  });
  assert(studioBoundsHeightOverflow.hasHeightOverflow === true, "studio layout bounds: height overflow");
  assert(studioBoundsHeightOverflow.hasConstraintOverflow === true, "studio layout bounds: overflow aggregate");
  const studioBoundsWidthOverflow = resolveStudioLayoutBounds({
    layoutWidth: 380,
    layoutHeight: 760,
    splitterWidth: 6,
    toolbarHeight: 40,
    errorBannerHeight: 0,
    minVisualWidth: 420,
    subpanelMinHeight: 300,
    bogaeAspectRatio: 16 / 9,
  });
  assert(studioBoundsWidthOverflow.hasWidthOverflow === true, "studio layout bounds: width overflow");
  assert(studioBoundsWidthOverflow.hasConstraintOverflow === true, "studio layout bounds: width overflow aggregate");
  const compatLayout = resolveRunLayoutProfile(["space2d", "space2d", "GRAPH"]);
  assert(
    JSON.stringify(compatLayout.families) === JSON.stringify(["space2d", "graph"]),
    "run layout profile: normalize and dedupe",
  );
  assert(
    JSON.stringify(resolveRunDockPanelOrder(["table", "text", "graph"])) === JSON.stringify(["table", "text", "graph"]),
    "run dock order: preserve lesson order",
  );
  assert(
    JSON.stringify(resolveRunDockPanelOrder(["structure"])) === JSON.stringify(["text", "graph", "table"]),
    "run dock order: structure maps to text",
  );
  const graphRequiredVisible = resolveRunDockPanelVisibility(["graph"], {});
  assert(graphRequiredVisible.graph === true, "run dock visibility: graph required");
  assert(graphRequiredVisible.table === false, "run dock visibility: table hidden by default");
  assert(graphRequiredVisible.text === false, "run dock visibility: text hidden by default");
  const runtimeOnlyVisible = resolveRunDockPanelVisibility([], {
    runtimeDerived: {
      observation: { channels: [{ key: "t" }, { key: "x" }] },
      views: { table: { columns: [{ key: "x" }], rows: [{ x: 1 }] } },
    },
    textMarkdown: "안내",
  });
  assert(runtimeOnlyVisible.graph === true, "run dock visibility: runtime observation => graph");
  assert(runtimeOnlyVisible.table === true, "run dock visibility: runtime table => table");
  assert(runtimeOnlyVisible.text === true, "run dock visibility: runtime text => text");
  const toggled = new Map();
  const runScreenLayout = new RunScreen({
    root: { dataset: {} },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenLayout.layoutEl = {
    classList: {
      toggle(name, enabled) {
        toggled.set(name, Boolean(enabled));
      },
    },
  };
  const appliedLayout = runScreenLayout.applyLessonLayoutProfile({ requiredViews: ["graph", "text"] });
  assert(appliedLayout.mode === "split", "run layout apply: mode");
  assert(toggled.get("run-layout--dock-only") === false, "run layout apply: dock_only class off");
  assert(toggled.get("run-layout--split") === true, "run layout apply: split class on");
  assert(runScreenLayout.root.dataset.requiredViews === "graph,text", "run layout apply: dataset requiredViews");
  assert(runScreenLayout.root.dataset.runLayoutMode === "split", "run layout apply: dataset mode");
  assert(runScreenLayout.root.dataset.runDockOrder === "graph,text,table", "run layout apply: dataset dock order");
  runScreenLayout.lesson = null;
  runScreenLayout.syncRuntimeLayoutProfile({}, "console-grid", {
    outputRows: [],
    outputLines: [],
    outputLog: [],
  });
  assert(runScreenLayout.root.dataset.runLayoutMode === "split", "run layout runtime sync: keep split on empty reset");
  assert(toggled.get("run-layout--split") === true, "run layout runtime sync: split class stays on");
  assert(toggled.get("run-layout--space-primary") === false, "run layout runtime sync: space_primary stays off");
  assert(typeof runScreenLayout.switchRunTab === "function", "run export: switchRunTab");
  assert(typeof runScreenLayout.syncDockTimeUi === "function", "run export: syncDockTimeUi");
  assert(typeof runScreenLayout.handleObserveCardAction === "function", "run export: handleObserveCardAction");
  assert(typeof runScreenLayout.focusDdnToken === "function", "run export: focusDdnToken");
  assert(typeof runScreenLayout.openObserveOutputRowsInDdn === "function", "run export: openObserveOutputRowsInDdn");
  const tabState = {};
  const makeToggleEl = (key) => ({
    classList: {
      add(name) {
        tabState[`${key}:${String(name)}`] = true;
      },
      remove(name) {
        tabState[`${key}:${String(name)}`] = false;
      },
      toggle(name, enabled) {
        tabState[`${key}:${name}`] = Boolean(enabled);
      },
    },
  });
  const tabButtons = {
    console: makeToggleEl("btn:console"),
    output: makeToggleEl("btn:output"),
    mirror: makeToggleEl("btn:mirror"),
    graph: makeToggleEl("btn:graph"),
    overlay: makeToggleEl("btn:overlay"),
  };
  const tabPanels = {
    console: makeToggleEl("panel:console"),
    output: makeToggleEl("panel:output"),
    mirror: makeToggleEl("panel:mirror"),
    graph: makeToggleEl("panel:graph"),
    overlay: makeToggleEl("panel:overlay"),
  };
  runScreenLayout.root = {
    dataset: {},
    querySelector(selector) {
      const buttonPrefix = "#run-tab-btn-";
      const panelPrefix = "#run-tab-panel-";
      if (selector.startsWith(buttonPrefix)) {
        return tabButtons[selector.slice(buttonPrefix.length)] ?? null;
      }
      if (selector.startsWith(panelPrefix)) {
        return tabPanels[selector.slice(panelPrefix.length)] ?? null;
      }
      return null;
    },
  };
  runScreenLayout.runTabPanels = new Map(Object.entries(tabPanels));
  runScreenLayout.switchRunTab("console");
  assert(runScreenLayout.activeRunTab === "console", "run tab switch: active tab");
  assert(tabState["btn:console:active"] === true, "run tab switch: console button active");
  assert(tabState["panel:console:hidden"] === false, "run tab switch: console panel shown");
  assert(tabState["panel:graph:hidden"] === true, "run tab switch: graph panel hidden");
  runScreenLayout.switchRunTab("output");
  assert(runScreenLayout.activeRunTab === "output", "run tab switch: output active tab");
  assert(tabState["btn:output:active"] === true, "run tab switch: output button active");
  assert(tabState["panel:output:hidden"] === false, "run tab switch: output panel shown");
  runScreenLayout.switchRunTab("overlay");
  assert(runScreenLayout.activeRunTab === "overlay", "run tab switch: overlay active tab");
  assert(tabState["btn:overlay:active"] === true, "run tab switch: overlay button active");
  assert(tabState["panel:overlay:hidden"] === false, "run tab switch: overlay panel shown");
  assert(typeof runScreenLayout.focusObserveFamily === "function", "run export: focusObserveFamily");
  let xAxisFocused = false;
  let graphScrolled = false;
  let tableScrolled = false;
  let textScrolled = false;
  let spaceScrolled = false;
  let selectedAxesPayload = null;
  const xAxisSelect = {
    value: "",
    options: [{ value: "__index__" }, { value: "t" }, { value: "x" }, { value: "y" }],
    focus() {
      xAxisFocused = true;
    },
  };
  const yAxisSelect = {
    value: "",
    options: [{ value: "x" }, { value: "y" }],
  };
  runScreenLayout.dockTargetSelectEl = { value: "" };
  runScreenLayout.dotbogi = {
    setSelectedAxes(payload) {
      selectedAxesPayload = payload;
    },
    renderGraph() {},
  };
  runScreenLayout.graphPanelEl = {
    scrollIntoView() {
      graphScrolled = true;
    },
  };
  runScreenLayout.runtimeTablePanelEl = {
    scrollIntoView() {
      tableScrolled = true;
    },
  };
  runScreenLayout.runtimeTextPanelEl = {
    scrollIntoView() {
      textScrolled = true;
    },
  };
  runScreenLayout.bogaeAreaEl = {
    scrollIntoView() {
      spaceScrolled = true;
    },
  };
  runScreenLayout.root = {
    dataset: {},
    querySelector(selector) {
      const buttonPrefix = "#run-tab-btn-";
      const panelPrefix = "#run-tab-panel-";
      if (selector.startsWith(buttonPrefix)) {
        return tabButtons[selector.slice(buttonPrefix.length)] ?? null;
      }
      if (selector.startsWith(panelPrefix)) {
        return tabPanels[selector.slice(panelPrefix.length)] ?? null;
      }
      if (selector === "#select-x-axis") {
        return xAxisSelect;
      }
      if (selector === "#select-y-axis") {
        return yAxisSelect;
      }
      return null;
    },
  };
  runScreenLayout.runtimeStatusEl = { textContent: "", dataset: {} };
  runScreenLayout.sliderPanel = { specs: [] };
  runScreenLayout.lastRuntimeDerived = {
    views: {
      families: ["graph"],
      contract: {
        by_family: {
          graph: { available: true, source: "view_meta" },
        },
      },
    },
  };
  runScreenLayout.focusObserveFamily("graph");
  assert(runScreenLayout.activeRunTab === "graph", "observe focus: graph switches graph tab");
  assert(runScreenLayout.dockTarget === "graph", "observe focus: graph target");
  assert(runScreenLayout.dockTargetSelectEl.value === "graph", "observe focus: graph select sync");
  assert(graphScrolled === true, "observe focus: graph scroll");
  assert(xAxisFocused === true, "observe focus: graph x axis focus");
  assert(
    runScreenLayout.runtimeStatusEl.textContent.includes("가이드: 권장: 카드 클릭 후 x/y축과 슬라이더를 조정해 추세를 확인하세요."),
    "observe focus: graph guide status",
  );
  assert(
    selectedAxesPayload?.xKey === "t" && selectedAxesPayload?.yKey === "x",
    "observe focus: graph axis auto selection",
  );
  runScreenLayout.lastRuntimeDerived = {
    views: {
      families: ["graph"],
      contract: {
        by_family: {
          graph: { available: true, source: "observation_output" },
        },
      },
    },
  };
  runScreenLayout.focusObserveFamily("graph");
  assert(
    runScreenLayout.runtimeStatusEl.textContent.includes("구조 출력 소스(strict)로 전환하세요."),
    "observe focus: non-strict guide status",
  );
  runScreenLayout.focusObserveFamily("table");
  assert(tableScrolled === true, "observe focus: table scroll");
  runScreenLayout.focusObserveFamily("text");
  assert(textScrolled === true, "observe focus: text scroll");
  runScreenLayout.focusObserveFamily("space2d");
  assert(runScreenLayout.dockTarget === "space2d", "observe focus: space2d target");
  assert(runScreenLayout.dockTargetSelectEl.value === "space2d", "observe focus: space2d select sync");
  assert(spaceScrolled === true, "observe focus: space2d scroll");
  const runScreenOutputAction = new RunScreen({
    root: null,
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  let switchedTab = "";
  let selectedRange = null;
  let guideMessage = "";
  runScreenOutputAction.switchRunTab = (tab) => {
    switchedTab = String(tab ?? "");
    return switchedTab;
  };
  runScreenOutputAction.runDdnPreviewEl = {
    value: "설정 {\n  보임 { table.row { 키: 속도. 값: 1.2. } }\n}\n",
    focus() {},
    setSelectionRange(start, end) {
      selectedRange = [Number(start), Number(end)];
    },
  };
  runScreenOutputAction.baseDdn = String(runScreenOutputAction.runDdnPreviewEl.value ?? "");
  runScreenOutputAction.setObserveGuideStatus = (message) => {
    guideMessage = String(message ?? "");
  };
  const outputActionOk = runScreenOutputAction.openObserveOutputRowsInDdn({ observeToken: "table.row" });
  assert(outputActionOk === true, "observe output action: result true");
  assert(switchedTab === "output", "observe output action: switch output");
  assert(Array.isArray(selectedRange) && selectedRange[0] >= 0, "observe output action: token focused");
  assert(guideMessage.includes("DDN 편집 영역으로 이동"), "observe output action: guide message");
  assert(guideMessage.includes("L2"), "observe output action: line guide");
  const runScreenOnboard = new RunScreen({
    root: { dataset: {} },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  let focusedFamilyFromOnboard = "";
  runScreenOnboard.dockTargetSelectEl = { value: "" };
  runScreenOnboard.switchRunTab = (tabId) => {
    runScreenOnboard.activeRunTab = String(tabId ?? "");
  };
  runScreenOnboard.focusObserveFamily = (family) => {
    focusedFamilyFromOnboard = String(family ?? "");
    runScreenOnboard.activeRunTab = "graph";
    return true;
  };
  runScreenOnboard.setRunOnboardingStatus = () => {};
  runScreenOnboard.syncDockGuideToggles = () => {};
  runScreenOnboard.applyDockGuideToggles = () => {};
  runScreenOnboard.getLessonUiPref = () => ({});
  runScreenOnboard.persistUiPrefs = () => {};
  runScreenOnboard.applyRunOnboardingProfile("student", { persist: false });
  assert(runScreenOnboard.activeRunTab === "overlay", "onboarding student: overlay tab");
  assert(runScreenOnboard.dockTarget === "space2d", "onboarding student: dock target");
  assert(runScreenOnboard.dockTargetSelectEl.value === "space2d", "onboarding student: select sync");
  runScreenOnboard.applyRunOnboardingProfile("teacher", { persist: false });
  assert(focusedFamilyFromOnboard === "graph", "onboarding teacher: focus graph family");
  assert(runScreenOnboard.activeRunTab === "graph", "onboarding teacher: graph tab");
  const autoExecuteClassState = {};
  const runScreenAutoExecute = new RunScreen({
    root: null,
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenAutoExecute.layoutEl = {
    classList: {
      toggle(name, enabled) {
        autoExecuteClassState[String(name)] = Boolean(enabled);
      },
    },
  };
  runScreenAutoExecute.syncInitialBogaeShellVisibility(true);
  assert(
    autoExecuteClassState["run-layout--keep-bogae-shell"] === true,
    "initial bogae shell class enabled",
  );
  let autoExecuteRestartCount = 0;
  runScreenAutoExecute.lesson = { id: "auto_execute_case" };
  runScreenAutoExecute.restart = async () => {
    autoExecuteRestartCount += 1;
    return true;
  };
  runScreenAutoExecute.requestAutoExecute();
  await Promise.resolve();
  assert(autoExecuteRestartCount === 0, "auto execute waits until screen visible");
  runScreenAutoExecute.screenVisible = true;
  runScreenAutoExecute.consumePendingAutoExecute();
  await Promise.resolve();
  assert(autoExecuteRestartCount === 1, "auto execute triggers restart after screen visible");
  const runScreenObserveHint = new RunScreen({
    root: null,
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenObserveHint.runObserveSummaryEl = { dataset: {} };
  runScreenObserveHint.runObserveSummaryTextEl = { textContent: "" };
  runScreenObserveHint.runObserveOutputMetaEl = { textContent: "" };
  runScreenObserveHint.runObserveOutputBodyEl = { innerHTML: "" };
  runScreenObserveHint.updateObserveSummary({
    observation: { channels: [{ key: "t" }] },
    outputRows: [{ key: "속도", value: "1.23" }],
    views: {
      graph: {
        series: [{ id: "y", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }] }],
        meta: { source: "view_meta" },
      },
      families: ["graph"],
      contract: {
        by_family: {
          graph: { available: true, source: "view_meta" },
        },
      },
    },
  });
  assert(
    runScreenObserveHint.runObserveSummaryTextEl.textContent.includes("관찰채널 1개"),
    "observe hint: one-line summary rendered",
  );
  assert(
    runScreenObserveHint.runObserveSummaryTextEl.textContent.includes("보임표 1행"),
    "observe hint: output row metric included",
  );
  assert(
    runScreenObserveHint.runObserveOutputBodyEl.innerHTML.includes("속도"),
    "observe hint: output rows panel preserved",
  );
  runScreenObserveHint.updateObserveSummary({
    observation: { channels: [{ key: "t" }] },
    views: {
      graph: {
        series: [{ id: "y", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }] }],
        meta: { source: "observation_output" },
      },
      families: ["graph"],
      contract: {
        by_family: {
          graph: { available: true, source: "observation_output" },
        },
      },
    },
  });
  assert(
    runScreenObserveHint.runObserveSummaryTextEl.textContent.includes("소스경고 1개"),
    "observe hint: non-strict warning summarized",
  );
  runScreenLayout.runtimeTickCounter = 7;
  runScreenLayout.runtimeTimeValue = 0.24;
  runScreenLayout.playbackPaused = true;
  runScreenLayout.playbackSpeed = 1.5;
  runScreenLayout.dockSpeedSelectEl = { value: "1" };
  runScreenLayout.dockTimeCursorEl = { max: "0", value: "0" };
  runScreenLayout.dockTimeTextEl = { textContent: "" };
  runScreenLayout.syncDockTimeUi();
  assert(runScreenLayout.dockSpeedSelectEl.value === "1.5", "run dock time: speed sync");
  assert(runScreenLayout.dockTimeCursorEl.value === "7", "run dock time: cursor sync");
  assert(
    runScreenLayout.dockTimeTextEl.textContent.includes("일시정지"),
    "run dock time: paused label",
  );

  const runScreenText = new RunScreen({
    root: { dataset: {} },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenText.lessonLayoutProfile = resolveRunLayoutProfile(["graph", "text"]);
  runScreenText.lesson = { requiredViews: ["graph", "text"] };
  runScreenText.graphPanelEl = {
    classList: {
      toggle() {},
    },
  };
  runScreenText.runtimeTablePanelEl = {
    classList: {
      toggle() {},
    },
  };
  runScreenText.runtimeTextPanelEl = {
    classList: {
      toggle(name, enabled) {
        this.last = [name, enabled];
      },
    },
  };
  runScreenText.runtimeTextBodyEl = { innerHTML: "" };
  runScreenText.overlayToggleBtn = { disabled: false, title: "" };
  const renderedText = runScreenText.renderRuntimeText("# 제목\n- 요약");
  assert(renderedText === true, "run text panel: rendered");
  assert(runScreenText.runtimeTextBodyEl.innerHTML.includes("<h1>제목</h1>"), "run text panel: heading html");
  assert(runScreenText.runtimeTextBodyEl.innerHTML.includes("<li>요약</li>"), "run text panel: list html");
  assert(runScreenText.overlayToggleBtn.disabled === false, "run text panel: overlay tab enabled with markdown");
  assert(
    String(runScreenText.overlayToggleBtn.title).includes("겹보기 탭"),
    "run text panel: overlay tab title",
  );
  assert(runScreenText.runtimeTextPanelEl.classList.last?.[1] === false, "run text panel: visible when required");

  const structureMarkdown = summarizeRuntimeStructureMarkdown({
    meta: { title: "회로 구조" },
    nodes: [{ id: "battery", label: "Battery" }, { id: "lamp", label: "Lamp" }],
    edges: [{ from: "battery", to: "lamp", directed: true }],
  });
  assert(structureMarkdown.includes("## 구조 요약"), "run structure summary: heading");
  assert(structureMarkdown.includes("- 제목: 회로 구조"), "run structure summary: title");
  assert(structureMarkdown.includes("- 노드: 2개"), "run structure summary: node count");
  assert(structureMarkdown.includes("- 간선: 1개"), "run structure summary: edge count");
  const structurePreviewHtml = buildRuntimeStructurePreviewHtml({
    meta: { title: "회로 구조" },
    nodes: [{ id: "battery", label: "Battery" }, { id: "lamp", label: "Lamp" }],
    edges: [{ from: "battery", to: "lamp", directed: true }],
  });
  assert(structurePreviewHtml.includes("<svg"), "run structure preview: svg");
  assert(structurePreviewHtml.includes("runtime-structure-node"), "run structure preview: node markup");
  const graphMarkdown = summarizeRuntimeGraphMarkdown({
    meta: { title: "속도 변화" },
    series: [{ id: "v", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }, { x: 2, y: 1.5 }] }],
  });
  assert(graphMarkdown.includes("## 그래프 요약"), "run graph summary: heading");
  assert(graphMarkdown.includes("- 제목: 속도 변화"), "run graph summary: title");
  assert(graphMarkdown.includes("- 계열: 1개"), "run graph summary: series count");
  const graphPreviewHtml = buildRuntimeGraphPreviewHtml({
    meta: { title: "속도 변화" },
    series: [{ id: "v", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }, { x: 2, y: 1.5 }] }],
  });
  assert(graphPreviewHtml.includes("<svg"), "run graph preview: svg");
  assert(graphPreviewHtml.includes("runtime-graph-line"), "run graph preview: line markup");

  const runScreenStructure = new RunScreen({
    root: { dataset: {} },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenStructure.screenVisible = true;
  runScreenStructure.lessonLayoutProfile = resolveRunLayoutProfile(["structure"]);
  runScreenStructure.lesson = { requiredViews: ["structure"] };
  runScreenStructure.dotbogi = { appendObservation() {} };
  runScreenStructure.bogae = { render() {}, renderConsoleGrid() {} };
  runScreenStructure.graphPanelEl = { classList: { toggle() {} } };
  runScreenStructure.runtimeTablePanelEl = { classList: { toggle() {} } };
  runScreenStructure.runtimeTextPanelEl = {
    classList: {
      toggle() {},
    },
  };
  runScreenStructure.runtimeTextBodyEl = { innerHTML: "" };
  runScreenStructure.runtimeStatusEl = { textContent: "" };
  runScreenStructure.runtimeTextPanelEl = { dataset: {}, title: "", classList: { toggle() {} } };
  runScreenStructure.overlayToggleBtn = { disabled: false, title: "" };
  runScreenStructure.overlay = {
    content: "",
    setContent(value) {
      this.content = String(value ?? "");
    },
  };
  runScreenStructure.renderCurrentRuntimeTable = () => false;
  runScreenStructure.applyRuntimeDerived({
    observation: null,
    views: {
      graph: null,
      table: null,
      text: null,
      space2d: null,
      structure: {
        meta: { title: "회로 구조" },
        nodes: [{ id: "battery", label: "Battery" }, { id: "lamp", label: "Lamp" }],
        edges: [{ from: "battery", to: "lamp", directed: true }],
      },
    },
  });
  assert(runScreenStructure.runtimeTextBodyEl.innerHTML.includes("구조 요약"), "run structure panel: renders summary html");
  assert(runScreenStructure.runtimeTextBodyEl.innerHTML.includes("runtime-structure-preview"), "run structure panel: renders structure preview");
  assert(runScreenStructure.runtimeTextBodyEl.innerHTML.includes("runtime-preview-summary"), "run structure panel: summary strip");
  assert(
    runScreenStructure.runtimeTextBodyEl.innerHTML.includes('data-preview-family="structure"'),
    "run structure panel: preview family metadata",
  );
  assert(
    runScreenStructure.runtimeTextBodyEl.innerHTML.includes("회로 구조 · 노드 2개 · 간선 1개"),
    "run structure panel: preview tooltip metadata",
  );
  assert(
    runScreenStructure.runtimeTextPanelEl.dataset.previewFamily === "structure",
    "run structure panel: text panel family metadata",
  );
  assert(
    runScreenStructure.runtimeTextPanelEl.dataset.previewHeader === "구조 미리보기",
    "run structure panel: text panel header metadata",
  );
  assert(
    runScreenStructure.runtimeStatusEl.textContent.includes("대표보기: 회로 구조 · 노드 2개 · 간선 1개") === false,
    "run structure panel: runtime hint keeps preview summary out of top status",
  );
  assert(runScreenStructure.overlay.content.includes("회로 구조"), "run structure panel: overlay content updated");
  const runScreenGraph = new RunScreen({
    root: { dataset: {} },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenGraph.lessonLayoutProfile = resolveRunLayoutProfile(["graph", "text"]);
  runScreenGraph.lesson = { requiredViews: ["graph", "text"] };
  runScreenGraph.runtimeTextPanelEl = { dataset: {}, title: "", classList: { toggle() {} } };
  runScreenGraph.runtimeTextBodyEl = { innerHTML: "" };
  runScreenGraph.runtimeStatusEl = { textContent: "" };
  runScreenGraph.graphPanelEl = { classList: { toggle() {} } };
  runScreenGraph.runtimeTablePanelEl = { classList: { toggle() {} } };
  runScreenGraph.overlayToggleBtn = { disabled: false, title: "" };
  const renderedGraphText = runScreenGraph.renderRuntimeTextContent({
    markdown: "그래프 설명",
    graph: {
      meta: { title: "속도 변화" },
      series: [{ id: "v", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }, { x: 2, y: 1.5 }] }],
    },
  });
  assert(renderedGraphText === true, "run graph text panel: rendered");
  assert(runScreenGraph.runtimeTextBodyEl.innerHTML.includes("runtime-graph-preview"), "run graph text panel: preview html");
  assert(runScreenGraph.runtimeTextBodyEl.innerHTML.includes("runtime-preview-summary"), "run graph text panel: summary strip");
  assert(
    runScreenGraph.runtimeTextBodyEl.innerHTML.includes('data-preview-family="graph"'),
    "run graph text panel: preview family metadata",
  );
  assert(
    runScreenGraph.runtimeTextBodyEl.innerHTML.includes("속도 변화 · 계열 1개 · 점 3개"),
    "run graph text panel: preview tooltip metadata",
  );
  assert(
    runScreenGraph.runtimeTextPanelEl.dataset.previewFamily === "graph",
    "run graph text panel: text panel family metadata",
  );
  assert(
    runScreenGraph.runtimeStatusEl.textContent.includes("대표보기: 속도 변화 · 계열 1개 · 점 3개") === false,
    "run graph text panel: runtime hint keeps preview summary out of top status",
  );
  assert(runScreenGraph.runtimeTextBodyEl.innerHTML.includes("runtime-text-markdown"), "run graph text panel: markdown html");

  const runScreenFallbackGraph = new RunScreen({
    root: { dataset: {} },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenFallbackGraph.screenVisible = true;
  runScreenFallbackGraph.lessonLayoutProfile = resolveRunLayoutProfile(["graph"]);
  runScreenFallbackGraph.lesson = { requiredViews: [] };
  runScreenFallbackGraph.dotbogi = { appendObservation() {} };
  runScreenFallbackGraph.bogae = { render() {}, renderConsoleGrid() {} };
  runScreenFallbackGraph.graphPanelEl = { classList: { toggle() {} } };
  runScreenFallbackGraph.runtimeTablePanelEl = { classList: { toggle() {} } };
  runScreenFallbackGraph.runtimeTextPanelEl = { dataset: {}, title: "", classList: { toggle() {} } };
  runScreenFallbackGraph.runtimeTextBodyEl = { innerHTML: "" };
  runScreenFallbackGraph.runtimeStatusEl = { textContent: "" };
  runScreenFallbackGraph.runMainGraphHostEl = { classList: { toggle() {} }, innerHTML: "" };
  runScreenFallbackGraph.runMainConsoleHostEl = { classList: { toggle() {} }, innerHTML: "" };
  runScreenFallbackGraph.layoutEl = { classList: { toggle() {} } };
  runScreenFallbackGraph.overlayToggleBtn = { disabled: false, title: "" };
  runScreenFallbackGraph.overlay = {
    content: "",
    setContent(value) {
      this.content = String(value ?? "");
    },
  };
  runScreenFallbackGraph.renderCurrentRuntimeTable = () => false;
  const appliedFallbackGraph = runScreenFallbackGraph.applyRuntimeDerived({
    observation: { values: { 합: 23, x: 15, y: 8 } },
    outputRows: [
      { key: "합", value: "23", source: "observation" },
      { key: "x", value: "15", source: "observation" },
      { key: "y", value: "8", source: "observation" },
    ],
    outputLines: [],
    outputLog: [],
    views: {
      graph: {
        axis: { x_min: -1, x_max: 1, y_min: 14, y_max: 24 },
        series: [{ id: "합", points: [{ x: 0, y: 23 }] }],
      },
      graphSource: "observation-fallback",
      families: ["graph"],
      contract: {
        by_family: {
          graph: { available: true, source: "observation-fallback" },
        },
      },
    },
  }, { forceView: true });
  assert(appliedFallbackGraph.mainVisualMode === "console-grid", "fallback graph derived: main visual console-grid");
  assert(
    !runScreenFallbackGraph.runtimeTextBodyEl.innerHTML.includes("runtime-graph-preview"),
    "fallback graph derived: text preview suppresses graph preview",
  );
  assert(
    runScreenFallbackGraph.runtimeStatusEl.textContent.includes("대표보기") === false,
    "fallback graph derived: runtime hint does not claim graph summary",
  );

  const runScreenDtGuard = new RunScreen({
    root: null,
    wasmState: { fpsLimit: 30, dtMax: 0 },
  });
  const guardedInput = runScreenDtGuard.getStepInput();
  assert(Number.isFinite(guardedInput.dt) && guardedInput.dt > 0, "run step input: dt guard for zero dtMax");
  runScreenDtGuard.wasmState.inputEnabled = true;
  runScreenDtGuard.heldInputMask = 1 << 1;
  runScreenDtGuard.pulsePressedMask = 1 << 7;
  runScreenDtGuard.lastInputToken = "KeyZ";
  const keyedInput = runScreenDtGuard.getStepInput();
  assert(keyedInput.keys === ((1 << 1) | (1 << 7)), "run step input: held + pulse keys");
  assert(keyedInput.lastKey === "KeyZ", "run step input: last key token");
  const consumedInput = runScreenDtGuard.getStepInput();
  assert(consumedInput.keys === (1 << 1), "run step input: pulse consumed after one frame");
  assert(consumedInput.lastKey === "", "run step input: last key consumed after one frame");
  runScreenDtGuard.wasmState.inputEnabled = false;
  runScreenDtGuard.heldInputMask = 1 << 3;
  runScreenDtGuard.pulsePressedMask = 1 << 4;
  runScreenDtGuard.lastInputToken = "ArrowRight";
  const disabledInput = runScreenDtGuard.getStepInput();
  assert(disabledInput.keys === 0, "run step input: disabled keys");
  assert(disabledInput.lastKey === "", "run step input: disabled last key");

  const runScreenWarnHint = new RunScreen({
    root: null,
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  const warnDiagnosticsState = { hidden: true };
  const warnBadgeState = { hidden: true };
  runScreenWarnHint.runMirrorDiagnosticsEl = {
    open: true,
    classList: {
      add(name) {
        if (name === "hidden") warnDiagnosticsState.hidden = true;
      },
      remove(name) {
        if (name === "hidden") warnDiagnosticsState.hidden = false;
      },
    },
  };
  runScreenWarnHint.runMirrorDiagnosticsSummaryEl = { title: "" };
  runScreenWarnHint.runMirrorDiagnosticsChipsEl = { innerHTML: "" };
  runScreenWarnHint.runMirrorDiagnosticsBodyEl = { textContent: "", innerHTML: "" };
  runScreenWarnHint.bogaeWarnBadgeEl = {
    dataset: {},
    title: "",
    classList: {
      toggle(name, enabled) {
        if (name === "hidden") warnBadgeState.hidden = Boolean(enabled);
      },
    },
  };
  runScreenWarnHint.runConsoleWarningSummaryEl = { textContent: "", dataset: {} };
  runScreenWarnHint.runExecTechEl = {
    open: true,
    classList: {
      toggle() {},
    },
  };
  runScreenWarnHint.runExecTechBodyEl = { textContent: "" };
  runScreenWarnHint.runtimeStatusEl = { textContent: "" };
  runScreenWarnHint.sliderPanel = { specs: [] };
  runScreenWarnHint.lastSpace2dMode = "none";
  runScreenWarnHint.runDdnPreviewEl = { value: "" };
  runScreenWarnHint.setParseWarnings([
    {
      code: "W_BLOCK_HEADER_COLON_DEPRECATED",
      message: "deprecated",
      span: { start: 0, end: 1 },
    },
  ]);
  assert(warnDiagnosticsState.hidden === false, "mirror diagnostics: visible on warning");
  assert(runScreenWarnHint.runExecTechEl.open === false, "mirror diagnostics: tech details collapsed by default");
  assert(warnBadgeState.hidden === false, "warning badge: visible on warning");
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("[문법]"),
    "mirror diagnostics: warning category rendered",
  );
  assert(
    runScreenWarnHint.runConsoleWarningSummaryEl.textContent.includes("경고 1건"),
    "console summary: warning count rendered",
  );
  runScreenWarnHint.updateRuntimeHint("실행 경로: wasm(strict)");
  assert(
    runScreenWarnHint.runtimeStatusEl.textContent.includes("점검 필요:"),
    "run hint: parse warning summary",
  );
  assert(
    runScreenWarnHint.runtimeStatusEl.textContent.includes("블록 헤더의 ':' 표기는 구식입니다."),
    "run hint: warning primary user message rendered",
  );
  runScreenWarnHint.setParseWarnings([]);
  assert(warnDiagnosticsState.hidden === true, "mirror diagnostics: hidden when warning cleared");
  assert(
    warnBadgeState.hidden === true,
    "warning badge: hidden when warning cleared",
  );
  runScreenWarnHint.updateRuntimeHint("실행 경로: wasm(strict)");
  assert(
    !runScreenWarnHint.runtimeStatusEl.textContent.includes("점검 필요:"),
    "run hint: parse warning summary cleared",
  );
  runScreenWarnHint.setParseWarnings([
    {
      code: "E_RUNTIME_EXEC_FAILED",
      message: "WASM/server failed",
      span: { start: 0, end: 1 },
    },
  ]);
  runScreenWarnHint.updateRuntimeHint("실행 경로: wasm(strict)");
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("[입력]"),
    "mirror diagnostics: runtime warning category rendered",
  );
  assert(
    runScreenWarnHint.runtimeStatusEl.textContent.includes("실행에 실패했습니다."),
    "run hint: runtime warning message rendered",
  );
  runScreenWarnHint.setObserveGuideStatus("권장: 카드 클릭 후 x/y축과 슬라이더를 조정해 추세를 확인하세요.", { ttlMs: 0 });
  runScreenWarnHint.updateRuntimeHint("실행 경로: wasm(strict)");
  assert(
    runScreenWarnHint.runtimeStatusEl.textContent.includes("점검 필요: 실행에 실패했습니다."),
    "run hint: warning message takes priority over observe guide",
  );
  assert(
    !runScreenWarnHint.runtimeStatusEl.textContent.includes("권장: 카드 클릭 후 x/y축"),
    "run hint: observe guide hidden while warning exists",
  );
  runScreenWarnHint.setParseWarnings([]);
  runScreenWarnHint.updateRuntimeHint("실행 경로: wasm(strict)");
  assert(
    runScreenWarnHint.runtimeStatusEl.textContent.includes("권장: 카드 클릭 후 x/y축과 슬라이더를 조정해 추세를 확인하세요."),
    "run hint: observe guide restored when warning cleared",
  );
  runScreenWarnHint.setParseWarnings([
    {
      code: "",
      message: "expected '.' or newline",
      span: { start: 0, end: 1 },
    },
  ]);
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("문장 끝에"),
    "mirror diagnostics: technical message mapped to korean user message",
  );
  runScreenWarnHint.setParseWarnings([
    {
      code: "E_PARSE_EXPECTED_PATH",
      message: "expected path",
      span: { start: 0, end: 1 },
    },
  ]);
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("경로가 필요한 위치입니다. 대상 경로를 지정해 주세요."),
    "mirror diagnostics: parse code mapped to korean user message",
  );
  runScreenWarnHint.setParseWarnings([
    {
      code: "E_PARSE_CALL_PIN_DUPLICATE",
      message: "duplicate pin",
      span: { start: 0, end: 1 },
    },
  ]);
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("동일한 핀(pin)이 중복되었습니다."),
    "mirror diagnostics: parse call pin mapped to korean user message",
  );
  runScreenWarnHint.setParseWarnings([
    {
      code: "E_IMPORT_ALIAS_DUPLICATE",
      message: "alias duplicate",
      span: { start: 0, end: 1 },
    },
  ]);
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("import 별칭이 중복되었습니다."),
    "mirror diagnostics: import alias mapped to korean user message",
  );
  runScreenWarnHint.setParseWarnings([
    {
      code: "E_RECEIVE_OUTSIDE_IMJA",
      message: "receive outside",
      span: { start: 0, end: 1 },
    },
  ]);
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("임자 블록 안에서만"),
    "mirror diagnostics: receive outside mapped to korean user message",
  );
  runScreenWarnHint.setParseWarnings([
    {
      technical_code: "E_API_RUN_DDN_TEXT_REQUIRED",
      technical_message: "ddn_text required",
      user_message: "실행할 DDN 본문이 비어 있습니다. 입력을 확인해 주세요.",
      span: { line: 1, column: 1 },
    },
  ]);
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("실행할 DDN 본문이 비어 있습니다."),
    "mirror diagnostics: api contract user_message preferred",
  );
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("ddn_text required"),
    "mirror diagnostics: api contract technical_message rendered",
  );
  assert(
    runScreenWarnHint.runMirrorDiagnosticsBodyEl.innerHTML.includes("E_API_RUN_DDN_TEXT_REQUIRED"),
    "mirror diagnostics: api contract technical_code rendered",
  );

  const runScreenWarnHotkey = new RunScreen({
    root: { classList: { contains() { return false; } } },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenWarnHotkey.isEditableTarget = () => false;
  let preventDefaultCount = 0;
  let ddnFocusCount = 0;
  let inspectorFocusCount = 0;
  let switchedWarningTab = "";
  runScreenWarnHotkey.focusRunDdnEditor = () => {
    ddnFocusCount += 1;
  };
  runScreenWarnHotkey.switchRunTab = (tabId) => {
    switchedWarningTab = String(tabId ?? "");
    return switchedWarningTab;
  };
  runScreenWarnHotkey.runInspectorMetaEl = { focus() { inspectorFocusCount += 1; } };
  runScreenWarnHotkey.setParseWarnings([{ code: "W_BLOCK_HEADER_COLON_DEPRECATED", message: "deprecated" }]);
  runScreenWarnHotkey.handleViewHotkeys({
    key: "d",
    altKey: true,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    target: null,
    preventDefault() {
      preventDefaultCount += 1;
    },
  });
  assert(ddnFocusCount === 1, "warning hotkey: Alt+D focuses ddn editor");
  assert(switchedWarningTab === "console", "warning hotkey: Alt+D switches console tab");
  assert(preventDefaultCount === 1, "warning hotkey: Alt+D prevents default");
  runScreenWarnHotkey.handleViewHotkeys({
    key: "i",
    altKey: true,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    target: null,
    preventDefault() {
      preventDefaultCount += 1;
    },
  });
  assert(inspectorFocusCount === 1, "warning hotkey: Alt+I focuses inspector");
  assert(switchedWarningTab === "mirror", "warning hotkey: Alt+I switches mirror tab");
  assert(preventDefaultCount === 2, "warning hotkey: Alt+I prevents default");

  const normalizedTable = normalizeRuntimeTableView({
    schema: "seamgrim.table.v0",
    columns: [
      { key: "t", type: "number" },
      { key: "celsius", type: "string" },
      { key: "fahrenheit", type: "string" },
    ],
    rows: [
      { t: 0, celsius: "80.0@C", fahrenheit: "176.0@F" },
      { t: 4, celsius: "40.0@C", fahrenheit: "104.0@F" },
    ],
  });
  assert(normalizedTable?.columns?.length === 3, "runtime table normalize: columns");
  assert(normalizedTable?.rows?.[0]?.cells?.[1] === "80.0@C", "runtime table normalize: string cell");
  assert(summarizeRuntimeTableView(normalizedTable) === "3열 · 2행 · table.v0", "runtime table summary: full rows");
  const truncatedSummary = summarizeRuntimeTableView(
    normalizeRuntimeTableView(
      {
        schema: "seamgrim.table.v0",
        meta: { source: "legacy" },
        columns: [{ key: "x", type: "number" }],
        rows: Array.from({ length: 30 }, (_, index) => ({ x: index })),
      },
      { maxRows: 24 },
    ),
  );
  assert(truncatedSummary === "1열 · 30행 중 24행 표시 · table.v0 · legacy", "runtime table summary: truncated rows");
  assert(resolveRuntimeTableCellMaxChars({ clientWidth: 420 }) === 48, "runtime table max chars: width clamp");
  assert(resolveRuntimeTableCellMaxChars({ clientWidth: 180 }) === 14, "runtime table max chars: width estimate");
  assert(resolveRuntimeTableCellMaxChars({ dataset: { cellMaxChars: "18" } }) === 18, "runtime table max chars: dataset override");
  const resizeTable = {
    columns: [{ key: "label", type: "string" }],
    rows: [{ label: "point-with-very-long-label-for-tooltip-check-1234567890" }],
  };
  const runScreenResize = new RunScreen({
    root: null,
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  runScreenResize.screenVisible = true;
  runScreenResize.runtimeTablePanelEl = {
    classList: {
      toggle() {},
    },
  };
  runScreenResize.runtimeTableMetaEl = { textContent: "" };
  runScreenResize.runtimeTableEl = { innerHTML: "", clientWidth: 300 };
  runScreenResize.lastRuntimeDerived = { views: { table: resizeTable } };
  runScreenResize.renderCurrentRuntimeTable(resizeTable);
  assert(
    runScreenResize.runtimeTableEl.innerHTML.includes("point-with-very-long-label-for…"),
    "runtime table resize: initial width truncation",
  );
  runScreenResize.runtimeTableEl.clientWidth = 180;
  const rerenderedTable = runScreenResize.refreshRuntimeTableForCurrentWidth();
  assert(rerenderedTable === true, "runtime table resize: rerender on width change");
  assert(
    runScreenResize.runtimeTableEl.innerHTML.includes("point-with-ve…"),
    "runtime table resize: narrower width truncation",
  );
  assert(
    runScreenResize.refreshRuntimeTableForCurrentWidth() === false,
    "runtime table resize: skip rerender when width unchanged",
  );
  const originalResizeObserver = globalThis.ResizeObserver;
  const originalWindow = globalThis.window;
  const resizeEvents = [];
  try {
    delete globalThis.ResizeObserver;
  } catch (_) {
    globalThis.ResizeObserver = undefined;
  }
  try {
    globalThis.window = {
      addEventListener(type, handler) {
        resizeEvents.push(["add", type, typeof handler]);
      },
      removeEventListener(type, handler) {
        resizeEvents.push(["remove", type, typeof handler]);
      },
    };
    const runScreenResizeFallback = new RunScreen({
      root: null,
      wasmState: { fpsLimit: 30, dtMax: 0.1 },
    });
    runScreenResizeFallback.runtimeTablePanelEl = {};
    runScreenResizeFallback.installRuntimeTableResizeObserver();
    assert(
      resizeEvents.some(([kind, type]) => kind === "add" && type === "resize"),
      "runtime table resize fallback: installs window resize listener",
    );
    runScreenResizeFallback.installRuntimeTableResizeObserver();
    assert(
      resizeEvents.some(([kind, type]) => kind === "remove" && type === "resize"),
      "runtime table resize fallback: removes previous window resize listener before reinstall",
    );
  } finally {
    globalThis.window = originalWindow;
    if (typeof originalResizeObserver === "undefined") {
      try {
        delete globalThis.ResizeObserver;
      } catch (_) {
        globalThis.ResizeObserver = undefined;
      }
    } else {
      globalThis.ResizeObserver = originalResizeObserver;
    }
  }
  const fakeContainer = { innerHTML: "" };
  const renderedTable = renderRuntimeTable(fakeContainer, {
    columns: [
      { key: "t", type: "number" },
      { key: "celsius", type: "string" },
      { key: "fahrenheit", type: "string" },
    ],
    rows: [
      { t: 0, celsius: "80.0@C", fahrenheit: "176.0@F" },
      { t: 4, celsius: "40.0@C", fahrenheit: "104.0@F" },
    ],
  });
  assert(renderedTable === true, "runtime table render: success");
  assert(fakeContainer.innerHTML.includes("<table"), "runtime table render: table html");
  assert(fakeContainer.innerHTML.includes("80.0@C"), "runtime table render: celsius text");
  assert(fakeContainer.innerHTML.includes("176.0@F"), "runtime table render: fahrenheit text");
  const renderedLinearTable = renderRuntimeTable(fakeContainer, {
    columns: [
      { key: "x", type: "number" },
      { key: "y", type: "number" },
      { key: "label", type: "string" },
    ],
    rows: [
      { x: -2, y: -4, label: "point(-2,-4)" },
      { x: 4, y: 5, label: "point(4,5)" },
    ],
  });
  assert(renderedLinearTable === true, "runtime table render: linear success");
  assert(fakeContainer.innerHTML.includes("point(-2,-4)"), "runtime table render: linear first label");
  assert(fakeContainer.innerHTML.includes("point(4,5)"), "runtime table render: linear last label");
  const renderedLongTextTable = renderRuntimeTable({ innerHTML: "", clientWidth: 227 }, {
    columns: [{ key: "label", type: "string" }],
    rows: [
      { label: "point-with-very-long-label-for-tooltip-check-1234567890" },
    ],
  });
  assert(renderedLongTextTable === true, "runtime table render: long text success");
  const renderedLongTextContainer = { innerHTML: "", clientWidth: 227 };
  renderRuntimeTable(renderedLongTextContainer, {
    columns: [{ key: "label", type: "string" }],
    rows: [
      { label: "point-with-very-long-label-for-tooltip-check-1234567890" },
    ],
  });
  assert(renderedLongTextContainer.innerHTML.includes("runtime-table-celltext"), "runtime table render: long text span");
  assert(renderedLongTextContainer.innerHTML.includes("point-with-very-long…"), "runtime table render: long text truncated");
  assert(
    renderedLongTextContainer.innerHTML.includes('title="point-with-very-long-label-for-tooltip-check-1234567890"'),
    "runtime table render: long text title",
  );

  console.log("seamgrim pendulum bogae fallback runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack || err));
  process.exit(1);
});
