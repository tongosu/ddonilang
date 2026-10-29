#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_local_share_and_packaging_browser: ok";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) {
    throw new Error(`missing file: ${file}`);
  }
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
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
      const rel = rawPath.replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, rel);
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, {
        "content-type": mimeType(file),
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
      if (UNSAFE_BROWSER_PORTS.has(address.port)) {
        server.close(() => {
          createServer(root).then(resolve, reject);
        });
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
    return url.pathname === "/api/lessons/inventory" || url.pathname === "/api/lesson-inventory";
  } catch (_) {
    return false;
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "studio_local_share_package.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1180, height: 760 },
      locale: "ko-KR",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        if (String(msg.text() ?? "").includes("Failed to load resource")) return;
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
    const result = await page.evaluate(async () => {
      const {
        buildStudioLocalPackageManifest,
        buildStudioLocalPackagePayload,
        importStudioLocalPackagePayload,
        validateStudioStaticBundle,
        formatStudioLocalPackageIndexText,
      } = await import("./studio_local_share_package.js");
      const lessons = [
        {
          lesson_id: "lesson.voltage",
          title: "전압 기초",
          source_text: '"전압" 보여주기.',
        },
        {
          lesson_id: "lesson.flow",
          title: "흐름 점검",
          source_text: '"흐름" 보여주기.',
        },
      ];
      const reports = [
        {
          report_id: "report.classroom",
          title: "교실 리포트",
          text: "과제\t판정\n전압 기초\t통과",
        },
      ];
      const manifest = buildStudioLocalPackageManifest({
        packageId: "studio.local.demo",
        title: "로컬 스튜디오 꾸러미",
        version: "1.0.0",
        lessons,
        reports,
        staticFiles: [
          { path: "index.html", mime: "text/html; charset=utf-8", byte_size: 100 },
          { path: "app.js", mime: "application/javascript; charset=utf-8", byte_size: 200 },
          { path: "styles.css", mime: "text/css; charset=utf-8", byte_size: 300 },
        ],
      });
      const payload = buildStudioLocalPackagePayload({ manifest, lessons, reports });
      const imported = importStudioLocalPackagePayload(payload);
      const bundleOk = validateStudioStaticBundle({
        manifest,
        availableFiles: ["index.html", "app.js", "styles.css", "lessons/lesson.voltage.ddn"],
      });
      const bundleFail = validateStudioStaticBundle({
        manifest,
        availableFiles: ["index.html", "app.js"],
      });
      return {
        manifest,
        payload,
        imported,
        bundleOk,
        bundleFail,
        text: formatStudioLocalPackageIndexText(payload),
      };
    });

    assert(result.manifest.__종류 === "studio_local_package_manifest", "manifest kind mismatch");
    assert(result.manifest.package_id === "studio.local.demo", "package id mismatch");
    assert(result.manifest.entry === "index.html", "entry mismatch");
    assert(result.manifest.lesson_count === 2, "lesson count mismatch");
    assert(result.manifest.report_count === 1, "report count mismatch");
    assert(result.manifest.required_static_files.join("|") === "index.html|app.js|styles.css", "required static files mismatch");
    assert(result.manifest.account_required === false && result.manifest.cloud_sync === false && result.manifest.public_registry === false, "manifest must be local only");
    assert(result.payload.__종류 === "studio_local_package_payload", "payload kind mismatch");
    assert(result.payload.import_export_format === "studio_local_package_payload_v1", "payload format mismatch");
    assert(result.payload.lessons[0].path === "lessons/lesson.voltage.ddn", "lesson path mismatch");
    assert(result.payload.reports[0].path === "reports/report.classroom.txt", "report path mismatch");
    assert(result.imported.__종류 === "studio_local_package_import_result", "import result kind mismatch");
    assert(result.imported.lesson_count === 2 && result.imported.report_count === 1, "import counts mismatch");
    assert(result.imported.lessons[1].title === "흐름 점검", "import order mismatch");
    assert(result.bundleOk.__종류 === "studio_local_static_bundle_check", "bundle check kind mismatch");
    assert(result.bundleOk.check_result === "통과", "bundle ok should pass");
    assert(result.bundleFail.check_result === "실패", "bundle fail should fail");
    assert(result.bundleFail.missing_files.includes("styles.css"), "missing styles.css should be reported");
    assert(result.text.startsWith("구분\t경로\t제목\t크기\n"), "index text header mismatch");
    assert(result.text.includes("package\tstudio.local.demo\t로컬 스튜디오 꾸러미"), "package text row missing");
    assert(result.text.includes("lesson\tlessons/lesson.voltage.ddn\t전압 기초"), "lesson text row missing");
    assert(result.text.includes("report\treports/report.classroom.txt\t교실 리포트"), "report text row missing");
    assert(!result.text.endsWith("\n"), "index text must not have trailing newline");

    if (failures.length > 0) {
      throw new Error(failures.join("\n"));
    }
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
