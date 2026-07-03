#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "ttonimaru_public_registry_seed: ok";

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
    "ttonimaru_project_share_ui.js",
    "ttonimaru_public_registry_seed.js",
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
    await page.waitForSelector("[data-ttonimaru-public-registry-seed][data-ttonimaru-public-registry-seed-status='ttonimaru_public_registry_seed_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TTONIMARU_PUBLIC_REGISTRY_SEED_ROWS,
        buildTtonimaruPublicRegistrySeed,
        formatTtonimaruPublicRegistrySeedText,
      } = await import("./ttonimaru_public_registry_seed.js");
      const registrySeed = buildTtonimaruPublicRegistrySeed({ rows: DEFAULT_TTONIMARU_PUBLIC_REGISTRY_SEED_ROWS });
      return {
        registrySeed,
        text: formatTtonimaruPublicRegistrySeedText(registrySeed),
      };
    });
    const seed = moduleResult.registrySeed;
    assert(seed.schema === "ddn.ttonimaru.public_registry_seed.v1", "schema mismatch");
    assert(seed.work_item === "KA4_PUBLIC_REGISTRY_SEED_V1", "work item mismatch");
    assert(seed.primary_coordinate === "카-4", "coordinate mismatch");
    assert(seed.depends_on_coordinate.join(",") === "카-3", "dependency mismatch");
    assert(seed.status === "ttonimaru_public_registry_seed_ready", "status mismatch");
    assert(seed.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(seed.public_registry_seed_claim === true, "public registry seed claim mismatch");
    assert(seed.seed_catalog_claim === true, "seed catalog claim mismatch");
    assert(seed.lineage_record_claim === true, "lineage claim mismatch");
    assert(seed.trust_badge_claim === true, "trust badge claim mismatch");
    assert(seed.registry_preview_claim === true, "registry preview claim mismatch");
    assert(seed.product_ui_change === true, "product UI change mismatch");
    assert(seed.runtime_claim === false, "runtime claim must stay false");
    assert(seed.public_registry_final_claim === false, "public registry final must stay false");
    assert(seed.registry_publish_claim === false, "registry publish must stay false");
    assert(seed.install_update_remove_claim === false, "install/update/remove must stay false");
    assert(seed.trust_signing_claim === false, "trust signing must stay false");
    assert(seed.moderation_claim === false, "moderation must stay false");
    assert(seed.team_membership_claim === false, "team membership must stay false");
    assert(seed.account_permission_claim === false, "account permission must stay false");
    assert(seed.cloud_sync_claim === false, "cloud sync must stay false");
    assert(seed.platform_hardening_claim === false, "platform hardening must stay false");
    assert(seed.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(seed.progress.current_stage_total === 5, "stage total mismatch");
    assert(seed.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(seed.progress.roadmap_v2_matrix_behavior_closed === 20, "roadmap closed mismatch");
    assert(seed.progress.roadmap_v2_matrix_behavior_percent === 22, "roadmap percent mismatch");
    assert(seed.progress.roadmap_v2_pack_evidence_reference_closed === 40, "pack ref mismatch");
    assert(seed.progress.roadmap_v2_pack_evidence_reference_percent === 44, "pack ref percent mismatch");
    assert(seed.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(seed.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(seed.registry_rows.map((row) => row.id).join(",") === "seed_catalog,lineage_record,trust_badge,seed_preview", "registry row order mismatch");
    assert(seed.artifacts.map((file) => file.kind).join(",") === "seed_catalog,lineage_record,trust_badge,seed_preview", "artifact order mismatch");
    assert(String(seed.registry_text).includes("registry_publish:false"), "registry text missing publish boundary");
    assert(String(moduleResult.text).includes("public_registry_seed_claim\ttrue"), "text missing seed claim");
    assert(String(moduleResult.text).includes("trust_signing_claim\tfalse"), "text missing signing boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t20/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-ttonimaru-public-registry-seed]");
      const buttons = Array.from(document.querySelectorAll(".ttonimaru-registry-btn[data-ttonimaru-registry]"));
      buttons.find((button) => button.getAttribute("data-ttonimaru-registry") === "trust_badge")?.click();
      document.querySelector("[data-ttonimaru-registry-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-ttonimaru-public-registry-seed-status") || "",
        copied: root?.getAttribute("data-ttonimaru-public-registry-seed-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-ttonimaru-registry-artifact]").length,
        progress: document.querySelector("[data-ttonimaru-registry-progress]")?.textContent || "",
        summary: document.querySelector("[data-ttonimaru-registry-summary]")?.textContent || "",
        title: document.querySelector("[data-ttonimaru-registry-active-title]")?.textContent || "",
        link: document.querySelector("[data-ttonimaru-registry-active-link]")?.textContent || "",
        preview: document.querySelector("[data-ttonimaru-registry-preview]")?.textContent || "",
        globalSchema: window.__TTONIMARU_PUBLIC_REGISTRY_SEED__?.schema || "",
        globalText: window.__TTONIMARU_PUBLIC_REGISTRY_SEED_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "ttonimaru_public_registry_seed_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `registry row count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("20/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("Curated seed catalog") && domResult.summary.includes("trust signing") && domResult.summary.includes("cloud sync"), "summary missing scope boundary");
    assert(domResult.title === "Trust badge", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("ttonimaru://registry/seed/local/trust-badge"), "trust badge URI mismatch");
    assert(domResult.preview.includes("registry.seed.catalog.detjson") && domResult.preview.includes("registry_publish:false"), "registry preview mismatch");
    assert(domResult.globalSchema === "ddn.ttonimaru.public_registry_seed.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t40/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("moderation_claim\tfalse"), "global text missing moderation boundary");

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
