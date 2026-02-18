// DDN Playground ??educational DDN runtime with sample management
// Based on wasm_smoke.js, refactored for playground use.
import {
  applyWasmParamDraftToControls,
  applyWasmParamFromUi,
  applyWasmLogicAndDispatchState,
  KEY_BITS,
  bindSpace2dCanvasPanZoom,
  buildObservationLensGraph,
  composeObservationRenderState,
  buildSourcePreview,
  collectStateResourceLines,
  createManagedRafStepLoop,
  createEmptyObservationState,
  createObservationLensState,
  gatherInputFromDom,
  applyLensPresetSelectionState,
  deleteLensPresetFromState,
  loadLensPresetState,
  initializeObservationLensUi,
  markLensPresetCustomState,
  normalizeLensPresetConfig,
  readWasmParamDraftFromControls,
  updateLensSelectorsFromObservation,
  renderGraphOrSpace2dCanvas,
  renderObservationChannelList,
  refreshLensPresetSelectElement,
  resetObservationRuntimeCaches,
  saveLensPresetToState,
  saveLensPresetState,
  stepWasmAndDispatchState,
} from "./wasm_page_common.js";
import {
  extractObservationChannelsFromState,
  extractStructuredViewsFromState,
} from "./seamgrim_runtime_state.js";
import { createWasmVm } from "./runtime/index.js";

const $ = (id) => document.getElementById(id);

