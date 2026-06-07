import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_numeric_track_result_compare_history_report_table_export: ok";
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
        lastRunKind: "space2d",
        lastRunChannels: 2,
        lastRunHash: MATH_HASH,
        lastRunAt: "2026-06-05T09:00:00.000Z",
        numericTrackRunResultLink: makeResultLink({
          lessonId: MATH_ID,
          focus: "근/구간",
          label: "수치트랙: 근/구간",
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
          focus: "시간 전개",
          label: "수치트랙: 시간 전개",
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
          focus: "범위 보고",
          label: "수치트랙: 범위 보고",
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
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE__?.schema === "seamgrim.numeric_track_result_compare_history_report_table.v1");
    await page.waitForSelector(".numeric-track-compare-history-report-table tbody tr");
    await page.waitForSelector("#btn-copy-numeric-track-result-compare-history-report-table-export");
    await page.click("#btn-copy-numeric-track-result-compare-history-report-table-export");
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT__?.schema === "seamgrim.numeric_track_result_compare_history_report_table_export.v1");

    const result = await page.evaluate(() => ({
      report: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT__,
      table: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE__,
      tableText: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_TEXT__,
      tableExport: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT__,
      exportText: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_EXPORT_TEXT__,
      panelText: document.querySelector("#numeric-track-result-compare-history-panel")?.textContent ?? "",
      tableSchema: document.querySelector("#numeric-track-result-compare-history-panel")?.dataset.reportTableSchema ?? "",
      tableRowCount: document.querySelector(".numeric-track-compare-history-report-table")?.dataset.rowCount ?? "",
      domRows: Array.from(document.querySelectorAll(".numeric-track-compare-history-report-table tbody tr")).map((row) => row.textContent ?? ""),
    }));

    assert(result.report.schema === "seamgrim.numeric_track_result_compare_history_report.v1", "report schema mismatch");
    assert(result.table.track_id === TRACK_ID, `table track mismatch: ${result.table.track_id}`);
    assert(result.table.source_schema === "seamgrim.numeric_track_result_compare_history_report.v1", "table source schema mismatch");
    assert(result.table.table_claim === "metadata_table", `table claim mismatch: ${result.table.table_claim}`);
    assert(result.table.replay_claim === false, "table must not claim replay");
    assert(result.table.pair_count === 2, `table pair count mismatch: ${result.table.pair_count}`);
    assert(result.table.column_count === 8, `table column count mismatch: ${result.table.column_count}`);
    assert(result.table.state_hash_changed_count === 2, `table hash changed count mismatch: ${result.table.state_hash_changed_count}`);
    assert(result.table.run_kind_changed_count === 1, `table run kind changed count mismatch: ${result.table.run_kind_changed_count}`);
    assert(result.table.channel_delta_total === 1, `table channel total mismatch: ${result.table.channel_delta_total}`);
    assert(result.table.channel_delta_abs_total === 1, `table channel abs total mismatch: ${result.table.channel_delta_abs_total}`);
    assert(result.table.columns?.join("|") === "index|latest_lesson_id|previous_lesson_id|same_lesson|same_focus|same_run_kind|channel_delta|state_hash_changed", "table columns mismatch");
    assert(result.table.rows?.[0]?.latest_lesson_id === ECON_ID, "first table row latest mismatch");
    assert(result.table.rows?.[0]?.previous_lesson_id === PHYS_ID, "first table row previous mismatch");
    assert(result.table.rows?.[0]?.same_run_kind === false, "first table row run-kind mismatch");
    assert(result.table.rows?.[0]?.channel_delta === 1, "first table row channel mismatch");
    assert(result.table.rows?.[1]?.latest_lesson_id === PHYS_ID, "second table row latest mismatch");
    assert(result.table.rows?.[1]?.previous_lesson_id === MATH_ID, "second table row previous mismatch");
    assert(result.table.rows?.[1]?.same_run_kind === true, "second table row run-kind mismatch");
    assert(result.tableSchema === "seamgrim.numeric_track_result_compare_history_report_table.v1", `panel table schema mismatch: ${result.tableSchema}`);
    assert(result.tableRowCount === "2", `panel row count mismatch: ${result.tableRowCount}`);
    assert(String(result.tableText).includes("schema\tseamgrim.numeric_track_result_compare_history_report_table.v1"), "table text schema missing");
    assert(String(result.tableText).includes("table_claim\tmetadata_table"), "table text claim missing");
    assert(String(result.tableText).includes("columns\tindex|latest_lesson_id|previous_lesson_id|same_lesson|same_focus|same_run_kind|channel_delta|state_hash_changed"), "table text columns missing");
    assert(String(result.tableText).includes(`${ECON_ID}\t${PHYS_ID}`), "table text first pair missing");
    assert(String(result.panelText).includes("보고서 표"), "panel table title missing");
    assert(String(result.panelText).includes("rows:2"), "panel table rows missing");
    assert(String(result.panelText).includes("표 복사"), "table export copy action missing");
    assert(result.domRows.some((row) => row.includes(ECON_ID) && row.includes(PHYS_ID)), "DOM first pair row missing");
    assert(result.domRows.some((row) => row.includes(PHYS_ID) && row.includes(MATH_ID)), "DOM second pair row missing");
    assert(result.tableExport.track_id === TRACK_ID, `export track mismatch: ${result.tableExport.track_id}`);
    assert(result.tableExport.source_schema === "seamgrim.numeric_track_result_compare_history_report_table.v1", "export source schema mismatch");
    assert(result.tableExport.export_claim === "metadata_text", `export claim mismatch: ${result.tableExport.export_claim}`);
    assert(result.tableExport.replay_claim === false, "table export must not claim replay");
    assert(result.tableExport.pair_count === 2, `export pair count mismatch: ${result.tableExport.pair_count}`);
    assert(result.tableExport.column_count === 8, `export column count mismatch: ${result.tableExport.column_count}`);
    assert(result.tableExport.state_hash_changed_count === 2, `export hash count mismatch: ${result.tableExport.state_hash_changed_count}`);
    assert(result.tableExport.run_kind_changed_count === 1, `export run kind count mismatch: ${result.tableExport.run_kind_changed_count}`);
    assert(result.tableExport.channel_delta_total === 1, `export channel total mismatch: ${result.tableExport.channel_delta_total}`);
    assert(result.tableExport.channel_delta_abs_total === 1, `export channel abs total mismatch: ${result.tableExport.channel_delta_abs_total}`);
    assert(result.tableExport.table?.schema === "seamgrim.numeric_track_result_compare_history_report_table.v1", "embedded table missing");
    assert(String(result.tableExport.table_text).includes("table_claim\tmetadata_table"), "embedded table text missing");
    assert(String(result.exportText).includes("schema\tseamgrim.numeric_track_result_compare_history_report_table_export.v1"), "export text schema missing");
    assert(String(result.exportText).includes("source_schema\tseamgrim.numeric_track_result_compare_history_report_table.v1"), "export text source missing");
    assert(String(result.exportText).includes("export_claim\tmetadata_text"), "export text claim missing");
    assert(String(result.exportText).includes("replay_claim\tfalse"), "export text replay boundary missing");
    assert(String(result.exportText).includes("table_text"), "export text table section missing");

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
