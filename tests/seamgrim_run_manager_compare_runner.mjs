import path from "node:path";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function createFakeCanvas() {
  return {
    clientWidth: 640,
    clientHeight: 320,
    width: 640,
    height: 320,
    addEventListener() {},
    getContext() {
      return {
        clearRect() {},
        fillRect() {},
        beginPath() {},
        closePath() {},
        moveTo() {},
        lineTo() {},
        stroke() {},
        fill() {},
        fillText() {},
        arc() {},
        setLineDash() {},
        save() {},
        restore() {},
        setTransform() {},
      };
    },
  };
}

function createFakeSelect() {
  const listeners = new Map();
  const options = [];
  return {
    value: "",
    addEventListener(type, handler) {
      const key = String(type);
      const list = listeners.get(key) ?? [];
      list.push(handler);
      listeners.set(key, list);
    },
    appendChild(child) {
      options.push(child);
      return child;
    },
    set innerHTML(_) {
      options.length = 0;
    },
    get innerHTML() {
      return "";
    },
  };
}

function createGraph(id, points) {
  return {
    axis: { x_min: 0, x_max: 4, y_min: 0, y_max: 10 },
    series: [{ id, points }],
  };
}

async function main() {
  const root = process.cwd();
  const runPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const dotbogiPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/dotbogi.js");

  globalThis.window = globalThis.window ?? {
    addEventListener() {},
    removeEventListener() {},
    devicePixelRatio: 1,
  };
  globalThis.document = globalThis.document ?? {
    createElement() {
      return { value: "", textContent: "" };
    },
  };

  const runMod = await import(pathToFileURL(runPath).href);
  const dotbogiMod = await import(pathToFileURL(dotbogiPath).href);
  const { RunScreen, buildRunManagerDisplayState } = runMod;
  const { DotbogiPanel } = dotbogiMod;

  assert(typeof buildRunManagerDisplayState === "function", "run manager compare: helper export");

  const screen = new RunScreen({
    root: null,
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  const activeRun = screen.normalizeRunManagerRun({
    id: "run-active",
    label: "진자 #1",
    visible: true,
    graph: createGraph("theta", [{ x: 0, y: 1 }, { x: 1, y: 2 }]),
    hash: { result: "blake3:activehash" },
  }, 0);
  const previousRun = screen.normalizeRunManagerRun({
    id: "run-prev",
    label: "진자 #2",
    visible: true,
    graph: createGraph("theta", [{ x: 0, y: 2 }, { x: 1, y: 3 }]),
    hash: { result: "blake3:prevhash" },
  }, 1);
  const hiddenRun = screen.normalizeRunManagerRun({
    id: "run-hidden",
    label: "진자 #3",
    visible: false,
    graph: createGraph("theta", [{ x: 0, y: 3 }, { x: 1, y: 4 }]),
    hash: { result: "blake3:hiddenhash" },
  }, 2);

  const display = buildRunManagerDisplayState({
    overlayRuns: [activeRun, previousRun, hiddenRun],
    activeOverlayRunId: "run-active",
  });
  assert(display.rows.length === 3, "run manager compare: row count");
  assert(display.rows[0].rowColor === "#22d3ee", "run manager compare: active row color");
  assert(display.overlaySeries.length === 1, "run manager compare: active row excluded from overlays");
  assert(display.baseVisible === true, "run manager compare: active base visible");

  const hiddenActiveDisplay = buildRunManagerDisplayState({
    overlayRuns: [{ ...activeRun, visible: false }, previousRun],
    activeOverlayRunId: "run-active",
  });
  assert(hiddenActiveDisplay.baseVisible === false, "run manager compare: hidden active hides base graph");

  const hiddenSoloDisplay = buildRunManagerDisplayState({
    overlayRuns: [activeRun, hiddenRun],
    activeOverlayRunId: "run-active",
    soloOverlayRunId: "run-hidden",
  });
  assert(hiddenSoloDisplay.overlaySeries.length === 0, "run manager compare: hidden solo row does not reappear");
  assert(hiddenSoloDisplay.baseVisible === false, "run manager compare: hidden solo suppresses base graph");

  const panel = new DotbogiPanel({
    graphCanvas: createFakeCanvas(),
    xAxisSelect: createFakeSelect(),
    yAxisSelect: createFakeSelect(),
  });
  panel.setPersistedGraph(createGraph("saved", [{ x: 0, y: 1 }, { x: 1, y: 4 }]), { render: false });
  panel.setBaseSeriesDisplay({ visible: true, alpha: 1, color: "#22d3ee", preferPersisted: true }, { render: false });
  const persistedBeforeClear = panel.buildRenderSeries();
  assert(persistedBeforeClear.series.length === 1, "run manager compare: persisted graph renders before clear");
  panel.clearTimeline({ preserveAxes: true, preserveView: true });
  const persistedAfterClear = panel.buildRenderSeries();
  assert(persistedAfterClear.series.length === 1, "run manager compare: persisted graph survives clearTimeline");
  panel.setBaseSeriesDisplay({ visible: false, alpha: 1, color: "#22d3ee" }, { render: false });
  const hiddenBaseSeries = panel.buildRenderSeries();
  assert(hiddenBaseSeries.series.length === 0, "run manager compare: base visibility hides persisted graph");

  const exportPanel = new DotbogiPanel({
    graphCanvas: createFakeCanvas(),
    xAxisSelect: createFakeSelect(),
    yAxisSelect: createFakeSelect(),
  });
  exportPanel.appendObservation({
    channels: [{ key: "t" }, { key: "theta" }],
    all_values: { t: 0, theta: 1 },
  });
  exportPanel.appendObservation({
    channels: [{ key: "t" }, { key: "theta" }],
    all_values: { t: 1, theta: 2 },
  });
  exportPanel.setSelectedAxes({ xKey: "t", yKey: "theta" });
  exportPanel.setPlaybackCursor(0, { render: false });
  const exportedGraph = exportPanel.exportCurrentGraphSnapshot();
  assert(exportedGraph?.series?.[0]?.points?.length === 2, "run manager compare: export current graph snapshot from timeline");
  const axisBeforeGrowth = exportPanel.getCurrentAxis();
  exportPanel.appendObservation({
    channels: [{ key: "t" }, { key: "theta" }],
    all_values: { t: 8, theta: 9 },
  });
  const axisAfterGrowth = exportPanel.getCurrentAxis();
  assert(
    Number(axisAfterGrowth?.x_max) > Number(axisBeforeGrowth?.x_max),
    "run manager compare: auto axis follows new timeline points",
  );
  exportPanel.setMaxPointsMode("2000");
  assert(exportPanel.getMaxPointsMode() === "2000", "run manager compare: set graph range 2000");
  assert(exportPanel.getMaxPoints() === 2000, "run manager compare: graph range 2000 max points");
  exportPanel.setMaxPointsMode("all");
  assert(exportPanel.getMaxPointsMode() === "all", "run manager compare: set graph range all");
  assert(Number.isFinite(exportPanel.getMaxPoints()) === false, "run manager compare: graph range all is unbounded");
  exportPanel.setMaxPointsMode("500");
  assert(exportPanel.getMaxPointsMode() === "500", "run manager compare: set graph range 500");
  assert(exportPanel.getMaxPoints() === 500, "run manager compare: graph range 500 max points");
  const trimPanel = new DotbogiPanel({
    graphCanvas: createFakeCanvas(),
    xAxisSelect: createFakeSelect(),
    yAxisSelect: createFakeSelect(),
  });
  trimPanel.setMaxPointsMode("500");
  for (let i = 0; i < 501; i += 1) {
    trimPanel.appendObservation({
      channels: [{ key: "t" }, { key: "theta" }],
      all_values: { t: i, theta: i },
    });
  }
  assert(trimPanel.timeline.length === 500, "run manager compare: graph range 500 trims timeline");
  trimPanel.setMaxPointsMode("all");
  for (let i = 501; i < 505; i += 1) {
    trimPanel.appendObservation({
      channels: [{ key: "t" }, { key: "theta" }],
      all_values: { t: i, theta: i },
    });
  }
  assert(trimPanel.timeline.length === 504, "run manager compare: graph range all keeps full tail after trim point");

  const dotbogiStub = {
    persistedGraph: null,
    baseDisplay: null,
    overlaySeries: null,
    setPersistedGraph(graph) {
      this.persistedGraph = graph;
    },
    setBaseSeriesDisplay(config) {
      this.baseDisplay = config;
    },
    setOverlaySeries(list) {
      this.overlaySeries = list;
    },
  };
  screen.dotbogi = dotbogiStub;
  screen.syncDockRangeLabels = () => {};
  screen.overlayRuns = [activeRun, previousRun, hiddenRun];
  screen.activeOverlayRunId = "run-active";
  screen.hoverOverlayRunId = "";
  screen.soloOverlayRunId = "";
  screen.lastGraphSnapshot = null;
  screen.syncRunManagerOverlaySeries();
  assert(dotbogiStub.persistedGraph?.series?.[0]?.id === "theta", "run manager compare: active graph pinned into dotbogi");
  assert(dotbogiStub.baseDisplay?.visible === true, "run manager compare: active row drives base visibility");
  assert(dotbogiStub.baseDisplay?.preferPersisted === true, "run manager compare: active graph prefers persisted snapshot");
  assert(Array.isArray(dotbogiStub.overlaySeries) && dotbogiStub.overlaySeries.length === 1, "run manager compare: visible previous run only");

  screen.overlayRuns[0].visible = false;
  screen.syncRunManagerOverlaySeries();
  assert(dotbogiStub.baseDisplay?.visible === false, "run manager compare: unchecked active row hides current graph");

  screen.activeOverlayRunId = "";
  screen.overlayRuns = [previousRun];
  screen.lastGraphSnapshot = createGraph("saved", [{ x: 0, y: 5 }, { x: 1, y: 6 }]);
  screen.syncRunManagerOverlaySeries();
  assert(dotbogiStub.persistedGraph?.series?.[0]?.id === "saved", "run manager compare: idle fallback keeps last graph snapshot");

  const resetScreen = new RunScreen({
    root: { dataset: {} },
    wasmState: { fpsLimit: 30, dtMax: 0.1, client: null },
  });
  let clearTimelineCalled = false;
  let persistedAfterReset = null;
  let syncCalled = false;
  resetScreen.dotbogi = {
    clearTimeline() {
      clearTimelineCalled = true;
    },
    setPersistedGraph(graph) {
      persistedAfterReset = graph;
    },
  };
  resetScreen.syncRunManagerOverlaySeries = () => {
    syncCalled = true;
  };
  resetScreen.clearRunErrorBanner = () => {};
  resetScreen.haltLoop = () => {};
  resetScreen.setEngineStatus = () => {};
  resetScreen.updateRuntimeHint = () => {};
  resetScreen.overlayRuns = [resetScreen.normalizeRunManagerRun({
    id: "run-reset",
    label: "진자 #reset",
    visible: true,
    graph: createGraph("before-reset", [{ x: 0, y: 7 }, { x: 1, y: 8 }]),
  }, 0)];
  resetScreen.activeOverlayRunId = "run-reset";
  resetScreen.lastGraphSnapshot = createGraph("persisted", [{ x: 0, y: 7 }, { x: 1, y: 8 }]);
  await resetScreen.handleResetExecutionControl();
  assert(clearTimelineCalled === true, "run manager compare: reset clears timeline while preserving graph");
  assert(persistedAfterReset?.series?.[0]?.id === "persisted", "run manager compare: reset restores persisted graph");
  assert(syncCalled === true, "run manager compare: reset resyncs compare overlays");
  assert(
    resetScreen.overlayRuns[0]?.graph?.series?.[0]?.id === "persisted",
    "run manager compare: reset preserves active run graph snapshot",
  );

  const fallbackCaptureScreen = new RunScreen({
    root: { dataset: {} },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });
  fallbackCaptureScreen.screenVisible = true;
  fallbackCaptureScreen.lesson = { id: "graphless", title: "관찰그래프" };
  fallbackCaptureScreen.sliderPanel = { getValues() { return {}; }, specs: [] };
  fallbackCaptureScreen.renderMainVisual = () => {};
  fallbackCaptureScreen.renderCurrentRuntimeTable = () => false;
  fallbackCaptureScreen.syncDockRangeLabels = () => {};
  fallbackCaptureScreen.syncDockPanelVisibility = () => {};
  fallbackCaptureScreen.syncRuntimeLayoutProfile = () => {};
  fallbackCaptureScreen.updateObserveSummary = () => {};
  fallbackCaptureScreen.updateRuntimeStatus = () => {};
  fallbackCaptureScreen.updateRuntimeHint = () => {};
  fallbackCaptureScreen.overlay = { setContent() {} };
  fallbackCaptureScreen.renderOverlayTabContent = () => {};
  fallbackCaptureScreen.renderRuntimeTextContent = () => {};
  fallbackCaptureScreen.setMainVisualMode = () => {};
  fallbackCaptureScreen.dotbogi = new DotbogiPanel({
    graphCanvas: createFakeCanvas(),
    xAxisSelect: createFakeSelect(),
    yAxisSelect: createFakeSelect(),
  });
  fallbackCaptureScreen.beginLiveRunCapture("graphless-ddn");
  fallbackCaptureScreen.applyRuntimeDerived({
    observation: {
      channels: [{ key: "t" }, { key: "theta" }],
      values: { t: 0, theta: 1 },
      all_values: { t: 0, theta: 1 },
    },
    views: { graph: null, text: null, table: null, structure: null, space2d: null },
  }, { forceView: true });
  fallbackCaptureScreen.applyRuntimeDerived({
    observation: {
      channels: [{ key: "t" }, { key: "theta" }],
      values: { t: 1, theta: 2 },
      all_values: { t: 1, theta: 2 },
    },
    views: { graph: null, text: null, table: null, structure: null, space2d: null },
  }, { forceView: true });
  assert(
    fallbackCaptureScreen.overlayRuns[0]?.graph?.series?.[0]?.points?.length === 2,
    "run manager compare: fallback observation graph captured into run history",
  );

  console.log("seamgrim run manager compare runner ok");
}

main().catch((error) => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