// ??????????????????????????????????????????????
// Sample manifest (extend this for curriculum)
// ??????????????????????????????????????????????
const SAMPLES = [
  // ?? 湲곗큹: lang/ ?뚯꽌 ?명솚 (WASM?먯꽌 諛붾줈 ?ㅽ뻾) ??
  {
    id: "hello_world",
    label: "Hello World",
    description: "媛??媛꾨떒??DDN ?꾨줈洹몃옩. ?몃? ?곸옄 ?섎굹.",
    category: "湲곗큹",
    source: [
      "留ㅽ떛:?吏곸뵪 = {",
      "  蹂닿컻_洹몃┝??媛濡?<- 160.",
      "  蹂닿컻_洹몃┝???몃줈 <- 120.",
      "",
      '  box <- ("寃?, "#蹂닿컻/2D.Rect", "x", 40, "y", 30, "w", 80, "h", 60, "梨꾩???, "#facc15ff") 吏앸쭪異?',
      "  蹂닿컻_洹몃┝??紐⑸줉 <- (box) 李⑤┝.",
      "}.",
    ].join("\n"),
  },
  {
    id: "moving_dot",
    label: "?吏곸씠????,
    description: "?꾨젅?꾩닔濡??먯씠 ?媛곸꽑 ?대룞?⑸땲??",
    category: "湲곗큹",
    source: [
      "留ㅽ떛:?吏곸뵪 = {",
      "  蹂닿컻_洹몃┝??媛濡?<- 200.",
      "  蹂닿컻_洹몃┝???몃줈 <- 120.",
      "",
      '  rect <- ("寃?, "#蹂닿컻/2D.Rect", "x", (20 + (?꾨젅?꾩닔 % 160)), "y", (20 + ((?꾨젅?꾩닔 * 2) % 80)), "w", 6, "h", 6, "梨꾩???, "#f59e0bff") 吏앸쭪異?',
      "  蹂닿컻_洹몃┝??紐⑸줉 <- (rect) 李⑤┝.",
      "",
      "  ?꾨젅?꾩닔 <- (?꾨젅?꾩닔 + 1).",
      "}.",
    ].join("\n"),
  },
  {
    id: "two_rects",
    label: "?ш컖????媛?,
    description: "鍮④컯/?뚮옉 ?ш컖?뺤씠 諛섎? 諛⑺뼢?쇰줈 ?대룞?⑸땲??",
    category: "湲곗큹",
    source: [
      "留ㅽ떛:?吏곸뵪 = {",
      "  蹂닿컻_洹몃┝??媛濡?<- 240.",
      "  蹂닿컻_洹몃┝???몃줈 <- 120.",
      "",
      '  r1 <- ("寃?, "#蹂닿컻/2D.Rect", "x", (?꾨젅?꾩닔 % 200), "y", 30, "w", 20, "h", 20, "梨꾩???, "#ef4444ff") 吏앸쭪異?',
      '  r2 <- ("寃?, "#蹂닿컻/2D.Rect", "x", (200 - (?꾨젅?꾩닔 % 200)), "y", 70, "w", 20, "h", 20, "梨꾩???, "#3b82f6ff") 吏앸쭪異?',
      "  蹂닿컻_洹몃┝??紐⑸줉 <- (r1, r2) 李⑤┝.",
      "",
      "  ?꾨젅?꾩닔 <- (?꾨젅?꾩닔 + 1).",
      "}.",
    ].join("\n"),
  },
  {
    id: "growing_box",
    label: "而ㅼ????곸옄",
    description: "?곸옄 ?ш린媛 ?꾨젅?꾩닔???곕씪 而ㅼ죱???묒븘吏묐땲??",
    category: "湲곗큹",
    source: [
      "留ㅽ떛:?吏곸뵪 = {",
      "  蹂닿컻_洹몃┝??媛濡?<- 200.",
      "  蹂닿컻_洹몃┝???몃줈 <- 200.",
      "",
      "  ?ш린 <- (10 + ((?꾨젅?꾩닔 * 2) % 80)).",
      "  ?щ갚 <- ((200 - ?ш린) / 2).",
      '  box <- ("寃?, "#蹂닿컻/2D.Rect", "x", ?щ갚, "y", ?щ갚, "w", ?ш린, "h", ?ш린, "梨꾩???, "#22c55eff") 吏앸쭪異?',
      "  蹂닿컻_洹몃┝??紐⑸줉 <- (box) 李⑤┝.",
      "",
      "  ?꾨젅?꾩닔 <- (?꾨젅?꾩닔 + 1).",
      "}.",
    ].join("\n"),
  },
  {
    id: "bouncing_dot",
    label: "?뺢린????,
    description: "?먯씠 醫뚯슦濡??뺣났?⑸땲?? (?쇰븣 議곌굔臾??쒖슜)",
    category: "湲곗큹",
    source: [
      "留ㅽ떛:?吏곸뵪 = {",
      "  蹂닿컻_洹몃┝??媛濡?<- 200.",
      "  蹂닿컻_洹몃┝???몃줈 <- 100.",
      "",
      "  // 二쇨린 = 160 ?꾨젅?? 諛섏＜湲?= 80",
      "  ?꾩튂 <- (?꾨젅?꾩닔 % 160).",
      "  (?꾩튂 > 80) ?쇰븣 {",
      "    ?꾩튂 <- (160 - ?꾩튂).",
      "  }",
      "  x <- (10 + (?꾩튂 * 2)).",
      "",
      '  dot <- ("寃?, "#蹂닿컻/2D.Rect", "x", x, "y", 42, "w", 16, "h", 16, "梨꾩???, "#a855f7ff") 吏앸쭪異?',
      "  蹂닿컻_洹몃┝??紐⑸줉 <- (dot) 李⑤┝.",
      "",
      "  ?꾨젅?꾩닔 <- (?꾨젅?꾩닔 + 1).",
      "}.",
    ].join("\n"),
  },
  // ?? teul-cli ?꾩슜 (?뚯씪 fetch ?꾩슂) ??
  {
    id: "line_graph_export",
    label: "吏곸꽑 洹몃옒??(teul-cli)",
    file: "../samples/01_line_graph_export.ddn",
    description: "teul-cli ?꾩슜. ?쒕쾭瑜?seamgrim_ui_mvp/ ?먯꽌 ?쒖옉?댁빞 ?⑸땲??",
    category: "怨좉툒 (teul-cli)",
    source: null,
  },
  {
    id: "parabola",
    label: "?щЪ??(teul-cli)",
    file: "../samples/02_parabola_export.ddn",
    description: "teul-cli ?꾩슜. ?쒕쾭瑜?seamgrim_ui_mvp/ ?먯꽌 ?쒖옉?댁빞 ?⑸땲??",
    category: "怨좉툒 (teul-cli)",
    source: null,
  },
  {
    id: "calculus_ascii",
    label: "誘몄쟻遺?蹂??(WASM)",
    file: "../samples/04_calculus_ascii.ddn",
    description: "WASM/teul-cli 怨듭슜. ?쒕쾭瑜?seamgrim_ui_mvp/ ?먯꽌 ?쒖옉?댁빞 ?⑸땲??",
    category: "怨좉툒 (WASM/teul-cli)",
    source: null,
  },
];

