#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "toolchain_registry_verification: ok";

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
    "toolchain_diagnostic_ui_lsp.js",
    "toolchain_registry_verification.js",
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
    await page.waitForSelector("[data-toolchain-registry-verification][data-toolchain-registry-verification-status='toolchain_registry_verification_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TOOLCHAIN_REGISTRY_VERIFICATION_ROWS,
        buildToolchainRegistryVerification,
        formatToolchainRegistryVerificationText,
      } = await import("./toolchain_registry_verification.js");
      const registry = buildToolchainRegistryVerification({ rows: DEFAULT_TOOLCHAIN_REGISTRY_VERIFICATION_ROWS });
      return { registry, text: formatToolchainRegistryVerificationText(registry) };
    });
    const registry = moduleResult.registry;
    assert(registry.schema === "ddn.toolchain.registry_verification.v1", "schema mismatch");
    assert(registry.work_item === "TA4_REGISTRY_VERIFICATION_V1", "work item mismatch");
    assert(registry.primary_coordinate === "타-4", "coordinate mismatch");
    assert(registry.depends_on_coordinate.join(",") === "타-3,타-2", "dependency mismatch");
    assert(registry.pack === "toolchain_pack_4_v1", "pack mismatch");
    assert(registry.status === "toolchain_registry_verification_ready", "status mismatch");
    assert(registry.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(registry.registry_verification_claim === true, "registry verification mismatch");
    assert(registry.publish_dry_run_claim === true, "publish dry-run mismatch");
    assert(registry.install_plan_claim === true, "install plan mismatch");
    assert(registry.digest_verify_claim === true, "digest verify mismatch");
    assert(registry.rollback_probe_claim === true, "rollback probe mismatch");
    assert(registry.public_registry_publish_claim === false, "public publish must stay false");
    assert(registry.install_execution_claim === false, "install execution must stay false");
    assert(registry.network_io_claim === false, "network IO must stay false");
    assert(registry.trust_signing_claim === false, "trust signing must stay false");
    assert(registry.runtime_claim === false, "runtime claim must stay false");
    assert(registry.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(registry.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(registry.progress.roadmap_v2_matrix_behavior_closed === 23, "roadmap closed mismatch");
    assert(registry.progress.roadmap_v2_matrix_behavior_percent === 26, "roadmap percent mismatch");
    assert(registry.progress.roadmap_v2_pack_evidence_reference_closed === 43, "pack ref mismatch");
    assert(registry.progress.roadmap_v2_pack_evidence_reference_percent === 48, "pack ref percent mismatch");
    assert(registry.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(registry.rows.map((row) => row.id).join(",") === "publish_manifest,install_plan,digest_verify,rollback_probe", "row order mismatch");
    assert(String(registry.verification_text).includes("network_io:false"), "verification text missing network boundary");
    assert(String(moduleResult.text).includes("public_registry_publish_claim\tfalse"), "text missing publish boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t43/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-toolchain-registry-verification]");
      const buttons = Array.from(document.querySelectorAll(".toolchain-registry-btn[data-toolchain-registry-row]"));
      buttons.find((button) => button.getAttribute("data-toolchain-registry-row") === "digest_verify")?.click();
      document.querySelector("[data-toolchain-registry-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-toolchain-registry-verification-status") || "",
        copied: root?.getAttribute("data-toolchain-registry-verification-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-toolchain-registry-artifact]").length,
        progress: document.querySelector("[data-toolchain-registry-progress]")?.textContent || "",
        summary: document.querySelector("[data-toolchain-registry-summary]")?.textContent || "",
        title: document.querySelector("[data-toolchain-registry-active-title]")?.textContent || "",
        link: document.querySelector("[data-toolchain-registry-active-link]")?.textContent || "",
        preview: document.querySelector("[data-toolchain-registry-preview]")?.textContent || "",
        globalSchema: window.__TOOLCHAIN_REGISTRY_VERIFICATION__?.schema || "",
        globalText: window.__TOOLCHAIN_REGISTRY_VERIFICATION_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "toolchain_registry_verification_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("23/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("public registry publish") && domResult.summary.includes("install execution") && domResult.summary.includes("network IO"), "summary missing boundary");
    assert(domResult.title === "Digest verify", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("toolchain://registry/verify/digest"), "digest URI mismatch");
    assert(domResult.preview.includes("public_registry_publish:false") && domResult.preview.includes("trust_signing:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.toolchain.registry_verification.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t23/90"), "global text missing roadmap matrix");
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
