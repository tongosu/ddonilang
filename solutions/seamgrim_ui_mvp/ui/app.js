import { createWasmLoader, applyWasmLogicAndDispatchState } from "./wasm_page_common.js";
import { BrowseScreen } from "./screens/browse.js";
import { EditorScreen, saveDdnToFile } from "./screens/editor.js";
import { RunScreen } from "./screens/run.js";
import {
  buildOverlaySessionRunsPayload,
  buildOverlayCompareSessionPayload,
  resolveOverlayCompareFromSession,
  buildSessionViewComboPayload,
  resolveSessionViewComboFromPayload,
} from "./overlay_session_contract.js";

const PROJECT_PREFIX = "solutions/seamgrim_ui_mvp/";

const appState = {
  currentLesson: null,
  currentScreen: "browse",
  wasm: {
    enabled: true,
    loader: null,
    client: null,
    fpsLimit: 30,
    dtMax: 0.1,
    langMode: "strict",
  },
  lessonsById: new Map(),
  screenListeners: new Set(),
};

function byId(id) {
  return document.getElementById(id);
}

function setScreen(name) {
  ["browse", "editor", "run"].forEach((screenName) => {
    const node = byId(`screen-${screenName}`);
    if (!node) return;
    node.classList.toggle("hidden", screenName !== name);
  });
  appState.currentScreen = name;
  appState.screenListeners.forEach((listener) => {
    try {
      listener(name);
    } catch (_) {
      // ignore screen listener errors
    }
  });
}

function onScreenChange(listener) {
  if (typeof listener !== "function") return;
  appState.screenListeners.add(listener);
}

function normalizePath(path) {
  return String(path ?? "")
    .replace(/\\/g, "/")
    .replace(/^\.\//, "")
    .replace(/^\//, "")
    .trim();
}

function normalizeSubject(raw) {
  const subject = String(raw ?? "").trim().toLowerCase();
  if (subject === "economy") return "econ";
  return subject;
}

function buildPathCandidates(path) {
  const normalized = normalizePath(path);
  if (!normalized) return [];
  if (/^https?:\/\//i.test(normalized)) return [normalized];

  const stripped = normalized.startsWith(PROJECT_PREFIX)
    ? normalized.slice(PROJECT_PREFIX.length)
    : normalized;
  const prefixed = normalized.startsWith(PROJECT_PREFIX)
    ? normalized
    : `${PROJECT_PREFIX}${normalized}`;

  const candidates = [
    normalized,
    `/${normalized}`,
    `./${normalized}`,
    stripped,
    `/${stripped}`,
    `./${stripped}`,
    prefixed,
    `/${prefixed}`,
    `./${prefixed}`,
  ].filter(Boolean);

  return Array.from(new Set(candidates));
}

async function fetchFirstOk(urls, parseAs = "text") {
  const tried = [];
  for (const url of urls) {
    tried.push(url);
    try {
      const response = await fetch(url, { cache: "no-cache" });
      if (!response.ok) continue;
      if (parseAs === "json") {
        return { ok: true, url, data: await response.json() };
      }
      return { ok: true, url, data: await response.text() };
    } catch (_) {
      // continue
    }
  }
  return { ok: false, url: tried[tried.length - 1] ?? "", data: null };
}

async function fetchJson(path) {
  const result = await fetchFirstOk(buildPathCandidates(path), "json");
  return result.ok ? result.data : null;
}

async function fetchText(pathCandidates) {
  const list = Array.isArray(pathCandidates)
    ? pathCandidates.flatMap((item) => buildPathCandidates(item))
    : buildPathCandidates(pathCandidates);
  const result = await fetchFirstOk(Array.from(new Set(list)), "text");
  return result.ok ? result.data : null;
}

function parseTomlMeta(text) {
  if (!text) return {};
  const out = {};
  const lines = String(text).replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  lines.forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) return;
    const match = trimmed.match(/^([A-Za-z0-9_]+)\s*=\s*(.+)$/);
    if (!match) return;
    const key = match[1].trim();
    const rawValue = match[2].trim();
    if (rawValue.startsWith('"') && rawValue.endsWith('"')) {
      out[key] = rawValue.slice(1, -1);
      return;
    }
    out[key] = rawValue;
  });
  return out;
}

function toLessonEntry(base) {
  const id = String(base.id ?? "").trim();
  if (!id) return null;
  return {
    id,
    title: String(base.title ?? id),
    description: String(base.description ?? ""),
    grade: String(base.grade ?? ""),
    subject: normalizeSubject(base.subject),
    quality: String(base.quality ?? "experimental"),
    source: String(base.source ?? "official"),
    ddnCandidates: Array.isArray(base.ddnCandidates) ? base.ddnCandidates.filter(Boolean) : [],
    textCandidates: Array.isArray(base.textCandidates) ? base.textCandidates.filter(Boolean) : [],
    metaCandidates: Array.isArray(base.metaCandidates) ? base.metaCandidates.filter(Boolean) : [],
  };
}

