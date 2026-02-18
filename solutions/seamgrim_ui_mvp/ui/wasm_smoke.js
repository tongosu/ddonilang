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
  dumpSourceDebug,
  gatherInputFromDom,
  applyLensPresetSelectionState,
  deleteLensPresetFromState,
  loadLensPresetState,
  initializeObservationLensUi,
  markLensPresetCustomState,
  normalizeLensPresetConfig,
  readWasmParamDraftFromControls,
  loadWasmParamDraftState,
  saveWasmParamDraftState,
  updateLensSelectorsFromObservation,
  renderGraphOrSpace2dCanvas,
  renderObservationChannelList,
  refreshLensPresetSelectElement,
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

let loopActive = false;
let latestGraph = null;
let latestSpace2d = null;
let latestLensGraph = null;
let latestGraphSource = "-";
let latestPatch = null;
let lastState = null;
let latestObservation = createEmptyObservationState({ includeValues: true });

const statusBox = $("status");
const rawBox = $("raw-json");
const toggleRaw = $("toggle-raw");
const patchListBox = $("patch-list");
const paramListBox = $("param-list");
const paramKeyInput = $("param-key");
const paramModeSelect = $("param-mode");
const paramInput = $("param-input");
const paramApplyBtn = $("param-apply");
const paramApplyStatus = $("param-apply-status");
const channelListBox = $("channel-list");
const graphCanvas = $("graph-canvas");
const gridToggle = $("graph-show-grid");
const axisToggle = $("graph-show-axis");
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
const patchFilterResource = $("patch-filter-resource");
const patchFilterComponent = $("patch-filter-component");
const patchFilterSignal = $("patch-filter-signal");
const patchPreferToggle = $("patch-prefer");
const paramShowFixed64 = $("param-show-fixed64");
const paramShowValue = $("param-show-value");
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
const LENS_PRESET_STORAGE_KEY = "seamgrim.wasm_smoke.lens_presets.v1";
const PARAM_STORAGE_KEY = "seamgrim.wasm_smoke.params.v1";
const loopController = createManagedRafStepLoop({
  getFps: () => Math.max(1, Number($("input-fps")?.value ?? 30) || 30),
  onStep: () => stepOnce(),
  onError: (err) => {
    setStatus([`status: step failed`, String(err?.message ?? err)]);
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

function setStatus(lines) {
  if (!statusBox) return;
  statusBox.textContent = lines.join("\n");
}

let wasmVm = null;

async function getOrCreateWasmVm(source = "") {
  if (wasmVm) return wasmVm;
  wasmVm = await createWasmVm({
    cacheBust: Date.now(),
    setStatus,
    ddnSourceText: source,
    missingExportMessage: "DdnWasmVm export ?꾨씫: scripts/build_wasm_tool.ps1 --features wasm ?꾩슂",
    formatReadyStatus: ({ buildInfo, cacheBust }) =>
      [
        buildInfo
          ? `status: wasm ready (v=${cacheBust}) | ${buildInfo}`
          : `status: wasm ready (v=${cacheBust})`,
      ],
    formatFallbackStatus: () => ["status: wasm ready (fallback) ???섑뵆 濡쒕뱶 ??'濡쒖쭅 ?곸슜'???꾨Ⅴ?몄슂."],
  });
  return wasmVm;
}

async function ensureWasm(source) {
  const handle = await getOrCreateWasmVm(source ?? "");
  return handle.ensureClient(source ?? "");
}

function gatherInput() {
  return gatherInputFromDom($, KEY_BITS);
}

function renderState(state) {
  if (!state) return;
  lastState = state;
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
  if (rawBox) {
    rawBox.value = JSON.stringify(state, null, 2);
  }
  latestPatch = Array.isArray(state.patch) ? state.patch : [];
  renderPatchList(latestPatch);
  renderParamList(state);
  renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph);
}

function renderPatchList(patch) {
  if (!patchListBox) return;
  if (!Array.isArray(patch) || !patch.length) {
    patchListBox.textContent = "-";
    return;
  }
  const allowResource = patchFilterResource?.checked ?? true;
  const allowComponent = patchFilterComponent?.checked ?? true;
  const allowSignal = patchFilterSignal?.checked ?? true;
  const recent = patch.slice(-20).map((op) => {
    if (!op || !op.op) return "-";
    const group = classifyPatchOp(op);
    if ((group === "resource" && !allowResource) || (group === "component" && !allowComponent)) {
      return null;
    }
    if (group === "signal" && !allowSignal) return null;
    if (op.op === "set_resource_json") {
      return `${op.op} tag=${op.tag}`;
    }
    if (op.op === "set_resource_fixed64") {
      return `${op.op} tag=${op.tag} value=${op.value}`;
    }
    if (op.op === "set_resource_value") {
      return `${op.op} tag=${op.tag} value=${op.value}`;
    }
    if (op.op === "set_component_json") {
      return `${op.op} entity=${op.entity} tag=${op.tag}`;
    }
    if (op.op === "remove_component") {
      return `${op.op} entity=${op.entity} tag=${op.tag}`;
    }
    return JSON.stringify(op);
  });
  const filtered = recent.filter((line) => line != null);
  patchListBox.textContent = filtered.length ? filtered.join("\n") : "-";
}

function classifyPatchOp(op) {
  if (!op?.op) return "other";
  if (op.op.startsWith("set_resource") || op.op === "div_assign_resource_fixed64") {
    return "resource";
  }
  if (op.op.startsWith("set_component") || op.op === "remove_component") {
    return "component";
  }
  if (op.op === "emit_signal" || op.op === "guard_violation") {
    return "signal";
  }
  return "other";
}

function updateViewsFromState(state) {
  if (!graphCanvas) return;
  const preferPatch = patchPreferToggle?.checked ?? false;
  const views = extractStructuredViewsFromState(state, { preferPatch });
  latestGraph = views.graph ?? null;
  latestSpace2d = views.space2d ?? null;
  latestGraphSource = views.graphSource ?? "-";
}

function renderParamList(state) {
  if (!paramListBox) return;
  const showFixed = paramShowFixed64?.checked ?? true;
  const showValue = paramShowValue?.checked ?? true;
  const lines = collectStateResourceLines(state, { showFixed, showValue });
  paramListBox.textContent = lines.length ? lines.join("\n") : "-";
}

function setParamApplyStatus(message) {
  if (!paramApplyStatus) return;
  paramApplyStatus.textContent = String(message ?? "param: -");
}

function saveParamInputs() {
  saveWasmParamDraftState({
    storageKey: PARAM_STORAGE_KEY,
    draft: readWasmParamDraftFromControls({
      keyInput: paramKeyInput,
      modeSelect: paramModeSelect,
      valueInput: paramInput,
    }),
  });
}

function restoreParamInputs() {
  const draft = loadWasmParamDraftState({
    storageKey: PARAM_STORAGE_KEY,
    fallback: {
      key: "",
      mode: "scalar",
      value: "",
    },
  });
  applyWasmParamDraftToControls({
    keyInput: paramKeyInput,
    modeSelect: paramModeSelect,
    valueInput: paramInput,
    draft,
  });
}

function renderGraphOrSpace2d(graph, space2d, lensGraph = null) {
  renderGraphOrSpace2dCanvas({
    canvas: graphCanvas,
    graph,
    space2d,
    lensGraph,
    viewState: space2dView,
    showGraphGrid: gridToggle?.checked ?? true,
    showGraphAxis: axisToggle?.checked ?? true,
    showSpaceGrid: false,
    showSpaceAxis: false,
    graphPreference: "prefer_graph",
  });
}

async function applyLogic() {
  const source = $("ddn-source").value ?? "";
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
}

async function stepOnce() {
  const source = $("ddn-source").value ?? "";
  await stepWasmAndDispatchState({
    sourceText: source,
    ensureWasm,
    gatherInput,
    patchMode: false,
    onFull: renderState,
  });
}

async function applyParamFromControls() {
  const source = $("ddn-source").value ?? "";
  const client = await ensureWasm(source);
  const applied = applyWasmParamFromUi({
    client,
    key: paramKeyInput?.value ?? "",
    rawValue: paramInput?.value ?? "",
    mode: paramModeSelect?.value ?? "scalar",
    errorPrefix: "wasm_smoke param",
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
  loopController.start();
}

function stopLoop() {
  loopController.stop();
}

  async function loadSample() {
    const target = "../samples/01_line_graph.ddn";
    const cacheBust = `v=${Date.now()}`;
    const res = await fetch(`${target}?${cacheBust}`, { cache: "no-store" });
  if (!res.ok) {
    setStatus([`status: sample load failed`, `${res.status} ${res.statusText}`, `path: ${target}`]);
    return;
  }
  const text = await res.text();
  $("ddn-source").value = text;
  setStatus([`status: sample loaded (${target})`]);
}

if (window.location.protocol === "file:") {
  setStatus([
    "status: file:// ?꾨줈?좎퐳 媛먯?",
    "WASM/ES module? HTTP ?쒕쾭媛 ?꾩슂?⑸땲??",
    "python -m http.server 8080  ?먮뒗  npx serve .",
    "?댄썑 http://localhost:8080/wasm_smoke.html 濡??묒냽?섏꽭??",
  ]);
}

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
restoreParamInputs();

if (toggleRaw) {
  toggleRaw.addEventListener("change", () => {
    if (!rawBox) return;
    rawBox.classList.toggle("hidden", !toggleRaw.checked);
  });
}
if (patchPreferToggle) {
  patchPreferToggle.addEventListener("change", () => {
    if (lastState) renderState(lastState);
  });
}
if (paramShowFixed64) {
  paramShowFixed64.addEventListener("change", () => {
    if (lastState) renderParamList(lastState);
  });
}
if (paramShowValue) {
  paramShowValue.addEventListener("change", () => {
    if (lastState) renderParamList(lastState);
  });
}
if (paramApplyBtn) {
  paramApplyBtn.addEventListener("click", () => {
    saveParamInputs();
    applyParamFromControls().catch((err) => setStatus([`param apply ?ㅻ쪟`, String(err?.message ?? err)]));
  });
}
if (paramInput) {
  paramInput.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    saveParamInputs();
    applyParamFromControls().catch((err) => setStatus([`param apply ?ㅻ쪟`, String(err?.message ?? err)]));
  });
}
if (paramKeyInput) paramKeyInput.addEventListener("input", saveParamInputs);
if (paramModeSelect) paramModeSelect.addEventListener("change", saveParamInputs);
if (paramInput) paramInput.addEventListener("input", saveParamInputs);
if (space2dAutoFitToggle) {
  space2dAutoFitToggle.addEventListener("change", () => {
    setSpace2dAutoFit(space2dAutoFitToggle.checked);
    renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph);
  });
}
if (space2dResetViewBtn) {
  space2dResetViewBtn.addEventListener("click", () => {
    resetSpace2dView({ forceAutoFit: true });
    renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph);
  });
}
if (lensEnableToggle) {
  lensEnableToggle.addEventListener("change", () => {
    syncLensConfigFromDom();
    applyLensSelectionToRender();
  });
}
if (lensXSelect) {
  lensXSelect.addEventListener("change", () => {
    syncLensConfigFromDom();
    applyLensSelectionToRender();
  });
}
if (lensYSelect) {
  lensYSelect.addEventListener("change", () => {
    syncLensConfigFromDom();
    applyLensSelectionToRender();
  });
}
if (lensY2Select) {
  lensY2Select.addEventListener("change", () => {
    syncLensConfigFromDom();
    applyLensSelectionToRender();
  });
}
if (lensPresetSelect) {
  lensPresetSelect.addEventListener("change", () => {
    applyLensPresetSelection(lensPresetSelect.value);
  });
}
if (lensPresetSaveBtn) {
  lensPresetSaveBtn.addEventListener("click", () => {
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
    setStatus([`status: ?뚯쫰 ?꾨━?????(${result.presetId})`]);
  });
}
if (lensPresetDeleteBtn) {
  lensPresetDeleteBtn.addEventListener("click", () => {
    const result = deleteLensPresetFromState({ lensState });
    if (!result.ok) return;
    refreshLensPresetSelect();
    if (lensPresetNameInput) lensPresetNameInput.value = "";
    saveLensPresets();
    setStatus([`status: ?뚯쫰 ?꾨━????젣 (${result.presetId})`]);
  });
}

  $("btn-init").addEventListener("click", () => {
    const source = $("ddn-source").value ?? "";
    ensureWasm(source).catch((err) => {
      const debugInfo = wasmVm?.getDebugInfo?.() ?? {};
      const lastBuildInfo = debugInfo.buildInfo ?? "";
      const lastPreprocessed = debugInfo.preprocessed ?? "";
      dumpSourceDebug(source);
      const preview = buildSourcePreview(source, 30);
      const header = lastBuildInfo
        ? `${String(err)} | ${lastBuildInfo}`
        : String(err);
      const preprocessedPreview = buildSourcePreview(lastPreprocessed, 30);
      setStatus([
        header,
        "source preview:",
        ...preview,
        "preprocess preview:",
        ...preprocessedPreview,
      ]);
    });
  });
$("btn-apply").addEventListener("click", () => applyLogic().catch((err) => setStatus([String(err)])));
$("btn-step").addEventListener("click", () => stepOnce().catch((err) => setStatus([String(err)])));
$("btn-start").addEventListener("click", startLoop);
$("btn-stop").addEventListener("click", stopLoop);
$("btn-load-sample").addEventListener("click", () => loadSample().catch((err) => setStatus([String(err)])));

if (gridToggle) gridToggle.addEventListener("change", () => renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph));
if (axisToggle) axisToggle.addEventListener("change", () => renderGraphOrSpace2d(latestGraph, latestSpace2d, latestLensGraph));
if (patchFilterResource) patchFilterResource.addEventListener("change", () => renderPatchList(latestPatch));
if (patchFilterComponent) patchFilterComponent.addEventListener("change", () => renderPatchList(latestPatch));
if (patchFilterSignal) patchFilterSignal.addEventListener("change", () => renderPatchList(latestPatch));