// ??????????????????????????????????????????????
// DOM references
// ??????????????????????????????????????????????
const statusBox = $("status");
const rawBox = $("raw-json");
const toggleRaw = $("toggle-raw");
const errorBox = $("error-box");
const graphCanvas = $("graph-canvas");
const gridToggle = $("graph-show-grid");
const axisToggle = $("graph-show-axis");
const patchPreferToggle = $("patch-prefer");
const sampleReinitToggle = $("sample-reinit");
const resetBtn = $("btn-reset");
const paramShowFixed64 = $("param-show-fixed64");
const paramShowValue = $("param-show-value");
const paramKeyInput = $("param-key");
const paramModeSelect = $("param-mode");
const paramInput = $("param-input");
const paramApplyBtn = $("param-apply");
const paramApplyStatus = $("param-apply-status");
const channelListBox = $("channel-list");
const space2dAutoFitToggle = $("space2d-auto-fit");
const space2dResetViewBtn = $("space2d-reset-view");
const lensEnableToggle = $("lens-enable");
const lensXSelect = $("lens-x-key");
const lensYSelect = $("lens-y-key");
const lensY2Select = $("lens-y2-key");
const lensPresetSelect = $("lens-preset");
const lensPresetNameInput = $("lens-preset-name");
const lensPresetSaveBtn = $("lens-preset-save");
const lensPresetDeleteBtn = $("lens-preset-delete");
const sampleSelect = $("sample-select");
const guideSidebar = $("guide-sidebar");

// ??????????????????????????????????????????????
// State
// ??????????????????????????????????????????????
let loopActive = false;
let latestGraph = null;
let latestSpace2d = null;
let latestLensGraph = null;
let latestGraphSource = "-";
let latestPatch = [];
let lastState = null;
let latestObservation = createEmptyObservationState({ includeValues: true });
const space2dView = {
  autoFit: true,
  zoom: 1,
  panPx: 0,
  panPy: 0,
  dragging: false,
  lastX: 0,
  lastY: 0,
};
const lensState = createObservationLensState({
  enabled: false,
  xKey: "__tick__",
  yKey: "",
  y2Key: "",
  presetId: "custom",
  maxPoints: 400,
  lastFrameToken: "",
});
const LENS_PRESET_STORAGE_KEY = "seamgrim.playground.lens_presets.v1";
const SESSION_STORAGE_KEY = "seamgrim_session_v0";
let sessionSaveTimer = null;
const loopController = createManagedRafStepLoop({
  getFps: () => Math.max(1, Number($("input-fps")?.value ?? 30) || 30),
  onStep: () => stepOnce(),
  onError: (err) => {
    showError(`猷⑦봽 ?ㅻ쪟: ${String(err?.message ?? err)}`);
  },
  isActive: () => loopActive,
  setActive: (next) => {
    loopActive = Boolean(next);
  },
});

function setSpace2dAutoFit(enabled) {
  space2dView.autoFit = Boolean(enabled);
  if (space2dAutoFitToggle) {
    space2dAutoFitToggle.checked = space2dView.autoFit;
  }
}

function resetSpace2dView({ forceAutoFit = false } = {}) {
  space2dView.zoom = 1;
  space2dView.panPx = 0;
  space2dView.panPy = 0;
  if (forceAutoFit) {
    setSpace2dAutoFit(true);
  }
}

function bindSpace2dInteractions() {
  bindSpace2dCanvasPanZoom({
    canvas: graphCanvas,
    viewState: space2dView,
    hasSpace2d: () => Boolean(latestSpace2d),
    setAutoFit: setSpace2dAutoFit,
    onRender: () => {
      renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph);
    },
  });
}

function renderChannelList(observation) {
  renderObservationChannelList({
    element: channelListBox,
    observation,
    maxRows: 80,
    numericMode: "plain",
    target: "text",
  });
}

function syncLensConfigFromDom() {
  lensState.enabled = lensEnableToggle?.checked ?? false;
  lensState.xKey = String(lensXSelect?.value ?? "__tick__");
  lensState.yKey = String(lensYSelect?.value ?? "");
  lensState.y2Key = String(lensY2Select?.value ?? "");
}

function normalizeLensPreset(raw) {
  return normalizeLensPresetConfig(raw);
}

function currentLensPreset() {
  return {
    enabled: Boolean(lensState.enabled),
    xKey: String(lensState.xKey ?? "__tick__"),
    yKey: String(lensState.yKey ?? ""),
    y2Key: String(lensState.y2Key ?? ""),
  };
}

