const BLANK_DDN_TEMPLATE = `설정 {
  제목: 새_교과.
  설명: "채비를 조절하면서 결과를 확인하세요.".
}.

채비 {
  계수:수 <- 1.
  프레임수:수 <- 0.
  t:수 <- 0.
  y:수 <- 0.
}.

(매마디)마다 {
  t <- 프레임수.
  y <- (계수 * t).
  t 보여주기.
  y 보여주기.
  프레임수 <- (프레임수 + 1).
}.`;

const STUDIO_READINESS_STAGE_READY = "ready";
const STUDIO_READINESS_STAGE_AUTOFIX = "autofix";
const STUDIO_READINESS_STAGE_BLOCKED = "blocked";

function normalizeReadinessStage(raw) {
  const text = String(raw ?? "").trim().toLowerCase();
  if (text === STUDIO_READINESS_STAGE_AUTOFIX) return STUDIO_READINESS_STAGE_AUTOFIX;
  if (text === STUDIO_READINESS_STAGE_BLOCKED) return STUDIO_READINESS_STAGE_BLOCKED;
  return STUDIO_READINESS_STAGE_READY;
}

function normalizeReadinessModel(rawModel = null) {
  const model = rawModel && typeof rawModel === "object" ? rawModel : {};
  const stage = normalizeReadinessStage(model.stage);
  const cause = String(model.user_cause ?? "").trim();
  const primary = model.primary_action && typeof model.primary_action === "object"
    ? model.primary_action
    : {};
  const buttonLabel = String(primary.label ?? "").trim();
  const buttonDetail = String(primary.detail ?? "").trim();
  const actionKind = String(primary.kind ?? "").trim().toLowerCase();
  return {
    stage,
    user_cause: cause || (stage === STUDIO_READINESS_STAGE_BLOCKED ? "실행 전 수정이 필요합니다." : "입력 준비됨"),
    primary_action: {
      kind: actionKind || (stage === STUDIO_READINESS_STAGE_BLOCKED ? "manual_fix_example" : "run"),
      label: buttonLabel || (stage === STUDIO_READINESS_STAGE_AUTOFIX ? "자동 수정 적용" : "수업 실행"),
      detail: buttonDetail,
    },
    autofix_available: Boolean(model.autofix_available),
    blocking_remaining: Boolean(model.blocking_remaining),
    manual_example: String(model.manual_example ?? "").trim(),
  };
}

function buildReadinessDiagnosticLayers(model) {
  const stage = normalizeReadinessStage(model?.stage);
  const cause = String(model?.user_cause ?? "").trim();
  const detail = String(model?.manual_example || model?.primary_action?.detail || "").trim();
  if (stage === STUDIO_READINESS_STAGE_AUTOFIX) {
    return {
      what: cause || "구식 표면이 감지되었습니다.",
      where: "세계 코드 입력 · 자동 수정 후보",
      how: detail ? `수정 적용을 눌러 현재 제품 표면으로 바꾸세요. ${detail}` : "수정 적용을 눌러 현재 제품 표면으로 바꾼 뒤 실행하세요.",
    };
  }
  if (stage === STUDIO_READINESS_STAGE_BLOCKED) {
    return {
      what: cause || "실행 전 수정이 필요합니다.",
      where: "세계 코드 입력 · 실행 전 검사",
      how: detail || "표시된 원인을 고친 뒤 다시 실행하세요.",
    };
  }
  return {
    what: cause || "현재 입력으로 실행할 수 있습니다.",
    where: "세계 코드 입력",
    how: detail || "수업 실행을 누르세요.",
  };
}

function extractFocusMatchLabel(text) {
  const normalized = String(text ?? "").trim();
  if (!normalized) return "";
  const withoutPrefix = normalized.replace(/^\/\/\s*매김 전환 후보:\s*/, "");
  const match = withoutPrefix.match(/^([A-Za-z0-9_가-힣]+)(?::[A-Za-z0-9_가-힣]+)?\s*(<-|=)/);
  return match ? String(match[1] ?? "").trim() : "";
}

function extractFocusMatchType(text) {
  const normalized = String(text ?? "").trim();
  if (!normalized) return "";
  const withoutPrefix = normalized.replace(/^\/\/\s*매김 전환 후보:\s*/, "");
  const match = withoutPrefix.match(/^[A-Za-z0-9_가-힣]+:([A-Za-z0-9_가-힣]+)/);
  return match ? String(match[1] ?? "").trim() : "";
}

function extractFocusMatchAssign(text) {
  const normalized = String(text ?? "").trim();
  if (!normalized) return "";
  const withoutPrefix = normalized.replace(/^\/\/\s*매김 전환 후보:\s*/, "");
  const match = withoutPrefix.match(/^[A-Za-z0-9_가-힣]+(?::[A-Za-z0-9_가-힣]+)?\s*(<-|=)/);
  return match ? String(match[1] ?? "").trim() : "";
}

function extractFocusMatchRange(text) {
  const normalized = String(text ?? "").trim();
  if (!normalized) return "";
  const match = normalized.match(/범위:\s*(.*?)\.\s*간격:/) ?? normalized.match(/범위:\s*(.*?)\.\s*[}]/);
  return match ? String(match[1] ?? "").trim() : "";
}

function extractFocusMatchStep(text) {
  const normalized = String(text ?? "").trim();
  if (!normalized) return "";
  const match = normalized.match(/간격:\s*(.*?)\.\s*[}]/);
  return match ? String(match[1] ?? "").trim() : "";
}

function findGuideHeaderRange(text) {
  const normalized = String(text ?? "");
  if (!normalized) return null;
  const lines = normalized.split("\n");
  let offset = 0;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.includes("// --- 매김 전환 초안 ---")) {
      let end = offset + line.length;
      for (let next = index + 1, nextOffset = end + 1; next < lines.length; next += 1) {
        const candidate = lines[next];
        if (!candidate.startsWith("//")) break;
        end = nextOffset + candidate.length;
        nextOffset = end + 1;
      }
      return { start: offset, end };
    }
    offset += line.length + 1;
  }
  return null;
}

