import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_numeric_report_workflow_consolidation: ok";
const TRACK_ID = "studio_numeric_curriculum_track_v1";
const MATH_ID = "rep_math_function_line_v1";
const PHYS_ID = "rep_phys_projectile_xy_v1";
const ECON_ID = "rep_econ_supply_demand_tax_v1";
const MATH_HASH = "blake3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const PHYS_HASH = "blake3:2222222222222222222222222222222222222222222222222222222222222222";
const ECON_HASH = "blake3:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

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

function listenOnSafePort(server, startPort) {
  return new Promise((resolve, reject) => {
    let port = startPort;
    const tryListen = () => {
      const onError = (error) => {
        server.off("listening", onListening);
        if (error?.code === "EADDRINUSE" && port < startPort + 50) {
          port += 1;
          tryListen();
          return;
        }
        reject(error);
      };
      const onListening = () => {
        server.off("error", onError);
        resolve({ server, baseUrl: `http://127.0.0.1:${port}` });
      };
      server.once("error", onError);
      server.once("listening", onListening);
      server.listen(port, "127.0.0.1");
    };
    tryListen();
  });
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
  return listenOnSafePort(server, 17680);
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|md)$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function makeResultLink({ lessonId, runKind, channels, stateHash, recordedAt, evidencePacks }) {
  return {
    schema: "seamgrim.numeric_track_run_result_link.v1",
    track_id: TRACK_ID,
    lesson_id: lessonId,
    preset_schema: "seamgrim.numeric_track_run_preset.v1",
    preset_focus: "numeric report workflow consolidation",
    preset_label: "numeric track report workflow consolidation",
    run_kind: runKind,
    channels,
    state_hash: stateHash,
    launch_kind: "browse_select_student",
    recorded_at: recordedAt,
    evidence_packs: evidencePacks,
  };
}

function seededRunPrefs() {
  return {
    lessons: {
      [MATH_ID]: {
        numericTrackRunResultLink: makeResultLink({
          lessonId: MATH_ID,
          runKind: "space2d",
          channels: 2,
          stateHash: MATH_HASH,
          recordedAt: "2026-06-05T09:00:00.000Z",
          evidencePacks: ["numeric_root_finding_bisection_v1"],
        }),
      },
      [PHYS_ID]: {
        numericTrackRunResultLink: makeResultLink({
          lessonId: PHYS_ID,
          runKind: "space2d",
          channels: 2,
          stateHash: PHYS_HASH,
          recordedAt: "2026-06-05T09:30:00.000Z",
          evidencePacks: ["ode_method_comparison_v1"],
        }),
      },
      [ECON_ID]: {
        numericTrackRunResultLink: makeResultLink({
          lessonId: ECON_ID,
          runKind: "obs_only",
          channels: 3,
          stateHash: ECON_HASH,
          recordedAt: "2026-06-05T10:00:00.000Z",
          evidencePacks: ["connect_flow_v1v_closure_v1"],
        }),
      },
    },
  };
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "screens/browse.js",
    "numeric_curriculum_track.js",
    "../lessons/index.json",
    "../lessons/active_allowlist.detjson",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    await context.addInitScript((prefs) => {
      window.localStorage.setItem("seamgrim.ui.run_prefs.v1", JSON.stringify(prefs));
    }, seededRunPrefs());
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error" && !String(msg.text() ?? "").includes("Failed to load resource")) {
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400 && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW__?.result_count === 3);
    await page.click("#btn-toggle-numeric-track-result-timeline");
    await page.waitForSelector("#numeric-track-result-timeline-panel:not(.hidden) #btn-show-numeric-track-result-compare-history:not(:disabled)");
    await page.click("#btn-show-numeric-track-result-compare-history");
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION__?.schema === "seamgrim.numeric_report_workflow_consolidation.v1");
    await page.waitForSelector(".numeric-report-workflow-consolidation");

    const result = await page.evaluate(() => {
      const strip = document.querySelector(".numeric-report-workflow-consolidation");
      return {
        workflow: window.__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION__,
        workflowText: window.__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_TEXT__,
        panelWorkflowSchema: document.querySelector("#numeric-track-result-compare-history-panel")?.getAttribute("data-workflow-schema") ?? "",
        stripText: strip?.textContent ?? "",
        stripStatus: strip?.getAttribute("data-status") ?? "",
        stripTone: strip?.getAttribute("data-tone") ?? "",
        copyButton: Boolean(document.querySelector("#btn-copy-numeric-report-workflow-consolidation")),
      };
    });

    const workflow = result.workflow;
    assert(workflow?.schema === "seamgrim.numeric_report_workflow_consolidation.v1", "workflow schema mismatch");
    assert(workflow.track_id === TRACK_ID, `track mismatch: ${workflow.track_id}`);
    assert(workflow.primary_coordinate === "마-3", `coordinate mismatch: ${workflow.primary_coordinate}`);
    assert(workflow.workflow_claim === "product_workflow_consolidation", `workflow claim mismatch: ${workflow.workflow_claim}`);
    assert(workflow.replay_claim === false, "workflow must not claim replay");
    assert(workflow.status === "workflow_ready", `workflow status mismatch: ${workflow.status}`);
    assert(workflow.tone === "success", `workflow tone mismatch: ${workflow.tone}`);
    assert(workflow.stage_count === 17, `stage count mismatch: ${workflow.stage_count}`);
    assert(workflow.ready_stage_count === 17, `ready stage count mismatch: ${workflow.ready_stage_count}`);
    assert(workflow.missing_stage_count === 0, `missing stage count mismatch: ${workflow.missing_stage_count}`);
    assert(workflow.pair_count === 2, `pair count mismatch: ${workflow.pair_count}`);
    assert(workflow.row_count === 2, `row count mismatch: ${workflow.row_count}`);
    assert(workflow.lesson_count === 3, `lesson count mismatch: ${workflow.lesson_count}`);
    assert(workflow.summary_status === "summary_ready", `summary status mismatch: ${workflow.summary_status}`);
    assert(workflow.summary_ready === true, "summary must be ready");
    assert(Array.isArray(workflow.source_schemas), "source schemas missing");
    assert(workflow.source_schemas.includes("seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary.v1"), "summary source schema missing");
    assert(String(result.workflowText).includes("workflow_claim\tproduct_workflow_consolidation"), "workflow text claim missing");
    assert(String(result.workflowText).includes("primary_coordinate\t마-3"), "workflow text coordinate missing");
    assert(String(result.workflowText).includes("status\tworkflow_ready"), "workflow text status missing");
    assert(result.panelWorkflowSchema === "seamgrim.numeric_report_workflow_consolidation.v1", "panel workflow schema missing");
    assert(String(result.stripText).includes("보고 workflow"), `strip label missing: ${result.stripText}`);
    assert(result.stripStatus === "workflow_ready", `strip status mismatch: ${result.stripStatus}`);
    assert(result.stripTone === "success", `strip tone mismatch: ${result.stripTone}`);
    assert(result.copyButton === true, "workflow copy button missing");

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
