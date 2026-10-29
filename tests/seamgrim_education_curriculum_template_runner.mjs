import fs from "fs/promises";
import http from "http";
import path from "path";
import { pathToFileURL } from "url";
import { chromium } from "playwright";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function fakeElement() {
  const classes = new Set(["hidden"]);
  return {
    textContent: "",
    innerHTML: "",
    classList: {
      add(name) {
        classes.add(name);
      },
      remove(name) {
        classes.delete(name);
      },
      contains(name) {
        return classes.has(name);
      },
    },
  };
}

async function readJson(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  return JSON.parse(text);
}

async function requireFile(filePath) {
  const stat = await fs.stat(filePath).catch(() => null);
  assert(stat?.isFile?.(), `missing file: ${filePath}`);
}

function mimeType(filePath) {
  if (filePath.endsWith(".html")) return "text/html; charset=utf-8";
  if (filePath.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (filePath.endsWith(".css")) return "text/css; charset=utf-8";
  if (filePath.endsWith(".json") || filePath.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (filePath.endsWith(".wasm")) return "application/wasm";
  if (filePath.endsWith(".toml") || filePath.endsWith(".ddn")) return "text/plain; charset=utf-8";
  if (filePath.endsWith(".md")) return "text/markdown; charset=utf-8";
  return "application/octet-stream";
}

function createStaticServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/favicon.ico") {
        res.writeHead(204, { "cache-control": "no-store" });
        res.end();
        return;
      }
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
      const filePath = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (filePath !== resolvedRoot && !filePath.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(filePath);
      res.writeHead(200, {
        "content-type": mimeType(filePath),
        "cache-control": "no-store",
        "access-control-allow-origin": "*",
      });
      res.end(bytes);
    } catch (_) {
      res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      res.end("not found");
    }
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("failed to bind static server"));
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    return pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory";
  } catch (_) {
    return false;
  }
}

async function waitVisible(page, selector) {
  await page.waitForFunction((sel) => {
    const node = document.querySelector(sel);
    if (!node) return false;
    return getComputedStyle(node).display !== "none" && !node.classList.contains("hidden");
  }, selector);
}