function saveLensPresets() {
  saveLensPresetState({
    storageKey: LENS_PRESET_STORAGE_KEY,
    presetId: lensState.presetId,
    presets: lensState.presets,
  });
}

function loadLensPresets() {
  const loaded = loadLensPresetState({
    storageKey: LENS_PRESET_STORAGE_KEY,
    defaultPreset: {
      enabled: false,
      xKey: "__tick__",
      yKey: "",
      y2Key: "",
    },
  });
  lensState.presets = loaded.presets;
  lensState.presetId = loaded.presetId;
}

function refreshLensPresetSelect() {
  lensState.presetId = refreshLensPresetSelectElement({
    selectEl: lensPresetSelect,
    presets: lensState.presets,
    presetId: lensState.presetId,
  });
}

function markLensPresetCustom() {
  markLensPresetCustomState({
    lensState,
    selectEl: lensPresetSelect,
    nameInputEl: lensPresetNameInput,
  });
}

function applyLensStateToDom() {
  if (lensEnableToggle) lensEnableToggle.checked = lensState.enabled;
  updateLensSelectors(latestObservation);
  syncLensConfigFromDom();
}

function applyLensSelectionToRender({ markCustom = true } = {}) {
  if (markCustom) {
    markLensPresetCustom();
  }
  latestLensGraph = buildObservationLensGraph({
    lensState,
    source: "observation-lens",
    includeSample: false,
  });
  renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph);
  saveLensPresets();
}

function applyLensPresetSelection(id) {
  const result = applyLensPresetSelectionState({
    lensState,
    id,
    normalizePreset: normalizeLensPreset,
  });
  if (!result.ok) return;
  if (result.mode === "custom") {
    if (lensPresetNameInput) lensPresetNameInput.value = "";
    refreshLensPresetSelect();
    saveLensPresets();
    return;
  }
  applyLensStateToDom();
  applyLensSelectionToRender({ markCustom: false });
  if (lensPresetNameInput) lensPresetNameInput.value = result.presetId;
  refreshLensPresetSelect();
  saveLensPresets();
}

function updateLensSelectors(observation) {
  updateLensSelectorsFromObservation({
    observation,
    lensState,
    xSelect: lensXSelect,
    ySelect: lensYSelect,
    y2Select: lensY2Select,
    onSynced: syncLensConfigFromDom,
  });
}

function resetWasm({ silent = false } = {}) {
  stopLoop();
  if (wasmVm) {
    wasmVm.invalidate();
    wasmVm = null;
  }
  lastState = null;
  latestGraph = null;
  latestSpace2d = null;
  const resetCaches = resetObservationRuntimeCaches({
    lensState,
    lastFrameToken: "",
    channelListElement: channelListBox,
    resetSpace2dView,
    observationFactory: () => createEmptyObservationState({ includeValues: true }),
  });
  latestLensGraph = resetCaches.lensGraph;
  latestObservation = resetCaches.observation;
  latestPatch = [];
  if (rawBox) rawBox.value = "";
  clearError();
  renderGraphOrSpace2d(null, null);
  if (!silent) {
    setStatus(["status: vm reset", "?섑뵆???좏깮?섍퀬 '濡쒖쭅 ?곸슜'???꾨Ⅴ?몄슂."]);
  }
}

// ??????????????????????????????????????????????
// Sample dropdown setup
// ??????????????????????????????????????????????
function populateSampleDropdown() {
  if (!sampleSelect) return;
  let lastCat = "";
  let optgroup = null;
  SAMPLES.forEach((s) => {
    if (s.category !== lastCat) {
      optgroup = document.createElement("optgroup");
      optgroup.label = s.category;
      sampleSelect.appendChild(optgroup);
      lastCat = s.category;
    }
    const opt = document.createElement("option");
    opt.value = s.id;
    opt.textContent = s.label;
    if (s.description) opt.title = s.description;
    (optgroup || sampleSelect).appendChild(opt);
  });
}

// ??????????????????????????????????????????????
// Guide sidebar toggle
// ??????????????????????????????????????????????
function toggleGuide() {
  if (guideSidebar) {
    guideSidebar.classList.toggle("collapsed");
  }
}

// ??????????????????????????????????????????????
// Status / Error display
// ??????????????????????????????????????????????
function setStatus(lines) {
  if (!statusBox) return;
  statusBox.textContent = lines.join("\n");
}

