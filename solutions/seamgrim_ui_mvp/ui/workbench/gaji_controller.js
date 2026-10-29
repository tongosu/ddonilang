// Existing gaji preview, shelf and remix controller; source edits and storage boundaries unchanged.
export function createGajiController({ activeFile, applyVirtualProjectSource, currentProject, currentVirtualProjectSource, ddnQuotedText, els, exportLocalGotganArtifact, hostBridge, importLocalGotganArtifact, loadLocalGotganStore, publishToLocalGotgan, pushDiagnostic, refreshProjectGraph, renderAll, runtimeBridge, searchLocalGotgan, state, triggerPortableDownload, workbenchGajiPresentationState, workbenchProjectRegistryState }) {
function closeGajiBundleDialog() {
  workbenchGajiPresentationState.pendingGajiBundle = null;
  els.gajiBundlePublish.disabled = true;
  els.gajiBundleDialog?.close?.();
}

function openGajiBundleDialog() {
  const file = activeFile();
  const source = currentVirtualProjectSource();
  if (!source || !file?.path?.endsWith(".ddn") || file.path === source.root_path || file.path === source.entry_path) {
    pushDiagnostic("E_GAJI_MODULE_SELECTION_INVALID", "뿌리·진입 파일이 아닌 DDN 모듈을 먼저 선택하세요.", "error");
    renderAll();
    return;
  }
  const leaf = file.path.split("/").at(-1)?.replace(/\.ddn$/u, "") || "내가지";
  els.gajiBundleId.value = leaf;
  els.gajiBundleVersion.value = "1.0.0";
  els.gajiBundleDescription.value = `${leaf} 재사용 모듈`;
  els.gajiBundleTags.value = "";
  els.gajiBundleSelected.textContent = `선택 모듈: ${file.path}`;
  els.gajiBundlePreview.textContent = "미리보기 전";
  els.gajiBundleStatus.textContent = "확인 전에는 발행하지 않습니다.";
  els.gajiBundlePublish.disabled = true;
  workbenchGajiPresentationState.pendingGajiBundle = null;
  els.gajiBundleDialog?.showModal?.();
  els.gajiBundleId?.focus?.();
}

async function previewGajiBundle() {
  const source = currentVirtualProjectSource();
  const selectedPath = activeFile()?.path ?? "";
  const request = {
    schema: "ddn.gaji_extraction_plan.v1",
    source,
    selected_modules: [selectedPath],
    gaji_id: String(els.gajiBundleId.value || "").trim(),
    version: String(els.gajiBundleVersion.value || "").trim(),
    description: String(els.gajiBundleDescription.value || "").trim(),
    tags: String(els.gajiBundleTags.value || "").split(",").map((item) => item.trim()).filter(Boolean),
    profile: "내",
    origin_project_ref: `project:${currentProject().id}`,
  };
  const plan = await runtimeBridge.planGajiExtraction(request);
  workbenchGajiPresentationState.pendingGajiBundle = { request, plan, sourceHash: plan.source_project_sha256 };
  els.gajiBundlePreview.textContent = [
    `가지: ${plan.gaji_id}@${plan.version} · 내 가지곳간`,
    `선택: ${plan.selected_modules.join(", ")}`,
    `자동 포함: ${plan.included_modules.filter((path) => !plan.selected_modules.includes(path)).join(", ") || "없음"}`,
    `공개 심볼: ${plan.public_symbols.map((item) => `${item.module_path}:${item.external_name}`).join(", ")}`,
    `외부 의존: ${Object.entries(plan.external_gaji_dependencies).map(([key, value]) => `${key}=${value}`).join(", ") || "없음"}`,
    `포함 파일: ${plan.files.map((file) => `${file.path} (${file.byte_length} bytes)`).join(", ")}`,
    `제외 파일: ${plan.excluded_files.join(", ") || "없음"}`,
    `content hash: ${plan.content_sha256}`,
    `계보: ${request.origin_project_ref}`,
  ].join("\n");
  els.gajiBundleStatus.textContent = "미리보기 완료 · 발행하면 immutable version으로 저장됩니다.";
  els.gajiBundlePublish.disabled = false;
}

async function publishGajiBundle() {
  if (!workbenchGajiPresentationState.pendingGajiBundle) return;
  const currentHash = await runtimeBridge.projectSourceSha256(currentVirtualProjectSource());
  if (currentHash !== workbenchGajiPresentationState.pendingGajiBundle.sourceHash) {
    throw new Error("E_GAJI_PUBLISH_SOURCE_STALE 미리보기 뒤 project source가 바뀌었습니다");
  }
  const artifact = await runtimeBridge.buildLocalGajiArtifact(workbenchGajiPresentationState.pendingGajiBundle.request);
  await publishToLocalGotgan(runtimeBridge, artifact);
  pushDiagnostic("W_GAJI_PUBLISHED", `${artifact.gaji_id}@${artifact.version}을 내 가지곳간에 발행했습니다.`);
  els.gajiBundleStatus.textContent = `발행 완료: ${artifact.gaji_id}@${artifact.version}`;
  els.gajiBundlePublish.disabled = true;
  workbenchGajiPresentationState.pendingGajiBundle = null;
}

function currentCapabilityIsViewOnly() {
  const role = state.capabilitySession?.role || state.serverProject.role || "";
  return Boolean((state.serverProject.projectId || state.capabilitySession?.project_id) && role === "view");
}

function selectedShelfProfiles() {
  return [...document.querySelectorAll("[data-testid='gaji-profile-filters'] input:checked")].map((node) => node.value);
}

async function refreshGajiShelf() {
  const result = await searchLocalGotgan(runtimeBridge, {
    query: els.gajiShelfSearch.value,
    profiles: selectedShelfProfiles(),
  });
  els.gajiShelfResults.replaceChildren();
  for (const row of result.rows ?? []) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "gaji-shelf-row";
    button.dataset.gajiPackageSha256 = row.package_sha256;
    button.setAttribute("role", "option");
    button.setAttribute("aria-label", `${row.gaji_id} ${row.version} 설치 미리보기`);
    for (const [tag, text] of [
      ["strong", `${row.gaji_id}@${row.version}`],
      ["span", `${row.profile} · ${row.description || "설명 없음"}`],
      ["small", `${row.public_symbols.join(", ")} · ${row.content_sha256.slice(0, 12)}`],
    ]) {
      const child = document.createElement(tag);
      child.textContent = text;
      button.append(child);
    }
    button.addEventListener("click", () => previewShelfInstall(row, button).catch(reportGajiShelfError));
    els.gajiShelfResults.append(button);
  }
  if (!(result.rows ?? []).length) els.gajiShelfResults.textContent = "검색 결과 없음";
  els.gajiInstallStatus.textContent = `검색 ${result.rows?.length ?? 0}개 · ${selectedShelfProfiles().join("/") || "선택 없음"}`;
}

