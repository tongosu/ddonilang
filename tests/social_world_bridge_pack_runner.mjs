#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "social_world_bridge_pack: ok";

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
    await page.waitForSelector("[data-social-world-bridge-pack][data-social-world-bridge-pack-status='social_world_bridge_pack_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_SOCIAL_WORLD_BRIDGE_ROWS,
        buildSocialWorldBridgePack,
        formatSocialWorldBridgePackText,
      } = await import("./social_world_bridge_pack.js");
      const bridge = buildSocialWorldBridgePack({ rows: DEFAULT_SOCIAL_WORLD_BRIDGE_ROWS });
      return { bridge, text: formatSocialWorldBridgePackText(bridge) };
    });
    const bridge = moduleResult.bridge;
    assert(bridge.schema === "ddn.social_world.bridge_pack.v1", "schema mismatch");
    assert(bridge.work_item === "PA2_SOCIAL_BRIDGE_PACK_V1", "work item mismatch");
    assert(bridge.primary_coordinate === "파-2", "coordinate mismatch");
    assert(bridge.depends_on_coordinate.join(",") === "파-1,파-0,타-2", "dependency mismatch");
    assert(bridge.pack === "social_world_econ_2_v1", "pack mismatch");
    assert(bridge.status === "social_world_bridge_pack_ready", "status mismatch");
    assert(bridge.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(bridge.social_bridge_pack_claim === true, "social bridge claim mismatch");
    assert(bridge.model_first_claim === true, "model-first mismatch");
    assert(bridge.agent_first_claim === true, "agent-first mismatch");
    assert(bridge.bridge_report_claim === true, "bridge report mismatch");
    assert(bridge.policy_handoff_claim === true, "policy handoff mismatch");
    assert(bridge.real_world_prediction_claim === false, "prediction must stay false");
    assert(bridge.policy_advice_claim === false, "policy advice must stay false");
    assert(bridge.agent_simulation_execution_claim === false, "agent simulation must stay false");
    assert(bridge.new_economic_theory_claim === false, "new theory must stay false");
    assert(bridge.state_hash_participation_claim === false, "state hash participation must stay false");
    assert(bridge.runtime_claim === false, "runtime claim must stay false");
    assert(bridge.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(bridge.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(bridge.progress.roadmap_v2_matrix_behavior_closed === 25, "roadmap closed mismatch");
    assert(bridge.progress.roadmap_v2_matrix_behavior_percent === 28, "roadmap percent mismatch");
    assert(bridge.progress.roadmap_v2_pack_evidence_reference_closed === 45, "pack ref mismatch");
    assert(bridge.progress.roadmap_v2_pack_evidence_reference_percent === 50, "pack ref percent mismatch");
    assert(bridge.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(bridge.rows.map((row) => row.id).join(",") === "model_first,agent_first,bridge_report,policy_handoff", "row order mismatch");
    assert(String(bridge.bridge_text).includes("real_world_prediction:false"), "bridge text missing prediction boundary");
    assert(String(bridge.bridge_text).includes("state_hash_participation:false"), "bridge text missing state hash boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t45/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-social-world-bridge-pack]");
      const buttons = Array.from(document.querySelectorAll(".social-bridge-btn[data-social-bridge-row]"));
      buttons.find((button) => button.getAttribute("data-social-bridge-row") === "bridge_report")?.click();
      document.querySelector("[data-social-bridge-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-social-world-bridge-pack-status") || "",
        copied: root?.getAttribute("data-social-world-bridge-pack-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-social-bridge-artifact]").length,
        progress: document.querySelector("[data-social-bridge-progress]")?.textContent || "",
        summary: document.querySelector("[data-social-bridge-summary]")?.textContent || "",
        title: document.querySelector("[data-social-bridge-active-title]")?.textContent || "",
        link: document.querySelector("[data-social-bridge-active-link]")?.textContent || "",
        preview: document.querySelector("[data-social-bridge-preview]")?.textContent || "",
        globalSchema: window.__SOCIAL_WORLD_BRIDGE_PACK__?.schema || "",
        globalText: window.__SOCIAL_WORLD_BRIDGE_PACK_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "social_world_bridge_pack_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("25/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("real-world prediction") && domResult.summary.includes("policy advice") && domResult.summary.includes("agent simulation execution"), "summary missing boundary");
    assert(domResult.title === "Bridge report", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("social-world://bridge/report"), "bridge report URI mismatch");
    assert(domResult.preview.includes("real_world_prediction:false") && domResult.preview.includes("state_hash_participation:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.social_world.bridge_pack.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t25/90"), "global text missing roadmap matrix");
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
