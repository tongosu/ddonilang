#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "social_world_template_registry: ok";

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
    "social_world_policy_ghost_ui.js",
    "social_world_template_registry.js",
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
    await page.waitForSelector("[data-social-world-template-registry][data-social-world-template-registry-status='social_world_template_registry_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_SOCIAL_WORLD_TEMPLATE_REGISTRY_ROWS,
        buildSocialWorldTemplateRegistry,
        formatSocialWorldTemplateRegistryText,
      } = await import("./social_world_template_registry.js");
      const registry = buildSocialWorldTemplateRegistry({ rows: DEFAULT_SOCIAL_WORLD_TEMPLATE_REGISTRY_ROWS });
      return { registry, text: formatSocialWorldTemplateRegistryText(registry) };
    });
    const registry = moduleResult.registry;
    assert(registry.schema === "ddn.social_world.template_registry.v1", "schema mismatch");
    assert(registry.work_item === "PA4_SOCIAL_TEMPLATE_REGISTRY_V1", "work item mismatch");
    assert(registry.primary_coordinate === "파-4", "coordinate mismatch");
    assert(registry.depends_on_coordinate.join(",") === "파-3,파-2,타-2", "dependency mismatch");
    assert(registry.pack === "social_world_econ_4_v1", "pack mismatch");
    assert(registry.status === "social_world_template_registry_ready", "status mismatch");
    assert(registry.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(registry.social_template_registry_claim === true, "template registry claim mismatch");
    assert(registry.template_catalog_claim === true, "template catalog mismatch");
    assert(registry.share_snapshot_claim === true, "share snapshot mismatch");
    assert(registry.remix_contract_claim === true, "remix contract mismatch");
    assert(registry.classroom_registry_claim === true, "classroom registry mismatch");
    assert(registry.public_template_publish_claim === false, "public publish must stay false");
    assert(registry.network_registry_sync_claim === false, "network sync must stay false");
    assert(registry.account_permission_change_claim === false, "account permission must stay false");
    assert(registry.policy_advice_claim === false, "policy advice must stay false");
    assert(registry.state_hash_participation_claim === false, "state hash participation must stay false");
    assert(registry.runtime_claim === false, "runtime claim must stay false");
    assert(registry.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(registry.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(registry.progress.roadmap_v2_matrix_behavior_closed === 27, "roadmap closed mismatch");
    assert(registry.progress.roadmap_v2_matrix_behavior_percent === 30, "roadmap percent mismatch");
    assert(registry.progress.roadmap_v2_pack_evidence_reference_closed === 47, "pack ref mismatch");
    assert(registry.progress.roadmap_v2_pack_evidence_reference_percent === 52, "pack ref percent mismatch");
    assert(registry.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(registry.rows.map((row) => row.id).join(",") === "template_catalog,share_snapshot,remix_contract,classroom_registry", "row order mismatch");
    assert(String(registry.template_text).includes("public_template_publish:false"), "template text missing publish boundary");
    assert(String(registry.template_text).includes("network_registry_sync:false"), "template text missing network boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t47/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-social-world-template-registry]");
      const buttons = Array.from(document.querySelectorAll(".social-template-btn[data-social-template-row]"));
      buttons.find((button) => button.getAttribute("data-social-template-row") === "remix_contract")?.click();
      document.querySelector("[data-social-template-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-social-world-template-registry-status") || "",
        copied: root?.getAttribute("data-social-world-template-registry-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-social-template-artifact]").length,
        progress: document.querySelector("[data-social-template-progress]")?.textContent || "",
        summary: document.querySelector("[data-social-template-summary]")?.textContent || "",
        title: document.querySelector("[data-social-template-active-title]")?.textContent || "",
        link: document.querySelector("[data-social-template-active-link]")?.textContent || "",
        preview: document.querySelector("[data-social-template-preview]")?.textContent || "",
        globalSchema: window.__SOCIAL_WORLD_TEMPLATE_REGISTRY__?.schema || "",
        globalText: window.__SOCIAL_WORLD_TEMPLATE_REGISTRY_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "social_world_template_registry_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("27/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("public template publish") && domResult.summary.includes("network registry sync") && domResult.summary.includes("account permission change"), "summary missing boundary");
    assert(domResult.title === "Remix contract", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("social-world://template-registry/remix"), "remix URI mismatch");
    assert(domResult.preview.includes("public_template_publish:false") && domResult.preview.includes("network_registry_sync:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.social_world.template_registry.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t27/90"), "global text missing roadmap matrix");
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
