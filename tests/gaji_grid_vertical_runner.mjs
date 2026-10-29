import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const root = path.resolve(import.meta.dirname, "..");
const wasmDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm");
const wasm = await import(pathToFileURL(path.join(wasmDir, "ddonirang_tool.js")));
await wasm.default(fs.readFileSync(path.join(wasmDir, "ddonirang_tool_bg.wasm")));
const source = JSON.parse(fs.readFileSync(path.join(root, "pack", "ddn_local_project_graph_v1", "fixtures", "project_run_grid_v1.json"), "utf8"));

function artifactRequest(projectSource, { id, version, originGaji = null, relation = "extracted" }) {
  return {
    schema: "ddn.gaji_extraction_plan.v1", source: projectSource, selected_modules: ["lib/grid.ddn"],
    gaji_id: id, version, description: "격자 시작 규칙", tags: ["격자", "게임"], profile: "내",
    origin_project_ref: "project:grid-vertical", origin_gaji: originGaji, relation,
  };
}
function sourceHash(projectSource) { return wasm.wasm_project_source_sha256_json(JSON.stringify(projectSource)); }
function lifecycle(projectSource, operation) {
  return JSON.parse(wasm.wasm_gaji_lifecycle_plan_json(JSON.stringify({
    schema: "ddn.gaji_lifecycle_plan.v1", source: projectSource,
    expected_source_sha256: sourceHash(projectSource), expected_lock_sha256: null, operation,
  })));
}
function consumer(id, scoreAlias = "규칙") {
  return {
    schema: "ddn.virtual_project_source.v1", kind: "seamgrim.local_project_tree.v0", root_path: "뿌리.ddn", entry_path: "app/main.ddn",
    files: [
      { path: "뿌리.ddn", role: "root", utf8_text: '프로젝트 { 이름: "격자 소비자". 첫글: "./app/main.ddn". 목표 { 앱: "./app/main.ddn". } 모듈길 { 로컬: "./lib". } }.' },
      { path: "app/main.ddn", role: "ddn_source", utf8_text: `쓰임 { ${scoreAlias}: "./vendor/${id}/ddn/lib/grid". }\n${scoreAlias}.시작점수 보여주기.` },
    ],
  };
}
function run(projectSource) { return JSON.parse(wasm.wasm_project_run_summary_json(JSON.stringify(projectSource), "앱", true, true)); }

const original = JSON.parse(wasm.wasm_local_gaji_artifact_json(JSON.stringify(artifactRequest(source, { id: "격자규칙", version: "1.0.0" }))));
let store = { schema: "ddn.local_gotgan_store.v1", artifacts: [] };
store = JSON.parse(wasm.wasm_publish_local_gotgan_json(JSON.stringify({ schema: "ddn.local_gotgan_publish_request.v1", store, artifact: original })));
const installed = lifecycle(consumer("격자규칙"), { kind: "install", alias: "규칙", artifact: original });
assert.deepEqual(run(installed.project_source).output_rows, ["3"]);

const remixSource = structuredClone(source);
remixSource.files.find((file) => file.path === "lib/grid.ddn").utf8_text = "드러냄 { 시작점수. }\n시작점수 <- 5.";
const remix = JSON.parse(wasm.wasm_local_gaji_artifact_json(JSON.stringify(artifactRequest(remixSource, {
  id: "격자규칙-내버전", version: "1.0.0", originGaji: "격자규칙@1.0.0", relation: "remix",
}))));
store = JSON.parse(wasm.wasm_publish_local_gotgan_json(JSON.stringify({ schema: "ddn.local_gotgan_publish_request.v1", store, artifact: remix })));
assert.equal(store.artifacts.find((item) => item.gaji_id === "격자규칙").package_sha256, original.package_sha256);
assert.notEqual(remix.content_sha256, original.content_sha256);
assert.equal(remix.provenance.origin_gaji, "격자규칙@1.0.0");

const remixInstalled = lifecycle(consumer("격자규칙-내버전", "내규칙"), { kind: "install", alias: "내규칙", artifact: remix });
assert.deepEqual(run(remixInstalled.project_source).output_rows, ["5"]);
assert.throws(() => lifecycle(remixInstalled.project_source, { kind: "remove", alias: "내규칙" }), /E_GAJI_REMOVE_REFERENCED.*app\/main\.ddn/);
const unused = structuredClone(remixInstalled.project_source);
unused.files.find((file) => file.path === "app/main.ddn").utf8_text = '"격자 규칙 분리" 보여주기.';
const removed = lifecycle(unused, { kind: "remove", alias: "내규칙" });
assert.equal(removed.project_source.files.some((file) => file.path.startsWith("vendor/격자규칙-내버전/")), false);
assert.equal(removed.after_lock_sha256, null);

console.log("gaji_grid_vertical: ok");