function gajiEditContext() {
  return JSON.stringify([state.projectId,state.projectEditGeneration,currentVirtualProjectSource()]);
}
function requireGajiEditContext(expected) {
  if(gajiEditContext()!==expected)throw new Error("E_PROJECT_EDIT_SOURCE_CONFLICT: 작품이나 원문이 바뀌어 가지 변경을 취소했습니다");
}
function reportGajiShelfError(err) {
  workbenchGajiPresentationState.pendingGajiInstall = null;
  els.gajiInstallApply.disabled = true;
  els.gajiInstallStatus.textContent = String(err?.message ?? err);
  pushDiagnostic("E_GAJI_SHELF", String(err?.message ?? err), "error");
}

async function previewShelfInstall(row, button) {
  const store = loadLocalGotganStore();
  const artifact = store.artifacts.find((item) => item.package_sha256 === row.package_sha256);
  if (!artifact) throw new Error("E_GAJI_SHELF_ARTIFACT_MISSING 선택한 가지 bytes가 없습니다");
  if (currentCapabilityIsViewOnly()) {
    workbenchGajiPresentationState.pendingGajiInstall = null;
    els.gajiInstallPreview.textContent = [
      `${row.gaji_id}@${row.version} · ${row.profile} 가지곳간`,
      `공개 심볼: ${row.public_symbols.join(", ")}`,
      `content hash: ${row.content_sha256}`,
      `origin: ${artifact.provenance.origin_project_ref || "-"}`,
      "보기 capability에서는 project/vendor/lock을 변경하지 않습니다.",
    ].join("\n");
    els.gajiInstallStatus.textContent = "E_GAJI_VIEW_CAPABILITY_WRITE 보기 권한으로 가지를 설치할 수 없습니다.";
    els.gajiInstallApply.disabled = true;
    return;
  }
  const source = currentVirtualProjectSource();
  if (!source || !(currentProject().authoredArtifact?.source_snapshot?.root_path || workbenchProjectRegistryState.nativeProjects.some((item) => item.id === state.projectId))) {
    throw new Error("E_GAJI_INSTALL_PROJECT_REQUIRED 네이티브 프로젝트를 먼저 여세요");
  }
  const alias = String(els.gajiInstallAlias.value || artifact.gaji_id).trim();
  els.gajiInstallAlias.value = alias;
  const installedPath = state.projectGraph?.graph?.gaji_aliases?.[alias] || null;
  const installedId = installedPath?.replace(/^\.\//, "").split("/")[1] || null;
  const installedMetadata = source.files.find((file) => file.path === `vendor/${installedId}/artifact.detjson`);
  const installedArtifact = installedMetadata ? JSON.parse(installedMetadata.utf8_text) : null;
  const kind = installedPath ? "update" : "install";
  const context=gajiEditContext();
  const sourceHash = await runtimeBridge.projectSourceSha256(source);
  requireGajiEditContext(context);
  const request = {
    schema: "ddn.gaji_lifecycle_plan.v1",
    source,
    expected_source_sha256: sourceHash,
    expected_lock_sha256: null,
    operation: { kind, alias, artifact },
  };
  const plan = await runtimeBridge.planGajiLifecycle(request);
  requireGajiEditContext(context);
  workbenchGajiPresentationState.pendingGajiInstall = { row, artifact, request, plan, context };
  document.querySelectorAll(".gaji-shelf-row").forEach((node) => node.classList.toggle("active", node === button));
  els.gajiInstallPreview.textContent = [
    `${row.gaji_id}@${row.version} · ${row.profile} 가지곳간`,
    `alias: ${alias}`,
    `공개 심볼: ${row.public_symbols.join(", ")}`,
    `dependency: ${Object.entries(artifact.dependencies).map(([key, value]) => `${key}@${value}`).join(", ") || "없음"}`,
    `origin: ${artifact.provenance.origin_project_ref || "-"} · ${artifact.provenance.relation}`,
    `원본 가지: ${artifact.provenance.origin_gaji || "없음"}`,
    `vendor 변화: ${plan.affected_files.join(", ")}`,
    ...(plan.version_diff ? [
      `버전: ${plan.version_diff.from_version || "없음"} → ${plan.version_diff.to_version || "없음"}`,
      `파일: +${plan.version_diff.added_files.join(",") || "없음"} / -${plan.version_diff.removed_files.join(",") || "없음"} / 변경 ${plan.version_diff.changed_files.join(",") || "없음"}`,
      `API: +${plan.version_diff.added_public_symbols.join(",") || "없음"} / -${plan.version_diff.removed_public_symbols.join(",") || "없음"}`,
      `의존성 변경: ${plan.version_diff.dependency_changed ? "예" : "아니오"} · 위험: ${plan.version_diff.risky ? "있음" : "없음"}`,
    ] : []),
    `lock: ${plan.before_lock_sha256 || "없음"} → ${plan.after_lock_sha256 || "없음"}`,
    `project graph는 적용 전 Rust core에서 재검증됨`,
  ].join("\n");
  const rollback = installedArtifact && installedArtifact.version !== artifact.version && artifact.version < installedArtifact.version;
  els.gajiInstallApply.textContent = rollback ? "보관 버전으로 되돌리기" : (kind === "update" ? "원자적으로 갱신" : "원자적으로 설치");
  els.gajiInstallStatus.textContent = currentCapabilityIsViewOnly() ? "보기 전용 링크에서는 설치할 수 없습니다." : `${rollback ? "되돌리기" : kind === "update" ? "갱신" : "설치"} 미리보기 완료`;
  els.gajiInstallApply.disabled = currentCapabilityIsViewOnly();
  els.gajiInstallRemove.disabled = currentCapabilityIsViewOnly() || !installedPath;
  els.gajiRemixStart.disabled = currentCapabilityIsViewOnly() || !installedPath;
}

async function applyShelfInstall() {
  if (!workbenchGajiPresentationState.pendingGajiInstall) return;
  if (currentCapabilityIsViewOnly()) throw new Error("E_GAJI_VIEW_CAPABILITY_WRITE 보기 권한으로 가지를 설치할 수 없습니다");
  const pending=workbenchGajiPresentationState.pendingGajiInstall;
  requireGajiEditContext(pending.context);
  const plan=await runtimeBridge.planGajiLifecycle(pending.request);
  requireGajiEditContext(pending.context);
  if(workbenchGajiPresentationState.pendingGajiInstall!==pending)throw new Error("E_PROJECT_EDIT_SOURCE_CONFLICT: 가지 선택이 바뀌었습니다");
  await applyVirtualProjectSource(plan.project_source);
  workbenchGajiPresentationState.pendingGajiInstall = null;
  els.gajiInstallApply.disabled = true;
  els.gajiInstallStatus.textContent = `${plan.alias} ${plan.operation === "install" ? "설치" : "갱신"} 완료 · lock ${plan.after_lock_sha256}`;
  pushDiagnostic("W_GAJI_INSTALLED", `${plan.alias} 가지를 project graph와 lock에 원자 ${plan.operation === "install" ? "설치" : "갱신"}했습니다.`);
  await refreshProjectGraph();
}

async function removeShelfInstall() {
  if (!workbenchGajiPresentationState.pendingGajiInstall) return;
  if (currentCapabilityIsViewOnly()) throw new Error("E_GAJI_VIEW_CAPABILITY_WRITE 보기 권한으로 가지를 제거할 수 없습니다");
  const pending=workbenchGajiPresentationState.pendingGajiInstall;
  requireGajiEditContext(pending.context);
  const request = {
    ...pending.request,
    operation: { kind: "remove", alias: String(els.gajiInstallAlias.value).trim() },
  };
  const plan = await runtimeBridge.planGajiLifecycle(request);
  requireGajiEditContext(pending.context);
  if(workbenchGajiPresentationState.pendingGajiInstall!==pending)throw new Error("E_PROJECT_EDIT_SOURCE_CONFLICT: 가지 선택이 바뀌었습니다");
  await applyVirtualProjectSource(plan.project_source);
  workbenchGajiPresentationState.pendingGajiInstall = null;
  els.gajiInstallApply.disabled = true;
  els.gajiInstallRemove.disabled = true;
  els.gajiInstallStatus.textContent = `${plan.alias} 제거 완료`;
  pushDiagnostic("W_GAJI_REMOVED", `${plan.alias} 가지를 project graph에서 원자 제거했습니다.`);
  await refreshProjectGraph();
}

function startShelfRemix() {
  if (!workbenchGajiPresentationState.pendingGajiInstall?.artifact) return;
  const artifact = workbenchGajiPresentationState.pendingGajiInstall.artifact;
  const primaryPath = artifact.provenance.extraction_modules[0];
  const primary = artifact.files.find((file) => file.path === `ddn/${primaryPath}`);
  if (!primary) throw new Error("E_GAJI_REMIX_SOURCE_MISSING 리믹스할 주 모듈 bytes가 없습니다");
  els.gajiRemixId.value = `${artifact.gaji_id}-내버전`;
  els.gajiRemixVersion.value = "1.0.0";
  els.gajiRemixSource.value = primary.utf8_text;
  els.gajiRemixEditor.hidden = false;
  els.gajiInstallStatus.textContent = `${artifact.gaji_id}@${artifact.version} 원본을 보존한 리믹스 편집 준비`;
}

async function publishShelfRemix() {
  if (!workbenchGajiPresentationState.pendingGajiInstall?.artifact) return;
  const origin = workbenchGajiPresentationState.pendingGajiInstall.artifact;
  const primaryPath = origin.provenance.extraction_modules[0];
  const files = origin.files.filter((file) => file.path.startsWith("ddn/")).map((file) => ({
    path: file.path.slice(4), role: "ddn_source", utf8_text: file.path === `ddn/${primaryPath}` ? els.gajiRemixSource.value : file.utf8_text,
  }));
  const moduleRows = ['(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn")',
    ...files.map((file, index) => `(이름=모듈${index + 1}, 머리="${ddnQuotedText(file.path)}", 원문="${ddnQuotedText(file.path)}")`)];
  files.push({
    path: "뿌리.ddn", role: "root",
    utf8_text: `(글판="v25", 모듬=차림[${moduleRows.join(", ")}], 시작=(모듈=앱, 정의=실행)) 얽음.`,
  });
  files.push({ path: "app/main.ddn", role: "ddn_source", utf8_text: '실행:움직씨 := { "리믹스 묶기" 보여주기. }.' });
  const request = {
    schema: "ddn.gaji_extraction_plan.v1",
    source: { schema: "ddn.virtual_project_source.v1", kind: "seamgrim.local_project_tree.v0", root_path: "뿌리.ddn", entry_path: "app/main.ddn", files },
    selected_modules: [...origin.provenance.extraction_modules],
    gaji_id: String(els.gajiRemixId.value).trim(), version: String(els.gajiRemixVersion.value).trim(),
    description: `${origin.description} 리믹스`, tags: [...origin.tags, "리믹스"], profile: "내",
    origin_project_ref: origin.provenance.origin_project_ref,
    origin_gaji: `${origin.gaji_id}@${origin.version}`, relation: "remix",
  };
  const remix = await runtimeBridge.buildLocalGajiArtifact(request);
  await publishToLocalGotgan(runtimeBridge, remix);
  const originalApi = origin.public_symbols.map((item) => item.external_name).sort().join(", ");
  const remixApi = remix.public_symbols.map((item) => item.external_name).sort().join(", ");
  els.gajiInstallPreview.textContent = [
    `원본 ${origin.gaji_id}@${origin.version}: ${origin.content_sha256}`,
    `리믹스 ${remix.gaji_id}@${remix.version}: ${remix.content_sha256}`,
    `API 원본: ${originalApi || "없음"}`,
    `API 리믹스: ${remixApi || "없음"}`,
    `계보: ${remix.provenance.origin_gaji} · ${remix.provenance.relation}`,
  ].join("\n");
  els.gajiRemixEditor.hidden = true;
  await refreshGajiShelf();
  els.gajiInstallStatus.textContent = `리믹스 발행 완료: ${remix.gaji_id}@${remix.version}`;
}

async function openGajiShelf() {
  workbenchGajiPresentationState.pendingGajiInstall = null;
  els.gajiInstallApply.disabled = true;
  els.gajiInstallRemove.disabled = true;
  els.gajiRemixStart.disabled = true;
  els.gajiRemixEditor.hidden = true;
  els.gajiInstallPreview.textContent = "설치할 가지를 선택하세요.";
  els.gajiShelfDialog?.showModal?.();
  await refreshGajiShelf();
  els.gajiShelfSearch?.focus?.();
}

function exportSelectedGajiArtifact() {
  const artifact = workbenchGajiPresentationState.pendingGajiInstall?.artifact;
  if (!artifact) throw new Error("E_GAJI_EXPORT_SELECTION_REQUIRED 내보낼 가지를 먼저 선택하세요");
  triggerPortableDownload({
    jsonText: exportLocalGotganArtifact(artifact),
    filename: `${artifact.gaji_id}-${artifact.version}.gaji.detjson`,
  });
  els.gajiInstallStatus.textContent = `가지 파일 내보내기 완료: ${artifact.gaji_id}@${artifact.version}`;
}

async function importGajiArtifactFile(file) {
  if (!file) return;
  const text = await hostBridge.readTextFile(file);
  const parsed = JSON.parse(text);
  await importLocalGotganArtifact(runtimeBridge, text);
  await refreshGajiShelf();
  els.gajiInstallStatus.textContent = `가지 파일 가져오기 완료: ${parsed.gaji_id}@${parsed.version}`;
}

  return { closeGajiBundleDialog, openGajiBundleDialog, previewGajiBundle, publishGajiBundle, currentCapabilityIsViewOnly, selectedShelfProfiles, refreshGajiShelf, reportGajiShelfError, previewShelfInstall, applyShelfInstall, removeShelfInstall, startShelfRemix, publishShelfRemix, openGajiShelf, exportSelectedGajiArtifact, importGajiArtifactFile };
}
