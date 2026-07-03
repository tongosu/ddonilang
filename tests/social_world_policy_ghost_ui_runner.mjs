#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "social_world_policy_ghost_ui: ok";

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
    "social_world_bridge_pack.js",
    "social_world_policy_ghost_ui.js",
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
    await page.waitForSelector("[data-social-world-policy-ghost-ui][data-social-world-policy-ghost-status='social_world_policy_ghost_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_SOCIAL_WORLD_POLICY_GHOST_ROWS,
        buildSocialWorldPolicyGhostUi,
        formatSocialWorldPolicyGhostUiText,
      } = await import("./social_world_policy_ghost_ui.js");
      const policyGhost = buildSocialWorldPolicyGhostUi({ rows: DEFAULT_SOCIAL_WORLD_POLICY_GHOST_ROWS });
      return { policyGhost, text: formatSocialWorldPolicyGhostUiText(policyGhost) };
    });
    const policyGhost = moduleResult.policyGhost;
    assert(policyGhost.schema === "ddn.social_world.policy_ghost_ui.v1", "schema mismatch");
    assert(policyGhost.work_item === "PA3_POLICY_GHOST_UI_V1", "work item mismatch");
    assert(policyGhost.primary_coordinate === "파-3", "coordinate mismatch");
    assert(policyGhost.depends_on_coordinate.join(",") === "파-2,파-1,타-2", "dependency mismatch");
    assert(policyGhost.pack === "social_world_econ_3_v1", "pack mismatch");
    assert(policyGhost.status === "social_world_policy_ghost_ready", "status mismatch");
    assert(policyGhost.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(policyGhost.policy_ghost_ui_claim === true, "policy ghost claim mismatch");
    assert(policyGhost.compare_run_claim === true, "compare run mismatch");
    assert(policyGhost.ghost_overlay_claim === true, "ghost overlay mismatch");
    assert(policyGhost.classroom_compare_claim === true, "classroom compare mismatch");
    assert(policyGhost.real_world_prediction_claim === false, "prediction must stay false");
    assert(policyGhost.policy_advice_claim === false, "policy advice must stay false");
    assert(policyGhost.agent_simulation_execution_claim === false, "agent simulation must stay false");
    assert(policyGhost.live_policy_deployment_claim === false, "live deployment must stay false");
    assert(policyGhost.state_hash_participation_claim === false, "state hash participation must stay false");
    assert(policyGhost.runtime_claim === false, "runtime claim must stay false");
    assert(policyGhost.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(policyGhost.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(policyGhost.progress.roadmap_v2_matrix_behavior_closed === 26, "roadmap closed mismatch");
    assert(policyGhost.progress.roadmap_v2_matrix_behavior_percent === 29, "roadmap percent mismatch");
    assert(policyGhost.progress.roadmap_v2_pack_evidence_reference_closed === 46, "pack ref mismatch");
    assert(policyGhost.progress.roadmap_v2_pack_evidence_reference_percent === 51, "pack ref percent mismatch");
    assert(policyGhost.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(policyGhost.rows.map((row) => row.id).join(",") === "baseline_run,policy_variant,ghost_overlay,classroom_compare", "row order mismatch");
    assert(String(policyGhost.ghost_text).includes("live_policy_deployment:false"), "ghost text missing deployment boundary");
    assert(String(policyGhost.ghost_text).includes("state_hash_participation:false"), "ghost text missing state hash boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t46/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-social-world-policy-ghost-ui]");
      const buttons = Array.from(document.querySelectorAll(".social-policy-btn[data-social-policy-row]"));
      buttons.find((button) => button.getAttribute("data-social-policy-row") === "ghost_overlay")?.click();
      document.querySelector("[data-social-policy-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-social-world-policy-ghost-status") || "",
        copied: root?.getAttribute("data-social-world-policy-ghost-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-social-policy-artifact]").length,
        progress: document.querySelector("[data-social-policy-progress]")?.textContent || "",
        summary: document.querySelector("[data-social-policy-summary]")?.textContent || "",
        title: document.querySelector("[data-social-policy-active-title]")?.textContent || "",
        link: document.querySelector("[data-social-policy-active-link]")?.textContent || "",
        preview: document.querySelector("[data-social-policy-preview]")?.textContent || "",
        globalSchema: window.__SOCIAL_WORLD_POLICY_GHOST_UI__?.schema || "",
        globalText: window.__SOCIAL_WORLD_POLICY_GHOST_UI_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "social_world_policy_ghost_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("26/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("real-world prediction") && domResult.summary.includes("policy advice") && domResult.summary.includes("live policy deployment"), "summary missing boundary");
    assert(domResult.title === "Ghost overlay", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("social-world://policy-ghost/overlay"), "ghost overlay URI mismatch");
    assert(domResult.preview.includes("real_world_prediction:false") && domResult.preview.includes("live_policy_deployment:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.social_world.policy_ghost_ui.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t26/90"), "global text missing roadmap matrix");
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