function mergeLessonEntry(map, nextEntry) {
  if (!nextEntry) return;
  const existing = map.get(nextEntry.id);
  if (!existing) {
    map.set(nextEntry.id, nextEntry);
    return;
  }

  const merged = {
    ...existing,
    ...nextEntry,
    ddnCandidates: Array.from(new Set([...(existing.ddnCandidates ?? []), ...(nextEntry.ddnCandidates ?? [])])),
    textCandidates: Array.from(new Set([...(existing.textCandidates ?? []), ...(nextEntry.textCandidates ?? [])])),
    metaCandidates: Array.from(new Set([...(existing.metaCandidates ?? []), ...(nextEntry.metaCandidates ?? [])])),
  };
  map.set(nextEntry.id, merged);
}

function lessonPathsFromId(prefix, lessonId) {
  return {
    ddn: `${prefix}/${lessonId}/lesson.ddn`,
    text: `${prefix}/${lessonId}/text.md`,
    meta: `${prefix}/${lessonId}/meta.toml`,
  };
}

async function loadCatalogLessons() {
  const merged = new Map();

  const indexJson = await fetchJson("solutions/seamgrim_ui_mvp/lessons/index.json");
  const indexLessons = Array.isArray(indexJson?.lessons) ? indexJson.lessons : [];
  indexLessons.forEach((row) => {
    const id = String(row.id ?? "").trim();
    if (!id) return;
    const paths = lessonPathsFromId("solutions/seamgrim_ui_mvp/lessons", id);
    mergeLessonEntry(
      merged,
      toLessonEntry({
        id,
        title: row.title,
        description: row.description,
        grade: row.grade,
        subject: row.subject,
        quality: "experimental",
        source: "official",
        ddnCandidates: [paths.ddn],
        textCandidates: [paths.text],
        metaCandidates: [paths.meta],
      }),
    );
  });

  const seedManifest = await fetchJson("solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson");
  const seeds = Array.isArray(seedManifest?.seeds) ? seedManifest.seeds : [];
  seeds.forEach((seed) => {
    const id = String(seed.seed_id ?? "").trim();
    if (!id) return;
    const fallback = lessonPathsFromId("solutions/seamgrim_ui_mvp/seed_lessons_v1", id);
    mergeLessonEntry(
      merged,
      toLessonEntry({
        id,
        title: id,
        description: "Seed lesson",
        grade: "all",
        subject: seed.subject,
        quality: "recommended",
        source: "seed",
        ddnCandidates: [seed.lesson_ddn, fallback.ddn],
        textCandidates: [seed.text_md, fallback.text],
        metaCandidates: [fallback.meta],
      }),
    );
  });

  const rewriteManifest = await fetchJson("solutions/seamgrim_ui_mvp/lessons_rewrite_v1/rewrite_manifest.detjson");
  const generated = Array.isArray(rewriteManifest?.generated) ? rewriteManifest.generated : [];
  generated.forEach((row) => {
    const id = String(row.lesson_id ?? "").trim();
    if (!id) return;
    const fallback = lessonPathsFromId("solutions/seamgrim_ui_mvp/lessons_rewrite_v1", id);
    mergeLessonEntry(
      merged,
      toLessonEntry({
        id,
        title: id,
        description: "Rewrite v1",
        grade: "all",
        subject: row.subject,
        quality: "reviewed",
        source: "rewrite",
        ddnCandidates: [row.generated_lesson_ddn, fallback.ddn],
        textCandidates: [row.generated_text_md, fallback.text],
        metaCandidates: [fallback.meta],
      }),
    );
  });

  const lessons = Array.from(merged.values()).sort((a, b) => String(a.title).localeCompare(String(b.title), "ko"));
  lessons.forEach((lesson) => {
    appState.lessonsById.set(lesson.id, lesson);
  });
  return lessons;
}

async function loadLessonById(lessonId) {
  const base = appState.lessonsById.get(lessonId);
  if (!base) throw new Error(`교과를 찾지 못했습니다: ${lessonId}`);

  const ddnText = await fetchText(base.ddnCandidates);
  if (!ddnText) {
    throw new Error(`lesson.ddn 로드 실패: ${lessonId}`);
  }

  const textMd = (await fetchText(base.textCandidates)) ?? "";
  const metaRaw = await fetchText(base.metaCandidates);
  const meta = parseTomlMeta(metaRaw);

  const lesson = {
    ...base,
    title: meta.title || base.title,
    description: meta.description || base.description,
    grade: meta.grade || base.grade,
    subject: normalizeSubject(meta.subject || base.subject),
    quality: String(meta.quality ?? base.quality ?? "experimental"),
    ddnText,
    textMd,
    meta,
  };

  appState.currentLesson = lesson;
  appState.lessonsById.set(lesson.id, lesson);
  return lesson;
}

