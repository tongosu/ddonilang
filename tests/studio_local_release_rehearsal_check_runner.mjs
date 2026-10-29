#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_local_release_rehearsal_check: ok";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

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
  for (const rel of ["index.html", "app.js", "studio_local_release_rehearsal_check.js"]) {
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-local-release-rehearsal-check][data-local-release-rehearsal-check-status='local_release_rehearsal_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_LOCAL_RELEASE_REHEARSAL_ROWS,
        buildLocalReleaseRehearsalCheck,
        formatLocalReleaseRehearsalCheckText,
      } = await import("./studio_local_release_rehearsal_check.js");
      const rehearsal = buildLocalReleaseRehearsalCheck({
        rehearsalRows: DEFAULT_LOCAL_RELEASE_REHEARSAL_ROWS,
      });
      return {
        rehearsal,
        text: formatLocalReleaseRehearsalCheckText(rehearsal),
      };
    });

    const rehearsal = moduleResult.rehearsal;
    assert(rehearsal.__종류 === "studio_local_release_rehearsal_check", "kind mismatch");
    assert(rehearsal.schema === "ddn.studio.local_release_rehearsal_check.v1", "schema mismatch");
    assert(rehearsal.status === "local_release_rehearsal_ready", `status mismatch: ${rehearsal.status}`);
    assert(rehearsal.next_state === "AWAIT_EXPLICIT_RELEASE_APPROVAL", "next state mismatch");
    assert(rehearsal.dry_run_only === true, "dry-run-only mismatch");
    assert(rehearsal.planned_asset_count === 4, `asset count mismatch: ${rehearsal.planned_asset_count}`);
    assert(rehearsal.all_planned_assets_generated_now === false, "asset generation flag mismatch");
    assert(rehearsal.release_approval_claim === false, "must not claim approval");
    assert(rehearsal.release_execution_claim === false, "must not claim execution");
    assert(rehearsal.archive_generation_claim === false, "must not claim archive generation");
    assert(rehearsal.publication_checksum_generation_claim === false, "must not claim checksum generation");
    assert(rehearsal.product_ui_change === true, "must claim product ui change");
    assert(rehearsal.rehearsal_row_count === 5, `row count mismatch: ${rehearsal.rehearsal_row_count}`);
    assert(rehearsal.ready_stage_count === 6, `ready stage mismatch: ${rehearsal.ready_stage_count}`);
    assert(rehearsal.progress.super_long_behavior_closed === 8, "super-long closed mismatch");
    assert(rehearsal.progress.super_long_percent === 44, "super-long percent mismatch");
    assert(rehearsal.progress.current_stage_closed === 4, "current stage closed mismatch");
    assert(rehearsal.progress.current_stage_total === 4, "current stage total mismatch");
    assert(rehearsal.progress.current_stage_percent === 100, "current stage percent mismatch");
    assert(rehearsal.progress.roadmap_v2_behavior_closed === 6, "roadmap closed mismatch");
    assert(rehearsal.progress.roadmap_v2_percent === 7, "roadmap percent mismatch");
    assert(rehearsal.progress.roadmap_v2_pack_evidence_reference_closed === 25, "pack evidence closed mismatch");
    assert(rehearsal.progress.roadmap_v2_pack_evidence_reference_percent === 28, "pack evidence percent mismatch");
    assert(rehearsal.next_item === "MA5_SEAMGRIM_CURRICULUM_5_LTS_PACK_CLOSURE_V1", "next item mismatch");
    assert(String(moduleResult.text).includes("dry_run_only\ttrue"), "formatted text missing dry-run flag");
    assert(String(moduleResult.text).includes("release_execution_claim\tfalse"), "formatted text missing release boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-local-release-rehearsal-check]");
      const buttons = Array.from(document.querySelectorAll("[data-rehearsal-check]"));
      const progress = document.querySelector("[data-rehearsal-check-progress]")?.textContent || "";
      const assets = document.querySelector("[data-rehearsal-check-assets]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-rehearsal-check") === "asset_plan_anchor")?.click();
      const title = document.querySelector("[data-rehearsal-check-active-title]")?.textContent || "";
      const lane = document.querySelector("[data-rehearsal-check-active-lane]")?.textContent || "";
      document.querySelector("[data-rehearsal-check-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-local-release-rehearsal-check-status") || "",
        copied: root?.getAttribute("data-local-release-rehearsal-check-copied") || "",
        buttonCount: buttons.length,
        progress,
        assets,
        title,
        lane,
        globalSchema: window.__SEAMGRIM_LOCAL_RELEASE_REHEARSAL_CHECK__?.schema || "",
        globalText: window.__SEAMGRIM_LOCAL_RELEASE_REHEARSAL_CHECK_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "local_release_rehearsal_ready", `dom status mismatch: ${domResult.rootStatus}`);
    assert(domResult.buttonCount === 5, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("4/4 follow-up") && domResult.progress.includes("100%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.progress.includes("dry-run only"), `dry-run text mismatch: ${domResult.progress}`);
    assert(domResult.assets.includes("4 assets") && domResult.assets.includes("generated_now=false"), `assets text mismatch: ${domResult.assets}`);
    assert(domResult.title === "asset plan", `title mismatch: ${domResult.title}`);
    assert(domResult.lane === "asset_plan", `lane mismatch: ${domResult.lane}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.local_release_rehearsal_check.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("rehearsal_id\tlane"), "global text missing header");

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