function showError(message) {
  if (!errorBox) return;
  errorBox.textContent = normalizeWasmErrorMessage(message);
  errorBox.classList.add("visible");
}

function clearError() {
  if (!errorBox) return;
  errorBox.textContent = "";
  errorBox.classList.remove("visible");
}

function normalizeWasmErrorMessage(messageOrErr) {
  const raw = String(messageOrErr?.message ?? messageOrErr ?? "");
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object") {
      if (typeof parsed.hint === "string" && parsed.hint.trim()) return parsed.hint.trim();
      if (typeof parsed.detail === "string" && parsed.detail.trim()) return parsed.detail.trim();
    }
  } catch (_) {
    // no-op
  }
  return raw;
}

function setParamApplyStatus(message) {
  if (!paramApplyStatus) return;
  paramApplyStatus.textContent = message;
}

function saveSession(state) {
  const sourceEl = $("ddn-source");
  const session = {
    schema: "seamgrim.session.v0",
    ts: new Date().toISOString(),
    ddn_text: sourceEl?.value ?? "",
    last_state_hash: state?.state_hash ?? null,
    last_view_hash: state?.view_hash ?? null,
    lens: currentLensPreset(),
    param: readWasmParamDraftFromControls({
      keyInput: paramKeyInput,
      modeSelect: paramModeSelect,
      valueInput: paramInput,
    }),
  };
  try {
    localStorage.setItem(SESSION_STORAGE_KEY, JSON.stringify(session));
  } catch (_) {
    // quota 珥덇낵 ?깆? 議곗슜??臾댁떆
  }
}

function restoreSession() {
  try {
    const raw = localStorage.getItem(SESSION_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || parsed.schema !== "seamgrim.session.v0") return null;
    return parsed;
  } catch (_) {
    return null;
  }
}

function scheduleSave(state) {
  clearTimeout(sessionSaveTimer);
  sessionSaveTimer = setTimeout(() => saveSession(state), 2000);
}

function applyRestoredSession(session) {
  if (!session || typeof session !== "object") return;
  if (typeof session.ddn_text === "string" && session.ddn_text.trim()) {
    const sourceEl = $("ddn-source");
    if (sourceEl) sourceEl.value = session.ddn_text;
  }
  const lens = session.lens;
  if (lens && typeof lens === "object") {
    lensState.enabled = Boolean(lens.enabled);
    lensState.xKey = String(lens.xKey ?? "__tick__");
    lensState.yKey = String(lens.yKey ?? "");
    lensState.y2Key = String(lens.y2Key ?? "");
  }
  const param = session.param;
  if (param && typeof param === "object") {
    applyWasmParamDraftToControls({
      keyInput: paramKeyInput,
      modeSelect: paramModeSelect,
      valueInput: paramInput,
      draft: param,
    });
  }
  applyLensStateToDom();
  applyLensSelectionToRender({ markCustom: false });
}

let wasmVm = null;

async function getOrCreateWasmVm(source = "") {
  if (wasmVm) return wasmVm;
  wasmVm = await createWasmVm({
    cacheBust: Date.now(),
    setStatus,
    clearStatusError: clearError,
    ddnSourceText: source,
    missingExportMessage: "DdnWasmVm export missing ??wasm build with --features wasm required",
    formatReadyStatus: ({ buildInfo }) =>
      [buildInfo ? `status: wasm ready | ${buildInfo}` : "status: wasm ready"],
    formatFallbackStatus: () => ["status: wasm ready (fallback)", "?섑뵆???좏깮?섍퀬 '濡쒖쭅 ?곸슜'???꾨Ⅴ?몄슂."],
  });
  return wasmVm;
}

async function ensureWasm(source) {
  const handle = await getOrCreateWasmVm(source ?? "");
  return handle.ensureClient(source ?? "");
}

// ??????????????????????????????????????????????
// Input gathering
// ??????????????????????????????????????????????
function gatherInput() {
  return gatherInputFromDom($, KEY_BITS);
}

