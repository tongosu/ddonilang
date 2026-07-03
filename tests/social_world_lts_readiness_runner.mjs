#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "social_world_lts_readiness: ok";

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
    "social_world_template_registry.js",
    "social_world_lts_readiness.js",
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
    await page.waitForSelector("[data-social-world-lts-readiness][data-social-world-lts-readiness-status='social_world_lts_readiness_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_SOCIAL_WORLD_LTS_ROWS,
        buildSocialWorldLtsReadiness,
        formatSocialWorldLtsReadinessText,
      } = await import("./social_world_lts_readiness.js");
      const lts = buildSocialWorldLtsReadiness({ rows: DEFAULT_SOCIAL_WORLD_LTS_ROWS });
      return { lts, text: formatSocialWorldLtsReadinessText(lts) };
    });
    const lts = moduleResult.lts;
    assert(lts.schema === "ddn.social_world.lts_readiness.v1", "schema mismatch");
    assert(lts.work_item === "PA5_SOCIAL_WORLD_LTS_V1", "work item mismatch");
    assert(lts.primary_coordinate === "파-5", "coordinate mismatch");
    assert(lts.depends_on_coordinate.join(",") === "파-4,파-3,타-2", "dependency mismatch");
    assert(lts.pack === "social_world_econ_5_v1", "pack mismatch");
    assert(lts.status === "social_world_lts_readiness_ready", "status mismatch");
    assert(lts.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(lts.social_world_lts_claim === true, "social world LTS claim mismatch");
    assert(lts.education_stability_claim === true, "education stability mismatch");
    assert(lts.policy_regression_claim === true, "policy regression mismatch");
    assert(lts.history_fixture_claim === true, "history fixture mismatch");
    assert(lts.lts_gate_claim === true, "LTS gate mismatch");
    assert(lts.remote_lts_certification_claim === false, "remote LTS certification must stay false");
    assert(lts.public_release_execution_claim === false, "public release execution must stay false");
    assert(lts.live_policy_deployment_claim === false, "live policy deployment must stay false");
    assert(lts.real_world_prediction_claim === false, "real world prediction must stay false");
    assert(lts.policy_advice_claim === false, "policy advice must stay false");
    assert(lts.state_hash_participation_claim === false, "state hash participation must stay false");
    assert(lts.runtime_claim === false, "runtime claim must stay false");
    assert(lts.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(lts.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(lts.progress.roadmap_v2_matrix_behavior_closed === 28, "roadmap closed mismatch");
    assert(lts.progress.roadmap_v2_matrix_behavior_percent === 31, "roadmap percent mismatch");
    assert(lts.progress.roadmap_v2_pack_evidence_reference_closed === 48, "pack ref mismatch");
    assert(lts.progress.roadmap_v2_pack_evidence_reference_percent === 53, "pack ref percent mismatch");
    assert(lts.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(lts.rows.map((row) => row.id).join(",") === "education_stability,policy_regression,history_fixture,lts_gate", "row order mismatch");
    assert(String(lts.lts_text).includes("remote_lts_certification:false"), "LTS text missing remote boundary");
    assert(String(lts.lts_text).includes("public_release_execution:false"), "LTS text missing public release boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t48/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-social-world-lts-readiness]");
      const buttons = Array.from(document.querySelectorAll(".social-lts-btn[data-social-lts-row]"));
      buttons.find((button) => button.getAttribute("data-social-lts-row") === "policy_regression")?.click();
      document.querySelector("[data-social-lts-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-social-world-lts-readiness-status") || "",
        copied: root?.getAttribute("data-social-world-lts-readiness-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-social-lts-artifact]").length,
        progress: document.querySelector("[data-social-lts-progress]")?.textContent || "",
        summary: document.querySelector("[data-social-lts-summary]")?.textContent || "",
        title: document.querySelector("[data-social-lts-active-title]")?.textContent || "",
        link: document.querySelector("[data-social-lts-active-link]")?.textContent || "",
        preview: document.querySelector("[data-social-lts-preview]")?.textContent || "",
        globalSchema: window.__SOCIAL_WORLD_LTS_READINESS__?.schema || "",
        globalText: window.__SOCIAL_WORLD_LTS_READINESS_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "social_world_lts_readiness_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("28/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("remote LTS certification") && domResult.summary.includes("public release execution") && domResult.summary.includes("live policy deployment"), "summary missing boundary");
    assert(domResult.title === "Policy regression", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("social-world://lts/policy-regression"), "policy regression URI mismatch");
    assert(domResult.preview.includes("remote_lts_certification:false") && domResult.preview.includes("live_policy_deployment:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.social_world.lts_readiness.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t28/90"), "global text missing roadmap matrix");
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