async function assertBrowserTeacherStudentFlow(root) {
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of ["index.html", "app.js", "styles.css", "screens/browse.js", "screens/run.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }
  const { server, baseUrl } = await createStaticServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1240, height: 820 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error" && !String(msg.text() ?? "").includes("Failed to load resource")) {
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400 && !res.url().endsWith("/favicon.ico") && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector(".lesson-card[data-lesson-id]");
    const browseState = await page.evaluate(() => ({
      cardCount: document.querySelectorAll(".lesson-card[data-lesson-id]").length,
      bodyText: document.body?.textContent ?? "",
      devRootExists: Boolean(document.querySelector("#dev-surface-root")),
      detailVisible: Boolean(document.querySelector("#catalog-detail-panel") && !document.querySelector("#catalog-detail-panel").classList.contains("hidden")),
    }));
    assert(browseState.cardCount >= 3, `expected representative course cards, got ${browseState.cardCount}`);
    assert(browseState.devRootExists === false, "dev surfaces must stay hidden on default teacher browse");
    for (const forbidden of ["릴리스 검토", "Question card", "자유 실험", "RPG Box", "Toolchain"]) {
      assert(!browseState.bodyText.includes(forbidden), `default browse leaked dev panel: ${forbidden}`);
    }
    if (!browseState.detailVisible) {
      await page.click(".lesson-card[data-lesson-id]");
      await waitVisible(page, "#catalog-detail-panel");
    }
    const detailState = await page.evaluate(() => ({
      title: document.querySelector("#detail-title")?.textContent?.trim() ?? "",
      hasStudentCta: Boolean(document.querySelector("#btn-open-in-studio")),
      hasTeacherCta: Boolean(document.querySelector("#btn-open-in-studio-teacher")),
      detailText: document.querySelector("#detail-curriculum")?.textContent ?? "",
    }));
    assert(detailState.title.length > 0 && detailState.title !== "교과 선택", `detail title mismatch: ${detailState.title}`);
    assert(detailState.hasStudentCta && detailState.hasTeacherCta, "detail panel needs student and teacher CTAs");
    assert(detailState.detailText.includes("DDN 원문"), "public detail must expose DDN source");

    await page.click("#btn-open-in-studio");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.onboardingProfile === "student");
    const studentState = await page.evaluate(() => ({
      launchKind: document.querySelector("#screen-run")?.dataset?.launchKind ?? "",
      onboarding: document.querySelector("#screen-run")?.dataset?.onboardingProfile ?? "",
      classroomSwitchDisplay: getComputedStyle(document.querySelector("[data-classroom-mode-switch]")).display,
      teacherPackageDisplay: getComputedStyle(document.querySelector("#btn-run-teacher-package-download")).display,
      teacherPackageDisabled: Boolean(document.querySelector("#btn-run-teacher-package-download")?.disabled),
      runButtonText: document.querySelector("#btn-run")?.textContent?.trim() ?? "",
    }));
    assert(studentState.launchKind === "browse_select_student", `student launch kind mismatch: ${studentState.launchKind}`);
    assert(studentState.onboarding === "student", `student onboarding mismatch: ${studentState.onboarding}`);
    assert(studentState.classroomSwitchDisplay === "none", "student launch must hide classroom mode switch");
    assert(studentState.teacherPackageDisplay === "none" || studentState.teacherPackageDisabled, "student launch must not expose teacher package download");
    assert(studentState.runButtonText.includes("수업 실행") || studentState.runButtonText.includes("실행"), `student run CTA mismatch: ${studentState.runButtonText}`);

    await page.evaluate(() => document.querySelector("#screen-run .main-shell-tab[data-main-tab-target='browse']")?.click());
    await waitVisible(page, "#screen-browse");
    await page.click(".lesson-card[data-lesson-id]");
    await waitVisible(page, "#catalog-detail-panel");
    await page.click("#btn-open-in-studio-teacher");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.onboardingProfile === "teacher");
    const teacherState = await page.evaluate(() => ({
      launchKind: document.querySelector("#screen-run")?.dataset?.launchKind ?? "",
      onboarding: document.querySelector("#screen-run")?.dataset?.onboardingProfile ?? "",
      classroomSwitchDisplay: getComputedStyle(document.querySelector("[data-classroom-mode-switch]")).display,
      teacherPressed: document.querySelector("[data-classroom-mode='teacher']")?.getAttribute("aria-pressed") ?? "",
      studentPressed: document.querySelector("[data-classroom-mode='student']")?.getAttribute("aria-pressed") ?? "",
      packageButtonDisplay: getComputedStyle(document.querySelector("#btn-run-teacher-package-download")).display,
      packageButtonDisabled: Boolean(document.querySelector("#btn-run-teacher-package-download")?.disabled),
      packageButtonText: document.querySelector("#btn-run-teacher-package-download")?.textContent?.trim() ?? "",
      reportButtonDisplay: getComputedStyle(document.querySelector("#btn-run-teacher-report-copy")).display,
    }));
    assert(teacherState.launchKind === "browse_select_teacher", `teacher launch kind mismatch: ${teacherState.launchKind}`);
    assert(teacherState.onboarding === "teacher", `teacher onboarding mismatch: ${teacherState.onboarding}`);
    assert(teacherState.classroomSwitchDisplay !== "none", "teacher launch must keep classroom mode switch");
    assert(teacherState.teacherPressed === "true" && teacherState.studentPressed === "false", "teacher classroom switch state mismatch");
    assert(teacherState.packageButtonDisplay !== "none", "teacher launch must expose package download");
    assert(teacherState.packageButtonDisabled === false, "teacher package download should be ready");
    assert(teacherState.packageButtonText.includes("배포"), `teacher package button text mismatch: ${teacherState.packageButtonText}`);
    assert(teacherState.reportButtonDisplay !== "none", "teacher launch must expose report copy");

    if (failures.length > 0) throw new Error(failures.join("\n"));
    await context.close();
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

