// Project rail, my-work index and file tabs share the original state objects.
export function createProjectNavigation({ state, els, MY_WORK_INDEX_KEY, localProjectEntryPath, currentProject, openServerProject, readPortableProjectCatalog, projectLink, escapeHtml, projectsForGlobalEntry, includesFilter, DEFERRED_LESSONS, selectProject, selectFile }) {
function displayedProjectTitle(project) {
  const path = localProjectEntryPath(project);
  const loaded = state.loadedFileTexts.get(path);
  return loaded !== undefined && state.fileTexts.get(path) !== loaded
    ? `${project.title} · 수정한 원문` : project.title;
}

function renderGlobalNav() {
  document.querySelectorAll("[data-global-entry]").forEach((button) => {
    button.classList.toggle("active", button.dataset.globalEntry === state.globalEntry);
  });
  const myWork = state.globalEntry === "my-work";
  els.myWorkPanel.classList.toggle("hidden", !myWork);
  els.projectList.classList.toggle("hidden", myWork);
  renderMyWork();
}

function setProjectRailOpen(open) {
  const isOpen = Boolean(open);
  els.root.dataset.projectRailOpen = isOpen ? "true" : "false";
  els.projectRailToggle?.setAttribute("aria-expanded", isOpen ? "true" : "false");
  els.projectRailToggle?.setAttribute("title", isOpen ? "프로젝트 목록 닫기" : "프로젝트 목록 열기");
}

function readMyWorkIndex() {
  try {
    const parsed = JSON.parse(globalThis.localStorage?.getItem?.(MY_WORK_INDEX_KEY) ?? "[]");
    return Array.isArray(parsed) ? parsed : [];
  } catch (_) {
    return [];
  }
}

function writeMyWorkIndex(rows = []) {
  globalThis.localStorage?.setItem?.(MY_WORK_INDEX_KEY, JSON.stringify(rows.slice(0, 20)));
}

function rememberMyWorkProject({
  projectId = "",
  title = "",
  headRevisionId = "",
  editCapability = "",
  viewCapability = "",
  source = "",
  role = "",
  nativeLocator = "",
  logicalInput = null,
} = {}) {
  const normalizedProjectId = String(projectId || "").trim();
  if (!normalizedProjectId) return;
  const next = {
    schema: "seamgrim.workbench.my_work_entry.v1",
    project_id: normalizedProjectId,
    title: String(title || currentProject().title || normalizedProjectId),
    head_revision_id: String(headRevisionId || ""),
    edit_capability: String(editCapability || ""),
    view_capability: String(viewCapability || ""),
    origin_kind: String(source || (editCapability || viewCapability ? "server_saved" : "local_only")),
    access_role: String(role || (editCapability ? "edit" : viewCapability ? "view" : "edit")),
    updated_at: new Date().toISOString(),
    ...(nativeLocator?{native_source_locator:nativeLocator,native_input_profile:logicalInput?{...logicalInput}:null}:{}),
  };
  const rows = readMyWorkIndex().filter((row) => String(row.project_id ?? "") !== normalizedProjectId);
  writeMyWorkIndex([next, ...rows]);
  state.myWorkRows = [next, ...rows];
}

async function refreshMyWorkIndex() {
  const rows = readMyWorkIndex();
  const verified = [];
  for (const row of rows) {
    const capability = String(row.edit_capability || row.view_capability || "");
    const projectId = String(row.project_id || "");
    let status = row.origin_kind==='native_source'?'Native 저장 위치 · 다시 열 때 확인':"server 확인 전";
    let head = String(row.head_revision_id || "");
    if (projectId && capability) {
      try {
        const payload = await openServerProject({ projectId, capability });
        status = "server 확인됨";
        head = String(payload.head_revision_id || head);
      } catch (err) {
        status = err?.status === 404 ? "삭제됨 또는 만료됨" : err?.status === 403 || err?.status === 401 ? "권한 만료" : "서버 확인 실패";
      }
    }
    verified.push({ ...row, verify_status: status, head_revision_id: head });
  }
  state.myWorkRows = verified;
  renderMyWork();
  return verified;
}

function renderMyWork() {
  if (!els.myWorkList) return;
  const filter = String(state.projectFilter || "").trim().toLowerCase();
  const rows = (state.myWorkRows.length ? state.myWorkRows : readMyWorkIndex())
    .filter((row) => state.showHiddenMyWork || !row.hidden_from_list)
    .filter((row) => !filter || String(row.title || row.project_id).toLowerCase().includes(filter));
  const portableRows = readPortableProjectCatalog()
    .filter((row) => state.showHiddenMyWork || !row.hidden_from_list)
    .filter((row) => !filter || String(row.title || row.project_id).toLowerCase().includes(filter));
  els.myWorkCount.textContent = `${rows.length + portableRows.length}`;
  els.myWorkList.innerHTML = "";
  const visibility = document.createElement("button");
  visibility.type = "button";
  visibility.dataset.myWorkVisibility = "toggle";
  visibility.textContent = state.showHiddenMyWork ? "숨긴 작업 접기" : "숨긴 작업 보기";
  els.myWorkList.appendChild(visibility);
  if (!rows.length && !portableRows.length) {
    const empty = document.createElement("span");
    empty.textContent = "표시할 작업 없음";
    els.myWorkList.appendChild(empty);
    return;
  }
  rows.forEach((row) => {
    const capability = String(row.edit_capability || row.view_capability || "");
    const role = row.edit_capability ? "edit" : "view";
    const href = capability
      ? projectLink({ projectId: row.project_id, role, capability })
      : "#";
    const link = document.createElement("a");
    link.className = "my-work-item";
    link.href = href;
    link.setAttribute("data-testid", "my-work-item");
    link.dataset.projectId = String(row.project_id || "");
    const origin = row.origin_kind==='native_source'?'Native 저장':row.origin_kind === "local_only" ? "로컬 전용" : "서버 저장";
    const access = row.access_role === "view" || (!row.edit_capability && row.view_capability) ? "보기 전용" : "편집 가능";
    link.innerHTML = `<strong>${escapeHtml(row.title || row.project_id)}</strong><span>${origin} · ${access} · ${escapeHtml(row.verify_status || (origin === "로컬 전용" ? "이 기기" : "server 확인 전"))} · head ${escapeHtml(row.head_revision_id || "-")}</span>`;
    appendMyWorkRow(link, row, "index");
  });
  portableRows.forEach((row) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "my-work-item";
    button.setAttribute("data-testid", "my-work-item");
    button.dataset.portableProjectId = String(row.project_id || "");
    button.innerHTML = `<strong>${escapeHtml(row.title || row.project_id)}</strong><span>받은 프로젝트 · 편집 가능 복제본 · ${escapeHtml(String(row.content_sha256 || "").slice(0, 12) || "-")}</span>`;
    appendMyWorkRow(button, row, "portable");
  });
}