function createAdvancedMenu({ onSmoke }) {
  const menu = byId("advanced-menu");
  const smokeBtn = byId("advanced-smoke");

  smokeBtn?.addEventListener("click", async () => {
    menu?.classList.add("hidden");
    if (typeof onSmoke === "function") {
      await onSmoke();
    }
  });

  window.addEventListener("click", (event) => {
    if (!menu || menu.classList.contains("hidden")) return;
    const target = event.target;
    if (!(target instanceof Element)) return;
    if (menu.contains(target)) return;
    const buttonIds = new Set([
      "btn-advanced-browse",
      "btn-advanced-editor",
      "btn-advanced-run",
    ]);
    if (buttonIds.has(target.id)) return;
    menu.classList.add("hidden");
  });

  return {
    toggle() {
      menu?.classList.toggle("hidden");
    },
    close() {
      menu?.classList.add("hidden");
    },
  };
}

function buildSessionContractScaffold() {
  const runs = buildOverlaySessionRunsPayload([]);
  const compare = buildOverlayCompareSessionPayload({
    enabled: false,
    baselineId: null,
    variantId: null,
  });
  const compareResolved = resolveOverlayCompareFromSession({
    runs: [],
    compare,
  });
  return {
    runs,
    compare,
    compareResolved,
    view_combo: buildSessionViewComboPayload({
      enabled: false,
      layout: "horizontal",
      overlayOrder: "graph",
    }),
    viewComboResolved: resolveSessionViewComboFromPayload({
      enabled: false,
      layout: "horizontal",
      overlay_order: "graph",
    }),
  };
}

async function main() {
  buildSessionContractScaffold();
  appState.wasm.loader = createWasmLoader({
    cacheBust: Date.now(),
    modulePath: "./wasm/ddonirang_tool.js",
    wrapperPath: "./wasm_ddn_wrapper.js",
    setStatus: () => {},
    clearStatusError: () => {},
  });

  const browseScreen = new BrowseScreen({
    root: byId("screen-browse"),
    onLessonSelect: async (lessonId) => {
      const lesson = await loadLessonById(lessonId);
      runScreen.loadLesson(lesson);
      setScreen("run");
    },
    onCreate: () => {
      editorScreen.loadBlank();
      setScreen("editor");
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
  });

  const editorScreen = new EditorScreen({
    root: byId("screen-editor"),
    onBack: () => {
      setScreen("browse");
    },
    onRun: (ddnText) => {
      const lesson = {
        id: appState.currentLesson?.id ?? "custom",
        title: appState.currentLesson?.title ?? "사용자 DDN",
        subject: appState.currentLesson?.subject ?? "",
        grade: appState.currentLesson?.grade ?? "",
        quality: appState.currentLesson?.quality ?? "experimental",
        ddnText,
        textMd: appState.currentLesson?.textMd ?? "",
        meta: appState.currentLesson?.meta ?? {},
      };
      appState.currentLesson = lesson;
      runScreen.loadLesson(lesson);
      setScreen("run");
    },
    onSave: (ddnText) => {
      saveDdnToFile(ddnText, "lesson.ddn");
      editorScreen.setSmokeResult("DDN 파일 저장 완료");
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
  });

  const runScreen = new RunScreen({
    root: byId("screen-run"),
    wasmState: appState.wasm,
    onBack: () => {
      setScreen("browse");
    },
    onEditDdn: ({ ddnText, title }) => {
      editorScreen.loadLesson(ddnText, { title, readOnly: true });
      setScreen("editor");
    },
    onOpenAdvanced: () => {
      advanced.toggle();
    },
  });

  const advanced = createAdvancedMenu({
    onSmoke: async () => {
      const source =
        appState.currentScreen === "editor"
          ? editorScreen.getDdn()
          : appState.currentLesson?.ddnText ?? "";
      if (!String(source).trim()) {
        editorScreen.setSmokeResult("Smoke: 검사할 DDN이 없습니다.");
        setScreen("editor");
        return;
      }
      try {
        const ensureWasm = (text) => appState.wasm.loader.ensure(text);
        const result = await applyWasmLogicAndDispatchState({
          sourceText: source,
          ensureWasm,
          mode: appState.wasm.langMode,
        });
        const hash = typeof result.client?.getStateHash === "function" ? result.client.getStateHash() : "-";
        editorScreen.setSmokeResult(`Smoke: 성공 · state_hash=${hash}`);
        if (appState.currentScreen !== "editor") {
          setScreen("editor");
        }
      } catch (err) {
        editorScreen.setSmokeResult(`Smoke: 실패 · ${String(err?.message ?? err)}`);
        if (appState.currentScreen !== "editor") {
          setScreen("editor");
        }
      }
    },
  });

  browseScreen.init();
  editorScreen.init();
  runScreen.init();
  onScreenChange((screenName) => {
    runScreen.setScreenVisible(screenName === "run");
  });

  try {
    const lessons = await loadCatalogLessons();
    browseScreen.setLessons(lessons);
  } catch (err) {
    console.error(err);
  }

  setScreen("browse");
}

void main();
