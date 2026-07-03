#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_release_review_packet_dashboard: ok";
const REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다";
const DEV_SURFACE_PANEL_IDS = [
  "teacher-feedback-preview-panel",
  "classroom-operations-panel-preview",
  "benchmark-baseline-local-snapshot",
  "release-review-packet-dashboard",
  "lesson-publication-review-surface",
  "ma3-regression-gate-matrix",
  "ma3-next-queue-coordinate-lock",
  "operations-preview-stage-closure",
  "productization-stage-rebase",
  "seamgrim-numeric-track-consolidation",
  "numeric-report-workflow-stage",
  "numeric-result-report-stage",
  "productization-stage-closure",
  "post-super-long-rebase",
  "public-release-approval-recheck",
  "local-release-rehearsal-check",
  "publication-artifact-dry-run",
  "teacher-feedback-loop-seed",
  "classroom-operations-triage",
  "benchmark-baseline-prep-dry-run",
  "next-roadmap-v2-coordinate-lock",
  "ma3-next-development-queue-rebase",
  "free-lab-experiment-report",
  "free-lab-ui-pack",
  "free-lab-share-pack",
  "free-lab-research-workflow",
  "rpg-story-package",
  "rpg-engine-adapter-lts",
  "ttonimaru-publication-read-api",
  "ttonimaru-project-share-ui",
  "ttonimaru-public-registry-seed",
  "ttonimaru-platform-hardening",
  "toolchain-diagnostic-ui-lsp",
  "toolchain-registry-verification",
  "toolchain-benchmark-lts",
  "social-world-bridge-pack",
  "social-world-policy-ghost-ui",
  "social-world-template-registry",
  "social-world-lts-readiness",
  "education-assessment-pack",
  "education-classroom-ui-pack",
  "education-publication-pack",
  "education-operations-lts",
  "question-card-smoke",
  "question-card-validation",
  "question-card-dev-assist",
  "question-card-author-tool-share",
  "question-card-workflow-hardening",
  "seulgi-proposal-ui",
  "seulgi-replay-safe-workflow",
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) throw new Error(`missing file: ${file}`);
}

async function assertDevSurfacePanelIdListCurrent(uiRoot) {
  const source = await fs.readFile(path.join(uiRoot, "dev_surfaces.js"), "utf-8");
  const sourceIds = Array.from(source.matchAll(/elementId:\s*"([^"]+)"/g), (match) => match[1])
    .filter((id) => id !== "definition.elementId");
  const expected = Array.from(new Set(sourceIds)).sort();
  const actual = Array.from(new Set(DEV_SURFACE_PANEL_IDS)).sort();
  assert(
    JSON.stringify(actual) === JSON.stringify(expected),
    `dev surface panel id guard is stale\nexpected=${expected.join(",")}\nactual=${actual.join(",")}`,
  );
}

