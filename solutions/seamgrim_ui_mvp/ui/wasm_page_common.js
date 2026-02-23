export const KEY_BITS = Object.freeze({
  up: 1,
  left: 2,
  down: 4,
  right: 8,
});

export function createEmptyObservationState({ includeValues = true } = {}) {
  const base = {
    channels: [],
    row: [],
  };
  if (includeValues) {
    base.values = {};
  }
  return base;
}

export function createObservationLensState({
  enabled = false,
  xKey = "__tick__",
  yKey = "",
  y2Key = "",
  presetId = "custom",
  maxPoints = 400,
  channelSig = "",
  lastFrameToken = "",
  includeRuns = false,
} = {}) {
  const defaultPreset = {
    enabled: Boolean(enabled),
    xKey: String(xKey),
    yKey: String(yKey),
    y2Key: String(y2Key),
  };
  const state = {
    enabled: Boolean(enabled),
    xKey: String(xKey),
    yKey: String(yKey),
    y2Key: String(y2Key),
    presetId: String(presetId),
    presets: {
      default: { ...defaultPreset },
    },
    maxPoints: Number.isFinite(maxPoints) ? Math.max(1, Math.trunc(maxPoints)) : 400,
    timeline: [],
    channelSig: String(channelSig),
    lastFrameToken,
  };
  if (includeRuns) {
    state.runs = [];
  }
  return state;
}

export function createEmptyStructuredViewRawSlots() {
  return {
    graph: null,
    space2d: null,
    table: null,
    text: null,
    structure: null,
  };
}

export function initializeObservationLensUi({
  lensState,
  observation,
  lensEnableToggle = null,
  lensPresetSelect = null,
  lensPresetNameInput = null,
  updateLensSelectors,
  syncLensConfigFromDom,
  refreshLensPresetSelect,
} = {}) {
  if (lensEnableToggle) {
    lensEnableToggle.checked = Boolean(lensState?.enabled);
  }
  if (typeof updateLensSelectors === "function") {
    updateLensSelectors(observation);
  }
  if (typeof syncLensConfigFromDom === "function") {
    syncLensConfigFromDom();
  }
  if (typeof refreshLensPresetSelect === "function") {
    refreshLensPresetSelect();
  }
  if (lensPresetSelect) {
    lensPresetSelect.value = String(lensState?.presetId ?? "custom");
  }
  if (lensPresetNameInput) {
    if (String(lensState?.presetId ?? "custom") !== "custom") {
      lensPresetNameInput.value = String(lensState?.presetId ?? "");
    } else {
      lensPresetNameInput.value = "";
    }
  }
}

export function syncWasmSettingsControlsFromState({
  wasmState,
  enabledToggle = null,
  langModeSelect = null,
  sampleSelect = null,
  inputEnabledToggle = null,
  fpsInput = null,
  dtMaxInput = null,
  fixedDtEnabledToggle = null,
  fixedDtValueInput = null,
  patchModeToggle = null,
  keyPresetSelect = null,
  keyUpInput = null,
  keyLeftInput = null,
  keyDownInput = null,
  keyRightInput = null,
  schemaMapInput = null,
  fixed64MapInput = null,
  wasmParamKeyInput = null,
  wasmParamModeSelect = null,
  wasmParamValueInput = null,
  lensEnableToggle = null,
  schemaPresetSelect = null,
  schemaPresetNameInput = null,
  lensPresetSelect = null,
  lensPresetNameInput = null,
} = {}) {
  const ws = wasmState && typeof wasmState === "object" ? wasmState : {};
  if (enabledToggle) enabledToggle.checked = Boolean(ws.enabled);
  if (langModeSelect) langModeSelect.value = String(ws.langMode ?? "compat");
  if (sampleSelect) sampleSelect.value = String(ws.sampleId ?? "");
  if (inputEnabledToggle) inputEnabledToggle.checked = Boolean(ws.inputEnabled);
  if (fpsInput && ws.fpsLimit !== undefined && ws.fpsLimit !== null) fpsInput.value = ws.fpsLimit;
  if (dtMaxInput && ws.dtMax !== undefined && ws.dtMax !== null) dtMaxInput.value = ws.dtMax;
  if (fixedDtEnabledToggle) fixedDtEnabledToggle.checked = Boolean(ws.fixedDtEnabled);
  if (fixedDtValueInput && ws.fixedDtValue !== undefined && ws.fixedDtValue !== null) {
    fixedDtValueInput.value = ws.fixedDtValue;
  }
  if (patchModeToggle) patchModeToggle.checked = Boolean(ws.patchMode);
  if (keyPresetSelect) keyPresetSelect.value = String(ws.keyPresetId ?? "custom");
  if (keyUpInput) keyUpInput.value = String(ws?.keyMapRaw?.up ?? "");
  if (keyLeftInput) keyLeftInput.value = String(ws?.keyMapRaw?.left ?? "");
  if (keyDownInput) keyDownInput.value = String(ws?.keyMapRaw?.down ?? "");
  if (keyRightInput) keyRightInput.value = String(ws?.keyMapRaw?.right ?? "");
  if (schemaMapInput) schemaMapInput.value = String(ws.schemaMapRaw ?? "");
  if (fixed64MapInput) fixed64MapInput.value = String(ws.fixed64MapRaw ?? "");
  if (wasmParamKeyInput) wasmParamKeyInput.value = String(ws.paramKey ?? "");
  if (wasmParamModeSelect) wasmParamModeSelect.value = String(ws.paramMode ?? "scalar");
  if (wasmParamValueInput) wasmParamValueInput.value = String(ws.paramValue ?? "");
  if (lensEnableToggle) lensEnableToggle.checked = Boolean(ws?.lens?.enabled);
  if (schemaPresetSelect) schemaPresetSelect.value = String(ws.schemaPresetId ?? "custom");
  if (schemaPresetNameInput) {
    if (String(ws.schemaPresetId ?? "custom") !== "custom") {
      schemaPresetNameInput.value = String(ws.schemaPresetId ?? "");
    } else {
      schemaPresetNameInput.value = "";
    }
  }
  if (lensPresetSelect) lensPresetSelect.value = String(ws?.lens?.presetId ?? "custom");
  if (lensPresetNameInput) {
    if (String(ws?.lens?.presetId ?? "custom") !== "custom") {
      lensPresetNameInput.value = String(ws?.lens?.presetId ?? "");
    } else {
      lensPresetNameInput.value = "";
    }
  }
}

export function resetObservationLensTimeline(
  lensState,
  { lastFrameToken = "", clearRuns = false } = {},
) {
  if (!lensState || typeof lensState !== "object") return lensState;
  lensState.timeline = [];
  lensState.lastFrameToken = lastFrameToken;
  if (clearRuns) {
    lensState.runs = [];
  }
  return lensState;
}

export function createRafStepLoop({
  getFps = () => 30,
  onStep = () => {},
  onError = null,
  requestFrame = (cb) => requestAnimationFrame(cb),
  cancelFrame = (id) => cancelAnimationFrame(id),
  getNow = () =>
    typeof performance !== "undefined" && typeof performance.now === "function"
      ? performance.now()
      : Date.now(),
} = {}) {
  let running = false;
  let handle = null;
  let lastFrameMs = null;

  const stopInternal = ({ cancelPending = true } = {}) => {
    running = false;
    if (cancelPending && handle !== null && handle !== undefined) {
      cancelFrame(handle);
    }
    handle = null;
  };

  const fail = (err) => {
    if (!running) return;
    stopInternal({ cancelPending: true });
    if (typeof onError === "function") {
      onError(err);
    }
  };

  const tick = (ts) => {
    if (!running) {
      handle = null;
      return;
    }
    const now = typeof ts === "number" ? ts : getNow();
    const fps = Math.max(1, Number(getFps?.() ?? 30) || 30);
    const interval = 1000 / fps;
    if (lastFrameMs !== null && now - lastFrameMs < interval) {
      handle = requestFrame(tick);
      return;
    }
    lastFrameMs = now;
    try {
      const maybePromise = onStep?.(now);
      if (maybePromise && typeof maybePromise.then === "function") {
        maybePromise.catch((err) => {
          fail(err);
        });
      }
    } catch (err) {
      fail(err);
      return;
    }
    if (!running) {
      handle = null;
      return;
    }
    handle = requestFrame(tick);
  };

  return {
    start() {
      if (running) return false;
      running = true;
      lastFrameMs = null;
      handle = requestFrame(tick);
      return true;
    },
    stop() {
      stopInternal({ cancelPending: true });
    },
    isRunning() {
      return running;
    },
    getHandle() {
      return handle;
    },
    resetClock() {
      lastFrameMs = null;
    },
  };
}

export function createManagedRafStepLoop({
  getFps = () => 30,
  onStep = () => {},
  onError = null,
  isActive = () => false,
  setActive = () => {},
  onStart = null,
  onStop = null,
  requestFrame = (cb) => requestAnimationFrame(cb),
  cancelFrame = (id) => cancelAnimationFrame(id),
  getNow = () =>
    typeof performance !== "undefined" && typeof performance.now === "function"
      ? performance.now()
      : Date.now(),
} = {}) {
  const notifyStop = () => {
    if (typeof onStop === "function") {
      onStop();
    }
  };
  const loop = createRafStepLoop({
    getFps,
    onStep,
    onError: (err) => {
      setActive(false);
      notifyStop();
      if (typeof onError === "function") {
        onError(err);
      }
    },
    requestFrame,
    cancelFrame,
    getNow,
  });
  return {
    start() {
      if (Boolean(isActive()) || loop.isRunning()) return false;
      setActive(true);
      if (typeof onStart === "function") {
        onStart();
      }
      loop.start();
      return true;
    },
    stop() {
      const wasActive = Boolean(isActive()) || loop.isRunning();
      setActive(false);
      loop.stop();
      notifyStop();
      return wasActive;
    },
    isActive() {
      return Boolean(isActive());
    },
    isRunning() {
      return loop.isRunning();
    },
    resetClock() {
      loop.resetClock();
    },
    getHandle() {
      return loop.getHandle();
    },
  };
}

export function normalizeLensPresetConfig(raw) {
  const obj = raw && typeof raw === "object" ? raw : {};
  return {
    enabled: Boolean(obj.enabled),
    xKey: String(obj.xKey ?? obj.x_key ?? "__tick__"),
    yKey: String(obj.yKey ?? obj.y_key ?? ""),
    y2Key: String(obj.y2Key ?? obj.y2_key ?? ""),
  };
}

