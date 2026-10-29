#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "ttonimaru_platform_hardening: ok";

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
    "ttonimaru_public_registry_seed.js",
    "ttonimaru_platform_hardening.js",
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
    await page.waitForSelector("[data-ttonimaru-platform-hardening][data-ttonimaru-platform-hardening-status='ttonimaru_platform_hardening_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TTONIMARU_PLATFORM_HARDENING_ROWS,
        buildTtonimaruPlatformHardening,
        formatTtonimaruPlatformHardeningText,
      } = await import("./ttonimaru_platform_hardening.js");
      const hardening = buildTtonimaruPlatformHardening({ rows: DEFAULT_TTONIMARU_PLATFORM_HARDENING_ROWS });
      return { hardening, text: formatTtonimaruPlatformHardeningText(hardening) };
    });
    const hardening = moduleResult.hardening;
    assert(hardening.schema === "ddn.ttonimaru.platform_hardening.v1", "schema mismatch");
    assert(hardening.work_item === "KA5_PLATFORM_HARDENING_V1", "work item mismatch");
    assert(hardening.primary_coordinate === "카-5", "coordinate mismatch");
    assert(hardening.depends_on_coordinate.join(",") === "카-4", "dependency mismatch");
    assert(hardening.status === "ttonimaru_platform_hardening_ready", "status mismatch");
    assert(hardening.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(hardening.platform_hardening_claim === true, "hardening claim mismatch");
    assert(hardening.auth_boundary_claim === true, "auth claim mismatch");
    assert(hardening.rbac_matrix_claim === true, "rbac claim mismatch");
    assert(hardening.audit_log_claim === true, "audit claim mismatch");
    assert(hardening.backup_plan_claim === true, "backup claim mismatch");
    assert(hardening.runtime_claim === false, "runtime claim must stay false");
    assert(hardening.public_registry_final_claim === false, "final claim must stay false");
    assert(hardening.registry_publish_claim === false, "publish claim must stay false");
    assert(hardening.production_deploy_claim === false, "production deploy must stay false");
    assert(hardening.cloud_account_claim === false, "cloud account must stay false");
    assert(hardening.cryptographic_signing_claim === false, "crypto signing must stay false");
    assert(hardening.production_backup_claim === false, "production backup must stay false");
    assert(hardening.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(hardening.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(hardening.progress.roadmap_v2_matrix_behavior_closed === 21, "roadmap closed mismatch");
    assert(hardening.progress.roadmap_v2_matrix_behavior_percent === 23, "roadmap percent mismatch");
    assert(hardening.progress.roadmap_v2_pack_evidence_reference_closed === 41, "pack ref mismatch");
    assert(hardening.progress.roadmap_v2_pack_evidence_reference_percent === 46, "pack ref percent mismatch");
    assert(hardening.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(hardening.hardening_rows.map((row) => row.id).join(",") === "auth_boundary,rbac_matrix,audit_log,backup_plan", "hardening row order mismatch");
    assert(String(hardening.hardening_text).includes("production_deploy:false"), "hardening text missing deploy boundary");
    assert(String(moduleResult.text).includes("platform_hardening_claim\ttrue"), "text missing hardening claim");
    assert(String(moduleResult.text).includes("cloud_account_claim\tfalse"), "text missing cloud boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-ttonimaru-platform-hardening]");
      const buttons = Array.from(document.querySelectorAll(".ttonimaru-hardening-btn[data-ttonimaru-hardening]"));
      buttons.find((button) => button.getAttribute("data-ttonimaru-hardening") === "audit_log")?.click();
      document.querySelector("[data-ttonimaru-hardening-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-ttonimaru-platform-hardening-status") || "",
        copied: root?.getAttribute("data-ttonimaru-platform-hardening-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-ttonimaru-hardening-artifact]").length,
        progress: document.querySelector("[data-ttonimaru-hardening-progress]")?.textContent || "",
        summary: document.querySelector("[data-ttonimaru-hardening-summary]")?.textContent || "",
        title: document.querySelector("[data-ttonimaru-hardening-active-title]")?.textContent || "",
        link: document.querySelector("[data-ttonimaru-hardening-active-link]")?.textContent || "",
        preview: document.querySelector("[data-ttonimaru-hardening-preview]")?.textContent || "",
        globalSchema: window.__TTONIMARU_PLATFORM_HARDENING__?.schema || "",
        globalText: window.__TTONIMARU_PLATFORM_HARDENING_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "ttonimaru_platform_hardening_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("21/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("Auth boundary") && domResult.summary.includes("cloud account") && domResult.summary.includes("cryptographic signing"), "summary missing scope boundary");
    assert(domResult.title === "Audit log", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("ttonimaru://platform/hardening/local/audit"), "audit URI mismatch");
    assert(domResult.preview.includes("platform.auth.boundary.detjson") && domResult.preview.includes("production_deploy:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.ttonimaru.platform_hardening.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t41/90"), "global text missing pack reference");
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
