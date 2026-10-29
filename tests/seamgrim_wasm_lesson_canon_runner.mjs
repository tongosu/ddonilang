import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");

async function main() {
  const runtimeUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "index.js"),
  ).href;
  const editorUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "screens", "editor.js"),
  ).href;
  const runtime = await import(runtimeUrl);
  const editor = await import(editorUrl);
  if (typeof runtime.createLessonCanonHydrator !== "function") {
    throw new Error("createLessonCanonHydrator export 누락");
  }
  if (typeof runtime.createWasmCanon !== "function") {
    throw new Error("createWasmCanon export 누락");
  }
  if (typeof runtime.summarizeFlatPlan !== "function") {
    throw new Error("summarizeFlatPlan export 누락");
  }
  if (typeof runtime.buildFlatPlanView !== "function") {
    throw new Error("buildFlatPlanView export 누락");
  }
  if (typeof editor.findFlatInstanceSelectionRange !== "function") {
    throw new Error("findFlatInstanceSelectionRange export 누락");
  }
  if (typeof editor.findFlatLinkSelectionRange !== "function") {
    throw new Error("findFlatLinkSelectionRange export 누락");
  }

  const wasmBytes = await fs.readFile(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool_bg.wasm"),
  );
  const ddnText = await fs.readFile(
    path.join(rootDir, "pack", "seamgrim_wasm_canon_contract_v1", "fixtures", "maegim_input.ddn"),
    "utf8",
  );
  const guseongText = await fs.readFile(
    path.join(rootDir, "pack", "seamgrim_wasm_canon_contract_v1", "fixtures", "guseong_input.ddn"),
    "utf8",
  );
  const alrimText = await fs.readFile(
    path.join(rootDir, "pack", "seamgrim_wasm_canon_contract_v1", "fixtures", "alrim_input.ddn"),
    "utf8",
  );

  const hydrator = runtime.createLessonCanonHydrator({
    cacheBust: 0,
    initInput: wasmBytes,
  });
  if (typeof hydrator.getRuntimeDiags !== "function") {
    throw new Error("createLessonCanonHydrator.getRuntimeDiags export 누락");
  }
  const canon = await runtime.createWasmCanon({
    cacheBust: 0,
    initInput: wasmBytes,
  });
  if (typeof canon.getLastBuildInfoDiag !== "function") {
    throw new Error("createWasmCanon.getLastBuildInfoDiag export 누락");
  }
  if (typeof canon.getLastInitDiag !== "function") {
    throw new Error("createWasmCanon.getLastInitDiag export 누락");
  }
  if (typeof canon.getLastPreprocessDiag !== "function") {
    throw new Error("createWasmCanon.getLastPreprocessDiag export 누락");
  }
  if (typeof canon.getLastCanonDiag !== "function") {
    throw new Error("createWasmCanon.getLastCanonDiag export 누락");
  }

  const hydrated = await hydrator.hydrateLessonCanon({
    id: "lesson-canon-smoke",
    title: "Lesson Canon Smoke",
    ddnText,
    maegimControlJson: "",
  });
  const parsed = JSON.parse(String(hydrated?.maegimControlJson ?? ""));
  if (String(parsed?.schema ?? "") !== "ddn.maegim_control_plan.v1") {
    throw new Error("hydrated maegim plan schema mismatch");
  }
  const controls = Array.isArray(parsed?.controls) ? parsed.controls : [];
  if (controls.length !== 2) {
    throw new Error(`hydrated maegim control count mismatch: ${controls.length}`);
  }
  if (!Array.isArray(hydrator.getCanonDiags?.()) || hydrator.getCanonDiags().length !== 0) {
    throw new Error("canon diags should be empty after successful maegim canon");
  }
  if (!Array.isArray(hydrator.getRuntimeDiags?.())) {
    throw new Error("runtime diags should be array");
  }

  const preserved = await hydrator.hydrateLessonCanon({
    id: "lesson-canon-preserve",
    title: "Lesson Canon Preserve",
    ddnText,
    maegimControlJson: '{"schema":"keep.me.v1"}',
  });
  if (String(preserved?.maegimControlJson ?? "") !== '{"schema":"keep.me.v1"}') {
    throw new Error("existing maegimControlJson should be preserved");
  }

  const flat = await hydrator.deriveFlatJson(guseongText);
  if (String(flat?.schema ?? "") !== "ddn.guseong_flatten_plan.v1") {
    throw new Error("hydrated flat plan schema mismatch");
  }
  const alrim = await canon.canonAlrimPlan(alrimText);
  if (String(alrim?.schema ?? "") !== "ddn.alrim_event_plan.v1") {
    throw new Error("alrim plan schema mismatch");
  }
  const handlers = Array.isArray(alrim?.handlers) ? alrim.handlers : [];
  if (handlers.length !== 3) {
    throw new Error(`alrim handler count mismatch: ${handlers.length}`);
  }
  if (String(handlers?.[1]?.scope ?? "") !== "root/seed:매틱") {
    throw new Error(`alrim handler scope mismatch: ${String(handlers?.[1]?.scope ?? "")}`);
  }
  const preprocessed = await canon.preprocessSource(alrimText);
  if (!String(preprocessed).trim()) {
    throw new Error("wasm canon preprocess output must not be empty");
  }
  if (canon.getLastPreprocessDiag() !== null) {
    throw new Error("successful wasm canon preprocess must clear preprocess diag");
  }
  const summary = runtime.summarizeFlatPlan(flat);
  if (!String(summary).includes("instance 2개")) {
    throw new Error(`flat summary mismatch: ${summary}`);
  }
  const view = runtime.buildFlatPlanView(flat);
  if (!Array.isArray(view?.instances) || view.instances.length !== 2) {
    throw new Error("flat view instances mismatch");
  }
  if (!Array.isArray(view?.links) || view.links.length !== 1) {
    throw new Error("flat view links mismatch");
  }
  if (String(view?.links?.[0]?.label ?? "") !== "b.꼭짓점 -> a.기준점") {
    throw new Error(`flat view link label mismatch: ${String(view?.links?.[0]?.label ?? "")}`);
  }
  const instanceRange = editor.findFlatInstanceSelectionRange(guseongText, "a");
  if (!instanceRange || guseongText.slice(instanceRange.start, instanceRange.end).trim() !== "a <- (L=1.0)인 진자_틀.") {
    throw new Error("flat instance selection range mismatch");
  }
  const linkRange = editor.findFlatLinkSelectionRange(guseongText, view.links[0]);
  if (!linkRange || guseongText.slice(linkRange.start, linkRange.end).trim() !== "a.기준점 <- b.꼭짓점.") {
    throw new Error("flat link selection range mismatch");
  }

  const failingHydrator = runtime.createLessonCanonHydrator({
    createCanon: async () => ({
      canonMaegimPlan() {
        throw new Error("maegim-test-fail");
      },
      canonFlatJson() {
        throw new Error("flat-test-fail");
      },
    }),
  });

  const maegimFailText = await failingHydrator.deriveMaegimControlJson("매틱:움직씨 = { x <- 1. }.");
  if (maegimFailText !== "") {
    throw new Error("maegim failure path must return empty string");
  }
  const maegimFailDiag = failingHydrator.getCanonDiags?.()?.[0] ?? null;
  if (String(maegimFailDiag?.code ?? "") !== "E_WASM_MAEGIM_PLAN_FALLBACK_FAILED") {
    throw new Error(`maegim failure diag mismatch: ${JSON.stringify(maegimFailDiag)}`);
  }

  const flatFail = await failingHydrator.deriveFlatJson("매틱:움직씨 = { x <- 1. }.", { quiet: true });
  if (flatFail !== null) {
    throw new Error("flat failure path must return null");
  }
  const flatFailDiag = failingHydrator.getCanonDiags?.()?.[0] ?? null;
  if (String(flatFailDiag?.code ?? "") !== "E_WASM_FLAT_PLAN_FALLBACK_FAILED") {
    throw new Error(`flat failure diag mismatch: ${JSON.stringify(flatFailDiag)}`);
  }

  const runtimeDiagHydrator = runtime.createLessonCanonHydrator({
    createCanon: async () => ({
      canonMaegimPlan() {
        throw new Error("runtime-diag-test-fail");
      },
      getLastInitDiag() {
        return {
          code: "E_WASM_CANON_MODULE_INIT_FAILED",
          message: "wasm canonical module 초기화에 실패했습니다.",
          detail: "init-diag-detail",
        };
      },
      getLastCanonDiag() {
        return {
          code: "E_WASM_CANON_JSON_PARSE_FAILED",
          message: "maegim canonical JSON 파싱에 실패했습니다.",
          detail: "runtime-diag-detail",
        };
      },
    }),
  });
  await runtimeDiagHydrator.deriveMaegimControlJson("매틱:움직씨 = { x <- 1. }.");
  const runtimeDiag = runtimeDiagHydrator.getCanonDiags?.()?.[0] ?? null;
  if (String(runtimeDiag?.code ?? "") !== "E_WASM_CANON_MODULE_INIT_FAILED") {
    throw new Error(`runtime diag passthrough mismatch: ${JSON.stringify(runtimeDiag)}`);
  }
  if (String(runtimeDiag?.detail ?? "") !== "init-diag-detail") {
    throw new Error(`runtime diag detail passthrough mismatch: ${JSON.stringify(runtimeDiag)}`);
  }
  const runtimeDiagRow = runtimeDiagHydrator.getRuntimeDiags?.()?.[0] ?? null;
  if (String(runtimeDiagRow?.code ?? "") !== "E_WASM_CANON_MODULE_INIT_FAILED") {
    throw new Error(`runtime diag snapshot mismatch: ${JSON.stringify(runtimeDiagRow)}`);
  }

  const createFailHydrator = runtime.createLessonCanonHydrator({
    createCanon: async () => {
      throw new Error("create-canon-fail");
    },
  });
  const createFailFlat = await createFailHydrator.deriveFlatJson("매틱:움직씨 = { x <- 1. }.", { quiet: true });
  if (createFailFlat !== null) {
    throw new Error("create-fail hydrator must return null for flat");
  }
  const createFailRuntimeDiag = createFailHydrator.getRuntimeDiags?.()?.[0] ?? null;
  if (String(createFailRuntimeDiag?.code ?? "") !== "E_WASM_CANON_RUNTIME_CREATE_FAILED") {
    throw new Error(`create-fail runtime diag mismatch: ${JSON.stringify(createFailRuntimeDiag)}`);
  }

  const loadFailCanon = await runtime.createWasmCanon({
    moduleFactory: async () => {
      throw new Error("module-load-test-fail");
    },
    cacheBust: 0,
  });
  let loadFailed = false;
  try {
    await loadFailCanon.canonFlatJson("매틱:움직씨 = { x <- 1. }.");
  } catch (_) {
    loadFailed = true;
  }
  if (!loadFailed) {
    throw new Error("wasm canon module-load failure path must throw");
  }
  if (String(loadFailCanon.getLastInitDiag?.()?.code ?? "") !== "E_WASM_CANON_MODULE_LOAD_FAILED") {
    throw new Error("wasm canon module-load diag mismatch");
  }

  const initFailCanon = await runtime.createWasmCanon({
    moduleFactory: async () => ({
      default() {
        throw new Error("module-init-test-fail");
      },
      wasm_canon_flat_json() {
        return "{}";
      },
    }),
    cacheBust: 0,
  });
  let initFailed = false;
  try {
    await initFailCanon.canonFlatJson("매틱:움직씨 = { x <- 1. }.");
  } catch (_) {
    initFailed = true;
  }
  if (!initFailed) {
    throw new Error("wasm canon module-init failure path must throw");
  }
  if (String(initFailCanon.getLastInitDiag?.()?.code ?? "") !== "E_WASM_CANON_MODULE_INIT_FAILED") {
    throw new Error("wasm canon module-init diag mismatch");
  }

  let flakyInitAttempts = 0;
  const flakyInitCanon = await runtime.createWasmCanon({
    moduleFactory: async () => ({
      default() {
        flakyInitAttempts += 1;
        if (flakyInitAttempts === 1) {
          throw new Error("module-init-flaky-fail-once");
        }
      },
      wasm_canon_flat_json() {
        return '{"schema":"ddn.guseong_flatten_plan.v1","instances":[],"links":[],"topo_order":[]}';
      },
    }),
    cacheBust: 0,
  });
  let flakyFailed = false;
  try {
    await flakyInitCanon.canonFlatJson("매틱:움직씨 = { x <- 1. }.");
  } catch (_) {
    flakyFailed = true;
  }
  if (!flakyFailed) {
    throw new Error("wasm canon flaky-init first call must fail");
  }
  if (String(flakyInitCanon.getLastInitDiag?.()?.code ?? "") !== "E_WASM_CANON_MODULE_INIT_FAILED") {
    throw new Error("wasm canon flaky-init first diag mismatch");
  }
  const flakyRetry = await flakyInitCanon.canonFlatJson("매틱:움직씨 = { x <- 1. }.");
  if (String(flakyRetry?.schema ?? "") !== "ddn.guseong_flatten_plan.v1") {
    throw new Error("wasm canon flaky-init retry parse mismatch");
  }
  if (flakyInitCanon.getLastInitDiag?.() !== null) {
    throw new Error("wasm canon flaky-init retry should clear init diag");
  }

  const buildInfoFailCanon = await runtime.createWasmCanon({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        throw new Error("build-info-test-fail");
      },
      wasm_canon_flat_json() {
        return "{}";
      },
    }),
    cacheBust: 0,
  });
  await buildInfoFailCanon.canonFlatJson("매틱:움직씨 = { x <- 1. }.");
  if (String(buildInfoFailCanon.getLastBuildInfoDiag?.()?.code ?? "") !== "E_WASM_CANON_BUILD_INFO_CALL_FAILED") {
    throw new Error("wasm canon build-info diag mismatch");
  }

  const preprocessMissingCanon = await runtime.createWasmCanon({
    moduleFactory: async () => ({
      default() {},
      wasm_canon_flat_json() {
        return "{}";
      },
    }),
    cacheBust: 0,
  });
  const passthrough = await preprocessMissingCanon.preprocessSource("x <- 1.");
  if (passthrough !== "x <- 1.") {
    throw new Error("wasm canon preprocess missing-api path must passthrough source");
  }
  if (
    String(preprocessMissingCanon.getLastPreprocessDiag?.()?.code ?? "") !==
    "E_WASM_CANON_PREPROCESS_API_MISSING"
  ) {
    throw new Error("wasm canon preprocess missing-api diag mismatch");
  }

  const preprocessFailCanon = await runtime.createWasmCanon({
    moduleFactory: async () => ({
      default() {},
      wasm_preprocess_source() {
        throw new Error("preprocess-test-fail");
      },
      wasm_canon_flat_json() {
        return "{}";
      },
    }),
    cacheBust: 0,
  });
  let preprocessFailed = false;
  try {
    await preprocessFailCanon.preprocessSource("x <- 1.");
  } catch (_) {
    preprocessFailed = true;
  }
  if (!preprocessFailed) {
    throw new Error("wasm canon preprocess failure path must throw");
  }
  if (
    String(preprocessFailCanon.getLastPreprocessDiag?.()?.code ?? "") !==
    "E_WASM_CANON_PREPROCESS_CALL_FAILED"
  ) {
    throw new Error("wasm canon preprocess failure diag mismatch");
  }

  console.log("seamgrim wasm lesson canon runner ok");
}

main().catch((error) => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
