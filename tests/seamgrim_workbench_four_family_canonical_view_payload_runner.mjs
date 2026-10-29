#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { extractCanonicalViewAuthorityInputsFromState } from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  buildCanonicalViewPayloadSet,
  buildCommittedSemanticViewPayload,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";
import { buildSemanticRenderModel } from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_renderer.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourceWasmDir = String(process.env.DDN_T5F_SOURCE_WASM_DIR ?? "").trim();
assert(sourceWasmDir, "DDN_T5F_SOURCE_WASM_DIR is required");

const wasmModule = await import(pathToFileURL(path.join(sourceWasmDir, "ddonirang_tool.js")).href);
const wrapper = await import(pathToFileURL(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm_ddn_wrapper.js")).href);
const wasmBytes = await fs.readFile(path.join(sourceWasmDir, "ddonirang_tool_bg.wasm"));
await wasmModule.default({ module_or_path: wasmBytes });

const lineSource = `설정 {
  제목: "정적 결과 그래프".
}.

채비 {
  a: 수 <- 2.
  b: 수 <- 1.
  x: 수 <- 2.
  y: 수 <- 5.
}.

x 보여주기.
y 보여주기.`;

const cases = [
  {
    family: "line",
    source: lineSource,
    ticks: 1,
  },
  {
    family: "pendulum",
    source: await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "lessons", "rep_phys_pendulum_wave1_v1", "lesson.ddn"), "utf8"),
    ticks: 2,
  },
  {
    family: "projectile",
    source: await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "lessons", "rep_phys_projectile_xy_v1", "lesson.ddn"), "utf8"),
    ticks: 3,
  },
  {
    family: "grid",
    source: await fs.readFile(path.join(root, "solutions", "seamgrim_ui_mvp", "lessons", "rep_grid_game_state_drop_v1", "lesson.ddn"), "utf8"),
    ticks: 1,
  },
];

function canonicalFor(family, state, frameId) {
  const inputs = extractCanonicalViewAuthorityInputsFromState(state);
  inputs.identity = {
    ...inputs.identity,
    state_hash: String(state?.state_hash ?? ""),
    source_hash: `sha256:t5f_${family}`,
    tick_id: Number(state?.tick_id ?? frameId),
  };
  return buildCanonicalViewPayloadSet(inputs);
}

function frameFromState(family, state, frameId) {
  return {
    frame_id: frameId,
    madi: Number(state?.tick_id ?? frameId),
    state_hash: String(state?.state_hash ?? ""),
    source_hash: `sha256:t5f_${family}`,
    wasm_state: state,
  };
}

async function execute(entry) {
  const vm = new wasmModule.DdnWasmVm(entry.source);
  const client = new wrapper.DdnWasmVmClient(vm);
  try {
    const states = [client.getStateParsed()];
    for (let index = 0; index < entry.ticks; index += 1) states.push(client.stepOneParsed());
    const frames = states.map((state, index) => frameFromState(entry.family, state, index));
    const run = { run_id: `run_t5f_${entry.family}`, source_hash: `sha256:t5f_${entry.family}`, frames };
    const canonicalHistory = frames.map((frame) => ({
      frame_id: frame.frame_id,
      madi: frame.madi,
      state_hash: frame.state_hash,
      canonical_views: canonicalFor(entry.family, frame.wasm_state, frame.frame_id),
    }));
    const frame = frames.at(-1);
    const canonicalViews = canonicalHistory.at(-1).canonical_views;
    const canonical = canonicalViews[`workbench_${entry.family}`];
    const resultSet = frame.wasm_state?.view_meta?.workbench;
    assert(resultSet?.schema === "seamgrim.workbench.authoritative_result_set.v1_candidate", `${entry.family}: Rust result-set schema missing`);
    assert(resultSet?.producer === "ddonirang_tool.wasm_runtime", `${entry.family}: Rust producer identity mismatch`);
    assert(canonical?.status === "available", `${entry.family}: canonical status ${canonical?.status}`);
    assert(canonical?.provenance?.classification === "AUTHORITATIVE_STRUCTURED", `${entry.family}: authority classification missing`);

    const semantic = buildCommittedSemanticViewPayload({
      project: { id: entry.family, primaryView: entry.family },
      run,
      frame,
      viewConfig: { primary_view: entry.family, role: "primary", mode: "ddn-main" },
      canonicalViewPayloads: canonicalViews,
      canonicalViewHistory: canonicalHistory,
    });
    const renderer = buildSemanticRenderModel(semantic);
    assert(renderer.status === "ready", `${entry.family}: renderer status ${renderer.status}`);
    assert(renderer.data_source === "canonical_workbench_result", `${entry.family}: raw-state renderer authority remains`);

    const runProjection = stableSemanticJson(canonical);
    const workbenchProjection = stableSemanticJson(canonicalFor(entry.family, frame.wasm_state, frame.frame_id)[`workbench_${entry.family}`]);
    const playgroundProjection = stableSemanticJson(canonicalFor(entry.family, frame.wasm_state, frame.frame_id)[`workbench_${entry.family}`]);
    assert(runProjection === workbenchProjection && workbenchProjection === playgroundProjection, `${entry.family}: cross-surface canonical byte parity failed`);

    return {
      family: entry.family,
      rust_result_schema: resultSet.results[entry.family].schema,
      canonical_family: canonical.family,
      canonical_status: canonical.status,
      canonical_hash: canonical.candidate_payload_hash,
      renderer_status: renderer.status,
      renderer_data_source: renderer.data_source,
      history_count: semantic.committed_trajectory.length,
      state_hash: frame.state_hash,
    };
  } finally {
    vm.free();
  }
}

const summary = [];
for (const entry of cases) summary.push(await execute(entry));

const expectedPath = path.join(root, "pack", "seamgrim_workbench_four_family_canonical_view_payload_v1", "expected.detjson");
if (process.argv.includes("--write-expected")) {
  await fs.mkdir(path.dirname(expectedPath), { recursive: true });
  await fs.writeFile(expectedPath, `${JSON.stringify(summary, null, 2)}\n`, "utf8");
  console.log(`wrote ${expectedPath}`);
} else {
  const expected = JSON.parse(await fs.readFile(expectedPath, "utf8"));
  assert(stableSemanticJson(summary) === stableSemanticJson(expected), "four-family canonical golden mismatch");
}

console.log(`seamgrim_workbench_four_family_canonical_view_payload: ok (${summary.length}/4)`);
