// Existing file drawer, source editor, completion and lesson presentation; original state and text retained.
export function createEditorPresentation({ activeFile, currentProject, displayedProjectTitle, els, escapeHtml, FIRST_STEPS_PROJECTS, localProjectTreePayloadFromSnapshot, runtimeBridge, selectFile, SOURCE_ANALYSIS_PROFILE_IDENTITY, state, workbenchProjectRegistryState, workbenchSourceProfile }) {
function renderFileDrawer(project) {
  const manyFiles = project.files.length > 1;
  els.fileDrawer.classList.remove("hidden"); // File commands stay available even for a single file.
  els.fileCount.textContent = `${project.files.length}`;
  els.fileList.innerHTML = "";
  const filter = String(state.fileFilter ?? "").trim().toLowerCase();
  project.files
    .filter((file) => !filter || file.path.toLowerCase().includes(filter) || file.role.toLowerCase().includes(filter))
    .forEach((file) => {
      const wrap = document.createElement("div");
      wrap.className = "file-row-wrap";
      const button = document.createElement("button");
      button.type = "button";
      button.className = `file-row${file.path === state.activeFilePath ? " active" : ""}`;
      button.dataset.filePath = file.path;
      button.setAttribute("data-testid", `file-row-${file.path.replace(/[^a-z0-9]/gi, "-")}`);
      button.innerHTML = `<strong>${escapeHtml(file.path)}</strong><span>${escapeHtml(file.role)}</span>`;
      button.addEventListener("click", () => selectFile(file.path));
      wrap.appendChild(button);
      if (workbenchProjectRegistryState.nativeProjects.some((item) => item.id === project.id)) {
        const actions = document.createElement("div");
        actions.className = "file-row-actions";
        actions.setAttribute("aria-label", `${file.path} 작업`);
        const definitions = [
          ["rename", "이름 변경", "✎"],
          ["duplicate", "복제", "⧉"],
          ["set_entry", "진입 파일 지정", "▶"],
          ["delete", "삭제", "×"],
        ];
        definitions.forEach(([action, label, glyph]) => {
          const actionButton = document.createElement("button");
          actionButton.type = "button";
          actionButton.className = "panel-action";
          actionButton.dataset.fileAction = action;
          actionButton.dataset.filePath = file.path;
          actionButton.title = `${file.path} ${label}`;
          actionButton.setAttribute("aria-label", `${file.path} ${label}`);
          actionButton.textContent = glyph;
          actions.appendChild(actionButton);
        });
        wrap.appendChild(actions);
      }
      els.fileList.appendChild(wrap);
    });
  renderProjectGraph();
}

function projectDiagnosticFix(code = "") {
  const fixes = {
    E_MODULE_MISSING: "쓰임 경로를 존재하는 DDN 파일로 바꾸거나 해당 모듈을 추가하세요.",
    E_IMPORT_CYCLE: "순환을 이루는 쓰임 중 하나를 제거하거나 공통 모듈로 분리하세요.",
    E_SYMBOL_NOT_EXPORTED: "대상 모듈의 드러냄 목록에 심볼을 추가하세요.",
    E_PROJECT_ENTRY_MISSING: "존재하는 DDN 파일을 진입 파일로 지정하세요.",
    E_GAJI_LOCK_STALE: "현재 project graph에서 lock을 다시 생성하세요.",
  };
  return fixes[code] || "표시된 파일과 경로를 확인한 뒤 다시 실행하세요.";
}

function renderProjectGraph() {
  if (!els.projectGraphSummary || !els.projectGraphList || !els.projectGraphDiagnostics) return;
  const row = state.projectGraph;
  els.projectGraphList.innerHTML = "";
  els.projectGraphDiagnostics.innerHTML = "";
  if (!row) {
    els.projectGraphSummary.textContent = "graph: 확인 전";
    return;
  }
  if (row.error) {
    els.projectGraphSummary.textContent = `graph: ${row.error.code || "오류"}`;
  } else {
    els.projectGraphSummary.textContent = `entry ${row.graph.entry_path} · files ${row.graph.file_count} · lock ${row.lockStatus}`;
    const edges = [
      `진입 → ${row.graph.entry_path}`,
      ...Object.entries(row.graph.module_paths ?? {}).map(([name, path]) => `모듈 ${name} → ${path}`),
      ...(row.symbols?.imports ?? []).map((item) => `${item.importer_path} —${item.alias}→ ${item.resolved_path}`),
      ...Object.entries(row.graph.gaji_aliases ?? {}).map(([name, path]) => `가지 ${name} → ${path}`),
    ];
    edges.forEach((text) => {
      const item = document.createElement("div");
      item.className = "project-graph-edge";
      item.textContent = text;
      els.projectGraphList.appendChild(item);
    });
  }
  const diagnostics = row.error ? [row.error] : [...(row.graph?.diagnostics ?? []), ...(row.symbols?.diagnostics ?? [])];
  diagnostics.forEach((diagnostic) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "project-graph-diagnostic";
    button.dataset.severity = "error";
    button.textContent = `${diagnostic.code}: ${diagnostic.message}\n위치: ${diagnostic.path || "프로젝트"}\n고침: ${projectDiagnosticFix(diagnostic.code)}`;
    if (diagnostic.path && currentProject().files.some((file) => file.path === diagnostic.path)) {
      button.addEventListener("click", () => selectFile(diagnostic.path));
    } else {
      button.disabled = true;
    }
    els.projectGraphDiagnostics.appendChild(button);
  });
}

