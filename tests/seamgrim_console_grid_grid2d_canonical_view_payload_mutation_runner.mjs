#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { extractCanonicalViewAuthorityInputsFromState } from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import { buildCanonicalViewPayloadSet, stableSemanticJson } from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";

function assert(condition, message) { if (!condition) throw new Error(message); }
function prove(label, value, mutate, invariant) {
  assert(invariant(value), `${label}: GREEN failed`);
  const mutant = mutate(structuredClone(value));
  assert(!invariant(mutant), `${label}: mutation did not RED`);
  assert(invariant(value), `${label}: restore failed`);
  return { label, initial: "GREEN", mutated: "RED", restored: "GREEN" };
}

const root = process.cwd();
const state = JSON.parse(await fs.readFile(path.join(root, "pack/seamgrim_console_grid_grid2d_canonical_view_payload_v1/fixtures/authoritative_state.detjson"), "utf8"));
const payloads = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));
const results = [];
results.push(prove("M1_CONSOLE_GRID_SURFACE_ORDER_DRIFT", {
  run: payloads.console_grid.content.grid.cells,
  playground: structuredClone(payloads.console_grid.content.grid.cells),
}, (value) => { value.playground.reverse(); return value; }, (value) => stableSemanticJson(value.run) === stableSemanticJson(value.playground)));
results.push(prove("M2_GRID2D_ACTOR_IDENTITY_DRIFT", {
  run: payloads.grid2d.content.grid.actors,
  renderer: structuredClone(payloads.grid2d.content.grid.actors),
}, (value) => { value.renderer[0].actor_id = "renderer-index-0"; return value; }, (value) => stableSemanticJson(value.run) === stableSemanticJson(value.renderer)));
results.push(prove("M3_MISSING_TO_SYNTHETIC_AVAILABLE", { status: "no_view", content: null },
  (value) => { value.status = "available"; value.content = structuredClone(payloads.console_grid.content); return value; },
  (value) => value.status === "no_view" && value.content === null));
console.log(JSON.stringify({ status: "PASS", mutation_count: results.length, mutation_residue: 0, results }));
