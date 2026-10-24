// Source analysis and repair keep the original state, authority and failure paths.
export function createSourceReadinessModel({ state, els, currentProject, activeRuntimeSourceText, localProjectEntryPath, analyzeSourceReadiness, workbenchSourceProfile, currentVirtualProjectSource, SOURCE_ANALYSIS_PROFILE_IDENTITY, runtimeBridge, projectEditDiagnostic, MODULE_DIRECTIVE_ANALYSIS_SNAPSHOT_SCHEMA, projectAuthoritativeSourceAnalysis, projectSourceAnalysisUnavailable, applySourceReadinessFix, SOURCE_READINESS_FIX_ID, pushDiagnostic, renderAll, syncNativeProjectFromState, markCurrentProjectExecutionStale, refreshProjectGraph, ONE_FILE_DDN_PROJECT_ID, writeOneFileDraft }) {
function sourceReadinessViewOnly() {
  const capabilityRole = state.capabilitySession?.role || state.serverProject.role || "";
  return Boolean((state.serverProject.projectId || state.capabilitySession?.project_id) && capabilityRole === "view");
}

function sourceReadinessApplyOutcome(status, details = {}) {
  const outcome = {
    schema: "seamgrim.workbench.source_readiness_apply_outcome.v1",
    status,
    code: String(details.code ?? ""),
    fix_it_id: String(details.fix_it_id ?? ""),
    changed: Boolean(details.changed),
    changed_count: Number(details.changed_count ?? 0),
    source_hash_before: String(details.source_hash_before ?? ""),
    source_hash_after: String(details.source_hash_after ?? ""),
  };
  state.sourceReadinessLastApplyOutcome = outcome;
  return outcome;
}

function sourceReadinessRunOutcome(status, details = {}) {
  const outcome = {
    schema: "seamgrim.workbench.source_readiness_run_outcome.v1",
    status,
    code: String(details.code ?? ""),
    source_hash: String(details.source_hash ?? ""),
    run_generation: Number(state.runSeq),
    run_record_count: state.runs.length,
    wasm_vm_entry_count: Number(state.wasmVmEntryCount),
  };
  state.sourceReadinessLastRunOutcome = outcome;
  return outcome;
}

function refreshPendingSourceReadiness(project = currentProject()) {
  const previousHash = String(state.sourceReadiness?.source_hash ?? "");
  const entryPath = localProjectEntryPath(project);
  const next = analyzeSourceReadiness(activeRuntimeSourceText(project), entryPath);
  state.sourceReadiness = next;
  if (previousHash && previousHash !== next.source_hash) {
    state.sourceReadinessRunBlocked = false;
  }
  return next;
}

function refreshSourceReadiness(project = currentProject()) {
  const entryPath = localProjectEntryPath(project);
  const pending = analyzeSourceReadiness(activeRuntimeSourceText(project), entryPath);
  if (
    state.sourceReadiness?.authority_status === "shared_frontend"
    && state.sourceReadiness.source_hash === pending.source_hash
  ) {
    return state.sourceReadiness;
  }
  return refreshPendingSourceReadiness(project);
}

function moduleDirectiveRegistryForProject(project) {
  const registry = project?.moduleDirectiveRegistry;
  if (!registry) return null;
  const known = registry.known_import_targets;
  const publicDefinitions = registry.public_definition_targets;
  if (!Array.isArray(known) || !Array.isArray(publicDefinitions)) return null;
  if (![...known, ...publicDefinitions].every((value) => typeof value === "string" && value.length > 0)) {
    return null;
  }
  return {
    known_import_targets: [...known],
    public_definition_targets: [...publicDefinitions],
  };
}

async function refreshAuthoritativeSourceReadiness(project = currentProject()) {
  const projectIdAtRequest = project.id;
  const entryPath = localProjectEntryPath(project);
  const pending = analyzeSourceReadiness(activeRuntimeSourceText(project), entryPath);
  const sourceProfileIdentity = workbenchSourceProfile(project);
  const projectSource = currentVirtualProjectSource(project);
  if (!projectSource && sourceProfileIdentity !== SOURCE_ANALYSIS_PROFILE_IDENTITY) {
    const rejected = { ...pending, ready: false, blocked: true, diagnostics: [{
      code: "E_SOURCE_PROFILE_REQUIRED", severity: "error",
      message: "문서의 글판이 없거나 지원되지 않습니다. 글판이 명시된 프로젝트를 가져오거나 새 v25 문서로 옮겨 주세요.",
    }] };
    state.sourceReadiness = rejected;
    return rejected;
  }
  if (!pending.ready) {
    state.sourceReadiness = pending;
    return pending;
  }

  // A native local project owns a typed file graph, not an isolated entry
  // string.  Its shared prepare/link frontdoor validates RD-457 metadata and
  // the ordinary declaration tail together before execution.  Routing that
  // graph through the single-file carrier would reject valid body statements
  // before the same shared project owner can bind the module identities.
  if (projectSource) {
    const sourceAtRequest = activeRuntimeSourceText(project);
    const generation = state.sourceAnalysisGeneration + 1;
    state.sourceAnalysisGeneration = generation;
    state.sourceAnalysisSourceText = sourceAtRequest;
    try {
      // The run path seals or refreshes its generated lock immediately after
      // this semantic preflight.  Do not require that future artifact from
      // the raw-source readiness gate itself.
      const prepared = await runtimeBridge.prepareProjectLink(projectSource, "", true, false);
      const rootProfile = String(prepared?.source_profile_identity ?? "");
      if (rootProfile !== SOURCE_ANALYSIS_PROFILE_IDENTITY
        || (sourceProfileIdentity && sourceProfileIdentity !== rootProfile)) {
        throw new Error("프로젝트 뿌리의 글판이 없거나 문서 글판과 일치하지 않습니다.");
      }
      project.resolvedRootSourceProfileIdentity = rootProfile;
      if (currentProject().id !== projectIdAtRequest) {
        return { ...pending, ready: false, blocked: true, superseded: true };
      }
      if (activeRuntimeSourceText(currentProject()) !== sourceAtRequest) {
        return { ...pending, ready: false, blocked: true, superseded: true };
      }
      const projected = {
        ...pending,
        authority_status: "shared_project_frontdoor",
        ready: true,
        blocked: false,
        stale: false,
        source_revision_identity: String(prepared?.prepare_link_sha256 ?? ""),
        analysis_generation: generation,
        source_profile_identity: rootProfile,
        analysis_outcome: "ACCEPTED",
        module_directive_metadata: null,
        diagnostics: [],
        candidate: null,
        candidates: [],
      };
      state.sourceReadiness = projected;
      return projected;
    } catch (error) {
      const diagnostic = projectEditDiagnostic(error);
      const rejected = {
        ...pending,
        authority_status: "shared_project_frontdoor",
        ready: false,
        blocked: true,
        stale: false,
        analysis_generation: generation,
        source_profile_identity: workbenchSourceProfile(project),
        analysis_outcome: "REJECTED",
        diagnostics: [{
          code: String(diagnostic.code ?? "E_PROJECT_PREPARE_LINK"),
          severity: "error",
          status_axis: "LANGUAGE_ERROR",
          message: String(diagnostic.message ?? error),
          path: String(diagnostic.path ?? entryPath),
          span: diagnostic.source_span ?? null,
        }],
        candidate: null,
        candidates: [],
      };
      state.sourceReadiness = rejected;
      return rejected;
    }
  }

  for (let attempt = 0; attempt < 2; attempt += 1) {
    const sourceAtRequest = activeRuntimeSourceText(project);
    const generation = state.sourceAnalysisGeneration + 1;
    state.sourceAnalysisGeneration = generation;
    state.sourceAnalysisSourceText = sourceAtRequest;
    try {
      const registry = moduleDirectiveRegistryForProject(project);
      const request = {
        source_text: sourceAtRequest,
        path: entryPath,
        analysis_generation: generation,
        source_profile_identity: sourceProfileIdentity,
      };
      const snapshot = registry
        ? await runtimeBridge.moduleDirectiveAnalysis({
            schema: MODULE_DIRECTIVE_ANALYSIS_SNAPSHOT_SCHEMA,
            ...request,
            module_registry: registry,
          })
        : await runtimeBridge.sourceAnalysis(request);
      if (currentProject().id !== projectIdAtRequest) {
        return { ...pending, ready: false, blocked: true, superseded: true };
      }
      const sourceStillCurrent = activeRuntimeSourceText(currentProject()) === sourceAtRequest;
      const generationStillCurrent = state.sourceAnalysisGeneration === generation;
      if (!sourceStillCurrent || !generationStillCurrent) continue;
      const projected = projectAuthoritativeSourceAnalysis(snapshot, {
        expectedAnalysisGeneration: generation,
        sourceHash: pending.source_hash,
        sourceText: sourceAtRequest,
      });
      if (projected.stale) continue;
      state.sourceReadiness = projected;
      return projected;
    } catch (error) {
      const unavailable = projectSourceAnalysisUnavailable(error, {
        analysisGeneration: generation,
        sourceHash: pending.source_hash,
      });
      state.sourceReadiness = unavailable;
      return unavailable;
    }
  }

  const stale = projectSourceAnalysisUnavailable("E_SOURCE_ANALYSIS_STALE", {
    analysisGeneration: state.sourceAnalysisGeneration,
    sourceHash: pending.source_hash,
  });
  stale.authority_status = "shared_frontend_stale_rejected";
  state.sourceReadiness = stale;
  return stale;
}

function getSourceReadinessCandidate() {
  const candidate = state.sourceReadiness?.candidate;
  return candidate ? JSON.parse(JSON.stringify(candidate)) : null;
}

function sameAuthoritativeRepairCandidate(left, right) {
  if (!left || !right) return false;
  return String(left.repair_proposal_identity ?? "") === String(right.repair_proposal_identity ?? "")
    && String(left.diagnostic_identity ?? "") === String(right.diagnostic_identity ?? "")
    && String(left.source_revision_identity ?? "") === String(right.source_revision_identity ?? "")
    && Number(left.analysis_generation) === Number(right.analysis_generation)
    && String(left.source_hash ?? "") === String(right.source_hash ?? "")
    && String(left.original_source ?? "") === String(right.original_source ?? "")
    && Number(left.start_offset_utf16) === Number(right.start_offset_utf16)
    && Number(left.end_offset_utf16) === Number(right.end_offset_utf16)
    && String(left.expected_slice ?? "") === String(right.expected_slice ?? "")
    && String(left.replacement ?? "") === String(right.replacement ?? "");
}

function sourceReadinessSnapshot() {
  const readiness = state.sourceReadiness ?? refreshSourceReadiness();
  const diagnostic = readiness.diagnostics?.[0] ?? null;
  const viewOnly = sourceReadinessViewOnly();
  return {
    schema: readiness.schema,
    policy_layer: readiness.policy_layer,
    authority_status: String(readiness.authority_status ?? ""),
    ready: Boolean(readiness.ready),
    blocked: Boolean(readiness.blocked),
    source_hash: String(readiness.source_hash ?? ""),
    source_revision_identity: String(readiness.source_revision_identity ?? ""),
    analysis_generation: Number(readiness.analysis_generation ?? 0),
    source_profile_identity: String(readiness.source_profile_identity ?? ""),
    analysis_outcome: String(readiness.analysis_outcome ?? ""),
    code: String(diagnostic?.code ?? ""),
    severity: String(diagnostic?.severity ?? ""),
    status_axis: String(diagnostic?.status_axis ?? ""),
    upstream_cause_code: String(diagnostic?.upstream_cause_code ?? ""),
    what: String(diagnostic?.what ?? ""),
    where: String(diagnostic?.where ?? ""),
    fix: String(diagnostic?.fix ?? ""),
    span: diagnostic?.span ? JSON.parse(JSON.stringify(diagnostic.span)) : null,
    diagnostic_identity: String(diagnostic?.diagnostic_identity ?? ""),
    repair_proposal_identity: String(readiness.candidate?.repair_proposal_identity ?? ""),
    fix_it_id: String(diagnostic?.fix_it?.id ?? ""),
    fix_it_trigger_authority: String(readiness.candidate?.trigger_authority ?? ""),
    fix_it_replacement_authority: String(readiness.candidate?.replacement_authority ?? ""),
    run_blocked: Boolean(state.sourceReadinessRunBlocked),
    apply_available: Boolean(readiness.candidate && !viewOnly),
    apply_disabled_reason: viewOnly ? "view_only" : readiness.candidate ? "" : "no_candidate",
    last_apply_outcome: { ...state.sourceReadinessLastApplyOutcome },
    last_run_outcome: { ...state.sourceReadinessLastRunOutcome },
    run_generation: Number(state.runSeq),
    run_record_count: state.runs.length,
    wasm_vm_entry_count: Number(state.wasmVmEntryCount),
    wasm_initialized: Boolean(state.wasm),
  };
}

function applySourceReadinessFixIt(candidateOverride = undefined) {
  const project = currentProject();
  const readiness = refreshSourceReadiness(project);
  const currentSource = activeRuntimeSourceText(project);
  if (sourceReadinessViewOnly()) {
    const outcome = sourceReadinessApplyOutcome("view_only_rejected", {
      code: "E_SOURCE_FIX_VIEW_ONLY",
      fix_it_id: SOURCE_READINESS_FIX_ID,
      source_hash_before: readiness.source_hash,
      source_hash_after: readiness.source_hash,
    });
    pushDiagnostic("E_SOURCE_FIX_VIEW_ONLY", "보기 전용 프로젝트에서는 원문 고침을 적용할 수 없습니다.", "error");
    renderAll();
    return outcome;
  }

  const candidate = candidateOverride === undefined ? getSourceReadinessCandidate() : candidateOverride;
  const candidateStillCurrent = candidate && (Array.isArray(readiness.candidates) ? readiness.candidates : [])
    .some((current) => sameAuthoritativeRepairCandidate(candidate, current));
  const authorityStale = candidate && (
    readiness.authority_status !== "shared_frontend"
    || String(candidate.source_revision_identity ?? "") !== String(readiness.source_revision_identity ?? "")
    || Number(candidate.analysis_generation) !== Number(readiness.analysis_generation)
    || !candidateStillCurrent
  );
  if (authorityStale) {
    const outcome = sourceReadinessApplyOutcome("stale_rejected", {
      code: "E_SOURCE_FIX_STALE",
      fix_it_id: SOURCE_READINESS_FIX_ID,
      source_hash_before: readiness.source_hash,
      source_hash_after: readiness.source_hash,
    });
    pushDiagnostic("E_SOURCE_FIX_STALE", "원문 또는 권위 분석 세대가 고침 후보를 만든 뒤 달라져 적용하지 않았습니다.", "error");
    renderAll();
    return outcome;
  }
  const result = applySourceReadinessFix(currentSource, candidate);
  if (result.status === "stale_rejected") {
    const outcome = sourceReadinessApplyOutcome("stale_rejected", {
      ...result,
      code: "E_SOURCE_FIX_STALE",
      fix_it_id: SOURCE_READINESS_FIX_ID,
    });
    pushDiagnostic("E_SOURCE_FIX_STALE", "원문이 고침 후보를 만든 뒤 달라져 적용하지 않았습니다.", "error");
    renderAll();
    return outcome;
  }
  if (result.status === "no_op") {
    const outcome = sourceReadinessApplyOutcome("no_op", {
      ...result,
      code: "W_SOURCE_FIX_NO_OP",
      fix_it_id: SOURCE_READINESS_FIX_ID,
    });
    pushDiagnostic("W_SOURCE_FIX_NO_OP", "적용할 구식 채비: 표면이 없습니다.");
    renderAll();
    return outcome;
  }

  const entryPath = localProjectEntryPath(project);
  state.fileTexts.set(entryPath, result.source);
  state.activeFilePath = entryPath;
  state.lastPortableExport = null;
  state.sourceReadinessRunBlocked = false;
  syncNativeProjectFromState(project);
  markCurrentProjectExecutionStale("source_readiness_fix");
  if (state.projectExecution?.failure) state.projectExecution.failure = null;
  clearTimeout(state.projectGraphRefreshTimer);
  state.projectGraphRefreshTimer = setTimeout(() => refreshProjectGraph().catch(() => {}), 250);
  if (project.id === ONE_FILE_DDN_PROJECT_ID && entryPath === "lesson.ddn") {
    writeOneFileDraft(result.source);
  }
  refreshPendingSourceReadiness(project);
  const outcome = sourceReadinessApplyOutcome("applied", {
    ...result,
    code: "W_SOURCE_FIX_APPLIED",
    fix_it_id: SOURCE_READINESS_FIX_ID,
  });
  pushDiagnostic("W_SOURCE_FIX_APPLIED", "고침을 적용했습니다. 콜론을 제거해 채비 {로 바꿨습니다.");
  // The fix action lives inside the verification modal. Close the modal before
  // restoring focus; otherwise the editor is inert and keyboard focus cannot
  // return to the corrected source.
  if (els.projectVerificationDrawer?.open) els.projectVerificationDrawer.close();
  renderAll();
  els.sourceEditor.focus();
  const focusOffset = Number(result.focus_offset_utf16 ?? 0);
  els.sourceEditor.setSelectionRange?.(focusOffset, focusOffset);
  return outcome;
}

  return { sourceReadinessViewOnly, sourceReadinessApplyOutcome, sourceReadinessRunOutcome, refreshPendingSourceReadiness, refreshSourceReadiness, moduleDirectiveRegistryForProject, refreshAuthoritativeSourceReadiness, getSourceReadinessCandidate, sameAuthoritativeRepairCandidate, sourceReadinessSnapshot, applySourceReadinessFixIt };
}
