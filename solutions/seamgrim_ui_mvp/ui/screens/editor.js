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

export class EditorScreen {
  constructor({ root, onBack, onRun, onSave, onOpenAdvanced } = {}) {
    this.root = root;
    this.onBack = typeof onBack === "function" ? onBack : () => {};
    this.onRun = typeof onRun === "function" ? onRun : () => {};
    this.onSave = typeof onSave === "function" ? onSave : () => {};
    this.onOpenAdvanced = typeof onOpenAdvanced === "function" ? onOpenAdvanced : () => {};
    this.readOnly = false;
  }

  init() {
    this.titleEl = this.root.querySelector("#editor-title");
    this.textarea = this.root.querySelector("#ddn-textarea");
    this.resultEl = this.root.querySelector("#editor-smoke-result");

    this.root.querySelector("#btn-back-to-browse")?.addEventListener("click", () => {
      this.onBack();
    });

    this.root.querySelector("#btn-run-from-editor")?.addEventListener("click", () => {
      this.onRun(this.getDdn());
    });

    this.root.querySelector("#btn-save-ddn")?.addEventListener("click", () => {
      this.onSave(this.getDdn());
    });

    this.root.querySelector("#btn-advanced-editor")?.addEventListener("click", () => {
      this.onOpenAdvanced();
    });
  }

  loadLesson(ddnText, { title = "DDN 보기", readOnly = true } = {}) {
    this.readOnly = Boolean(readOnly);
    if (this.textarea) {
      this.textarea.value = String(ddnText ?? "");
      this.textarea.readOnly = this.readOnly;
    }
    if (this.titleEl) {
      this.titleEl.textContent = this.readOnly ? `${title} (읽기 전용)` : title;
    }
    this.setSmokeResult(this.readOnly ? "교과 DDN 읽기 전용 모드" : "편집 모드");
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
  }

  getDdn() {
    return String(this.textarea?.value ?? "");
  }

  setSmokeResult(message) {
    if (!this.resultEl) return;
    this.resultEl.textContent = String(message ?? "");
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