function escapeRegex(text) {
  return String(text ?? "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function findLineRangeByRegex(text, regex) {
  const sourceText = String(text ?? "");
  const match = sourceText.match(regex);
  if (!match || typeof match.index !== "number") return null;
  const start = match.index;
  const body = String(match[0] ?? "");
  return {
    start,
    end: start + body.length,
  };
}

function escapeText(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function escapeAttribute(value) {
  return escapeText(value).replace(/"/g, "&quot;");
}

const VIEW_AUTHORING_CAMERA_POLICIES = Object.freeze(["fixed", "fit", "follow"]);
const VIEW_AUTHORING_MEASURE_COMPONENTS = Object.freeze(["angle", "length", "vector", "axes", "trail"]);
const VIEW_AUTHORING_ANCHORS = Object.freeze(["obs.theta", "obs.angular_velocity", "obs.speed", "obs.pivot", "obs.bob", "obs.time"]);
const VIEW_AUTHORING_CONDITION_KINDS = Object.freeze(["zero_crossing", "sign_change", "extremum", "tolerance_entry"]);
const VIEW_AUTHORING_ACTION_KINDS = Object.freeze(["highlight", "visibility", "camera_focus"]);

function stableCanonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableCanonicalJson(item)).join(",")}]`;
  }
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableCanonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function fnv1a32(text = "") {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function normalizeList(value, allowed) {
  const set = new Set(Array.isArray(value) ? value.map((item) => String(item ?? "").trim()).filter(Boolean) : []);
  return allowed.filter((item) => set.has(item));
}

function normalizeViewReactionAuthoringModel(raw = null) {
  const source = raw && typeof raw === "object" ? raw : {};
  const cameraPolicy = String(source.camera_policy ?? "").trim();
  const semanticAnchor = String(source.semantic_anchor ?? "").trim();
  const conditionKind = String(source.condition_kind ?? "").trim();
  const actionKind = String(source.action_kind ?? "").trim();
  const priority = Math.max(0, Math.trunc(Number(source.priority ?? 10) || 0));
  const rules = Array.isArray(source.rules) ? source.rules : [];
  return {
    schema: "seamgrim.view_reaction_authoring.ui_state.v1",
    camera_policy: VIEW_AUTHORING_CAMERA_POLICIES.includes(cameraPolicy) ? cameraPolicy : "fit",
    measurement_components: normalizeList(source.measurement_components, VIEW_AUTHORING_MEASURE_COMPONENTS).length
      ? normalizeList(source.measurement_components, VIEW_AUTHORING_MEASURE_COMPONENTS)
      : [...VIEW_AUTHORING_MEASURE_COMPONENTS],
    semantic_anchor: VIEW_AUTHORING_ANCHORS.includes(semanticAnchor) ? semanticAnchor : "obs.bob",
    condition_kind: VIEW_AUTHORING_CONDITION_KINDS.includes(conditionKind) ? conditionKind : "extremum",
    action_kind: VIEW_AUTHORING_ACTION_KINDS.includes(actionKind) ? actionKind : "highlight",
    priority,
    rules: rules.map((rule, index) => normalizeViewReactionRule(rule, index)).filter(Boolean),
  };
}

function normalizeViewReactionRule(rule, index = 0) {
  const source = rule && typeof rule === "object" ? rule : {};
  const conditionKind = String(source.condition_kind ?? "").trim();
  const actionKind = String(source.action_kind ?? "").trim();
  const targetAnchor = String(source.semantic_anchor ?? source.target_anchor ?? "").trim();
  return {
    rule_id: String(source.rule_id ?? `author.rule.${index + 1}`).trim() || `author.rule.${index + 1}`,
    condition_kind: VIEW_AUTHORING_CONDITION_KINDS.includes(conditionKind) ? conditionKind : "extremum",
    action_kind: VIEW_AUTHORING_ACTION_KINDS.includes(actionKind) ? actionKind : "highlight",
    semantic_anchor: VIEW_AUTHORING_ANCHORS.includes(targetAnchor) ? targetAnchor : "obs.bob",
    priority: Math.max(0, Math.trunc(Number(source.priority ?? 10) || 0)),
    target_property: String(source.target_property ?? actionKindToTargetProperty(actionKind)).trim(),
  };
}

function actionKindToTargetProperty(kind) {
  const actionKind = String(kind ?? "").trim();
  if (actionKind === "visibility") return "visibility";
  if (actionKind === "camera_focus") return "camera_focus";
  return "style";
}

function buildViewReactionAuthoringArtifact(model) {
  const normalized = normalizeViewReactionAuthoringModel(model);
  const rules = normalized.rules.length
    ? normalized.rules
    : [normalizeViewReactionRule({
      rule_id: "author.rule.1",
      condition_kind: normalized.condition_kind,
      action_kind: normalized.action_kind,
      semantic_anchor: normalized.semantic_anchor,
      priority: normalized.priority,
    }, 0)];
  const diagnostics = [];
  const byWrite = new Map();
  for (const rule of rules) {
    const writeKey = `${rule.semantic_anchor}::${rule.target_property}::${rule.priority}`;
    const rows = byWrite.get(writeKey) ?? [];
    rows.push(rule);
    byWrite.set(writeKey, rows);
  }
  for (const [writeKey, rows] of byWrite.entries()) {
    if (rows.length <= 1) continue;
    diagnostics.push({
      code: "E_VIEW_AUTHOR_REACTION_CONFLICT",
      severity: "error",
      message: "같은 닻/속성/우선순위 반응 충돌입니다. 암묵 last-write-wins를 적용하지 않습니다.",
      write_key: writeKey,
      rule_ids: rows.map((row) => row.rule_id).sort(),
      fix: "우선순위를 다르게 지정하거나 같은 속성을 바꾸는 반응을 하나만 남기세요.",
    });
  }
  const artifact = {
    schema: "seamgrim.view_reaction_authoring.v1",
    source: "editor.lesson_plan_authoring",
    camera: {
      allowed_policies: [...VIEW_AUTHORING_CAMERA_POLICIES],
      selected_policy: normalized.camera_policy,
    },
    measurements: {
      allowed_components: [...VIEW_AUTHORING_MEASURE_COMPONENTS],
      selected_components: normalized.measurement_components,
    },
    semantic_anchor: normalized.semantic_anchor,
    supported_conditions: [...VIEW_AUTHORING_CONDITION_KINDS],
    supported_actions: [...VIEW_AUTHORING_ACTION_KINDS],
    reaction_rules: rules,
    implicit_lww_allowed: false,
    state_hash_participation: false,
    live_online_reaction_claim: false,
    sealed_trace_replay_scope: true,
    diagnostics,
  };
  return {
    ...artifact,
    artifact_hash: `candidate:fnv1a32:view_reaction_authoring:${fnv1a32(stableCanonicalJson(artifact))}`,
  };
}

export function findFlatInstanceSelectionRange(text, instanceName) {
  const name = String(instanceName ?? "").trim();
  if (!name) return null;
  return findLineRangeByRegex(
    text,
    new RegExp(`^\\s*${escapeRegex(name)}\\s*<-.*$`, "mu"),
  );
}

export function findFlatLinkSelectionRange(text, link) {
  const dstInstance = String(link?.dstInstance ?? "").trim();
  const dstPort = String(link?.dstPort ?? "").trim();
  const srcInstance = String(link?.srcInstance ?? "").trim();
  const srcPort = String(link?.srcPort ?? "").trim();
  if (!dstInstance || !dstPort || !srcInstance || !srcPort) return null;
  return findLineRangeByRegex(
    text,
    new RegExp(
      `^\\s*${escapeRegex(dstInstance)}\\s*\\.\\s*${escapeRegex(dstPort)}\\s*<-\\s*${escapeRegex(srcInstance)}\\s*\\.\\s*${escapeRegex(srcPort)}\\s*\\.?\\s*$`,
      "mu",
    ),
  );
}

const LOCAL_PROJECT_ENTRY_PATH = "app/main.ddn";
const LOCAL_PROJECT_ROOT_PATH = "뿌리.ddn";
const LOCAL_PROJECT_LOCK_PATH = "ddn.lock";

export class EditorScreen {
  constructor({ root, onBack, onRun, onSave, onSaveLessonPlan, onSelectLesson, onOpenAdvanced, onSourceChange, onAutofix, onApplyFixit, onProjectRunSummary } = {}) {
    this.root = root;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onRun = typeof onRun === "function" ? onRun : () => {};
    this.onSave = typeof onSave === "function" ? onSave : () => {};
    this.onSaveLessonPlan = typeof onSaveLessonPlan === "function" ? onSaveLessonPlan : () => {};
    this.onSelectLesson = typeof onSelectLesson === "function" ? onSelectLesson : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};
    this.onSourceChange = typeof onSourceChange === "function" ? onSourceChange : () => {};
    this.onAutofix = typeof onAutofix === "function" ? onAutofix : null;
    this.onApplyFixit = typeof onApplyFixit === "function" ? onApplyFixit : null;
    this.onProjectRunSummary = typeof onProjectRunSummary === "function" ? onProjectRunSummary : null;
    this.readOnly = false;
    this.focusMatches = [];
    this.focusIndex = -1;
    this.guideHeaderRange = null;
    this.warningShowsGuide = false;
    this.readinessModel = normalizeReadinessModel(null);
    this.fixitModel = null;
    this.editorMode = "world-code";
    this.expanded = false;
    this.lessonPlanModel = parseLessonPlanToml(buildBlankLessonPlanToml());
    this.lessonPlanText = serializeLessonPlanToml(this.lessonPlanModel);
    this.lessonPlanSelectedIndex = 0;
    this.lessonChoices = [];
    this.viewReactionAuthoringModel = normalizeViewReactionAuthoringModel(null);
    this.viewReactionAuthoringArtifact = buildViewReactionAuthoringArtifact(this.viewReactionAuthoringModel);
    this.projectMode = false;
    this.projectFiles = new Map();
    this.currentProjectPath = LOCAL_PROJECT_ENTRY_PATH;
    this.projectEntryPath = LOCAL_PROJECT_ENTRY_PATH;
    this.projectRunSummary = null;
    this.projectLastGraphHash = "";
    this.projectRunSeq = 0;
    this.projectRuntimeStatus = "idle";
    this.projectActiveRunId = "";
    this.projectRunHistory = [];
    this.projectLoadedPayload = null;
    this.projectLoadedPayload = null;
  }

  init() {
    this.titleEl = this.root.querySelector("#editor-title");
    this.textarea = this.root.querySelector("#ddn-textarea");
    this.resultEl = this.root.querySelector("#editor-smoke-result");
    this.canonSummaryEl = this.root.querySelector("#editor-canon-summary");
    this.canonTopoEl = this.root.querySelector("#editor-canon-topo");
    this.canonInstancesEl = this.root.querySelector("#editor-canon-instances");
    this.canonLinksEl = this.root.querySelector("#editor-canon-links");
    this.focusBarEl = this.root.querySelector("#editor-focus-bar");
    this.focusSummaryEl = this.root.querySelector("#editor-focus-summary");
    this.focusWarningEl = this.root.querySelector("#editor-focus-warning");
    this.focusSelectEl = this.root.querySelector("#editor-focus-select");
    this.focusPrevBtn = this.root.querySelector("#btn-editor-focus-prev");
    this.focusNextBtn = this.root.querySelector("#btn-editor-focus-next");
    this.runGateReasonEl = this.root.querySelector("#editor-run-gate-reason");
    this.readinessCardEl = this.root.querySelector("#editor-readiness-card");
    this.readinessStageEl = this.root.querySelector("#editor-readiness-stage");
    this.readinessCauseEl = this.root.querySelector("#editor-readiness-cause");
    this.readinessDiagnosticEl = this.root.querySelector("#editor-readiness-diagnostic");
    this.readinessActionBtn = this.root.querySelector("#btn-editor-readiness-action");
    this.fixitCardEl = this.root.querySelector("#editor-fixit-card");
    this.fixitSummaryEl = this.root.querySelector("#editor-fixit-summary");
    this.fixitApplyBtn = this.root.querySelector("#btn-editor-fixit-apply");
    this.loadFileInputEl = this.root.querySelector("#input-editor-ddn-file");
    this.expandBtn = this.root.querySelector("#btn-editor-expand");
    this.worldCodeTabBtn = this.root.querySelector("#btn-editor-world-code");
    this.lessonPlanTabBtn = this.root.querySelector("#btn-editor-lesson-plan");
    this.lessonPlanAuthoringEl = this.root.querySelector("[data-lesson-plan-authoring]");
    this.projectWorkbenchPanel = this.root.querySelector("[data-project-workbench-panel]");
    this.projectCurrentFileEl = this.root.querySelector("[data-project-current-file]");
    this.projectEntryFileEl = this.root.querySelector("[data-project-entry-file]");
    this.projectNavigatorEl = this.root.querySelector("[data-project-file-navigator]");
    this.projectDependencyStatusEl = this.root.querySelector("[data-project-dependency-status]");
    this.projectRunLifecycleEl = this.root.querySelector("[data-project-run-lifecycle]");
    this.projectRunStatusEl = this.root.querySelector("[data-project-run-status]");
    this.projectActiveRunIdEl = this.root.querySelector("[data-project-active-run-id]");
    this.projectRunArchiveCountEl = this.root.querySelector("[data-project-run-archive-count]");
    this.projectRunHistoryEl = this.root.querySelector("[data-project-run-history]");
    this.projectObservationTraceEl = this.root.querySelector("[data-project-observation-trace]");
    this.projectObservationTraceHashEl = this.root.querySelector("[data-project-observation-trace-hash]");
    this.projectObservationRowsEl = this.root.querySelector("[data-project-observation-rows]");
    this.projectRunResultEl = this.root.querySelector("[data-project-run-result]");
    this.projectEnableBtn = this.root.querySelector("#btn-project-enable");
    this.projectCreateBtn = this.root.querySelector("#btn-project-file-create");
    this.projectRenameBtn = this.root.querySelector("#btn-project-file-rename");
    this.projectDeleteBtn = this.root.querySelector("#btn-project-file-delete");
    this.projectRunBtn = this.root.querySelector("#btn-project-run-summary");

    this.root.querySelector("#btn-back-to-browse")?.addEventListener("click", () => {
      this.onBack();
    });

    this.root.querySelector("#btn-run-from-editor")?.addEventListener("click", () => {
      this.onRun(this.getDdn(), { readinessModel: this.getStudioReadinessModel() });
    });

    this.root.querySelector("#btn-save-ddn")?.addEventListener("click", () => {
      if (this.editorMode === "lesson-plan") {
        this.syncLessonPlanFromEditorFields();
        this.onSaveLessonPlan(this.getLessonPlanText(), { model: this.getLessonPlanModel() });
        return;
      }
      this.onSave(this.getDdn());
    });
    this.root.querySelector("#btn-load-ddn")?.addEventListener("click", () => {
      void this.handleLoadFromLocalFile();
    });

    this.root.querySelector("#btn-advanced-editor")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });
    this.expandBtn?.addEventListener("click", () => {
      this.setExpanded(!this.expanded);
    });
    this.worldCodeTabBtn?.addEventListener("click", () => {
      this.setEditorMode("world-code");
    });
    this.lessonPlanTabBtn?.addEventListener("click", () => {
      this.setEditorMode("lesson-plan");
    });

    this.textarea?.addEventListener("input", () => {
      this.syncCurrentProjectFileFromTextarea();
      this.emitSourceChange();
    });

    this.projectEnableBtn?.addEventListener("click", () => {
      this.enableProjectMode();
    });
    this.projectCreateBtn?.addEventListener("click", () => {
      this.handleCreateProjectFile();
    });
    this.projectRenameBtn?.addEventListener("click", () => {
      this.handleRenameProjectFile();
    });
    this.projectDeleteBtn?.addEventListener("click", () => {
      this.handleDeleteProjectFile();
    });
    this.projectRunBtn?.addEventListener("click", () => {
      void this.handleProjectRunSummary();
    });

    this.focusPrevBtn?.addEventListener("click", () => {
      this.moveFocus(-1);
    });

    this.focusNextBtn?.addEventListener("click", () => {
      this.moveFocus(1);
    });

    this.focusSelectEl?.addEventListener("change", (event) => {
      const nextValue = Number(event?.target?.value);
      if (Number.isFinite(nextValue)) {
        this.selectFocusIndex(nextValue);
      }
    });

    this.focusWarningEl?.addEventListener("click", () => {
      this.toggleGuideHeaderSelection();
    });
    this.readinessActionBtn?.addEventListener("click", () => {
      void this.handleReadinessAction();
    });
    this.fixitApplyBtn?.addEventListener("click", () => {
      void this.handleFixitApply();
    });
    this.setStudioReadinessModel(null);
    this.setFixitModel(null);
    this.renderLessonPlanAuthoring();
    this.renderProjectWorkbench();
    this.setEditorMode("world-code");
    this.setExpanded(false);
  }

  setExpanded(expanded) {
    this.expanded = Boolean(expanded);
    this.root?.classList?.toggle?.("editor-expanded", this.expanded);
    if (this.root?.dataset) this.root.dataset.editorExpanded = this.expanded ? "1" : "0";
    if (this.expandBtn) {
      this.expandBtn.setAttribute("aria-pressed", this.expanded ? "true" : "false");
      this.expandBtn.title = this.expanded ? "편집 영역 기본 크기" : "편집 영역 확장";
      this.expandBtn.setAttribute("aria-label", this.expanded ? "편집 영역 기본 크기" : "편집 영역 확장");
    }
  }

  setEditorMode(mode) {
    const next = String(mode ?? "").trim() === "lesson-plan" ? "lesson-plan" : "world-code";
    this.editorMode = next;
    if (this.root?.dataset) this.root.dataset.editorMode = next;
    const isLessonPlan = next === "lesson-plan";
    this.worldCodeTabBtn?.classList?.toggle?.("active", !isLessonPlan);
    this.lessonPlanTabBtn?.classList?.toggle?.("active", isLessonPlan);
    this.worldCodeTabBtn?.setAttribute?.("aria-selected", isLessonPlan ? "false" : "true");
    this.lessonPlanTabBtn?.setAttribute?.("aria-selected", isLessonPlan ? "true" : "false");
    this.textarea?.classList?.toggle?.("hidden", isLessonPlan);
    this.readinessCardEl?.classList?.toggle?.("hidden", isLessonPlan);
    this.fixitCardEl?.classList?.toggle?.("hidden", isLessonPlan || this.fixitCardEl?.dataset?.ready === "false");
    this.focusBarEl?.classList?.toggle?.("hidden", isLessonPlan || this.focusMatches.length <= 0);
    this.lessonPlanAuthoringEl?.classList?.toggle?.("hidden", !isLessonPlan);
    this.projectWorkbenchPanel?.classList?.toggle?.("hidden", isLessonPlan);
  }

  async handleLoadFromLocalFile() {
    const file = await pickDdnFileFromLocal(this.loadFileInputEl);
    if (!file) return false;
    return this.applyLoadedFile(file);
  }

  async applyLoadedFile(file) {
    try {
      const text = await readTextFromLocalFile(file);
      this.replaceDdn(text, { emitSourceChange: true });
      const fileName = String(file?.name ?? "파일").trim() || "파일";
      this.setSmokeResult(`불러오기 완료: ${fileName}`);
      return true;
    } catch (error) {
      this.setSmokeResult(`불러오기 실패: ${String(error?.message ?? error)}`);
      return false;
    }
  }

  setLessonChoices(lessonChoices = []) {
    this.lessonChoices = Array.isArray(lessonChoices)
      ? lessonChoices.map((row) => ({
        lesson_id: String(row?.lesson_id ?? row?.id ?? "").trim(),
        title: String(row?.title ?? row?.label ?? row?.lesson_id ?? row?.id ?? "").trim(),
        grade: String(row?.grade ?? "").trim(),
        subject: String(row?.subject ?? "").trim(),
      })).filter((row) => row.lesson_id)
      : [];
  }

  resetLocalProjectWorkbench() {
    this.projectMode = false;
    this.projectFiles.clear();
    this.currentProjectPath = LOCAL_PROJECT_ENTRY_PATH;
    this.projectEntryPath = LOCAL_PROJECT_ENTRY_PATH;
    this.projectRunSummary = null;
    this.projectLastGraphHash = "";
    this.projectRunSeq = 0;
    this.projectRuntimeStatus = "idle";
    this.projectActiveRunId = "";
    this.projectRunHistory = [];
    if (this.textarea) this.textarea.readOnly = this.readOnly;
    this.renderProjectWorkbench();
  }

  getCurrentTextareaText() {
    return String(this.textarea?.value ?? "");
  }

  isProjectReadonlyPath(path) {
    const normalized = String(path ?? "").trim();
    return normalized === LOCAL_PROJECT_ROOT_PATH
      || normalized === LOCAL_PROJECT_LOCK_PATH
      || normalized.startsWith("vendor/");
  }

  normalizeProjectPath(path) {
    return String(path ?? "")
      .replace(/\\/g, "/")
      .replace(/^\/+/, "")
      .replace(/\/+/g, "/")
      .trim();
  }

  setProjectFile(path, role, text, { readOnly = false } = {}) {
    const normalizedPath = this.normalizeProjectPath(path);
    if (!normalizedPath) return;
    this.projectFiles.set(normalizedPath, {
      path: normalizedPath,
      role: String(role ?? "ddn_source").trim() || "ddn_source",
      utf8_text: String(text ?? ""),
      readOnly: Boolean(readOnly) || this.isProjectReadonlyPath(normalizedPath),
    });
  }

  buildProjectRootText() {
    return `프로젝트 {
  이름: "셈그림 로컬 프로젝트".
  뿌리: "./".
  첫글: "./${LOCAL_PROJECT_ENTRY_PATH}".

  목표 {
    앱: "./${LOCAL_PROJECT_ENTRY_PATH}".
  }

  모듈길 {
    로컬: "./lib".
  }
}.
`;
  }

  seedLocalProjectFilesFromCurrent() {
    const entryText = this.getCurrentTextareaText().trim()
      ? this.getCurrentTextareaText()
      : `쓰임 {
  도움: "./lib/helper".
}
도움.값 보여주기.
`;
    this.projectFiles.clear();
    this.setProjectFile(LOCAL_PROJECT_ROOT_PATH, "root", this.buildProjectRootText(), { readOnly: true });
    this.setProjectFile(LOCAL_PROJECT_ENTRY_PATH, "ddn_source", entryText);
    this.setProjectFile("lib/helper.ddn", "ddn_source", `드러냄 {
값.
}
값 <- 11.
`);
    this.setProjectFile(LOCAL_PROJECT_LOCK_PATH, "lock", `{
  "schema": "ddn.lock.v0",
  "pins": []
}
`, { readOnly: true });
    this.setProjectFile("vendor/edu.physics/gaji.toml", "gaji_manifest", `id = "edu.physics"
