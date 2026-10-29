#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_public_release_approval_recheck: ok";
const REQUIRED = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다";

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
  for (const rel of ["index.html", "app.js", "studio_public_release_approval_recheck.js"]) {
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
    await page.waitForSelector("[data-public-release-approval-recheck][data-public-release-approval-recheck-status='approval_recheck_waiting']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_PUBLIC_RELEASE_APPROVAL_RECHECK_ROWS,
        REQUIRED_PUBLIC_RELEASE_APPROVAL_PHRASE,
        buildPublicReleaseApprovalRecheck,
        formatPublicReleaseApprovalRecheckText,
      } = await import("./studio_public_release_approval_recheck.js");
      const recheck = buildPublicReleaseApprovalRecheck({
        approvalRows: DEFAULT_PUBLIC_RELEASE_APPROVAL_RECHECK_ROWS,
      });
      return {
        required: REQUIRED_PUBLIC_RELEASE_APPROVAL_PHRASE,
        recheck,
        text: formatPublicReleaseApprovalRecheckText(recheck),
      };
    });

    const recheck = moduleResult.recheck;
    assert(moduleResult.required === REQUIRED, "exported required phrase mismatch");
    assert(recheck.__종류 === "studio_public_release_approval_recheck", "kind mismatch");
    assert(recheck.schema === "ddn.studio.public_release_approval_recheck.v1", "schema mismatch");
    assert(recheck.status === "approval_recheck_waiting", `status mismatch: ${recheck.status}`);
    assert(recheck.required_approval_phrase === REQUIRED, "required phrase mismatch");
    assert(recheck.next_state === "AWAIT_EXPLICIT_RELEASE_APPROVAL", "next state mismatch");
    assert(recheck.generic_next_dev_request_is_approval === false, "generic request must not approve");
    assert(recheck.current_request_is_release_approval === false, "current request must not approve");
    assert(recheck.release_approval_claim === false, "must not claim approval");
    assert(recheck.release_execution_claim === false, "must not claim execution");
    assert(recheck.product_ui_change === true, "must claim product ui change");
    assert(recheck.approval_row_count === 5, `row count mismatch: ${recheck.approval_row_count}`);
    assert(recheck.ready_stage_count === 6, `ready stage mismatch: ${recheck.ready_stage_count}`);
    assert(recheck.progress.super_long_percent === 50, "super-long percent mismatch");
    assert(recheck.progress.current_stage_closed === 2, "followup closed mismatch");
    assert(recheck.progress.current_stage_percent === 25, "followup percent mismatch");
    assert(recheck.progress.roadmap_v2_behavior_closed === 51, "roadmap closed mismatch");
    assert(recheck.progress.roadmap_v2_percent === 57, "roadmap percent mismatch");
    assert(recheck.next_item === "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1", "next item mismatch");
    assert(String(moduleResult.text).includes(`required_approval_phrase\t${REQUIRED}`), "formatted text missing required phrase");
    assert(String(moduleResult.text).includes("release_execution_claim\tfalse"), "formatted text missing release boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-public-release-approval-recheck]");
      const buttons = Array.from(document.querySelectorAll("[data-approval-recheck]"));
      const progress = document.querySelector("[data-approval-recheck-progress]")?.textContent || "";
      const phrase = document.querySelector("[data-approval-recheck-phrase]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-approval-recheck") === "current_request_rejected")?.click();
      const title = document.querySelector("[data-approval-recheck-active-title]")?.textContent || "";
      const lane = document.querySelector("[data-approval-recheck-active-lane]")?.textContent || "";
      document.querySelector("[data-approval-recheck-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-public-release-approval-recheck-status") || "",
        copied: root?.getAttribute("data-public-release-approval-recheck-copied") || "",
        buttonCount: buttons.length,
        progress,
        phrase,
        title,
        lane,
        globalSchema: window.__SEAMGRIM_PUBLIC_RELEASE_APPROVAL_RECHECK__?.schema || "",
        globalText: window.__SEAMGRIM_PUBLIC_RELEASE_APPROVAL_RECHECK_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "approval_recheck_waiting", `dom status mismatch: ${domResult.rootStatus}`);
    assert(domResult.buttonCount === 5, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("2/8 follow-up") && domResult.progress.includes("25%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.progress.includes("AWAIT_EXPLICIT_RELEASE_APPROVAL"), `state text mismatch: ${domResult.progress}`);
    assert(domResult.phrase === REQUIRED, `phrase text mismatch: ${domResult.phrase}`);
    assert(domResult.title === "current request", `title mismatch: ${domResult.title}`);
    assert(domResult.lane === "current_request_boundary", `lane mismatch: ${domResult.lane}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.public_release_approval_recheck.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("approval_id\tlane"), "global text missing header");

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
