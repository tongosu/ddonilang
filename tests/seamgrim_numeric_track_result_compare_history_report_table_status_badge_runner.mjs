import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_numeric_track_result_compare_history_report_table_status_badge: ok";
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

function makeResultLink({ lessonId, focus, label, runKind, channels, stateHash, recordedAt, evidencePacks }) {
  return {
    schema: "seamgrim.numeric_track_run_result_link.v1",
    track_id: TRACK_ID,
    lesson_id: lessonId,
    preset_schema: "seamgrim.numeric_track_run_preset.v1",
    preset_focus: focus,
    preset_label: label,
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
          focus: "root finding",
          label: "numeric track: root finding",
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
          focus: "ode",
          label: "numeric track: ode",
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
          focus: "range report",
          label: "numeric track: range report",
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
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE__?.schema === "seamgrim.numeric_track_result_compare_history_report_table_status_badge.v1");
    await page.waitForSelector(".numeric-track-compare-history-report-table-status-badge");

    const result = await page.evaluate(() => ({
      status: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS__,
      badge: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE__,
      badgeText: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_TEXT__,
      badgeDomText: document.querySelector(".numeric-track-compare-history-report-table-status-badge")?.textContent ?? "",
      badgeTone: document.querySelector(".numeric-track-compare-history-report-table-status-badge")?.dataset.tone ?? "",
      panelText: document.querySelector("#numeric-track-result-compare-history-panel")?.textContent ?? "",
    }));

    assert(result.status?.schema === "seamgrim.numeric_track_result_compare_history_report_table_status.v1", "status schema mismatch");
    assert(result.badge.track_id === TRACK_ID, `badge track mismatch: ${result.badge.track_id}`);
    assert(result.badge.source_schema === "seamgrim.numeric_track_result_compare_history_report_table_status.v1", "badge source schema mismatch");
    assert(result.badge.badge_claim === "metadata_badge", `badge claim mismatch: ${result.badge.badge_claim}`);
    assert(result.badge.replay_claim === false, "badge must not claim replay");
    assert(result.badge.label === "변화있음", `badge label mismatch: ${result.badge.label}`);
    assert(result.badge.tone === "warning", `badge tone mismatch: ${result.badge.tone}`);
    assert(result.badge.has_changes === true, "badge should preserve change flag");
    assert(result.badge.reason_count === 3, `badge reason count mismatch: ${result.badge.reason_count}`);
    assert(result.badge.status_reasons?.join("|") === "state_hash_changed|run_kind_changed|channel_delta_nonzero", "badge reasons mismatch");
    assert(result.badge.pair_count === 2, `badge pair count mismatch: ${result.badge.pair_count}`);
    assert(result.badge.lesson_count === 3, `badge lesson count mismatch: ${result.badge.lesson_count}`);
    assert(result.badge.state_hash_changed_count === 2, `badge hash count mismatch: ${result.badge.state_hash_changed_count}`);
    assert(result.badge.run_kind_changed_count === 1, `badge run-kind count mismatch: ${result.badge.run_kind_changed_count}`);
    assert(result.badge.channel_delta_abs_total === 1, `badge channel abs mismatch: ${result.badge.channel_delta_abs_total}`);
    assert(result.badge.status?.schema === "seamgrim.numeric_track_result_compare_history_report_table_status.v1", "embedded status missing");
    assert(String(result.badgeText).includes("schema\tseamgrim.numeric_track_result_compare_history_report_table_status_badge.v1"), "badge text schema missing");
    assert(String(result.badgeText).includes("badge_claim\tmetadata_badge"), "badge text claim missing");
    assert(String(result.badgeText).includes("replay_claim\tfalse"), "badge text replay boundary missing");
    assert(String(result.badgeText).includes("label\t변화있음"), "badge text label missing");
    assert(String(result.badgeText).includes("tone\twarning"), "badge text tone missing");
    assert(result.badgeDomText === "변화있음", `badge DOM text mismatch: ${result.badgeDomText}`);
    assert(result.badgeTone === "warning", `badge DOM tone mismatch: ${result.badgeTone}`);
    assert(String(result.panelText).includes("변화있음"), "panel badge label missing");

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
