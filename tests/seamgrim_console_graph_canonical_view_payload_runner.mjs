#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import {
  extractCanonicalViewAuthorityInputsFromState,
} from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  buildCanonicalViewPayloadSet,
  buildCommittedSemanticViewPayload,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";
import { extractRuntimeDerived } from "../solutions/seamgrim_ui_mvp/ui/screens/run.js";
import { buildSeamgrimViewPlanRuntime } from "../solutions/seamgrim_ui_mvp/ui/view_plan_runtime.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const root = process.cwd();
const packRoot = path.join(root, "pack", "seamgrim_console_graph_canonical_view_payload_v1");
const state = JSON.parse(await fs.readFile(path.join(packRoot, "fixtures", "authoritative_state.detjson"), "utf8"));
const stateBytesBefore = stableSemanticJson(state);
const authorityInputs = extractCanonicalViewAuthorityInputsFromState(state);
const directPayloads = buildCanonicalViewPayloadSet(authorityInputs);
const runPayloads = extractRuntimeDerived(state).canonicalViewPayloads;
const workbenchPayloads = buildCommittedSemanticViewPayload({
  project: { id: "t5b", primaryView: "line" },
  run: { run_id: "t5b", source_hash: state.source_hash, frames: [] },
  frame: {
    frame_id: 7,
    madi: state.tick_id,
    state_hash: state.state_hash,
    source_hash: state.source_hash,
    wasm_state: state,
  },
  canonicalViewPayloads: buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state)),
}).canonical_views;

assert(directPayloads.console.status === "available", "canonical console must be available");
assert(directPayloads.graph.status === "available", "canonical graph must be available");
assert(
  directPayloads.console.provenance.input_authority === "native_authoritative_structured_input",
  "console native provenance missing",
);
assert(
  directPayloads.graph.provenance.input_authority === "native_authoritative_structured_input",
  "graph native provenance missing",
);

for (const family of ["console", "graph"]) {
  const directBytes = stableSemanticJson(directPayloads[family]);
  assert(directBytes === stableSemanticJson(runPayloads[family]), `${family} Run payload parity failed`);
  assert(directBytes === stableSemanticJson(workbenchPayloads[family]), `${family} Workbench payload parity failed`);
  if (process.argv.includes("--emit")) {
    console.log(`${family.toUpperCase()}=${JSON.stringify(JSON.parse(directBytes), null, 2)}`);
  } else {
    const expected = JSON.parse(await fs.readFile(path.join(packRoot, "expected", `${family}.payload.detjson`), "utf8"));
    assert(directBytes === stableSemanticJson(expected), `${family} exact structured golden mismatch`);
  }
}

const missingIdentity = buildCanonicalViewPayloadSet({
  ...authorityInputs,
  identity: { ...authorityInputs.identity, state_hash: "", result_identity: "" },
});
assert(missingIdentity.console.status === "unavailable", "missing console identity must fail closed");
assert(missingIdentity.graph.status === "unavailable", "missing graph identity must fail closed");

const unsupportedState = structuredClone(state);
unsupportedState.resources.value_json["graph.result"].schema = "seamgrim.graph.v999";
const unsupported = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(unsupportedState));
assert(unsupported.graph.status === "unsupported", "unsupported graph schema must fail closed");
assert(unsupported.graph.content === null, "unsupported graph must not synthesize content");

const noGraphState = structuredClone(state);
delete noGraphState.resources.value_json["graph.result"];
const noGraph = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(noGraphState));
assert(noGraph.graph.status === "no_view", "missing graph must be explicit no_view");
assert(noGraph.graph.content === null, "missing graph must not synthesize graph content");

const unrelatedStructuredResourceState = structuredClone(noGraphState);
unrelatedStructuredResourceState.resources.value_json["table.result"] = {
  schema: "seamgrim.table.v0",
  rows: [["not", "a", "graph"]],
};
const unrelatedStructuredResource = buildCanonicalViewPayloadSet(
  extractCanonicalViewAuthorityInputsFromState(unrelatedStructuredResourceState),
);
assert(unrelatedStructuredResource.graph.status === "no_view", "non-graph structured resources must stay outside graph authority");

const graphBefore = directPayloads.graph.content.graph;
const planRuntime = buildSeamgrimViewPlanRuntime({
  viewPlan: {
    schema: "seamgrim.view_plan.v1",
    lesson_id: "t5b",
    anchors: [
      { anchor_id: "obs.time", source_ref: { kind: "observation", key: "time" }, unit: "s" },
      { anchor_id: "obs.position", source_ref: { kind: "observation", key: "position.x" }, unit: "m" },
    ],
    measures: [],
    graph_cursor: {
      cursor_id: "graph.current.position",
      x_anchor: "obs.time",
      y_anchor: "obs.position",
      series_id: "position.x",
    },
  },
  observation: {
    values: { time: 2, "position.x": 12.5, tick_id: 7 },
    all_values: { time: 2, "position.x": 12.5, tick_id: 7 },
  },
  views: { graph: graphBefore },
});
const graphAfter = planRuntime.views.graph;
assert(
  stableSemanticJson(graphBefore.series) === stableSemanticJson(graphAfter.series),
  "view-plan adapter changed canonical base series",
);
assert(
  stableSemanticJson(directPayloads.graph.content.series_identities)
    === stableSemanticJson(graphAfter.series.map((row) => row.id ?? row.name)),
  "renderer/presentation adapter changed series identities",
);
const noBaseGraphRuntime = buildSeamgrimViewPlanRuntime({
  viewPlan: {
    schema: "seamgrim.view_plan.v1",
    lesson_id: "t5b-no-graph",
    anchors: [
      { anchor_id: "obs.time", source_ref: { kind: "observation", key: "time" } },
      { anchor_id: "obs.position", source_ref: { kind: "observation", key: "position.x" } },
    ],
    measures: [],
    graph_cursor: { x_anchor: "obs.time", y_anchor: "obs.position" },
  },
  observation: { values: { time: 2, "position.x": 12.5 } },
  views: {},
});
assert(
  !Array.isArray(noBaseGraphRuntime.views.graph?.series),
  "view-plan compatibility cursor metadata must not create base graph series meaning",
);

const runSource = await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "screens", "run.js"), "utf8");
const workbenchSource = await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "workbench", "workbench.js"), "utf8");
assert(runSource.includes("buildCanonicalViewPayloadSet"), "Run product path is not bound to canonical payload builder");
assert(workbenchSource.includes("buildCanonicalViewPayloadSet"), "Workbench product path is not bound to canonical payload builder");
assert(stableSemanticJson(state) === stateBytesBefore, "canonical payload projection mutated authoritative state");

console.log(JSON.stringify({
  status: "PASS",
  schema: directPayloads.console.schema,
  console_entry_count: directPayloads.console.content.ordered_entries.length,
  graph_series_count: directPayloads.graph.content.graph.series.length,
  run_workbench_console_byte_parity: true,
  run_workbench_graph_byte_parity: true,
  view_plan_base_graph_meaning_after: 0,
  state_hash_drift: 0,
  storage_drift: 0,
}));
