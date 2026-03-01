import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function createMemoryStorage() {
  const table = new Map();
  return {
    getItem(key) {
      return table.has(key) ? table.get(key) : null;
    },
    setItem(key, value) {
      table.set(String(key), String(value));
    },
    removeItem(key) {
      table.delete(String(key));
    },
    clear() {
      table.clear();
    },
  };
}

function createFakeCanvas() {
  const listeners = new Map();
  return {
    _listeners: listeners,
    dataset: {},
    addEventListener(type, handler) {
      const list = listeners.get(type) ?? [];
      list.push(handler);
      listeners.set(type, list);
    },
    emit(type, payload) {
      const list = listeners.get(type) ?? [];
      list.forEach((handler) => handler(payload));
    },
    setPointerCapture() {
      // noop
    },
    releasePointerCapture() {
      // noop
    },
  };
}

function createFakeCanvas2d() {
  const calls = {
    fillText: [],
    arc: 0,
    lineTo: 0,
    fill: 0,
    stroke: 0,
  };
  const ctx = {
    fillStyle: "",
    strokeStyle: "",
    lineWidth: 1,
    globalAlpha: 1,
    font: "",
    clearRect() {},
    fillRect() {},
    beginPath() {},
    closePath() {},
    moveTo() {},
    lineTo() {
      calls.lineTo += 1;
    },
    stroke() {
      calls.stroke += 1;
    },
    fill() {
      calls.fill += 1;
    },
    setLineDash() {},
    arc() {
      calls.arc += 1;
    },
    fillText(text) {
      calls.fillText.push(String(text ?? ""));
    },
    strokeRect() {},
  };
  return {
    width: 320,
    height: 180,
    getContext(type) {
      if (type !== "2d") return null;
      return ctx;
    },
    calls,
  };
}

function createFakeSelect() {
  let options = [];
  return {
    value: "",
    get options() {
      return options;
    },
    set options(next) {
      options = Array.isArray(next) ? next : [];
    },
    get children() {
      return options;
    },
    set innerHTML(_) {
      options = [];
    },
    get innerHTML() {
      return "";
    },
  };
}

