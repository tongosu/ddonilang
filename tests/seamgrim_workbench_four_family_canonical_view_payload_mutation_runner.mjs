#!/usr/bin/env node

import {
  buildCanonicalWorkbenchViewPayload,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";
import { buildSemanticRenderModel } from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_renderer.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const schemas = {
  line: "seamgrim.workbench.line_result.v1_candidate",
  pendulum: "seamgrim.workbench.pendulum_result.v1_candidate",
  projectile: "seamgrim.workbench.projectile_result.v1_candidate",
  grid: "seamgrim.workbench.grid_result.v1_candidate",
};

const contents = {
  line: { current: { x: 2, y: 5 }, equation: { a: 2, b: 1 } },
  pendulum: {
    current: {
      theta: 0.25,
      omega: -0.1,
      energy: 1.4,
      kinetic_energy: 0.4,
      potential_energy: 1,
      length: 2,
      physical_bob: { x: 0.49, y: -1.94 },
      normalized_bob: { x: 0.245, y: 0.97 },
    },
  },
  projectile: { current: { t: 1, x: 0.8, y: 1.5, vx: 8, vy: 14.02 } },
  grid: {
    position: { x: 3, y: 0 },
    dimensions: { columns: 8, rows: 8 },
    space2d: {
      schema: "seamgrim.space2d.v0",
      drawlist: [{ kind: "rect", x: 0, y: 0, w: 1, h: 1 }],
    },
  },
};

function resultSet() {
  return {
    schema: "seamgrim.workbench.authoritative_result_set.v1_candidate",
    producer: "ddonirang_tool.wasm_runtime",
    provenance: "native_authoritative_structured_result",
    results: Object.fromEntries(Object.keys(schemas).map((family) => [family, {
      schema: schemas[family],
      family,
      status: "available",
      producer: "ddonirang_tool.wasm_runtime",
      provenance: "native_authoritative_structured_result",
      content: structuredClone(contents[family]),
    }])),
  };
}

function payload(family, set = resultSet()) {
  return buildCanonicalWorkbenchViewPayload({
    identity: { state_hash: "blake3:t5f_mutation", source_hash: "sha256:t5f_mutation", tick_id: 1 },
    projection: {
      status: "available",
      result_set: set,
      source: "structured_view_meta",
      provenance: "native_authoritative_structured_result",
      classification: "AUTHORITATIVE_STRUCTURED",
    },
    family,
  });
}

for (const family of Object.keys(schemas)) {
  assert(payload(family).status === "available", `${family}: GREEN baseline unavailable`);
}

const m1 = resultSet();
m1.schema = "seamgrim.workbench.authoritative_result_set.v999";
assert(payload("line", m1).status === "unsupported", "M1 unknown result-set schema did not fail closed");

const m2 = resultSet();
m2.results.projectile.content.current.vy = "14.02";
assert(payload("projectile", m2).status === "invalid", "M2 non-numeric projectile velocity did not fail closed");

const m3 = resultSet();
m3.results.pendulum.producer = "workbench.js";
assert(payload("pendulum", m3).status === "invalid", "M3 JS producer substitution did not fail closed");

const m4 = resultSet();
delete m4.results.grid;
assert(payload("grid", m4).status === "no_view", "M4 missing family did not remain no_view");

const rawOnly = buildSemanticRenderModel({
  committed_frame: { frame_id: 1, madi: 1, state_hash: "blake3:raw_decoy" },
  ordered_observations: [
    { name: "x", value: 1 },
    { name: "y", value: 2 },
    { name: "vx", value: 3 },
    { name: "vy", value: 4 },
  ],
  committed_resources: { x: 1, y: 2, vx: 3, vy: 4 },
  canonical_views: null,
  view_config: { primary_view: "projectile", role: "primary" },
});
assert(rawOnly.status === "canonical_unavailable", "M5 raw-only decoy regained renderer success");

console.log("seamgrim_workbench_four_family_canonical_view_payload_mutation: ok (5/5 RED detected)");
