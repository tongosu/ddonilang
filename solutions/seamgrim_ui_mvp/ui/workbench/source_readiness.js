export const SOURCE_READINESS_SCHEMA = "seamgrim.workbench.source_readiness.v1";
export const SOURCE_READINESS_DIAGNOSTIC_CODE = "W_BLOCK_HEADER_COLON_DEPRECATED";
export const SOURCE_READINESS_FIX_ID = "FIX_SOURCE_REMOVE_SETUP_COLON";
export const SOURCE_ANALYSIS_SNAPSHOT_SCHEMA = "ddn.source_analysis_snapshot.candidate.v1";
export const MODULE_DIRECTIVE_ANALYSIS_SNAPSHOT_SCHEMA = "ddn.module_directive_analysis.candidate.v1";
// Workbench single-file execution is the V1 supported ingress. Keep the
// profile explicit so the preflight and the shared Rust/WASM runtime accept
// exactly the same V25 source surface as the official native frontdoor.
export const SOURCE_ANALYSIS_PROFILE_IDENTITY = "v1-core-v25";
export const REPAIR_PROPOSAL_SCHEMA = "ddn.source_analysis_repair_proposal.candidate.v1";
const UTF8_ENCODER = new TextEncoder();

function stableSourceHash(source) {
  let hash = 0x811c9dc5;
  for (let index = 0; index < source.length; index += 1) {
    hash ^= source.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return `candidate:fnv1a32:${(hash >>> 0).toString(16).padStart(8, "0")}`;
}

export function analyzeSourceReadiness(sourceText, path = "lesson.ddn") {
  const source = String(sourceText ?? "");
  return {
    schema: SOURCE_READINESS_SCHEMA,
    policy_layer: "workbench_source_identity",
    authority_status: "awaiting_shared_frontend",
    ready: true,
    blocked: false,
    source_hash: stableSourceHash(source),
    path: String(path),
    diagnostics: [],
    candidate: null,
    candidates: [],
  };
}

function utf8ByteOffsetToUtf16(source, byteOffset) {
  const target = Number(byteOffset);
  if (!Number.isInteger(target) || target < 0) return -1;
  let bytes = 0;
  let utf16 = 0;
  for (const character of source) {
    if (bytes === target) return utf16;
    bytes += UTF8_ENCODER.encode(character).length;
    utf16 += character.length;
    if (bytes > target) return -1;
  }
  return bytes === target ? utf16 : -1;
}

function buildAuthoritativeWarningFix(source, sourceHash, snapshot, diagnostic) {
  const repair = diagnostic?.repair_proposal;
  const originalSpan = repair?.original_source_span;
  if (
    diagnostic.code !== SOURCE_READINESS_DIAGNOSTIC_CODE
    || diagnostic.severity !== "warning"
    || !repair
    || repair.schema !== REPAIR_PROPOSAL_SCHEMA
    || !String(diagnostic.diagnostic_identity ?? "")
    || repair.diagnostic_identity !== diagnostic.diagnostic_identity
    || repair.source_revision_identity !== snapshot.source_revision_identity
    || Number(repair.analysis_generation) !== Number(snapshot.analysis_generation)
    || !String(repair.proposal_identity ?? "")
    || !String(repair.fix_id ?? "")
    || originalSpan?.coordinate_space !== "original-source"
    || originalSpan?.encoding !== "utf-8-byte"
    || originalSpan?.half_open !== true
    || !Number.isInteger(originalSpan?.start)
    || !Number.isInteger(originalSpan?.end)
    || typeof repair.expected_text !== "string"
    || repair.expected_text.length === 0
    || typeof repair.replacement_text !== "string"
  ) {
    return null;
  }
  const spanStart = utf8ByteOffsetToUtf16(source, originalSpan.start);
  const spanEnd = utf8ByteOffsetToUtf16(source, originalSpan.end);
  if (
    spanStart < 0
    || spanEnd <= spanStart
    || source.slice(spanStart, spanEnd) !== repair.expected_text
  ) return null;
  return {
    id: repair.fix_id,
    trigger_authority: "shared_frontend_diagnostic",
    trigger_code: diagnostic.code,
    trigger_severity: diagnostic.severity,
    diagnostic_identity: repair.diagnostic_identity,
    repair_proposal_identity: repair.proposal_identity,
    replacement_authority: "rust_shared_frontend_repair_proposal",
    replacement_offset_source: "authoritative_original_source_span",
    source_revision_identity: repair.source_revision_identity,
    analysis_generation: Number(repair.analysis_generation),
    source_hash: sourceHash,
    expected_slice: repair.expected_text,
    replacement: repair.replacement_text,
    start_offset_utf16: spanStart,
    end_offset_utf16: spanEnd,
    original_source: source,
    path: diagnostic.path,
    span: { ...originalSpan },
    diagnostic_span: diagnostic.span ? { ...diagnostic.span } : null,
  };
}

function cloneDiagnostic(row, statusAxis) {
  return {
    code: String(row?.code ?? ""),
    severity: String(row?.severity ?? "error"),
    message: String(row?.message ?? ""),
    path: String(row?.path ?? ""),
    span: row?.span ? JSON.parse(JSON.stringify(row.span)) : null,
    diagnostic_identity: String(row?.diagnostic_identity ?? ""),
    repair_proposal: row?.repair_proposal ? JSON.parse(JSON.stringify(row.repair_proposal)) : null,
    status_axis: statusAxis,
    authority: "shared_frontend",
  };
}

export function projectAuthoritativeSourceAnalysis(snapshot, {
  expectedAnalysisGeneration,
  expectedSourceProfileIdentity = SOURCE_ANALYSIS_PROFILE_IDENTITY,
  sourceHash = "",
  sourceText = "",
} = {}) {
  if (
    !snapshot
    || ![SOURCE_ANALYSIS_SNAPSHOT_SCHEMA, MODULE_DIRECTIVE_ANALYSIS_SNAPSHOT_SCHEMA]
      .includes(snapshot.schema)
  ) {
    throw new Error("E_SOURCE_ANALYSIS_SNAPSHOT_INVALID");
  }
  if (Number(snapshot.analysis_generation) !== Number(expectedAnalysisGeneration)) {
    return {
      schema: SOURCE_READINESS_SCHEMA,
      policy_layer: "workbench_source_readiness_projection",
      authority_status: "shared_frontend_stale_rejected",
      ready: false,
      blocked: true,
      stale: true,
      source_hash: String(sourceHash),
      source_revision_identity: String(snapshot.source_revision_identity ?? ""),
      analysis_generation: Number(snapshot.analysis_generation),
      source_profile_identity: String(snapshot.source_profile_identity ?? ""),
      analysis_outcome: String(snapshot.outcome ?? "UNKNOWN"),
      diagnostics: [],
      candidate: null,
      candidates: [],
    };
  }
  if (String(snapshot.source_profile_identity ?? "") !== String(expectedSourceProfileIdentity)) {
    return projectSourceAnalysisUnavailable("E_SOURCE_ANALYSIS_PROFILE_MISMATCH", {
      analysisGeneration: expectedAnalysisGeneration,
      sourceHash,
    });
  }
  const outcome = String(snapshot.outcome ?? "UNKNOWN");
  if (!["ACCEPTED", "REJECTED", "UNSUPPORTED"].includes(outcome)) {
    throw new Error("E_SOURCE_ANALYSIS_OUTCOME_UNKNOWN");
  }
  const diagnostics = (Array.isArray(snapshot.diagnostics) ? snapshot.diagnostics : [])
    .map((row) => cloneDiagnostic(
      row,
      String(row?.severity ?? "error") === "warning" ? "LANGUAGE_WARNING" : "LANGUAGE_ERROR",
    ));
  const hasLanguageError = diagnostics.some((row) => row.status_axis === "LANGUAGE_ERROR");
  const blocked = outcome !== "ACCEPTED" || hasLanguageError;
  const source = String(sourceText);
  const candidates = [];
  if (outcome === "ACCEPTED") {
    diagnostics.forEach((diagnostic, index) => {
      const repairCandidate = buildAuthoritativeWarningFix(
        source,
        String(sourceHash),
        snapshot,
        diagnostic,
      );
      if (!repairCandidate) return;
      candidates.push(repairCandidate);
      diagnostics[index] = {
        ...diagnostic,
        fix_it: {
          id: repairCandidate.id,
          diagnostic_identity: repairCandidate.diagnostic_identity,
          repair_proposal_identity: repairCandidate.repair_proposal_identity,
          source_revision_identity: repairCandidate.source_revision_identity,
          analysis_generation: repairCandidate.analysis_generation,
          expected_slice: repairCandidate.expected_slice,
          replacement: repairCandidate.replacement,
          start_offset_utf16: repairCandidate.start_offset_utf16,
          end_offset_utf16: repairCandidate.end_offset_utf16,
        },
      };
    });
  }
  const candidate = candidates[0] ?? null;
  return {
    schema: SOURCE_READINESS_SCHEMA,
    policy_layer: "workbench_source_readiness_projection",
    authority_status: "shared_frontend",
    ready: !blocked,
    blocked,
    stale: false,
    source_hash: String(sourceHash),
    source_revision_identity: String(snapshot.source_revision_identity ?? ""),
    analysis_generation: Number(snapshot.analysis_generation),
    source_profile_identity: String(snapshot.source_profile_identity ?? ""),
    analysis_outcome: outcome,
    module_directive_metadata: snapshot.schema === MODULE_DIRECTIVE_ANALYSIS_SNAPSHOT_SCHEMA
      ? JSON.parse(JSON.stringify(snapshot.metadata ?? null))
      : null,
    diagnostics,
    candidate,
    candidates,
  };
}

export function projectSourceAnalysisUnavailable(error, {
  analysisGeneration = 0,
  sourceHash = "",
} = {}) {
  return {
    schema: SOURCE_READINESS_SCHEMA,
    policy_layer: "workbench_source_readiness_projection",
    authority_status: "shared_frontend_unavailable",
    ready: false,
    blocked: true,
    stale: false,
    source_hash: String(sourceHash),
    source_revision_identity: "",
    analysis_generation: Number(analysisGeneration),
    source_profile_identity: SOURCE_ANALYSIS_PROFILE_IDENTITY,
    analysis_outcome: "UNSUPPORTED",
    diagnostics: [{
      code: "E_SOURCE_ANALYSIS_UNAVAILABLE",
      severity: "error",
      message: String(error?.message ?? error ?? "공통 원문 분석을 사용할 수 없습니다."),
      path: "",
      span: null,
      status_axis: "PRODUCT_READINESS",
      authority: "workbench_product",
    }],
    candidate: null,
    candidates: [],
  };
}

// Workbench receives this only from the shared RuntimeBridge.  Keep every
// value in the already-versioned run-summary envelope exact: a browser-side
// projection must never publish a receipt for different raw source bytes,
// profile, or committed state.
export function sharedRunSummaryBindingValid(summary, {
  sourceProfileIdentity = SOURCE_ANALYSIS_PROFILE_IDENTITY,
  sourceBytesSha256 = "",
  stateHash = "",
} = {}) {
  return Boolean(
    summary
    && typeof summary === "object"
    && String(sourceProfileIdentity ?? "")
    && String(summary.source_profile_identity ?? "") === String(sourceProfileIdentity)
    && String(sourceBytesSha256 ?? "")
    && String(summary.execution_envelope?.source_bytes_sha256 ?? "") === String(sourceBytesSha256)
    && String(stateHash ?? "")
    && String(summary.state_hash ?? "") === String(stateHash)
    && summary.work_cost_receipt
    && typeof summary.work_cost_receipt === "object"
  );
}

export function applySourceReadinessFix(sourceText, candidate) {
  const source = String(sourceText ?? "");
  if (!candidate) {
    return {
      status: "no_op",
      changed: false,
      source,
      source_hash_before: stableSourceHash(source),
      source_hash_after: stableSourceHash(source),
    };
  }
  const start = Number(candidate.start_offset_utf16 ?? -1);
  const end = Number(candidate.end_offset_utf16 ?? -1);
  const stale = source !== candidate.original_source
    || stableSourceHash(source) !== candidate.source_hash
    || start < 0
    || end <= start
    || source.slice(start, end) !== candidate.expected_slice;
  if (stale) {
    return {
      status: "stale_rejected",
      changed: false,
      source,
      source_hash_before: stableSourceHash(source),
      source_hash_after: stableSourceHash(source),
    };
  }
  const nextSource = `${source.slice(0, start)}${candidate.replacement}${source.slice(end)}`;
  return {
    status: "applied",
    changed: true,
    source: nextSource,
    changed_count: 1,
    source_hash_before: candidate.source_hash,
    source_hash_after: stableSourceHash(nextSource),
    focus_offset_utf16: start,
  };
}
