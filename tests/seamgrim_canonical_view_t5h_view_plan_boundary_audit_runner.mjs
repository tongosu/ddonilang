#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function stableJson(value) {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

async function collectJavaScriptFiles(directory) {
  const result = [];
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) result.push(...await collectJavaScriptFiles(absolute));
    else if (entry.isFile() && entry.name.endsWith(".js")) result.push(absolute);
  }
  return result.sort();
}

const root = path.resolve(import.meta.dirname, "..");
const contract = JSON.parse(await fs.readFile(
  path.join(root, "pack/seamgrim_canonical_view_t5h_view_plan_boundary_audit_v1/contract.detjson"),
  "utf8",
));
assert(contract.schema === "seamgrim.canonical_view.t5h_view_plan_boundary_audit.v1", "view-plan audit contract schema mismatch");

const modulePath = String(process.env.DDN_T5H_VIEW_PLAN_MODULE_PATH ?? "").trim()
  || path.join(root, contract.presentation_adapter);
const moduleUrl = `${pathToFileURL(modulePath).href}?audit=${Date.now()}`;
const { buildSeamgrimViewPlanRuntime } = await import(moduleUrl);
assert(typeof buildSeamgrimViewPlanRuntime === "function", "view-plan runtime export missing");

const viewPlan = JSON.parse(await fs.readFile(
  path.join(root, "solutions/seamgrim_ui_mvp/lessons/rep_phys_pendulum_wave1_v1/view_plan.detjson"),
  "utf8",
));
const observation = {
  values: { theta: 0.5, omega: 0.25, speed: 0.25, kinetic_energy: 0.03125, potential_energy: 1.2, bob_x: 0.479426, bob_y: -0.877583, t: 2, tick_id: 7 },
  all_values: { theta: 0.5, omega: 0.25, speed: 0.25, kinetic_energy: 0.03125, potential_energy: 1.2, bob_x: 0.479426, bob_y: -0.877583, t: 2, tick_id: 7 },
};
const baseSpace2d = {
  schema: "seamgrim.space2d.v0",
  source: "authoritative_fixture",
  points: [{ id: "pivot", x: 0, y: 0 }, { id: "bob", x: 0.479426, y: -0.877583 }],
  shapes: [{ kind: "line", id: "rod", from: "pivot", to: "bob" }],
  meta: { view_roles: ["world_geometry"], authoritative_fixture: true },
};
const baseBefore = stableJson(baseSpace2d);
const adapted = buildSeamgrimViewPlanRuntime({
  viewPlan,
  observation,
  views: { space2d: baseSpace2d, space2dAuthorityClass: "AUTHORITATIVE_STRUCTURED", space2dCanonicalPayloadHash: "candidate:fixture" },
});
assert(adapted?.views?.space2dAuthorityClass === "PRESENTATION_ONLY", "view-plan authority class escaped presentation-only boundary");
assert(adapted?.views?.space2d?.meta?.authority_class === "PRESENTATION_ONLY", "composed view-plan payload authority drift");
assert(adapted?.views?.space2d?.meta?.base_authority_class === "AUTHORITATIVE_STRUCTURED", "authoritative base custody lost");
assert(adapted?.views?.space2d?.meta?.state_hash_participation === false, "view-plan entered state hash");
assert(stableJson(baseSpace2d) === baseBefore, "view-plan mutated authoritative input");
assert(stableJson(adapted.views.space2d.points) === stableJson(baseSpace2d.points), "view-plan changed authoritative point identity/order/value");
assert(stableJson(adapted.views.space2d.shapes.slice(0, baseSpace2d.shapes.length)) === stableJson(baseSpace2d.shapes), "view-plan changed authoritative shape identity/order/value");

const noBase = buildSeamgrimViewPlanRuntime({ viewPlan, observation, views: {} });
assert(noBase?.report?.presentation_only === true, "no-base view-plan report lost presentation-only marker");
assert(noBase?.report?.state_hash_participation === false, "no-base view-plan entered state hash");
assert(noBase?.views?.space2dAuthorityClass === "PRESENTATION_ONLY", "no-base view-plan authority drift");
assert(noBase?.views?.space2d?.meta?.canonical_space2d_synthesis === false, "no-base view-plan claimed canonical synthesis");
assert((noBase?.views?.space2d?.points ?? []).length === 0, "no-base view-plan created semantic world points");

