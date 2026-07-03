#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "rpg_story_package: ok";

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
    "rpg_box_authoring_ui.js",
    "rpg_story_package.js",
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
    await page.waitForSelector("[data-rpg-story-package][data-rpg-story-package-status='rpg_story_package_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_RPG_STORY_PACKAGE_ROWS,
        buildRpgStoryPackage,
        formatRpgStoryPackageText,
      } = await import("./rpg_story_package.js");
      const storyPackage = buildRpgStoryPackage({ rows: DEFAULT_RPG_STORY_PACKAGE_ROWS });
      return {
        storyPackage,
        text: formatRpgStoryPackageText(storyPackage),
      };
    });
    const storyPackage = moduleResult.storyPackage;
    assert(storyPackage.schema === "ddn.seamgrim.rpg_story.package.v1", "schema mismatch");
    assert(storyPackage.work_item === "CHA4_RPG_STORY_PACKAGE_V1", "work item mismatch");
    assert(storyPackage.primary_coordinate === "차-4", "coordinate mismatch");
    assert(storyPackage.status === "rpg_story_package_ready", "status mismatch");
    assert(storyPackage.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(storyPackage.story_package_claim === true, "story package claim mismatch");
    assert(storyPackage.manifest_claim === true, "manifest claim mismatch");
    assert(storyPackage.map_snapshot_claim === true, "map snapshot claim mismatch");
    assert(storyPackage.script_bundle_claim === true, "script bundle claim mismatch");
    assert(storyPackage.playtest_transcript_claim === true, "playtest transcript claim mismatch");
    assert(storyPackage.product_ui_change === true, "product UI change mismatch");
    assert(storyPackage.runtime_claim === false, "runtime claim must stay false");
    assert(storyPackage.engine_adapter_claim === false, "engine adapter claim must stay false");
    assert(storyPackage.registry_publish_claim === false, "registry publish claim must stay false");
    assert(storyPackage.public_upload_claim === false, "public upload claim must stay false");
    assert(storyPackage.cloud_sync_claim === false, "cloud sync claim must stay false");
    assert(storyPackage.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(storyPackage.progress.current_stage_total === 5, "stage total mismatch");
    assert(storyPackage.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(storyPackage.progress.roadmap_v2_matrix_behavior_closed === 16, "roadmap closed mismatch");
    assert(storyPackage.progress.roadmap_v2_matrix_behavior_percent === 18, "roadmap percent mismatch");
    assert(storyPackage.progress.roadmap_v2_pack_evidence_reference_closed === 36, "pack ref mismatch");
    assert(storyPackage.progress.roadmap_v2_pack_evidence_reference_percent === 40, "pack ref percent mismatch");
    assert(storyPackage.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(storyPackage.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(storyPackage.artifacts.map((row) => row.id).join(",") === "manifest,map_snapshot,script_bundle,playtest_transcript", "artifact order mismatch");
    assert(storyPackage.package_files.map((file) => file.kind).join(",") === "manifest,map_snapshot,script_bundle,playtest_transcript", "package file order mismatch");
    assert(String(storyPackage.package_text).includes("coordinate:차-4"), "package text missing coordinate");
    assert(String(moduleResult.text).includes("story_package_claim\ttrue"), "text missing story package claim");
    assert(String(moduleResult.text).includes("engine_adapter_claim\tfalse"), "text missing adapter boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t16/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-rpg-story-package]");
      const buttons = Array.from(document.querySelectorAll(".rpg-story-package-btn[data-rpg-story-package]"));
      buttons.find((button) => button.getAttribute("data-rpg-story-package") === "script_bundle")?.click();
      document.querySelector("[data-rpg-story-package-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-rpg-story-package-status") || "",
        copied: root?.getAttribute("data-rpg-story-package-copied") || "",
        buttonCount: buttons.length,
        fileCount: document.querySelectorAll("[data-rpg-story-package-file]").length,
        progress: document.querySelector("[data-rpg-story-package-progress]")?.textContent || "",
        summary: document.querySelector("[data-rpg-story-package-summary]")?.textContent || "",
        title: document.querySelector("[data-rpg-story-package-active-title]")?.textContent || "",
        link: document.querySelector("[data-rpg-story-package-active-link]")?.textContent || "",
        preview: document.querySelector("[data-rpg-story-package-preview]")?.textContent || "",
        globalSchema: window.__SEAMGRIM_RPG_STORY_PACKAGE__?.schema || "",
        globalText: window.__SEAMGRIM_RPG_STORY_PACKAGE_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "rpg_story_package_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `artifact count mismatch: ${domResult.buttonCount}`);
    assert(domResult.fileCount === 4, `package file count mismatch: ${domResult.fileCount}`);
    assert(domResult.progress.includes("16/90 ROADMAP") && domResult.progress.includes("5/5 stage"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("manifest") && domResult.summary.includes("script bundle") && domResult.summary.includes("engine adapter"), "summary missing package workflow");
    assert(domResult.title === "Script bundle", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("seamgrim://rpg-story-package/local/script/"), "script bundle local URI mismatch");
    assert(domResult.preview.includes("story.manifest.detjson") && domResult.preview.includes("playtest.transcript.txt"), "package preview mismatch");
    assert(domResult.globalSchema === "ddn.seamgrim.rpg_story.package.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t36/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("cloud_sync_claim\tfalse"), "global text missing cloud boundary");

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
