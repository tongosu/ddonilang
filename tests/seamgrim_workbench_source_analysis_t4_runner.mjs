#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  analyzeSourceReadiness,
  projectAuthoritativeSourceAnalysis,
  projectSourceAnalysisUnavailable,
  SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
  SOURCE_ANALYSIS_PROFILE_IDENTITY,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/source_readiness.js";
import {
  RuntimeBridgeV1,
  WasmRuntimeAdapter,
} from "../solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), "utf8");

function snapshot({ outcome = "ACCEPTED", generation = 4, diagnostics = [] } = {}) {
  return {
    schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
    source_revision_identity: "blake3:authoritative-revision",
    analysis_generation: generation,
    source_profile_identity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
    outcome,
    diagnostics,
  };
}

const accepted = projectAuthoritativeSourceAnalysis(snapshot(), {
  expectedAnalysisGeneration: 4,
  sourceHash: "candidate:fnv1a32:valid",
});
assert.equal(accepted.ready, true);
assert.equal(accepted.blocked, false);
assert.equal(accepted.authority_status, "shared_frontend");
assert.equal(accepted.source_revision_identity, "blake3:authoritative-revision");

const warning = projectAuthoritativeSourceAnalysis(snapshot({
  diagnostics: [{
    code: "W_FRONTEND_SENTINEL",
    severity: "warning",
    message: "warning sentinel",
    path: "lesson.ddn",
    span: { encoding: "utf-8-byte", start: 3, end: 6 },
  }],
}), { expectedAnalysisGeneration: 4 });
assert.equal(warning.ready, true, "LANGUAGE_WARNING must not block execution");
assert.equal(warning.diagnostics[0].status_axis, "LANGUAGE_WARNING");
assert.equal(warning.diagnostics[0].code, "W_FRONTEND_SENTINEL");

const rejected = projectAuthoritativeSourceAnalysis(snapshot({
  outcome: "REJECTED",
  diagnostics: [{
    code: "E_PARSE_SENTINEL",
    severity: "error",
    message: "rejected sentinel",
    path: "lesson.ddn",
    span: { encoding: "utf-8-byte", start: 7, end: 10 },
  }],
}), { expectedAnalysisGeneration: 4 });
assert.equal(rejected.ready, false);
assert.equal(rejected.blocked, true);
assert.equal(rejected.diagnostics[0].status_axis, "LANGUAGE_ERROR");
assert.equal(rejected.diagnostics[0].code, "E_PARSE_SENTINEL", "JS changed authoritative code");
assert.deepEqual(rejected.diagnostics[0].span, { encoding: "utf-8-byte", start: 7, end: 10 });

const stale = projectAuthoritativeSourceAnalysis(snapshot({ generation: 3 }), {
  expectedAnalysisGeneration: 4,
});
assert.equal(stale.stale, true, "revision/generation mismatch must be rejected");
assert.equal(stale.ready, false);

const productReadiness = projectSourceAnalysisUnavailable(new Error("frontend unavailable"), {
  analysisGeneration: 5,
});
assert.equal(productReadiness.ready, false);
assert.equal(productReadiness.diagnostics[0].status_axis, "PRODUCT_READINESS");
assert.equal(productReadiness.diagnostics[0].authority, "workbench_product");

const chaebiSource = "매틱:움직씨 = { 채비: { 값:수 <- 1. }. }";
const chaebiIdentity = analyzeSourceReadiness(chaebiSource);
assert.equal(chaebiIdentity.authority_status, "awaiting_shared_frontend");
assert.equal(chaebiIdentity.ready, true, "JS must not block before shared frontend analysis");
assert.equal(chaebiIdentity.diagnostics.length, 0);
const chaebiStart = Buffer.byteLength(chaebiSource.slice(0, chaebiSource.indexOf("채비:")), "utf8");
const chaebiColon = chaebiStart + Buffer.byteLength("채비", "utf8");
const chaebiDiagnosticIdentity = "ddn.source_analysis_diagnostic.candidate.v1:blake3:t4";
const authoritativeChaebi = projectAuthoritativeSourceAnalysis(snapshot({
  diagnostics: [{
    code: "W_BLOCK_HEADER_COLON_DEPRECATED",
    severity: "warning",
    message: "블록 헤더의 `채비:` 표기는 예정된 비권장입니다.",
    path: "lesson.ddn",
    span: {
      encoding: "utf-8-byte",
      start: chaebiStart,
      end: chaebiStart + Buffer.byteLength("채비:", "utf8"),
    },
    diagnostic_identity: chaebiDiagnosticIdentity,
    repair_proposal: {
      schema: "ddn.source_analysis_repair_proposal.candidate.v1",
      fix_id: "FIX_SOURCE_REMOVE_SETUP_COLON",
      source_revision_identity: "blake3:authoritative-revision",
      analysis_generation: 4,
      diagnostic_identity: chaebiDiagnosticIdentity,
      original_source_span: {
        coordinate_space: "original-source",
        encoding: "utf-8-byte",
        half_open: true,
        start: chaebiColon,
        end: chaebiColon + 1,
      },
      expected_text: ":",
      replacement_text: "",
      proposal_identity: "ddn.source_analysis_repair_proposal.candidate.v1:blake3:t4",
      explanation: "비권장 블록 헤더의 콜론을 제거합니다.",
    },
  }],
}), {
  expectedAnalysisGeneration: 4,
  sourceHash: chaebiIdentity.source_hash,
  sourceText: chaebiSource,
});
assert.equal(authoritativeChaebi.ready, true);
assert.equal(authoritativeChaebi.blocked, false);
assert.equal(authoritativeChaebi.diagnostics[0].status_axis, "LANGUAGE_WARNING");
assert.equal(authoritativeChaebi.candidate.trigger_authority, "shared_frontend_diagnostic");
assert.equal(authoritativeChaebi.candidate.replacement_authority, "rust_shared_frontend_repair_proposal");