function createFakeRafHarness() {
  let nextId = 1;
  const table = new Map();
  const request = (cb) => {
    const id = nextId++;
    table.set(id, cb);
    return id;
  };
  const cancel = (id) => {
    table.delete(id);
  };
  const runOne = (ts) => {
    const first = table.entries().next();
    if (first.done) return false;
    const [id, cb] = first.value;
    table.delete(id);
    cb(ts);
    return true;
  };
  const pending = () => table.size;
  return { request, cancel, runOne, pending };
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/wasm_page_common.js");
  const runtimeStatePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js");
  const ddnPreprocessPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/runtime/ddn_preprocess.js");
  const controlParserPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/control_parser.js");
  const common = await import(pathToFileURL(modulePath).href);
  const runtimeState = await import(pathToFileURL(runtimeStatePath).href);
  const ddnPreprocess = await import(pathToFileURL(ddnPreprocessPath).href);
  const controlParser = await import(pathToFileURL(controlParserPath).href);

  const {
    applyWasmParamDraftToControls,
    applyWasmParamFromUi,
    normalizeLensPresetConfig,
    normalizeWasmParamDraft,
    normalizeWasmParamMode,
    normalizeRenderPoints,
    loadLensPresetState,
    loadWasmParamDraftState,
    readWasmParamDraftFromControls,
    saveLensPresetState,
    saveWasmParamDraftState,
    applyLensPresetSelectionState,
    saveLensPresetToState,
    deleteLensPresetFromState,
    bindSpace2dCanvasPanZoom,
    bindSpace2dCanvasWorldInteractions,
    buildTagValueRowsFromStore,
    buildTagValueTableFromStore,
    applyWasmLogicAndDispatchState,
    applyWasmLogicFromSource,
    buildObservationLensGraph,
    collectStateResourceLines,
    computeWasmStepDeltaSeconds,
    composeObservationRenderState,
    createEmptyObservationState,
    createManagedRafStepLoop,
    createObservationLensState,
    createEmptyStructuredViewRawSlots,
    createRafStepLoop,
    normalizeWasmStepInput,
    observationChannelSignature,
    parsePatchJsonObject,
    pushObservationLensSample,
    processPatchOperations,
    removePatchComponentStoreEntry,
    renderGraphCanvas2d,
    renderGraphOrSpace2dCanvas,
    renderObservationChannelList,
    renderSpace2dCanvas2d,
    applyObservationRenderEffects,
    dispatchWasmStateApply,
    dispatchWasmStateApplyWithSource,
    inferPatchSchemaFromJsonObject,
    initializeObservationLensUi,
    isPatchScalarValue,
    upsertPatchComponentStoreEntry,
    upsertPatchScalarStoreEntry,
    resetObservationLensTimeline,
    resetObservationRuntimeCaches,
    resolveWasmClientForSource,
    isGraphEmptyForRender,
    syncWasmSettingsControlsFromState,
    stepWasmAndDispatchState,
    stepWasmClientWithTimingAndDispatch,
    stepWasmClientAndDispatchState,
    stepWasmClientParsed,
    stepWasmWithInputFromSource,
    syncObservationLensFrame,
    toObservationFiniteNumber,
    updateWasmClientLogic,
    updateLensSelectorsFromObservation,
  } = common;
  const {
    extractObservationChannelsFromState,
    extractStructuredViewsFromState,
    normalizeWasmStatePayload,
  } = runtimeState;
  const { preprocessDdnText } = ddnPreprocess;
  const { buildControlSpecsFromDdn } = controlParser;

  const preprocessLegacyShow = preprocessDdnText('"안녕"을 보여주기.\n"세계"를 보여주기.\n// "주석"을 보여주기.\n');
  assert(preprocessLegacyShow.bodyText.includes("매틱:움직씨 = {"), "ddn preprocess: wrap movement seed");
  assert(preprocessLegacyShow.bodyText.includes("보개_출력_줄들 <- () 차림."), "ddn preprocess: reset show lines");
  assert(
    preprocessLegacyShow.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("안녕") 글로) 추가.'),
    "ddn preprocess: 을 rewrite + show compatibility",
  );
  assert(
    preprocessLegacyShow.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("세계") 글로) 추가.'),
    "ddn preprocess: 를 rewrite + show compatibility",
  );
  assert(preprocessLegacyShow.bodyText.includes('// "주석"을 보여주기.'), "ddn preprocess: comment keep");
  assert(!preprocessLegacyShow.bodyText.includes('"안녕" 보여주기.'), "ddn preprocess: show canonical removed");

  const preprocessBogaeBlock = preprocessDdnText(`
매틱:움직씨 = {
  모양: {
    선(0, 0, bob_x, bob_y, 색="#9ca3af", 굵기=0.02).
    원(bob_x, bob_y, r=0.08, 색="#38bdf8", 선색="#0ea5e9", 굵기=0.02).
    점(0, 0, 크기=0.045, 색="#f59e0b").
  }.
}.
`);
  assert(!preprocessBogaeBlock.bodyText.includes("보개 {"), "ddn preprocess: bogae block removed");
  assert(!preprocessBogaeBlock.bodyText.includes("모양"), "ddn preprocess: moyang block removed");
  assert(
    preprocessBogaeBlock.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("space2d") 글로) 추가.'),
    "ddn preprocess: bogae block emits space2d marker",
  );
  assert(
    preprocessBogaeBlock.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("space2d.shape") 글로) 추가.'),
    "ddn preprocess: bogae block emits shape marker",
  );
  assert(
    preprocessBogaeBlock.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("line") 글로) 추가.'),
    "ddn preprocess: bogae block emits line kind",
  );
  assert(
    preprocessBogaeBlock.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("circle") 글로) 추가.'),
    "ddn preprocess: bogae block emits circle kind",
  );
  assert(
    preprocessBogaeBlock.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("point") 글로) 추가.'),
    "ddn preprocess: bogae block emits point kind",
  );

  const preprocessBogaeMadang = preprocessDdnText(`
보개마당 {
  토막(이름="도입", 마디=0..60) {
    자막(글="줄길이를 늘리면 주기가 길어진다.", 자리=(0.0, 0.86)).
  }.
}.
`);
  assert(!preprocessBogaeMadang.bodyText.includes("보개마당 {"), "ddn preprocess: bogae madang block removed");
  assert(
    preprocessBogaeMadang.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("text.overlay") 글로) 추가.'),
    "ddn preprocess: bogae madang emits text overlay marker",
  );
  assert(
    preprocessBogaeMadang.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("markdown") 글로) 추가.'),
    "ddn preprocess: bogae madang emits markdown key",
  );
  assert(
    preprocessBogaeMadang.bodyText.includes('보개_출력_줄들 <- (보개_출력_줄들, ("줄길이를 늘리면 주기가 길어진다.") 글로) 추가.'),
    "ddn preprocess: bogae madang emits subtitle text",
  );

  const preprocessLegacyLifecycle = preprocessDdnText(`
채비: {
  길이:수 <- 1.
}.
(시작)할때 {
  t <- 0.
  theta <- 0.5.
}.
(매마디)마다 {
  t <- t + 0.1.
  t 보여주기.
}.
`);
  const lifecycleBody = preprocessLegacyLifecycle.bodyText;
  const lifecycleChabiPos = lifecycleBody.indexOf("채비:");
  const lifecycleMovementPos = lifecycleBody.indexOf("매틱:움직씨 = {");
  const lifecycleStartGuardPos = lifecycleBody.indexOf("{ __wasm_start_once <= 0 }인것 일때 {");
  const lifecycleStartInitPos = lifecycleBody.indexOf("t <- 0.");
  const lifecycleResetPos = lifecycleBody.indexOf("보개_출력_줄들 <- () 차림.");
  assert(
    lifecycleChabiPos >= 0 && lifecycleMovementPos > lifecycleChabiPos,
    "ddn preprocess: keep top-level decls outside movement",
  );
  assert(
    !lifecycleBody.includes("(시작)할때") && !lifecycleBody.includes("(매마디)마다"),
    "ddn preprocess: legacy lifecycle hooks rewritten into movement",
  );
  assert(
    lifecycleStartGuardPos >= 0 && lifecycleStartInitPos > lifecycleStartGuardPos,
    "ddn preprocess: place start body inside wasm start-once guard",
  );
  assert(
    !lifecycleBody.includes("내부시작완료"),
    "ddn preprocess: remove legacy synthetic guard key",
  );
  assert(
    lifecycleResetPos > lifecycleMovementPos,
    "ddn preprocess: inject output reset inside movement seed",
  );

  const controlSplit = buildControlSpecsFromDdn(`
#기본관찰: 각도
#기본관찰x: tick
채비: {
  길이: 상수 <- 1.2.
  각도: 변수 <- 0.5. // 기본관찰
  tick: 변수 <- 0. // 기본관찰x
  질량: 수 <- 1.0.
}
`);
  assert(controlSplit.source === "prep", "control parser: prep(채비) source");
  assert(controlSplit.specs.some((item) => item.name === "길이"), "control parser: constant slider included");
  assert(!controlSplit.specs.some((item) => item.name === "각도"), "control parser: variable excluded from sliders");
  assert(controlSplit.axisKeys.includes("각도"), "control parser: variable exposed as axis key");
  assert(controlSplit.axisKeys.includes("tick"), "control parser: x variable exposed as axis key");
  assert(!controlSplit.axisKeys.includes("길이"), "control parser: constant excluded from axis keys");
  assert(controlSplit.defaultAxisKey === "각도", "control parser: default observation key priority");
  assert(controlSplit.defaultXAxisKey === "tick", "control parser: default x observation key priority");

  const controlDecimalDefaults = buildControlSpecsFromDdn(`
채비: {
  g:수 <- 9.8. // 범위(1, 20, 0.1)
  theta0:수 <- 0.5. // 범위(-1.2, 1.2, 0.05)
  dt:수 <- 0.02. // 범위(0.005, 0.1, 0.005)
}
`);
  const gSpec = controlDecimalDefaults.specs.find((item) => item.name === "g");
  const theta0Spec = controlDecimalDefaults.specs.find((item) => item.name === "theta0");
  const dtSpec = controlDecimalDefaults.specs.find((item) => item.name === "dt");
  assert(gSpec && Math.abs(gSpec.value - 9.8) < 1e-9, "control parser: decimal default g");
  assert(theta0Spec && Math.abs(theta0Spec.value - 0.5) < 1e-9, "control parser: decimal default theta0");
  assert(dtSpec && Math.abs(dtSpec.value - 0.02) < 1e-9, "control parser: decimal default dt");

  const controlPolicySeedIgnore = buildControlSpecsFromDdn(`
채비: {
  길이:상수 <- 1.2.
  각도:변수 <- 0.5.
}
씨앗: {
  감춰진축:변수 <- 7.
}
`);
  assert(controlPolicySeedIgnore.axisKeys.includes("각도"), "control policy: 채비 변수 axis include");
  assert(!controlPolicySeedIgnore.axisKeys.includes("감춰진축"), "control policy: 씨앗 변수 axis exclude");
  assert(
    !controlPolicySeedIgnore.specs.some((item) => item.name === "감춰진축"),
    "control policy: 씨앗 변수 slider exclude",
  );

  const empty0 = createEmptyObservationState();
  assert(Array.isArray(empty0.channels) && Array.isArray(empty0.row), "empty observation: shape");
  assert(typeof empty0.values === "object" && empty0.values !== null, "empty observation: values");
  const empty1 = createEmptyObservationState({ includeValues: false });
  assert(!Object.prototype.hasOwnProperty.call(empty1, "values"), "empty observation: optional values");
  const rawSlots0 = createEmptyStructuredViewRawSlots();
  const rawSlots1 = createEmptyStructuredViewRawSlots();
  assert(rawSlots0.graph === null && rawSlots0.space2d === null, "raw slots: null defaults");
  assert(rawSlots0 !== rawSlots1, "raw slots: fresh object");

  const priorityPayload = {
    schema: "seamgrim.engine_response.v0",
    tick_id: 0,
    state_hash: "blake3:test",
    resources: {
      json: {},
      fixed64: {},
      handle: {},
      value: {
        "그래프_접두어": '[{"x":0,"y":1},{"x":1,"y":2}]',
      },
    },
    state: {
      streams: {
        "속도흐름": { capacity: 4, head: 1, len: 2, buffer: [10, 20, null, null] },
      },
      channels: [],
      row: [],
      resources: {
        json: {},
        fixed64: {},
        handle: {},
        value: {},
      },
    },
    view_meta: {
      graph_hints: [
        { series_id: "hint", source: "속도흐름", y_label: "hint", overlay: true },
      ],
      draw_list: [{ kind: "line", x1: 0, y1: 0, x2: 1, y2: 1 }],
      space2d: { schema: "seamgrim.space2d.v0", points: [{ x: 0, y: 0 }] },
    },
  };
  const normalizedPriority = normalizeWasmStatePayload(priorityPayload);
  assert(typeof normalizedPriority === "object" && normalizedPriority !== null, "runtime state: normalized payload");
  const priorityViews = extractStructuredViewsFromState(priorityPayload);
  assert(priorityViews && typeof priorityViews === "object", "runtime state: structured views object");
  assert(Object.prototype.hasOwnProperty.call(priorityViews, "space2d"), "runtime state: structured space2d key");

  const showPayload = {
    schema: "seamgrim.engine_response.v0",
    tick_id: 0,
    state_hash: "blake3:show",
    resources: {
      json: {},
      fixed64: {},
      handle: {},
      value: {
        "보개_출력_줄들": '차림["1", "2", "3", "4"]',
      },
    },
    patch: [
      {
        op: "set_resource_value",
        tag: "보개_출력_줄들",
        value: '차림["1", "2", "3", "4"]',
      },
    ],
    state: {
      channels: [],
      row: [],
      resources: {
        json: {},
        fixed64: {},
        handle: {},
        value: {
          "보개_출력_줄들": '차림["1", "2", "3", "4"]',
        },
      },
    },
  };
  const showViews = extractStructuredViewsFromState(showPayload, { preferPatch: true });
  assert(showViews && typeof showViews === "object", "runtime state: show payload views");
  assert(Object.prototype.hasOwnProperty.call(showViews, "graph"), "runtime state: show payload graph key");
  assert(Object.prototype.hasOwnProperty.call(showViews, "text"), "runtime state: show payload text key");

  const outputLinesPayload = {
    schema: "seamgrim.state.v0",
    channels: [{ key: "보개_출력_줄들", dtype: "text", role: "state" }],
    row: [
      JSON.stringify([
        "space2d",
        "space2d.shape",
        "line",
        "x1",
        "0",
        "y1",
        "0",
        "x2",
        "1",
        "y2",
        "1",
        "stroke",
        "#9ca3af",
        "width",
        "0.02",
      ]),
    ],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
  };
  const outputLineViews = extractStructuredViewsFromState(outputLinesPayload, { preferPatch: false });
  assert(outputLineViews?.space2d && typeof outputLineViews.space2d === "object", "output lines: space2d extracted");
  assert(Array.isArray(outputLineViews.space2d.shapes), "output lines: shapes array");
  assert(outputLineViews.space2d.shapes.length === 1, "output lines: shape count");
  assert(outputLineViews.space2d.shapes[0].kind === "line", "output lines: line kind");

  const outputLinesLegacyPayload = {
    schema: "seamgrim.state.v0",
    channels: [{ key: "보개_출력_줄들", dtype: "text", role: "state" }],
    row: ['차림["space2d","space2d.shape","point","x","0.25","y","-0.5","size","0.08","color","#22c55e"]'],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
  };
  const outputLineLegacyViews = extractStructuredViewsFromState(outputLinesLegacyPayload, { preferPatch: false });
  assert(outputLineLegacyViews?.space2d && typeof outputLineLegacyViews.space2d === "object", "output lines legacy: space2d extracted");
  assert(Array.isArray(outputLineLegacyViews.space2d.shapes), "output lines legacy: shapes array");
  assert(outputLineLegacyViews.space2d.shapes.length === 1, "output lines legacy: shape count");
  assert(outputLineLegacyViews.space2d.shapes[0].kind === "point", "output lines legacy: point kind");

  const outputLinesLegacyNegativeWrapPayload = {
    schema: "seamgrim.state.v0",
    channels: [{ key: "보개_출력_줄들", dtype: "text", role: "state" }],
    row: ['차림["space2d","space2d.shape","line","x1","0","y1","0","x2","-1.970185","y2","-1.000444","stroke","#9ca3af","width","0.02"]'],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
  };
  const outputLineLegacyNegativeWrapViews = extractStructuredViewsFromState(outputLinesLegacyNegativeWrapPayload, { preferPatch: false });
  assert(outputLineLegacyNegativeWrapViews?.space2d && typeof outputLineLegacyNegativeWrapViews.space2d === "object", "output lines legacy negative wrap: space2d extracted");
  const wrappedLine = outputLineLegacyNegativeWrapViews.space2d.shapes?.[0];
  assert(wrappedLine?.kind === "line", "output lines legacy negative wrap: line kind");
  assert(Math.abs(Number(wrappedLine?.x2) - (-0.029815)) < 1e-4, "output lines legacy negative wrap: x2 decoded");
  assert(Math.abs(Number(wrappedLine?.y2) - (-0.999556)) < 1e-4, "output lines legacy negative wrap: y2 decoded");

  const outputLinesResourcePayload = {
    schema: "seamgrim.state.v0",
    channels: [],
    row: [],
    resources: {
      json: {},
      fixed64: {},
      handle: {},
      value: {
        "보개_출력_줄들": '차림["space2d","space2d.shape","circle","x","0","y","0","r","0.5","fill","#38bdf8"]',
      },
    },
    patch: [
      {
        op: "set_resource_value",
        tag: "보개_출력_줄들",
        value: '차림["space2d","space2d.shape","circle","x","0","y","0","r","0.5","fill","#38bdf8"]',
      },
    ],
  };
  const outputLineResourceViews = extractStructuredViewsFromState(outputLinesResourcePayload, { preferPatch: false });
  assert(outputLineResourceViews?.space2d && typeof outputLineResourceViews.space2d === "object", "output lines resource: space2d extracted");
  assert(Array.isArray(outputLineResourceViews.space2d.shapes), "output lines resource: shapes array");
  assert(outputLineResourceViews.space2d.shapes.length === 1, "output lines resource: shape count");
  assert(outputLineResourceViews.space2d.shapes[0].kind === "circle", "output lines resource: circle kind");

  const outputLinesTextPayload = {
    schema: "seamgrim.state.v0",
    channels: [{ key: "보개_출력_줄들", dtype: "text", role: "state" }],
    row: [JSON.stringify(["text.overlay", "markdown", "도입 설명", "x", "0.0", "y", "0.86"])],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
  };
  const outputLineTextViews = extractStructuredViewsFromState(outputLinesTextPayload, { preferPatch: false });
  assert(outputLineTextViews?.text && typeof outputLineTextViews.text === "object", "output lines text: text extracted");
  assert(outputLineTextViews.text.markdown === "도입 설명", "output lines text: markdown value");
  assert(Math.abs(Number(outputLineTextViews.text.x) - 0.0) < 1e-9, "output lines text: x parsed");
  assert(Math.abs(Number(outputLineTextViews.text.y) - 0.86) < 1e-9, "output lines text: y parsed");

  const manifestPayload = {
    schema: "seamgrim.state.v0",
    channels: [
      { key: "x", dtype: "number", role: "state" },
      { key: "y", dtype: "number", role: "state" },
      { key: "theta", dtype: "number", role: "state" },
    ],
    row: [10, 20, 0.33],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
    observation_manifest: {
      schema: "ddn.observation_manifest.v0",
      version: "20.6.33",
      nodes: [
        { name: "y", dtype: "number", role: "상태" },
        { name: "x", dtype: "number", role: "상태" },
      ],
    },
  };
  const manifestObservation = extractObservationChannelsFromState(manifestPayload);
  assert(manifestObservation.channels.length === 2, "manifest channels: count");
  assert(manifestObservation.channels[0].key === "y", "manifest channels: manifest order");
  assert(manifestObservation.row[0] === 20, "manifest row: value mapped by key");
  assert(manifestObservation.values.theta === undefined, "manifest values: hidden key absent");
  assert(manifestObservation.all_values.theta === 0.33, "manifest values: hidden key preserved");

  const stringChannelPayload = {
    schema: "seamgrim.state.v0",
    channels: ["t", "theta"],
    row: [0.02, 0.51],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
  };
  const stringChannelObservation = extractObservationChannelsFromState(stringChannelPayload);
  assert(stringChannelObservation.channels[0].key === "t", "string channels: key normalize");
  assert(stringChannelObservation.values.theta === 0.51, "string channels: values map");

  const nameChannelPayload = {
    schema: "seamgrim.state.v0",
    channels: [
      { name: "time", dtype: "number", role: "state" },
      { name: "angle", dtype: "number", role: "state" },
    ],
    row: [0.04, 0.48],
    resources: { json: {}, fixed64: {}, handle: {}, value: {} },
  };
  const nameChannelObservation = extractObservationChannelsFromState(nameChannelPayload);
  assert(nameChannelObservation.channels[1].key === "angle", "name channels: key fallback");
  assert(nameChannelObservation.values.time === 0.04, "name channels: values map");

  const lensBase = createObservationLensState({ maxPoints: 240, includeRuns: true, lastFrameToken: null });
  assert(lensBase.maxPoints === 240, "lens state: maxPoints");
  assert(Array.isArray(lensBase.timeline) && lensBase.timeline.length === 0, "lens state: timeline init");
  assert(Array.isArray(lensBase.runs) && lensBase.runs.length === 0, "lens state: runs init");
  assert(lensBase.presets?.default?.xKey === "__tick__", "lens state: default preset");
  lensBase.timeline.push({ __tick__: 1 });
  lensBase.runs.push({ id: "r1" });
  resetObservationLensTimeline(lensBase, { lastFrameToken: null, clearRuns: true });
  assert(lensBase.timeline.length === 0, "lens reset: timeline cleared");
  assert(lensBase.runs.length === 0, "lens reset: runs cleared");
  assert(lensBase.lastFrameToken === null, "lens reset: frame token");
  const channelSlot = { textContent: "before" };
  const runtimeCacheReset = resetObservationRuntimeCaches({
    lensState: lensBase,
    lastFrameToken: "token:1",
    clearRuns: true,
    channelListElement: channelSlot,
    resetSpace2dView: () => {
      lensBase._spaceReset = (lensBase._spaceReset ?? 0) + 1;
    },
    observationFactory: () => createEmptyObservationState({ includeValues: false }),
  });
  assert(runtimeCacheReset.lensGraph === null, "runtime cache reset: lensGraph");
  assert(
    Array.isArray(runtimeCacheReset.observation.channels) &&
      runtimeCacheReset.observation.values === undefined,
    "runtime cache reset: observation factory",
  );
  assert(lensBase._spaceReset === 1, "runtime cache reset: space2d reset callback");
  assert(channelSlot.textContent === "-", "runtime cache reset: channel placeholder");

  const normalizedStepInput = normalizeWasmStepInput({
    keys: "7.9",
    lastKey: 123,
    px: "10.7",
    py: "-9.2",
    dt: "-0.1",
  });
  assert(normalizedStepInput.keys === 7, "normalize step input: keys");
  assert(normalizedStepInput.lastKey === "123", "normalize step input: key name");
  assert(normalizedStepInput.px === 10 && normalizedStepInput.py === -9, "normalize step input: point");
  assert(normalizedStepInput.dt === 0, "normalize step input: dt clamp");

  const paramClient = {
    setParamParsed(key, value) {
      return { ok: true, path: "scalar", key, value };
    },
    setParamFixed64Parsed(key, rawI64) {
      return { ok: true, path: "fixed64", key, rawI64 };
    },
    setParamFixed64StringParsed(key, rawI64Text) {
      return { ok: true, path: "fixed64_str", key, rawI64Text };
    },
  };
  const paramScalar = applyWasmParamFromUi({
    client: paramClient,
    key: "속도",
    rawValue: "12.5",
    mode: "scalar",
  });
  assert(paramScalar.ok && paramScalar.valueKind === "number", "param helper: scalar number");
  assert(paramScalar.result?.path === "scalar", "param helper: scalar dispatch");
  const paramBool = applyWasmParamFromUi({
    client: paramClient,
    key: "활성",
    rawValue: "true",
    mode: "scalar",
  });
  assert(paramBool.ok && paramBool.valueKind === "boolean", "param helper: scalar bool");
  const paramString = applyWasmParamFromUi({
    client: paramClient,
    key: "이름",
    rawValue: "hello",
    mode: "scalar",
  });
  assert(paramString.ok && paramString.valueKind === "string", "param helper: scalar string fallback");
  const paramFixed = applyWasmParamFromUi({
    client: paramClient,
    key: "속도_raw",
    rawValue: "4294967296",
    mode: "fixed64_raw",
  });
  assert(paramFixed.ok && paramFixed.valueKind === "raw_i64", "param helper: fixed64 mode");
  assert(paramFixed.result?.path === "fixed64_str", "param helper: fixed64 string dispatch");
  const paramClientLegacy = {
    setParamParsed: paramClient.setParamParsed,
    setParamFixed64Parsed: paramClient.setParamFixed64Parsed,
  };
  const paramFixedLegacy = applyWasmParamFromUi({
    client: paramClientLegacy,
    key: "속도_raw",
    rawValue: "4294967296",
    mode: "fixed64_raw",
  });
  assert(paramFixedLegacy.ok, "param helper: fixed64 legacy fallback");
  assert(paramFixedLegacy.result?.path === "fixed64", "param helper: fixed64 numeric fallback");
  const paramFixedOverflowLegacy = applyWasmParamFromUi({
    client: paramClientLegacy,
    key: "속도_raw",
    rawValue: "9223372036854775807",
    mode: "fixed64_raw",
  });
  assert(!paramFixedOverflowLegacy.ok, "param helper: fixed64 overflow reject on legacy api");
  const paramBad = applyWasmParamFromUi({
    client: paramClient,
    key: "",
    rawValue: "1",
    mode: "scalar",
  });
  assert(!paramBad.ok, "param helper: empty key reject");
  assert(normalizeWasmParamMode("fixed64_raw") === "fixed64_raw", "param mode normalize: fixed64");
  assert(normalizeWasmParamMode("other") === "scalar", "param mode normalize: fallback");
  const normalizedDraft = normalizeWasmParamDraft({
    key: 7,
    mode: "fixed64_raw",
    value: 123,
  });
  assert(normalizedDraft.key === "7", "param draft normalize: key");
  assert(normalizedDraft.mode === "fixed64_raw", "param draft normalize: mode");
  assert(normalizedDraft.value === "123", "param draft normalize: value");
  const paramControls = {
    keyInput: { value: " 중력 " },
    modeSelect: { value: "fixed64_raw" },
    valueInput: { value: "42" },
  };
  const readDraft = readWasmParamDraftFromControls(paramControls);
  assert(readDraft.key === " 중력 ", "param controls read: key");
  assert(readDraft.mode === "fixed64_raw", "param controls read: mode");
  assert(readDraft.value === "42", "param controls read: value");
  const writeControls = {
    keyInput: { value: "" },
    modeSelect: { value: "" },
    valueInput: { value: "" },
  };
  const written = applyWasmParamDraftToControls({
    ...writeControls,
    draft: { key: "속도", mode: "invalid", value: 10 },
  });
  assert(written.mode === "scalar", "param controls write: mode normalize");
  assert(writeControls.keyInput.value === "속도", "param controls write: key");
  assert(writeControls.modeSelect.value === "scalar", "param controls write: mode");
  assert(writeControls.valueInput.value === "10", "param controls write: value");

  const delta0 = computeWasmStepDeltaSeconds({
    nowMs: 1500,
    lastTickMs: 1000,
    dtMin: 0.25,
    dtMax: 0.4,
  });
  assert(delta0.dtSec === 0.4, "step delta: clamp max");
  const delta1 = computeWasmStepDeltaSeconds({
    nowMs: 1005,
    lastTickMs: 1000,
    dtMin: 0.02,
    dtMax: 0.5,
  });
  assert(delta1.dtSec === 0.02, "step delta: clamp min");
  const delta2 = computeWasmStepDeltaSeconds({
    nowMs: 2000,
    lastTickMs: 1000,
    fixedDtEnabled: true,
    fixedDtValue: 0.0333,
    dtMin: 0.05,
    dtMax: 0.1,
  });
  assert(Math.abs(delta2.dtSec - 0.05) < 1e-9, "step delta: fixed dt + min clamp");

  const raf = createFakeRafHarness();
  const rafErrors = [];
  let rafSteps = 0;
  const rafLoop = createRafStepLoop({
    requestFrame: raf.request,
    cancelFrame: raf.cancel,
    getFps: () => 10,
    getNow: () => 0,
    onStep: () => {
      rafSteps += 1;
    },
    onError: (err) => {
      rafErrors.push(String(err?.message ?? err));
    },
  });
  assert(rafLoop.start() === true, "raf loop: start");
  assert(raf.pending() === 1, "raf loop: scheduled");
  raf.runOne(0);
  assert(rafSteps === 1, "raf loop: first step");
  raf.runOne(50);
  assert(rafSteps === 1, "raf loop: fps gate");
  raf.runOne(120);
  assert(rafSteps === 2, "raf loop: second step");
  rafLoop.stop();
  assert(raf.pending() === 0, "raf loop: stop clears schedule");
  assert(rafLoop.isRunning() === false, "raf loop: stopped");
  const rafErrHarness = createFakeRafHarness();
  const rafErrLoop = createRafStepLoop({
    requestFrame: rafErrHarness.request,
    cancelFrame: rafErrHarness.cancel,
    onStep: () => {
      throw new Error("boom");
    },
    onError: (err) => {
      rafErrors.push(String(err?.message ?? err));
    },
  });
  rafErrLoop.start();
  rafErrHarness.runOne(0);
  assert(rafErrors.some((item) => item.includes("boom")), "raf loop: error callback");
  assert(rafErrLoop.isRunning() === false, "raf loop: stops on error");

  const managedHarness = createFakeRafHarness();
  let managedActive = false;
  let managedSteps = 0;
  let managedStartCount = 0;
  let managedStopCount = 0;
  const managedErrors = [];
  const managedLoop = createManagedRafStepLoop({
    requestFrame: managedHarness.request,
    cancelFrame: managedHarness.cancel,
    getFps: () => 30,
    isActive: () => managedActive,
    setActive: (next) => {
      managedActive = Boolean(next);
    },
    onStart: () => {
      managedStartCount += 1;
    },
    onStop: () => {
      managedStopCount += 1;
    },
    onStep: () => {
      managedSteps += 1;
    },
    onError: (err) => {
      managedErrors.push(String(err?.message ?? err));
    },
  });
  assert(managedLoop.start() === true, "managed raf: start");
  assert(managedLoop.isActive() === true, "managed raf: active");
  managedHarness.runOne(0);
  assert(managedSteps === 1, "managed raf: step");
  assert(managedStartCount === 1, "managed raf: onStart");
  assert(managedLoop.stop() === true, "managed raf: stop");
  assert(managedLoop.isActive() === false, "managed raf: inactive after stop");
  assert(managedStopCount === 1, "managed raf: onStop");
  assert(managedLoop.start() === true, "managed raf: restart");
  managedLoop.stop();
  const managedFailHarness = createFakeRafHarness();
  const managedFailLoop = createManagedRafStepLoop({
    requestFrame: managedFailHarness.request,
    cancelFrame: managedFailHarness.cancel,
    isActive: () => managedActive,
    setActive: (next) => {
      managedActive = Boolean(next);
    },
    onStep: () => {
      throw new Error("managed-boom");
    },
    onError: (err) => {
      managedErrors.push(String(err?.message ?? err));
    },
  });
  managedFailLoop.start();
  managedFailHarness.runOne(10);
  assert(managedActive === false, "managed raf: error clears active");
  assert(managedErrors.some((item) => item.includes("managed-boom")), "managed raf: error callback");

  const n0 = normalizeLensPresetConfig({ enabled: 1, x_key: "x", y_key: "y", y2_key: "z" });
  assert(n0.enabled === true, "normalize: enabled");
  assert(n0.xKey === "x" && n0.yKey === "y" && n0.y2Key === "z", "normalize: key map");
  const n1 = normalizeLensPresetConfig(null);
  assert(n1.enabled === false && n1.xKey === "__tick__", "normalize: defaults");

  let ensureArg = "";
  const fakeClient = {
    updateCalls: [],
    updateModeCalls: [],
    updateLogic(body) {
      this.updateCalls.push(String(body ?? ""));
    },
    updateLogicWithMode(body, mode) {
      this.updateModeCalls.push({ body: String(body ?? ""), mode: String(mode ?? "") });
    },
    getStateParsed() {
      return { schema: "seamgrim.state.v0", tick_id: 1 };
    },
    stepOneWithInputParsed(keys, lastKey, px, py, dt) {
      return { schema: "seamgrim.state.v0", keys, lastKey, px, py, dt };
    },
  };
  const ensureWasm = async (body) => {
    ensureArg = String(body ?? "");
    return fakeClient;
  };
  const resolved = await resolveWasmClientForSource({
    sourceText: "#이름: 테스트\n매틱:움직씨 = { x <- 1. }.",
    ensureWasm,
  });
  assert(resolved.client === fakeClient, "resolve wasm client: client");
  assert(ensureArg.startsWith("매틱:움직씨"), "resolve wasm client: meta strip");
  updateWasmClientLogic({ client: fakeClient, sourceBody: "x <- 1." });
  assert(fakeClient.updateCalls.length === 1, "update wasm logic: compat");
  updateWasmClientLogic({ client: fakeClient, sourceBody: "x <- 2.", mode: "strict" });
  assert(fakeClient.updateModeCalls.length === 1, "update wasm logic: mode");
  const applied = await applyWasmLogicFromSource({
    sourceText: "매틱:움직씨 = { x <- 3. }.",
    ensureWasm,
    mode: "strict",
  });
  assert(applied.state?.schema === "seamgrim.state.v0", "apply wasm logic: state");
  const stepped = await stepWasmWithInputFromSource({
    sourceText: "매틱:움직씨 = { x <- 4. }.",
    ensureWasm,
    gatherInput: () => ({ keys: 3, lastKey: "w", px: 10, py: 20, dt: 0.1 }),
  });
  assert(stepped.state?.keys === 3, "step wasm logic: input keys");
  assert(stepped.state?.lastKey === "w", "step wasm logic: input last key");
  const applyDispatchLensState = createObservationLensState({
    enabled: false,
    xKey: "__tick__",
    yKey: "",
    y2Key: "",
    maxPoints: 10,
  });
  let applyDispatchRendered = null;
  const applyDispatched = await applyWasmLogicAndDispatchState({
    sourceText: "매틱:움직씨 = { x <- 10. }.",
    ensureWasm,
    resetCachesOptions: {
      lensState: applyDispatchLensState,
      lastFrameToken: "apply:1",
      observationFactory: () => createEmptyObservationState({ includeValues: false }),
    },
    patchMode: false,
    onFull: (stateJson) => {
      applyDispatchRendered = stateJson;
    },
  });
  assert(applyDispatchRendered?.schema === "seamgrim.state.v0", "apply+dispatch: onFull");
  assert(applyDispatched.resetCaches?.lensGraph === null, "apply+dispatch: reset lens graph");
  assert(
    Array.isArray(applyDispatched.resetCaches?.observation?.channels),
    "apply+dispatch: reset observation",
  );
  let applyDispatchSource = "";
  await applyWasmLogicAndDispatchState({
    sourceText: "#이름: source-aware\n매틱:움직씨 = { x <- 12. }.",
    ensureWasm,
    patchMode: false,
    onFullWithSource: (_stateJson, source) => {
      applyDispatchSource = String(source ?? "");
    },
  });
  assert(
    applyDispatchSource.startsWith("매틱:움직씨"),
    "apply+dispatch: source-aware callback",
  );
  let stepDispatchRendered = null;
  const stepDispatched = await stepWasmAndDispatchState({
    sourceText: "매틱:움직씨 = { x <- 11. }.",
    ensureWasm,
    gatherInput: () => ({ keys: 9, lastKey: "b", px: 2, py: 3, dt: 0.2 }),
    patchMode: false,
    onFull: (stateJson) => {
      stepDispatchRendered = stateJson;
    },
  });
  assert(stepDispatchRendered?.schema === "seamgrim.state.v0", "step+dispatch: onFull");
  assert(stepDispatched.input?.keys === 9, "step+dispatch: input");
  let stepDispatchSource = "";
  await stepWasmAndDispatchState({
    sourceText: "#설명: step-source\n매틱:움직씨 = { x <- 13. }.",
    ensureWasm,
    gatherInput: () => ({ keys: 1, lastKey: "c", px: 0, py: 0, dt: 0.05 }),
    patchMode: false,
    onFullWithSource: (_stateJson, source) => {
      stepDispatchSource = String(source ?? "");
    },
  });
  assert(
    stepDispatchSource.startsWith("매틱:움직씨"),
    "step+dispatch: source-aware callback",
  );
  const directStepped = stepWasmClientParsed({
    client: fakeClient,
    input: { keys: "5", lastKey: "k", px: "9.8", py: 2, dt: "0.25" },
  });
  assert(directStepped.state?.keys === 5, "step wasm client parsed: keys");
  assert(directStepped.state?.px === 9, "step wasm client parsed: px");
  let missingApiError = "";
  try {
    stepWasmClientParsed({ client: {} });
  } catch (err) {
    missingApiError = String(err?.message ?? err);
  }
  assert(missingApiError.includes("stepOneWithInputParsed"), "step wasm client parsed: missing api");

  assert(
    observationChannelSignature([{ key: "a" }, { key: "b" }]) === "a|b",
    "channel signature",
  );
  assert(toObservationFiniteNumber("3.25") === 3.25, "finite number: string");
  assert(toObservationFiniteNumber("abc") === null, "finite number: invalid");

  const previousStorage = globalThis.localStorage;
  globalThis.localStorage = createMemoryStorage();
  try {
    saveWasmParamDraftState({
      storageKey: "param.test",
      draft: { key: "g", mode: "fixed64_raw", value: "123" },
    });
    const loadedParamDraft = loadWasmParamDraftState({
      storageKey: "param.test",
      fallback: { key: "", mode: "scalar", value: "" },
    });
    assert(loadedParamDraft.key === "g", "param storage: key");
    assert(loadedParamDraft.mode === "fixed64_raw", "param storage: mode");
    assert(loadedParamDraft.value === "123", "param storage: value");

    saveLensPresetState({
      storageKey: "lens.test",
      presetId: "demo",
      presets: {
        default: { enabled: false, xKey: "__tick__", yKey: "", y2Key: "" },
        demo: { enabled: true, xKey: "tick_id", yKey: "v", y2Key: "" },
      },
    });
    const loaded = loadLensPresetState({
      storageKey: "lens.test",
      defaultPreset: { enabled: false, xKey: "__tick__", yKey: "", y2Key: "" },
      preferredPresetId: "demo",
    });
    assert(loaded.presetId === "demo", "load/save: active preset");
    assert(Boolean(loaded.presets.demo), "load/save: presets table");
  } finally {
    globalThis.localStorage = previousStorage;
  }

  const lensState = {
    enabled: false,
    xKey: "__tick__",
    yKey: "",
    y2Key: "",
    presetId: "custom",
    presets: {
      default: { enabled: false, xKey: "__tick__", yKey: "", y2Key: "" },
      demo: { enabled: true, xKey: "tick_id", yKey: "E", y2Key: "v" },
    },
  };

  const s0 = applyLensPresetSelectionState({ lensState, id: "demo" });
  assert(s0.ok && lensState.presetId === "demo", "apply preset: presetId");
  assert(lensState.enabled === true && lensState.yKey === "E", "apply preset: field update");
  const s1 = applyLensPresetSelectionState({ lensState, id: "custom" });
  assert(s1.ok && lensState.presetId === "custom", "apply preset: custom");
  const s2 = applyLensPresetSelectionState({ lensState, id: "missing" });
  assert(!s2.ok, "apply preset: missing");

  const w0 = saveLensPresetToState({
    lensState,
    presetName: "alpha",
    currentPreset: { enabled: true, xKey: "__index__", yKey: "x", y2Key: "" },
  });
  assert(w0.ok && w0.presetId === "alpha", "save preset: explicit name");
  lensState.presetId = "alpha";
  const d0 = deleteLensPresetFromState({ lensState });
  assert(d0.ok && d0.presetId === "alpha", "delete preset: success");
  lensState.presetId = "default";
  const d1 = deleteLensPresetFromState({ lensState });
  assert(!d1.ok, "delete preset: protected");

  const initLensState = {
    enabled: true,
    presetId: "demo",
  };
  const initLensEnable = { checked: false };
  const initLensSelect = { value: "" };
  const initLensName = { value: "" };
  let initUpdateCalled = 0;
  let initSyncCalled = 0;
  let initRefreshCalled = 0;
  initializeObservationLensUi({
    lensState: initLensState,
    observation: { channels: [], row: [] },
    lensEnableToggle: initLensEnable,
    lensPresetSelect: initLensSelect,
    lensPresetNameInput: initLensName,
    updateLensSelectors: () => {
      initUpdateCalled += 1;
    },
    syncLensConfigFromDom: () => {
      initSyncCalled += 1;
    },
    refreshLensPresetSelect: () => {
      initRefreshCalled += 1;
    },
  });
  assert(initLensEnable.checked === true, "init lens ui: enable toggle");
  assert(initLensSelect.value === "demo", "init lens ui: preset select");
  assert(initLensName.value === "demo", "init lens ui: preset name");
  assert(initUpdateCalled === 1, "init lens ui: update selectors");
  assert(initSyncCalled === 1, "init lens ui: sync config");
  assert(initRefreshCalled === 1, "init lens ui: refresh select");
  initializeObservationLensUi({
    lensState: { enabled: false, presetId: "custom" },
    lensPresetNameInput: initLensName,
  });
  assert(initLensName.value === "", "init lens ui: custom clears name");
  const wasmControls = {
    enabledToggle: { checked: false },
    langModeSelect: { value: "" },
    sampleSelect: { value: "" },
    inputEnabledToggle: { checked: false },
    fpsInput: { value: "" },
    dtMaxInput: { value: "" },
    fixedDtEnabledToggle: { checked: false },
    fixedDtValueInput: { value: "" },
    patchModeToggle: { checked: false },
    keyPresetSelect: { value: "" },
    keyUpInput: { value: "" },
    keyLeftInput: { value: "" },
    keyDownInput: { value: "" },
    keyRightInput: { value: "" },
    schemaMapInput: { value: "" },
    fixed64MapInput: { value: "" },
    lensEnableToggle: { checked: false },
    schemaPresetSelect: { value: "" },
    schemaPresetNameInput: { value: "" },
    lensPresetSelect: { value: "" },
    lensPresetNameInput: { value: "" },
  };
  syncWasmSettingsControlsFromState({
    wasmState: {
      enabled: true,
      langMode: "strict",
      sampleId: "calculus_ascii",
      inputEnabled: true,
      fpsLimit: 60,
      dtMax: 0.2,
      fixedDtEnabled: true,
      fixedDtValue: 0.0167,
      patchMode: true,
      keyPresetId: "wasd_arrows",
      keyMapRaw: { up: "w", left: "a", down: "s", right: "d" },
      schemaMapRaw: "x=y",
      fixed64MapRaw: "a=b",
      schemaPresetId: "presetA",
      lens: { enabled: true, presetId: "lensA" },
    },
    ...wasmControls,
  });
  assert(wasmControls.enabledToggle.checked === true, "sync wasm controls: enabled");
  assert(wasmControls.langModeSelect.value === "strict", "sync wasm controls: lang mode");
  assert(wasmControls.sampleSelect.value === "calculus_ascii", "sync wasm controls: sample");
  assert(wasmControls.inputEnabledToggle.checked === true, "sync wasm controls: input enabled");
  assert(wasmControls.fpsInput.value === 60, "sync wasm controls: fps");
  assert(wasmControls.dtMaxInput.value === 0.2, "sync wasm controls: dt max");
  assert(wasmControls.fixedDtEnabledToggle.checked === true, "sync wasm controls: fixed enabled");
  assert(wasmControls.fixedDtValueInput.value === 0.0167, "sync wasm controls: fixed value");
  assert(wasmControls.patchModeToggle.checked === true, "sync wasm controls: patch mode");
  assert(wasmControls.keyPresetSelect.value === "wasd_arrows", "sync wasm controls: key preset");
  assert(wasmControls.keyUpInput.value === "w", "sync wasm controls: key up");
  assert(wasmControls.schemaMapInput.value === "x=y", "sync wasm controls: schema map");
  assert(wasmControls.fixed64MapInput.value === "a=b", "sync wasm controls: fixed64 map");
  assert(wasmControls.lensEnableToggle.checked === true, "sync wasm controls: lens enabled");
  assert(wasmControls.schemaPresetSelect.value === "presetA", "sync wasm controls: schema preset");
  assert(
    wasmControls.schemaPresetNameInput.value === "presetA",
    "sync wasm controls: schema preset name",
  );
  assert(wasmControls.lensPresetSelect.value === "lensA", "sync wasm controls: lens preset");
  assert(wasmControls.lensPresetNameInput.value === "lensA", "sync wasm controls: lens preset name");
  syncWasmSettingsControlsFromState({
    wasmState: { schemaPresetId: "custom", lens: { presetId: "custom" } },
    schemaPresetNameInput: wasmControls.schemaPresetNameInput,
    lensPresetNameInput: wasmControls.lensPresetNameInput,
  });
  assert(wasmControls.schemaPresetNameInput.value === "", "sync wasm controls: schema custom clears");
  assert(wasmControls.lensPresetNameInput.value === "", "sync wasm controls: lens custom clears");

  const canvas = createFakeCanvas();
  const viewState = {
    autoFit: true,
    zoom: 1,
    panPx: 0,
    panPy: 0,
    dragging: false,
    lastX: 0,
    lastY: 0,
  };
  let renderCount = 0;
  bindSpace2dCanvasPanZoom({
    canvas,
    viewState,
    hasSpace2d: () => true,
    setAutoFit: (enabled) => {
      viewState.autoFit = Boolean(enabled);
    },
    onRender: () => {
      renderCount += 1;
    },
  });

  let prevented = false;
  canvas.emit("wheel", {
    deltaY: -1,
    preventDefault() {
      prevented = true;
    },
  });
  assert(prevented, "space2d: wheel preventDefault");
  assert(viewState.autoFit === false, "space2d: wheel clears auto-fit");
  assert(viewState.zoom > 1, "space2d: zoom-in");
  assert(renderCount >= 1, "space2d: wheel render");

  canvas.emit("pointerdown", { button: 0, clientX: 10, clientY: 20, pointerId: 1 });
  assert(viewState.dragging === true, "space2d: pointerdown drag start");
  canvas.emit("pointermove", { clientX: 18, clientY: 29 });
  assert(viewState.panPx === 8 && viewState.panPy === 9, "space2d: pan delta");
  canvas.emit("pointerup", {});
  assert(viewState.dragging === false, "space2d: pointerup drag stop");

  const worldCanvas = createFakeCanvas();
  const worldState = {
    auto: true,
    range: null,
    panX: 0,
    panY: 0,
    zoom: 1,
    dragging: false,
    lastClientX: 0,
    lastClientY: 0,
  };
  let manualSyncCount = 0;
  let worldRenderCount = 0;
  bindSpace2dCanvasWorldInteractions({
    canvas: worldCanvas,
    viewState: worldState,
    hasContent: () => true,
    computeBaseRange: () => ({ xMin: 0, xMax: 10, yMin: 0, yMax: 10 }),
    getRenderMeta: () => ({
      range: { xMin: 0, xMax: 10, yMin: 0, yMax: 10 },
      width: 100,
      height: 100,
      pad: 10,
    }),
    onManualViewEnsured: () => {
      manualSyncCount += 1;
    },
    onViewChanged: () => {
      worldRenderCount += 1;
    },
  });
  assert(worldCanvas.dataset.space2dInteractionBound === "1", "world bind: mark dataset");
  worldCanvas.emit("pointerdown", { button: 0, pointerId: 1, clientX: 50, clientY: 50 });
  assert(worldState.auto === false, "world bind: manual mode on pointerdown");
  assert(manualSyncCount >= 1, "world bind: manual sync callback");
  worldCanvas.emit("pointermove", { pointerId: 1, clientX: 60, clientY: 45 });
  assert(worldState.panX !== 0 || worldState.panY !== 0, "world bind: drag updates pan");
  worldCanvas.emit("wheel", {
    deltaY: -1,
    preventDefault() {},
  });
  assert(worldState.zoom > 1, "world bind: wheel zoom-in");
  worldCanvas.emit("pointerdown", { button: 0, pointerId: 2, clientX: 70, clientY: 50 });
  const zoomBeforePinch = worldState.zoom;
  worldCanvas.emit("pointermove", { pointerId: 2, clientX: 80, clientY: 50 });
  assert(worldState.zoom !== zoomBeforePinch, "world bind: pinch updates zoom");
  worldCanvas.emit("pointerup", { pointerId: 1 });
  worldCanvas.emit("pointerup", { pointerId: 2 });
  assert(worldRenderCount >= 3, "world bind: render callback count");

  const lensSelState = {
    channelSig: "",
    xKey: "__tick__",
    yKey: "",
    y2Key: "",
  };
  const xSelect = createFakeSelect();
  const ySelect = createFakeSelect();
  const y2Select = createFakeSelect();
  let syncedCount = 0;
  const changed1 = updateLensSelectorsFromObservation({
    observation: {
      channels: [{ key: "tick_id" }, { key: "energy" }],
    },
    lensState: lensSelState,
    xSelect,
    ySelect,
    y2Select,
    onSynced: () => {
      syncedCount += 1;
    },
  });
  assert(changed1 === true, "lens selectors: first update");
  assert(xSelect.options.length === 4, "lens selectors: x options");
  assert(ySelect.options.length === 2, "lens selectors: y options");
  assert(y2Select.options.length === 3, "lens selectors: y2 options");
  assert(ySelect.value === "tick_id", "lens selectors: y default first channel");
  const changed2 = updateLensSelectorsFromObservation({
    observation: {
      channels: [{ key: "tick_id" }, { key: "energy" }],
    },
    lensState: lensSelState,
    xSelect,
    ySelect,
    y2Select,
    onSynced: () => {
      syncedCount += 1;
    },
  });
  assert(changed2 === false, "lens selectors: unchanged signature skip");
  const changed3 = updateLensSelectorsFromObservation({
    observation: {
      channels: [{ key: "tick_id" }, { key: "energy" }, { key: "speed" }],
    },
    lensState: lensSelState,
    xSelect,
    ySelect,
    y2Select,
    onSynced: () => {
      syncedCount += 1;
    },
  });
  assert(changed3 === true, "lens selectors: signature change refresh");
  assert(syncedCount === 2, "lens selectors: sync callback count");
  const textTarget = { textContent: "" };
  const valueTarget = { value: "" };
  const rendered = renderObservationChannelList({
    element: textTarget,
    observation: {
      channels: [{ key: "tick_id", dtype: "fixed64", role: "state" }],
      row: [1.25],
    },
    target: "text",
  });
  assert(rendered === true, "channel list render: has rows");
  assert(textTarget.textContent.includes("tick_id"), "channel list render: text target");
  const renderedEmpty = renderObservationChannelList({
    element: valueTarget,
    observation: { channels: [], row: [] },
    target: "value",
  });
  assert(renderedEmpty === false, "channel list render: empty");
  assert(valueTarget.value === "-", "channel list render: empty marker");
  const resourceLines = collectStateResourceLines(
    {
      resources: {
        fixed64: { b: 2, a: 1 },
        value: { x: "hello" },
      },
    },
    { showFixed: true, showValue: true },
  );
  assert(resourceLines[0] === "fixed64 a = 1", "resource lines: fixed sorted");
  assert(resourceLines[2] === "value x = hello", "resource lines: value line");
  const kvRows = buildTagValueRowsFromStore({ b: 2, a: 1 });
  assert(kvRows.length === 2 && kvRows[0].tag === "a", "key-value rows: sorted");
  const kvTable = buildTagValueTableFromStore({
    store: { z: 9, x: 7 },
    source: "fixed64",
  });
  assert(kvTable.schema === "seamgrim.table.v0", "key-value table: schema");
  assert(kvTable.rows[0].tag === "x", "key-value table: sorted rows");
  assert(kvTable.meta?.source === "fixed64", "key-value table: source");
  let applyMode = "";
  dispatchWasmStateApply({
    stateJson: { patch: [{ op: "set_resource_value" }] },
    patchMode: true,
    onPatch: () => {
      applyMode = "patch";
    },
    onFull: () => {
      applyMode = "full";
    },
  });
  assert(applyMode === "patch", "state apply dispatch: patch");
  dispatchWasmStateApply({
    stateJson: { patch: [{ op: "set_resource_value" }] },
    patchMode: false,
    onPatch: () => {
      applyMode = "patch";
    },
    onFull: () => {
      applyMode = "full";
    },
  });
  assert(applyMode === "full", "state apply dispatch: full");
  let sourcedPatch = "";
  dispatchWasmStateApplyWithSource({
    stateJson: { patch: [{ op: "set_resource_json" }] },
    sourceText: "SOURCE:PATCH",
    patchMode: true,
    onPatchWithSource: (_next, source) => {
      sourcedPatch = source;
    },
  });
  assert(sourcedPatch === "SOURCE:PATCH", "state apply with source: patch callback source");
  let sourcedFull = "";
  dispatchWasmStateApplyWithSource({
    stateJson: { patch: [] },
    sourceText: "SOURCE:FULL",
    patchMode: false,
    onFullWithSource: (_next, source) => {
      sourcedFull = source;
    },
  });
  assert(sourcedFull === "SOURCE:FULL", "state apply with source: full callback source");
  let steppedWithSource = "";
  const steppedDispatchedDirect = stepWasmClientAndDispatchState({
    client: fakeClient,
    input: { keys: 2, lastKey: "z", px: 1, py: 1, dt: 0.1 },
    sourceText: "SOURCE:DIRECT",
    patchMode: false,
    onFullWithSource: (_next, source) => {
      steppedWithSource = source;
    },
  });
  assert(steppedDispatchedDirect.state?.schema === "seamgrim.state.v0", "step+dispatch direct: state");
  assert(steppedWithSource === "SOURCE:DIRECT", "step+dispatch direct: source callback");
  let timedSource = "";
  const timedStepped = stepWasmClientWithTimingAndDispatch({
    client: fakeClient,
    nowMs: 1200,
    lastTickMs: 1000,
    fixedDtEnabled: true,
    fixedDtValue: 0.0333,
    dtMin: 0.01,
    dtMax: 0.2,
    inputEnabled: true,
    keys: "4.9",
    lastKey: "ArrowLeft",
    px: "12.7",
    py: "-2.2",
    clearLastKeyWhenFixedDt: true,
    sourceText: "SOURCE:TIMED",
    patchMode: false,
    onFullWithSource: (_next, source) => {
      timedSource = source;
    },
  });
  assert(timedStepped.nextTickMs === 1200, "timed step: next tick");
  assert(Math.abs(timedStepped.dtSec - 0.0333) < 1e-9, "timed step: fixed dt");
  assert(timedStepped.input.keys === 4.9, "timed step: input keys");
  assert(timedStepped.input.px === 13 && timedStepped.input.py === -2, "timed step: rounded pointer");
  assert(timedStepped.nextLastKey === "", "timed step: clear last key on fixed dt");
  assert(timedSource === "SOURCE:TIMED", "timed step: source-aware callback");
  const timedDisabled = stepWasmClientWithTimingAndDispatch({
    client: fakeClient,
    nowMs: 900,
    lastTickMs: 500,
    inputEnabled: false,
    keys: 7,
    lastKey: "x",
    px: 99,
    py: 88,
    sourceText: "SOURCE:DISABLED",
    patchMode: false,
  });
  assert(timedDisabled.input.keys === 0, "timed step: disabled keys");
  assert(timedDisabled.input.lastKey === "", "timed step: disabled last key");
  assert(timedDisabled.input.px === 0 && timedDisabled.input.py === 0, "timed step: disabled pointer");
  assert(timedDisabled.nextLastKey === "", "timed step: disabled next last key");
  const parsedPatch0 = parsePatchJsonObject('{"schema":"seamgrim.graph.v0"}');
  assert(parsedPatch0?.obj?.schema === "seamgrim.graph.v0", "patch parse: valid json");
  assert(parsePatchJsonObject("{invalid") === null, "patch parse: invalid json");
  assert(inferPatchSchemaFromJsonObject({ schema: "seamgrim.text.v0" }) === "seamgrim.text.v0", "patch schema infer: explicit");
  assert(
    inferPatchSchemaFromJsonObject({ columns: [], rows: [] }) === "seamgrim.table.v0",
    "patch schema infer: table fallback",
  );
  assert(isPatchScalarValue("1") === true && isPatchScalarValue(1) === true, "patch scalar: allowed");
  assert(isPatchScalarValue(false) === false, "patch scalar: denied");
  const componentStore = {};
  const upsertedComponent = upsertPatchComponentStoreEntry({
    componentStore,
    op: { op: "set_component_json", entity: 1, tag: 2 },
    raw: "{\"x\":1}",
    obj: { x: 1 },
  });
  assert(upsertedComponent.stored === true, "patch component upsert: stored");
  assert(componentStore["1:2"]?.raw === "{\"x\":1}", "patch component upsert: raw");
  const removedComponent = removePatchComponentStoreEntry({
    componentStore,
    op: { op: "remove_component", entity: 1, tag: 2 },
  });
  assert(removedComponent.removed === true, "patch component remove: removed");
  assert(removedComponent.entry?.raw === "{\"x\":1}", "patch component remove: entry");
  const fixedStore = {};
  const scalarUpsert0 = upsertPatchScalarStoreEntry({
    store: fixedStore,
    tag: "mass",
    value: 12.5,
    asString: true,
  });
  assert(scalarUpsert0.updated === true && fixedStore.mass === "12.5", "patch scalar upsert: asString");
  const scalarUpsert1 = upsertPatchScalarStoreEntry({
    store: fixedStore,
    tag: "name",
    value: "alpha",
    asString: false,
  });
  assert(scalarUpsert1.updated === true && fixedStore.name === "alpha", "patch scalar upsert: raw");
  const scalarUpsert2 = upsertPatchScalarStoreEntry({
    store: fixedStore,
    tag: "invalid",
    value: false,
    asString: false,
  });
  assert(scalarUpsert2.updated === false, "patch scalar upsert: invalid value");
  const opKinds = [];
  const patchFlags = processPatchOperations({
    patch: [
      { op: "set_resource_json", value: "{\"ok\":true}" },
      { op: "set_resource_fixed64", tag: "x", value: "1.25" },
      { op: "set_resource_value", tag: "name", value: "alpha" },
      { op: "remove_component", entity: 1, tag: 2 },
      { op: "unknown_op" },
    ],
    onJsonOp: (op) => {
      opKinds.push(op.op);
      return { changed: true };
    },
    onFixed64Op: (op) => {
      opKinds.push(op.op);
      return { changed: false, fixed64Changed: true, requireFull: false };
    },
    onValueOp: (op) => {
      opKinds.push(op.op);
      return { valueChanged: true, requireFull: true };
    },
    onRemoveComponentOp: (op) => {
      opKinds.push(op.op);
      return true;
    },
  });
  assert(opKinds.length === 4, "patch ops: dispatch count");
  assert(patchFlags.changed === true, "patch ops: changed flag");
  assert(patchFlags.fixed64Changed === true, "patch ops: fixed64 flag");
  assert(patchFlags.valueChanged === true, "patch ops: value flag");
  assert(patchFlags.requireFull === true, "patch ops: requireFull flag");
  let effectRenderCount = 0;
  let channelsChangedCount = 0;
  applyObservationRenderEffects({
    changed: false,
    forceRender: true,
    channelsChanged: true,
    onRender: () => {
      effectRenderCount += 1;
    },
    onChannelsChanged: () => {
      channelsChangedCount += 1;
    },
  });
  assert(effectRenderCount === 1, "render effects: forced render");
  assert(channelsChangedCount === 1, "render effects: channel change");

  const normPoints = normalizeRenderPoints([[0, 1], { x: 2, y: 3 }, { x: "bad", y: 1 }]);
  assert(normPoints.length === 2, "render points normalize: valid count");
  assert(isGraphEmptyForRender({ series: [] }) === true, "render graph empty: no series");
  assert(
    isGraphEmptyForRender({ series: [{ points: [{ x: 1, y: 2 }] }] }) === false,
    "render graph empty: has points",
  );
  const renderCanvas = createFakeCanvas2d();
  const graphRendered = renderGraphCanvas2d({
    canvas: renderCanvas,
    graph: { series: [{ points: [{ x: 0, y: 1 }, { x: 1, y: 2 }] }] },
    showGrid: true,
    showAxis: true,
  });
  assert(graphRendered === true, "render graph canvas: rendered");
  const spaceRendered = renderSpace2dCanvas2d({
    canvas: renderCanvas,
    space2d: {
      points: [{ x: 0, y: 0 }],
      shapes: [{ kind: "line", x1: 0, y1: 0, x2: 1, y2: 1 }],
    },
    viewState: { autoFit: true, zoom: 1, panPx: 0, panPy: 0 },
    showGrid: true,
    showAxis: true,
  });
  assert(spaceRendered === true, "render space2d canvas: rendered");
  const primitiveLineToBefore = renderCanvas.calls.lineTo;
  const primitiveFillBefore = renderCanvas.calls.fill;
  const primitiveRendered = renderSpace2dCanvas2d({
    canvas: renderCanvas,
    space2d: {
      points: [{ x: 0, y: 0 }],
      drawlist: [
        { kind: "line", x1: -1, y1: -1, x2: 1, y2: 1 },
        { kind: "circle", x: 0, y: 0, r: 0.5 },
        { kind: "arrow", x1: -1, y1: 1, x2: 1, y2: 1 },
        { kind: "text", x: 0, y: 0.8, text: "TXT" },
        {
          kind: "curve",
          points: [
            { x: -1, y: -0.5 },
            { x: 0, y: 0.2 },
            { x: 1, y: 0.6 },
          ],
        },
        {
          kind: "fill",
          points: [
            { x: -0.8, y: -0.2 },
            { x: 0, y: -0.6 },
            { x: 0.8, y: -0.2 },
          ],
          fill: "#44cc88aa",
        },
      ],
    },
    primitiveSource: "drawlist",
    viewState: { autoFit: true, zoom: 1, panPx: 0, panPy: 0 },
    showGrid: false,
    showAxis: false,
  });
  assert(primitiveRendered === true, "render primitives minset: rendered");
  assert(renderCanvas.calls.arc > 0, "render primitives minset: circle");
  assert(renderCanvas.calls.fillText.includes("TXT"), "render primitives minset: text");
  assert(renderCanvas.calls.lineTo > primitiveLineToBefore, "render primitives minset: line/arrow/curve/fill");
  assert(renderCanvas.calls.fill > primitiveFillBefore, "render primitives minset: fill");
  const drawlistTextStart = renderCanvas.calls.fillText.length;
  const drawlistSourceRendered = renderSpace2dCanvas2d({
    canvas: renderCanvas,
    space2d: {
      points: [{ x: 0, y: 0 }],
      shapes: [{ kind: "text", x: 0, y: 0, text: "SHAPE_TXT" }],
      drawlist: [{ kind: "text", x: 0, y: 0, text: "DRAW_TXT" }],
    },
    primitiveSource: "drawlist",
    viewState: { autoFit: true, zoom: 1, panPx: 0, panPy: 0 },
  });
  assert(drawlistSourceRendered === true, "render source mode: drawlist");
  const drawlistTexts = renderCanvas.calls.fillText.slice(drawlistTextStart);
  assert(drawlistTexts.includes("DRAW_TXT"), "render source mode: drawlist text rendered");
  assert(!drawlistTexts.includes("SHAPE_TXT"), "render source mode: drawlist ignores shapes");
  const shapeTextStart = renderCanvas.calls.fillText.length;
  const shapeSourceRendered = renderSpace2dCanvas2d({
    canvas: renderCanvas,
    space2d: {
      points: [{ x: 0, y: 0 }],
      shapes: [{ kind: "text", x: 0, y: 0, text: "SHAPE_TXT" }],
      drawlist: [{ kind: "text", x: 0, y: 0, text: "DRAW_TXT" }],
    },
    primitiveSource: "shapes",
    viewState: { autoFit: true, zoom: 1, panPx: 0, panPy: 0 },
  });
  assert(shapeSourceRendered === true, "render source mode: shapes");
  const shapeTexts = renderCanvas.calls.fillText.slice(shapeTextStart);
  assert(shapeTexts.includes("SHAPE_TXT"), "render source mode: shape text rendered");
  assert(!shapeTexts.includes("DRAW_TXT"), "render source mode: shapes ignores drawlist");
  const route0 = renderGraphOrSpace2dCanvas({
    canvas: renderCanvas,
    graph: { series: [] },
    space2d: { points: [{ x: 0, y: 0 }] },
    graphPreference: "prefer_non_empty_graph",
  });
  assert(route0 === "space2d", "render route: prefer non-empty graph");
  const route1 = renderGraphOrSpace2dCanvas({
    canvas: renderCanvas,
    graph: { series: [] },
    space2d: { points: [{ x: 0, y: 0 }] },
    graphPreference: "prefer_graph",
  });
  assert(route1 === "graph", "render route: prefer graph");
  const routeTextStart = renderCanvas.calls.fillText.length;
  const route2 = renderGraphOrSpace2dCanvas({
    canvas: renderCanvas,
    graph: { series: [] },
    space2d: {
      points: [{ x: 0, y: 0 }],
      shapes: [{ kind: "text", x: 0, y: 0, text: "ROUTE_SHAPE" }],
      drawlist: [{ kind: "text", x: 0, y: 0, text: "ROUTE_DRAW" }],
    },
    graphPreference: "prefer_non_empty_graph",
    spacePrimitiveSource: "drawlist",
  });
  assert(route2 === "space2d", "render route: space primitive source");
  const routeTexts = renderCanvas.calls.fillText.slice(routeTextStart);
  assert(routeTexts.includes("ROUTE_DRAW"), "render route: drawlist passthrough");
  assert(!routeTexts.includes("ROUTE_SHAPE"), "render route: shape bypass");

  const graphLensState = {
    enabled: true,
    xKey: "__tick__",
    yKey: "energy",
    y2Key: "speed",
    maxPoints: 2,
    timeline: [],
  };
  pushObservationLensSample({
    lensState: graphLensState,
    observation: {
      channels: [{ key: "energy" }, { key: "speed" }, { key: "label" }],
      row: [1.5, "2.5", "skip"],
    },
    tickId: 10,
  });
  pushObservationLensSample({
    lensState: graphLensState,
    observation: {
      channels: [{ key: "energy" }, { key: "speed" }],
      row: [3.0, 4.0],
    },
    tickId: 11,
  });
  pushObservationLensSample({
    lensState: graphLensState,
    observation: {
      channels: [{ key: "energy" }, { key: "speed" }],
      row: [5.0, 6.0],
    },
    tickId: 12,
  });
  assert(graphLensState.timeline.length === 2, "lens sample: maxPoints trim");
  assert(graphLensState.timeline[0].__tick__ === 11, "lens sample: keeps newest");
  assert(graphLensState.timeline[0].__index__ === 0, "lens sample: reindex");

  const graph = buildObservationLensGraph({
    lensState: graphLensState,
    includeSample: true,
    metaUpdate: "replace",
  });
  assert(graph?.schema === "seamgrim.graph.v0", "lens graph: schema");
  assert(Array.isArray(graph?.series) && graph.series.length === 2, "lens graph: dual series");
  assert(graph?.sample?.var === "__tick__", "lens graph: sample var");
  assert(graph?.meta?.update === "replace", "lens graph: meta update");
  const syncLensState = {
    enabled: true,
    xKey: "__tick__",
    yKey: "energy",
    y2Key: "",
    timeline: [],
    lastFrameToken: "",
  };
  const syncStateJson = { tick_id: 7, frame_id: 3, state_hash: "h1" };
  const syncObservation = {
    channels: [{ key: "energy" }],
    row: [8],
  };
  const sync0 = syncObservationLensFrame({
    lensState: syncLensState,
    observation: syncObservation,
    stateJson: syncStateJson,
    graphOptions: { includeSample: false, source: "observation-lens" },
  });
  assert(sync0.samplePushed === true, "lens sync: first push");
  assert(syncLensState.timeline.length === 1, "lens sync: timeline append");
  const sync1 = syncObservationLensFrame({
    lensState: syncLensState,
    observation: syncObservation,
    stateJson: syncStateJson,
    graphOptions: { includeSample: false, source: "observation-lens" },
  });
  assert(sync1.samplePushed === false, "lens sync: same frame skip");
  assert(syncLensState.timeline.length === 1, "lens sync: no duplicate sample");
  const sync2 = syncObservationLensFrame({
    lensState: syncLensState,
    observation: syncObservation,
    stateJson: { tick_id: 8, frame_id: 4, state_hash: "h2" },
    graphOptions: { includeSample: false, source: "observation-lens" },
  });
  assert(sync2.samplePushed === true, "lens sync: new frame push");
  assert(syncLensState.timeline.length === 2, "lens sync: second sample append");
  assert(sync2.graph?.series?.length === 1, "lens sync: graph build");
  const composed = composeObservationRenderState({
    stateJson: {
      tick_id: 20,
      frame_id: 9,
      state_hash: "blake3:demo",
      channels: [{ key: "energy" }],
      row: [10],
    },
    observation: {
      channels: [{ key: "energy" }],
      row: [10],
    },
    lensState: {
      enabled: true,
      xKey: "__tick__",
      yKey: "energy",
      y2Key: "",
      timeline: [],
      maxPoints: 10,
      lastFrameToken: "",
    },
    graphOptions: {
      source: "observation-lens",
      includeSample: false,
    },
    patchCount: 1,
    tickTimeDigits: 2,
  });
  assert(Array.isArray(composed.statusLines) && composed.statusLines.length >= 6, "compose: status lines");
  assert(composed.channelCount === 1, "compose: channel count");
  assert(composed.samplePushed === true, "compose: sample pushed");
  assert(composed.lensGraph?.series?.length === 1, "compose: lens graph");
  graphLensState.enabled = false;
  const graphOff = buildObservationLensGraph({ lensState: graphLensState });
  assert(graphOff === null, "lens graph: disabled");

  console.log("seamgrim ui common ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