function renderSource(project) {
  const file = activeFile(project);
  els.sourceTitle.textContent = file.path;
  els.sourceEditor.value = state.fileTexts.get(file.path) ?? "";
}

function completionPosition(source, offset) {
  const before = source.slice(0, offset);
  const line = before.lastIndexOf("\n") + 1;
  return {
    line: before.slice(0, line).split("\n").length - 1,
    character: before.slice(line).length,
  };
}

function completionOffset(source, position) {
  const lines = source.split("\n");
  if (position.line < 0 || position.line >= lines.length) return null;
  const lineStart = lines.slice(0, position.line).reduce((total, line) => total + line.length + 1, 0);
  if (position.character < 0 || position.character > lines[position.line].length) return null;
  return lineStart + position.character;
}

function closeCompletionPopup() {
  state.completionGeneration += 1;
  state.completionItems = [];
  state.completionSelectedIndex = 0;
  if (els.completionPopup) {
    els.completionPopup.hidden = true;
    els.completionPopup.replaceChildren();
  }
  els.sourceEditor?.removeAttribute("aria-activedescendant");
}

function renderCompletionPopup() {
  const popup = els.completionPopup;
  if (!popup || !state.completionItems.length) {
    if (popup) popup.hidden = true;
    els.sourceEditor?.removeAttribute("aria-activedescendant");
    return;
  }
  popup.replaceChildren();
  popup.hidden = false;
  state.completionItems.forEach((item, index) => {
    const option = document.createElement("button");
    option.type = "button";
    option.className = "completion-option";
    option.id = `completion-option-${index}`;
    option.setAttribute("role", "option");
    option.setAttribute("aria-selected", index === state.completionSelectedIndex ? "true" : "false");
    option.dataset.testid = `completion-option-${index}`;
    const label = document.createElement("span");
    label.textContent = String(item.label ?? "");
    const detail = document.createElement("span");
    detail.className = "completion-option-detail";
    detail.textContent = String(item.canonical_detail ?? "").replace(/^local definition$/u, "이 원문에서 정의한 이름");
    option.append(label, detail);
    option.addEventListener("mousedown", (event) => event.preventDefault());
    option.addEventListener("click", () => selectCompletion(index));
    popup.appendChild(option);
  });
  els.sourceEditor.setAttribute("aria-activedescendant", `completion-option-${state.completionSelectedIndex}`);
}

function selectCompletion(index = state.completionSelectedIndex) {
  const item = state.completionItems[index];
  if (!item) return;
  const source = els.sourceEditor.value;
  const start = completionOffset(source, item.replacement_range?.start);
  const end = completionOffset(source, item.replacement_range?.end);
  if (start === null || end === null || start > end) {
    closeCompletionPopup();
    return;
  }
  const next = `${source.slice(0, start)}${String(item.insert_text ?? "")}${source.slice(end)}`;
  els.sourceEditor.value = next;
  els.sourceEditor.setSelectionRange(start + String(item.insert_text ?? "").length, start + String(item.insert_text ?? "").length);
  closeCompletionPopup();
  els.sourceEditor.dispatchEvent(new Event("input", { bubbles: true }));
  els.sourceEditor.focus();
}

