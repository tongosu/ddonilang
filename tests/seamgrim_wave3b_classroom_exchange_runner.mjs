#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_classroom_exchange: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LESSON_ID = "elem_math_number_line";

function assert(condition, message) {
  if (!condition) throw new Error(message);
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
      const rawPath = decodeURIComponent(url.pathname === "/" ? PROJECT_URL : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
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
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/") || pathname.startsWith("/lessons_rewrite_v1/"))
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|detjson|md)$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function installFailureCapture(page, failures) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    if (String(msg.text() ?? "").includes("Failed to load resource")) return;
    failures.push(`console error: ${msg.text()}`);
  });
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`response ${res.status()}: ${res.url()}`);
  });
}

async function waitVisible(page, selector) {
  await page.waitForFunction((sel) => {
    const node = document.querySelector(sel);
    return node && !String(node.className ?? "").split(/\s+/).includes("hidden");
  }, selector, { timeout: 30_000 });
}

async function openPublishWithCurrentLesson(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&catalog_mode=full&lesson=${encodeURIComponent(LESSON_ID)}`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, {
    timeout: 30_000,
  });
  const publishTab = page.locator('#screen-run:not(.hidden) .main-shell-tab[data-main-tab-target="publish"][data-shell-space="publish"]');
  await publishTab.waitFor({ state: "visible", timeout: 20_000 });
  await publishTab.click();
  await waitVisible(page, "#screen-publish");
}

async function exportCurrentLessonBundle(page) {
  await page.locator("#deployment-profile-control-name").fill("number_line_value");
  await page.locator("#deployment-profile-control-name").dispatchEvent("change");
  await page.locator("#btn-copy-current-lesson-bundle").click();
  await page.waitForFunction(() => window.__STUDIO_CLASSROOM_EXCHANGE_EXPORT_ACTION__, null, {
    timeout: 30_000,
  });
  const model = await page.evaluate(() => window.__STUDIO_CLASSROOM_EXCHANGE_EXPORT_ACTION__);
  assert(model?.payload?.classroom_exchange?.payload_hash, `current lesson bundle export did not build payload: ${model?.error ?? "missing action"}`);
  assert(model?.copied === true, "current lesson bundle copy failed");
  assert(model?.kind === "lesson_bundle", "export kind is not lesson_bundle");
  assert(model?.payload?.classroom_exchange?.schema === "seamgrim.classroom_exchange.v0", "missing classroom_exchange.v0 schema");
  assert(model?.payload?.classroom_exchange?.deployment_profile_hash, "missing deployment profile hash in exchange");
  assert(model?.payload?.classroom_exchange?.payload_hash === model?.classroom_exchange_hash, "exchange hash mismatch in export model");
  assert(model?.payload?.manifest?.deployment_profile_hash === model?.deployment_profile_hash, "manifest deployment hash mismatch");
  assert(model?.payload?.package_hash === model?.package_hash, "payload package hash mismatch");
  assert(Array.isArray(model?.payload?.lessons) && model.payload.lessons.length === 1, "current lesson bundle must export one lesson");
  assert(String(model.payload.lessons[0]?.lesson_id ?? "") === LESSON_ID, "wrong lesson id in bundle");
  assert(String(model.payload.lessons[0]?.source_text ?? "").trim().length > 20, "lesson DDN source missing from bundle");
  assert(Array.isArray(model?.payload?.reports) && model.payload.reports.some((row) => row.report_id === "lesson_plan_v0"), "lesson_plan.v0 report missing from bundle");
  return model.payload;
}

async function openPublishForPaste(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=publish&catalog_mode=full`, { waitUntil: "domcontentloaded" });
  await waitVisible(page, "#screen-publish");
  await page.locator("#btn-paste-local-package").click();
  await page.waitForSelector("#local-package-paste-text", { state: "visible", timeout: 20_000 });
}

async function pastePayload(page, payload) {
  const text = JSON.stringify(payload, null, 2);
  await page.evaluate((value) => {
    const node = document.querySelector("#local-package-paste-text");
    if (!node) throw new Error("paste textarea missing");
    node.value = value;
    node.dispatchEvent(new Event("input", { bubbles: true }));
  }, text);
  await page.locator("#btn-local-package-paste-open").click();
}

