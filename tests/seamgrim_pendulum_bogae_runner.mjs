import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function collectGroupIds(space2d) {
  const shapes = Array.isArray(space2d?.shapes) ? space2d.shapes : [];
  return shapes.map((shape) => String(shape?.group_id ?? "").trim()).filter(Boolean);
}

async function main() {
  const root = process.cwd();
  const runPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const runMod = await import(pathToFileURL(runPath).href);
  const {
    RunScreen,
    normalizeRuntimeTableView,
    resolveRunDockPanelOrder,
    resolveRunDockPanelVisibility,
    resolveRunLayoutProfile,
    resolveRuntimeTableCellMaxChars,
    summarizeRuntimeTableView,
    buildRuntimeStructurePreviewHtml,
    buildRuntimeGraphPreviewHtml,
    summarizeRuntimeGraphMarkdown,
    summarizeRuntimeStructureMarkdown,
    renderRuntimeTable,
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
  assert(typeof resolveRuntimeTableCellMaxChars === "function", "run export: resolveRuntimeTableCellMaxChars");
  assert(typeof buildRuntimeStructurePreviewHtml === "function", "run export: buildRuntimeStructurePreviewHtml");
  assert(typeof buildRuntimeGraphPreviewHtml === "function", "run export: buildRuntimeGraphPreviewHtml");
  assert(typeof summarizeRuntimeTableView === "function", "run export: summarizeRuntimeTableView");
  assert(typeof summarizeRuntimeGraphMarkdown === "function", "run export: summarizeRuntimeGraphMarkdown");
  assert(typeof summarizeRuntimeStructureMarkdown === "function", "run export: summarizeRuntimeStructureMarkdown");
  assert(typeof renderRuntimeTable === "function", "run export: renderRuntimeTable");
  assert(typeof RunScreen === "function", "run export: RunScreen");

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
  assert(
    collectGroupIds(pointFromGraph).join(",") === "graph.axis.x,graph.axis.y,graph.point.focus",
    "point from graph: group_id",
  );

  const graphOnlyLayout = resolveRunLayoutProfile(["graph", "text"]);
  assert(graphOnlyLayout.mode === "dock_only", "run layout profile: graph/text => dock_only");
  assert(graphOnlyLayout.hasSpatial === false, "run layout profile: graph/text spatial false");
  const splitLayout = resolveRunLayoutProfile(["space2d", "graph"]);
  assert(splitLayout.mode === "split", "run layout profile: space2d+graph => split");
  assert(splitLayout.hasSpatial === true, "run layout profile: space2d+graph spatial true");
  const spaceOnlyLayout = resolveRunLayoutProfile(["space2d"]);
  assert(spaceOnlyLayout.mode === "space_primary", "run layout profile: space2d => space_primary");
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
  assert(appliedLayout.mode === "dock_only", "run layout apply: mode");
  assert(toggled.get("run-layout--dock-only") === true, "run layout apply: dock_only class");
  assert(toggled.get("run-layout--split") === false, "run layout apply: split class off");
  assert(runScreenLayout.root.dataset.requiredViews === "graph,text", "run layout apply: dataset requiredViews");
  assert(runScreenLayout.root.dataset.runLayoutMode === "dock_only", "run layout apply: dataset mode");
  assert(runScreenLayout.root.dataset.runDockOrder === "graph,text,table", "run layout apply: dataset dock order");
  assert(typeof runScreenLayout.switchRunTab === "function", "run export: switchRunTab");
  assert(typeof runScreenLayout.syncDockTimeUi === "function", "run export: syncDockTimeUi");
  const tabState = {};
  const makeToggleEl = (key) => ({
    classList: {
      toggle(name, enabled) {
        tabState[`${key}:${name}`] = Boolean(enabled);
      },
    },
  });
  const tabButtons = {
    lesson: makeToggleEl("btn:lesson"),
    ddn: makeToggleEl("btn:ddn"),
    formula: makeToggleEl("btn:formula"),
    inspector: makeToggleEl("btn:inspector"),
  };
  const tabPanels = {
    lesson: makeToggleEl("panel:lesson"),
    ddn: makeToggleEl("panel:ddn"),
    formula: makeToggleEl("panel:formula"),
    inspector: makeToggleEl("panel:inspector"),
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
  runScreenLayout.switchRunTab("ddn");
  assert(runScreenLayout.activeRunTab === "ddn", "run tab switch: active tab");
  assert(tabState["btn:ddn:active"] === true, "run tab switch: ddn button active");
  assert(tabState["panel:ddn:hidden"] === false, "run tab switch: ddn panel shown");
  assert(tabState["panel:lesson:hidden"] === true, "run tab switch: lesson panel hidden");
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
  assert(runScreenText.overlayToggleBtn.disabled === true, "run text panel: overlay toggle disabled without spatial");
  assert(
    String(runScreenText.overlayToggleBtn.title).includes("오른쪽 패널"),
    "run text panel: overlay disabled title",
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
  runScreenStructure.bogae = { render() {} };
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
    runScreenStructure.runtimeStatusEl.textContent.includes("대표보기: 회로 구조 · 노드 2개 · 간선 1개"),
    "run structure panel: runtime hint includes preview summary",
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
    runScreenGraph.runtimeStatusEl.textContent.includes("대표보기: 속도 변화 · 계열 1개 · 점 3개"),
    "run graph text panel: runtime hint includes preview summary",
  );
  assert(runScreenGraph.runtimeTextBodyEl.innerHTML.includes("runtime-text-markdown"), "run graph text panel: markdown html");

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
  runScreenWarnHint.runtimeStatusEl = { textContent: "" };
  runScreenWarnHint.sliderPanel = { specs: [] };
  runScreenWarnHint.lastSpace2dMode = "none";
  runScreenWarnHint.setParseWarnings([
    {
      code: "W_BLOCK_HEADER_COLON_DEPRECATED",
      message: "deprecated",
      span: { start: 0, end: 1 },
    },
  ]);
  runScreenWarnHint.updateRuntimeHint("실행 경로: wasm(strict)");
  assert(
    runScreenWarnHint.runtimeStatusEl.textContent.includes("문법경고: W_BLOCK_HEADER_COLON_DEPRECATED"),
    "run hint: parse warning summary",
  );
  runScreenWarnHint.setParseWarnings([]);
  runScreenWarnHint.updateRuntimeHint("실행 경로: wasm(strict)");
  assert(
    !runScreenWarnHint.runtimeStatusEl.textContent.includes("문법경고:"),
    "run hint: parse warning summary cleared",
  );

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
