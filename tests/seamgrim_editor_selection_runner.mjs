import fs from "fs/promises";
import path from "path";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function importEditorScreenModule(modulePath) {
  const source = await fs.readFile(modulePath, "utf8");
  const encoded = Buffer.from(String(source), "utf8").toString("base64");
  return import(`data:text/javascript;base64,${encoded}`);
}

function createFakeElement(tag = "div") {
  const listeners = new Map();
  const classes = new Set();
  const children = [];
  const node = {
    tagName: String(tag).toUpperCase(),
    textContent: "",
    value: "",
    readOnly: false,
    disabled: false,
    selectionStart: 0,
    selectionEnd: 0,
    focused: false,
    children,
    addEventListener(type, handler) {
      const key = String(type);
      const list = listeners.get(key) ?? [];
      list.push(handler);
      listeners.set(key, list);
    },
    async emitAsync(type, payload = {}) {
      const key = String(type);
      const list = listeners.get(key) ?? [];
      for (const handler of list) {
        await handler({
          target: payload?.target ?? node,
          currentTarget: node,
          preventDefault() {},
          stopPropagation() {},
        });
      }
    },
    focus() {
      this.focused = true;
    },
    setSelectionRange(start, end) {
      this.selectionStart = Number(start);
      this.selectionEnd = Number(end);
    },
    appendChild(child) {
      children.push(child);
      return child;
    },
    classList: {
      add(cls) {
        classes.add(String(cls));
      },
      remove(cls) {
        classes.delete(String(cls));
      },
      toggle(cls, force) {
        const key = String(cls);
        if (force === true) {
          classes.add(key);
          return true;
        }
        if (force === false) {
          classes.delete(key);
          return false;
        }
        if (classes.has(key)) {
          classes.delete(key);
          return false;
        }
        classes.add(key);
        return true;
      },
      contains(cls) {
        return classes.has(String(cls));
      },
    },
  };
  return node;
}

function createEditorRoot() {
  const idMap = new Map();
  [
    "editor-title",
    "ddn-textarea",
    "editor-smoke-result",
    "editor-focus-bar",
    "editor-focus-summary",
    "editor-focus-warning",
    "editor-focus-select",
    "btn-editor-focus-prev",
    "btn-editor-focus-next",
    "btn-back-to-browse",
    "btn-run-from-editor",
    "btn-save-ddn",
    "btn-advanced-editor",
  ].forEach((id) => {
    idMap.set(id, createFakeElement(id === "ddn-textarea" ? "textarea" : "div"));
  });

  return {
    root: {
      querySelector(selector) {
        const key = String(selector ?? "");
        if (!key.startsWith("#")) return null;
        return idMap.get(key.slice(1)) ?? null;
      },
    },
    titleEl: idMap.get("editor-title"),
    textarea: idMap.get("ddn-textarea"),
    resultEl: idMap.get("editor-smoke-result"),
    focusBarEl: idMap.get("editor-focus-bar"),
    focusSummaryEl: idMap.get("editor-focus-summary"),
    focusWarningEl: idMap.get("editor-focus-warning"),
    focusSelectEl: idMap.get("editor-focus-select"),
    focusPrevBtn: idMap.get("btn-editor-focus-prev"),
    focusNextBtn: idMap.get("btn-editor-focus-next"),
  };
}

