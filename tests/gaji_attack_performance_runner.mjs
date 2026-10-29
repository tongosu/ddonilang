import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";

const root = path.resolve(import.meta.dirname, "..");
const wasmDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm");
const wasm = await import(pathToFileURL(path.join(wasmDir, "ddonirang_tool.js")));
await wasm.default(fs.readFileSync(path.join(wasmDir, "ddonirang_tool_bg.wasm")));
const gridSource = JSON.parse(fs.readFileSync(path.join(root, "pack", "ddn_local_project_graph_v1", "fixtures", "project_run_grid_v1.json"), "utf8"));
const fixture = {
  schema: "ddn.gaji_extraction_plan.v1", source: gridSource, selected_modules: ["lib/grid.ddn"],
  gaji_id: "성능가지", version: "1.0.0", description: "검색 규모", tags: ["성능"], profile: "내", origin_project_ref: "project:performance",
};

function artifact(index) {
  const request = structuredClone(fixture);
  request.gaji_id = `성능가지-${String(index).padStart(3, "0")}`;
  request.description = `검색 규모 가지 ${index}`;
  return JSON.parse(wasm.wasm_local_gaji_artifact_json(JSON.stringify(request)));
}
function percentile(rows, p) { return rows.slice().sort((a, b) => a - b)[Math.min(rows.length - 1, Math.ceil(rows.length * p) - 1)]; }

let store = { schema: "ddn.local_gotgan_store.v1", artifacts: [] };
const artifacts = [];
for (let index = 0; index < 105; index += 1) {
  const item = artifact(index);
  artifacts.push(item);
  store = JSON.parse(wasm.wasm_publish_local_gotgan_json(JSON.stringify({ schema: "ddn.local_gotgan_publish_request.v1", store, artifact: item })));
}
const searchTimes = [];
let firstOrder = null;
for (let index = 0; index < 30; index += 1) {
  const started = performance.now();
  const result = JSON.parse(wasm.wasm_search_local_gotgan_json(JSON.stringify({
    schema: "ddn.local_gotgan_search_request.v1", store, query: "검색 규모", profiles: ["내"],
  })));
  searchTimes.push(performance.now() - started);
  assert.equal(result.rows.length, 105);
  const order = result.rows.map((row) => row.gaji_id).join("|");
  firstOrder ??= order;
  assert.equal(order, firstOrder);
}

let project = {
  schema: "ddn.virtual_project_source.v1", kind: "seamgrim.local_project_tree.v0", root_path: "뿌리.ddn", entry_path: "app/main.ddn",
  files: [
    { path: "뿌리.ddn", role: "root", utf8_text: '프로젝트 { 이름: "규모묶음". 첫글: "./app/main.ddn". 목표 { 앱: "./app/main.ddn". } 모듈길 { 로컬: "./lib". } }.' },
    { path: "app/main.ddn", role: "ddn_source", utf8_text: '"규모" 보여주기.' },
  ],
};
const installTimes = [];
for (let index = 0; index < 20; index += 1) {
  const request = {
    schema: "ddn.gaji_lifecycle_plan.v1", source: project,
    expected_source_sha256: wasm.wasm_project_source_sha256_json(JSON.stringify(project)), expected_lock_sha256: null,
    operation: { kind: "install", alias: `가지${index}`, artifact: artifacts[index] },
  };
  const started = performance.now();
  project = JSON.parse(wasm.wasm_gaji_lifecycle_plan_json(JSON.stringify(request))).project_source;
  installTimes.push(performance.now() - started);
}
const graph = JSON.parse(wasm.wasm_project_graph_json(JSON.stringify(project)));
assert.equal(Object.keys(graph.gaji_aliases).length, 20);

const secret = "CAPABILITY_SECRET_SHOULD_NOT_LEAK";
const stale = {
  schema: "ddn.gaji_lifecycle_plan.v1", source: project, expected_source_sha256: secret, expected_lock_sha256: null,
  operation: { kind: "remove", alias: "가지0" },
};
let staleMessage = "";
try { wasm.wasm_gaji_lifecycle_plan_json(JSON.stringify(stale)); } catch (error) { staleMessage = String(error); }
assert.match(staleMessage, /E_GAJI_LIFECYCLE_SOURCE_STALE/);
assert.equal(staleMessage.includes(secret), false);
assert.equal(staleMessage.includes(root), false);

console.log(JSON.stringify({
  schema: "ddn.gaji_attack_performance.v1", artifacts: 105, installed_gaji: 20, iterations: 30,
  search_ms: { p50: percentile(searchTimes, 0.5), p95: percentile(searchTimes, 0.95), max: Math.max(...searchTimes) },
  install_ms: { p50: percentile(installTimes, 0.5), p95: percentile(installTimes, 0.95), max: Math.max(...installTimes) },
  deterministic_order: true, secret_and_host_path_absent: true,
}));