function appendMyWorkRow(entry, row, store) {
  const wrapper = document.createElement("div");
  wrapper.className = "my-work-row";
  const button = document.createElement("button");
  button.type = "button";
  button.dataset.myWorkVisibility = store;
  button.dataset.projectId = String(row.project_id || "");
  button.textContent = row.hidden_from_list ? "다시 표시" : "목록에서 숨기기";
  button.title = "원문과 저장 기록은 삭제하지 않습니다";
  wrapper.append(entry, button);
  els.myWorkList.appendChild(wrapper);
}

function renderProjects() {
  const visible = projectsForGlobalEntry().filter((project) => includesFilter(project, state.projectFilter));
  els.projectCount.textContent = `${visible.length}`;
  els.projectList.innerHTML = "";
  visible.forEach((project) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = `project-card${project.id === state.projectId ? " active" : ""}`;
    button.dataset.projectId = project.id;
    button.setAttribute("role", "option");
    button.setAttribute("aria-selected", project.id === state.projectId ? "true" : "false");
    button.setAttribute("data-testid", `project-card-${project.id}`);
    button.innerHTML = `<strong>${escapeHtml(project.title)}</strong><span>${escapeHtml(project.kind)} · ${escapeHtml(project.tags.join(", "))}</span><span>${escapeHtml(project.lessonDisposition || (project.logicalInput ? "키보드로 실행" : "실행 작품"))}</span>`;
    button.addEventListener("click", () => selectProject(project.id));
    els.projectList.appendChild(button);
  });
  if (state.globalEntry === "official-collection") {
    const deferred = DEFERRED_LESSONS.filter(project => includesFilter(project, state.projectFilter));
    for (const lesson of deferred) {
      const card = document.createElement("div");
      card.className = "project-card";
      card.dataset.lessonDisposition = "이월";
      card.dataset.lessonId = lesson.id;
      card.setAttribute("data-testid", `deferred-lesson-${lesson.id}`);
      // No click handler: historical event syntax is not an alpha1 run target.
      card.innerHTML = `<strong>${escapeHtml(lesson.title)}</strong><span>이월 · ${escapeHtml(lesson.stage)}</span><span>${escapeHtml(lesson.reason)}이 필요합니다. 원 교재는 보관합니다.</span>`;
      els.projectList.appendChild(card);
    }
    els.projectCount.textContent = `${visible.length} 실행 · ${deferred.length} 이월`;
  }
}

function renderFileTabs(project) {
  const visibleTabs = project.files.slice(0, Math.min(3, project.files.length));
  els.fileTabs.innerHTML = "";
  if (project.files.length <= 1) {
    els.fileTabs.classList.add("hidden");
    return;
  }
  els.fileTabs.classList.remove("hidden");
  visibleTabs.forEach((file) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = `file-tab${file.path === state.activeFilePath ? " active" : ""}`;
    button.dataset.filePath = file.path;
    button.setAttribute("data-testid", `file-tab-${file.path.replace(/[^a-z0-9]/gi, "-")}`);
    button.textContent = file.path;
    button.addEventListener("click", () => selectFile(file.path));
    els.fileTabs.appendChild(button);
  });
}

  return { displayedProjectTitle, renderGlobalNav, setProjectRailOpen, readMyWorkIndex, writeMyWorkIndex, rememberMyWorkProject, refreshMyWorkIndex, renderMyWork, appendMyWorkRow, renderProjects, renderFileTabs };
}
