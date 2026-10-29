// Existing portable project and input-tape actions; format checks, authority and failure paths unchanged.
export function createPortableActions({ activeRun, activeRuntimeSourceText, buildPortableProjectArtifact, buildPortableProjectInput, currentLocalProjectTreePayload, currentPortableMetadata, currentProject, currentVirtualProjectSource, els, gajiIdsFromLocalProjectPayload, hostBridge, localProjectEntryPath, localProjectTreePayloadFromSnapshot, normalizePortableMetadataDraft, normalizePortableTextArray, parsePortableProjectArtifactBytes, portableListFromText, portableProjectDiagnostic, portableProjectFromArtifact, pushDiagnostic, readPortableProjectCatalog, rememberMyWorkProject, rememberPortableProject, renderAll, runtimeBridge, seedImportedProjectExecutionRecord, selectProject, selectUnversionedDocumentProfile, SOURCE_ANALYSIS_PROFILE_IDENTITY, state, workbenchProjectRegistryState, workbenchSourceProfile, writeWorkbenchCapabilitySession }) {
function renderPortableExportStatus() {
  if (!els.portableExportStatus) return;
  const row = state.lastPortableExport;
  if (!row) {
    els.portableExportStatus.textContent = "";
    return;
  }
  const hash = String(row.file?.content_sha256 ?? "").slice(0, 12) || "-";
  const bytes = Number(row.file?.byte_length ?? 0);
  els.portableExportStatus.textContent = `내보냄: ${hash} · ${bytes}바이트`;
}

function renderPortableImportPreview() {
  const preview = els.portableImportPreview;
  if (!preview) return;
  const row = state.pendingPortableImport;
  preview.classList.toggle("hidden", !row);
  preview.dataset.state = row?.error ? "error" : row ? "ready" : "";
  if (!row) return;
  if (row.error) {
    els.portableImportTitle.textContent = "가져오기 실패";
    els.portableImportIntegrity.textContent = `진단: ${row.error.code}`;
    els.portableImportTrust.textContent = "현재 작업 보존됨";
    els.portableImportDetails.textContent = row.error.message;
    els.portableOpen.disabled = true;
    return;
  }
  const artifact = row.artifact;
  const snapshot = artifact.project_snapshot ?? {};
  const metadata = snapshot.metadata ?? {};
  const title = String(metadata.title || snapshot.project_name || "받은 셈그림 프로젝트");
  const tags = normalizePortableTextArray(metadata.tags).join(", ") || "-";
  const fields = row.validation?.field_audit?.present_current_fields ?? [];
  const localProjectPayload = localProjectTreePayloadFromSnapshot(snapshot);
  els.portableImportTitle.textContent = title;
  els.portableImportIntegrity.textContent = `파일 무결성 확인됨 · ${String(artifact.content_sha256 || "").slice(0, 12)}`;
  els.portableImportTrust.textContent = "받은 프로젝트 · 공식 여부 확인 안 됨";
  if (localProjectPayload) {
    const entry = String(localProjectPayload.entry_path ?? "-");
    const fileCount = Array.isArray(localProjectPayload.files) ? localProjectPayload.files.length : 0;
    const hasLock = (localProjectPayload.files ?? []).some((file) => String(file?.path ?? "") === "ddn.lock");
    const gajiCount = gajiIdsFromLocalProjectPayload(localProjectPayload).length;
    els.portableImportDetails.textContent =
      `설명: ${String(metadata.summary || "-")} · 태그: ${tags} · entry ${entry} · files ${fileCount} · lock ${hasLock ? "checked" : "missing"} · 가지 ${gajiCount} · 부속 필드 ${fields.length}종`;
  } else {
    els.portableImportDetails.textContent = `설명: ${String(metadata.summary || "-")} · 태그: ${tags} · source ${String(snapshot.ddn_text || "").length}자 · 부속 필드 ${fields.length}종`;
  }
  els.portableOpen.disabled = false;
}

function renderPortableMetadataEditor() {
  const metadata = currentPortableMetadata();
  if (els.portableMetaTitle && document.activeElement !== els.portableMetaTitle) els.portableMetaTitle.value = metadata.title;
  if (els.portableMetaSummary && document.activeElement !== els.portableMetaSummary) els.portableMetaSummary.value = metadata.summary;
  if (els.portableMetaContentKind) els.portableMetaContentKind.value = metadata.content_kinds[0] || "simulation";
  if (els.portableMetaTags && document.activeElement !== els.portableMetaTags) els.portableMetaTags.value = metadata.tags.join(", ");
  if (els.portableMetaSubject && document.activeElement !== els.portableMetaSubject) els.portableMetaSubject.value = metadata.subject;
  if (els.portableMetaLevel && document.activeElement !== els.portableMetaLevel) els.portableMetaLevel.value = metadata.level;
  if (els.portableMetaRequiredViews && document.activeElement !== els.portableMetaRequiredViews) els.portableMetaRequiredViews.value = metadata.required_views.join(", ");
  if (els.portableMetaRequiredGaji && document.activeElement !== els.portableMetaRequiredGaji) els.portableMetaRequiredGaji.value = metadata.required_gaji.join(", ");
  if (els.portableMetadataStatus) {
    els.portableMetadataStatus.textContent = `metadata: ${metadata.content_kinds[0]} · ${metadata.tags.length} tags · artifact hash 입력`;
  }
}

function triggerPortableDownload({ jsonText = "", filename = "seamgrim-project.seamgrim-project.json" } = {}) {
  return hostBridge.downloadTextFile({
    text: jsonText,
    filename,
    mimeType: "application/json;charset=utf-8",
  });
}

function tapeStatus(message) {
  document.querySelector("[data-testid='input-tape-status']").textContent = message;
}

async function exportCurrentInputTape() {
  const run = activeRun();
  if (run?.restored_without_tape_history) {
    throw new Error("E_RESTORED_INPUT_HISTORY_UNAVAILABLE: 복원한 실행에는 이전 입력 기록 전체가 없어 Tape V1로 저장하지 않습니다. 새로 시작한 실행을 기록하세요.");
  }
  if (!run?.frames?.length) throw new Error("저장할 실행이 없습니다. 먼저 실행하세요.");
  if (state.executionLifecycle.status === "running") throw new Error("일시정지하거나 중지한 뒤 저장하세요.");
  const receipts = run.frames.map((frame) => {
    if (Object.keys(frame.wasm_state?.param_overrides ?? {}).length) {
      throw new Error("조절값을 바꾼 실행은 Tape V1만으로 재현할 수 없어 저장하지 않습니다.");
    }
    if (!frame.wasm_state?.input_receipt) throw new Error("실제 입력 영수증이 없는 실행은 저장하지 않습니다.");
    return frame.wasm_state.input_receipt;
  });
  // Existing CLI Tape V1 header default; it does not set simulation dt or
  // prescribe Alpha2 real-time rate. All calculation still uses Rust dt=1.
  const bytes = await runtimeBridge.exportInputTapeV1({ inputReceipts: receipts, madiHz: 60 });
  hostBridge.downloadFileBytes({ bytes, filename: "input-v1.tape" });
  tapeStatus(`입력 기록 ${receipts.length}마디 저장`);
}

async function checkInputTapeFile(file) {
  if (state.executionLifecycle.status === "running") throw new Error("일시정지하거나 중지한 뒤 확인하세요.");
  const project = currentProject();
  if (currentVirtualProjectSource(project)) {
    throw new Error("여러 파일 작품의 입력 기록 확인은 아직 지원하지 않습니다.");
  }
  const tapeBytes = new Uint8Array(await hostBridge.readFileBytes(file));
  const result = await runtimeBridge.runSummaryWithTapeV1({
    sourceText: activeRuntimeSourceText(project), sourcePath: localProjectEntryPath(project),
    sourceProfileIdentity: workbenchSourceProfile(project), tapeBytes,
  });
  // Host verification result, not a replacement for the installed live VM.
  state.lastTapeV1Verification = result;
  tapeStatus(`입력 기록 계산 완료: ${result.state_hash}`);
}

document.querySelector("[data-testid='input-tape-export']").addEventListener("click", () => {
  exportCurrentInputTape().catch((error) => tapeStatus(String(error.message ?? error)));
});
document.querySelector("[data-testid='input-tape-import']").addEventListener("click", () => {
  document.querySelector("[data-testid='input-tape-file']").click();
});
document.querySelector("[data-testid='input-tape-file']").addEventListener("change", (event) => {
  const file = event.target.files?.[0];
  event.target.value = "";
  if (file) checkInputTapeFile(file).catch((error) => tapeStatus(String(error.message ?? error)));
});

async function buildCurrentPortableProjectArtifact() {
  const input = buildPortableProjectInput();
  const localProjectTreePayload = await currentLocalProjectTreePayload();
  if (localProjectTreePayload) {
    input.localPackagePayload = localProjectTreePayload;
  }
  const result = await buildPortableProjectArtifact(input);
  state.lastPortableExport = result;
  renderPortableExportStatus();
  return result;
}

async function exportPortableProject({ download = true } = {}) {
  try {
    const result = await buildCurrentPortableProjectArtifact();
    if (download) {
      triggerPortableDownload({
        jsonText: result.file.json_text,
        filename: result.file.filename,
      });
    }
    pushDiagnostic("W_PORTABLE_PROJECT_EXPORTED", `휴대형 프로젝트 ${result.file.filename} 내보내기 준비 완료`);
    renderPortableExportStatus();
    return result;
  } catch (err) {
    const diagnostic = portableProjectDiagnostic(err);
    pushDiagnostic(diagnostic.code, diagnostic.message, "error");
    renderPortableExportStatus();
    throw err;
  }
}

async function previewPortableProjectFile(file) {
  if (!file) {
    state.pendingPortableImport = null;
    renderPortableImportPreview();
    return null;
  }
  try {
    const bytes = await hostBridge.readFileBytes(file);
    const parsed = await parsePortableProjectArtifactBytes(bytes);
    state.pendingPortableImport = {
      schema: "seamgrim.workbench.pending_portable_import.v1",
      file_name: String(file.name || ""),
      artifact: parsed.artifact,
      validation: parsed.validation,
      byte_length: parsed.byte_length,
    };
    renderPortableImportPreview();
    pushDiagnostic("W_PORTABLE_PROJECT_PREVIEW_READY", "파일 무결성을 확인했습니다. 열기 전까지 실행하지 않습니다.");
    return state.pendingPortableImport;
  } catch (err) {
    const diagnostic = portableProjectDiagnostic(err);
    state.pendingPortableImport = {
      schema: "seamgrim.workbench.pending_portable_import.v1",
      file_name: String(file.name || ""),
      error: diagnostic,
    };
    renderPortableImportPreview();
    pushDiagnostic(diagnostic.code, diagnostic.message, "error");
    return null;
  } finally {
    if (els.portableImportInput) {
      els.portableImportInput.value = "";
    }
  }
}

async function openPendingPortableProject() {
  const pending = state.pendingPortableImport;
  if (!pending?.artifact || pending.error) return null;
  const artifact = pending.artifact;
  const importedAt = new Date().toISOString();
  const hashPrefix = String(artifact.content_sha256 || "portable").slice(0, 12) || "portable";
  const duplicateCount = readPortableProjectCatalog()
    .filter((row) => String(row.content_sha256 || "") === String(artifact.content_sha256 || ""))
    .length;
  const project = portableProjectFromArtifact(artifact, {
    catalogId: `portable_${hashPrefix}_${String(duplicateCount + 1).padStart(2, "0")}`,
    importedAt,
  });
  if (!localProjectTreePayloadFromSnapshot(project.portableSnapshot ?? {})
    && !workbenchSourceProfile(project)
    && !project.portableSnapshot?.metadata?.source_profile_identity) {
    const accepted = globalThis.confirm("이 파일에는 글판이 없습니다. v25 (실행 프로필 v1-core-v25) 문서로 열까요? 원본 파일은 바꾸지 않습니다.");
    if (!accepted) return null;
    selectUnversionedDocumentProfile(project, SOURCE_ANALYSIS_PROFILE_IDENTITY);
  }
  workbenchProjectRegistryState.receivedPortableProjects = [
    project,
    ...workbenchProjectRegistryState.receivedPortableProjects.filter((row) => row.id !== project.id),
  ];
  rememberPortableProject(project);
  state.pendingPortableImport = null;
  await selectProject(project.id);
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "portable_import",
    server_saved: false,
    local_saved: true,
    user_label: `받은 프로젝트 ${hashPrefix}`,
  };
  state.runs = [];
  state.activeRunId = "";
  state.cursorIndex = -1;
  seedImportedProjectExecutionRecord(project);
  pushDiagnostic("W_PORTABLE_PROJECT_IMPORTED", "받은 프로젝트로 열었습니다. 실행은 사용자가 직접 시작해야 합니다.");
  renderAll();
  return project;
}