version = "0.1.0"
`, { readOnly: true });
    this.currentProjectPath = LOCAL_PROJECT_ENTRY_PATH;
    this.projectEntryPath = LOCAL_PROJECT_ENTRY_PATH;
  }

  enableProjectMode() {
    if (!this.projectMode) {
      this.seedLocalProjectFilesFromCurrent();
      this.projectMode = true;
      this.projectRunSummary = null;
      this.projectLastGraphHash = "";
      this.projectRunSeq = 0;
      this.projectRuntimeStatus = "idle";
      this.projectActiveRunId = "";
      this.projectRunHistory = [];
      this.projectLoadedPayload = null;
      this.openProjectFile(LOCAL_PROJECT_ENTRY_PATH, { emitSourceChange: false });
      this.setSmokeResult("로컬 프로젝트 모드");
    }
    this.renderProjectWorkbench();
  }

  syncCurrentProjectFileFromTextarea() {
    if (!this.projectMode) return;
    const file = this.projectFiles.get(this.currentProjectPath);
    if (!file || file.readOnly) return;
    file.utf8_text = this.getCurrentTextareaText();
    this.projectFiles.set(this.currentProjectPath, file);
  }

  openProjectFile(path, { emitSourceChange = true } = {}) {
    if (!this.projectMode) return;
    this.syncCurrentProjectFileFromTextarea();
    const normalizedPath = this.normalizeProjectPath(path);
    const file = this.projectFiles.get(normalizedPath);
    if (!file || !this.textarea) return;
    this.currentProjectPath = normalizedPath;
    this.textarea.value = String(file.utf8_text ?? "");
    this.textarea.readOnly = this.readOnly || Boolean(file.readOnly);
    this.loadFocusMatches(this.textarea.value, { readOnly: this.textarea.readOnly });
    this.renderProjectWorkbench();
    if (emitSourceChange) this.emitSourceChange();
  }

  projectFileRows() {
    return [...this.projectFiles.values()].sort((a, b) => String(a.path).localeCompare(String(b.path), "ko"));
  }

  renderProjectWorkbench() {
    if (!this.projectWorkbenchPanel) return;
    const activeFile = this.projectFiles.get(this.currentProjectPath) ?? null;
    const readOnly = Boolean(activeFile?.readOnly) || this.isProjectReadonlyPath(this.currentProjectPath);
    if (this.projectWorkbenchPanel.dataset) {
      this.projectWorkbenchPanel.dataset.projectMode = this.projectMode ? "1" : "0";
      this.projectWorkbenchPanel.dataset.currentPath = this.projectMode ? this.currentProjectPath : "";
      this.projectWorkbenchPanel.dataset.currentReadonly = readOnly ? "1" : "0";
      this.projectWorkbenchPanel.dataset.entryPath = this.projectEntryPath;
    }
    if (this.projectCurrentFileEl) {
      this.projectCurrentFileEl.textContent = this.projectMode
        ? `${this.currentProjectPath}${readOnly ? " · 읽기 전용" : ""}`
        : "한 파일";
    }
    if (this.projectEntryFileEl) {
      this.projectEntryFileEl.textContent = `entry: ${this.projectEntryPath}`;
    }
    if (this.projectEnableBtn) {
      this.projectEnableBtn.textContent = this.projectMode ? "프로젝트 모드 켜짐" : "프로젝트 모드";
      this.projectEnableBtn.setAttribute("aria-pressed", this.projectMode ? "true" : "false");
    }
    if (this.projectCreateBtn) this.projectCreateBtn.disabled = !this.projectMode;
    const protectedCurrent = !this.projectMode || readOnly || this.currentProjectPath === this.projectEntryPath;
    if (this.projectRenameBtn) this.projectRenameBtn.disabled = protectedCurrent;
    if (this.projectDeleteBtn) this.projectDeleteBtn.disabled = protectedCurrent;
    if (this.projectRunBtn) this.projectRunBtn.disabled = !this.projectMode || !this.onProjectRunSummary;

    if (this.projectNavigatorEl) {
      this.projectNavigatorEl.innerHTML = "";
      if (!this.projectMode) {
        const hint = document.createElement("span");
        hint.className = "project-file-empty";
        hint.textContent = "현재 원문을 프로젝트로 바꾸려면 프로젝트 모드를 누르세요.";
        this.projectNavigatorEl.appendChild(hint);
      } else {
        this.projectFileRows().forEach((file) => {
          const button = document.createElement("button");
          button.type = "button";
          button.className = "project-file-row";
          button.dataset.projectFilePath = file.path;
          button.dataset.projectRole = file.role;
          button.dataset.selected = file.path === this.currentProjectPath ? "1" : "0";
          button.dataset.readonly = file.readOnly ? "1" : "0";
          button.setAttribute("data-testid", `project-file-${file.path.replace(/[^A-Za-z0-9가-힣_-]+/g, "-")}`);
          button.innerHTML = `
            <span>${escapeText(file.path)}</span>
            <small>${escapeText(file.role)}${file.readOnly ? " · lock/vendor" : ""}${file.path === this.projectEntryPath ? " · entry" : ""}</small>
          `;
          button.addEventListener("click", () => this.openProjectFile(file.path));
          this.projectNavigatorEl.appendChild(button);
        });
      }
    }

    if (this.projectDependencyStatusEl) {
      if (!this.projectMode) {
        this.projectDependencyStatusEl.textContent = "단일 파일 편집";
      } else {
        const graphHash = String(this.projectRunSummary?.project_graph_sha256 ?? "").trim();
        const rows = Array.isArray(this.projectRunSummary?.output_rows) ? this.projectRunSummary.output_rows.length : 0;
        this.projectDependencyStatusEl.textContent = graphHash
          ? `graph ${graphHash.slice(0, 12)} · 출력 ${rows}줄`
          : `${this.projectFiles.size}개 파일 · 실행 전`;
      }
    }
    if (this.projectRunLifecycleEl?.dataset) {
      this.projectRunLifecycleEl.dataset.runtimeStatus = this.projectRuntimeStatus;
      this.projectRunLifecycleEl.dataset.activeRunId = this.projectActiveRunId;
      this.projectRunLifecycleEl.dataset.archiveCount = String(this.projectRunHistory.length);
    }
    if (this.projectRunStatusEl) {
      this.projectRunStatusEl.textContent = `status ${this.projectRuntimeStatus}`;
    }
    if (this.projectActiveRunIdEl) {
      this.projectActiveRunIdEl.textContent = this.projectActiveRunId ? `active ${this.projectActiveRunId}` : "run 없음";
    }
    if (this.projectRunArchiveCountEl) {
      this.projectRunArchiveCountEl.textContent = `archive ${this.projectRunHistory.length}`;
    }
    if (this.projectRunHistoryEl) {
      this.projectRunHistoryEl.innerHTML = "";
      this.projectRunHistoryEl.className = "project-run-history";
      if (!this.projectRunHistory.length) {
        const empty = document.createElement("li");
        empty.className = "project-run-history-empty";
        empty.textContent = "실행 이력 없음";
        this.projectRunHistoryEl.appendChild(empty);
      } else {
        this.projectRunHistory.forEach((run) => {
          const item = document.createElement("li");
          item.className = "project-run-history-row";
          item.dataset.projectRunId = run.run_id;
          item.dataset.projectRunStatus = run.status;
          item.dataset.sourceCandidateHash = run.source_candidate_hash;
          item.dataset.summarySha256 = run.summary_sha256;
          item.textContent = `${run.run_id} · ${run.status} · 출력 ${run.output_rows.length}줄 · ${run.summary_sha256.slice(0, 12) || "no-summary"}`;
          this.projectRunHistoryEl.appendChild(item);
        });
      }
    }
    const observationTrace = this.projectRunSummary?.project_observation_trace;
    const observationRows = Array.isArray(observationTrace?.rows) ? observationTrace.rows : [];
    const observationTraceHash = String(observationTrace?.trace_sha256 ?? "").trim();
    if (this.projectObservationTraceEl?.dataset) {
      this.projectObservationTraceEl.dataset.rowCount = String(observationRows.length);
      this.projectObservationTraceEl.dataset.traceSha256 = observationTraceHash;
    }
    if (this.projectObservationTraceHashEl) {
      this.projectObservationTraceHashEl.textContent = observationTraceHash ? `trace ${observationTraceHash.slice(0, 12)}` : "trace 없음";
    }
    if (this.projectObservationRowsEl) {
      this.projectObservationRowsEl.innerHTML = "";
      this.projectObservationRowsEl.className = "project-observation-rows";
      if (!observationRows.length) {
        const empty = document.createElement("li");
        empty.className = "project-observation-empty";
        empty.textContent = "관찰값 없음";
        this.projectObservationRowsEl.appendChild(empty);
      } else {
        observationRows.forEach((row) => {
          const item = document.createElement("li");
          item.className = "project-observation-row";
          item.dataset.projectObservationId = String(row?.observation_id ?? "");
          item.dataset.projectObservationIndex = String(row?.index ?? "");
          item.dataset.projectObservationName = String(row?.name ?? "");
          item.dataset.projectObservationValue = String(row?.value_text ?? "");
          item.dataset.stateHashParticipation = row?.state_hash_participation ? "1" : "0";
          item.textContent = `${row?.name ?? "관찰"} = ${row?.value_text ?? ""}`;
          this.projectObservationRowsEl.appendChild(item);
        });
      }
    }
    if (this.projectRunResultEl) {
      this.projectRunResultEl.textContent = this.projectRunSummary
        ? JSON.stringify(this.projectRunSummary, null, 2)
        : "프로젝트 실행 대기";
    }
  }

  nextAvailableProjectPath(basePath) {
    const normalized = this.normalizeProjectPath(basePath);
    if (!this.projectFiles.has(normalized)) return normalized;
    const match = normalized.match(/^(.*?)(\.[^./]+)?$/);
    const prefix = match?.[1] ?? normalized;
    const suffix = match?.[2] ?? "";
    for (let index = 2; index < 100; index += 1) {
      const candidate = `${prefix}_${index}${suffix}`;
      if (!this.projectFiles.has(candidate)) return candidate;
    }
    return `${prefix}_${Date.now()}${suffix}`;
  }

  handleCreateProjectFile() {
    this.enableProjectMode();
    this.syncCurrentProjectFileFromTextarea();
    const base = this.projectFiles.has("lib/helper.ddn") ? "lib/helper_extra.ddn" : "lib/helper.ddn";
    const path = this.nextAvailableProjectPath(base);
    this.setProjectFile(path, "ddn_source", `드러냄 {
값.
}
값 <- ${path.includes("extra") ? "13" : "11"}.
`);
    this.openProjectFile(path);
    this.setSmokeResult(`프로젝트 파일 추가: ${path}`);
  }

  handleRenameProjectFile() {
    if (!this.projectMode) return;
    this.syncCurrentProjectFileFromTextarea();
    const current = this.projectFiles.get(this.currentProjectPath);
    if (!current || current.readOnly || current.path === this.projectEntryPath) return;
    const base = current.path === "lib/helper.ddn"
      ? "lib/helper_renamed.ddn"
      : current.path.replace(/(\.[^./]+)?$/, "_renamed$1");
    const nextPath = this.nextAvailableProjectPath(base);
    this.projectFiles.delete(current.path);
    this.projectFiles.set(nextPath, { ...current, path: nextPath });
    const oldImport = `./${current.path.replace(/\.ddn$/u, "")}`;
    const nextImport = `./${nextPath.replace(/\.ddn$/u, "")}`;
    this.projectFiles.forEach((file, filePath) => {
      if (file.role !== "ddn_source" || file.readOnly) return;
      file.utf8_text = String(file.utf8_text ?? "").split(oldImport).join(nextImport);
      this.projectFiles.set(filePath, file);
    });
    this.currentProjectPath = nextPath;
    this.openProjectFile(nextPath);
    this.setSmokeResult(`프로젝트 파일 이름 변경: ${nextPath}`);
  }

  handleDeleteProjectFile() {
    if (!this.projectMode) return;
    const current = this.projectFiles.get(this.currentProjectPath);
    if (!current || current.readOnly || current.path === this.projectEntryPath) return;
    this.projectFiles.delete(current.path);
    this.currentProjectPath = this.projectEntryPath;
    this.openProjectFile(this.projectEntryPath);
    this.setSmokeResult(`프로젝트 파일 삭제: ${current.path}`);
  }

  buildVirtualProjectSource() {
    this.syncCurrentProjectFileFromTextarea();
    return {
      schema: "ddn.virtual_project_source.v1",
      kind: "seamgrim.local_project_tree.v0",
      root_path: LOCAL_PROJECT_ROOT_PATH,
      entry_path: this.projectEntryPath,
      files: this.projectFileRows().map((file) => ({
        path: file.path,
        role: file.role,
        utf8_text: String(file.utf8_text ?? ""),
      })),
    };
  }

  getLoadedProjectPayload() {
    if (!this.projectLoadedPayload || typeof this.projectLoadedPayload !== "object") return null;
    try {
      return JSON.parse(JSON.stringify(this.projectLoadedPayload));
    } catch (_) {
      return null;
    }
  }

  nextProjectRunId() {
    this.projectRunSeq += 1;
    return `project_run_${String(this.projectRunSeq).padStart(3, "0")}`;
  }

  buildProjectSourceCandidateHash(source) {
    return `candidate:fnv1a32:project_source:${fnv1a32(stableCanonicalJson(source ?? {}))}`;
  }

  pushProjectRunHistory(record) {
    const row = record && typeof record === "object" ? record : {};
    const runId = String(row.run_id ?? "").trim();
    if (!runId) return;
    const next = {
      schema: "seamgrim.project_run_lifecycle_record.v1",
      run_id: runId,
      status: String(row.status ?? "unknown"),
      source_candidate_hash: String(row.source_candidate_hash ?? ""),
      project_graph_sha256: String(row.project_graph_sha256 ?? ""),
      bounded_state_sha256: String(row.bounded_state_sha256 ?? ""),
      summary_sha256: String(row.summary_sha256 ?? ""),
      output_rows: Array.isArray(row.output_rows) ? row.output_rows.map((value) => String(value ?? "")) : [],
      diagnostics: Array.isArray(row.diagnostics) ? row.diagnostics : [],
    };
    this.projectRunHistory = [next, ...this.projectRunHistory.filter((item) => item.run_id !== runId)].slice(0, 8);
  }

  isProjectModeActive() {
    return Boolean(this.projectMode);
  }

  loadVirtualProjectSource(source = {}, { title = "로컬 프로젝트", readOnly = false, lessonPlanText = "", lessonPlan = null, lessonId = "local_project", lessonChoices = null } = {}) {
    const row = source && typeof source === "object" ? source : {};
    const files = Array.isArray(row.files) ? row.files : [];
    if (!files.length) {
      this.loadLesson("", { title, readOnly, lessonPlanText, lessonPlan, lessonId, lessonChoices });
      return false;
    }
    this.readOnly = Boolean(readOnly);
    if (Array.isArray(lessonChoices)) {
      this.setLessonChoices(lessonChoices);
    }
    this.projectMode = true;
    try {
      this.projectLoadedPayload = JSON.parse(JSON.stringify(row));
    } catch (_) {
      this.projectLoadedPayload = null;
    }
    this.projectFiles.clear();
    this.projectEntryPath = this.normalizeProjectPath(row.entry_path ?? LOCAL_PROJECT_ENTRY_PATH) || LOCAL_PROJECT_ENTRY_PATH;
    files.forEach((file) => {
      const filePath = this.normalizeProjectPath(file?.path ?? "");
      if (!filePath) return;
      this.setProjectFile(
        filePath,
        String(file?.role ?? "ddn_source").trim() || "ddn_source",
        String(file?.utf8_text ?? file?.text ?? ""),
        { readOnly: Boolean(file?.readOnly ?? file?.readonly) || this.isProjectReadonlyPath(filePath) },
      );
    });
    if (!this.projectFiles.has(this.projectEntryPath)) {
      this.setProjectFile(this.projectEntryPath, "ddn_source", String(row.ddn_text ?? ""));
    }
    this.currentProjectPath = this.projectEntryPath;
    const entry = this.projectFiles.get(this.projectEntryPath);
    if (this.textarea) {
      this.textarea.value = String(entry?.utf8_text ?? "");
      this.textarea.readOnly = this.readOnly || Boolean(entry?.readOnly);
    }
    if (this.titleEl) {
      this.titleEl.textContent = this.readOnly ? `${title} (읽기 전용)` : title;
    }
    this.projectRunSummary = null;
    this.projectLastGraphHash = String(row.project_graph_sha256 ?? "");
    this.projectRunSeq = 0;
    this.projectRuntimeStatus = "idle";
    this.projectActiveRunId = "";
    this.projectRunHistory = [];
    this.setSmokeResult(this.readOnly ? "로컬 프로젝트 읽기 전용 모드" : "로컬 프로젝트 편집 모드");
    this.loadFocusMatches(String(entry?.utf8_text ?? ""), { readOnly: this.readOnly || Boolean(entry?.readOnly) });
    this.setStudioReadinessModel(null);
    this.setFixitModel(null);
    this.loadLessonPlan(lessonPlanText, {
      lessonPlan,
      lessonId,
      title,
    });
    this.renderProjectWorkbench();
    this.emitSourceChange();
    return true;
  }

  async handleProjectRunSummary() {
    this.enableProjectMode();
    if (!this.onProjectRunSummary) {
      this.setSmokeResult("프로젝트 실행 API가 연결되지 않았습니다.");
      return;
    }
    const runId = this.nextProjectRunId();
    const source = this.buildVirtualProjectSource();
    const sourceCandidateHash = this.buildProjectSourceCandidateHash(source);
    this.projectRuntimeStatus = "running";
    this.projectActiveRunId = runId;
    this.pushProjectRunHistory({
      run_id: runId,
      status: "running",
      source_candidate_hash: sourceCandidateHash,
    });
    this.renderProjectWorkbench();
    try {
      const summary = await this.onProjectRunSummary(source, { target: "앱" });
      this.projectRunSummary = summary && typeof summary === "object" ? summary : null;
      this.projectLastGraphHash = String(this.projectRunSummary?.project_graph_sha256 ?? "");
      this.projectRuntimeStatus = "completed";
      this.pushProjectRunHistory({
        run_id: runId,
        status: "completed",
        source_candidate_hash: sourceCandidateHash,
        project_graph_sha256: this.projectLastGraphHash,
        bounded_state_sha256: String(this.projectRunSummary?.bounded_state_sha256 ?? ""),
        summary_sha256: String(this.projectRunSummary?.summary_sha256 ?? ""),
        output_rows: Array.isArray(this.projectRunSummary?.output_rows) ? this.projectRunSummary.output_rows : [],
        diagnostics: Array.isArray(this.projectRunSummary?.diagnostics) ? this.projectRunSummary.diagnostics : [],
      });
      this.renderProjectWorkbench();
      const outputRows = Array.isArray(this.projectRunSummary?.output_rows)
        ? this.projectRunSummary.output_rows.join(", ")
        : "";
      this.setSmokeResult(outputRows ? `프로젝트 실행 완료: ${outputRows}` : "프로젝트 실행 완료");
    } catch (error) {
      this.projectRuntimeStatus = "failed";
      this.projectRunSummary = {
        schema: "ddn.local_project_run_summary.ui_error.v1",
        run_id: runId,
        diagnostics: [{
          code: "E_PROJECT_RUN_UI_FAILED",
          message: String(error?.message ?? error),
        }],
      };
      this.pushProjectRunHistory({
        run_id: runId,
        status: "failed",
        source_candidate_hash: sourceCandidateHash,
        diagnostics: this.projectRunSummary.diagnostics,
      });
      this.renderProjectWorkbench();
      this.setSmokeResult(`프로젝트 실행 실패: ${String(error?.message ?? error)}`);
    }
  }

  loadLesson(ddnText, { title = "DDN 보기", readOnly = true, focusText = "", focusTexts = [], lessonPlanText = "", lessonPlan = null, lessonId = "", lessonChoices = null } = {}) {
    this.readOnly = Boolean(readOnly);
    this.resetLocalProjectWorkbench();
    if (Array.isArray(lessonChoices)) {
      this.setLessonChoices(lessonChoices);
    }
    const normalizedText = String(ddnText ?? "");
    if (this.textarea) {
      this.textarea.value = normalizedText;
      this.textarea.readOnly = this.readOnly;
    }
    if (this.titleEl) {
      this.titleEl.textContent = this.readOnly ? `${title} (읽기 전용)` : title;
    }
    this.setSmokeResult(this.readOnly ? "교과 DDN 읽기 전용 모드" : "편집 모드");
    this.loadFocusMatches(normalizedText, { readOnly: this.readOnly, focusText, focusTexts });
    this.setStudioReadinessModel(null);
    this.setFixitModel(null);
    this.loadLessonPlan(lessonPlanText, {
      lessonPlan,
      lessonId,
      title,
    });
    this.emitSourceChange();
  }

  loadBlank() {
    this.readOnly = false;
    this.resetLocalProjectWorkbench();
    if (this.textarea) {
      this.textarea.value = BLANK_DDN_TEMPLATE;
      this.textarea.readOnly = false;
    }
    if (this.titleEl) {
      this.titleEl.textContent = "DDN 편집";
    }
    this.setSmokeResult("새 DDN 편집 모드");
    this.loadFocusMatches(BLANK_DDN_TEMPLATE, { readOnly: false });
    this.setStudioReadinessModel(null);
    this.setFixitModel(null);
    this.loadLessonPlan("", { lessonId: "custom", title: "새 수업 구성" });
    this.emitSourceChange();
  }

  loadLessonPlan(text = "", { lessonPlan = null, lessonId = "", title = "", lessonChoices = null } = {}) {
    if (Array.isArray(lessonChoices)) {
      this.setLessonChoices(lessonChoices);
    }
    const sourceText = String(text ?? "").trim()
      ? String(text ?? "")
      : buildBlankLessonPlanToml({
        lessonId: String(lessonId ?? "").trim() || "custom",
        title: String(title ?? "").trim() || "새 수업 구성",
      });
    const parsed = lessonPlan && typeof lessonPlan === "object"
      ? lessonPlan
      : parseLessonPlanToml(sourceText, {
        fallbackLessonId: String(lessonId ?? "").trim(),
        fallbackTitle: String(title ?? "").trim(),
      });
    this.lessonPlanModel = parsed;
    this.lessonPlanText = serializeLessonPlanToml(parsed);
    const activityCount = Array.isArray(parsed.activities) ? parsed.activities.length : 0;
    this.lessonPlanSelectedIndex = Math.min(Math.max(0, this.lessonPlanSelectedIndex), Math.max(0, activityCount - 1));
    this.renderLessonPlanAuthoring();
  }

  getLessonPlanModel() {
    return this.lessonPlanModel && typeof this.lessonPlanModel === "object"
      ? JSON.parse(JSON.stringify(this.lessonPlanModel))
      : parseLessonPlanToml(this.lessonPlanText);
  }

  getLessonPlanText() {
    this.syncLessonPlanFromEditorFields();
    return serializeLessonPlanToml(this.lessonPlanModel);
  }

  syncLessonPlanFromEditorFields() {
    if (!this.lessonPlanCodeEl) return;
    const codeText = String(this.lessonPlanCodeEl.value ?? "");
    const parsed = parseLessonPlanToml(codeText, {
      fallbackLessonId: this.lessonPlanModel?.lesson_id ?? "",
      fallbackTitle: this.lessonPlanModel?.title ?? "",
    });
    this.lessonPlanModel = parsed;
    this.lessonPlanText = serializeLessonPlanToml(parsed);
  }

  updateLessonPlanRoot(patch = {}) {
    const model = this.getLessonPlanModel();
    this.lessonPlanModel = parseLessonPlanToml(serializeLessonPlanToml({ ...model, ...patch }), {
      fallbackLessonId: model.lesson_id,
      fallbackTitle: model.title,
    });
    this.lessonPlanText = serializeLessonPlanToml(this.lessonPlanModel);
    this.renderLessonPlanAuthoring();
  }

  updateLessonPlanActivity(index, patch = {}) {
    const model = this.getLessonPlanModel();
    const rows = Array.isArray(model.activities) ? model.activities : [];
    const targetIndex = Math.max(0, Math.min(rows.length - 1, Number(index) || 0));
    rows[targetIndex] = {
      ...(rows[targetIndex] ?? {}),
      ...patch,
    };
    this.lessonPlanModel = parseLessonPlanToml(serializeLessonPlanToml({ ...model, activities: rows }), {
      fallbackLessonId: model.lesson_id,
      fallbackTitle: model.title,
    });
    this.lessonPlanSelectedIndex = targetIndex;
    this.lessonPlanText = serializeLessonPlanToml(this.lessonPlanModel);
    this.renderLessonPlanAuthoring();
  }

  mutateLessonPlanActivities(kind) {
    const model = this.getLessonPlanModel();
    const rows = Array.isArray(model.activities) ? [...model.activities] : [];
    const index = Math.max(0, Math.min(rows.length - 1, Number(this.lessonPlanSelectedIndex) || 0));
    if (kind === "add") {
      rows.splice(index + 1, 0, {
        activity_id: `activity-${rows.length + 1}`,
        kind: "observe",
        title: `활동 ${rows.length + 1}`,
        prompt: "새 활동 안내를 입력하세요.",
        allowed_controls: [],
        anchor_refs: [],
      });
      this.lessonPlanSelectedIndex = index + 1;
    } else if (kind === "delete" && rows.length > 1) {
      rows.splice(index, 1);
      this.lessonPlanSelectedIndex = Math.max(0, index - 1);
    } else if (kind === "up" && index > 0) {
      [rows[index - 1], rows[index]] = [rows[index], rows[index - 1]];
      this.lessonPlanSelectedIndex = index - 1;
    } else if (kind === "down" && index < rows.length - 1) {
      [rows[index + 1], rows[index]] = [rows[index], rows[index + 1]];
      this.lessonPlanSelectedIndex = index + 1;
    }
    this.lessonPlanModel = parseLessonPlanToml(serializeLessonPlanToml({ ...model, activities: rows }), {
      fallbackLessonId: model.lesson_id,
      fallbackTitle: model.title,
    });
    this.lessonPlanText = serializeLessonPlanToml(this.lessonPlanModel);
    this.renderLessonPlanAuthoring();
  }

  applyLessonPlanCodeEditor() {
    const parsed = parseLessonPlanToml(String(this.lessonPlanCodeEl?.value ?? ""), {
      fallbackLessonId: this.lessonPlanModel?.lesson_id ?? "",
      fallbackTitle: this.lessonPlanModel?.title ?? "",
    });
    this.lessonPlanModel = parsed;
    this.lessonPlanText = serializeLessonPlanToml(parsed);
    this.lessonPlanSelectedIndex = Math.min(this.lessonPlanSelectedIndex, Math.max(0, parsed.activities.length - 1));
    this.renderLessonPlanAuthoring();
    this.setSmokeResult(parsed.diagnostics.length ? `수업 구성 진단 ${parsed.diagnostics.length}건` : "수업 구성 코드 반영 완료");
  }

  renderLessonPlanAuthoring() {
    if (!this.lessonPlanAuthoringEl) return;
    const model = this.lessonPlanModel && typeof this.lessonPlanModel === "object"
      ? this.lessonPlanModel
      : parseLessonPlanToml(this.lessonPlanText);
    const activities = Array.isArray(model.activities) ? model.activities : [];
    const selectedIndex = Math.max(0, Math.min(activities.length - 1, Number(this.lessonPlanSelectedIndex) || 0));
    this.lessonPlanSelectedIndex = selectedIndex;
    const selected = activities[selectedIndex] ?? {};
    const diagText = Array.isArray(model.diagnostics) && model.diagnostics.length
      ? `진단 ${model.diagnostics.length}건: ${model.diagnostics.map((row) => row.code).join(", ")}`
      : "진단 없음";
    const selectedLessonId = String(model.lesson_id ?? "").trim();
    const lessonChoices = this.lessonChoices.length
      ? this.lessonChoices
      : [{ lesson_id: selectedLessonId || "custom", title: model.title || selectedLessonId || "custom", grade: "", subject: "" }];
    const lessonChoiceOptions = lessonChoices.map((row) => {
      const id = String(row.lesson_id ?? "").trim();
      const labelParts = [
        String(row.title ?? id).trim() || id,
        String(row.grade ?? "").trim(),
        String(row.subject ?? "").trim(),
      ].filter(Boolean);
      return `<option value="${escapeAttribute(id)}"${id === selectedLessonId ? " selected" : ""}>${escapeText(labelParts.join(" · "))}</option>`;
    }).join("");
    const rootArrayText = (value) => (Array.isArray(value) ? value : []).join(", ");
    this.lessonPlanAuthoringEl.innerHTML = `
      <div class="lesson-plan-editor" data-lesson-plan-editor>
        <div class="lesson-plan-editor-head">
          <div>
            <strong>${model.title || "수업 구성"}</strong>
            <span data-lesson-plan-status>${activities.length}개 활동 · ${diagText}</span>
          </div>
          <div class="lesson-plan-editor-actions">
            <button type="button" data-lesson-plan-action="add">활동 추가</button>
            <button type="button" data-lesson-plan-action="delete">삭제</button>
            <button type="button" data-lesson-plan-action="up">위로</button>
            <button type="button" data-lesson-plan-action="down">아래로</button>
            <button type="button" data-lesson-plan-action="apply-code">코드 반영</button>
          </div>
        </div>
        <section class="lesson-plan-classroom-fields" data-lesson-plan-classroom-fields data-testid="lesson-plan-classroom-fields" aria-label="교사 수업 구성">
          <label>12세계
            <select data-lesson-plan-world-select data-testid="foundation-lesson-select">${lessonChoiceOptions}</select>
          </label>
          <label>수업 제목
            <input data-lesson-plan-root-field="title" data-testid="lesson-plan-title-field" type="text" value="${escapeAttribute(model.title)}" />
          </label>
          <label>수업 질문
            <textarea data-lesson-plan-root-field="lesson_question" data-testid="lesson-plan-question-field" rows="2">${escapeText(model.lesson_question)}</textarea>
          </label>
          <label>예상 질문
            <textarea data-lesson-plan-root-field="prediction_question" data-testid="lesson-plan-prediction-field" rows="2">${escapeText(model.prediction_question)}</textarea>
          </label>
          <label>설명 질문
            <textarea data-lesson-plan-root-field="explanation_question" data-testid="lesson-plan-explanation-field" rows="2">${escapeText(model.explanation_question)}</textarea>
          </label>
          <label>오개념
            <input data-lesson-plan-root-field="misconception_refs" data-testid="lesson-plan-misconception-field" type="text" value="${escapeAttribute(rootArrayText(model.misconception_refs))}" />
          </label>
          <label>읽기 확인
            <input data-lesson-plan-root-field="reading_check" data-testid="lesson-plan-reading-check-field" type="text" value="${escapeAttribute(model.reading_check)}" />
          </label>
          <label>루브릭
            <textarea data-lesson-plan-root-field="rubric" data-testid="lesson-plan-rubric-field" rows="2">${escapeText(model.rubric)}</textarea>
          </label>
          <label>허용 조절
            <input data-lesson-plan-root-field="allowed_controls" data-testid="lesson-plan-root-allowed-controls" type="text" value="${escapeAttribute(rootArrayText(model.allowed_controls))}" />
          </label>
          <label>조절 범위
            <input data-lesson-plan-root-field="control_ranges" data-testid="lesson-plan-control-ranges" type="text" value="${escapeAttribute(rootArrayText(model.control_ranges))}" />
          </label>
          <label>기본 보기
            <select data-lesson-plan-root-field="primary_view" data-testid="lesson-plan-root-primary-view">
              ${["space2d", "graph", "table", "text"].map((view) => `<option value="${view}"${model.primary_view === view ? " selected" : ""}>${view}</option>`).join("")}
            </select>
          </label>
          <label>보조 보기
            <input data-lesson-plan-root-field="secondary_views" data-testid="lesson-plan-root-secondary-views" type="text" value="${escapeAttribute(rootArrayText(model.secondary_views))}" />
          </label>
          <label>배포 잠금
            <select data-lesson-plan-root-field="deployment_profile_lock" data-testid="lesson-plan-deployment-lock">
              ${["locked", "review", "unlocked"].map((value) => `<option value="${value}"${model.deployment_profile_lock === value ? " selected" : ""}>${value}</option>`).join("")}
            </select>
          </label>
        </section>
        <div class="lesson-plan-editor-grid">
          <div class="lesson-plan-activity-list" data-lesson-plan-activity-list role="list"></div>
          <div class="lesson-plan-activity-editor" data-lesson-plan-activity-editor>
            <label>종류<select data-lesson-plan-field="kind">${LESSON_PLAN_ACTIVITY_KINDS.map((kind) => `<option value="${kind}"${selected.kind === kind ? " selected" : ""}>${getActivityKindLabel(kind)}</option>`).join("")}</select></label>
            <label>제목<input data-lesson-plan-field="title" type="text" value="${escapeAttribute(selected.title)}" /></label>
            <label>안내<textarea data-lesson-plan-field="prompt" rows="3">${escapeText(selected.prompt)}</textarea></label>
            <label>scene_ref<input data-lesson-plan-field="scene_ref" data-testid="lesson-plan-scene-ref" type="text" value="${escapeAttribute(selected.scene_ref)}" /></label>
            <label>primary_view<input data-lesson-plan-field="primary_view" data-testid="lesson-plan-primary-view" type="text" value="${escapeAttribute(selected.primary_view)}" /></label>
            <label>camera_policy<input data-lesson-plan-field="camera_policy" data-testid="lesson-plan-camera-policy-field" type="text" value="${escapeAttribute(selected.camera_policy)}" /></label>
            <label>측정 부품<input data-lesson-plan-field="measurement_components" data-testid="lesson-plan-measurements-field" type="text" value="${escapeAttribute((selected.measurement_components ?? []).join(", "))}" /></label>
            <label>반응 규칙<input data-lesson-plan-field="reaction_rule_refs" data-testid="lesson-plan-reaction-rules-field" type="text" value="${escapeAttribute((selected.reaction_rule_refs ?? []).join(", "))}" /></label>
            <label>Anchor<input data-lesson-plan-field="anchor_refs" type="text" value="${escapeAttribute((selected.anchor_refs ?? []).join(", "))}" /></label>
            <label>허용 조절<input data-lesson-plan-field="allowed_controls" type="text" value="${escapeAttribute((selected.allowed_controls ?? []).join(", "))}" /></label>
            <label>질문<input data-lesson-plan-field="question" type="text" value="${escapeAttribute(selected.question)}" /></label>
            <label>preset_ref<input data-lesson-plan-field="preset_ref" type="text" value="${escapeAttribute(selected.preset_ref)}" /></label>
            <label>view_authoring_hash<input data-lesson-plan-field="view_authoring_hash" data-testid="lesson-plan-view-authoring-hash" type="text" value="${escapeAttribute(selected.view_authoring_hash)}" /></label>
            <label>check_ref<input data-lesson-plan-field="check_ref" type="text" value="${escapeAttribute(selected.check_ref)}" /></label>
          </div>
        </div>
        ${this.renderViewReactionAuthoringPanel()}
        <textarea class="lesson-plan-code-editor" data-lesson-plan-code spellcheck="false">${escapeText(serializeLessonPlanToml(model))}</textarea>
      </div>
    `;
    this.lessonPlanCodeEl = this.lessonPlanAuthoringEl.querySelector("[data-lesson-plan-code]");
    this.bindViewReactionAuthoringControls();
    const listEl = this.lessonPlanAuthoringEl.querySelector("[data-lesson-plan-activity-list]");
    activities.forEach((activity, index) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "lesson-plan-activity-row";
      button.dataset.lessonPlanActivityIndex = String(index);
      button.dataset.selected = index === selectedIndex ? "1" : "0";
      button.textContent = `${index + 1}. ${getActivityKindLabel(activity.kind)} · ${activity.title || activity.activity_id}`;
      button.addEventListener("click", () => {
        this.lessonPlanSelectedIndex = index;
        this.renderLessonPlanAuthoring();
      });
      listEl?.appendChild(button);
    });
    this.lessonPlanAuthoringEl.querySelectorAll("[data-lesson-plan-action]").forEach((button) => {
      button.addEventListener("click", () => {
        const action = String(button.dataset.lessonPlanAction ?? "");
        if (action === "apply-code") {
          this.applyLessonPlanCodeEditor();
        } else {
          this.mutateLessonPlanActivities(action);
        }
      });
    });
    this.lessonPlanAuthoringEl.querySelector("[data-lesson-plan-world-select]")?.addEventListener("change", (event) => {
      const nextLessonId = String(event?.target?.value ?? "").trim();
      if (!nextLessonId || nextLessonId === String(this.lessonPlanModel?.lesson_id ?? "").trim()) return;
      this.onSelectLesson(nextLessonId, { source: "lesson_plan_authoring" });
    });
    this.lessonPlanAuthoringEl.querySelectorAll("[data-lesson-plan-root-field]").forEach((field) => {
      field.addEventListener("change", () => {
        const key = String(field.dataset.lessonPlanRootField ?? "");
        const value = String(field.value ?? "");
        const patch = ["allowed_controls", "control_ranges", "misconception_refs", "secondary_views"].includes(key)
          ? { [key]: value.split(",").map((item) => item.trim()).filter(Boolean) }
          : { [key]: value };
        this.updateLessonPlanRoot(patch);
      });
    });
    this.lessonPlanAuthoringEl.querySelectorAll("[data-lesson-plan-field]").forEach((field) => {
      field.addEventListener("change", () => {
        const key = String(field.dataset.lessonPlanField ?? "");
        const value = String(field.value ?? "");
        const patch = ["anchor_refs", "allowed_controls", "measurement_components", "reaction_rule_refs"].includes(key)
          ? { [key]: value.split(",").map((item) => item.trim()).filter(Boolean) }
          : { [key]: value };
        this.updateLessonPlanActivity(this.lessonPlanSelectedIndex, patch);
      });
    });
  }

  renderViewReactionAuthoringPanel() {
    const model = normalizeViewReactionAuthoringModel(this.viewReactionAuthoringModel);
    const artifact = buildViewReactionAuthoringArtifact(model);
    this.viewReactionAuthoringArtifact = artifact;
    const optionList = (values, selected) => values
      .map((value) => `<option value="${escapeAttribute(value)}"${value === selected ? " selected" : ""}>${escapeText(value)}</option>`)
      .join("");
    const measurementChecks = VIEW_AUTHORING_MEASURE_COMPONENTS.map((component) => `
      <label class="view-author-measure-row">
        <input type="checkbox" data-view-author-measure="${escapeAttribute(component)}"${model.measurement_components.includes(component) ? " checked" : ""} />
        <span>${escapeText(component)}</span>
      </label>
    `).join("");
    const ruleRows = artifact.reaction_rules.map((rule) => `
      <li data-view-author-rule-row="${escapeAttribute(rule.rule_id)}">
        <span>${escapeText(rule.condition_kind)}</span>
        <span>${escapeText(rule.action_kind)}</span>
        <span>${escapeText(rule.semantic_anchor)}</span>
        <span>우선순위 ${escapeText(rule.priority)}</span>
      </li>
    `).join("");
    return `
      <section class="view-reaction-authoring-panel" data-view-reaction-authoring-panel data-testid="view-reaction-authoring-panel" aria-label="보기와 반응 연출 저작">
        <div class="view-reaction-authoring-head">
          <strong>보기·반응 연출</strong>
          <span data-view-author-status>${artifact.diagnostics.length ? `진단 ${artifact.diagnostics.length}건` : "저작 가능"}</span>
        </div>
        <div class="view-reaction-authoring-grid">
          <label>카메라
            <select data-view-author-camera-policy data-testid="view-author-camera-policy">
              ${optionList(VIEW_AUTHORING_CAMERA_POLICIES, model.camera_policy)}
            </select>
          </label>
          <fieldset>
            <legend>측정 부품</legend>
            ${measurementChecks}
          </fieldset>
          <label>의미 닻
            <select data-view-author-anchor data-testid="view-author-anchor">
              ${optionList(VIEW_AUTHORING_ANCHORS, model.semantic_anchor)}
            </select>
          </label>
          <label>조건
            <select data-view-author-condition-kind data-testid="view-author-condition-kind">
              ${optionList(VIEW_AUTHORING_CONDITION_KINDS, model.condition_kind)}
            </select>
          </label>
          <label>반응 동작
            <select data-view-author-action-kind data-testid="view-author-action-kind">
              ${optionList(VIEW_AUTHORING_ACTION_KINDS, model.action_kind)}
            </select>
          </label>
          <label>우선순위
            <input data-view-author-priority data-testid="view-author-priority" type="number" min="0" step="1" value="${escapeAttribute(model.priority)}" />
          </label>
        </div>
        <div class="view-reaction-authoring-actions">
          <button type="button" data-view-author-add-rule data-testid="view-author-add-rule">규칙 추가</button>
          <button type="button" data-view-author-conflict-demo data-testid="view-author-conflict-demo">충돌 검사</button>
        </div>
        <ul class="view-reaction-rule-list" data-view-author-rule-list>${ruleRows}</ul>
        <div class="view-reaction-diagnostics" data-view-author-diagnostics data-testid="view-author-diagnostics">
          ${artifact.diagnostics.length ? escapeText(artifact.diagnostics.map((diag) => diag.code).join(", ")) : "진단 없음"}
        </div>
        <textarea class="view-reaction-artifact" data-view-author-artifact data-testid="view-author-artifact" readonly>${escapeText(JSON.stringify(artifact, null, 2))}</textarea>
      </section>
    `;
  }

  bindViewReactionAuthoringControls() {
    const panel = this.lessonPlanAuthoringEl?.querySelector("[data-view-reaction-authoring-panel]");
    if (!panel) return;
    this.viewReactionAuthoringPanelEl = panel;
    this.viewReactionAuthoringArtifactEl = panel.querySelector("[data-view-author-artifact]");
    this.viewReactionAuthoringDiagnosticsEl = panel.querySelector("[data-view-author-diagnostics]");
    const sync = () => this.updateViewReactionAuthoringFromControls();
    panel.querySelector("[data-view-author-camera-policy]")?.addEventListener("change", sync);
    panel.querySelector("[data-view-author-anchor]")?.addEventListener("change", sync);
    panel.querySelector("[data-view-author-condition-kind]")?.addEventListener("change", sync);
    panel.querySelector("[data-view-author-action-kind]")?.addEventListener("change", sync);
    panel.querySelector("[data-view-author-priority]")?.addEventListener("input", sync);
    panel.querySelectorAll("[data-view-author-measure]").forEach((input) => {
      input.addEventListener("change", sync);
    });
    panel.querySelector("[data-view-author-add-rule]")?.addEventListener("click", () => {
      this.updateViewReactionAuthoringFromControls();
      const model = normalizeViewReactionAuthoringModel(this.viewReactionAuthoringModel);
      model.rules = [
        ...model.rules,
        normalizeViewReactionRule({
          rule_id: `author.rule.${model.rules.length + 1}`,
          condition_kind: model.condition_kind,
          action_kind: model.action_kind,
          semantic_anchor: model.semantic_anchor,
          priority: model.priority,
        }, model.rules.length),
      ];
      this.viewReactionAuthoringModel = normalizeViewReactionAuthoringModel(model);
      this.renderLessonPlanAuthoring();
    });
    panel.querySelector("[data-view-author-conflict-demo]")?.addEventListener("click", () => {
      const model = normalizeViewReactionAuthoringModel(this.viewReactionAuthoringModel);
      model.rules = [
        normalizeViewReactionRule({
          rule_id: "author.conflict.a",
          condition_kind: "zero_crossing",
          action_kind: "highlight",
          semantic_anchor: "obs.bob",
          priority: 10,
        }, 0),
        normalizeViewReactionRule({
          rule_id: "author.conflict.b",
          condition_kind: "extremum",
          action_kind: "highlight",
          semantic_anchor: "obs.bob",
          priority: 10,
        }, 1),
      ];
      this.viewReactionAuthoringModel = normalizeViewReactionAuthoringModel(model);
      this.renderLessonPlanAuthoring();
    });
    this.syncViewReactionAuthoringArtifactPanel();
  }

  updateViewReactionAuthoringFromControls() {
    const panel = this.viewReactionAuthoringPanelEl ?? this.lessonPlanAuthoringEl?.querySelector("[data-view-reaction-authoring-panel]");
    if (!panel) return;
    const selectedMeasurements = Array.from(panel.querySelectorAll("[data-view-author-measure]"))
      .filter((input) => input.checked)
      .map((input) => String(input.dataset.viewAuthorMeasure ?? "").trim())
      .filter(Boolean);
    const current = normalizeViewReactionAuthoringModel(this.viewReactionAuthoringModel);
    this.viewReactionAuthoringModel = normalizeViewReactionAuthoringModel({
      ...current,
      camera_policy: panel.querySelector("[data-view-author-camera-policy]")?.value ?? current.camera_policy,
      measurement_components: selectedMeasurements,
      semantic_anchor: panel.querySelector("[data-view-author-anchor]")?.value ?? current.semantic_anchor,
      condition_kind: panel.querySelector("[data-view-author-condition-kind]")?.value ?? current.condition_kind,
      action_kind: panel.querySelector("[data-view-author-action-kind]")?.value ?? current.action_kind,
      priority: panel.querySelector("[data-view-author-priority]")?.value ?? current.priority,
    });
    this.syncViewReactionAuthoringArtifactPanel();
  }

  syncViewReactionAuthoringArtifactPanel() {
    const panel = this.viewReactionAuthoringPanelEl ?? this.lessonPlanAuthoringEl?.querySelector("[data-view-reaction-authoring-panel]");
    if (!panel) return;
    const artifact = buildViewReactionAuthoringArtifact(this.viewReactionAuthoringModel);
    this.viewReactionAuthoringArtifact = artifact;
    panel.dataset.viewAuthoringReady = artifact.diagnostics.length ? "0" : "1";
    panel.dataset.viewAuthoringSchema = artifact.schema;
    panel.dataset.viewAuthoringCameraPolicy = artifact.camera.selected_policy;
    panel.dataset.viewAuthoringMeasurements = artifact.measurements.selected_components.join(",");
    panel.dataset.viewAuthoringConditionKinds = artifact.supported_conditions.join(",");
    panel.dataset.viewAuthoringActionKinds = artifact.supported_actions.join(",");
    panel.dataset.viewAuthoringRuleCount = String(artifact.reaction_rules.length);
    panel.dataset.viewAuthoringDiagnostics = artifact.diagnostics.map((diag) => diag.code).join(",");
    panel.dataset.viewAuthoringImplicitLwwAllowed = artifact.implicit_lww_allowed ? "1" : "0";
    panel.dataset.viewAuthoringStateHashParticipation = artifact.state_hash_participation ? "1" : "0";
    panel.dataset.viewAuthoringArtifactHash = artifact.artifact_hash;
    const status = panel.querySelector("[data-view-author-status]");
    if (status) status.textContent = artifact.diagnostics.length ? `진단 ${artifact.diagnostics.length}건` : "저작 가능";
    const diagnostics = this.viewReactionAuthoringDiagnosticsEl ?? panel.querySelector("[data-view-author-diagnostics]");
    if (diagnostics) {
      diagnostics.textContent = artifact.diagnostics.length
        ? artifact.diagnostics.map((diag) => diag.code).join(", ")
        : "진단 없음";
    }
    const artifactEl = this.viewReactionAuthoringArtifactEl ?? panel.querySelector("[data-view-author-artifact]");
    if (artifactEl) artifactEl.value = JSON.stringify(artifact, null, 2);
    if (typeof window !== "undefined") {
      window.__SEAMGRIM_VIEW_REACTION_AUTHORING__ = artifact;
    }
    this.applyViewReactionAuthoringToLessonPlanActivity(artifact);
  }

  applyViewReactionAuthoringToLessonPlanActivity(artifact) {
    if (!artifact || typeof artifact !== "object") return;
    const model = this.getLessonPlanModel();
    const rows = Array.isArray(model.activities) && model.activities.length ? [...model.activities] : [];
    if (!rows.length) return;
    const index = Math.max(0, Math.min(rows.length - 1, Number(this.lessonPlanSelectedIndex) || 0));
    const current = rows[index] ?? {};
    rows[index] = {
      ...current,
      scene_ref: String(current.scene_ref ?? "").trim() || "scene.main",
      primary_view: String(current.primary_view ?? "").trim() || "space2d",
      camera_policy: String(artifact.camera?.selected_policy ?? current.camera_policy ?? "").trim(),
      measurement_components: Array.isArray(artifact.measurements?.selected_components)
        ? artifact.measurements.selected_components.map((item) => String(item ?? "").trim()).filter(Boolean)
        : [],
      reaction_rule_refs: Array.isArray(artifact.reaction_rules)
        ? artifact.reaction_rules.map((rule) => String(rule?.rule_id ?? "").trim()).filter(Boolean)
        : [],
      view_authoring_hash: String(artifact.artifact_hash ?? "").trim(),
      view_plan_ref: String(current.view_plan_ref ?? "").trim() || "view_plan.v1",
    };
    this.lessonPlanModel = parseLessonPlanToml(serializeLessonPlanToml({ ...model, activities: rows }), {
      fallbackLessonId: model.lesson_id,
      fallbackTitle: model.title,
    });
    this.lessonPlanText = serializeLessonPlanToml(this.lessonPlanModel);
    if (this.lessonPlanCodeEl) {
      this.lessonPlanCodeEl.value = this.lessonPlanText;
    }
    const fieldMap = {
      scene_ref: rows[index].scene_ref,
      primary_view: rows[index].primary_view,
      camera_policy: rows[index].camera_policy,
      measurement_components: rows[index].measurement_components.join(", "),
      reaction_rule_refs: rows[index].reaction_rule_refs.join(", "),
      view_authoring_hash: rows[index].view_authoring_hash,
    };
    Object.entries(fieldMap).forEach(([key, value]) => {
      const field = this.lessonPlanAuthoringEl?.querySelector(`[data-lesson-plan-field="${key}"]`);
      if (field && field.value !== value) field.value = value;
    });
  }

  getDdn() {
    if (this.projectMode) {
      this.syncCurrentProjectFileFromTextarea();
      return String(this.projectFiles.get(this.projectEntryPath)?.utf8_text ?? "");
    }
    return String(this.textarea?.value ?? "");
  }

  replaceDdn(nextText, { emitSourceChange = true } = {}) {
    if (!this.textarea) return;
    this.textarea.value = String(nextText ?? "");
    if (this.projectMode) {
      const file = this.projectFiles.get(this.currentProjectPath);
      if (file && !file.readOnly) {
        file.utf8_text = this.textarea.value;
        this.projectFiles.set(this.currentProjectPath, file);
      }
    }
    if (emitSourceChange) {
      this.emitSourceChange();
    }
  }

  setSmokeResult(message) {
    if (!this.resultEl) return;
    this.resultEl.textContent = String(message ?? "");
  }

  setCanonSummary(message) {
    if (!this.canonSummaryEl) return;
    this.canonSummaryEl.textContent = String(message ?? "구성: -");
  }

  setCanonFlatView(viewModel) {
    const topoOrder = Array.isArray(viewModel?.topoOrder) ? viewModel.topoOrder : [];
    const instances = Array.isArray(viewModel?.instances) ? viewModel.instances : [];
    const links = Array.isArray(viewModel?.links) ? viewModel.links : [];

    this.setCanonSummary(String(viewModel?.summaryText ?? "구성: -"));

    if (this.canonTopoEl) {
      this.canonTopoEl.innerHTML = "";
      if (topoOrder.length > 0) {
        topoOrder.forEach((name, index) => {
          if (index > 0) {
            const arrow = document.createElement("span");
            arrow.className = "editor-canon-sep";
            arrow.textContent = " -> ";
            this.canonTopoEl.appendChild(arrow);
          }
          const button = document.createElement("button");
          button.type = "button";
          button.className = "editor-canon-chip";
          button.textContent = String(name ?? "");
          button.addEventListener("click", () => {
            this.selectFlatInstanceByName(name);
          });
          this.canonTopoEl.appendChild(button);
        });
      } else {
        this.canonTopoEl.textContent = "-";
      }
    }

    if (this.canonInstancesEl) {
      this.canonInstancesEl.innerHTML = "";
      if (instances.length > 0) {
        instances.forEach((row) => {
          const li = document.createElement("li");
          const button = document.createElement("button");
          button.type = "button";
          button.className = "editor-canon-item";
          button.textContent = String(row?.label ?? row?.name ?? "-");
          button.addEventListener("click", () => {
            this.selectFlatInstanceByName(row?.name ?? "");
          });
          li.appendChild(button);
          this.canonInstancesEl.appendChild(li);
        });
      } else {
        const li = document.createElement("li");
        li.textContent = "-";
        this.canonInstancesEl.appendChild(li);
      }
    }

    if (this.canonLinksEl) {
      this.canonLinksEl.innerHTML = "";
      if (links.length > 0) {
        links.forEach((row) => {
          const li = document.createElement("li");
          const button = document.createElement("button");
          button.type = "button";
          button.className = "editor-canon-item";
          button.textContent = String(row?.label ?? "-");
          button.addEventListener("click", () => {
            this.selectFlatLink(row);
          });
          li.appendChild(button);
          this.canonLinksEl.appendChild(li);
        });
      } else {
        const li = document.createElement("li");
        li.textContent = "-";
        this.canonLinksEl.appendChild(li);
      }
    }
  }

  selectFlatInstanceByName(instanceName) {
    const range = findFlatInstanceSelectionRange(this.getDdn(), instanceName);
    if (!range) return;
    this.selectRange(range.start, range.end);
  }

  selectFlatLink(link) {
    const range = findFlatLinkSelectionRange(this.getDdn(), link);
    if (!range) return;
    this.selectRange(range.start, range.end);
  }

  emitSourceChange() {
    try {
      this.onSourceChange(this.getDdn());
    } catch (_) {
      // ignore editor source change errors
    }
  }

  getStudioReadinessModel() {
    return normalizeReadinessModel(this.readinessModel);
  }

  setStudioReadinessModel(rawModel = null) {
    const model = normalizeReadinessModel(rawModel);
    this.readinessModel = model;
    if (this.readinessCardEl?.dataset) {
      this.readinessCardEl.dataset.stage = model.stage;
    }
    if (this.readinessStageEl) {
      if (model.stage === STUDIO_READINESS_STAGE_AUTOFIX) {
        this.readinessStageEl.textContent = "자동 수정 가능";
      } else if (model.stage === STUDIO_READINESS_STAGE_BLOCKED) {
        this.readinessStageEl.textContent = "실행 차단";
      } else {
        this.readinessStageEl.textContent = "입력 준비됨";
      }
    }
    if (this.readinessCauseEl) {
      this.readinessCauseEl.textContent = model.user_cause;
    }
    if (this.readinessDiagnosticEl) {
      const diagnostic = buildReadinessDiagnosticLayers(model);
      this.readinessDiagnosticEl.dataset.ready = model.stage === STUDIO_READINESS_STAGE_READY ? "true" : "false";
      this.readinessDiagnosticEl.dataset.stage = model.stage;
      this.readinessDiagnosticEl.querySelector("[data-diagnostic-layer='what']")?.replaceChildren(document.createTextNode(diagnostic.what));
      this.readinessDiagnosticEl.querySelector("[data-diagnostic-layer='where']")?.replaceChildren(document.createTextNode(diagnostic.where));
      this.readinessDiagnosticEl.querySelector("[data-diagnostic-layer='how']")?.replaceChildren(document.createTextNode(diagnostic.how));
    }
    if (this.readinessActionBtn) {
      this.readinessActionBtn.textContent = String(model.primary_action?.label ?? "바로 실행");
      this.readinessActionBtn.disabled = this.readOnly;
      this.readinessActionBtn.title = String(model.primary_action?.detail ?? "");
    }
    if (this.runGateReasonEl) {
      if (model.stage === STUDIO_READINESS_STAGE_BLOCKED) {
        this.runGateReasonEl.textContent = `실행 차단: ${model.user_cause}`;
      } else if (model.stage === STUDIO_READINESS_STAGE_AUTOFIX) {
        this.runGateReasonEl.textContent = "자동 수정 가능";
      } else {
        this.runGateReasonEl.textContent = "입력 준비됨";
      }
    }
  }

  async handleReadinessAction() {
    const model = this.getStudioReadinessModel();
    const actionKind = String(model?.primary_action?.kind ?? "").trim().toLowerCase();
    if (actionKind === "autofix" && typeof this.onAutofix === "function") {
      await this.onAutofix(this.getDdn(), {
        source: "editor_readiness_card",
      });
      return;
    }
    if (actionKind === "manual_fix_example") {
      const detail = String(model.manual_example || model.primary_action?.detail || "").trim();
      this.setSmokeResult(detail || "수정 예시를 참고해 입력을 바꿔 주세요.");
      return;
    }
    this.onRun(this.getDdn(), { readinessModel: model });
  }

  setFixitModel(rawPreview = null) {
    const preview = rawPreview && typeof rawPreview === "object" ? rawPreview : null;
    const source = this.getDdn();
    const previewText = String(preview?.preview_text ?? source);
    const fixitCount = Number(preview?.fixit_count ?? 0);
    const ready = Boolean(preview && fixitCount > 0 && previewText !== source && !this.readOnly);
    this.fixitModel = ready ? preview : null;
    if (this.fixitCardEl) {
      this.fixitCardEl.classList.toggle("hidden", !ready);
      if (this.fixitCardEl.dataset) {
        this.fixitCardEl.dataset.ready = ready ? "true" : "false";
      }
    }
    if (this.fixitSummaryEl) {
      const diagnostics = Number(preview?.diagnostic_count ?? 0);
      this.fixitSummaryEl.textContent = ready
        ? `일괄 적용 가능: 진단 ${diagnostics}건, 수정 ${fixitCount}건`
        : "수정 후보 없음";
    }
    if (this.fixitApplyBtn) {
      this.fixitApplyBtn.disabled = !ready;
    }
  }

  async handleFixitApply() {
    const preview = this.fixitModel;
    if (!preview || typeof this.onApplyFixit !== "function") return;
    await this.onApplyFixit(preview, {
      sourceText: this.getDdn(),
    });
  }

  loadFocusMatches(value, { readOnly = true, focusText = "", focusTexts = [] } = {}) {
    this.focusMatches = [];
    this.focusIndex = -1;
    this.warningShowsGuide = false;
    const normalizedValue = String(value ?? "");
    this.guideHeaderRange = !readOnly ? findGuideHeaderRange(normalizedValue) : null;
    if (!readOnly) {
      const rows = Array.isArray(focusTexts) && focusTexts.length > 0 ? focusTexts : [focusText];
      const seen = new Set();
      rows.forEach((row) => {
        const needle = String(row ?? "");
        if (!needle || seen.has(needle)) return;
        seen.add(needle);
        const start = normalizedValue.indexOf(needle);
        if (start < 0) return;
        this.focusMatches.push({
          text: needle,
          start,
          end: start + needle.length,
          label: extractFocusMatchLabel(needle),
          valueType: extractFocusMatchType(needle),
          assign: extractFocusMatchAssign(needle),
          range: extractFocusMatchRange(needle),
          step: extractFocusMatchStep(needle),
        });
      });
      this.focusMatches.sort((a, b) => a.start - b.start);
    }
    if (this.focusMatches.length > 0) {
      this.selectFocusIndex(0);
      return;
    }
    this.updateFocusBar();
  }

  moveFocus(offset) {
    if (!Array.isArray(this.focusMatches) || this.focusMatches.length === 0) return;
    const delta = Number(offset) || 0;
    const nextIndex =
      (((this.focusIndex >= 0 ? this.focusIndex : 0) + delta) % this.focusMatches.length + this.focusMatches.length) %
      this.focusMatches.length;
    this.selectFocusIndex(nextIndex);
  }

  selectGuideHeader() {
    if (!this.guideHeaderRange || this.readOnly) return;
    this.warningShowsGuide = true;
    this.selectRange(this.guideHeaderRange.start, this.guideHeaderRange.end);
    this.updateFocusBar();
  }

  selectCurrentFocusMatch() {
    if (!Array.isArray(this.focusMatches) || this.focusMatches.length === 0 || this.readOnly) return;
    const match = this.focusMatches[this.focusIndex >= 0 ? this.focusIndex : 0];
    if (!match) return;
    this.warningShowsGuide = false;
    this.selectRange(match.start, match.end);
    this.updateFocusBar();
  }

  toggleGuideHeaderSelection() {
    if (this.warningShowsGuide) {
      this.selectCurrentFocusMatch();
      return;
    }
    this.selectGuideHeader();
  }

  selectFocusIndex(index) {
    if (!Array.isArray(this.focusMatches) || this.focusMatches.length === 0) return;
    this.focusIndex = Math.max(0, Math.min(this.focusMatches.length - 1, Number(index) || 0));
    this.warningShowsGuide = false;
    const match = this.focusMatches[this.focusIndex];
    this.selectRange(match.start, match.end);
    this.updateFocusBar();
  }

  updateFocusBar() {
    const hasMatches = Array.isArray(this.focusMatches) && this.focusMatches.length > 0 && !this.readOnly;
    this.focusBarEl?.classList?.toggle("hidden", !hasMatches);
    if (!hasMatches) return;
    if (this.focusSelectEl) {
      this.focusSelectEl.innerHTML = "";
      this.focusMatches.forEach((match, index) => {
        const option = document.createElement("option");
        option.value = String(index);
        const head = match?.label
          ? match?.valueType
            ? `${match.label}:${match.valueType}`
            : match.label
          : `후보 ${index + 1}`;
        option.textContent = head;
        option.selected = index === this.focusIndex;
        this.focusSelectEl.appendChild(option);
      });
      this.focusSelectEl.disabled = this.focusMatches.length <= 1;
    }
    if (this.focusSummaryEl) {
      const currentMatch = this.focusMatches[this.focusIndex] ?? null;
      const head = currentMatch?.label
        ? currentMatch?.valueType
          ? `${currentMatch.label}:${currentMatch.valueType}`
          : currentMatch.label
        : "";
      const headWithAssign = head && currentMatch?.assign ? `${head} ${currentMatch.assign}` : head;
      const suffixParts = [headWithAssign, currentMatch?.range, currentMatch?.step].filter(Boolean);
      const suffix = suffixParts.length > 0 ? ` · ${suffixParts.join(" · ")}` : "";
      this.focusSummaryEl.textContent = `매김 전환 후보 ${this.focusIndex + 1}/${this.focusMatches.length}${suffix}`;
    }
    if (this.focusWarningEl) {
      this.focusWarningEl.textContent = this.warningShowsGuide
        ? "후보로 돌아가기 · 원래 줄 삭제/교체"
        : "안내 보기 · 원래 줄 삭제/교체";
      this.focusWarningEl.title = this.warningShowsGuide
        ? "클릭해 현재 후보 줄로 돌아가기"
        : "클릭해 상단 전환 안내 선택";
      this.focusWarningEl.disabled = !this.guideHeaderRange;
    }
    if (this.focusPrevBtn) {
      this.focusPrevBtn.disabled = this.focusMatches.length <= 1;
    }
    if (this.focusNextBtn) {
      this.focusNextBtn.disabled = this.focusMatches.length <= 1;
    }
  }

  selectRange(start, end) {
    if (!this.textarea) return;
    try {
      if (typeof this.textarea.focus === "function") {
        this.textarea.focus();
      }
      if (typeof this.textarea.setSelectionRange === "function") {
        this.textarea.setSelectionRange(start, end);
      } else {
        this.textarea.selectionStart = start;
        this.textarea.selectionEnd = end;
      }
    } catch (_) {
      // ignore selection errors
    }
  }
}

export function saveDdnToFile(text, filename = "lesson.ddn") {
  const blob = new Blob([String(text ?? "")], { type: "text/plain;charset=utf-8" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = filename;
  link.click();
  setTimeout(() => URL.revokeObjectURL(link.href), 800);
}

export async function readTextFromLocalFile(file) {
  if (!file || typeof file !== "object") {
    throw new Error("선택된 파일이 없습니다.");
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  if (bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
    throw new Error("E_SOURCE_UTF8_BOM_FORBIDDEN: 원문은 BOM 없는 UTF-8이어야 합니다");
  }
  return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
}

export function pickDdnFileFromLocal(inputEl = null) {
  const input = inputEl && typeof inputEl === "object"
    ? inputEl
    : document.createElement("input");
  input.type = "file";
  input.accept = ".ddn,.txt,.md,text/plain,text/markdown";
  input.classList?.add?.("hidden");
  if (!input.parentNode) {
    document.body.appendChild(input);
  }
  input.value = "";
  return new Promise((resolve) => {
    let settled = false;
    const finish = (file) => {
      if (settled) return;
      settled = true;
      input.removeEventListener("change", handleChange);
      window.removeEventListener("focus", handleWindowFocus);
      resolve(file ?? null);
    };
    const handleChange = () => {
      finish(input.files?.[0] ?? null);
    };
    const handleWindowFocus = () => {
      setTimeout(() => {
        finish(input.files?.[0] ?? null);
      }, 240);
    };
    input.addEventListener("change", handleChange);
    window.addEventListener("focus", handleWindowFocus, { once: true });
    input.click();
  });
}
import {
  buildBlankLessonPlanToml,
  getActivityKindLabel,
  LESSON_PLAN_ACTIVITY_KINDS,
  parseLessonPlanToml,
  serializeLessonPlanToml,
} from "../lesson_plan_v0.js";