const graphPlan = {
  schema: "seamgrim.view_plan.v1",
  lesson_id: "t5h-view-plan-graph",
  anchors: [
    { anchor_id: "obs.time", source_ref: { kind: "observation", key: "time" } },
    { anchor_id: "obs.position", source_ref: { kind: "observation", key: "position.x" } },
  ],
  measures: [],
  graph_cursor: { x_anchor: "obs.time", y_anchor: "obs.position", series_id: "position.x" },
};
const baseGraph = { schema: "seamgrim.graph.v0", series: [{ id: "position.x", points: [[0, 0], [2, 12.5]] }] };
const graphAdapted = buildSeamgrimViewPlanRuntime({
  viewPlan: graphPlan,
  observation: { values: { time: 2, "position.x": 12.5, tick_id: 7 } },
  views: { graph: baseGraph },
});
assert(stableJson(graphAdapted.views.graph.series) === stableJson(baseGraph.series), "view-plan changed canonical graph series");
const graphNoBase = buildSeamgrimViewPlanRuntime({
  viewPlan: graphPlan,
  observation: { values: { time: 2, "position.x": 12.5 } },
  views: {},
});
assert(!Array.isArray(graphNoBase?.views?.graph?.series), "view-plan created missing graph series");

const [adapterSource, canonicalOwnerSource, runSource] = await Promise.all([
  fs.readFile(modulePath, "utf8"),
  fs.readFile(path.join(root, contract.canonical_owner), "utf8"),
  fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js"), "utf8"),
]);
assert(!canonicalOwnerSource.includes("view_plan_runtime"), "canonical owner imports presentation adapter");
assert(!adapterSource.includes("buildCanonicalViewPayloadSet"), "presentation adapter builds canonical payload set");
assert(!adapterSource.includes("seamgrim.semantic_view_payload"), "presentation adapter emits canonical payload schema");
assert(runSource.includes('canonicalViewPayloads?.space2d?.status === "available"'), "Run does not select canonical space2d base explicitly");
assert(runSource.includes("space2d: canonicalViewPlanBase"), "Run does not pass exact canonical base to view-plan adapter");

const uiRoot = path.join(root, "solutions/seamgrim_ui_mvp/ui");
const callerPaths = [];
for (const file of await collectJavaScriptFiles(uiRoot)) {
  if (path.resolve(file) === path.resolve(path.join(root, contract.presentation_adapter))) continue;
  const source = await fs.readFile(file, "utf8");
  if (source.includes("buildSeamgrimViewPlanRuntime")) callerPaths.push(path.relative(root, file).replaceAll("\\", "/"));
}
assert(stableJson(callerPaths) === stableJson(contract.expected_product_callers), `view-plan caller inventory drift: ${stableJson(callerPaths)}`);

const validateOnly = String(process.env.DDN_T5H_VIEW_PLAN_VALIDATE_ONLY ?? "") === "1";
const existingRunnerResults = [];
if (!validateOnly) {
  for (const runner of contract.required_existing_runners) {
    const run = spawnSync(process.execPath, [runner], { cwd: root, env: process.env, encoding: "utf8", windowsHide: true, timeout: 120_000 });
    assert(run.status === 0, `${runner} failed with ${run.status}\n${run.stdout ?? ""}\n${run.stderr ?? ""}`);
    existingRunnerResults.push(runner);
  }
}

console.log(JSON.stringify({
  status: "PASS",
  audit_result: "T5H_VIEW_PLAN_PRESENTATION_ONLY_BOUNDARY_REPRODUCED",
  product_caller_count: callerPaths.length,
  authoritative_point_order_preserved: true,
  authoritative_shape_prefix_preserved: true,
  no_base_canonical_space2d_synthesis: 0,
  no_base_graph_series_synthesis: 0,
  state_hash_participation: 0,
  existing_runner_count: existingRunnerResults.length,
  validate_only: validateOnly,
  product_change: 0,
}));