const bridgeInput = Object.freeze({
  source_text: "값:수 <- 1.",
  path: "lesson.ddn",
  analysis_generation: 11,
  source_profile_identity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
});
const bridgeSnapshot = Object.freeze(snapshot({ generation: 11 }));
const calls = [];
const bridge = new RuntimeBridgeV1({
  adapter: {
    sourceAnalysis(request) {
      calls.push(request);
      return bridgeSnapshot;
    },
  },
});
assert.equal(await bridge.sourceAnalysis(bridgeInput), bridgeSnapshot, "snapshot identity changed");
assert.equal(calls[0], bridgeInput, "analysis request identity changed");

const wasmCalls = [];
const wasmModule = {
  wasm_project_prepare_link_json() { return "{}"; },
  wasm_project_edit_plan_json() { return "{}"; },
  wasm_project_source_sha256_json() { return "sha256"; },
  wasm_source_analysis_snapshot_json(requestJson) {
    wasmCalls.push(JSON.parse(requestJson));
    return JSON.stringify(bridgeSnapshot);
  },
};
const adapter = new WasmRuntimeAdapter({
  createWasmVmImpl: async () => ({ client: { wasmModule } }),
});
assert.deepEqual(await adapter.sourceAnalysis(bridgeInput), bridgeSnapshot);
assert.deepEqual(wasmCalls, [bridgeInput]);

const noQueryAdapter = new WasmRuntimeAdapter({
  createWasmVmImpl: async () => ({
    client: {
      wasmModule: {
        wasm_project_prepare_link_json() { return "{}"; },
        wasm_project_edit_plan_json() { return "{}"; },
        wasm_project_source_sha256_json() { return "sha256"; },
      },
    },
  }),
});
await assert.rejects(
  () => noQueryAdapter.sourceAnalysis(bridgeInput),
  /E_SOURCE_ANALYSIS_WASM_REQUIRED/u,
  "missing authoritative query must not fall back",
);

const workbenchText = read("solutions/seamgrim_ui_mvp/ui/workbench/workbench.js");
const sourceReadinessText = read("solutions/seamgrim_ui_mvp/ui/workbench/source_readiness.js");
const runtimeBridgeText = read("solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js");
const hostBridgeText = read("solutions/seamgrim_ui_mvp/ui/host/host_bridge.js");
assert.match(workbenchText, /await runtimeBridge\.sourceAnalysis\(/u);
assert.match(workbenchText, /sourceStillCurrent/u);
assert.match(workbenchText, /generationStillCurrent/u);
assert.doesNotMatch(workbenchText, /\bcreateWasmVm\s*\(/u);
assert.doesNotMatch(runtimeBridgeText, /findLegacySetupColon|PRODUCT_READINESS/u);
assert.doesNotMatch(hostBridgeText, /sourceAnalysis|source_readiness|LANGUAGE_ERROR/u);
assert.doesNotMatch(sourceReadinessText, /findLegacySetupColon|E_SOURCE_LEGACY_SETUP_COLON/u);

let sourceBuiltWasm = "not_requested";
const sourceWasmDir = String(process.env.DDN_T4_SOURCE_WASM_DIR ?? "").trim();
if (sourceWasmDir) {
  const modulePath = path.join(sourceWasmDir, "ddonirang_tool.js");
  const wasmPath = path.join(sourceWasmDir, "ddonirang_tool_bg.wasm");
  const module = await import(`${pathToFileURL(modulePath).href}?t4=${Date.now()}`);
  module.initSync({ module: fs.readFileSync(wasmPath) });
  const analyze = (sourceText, analysisGeneration) => JSON.parse(
    module.wasm_source_analysis_snapshot_json(JSON.stringify({
      source_text: sourceText,
      path: "lesson.ddn",
      analysis_generation: analysisGeneration,
      source_profile_identity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
    })),
  );
  const sourceAccepted = analyze("매틱:움직씨 = { 채비 { 값:수 <- 0. }. }", 21);
  const sourceRejected = analyze("매틱:움직씨 = { 값:수 <- . }", 22);
  const sourceWarning = analyze("매틱:움직씨 = { 채비: { 값:수 <- 0. }. }", 23);
  assert.equal(sourceAccepted.outcome, "ACCEPTED");
  assert.equal(sourceRejected.outcome, "REJECTED");
  assert.equal(sourceRejected.diagnostics[0].severity, "error");
  assert.equal(sourceWarning.outcome, "ACCEPTED");
  assert.ok(sourceWarning.diagnostics.some((row) => row.code === "W_BLOCK_HEADER_COLON_DEPRECATED"));
  sourceBuiltWasm = "PASS";
}

console.log(JSON.stringify({
  schema: "seamgrim.workbench.source_analysis_t4.runner_result.v1",
  authoritative_query_consumers_before: 1,
  authoritative_query_consumers_after: 1,
  js_language_validity_rules_before: 1,
  js_language_validity_rules_after: 0,
  semantic_fallbacks_before: 7,
  semantic_fallbacks_after: 7,
  chaebi_rules_migrated: 1,
  chaebi_rules_pending: 0,
  high_parity_risks_before: 20,
  high_parity_risks_after: 19,
  stale_guard: "source_revision_and_analysis_generation_fail_closed",
  new_js_semantic_rules: 0,
  new_semantic_fallbacks: 0,
  source_built_wasm: sourceBuiltWasm,
  result: "PASS",
}));