export function loadLensPresetState({
  storageKey,
  defaultPreset,
  preferredPresetId = "",
  normalizePreset = normalizeLensPresetConfig,
  onError,
} = {}) {
  const normalizedDefault = normalizePreset(defaultPreset);
  const preferredId = String(preferredPresetId ?? "").trim();
  const fallbackPresetId = preferredId === "default" ? "default" : "custom";
  const fallback = {
    presets: { default: normalizedDefault },
    presetId: fallbackPresetId,
  };
  try {
    if (typeof localStorage === "undefined") {
      return fallback;
    }
    const raw = localStorage.getItem(String(storageKey ?? ""));
    if (!raw) {
      return fallback;
    }
    const payload = JSON.parse(raw);
    const presetsRaw = payload?.presets ?? {};
    const next = { default: normalizedDefault };
    Object.entries(presetsRaw).forEach(([id, preset]) => {
      const key = String(id ?? "").trim();
      if (!key || key === "custom") return;
      next[key] = normalizePreset(preset);
    });
    const activeId = preferredId || String(payload?.activeId ?? "custom");
    return {
      presets: next,
      presetId: activeId in next ? activeId : "custom",
    };
  } catch (err) {
    if (typeof onError === "function") {
      onError(err);
    }
    return fallback;
  }
}

export function saveLensPresetState({
  storageKey,
  presetId,
  presets,
  onError,
} = {}) {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(
      String(storageKey ?? ""),
      JSON.stringify({
        activeId: String(presetId ?? "custom"),
        presets: presets && typeof presets === "object" ? presets : {},
      }),
    );
  } catch (err) {
    if (typeof onError === "function") {
      onError(err);
    }
  }
}

export function refreshLensPresetSelectElement({
  selectEl,
  presets,
  presetId,
  customLabel = "Custom(미저장)",
} = {}) {
  const ids = Object.keys(presets && typeof presets === "object" ? presets : {}).filter(
    (id) => id !== "custom",
  );
  ids.sort();
  const nextPresetId =
    ids.includes(String(presetId ?? "")) || String(presetId ?? "") === "custom"
      ? String(presetId ?? "custom")
      : "custom";
  if (!selectEl) {
    return nextPresetId;
  }
  selectEl.innerHTML = "";
  const addOption = (id, label) => {
    const option = document.createElement("option");
    option.value = String(id);
    option.textContent = String(label);
    selectEl.appendChild(option);
  };
  addOption("custom", customLabel);
  ids.forEach((id) => addOption(id, id));
  selectEl.value = nextPresetId;
  return nextPresetId;
}

export function observationChannelSignature(channels) {
  if (!Array.isArray(channels)) return "";
  return channels.map((entry) => String(entry?.key ?? "")).join("|");
}

function setLensSelectOptions(select, options, selectedValue) {
  if (!select) return;
  const normalized = Array.isArray(options) ? options : [];
  const previous = selectedValue ?? select.value;
  if (typeof document !== "undefined" && typeof document.createElement === "function" && typeof select.appendChild === "function") {
    select.innerHTML = "";
    normalized.forEach((opt) => {
      const node = document.createElement("option");
      node.value = String(opt.value);
      node.textContent = String(opt.label);
      select.appendChild(node);
    });
  } else {
    select.options = normalized.map((opt) => ({
      value: String(opt.value),
      textContent: String(opt.label),
    }));
  }
  const fallback = normalized[0]?.value ?? "";
  const hasPrevious = normalized.some((opt) => String(opt.value) === String(previous));
  select.value = String(hasPrevious ? previous : fallback);
}

export function updateLensSelectorsFromObservation({
  observation,
  lensState,
  xSelect,
  ySelect,
  y2Select,
  onSynced,
} = {}) {
  if (!lensState || typeof lensState !== "object") return false;
  const channels = Array.isArray(observation?.channels) ? observation.channels : [];
  const sig = observationChannelSignature(channels);
  const hasXOptions = Boolean(
    xSelect &&
      (
        (xSelect.options && Number(xSelect.options.length) > 0) ||
        (xSelect.children && Number(xSelect.children.length) > 0)
      ),
  );
  if (sig === String(lensState.channelSig ?? "") && hasXOptions) {
    return false;
  }
  lensState.channelSig = sig;
  const keys = channels.map((entry) => String(entry?.key ?? "")).filter(Boolean);
  const xOptions = [
    { value: "__tick__", label: "[tick] tick_id" },
    { value: "__index__", label: "[index] sample index" },
    ...keys.map((key) => ({ value: key, label: key })),
  ];
  const yOptions = keys.map((key) => ({ value: key, label: key }));
  const y2Options = [{ value: "", label: "(none)" }, ...yOptions];
  setLensSelectOptions(xSelect, xOptions, lensState.xKey || "__tick__");
  setLensSelectOptions(ySelect, yOptions, lensState.yKey || yOptions[0]?.value || "");
  setLensSelectOptions(y2Select, y2Options, lensState.y2Key || "");
  if (typeof onSynced === "function") {
    onSynced();
  }
  return true;
}

export function toObservationFiniteNumber(value) {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (typeof value === "string") {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) return parsed;
  }
  return null;
}

export function formatObservationCellValue(value, { numericMode = "plain" } = {}) {
  if (value === undefined) return "-";
  if (value === null) return "null";
  if (typeof value === "number") {
    if (!Number.isFinite(value)) return String(value);
    if (numericMode === "compact") {
      if (Math.abs(value) >= 1000 || (Math.abs(value) > 0 && Math.abs(value) < 0.001)) {
        return value.toExponential(4);
      }
      return Number(value.toFixed(6)).toString();
    }
    return String(value);
  }
  if (typeof value === "string") return value;
  if (typeof value === "boolean") return value ? "true" : "false";
  try {
    return JSON.stringify(value);
  } catch (_) {
    return String(value);
  }
}

export function lensLabelFromState(
  lensState,
  { offLabel = "off", emptyLabel = "-", joiner = " + " } = {},
) {
  if (!lensState?.enabled) return offLabel;
  const primary = String(lensState?.yKey ?? "").trim() || emptyLabel;
  const secondary = String(lensState?.y2Key ?? "").trim();
  return secondary ? `${primary}${joiner}${secondary}` : primary;
}

export function buildObservationChannelLines({
  observation,
  maxRows = 80,
  numericMode = "plain",
} = {}) {
  const channels = Array.isArray(observation?.channels) ? observation.channels : [];
  const row = Array.isArray(observation?.row) ? observation.row : [];
  if (!channels.length) return [];
  const lines = channels.slice(0, maxRows).map((channel, index) => {
    const key = String(channel?.key ?? "");
    const dtype = String(channel?.dtype ?? "unknown");
    const role = String(channel?.role ?? "state");
    const value = formatObservationCellValue(row[index], { numericMode });
    return `${key} [${dtype}/${role}] = ${value}`;
  });
  if (channels.length > maxRows) {
    lines.push(`... +${channels.length - maxRows} more`);
  }
  return lines;
}

export function renderObservationChannelList({
  element,
  observation,
  maxRows = 80,
  numericMode = "plain",
  emptyText = "-",
  target = "text",
} = {}) {
  if (!element) return false;
  const lines = buildObservationChannelLines({
    observation,
    maxRows,
    numericMode,
  });
  const text = lines.length ? lines.join("\n") : emptyText;
  if (target === "value") {
    element.value = text;
  } else {
    element.textContent = text;
  }
  return lines.length > 0;
}

export function buildObservationStatusLines({
  stateJson,
  lensState,
  channelCount = null,
  patchCount = 0,
  tickTimeDigits = 2,
} = {}) {
  const channelCountFromState = Array.isArray(stateJson?.channels) ? stateJson.channels.length : null;
  const effectiveChannelCount = Number.isFinite(channelCount)
    ? channelCount
    : (
        Number.isFinite(channelCountFromState)
          ? channelCountFromState
          : (Array.isArray(stateJson?.row) ? stateJson.row.length : 0)
      );
  const tickTime = Number(stateJson?.tick_time_ms);
  const tickTimeText = Number.isFinite(tickTime) ? tickTime.toFixed(tickTimeDigits) : "-";
  return [
    `state_hash: ${stateJson?.state_hash ?? "-"}`,
    `tick_id: ${stateJson?.tick_id ?? "-"}`,
    `frame_id: ${stateJson?.frame_id ?? "-"}`,
    `tick_time_ms: ${tickTimeText}`,
    `channels: ${effectiveChannelCount}`,
    `lens: ${lensLabelFromState(lensState)}`,
    `patch: ${Number.isFinite(patchCount) ? patchCount : 0}`,
  ];
}

export function composeObservationRenderState({
  stateJson,
  observation,
  lensState,
  graphOptions = null,
  channelCount = null,
  patchCount = 0,
  tickTimeDigits = 2,
} = {}) {
  const lensSync = syncObservationLensFrame({
    lensState,
    observation,
    stateJson,
    tickId: stateJson?.tick_id,
    graphOptions: graphOptions && typeof graphOptions === "object" ? graphOptions : {},
  });
  const effectiveChannelCount = Number.isFinite(channelCount)
    ? channelCount
    : (Array.isArray(observation?.channels) ? observation.channels.length : 0);
  const statusLines = buildObservationStatusLines({
    stateJson,
    lensState,
    channelCount: effectiveChannelCount,
    patchCount,
    tickTimeDigits,
  });
  return {
    lensGraph: lensSync.graph,
    samplePushed: Boolean(lensSync.samplePushed),
    statusLines,
    channelCount: effectiveChannelCount,
  };
}

export function pushObservationLensSample({
  lensState,
  observation,
  tickId,
} = {}) {
  if (!lensState || typeof lensState !== "object") return false;
  const channels = Array.isArray(observation?.channels) ? observation.channels : [];
  const row = Array.isArray(observation?.row) ? observation.row : [];
  const timeline = Array.isArray(lensState.timeline) ? lensState.timeline : [];
  lensState.timeline = timeline;
  const sample = {
    __tick__: Number.isFinite(tickId) ? Number(tickId) : timeline.length,
    __index__: timeline.length,
  };
  channels.forEach((channel, index) => {
    const key = String(channel?.key ?? "").trim();
    if (!key) return;
    const numeric = toObservationFiniteNumber(row[index]);
    if (numeric !== null) {
      sample[key] = numeric;
    }
  });
  timeline.push(sample);
  const maxPoints = Number.isFinite(lensState.maxPoints) ? Math.max(1, Math.trunc(lensState.maxPoints)) : 400;
  if (timeline.length > maxPoints) {
    timeline.splice(0, timeline.length - maxPoints);
  }
  timeline.forEach((item, index) => {
    item.__index__ = index;
  });
  return true;
}

function pointsFromObservationLensSeries(samples, xKey, yKey) {
  const points = [];
  samples.forEach((sample, index) => {
    const y = toObservationFiniteNumber(sample?.[yKey]);
    if (y === null) return;
    const xRaw =
      xKey === "__index__" ? index :
      xKey === "__tick__" ? sample?.__tick__ :
      sample?.[xKey];
    const x = toObservationFiniteNumber(xRaw);
    if (x === null) return;
    points.push({ x, y });
  });
  return points;
}

