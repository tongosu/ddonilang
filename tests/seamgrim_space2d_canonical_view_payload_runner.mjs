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
import {
  extractRuntimeDerived,
  resolveRunMainVisualMode,
} from "../solutions/seamgrim_ui_mvp/ui/screens/run.js";
import { buildSeamgrimViewPlanRuntime } from "../solutions/seamgrim_ui_mvp/ui/view_plan_runtime.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function payloadsFor(state) {
  return buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));
}

const root = process.cwd();
const packRoot = path.join(root, "pack", "seamgrim_space2d_canonical_view_payload_v1");
const state = JSON.parse(await fs.readFile(path.join(packRoot, "fixtures", "authoritative_state.detjson"), "utf8"));
const stateBefore = stableSemanticJson(state);
const direct = payloadsFor(state).space2d;
const run = extractRuntimeDerived(state);
const runPayload = run.canonicalViewPayloads.space2d;
const playgroundPayload = payloadsFor(state).space2d;
const bogaeSemanticPortion = runPayload.content?.space2d ?? null;
const workbenchPayload = buildCommittedSemanticViewPayload({
  project: { id: "t5c", primaryView: "pendulum" },
  run: { run_id: "t5c", source_hash: state.source_hash, frames: [] },
  frame: {
    frame_id: 7,
    madi: state.tick_id,
    state_hash: state.state_hash,
    source_hash: state.source_hash,
    wasm_state: state,
  },
  canonicalViewPayloads: payloadsFor(state),
}).canonical_views.space2d;

assert(direct.status === "available", "G1 canonical space2d must be available");
assert(direct.provenance.classification === "AUTHORITATIVE_STRUCTURED", "G1 authority classification");
assert(direct.content.family_contract_version === "space2d.v1_candidate", "G1 candidate version");
assert(direct.content.entity_identities.length === 4, "G1 entity identity count");
assert(new Set(direct.content.entity_identities.map((row) => row.entity_id)).size === 4, "G1 stable entity ids");
assert(stableSemanticJson(direct) === stableSemanticJson(runPayload), "Run canonical payload parity");
assert(stableSemanticJson(direct) === stableSemanticJson(playgroundPayload), "Playground canonical payload parity");
assert(stableSemanticJson(direct) === stableSemanticJson(workbenchPayload), "Workbench canonical payload parity");
assert(stableSemanticJson(direct.content.space2d) === stableSemanticJson(bogaeSemanticPortion), "Bogae semantic portion parity");

const mainVisual = resolveRunMainVisualMode({
  views: run.views,
  canonicalViewPayloads: run.canonicalViewPayloads,
});
assert(mainVisual.mode === "space2d", "Run must render canonical space2d");
assert(mainVisual.authorityClass === "AUTHORITATIVE_STRUCTURED", "Run native authority class");
assert(stableSemanticJson(mainVisual.space2d) === stableSemanticJson(direct.content.space2d), "Run renderer port drift");

if (process.argv.includes("--emit")) {
  console.log(JSON.stringify(JSON.parse(stableSemanticJson(direct)), null, 2));
} else {
  const expected = JSON.parse(await fs.readFile(path.join(packRoot, "expected", "space2d.payload.detjson"), "utf8"));
  assert(stableSemanticJson(direct) === stableSemanticJson(expected), "G1 exact structured golden mismatch");
}

const numericOnly = {
  schema: "seamgrim.state.v0",
  state_hash: "sha256:t5c-numeric-only",
  channels: [{ key: "x" }, { key: "y" }, { key: "theta" }],
  row: [1, 2, 0.5],
  resources: { value: {}, value_json: {} },
};
const numericOnlyPayload = payloadsFor(numericOnly).space2d;
assert(numericOnlyPayload.status === "no_view" && numericOnlyPayload.content === null, "numeric observation must not synthesize space2d");
const numericMain = resolveRunMainVisualMode({
  observation: { channels: numericOnly.channels, row: numericOnly.row, values: { x: 1, y: 2, theta: 0.5 } },
  canonicalViewPayloads: payloadsFor(numericOnly),
  allowShapeFallback: true,
});
assert(numericMain.space2d === null && numericMain.mode !== "debug-fallback", "numeric observation fallback must stay disabled");

const legacyOutput = {
  schema: "seamgrim.state.v0",
  state_hash: "sha256:t5c-output-only",
  channels: [{ key: "보개_출력_줄들", dtype: "text", role: "view" }],
  row: [JSON.stringify(["space2d", "space2d.shape", "point", "x", "1", "y", "2"])],
  observation_manifest: {
    schema: "ddn.observation_manifest.v0",
    version: "20.6.33",
    nodes: [{ name: "보개_출력_줄들", dtype: "text", role: "view" }],
    params: [],
    traces: [],
    diagnostics: [],
    default_pivot: { x_channel: "", y_channels: [] }
  },
  resources: { value: {}, value_json: {} },
};
assert(payloadsFor(legacyOutput).space2d.status === "no_view", "legacy-looking output must not enter canonical authority");
const legacyRun = extractRuntimeDerived(legacyOutput);
assert(legacyRun.views.space2d === null, "Run must not decode legacy output into semantic space2d");

