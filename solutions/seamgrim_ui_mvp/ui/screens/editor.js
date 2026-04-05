const BLANK_DDN_TEMPLATE = `#이름: 새 교과
#설명: 채비를 조절하면서 결과를 확인하세요.

채비: {
  계수 <- 1.
}.

(매마디)마다 {
  t <- 프레임수.
  y <- (계수 * t).
  t 보여주기.
  y 보여주기.
  프레임수 <- (프레임수 + 1).
}.`;

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
  constructor({ root, onBack, onRun, onSave, onOpenAdvanced, onSourceChange, onOpenBlock } = {}) {
    this.root = root;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onRun = typeof onRun === "function" ? onRun : () => {};
    this.onSave = typeof onSave === "function" ? onSave : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};
    this.onSourceChange = typeof onSourceChange === "function" ? onSourceChange : () => {};
    this.onOpenBlock = typeof onOpenBlock === "function" ? onOpenBlock : () => {};
    this.readOnly = false;
    this.focusMatches = [];
    this.focusIndex = -1;
    this.guideHeaderRange = null;
    this.warningShowsGuide = false;
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

    this.root.querySelector("#btn-back-to-browse")?.addEventListener("click", () => {
      this.onBack();
    });

    this.root.querySelector("#btn-run-from-editor")?.addEventListener("click", () => {
      this.onRun(this.getDdn());
    });

    this.root.querySelector("#btn-save-ddn")?.addEventListener("click", () => {
      this.onSave(this.getDdn());
    });

    this.root.querySelector("#btn-block-mode")?.addEventListener("click", () => {
      this.onOpenBlock(this.getDdn(), {
        title: String(this.titleEl?.textContent ?? "DDN 편집"),
      });
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
    this.emitSourceChange();
  }

  getDdn() {
    return String(this.textarea?.value ?? "");
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