export function buildObservationLensGraph({
  lensState,
  includeSample = false,
  metaUpdate = null,
  source = "observation-lens",
  colorPrimary = "#67e8f9",
  colorSecondary = "#f59e0b",
} = {}) {
  if (!lensState || typeof lensState !== "object") return null;
  if (!lensState.enabled) return null;
  if (!lensState.yKey) return null;
  const samples = Array.isArray(lensState.timeline) ? lensState.timeline : [];
  if (!samples.length) return null;
  const xKey = String(lensState.xKey || "__tick__");
  const yKey = String(lensState.yKey || "");
  const y2Key = String(lensState.y2Key || "");
  const series = [];
  const y1Points = pointsFromObservationLensSeries(samples, xKey, yKey);
  if (y1Points.length) {
    series.push({ id: yKey, label: yKey, points: y1Points, color: colorPrimary });
  }
  if (y2Key && y2Key !== yKey) {
    const y2Points = pointsFromObservationLensSeries(samples, xKey, y2Key);
    if (y2Points.length) {
      series.push({ id: y2Key, label: y2Key, points: y2Points, color: colorSecondary });
    }
  }
  if (!series.length) return null;
  const graph = {
    schema: "seamgrim.graph.v0",
    graph_kind: "xy",
    series,
    meta: {
      source,
      x_key: xKey,
      y_key: yKey,
      y2_key: y2Key || null,
    },
  };
  if (metaUpdate !== null && metaUpdate !== undefined) {
    graph.meta.update = metaUpdate;
  }
  if (includeSample) {
    graph.sample = {
      var: xKey,
      x_min: Number.NaN,
      x_max: Number.NaN,
      step: Number.NaN,
    };
  }
  return graph;
}

export function buildObservationFrameToken(stateJson) {
  return `${stateJson?.tick_id ?? "-"}:${stateJson?.frame_id ?? "-"}:${stateJson?.state_hash ?? "-"}`;
}

export function syncObservationLensFrame({
  lensState,
  observation,
  stateJson = null,
  tickId,
  graphOptions = null,
} = {}) {
  if (!lensState || typeof lensState !== "object") {
    return {
      graph: null,
      frameToken: null,
      samplePushed: false,
    };
  }
  const frameToken = buildObservationFrameToken(stateJson);
  const prevToken = String(lensState.lastFrameToken ?? "");
  let samplePushed = false;
  if (prevToken !== frameToken) {
    pushObservationLensSample({
      lensState,
      observation,
      tickId: tickId ?? stateJson?.tick_id,
    });
    lensState.lastFrameToken = frameToken;
    samplePushed = true;
  }
  const options = graphOptions && typeof graphOptions === "object" ? graphOptions : {};
  const graph = buildObservationLensGraph({
    lensState,
    ...options,
  });
  return {
    graph,
    frameToken,
    samplePushed,
  };
}

export function normalizeRenderPoints(points) {
  if (!Array.isArray(points)) return [];
  const out = [];
  points.forEach((pt) => {
    if (Array.isArray(pt) && pt.length >= 2) {
      out.push({ x: Number(pt[0]), y: Number(pt[1]) });
      return;
    }
    if (pt && Number.isFinite(pt.x) && Number.isFinite(pt.y)) {
      out.push({ x: Number(pt.x), y: Number(pt.y) });
    }
  });
  return out.filter((pt) => Number.isFinite(pt.x) && Number.isFinite(pt.y));
}

export function isGraphEmptyForRender(graph) {
  if (!graph) return true;
  const rawSeries = Array.isArray(graph.series) ? graph.series : [];
  if (!rawSeries.length) return true;
  const series = rawSeries
    .map((item) => ({ ...item, points: normalizeRenderPoints(item?.points ?? []) }))
    .filter((item) => item.points.length > 0);
  return series.length === 0;
}

function setupCanvasBase(canvas, bgColor = "#0b1020") {
  if (!canvas || typeof canvas.getContext !== "function") return null;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  const w = Number(canvas.width) || 0;
  const h = Number(canvas.height) || 0;
  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = bgColor;
  ctx.fillRect(0, 0, w, h);
  return { ctx, w, h };
}

function drawCanvasPlaceholder({
  ctx,
  text,
  x = 12,
  y = 20,
  color = "#94a3b8",
  font = "12px 'IBM Plex Mono', ui-monospace",
} = {}) {
  if (!ctx) return;
  ctx.fillStyle = color;
  ctx.font = font;
  ctx.fillText(String(text ?? "-"), x, y);
}

