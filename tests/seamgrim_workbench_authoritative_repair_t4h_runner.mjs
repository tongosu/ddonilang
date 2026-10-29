#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import {
  analyzeSourceReadiness,
  applySourceReadinessFix,
  projectAuthoritativeSourceAnalysis,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/source_readiness.js";
import { buildDiagnosticFixitPreview } from "../solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_preview.js";

const root = process.cwd();
const sourceWasmDir = String(process.env.DDN_T4H_SOURCE_WASM_DIR ?? "").trim();
const trackedWasmDir = String(
  process.env.DDN_T4H_TRACKED_WASM_DIR
    ?? path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm"),
).trim();
assert(sourceWasmDir, "DDN_T4H_SOURCE_WASM_DIR is required");

async function loadAnalyzer(directory, tag) {
  const modulePath = path.join(directory, "ddonirang_tool.js");
  const wasmPath = path.join(directory, "ddonirang_tool_bg.wasm");
  assert(fs.existsSync(modulePath), `${tag} JS binding missing: ${modulePath}`);
  assert(fs.existsSync(wasmPath), `${tag} WASM missing: ${wasmPath}`);
  const module = await import(`${pathToFileURL(modulePath).href}?t4h=${tag}-${Date.now()}`);
  module.initSync({ module: fs.readFileSync(wasmPath) });
  return (sourceText, generation) => JSON.parse(module.wasm_source_analysis_snapshot_json(JSON.stringify({
    source_text: sourceText,
    path: "lesson.ddn",
    analysis_generation: generation,
    source_profile_identity: "strict",
  })));
}

const sourceAnalyze = await loadAnalyzer(sourceWasmDir, "source");
const trackedAnalyze = await loadAnalyzer(trackedWasmDir, "tracked");
const cases = [
  ["A_ASCII", "매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"],
  ["B_KOREAN_COMMENT", "// 앞한글\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"],
  ["C_NON_BMP_COMMENT", "// 😀\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"],
  ["D_COMBINING_COMMENT", "// e\u0301\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"],
  ["E_MULTIPLE", "매틱:움직씨 = { 채비: { 값:수 <- 1. }. 채비: { 둘:수 <- 2. }. }"],
  ["F_STRING_DECOY", "설정 { 제목: \"채비: {\". }.\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"],
  ["G_COMMENT_DECOY", "// 채비: { 주석 decoy\n매틱:움직씨 = { 채비: { 값:수 <- 1. }. }"],
  ["H_OPAQUE_DECOY", "매틱:움직씨 = { 채비 { 이름:글 <- (x=1) 글무늬{\"채비:\"}. }. 채비: { 값:수 <- 1. }. }"],
];

function expectedColonBytes(caseId, source) {
  const starts = caseId === "E_MULTIPLE"
    ? [...source.matchAll(/채비:/gu)].map((match) => match.index)
    : [source.lastIndexOf("채비:")];
  return starts.map((start) => Buffer.byteLength(source.slice(0, start + "채비".length), "utf8"));
}

function warnings(snapshot) {
  return snapshot.diagnostics.filter((row) => row.code === "W_BLOCK_HEADER_COLON_DEPRECATED");
}

