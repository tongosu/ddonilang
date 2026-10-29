#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_classroom_mode_browser: ok";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) {
    throw new Error(`missing file: ${file}`);
  }
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  return "application/octet-stream";
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
      const rel = rawPath.replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, rel);
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, {
        "content-type": mimeType(file),
        "cache-control": "no-store",
        "access-control-allow-origin": "*",
      });
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
      if (!address || typeof address === "string") {
        reject(new Error("failed to bind static server"));
        return;
      }
      if (UNSAFE_BROWSER_PORTS.has(address.port)) {
        server.close(() => {
          createServer(root).then(resolve, reject);
        });
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
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
  for (const rel of [
    "index.html",
    "studio_classroom_mode.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1180, height: 760 },
      locale: "ko-KR",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        if (String(msg.text() ?? "").includes("Failed to load resource")) return;
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
        buildClassroomAssignmentList,
        buildClassroomRunResultSummary,
        buildClassroomSuiteCheckView,
        buildClassroomExportReport,
        formatClassroomExportReportText,
      } = await import("./studio_classroom_mode.js");
      const assignments = [
        {
          assignment_id: "a01",
          title: "전압 기초",
          lesson_id: "lesson.voltage",
          goals: ["전압 변화 읽기"],
          missions: ["표에서 최대 전압 찾기"],
          result_views: ["graph", "table"],
          result_views_label: "그래프, 표",
          due_label: "오늘",
          status: "open",
        },
        {
          assignment_id: "a02",
          title: "유량 점검",
          lesson_id: "lesson.flow",
          goals: ["유량 비교"],
          missions: ["결과표에서 흐름 차이 설명"],
          result_views: ["table"],
          result_views_label: "표",
          due_label: "내일",
          status: "closed",
        },
      ];
      const list = buildClassroomAssignmentList(assignments);
      const suiteCheck = {
        __이음관계종류: "endpoint_solve_range_case_suite_check",
        판정: "실패",
        전체통과: false,
        개수: 3,
        통과개수: 2,
        실패개수: 1,
        실패케이스들: ["flow_out_of_range"],
        기대실패통과케이스들: ["unexpected_success"],
        기대통과실패케이스들: ["unexpected_failure"],
      };
      const passSuiteCheck = {
        __이음관계종류: "endpoint_solve_range_case_suite_check",
        판정: "통과",
        전체통과: true,
        개수: 1,
        통과개수: 1,
        실패개수: 0,
        실패케이스들: [],
        기대실패통과케이스들: [],
        기대통과실패케이스들: [],
      };
      const view = buildClassroomSuiteCheckView(suiteCheck);
      const passView = buildClassroomSuiteCheckView(passSuiteCheck);
      const summary = buildClassroomRunResultSummary({
        assignment: assignments[0],
        runResult: { exit_code: 0, stdout: ["ok"], stderr: [] },
        suiteCheck,
      });
      const passSummary = buildClassroomRunResultSummary({
        assignment: assignments[1],
        runResult: { exit_code: 0, stdout: ["ok"], stderr: [] },
        suiteCheck: passSuiteCheck,
      });
      const report = buildClassroomExportReport({
        assignmentList: list,
        resultSummaries: [summary, passSummary],
      });
      return {
        list,
        view,
        passView,
        summary,
        passSummary,
        report,
        text: formatClassroomExportReportText(report),
      };
    });

    assert(result.list.__종류 === "studio_classroom_assignment_list", "assignment list kind mismatch");
    assert(result.list.assignment_count === 2, "assignment count mismatch");
    assert(result.list.open_count === 1 && result.list.closed_count === 1, "open/closed counts mismatch");
    assert(result.list.assignments[0].assignment_id === "a01", "assignment order not preserved");
    assert(result.list.account_required === false && result.list.cloud_sync === false, "assignment list must be local only");
    assert(result.view.__종류 === "studio_classroom_suite_check_view", "suite check view kind mismatch");
    assert(result.view.judgement === "실패", "suite check judgement mismatch");
    assert(result.view.failed_cases.includes("flow_out_of_range"), "failed case missing");
    assert(result.view.expected_fail_passed_cases.includes("unexpected_success"), "unexpected success mismatch missing");
    assert(result.view.expected_pass_failed_cases.includes("unexpected_failure"), "unexpected failure mismatch missing");
    assert(result.passView.judgement === "통과" && result.passView.overall_pass === true, "pass suite view mismatch");
    assert(result.summary.__종류 === "studio_classroom_run_result_summary", "run summary kind mismatch");
    assert(result.summary.run_status === "실패", "suite failure should dominate run status");
    assert(result.summary.failed_case_count === 1, "failed case count mismatch");
    assert(result.passSummary.run_status === "통과", "pass summary status mismatch");
    assert(result.report.__종류 === "studio_classroom_export_report", "report kind mismatch");
    assert(result.report.generated_locally === true, "report must be local");
    assert(result.report.account_required === false && result.report.cloud_sync === false, "report must not claim account/cloud");
    assert(result.report.assignment_count === 2 && result.report.summary_count === 2, "report counts mismatch");
    assert(result.text.startsWith("수업 코드\t수업 제목\t수업 목표\t오늘 활동\t결과 확인\t배포 상태\t실행 결과\t확인 필요\t비고\n"), "report text header mismatch");
    assert(!result.text.includes("실패케이스") && !result.text.includes("불일치"), "report text must not expose internal QA headers");
    assert(result.text.includes("a01\t전압 기초\t전압 변화 읽기\t표에서 최대 전압 찾기\t그래프, 표\t열림\t실패\tflow_out_of_range\tunexpected_success|unexpected_failure"), "failed report row mismatch");
    assert(result.text.includes("a02\t유량 점검\t유량 비교\t결과표에서 흐름 차이 설명\t표\t닫힘\t통과\t\t"), "pass report row mismatch");
    assert(!result.text.endsWith("\n"), "report text must not have trailing newline");

    if (failures.length > 0) {
      throw new Error(failures.join("\n"));
    }
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
