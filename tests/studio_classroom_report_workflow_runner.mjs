#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_classroom_report_workflow: ok";

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
    return url.pathname === "/api/lessons/inventory" || url.pathname === "/api/lesson-inventory";
  } catch (_) {
    return false;
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of ["index.html", "studio_classroom_mode.js"]) {
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    const result = await page.evaluate(async () => {
      const {
        buildClassroomReportWorkflow,
        formatClassroomReportWorkflowText,
      } = await import("./studio_classroom_mode.js");
      const assignments = [
        {
          assignment_id: "a01",
          title: "수치 근 찾기 보고",
          lesson_id: "rep_math_function_line_v1",
          due_label: "오늘",
          status: "open",
        },
        {
          assignment_id: "a02",
          title: "물리 시간 전개 보고",
          lesson_id: "rep_phys_projectile_xy_v1",
          due_label: "내일",
          status: "closed",
        },
      ];
      const runResults = {
        a01: { assignment_id: "a01", exit_code: 0, stdout: ["ok"], stderr: [] },
        a02: { assignment_id: "a02", exit_code: 0, stdout: ["ok"], stderr: [] },
      };
      const suiteChecks = {
        a01: {
          assignment_id: "a01",
          __이음관계종류: "endpoint_solve_range_case_suite_check",
          판정: "실패",
          전체통과: false,
          개수: 3,
          통과개수: 2,
          실패개수: 1,
          실패케이스들: ["root_out_of_range"],
          기대실패통과케이스들: ["unexpected_success"],
          기대통과실패케이스들: [],
        },
        a02: {
          assignment_id: "a02",
          __이음관계종류: "endpoint_solve_range_case_suite_check",
          판정: "통과",
          전체통과: true,
          개수: 1,
          통과개수: 1,
          실패개수: 0,
          실패케이스들: [],
          기대실패통과케이스들: [],
          기대통과실패케이스들: [],
        },
      };
      const workflow = buildClassroomReportWorkflow({ assignments, runResults, suiteChecks });
      return {
        workflow,
        text: formatClassroomReportWorkflowText(workflow),
      };
    });

    const workflow = result.workflow;
    assert(workflow.__종류 === "studio_classroom_report_workflow", "workflow kind mismatch");
    assert(workflow.schema === "seamgrim.classroom_report_workflow.v1", "workflow schema mismatch");
    assert(workflow.primary_coordinate === "마-3", `primary coordinate mismatch: ${workflow.primary_coordinate}`);
    assert(workflow.support_coordinate === "하-3", `support coordinate mismatch: ${workflow.support_coordinate}`);
    assert(workflow.workflow_claim === "classroom_report_workflow", `workflow claim mismatch: ${workflow.workflow_claim}`);
    assert(workflow.generated_locally === true, "workflow must be local");
    assert(workflow.account_required === false, "workflow must not require account");
    assert(workflow.cloud_sync === false, "workflow must not claim cloud sync");
    assert(workflow.permission_system === false, "workflow must not claim permission system");
    assert(workflow.replay_claim === false, "workflow must not claim replay");
    assert(workflow.status === "classroom_report_ready", `status mismatch: ${workflow.status}`);
    assert(workflow.stage_count === 6, `stage count mismatch: ${workflow.stage_count}`);
    assert(workflow.ready_stage_count === 6, `ready stage count mismatch: ${workflow.ready_stage_count}`);
    assert(workflow.missing_stage_count === 0, `missing stage count mismatch: ${workflow.missing_stage_count}`);
    assert(workflow.assignment_count === 2, `assignment count mismatch: ${workflow.assignment_count}`);
    assert(workflow.open_count === 1 && workflow.closed_count === 1, "open/closed count mismatch");
    assert(workflow.summary_count === 2, `summary count mismatch: ${workflow.summary_count}`);
    assert(workflow.pass_count === 1 && workflow.fail_count === 1, "pass/fail count mismatch");
    assert(workflow.suite_check_count === 2, `suite check count mismatch: ${workflow.suite_check_count}`);
    assert(workflow.mismatch_case_count === 1, `mismatch count mismatch: ${workflow.mismatch_case_count}`);
    assert(workflow.export_text_line_count === 3, `export text line count mismatch: ${workflow.export_text_line_count}`);
    assert(String(workflow.export_text).includes("a01\t수치 근 찾기 보고\t열림\t실패\troot_out_of_range\tunexpected_success"), "failed export row missing");
    assert(String(workflow.export_text).includes("a02\t물리 시간 전개 보고\t닫힘\t통과\t\t"), "pass export row missing");
    assert(String(result.text).includes("schema\tseamgrim.classroom_report_workflow.v1"), "workflow text schema missing");
    assert(String(result.text).includes("support_coordinate\t하-3"), "workflow text support coordinate missing");
    assert(String(result.text).includes("status\tclassroom_report_ready"), "workflow text status missing");
    assert(!String(result.text).endsWith("\n"), "workflow text must not have trailing newline");

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
