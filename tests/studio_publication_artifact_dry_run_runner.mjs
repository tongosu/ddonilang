#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_publication_artifact_dry_run: ok";

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
  for (const rel of ["index.html", "app.js", "studio_publication_artifact_dry_run.js"]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1180, height: 760 }, locale: "ko-KR" });
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
    await page.waitForSelector("[data-publication-artifact-dry-run][data-publication-artifact-dry-run-status='publication_artifact_dry_run_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_PUBLICATION_ARTIFACT_DRY_RUN_ROWS,
        buildPublicationArtifactDryRun,
        formatPublicationArtifactDryRunText,
      } = await import("./studio_publication_artifact_dry_run.js");
      const dryRun = buildPublicationArtifactDryRun({
        artifactRows: DEFAULT_PUBLICATION_ARTIFACT_DRY_RUN_ROWS,
      });
      return {
        dryRun,
        text: formatPublicationArtifactDryRunText(dryRun),
      };
    });

    const dryRun = moduleResult.dryRun;
    assert(dryRun.__종류 === "studio_publication_artifact_dry_run", "kind mismatch");
    assert(dryRun.schema === "ddn.studio.publication_artifact_dry_run.v1", "schema mismatch");
    assert(dryRun.status === "publication_artifact_dry_run_ready", `status mismatch: ${dryRun.status}`);
    assert(dryRun.planned_artifact_count === 4, `artifact count mismatch: ${dryRun.planned_artifact_count}`);
    assert(dryRun.all_planned_artifacts_generated_now === false, "artifact generation aggregate mismatch");
    assert(dryRun.artifact_generation_claim === false, "must not claim artifact generation");
    assert(dryRun.archive_generation_claim === false, "must not claim archive generation");
    assert(dryRun.publication_checksum_generation_claim === false, "must not claim checksum generation");
    assert(dryRun.artifact_signing_claim === false, "must not claim signing");
    assert(dryRun.release_execution_claim === false, "must not claim release execution");
    assert(dryRun.product_ui_change === true, "must claim product ui change");
    assert(dryRun.artifact_row_count === 4, `row count mismatch: ${dryRun.artifact_row_count}`);
    assert(dryRun.ready_stage_count === 6, `ready stage mismatch: ${dryRun.ready_stage_count}`);
    assert(dryRun.checksum_policy.manifest_path === "build/studio_release/SHA256SUMS.txt", "checksum path mismatch");
    assert(dryRun.checksum_policy.signing === "excluded_v1_approval_gated", "checksum signing policy mismatch");
    assert(dryRun.progress.super_long_percent === 50, "super-long percent mismatch");
    assert(dryRun.progress.current_stage_closed === 4, "followup closed mismatch");
    assert(dryRun.progress.current_stage_percent === 50, "followup percent mismatch");
    assert(dryRun.progress.roadmap_v2_behavior_closed === 90, "roadmap closed mismatch");
    assert(dryRun.progress.roadmap_v2_percent === 100, "roadmap percent mismatch");
    assert(dryRun.next_item === "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1", "next item mismatch");
    assert(String(moduleResult.text).includes("planned_artifact_count\t4"), "formatted text missing artifact count");
    assert(String(moduleResult.text).includes("artifact_signing_claim\tfalse"), "formatted text missing signing boundary");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-publication-artifact-dry-run]");
      const buttons = Array.from(document.querySelectorAll("[data-artifact-dry-run]"));
      const progress = document.querySelector("[data-artifact-dry-run-progress]")?.textContent || "";
      const summary = document.querySelector("[data-artifact-dry-run-summary]")?.textContent || "";
      buttons.find((button) => button.getAttribute("data-artifact-dry-run") === "studio-checksum-manifest")?.click();
      const title = document.querySelector("[data-artifact-dry-run-active-title]")?.textContent || "";
      const kind = document.querySelector("[data-artifact-dry-run-active-kind]")?.textContent || "";
      const pathText = document.querySelector("[data-artifact-dry-run-active-path]")?.textContent || "";
      document.querySelector("[data-artifact-dry-run-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-publication-artifact-dry-run-status") || "",
        copied: root?.getAttribute("data-publication-artifact-dry-run-copied") || "",
        buttonCount: buttons.length,
        progress,
        summary,
        title,
        kind,
        pathText,
        globalSchema: window.__SEAMGRIM_PUBLICATION_ARTIFACT_DRY_RUN__?.schema || "",
        globalText: window.__SEAMGRIM_PUBLICATION_ARTIFACT_DRY_RUN_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "publication_artifact_dry_run_ready", `dom status mismatch: ${domResult.rootStatus}`);
    assert(domResult.buttonCount === 4, `dom button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("4/8 follow-up") && domResult.progress.includes("50%"), `progress text mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("4 artifacts") && domResult.summary.includes("generated_now=false"), `summary text mismatch: ${domResult.summary}`);
    assert(domResult.title === "checksum manifest", `title mismatch: ${domResult.title}`);
    assert(domResult.kind === "checksum_manifest", `kind mismatch: ${domResult.kind}`);
    assert(domResult.pathText === "build/studio_release/SHA256SUMS.txt", `path mismatch: ${domResult.pathText}`);
    assert(domResult.copied === "true", "copy state not marked");
    assert(domResult.globalSchema === "ddn.studio.publication_artifact_dry_run.v1", "global schema mismatch");
    assert(String(domResult.globalText).includes("artifact_id\tkind\tgenerated_now\tplanned_path"), "global text missing header");

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