async function requestCompletion() {
  const file = activeFile();
  const source = els.sourceEditor?.value ?? "";
  if (!file?.path?.endsWith(".ddn") || els.sourceEditor?.readOnly
      || document.activeElement !== els.sourceEditor) {
    closeCompletionPopup();
    return;
  }
  const generation = ++state.completionGeneration;
  const version = state.completionDocumentVersion;
  const projectId = state.projectId;
  const filePath = file.path;
  const cursor = els.sourceEditor.selectionStart ?? source.length;
  const result = await runtimeBridge.sharedCompletion({
    schema: "ddonilang.completion.v1-candidate",
    profile: "v1-core-v25",
    document_identity: `seamgrim://${encodeURIComponent(currentProject().id)}/${encodeURIComponent(file.path)}`,
    document_version: version,
    source,
    position: completionPosition(source, cursor),
    encoding: "utf-16",
  }).catch(() => null);
  if (
    generation !== state.completionGeneration
    || projectId !== state.projectId
    || filePath !== state.activeFilePath
    || document.activeElement !== els.sourceEditor
    || version !== state.completionDocumentVersion
    || source !== els.sourceEditor.value
    || !result
  ) return;
  state.completionItems = Array.isArray(result.candidates) ? result.candidates : [];
  state.completionSelectedIndex = 0;
  renderCompletionPopup();
}

function renderProjectMeta(project) {
  els.title.textContent = displayedProjectTitle(project);
  const profile = workbenchSourceProfile(project);
  const payload = localProjectTreePayloadFromSnapshot(project.portableSnapshot ?? {});
  const head = payload ? ` · 실행 대상: ${String(payload.root_path ?? "미지정")}` : "";
  els.kind.textContent = `${project.kind} · ${profile === SOURCE_ANALYSIS_PROFILE_IDENTITY ? "글판 v25 ↔ v1-core-v25" : "글판 미확인"}${head}`;
  els.summary.textContent = project.summary;
  renderFirstSteps(project);
}

function renderFirstSteps(project) {
  const panel = document.querySelector("[data-testid='first-steps-panel']");
  const guide = document.querySelector("[data-testid='first-steps-guide']");
  if (!panel || !guide) return;
  const row = project.course;
  if (row) {
    const documents=panel.querySelectorAll('.first-steps-documents a');
    documents[0].href=row.asset_root ? `${row.asset_root}${row.student_pdf ?? '학생용_첫걸음_11-16차시.pdf'}` : '../../first_steps/학생용_첫걸음_10차시.pdf';
    documents[1].href=row.asset_root ? `${row.asset_root}${row.teacher_pdf ?? '교사용_안내와_해답.pdf'}` : '../../first_steps/교사용_안내와_해답.pdf';
  }
  guide.classList.toggle("hidden", !row);
  if (!row) {
    guide.open = false;
    return;
  }
  panel.querySelector("[data-testid='first-steps-title']").textContent = `${row.id} · ${row.title}`;
  panel.querySelector("[data-testid='first-steps-reason']").textContent = row.reason;
  panel.querySelector('[data-testid="first-steps-input-contract"]').hidden=!row.binding_file;
  const expected = panel.querySelector("[data-testid='first-steps-expected']");
  if (expected.dataset.courseId !== row.id) {
    expected.open = false;
    expected.dataset.courseId = row.id;
  }
  expected.querySelector("pre").textContent = row.expected_description ?? (Array.isArray(row.expected)
    ? row.expected.join("\n") : `의도한 오류: ${row.expected.reject}`);
  const index = FIRST_STEPS_PROJECTS.findIndex(item => item.id === project.id);
  panel.querySelector("[data-testid='first-steps-previous']").disabled = index <= 0;
  panel.querySelector("[data-testid='first-steps-next']").disabled = index >= FIRST_STEPS_PROJECTS.length - 1;
}

  return { renderFileDrawer, projectDiagnosticFix, renderProjectGraph, renderSource, completionPosition, completionOffset, closeCompletionPopup, renderCompletionPopup, selectCompletion, requestCompletion, renderProjectMeta, renderFirstSteps };
}
