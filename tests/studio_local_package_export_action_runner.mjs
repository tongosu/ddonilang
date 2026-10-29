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
    required_views: ["graph", "table"],
    goals: ["속도 변화를 읽는다"],
    missions: ["속도 기록 표를 보고 증가량을 찾기"],
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
    sessionLabel: "2학년 3반 4교시",
    lessons: [lesson],
    reports: [report],
  });
  const payload = helper.buildStudioLocalPackagePayload({ manifest, lessons: [lesson], reports: [report] });
  const validation = helper.validateStudioLocalPackagePayload(payload);
  assert(validation.valid === true, `valid package rejected: ${validation.error}`);
  assert(validation.lesson_count === 1 && validation.report_count === 1, "valid package counts mismatch");
  const imported = helper.importStudioLocalPackagePayload(payload);
  assert(imported.lesson_count === 1 && imported.report_count === 1, "imported package counts mismatch");
  assert(payload.delivery_mode === "studio_json_import", `payload delivery mode mismatch: ${payload.delivery_mode}`);
  assert(payload.open_with === "seamgrim_studio_local_package_import", `payload open_with mismatch: ${payload.open_with}`);
  assert(payload.student_entry_label === "배포 열기", `payload student entry mismatch: ${payload.student_entry_label}`);
  assert(
    Array.isArray(payload.student_instructions)
      && payload.student_instructions.some((item) => String(item).includes("수업: 속도 기록"))
      && payload.student_instructions.some((item) => String(item).includes("수업 코드: rep_physics_velocity_history_v1"))
      && payload.student_instructions.some((item) => String(item).includes("목표: 속도 변화를 읽는다"))
      && payload.student_instructions.some((item) => String(item).includes("오늘 활동: 속도 기록 표"))
      && payload.student_instructions.some((item) => String(item).includes("붙여넣기 열기"))
      && payload.student_instructions.some((item) => String(item).includes("받은 수업 실행"))
      && payload.student_instructions.some((item) => String(item).includes("결과 확인: 그래프, 표"))
      && payload.student_instructions.some((item) => String(item).includes("결과 복사"))
      && payload.student_instructions.some((item) => String(item).includes("배포 코드: studio.local.validation")),
    `payload student instructions missing: ${JSON.stringify(payload.student_instructions)}`,
  );
  assert(manifest.delivery_mode === "studio_json_import", `manifest delivery mode mismatch: ${manifest.delivery_mode}`);
  assert(manifest.open_with === "seamgrim_studio_local_package_import", `manifest open_with mismatch: ${manifest.open_with}`);
  assert(manifest.student_entry_label === "배포 열기", `manifest student entry mismatch: ${manifest.student_entry_label}`);
  assert(manifest.session_label === "2학년 3반 4교시", `manifest session label mismatch: ${manifest.session_label}`);
  assert(
    Array.isArray(manifest.student_instructions)
      && manifest.student_instructions.some((item) => String(item).includes("차시: 2학년 3반 4교시"))
      && manifest.student_instructions.some((item) => String(item).includes("수업: 속도 기록"))
      && manifest.student_instructions.some((item) => String(item).includes("수업 코드: rep_physics_velocity_history_v1"))
      && manifest.student_instructions.some((item) => String(item).includes("목표: 속도 변화를 읽는다"))
      && manifest.student_instructions.some((item) => String(item).includes("오늘 활동: 속도 기록 표"))
      && manifest.student_instructions.some((item) => String(item).includes("붙여넣기 열기"))
      && manifest.student_instructions.some((item) => String(item).includes("결과 확인: 그래프, 표"))
      && manifest.student_instructions.some((item) => String(item).includes("결과 복사"))
      && manifest.student_instructions.some((item) => String(item).includes("배포 코드: studio.local.validation")),
    `manifest student instructions missing: ${JSON.stringify(manifest.student_instructions)}`,
  );
  assert(
    Array.isArray(manifest.materials_summary)
      && manifest.materials_summary.some((item) => String(item).includes("교과 1개") && String(item).includes("속도 기록"))
      && manifest.materials_summary.some((item) => String(item).includes("교사용 자료 1개") && String(item).includes("속도 기록 리포트")),
    `manifest materials summary missing: ${JSON.stringify(manifest.materials_summary)}`,
  );
  assert(
    Array.isArray(manifest.student_materials_summary)
      && manifest.student_materials_summary.some((item) => String(item).includes("교과 1개") && String(item).includes("속도 기록"))
      && !manifest.student_materials_summary.some((item) => String(item).includes("교사용 자료")),
    `manifest student materials summary mismatch: ${JSON.stringify(manifest.student_materials_summary)}`,
  );
  assert(
    Array.isArray(payload.student_materials_summary)
      && payload.student_materials_summary.some((item) => String(item).includes("교과 1개") && String(item).includes("속도 기록"))
      && !payload.student_materials_summary.some((item) => String(item).includes("교사용 자료")),
    `payload student materials summary mismatch: ${JSON.stringify(payload.student_materials_summary)}`,
  );

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
  for (const rel of ["index.html", "app.js", "styles.css", "dev_surfaces.css", "dev_surfaces.js", "screens/browse.js", "screens/run.js", "studio_local_share_package.js", "preview_payload_loader.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }
  const appJs = await fs.readFile(path.join(uiRoot, "app.js"), "utf-8");
  const indexHtml = await fs.readFile(path.join(uiRoot, "index.html"), "utf-8");
  const runJs = await fs.readFile(path.join(uiRoot, "screens", "run.js"), "utf-8");
  const browseJs = await fs.readFile(path.join(uiRoot, "screens", "browse.js"), "utf-8");
  const previewPayloadLoaderJs = await fs.readFile(path.join(uiRoot, "preview_payload_loader.js"), "utf-8");
  const stylesCss = await fs.readFile(path.join(uiRoot, "styles.css"), "utf-8");
  const devSurfacesCss = await fs.readFile(path.join(uiRoot, "dev_surfaces.css"), "utf-8");
  const devSurfacesJs = await fs.readFile(path.join(uiRoot, "dev_surfaces.js"), "utf-8");
  const lessonsRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "lessons");
  const lessonIndex = JSON.parse(await fs.readFile(path.join(lessonsRoot, "index.json"), "utf-8"));
  const activeAllowlist = JSON.parse(await fs.readFile(path.join(lessonsRoot, "active_allowlist.detjson"), "utf-8"));
  assert(appJs.includes('setLocalPackageImportStatus("loading"'), "local package loading status update is missing");
  assert(appJs.includes("배포 파일을 확인하는 중입니다"), "local package loading message is missing");
  assert(!indexHtml.includes('id="advanced-menu"'), "advanced menu should not ship in the default teacher HTML");
  assert(appJs.includes("function ensureAdvancedMenuRoot()"), "advanced menu should be created only through the dev opt-in path");
  assert(!appJs.includes('from "./dev_surfaces.js"'), "dev surfaces must not be statically imported by the teacher app");
  assert(appJs.includes('import("./dev_surfaces.js")'), "dev surfaces must stay behind the dev opt-in dynamic import");
  assert(!stylesCss.includes(".education-publication-pack"), "education publication dev panel CSS should not ship in the default teacher stylesheet");
  assert(devSurfacesCss.includes(".education-publication-pack"), "education publication dev panel CSS should be in dev_surfaces.css");
  assert(devSurfacesCss.includes(".question-card-smoke"), "question card dev panel CSS should be in dev_surfaces.css");
  assert(devSurfacesJs.includes("data-dev-surfaces-css"), "dev surfaces should load their stylesheet only on opt-in");
  assert(!runJs.includes("path: `lessons/${lessonId}.ddn`"), "run export must not bypass portable lesson package paths");
  assert(!runJs.includes("path: `reports/${lessonId}.classroom_report.tsv`"), "run export must not bypass portable report package paths");
  assert(browseJs.includes("DDN 교과 실행"), "teacher lesson cards must expose the DDN course surface");
  assert(browseJs.includes("학생 실행") && browseJs.includes("교사용 배포"), "teacher lesson cards must expose classroom delivery flow");
  assert(browseJs.includes("data-course-catalog-summary"), "teacher catalog summary should be mounted by BrowseScreen");
  assert(browseJs.includes("data-course-goals"), "teacher lesson cards must expose learning goals");
  assert(browseJs.includes("data-course-missions"), "teacher lesson cards must expose classroom activities");
  assert(browseJs.includes("pasted-teacher-package.json"), "browse screen must support pasted local package JSON import");
  assert(browseJs.includes("updateLocalPackagePastePreview"), "browse screen must preview pasted local package JSON before import");
  assert(browseJs.includes("buildCoursePackageExportModel"), "browse screen must build a representative course package");
  assert(indexHtml.includes('id="btn-copy-course-package"'), "course package copy button is missing");
  assert(indexHtml.includes('id="btn-download-course-package"'), "course package download button is missing");
  assert(indexHtml.includes('data-course-package-export-status'), "course package export status is missing");
  assert(browseJs.includes("handleDownloadCoursePackage"), "browse screen must download representative course package");
  assert(stylesCss.includes('.local-package-import-status[data-state="loading"]'), "local package loading status style is missing");
  assert(stylesCss.includes(".course-package-export-status"), "course package export status style is missing");
  assert(indexHtml.includes('id="btn-paste-local-package"'), "local package paste button is missing");
  assert(indexHtml.includes('id="local-package-paste-text"'), "local package paste textarea is missing");
  assert(indexHtml.includes('data-local-package-paste-preview'), "local package paste preview is missing");
  assert(stylesCss.includes(".local-package-paste-panel"), "local package paste panel style is missing");
  assert(stylesCss.includes(".local-package-paste-preview"), "local package paste preview style is missing");
  assert(stylesCss.includes('.local-package-paste-preview[data-state="error"]'), "local package paste error preview style is missing");
  assert(indexHtml.includes('data-run-package-lesson-switch'), "run screen package lesson switch is missing");
  assert(runJs.includes("syncPackageLessonSwitcher"), "run screen must sync package lesson switcher");
  assert(appJs.includes("imported_lesson_ids"), "local package import should register all package lessons");
  assert(stylesCss.includes(".card-course-surface"), "course surface card style is missing");
  assert(stylesCss.includes(".card-course-result"), "course result card style is missing");
  assert(stylesCss.includes(".card-course-delivery"), "course delivery card style is missing");
  assert(stylesCss.includes(".card-course-goals"), "course goals card style is missing");
  assert(stylesCss.includes(".card-course-missions"), "course missions card style is missing");
  assert(stylesCss.includes(".detail-course-flow"), "detail course flow style is missing");
  assert(stylesCss.includes(".detail-course-sequence"), "detail course sequence style is missing");
  assert(stylesCss.includes(".run-package-lesson-switch"), "package lesson switch style is missing");
  assert(stylesCss.includes(".course-catalog-summary"), "course catalog summary style is missing");
  assert(stylesCss.includes(".course-subject-shortcut"), "course subject shortcut style is missing");
  assert(previewPayloadLoaderJs.includes('PROJECT_PREFIX = "solutions/seamgrim_ui_mvp/"'), "preview loader must prefer project-prefixed static lesson assets");
  const lessonRowsById = new Map((lessonIndex.lessons ?? []).map((lesson) => [String(lesson?.id ?? ""), lesson]));
  const activeIds = Array.isArray(activeAllowlist.lesson_ids) ? activeAllowlist.lesson_ids : [];
  assert(activeIds.length === 15, `active allowlist count mismatch: ${activeIds.length}`);
  for (const lessonId of activeIds) {
    const lesson = lessonRowsById.get(String(lessonId));
    assert(lesson, `active lesson missing from lesson index: ${lessonId}`);
    assert(Array.isArray(lesson.goals) && lesson.goals.length >= 2, `active lesson missing teacher goals: ${lessonId}`);
    assert(Array.isArray(lesson.missions) && lesson.missions.length >= 2, `active lesson missing classroom missions: ${lessonId}`);
    assert(Array.isArray(lesson.required_views) && lesson.required_views.length > 0, `active lesson missing result views: ${lessonId}`);
  }
  await assertLocalPackageValidation(uiRoot);
  const localPackageHelper = await import(pathToFileURL(path.join(uiRoot, "studio_local_share_package.js")).href);

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
    const apiInventoryRequests = [];
    page.on("request", (req) => {
      const pathname = new URL(req.url()).pathname;
      if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") {
        apiInventoryRequests.push(req.url());
      }
    });
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

    await page.addInitScript(() => {
      window.localStorage.setItem("seamgrim.dev_surfaces", "1");
    });
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    await page.waitForSelector(".lesson-card[data-lesson-id^='rep_']");
    await page.waitForSelector("[data-detail-ddn-preview][data-detail-ddn-preview-status='loaded']");
    assert(apiInventoryRequests.length === 0, `default teacher catalog should not probe API inventory: ${apiInventoryRequests.join(",")}`);
    const defaultDevSurfaceState = await page.evaluate(() => ({
      bodyEnabled: document.body.classList.contains("dev-surfaces-enabled"),
      devRootExists: Boolean(document.querySelector("#dev-surface-root")),
      templateExists: Boolean(document.querySelector("#dev-surface-template")),
      advancedMenuExists: Boolean(document.querySelector("#advanced-menu")),
      browseStatusRailExists: Boolean(document.querySelector("#screen-browse [data-shell-status-rail]")),
      shellStatusRailDisplays: Array.from(document.querySelectorAll("[data-shell-status-rail]")).map((node) => getComputedStyle(node).display),
      studioSourceLabelDisplay: getComputedStyle(document.querySelector("#studio-source-label")).display,
      runLocalSaveStatusDisplay: getComputedStyle(document.querySelector("#run-local-save-status")).display,
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
      devPanelRootsPresent: [
        ".classroom-operations-panel-preview",
        ".lesson-publication-review-surface",
        ".operations-preview-stage-closure",
        ".publication-artifact-dry-run",
        ".education-publication-pack",
        ".education-operations-lts",
        ".question-card-smoke",
        ".question-card-validation",
        ".question-card-dev-assist",
        ".question-card-author-tool-share",
        ".question-card-workflow-hardening",
        ".free-lab-experiment-report",
      ].filter((selector) => Boolean(document.querySelector(selector))),
      localPackageFileAccept: document.querySelector("#input-local-package-file")?.getAttribute("accept") ?? "",
      localPackagePasteButtonText: document.querySelector("#btn-paste-local-package")?.textContent?.trim() ?? "",
      localPackagePasteButtonControls: document.querySelector("#btn-paste-local-package")?.getAttribute("aria-controls") ?? "",
      localPackagePasteButtonExpanded: document.querySelector("#btn-paste-local-package")?.getAttribute("aria-expanded") ?? "",
      localPackagePastePanelDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-panel]")).display,
      localPackagePastePanelAriaHidden: document.querySelector("[data-local-package-paste-panel]")?.getAttribute("aria-hidden") ?? "",
      localPackagePasteTextareaPlaceholder: document.querySelector("#local-package-paste-text")?.getAttribute("placeholder") ?? "",
      localPackagePastePreviewDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-preview]")).display,
      localPackagePastePreviewState: document.querySelector("[data-local-package-paste-preview]")?.dataset?.state ?? "",
      localPackagePastePreviewRole: document.querySelector("[data-local-package-paste-preview]")?.getAttribute("role") ?? "",
      localPackagePastePreviewLive: document.querySelector("[data-local-package-paste-preview]")?.getAttribute("aria-live") ?? "",
      coursePackageCopyButtonText: document.querySelector("#btn-copy-course-package")?.textContent?.trim() ?? "",
      coursePackageDownloadButtonText: document.querySelector("#btn-download-course-package")?.textContent?.trim() ?? "",
      coursePackageExportStatusDisplay: getComputedStyle(document.querySelector("[data-course-package-export-status]")).display,
      coursePackageExportStatusState: document.querySelector("[data-course-package-export-status]")?.dataset?.state ?? "",
      coursePackageExportStatusRole: document.querySelector("[data-course-package-export-status]")?.getAttribute("role") ?? "",
      coursePackageExportStatusLive: document.querySelector("[data-course-package-export-status]")?.getAttribute("aria-live") ?? "",
      mainShellTabTexts: Array.from(document.querySelectorAll("#screen-browse .main-shell-tab")).map((node) => node.textContent?.trim() || ""),
      browseTabTexts: Array.from(document.querySelectorAll("#screen-browse .browse-tab[data-tab]")).map((node) => node.textContent?.trim() || ""),
      createButtonText: document.querySelector("#btn-create")?.textContent?.trim() || "",
      queryPlaceholder: document.querySelector("#filter-query")?.getAttribute("placeholder") ?? "",
      catalogBodyWidth: Math.round(document.querySelector(".catalog-body")?.getBoundingClientRect?.().width ?? 0),
      catalogSummaryRect: (() => {
        const rect = document.querySelector("[data-course-catalog-summary]")?.getBoundingClientRect?.();
        return rect ? { x: Math.round(rect.x), width: Math.round(rect.width) } : null;
      })(),
      cardGridRect: (() => {
        const rect = document.querySelector("#lesson-card-grid")?.getBoundingClientRect?.();
        return rect ? { x: Math.round(rect.x), width: Math.round(rect.width) } : null;
      })(),
      visibleLessonIds: Array.from(document.querySelectorAll(".lesson-card")).map((node) => node.dataset.lessonId || ""),
      cardTagNames: Array.from(document.querySelectorAll(".lesson-card")).map((node) => node.tagName),
      cardRoles: Array.from(document.querySelectorAll(".lesson-card")).map((node) => node.getAttribute("role") || ""),
      cardTabIndexes: Array.from(document.querySelectorAll(".lesson-card")).map((node) => node.getAttribute("tabindex") || ""),
      studentLaunchButtonTexts: Array.from(document.querySelectorAll(".lesson-card .card-launch-btn[data-launch-profile='student']")).map((node) => node.textContent?.trim() || ""),
      teacherLaunchButtonTexts: Array.from(document.querySelectorAll(".lesson-card .card-launch-btn[data-launch-profile='teacher']")).map((node) => node.textContent?.trim() || ""),
      courseSequenceTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-sequence]")).map((node) => node.textContent?.trim() || ""),
      courseSummaryText: document.querySelector("[data-course-catalog-summary]")?.textContent?.trim() || "",
      courseSummaryTotal: document.querySelector("[data-course-catalog-summary]")?.getAttribute("data-course-catalog-total") || "",
      courseSummaryVisible: document.querySelector("[data-course-catalog-summary]")?.getAttribute("data-course-catalog-visible") || "",
      courseSubjectShortcutTexts: Array.from(document.querySelectorAll("[data-course-subject-filter]")).map((node) => node.textContent?.trim() || ""),
      courseSubjectShortcutFilters: Array.from(document.querySelectorAll("[data-course-subject-filter]")).map((node) => node.getAttribute("data-course-subject-filter") || ""),
      velocityCardText: document.querySelector(".lesson-card[data-lesson-id='rep_physics_velocity_history_v1']")?.textContent?.trim() || "",
      courseSurfaceTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-surface]")).map((node) => node.textContent?.trim() || ""),
      courseResultTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-result]")).map((node) => node.textContent?.trim() || ""),
      courseDeliveryTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-delivery]")).map((node) => node.textContent?.trim() || ""),
      courseReadinessTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-readiness]")).map((node) => node.textContent?.trim() || ""),
      courseActivityTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-activity]")).map((node) => node.textContent?.trim() || ""),
      courseGoalTexts: Array.from(document.querySelectorAll(".lesson-card [data-course-goals]")).map((node) => node.textContent?.trim() || ""),
      cardSubjects: Array.from(document.querySelectorAll(".lesson-card .card-meta")).map((node) => node.textContent?.trim() || ""),
      initialDetailVisible: !document.querySelector("#catalog-detail-panel")?.classList?.contains("hidden"),
      initialDetailTitle: document.querySelector("#detail-title")?.textContent?.trim() || "",
      initialDetailText: document.querySelector("#detail-curriculum")?.textContent?.trim() || "",
      initialDetailCourseSequenceText: document.querySelector("[data-detail-course-sequence]")?.textContent?.trim() || "",
      initialDdnPreviewStatus: document.querySelector("[data-detail-ddn-preview]")?.getAttribute("data-detail-ddn-preview-status") || "",
      initialDdnPreviewText: document.querySelector("[data-detail-ddn-preview-code]")?.textContent?.trim() || "",
      initialDdnPreviewPath: document.querySelector("[data-detail-ddn-preview-path]")?.textContent?.trim() || "",
      initialRunReadinessText: document.querySelector("[data-detail-run-readiness]")?.textContent?.trim() || "",
      selectedLessonIds: Array.from(document.querySelectorAll(".lesson-card-selected")).map((node) => node.dataset.lessonId || ""),
      selectedAriaPressed: Array.from(document.querySelectorAll(".lesson-card-selected")).map((node) => node.getAttribute("aria-pressed") || ""),
      visibleText: document.body.innerText || "",
    }));
    assert(defaultDevSurfaceState.templateExists === false, "dev surface template should not ship in the default teacher UI");
    assert(defaultDevSurfaceState.bodyEnabled === false, "dev surfaces should not be enabled by default or stale localStorage");
    assert(defaultDevSurfaceState.devRootExists === false, "dev surface root should not mount on the default teacher UI");
    assert(defaultDevSurfaceState.advancedMenuExists === false, "advanced menu should not mount on the default teacher UI");
    assert(defaultDevSurfaceState.browseStatusRailExists === false, "browse teacher UI should not expose draft/session status rail");
    assert(
      defaultDevSurfaceState.shellStatusRailDisplays.every((display) => display === "none"),
      `shell status rails should be hidden from the default teacher UI: ${defaultDevSurfaceState.shellStatusRailDisplays.join(",")}`,
    );
    assert(defaultDevSurfaceState.studioSourceLabelDisplay === "none", `source status should be hidden by default: ${defaultDevSurfaceState.studioSourceLabelDisplay}`);
    assert(defaultDevSurfaceState.runLocalSaveStatusDisplay === "none", `save status should be hidden by default: ${defaultDevSurfaceState.runLocalSaveStatusDisplay}`);
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
    assert(
      defaultDevSurfaceState.devPanelRootsPresent.length === 0,
      `dev panel roots should not exist in the default teacher DOM: ${defaultDevSurfaceState.devPanelRootsPresent.join(",")}`,
    );
    ["준비 중", "게시 이력", "검토 요청", "Smoke 검증", "A 실시간 입력", "새 작업", "저장 대기", "세션 대기"].forEach((text) => {
      assert(
        !defaultDevSurfaceState.visibleText.includes(text),
        `default teacher UI should not expose dev menu text: ${text}`,
      );
    });
    const devPage = await context.newPage();
    await devPage.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await waitVisible(devPage, "#screen-browse");
    await devPage.waitForSelector("link[data-dev-surfaces-css]", { state: "attached" });
    await devPage.waitForSelector("#dev-surface-root");
    const devSurfaceOptInState = await devPage.evaluate(() => ({
      bodyEnabled: document.body.classList.contains("dev-surfaces-enabled"),
      stylesheetHref: document.querySelector("link[data-dev-surfaces-css]")?.getAttribute("href") ?? "",
      devRootExists: Boolean(document.querySelector("#dev-surface-root")),
      advancedMenuExists: Boolean(document.querySelector("#advanced-menu")),
      advancedBrowseDisplay: getComputedStyle(document.querySelector("#btn-advanced-browse")).display,
      devPanelRootsPresent: [
        ".education-publication-pack",
        ".education-operations-lts",
        ".question-card-smoke",
        ".question-card-validation",
        ".question-card-dev-assist",
        ".question-card-author-tool-share",
        ".question-card-workflow-hardening",
      ].filter((selector) => Boolean(document.querySelector(selector))),
    }));
    await devPage.close();
    assert(devSurfaceOptInState.bodyEnabled === true, "dev surfaces should be enabled only by explicit opt-in");
    assert(
      devSurfaceOptInState.stylesheetHref.endsWith("dev_surfaces.css"),
      `dev surface stylesheet should load only on opt-in: ${devSurfaceOptInState.stylesheetHref}`,
    );
    assert(devSurfaceOptInState.devRootExists === true, "dev surface root should mount on opt-in");
    assert(devSurfaceOptInState.advancedMenuExists === true, "advanced menu should mount on opt-in");
    assert(devSurfaceOptInState.advancedBrowseDisplay !== "none", `advanced browse button should be visible on opt-in: ${devSurfaceOptInState.advancedBrowseDisplay}`);
    assert(
      devSurfaceOptInState.devPanelRootsPresent.length === 7,
      `expected dev panel roots on opt-in: ${devSurfaceOptInState.devPanelRootsPresent.join(",")}`,
    );
    assert(defaultDevSurfaceState.localPackageFileAccept === ".json,application/json", `local package file accept mismatch: ${defaultDevSurfaceState.localPackageFileAccept}`);
    assert(defaultDevSurfaceState.localPackagePasteButtonText === "붙여넣기 열기", `local package paste button mismatch: ${defaultDevSurfaceState.localPackagePasteButtonText}`);
    assert(defaultDevSurfaceState.localPackagePasteButtonControls === "local-package-paste-panel", `local package paste controls mismatch: ${defaultDevSurfaceState.localPackagePasteButtonControls}`);
    assert(defaultDevSurfaceState.localPackagePasteButtonExpanded === "false", `local package paste expanded initial mismatch: ${defaultDevSurfaceState.localPackagePasteButtonExpanded}`);
    assert(defaultDevSurfaceState.localPackagePastePanelDisplay === "none", `local package paste panel should start hidden: ${defaultDevSurfaceState.localPackagePastePanelDisplay}`);
    assert(defaultDevSurfaceState.localPackagePastePanelAriaHidden === "true", `local package paste panel aria-hidden initial mismatch: ${defaultDevSurfaceState.localPackagePastePanelAriaHidden}`);
    assert(defaultDevSurfaceState.localPackagePasteTextareaPlaceholder.includes("배포 JSON"), `local package paste placeholder mismatch: ${defaultDevSurfaceState.localPackagePasteTextareaPlaceholder}`);
    assert(defaultDevSurfaceState.localPackagePastePreviewDisplay === "none", `local package paste preview should start hidden: ${defaultDevSurfaceState.localPackagePastePreviewDisplay}`);
    assert(defaultDevSurfaceState.localPackagePastePreviewState === "idle", `local package paste preview state mismatch: ${defaultDevSurfaceState.localPackagePastePreviewState}`);
    assert(defaultDevSurfaceState.localPackagePastePreviewRole === "status", `local package paste preview role mismatch: ${defaultDevSurfaceState.localPackagePastePreviewRole}`);
    assert(defaultDevSurfaceState.localPackagePastePreviewLive === "polite", `local package paste preview aria-live mismatch: ${defaultDevSurfaceState.localPackagePastePreviewLive}`);
    assert(defaultDevSurfaceState.coursePackageCopyButtonText === "대표 묶음 복사", `course package copy button mismatch: ${defaultDevSurfaceState.coursePackageCopyButtonText}`);
    assert(defaultDevSurfaceState.coursePackageDownloadButtonText === "대표 묶음 저장", `course package download button mismatch: ${defaultDevSurfaceState.coursePackageDownloadButtonText}`);
    assert(defaultDevSurfaceState.coursePackageExportStatusDisplay === "none", `course package export status should start hidden: ${defaultDevSurfaceState.coursePackageExportStatusDisplay}`);
    assert(defaultDevSurfaceState.coursePackageExportStatusState === "idle", `course package export status state mismatch: ${defaultDevSurfaceState.coursePackageExportStatusState}`);
    assert(defaultDevSurfaceState.coursePackageExportStatusRole === "status", `course package export status role mismatch: ${defaultDevSurfaceState.coursePackageExportStatusRole}`);
    assert(defaultDevSurfaceState.coursePackageExportStatusLive === "polite", `course package export status aria-live mismatch: ${defaultDevSurfaceState.coursePackageExportStatusLive}`);
    await page.click("#btn-paste-local-package");
    await page.click("#btn-local-package-paste-open");
    const emptyPasteState = await page.evaluate(() => ({
      activeId: document.activeElement?.id ?? "",
      buttonExpanded: document.querySelector("#btn-paste-local-package")?.getAttribute("aria-expanded") ?? "",
      panelOpen: document.querySelector("[data-local-package-paste-panel]")?.dataset?.open ?? "",
      panelAriaHidden: document.querySelector("[data-local-package-paste-panel]")?.getAttribute("aria-hidden") ?? "",
      statusText: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      statusDisplay: getComputedStyle(document.querySelector("[data-local-package-import-status]")).display,
      previewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      previewDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-preview]")).display,
      currentScreen: document.querySelector("#screen-browse")?.classList.contains("hidden") ? "run" : "browse",
    }));
    assert(emptyPasteState.activeId === "local-package-paste-text", `empty paste should focus textarea: ${emptyPasteState.activeId}`);
    assert(emptyPasteState.buttonExpanded === "true", `empty paste should keep paste button expanded: ${emptyPasteState.buttonExpanded}`);
    assert(emptyPasteState.panelOpen === "1", `empty paste should keep paste panel open: ${emptyPasteState.panelOpen}`);
    assert(emptyPasteState.panelAriaHidden === "false", `empty paste should keep panel exposed: ${emptyPasteState.panelAriaHidden}`);
    assert(emptyPasteState.statusText === "붙여넣을 배포 JSON이 없습니다.", `empty paste status mismatch: ${emptyPasteState.statusText}`);
    assert(emptyPasteState.statusState === "error", `empty paste status state mismatch: ${emptyPasteState.statusState}`);
    assert(emptyPasteState.statusDisplay !== "none", `empty paste status should be visible: ${emptyPasteState.statusDisplay}`);
    assert(emptyPasteState.previewText === "", `empty paste should not show preview: ${emptyPasteState.previewText}`);
    assert(emptyPasteState.previewDisplay === "none", `empty paste preview should stay hidden: ${emptyPasteState.previewDisplay}`);
    assert(emptyPasteState.currentScreen === "browse", `empty paste should stay on browse: ${emptyPasteState.currentScreen}`);
    await page.click("#btn-local-package-paste-cancel");
    const canceledEmptyPasteState = await page.evaluate(() => ({
      buttonExpanded: document.querySelector("#btn-paste-local-package")?.getAttribute("aria-expanded") ?? "",
      panelOpen: document.querySelector("[data-local-package-paste-panel]")?.dataset?.open ?? "",
      panelAriaHidden: document.querySelector("[data-local-package-paste-panel]")?.getAttribute("aria-hidden") ?? "",
      statusText: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      statusDisplay: getComputedStyle(document.querySelector("[data-local-package-import-status]")).display,
      previewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      previewDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-preview]")).display,
    }));
    assert(canceledEmptyPasteState.buttonExpanded === "false", `paste cancel should collapse paste button: ${canceledEmptyPasteState.buttonExpanded}`);
    assert(canceledEmptyPasteState.panelOpen === "0", `paste cancel should close panel: ${canceledEmptyPasteState.panelOpen}`);
    assert(canceledEmptyPasteState.panelAriaHidden === "true", `paste cancel should hide panel from accessibility tree: ${canceledEmptyPasteState.panelAriaHidden}`);
    assert(canceledEmptyPasteState.statusText === "", `paste cancel should clear stale error text: ${canceledEmptyPasteState.statusText}`);
    assert(canceledEmptyPasteState.statusState === "idle", `paste cancel should reset status state: ${canceledEmptyPasteState.statusState}`);
    assert(canceledEmptyPasteState.statusDisplay === "none", `paste cancel should hide stale error: ${canceledEmptyPasteState.statusDisplay}`);
    assert(canceledEmptyPasteState.previewText === "", `paste cancel should clear preview text: ${canceledEmptyPasteState.previewText}`);
    assert(canceledEmptyPasteState.previewDisplay === "none", `paste cancel should hide preview: ${canceledEmptyPasteState.previewDisplay}`);
    await page.click("#btn-paste-local-package");
    await page.keyboard.press("Escape");
    const escapePasteState = await page.evaluate(() => ({
      activeId: document.activeElement?.id ?? "",
      buttonExpanded: document.querySelector("#btn-paste-local-package")?.getAttribute("aria-expanded") ?? "",
      panelOpen: document.querySelector("[data-local-package-paste-panel]")?.dataset?.open ?? "",
      panelAriaHidden: document.querySelector("[data-local-package-paste-panel]")?.getAttribute("aria-hidden") ?? "",
      panelDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-panel]")).display,
      statusText: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
    }));
    assert(escapePasteState.activeId === "btn-paste-local-package", `Escape should return focus to paste button: ${escapePasteState.activeId}`);
    assert(escapePasteState.buttonExpanded === "false", `Escape should collapse paste button: ${escapePasteState.buttonExpanded}`);
    assert(escapePasteState.panelOpen === "0", `Escape should close paste panel: ${escapePasteState.panelOpen}`);
    assert(escapePasteState.panelAriaHidden === "true", `Escape should hide paste panel from accessibility tree: ${escapePasteState.panelAriaHidden}`);
    assert(escapePasteState.panelDisplay === "none", `Escape should hide paste panel: ${escapePasteState.panelDisplay}`);
    assert(escapePasteState.statusText === "", `Escape should not leave paste status text: ${escapePasteState.statusText}`);
    assert(escapePasteState.statusState === "idle", `Escape should keep paste status idle: ${escapePasteState.statusState}`);
    await page.click("#btn-paste-local-package");
    await page.fill("#local-package-paste-text", "{");
    await page.keyboard.press("Control+Enter");
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.file_name === "pasted-teacher-package.json");
    const invalidPasteState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      panelOpen: document.querySelector("[data-local-package-paste-panel]")?.dataset?.open ?? "",
      panelDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-panel]")).display,
      pasteTextValue: document.querySelector("#local-package-paste-text")?.value ?? "",
      previewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      previewState: document.querySelector("[data-local-package-paste-preview]")?.dataset?.state ?? "",
      previewDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-preview]")).display,
      statusText: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      currentScreen: document.querySelector("#screen-browse")?.classList.contains("hidden") ? "run" : "browse",
    }));
    assert(invalidPasteState.importAction?.imported === false, "invalid pasted package should be rejected");
    assert(
      invalidPasteState.importAction?.error === "studio_local_package_invalid_json",
      `invalid pasted package error mismatch: ${invalidPasteState.importAction?.error}`,
    );
    assert(invalidPasteState.panelOpen === "1", `invalid paste should keep panel open: ${invalidPasteState.panelOpen}`);
    assert(invalidPasteState.panelDisplay !== "none", `invalid paste should keep panel visible: ${invalidPasteState.panelDisplay}`);
    assert(invalidPasteState.pasteTextValue === "{", `invalid paste should preserve input for retry: ${invalidPasteState.pasteTextValue}`);
    assert(invalidPasteState.previewText.includes("JSON 형식"), `invalid paste preview should guide JSON fix: ${invalidPasteState.previewText}`);
    assert(invalidPasteState.previewState === "pending", `invalid paste preview state mismatch: ${invalidPasteState.previewState}`);
    assert(invalidPasteState.previewDisplay !== "none", `invalid paste preview should stay visible: ${invalidPasteState.previewDisplay}`);
    assert(
      invalidPasteState.statusText.includes("배포 묶음 JSON을 읽을 수 없습니다."),
      `invalid paste status mismatch: ${invalidPasteState.statusText}`,
    );
    assert(invalidPasteState.statusState === "error", `invalid paste status state mismatch: ${invalidPasteState.statusState}`);
    assert(invalidPasteState.currentScreen === "browse", `invalid paste should stay on browse: ${invalidPasteState.currentScreen}`);
    await page.click("#btn-paste-local-package");
    const retryPasteStartState = await page.evaluate(() => ({
      panelOpen: document.querySelector("[data-local-package-paste-panel]")?.dataset?.open ?? "",
      pasteTextValue: document.querySelector("#local-package-paste-text")?.value ?? "",
      previewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      previewState: document.querySelector("[data-local-package-paste-preview]")?.dataset?.state ?? "",
      statusText: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      statusDisplay: getComputedStyle(document.querySelector("[data-local-package-import-status]")).display,
    }));
    assert(retryPasteStartState.panelOpen === "1", `retry paste should keep panel open: ${retryPasteStartState.panelOpen}`);
    assert(retryPasteStartState.pasteTextValue === "{", `retry paste should preserve input for editing: ${retryPasteStartState.pasteTextValue}`);
    assert(retryPasteStartState.previewText.includes("JSON 형식"), `retry paste should keep JSON preview guidance: ${retryPasteStartState.previewText}`);
    assert(retryPasteStartState.previewState === "pending", `retry paste preview state mismatch: ${retryPasteStartState.previewState}`);
    assert(retryPasteStartState.statusText === "", `retry paste should clear stale status text: ${retryPasteStartState.statusText}`);
    assert(retryPasteStartState.statusState === "idle", `retry paste should reset status state: ${retryPasteStartState.statusState}`);
    assert(retryPasteStartState.statusDisplay === "none", `retry paste should hide stale status: ${retryPasteStartState.statusDisplay}`);
    await page.fill("#local-package-paste-text", "{}");
    const invalidPayloadPreviewState = await page.evaluate(() => ({
      previewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      previewState: document.querySelector("[data-local-package-paste-preview]")?.dataset?.state ?? "",
      previewDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-preview]")).display,
      statusText: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
    }));
    assert(invalidPayloadPreviewState.previewState === "error", `invalid payload preview state mismatch: ${invalidPayloadPreviewState.previewState}`);
    assert(invalidPayloadPreviewState.previewDisplay !== "none", `invalid payload preview should be visible: ${invalidPayloadPreviewState.previewDisplay}`);
    assert(
      invalidPayloadPreviewState.previewText.includes("배포 형식")
        && invalidPayloadPreviewState.previewText.includes("셈그림 배포 묶음 파일이 아닙니다."),
      `invalid payload preview mismatch: ${invalidPayloadPreviewState.previewText}`,
    );
    assert(invalidPayloadPreviewState.statusText === "" && invalidPayloadPreviewState.statusState === "idle", `invalid payload preview should not set import status: ${invalidPayloadPreviewState.statusText}/${invalidPayloadPreviewState.statusState}`);
    await page.evaluate(() => {
      document.querySelector("#btn-open-local-package")?.click();
    });
    const switchToFileOpenState = await page.evaluate(() => ({
      buttonExpanded: document.querySelector("#btn-paste-local-package")?.getAttribute("aria-expanded") ?? "",
      panelOpen: document.querySelector("[data-local-package-paste-panel]")?.dataset?.open ?? "",
      panelAriaHidden: document.querySelector("[data-local-package-paste-panel]")?.getAttribute("aria-hidden") ?? "",
      pasteTextValue: document.querySelector("#local-package-paste-text")?.value ?? "",
      previewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      previewDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-preview]")).display,
      statusText: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      statusDisplay: getComputedStyle(document.querySelector("[data-local-package-import-status]")).display,
    }));
    assert(switchToFileOpenState.buttonExpanded === "false", `file open should collapse paste button: ${switchToFileOpenState.buttonExpanded}`);
    assert(switchToFileOpenState.panelOpen === "0", `file open should close paste panel: ${switchToFileOpenState.panelOpen}`);
    assert(switchToFileOpenState.panelAriaHidden === "true", `file open should hide paste panel from accessibility tree: ${switchToFileOpenState.panelAriaHidden}`);
    assert(switchToFileOpenState.pasteTextValue === "", `file open should clear stale paste text: ${switchToFileOpenState.pasteTextValue}`);
    assert(switchToFileOpenState.previewText === "", `file open should clear paste preview text: ${switchToFileOpenState.previewText}`);
    assert(switchToFileOpenState.previewDisplay === "none", `file open should hide paste preview: ${switchToFileOpenState.previewDisplay}`);
    assert(switchToFileOpenState.statusText === "", `file open should clear stale paste status: ${switchToFileOpenState.statusText}`);
    assert(switchToFileOpenState.statusState === "idle", `file open should reset paste status state: ${switchToFileOpenState.statusState}`);
    assert(switchToFileOpenState.statusDisplay === "none", `file open should hide stale paste status: ${switchToFileOpenState.statusDisplay}`);
    assert(
      defaultDevSurfaceState.mainShellTabTexts.join("|") === "교과|수업",
      `teacher main tabs should use classroom labels: ${defaultDevSurfaceState.mainShellTabTexts.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.browseTabTexts.join("|") === "대표 교과|샘플 교과|전체 검색",
      `teacher browse tabs should use course-facing labels: ${defaultDevSurfaceState.browseTabTexts.join("|")}`,
    );
    assert(defaultDevSurfaceState.createButtonText === "교과 만들기", `create button label mismatch: ${defaultDevSurfaceState.createButtonText}`);
    assert(defaultDevSurfaceState.queryPlaceholder === "교과명, 과목, 활동 검색", `query placeholder mismatch: ${defaultDevSurfaceState.queryPlaceholder}`);
    assert(
      defaultDevSurfaceState.catalogBodyWidth >= 1000,
      `catalog body should use desktop width: ${defaultDevSurfaceState.catalogBodyWidth}`,
    );
    assert(
      defaultDevSurfaceState.catalogSummaryRect && defaultDevSurfaceState.catalogSummaryRect.width >= defaultDevSurfaceState.catalogBodyWidth - 40,
      `course summary should span the catalog width: ${JSON.stringify(defaultDevSurfaceState.catalogSummaryRect)}`,
    );
    assert(
      defaultDevSurfaceState.cardGridRect && defaultDevSurfaceState.cardGridRect.x < 80 && defaultDevSurfaceState.cardGridRect.width >= 900,
      `lesson grid should stay in the primary desktop column: ${JSON.stringify(defaultDevSurfaceState.cardGridRect)}`,
    );
    ["★", "◇", "🔍", "+ 만들기"].forEach((text) => {
      assert(!defaultDevSurfaceState.visibleText.includes(text), `default teacher UI should not expose generic tab/create marker: ${text}`);
    });
    assert(defaultDevSurfaceState.visibleLessonIds.length > 0, "default teacher catalog should show course lessons");
    assert(
      defaultDevSurfaceState.courseSequenceTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.courseSequenceTexts[0] === "추천 순서 1"
        && defaultDevSurfaceState.courseSequenceTexts.every((text, index) => text === `추천 순서 ${index + 1}`),
      `default teacher catalog sequence mismatch: ${defaultDevSurfaceState.courseSequenceTexts.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.cardTagNames.every((tagName) => tagName === "ARTICLE"),
      `lesson cards should not be nested button containers: ${defaultDevSurfaceState.cardTagNames.join(",")}`,
    );
    assert(
      defaultDevSurfaceState.cardRoles.every((role) => role === "button"),
      `lesson cards should expose button role for keyboard detail open: ${defaultDevSurfaceState.cardRoles.join(",")}`,
    );
    assert(
      defaultDevSurfaceState.cardTabIndexes.every((tabIndex) => tabIndex === "0"),
      `lesson cards should be keyboard focusable: ${defaultDevSurfaceState.cardTabIndexes.join(",")}`,
    );
    assert(
      defaultDevSurfaceState.studentLaunchButtonTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.studentLaunchButtonTexts.every((text) => text === "▶ 학생 시작"),
      `student launch button label mismatch: ${defaultDevSurfaceState.studentLaunchButtonTexts.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.teacherLaunchButtonTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.teacherLaunchButtonTexts.every((text) => text === "교사용 배포"),
      `teacher launch button label mismatch: ${defaultDevSurfaceState.teacherLaunchButtonTexts.join("|")}`,
    );
    const mobilePage = await context.newPage();
    await mobilePage.setViewportSize({ width: 390, height: 844 });
    await mobilePage.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await waitVisible(mobilePage, "#screen-browse");
    await mobilePage.waitForSelector(".lesson-card[data-lesson-id='rep_physics_velocity_history_v1']");
    await mobilePage.click(".lesson-card[data-lesson-id='rep_physics_velocity_history_v1']");
    const mobileBrowseDetailState = await mobilePage.evaluate(() => {
      const viewportWidth = document.documentElement.clientWidth;
      const flowGrid = document.querySelector("[data-detail-course-flow] .detail-course-flow-grid");
      const shortcutOverflows = Array.from(document.querySelectorAll(".course-subject-shortcut"))
        .map((node) => {
          const rect = node.getBoundingClientRect();
          return { text: node.textContent?.trim() || "", left: Math.round(rect.left), right: Math.round(rect.right), width: Math.round(rect.width) };
        })
        .filter((row) => row.left < 0 || row.right > viewportWidth + 1);
      const flowItems = Array.from(document.querySelectorAll("[data-detail-course-flow] .detail-course-flow-item")).map((node) => {
        const rect = node.getBoundingClientRect();
        return { text: node.textContent?.trim() || "", left: Math.round(rect.left), right: Math.round(rect.right), width: Math.round(rect.width) };
      });
      return {
        viewportWidth,
        scrollWidth: document.documentElement.scrollWidth,
        detailVisible: !document.querySelector("#catalog-detail-panel")?.classList?.contains("hidden"),
        flowGridColumns: flowGrid ? getComputedStyle(flowGrid).gridTemplateColumns : "",
        flowItemCount: flowItems.length,
        flowItemOverflows: flowItems.filter((row) => row.left < 0 || row.right > viewportWidth + 1),
        shortcutOverflows,
      };
    });
    assert(mobileBrowseDetailState.detailVisible === true, "mobile lesson detail should open from a card");
    assert(
      mobileBrowseDetailState.scrollWidth <= mobileBrowseDetailState.viewportWidth,
      `mobile browse detail should not force horizontal scroll: ${JSON.stringify(mobileBrowseDetailState)}`,
    );
    assert(mobileBrowseDetailState.flowItemCount === 3, `mobile detail flow item count mismatch: ${mobileBrowseDetailState.flowItemCount}`);
    assert(
      String(mobileBrowseDetailState.flowGridColumns || "").split(" ").length === 1,
      `mobile detail flow should collapse to one column: ${mobileBrowseDetailState.flowGridColumns}`,
    );
    assert(
      mobileBrowseDetailState.flowItemOverflows.length === 0,
      `mobile detail flow items should not overflow: ${JSON.stringify(mobileBrowseDetailState.flowItemOverflows)}`,
    );
    assert(
      mobileBrowseDetailState.shortcutOverflows.length === 0,
      `mobile subject shortcuts should not overflow: ${JSON.stringify(mobileBrowseDetailState.shortcutOverflows)}`,
    );
    await mobilePage.click("#btn-open-in-studio");
    await waitVisible(mobilePage, "#screen-run");
    await mobilePage.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.onboarding_profile === "student");
    const mobileRunLayoutState = await mobilePage.evaluate(() => {
      const overflowers = Array.from(document.querySelectorAll("#screen-run *"))
        .filter((node) => {
          const rect = node.getBoundingClientRect();
          return rect.width > 0 && (rect.left < -1 || rect.right > window.innerWidth + 1);
        })
        .map((node) => ({
          id: node.id || "",
          className: String(node.className || ""),
          text: String(node.textContent || "").trim().slice(0, 80),
        }));
      const rect = (selector) => {
        const node = document.querySelector(selector);
        const box = node?.getBoundingClientRect?.();
        return box ? { width: Math.round(box.width), height: Math.round(box.height) } : null;
      };
      return {
        scrollWidth: document.documentElement.scrollWidth,
        viewportWidth: window.innerWidth,
        controlBar: rect("#screen-run .run-control-bar"),
        layout: rect("#screen-run .run-layout"),
        overflowers,
        forbiddenText: ["새 작업", "저장 대기", "세션 대기"].filter((text) => document.body.innerText.includes(text)),
      };
    });
    await mobilePage.close();
    assert(
      mobileRunLayoutState.scrollWidth <= mobileRunLayoutState.viewportWidth,
      `mobile run screen should not force horizontal scroll: ${JSON.stringify(mobileRunLayoutState)}`,
    );
    assert(
      mobileRunLayoutState.overflowers.length === 0,
      `mobile run controls should not overflow viewport: ${JSON.stringify(mobileRunLayoutState.overflowers)}`,
    );
    assert(
      mobileRunLayoutState.forbiddenText.length === 0,
      `mobile run screen should hide draft/save/session text: ${mobileRunLayoutState.forbiddenText.join(",")}`,
    );
    assert(
      mobileRunLayoutState.controlBar && mobileRunLayoutState.controlBar.height > 42,
      `mobile run control bar should wrap instead of clipping controls: ${JSON.stringify(mobileRunLayoutState.controlBar)}`,
    );
    assert(
      defaultDevSurfaceState.visibleLessonIds.includes("rep_physics_velocity_history_v1"),
      `default teacher catalog missing velocity history lesson: ${defaultDevSurfaceState.visibleLessonIds.join(",")}`,
    );
    assert(defaultDevSurfaceState.initialDetailVisible === true, "default teacher catalog should auto-open the first course detail");
    assert(
      defaultDevSurfaceState.initialDetailTitle === "속도 기록 그래프",
      `initial course detail title mismatch: ${defaultDevSurfaceState.initialDetailTitle}`,
    );
    assert(
      defaultDevSurfaceState.initialDetailCourseSequenceText === "추천 순서 1",
      `initial course detail sequence mismatch: ${defaultDevSurfaceState.initialDetailCourseSequenceText}`,
    );
    assert(
      defaultDevSurfaceState.initialDetailText.includes("학습목표")
        && defaultDevSurfaceState.initialDetailText.includes("수업 활동")
        && defaultDevSurfaceState.initialDetailText.includes("DDN 원문")
        && defaultDevSurfaceState.initialDetailText.includes("학생 화면 미리보기")
        && defaultDevSurfaceState.initialDetailText.includes("실행 전 확인")
        && defaultDevSurfaceState.initialDetailText.includes("결과 확인"),
      `initial course detail missing classroom sections: ${defaultDevSurfaceState.initialDetailText}`,
    );
    assert(
      defaultDevSurfaceState.initialRunReadinessText.includes("학생으로 실행")
        && defaultDevSurfaceState.initialRunReadinessText.includes("수업 준비 상태: 수업 준비 완료")
        && defaultDevSurfaceState.initialRunReadinessText.includes("결과 확인: 그래프, 표")
        && defaultDevSurfaceState.initialRunReadinessText.includes("첫 활동:")
        && defaultDevSurfaceState.initialRunReadinessText.includes("교사용 배포 준비"),
      `initial run readiness mismatch: ${defaultDevSurfaceState.initialRunReadinessText}`,
    );
    assert(defaultDevSurfaceState.initialDdnPreviewStatus === "loaded", `initial DDN preview status mismatch: ${defaultDevSurfaceState.initialDdnPreviewStatus}`);
    assert(
      defaultDevSurfaceState.initialDdnPreviewText.includes("(시작)할때")
        && defaultDevSurfaceState.initialDdnPreviewText.includes("이력.만들기")
        && defaultDevSurfaceState.initialDdnPreviewText.includes("속도이력"),
      `initial DDN preview missing source content: ${defaultDevSurfaceState.initialDdnPreviewText}`,
    );
    assert(
      defaultDevSurfaceState.initialDdnPreviewPath.includes("rep_physics_velocity_history_v1/lesson.ddn"),
      `initial DDN preview source path mismatch: ${defaultDevSurfaceState.initialDdnPreviewPath}`,
    );
    assert(
      defaultDevSurfaceState.selectedLessonIds.join("|") === "rep_physics_velocity_history_v1",
      `initial selected course mismatch: ${defaultDevSurfaceState.selectedLessonIds.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.selectedAriaPressed.every((value) => value === "true"),
      `initial selected course aria mismatch: ${defaultDevSurfaceState.selectedAriaPressed.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.velocityCardText.includes("속도 기록 그래프")
        && defaultDevSurfaceState.velocityCardText.includes("매 단계마다 속도")
        && defaultDevSurfaceState.velocityCardText.includes("오늘 활동:")
        && defaultDevSurfaceState.velocityCardText.includes("결과 확인: 그래프")
        && defaultDevSurfaceState.velocityCardText.includes("수업 준비 완료")
        && defaultDevSurfaceState.velocityCardText.includes("교사용 배포"),
      `velocity history card missing course activity signals: ${defaultDevSurfaceState.velocityCardText}`,
    );
    assert(
      defaultDevSurfaceState.courseSummaryText.includes(`${defaultDevSurfaceState.visibleLessonIds.length}개 대표 교과`)
        && defaultDevSurfaceState.courseSummaryText.includes("대표 과목:")
        && defaultDevSurfaceState.courseSummaryText.includes("물리")
        && defaultDevSurfaceState.courseSummaryText.includes("수학")
        && defaultDevSurfaceState.courseSummaryText.includes("경제")
        && defaultDevSurfaceState.courseSummaryText.includes("DDN 실행")
        && defaultDevSurfaceState.courseSummaryText.includes("교사용 배포 가능")
        && defaultDevSurfaceState.courseSummaryText.includes("전체 검색으로 나머지 교과 확인"),
      `course summary mismatch: ${defaultDevSurfaceState.courseSummaryText}`,
    );
    assert(
      Number(defaultDevSurfaceState.courseSummaryTotal) >= Number(defaultDevSurfaceState.courseSummaryVisible)
        && Number(defaultDevSurfaceState.courseSummaryVisible) === defaultDevSurfaceState.visibleLessonIds.length,
      `course summary count mismatch: total=${defaultDevSurfaceState.courseSummaryTotal} visible=${defaultDevSurfaceState.courseSummaryVisible}`,
    );
    assert(
      defaultDevSurfaceState.courseSubjectShortcutTexts.some((text) => text === "전체")
        && defaultDevSurfaceState.courseSubjectShortcutTexts.some((text) => text.startsWith("물리 "))
        && defaultDevSurfaceState.courseSubjectShortcutTexts.some((text) => text.startsWith("수학 "))
        && defaultDevSurfaceState.courseSubjectShortcutTexts.some((text) => text.startsWith("경제 "))
        && defaultDevSurfaceState.courseSubjectShortcutFilters.includes("math"),
      `course subject shortcuts mismatch: ${defaultDevSurfaceState.courseSubjectShortcutTexts.join("|")} filters=${defaultDevSurfaceState.courseSubjectShortcutFilters.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.visibleLessonIds.every((id) => !String(id).includes("ddonirang") && !String(id).startsWith("rep_cs_")),
      `default teacher catalog leaked internal lessons: ${defaultDevSurfaceState.visibleLessonIds.join(",")}`,
    );
    await page.click("#btn-copy-course-package");
    await page.waitForFunction(() => window.__STUDIO_COURSE_PACKAGE_EXPORT_ACTION__?.schema === "seamgrim.course_package_export_action.v1");
    const coursePackageExportState = await page.evaluate(() => ({
      action: window.__STUDIO_COURSE_PACKAGE_EXPORT_ACTION__ ?? null,
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      statusText: document.querySelector("[data-course-package-export-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-course-package-export-status]")?.dataset?.state ?? "",
      buttonText: document.querySelector("#btn-copy-course-package")?.textContent?.trim() ?? "",
      buttonDisabled: Boolean(document.querySelector("#btn-copy-course-package")?.disabled),
    }));
    assert(coursePackageExportState.action?.copied === true, `course package copy failed: ${JSON.stringify(coursePackageExportState.action)}`);
    assert(coursePackageExportState.action?.lesson_count === defaultDevSurfaceState.visibleLessonIds.length, `course package lesson count mismatch: ${coursePackageExportState.action?.lesson_count}`);
    assert(coursePackageExportState.action?.multi_lesson === true, `course package copy should be multi lesson: ${JSON.stringify(coursePackageExportState.action)}`);
    assert(
      Array.isArray(coursePackageExportState.action?.lesson_ids)
        && coursePackageExportState.action.lesson_ids.join("|") === defaultDevSurfaceState.visibleLessonIds.join("|"),
      `course package copy lesson ids mismatch: ${JSON.stringify(coursePackageExportState.action?.lesson_ids)}`,
    );
    assert(coursePackageExportState.action?.report_count === 3, `course package report count mismatch: ${coursePackageExportState.action?.report_count}`);
    assert(coursePackageExportState.action?.student_material_count === 2, `course package student material count mismatch: ${coursePackageExportState.action?.student_material_count}`);
    assert(coursePackageExportState.statusState === "ok", `course package export status state mismatch: ${coursePackageExportState.statusState}`);
    assert(coursePackageExportState.statusText.includes("대표 교과 배포 묶음을 복사했습니다"), `course package export status mismatch: ${coursePackageExportState.statusText}`);
    assert(
      coursePackageExportState.statusText.includes("학생 자료 2개")
        && coursePackageExportState.statusText.includes("전체 자료 3개")
        && !coursePackageExportState.statusText.includes("학생 자료 3개"),
      `course package export material status mismatch: ${coursePackageExportState.statusText}`,
    );
    assert(coursePackageExportState.buttonText === "대표 묶음 복사", `course package copy button should restore label: ${coursePackageExportState.buttonText}`);
    assert(coursePackageExportState.buttonDisabled === false, "course package copy button should be re-enabled");
    const copiedCoursePayload = JSON.parse(coursePackageExportState.copiedText);
    const copiedValidation = localPackageHelper.validateStudioLocalPackagePayload(copiedCoursePayload);
    assert(copiedValidation.valid === true, `copied course package should validate: ${copiedValidation.error}`);
    assert(copiedCoursePayload.manifest?.package_id === "studio.local.course.representative.v1", `course package id mismatch: ${copiedCoursePayload.manifest?.package_id}`);
    assert(copiedCoursePayload.manifest?.lesson_count === defaultDevSurfaceState.visibleLessonIds.length, `course package manifest lesson count mismatch: ${copiedCoursePayload.manifest?.lesson_count}`);
    assert(copiedCoursePayload.manifest?.report_count === 3, `course package manifest report count mismatch: ${copiedCoursePayload.manifest?.report_count}`);
    assert(copiedCoursePayload.lessons?.length === defaultDevSurfaceState.visibleLessonIds.length, `course package payload lesson count mismatch: ${copiedCoursePayload.lessons?.length}`);
    assert(copiedCoursePayload.lessons.every((lesson) => String(lesson.source_text ?? "").trim().length > 0), "course package lessons must include DDN source text");
    const copiedCourseReportTitles = (copiedCoursePayload.reports ?? []).map((report) => String(report.title ?? ""));
    assert(copiedCourseReportTitles.includes("대표 교과 학생 배포 안내문"), `course package student guide missing: ${copiedCourseReportTitles.join("|")}`);
    assert(copiedCourseReportTitles.includes("대표 교과 교사용 준비 체크리스트"), `course package teacher checklist missing: ${copiedCourseReportTitles.join("|")}`);
    assert(copiedCourseReportTitles.includes("대표 교과 학생 명단 양식"), `course package roster template missing: ${copiedCourseReportTitles.join("|")}`);
    const copiedCourseRosterReport = copiedCoursePayload.reports?.find((report) => String(report?.report_id ?? "") === "representative_course_student_roster_template");
    assert(copiedCourseRosterReport, `course package roster template report missing: ${JSON.stringify(copiedCoursePayload.reports)}`);
    assert(
      String(copiedCourseRosterReport?.title ?? "") === "대표 교과 학생 명단 양식"
        && String(copiedCourseRosterReport?.path ?? "") === "reports/representative_course_student_roster_template.tsv"
        && String(copiedCourseRosterReport?.mime ?? "").includes("text/tab-separated-values")
        && String(copiedCourseRosterReport?.text ?? "").includes("# 셈그림 대표 교과 학생 명단")
        && String(copiedCourseRosterReport?.text ?? "").includes("학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드")
        && copiedCoursePayload.lessons.every((lesson) => String(copiedCourseRosterReport?.text ?? "").includes(`\t${lesson.lesson_id}\t`))
        && !String(copiedCourseRosterReport?.text ?? "").includes("예시학생"),
      `course package roster template text mismatch: ${JSON.stringify(copiedCourseRosterReport)}`,
    );
    assert(
      copiedCoursePayload.manifest?.materials_summary?.some((item) => String(item).includes("교사용 자료 3개"))
        && copiedCoursePayload.manifest?.materials_summary?.some((item) => String(item).includes("교사용 준비 체크리스트"))
        && copiedCoursePayload.manifest?.materials_summary?.some((item) => String(item).includes("학생 명단 양식")),
      `course package materials summary mismatch: ${JSON.stringify(copiedCoursePayload.manifest?.materials_summary)}`,
    );
    assert(
      copiedCoursePayload.manifest?.student_materials_summary?.some((item) => String(item).includes(`교과 ${defaultDevSurfaceState.visibleLessonIds.length}개`)),
      `course package student materials summary mismatch: ${JSON.stringify(copiedCoursePayload.manifest?.student_materials_summary)}`,
    );
    assert(
      copiedCoursePayload.manifest?.student_materials_summary?.some((item) => String(item).includes("학생 자료 2개"))
        && copiedCoursePayload.manifest?.student_materials_summary?.some((item) => String(item).includes("학생 배포 안내문"))
        && copiedCoursePayload.manifest?.student_materials_summary?.some((item) => String(item).includes("학생 명단 양식")),
      `course package student materials detail mismatch: ${JSON.stringify(copiedCoursePayload.manifest?.student_materials_summary)}`,
    );
    assert(
      copiedCoursePayload.manifest?.student_instructions?.some((item) => String(item).includes("배포 수업 선택에서 오늘 수업"))
        && copiedCoursePayload.manifest?.student_instructions?.some((item) => String(item).includes(`교과 수: ${defaultDevSurfaceState.visibleLessonIds.length}`))
        && copiedCoursePayload.manifest?.student_instructions?.some((item) => String(item).includes("첫 수업: 속도 기록 그래프"))
        && copiedCoursePayload.manifest?.student_instructions?.some((item) => String(item).includes("첫 수업 코드: rep_physics_velocity_history_v1"))
        && copiedCoursePayload.student_instructions?.some((item) => String(item).includes("배포 수업 선택에서 오늘 수업"))
        && copiedCoursePayload.student_instructions?.some((item) => String(item).includes(`교과 수: ${defaultDevSurfaceState.visibleLessonIds.length}`))
        && copiedCoursePayload.student_instructions?.some((item) => String(item).includes("첫 수업: 속도 기록 그래프"))
        && copiedCoursePayload.student_instructions?.some((item) => String(item).includes("첫 수업 코드: rep_physics_velocity_history_v1")),
      `course package multi lesson student instructions mismatch: manifest=${JSON.stringify(copiedCoursePayload.manifest?.student_instructions)} payload=${JSON.stringify(copiedCoursePayload.student_instructions)}`,
    );
    await page.click("#btn-paste-local-package");
    await page.fill("#local-package-paste-text", coursePackageExportState.copiedText);
    await page.waitForFunction(() => document.querySelector("[data-local-package-paste-preview]")?.dataset?.state === "ok");
    const copiedCoursePastePreviewState = await page.evaluate(() => ({
      previewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      previewState: document.querySelector("[data-local-package-paste-preview]")?.dataset?.state ?? "",
    }));
    assert(copiedCoursePastePreviewState.previewState === "ok", `copied course paste preview state mismatch: ${copiedCoursePastePreviewState.previewState}`);
    assert(
      copiedCoursePastePreviewState.previewText.includes(`교과 ${defaultDevSurfaceState.visibleLessonIds.length}개`)
        && copiedCoursePastePreviewState.previewText.includes("첫 수업 속도 기록 그래프")
        && copiedCoursePastePreviewState.previewText.includes("배포 수업 선택 가능")
        && copiedCoursePastePreviewState.previewText.includes("학생 자료 2개")
        && copiedCoursePastePreviewState.previewText.includes("학생 명단 양식")
        && !copiedCoursePastePreviewState.previewText.includes("교사용 자료"),
      `copied course paste preview mismatch: ${copiedCoursePastePreviewState.previewText}`,
    );
    await page.click("#btn-local-package-paste-open");
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.package_id === "studio.local.course.representative.v1");
    await waitVisible(page, "#screen-run");
    const importedCoursePackageState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      switchHidden: document.querySelector("[data-run-package-lesson-switch]")?.classList?.contains("hidden") ?? true,
      optionTexts: Array.from(document.querySelectorAll("#run-package-lesson-select option")).map((node) => node.textContent?.trim() || ""),
      optionValues: Array.from(document.querySelectorAll("#run-package-lesson-select option")).map((node) => node.getAttribute("value") || ""),
      titleText: document.querySelector("#run-lesson-title")?.textContent?.trim() ?? "",
      briefText: document.querySelector("[data-run-lesson-brief]")?.textContent?.trim() ?? "",
      deliveryInstructionsText: document.querySelector("[data-run-delivery-instructions]")?.textContent?.trim() ?? "",
      deliveryInstructionsReady: document.querySelector("[data-run-delivery-instructions]")?.dataset?.ready ?? "",
      classroomModeAccess: document.querySelector("#screen-run")?.dataset?.classroomModeAccess ?? "",
    }));
    assert(importedCoursePackageState.importAction?.imported === true, "copied course package should import through paste");
    assert(importedCoursePackageState.importAction?.lesson_count === defaultDevSurfaceState.visibleLessonIds.length, `copied course package import count mismatch: ${JSON.stringify(importedCoursePackageState.importAction)}`);
    assert(importedCoursePackageState.importAction?.report_count === 3, `copied course package import report count mismatch: ${JSON.stringify(importedCoursePackageState.importAction)}`);
    assert(
      importedCoursePackageState.importAction?.report_titles?.includes("대표 교과 학생 배포 안내문")
        && importedCoursePackageState.importAction?.report_titles?.includes("대표 교과 교사용 준비 체크리스트")
        && importedCoursePackageState.importAction?.report_titles?.includes("대표 교과 학생 명단 양식"),
      `copied course package import report titles mismatch: ${JSON.stringify(importedCoursePackageState.importAction?.report_titles)}`,
    );
    assert(importedCoursePackageState.switchHidden === false, "copied course package should show package lesson switch");
    assert(importedCoursePackageState.optionTexts.length === defaultDevSurfaceState.visibleLessonIds.length, `copied course package option count mismatch: ${importedCoursePackageState.optionTexts.join("|")}`);
    const expectedCoursePackageOptionTexts = copiedCoursePayload.lessons.map((lesson, index) => `${index + 1}. ${lesson.title}`);
    assert(
      importedCoursePackageState.optionTexts.join("|") === expectedCoursePackageOptionTexts.join("|"),
      `copied course package option order mismatch: actual=${importedCoursePackageState.optionTexts.join("|")} expected=${expectedCoursePackageOptionTexts.join("|")}`,
    );
    assert(importedCoursePackageState.titleText.includes("속도 기록 그래프"), `copied course package initial title mismatch: ${importedCoursePackageState.titleText}`);
    assert(importedCoursePackageState.briefText.includes("교사가 보낸 배포 파일"), `copied course package brief mismatch: ${importedCoursePackageState.briefText}`);
    assert(
      importedCoursePackageState.deliveryInstructionsReady === "1"
        && importedCoursePackageState.deliveryInstructionsText.includes("학생 자료 2개")
        && importedCoursePackageState.deliveryInstructionsText.includes("학생 명단 양식"),
      `copied course package delivery instructions mismatch: ${importedCoursePackageState.deliveryInstructionsText}`,
    );
    assert(
      importedCoursePackageState.deliveryInstructionsText.includes("배포 수업 선택에서 오늘 수업"),
      `copied course package delivery instructions should explain lesson switching: ${importedCoursePackageState.deliveryInstructionsText}`,
    );
    assert(
      importedCoursePackageState.deliveryInstructionsText.includes(`교과 수: ${defaultDevSurfaceState.visibleLessonIds.length}`)
        && importedCoursePackageState.deliveryInstructionsText.includes("첫 수업: 속도 기록 그래프")
        && importedCoursePackageState.deliveryInstructionsText.includes("첫 수업 코드: rep_physics_velocity_history_v1"),
      `copied course package delivery instructions should label multi lesson context: ${importedCoursePackageState.deliveryInstructionsText}`,
    );
    assert(importedCoursePackageState.classroomModeAccess === "student", `local package import should keep student-only classroom access: ${importedCoursePackageState.classroomModeAccess}`);
    const secondCourseLesson = copiedCoursePayload.lessons[1];
    await page.selectOption("#run-package-lesson-select", importedCoursePackageState.optionValues[1]);
    await page.waitForFunction(
      (title) => document.querySelector("#run-lesson-title")?.textContent?.includes(title),
      secondCourseLesson.title,
    );
    const importedCourseSwitchState = await page.evaluate(() => ({
      titleText: document.querySelector("#run-lesson-title")?.textContent?.trim() ?? "",
      selectedText: document.querySelector("#run-package-lesson-select option:checked")?.textContent?.trim() ?? "",
      switchHidden: document.querySelector("[data-run-package-lesson-switch]")?.classList?.contains("hidden") ?? true,
      deliveryInstructionsText: document.querySelector("[data-run-delivery-instructions]")?.textContent?.trim() ?? "",
      inputRegistry: JSON.parse(localStorage.getItem("seamgrim.input_registry.v0") || "{}"),
    }));
    assert(importedCourseSwitchState.titleText.includes(secondCourseLesson.title), `copied course switched title mismatch: ${importedCourseSwitchState.titleText}`);
    assert(importedCourseSwitchState.selectedText === `2. ${secondCourseLesson.title}`, `copied course switched select mismatch: ${importedCourseSwitchState.selectedText}`);
    assert(importedCourseSwitchState.switchHidden === false, "copied course switch should remain visible after selecting second lesson");
    assert(
      importedCourseSwitchState.deliveryInstructionsText.includes("학생 자료 2개")
        && importedCourseSwitchState.deliveryInstructionsText.includes("학생 명단 양식"),
      `copied course switched delivery instructions mismatch: ${importedCourseSwitchState.deliveryInstructionsText}`,
    );
    assert(
      importedCourseSwitchState.deliveryInstructionsText.includes("배포 수업 선택에서 오늘 수업"),
      `copied course switched delivery instructions should keep lesson switching hint: ${importedCourseSwitchState.deliveryInstructionsText}`,
    );
    assert(
      importedCourseSwitchState.deliveryInstructionsText.includes(`교과 수: ${defaultDevSurfaceState.visibleLessonIds.length}`)
        && importedCourseSwitchState.deliveryInstructionsText.includes("첫 수업: 속도 기록 그래프")
        && importedCourseSwitchState.deliveryInstructionsText.includes("첫 수업 코드: rep_physics_velocity_history_v1"),
      `copied course switched delivery instructions should keep multi lesson context: ${importedCourseSwitchState.deliveryInstructionsText}`,
    );
    assert(
      importedCourseSwitchState.deliveryInstructionsText.includes(`오늘 수업: ${secondCourseLesson.title}`)
        && importedCourseSwitchState.deliveryInstructionsText.includes(`오늘 수업 코드: ${secondCourseLesson.lesson_id}`),
      `copied course switched delivery instructions should name selected lesson: ${importedCourseSwitchState.deliveryInstructionsText}`,
    );
    assert(
      importedCourseSwitchState.inputRegistry?.inputs?.selected_id === `local_package:studio.local.course.representative.v1:${secondCourseLesson.lesson_id}`,
      `copied course switched source mismatch: ${importedCourseSwitchState.inputRegistry?.inputs?.selected_id}`,
    );
    await page.evaluate(() => {
      document.querySelector("#btn-run-student-roster-template-copy")?.click();
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_COPY_ACTION__?.multi_lesson === true);
    const importedCourseRosterTemplateCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_COPY_ACTION__ ?? null,
    }));
    assert(importedCourseRosterTemplateCopyState.action?.lesson_id === secondCourseLesson.lesson_id, `course roster template copy should keep selected lesson context: ${JSON.stringify(importedCourseRosterTemplateCopyState.action)}`);
    assert(importedCourseRosterTemplateCopyState.action?.package_id === "studio.local.course.representative.v1", `course roster template copy package mismatch: ${JSON.stringify(importedCourseRosterTemplateCopyState.action)}`);
    assert(
      Array.isArray(importedCourseRosterTemplateCopyState.action?.lesson_ids)
        && importedCourseRosterTemplateCopyState.action.lesson_ids.join("|") === copiedCoursePayload.lessons.map((lesson) => lesson.lesson_id).join("|"),
      `course roster template copy should expose all lesson ids: ${JSON.stringify(importedCourseRosterTemplateCopyState.action)}`,
    );
    assert(
      importedCourseRosterTemplateCopyState.copiedText.includes("# 셈그림 대표 교과 학생 명단")
        && importedCourseRosterTemplateCopyState.copiedText.includes("학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드")
        && copiedCoursePayload.lessons.every((lesson) => importedCourseRosterTemplateCopyState.copiedText.includes(`\t${lesson.lesson_id}\t`)),
      `course roster template copy text should remain course-scoped: ${importedCourseRosterTemplateCopyState.copiedText}`,
    );
    const [importedCourseRosterTemplateDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.evaluate(() => {
        document.querySelector("#btn-run-student-roster-template-download")?.click();
      }),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_DOWNLOAD_ACTION__?.multi_lesson === true);
    const importedCourseRosterTemplateDownloadPath = await importedCourseRosterTemplateDownload.path();
    const importedCourseRosterTemplateDownloadedText = importedCourseRosterTemplateDownloadPath
      ? await fs.readFile(importedCourseRosterTemplateDownloadPath, "utf-8")
      : "";
    const importedCourseRosterTemplateDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(
      Array.isArray(importedCourseRosterTemplateDownloadState.action?.lesson_ids)
        && importedCourseRosterTemplateDownloadState.action.lesson_ids.join("|") === copiedCoursePayload.lessons.map((lesson) => lesson.lesson_id).join("|"),
      `course roster template download should expose all lesson ids: ${JSON.stringify(importedCourseRosterTemplateDownloadState.action)}`,
    );
    assert(importedCourseRosterTemplateDownloadState.action?.file_name === "representative_course_student_roster_template.tsv", `course roster template filename mismatch: ${importedCourseRosterTemplateDownloadState.action?.file_name}`);
    assert(importedCourseRosterTemplateDownload.suggestedFilename() === importedCourseRosterTemplateDownloadState.action.file_name, "course roster template suggested filename mismatch");
    assert(importedCourseRosterTemplateDownloadedText.trim() === importedCourseRosterTemplateCopyState.copiedText.trim(), "course roster template copy/download mismatch");
    await page.evaluate(() => {
      const button = document.querySelector("#btn-run-local-package-guide-copy");
      if (button) {
        button.disabled = false;
        button.click();
      }
    });
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_COPY_ACTION__?.multi_lesson === true);
    const importedCourseStudentGuideCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_COPY_ACTION__ ?? null,
    }));
    assert(importedCourseStudentGuideCopyState.action?.lesson_id === secondCourseLesson.lesson_id, `course student guide copy should keep selected lesson context: ${JSON.stringify(importedCourseStudentGuideCopyState.action)}`);
    assert(
      Array.isArray(importedCourseStudentGuideCopyState.action?.lesson_ids)
        && importedCourseStudentGuideCopyState.action.lesson_ids.join("|") === copiedCoursePayload.lessons.map((lesson) => lesson.lesson_id).join("|"),
      `course student guide copy should expose all lesson ids: ${JSON.stringify(importedCourseStudentGuideCopyState.action)}`,
    );
    assert(
      importedCourseStudentGuideCopyState.copiedText.includes(`교과 수: ${copiedCoursePayload.lessons.length}`)
        && importedCourseStudentGuideCopyState.copiedText.includes("배포 수업 선택에서 오늘 수업")
        && copiedCoursePayload.lessons.every((lesson) => importedCourseStudentGuideCopyState.copiedText.includes(String(lesson.lesson_id ?? ""))),
      `course student guide copy should remain course-scoped: ${importedCourseStudentGuideCopyState.copiedText}`,
    );
    const [importedCourseStudentGuideDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.evaluate(() => {
        const button = document.querySelector("#btn-run-local-package-guide-download");
        if (button) {
          button.disabled = false;
          button.click();
        }
      }),
    ]);
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_DOWNLOAD_ACTION__?.multi_lesson === true);
    const importedCourseStudentGuideDownloadPath = await importedCourseStudentGuideDownload.path();
    const importedCourseStudentGuideDownloadedText = importedCourseStudentGuideDownloadPath
      ? await fs.readFile(importedCourseStudentGuideDownloadPath, "utf-8")
      : "";
    const importedCourseStudentGuideDownloadState = await page.evaluate(() => ({
      action: window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(
      Array.isArray(importedCourseStudentGuideDownloadState.action?.lesson_ids)
        && importedCourseStudentGuideDownloadState.action.lesson_ids.join("|") === copiedCoursePayload.lessons.map((lesson) => lesson.lesson_id).join("|"),
      `course student guide download should expose all lesson ids: ${JSON.stringify(importedCourseStudentGuideDownloadState.action)}`,
    );
    assert(String(importedCourseStudentGuideDownloadState.action?.file_name ?? "").includes("studio.local.course.representative.v1"), `course student guide filename mismatch: ${importedCourseStudentGuideDownloadState.action?.file_name}`);
    assert(importedCourseStudentGuideDownload.suggestedFilename() === importedCourseStudentGuideDownloadState.action.file_name, "course student guide suggested filename mismatch");
    assert(importedCourseStudentGuideDownloadedText.trim() === importedCourseStudentGuideCopyState.copiedText.trim(), "course student guide copy/download mismatch");
    await page.evaluate(() => {
      const button = document.querySelector("#btn-run-local-package-checklist-copy");
      if (button) {
        button.disabled = false;
        button.click();
      }
    });
    await page.waitForFunction(() => window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_COPY_ACTION__?.multi_lesson === true);
    const importedCourseChecklistCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_COPY_ACTION__ ?? null,
    }));
    assert(importedCourseChecklistCopyState.action?.lesson_id === secondCourseLesson.lesson_id, `course checklist copy should keep selected lesson context: ${JSON.stringify(importedCourseChecklistCopyState.action)}`);
    assert(
      Array.isArray(importedCourseChecklistCopyState.action?.lesson_ids)
        && importedCourseChecklistCopyState.action.lesson_ids.join("|") === copiedCoursePayload.lessons.map((lesson) => lesson.lesson_id).join("|"),
      `course checklist copy should expose all lesson ids: ${JSON.stringify(importedCourseChecklistCopyState.action)}`,
    );
    assert(
      (
        importedCourseChecklistCopyState.copiedText.includes("셈그림 대표 교과 준비 체크리스트")
          || importedCourseChecklistCopyState.copiedText.includes("셈그림 대표 교과 교사용 준비 체크리스트")
      )
        && importedCourseChecklistCopyState.copiedText.includes("수업 전")
        && importedCourseChecklistCopyState.copiedText.includes("수업 중")
        && importedCourseChecklistCopyState.copiedText.includes("포함 교과")
        && copiedCoursePayload.lessons.every((lesson) => importedCourseChecklistCopyState.copiedText.includes(String(lesson.lesson_id ?? ""))),
      `course checklist copy text mismatch: ${importedCourseChecklistCopyState.copiedText}`,
    );
    const [importedCourseChecklistDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.evaluate(() => {
        const button = document.querySelector("#btn-run-local-package-checklist-download");
        if (button) {
          button.disabled = false;
          button.click();
        }
      }),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_DOWNLOAD_ACTION__?.multi_lesson === true);
    const importedCourseChecklistDownloadPath = await importedCourseChecklistDownload.path();
    const importedCourseChecklistDownloadedText = importedCourseChecklistDownloadPath
      ? await fs.readFile(importedCourseChecklistDownloadPath, "utf-8")
      : "";
    const importedCourseChecklistDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(
      Array.isArray(importedCourseChecklistDownloadState.action?.lesson_ids)
        && importedCourseChecklistDownloadState.action.lesson_ids.join("|") === copiedCoursePayload.lessons.map((lesson) => lesson.lesson_id).join("|"),
      `course checklist download should expose all lesson ids: ${JSON.stringify(importedCourseChecklistDownloadState.action)}`,
    );
    assert(importedCourseChecklistDownloadState.action?.file_name === "representative_course_teacher_preparation_checklist.txt", `course checklist filename mismatch: ${importedCourseChecklistDownloadState.action?.file_name}`);
    assert(importedCourseChecklistDownload.suggestedFilename() === importedCourseChecklistDownloadState.action.file_name, "course checklist suggested filename mismatch");
    assert(importedCourseChecklistDownloadedText.trim() === importedCourseChecklistCopyState.copiedText.trim(), "course checklist copy/download mismatch");
    await page.evaluate(() => {
      const button = document.querySelector("#btn-run-local-package-code-copy");
      if (button) {
        button.disabled = false;
        button.click();
      }
    });
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_CODE_COPY_ACTION__?.multi_lesson === true);
    const importedCoursePackageCodeCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__STUDIO_LOCAL_PACKAGE_CODE_COPY_ACTION__ ?? null,
    }));
    assert(importedCoursePackageCodeCopyState.action?.package_id === "studio.local.course.representative.v1", `course package code package mismatch: ${JSON.stringify(importedCoursePackageCodeCopyState.action)}`);
    assert(importedCoursePackageCodeCopyState.action?.lesson_id === secondCourseLesson.lesson_id, `course package code should keep selected lesson context: ${JSON.stringify(importedCoursePackageCodeCopyState.action)}`);
    assert(
      Array.isArray(importedCoursePackageCodeCopyState.action?.lesson_ids)
        && importedCoursePackageCodeCopyState.action.lesson_ids.join("|") === copiedCoursePayload.lessons.map((lesson) => lesson.lesson_id).join("|"),
      `course package code should expose all lesson ids: ${JSON.stringify(importedCoursePackageCodeCopyState.action)}`,
    );
    assert(importedCoursePackageCodeCopyState.copiedText === "studio.local.course.representative.v1", `course package code clipboard mismatch: ${importedCoursePackageCodeCopyState.copiedText}`);
    await page.evaluate(() => {
      document.querySelector("#screen-run .main-shell-tab[data-main-tab-target='browse']")?.click();
    });
    await waitVisible(page, "#screen-browse");
    const [coursePackageDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-download-course-package"),
    ]);
    await page.waitForFunction(() => window.__STUDIO_COURSE_PACKAGE_DOWNLOAD_ACTION__?.downloaded === true);
    const coursePackageDownloadedPath = await coursePackageDownload.path();
    const coursePackageDownloadedText = coursePackageDownloadedPath ? await fs.readFile(coursePackageDownloadedPath, "utf-8") : "";
    const coursePackageDownloadedPayload = JSON.parse(coursePackageDownloadedText);
    const coursePackageDownloadState = await page.evaluate(() => ({
      action: window.__STUDIO_COURSE_PACKAGE_DOWNLOAD_ACTION__ ?? null,
      statusText: document.querySelector("[data-course-package-export-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-course-package-export-status]")?.dataset?.state ?? "",
      buttonText: document.querySelector("#btn-download-course-package")?.textContent?.trim() ?? "",
      buttonDisabled: Boolean(document.querySelector("#btn-download-course-package")?.disabled),
    }));
    const downloadedCourseValidation = localPackageHelper.validateStudioLocalPackagePayload(coursePackageDownloadedPayload);
    assert(downloadedCourseValidation.valid === true, `downloaded course package should validate: ${downloadedCourseValidation.error}`);
    assert(coursePackageDownloadState.action?.schema === "seamgrim.course_package_export_action.v1", `course package download schema mismatch: ${coursePackageDownloadState.action?.schema}`);
    assert(coursePackageDownloadState.action?.downloaded === true, "course package download action mismatch");
    assert(coursePackageDownloadState.action?.lesson_count === defaultDevSurfaceState.visibleLessonIds.length, `course package download lesson count mismatch: ${coursePackageDownloadState.action?.lesson_count}`);
    assert(coursePackageDownloadState.action?.multi_lesson === true, `course package download should be multi lesson: ${JSON.stringify(coursePackageDownloadState.action)}`);
    assert(
      Array.isArray(coursePackageDownloadState.action?.lesson_ids)
        && coursePackageDownloadState.action.lesson_ids.join("|") === copiedCoursePayload.lessons.map((lesson) => lesson.lesson_id).join("|"),
      `course package download lesson ids mismatch: ${JSON.stringify(coursePackageDownloadState.action?.lesson_ids)}`,
    );
    assert(coursePackageDownloadState.action?.report_count === 3, `course package download report count mismatch: ${coursePackageDownloadState.action?.report_count}`);
    assert(coursePackageDownloadState.action?.student_material_count === 2, `course package download student material count mismatch: ${coursePackageDownloadState.action?.student_material_count}`);
    assert(String(coursePackageDownloadState.action?.file_name ?? "").endsWith(".json"), `course package download filename mismatch: ${coursePackageDownloadState.action?.file_name}`);
    assert(coursePackageDownload.suggestedFilename() === coursePackageDownloadState.action.file_name, "course package suggested filename mismatch");
    assert(coursePackageDownloadState.statusState === "ok", `course package download status state mismatch: ${coursePackageDownloadState.statusState}`);
    assert(coursePackageDownloadState.statusText.includes("대표 교과 배포 묶음을 저장했습니다"), `course package download status mismatch: ${coursePackageDownloadState.statusText}`);
    assert(
      coursePackageDownloadState.statusText.includes("학생 자료 2개")
        && coursePackageDownloadState.statusText.includes("전체 자료 3개")
        && !coursePackageDownloadState.statusText.includes("학생 자료 3개"),
      `course package download material status mismatch: ${coursePackageDownloadState.statusText}`,
    );
    assert(coursePackageDownloadState.buttonText === "대표 묶음 저장", `course package download button should restore label: ${coursePackageDownloadState.buttonText}`);
    assert(coursePackageDownloadState.buttonDisabled === false, "course package download button should be re-enabled");
    assert(coursePackageDownloadedPayload.manifest?.package_id === copiedCoursePayload.manifest?.package_id, `downloaded course package id mismatch: ${coursePackageDownloadedPayload.manifest?.package_id}`);
    assert(coursePackageDownloadedPayload.manifest?.lesson_count === copiedCoursePayload.manifest?.lesson_count, `downloaded course package lesson count mismatch: ${coursePackageDownloadedPayload.manifest?.lesson_count}`);
    assert(coursePackageDownloadedPayload.manifest?.report_count === 3, `downloaded course package report count mismatch: ${coursePackageDownloadedPayload.manifest?.report_count}`);
    assert(
      coursePackageDownloadedPayload.manifest?.student_materials_summary?.some((item) => String(item).includes("학생 자료 2개")),
      `downloaded course package student summary mismatch: ${JSON.stringify(coursePackageDownloadedPayload.manifest?.student_materials_summary)}`,
    );
    assert(
      coursePackageDownloadedPayload.lessons?.map((lesson) => lesson.lesson_id).join("|") === copiedCoursePayload.lessons?.map((lesson) => lesson.lesson_id).join("|"),
      "downloaded course package lesson ids should match copied payload",
    );
    await page.click("[data-course-subject-filter='math']");
    await page.waitForFunction(() => {
      const cards = Array.from(document.querySelectorAll(".lesson-card"));
      return cards.length > 0 && cards.every((card) => card.textContent.includes("수학"));
    });
    const mathShortcutState = await page.evaluate(() => ({
      subjectValue: document.querySelector("#filter-subject")?.value ?? "",
      visibleLessonIds: Array.from(document.querySelectorAll(".lesson-card")).map((node) => node.dataset.lessonId || ""),
      cardSubjects: Array.from(document.querySelectorAll(".lesson-card .card-meta")).map((node) => node.textContent?.trim() || ""),
      summaryText: document.querySelector("[data-course-catalog-summary]")?.textContent?.trim() || "",
      shortcutTexts: Array.from(document.querySelectorAll("[data-course-subject-filter]")).map((node) => node.textContent?.trim() || ""),
      shortcutFilters: Array.from(document.querySelectorAll("[data-course-subject-filter]")).map((node) => node.getAttribute("data-course-subject-filter") || ""),
      activeShortcutText: document.querySelector("[data-course-subject-filter='math']")?.classList.contains("active")
        ? document.querySelector("[data-course-subject-filter='math']")?.textContent?.trim() || ""
        : "",
    }));
    assert(mathShortcutState.subjectValue === "math", `math subject shortcut did not sync select: ${mathShortcutState.subjectValue}`);
    assert(mathShortcutState.visibleLessonIds.length > 0, "math subject shortcut should keep at least one course visible");
    assert(
      mathShortcutState.cardSubjects.every((text) => text.includes("수학")),
      `math subject shortcut leaked non-math courses: ${mathShortcutState.cardSubjects.join("|")}`,
    );
    assert(mathShortcutState.summaryText.includes("대표 과목: 수학"), `math shortcut summary mismatch: ${mathShortcutState.summaryText}`);
    assert(
      mathShortcutState.shortcutFilters.includes("physics")
        && mathShortcutState.shortcutFilters.includes("math")
        && mathShortcutState.shortcutFilters.includes("econ")
        && mathShortcutState.shortcutTexts.some((text) => text.startsWith("물리 "))
        && mathShortcutState.shortcutTexts.some((text) => text.startsWith("경제 ")),
      `subject shortcuts should stay available after filtering: ${mathShortcutState.shortcutTexts.join("|")} filters=${mathShortcutState.shortcutFilters.join("|")}`,
    );
    assert(mathShortcutState.activeShortcutText.startsWith("수학 "), `math shortcut active state mismatch: ${mathShortcutState.activeShortcutText}`);
    await page.click("[data-course-subject-filter='__all__']");
    await page.waitForFunction(
      (officialCount) => document.querySelectorAll(".lesson-card").length === officialCount,
      defaultDevSurfaceState.visibleLessonIds.length,
    );
    await page.click(".browse-tab[data-tab='search']");
    await page.waitForFunction(
      (officialCount) => document.querySelectorAll(".lesson-card").length >= officialCount,
      defaultDevSurfaceState.visibleLessonIds.length,
    );
    const fullSearchState = await page.evaluate(() => ({
      visibleLessonIds: Array.from(document.querySelectorAll(".lesson-card")).map((node) => node.dataset.lessonId || ""),
      summaryVisible: document.querySelector("[data-course-catalog-summary]")?.classList.contains("hidden") === false,
    }));
    assert(
      fullSearchState.visibleLessonIds.length >= defaultDevSurfaceState.visibleLessonIds.length,
      `full search should include at least the representative lessons: ${fullSearchState.visibleLessonIds.join(",")}`,
    );
    assert(
      fullSearchState.visibleLessonIds.some((id) => String(id).includes("ddonirang") || String(id).startsWith("rep_cs_")),
      `full search should expose non-default allowlist lessons on demand: ${fullSearchState.visibleLessonIds.join(",")}`,
    );
    assert(fullSearchState.summaryVisible === false, "representative course summary should stay scoped to the official tab");
    await page.click(".browse-tab[data-tab='official']");
    await page.waitForSelector(".lesson-card[data-lesson-id='rep_physics_velocity_history_v1']");
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
      defaultDevSurfaceState.courseResultTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.courseResultTexts.every((text) => text.startsWith("결과 확인:") && text.length > "결과 확인:".length),
      `course result text mismatch: ${defaultDevSurfaceState.courseResultTexts.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.courseReadinessTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.courseReadinessTexts.every((text) => text.includes("수업 준비 완료") && text.includes("결과 보기") && text.includes("배포")),
      `course readiness text mismatch: ${defaultDevSurfaceState.courseReadinessTexts.join("|")}`,
    );
    assert(
      defaultDevSurfaceState.courseActivityTexts.length === defaultDevSurfaceState.visibleLessonIds.length
        && defaultDevSurfaceState.courseActivityTexts.every((text) => text.startsWith("오늘 활동:") && text.length > "오늘 활동:".length),
      `course activity text mismatch: ${defaultDevSurfaceState.courseActivityTexts.join("|")}`,
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
    await page.click(".lesson-card[data-lesson-id='rep_physics_velocity_history_v1']");
    const defaultLessonDetail = await page.evaluate(() => ({
      visible: !document.querySelector("#catalog-detail-panel")?.classList?.contains("hidden"),
      text: document.querySelector("#detail-curriculum")?.textContent ?? "",
      flowText: document.querySelector("[data-detail-course-flow]")?.textContent?.trim() ?? "",
      courseSequenceText: document.querySelector("[data-detail-course-sequence]")?.textContent?.trim() ?? "",
      flowItemCount: document.querySelectorAll("[data-detail-course-flow] .detail-course-flow-item").length,
      studentButtonText: document.querySelector("#btn-open-in-studio")?.textContent?.trim() ?? "",
      teacherButtonText: document.querySelector("#btn-open-in-studio-teacher")?.textContent?.trim() ?? "",
    }));
    assert(defaultLessonDetail.visible === true, "lesson detail panel should open from the default teacher catalog");
    assert(defaultLessonDetail.studentButtonText === "학생으로 실행", `detail student button mismatch: ${defaultLessonDetail.studentButtonText}`);
    assert(defaultLessonDetail.teacherButtonText === "교사용 배포 준비", `detail teacher button mismatch: ${defaultLessonDetail.teacherButtonText}`);
    assert(defaultLessonDetail.courseSequenceText === "추천 순서 1", `detail course sequence mismatch: ${defaultLessonDetail.courseSequenceText}`);
    assert(defaultLessonDetail.text.includes("학생 화면 미리보기"), `detail panel missing student preview: ${defaultLessonDetail.text}`);
    assert(defaultLessonDetail.text.includes("실행 버튼: 받은 수업 실행"), `detail panel missing student run CTA preview: ${defaultLessonDetail.text}`);
    assert(defaultLessonDetail.text.includes("학습목표"), `detail panel missing learning goals: ${defaultLessonDetail.text}`);
    assert(defaultLessonDetail.text.includes("수업 활동"), `detail panel missing classroom activities: ${defaultLessonDetail.text}`);
    assert(defaultLessonDetail.text.includes("결과 확인"), `detail panel missing result check section: ${defaultLessonDetail.text}`);
    assert(defaultLessonDetail.flowItemCount === 3, `detail course flow item count mismatch: ${defaultLessonDetail.flowItemCount}`);
    assert(
      defaultLessonDetail.flowText.includes("학생 활동")
        && defaultLessonDetail.flowText.includes("결과 확인")
        && defaultLessonDetail.flowText.includes("그래프")
        && defaultLessonDetail.flowText.includes("표")
        && defaultLessonDetail.flowText.includes("배포"),
      `detail course flow mismatch: ${defaultLessonDetail.flowText}`,
    );
    assert(defaultLessonDetail.text.includes("기록 길이가 이력 용량"), `detail panel missing velocity activity: ${defaultLessonDetail.text}`);
    await page.click("#btn-detail-close");
    await page.focus(".lesson-card[data-lesson-id='rep_physics_velocity_history_v1']");
    await page.keyboard.press("Enter");
    const keyboardLessonDetailVisible = await page.evaluate(() => !document.querySelector("#catalog-detail-panel")?.classList?.contains("hidden"));
    assert(keyboardLessonDetailVisible === true, "lesson detail panel should open from keyboard on focused card");
    await page.click("#btn-open-in-studio");
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
        advancedRunDisplay: display("#btn-advanced-run"),
        lessonBriefText: document.querySelector("[data-run-lesson-brief]")?.textContent?.trim() ?? "",
        lessonSummaryText: document.querySelector("#run-lesson-summary")?.textContent?.trim() ?? "",
        deliveryStatusDisplay: display("[data-run-delivery-status]"),
        deliveryStatusText: document.querySelector("[data-run-delivery-status]")?.textContent?.trim() ?? "",
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
    assert(studentStartUi.advancedRunDisplay === "none", `student advanced run button should be hidden: ${studentStartUi.advancedRunDisplay}`);
    assert(studentStartUi.deliveryStatusDisplay === "none", `normal student launch should not show package delivery status: ${studentStartUi.deliveryStatusDisplay}`);
    assert(studentStartUi.deliveryStatusText === "", `normal student launch delivery status should be empty: ${studentStartUi.deliveryStatusText}`);
    assert(
      studentStartUi.lessonBriefText.includes("속도 기록 그래프")
        && studentStartUi.lessonBriefText.includes("목표:")
        && studentStartUi.lessonBriefText.includes("활동:")
        && studentStartUi.lessonBriefText.includes("기록 길이")
        && studentStartUi.lessonBriefText.includes("결과 확인:")
        && studentStartUi.lessonBriefText.includes("그래프")
        && studentStartUi.lessonBriefText.includes("표"),
      `student lesson brief missing course activity context: ${studentStartUi.lessonBriefText}`,
    );
    assert(
      studentStartUi.lessonSummaryText.includes("결과 확인:")
        && studentStartUi.lessonSummaryText.includes("그래프")
        && studentStartUi.lessonSummaryText.includes("표"),
      `student lesson summary missing result check context: ${studentStartUi.lessonSummaryText}`,
    );
    assert(!studentStartUi.bodyText.includes("교과 원문"), "student mode should not show source text panel label");
    assert(!studentStartUi.bodyText.includes("저장 대기"), "student mode should not show save status");
    assert(!studentStartUi.bodyText.includes("세션 복원됨"), "student mode should not show session restore status");
    assert(!studentStartUi.bodyText.includes("+ 새로 만들기"), "student mode should not show new draft button");
    assert(!studentStartUi.bodyText.includes("배포 복사"), "student mode should not show teacher package copy button");
    await page.evaluate(() => {
      document.querySelector("#screen-run .main-shell-tab[data-main-tab-target='browse']")?.click();
    });
    await waitVisible(page, "#screen-browse");
    await page.click(".lesson-card[data-lesson-id='rep_physics_velocity_history_v1']");
    await page.click("#btn-open-in-studio-teacher");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.onboardingProfile === "teacher");
    await page.waitForFunction(() => document.querySelector("#run-tab-btn-mirror")?.classList?.contains("active"));
    await page.waitForFunction(() => document.querySelector("#run-inspector-tools")?.open === true);
    await page.waitForFunction(() => document.querySelector("#btn-run-teacher-package-copy")?.disabled === false);
    await page.waitForFunction(() => document.querySelector("#btn-run-local-package-guide-copy")?.disabled === false);
    await page.waitForFunction(() => document.querySelector("#btn-run-local-package-guide-download")?.disabled === false);
    await page.waitForFunction(() => document.querySelector("#btn-run-local-package-code-copy")?.disabled === false);
    await page.waitForFunction(() => document.querySelector("#btn-run-local-package-checklist-copy")?.disabled === false);
    await page.waitForFunction(() => document.querySelector("#btn-run-local-package-checklist-download")?.disabled === false);
    const classroomSessionLabel = "2학년 3반 4교시";
    await page.fill("#run-local-package-session-input", classroomSessionLabel);
    await page.waitForFunction(
      (session) => document.querySelector("[data-run-local-package-guide]")?.textContent?.includes(session),
      classroomSessionLabel,
    );
    await page.click("#btn-run-local-package-code-copy");
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_CODE_COPY_ACTION__?.copied === true);
    const packageCodeCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__STUDIO_LOCAL_PACKAGE_CODE_COPY_ACTION__ ?? null,
      codeText: document.querySelector("[data-run-local-package-code]")?.textContent?.trim() ?? "",
      codeReady: document.querySelector("[data-run-local-package-code]")?.dataset?.ready ?? "",
      codeDatasetPackageId: document.querySelector("[data-run-local-package-code]")?.dataset?.packageId ?? "",
      codeButtonReady: document.querySelector("#btn-run-local-package-code-copy")?.dataset?.ready ?? "",
      codeButtonPackageId: document.querySelector("#btn-run-local-package-code-copy")?.dataset?.packageId ?? "",
      codeButtonTitle: document.querySelector("#btn-run-local-package-code-copy")?.getAttribute("title") ?? "",
    }));
    assert(packageCodeCopyState.action?.schema === "seamgrim.local_package_code_copy_action.v1", "package code copy schema mismatch");
    assert(packageCodeCopyState.action?.copied === true, "package code copy action mismatch");
    assert(packageCodeCopyState.action?.session_label === classroomSessionLabel, `package code copy session mismatch: ${packageCodeCopyState.action?.session_label}`);
    assert(packageCodeCopyState.action?.multi_lesson === false, `single lesson package code should not be multi lesson: ${JSON.stringify(packageCodeCopyState.action)}`);
    assert(Array.isArray(packageCodeCopyState.action?.lesson_ids) && packageCodeCopyState.action.lesson_ids.join("|") === "rep_physics_velocity_history_v1", `single lesson package code ids mismatch: ${JSON.stringify(packageCodeCopyState.action)}`);
    assert(packageCodeCopyState.action?.account_required === false && packageCodeCopyState.action?.cloud_sync === false && packageCodeCopyState.action?.public_registry === false, "package code copy boundary mismatch");
    assert(packageCodeCopyState.copiedText === packageCodeCopyState.action?.package_id, `package code clipboard mismatch: ${packageCodeCopyState.copiedText}`);
    assert(packageCodeCopyState.codeText === `배포 코드: ${packageCodeCopyState.action?.package_id}`, `package code text mismatch: ${packageCodeCopyState.codeText}`);
    assert(packageCodeCopyState.codeReady === "1" && packageCodeCopyState.codeButtonReady === "1", "package code should be ready");
    assert(packageCodeCopyState.codeDatasetPackageId === packageCodeCopyState.action?.package_id, "package code element dataset mismatch");
    assert(packageCodeCopyState.codeButtonPackageId === packageCodeCopyState.action?.package_id, "package code button dataset mismatch");
    assert(packageCodeCopyState.codeButtonTitle.includes("배포 코드"), `package code button title mismatch: ${packageCodeCopyState.codeButtonTitle}`);
    await page.click("#btn-run-local-package-guide-copy");
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_COPY_ACTION__?.copied === true);
    const guideCopyState = await page.evaluate(() => ({
      guideClipboard: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      guideAction: window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_COPY_ACTION__ ?? null,
      guideButtonReady: document.querySelector("#btn-run-local-package-guide-copy")?.dataset?.ready ?? "",
      guideButtonTitle: document.querySelector("#btn-run-local-package-guide-copy")?.getAttribute("title") ?? "",
      guideDownloadButtonReady: document.querySelector("#btn-run-local-package-guide-download")?.dataset?.ready ?? "",
      guideDownloadButtonTitle: document.querySelector("#btn-run-local-package-guide-download")?.getAttribute("title") ?? "",
      guideDownloadButtonText: document.querySelector("#btn-run-local-package-guide-download")?.textContent?.trim() ?? "",
      checklistText: document.querySelector("[data-run-local-package-checklist]")?.textContent?.trim() ?? "",
      checklistReady: document.querySelector("[data-run-local-package-checklist]")?.dataset?.ready ?? "",
      checklistCopyReady: document.querySelector("#btn-run-local-package-checklist-copy")?.dataset?.ready ?? "",
      checklistCopyTitle: document.querySelector("#btn-run-local-package-checklist-copy")?.getAttribute("title") ?? "",
      checklistDownloadReady: document.querySelector("#btn-run-local-package-checklist-download")?.dataset?.ready ?? "",
      checklistDownloadTitle: document.querySelector("#btn-run-local-package-checklist-download")?.getAttribute("title") ?? "",
    }));
    assert(guideCopyState.guideButtonReady === "1", `student guide copy readiness mismatch: ${guideCopyState.guideButtonReady}`);
    assert(guideCopyState.guideButtonTitle.includes("배포 열기 안내문"), `student guide copy title mismatch: ${guideCopyState.guideButtonTitle}`);
    assert(guideCopyState.guideDownloadButtonReady === "1", `student guide download readiness mismatch: ${guideCopyState.guideDownloadButtonReady}`);
    assert(guideCopyState.guideDownloadButtonTitle.includes("배포 열기 안내문"), `student guide download title mismatch: ${guideCopyState.guideDownloadButtonTitle}`);
    assert(guideCopyState.guideDownloadButtonText === "안내 저장", `student guide download label mismatch: ${guideCopyState.guideDownloadButtonText}`);
    assert(guideCopyState.checklistText.includes("교사용 준비") && guideCopyState.checklistText.includes(classroomSessionLabel), `teacher checklist preview mismatch: ${guideCopyState.checklistText}`);
    assert(guideCopyState.checklistReady === "1", `teacher checklist readiness mismatch: ${guideCopyState.checklistReady}`);
    assert(guideCopyState.checklistCopyReady === "1" && guideCopyState.checklistDownloadReady === "1", "teacher checklist buttons should be ready");
    assert(guideCopyState.checklistCopyTitle.includes("준비 체크리스트"), `teacher checklist copy title mismatch: ${guideCopyState.checklistCopyTitle}`);
    assert(guideCopyState.checklistDownloadTitle.includes("준비 체크리스트"), `teacher checklist download title mismatch: ${guideCopyState.checklistDownloadTitle}`);
    assert(guideCopyState.guideAction?.schema === "seamgrim.local_package_student_guide_copy_action.v1", "student guide copy schema mismatch");
    assert(guideCopyState.guideAction?.copied === true, "student guide copy action mismatch");
    assert(guideCopyState.guideAction?.session_label === classroomSessionLabel, `student guide copy session mismatch: ${guideCopyState.guideAction?.session_label}`);
    assert(guideCopyState.guideAction?.multi_lesson === false, `single lesson student guide should not be multi lesson: ${JSON.stringify(guideCopyState.guideAction)}`);
    assert(Array.isArray(guideCopyState.guideAction?.lesson_ids) && guideCopyState.guideAction.lesson_ids.join("|") === "rep_physics_velocity_history_v1", `single lesson student guide ids mismatch: ${JSON.stringify(guideCopyState.guideAction)}`);
    assert(guideCopyState.guideAction?.account_required === false && guideCopyState.guideAction?.cloud_sync === false && guideCopyState.guideAction?.public_registry === false, "student guide copy boundary mismatch");
    assert(
      guideCopyState.guideClipboard.includes("배포 안내")
        && guideCopyState.guideClipboard.includes(`차시: ${classroomSessionLabel}`)
        && guideCopyState.guideClipboard.includes("수업: 속도 기록 그래프")
        && guideCopyState.guideClipboard.includes(`수업 코드: ${guideCopyState.guideAction?.lesson_id}`)
        && guideCopyState.guideClipboard.includes("목표:")
        && guideCopyState.guideClipboard.includes("오늘 활동:")
        && guideCopyState.guideClipboard.includes("배포 열기")
        && guideCopyState.guideClipboard.includes("붙여넣기 열기")
        && guideCopyState.guideClipboard.includes("JSON 배포 파일")
        && guideCopyState.guideClipboard.includes("받은 수업 실행")
        && guideCopyState.guideClipboard.includes("결과 확인: 그래프, 표")
        && guideCopyState.guideClipboard.includes("결과 복사")
        && guideCopyState.guideClipboard.includes("제출 결과에는 학생 이름, 차시, 수업 코드, 배포 코드, 상태 기록")
        && guideCopyState.guideClipboard.includes(`배포 코드: ${guideCopyState.guideAction?.package_id}`),
      `student guide clipboard mismatch: ${guideCopyState.guideClipboard}`,
    );
    const [guideDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-local-package-guide-download"),
    ]);
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_DOWNLOAD_ACTION__?.downloaded === true);
    const guideDownloadPath = await guideDownload.path();
    const guideDownloadedText = guideDownloadPath ? await fs.readFile(guideDownloadPath, "utf-8") : "";
    const guideDownloadState = await page.evaluate(() => ({
      action: window.__STUDIO_LOCAL_PACKAGE_STUDENT_GUIDE_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(guideDownloadState.action?.schema === "seamgrim.local_package_student_guide_download_action.v1", "student guide download schema mismatch");
    assert(guideDownloadState.action?.downloaded === true, "student guide download action mismatch");
    assert(guideDownloadState.action?.lesson_id === "rep_physics_velocity_history_v1", `student guide download lesson mismatch: ${guideDownloadState.action?.lesson_id}`);
    assert(guideDownloadState.action?.session_label === classroomSessionLabel, `student guide download session mismatch: ${guideDownloadState.action?.session_label}`);
    assert(guideDownloadState.action?.multi_lesson === false, `single lesson student guide download should not be multi lesson: ${JSON.stringify(guideDownloadState.action)}`);
    assert(Array.isArray(guideDownloadState.action?.lesson_ids) && guideDownloadState.action.lesson_ids.join("|") === "rep_physics_velocity_history_v1", `single lesson student guide download ids mismatch: ${JSON.stringify(guideDownloadState.action)}`);
    assert(guideDownloadState.action?.account_required === false && guideDownloadState.action?.cloud_sync === false && guideDownloadState.action?.public_registry === false, "student guide download boundary mismatch");
    assert(String(guideDownloadState.action?.file_name ?? "").endsWith(".txt"), `student guide download filename mismatch: ${guideDownloadState.action?.file_name}`);
    assert(String(guideDownloadState.action?.file_name ?? "").includes(guideCopyState.guideAction?.package_id), `student guide download filename should include package id: ${guideDownloadState.action?.file_name}`);
    assert(String(guideDownloadState.action?.file_name ?? "").includes("_4_"), `student guide download filename should include session marker: ${guideDownloadState.action?.file_name}`);
    assert(guideDownload.suggestedFilename() === guideDownloadState.action.file_name, "student guide suggested filename mismatch");
    assert(guideDownloadedText.trim() === guideCopyState.guideClipboard.trim(), "student guide downloaded text mismatch");
    await page.click("#btn-run-local-package-checklist-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_COPY_ACTION__?.copied === true);
    const checklistCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_COPY_ACTION__ ?? null,
    }));
    assert(checklistCopyState.action?.schema === "seamgrim.teacher_preparation_checklist_copy_action.v1", "teacher checklist copy schema mismatch");
    assert(checklistCopyState.action?.copied === true, "teacher checklist copy action mismatch");
    assert(checklistCopyState.action?.session_label === classroomSessionLabel, `teacher checklist copy session mismatch: ${checklistCopyState.action?.session_label}`);
    assert(checklistCopyState.action?.multi_lesson === false, `single lesson checklist copy should not be multi lesson: ${JSON.stringify(checklistCopyState.action)}`);
    assert(Array.isArray(checklistCopyState.action?.lesson_ids) && checklistCopyState.action.lesson_ids.join("|") === "rep_physics_velocity_history_v1", `single lesson checklist copy ids mismatch: ${JSON.stringify(checklistCopyState.action)}`);
    assert(
      checklistCopyState.copiedText.includes("셈그림 교사용 배포 준비 체크리스트")
        && checklistCopyState.copiedText.includes(`차시: ${classroomSessionLabel}`)
        && checklistCopyState.copiedText.includes("배포 전")
        && checklistCopyState.copiedText.includes("수업 중")
        && checklistCopyState.copiedText.includes("수업 후")
        && checklistCopyState.copiedText.includes("학생 명단 양식"),
      `teacher checklist copy text mismatch: ${checklistCopyState.copiedText}`,
    );
    const [checklistDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-local-package-checklist-download"),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_DOWNLOAD_ACTION__?.downloaded === true);
    const checklistDownloadPath = await checklistDownload.path();
    const checklistDownloadedText = checklistDownloadPath ? await fs.readFile(checklistDownloadPath, "utf-8") : "";
    const checklistDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_TEACHER_PREPARATION_CHECKLIST_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(checklistDownloadState.action?.schema === "seamgrim.teacher_preparation_checklist_download_action.v1", "teacher checklist download schema mismatch");
    assert(checklistDownloadState.action?.downloaded === true, "teacher checklist download action mismatch");
    assert(checklistDownloadState.action?.session_label === classroomSessionLabel, `teacher checklist download session mismatch: ${checklistDownloadState.action?.session_label}`);
    assert(checklistDownloadState.action?.multi_lesson === false, `single lesson checklist download should not be multi lesson: ${JSON.stringify(checklistDownloadState.action)}`);
    assert(Array.isArray(checklistDownloadState.action?.lesson_ids) && checklistDownloadState.action.lesson_ids.join("|") === "rep_physics_velocity_history_v1", `single lesson checklist download ids mismatch: ${JSON.stringify(checklistDownloadState.action)}`);
    assert(String(checklistDownloadState.action?.file_name ?? "").endsWith("_teacher_preparation_checklist.txt"), `teacher checklist filename mismatch: ${checklistDownloadState.action?.file_name}`);
    assert(checklistDownload.suggestedFilename() === checklistDownloadState.action.file_name, "teacher checklist suggested filename mismatch");
    assert(checklistDownloadedText.trim() === checklistCopyState.copiedText.trim(), "teacher checklist downloaded text mismatch");
    await page.click("#btn-run-teacher-package-copy");
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_EXPORT_ACTION__?.copied === true);
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-local-package-download"),
    ]);
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_DOWNLOAD_ACTION__?.downloaded === true);

    const state = await page.evaluate(() => ({
      schema: document.querySelector("[data-run-local-package-export]")?.dataset?.schema ?? "",
      state: document.querySelector("[data-run-local-package-export]")?.dataset?.state ?? "",
      packageId: document.querySelector("[data-run-local-package-export]")?.dataset?.packageId ?? "",
      meta: document.querySelector("[data-run-local-package-meta]")?.textContent?.trim() ?? "",
      metaValue: document.querySelector("[data-run-local-package-meta]")?.dataset?.value ?? "",
      guideText: document.querySelector("[data-run-local-package-guide]")?.textContent?.trim() ?? "",
      guideReady: document.querySelector("[data-run-local-package-guide]")?.dataset?.ready ?? "",
      materialsText: document.querySelector("[data-run-local-package-materials]")?.textContent?.trim() ?? "",
      materialsLessonCount: document.querySelector("[data-run-local-package-materials]")?.dataset?.lessonCount ?? "",
      materialsReportCount: document.querySelector("[data-run-local-package-materials]")?.dataset?.reportCount ?? "",
      codeText: document.querySelector("[data-run-local-package-code]")?.textContent?.trim() ?? "",
      codeReady: document.querySelector("[data-run-local-package-code]")?.dataset?.ready ?? "",
      codeButtonReady: document.querySelector("#btn-run-local-package-code-copy")?.dataset?.ready ?? "",
      indexText: document.querySelector("[data-run-local-package-text]")?.textContent ?? "",
      presetViewsText: document.querySelector("[data-run-preset-views]")?.textContent?.trim() ?? "",
      saveButtonText: document.querySelector("#btn-run-teacher-package-download")?.textContent?.trim() ?? "",
      packagePanelSaveButtonText: document.querySelector("#btn-run-local-package-download")?.textContent?.trim() ?? "",
      packagePanelSaveButtonFileCount: document.querySelector("#btn-run-local-package-download")?.dataset?.fileCount ?? "",
      sessionInputValue: document.querySelector("#run-local-package-session-input")?.value ?? "",
      activeRunTab: Array.from(document.querySelectorAll("[id^='run-tab-btn-']")).find((node) => node.classList.contains("active"))?.id ?? "",
      packageToolsOpen: document.querySelector("#run-inspector-tools")?.open === true,
      execTechDisplay: getComputedStyle(document.querySelector("#run-exec-tech")).display,
      diagnosticsDisplay: getComputedStyle(document.querySelector("#run-mirror-diagnostics")).display,
      inspectorMetaDisplay: getComputedStyle(document.querySelector("#run-inspector-meta")).display,
      inspectorToolsSummaryDisplay: getComputedStyle(document.querySelector("#run-inspector-tools > summary")).display,
      inspectorActionsDisplay: getComputedStyle(document.querySelector("#run-inspector-tools > .run-inspector-actions")).display,
      classroomReportDisplay: getComputedStyle(document.querySelector("[data-run-classroom-report-export]")).display,
      localPackageDisplay: getComputedStyle(document.querySelector("[data-run-local-package-export]")).display,
      classroomReportWhiteSpace: getComputedStyle(document.querySelector("[data-run-classroom-report-text]")).whiteSpace,
      localPackageWhiteSpace: getComputedStyle(document.querySelector("[data-run-local-package-text]")).whiteSpace,
      localPackageGuideWhiteSpace: getComputedStyle(document.querySelector("[data-run-local-package-guide]")).whiteSpace,
      teacherVisibleText: document.querySelector("#screen-run")?.innerText ?? "",
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
    assert(state.activeRunTab === "run-tab-btn-mirror", `teacher package flow should land on mirror tab: ${state.activeRunTab}`);
    assert(state.packageToolsOpen === true, "teacher package tools should open automatically from browse teacher launch");
    assert(state.execTechDisplay === "none", `teacher package flow should hide technical execution details: ${state.execTechDisplay}`);
    assert(state.diagnosticsDisplay === "none", `teacher package flow should hide diagnostics detail: ${state.diagnosticsDisplay}`);
    assert(state.inspectorMetaDisplay === "none", `teacher package flow should hide inspector metadata: ${state.inspectorMetaDisplay}`);
    assert(state.inspectorToolsSummaryDisplay === "none", `teacher package flow should hide inspector tool header: ${state.inspectorToolsSummaryDisplay}`);
    assert(state.inspectorActionsDisplay === "none", `teacher package flow should hide save/session actions: ${state.inspectorActionsDisplay}`);
    assert(state.classroomReportDisplay !== "none", `teacher package flow should keep classroom report visible: ${state.classroomReportDisplay}`);
    assert(state.localPackageDisplay !== "none", `teacher package flow should keep local package export visible: ${state.localPackageDisplay}`);
    assert(state.classroomReportWhiteSpace === "pre-wrap", `teacher report should wrap on narrow screens: ${state.classroomReportWhiteSpace}`);
    assert(state.localPackageWhiteSpace === "pre-wrap", `teacher package index should wrap on narrow screens: ${state.localPackageWhiteSpace}`);
    assert(state.localPackageGuideWhiteSpace === "pre-wrap", `student guide summary should preserve lines: ${state.localPackageGuideWhiteSpace}`);
    ["기술 상세", "검증 정보", "bridge", "view warn", "저장/내보내기", "세션 저장", "저장 작업 대기"].forEach((text) => {
      assert(!state.teacherVisibleText.includes(text), `teacher package flow should not expose technical text: ${text}`);
    });
    assert(
      state.meta.includes("교사 시작")
        && state.meta.includes("Studio 배포 열기용")
        && state.meta.includes("교과 1개")
        && state.meta.includes("교사용 자료 4개"),
      `meta mismatch: ${state.meta}`,
    );
    assert(Number(state.metaValue) >= 5, `meta value mismatch: ${state.metaValue}`);
    assert(
      state.materialsText.includes("포함 자료:")
        && state.materialsText.includes("교과 1개")
        && state.materialsText.includes("수업 리포트")
        && state.materialsText.includes("학생 배포 안내문")
        && state.materialsText.includes("교사용 준비 체크리스트")
        && state.materialsText.includes("학생 명단 양식"),
      `package materials summary mismatch: ${state.materialsText}`,
    );
    assert(state.materialsLessonCount === "1" && state.materialsReportCount === "4", `package materials counts mismatch: ${state.materialsLessonCount}/${state.materialsReportCount}`);
    assert(state.codeText === `배포 코드: ${downloadedPayload.manifest.package_id}`, `package code display mismatch: ${state.codeText}`);
    assert(state.codeReady === "1" && state.codeButtonReady === "1", "package code display should be ready");
    assert(state.presetViewsText === "결과 확인: 그래프, 표", `teacher preset views label mismatch: ${state.presetViewsText}`);
    assert(state.sessionInputValue === classroomSessionLabel, `teacher package session input mismatch: ${state.sessionInputValue}`);
    assert(
      state.guideText.includes("학생 배포 안내문")
        && state.guideText.includes(`차시: ${classroomSessionLabel}`)
        && state.guideText.includes("수업: 속도 기록 그래프")
        && state.guideText.includes("수업 코드:")
        && state.guideText.includes("목표:")
        && state.guideText.includes("오늘 활동:")
        && state.guideText.includes("배포 코드:")
        && state.guideText.includes("배포 열기")
        && state.guideText.includes("붙여넣기 열기")
        && state.guideText.includes("받은 수업 실행")
        && state.guideText.includes("결과 확인: 그래프, 표")
        && state.guideText.includes("결과 복사"),
      `student guide text mismatch: ${state.guideText}`,
    );
    assert(state.guideReady === "1", `student guide readiness mismatch: ${state.guideReady}`);
    assert(state.indexText.startsWith("구분\t경로\t제목\t크기"), "index header mismatch");
    assert(
      state.indexText.includes("\nguide\t학생 배포 안내문\t배포 열기/붙여넣기 열기 → 받은 수업 실행 · 결과 확인: 그래프, 표를 확인합니다\t0"),
      `student guide row missing result summary:\n${state.indexText}`,
    );
    assert(state.indexText.includes("\nlesson\tlessons/"), `lesson index row missing:\n${state.indexText}`);
    assert(state.indexText.includes("\nreport\treports/"), `report index row missing:\n${state.indexText}`);
    assert(state.saveButtonText.startsWith("배포 저장 "), `save button text mismatch: ${state.saveButtonText}`);
    assert(state.packagePanelSaveButtonText.startsWith("패키지 저장 "), `package panel save button text mismatch: ${state.packagePanelSaveButtonText}`);
    assert(state.packagePanelSaveButtonFileCount === String(state.downloadPayload?.file_count ?? ""), `package panel save file count mismatch: ${state.packagePanelSaveButtonFileCount}`);
    assert(copiedPayload.__종류 === "studio_local_package_payload", "clipboard payload kind mismatch");
    assert(copiedPayload.manifest?.__종류 === "studio_local_package_manifest", "clipboard manifest kind mismatch");
    assert(state.payload?.multi_lesson === false, `single lesson package copy should not be multi lesson: ${JSON.stringify(state.payload)}`);
    assert(Array.isArray(state.payload?.lesson_ids) && state.payload.lesson_ids.join("|") === downloadedPayload.lessons[0].lesson_id, `single lesson package copy ids mismatch: ${JSON.stringify(state.payload?.lesson_ids)}`);
    assert(state.downloadPayload?.multi_lesson === false, `single lesson package download should not be multi lesson: ${JSON.stringify(state.downloadPayload)}`);
    assert(Array.isArray(state.downloadPayload?.lesson_ids) && state.downloadPayload.lesson_ids.join("|") === downloadedPayload.lessons[0].lesson_id, `single lesson package download ids mismatch: ${JSON.stringify(state.downloadPayload?.lesson_ids)}`);
    assert(copiedPayload.manifest?.account_required === false, "account boundary mismatch");
    assert(copiedPayload.manifest?.cloud_sync === false, "cloud boundary mismatch");
    assert(copiedPayload.manifest?.public_registry === false, "registry boundary mismatch");
    assert(copiedPayload.manifest?.session_label === classroomSessionLabel, `clipboard manifest session mismatch: ${copiedPayload.manifest?.session_label}`);
    assert(copiedPayload.delivery_mode === "studio_json_import", `download delivery mode mismatch: ${copiedPayload.delivery_mode}`);
    assert(copiedPayload.open_with === "seamgrim_studio_local_package_import", `download open_with mismatch: ${copiedPayload.open_with}`);
    assert(copiedPayload.student_entry_label === "배포 열기", `download entry label mismatch: ${copiedPayload.student_entry_label}`);
    assert(
      Array.isArray(copiedPayload.student_instructions)
        && copiedPayload.student_instructions.some((item) => String(item).includes(`차시: ${classroomSessionLabel}`))
        && copiedPayload.student_instructions.some((item) => String(item).includes("수업: 속도 기록 그래프"))
        && copiedPayload.student_instructions.some((item) => String(item).includes(`수업 코드: ${downloadedPayload?.lessons?.[0]?.lesson_id ?? "rep_physics_velocity_history_v1"}`))
        && copiedPayload.student_instructions.some((item) => String(item).includes("목표:"))
        && copiedPayload.student_instructions.some((item) => String(item).includes("오늘 활동:"))
        && copiedPayload.student_instructions.some((item) => String(item).includes("배포 열기"))
        && copiedPayload.student_instructions.some((item) => String(item).includes("붙여넣기 열기"))
        && copiedPayload.student_instructions.some((item) => String(item).includes("결과 확인: 그래프, 표"))
        && copiedPayload.student_instructions.some((item) => String(item).includes("결과 복사"))
        && copiedPayload.student_instructions.some((item) => String(item).includes(`배포 코드: ${copiedPayload.manifest?.package_id}`)),
      `download student instructions missing: ${JSON.stringify(copiedPayload.student_instructions)}`,
    );
    assert(copiedPayload.manifest?.delivery_mode === "studio_json_import", `manifest delivery mode mismatch: ${copiedPayload.manifest?.delivery_mode}`);
    assert(copiedPayload.manifest?.open_with === "seamgrim_studio_local_package_import", `manifest open_with mismatch: ${copiedPayload.manifest?.open_with}`);
    assert(
      Array.isArray(copiedPayload.manifest?.student_instructions)
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes(`차시: ${classroomSessionLabel}`))
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes("수업: 속도 기록 그래프"))
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes(`수업 코드: ${downloadedPayload?.lessons?.[0]?.lesson_id ?? "rep_physics_velocity_history_v1"}`))
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes("목표:"))
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes("오늘 활동:"))
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes("붙여넣기 열기"))
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes("결과 확인: 그래프, 표"))
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes("결과 복사"))
        && copiedPayload.manifest.student_instructions.some((item) => String(item).includes(`배포 코드: ${copiedPayload.manifest?.package_id}`)),
      `download manifest student instructions missing: ${JSON.stringify(copiedPayload.manifest?.student_instructions)}`,
    );
    assert(copiedPayload.lessons?.[0]?.subject === "physics", `package lesson subject mismatch: ${copiedPayload.lessons?.[0]?.subject}`);
    assert(copiedPayload.lessons?.[0]?.grade === "middle", `package lesson grade mismatch: ${copiedPayload.lessons?.[0]?.grade}`);
    assert(
      Array.isArray(copiedPayload.lessons?.[0]?.required_views)
        && copiedPayload.lessons[0].required_views.includes("graph")
        && copiedPayload.lessons[0].required_views.includes("table"),
      `package lesson views mismatch: ${JSON.stringify(copiedPayload.lessons?.[0]?.required_views)}`,
    );
    assert(
      Array.isArray(copiedPayload.lessons?.[0]?.goals)
        && copiedPayload.lessons[0].goals.some((item) => String(item).includes("속도가 일정한 가속도")),
      `package lesson goals missing: ${JSON.stringify(copiedPayload.lessons?.[0]?.goals)}`,
    );
    assert(
      Array.isArray(copiedPayload.lessons?.[0]?.missions)
        && copiedPayload.lessons[0].missions.some((item) => String(item).includes("기록 길이")),
      `package lesson missions missing: ${JSON.stringify(copiedPayload.lessons?.[0]?.missions)}`,
    );
    assert(state.payload?.schema === "seamgrim.local_package_export_action.v1", "instrumentation schema mismatch");
    assert(state.payload?.copied === true, "instrumentation copied mismatch");
    assert(state.payload?.lesson_count === 1 && state.payload?.report_count === 4, "instrumentation counts mismatch");
    assert(state.payload?.account_required === false && state.payload?.cloud_sync === false && state.payload?.public_registry === false && state.payload?.remote_save === false, "instrumentation boundary mismatch");
    assert(state.payload?.payload_text.trim() === state.copied, "payload text clipboard mismatch");
    assert(state.downloadPayload?.downloaded === true, "download instrumentation mismatch");
    assert(String(state.downloadPayload?.file_name ?? "").endsWith(".json"), "download filename mismatch");
    assert(String(state.downloadPayload?.file_name ?? "").includes(state.downloadPayload?.package_id), `download filename should include package id: ${state.downloadPayload?.file_name}`);
    assert(String(state.downloadPayload?.file_name ?? "").includes("_2_3_4"), `download filename should include session marker: ${state.downloadPayload?.file_name}`);
    assert(state.downloadPayload?.lesson_id === downloadedPayload.lessons[0].lesson_id, `download instrumentation lesson id mismatch: ${state.downloadPayload?.lesson_id}`);
    assert(state.downloadPayload?.package_id === downloadedPayload.manifest.package_id, `download instrumentation package id mismatch: ${state.downloadPayload?.package_id}`);
    assert(state.downloadPayload?.session_label === classroomSessionLabel, `download instrumentation session mismatch: ${state.downloadPayload?.session_label}`);
    assert(download.suggestedFilename() === state.downloadPayload.file_name, "suggested filename mismatch");
    assert(downloadedPayload.__종류 === "studio_local_package_payload", "downloaded payload kind mismatch");
    assert(downloadedPayload.manifest?.session_label === classroomSessionLabel, `downloaded manifest session mismatch: ${downloadedPayload.manifest?.session_label}`);
    assert(
      Array.isArray(downloadedPayload.manifest?.materials_summary)
        && downloadedPayload.manifest.materials_summary.some((item) => String(item).includes("교과 1개") && String(item).includes("속도 기록 그래프"))
        && downloadedPayload.manifest.materials_summary.some((item) => String(item).includes("교사용 자료 4개") && String(item).includes("학생 명단 양식")),
      `downloaded materials summary mismatch: ${JSON.stringify(downloadedPayload.manifest?.materials_summary)}`,
    );
    assert(
      Array.isArray(downloadedPayload.manifest?.student_materials_summary)
        && downloadedPayload.manifest.student_materials_summary.some((item) => String(item).includes("교과 1개") && String(item).includes("속도 기록 그래프"))
        && downloadedPayload.manifest.student_materials_summary.some((item) => String(item).includes("학생 자료 2개") && String(item).includes("학생 배포 안내문") && String(item).includes("학생 명단 양식"))
        && !downloadedPayload.manifest.student_materials_summary.some((item) => String(item).includes("교사용 자료")),
      `downloaded student materials summary mismatch: ${JSON.stringify(downloadedPayload.manifest?.student_materials_summary)}`,
    );
    assert(
      Array.isArray(downloadedPayload.student_materials_summary)
        && downloadedPayload.student_materials_summary.join("\n") === downloadedPayload.manifest.student_materials_summary.join("\n"),
      `downloaded payload student materials summary mismatch: ${JSON.stringify(downloadedPayload.student_materials_summary)}`,
    );
    assert(downloadedText.trim() === state.copied, "downloaded payload text mismatch");
    assert(
      String(downloadedPayload.reports?.[0]?.text ?? "").startsWith("수업 코드\t수업 제목\t수업 목표\t오늘 활동\t결과 확인\t배포 상태\t실행 결과\t확인 필요\t비고"),
      `downloaded report header mismatch: ${downloadedPayload.reports?.[0]?.text}`,
    );
    assert(
      String(downloadedPayload.reports?.[0]?.text ?? "").includes("속도가 일정한 가속도")
        && String(downloadedPayload.reports?.[0]?.text ?? "").includes("기록 길이")
        && String(downloadedPayload.reports?.[0]?.text ?? "").includes("그래프, 표"),
      `downloaded report should preserve teacher course context: ${downloadedPayload.reports?.[0]?.text}`,
    );
    const studentGuideReport = downloadedPayload.reports?.find((report) => String(report?.report_id ?? "").includes("student_guide"));
    assert(studentGuideReport, `downloaded package missing student guide report: ${JSON.stringify(downloadedPayload.reports)}`);
    assert(
      String(studentGuideReport?.title ?? "").includes("학생 배포 안내문")
        && String(studentGuideReport?.path ?? "").endsWith("_student_guide.txt")
        && String(studentGuideReport?.path ?? "").includes(downloadedPayload.manifest.package_id)
        && String(studentGuideReport?.text ?? "").includes("배포 안내")
        && String(studentGuideReport?.text ?? "").includes(`차시: ${classroomSessionLabel}`)
        && String(studentGuideReport?.text ?? "").includes(`수업 코드: ${downloadedPayload.lessons[0].lesson_id}`)
        && String(studentGuideReport?.text ?? "").includes(`배포 코드: ${downloadedPayload.manifest.package_id}`)
        && String(studentGuideReport?.text ?? "").includes("배포 열기")
        && String(studentGuideReport?.text ?? "").includes("결과 복사"),
      `downloaded student guide report mismatch: ${JSON.stringify(studentGuideReport)}`,
    );
    const teacherChecklistReport = downloadedPayload.reports?.find((report) => String(report?.report_id ?? "").includes("teacher_preparation_checklist"));
    assert(teacherChecklistReport, `downloaded package missing teacher checklist report: ${JSON.stringify(downloadedPayload.reports)}`);
    assert(
      String(teacherChecklistReport?.title ?? "").includes("교사용 준비 체크리스트")
        && String(teacherChecklistReport?.path ?? "").endsWith("_teacher_preparation_checklist.txt")
        && String(teacherChecklistReport?.text ?? "").includes("셈그림 교사용 배포 준비 체크리스트")
        && String(teacherChecklistReport?.text ?? "").includes(`수업 코드: ${downloadedPayload.lessons[0].lesson_id}`)
        && String(teacherChecklistReport?.text ?? "").includes(`차시: ${classroomSessionLabel}`)
        && String(teacherChecklistReport?.text ?? "").includes(`배포 코드: ${downloadedPayload.manifest.package_id}`)
        && String(teacherChecklistReport?.text ?? "").includes("배포 전")
        && String(teacherChecklistReport?.text ?? "").includes("수업 중")
        && String(teacherChecklistReport?.text ?? "").includes("수업 후"),
      `downloaded teacher checklist report mismatch: ${JSON.stringify(teacherChecklistReport)}`,
    );
    const rosterTemplateReport = downloadedPayload.reports?.find((report) => String(report?.report_id ?? "").includes("student_roster_template"));
    assert(rosterTemplateReport, `downloaded package missing roster template report: ${JSON.stringify(downloadedPayload.reports)}`);
    assert(
      String(rosterTemplateReport?.title ?? "").includes("학생 명단 양식")
        && String(rosterTemplateReport?.path ?? "").endsWith("_student_roster_template.tsv")
        && String(rosterTemplateReport?.mime ?? "").includes("text/tab-separated-values")
        && String(rosterTemplateReport?.text ?? "").includes("# 셈그림 학생 명단 양식")
        && String(rosterTemplateReport?.text ?? "").includes(`수업 코드\t${downloadedPayload.lessons[0].lesson_id}`)
        && String(rosterTemplateReport?.text ?? "").includes(`차시\t${classroomSessionLabel}`)
        && String(rosterTemplateReport?.text ?? "").includes(`배포 코드\t${downloadedPayload.manifest.package_id}`)
        && String(rosterTemplateReport?.text ?? "").includes("번호\t이름"),
      `downloaded roster template report mismatch: ${JSON.stringify(rosterTemplateReport)}`,
    );
    assert(
      state.indexText.includes("학생 명단 양식") && state.indexText.includes("student_roster_template"),
      `package index should list roster template report:\n${state.indexText}`,
    );
    assert(
      state.indexText.includes("교사용 준비 체크리스트") && state.indexText.includes("teacher_preparation_checklist"),
      `package index should list teacher checklist report:\n${state.indexText}`,
    );
    assert(
      state.indexText.includes("학생 배포 안내문") && state.indexText.includes("student_guide"),
      `package index should list student guide report:\n${state.indexText}`,
    );

    await page.click("#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target='browse']");
    await waitVisible(page, "#screen-browse");
    await page.click("#btn-paste-local-package");
    await page.waitForFunction(() => getComputedStyle(document.querySelector("[data-local-package-paste-panel]")).display !== "none");
    await page.fill("#local-package-paste-text", state.copied);
    const validPastePreviewState = await page.evaluate(() => ({
      previewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      previewState: document.querySelector("[data-local-package-paste-preview]")?.dataset?.state ?? "",
      previewDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-preview]")).display,
    }));
    assert(validPastePreviewState.previewState === "ok", `valid paste preview state mismatch: ${validPastePreviewState.previewState}`);
    assert(validPastePreviewState.previewDisplay !== "none", `valid paste preview should be visible: ${validPastePreviewState.previewDisplay}`);
    assert(validPastePreviewState.previewText.includes(downloadedPayload.manifest.title), `valid paste preview missing title: ${validPastePreviewState.previewText}`);
    assert(validPastePreviewState.previewText.includes("실행할 수업 속도 기록 그래프"), `valid paste preview missing lesson title: ${validPastePreviewState.previewText}`);
    assert(validPastePreviewState.previewText.includes(classroomSessionLabel), `valid paste preview missing session: ${validPastePreviewState.previewText}`);
    assert(validPastePreviewState.previewText.includes("교과 1개"), `valid paste preview missing lesson count: ${validPastePreviewState.previewText}`);
    assert(validPastePreviewState.previewText.includes("학생 자료 2개"), `valid paste preview missing student material count: ${validPastePreviewState.previewText}`);
    assert(validPastePreviewState.previewText.includes("학생 명단 양식"), `valid paste preview missing material title: ${validPastePreviewState.previewText}`);
    assert(!validPastePreviewState.previewText.includes("교사용 자료"), `valid paste preview should not expose teacher material label: ${validPastePreviewState.previewText}`);
    await page.keyboard.press("Control+Enter");
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.file_name === "pasted-teacher-package.json" && window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true);
    await waitVisible(page, "#screen-run");
    const pasteImportState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      rail: window.__SEAMGRIM_RUN_PRESET_RAIL__ ?? null,
      runText: document.querySelector("#run-ddn-preview")?.value ?? "",
      deliveryStatusText: document.querySelector("[data-run-delivery-status]")?.textContent?.trim() ?? "",
      deliveryInstructionsText: document.querySelector("[data-run-delivery-instructions]")?.textContent?.trim() ?? "",
      pastePanelOpen: document.querySelector("[data-local-package-paste-panel]")?.dataset?.open ?? "",
      pasteTextValue: document.querySelector("#local-package-paste-text")?.value ?? "",
      pastePreviewText: document.querySelector("[data-local-package-paste-preview]")?.textContent?.trim() ?? "",
      pastePreviewDisplay: getComputedStyle(document.querySelector("[data-local-package-paste-preview]")).display,
    }));
    assert(pasteImportState.importAction?.imported === true, "pasted package should import");
    assert(pasteImportState.importAction?.file_name === "pasted-teacher-package.json", `paste import file name mismatch: ${pasteImportState.importAction?.file_name}`);
    assert(pasteImportState.importAction?.lesson_count === 1 && pasteImportState.importAction?.report_count === 4, `paste import counts mismatch: ${JSON.stringify(pasteImportState.importAction)}`);
    assert(
      Array.isArray(pasteImportState.importAction?.materials_summary)
        && pasteImportState.importAction.materials_summary.some((item) => String(item).includes("교사용 자료 4개") && String(item).includes("학생 명단 양식")),
      `paste import materials summary mismatch: ${JSON.stringify(pasteImportState.importAction?.materials_summary)}`,
    );
    assert(
      Array.isArray(pasteImportState.importAction?.student_materials_summary)
        && pasteImportState.importAction.student_materials_summary.some((item) => String(item).includes("교과 1개") && String(item).includes("속도 기록 그래프"))
        && pasteImportState.importAction.student_materials_summary.some((item) => String(item).includes("학생 자료 2개") && String(item).includes("학생 배포 안내문") && String(item).includes("학생 명단 양식")),
      `paste import student materials summary mismatch: ${JSON.stringify(pasteImportState.importAction?.student_materials_summary)}`,
    );
    assert(pasteImportState.importAction?.account_required === false && pasteImportState.importAction?.cloud_sync === false && pasteImportState.importAction?.public_registry === false, "paste import boundary mismatch");
    assert(pasteImportState.rail?.launch_kind === "local_package_import", `paste import launch kind mismatch: ${pasteImportState.rail?.launch_kind}`);
    assert(pasteImportState.rail?.onboarding_profile === "student", `paste import onboarding mismatch: ${pasteImportState.rail?.onboarding_profile}`);
    assert(pasteImportState.runText.trim() === String(downloadedPayload.lessons[0].source_text ?? "").trim(), "pasted package run source mismatch");
    assert(pasteImportState.deliveryStatusText.includes("받은 배포 파일 준비됨"), `paste import delivery status mismatch: ${pasteImportState.deliveryStatusText}`);
    assert(pasteImportState.deliveryInstructionsText.includes("받은 수업 안내"), `paste import delivery instructions mismatch: ${pasteImportState.deliveryInstructionsText}`);
    assert(pasteImportState.deliveryInstructionsText.includes("받은 자료:"), `paste import delivery instructions missing materials: ${pasteImportState.deliveryInstructionsText}`);
    assert(pasteImportState.deliveryInstructionsText.includes("교과 1개") && pasteImportState.deliveryInstructionsText.includes("학생 자료 2개") && pasteImportState.deliveryInstructionsText.includes("학생 명단 양식"), `paste import delivery instructions material summary mismatch: ${pasteImportState.deliveryInstructionsText}`);
    assert(!pasteImportState.deliveryInstructionsText.includes("교사용 자료"), `paste import student instructions should not show teacher material label: ${pasteImportState.deliveryInstructionsText}`);
    assert(pasteImportState.pasteTextValue === "", "pasted package text should be cleared after successful import");
    assert(pasteImportState.pastePreviewText === "", `pasted package preview should be cleared after successful import: ${pasteImportState.pastePreviewText}`);
    assert(pasteImportState.pastePreviewDisplay === "none", `pasted package preview should be hidden after successful import: ${pasteImportState.pastePreviewDisplay}`);

    await page.evaluate(() => {
      document.querySelector("#screen-run .main-shell-tab[data-main-tab-target='browse']")?.click();
    });
    await waitVisible(page, "#screen-browse");
    await page.setInputFiles("#input-local-package-file", downloadedPath);
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true);
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => document.activeElement?.id === "btn-run");
    const importState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      rail: window.__SEAMGRIM_RUN_PRESET_RAIL__ ?? null,
      activeElementId: document.activeElement?.id ?? "",
      runText: document.querySelector("#run-ddn-preview")?.value ?? "",
      sourceLabel: document.querySelector("[data-shell-source-label]")?.textContent?.trim() ?? "",
      browseImportStatus: document.querySelector("[data-local-package-import-status]")?.textContent?.trim() ?? "",
      browseImportStatusState: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
      browseImportStatusRole: document.querySelector("[data-local-package-import-status]")?.getAttribute("role") ?? "",
      browseImportStatusLive: document.querySelector("[data-local-package-import-status]")?.getAttribute("aria-live") ?? "",
      browseImportStatusAtomic: document.querySelector("[data-local-package-import-status]")?.getAttribute("aria-atomic") ?? "",
      launchKindDataset: document.querySelector("#screen-run")?.dataset?.launchKind ?? "",
      mainShellTabsDisplay: getComputedStyle(document.querySelector("#screen-run .main-shell-tabs")).display,
      classroomMode: document.querySelector("[data-classroom-mode-switch]")?.dataset?.mode ?? "",
      classroomSwitchDisplay: getComputedStyle(document.querySelector("[data-classroom-mode-switch]")).display,
      runButtonText: document.querySelector("#btn-run")?.textContent?.trim() ?? "",
      advancedRunDisplay: getComputedStyle(document.querySelector("#btn-advanced-run")).display,
      mirrorTabDisplay: getComputedStyle(document.querySelector("#run-tab-btn-mirror")).display,
      preRunMirrorHashText: document.querySelector("#run-mirror-hash")?.textContent?.trim() ?? "",
      preRunCopyHashDisabled: Boolean(document.querySelector("#btn-run-copy-hash")?.disabled),
      studentPressed: document.querySelector("[data-classroom-mode='student']")?.getAttribute("aria-pressed") ?? "",
      teacherPressed: document.querySelector("[data-classroom-mode='teacher']")?.getAttribute("aria-pressed") ?? "",
      lessonBriefText: document.querySelector("[data-run-lesson-brief]")?.textContent?.trim() ?? "",
      lessonSummaryText: document.querySelector("#run-lesson-summary")?.textContent?.trim() ?? "",
      deliveryStatusText: document.querySelector("[data-run-delivery-status]")?.textContent?.trim() ?? "",
      deliveryStatusState: document.querySelector("[data-run-delivery-status]")?.dataset?.state ?? "",
      deliveryStatusDisplay: getComputedStyle(document.querySelector("[data-run-delivery-status]")).display,
      deliveryStatusRole: document.querySelector("[data-run-delivery-status]")?.getAttribute("role") ?? "",
      deliveryStatusLive: document.querySelector("[data-run-delivery-status]")?.getAttribute("aria-live") ?? "",
      deliveryStatusAtomic: document.querySelector("[data-run-delivery-status]")?.getAttribute("aria-atomic") ?? "",
      deliveryInstructionsDisplay: getComputedStyle(document.querySelector("[data-run-delivery-instructions]")).display,
      deliveryInstructionsReady: document.querySelector("[data-run-delivery-instructions]")?.dataset?.ready ?? "",
      deliveryInstructionsText: document.querySelector("[data-run-delivery-instructions]")?.textContent?.trim() ?? "",
      deliveryInstructionsItems: Array.from(document.querySelectorAll("[data-run-delivery-instructions] li")).map((node) => node.textContent?.trim() ?? ""),
      deliveryResultDisplay: getComputedStyle(document.querySelector("[data-run-delivery-result]")).display,
      deliveryResultReady: document.querySelector("[data-run-delivery-result]")?.dataset?.ready ?? "",
      deliveryResultCopyDisabled: Boolean(document.querySelector("#btn-run-delivery-result-copy")?.disabled),
      deliveryResultDownloadDisabled: Boolean(document.querySelector("#btn-run-delivery-result-download")?.disabled),
      inputRegistry: JSON.parse(localStorage.getItem("seamgrim.input_registry.v0") || "{}"),
    }));
    assert(importState.importAction?.schema === "seamgrim.local_package_import_action.v1", "import action schema mismatch");
    assert(importState.importAction?.package_id === downloadedPayload.manifest.package_id, "import package id mismatch");
    assert(importState.importAction?.session_label === classroomSessionLabel, `import session label mismatch: ${importState.importAction?.session_label}`);
    assert(importState.importAction?.lesson_id === downloadedPayload.lessons[0].lesson_id, "import lesson id mismatch");
    assert(importState.importAction?.lesson_title === "속도 기록 그래프", `import lesson title mismatch: ${importState.importAction?.lesson_title}`);
    assert(importState.importAction?.lesson_count === 1 && importState.importAction?.report_count === 4, `import counts mismatch: ${JSON.stringify(importState.importAction)}`);
    assert(
      Array.isArray(importState.importAction?.materials_summary)
        && importState.importAction.materials_summary.some((item) => String(item).includes("교사용 자료 4개") && String(item).includes("학생 명단 양식")),
      `import materials summary mismatch: ${JSON.stringify(importState.importAction?.materials_summary)}`,
    );
    assert(
      Array.isArray(importState.importAction?.student_materials_summary)
        && importState.importAction.student_materials_summary.some((item) => String(item).includes("교과 1개") && String(item).includes("속도 기록 그래프"))
        && importState.importAction.student_materials_summary.some((item) => String(item).includes("학생 자료 2개") && String(item).includes("학생 배포 안내문") && String(item).includes("학생 명단 양식")),
      `import student materials summary mismatch: ${JSON.stringify(importState.importAction?.student_materials_summary)}`,
    );
    assert(importState.importAction?.account_required === false && importState.importAction?.cloud_sync === false && importState.importAction?.public_registry === false, "import boundary mismatch");
    assert(importState.rail?.launch_kind === "local_package_import", `import launch kind mismatch: ${importState.rail?.launch_kind}`);
    assert(importState.rail?.launch_label === "배포 열기", `import launch label mismatch: ${importState.rail?.launch_label}`);
    assert(importState.rail?.onboarding_profile === "student", `import onboarding mismatch: ${importState.rail?.onboarding_profile}`);
    assert(importState.activeElementId === "btn-run", `student import should focus run CTA: ${importState.activeElementId}`);
    assert(importState.launchKindDataset === "local_package_import", `import launch dataset mismatch: ${importState.launchKindDataset}`);
    assert(importState.mainShellTabsDisplay === "none", `student import should hide shell tabs: ${importState.mainShellTabsDisplay}`);
    assert(importState.classroomMode === "student" && importState.studentPressed === "true" && importState.teacherPressed === "false", "import should open in student mode");
    assert(importState.classroomSwitchDisplay === "none", `student import should hide classroom mode switch: ${importState.classroomSwitchDisplay}`);
    assert(importState.runButtonText.includes("받은 수업 실행"), `student import run CTA mismatch: ${importState.runButtonText}`);
    assert(importState.advancedRunDisplay === "none", `student import should hide advanced run button: ${importState.advancedRunDisplay}`);
    assert(importState.mirrorTabDisplay === "none", `student import should hide mirror tab: ${importState.mirrorTabDisplay}`);
    assert(importState.preRunMirrorHashText === "상태 기록: -", `student import should wait for explicit run: ${importState.preRunMirrorHashText}`);
    assert(importState.preRunCopyHashDisabled === true, "student import should not expose a run hash before explicit run");
    assert(importState.deliveryStatusDisplay !== "none", `student import should show package delivery status: ${importState.deliveryStatusDisplay}`);
    assert(importState.deliveryStatusState === "ready", `student import delivery status state mismatch: ${importState.deliveryStatusState}`);
    assert(
      importState.deliveryStatusText.includes("받은 배포 파일 준비됨")
        && importState.deliveryStatusText.includes(`차시 ${classroomSessionLabel}`)
        && importState.deliveryStatusText.includes("결과 확인: 그래프, 표")
        && importState.deliveryStatusText.includes(downloadedPayload.manifest.title),
      `student import delivery status mismatch: ${importState.deliveryStatusText}`,
    );
    assert(importState.deliveryStatusRole === "status", `delivery status role mismatch: ${importState.deliveryStatusRole}`);
    assert(importState.deliveryStatusLive === "polite", `delivery status aria-live mismatch: ${importState.deliveryStatusLive}`);
    assert(importState.deliveryStatusAtomic === "true", `delivery status aria-atomic mismatch: ${importState.deliveryStatusAtomic}`);
    assert(importState.deliveryInstructionsDisplay !== "none", `student import should show delivery instructions: ${importState.deliveryInstructionsDisplay}`);
    assert(importState.deliveryInstructionsReady === "1", `student import delivery instructions readiness mismatch: ${importState.deliveryInstructionsReady}`);
    assert(importState.deliveryInstructionsItems.length >= 6, `student import delivery instructions item count mismatch: ${importState.deliveryInstructionsItems.length}`);
    assert(importState.deliveryResultDisplay === "none", `student import should hide result copy before run: ${importState.deliveryResultDisplay}`);
    assert(importState.deliveryResultReady === "0", `student import result copy should start unready: ${importState.deliveryResultReady}`);
    assert(importState.deliveryResultCopyDisabled === true, "student import should disable result copy before run");
    assert(importState.deliveryResultDownloadDisabled === true, "student import should disable result download before run");
    assert(
      importState.deliveryInstructionsText.includes("받은 수업 안내")
        && importState.deliveryInstructionsText.includes("받은 자료:")
        && importState.deliveryInstructionsText.includes("교과 1개")
        && importState.deliveryInstructionsText.includes("학생 자료 2개")
        && importState.deliveryInstructionsText.includes("학생 명단 양식")
        && importState.deliveryInstructionsText.includes(`차시: ${classroomSessionLabel}`)
        && importState.deliveryInstructionsText.includes("수업: 속도 기록 그래프")
        && importState.deliveryInstructionsText.includes("목표:")
        && importState.deliveryInstructionsText.includes("오늘 활동:")
        && importState.deliveryInstructionsText.includes("배포 열기")
        && importState.deliveryInstructionsText.includes("붙여넣기 열기")
        && importState.deliveryInstructionsText.includes("받은 수업 실행")
        && importState.deliveryInstructionsText.includes("결과 확인: 그래프, 표"),
      `student import delivery instructions mismatch: ${importState.deliveryInstructionsText}`,
    );
    assert(!importState.deliveryInstructionsText.includes("교사용 자료"), `student import instructions should not show teacher material label: ${importState.deliveryInstructionsText}`);
    assert(
      importState.lessonBriefText.includes("교사가 보낸 배포 파일")
        && importState.lessonBriefText.includes(downloadedPayload.manifest.title)
        && importState.lessonBriefText.includes("속도 기록 그래프")
        && importState.lessonBriefText.includes("활동:")
        && importState.lessonBriefText.includes("기록 길이")
        && importState.lessonBriefText.includes("결과 확인:")
        && importState.lessonBriefText.includes("그래프")
        && importState.lessonBriefText.includes("표"),
      `imported lesson brief should preserve package course context: ${importState.lessonBriefText}`,
    );
    assert(
      importState.lessonSummaryText.includes("결과 확인:")
        && importState.lessonSummaryText.includes("그래프")
        && importState.lessonSummaryText.includes("표"),
      `imported lesson summary should preserve result check context: ${importState.lessonSummaryText}`,
    );
    assert(importState.runText.trim() === String(downloadedPayload.lessons[0].source_text ?? "").trim(), "imported run source mismatch");
    assert(importState.sourceLabel.includes(downloadedPayload.lessons[0].title), `imported source label mismatch: ${importState.sourceLabel}`);
    assert(importState.browseImportStatusState === "ok", `import status state mismatch: ${importState.browseImportStatusState}`);
    assert(importState.browseImportStatus.includes(downloadedPayload.manifest.title), `import status text mismatch: ${importState.browseImportStatus}`);
    assert(importState.browseImportStatus.includes("실행할 수업 속도 기록 그래프"), `import status missing lesson title: ${importState.browseImportStatus}`);
    assert(importState.browseImportStatus.includes("교과 1개"), `import status missing lesson count: ${importState.browseImportStatus}`);
    assert(importState.browseImportStatus.includes("학생 자료 2개"), `import status missing student material count: ${importState.browseImportStatus}`);
    assert(importState.browseImportStatus.includes("학생 명단 양식"), `import status missing material title: ${importState.browseImportStatus}`);
    assert(!importState.browseImportStatus.includes("교사용 자료"), `import status should not expose teacher material label: ${importState.browseImportStatus}`);
    assert(importState.browseImportStatusRole === "status", `import status role mismatch: ${importState.browseImportStatusRole}`);
    assert(importState.browseImportStatusLive === "polite", `import status aria-live mismatch: ${importState.browseImportStatusLive}`);
    assert(importState.browseImportStatusAtomic === "true", `import status aria-atomic mismatch: ${importState.browseImportStatusAtomic}`);

    await page.click("#btn-run");
    await page.waitForFunction(() => {
      const text = document.querySelector("#run-mirror-hash")?.textContent?.trim() ?? "";
      return text && text !== "상태 기록: -";
    });
    const importedRunState = await page.evaluate(() => ({
      mirrorHashText: document.querySelector("#run-mirror-hash")?.textContent?.trim() ?? "",
      mirrorHashTitle: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      copyHashDisabled: Boolean(document.querySelector("#btn-run-copy-hash")?.disabled),
      copyHashText: document.querySelector("#btn-run-copy-hash")?.textContent?.trim() ?? "",
      copyHashTitle: document.querySelector("#btn-run-copy-hash")?.getAttribute("title") ?? "",
      copyHashStudentDelivery: document.querySelector("#btn-run-copy-hash")?.dataset?.studentDelivery ?? "",
      deliveryResultDisplay: getComputedStyle(document.querySelector("[data-run-delivery-result]")).display,
      deliveryResultReady: document.querySelector("[data-run-delivery-result]")?.dataset?.ready ?? "",
      deliveryResultCopyText: document.querySelector("#btn-run-delivery-result-copy")?.textContent?.trim() ?? "",
      deliveryResultCopyTitle: document.querySelector("#btn-run-delivery-result-copy")?.getAttribute("title") ?? "",
      deliveryResultCopyDisabled: Boolean(document.querySelector("#btn-run-delivery-result-copy")?.disabled),
      deliveryResultDownloadText: document.querySelector("#btn-run-delivery-result-download")?.textContent?.trim() ?? "",
      deliveryResultDownloadTitle: document.querySelector("#btn-run-delivery-result-download")?.getAttribute("title") ?? "",
      deliveryResultDownloadDisabled: Boolean(document.querySelector("#btn-run-delivery-result-download")?.disabled),
      deliveryStudentNameDisplay: getComputedStyle(document.querySelector(".run-delivery-student-name-field")).display,
      deliveryStudentNamePlaceholder: document.querySelector("#run-delivery-student-name")?.getAttribute("placeholder") ?? "",
      runButtonText: document.querySelector("#btn-run")?.textContent?.trim() ?? "",
      lessonBriefText: document.querySelector("[data-run-lesson-brief]")?.textContent?.trim() ?? "",
      deliveryStatusText: document.querySelector("[data-run-delivery-status]")?.textContent?.trim() ?? "",
      deliveryStatusState: document.querySelector("[data-run-delivery-status]")?.dataset?.state ?? "",
      deliveryInstructionsText: document.querySelector("[data-run-delivery-instructions]")?.textContent?.trim() ?? "",
    }));
    assert(importedRunState.mirrorHashText.startsWith("상태 기록: "), `imported package run hash missing: ${importedRunState.mirrorHashText}`);
    assert(!importedRunState.mirrorHashText.endsWith("-"), `imported package run hash stayed empty: ${importedRunState.mirrorHashText}`);
    assert(importedRunState.mirrorHashTitle.includes("전체 상태 기록:"), `imported package run hash title mismatch: ${importedRunState.mirrorHashTitle}`);
    assert(importedRunState.copyHashDisabled === true, "imported package run should keep result copy disabled until student name is entered");
    assert(importedRunState.copyHashText === "결과 복사", `student result copy label mismatch: ${importedRunState.copyHashText}`);
    assert(importedRunState.copyHashTitle.includes("학생 이름"), `student result copy title should ask for student name: ${importedRunState.copyHashTitle}`);
    assert(importedRunState.copyHashStudentDelivery === "1", `student result copy mode mismatch: ${importedRunState.copyHashStudentDelivery}`);
    assert(importedRunState.deliveryResultDisplay !== "none", `student delivery result copy should be visible after run: ${importedRunState.deliveryResultDisplay}`);
    assert(importedRunState.deliveryResultReady === "1", `student delivery result copy readiness mismatch: ${importedRunState.deliveryResultReady}`);
    assert(importedRunState.deliveryResultCopyText === "결과 복사", `student delivery result copy label mismatch: ${importedRunState.deliveryResultCopyText}`);
    assert(importedRunState.deliveryResultCopyTitle.includes("학생 이름"), `student delivery result copy title should ask for student name: ${importedRunState.deliveryResultCopyTitle}`);
    assert(importedRunState.deliveryResultCopyDisabled === true, "student delivery result copy should stay disabled until student name is entered");
    assert(importedRunState.deliveryResultDownloadText === "결과 저장", `student delivery result download label mismatch: ${importedRunState.deliveryResultDownloadText}`);
    assert(importedRunState.deliveryResultDownloadTitle.includes("학생 이름"), `student delivery result download title should ask for student name: ${importedRunState.deliveryResultDownloadTitle}`);
    assert(importedRunState.deliveryResultDownloadDisabled === true, "student delivery result download should stay disabled until student name is entered");
    assert(importedRunState.deliveryStudentNameDisplay !== "none", `student name field should be visible after run: ${importedRunState.deliveryStudentNameDisplay}`);
    assert(importedRunState.deliveryStudentNamePlaceholder === "학생 이름", `student name placeholder mismatch: ${importedRunState.deliveryStudentNamePlaceholder}`);
    assert(importedRunState.runButtonText.includes("받은 수업 실행"), `imported package run CTA should stay student-facing: ${importedRunState.runButtonText}`);
    assert(importedRunState.deliveryStatusState === "done", `imported package delivery status should complete: ${importedRunState.deliveryStatusState}`);
    assert(
      importedRunState.deliveryStatusText.includes("받은 수업 실행 완료")
        && importedRunState.deliveryStatusText.includes(`차시 ${classroomSessionLabel}`)
        && importedRunState.deliveryStatusText.includes("결과 확인: 그래프, 표")
        && importedRunState.deliveryStatusText.includes(downloadedPayload.manifest.title),
      `imported package delivery complete text mismatch: ${importedRunState.deliveryStatusText}`,
    );
    assert(importedRunState.deliveryInstructionsText.includes("받은 수업 안내"), `imported package should keep delivery instructions after run: ${importedRunState.deliveryInstructionsText}`);
    assert(importedRunState.deliveryInstructionsText.includes(`차시: ${classroomSessionLabel}`), `imported package should keep session after run: ${importedRunState.deliveryInstructionsText}`);
    assert(importedRunState.deliveryInstructionsText.includes("오늘 활동:"), `imported package should keep activity after run: ${importedRunState.deliveryInstructionsText}`);
    assert(importedRunState.lessonBriefText.includes("교사가 보낸 배포 파일"), `imported package run should keep delivery context: ${importedRunState.lessonBriefText}`);
    assert(importedRunState.lessonBriefText.includes(downloadedPayload.manifest.title), `imported package run should keep package title: ${importedRunState.lessonBriefText}`);

    await page.fill("#run-delivery-student-name", "김가온");
    await page.waitForFunction(() => {
      return !document.querySelector("#btn-run-copy-hash")?.disabled
        && !document.querySelector("#btn-run-delivery-result-copy")?.disabled
        && !document.querySelector("#btn-run-delivery-result-download")?.disabled;
    });
    const namedStudentRunState = await page.evaluate(() => ({
      copyHashDisabled: Boolean(document.querySelector("#btn-run-copy-hash")?.disabled),
      copyHashTitle: document.querySelector("#btn-run-copy-hash")?.getAttribute("title") ?? "",
      deliveryResultCopyDisabled: Boolean(document.querySelector("#btn-run-delivery-result-copy")?.disabled),
      deliveryResultCopyTitle: document.querySelector("#btn-run-delivery-result-copy")?.getAttribute("title") ?? "",
      deliveryResultDownloadDisabled: Boolean(document.querySelector("#btn-run-delivery-result-download")?.disabled),
      deliveryResultDownloadTitle: document.querySelector("#btn-run-delivery-result-download")?.getAttribute("title") ?? "",
      deliveryStatusText: document.querySelector("[data-run-delivery-status]")?.textContent?.trim() ?? "",
    }));
    assert(namedStudentRunState.copyHashDisabled === false, "student result copy should enable after student name is entered");
    assert(namedStudentRunState.copyHashTitle.includes("교사에게 보낼 수업 결과"), `student result copy enabled title mismatch: ${namedStudentRunState.copyHashTitle}`);
    assert(namedStudentRunState.deliveryResultCopyDisabled === false, "student delivery result copy should enable after student name is entered");
    assert(namedStudentRunState.deliveryResultCopyTitle.includes("교사에게 보낼 수업 결과"), `student delivery result copy enabled title mismatch: ${namedStudentRunState.deliveryResultCopyTitle}`);
    assert(namedStudentRunState.deliveryResultDownloadDisabled === false, "student delivery result download should enable after student name is entered");
    assert(namedStudentRunState.deliveryResultDownloadTitle.includes("교사에게 보낼 수업 결과"), `student delivery result download enabled title mismatch: ${namedStudentRunState.deliveryResultDownloadTitle}`);
    assert(!namedStudentRunState.deliveryStatusText.includes("학생 이름 입력 필요"), `student delivery status should clear name prompt: ${namedStudentRunState.deliveryStatusText}`);
    await page.click("#btn-run-delivery-result-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_STATE_HASH_COPY_ACTION__?.copied === true);
    const studentResultCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_RUN_STATE_HASH_COPY_ACTION__ ?? null,
    }));
    assert(studentResultCopyState.action?.schema === "seamgrim.run_state_hash_copy_action.v1", "student result copy action schema mismatch");
    assert(studentResultCopyState.action?.student_delivery === true, "student result copy should be marked as delivery result");
    assert(studentResultCopyState.action?.submission_method === "clipboard", `student result copy submission method mismatch: ${studentResultCopyState.action?.submission_method}`);
    assert(studentResultCopyState.action?.launch_kind === "local_package_import", `student result copy launch kind mismatch: ${studentResultCopyState.action?.launch_kind}`);
    assert(studentResultCopyState.action?.student_name === "김가온", `student result copy name mismatch: ${studentResultCopyState.action?.student_name}`);
    assert(studentResultCopyState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `student result copy lesson id mismatch: ${studentResultCopyState.action?.lesson_id}`);
    assert(studentResultCopyState.action?.package_id === downloadedPayload.manifest.package_id, `student result copy package id mismatch: ${studentResultCopyState.action?.package_id}`);
    assert(studentResultCopyState.action?.session_label === classroomSessionLabel, `student result copy session mismatch: ${studentResultCopyState.action?.session_label}`);
    assert(studentResultCopyState.copiedText.includes("학생: 김가온"), `student result copy missing student name: ${studentResultCopyState.copiedText}`);
    assert(studentResultCopyState.copiedText.includes(`차시: ${classroomSessionLabel}`), `student result copy missing session: ${studentResultCopyState.copiedText}`);
    assert(studentResultCopyState.copiedText.includes("수업: 속도 기록 그래프"), `student result copy missing lesson title: ${studentResultCopyState.copiedText}`);
    assert(studentResultCopyState.copiedText.includes(`수업 코드: ${downloadedPayload.lessons[0].lesson_id}`), `student result copy missing lesson id: ${studentResultCopyState.copiedText}`);
    assert(studentResultCopyState.copiedText.includes(downloadedPayload.manifest.title), `student result copy missing package title: ${studentResultCopyState.copiedText}`);
    assert(studentResultCopyState.copiedText.includes(`배포 코드: ${downloadedPayload.manifest.package_id}`), `student result copy missing package id: ${studentResultCopyState.copiedText}`);
    assert(studentResultCopyState.copiedText.includes("결과 확인: 그래프, 표"), `student result copy missing result instruction: ${studentResultCopyState.copiedText}`);
    assert(studentResultCopyState.copiedText.includes("상태 기록:"), `student result copy missing state hash: ${studentResultCopyState.copiedText}`);
    assert(studentResultCopyState.action?.text === studentResultCopyState.copiedText, "student result copy action text mismatch");
    const [studentResultDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-delivery-result-download"),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_STATE_HASH_DOWNLOAD_ACTION__?.downloaded === true);
    const studentResultDownloadPath = await studentResultDownload.path();
    const studentResultDownloadedText = studentResultDownloadPath ? await fs.readFile(studentResultDownloadPath, "utf-8") : "";
    const studentResultDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_RUN_STATE_HASH_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(studentResultDownloadState.action?.schema === "seamgrim.run_state_hash_download_action.v1", "student result download action schema mismatch");
    assert(studentResultDownloadState.action?.student_delivery === true, "student result download should be marked as delivery result");
    assert(studentResultDownloadState.action?.submission_method === "file", `student result download submission method mismatch: ${studentResultDownloadState.action?.submission_method}`);
    assert(studentResultDownloadState.action?.launch_kind === "local_package_import", `student result download launch kind mismatch: ${studentResultDownloadState.action?.launch_kind}`);
    assert(studentResultDownloadState.action?.student_name === "김가온", `student result download name mismatch: ${studentResultDownloadState.action?.student_name}`);
    assert(studentResultDownloadState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `student result download lesson id mismatch: ${studentResultDownloadState.action?.lesson_id}`);
    assert(studentResultDownloadState.action?.package_id === downloadedPayload.manifest.package_id, `student result download package id mismatch: ${studentResultDownloadState.action?.package_id}`);
    assert(studentResultDownloadState.action?.session_label === classroomSessionLabel, `student result download session mismatch: ${studentResultDownloadState.action?.session_label}`);
    assert(String(studentResultDownloadState.action?.file_name ?? "").endsWith(".txt"), `student result download filename mismatch: ${studentResultDownloadState.action?.file_name}`);
    assert(String(studentResultDownloadState.action?.file_name ?? "").includes(downloadedPayload.manifest.package_id), `student result download filename should include package id: ${studentResultDownloadState.action?.file_name}`);
    assert(String(studentResultDownloadState.action?.file_name ?? "").includes("_4_"), `student result download filename should include session marker: ${studentResultDownloadState.action?.file_name}`);
    assert(String(studentResultDownloadState.action?.file_name ?? "").includes("student_result"), `student result download filename should keep result marker: ${studentResultDownloadState.action?.file_name}`);
    assert(studentResultDownload.suggestedFilename() === studentResultDownloadState.action.file_name, "student result suggested filename mismatch");
    assert(studentResultDownloadState.action?.text === studentResultCopyState.copiedText, "student result download action text mismatch");
    assert(studentResultDownloadedText.trim() === studentResultCopyState.copiedText.trim(), "student result downloaded text mismatch");

    await page.evaluate(() => {
      document.querySelector("#screen-run .main-shell-tab[data-main-tab-target='browse']")?.click();
    });
    await waitVisible(page, "#screen-browse");
    await page.click(".lesson-card[data-lesson-id='rep_physics_velocity_history_v1']");
    await page.click("#btn-open-in-studio-teacher");
    await waitVisible(page, "#screen-run");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_PRESET_RAIL__?.onboarding_profile === "teacher");
    const teacherReviewPreState = await page.evaluate(() => ({
      reviewDisplay: getComputedStyle(document.querySelector("[data-run-student-result-review]")).display,
      rosterDisplay: getComputedStyle(document.querySelector("#run-student-roster-input")).display,
      rosterPlaceholder: document.querySelector("#run-student-roster-input")?.getAttribute("placeholder") ?? "",
      rosterFileAccept: document.querySelector("#input-run-student-roster-file")?.getAttribute("accept") ?? "",
      rosterFileMultiple: Boolean(document.querySelector("#input-run-student-roster-file")?.multiple),
      rosterFileOpenText: document.querySelector("#btn-run-student-roster-file-open")?.textContent?.trim() ?? "",
      rosterTemplateCopyText: document.querySelector("#btn-run-student-roster-template-copy")?.textContent?.trim() ?? "",
      rosterTemplateDownloadText: document.querySelector("#btn-run-student-roster-template-download")?.textContent?.trim() ?? "",
      resultFileAccept: document.querySelector("#input-run-student-result-file")?.getAttribute("accept") ?? "",
      resultFileMultiple: Boolean(document.querySelector("#input-run-student-result-file")?.multiple),
      resultFileOpenText: document.querySelector("#btn-run-student-result-file-open")?.textContent?.trim() ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      rosterOnlyClearText: document.querySelector("#btn-run-student-roster-only-clear")?.textContent?.trim() ?? "",
      resultOnlyClearText: document.querySelector("#btn-run-student-result-only-clear")?.textContent?.trim() ?? "",
      resultClearText: document.querySelector("#btn-run-student-result-clear")?.textContent?.trim() ?? "",
      missingReminderDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-copy")?.disabled),
      missingReminderText: document.querySelector("#btn-run-student-missing-reminder-copy")?.textContent?.trim() ?? "",
      missingReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-download")?.disabled),
      missingReminderDownloadText: document.querySelector("#btn-run-student-missing-reminder-download")?.textContent?.trim() ?? "",
      reviewReminderDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-copy")?.disabled),
      reviewReminderText: document.querySelector("#btn-run-student-review-reminder-copy")?.textContent?.trim() ?? "",
      reviewReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-download")?.disabled),
      reviewReminderDownloadText: document.querySelector("#btn-run-student-review-reminder-download")?.textContent?.trim() ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      statusRole: document.querySelector("[data-run-student-result-status]")?.getAttribute("role") ?? "",
      statusAriaLive: document.querySelector("[data-run-student-result-status]")?.getAttribute("aria-live") ?? "",
    }));
    assert(teacherReviewPreState.reviewDisplay !== "none", `teacher student result review should be visible: ${teacherReviewPreState.reviewDisplay}`);
    assert(teacherReviewPreState.rosterDisplay !== "none", `teacher roster input should be visible: ${teacherReviewPreState.rosterDisplay}`);
    assert(teacherReviewPreState.rosterPlaceholder.includes("번호+이름") && teacherReviewPreState.rosterPlaceholder.includes("미제출"), `teacher roster placeholder mismatch: ${teacherReviewPreState.rosterPlaceholder}`);
    assert(teacherReviewPreState.rosterFileAccept.includes(".tsv") && teacherReviewPreState.rosterFileAccept.includes(".csv"), `teacher roster file accept mismatch: ${teacherReviewPreState.rosterFileAccept}`);
    assert(teacherReviewPreState.rosterFileMultiple === true, "teacher roster file input should accept multiple files");
    assert(teacherReviewPreState.rosterFileOpenText === "명단 파일 열기", `teacher roster file open label mismatch: ${teacherReviewPreState.rosterFileOpenText}`);
    assert(teacherReviewPreState.rosterTemplateCopyText === "명단 양식 복사", `teacher roster template copy label mismatch: ${teacherReviewPreState.rosterTemplateCopyText}`);
    assert(teacherReviewPreState.rosterTemplateDownloadText === "명단 양식 저장", `teacher roster template download label mismatch: ${teacherReviewPreState.rosterTemplateDownloadText}`);
    assert(
      teacherReviewPreState.resultFileAccept.includes(".txt")
        && teacherReviewPreState.resultFileAccept.includes(".tsv")
        && teacherReviewPreState.resultFileAccept.includes("text/plain")
        && teacherReviewPreState.resultFileAccept.includes("text/tab-separated-values"),
      `teacher result file accept mismatch: ${teacherReviewPreState.resultFileAccept}`,
    );
    assert(teacherReviewPreState.resultFileMultiple === true, "teacher result file input should accept multiple files");
    assert(teacherReviewPreState.resultFileOpenText === "결과 파일 열기", `teacher result file open label mismatch: ${teacherReviewPreState.resultFileOpenText}`);
    assert(teacherReviewPreState.resultReviewDisabled === true, "teacher result review should start disabled until input exists");
    assert(teacherReviewPreState.rosterOnlyClearText === "명단만 비우기", `teacher roster-only clear label mismatch: ${teacherReviewPreState.rosterOnlyClearText}`);
    assert(teacherReviewPreState.resultOnlyClearText === "결과만 비우기", `teacher result-only clear label mismatch: ${teacherReviewPreState.resultOnlyClearText}`);
    assert(teacherReviewPreState.resultClearText === "입력 비우기", `teacher result clear label mismatch: ${teacherReviewPreState.resultClearText}`);
    assert(teacherReviewPreState.missingReminderDisabled === true, "missing reminder should start disabled");
    assert(teacherReviewPreState.missingReminderText === "미제출 안내 복사", `missing reminder label mismatch: ${teacherReviewPreState.missingReminderText}`);
    assert(teacherReviewPreState.missingReminderDownloadDisabled === true, "missing reminder download should start disabled");
    assert(teacherReviewPreState.missingReminderDownloadText === "미제출 안내 저장", `missing reminder download label mismatch: ${teacherReviewPreState.missingReminderDownloadText}`);
    assert(teacherReviewPreState.reviewReminderDisabled === true, "review reminder should start disabled");
    assert(teacherReviewPreState.reviewReminderText === "확인 필요 안내 복사", `review reminder label mismatch: ${teacherReviewPreState.reviewReminderText}`);
    assert(teacherReviewPreState.reviewReminderDownloadDisabled === true, "review reminder download should start disabled");
    assert(teacherReviewPreState.reviewReminderDownloadText === "확인 필요 안내 저장", `review reminder download label mismatch: ${teacherReviewPreState.reviewReminderDownloadText}`);
    assert(teacherReviewPreState.statusText === "결과 대기", `teacher review initial status mismatch: ${teacherReviewPreState.statusText}`);
    assert(teacherReviewPreState.statusRole === "status", `teacher review status role mismatch: ${teacherReviewPreState.statusRole}`);
    assert(teacherReviewPreState.statusAriaLive === "polite", `teacher review status aria-live mismatch: ${teacherReviewPreState.statusAriaLive}`);
    await page.click("#btn-run-student-roster-template-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_COPY_ACTION__?.copied === true);
    const rosterTemplateCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_COPY_ACTION__ ?? null,
    }));
    assert(rosterTemplateCopyState.action?.schema === "seamgrim.student_roster_template_copy_action.v1", "student roster template copy schema mismatch");
    assert(rosterTemplateCopyState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `student roster template copy lesson id mismatch: ${rosterTemplateCopyState.action?.lesson_id}`);
    assert(rosterTemplateCopyState.action?.package_id === downloadedPayload.manifest.package_id, `student roster template copy package id mismatch: ${rosterTemplateCopyState.action?.package_id}`);
    assert(rosterTemplateCopyState.action?.session_label === classroomSessionLabel, `student roster template copy session mismatch: ${rosterTemplateCopyState.action?.session_label}`);
    assert(String(rosterTemplateCopyState.action?.file_name ?? "").endsWith(".tsv"), `student roster template copy filename mismatch: ${rosterTemplateCopyState.action?.file_name}`);
    assert(
      String(rosterTemplateCopyState.action?.text ?? "").includes("# 셈그림 학생 명단 양식")
        && String(rosterTemplateCopyState.action?.text ?? "").includes(`수업 코드\t${downloadedPayload.lessons[0].lesson_id}`)
        && String(rosterTemplateCopyState.action?.text ?? "").includes(`차시\t${classroomSessionLabel}`)
        && String(rosterTemplateCopyState.action?.text ?? "").includes(`배포 코드\t${downloadedPayload.manifest.package_id}`)
        && String(rosterTemplateCopyState.action?.text ?? "").includes("번호\t이름"),
      `student roster template copy action text mismatch: ${rosterTemplateCopyState.action?.text}`,
    );
    assert(rosterTemplateCopyState.copiedText.trim() === String(rosterTemplateCopyState.action?.text ?? "").trim(), "student roster template copied text mismatch");
    const [rosterTemplateDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-student-roster-template-download"),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_DOWNLOAD_ACTION__?.downloaded === true);
    const rosterTemplateDownloadPath = await rosterTemplateDownload.path();
    const rosterTemplateDownloadedText = rosterTemplateDownloadPath ? await fs.readFile(rosterTemplateDownloadPath, "utf-8") : "";
    const rosterTemplateDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_ROSTER_TEMPLATE_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(rosterTemplateDownloadState.action?.schema === "seamgrim.student_roster_template_download_action.v1", "student roster template download schema mismatch");
    assert(rosterTemplateDownloadState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `student roster template lesson id mismatch: ${rosterTemplateDownloadState.action?.lesson_id}`);
    assert(rosterTemplateDownloadState.action?.package_id === downloadedPayload.manifest.package_id, `student roster template package id mismatch: ${rosterTemplateDownloadState.action?.package_id}`);
    assert(rosterTemplateDownloadState.action?.session_label === classroomSessionLabel, `student roster template session mismatch: ${rosterTemplateDownloadState.action?.session_label}`);
    assert(String(rosterTemplateDownloadState.action?.file_name ?? "").endsWith(".tsv"), `student roster template filename mismatch: ${rosterTemplateDownloadState.action?.file_name}`);
    assert(String(rosterTemplateDownloadState.action?.file_name ?? "").includes(downloadedPayload.lessons[0].lesson_id), `student roster template filename should include lesson id: ${rosterTemplateDownloadState.action?.file_name}`);
    assert(String(rosterTemplateDownloadState.action?.file_name ?? "").includes(downloadedPayload.manifest.package_id), `student roster template filename should include package id: ${rosterTemplateDownloadState.action?.file_name}`);
    assert(String(rosterTemplateDownloadState.action?.file_name ?? "").includes("_4_"), `student roster template filename should include session marker: ${rosterTemplateDownloadState.action?.file_name}`);
    assert(rosterTemplateDownload.suggestedFilename() === rosterTemplateDownloadState.action.file_name, "student roster template suggested filename mismatch");
    assert(
      String(rosterTemplateDownloadState.action?.text ?? "").includes("# 셈그림 학생 명단 양식")
        && String(rosterTemplateDownloadState.action?.text ?? "").includes(`수업 코드\t${downloadedPayload.lessons[0].lesson_id}`)
        && String(rosterTemplateDownloadState.action?.text ?? "").includes(`차시\t${classroomSessionLabel}`)
        && String(rosterTemplateDownloadState.action?.text ?? "").includes(`배포 코드\t${downloadedPayload.manifest.package_id}`)
        && String(rosterTemplateDownloadState.action?.text ?? "").includes("번호\t이름"),
      `student roster template action text mismatch: ${rosterTemplateDownloadState.action?.text}`,
    );
    assert(rosterTemplateDownloadedText.trim() === String(rosterTemplateDownloadState.action?.text ?? "").trim(), `student roster template downloaded text mismatch: ${rosterTemplateDownloadedText}`);
    assert(rosterTemplateDownloadedText.trim() === rosterTemplateCopyState.copiedText.trim(), "student roster template copy/download mismatch");
    await page.setInputFiles("#input-run-student-result-file", {
      name: studentResultDownload.suggestedFilename(),
      mimeType: "text/plain",
      buffer: Buffer.from(studentResultDownloadedText, "utf-8"),
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__?.loaded === true);
    const resultFileLoadState = await page.evaluate(() => ({
      inputText: document.querySelector("#run-student-result-input")?.value ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      activeId: document.activeElement?.id ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      action: window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__ ?? null,
    }));
    assert(resultFileLoadState.action?.schema === "seamgrim.student_result_file_load_action.v1", "student result file load schema mismatch");
    assert(resultFileLoadState.action?.file_count === 1, `student result file load count mismatch: ${resultFileLoadState.action?.file_count}`);
    assert(Array.isArray(resultFileLoadState.action?.file_names) && resultFileLoadState.action.file_names[0] === studentResultDownload.suggestedFilename(), `student result file load names mismatch: ${resultFileLoadState.action?.file_names}`);
    assert(resultFileLoadState.action?.result_count === 1 && resultFileLoadState.action?.accepted_count === 1 && resultFileLoadState.action?.rejected_count === 0, `student result file load preview counts mismatch: ${JSON.stringify(resultFileLoadState.action)}`);
    assert(String(resultFileLoadState.action?.next_action ?? "").includes("결과표 저장"), `student result file load next action metadata mismatch: ${JSON.stringify(resultFileLoadState.action)}`);
    assert(resultFileLoadState.statusText.includes("결과 파일 1개 불러옴"), `student result file load status mismatch: ${resultFileLoadState.statusText}`);
    assert(resultFileLoadState.statusText.includes("학생 결과 1건"), `student result file load count preview mismatch: ${resultFileLoadState.statusText}`);
    assert(resultFileLoadState.statusText.includes("다음 행동: 결과 확인"), `student result file load next action mismatch: ${resultFileLoadState.statusText}`);
    assert(resultFileLoadState.resultReviewDisabled === false, "student result file load should enable review button");
    assert(resultFileLoadState.activeId === "btn-run-student-result-review", `student result file load should focus review button: ${resultFileLoadState.activeId}`);
    assert(resultFileLoadState.inputText.trim() === studentResultCopyState.copiedText.trim(), "student result file should populate review textarea");
    await page.setInputFiles("#input-run-student-result-file", {
      name: studentResultDownload.suggestedFilename(),
      mimeType: "text/plain",
      buffer: Buffer.from(studentResultDownloadedText, "utf-8"),
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__?.skipped_duplicate_count === 1);
    const duplicateResultFileLoadState = await page.evaluate(() => ({
      inputText: document.querySelector("#run-student-result-input")?.value ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      action: window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__ ?? null,
    }));
    assert(duplicateResultFileLoadState.action?.loaded === true, "duplicate student result load should keep current input usable");
    assert(duplicateResultFileLoadState.action?.loaded_file_count === 0, `duplicate student result load should not append files: ${JSON.stringify(duplicateResultFileLoadState.action)}`);
    assert(duplicateResultFileLoadState.action?.skipped_duplicate_count === 1, `duplicate student result skip count mismatch: ${JSON.stringify(duplicateResultFileLoadState.action)}`);
    assert(Array.isArray(duplicateResultFileLoadState.action?.duplicate_file_names) && duplicateResultFileLoadState.action.duplicate_file_names[0] === studentResultDownload.suggestedFilename(), `duplicate student result file names mismatch: ${duplicateResultFileLoadState.action?.duplicate_file_names}`);
    assert(duplicateResultFileLoadState.action?.result_count === 1 && duplicateResultFileLoadState.action?.accepted_count === 1 && duplicateResultFileLoadState.action?.rejected_count === 0, `duplicate student result preview counts mismatch: ${JSON.stringify(duplicateResultFileLoadState.action)}`);
    assert(duplicateResultFileLoadState.statusText.includes("결과 파일 0개 불러옴"), `duplicate student result load status mismatch: ${duplicateResultFileLoadState.statusText}`);
    assert(duplicateResultFileLoadState.statusText.includes("중복 1건 건너뜀"), `duplicate student result load should report skipped duplicate: ${duplicateResultFileLoadState.statusText}`);
    assert(duplicateResultFileLoadState.inputText.trim() === studentResultCopyState.copiedText.trim(), "duplicate student result load should not duplicate input text");
    assert(duplicateResultFileLoadState.resultReviewDisabled === false, "duplicate student result load should keep review enabled");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__?.accepted === true);
    const teacherReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      review: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__ ?? null,
    }));
    assert(teacherReviewState.statusState === "ok", `teacher review status state mismatch: ${teacherReviewState.statusState}`);
    assert(teacherReviewState.statusText.includes("확인됨"), `teacher review status text mismatch: ${teacherReviewState.statusText}`);
    assert(teacherReviewState.review?.schema === "seamgrim.student_result_return_review.v1", "teacher review schema mismatch");
    assert(teacherReviewState.review?.student_name === "김가온", `teacher review student name mismatch: ${teacherReviewState.review?.student_name}`);
    assert(teacherReviewState.review?.session_label === classroomSessionLabel, `teacher review session mismatch: ${teacherReviewState.review?.session_label}`);
    assert(teacherReviewState.review?.lesson_title === "속도 기록 그래프", `teacher review lesson title mismatch: ${teacherReviewState.review?.lesson_title}`);
    assert(teacherReviewState.review?.lesson_id === downloadedPayload.lessons[0].lesson_id, `teacher review lesson id mismatch: ${teacherReviewState.review?.lesson_id}`);
    assert(teacherReviewState.review?.package_id === downloadedPayload.manifest.package_id, `teacher review package id mismatch: ${teacherReviewState.review?.package_id}`);
    assert(teacherReviewState.review?.session_match === true, "teacher review session match mismatch");
    assert(teacherReviewState.review?.lesson_match === true, "teacher review lesson match mismatch");
    assert(teacherReviewState.review?.lesson_id_match === true, "teacher review lesson id match mismatch");
    assert(teacherReviewState.review?.package_id_match === true, "teacher review package id match mismatch");
    assert(teacherReviewState.review?.state_hash && teacherReviewState.review.state_hash !== "-", `teacher review state hash missing: ${teacherReviewState.review?.state_hash}`);
    assert(teacherReviewState.review?.account_required === false && teacherReviewState.review?.cloud_sync === false && teacherReviewState.review?.permission_system === false, "teacher review boundary mismatch");

    const missingNameStudentText = studentResultCopyState.copiedText
      .split(/\r?\n/u)
      .filter((line) => !line.startsWith("학생:"))
      .join("\n");
    await page.fill("#run-student-result-input", missingNameStudentText);
    const missingNamePreviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
    }));
    assert(missingNamePreviewState.statusState === "idle", `missing name preview state mismatch: ${missingNamePreviewState.statusState}`);
    assert(missingNamePreviewState.statusText.includes("사전 확인 0/1"), `missing name preview should preflight mismatch: ${missingNamePreviewState.statusText}`);
    assert(missingNamePreviewState.statusText.includes("학생 이름 확인"), `missing name preview should show name reason: ${missingNamePreviewState.statusText}`);
    assert(missingNamePreviewState.resultReviewDisabled === false, "missing name preview should keep review action enabled");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.rejected_count === 1);
    const missingNameReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(
      missingNameReviewState.statusText.includes("확인 필요")
        && missingNameReviewState.statusText.includes("학생 이름 확인"),
      `missing name review status mismatch: ${missingNameReviewState.statusText}`,
    );
    assert(missingNameReviewState.tableText.includes("학생 이름 확인"), `missing name review table missing note: ${missingNameReviewState.tableText}`);
    assert(missingNameReviewState.batch?.rows?.[0]?.student_name_missing === true, "missing name review flag mismatch");

    const wrongSessionStudentText = studentResultCopyState.copiedText.replace(
      `차시: ${classroomSessionLabel}`,
      "차시: 2학년 1반 1교시",
    );
    await page.fill("#run-student-result-input", wrongSessionStudentText);
    const wrongSessionPreviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
    }));
    assert(wrongSessionPreviewState.statusState === "idle", `wrong session preview state mismatch: ${wrongSessionPreviewState.statusState}`);
    assert(wrongSessionPreviewState.statusText.includes("사전 확인 0/1"), `wrong session preview should preflight mismatch: ${wrongSessionPreviewState.statusText}`);
    assert(wrongSessionPreviewState.statusText.includes("확인 필요 1건"), `wrong session preview should show review count: ${wrongSessionPreviewState.statusText}`);
    assert(wrongSessionPreviewState.statusText.includes("차시 확인"), `wrong session preview should show session reason: ${wrongSessionPreviewState.statusText}`);
    assert(wrongSessionPreviewState.resultReviewDisabled === false, "wrong session preview should keep review action enabled");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.rejected_count === 1);
    const wrongSessionReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(
      wrongSessionReviewState.statusText.includes("확인 필요")
        && wrongSessionReviewState.statusText.includes("차시 확인"),
      `wrong session review status mismatch: ${wrongSessionReviewState.statusText}`,
    );
    assert(wrongSessionReviewState.tableText.includes("차시 확인"), `wrong session review table missing note: ${wrongSessionReviewState.tableText}`);
    assert(wrongSessionReviewState.batch?.rows?.[0]?.session_match === false, "wrong session review flag mismatch");

    const wrongPackageStudentText = studentResultCopyState.copiedText.replace(
      `배포 코드: ${downloadedPayload.manifest.package_id}`,
      "배포 코드: wrong.classroom.package",
    );
    await page.fill("#run-student-result-input", wrongPackageStudentText);
    const wrongPackagePreviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
    }));
    assert(wrongPackagePreviewState.statusState === "idle", `wrong package preview state mismatch: ${wrongPackagePreviewState.statusState}`);
    assert(wrongPackagePreviewState.statusText.includes("사전 확인 0/1"), `wrong package preview should preflight mismatch: ${wrongPackagePreviewState.statusText}`);
    assert(wrongPackagePreviewState.statusText.includes("확인 필요 1건"), `wrong package preview should show review count: ${wrongPackagePreviewState.statusText}`);
    assert(wrongPackagePreviewState.statusText.includes("배포 코드 확인"), `wrong package preview should show package-code reason: ${wrongPackagePreviewState.statusText}`);
    assert(wrongPackagePreviewState.statusText.includes("다음 행동: 결과 확인"), `wrong package preview next action mismatch: ${wrongPackagePreviewState.statusText}`);
    assert(wrongPackagePreviewState.resultReviewDisabled === false, "wrong package preview should keep review action enabled");

    const multiStudentText = [
      studentResultCopyState.copiedText,
      studentResultCopyState.copiedText
        .replace("학생: 김가온", "학생: 이하늘")
        .replace(/상태 기록:.*/u, "상태 기록: blake3:student-two-hash"),
    ].join("\n\n");
    await page.fill("#run-student-result-input", multiStudentText);
    const multiStudentPreviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
    }));
    assert(multiStudentPreviewState.statusState === "idle", `multi student preview state mismatch: ${multiStudentPreviewState.statusState}`);
    assert(multiStudentPreviewState.statusText.includes("입력 수정됨"), `multi student preview should mark edited input: ${multiStudentPreviewState.statusText}`);
    assert(multiStudentPreviewState.statusText.includes("학생 결과 2건"), `multi student preview count mismatch: ${multiStudentPreviewState.statusText}`);
    assert(multiStudentPreviewState.statusText.includes("사전 확인 2/2"), `multi student preview should preflight accepted count: ${multiStudentPreviewState.statusText}`);
    assert(multiStudentPreviewState.statusText.includes("다음 행동: 결과 확인"), `multi student preview next action mismatch: ${multiStudentPreviewState.statusText}`);
    assert(multiStudentPreviewState.resultReviewDisabled === false, "multi student preview should keep review action enabled");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.row_count === 2);
    const teacherBatchReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      summaryDisplay: getComputedStyle(document.querySelector("[data-run-student-result-summary]")).display,
      tableDisplay: getComputedStyle(document.querySelector("[data-run-student-result-table]")).display,
      tableHeaderText: document.querySelector("[data-run-student-result-table] thead")?.textContent?.trim() ?? "",
      tableRows: Array.from(document.querySelectorAll("[data-run-student-result-table] tbody tr")).map((row) => ({
        state: row.getAttribute("data-student-result-row") || "",
        text: row.textContent?.trim() || "",
      })),
      copyDisabled: Boolean(document.querySelector("#btn-run-student-result-report-copy")?.disabled),
      downloadDisabled: Boolean(document.querySelector("#btn-run-student-result-report-download")?.disabled),
      downloadText: document.querySelector("#btn-run-student-result-report-download")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(teacherBatchReviewState.statusState === "ok", `teacher batch review status mismatch: ${teacherBatchReviewState.statusState}`);
    assert(teacherBatchReviewState.statusText.includes("확인됨 2/2"), `teacher batch review status text mismatch: ${teacherBatchReviewState.statusText}`);
    assert(teacherBatchReviewState.summaryDisplay !== "none", `teacher batch summary should be visible: ${teacherBatchReviewState.summaryDisplay}`);
    assert(teacherBatchReviewState.tableDisplay !== "none", `teacher batch table should be visible: ${teacherBatchReviewState.tableDisplay}`);
    assert(teacherBatchReviewState.summaryText.includes("학생 결과 2건"), `teacher batch summary mismatch: ${teacherBatchReviewState.summaryText}`);
    assert(teacherBatchReviewState.summaryText.includes("다음 행동: 결과표 저장 후 입력 비우기"), `teacher batch summary missing next action: ${teacherBatchReviewState.summaryText}`);
    assert(
      teacherBatchReviewState.tableHeaderText.includes("수업 코드")
        && teacherBatchReviewState.tableHeaderText.includes("배포 코드"),
      `teacher batch table header missing code columns: ${teacherBatchReviewState.tableHeaderText}`,
    );
    assert(teacherBatchReviewState.tableRows.length === 2, `teacher batch table row count mismatch: ${teacherBatchReviewState.tableRows.length}`);
    assert(teacherBatchReviewState.tableRows.every((row) => row.state === "ok"), `teacher batch rows should be accepted: ${JSON.stringify(teacherBatchReviewState.tableRows)}`);
    assert(teacherBatchReviewState.tableRows[0].text.includes("김가온") && teacherBatchReviewState.tableRows[1].text.includes("하늘"), `teacher batch table names mismatch: ${JSON.stringify(teacherBatchReviewState.tableRows)}`);
    assert(teacherBatchReviewState.tableRows.every((row) => row.text.includes(classroomSessionLabel)), `teacher batch table session mismatch: ${JSON.stringify(teacherBatchReviewState.tableRows)}`);
    assert(teacherBatchReviewState.tableRows.every((row) => row.text.includes(downloadedPayload.lessons[0].lesson_id)), `teacher batch table lesson id mismatch: ${JSON.stringify(teacherBatchReviewState.tableRows)}`);
    assert(teacherBatchReviewState.tableRows.every((row) => row.text.includes(downloadedPayload.manifest.package_id)), `teacher batch table package id mismatch: ${JSON.stringify(teacherBatchReviewState.tableRows)}`);
    assert(teacherBatchReviewState.copyDisabled === false, "teacher batch report copy should be enabled");
    assert(teacherBatchReviewState.downloadDisabled === false, "teacher batch report download should be enabled");
    assert(teacherBatchReviewState.downloadText === "결과표 저장", `teacher batch report download label mismatch: ${teacherBatchReviewState.downloadText}`);
    assert(teacherBatchReviewState.batch?.schema === "seamgrim.student_result_return_batch_review.v1", "teacher batch schema mismatch");
    assert(teacherBatchReviewState.batch?.accepted_count === 2 && teacherBatchReviewState.batch?.rejected_count === 0, "teacher batch count mismatch");
    assert(String(teacherBatchReviewState.batch?.next_action ?? "").includes("결과표 저장"), `teacher batch next action mismatch: ${teacherBatchReviewState.batch?.next_action}`);
    assert(teacherBatchReviewState.batch?.report_text?.startsWith("# 셈그림 학생 결과표"), "teacher batch report metadata title mismatch");
    assert(teacherBatchReviewState.batch?.report_text?.includes(`수업 코드\t${downloadedPayload.lessons[0].lesson_id}`), `teacher batch report metadata missing lesson id: ${teacherBatchReviewState.batch?.report_text}`);
    assert(teacherBatchReviewState.batch?.report_text?.includes(`차시\t${classroomSessionLabel}`), `teacher batch report metadata missing session: ${teacherBatchReviewState.batch?.report_text}`);
    assert(teacherBatchReviewState.batch?.report_text?.includes(`배포 코드\t${downloadedPayload.manifest.package_id}`), `teacher batch report metadata missing package id: ${teacherBatchReviewState.batch?.report_text}`);
    assert(teacherBatchReviewState.batch?.report_text?.includes("명단 맥락 확인\t통과"), `teacher batch report metadata missing roster context: ${teacherBatchReviewState.batch?.report_text}`);
    assert(teacherBatchReviewState.batch?.report_text?.includes("다음 행동\t결과표 저장 후 입력 비우기"), `teacher batch report metadata missing next action: ${teacherBatchReviewState.batch?.report_text}`);
    assert(teacherBatchReviewState.batch?.report_text?.includes("학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드\t확인 상태\t상태 기록\t비고"), "teacher batch report table header mismatch");
    assert(teacherBatchReviewState.batch?.report_text?.includes(downloadedPayload.lessons[0].lesson_id), `teacher batch report missing lesson id: ${teacherBatchReviewState.batch?.report_text}`);
    assert(teacherBatchReviewState.batch?.report_text?.includes(downloadedPayload.manifest.package_id), `teacher batch report missing package id: ${teacherBatchReviewState.batch?.report_text}`);
    await page.click("#btn-run-student-result-report-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__?.copied === true);
    const teacherBatchCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__ ?? null,
    }));
    assert(teacherBatchCopyState.action?.schema === "seamgrim.student_result_return_report_copy_action.v1", "teacher batch copy schema mismatch");
    assert(teacherBatchCopyState.action?.row_count === 2, `teacher batch copy row count mismatch: ${teacherBatchCopyState.action?.row_count}`);
    assert(teacherBatchCopyState.action?.roster_count === 0 && teacherBatchCopyState.action?.missing_count === 0, "teacher batch copy roster/missing count mismatch");
    assert(teacherBatchCopyState.action?.roster_context_match === true, "teacher batch copy roster context should default to matched");
    assert(teacherBatchCopyState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `teacher batch copy lesson id mismatch: ${teacherBatchCopyState.action?.lesson_id}`);
    assert(teacherBatchCopyState.action?.package_id === downloadedPayload.manifest.package_id, `teacher batch copy package id mismatch: ${teacherBatchCopyState.action?.package_id}`);
    assert(teacherBatchCopyState.action?.session_label === classroomSessionLabel, `teacher batch copy session mismatch: ${teacherBatchCopyState.action?.session_label}`);
    assert(teacherBatchCopyState.copiedText.includes("김가온") && teacherBatchCopyState.copiedText.includes("이하늘"), `teacher batch copied text missing names: ${teacherBatchCopyState.copiedText}`);
    assert(teacherBatchCopyState.copiedText.startsWith("# 셈그림 학생 결과표"), `teacher batch copied text missing metadata title: ${teacherBatchCopyState.copiedText}`);
    assert(teacherBatchCopyState.copiedText.includes("명단 맥락 확인\t통과"), `teacher batch copied text missing roster metadata status: ${teacherBatchCopyState.copiedText}`);
    assert(teacherBatchCopyState.copiedText.includes(classroomSessionLabel), `teacher batch copied text missing session: ${teacherBatchCopyState.copiedText}`);
    assert(teacherBatchCopyState.copiedText.includes(downloadedPayload.lessons[0].lesson_id), `teacher batch copied text missing lesson id: ${teacherBatchCopyState.copiedText}`);
    assert(teacherBatchCopyState.copiedText.includes(downloadedPayload.manifest.package_id), `teacher batch copied text missing package id: ${teacherBatchCopyState.copiedText}`);
    assert(teacherBatchCopyState.copiedText.includes("확인됨"), `teacher batch copied text missing status: ${teacherBatchCopyState.copiedText}`);
    const [studentResultsDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-student-result-report-download"),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_DOWNLOAD_ACTION__?.downloaded === true);
    const studentResultsDownloadPath = await studentResultsDownload.path();
    const studentResultsDownloadedText = studentResultsDownloadPath ? await fs.readFile(studentResultsDownloadPath, "utf-8") : "";
    const teacherBatchDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(teacherBatchDownloadState.action?.schema === "seamgrim.student_result_return_report_download_action.v1", "teacher batch download schema mismatch");
    assert(teacherBatchDownloadState.action?.row_count === 2, `teacher batch download row count mismatch: ${teacherBatchDownloadState.action?.row_count}`);
    assert(teacherBatchDownloadState.action?.accepted_count === 2 && teacherBatchDownloadState.action?.rejected_count === 0, "teacher batch download accepted/rejected mismatch");
    assert(teacherBatchDownloadState.action?.roster_count === 0 && teacherBatchDownloadState.action?.missing_count === 0, "teacher batch download roster/missing count mismatch");
    assert(teacherBatchDownloadState.action?.roster_context_match === true, "teacher batch download roster context should default to matched");
    assert(teacherBatchDownloadState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `teacher batch download lesson id mismatch: ${teacherBatchDownloadState.action?.lesson_id}`);
    assert(teacherBatchDownloadState.action?.package_id === downloadedPayload.manifest.package_id, `teacher batch download package id mismatch: ${teacherBatchDownloadState.action?.package_id}`);
    assert(teacherBatchDownloadState.action?.session_label === classroomSessionLabel, `teacher batch download session mismatch: ${teacherBatchDownloadState.action?.session_label}`);
    assert(String(teacherBatchDownloadState.action?.next_action ?? "").includes("결과표 저장"), `teacher batch download next action mismatch: ${JSON.stringify(teacherBatchDownloadState.action)}`);
    assert(String(teacherBatchDownloadState.action?.file_name ?? "").endsWith(".tsv"), `teacher batch download filename mismatch: ${teacherBatchDownloadState.action?.file_name}`);
    assert(String(teacherBatchDownloadState.action?.file_name ?? "").includes(downloadedPayload.lessons[0].lesson_id), `teacher batch download filename should include lesson id: ${teacherBatchDownloadState.action?.file_name}`);
    assert(String(teacherBatchDownloadState.action?.file_name ?? "").includes("_4_"), `teacher batch download filename should include session marker: ${teacherBatchDownloadState.action?.file_name}`);
    assert(String(teacherBatchDownloadState.action?.file_name ?? "").includes(downloadedPayload.manifest.package_id), `teacher batch download filename should include package id: ${teacherBatchDownloadState.action?.file_name}`);
    assert(studentResultsDownload.suggestedFilename() === teacherBatchDownloadState.action.file_name, "teacher batch suggested filename mismatch");
    assert(studentResultsDownloadedText.startsWith("# 셈그림 학생 결과표"), `teacher batch downloaded text missing metadata title: ${studentResultsDownloadedText}`);
    assert(studentResultsDownloadedText.includes("명단 맥락 확인\t통과"), `teacher batch downloaded text missing roster metadata status: ${studentResultsDownloadedText}`);
    assert(studentResultsDownloadedText.trim() === teacherBatchCopyState.copiedText.trim(), "teacher batch downloaded TSV mismatch");
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);
    await page.setInputFiles("#input-run-student-result-file", {
      name: studentResultsDownload.suggestedFilename(),
      mimeType: "text/tab-separated-values",
      buffer: Buffer.from(studentResultsDownloadedText, "utf-8"),
    });
    await page.waitForFunction((fileName) => window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__?.file_names?.[0] === fileName, studentResultsDownload.suggestedFilename());
    const resultReportReloadState = await page.evaluate(() => ({
      inputText: document.querySelector("#run-student-result-input")?.value ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      action: window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__ ?? null,
    }));
    assert(resultReportReloadState.action?.schema === "seamgrim.student_result_file_load_action.v1", "student result report reload schema mismatch");
    assert(resultReportReloadState.action?.file_count === 1, `student result report reload count mismatch: ${resultReportReloadState.action?.file_count}`);
    assert(resultReportReloadState.action?.result_count === 2 && resultReportReloadState.action?.accepted_count === 2 && resultReportReloadState.action?.rejected_count === 0, `student result report reload preview counts mismatch: ${JSON.stringify(resultReportReloadState.action)}`);
    assert(String(resultReportReloadState.action?.next_action ?? "").includes("결과표 저장"), `student result report reload next action mismatch: ${JSON.stringify(resultReportReloadState.action)}`);
    assert(resultReportReloadState.statusText.includes("학생 결과 2건"), `student result report reload preview count mismatch: ${resultReportReloadState.statusText}`);
    assert(resultReportReloadState.resultReviewDisabled === false, "student result report reload should enable review");
    assert(resultReportReloadState.inputText.trim() === studentResultsDownloadedText.trim(), "student result report reload should populate saved report text");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.accepted_count === 2);
    const resultReportReloadReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(resultReportReloadReviewState.statusText.includes("확인됨 2/2"), `student result report reload review status mismatch: ${resultReportReloadReviewState.statusText}`);
    assert(resultReportReloadReviewState.tableText.includes("김가온") && resultReportReloadReviewState.tableText.includes("이하늘"), `student result report reload table missing names: ${resultReportReloadReviewState.tableText}`);
    assert(resultReportReloadReviewState.batch?.row_count === 2 && resultReportReloadReviewState.batch?.rejected_count === 0, "student result report reload batch counts mismatch");
    assert(String(resultReportReloadReviewState.batch?.report_text ?? "").includes("다음 행동\t결과표 저장 후 입력 비우기"), `student result report reload report should preserve next action: ${resultReportReloadReviewState.batch?.report_text}`);
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);
    const blankCourseRosterText = [
      "# 셈그림 대표 교과 학생 명단",
      `수업 코드\t${downloadedPayload.lessons[0].lesson_id}`,
      `차시\t${classroomSessionLabel}`,
      `배포 코드\t${downloadedPayload.manifest.package_id}`,
      "학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드",
      `\t\t${downloadedPayload.lessons[0].title}\t${downloadedPayload.lessons[0].lesson_id}\t${downloadedPayload.manifest.title}\t${downloadedPayload.manifest.package_id}`,
      `\t\t다른 수업\tother_lesson\t${downloadedPayload.manifest.title}\t${downloadedPayload.manifest.package_id}`,
      `김가온\t${classroomSessionLabel}\t${downloadedPayload.lessons[0].title}\t${downloadedPayload.lessons[0].lesson_id}\t${downloadedPayload.manifest.title}\t${downloadedPayload.manifest.package_id}`,
    ].join("\n");
    await page.fill("#run-student-roster-input", blankCourseRosterText);
    await page.fill("#run-student-result-input", studentResultCopyState.copiedText);
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.row_count === 1);
    const blankCourseRosterReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(blankCourseRosterReviewState.batch?.roster_count === 1, `blank course roster should only count named students: ${JSON.stringify(blankCourseRosterReviewState.batch)}`);
    assert(blankCourseRosterReviewState.batch?.roster_entry_count === 1, `blank course roster entry count mismatch: ${JSON.stringify(blankCourseRosterReviewState.batch)}`);
    assert(blankCourseRosterReviewState.batch?.missing_count === 0, `blank course roster should not create missing lesson-title rows: ${JSON.stringify(blankCourseRosterReviewState.batch)}`);
    assert(blankCourseRosterReviewState.batch?.accepted_count === 1 && blankCourseRosterReviewState.batch?.rejected_count === 0, `blank course roster review count mismatch: ${JSON.stringify(blankCourseRosterReviewState.batch)}`);
    assert(blankCourseRosterReviewState.statusText.includes("확인됨 1/1"), `blank course roster status mismatch: ${blankCourseRosterReviewState.statusText}`);
    assert(
      blankCourseRosterReviewState.summaryText.includes("명단 1명")
        && blankCourseRosterReviewState.summaryText.includes("미제출 0명"),
      `blank course roster summary mismatch: ${blankCourseRosterReviewState.summaryText}`,
    );
    assert(blankCourseRosterReviewState.tableText.includes("김가온"), `blank course roster table missing named student: ${blankCourseRosterReviewState.tableText}`);
    assert(!blankCourseRosterReviewState.tableText.includes("다른 수업") && !blankCourseRosterReviewState.tableText.includes("other_lesson"), `blank course roster should ignore blank lesson rows: ${blankCourseRosterReviewState.tableText}`);
    assert(blankCourseRosterReviewState.batch?.rows?.every((row) => row.student_name !== "다른 수업" && row.student_name !== "other_lesson"), `blank course roster rows should not promote lesson cells: ${JSON.stringify(blankCourseRosterReviewState.batch?.rows)}`);
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);
    await page.setInputFiles("#input-run-student-result-file", [
      {
        name: studentResultDownload.suggestedFilename(),
        mimeType: "text/plain",
        buffer: Buffer.from(studentResultDownloadedText, "utf-8"),
      },
      {
        name: studentResultsDownload.suggestedFilename(),
        mimeType: "text/tab-separated-values",
        buffer: Buffer.from(studentResultsDownloadedText, "utf-8"),
      },
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__?.file_count === 2);
    const mixedResultFileLoadState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      action: window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__ ?? null,
    }));
    assert(mixedResultFileLoadState.action?.result_count === 3, `mixed result file load result count mismatch: ${JSON.stringify(mixedResultFileLoadState.action)}`);
    assert(mixedResultFileLoadState.action?.accepted_count === 1 && mixedResultFileLoadState.action?.rejected_count === 2, `mixed result file load preview counts mismatch: ${JSON.stringify(mixedResultFileLoadState.action)}`);
    assert(mixedResultFileLoadState.statusText.includes("결과 파일 2개 불러옴"), `mixed result file load status mismatch: ${mixedResultFileLoadState.statusText}`);
    assert(mixedResultFileLoadState.statusText.includes("학생 결과 3건"), `mixed result file load result count status mismatch: ${mixedResultFileLoadState.statusText}`);
    assert(mixedResultFileLoadState.statusText.includes("확인 필요 2건"), `mixed result file load rejected status mismatch: ${mixedResultFileLoadState.statusText}`);
    assert(mixedResultFileLoadState.resultReviewDisabled === false, "mixed result file load should enable review");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.row_count === 3);
    const mixedResultReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(mixedResultReviewState.statusText.includes("확인 필요 1/3"), `mixed result review status mismatch: ${mixedResultReviewState.statusText}`);
    assert(mixedResultReviewState.tableText.includes("학생 이름 중복") && mixedResultReviewState.tableText.includes("상태 기록 중복"), `mixed result review should flag duplicates: ${mixedResultReviewState.tableText}`);
    assert(mixedResultReviewState.batch?.accepted_count === 1 && mixedResultReviewState.batch?.rejected_count === 2, "mixed result review counts mismatch");

    const rosterFileText = [
      "# 셈그림 학생 명단 양식",
      `수업 코드\t${downloadedPayload.lessons[0].lesson_id}`,
      `차시\t${classroomSessionLabel}`,
      `배포 코드\t${downloadedPayload.manifest.package_id}`,
      "번호,이름",
      "1,김가온",
      "2,이하늘",
      "3,박다온",
    ].join("\n");
    await page.setInputFiles("#input-run-student-roster-file", {
      name: "classroom-roster.csv",
      mimeType: "text/csv",
      buffer: Buffer.from(rosterFileText, "utf-8"),
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ROSTER_FILE_LOAD_ACTION__?.loaded === true);
    const rosterFileLoadState = await page.evaluate(() => ({
      rosterText: document.querySelector("#run-student-roster-input")?.value ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      activeId: document.activeElement?.id ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      action: window.__SEAMGRIM_STUDENT_ROSTER_FILE_LOAD_ACTION__ ?? null,
    }));
    assert(rosterFileLoadState.action?.schema === "seamgrim.student_roster_file_load_action.v1", "student roster file load schema mismatch");
    assert(rosterFileLoadState.action?.file_count === 1, `student roster file load count mismatch: ${rosterFileLoadState.action?.file_count}`);
    assert(Array.isArray(rosterFileLoadState.action?.file_names) && rosterFileLoadState.action.file_names[0] === "classroom-roster.csv", `student roster file load names mismatch: ${rosterFileLoadState.action?.file_names}`);
    assert(rosterFileLoadState.action?.roster_count === 3, `student roster file load roster count mismatch: ${rosterFileLoadState.action?.roster_count}`);
    assert(rosterFileLoadState.action?.roster_metadata?.lesson_id === downloadedPayload.lessons[0].lesson_id, `student roster file load lesson metadata mismatch: ${JSON.stringify(rosterFileLoadState.action?.roster_metadata)}`);
    assert(rosterFileLoadState.action?.roster_metadata?.session_label === classroomSessionLabel, `student roster file load session metadata mismatch: ${JSON.stringify(rosterFileLoadState.action?.roster_metadata)}`);
    assert(rosterFileLoadState.action?.roster_metadata?.package_id === downloadedPayload.manifest.package_id, `student roster file load package metadata mismatch: ${JSON.stringify(rosterFileLoadState.action?.roster_metadata)}`);
    assert(rosterFileLoadState.action?.roster_context_match === true, "student roster file load context should match current package");
    assert(String(rosterFileLoadState.action?.next_action ?? "").includes("결과 확인"), `student roster file load next action metadata mismatch: ${JSON.stringify(rosterFileLoadState.action)}`);
    assert(rosterFileLoadState.statusText.includes("명단 파일 1개 불러옴"), `student roster file load status mismatch: ${rosterFileLoadState.statusText}`);
    assert(rosterFileLoadState.statusText.includes("학생 결과 3건"), `student roster file load should preserve result count preview: ${rosterFileLoadState.statusText}`);
    assert(rosterFileLoadState.statusText.includes("명단 3명"), `student roster file load count preview mismatch: ${rosterFileLoadState.statusText}`);
    assert(rosterFileLoadState.statusText.includes("다음 행동: 결과 확인"), `student roster file load next action mismatch: ${rosterFileLoadState.statusText}`);
    assert(rosterFileLoadState.resultReviewDisabled === false, "student roster file load should enable review button");
    assert(rosterFileLoadState.activeId === "btn-run-student-result-review", `student roster file load should focus review button: ${rosterFileLoadState.activeId}`);
    assert(rosterFileLoadState.rosterText.trim() === rosterFileText, `student roster file should populate roster textarea: ${rosterFileLoadState.rosterText}`);
    await page.fill("#run-student-result-input", multiStudentText);
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.missing_count === 1);
    const rosterReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      missingReminderDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-copy")?.disabled),
      missingReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-download")?.disabled),
      reviewReminderDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-copy")?.disabled),
      reviewReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-download")?.disabled),
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(rosterReviewState.statusState === "error", `roster review status should require review: ${rosterReviewState.statusState}`);
    assert(rosterReviewState.statusText.includes("미제출 1명"), `roster review status missing absent count: ${rosterReviewState.statusText}`);
    assert(rosterReviewState.summaryText.includes("명단 3명"), `roster review summary missing roster count: ${rosterReviewState.summaryText}`);
    assert(rosterReviewState.summaryText.includes("미제출 1명"), `roster review summary missing absent count: ${rosterReviewState.summaryText}`);
    assert(rosterReviewState.summaryText.includes("다음 행동: 미제출 안내 복사"), `roster review summary missing next action: ${rosterReviewState.summaryText}`);
    assert(rosterReviewState.tableText.includes("박다온"), `roster review table missing absent student: ${rosterReviewState.tableText}`);
    assert(rosterReviewState.tableText.includes(classroomSessionLabel), `roster review table missing session: ${rosterReviewState.tableText}`);
    assert(rosterReviewState.tableText.includes("미제출"), `roster review table missing absent note: ${rosterReviewState.tableText}`);
    assert(rosterReviewState.batch?.accepted === false, "roster batch should require review while a student is missing");
    assert(rosterReviewState.batch?.roster_count === 3, `roster count mismatch: ${rosterReviewState.batch?.roster_count}`);
    assert(rosterReviewState.batch?.row_count === 3, `roster row count mismatch: ${rosterReviewState.batch?.row_count}`);
    assert(rosterReviewState.batch?.accepted_count === 2 && rosterReviewState.batch?.rejected_count === 1, "roster accepted/rejected count mismatch");
    assert(String(rosterReviewState.batch?.next_action ?? "").includes("미제출 안내"), `roster next action mismatch: ${rosterReviewState.batch?.next_action}`);
    assert(rosterReviewState.batch?.rows?.some((row) => row.student_name === "박다온" && row.roster_missing === true), "roster missing row mismatch");
    assert(String(rosterReviewState.batch?.report_text ?? "").includes("박다온"), `roster report missing absent student: ${rosterReviewState.batch?.report_text}`);
    assert(rosterReviewState.missingReminderDisabled === false, "missing reminder should be enabled when roster has absent students");
    assert(rosterReviewState.missingReminderDownloadDisabled === false, "missing reminder download should be enabled when roster has absent students");
    assert(rosterReviewState.reviewReminderDisabled === true, "review reminder should stay disabled for missing-only rows");
    assert(rosterReviewState.reviewReminderDownloadDisabled === true, "review reminder download should stay disabled for missing-only rows");
    await page.evaluate(() => {
      document.querySelector("#btn-run-student-missing-reminder-copy")?.click();
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_MISSING_REMINDER_COPY_ACTION__?.copied === true);
    const missingReminderState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_MISSING_REMINDER_COPY_ACTION__ ?? null,
    }));
    assert(missingReminderState.action?.schema === "seamgrim.student_missing_reminder_copy_action.v1", "missing reminder schema mismatch");
    assert(missingReminderState.action?.missing_count === 1, `missing reminder count mismatch: ${missingReminderState.action?.missing_count}`);
    assert(Array.isArray(missingReminderState.action?.missing_names) && missingReminderState.action.missing_names.join(",") === "박다온", `missing reminder names mismatch: ${missingReminderState.action?.missing_names}`);
    assert(missingReminderState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `missing reminder lesson id mismatch: ${missingReminderState.action?.lesson_id}`);
    assert(missingReminderState.action?.package_id === downloadedPayload.manifest.package_id, `missing reminder package id mismatch: ${missingReminderState.action?.package_id}`);
    assert(missingReminderState.action?.session_label === classroomSessionLabel, `missing reminder session mismatch: ${missingReminderState.action?.session_label}`);
    assert(missingReminderState.action?.roster_context_match === true, "missing reminder roster context should match current package");
    assert(missingReminderState.action?.roster_metadata?.package_id === downloadedPayload.manifest.package_id, `missing reminder roster metadata mismatch: ${JSON.stringify(missingReminderState.action?.roster_metadata)}`);
    assert(missingReminderState.copiedText.includes("미제출 안내"), `missing reminder text missing title: ${missingReminderState.copiedText}`);
    assert(missingReminderState.copiedText.includes(`차시: ${classroomSessionLabel}`), `missing reminder text missing session: ${missingReminderState.copiedText}`);
    assert(missingReminderState.copiedText.includes(`수업 코드: ${downloadedPayload.lessons[0].lesson_id}`), `missing reminder text missing lesson id: ${missingReminderState.copiedText}`);
    assert(missingReminderState.copiedText.includes(`배포 코드: ${downloadedPayload.manifest.package_id}`), `missing reminder text missing package id: ${missingReminderState.copiedText}`);
    assert(missingReminderState.copiedText.includes("대상: 박다온"), `missing reminder text missing student: ${missingReminderState.copiedText}`);
    assert(missingReminderState.copiedText.includes("배포 열기 -> 받은 수업 실행 -> 결과 확인 -> 결과 복사"), `missing reminder text missing flow: ${missingReminderState.copiedText}`);
    const [missingReminderDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-student-missing-reminder-download"),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_MISSING_REMINDER_DOWNLOAD_ACTION__?.downloaded === true);
    const missingReminderDownloadPath = await missingReminderDownload.path();
    const missingReminderDownloadedText = missingReminderDownloadPath ? await fs.readFile(missingReminderDownloadPath, "utf-8") : "";
    const missingReminderDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_MISSING_REMINDER_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(missingReminderDownloadState.action?.schema === "seamgrim.student_missing_reminder_download_action.v1", "missing reminder download schema mismatch");
    assert(missingReminderDownloadState.action?.missing_count === 1, `missing reminder download count mismatch: ${missingReminderDownloadState.action?.missing_count}`);
    assert(Array.isArray(missingReminderDownloadState.action?.missing_names) && missingReminderDownloadState.action.missing_names.join(",") === "박다온", `missing reminder download names mismatch: ${missingReminderDownloadState.action?.missing_names}`);
    assert(missingReminderDownloadState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `missing reminder download lesson id mismatch: ${missingReminderDownloadState.action?.lesson_id}`);
    assert(missingReminderDownloadState.action?.package_id === downloadedPayload.manifest.package_id, `missing reminder download package id mismatch: ${missingReminderDownloadState.action?.package_id}`);
    assert(missingReminderDownloadState.action?.session_label === classroomSessionLabel, `missing reminder download session mismatch: ${missingReminderDownloadState.action?.session_label}`);
    assert(missingReminderDownloadState.action?.roster_context_match === true, "missing reminder download roster context should match current package");
    assert(missingReminderDownloadState.action?.roster_metadata?.package_id === downloadedPayload.manifest.package_id, `missing reminder download roster metadata mismatch: ${JSON.stringify(missingReminderDownloadState.action?.roster_metadata)}`);
    assert(String(missingReminderDownloadState.action?.file_name ?? "").endsWith(".txt"), `missing reminder download filename mismatch: ${missingReminderDownloadState.action?.file_name}`);
    assert(String(missingReminderDownloadState.action?.file_name ?? "").includes(downloadedPayload.manifest.package_id), `missing reminder download filename should include package id: ${missingReminderDownloadState.action?.file_name}`);
    assert(String(missingReminderDownloadState.action?.file_name ?? "").includes("_4_"), `missing reminder download filename should include session marker: ${missingReminderDownloadState.action?.file_name}`);
    assert(missingReminderDownload.suggestedFilename() === missingReminderDownloadState.action.file_name, "missing reminder suggested filename mismatch");
    assert(missingReminderDownloadedText.trim() === missingReminderState.copiedText.trim(), "missing reminder downloaded text mismatch");

    await page.click("#btn-run-student-roster-only-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ROSTER_ONLY_CLEAR_ACTION__?.cleared === true);
    const rosterOnlyClearState = await page.evaluate(() => ({
      rosterText: document.querySelector("#run-student-roster-input")?.value ?? "",
      resultText: document.querySelector("#run-student-result-input")?.value ?? "",
      activeId: document.activeElement?.id ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      summaryDisplay: getComputedStyle(document.querySelector("[data-run-student-result-summary]")).display,
      tableDisplay: getComputedStyle(document.querySelector("[data-run-student-result-table]")).display,
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      copyDisabled: Boolean(document.querySelector("#btn-run-student-result-report-copy")?.disabled),
      downloadDisabled: Boolean(document.querySelector("#btn-run-student-result-report-download")?.disabled),
      missingReminderDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-copy")?.disabled),
      reviewReminderDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-copy")?.disabled),
      action: window.__SEAMGRIM_STUDENT_ROSTER_ONLY_CLEAR_ACTION__ ?? null,
      review: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__ ?? null,
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(rosterOnlyClearState.action?.schema === "seamgrim.student_roster_only_clear_action.v1", "student roster-only clear schema mismatch");
    assert(rosterOnlyClearState.action?.result_preserved === true && rosterOnlyClearState.action?.result_count === 2, `student roster-only clear action mismatch: ${JSON.stringify(rosterOnlyClearState.action)}`);
    assert(rosterOnlyClearState.rosterText === "", "student roster-only clear should empty only roster input");
    assert(rosterOnlyClearState.resultText.trim() === multiStudentText, "student roster-only clear should preserve result text");
    assert(rosterOnlyClearState.statusState === "idle", `student roster-only clear status state mismatch: ${rosterOnlyClearState.statusState}`);
    assert(rosterOnlyClearState.statusText.includes("명단만 비움") && rosterOnlyClearState.statusText.includes("학생 결과 2건") && rosterOnlyClearState.statusText.includes("결과 확인"), `student roster-only clear status mismatch: ${rosterOnlyClearState.statusText}`);
    assert(rosterOnlyClearState.summaryDisplay === "none" && rosterOnlyClearState.tableDisplay === "none", `student roster-only clear should hide outputs: ${rosterOnlyClearState.summaryDisplay}/${rosterOnlyClearState.tableDisplay}`);
    assert(rosterOnlyClearState.resultReviewDisabled === false, "student roster-only clear should keep result review enabled");
    assert(rosterOnlyClearState.copyDisabled === true && rosterOnlyClearState.downloadDisabled === true, "student roster-only clear should disable report actions");
    assert(rosterOnlyClearState.missingReminderDisabled === true && rosterOnlyClearState.reviewReminderDisabled === true, "student roster-only clear should disable reminder actions");
    assert(rosterOnlyClearState.review === null && rosterOnlyClearState.batch === null, "student roster-only clear should clear review instrumentation");
    assert(rosterOnlyClearState.activeId === "run-student-roster-input", `student roster-only clear should focus roster input: ${rosterOnlyClearState.activeId}`);
    await page.fill("#run-student-roster-input", rosterFileText);

    await page.click("#btn-run-student-result-only-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_ONLY_CLEAR_ACTION__?.cleared === true);
    const resultOnlyClearState = await page.evaluate(() => ({
      rosterText: document.querySelector("#run-student-roster-input")?.value ?? "",
      resultText: document.querySelector("#run-student-result-input")?.value ?? "",
      activeId: document.activeElement?.id ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      summaryDisplay: getComputedStyle(document.querySelector("[data-run-student-result-summary]")).display,
      tableDisplay: getComputedStyle(document.querySelector("[data-run-student-result-table]")).display,
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      copyDisabled: Boolean(document.querySelector("#btn-run-student-result-report-copy")?.disabled),
      downloadDisabled: Boolean(document.querySelector("#btn-run-student-result-report-download")?.disabled),
      missingReminderDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-copy")?.disabled),
      reviewReminderDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-copy")?.disabled),
      action: window.__SEAMGRIM_STUDENT_RESULT_ONLY_CLEAR_ACTION__ ?? null,
      review: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__ ?? null,
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(resultOnlyClearState.action?.schema === "seamgrim.student_result_only_clear_action.v1", "student result-only clear schema mismatch");
    assert(resultOnlyClearState.action?.roster_preserved === true && resultOnlyClearState.action?.roster_count === 3, `student result-only clear action mismatch: ${JSON.stringify(resultOnlyClearState.action)}`);
    assert(resultOnlyClearState.rosterText.trim() === rosterFileText, "student result-only clear should preserve roster text");
    assert(resultOnlyClearState.resultText === "", "student result-only clear should empty only result input");
    assert(resultOnlyClearState.statusState === "idle", `student result-only clear status state mismatch: ${resultOnlyClearState.statusState}`);
    assert(resultOnlyClearState.statusText.includes("결과만 비움") && resultOnlyClearState.statusText.includes("명단 3명") && resultOnlyClearState.statusText.includes("학생 결과 붙여넣기"), `student result-only clear status mismatch: ${resultOnlyClearState.statusText}`);
    assert(resultOnlyClearState.summaryDisplay === "none" && resultOnlyClearState.tableDisplay === "none", `student result-only clear should hide outputs: ${resultOnlyClearState.summaryDisplay}/${resultOnlyClearState.tableDisplay}`);
    assert(resultOnlyClearState.resultReviewDisabled === false, "student result-only clear should keep roster review enabled");
    assert(resultOnlyClearState.copyDisabled === true && resultOnlyClearState.downloadDisabled === true, "student result-only clear should disable report actions");
    assert(resultOnlyClearState.missingReminderDisabled === true && resultOnlyClearState.reviewReminderDisabled === true, "student result-only clear should disable reminder actions");
    assert(resultOnlyClearState.review === null && resultOnlyClearState.batch === null, "student result-only clear should clear review instrumentation");
    assert(resultOnlyClearState.activeId === "run-student-result-input", `student result-only clear should focus result input: ${resultOnlyClearState.activeId}`);

    const duplicatedStudentText = [
      studentResultCopyState.copiedText,
      studentResultCopyState.copiedText,
    ].join("\n\n");
    await page.fill("#run-student-roster-input", "");
    await page.fill("#run-student-result-input", duplicatedStudentText);
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.rejected_count === 2);
    const duplicateReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      reviewReminderDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-copy")?.disabled),
      reviewReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-download")?.disabled),
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(duplicateReviewState.statusState === "error", `duplicate review status should require review: ${duplicateReviewState.statusState}`);
    assert(duplicateReviewState.statusText.includes("확인 필요 0/2"), `duplicate review status text mismatch: ${duplicateReviewState.statusText}`);
    assert(duplicateReviewState.summaryText.includes("다음 행동: 확인 필요 안내 복사"), `duplicate review summary missing next action: ${duplicateReviewState.summaryText}`);
    assert(duplicateReviewState.tableText.includes("학생 이름 중복"), `duplicate review table missing name note: ${duplicateReviewState.tableText}`);
    assert(duplicateReviewState.tableText.includes("상태 기록 중복"), `duplicate review table missing hash note: ${duplicateReviewState.tableText}`);
    assert(duplicateReviewState.batch?.accepted === false, "duplicate batch should not be accepted");
    assert(duplicateReviewState.batch?.rows?.every((row) => row.duplicate_student_name === true && row.duplicate_state_hash === true), "duplicate flags mismatch");
    assert(String(duplicateReviewState.batch?.next_action ?? "").includes("확인 필요 안내"), `duplicate next action mismatch: ${duplicateReviewState.batch?.next_action}`);
    assert(String(duplicateReviewState.batch?.report_text ?? "").includes("학생 이름 중복|상태 기록 중복"), `duplicate report missing notes: ${duplicateReviewState.batch?.report_text}`);
    assert(duplicateReviewState.reviewReminderDisabled === false, "review reminder should be enabled for duplicate rows");
    assert(duplicateReviewState.reviewReminderDownloadDisabled === false, "review reminder download should be enabled for duplicate rows");
    await page.click("#btn-run-student-review-reminder-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_COPY_ACTION__?.copied === true);
    const reviewReminderState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_COPY_ACTION__ ?? null,
    }));
    assert(reviewReminderState.action?.schema === "seamgrim.student_review_reminder_copy_action.v1", "review reminder schema mismatch");
    assert(reviewReminderState.action?.review_count === 2, `review reminder count mismatch: ${reviewReminderState.action?.review_count}`);
    assert(Array.isArray(reviewReminderState.action?.student_names) && reviewReminderState.action.student_names.join(",") === "김가온,김가온", `review reminder names mismatch: ${reviewReminderState.action?.student_names}`);
    assert(reviewReminderState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `review reminder lesson id mismatch: ${reviewReminderState.action?.lesson_id}`);
    assert(reviewReminderState.action?.package_id === downloadedPayload.manifest.package_id, `review reminder package id mismatch: ${reviewReminderState.action?.package_id}`);
    assert(reviewReminderState.action?.session_label === classroomSessionLabel, `review reminder session mismatch: ${reviewReminderState.action?.session_label}`);
    assert(reviewReminderState.action?.roster_context_match === true, "review reminder roster context should default to matched");
    assert(reviewReminderState.copiedText.includes("확인 필요 안내"), `review reminder text missing title: ${reviewReminderState.copiedText}`);
    assert(reviewReminderState.copiedText.includes(`차시: ${classroomSessionLabel}`), `review reminder text missing session: ${reviewReminderState.copiedText}`);
    assert(reviewReminderState.copiedText.includes(`수업 코드: ${downloadedPayload.lessons[0].lesson_id}`), `review reminder text missing lesson id: ${reviewReminderState.copiedText}`);
    assert(reviewReminderState.copiedText.includes(`배포 코드: ${downloadedPayload.manifest.package_id}`), `review reminder text missing package id: ${reviewReminderState.copiedText}`);
    assert(reviewReminderState.copiedText.includes("학생 이름 중복"), `review reminder text missing name note: ${reviewReminderState.copiedText}`);
    assert(reviewReminderState.copiedText.includes("상태 기록 중복"), `review reminder text missing hash note: ${reviewReminderState.copiedText}`);
    assert(reviewReminderState.copiedText.includes("다시 제출"), `review reminder text missing resubmit instruction: ${reviewReminderState.copiedText}`);
    const [reviewReminderDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-student-review-reminder-download"),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_DOWNLOAD_ACTION__?.downloaded === true);
    const reviewReminderDownloadPath = await reviewReminderDownload.path();
    const reviewReminderDownloadedText = reviewReminderDownloadPath ? await fs.readFile(reviewReminderDownloadPath, "utf-8") : "";
    const reviewReminderDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(reviewReminderDownloadState.action?.schema === "seamgrim.student_review_reminder_download_action.v1", "review reminder download schema mismatch");
    assert(reviewReminderDownloadState.action?.review_count === 2, `review reminder download count mismatch: ${reviewReminderDownloadState.action?.review_count}`);
    assert(Array.isArray(reviewReminderDownloadState.action?.student_names) && reviewReminderDownloadState.action.student_names.join(",") === "김가온,김가온", `review reminder download names mismatch: ${reviewReminderDownloadState.action?.student_names}`);
    assert(reviewReminderDownloadState.action?.lesson_id === downloadedPayload.lessons[0].lesson_id, `review reminder download lesson id mismatch: ${reviewReminderDownloadState.action?.lesson_id}`);
    assert(reviewReminderDownloadState.action?.package_id === downloadedPayload.manifest.package_id, `review reminder download package id mismatch: ${reviewReminderDownloadState.action?.package_id}`);
    assert(reviewReminderDownloadState.action?.session_label === classroomSessionLabel, `review reminder download session mismatch: ${reviewReminderDownloadState.action?.session_label}`);
    assert(reviewReminderDownloadState.action?.roster_context_match === true, "review reminder download roster context should default to matched");
    assert(String(reviewReminderDownloadState.action?.file_name ?? "").endsWith(".txt"), `review reminder download filename mismatch: ${reviewReminderDownloadState.action?.file_name}`);
    assert(String(reviewReminderDownloadState.action?.file_name ?? "").includes(downloadedPayload.manifest.package_id), `review reminder download filename should include package id: ${reviewReminderDownloadState.action?.file_name}`);
    assert(String(reviewReminderDownloadState.action?.file_name ?? "").includes("_4_"), `review reminder download filename should include session marker: ${reviewReminderDownloadState.action?.file_name}`);
    assert(reviewReminderDownload.suggestedFilename() === reviewReminderDownloadState.action.file_name, "review reminder suggested filename mismatch");
    assert(reviewReminderDownloadedText.trim() === reviewReminderState.copiedText.trim(), "review reminder downloaded text mismatch");
    await page.fill("#run-student-result-input", studentResultCopyState.copiedText);
    const staleResetState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      summaryDisplay: getComputedStyle(document.querySelector("[data-run-student-result-summary]")).display,
      tableDisplay: getComputedStyle(document.querySelector("[data-run-student-result-table]")).display,
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      copyDisabled: Boolean(document.querySelector("#btn-run-student-result-report-copy")?.disabled),
      downloadDisabled: Boolean(document.querySelector("#btn-run-student-result-report-download")?.disabled),
      missingReminderDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-copy")?.disabled),
      missingReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-download")?.disabled),
      reviewReminderDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-copy")?.disabled),
      reviewReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-download")?.disabled),
      review: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__ ?? null,
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(staleResetState.statusState === "idle", `stale reset status state mismatch: ${staleResetState.statusState}`);
    assert(staleResetState.statusText.includes("입력 수정됨") && staleResetState.statusText.includes("결과 확인"), `stale reset status text mismatch: ${staleResetState.statusText}`);
    assert(staleResetState.summaryDisplay === "none" && staleResetState.tableDisplay === "none", `stale reset should hide previous outputs: ${staleResetState.summaryDisplay}/${staleResetState.tableDisplay}`);
    assert(staleResetState.resultReviewDisabled === false, "stale reset should keep review enabled for edited input");
    assert(staleResetState.copyDisabled === true && staleResetState.downloadDisabled === true, "stale reset should disable report actions");
    assert(staleResetState.missingReminderDisabled === true && staleResetState.reviewReminderDisabled === true, "stale reset should disable reminder actions");
    assert(staleResetState.missingReminderDownloadDisabled === true && staleResetState.reviewReminderDownloadDisabled === true, "stale reset should disable reminder downloads");
    assert(staleResetState.review === null && staleResetState.batch === null, "stale reset should clear review instrumentation");
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);
    const clearReviewState = await page.evaluate(() => ({
      rosterText: document.querySelector("#run-student-roster-input")?.value ?? "",
      resultText: document.querySelector("#run-student-result-input")?.value ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      summaryDisplay: getComputedStyle(document.querySelector("[data-run-student-result-summary]")).display,
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      tableDisplay: getComputedStyle(document.querySelector("[data-run-student-result-table]")).display,
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      copyDisabled: Boolean(document.querySelector("#btn-run-student-result-report-copy")?.disabled),
      downloadDisabled: Boolean(document.querySelector("#btn-run-student-result-report-download")?.disabled),
      missingReminderDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-copy")?.disabled),
      missingReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-download")?.disabled),
      reviewReminderDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-copy")?.disabled),
      reviewReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-download")?.disabled),
      review: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REVIEW__ ?? null,
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
      action: window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__ ?? null,
    }));
    assert(clearReviewState.action?.schema === "seamgrim.student_result_clear_action.v1", "student result clear schema mismatch");
    assert(clearReviewState.rosterText === "" && clearReviewState.resultText === "", "student result clear should empty inputs");
    assert(clearReviewState.statusText === "결과 대기" && clearReviewState.statusState === "idle", `student result clear status mismatch: ${clearReviewState.statusState}/${clearReviewState.statusText}`);
    assert(clearReviewState.summaryDisplay === "none" && clearReviewState.summaryText === "", `student result clear summary mismatch: ${clearReviewState.summaryDisplay}/${clearReviewState.summaryText}`);
    assert(clearReviewState.tableDisplay === "none" && clearReviewState.tableText === "", `student result clear table mismatch: ${clearReviewState.tableDisplay}/${clearReviewState.tableText}`);
    assert(clearReviewState.resultReviewDisabled === true, "student result clear should disable review action");
    assert(clearReviewState.copyDisabled === true && clearReviewState.downloadDisabled === true, "student result clear should disable report actions");
    assert(clearReviewState.missingReminderDisabled === true && clearReviewState.reviewReminderDisabled === true, "student result clear should disable reminder actions");
    assert(clearReviewState.missingReminderDownloadDisabled === true && clearReviewState.reviewReminderDownloadDisabled === true, "student result clear should disable reminder downloads");
    assert(clearReviewState.review === null && clearReviewState.batch === null, "student result clear should clear review instrumentation");

    const duplicateRosterText = [
      "# 셈그림 학생 명단 양식",
      `수업 코드\t${downloadedPayload.lessons[0].lesson_id}`,
      `차시\t${classroomSessionLabel}`,
      `배포 코드\t${downloadedPayload.manifest.package_id}`,
      "번호\t이름",
      "1\t김가온",
      "2\t김가온",
      "3\t이하늘",
    ].join("\n");
    await page.setInputFiles("#input-run-student-roster-file", {
      name: "duplicate-roster.tsv",
      mimeType: "text/tab-separated-values",
      buffer: Buffer.from(duplicateRosterText, "utf-8"),
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_ROSTER_FILE_LOAD_ACTION__?.file_names?.[0] === "duplicate-roster.tsv");
    const duplicateRosterLoadState = await page.evaluate(() => ({
      rosterText: document.querySelector("#run-student-roster-input")?.value ?? "",
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
      action: window.__SEAMGRIM_STUDENT_ROSTER_FILE_LOAD_ACTION__ ?? null,
    }));
    assert(duplicateRosterLoadState.action?.loaded === true, "duplicate roster load should succeed");
    assert(duplicateRosterLoadState.action?.roster_entry_count === 3, `duplicate roster entry count mismatch: ${JSON.stringify(duplicateRosterLoadState.action)}`);
    assert(duplicateRosterLoadState.action?.roster_count === 2, `duplicate roster deduped count mismatch: ${JSON.stringify(duplicateRosterLoadState.action)}`);
    assert(duplicateRosterLoadState.action?.duplicate_roster_count === 1, `duplicate roster count mismatch: ${JSON.stringify(duplicateRosterLoadState.action)}`);
    assert(Array.isArray(duplicateRosterLoadState.action?.duplicate_roster_names) && duplicateRosterLoadState.action.duplicate_roster_names.join(",") === "김가온", `duplicate roster names mismatch: ${JSON.stringify(duplicateRosterLoadState.action)}`);
    assert(String(duplicateRosterLoadState.action?.next_action ?? "").includes("학생 결과 붙여넣기"), `duplicate roster load next action mismatch: ${JSON.stringify(duplicateRosterLoadState.action)}`);
    assert(duplicateRosterLoadState.statusText.includes("명단 2명") && duplicateRosterLoadState.statusText.includes("명단 중복 1명"), `duplicate roster status mismatch: ${duplicateRosterLoadState.statusText}`);
    assert(duplicateRosterLoadState.resultReviewDisabled === false, "duplicate roster load should keep review enabled");
    assert(duplicateRosterLoadState.rosterText.trim() === duplicateRosterText, "duplicate roster load should preserve source text");
    await page.fill("#run-student-result-input", multiStudentText);
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.duplicate_roster_count === 1);
    const duplicateRosterReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(duplicateRosterReviewState.batch?.roster_entry_count === 3, `duplicate roster review entry count mismatch: ${JSON.stringify(duplicateRosterReviewState.batch)}`);
    assert(duplicateRosterReviewState.batch?.roster_count === 2, `duplicate roster review deduped count mismatch: ${JSON.stringify(duplicateRosterReviewState.batch)}`);
    assert(duplicateRosterReviewState.batch?.accepted === false, "duplicate roster should make the batch require teacher action");
    assert(duplicateRosterReviewState.batch?.accepted_count === 2 && duplicateRosterReviewState.batch?.rejected_count === 0, `duplicate roster should keep valid result rows while blocking batch: ${JSON.stringify(duplicateRosterReviewState.batch)}`);
    assert(duplicateRosterReviewState.batch?.duplicate_roster_count === 1, `duplicate roster review count mismatch: ${JSON.stringify(duplicateRosterReviewState.batch)}`);
    assert(Array.isArray(duplicateRosterReviewState.batch?.duplicate_roster_names) && duplicateRosterReviewState.batch.duplicate_roster_names.join(",") === "김가온", `duplicate roster review names mismatch: ${JSON.stringify(duplicateRosterReviewState.batch)}`);
    assert(duplicateRosterReviewState.statusText.includes("명단 확인 필요"), `duplicate roster review status should require roster action: ${duplicateRosterReviewState.statusText}`);
    assert(duplicateRosterReviewState.statusText.includes("명단 중복 1명"), `duplicate roster review status should include duplicate count: ${duplicateRosterReviewState.statusText}`);
    assert(duplicateRosterReviewState.statusText.includes("김가온"), `duplicate roster review status should include duplicate name: ${duplicateRosterReviewState.statusText}`);
    assert(duplicateRosterReviewState.summaryText.includes("명단 중복 1명"), `duplicate roster review summary should include duplicate count: ${duplicateRosterReviewState.summaryText}`);
    assert(duplicateRosterReviewState.summaryText.includes("김가온"), `duplicate roster review summary should include duplicate name: ${duplicateRosterReviewState.summaryText}`);
    assert(duplicateRosterReviewState.summaryText.includes("명단 중복") && duplicateRosterReviewState.summaryText.includes("정리"), `duplicate roster review summary should guide next action: ${duplicateRosterReviewState.summaryText}`);
    assert(duplicateRosterReviewState.tableText.includes("김가온") && duplicateRosterReviewState.tableText.includes("이하늘"), `duplicate roster review table missing names: ${duplicateRosterReviewState.tableText}`);
    await page.click("#btn-run-student-result-report-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__?.duplicate_roster_count === 1);
    const duplicateRosterCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__ ?? null,
    }));
    assert(duplicateRosterCopyState.action?.duplicate_roster_count === 1, `duplicate roster report copy count mismatch: ${JSON.stringify(duplicateRosterCopyState.action)}`);
    assert(Array.isArray(duplicateRosterCopyState.action?.duplicate_roster_names) && duplicateRosterCopyState.action.duplicate_roster_names.join(",") === "김가온", `duplicate roster report copy names mismatch: ${JSON.stringify(duplicateRosterCopyState.action)}`);
    assert(duplicateRosterCopyState.copiedText.includes("명단 중복 확인\t필요"), `duplicate roster copied report should preserve duplicate status: ${duplicateRosterCopyState.copiedText}`);
    assert(duplicateRosterCopyState.copiedText.includes("명단 중복 이름\t김가온"), `duplicate roster copied report should preserve duplicate names: ${duplicateRosterCopyState.copiedText}`);
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);
    await page.setInputFiles("#input-run-student-result-file", {
      name: "duplicate-roster-results.tsv",
      mimeType: "text/tab-separated-values",
      buffer: Buffer.from(duplicateRosterCopyState.copiedText, "utf-8"),
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__?.file_names?.[0] === "duplicate-roster-results.tsv");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.duplicate_roster_count === 1);
    const duplicateRosterReportReloadState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(duplicateRosterReportReloadState.statusText.includes("명단 중복 1명"), `duplicate roster report reload status mismatch: ${duplicateRosterReportReloadState.statusText}`);
    assert(duplicateRosterReportReloadState.statusText.includes("명단 확인 필요"), `duplicate roster report reload should require roster action: ${duplicateRosterReportReloadState.statusText}`);
    assert(duplicateRosterReportReloadState.statusText.includes("김가온"), `duplicate roster report reload status should include duplicate name: ${duplicateRosterReportReloadState.statusText}`);
    assert(duplicateRosterReportReloadState.summaryText.includes("명단 중복 1명"), `duplicate roster report reload summary mismatch: ${duplicateRosterReportReloadState.summaryText}`);
    assert(duplicateRosterReportReloadState.summaryText.includes("김가온"), `duplicate roster report reload summary should include duplicate name: ${duplicateRosterReportReloadState.summaryText}`);
    assert(duplicateRosterReportReloadState.summaryText.includes("명단 중복") && duplicateRosterReportReloadState.summaryText.includes("정리"), `duplicate roster report reload should guide next action: ${duplicateRosterReportReloadState.summaryText}`);
    assert(duplicateRosterReportReloadState.batch?.accepted === false, "duplicate roster report reload should keep batch blocked");
    assert(duplicateRosterReportReloadState.batch?.accepted_count === 2 && duplicateRosterReportReloadState.batch?.rejected_count === 0, `duplicate roster report reload valid row counts mismatch: ${JSON.stringify(duplicateRosterReportReloadState.batch)}`);
    assert(duplicateRosterReportReloadState.batch?.duplicate_roster_count === 1, `duplicate roster report reload count mismatch: ${JSON.stringify(duplicateRosterReportReloadState.batch)}`);
    assert(Array.isArray(duplicateRosterReportReloadState.batch?.duplicate_roster_names) && duplicateRosterReportReloadState.batch.duplicate_roster_names.join(",") === "김가온", `duplicate roster report reload names mismatch: ${JSON.stringify(duplicateRosterReportReloadState.batch)}`);
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);

    const wrongRosterPackageText = rosterFileText.replace(
      `배포 코드\t${downloadedPayload.manifest.package_id}`,
      "배포 코드\twrong.classroom.package",
    );
    await page.fill("#run-student-roster-input", wrongRosterPackageText);
    await page.fill("#run-student-result-input", multiStudentText);
    const wrongRosterInputState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
    }));
    assert(wrongRosterInputState.statusState === "idle", `wrong roster status state mismatch: ${wrongRosterInputState.statusState}`);
    assert(wrongRosterInputState.statusText.includes("명단 3명"), `wrong roster status should keep roster count: ${wrongRosterInputState.statusText}`);
    assert(wrongRosterInputState.statusText.includes("명단 배포 코드 확인"), `wrong roster status should show metadata mismatch: ${wrongRosterInputState.statusText}`);
    assert(wrongRosterInputState.resultReviewDisabled === false, "wrong roster input should keep review action enabled");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.roster_context_match === false);
    const wrongRosterReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(wrongRosterReviewState.statusText.includes("확인 필요 0/3"), `wrong roster review status mismatch: ${wrongRosterReviewState.statusText}`);
    assert(wrongRosterReviewState.tableText.includes("명단 배포 코드 확인"), `wrong roster review table missing note: ${wrongRosterReviewState.tableText}`);
    assert(wrongRosterReviewState.batch?.roster_metadata?.package_id === "wrong.classroom.package", `wrong roster metadata mismatch: ${JSON.stringify(wrongRosterReviewState.batch?.roster_metadata)}`);
    assert(wrongRosterReviewState.batch?.roster_package_id_match === false, "wrong roster package should not match current package");
    await page.click("#btn-run-student-result-report-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__?.roster_context_match === false);
    const wrongRosterCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__ ?? null,
    }));
    assert(wrongRosterCopyState.action?.roster_metadata?.package_id === "wrong.classroom.package", `wrong roster copy metadata mismatch: ${JSON.stringify(wrongRosterCopyState.action?.roster_metadata)}`);
    assert(wrongRosterCopyState.action?.roster_package_id_match === false, "wrong roster copy package should not match current package");
    assert(wrongRosterCopyState.copiedText.includes("명단 맥락 확인\t필요"), `wrong roster copied report should preserve context status: ${wrongRosterCopyState.copiedText}`);
    await page.click("#btn-run-student-review-reminder-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_COPY_ACTION__?.roster_context_match === false);
    const wrongRosterReviewReminderState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_COPY_ACTION__ ?? null,
    }));
    assert(wrongRosterReviewReminderState.action?.roster_metadata?.package_id === "wrong.classroom.package", `wrong roster review reminder metadata mismatch: ${JSON.stringify(wrongRosterReviewReminderState.action?.roster_metadata)}`);
    assert(wrongRosterReviewReminderState.action?.roster_package_id_match === false, "wrong roster review reminder package should not match current package");
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);
    await page.setInputFiles("#input-run-student-result-file", {
      name: "wrong-roster-student-results.tsv",
      mimeType: "text/tab-separated-values",
      buffer: Buffer.from(wrongRosterCopyState.copiedText, "utf-8"),
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__?.file_names?.[0] === "wrong-roster-student-results.tsv");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.rejected_count === 3);
    const wrongRosterReportReloadState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(wrongRosterReportReloadState.statusText.includes("확인 필요 0/3"), `wrong roster report reload status mismatch: ${wrongRosterReportReloadState.statusText}`);
    assert(wrongRosterReportReloadState.tableText.includes("명단 배포 코드 확인"), `wrong roster report reload should preserve notes: ${wrongRosterReportReloadState.tableText}`);
    assert(wrongRosterReportReloadState.batch?.accepted_count === 0 && wrongRosterReportReloadState.batch?.rejected_count === 3, "wrong roster report reload counts mismatch");
    assert(wrongRosterReportReloadState.batch?.roster_context_match === false, "wrong roster report reload should preserve roster context mismatch");
    assert(wrongRosterReportReloadState.batch?.roster_package_id_match === false, "wrong roster report reload should preserve roster package mismatch");
    await page.fill("#run-student-roster-input", rosterFileText);
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.roster_context_match === true);
    const correctedRosterReportReloadState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(correctedRosterReportReloadState.statusText.includes("확인 필요 2/3"), `corrected roster report reload status mismatch: ${correctedRosterReportReloadState.statusText}`);
    assert(!correctedRosterReportReloadState.tableText.includes("명단 배포 코드 확인"), `corrected roster should clear stale report roster notes: ${correctedRosterReportReloadState.tableText}`);
    assert(correctedRosterReportReloadState.tableText.includes("박다온") && correctedRosterReportReloadState.tableText.includes("미제출"), `corrected roster should keep missing student note: ${correctedRosterReportReloadState.tableText}`);
    assert(correctedRosterReportReloadState.batch?.accepted_count === 2 && correctedRosterReportReloadState.batch?.rejected_count === 1, `corrected roster report reload counts mismatch: ${JSON.stringify(correctedRosterReportReloadState.batch)}`);
    assert(correctedRosterReportReloadState.batch?.missing_count === 1, `corrected roster report reload missing count mismatch: ${JSON.stringify(correctedRosterReportReloadState.batch)}`);
    assert(correctedRosterReportReloadState.batch?.roster_context_match === true, "corrected roster should override stale report roster context mismatch");
    assert(correctedRosterReportReloadState.batch?.roster_package_id_match === true, "corrected roster package should match current package");
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);

    const oneStudentRosterText = [
      "# 셈그림 학생 명단 양식",
      `수업 코드\t${downloadedPayload.lessons[0].lesson_id}`,
      `차시\t${classroomSessionLabel}`,
      `배포 코드\t${downloadedPayload.manifest.package_id}`,
      "번호,이름",
      "1,김가온",
    ].join("\n");
    const twoStudentRosterText = [
      "# 셈그림 학생 명단 양식",
      `수업 코드\t${downloadedPayload.lessons[0].lesson_id}`,
      `차시\t${classroomSessionLabel}`,
      `배포 코드\t${downloadedPayload.manifest.package_id}`,
      "번호,이름",
      "1,김가온",
      "2,이하늘",
    ].join("\n");
    await page.fill("#run-student-roster-input", oneStudentRosterText);
    await page.fill("#run-student-result-input", multiStudentText);
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.extra_count === 1);
    const extraStudentReviewState = await page.evaluate(() => ({
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(extraStudentReviewState.tableText.includes("이하늘") && extraStudentReviewState.tableText.includes("명단에 없음"), `extra student review should flag roster extra: ${extraStudentReviewState.tableText}`);
    assert(extraStudentReviewState.batch?.accepted_count === 1 && extraStudentReviewState.batch?.rejected_count === 1 && extraStudentReviewState.batch?.extra_count === 1, `extra student review counts mismatch: ${JSON.stringify(extraStudentReviewState.batch)}`);
    await page.click("#btn-run-student-result-report-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__?.rejected_count === 1);
    const extraStudentCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__ ?? null,
    }));
    assert(extraStudentCopyState.copiedText.includes("명단에 없음"), `extra student copied report should preserve extra note: ${extraStudentCopyState.copiedText}`);
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);
    await page.setInputFiles("#input-run-student-result-file", {
      name: "extra-student-results.tsv",
      mimeType: "text/tab-separated-values",
      buffer: Buffer.from(extraStudentCopyState.copiedText, "utf-8"),
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__?.file_names?.[0] === "extra-student-results.tsv");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.rejected_count === 1);
    const extraStudentReportReloadState = await page.evaluate(() => ({
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(extraStudentReportReloadState.tableText.includes("명단에 없음"), `extra student report reload should preserve extra note before roster correction: ${extraStudentReportReloadState.tableText}`);
    assert(extraStudentReportReloadState.batch?.accepted_count === 1 && extraStudentReportReloadState.batch?.rejected_count === 1, `extra student report reload counts mismatch: ${JSON.stringify(extraStudentReportReloadState.batch)}`);
    await page.fill("#run-student-roster-input", twoStudentRosterText);
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.accepted === true);
    const correctedExtraStudentReportReloadState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(correctedExtraStudentReportReloadState.statusText.includes("확인됨 2/2"), `corrected extra student status mismatch: ${correctedExtraStudentReportReloadState.statusText}`);
    assert(!correctedExtraStudentReportReloadState.tableText.includes("명단에 없음"), `corrected extra student should clear stale extra note: ${correctedExtraStudentReportReloadState.tableText}`);
    assert(correctedExtraStudentReportReloadState.batch?.accepted_count === 2 && correctedExtraStudentReportReloadState.batch?.rejected_count === 0 && correctedExtraStudentReportReloadState.batch?.extra_count === 0, `corrected extra student counts mismatch: ${JSON.stringify(correctedExtraStudentReportReloadState.batch)}`);
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);

    await page.fill("#run-student-roster-input", rosterFileText);
    const rosterOnlyInputState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      resultReviewDisabled: Boolean(document.querySelector("#btn-run-student-result-review")?.disabled),
    }));
    assert(rosterOnlyInputState.statusState === "idle", `roster-only input status state mismatch: ${rosterOnlyInputState.statusState}`);
    assert(
      rosterOnlyInputState.statusText.includes("입력 준비됨")
        && rosterOnlyInputState.statusText.includes("학생 결과 대기")
        && rosterOnlyInputState.statusText.includes("명단 3명")
        && rosterOnlyInputState.statusText.includes("학생 결과 붙여넣기"),
      `roster-only input status mismatch: ${rosterOnlyInputState.statusText}`,
    );
    assert(rosterOnlyInputState.resultReviewDisabled === false, "roster-only input should enable review action");
    await page.click("#btn-run-student-result-review");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.missing_count === 3);
    const rosterOnlyReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      statusState: document.querySelector("[data-run-student-result-status]")?.dataset?.state ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      tableText: document.querySelector("[data-run-student-result-table]")?.textContent?.trim() ?? "",
      missingReminderDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-copy")?.disabled),
      missingReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-missing-reminder-download")?.disabled),
      reviewReminderDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-copy")?.disabled),
      reviewReminderDownloadDisabled: Boolean(document.querySelector("#btn-run-student-review-reminder-download")?.disabled),
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(rosterOnlyReviewState.statusState === "error", `roster-only review should require action: ${rosterOnlyReviewState.statusState}`);
    assert(rosterOnlyReviewState.statusText.includes("미제출 3명"), `roster-only review status mismatch: ${rosterOnlyReviewState.statusText}`);
    assert(rosterOnlyReviewState.summaryText.includes("명단 3명") && rosterOnlyReviewState.summaryText.includes("미제출 3명"), `roster-only summary mismatch: ${rosterOnlyReviewState.summaryText}`);
    assert(rosterOnlyReviewState.tableText.includes("김가온") && rosterOnlyReviewState.tableText.includes("이하늘") && rosterOnlyReviewState.tableText.includes("박다온"), `roster-only table missing names: ${rosterOnlyReviewState.tableText}`);
    assert(rosterOnlyReviewState.batch?.row_count === 3 && rosterOnlyReviewState.batch?.accepted_count === 0 && rosterOnlyReviewState.batch?.rejected_count === 3, "roster-only batch counts mismatch");
    assert(rosterOnlyReviewState.batch?.rows?.every((row) => row.roster_missing === true), "roster-only rows should all be missing");
    assert(rosterOnlyReviewState.missingReminderDisabled === false, "roster-only review should enable missing reminder");
    assert(rosterOnlyReviewState.missingReminderDownloadDisabled === false, "roster-only review should enable missing reminder download");
    assert(rosterOnlyReviewState.reviewReminderDisabled === true, "roster-only review should not enable review reminder");
    assert(rosterOnlyReviewState.reviewReminderDownloadDisabled === true, "roster-only review should not enable review reminder download");
    await page.click("#btn-run-student-result-clear");
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_CLEAR_ACTION__?.cleared === true);

    await page.evaluate(() => {
      document.querySelector("#screen-run .main-shell-tab[data-main-tab-target='browse']")?.click();
    });
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

    await page.evaluate(() => {
      document.querySelector("#screen-run .main-shell-tab[data-main-tab-target='browse']")?.click();
    });
    await waitVisible(page, "#screen-browse");
    const multiPackageLessons = [
      {
        lesson_id: "multi_velocity",
        title: "속도 묶음 1",
        description: "첫 번째 배포 수업",
        grade: "middle",
        subject: "physics",
        required_views: ["graph", "table"],
        goals: ["속도 변화를 확인한다"],
        missions: ["첫 번째 그래프를 읽기"],
        source_text: "보임 { text: \"속도 묶음 1\" }",
      },
      {
        lesson_id: "multi_line",
        title: "함수 직선 배포",
        description: "두 번째 배포 수업",
        grade: "middle",
        subject: "math",
        required_views: ["graph", "text"],
        goals: ["직선 변화를 확인한다"],
        missions: ["두 번째 그래프를 읽기"],
        source_text: "보임 { text: \"함수 직선 배포\" }",
      },
    ];
    const multiPackageManifest = localPackageHelper.buildStudioLocalPackageManifest({
      packageId: "studio.local.multi.lesson.switch",
      title: "대표 교과 묶음 배포",
      sessionLabel: "2학년 3반",
      lessons: multiPackageLessons,
      reports: [],
    });
    const multiPackagePayload = localPackageHelper.buildStudioLocalPackagePayload({
      manifest: multiPackageManifest,
      lessons: multiPackageLessons,
      reports: [],
    });
    await page.setInputFiles("#input-local-package-file", {
      name: "multi-lesson-teacher-package.json",
      mimeType: "application/json",
      buffer: Buffer.from(JSON.stringify(multiPackagePayload), "utf-8"),
    });
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.file_name === "multi-lesson-teacher-package.json");
    await waitVisible(page, "#screen-run");
    const multiImportState = await page.evaluate(() => ({
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      titleText: document.querySelector("#run-lesson-title")?.textContent?.trim() ?? "",
      switchHidden: document.querySelector("[data-run-package-lesson-switch]")?.classList?.contains("hidden") ?? true,
      switchAriaHidden: document.querySelector("[data-run-package-lesson-switch]")?.getAttribute("aria-hidden") ?? "",
      selectDisabled: Boolean(document.querySelector("#run-package-lesson-select")?.disabled),
      optionTexts: Array.from(document.querySelectorAll("#run-package-lesson-select option")).map((node) => node.textContent?.trim() || ""),
      optionValues: Array.from(document.querySelectorAll("#run-package-lesson-select option")).map((node) => node.getAttribute("value") || ""),
      selectedText: document.querySelector("#run-package-lesson-select option:checked")?.textContent?.trim() ?? "",
    }));
    assert(multiImportState.importAction?.imported === true, "multi lesson package should import");
    assert(multiImportState.importAction?.lesson_count === 2, `multi lesson import count mismatch: ${JSON.stringify(multiImportState.importAction)}`);
    assert(
      Array.isArray(multiImportState.importAction?.imported_lesson_ids)
        && multiImportState.importAction.imported_lesson_ids.length === 2,
      `multi lesson imported ids mismatch: ${JSON.stringify(multiImportState.importAction?.imported_lesson_ids)}`,
    );
    assert(multiImportState.switchHidden === false, "multi lesson package switch should be visible");
    assert(multiImportState.switchAriaHidden === "false", `multi lesson switch aria mismatch: ${multiImportState.switchAriaHidden}`);
    assert(multiImportState.selectDisabled === false, "multi lesson package select should be enabled");
    assert(multiImportState.optionTexts.length === 2, `multi lesson package option count mismatch: ${multiImportState.optionTexts.join("|")}`);
    assert(
      multiImportState.optionTexts[0].includes("1. 속도 묶음 1")
        && multiImportState.optionTexts[1].includes("2. 함수 직선 배포"),
      `multi lesson package options mismatch: ${multiImportState.optionTexts.join("|")}`,
    );
    assert(multiImportState.selectedText.includes("속도 묶음 1"), `multi lesson initial selection mismatch: ${multiImportState.selectedText}`);
    await page.click("#btn-run");
    await page.waitForFunction(() => {
      const text = document.querySelector("#run-mirror-hash")?.textContent?.trim() ?? "";
      return text && text !== "상태 기록: -";
    });
    await page.fill("#run-delivery-student-name", "이다온");
    await page.waitForFunction(() => !document.querySelector("#btn-run-delivery-result-copy")?.disabled);
    await page.selectOption("#run-package-lesson-select", multiImportState.optionValues[1]);
    await page.waitForFunction(() => document.querySelector("#run-lesson-title")?.textContent?.includes("함수 직선 배포"));
    const multiSwitchAfterRunResetState = await page.evaluate(() => ({
      mirrorHashText: document.querySelector("#run-mirror-hash")?.textContent?.trim() ?? "",
      deliveryResultReady: document.querySelector("[data-run-delivery-result]")?.dataset?.ready ?? "",
      deliveryResultCopyDisabled: Boolean(document.querySelector("#btn-run-delivery-result-copy")?.disabled),
      deliveryResultDownloadDisabled: Boolean(document.querySelector("#btn-run-delivery-result-download")?.disabled),
      deliveryStatusText: document.querySelector("[data-run-delivery-status]")?.textContent?.trim() ?? "",
      selectedText: document.querySelector("#run-package-lesson-select option:checked")?.textContent?.trim() ?? "",
    }));
    assert(multiSwitchAfterRunResetState.selectedText.includes("함수 직선 배포"), `multi lesson switch after run selection mismatch: ${multiSwitchAfterRunResetState.selectedText}`);
    assert(multiSwitchAfterRunResetState.mirrorHashText === "상태 기록: -", `multi lesson switch after run should clear stale hash: ${multiSwitchAfterRunResetState.mirrorHashText}`);
    assert(multiSwitchAfterRunResetState.deliveryResultReady === "0", `multi lesson switch after run should hide stale result panel: ${multiSwitchAfterRunResetState.deliveryResultReady}`);
    assert(
      multiSwitchAfterRunResetState.deliveryResultCopyDisabled === true
        && multiSwitchAfterRunResetState.deliveryResultDownloadDisabled === true,
      `multi lesson switch after run should disable stale result actions: ${JSON.stringify(multiSwitchAfterRunResetState)}`,
    );
    assert(multiSwitchAfterRunResetState.deliveryStatusText.includes("받은 배포 파일 준비됨"), `multi lesson switch after run status mismatch: ${multiSwitchAfterRunResetState.deliveryStatusText}`);
    await page.selectOption("#run-package-lesson-select", multiImportState.optionValues[1]);
    await page.waitForFunction(() => document.querySelector("#run-lesson-title")?.textContent?.includes("함수 직선 배포"));
    const multiSwitchState = await page.evaluate(() => ({
      titleText: document.querySelector("#run-lesson-title")?.textContent?.trim() ?? "",
      briefText: document.querySelector("[data-run-lesson-brief]")?.textContent?.trim() ?? "",
      selectedText: document.querySelector("#run-package-lesson-select option:checked")?.textContent?.trim() ?? "",
      switchHidden: document.querySelector("[data-run-package-lesson-switch]")?.classList?.contains("hidden") ?? true,
      deliveryInstructionsText: document.querySelector("[data-run-delivery-instructions]")?.textContent?.trim() ?? "",
      inputRegistry: JSON.parse(localStorage.getItem("seamgrim.input_registry.v0") || "{}"),
    }));
    assert(multiSwitchState.titleText.includes("함수 직선 배포"), `multi lesson switched title mismatch: ${multiSwitchState.titleText}`);
    assert(multiSwitchState.briefText.includes("교사가 보낸 배포 파일"), `multi lesson switched brief mismatch: ${multiSwitchState.briefText}`);
    assert(multiSwitchState.selectedText.includes("함수 직선 배포"), `multi lesson switched select mismatch: ${multiSwitchState.selectedText}`);
    assert(multiSwitchState.switchHidden === false, "multi lesson switch should remain visible after switching lessons");
    assert(
      multiSwitchState.deliveryInstructionsText.includes("배포 수업 선택에서 오늘 수업"),
      `multi lesson switched delivery instructions missing lesson switch hint: ${multiSwitchState.deliveryInstructionsText}`,
    );
    assert(
      multiSwitchState.deliveryInstructionsText.includes("교과 수: 2")
        && multiSwitchState.deliveryInstructionsText.includes("첫 수업: 속도 묶음 1")
        && multiSwitchState.deliveryInstructionsText.includes("첫 수업 코드: multi_velocity"),
      `multi lesson switched delivery instructions missing multi lesson context: ${multiSwitchState.deliveryInstructionsText}`,
    );
    assert(
      multiSwitchState.deliveryInstructionsText.includes("오늘 수업: 함수 직선 배포")
        && multiSwitchState.deliveryInstructionsText.includes("오늘 수업 코드: multi_line"),
      `multi lesson switched delivery instructions should name selected lesson: ${multiSwitchState.deliveryInstructionsText}`,
    );
    assert(
      multiSwitchState.inputRegistry?.inputs?.selected_id === "local_package:studio.local.multi.lesson.switch:multi_line",
      `multi lesson switched source mismatch: ${multiSwitchState.inputRegistry?.inputs?.selected_id}`,
    );
    await page.evaluate(() => {
      delete window.__SEAMGRIM_RUN_STATE_HASH_COPY_ACTION__;
      window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ = "";
    });
    await page.click("#btn-run");
    await page.waitForFunction(() => {
      const text = document.querySelector("#run-mirror-hash")?.textContent?.trim() ?? "";
      return text && text !== "상태 기록: -";
    });
    await page.fill("#run-delivery-student-name", "이다온");
    await page.waitForFunction(() => !document.querySelector("#btn-run-delivery-result-copy")?.disabled);
    await page.click("#btn-run-delivery-result-copy");
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_STATE_HASH_COPY_ACTION__?.copied === true);
    const multiLessonStudentResultState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_RUN_STATE_HASH_COPY_ACTION__ ?? null,
    }));
    assert(multiLessonStudentResultState.action?.schema === "seamgrim.run_state_hash_copy_action.v1", "multi lesson student result copy schema mismatch");
    assert(multiLessonStudentResultState.action?.student_delivery === true, "multi lesson student result should be delivery result");
    assert(multiLessonStudentResultState.action?.submission_method === "clipboard", `multi lesson student result submission method mismatch: ${multiLessonStudentResultState.action?.submission_method}`);
    assert(multiLessonStudentResultState.action?.lesson_id === "multi_line", `multi lesson result lesson id mismatch: ${multiLessonStudentResultState.action?.lesson_id}`);
    assert(multiLessonStudentResultState.action?.package_id === "studio.local.multi.lesson.switch", `multi lesson result package id mismatch: ${multiLessonStudentResultState.action?.package_id}`);
    assert(multiLessonStudentResultState.action?.session_label === "2학년 3반", `multi lesson result session mismatch: ${multiLessonStudentResultState.action?.session_label}`);
    assert(multiLessonStudentResultState.action?.student_name === "이다온", `multi lesson result student name mismatch: ${multiLessonStudentResultState.action?.student_name}`);
    assert(
      multiLessonStudentResultState.copiedText.includes("학생: 이다온")
        && multiLessonStudentResultState.copiedText.includes("차시: 2학년 3반")
        && multiLessonStudentResultState.copiedText.includes("수업: 함수 직선 배포")
        && multiLessonStudentResultState.copiedText.includes("수업 코드: multi_line")
        && multiLessonStudentResultState.copiedText.includes("배포 묶음: 대표 교과 묶음 배포")
        && multiLessonStudentResultState.copiedText.includes("배포 코드: studio.local.multi.lesson.switch")
        && multiLessonStudentResultState.copiedText.includes("상태 기록:"),
      `multi lesson student result text mismatch: ${multiLessonStudentResultState.copiedText}`,
    );
    assert(
      !multiLessonStudentResultState.copiedText.includes("수업 코드: multi_velocity"),
      `multi lesson student result kept stale first lesson id: ${multiLessonStudentResultState.copiedText}`,
    );
    const [multiLessonStudentResultDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.click("#btn-run-delivery-result-download"),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_RUN_STATE_HASH_DOWNLOAD_ACTION__?.downloaded === true);
    const multiLessonStudentResultDownloadPath = await multiLessonStudentResultDownload.path();
    const multiLessonStudentResultDownloadedText = multiLessonStudentResultDownloadPath ? await fs.readFile(multiLessonStudentResultDownloadPath, "utf-8") : "";
    const multiLessonStudentResultDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_RUN_STATE_HASH_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(multiLessonStudentResultDownloadState.action?.schema === "seamgrim.run_state_hash_download_action.v1", "multi lesson student result download schema mismatch");
    assert(multiLessonStudentResultDownloadState.action?.student_delivery === true, "multi lesson student result download should be delivery result");
    assert(multiLessonStudentResultDownloadState.action?.submission_method === "file", `multi lesson student result download method mismatch: ${multiLessonStudentResultDownloadState.action?.submission_method}`);
    assert(multiLessonStudentResultDownloadState.action?.lesson_id === "multi_line", `multi lesson result download lesson id mismatch: ${multiLessonStudentResultDownloadState.action?.lesson_id}`);
    assert(multiLessonStudentResultDownloadState.action?.package_id === "studio.local.multi.lesson.switch", `multi lesson result download package id mismatch: ${multiLessonStudentResultDownloadState.action?.package_id}`);
    assert(
      String(multiLessonStudentResultDownloadState.action?.file_name ?? "").includes("multi_line")
        && !String(multiLessonStudentResultDownloadState.action?.file_name ?? "").includes("multi_velocity"),
      `multi lesson student result download filename should use selected lesson: ${multiLessonStudentResultDownloadState.action?.file_name}`,
    );
    assert(multiLessonStudentResultDownload.suggestedFilename() === multiLessonStudentResultDownloadState.action.file_name, "multi lesson student result download suggested filename mismatch");
    assert(multiLessonStudentResultDownloadedText.trim() === multiLessonStudentResultState.copiedText.trim(), "multi lesson student result downloaded text mismatch");
    const multiPackageMixedLessonText = [
      multiLessonStudentResultState.copiedText
        .replace("수업: 함수 직선 배포", "수업: 속도 묶음 1")
        .replace("수업 코드: multi_line", "수업 코드: multi_velocity"),
      multiLessonStudentResultState.copiedText,
    ].join("\n\n");
    await page.evaluate((resultText) => {
      const roster = document.querySelector("#run-student-roster-input");
      const input = document.querySelector("#run-student-result-input");
      if (roster) {
        roster.value = "";
        roster.dispatchEvent(new Event("input", { bubbles: true }));
      }
      if (input) {
        input.value = resultText;
        input.dispatchEvent(new Event("input", { bubbles: true }));
      }
      document.querySelector("#btn-run-student-result-review")?.click();
    }, multiPackageMixedLessonText);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.row_count === 2);
    const multiPackageMixedReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      tableRows: Array.from(document.querySelectorAll("[data-run-student-result-table] tbody tr")).map((row) => ({
        state: row.getAttribute("data-student-result-row") || "",
        text: row.textContent?.trim() || "",
      })),
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(multiPackageMixedReviewState.statusText.includes("확인됨 2/2"), `multi package mixed lesson review status mismatch: ${multiPackageMixedReviewState.statusText}`);
    assert(multiPackageMixedReviewState.batch?.accepted === true, `multi package mixed lesson batch should pass: ${JSON.stringify(multiPackageMixedReviewState.batch)}`);
    assert(multiPackageMixedReviewState.batch?.accepted_count === 2 && multiPackageMixedReviewState.batch?.rejected_count === 0, `multi package mixed lesson counts mismatch: ${JSON.stringify(multiPackageMixedReviewState.batch)}`);
    assert(
      String(multiPackageMixedReviewState.batch?.report_text ?? "").includes("수업 범위\t여러 수업 2개")
        && String(multiPackageMixedReviewState.batch?.report_text ?? "").includes("수업 코드 목록\tmulti_velocity|multi_line")
        && String(multiPackageMixedReviewState.batch?.report_text ?? "").includes("수업 제목 목록\t속도 묶음 1|함수 직선 배포"),
      `multi package mixed lesson report should summarize all lessons: ${multiPackageMixedReviewState.batch?.report_text}`,
    );
    assert(
      multiPackageMixedReviewState.batch?.rows?.every((row) => row.student_name === "이다온" && row.duplicate_student_name === false && row.duplicate_state_hash === false),
      `same student/hash across different package lessons should not be a duplicate: ${JSON.stringify(multiPackageMixedReviewState.batch?.rows)}`,
    );
    assert(
      multiPackageMixedReviewState.tableRows.some((row) => row.state === "ok" && row.text.includes("multi_velocity"))
        && multiPackageMixedReviewState.tableRows.some((row) => row.state === "ok" && row.text.includes("multi_line")),
      `multi package mixed lesson table mismatch: ${JSON.stringify(multiPackageMixedReviewState.tableRows)}`,
    );
    await page.evaluate(() => {
      document.querySelector("#btn-run-student-result-report-copy")?.click();
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__?.multi_lesson === true);
    const multiPackageMixedReportCopyState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_COPY_ACTION__ ?? null,
    }));
    assert(multiPackageMixedReportCopyState.action?.lesson_id === "", `multi lesson report copy should not pin first lesson id: ${JSON.stringify(multiPackageMixedReportCopyState.action)}`);
    assert(
      Array.isArray(multiPackageMixedReportCopyState.action?.lesson_ids)
        && multiPackageMixedReportCopyState.action.lesson_ids.join("|") === "multi_velocity|multi_line",
      `multi lesson report copy lesson ids mismatch: ${JSON.stringify(multiPackageMixedReportCopyState.action)}`,
    );
    assert(
      String(multiPackageMixedReportCopyState.action?.next_action ?? "").includes("결과표 저장"),
      `multi lesson report copy should preserve next action: ${JSON.stringify(multiPackageMixedReportCopyState.action)}`,
    );
    assert(multiPackageMixedReportCopyState.copiedText.includes("수업 코드 목록\tmulti_velocity|multi_line"), `multi lesson copied report missing lesson list: ${multiPackageMixedReportCopyState.copiedText}`);
    assert(multiPackageMixedReportCopyState.copiedText.includes("다음 행동\t결과표 저장 후 입력 비우기"), `multi lesson copied report missing next action: ${multiPackageMixedReportCopyState.copiedText}`);
    const [multiPackageMixedReportDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.evaluate(() => {
        document.querySelector("#btn-run-student-result-report-download")?.click();
      }),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_DOWNLOAD_ACTION__?.multi_lesson === true);
    const multiPackageMixedReportDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_RESULT_RETURN_REPORT_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(
      String(multiPackageMixedReportDownloadState.action?.file_name ?? "").includes("studio.local.multi.lesson.switch")
        && !String(multiPackageMixedReportDownloadState.action?.file_name ?? "").includes("multi_velocity"),
      `multi lesson report filename should use package scope: ${multiPackageMixedReportDownloadState.action?.file_name}`,
    );
    assert(
      String(multiPackageMixedReportDownloadState.action?.next_action ?? "").includes("결과표 저장"),
      `multi lesson report download should preserve next action: ${JSON.stringify(multiPackageMixedReportDownloadState.action)}`,
    );
    assert(multiPackageMixedReportDownload.suggestedFilename() === multiPackageMixedReportDownloadState.action.file_name, "multi lesson report suggested filename mismatch");
    const multiPackageMixedReportDownloadPath = await multiPackageMixedReportDownload.path();
    const multiPackageMixedReportDownloadedText = multiPackageMixedReportDownloadPath ? await fs.readFile(multiPackageMixedReportDownloadPath, "utf-8") : "";
    await page.evaluate(() => {
      const roster = document.querySelector("#run-student-roster-input");
      const input = document.querySelector("#run-student-result-input");
      if (roster) {
        roster.value = "";
        roster.dispatchEvent(new Event("input", { bubbles: true }));
      }
      if (input) {
        input.value = "";
        input.dispatchEvent(new Event("input", { bubbles: true }));
      }
    });
    await page.setInputFiles("#input-run-student-result-file", {
      name: multiPackageMixedReportDownloadState.action.file_name,
      mimeType: "text/tab-separated-values",
      buffer: Buffer.from(multiPackageMixedReportDownloadedText, "utf-8"),
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__?.multi_lesson === true);
    const multiPackageMixedReportReloadState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      action: window.__SEAMGRIM_STUDENT_RESULT_FILE_LOAD_ACTION__ ?? null,
    }));
    assert(
      Array.isArray(multiPackageMixedReportReloadState.action?.lesson_ids)
        && multiPackageMixedReportReloadState.action.lesson_ids.join("|") === "multi_velocity|multi_line",
      `multi lesson report reload should preserve lesson ids: ${JSON.stringify(multiPackageMixedReportReloadState.action)}`,
    );
    assert(multiPackageMixedReportReloadState.action?.package_id === "studio.local.multi.lesson.switch", `multi lesson report reload package mismatch: ${JSON.stringify(multiPackageMixedReportReloadState.action)}`);
    assert(multiPackageMixedReportReloadState.statusText.includes("학생 결과 2건"), `multi lesson report reload status mismatch: ${multiPackageMixedReportReloadState.statusText}`);
    await page.evaluate(() => {
      document.querySelector("#btn-run-student-result-review")?.click();
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.row_count === 2);
    const multiPackageMixedReportReloadReviewState = await page.evaluate(() => ({
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(
      String(multiPackageMixedReportReloadReviewState.batch?.report_text ?? "").includes("수업 코드 목록\tmulti_velocity|multi_line"),
      `multi lesson report reload review should keep lesson list: ${multiPackageMixedReportReloadReviewState.batch?.report_text}`,
    );
    const multiPackageCourseRosterText = [
      "# 셈그림 대표 교과 학생 명단",
      "학생\t차시\t수업\t수업 코드\t배포 묶음\t배포 코드",
      "이다온\t2학년 3반\t속도 묶음 1\tmulti_velocity\t대표 교과 묶음 배포\tstudio.local.multi.lesson.switch",
      "이다온\t2학년 3반\t함수 직선 배포\tmulti_line\t대표 교과 묶음 배포\tstudio.local.multi.lesson.switch",
    ].join("\n");
    await page.evaluate((rosterText) => {
      const roster = document.querySelector("#run-student-roster-input");
      const input = document.querySelector("#run-student-result-input");
      if (roster) {
        roster.value = rosterText;
        roster.dispatchEvent(new Event("input", { bubbles: true }));
      }
      if (input) {
        input.value = "";
        input.dispatchEvent(new Event("input", { bubbles: true }));
      }
    }, multiPackageCourseRosterText);
    const multiPackageCourseRosterPreviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
    }));
    assert(multiPackageCourseRosterPreviewState.statusText.includes("제출 대상 2건"), `multi package course roster preview should count lesson rows: ${multiPackageCourseRosterPreviewState.statusText}`);
    await page.evaluate(({ rosterText, resultText }) => {
      const roster = document.querySelector("#run-student-roster-input");
      const input = document.querySelector("#run-student-result-input");
      if (roster) {
        roster.value = rosterText;
        roster.dispatchEvent(new Event("input", { bubbles: true }));
      }
      if (input) {
        input.value = resultText;
        input.dispatchEvent(new Event("input", { bubbles: true }));
      }
      document.querySelector("#btn-run-student-result-review")?.click();
    }, { rosterText: multiPackageCourseRosterText, resultText: multiPackageMixedLessonText });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.roster_entry_count === 2);
    const multiPackageRosterReviewState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(multiPackageRosterReviewState.batch?.accepted === true, `multi package course roster should pass: ${JSON.stringify(multiPackageRosterReviewState.batch)}`);
    assert(multiPackageRosterReviewState.batch?.roster_entry_count === 2, `multi package course roster entry count mismatch: ${JSON.stringify(multiPackageRosterReviewState.batch)}`);
    assert(multiPackageRosterReviewState.batch?.duplicate_roster_count === 0, `same student across roster lesson rows should not be duplicate: ${JSON.stringify(multiPackageRosterReviewState.batch)}`);
    assert(multiPackageRosterReviewState.batch?.missing_count === 0 && multiPackageRosterReviewState.batch?.extra_count === 0, `multi package course roster should match results: ${JSON.stringify(multiPackageRosterReviewState.batch)}`);
    assert(multiPackageRosterReviewState.batch?.roster_count_kind === "lesson_targets", `multi package course roster kind mismatch: ${JSON.stringify(multiPackageRosterReviewState.batch)}`);
    assert(multiPackageRosterReviewState.statusText.includes("확인됨 2/2"), `multi package course roster status mismatch: ${multiPackageRosterReviewState.statusText}`);
    assert(multiPackageRosterReviewState.statusText.includes("제출 대상 2건"), `multi package course roster status should use target label: ${multiPackageRosterReviewState.statusText}`);
    assert(!multiPackageRosterReviewState.summaryText.includes("명단 중복"), `multi package course roster should not show duplicate summary: ${multiPackageRosterReviewState.summaryText}`);

    const multiPackageOneLessonResultText = multiPackageMixedLessonText.split(/\n\n/u)[0] ?? "";
    await page.evaluate(({ rosterText, resultText }) => {
      const roster = document.querySelector("#run-student-roster-input");
      const input = document.querySelector("#run-student-result-input");
      if (roster) {
        roster.value = rosterText;
        roster.dispatchEvent(new Event("input", { bubbles: true }));
      }
      if (input) {
        input.value = resultText;
        input.dispatchEvent(new Event("input", { bubbles: true }));
      }
      document.querySelector("#btn-run-student-result-review")?.click();
    }, { rosterText: multiPackageCourseRosterText, resultText: multiPackageOneLessonResultText });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.missing_count === 1);
    const multiPackageCourseRosterMissingState = await page.evaluate(() => ({
      statusText: document.querySelector("[data-run-student-result-status]")?.textContent?.trim() ?? "",
      summaryText: document.querySelector("[data-run-student-result-summary]")?.textContent?.trim() ?? "",
      batch: window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__ ?? null,
    }));
    assert(multiPackageCourseRosterMissingState.batch?.accepted === false, `multi package course roster missing should fail: ${JSON.stringify(multiPackageCourseRosterMissingState.batch)}`);
    assert(multiPackageCourseRosterMissingState.batch?.roster_count === 2, `multi package course roster missing roster count mismatch: ${JSON.stringify(multiPackageCourseRosterMissingState.batch)}`);
    assert(multiPackageCourseRosterMissingState.batch?.roster_count_kind === "lesson_targets", `multi package course roster missing kind mismatch: ${JSON.stringify(multiPackageCourseRosterMissingState.batch)}`);
    assert(multiPackageCourseRosterMissingState.batch?.roster_entry_count === 2, `multi package course roster missing entry count mismatch: ${JSON.stringify(multiPackageCourseRosterMissingState.batch)}`);
    assert(multiPackageCourseRosterMissingState.batch?.missing_count === 1, `multi package course roster missing count mismatch: ${JSON.stringify(multiPackageCourseRosterMissingState.batch)}`);
    assert(
      multiPackageCourseRosterMissingState.batch?.rows?.some((row) => row.student_name === "이다온" && row.lesson_id === "multi_line" && row.roster_missing === true),
      `multi package course roster should mark only missing lesson row: ${JSON.stringify(multiPackageCourseRosterMissingState.batch?.rows)}`,
    );
    assert(multiPackageCourseRosterMissingState.summaryText.includes("미제출 1명"), `multi package course roster missing summary mismatch: ${multiPackageCourseRosterMissingState.summaryText}`);
    await page.evaluate(() => {
      document.querySelector("#btn-run-student-missing-reminder-copy")?.click();
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_MISSING_REMINDER_COPY_ACTION__?.text?.includes("multi_line"));
    const multiPackageMissingReminderState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_MISSING_REMINDER_COPY_ACTION__ ?? null,
    }));
    assert(multiPackageMissingReminderState.action?.missing_count === 1, `multi package missing reminder count mismatch: ${JSON.stringify(multiPackageMissingReminderState.action)}`);
    assert(String(multiPackageMissingReminderState.action?.next_action ?? "").includes("미제출 안내"), `multi package missing reminder should preserve next action: ${JSON.stringify(multiPackageMissingReminderState.action)}`);
    assert(
      Array.isArray(multiPackageMissingReminderState.action?.missing_targets)
        && multiPackageMissingReminderState.action.missing_targets.some((target) => target.student_name === "이다온" && target.lesson_id === "multi_line"),
      `multi package missing reminder target mismatch: ${JSON.stringify(multiPackageMissingReminderState.action)}`,
    );
    assert(multiPackageMissingReminderState.action?.lesson_id === "multi_line", `single missing reminder should use missing lesson id: ${JSON.stringify(multiPackageMissingReminderState.action)}`);
    assert(multiPackageMissingReminderState.copiedText.includes("대상 수업:"), `multi package missing reminder missing target heading: ${multiPackageMissingReminderState.copiedText}`);
    assert(multiPackageMissingReminderState.copiedText.includes("함수 직선 배포") && multiPackageMissingReminderState.copiedText.includes("multi_line"), `multi package missing reminder should name the missing lesson: ${multiPackageMissingReminderState.copiedText}`);
    await page.evaluate((rosterText) => {
      const roster = document.querySelector("#run-student-roster-input");
      const input = document.querySelector("#run-student-result-input");
      if (roster) {
        roster.value = rosterText;
        roster.dispatchEvent(new Event("input", { bubbles: true }));
      }
      if (input) {
        input.value = "";
        input.dispatchEvent(new Event("input", { bubbles: true }));
      }
      document.querySelector("#btn-run-student-result-review")?.click();
    }, multiPackageCourseRosterText);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.missing_count === 2);
    const [multiPackageMissingReminderDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.evaluate(() => {
        document.querySelector("#btn-run-student-missing-reminder-download")?.click();
      }),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_MISSING_REMINDER_DOWNLOAD_ACTION__?.multi_lesson === true);
    const multiPackageMissingReminderDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_MISSING_REMINDER_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(
      Array.isArray(multiPackageMissingReminderDownloadState.action?.lesson_ids)
        && multiPackageMissingReminderDownloadState.action.lesson_ids.join("|") === "multi_velocity|multi_line",
      `multi lesson missing reminder lesson ids mismatch: ${JSON.stringify(multiPackageMissingReminderDownloadState.action)}`,
    );
    assert(multiPackageMissingReminderDownloadState.action?.lesson_id === "", `multi lesson missing reminder should not pin first lesson: ${JSON.stringify(multiPackageMissingReminderDownloadState.action)}`);
    assert(
      String(multiPackageMissingReminderDownloadState.action?.file_name ?? "").includes("studio.local.multi.lesson.switch")
        && !String(multiPackageMissingReminderDownloadState.action?.file_name ?? "").includes("multi_velocity"),
      `multi lesson missing reminder filename should use package scope: ${multiPackageMissingReminderDownloadState.action?.file_name}`,
    );
    assert(multiPackageMissingReminderDownload.suggestedFilename() === multiPackageMissingReminderDownloadState.action.file_name, "multi lesson missing reminder suggested filename mismatch");

    const multiPackageReviewNeededText = multiPackageMixedLessonText.replace(/상태 기록: .+/gu, "상태 기록: -");
    await page.evaluate((resultText) => {
      const roster = document.querySelector("#run-student-roster-input");
      const input = document.querySelector("#run-student-result-input");
      if (roster) {
        roster.value = "";
        roster.dispatchEvent(new Event("input", { bubbles: true }));
      }
      if (input) {
        input.value = resultText;
        input.dispatchEvent(new Event("input", { bubbles: true }));
      }
      document.querySelector("#btn-run-student-result-review")?.click();
    }, multiPackageReviewNeededText);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_RESULT_RETURN_BATCH_REVIEW__?.rejected_count === 2);
    await page.evaluate(() => {
      document.querySelector("#btn-run-student-review-reminder-copy")?.click();
    });
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_COPY_ACTION__?.multi_lesson === true);
    const multiPackageReviewReminderState = await page.evaluate(() => ({
      copiedText: window.__STUDIO_LOCAL_PACKAGE_COPIED_TEXT__ ?? "",
      action: window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_COPY_ACTION__ ?? null,
    }));
    assert(
      Array.isArray(multiPackageReviewReminderState.action?.review_targets)
        && multiPackageReviewReminderState.action.review_targets.some((target) => target.lesson_id === "multi_velocity")
        && multiPackageReviewReminderState.action.review_targets.some((target) => target.lesson_id === "multi_line"),
      `multi lesson review reminder targets mismatch: ${JSON.stringify(multiPackageReviewReminderState.action)}`,
    );
    assert(String(multiPackageReviewReminderState.action?.next_action ?? "").includes("확인 필요 안내"), `multi lesson review reminder should preserve next action: ${JSON.stringify(multiPackageReviewReminderState.action)}`);
    assert(multiPackageReviewReminderState.action?.lesson_id === "", `multi lesson review reminder should not pin first lesson: ${JSON.stringify(multiPackageReviewReminderState.action)}`);
    assert(multiPackageReviewReminderState.copiedText.includes("multi_velocity") && multiPackageReviewReminderState.copiedText.includes("multi_line"), `multi lesson review reminder text should list lesson targets: ${multiPackageReviewReminderState.copiedText}`);
    const [multiPackageReviewReminderDownload] = await Promise.all([
      page.waitForEvent("download"),
      page.evaluate(() => {
        document.querySelector("#btn-run-student-review-reminder-download")?.click();
      }),
    ]);
    await page.waitForFunction(() => window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_DOWNLOAD_ACTION__?.multi_lesson === true);
    const multiPackageReviewReminderDownloadState = await page.evaluate(() => ({
      action: window.__SEAMGRIM_STUDENT_REVIEW_REMINDER_DOWNLOAD_ACTION__ ?? null,
    }));
    assert(
      String(multiPackageReviewReminderDownloadState.action?.file_name ?? "").includes("studio.local.multi.lesson.switch")
        && !String(multiPackageReviewReminderDownloadState.action?.file_name ?? "").includes("multi_velocity"),
      `multi lesson review reminder filename should use package scope: ${multiPackageReviewReminderDownloadState.action?.file_name}`,
    );
    assert(multiPackageReviewReminderDownload.suggestedFilename() === multiPackageReviewReminderDownloadState.action.file_name, "multi lesson review reminder suggested filename mismatch");

    await page.evaluate(() => {
      document.querySelector("#screen-run .main-shell-tab[data-main-tab-target='browse']")?.click();
    });
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