const payloads = new Map();
for (const [caseId, source] of cases) {
  const generation = 200 + caseId.charCodeAt(0);
  const sourceSnapshot = sourceAnalyze(source, generation);
  const trackedSnapshot = trackedAnalyze(source, generation);
  assert.deepEqual(trackedSnapshot, sourceSnapshot, `${caseId}: source/tracked WASM payload drift`);
  assert.equal(sourceSnapshot.outcome, "ACCEPTED", `${caseId}: source rejected`);
  const rows = warnings(sourceSnapshot);
  const expected = expectedColonBytes(caseId, source);
  assert.equal(rows.length, expected.length, `${caseId}: warning count`);
  assert.equal(new Set(rows.map((row) => row.diagnostic_identity)).size, rows.length, `${caseId}: diagnostic identity collision`);
  assert.equal(new Set(rows.map((row) => row.repair_proposal?.proposal_identity)).size, rows.length, `${caseId}: proposal identity collision`);
  rows.forEach((row, index) => {
    const proposal = row.repair_proposal;
    assert(proposal, `${caseId}: repair proposal missing`);
    assert.equal(proposal.source_revision_identity, sourceSnapshot.source_revision_identity, `${caseId}: revision drift`);
    assert.equal(proposal.analysis_generation, generation, `${caseId}: generation drift`);
    assert.equal(proposal.diagnostic_identity, row.diagnostic_identity, `${caseId}: diagnostic binding drift`);
    assert.equal(proposal.original_source_span.coordinate_space, "original-source", `${caseId}: coordinate space`);
    assert.equal(proposal.original_source_span.encoding, "utf-8-byte", `${caseId}: encoding`);
    assert.equal(proposal.original_source_span.half_open, true, `${caseId}: half-open marker`);
    assert.equal(proposal.original_source_span.start, expected[index], `${caseId}: raw colon start`);
    assert.equal(proposal.original_source_span.end, expected[index] + 1, `${caseId}: raw colon end`);
    assert.equal(proposal.expected_text, ":", `${caseId}: expected text`);
    assert.equal(proposal.replacement_text, "", `${caseId}: replacement text`);
    assert(!JSON.stringify(proposal).includes("utf16"), `${caseId}: browser-only identity leaked into payload`);
  });
  const local = analyzeSourceReadiness(source);
  const projected = projectAuthoritativeSourceAnalysis(sourceSnapshot, {
    expectedAnalysisGeneration: generation,
    sourceHash: local.source_hash,
    sourceText: source,
  });
  assert.equal(projected.candidates.length, expected.length, `${caseId}: Workbench candidate count`);
  projected.candidates.forEach((candidate, index) => {
    assert.equal(candidate.replacement_authority, "rust_shared_frontend_repair_proposal", `${caseId}: replacement authority`);
    assert.equal(candidate.replacement_offset_source, "authoritative_original_source_span", `${caseId}: span authority`);
    assert.equal(candidate.repair_proposal_identity, rows[index].repair_proposal.proposal_identity, `${caseId}: proposal projection`);
  });
  payloads.set(caseId, { source, generation, snapshot: sourceSnapshot, projected });
}

const nonBmp = payloads.get("C_NON_BMP_COMMENT");
assert.equal(
  nonBmp.projected.candidate.start_offset_utf16,
  nonBmp.source.lastIndexOf("채비:") + "채비".length,
  "non-BMP UTF-8 to UTF-16 projection drift",
);

const multiple = payloads.get("E_MULTIPLE");
const secondApply = applySourceReadinessFix(multiple.source, multiple.projected.candidates[1]);
assert.equal(secondApply.status, "applied");
assert(secondApply.source.includes("채비: { 값:수"), "first occurrence changed by second proposal");
assert(secondApply.source.includes("채비 { 둘:수"), "second occurrence was not changed");

