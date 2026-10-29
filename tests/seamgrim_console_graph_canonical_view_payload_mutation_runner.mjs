#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import {
  extractCanonicalViewAuthorityInputsFromState,
} from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  buildCanonicalViewPayloadSet,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function proveMutationRed({ label, greenValue, mutate, invariant }) {
  assert(invariant(greenValue), `${label}: initial GREEN failed`);
  const mutant = mutate(structuredClone(greenValue));
  assert(!invariant(mutant), `${label}: mutation did not turn RED`);
  assert(invariant(greenValue), `${label}: restored GREEN failed`);
  return { label, initial: "GREEN", mutated: "RED", restored: "GREEN" };
}

const root = process.cwd();
const fixturePath = path.join(
  root,
  "pack",
  "seamgrim_console_graph_canonical_view_payload_v1",
  "fixtures",
  "authoritative_state.detjson",
);
const state = JSON.parse(await fs.readFile(fixturePath, "utf8"));
const payloads = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));

const noGraphState = structuredClone(state);
delete noGraphState.resources.value_json["graph.result"];
const noGraphPayload = buildCanonicalViewPayloadSet(
  extractCanonicalViewAuthorityInputsFromState(noGraphState),
).graph;

const results = [];
results.push(proveMutationRed({
  label: "M1_LEGACY_GRAPH_SYNTHESIS_REINTRODUCED",
  greenValue: noGraphPayload,
  mutate(value) {
    value.status = "available";
    value.content = structuredClone(payloads.graph.content);
    return value;
  },
  invariant(value) {
    return value.status === "no_view" && value.content === null;
  },
}));

const parityPair = {
  run: payloads.console,
  workbench: structuredClone(payloads.console),
};
results.push(proveMutationRed({
  label: "M2_WORKBENCH_CONSOLE_ORDER_DRIFT",
  greenValue: parityPair,
  mutate(value) {
    value.workbench.content.ordered_entries.reverse();
    return value;
  },
  invariant(value) {
    return stableSemanticJson(value.run) === stableSemanticJson(value.workbench);
  },
}));

const graphIdentityPair = {
  declared: payloads.graph.content.series_identities,
  adapted: payloads.graph.content.graph.series.map((series) => series.id ?? series.name),
};
results.push(proveMutationRed({
  label: "M3_VIEW_PLAN_SERIES_IDENTITY_DRIFT",
  greenValue: graphIdentityPair,
  mutate(value) {
    value.adapted.reverse();
    return value;
  },
  invariant(value) {
    return stableSemanticJson(value.declared) === stableSemanticJson(value.adapted);
  },
}));

console.log(JSON.stringify({
  status: "PASS",
  mutation_count: results.length,
  mutation_residue: 0,
  results,
}));
