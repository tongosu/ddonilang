#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "free_lab_share_pack: ok";

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
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
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
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "free_lab_ui_pack.js",
    "free_lab_share_pack.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 840 }, locale: "ko-KR" });
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
    await page.waitForSelector("[data-free-lab-share-pack][data-free-lab-share-pack-status='free_lab_share_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_FREE_LAB_SHARE_ROWS,
        buildFreeLabSharePack,
        formatFreeLabSharePackText,
      } = await import("./free_lab_share_pack.js");
      const pack = buildFreeLabSharePack({ rows: DEFAULT_FREE_LAB_SHARE_ROWS });
      return {
        pack,
        text: formatFreeLabSharePackText(pack),
      };
    });
    const pack = moduleResult.pack;
    assert(pack.schema === "ddn.seamgrim.free_lab.share_pack.v1", "schema mismatch");
    assert(pack.work_item === "BA4_FREE_LAB_SHARE_PACK_CLOSURE_V1", "work item mismatch");
    assert(pack.primary_coordinate === "바-4", "coordinate mismatch");
    assert(pack.status === "free_lab_share_ready", "status mismatch");
    assert(pack.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(pack.local_share_claim === true, "local share claim mismatch");
    assert(pack.remix_link_claim === true, "remix link claim mismatch");
    assert(pack.product_ui_change === true, "product UI change mismatch");
    assert(pack.runtime_claim === false, "runtime claim must stay false");
    assert(pack.public_upload_claim === false, "public upload claim must stay false");
    assert(pack.registry_publish_claim === false, "registry publish claim must stay false");
    assert(pack.cloud_sync_claim === false, "cloud sync claim must stay false");
    assert(pack.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(pack.progress.current_stage_total === 5, "stage total mismatch");
    assert(pack.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(pack.progress.roadmap_v2_matrix_behavior_closed === 11, "roadmap closed mismatch");
    assert(pack.progress.roadmap_v2_matrix_behavior_percent === 12, "roadmap percent mismatch");
    assert(pack.progress.roadmap_v2_pack_evidence_reference_closed === 30, "pack ref mismatch");
    assert(pack.progress.roadmap_v2_pack_evidence_reference_percent === 33, "pack ref percent mismatch");
    assert(pack.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(pack.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(pack.shares.map((share) => share.id).join(",") === "snapshot,remix,handoff", "share order mismatch");
    assert(pack.shares.every((share) => share.share_link.startsWith("seamgrim://free-lab/local/")), "share links must be local");
    assert(String(moduleResult.text).includes("local_share_claim\ttrue"), "text missing local share claim");
    assert(String(moduleResult.text).includes("public_upload_claim\tfalse"), "text missing public upload boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t11/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-free-lab-share-pack]");
      const buttons = Array.from(document.querySelectorAll("[data-free-lab-share]"));
      buttons.find((button) => button.getAttribute("data-free-lab-share") === "handoff")?.click();
      document.querySelector("[data-free-lab-share-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-free-lab-share-pack-status") || "",
        copied: root?.getAttribute("data-free-lab-share-pack-copied") || "",
        buttonCount: buttons.length,
        progress: document.querySelector("[data-free-lab-share-progress]")?.textContent || "",
        summary: document.querySelector("[data-free-lab-share-summary]")?.textContent || "",
        title: document.querySelector("[data-free-lab-share-active-title]")?.textContent || "",
        link: document.querySelector("[data-free-lab-share-active-link]")?.textContent || "",
        globalSchema: window.__SEAMGRIM_FREE_LAB_SHARE_PACK__?.schema || "",
        globalText: window.__SEAMGRIM_FREE_LAB_SHARE_PACK_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "free_lab_share_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 3, `share count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("11/90 ROADMAP") && domResult.progress.includes("5/5 stage"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("local remix/share link") && domResult.summary.includes("registry publish"), "summary missing share boundary");
    assert(domResult.title === "인수인계", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("seamgrim://free-lab/local/handoff/"), "handoff link mismatch");
    assert(domResult.globalSchema === "ddn.seamgrim.free_lab.share_pack.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t30/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("registry_publish_claim\tfalse"), "global text missing false claim boundary");

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
