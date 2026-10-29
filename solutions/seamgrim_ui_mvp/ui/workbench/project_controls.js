// Existing authored-source control discovery and rendering; accepted inputs and timing unchanged.
export function createProjectControls({ buildProjectControlModel, currentProject, els, emptyProjectControlModel, normalizeActionMap, renderExplorationPresets, runtimeBridge, state }) {
function currentProjectDdnSources(project = currentProject()) {
  return (Array.isArray(project?.files) ? project.files : [])
    .filter((file) => file?.role !== "meta" && String(file?.path ?? "").toLowerCase().endsWith(".ddn"))
    .map((file) => ({
      source_path: String(file.path),
      source_text: String(state.fileTexts.get(file.path) ?? file.fallback ?? ""),
    }))
    .filter((file) => file.source_text.trim());
}

function declaredActionMapFromProject(project = currentProject()) {
  const candidates = [
    project?.portableSnapshot?.controls_snapshot?.action_map,
    project?.actionMap,
  ];
  for (const file of Array.isArray(project?.files) ? project.files : []) {
    const text = String(state.fileTexts.get(file.path) ?? file.fallback ?? "").trim();
    if (String(file.path ?? "").split("/").at(-1) === "action_map.json") {
      let declared;
      try { declared = JSON.parse(text); }
      catch { throw new Error("E_INPUT_ACTION_MAP_INVALID: 입력 설정을 읽을 수 없습니다. action_map.json을 확인하세요."); }
      const normalized = normalizeActionMap(declared, project?.id ?? state.projectId);
      if (!normalized) throw new Error("E_INPUT_ACTION_MAP_INVALID: 입력 설정의 형식·프로젝트·시간 간격을 확인하세요.");
      return normalized;
    }
    if (!text || !String(file.path ?? "").toLowerCase().endsWith(".json")) continue;
    try {
      candidates.push(JSON.parse(text));
    } catch (_) {
      // Other project JSON files are not action-map payloads.
    }
  }
  for (const candidate of candidates) {
    const normalized = normalizeActionMap(candidate, project?.id ?? state.projectId);
    if (normalized) return normalized;
  }
  return null;
}

async function refreshProjectControls(project = currentProject()) {
  const generation = state.projectControlsGeneration + 1;
  state.projectControlsGeneration = generation;
  const projectId = String(project?.id ?? state.projectId);
  state.projectControls = emptyProjectControlModel(projectId, "loading");
  renderProjectControls();
  renderExplorationPresets(project);
  try {
    const planRows = [];
    for (const { source_path, source_text } of currentProjectDdnSources(project)) {
      planRows.push({
        source_path,
        plan: await runtimeBridge.canonMaegimPlan(source_text),
      });
    }
    if (generation !== state.projectControlsGeneration || state.projectId !== projectId) return null;
    state.projectControls = buildProjectControlModel({
      projectId,
      planRows,
      actionMap: declaredActionMapFromProject(project),
    });
    renderProjectControls();
    return state.projectControls;
  } catch (err) {
    if (generation !== state.projectControlsGeneration || state.projectId !== projectId) return null;
    state.projectControls = {
      ...emptyProjectControlModel(projectId, "error"),
      error: {
        code: "E_PROJECT_AUTHORING_PLAN",
        message: String(err?.message ?? err),
      },
    };
    renderProjectControls();
    return state.projectControls;
  }
}

function scheduleProjectControlsRefresh() {
  clearTimeout(state.projectControlsRefreshTimer);
  state.projectControlsRefreshTimer = setTimeout(() => {
    refreshProjectControls(currentProject()).catch(() => {});
  }, 250);
}

function setupControlTestId(name) {
  const slug = String(name ?? "")
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}_-]+/gu, "-")
    .replace(/^-+|-+$/gu, "");
  return `setup-${slug || "control"}`;
}

function renderProjectControls() {
  if (!els.controlGrid || !els.setupControls || !els.actionControls) return;
  const model = state.projectControls ?? emptyProjectControlModel(state.projectId);
  const controls = model.project_id === state.projectId && Array.isArray(model.controls) ? model.controls : [];
  const actions = model.project_id === state.projectId && Array.isArray(model.actions) ? model.actions : [];
  els.controlGrid.hidden = !controls.length && !actions.length;
  els.setupControls.hidden = !controls.length;
  els.actionControls.hidden = !actions.length;
  els.root.dataset.projectControlStatus = String(model.status ?? "idle");
  els.root.dataset.projectControlCount = String(controls.length);
  els.root.dataset.projectActionCount = String(actions.length);
  els.root.dataset.projectControlSource = controls.length ? String(model.source ?? "") : "";
  els.root.dataset.projectAuthoringPlanHash = String(model.authoring_plan_hash ?? "");
  els.root.dataset.projectActionMapHash = String(model.action_map?.action_map_hash ?? "");

  if (els.setupControlList) {
    els.setupControlList.innerHTML = "";
    for (const control of controls) {
      const label = document.createElement("label");
      label.textContent = control.label;
      label.dataset.controlName = control.name;
      const input = document.createElement("input");
      input.type = "number";
      input.value = String(model.control_values?.[control.name] ?? control.value);
      input.min = String(control.min);
      input.max = String(control.max);
      input.step = String(control.step);
      input.dataset.controlName = control.name;
      input.dataset.controlSourcePath = control.source_path;
      input.dataset.testid = setupControlTestId(control.name);
      input.setAttribute("aria-label", `${control.label} 채비 조절값`);
      label.appendChild(input);
      els.setupControlList.appendChild(label);
    }
  }

  if (els.actionButtonList) {
    els.actionButtonList.innerHTML = "";
    for (const action of actions) {
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.actionId = action.action_id;
      button.dataset.testid = `action-${action.action_id}`;
      button.title = `${action.label} 입력`;
      button.setAttribute("aria-label", `${action.label} 입력`);
      const glyph = document.createElement("span");
      glyph.setAttribute("aria-hidden", "true");
      glyph.textContent = action.glyph;
      const text = document.createElement("span");
      text.textContent = action.label;
      button.append(glyph, text);
      els.actionButtonList.appendChild(button);
    }
  }

  els.setupStatus.textContent = state.pendingRecipe?.project_id === state.projectId
    ? "다음 실행에 적용할 초기값을 저장했습니다."
    : controls.length
      ? `조절 가능한 초기값 ${controls.length}개`
      : model.status === "error"
        ? "조절값을 읽지 못했습니다."
        : "이 프로젝트에는 조절값이 없습니다.";
  const queuedAction = actions.find((action) => action.action_id === state.nextInput?.action_id);
  els.actionStatus.textContent = state.nextInput
    ? `다음 마디 입력: ${queuedAction?.label ?? state.nextInput.intent}`
    : "대기 중인 입력이 없습니다.";
}

  return { currentProjectDdnSources, declaredActionMapFromProject, refreshProjectControls, scheduleProjectControlsRefresh, setupControlTestId, renderProjectControls };
}
