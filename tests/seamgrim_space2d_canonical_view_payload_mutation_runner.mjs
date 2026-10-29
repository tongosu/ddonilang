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
import { buildSeamgrimViewPlanRuntime } from "../solutions/seamgrim_ui_mvp/ui/view_plan_runtime.js";

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
const state = JSON.parse(await fs.readFile(
  path.join(root, "pack", "seamgrim_space2d_canonical_view_payload_v1", "fixtures", "authoritative_state.detjson"),
  "utf8",
));
const payload = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state)).space2d;
const missingState = structuredClone(state);
delete missingState.view_meta.space2d;
missingState.channels = [{ key: "x" }, { key: "y" }];
missingState.row = [1, 2];
const missingPayload = buildCanonicalViewPayloadSet(
  extractCanonicalViewAuthorityInputsFromState(missingState),
).space2d;

const results = [];
results.push(proveMutationRed({
  label: "M1_MISSING_SPACE2D_OBSERVATION_WORLD_SYNTHESIS",
  greenValue: missingPayload,
  mutate(value) {
    value.status = "available";
    value.content = structuredClone(payload.content);
    return value;
  },
  invariant(value) {
    return value.status === "no_view" && value.content === null;
  },
}));

const base = payload.content.space2d;
const plan = {
  schema: "seamgrim.view_plan.v1",
  lesson_id: "t5c-mutation",
  anchors: [],
  measures: [],
};
const adapted = buildSeamgrimViewPlanRuntime({
  viewPlan: plan,
  observation: {},
  views: {
    space2d: base,
    space2dCanonicalPayloadHash: payload.candidate_payload_hash,
  },
});
const basePair = {
  before: base.shapes,
  after: structuredClone(adapted.views.space2d.shapes.slice(0, base.shapes.length)),
};
results.push(proveMutationRed({
  label: "M2_VIEW_PLAN_BASE_COORDINATE_DRIFT",
  greenValue: basePair,
  mutate(value) {
    value.after[0].x2 += 1;
    return value;
  },
  invariant(value) {
    return stableSemanticJson(value.before) === stableSemanticJson(value.after);
  },
}));

const surfacePair = {
  run: base.shapes,
  playground: structuredClone(base.shapes),
};
results.push(proveMutationRed({
  label: "M3_RUN_PLAYGROUND_SHAPE_ORDER_DRIFT",
  greenValue: surfacePair,
  mutate(value) {
    value.playground.reverse();
    return value;
  },
  invariant(value) {
    return stableSemanticJson(value.run) === stableSemanticJson(value.playground);
  },
}));

console.log(JSON.stringify({
  status: "PASS",
  mutation_count: results.length,
  mutation_residue: 0,
  results,
}));
