#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "education_assessment_pack: ok";

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
    "social_world_lts_readiness.js",
    "education_assessment_pack.js",
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
    await page.waitForSelector("[data-education-assessment-pack][data-education-assessment-pack-status='education_assessment_pack_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_EDUCATION_ASSESSMENT_ROWS,
        buildEducationAssessmentPack,
        formatEducationAssessmentPackText,
      } = await import("./education_assessment_pack.js");
      const assessment = buildEducationAssessmentPack({ rows: DEFAULT_EDUCATION_ASSESSMENT_ROWS });
      return { assessment, text: formatEducationAssessmentPackText(assessment) };
    });
    const assessment = moduleResult.assessment;
    assert(assessment.schema === "ddn.education.assessment_pack.v1", "schema mismatch");
    assert(assessment.work_item === "HA2_EDUCATION_ASSESSMENT_PACK_V1", "work item mismatch");
    assert(assessment.primary_coordinate === "하-2", "coordinate mismatch");
    assert(assessment.depends_on_coordinate.join(",") === "타-2,하-1,마-2", "dependency mismatch");
    assert(assessment.pack === "education_curriculum_2_v1", "pack mismatch");
    assert(assessment.status === "education_assessment_pack_ready", "status mismatch");
    assert(assessment.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(assessment.education_assessment_pack_claim === true, "assessment pack claim mismatch");
    assert(assessment.teacher_notes_claim === true, "teacher notes mismatch");
    assert(assessment.student_sheet_claim === true, "student sheet mismatch");
    assert(assessment.rubric_mapping_claim === true, "rubric mapping mismatch");
    assert(assessment.sample_submission_claim === true, "sample submission mismatch");
    assert(assessment.pack_handoff_claim === true, "pack handoff mismatch");
    assert(assessment.gradebook_write_claim === false, "gradebook write must stay false");
    assert(assessment.student_personal_data_collection_claim === false, "student data collection must stay false");
    assert(assessment.remote_classroom_sync_claim === false, "remote classroom sync must stay false");
    assert(assessment.live_submission_claim === false, "live submission must stay false");
    assert(assessment.account_permission_change_claim === false, "account permission must stay false");
    assert(assessment.state_hash_participation_claim === false, "state hash participation must stay false");
    assert(assessment.runtime_claim === false, "runtime claim must stay false");
    assert(assessment.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(assessment.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(assessment.progress.roadmap_v2_matrix_behavior_closed === 29, "roadmap closed mismatch");
    assert(assessment.progress.roadmap_v2_matrix_behavior_percent === 32, "roadmap percent mismatch");
    assert(assessment.progress.roadmap_v2_pack_evidence_reference_closed === 49, "pack ref mismatch");
    assert(assessment.progress.roadmap_v2_pack_evidence_reference_percent === 54, "pack ref percent mismatch");
    assert(assessment.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(assessment.rows.map((row) => row.id).join(",") === "teacher_notes,student_sheet,rubric_mapping,sample_submission,pack_handoff", "row order mismatch");
    assert(String(assessment.assessment_text).includes("gradebook_write:false"), "assessment text missing gradebook boundary");
    assert(String(assessment.assessment_text).includes("student_personal_data_collection:false"), "assessment text missing data boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t49/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-education-assessment-pack]");
      const buttons = Array.from(document.querySelectorAll(".education-assessment-btn[data-education-assessment-row]"));
      buttons.find((button) => button.getAttribute("data-education-assessment-row") === "rubric_mapping")?.click();
      document.querySelector("[data-education-assessment-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-education-assessment-pack-status") || "",
        copied: root?.getAttribute("data-education-assessment-pack-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-education-assessment-artifact]").length,
        progress: document.querySelector("[data-education-assessment-progress]")?.textContent || "",
        summary: document.querySelector("[data-education-assessment-summary]")?.textContent || "",
        title: document.querySelector("[data-education-assessment-active-title]")?.textContent || "",
        link: document.querySelector("[data-education-assessment-active-link]")?.textContent || "",
        preview: document.querySelector("[data-education-assessment-preview]")?.textContent || "",
        globalSchema: window.__EDUCATION_ASSESSMENT_PACK__?.schema || "",
        globalText: window.__EDUCATION_ASSESSMENT_PACK_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "education_assessment_pack_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("29/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("gradebook write") && domResult.summary.includes("student personal data collection") && domResult.summary.includes("remote classroom sync"), "summary missing boundary");
    assert(domResult.title === "Rubric mapping", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("education://assessment/rubric-mapping"), "rubric URI mismatch");
    assert(domResult.preview.includes("gradebook_write:false") && domResult.preview.includes("remote_classroom_sync:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.education.assessment_pack.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t29/90"), "global text missing roadmap matrix");
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
