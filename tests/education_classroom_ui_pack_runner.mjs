#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "education_classroom_ui_pack: ok";

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
    "education_classroom_ui_pack.js",
    "education_classroom_ui_pack.js",
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
    await page.waitForSelector("[data-education-classroom-ui-pack][data-education-classroom-ui-pack-status='education_classroom_ui_pack_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_EDUCATION_CLASSROOM_UI_ROWS,
        buildEducationClassroomUiPack,
        formatEducationClassroomUiPackText,
      } = await import("./education_classroom_ui_pack.js");
      const classroom = buildEducationClassroomUiPack({ rows: DEFAULT_EDUCATION_CLASSROOM_UI_ROWS });
      return { classroom, text: formatEducationClassroomUiPackText(classroom) };
    });
    const classroom = moduleResult.classroom;
    assert(classroom.schema === "ddn.education.classroom_ui_pack.v1", "schema mismatch");
    assert(classroom.work_item === "HA3_CLASSROOM_UI_PACK_V1", "work item mismatch");
    assert(classroom.primary_coordinate === "하-3", "coordinate mismatch");
    assert(classroom.depends_on_coordinate.join(",") === "하-2,마-3,타-3", "dependency mismatch");
    assert(classroom.pack === "education_curriculum_3_v1", "pack mismatch");
    assert(classroom.status === "education_classroom_ui_pack_ready", "status mismatch");
    assert(classroom.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(classroom.education_classroom_ui_pack_claim === true, "classroom pack claim mismatch");
    assert(classroom.teacher_mode_claim === true, "teacher notes mismatch");
    assert(classroom.student_mode_claim === true, "student sheet mismatch");
    assert(classroom.assignment_panel_claim === true, "rubric mapping mismatch");
    assert(classroom.report_review_claim === true, "sample submission mismatch");
    assert(classroom.classroom_handoff_claim === true, "pack handoff mismatch");
    assert(classroom.gradebook_write_claim === false, "gradebook write must stay false");
    assert(classroom.student_personal_data_collection_claim === false, "student data collection must stay false");
    assert(classroom.remote_classroom_sync_claim === false, "remote classroom sync must stay false");
    assert(classroom.live_submission_claim === false, "live submission must stay false");
    assert(classroom.account_permission_change_claim === false, "account permission must stay false");
    assert(classroom.state_hash_participation_claim === false, "state hash participation must stay false");
    assert(classroom.runtime_claim === false, "runtime claim must stay false");
    assert(classroom.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(classroom.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(classroom.progress.roadmap_v2_matrix_behavior_closed === 30, "roadmap closed mismatch");
    assert(classroom.progress.roadmap_v2_matrix_behavior_percent === 33, "roadmap percent mismatch");
    assert(classroom.progress.roadmap_v2_pack_evidence_reference_closed === 50, "pack ref mismatch");
    assert(classroom.progress.roadmap_v2_pack_evidence_reference_percent === 56, "pack ref percent mismatch");
    assert(classroom.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(classroom.rows.map((row) => row.id).join(",") === "teacher_mode,student_mode,assignment_panel,report_review,classroom_handoff", "row order mismatch");
    assert(String(classroom.classroom_text).includes("gradebook_write:false"), "classroom text missing gradebook boundary");
    assert(String(classroom.classroom_text).includes("student_personal_data_collection:false"), "classroom text missing data boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t50/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-education-classroom-ui-pack]");
      const buttons = Array.from(document.querySelectorAll(".education-classroom-btn[data-education-classroom-row]"));
      buttons.find((button) => button.getAttribute("data-education-classroom-row") === "student_mode")?.click();
      document.querySelector("[data-education-classroom-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-education-classroom-ui-pack-status") || "",
        copied: root?.getAttribute("data-education-classroom-ui-pack-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-education-classroom-artifact]").length,
        progress: document.querySelector("[data-education-classroom-progress]")?.textContent || "",
        summary: document.querySelector("[data-education-classroom-summary]")?.textContent || "",
        title: document.querySelector("[data-education-classroom-active-title]")?.textContent || "",
        link: document.querySelector("[data-education-classroom-active-link]")?.textContent || "",
        preview: document.querySelector("[data-education-classroom-preview]")?.textContent || "",
        globalSchema: window.__EDUCATION_CLASSROOM_UI_PACK__?.schema || "",
        globalText: window.__EDUCATION_CLASSROOM_UI_PACK_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "education_classroom_ui_pack_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("30/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("gradebook write") && domResult.summary.includes("student personal data collection") && domResult.summary.includes("remote classroom sync"), "summary missing boundary");
    assert(domResult.title === "Student mode", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("education://classroom/student-mode"), "rubric URI mismatch");
    assert(domResult.preview.includes("gradebook_write:false") && domResult.preview.includes("remote_classroom_sync:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.education.classroom_ui_pack.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t30/90"), "global text missing roadmap matrix");
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
