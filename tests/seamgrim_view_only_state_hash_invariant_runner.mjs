import fs from "node:fs";
import path from "node:path";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function readText(filePath) {
  return fs.readFileSync(filePath, "utf-8");
}

function extractFunctionBody(text, signature) {
  const start = text.indexOf(signature);
  if (start < 0) return null;
  const braceStart = text.indexOf("{", start);
  if (braceStart < 0) return null;
  let depth = 0;
  for (let i = braceStart; i < text.length; i += 1) {
    const ch = text[i];
    if (ch === "{") depth += 1;
    if (ch === "}") {
      depth -= 1;
      if (depth === 0) {
        return text.slice(braceStart + 1, i);
      }
    }
  }
  return null;
}

function assertNoForbidden(label, text, forbiddenTokens) {
  const hits = [];
  for (const token of forbiddenTokens) {
    if (text.includes(token)) hits.push(token);
  }
  assert(hits.length === 0, `${label}:forbidden:${hits.join(",")}`);
}

const root = process.cwd();
const runJsPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
const indexHtmlPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/index.html");
const runJs = readText(runJsPath);
const indexHtml = readText(indexHtmlPath);

const forbiddenStateMutationTokens = [
  "this.setHash(",
  "applyWasmLogicAndDispatchState(",
  "stepWasmClientParsed(",
  "this.restart(",
  "this.reportRuntimeFailure(",
];

const requiredViewOnlyControls = [
  'id="btn-dock-zoom-in"',
  'id="btn-dock-zoom-out"',
  'id="btn-dock-pan-left"',
  'id="btn-dock-pan-right"',
  'id="btn-dock-pan-up"',
  'id="btn-dock-pan-down"',
  'id="btn-bogae-fullscreen"',
  'id="btn-overlay-toggle"',
  'id="run-tab-btn-graph"',
  'id="run-tab-btn-mirror"',
];
for (const token of requiredViewOnlyControls) {
  assert(indexHtml.includes(token), `index_html_missing:${token}`);
}

const overlayHandlerBody = extractFunctionBody(
  runJs,
  'this.root.querySelector("#btn-overlay-toggle")?.addEventListener("click", () =>',
);
assert(overlayHandlerBody, "overlay_handler_missing");
assert(overlayHandlerBody.includes("this.switchRunTab(SUBPANEL_TAB.OVERLAY)"), "overlay_handler_overlay_tab_missing");
assertNoForbidden("overlay_handler", overlayHandlerBody, forbiddenStateMutationTokens);

const bindDockBody = extractFunctionBody(runJs, "bindViewDockUi() {");
assert(bindDockBody, "bind_view_dock_ui_missing");
[
  "#btn-bogae-fullscreen",
  "#btn-dock-pan-left",
  "#btn-dock-pan-right",
  "#btn-dock-pan-up",
  "#btn-dock-pan-down",
  "#btn-dock-zoom-in",
  "#btn-dock-zoom-out",
  "#btn-dock-space-autoscale",
  "#btn-dock-graph-autoscale",
  "#btn-graph-autoscale",
].forEach((token) => {
  assert(bindDockBody.includes(token), `bind_view_dock_ui_handler_missing:${token}`);
});
assertNoForbidden("bind_view_dock_ui", bindDockBody, forbiddenStateMutationTokens);

const switchRunTabBody = extractFunctionBody(runJs, "switchRunTab(tabId) {");
assert(switchRunTabBody, "switch_run_tab_missing");
assertNoForbidden("switch_run_tab", switchRunTabBody, forbiddenStateMutationTokens);

const panDockTargetBody = extractFunctionBody(runJs, "panDockTarget(dx, dy) {");
assert(panDockTargetBody, "pan_dock_target_missing");
assertNoForbidden("pan_dock_target", panDockTargetBody, forbiddenStateMutationTokens);

const zoomDockTargetBody = extractFunctionBody(runJs, "zoomDockTarget(factor) {");
assert(zoomDockTargetBody, "zoom_dock_target_missing");
assertNoForbidden("zoom_dock_target", zoomDockTargetBody, forbiddenStateMutationTokens);

const toggleFullscreenBody = extractFunctionBody(runJs, "async toggleBogaeFullscreen() {");
assert(toggleFullscreenBody, "toggle_fullscreen_missing");
assertNoForbidden("toggle_fullscreen", toggleFullscreenBody, forbiddenStateMutationTokens);

console.log("seamgrim view-only state_hash invariant runner ok");