// ??????????????????????????????????????????????
// State rendering
// ??????????????????????????????????????????????
function renderState(state) {
  if (!state) return;
  lastState = state;
  clearError();
  const observation = extractObservationChannelsFromState(state);
  latestObservation = observation;
  updateLensSelectors(observation);
  const patchCount = Array.isArray(state.patch) ? state.patch.length : 0;
  const renderStateInfo = composeObservationRenderState({
    stateJson: state,
    observation,
    lensState,
    graphOptions: {
      source: "observation-lens",
      includeSample: false,
    },
    patchCount,
    tickTimeDigits: 2,
  });
  latestLensGraph = renderStateInfo.lensGraph;
  updateViewsFromState(state);
  setStatus([...renderStateInfo.statusLines, `graph_source: ${latestGraphSource}`]);
  renderChannelList(observation);
  if (rawBox) rawBox.value = JSON.stringify(state, null, 2);
  latestPatch = Array.isArray(state.patch) ? state.patch : [];
  renderParamList(state);
  renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph);
  scheduleSave(state);
}

function renderParamList(state) {
  const showFixed = paramShowFixed64?.checked ?? true;
  const showValue = paramShowValue?.checked ?? true;
  const lines = collectStateResourceLines(state, { showFixed, showValue });
  // Append param info to status
  if (lines.length) {
    const current = statusBox?.textContent ?? "";
    statusBox.textContent = current + "\n---\n" + lines.join("\n");
  }
}

// ??????????????????????????????????????????????
// Graph / Space2D extraction from state
// ??????????????????????????????????????????????
function updateViewsFromState(state) {
  if (!graphCanvas) return;
  const preferPatch = patchPreferToggle?.checked ?? false;
  const views = extractStructuredViewsFromState(state, { preferPatch });
  latestGraph = views.graph ?? null;
  latestSpace2d = views.space2d ?? null;
  latestGraphSource = views.graphSource ?? "-";
}

// ??????????????????????????????????????????????
// Graph rendering
// ??????????????????????????????????????????????
function renderGraphOrSpace2d(graph, space2d, lensGraph = null) {
  renderGraphOrSpace2dCanvas({
    canvas: graphCanvas,
    graph,
    space2d,
    lensGraph,
    viewState: space2dView,
    showGraphGrid: gridToggle?.checked ?? true,
    showGraphAxis: axisToggle?.checked ?? true,
    showSpaceGrid: gridToggle?.checked ?? false,
    showSpaceAxis: axisToggle?.checked ?? false,
    graphPreference: "prefer_non_empty_graph",
  });
}

// ??????????????????????????????????????????????
// Core actions
// ??????????????????????????????????????????????
async function applyLogic() {
  clearError();
  const source = $("ddn-source").value ?? "";
  try {
    const result = await applyWasmLogicAndDispatchState({
      sourceText: source,
      ensureWasm,
      resetCachesOptions: {
        lensState,
        lastFrameToken: "",
        channelListElement: channelListBox,
        resetSpace2dView,
        observationFactory: () => createEmptyObservationState({ includeValues: true }),
      },
      patchMode: false,
      onFull: renderState,
    });
    const resetCaches = result.resetCaches;
    latestLensGraph = resetCaches.lensGraph;
    latestObservation = resetCaches.observation;
  } catch (err) {
    showError(`濡쒖쭅 ?곸슜 ?ㅻ쪟: ${normalizeWasmErrorMessage(err)}`);
  }
}

async function stepOnce() {
  const source = $("ddn-source").value ?? "";
  try {
    await stepWasmAndDispatchState({
      sourceText: source,
      ensureWasm,
      gatherInput,
      patchMode: false,
      onFull: renderState,
    });
  } catch (err) {
    showError(`?ㅽ뀦 ?ㅻ쪟: ${normalizeWasmErrorMessage(err)}`);
  }
}

async function applyParamFromControls() {
  const source = $("ddn-source").value ?? "";
  const client = await ensureWasm(source);
  const applied = applyWasmParamFromUi({
    client,
    key: paramKeyInput?.value ?? "",
    rawValue: paramInput?.value ?? "",
    mode: paramModeSelect?.value ?? "scalar",
    errorPrefix: "playground param",
  });
  if (!applied.ok) {
    setParamApplyStatus(`param: ?ㅽ뙣 (${applied.error ?? "unknown"})`);
    throw new Error(applied.error ?? "param apply failed");
  }
  const state = client.getStateParsed();
  renderState(state);
  const hash = applied.result?.state_hash ?? state?.state_hash ?? "-";
  setParamApplyStatus(
    `param: ok key=${applied.key} mode=${applied.mode} kind=${applied.valueKind} hash=${hash}`,
  );
}

function startLoop() {
  clearError();
  loopController.start();
}

