import { createWasmLoader, applyWasmLogicAndDispatchState } from "./wasm_page_common.js";
import { BrowseScreen } from "./screens/browse.js";
import { EditorScreen, saveDdnToFile } from "./screens/editor.js";
import { RunScreen } from "./screens/run.js";

const PROJECT_PREFIX = "solutions/seamgrim_ui_mvp/";
const SIM_CORE_POLICY_CLASS = "policy-sim-core";

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

function readWindowStringArray(key, fallback = []) {
  try {
    const value = window?.[key];
    if (!Array.isArray(value)) return fallback;
    const out = [];
    const seen = new Set();
    value.forEach((item) => {
      const row = String(item ?? "").trim();
      if (!row || seen.has(row)) return;
      seen.add(row);
      out.push(row);
    });
    return out;
  } catch (_) {
    return fallback;
  }
}

function readWindowBoolean(key, fallback = false) {
  try {
    const value = window?.[key];
    if (typeof value === "boolean") return value;
    const text = String(value ?? "").trim().toLowerCase();
    if (!text) return fallback;
    if (text === "1" || text === "true" || text === "yes" || text === "on") return true;
    if (text === "0" || text === "false" || text === "no" || text === "off") return false;
    return fallback;
  } catch (_) {
    return fallback;
  }
}

function applySimCorePolicy() {
  const enabled = readWindowBoolean("SEAMGRIM_SIM_CORE_POLICY", true);
  try {
    document?.body?.classList?.toggle(SIM_CORE_POLICY_CLASS, enabled);
  } catch (_) {
    // ignore class toggle errors
  }
  return enabled;
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

function isProjectPrefixedHost() {
  try {
    const pathname = String(window?.location?.pathname ?? "").trim();
    if (!pathname) return false;
    return pathname.includes(`/${PROJECT_PREFIX}`) || pathname.startsWith(`/${PROJECT_PREFIX.slice(0, -1)}`);
  } catch (_) {
    return false;
  }
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

  // 404 노이즈를 줄이기 위해 절대 경로 후보만 최소 집합으로 유지한다.
  const primary = `/${stripped}`;
  const secondary = `/${prefixed}`;
  if (primary === secondary) return [primary];
  return isProjectPrefixedHost() ? [secondary, primary] : [primary, secondary];
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

function prioritizeLessonCandidates(candidates, source) {
  const list = Array.isArray(candidates) ? candidates.filter(Boolean).map((item) => String(item)) : [];
  if (list.length <= 1) return list;
  const mode = String(source ?? "").trim().toLowerCase();
  const rankOf = (path) => {
    const normalized = normalizePath(path);
    if (mode === "rewrite") {
      if (normalized.includes("lessons_rewrite_v1/")) return 0;
      if (normalized.includes("lessons/")) return 1;
      return 2;
    }
    if (mode === "seed") {
      if (normalized.includes("seed_lessons_v1/")) return 0;
      if (normalized.includes("lessons/")) return 1;
      return 2;
    }
    return 0;
  };
  return [...list].sort((a, b) => rankOf(a) - rankOf(b));
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

function sourceToLessonPrefix(source) {
  const normalized = String(source ?? "").trim().toLowerCase();
  if (normalized === "seed") return "solutions/seamgrim_ui_mvp/seed_lessons_v1";
  if (normalized === "rewrite") return "solutions/seamgrim_ui_mvp/lessons_rewrite_v1";
  return "solutions/seamgrim_ui_mvp/lessons";
}

function toStringArray(value) {
  if (Array.isArray(value)) {
    return value.map((item) => String(item ?? "").trim()).filter(Boolean);
  }
  const single = String(value ?? "").trim();
  return single ? [single] : [];
}

function resolveSelectionCandidates(selection, keys) {
  const out = [];
  keys.forEach((key) => {
    out.push(...toStringArray(selection?.[key]));
  });
  return Array.from(new Set(out));
}

function ensureLessonEntryFromSelection(selection) {
  if (typeof selection === "string") {
    return String(selection).trim();
  }
  if (!selection || typeof selection !== "object") {
    return "";
  }
  const id = String(selection.id ?? selection.lesson_id ?? "").trim();
  if (!id) return "";

  const source = String(selection.source ?? "federated");
  const fallback = lessonPathsFromId(sourceToLessonPrefix(source), id);
  const ddnCandidates = resolveSelectionCandidates(selection, ["ddnCandidates", "ddn_path", "lesson_ddn_path"]);
  const textCandidates = resolveSelectionCandidates(selection, ["textCandidates", "text_path", "text_md_path"]);
  const metaCandidates = resolveSelectionCandidates(selection, ["metaCandidates", "meta_path"]);

  const nextEntry = toLessonEntry({
    id,
    title: selection.title,
    description: selection.description,
    grade: selection.grade,
    subject: selection.subject,
    quality: selection.quality,
    source,
    ddnCandidates: ddnCandidates.length ? ddnCandidates : [fallback.ddn],
    textCandidates: textCandidates.length ? textCandidates : [fallback.text],
    // meta.toml is optional; do not synthesize fallback paths that trigger noisy 404s.
    metaCandidates,
  });
  mergeLessonEntry(appState.lessonsById, nextEntry);
  return id;
}

function mergeCatalogFromInventoryPayload(merged, payload) {
  const rows = Array.isArray(payload?.lessons) ? payload.lessons : [];
  rows.forEach((row) => {
    const id = String(row?.id ?? row?.lesson_id ?? "").trim();
    if (!id) return;
    const source = String(row?.source ?? "official").trim() || "official";
    const fallback = lessonPathsFromId(sourceToLessonPrefix(source), id);
    const ddnCandidates = resolveSelectionCandidates(row, ["ddnCandidates", "ddn_path", "lesson_ddn_path"]);
    const textCandidates = resolveSelectionCandidates(row, ["textCandidates", "text_path", "text_md_path"]);
    const metaCandidates = resolveSelectionCandidates(row, ["metaCandidates", "meta_path"]);
    mergeLessonEntry(
      merged,
      toLessonEntry({
        id,
        title: row?.title ?? row?.name ?? id,
        description: row?.description ?? "",
        grade: row?.grade ?? "all",
        subject: row?.subject ?? "",
        quality: row?.quality ?? "experimental",
        source,
        ddnCandidates: ddnCandidates.length ? ddnCandidates : [fallback.ddn],
        textCandidates: textCandidates.length ? textCandidates : [fallback.text],
        metaCandidates,
      }),
    );
  });
}

async function loadCatalogLessons() {
  const merged = new Map();

  const inventoryApi = await fetchFirstOk(["/api/lessons/inventory", "/api/lesson-inventory"], "json");
  if (inventoryApi.ok) {
    mergeCatalogFromInventoryPayload(merged, inventoryApi.data);
  }

  if (merged.size === 0) {
    const indexJson = await fetchJson("solutions/seamgrim_ui_mvp/lessons/index.json");
    const indexLessons = Array.isArray(indexJson?.lessons) ? indexJson.lessons : [];
    indexLessons.forEach((row) => {
      const id = String(row.id ?? "").trim();
      if (!id) return;
      const paths = lessonPathsFromId("solutions/seamgrim_ui_mvp/lessons", id);
      const rowMetaCandidates = resolveSelectionCandidates(row, ["metaCandidates", "meta_path"]);
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
          metaCandidates: rowMetaCandidates,
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
          metaCandidates: resolveSelectionCandidates(seed, ["metaCandidates", "meta_path", "meta_toml"]),
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
          metaCandidates: resolveSelectionCandidates(row, ["metaCandidates", "meta_path", "meta_toml"]),
        }),
      );
    });
  }

  const lessons = Array.from(merged.values()).sort((a, b) => String(a.title).localeCompare(String(b.title), "ko"));
  if (lessons.length === 0) {
    throw new Error(
      "교과 카탈로그를 찾지 못했습니다. ddn_exec_server.py(8787)로 실행했는지 확인하세요.",
    );
  }
  lessons.forEach((lesson) => {
    appState.lessonsById.set(lesson.id, lesson);
  });
  return lessons;
}

