#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { extractCanonicalViewAuthorityInputsFromState } from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  buildCanonicalGrid2dViewPayload,
  buildCanonicalViewPayloadSet,
  buildCommittedSemanticViewPayload,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";
import { canonicalViewPayloadToCellGrid } from "../solutions/seamgrim_ui_mvp/ui/runtime/cell_grid_renderer.js";
import { NurimakerGridRenderer } from "../solutions/seamgrim_ui_mvp/ui/runtime/nurimaker_grid_renderer.js";
import { extractRuntimeDerived } from "../solutions/seamgrim_ui_mvp/ui/screens/run.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const root = process.cwd();
const packRoot = path.join(root, "pack/seamgrim_console_grid_grid2d_canonical_view_payload_v1");
const state = JSON.parse(await fs.readFile(path.join(packRoot, "fixtures/authoritative_state.detjson"), "utf8"));
const before = stableSemanticJson(state);
const inputs = extractCanonicalViewAuthorityInputsFromState(state);
const direct = buildCanonicalViewPayloadSet(inputs);
const run = extractRuntimeDerived(state).canonicalViewPayloads;
const playground = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));
const workbench = buildCommittedSemanticViewPayload({
  project: { id: "t5e", primaryView: "grid" },
  run: { run_id: "t5e", source_hash: state.source_hash, frames: [] },
  frame: { frame_id: 1, madi: state.tick_id, state_hash: state.state_hash, source_hash: state.source_hash, wasm_state: state },
  canonicalViewPayloads: direct,
}).canonical_views;

for (const family of ["console_grid", "grid2d"]) {
  assert(direct[family].status === "available", `${family} unavailable`);
  const bytes = stableSemanticJson(direct[family]);
  assert(bytes === stableSemanticJson(run[family]), `${family} Run parity failed`);
  assert(bytes === stableSemanticJson(playground[family]), `${family} Playground parity failed`);
  assert(bytes === stableSemanticJson(workbench[family]), `${family} Workbench parity failed`);
  if (!process.argv.includes("--emit")) {
    const expected = JSON.parse(await fs.readFile(path.join(packRoot, `expected/${family}.payload.detjson`), "utf8"));
    assert(bytes === stableSemanticJson(expected), `${family} exact golden mismatch`);
  } else {
    console.log(`${family.toUpperCase()}=${JSON.stringify(JSON.parse(bytes), null, 2)}`);
  }
}

assert(direct.console_grid.content.grid.rows === 2, "console-grid row count drift");
assert(direct.console_grid.content.grid.cols === 2, "console-grid Unicode code-point width drift");
assert(direct.console_grid.content.grid.cells.map((cell) => cell.text).join("") === "가😀#P", "console-grid ordering drift");
assert(direct.grid2d.content.grid.tiles.flat().join(",") === "1,0,2,0,3,0", "grid2d tile order drift");
assert(direct.grid2d.content.grid.actors.map((actor) => actor.actor_id).join(",") === "player,goal", "grid2d actor identity/order drift");

const consoleRender = canonicalViewPayloadToCellGrid(direct.console_grid);
const gridRender = canonicalViewPayloadToCellGrid(direct.grid2d, {
  gridState: state.view_meta.grid2d,
  actorList: state.view_meta.grid2d.actors,
});
assert(consoleRender.cells.map((cell) => cell.text).join("") === "가😀#P", "console presentation adapter drift");
assert(gridRender.cells.some((cell) => cell.actor_id === "player"), "grid2d presentation actor missing");

const missingIdentity = structuredClone(state);
delete missingIdentity.state_hash;
delete missingIdentity.result_identity;
const missing = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(missingIdentity));
assert(missing.console_grid.status === "unavailable" && missing.grid2d.status === "unavailable", "missing identity not fail-closed");

const noConsole = structuredClone(state);
delete noConsole.console_log;
assert(buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(noConsole)).console_grid.status === "no_view", "no console must stay no_view");

const unsupported = structuredClone(state);
unsupported.view_meta.grid2d.schema = "seamgrim.nurimaker.grid_state.v999";
const unsupportedPayload = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(unsupported)).grid2d;
assert(unsupportedPayload.status === "unsupported", "unknown grid2d schema must be unsupported");
assert(unsupportedPayload.diagnostic?.code === "E_CANONICAL_GRID2D_UNSUPPORTED_CANDIDATE", "grid2d unsupported diagnostic drift");

const invalid = structuredClone(state);
invalid.view_meta.grid2d.tiles[0].pop();
assert(buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(invalid)).grid2d.status === "invalid", "ragged grid must be invalid");

const ambiguous = structuredClone(state);
delete ambiguous.view_meta.grid2d;
ambiguous.resources.value_json = {
  "grid.a": state.view_meta.grid2d,
  "grid.b": structuredClone(state.view_meta.grid2d),
};
assert(buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(ambiguous)).grid2d.status === "invalid", "ambiguous grid2d must fail closed");

const renderer = new NurimakerGridRenderer(null, {
  gridState: state.view_meta.grid2d,
  actorList: state.view_meta.grid2d.actors,
  canonicalIdentity: inputs.identity,
});
renderer.render();
assert(stableSemanticJson(renderer.getLastCanonicalPayload()) === stableSemanticJson(direct.grid2d), "Nurimaker canonical owner drift");

const rendererWithoutIdentity = new NurimakerGridRenderer(null, {
  gridState: state.view_meta.grid2d,
  actorList: state.view_meta.grid2d.actors,
});
rendererWithoutIdentity.render();
assert(rendererWithoutIdentity.getLastCanonicalPayload().status === "unavailable", "grid renderer invented identity");

assert(stableSemanticJson(state) === before, "projection mutated source state");
assert(typeof buildCanonicalGrid2dViewPayload === "function", "grid2d canonical builder missing");
const runSource = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js"), "utf8");
const bogaeSource = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/components/bogae.js"), "utf8");
const gridRendererSource = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/runtime/nurimaker_grid_renderer.js"), "utf8");
assert(runSource.includes("this.bogae.renderCanonicalCellGrid(canonicalCellGridPayload)"), "Run active console-grid canonical binding missing");
assert(bogaeSource.includes("canonicalViewPayloadToCellGrid(payload)"), "Bogae canonical presentation adapter missing");
assert(gridRendererSource.includes("buildCanonicalGrid2dViewPayload"), "Nurimaker canonical grid2d owner missing");

console.log(JSON.stringify({
  status: "PASS",
  families: ["console-grid", "grid2d"],
  run_workbench_playground_byte_parity: true,
  workbench_grid_merged: 0,
  hidden_semantic_fallback: 0,
  ambiguous_resource_success: 0,
  state_hash_drift: 0,
  storage_drift: 0,
}));