const stringCase = payloads.get("F_STRING_DECOY");
assert.equal(stringCase.snapshot.source_revision_identity, "blake3:3159cfef3cdbeeeb74ed267fe9cae2a12944661234c0a942b38c1339057abaab", "F: native/WASM revision seal drift");
assert.equal(stringCase.snapshot.diagnostics[0].diagnostic_identity, "ddn.source_analysis_diagnostic.candidate.v1:blake3:9ac895408889cb0cd9b7b2aaf73a98eb72637242512f8204fbdc9649c6e79e1f", "F: native/WASM diagnostic identity seal drift");
assert.equal(stringCase.snapshot.diagnostics[0].repair_proposal.proposal_identity, "ddn.source_analysis_repair_proposal.candidate.v1:blake3:b36a7c471fdcb01548dd56902ec27e449faeaf9bac72da74a7c67c5a248f7158", "F: native/WASM proposal identity seal drift");
const stringApply = applySourceReadinessFix(stringCase.source, stringCase.projected.candidate);
assert.equal(stringApply.status, "applied");
assert(stringApply.source.includes('제목: "채비: {"'), "string decoy was mutated");
assert(stringApply.source.includes("매틱:움직씨 = { 채비 {"), "real header was not repaired");
assert(!stringApply.source.includes("매틱:움직씨 = { 채비: {"), "deprecated real header remained");
assert.equal(
  applySourceReadinessFix(`${stringCase.source}\n`, stringCase.projected.candidate).status,
  "stale_rejected",
  "stale proposal was applied",
);
const reanalyzed = sourceAnalyze(stringApply.source, stringCase.generation + 1);
assert.notEqual(reanalyzed.source_revision_identity, stringCase.snapshot.source_revision_identity, "revision did not advance");
assert.equal(warnings(reanalyzed).length, 0, "diagnostic remained after authoritative reanalysis");

const studioPreview = buildDiagnosticFixitPreview({
  sourceText: stringCase.source,
  diagnostics: warnings(stringCase.snapshot),
});
assert.equal(studioPreview.fixit_count, 1, "Studio authoritative proposal preview missing");
assert(studioPreview.preview_text.includes('제목: "채비: {"'), "Studio preview changed string decoy");
assert(studioPreview.preview_text.includes("매틱:움직씨 = { 채비 {"), "Studio preview did not repair real header");
assert.equal(studioPreview.items[0].patch.reason, "authoritative_repair_proposal");

const unsupportedSnapshot = structuredClone(stringCase.snapshot);
unsupportedSnapshot.diagnostics[0].repair_proposal.schema = "unsupported";
const unavailable = projectAuthoritativeSourceAnalysis(unsupportedSnapshot, {
  expectedAnalysisGeneration: stringCase.generation,
  sourceHash: analyzeSourceReadiness(stringCase.source).source_hash,
  sourceText: stringCase.source,
});
assert.equal(unavailable.ready, true, "diagnostic was invalidated by unavailable repair");
assert.equal(unavailable.candidate, null, "unsupported proposal did not fail closed");

const readinessSource = fs.readFileSync(path.join(root, "solutions/seamgrim_ui_mvp/ui/workbench/source_readiness.js"), "utf8");
const studioSource = fs.readFileSync(path.join(root, "solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_preview.js"), "utf8");
assert(!/locateTemporaryChaebiRepairColon|skipRepairOpaqueTemplate|REPAIR_IDENTIFIER_CHAR/u.test(readinessSource), "active JS locator remains");
assert(!/replacement\s*:\s*""|expected_slice\s*:\s*":"/u.test(readinessSource), "JS replacement authority was reintroduced");
assert(!/lastIndexOf\(":"\)/u.test(readinessSource), "JS occurrence inference remains");
assert(/code === "W_BLOCK_HEADER_COLON_DEPRECATED"[\s\S]*findAuthoritativeRepairProposalPatch/u.test(studioSource), "Studio warning path does not consume proposal");
assert(!/code === "E_BLOCK_HEADER_COLON_FORBIDDEN"\s*\|\|\s*code === "W_BLOCK_HEADER_COLON_DEPRECATED"/u.test(studioSource), "Studio line-regex still owns warning repair");

console.log(JSON.stringify({
  schema: "seamgrim.workbench.authoritative_repair_t4h.runner_result.v1",
  fixtures: "8/8",
  multiple_occurrence: "PASS",
  string_wrong_apply: "0",
  non_bmp_utf16_projection: "PASS",
  source_tracked_wasm_payload_parity: "PASS",
  active_js_chaebi_locator_calls: 0,
  js_repair_replacement_invention: 0,
  stale_apply: 0,
  fuzzy_rebase: 0,
  auto_apply: 0,
  post_edit_reanalysis: "PASS",
  native_compatible_payload: true,
  result: "PASS",
}));
