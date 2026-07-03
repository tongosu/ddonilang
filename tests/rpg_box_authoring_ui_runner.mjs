#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "rpg_box_authoring_ui: ok";

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
    "screens/rpg_box.js",
    "block_editor/rpgbox_palette.js",
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
    await page.waitForSelector("[data-rpg-box-authoring-ui][data-rpg-box-authoring-ui-status='rpg_box_authoring_ui_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_RPG_BOX_AUTHORING_ROWS,
        buildRpgBoxAuthoringUi,
        formatRpgBoxAuthoringUiText,
      } = await import("./rpg_box_authoring_ui.js");
      const authoring = buildRpgBoxAuthoringUi({ rows: DEFAULT_RPG_BOX_AUTHORING_ROWS });
      return {
        authoring,
        text: formatRpgBoxAuthoringUiText(authoring),
      };
    });
    const authoring = moduleResult.authoring;
    assert(authoring.schema === "ddn.seamgrim.rpg_box.authoring_ui.v1", "schema mismatch");
    assert(authoring.work_item === "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1", "work item mismatch");
    assert(authoring.primary_coordinate === "차-3", "coordinate mismatch");
    assert(authoring.status === "rpg_box_authoring_ui_ready", "status mismatch");
    assert(authoring.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(authoring.authoring_ui_claim === true, "authoring UI claim mismatch");
    assert(authoring.map_editor_claim === true, "map editor claim mismatch");
    assert(authoring.script_block_claim === true, "script block claim mismatch");
    assert(authoring.playtest_handoff_claim === true, "playtest handoff claim mismatch");
    assert(authoring.product_ui_change === true, "product UI change mismatch");
    assert(authoring.runtime_claim === false, "runtime claim must stay false");
    assert(authoring.story_package_claim === false, "story package claim must stay false");
    assert(authoring.engine_adapter_claim === false, "engine adapter claim must stay false");
    assert(authoring.registry_publish_claim === false, "registry publish claim must stay false");
    assert(authoring.cloud_sync_claim === false, "cloud sync claim must stay false");
    assert(authoring.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(authoring.progress.current_stage_total === 5, "stage total mismatch");
    assert(authoring.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(authoring.progress.roadmap_v2_matrix_behavior_closed === 15, "roadmap closed mismatch");
    assert(authoring.progress.roadmap_v2_matrix_behavior_percent === 17, "roadmap percent mismatch");
    assert(authoring.progress.roadmap_v2_pack_evidence_reference_closed === 35, "pack ref mismatch");
    assert(authoring.progress.roadmap_v2_pack_evidence_reference_percent === 39, "pack ref percent mismatch");
    assert(authoring.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(authoring.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(authoring.panels.map((row) => row.id).join(",") === "map_editor,script_blocks,playtest", "panel order mismatch");
    assert(authoring.map_cells.map((cell) => cell.id).join(",") === "start,door,npc", "map cell order mismatch");
    assert(authoring.script_lines.some((line) => String(line).includes("open_door")), "script action missing");
    assert(String(authoring.playtest_ddn).includes("roadmap matrix behavior: 15/90 = 17%"), "playtest DDN missing progress");
    assert(String(moduleResult.text).includes("authoring_ui_claim\ttrue"), "text missing authoring claim");
    assert(String(moduleResult.text).includes("engine_adapter_claim\tfalse"), "text missing adapter boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t15/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-rpg-box-authoring-ui]");
      const buttons = Array.from(document.querySelectorAll("[data-rpg-box-authoring]"));
      buttons.find((button) => button.getAttribute("data-rpg-box-authoring") === "script_blocks")?.click();
      document.querySelector("[data-rpg-box-authoring-copy]")?.click();
      document.querySelector("[data-rpg-box-authoring-open]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 50));
      return {
        rootStatus: root?.getAttribute("data-rpg-box-authoring-ui-status") || "",
        copied: root?.getAttribute("data-rpg-box-authoring-ui-copied") || "",
        opened: root?.getAttribute("data-rpg-box-authoring-ui-opened") || "",
        buttonCount: buttons.length,
        cellCount: document.querySelectorAll("[data-rpg-box-authoring-cell]").length,
        progress: document.querySelector("[data-rpg-box-authoring-progress]")?.textContent || "",
        summary: document.querySelector("[data-rpg-box-authoring-summary]")?.textContent || "",
        title: document.querySelector("[data-rpg-box-authoring-active-title]")?.textContent || "",
        link: document.querySelector("[data-rpg-box-authoring-active-link]")?.textContent || "",
        script: document.querySelector("[data-rpg-box-authoring-script]")?.textContent || "",
        globalSchema: window.__SEAMGRIM_RPG_BOX_AUTHORING_UI__?.schema || "",
        globalText: window.__SEAMGRIM_RPG_BOX_AUTHORING_UI_TEXT__ || "",
        lastOpenWorkItem: window.__SEAMGRIM_RPG_BOX_LAST_OPEN__?.work_item || "",
      };
    });
    assert(domResult.rootStatus === "rpg_box_authoring_ui_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.opened === "true", "open state missing");
    assert(domResult.buttonCount === 3, `panel count mismatch: ${domResult.buttonCount}`);
    assert(domResult.cellCount === 3, `map cell count mismatch: ${domResult.cellCount}`);
    assert(domResult.progress.includes("15/90 ROADMAP") && domResult.progress.includes("5/5 stage"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("map editor") && domResult.summary.includes("script blocks") && domResult.summary.includes("playtest"), "summary missing authoring workflow");
    assert(domResult.title === "Script blocks", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("seamgrim://rpg-box/local/script/"), "script handoff link mismatch");
    assert(domResult.script.includes("open_door") && domResult.script.includes("안녕 용사 방문=1"), "script preview mismatch");
    assert(domResult.globalSchema === "ddn.seamgrim.rpg_box.authoring_ui.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t35/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("cloud_sync_claim\tfalse"), "global text missing cloud boundary");
    assert(domResult.lastOpenWorkItem === "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1", "playtest handoff work item mismatch");

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