async function main() {
  const root = process.cwd();
  const loaderPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/lesson_loader_contract.js");
  const browsePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/browse.js");
  const loader = await import(pathToFileURL(loaderPath).href);
  const browse = await import(pathToFileURL(browsePath).href);

  assert(typeof loader.parseTomlMeta === "function", "parseTomlMeta export");
  assert(typeof browse.BrowseScreen === "function", "BrowseScreen export");

  const seedMetaPath = path.resolve(root, "solutions/seamgrim_ui_mvp/seed_lessons_v1/roguelike_grid_pathfind_v1/meta.toml");
  const seedMeta = loader.parseTomlMeta(await fs.readFile(seedMetaPath, "utf8"));
  assert(seedMeta.schema === "CurriculumMetaV1", "seed meta schema");
  assert(seedMeta.lesson_id === "roguelike_grid_pathfind_v1", "seed meta lesson_id");
  assert(seedMeta.subject === "game", "seed meta subject");
  assert(seedMeta.grade === "all", "seed meta grade alias");
  assert(seedMeta.required_views.includes("console_grid"), "seed meta console_grid view");
  assert(seedMeta.required_views.includes("grid2d"), "seed meta grid2d view");
  assert(seedMeta.learning_goals.length >= 2, "seed meta learning goals");

  const koreanMeta = loader.parseTomlMeta(`
schema = "CurriculumMetaV1"
lesson_id = "ko_alias_lesson"
title = "한국어 키 차시"
"과목" = "경제"
"학년군" = "중등"
"단원" = "수요와 공급"
"차시" = "1차시"
"난이도" = "기본"
"학습목표" = ["목표 A"]
"핵심개념" = ["개념 A"]
"선수개념" = []
"오개념" = []
"허용조작" = []
"필수계기판" = ["graph", "table"]
evidence = ["checker"]
defaults = { "최대마디" = "24" }
`);
  assert(koreanMeta.subject === "경제", "korean alias subject");
  assert(koreanMeta.grade === "중등", "korean alias grade");
  assert(koreanMeta.learning_goals[0] === "목표 A", "korean alias learning_goals");
  assert(JSON.stringify(koreanMeta.required_views) === JSON.stringify(["graph", "table"]), "korean alias required_views");
  assert(koreanMeta.defaults?.["최대마디"] === "24", "inline table parse");

  const manifestPath = path.resolve(root, "solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson");
  const manifest = await readJson(manifestPath);
  const roguelikeSeed = manifest.seeds.find((row) => row?.seed_id === "roguelike_grid_pathfind_v1");
  assert(roguelikeSeed, "roguelike seed manifest row");
  assert(String(roguelikeSeed.meta_toml ?? "").endsWith("/meta.toml"), "roguelike seed meta_toml");
  assert(manifest.featured_seed_ids.includes("roguelike_grid_pathfind_v1"), "roguelike featured seed still exposed");

  const screen = new browse.BrowseScreen({ root: { querySelector: () => null } });
  screen.detailPanelEl = fakeElement();
  screen.detailSubjectBadgeEl = fakeElement();
  screen.detailTitleEl = fakeElement();
  screen.detailDescEl = fakeElement();
  screen.detailKeywordsEl = fakeElement();
  screen.detailCurriculumEl = fakeElement();
  screen.showLessonDetail({
    id: "roguelike_grid_pathfind_v1",
    title: seedMeta.title,
    description: "meta detail",
    subject: seedMeta.subject,
    grade: seedMeta.grade,
    curriculumMeta: {
      unit: seedMeta.unit,
      lesson: seedMeta.lesson,
      difficulty: seedMeta.difficulty,
      learningGoals: seedMeta.learning_goals,
      coreConcepts: seedMeta.core_concepts,
      requiredViews: seedMeta.required_views,
      teacherNotesRef: seedMeta.teacher_notes_ref,
      studentSheetRef: seedMeta.student_sheet_ref,
    },
  });
  assert(!screen.detailPanelEl.classList.contains("hidden"), "detail panel visible");
  assert(screen.detailCurriculumEl.innerHTML.includes("학습목표"), "detail learning goals title");
  assert(screen.detailCurriculumEl.innerHTML.includes("벽이 있는 격자"), "detail learning goal content");
  assert(screen.detailCurriculumEl.innerHTML.includes("핵심개념"), "detail core concepts title");
  assert(screen.detailCurriculumEl.innerHTML.includes("std_grid"), "detail core concept content");
  assert(screen.detailCurriculumEl.innerHTML.includes("결과 확인: 콘솔 격자, 2D 격자"), "detail required views");

  await assertBrowserTeacherStudentFlow(root);

  console.log("seamgrim education curriculum template runner ok");
}

main().catch((error) => {
  console.error(error?.stack ?? String(error));
  process.exit(1);
});