async function assertReleaseDashboardCssIsDevOnly(uiRoot) {
  const [defaultCss, devCss] = await Promise.all([
    fs.readFile(path.join(uiRoot, "styles.css"), "utf-8"),
    fs.readFile(path.join(uiRoot, "dev_surfaces.css"), "utf-8"),
  ]);
  for (const token of [
    ".teacher-feedback-preview-panel",
    ".classroom-operations-panel-preview",
    ".benchmark-baseline-local-snapshot",
    ".ma3-regression-gate-matrix",
    ".ma3-next-queue-coordinate-lock",
    ".release-review-packet-dashboard",
    ".release-review-dashboard-head",
    ".lesson-publication-review-surface",
    ".lesson-publication-surface-head",
    ".operations-preview-stage-closure",
    ".public-release-approval-recheck",
    ".local-release-rehearsal-check",
    ".publication-artifact-dry-run",
    ".teacher-feedback-loop-seed",
    ".classroom-operations-triage",
    ".free-lab-experiment-report",
    ".productization-stage-rebase",
    ".productization-stage-closure",
    ".post-super-long-rebase",
    ".benchmark-baseline-prep-dry-run",
    ".ma3-next-development-queue-rebase",
    ".rpg-story-package",
    ".rpg-engine-adapter-lts",
    ".ttonimaru-publication-read-api",
    ".ttonimaru-project-share-ui",
    ".ttonimaru-public-registry-seed",
    ".ttonimaru-platform-hardening",
    ".toolchain-diagnostic-ui-lsp",
    ".toolchain-registry-verification",
    ".toolchain-benchmark-lts",
    ".social-world-template-registry",
    ".social-world-lts-readiness",
    ".social-world-policy-ghost-ui",
    ".social-world-bridge-pack",
    ".education-assessment-pack",
    ".education-classroom-ui-pack",
  ]) {
    assert(!defaultCss.includes(token), `default teacher CSS leaked dev panel token: ${token}`);
    assert(devCss.includes(token), `dev surface CSS missing panel token: ${token}`);
  }
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  return "text/plain; charset=utf-8";
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
      else resolve({
        server,
        baseUrl: `http://127.0.0.1:${address.port}`,
        publicBaseUrl: `http://studio.example.test:${address.port}`,
      });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    return (
      url.pathname === "/api/lessons/inventory" ||
      url.pathname === "/api/lesson-inventory" ||
      url.pathname.startsWith("/lessons/") ||
      url.pathname.startsWith("/solutions/seamgrim_ui_mvp/lessons/") ||
      url.pathname.startsWith("/seed_lessons_v1/")
    );
  } catch (_) {
    return false;
  }
}

async function assertDefaultDevSurfacesHidden(page, baseUrl) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse .catalog-body");
  const state = await page.evaluate((ids) => {
    const visibleText = String(document.body.innerText ?? "");
    const forbiddenVisibleTerms = [
      "릴리스",
      "release",
      "운영 패널",
      "operations",
      "질문 카드",
      "실험 패널",
      "free lab",
      "roadmap",
    ];
    return {
      bodyDevClass: document.body.classList.contains("dev-surfaces-enabled"),
      devRootCount: document.querySelectorAll("#dev-surface-root").length,
      visibleIds: ids.filter((id) => document.getElementById(id)),
      forbiddenVisibleTerms: forbiddenVisibleTerms.filter((term) => visibleText.toLowerCase().includes(term.toLowerCase())),
      devSurfaceResourceUrls: performance.getEntriesByType("resource")
        .map((entry) => entry.name)
        .filter((name) => (
          name.includes("/dev_surfaces.js")
          || name.endsWith("dev_surfaces.js")
          || name.includes("/dev_surfaces.css")
          || name.endsWith("dev_surfaces.css")
        )),
    };
  }, DEV_SURFACE_PANEL_IDS);
  assert(state.bodyDevClass === false, "default UI must not enable dev surface body class");
  assert(state.devRootCount === 0, `default UI leaked dev surface root: ${state.devRootCount}`);
  assert(state.visibleIds.length === 0, `default UI leaked dev panels: ${state.visibleIds.join(", ")}`);
  assert(
    state.forbiddenVisibleTerms.length === 0,
    `default teacher UI leaked dev-surface terms: ${state.forbiddenVisibleTerms.join(", ")}`
  );
  assert(
    state.devSurfaceResourceUrls.length === 0,
    `default UI loaded dev surface assets: ${state.devSurfaceResourceUrls.join(", ")}`
  );

  await page.evaluate(() => localStorage.setItem("seamgrim.dev_surfaces", "1"));
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse .catalog-body");
  const staleStorageState = await page.evaluate((ids) => {
    const visibleText = String(document.body.innerText ?? "");
    const forbiddenVisibleTerms = [
      "릴리스",
      "release",
      "운영 패널",
      "operations",
      "질문 카드",
      "실험 패널",
      "free lab",
      "roadmap",
    ];
    return {
      stored: localStorage.getItem("seamgrim.dev_surfaces"),
      bodyDevClass: document.body.classList.contains("dev-surfaces-enabled"),
      devRootCount: document.querySelectorAll("#dev-surface-root").length,
      visibleIds: ids.filter((id) => document.getElementById(id)),
      forbiddenVisibleTerms: forbiddenVisibleTerms.filter((term) => visibleText.toLowerCase().includes(term.toLowerCase())),
      devSurfaceResourceUrls: performance.getEntriesByType("resource")
        .map((entry) => entry.name)
        .filter((name) => (
          name.includes("/dev_surfaces.js")
          || name.endsWith("dev_surfaces.js")
          || name.includes("/dev_surfaces.css")
          || name.endsWith("dev_surfaces.css")
        )),
    };
  }, DEV_SURFACE_PANEL_IDS);
  assert(staleStorageState.stored === "1", "default UI stale dev storage setup failed");
  assert(staleStorageState.bodyDevClass === false, "default UI must ignore stale dev surface storage");
  assert(staleStorageState.devRootCount === 0, `default UI stale storage leaked dev surface root: ${staleStorageState.devRootCount}`);
  assert(staleStorageState.visibleIds.length === 0, `default UI stale storage leaked dev panels: ${staleStorageState.visibleIds.join(", ")}`);
  assert(
    staleStorageState.forbiddenVisibleTerms.length === 0,
    `default teacher UI stale storage leaked dev-surface terms: ${staleStorageState.forbiddenVisibleTerms.join(", ")}`
  );
  assert(
    staleStorageState.devSurfaceResourceUrls.length === 0,
    `default UI stale storage loaded dev surface assets: ${staleStorageState.devSurfaceResourceUrls.join(", ")}`
  );
  await page.evaluate(() => localStorage.removeItem("seamgrim.dev_surfaces"));
}