function stopLoop() {
  loopController.stop();
}

// ??????????????????????????????????????????????
// Sample loading
// ??????????????????????????????????????????????
async function loadSampleById(id) {
  const sample = SAMPLES.find((s) => s.id === id);
  if (!sample) {
    console.warn(`[playground] sample not found: id=${id}`);
    return;
  }
  if (sampleReinitToggle?.checked) {
    resetWasm({ silent: true });
  }
  clearError();
  setStatus([`status: loading ${sample.label}...`]);

  // 1) fetch ?쒕룄
  if (sample.file) {
    try {
      const cacheBust = `v=${Date.now()}`;
      const res = await fetch(`${sample.file}?${cacheBust}`, { cache: "no-store" });
      if (res.ok) {
        const text = await res.text();
        $("ddn-source").value = text;
        setStatus([`status: "${sample.label}" loaded`, sample.description || ""]);
        return;
      }
      console.warn(`[playground] fetch ${sample.file}: ${res.status}, trying inline`);
    } catch (err) {
      console.warn(`[playground] fetch failed: ${err.message}, trying inline`);
    }
  }

  // 2) ?몃씪???뚯뒪 fallback
  if (sample.source) {
    $("ddn-source").value = sample.source;
    setStatus([`status: "${sample.label}" loaded (inline)`, sample.description || ""]);
    return;
  }

  // 3) ?????놁쓬
  setStatus([`status: sample load failed`]);
  showError(`?섑뵆 "${sample.label}" 濡쒕뱶 ?ㅽ뙣.\nHTTP ?쒕쾭瑜?samples/ ?곸쐞 ?붾젆?곕━?먯꽌 ?쒖옉?섏꽭??`);
}