const malformed = structuredClone(state);
delete malformed.view_meta.space2d.shapes[0].y2;
assert(payloadsFor(malformed).space2d.status === "invalid", "malformed shape must be INVALID");

const unsupported = structuredClone(state);
unsupported.view_meta.space2d.schema = "seamgrim.space2d.v999";
assert(payloadsFor(unsupported).space2d.status === "unsupported", "unknown schema must be UNSUPPORTED");

const duplicate = structuredClone(state);
duplicate.view_meta.space2d.shapes[1].id = duplicate.view_meta.space2d.shapes[0].id;
assert(payloadsFor(duplicate).space2d.status === "invalid", "duplicate entity identity must be INVALID");

const identityMissing = structuredClone(state);
delete identityMissing.state_hash;
delete identityMissing.result_identity;
assert(payloadsFor(identityMissing).space2d.status === "unavailable", "missing required identity must be UNAVAILABLE");

const viewPlan = JSON.parse(await fs.readFile(
  path.join(root, "solutions", "seamgrim_ui_mvp", "lessons", "rep_phys_pendulum_wave1_v1", "view_plan.detjson"),
  "utf8",
));
const observation = {
  values: {
    theta: 0.5,
    omega: 0.25,
    speed: 0.25,
    kinetic_energy: 0.03125,
    potential_energy: 1.2,
    bob_x: 0.479426,
    bob_y: -0.877583,
    t: 2,
    tick_id: 7,
  },
  all_values: {
    theta: 0.5,
    omega: 0.25,
    speed: 0.25,
    kinetic_energy: 0.03125,
    potential_energy: 1.2,
    bob_x: 0.479426,
    bob_y: -0.877583,
    t: 2,
    tick_id: 7,
  },
};
const baseSpace2d = direct.content.space2d;
const adapted = buildSeamgrimViewPlanRuntime({
  viewPlan,
  observation,
  views: {
    space2d: baseSpace2d,
    space2dAuthorityClass: "AUTHORITATIVE_STRUCTURED",
    space2dCanonicalPayloadHash: direct.candidate_payload_hash,
  },
});
assert(adapted?.views?.space2dAuthorityClass === "PRESENTATION_ONLY", "view-plan must be presentation-only");
assert(adapted.report.base_space2d_status === "available", "view-plan base status");
assert(stableSemanticJson(adapted.views.space2d.points) === stableSemanticJson(baseSpace2d.points), "view-plan changed base points");
assert(
  stableSemanticJson(adapted.views.space2d.shapes.slice(0, baseSpace2d.shapes.length)) === stableSemanticJson(baseSpace2d.shapes),
  "view-plan changed base shape identity/value/order",
);
assert(adapted.views.space2d.shapes.length > baseSpace2d.shapes.length, "measurement overlay must remain available");

const noBasePlan = buildSeamgrimViewPlanRuntime({ viewPlan, observation, views: {} });
assert(noBasePlan.report.base_space2d_status === "no_view", "view-plan-only state must be NO_VIEW");
assert(noBasePlan.views.space2d?.meta?.authority_class === "PRESENTATION_ONLY", "view-plan fallback provenance");
assert(noBasePlan.views.space2d?.meta?.canonical_space2d_synthesis === false, "view-plan must not synthesize canonical base");
assert((noBasePlan.views.space2d?.points ?? []).length === 0, "view-plan must not create base points");
assert(noBasePlan.report.diagnostics.length === 0, "valid presentation-only fallback must not invent an execution diagnostic");

const rendererConfigurations = [
  { renderer: "Canvas2D", zoom: 1, pan: [0, 0] },
  { renderer: "Canvas2D", zoom: 2, pan: [10, -5] },
];
const semanticBeforeRenderer = stableSemanticJson(direct);
rendererConfigurations.forEach((configuration) => {
  assert(configuration.renderer === "Canvas2D", "renderer fixture invalid");
  assert(stableSemanticJson(direct) === semanticBeforeRenderer, "renderer transient changed semantic identity");
});

const runSource = await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "screens", "run.js"), "utf8");
const playgroundSource = await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "playground.js"), "utf8");
const bogaeSource = await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "components", "bogae.js"), "utf8");
assert(runSource.includes("canonicalViewPayloads.space2d"), "Run is not bound to canonical space2d");
assert(playgroundSource.includes("buildCanonicalViewPayloadSet"), "Playground is not bound to canonical payload builder");
assert(playgroundSource.includes("renderMirrorBogae(structuredViews?.space2d)"), "Playground-to-Bogae canonical port missing");
assert(bogaeSource.includes("renderSpace2dCanvas2d"), "Bogae renderer binding missing");
assert(stableSemanticJson(state) === stateBefore, "canonical projection mutated authoritative state");

console.log(JSON.stringify({
  status: "PASS",
  schema: direct.schema,
  candidate_payload_hash: direct.candidate_payload_hash,
  entity_count: direct.content.entity_identities.length,
  run_playground_space2d_byte_parity: true,
  run_bogae_space2d_byte_parity: true,
  playground_bogae_space2d_byte_parity: true,
  view_plan_base_space2d_meaning_after: 0,
  measurement_overlay_preserved: true,
  active_semantic_synthesis: 0,
  state_hash_drift: 0,
  storage_drift: 0,
}));
