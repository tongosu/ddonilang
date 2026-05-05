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
      label: buttonLabel || (stage === STUDIO_READINESS_STAGE_AUTOFIX ? "자동 수정 적용" : "작업실에서 실행"),
      detail: buttonDetail,
    },
    autofix_available: Boolean(model.autofix_available),
    blocking_remaining: Boolean(model.blocking_remaining),
    manual_example: String(model.manual_example ?? "").trim(),
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

export class EditorScreen {
  constructor({ root, onBack, onRun, onSave, onOpenAdvanced, onSourceChange, onAutofix } = {}) {
    this.root = root;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onRun = typeof onRun === "function" ? onRun : () => {};
    this.onSave = typeof onSave === "function" ? onSave : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};
    this.onSourceChange = typeof onSourceChange === "function" ? onSourceChange : () => {};
    this.onAutofix = typeof onAutofix === "function" ? onAutofix : null;
    this.readOnly = false;
    this.focusMatches = [];
    this.focusIndex = -1;
    this.guideHeaderRange = null;
    this.warningShowsGuide = false;
    this.readinessModel = normalizeReadinessModel(null);
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
    this.readinessActionBtn = this.root.querySelector("#btn-editor-readiness-action");
    this.loadFileInputEl = this.root.querySelector("#input-editor-ddn-file");

    this.root.querySelector("#btn-back-to-browse")?.addEventListener("click", () => {
      this.onBack();
    });

    this.root.querySelector("#btn-run-from-editor")?.addEventListener("click", () => {
      this.onRun(this.getDdn(), { readinessModel: this.getStudioReadinessModel() });
    });

    this.root.querySelector("#btn-save-ddn")?.addEventListener("click", () => {
      this.onSave(this.getDdn());
    });
    this.root.querySelector("#btn-load-ddn")?.addEventListener("click", () => {
      void this.handleLoadFromLocalFile();
    });

    this.root.querySelector("#btn-advanced-editor")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });

    this.textarea?.addEventListener("input", () => {
      this.emitSourceChange();
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
    this.setStudioReadinessModel(null);
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

  loadLesson(ddnText, { title = "DDN 보기", readOnly = true, focusText = "", focusTexts = [] } = {}) {
    this.readOnly = Boolean(readOnly);
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
    this.emitSourceChange();
  }

  loadBlank() {
    this.readOnly = false;
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
    this.emitSourceChange();
  }

  getDdn() {
    return String(this.textarea?.value ?? "");
  }

  replaceDdn(nextText, { emitSourceChange = true } = {}) {
    if (!this.textarea) return;
    this.textarea.value = String(nextText ?? "");
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
  const text = await file.text();
  return String(text ?? "");
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