// ??????????????????????????????????????????????
// Event wiring
// ??????????????????????????????????????????????
function setupEvents() {
  setSpace2dAutoFit(true);
  if (graphCanvas) graphCanvas.style.touchAction = "none";
  bindSpace2dInteractions();
  loadLensPresets();
  initializeObservationLensUi({
    lensState,
    observation: latestObservation,
    lensEnableToggle,
    lensPresetSelect,
    lensPresetNameInput,
    updateLensSelectors,
    syncLensConfigFromDom,
    refreshLensPresetSelect,
  });
  applyRestoredSession(restoreSession());

  // Guide toggle
  $("btn-guide")?.addEventListener("click", toggleGuide);

  // Sample dropdown
  sampleSelect?.addEventListener("change", () => {
    const id = sampleSelect.value;
    console.log(`[playground] sample selected: id=${id}`);
    if (id) loadSampleById(id).catch((err) => {
      console.error("[playground] unhandled sample load error:", err);
      showError(String(err));
    });
  });

  // WASM init
  $("btn-init")?.addEventListener("click", () => {
    const source = $("ddn-source").value ?? "";
    ensureWasm(source).catch((err) => {
      const debugInfo = wasmVm?.getDebugInfo?.() ?? {};
      const lastBuildInfo = debugInfo.buildInfo ?? "";
      const lastPreprocessed = debugInfo.preprocessed ?? "";
      const preview = buildSourcePreview(source, 20);
      const preprocessedPreview = buildSourcePreview(lastPreprocessed, 20);
      const header = lastBuildInfo
        ? `${String(err)} | ${lastBuildInfo}`
        : String(err);
      showError([header, "\nsource:", ...preview, "\npreprocessed:", ...preprocessedPreview].join("\n"));
    });
  });
  resetBtn?.addEventListener("click", () => resetWasm());

  // Core controls
  $("btn-apply")?.addEventListener("click", () => applyLogic());
  $("btn-step")?.addEventListener("click", () => stepOnce());
  $("btn-start")?.addEventListener("click", startLoop);
  $("btn-stop")?.addEventListener("click", stopLoop);

  // Toggle raw JSON
  toggleRaw?.addEventListener("change", () => {
    if (rawBox) rawBox.style.display = toggleRaw.checked ? "block" : "none";
  });

  // Param toggles
  paramShowFixed64?.addEventListener("change", () => { if (lastState) renderState(lastState); });
  paramShowValue?.addEventListener("change", () => { if (lastState) renderState(lastState); });
  paramApplyBtn?.addEventListener("click", () => {
    applyParamFromControls().catch((err) => {
      showError(`?뚮씪誘명꽣 ?곸슜 ?ㅻ쪟: ${normalizeWasmErrorMessage(err)}`);
    });
  });
  paramInput?.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    applyParamFromControls().catch((err) => {
      showError(`?뚮씪誘명꽣 ?곸슜 ?ㅻ쪟: ${normalizeWasmErrorMessage(err)}`);
    });
  });

  // Graph toggles
  gridToggle?.addEventListener("change", () => renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph));
  axisToggle?.addEventListener("change", () => renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph));
  patchPreferToggle?.addEventListener("change", () => { if (lastState) renderState(lastState); });
  lensEnableToggle?.addEventListener("change", () => {
    syncLensConfigFromDom();
    applyLensSelectionToRender();
  });
  lensXSelect?.addEventListener("change", () => {
    syncLensConfigFromDom();
    applyLensSelectionToRender();
  });
  lensYSelect?.addEventListener("change", () => {
    syncLensConfigFromDom();
    applyLensSelectionToRender();
  });
  lensY2Select?.addEventListener("change", () => {
    syncLensConfigFromDom();
    applyLensSelectionToRender();
  });
  lensPresetSelect?.addEventListener("change", () => {
    applyLensPresetSelection(lensPresetSelect.value);
  });
  lensPresetSaveBtn?.addEventListener("click", () => {
    const result = saveLensPresetToState({
      lensState,
      presetName: lensPresetNameInput?.value,
      currentPreset: currentLensPreset(),
      normalizePreset: normalizeLensPreset,
    });
    if (!result.ok) return;
    refreshLensPresetSelect();
    if (lensPresetSelect) lensPresetSelect.value = result.presetId;
    if (lensPresetNameInput) lensPresetNameInput.value = result.presetId;
    saveLensPresets();
    scheduleSave(lastState);
    setStatus([`status: ?뚯쫰 ?꾨━?????(${result.presetId})`]);
  });
  lensPresetDeleteBtn?.addEventListener("click", () => {
    const result = deleteLensPresetFromState({ lensState });
    if (!result.ok) return;
    refreshLensPresetSelect();
    if (lensPresetNameInput) lensPresetNameInput.value = "";
    saveLensPresets();
    scheduleSave(lastState);
    setStatus([`status: ?뚯쫰 ?꾨━????젣 (${result.presetId})`]);
  });
  space2dAutoFitToggle?.addEventListener("change", () => {
    setSpace2dAutoFit(space2dAutoFitToggle.checked);
    renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph);
  });
  space2dResetViewBtn?.addEventListener("click", () => {
    resetSpace2dView({ forceAutoFit: true });
    renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph);
  });

  // Keyboard shortcuts
  document.addEventListener("keydown", (e) => {
    // Ctrl+Enter = step once
    if (e.ctrlKey && e.key === "Enter") {
      e.preventDefault();
      stepOnce();
    }
    // Ctrl+Shift+Enter = apply logic
    if (e.ctrlKey && e.shiftKey && e.key === "Enter") {
      e.preventDefault();
      applyLogic();
    }
  });

  $("ddn-source")?.addEventListener("input", () => {
    scheduleSave(lastState);
  });
  paramKeyInput?.addEventListener("input", () => scheduleSave(lastState));
  paramModeSelect?.addEventListener("change", () => scheduleSave(lastState));
  paramInput?.addEventListener("input", () => scheduleSave(lastState));
}

// ??????????????????????????????????????????????
// Curriculum extension point
// ??????????????????????????????????????????????
// To add curriculum content:
// 1. Add entries to SAMPLES array above with category grouping
// 2. Put .ddn files in ../samples/ directory
// 3. Optionally add guide sections by appending to #guide-curriculum-slot:
//
//    const slot = document.getElementById("guide-curriculum-slot");
//    slot.innerHTML = `<h3>Lesson 1: ...</h3><p>...</p>`;
//
// Or load curriculum data from an external JSON:
//
//    fetch("../curriculum/lessons.json")
//      .then(r => r.json())
//      .then(lessons => { ... populate SAMPLES and guide ... });

// ??????????????????????????????????????????????
// Init
// ??????????????????????????????????????????????
populateSampleDropdown();
setupEvents();

if (window.location.protocol === "file:") {
  setStatus([
    "status: file:// protocol detected",
    "WASM/ES module requires HTTP server.",
    "python -m http.server 8080",
    "then open http://localhost:8080/playground.html",
  ]);
}
