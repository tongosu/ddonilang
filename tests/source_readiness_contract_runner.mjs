#!/usr/bin/env node
import assert from "node:assert/strict";
import {
  analyzeSourceReadiness,
  applySourceReadinessFix,
  projectAuthoritativeSourceAnalysis,
  sharedRunSummaryBindingValid,
  REPAIR_PROPOSAL_SCHEMA,
  SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
  SOURCE_ANALYSIS_PROFILE_IDENTITY,
  SOURCE_READINESS_DIAGNOSTIC_CODE,
  SOURCE_READINESS_FIX_ID,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/source_readiness.js";

const source = [
  '설정 { 제목: "R10". }.',
  "설명: \"진단 복구\".",
  "값:수 <- 1.",
  "// 채비: { 문자열/주석은 무시",
  '글무늬 { "채비: {" }',
  "# 보존 줄.",
  "채비: {",
  "  값 보여주기.",
  "}.",
].join("\n");

const localIdentity = analyzeSourceReadiness(source, "lesson.ddn");
assert.equal(localIdentity.ready, true);
assert.equal(localIdentity.blocked, false);
assert.equal(localIdentity.authority_status, "awaiting_shared_frontend");
assert.equal(localIdentity.diagnostics.length, 0, "JS must not classify 채비: language validity");
assert.equal(localIdentity.candidate, null, "JS must not create a repair without an authoritative warning");

const keywordOffset = source.lastIndexOf("채비:");
const warningStart = Buffer.byteLength(source.slice(0, keywordOffset), "utf8");
const warningEnd = warningStart + Buffer.byteLength("채비:", "utf8");
const colonStart = warningStart + Buffer.byteLength("채비", "utf8");
const diagnosticIdentity = "ddn.source_analysis_diagnostic.candidate.v1:blake3:contract";
const snapshot = {
  schema: SOURCE_ANALYSIS_SNAPSHOT_SCHEMA,
  source_revision_identity: "blake3:authoritative-chaebi-revision",
  analysis_generation: 17,
  source_profile_identity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
  outcome: "ACCEPTED",
  diagnostics: [{
    code: SOURCE_READINESS_DIAGNOSTIC_CODE,
    severity: "warning",
    message: "블록 헤더의 `채비:` 표기는 예정된 비권장입니다. `채비 {` 표기로 전환하세요",
    path: "lesson.ddn",
    span: { encoding: "utf-8-byte", start: warningStart, end: warningEnd },
    diagnostic_identity: diagnosticIdentity,
    repair_proposal: {
      schema: REPAIR_PROPOSAL_SCHEMA,
      fix_id: SOURCE_READINESS_FIX_ID,
      source_revision_identity: "blake3:authoritative-chaebi-revision",
      analysis_generation: 17,
      diagnostic_identity: diagnosticIdentity,
      original_source_span: {
        coordinate_space: "original-source",
        encoding: "utf-8-byte",
        half_open: true,
        start: colonStart,
        end: colonStart + 1,
      },
      expected_text: ":",
      replacement_text: "",
      proposal_identity: "ddn.source_analysis_repair_proposal.candidate.v1:blake3:contract",
      explanation: "비권장 블록 헤더의 콜론을 제거합니다.",
    },
  }],
};
const readiness = projectAuthoritativeSourceAnalysis(snapshot, {
  expectedAnalysisGeneration: 17,
  sourceHash: localIdentity.source_hash,
  sourceText: source,
});
assert.equal(readiness.ready, true);
assert.equal(readiness.blocked, false);
assert.equal(readiness.authority_status, "shared_frontend");
assert.equal(readiness.analysis_outcome, "ACCEPTED");
assert.equal(readiness.diagnostics[0].code, SOURCE_READINESS_DIAGNOSTIC_CODE);
assert.equal(readiness.diagnostics[0].severity, "warning");
assert.equal(readiness.diagnostics[0].status_axis, "LANGUAGE_WARNING");
assert.equal(readiness.diagnostics[0].fix_it.id, SOURCE_READINESS_FIX_ID);
assert.equal(readiness.candidate.trigger_authority, "shared_frontend_diagnostic");
assert.equal(
  readiness.candidate.replacement_authority,
  "rust_shared_frontend_repair_proposal",
);
assert.equal(readiness.candidate.replacement_offset_source, "authoritative_original_source_span");
assert.equal(readiness.candidates.length, 1);
assert.equal(readiness.candidate.source_revision_identity, snapshot.source_revision_identity);
assert.equal(readiness.candidate.analysis_generation, snapshot.analysis_generation);
assert.equal(readiness.candidate.start_offset_utf16, source.lastIndexOf(":"));

const wrongProfile = projectAuthoritativeSourceAnalysis({
  ...snapshot,
  source_profile_identity: "wrong-profile",
  diagnostics: [],
}, {
  expectedAnalysisGeneration: 17,
  sourceHash: localIdentity.source_hash,
  sourceText: source,
});
assert.equal(wrongProfile.ready, false, "wrong source profile must not authorize readiness");
assert.equal(wrongProfile.blocked, true, "wrong source profile must fail closed");
assert.equal(wrongProfile.authority_status, "shared_frontend_unavailable");
assert.equal(wrongProfile.diagnostics[0].code, "E_SOURCE_ANALYSIS_UNAVAILABLE");

const profilePrefix = projectAuthoritativeSourceAnalysis({
  ...snapshot,
  source_profile_identity: `${SOURCE_ANALYSIS_PROFILE_IDENTITY}-compat`,
  diagnostics: [],
}, {
  expectedAnalysisGeneration: 17,
  sourceHash: localIdentity.source_hash,
  sourceText: source,
});
assert.equal(profilePrefix.ready, false, "profile prefixes must not become aliases");
assert.equal(profilePrefix.blocked, true, "profile prefixes must fail closed");

const sharedSummary = {
  source_profile_identity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
  execution_envelope: { source_bytes_sha256: "a".repeat(64) },
  state_hash: "blake3:committed-state",
  work_cost_receipt: { schema: "ddn.work_cost_receipt.v1" },
};
const {normalizeWasmStatePayload}=await import('../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js');
const delivered={state_hash:sharedSummary.state_hash,temporary_committed_summary:sharedSummary};
assert.strictEqual(normalizeWasmStatePayload(JSON.stringify(delivered)).temporary_committed_summary.state_hash,sharedSummary.state_hash);
assert.equal(Object.hasOwn(normalizeWasmStatePayload({state_hash:'unchanged'}),'temporary_committed_summary'),false);
assert.equal(sharedRunSummaryBindingValid(sharedSummary, {
  sourceProfileIdentity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
  sourceBytesSha256: "a".repeat(64),
  stateHash: "blake3:committed-state",
}), true, "exact shared run summary binding must remain valid");
for (const [label, mutated] of [
  ["source", { ...sharedSummary, execution_envelope: { source_bytes_sha256: "b".repeat(64) } }],
  ["profile", { ...sharedSummary, source_profile_identity: `${SOURCE_ANALYSIS_PROFILE_IDENTITY}-compat` }],
  ["state", { ...sharedSummary, state_hash: "blake3:stale-state" }],
  ["receipt", { ...sharedSummary, work_cost_receipt: null }],
]) {
  assert.equal(sharedRunSummaryBindingValid(mutated, {
    sourceProfileIdentity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
    sourceBytesSha256: "a".repeat(64),
    stateHash: "blake3:committed-state",
  }), false, `shared run summary ${label} mismatch must fail closed`);
}

const applied = applySourceReadinessFix(source, readiness.candidate);
assert.equal(applied.status, "applied");
assert.equal(applied.changed, true);
assert.equal(applied.source.split("\n")[6], "채비 {");
assert.equal(applySourceReadinessFix(applied.source, readiness.candidate).status, "stale_rejected");
assert.equal(applySourceReadinessFix(applied.source, null).status, "no_op");

for (const negative of [
  '문자열: "채비: {".',
  "// 채비: { 주석",
  "값:수 <- 1.",
  "글무늬 { 채비: { 내부 템플릿 }. }",
  "채비 {",
]) {
  const result = analyzeSourceReadiness(negative);
  assert.equal(result.ready, true, `local identity unexpectedly blocked: ${negative}`);
  assert.equal(result.diagnostics.length, 0);
  const authoritativeNoWarning = projectAuthoritativeSourceAnalysis({ ...snapshot, diagnostics: [] }, {
    expectedAnalysisGeneration: 17,
    sourceHash: result.source_hash,
    sourceText: negative,
  });
  assert.equal(authoritativeNoWarning.candidate, null, `fix triggered without authoritative warning: ${negative}`);
}

const mutatedError = projectAuthoritativeSourceAnalysis({
  ...snapshot,
  diagnostics: [{ ...snapshot.diagnostics[0], severity: "error" }],
}, {
  expectedAnalysisGeneration: 17,
  sourceHash: localIdentity.source_hash,
  sourceText: source,
});
assert.equal(mutatedError.ready, false, "error mutation must be detected as a run block");
assert.equal(mutatedError.candidate, null, "error mutation must not own the warning repair trigger");

const wrongCode = projectAuthoritativeSourceAnalysis({
  ...snapshot,
  diagnostics: [{ ...snapshot.diagnostics[0], code: "W_OTHER" }],
}, {
  expectedAnalysisGeneration: 17,
  sourceHash: localIdentity.source_hash,
  sourceText: source,
});
assert.equal(wrongCode.ready, true);
assert.equal(wrongCode.candidate, null, "non-authoritative code triggered the R10 repair");

const unsupportedProposal = projectAuthoritativeSourceAnalysis({
  ...snapshot,
  diagnostics: [{
    ...snapshot.diagnostics[0],
    repair_proposal: { ...snapshot.diagnostics[0].repair_proposal, schema: "unknown" },
  }],
}, {
  expectedAnalysisGeneration: 17,
  sourceHash: localIdentity.source_hash,
  sourceText: source,
});
assert.equal(unsupportedProposal.ready, true, "diagnostic must remain valid when repair is unavailable");
assert.equal(unsupportedProposal.candidate, null, "unsupported repair schema must fail closed");

console.log(JSON.stringify({
  schema: "seamgrim.source_readiness.contract_runner.v2",
  diagnostic_code: SOURCE_READINESS_DIAGNOSTIC_CODE,
  fix_it_id: SOURCE_READINESS_FIX_ID,
  authoritative_span: readiness.diagnostics[0].span,
  trigger_authority: readiness.candidate.trigger_authority,
  replacement_authority: readiness.candidate.replacement_authority,
  run_blocked: readiness.blocked,
  applied_status: applied.status,
  stale_status: applySourceReadinessFix(applied.source, readiness.candidate).status,
  negative_cases: 5,
  js_language_validity_rules: 0,
}));
console.log("source_readiness_contract: ok");
