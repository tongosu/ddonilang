import {
  readWasmParamDraftFromControls,
  applyWasmParamFromUi,
  applyLensPresetSelectionState,
  applyObservationRenderEffects,
  applyWasmLogicAndDispatchState,
  buildTagValueTableFromStore,
  buildObservationLensGraph,
  composeObservationRenderState,
  bindSpace2dCanvasWorldInteractions,
  buildSourcePreview,
  createManagedRafStepLoop,
  createEmptyObservationState,
  createObservationLensState,
  createEmptyStructuredViewRawSlots,
  createWasmLoader,
  deleteLensPresetFromState,
  lensLabelFromState,
  loadLensPresetState,
  markLensPresetCustomState,
  normalizeLensPresetConfig,
  normalizeWasmParamMode,
  parsePatchJsonObject,
  processPatchOperations,
  removePatchComponentStoreEntry,
  refreshLensPresetSelectElement,
  saveLensPresetToState,
  saveLensPresetState,
  renderObservationChannelList,
  resetObservationLensTimeline,
  syncWasmSettingsControlsFromState,
  stepWasmClientWithTimingAndDispatch,
  updateWasmClientLogic,
  upsertPatchComponentStoreEntry,
  upsertPatchScalarStoreEntry,
  updateLensSelectorsFromObservation,
  stripMetaHeader,
} from "./wasm_page_common.js";
import {
  extractObservationChannelsFromState,
  extractStructuredViewsFromState,
} from "./seamgrim_runtime_state.js";

const $ = (id) => document.getElementById(id);

const state = {
  runs: [],
  activeRunId: null,
  hoverRunId: null,
  soloRunId: null,
  snapshots: [],
  ddnPresets: [],
  logs: [],
  sam: null,
  geoul: null,
  viewConfig: null,
  contractView: "plain",
  workspaceMode: "basic",
  activeView: "view-graph",
  viewCombo: false,
  viewComboLayout: "horizontal",
  viewOverlayOrder: "graph",
  scene: null,
  table: null,
  textDoc: null,
  structure: null,
  space2d: null,
  space2dRange: null,
  space2dView: {
    auto: true,
    range: null,
    panX: 0,
    panY: 0,
    zoom: 1,
    dragging: false,
    lastClientX: 0,
    lastClientY: 0,
    lastRender: null,
  },
  structureLayout: null,
  tableView: {
    pageSize: 50,
    page: 0,
    precision: 3,
  },
  structureView: {
    layout: "circle",
    showLabels: true,
    nodeSize: 10,
  },
  time: {
    frames: [],
    index: 0,
    playing: false,
    timer: null,
    loop: true,
    lastKey: null,
    ddnJob: {
      running: false,
      cancelled: false,
      done: 0,
      total: 0,
    },
  },
  lessons: {
    list: [],
    groups: [],
    filter: {
      grade: "all",
      subject: "all",
      query: "",
      schema: "all",
    },
    activeId: null,
    meta: null,
    ddnText: "",
    ddnSource: "-",
    useAge3Preview: true,
    schemaStatus: {},
  },
  schemaStatus: {
    graph: "-",
    snapshot: "-",
    session: "-",
  },
  inputRegistry: {
    items: [],
    selectedId: null,
  },
  lastSession: null,
  layerCounter: 0,
  compare: {
    enabled: false,
    baselineId: null,
    variantId: null,
    axisSig: null,
    seriesId: null,
    savedVisibility: {},
    blockReason: "",
    sequence: {
      playing: false,
      timer: null,
      intervalMs: 800,
      showVariant: false,
    },
  },
  controls: {
    specs: [],
    values: {},
    metaRaw: "",
    autoRun: true,
    editorDebounceMs: 420,
  },
  mediaExport: {
    mode: "idle",
    recorder: null,
    stream: null,
    chunks: [],
    mimeType: "",
    format: "webm",
    timer: null,
    startedAt: 0,
    canvasId: "",
    frameTimer: null,
    gifFrames: [],
    gifWidth: 0,
    gifHeight: 0,
    gifDelayCs: 3,
    gifCaptureCanvas: null,
  },
  wasm: {
    enabled: false,
    active: false,
    vmClient: null,
    runIds: [],
    lastTickMs: null,
    lastFrameMs: null,
    lastChannelCount: 0,
    lastGraphSource: "-",
    lastObservation: createEmptyObservationState({ includeValues: false }),
    lens: createObservationLensState({
      enabled: false,
      xKey: "__tick__",
      yKey: "",
      y2Key: "",
      presetId: "custom",
      maxPoints: 240,
      lastFrameToken: null,
      includeRuns: true,
    }),
    langMode: "compat",
    sampleId: "line_graph_compat",
    keysPressed: 0,
    lastKeyName: "",
    pointerX: 0,
    pointerY: 0,
    fpsLimit: 30,
    dtMin: 0,
    dtMax: 0.1,
    fixedDtEnabled: false,
    fixedDtValue: 0.0333,
    patchMode: true,
    inputEnabled: true,
    keyPresetId: "wasd_arrows",
    keyMap: {
      up: ["w", "arrowup"],
      left: ["a", "arrowleft"],
      down: ["s", "arrowdown"],
      right: ["d", "arrowright"],
    },
    keyMapRaw: {
      up: "w,ArrowUp",
      left: "a,ArrowLeft",
      down: "s,ArrowDown",
      right: "d,ArrowRight",
    },
    schemaMapRaw: "",
    schemaMap: {},
    schemaPresetId: "default",
    schemaPresets: { default: "" },
    fixed64MapRaw: "",
    fixed64Map: {},
    paramKey: "",
    paramMode: "scalar",
    paramValue: "",
    fixed64Store: {},
    valueStore: {},
    componentStore: {},
    lastViewRaw: createEmptyStructuredViewRawSlots(),
    loopHandle: null,
  },
  viewUpdate: {
    "view-graph": { update: "replace", tick: null },
    "view-2d": { update: "append", tick: null },
    "view-table": { update: "replace", tick: null },
    "view-text": { update: "replace", tick: null },
    "view-structure": { update: "replace", tick: null },
  },
};

const overlayPalette = [
  { line: "#8ecae6", point: "#219ebc" },
  { line: "#ffafcc", point: "#ff5d8f" },
  { line: "#bde0fe", point: "#4361ee" },
  { line: "#c7f9cc", point: "#38b000" },
  { line: "#ffd6a5", point: "#f9844a" },
];

const VIEW_CONTRACTS = {
  "seamgrim.graph.v0": {
    title: "그래프(축형)",
    updateModes: ["append", "replace"],
    required: [
      {
        id: "series.points",
        label: "series[].points(x,y)",
        check: (graph) =>
          Array.isArray(graph?.series) &&
          graph.series.length > 0 &&
          Array.isArray(graph.series[0]?.points) &&
          graph.series[0].points.length > 0,
      },
      {
        id: "axis",
        label: "axis.x_min/x_max/y_min/y_max",
        check: (graph) =>
          Number.isFinite(graph?.axis?.x_min) &&
          Number.isFinite(graph?.axis?.x_max) &&
          Number.isFinite(graph?.axis?.y_min) &&
          Number.isFinite(graph?.axis?.y_max),
      },
      {
        id: "sample",
        label: "sample.x_min/x_max/step",
        check: (graph) =>
          Number.isFinite(graph?.sample?.x_min) &&
          Number.isFinite(graph?.sample?.x_max) &&
          Number.isFinite(graph?.sample?.step),
      },
      {
        id: "meta.update",
        label: "meta.update(append/replace)",
        check: (graph) =>
          typeof graph?.meta?.update === "string" &&
          ["append", "replace"].includes(graph.meta.update),
      },
    ],
  },
};

const SAM_SCHEMA_ALIASES = new Set([
  "sam.input.v0",
  "ddn.input_snapshot.v1",
  "seamgrim.sam.v0",
]);

const GEOUL_SCHEMA_ALIASES = new Set([
  "geoul.record.v0",
  "bogae_web_playback_v1",
  "seamgrim.geoul.v0",
]);

// --- WASM VM 연결 예시 ---
// 1) 빌드 후 산출물을 UI 경로로 복사:
//   scripts/copy_wasm_tool_to_ui.ps1
// 2) ddonirang_tool.js/wasm이 ui/wasm/ 아래에 있어야 한다.
const WASM_EXAMPLE_ENABLED = true;
const WASM_SAMPLES = [
  {
    id: "line_graph_compat",
    label: "01 line graph (compat)",
    file: "01_line_graph.ddn",
    mode: "compat",
  },
  {
    id: "line_graph_strict",
    label: "01 line graph (strict)",
    file: "01_line_graph_strict.ddn",
    mode: "strict",
  },
  {
    id: "line_graph_export",
    label: "01 line graph export",
    file: "01_line_graph_export.ddn",
    mode: "compat",
  },
  {
    id: "parabola_export",
    label: "02 parabola export",
    file: "02_parabola_export.ddn",
    mode: "compat",
  },
  {
    id: "space2d_helpers",
    label: "03 space2d helpers",
    file: "03_space2d_gaji_helpers.ddn",
    mode: "compat",
  },
  {
    id: "time_motion_export",
    label: "03 time motion export",
    file: "03_time_motion_export.ddn",
    mode: "compat",
  },
  {
    id: "calculus_ascii",
    label: "04 calculus ascii",
    file: "04_calculus_ascii.ddn",
    mode: "compat",
  },
];

function resolveWasmSample(id) {
  return WASM_SAMPLES.find((sample) => sample.id === id) ?? WASM_SAMPLES[0] ?? null;
}

const wasmLoader = createWasmLoader({
  cacheBust: Date.now(),
  setStatus: (lines) => {
    if (!Array.isArray(lines) || !lines.length) return;
    log(lines.join(" | "));
  },
  missingExportMessage: "DdnWasmVm export 누락: scripts/build_wasm_tool.ps1 --features wasm 필요",
});
let wasmLoopController = null;
let ddnEditorAutoRunTimer = null;

async function initWasmVmExample() {
  if (!WASM_EXAMPLE_ENABLED) return;
  try {
    const source = $("ddn-editor")?.value ?? "";
    const mode = state.wasm.langMode ?? "compat";
    const { client, state: stateJson } = await applyWasmLogicAndDispatchState({
      sourceText: source,
      ensureWasm: (text) => wasmLoader.ensure(text),
      mode,
      patchMode: false,
      onFullWithSource: applyWasmStateToViews,
    });
    state.wasm.enabled = true;
    state.wasm.vmClient = client;
    state.wasm.lastTickMs = null;
    state.wasm.lastFrameMs = null;
    clearWasmLensTimeline();
    initWasmPointerBindings();
    const enabledToggle = $("wasm-enabled");
    if (enabledToggle) enabledToggle.checked = true;
    startWasmLoop();
    updateWasmStatus();
    log(
      `wasm tick=${stateJson.tick_id} frame=${stateJson.frame_id} dt_ms=${stateJson.tick_time_ms.toFixed(2)} hash=${stateJson.state_hash}`,
    );
  } catch (err) {
    const source = $("ddn-editor")?.value ?? "";
    const body = stripMetaHeader(source);
    const lastBuildInfo = wasmLoader.getLastBuildInfo();
    const preview = buildSourcePreview(body, 12).join(" / ");
    const buildSuffix = lastBuildInfo ? ` | ${lastBuildInfo}` : "";
    log(`wasm init 실패: ${err?.message ?? err}${buildSuffix}`);
    if (preview) {
      log(`wasm source preview: ${preview}`);
    }
  }
}

function updateWasmLogicFromEditor() {
  if (!state.wasm?.enabled || !state.wasm.vmClient) return;
  const source = $("ddn-editor")?.value ?? "";
  try {
    const mode = state.wasm.langMode ?? "compat";
    updateWasmClientLogic({
      client: state.wasm.vmClient,
      sourceBody: stripMetaHeader(source),
      mode,
    });
  } catch (err) {
    log(`wasm logic 업데이트 실패: ${err?.message ?? err}`);
  }
}

function normalizeLangMode(raw) {
  const value = String(raw ?? "").trim().toLowerCase();
  if (value === "strict") return "strict";
  return "compat";
}

function setWasmLangMode(mode, options = {}) {
  const next = normalizeLangMode(mode);
  state.wasm.langMode = next;
  const select = $("wasm-lang-mode");
  if (select) select.value = next;
  if (options.updateLogic !== false) {
    updateWasmLogicFromEditor();
  }
  updateWasmStatus();
  if (options.save !== false) {
    saveWasmSettings();
  }
}

function initWasmSampleSelect(select) {
  if (!select) return;
  select.innerHTML = "";
  WASM_SAMPLES.forEach((sample) => {
    const opt = document.createElement("option");
    opt.value = sample.id;
    opt.textContent = sample.label;
    select.appendChild(opt);
  });
  const resolved = resolveWasmSample(state.wasm.sampleId);
  if (resolved) {
    select.value = resolved.id;
    state.wasm.sampleId = resolved.id;
  }
}

async function loadWasmSample(sampleId, options = {}) {
  const sample = resolveWasmSample(sampleId);
  if (!sample) {
    log("샘플을 찾을 수 없습니다.");
    return;
  }
  try {
    const response = await fetch(`/samples/${sample.file}`);
    if (!response.ok) {
      log(`샘플 로드 실패: ${response.status} ${response.statusText}`);
      return;
    }
    const text = await response.text();
    const editor = $("ddn-editor");
    if (editor) editor.value = text;
    state.wasm.sampleId = sample.id;
    const sampleSelect = $("wasm-sample-select");
    if (sampleSelect) sampleSelect.value = sample.id;
    if (options.applyMode !== false && sample.mode) {
      setWasmLangMode(sample.mode, { updateLogic: false, save: false });
    }
    if (options.updateLogic !== false) {
      clearWasmLensTimeline();
      updateWasmLogicFromEditor();
    }
    saveWasmSettings();
  } catch (err) {
    log(`샘플 로드 실패: ${err?.message ?? err}`);
  }
}

function extractViewsFromStateJson(stateJson) {
  return extractStructuredViewsFromState(stateJson, {
    preferPatch: Boolean(state.wasm?.patchMode),
  });
}

function componentKey(entity, tag) {
  return `${entity}:${tag}`;
}

function clearWasmGraph() {
  if (state.wasm.runIds.length) {
    state.wasm.runIds.forEach((id) => removeRun(id));
    state.wasm.runIds = [];
  }
  state.wasm.lastViewRaw.graph = null;
}

function clearViewBySchema(schema, raw) {
  if (!schema) return false;
  const target = resolveSchemaTarget(schema, null);
  if (target === "graph") {
    if (raw && raw !== state.wasm.lastViewRaw.graph) return false;
    clearWasmGraph();
    return true;
  }
  if (target === "space2d") {
    if (raw && raw !== state.wasm.lastViewRaw.space2d) return false;
    state.space2d = null;
    state.space2dRange = null;
    state.wasm.lastViewRaw.space2d = null;
    return true;
  }
  if (target === "text") {
    if (raw && raw !== state.wasm.lastViewRaw.text) return false;
    state.textDoc = null;
    state.wasm.lastViewRaw.text = null;
    return true;
  }
  if (target === "table") {
    if (raw && raw !== state.wasm.lastViewRaw.table) return false;
    state.table = null;
    state.wasm.lastViewRaw.table = null;
    return true;
  }
  if (target === "structure") {
    if (raw && raw !== state.wasm.lastViewRaw.structure) return false;
    state.structure = null;
    state.structureLayout = null;
    state.wasm.lastViewRaw.structure = null;
    return true;
  }
  return false;
}

function applyJsonViewUpdate(raw, obj, sourceText) {
  const schema = obj?.schema ?? "";
  const target = resolveSchemaTarget(schema, obj);
  if (!raw || !obj) return false;

  if (target === "graph") {
    if (raw === state.wasm.lastViewRaw.graph) return false;
    let validated = null;
    try {
      validated = validateGraphData(obj);
    } catch (err) {
      log(`graph 스키마 오류: ${err.message}`);
    }
    if (!validated) return false;
    const runs = createRunsFromGraph(
      validated,
      { kind: "wasm", text: sourceText },
      { sample: validated.sample, time: readTimeControls() },
      { space2d: state.space2d ?? null, textDoc: state.textDoc ?? null },
    );
    if (!state.viewConfig) {
      state.viewConfig = getViewConfigFromData(validated, state.runs);
      applyViewControlsFromGraph(validated, state.viewConfig);
    }
    if (state.viewConfig) applyViewToGraph(validated, state.viewConfig);
    upsertWasmRuns(runs);
    state.wasm.lastViewRaw.graph = raw;
    return true;
  }

  if (target === "space2d") {
    if (raw === state.wasm.lastViewRaw.space2d) return false;
    try {
      state.space2d = validateSpace2dData(obj);
      state.space2dRange = computeSpace2dRange(state.space2d);
      state.wasm.lastViewRaw.space2d = raw;
      return true;
    } catch (err) {
      log(`2D 스키마 오류: ${err.message}`);
      return false;
    }
  }

  if (target === "text") {
    if (raw === state.wasm.lastViewRaw.text) return false;
    state.textDoc = normalizeTextDoc(obj);
    state.wasm.lastViewRaw.text = raw;
    return true;
  }

  if (target === "table") {
    if (raw === state.wasm.lastViewRaw.table) return false;
    const normalized = normalizeTableData(obj);
    if (normalized) {
      state.table = normalized;
      state.tableView.page = 0;
      state.wasm.lastViewRaw.table = raw;
      return true;
    }
    return false;
  }

  if (target === "structure") {
    if (raw === state.wasm.lastViewRaw.structure) return false;
    try {
      state.structure = { ...validateStructureData(obj), selection: null };
      state.structureLayout = null;
      state.wasm.lastViewRaw.structure = raw;
      return true;
    } catch (err) {
      log(`structure 스키마 오류: ${err.message}`);
      return false;
    }
  }

  return false;
}

function coerceFixed64Number(value) {
  if (typeof value === "number") {
    return Number.isFinite(value) ? value : null;
  }
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (!trimmed) return null;
    const parsed = Number(trimmed);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

function coerceFixed64Bool(value) {
  if (typeof value === "boolean") return value;
  if (typeof value === "number") return value !== 0;
  const raw = String(value ?? "").trim().toLowerCase();
  if (["true", "1", "yes", "y"].includes(raw)) return true;
  if (["false", "0", "no", "n"].includes(raw)) return false;
  return null;
}

function applyFixed64GraphAxis(field, value) {
  if (!["x_min", "x_max", "y_min", "y_max"].includes(field)) return false;
  let changed = false;
  state.runs.forEach((run) => {
    if (!run?.graph) return;
    run.graph.axis = run.graph.axis ?? {};
    if (run.graph.axis[field] !== value) {
      run.graph.axis[field] = value;
      changed = true;
    }
  });
  return changed;
}

function applyFixed64GraphSample(field, value) {
  if (!["x_min", "x_max", "step"].includes(field)) return false;
  let changed = false;
  state.runs.forEach((run) => {
    if (!run?.graph) return;
    run.graph.sample = run.graph.sample ?? {};
    if (run.graph.sample[field] !== value) {
      run.graph.sample[field] = value;
      changed = true;
    }
  });
  return changed;
}

function applyFixed64GraphView(field, value) {
  const viewAuto = $("view-auto");
  const viewInputs = {
    x_min: "view-x-min",
    x_max: "view-x-max",
    y_min: "view-y-min",
    y_max: "view-y-max",
    pan_x: "pan-x",
    pan_y: "pan-y",
    zoom: "zoom",
  };
  if (field === "auto") {
    const next = coerceFixed64Bool(value);
    if (next === null) return false;
    if (viewAuto) viewAuto.checked = next;
    updateViewInputsEnabled();
    state.runs.forEach((run) => {
      if (!run?.graph) return;
      run.graph.view = run.graph.view ?? {};
      run.graph.view.auto = next;
    });
    return true;
  }
  if (!(field in viewInputs)) return false;
  const input = $(viewInputs[field]);
  if (input) input.value = String(value);
  if (viewAuto) viewAuto.checked = false;
  updateViewInputsEnabled();
  state.runs.forEach((run) => {
    if (!run?.graph) return;
    run.graph.view = run.graph.view ?? {};
    run.graph.view[field] = value;
  });
  return true;
}

function applyFixed64Space2dView(field, value) {
  const autoInput = $("space2d-auto");
  const viewInputs = {
    x_min: "space2d-x-min",
    x_max: "space2d-x-max",
    y_min: "space2d-y-min",
    y_max: "space2d-y-max",
    pan_x: "space2d-pan-x",
    pan_y: "space2d-pan-y",
    zoom: "space2d-zoom",
  };
  if (field === "auto") {
    const next = coerceFixed64Bool(value);
    if (next === null) return false;
    state.space2dView.auto = next;
    if (autoInput) autoInput.checked = next;
    if (next) {
      state.space2dView.range = null;
    }
    updateSpace2dInputsEnabled();
    return true;
  }
  if (!(field in viewInputs)) return false;
  if (["x_min", "x_max", "y_min", "y_max"].includes(field)) {
    state.space2dView.auto = false;
    if (autoInput) autoInput.checked = false;
    const baseRange =
      state.space2dView.range ??
      state.space2dRange ??
      computeSpace2dRange(state.space2d) ?? {
        xMin: 0,
        xMax: 0,
        yMin: 0,
        yMax: 0,
      };
    state.space2dView.range = { ...baseRange };
    const rangeKey =
      field === "x_min"
        ? "xMin"
        : field === "x_max"
          ? "xMax"
          : field === "y_min"
            ? "yMin"
            : "yMax";
    state.space2dView.range[rangeKey] = value;
    updateSpace2dInputsEnabled();
  } else if (field === "pan_x") {
    state.space2dView.panX = value;
  } else if (field === "pan_y") {
    state.space2dView.panY = value;
  } else if (field === "zoom") {
    state.space2dView.zoom = value;
  }
  const input = $(viewInputs[field]);
  if (input) input.value = String(value);
  return true;
}

function applyFixed64Mappings(tag, value) {
  const targets = state.wasm?.fixed64Map?.[tag];
  if (!targets || !targets.length) return false;
  let changed = false;
  targets.forEach((target) => {
    const parts = target.split(".").filter(Boolean);
    if (!parts.length) return;
    if (parts[0] === "graph") {
      const section = parts[1];
      const field = parts.slice(2).join("_");
      if (!section || !field) return;
      if (section === "axis") {
        changed = applyFixed64GraphAxis(field, value) || changed;
      } else if (section === "sample") {
        changed = applyFixed64GraphSample(field, value) || changed;
      } else if (section === "view") {
        changed = applyFixed64GraphView(field, value) || changed;
      }
      return;
    }
    if (parts[0] === "space2d") {
      const section = parts[1];
      const field = parts.slice(2).join("_");
      if (section !== "view" || !field) return;
      changed = applyFixed64Space2dView(field, value) || changed;
    }
  });
  return changed;
}

function upsertWasmRuns(runs) {
  if (!runs.length) return;
  if (!state.wasm.runIds.length || state.wasm.runIds.length !== runs.length) {
    runs.forEach((run, idx) => addRun(run, { activate: idx === 0 }));
    state.wasm.runIds = runs.map((run) => run.id);
    return;
  }
  runs.forEach((incoming, idx) => {
    const existingId = state.wasm.runIds[idx];
    const existing = state.runs.find((run) => run.id === existingId);
    if (existing) {
      replaceRunPreserve(existing, incoming);
    } else {
      addRun(incoming, { activate: idx === 0 });
      state.wasm.runIds[idx] = incoming.id;
    }
  });
}

function applyWasmStateToViews(stateJson, sourceText) {
  if (!stateJson) return;
  const observationResult = updateWasmObservationFromState(stateJson);
  const {
    graph,
    graphSource,
    graphRaw,
    space2d,
    space2dRaw,
    table,
    tableRaw,
    text,
    textRaw,
    structure,
    structureRaw,
  } = extractViewsFromStateJson(stateJson);
  state.wasm.lastGraphSource = graphSource ?? "-";
  let changed = false;
  if (graph && graphRaw) changed = applyJsonViewUpdate(graphRaw, graph, sourceText) || changed;
  if (space2d && space2dRaw) changed = applyJsonViewUpdate(space2dRaw, space2d, sourceText) || changed;
  if (text && textRaw) changed = applyJsonViewUpdate(textRaw, text, sourceText) || changed;
  if (table && tableRaw) changed = applyJsonViewUpdate(tableRaw, table, sourceText) || changed;
  if (structure && structureRaw)
    changed = applyJsonViewUpdate(structureRaw, structure, sourceText) || changed;
  applyObservationRenderEffects({
    changed,
    forceRender: observationResult.forceRender,
    channelsChanged: observationResult.channelsChanged,
    onRender: () => {
      renderAll();
      renderTextView();
      renderTableView();
      renderStructureView();
      renderInspector();
    },
    onChannelsChanged: () => {
      updateWasmStatus();
      renderDockSummary();
    },
  });
}

function processPatchJsonOp(op, sourceText) {
  const parsed = parsePatchJsonObject(op?.value);
  if (!parsed) return false;
  const { raw, obj } = parsed;
  upsertPatchComponentStoreEntry({
    componentStore: state.wasm.componentStore,
    op,
    raw,
    obj,
    keyBuilder: componentKey,
  });
  return applyJsonViewUpdate(raw, obj, sourceText);
}

function processPatchFixed64Op(op) {
  const updated = upsertPatchScalarStoreEntry({
    store: state.wasm.fixed64Store || (state.wasm.fixed64Store = {}),
    tag: op?.tag,
    value: op?.value,
    asString: true,
  });
  if (!updated.updated) {
    return { changed: false, fixed64Changed: false, requireFull: false };
  }
  let changed = false;
  const numeric = coerceFixed64Number(op.value);
  if (numeric !== null) {
    changed = applyFixed64Mappings(op.tag, numeric) || changed;
  }
  const requireFull = op.tag === "보개_그림판_가로" || op.tag === "보개_그림판_세로";
  return { changed, fixed64Changed: true, requireFull };
}

function processPatchValueOp(op) {
  const updated = upsertPatchScalarStoreEntry({
    store: state.wasm.valueStore || (state.wasm.valueStore = {}),
    tag: op?.tag,
    value: op?.value,
    asString: false,
  });
  if (!updated.updated) {
    return { valueChanged: false, requireFull: false };
  }
  return {
    valueChanged: true,
    requireFull: op.tag === "보개_그림판_목록",
  };
}

function processPatchRemoveComponentOp(op) {
  const removed = removePatchComponentStoreEntry({
    componentStore: state.wasm.componentStore,
    op,
    keyBuilder: componentKey,
  });
  if (removed.removed && removed.entry) {
    return clearViewBySchema(removed.entry.schema, removed.entry.raw);
  }
  return false;
}

function applyWasmPatchToViews(stateJson, sourceText) {
  if (!stateJson || !Array.isArray(stateJson.patch) || !stateJson.patch.length) return;
  const observationResult = updateWasmObservationFromState(stateJson);
  const patchResult = processPatchOperations({
    patch: stateJson.patch,
    onJsonOp: (op) => processPatchJsonOp(op, sourceText),
    onFixed64Op: (op) => processPatchFixed64Op(op),
    onValueOp: (op) => processPatchValueOp(op),
    onRemoveComponentOp: (op) => processPatchRemoveComponentOp(op),
  });
  let changed = Boolean(patchResult.changed);
  const fixed64Changed = Boolean(patchResult.fixed64Changed);
  const valueChanged = Boolean(patchResult.valueChanged);
  const requireFull = Boolean(patchResult.requireFull);
  if (requireFull) {
    applyWasmStateToViews(stateJson, sourceText);
    return;
  }
  if (fixed64Changed) {
    const shouldUpdate =
      !state.table || (state.table?.meta && state.table.meta.source === "fixed64");
    if (shouldUpdate) {
      state.table = buildTagValueTableFromStore({
        store: state.wasm.fixed64Store || {},
        source: "fixed64",
      });
      state.tableView.page = 0;
      changed = true;
    }
  }
  if (valueChanged) {
    const shouldUpdate = !state.table || (state.table?.meta && state.table.meta.source === "value");
    if (shouldUpdate) {
      state.table = buildTagValueTableFromStore({
        store: state.wasm.valueStore || {},
        source: "value",
      });
      state.tableView.page = 0;
      changed = true;
    }
  }
  applyObservationRenderEffects({
    changed,
    forceRender: observationResult.forceRender,
    channelsChanged: observationResult.channelsChanged,
    onRender: () => {
      renderAll();
      renderTextView();
      renderTableView();
      renderStructureView();
      renderInspector();
    },
    onChannelsChanged: () => {
      updateWasmStatus();
      renderDockSummary();
    },
  });
}

function startWasmLoop() {
  if (!state.wasm.enabled || !state.wasm.vmClient) return;
  if (!wasmLoopController) {
    wasmLoopController = createManagedRafStepLoop({
      getFps: () => Math.max(1, Number(state.wasm.fpsLimit) || 30),
      isActive: () => Boolean(state.wasm.active),
      setActive: (next) => {
        state.wasm.active = Boolean(next);
      },
      onStart: () => {
        state.wasm.loopHandle = 1;
      },
      onStop: () => {
        state.wasm.loopHandle = null;
      },
      onStep: (now) => {
        if (!state.wasm.active || !state.wasm.vmClient) {
          wasmLoopController.stop();
          return;
        }
        state.wasm.lastFrameMs = now;

        const stepped = stepWasmClientWithTimingAndDispatch({
          client: state.wasm.vmClient,
          nowMs: now,
          lastTickMs: state.wasm.lastTickMs,
          fixedDtEnabled: state.wasm.fixedDtEnabled,
          fixedDtValue: state.wasm.fixedDtValue,
          dtMin: state.wasm.dtMin,
          dtMax: state.wasm.dtMax,
          inputEnabled: state.wasm.inputEnabled,
          keys: state.wasm.keysPressed,
          lastKey: state.wasm.lastKeyName,
          px: state.wasm.pointerX,
          py: state.wasm.pointerY,
          clearLastKeyWhenFixedDt: true,
          sourceText: $("ddn-editor")?.value ?? "",
          patchMode: state.wasm.patchMode,
          onPatchWithSource: applyWasmPatchToViews,
          onFullWithSource: applyWasmStateToViews,
          errorPrefix: "startWasmLoop",
        });
        state.wasm.lastTickMs = stepped.nextTickMs;
        state.wasm.lastKeyName = stepped.nextLastKey;
      },
      onError: (err) => {
        log(`wasm step 실패: ${err?.message ?? err}`);
      },
    });
  }
  wasmLoopController.start();
}

function stopWasmLoop() {
  wasmLoopController?.stop();
}

function syncWasmLensConfigFromDom() {
  const lens = state.wasm.lens;
  lens.enabled = $("wasm-lens-enable")?.checked ?? false;
  lens.xKey = String($("wasm-lens-x-key")?.value ?? "__tick__");
  lens.yKey = String($("wasm-lens-y-key")?.value ?? "");
  lens.y2Key = String($("wasm-lens-y2-key")?.value ?? "");
}

function normalizeWasmLensPreset(raw) {
  return normalizeLensPresetConfig(raw);
}

function currentWasmLensPreset() {
  const lens = state.wasm.lens;
  return {
    enabled: Boolean(lens.enabled),
    xKey: String(lens.xKey ?? "__tick__"),
    yKey: String(lens.yKey ?? ""),
    y2Key: String(lens.y2Key ?? ""),
  };
}

function markWasmLensPresetCustom() {
  markLensPresetCustomState({
    lensState: state.wasm.lens,
    selectEl: $("wasm-lens-preset"),
    nameInputEl: $("wasm-lens-preset-name"),
  });
}

function applyWasmLensStateToDom() {
  const lens = state.wasm.lens;
  if ($("wasm-lens-enable")) $("wasm-lens-enable").checked = lens.enabled;
  updateWasmLensSelectors(state.wasm.lastObservation);
  syncWasmLensConfigFromDom();
}

function applyWasmLensSelectionToViews(options = {}) {
  const markCustom = options.markCustom !== false;
  if (markCustom) {
    markWasmLensPresetCustom();
  }
  state.wasm.lens.runs = buildWasmLensRuns();
  renderWasmChannelSummary(state.wasm.lastObservation);
  renderWasmChannelList(state.wasm.lastObservation);
  renderAll();
  renderDockSummary();
  updateWasmStatus();
}

function renderWasmChannelSummary(observation) {
  const box = $("wasm-channel-summary");
  if (!box) return;
  const channels = Array.isArray(observation?.channels) ? observation.channels : [];
  const lensLabel = lensLabelFromState(state.wasm.lens);
  box.textContent = `관찰 채널: ${channels.length} · 렌즈: ${lensLabel}`;
}

function renderWasmChannelList(observation) {
  renderObservationChannelList({
    element: $("wasm-channel-list"),
    observation,
    maxRows: 80,
    numericMode: "compact",
    target: "value",
  });
}

function updateWasmLensSelectors(observation) {
  const lens = state.wasm.lens;
  const xSelect = $("wasm-lens-x-key");
  const ySelect = $("wasm-lens-y-key");
  const y2Select = $("wasm-lens-y2-key");
  updateLensSelectorsFromObservation({
    observation,
    lensState: lens,
    xSelect,
    ySelect,
    y2Select,
    onSynced: syncWasmLensConfigFromDom,
  });
}

function buildWasmLensRuns(graph = null) {
  const nextGraph =
    graph ??
    buildObservationLensGraph({
      lensState: state.wasm.lens,
      source: "observation-lens",
      includeSample: true,
      metaUpdate: "replace",
    });
  if (!nextGraph) return [];
  return createRunsFromGraph(
    nextGraph,
    { kind: "wasm_lens", text: "observation-lens" },
    null,
  );
}

function clearWasmLensTimeline() {
  const lens = state.wasm.lens;
  resetObservationLensTimeline(lens, { lastFrameToken: null, clearRuns: true });
}

function updateWasmObservationFromState(stateJson) {
  const observation = extractObservationChannelsFromState(stateJson);
  const lens = state.wasm.lens;
  const prevChannelCount = Number(state.wasm.lastChannelCount ?? 0);
  const nextChannelCount = observation.channels.length;
  state.wasm.lastChannelCount = nextChannelCount;
  state.wasm.lastObservation = observation;

  renderWasmChannelSummary(observation);
  renderWasmChannelList(observation);
  updateWasmLensSelectors(observation);

  const prevLensActive = lens.enabled && Array.isArray(lens.runs) && lens.runs.length > 0;
  const renderStateInfo = composeObservationRenderState({
    stateJson,
    observation,
    lensState: lens,
    graphOptions: {
      source: "observation-lens",
      includeSample: true,
      metaUpdate: "replace",
    },
  });
  lens.runs = buildWasmLensRuns(renderStateInfo.lensGraph);
  const nextLensActive = lens.enabled && lens.runs.length > 0;
  return {
    channelsChanged: prevChannelCount !== nextChannelCount,
    forceRender:
      (nextLensActive && renderStateInfo.samplePushed) ||
      prevLensActive !== nextLensActive,
  };
}

function getWasmLensRenderRuns() {
  const lens = state.wasm.lens;
  if (!lens.enabled) return [];
  return Array.isArray(lens.runs) ? lens.runs : [];
}

function getGraphRunsForRender(baseRuns) {
  const lensRuns = getWasmLensRenderRuns();
  if (lensRuns.length) return lensRuns;
  return Array.isArray(baseRuns) ? baseRuns : [];
}

function updateWasmStatus() {
  const status = $("wasm-status");
  if (!status) return;
  if (!state.wasm.enabled) {
    status.textContent = "wasm: 비활성";
    return;
  }
  const running = state.wasm.active ? "running" : "stopped";
  const dtInfo = state.wasm.fixedDtEnabled
    ? `dt=${state.wasm.fixedDtValue}s`
    : `dt≤${state.wasm.dtMax}s`;
  const patchInfo = state.wasm.patchMode ? "patch" : "full";
  const modeInfo = state.wasm.langMode ?? "compat";
  const channels = Math.max(0, Number(state.wasm.lastChannelCount) || 0);
  const lensInfo = lensLabelFromState(state.wasm.lens, { joiner: "+" });
  const graphSource = String(state.wasm.lastGraphSource ?? "-");
  status.textContent =
    `wasm: ${running} · mode=${modeInfo} · fps=${state.wasm.fpsLimit} · ${dtInfo} · ${patchInfo} · channels=${channels} · lens=${lensInfo} · graph=${graphSource}`;
}

function bindWasmControls() {
  const enabled = $("wasm-enabled");
  const langModeSelect = $("wasm-lang-mode");
  const inputEnabled = $("wasm-input-enabled");
  const sampleSelect = $("wasm-sample-select");
  const sampleLoad = $("wasm-sample-load");
  const fpsInput = $("wasm-fps-limit");
  const dtMaxInput = $("wasm-dt-max");
  const fixedEnabled = $("wasm-fixed-dt-enabled");
  const fixedValue = $("wasm-fixed-dt");
  const patchMode = $("wasm-patch-mode");
  const presetSelect = $("wasm-key-preset");
  const keyUp = $("wasm-key-up");
  const keyLeft = $("wasm-key-left");
  const keyDown = $("wasm-key-down");
  const keyRight = $("wasm-key-right");
  const schemaPresetSelect = $("wasm-schema-preset");
  const schemaPresetName = $("wasm-schema-preset-name");
  const schemaPresetSave = $("wasm-schema-save");
  const schemaPresetDelete = $("wasm-schema-delete");
  const schemaMapInput = $("wasm-schema-map");
  const fixed64MapInput = $("wasm-fixed64-map");
  const wasmParamKeyInput = $("wasm-param-key");
  const wasmParamModeSelect = $("wasm-param-mode");
  const wasmParamInput = $("wasm-param-input");
  const wasmParamApplyBtn = $("wasm-param-apply");
  const wasmParamStatus = $("wasm-param-status");
  const lensEnable = $("wasm-lens-enable");
  const lensXSelect = $("wasm-lens-x-key");
  const lensYSelect = $("wasm-lens-y-key");
  const lensY2Select = $("wasm-lens-y2-key");
  const lensPresetSelect = $("wasm-lens-preset");
  const lensPresetName = $("wasm-lens-preset-name");
  const lensPresetSave = $("wasm-lens-preset-save");
  const lensPresetDelete = $("wasm-lens-preset-delete");
  const saveBtn = $("wasm-settings-save");
  const loadBtn = $("wasm-settings-load");
  const reloadBtn = $("wasm-reload");
  const stopBtn = $("wasm-stop");

  const setWasmParamStatus = (message) => {
    if (!wasmParamStatus) return;
    wasmParamStatus.textContent = String(message ?? "param: -");
  };

  const applyWasmParamByControls = async () => {
    if (!state.wasm.vmClient) {
      await initWasmVmExample();
    }
    const client = state.wasm.vmClient;
    if (!client) {
      throw new Error("WASM client가 준비되지 않았습니다.");
    }
    const applied = applyWasmParamFromUi({
      client,
      key: wasmParamKeyInput?.value ?? "",
      rawValue: wasmParamInput?.value ?? "",
      mode: wasmParamModeSelect?.value ?? "scalar",
      errorPrefix: "app wasm param",
    });
    if (!applied.ok) {
      setWasmParamStatus(`param: 실패 (${applied.error ?? "unknown"})`);
      throw new Error(applied.error ?? "param apply failed");
    }
    const stateJson = client.getStateParsed();
    applyWasmStateToViews(stateJson, $("ddn-editor")?.value ?? "");
    updateWasmStatus();
    const hash = applied.result?.state_hash ?? stateJson?.state_hash ?? "-";
    const paramDraft = readWasmParamDraftFromControls({
      keyInput: wasmParamKeyInput,
      modeSelect: wasmParamModeSelect,
      valueInput: wasmParamInput,
    });
    state.wasm.paramKey = paramDraft.key;
    state.wasm.paramMode = paramDraft.mode;
    state.wasm.paramValue = paramDraft.value;
    saveWasmSettings();
    setWasmParamStatus(
      `param: ok key=${applied.key} mode=${applied.mode} kind=${applied.valueKind} hash=${hash}`,
    );
    log(
      `WASM param 적용: key=${applied.key}, mode=${applied.mode}, kind=${applied.valueKind}, hash=${hash}`,
    );
  };

  function reconcileWasmSettingsStateFromStores() {
    if (state.wasm.schemaPresets?.[state.wasm.schemaPresetId] !== undefined) {
      const presetRaw = state.wasm.schemaPresets[state.wasm.schemaPresetId];
      if (state.wasm.schemaMapRaw && state.wasm.schemaMapRaw !== presetRaw) {
        state.wasm.schemaPresetId = "custom";
      } else {
        state.wasm.schemaMapRaw = presetRaw;
      }
    } else {
      state.wasm.schemaPresetId = "custom";
    }
    if (!WASM_KEY_PRESETS[state.wasm.keyPresetId]) {
      state.wasm.keyPresetId = "custom";
    }
  }

  loadWasmSettings();
  initWasmSampleSelect(sampleSelect);
  loadSchemaPresets();
  reconcileWasmSettingsStateFromStores();
  state.wasm.schemaMap = parseSchemaMap(state.wasm.schemaMapRaw);
  state.wasm.fixed64Map = parseFixed64Map(state.wasm.fixed64MapRaw);
  loadWasmLensPresets();

  const refreshLensPresetSelect = () => {
    state.wasm.lens.presetId = refreshLensPresetSelectElement({
      selectEl: lensPresetSelect,
      presets: state.wasm.lens.presets,
      presetId: state.wasm.lens.presetId,
    });
  };

  const applyLensPresetSelection = (id) => {
    const result = applyLensPresetSelectionState({
      lensState: state.wasm.lens,
      id,
      normalizePreset: normalizeWasmLensPreset,
    });
    if (!result.ok) return;
    if (result.mode === "custom") {
      if (lensPresetName) lensPresetName.value = "";
      refreshLensPresetSelect();
      saveWasmLensPresets();
      saveWasmSettings();
      return;
    }
    applyWasmLensStateToDom();
    applyWasmLensSelectionToViews({ markCustom: false });
    if (lensPresetName) lensPresetName.value = result.presetId;
    refreshLensPresetSelect();
    saveWasmLensPresets();
    saveWasmSettings();
  };

  const refreshLensFromControls = (options = {}) => {
    syncWasmLensConfigFromDom();
    applyWasmLensSelectionToViews({ markCustom: options.markCustom !== false });
    saveWasmLensPresets();
    saveWasmSettings();
  };

  refreshLensPresetSelect();

  if (langModeSelect) {
    langModeSelect.value = normalizeLangMode(state.wasm.langMode);
    langModeSelect.addEventListener("change", () => {
      setWasmLangMode(langModeSelect.value);
    });
  }

  if (sampleSelect) {
    const sample = resolveWasmSample(state.wasm.sampleId);
    if (sample) {
      sampleSelect.value = sample.id;
      state.wasm.sampleId = sample.id;
    }
    sampleSelect.addEventListener("change", () => {
      state.wasm.sampleId = sampleSelect.value;
      saveWasmSettings();
    });
  }

  if (sampleLoad) {
    sampleLoad.addEventListener("click", () => {
      const targetId = sampleSelect?.value ?? state.wasm.sampleId;
      loadWasmSample(targetId, { applyMode: true, updateLogic: true });
    });
  }

  if (enabled) {
    enabled.addEventListener("change", () => {
      state.wasm.enabled = enabled.checked;
      if (state.wasm.enabled) {
        startWasmLoop();
      } else {
        stopWasmLoop();
      }
      updateWasmStatus();
      saveWasmSettings();
    });
  }

  if (inputEnabled) {
    inputEnabled.addEventListener("change", () => {
      state.wasm.inputEnabled = inputEnabled.checked;
      saveWasmSettings();
    });
  }

  if (fpsInput) {
    fpsInput.addEventListener("input", () => {
      const next = Number(fpsInput.value);
      if (Number.isFinite(next) && next > 0) {
        state.wasm.fpsLimit = next;
        saveWasmSettings();
      }
    });
  }

  if (dtMaxInput) {
    dtMaxInput.addEventListener("input", () => {
      const next = Number(dtMaxInput.value);
      if (Number.isFinite(next) && next >= 0) {
        state.wasm.dtMax = next;
        saveWasmSettings();
      }
    });
  }

  if (fixedEnabled) {
    fixedEnabled.addEventListener("change", () => {
      state.wasm.fixedDtEnabled = fixedEnabled.checked;
      updateWasmStatus();
      saveWasmSettings();
    });
  }

  if (fixedValue) {
    fixedValue.addEventListener("input", () => {
      const next = Number(fixedValue.value);
      if (Number.isFinite(next) && next >= 0) {
        state.wasm.fixedDtValue = next;
        saveWasmSettings();
      }
    });
  }

  if (patchMode) {
    patchMode.addEventListener("change", () => {
      state.wasm.patchMode = patchMode.checked;
      updateWasmStatus();
      saveWasmSettings();
    });
  }

  if (presetSelect) {
    presetSelect.innerHTML = "";
    Object.entries(WASM_KEY_PRESETS).forEach(([id, preset]) => {
      const opt = document.createElement("option");
      opt.value = id;
      opt.textContent = preset.label;
      presetSelect.appendChild(opt);
    });
    presetSelect.addEventListener("change", () => {
      const id = presetSelect.value;
      state.wasm.keyPresetId = id;
      const preset = WASM_KEY_PRESETS[id];
      if (preset?.map) {
        state.wasm.keyMapRaw = { ...preset.map };
        syncWasmKeyMapFromRaw();
        if (keyUp) keyUp.value = state.wasm.keyMapRaw.up;
        if (keyLeft) keyLeft.value = state.wasm.keyMapRaw.left;
        if (keyDown) keyDown.value = state.wasm.keyMapRaw.down;
        if (keyRight) keyRight.value = state.wasm.keyMapRaw.right;
      }
      saveWasmSettings();
    });
  }

  const syncKeyMap = () => {
    state.wasm.keyMapRaw = {
      up: keyUp?.value ?? state.wasm.keyMapRaw.up,
      left: keyLeft?.value ?? state.wasm.keyMapRaw.left,
      down: keyDown?.value ?? state.wasm.keyMapRaw.down,
      right: keyRight?.value ?? state.wasm.keyMapRaw.right,
    };
    syncWasmKeyMapFromRaw();
    if (presetSelect) {
      presetSelect.value = "custom";
      state.wasm.keyPresetId = "custom";
    }
    saveWasmSettings();
  };
  if (keyUp) keyUp.addEventListener("input", syncKeyMap);
  if (keyLeft) keyLeft.addEventListener("input", syncKeyMap);
  if (keyDown) keyDown.addEventListener("input", syncKeyMap);
  if (keyRight) keyRight.addEventListener("input", syncKeyMap);

  const refreshSchemaPresetSelect = () => {
    if (!schemaPresetSelect) return;
    if (!state.wasm.schemaPresets?.default) {
      state.wasm.schemaPresets = { default: "", ...(state.wasm.schemaPresets ?? {}) };
    }
    const presetIds = Object.keys(state.wasm.schemaPresets ?? {}).filter(
      (id) => id !== "custom",
    );
    presetIds.sort();
    schemaPresetSelect.innerHTML = "";
    const addOption = (id, label) => {
      const opt = document.createElement("option");
      opt.value = id;
      opt.textContent = label;
      schemaPresetSelect.appendChild(opt);
    };
    addOption("custom", "Custom(미저장)");
    presetIds.forEach((id) => {
      addOption(id, id);
    });
    if (!presetIds.includes(state.wasm.schemaPresetId) && state.wasm.schemaPresetId !== "custom") {
      state.wasm.schemaPresetId = "custom";
    }
    schemaPresetSelect.value = state.wasm.schemaPresetId;
  };

  const syncWasmSettingsUiFromState = ({ applyLensSelection = true } = {}) => {
    refreshSchemaPresetSelect();
    refreshLensPresetSelect();
    syncWasmSettingsControlsFromState({
      wasmState: state.wasm,
      enabledToggle: enabled,
      langModeSelect,
      sampleSelect,
      inputEnabledToggle: inputEnabled,
      fpsInput,
      dtMaxInput,
      fixedDtEnabledToggle: fixedEnabled,
      fixedDtValueInput: fixedValue,
      patchModeToggle: patchMode,
      keyPresetSelect: presetSelect,
      keyUpInput: keyUp,
      keyLeftInput: keyLeft,
      keyDownInput: keyDown,
      keyRightInput: keyRight,
      schemaMapInput,
      fixed64MapInput,
      wasmParamKeyInput,
      wasmParamModeSelect,
      wasmParamValueInput: wasmParamInput,
      lensEnableToggle: lensEnable,
      schemaPresetSelect,
      schemaPresetNameInput: schemaPresetName,
      lensPresetSelect,
      lensPresetNameInput: lensPresetName,
    });
    syncWasmKeyMapFromRaw();
    state.wasm.schemaMap = parseSchemaMap(state.wasm.schemaMapRaw);
    state.wasm.fixed64Map = parseFixed64Map(state.wasm.fixed64MapRaw);
    applyWasmLensStateToDom();
    if (applyLensSelection) {
      applyWasmLensSelectionToViews({ markCustom: false });
    }
  };

  const applySchemaPresetSelection = (id) => {
    if (id === "custom") {
      state.wasm.schemaPresetId = "custom";
      if (schemaPresetName) schemaPresetName.value = "";
      refreshSchemaPresetSelect();
      saveSchemaPresets();
      saveWasmSettings();
      return;
    }
    const presetRaw = state.wasm.schemaPresets?.[id];
    if (presetRaw === undefined) return;
    state.wasm.schemaPresetId = id;
    state.wasm.schemaMapRaw = presetRaw;
    state.wasm.schemaMap = parseSchemaMap(state.wasm.schemaMapRaw);
    if (schemaMapInput) schemaMapInput.value = state.wasm.schemaMapRaw ?? "";
    if (schemaPresetName) schemaPresetName.value = id;
    refreshSchemaPresetSelect();
    saveSchemaPresets();
    saveWasmSettings();
  };

  refreshSchemaPresetSelect();
  if (schemaPresetSelect) {
    schemaPresetSelect.addEventListener("change", () => {
      applySchemaPresetSelection(schemaPresetSelect.value);
    });
  }
  if (schemaPresetSave) {
    schemaPresetSave.addEventListener("click", () => {
      const rawName = schemaPresetName?.value?.trim() ?? "";
      const presetId = rawName || (state.wasm.schemaPresetId !== "custom" ? state.wasm.schemaPresetId : "");
      if (!presetId) {
        log("저장할 프리셋 이름을 입력해주세요.");
        return;
      }
      if (presetId === "custom") {
        log("custom 이름은 예약되어 있습니다.");
        return;
      }
      state.wasm.schemaPresets = state.wasm.schemaPresets ?? { default: "" };
      state.wasm.schemaPresets[presetId] = state.wasm.schemaMapRaw ?? "";
      state.wasm.schemaPresetId = presetId;
      refreshSchemaPresetSelect();
      saveSchemaPresets();
      saveWasmSettings();
      log(`스키마 프리셋 저장: ${presetId}`);
    });
  }
  if (schemaPresetDelete) {
    schemaPresetDelete.addEventListener("click", () => {
      const id = state.wasm.schemaPresetId;
      if (id === "default" || id === "custom") {
        log("default/custom 프리셋은 삭제할 수 없습니다.");
        return;
      }
      if (!state.wasm.schemaPresets?.[id]) {
        log("삭제할 프리셋이 없습니다.");
        return;
      }
      delete state.wasm.schemaPresets[id];
      state.wasm.schemaPresetId = "default";
      state.wasm.schemaMapRaw = state.wasm.schemaPresets.default ?? "";
      state.wasm.schemaMap = parseSchemaMap(state.wasm.schemaMapRaw);
      if (schemaMapInput) schemaMapInput.value = state.wasm.schemaMapRaw ?? "";
      refreshSchemaPresetSelect();
      saveSchemaPresets();
      saveWasmSettings();
      log(`스키마 프리셋 삭제: ${id}`);
    });
  }

  if (schemaMapInput) {
    schemaMapInput.addEventListener("input", () => {
      state.wasm.schemaMapRaw = schemaMapInput.value;
      state.wasm.schemaMap = parseSchemaMap(state.wasm.schemaMapRaw);
      if (schemaPresetSelect) {
        state.wasm.schemaPresetId = "custom";
        schemaPresetSelect.value = "custom";
      }
      if (schemaPresetName) schemaPresetName.value = "";
      saveWasmSettings();
    });
  }

  if (fixed64MapInput) {
    fixed64MapInput.addEventListener("input", () => {
      state.wasm.fixed64MapRaw = fixed64MapInput.value;
      state.wasm.fixed64Map = parseFixed64Map(state.wasm.fixed64MapRaw);
      saveWasmSettings();
    });
  }

  const syncParamControls = () => {
    const paramDraft = readWasmParamDraftFromControls({
      keyInput: wasmParamKeyInput,
      modeSelect: wasmParamModeSelect,
      valueInput: wasmParamInput,
    });
    state.wasm.paramKey = paramDraft.key;
    state.wasm.paramMode = paramDraft.mode;
    state.wasm.paramValue = paramDraft.value;
    saveWasmSettings();
  };
  if (wasmParamKeyInput) wasmParamKeyInput.addEventListener("input", syncParamControls);
  if (wasmParamModeSelect) wasmParamModeSelect.addEventListener("change", syncParamControls);
  if (wasmParamInput) wasmParamInput.addEventListener("input", syncParamControls);

  if (wasmParamApplyBtn) {
    wasmParamApplyBtn.addEventListener("click", () => {
      applyWasmParamByControls().catch((err) => {
        log(`WASM param 적용 실패: ${err?.message ?? err}`);
      });
    });
  }
  if (wasmParamInput) {
    wasmParamInput.addEventListener("keydown", (event) => {
      if (event.key !== "Enter") return;
      event.preventDefault();
      applyWasmParamByControls().catch((err) => {
        log(`WASM param 적용 실패: ${err?.message ?? err}`);
      });
    });
  }

  if (saveBtn) {
    saveBtn.addEventListener("click", () => {
      saveWasmSettings();
      saveWasmLensPresets();
      log("WASM 설정 저장 완료");
    });
  }

  if (loadBtn) {
    loadBtn.addEventListener("click", () => {
      loadWasmSettings();
      loadSchemaPresets();
      loadWasmLensPresets();
      reconcileWasmSettingsStateFromStores();
      syncWasmSettingsUiFromState({ applyLensSelection: true });
      log("WASM 설정 불러오기 완료");
    });
  }

  if (reloadBtn) {
    reloadBtn.addEventListener("click", () => {
      updateWasmLogicFromEditor();
      clearWasmLensTimeline();
      state.wasm.lastViewRaw = createEmptyStructuredViewRawSlots();
      updateWasmStatus();
    });
  }

  if (stopBtn) {
    stopBtn.addEventListener("click", () => {
      stopWasmLoop();
      updateWasmStatus();
    });
  }

  if (lensPresetSelect) {
    lensPresetSelect.addEventListener("change", () => {
      applyLensPresetSelection(lensPresetSelect.value);
    });
  }
  if (lensPresetSave) {
    lensPresetSave.addEventListener("click", () => {
      const result = saveLensPresetToState({
        lensState: state.wasm.lens,
        presetName: lensPresetName?.value,
        currentPreset: currentWasmLensPreset(),
        normalizePreset: normalizeWasmLensPreset,
      });
      if (!result.ok) {
        if (result.reason === "missing_name") {
          log("저장할 렌즈 프리셋 이름을 입력해주세요.");
          return;
        }
        if (result.reason === "reserved_name") {
          log("custom 이름은 예약되어 있습니다.");
          return;
        }
        log("렌즈 프리셋 저장에 실패했습니다.");
        return;
      }
      refreshLensPresetSelect();
      if (lensPresetSelect) lensPresetSelect.value = result.presetId;
      if (lensPresetName) lensPresetName.value = result.presetId;
      saveWasmLensPresets();
      saveWasmSettings();
      log(`렌즈 프리셋 저장: ${result.presetId}`);
    });
  }
  if (lensPresetDelete) {
    lensPresetDelete.addEventListener("click", () => {
      const result = deleteLensPresetFromState({ lensState: state.wasm.lens });
      if (!result.ok) {
        if (result.reason === "missing_preset") {
          log("삭제할 렌즈 프리셋이 없습니다.");
          return;
        }
        log("default/custom 렌즈 프리셋은 삭제할 수 없습니다.");
        return;
      }
      refreshLensPresetSelect();
      if (lensPresetName) lensPresetName.value = "";
      saveWasmLensPresets();
      saveWasmSettings();
      log(`렌즈 프리셋 삭제: ${result.presetId}`);
    });
  }
  if (lensEnable) {
    lensEnable.addEventListener("change", () => {
      refreshLensFromControls();
    });
  }
  if (lensXSelect) {
    lensXSelect.addEventListener("change", () => {
      refreshLensFromControls();
    });
  }
  if (lensYSelect) {
    lensYSelect.addEventListener("change", () => {
      refreshLensFromControls();
    });
  }
  if (lensY2Select) {
    lensY2Select.addEventListener("change", () => {
      refreshLensFromControls();
    });
  }

  syncWasmSettingsUiFromState({ applyLensSelection: true });
  updateWasmStatus();
  syncKeyMap();
}

// (수식 샘플 제거됨)

function log(message) {
  state.logs.unshift({ message, ts: new Date().toISOString() });
  renderLogs();
}

function upsertInputItem(item) {
  const next = {
    id: item.id,
    type: item.type,
    label: item.label ?? item.id,
    payload: item.payload ?? null,
    derived_ddn: item.derived_ddn ?? null,
    ts: new Date().toISOString(),
  };
  const items = state.inputRegistry.items;
  const idx = items.findIndex((entry) => entry.id === next.id);
  if (idx >= 0) {
    items[idx] = { ...items[idx], ...next };
  } else {
    items.unshift(next);
  }
  return next;
}

function selectInputItem(id) {
  state.inputRegistry.selectedId = id;
  renderInputRegistry();
}

function getSelectedInputItem() {
  const id = state.inputRegistry.selectedId;
  if (!id) return null;
  return state.inputRegistry.items.find((item) => item.id === id) ?? null;
}

function renderInputRegistry() {
  const list = $("input-registry");
  const selected = $("input-selected");
  if (!list || !selected) return;
  const active = getSelectedInputItem();
  selected.textContent = active ? `selected: ${active.label}` : "selected: -";
  list.innerHTML = "";
  if (!state.inputRegistry.items.length) {
    list.textContent = "-";
    return;
  }
  state.inputRegistry.items.forEach((item) => {
    const row = document.createElement("div");
    row.className = "input-registry-item";
    if (item.id === state.inputRegistry.selectedId) row.classList.add("active");
    const title = document.createElement("div");
    title.textContent = item.label ?? item.id;
    const meta = document.createElement("div");
    meta.className = "input-registry-meta";
    meta.textContent = `${item.type} · ${item.id}`;
    row.appendChild(title);
    row.appendChild(meta);
    list.appendChild(row);
  });
}

function hashString(text) {
  let hash = 0;
  for (let i = 0; i < text.length; i++) {
    hash = (hash << 5) - hash + text.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash).toString(16);
}

function normalizeNewlines(text) {
  return text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function splitMetaHeader(text) {
  const lines = normalizeNewlines(text).split("\n");
  const meta = {};
  let idx = 0;
  while (idx < lines.length) {
    const raw = lines[idx];
    const trimmed = raw.replace(/^[ \t\uFEFF]+/, "");
    if (!trimmed) {
      idx += 1;
      continue;
    }
    if (trimmed.startsWith("#") && trimmed.includes(":")) {
      const sliced = trimmed.slice(1);
      const [keyRaw, ...rest] = sliced.split(":");
      const key = keyRaw.trim();
      if (!key) break;
      meta[key] = rest.join(":").trim();
      idx += 1;
      continue;
    }
    break;
  }
  const body = lines.slice(idx).join("\n");
  return { meta, body };
}

function findControlMeta(meta) {
  if (!meta || typeof meta !== "object") return "";
  const keys = Object.keys(meta);
  const direct = meta["control"] ?? meta["조절"] ?? meta["CONTROL"] ?? meta["Control"];
  if (direct) return direct;
  const key = keys.find((k) => ["control", "조절"].includes(k.toLowerCase()));
  return key ? meta[key] : "";
}

function parseControlSpec(raw) {
  if (!raw) return null;
  const match = raw.match(
    /^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*=\s*([+-]?\d+(?:\.\d+)?)\s*(.*)$/u,
  );
  if (!match) return null;
  const name = match[1];
  const type = match[2] || "수";
  const value = Number(match[3]);
  const rest = match[4] ?? "";
  if (!Number.isFinite(value)) return null;
  let min = null;
  let max = null;
  let step = null;
  let unit = "";
  const rangeMatch = rest.match(/\[\s*([+-]?\d+(?:\.\d+)?)\s*\.\.\s*([+-]?\d+(?:\.\d+)?)\s*\]/);
  if (rangeMatch) {
    const a = Number(rangeMatch[1]);
    const b = Number(rangeMatch[2]);
    if (Number.isFinite(a) && Number.isFinite(b)) {
      min = Math.min(a, b);
      max = Math.max(a, b);
    }
  }
  const stepMatch = rest.match(/step\s*=\s*([+-]?\d+(?:\.\d+)?)/i);
  if (stepMatch) {
    const s = Number(stepMatch[1]);
    if (Number.isFinite(s) && s > 0) step = s;
  }
  const unitMatch = rest.match(/unit\s*=\s*([^\s\]]+)/i);
  if (unitMatch) unit = unitMatch[1];
  if (!Number.isFinite(min) || !Number.isFinite(max)) {
    min = Number.isFinite(value) ? value - 1 : 0;
    max = Number.isFinite(value) ? value + 1 : 1;
  }
  if (!Number.isFinite(step) || step <= 0) {
    const span = Math.abs(max - min);
    step = span > 0 ? Number((span / 100).toFixed(4)) : 0.01;
  }
  return {
    id: name,
    name,
    type,
    value,
    min,
    max,
    step,
    unit,
    raw: raw.trim(),
  };
}

function parseControlMetaLine(line) {
  if (!line) return [];
  return line
    .split(";")
    .map((entry) => entry.trim())
    .filter(Boolean)
    .map((entry) => parseControlSpec(entry))
    .filter(Boolean);
}

function readControlValuesFromDdn(text, specs) {
  if (!specs.length) return {};
  const { body } = splitMetaHeader(text);
  const lines = normalizeNewlines(body).split("\n");
  const values = {};
  specs.forEach((spec) => {
    const pattern = new RegExp(
      `^\\s*(${spec.name}\\s*(?::[^\\s]+)?)\\s*<-\\s*([+-]?\\d+(?:\\.\\d+)?)\\s*\\.\\s*$`,
    );
    for (const line of lines) {
      const match = line.match(pattern);
      if (match) {
        const num = Number(match[2]);
        if (Number.isFinite(num)) values[spec.name] = num;
        break;
      }
    }
  });
  return values;
}

function splitMetaHeaderLines(text) {
  const lines = normalizeNewlines(text).split("\n");
  let idx = 0;
  while (idx < lines.length) {
    const raw = lines[idx];
    const trimmed = raw.replace(/^[ \t\uFEFF]+/, "");
    if (!trimmed) {
      idx += 1;
      continue;
    }
    if (trimmed.startsWith("#") && trimmed.includes(":")) {
      idx += 1;
      continue;
    }
    break;
  }
  const layers = sortRunsForRender(state.runs).map((run) => ({
    id: run.id,
    label: run.label,
    visible: run.visible,
    layer: run.layerIndex ?? 0,
    series_id: run.seriesId ?? null,
    points: run.points?.length ?? 0,
    colors: run.colors,
    source: run.source,
    inputs: run.inputs,
  }));
  if (!layers.length && (state.space2d?.points?.length || state.space2d?.shapes?.length)) {
    const pointCount = state.space2d.points?.length ?? 0;
    const shapeCount = state.space2d.shapes?.length ?? 0;
    layers.push({
      id: "space2d",
      label: state.space2d.meta?.title ?? "2d",
      visible: true,
      layer: 0,
      series_id: null,
      points: pointCount || shapeCount,
      colors: { line: "#8ecae6", point: "#219ebc" },
      source: { kind: "space2d", text: "" },
      inputs: null,
    });
  }

  return {
    headerLines: lines.slice(0, idx),
    bodyLines: lines.slice(idx),
  };
}

function applyControlValuesToDdnText(text, specs, values) {
  if (!specs.length) return text;
  const { headerLines, bodyLines } = splitMetaHeaderLines(text);
  const nextLines = [...bodyLines];
  const updated = new Set();

  specs.forEach((spec) => {
    const value = values[spec.name];
    if (!Number.isFinite(value)) return;
    const pattern = new RegExp(
      `^\\s*(${spec.name}\\s*(?::[^\\s]+)?)\\s*<-\\s*.+\\.\\s*$`,
    );
    for (let i = 0; i < nextLines.length; i += 1) {
      const line = nextLines[i];
      const match = line.match(pattern);
      if (match) {
        nextLines[i] = `${match[1]} <- ${formatNumber(value)}.`;
        updated.add(spec.name);
        return;
      }
    }
  });

  const missing = specs.filter((spec) => !updated.has(spec.name));
  if (missing.length) {
    let insertAt = 0;
    let braceIndex = -1;
    for (let i = 0; i < nextLines.length; i += 1) {
      if (/^\s*(그릇채비|채비)\s*:\s*\{\s*$/.test(nextLines[i])) {
        braceIndex = i + 1;
        break;
      }
      if (/^\s*(그릇채비|채비)\s*:/.test(nextLines[i]) && nextLines[i].includes("{")) {
        braceIndex = i + 1;
        break;
      }
    }
    if (braceIndex >= 0) {
      insertAt = braceIndex;
    } else {
      while (insertAt < nextLines.length) {
        const trimmed = nextLines[insertAt].trim();
        if (!trimmed || trimmed.startsWith("#")) {
          insertAt += 1;
          continue;
        }
        break;
      }
    }
    const insertLines = missing.map((spec) => {
      const value = values[spec.name];
      return `${spec.name} <- ${formatNumber(value)}.`;
    });
    nextLines.splice(insertAt, 0, ...insertLines, "");
  }

  return [...headerLines, ...nextLines].join("\n");
}

function refreshDdnControlsFromText(text, options = {}) {
  const { meta } = splitMetaHeader(text);
  const raw = findControlMeta(meta);
  state.controls.metaRaw = raw || "";
  const specs = parseControlMetaLine(raw);
  state.controls.specs = specs;
  const list = $("ddn-control-list");
  const status = $("ddn-control-status");
  if (!list) return;
  list.innerHTML = "";
  if (!specs.length) {
    if (status) status.textContent = "control meta: -";
    list.textContent = "control 메타가 없습니다.";
    return;
  }
  const valuesFromDdn = readControlValuesFromDdn(text, specs);
  const values = {};
  specs.forEach((spec) => {
    if (options.preserveValues && Number.isFinite(state.controls.values?.[spec.name])) {
      values[spec.name] = state.controls.values[spec.name];
      return;
    }
    if (Number.isFinite(valuesFromDdn[spec.name])) {
      values[spec.name] = valuesFromDdn[spec.name];
      return;
    }
    values[spec.name] = spec.value;
  });
  state.controls.values = values;
  if (status) {
    status.textContent = `control meta: ${specs.length}개`;
  }
  specs.forEach((spec) => {
    const row = document.createElement("div");
    row.className = "control-item";
    const label = document.createElement("div");
    label.className = "control-label";
    label.textContent = `${spec.name}${spec.unit ? ` (${spec.unit})` : ""}`;
    const rangeWrap = document.createElement("div");
    rangeWrap.className = "control-range";
    const range = document.createElement("input");
    range.type = "range";
    range.min = spec.min;
    range.max = spec.max;
    range.step = spec.step;
    range.value = values[spec.name];
    const number = document.createElement("input");
    number.type = "number";
    number.className = "control-number";
    number.min = spec.min;
    number.max = spec.max;
    number.step = spec.step;
    number.value = values[spec.name];

    const commit = (next) => {
      const numeric = Number(next);
      if (!Number.isFinite(numeric)) return;
      state.controls.values[spec.name] = numeric;
      const ddnText = $("ddn-editor").value;
      const updated = applyControlValuesToDdnText(ddnText, specs, state.controls.values);
      $("ddn-editor").value = updated;
      updateSceneFromDdn(updated);
      updateDdnTimeStatus();
      refreshDdnControlsFromText(updated, { preserveValues: true });
      if (state.controls.autoRun) runDdnBridge({ mode: "auto" });
    };

    range.addEventListener("input", () => {
      number.value = range.value;
      state.controls.values[spec.name] = Number(range.value);
    });
    range.addEventListener("change", () => commit(range.value));
    number.addEventListener("input", () => {
      range.value = number.value;
      state.controls.values[spec.name] = Number(number.value);
    });
    number.addEventListener("change", () => commit(number.value));

    rangeWrap.appendChild(range);

    row.appendChild(label);
    row.appendChild(rangeWrap);
    row.appendChild(number);
    list.appendChild(row);
  });
}

function normalizeDdnForHash(text) {
  const { body } = splitMetaHeader(text);
  return body.replace(/^\n+/, "");
}

function extractDdnMeta(text) {
  const { meta } = splitMetaHeader(text);
  return {
    name: meta["이름"] || meta.name || "",
    desc: meta["설명"] || meta.desc || "",
  };
}

function extractTimeRangeFromDdn(text) {
  const match = text.match(
    /t목록\s*<-\s*\(\s*([+-]?\d+(?:\.\d+)?)\s*,\s*([+-]?\d+(?:\.\d+)?)\s*,\s*([+-]?\d+(?:\.\d+)?)\s*\)\s*범위\s*\./,
  );
  if (!match) return null;
  const tMin = Number(match[1]);
  const tMax = Number(match[2]);
  const step = Number(match[3]);
  if (!Number.isFinite(tMin) || !Number.isFinite(tMax) || !Number.isFinite(step)) return null;
  return { t_min: tMin, t_max: tMax, step };
}

function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/\"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function parseBogaeSceneFromDdn(text) {
  if (!text) return null;
  const start = text.indexOf("보개장면");
  if (start < 0) return null;
  const bracePos = text.indexOf("{", start);
  if (bracePos < 0) return null;
  let depth = 0;
  let end = -1;
  for (let i = bracePos; i < text.length; i += 1) {
    const ch = text[i];
    if (ch === "{") depth += 1;
    if (ch === "}") {
      depth -= 1;
      if (depth === 0) {
        end = i;
        break;
      }
    }
  }
  if (end < 0) return null;
  const body = text.slice(bracePos + 1, end);
  const segments = [];
  const madiRegex = /마디\s*\d+\s*\(\s*tick\s*=\s*([0-9]+)\s*~\s*([0-9]+)\s*\)\s*:\s*\{/g;
  let match;
  while ((match = madiRegex.exec(body)) !== null) {
    const tickStart = Number(match[1]);
    const tickEnd = Number(match[2]);
    const blockStart = match.index + match[0].length;
    let depth2 = 1;
    let i = blockStart;
    while (i < body.length) {
      const ch = body[i];
      if (ch === "{") depth2 += 1;
      if (ch === "}") {
        depth2 -= 1;
        if (depth2 === 0) break;
      }
      i += 1;
    }
    const block = body.slice(blockStart, i);
    const captions = [];
    const captionBlocks = [];
    const emphasisTokens = new Set();
    const views = new Set();
    const targets = new Set();
    const progressTargets = new Set();
    const lines = block.split("\n");
    lines.forEach((line) => {
      if (!line.includes("나타나기") || !line.includes("글")) return;
      const textMatch = line.match(/글\s*=\s*\"([^\"]+)\"/) || line.match(/글\s*=\s*'([^']+)'/);
      if (!textMatch) return;
      const kindMatch = line.match(/\(#([^,\s]+)[^)]*?글/);
      const rawKind = kindMatch ? kindMatch[1] : "자막";
      const kind = normalizeSceneCaptionKind(rawKind);
      const text = textMatch[1];
      captions.push(text);
      captionBlocks.push({ kind, text });
    });
    lines.forEach((line) => {
      if (!line.includes("강조하기")) return;
      for (const token of line.matchAll(/id\s*=\s*\"([^\"]+)\"/g)) emphasisTokens.add(token[1]);
      for (const token of line.matchAll(/id\s*=\s*'([^']+)'/g)) emphasisTokens.add(token[1]);
      for (const token of line.matchAll(/토큰\s*=\s*\"([^\"]+)\"/g)) emphasisTokens.add(token[1]);
      for (const token of line.matchAll(/토큰\s*=\s*'([^']+)'/g)) emphasisTokens.add(token[1]);
      for (const token of line.matchAll(/token\s*=\s*\"([^\"]+)\"/g)) emphasisTokens.add(token[1]);
      for (const token of line.matchAll(/token\s*=\s*'([^']+)'/g)) emphasisTokens.add(token[1]);
      for (const token of line.matchAll(/series\s*=\s*\"([^\"]+)\"/g)) emphasisTokens.add(token[1]);
      for (const token of line.matchAll(/series\s*=\s*'([^']+)'/g)) emphasisTokens.add(token[1]);
    });
    for (const view of block.matchAll(/보개\s*=\s*\"([^\"]+)\"/g)) views.add(view[1]);
    for (const view of block.matchAll(/보개\s*=\s*'([^']+)'/g)) views.add(view[1]);
    for (const token of block.matchAll(/id\s*=\s*\"([^\"]+)\"/g)) targets.add(token[1]);
    for (const token of block.matchAll(/id\s*=\s*'([^']+)'/g)) targets.add(token[1]);
    for (const token of block.matchAll(/토큰\s*=\s*\"([^\"]+)\"/g)) targets.add(token[1]);
    for (const token of block.matchAll(/토큰\s*=\s*'([^']+)'/g)) targets.add(token[1]);
    for (const token of block.matchAll(/token\s*=\s*\"([^\"]+)\"/g)) targets.add(token[1]);
    for (const token of block.matchAll(/token\s*=\s*'([^']+)'/g)) targets.add(token[1]);
    for (const token of block.matchAll(/series\s*=\s*\"([^\"]+)\"/g)) targets.add(token[1]);
    for (const token of block.matchAll(/series\s*=\s*'([^']+)'/g)) targets.add(token[1]);
    for (const line of block.split("\n")) {
      if (!line.includes("그려지기")) continue;
      if (!line.includes("progress")) continue;
      for (const token of line.matchAll(/id\s*=\s*\"([^\"]+)\"/g)) progressTargets.add(token[1]);
      for (const token of line.matchAll(/id\s*=\s*'([^']+)'/g)) progressTargets.add(token[1]);
      for (const token of line.matchAll(/토큰\s*=\s*\"([^\"]+)\"/g)) progressTargets.add(token[1]);
      for (const token of line.matchAll(/토큰\s*=\s*'([^']+)'/g)) progressTargets.add(token[1]);
      for (const token of line.matchAll(/token\s*=\s*\"([^\"]+)\"/g)) progressTargets.add(token[1]);
      for (const token of line.matchAll(/token\s*=\s*'([^']+)'/g)) progressTargets.add(token[1]);
      for (const token of line.matchAll(/series\s*=\s*\"([^\"]+)\"/g)) progressTargets.add(token[1]);
      for (const token of line.matchAll(/series\s*=\s*'([^']+)'/g)) progressTargets.add(token[1]);
    }
    segments.push({
      tick_start: Number.isFinite(tickStart) ? tickStart : 0,
      tick_end: Number.isFinite(tickEnd) ? tickEnd : 0,
      captions,
      caption_blocks: captionBlocks,
      views: Array.from(views),
      targets: Array.from(targets),
      progress_targets: Array.from(progressTargets),
      emphasis_tokens: Array.from(emphasisTokens),
    });
    madiRegex.lastIndex = i + 1;
  }
  if (!segments.length) return null;
  return { segments };
}

function normalizeSceneCaptionKind(raw) {
  const key = String(raw ?? "").replace(/^#/, "").trim().toLowerCase();
  if (["title", "제목"].includes(key)) return "제목";
  if (["caption", "subtitle", "자막"].includes(key)) return "자막";
  if (["explain", "설명", "해설"].includes(key)) return "해설";
  return raw ? raw.replace(/^#/, "") : "자막";
}

function normalizeSceneViewKey(raw) {
  const key = String(raw ?? "").toLowerCase();
  if (["graph", "그래프", "axis"].includes(key)) return "view-graph";
  if (["2d", "space2d", "공간", "공간2d"].includes(key)) return "view-2d";
  if (["text", "글", "자막", "해설"].includes(key)) return "view-text";
  if (["table", "표"].includes(key)) return "view-table";
  if (["structure", "구조"].includes(key)) return "view-structure";
  return null;
}

function normalizeSceneTarget(raw) {
  const key = String(raw ?? "").trim().toLowerCase();
  if (key.startsWith("series:")) return key.slice("series:".length);
  return key;
}

function getSceneTargetSet(segment) {
  const targets = segment?.targets ?? [];
  if (!targets.length) return null;
  const set = new Set();
  targets.forEach((target) => {
    const key = normalizeSceneTarget(target);
    if (key) set.add(key);
  });
  return set.size ? set : null;
}

function getSceneEmphasisTokenSet(segment) {
  const targets = segment?.emphasis_tokens ?? [];
  if (!targets.length) return null;
  const set = new Set();
  targets.forEach((target) => {
    const key = normalizeSceneTarget(target);
    if (key) set.add(key);
  });
  return set.size ? set : null;
}

function getSceneProgressTargetSet(segment) {
  const targets = segment?.progress_targets ?? [];
  if (!targets.length) return null;
  const set = new Set();
  targets.forEach((target) => {
    const key = normalizeSceneTarget(target);
    if (key) set.add(key);
  });
  return set.size ? set : null;
}

function runMatchesSceneTarget(run, targetSet) {
  if (!targetSet) return true;
  const candidates = [
    run?.label,
    run?.seriesId,
    run?.graph?.series?.[0]?.id,
    run?.graph?.series?.[0]?.label,
    ...(run?.graph?.meta?.series_labels ?? []),
  ];
  return candidates.some((value) => targetSet.has(normalizeSceneTarget(value)));
}

function shapeMatchesSceneTarget(shape, targetSet) {
  if (!targetSet) return true;
  const candidates = [shape?.token, shape?.id, shape?.name, shape?.label];
  return candidates.some((value) => targetSet.has(normalizeSceneTarget(value)));
}

function getSceneShapeFactor(shape) {
  const segment = getActiveSceneSegment();
  const targetSet = getSceneTargetSet(segment);
  if (!targetSet) return 1;
  return shapeMatchesSceneTarget(shape, targetSet) ? 1 : 0.25;
}

function getSceneRunFactor(run) {
  const segment = getActiveSceneSegment();
  const targetSet = getSceneTargetSet(segment);
  if (!targetSet) return 1;
  return runMatchesSceneTarget(run, targetSet) ? 1 : 0.25;
}

function getSceneProgressForRun(run) {
  const segment = getActiveSceneSegment();
  const targetSet = getSceneProgressTargetSet(segment);
  if (!targetSet) return 1;
  if (!runMatchesSceneTarget(run, targetSet)) return 1;
  const tick = Number.isFinite(state.time.index) ? state.time.index : 0;
  const start = Number.isFinite(segment?.tick_start) ? segment.tick_start : 0;
  const end = Number.isFinite(segment?.tick_end) ? segment.tick_end : start;
  const span = end - start;
  if (span <= 0) {
    return tick >= start ? 1 : 0;
  }
  const raw = (tick - start) / span;
  return Math.min(1, Math.max(0, raw));
}

function getSceneProgressForShape(shape) {
  const segment = getActiveSceneSegment();
  const targetSet = getSceneProgressTargetSet(segment);
  if (!targetSet) return 1;
  if (!shapeMatchesSceneTarget(shape, targetSet)) return 1;
  const tick = Number.isFinite(state.time.index) ? state.time.index : 0;
  const start = Number.isFinite(segment?.tick_start) ? segment.tick_start : 0;
  const end = Number.isFinite(segment?.tick_end) ? segment.tick_end : start;
  const span = end - start;
  if (span <= 0) {
    return tick >= start ? 1 : 0;
  }
  const raw = (tick - start) / span;
  return Math.min(1, Math.max(0, raw));
}

function getActiveSceneSegment() {
  if (!state.scene?.segments?.length) return null;
  const tick = Number.isFinite(state.time.index) ? state.time.index : 0;
  const segment = state.scene.segments.find((seg) => tick >= seg.tick_start && tick <= seg.tick_end);
  return segment ?? state.scene.segments[0];
}

function updateSceneFromDdn(text) {
  state.scene = parseBogaeSceneFromDdn(text);
  renderScenePreview();
  renderTextView();
  renderCaptionOverlay();
}

function renderMarkdownInline(text) {
  let safe = escapeHtml(text);
  safe = safe.replace(/`([^`]+)`/g, (_m, code) => `<code>${escapeHtml(code)}</code>`);
  safe = safe.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_m, label, url) => {
    const href = encodeURI(url);
    return `<a href="${href}" target="_blank" rel="noopener noreferrer">${escapeHtml(label)}</a>`;
  });
  return safe;
}

function renderMarkdownSafe(text) {
  const blocks = text.split(/```/);
  let html = "";
  blocks.forEach((block, idx) => {
    if (idx % 2 === 1) {
      const lines = block.split("\n");
      if (lines.length > 1) lines.shift();
      html += `<pre><code>${escapeHtml(lines.join("\n"))}</code></pre>`;
      return;
    }
    const lines = block.split("\n");
    let inList = false;
    lines.forEach((line) => {
      if (/^###\s+/.test(line)) {
        if (inList) {
          html += "</ul>";
          inList = false;
        }
        html += `<h3>${renderMarkdownInline(line.replace(/^###\s+/, ""))}</h3>`;
        return;
      }
      if (/^##\s+/.test(line)) {
        if (inList) {
          html += "</ul>";
          inList = false;
        }
        html += `<h2>${renderMarkdownInline(line.replace(/^##\s+/, ""))}</h2>`;
        return;
      }
      if (/^#\s+/.test(line)) {
        if (inList) {
          html += "</ul>";
          inList = false;
        }
        html += `<h1>${renderMarkdownInline(line.replace(/^#\s+/, ""))}</h1>`;
        return;
      }
      if (/^-\s+/.test(line)) {
        if (!inList) {
          html += "<ul>";
          inList = true;
        }
        html += `<li>${renderMarkdownInline(line.replace(/^-\\s+/, ""))}</li>`;
        return;
      }
      if (inList) {
        html += "</ul>";
        inList = false;
      }
      if (!line.trim()) {
        html += "<br />";
        return;
      }
      html += `<p>${renderMarkdownInline(line)}</p>`;
    });
    if (inList) html += "</ul>";
  });
  return html;
}

function parseCsv(text) {
  const rows = [];
  let current = "";
  let row = [];
  let inQuotes = false;
  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];
    const next = text[i + 1];
    if (ch === '"' && next === '"') {
      current += '"';
      i += 1;
      continue;
    }
    if (ch === '"') {
      inQuotes = !inQuotes;
      continue;
    }
    if (!inQuotes && (ch === "," || ch === "\n" || ch === "\r")) {
      if (ch === "\r" && next === "\n") i += 1;
      row.push(current);
      current = "";
      if (ch !== ",") {
        if (row.some((cell) => cell !== "")) rows.push(row);
        row = [];
      }
      continue;
    }
    current += ch;
  }
  if (current.length || row.length) {
    row.push(current);
    if (row.some((cell) => cell !== "")) rows.push(row);
  }
  return rows;
}

function rowsToTableData(rows) {
  if (!rows || rows.length === 0) return null;
  const headerRow = rows[0];
  const columns = headerRow.map((label, idx) => ({ key: `c${idx}`, label: label || `c${idx + 1}` }));
  const rowObjects = rows.slice(1).map((row) => {
    const obj = {};
    headerRow.forEach((_h, idx) => {
      const raw = row[idx] ?? "";
      const num = Number(raw);
      obj[`c${idx}`] = Number.isFinite(num) && raw.trim() !== "" ? num : raw;
    });
    return obj;
  });
  return { schema: "seamgrim.table.v0", columns, rows: rowObjects };
}

function normalizeTableData(data) {
  if (!data) return null;
  if (data.matrix && Array.isArray(data.matrix.values)) {
    const values = data.matrix.values;
    const colLabels =
      data.matrix.col_labels ?? (values[0] ? values[0].map((_, idx) => `c${idx + 1}`) : []);
    const rowLabels =
      data.matrix.row_labels ?? values.map((_, idx) => `r${idx + 1}`);
    const columns = [{ key: "__row__", label: "#" }].concat(
      colLabels.map((label, idx) => ({ key: `c${idx}`, label, type: "number" })),
    );
    const rows = values.map((rowValues, rowIdx) => {
      const row = { "__row__": rowLabels[rowIdx] ?? rowIdx + 1 };
      rowValues.forEach((value, colIdx) => {
        row[`c${colIdx}`] = value;
      });
      return row;
    });
    return {
      schema: "seamgrim.table.v0",
      columns,
      rows,
      meta: data.meta ?? {},
    };
  }
  if (Array.isArray(data.columns) && Array.isArray(data.rows)) {
    return data;
  }
  return null;
}

function validateGraphData(data) {
  if (!data || data.schema !== "seamgrim.graph.v0") {
    throw new Error("지원하지 않는 그래프 스키마입니다.");
  }
  if (!Array.isArray(data.series) || data.series.length === 0) {
    throw new Error("그래프 series가 없습니다.");
  }
  data.series.forEach((series) => {
    if (!Array.isArray(series.points)) {
      throw new Error("points 배열이 없습니다.");
    }
    series.points.forEach((point) => {
      if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) {
        throw new Error("point 값이 숫자가 아닙니다.");
      }
    });
  });
  return data;
}

function validateSpace2dData(data) {
  if (!data || data.schema !== "seamgrim.space2d.v0") {
    throw new Error("지원하지 않는 2D 보개 스키마입니다.");
  }
  const points = Array.isArray(data.points) ? data.points : [];
  const shapes = Array.isArray(data.shapes) ? data.shapes : [];
  const drawItems = extractDrawlistItems(data.drawlist);
  if (points.length === 0 && shapes.length === 0 && drawItems.length === 0) {
    throw new Error("2D points 또는 shapes가 없습니다.");
  }
  points.forEach((point) => {
    if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) {
      throw new Error("2D point 값이 숫자가 아닙니다.");
    }
  });
  shapes.forEach((shape) => {
    if (!shape || typeof shape !== "object") {
      throw new Error("2D shape 형식이 올바르지 않습니다.");
    }
    const kind = String(shape.kind ?? "").toLowerCase();
    if (kind === "line") {
      if (![shape.x1, shape.y1, shape.x2, shape.y2].every(Number.isFinite)) {
        throw new Error("2D line shape 좌표가 올바르지 않습니다.");
      }
      return;
    }
    if (kind === "circle") {
      if (![shape.x, shape.y, shape.r].every(Number.isFinite)) {
        throw new Error("2D circle shape 좌표가 올바르지 않습니다.");
      }
      return;
    }
    if (kind === "point") {
      if (![shape.x, shape.y].every(Number.isFinite)) {
        throw new Error("2D point shape 좌표가 올바르지 않습니다.");
      }
      return;
    }
    if (![shape.x, shape.y].every(Number.isFinite)) {
      throw new Error("2D shape 좌표가 올바르지 않습니다.");
    }
  });
  drawItems.forEach((item) => {
    if (!item || typeof item !== "object") {
      throw new Error("2D drawlist 항목이 올바르지 않습니다.");
    }
  });
  return { ...data, points, shapes };
}

function normalizeTextDoc(data) {
  if (!data) return null;
  if (typeof data === "string") {
    return { content: data, format: "plain" };
  }
  return {
    content: data.content ?? "",
    format: data.format ?? "markdown",
  };
}

function validateStructureData(data) {
  if (!Array.isArray(data?.nodes) || !Array.isArray(data?.edges)) {
    throw new Error("구조 스키마가 올바르지 않습니다.");
  }
  const nodeIds = new Set(data.nodes.map((node) => node.id));
  const badEdges = data.edges.filter((edge) => !nodeIds.has(edge.from) || !nodeIds.has(edge.to));
  if (badEdges.length) {
    log("구조 오류: 존재하지 않는 노드 참조가 있습니다.");
  }
  return data;
}

function injectTimeValueToDdn(text, timeValue) {
  if (!Number.isFinite(timeValue)) {
    return { ok: true, text, mode: "none" };
  }
  if (text.includes("{{t}}")) {
    return {
      ok: true,
      text: text.split("{{t}}").join(formatNumber(timeValue)),
      mode: "placeholder",
    };
  }

  const hasHidden = text.includes("#바탕숨김");
  const { headerLines, bodyLines } = splitMetaHeaderLines(text);
  const assignLine = `t <- ${formatNumber(timeValue)}.`;

  for (let i = 0; i < bodyLines.length; i += 1) {
    const line = bodyLines[i];
    if (/^\s*t\s*<-\s*.+\.\s*$/.test(line)) {
      bodyLines[i] = assignLine;
      return { ok: true, text: [...headerLines, ...bodyLines].join("\n"), mode: "replace" };
    }
  }

  if (hasHidden) {
    return {
      ok: false,
      text,
      warning: "바탕숨김 DDN은 {{t}} 또는 t <- 선언이 필요합니다.",
    };
  }

  let insertAt = 0;
  while (insertAt < bodyLines.length) {
    const trimmed = bodyLines[insertAt].trim();
    if (!trimmed || trimmed.startsWith("#")) {
      insertAt += 1;
      continue;
    }
    break;
  }
  bodyLines.splice(insertAt, 0, assignLine, "");
  return { ok: true, text: [...headerLines, ...bodyLines].join("\n"), mode: "insert" };
}

function updateDdnFrameProgress({ running, done, total, message }) {
  const bar = $("ddn-frame-progress");
  const status = $("ddn-frame-status");
  const stopButton = $("ddn-frame-stop");
  const safeTotal = total > 0 ? total : 0;
  const ratio = safeTotal ? Math.min(1, done / safeTotal) : 0;
  if (bar) bar.style.width = `${Math.round(ratio * 100)}%`;
  if (status) {
    if (message) {
      status.textContent = message;
    } else if (running) {
      status.textContent = `DDN 프레임: ${done}/${safeTotal}`;
    } else {
      status.textContent = "DDN 프레임: -";
    }
  }
  if (stopButton) {
    stopButton.disabled = !running;
  }
}

function startDdnFrameJob(total) {
  state.time.ddnJob = { running: true, cancelled: false, done: 0, total };
  updateDdnFrameProgress({ running: true, done: 0, total });
}

function finishDdnFrameJob(message) {
  state.time.ddnJob.running = false;
  updateDdnFrameProgress({
    running: false,
    done: state.time.ddnJob.done,
    total: state.time.ddnJob.total,
    message,
  });
}

function cancelDdnFrameJob() {
  if (!state.time.ddnJob.running) return;
  state.time.ddnJob.cancelled = true;
  updateDdnFrameProgress({
    running: true,
    done: state.time.ddnJob.done,
    total: state.time.ddnJob.total,
    message: "DDN 프레임: 중단 요청됨",
  });
}

async function sha256Hex(text) {
  const buffer = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return Array.from(new Uint8Array(buffer))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function canonicalJson(data) {
  return JSON.stringify(data);
}

function normalizePoints(points, digits = 4) {
  return points.map((point) => ({
    x: Number(Number(point.x).toFixed(digits)),
    y: Number(Number(point.y).toFixed(digits)),
  }));
}

function frameSignature(points) {
  return hashString(JSON.stringify(normalizePoints(points ?? [], 4)));
}

function normalizeShapeSignature(shape) {
  if (!shape || typeof shape !== "object") return shape;
  const out = {};
  Object.keys(shape)
    .sort()
    .forEach((key) => {
      out[key] = shape[key];
    });
  return out;
}

function extractDrawlistItems(drawlist) {
  if (!drawlist) return [];
  if (Array.isArray(drawlist)) return drawlist;
  if (Array.isArray(drawlist.items)) return drawlist.items;
  if (Array.isArray(drawlist.list)) return drawlist.list;
  return [];
}

function extractSpace2dCamera(space2d) {
  if (!space2d) return null;
  return space2d.camera ?? space2d.view ?? space2d.drawlist?.camera ?? null;
}

function normalizeDrawItemSignature(item) {
  if (!item || typeof item !== "object") return item;
  const out = {};
  Object.keys(item)
    .sort()
    .forEach((key) => {
      const value = item[key];
      if (Array.isArray(value)) {
        out[key] = value.map((entry) => {
          if (entry && typeof entry === "object" && "x" in entry && "y" in entry) {
            return {
              x: Number(Number(entry.x).toFixed(4)),
              y: Number(Number(entry.y).toFixed(4)),
            };
          }
          return entry;
        });
      } else if (value && typeof value === "object" && "x" in value && "y" in value) {
        out[key] = {
          x: Number(Number(value.x).toFixed(4)),
          y: Number(Number(value.y).toFixed(4)),
        };
      } else {
        out[key] = value;
      }
    });
  return out;
}

function space2dSignature(space2d) {
  if (!space2d) return "none";
  const points = normalizePoints(space2d.points ?? [], 4);
  const shapes = (space2d.shapes ?? []).map(normalizeShapeSignature);
  const drawItems = extractDrawlistItems(space2d.drawlist);
  const drawlist = drawItems.map(normalizeDrawItemSignature);
  return hashString(JSON.stringify({ points, shapes, drawlist }));
}

function textSignature(textDoc) {
  if (!textDoc || !textDoc.content) return "none";
  return hashString(JSON.stringify({ content: textDoc.content, format: textDoc.format ?? "markdown" }));
}

function framesAreStatic(frames) {
  if (!frames.length) return false;
  const first = frameSignature(frames[0].points ?? []);
  const firstSpace = space2dSignature(frames[0].space2d ?? null);
  const firstText = textSignature(frames[0].textDoc ?? null);
  return frames.every(
    (frame) =>
      frameSignature(frame.points ?? []) === first &&
      space2dSignature(frame.space2d ?? null) === firstSpace &&
      textSignature(frame.textDoc ?? null) === firstText,
  );
}

function buildSyntheticFramesFromPoints(points, timeState) {
  if (!points || !points.length) return [];
  const tMin = Number.isFinite(timeState.t_min) ? timeState.t_min : 0;
  const tMax = Number.isFinite(timeState.t_max) ? timeState.t_max : tMin + points.length - 1;
  let step = Number.isFinite(timeState.step) && timeState.step > 0 ? timeState.step : null;
  let count = 0;
  if (step) {
    count = Math.floor((tMax - tMin) / step) + 1;
  } else {
    count = points.length;
    step = count > 1 ? (tMax - tMin) / (count - 1) : 1;
  }
  if (!Number.isFinite(step) || step <= 0) step = 1;
  if (!Number.isFinite(count) || count <= 0) count = points.length;
  if (count > 200) {
    count = 200;
    step = (tMax - tMin) / Math.max(count - 1, 1);
  }
  const frames = [];
  for (let i = 0; i < count; i += 1) {
    const ratio = count === 1 ? 0 : i / (count - 1);
    const idx = Math.round(ratio * (points.length - 1));
    const t = tMin + step * i;
    frames.push({ t: Number(t.toFixed(4)), points: [points[idx]] });
  }
  return frames;
}

async function computeResultHash(points) {
  const normalized = normalizePoints(points, 4);
  return sha256Hex(canonicalJson(normalized));
}

async function computeLocalInputHash(payload) {
  const text = canonicalJson(payload);
  const hash = await sha256Hex(text);
  return `local:${hash}`;
}
function readNumber(id, fallback = null) {
  const raw = $(id).value.trim();
  if (!raw) return fallback;
  const value = Number(raw);
  return Number.isFinite(value) ? value : fallback;
}

function readTimeControls() {
  return {
    enabled: $("time-enabled")?.checked ?? false,
    t_min: readNumber("t-min", 0),
    t_max: readNumber("t-max", 1),
    step: readNumber("t-step", 0.5),
    now: readNumber("t-now", 0),
    interval: readNumber("t-interval", 300),
    loop: $("time-loop")?.checked ?? true,
  };
}

function syncTimeCursorRange() {
  const timeState = readTimeControls();
  const slider = $("t-cursor");
  slider.min = Number.isFinite(timeState.t_min) ? timeState.t_min : 0;
  slider.max = Number.isFinite(timeState.t_max) ? timeState.t_max : 1;
  slider.step = Number.isFinite(timeState.step) ? timeState.step : 0.1;
  slider.value = Number.isFinite(timeState.now) ? timeState.now : slider.min;
  renderTimelineBar();
}

function setTimeNow(value, options = {}) {
  if (!Number.isFinite(value)) return;
  $("t-now").value = formatNumber(value);
  const slider = $("t-cursor");
  if (slider) slider.value = value;
  if (options.render) applyTimeFrame({ keepPlaying: true });
  renderTimelineBar();
}

function markTimeFramesDirty() {
  state.time.frames = [];
  state.time.index = 0;
  state.time.lastKey = null;
  renderTimelineBar();
}

function getTimelineFrames(timeState) {
  const run = getActiveRun();
  if (run?.timeFrames?.length) {
    return run.timeFrames.map((frame) => frame.t);
  }
  if (state.time.frames?.length) {
    return state.time.frames.map((frame) => frame.t);
  }
  if (!timeState.enabled) return [];
  const frames = [];
  let t = timeState.t_min;
  let guard = 0;
  while (t <= timeState.t_max + 1e-9 && guard < 200) {
    frames.push(Number(t.toFixed(4)));
    t += timeState.step;
    guard += 1;
  }
  return frames;
}

function jumpToTime(value) {
  if (!Number.isFinite(value)) return;
  setTimeNow(value, { render: false });
  const run = getActiveRun();
  if (run?.timeFrames?.length) {
    const idx = findNearestFrameIndex(run.timeFrames, value);
    state.time.index = idx;
    applyTimeFrame();
    return;
  }
  renderAll();
  renderInspector();
}

function renderTimelineBar() {
  const track = $("timeline-track");
  const marks = $("timeline-marks");
  const playhead = $("timeline-playhead");
  const meta = $("timeline-meta");
  if (!track || !marks || !playhead || !meta) return;

  const timeState = readTimeControls();
  if (!timeState.enabled) {
    marks.innerHTML = "";
    playhead.style.left = "0%";
    track.classList.add("disabled");
    meta.textContent = "t 타임라인: 꺼짐";
    return;
  }
  track.classList.remove("disabled");
  const frames = getTimelineFrames(timeState);
  const fallbackMin = Number.isFinite(timeState.t_min) ? timeState.t_min : 0;
  const fallbackMax = Number.isFinite(timeState.t_max) ? timeState.t_max : fallbackMin + 1;
  const tMin = frames.length ? Math.min(...frames) : fallbackMin;
  const tMax = frames.length ? Math.max(...frames) : fallbackMax;
  const range = tMax - tMin || 1;

  const maxMarks = 60;
  const step = Math.max(1, Math.ceil(frames.length / maxMarks));
  marks.innerHTML = "";
  frames.forEach((t, idx) => {
    if (idx % step !== 0 && idx !== frames.length - 1) return;
    const mark = document.createElement("span");
    mark.className = "timeline-mark";
    const pos = ((t - tMin) / range) * 100;
    mark.style.left = `${Math.min(100, Math.max(0, pos))}%`;
    mark.title = `t=${formatNumber(t)}`;
    mark.addEventListener("click", (event) => {
      event.stopPropagation();
      jumpToTime(t);
    });
    marks.appendChild(mark);
  });

  const now = Number.isFinite(timeState.now) ? timeState.now : tMin;
  const nowPos = ((now - tMin) / range) * 100;
  playhead.style.left = `${Math.min(100, Math.max(0, nowPos))}%`;
  meta.textContent = `t 타임라인: ${formatNumber(now)} / ${formatNumber(tMin)}..${formatNumber(tMax)}`;
}

function formatNumber(value) {
  if (!Number.isFinite(value)) return "0";
  if (Number.isInteger(value)) return value.toString();
  return Number(value.toFixed(4)).toString();
}

function registerDdnInput(ddnText, options = {}) {
  const meta = extractDdnMeta(ddnText);
  const label = options.label || meta.name || "DDN";
  const item = upsertInputItem({
    id: "ddn:editor",
    type: "ddn",
    label,
    payload: { text: ddnText },
  });
  selectInputItem(item.id);
  return item;
}

function registerLessonInput(lessonId, meta, ddnText) {
  const label = meta?.title || lessonId || "lesson";
  const item = upsertInputItem({
    id: `lesson:${lessonId}`,
    type: "lesson",
    label,
    payload: {
      lesson_id: lessonId,
      required_views: meta?.required_views ?? [],
    },
    derived_ddn: ddnText ?? null,
  });
  selectInputItem(item.id);
  return item;
}

function getUpdateMetaFromControls() {
  const mode = $("update-mode").value;
  const tickRaw = $("update-tick").value.trim();
  const tickValue = tickRaw ? Number(tickRaw) : null;
  return {
    update: mode,
    tick: Number.isFinite(tickValue) ? tickValue : null,
  };
}

// (수식 기반 DDN 생성/미리보기 제거됨)
function pickOverlayColors(label) {
  const base = label || "overlay";
  let hash = 0;
  for (let i = 0; i < base.length; i++) {
    hash = (hash << 5) - hash + base.charCodeAt(i);
    hash |= 0;
  }
  const idx = Math.abs(hash) % overlayPalette.length;
  return overlayPalette[idx];
}

function graphLabel(graph, series = null) {
  return (
    series?.label ??
    series?.id ??
    graph?.series?.[0]?.label ??
    graph?.meta?.input_name ??
    graph?.meta?.input_desc ??
    "오버레이"
  );
}

function getGraphKind(graph) {
  return graph?.meta?.graph_kind ?? graph?.meta?.kind ?? graph?.schema ?? "";
}

function getAxisSignature(graph) {
  const meta = graph?.meta ?? {};
  return {
    graphKind: getGraphKind(graph),
    sampleVar: graph?.sample?.var ?? null,
    xKind: meta.axis_x_kind ?? meta.x_kind ?? meta.axis_kind ?? null,
    xUnit: meta.axis_x_unit ?? meta.x_unit ?? meta.axis_unit ?? null,
    yKind: meta.axis_y_kind ?? meta.y_kind ?? null,
    yUnit: meta.axis_y_unit ?? meta.y_unit ?? null,
  };
}

function axisSignatureEquals(a, b) {
  return JSON.stringify(a ?? {}) === JSON.stringify(b ?? {});
}

function getRunSeriesKey(run) {
  return run?.seriesId ?? run?.graph?.series?.[0]?.id ?? null;
}

function canOverlayCompare(baselineRun, variantRun) {
  if (!baselineRun?.graph || !variantRun?.graph) {
    return { ok: false, reason: "그래프 데이터가 없습니다." };
  }
  const baselineSig = getAxisSignature(baselineRun.graph);
  const variantSig = getAxisSignature(variantRun.graph);
  if (!axisSignatureEquals(baselineSig, variantSig)) {
    return { ok: false, reason: "축 의미(종류/단위)가 달라 비교할 수 없습니다." };
  }
  const baselineSeries = getRunSeriesKey(baselineRun);
  const variantSeries = getRunSeriesKey(variantRun);
  if (baselineSeries && variantSeries && baselineSeries !== variantSeries) {
    return { ok: false, reason: "series_id가 달라 비교할 수 없습니다." };
  }
  return { ok: true, reason: "" };
}

function syncCompareSequenceButtons() {
  const playBtn = $("compare-play");
  const stopBtn = $("compare-stop");
  const baseline = state.runs.find((run) => run.id === state.compare.baselineId);
  const variant = state.runs.find((run) => run.id === state.compare.variantId);
  const ready = state.compare.enabled && baseline && variant && !state.compare.blockReason;
  if (playBtn) playBtn.disabled = !ready || state.compare.sequence.playing;
  if (stopBtn) stopBtn.disabled = !state.compare.sequence.playing;
}

function applyCompareSequenceVisibility() {
  if (!state.compare.enabled) return;
  const baselineId = state.compare.baselineId;
  const variantId = state.compare.variantId;
  const showVariant = state.compare.sequence.showVariant;
  state.runs.forEach((run) => {
    if (!(run.id in state.compare.savedVisibility)) {
      state.compare.savedVisibility[run.id] = run.visible;
    }
    if (run.id === baselineId) {
      run.visible = !showVariant;
      return;
    }
    if (run.id === variantId) {
      run.visible = showVariant;
      return;
    }
    run.visible = false;
  });
  renderOverlayList();
  renderRunList();
  renderAll();
}

function stopCompareSequence(options = {}) {
  if (state.compare.sequence.timer) {
    clearInterval(state.compare.sequence.timer);
    state.compare.sequence.timer = null;
  }
  const wasPlaying = state.compare.sequence.playing;
  state.compare.sequence.playing = false;
  state.compare.sequence.showVariant = false;
  if (options.restoreVisibility !== false) {
    syncCompareVisibility();
    renderOverlayList();
    renderRunList();
    renderAll();
  }
  syncCompareSequenceButtons();
  updateCompareStatusUI();
  if (wasPlaying && options.log !== false) {
    log("before/after 순차 비교를 중지했습니다.");
  }
}

function startCompareSequence() {
  if (!state.compare.enabled) {
    log("비교 모드를 먼저 켜주세요.");
    return;
  }
  const baseline = state.runs.find((run) => run.id === state.compare.baselineId);
  const variant = state.runs.find((run) => run.id === state.compare.variantId);
  if (!baseline || !variant) {
    log("before/after 비교에는 baseline과 variant가 모두 필요합니다.");
    return;
  }
  if (state.compare.blockReason) {
    log(`비교 불가 상태입니다: ${state.compare.blockReason}`);
    return;
  }
  const intervalRaw = Number($("compare-interval")?.value ?? state.compare.sequence.intervalMs);
  const intervalMs = Math.min(5000, Math.max(120, Number.isFinite(intervalRaw) ? intervalRaw : 800));
  state.compare.sequence.intervalMs = intervalMs;

  stopCompareSequence({ restoreVisibility: false, log: false });
  state.compare.sequence.playing = true;
  state.compare.sequence.showVariant = false;
  applyCompareSequenceVisibility();
  state.compare.sequence.timer = setInterval(() => {
    state.compare.sequence.showVariant = !state.compare.sequence.showVariant;
    applyCompareSequenceVisibility();
    updateCompareStatusUI();
  }, intervalMs);
  syncCompareSequenceButtons();
  updateCompareStatusUI();
  log(`before/after 순차 비교 시작 (${intervalMs}ms)`);
}

function updateCompareStatusUI() {
  const el = $("compare-status");
  if (!el) return;
  const toggle = $("compare-enabled");
  if (toggle) toggle.checked = state.compare.enabled;
  const baseline = state.runs.find((run) => run.id === state.compare.baselineId);
  const variant = state.runs.find((run) => run.id === state.compare.variantId);
  if (!state.compare.enabled) {
    el.textContent = "비교 모드: 꺼짐";
    syncCompareSequenceButtons();
    return;
  }
  if (state.compare.blockReason) {
    el.textContent = `비교 불가: ${state.compare.blockReason}`;
    syncCompareSequenceButtons();
    return;
  }
  const seq =
    state.compare.sequence.playing
      ? ` / 순차: ${state.compare.sequence.showVariant ? "after" : "before"} (${state.compare.sequence.intervalMs}ms)`
      : "";
  el.textContent = `baseline: ${baseline?.label ?? "-"} / variant: ${variant?.label ?? "-"}${seq}`;
  syncCompareSequenceButtons();
}

function syncCompareVisibility() {
  if (!state.compare.enabled) return;
  if (state.compare.sequence.playing) return;
  const baselineId = state.compare.baselineId;
  const variantId = state.compare.variantId;
  state.runs.forEach((run) => {
    if (!(run.id in state.compare.savedVisibility)) {
      state.compare.savedVisibility[run.id] = run.visible;
    }
    run.visible = run.id === baselineId || run.id === variantId;
  });
}

function clearCompareState() {
  stopCompareSequence({ restoreVisibility: false, log: false });
  state.compare.enabled = false;
  state.compare.baselineId = null;
  state.compare.variantId = null;
  state.compare.axisSig = null;
  state.compare.seriesId = null;
  state.compare.blockReason = "";
  const saved = state.compare.savedVisibility;
  Object.keys(saved).forEach((id) => {
    const run = state.runs.find((item) => item.id === id);
    if (run) run.visible = saved[id];
  });
  state.compare.savedVisibility = {};
  state.runs.forEach((run) => {
    if (run.compareRole) run.compareRole = null;
  });
  renderOverlayList();
  renderRunList();
  renderAll();
  updateCompareStatusUI();
}

function setCompareEnabled(enabled) {
  if (!enabled) {
    stopCompareSequence({ restoreVisibility: true, log: false });
  }
  state.compare.enabled = enabled;
  state.compare.blockReason = "";
  if (!enabled) {
    clearCompareState();
    return;
  }
  const active = getActiveRun() ?? state.runs[0] ?? null;
  if (active) {
    active.compareRole = "baseline";
    state.compare.baselineId = active.id;
    state.compare.axisSig = getAxisSignature(active.graph);
    state.compare.seriesId = getRunSeriesKey(active);
    state.compare.variantId = null;
    syncCompareVisibility();
    setActiveRun(active.id);
  } else {
    state.compare.baselineId = null;
  }
  updateCompareStatusUI();
  renderOverlayList();
  renderRunList();
  renderAll();
}

function setCompareBaseline(run) {
  if (!run) return;
  stopCompareSequence({ restoreVisibility: false, log: false });
  state.compare.enabled = true;
  state.compare.blockReason = "";
  state.compare.baselineId = run.id;
  state.compare.variantId = null;
  state.compare.axisSig = getAxisSignature(run.graph);
  state.compare.seriesId = getRunSeriesKey(run);
  state.runs.forEach((item) => {
    item.compareRole = item.id === run.id ? "baseline" : null;
  });
  syncCompareVisibility();
  updateCompareStatusUI();
  renderOverlayList();
  renderRunList();
  renderAll();
}

function makeRunKey(label, inputHash, points) {
  const fallback = hashString(JSON.stringify(points ?? []));
  return `${label}|${inputHash || fallback}`;
}

function createRun({ graph, source, inputs, timeFrames = null, seriesId = null, space2d = null, textDoc = null }) {
  const series = graph?.series?.[0] ?? null;
  const label = graphLabel(graph, series);
  const points = series?.points ?? [];
  const fullPoints = Array.isArray(points) ? points.slice() : [];
  const inputHash = graph?.meta?.source_input_hash ?? "";
  const key = makeRunKey(label, inputHash, points);
  const colors = pickOverlayColors(key);
  return {
    id: `run-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
    label,
    source,
    inputs,
    graph,
    points,
    fullPoints,
    colors,
    visible: true,
    key,
    timeFrames,
    highlight: null,
    seriesId: seriesId ?? series?.id ?? null,
    layerIndex: null,
    opacity: 1,
    compareRole: null,
    space2d,
    textDoc,
  };
}

function cloneGraph(graph) {
  return JSON.parse(JSON.stringify(graph));
}

function createRunsFromGraph(graph, source, inputs, extras = {}) {
  const seriesList = Array.isArray(graph?.series) ? graph.series : [];
  if (seriesList.length <= 1) {
    const run = createRun({
      graph,
      source,
      inputs,
      seriesId: seriesList[0]?.id ?? null,
      space2d: extras.space2d ?? null,
      textDoc: extras.textDoc ?? null,
    });
    return [run];
  }
  return seriesList.map((series) => {
    const nextGraph = cloneGraph(graph);
    nextGraph.series = [series];
    return createRun({
      graph: nextGraph,
      source,
      inputs,
      seriesId: series?.id ?? null,
      space2d: extras.space2d ?? null,
      textDoc: extras.textDoc ?? null,
    });
  });
}

function ensureLayerIndex(run) {
  if (!Number.isFinite(run.layerIndex)) {
    run.layerIndex = state.layerCounter;
    state.layerCounter += 1;
  } else if (run.layerIndex >= state.layerCounter) {
    state.layerCounter = run.layerIndex + 1;
  }
  if (!Number.isFinite(run.opacity)) {
    run.opacity = 1;
  }
}

function ensureLayerIndices(runs) {
  runs.forEach((run) => ensureLayerIndex(run));
}

function sortRunsForRender(runs) {
  return [...runs].sort((a, b) => (a.layerIndex ?? 0) - (b.layerIndex ?? 0));
}

function sortRunsForList(runs) {
  return [...runs].sort((a, b) => (b.layerIndex ?? 0) - (a.layerIndex ?? 0));
}

function moveRunLayer(runId, direction) {
  ensureLayerIndices(state.runs);
  const ordered = sortRunsForRender(state.runs);
  const idx = ordered.findIndex((run) => run.id === runId);
  if (idx === -1) return;
  const nextIdx = idx + direction;
  if (nextIdx < 0 || nextIdx >= ordered.length) return;
  const current = ordered[idx];
  const target = ordered[nextIdx];
  const temp = current.layerIndex;
  current.layerIndex = target.layerIndex;
  target.layerIndex = temp;
  renderOverlayList();
  renderRunList();
  renderAll();
  renderInspector();
}

function setRunLayerIndex(runId, value) {
  if (!Number.isFinite(value)) return;
  const run = state.runs.find((item) => item.id === runId);
  if (!run) return;
  run.layerIndex = Math.round(value);
  ensureLayerIndices(state.runs);
  renderOverlayList();
  renderRunList();
  renderAll();
  renderInspector();
}

function setRunOpacity(runId, value) {
  const run = state.runs.find((item) => item.id === runId);
  if (!run) return;
  const next = Math.min(1, Math.max(0, value));
  run.opacity = next;
  renderOverlayList();
  renderAll();
}

function appendRunPoints(targetRun, incomingRun) {
  const incomingPoints = incomingRun.points ?? [];
  if (!incomingPoints.length) return;
  const combined = (targetRun.points ?? []).concat(incomingPoints);
  targetRun.points = combined;
  if (targetRun.graph?.series?.[0]) {
    targetRun.graph.series[0].points = combined;
  }
  const range = computeRangeFromPoints(combined);
  if (targetRun.graph?.axis) {
    targetRun.graph.axis.x_min = range.xMin;
    targetRun.graph.axis.x_max = range.xMax;
    targetRun.graph.axis.y_min = range.yMin;
    targetRun.graph.axis.y_max = range.yMax;
  }
  if (incomingRun.graph?.meta?.tick !== undefined) {
    targetRun.graph.meta = {
      ...targetRun.graph.meta,
      tick: incomingRun.graph.meta.tick,
    };
  }
}

function findAppendTarget(run) {
  if (!run.graph?.meta || run.graph.meta.update !== "append") return null;
  if (run.seriesId) {
    return state.runs.find((item) => item.seriesId === run.seriesId);
  }
  return state.runs.find((item) => item.label === run.label);
}

function addRun(run, options = {}) {
  const target = state.compare.enabled ? null : findAppendTarget(run);
  if (target) {
    appendRunPoints(target, run);
    if (options.activate !== false) {
      setActiveRun(target.id);
    }
    renderOverlayList();
    renderRunList();
    renderInspector();
    renderAll();
    return;
  }
  ensureLayerIndex(run);
  state.runs.unshift(run);
  if (options.activate !== false) {
    setActiveRun(run.id);
  }
  renderOverlayList();
  renderRunList();
}

function addRuns(runs, options = {}) {
  runs.forEach((run, idx) => {
    addRun(run, { activate: options.activate !== false && idx === 0 });
  });
}

function matchAutoRun(existing, incoming) {
  if (existing?.source?.kind !== "ddn" || incoming?.source?.kind !== "ddn") return false;
  if (existing.seriesId && incoming.seriesId && existing.seriesId !== incoming.seriesId) return false;
  if (existing.source?.text && incoming.source?.text && existing.source.text === incoming.source.text) {
    return true;
  }
  return existing.label === incoming.label;
}

function replaceRunPreserve(existing, incoming) {
  const preserved = {
    id: existing.id,
    layerIndex: existing.layerIndex,
    colors: existing.colors,
    visible: existing.visible,
    opacity: existing.opacity,
    compareRole: existing.compareRole,
  };
  Object.assign(existing, incoming);
  existing.id = preserved.id;
  existing.layerIndex = preserved.layerIndex;
  existing.colors = preserved.colors;
  existing.visible = preserved.visible;
  existing.opacity = preserved.opacity;
  existing.compareRole = preserved.compareRole;
}

function addRunsAutoReplace(runs) {
  if (!runs.length) return;
  const matched = new Set();
  runs.forEach((incoming) => {
    const match = state.runs.find((run) => !matched.has(run.id) && matchAutoRun(run, incoming));
    if (match) {
      matched.add(match.id);
      replaceRunPreserve(match, incoming);
      return;
    }
    ensureLayerIndex(incoming);
    state.runs.unshift(incoming);
    if (!state.activeRunId) state.activeRunId = incoming.id;
  });
  renderOverlayList();
  renderRunList();
  renderAll();
  renderInspector();
  renderTimelineBar();
}

function setActiveRun(runId) {
  state.activeRunId = runId;
  const run = getActiveRun();
  if (run?.timeFrames?.length) {
    applyTimeFrame();
    renderRunList();
    renderTimelineBar();
    return;
  }
  if (run?.space2d) {
    state.space2d = run.space2d;
    state.space2dRange = computeSpace2dRange(run.space2d);
  }
  if (run?.textDoc) {
    state.textDoc = run.textDoc;
  }
  renderRunList();
  renderInspector();
  renderAll();
  renderTimelineBar();
}

function getActiveRun() {
  return state.runs.find((run) => run.id === state.activeRunId) ?? null;
}

function getVisibleRuns() {
  ensureLayerIndices(state.runs);
  if (state.soloRunId) {
    const solo = state.runs.find((run) => run.id === state.soloRunId);
    if (solo && solo.visible !== false) {
      return sortRunsForRender([solo]);
    }
    state.soloRunId = null;
  }
  return sortRunsForRender(state.runs.filter((run) => run.visible));
}

function removeRun(runId) {
  state.runs = state.runs.filter((run) => run.id !== runId);
  if (state.compare.baselineId === runId) {
    stopCompareSequence({ restoreVisibility: false, log: false });
    state.compare.baselineId = null;
    state.compare.axisSig = null;
    state.compare.seriesId = null;
  }
  if (state.compare.variantId === runId) {
    stopCompareSequence({ restoreVisibility: false, log: false });
    state.compare.variantId = null;
  }
  if (state.activeRunId === runId) {
    const next = sortRunsForList(state.runs)[0];
    state.activeRunId = next?.id ?? null;
  }
  if (state.soloRunId === runId) {
    state.soloRunId = null;
  }
  if (!state.runs.length) {
    log(state.controls.autoRun ? "실행 목록이 비었습니다." : "실행 목록이 비었습니다. 자동 실행이 꺼져 있습니다.");
  }
  updateCompareStatusUI();
  renderOverlayList();
  renderRunList();
  renderInspector();
  renderAll();
  renderTimelineBar();
}

function clearRuns() {
  stopCompareSequence({ restoreVisibility: false, log: false });
  state.runs = [];
  state.activeRunId = null;
  state.soloRunId = null;
  state.layerCounter = 0;
  state.compare.baselineId = null;
  state.compare.variantId = null;
  state.compare.axisSig = null;
  state.compare.seriesId = null;
  state.compare.blockReason = "";
  state.compare.savedVisibility = {};
  log(state.controls.autoRun ? "실행 목록을 비웠습니다." : "실행 목록을 비웠습니다. 자동 실행이 꺼져 있습니다.");
  renderOverlayList();
  renderRunList();
  renderInspector();
  renderAll();
  renderTimelineBar();
}

function computeRangeFromPoints(points) {
  if (!points || points.length === 0) {
    return { xMin: 0, xMax: 1, yMin: 0, yMax: 1 };
  }
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const xMin = Math.min(...xs);
  const xMax = Math.max(...xs);
  const yMin = Math.min(...ys);
  const yMax = Math.max(...ys);
  return {
    xMin,
    xMax: xMax === xMin ? xMin + 1 : xMax,
    yMin,
    yMax: yMax === yMin ? yMin + 1 : yMax,
  };
}

function applyPanZoom(range, panX, panY, zoom) {
  const zoomSafe = Number.isFinite(zoom) && zoom > 0 ? zoom : 1;
  const xMid = (range.xMin + range.xMax) / 2 + panX;
  const yMid = (range.yMin + range.yMax) / 2 + panY;
  const xHalf = (range.xMax - range.xMin) / 2 / zoomSafe;
  const yHalf = (range.yMax - range.yMin) / 2 / zoomSafe;
  return {
    xMin: xMid - xHalf,
    xMax: xMid + xHalf,
    yMin: yMid - yHalf,
    yMax: yMid + yHalf,
  };
}

function collectPointsFromRuns(runs) {
  return runs.flatMap((run) => run.fullPoints ?? run.points ?? []);
}

function collectPointsFromShapes(shapes) {
  if (!Array.isArray(shapes)) return [];
  const points = [];
  shapes.forEach((shape) => {
    if (!shape) return;
    const kind = String(shape.kind ?? "").toLowerCase();
    if (kind === "line") {
      if (Number.isFinite(shape.x1) && Number.isFinite(shape.y1)) {
        points.push({ x: shape.x1, y: shape.y1 });
      }
      if (Number.isFinite(shape.x2) && Number.isFinite(shape.y2)) {
        points.push({ x: shape.x2, y: shape.y2 });
      }
      return;
    }
    if (kind === "circle") {
      if (Number.isFinite(shape.x) && Number.isFinite(shape.y) && Number.isFinite(shape.r)) {
        points.push({ x: shape.x - shape.r, y: shape.y - shape.r });
        points.push({ x: shape.x + shape.r, y: shape.y + shape.r });
      }
      return;
    }
    if (kind === "point") {
      if (Number.isFinite(shape.x) && Number.isFinite(shape.y)) {
        const size = Number.isFinite(shape.size) ? shape.size : 0;
        points.push({ x: shape.x - size, y: shape.y - size });
        points.push({ x: shape.x + size, y: shape.y + size });
      }
      return;
    }
    if (Number.isFinite(shape.x) && Number.isFinite(shape.y)) {
      points.push({ x: shape.x, y: shape.y });
    }
  });
  return points;
}

function normalizeDrawKind(item) {
  const raw = item?.kind ?? item?.shape ?? item?.type ?? item?.도형 ?? item?.형태;
  const key = String(raw ?? "").toLowerCase();
  if (["line", "선", "segment"].includes(key)) return "line";
  if (["circle", "원"].includes(key)) return "circle";
  if (["point", "점"].includes(key)) return "point";
  if (["polyline", "선들", "lines"].includes(key)) return "polyline";
  if (["rect", "rectangle", "사각형"].includes(key)) return "rect";
  if (["polygon", "다각형"].includes(key)) return "polygon";
  if (["text", "글", "label"].includes(key)) return "text";
  if (["arrow", "화살표"].includes(key)) return "arrow";
  return key;
}

function readPointCandidate(value) {
  if (!value) return null;
  if (Array.isArray(value) && value.length >= 2) {
    const x = Number(value[0]);
    const y = Number(value[1]);
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
  }
  if (typeof value === "object") {
    const x = Number(value.x ?? value.cx ?? value[0]);
    const y = Number(value.y ?? value.cy ?? value[1]);
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
  }
  return null;
}

function readPointFromKeys(item, keys) {
  for (const key of keys) {
    if (key in item) {
      const point = readPointCandidate(item[key]);
      if (point) return point;
    }
  }
  return null;
}

function normalizePointList(raw) {
  if (!Array.isArray(raw)) return [];
  return raw
    .map((entry) => readPointCandidate(entry))
    .filter((point) => point && Number.isFinite(point.x) && Number.isFinite(point.y));
}

function resolveLineEndpoints(item) {
  const x1 = Number(item.x1);
  const y1 = Number(item.y1);
  const x2 = Number(item.x2);
  const y2 = Number(item.y2);
  if ([x1, y1, x2, y2].every(Number.isFinite)) {
    return { x1, y1, x2, y2 };
  }
  const start = readPointFromKeys(item, ["start", "from", "시작", "source", "a"]);
  const end = readPointFromKeys(item, ["end", "to", "끝", "target", "b"]);
  if (start && end) {
    return { x1: start.x, y1: start.y, x2: end.x, y2: end.y };
  }
  return null;
}

function resolveCircle(item) {
  const cx = Number(item.cx ?? item.x);
  const cy = Number(item.cy ?? item.y);
  const r = Number(item.r ?? item.radius ?? item.반지름);
  if ([cx, cy, r].every(Number.isFinite)) {
    return { x: cx, y: cy, r };
  }
  const center = readPointFromKeys(item, ["center", "중심", "position", "위치"]);
  if (center && Number.isFinite(r)) {
    return { x: center.x, y: center.y, r };
  }
  return null;
}

function resolvePoint(item) {
  const x = Number(item.x);
  const y = Number(item.y);
  if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
  const pos = readPointFromKeys(item, ["position", "pos", "위치"]);
  if (pos) return pos;
  return null;
}

function resolveRect(item) {
  const x1 = Number(item.x1);
  const y1 = Number(item.y1);
  const x2 = Number(item.x2);
  const y2 = Number(item.y2);
  if ([x1, y1, x2, y2].every(Number.isFinite)) {
    return { x1, y1, x2, y2 };
  }
  const width = Number(item.width ?? item.w ?? item.너비);
  const height = Number(item.height ?? item.h ?? item.높이);
  if (!Number.isFinite(width) || !Number.isFinite(height)) return null;
  const anchor = String(item.anchor ?? item.origin ?? "").toLowerCase();
  const center = anchor === "center" || anchor === "mid" || item.center === true || item.중심 === true;
  const x = Number(item.x);
  const y = Number(item.y);
  if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
  if (center) {
    return {
      x1: x - width / 2,
      y1: y - height / 2,
      x2: x + width / 2,
      y2: y + height / 2,
    };
  }
  return { x1: x, y1: y, x2: x + width, y2: y + height };
}

function collectPointsFromDrawlist(items) {
  if (!Array.isArray(items)) return [];
  const points = [];
  items.forEach((item) => {
    if (!item || typeof item !== "object") return;
    const kind = normalizeDrawKind(item);
    if (kind === "line" || kind === "arrow") {
      const line = resolveLineEndpoints(item);
      if (!line) return;
      points.push({ x: line.x1, y: line.y1 });
      points.push({ x: line.x2, y: line.y2 });
      return;
    }
    if (kind === "circle") {
      const circle = resolveCircle(item);
      if (!circle) return;
      points.push({ x: circle.x - circle.r, y: circle.y - circle.r });
      points.push({ x: circle.x + circle.r, y: circle.y + circle.r });
      return;
    }
    if (kind === "point") {
      const point = resolvePoint(item);
      if (!point) return;
      const size = Number(item.size ?? item.크기 ?? 0);
      if (Number.isFinite(size) && size > 0) {
        points.push({ x: point.x - size, y: point.y - size });
        points.push({ x: point.x + size, y: point.y + size });
      } else {
        points.push(point);
      }
      return;
    }
    if (kind === "polyline" || kind === "polygon") {
      const list = normalizePointList(item.points ?? item.coords ?? item.vertices ?? item.점들 ?? item.좌표);
      points.push(...list);
      return;
    }
    if (kind === "rect") {
      const rect = resolveRect(item);
      if (!rect) return;
      points.push({ x: rect.x1, y: rect.y1 });
      points.push({ x: rect.x2, y: rect.y2 });
      return;
    }
    if (kind === "text") {
      const pos = resolvePoint(item);
      if (pos) points.push(pos);
      return;
    }
  });
  return points;
}

function computeSpace2dRange(space2d) {
  if (!space2d) return null;
  const points = Array.isArray(space2d.points) ? space2d.points : [];
  const shapes = Array.isArray(space2d.shapes) ? space2d.shapes : [];
  const drawItems = extractDrawlistItems(space2d.drawlist);
  const shapePoints = collectPointsFromShapes(shapes);
  const drawPoints = collectPointsFromDrawlist(drawItems);
  const rangePoints = points.concat(shapePoints, drawPoints);
  if (!rangePoints.length) return null;
  return computeRangeFromPoints(rangePoints);
}

function computeSpace2dRangeFromFrames(frames) {
  if (!Array.isArray(frames) || frames.length === 0) return null;
  const points = [];
  frames.forEach((frame) => {
    if (!frame?.space2d) return;
    const range = computeSpace2dRange(frame.space2d);
    if (!range) return;
    points.push({ x: range.xMin, y: range.yMin });
    points.push({ x: range.xMax, y: range.yMax });
  });
  if (!points.length) return null;
  return computeRangeFromPoints(points);
}

function getViewToggleState() {
  return {
    showGrid: $("toggle-grid")?.checked ?? true,
    showAxis: $("toggle-axis")?.checked ?? true,
  };
}

function getViewConfigFromControls(runs) {
  const auto = $("view-auto").checked;
  const panX = readNumber("pan-x", 0) ?? 0;
  const panY = readNumber("pan-y", 0) ?? 0;
  const zoom = readNumber("zoom", 1) ?? 1;
  const allPoints = collectPointsFromRuns(runs);
  let baseRange = null;
  let usedAuto = auto;

  if (!auto) {
    const xMin = readNumber("view-x-min", null);
    const xMax = readNumber("view-x-max", null);
    const yMin = readNumber("view-y-min", null);
    const yMax = readNumber("view-y-max", null);
    if ([xMin, xMax, yMin, yMax].some((v) => v === null)) {
      usedAuto = true;
    } else if (xMax <= xMin || yMax <= yMin) {
      throw new Error("표시 범위는 최소 < 최대 여야 합니다.");
    } else {
      baseRange = { xMin, xMax, yMin, yMax };
    }
  }

  if (usedAuto || !baseRange) {
    baseRange = computeRangeFromPoints(allPoints);
  }

  const range = applyPanZoom(baseRange, panX, panY, zoom);
  const toggles = getViewToggleState();
  return {
    auto: usedAuto,
    panX,
    panY,
    zoom,
    range,
    showGrid: toggles.showGrid,
    showAxis: toggles.showAxis,
  };
}

function getSpace2dViewConfigFromControls() {
  const auto = $("space2d-auto")?.checked ?? true;
  const panX = readNumber("space2d-pan-x", 0) ?? 0;
  const panY = readNumber("space2d-pan-y", 0) ?? 0;
  const zoom = readNumber("space2d-zoom", 1) ?? 1;
  let range = null;
  if (!auto) {
    const xMin = readNumber("space2d-x-min", null);
    const xMax = readNumber("space2d-x-max", null);
    const yMin = readNumber("space2d-y-min", null);
    const yMax = readNumber("space2d-y-max", null);
    if ([xMin, xMax, yMin, yMax].every(Number.isFinite)) {
      if (xMax <= xMin || yMax <= yMin) {
        throw new Error("2D 표시 범위는 최소 < 최대 여야 합니다.");
      }
      range = { xMin, xMax, yMin, yMax };
    }
  }
  return { auto, range, panX, panY, zoom };
}

function getViewConfigFromData(graph, runs) {
  const allPoints = collectPointsFromRuns(runs);
  const baseRange =
    graph?.view && Number.isFinite(graph.view.x_min)
      ? {
          xMin: graph.view.x_min,
          xMax: graph.view.x_max,
          yMin: graph.view.y_min,
          yMax: graph.view.y_max,
        }
      : graph?.axis && Number.isFinite(graph.axis.x_min)
        ? {
            xMin: graph.axis.x_min,
            xMax: graph.axis.x_max,
            yMin: graph.axis.y_min,
            yMax: graph.axis.y_max,
          }
        : computeRangeFromPoints(allPoints);

  const panX = Number.isFinite(graph?.view?.pan_x) ? graph.view.pan_x : 0;
  const panY = Number.isFinite(graph?.view?.pan_y) ? graph.view.pan_y : 0;
  const zoom = Number.isFinite(graph?.view?.zoom) ? graph.view.zoom : 1;
  const range = applyPanZoom(baseRange, panX, panY, zoom);
  const toggles = getViewToggleState();
  return {
    auto: graph?.view?.auto ?? graph?.view == null,
    panX,
    panY,
    zoom,
    range,
    showGrid: toggles.showGrid,
    showAxis: toggles.showAxis,
  };
}

function applyViewControlsFromGraph(graph, viewConfig) {
  const view = graph?.view;
  $("view-auto").checked = view?.auto ?? viewConfig.auto ?? true;
  $("view-x-min").value = view?.x_min ?? "";
  $("view-x-max").value = view?.x_max ?? "";
  $("view-y-min").value = view?.y_min ?? "";
  $("view-y-max").value = view?.y_max ?? "";
  $("pan-x").value = view?.pan_x ?? viewConfig.panX ?? 0;
  $("pan-y").value = view?.pan_y ?? viewConfig.panY ?? 0;
  $("zoom").value = view?.zoom ?? viewConfig.zoom ?? 1;
  if (viewConfig.showGrid !== undefined) {
    $("toggle-grid").checked = viewConfig.showGrid;
  }
  if (viewConfig.showAxis !== undefined) {
    $("toggle-axis").checked = viewConfig.showAxis;
  }
  updateViewInputsEnabled();

  if (graph?.meta?.update) $("update-mode").value = graph.meta.update;
  if (Number.isFinite(graph?.meta?.tick)) {
    $("update-tick").value = graph.meta.tick;
  } else {
    $("update-tick").value = "";
  }
}

function updateViewInputsEnabled() {
  const disabled = $("view-auto").checked;
  ["view-x-min", "view-x-max", "view-y-min", "view-y-max"].forEach((id) => {
    $(id).disabled = disabled;
  });
}

function updateSpace2dInputsEnabled() {
  const disabled = $("space2d-auto")?.checked ?? true;
  ["space2d-x-min", "space2d-x-max", "space2d-y-min", "space2d-y-max"].forEach((id) => {
    const input = $(id);
    if (input) input.disabled = disabled;
  });
}

function syncSpace2dViewControlValues() {
  const view = state.space2dView ?? {};
  if ($("space2d-auto")) $("space2d-auto").checked = view.auto ?? true;
  if ($("space2d-x-min")) $("space2d-x-min").value = view.range?.xMin ?? "";
  if ($("space2d-x-max")) $("space2d-x-max").value = view.range?.xMax ?? "";
  if ($("space2d-y-min")) $("space2d-y-min").value = view.range?.yMin ?? "";
  if ($("space2d-y-max")) $("space2d-y-max").value = view.range?.yMax ?? "";
  if ($("space2d-pan-x")) $("space2d-pan-x").value = view.panX ?? 0;
  if ($("space2d-pan-y")) $("space2d-pan-y").value = view.panY ?? 0;
  if ($("space2d-zoom")) $("space2d-zoom").value = view.zoom ?? 1;
  updateSpace2dInputsEnabled();
}

function resetSpace2dView(options = {}) {
  const forceAuto = Boolean(options.forceAutoFit);
  state.space2dView.panX = 0;
  state.space2dView.panY = 0;
  state.space2dView.zoom = 1;
  state.space2dView.dragging = false;
  if (forceAuto) {
    state.space2dView.auto = true;
    state.space2dView.range = null;
  } else if (state.space2dView.auto) {
    state.space2dView.range = null;
  }
  syncSpace2dViewControlValues();
}

function slicePointsByProgress(points, progress) {
  if (!Array.isArray(points) || !points.length) return points ?? [];
  if (!Number.isFinite(progress)) return points;
  if (progress >= 1) return points;
  if (progress <= 0) return [];
  const count = Math.max(0, Math.min(points.length, Math.round(progress * points.length)));
  return points.slice(0, count);
}
function renderGraphCanvas(runs, viewRange, highlightPoints = null) {
  const canvas = $("canvas");
  const ctx = canvas.getContext("2d");
  const w = canvas.width;
  const h = canvas.height;
  const useRuns = runs.length > 0;

  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = "#0f172a";
  ctx.fillRect(0, 0, w, h);

  const allPoints = collectPointsFromRuns(runs);
  if (!allPoints.length) return;

  const pad = 40;
  const range = viewRange ?? computeRangeFromPoints(allPoints);
  const xMin = range.xMin;
  const xMax = range.xMax;
  const yMin = range.yMin;
  const yMax = range.yMax;
  const xRange = xMax - xMin || 1;
  const yRange = yMax - yMin || 1;
  const mapX = (x) => pad + ((x - xMin) / xRange) * (w - pad * 2);
  const mapY = (y) => h - pad - ((y - yMin) / yRange) * (h - pad * 2);

  const viewConfig = state.viewConfig ?? {};
  const showGrid = viewConfig.showGrid ?? true;
  const showAxis = viewConfig.showAxis ?? true;

  if (showGrid) {
    const gridCount = 5;
    ctx.strokeStyle = "rgba(255,255,255,0.15)";
    ctx.lineWidth = 1;
    for (let i = 0; i < gridCount; i++) {
      const t = i / (gridCount - 1);
      const gx = pad + t * (w - pad * 2);
      const gy = pad + t * (h - pad * 2);
      ctx.beginPath();
      ctx.moveTo(gx, pad);
      ctx.lineTo(gx, h - pad);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(pad, gy);
      ctx.lineTo(w - pad, gy);
      ctx.stroke();
    }

    const formatTick = (value) => {
      if (!Number.isFinite(value)) return "0";
      if (Number.isInteger(value)) return value.toString();
      return Number(value.toFixed(3)).toString();
    };

    ctx.fillStyle = "rgba(255,255,255,0.6)";
    ctx.font = "12px " + getComputedStyle(document.body).fontFamily;
    for (let i = 0; i < gridCount; i++) {
      const t = i / (gridCount - 1);
      const xVal = xMin + t * xRange;
      const yVal = yMax - t * yRange;
      const gx = pad + t * (w - pad * 2);
      const gy = pad + t * (h - pad * 2);
      ctx.fillText(formatTick(xVal), gx - 8, h - 10);
      ctx.fillText(formatTick(yVal), 6, gy + 4);
    }
  }

  if (showAxis) {
    ctx.strokeStyle = "rgba(255,255,255,0.35)";
    ctx.lineWidth = 1.5;
    if (xMin <= 0 && xMax >= 0) {
      const xZero = mapX(0);
      ctx.beginPath();
      ctx.moveTo(xZero, pad);
      ctx.lineTo(xZero, h - pad);
      ctx.stroke();
    }
    if (yMin <= 0 && yMax >= 0) {
      const yZero = mapY(0);
      ctx.beginPath();
      ctx.moveTo(pad, yZero);
      ctx.lineTo(w - pad, yZero);
      ctx.stroke();
    }
  }

  const orderedRuns = sortRunsForRender(runs);
  orderedRuns.forEach((run) => {
    const basePoints = run.fullPoints ?? run.points ?? [];
    if (!basePoints.length) return;
    const isHover = run.id === state.hoverRunId;
    const isActive = run.id === state.activeRunId;
    const lineWidth = isHover ? 3 : isActive ? 2.5 : 2;
    const baseOpacity = Number.isFinite(run.opacity) ? Math.min(1, Math.max(0, run.opacity)) : 1;
    const sceneFactor = useRuns ? getSceneRunFactor(run) : 1;
    const opacity = Math.min(1, Math.max(0, baseOpacity * sceneFactor));
    const progress = useRuns ? getSceneProgressForRun(run) : 1;
    const points = slicePointsByProgress(basePoints, progress);
    if (!points.length) return;
    ctx.save();
    ctx.globalAlpha = opacity;
    ctx.strokeStyle = run.colors?.line ?? "#8ecae6";
    ctx.lineWidth = lineWidth;
    if (run.compareRole === "variant") {
      ctx.setLineDash([6, 4]);
    }
    ctx.beginPath();
    points.forEach((p, i) => {
      const x = mapX(p.x);
      const y = mapY(p.y);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.stroke();

    ctx.fillStyle = run.colors?.point ?? "#219ebc";
    points.forEach((p) => {
      const x = mapX(p.x);
      const y = mapY(p.y);
      ctx.beginPath();
      ctx.arc(x, y, isHover ? 4 : 3, 0, Math.PI * 2);
      ctx.fill();
    });
    ctx.restore();
  });

  if (highlightPoints && highlightPoints.length) {
    ctx.strokeStyle = "#f4a261";
    ctx.lineWidth = 2.5;
    ctx.beginPath();
    highlightPoints.forEach((p, i) => {
      const x = mapX(p.x);
      const y = mapY(p.y);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.stroke();

    ctx.fillStyle = "#f4a261";
    highlightPoints.forEach((p) => {
      const x = mapX(p.x);
      const y = mapY(p.y);
      ctx.beginPath();
      ctx.arc(x, y, 4.5, 0, Math.PI * 2);
      ctx.fill();
    });
  }
}

function formatTableValue(value) {
  if (value === null || value === undefined) return "-";
  if (typeof value === "number" && Number.isFinite(value)) {
    const precision = Number.isFinite(state.tableView.precision) ? state.tableView.precision : 3;
    return Number(value.toFixed(precision)).toString();
  }
  if (typeof value === "boolean") return value ? "true" : "false";
  return String(value);
}

function buildTableFromGraphPoints(points) {
  return {
    schema: "seamgrim.table.v0",
    columns: [
      { key: "idx", label: "#", type: "number" },
      { key: "x", label: "x", type: "number" },
      { key: "y", label: "y", type: "number" },
    ],
    rows: points.map((p, idx) => ({ idx: idx + 1, x: p.x, y: p.y })),
  };
}

function renderTableView() {
  const tableEl = $("data-table");
  const thead = tableEl.querySelector("thead");
  const tbody = tableEl.querySelector("tbody");
  thead.innerHTML = "";
  tbody.innerHTML = "";

  let tableData = state.table;
  if (!tableData) {
    const activeRun = getActiveRun();
    if (activeRun?.points?.length) {
      tableData = buildTableFromGraphPoints(activeRun.points);
    }
  }

  if (!tableData) return;

  const columns = tableData.columns ?? [];
  const rows = tableData.rows ?? [];
  const headerRow = document.createElement("tr");
  columns.forEach((col) => {
    const th = document.createElement("th");
    th.textContent = col.label ?? col.key;
    headerRow.appendChild(th);
  });
  thead.appendChild(headerRow);

  const pageSize = Math.max(1, Number(state.tableView.pageSize) || 50);
  const pageCount = Math.max(1, Math.ceil(rows.length / pageSize));
  if (state.tableView.page >= pageCount) state.tableView.page = pageCount - 1;
  const start = state.tableView.page * pageSize;
  const end = Math.min(rows.length, start + pageSize);

  rows.slice(start, end).forEach((row) => {
    const tr = document.createElement("tr");
    columns.forEach((col) => {
      const td = document.createElement("td");
      td.textContent = formatTableValue(row[col.key]);
      tr.appendChild(td);
    });
    tbody.appendChild(tr);
  });

  $("table-page-info").textContent = `${state.tableView.page + 1} / ${pageCount}`;
}

function renderTextView() {
  const el = $("text-content");
  if (!el) return;
  if (!state.textDoc) {
    el.innerHTML = "문서를 불러오면 여기에 표시됩니다.";
    renderCaptionOverlay();
    return;
  }
  if (state.textDoc.format === "plain") {
    el.textContent = state.textDoc.content ?? "";
    applySceneTextHighlights(el);
    renderCaptionOverlay();
    return;
  }
  el.innerHTML = renderMarkdownSafe(state.textDoc.content ?? "");
  applySceneTextHighlights(el);
  renderCaptionOverlay();
}

function stripMarkdownToText(text) {
  if (!text) return "";
  return text
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`[^`]*`/g, " ")
    .replace(/!\[[^\]]*]\([^)]*\)/g, " ")
    .replace(/\[([^\]]+)]\([^)]*\)/g, "$1")
    .replace(/[#>*_\-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function buildCaptionSnippet(doc) {
  if (!doc) return "";
  const raw = doc.format === "plain" ? doc.content ?? "" : stripMarkdownToText(doc.content ?? "");
  if (!raw) return "";
  const snippet = raw.length > 140 ? `${raw.slice(0, 140)}…` : raw;
  return snippet;
}

function buildSceneCaptionSummary(segment) {
  const blocks = Array.isArray(segment?.caption_blocks) ? segment.caption_blocks : [];
  if (blocks.length) {
    const inline = blocks.map((block) => block.text).join(" ");
    const detailHtml = blocks
      .map(
        (block) =>
          `<div class="scene-caption-item"><span class="scene-caption-kind">${escapeHtml(
            block.kind ?? "자막",
          )}</span> ${escapeHtml(block.text)}</div>`,
      )
      .join("");
    return { inline, detailHtml };
  }
  const captions = Array.isArray(segment?.captions) ? segment.captions : [];
  if (captions.length) {
    const inline = captions.join(" ");
    const detailHtml = captions
      .map((text) => `<div class="scene-caption-item">${escapeHtml(text)}</div>`)
      .join("");
    return { inline, detailHtml };
  }
  return { inline: "", detailHtml: "" };
}

function applySceneTextHighlights(root) {
  const segment = getActiveSceneSegment();
  const tokenSet = getSceneEmphasisTokenSet(segment);
  if (!root || !tokenSet || tokenSet.size === 0) return;
  const tokens = Array.from(tokenSet).filter(Boolean);
  if (!tokens.length) return;
  const pattern = new RegExp(tokens.map(escapeRegExp).join("|"), "gi");
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode: (node) => {
      if (!node?.textContent?.trim()) return NodeFilter.FILTER_REJECT;
      const parent = node.parentElement;
      if (!parent) return NodeFilter.FILTER_REJECT;
      const tag = parent.tagName;
      if (["SCRIPT", "STYLE", "CODE", "PRE", "A"].includes(tag)) {
        return NodeFilter.FILTER_REJECT;
      }
      if (parent.classList.contains("scene-token")) {
        return NodeFilter.FILTER_REJECT;
      }
      return NodeFilter.FILTER_ACCEPT;
    },
  });
  const nodes = [];
  while (walker.nextNode()) nodes.push(walker.currentNode);
  nodes.forEach((node) => {
    const text = node.textContent;
    if (!text || !pattern.test(text)) return;
    pattern.lastIndex = 0;
    const frag = document.createDocumentFragment();
    let lastIndex = 0;
    let match;
    while ((match = pattern.exec(text)) !== null) {
      if (match.index > lastIndex) {
        frag.appendChild(document.createTextNode(text.slice(lastIndex, match.index)));
      }
      const span = document.createElement("span");
      span.className = "scene-token";
      span.textContent = match[0];
      frag.appendChild(span);
      lastIndex = match.index + match[0].length;
    }
    if (lastIndex < text.length) {
      frag.appendChild(document.createTextNode(text.slice(lastIndex)));
    }
    node.parentNode.replaceChild(frag, node);
  });
}

function renderCaptionOverlay() {
  const graphCaption = $("graph-caption");
  const spaceCaption = $("space2d-caption");
  const captionBox = $("view-caption-box");
  const captionBody = $("view-caption-body");
  const sceneSegment = getActiveSceneSegment();
  const sceneSummary = buildSceneCaptionSummary(sceneSegment);
  const sceneText = sceneSummary.inline;
  const baseDoc = state.textDoc?.content ?? "";
  if (!sceneText && !baseDoc) {
    if (graphCaption) graphCaption.classList.add("hidden");
    if (spaceCaption) spaceCaption.classList.add("hidden");
    if (captionBox) captionBox.classList.add("hidden");
    if (captionBody) captionBody.textContent = "";
    return;
  }
  const snippet = sceneText || buildCaptionSnippet(state.textDoc);
  if (graphCaption) {
    graphCaption.textContent = snippet;
    graphCaption.classList.toggle("hidden", !snippet);
  }
  if (spaceCaption) {
    spaceCaption.textContent = snippet;
    spaceCaption.classList.toggle("hidden", !snippet);
  }
  if (captionBox) captionBox.classList.remove("hidden");
  if (captionBody) {
    const banner = sceneText
      ? `<div class="caption-now">현재: ${escapeHtml(sceneText)}</div>`
      : "";
    const detail = sceneSummary.detailHtml
      ? `<div class="scene-caption-stack">${sceneSummary.detailHtml}</div>`
      : "";
    if (state.textDoc?.format === "plain") {
      captionBody.innerHTML = `${banner}${detail}${escapeHtml(state.textDoc.content ?? "")}`;
    } else if (state.textDoc?.content) {
      captionBody.innerHTML = `${banner}${detail}${renderMarkdownSafe(state.textDoc.content ?? "")}`;
    } else {
      captionBody.innerHTML = `${banner}${detail}`;
    }
  }

  const viewFocusTargets = new Set();
  if (sceneSegment?.views?.length) {
    sceneSegment.views.forEach((view) => {
      const key = normalizeSceneViewKey(view);
      if (key) viewFocusTargets.add(key);
    });
  }
  const hasFocusTargets = viewFocusTargets.size > 0;
  ["view-graph", "view-2d", "view-text", "view-table", "view-structure"].forEach((id) => {
    const pane = $(id);
    if (!pane) return;
    const focused = viewFocusTargets.has(id);
    pane.classList.toggle("view-focus", focused);
    pane.classList.toggle("view-dim", hasFocusTargets && !focused);
  });
}

function computeStructureLayout(structure, layoutType) {
  const nodes = structure.nodes ?? [];
  if (!nodes.length) return [];
  const allHaveXY = nodes.every((node) => Number.isFinite(node.x) && Number.isFinite(node.y));
  if (allHaveXY) {
    const xs = nodes.map((node) => node.x);
    const ys = nodes.map((node) => node.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    const rangeX = maxX - minX || 1;
    const rangeY = maxY - minY || 1;
    return nodes.map((node) => ({
      ...node,
      _pos: {
        x: (node.x - minX) / rangeX,
        y: (node.y - minY) / rangeY,
      },
    }));
  }

  if (layoutType === "dag") {
    const edges = structure.edges ?? [];
    const indegree = new Map();
    const out = new Map();
    nodes.forEach((node) => {
      indegree.set(node.id, 0);
      out.set(node.id, []);
    });
    edges.forEach((edge) => {
      if (!indegree.has(edge.to) || !out.has(edge.from)) return;
      indegree.set(edge.to, (indegree.get(edge.to) || 0) + 1);
      out.get(edge.from).push(edge.to);
    });
    const queue = [];
    indegree.forEach((count, id) => {
      if (count === 0) queue.push(id);
    });
    const order = [];
    while (queue.length) {
      const id = queue.shift();
      order.push(id);
      out.get(id).forEach((next) => {
        indegree.set(next, indegree.get(next) - 1);
        if (indegree.get(next) === 0) queue.push(next);
      });
    }
    if (order.length !== nodes.length) {
      layoutType = "circle";
    } else {
      const level = new Map();
      order.forEach((id) => {
        const outs = out.get(id) ?? [];
        outs.forEach((next) => {
          const nextLevel = (level.get(id) ?? 0) + 1;
          level.set(next, Math.max(level.get(next) ?? 0, nextLevel));
        });
      });
      const maxLevel = Math.max(0, ...Array.from(level.values()));
      const groups = new Map();
      nodes.forEach((node) => {
        const lv = level.get(node.id) ?? 0;
        if (!groups.has(lv)) groups.set(lv, []);
        groups.get(lv).push(node);
      });
      return nodes.map((node) => {
        const lv = level.get(node.id) ?? 0;
        const group = groups.get(lv) ?? [node];
        const idx = group.findIndex((n) => n.id === node.id);
        const x = group.length === 1 ? 0.5 : idx / (group.length - 1);
        const y = maxLevel === 0 ? 0.5 : lv / maxLevel;
        return { ...node, _pos: { x, y } };
      });
    }
  }

  const count = nodes.length;
  return nodes.map((node, idx) => {
    const angle = (Math.PI * 2 * idx) / count;
    return {
      ...node,
      _pos: {
        x: 0.5 + 0.35 * Math.cos(angle),
        y: 0.5 + 0.35 * Math.sin(angle),
      },
    };
  });
}

function render2dView() {
  const canvas = $("canvas-2d");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  const w = canvas.width;
  const h = canvas.height;
  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = "#0f172a";
  ctx.fillRect(0, 0, w, h);

  const runs = getVisibleRuns();
  const fallbackPoints = state.space2d?.points ?? [];
  const drawItems = extractDrawlistItems(state.space2d?.drawlist);
  const hasDrawlist = drawItems.length > 0;
  const shapeList = hasDrawlist ? [] : state.space2d?.shapes ?? [];
  const hasSpace2d = fallbackPoints.length > 0 || shapeList.length > 0 || hasDrawlist;
  const useRuns = runs.length > 0 && !hasSpace2d;
  const allPoints = useRuns ? collectPointsFromRuns(runs) : fallbackPoints;
  const shapePoints = useRuns ? [] : collectPointsFromShapes(shapeList);
  const drawPoints = useRuns ? [] : collectPointsFromDrawlist(drawItems);
  const rangePoints = allPoints.concat(shapePoints, drawPoints);
  const summary = $("space2d-summary");
  if (!rangePoints.length) {
    if (summary) summary.textContent = "2D 데이터: -";
    state.space2dView.lastRender = null;
    return;
  }
  if (summary) {
    const label = useRuns
      ? (getActiveRun()?.label ?? "run")
      : (state.space2d?.meta?.title ?? "space2d");
    const stats = [`points ${allPoints.length}`];
    if (shapeList.length) stats.push(`shapes ${shapeList.length}`);
    if (drawItems.length) stats.push(`draw ${drawItems.length}`);
    summary.textContent = `2D 데이터: ${label} · ${stats.join(" / ")}`;
  }

  const pad = 40;
  let range = computeRangeFromPoints(rangePoints);
  const camera = extractSpace2dCamera(state.space2d);
  if (camera) {
    const xMin = Number(camera.x_min ?? camera.xMin);
    const xMax = Number(camera.x_max ?? camera.xMax);
    const yMin = Number(camera.y_min ?? camera.yMin);
    const yMax = Number(camera.y_max ?? camera.yMax);
    if ([xMin, xMax, yMin, yMax].every(Number.isFinite)) {
      range = { xMin, xMax, yMin, yMax };
    }
    const panX = Number(camera.pan_x ?? camera.panX ?? 0);
    const panY = Number(camera.pan_y ?? camera.panY ?? 0);
    const zoom = Number(camera.zoom ?? 1);
    range = applyPanZoom(range, panX, panY, zoom);
  }
  const fallbackRange = state.space2dRange;
  if (!camera && !useRuns && fallbackRange) {
    range = { ...fallbackRange };
  }
  if (state.space2dView) {
    if (!state.space2dView.auto && state.space2dView.range) {
      range = { ...state.space2dView.range };
    }
    range = applyPanZoom(
      range,
      state.space2dView.panX ?? 0,
      state.space2dView.panY ?? 0,
      state.space2dView.zoom ?? 1,
    );
  }

  const xRange = range.xMax - range.xMin || 1;
  const yRange = range.yMax - range.yMin || 1;
  const aspect = (w - pad * 2) / (h - pad * 2);
  if (xRange / yRange > aspect) {
    const target = xRange / aspect;
    const extra = (target - yRange) / 2;
    range = { ...range, yMin: range.yMin - extra, yMax: range.yMax + extra };
  } else {
    const target = yRange * aspect;
    const extra = (target - xRange) / 2;
    range = { ...range, xMin: range.xMin - extra, xMax: range.xMax + extra };
  }

  const xMin = range.xMin;
  const xMax = range.xMax;
  const yMin = range.yMin;
  const yMax = range.yMax;
  state.space2dView.lastRender = {
    range: { xMin, xMax, yMin, yMax },
    width: w,
    height: h,
    pad,
  };
  const mapX = (x) => pad + ((x - xMin) / (xMax - xMin || 1)) * (w - pad * 2);
  const mapY = (y) => h - pad - ((y - yMin) / (yMax - yMin || 1)) * (h - pad * 2);

  const viewConfig = state.viewConfig ?? {};
  const showGrid = viewConfig.showGrid ?? true;
  const showAxis = viewConfig.showAxis ?? true;

  if (showGrid) {
    const gridCount = 5;
    ctx.strokeStyle = "rgba(255,255,255,0.15)";
    ctx.lineWidth = 1;
    for (let i = 0; i < gridCount; i++) {
      const t = i / (gridCount - 1);
      const gx = pad + t * (w - pad * 2);
      const gy = pad + t * (h - pad * 2);
      ctx.beginPath();
      ctx.moveTo(gx, pad);
      ctx.lineTo(gx, h - pad);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(pad, gy);
      ctx.lineTo(w - pad, gy);
      ctx.stroke();
    }
  }

  if (showAxis) {
    ctx.strokeStyle = "rgba(255,255,255,0.35)";
    ctx.lineWidth = 1.5;
    if (xMin <= 0 && xMax >= 0) {
      const xZero = mapX(0);
      ctx.beginPath();
      ctx.moveTo(xZero, pad);
      ctx.lineTo(xZero, h - pad);
      ctx.stroke();
    }
    if (yMin <= 0 && yMax >= 0) {
      const yZero = mapY(0);
      ctx.beginPath();
      ctx.moveTo(pad, yZero);
      ctx.lineTo(w - pad, yZero);
      ctx.stroke();
    }
  }

  const orderedRuns = useRuns
    ? sortRunsForRender(runs)
    : [
        {
          points: fallbackPoints,
          colors: { line: "#8ecae6", point: "#219ebc" },
          highlight: [],
          id: "space2d",
        },
      ];
  orderedRuns.forEach((run) => {
    const points = run.points ?? [];
    if (!points.length) return;
    const baseOpacity = Number.isFinite(run.opacity) ? Math.min(1, Math.max(0, run.opacity)) : 1;
    const sceneFactor = useRuns ? getSceneRunFactor(run) : 1;
    const opacity = Math.min(1, Math.max(0, baseOpacity * sceneFactor));
    ctx.save();
    ctx.globalAlpha = opacity;
    ctx.strokeStyle = run.colors?.line ?? "#8ecae6";
    ctx.lineWidth = run.id === state.activeRunId ? 2.5 : 2;
    ctx.beginPath();
    points.forEach((p, i) => {
      const x = mapX(p.x);
      const y = mapY(p.y);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.stroke();

    const highlight = run.highlight ?? [];
    if (highlight.length) {
      ctx.fillStyle = run.colors?.point ?? "#219ebc";
      highlight.forEach((p) => {
        ctx.beginPath();
        ctx.arc(mapX(p.x), mapY(p.y), 3, 0, Math.PI * 2);
        ctx.fill();
      });
    }
    ctx.restore();
  });

  if (!useRuns && shapeList.length) {
    const scaleX = (w - pad * 2) / (xMax - xMin || 1);
    const scaleY = (h - pad * 2) / (yMax - yMin || 1);
    const scale = Math.min(scaleX, scaleY);
    shapeList.forEach((shape) => {
      if (!shape) return;
      const kind = String(shape.kind ?? "").toLowerCase();
      const baseAlpha = Number.isFinite(shape.opacity)
        ? shape.opacity
        : Number.isFinite(shape.alpha)
          ? shape.alpha
          : 1;
      const sceneFactor = getSceneShapeFactor(shape);
      const progress = getSceneProgressForShape(shape);
      if (progress <= 0) return;
      const alpha = Math.min(1, Math.max(0, baseAlpha * sceneFactor * progress));
      if (kind === "line") {
        if (![shape.x1, shape.y1, shape.x2, shape.y2].every(Number.isFinite)) return;
        ctx.save();
        ctx.globalAlpha = alpha;
        const stroke = shape.stroke ?? "#f97316";
        const width = Math.max(1, (Number(shape.width) || 0.01) * scale);
        ctx.strokeStyle = stroke;
        ctx.lineWidth = width;
        ctx.beginPath();
        ctx.moveTo(mapX(shape.x1), mapY(shape.y1));
        ctx.lineTo(mapX(shape.x2), mapY(shape.y2));
        ctx.stroke();
        ctx.restore();
        return;
      }
      if (kind === "circle") {
        if (![shape.x, shape.y, shape.r].every(Number.isFinite)) return;
        ctx.save();
        ctx.globalAlpha = alpha;
        const radius = Math.max(1, shape.r * scale);
        ctx.beginPath();
        ctx.arc(mapX(shape.x), mapY(shape.y), radius, 0, Math.PI * 2);
        if (shape.fill) {
          ctx.fillStyle = shape.fill;
          ctx.fill();
        }
        const stroke = shape.stroke ?? "#38bdf8";
        const width = Math.max(1, (Number(shape.width) || 0.01) * scale);
        ctx.strokeStyle = stroke;
        ctx.lineWidth = width;
        ctx.stroke();
        ctx.restore();
        return;
      }
      if (kind === "point") {
        if (![shape.x, shape.y].every(Number.isFinite)) return;
        ctx.save();
        ctx.globalAlpha = alpha;
        const size = Math.max(2, (Number(shape.size) || 0.03) * scale);
        const color = shape.color ?? shape.stroke ?? "#f8fafc";
        ctx.fillStyle = color;
        ctx.beginPath();
        ctx.arc(mapX(shape.x), mapY(shape.y), size, 0, Math.PI * 2);
        ctx.fill();
        ctx.restore();
        return;
      }
    });
  }

  if (!useRuns && drawItems.length) {
    const scaleX = (w - pad * 2) / (xMax - xMin || 1);
    const scaleY = (h - pad * 2) / (yMax - yMin || 1);
    const scale = Math.min(scaleX, scaleY);
    drawItems.forEach((item) => {
      if (!item || typeof item !== "object") return;
      const kind = normalizeDrawKind(item);
      const baseAlpha = Number.isFinite(item.opacity)
        ? item.opacity
        : Number.isFinite(item.alpha)
          ? item.alpha
          : 1;
      const tokenized = {
        token: item.token ?? item.토큰,
        id: item.id ?? item.name ?? item.label,
        label: item.label,
      };
      const sceneFactor = getSceneShapeFactor(tokenized);
      const progress = getSceneProgressForShape(tokenized);
      if (progress <= 0) return;
      const alpha = Math.min(1, Math.max(0, baseAlpha * sceneFactor * progress));
      const stroke = item.stroke ?? item.color ?? item.색 ?? "#f97316";
      const fill = item.fill ?? item.채움 ?? null;
      const width = Number(item.width ?? item.굵기 ?? 0.02);
      const lineWidth = Math.max(1, width * scale);

      if (kind === "line") {
        const line = resolveLineEndpoints(item);
        if (!line) return;
        const x2 = line.x1 + (line.x2 - line.x1) * progress;
        const y2 = line.y1 + (line.y2 - line.y1) * progress;
        ctx.save();
        ctx.globalAlpha = alpha;
        ctx.strokeStyle = stroke;
        ctx.lineWidth = lineWidth;
        ctx.beginPath();
        ctx.moveTo(mapX(line.x1), mapY(line.y1));
        ctx.lineTo(mapX(x2), mapY(y2));
        ctx.stroke();
        ctx.restore();
        return;
      }
      if (kind === "arrow") {
        const line = resolveLineEndpoints(item);
        if (!line) return;
        const dx = line.x2 - line.x1;
        const dy = line.y2 - line.y1;
        const length = Math.hypot(dx, dy) || 1;
        const x2 = line.x1 + dx * progress;
        const y2 = line.y1 + dy * progress;
        const head = Number(item.head_size ?? item.head ?? 0.18);
        const headLen = Number.isFinite(head) && head > 0 ? head : 0.18;
        const ux = dx / length;
        const uy = dy / length;
        const leftX = x2 - ux * headLen - uy * headLen * 0.5;
        const leftY = y2 - uy * headLen + ux * headLen * 0.5;
        const rightX = x2 - ux * headLen + uy * headLen * 0.5;
        const rightY = y2 - uy * headLen - ux * headLen * 0.5;
        ctx.save();
        ctx.globalAlpha = alpha;
        ctx.strokeStyle = stroke;
        ctx.lineWidth = lineWidth;
        ctx.beginPath();
        ctx.moveTo(mapX(line.x1), mapY(line.y1));
        ctx.lineTo(mapX(x2), mapY(y2));
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(mapX(x2), mapY(y2));
        ctx.lineTo(mapX(leftX), mapY(leftY));
        ctx.lineTo(mapX(rightX), mapY(rightY));
        ctx.closePath();
        ctx.fillStyle = fill ?? stroke;
        ctx.fill();
        ctx.restore();
        return;
      }
      if (kind === "circle") {
        const circle = resolveCircle(item);
        if (!circle) return;
        ctx.save();
        ctx.globalAlpha = alpha;
        const radius = Math.max(1, circle.r * scale);
        ctx.beginPath();
        ctx.arc(mapX(circle.x), mapY(circle.y), radius, 0, Math.PI * 2);
        if (fill) {
          ctx.fillStyle = fill;
          ctx.fill();
        }
        ctx.strokeStyle = stroke;
        ctx.lineWidth = lineWidth;
        ctx.stroke();
        ctx.restore();
        return;
      }
      if (kind === "point") {
        const point = resolvePoint(item);
        if (!point) return;
        const size = Number(item.size ?? item.크기 ?? 0.05);
        ctx.save();
        ctx.globalAlpha = alpha;
        ctx.fillStyle = fill ?? stroke;
        ctx.beginPath();
        ctx.arc(mapX(point.x), mapY(point.y), Math.max(2, size * scale), 0, Math.PI * 2);
        ctx.fill();
        ctx.restore();
        return;
      }
      if (kind === "polyline" || kind === "polygon") {
        const listRaw = normalizePointList(item.points ?? item.coords ?? item.vertices ?? item.점들 ?? item.좌표);
        const list = progress < 1 ? slicePointsByProgress(listRaw, progress) : listRaw;
        if (!list.length) return;
        ctx.save();
        ctx.globalAlpha = alpha;
        ctx.strokeStyle = stroke;
        ctx.lineWidth = lineWidth;
        ctx.beginPath();
        list.forEach((p, idx) => {
          const x = mapX(p.x);
          const y = mapY(p.y);
          if (idx === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        });
        if (kind === "polygon" && progress >= 1) {
          ctx.closePath();
          if (fill) {
            ctx.fillStyle = fill;
            ctx.fill();
          }
        }
        ctx.stroke();
        ctx.restore();
        return;
      }
      if (kind === "rect") {
        const rect = resolveRect(item);
        if (!rect) return;
        ctx.save();
        ctx.globalAlpha = alpha;
        ctx.strokeStyle = stroke;
        ctx.lineWidth = lineWidth;
        const x1 = mapX(rect.x1);
        const y1 = mapY(rect.y1);
        const x2 = mapX(rect.x2);
        const y2 = mapY(rect.y2);
        const left = Math.min(x1, x2);
        const top = Math.min(y1, y2);
        const widthPx = Math.abs(x2 - x1);
        const heightPx = Math.abs(y2 - y1);
        if (fill) {
          ctx.fillStyle = fill;
          ctx.fillRect(left, top, widthPx, heightPx);
        }
        ctx.strokeRect(left, top, widthPx, heightPx);
        ctx.restore();
        return;
      }
      if (kind === "text") {
        const pos = resolvePoint(item);
        const content = String(item.text ?? item.label ?? item.글 ?? item.내용 ?? "");
        if (!pos || !content) return;
        const size = Number(item.size ?? item.크기 ?? 0.2);
        const fontSize = Math.max(10, Number.isFinite(size) ? size * scale * 1.2 : 14);
        const alignRaw = String(item.align ?? item.정렬 ?? "center").toLowerCase();
        const baselineRaw = String(item.baseline ?? item.기준 ?? "middle").toLowerCase();
        ctx.save();
        ctx.globalAlpha = alpha;
        ctx.fillStyle = fill ?? stroke;
        ctx.font = `${Math.round(fontSize)}px ${getComputedStyle(document.body).fontFamily}`;
        ctx.textAlign = ["left", "right", "center", "start", "end"].includes(alignRaw)
          ? alignRaw
          : "center";
        ctx.textBaseline = ["top", "middle", "bottom", "alphabetic", "hanging"].includes(baselineRaw)
          ? baselineRaw
          : "middle";
        ctx.fillText(content, mapX(pos.x), mapY(pos.y));
        ctx.restore();
        return;
      }
      if (kind === "rect") {
        const rect = resolveRect(item);
        if (!rect) return;
        const x = mapX(rect.x1);
        const y = mapY(rect.y2);
        const wPx = Math.abs(mapX(rect.x2) - mapX(rect.x1));
        const hPx = Math.abs(mapY(rect.y1) - mapY(rect.y2));
        ctx.save();
        ctx.globalAlpha = alpha;
        if (fill) {
          ctx.fillStyle = fill;
          ctx.fillRect(x, y, wPx, hPx);
        }
        ctx.strokeStyle = stroke;
        ctx.lineWidth = lineWidth;
        ctx.strokeRect(x, y, wPx, hPx);
        ctx.restore();
        return;
      }
      if (kind === "text") {
        const pos = resolvePoint(item);
        const text = item.text ?? item.글 ?? item.label;
        if (!pos || !text) return;
        const size = Number(item.size ?? item.크기 ?? 0.18);
        const fontSize = Math.max(10, size * scale);
        ctx.save();
        ctx.globalAlpha = alpha;
        ctx.fillStyle = fill ?? stroke ?? "#f8fafc";
        ctx.font = `${Math.round(fontSize)}px ${getComputedStyle(document.body).fontFamily}`;
        ctx.textAlign = item.align ?? "center";
        ctx.textBaseline = item.baseline ?? "middle";
        ctx.fillText(String(text), mapX(pos.x), mapY(pos.y));
        ctx.restore();
      }
    });
  }
}

function renderStructureSelection() {
  const el = $("structure-selection");
  if (!el) return;
  const selection = state.structure?.selection;
  if (!selection) {
    el.textContent = "선택 없음";
    return;
  }
  if (selection.kind === "node") {
    const meta = selection.node.meta ? JSON.stringify(selection.node.meta) : "-";
    el.textContent = `노드 ${selection.node.id} (${selection.node.label ?? "-"}) | meta=${meta}`;
    return;
  }
  el.textContent = "선택 없음";
}

function renderStructureView() {
  const canvas = $("structure-canvas");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  const w = canvas.width;
  const h = canvas.height;
  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = "#0f172a";
  ctx.fillRect(0, 0, w, h);

  const structure = state.structure;
  if (!structure) {
    ctx.fillStyle = "rgba(255,255,255,0.6)";
    ctx.font = "14px " + getComputedStyle(document.body).fontFamily;
    ctx.fillText("구조 데이터를 불러오면 표시됩니다.", 24, 40);
    state.structureLayout = null;
    renderStructureSelection();
    return;
  }

  const layoutType = state.structureView.layout ?? structure.layout?.type ?? "circle";
  const nodes = computeStructureLayout(structure, layoutType);
  state.structureLayout = nodes;
  const nodeMap = new Map(nodes.map((node) => [node.id, node]));
  const edges = structure.edges ?? [];
  const pad = 40;
  const mapX = (x) => pad + x * (w - pad * 2);
  const mapY = (y) => pad + y * (h - pad * 2);
  const nodeSize = state.structureView.nodeSize ?? 10;

  ctx.strokeStyle = "rgba(255,255,255,0.4)";
  ctx.lineWidth = 1.5;
  edges.forEach((edge) => {
    const from = nodeMap.get(edge.from);
    const to = nodeMap.get(edge.to);
    if (!from || !to) return;
    const x1 = mapX(from._pos.x);
    const y1 = mapY(from._pos.y);
    const x2 = mapX(to._pos.x);
    const y2 = mapY(to._pos.y);
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
    if (edge.directed !== false) {
      const angle = Math.atan2(y2 - y1, x2 - x1);
      const arrowLen = 8;
      ctx.beginPath();
      ctx.moveTo(x2, y2);
      ctx.lineTo(x2 - arrowLen * Math.cos(angle - 0.3), y2 - arrowLen * Math.sin(angle - 0.3));
      ctx.lineTo(x2 - arrowLen * Math.cos(angle + 0.3), y2 - arrowLen * Math.sin(angle + 0.3));
      ctx.closePath();
      ctx.fillStyle = "rgba(255,255,255,0.4)";
      ctx.fill();
    }
  });

  nodes.forEach((node) => {
    const x = mapX(node._pos.x);
    const y = mapY(node._pos.y);
    const selected = state.structure?.selection?.node?.id === node.id;
    ctx.beginPath();
    ctx.fillStyle = selected ? "#f4a261" : "#8ecae6";
    ctx.arc(x, y, nodeSize, 0, Math.PI * 2);
    ctx.fill();
    if (state.structureView.showLabels !== false) {
      ctx.fillStyle = "rgba(255,255,255,0.85)";
      ctx.font = "12px " + getComputedStyle(document.body).fontFamily;
      ctx.fillText(node.label ?? node.id, x + nodeSize + 6, y + 4);
    }
  });
  renderStructureSelection();
}

function renderOverlayList() {
  const list = $("overlay-list");
  list.innerHTML = "";
  ensureLayerIndices(state.runs);
  const runs = sortRunsForList(state.runs);
  runs.forEach((run, idx) => {
    const chip = document.createElement("div");
    chip.className = "overlay-chip";
    const swatch = document.createElement("span");
    swatch.className = "overlay-swatch";
    swatch.style.background = run.colors?.line ?? "#8ecae6";
    const meta = document.createElement("div");
    meta.className = "overlay-meta";
    const label = document.createElement("div");
    label.className = "overlay-title";
    const roleLabel =
      run.compareRole === "baseline"
        ? "기준"
        : run.compareRole === "variant"
          ? "비교"
          : "";
    label.textContent = `${run.label ?? "레이어"}${roleLabel ? ` · ${roleLabel}` : ""}`;
    const order = document.createElement("div");
    order.className = "overlay-order";
    order.textContent = `겹 ${Number.isFinite(run.layerIndex) ? run.layerIndex : idx}`;
    meta.appendChild(label);
    meta.appendChild(order);
    const editor = document.createElement("div");
    editor.className = "overlay-editor";
    const orderLabel = document.createElement("label");
    orderLabel.className = "overlay-input";
    orderLabel.textContent = "순서";
    const orderInput = document.createElement("input");
    orderInput.type = "number";
    orderInput.value = Number.isFinite(run.layerIndex) ? run.layerIndex : idx;
    orderInput.addEventListener("change", () => {
      setRunLayerIndex(run.id, Number(orderInput.value));
    });
    orderLabel.appendChild(orderInput);
    const opacityLabel = document.createElement("label");
    opacityLabel.className = "overlay-input";
    opacityLabel.textContent = "투명도";
    const opacityInput = document.createElement("input");
    opacityInput.type = "range";
    opacityInput.min = "0";
    opacityInput.max = "100";
    opacityInput.value = Math.round((Number.isFinite(run.opacity) ? run.opacity : 1) * 100);
    opacityInput.addEventListener("input", () => {
      setRunOpacity(run.id, Number(opacityInput.value) / 100);
    });
    opacityLabel.appendChild(opacityInput);
    editor.appendChild(orderLabel);
    editor.appendChild(opacityLabel);
    const toggle = document.createElement("button");
    toggle.className = "ghost";
    toggle.textContent = run.visible ? "숨김" : "표시";
    toggle.addEventListener("click", () => {
      run.visible = !run.visible;
      if (!run.visible) {
        if (state.soloRunId === run.id) state.soloRunId = null;
        if (state.activeRunId === run.id) {
          const next = sortRunsForList(state.runs.filter((item) => item.visible))[0];
          state.activeRunId = next?.id ?? null;
        }
        if (state.compare.baselineId === run.id || state.compare.variantId === run.id) {
          state.compare.blockReason = "비교 레이어가 숨김 처리되었습니다.";
          updateCompareStatusUI();
        }
      } else if (!state.activeRunId) {
        state.activeRunId = run.id;
        if (state.compare.blockReason) {
          state.compare.blockReason = "";
          updateCompareStatusUI();
        }
      }
      renderOverlayList();
      renderRunList();
      renderInspector();
      renderAll();
    });
    const solo = document.createElement("button");
    solo.className = "ghost";
    solo.textContent = state.soloRunId === run.id ? "solo 해제" : "solo";
    solo.addEventListener("click", () => {
      state.soloRunId = state.soloRunId === run.id ? null : run.id;
      renderOverlayList();
      renderAll();
    });
    const moveUp = document.createElement("button");
    moveUp.className = "ghost";
    moveUp.textContent = "위";
    moveUp.addEventListener("click", () => moveRunLayer(run.id, 1));
    const moveDown = document.createElement("button");
    moveDown.className = "ghost";
    moveDown.textContent = "아래";
    moveDown.addEventListener("click", () => moveRunLayer(run.id, -1));
    const controls = document.createElement("div");
    controls.className = "overlay-controls";
    controls.appendChild(toggle);
    controls.appendChild(solo);
    controls.appendChild(moveUp);
    controls.appendChild(moveDown);
    chip.addEventListener("mouseenter", () => {
      state.hoverRunId = run.id;
      renderAll();
    });
    chip.addEventListener("mouseleave", () => {
      state.hoverRunId = null;
      renderAll();
    });
    chip.appendChild(swatch);
    chip.appendChild(meta);
    chip.appendChild(editor);
    chip.appendChild(controls);
    list.appendChild(chip);
  });
  const visibleCount = getVisibleRuns().length;
  $("overlay-label").textContent = `레이어(${visibleCount})`;
}

function renderRunList() {
  const list = $("run-list");
  list.innerHTML = "";
  ensureLayerIndices(state.runs);
  const runs = sortRunsForList(state.runs);
  if (!runs.length) {
    const empty = document.createElement("div");
    empty.className = "run-list-empty";
    empty.textContent = state.controls.autoRun
      ? "실행 없음 · DDN 실행을 눌러 그래프를 만드세요."
      : "실행 없음 · 자동 실행이 꺼져 있습니다.";
    list.appendChild(empty);
    renderDockSummary();
    return;
  }
  runs.forEach((run) => {
    const row = document.createElement("div");
    row.className = "run-list-item";
    if (run.id === state.activeRunId) row.classList.add("active");

    const swatch = document.createElement("span");
    swatch.className = "overlay-swatch";
    swatch.style.background = run.colors?.line ?? "#8ecae6";

    const label = document.createElement("span");
    const roleLabel =
      run.compareRole === "baseline"
        ? "기준"
        : run.compareRole === "variant"
          ? "비교"
          : "";
    label.textContent = `${run.label ?? "run"}${roleLabel ? ` · ${roleLabel}` : ""} (겹 ${run.layerIndex ?? 0}, ${run.points?.length ?? 0})`;

    const actions = document.createElement("div");
    const selectBtn = document.createElement("button");
    selectBtn.className = "ghost";
    selectBtn.textContent = "선택";
    selectBtn.addEventListener("click", () => setActiveRun(run.id));
    const deleteBtn = document.createElement("button");
    deleteBtn.className = "ghost";
    deleteBtn.textContent = "삭제";
    deleteBtn.addEventListener("click", () => removeRun(run.id));
    actions.appendChild(selectBtn);
    actions.appendChild(deleteBtn);

    row.appendChild(swatch);
    row.appendChild(label);
    row.appendChild(actions);
    list.appendChild(row);
  });
  renderDockSummary();
}

function renderSnapshots() {
  const list = $("snapshot-list");
  list.innerHTML = "";
  state.snapshots.forEach((snap) => {
    const li = document.createElement("li");
    const text = document.createElement("span");
    text.textContent = `${snap.ts} | ${snap.note}`;
    const btn = document.createElement("button");
    btn.textContent = "불러오기";
    btn.addEventListener("click", () => {
      if (snap.run?.graph) {
        const runs = createRunsFromGraph(snap.run.graph, snap.run.source, snap.run.inputs);
        if (Number.isFinite(snap.run.layer_index)) {
          runs.forEach((run) => {
            run.layerIndex = snap.run.layer_index;
          });
        }
        addRuns(runs);
      } else {
        log("스냅샷에 그래프 데이터가 없습니다.");
      }
    });
    li.appendChild(text);
    li.appendChild(btn);
    list.appendChild(li);
  });
}

function renderLogs() {
  const list = $("log-list");
  const fixedList = $("log-list-fixed");
  const empty = $("log-empty");
  const fixedEmpty = $("log-empty-fixed");
  const lists = [list, fixedList].filter(Boolean);
  lists.forEach((el) => (el.innerHTML = ""));
  if (!state.logs.length) {
    if (empty) empty.classList.remove("hidden");
    if (fixedEmpty) fixedEmpty.classList.remove("hidden");
    return;
  }
  if (empty) empty.classList.add("hidden");
  if (fixedEmpty) fixedEmpty.classList.add("hidden");
  state.logs.forEach((entry, idx) => {
    const li = document.createElement("li");
    li.textContent = `${entry.ts} - ${entry.message}`;
    if (idx === 0) li.classList.add("log-entry-latest");
    lists.forEach((el) => el.appendChild(li.cloneNode(true)));
  });
}

function renderSamSummary() {
  const el = $("sam-summary");
  if (!state.sam) {
    el.textContent = "sam: 미로드";
    return;
  }
  const schema = state.sam.summary.schema ?? "-";
  const recordCount = state.sam.recordCount ?? "-";
  const recordLabel = state.sam.recordLabel ?? "records";
  const size = state.sam.summary.size ?? "-";
  const hash = state.sam.summary.hash ?? "-";
  const verdict = state.sam.validation?.ok ? "ok" : "warn";
  const keyInfo = state.sam.inputKeySample?.length
    ? `${state.sam.inputKeySample.join(", ")}${state.sam.inputKeyCount > 5 ? "..." : ""}`
    : "-";
  el.innerHTML = [
    `<div><strong>sam</strong> ${state.sam.name}</div>`,
    `<div>schema: ${schema}</div>`,
    `<div>${recordLabel}: ${recordCount}</div>`,
    `<div>keys: ${keyInfo} (${state.sam.inputKeyCount ?? 0})</div>`,
    `<div>size: ${size}</div>`,
    `<div>hash: ${hash}</div>`,
    `<div>검증: ${verdict}</div>`,
  ].join("");
}

function renderGeoulSummary() {
  const el = $("geoul-summary");
  if (!state.geoul) {
    el.textContent = "geoul: 미로드";
    return;
  }
  const schema = state.geoul.summary.schema ?? "-";
  const frameCount = state.geoul.frameCount ?? "-";
  const size = state.geoul.summary.size ?? "-";
  const hash = state.geoul.summary.hash ?? "-";
  const hashes = state.geoul.hashes ?? {};
  const range = state.geoul.frameRange
    ? `${state.geoul.frameRange.min}..${state.geoul.frameRange.max}`
    : "-";
  const verdict = state.geoul.validation?.ok ? "ok" : "warn";
  el.innerHTML = [
    `<div><strong>geoul</strong> ${state.geoul.name}</div>`,
    `<div>schema: ${schema}</div>`,
    `<div>frames: ${frameCount} (madi ${range})</div>`,
    `<div>state_hash: ${hashes.state ?? "-"}</div>`,
    `<div>trace_hash: ${hashes.trace ?? "-"}</div>`,
    `<div>bogae_hash: ${hashes.bogae ?? "-"}</div>`,
    `<div>size: ${size}</div>`,
    `<div>hash: ${hash}</div>`,
    `<div>검증: ${verdict}</div>`,
  ].join("");
}
function renderContractInspector(graph) {
  const badge = $("contract-status");
  const summary = $("contract-summary");
  const fields = $("contract-fields");
  const ddn = $("contract-ddn");
  fields.innerHTML = "";
  ddn.textContent = "";

  if (!graph) {
    badge.textContent = "-";
    badge.className = "contract-badge";
    summary.textContent = "스키마를 불러오면 상세가 표시됩니다.";
    ddn.classList.add("hidden");
    summary.classList.remove("hidden");
    fields.classList.remove("hidden");
    return;
  }

  const contract = VIEW_CONTRACTS[graph.schema];
  if (!contract) {
    badge.textContent = "unknown";
    badge.className = "contract-badge";
    summary.textContent = `알 수 없는 스키마: ${graph.schema ?? "-"}`;
    ddn.classList.add("hidden");
    summary.classList.remove("hidden");
    fields.classList.remove("hidden");
    return;
  }

  const update = graph.meta?.update ?? "replace";
  const tick = Number.isFinite(graph.meta?.tick) ? graph.meta.tick : "-";
  const sampleVar = graph.sample?.var ?? "x";
  const sample = graph.sample
    ? `${sampleVar} ${graph.sample.x_min}..${graph.sample.x_max} step ${graph.sample.step}`
    : "미지정";
  const view = graph.view
    ? `x ${graph.view.x_min}..${graph.view.x_max}, y ${graph.view.y_min}..${graph.view.y_max}`
    : "auto";

  summary.innerHTML = [
    `<div><strong>${contract.title}</strong> (${graph.schema})</div>`,
    `<div>update: ${update} / tick: ${tick}</div>`,
    `<div>sample: ${sample}</div>`,
    `<div>view: ${view}</div>`,
  ].join("");

  let okCount = 0;
  contract.required.forEach((rule) => {
    const ok = rule.check(graph);
    if (ok) okCount += 1;
    const row = document.createElement("div");
    row.className = "contract-field";
    const label = document.createElement("span");
    label.textContent = rule.label;
    const status = document.createElement("span");
    status.className = `status ${ok ? "ok" : "warn"}`;
    status.textContent = ok ? "ok" : "warn";
    row.appendChild(label);
    row.appendChild(status);
    fields.appendChild(row);
  });

  badge.textContent = okCount === contract.required.length ? "ok" : "warn";
  badge.className = `contract-badge ${okCount === contract.required.length ? "ok" : "warn"}`;

  const ddnLines = [];
  ddnLines.push("보개계약: {");
  ddnLines.push(`  스키마: "${graph.schema}".`);
  ddnLines.push(`  업데이트: "${update}".`);
  if (Number.isFinite(graph.meta?.tick)) {
    ddnLines.push(`  틱: ${graph.meta.tick}.`);
  }
  if (graph.sample) {
    ddnLines.push(
      `  표본: (var="${sampleVar}", x_min=${graph.sample.x_min}, x_max=${graph.sample.x_max}, step=${graph.sample.step}).`,
    );
  }
  if (graph.view) {
    ddnLines.push(
      `  표시: (x_min=${graph.view.x_min}, x_max=${graph.view.x_max}, y_min=${graph.view.y_min}, y_max=${graph.view.y_max}, pan_x=${graph.view.pan_x}, pan_y=${graph.view.pan_y}, zoom=${graph.view.zoom}).`,
    );
  }
  if (graph.series?.length) {
    const label = graph.series[0]?.label ?? "f(x)";
    const count = graph.series[0]?.points?.length ?? 0;
    ddnLines.push(`  시리즈: "${label}".`);
    ddnLines.push(`  점열_개수: ${count}.`);
  }
  ddnLines.push("}.");
  ddn.textContent = ddnLines.join("\n");

  if (state.contractView === "ddn") {
    summary.classList.add("hidden");
    fields.classList.add("hidden");
    ddn.classList.remove("hidden");
  } else {
    summary.classList.remove("hidden");
    fields.classList.remove("hidden");
    ddn.classList.add("hidden");
  }
}

async function updateBridgeCheck(run) {
  const target = $("bridge-check");
  if (!run || !run.graph) {
    target.textContent = "bridge_check: -";
    target.className = "status-pill";
    return;
  }
  const sourceText = run.source?.text;
  const graphInputHash = run.graph.meta?.source_input_hash ?? "";
  if (!sourceText || graphInputHash.startsWith("local:") || run.source?.kind !== "ddn") {
    target.textContent = "bridge_check: n/a";
    target.className = "status-pill";
    return;
  }
  try {
    const normalized = normalizeDdnForHash(sourceText);
    const inputHash = await sha256Hex(normalized);
    const points = run.graph.series?.[0]?.points ?? [];
    const resultHash = await computeResultHash(points);
    const graphResultHash = run.graph.meta?.result_hash ?? "";
    const inputOk = graphInputHash === inputHash;
    const resultOk = graphResultHash ? graphResultHash === resultHash : true;
    const ok = inputOk && resultOk;
    target.textContent = ok ? "bridge_check: ok" : "bridge_check: warn";
    target.className = `status-pill ${ok ? "ok" : "warn"}`;
  } catch (err) {
    target.textContent = "bridge_check: warn";
    target.className = "status-pill warn";
  }
}

function buildScenePreview() {
  ensureLayerIndices(state.runs);
  const time = readTimeControls();
  const run = getActiveRun();
  const layers = sortRunsForRender(state.runs).map((item) => ({
    id: item.id,
    label: item.label ?? null,
    visible: item.visible ?? true,
    order: item.layerIndex ?? 0,
    opacity: Number.isFinite(item.opacity) ? item.opacity : 1,
    update: item.graph?.meta?.update ?? null,
    tick: Number.isFinite(item.graph?.meta?.tick) ? item.graph.meta.tick : null,
    series_id: item.seriesId ?? null,
    points: (item.fullPoints ?? item.points)?.length ?? 0,
  }));
  if (!layers.length && state.space2d?.points?.length) {
    layers.push({
      id: "space2d",
      label: state.space2d.meta?.title ?? "2d",
      visible: true,
      order: 0,
      update: null,
      tick: null,
      series_id: null,
      points: state.space2d.points.length,
    });
  }
  const requiredViews =
    Array.isArray(state.lessons.meta?.required_views) && state.lessons.meta.required_views.length
      ? state.lessons.meta.required_views
      : ["graph"];
  const inputHashes = run?.graph?.meta ?? {};
  const activeInput = run?.source?.kind
    ? {
        kind: run.source.kind,
        label: run.label ?? null,
        input_hash: inputHashes.source_input_hash ?? null,
        result_hash: inputHashes.result_hash ?? null,
      }
    : null;
  const frames = (state.time.frames ?? []).map((frame) => frame.t);
  const view = state.viewConfig
    ? {
        range: state.viewConfig.range,
        pan_x: state.viewConfig.panX,
        pan_y: state.viewConfig.panY,
        zoom: state.viewConfig.zoom,
        grid: state.viewConfig.showGrid,
        axis: state.viewConfig.showAxis,
      }
    : null;
  return {
    schema: "seamgrim.scene.v0",
    ts: new Date().toISOString(),
    view: {
      kind: state.activeView ?? "view-graph",
      config: view,
    },
    inputs: activeInput,
    required_views: requiredViews,
    layers,
    timeline: time.enabled
      ? {
          t_min: time.t_min,
          t_max: time.t_max,
          step: time.step,
          now: time.now,
          playing: state.time.playing,
          frame_count: frames.length,
          frame_sample: frames.slice(0, 12),
        }
      : null,
    hashes: {
      input_hash: inputHashes.source_input_hash ?? null,
      result_hash: inputHashes.result_hash ?? null,
    },
    bogae_scene: state.scene?.segments ?? null,
  };
}

function renderScenePreview() {
  const el = $("scene-json");
  if (!el) return;
  const scene = buildScenePreview();
  el.textContent = JSON.stringify(scene, null, 2);
}

function renderInspector() {
  const run = getActiveRun();
  $("run-status").textContent = run ? `run: ${run.label ?? "-"}` : "run: -";
  if (!run) {
    $("inspector-input-hash").textContent = "input_hash: -";
    $("inspector-result-hash").textContent = "result_hash: -";
    $("input-hash").textContent = "input_hash: -";
    $("result-hash").textContent = "result_hash: -";
    $("ddn-meta-name").textContent = "이름: -";
    $("ddn-meta-desc").textContent = "설명: -";
    $("schema-graph").textContent = "graph: -";
    $("schema-snapshot").textContent = `snapshot: ${state.schemaStatus.snapshot}`;
    $("schema-session").textContent = `session: ${state.schemaStatus.session}`;
    renderContractInspector(null);
    renderScenePreview();
    renderInputRegistry();
    return;
  }

  const graph = run.graph;
  const inputHash = graph?.meta?.source_input_hash ?? "-";
  const resultHash = graph?.meta?.result_hash ?? "-";
  $("inspector-input-hash").textContent = `input_hash: ${inputHash}`;
  $("inspector-result-hash").textContent = `result_hash: ${resultHash}`;
  $("input-hash").textContent = `input_hash: ${inputHash}`;
  $("result-hash").textContent = `result_hash: ${resultHash}`;

  const meta = run.source?.text ? extractDdnMeta(run.source.text) : { name: "", desc: "" };
  $("ddn-meta-name").textContent = meta.name ? `이름: ${meta.name}` : "이름: -";
  $("ddn-meta-desc").textContent = meta.desc ? `설명: ${meta.desc}` : "설명: -";

  $("schema-graph").textContent = `graph: ${graph?.schema ?? "-"}`;
  $("schema-snapshot").textContent = `snapshot: ${state.schemaStatus.snapshot}`;
  $("schema-session").textContent = `session: ${state.schemaStatus.session}`;

  $("result-json").textContent = JSON.stringify(graph ?? {}, null, 2);
  renderContractInspector(graph);
  updateBridgeCheck(run);
  renderScenePreview();
  renderInputRegistry();
}

function buildGraphPayload(points, label, sampleRange, metaOverrides = {}) {
  const axisRange = computeRangeFromPoints(points);
  const updateMeta = getUpdateMetaFromControls();
  if (metaOverrides.update) updateMeta.update = metaOverrides.update;
  if (metaOverrides.tick !== undefined) updateMeta.tick = metaOverrides.tick;
  const data = {
    schema: "seamgrim.graph.v0",
    sample: {
      var: sampleRange.var ?? "x",
      x_min: sampleRange.x_min,
      x_max: sampleRange.x_max,
      step: sampleRange.step,
    },
    axis: {
      x_min: axisRange.xMin,
      x_max: axisRange.xMax,
      y_min: axisRange.yMin,
      y_max: axisRange.yMax,
    },
    series: [{ id: "main", label: label, points }],
    meta: {
      update: updateMeta.update,
      tick: updateMeta.tick,
    },
  };
  if (!Number.isFinite(updateMeta.tick)) {
    delete data.meta.tick;
  }
  return data;
}

function applyViewToGraph(graph, viewConfig) {
  if (!graph) return;
  if (!viewConfig) return;
  graph.view = {
    auto: viewConfig.auto,
    x_min: viewConfig.range.xMin,
    x_max: viewConfig.range.xMax,
    y_min: viewConfig.range.yMin,
    y_max: viewConfig.range.yMax,
    pan_x: viewConfig.panX,
    pan_y: viewConfig.panY,
    zoom: viewConfig.zoom,
  };
}

function renderAll() {
  const baseRuns = getVisibleRuns();
  const runs = getGraphRunsForRender(baseRuns);
  const lensRuns = getWasmLensRenderRuns();
  if (runs.length) {
    try {
      state.viewConfig = getViewConfigFromControls(runs);
    } catch (err) {
      log(err.message);
      return;
    }
    const activeRun = lensRuns.length ? lensRuns[0] : getActiveRun();
    if (activeRun?.graph) {
      applyViewToGraph(activeRun.graph, state.viewConfig);
    }
    const highlight =
      activeRun && runs.some((run) => run.id === activeRun.id) ? activeRun.highlight ?? null : null;
    renderGraphCanvas(runs, state.viewConfig.range, highlight);
  } else {
    renderGraphCanvas([], null, null);
  }
  render2dView();
  renderTableView();
  renderTextView();
  renderStructureView();
}

function updateGraphViewFromControls() {
  const runs = getGraphRunsForRender(getVisibleRuns());
  if (!runs.length) return;
  try {
    const viewConfig = getViewConfigFromControls(runs);
    state.viewConfig = viewConfig;
    const lensRuns = getWasmLensRenderRuns();
    const activeRun = lensRuns.length ? lensRuns[0] : getActiveRun();
    if (activeRun?.graph) {
      applyViewToGraph(activeRun.graph, viewConfig);
    }
    renderAll();
    renderInspector();
  } catch (err) {
    log(err.message);
  }
}

function updateSpace2dViewFromControls() {
  try {
    const config = getSpace2dViewConfigFromControls();
    state.space2dView = {
      ...state.space2dView,
      ...config,
    };
    render2dView();
    renderInspector();
  } catch (err) {
    log(err.message);
  }
}

function bindSpace2dCanvasInteractions() {
  const canvas = $("canvas-2d");
  bindSpace2dCanvasWorldInteractions({
    canvas,
    viewState: state.space2dView,
    hasContent: () => Boolean(state.space2d || getVisibleRuns().length),
    computeBaseRange: () =>
      state.space2dRange ??
      computeSpace2dRange(state.space2d) ??
      state.space2dView.lastRender?.range ??
      { xMin: 0, xMax: 1, yMin: 0, yMax: 1 },
    getRenderMeta: () => state.space2dView.lastRender,
    onManualViewEnsured: () => {
      syncSpace2dViewControlValues();
    },
    onViewChanged: () => {
      syncSpace2dViewControlValues();
      render2dView();
      renderInspector();
    },
    boundDatasetKey: "space2dInteractionBound",
  });
}

function updateGraphMetaFromControls() {
  const run = getActiveRun();
  if (!run?.graph) return;
  const updateMeta = getUpdateMetaFromControls();
  state.viewUpdate[state.activeView] = {
    update: updateMeta.update,
    tick: Number.isFinite(updateMeta.tick) ? updateMeta.tick : null,
  };
  run.graph.meta = {
    ...run.graph.meta,
    update: updateMeta.update,
  };
  if (Number.isFinite(updateMeta.tick)) {
    run.graph.meta.tick = updateMeta.tick;
  } else {
    delete run.graph.meta.tick;
  }
  renderInspector();
}

function getUpdateStateForView(viewId) {
  if (state.viewUpdate?.[viewId]) return state.viewUpdate[viewId];
  return state.viewUpdate["view-graph"];
}

function applyUpdateControlsForView(viewId) {
  const meta = getUpdateStateForView(viewId);
  if ($("update-mode")) $("update-mode").value = meta.update ?? "replace";
  if ($("update-tick")) {
    $("update-tick").value = Number.isFinite(meta.tick) ? meta.tick : "";
  }
}

function setContractView(mode) {
  state.contractView = mode;
  document.querySelectorAll(".contract-toggle").forEach((btn) => {
    const active = btn.dataset.contractView === mode;
    btn.classList.toggle("active", active);
  });
  renderInspector();
}

function updateAxisLabels() {
  const axisVar = "x";
  if ($("axis-var")) {
    $("axis-var").value = axisVar;
  }
  document.querySelectorAll(".axis-label").forEach((el) => {
    el.textContent = axisVar;
  });
}

function applyTimeModeUi() {
  const axisVar = $("axis-var");
  if (axisVar) {
    axisVar.disabled = true;
    axisVar.value = "x";
  }
  updateAxisLabels();
  updateDdnTimeStatus();
}

function updateDdnTimeStatus() {
  const toggle = $("ddn-time-toggle");
  const status = $("ddn-time-status");
  if (!toggle && !status) return;
  const enabled = $("time-enabled")?.checked ?? false;
  const ddnText = $("ddn-editor")?.value ?? "";
  const ddnRange = extractTimeRangeFromDdn(ddnText);
  if (toggle) {
    toggle.textContent = enabled ? "시간 샘플링: 켜짐" : "시간 샘플링: 꺼짐";
  }
  if (status) {
    if (ddnRange) {
      status.textContent = `t 범위: ${formatNumber(ddnRange.t_min)}..${formatNumber(
        ddnRange.t_max,
      )} / dt=${formatNumber(ddnRange.step)} (DDN)`;
    } else {
      const tMin = readNumber("t-min", 0);
      const tMax = readNumber("t-max", 0);
      const dt = readNumber("t-step", 0);
      status.textContent = `t 범위: ${formatNumber(tMin)}..${formatNumber(tMax)} / dt=${formatNumber(dt)}`;
    }
  }
}

function scheduleDdnEditorAutoRun() {
  if (!state.controls.autoRun) return;
  if (ddnEditorAutoRunTimer) {
    clearTimeout(ddnEditorAutoRunTimer);
    ddnEditorAutoRunTimer = null;
  }
  const waitMs = Math.max(120, Number(state.controls.editorDebounceMs ?? 420) || 420);
  ddnEditorAutoRunTimer = setTimeout(() => {
    ddnEditorAutoRunTimer = null;
    runDdnBridge({ mode: "auto" });
  }, waitMs);
}

function focusViewDock() {
  const dock = document.querySelector(".view-dock");
  if (!dock) return;
  setWorkspaceMode("basic");
  const runTab = document.querySelector('.main-tabs .tab[data-tab="run-tab"]');
  if (runTab) runTab.click();
  const graphTab = document.querySelector('.view-group .tab[data-tab="view-graph"]');
  if (graphTab) graphTab.click();
  dock.scrollIntoView({ behavior: "smooth", block: "center" });
  dock.classList.add("dock-highlight");
  setTimeout(() => dock.classList.remove("dock-highlight"), 1200);
}

function findNearestFrameIndex(frames, tValue) {
  if (!frames.length) return 0;
  let closest = 0;
  let best = Math.abs(frames[0].t - tValue);
  frames.forEach((frame, idx) => {
    const diff = Math.abs(frame.t - tValue);
    if (diff < best) {
      best = diff;
      closest = idx;
    }
  });
  return closest;
}

function applyTimeFrame(options = {}) {
  const run = getActiveRun();
  if (!run?.timeFrames?.length) return;
  const updateMeta = getUpdateMetaFromControls();
  const frame = run.timeFrames[state.time.index];
  if (!frame) return;
  const hasStatic = Array.isArray(run.staticPoints) && run.staticPoints.length;
  if (hasStatic) {
    run.points = run.staticPoints;
  } else if (updateMeta.update === "append") {
    const accumulated = run.timeFrames
      .slice(0, state.time.index + 1)
      .flatMap((item) => item.points ?? []);
    run.points = accumulated;
  } else {
    run.points = frame.points;
  }
  if (hasStatic) {
    run.fullPoints = run.staticPoints;
  } else if (!Array.isArray(run.fullPoints) || run.fullPoints.length === 0) {
    run.fullPoints = run.points ?? [];
  }
  run.highlight = frame.points;
  run.graph.series = [{ ...run.graph.series[0], points: run.points }];
  run.graph.meta.tick = frame.t;
  if (frame.space2d) {
    state.space2d = frame.space2d;
    if (!state.space2dRange) {
      state.space2dRange = computeSpace2dRange(frame.space2d);
    }
  }
  if (frame.textDoc) {
    state.textDoc = frame.textDoc;
  }
  setTimeNow(frame.t, { render: false });
  renderAll();
  renderInspector();
}

function startTimePlayback() {
  const timeState = readTimeControls();
  if (!timeState.enabled) {
    log("시간 샘플링을 켜주세요.");
    return;
  }
  const run = getActiveRun();
  if (!run?.timeFrames?.length) {
    log("시간 프레임이 없습니다. DDN 실행(브리지)을 먼저 하세요.");
    return;
  }
  if (state.time.timer) return;
  const interval = Math.max(30, timeState.interval ?? 300);
  state.time.playing = true;
  state.time.loop = timeState.loop;
  state.time.timer = setInterval(() => {
    stepTimeFrame({ auto: true });
  }, interval);
}

function stopTimePlayback() {
  if (state.time.timer) {
    clearInterval(state.time.timer);
    state.time.timer = null;
  }
  state.time.playing = false;
}

function stepTimeFrame(options = {}) {
  const run = getActiveRun();
  if (!run?.timeFrames?.length) return;
  const maxIndex = run.timeFrames.length - 1;
  if (state.time.index >= maxIndex) {
    if (state.time.loop) {
      state.time.index = 0;
    } else {
      if (options.auto) stopTimePlayback();
      return;
    }
  } else {
    state.time.index += 1;
  }
  applyTimeFrame({ keepPlaying: options.auto });
}
async function fetchGraphFromBridge(baseUrl, ddnText) {
  const formatBridgeError = (raw) => {
    if (!raw) return "bridge error";
    let message = raw;
    if (message.includes("FORMULA_IDENT_NOT_ASCII1")) {
      message += " (힌트: #ascii1 수식의 변수는 한 글자여야 합니다)";
    }
    if (message.includes("odd number of values")) {
      message += " (힌트: 출력은 x,y 쌍 또는 한 줄에 x,y[,t] 형식이어야 합니다)";
    }
    if (message.includes("mixed output")) {
      message += " (힌트: 출력 형식을 한 가지로 통일하세요)";
    }
    return message;
  };
  const response = await fetch(`${baseUrl}/api/run`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ ddn_text: ddnText }),
  });
    const payload = await response.json();
    if (!response.ok || !payload.ok) {
      throw new Error(formatBridgeError(payload.error || `bridge error: ${response.status}`));
    }
    const graph = validateGraphData(payload.graph);
    const space2d = payload.space2d ? validateSpace2dData(payload.space2d) : null;
    const textDoc = payload.text ? normalizeTextDoc(payload.text) : null;
    const table = payload.table ? normalizeTableData(payload.table) : null;
    const structure = payload.structure ? validateStructureData(payload.structure) : null;
    return { graph, space2d, textDoc, table, structure };
  }

async function buildDdnTimeFrames(ddnText, baseUrl, timeState) {
  const ddnRange = extractTimeRangeFromDdn(ddnText);
  const effective = ddnRange
    ? { ...timeState, t_min: ddnRange.t_min, t_max: ddnRange.t_max, step: ddnRange.step }
    : { ...timeState };
  if (effective.step <= 0) throw new Error("dt는 0보다 커야 합니다.");
  if (effective.t_max < effective.t_min) {
    throw new Error("t 최대는 t 최소보다 커야 합니다.");
  }
  const frames = [];
  let t = effective.t_min;
  let guard = 0;
  const total = Math.floor((effective.t_max - effective.t_min) / effective.step) + 1;
  startDdnFrameJob(total);
  while (t <= effective.t_max + 1e-9) {
    if (guard > 80) {
      throw new Error("프레임이 너무 많습니다. t 범위를 줄여주세요.");
    }
    if (state.time.ddnJob.cancelled) {
      throw new Error("DDN 프레임 생성이 중단되었습니다.");
    }
    const injected = injectTimeValueToDdn(ddnText, t);
    if (!injected.ok) {
      throw new Error(injected.warning || "DDN 시간 주입에 실패했습니다.");
    }
    const { graph, space2d, textDoc, table, structure } = await fetchGraphFromBridge(baseUrl, injected.text);
    const points = graph?.series?.[0]?.points ?? [];
    frames.push({ t: Number(t.toFixed(4)), points, graph, space2d, textDoc, table, structure });
    t += effective.step;
    guard += 1;
    state.time.ddnJob.done = frames.length;
    updateDdnFrameProgress({
      running: true,
      done: state.time.ddnJob.done,
      total: state.time.ddnJob.total,
    });
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  finishDdnFrameJob();
  return { frames, timeState: effective, source: ddnRange ? "ddn" : "view" };
}

async function runDdnBridge(options = {}) {
  const ddnText = $("ddn-editor").value;
  if (!ddnText.trim()) {
    log("DDN 텍스트를 입력하세요.");
    return false;
  }
  updateWasmLogicFromEditor();
  const autoReplace = options.mode === "auto";
  updateSceneFromDdn(ddnText);
  registerDdnInput(ddnText);
  const baseUrl = $("bridge-url").value.trim();
  if (!baseUrl) {
    log("브리지 URL을 입력하세요.");
    return false;
  }
  try {
    const timeState = readTimeControls();
    if (timeState.enabled) {
      if (ddnText.includes("#바탕숨김") && !ddnText.includes("{{t}}")) {
        log("주의: #바탕숨김 DDN은 {{t}} 또는 t <- 선언이 필요합니다.");
      }
      const built = await buildDdnTimeFrames(ddnText, baseUrl, timeState);
      let frames = built.frames;
      let useSynthetic = false;
      let staticPoints = null;
      const effectiveTime = built.timeState;
      if (!frames.length) throw new Error("프레임 생성 실패");
      if (frames.length > 1 && framesAreStatic(frames)) {
        const baseGraph = frames[0].graph ?? null;
        const baseSpace2d = frames[0].space2d ?? null;
        const baseTextDoc = frames[0].textDoc ?? null;
        const basePoints = baseGraph?.series?.[0]?.points ?? frames[0].points ?? [];
        const synthetic = buildSyntheticFramesFromPoints(basePoints, effectiveTime);
        if (synthetic.length) {
          frames = synthetic.map((frame) => ({
            ...frame,
            graph: baseGraph,
            space2d: baseSpace2d,
            textDoc: baseTextDoc,
          }));
          useSynthetic = true;
          staticPoints = basePoints;
          log("DDN 시간 프레임이 정적이라 점열 기반 프레임으로 변환했습니다.");
        }
      }
      const idx = findNearestFrameIndex(frames, effectiveTime.now ?? effectiveTime.t_min);
      state.space2dRange = computeSpace2dRangeFromFrames(frames);
      state.time.frames = frames;
      state.time.index = idx;
      const frame = frames[idx];
      const graph = frame.graph ?? frames[0].graph;
      const runs = createRunsFromGraph(
        graph,
        { kind: "ddn", text: ddnText },
        { sample: graph.sample, time: effectiveTime },
        { space2d: frame.space2d ?? null, textDoc: frame.textDoc ?? null },
      );
      runs.forEach((run) => {
        run.timeFrames = frames;
        if (useSynthetic && Array.isArray(staticPoints) && staticPoints.length) {
          run.staticPoints = run.graph?.series?.[0]?.points ?? staticPoints;
        }
      });
      if (state.compare.enabled) {
        if (runs.length !== 1) {
          log("비교 모드는 단일 series만 지원합니다.");
          addRuns(runs);
        } else {
          const incoming = runs[0];
          const baseline = state.runs.find((run) => run.id === state.compare.baselineId) ?? null;
          if (!baseline) {
            addRun(incoming);
            setCompareBaseline(incoming);
          } else {
            const check = canOverlayCompare(baseline, incoming);
            if (!check.ok) {
              state.compare.blockReason = check.reason;
              updateCompareStatusUI();
              log(`비교 불가: ${check.reason}`);
            } else {
              state.compare.blockReason = "";
              if (state.compare.variantId) {
                removeRun(state.compare.variantId);
              }
              incoming.compareRole = "variant";
              incoming.layerIndex = (baseline.layerIndex ?? 0) + 1;
              addRun(incoming, { activate: false });
              baseline.compareRole = "baseline";
              baseline.visible = true;
              incoming.visible = true;
              state.compare.variantId = incoming.id;
              setActiveRun(incoming.id);
              syncCompareVisibility();
              updateCompareStatusUI();
            }
          }
        }
      } else if (autoReplace) {
        addRunsAutoReplace(runs);
      } else {
        addRuns(runs);
      }
      applyTimeFrame();
      renderInspector();
      if (built.source === "ddn") {
        log("DDN 실행 완료(시간 프레임: DDN 범위)");
      } else {
        log("DDN 실행 완료(시간 프레임: View Dock 범위)");
      }
      return true;
    }

      const { graph, space2d, textDoc, table, structure } = await fetchGraphFromBridge(baseUrl, ddnText);
      const runs = createRunsFromGraph(
        graph,
        { kind: "ddn", text: ddnText },
        { sample: graph.sample, time: readTimeControls() },
        { space2d: space2d ?? null, textDoc: textDoc ?? null },
      );
      if (state.compare.enabled) {
        if (runs.length !== 1) {
          log("비교 모드는 단일 series만 지원합니다.");
          addRuns(runs);
        } else {
          const incoming = runs[0];
          const baseline = state.runs.find((run) => run.id === state.compare.baselineId) ?? null;
          if (!baseline) {
            addRun(incoming);
            setCompareBaseline(incoming);
          } else {
            const check = canOverlayCompare(baseline, incoming);
            if (!check.ok) {
              state.compare.blockReason = check.reason;
              updateCompareStatusUI();
              log(`비교 불가: ${check.reason}`);
            } else {
              state.compare.blockReason = "";
              if (state.compare.variantId) {
                removeRun(state.compare.variantId);
              }
              incoming.compareRole = "variant";
              incoming.layerIndex = (baseline.layerIndex ?? 0) + 1;
              addRun(incoming, { activate: false });
              baseline.compareRole = "baseline";
              baseline.visible = true;
              incoming.visible = true;
              state.compare.variantId = incoming.id;
              setActiveRun(incoming.id);
              syncCompareVisibility();
              updateCompareStatusUI();
            }
          }
        }
      } else if (autoReplace) {
        addRunsAutoReplace(runs);
      } else {
        addRuns(runs);
      }
      if (space2d) {
        state.space2d = space2d;
        state.space2dRange = computeSpace2dRange(space2d);
      }
      if (textDoc) {
        state.textDoc = textDoc;
      }
      if (table) {
        state.table = table;
        state.tableView.page = 0;
      }
      if (structure) {
        state.structure = { ...structure, selection: null };
        state.structureLayout = null;
      }
      render2dView();
      renderTextView();
      renderTableView();
      renderStructureView();
      renderInspector();
      log("DDN 실행 완료");
      return true;
  } catch (err) {
    finishDdnFrameJob("DDN 프레임: 실패");
    log(err.message);
    return false;
  }
}

function triggerBlobDownload(blob, filename) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

function updateMediaExportStatus(text, level = "") {
  const el = $("media-export-status");
  if (!el) return;
  el.textContent = text;
  el.className = ["hint", level].filter(Boolean).join(" ");
}

function syncMediaExportButtons() {
  const startBtn = $("media-export-start");
  const stopBtn = $("media-export-stop");
  const active = state.mediaExport.mode !== "idle";
  if (startBtn) startBtn.disabled = Boolean(active);
  if (stopBtn) stopBtn.disabled = !active;
}

function resetMediaExportState() {
  if (state.mediaExport.timer) {
    clearTimeout(state.mediaExport.timer);
  }
  if (state.mediaExport.frameTimer) {
    clearInterval(state.mediaExport.frameTimer);
  }
  if (state.mediaExport.stream) {
    state.mediaExport.stream.getTracks().forEach((track) => track.stop());
  }
  state.mediaExport.mode = "idle";
  state.mediaExport.recorder = null;
  state.mediaExport.stream = null;
  state.mediaExport.chunks = [];
  state.mediaExport.mimeType = "";
  state.mediaExport.format = "webm";
  state.mediaExport.timer = null;
  state.mediaExport.frameTimer = null;
  state.mediaExport.startedAt = 0;
  state.mediaExport.canvasId = "";
  state.mediaExport.gifFrames = [];
  state.mediaExport.gifWidth = 0;
  state.mediaExport.gifHeight = 0;
  state.mediaExport.gifDelayCs = 3;
  state.mediaExport.gifCaptureCanvas = null;
  syncMediaExportButtons();
}

function selectMediaRecorderMimeType() {
  const supported = typeof MediaRecorder !== "undefined";
  if (!supported) return null;
  const candidates = ["video/webm;codecs=vp9", "video/webm;codecs=vp8", "video/webm"];
  for (const candidate of candidates) {
    if (
      typeof MediaRecorder.isTypeSupported !== "function" ||
      MediaRecorder.isTypeSupported(candidate)
    ) {
      return candidate;
    }
  }
  return format === "webm" ? "video/webm" : null;
}

function resolveMediaExportCanvas() {
  if (state.activeView === "view-2d") return $("canvas-2d");
  if (state.activeView === "view-structure") return $("structure-canvas");
  if (state.activeView === "view-graph") return $("canvas");
  if (state.viewCombo) return $("canvas");
  return null;
}

function readMediaExportOptions() {
  const format = String($("media-export-format")?.value || "webm").toLowerCase() === "gif"
    ? "gif"
    : "webm";
  const fpsRaw = Number($("media-export-fps")?.value ?? 30);
  const durationRaw = Number($("media-export-duration")?.value ?? 6);
  const fps = Math.min(120, Math.max(1, Number.isFinite(fpsRaw) ? fpsRaw : 30));
  const durationSec = Math.min(120, Math.max(0, Number.isFinite(durationRaw) ? durationRaw : 6));
  return { format, fps, durationSec };
}

function buildGifPalette332() {
  const palette = new Uint8Array(256 * 3);
  for (let i = 0; i < 256; i++) {
    const r = (i >> 5) & 0x07;
    const g = (i >> 2) & 0x07;
    const b = i & 0x03;
    palette[i * 3] = Math.round((r * 255) / 7);
    palette[i * 3 + 1] = Math.round((g * 255) / 7);
    palette[i * 3 + 2] = Math.round((b * 255) / 3);
  }
  return palette;
}

const GIF_PALETTE_332 = buildGifPalette332();

function rgbaToIndexed332(rgbaBytes) {
  const pixels = Math.floor((rgbaBytes?.length ?? 0) / 4);
  const out = new Uint8Array(pixels);
  for (let i = 0; i < pixels; i++) {
    const p = i * 4;
    const r3 = rgbaBytes[p] >> 5;
    const g3 = rgbaBytes[p + 1] >> 5;
    const b2 = rgbaBytes[p + 2] >> 6;
    out[i] = (r3 << 5) | (g3 << 2) | b2;
  }
  return out;
}

function writeGifWord(bytes, value) {
  bytes.push(value & 0xff, (value >> 8) & 0xff);
}

function lzwCompressGifIndices(indices, minCodeSize = 8) {
  const clearCode = 1 << minCodeSize;
  const endCode = clearCode + 1;
  const maxCode = 4095;

  const resetTable = () => {
    const table = new Map();
    for (let i = 0; i < clearCode; i++) {
      table.set(String(i), i);
    }
    return table;
  };

  const output = [];
  let table = resetTable();
  let codeSize = minCodeSize + 1;
  let nextCode = endCode + 1;
  let bitBuffer = 0;
  let bitCount = 0;

  const emitCode = (code) => {
    bitBuffer |= code << bitCount;
    bitCount += codeSize;
    while (bitCount >= 8) {
      output.push(bitBuffer & 0xff);
      bitBuffer >>= 8;
      bitCount -= 8;
    }
  };

  const clearTable = () => {
    table = resetTable();
    codeSize = minCodeSize + 1;
    nextCode = endCode + 1;
  };

  emitCode(clearCode);
  if (!indices.length) {
    emitCode(endCode);
    if (bitCount > 0) output.push(bitBuffer & 0xff);
    return new Uint8Array(output);
  }

  let prefix = String(indices[0]);
  for (let i = 1; i < indices.length; i++) {
    const k = indices[i];
    const candidate = `${prefix},${k}`;
    if (table.has(candidate)) {
      prefix = candidate;
      continue;
    }

    emitCode(table.get(prefix) ?? 0);
    if (nextCode <= maxCode) {
      table.set(candidate, nextCode);
      nextCode += 1;
      if (nextCode === 1 << codeSize && codeSize < 12) {
        codeSize += 1;
      }
    } else {
      emitCode(clearCode);
      clearTable();
    }
    prefix = String(k);
  }

  emitCode(table.get(prefix) ?? 0);
  emitCode(endCode);
  if (bitCount > 0) output.push(bitBuffer & 0xff);
  return new Uint8Array(output);
}

function appendGifSubBlocks(bytes, payload) {
  let offset = 0;
  while (offset < payload.length) {
    const size = Math.min(255, payload.length - offset);
    bytes.push(size);
    for (let i = 0; i < size; i++) {
      bytes.push(payload[offset + i]);
    }
    offset += size;
  }
  bytes.push(0x00);
}

function encodeGifFromIndexedFrames(width, height, frames, delayCs) {
  const bytes = [];
  const header = "GIF89a";
  for (let i = 0; i < header.length; i++) {
    bytes.push(header.charCodeAt(i));
  }

  writeGifWord(bytes, width);
  writeGifWord(bytes, height);
  bytes.push(0xf7); // global color table(256)
  bytes.push(0x00); // background color index
  bytes.push(0x00); // pixel aspect ratio
  for (let i = 0; i < GIF_PALETTE_332.length; i++) {
    bytes.push(GIF_PALETTE_332[i]);
  }

  // Netscape loop extension (infinite loop)
  bytes.push(
    0x21,
    0xff,
    0x0b,
    0x4e,
    0x45,
    0x54,
    0x53,
    0x43,
    0x41,
    0x50,
    0x45,
    0x32,
    0x2e,
    0x30,
    0x03,
    0x01,
    0x00,
    0x00,
    0x00,
  );

  const frameDelay = Math.max(1, Math.min(65535, delayCs));
  for (const indices of frames) {
    bytes.push(0x21, 0xf9, 0x04, 0x00);
    writeGifWord(bytes, frameDelay);
    bytes.push(0x00, 0x00);

    bytes.push(0x2c);
    writeGifWord(bytes, 0);
    writeGifWord(bytes, 0);
    writeGifWord(bytes, width);
    writeGifWord(bytes, height);
    bytes.push(0x00); // no local color table

    bytes.push(0x08); // LZW min code size
    const compressed = lzwCompressGifIndices(indices, 8);
    appendGifSubBlocks(bytes, compressed);
  }

  bytes.push(0x3b);
  return new Uint8Array(bytes);
}

function prepareGifCaptureCanvas(sourceCanvas) {
  const maxSide = 360;
  const maxSrcSide = Math.max(sourceCanvas.width || 0, sourceCanvas.height || 0, 1);
  const scale = Math.min(1, maxSide / maxSrcSide);
  const width = Math.max(1, Math.round((sourceCanvas.width || 1) * scale));
  const height = Math.max(1, Math.round((sourceCanvas.height || 1) * scale));
  const captureCanvas = document.createElement("canvas");
  captureCanvas.width = width;
  captureCanvas.height = height;
  return captureCanvas;
}

function captureGifIndexedFrame(sourceCanvas, captureCanvas) {
  const ctx = captureCanvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) return null;
  ctx.clearRect(0, 0, captureCanvas.width, captureCanvas.height);
  ctx.drawImage(sourceCanvas, 0, 0, captureCanvas.width, captureCanvas.height);
  const image = ctx.getImageData(0, 0, captureCanvas.width, captureCanvas.height);
  return rgbaToIndexed332(image.data);
}

function startGifExport(canvas, fps, durationSec) {
  const startedAt = Date.now();
  const captureCanvas = prepareGifCaptureCanvas(canvas);
  const delayCs = Math.max(1, Math.round(100 / fps));
  const intervalMs = Math.max(16, Math.round(1000 / fps));
  const maxFrames = 600;

  state.mediaExport.mode = "gif";
  state.mediaExport.format = "gif";
  state.mediaExport.startedAt = startedAt;
  state.mediaExport.canvasId = canvas.id || "canvas";
  state.mediaExport.gifFrames = [];
  state.mediaExport.gifWidth = captureCanvas.width;
  state.mediaExport.gifHeight = captureCanvas.height;
  state.mediaExport.gifDelayCs = delayCs;
  state.mediaExport.gifCaptureCanvas = captureCanvas;
  syncMediaExportButtons();

  const capture = () => {
    const frame = captureGifIndexedFrame(canvas, captureCanvas);
    if (!frame) return;
    if (state.mediaExport.gifFrames.length >= maxFrames) {
      stopMediaExport({ reason: "프레임 한도 도달" });
      return;
    }
    state.mediaExport.gifFrames.push(frame);
  };

  capture();
  state.mediaExport.frameTimer = setInterval(capture, intervalMs);
  if (durationSec > 0) {
    state.mediaExport.timer = setTimeout(() => {
      stopMediaExport({ reason: "자동 중지" });
    }, durationSec * 1000);
  }
  updateMediaExportStatus(
    `내보내기 상태: 녹화 중 (GIF ${fps}fps${durationSec > 0 ? `, ${durationSec}s` : ""})`,
    "ok",
  );
  log(`미디어 녹화 시작: GIF ${fps}fps`);
}

function startWebmExport(canvas, fps, durationSec) {
  if (typeof MediaRecorder === "undefined") {
    updateMediaExportStatus("내보내기 상태: 이 브라우저는 MediaRecorder를 지원하지 않습니다.", "error");
    log("MediaRecorder 미지원 브라우저입니다.");
    return;
  }
  if (typeof canvas.captureStream !== "function") {
    updateMediaExportStatus("내보내기 상태: 캔버스 captureStream 미지원입니다.", "error");
    log("캔버스 captureStream 미지원 환경입니다.");
    return;
  }
  const mimeType = selectMediaRecorderMimeType();
  if (!mimeType) {
    updateMediaExportStatus("내보내기 상태: WebM 녹화를 지원하지 않습니다.", "error");
    log("WebM 녹화 미지원 환경입니다.");
    return;
  }
  const stream = canvas.captureStream(fps);
  const recorder = new MediaRecorder(stream, { mimeType });
  const chunks = [];
  const startedAt = Date.now();

  state.mediaExport.mode = "recorder";
  state.mediaExport.recorder = recorder;
  state.mediaExport.stream = stream;
  state.mediaExport.chunks = chunks;
  state.mediaExport.mimeType = mimeType;
  state.mediaExport.format = "webm";
  state.mediaExport.startedAt = startedAt;
  state.mediaExport.canvasId = canvas.id || "canvas";
  syncMediaExportButtons();

  recorder.ondataavailable = (event) => {
    if (event.data && event.data.size > 0) {
      chunks.push(event.data);
    }
  };

  recorder.onerror = (event) => {
    const message = event?.error?.message || "녹화 오류";
    updateMediaExportStatus(`내보내기 상태: 실패 (${message})`, "error");
    log(`미디어 녹화 실패: ${message}`);
  };

  recorder.onstop = () => {
    const tookMs = Date.now() - startedAt;
    const stamp = new Date().toISOString().replace(/[:.]/g, "-");
    if (!chunks.length) {
      updateMediaExportStatus("내보내기 상태: 실패 (기록된 프레임 없음)", "error");
      resetMediaExportState();
      return;
    }
    const blob = new Blob(chunks, { type: state.mediaExport.mimeType || mimeType });
    const filename = `seamgrim_${state.mediaExport.canvasId}_${stamp}.webm`;
    triggerBlobDownload(blob, filename);
    updateMediaExportStatus(`내보내기 상태: 완료 (WEBM / ${(tookMs / 1000).toFixed(1)}s)`, "ok");
    log(`미디어 내보내기 완료: ${filename}`);
    resetMediaExportState();
  };

  recorder.start(200);
  if (durationSec > 0) {
    state.mediaExport.timer = setTimeout(() => {
      stopMediaExport({ reason: "자동 중지" });
    }, durationSec * 1000);
  }
  updateMediaExportStatus(
    `내보내기 상태: 녹화 중 (WEBM ${fps}fps${durationSec > 0 ? `, ${durationSec}s` : ""})`,
    "ok",
  );
  log(`미디어 녹화 시작: WEBM ${fps}fps`);
}

function stopMediaExport(options = {}) {
  const mode = state.mediaExport.mode;
  if (mode === "idle") return;
  if (state.mediaExport.timer) {
    clearTimeout(state.mediaExport.timer);
    state.mediaExport.timer = null;
  }
  if (state.mediaExport.frameTimer) {
    clearInterval(state.mediaExport.frameTimer);
    state.mediaExport.frameTimer = null;
  }

  const reason = options.reason || "녹화 중지";
  if (mode === "recorder") {
    const recorder = state.mediaExport.recorder;
    if (!recorder) {
      resetMediaExportState();
      return;
    }
    if (recorder.state !== "inactive") {
      updateMediaExportStatus(`내보내기 상태: ${reason} 처리 중...`);
      recorder.stop();
      return;
    }
    resetMediaExportState();
    return;
  }

  if (mode === "gif") {
    const frames = state.mediaExport.gifFrames;
    if (!frames.length) {
      updateMediaExportStatus("내보내기 상태: 실패 (기록된 프레임 없음)", "error");
      resetMediaExportState();
      return;
    }
    updateMediaExportStatus(`내보내기 상태: ${reason} 처리 중...`);
    const bytes = encodeGifFromIndexedFrames(
      state.mediaExport.gifWidth,
      state.mediaExport.gifHeight,
      frames,
      state.mediaExport.gifDelayCs,
    );
    const blob = new Blob([bytes], { type: "image/gif" });
    const stamp = new Date().toISOString().replace(/[:.]/g, "-");
    const filename = `seamgrim_${state.mediaExport.canvasId}_${stamp}.gif`;
    triggerBlobDownload(blob, filename);
    const tookMs = Date.now() - state.mediaExport.startedAt;
    updateMediaExportStatus(`내보내기 상태: 완료 (GIF / ${(tookMs / 1000).toFixed(1)}s)`, "ok");
    log(`미디어 내보내기 완료: ${filename}`);
    resetMediaExportState();
    return;
  }
}

function startMediaExport() {
  if (state.mediaExport.mode !== "idle") {
    log("이미 녹화 중입니다.");
    return;
  }
  const canvas = resolveMediaExportCanvas();
  if (!canvas) {
    updateMediaExportStatus("내보내기 상태: 캔버스 뷰(그래프/2D/구조)에서만 녹화할 수 있습니다.", "error");
    log("녹화 대상 캔버스가 없습니다. 그래프/2D/구조 뷰로 전환하세요.");
    return;
  }

  const { format, fps, durationSec } = readMediaExportOptions();
  try {
    if (format === "gif") {
      startGifExport(canvas, fps, durationSec);
      return;
    }
    startWebmExport(canvas, fps, durationSec);
  } catch (err) {
    updateMediaExportStatus(
      `내보내기 상태: 실패 (${err?.message || err})`,
      "error",
    );
    log(`미디어 내보내기 시작 실패: ${err?.message || err}`);
    resetMediaExportState();
  }
}

function exportDdnText() {
  const ddnText = $("ddn-editor").value.trim();
  if (!ddnText) {
    log("내보낼 DDN 텍스트가 없습니다.");
    return;
  }
  const meta = extractDdnMeta(ddnText);
  const filename = meta.name ? `${meta.name}.ddn` : "seamgrim.ddn";
  const blob = new Blob([ddnText], { type: "text/plain" });
  triggerBlobDownload(blob, filename);
}

async function importDdnText() {
  const file = $("ddn-import-file").files?.[0];
  if (!file) {
    log("불러올 DDN 파일을 선택하세요.");
    return;
  }
  const text = await file.text();
  $("ddn-editor").value = text;
  updateSceneFromDdn(text);
  registerDdnInput(text, { label: file.name });
  updateDdnTimeStatus();
  refreshDdnControlsFromText(text);
  log("DDN 불러오기 완료");
}

async function loadGraphFile() {
  const file = $("graph-file").files?.[0];
  if (!file) {
    log("불러올 파일을 선택하세요.");
    return;
  }
  try {
    const text = await file.text();
    const data = validateGraphData(JSON.parse(text));
    const runs = createRunsFromGraph(data, { kind: "graph", text: "" }, { sample: data.sample });
    const viewConfig = getViewConfigFromData(data, state.runs);
    state.viewConfig = viewConfig;
    applyViewControlsFromGraph(data, viewConfig);
    applyViewToGraph(data, viewConfig);
    addRuns(runs);
  } catch (err) {
    log(err.message);
  }
}

async function loadTableFile() {
  const file = $("table-file").files?.[0];
  if (!file) {
    log("불러올 표 파일을 선택하세요.");
    return;
  }
  try {
    let tableData = null;
    if (file.name.toLowerCase().endsWith(".csv")) {
      const text = await file.text();
      const rows = parseCsv(text);
      if (!rows.length) throw new Error("CSV 내용이 비었습니다.");
      const headers = rows.shift();
      const columns = headers.map((label, idx) => ({ key: `c${idx}`, label: label || `c${idx + 1}` }));
      const rowObjects = rows.map((row) => {
        const obj = {};
        headers.forEach((_h, idx) => {
          const raw = row[idx] ?? "";
          const num = Number(raw);
          obj[`c${idx}`] = Number.isFinite(num) && raw.trim() !== "" ? num : raw;
        });
        return obj;
      });
      tableData = { schema: "seamgrim.table.v0", columns, rows: rowObjects };
    } else {
      const text = await file.text();
      const data = JSON.parse(text);
      if (data.schema && data.schema !== "seamgrim.table.v0") {
        throw new Error("지원하지 않는 표 스키마입니다.");
      }
      tableData = normalizeTableData(data) ?? data;
      if (!tableData || !Array.isArray(tableData.columns) || !Array.isArray(tableData.rows)) {
        throw new Error("표 데이터가 올바르지 않습니다.");
      }
    }
    state.table = tableData;
    state.tableView.page = 0;
    document.querySelector('.view-group .tab[data-tab="view-table"]')?.click();
    renderTableView();
    log("표 불러오기 완료");
  } catch (err) {
    log(err.message);
  }
}

async function loadTextFile() {
  const file = $("text-file").files?.[0];
  if (!file) {
    log("불러올 문서를 선택하세요.");
    return;
  }
  try {
    const text = await file.text();
    if (file.name.toLowerCase().endsWith(".json")) {
      const data = JSON.parse(text);
      if (data.schema && data.schema !== "seamgrim.text.v0") {
        throw new Error("지원하지 않는 문서 스키마입니다.");
      }
      state.textDoc = { content: data.content ?? "", format: data.format ?? "markdown" };
    } else {
      const isPlain = file.name.toLowerCase().endsWith(".txt");
      state.textDoc = { content: text, format: isPlain ? "plain" : "markdown" };
    }
    document.querySelector('.view-group .tab[data-tab="view-text"]')?.click();
    renderTextView();
    log("문서 불러오기 완료");
  } catch (err) {
    log(err.message);
  }
}

async function loadStructureFile() {
  const file = $("structure-file").files?.[0];
  if (!file) {
    log("불러올 구조 파일을 선택하세요.");
    return;
  }
  try {
    const text = await file.text();
    const data = JSON.parse(text);
    if (data.schema && data.schema !== "seamgrim.structure.v0") {
      throw new Error("지원하지 않는 구조 스키마입니다.");
    }
    const checked = validateStructureData(data);
    state.structure = { ...checked, selection: null };
    if (checked.layout?.type) {
      state.structureView.layout = checked.layout.type;
      $("structure-layout").value = checked.layout.type;
    }
    document.querySelector('.view-group .tab[data-tab="view-structure"]')?.click();
    renderStructureView();
    log("구조 불러오기 완료");
  } catch (err) {
    log(err.message);
  }
}

async function loadSpace2dFile() {
  const file = $("space2d-file").files?.[0];
  if (!file) {
    log("불러올 2D 파일을 선택하세요.");
    return;
  }
  try {
    const text = await file.text();
    const data = validateSpace2dData(JSON.parse(text));
    state.space2d = data;
    state.space2dRange = computeSpace2dRange(data);
    document.querySelector('.view-group .tab[data-tab="view-2d"]')?.click();
    render2dView();
    log("2D 보개 불러오기 완료");
  } catch (err) {
    log(err.message);
  }
}

function exportGraph() {
  const run = getActiveRun();
  if (!run?.graph) {
    log("내보낼 그래프가 없습니다.");
    return;
  }
  const graph = JSON.parse(JSON.stringify(run.graph));
  if (state.viewConfig) applyViewToGraph(graph, state.viewConfig);
  const blob = new Blob([JSON.stringify(graph, null, 2)], { type: "application/json" });
  triggerBlobDownload(blob, "seamgrim.graph.v0.json");
}

function createSnapshotFromRun(run) {
  return {
    schema: "seamgrim.snapshot.v0",
    ts: new Date().toISOString(),
    note: run.label ?? "snapshot",
    run: {
      id: run.id,
      label: run.label,
      layer_index: run.layerIndex,
      source: run.source,
      inputs: run.inputs,
      graph: run.graph,
      hash: {
        input: run.graph?.meta?.source_input_hash ?? "",
        result: run.graph?.meta?.result_hash ?? "",
      },
    },
    layers: state.runs.map((item) => ({
      id: item.id,
      label: item.label,
      layer_index: item.layerIndex,
      visible: item.visible,
    })),
  };
}

function saveSnapshot() {
  const run = getActiveRun();
  if (!run?.graph) {
    log("스냅샷을 저장할 그래프가 없습니다.");
    return;
  }
  const snap = createSnapshotFromRun(run);
  state.snapshots.unshift(snap);
  state.schemaStatus.snapshot = snap.schema;
  renderSnapshots();
}

async function loadSnapshots() {
  const file = $("snapshot-file").files?.[0];
  if (!file) {
    log("불러올 파일을 선택하세요.");
    return;
  }
  try {
    const text = await file.text();
    const data = JSON.parse(text);
    if (data.schema === "seamgrim.snapshot.v0") {
      state.snapshots.unshift(data);
      renderSnapshots();
      state.schemaStatus.snapshot = data.schema;
      log("스냅샷 불러오기 완료");
      return;
    }
    if (Array.isArray(data.snapshots)) {
      state.snapshots = data.snapshots.concat(state.snapshots);
      renderSnapshots();
      log("스냅샷 묶음 불러오기 완료");
      return;
    }
    throw new Error("스냅샷 스키마가 올바르지 않습니다.");
  } catch (err) {
    log(err.message);
  }
}

function exportSnapshots() {
  if (!state.snapshots.length) {
    log("내보낼 스냅샷이 없습니다.");
    return;
  }
  const payload = state.snapshots[0];
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: "application/json" });
  triggerBlobDownload(blob, "seamgrim.snapshot.v0.json");
}

function exportSession() {
  const session = {
    schema: "seamgrim.session.v0",
    ts: new Date().toISOString(),
    lesson: state.lessons.activeId,
    ddn_text: $("ddn-editor").value,
    controls: {
      meta: state.controls.metaRaw || null,
      values: state.controls.values || {},
    },
    inputs: {
      registry: state.inputRegistry.items,
      selected_id: state.inputRegistry.selectedId,
    },
    text_doc: state.textDoc,
    space2d: state.space2d,
    space2d_view: {
      auto: state.space2dView.auto,
      range: state.space2dView.range,
      panX: state.space2dView.panX,
      panY: state.space2dView.panY,
      zoom: state.space2dView.zoom,
    },
    wasm_lens: {
      enabled: Boolean(state.wasm.lens.enabled),
      x_key: String(state.wasm.lens.xKey ?? "__tick__"),
      y_key: String(state.wasm.lens.yKey ?? ""),
      y2_key: String(state.wasm.lens.y2Key ?? ""),
      preset_id: String(state.wasm.lens.presetId ?? "custom"),
    },
    time: readTimeControls(),
    view: state.viewConfig,
    view_combo: {
      enabled: state.viewCombo,
      layout: state.viewComboLayout,
      overlay_order: state.viewOverlayOrder,
    },
    table_view: state.tableView,
    structure_view: state.structureView,
    runs: state.runs.map((run) => ({
      id: run.id,
      label: run.label,
      visible: run.visible,
      layer_index: run.layerIndex,
      compare_role: run.compareRole ?? null,
      source: run.source,
      inputs: run.inputs,
      graph: run.graph,
      space2d: run.space2d ?? null,
      text_doc: run.textDoc ?? null,
    })),
    active_run_id: state.activeRunId,
    compare: {
      enabled: state.compare.enabled,
      baseline_id: state.compare.baselineId,
      variant_id: state.compare.variantId,
    },
  };
  state.schemaStatus.session = session.schema;
  const blob = new Blob([JSON.stringify(session, null, 2)], { type: "application/json" });
  triggerBlobDownload(blob, "seamgrim.session.v0.json");
}

function restoreInputRegistryFromSession(data) {
  state.inputRegistry.items = [];
  state.inputRegistry.selectedId = null;
  if (Array.isArray(data.inputs?.registry) && data.inputs.registry.length) {
    state.inputRegistry.items = data.inputs.registry;
    state.inputRegistry.selectedId = data.inputs.selected_id ?? null;
    renderInputRegistry();
    return;
  }
  if (data.lesson) {
    const label = data.lesson;
    upsertInputItem({
      id: `lesson:${data.lesson}`,
      type: "lesson",
      label,
      payload: { lesson_id: data.lesson },
      derived_ddn: data.ddn_text ?? null,
    });
  }
  if (data.ddn_text) {
    const meta = extractDdnMeta(data.ddn_text);
    upsertInputItem({
      id: "ddn:editor",
      type: "ddn",
      label: meta.name || "DDN",
      payload: { text: data.ddn_text },
    });
  }
  if (data.lesson) {
    state.inputRegistry.selectedId = `lesson:${data.lesson}`;
  } else if (data.ddn_text) {
    state.inputRegistry.selectedId = "ddn:editor";
  }
  renderInputRegistry();
}

async function loadSession() {
  const file = $("session-file").files?.[0];
  if (!file) {
    log("불러올 세션 파일을 선택하세요.");
    return;
  }
  try {
    const text = await file.text();
    const data = JSON.parse(text);
    if (data.schema !== "seamgrim.session.v0") {
      throw new Error("세션 스키마가 올바르지 않습니다.");
    }
    state.schemaStatus.session = data.schema;
    $("ddn-editor").value = data.ddn_text ?? "";
    refreshDdnControlsFromText($("ddn-editor").value);
    if (data.controls?.values) {
      state.controls.values = { ...state.controls.values, ...data.controls.values };
      refreshDdnControlsFromText($("ddn-editor").value, { preserveValues: true });
    }
    restoreInputRegistryFromSession(data);
    if (data.time) {
      $("time-enabled").checked = data.time.enabled ?? false;
      if (Number.isFinite(data.time.t_min)) $("t-min").value = data.time.t_min;
      if (Number.isFinite(data.time.t_max)) $("t-max").value = data.time.t_max;
      if (Number.isFinite(data.time.step)) $("t-step").value = data.time.step;
      if (Number.isFinite(data.time.now)) $("t-now").value = data.time.now;
      if (Number.isFinite(data.time.interval)) $("t-interval").value = data.time.interval;
      $("time-loop").checked = data.time.loop ?? true;
      applyTimeModeUi();
      syncTimeCursorRange();
    }
    if (data.view) {
      state.viewConfig = data.view;
      $("view-auto").checked = data.view.auto ?? true;
      $("view-x-min").value = data.view.x_min ?? "";
      $("view-x-max").value = data.view.x_max ?? "";
      $("view-y-min").value = data.view.y_min ?? "";
      $("view-y-max").value = data.view.y_max ?? "";
      $("pan-x").value = data.view.panX ?? data.view.pan_x ?? 0;
      $("pan-y").value = data.view.panY ?? data.view.pan_y ?? 0;
      $("zoom").value = data.view.zoom ?? 1;
      $("toggle-grid").checked = data.view.showGrid ?? true;
      $("toggle-axis").checked = data.view.showAxis ?? true;
      updateViewInputsEnabled();
    }
    if (data.space2d_view) {
      state.space2dView = {
        ...state.space2dView,
        ...data.space2d_view,
        range: data.space2d_view.range ?? state.space2dView.range,
      };
      syncSpace2dViewControlValues();
    }
    if (data.wasm_lens && typeof data.wasm_lens === "object") {
      state.wasm.lens.enabled = Boolean(data.wasm_lens.enabled);
      state.wasm.lens.xKey = String(data.wasm_lens.x_key ?? "__tick__");
      state.wasm.lens.yKey = String(data.wasm_lens.y_key ?? "");
      state.wasm.lens.y2Key = String(data.wasm_lens.y2_key ?? "");
      state.wasm.lens.presetId = String(data.wasm_lens.preset_id ?? state.wasm.lens.presetId ?? "custom");
      clearWasmLensTimeline();
      if ($("wasm-lens-enable")) $("wasm-lens-enable").checked = state.wasm.lens.enabled;
      const lensPresetSelect = $("wasm-lens-preset");
      if (lensPresetSelect) {
        const hasPreset = Array.from(lensPresetSelect.options).some(
          (opt) => opt.value === state.wasm.lens.presetId,
        );
        if (!hasPreset) state.wasm.lens.presetId = "custom";
        lensPresetSelect.value = state.wasm.lens.presetId;
      }
      updateWasmLensSelectors(state.wasm.lastObservation);
      syncWasmLensConfigFromDom();
      state.wasm.lens.runs = buildWasmLensRuns();
      renderWasmChannelSummary(state.wasm.lastObservation);
      renderWasmChannelList(state.wasm.lastObservation);
      updateWasmStatus();
    }
    if (data.view_combo) {
      state.viewCombo = data.view_combo.enabled ?? state.viewCombo;
      state.viewComboLayout = data.view_combo.layout ?? state.viewComboLayout;
      state.viewOverlayOrder = data.view_combo.overlay_order ?? state.viewOverlayOrder;
      if ($("view-combo-toggle")) $("view-combo-toggle").checked = state.viewCombo;
      if ($("view-combo-layout")) $("view-combo-layout").value = state.viewComboLayout;
      if ($("view-combo-overlay-order")) $("view-combo-overlay-order").value = state.viewOverlayOrder;
      updateViewComboLayout();
    }
    if (data.table_view) {
      state.tableView = { ...state.tableView, ...data.table_view };
      if ($("table-page-size")) $("table-page-size").value = state.tableView.pageSize ?? 50;
      if ($("table-precision")) $("table-precision").value = state.tableView.precision ?? 3;
    }
    if (data.structure_view) {
      state.structureView = { ...state.structureView, ...data.structure_view };
      if ($("structure-layout")) $("structure-layout").value = state.structureView.layout ?? "circle";
      if ($("structure-node-size")) $("structure-node-size").value = state.structureView.nodeSize ?? 10;
      if ($("structure-show-labels"))
        $("structure-show-labels").checked = state.structureView.showLabels ?? true;
    }
    if (data.text_doc) {
      state.textDoc = data.text_doc;
    }
    if (data.space2d) {
      try {
        state.space2d = validateSpace2dData(data.space2d);
        state.space2dRange = computeSpace2dRange(state.space2d);
      } catch (err) {
        log("세션 space2d 복원 실패: " + err.message);
      }
    }
    state.layerCounter = 0;
    state.runs = (data.runs ?? []).map((raw) => {
      const run = createRun({
        graph: raw.graph,
        source: raw.source,
        inputs: raw.inputs,
        space2d: raw.space2d ?? null,
        textDoc: raw.text_doc ?? null,
      });
      run.id = raw.id ?? run.id;
      run.label = raw.label ?? run.label;
      run.visible = raw.visible ?? true;
      if (Number.isFinite(raw.layer_index)) {
        run.layerIndex = raw.layer_index;
      }
      if (raw.compare_role) {
        run.compareRole = raw.compare_role;
      }
      return run;
    });
    ensureLayerIndices(state.runs);
    state.activeRunId = data.active_run_id ?? sortRunsForList(state.runs)[0]?.id ?? null;
    if (state.activeRunId) {
      setActiveRun(state.activeRunId);
    }
    if (data.compare?.enabled) {
      state.compare.enabled = true;
      const baseline = state.runs.find((run) => run.compareRole === "baseline");
      const variant = state.runs.find((run) => run.compareRole === "variant");
      state.compare.baselineId = baseline?.id ?? data.compare.baseline_id ?? null;
      state.compare.variantId = variant?.id ?? data.compare.variant_id ?? null;
      if (baseline?.graph) {
        state.compare.axisSig = getAxisSignature(baseline.graph);
        state.compare.seriesId = getRunSeriesKey(baseline);
      }
      syncCompareVisibility();
    }
    renderOverlayList();
    renderRunList();
    renderInspector();
    renderAll();
    updateCompareStatusUI();
    log("세션 불러오기 완료");
  } catch (err) {
    log(err.message);
  }
}
function summarizeFile(text) {
  const summary = {
    size: text.length,
    hash: hashString(text),
    schema: null,
  };
  try {
    const data = JSON.parse(text);
    summary.schema = data.schema ?? data.kind ?? null;
    if (data.bogae_hash) summary.bogae_hash = data.bogae_hash;
    if (data.state_hash) summary.state_hash = data.state_hash;
    if (data.trace_hash) summary.trace_hash = data.trace_hash;
    if (data.kind) summary.kind = data.kind;
  } catch (_) {
    summary.schema = null;
  }
  return summary;
}

function validateSam(obj) {
  const warnings = [];
  if (!obj || typeof obj !== "object") {
    warnings.push("샘 JSON이 객체가 아닙니다.");
    return { ok: false, warnings };
  }
  const schema =
    typeof obj.schema === "string"
      ? obj.schema
      : typeof obj.kind === "string"
        ? obj.kind
        : null;
  if (!schema) {
    warnings.push("샘 schema 누락.");
  } else if (!SAM_SCHEMA_ALIASES.has(schema)) {
    warnings.push(`샘 schema 미확인: ${schema}`);
  }
  const records = Array.isArray(obj.records) ? obj.records : null;
  const netEvents = Array.isArray(obj.net_events) ? obj.net_events : null;
  if (!records && !netEvents) {
    warnings.push("records/net_events가 없습니다.");
  }
  return { ok: warnings.length === 0, warnings, schema };
}

function validateGeoul(obj) {
  const warnings = [];
  if (!obj || typeof obj !== "object") {
    warnings.push("거울 JSON이 객체가 아닙니다.");
    return { ok: false, warnings };
  }
  const schema = typeof obj.schema === "string" ? obj.schema : null;
  if (!schema) {
    warnings.push("거울 schema 누락.");
  } else if (!GEOUL_SCHEMA_ALIASES.has(schema)) {
    warnings.push(`거울 schema 미확인: ${schema}`);
  }
  const frames = Array.isArray(obj.frames) ? obj.frames : null;
  if (schema === "bogae_web_playback_v1") {
    if (!frames) warnings.push("frames 배열이 없습니다.");
  } else if (!obj.state_hash && !obj.trace_hash && !obj.bogae_hash && !frames) {
    warnings.push("hash/frames 정보가 없습니다.");
  }
  return { ok: warnings.length === 0, warnings, schema };
}

async function loadSamFile() {
  const file = $("sam-file").files?.[0];
  if (!file) {
    log("샘 파일을 선택하세요.");
    return;
  }
  const text = await file.text();
  const summary = summarizeFile(text);
  let obj = null;
  try {
    obj = JSON.parse(text);
  } catch (_) {
    obj = null;
  }
  const validation = obj ? validateSam(obj) : { ok: false, warnings: ["샘 JSON 파싱 실패"] };
  const records = obj && Array.isArray(obj.records) ? obj.records : null;
  const netEvents = obj && Array.isArray(obj.net_events) ? obj.net_events : null;
  const recordCount = records ? records.length : netEvents ? netEvents.length : null;
  const recordLabel = records ? "records" : netEvents ? "net_events" : "records";
  const inputKeys = new Set();
  if (records) {
    records.forEach((record) => {
      if (record && typeof record.inputs === "object" && record.inputs) {
        Object.keys(record.inputs).forEach((key) => inputKeys.add(key));
      }
    });
  }
  if (netEvents) {
    netEvents.forEach((event) => {
      if (event && typeof event.inputs === "object" && event.inputs) {
        Object.keys(event.inputs).forEach((key) => inputKeys.add(key));
      }
    });
  }
  state.sam = {
    name: file.name,
    summary,
    validation,
    recordCount,
    recordLabel,
    inputKeyCount: inputKeys.size,
    inputKeySample: Array.from(inputKeys).slice(0, 5),
  };
  renderSamSummary();
  log("sam 로드 완료");
}

async function loadGeoulFile() {
  const file = $("geoul-file").files?.[0];
  if (!file) {
    log("거울 파일을 선택하세요.");
    return;
  }
  const text = await file.text();
  const summary = summarizeFile(text);
  let obj = null;
  try {
    obj = JSON.parse(text);
  } catch (_) {
    obj = null;
  }
  const validation = obj ? validateGeoul(obj) : { ok: false, warnings: ["거울 JSON 파싱 실패"] };
  const frames = obj && Array.isArray(obj.frames) ? obj.frames : null;
  const frameCount = frames ? frames.length : null;
  const frameRange = frames?.length
    ? {
        min: frames[0]?.madi ?? 0,
        max: frames[frames.length - 1]?.madi ?? frames.length - 1,
      }
    : null;
  state.geoul = {
    name: file.name,
    summary,
    validation,
    frameCount,
    frameRange,
    hashes: {
      state: obj?.state_hash,
      trace: obj?.trace_hash,
      bogae: obj?.bogae_hash,
    },
  };
  renderGeoulSummary();
  log("geoul 로드 완료");
}

function saveDdnPreset() {
  const ddnText = $("ddn-editor").value.trim();
  if (!ddnText) {
    log("저장할 DDN 텍스트가 없습니다.");
    return;
  }
  const metaInfo = extractDdnMeta(ddnText);
  const time = readTimeControls();
  const view = state.viewConfig;
  const updateMeta = getUpdateMetaFromControls();
  const preset = {
    id: `ddn-${Date.now()}`,
    ts: new Date().toISOString(),
    note: metaInfo.name || "DDN preset",
    ddnText,
    meta: {
      time,
      view,
      update: updateMeta.update,
      tick: updateMeta.tick,
    },
  };
  state.ddnPresets.unshift(preset);
  renderDdnPresets();
  log("DDN 프리셋 저장 완료");
}

function applyPresetToControls(preset) {
  const meta = preset.meta ?? {};
  if (meta.time) {
    $("time-enabled").checked = meta.time.enabled ?? false;
    if (Number.isFinite(meta.time.t_min)) $("t-min").value = meta.time.t_min;
    if (Number.isFinite(meta.time.t_max)) $("t-max").value = meta.time.t_max;
    if (Number.isFinite(meta.time.step)) $("t-step").value = meta.time.step;
    if (Number.isFinite(meta.time.now)) $("t-now").value = meta.time.now;
    if (Number.isFinite(meta.time.interval)) $("t-interval").value = meta.time.interval;
    $("time-loop").checked = meta.time.loop ?? true;
    applyTimeModeUi();
    syncTimeCursorRange();
    stopTimePlayback();
  }
  if (meta.view) {
    state.viewConfig = meta.view;
    $("view-auto").checked = meta.view.auto ?? true;
    $("view-x-min").value = meta.view.x_min ?? "";
    $("view-x-max").value = meta.view.x_max ?? "";
    $("view-y-min").value = meta.view.y_min ?? "";
    $("view-y-max").value = meta.view.y_max ?? "";
    $("pan-x").value = meta.view.panX ?? meta.view.pan_x ?? 0;
    $("pan-y").value = meta.view.panY ?? meta.view.pan_y ?? 0;
    $("zoom").value = meta.view.zoom ?? 1;
    updateViewInputsEnabled();
  }
  if (meta.update) $("update-mode").value = meta.update;
  if (Number.isFinite(meta.tick)) {
    $("update-tick").value = meta.tick;
  } else {
    $("update-tick").value = "";
  }
}

function renderDdnPresets() {
  const list = $("ddn-preset-list");
  list.innerHTML = "";
  state.ddnPresets.forEach((preset) => {
    const li = document.createElement("li");
    const text = document.createElement("span");
    text.textContent = `${preset.ts} | ${preset.note}`;
    const loadBtn = document.createElement("button");
    loadBtn.textContent = "불러오기";
    loadBtn.addEventListener("click", () => {
      $("ddn-editor").value = preset.ddnText ?? "";
      applyPresetToControls(preset);
      refreshDdnControlsFromText($("ddn-editor").value);
      log("프리셋 적용 완료");
    });
    const deleteBtn = document.createElement("button");
    deleteBtn.textContent = "삭제";
    deleteBtn.addEventListener("click", () => {
      state.ddnPresets = state.ddnPresets.filter((item) => item.id !== preset.id);
      renderDdnPresets();
    });
    li.appendChild(text);
    li.appendChild(loadBtn);
    li.appendChild(deleteBtn);
    list.appendChild(li);
  });
}

function exportDdnPresets() {
  const payload = {
    schema: "seamgrim.ddn_presets.v0",
    presets: state.ddnPresets,
  };
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "seamgrim_ddn_presets.json";
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

async function importDdnPresets() {
  const file = $("ddn-preset-file").files?.[0];
  if (!file) {
    log("불러올 프리셋 파일을 선택하세요.");
    return;
  }
  try {
    const text = await file.text();
    const data = JSON.parse(text);
    if (data.schema !== "seamgrim.ddn_presets.v0" || !Array.isArray(data.presets)) {
      throw new Error("프리셋 스키마가 올바르지 않습니다.");
    }
    state.ddnPresets = data.presets;
    renderDdnPresets();
    log("DDN 프리셋 불러오기 완료");
  } catch (err) {
    log(err.message);
  }
}

function updateDockVisibility() {
  const viewKey = (state.activeView || "view-graph").replace("view-", "");
  const groups = Array.from(document.querySelectorAll(".dock-group[data-view]"));
  let visibleCount = 0;
  groups.forEach((group) => {
    const expected = group.dataset.view;
    let show = expected === viewKey;
    if (state.viewCombo) {
      show = expected === "graph" || expected === "2d";
    }
    group.classList.toggle("hidden", !show);
    if (show) visibleCount += 1;
  });
  const empty = $("view-dock-empty");
  if (empty) empty.classList.toggle("hidden", visibleCount > 0);
  renderDockSummary();
}

function renderDockSummary() {
  const summary = $("view-dock-summary");
  if (!summary) return;
  const viewKey = (state.activeView || "view-graph").replace("view-", "");
  const viewNames = {
    graph: "그래프",
    "2d": "2D",
    table: "표",
    text: "문서",
    structure: "구조",
  };
  const viewLabel = viewNames[viewKey] ?? viewKey;
  const runCount = getGraphRunsForRender(getVisibleRuns()).length;
  const lens = state.wasm.lens;
  const lensInfo = lens.enabled
    ? `렌즈:${lens.yKey || "-"}${lens.y2Key ? `+${lens.y2Key}` : ""}`
    : "렌즈:off";
  const autoRun = state.controls.autoRun ? "자동 실행" : "수동 실행";
  const timeEnabled = $("time-enabled")?.checked ? "시간샘플링" : "시간샘플링 꺼짐";
  summary.textContent = `현재 뷰: ${viewLabel} · 실행: ${runCount} · ${lensInfo} · ${autoRun} · ${timeEnabled}`;
}

function updateViewComboLayout() {
  const group = document.querySelector(".view-group");
  if (!group) return;
  const layoutGroup = $("view-combo-layout-group");
  const layoutSelect = $("view-combo-layout");
  const overlayOrderGroup = $("view-combo-overlay-order-group");
  const overlayOrderSelect = $("view-combo-overlay-order");
  if (layoutGroup) layoutGroup.classList.toggle("hidden", !state.viewCombo);
  if (layoutSelect && state.viewComboLayout) {
    layoutSelect.value = state.viewComboLayout;
  }
  const showOverlayOrder = state.viewCombo && state.viewComboLayout === "overlay";
  if (overlayOrderGroup) overlayOrderGroup.classList.toggle("hidden", !showOverlayOrder);
  if (overlayOrderSelect && state.viewOverlayOrder) {
    overlayOrderSelect.value = state.viewOverlayOrder;
  }
  const panes = Array.from(group.querySelectorAll(".tab-body"));
  const graphPane = $("view-graph");
  const spacePane = $("view-2d");
  if (state.viewCombo) {
    group.classList.add("view-combo");
    group.classList.toggle("view-combo-horizontal", state.viewComboLayout === "horizontal");
    group.classList.toggle("view-combo-vertical", state.viewComboLayout === "vertical");
    group.classList.toggle("view-combo-overlay", state.viewComboLayout === "overlay");
    group.classList.toggle("view-overlay-top-graph", showOverlayOrder && state.viewOverlayOrder === "graph");
    group.classList.toggle("view-overlay-top-2d", showOverlayOrder && state.viewOverlayOrder === "2d");
    panes.forEach((pane) => {
      if (pane === graphPane || pane === spacePane) {
        pane.classList.remove("hidden");
      } else {
        pane.classList.add("hidden");
      }
    });
    return;
  }
  group.classList.remove("view-combo");
  group.classList.remove("view-combo-horizontal", "view-combo-vertical", "view-combo-overlay");
  group.classList.remove("view-overlay-top-graph", "view-overlay-top-2d");
  panes.forEach((pane) => pane.classList.add("hidden"));
  const target = state.activeView ?? "view-graph";
  group.querySelector(`#${target}`)?.classList.remove("hidden");
}

function setViewCombo(enabled) {
  state.viewCombo = enabled;
  const toggle = $("view-combo-toggle");
  if (toggle) toggle.checked = enabled;
  if (enabled) {
    const layout = $("view-combo-layout")?.value;
    if (layout) state.viewComboLayout = layout;
    const overlayOrder = $("view-combo-overlay-order")?.value;
    if (overlayOrder) state.viewOverlayOrder = overlayOrder;
  }
  if (enabled && !["view-graph", "view-2d"].includes(state.activeView)) {
    state.activeView = "view-graph";
  }
  updateDockVisibility();
  updateViewComboLayout();
  renderAll();
}

function setActiveView(viewId) {
  state.activeView = viewId;
  applyUpdateControlsForView(viewId);
  updateDockVisibility();
  updateViewComboLayout();
  renderAll();
}

function ownedTabsInGroup(group) {
  return Array.from(group.querySelectorAll(".tab")).filter(
    (tab) => tab.closest(".tab-group") === group,
  );
}

function ownedTabBodiesInGroup(group) {
  return Array.from(group.querySelectorAll(".tab-body")).filter(
    (body) => body.closest(".tab-group") === group,
  );
}

function getMainTabButton(tabId) {
  return document.querySelector(`.main-tabs .tab[data-tab="${tabId}"]`);
}

function setWorkspaceMode(mode, options = {}) {
  const normalized = mode === "advanced" ? "advanced" : "basic";
  const ensureTab = options.ensureTab !== false;
  state.workspaceMode = normalized;

  const basicBtn = $("workspace-mode-basic");
  const advancedBtn = $("workspace-mode-advanced");
  basicBtn?.classList.toggle("active", normalized === "basic");
  advancedBtn?.classList.toggle("active", normalized === "advanced");

  const basicTabIds = ["lesson-tab", "ddn-tab", "run-tab"];
  basicTabIds.forEach((tabId) => {
    const tab = getMainTabButton(tabId);
    if (tab) tab.classList.toggle("hidden", normalized !== "basic");
  });

  const toolsTab = getMainTabButton("tools-tab");
  if (toolsTab) toolsTab.classList.toggle("hidden", normalized !== "advanced");

  if (normalized === "advanced") {
    if (ensureTab) {
      toolsTab?.click();
    } else {
      renderLogs();
    }
    return;
  }

  if (ensureTab) {
    const active = document.querySelector(".main-tabs .tab.active");
    if (active?.dataset.tab === "tools-tab") {
      const fallback =
        getMainTabButton("ddn-tab") ??
        getMainTabButton("lesson-tab") ??
        getMainTabButton("run-tab");
      fallback?.click();
    }
  }
}

function initTabs() {
  document.querySelectorAll(".tab-group").forEach((group) => {
    const tabs = ownedTabsInGroup(group);
    const bodies = ownedTabBodiesInGroup(group);
    tabs.forEach((tab) => {
      tab.addEventListener("click", () => {
        tabs.forEach((t) => t.classList.remove("active"));
        bodies.forEach((b) => b.classList.add("hidden"));
        tab.classList.add("active");
        const target = tab.dataset.tab;
        const body = bodies.find((pane) => pane.id === target);
        if (body) body.classList.remove("hidden");
        if (group.classList.contains("view-group")) {
          if (state.viewCombo && !["view-graph", "view-2d"].includes(target)) {
            state.viewCombo = false;
            const toggle = $("view-combo-toggle");
            if (toggle) toggle.checked = false;
          }
          setActiveView(target);
        }
        if (group.classList.contains("main-tabs")) {
          if (target === "tools-tab") {
            setWorkspaceMode("advanced", { ensureTab: false });
          } else {
            setWorkspaceMode("basic", { ensureTab: false });
          }
          if (target === "run-tab") updateDockVisibility();
        }
      });
    });
  });
}

async function loadLessonSchemaStatus() {
  try {
    const response = await fetch("/lessons/schema_status.json");
    if (!response.ok) return;
    const payload = await response.json();
    const rows = Array.isArray(payload?.lessons) ? payload.lessons : [];
    const map = {};
    rows.forEach((row) => {
      const id = String(row?.lesson_id ?? "").trim();
      if (!id) return;
      map[id] = row;
    });
    state.lessons.schemaStatus = map;
  } catch (_) {
    // optional index
  }
}

async function loadLessonCatalog() {
  try {
    const response = await fetch("/lessons/index.json");
    if (!response.ok) throw new Error("lesson index not found");
    const data = await response.json();
    state.lessons.list = data.lessons ?? [];
    state.lessons.groups = data.groups ?? [];
    await loadLessonSchemaStatus();
    renderLessonFilters();
    renderLessonList();
  } catch (_) {
    state.lessons.list = [
      {
        id: "math_line",
        title: "수학 · 직선",
        description: "일차 함수 기본",
        required_views: ["graph"],
        ddn: sampleDdnLine,
      },
      {
        id: "physics_motion",
        title: "물리 · 등가속도",
        description: "t축 등가속도 운동",
        required_views: ["graph", "table"],
        ddn: sampleDdnTime,
      },
    ];
    state.lessons.groups = [
      {
        id: "grade-middle",
        title: "중등",
        children: [
          {
            id: "subject-math",
            title: "수학",
            lessons: ["math_line"],
          },
          {
            id: "subject-physics",
            title: "물리",
            lessons: ["physics_motion"],
          },
        ],
      },
    ];
    await loadLessonSchemaStatus();
    renderLessonFilters();
    renderLessonList();
  }
}

function renderLessonList() {
  const list = $("lesson-list");
  list.innerHTML = "";
  const lessonMap = new Map(state.lessons.list.map((lesson) => [lesson.id, lesson]));
  const profileRank = {
    age3_target: 0,
    modern_partial: 1,
    mixed: 2,
    legacy: 3,
    unknown: 4,
  };

  const matchesFilter = (lesson) => {
    if (!lesson) return false;
    const grade = lesson.grade ?? "unknown";
    const subject = lesson.subject ?? "unknown";
    const gradeFilter = state.lessons.filter.grade ?? "all";
    const subjectFilter = state.lessons.filter.subject ?? "all";
    const schemaFilter = state.lessons.filter.schema ?? "all";
    const query = (state.lessons.filter.query ?? "").trim().toLowerCase();
    if (gradeFilter !== "all" && gradeFilter !== grade) return false;
    if (subjectFilter !== "all" && subjectFilter !== subject) return false;
    if (schemaFilter !== "all") {
      const status = state.lessons.schemaStatus?.[lesson.id] ?? null;
      if (schemaFilter === "preview" && !status?.has_preview) return false;
      if (schemaFilter === "legacy") {
        const legacyCount = Number(status?.effective_counts?.legacy_show ?? 0);
        if (!(legacyCount > 0)) return false;
      }
      if (
        schemaFilter === "age3_target" ||
        schemaFilter === "modern_partial" ||
        schemaFilter === "mixed" ||
        schemaFilter === "unknown"
      ) {
        if ((status?.effective_profile ?? "unknown") !== schemaFilter) return false;
      }
    }
    if (query) {
      const targets = [
        lesson.id,
        lesson.title,
        lesson.description,
        ...(Array.isArray(lesson.goals) ? lesson.goals : []),
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase();
      if (!targets.includes(query)) return false;
    }
    return true;
  };

  const matchedLessons = state.lessons.list.filter((lesson) => matchesFilter(lesson));
  updateLessonSchemaSummary(matchedLessons.length);
  const hasMatch = matchedLessons.length > 0;
  if (!hasMatch) {
    const empty = document.createElement("div");
    empty.className = "lesson-empty";
    empty.textContent = "선택한 조건에 맞는 교과가 없습니다.";
    list.appendChild(empty);
    return;
  }

  const lessonPriority = (lesson) => {
    const status = state.lessons.schemaStatus?.[lesson?.id] ?? null;
    const profile = status?.effective_profile ?? "unknown";
    const rank = profileRank[profile] ?? profileRank.unknown;
    const previewPenalty = status?.has_preview ? 0 : 1;
    const legacyPenalty = Number(status?.effective_counts?.legacy_show ?? 0) > 0 ? 1 : 0;
    const title = String(lesson?.title ?? lesson?.id ?? "").toLowerCase();
    return [rank, previewPenalty, legacyPenalty, title];
  };

  const comparePriority = (a, b) => {
    const pa = lessonPriority(a);
    const pb = lessonPriority(b);
    for (let i = 0; i < pa.length; i++) {
      if (pa[i] < pb[i]) return -1;
      if (pa[i] > pb[i]) return 1;
    }
    return 0;
  };

  const renderLessonItem = (lesson, container) => {
    if (!lesson || !matchesFilter(lesson)) return false;
    const item = document.createElement("div");
    item.className = "lesson-item";
    if (lesson.id === state.lessons.activeId) item.classList.add("active");
    const header = document.createElement("div");
    header.className = "lesson-item-header";
    const title = document.createElement("strong");
    title.textContent = lesson.title ?? lesson.id;
    const badges = document.createElement("div");
    badges.className = "lesson-badges";
    const badgeDefs = [
      { key: "grade", value: lesson.grade },
      { key: "subject", value: lesson.subject },
      { key: "level", value: lesson.level },
      { key: "source", value: lesson.source },
    ];
    const badgeLabels = {
      grade: { middle: "중등", high: "고등", college: "대학" },
      subject: { math: "수학", physics: "물리", econ: "경제" },
      level: { intro: "입문", intermediate: "중급", advanced: "심화" },
      source: { ssot: "교과" },
    };
    badgeDefs.forEach((badge) => {
      if (!badge.value) return;
      const span = document.createElement("span");
      span.className = "lesson-badge";
      span.textContent = badgeLabels[badge.key]?.[badge.value] ?? badge.value;
      badges.appendChild(span);
    });
    header.appendChild(title);
    header.appendChild(badges);
    const desc = document.createElement("span");
    desc.textContent = lesson.description ?? "";
    item.appendChild(header);
    item.appendChild(desc);
    const schemaHint = state.lessons.schemaStatus?.[lesson.id];
    if (schemaHint) {
      const info = document.createElement("div");
      info.className = "lesson-goals";
      const previewFlag = schemaHint.has_preview ? "preview" : "legacy";
      info.textContent = `DDN: ${previewFlag} · ${schemaHint.effective_profile ?? "unknown"}`;
      item.appendChild(info);
    }
    if (Array.isArray(lesson.goals) && lesson.goals.length) {
      const goals = document.createElement("div");
      goals.className = "lesson-goals";
      goals.textContent = `목표: ${lesson.goals.join(" · ")}`;
      item.appendChild(goals);
    }
    item.addEventListener("click", () => selectLesson(lesson));
    container.appendChild(item);
    return true;
  };

  const renderGroup = (group, container, level = 0) => {
    const wrapper = document.createElement("div");
    wrapper.className = `lesson-group level-${level}`;
    const title = document.createElement("div");
    title.className = "lesson-group-title";
    title.textContent = group.title ?? group.id ?? "그룹";
    wrapper.appendChild(title);
    const body = document.createElement("div");
    body.className = "lesson-group-body";
    wrapper.appendChild(body);
    let added = 0;

    if (Array.isArray(group.lessons)) {
      const rows = group.lessons
        .map((lessonRef) => (typeof lessonRef === "string" ? lessonMap.get(lessonRef) : lessonRef))
        .filter(Boolean)
        .sort(comparePriority);
      rows.forEach((lesson) => {
        if (renderLessonItem(lesson, body)) added += 1;
      });
    }
    if (Array.isArray(group.children)) {
      group.children.forEach((child) => {
        if (renderGroup(child, body, level + 1)) added += 1;
      });
    }
    const showEmptyGroups =
      (state.lessons.filter.grade ?? "all") === "all" &&
      (state.lessons.filter.subject ?? "all") === "all";
    if (added > 0 || (showEmptyGroups && level === 0)) {
      if (added === 0) {
        const empty = document.createElement("div");
        empty.className = "lesson-empty";
        empty.textContent = "준비중";
        body.appendChild(empty);
      }
      container.appendChild(wrapper);
      return true;
    }
    return false;
  };

  if (Array.isArray(state.lessons.groups) && state.lessons.groups.length) {
    state.lessons.groups.forEach((group) => renderGroup(group, list, 0));
    return;
  }

  state.lessons.list
    .slice()
    .sort(comparePriority)
    .forEach((lesson) => renderLessonItem(lesson, list));
}

function updateLessonSchemaSummary(filteredCount = null) {
  const summaryEl = $("lesson-schema-summary");
  if (!summaryEl) return;
  const rows = Object.values(state.lessons.schemaStatus ?? {});
  if (!rows.length) {
    summaryEl.textContent = "스키마 요약: 상태 파일 없음";
    return;
  }
  const countBy = (profile) => rows.filter((row) => (row?.effective_profile ?? "unknown") === profile).length;
  const total = rows.length;
  const age3 = countBy("age3_target");
  const partial = countBy("modern_partial");
  const legacy = countBy("legacy");
  const mixed = countBy("mixed");
  const unknown = countBy("unknown");
  const visible = Number.isFinite(filteredCount) ? filteredCount : state.lessons.list.length;
  summaryEl.textContent =
    `스키마 요약: AGE3 ${age3}/${total} · 전환중 ${partial} · 레거시 ${legacy}` +
    ` · 혼합 ${mixed} · 미확인 ${unknown} · 현재 목록 ${visible}`;
}

function renderLessonFilters() {
  const gradeSelect = $("lesson-grade-filter");
  const subjectSelect = $("lesson-subject-filter");
  const schemaSelect = $("lesson-schema-filter");
  const searchInput = $("lesson-search");
  if (!gradeSelect || !subjectSelect) return;
  const gradeLabels = {
    middle: "중등",
    high: "고등",
    college: "대학",
  };
  const subjectLabels = {
    math: "수학",
    physics: "물리",
    econ: "경제",
  };
  const grades = new Set();
  const subjects = new Set();
  state.lessons.list.forEach((lesson) => {
    if (lesson.grade) grades.add(lesson.grade);
    if (lesson.subject) subjects.add(lesson.subject);
  });
  const ingestGroup = (group, level = 0) => {
    if (!group) return;
    if (level === 0 && typeof group.id === "string" && group.id.startsWith("grade-")) {
      grades.add(group.id.replace("grade-", ""));
    }
    if (level === 1 && typeof group.id === "string" && group.id.startsWith("subject-")) {
      subjects.add(group.id.replace("subject-", ""));
    }
    if (Array.isArray(group.children)) {
      group.children.forEach((child) => ingestGroup(child, level + 1));
    }
  };
  if (Array.isArray(state.lessons.groups)) {
    state.lessons.groups.forEach((group) => ingestGroup(group, 0));
  }
  const buildOptions = (select, items, current, labels) => {
    select.innerHTML = "";
    const allOpt = document.createElement("option");
    allOpt.value = "all";
    allOpt.textContent = "전체";
    select.appendChild(allOpt);
    Array.from(items)
      .sort()
      .forEach((value) => {
        const opt = document.createElement("option");
        opt.value = value;
        opt.textContent = labels[value] ?? value;
        select.appendChild(opt);
      });
    select.value = current ?? "all";
  };
  buildOptions(gradeSelect, grades, state.lessons.filter.grade, gradeLabels);
  buildOptions(subjectSelect, subjects, state.lessons.filter.subject, subjectLabels);
  if (schemaSelect) {
    schemaSelect.innerHTML = "";
    [
      { value: "all", label: "전체" },
      { value: "preview", label: "preview 가능" },
      { value: "age3_target", label: "AGE3 목표형" },
      { value: "modern_partial", label: "전환중(보임)" },
      { value: "legacy", label: "레거시" },
      { value: "mixed", label: "혼합" },
      { value: "unknown", label: "미확인" },
    ].forEach((item) => {
      const opt = document.createElement("option");
      opt.value = item.value;
      opt.textContent = item.label;
      schemaSelect.appendChild(opt);
    });
    schemaSelect.value = state.lessons.filter.schema ?? "all";
  }
  if (searchInput) searchInput.value = state.lessons.filter.query ?? "";
}

async function fetchFirstOkText(urls) {
  for (const url of urls) {
    try {
      const res = await fetch(url);
      if (!res.ok) continue;
      return { url, text: await res.text() };
    } catch (_) {
      // ignore and continue fallback chain
    }
  }
  return null;
}

async function loadLessonDdnText(lessonId, usePreview = true) {
  if (!lessonId) return null;
  const ddnCandidates = usePreview
    ? [
        `/lessons/${lessonId}/lesson.age3.preview.ddn`,
        `/lessons/${lessonId}/lesson.ddn`,
      ]
    : [
        `/lessons/${lessonId}/lesson.ddn`,
        `/lessons/${lessonId}/lesson.age3.preview.ddn`,
      ];
  const ddnLoaded = await fetchFirstOkText(ddnCandidates);
  if (!ddnLoaded) return null;
  return {
    text: ddnLoaded.text,
    source: ddnLoaded.url.split("/").pop() || ddnLoaded.url,
  };
}

async function selectLesson(lesson) {
  if (!lesson) return;
  state.lessons.activeId = lesson.id;
  let meta = lesson;
  let ddnText = lesson.ddn ?? "";
  let ddnSource = lesson.ddn ? "embedded" : "-";

  try {
    if (!lesson.ddn && lesson.id) {
      const metaRes = await fetch(`/lessons/${lesson.id}/meta.toml`);
      if (metaRes.ok) {
        const metaText = await metaRes.text();
        meta = { ...meta, ...parseToml(metaText) };
      }
      const usePreview = $("lesson-use-age3-preview")?.checked ?? state.lessons.useAge3Preview;
      state.lessons.useAge3Preview = usePreview;
      const ddnLoaded = await loadLessonDdnText(lesson.id, usePreview);
      if (ddnLoaded) {
        ddnText = ddnLoaded.text;
        ddnSource = ddnLoaded.source;
      }
    }
  } catch (_) {
    // fallback to embedded
  }
  state.lessons.meta = meta;
  state.lessons.ddnText = ddnText;
  state.lessons.ddnSource = ddnSource;
  $("lesson-title").textContent = meta.title || meta.id || lesson.id || "lesson";
  $("lesson-desc").textContent = meta.description || "-";
  const views = meta.required_views || ["graph"];
  $("lesson-meta").textContent = `required_views: ${views.join(", ")}`;
  const lessonSchemaEl = $("lesson-schema-status");
  if (lessonSchemaEl) {
    lessonSchemaEl.textContent = resolveLessonSchemaStatusLabel(lesson.id, ddnText);
  }
  const lessonSourceEl = $("lesson-ddn-source");
  if (lessonSourceEl) {
    lessonSourceEl.textContent = `DDN 소스: ${state.lessons.ddnSource}`;
  }
  if (views.includes("2d")) {
    state.viewUpdate["view-2d"] = { update: "append", tick: null };
    if (state.activeView === "view-2d") {
      applyUpdateControlsForView("view-2d");
    }
  }
  if (ddnText) {
    $("ddn-editor").value = ddnText;
    updateSceneFromDdn(ddnText);
    refreshDdnControlsFromText(ddnText);
  }
  updateDdnTimeStatus();
  applyTimeDefaultsFromDdn(ddnText);
  applyRequiredViews(views);
  registerLessonInput(lesson.id, meta, ddnText);
  const loaded = await loadLessonAssets(lesson.id, views);
  applyLessonAutoView(views, loaded);
  await maybeAutoRunLesson(ddnText);
  renderLessonList();
  // 교과 로드 후 DDN 탭으로 자동 전환
  setWorkspaceMode("basic");
  const ddnTab = document.querySelector('.main-tabs .tab[data-tab="ddn-tab"]');
  if (ddnTab) ddnTab.click();
}

function parseToml(text) {
  const out = {};
  const lines = normalizeNewlines(text).split("\n");
  lines.forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) return;
    const [key, rest] = trimmed.split("=");
    if (!key || !rest) return;
    const cleanKey = key.trim();
    const valueRaw = rest.trim();
    if (valueRaw.startsWith("[")) {
      const items = valueRaw
        .replace(/\[/g, "")
        .replace(/\]/g, "")
        .split(",")
        .map((item) => item.trim().replace(/^"|"$/g, ""))
        .filter((item) => item);
      out[cleanKey] = items;
      return;
    }
    out[cleanKey] = valueRaw.replace(/^"|"$/g, "");
  });
  return out;
}

async function fetchLessonAsset(lessonId, filename) {
  try {
    const response = await fetch(`/lessons/${lessonId}/${filename}`);
    if (!response.ok) return null;
    return await response.text();
  } catch (_) {
    return null;
  }
}

async function loadLessonAssets(lessonId, views) {
  state.table = null;
  state.textDoc = null;
  state.structure = null;
  state.space2d = null;
  state.space2dRange = null;
  const loaded = {
    table: false,
    text: false,
    structure: false,
    space2d: false,
  };

  if (views.includes("table")) {
    const tableText = (await fetchLessonAsset(lessonId, "table.json")) ?? (await fetchLessonAsset(lessonId, "table.csv"));
    if (tableText) {
      try {
        let tableData = null;
        if (tableText.trim().startsWith("{")) {
          const data = JSON.parse(tableText);
          tableData = normalizeTableData(data) ?? data;
        } else {
          const rows = parseCsv(tableText);
          tableData = rowsToTableData(rows);
        }
        if (tableData) {
          state.table = tableData;
          state.tableView.page = 0;
          loaded.table = true;
        }
      } catch (err) {
        log(`표 불러오기 실패: ${err.message}`);
      }
    }
  }

  if (views.includes("text")) {
    const text = await fetchLessonAsset(lessonId, "text.md");
    if (text) {
      state.textDoc = { content: text, format: "markdown" };
      loaded.text = true;
    }
  }

  if (views.includes("structure")) {
    const structText = await fetchLessonAsset(lessonId, "structure.json");
    if (structText) {
      try {
        const data = JSON.parse(structText);
        const checked = validateStructureData(data);
        state.structure = { ...checked, selection: null };
        loaded.structure = true;
      } catch (err) {
        log(`구조 불러오기 실패: ${err.message}`);
      }
    }
  }

  if (views.includes("2d")) {
    const spaceText = await fetchLessonAsset(lessonId, "space2d.json");
    if (spaceText) {
      try {
        const data = validateSpace2dData(JSON.parse(spaceText));
        state.space2d = data;
        state.space2dRange = computeSpace2dRange(data);
        loaded.space2d = true;
      } catch (err) {
        log(`2D 보개 불러오기 실패: ${err.message}`);
      }
    }
  }

  renderTableView();
  renderTextView();
  renderStructureView();
  render2dView();
  return loaded;
}

function isLessonAutoViewEnabled() {
  const checkbox = $("lesson-auto-view");
  if (!checkbox) return true;
  return checkbox.checked;
}

function isLessonAutoRunEnabled() {
  const checkbox = $("lesson-auto-run");
  if (!checkbox) return false;
  return checkbox.checked;
}

function applyTimeDefaultsFromDdn(ddnText) {
  const range = extractTimeRangeFromDdn(ddnText);
  if (!range) return false;
  $("time-enabled").checked = true;
  $("t-min").value = range.t_min;
  $("t-max").value = range.t_max;
  $("t-step").value = range.step;
  $("t-now").value = range.t_min;
  applyTimeModeUi();
  syncTimeCursorRange();
  markTimeFramesDirty();
  updateDdnTimeStatus();
  return true;
}

async function maybeAutoRunLesson(ddnText) {
  if (!isLessonAutoRunEnabled()) return;
  if (!ddnText?.trim()) return;
  const baseUrl = $("bridge-url").value.trim();
  if (!baseUrl) {
    log("브리지 URL이 없어 교과 자동 실행을 건너뜁니다.");
    return;
  }
  const ok = await runDdnBridge({ mode: "auto" });
  const fromPreview = String(state.lessons.ddnSource || "").includes(".age3.preview.ddn");
  if (ok || !fromPreview) return;
  const lessonId = state.lessons.activeId;
  if (!lessonId) return;
  const fallback = await loadLessonDdnText(lessonId, false);
  if (!fallback?.text) return;
  state.lessons.ddnText = fallback.text;
  state.lessons.ddnSource = fallback.source;
  $("ddn-editor").value = fallback.text;
  updateSceneFromDdn(fallback.text);
  refreshDdnControlsFromText(fallback.text);
  applyTimeDefaultsFromDdn(fallback.text);
  const lessonSourceEl = $("lesson-ddn-source");
  if (lessonSourceEl) {
    lessonSourceEl.textContent = `DDN 소스: ${state.lessons.ddnSource}`;
  }
  const lessonSchemaEl = $("lesson-schema-status");
  if (lessonSchemaEl) {
    lessonSchemaEl.textContent = resolveLessonSchemaStatusLabel(lessonId, fallback.text);
  }
  const previewToggle = $("lesson-use-age3-preview");
  if (previewToggle) previewToggle.checked = false;
  state.lessons.useAge3Preview = false;
  saveLessonPreviewMode();
  log("AGE3 preview 자동실행 실패: lesson.ddn로 자동 폴백합니다.");
  await runDdnBridge({ mode: "auto" });
}

function pickAutoView(views, loaded) {
  const viewOrder = Array.isArray(views) && views.length ? views : ["graph"];
  const map = {
    "2d": { id: "view-2d", ok: loaded?.space2d },
    "table": { id: "view-table", ok: loaded?.table },
    "text": { id: "view-text", ok: loaded?.text },
    "structure": { id: "view-structure", ok: loaded?.structure },
    "graph": { id: "view-graph", ok: true },
  };
  for (const view of viewOrder) {
    const entry = map[view];
    if (!entry) continue;
    if (view === "graph") return entry.id;
    if (entry.ok) return entry.id;
  }
  if (loaded?.space2d) return "view-2d";
  if (loaded?.structure) return "view-structure";
  if (loaded?.table) return "view-table";
  if (loaded?.text) return "view-text";
  return "view-graph";
}

function applyLessonAutoView(views, loaded) {
  if (!isLessonAutoViewEnabled()) return;
  const target = pickAutoView(views, loaded);
  if (!target) return;
  const tab = document.querySelector(`.view-group .tab[data-tab="${target}"]`);
  if (tab) tab.click();
  const wantsCombo = Array.isArray(views) && views.includes("graph") && views.includes("2d") && loaded?.space2d;
  if (wantsCombo) {
    const layout = resolveLessonComboLayout(state.lessons.meta);
    if (layout) state.viewComboLayout = layout;
    const overlayOrder = resolveLessonOverlayOrder(state.lessons.meta);
    if (overlayOrder) state.viewOverlayOrder = overlayOrder;
    setViewCombo(true);
  } else if (state.viewCombo) {
    setViewCombo(false);
  }
}

function resolveLessonComboLayout(meta) {
  if (!meta) return null;
  const raw = String(meta.view_combo_layout ?? meta.view_layout ?? "").toLowerCase();
  if (raw === "horizontal" || raw === "vertical" || raw === "overlay") return raw;
  const preset = String(meta.layout_preset ?? "").toLowerCase();
  if (preset.includes("좌") || preset.includes("우") || preset.includes("left") || preset.includes("right")) {
    return "horizontal";
  }
  if (preset.includes("상") || preset.includes("하") || preset.includes("top") || preset.includes("bottom")) {
    return "vertical";
  }
  if (preset.includes("겹") || preset.includes("overlay") || preset.includes("stack")) {
    return "overlay";
  }
  return null;
}

function resolveLessonOverlayOrder(meta) {
  if (!meta) return null;
  const raw = String(
    meta.view_overlay_order ?? meta.view_combo_overlay_order ?? meta.view_overlay ?? "",
  ).toLowerCase();
  if (raw === "graph" || raw === "2d") return raw;
  return null;
}

function detectLessonSchemaProfile(ddnText) {
  const text = String(ddnText || "");
  const hasLegacyShow = /보여주기\./.test(text);
  const hasMamadi = /\(매마디\)마다/.test(text);
  const hasViewBlock = /보임\s*\{/.test(text);
  if (hasViewBlock && hasMamadi && !hasLegacyShow) {
    return "스키마: AGE3 목표형(매마디/보임)";
  }
  if (hasLegacyShow && hasMamadi) {
    return "스키마: 전환 필요(매마디 + 보여주기)";
  }
  if (hasLegacyShow) {
    return "스키마: 레거시(보여주기)";
  }
  return "스키마: 혼합/미확인";
}

function formatLessonSchemaProfile(profile) {
  const key = String(profile || "unknown");
  const map = {
    age3_target: "AGE3 목표형(매마디/보임)",
    modern_partial: "전환중(보임)",
    legacy: "레거시(보여주기)",
    mixed: "혼합",
    unknown: "미확인",
  };
  return map[key] ?? map.unknown;
}

function resolveLessonSchemaStatusLabel(lessonId, ddnText) {
  const status = state.lessons.schemaStatus?.[lessonId];
  if (status?.effective_profile) {
    return `스키마: ${formatLessonSchemaProfile(status.effective_profile)}`;
  }
  return detectLessonSchemaProfile(ddnText);
}

function applyRequiredViews(views) {
  const graphTab = document.querySelector('.view-group .tab[data-tab="view-graph"]');
  const spaceTab = document.querySelector('.view-group .tab[data-tab="view-2d"]');
  const tableTab = document.querySelector('.view-group .tab[data-tab="view-table"]');
  const textTab = document.querySelector('.view-group .tab[data-tab="view-text"]');
  const structureTab = document.querySelector('.view-group .tab[data-tab="view-structure"]');
  const wants2d = views.includes("2d");
  const wantsTable = views.includes("table");
  const wantsText = views.includes("text");
  const wantsStructure = views.includes("structure");
  if (spaceTab) spaceTab.classList.toggle("hidden", !wants2d);
  if (tableTab) tableTab.classList.toggle("hidden", !wantsTable);
  if (textTab) textTab.classList.toggle("hidden", !wantsText);
  if (structureTab) structureTab.classList.toggle("hidden", !wantsStructure);
  if (!wants2d && spaceTab?.classList.contains("active")) {
    graphTab?.click();
  }
  if (!wantsTable && tableTab?.classList.contains("active")) {
    graphTab?.click();
  }
  if (!wantsText && textTab?.classList.contains("active")) {
    graphTab?.click();
  }
  if (!wantsStructure && structureTab?.classList.contains("active")) {
    graphTab?.click();
  }
}

$("lesson-apply").addEventListener("click", () => {
  if (state.lessons.ddnText) {
    $("ddn-editor").value = state.lessons.ddnText;
    refreshDdnControlsFromText(state.lessons.ddnText);
    const lessonSchemaEl = $("lesson-schema-status");
    if (lessonSchemaEl) {
      lessonSchemaEl.textContent = resolveLessonSchemaStatusLabel(state.lessons.activeId, state.lessons.ddnText);
    }
    const lessonSourceEl = $("lesson-ddn-source");
    if (lessonSourceEl) {
      lessonSourceEl.textContent = `DDN 소스: ${state.lessons.ddnSource}`;
    }
    log("lesson.ddn 로드 완료");
  } else {
    log("lesson.ddn가 없습니다.");
  }
});

$("lesson-reset").addEventListener("click", () => {
  if (state.lessons.ddnText) {
    $("ddn-editor").value = state.lessons.ddnText;
    refreshDdnControlsFromText(state.lessons.ddnText);
    const lessonSchemaEl = $("lesson-schema-status");
    if (lessonSchemaEl) {
      lessonSchemaEl.textContent = resolveLessonSchemaStatusLabel(state.lessons.activeId, state.lessons.ddnText);
    }
    const lessonSourceEl = $("lesson-ddn-source");
    if (lessonSourceEl) {
      lessonSourceEl.textContent = `DDN 소스: ${state.lessons.ddnSource}`;
    }
    log("lesson.ddn로 복원했습니다.");
  }
});

const lessonPreviewToggle = $("lesson-use-age3-preview");
if (lessonPreviewToggle) {
  loadLessonPreviewMode();
  lessonPreviewToggle.checked = Boolean(state.lessons.useAge3Preview);
  lessonPreviewToggle.addEventListener("change", async () => {
    state.lessons.useAge3Preview = lessonPreviewToggle.checked;
    saveLessonPreviewMode();
    const active = state.lessons.list.find((item) => item.id === state.lessons.activeId);
    if (!active) return;
    await selectLesson(active);
    log(
      state.lessons.useAge3Preview
        ? "교과 DDN 로드 모드: AGE3 preview 우선"
        : "교과 DDN 로드 모드: 기본 lesson.ddn 우선",
    );
  });
}

const gradeFilter = $("lesson-grade-filter");
if (gradeFilter) {
  gradeFilter.addEventListener("change", (event) => {
    state.lessons.filter.grade = event.target.value || "all";
    renderLessonList();
  });
}
const subjectFilter = $("lesson-subject-filter");
if (subjectFilter) {
  subjectFilter.addEventListener("change", (event) => {
    state.lessons.filter.subject = event.target.value || "all";
    renderLessonList();
  });
}
const lessonSearch = $("lesson-search");
if (lessonSearch) {
  lessonSearch.addEventListener("input", (event) => {
    state.lessons.filter.query = event.target.value || "";
    renderLessonList();
  });
}
const lessonSchemaFilter = $("lesson-schema-filter");
if (lessonSchemaFilter) {
  lessonSchemaFilter.addEventListener("change", (event) => {
    state.lessons.filter.schema = event.target.value || "all";
    renderLessonList();
  });
}

$("ddn-run").addEventListener("click", runDdnBridge);
$("ddn-export").addEventListener("click", exportDdnText);
$("ddn-import").addEventListener("click", importDdnText);
if ($("ddn-editor")) {
  $("ddn-editor").addEventListener("input", () => {
    updateDdnTimeStatus();
    refreshDdnControlsFromText($("ddn-editor").value, { preserveValues: true });
    scheduleDdnEditorAutoRun();
  });
}
$("ddn-frame-stop").addEventListener("click", cancelDdnFrameJob);
const ddnTimeToggle = $("ddn-time-toggle");
if (ddnTimeToggle) {
  ddnTimeToggle.addEventListener("click", () => {
    const checkbox = $("time-enabled");
    checkbox.checked = !checkbox.checked;
    checkbox.dispatchEvent(new Event("change", { bubbles: true }));
    updateDdnTimeStatus();
  });
}
if ($("ddn-time-focus")) $("ddn-time-focus").addEventListener("click", focusViewDock);
$("ddn-save").addEventListener("click", saveDdnPreset);
$("ddn-export-presets").addEventListener("click", exportDdnPresets);
$("ddn-import-presets").addEventListener("click", importDdnPresets);
const mediaExportStartBtn = $("media-export-start");
if (mediaExportStartBtn) {
  mediaExportStartBtn.addEventListener("click", startMediaExport);
}
const mediaExportStopBtn = $("media-export-stop");
if (mediaExportStopBtn) {
  mediaExportStopBtn.addEventListener("click", () => {
    stopMediaExport({ reason: "사용자 중지" });
  });
}
const controlAutoRun = $("ddn-control-auto-run");
if (controlAutoRun) {
  state.controls.autoRun = controlAutoRun.checked;
  controlAutoRun.addEventListener("change", () => {
    state.controls.autoRun = controlAutoRun.checked;
    if (!state.controls.autoRun && ddnEditorAutoRunTimer) {
      clearTimeout(ddnEditorAutoRunTimer);
      ddnEditorAutoRunTimer = null;
    }
  });
}
$("snapshot-save").addEventListener("click", saveSnapshot);
$("load-graph").addEventListener("click", loadGraphFile);
$("export-graph").addEventListener("click", exportGraph);
$("load-snapshots").addEventListener("click", loadSnapshots);
$("export-snapshots").addEventListener("click", exportSnapshots);
$("load-session").addEventListener("click", loadSession);
$("export-session").addEventListener("click", exportSession);
$("load-table").addEventListener("click", loadTableFile);
$("load-text").addEventListener("click", loadTextFile);
$("load-structure").addEventListener("click", loadStructureFile);
const loadSpace2dBtn = $("load-space2d");
if (loadSpace2dBtn) {
  loadSpace2dBtn.addEventListener("click", loadSpace2dFile);
}
$("load-sam").addEventListener("click", loadSamFile);
$("load-geoul").addEventListener("click", loadGeoulFile);
$("clear-overlay").addEventListener("click", () => {
  state.soloRunId = null;
  state.runs.forEach((run) => {
    run.visible = true;
  });
  renderOverlayList();
  renderAll();
});

$("time-enabled").addEventListener("change", () => {
  applyTimeModeUi();
  syncTimeCursorRange();
  stopTimePlayback();
  markTimeFramesDirty();
  updateDdnTimeStatus();
});

["t-min", "t-max", "t-step"].forEach((id) => {
  $(id).addEventListener("input", () => {
    syncTimeCursorRange();
    markTimeFramesDirty();
    updateDdnTimeStatus();
  });
});

$("t-now").addEventListener("input", () => {
  syncTimeCursorRange();
  updateDdnTimeStatus();
  const run = getActiveRun();
  if (run?.timeFrames?.length) {
    const idx = findNearestFrameIndex(run.timeFrames, readTimeControls().now ?? 0);
    state.time.index = idx;
    applyTimeFrame();
  }
});

$("t-cursor").addEventListener("input", (event) => {
  const value = Number(event.target.value);
  setTimeNow(value, { render: false });
  updateDdnTimeStatus();
  const run = getActiveRun();
  if (run?.timeFrames?.length) {
    const idx = findNearestFrameIndex(run.timeFrames, value);
    state.time.index = idx;
    applyTimeFrame();
  }
});

$("timeline-track").addEventListener("click", (event) => {
  const timeState = readTimeControls();
  if (!timeState.enabled) return;
  const rect = event.currentTarget.getBoundingClientRect();
  const ratio = rect.width > 0 ? (event.clientX - rect.left) / rect.width : 0;
  const tMin = Number.isFinite(timeState.t_min) ? timeState.t_min : 0;
  const tMax = Number.isFinite(timeState.t_max) ? timeState.t_max : tMin + 1;
  const target = tMin + ratio * (tMax - tMin);
  jumpToTime(Number(target.toFixed(4)));
});

$("t-interval").addEventListener("input", () => {
  if (state.time.playing) {
    stopTimePlayback();
    startTimePlayback();
  }
  updateDdnTimeStatus();
});

$("time-loop").addEventListener("change", () => {
  state.time.loop = $("time-loop").checked;
});

$("time-play").addEventListener("click", startTimePlayback);
$("time-stop").addEventListener("click", stopTimePlayback);
$("time-step").addEventListener("click", () => stepTimeFrame({ auto: false }));

["table-page-size", "table-precision"].forEach((id) => {
  $(id).addEventListener("input", () => {
    if (id === "table-page-size") {
      state.tableView.pageSize = Number($(id).value) || 50;
      state.tableView.page = 0;
    }
    if (id === "table-precision") {
      state.tableView.precision = Number($(id).value) || 3;
    }
    renderTableView();
  });
});

$("table-prev").addEventListener("click", () => {
  state.tableView.page = Math.max(0, state.tableView.page - 1);
  renderTableView();
});
$("table-next").addEventListener("click", () => {
  state.tableView.page += 1;
  renderTableView();
});

$("structure-layout").addEventListener("change", () => {
  state.structureView.layout = $("structure-layout").value;
  renderStructureView();
});
$("structure-node-size").addEventListener("input", () => {
  state.structureView.nodeSize = Number($("structure-node-size").value) || 10;
  renderStructureView();
});
$("structure-show-labels").addEventListener("change", () => {
  state.structureView.showLabels = $("structure-show-labels").checked;
  renderStructureView();
});

if ($("structure-canvas")) {
  $("structure-canvas").addEventListener("click", (event) => {
    if (!state.structureLayout || !state.structure) return;
    const canvas = $("structure-canvas");
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    const pad = 40;
    const w = canvas.width;
    const h = canvas.height;
    const mapX = (nx) => pad + nx * (w - pad * 2);
    const mapY = (ny) => pad + ny * (h - pad * 2);
    const radius = (state.structureView.nodeSize ?? 10) + 4;
    let best = null;
    state.structureLayout.forEach((node) => {
      const nx = mapX(node._pos.x);
      const ny = mapY(node._pos.y);
      const dx = x - nx;
      const dy = y - ny;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist <= radius && (!best || dist < best.dist)) {
        best = { node, dist };
      }
    });
    if (best) {
      state.structure.selection = { kind: "node", node: best.node };
    } else {
      state.structure.selection = null;
    }
    renderStructureView();
  });
}

["view-x-min", "view-x-max", "view-y-min", "view-y-max", "pan-x", "pan-y", "zoom"].forEach((id) => {
  $(id).addEventListener("input", updateGraphViewFromControls);
});

$("view-auto").addEventListener("change", () => {
  updateViewInputsEnabled();
  updateGraphViewFromControls();
});

["space2d-x-min", "space2d-x-max", "space2d-y-min", "space2d-y-max", "space2d-pan-x", "space2d-pan-y", "space2d-zoom"].forEach(
  (id) => {
    const input = $(id);
    if (input) input.addEventListener("input", updateSpace2dViewFromControls);
  },
);

const space2dAuto = $("space2d-auto");
if (space2dAuto) {
  space2dAuto.addEventListener("change", () => {
    updateSpace2dInputsEnabled();
    updateSpace2dViewFromControls();
  });
}
const space2dResetViewBtn = $("space2d-reset-view");
if (space2dResetViewBtn) {
  space2dResetViewBtn.addEventListener("click", () => {
    resetSpace2dView({ forceAutoFit: true });
    render2dView();
    renderInspector();
  });
}

$("toggle-grid").addEventListener("change", updateGraphViewFromControls);
$("toggle-axis").addEventListener("change", updateGraphViewFromControls);
$("update-mode").addEventListener("change", updateGraphMetaFromControls);
$("update-tick").addEventListener("input", updateGraphMetaFromControls);
if ($("view-combo-toggle")) {
  $("view-combo-toggle").addEventListener("change", (event) => {
    setViewCombo(event.target.checked);
  });
}
if ($("view-combo-layout")) {
  $("view-combo-layout").addEventListener("change", (event) => {
    state.viewComboLayout = event.target.value;
    updateViewComboLayout();
  });
}
if ($("view-combo-overlay-order")) {
  $("view-combo-overlay-order").addEventListener("change", (event) => {
    state.viewOverlayOrder = event.target.value;
    updateViewComboLayout();
  });
}

document.querySelectorAll(".contract-toggle").forEach((btn) => {
  btn.addEventListener("click", () => {
    const mode = btn.dataset.contractView || "plain";
    setContractView(mode);
  });
});

// --- Key Canon Table v1 ---
const KEY_CANON_TABLE = {
  version: "keymap_v1",
  aliases: {
    "왼쪽": "ArrowLeft", "오른쪽": "ArrowRight", "위": "ArrowUp", "아래": "ArrowDown",
    "좌": "ArrowLeft", "우": "ArrowRight", "상": "ArrowUp", "하": "ArrowDown",
    "왼쪽화살표": "ArrowLeft", "오른쪽화살표": "ArrowRight",
    "위화살표": "ArrowUp", "아래화살표": "ArrowDown",
    "엔터": "Enter", "리턴": "Enter",
    "이스케이프": "Escape", "esc": "Escape",
    "백스페이스": "Backspace", "삭제": "Delete", "딜리트": "Delete",
    "탭": "Tab", "스페이스": "Space", "공백": "Space",
    "쉬프트": "Shift", "시프트": "Shift",
    "컨트롤": "Control", "컨트롤키": "Control",
    "알트": "Alt", "메타": "Meta", "윈도우키": "Meta", "커맨드": "Meta",
    "캡스락": "CapsLock", "홈": "Home", "엔드": "End",
    "페이지업": "PageUp", "페이지다운": "PageDown",
    "인서트": "Insert", "프린트스크린": "PrintScreen",
    "스크롤락": "ScrollLock", "일시정지": "Pause", "넘락": "NumLock",
    "한영": "HangulMode", "한자": "HanjaMode",
  },
};

function normalizeKeyAlias(raw) {
  if (!raw) return { raw_key: raw, canon_key: null, canon_version: KEY_CANON_TABLE.version, matched: false };
  const normalized = raw.replace(/\s/g, "")
    .replace(/[\uFF01-\uFF5E]/g, (ch) => String.fromCharCode(ch.charCodeAt(0) - 0xFEE0))
    .toLowerCase();
  const canon = KEY_CANON_TABLE.aliases[normalized] ?? null;
  return {
    raw_key: raw,
    canon_key: canon,
    canon_version: KEY_CANON_TABLE.version,
    matched: canon !== null,
  };
}

function recordKeyEvent(e) {
  return {
    schema: "input.key_event.v1",
    raw_key: e.key,
    raw_code: e.code,
    canon_key: normalizeKeyAlias(e.key).canon_key || e.key,
    canon_code: e.code,
    canon_version: KEY_CANON_TABLE.version,
    source: "browser",
    modifiers: {
      ctrl: e.ctrlKey,
      shift: e.shiftKey,
      alt: e.altKey,
      meta: e.metaKey,
    },
    ts: new Date().toISOString(),
  };
}

const WASM_KEY_BITS = {
  w: 1 << 0,
  a: 1 << 1,
  s: 1 << 2,
  d: 1 << 3,
};

const WASM_SETTINGS_KEY = "seamgrim.wasm.settings.v1";
const WASM_SCHEMA_PRESETS_KEY = "seamgrim.wasm.schema_presets.v1";
const WASM_LENS_PRESETS_KEY = "seamgrim.wasm.lens_presets.v1";
const LESSON_PREVIEW_MODE_KEY = "seamgrim.lesson.use_age3_preview.v1";

function loadLessonPreviewMode() {
  try {
    if (typeof localStorage === "undefined") return;
    const raw = localStorage.getItem(LESSON_PREVIEW_MODE_KEY);
    if (raw == null) return;
    state.lessons.useAge3Preview = raw === "1";
  } catch (_) {
    // ignore local storage errors
  }
}

function saveLessonPreviewMode() {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(LESSON_PREVIEW_MODE_KEY, state.lessons.useAge3Preview ? "1" : "0");
  } catch (_) {
    // ignore local storage errors
  }
}
const WASM_SCHEMA_TARGETS = ["graph", "space2d", "table", "text", "structure"];
const WASM_KEY_PRESETS = {
  wasd_arrows: {
    label: "WASD + Arrows",
    map: {
      up: "w,ArrowUp",
      left: "a,ArrowLeft",
      down: "s,ArrowDown",
      right: "d,ArrowRight",
    },
  },
  ijkl_arrows: {
    label: "IJKL + Arrows",
    map: {
      up: "i,ArrowUp",
      left: "j,ArrowLeft",
      down: "k,ArrowDown",
      right: "l,ArrowRight",
    },
  },
  wasd_only: {
    label: "WASD Only",
    map: {
      up: "w",
      left: "a",
      down: "s",
      right: "d",
    },
  },
  arrows_only: {
    label: "Arrows Only",
    map: {
      up: "ArrowUp",
      left: "ArrowLeft",
      down: "ArrowDown",
      right: "ArrowRight",
    },
  },
  custom: {
    label: "Custom",
    map: null,
  },
};

function parseKeyList(raw) {
  return String(raw ?? "")
    .split(/[,\s]+/)
    .map((item) => item.trim().toLowerCase())
    .filter(Boolean);
}

function syncWasmKeyMapFromRaw() {
  state.wasm.keyMap = {
    up: parseKeyList(state.wasm.keyMapRaw.up),
    left: parseKeyList(state.wasm.keyMapRaw.left),
    down: parseKeyList(state.wasm.keyMapRaw.down),
    right: parseKeyList(state.wasm.keyMapRaw.right),
  };
}

function parseSchemaMap(raw) {
  const map = {};
  String(raw ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .forEach((line) => {
      if (line.startsWith("#")) return;
      const [schema, targetRaw] = line.split("=").map((part) => part.trim());
      if (!schema || !targetRaw) return;
      if (WASM_SCHEMA_TARGETS.includes(targetRaw)) {
        map[schema] = targetRaw;
      }
    });
  return map;
}

function parseFixed64Map(raw) {
  const map = {};
  String(raw ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .forEach((line) => {
      if (line.startsWith("#")) return;
      const [targetRaw, tagRaw] = line.split("=").map((part) => part.trim());
      if (!targetRaw || !tagRaw) return;
      if (!targetRaw.startsWith("graph.") && !targetRaw.startsWith("space2d.")) return;
      const list = map[tagRaw] ?? [];
      list.push(targetRaw);
      map[tagRaw] = list;
    });
  return map;
}

function resolveSchemaTarget(schema, obj) {
  if (schema === "seamgrim.graph.v0") return "graph";
  if (schema === "seamgrim.space2d.v0") return "space2d";
  if (schema === "seamgrim.table.v0") return "table";
  if (schema === "seamgrim.text.v0") return "text";
  if (schema === "seamgrim.structure.v0" || schema === "seamgrim.structure") return "structure";
  if (!schema) {
    if (obj?.matrix || (Array.isArray(obj?.columns) && Array.isArray(obj?.rows))) return "table";
    if (Array.isArray(obj?.nodes) && Array.isArray(obj?.edges)) return "structure";
  }
  const custom = state.wasm?.schemaMap?.[schema];
  if (custom) return custom;
  return null;
}

function getWasmSettingsPayload() {
  return {
    fpsLimit: state.wasm.fpsLimit,
    dtMax: state.wasm.dtMax,
    fixedDtEnabled: state.wasm.fixedDtEnabled,
    fixedDtValue: state.wasm.fixedDtValue,
    inputEnabled: state.wasm.inputEnabled,
    patchMode: state.wasm.patchMode,
    langMode: state.wasm.langMode,
    sampleId: state.wasm.sampleId,
    keyMapRaw: state.wasm.keyMapRaw,
    keyPresetId: state.wasm.keyPresetId,
    schemaMapRaw: state.wasm.schemaMapRaw,
    fixed64MapRaw: state.wasm.fixed64MapRaw,
    paramKey: state.wasm.paramKey,
    paramMode: state.wasm.paramMode,
    paramValue: state.wasm.paramValue,
    schemaPresetId: state.wasm.schemaPresetId,
    lensEnabled: Boolean(state.wasm.lens.enabled),
    lensXKey: String(state.wasm.lens.xKey ?? "__tick__"),
    lensYKey: String(state.wasm.lens.yKey ?? ""),
    lensY2Key: String(state.wasm.lens.y2Key ?? ""),
    lensPresetId: String(state.wasm.lens.presetId ?? "custom"),
  };
}

function applyWasmSettings(payload) {
  if (!payload || typeof payload !== "object") return;
  if (Number.isFinite(payload.fpsLimit)) state.wasm.fpsLimit = payload.fpsLimit;
  if (Number.isFinite(payload.dtMax)) state.wasm.dtMax = payload.dtMax;
  if (typeof payload.fixedDtEnabled === "boolean") state.wasm.fixedDtEnabled = payload.fixedDtEnabled;
  if (Number.isFinite(payload.fixedDtValue)) state.wasm.fixedDtValue = payload.fixedDtValue;
  if (typeof payload.inputEnabled === "boolean") state.wasm.inputEnabled = payload.inputEnabled;
  if (typeof payload.patchMode === "boolean") state.wasm.patchMode = payload.patchMode;
  if (typeof payload.langMode === "string") {
    state.wasm.langMode = normalizeLangMode(payload.langMode);
  }
  if (typeof payload.sampleId === "string") {
    state.wasm.sampleId = payload.sampleId;
  }
  if (payload.keyMapRaw && typeof payload.keyMapRaw === "object") {
    state.wasm.keyMapRaw = {
      up: payload.keyMapRaw.up ?? state.wasm.keyMapRaw.up,
      left: payload.keyMapRaw.left ?? state.wasm.keyMapRaw.left,
      down: payload.keyMapRaw.down ?? state.wasm.keyMapRaw.down,
      right: payload.keyMapRaw.right ?? state.wasm.keyMapRaw.right,
    };
  }
  if (typeof payload.keyPresetId === "string") {
    state.wasm.keyPresetId = payload.keyPresetId;
  }
  if (typeof payload.schemaMapRaw === "string") {
    state.wasm.schemaMapRaw = payload.schemaMapRaw;
  }
  if (typeof payload.fixed64MapRaw === "string") {
    state.wasm.fixed64MapRaw = payload.fixed64MapRaw;
  }
  if (typeof payload.paramKey === "string") {
    state.wasm.paramKey = payload.paramKey;
  }
  if (typeof payload.paramMode === "string") {
    state.wasm.paramMode = normalizeWasmParamMode(payload.paramMode);
  }
  if (typeof payload.paramValue === "string") {
    state.wasm.paramValue = payload.paramValue;
  }
  if (typeof payload.schemaPresetId === "string") {
    state.wasm.schemaPresetId = payload.schemaPresetId;
  }
  if (typeof payload.lensEnabled === "boolean") {
    state.wasm.lens.enabled = payload.lensEnabled;
  }
  if (typeof payload.lensXKey === "string") {
    state.wasm.lens.xKey = payload.lensXKey;
  }
  if (typeof payload.lensYKey === "string") {
    state.wasm.lens.yKey = payload.lensYKey;
  }
  if (typeof payload.lensY2Key === "string") {
    state.wasm.lens.y2Key = payload.lensY2Key;
  }
  if (typeof payload.lensPresetId === "string") {
    state.wasm.lens.presetId = payload.lensPresetId;
  }
  syncWasmKeyMapFromRaw();
  state.wasm.schemaMap = parseSchemaMap(state.wasm.schemaMapRaw);
  state.wasm.fixed64Map = parseFixed64Map(state.wasm.fixed64MapRaw);
}

function loadWasmSettings() {
  try {
    if (typeof localStorage === "undefined") return;
    const raw = localStorage.getItem(WASM_SETTINGS_KEY);
    if (!raw) return;
    applyWasmSettings(JSON.parse(raw));
  } catch (err) {
    log(`WASM 설정 로드 실패: ${err?.message ?? err}`);
  }
}

function saveWasmSettings() {
  try {
    if (typeof localStorage === "undefined") return;
    const payload = getWasmSettingsPayload();
    localStorage.setItem(WASM_SETTINGS_KEY, JSON.stringify(payload));
  } catch (err) {
    log(`WASM 설정 저장 실패: ${err?.message ?? err}`);
  }
}

function loadSchemaPresets() {
  try {
    if (typeof localStorage === "undefined") return;
    const raw = localStorage.getItem(WASM_SCHEMA_PRESETS_KEY);
    if (!raw) {
      state.wasm.schemaPresets = { default: state.wasm.schemaMapRaw ?? "" };
      state.wasm.schemaPresetId = "default";
      return;
    }
    const payload = JSON.parse(raw);
    const presets = payload?.presets ?? {};
    const activeId = payload?.activeId ?? "default";
    state.wasm.schemaPresets = { default: "", ...presets };
    state.wasm.schemaPresetId = activeId;
    if (!(activeId in state.wasm.schemaPresets)) {
      state.wasm.schemaPresetId = "default";
    }
  } catch (err) {
    log(`스키마 프리셋 로드 실패: ${err?.message ?? err}`);
  }
}

function saveSchemaPresets() {
  try {
    if (typeof localStorage === "undefined") return;
    const payload = {
      activeId: state.wasm.schemaPresetId,
      presets: state.wasm.schemaPresets,
    };
    localStorage.setItem(WASM_SCHEMA_PRESETS_KEY, JSON.stringify(payload));
  } catch (err) {
    log(`스키마 프리셋 저장 실패: ${err?.message ?? err}`);
  }
}

function loadWasmLensPresets() {
  const loaded = loadLensPresetState({
    storageKey: WASM_LENS_PRESETS_KEY,
    defaultPreset: {
      enabled: false,
      xKey: "__tick__",
      yKey: "",
      y2Key: "",
    },
    preferredPresetId: state.wasm.lens.presetId,
    normalizePreset: normalizeWasmLensPreset,
    onError: (err) => {
      log(`렌즈 프리셋 로드 실패: ${err?.message ?? err}`);
    },
  });
  state.wasm.lens.presets = loaded.presets;
  state.wasm.lens.presetId = loaded.presetId;
}

function saveWasmLensPresets() {
  saveLensPresetState({
    storageKey: WASM_LENS_PRESETS_KEY,
    presetId: state.wasm.lens.presetId,
    presets: state.wasm.lens.presets,
    onError: (err) => {
      log(`렌즈 프리셋 저장 실패: ${err?.message ?? err}`);
    },
  });
}

function wasmKeyBitFromName(name) {
  if (!name) return 0;
  const raw = String(name).toLowerCase();
  const map = state.wasm?.keyMap;
  if (map) {
    if (map.up?.includes(raw)) return WASM_KEY_BITS.w;
    if (map.left?.includes(raw)) return WASM_KEY_BITS.a;
    if (map.down?.includes(raw)) return WASM_KEY_BITS.s;
    if (map.right?.includes(raw)) return WASM_KEY_BITS.d;
  }
  if (["w", "i", "up", "arrowup"].includes(raw)) return WASM_KEY_BITS.w;
  if (["a", "j", "left", "arrowleft"].includes(raw)) return WASM_KEY_BITS.a;
  if (["s", "k", "down", "arrowdown"].includes(raw)) return WASM_KEY_BITS.s;
  if (["d", "l", "right", "arrowright"].includes(raw)) return WASM_KEY_BITS.d;
  return 0;
}

function handleWasmKeyEvent(event, pressed) {
  if (!state.wasm?.enabled || !state.wasm.inputEnabled) return;
  const bit = wasmKeyBitFromName(event.key);
  if (!bit) return;
  if (pressed) {
    state.wasm.keysPressed |= bit;
    state.wasm.lastKeyName = event.key;
  } else {
    state.wasm.keysPressed &= ~bit;
  }
}

function initWasmPointerBindings() {
  const attach = (id) => {
    const canvas = $(id);
    if (!canvas) return;
    const update = (event) => {
      if (!state.wasm?.enabled || !state.wasm.inputEnabled) return;
      const rect = canvas.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;
      state.wasm.pointerX = Math.round(x);
      state.wasm.pointerY = Math.round(y);
    };
    canvas.addEventListener("mousemove", update);
    canvas.addEventListener("pointermove", update);
  };
  attach("canvas");
  attach("canvas-2d");
}

window.addEventListener("keydown", (e) => {
  recordKeyEvent(e);
  handleWasmKeyEvent(e, true);
  if (e.ctrlKey && e.key === "Enter") {
    e.preventDefault();
    runDdnBridge();
  }
  if (e.ctrlKey && e.key.toLowerCase() === "s") {
    e.preventDefault();
    saveSnapshot();
  }
});

window.addEventListener("keyup", (e) => {
  handleWasmKeyEvent(e, false);
});

initTabs();
const workspaceModeBasicBtn = $("workspace-mode-basic");
const workspaceModeAdvancedBtn = $("workspace-mode-advanced");
workspaceModeBasicBtn?.addEventListener("click", () => {
  setWorkspaceMode("basic");
});
workspaceModeAdvancedBtn?.addEventListener("click", () => {
  setWorkspaceMode("advanced");
});
setWorkspaceMode("basic", { ensureTab: false });
setActiveView("view-graph");
updateViewInputsEnabled();
updateSpace2dInputsEnabled();
updateAxisLabels();
applyTimeModeUi();
syncTimeCursorRange();
setContractView("plain");
loadLessonCatalog();
bindWasmControls();
renderSamSummary();
renderGeoulSummary();
renderOverlayList();
renderDdnPresets();
renderInputRegistry();
updateDdnTimeStatus();
state.tableView.pageSize = readNumber("table-page-size", 50) ?? 50;
state.tableView.precision = readNumber("table-precision", 3) ?? 3;
state.structureView.layout = $("structure-layout").value;
state.structureView.nodeSize = readNumber("structure-node-size", 10) ?? 10;
state.structureView.showLabels = $("structure-show-labels").checked;
state.space2dView = getSpace2dViewConfigFromControls();
bindSpace2dCanvasInteractions();
renderLogs();
syncMediaExportButtons();
updateMediaExportStatus("내보내기 상태: 대기");
log("UI 준비 완료");
initWasmVmExample();
updateCompareStatusUI();
const logDockToggle = $("log-dock-toggle");
if (logDockToggle) {
  logDockToggle.addEventListener("click", () => {
    const dock = $("log-dock");
    if (!dock) return;
    dock.classList.toggle("collapsed");
    logDockToggle.textContent = dock.classList.contains("collapsed") ? "펼치기" : "접기";
  });
}
const compareToggle = $("compare-enabled");
if (compareToggle) {
  compareToggle.addEventListener("change", () => {
    setCompareEnabled(compareToggle.checked);
  });
}
const compareFreeze = $("compare-freeze");
if (compareFreeze) {
  compareFreeze.addEventListener("click", () => {
    const run = getActiveRun();
    if (!run) {
      log("baseline으로 고정할 run이 없습니다.");
      return;
    }
    setCompareBaseline(run);
  });
}
const compareRunBtn = $("compare-run");
if (compareRunBtn) {
  compareRunBtn.addEventListener("click", () => {
    if (!state.compare.enabled) {
      log("비교 모드를 먼저 켜주세요.");
      return;
    }
    runDdnBridge();
  });
}
const compareIntervalInput = $("compare-interval");
if (compareIntervalInput) {
  compareIntervalInput.addEventListener("change", () => {
    const raw = Number(compareIntervalInput.value ?? state.compare.sequence.intervalMs);
    state.compare.sequence.intervalMs = Math.min(5000, Math.max(120, Number.isFinite(raw) ? raw : 800));
    compareIntervalInput.value = String(state.compare.sequence.intervalMs);
    if (state.compare.sequence.playing) {
      startCompareSequence();
    } else {
      updateCompareStatusUI();
    }
  });
}
const comparePlayBtn = $("compare-play");
if (comparePlayBtn) {
  comparePlayBtn.addEventListener("click", () => {
    startCompareSequence();
  });
}
const compareStopBtn = $("compare-stop");
if (compareStopBtn) {
  compareStopBtn.addEventListener("click", () => {
    stopCompareSequence({ restoreVisibility: true });
  });
}
