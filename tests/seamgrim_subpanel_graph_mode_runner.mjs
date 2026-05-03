import path from "node:path";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/subpanel_tab_policy.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const { SUBPANEL_TAB, resolveGraphTabMode, resolveSubpanelTabs } = mod;

  assert(typeof resolveSubpanelTabs === "function", "subpanel graph mode: resolveSubpanelTabs export");
  assert(typeof resolveGraphTabMode === "function", "subpanel graph mode: resolveGraphTabMode export");

  const expectedTabs = [SUBPANEL_TAB.MAEGIM, SUBPANEL_TAB.OUTPUT, SUBPANEL_TAB.MIRROR, SUBPANEL_TAB.GRAPH, SUBPANEL_TAB.OVERLAY];
  const simTabs = resolveSubpanelTabs("sim");
  const graphTabs = resolveSubpanelTabs("graph");
  const tableTabs = resolveSubpanelTabs("table");

  assert(JSON.stringify(simTabs) === JSON.stringify(expectedTabs), "subpanel graph mode: sim tabs fixed");
  assert(JSON.stringify(graphTabs) === JSON.stringify(expectedTabs), "subpanel graph mode: graph tabs fixed");
  assert(JSON.stringify(tableTabs) === JSON.stringify(expectedTabs), "subpanel graph mode: table tabs fixed");

  assert(resolveGraphTabMode("sim") === "graph", "subpanel graph mode: sim => graph");
  assert(resolveGraphTabMode("graph") === "graph", "subpanel graph mode: graph => graph");
  assert(resolveGraphTabMode("table") === "graph", "subpanel graph mode: table => graph");
  assert(resolveGraphTabMode("unknown") === "graph", "subpanel graph mode: fallback => graph");

  console.log("seamgrim subpanel graph mode runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
