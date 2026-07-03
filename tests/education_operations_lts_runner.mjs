#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "education_operations_lts: ok";

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
    "education_publication_pack.js",
    "education_operations_lts.js",
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
    await page.waitForSelector("[data-education-operations-lts][data-education-operations-lts-status='education_operations_lts_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_EDUCATION_OPERATIONS_LTS_ROWS,
        buildEducationOperationsLts,
        formatEducationOperationsLtsText,
      } = await import("./education_operations_lts.js");
      const operations = buildEducationOperationsLts({ rows: DEFAULT_EDUCATION_OPERATIONS_LTS_ROWS });
      return { operations, text: formatEducationOperationsLtsText(operations) };
    });
    const operations = moduleResult.operations;
    assert(operations.schema === "ddn.education.operations_lts.v1", "schema mismatch");
    assert(operations.work_item === "HA5_EDUCATION_OPERATIONS_LTS_V1", "work item mismatch");
    assert(operations.primary_coordinate === "하-5", "coordinate mismatch");
    assert(operations.depends_on_coordinate.join(",") === "하-4,마-5,타-5", "dependency mismatch");
    assert(operations.pack === "education_curriculum_5_v1", "pack mismatch");
    assert(operations.status === "education_operations_lts_ready", "status mismatch");
    assert(operations.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(operations.education_operations_lts_claim === true, "operations LTS claim mismatch");
    assert(operations.submission_versioning_claim === true, "submission versioning mismatch");
    assert(operations.assessment_archive_claim === true, "assessment archive mismatch");
    assert(operations.curriculum_version_lock_claim === true, "curriculum lock mismatch");
    assert(operations.lts_gate_claim === true, "LTS gate mismatch");
    assert(operations.operations_handoff_claim === true, "operations handoff mismatch");
    assert(operations.remote_lts_certification_claim === false, "remote LTS must stay false");
    assert(operations.live_submission_claim === false, "live submission must stay false");
    assert(operations.gradebook_write_claim === false, "gradebook write must stay false");
    assert(operations.student_personal_data_collection_claim === false, "student PII must stay false");
    assert(operations.remote_classroom_sync_claim === false, "remote classroom sync must stay false");
    assert(operations.release_execution_claim === false, "release execution must stay false");
    assert(operations.registry_publish_claim === false, "registry publish must stay false");
    assert(operations.account_permission_change_claim === false, "account permission must stay false");
    assert(operations.state_hash_participation_claim === false, "state hash participation must stay false");
    assert(operations.runtime_claim === false, "runtime claim must stay false");
    assert(operations.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(operations.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(operations.progress.roadmap_v2_matrix_behavior_closed === 32, "roadmap closed mismatch");
    assert(operations.progress.roadmap_v2_matrix_behavior_percent === 36, "roadmap percent mismatch");
    assert(operations.progress.roadmap_v2_pack_evidence_reference_closed === 52, "pack ref mismatch");
    assert(operations.progress.roadmap_v2_pack_evidence_reference_percent === 58, "pack ref percent mismatch");
    assert(operations.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(operations.rows.map((row) => row.id).join(",") === "submission_versioning,assessment_archive,curriculum_version_lock,lts_gate,operations_handoff", "row order mismatch");
    assert(String(operations.operations_text).includes("remote_lts_certification:false"), "operations text missing remote LTS boundary");
    assert(String(operations.operations_text).includes("gradebook_write:false"), "operations text missing gradebook boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t52/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-education-operations-lts]");
      const buttons = Array.from(document.querySelectorAll(".education-operations-btn[data-education-operations-row]"));
      buttons.find((button) => button.getAttribute("data-education-operations-row") === "lts_gate")?.click();
      document.querySelector("[data-education-operations-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-education-operations-lts-status") || "",
        copied: root?.getAttribute("data-education-operations-lts-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-education-operations-artifact]").length,
        progress: document.querySelector("[data-education-operations-progress]")?.textContent || "",
        summary: document.querySelector("[data-education-operations-summary]")?.textContent || "",
        title: document.querySelector("[data-education-operations-active-title]")?.textContent || "",
        link: document.querySelector("[data-education-operations-active-link]")?.textContent || "",
        preview: document.querySelector("[data-education-operations-preview]")?.textContent || "",
        globalSchema: window.__EDUCATION_OPERATIONS_LTS__?.schema || "",
        globalText: window.__EDUCATION_OPERATIONS_LTS_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "education_operations_lts_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("32/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("remote LTS certification") && domResult.summary.includes("live submission") && domResult.summary.includes("gradebook write"), "summary missing boundary");
    assert(domResult.title === "LTS gate", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("education://operations/lts-gate"), "LTS gate URI mismatch");
    assert(domResult.preview.includes("remote_lts_certification:false") && domResult.preview.includes("release_execution:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.education.operations_lts.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t32/90"), "global text missing roadmap matrix");
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