async function loadLessonById(lessonId) {
  const base = appState.lessonsById.get(lessonId);
  if (!base) throw new Error(`교과를 찾지 못했습니다: ${lessonId}`);

  base.ddnCandidates = prioritizeLessonCandidates(base.ddnCandidates, base.source);
  base.textCandidates = prioritizeLessonCandidates(base.textCandidates, base.source);
  base.metaCandidates = prioritizeLessonCandidates(base.metaCandidates, base.source);

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

async function main() {
  applySimCorePolicy();

  appState.wasm.loader = createWasmLoader({
    cacheBust: Date.now(),
    modulePath: "./wasm/ddonirang_tool.js",
    wrapperPath: "./wasm_ddn_wrapper.js",
    setStatus: () => {},
    clearStatusError: () => {},
  });

  const federatedApiCandidates = readWindowStringArray("SEAMGRIM_FEDERATED_API_CANDIDATES", [
    "/api/lessons/inventory",
  ]);
  const allowFederatedFileFallback = readWindowBoolean("SEAMGRIM_ENABLE_FEDERATED_FILE_FALLBACK", false);
  const allowShapeFallback = readWindowBoolean("SEAMGRIM_ENABLE_SHAPE_FALLBACK", false);
  const federatedFileCandidates = allowFederatedFileFallback
    ? readWindowStringArray("SEAMGRIM_FEDERATED_FILE_CANDIDATES", [])
    : [];

  const browseScreen = new BrowseScreen({
    root: byId("screen-browse"),
    federatedApiCandidates,
    federatedFileCandidates,
    onLessonSelect: async (selection) => {
      const lessonId = ensureLessonEntryFromSelection(selection);
      if (!lessonId) {
        throw new Error("교과를 찾지 못했습니다: invalid lesson selection");
      }
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
    allowShapeFallback,
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
