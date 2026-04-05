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
  const canon = await runtime.createWasmCanon({
    cacheBust: 0,
    initInput: wasmBytes,
  });

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

  console.log("seamgrim wasm lesson canon runner ok");
}

main().catch((error) => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