async function importBundleInNewContext(browser, baseUrl, payload, failures) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  await context.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
  const page = await context.newPage();
  installFailureCapture(page, failures);
  await openPublishForPaste(page, baseUrl);
  await pastePayload(page, payload);
  await waitVisible(page, "#screen-run");
  await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true, null, { timeout: 30_000 });
  const result = await page.evaluate(() => {
    const action = window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__;
    return {
      action,
      visibleRun: !document.querySelector("#screen-run")?.classList?.contains("hidden"),
      teacherProfileEditorsInRun: document.querySelectorAll("#screen-run [data-deployment-profile-editor]").length,
      onboardingProfile: document.querySelector("#screen-run")?.dataset?.onboardingProfile ?? "",
      title: document.querySelector("[data-run-title]")?.textContent?.trim() ?? "",
      localPackageHash: document.querySelector("#screen-run")?.dataset?.localPackageHash ?? "",
    };
  });
  assert(result.visibleRun, "import did not open run screen");
  assert(result.action?.classroom_exchange_hash === payload.classroom_exchange.payload_hash, "import did not preserve exchange hash");
  assert(result.action?.deployment_profile_hash === payload.manifest.deployment_profile_hash, "import did not preserve deployment hash");
  assert(result.teacherProfileEditorsInRun === 0, "deployment profile editor leaked outside publish screen");
  assert(result.onboardingProfile === "student", "imported lesson did not open student onboarding profile");
  await context.close();
  return result;
}

function cloneWith(payload, mutator) {
  const next = JSON.parse(JSON.stringify(payload));
  mutator(next);
  return next;
}

async function expectRejected(page, baseUrl, payload, expectedText) {
  await openPublishForPaste(page, baseUrl);
  await pastePayload(page, payload);
  await page.waitForFunction(() => {
    const node = document.querySelector("[data-local-package-import-status]");
    return node?.dataset?.state === "error" && String(node.textContent ?? "").trim();
  }, null, { timeout: 30_000 });
  const text = await page.locator("[data-local-package-import-status]").textContent();
  assert(String(text).includes(expectedText), `expected Korean diagnostic containing ${expectedText}, got: ${text}`);
  assert(!/studio_local_package_|Error:|http:\/\/|https:\/\/|[A-Z]:\\/u.test(String(text)), `internal diagnostic leaked: ${text}`);
  return String(text);
}

async function main() {
  const { server, baseUrl } = await createServer(process.cwd());
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.grantPermissions(["clipboard-read", "clipboard-write"], { origin: baseUrl });
    const page = await context.newPage();
    installFailureCapture(page, failures);

    await openPublishWithCurrentLesson(page, baseUrl);
    const payload = await exportCurrentLessonBundle(page);
    await importBundleInNewContext(browser, baseUrl, payload, failures);

    const tamperPage = await context.newPage();
    installFailureCapture(tamperPage, failures);
    await expectRejected(
      tamperPage,
      baseUrl,
      cloneWith(payload, (next) => { next.classroom_exchange.schema = "bad.schema"; }),
      "수업 교환 봉투 형식",
    );
    await expectRejected(
      tamperPage,
      baseUrl,
      cloneWith(payload, (next) => { next.classroom_exchange.payload_hash = "candidate:fnv1a32:classroom_exchange_payload:00000000"; }),
      "수업 교환 봉투 해시",
    );
    await expectRejected(
      tamperPage,
      baseUrl,
      cloneWith(payload, (next) => { next.manifest.deployment_profile.allowed_controls[0].min = -999; }),
      "배포 프로필 해시",
    );
    await expectRejected(
      tamperPage,
      baseUrl,
      cloneWith(payload, (next) => { next.package_hash = "candidate:fnv1a32:studio_local_package:00000000"; }),
      "배포 묶음 해시",
    );
    await expectRejected(
      tamperPage,
      baseUrl,
      cloneWith(payload, (next) => { next.classroom_exchange.project_snapshot.oversized = "x".repeat(2_000_050); }),
      "허용 크기",
    );

    await context.close();
    assert(failures.length === 0, failures.join("\n"));
    console.log(OK);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((error) => {
  console.error(error?.stack || String(error));
  process.exit(1);
});