export function renderGraphCanvas2d({
  canvas,
  graph,
  showGrid = true,
  showAxis = true,
  pad = 28,
  emptyText = "graph: -",
  noPointsText = "graph: (no points)",
  palette = null,
} = {}) {
  const base = setupCanvasBase(canvas);
  if (!base) return false;
  const { ctx, w, h } = base;
  const rawSeries = Array.isArray(graph?.series) ? graph.series : [];
  if (!rawSeries.length) {
    drawCanvasPlaceholder({ ctx, text: emptyText });
    return false;
  }
  const series = rawSeries
    .map((item) => ({ ...item, points: normalizeRenderPoints(item?.points ?? []) }))
    .filter((item) => item.points.length > 0);
  if (!series.length) {
    drawCanvasPlaceholder({ ctx, text: noPointsText });
    return false;
  }
  const allPoints = series.flatMap((item) => item.points);
  let xMin = graph?.axis?.x_min;
  let xMax = graph?.axis?.x_max;
  let yMin = graph?.axis?.y_min;
  let yMax = graph?.axis?.y_max;
  if (![xMin, xMax, yMin, yMax].every(Number.isFinite)) {
    xMin = Math.min(...allPoints.map((p) => p.x));
    xMax = Math.max(...allPoints.map((p) => p.x));
    yMin = Math.min(...allPoints.map((p) => p.y));
    yMax = Math.max(...allPoints.map((p) => p.y));
  }
  if (xMax === xMin) xMax = xMin + 1;
  if (yMax === yMin) yMax = yMin + 1;
  const scaleX = (w - pad * 2) / (xMax - xMin);
  const scaleY = (h - pad * 2) / (yMax - yMin);

  if (showGrid) {
    ctx.strokeStyle = "rgba(148,163,184,0.15)";
    ctx.lineWidth = 1;
    for (let i = 1; i <= 4; i += 1) {
      const gx = pad + ((w - pad * 2) * i) / 5;
      const gy = pad + ((h - pad * 2) * i) / 5;
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
    ctx.strokeStyle = "#1f2a44";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(pad, pad);
    ctx.lineTo(pad, h - pad);
    ctx.lineTo(w - pad, h - pad);
    ctx.stroke();
    if (xMin < 0 && xMax > 0) {
      const x0 = pad + (0 - xMin) * scaleX;
      ctx.beginPath();
      ctx.moveTo(x0, pad);
      ctx.lineTo(x0, h - pad);
      ctx.stroke();
    }
    if (yMin < 0 && yMax > 0) {
      const y0 = h - pad - (0 - yMin) * scaleY;
      ctx.beginPath();
      ctx.moveTo(pad, y0);
      ctx.lineTo(w - pad, y0);
      ctx.stroke();
    }
  }

  const activePalette = Array.isArray(palette) && palette.length
    ? palette
    : ["#67e8f9", "#22c55e", "#38bdf8", "#a855f7", "#f59e0b", "#ef4444"];
  series.forEach((item, idx) => {
    const color = item.color ?? activePalette[idx % activePalette.length];
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.beginPath();
    item.points.forEach((pt, index) => {
      const x = pad + (pt.x - xMin) * scaleX;
      const y = h - pad - (pt.y - yMin) * scaleY;
      if (index === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.stroke();
  });

  ctx.fillStyle = "#e2e8f0";
  ctx.font = "11px 'IBM Plex Mono', ui-monospace";
  ctx.fillText(`x:[${xMin.toFixed(2)}, ${xMax.toFixed(2)}]`, pad, 16);
  ctx.fillText(`y:[${yMin.toFixed(2)}, ${yMax.toFixed(2)}]`, pad, 30);
  return true;
}

function normalizeSpace2dPrimitiveSource(source) {
  const raw = String(source ?? "auto").trim().toLowerCase();
  if (raw === "drawlist") return "drawlist";
  if (raw === "shapes") return "shapes";
  if (raw === "both") return "both";
  if (raw === "none") return "none";
  return "auto";
}

function normalizeSpace2dKind(kind) {
  const raw = String(kind ?? "").trim().toLowerCase();
  if (raw === "poly") return "polygon";
  return raw;
}

function normalizeSpace2dPointList(points) {
  if (!Array.isArray(points)) return [];
  const out = [];
  points.forEach((item) => {
    if (Array.isArray(item) && item.length >= 2) {
      const x = Number(item[0]);
      const y = Number(item[1]);
      if (Number.isFinite(x) && Number.isFinite(y)) out.push({ x, y });
      return;
    }
    if (!item || typeof item !== "object") return;
    const x = Number(item.x ?? item[0]);
    const y = Number(item.y ?? item[1]);
    if (Number.isFinite(x) && Number.isFinite(y)) out.push({ x, y });
  });
  return out;
}

function resolveSpace2dPoint(item) {
  if (!item || typeof item !== "object") return null;
  const x = Number(item.x ?? item.cx ?? item[0]);
  const y = Number(item.y ?? item.cy ?? item[1]);
  if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
  return { x, y };
}

function resolveSpace2dLine(item) {
  if (!item || typeof item !== "object") return null;
  const x1 = Number(item.x1);
  const y1 = Number(item.y1);
  const x2 = Number(item.x2);
  const y2 = Number(item.y2);
  if (![x1, y1, x2, y2].every(Number.isFinite)) return null;
  return { x1, y1, x2, y2 };
}

function resolveSpace2dRect(item) {
  if (!item || typeof item !== "object") return null;
  const x = Number(item.x ?? item.x0);
  const y = Number(item.y ?? item.y0);
  const w = Number(item.w ?? item.width);
  const h = Number(item.h ?? item.height);
  if ([x, y, w, h].every(Number.isFinite)) {
    return {
      x1: x,
      y1: y,
      x2: x + w,
      y2: y + h,
    };
  }
  const x1 = Number(item.x1);
  const y1 = Number(item.y1);
  const x2 = Number(item.x2);
  const y2 = Number(item.y2);
  if ([x1, y1, x2, y2].every(Number.isFinite)) {
    return { x1, y1, x2, y2 };
  }
  return null;
}

function resolveSpace2dCircle(item) {
  if (!item || typeof item !== "object") return null;
  const x = Number(item.x ?? item.cx);
  const y = Number(item.y ?? item.cy);
  const r = Number(item.r);
  if (![x, y, r].every(Number.isFinite)) return null;
  return { x, y, r };
}

function resolveSpace2dCurvePoints(item) {
  if (!item || typeof item !== "object") return [];
  return normalizeSpace2dPointList(item.points ?? item.coords ?? item.vertices ?? item.samples);
}

function resolveSpace2dFillPolygon(item) {
  if (!item || typeof item !== "object") return [];
  return normalizeSpace2dPointList(item.points ?? item.coords ?? item.vertices);
}

function collectSpace2dRangePoints(item) {
  if (!item || typeof item !== "object") return [];
  const kind = normalizeSpace2dKind(item.kind);
  if (kind === "line" || kind === "arrow") {
    const line = resolveSpace2dLine(item);
    if (!line) return [];
    return [
      { x: line.x1, y: line.y1 },
      { x: line.x2, y: line.y2 },
    ];
  }
  if (kind === "circle") {
    const circle = resolveSpace2dCircle(item);
    if (!circle) return [];
    return [
      { x: circle.x - circle.r, y: circle.y - circle.r },
      { x: circle.x + circle.r, y: circle.y + circle.r },
    ];
  }
  if (kind === "rect") {
    const rect = resolveSpace2dRect(item);
    if (!rect) return [];
    return [
      { x: rect.x1, y: rect.y1 },
      { x: rect.x2, y: rect.y2 },
    ];
  }
  if (kind === "point" || kind === "text") {
    const point = resolveSpace2dPoint(item);
    return point ? [point] : [];
  }
  if (kind === "curve" || kind === "polyline" || kind === "polygon") {
    return resolveSpace2dCurvePoints(item);
  }
  if (kind === "fill") {
    return resolveSpace2dFillPolygon(item);
  }
  return [];
}

function selectSpace2dPrimitiveItems({
  drawlist,
  shapes,
  hasShapesField,
  sourceMode,
}) {
  if (sourceMode === "drawlist") return drawlist;
  if (sourceMode === "shapes") return shapes;
  if (sourceMode === "both") return [...shapes, ...drawlist];
  if (sourceMode === "none") return [];
  return hasShapesField ? shapes : drawlist;
}

export function renderSpace2dCanvas2d({
  canvas,
  space2d,
  viewState = null,
  primitiveSource = "auto",
  showGrid = false,
  showAxis = false,
  pad = 28,
  emptyText = "graph/space2d: -",
  noPointsText = "space2d: (no points)",
} = {}) {
  const base = setupCanvasBase(canvas);
  if (!base) return false;
  const { ctx, w, h } = base;
  if (!space2d) {
    drawCanvasPlaceholder({ ctx, text: emptyText });
    return false;
  }
  const points = normalizeRenderPoints(space2d?.points ?? []);
  const drawlist = Array.isArray(space2d?.drawlist) ? space2d.drawlist : [];
  const hasShapesField = Array.isArray(space2d?.shapes);
  const shapes = hasShapesField ? space2d.shapes : [];
  const sourceMode = normalizeSpace2dPrimitiveSource(primitiveSource);
  const items = selectSpace2dPrimitiveItems({
    drawlist,
    shapes,
    hasShapesField,
    sourceMode,
  });
  const rangePoints = [...points];
  items.forEach((item) => {
    rangePoints.push(...collectSpace2dRangePoints(item));
  });
  if (!rangePoints.length) {
    drawCanvasPlaceholder({ ctx, text: noPointsText });
    return false;
  }

  const cam = space2d?.camera ?? null;
  let xMin = Number(cam?.x_min);
  let xMax = Number(cam?.x_max);
  let yMin = Number(cam?.y_min);
  let yMax = Number(cam?.y_max);
  if (![xMin, xMax, yMin, yMax].every(Number.isFinite)) {
    xMin = Math.min(...rangePoints.map((p) => p.x));
    xMax = Math.max(...rangePoints.map((p) => p.x));
    yMin = Math.min(...rangePoints.map((p) => p.y));
    yMax = Math.max(...rangePoints.map((p) => p.y));
  }
  const autoFit = Boolean(viewState?.autoFit ?? true);
  const customRange = viewState?.range ?? null;
  const customXMin = Number(customRange?.xMin ?? customRange?.x_min);
  const customXMax = Number(customRange?.xMax ?? customRange?.x_max);
  const customYMin = Number(customRange?.yMin ?? customRange?.y_min);
  const customYMax = Number(customRange?.yMax ?? customRange?.y_max);
  const hasCustomRange =
    [customXMin, customXMax, customYMin, customYMax].every(Number.isFinite) &&
    customXMax > customXMin &&
    customYMax > customYMin;
  if (!autoFit && hasCustomRange) {
    xMin = customXMin;
    xMax = customXMax;
    yMin = customYMin;
    yMax = customYMax;
  }
  if (xMax === xMin) xMax = xMin + 1;
  if (yMax === yMin) yMax = yMin + 1;

  const scaleX = (w - pad * 2) / (xMax - xMin);
  const scaleY = (h - pad * 2) / (yMax - yMin);
  const centerX = (xMin + xMax) / 2;
  const centerY = (yMin + yMax) / 2;
  const zoomRaw = Number(viewState?.zoom);
  const zoom = autoFit ? 1 : (Number.isFinite(zoomRaw) ? zoomRaw : 1);
  const panPxRaw = Number(viewState?.panPx);
  const panPyRaw = Number(viewState?.panPy);
  const panPx = autoFit ? 0 : (Number.isFinite(panPxRaw) ? panPxRaw : 0);
  const panPy = autoFit ? 0 : (Number.isFinite(panPyRaw) ? panPyRaw : 0);
  const mapX = (x) => (w / 2) + panPx + ((x - centerX) * scaleX * zoom);
  const mapY = (y) => (h / 2) + panPy - ((y - centerY) * scaleY * zoom);

  ctx.strokeStyle = "#1f2a44";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(pad, pad);
  ctx.lineTo(pad, h - pad);
  ctx.lineTo(w - pad, h - pad);
  ctx.stroke();

  if (showGrid) {
    const gridCount = 5;
    ctx.strokeStyle = "rgba(148,163,184,0.25)";
    ctx.lineWidth = 1;
    for (let i = 0; i < gridCount; i += 1) {
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
    ctx.strokeStyle = "rgba(226,232,240,0.35)";
    ctx.lineWidth = 1.5;
    if (xMin <= 0 && xMax >= 0) {
      const x0 = mapX(0);
      ctx.beginPath();
      ctx.moveTo(x0, pad);
      ctx.lineTo(x0, h - pad);
      ctx.stroke();
    }
    if (yMin <= 0 && yMax >= 0) {
      const y0 = mapY(0);
      ctx.beginPath();
      ctx.moveTo(pad, y0);
      ctx.lineTo(w - pad, y0);
      ctx.stroke();
    }
  }

  ctx.fillStyle = "#facc15";
  points.forEach((pt) => {
    const x = mapX(pt.x);
    const y = mapY(pt.y);
    ctx.beginPath();
    ctx.arc(x, y, 3, 0, Math.PI * 2);
    ctx.fill();
  });

  ctx.strokeStyle = "#a78bfa";
  items.forEach((shape) => {
    if (!shape || typeof shape !== "object") return;
    const kind = normalizeSpace2dKind(shape.kind);
    const style = String(shape.style ?? "solid").toLowerCase();
    if (style === "dashed") ctx.setLineDash([6, 4]);
    else if (style === "dotted") ctx.setLineDash([2, 4]);
    else ctx.setLineDash([]);
    const opacityRaw = Number(shape.opacity);
    const opacity = Number.isFinite(opacityRaw)
      ? Math.max(0, Math.min(1, opacityRaw))
      : 1;
    const widthRaw = Number(shape.width);
    const width = Number.isFinite(widthRaw) ? Math.max(1, widthRaw) : 1.5;
    const strokeColor = String(shape.stroke ?? shape.color ?? "#a78bfa");
    const drawColor = String(shape.color ?? strokeColor);
    ctx.globalAlpha = opacity;
    ctx.strokeStyle = strokeColor;
    ctx.fillStyle = drawColor;
    ctx.lineWidth = width;

    if (kind === "line") {
      const line = resolveSpace2dLine(shape);
      if (!line) return;
      const x1 = mapX(line.x1);
      const y1 = mapY(line.y1);
      const x2 = mapX(line.x2);
      const y2 = mapY(line.y2);
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();
    } else if (kind === "arrow") {
      const line = resolveSpace2dLine(shape);
      if (!line) return;
      const x1 = mapX(line.x1);
      const y1 = mapY(line.y1);
      const x2 = mapX(line.x2);
      const y2 = mapY(line.y2);
      if (![x1, y1, x2, y2].every(Number.isFinite)) return;
      const headSizeRaw = Number(shape.head_size);
      const headSize = Number.isFinite(headSizeRaw) ? Math.max(4, headSizeRaw) : 8;
      const angle = Math.atan2(y2 - y1, x2 - x1);

      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();

      ctx.beginPath();
      ctx.moveTo(x2, y2);
      ctx.lineTo(
        x2 - headSize * Math.cos(angle - Math.PI / 7),
        y2 - headSize * Math.sin(angle - Math.PI / 7),
      );
      ctx.lineTo(
        x2 - headSize * Math.cos(angle + Math.PI / 7),
        y2 - headSize * Math.sin(angle + Math.PI / 7),
      );
      ctx.closePath();
      ctx.fill();

      const labelText = String(shape.label ?? "").trim();
      if (labelText) {
        const labelOffset = shape.label_offset && typeof shape.label_offset === "object"
          ? shape.label_offset
          : {};
        const dx = Number(labelOffset.dx);
        const dy = Number(labelOffset.dy);
        const ox = Number.isFinite(dx) ? dx : 0;
        const oy = Number.isFinite(dy) ? dy : -8;
        ctx.font = "11px 'IBM Plex Mono', ui-monospace";
        ctx.fillStyle = drawColor;
        ctx.fillText(labelText, x2 + ox, y2 + oy);
      }
    } else if (kind === "circle") {
      const circle = resolveSpace2dCircle(shape);
      if (!circle) return;
      const x = mapX(circle.x);
      const y = mapY(circle.y);
      const rr = Math.max(1, Math.abs(circle.r * Math.min(scaleX, scaleY) * zoom));
      ctx.beginPath();
      ctx.arc(x, y, rr, 0, Math.PI * 2);
      ctx.stroke();
    } else if (kind === "rect") {
      const rect = resolveSpace2dRect(shape);
      if (!rect) return;
      const x1 = mapX(rect.x1);
      const y1 = mapY(rect.y1);
      const x2 = mapX(rect.x2);
      const y2 = mapY(rect.y2);
      const x0 = Math.min(x1, x2);
      const y0 = Math.min(y1, y2);
      const w0 = Math.abs(x2 - x1);
      const h0 = Math.abs(y2 - y1);
      const fill = shape.fill ?? shape.fill_color ?? null;
      const stroke = shape.color ?? shape.stroke ?? "#a78bfa";
      if (fill) {
        ctx.fillStyle = String(fill);
        ctx.fillRect(x0, y0, w0, h0);
      }
      ctx.strokeStyle = String(stroke);
      ctx.strokeRect(x0, y0, w0, h0);
    } else if (kind === "text") {
      const point = resolveSpace2dPoint(shape);
      if (!point) return;
      const text = shape.text ?? shape.label ?? "";
      if (!text) return;
      const x0 = mapX(point.x);
      const y0 = mapY(point.y);
      const size = Number(shape.size);
      const fontSize = Number.isFinite(size) ? Math.max(8, size) : 11;
      ctx.fillStyle = String(shape.color ?? "#e2e8f0");
      ctx.font = `${fontSize}px 'IBM Plex Mono', ui-monospace`;
      ctx.fillText(String(text), x0, y0);
    } else if (kind === "point") {
      const point = resolveSpace2dPoint(shape);
      if (!point) return;
      const px = mapX(point.x);
      const py = mapY(point.y);
      ctx.beginPath();
      ctx.arc(px, py, 2, 0, Math.PI * 2);
      ctx.fill();
    } else if (kind === "curve" || kind === "polyline" || kind === "polygon") {
      const list = resolveSpace2dCurvePoints(shape);
      if (list.length < 2) return;
      ctx.beginPath();
      list.forEach((pt, idx) => {
        const x = mapX(pt.x);
        const y = mapY(pt.y);
        if (idx === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      });
      const shouldClose = kind === "polygon";
      const fill = shape.fill ?? shape.fill_color ?? null;
      if (shouldClose) ctx.closePath();
      if (fill && shouldClose) {
        ctx.fillStyle = String(fill);
        ctx.fill();
      }
      ctx.stroke();
    } else if (kind === "fill") {
      const polygon = resolveSpace2dFillPolygon(shape);
      if (polygon.length < 3) return;
      ctx.beginPath();
      polygon.forEach((pt, idx) => {
        const x = mapX(pt.x);
        const y = mapY(pt.y);
        if (idx === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      });
      ctx.closePath();
      ctx.fillStyle = String(shape.fill ?? shape.fill_color ?? "rgba(167,139,250,0.25)");
      ctx.fill();
      if (shape.stroke || shape.color) {
        ctx.strokeStyle = String(shape.stroke ?? shape.color);
        ctx.stroke();
      }
    }
    ctx.setLineDash([]);
    ctx.globalAlpha = 1;
  });

  ctx.fillStyle = "#e2e8f0";
  ctx.font = "11px 'IBM Plex Mono', ui-monospace";
  ctx.fillText(
    `space2d zoom=${zoom.toFixed(2)} pan=(${Math.round(panPx)}, ${Math.round(panPy)})`,
    pad,
    16,
  );
  return true;
}

export function renderGraphOrSpace2dCanvas({
  canvas,
  graph,
  space2d,
  lensGraph = null,
  viewState = null,
  spacePrimitiveSource = "auto",
  showGraphGrid = true,
  showGraphAxis = true,
  showSpaceGrid = false,
  showSpaceAxis = false,
  graphPreference = "prefer_graph",
} = {}) {
  const selectedGraph = lensGraph ?? graph;
  if (graphPreference === "prefer_non_empty_graph") {
    if (selectedGraph && (!space2d || !isGraphEmptyForRender(selectedGraph))) {
      renderGraphCanvas2d({
        canvas,
        graph: selectedGraph,
        showGrid: showGraphGrid,
        showAxis: showGraphAxis,
      });
      return "graph";
    }
    renderSpace2dCanvas2d({
      canvas,
      space2d,
      viewState,
      primitiveSource: spacePrimitiveSource,
      showGrid: showSpaceGrid,
      showAxis: showSpaceAxis,
    });
    return "space2d";
  }
  if (selectedGraph) {
    renderGraphCanvas2d({
      canvas,
      graph: selectedGraph,
      showGrid: showGraphGrid,
      showAxis: showGraphAxis,
    });
    return "graph";
  }
  renderSpace2dCanvas2d({
    canvas,
    space2d,
    viewState,
    primitiveSource: spacePrimitiveSource,
    showGrid: showSpaceGrid,
    showAxis: showSpaceAxis,
  });
  return "space2d";
}

export function collectStateResourceLines(
  state,
  {
    showFixed = true,
    showValue = true,
    fixedPrefix = "fixed64",
    valuePrefix = "value",
  } = {},
) {
  const lines = [];
  const resources = state?.resources;
  if (showFixed && resources?.fixed64 && typeof resources.fixed64 === "object") {
    const entries = Object.entries(resources.fixed64).sort(([a], [b]) => a.localeCompare(b));
    entries.forEach(([tag, value]) => {
      lines.push(`${fixedPrefix} ${tag} = ${value}`);
    });
  }
  if (showValue && resources?.value && typeof resources.value === "object") {
    const entries = Object.entries(resources.value).sort(([a], [b]) => a.localeCompare(b));
    entries.forEach(([tag, value]) => {
      lines.push(`${valuePrefix} ${tag} = ${value}`);
    });
  }
  return lines;
}

export function dispatchWasmStateApply({
  stateJson,
  patchMode = false,
  onPatch = null,
  onFull = null,
} = {}) {
  const hasPatch = Array.isArray(stateJson?.patch) && stateJson.patch.length > 0;
  if (patchMode && hasPatch && typeof onPatch === "function") {
    onPatch(stateJson);
    return "patch";
  }
  if (typeof onFull === "function") {
    onFull(stateJson);
  }
  return "full";
}

export function dispatchWasmStateApplyWithSource({
  stateJson,
  sourceText = "",
  patchMode = false,
  onPatchWithSource = null,
  onFullWithSource = null,
  dispatch = dispatchWasmStateApply,
} = {}) {
  const source = String(sourceText ?? "");
  return dispatch({
    stateJson,
    patchMode: Boolean(patchMode),
    onPatch:
      typeof onPatchWithSource === "function"
        ? (nextState) => onPatchWithSource(nextState, source)
        : undefined,
    onFull:
      typeof onFullWithSource === "function"
        ? (nextState) => onFullWithSource(nextState, source)
        : undefined,
  });
}

export function parsePatchJsonObject(raw) {
  if (typeof raw !== "string") return null;
  try {
    return {
      raw,
      obj: JSON.parse(raw),
    };
  } catch (_) {
    return null;
  }
}

export function inferPatchSchemaFromJsonObject(obj) {
  const schema = obj?.schema ?? "";
  if (schema) return schema;
  if (obj?.matrix || (Array.isArray(obj?.columns) && Array.isArray(obj?.rows))) {
    return "seamgrim.table.v0";
  }
  if (Array.isArray(obj?.nodes) && Array.isArray(obj?.edges)) {
    return "seamgrim.structure.v0";
  }
  return "";
}

export function isPatchScalarValue(value) {
  return typeof value === "string" || typeof value === "number";
}

export function upsertPatchComponentStoreEntry({
  componentStore,
  op,
  raw,
  obj,
  keyBuilder = (entity, tag) => `${entity}:${tag}`,
} = {}) {
  if (!componentStore || typeof componentStore !== "object") {
    return { stored: false, reason: "invalid_store" };
  }
  if (op?.op !== "set_component_json") {
    return { stored: false, reason: "not_component_json" };
  }
  if (!Number.isFinite(op?.entity) || !Number.isFinite(op?.tag)) {
    return { stored: false, reason: "invalid_component_target" };
  }
  const key = keyBuilder(op.entity, op.tag);
  if (!key) {
    return { stored: false, reason: "invalid_component_key" };
  }
  const schema = inferPatchSchemaFromJsonObject(obj);
  componentStore[key] = {
    raw,
    schema,
  };
  return { stored: true, key, schema };
}

export function removePatchComponentStoreEntry({
  componentStore,
  op,
  keyBuilder = (entity, tag) => `${entity}:${tag}`,
} = {}) {
  if (!componentStore || typeof componentStore !== "object") {
    return { removed: false, reason: "invalid_store", entry: null };
  }
  if (op?.op !== "remove_component") {
    return { removed: false, reason: "not_remove_component", entry: null };
  }
  if (!Number.isFinite(op?.entity) || !Number.isFinite(op?.tag)) {
    return { removed: false, reason: "invalid_component_target", entry: null };
  }
  const key = keyBuilder(op.entity, op.tag);
  if (!key) {
    return { removed: false, reason: "invalid_component_key", entry: null };
  }
  const entry = componentStore[key] ?? null;
  delete componentStore[key];
  return { removed: true, key, entry };
}

export function upsertPatchScalarStoreEntry({
  store,
  tag,
  value,
  asString = false,
} = {}) {
  if (!store || typeof store !== "object") return { updated: false, reason: "invalid_store" };
  if (typeof tag !== "string") return { updated: false, reason: "invalid_tag" };
  if (!isPatchScalarValue(value)) return { updated: false, reason: "invalid_value" };
  store[tag] = asString ? String(value) : value;
  return { updated: true };
}

function mergePatchFlags(flags, next) {
  if (next === true) {
    flags.changed = true;
    return;
  }
  if (!next || typeof next !== "object") return;
  if (next.changed) flags.changed = true;
  if (next.fixed64Changed) flags.fixed64Changed = true;
  if (next.valueChanged) flags.valueChanged = true;
  if (next.requireFull) flags.requireFull = true;
}

export function processPatchOperations({
  patch,
  onJsonOp = null,
  onFixed64Op = null,
  onValueOp = null,
  onRemoveComponentOp = null,
} = {}) {
  const flags = {
    changed: false,
    fixed64Changed: false,
    valueChanged: false,
    requireFull: false,
  };
  if (!Array.isArray(patch) || !patch.length) return flags;
  for (const op of patch) {
    if (!op || !op.op) continue;
    if (op.op === "set_resource_json" || op.op === "set_component_json") {
      if (typeof onJsonOp === "function") {
        mergePatchFlags(flags, onJsonOp(op));
      }
      continue;
    }
    if (op.op === "set_resource_fixed64") {
      if (typeof onFixed64Op === "function") {
        mergePatchFlags(flags, onFixed64Op(op));
      }
      continue;
    }
    if (op.op === "set_resource_value") {
      if (typeof onValueOp === "function") {
        mergePatchFlags(flags, onValueOp(op));
      }
      continue;
    }
    if (op.op === "remove_component") {
      if (typeof onRemoveComponentOp === "function") {
        mergePatchFlags(flags, onRemoveComponentOp(op));
      }
      continue;
    }
  }
  return flags;
}

export function buildTagValueRowsFromStore(store) {
  if (!store || typeof store !== "object") return [];
  return Object.entries(store)
    .sort(([a], [b]) => String(a).localeCompare(String(b)))
    .map(([tag, value]) => ({ tag, value }));
}

export function buildTagValueTableFromStore({
  store,
  source = "value",
} = {}) {
  return {
    schema: "seamgrim.table.v0",
    columns: [
      { key: "tag", label: "tag" },
      { key: "value", label: "value" },
    ],
    rows: buildTagValueRowsFromStore(store),
    meta: { source: String(source ?? "value") },
  };
}

export function applyObservationRenderEffects({
  changed = false,
  forceRender = false,
  channelsChanged = false,
  onRender = null,
  onChannelsChanged = null,
} = {}) {
  if ((changed || forceRender) && typeof onRender === "function") {
    onRender();
  }
  if (channelsChanged && typeof onChannelsChanged === "function") {
    onChannelsChanged();
  }
}

export function markLensPresetCustomState({ lensState, selectEl, nameInputEl } = {}) {
  if (lensState && typeof lensState === "object") {
    lensState.presetId = "custom";
  }
  if (selectEl) {
    selectEl.value = "custom";
  }
  if (nameInputEl) {
    nameInputEl.value = "";
  }
}

export function applyLensPresetSelectionState({
  lensState,
  id,
  normalizePreset = normalizeLensPresetConfig,
} = {}) {
  const nextId = String(id ?? "custom");
  if (!lensState || typeof lensState !== "object") {
    return { ok: false, reason: "invalid_state" };
  }
  if (nextId === "custom") {
    lensState.presetId = "custom";
    return { ok: true, mode: "custom" };
  }
  const rawPreset = lensState.presets?.[nextId];
  if (!rawPreset) {
    return { ok: false, reason: "missing_preset" };
  }
  const preset = normalizePreset(rawPreset);
  lensState.presetId = nextId;
  lensState.enabled = preset.enabled;
  lensState.xKey = preset.xKey;
  lensState.yKey = preset.yKey;
  lensState.y2Key = preset.y2Key;
  return { ok: true, mode: "preset", presetId: nextId, preset };
}

export function saveLensPresetToState({
  lensState,
  presetName,
  currentPreset,
  normalizePreset = normalizeLensPresetConfig,
} = {}) {
  if (!lensState || typeof lensState !== "object") {
    return { ok: false, reason: "invalid_state" };
  }
  const rawName = String(presetName ?? "").trim();
  const fallbackId =
    lensState.presetId && lensState.presetId !== "custom" ? String(lensState.presetId) : "";
  const presetId = rawName || fallbackId;
  if (!presetId) {
    return { ok: false, reason: "missing_name" };
  }
  if (presetId === "custom") {
    return { ok: false, reason: "reserved_name" };
  }
  const normalized = normalizePreset(currentPreset);
  lensState.presets = lensState.presets && typeof lensState.presets === "object" ? lensState.presets : {};
  lensState.presets[presetId] = normalized;
  lensState.presetId = presetId;
  return { ok: true, presetId };
}

export function deleteLensPresetFromState({ lensState } = {}) {
  if (!lensState || typeof lensState !== "object") {
    return { ok: false, reason: "invalid_state" };
  }
  const id = String(lensState.presetId ?? "");
  if (!id || id === "default" || id === "custom") {
    return { ok: false, reason: "protected_preset" };
  }
  if (!lensState.presets || !Object.prototype.hasOwnProperty.call(lensState.presets, id)) {
    return { ok: false, reason: "missing_preset" };
  }
  delete lensState.presets[id];
  lensState.presetId = "custom";
  return { ok: true, presetId: id };
}

function clampNumber(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

export function bindSpace2dCanvasPanZoom({
  canvas,
  viewState,
  hasSpace2d,
  setAutoFit,
  onRender,
  zoomMin = 0.25,
  zoomMax = 12,
} = {}) {
  if (!canvas || !viewState || typeof viewState !== "object") return;

  canvas.addEventListener(
    "wheel",
    (event) => {
      if (!hasSpace2d?.()) return;
      event.preventDefault();
      if (viewState.autoFit) {
        setAutoFit?.(false);
      }
      const factor = event.deltaY < 0 ? 1.12 : 0.9;
      viewState.zoom = clampNumber((viewState.zoom ?? 1) * factor, zoomMin, zoomMax);
      onRender?.();
    },
    { passive: false },
  );

  canvas.addEventListener("pointerdown", (event) => {
    if (event.button !== 0 || !hasSpace2d?.()) return;
    viewState.dragging = true;
    viewState.lastX = event.clientX;
    viewState.lastY = event.clientY;
    canvas.setPointerCapture?.(event.pointerId);
  });

  canvas.addEventListener("pointermove", (event) => {
    if (!viewState.dragging) return;
    if (!hasSpace2d?.()) {
      viewState.dragging = false;
      return;
    }
    if (viewState.autoFit) {
      setAutoFit?.(false);
    }
    const dx = event.clientX - (viewState.lastX ?? event.clientX);
    const dy = event.clientY - (viewState.lastY ?? event.clientY);
    viewState.lastX = event.clientX;
    viewState.lastY = event.clientY;
    viewState.panPx = (viewState.panPx ?? 0) + dx;
    viewState.panPy = (viewState.panPy ?? 0) + dy;
    onRender?.();
  });

  const stopDrag = () => {
    viewState.dragging = false;
  };
  canvas.addEventListener("pointerup", stopDrag);
  canvas.addEventListener("pointercancel", stopDrag);
  canvas.addEventListener("pointerleave", stopDrag);
}

export function bindSpace2dCanvasWorldInteractions({
  canvas,
  viewState,
  hasContent,
  computeBaseRange,
  getRenderMeta,
  onManualViewEnsured,
  onViewChanged,
  zoomMin = 0.25,
  zoomMax = 12,
  boundDatasetKey = "space2dInteractionBound",
} = {}) {
  if (!canvas || !viewState || typeof viewState !== "object") return;
  if (boundDatasetKey) {
    if (canvas.dataset?.[boundDatasetKey] === "1") return;
    if (canvas.dataset) {
      canvas.dataset[boundDatasetKey] = "1";
    }
  }

  const activePointers = new Map();
  let dragPointerId = null;
  let pinchBase = null;

  const ensureManualView = () => {
    if (!viewState.auto) return;
    const baseRange =
      computeBaseRange?.() ??
      getRenderMeta?.()?.range ??
      { xMin: 0, xMax: 1, yMin: 0, yMax: 1 };
    viewState.auto = false;
    viewState.range = { ...baseRange };
    onManualViewEnsured?.();
  };

  const getPointerArray = () => Array.from(activePointers.values());
  const getPinchDistance = (a, b) => Math.hypot((a?.x ?? 0) - (b?.x ?? 0), (a?.y ?? 0) - (b?.y ?? 0));
  const getPinchMidpoint = (a, b) => ({
    x: ((a?.x ?? 0) + (b?.x ?? 0)) / 2,
    y: ((a?.y ?? 0) + (b?.y ?? 0)) / 2,
  });
  const getRenderScale = () => {
    const meta = getRenderMeta?.();
    if (!meta?.range) return null;
    const drawWidth = Math.max(1, Number(meta.width ?? 0) - Number(meta.pad ?? 0) * 2);
    const drawHeight = Math.max(1, Number(meta.height ?? 0) - Number(meta.pad ?? 0) * 2);
    return {
      unitPerPxX: (meta.range.xMax - meta.range.xMin) / drawWidth,
      unitPerPxY: (meta.range.yMax - meta.range.yMin) / drawHeight,
    };
  };
  const beginPinch = () => {
    if (activePointers.size < 2) {
      pinchBase = null;
      return;
    }
    const [p1, p2] = getPointerArray();
    const distance = getPinchDistance(p1, p2);
    const mid = getPinchMidpoint(p1, p2);
    pinchBase = {
      distance: Math.max(1e-6, distance),
      zoom: Number(viewState.zoom ?? 1),
      panX: Number(viewState.panX ?? 0),
      panY: Number(viewState.panY ?? 0),
      midX: mid.x,
      midY: mid.y,
    };
  };

  canvas.addEventListener(
    "wheel",
    (event) => {
      if (!hasContent?.()) return;
      event.preventDefault();
      ensureManualView();
      const factor = event.deltaY < 0 ? 1.12 : 0.9;
      const nextZoom = Number(viewState.zoom ?? 1) * factor;
      viewState.zoom = clampNumber(nextZoom, zoomMin, zoomMax);
      onViewChanged?.();
    },
    { passive: false },
  );

  canvas.addEventListener("pointerdown", (event) => {
    if (event.pointerType === "mouse" && event.button !== 0) return;
    if (!hasContent?.()) return;
    ensureManualView();
    activePointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    viewState.lastClientX = event.clientX;
    viewState.lastClientY = event.clientY;
    if (activePointers.size >= 2) {
      dragPointerId = null;
      viewState.dragging = false;
      beginPinch();
    } else {
      dragPointerId = event.pointerId;
      viewState.dragging = true;
      pinchBase = null;
    }
    canvas.setPointerCapture?.(event.pointerId);
  });

  canvas.addEventListener("pointermove", (event) => {
    if (activePointers.has(event.pointerId)) {
      activePointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    }
    const scale = getRenderScale();
    if (activePointers.size >= 2) {
      if (!pinchBase) beginPinch();
      if (!pinchBase || !scale) return;
      const [p1, p2] = getPointerArray();
      const distance = getPinchDistance(p1, p2);
      const mid = getPinchMidpoint(p1, p2);
      const ratio = Math.max(0.1, distance / Math.max(1e-6, pinchBase.distance));
      viewState.zoom = clampNumber(pinchBase.zoom * ratio, zoomMin, zoomMax);
      viewState.panX = pinchBase.panX - (mid.x - pinchBase.midX) * scale.unitPerPxX;
      viewState.panY = pinchBase.panY + (mid.y - pinchBase.midY) * scale.unitPerPxY;
      onViewChanged?.();
      return;
    }
    if (!viewState.dragging || dragPointerId !== event.pointerId || !scale) return;
    const dx = event.clientX - Number(viewState.lastClientX ?? event.clientX);
    const dy = event.clientY - Number(viewState.lastClientY ?? event.clientY);
    viewState.lastClientX = event.clientX;
    viewState.lastClientY = event.clientY;
    viewState.panX = Number(viewState.panX ?? 0) - dx * scale.unitPerPxX;
    viewState.panY = Number(viewState.panY ?? 0) + dy * scale.unitPerPxY;
    onViewChanged?.();
  });

  const releaseDrag = (event) => {
    activePointers.delete(event.pointerId);
    if (typeof canvas.releasePointerCapture === "function") {
      try {
        canvas.releasePointerCapture(event.pointerId);
      } catch (_) {
        // pointer capture not held
      }
    }
    if (dragPointerId === event.pointerId) {
      dragPointerId = null;
      viewState.dragging = false;
    }
    if (activePointers.size >= 2) {
      beginPinch();
      viewState.dragging = false;
      dragPointerId = null;
      return;
    }
    pinchBase = null;
    if (activePointers.size === 1) {
      const [remainingId, point] = activePointers.entries().next().value;
      dragPointerId = remainingId;
      viewState.dragging = true;
      viewState.lastClientX = point.x;
      viewState.lastClientY = point.y;
    } else {
      dragPointerId = null;
      viewState.dragging = false;
    }
  };
  canvas.addEventListener("pointerup", releaseDrag);
  canvas.addEventListener("pointercancel", releaseDrag);
  canvas.addEventListener("pointerleave", releaseDrag);
}

function normalizeLines(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
}

export function stripMetaHeader(text) {
  const lines = normalizeLines(text);
  let idx = 0;
  while (idx < lines.length) {
    const trimmed = lines[idx].replace(/^[ \t\uFEFF]+/, "");
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
  return lines.slice(idx).join("\n");
}

export function buildSourcePreview(source, maxLines = 30) {
  const lines = normalizeLines(source);
  const clipped = lines.slice(0, maxLines);
  const numbered = clipped.map((line, idx) => `${String(idx + 1).padStart(3, " ")}: ${line}`);
  if (lines.length > maxLines) {
    numbered.push(`... (${lines.length - maxLines} more lines)`);
  }
  return numbered;
}

export function dumpSourceDebug(source, label = "DDN source (debug)") {
  const lines = normalizeLines(source);
  const numbered = lines.map((line, idx) => `${String(idx + 1).padStart(3, " ")}: ${line}`);
  console.groupCollapsed(label);
  console.log(numbered.join("\n"));
  console.log(`lines=${lines.length}, chars=${String(source ?? "").length}`);
  console.groupEnd();
}

export function gatherInputFromDom(getNode, keyBits = KEY_BITS) {
  let keys = 0;
  if (getNode("key-up")?.checked) keys |= keyBits.up;
  if (getNode("key-left")?.checked) keys |= keyBits.left;
  if (getNode("key-down")?.checked) keys |= keyBits.down;
  if (getNode("key-right")?.checked) keys |= keyBits.right;

  const lastKey = getNode("input-last-key")?.value ?? "";
  const px = Number(getNode("input-px")?.value ?? 0);
  const py = Number(getNode("input-py")?.value ?? 0);
  const dt = Number(getNode("input-dt")?.value ?? 0);

  return {
    keys,
    lastKey,
    px: Number.isFinite(px) ? Math.trunc(px) : 0,
    py: Number.isFinite(py) ? Math.trunc(py) : 0,
    dt: Number.isFinite(dt) && dt >= 0 ? dt : 0,
  };
}

export function normalizeWasmStepInput(input = {}) {
  const keys = Number(input.keys ?? 0);
  const px = Number(input.px ?? 0);
  const py = Number(input.py ?? 0);
  const dt = Number(input.dt ?? 0);
  return {
    keys: Number.isFinite(keys) ? Math.trunc(keys) : 0,
    lastKey: String(input.lastKey ?? ""),
    px: Number.isFinite(px) ? Math.trunc(px) : 0,
    py: Number.isFinite(py) ? Math.trunc(py) : 0,
    dt: Number.isFinite(dt) && dt >= 0 ? dt : 0,
  };
}

function coerceWasmScalarParamValue(rawValue) {
  const text = String(rawValue ?? "");
  const trimmed = text.trim();
  if (!trimmed) {
    return { ok: false, error: "값이 비어 있습니다." };
  }
  try {
    const parsed = JSON.parse(trimmed);
    if (typeof parsed === "number") {
      if (!Number.isFinite(parsed)) {
        return { ok: false, error: "유한 숫자만 허용됩니다." };
      }
      return { ok: true, value: parsed, kind: "number" };
    }
    if (typeof parsed === "boolean") {
      return { ok: true, value: parsed, kind: "boolean" };
    }
    if (typeof parsed === "string") {
      return { ok: true, value: parsed, kind: "string" };
    }
  } catch (_) {
    // fall through
  }
  const lower = trimmed.toLowerCase();
  if (lower === "true" || lower === "false") {
    return { ok: true, value: lower === "true", kind: "boolean" };
  }
  const numeric = Number(trimmed);
  if (Number.isFinite(numeric)) {
    return { ok: true, value: numeric, kind: "number" };
  }
  return { ok: true, value: text, kind: "string" };
}

function coerceWasmFixed64RawI64(rawValue) {
  const trimmed = String(rawValue ?? "").trim();
  if (!trimmed) {
    return { ok: false, error: "raw_i64 값이 비어 있습니다." };
  }
  if (!/^-?\d+$/.test(trimmed)) {
    return { ok: false, error: "raw_i64는 정수 문자열이어야 합니다." };
  }
  const parsed = Number(trimmed);
  const asSafeNumber = Number.isFinite(parsed) && Number.isSafeInteger(parsed) ? parsed : null;
  return { ok: true, text: trimmed, asSafeNumber };
}

export function normalizeWasmParamMode(mode) {
  return String(mode ?? "scalar").trim().toLowerCase() === "fixed64_raw" ? "fixed64_raw" : "scalar";
}

export function normalizeWasmParamDraft(raw = {}) {
  const obj = raw && typeof raw === "object" ? raw : {};
  return {
    key: String(obj.key ?? ""),
    mode: normalizeWasmParamMode(obj.mode),
    value: String(obj.value ?? ""),
  };
}

export function readWasmParamDraftFromControls({
  keyInput = null,
  modeSelect = null,
  valueInput = null,
} = {}) {
  return normalizeWasmParamDraft({
    key: keyInput?.value,
    mode: modeSelect?.value,
    value: valueInput?.value,
  });
}

export function applyWasmParamDraftToControls({
  keyInput = null,
  modeSelect = null,
  valueInput = null,
  draft = null,
} = {}) {
  const normalized = normalizeWasmParamDraft(draft);
  if (keyInput) keyInput.value = normalized.key;
  if (modeSelect) modeSelect.value = normalized.mode;
  if (valueInput) valueInput.value = normalized.value;
  return normalized;
}

export function loadWasmParamDraftState({
  storageKey,
  fallback = null,
  onError,
} = {}) {
  const defaultDraft = normalizeWasmParamDraft(fallback);
  try {
    if (typeof localStorage === "undefined") return defaultDraft;
    const raw = localStorage.getItem(String(storageKey ?? ""));
    if (!raw) return defaultDraft;
    return normalizeWasmParamDraft(JSON.parse(raw));
  } catch (err) {
    if (typeof onError === "function") {
      onError(err);
    }
    return defaultDraft;
  }
}

export function saveWasmParamDraftState({
  storageKey,
  draft = null,
  onError,
} = {}) {
  const normalized = normalizeWasmParamDraft(draft);
  try {
    if (typeof localStorage === "undefined") return normalized;
    localStorage.setItem(String(storageKey ?? ""), JSON.stringify(normalized));
  } catch (err) {
    if (typeof onError === "function") {
      onError(err);
    }
  }
  return normalized;
}

export function applyWasmParamFromUi({
  client,
  key,
  rawValue,
  mode = "scalar",
  errorPrefix = "applyWasmParamFromUi",
} = {}) {
  if (!client || typeof client !== "object") {
    return { ok: false, error: `${errorPrefix}: client가 필요합니다.` };
  }
  const targetKey = String(key ?? "").trim();
  if (!targetKey) {
    return { ok: false, error: "param key가 비어 있습니다." };
  }
  const modeKey = String(mode ?? "scalar").trim().toLowerCase();
  try {
    if (modeKey === "fixed64_raw" || modeKey === "fixed64") {
      const parsed = coerceWasmFixed64RawI64(rawValue);
      if (!parsed.ok) return parsed;
      let result = null;
      if (typeof client.setParamFixed64StringParsed === "function") {
        result = client.setParamFixed64StringParsed(targetKey, parsed.text);
      } else if (typeof client.setParamFixed64Parsed === "function") {
        if (!Number.isSafeInteger(parsed.asSafeNumber)) {
          return {
            ok: false,
            error:
              "현재 wasm 빌드는 set_param_fixed64_str를 지원하지 않습니다. JS 안전 정수 범위 raw_i64만 허용됩니다.",
          };
        }
        result = client.setParamFixed64Parsed(targetKey, parsed.asSafeNumber);
      } else {
        return { ok: false, error: "set_param_fixed64 API를 지원하지 않는 wasm 빌드입니다." };
      }
      return {
        ok: true,
        mode: "fixed64_raw",
        key: targetKey,
        value: parsed.text,
        valueKind: "raw_i64",
        result,
      };
    }
    if (typeof client.setParamParsed !== "function") {
      return { ok: false, error: "set_param API를 지원하지 않는 wasm 빌드입니다." };
    }
    const parsed = coerceWasmScalarParamValue(rawValue);
    if (!parsed.ok) return parsed;
    const result = client.setParamParsed(targetKey, parsed.value);
    return {
      ok: true,
      mode: "scalar",
      key: targetKey,
      value: parsed.value,
      valueKind: parsed.kind,
      result,
    };
  } catch (err) {
    return { ok: false, error: `${errorPrefix}: ${String(err?.message ?? err)}` };
  }
}

export function stepWasmClientParsed({
  client,
  input = {},
  errorPrefix = "stepWasmClientParsed",
} = {}) {
  if (!client || typeof client !== "object") {
    throw new Error(`${errorPrefix}: client가 필요합니다.`);
  }
  const normalizedInput = normalizeWasmStepInput(input);
  const hasWithInput = typeof client.stepOneWithInputParsed === "function";
  const hasStepOne = typeof client.stepOneParsed === "function";
  if (!hasWithInput && !hasStepOne) {
    throw new Error(`${errorPrefix}: stepOneWithInputParsed/stepOneParsed API가 없습니다.`);
  }
  if (hasWithInput) {
    try {
      const state = client.stepOneWithInputParsed(
        normalizedInput.keys,
        normalizedInput.lastKey,
        normalizedInput.px,
        normalizedInput.py,
        normalizedInput.dt,
      );
      return { state, input: normalizedInput };
    } catch (withInputErr) {
      if (!hasStepOne) {
        throw new Error(
          `${errorPrefix}: stepOneWithInputParsed 실패 (${String(withInputErr?.message ?? withInputErr)})`,
        );
      }
      const state = client.stepOneParsed();
      return {
        state,
        input: normalizedInput,
        fallback: {
          mode: "step_one",
          reason: String(withInputErr?.message ?? withInputErr),
        },
      };
    }
  }
  const state = client.stepOneParsed();
  return { state, input: normalizedInput };
}

export function stepWasmClientAndDispatchState({
  client,
  input = {},
  sourceText = "",
  patchMode = false,
  onPatchWithSource = null,
  onFullWithSource = null,
  errorPrefix = "stepWasmClientAndDispatchState",
  dispatch = dispatchWasmStateApply,
} = {}) {
  const stepped = stepWasmClientParsed({
    client,
    input,
    errorPrefix,
  });
  dispatchWasmStateApplyWithSource({
    stateJson: stepped.state,
    sourceText,
    patchMode,
    onPatchWithSource,
    onFullWithSource,
    dispatch,
  });
  return stepped;
}

export function computeWasmStepDeltaSeconds({
  nowMs,
  lastTickMs = null,
  fixedDtEnabled = false,
  fixedDtValue = 0,
  dtMin = 0,
  dtMax = 0,
} = {}) {
  const now = Number.isFinite(nowMs) ? Number(nowMs) : 0;
  const last = Number.isFinite(lastTickMs) ? Number(lastTickMs) : now;
  let dtSec = Math.max(0, (now - last) / 1000);
  if (fixedDtEnabled) {
    const fixed = Number(fixedDtValue);
    if (Number.isFinite(fixed) && fixed >= 0) {
      dtSec = fixed;
    }
  }
  const min = Number.isFinite(dtMin) ? Number(dtMin) : 0;
  const max = Number.isFinite(dtMax) ? Number(dtMax) : 0;
  if (max > 0) dtSec = Math.min(dtSec, max);
  if (min > 0) dtSec = Math.max(dtSec, min);
  return { dtSec, nextTickMs: now };
}

export function stepWasmClientWithTimingAndDispatch({
  client,
  nowMs,
  lastTickMs = null,
  fixedDtEnabled = false,
  fixedDtValue = 0,
  dtMin = 0,
  dtMax = 0,
  inputEnabled = true,
  keys = 0,
  lastKey = "",
  px = 0,
  py = 0,
  clearLastKeyWhenFixedDt = true,
  sourceText = "",
  patchMode = false,
  onPatchWithSource = null,
  onFullWithSource = null,
  errorPrefix = "stepWasmClientWithTimingAndDispatch",
  dispatch = dispatchWasmStateApply,
} = {}) {
  const delta = computeWasmStepDeltaSeconds({
    nowMs,
    lastTickMs,
    fixedDtEnabled,
    fixedDtValue,
    dtMin,
    dtMax,
  });
  const enabled = Boolean(inputEnabled);
  const parsedKeys = Number(keys ?? 0);
  const parsedPx = Number(px ?? 0);
  const parsedPy = Number(py ?? 0);
  const normalizedLastKey = enabled ? String(lastKey ?? "") : "";
  const input = {
    keys: enabled && Number.isFinite(parsedKeys) ? parsedKeys : 0,
    lastKey: normalizedLastKey,
    px: enabled && Number.isFinite(parsedPx) ? Math.round(parsedPx) : 0,
    py: enabled && Number.isFinite(parsedPy) ? Math.round(parsedPy) : 0,
    dt: delta.dtSec,
  };
  const stepped = stepWasmClientAndDispatchState({
    client,
    input,
    sourceText,
    patchMode,
    onPatchWithSource,
    onFullWithSource,
    errorPrefix,
    dispatch,
  });
  const shouldClearLastKey = Boolean(clearLastKeyWhenFixedDt) && Boolean(fixedDtEnabled);
  return {
    stepped,
    input,
    dtSec: delta.dtSec,
    nextTickMs: delta.nextTickMs,
    nextLastKey: shouldClearLastKey ? "" : normalizedLastKey,
  };
}

export function resetObservationRuntimeCaches({
  lensState,
  lastFrameToken = "",
  clearRuns = false,
  observationFactory = () => createEmptyObservationState({ includeValues: true }),
  channelListElement = null,
  channelPlaceholder = "-",
  resetSpace2dView = null,
  resetSpace2dViewArgs = { forceAutoFit: true },
} = {}) {
  resetObservationLensTimeline(lensState, { lastFrameToken, clearRuns });
  if (typeof resetSpace2dView === "function") {
    resetSpace2dView(resetSpace2dViewArgs);
  }
  if (channelListElement) {
    if ("textContent" in channelListElement) {
      channelListElement.textContent = channelPlaceholder;
    } else if ("value" in channelListElement) {
      channelListElement.value = channelPlaceholder;
    }
  }
  const observation =
    typeof observationFactory === "function"
      ? observationFactory()
      : createEmptyObservationState({ includeValues: true });
  return {
    lensGraph: null,
    observation,
  };
}

export async function resolveWasmClientForSource({
  sourceText,
  ensureWasm,
} = {}) {
  if (typeof ensureWasm !== "function") {
    throw new Error("resolveWasmClientForSource: ensureWasm 함수가 필요합니다.");
  }
  const body = stripMetaHeader(sourceText ?? "");
  const client = await ensureWasm(body);
  return { client, body };
}

export function updateWasmClientLogic({
  client,
  sourceBody,
  mode = null,
} = {}) {
  if (!client || typeof client !== "object") {
    throw new Error("updateWasmClientLogic: client가 필요합니다.");
  }
  const body = String(sourceBody ?? "");
  if (mode !== null && mode !== undefined && typeof client.updateLogicWithMode === "function") {
    client.updateLogicWithMode(body, String(mode));
    return;
  }
  if (typeof client.updateLogic === "function") {
    client.updateLogic(body);
    return;
  }
  throw new Error("updateWasmClientLogic: update API가 없습니다.");
}

export async function applyWasmLogicFromSource({
  sourceText,
  ensureWasm,
  mode = null,
} = {}) {
  const { client, body } = await resolveWasmClientForSource({
    sourceText,
    ensureWasm,
  });
  updateWasmClientLogic({
    client,
    sourceBody: body,
    mode,
  });
  if (typeof client.getStateParsed !== "function") {
    throw new Error("applyWasmLogicFromSource: getStateParsed API가 없습니다.");
  }
  const state = client.getStateParsed();
  return { client, body, state };
}

export async function applyWasmLogicAndDispatchState({
  sourceText,
  ensureWasm,
  mode = null,
  resetCachesOptions = null,
  patchMode = false,
  onPatch = null,
  onFull = null,
  onPatchWithSource = null,
  onFullWithSource = null,
  sourceForDispatch = null,
  dispatch = dispatchWasmStateApply,
} = {}) {
  const result = await applyWasmLogicFromSource({
    sourceText,
    ensureWasm,
    mode,
  });
  const resetCaches =
    resetCachesOptions && typeof resetCachesOptions === "object"
      ? resetObservationRuntimeCaches(resetCachesOptions)
      : null;
  const sourceTextDispatch =
    sourceForDispatch === null || sourceForDispatch === undefined
      ? String(result.body ?? "")
      : String(sourceForDispatch);
  const hasSourceAwareHandlers =
    typeof onPatchWithSource === "function" || typeof onFullWithSource === "function";
  if (hasSourceAwareHandlers) {
    dispatchWasmStateApplyWithSource({
      stateJson: result.state,
      sourceText: sourceTextDispatch,
      patchMode: Boolean(patchMode),
      onPatchWithSource,
      onFullWithSource,
      dispatch,
    });
  } else {
    dispatch({
      stateJson: result.state,
      patchMode: Boolean(patchMode),
      onPatch: typeof onPatch === "function" ? onPatch : undefined,
      onFull: typeof onFull === "function" ? onFull : () => {},
    });
  }
  return {
    ...result,
    resetCaches,
  };
}

export async function stepWasmWithInputFromSource({
  sourceText,
  ensureWasm,
  gatherInput,
} = {}) {
  const { client, body } = await resolveWasmClientForSource({
    sourceText,
    ensureWasm,
  });
  const rawInput =
    typeof gatherInput === "function"
      ? gatherInput()
      : { keys: 0, lastKey: "", px: 0, py: 0, dt: 0 };
  const stepped = stepWasmClientParsed({
    client,
    input: rawInput,
    errorPrefix: "stepWasmWithInputFromSource",
  });
  return { client, body, state: stepped.state, input: stepped.input };
}

export async function stepWasmAndDispatchState({
  sourceText,
  ensureWasm,
  gatherInput,
  patchMode = false,
  onPatch = null,
  onFull = null,
  onPatchWithSource = null,
  onFullWithSource = null,
  sourceForDispatch = null,
  dispatch = dispatchWasmStateApply,
} = {}) {
  const result = await stepWasmWithInputFromSource({
    sourceText,
    ensureWasm,
    gatherInput,
  });
  const sourceTextDispatch =
    sourceForDispatch === null || sourceForDispatch === undefined
      ? String(result.body ?? "")
      : String(sourceForDispatch);
  const hasSourceAwareHandlers =
    typeof onPatchWithSource === "function" || typeof onFullWithSource === "function";
  if (hasSourceAwareHandlers) {
    dispatchWasmStateApplyWithSource({
      stateJson: result.state,
      sourceText: sourceTextDispatch,
      patchMode: Boolean(patchMode),
      onPatchWithSource,
      onFullWithSource,
      dispatch,
    });
  } else {
    dispatch({
      stateJson: result.state,
      patchMode: Boolean(patchMode),
      onPatch: typeof onPatch === "function" ? onPatch : undefined,
      onFull: typeof onFull === "function" ? onFull : () => {},
    });
  }
  return result;
}

function normalizeStatusLines(lines) {
  if (Array.isArray(lines)) return lines;
  if (typeof lines === "string") return [lines];
  return [];
}

export function createWasmLoader({
  cacheBust = Date.now(),
  modulePath = "./wasm/ddonirang_tool.js",
  wrapperPath = "./wasm_ddn_wrapper.js",
  fallbackSource = "매틱:움직씨 = { 프레임수 <- 0. 프레임수 <- (프레임수 + 1). }.",
  setStatus,
  clearStatusError,
  loadingStatus = "status: wasm loading...",
  missingExportMessage = "DdnWasmVm export missing",
  formatReadyStatus,
  formatFallbackStatus,
} = {}) {
  let vmClient = null;
  let lastBuildInfo = "";
  let lastPreprocessed = "";

  function withCacheBust(path) {
    const sep = path.includes("?") ? "&" : "?";
    return `${path}${sep}v=${cacheBust}`;
  }

  function reset() {
    vmClient = null;
    lastBuildInfo = "";
    lastPreprocessed = "";
  }

  async function ensure(source) {
    if (vmClient) return vmClient;

    if (typeof setStatus === "function") {
      setStatus(normalizeStatusLines(loadingStatus));
    }
    if (typeof clearStatusError === "function") {
      clearStatusError();
    }

    const wasmModule = await import(withCacheBust(modulePath));
    if (typeof wasmModule.default === "function") {
      await wasmModule.default();
    }

    const { DdnWasmVm } = wasmModule;
    if (typeof wasmModule.wasm_build_info === "function") {
      try {
        lastBuildInfo = wasmModule.wasm_build_info();
      } catch (err) {
        lastBuildInfo = `build_info error: ${String(err?.message ?? err)}`;
      }
    }

    if (typeof DdnWasmVm !== "function") {
      throw new Error(missingExportMessage);
    }

    const wrapper = await import(withCacheBust(wrapperPath));
    const needsSource = DdnWasmVm.length > 0;
    const sourceText = stripMetaHeader(source);
    const cleaned = sourceText.trim();

    if (typeof wasmModule.wasm_preprocess_source === "function") {
      try {
        lastPreprocessed = wasmModule.wasm_preprocess_source(sourceText);
      } catch (err) {
        lastPreprocessed = `preprocess error: ${String(err?.message ?? err)}`;
      }
    }

    let vm;
    if (needsSource && !cleaned) {
      vm = new DdnWasmVm(fallbackSource);
      vmClient = new wrapper.DdnWasmVmClient(vm);
      if (typeof setStatus === "function") {
        const lines =
          typeof formatFallbackStatus === "function"
            ? formatFallbackStatus({ cacheBust, buildInfo: lastBuildInfo })
            : ["status: wasm ready (fallback)"];
        setStatus(normalizeStatusLines(lines));
      }
      return vmClient;
    }

    vm = needsSource ? new DdnWasmVm(cleaned) : new DdnWasmVm();
    vmClient = new wrapper.DdnWasmVmClient(vm);
    if (!needsSource && cleaned) {
      vmClient.updateLogic(cleaned);
    }

    let buildInfo = lastBuildInfo;
    if (!buildInfo && typeof vm.get_build_info === "function") {
      try {
        buildInfo = vm.get_build_info();
      } catch (err) {
        buildInfo = `build_info error: ${String(err?.message ?? err)}`;
      }
    }

    if (typeof setStatus === "function") {
      const lines =
        typeof formatReadyStatus === "function"
          ? formatReadyStatus({ cacheBust, buildInfo })
          : [buildInfo ? `status: wasm ready | ${buildInfo}` : "status: wasm ready"];
      setStatus(normalizeStatusLines(lines));
    }
    return vmClient;
  }

  return {
    ensure,
    reset,
    getLastBuildInfo: () => lastBuildInfo,
    getLastPreprocessed: () => lastPreprocessed,
    getCacheBust: () => cacheBust,
  };
}