async function assertNonLocalDevSurfacesBlocked(page, publicBaseUrl) {
  await page.goto(`${publicBaseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse .catalog-body");
  const state = await page.evaluate(() => ({
    hostname: window.location.hostname,
    bodyDevClass: document.body.classList.contains("dev-surfaces-enabled"),
    devRootCount: document.querySelectorAll("#dev-surface-root").length,
    devSurfaceResourceUrls: performance.getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => (
        name.includes("/dev_surfaces.js")
        || name.endsWith("dev_surfaces.js")
        || name.includes("/dev_surfaces.css")
        || name.endsWith("dev_surfaces.css")
      )),
  }));
  assert(state.hostname === "studio.example.test", `non-local host mismatch: ${state.hostname}`);
  assert(state.bodyDevClass === false, "non-local UI must ignore devSurfaces query");
  assert(state.devRootCount === 0, `non-local UI leaked dev surface root: ${state.devRootCount}`);
  assert(
    state.devSurfaceResourceUrls.length === 0,
    `non-local query loaded dev_surfaces.js: ${state.devSurfaceResourceUrls.join(", ")}`
  );

  await page.evaluate(() => localStorage.setItem("seamgrim.dev_surfaces", "1"));
  await page.goto(`${publicBaseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse .catalog-body");
  const storageState = await page.evaluate(() => ({
    hostname: window.location.hostname,
    stored: localStorage.getItem("seamgrim.dev_surfaces"),
    bodyDevClass: document.body.classList.contains("dev-surfaces-enabled"),
    devRootCount: document.querySelectorAll("#dev-surface-root").length,
    devSurfaceResourceUrls: performance.getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => (
        name.includes("/dev_surfaces.js")
        || name.endsWith("dev_surfaces.js")
        || name.includes("/dev_surfaces.css")
        || name.endsWith("dev_surfaces.css")
      )),
  }));
  assert(storageState.hostname === "studio.example.test", `non-local storage host mismatch: ${storageState.hostname}`);
  assert(storageState.stored === "1", "non-local dev surface storage setup failed");
  assert(storageState.bodyDevClass === false, "non-local UI must ignore dev surface storage");
  assert(storageState.devRootCount === 0, `non-local storage leaked dev surface root: ${storageState.devRootCount}`);
  assert(
    storageState.devSurfaceResourceUrls.length === 0,
    `non-local storage loaded dev_surfaces.js: ${storageState.devSurfaceResourceUrls.join(", ")}`
  );
  await page.evaluate(() => localStorage.removeItem("seamgrim.dev_surfaces"));

  await page.addInitScript(() => {
    window.SEAMGRIM_DEV_SURFACES = true;
  });
  await page.goto(`${publicBaseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse .catalog-body");
  const globalOverrideState = await page.evaluate(() => ({
    hostname: window.location.hostname,
    globalFlag: window.SEAMGRIM_DEV_SURFACES === true,
    bodyDevClass: document.body.classList.contains("dev-surfaces-enabled"),
    devRootCount: document.querySelectorAll("#dev-surface-root").length,
    legacyBrowseControlCount: [
      "#btn-filter-numeric-track",
      "#btn-filter-numeric-track-results",
      "#btn-preset-featured-seed-quick-recent",
      "#btn-copy-browse-preset-link",
      "#filter-quality",
      "#filter-sort",
    ].filter((selector) => document.querySelector(selector)).length,
    devSurfaceResourceUrls: performance.getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => (
        name.includes("/dev_surfaces.js")
        || name.endsWith("dev_surfaces.js")
        || name.includes("/dev_surfaces.css")
        || name.endsWith("dev_surfaces.css")
      )),
  }));
  assert(globalOverrideState.hostname === "studio.example.test", `non-local global host mismatch: ${globalOverrideState.hostname}`);
  assert(globalOverrideState.globalFlag === true, "non-local dev surface global override setup failed");
  assert(globalOverrideState.bodyDevClass === false, "non-local UI must ignore dev surface global override");
  assert(globalOverrideState.devRootCount === 0, `non-local global override leaked dev surface root: ${globalOverrideState.devRootCount}`);
  assert(
    globalOverrideState.legacyBrowseControlCount === 0,
    `non-local global override leaked legacy browse controls: ${globalOverrideState.legacyBrowseControlCount}`
  );
  assert(
    globalOverrideState.devSurfaceResourceUrls.length === 0,
    `non-local global override loaded dev_surfaces.js: ${globalOverrideState.devSurfaceResourceUrls.join(", ")}`
  );
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of ["index.html", "app.js", "studio_release_review_packet_dashboard.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }
  await assertDevSurfacePanelIdListCurrent(uiRoot);
  await assertReleaseDashboardCssIsDevOnly(uiRoot);

  const { server, baseUrl, publicBaseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({
      headless: true,
      args: ["--host-resolver-rules=MAP studio.example.test 127.0.0.1"],
    });
    const context = await browser.newContext({ viewport: { width: 1180, height: 760 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error" && !String(msg.text() ?? "").includes("Failed to load resource")) {
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => {
      const errorText = req.failure()?.errorText || "";
      if (errorText === "net::ERR_ABORTED" && isAllowedFallback404(req.url())) return;
      failures.push(`request failed: ${req.url()} ${errorText}`);
    });
    page.on("response", (res) => {
      if (res.status() >= 400 && !res.url().endsWith("/favicon.ico") && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await assertDefaultDevSurfacesHidden(page, baseUrl);
    await assertNonLocalDevSurfacesBlocked(page, publicBaseUrl);
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-release-review-packet-dashboard][data-release-review-status='release_review_dashboard_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_RELEASE_REVIEW_MATERIALS,
        DEFAULT_RELEASE_REVIEW_SNAPSHOT_ROWS,
        buildReleaseReviewPacketDashboard,
        formatReleaseReviewPacketDashboardText,
      } = await import("./studio_release_review_packet_dashboard.js");
      const dashboard = buildReleaseReviewPacketDashboard({
        snapshotRows: DEFAULT_RELEASE_REVIEW_SNAPSHOT_ROWS,
        reviewMaterials: DEFAULT_RELEASE_REVIEW_MATERIALS,
      });
      return {
        dashboard,
        text: formatReleaseReviewPacketDashboardText(dashboard),
      };
    });

    const dashboard = moduleResult.dashboard;
    assert(dashboard.__종류 === "studio_release_review_packet_dashboard", "dashboard kind mismatch");
    assert(dashboard.schema === "ddn.studio.release_review_packet_dashboard.v1", "dashboard schema mismatch");
    assert(dashboard.workflow_claim === "release_review_packet_dashboard", "workflow claim mismatch");
    assert(dashboard.product_ui_change === true, "dashboard must claim product ui change");
    assert(dashboard.runtime_claim === false, "dashboard must not claim runtime");
    assert(dashboard.release_approval_claim === false, "dashboard must not approve release");
    assert(dashboard.release_execution_claim === false, "dashboard must not execute release");
    assert(dashboard.public_release_claim === false, "dashboard must not claim public release");
    assert(dashboard.generic_next_dev_request_is_approval === false, "generic next-dev request must not approve");
    assert(dashboard.required_approval_phrase === REQUIRED_APPROVAL, "approval phrase mismatch");
    assert(dashboard.next_state === "AWAIT_EXPLICIT_RELEASE_APPROVAL", "next state mismatch");
    assert(dashboard.status === "release_review_dashboard_ready", `status mismatch: ${dashboard.status}`);
    assert(dashboard.dashboard_row_count === 6, `dashboard count mismatch: ${dashboard.dashboard_row_count}`);
    assert(dashboard.ready_stage_count === 6, `ready stage mismatch: ${dashboard.ready_stage_count}`);
    assert(dashboard.progress.super_long_behavior_closed === 18, "super-long closed mismatch");
    assert(dashboard.progress.super_long_percent === 100, "super-long percent mismatch");
    assert(dashboard.progress.current_stage_closed === 5, "current stage closed mismatch");
    assert(dashboard.progress.current_stage_percent === 63, "current stage percent mismatch");
    assert(dashboard.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(dashboard.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(String(moduleResult.text).includes("release_approval_claim\tfalse"), "formatted text missing approval boundary");
    assert(String(moduleResult.text).includes(REQUIRED_APPROVAL), "formatted text missing approval phrase");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-release-review-packet-dashboard]");
      const buttons = Array.from(document.querySelectorAll("[data-release-review-dashboard]"));
      const firstTitle = document.querySelector("[data-release-review-active-title]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-release-review-dashboard") === "registry_share_review_dashboard_card")?.click();
      const registryTitle = document.querySelector("[data-release-review-active-title]")?.textContent || "";
      const registryLane = document.querySelector("[data-release-review-active-lane]")?.textContent || "";
      const phrase = document.querySelector("[data-release-review-approval-phrase]")?.textContent || "";
      document.querySelector("[data-release-review-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        status: root?.getAttribute("data-release-review-status") || "",
        copied: root?.getAttribute("data-release-review-copied") || "",
        buttonCount: buttons.length,
        firstTitle,
        registryTitle,
        registryLane,
        phrase,
        globalSchema: window.__SEAMGRIM_RELEASE_REVIEW_PACKET_DASHBOARD__?.schema || "",
        globalText: window.__SEAMGRIM_RELEASE_REVIEW_PACKET_DASHBOARD_TEXT__ || "",
      };
    });
    assert(domResult.status === "release_review_dashboard_ready", `dom status mismatch: ${domResult.status}`);
    assert(domResult.buttonCount === 6, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.firstTitle.includes("승인 대기"), `first title mismatch: ${domResult.firstTitle}`);
    assert(domResult.registryTitle.includes("Registry/share"), `registry title mismatch: ${domResult.registryTitle}`);
    assert(domResult.registryLane === "registry_share_review", `registry lane mismatch: ${domResult.registryLane}`);
    assert(domResult.phrase === REQUIRED_APPROVAL, `dom approval phrase mismatch: ${domResult.phrase}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.release_review_packet_dashboard.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("dashboard_id\tdashboard_lane"), "global text missing dashboard header");

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
