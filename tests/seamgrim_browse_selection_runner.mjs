import path from "path";
import { pathToFileURL } from "url";

const originalEmitWarning = process.emitWarning?.bind(process);
if (typeof originalEmitWarning === "function") {
  process.emitWarning = (warning, ...args) => {
    const warningText =
      typeof warning === "string" ? warning : String(warning?.message ?? "");
    const firstArg = args[0];
    const warningCode =
      typeof firstArg === "string" ? firstArg : String(firstArg?.code ?? "");
    if (
      warningCode === "MODULE_TYPELESS_PACKAGE_JSON" ||
      warningText.includes("MODULE_TYPELESS_PACKAGE_JSON")
    ) {
      return;
    }
    return originalEmitWarning(warning, ...args);
  };
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function importBrowseScreenModule(modulePath) {
  const moduleUrl = pathToFileURL(modulePath).href;
  return import(moduleUrl);
}

function createMemoryStorage() {
  const table = new Map();
  return {
    getItem(key) {
      return table.has(String(key)) ? table.get(String(key)) : null;
    },
    setItem(key, value) {
      table.set(String(key), String(value));
    },
    removeItem(key) {
      table.delete(String(key));
    },
  };
}

function createFakeElement(tag = "div") {
  const listeners = new Map();
  const classes = new Set();
  const node = {
    tagName: String(tag).toUpperCase(),
    dataset: {},
    children: [],
    value: "",
    checked: false,
    textContent: "",
    className: "",
    _innerHTML: "",
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
    appendChild(child) {
      this.children.push(child);
      return child;
    },
    removeChild(child) {
      const idx = this.children.indexOf(child);
      if (idx >= 0) this.children.splice(idx, 1);
    },
    querySelector() {
      return null;
    },
    querySelectorAll() {
      return [];
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
  Object.defineProperty(node, "innerHTML", {
    get() {
      return this._innerHTML;
    },
    set(value) {
      this._innerHTML = String(value ?? "");
      this.children = [];
    },
  });
  return node;
}

function createBrowseRoot({ withDetailPanel = false } = {}) {
  const idMap = new Map();
  const root = createFakeElement("section");
  const tabOfficial = createFakeElement("button");
  const tabExamples = createFakeElement("button");
  const tabSearch = createFakeElement("button");
  tabOfficial.dataset.tab = "official";
  tabExamples.dataset.tab = "examples";
  tabSearch.dataset.tab = "search";

  const requiredIds = [
    "btn-create",
    "btn-preset-featured-seed-quick-recent",
    "btn-copy-browse-preset-link",
    "btn-advanced-browse",
    "filter-grade",
    "filter-subject",
    "filter-quality",
    "filter-seed-scope",
    "filter-run-status",
    "filter-run-launch",
    "filter-warning-status",
    "filter-sort",
    "filter-query",
    "browse-legacy-guide-hint",
    "lesson-card-grid",
  ];
  if (withDetailPanel) {
    requiredIds.push(
      "catalog-detail-panel",
      "detail-subject-badge",
      "detail-title",
      "detail-desc",
      "detail-keywords",
      "btn-open-in-studio",
      "btn-detail-close",
    );
  }
  requiredIds.forEach((id) => {
    idMap.set(id, createFakeElement("div"));
  });
  if (withDetailPanel) {
    idMap.get("catalog-detail-panel")?.classList?.add?.("hidden");
  }
  idMap.get("filter-sort").value = "recent";
  idMap.get("lesson-card-grid").appendChild = function appendChild(child) {
    this.children.push(child);
    return child;
  };

  root.querySelector = (selector) => {
    const key = String(selector ?? "");
    if (!key.startsWith("#")) return null;
    return idMap.get(key.slice(1)) ?? null;
  };
  root.querySelectorAll = (selector) => {
    const key = String(selector ?? "");
    if (key === ".browse-tab[data-tab]") {
      return [tabOfficial, tabExamples, tabSearch];
    }
    return [];
  };

  return {
    root,
    tabOfficial,
    tabExamples,
    tabSearch,
    qualitySelect: idMap.get("filter-quality"),
    seedScopeSelect: idMap.get("filter-seed-scope"),
    runLaunchSelect: idMap.get("filter-run-launch"),
    presetFeaturedSeedQuickRecentButton: idMap.get("btn-preset-featured-seed-quick-recent"),
    copyBrowsePresetLinkButton: idMap.get("btn-copy-browse-preset-link"),
    grid: idMap.get("lesson-card-grid"),
    detailPanel: idMap.get("catalog-detail-panel"),
    detailOpenBtn: idMap.get("btn-open-in-studio"),
  };
}

async function main() {
  const rootDir = process.cwd();
  const modulePath = path.resolve(rootDir, "solutions/seamgrim_ui_mvp/ui/screens/browse.js");
  const { BrowseScreen } = await importBrowseScreenModule(modulePath);

  const previousWindow = globalThis.window;
  const previousDocument = globalThis.document;
  const previousNavigator = globalThis.navigator;
  const previousFetch = globalThis.fetch;

  const setGlobal = (key, value) => {
    try {
      globalThis[key] = value;
      return;
    } catch (_) {
      // fallback below
    }
    Object.defineProperty(globalThis, key, {
      value,
      configurable: true,
      writable: true,
      enumerable: true,
    });
  };

  const { root, tabExamples, tabSearch, qualitySelect, grid } = createBrowseRoot();
  const selected = [];
  const selectionEvents = [];
  const openedExamples = [];
  let inventoryFetchCount = 0;
  const requestedUrls = [];

  setGlobal("window", {
    location: { href: "http://127.0.0.1:8787/" },
    localStorage: createMemoryStorage(),
    addEventListener() {},
    setTimeout(fn) {
      if (typeof fn === "function") fn();
      return 1;
    },
  });
  setGlobal("document", {
    body: createFakeElement("body"),
    createElement(tag) {
      return createFakeElement(tag);
    },
    execCommand() {
      return true;
    },
  });
  setGlobal("navigator", {});
  setGlobal("fetch", async (url) => {
    const pathText = String(url ?? "");
    requestedUrls.push(pathText);
    if (pathText.includes("/api/lessons/inventory")) {
      inventoryFetchCount += 1;
      return {
        ok: true,
        async json() {
          return {
            lessons: [
              {
                id: "federated_case_v1",
                title: "federated case",
                subject: "econ",
                ddn_path: ["/federated/lesson.ddn"],
                text_path: ["/federated/text.md"],
                structure_path: ["/federated/structure.json"],
                meta_path: ["/federated/meta.toml"],
                maegim_control_warning_count: 2,
                maegim_control_warning_codes: ["W_LEGACY_RANGE_COMMENT_DEPRECATED"],
                maegim_control_warning_names: ["g", "L"],
                maegim_control_warning_examples: [
                  "g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }.",
                  "L <- (1) 매김 { 범위: 0.2..3. 간격: 0.1. }.",
                ],
                maegim_control_warning_source: "legacy",
              },
              {
                id: "rewrite_case_v1",
                title: "rewrite case",
                subject: "physics",
                source: "rewrite",
                ddnCandidates: ["/rewrite/lesson.ddn"],
                textCandidates: ["/rewrite/text.md"],
                metaCandidates: ["/rewrite/meta.toml"],
                maegim_control_warning_count: 1,
                maegim_control_warning_codes: ["W_LEGACY_RANGE_COMMENT_DEPRECATED"],
                maegim_control_warning_names: ["theta"],
                maegim_control_warning_examples: [
                  "theta:수 <- (0.5) 매김 { 범위: -1.2..1.2. 간격: 0.05. }.",
                ],
                maegim_control_warning_source: "rewrite_legacy",
              },
              {
                id: "seed_case_v1",
                title: "seed case",
                subject: "math",
                source: "seed",
                lesson_ddn_path: "/seed/lesson.ddn",
                text_md_path: "/seed/text.md",
              },
            ],
          };
        },
      };
    }
    if (pathText.includes("/samples/index.json")) {
      return {
        ok: true,
        async json() {
          return {
            samples: [
              {
                id: "sample_console_v1",
                title: "sample console",
                subject: "console-grid",
                source: "sample",
                first_run_path: "hello",
                tags: ["first_run", "hello"],
                ddn_path: ["/samples/sample_console_v1.ddn"],
              },
              {
                id: "sample_space_v1",
                title: "sample space",
                subject: "space2d",
                source: "sample",
                first_run_path: "movement",
                tags: ["first_run", "movement"],
                ddn_path: ["/samples/sample_space_v1.ddn"],
              },
            ],
          };
        },
      };
    }
    if (pathText.includes("/structure.json")) {
      return {
        ok: true,
        async json() {
          return {
            schema: "seamgrim.structure.v0",
            meta: { title: "구조 미리보기" },
            nodes: [{ id: "A", label: "시작" }, { id: "B", label: "끝" }],
            edges: [{ from: "A", to: "B", directed: true }],
          };
        },
      };
    }
    if (pathText.includes("/graph.json")) {
      return {
        ok: true,
        async json() {
          return {
            schema: "seamgrim.graph.v0",
            meta: { title: "선형 추세" },
            series: [
              {
                id: "y",
                points: [{ x: 0, y: 1 }, { x: 1, y: 3 }, { x: 2, y: 2 }],
              },
            ],
          };
        },
      };
    }
    if (pathText.includes("/table.json")) {
      return {
        ok: true,
        async json() {
          return {
            schema: "seamgrim.table.v0",
            columns: [{ key: "coef", label: "계수" }, { key: "value", label: "값" }],
            rows: [{ coef: "a", value: 1.5 }, { coef: "b", value: -1 }],
          };
        },
      };
    }
    if (pathText.includes("/space2d.json")) {
      return {
        ok: true,
        async json() {
          return {
            schema: "seamgrim.space2d.v0",
            meta: { title: "궤적" },
            points: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
          };
        },
      };
    }
    return {
      ok: false,
      async json() {
        return {};
      },
    };
  });

  try {
    const screen = new BrowseScreen({
      root,
      onLessonSelect: async (lesson, options = {}) => {
        selected.push(lesson);
        selectionEvents.push({
          lessonId: String(lesson?.id ?? ""),
          autoExecute: Boolean(options?.autoExecute),
        });
      },
      onCreate: () => {},
      onOpenLegacyGuideExample: ({ lesson, example }) => {
        openedExamples.push({
          lessonId: String(lesson?.id ?? ""),
          example: String(example ?? ""),
        });
      },
      onOpenAdvanced: () => {},
    });
    screen.init();
    screen.setLessons([
      {
        id: "official_recommended",
        title: "official recommended",
        description: "",
        grade: "all",
        subject: "math",
        source: "official",
        quality: "recommended",
      },
      {
        id: "official_reviewed",
        title: "official reviewed",
        description: "",
        grade: "all",
        subject: "math",
        source: "official",
        quality: "reviewed",
      },
      {
        id: "official_experimental",
        title: "official experimental",
        description: "",
        grade: "all",
        subject: "math",
        source: "official",
        quality: "experimental",
      },
    ]);
    assert(Array.isArray(grid.children) && grid.children.length === 3, "official cards rendered");
    assert(
      root.querySelector("#btn-preset-featured-seed-quick-recent")?.disabled === true,
      "featured quick recent preset disabled when quick-launch history absent",
    );
    assert(
      root.querySelector("#btn-copy-browse-preset-link")?.disabled === true,
      "browse preset copy button disabled when preset inactive",
    );

    qualitySelect.value = "recommended";
    await qualitySelect.emitAsync("change");
    assert(screen.filter.quality === "recommended", "quality filter value updated");
    assert(Array.isArray(grid.children) && grid.children.length === 1, "quality filter reduced cards");
    const onlyCard = grid.children[0];
    assert(
      String(onlyCard?.dataset?.lessonId ?? "") === "official_recommended",
      "quality filter keeps recommended card",
    );

    qualitySelect.value = "";
    await qualitySelect.emitAsync("change");
    assert(Array.isArray(grid.children) && grid.children.length === 3, "quality filter reset restores cards");

    const featuredRoot = createBrowseRoot();
    globalThis.window?.localStorage?.setItem(
      "seamgrim.ui.run_prefs.v1",
      JSON.stringify({
        lessons: {
          bio_sir_transition_visual_seed_v2: {
            lastRunKind: "space2d",
            lastRunChannels: 3,
            lastRunAt: "2026-03-13T03:00:00.000Z",
            lastRunHash: "abc123featuredseed",
            lastLaunchKind: "featured_seed_quick",
          },
          econ_supply_demand_seed_v1: {
            lastRunKind: "space2d",
            lastRunChannels: 3,
            lastRunAt: "2026-03-13T02:00:00.000Z",
            lastRunHash: "abc123legacyseed",
            lastLaunchKind: "browse_select",
          },
        },
      }),
    );
    const featuredScreen = new BrowseScreen({
      root: featuredRoot.root,
      onLessonSelect: async () => {},
      onCreate: () => {},
      onOpenLegacyGuideExample: () => {},
      onOpenAdvanced: () => {},
    });
    featuredScreen.init();
    featuredScreen.setLessons([
      {
        id: "official_reviewed_v1",
        title: "official reviewed v1",
        description: "",
        grade: "all",
        subject: "math",
        source: "official",
        quality: "reviewed",
      },
      {
        id: "econ_supply_demand_seed_v1",
        title: "seed legacy",
        description: "",
        grade: "all",
        subject: "econ",
        source: "seed",
        quality: "recommended",
      },
      {
        id: "bio_sir_transition_visual_seed_v2",
        title: "seed featured",
        description: "",
        grade: "all",
        subject: "biology",
        source: "seed",
        quality: "recommended",
      },
    ]);
    const presetFeaturedSeedQuickRecentButton = featuredRoot.presetFeaturedSeedQuickRecentButton;
    const copyBrowsePresetLinkButton = featuredRoot.copyBrowsePresetLinkButton;
    assert(
      presetFeaturedSeedQuickRecentButton?.disabled === false,
      "featured quick recent preset enabled when quick-launch lesson exists",
    );
    assert(
      copyBrowsePresetLinkButton?.disabled === true,
      "browse preset copy button disabled before preset selection",
    );
    const featuredScopeSelect = featuredRoot.seedScopeSelect;
    featuredScopeSelect.value = "featured_seed";
    await featuredScopeSelect.emitAsync("change");
    assert(featuredScreen.filter.seedScope === "featured_seed", "featured seed scope value updated");
    const featuredScopePrefsRaw = globalThis.window?.localStorage?.getItem("seamgrim.ui.browse_prefs.v1");
    const featuredScopePrefs = featuredScopePrefsRaw ? JSON.parse(featuredScopePrefsRaw) : {};
    assert(featuredScopePrefs?.seedScope === "featured_seed", "featured seed scope saved to browse prefs");
    const seedScopePersistRoot = createBrowseRoot();
    const seedScopePersistScreen = new BrowseScreen({
      root: seedScopePersistRoot.root,
      onLessonSelect: async () => {},
      onCreate: () => {},
      onOpenLegacyGuideExample: () => {},
      onOpenAdvanced: () => {},
    });
    seedScopePersistScreen.init();
    assert(seedScopePersistScreen.filter.seedScope === "featured_seed", "featured seed scope restored from browse prefs");
    assert(
      String(seedScopePersistRoot.seedScopeSelect?.value ?? "") === "featured_seed",
      "featured seed scope select restored from browse prefs",
    );
    assert(Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 1, "featured scope keeps only new seeds");
    assert(
      String(featuredRoot.grid.children[0]?.dataset?.lessonId ?? "") === "bio_sir_transition_visual_seed_v2",
      "featured scope keeps only featured seed card",
    );
    featuredScopeSelect.value = "seed_only";
    await featuredScopeSelect.emitAsync("change");
    assert(featuredScreen.filter.seedScope === "seed_only", "seed-only scope value updated");
    assert(Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 2, "seed-only scope keeps all seeds");
    featuredScopeSelect.value = "";
    await featuredScopeSelect.emitAsync("change");
    assert(Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 3, "seed scope reset restores cards");

    const featuredSortSelect = featuredRoot.root.querySelector("#filter-sort");
    featuredSortSelect.value = "featured_seed";
    await featuredSortSelect.emitAsync("change");
    assert(featuredScreen.filter.sort === "featured_seed", "featured seed sort value updated");
    assert(Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 3, "featured sort keeps cards");
    assert(
      String(featuredRoot.grid.children[0]?.dataset?.lessonId ?? "") === "bio_sir_transition_visual_seed_v2",
      "featured sort puts new seed first",
    );
    assert(
      String(featuredRoot.grid.children[0]?.innerHTML ?? "").includes("신규 seed"),
      "featured seed badge rendered on card",
    );
    assert(
      String(featuredRoot.grid.children[0]?.innerHTML ?? "").includes("Alt+6 실행"),
      "featured quick launch badge rendered on card",
    );
    assert(
      !String(featuredRoot.grid.children[1]?.innerHTML ?? "").includes("Alt+6 실행"),
      "non quick-launch card does not render featured quick badge",
    );
    assert(
      String(featuredRoot.grid.children[1]?.dataset?.lessonId ?? "") === "econ_supply_demand_seed_v1",
      "featured sort keeps old seed next",
    );
    assert(
      String(featuredRoot.grid.children[2]?.dataset?.lessonId ?? "") === "official_reviewed_v1",
      "featured sort keeps non-seed last",
    );
    featuredSortSelect.value = "featured_seed_quick_recent";
    await featuredSortSelect.emitAsync("change");
    assert(
      featuredScreen.filter.sort === "featured_seed_quick_recent",
      "featured quick recent sort value updated",
    );
    assert(Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 3, "featured quick recent sort keeps cards");
    assert(
      String(featuredRoot.grid.children[0]?.dataset?.lessonId ?? "") === "bio_sir_transition_visual_seed_v2",
      "featured quick recent sort puts Alt+6 lesson first",
    );
    assert(
      String(featuredRoot.grid.children[1]?.dataset?.lessonId ?? "") === "econ_supply_demand_seed_v1",
      "featured quick recent sort keeps recent non-quick second",
    );
    assert(
      String(featuredRoot.grid.children[2]?.dataset?.lessonId ?? "") === "official_reviewed_v1",
      "featured quick recent sort keeps non-run lesson last",
    );
    const featuredQuickRecentPrefsRaw = globalThis.window?.localStorage?.getItem("seamgrim.ui.browse_prefs.v1");
    const featuredQuickRecentPrefs = featuredQuickRecentPrefsRaw ? JSON.parse(featuredQuickRecentPrefsRaw) : {};
    assert(
      featuredQuickRecentPrefs?.sort === "featured_seed_quick_recent",
      "featured quick recent sort saved to browse prefs",
    );
    featuredScopeSelect.value = "";
    await featuredScopeSelect.emitAsync("change");
    const featuredSortResetSelect = featuredRoot.root.querySelector("#filter-sort");
    featuredSortResetSelect.value = "default";
    await featuredSortResetSelect.emitAsync("change");
    const runLaunchResetSelect = featuredRoot.runLaunchSelect;
    runLaunchResetSelect.value = "";
    await runLaunchResetSelect.emitAsync("change");
    await presetFeaturedSeedQuickRecentButton.emitAsync("click");
    assert(
      featuredScreen.filter.sort === "featured_seed_quick_recent",
      "featured quick recent preset click applies sort",
    );
    assert(
      featuredScreen.filter.runLaunch === "featured_seed_quick",
      "featured quick recent preset click applies run launch filter",
    );
    assert(
      presetFeaturedSeedQuickRecentButton?.classList?.contains("active") === true,
      "featured quick recent preset active style applied",
    );
    assert(
      Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 1,
      "featured quick recent preset narrows to quick-launch lessons",
    );
    assert(
      String(featuredRoot.grid.children[0]?.dataset?.lessonId ?? "") === "bio_sir_transition_visual_seed_v2",
      "featured quick recent preset keeps Alt+6 lesson",
    );
    assert(
      copyBrowsePresetLinkButton?.disabled === false,
      "browse preset copy button enabled after preset activation",
    );
    assert(
      String(copyBrowsePresetLinkButton?.title ?? "").includes("browsePreset=featured_seed_quick_recent"),
      "browse preset copy button title includes deep-link query",
    );
    await copyBrowsePresetLinkButton.emitAsync("click");
    const toastHostExists = Array.isArray(globalThis.document?.body?.children)
      ? globalThis.document.body.children.some(
          (node) => String(node?.className ?? "").trim() === "ui-toast-host",
        )
      : false;
    assert(toastHostExists, "browse preset copy click shows global toast host");
    const applyPresetOk = featuredScreen.applyBrowsePreset("featured_seed_quick_recent");
    assert(applyPresetOk === true, "applyBrowsePreset accepted featured quick recent");
    assert(
      featuredScreen.filter.sort === "featured_seed_quick_recent" &&
        featuredScreen.filter.runLaunch === "featured_seed_quick",
      "applyBrowsePreset set featured quick recent filters",
    );
    const applyPresetUnknown = featuredScreen.applyBrowsePreset("unknown_preset");
    assert(applyPresetUnknown === false, "applyBrowsePreset rejects unknown preset");
    const runLaunchSelect = featuredRoot.runLaunchSelect;
    runLaunchSelect.value = "featured_seed_quick";
    await runLaunchSelect.emitAsync("change");
    assert(featuredScreen.filter.runLaunch === "featured_seed_quick", "run launch filter value updated");
    assert(
      Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 1,
      "run launch filter keeps only quick-launch lessons",
    );
    assert(
      String(featuredRoot.grid.children[0]?.dataset?.lessonId ?? "") === "bio_sir_transition_visual_seed_v2",
      "run launch filter keeps featured quick-launch lesson",
    );
    const runLaunchPrefsRaw = globalThis.window?.localStorage?.getItem("seamgrim.ui.browse_prefs.v1");
    const runLaunchPrefs = runLaunchPrefsRaw ? JSON.parse(runLaunchPrefsRaw) : {};
    assert(runLaunchPrefs?.runLaunch === "featured_seed_quick", "run launch filter saved to browse prefs");
    const runLaunchPersistRoot = createBrowseRoot();
    const runLaunchPersistScreen = new BrowseScreen({
      root: runLaunchPersistRoot.root,
      onLessonSelect: async () => {},
      onCreate: () => {},
      onOpenLegacyGuideExample: () => {},
      onOpenAdvanced: () => {},
    });
    runLaunchPersistScreen.init();
    assert(
      runLaunchPersistScreen.filter.runLaunch === "featured_seed_quick",
      "run launch filter restored from browse prefs",
    );
    assert(
      String(runLaunchPersistRoot.runLaunchSelect?.value ?? "") === "featured_seed_quick",
      "run launch select restored from browse prefs",
    );
    runLaunchSelect.value = "none";
    await runLaunchSelect.emitAsync("change");
    assert(
      Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 1,
      "run launch none filter keeps lessons without launch history",
    );
    assert(
      String(featuredRoot.grid.children[0]?.dataset?.lessonId ?? "") === "official_reviewed_v1",
      "run launch none filter keeps launch-empty lesson",
    );
    runLaunchSelect.value = "";
    await runLaunchSelect.emitAsync("change");
    assert(
      Array.isArray(featuredRoot.grid.children) && featuredRoot.grid.children.length === 3,
      "run launch filter reset restores cards",
    );
    assert(
      copyBrowsePresetLinkButton?.disabled === true,
      "browse preset copy button disabled after preset reset",
    );

    await tabSearch.emitAsync("click");
    assert(screen.activeTab === "search", "browse search tab activation");
    assert(inventoryFetchCount >= 1, "inventory api fetch called");
    assert(
      requestedUrls.every((item) => !String(item).includes("build/reports/seamgrim_lesson_inventory.json")),
      "default search does not probe build/reports inventory",
    );
    assert(Array.isArray(screen.searchResults) && screen.searchResults.length === 3, "federated results loaded");
    assert(Array.isArray(grid.children) && grid.children.length === 3, "search cards rendered");

    for (const card of grid.children) {
      assert(card && typeof card.emitAsync === "function", "rendered card is clickable");
      await card.emitAsync("click");
    }

    assert(selected.length === 3, "onLessonSelect called for each card");
    const selectedById = new Map(selected.map((row) => [String(row?.id ?? ""), row]));
    assert(selectedById.has("federated_case_v1"), "federated case selected");
    assert(selectedById.has("rewrite_case_v1"), "rewrite case selected");
    assert(selectedById.has("seed_case_v1"), "seed case selected");
    assert(selectionEvents.every((row) => row.autoExecute === true), "browse selection autoExecute forwarded");

    const federated = selectedById.get("federated_case_v1");
    assert(federated?.source === "federated", "federated case source default");
    assert(
      Array.isArray(federated?.ddnCandidates) && federated.ddnCandidates.includes("/federated/lesson.ddn"),
      "federated case ddn path",
    );
    assert(
      Array.isArray(federated?.structureCandidates) && federated.structureCandidates.includes("/federated/structure.json"),
      "federated case structure path",
    );
    assert(Number(federated?.maegimControlWarningCount) === 2, "federated case legacy warning count");
    assert(
      Array.isArray(federated?.maegimControlWarningCodes) &&
        federated.maegimControlWarningCodes.includes("W_LEGACY_RANGE_COMMENT_DEPRECATED"),
      "federated case legacy warning code",
    );
    assert(
      Array.isArray(federated?.maegimControlWarningNames) &&
        federated.maegimControlWarningNames.includes("g") &&
        federated.maegimControlWarningNames.includes("L"),
      "federated case legacy warning names",
    );
    assert(
      Array.isArray(federated?.maegimControlWarningExamples) &&
        federated.maegimControlWarningExamples.includes("g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }."),
      "federated case legacy warning examples",
    );
    const federatedCard = grid.children.find((card) => String(card?.dataset?.lessonId ?? "") === "federated_case_v1");
    assert(
      String(federatedCard?.innerHTML ?? "").includes("구식범위주석 2건"),
      "browse card renders legacy warning badge",
    );
    const previewProbe = createFakeElement("div");
    await screen.hydrateLessonPreview(previewProbe, federated);
    assert(
      String(previewProbe.innerHTML ?? "").includes("runtime-structure-preview"),
      "browse structure preview rendered",
    );
    assert(String(previewProbe?.dataset?.previewFamily ?? "") === "structure", "browse structure preview family metadata");
    assert(String(previewProbe?.dataset?.previewMode ?? "") === "json", "browse structure preview mode metadata");
    assert(String(previewProbe?.dataset?.previewCount ?? "") === "1", "browse structure preview count metadata");
    assert(String(previewProbe?.dataset?.previewFamilies ?? "") === "structure", "browse structure preview families metadata");
    assert(String(previewProbe?.dataset?.previewTitle ?? "") === "구조 미리보기", "browse structure preview title metadata");
    assert(String(previewProbe?.dataset?.previewHeader ?? "") === "구조 미리보기", "browse structure preview header metadata");
    assert(
      String(previewProbe?.dataset?.previewSummary ?? "").includes("구조 미리보기 · 노드 2개 · 간선 1개"),
      "browse structure preview summary metadata",
    );
    assert(
      String(previewProbe?.title ?? "").includes("구조 미리보기 · 노드 2개 · 간선 1개"),
      "browse structure preview tooltip metadata",
    );
    const tablePreviewProbe = createFakeElement("div");
    await screen.hydrateLessonPreview(tablePreviewProbe, {
      id: "table_preview_case",
      requiredViews: ["table", "text"],
      tableCandidates: ["/preview/table.json"],
      textCandidates: ["/preview/text.md"],
    });
    assert(
      String(tablePreviewProbe.innerHTML ?? "").includes("lesson-card-table-preview"),
      "browse table preview rendered",
    );
    assert(String(tablePreviewProbe?.dataset?.previewFamily ?? "") === "table", "browse table preview family metadata");
    assert(String(tablePreviewProbe?.dataset?.previewMode ?? "") === "json", "browse table preview mode metadata");
    assert(String(tablePreviewProbe?.dataset?.previewCount ?? "") === "1", "browse table preview count metadata");
    assert(String(tablePreviewProbe?.dataset?.previewTitle ?? "") === "", "browse table preview title metadata");
    const spacePreviewProbe = createFakeElement("div");
    await screen.hydrateLessonPreview(spacePreviewProbe, {
      id: "space_preview_case",
      requiredViews: ["space2d"],
      space2dCandidates: ["/preview/space2d.json"],
    });
    assert(
      String(spacePreviewProbe.innerHTML ?? "").includes("lesson-card-space2d-canvas"),
      "browse space2d preview rendered",
    );
    assert(String(spacePreviewProbe?.dataset?.previewFamily ?? "") === "space2d", "browse space2d preview family metadata");
    assert(String(spacePreviewProbe?.dataset?.previewMode ?? "") === "json", "browse space2d preview mode metadata");
    assert(String(spacePreviewProbe?.dataset?.previewCount ?? "") === "1", "browse space2d preview count metadata");
    assert(String(spacePreviewProbe?.dataset?.previewTitle ?? "") === "궤적", "browse space2d preview title metadata");
    const graphPreviewProbe = createFakeElement("div");
    await screen.hydrateLessonPreview(graphPreviewProbe, {
      id: "graph_preview_case",
      requiredViews: ["graph"],
      graphCandidates: ["/preview/graph.json"],
      textCandidates: ["/preview/text.md"],
    });
    assert(
      String(graphPreviewProbe.innerHTML ?? "").includes("lesson-card-graph-canvas"),
      "browse graph preview rendered from graph asset",
    );
    assert(String(graphPreviewProbe?.dataset?.previewFamily ?? "") === "graph", "browse graph preview family metadata");
    assert(String(graphPreviewProbe?.dataset?.previewMode ?? "") === "json", "browse graph preview mode metadata");
    assert(String(graphPreviewProbe?.dataset?.previewCount ?? "") === "1", "browse graph preview count metadata");
    assert(String(graphPreviewProbe?.dataset?.previewTitle ?? "") === "선형 추세", "browse graph preview title metadata");
    const inferredPreviewProbe = createFakeElement("div");
    await screen.hydrateLessonPreview(inferredPreviewProbe, {
      id: "inferred_preview_case",
      requiredViews: [],
      structureCandidates: ["/preview/structure.json"],
      graphCandidates: ["/preview/graph.json"],
      space2dCandidates: ["/preview/space2d.json"],
    });
    assert(
      String(inferredPreviewProbe.innerHTML ?? "").includes("lesson-card-space2d-canvas"),
      "browse inferred preview prefers canonical space2d family",
    );
    assert(
      String(inferredPreviewProbe?.dataset?.previewFamily ?? "") === "space2d",
      "browse inferred preview family metadata uses chosen family",
    );
    assert(String(inferredPreviewProbe?.dataset?.previewFamilies ?? "") === "space2d", "browse inferred preview families metadata");
    assert(String(inferredPreviewProbe?.dataset?.previewTitle ?? "") === "궤적", "browse inferred preview title metadata");
    const inferredCard = screen.createLessonCard({
      id: "inferred_preview_card_case",
      title: "inferred preview card",
      description: "",
      grade: "all",
      subject: "math",
      source: "official",
      quality: "reviewed",
      requiredViews: [],
      structureCandidates: ["/preview/structure.json"],
      graphCandidates: ["/preview/graph.json"],
      space2dCandidates: ["/preview/space2d.json"],
    });
    assert(
      Array.isArray(inferredCard.children) && inferredCard.children.length === 1,
      "browse card attaches preview even without requiredViews when preview assets exist",
    );
    screen.showLegacyWarningGuide(federated);
    const legacyGuideHint = root.querySelector("#browse-legacy-guide-hint");
    assert(
      String(legacyGuideHint?.textContent ?? "").includes("구식 범위주석 2건"),
      "legacy warning guide hint rendered",
    );
    assert(
      String(legacyGuideHint?.textContent ?? "").includes("매김"),
      "legacy warning guide hint includes maegim guidance",
    );
    assert(
      String(legacyGuideHint?.textContent ?? "").includes("원래 `// 범위(...)`가 붙은 선언 줄을 지우거나"),
      "legacy warning guide hint includes delete-or-replace warning",
    );
    assert(
      String(legacyGuideHint?.textContent ?? "").includes("대상 항목: g, L"),
      "legacy warning guide hint includes warning names",
    );
    assert(
      String(legacyGuideHint?.textContent ?? "").includes("g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }."),
      "legacy warning guide hint includes autofix example",
    );
    assert(legacyGuideHint?.classList?.contains("hidden") === false, "legacy warning guide hint shown");
    const openExampleButton = Array.isArray(legacyGuideHint?.children) ? legacyGuideHint.children[0] : null;
    assert(openExampleButton && typeof openExampleButton.emitAsync === "function", "legacy guide open button rendered");
    await openExampleButton.emitAsync("click");
    assert(openedExamples.length === 1, "legacy guide open callback called");
    assert(openedExamples[0]?.lessonId === "federated_case_v1", "legacy guide open callback lesson id");
    assert(
      openedExamples[0]?.example === "g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }.",
      "legacy guide open callback example",
    );
    const detailRoot = createBrowseRoot({ withDetailPanel: true });
    const detailSelections = [];
    const detailScreen = new BrowseScreen({
      root: detailRoot.root,
      onLessonSelect: async (lesson, options = {}) => {
        detailSelections.push({
          lessonId: String(lesson?.id ?? ""),
          autoExecute: Boolean(options?.autoExecute),
        });
      },
      onCreate: () => {},
      onOpenLegacyGuideExample: () => {},
      onOpenAdvanced: () => {},
    });
    detailScreen.init();
    detailScreen.setLessons([
      {
        id: "detail_case_v1",
        title: "detail case",
        description: "detail description",
        grade: "all",
        subject: "math",
        source: "official",
        quality: "reviewed",
      },
    ]);
    const detailCard = detailRoot.grid.children[0];
    await detailCard.emitAsync("click");
    assert(detailSelections.length === 0, "detail card click opens panel before selection");
    assert(detailRoot.detailPanel?.classList?.contains("hidden") === false, "detail panel shown");
    await detailRoot.detailOpenBtn.emitAsync("click");
    assert(detailSelections.length === 1, "detail CTA triggers selection");
    assert(detailSelections[0].lessonId === "detail_case_v1", "detail CTA forwards lesson");
    assert(detailSelections[0].autoExecute === true, "detail CTA forwards autoExecute");
    const warningSelect = root.querySelector("#filter-warning-status");
    warningSelect.value = "has_legacy_warning";
    await warningSelect.emitAsync("change");
    assert(screen.filter.warningStatus === "has_legacy_warning", "warning filter value updated");
    assert(Array.isArray(grid.children) && grid.children.length === 2, "warning filter reduces cards");
    assert(
      String(grid.children[0]?.dataset?.lessonId ?? "") === "federated_case_v1",
      "warning filter keeps highest warned lesson first",
    );
    assert(
      String(grid.children[1]?.dataset?.lessonId ?? "") === "rewrite_case_v1",
      "warning filter keeps additional warned lesson",
    );
    warningSelect.value = "clean";
    await warningSelect.emitAsync("change");
    assert(Array.isArray(grid.children) && grid.children.length === 1, "clean filter excludes warned lesson");
    warningSelect.value = "";
    await warningSelect.emitAsync("change");
    assert(Array.isArray(grid.children) && grid.children.length === 3, "warning filter reset restores cards");
    const sortSelect = root.querySelector("#filter-sort");
    sortSelect.value = "legacy_warning";
    await sortSelect.emitAsync("change");
    assert(screen.filter.sort === "legacy_warning", "warning sort value updated");
    const browsePrefsRaw = globalThis.window?.localStorage?.getItem("seamgrim.ui.browse_prefs.v1");
    const browsePrefs = browsePrefsRaw ? JSON.parse(browsePrefsRaw) : {};
    assert(browsePrefs?.sort === "legacy_warning", "warning sort saved to browse prefs");
    assert(String(browsePrefs?.seedScope ?? "") === "", "seed scope saved as empty when unset");
    assert(Array.isArray(grid.children) && grid.children.length === 3, "warning sort keeps cards");
    assert(
      String(grid.children[0]?.dataset?.lessonId ?? "") === "federated_case_v1",
      "warning sort puts highest warning count first",
    );
    assert(
      String(grid.children[1]?.dataset?.lessonId ?? "") === "rewrite_case_v1",
      "warning sort keeps second warning count next",
    );
    assert(
      String(grid.children[2]?.dataset?.lessonId ?? "") === "seed_case_v1",
      "warning sort leaves clean lesson last",
    );
    const persistedRoot = createBrowseRoot();
    const persistedScreen = new BrowseScreen({
      root: persistedRoot.root,
      onLessonSelect: async () => {},
      onCreate: () => {},
      onOpenLegacyGuideExample: () => {},
      onOpenAdvanced: () => {},
    });
    persistedScreen.init();
    assert(persistedScreen.filter.sort === "legacy_warning", "warning sort restored from browse prefs");
    assert(
      String(persistedRoot.root.querySelector("#filter-sort")?.value ?? "") === "legacy_warning",
      "warning sort select restored from browse prefs",
    );
    sortSelect.value = "recent";
    await sortSelect.emitAsync("change");

    await tabExamples.emitAsync("click");
    assert(screen.activeTab === "examples", "browse examples tab activation");
    assert(Array.isArray(screen.sampleResults) && screen.sampleResults.length === 2, "sample results loaded");
    assert(Array.isArray(grid.children) && grid.children.length === 2, "sample cards rendered");
    await grid.children[0].emitAsync("click");
    const sampleSelected = selected[selected.length - 1];
    assert(String(sampleSelected?.source ?? "") === "sample", "sample selection source preserved");
    assert(
      Array.isArray(sampleSelected?.ddnCandidates) &&
        sampleSelected.ddnCandidates.includes("/samples/sample_console_v1.ddn"),
      "sample selection ddn path preserved",
    );
    assert(String(sampleSelected?.firstRunPath ?? "") === "hello", "sample selection first_run_path preserved");
    assert(
      Array.isArray(sampleSelected?.tags) && sampleSelected.tags.includes("first_run") && sampleSelected.tags.includes("hello"),
      "sample selection tags preserved",
    );
    assert(
      String(grid.children[0]?.innerHTML ?? "").includes("첫실행 첫 인사"),
      "sample card renders first-run badge",
    );
    assert(
      String(grid.children[0]?.innerHTML ?? "").includes("첫 시작 첫 인사 · 첫 인사 -> 움직임 -> 매김 조절 -> 되돌려보기/거울"),
      "sample card renders first-run hint",
    );

    const rewrite = selectedById.get("rewrite_case_v1");
    assert(rewrite?.source === "rewrite", "rewrite case source");
    assert(
      Array.isArray(rewrite?.ddnCandidates) && rewrite.ddnCandidates.includes("/rewrite/lesson.ddn"),
      "rewrite case ddn path",
    );
    assert(
      Array.isArray(rewrite?.metaCandidates) && rewrite.metaCandidates.includes("/rewrite/meta.toml"),
      "rewrite case meta path",
    );

    const seed = selectedById.get("seed_case_v1");
    assert(seed?.source === "seed", "seed case source");
    assert(
      Array.isArray(seed?.ddnCandidates) && seed.ddnCandidates.includes("/seed/lesson.ddn"),
      "seed case ddn path",
    );
    assert(
      Array.isArray(seed?.textCandidates) && seed.textCandidates.includes("/seed/text.md"),
      "seed case text path",
    );

    const probeRoot = createBrowseRoot();
    const probeRequested = [];
    setGlobal("fetch", async (url) => {
      const pathText = String(url ?? "");
      probeRequested.push(pathText);
      return {
        ok: false,
        async json() {
          return {};
        },
      };
    });
    const probeScreen = new BrowseScreen({
      root: probeRoot.root,
      onLessonSelect: async () => {},
      onCreate: () => {},
      onOpenLegacyGuideExample: () => {},
      onOpenAdvanced: () => {},
    });
    probeScreen.init();
    await probeRoot.tabSearch.emitAsync("click");
    assert(probeScreen.federatedLoadState === "unavailable", "default probe unavailable when api missing");
    assert(
      probeRequested.includes("/api/lessons/inventory"),
      "default probe attempts inventory api",
    );
    assert(
      probeRequested.every((item) => !String(item).includes("build/reports/seamgrim_lesson_inventory.json")),
      "default probe does not attempt file inventory",
    );

    const fileRoot = createBrowseRoot();
    const fileRequested = [];
    setGlobal("fetch", async (url) => {
      const pathText = String(url ?? "");
      fileRequested.push(pathText);
      if (pathText.includes("build/reports/seamgrim_lesson_inventory.json")) {
        return {
          ok: true,
          async json() {
            return {
              lessons: [
                {
                  id: "file_inventory_case_v1",
                  title: "file inventory case",
                  source: "federated",
                  ddn_path: ["/file/lesson.ddn"],
                },
              ],
            };
          },
        };
      }
      return {
        ok: false,
        async json() {
          return {};
        },
      };
    });
    const fileScreen = new BrowseScreen({
      root: fileRoot.root,
      federatedFileCandidates: ["build/reports/seamgrim_lesson_inventory.json"],
      onLessonSelect: async () => {},
      onCreate: () => {},
      onOpenAdvanced: () => {},
    });
    fileScreen.init();
    await fileRoot.tabSearch.emitAsync("click");
    assert(fileScreen.federatedLoadState === "loaded", "file fallback loads when configured");
    assert(
      fileRequested.some((item) => String(item).includes("build/reports/seamgrim_lesson_inventory.json")),
      "file fallback requested configured inventory path",
    );
    assert(
      Array.isArray(fileScreen.searchResults) &&
        fileScreen.searchResults.some((row) => String(row?.id ?? "") === "file_inventory_case_v1"),
      "file fallback result parsed",
    );

    console.log("seamgrim browse selection runner ok");
    return 0;
  } finally {
    setGlobal("window", previousWindow);
    setGlobal("document", previousDocument);
    setGlobal("navigator", previousNavigator);
    setGlobal("fetch", previousFetch);
  }
}

main().catch((err) => {
  const msg = String(err?.message ?? err);
  console.error(`check=browse_selection_flow detail=${msg}`);
  process.exit(1);
});
