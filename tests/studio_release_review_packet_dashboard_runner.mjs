#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_release_review_packet_dashboard: ok";
const REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다";

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
    return (
      url.pathname === "/api/lessons/inventory" ||
      url.pathname === "/api/lesson-inventory" ||
      url.pathname.startsWith("/lessons/") ||
      url.pathname.startsWith("/seed_lessons_v1/")
    );
  } catch (_) {
    return false;
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of ["index.html", "app.js", "studio_release_review_packet_dashboard.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1180, height: 760 }, locale: "ko-KR" });
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
