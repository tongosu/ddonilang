#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { chromium } from "playwright";

const OK = "studio_local_package_export_action: ok";
const LEGACY_LOCAL_PACKAGE_COPY_BUTTON_ID = "btn-run-local-package-copy";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) throw new Error(`missing file: ${file}`);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml") || file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  if (file.endsWith(".md")) return "text/markdown; charset=utf-8";
  return "application/octet-stream";
}

function createServer(root) {
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
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
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
      if (!address || typeof address === "string") reject(new Error("failed to bind static server"));
      else resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
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
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if ((pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/")) && /\/(?:graph|table|space2d|text|maegim_control)\.(?:json|md)$/i.test(pathname)) return true;
  } catch (_) {
    return false;
  }
  return false;
}

async function waitVisible(page, selector) {
  await page.waitForFunction((sel) => {
    const node = document.querySelector(sel);
    return node && !String(node.className ?? "").split(/\s+/).includes("hidden");
  }, selector);
}

async function assertLocalPackageValidation(uiRoot) {
  const helper = await import(pathToFileURL(path.join(uiRoot, "studio_local_share_package.js")).href);
  assert(
    helper.formatStudioLocalPackageError(new Error("studio_local_package_missing_lesson")) === "배포 묶음에 실행할 교과가 없습니다.",
    "known package error should be formatted for teachers",
  );
  assert(
    helper.formatStudioLocalPackageError(new Error("SyntaxError: Unexpected token < in JSON")) === "배포 묶음을 열 수 없습니다.",
    "unknown browser/internal package errors should not leak to teachers",
  );
  assert(
    helper.formatStudioLocalPackageError(new Error("배포 파일에 실행할 교과 원문이 없습니다.")) === "배포 파일에 실행할 교과 원문이 없습니다.",
    "already localized package errors should be preserved",
  );
  const lesson = {
    lesson_id: "rep_physics_velocity_history_v1",
    title: "속도 기록",
    source_text: "보임 { text: \"속도 기록\" }",
  };
  const report = {
    report_id: "rep_physics_velocity_history_v1_report",
    title: "속도 기록 리포트",
    text: "mode\tteacher",
  };
  const manifest = helper.buildStudioLocalPackageManifest({
    packageId: "studio.local.validation",
    title: "검증 배포 묶음",
    lessons: [lesson],
    reports: [report],
  });
  const payload = helper.buildStudioLocalPackagePayload({ manifest, lessons: [lesson], reports: [report] });
  const validation = helper.validateStudioLocalPackagePayload(payload);
  assert(validation.valid === true, `valid package rejected: ${validation.error}`);
  assert(validation.lesson_count === 1 && validation.report_count === 1, "valid package counts mismatch");
  const imported = helper.importStudioLocalPackagePayload(payload);
  assert(imported.lesson_count === 1 && imported.report_count === 1, "imported package counts mismatch");

  const unsafeIdGeneratedPayload = helper.buildStudioLocalPackagePayload({
    lessons: [{
      lesson_id: "lesson/id:with spaces",
      title: "표시 ID 보존",
      source_text: "보임 { text: \"표시 ID 보존\" }",
    }],
    reports: [{
      report_id: "report/id:with spaces",
      title: "표시 ID 보존 리포트",
      text: "mode\tteacher",
    }],
  });
  assert(
    unsafeIdGeneratedPayload.lessons[0].lesson_id === "lesson/id:with spaces",
    "generated package should preserve display lesson id",
  );
  assert(
    unsafeIdGeneratedPayload.reports[0].report_id === "report/id:with spaces",
    "generated package should preserve display report id",
  );
  assert(
    unsafeIdGeneratedPayload.lessons[0].path === "lessons/lesson_id_with_spaces.ddn",
    `generated lesson path should be portable: ${unsafeIdGeneratedPayload.lessons[0].path}`,
  );
  assert(
    unsafeIdGeneratedPayload.reports[0].path === "reports/report_id_with_spaces.txt",
    `generated report path should be portable: ${unsafeIdGeneratedPayload.reports[0].path}`,
  );
  const unsafeIdGeneratedValidation = helper.validateStudioLocalPackagePayload(unsafeIdGeneratedPayload);
  assert(unsafeIdGeneratedValidation.valid === true, `portable unsafe-id package rejected: ${unsafeIdGeneratedValidation.error}`);

  const emptyLessonPayload = helper.buildStudioLocalPackagePayload({
    lessons: [{ ...lesson, source_text: "" }],
    reports: [report],
  });
  const emptyValidation = helper.validateStudioLocalPackagePayload(emptyLessonPayload);
  assert(emptyValidation.valid === false, "empty lesson source should be rejected");
  assert(emptyValidation.error === "studio_local_package_empty_lesson_source", `empty lesson error mismatch: ${emptyValidation.error}`);

  const mismatchedPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      lesson_count: 2,
    },
  };
  const mismatchValidation = helper.validateStudioLocalPackagePayload(mismatchedPayload);
  assert(mismatchValidation.valid === false, "mismatched lesson count should be rejected");
  assert(mismatchValidation.error === "studio_local_package_lesson_count_mismatch", `mismatch error mismatch: ${mismatchValidation.error}`);

  const invalidCountPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      lesson_count: "one",
    },
  };
  const invalidCountValidation = helper.validateStudioLocalPackagePayload(invalidCountPayload);
  assert(invalidCountValidation.valid === false, "invalid manifest count should be rejected");
  assert(
    invalidCountValidation.error === "studio_local_package_lesson_count_mismatch",
    `invalid count error mismatch: ${invalidCountValidation.error}`,
  );

  const missingLessonFilePayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: payload.manifest.files.filter((item) => item.path !== "lessons/rep_physics_velocity_history_v1.ddn"),
      file_count: payload.manifest.files.length - 1,
    },
  };
  const missingLessonFileValidation = helper.validateStudioLocalPackagePayload(missingLessonFilePayload);
  assert(missingLessonFileValidation.valid === false, "missing manifest lesson file should be rejected");
  assert(
    missingLessonFileValidation.error === "studio_local_package_manifest_missing_lesson_file",
    `missing lesson file error mismatch: ${missingLessonFileValidation.error}`,
  );

  const lessonByteSizeMismatchPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: payload.manifest.files.map((item) => (
        item.path === "lessons/rep_physics_velocity_history_v1.ddn"
          ? { ...item, byte_size: Number(item.byte_size ?? 0) + 1 }
          : item
      )),
    },
  };
  const byteSizeValidation = helper.validateStudioLocalPackagePayload(lessonByteSizeMismatchPayload);
  assert(byteSizeValidation.valid === false, "lesson byte size mismatch should be rejected");
  assert(
    byteSizeValidation.error === "studio_local_package_lesson_byte_size_mismatch",
    `lesson byte size error mismatch: ${byteSizeValidation.error}`,
  );

  const invalidByteSizePayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: payload.manifest.files.map((item) => (
        item.path === "lessons/rep_physics_velocity_history_v1.ddn"
          ? { ...item, byte_size: "many" }
          : item
      )),
    },
  };
  const invalidByteSizeValidation = helper.validateStudioLocalPackagePayload(invalidByteSizePayload);
  assert(invalidByteSizeValidation.valid === false, "invalid manifest byte_size should be rejected");
  assert(
    invalidByteSizeValidation.error === "studio_local_package_lesson_byte_size_mismatch",
    `invalid byte_size error mismatch: ${invalidByteSizeValidation.error}`,
  );

  const lessonTypeMismatchPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: payload.manifest.files.map((item) => (
        item.path === "lessons/rep_physics_velocity_history_v1.ddn"
          ? { ...item, type: "asset" }
          : item
      )),
    },
  };
  const lessonTypeValidation = helper.validateStudioLocalPackagePayload(lessonTypeMismatchPayload);
  assert(lessonTypeValidation.valid === false, "lesson manifest type mismatch should be rejected");
  assert(
    lessonTypeValidation.error === "studio_local_package_lesson_type_mismatch",
    `lesson type error mismatch: ${lessonTypeValidation.error}`,
  );

  const entryMismatchPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      entry: "app.js",
    },
  };
  const entryValidation = helper.validateStudioLocalPackagePayload(entryMismatchPayload);
  assert(entryValidation.valid === false, "manifest entry mismatch should be rejected");
  assert(
    entryValidation.error === "studio_local_package_entry_mismatch",
    `entry mismatch error mismatch: ${entryValidation.error}`,
  );

  const formatMismatchPayload = {
    ...payload,
    import_export_format: "studio_local_package_payload_v0",
  };
  const formatValidation = helper.validateStudioLocalPackagePayload(formatMismatchPayload);
  assert(formatValidation.valid === false, "format mismatch should be rejected");
  assert(
    formatValidation.error === "studio_local_package_format_mismatch",
    `format mismatch error mismatch: ${formatValidation.error}`,
  );

  const remoteBoundaryPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      cloud_sync: true,
    },
  };
  const remoteBoundaryValidation = helper.validateStudioLocalPackagePayload(remoteBoundaryPayload);
  assert(remoteBoundaryValidation.valid === false, "remote boundary package should be rejected");
  assert(
    remoteBoundaryValidation.error === "studio_local_package_remote_boundary_mismatch",
    `remote boundary error mismatch: ${remoteBoundaryValidation.error}`,
  );

  const unsafePathPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: payload.manifest.files.map((item) => (
        item.path === "lessons/rep_physics_velocity_history_v1.ddn"
          ? { ...item, path: "../rep_physics_velocity_history_v1.ddn" }
          : item
      )),
    },
    lessons: payload.lessons.map((item) => ({
      ...item,
      path: "../rep_physics_velocity_history_v1.ddn",
    })),
  };
  const unsafePathValidation = helper.validateStudioLocalPackagePayload(unsafePathPayload);
  assert(unsafePathValidation.valid === false, "unsafe package path should be rejected");
  assert(
    unsafePathValidation.error === "studio_local_package_unsafe_path",
    `unsafe path error mismatch: ${unsafePathValidation.error}`,
  );

  const absolutePathPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: payload.manifest.files.map((item) => (
        item.path === "lessons/rep_physics_velocity_history_v1.ddn"
          ? { ...item, path: "/lessons/rep_physics_velocity_history_v1.ddn" }
          : item
      )),
    },
  };
  const absolutePathValidation = helper.validateStudioLocalPackagePayload(absolutePathPayload);
  assert(absolutePathValidation.valid === false, "absolute package path should be rejected");
  assert(
    absolutePathValidation.error === "studio_local_package_unsafe_path",
    `absolute path error mismatch: ${absolutePathValidation.error}`,
  );

  const urlPathPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: payload.manifest.files.map((item) => (
        item.path === "lessons/rep_physics_velocity_history_v1.ddn"
          ? { ...item, path: "https://example.test/rep_physics_velocity_history_v1.ddn" }
          : item
      )),
    },
  };
  const urlPathValidation = helper.validateStudioLocalPackagePayload(urlPathPayload);
  assert(urlPathValidation.valid === false, "url package path should be rejected");
  assert(
    urlPathValidation.error === "studio_local_package_unsafe_path",
    `url path error mismatch: ${urlPathValidation.error}`,
  );

  const unsafeLessonRowPathPayload = {
    ...payload,
    lessons: payload.lessons.map((item) => ({
      ...item,
      path: "/lessons/rep_physics_velocity_history_v1.ddn",
    })),
  };
  const unsafeLessonRowPathValidation = helper.validateStudioLocalPackagePayload(unsafeLessonRowPathPayload);
  assert(unsafeLessonRowPathValidation.valid === false, "unsafe lesson row path should be rejected");
  assert(
    unsafeLessonRowPathValidation.error === "studio_local_package_unsafe_path",
    `unsafe lesson row path error mismatch: ${unsafeLessonRowPathValidation.error}`,
  );

  const overlappingStaticPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: payload.manifest.files.map((item) => (
        item.path === "app.js"
          ? { ...item, byte_size: payload.lessons[0].byte_size }
          : item
      )),
    },
    lessons: payload.lessons.map((item) => ({
      ...item,
      path: "app.js",
    })),
  };
  const overlappingStaticValidation = helper.validateStudioLocalPackagePayload(overlappingStaticPayload);
  assert(overlappingStaticValidation.valid === false, "lesson path overlapping static bundle should be rejected");
  assert(
    overlappingStaticValidation.error === "studio_local_package_duplicate_path",
    `static overlap error mismatch: ${overlappingStaticValidation.error}`,
  );

  const extraStaticOverlapPayload = {
    ...payload,
    manifest: {
      ...payload.manifest,
      files: [
        ...payload.manifest.files,
        {
          path: "extras/help.js",
          type: "static",
          byte_size: payload.lessons[0].byte_size,
          mime: "application/javascript; charset=utf-8",
        },
      ],
      file_count: payload.manifest.files.length + 1,
    },
    lessons: payload.lessons.map((item) => ({
      ...item,
      path: "extras/help.js",
    })),
  };
  const extraStaticOverlapValidation = helper.validateStudioLocalPackagePayload(extraStaticOverlapPayload);
  assert(extraStaticOverlapValidation.valid === false, "lesson path overlapping extra static file should be rejected");
  assert(
    extraStaticOverlapValidation.error === "studio_local_package_duplicate_path",
    `extra static overlap error mismatch: ${extraStaticOverlapValidation.error}`,
  );

  const assetPayload = helper.buildStudioLocalPackagePayload({
    lessons: [lesson],
    reports: [report],
    assets: [{ path: "assets/teacher-notes.txt", role: "teacher_notes", byte_size: 7 }],
  });
  const invalidAssetByteSizePayload = {
    ...assetPayload,
    manifest: {
      ...assetPayload.manifest,
      files: assetPayload.manifest.files.map((item) => (
        item.path === "assets/teacher-notes.txt"
          ? { ...item, byte_size: 0 }
          : item
      )),
    },
    assets: assetPayload.assets.map((item) => (
      item.path === "assets/teacher-notes.txt"
        ? { ...item, byte_size: "many" }
        : item
    )),
  };
  const invalidAssetByteSizeValidation = helper.validateStudioLocalPackagePayload(invalidAssetByteSizePayload);
  assert(invalidAssetByteSizeValidation.valid === false, "invalid asset row byte_size should be rejected");
  assert(
    invalidAssetByteSizeValidation.error === "studio_local_package_asset_byte_size_mismatch",
    `invalid asset byte_size error mismatch: ${invalidAssetByteSizeValidation.error}`,
  );
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of ["index.html", "app.js", "styles.css", "screens/browse.js", "screens/run.js", "studio_local_share_package.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }
  const appJs = await fs.readFile(path.join(uiRoot, "app.js"), "utf-8");
  const runJs = await fs.readFile(path.join(uiRoot, "screens", "run.js"), "utf-8");
  const browseJs = await fs.readFile(path.join(uiRoot, "screens", "browse.js"), "utf-8");
  const stylesCss = await fs.readFile(path.join(uiRoot, "styles.css"), "utf-8");
  assert(appJs.includes('setLocalPackageImportStatus("loading"'), "local package loading status update is missing");
  assert(appJs.includes("배포 파일을 확인하는 중입니다"), "local package loading message is missing");
  assert(!runJs.includes("path: `lessons/${lessonId}.ddn`"), "run export must not bypass portable lesson package paths");
  assert(!runJs.includes("path: `reports/${lessonId}.classroom_report.tsv`"), "run export must not bypass portable report package paths");
  assert(browseJs.includes("DDN 교과 실행"), "teacher lesson cards must expose the DDN course surface");
  assert(browseJs.includes("학생 실행") && browseJs.includes("교사용 배포"), "teacher lesson cards must expose classroom delivery flow");
  assert(browseJs.includes("data-course-catalog-summary"), "teacher catalog summary should be mounted by BrowseScreen");
  assert(browseJs.includes("data-course-goals"), "teacher lesson cards must expose learning goals");
  assert(stylesCss.includes('.local-package-import-status[data-state="loading"]'), "local package loading status style is missing");
  assert(stylesCss.includes(".card-course-surface"), "course surface card style is missing");
  assert(stylesCss.includes(".card-course-delivery"), "course delivery card style is missing");
  assert(stylesCss.includes(".card-course-goals"), "course goals card style is missing");
  assert(stylesCss.includes(".course-catalog-summary"), "course catalog summary style is missing");
  await assertLocalPackageValidation(uiRoot);

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR", acceptDownloads: true });
    await context.addInitScript(() => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          async writeText(value) {
            window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ = String(value ?? "");
          },
        },
      });
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      const text = String(msg.text() ?? "");
      if (msg.type() === "error" && !text.includes("Failed to load resource") && !text.includes("[RunScreen.restart] wasm execution failed")) {
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
    await waitVisible(page, "#screen-browse");
    await page.waitForSelector(".lesson-card[data-lesson-id^='rep_']");
    const defaultDevSurfaceState = await page.evaluate(() => ({
      bodyEnabled: document.body.classList.contains("dev-surfaces-enabled"),
      devRootExists: Boolean(document.querySelector("#dev-surface-root")),
      templateExists: Boolean(document.querySelector("#dev-surface-template")),
      advancedBrowseDisplay: getComputedStyle(document.querySelector("#btn-advanced-browse")).display,
      advancedEditorDisplay: getComputedStyle(document.querySelector("#btn-advanced-editor")).display,
      advancedRunDisplay: getComputedStyle(document.querySelector("#btn-advanced-run")).display,
      numericTrackExists: Boolean(document.querySelector("#btn-filter-numeric-track")),
      numericResultsExists: Boolean(document.querySelector("#btn-filter-numeric-track-results")),
      featuredSeedExists: Boolean(document.querySelector("#btn-preset-featured-seed-quick-recent")),
      numericSummaryExists: Boolean(document.querySelector("#btn-copy-numeric-track-result-summary")),
      numericTimelineExists: Boolean(document.querySelector("#btn-toggle-numeric-track-result-timeline")),
      presetLinkExists: Boolean(document.querySelector("#btn-copy-browse-preset-link")),
      qualityExists: Boolean(document.querySelector("#filter-quality")),
      seedScopeExists: Boolean(document.querySelector("#filter-seed-scope")),
      runStatusExists: Boolean(document.querySelector("#filter-run-status")),
      runLaunchExists: Boolean(document.querySelector("#filter-run-launch")),
      warningStatusExists: Boolean(document.querySelector("#filter-warning-status")),
      launchProfileExists: Boolean(document.querySelector("#filter-launch-profile")),
      sortExists: Boolean(document.querySelector("#filter-sort")),
      publicationPrepExists: Boolean(document.querySelector("[data-run-publication-prep-export]")),
      registrySeedExists: Boolean(document.querySelector("[data-run-registry-seed-export]")),
      approvalContinuityExists: Boolean(document.querySelector("[data-run-approval-continuity-export]")),
      benchmarkLtsExists: Boolean(document.querySelector("[data-run-benchmark-lts-export]")),
      educationOperationsExists: Boolean(document.querySelector("[data-run-education-operations-lts-export]")),
      localPackageFileAccept: document.querySelector("#input-local-package-file")?.getAttribute("accept") ?? "",
      visibleLessonIds: Array.from(document.querySelectorAll(".lesson-card")).map((node) => node.dataset.lessonId || ""),
      courseSummaryText: document.querySelector("[data-course-catalog-summary]")?.textContent?.trim() || "",
      courseSurfaceTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-surface]")).map((node) => node.textContent?.trim() || ""),
      courseDeliveryTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-delivery]")).map((node) => node.textContent?.trim() || ""),
      courseGoalTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-goals]")).map((node) => node.textContent?.trim() || ""),
      cardSubjects: Array.from(document.querySelectorAll(".lesson-card .card-meta")).map((node) => node.textContent?.trim() || ""),
    }));
    assert(defaultDevSurfaceState.templateExists === false, "dev surface template should not ship in the default teacher UI");
    assert(defaultDevSurfaceState.bodyEnabled === false, "dev surfaces should not be enabled by default");
    assert(defaultDevSurfaceState.devRootExists === false, "dev surface root should not mount on the default teacher UI");
    assert(defaultDevSurfaceState.advancedBrowseDisplay === "none", `advanced browse button should be hidden by default: ${defaultDevSurfaceState.advancedBrowseDisplay}`);
    assert(defaultDevSurfaceState.advancedEditorDisplay === "none", `advanced editor button should be hidden by default: ${defaultDevSurfaceState.advancedEditorDisplay}`);
    assert(defaultDevSurfaceState.advancedRunDisplay === "none", `advanced run button should be hidden by default: ${defaultDevSurfaceState.advancedRunDisplay}`);
    assert(defaultDevSurfaceState.numericTrackExists === false, "numeric track filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.numericResultsExists === false, "numeric result filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.featuredSeedExists === false, "featured seed shortcut should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.numericSummaryExists === false, "numeric summary export should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.numericTimelineExists === false, "numeric timeline should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.presetLinkExists === false, "preset link copy should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.qualityExists === false, "quality filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.seedScopeExists === false, "seed scope filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.runStatusExists === false, "run status filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.runLaunchExists === false, "run launch filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.warningStatusExists === false, "legacy warning filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.launchProfileExists === false, "launch profile filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.sortExists === false, "sort filter should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.publicationPrepExists === false, "publication prep export should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.registrySeedExists === false, "registry seed export should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.approvalContinuityExists === false, "approval continuity export should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.benchmarkLtsExists === false, "benchmark LTS export should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.educationOperationsExists === false, "education operations export should not exist in the default teacher DOM");
    assert(defaultDevSurfaceState.localPackageFileAccept === ".json,application/json", `local package file accept mismatch: ${defaultDevSurfaceState.localPackageFileAccept}`);
    assert(defaultDevSurfaceState.visibleLessonIds.length > 0, "default teacher catalog should show course lessons");
    assert(
      defaultDevSurfaceState.courseSummaryText.includes(`${defaultDevSurfaceState.visibleLessonIds.length}개 대표 교과`)
        && defaultDevSurfaceState.courseSummaryText.includes("DDN 실행")
        && defaultDevSurfaceState.courseSummaryText.includes("교사용 배포"),
      `course summary mismatch: ${defaultDevSurfaceState.courseSummaryText}`,
    );
    assert(
      defaultDevSurfaceState.visibleLessonIds.every((id) => !String(id).includes("ddonirang") && !String(id).startsWith("rep_cs_")),
      `default teacher catalog leaked internal lessons: ${defaultDevSurfaceState.visibleLessonIds.join(",")}`,
    );
    assert(
      defaultDevSurfaceState.courseSurfaceTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.courseSurfaceTexts.every((text) => text.startsWith("DDN 교과 실행")),
      `course surface text mismatch: ${defaultDevSurfaceState.courseSurfaceTexts.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.courseDeliveryTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.courseDeliveryTexts.every((text) => text.includes("학생 실행") && text.includes("교사용 배포")),
      `course delivery text mismatch: ${defaultDevSurfaceState.courseDeliveryTexts.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.courseGoalTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.courseGoalTexts.every((text) => text.length > 0),
      `course goals missing: ${defaultDevSurfaceState.courseGoalTexts.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.cardSubjects.every((text) => /물리|수학|경제|과학/.test(text)),
      `default teacher catalog should stay on classroom subjects: ${defaultDevSurfaceState.cardSubjects.join("|")}`,
    );
    await page.click(".lesson-card[data-lesson-id^='rep_']");
    const defaultLessonDetail = await page.evaluate(() => ({
      visible: !document.querySelector("#catalog-detail-panel")?.classList?.contains("hidden"),
      text: document.querySelector("#detail-curriculum")?.textContent ?? "",
    }));
    assert(defaultLessonDetail.visible === true, "lesson detail panel should open from the default teacher catalog");
    assert(defaultLessonDetail.text.includes("학습목표"), `detail panel missing learning goals: ${defaultLessonDetail.text}`);
    assert(defaultLessonDetail.text.includes("수업 활동"), `detail panel missing classroom activities: ${defaultLessonDetail.text}`);
    assert(defaultLessonDetail.text.includes("수업 보기"), `detail panel missing required views: ${defaultLessonDetail.text}`);
    await page.waitForSelector(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='student']");
    await page.click(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='student']");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.onboarding_profile === "student");
    const studentStartUi = await page.evaluate(() => {
      const display = (selector) => {
        const node = document.querySelector(selector);
        return node ? getComputedStyle(node).display : "";
      };
      return {
        profile: document.querySelector("#screen-run")?.dataset?.onboardingProfile ?? "",
        sourcePanelDisplay: display(".run-editor-panel"),
        sourceLabelDisplay: display("#screen-run [data-shell-source-label]"),
        saveStatusDisplay: display("#screen-run [data-shell-save-status]"),
        sessionStatusDisplay: display("#screen-run [data-shell-session-status]"),
        presetRailDisplay: display("#screen-run [data-run-preset-rail]"),
        newDisplay: display("#btn-studio-new"),
        openDisplay: display("#btn-ddn-open"),
        saveDisplay: display("#btn-ddn-save"),
        teacherReportDisplay: display("#btn-run-teacher-report-copy"),
        teacherPackageCopyDisplay: display("#btn-run-teacher-package-copy"),
        teacherPackageDownloadDisplay: display("#btn-run-teacher-package-download"),
        inspectorToolsDisplay: display("#run-inspector-tools"),
        bodyText: document.querySelector("#screen-run")?.innerText ?? "",
      };
    });
    assert(studentStartUi.profile === "student", `student profile mismatch: ${studentStartUi.profile}`);
    assert(studentStartUi.sourcePanelDisplay === "none", `student source panel should be hidden: ${studentStartUi.sourcePanelDisplay}`);
    assert(studentStartUi.sourceLabelDisplay === "none", `student source label should be hidden: ${studentStartUi.sourceLabelDisplay}`);
    assert(studentStartUi.saveStatusDisplay === "none", `student save status should be hidden: ${studentStartUi.saveStatusDisplay}`);
    assert(studentStartUi.sessionStatusDisplay === "none", `student session status should be hidden: ${studentStartUi.sessionStatusDisplay}`);
    assert(studentStartUi.presetRailDisplay === "none", `student preset rail should be hidden: ${studentStartUi.presetRailDisplay}`);
    assert(studentStartUi.newDisplay === "none", `student new button should be hidden: ${studentStartUi.newDisplay}`);
    assert(studentStartUi.openDisplay === "none", `student open button should be hidden: ${studentStartUi.openDisplay}`);
    assert(studentStartUi.saveDisplay === "none", `student save button should be hidden: ${studentStartUi.saveDisplay}`);
    assert(studentStartUi.teacherReportDisplay === "none", `student report copy should be hidden: ${studentStartUi.teacherReportDisplay}`);
    assert(studentStartUi.teacherPackageCopyDisplay === "none", `student package copy should be hidden: ${studentStartUi.teacherPackageCopyDisplay}`);
    assert(studentStartUi.teacherPackageDownloadDisplay === "none", `student package download should be hidden: ${studentStartUi.teacherPackageDownloadDisplay}`);
    assert(studentStartUi.inspectorToolsDisplay === "none", `student inspector tools should be hidden: ${studentStartUi.inspectorToolsDisplay}`);
    assert(!studentStartUi.bodyText.includes("교과 원문"), "student mode should not show source text panel label");
    assert(!studentStartUi.bodyText.includes("저장 대기"), "student mode should not show save status");
    assert(!studentStartUi.bodyText.includes("세션 복원됨"), "student mode should not show session restore status");
    assert(!studentStartUi.bodyText.includes("+ 새로 만들기"), "student mode should not show new draft button");
    assert(!studentStartUi.bodyText.includes("배포 복사"), "student mode should not show teacher package copy button");
    await page.click("#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target='browse']");
    await waitVisible(page, "#screen-browse");
    await page.click(".lesson-card[data-lesson-id^='rep_'] .card-launch-btn[data-launch-profile='teacher']");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.onboardingProfile === "teacher");
    await page.click("#run-tab-btn-mirror");
    await page.evaluate(() => {
      const tools = document.querySelector("#run-inspector-tools");
      if (tools) tools.open = true;
    });
    await page.click("#btn-run-teacher-package-copy");
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_EXPORT_ACTION__?.copied === true);
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-teacher-package-download"),
    ]);
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_DOWNLOAD_ACTION__?.downloaded === true);

    const state = await page.evaluate(() => ({
      schema: document.querySelector("[data-run-local-package-export]")?.dataset?.schema ?? "",
      state: document.querySelector("[data-run-local-package-export]")?.dataset?.state ?? "",
      packageId: document.querySelector("[data-run-local-package-export]")?.dataset?.packageId ?? "",
      meta: document.querySelector("[data-run-local-package-meta]")?.textContent?.trim() ?? "",
      metaValue: document.querySelector("[data-run-local-package-meta]")?.dataset?.value ?? "",
      indexText: document.querySelector("[data-run-local-package-text]")?.textContent ?? "",
      saveButtonText: document.querySelector("#btn-run-teacher-package-download")?.textContent?.trim() ?? "",
      copied: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      payload: window.__STUDIO_LOCAL_PACKAGE_EXPORT_ACTION__ ?? null,
      downloadPayload: window.__STUDIO_LOCAL_PACKAGE_DOWNLOAD_ACTION__ ?? null,
    }));
    const copiedPayload = JSON.parse(state.copied);
    const downloadedPath = await download.path();
    const downloadedText = downloadedPath ? await fs.readFile(downloadedPath, "utf-8") : "";
    const downloadedPayload = JSON.parse(downloadedText);

    assert(state.schema === "seamgrim.local_package_export_action.v1", `schema mismatch: ${state.schema}`);
    assert(state.state === "ready", `state mismatch: ${state.state}`);
    assert(state.packageId.startsWith("studio.local."), `package id mismatch: ${state.packageId}`);
    assert(state.meta.includes("교사 시작"), `meta mismatch: ${state.meta}`);
    assert(Number(state.metaValue) >= 5, `meta value mismatch: ${state.metaValue}`);
    assert(state.indexText.startsWith("구분\t경로\t제목\t크기"), "index header mismatch");
    assert(state.indexText.includes("\nlesson\tlessons/"), `lesson index row missing:\n${state.indexText}`);
    assert(state.indexText.includes("\nreport\treports/"), `report index row missing:\n${state.indexText}`);
    assert(state.saveButtonText.startsWith("배포 저장 "), `save button text mismatch: ${state.saveButtonText}`);
    assert(copiedPayload.__종류 === "studio_local_package_payload", "clipboard payload kind mismatch");
    assert(copiedPayload.manifest?.__종류 === "studio_local_package_manifest", "clipboard manifest kind mismatch");
    assert(copiedPayload.manifest?.account_required === false, "account boundary mismatch");
    assert(copiedPayload.manifest?.cloud_sync === false, "cloud boundary mismatch");
    assert(copiedPayload.manifest?.public_registry === false, "registry boundary mismatch");
    assert(state.payload?.schema === "seamgrim.local_package_export_action.v1", "instrumentation schema mismatch");
    assert(state.payload?.copied === true, "instrumentation copied mismatch");
    assert(state.payload?.lesson_count === 1 && state.payload?.report_count === 1, "instrumentation counts mismatch");
    assert(state.payload?.account_required === false && state.payload?.cloud_sync === false && state.payload?.public_registry === false && state.payload?.remote_save === false, "instrumentation boundary mismatch");
    assert(state.payload?.payload_text.trim() === state.copied, "payload text clipboard mismatch");
    assert(state.downloadPayload?.downloaded === true, "download instrumentation mismatch");
    assert(String(state.downloadPayload?.file_name ?? "").endsWith(".json"), "download filename mismatch");
    assert(download.suggestedFilename() === state.downloadPayload.file_name, "suggested filename mismatch");
    assert(downloadedPayload.__종류 === "studio_local_package_payload", "downloaded payload kind mismatch");
    assert(downloadedText.trim() === state.copied, "downloaded payload text mismatch");

    await page.click("#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target='browse']");
    await waitVisible(page, "#screen-browse");
    await page.setInputFiles("#input-local-package-file", downloadedPath);
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true);
    await waitVisible(page, "#screen-run");
    const importState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      rail: window.__SEAMGRIM_RUN_PRESET_RAIL__ ?? null,
      runText: document.querySelector("#run-ddn-preview")?.value ?? "",
      sourceLabel: document.querySelector("[data-shell-source-label]")?.textContent?.trim() ?? "",
      browseImportStatus: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      browseImportStatusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      browseImportStatusRole: document.querySelector("[data-local-package-import-status]")?.getAttribute("role") ?? "",
      browseImportStatusLive: document.querySelector("[data-local-package-import-status]")?.getAttribute("aria-live") ?? "",
      browseImportStatusAtomic: document.querySelector("[data-local-package-import-status]")?.getAttribute("aria-atomic") ?? "",
      classroomMode: document.querySelector("[data-classroom-mode-switch]")?.dataset?.mode ?? "",
      studentPressed: document.querySelector("[data-classroom-mode='student']")?.getAttribute("aria-pressed") ?? "",
      teacherPressed: document.querySelector("[data-classroom-mode='teacher']")?.getAttribute("aria-pressed") ?? "",
      inputRegistry: JSON.parse(localStorage.getItem("seamgrim.input_registry.v0") || "{}"),
    }));
    assert(importState.importAction?.schema === "seamgrim.local_package_import_action.v1", "import action schema mismatch");
    assert(importState.importAction?.package_id === downloadedPayload.manifest.package_id, "import package id mismatch");
    assert(importState.importAction?.lesson_id === downloadedPayload.lessons[0].lesson_id, "import lesson id mismatch");
    assert(importState.importAction?.account_required === false && importState.importAction?.cloud_sync === false && importState.importAction?.public_registry === false, "import boundary mismatch");
    assert(importState.rail?.launch_kind === "local_package_import", `import launch kind mismatch: ${importState.rail?.launch_kind}`);
    assert(importState.rail?.launch_label === "배포 열기", `import launch label mismatch: ${importState.rail?.launch_label}`);
    assert(importState.rail?.onboarding_profile === "student", `import onboarding mismatch: ${importState.rail?.onboarding_profile}`);
    assert(importState.classroomMode === "student" && importState.studentPressed === "true" && importState.teacherPressed === "false", "import should open in student mode");
    assert(importState.runText.trim() === String(downloadedPayload.lessons[0].source_text ?? "").trim(), "imported run source mismatch");
    assert(importState.sourceLabel.includes(downloadedPayload.lessons[0].title), `imported source label mismatch: ${importState.sourceLabel}`);
    assert(importState.browseImportStatusState === "ok", `import status state mismatch: ${importState.browseImportStatusState}`);
    assert(importState.browseImportStatus.includes(downloadedPayload.manifest.title), `import status text mismatch: ${importState.browseImportStatus}`);
    assert(importState.browseImportStatusRole === "status", `import status role mismatch: ${importState.browseImportStatusRole}`);
    assert(importState.browseImportStatusLive === "polite", `import status aria-live mismatch: ${importState.browseImportStatusLive}`);
    assert(importState.browseImportStatusAtomic === "true", `import status aria-atomic mismatch: ${importState.browseImportStatusAtomic}`);

    await page.click("#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target='browse']");
    await waitVisible(page, "#screen-browse");
    const unsafeIdPayload = {
      ...downloadedPayload,
      manifest: {
        ...downloadedPayload.manifest,
        package_id: "teacher/package:id with spaces",
      },
      lessons: downloadedPayload.lessons.map((item) => ({
        ...item,
        lesson_id: "lesson/id:with spaces",
      })),
    };
    await page.setInputFiles("#input-local-package-file", {
      name: "unsafe-id-teacher-package.json",
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(unsafeIdPayload), "utf-8"),
    });
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.file_name === "unsafe-id-teacher-package.json");
    await waitVisible(page, "#screen-run");
    const unsafeIdImportState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      inputRegistry: JSON.parse(localStorage.getItem("seamgrim.input_registry.v0") || "{}"),
    }));
    assert(unsafeIdImportState.importAction?.imported === true, "unsafe display id package should still import");
    assert(
      unsafeIdImportState.importAction?.package_id === "teacher/package:id with spaces",
      `unsafe id import should keep original package id for traceability: ${unsafeIdImportState.importAction?.package_id}`,
    );
    assert(
      unsafeIdImportState.importAction?.lesson_id === "lesson/id:with spaces",
      `unsafe id import should keep original lesson id for traceability: ${unsafeIdImportState.importAction?.lesson_id}`,
    );
    assert(
      unsafeIdImportState.inputRegistry?.inputs?.selected_id === "local_package:teacher_package_id_with_spaces:lesson_id_with_spaces",
      `unsafe id source should be normalized: ${unsafeIdImportState.inputRegistry?.inputs?.selected_id}`,
    );

    await page.click("#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target='browse']");
    await waitVisible(page, "#screen-browse");
    const brokenPayload = {
      ...downloadedPayload,
      manifest: {
        ...downloadedPayload.manifest,
        lesson_count: 2,
      },
    };
    await page.setInputFiles("#input-local-package-file", {
      name: "broken-teacher-package.json",
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(brokenPayload), "utf-8"),
    });
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.file_name === "broken-teacher-package.json");
    const brokenImportState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      currentScreen: document.querySelector("#screen-browse")?.classList.contains("hidden") ? "run" : "browse",
      importStatus: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      importStatusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      importStatusRole: document.querySelector("[data-local-package-import-status]")?.getAttribute("role") ?? "",
      importStatusLive: document.querySelector("[data-local-package-import-status]")?.getAttribute("aria-live") ?? "",
      importStatusAtomic: document.querySelector("[data-local-package-import-status]")?.getAttribute("aria-atomic") ?? "",
      openPackageButtonText: document.querySelector("#btn-open-local-package")?.textContent?.trim() ?? "",
      openPackageButtonDisabled: document.querySelector("#btn-open-local-package")?.disabled ?? false,
    }));
    assert(brokenImportState.importAction?.schema === "seamgrim.local_package_import_action.v1", "broken import action schema mismatch");
    assert(brokenImportState.importAction?.imported === false, "broken package should be rejected");
    assert(brokenImportState.importAction?.error === "studio_local_package_lesson_count_mismatch", `broken import error mismatch: ${brokenImportState.importAction?.error}`);
    assert(
      brokenImportState.importAction?.error_message === "배포 묶음의 교과 개수가 manifest와 맞지 않습니다.",
      `broken import message mismatch: ${brokenImportState.importAction?.error_message}`,
    );
    assert(brokenImportState.importStatusState === "error", `broken import status state mismatch: ${brokenImportState.importStatusState}`);
    assert(
      brokenImportState.importStatus.includes("배포 묶음의 교과 개수가 manifest와 맞지 않습니다."),
      `broken import status text mismatch: ${brokenImportState.importStatus}`,
    );
    assert(brokenImportState.importStatusRole === "status", `broken import status role mismatch: ${brokenImportState.importStatusRole}`);
    assert(brokenImportState.importStatusLive === "polite", `broken import aria-live mismatch: ${brokenImportState.importStatusLive}`);
    assert(brokenImportState.importStatusAtomic === "true", `broken import aria-atomic mismatch: ${brokenImportState.importStatusAtomic}`);
    assert(brokenImportState.openPackageButtonText === "배포 열기", `open package button text mismatch: ${brokenImportState.openPackageButtonText}`);
    assert(brokenImportState.openPackageButtonDisabled === false, "open package button should be re-enabled after failed import");
    assert(brokenImportState.currentScreen === "browse", `broken import should stay on browse: ${brokenImportState.currentScreen}`);

    await page.setInputFiles("#input-local-package-file", {
      name: "invalid-json-teacher-package.json",
      mimeType: "application/json",
      buffer: Buffer.from("{", "utf-8"),
    });
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.file_name === "invalid-json-teacher-package.json");
    const invalidJsonImportState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      currentScreen: document.querySelector("#screen-browse")?.classList.contains("hidden") ? "run" : "browse",
      importStatus: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      importStatusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      importStatusRole: document.querySelector("[data-local-package-import-status]")?.getAttribute("role") ?? "",
      importStatusLive: document.querySelector("[data-local-package-import-status]")?.getAttribute("aria-live") ?? "",
      importStatusAtomic: document.querySelector("[data-local-package-import-status]")?.getAttribute("aria-atomic") ?? "",
      openPackageButtonText: document.querySelector("#btn-open-local-package")?.textContent?.trim() ?? "",
      openPackageButtonDisabled: document.querySelector("#btn-open-local-package")?.disabled ?? false,
    }));
    assert(invalidJsonImportState.importAction?.schema === "seamgrim.local_package_import_action.v1", "invalid json import action schema mismatch");
    assert(invalidJsonImportState.importAction?.imported === false, "invalid json package should be rejected");
    assert(
      invalidJsonImportState.importAction?.error === "studio_local_package_invalid_json",
      `invalid json error mismatch: ${invalidJsonImportState.importAction?.error}`,
    );
    assert(
      invalidJsonImportState.importAction?.error_message === "배포 묶음 JSON을 읽을 수 없습니다.",
      `invalid json message mismatch: ${invalidJsonImportState.importAction?.error_message}`,
    );
    assert(invalidJsonImportState.importStatusState === "error", `invalid json status state mismatch: ${invalidJsonImportState.importStatusState}`);
    assert(
      invalidJsonImportState.importStatus.includes("배포 묶음 JSON을 읽을 수 없습니다."),
      `invalid json status text mismatch: ${invalidJsonImportState.importStatus}`,
    );
    assert(invalidJsonImportState.importStatusRole === "status", `invalid json status role mismatch: ${invalidJsonImportState.importStatusRole}`);
    assert(invalidJsonImportState.importStatusLive === "polite", `invalid json aria-live mismatch: ${invalidJsonImportState.importStatusLive}`);
    assert(invalidJsonImportState.importStatusAtomic === "true", `invalid json aria-atomic mismatch: ${invalidJsonImportState.importStatusAtomic}`);
    assert(invalidJsonImportState.openPackageButtonText === "배포 열기", `invalid json open button text mismatch: ${invalidJsonImportState.openPackageButtonText}`);
    assert(invalidJsonImportState.openPackageButtonDisabled === false, "open package button should be re-enabled after invalid json import");
    assert(invalidJsonImportState.currentScreen === "browse", `invalid json import should stay on browse: ${invalidJsonImportState.currentScreen}`);

    if (failures.length > 0) throw new Error(failures.join("\n"));
    await context.close();
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
