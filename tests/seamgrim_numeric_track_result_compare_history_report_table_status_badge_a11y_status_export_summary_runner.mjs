import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary: ok";
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
    preset_focus: "numeric a11y status export summary",
    preset_label: "numeric track a11y status export summary",
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
        lastRunKind: "space2d",
        lastRunChannels: 2,
        lastRunHash: MATH_HASH,
        lastRunAt: "2026-06-05T09:00:00.000Z",
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
        lastRunKind: "space2d",
        lastRunChannels: 2,
        lastRunHash: PHYS_HASH,
        lastRunAt: "2026-06-05T09:30:00.000Z",
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
        lastRunKind: "obs_only",
        lastRunChannels: 3,
        lastRunHash: ECON_HASH,
        lastRunAt: "2026-06-05T10:00:00.000Z",
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW__?.result_count === 3);
    await page.click("#btn-toggle-numeric-track-result-timeline");
    await page.waitForSelector("#numeric-track-result-timeline-panel:not(.hidden) #btn-show-numeric-track-result-compare-history:not(:disabled)");
    await page.click("#btn-show-numeric-track-result-compare-history");
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY__?.schema === "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary.v1");
    await page.waitForSelector(".numeric-track-compare-history-report-table-status-badge-a11y-status-export-summary");

    const result = await page.evaluate(() => {
      const pill = document.querySelector(".numeric-track-compare-history-report-table-status-badge-a11y-status-export-summary");
      return {
        summary: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY__,
        summaryText: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_TEXT__,
        pillText: pill?.textContent ?? "",
        pillTone: pill?.getAttribute("data-tone") ?? "",
        pillStatus: pill?.getAttribute("data-status") ?? "",
      };
    });

    const summary = result.summary;
    assert(summary?.schema === "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary.v1", "summary schema mismatch");
    assert(summary.track_id === TRACK_ID, `track mismatch: ${summary.track_id}`);
    assert(summary.source_schema === "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export.v1", "source schema mismatch");
    assert(summary.summary_claim === "metadata_summary", `summary claim mismatch: ${summary.summary_claim}`);
    assert(summary.export_claim === "metadata_text", `export claim mismatch: ${summary.export_claim}`);
    assert(summary.status_claim === "metadata_status", `status claim mismatch: ${summary.status_claim}`);
    assert(summary.replay_claim === false, "summary must not claim replay");
    assert(summary.status === "summary_ready", `summary status mismatch: ${summary.status}`);
    assert(summary.tone === "success", `summary tone mismatch: ${summary.tone}`);
    assert(summary.ready === true, "summary must be ready");
    assert(summary.status_value === "a11y_ready", `status value mismatch: ${summary.status_value}`);
    assert(summary.source_ready === true, "source must be ready");
    assert(summary.summary_check_count === 5, `summary check count mismatch: ${summary.summary_check_count}`);
    assert(summary.passing_summary_check_count === 5, `summary passing count mismatch: ${summary.passing_summary_check_count}`);
    assert(summary.missing_summary_check_count === 0, `summary missing count mismatch: ${summary.missing_summary_check_count}`);
    assert(summary.status_text_line_count >= 10, `status text line count too small: ${summary.status_text_line_count}`);
    assert(summary.status_export?.schema === "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export.v1", "embedded export missing");
    assert(String(result.summaryText).includes("summary_claim\tmetadata_summary"), "summary text claim missing");
    assert(String(result.summaryText).includes("status\tsummary_ready"), "summary text status missing");
    assert(String(result.summaryText).includes("ready\ttrue"), "summary text ready missing");
    assert(String(result.pillText).includes("summary_ready"), `pill text mismatch: ${result.pillText}`);
    assert(result.pillTone === "success", `pill tone mismatch: ${result.pillTone}`);
    assert(result.pillStatus === "summary_ready", `pill status mismatch: ${result.pillStatus}`);

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