function updatePortableMetadataDraftFromInputs() {
  state.portableMetadataDraft = normalizePortableMetadataDraft({
    title: els.portableMetaTitle?.value ?? "",
    summary: els.portableMetaSummary?.value ?? "",
    content_kinds: [els.portableMetaContentKind?.value ?? "simulation"],
    tags: portableListFromText(els.portableMetaTags?.value ?? "", 8),
    subject: els.portableMetaSubject?.value ?? "",
    level: els.portableMetaLevel?.value ?? "",
    required_views: portableListFromText(els.portableMetaRequiredViews?.value ?? "", 8),
    required_gaji: portableListFromText(els.portableMetaRequiredGaji?.value ?? "", 8),
    determinism_grade: currentProject().viewGrade,
  });
  state.lastPortableExport = null;
  renderPortableExportStatus();
  renderPortableMetadataEditor();
}

function updateServerStateFromPayload(payload = {}, capabilities = {}) {
  state.serverProject = {
    ...state.serverProject,
    projectId: String(payload.project_id ?? state.serverProject.projectId ?? ""),
    headRevisionId: String(payload.revision_id ?? payload.head_revision_id ?? state.serverProject.headRevisionId ?? ""),
    contentSha256: String(payload.content_sha256 ?? state.serverProject.contentSha256 ?? ""),
    packageHash: String(payload.package_hash ?? state.serverProject.packageHash ?? ""),
    editCapability: String(capabilities.editCapability ?? payload.edit_capability ?? state.serverProject.editCapability ?? ""),
    viewCapability: String(capabilities.viewCapability ?? payload.view_capability ?? state.serverProject.viewCapability ?? ""),
    role: String(payload.capability_role ?? state.capabilitySession?.role ?? state.serverProject.role ?? ""),
  };
  if (state.serverProject.projectId && state.serverProject.editCapability) {
    state.capabilitySession = writeWorkbenchCapabilitySession({
      projectId: state.serverProject.projectId,
      role: "edit",
      capability: state.serverProject.editCapability,
    }) ?? state.capabilitySession;
  }
  if (state.serverProject.projectId && (state.serverProject.editCapability || state.serverProject.viewCapability)) {
    rememberMyWorkProject({
      projectId: state.serverProject.projectId,
      title: currentProject().title,
      headRevisionId: state.serverProject.headRevisionId,
      editCapability: state.serverProject.editCapability,
      viewCapability: state.serverProject.viewCapability,
    });
  }
}

  return { renderPortableExportStatus, renderPortableImportPreview, renderPortableMetadataEditor, triggerPortableDownload, tapeStatus, exportCurrentInputTape, checkInputTapeFile, buildCurrentPortableProjectArtifact, exportPortableProject, previewPortableProjectFile, openPendingPortableProject, updatePortableMetadataDraftFromInputs, updateServerStateFromPayload };
}