async function main() {
  const rootDir = process.cwd();
  const modulePath = path.resolve(rootDir, "solutions/seamgrim_ui_mvp/ui/screens/editor.js");
  const { EditorScreen } = await importEditorScreenModule(modulePath);
  const previousDocument = globalThis.document;
  globalThis.document = {
    createElement(tag) {
      return createFakeElement(tag);
    },
  };
  const { root, titleEl, textarea, resultEl, focusBarEl, focusSummaryEl, focusWarningEl, focusSelectEl, focusPrevBtn, focusNextBtn } =
    createEditorRoot();

  const screen = new EditorScreen({
    root,
    onBack: () => {},
    onRun: () => {},
    onSave: () => {},
    onOpenAdvanced: () => {},
  });
  screen.init();

  const candidate = "// 매김 전환 후보: g:수 <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }.";
  const secondCandidate = "// 매김 전환 후보: L:수 = (1) 매김 { 범위: 0.2..3. 간격: 0.1. }.";
  const ddnText = [
    "// --- 매김 전환 초안 ---",
    "(시작)할때 {",
    "  g <- 9.8. // 범위(1, 20, 0.1)",
    `  ${candidate}`,
    "  L <- 1.0. // 범위(0.2, 3, 0.1)",
    `  ${secondCandidate}`,
    "}.",
    "",
  ].join("\n");

  screen.loadLesson(ddnText, {
    title: "pendulum · 매김 전환 초안",
    readOnly: false,
    focusText: candidate,
    focusTexts: [candidate, secondCandidate],
  });

  const expectedStart = ddnText.indexOf(candidate);
  const expectedEnd = expectedStart + candidate.length;
  const secondStart = ddnText.indexOf(secondCandidate);
  const secondEnd = secondStart + secondCandidate.length;

  assert(titleEl.textContent === "pendulum · 매김 전환 초안", "editable title");
  assert(resultEl.textContent === "편집 모드", "editable mode result");
  assert(textarea.readOnly === false, "editable textarea");
  assert(textarea.focused === true, "textarea focused");
  assert(textarea.selectionStart === expectedStart, "selection start");
  assert(textarea.selectionEnd === expectedEnd, "selection end");
  assert(textarea.value === ddnText, "textarea value");
  assert(focusBarEl.classList.contains("hidden") === false, "focus bar visible");
  assert(focusSummaryEl.textContent === "매김 전환 후보 1/2 · g:수 <- · 1..20 · 0.1", "focus summary initial");
  assert(
    focusWarningEl.textContent === "안내 보기 · 원래 줄 삭제/교체",
    "focus warning initial",
  );
  assert(focusWarningEl.disabled === false, "focus warning enabled");
  assert(focusSelectEl.disabled === false, "focus select enabled");
  assert(focusSelectEl.children.length === 2, "focus select options");
  assert(focusSelectEl.children[0]?.textContent === "g:수", "focus select first option");
  assert(focusSelectEl.children[1]?.textContent === "L:수", "focus select second option");
  assert(focusPrevBtn.disabled === false, "prev enabled");
  assert(focusNextBtn.disabled === false, "next enabled");

  await focusNextBtn.emitAsync("click");
  assert(textarea.selectionStart === secondStart, "second selection start");
  assert(textarea.selectionEnd === secondEnd, "second selection end");
  assert(focusSummaryEl.textContent === "매김 전환 후보 2/2 · L:수 = · 0.2..3 · 0.1", "focus summary next");

  await focusPrevBtn.emitAsync("click");
  assert(textarea.selectionStart === expectedStart, "selection start after prev");
  assert(textarea.selectionEnd === expectedEnd, "selection end after prev");
  assert(focusSummaryEl.textContent === "매김 전환 후보 1/2 · g:수 <- · 1..20 · 0.1", "focus summary prev");

  focusSelectEl.value = "1";
  await focusSelectEl.emitAsync("change", { target: focusSelectEl });
  assert(textarea.selectionStart === secondStart, "selection start after select");
  assert(textarea.selectionEnd === secondEnd, "selection end after select");
  assert(focusSummaryEl.textContent === "매김 전환 후보 2/2 · L:수 = · 0.2..3 · 0.1", "focus summary select");

  const guideStart = ddnText.indexOf("// --- 매김 전환 초안 ---");
  const guideEnd = ddnText.indexOf("(시작)할때 {") - 1;
  await focusWarningEl.emitAsync("click");
  assert(textarea.selectionStart === guideStart, "guide selection start");
  assert(textarea.selectionEnd === guideEnd, "guide selection end");
  assert(focusWarningEl.textContent === "후보로 돌아가기 · 원래 줄 삭제/교체", "focus warning text after guide jump");
  assert(focusWarningEl.title === "클릭해 현재 후보 줄로 돌아가기", "focus warning title after guide jump");
  await focusWarningEl.emitAsync("click");
  assert(textarea.selectionStart === secondStart, "focus selection start after warning toggle back");
  assert(textarea.selectionEnd === secondEnd, "focus selection end after warning toggle back");
  assert(focusWarningEl.textContent === "안내 보기 · 원래 줄 삭제/교체", "focus warning text after toggle back");
  assert(focusWarningEl.title === "클릭해 상단 전환 안내 선택", "focus warning title after toggle back");

  screen.loadLesson(ddnText, {
    title: "pendulum",
    readOnly: true,
    focusText: candidate,
    focusTexts: [candidate, secondCandidate],
  });

  assert(titleEl.textContent === "pendulum (읽기 전용)", "readonly title");
  assert(resultEl.textContent === "교과 DDN 읽기 전용 모드", "readonly result");
  assert(focusBarEl.classList.contains("hidden") === true, "focus bar hidden");

  console.log("seamgrim editor selection runner ok");
  globalThis.document = previousDocument;
}

main().catch((error) => {
  console.error(error?.message ?? String(error));
  process.exit(1);
});
