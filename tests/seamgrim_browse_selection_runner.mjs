import path from "path";
import fs from "fs/promises";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function importBrowseScreenModule(modulePath) {
  // Load as data URL to avoid Node typeless-package reparsing warning noise.
  const source = await fs.readFile(modulePath, "utf8");
  const encoded = Buffer.from(String(source), "utf8").toString("base64");
  const moduleUrl = `data:text/javascript;base64,${encoded}`;
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

function createBrowseRoot() {
  const idMap = new Map();
  const root = createFakeElement("section");
  const tabOfficial = createFakeElement("button");
  const tabSearch = createFakeElement("button");
  tabOfficial.dataset.tab = "official";
  tabSearch.dataset.tab = "search";

  const requiredIds = [
    "btn-create",
    "btn-advanced-browse",
    "filter-grade",
    "filter-subject",
    "filter-quality",
    "filter-run-status",
    "filter-sort",
    "filter-query",
    "lesson-card-grid",
  ];
  requiredIds.forEach((id) => {
    idMap.set(id, createFakeElement("div"));
  });
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
      return [tabOfficial, tabSearch];
    }
    return [];
  };

  return {
    root,
    tabOfficial,
    tabSearch,
    qualitySelect: idMap.get("filter-quality"),
    grid: idMap.get("lesson-card-grid"),
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

  const { root, tabSearch, qualitySelect, grid } = createBrowseRoot();
  const selected = [];
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
                meta_path: ["/federated/meta.toml"],
              },
              {
                id: "rewrite_case_v1",
                title: "rewrite case",
                subject: "physics",
                source: "rewrite",
                ddnCandidates: ["/rewrite/lesson.ddn"],
                textCandidates: ["/rewrite/text.md"],
                metaCandidates: ["/rewrite/meta.toml"],
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
      onLessonSelect: async (lesson) => {
        selected.push(lesson);
      },
      onCreate: () => {},
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

    const federated = selectedById.get("federated_case_v1");
    assert(federated?.source === "federated", "federated case source default");
    assert(
      Array.isArray(federated?.ddnCandidates) && federated.ddnCandidates.includes("/federated/lesson.ddn"),
      "federated case ddn path",
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
