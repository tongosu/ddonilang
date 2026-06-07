import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "studio_numeric_result_report_consolidation: ok";
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
    preset_focus: "numeric result report consolidation",
    preset_label: "numeric track result report consolidation",
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

function seededLessons() {
  return [
    { id: MATH_ID, title: "함수 그래프와 근", subject: "math" },
    { id: PHYS_ID, title: "포물선 운동", subject: "physics" },
    { id: ECON_ID, title: "수요 공급과 세금", subject: "economics" },
  ];
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
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
    const result = await page.evaluate(async ({ baseUrl: pageBaseUrl, lessons, runPrefs }) => {
      const mod = await import(`${pageBaseUrl}/solutions/seamgrim_ui_mvp/ui/numeric_curriculum_track.js`);
      const consolidation = mod.buildNumericResultReportConsolidation({ lessons, runPrefs });
      const text = mod.formatNumericResultReportConsolidationText(consolidation);
      return { consolidation, text };
    }, { baseUrl, lessons: seededLessons(), runPrefs: seededRunPrefs() });

    const row = result.consolidation;
    assert(row?.schema === "seamgrim.numeric_result_report_consolidation.v1", "schema mismatch");
    assert(row.track_id === TRACK_ID, `track mismatch: ${row.track_id}`);
    assert(row.primary_coordinate === "마-3", `primary coordinate mismatch: ${row.primary_coordinate}`);
    assert(row.support_coordinate === "다-2", `support coordinate mismatch: ${row.support_coordinate}`);
    assert(row.workflow_claim === "numeric_result_report_consolidation", `workflow claim mismatch: ${row.workflow_claim}`);
    assert(row.replay_claim === false, "must not claim replay");
    assert(row.runtime_claim === false, "must not claim runtime");
    assert(row.lesson_schema_change === false, "must not claim lesson schema change");
    assert(row.active_allowlist_mutation === false, "must not mutate active allowlist");
    assert(row.status === "numeric_result_report_ready", `status mismatch: ${row.status}`);
    assert(row.tone === "success", `tone mismatch: ${row.tone}`);
    assert(row.result_count === 3, `result count mismatch: ${row.result_count}`);
    assert(row.history_result_count === 3, `history count mismatch: ${row.history_result_count}`);
    assert(row.summary_result_count === 3, `summary count mismatch: ${row.summary_result_count}`);
    assert(row.timeline_result_count === 3, `timeline count mismatch: ${row.timeline_result_count}`);
    assert(row.pair_count === 2, `pair count mismatch: ${row.pair_count}`);
    assert(row.evidence_pack_count === 3, `evidence pack count mismatch: ${row.evidence_pack_count}`);
    assert(row.latest_recorded_at === "2026-06-05T10:00:00.000Z", `latest recorded mismatch: ${row.latest_recorded_at}`);
    assert(row.report_workflow_status === "workflow_ready", `report workflow status mismatch: ${row.report_workflow_status}`);
    assert(row.report_workflow_stage_count === 17, `report workflow stage count mismatch: ${row.report_workflow_stage_count}`);
    assert(row.report_workflow_ready_stage_count === 17, `report workflow ready count mismatch: ${row.report_workflow_ready_stage_count}`);
    assert(row.stage_count === 10, `stage count mismatch: ${row.stage_count}`);
    assert(row.ready_stage_count === 10, `ready stage count mismatch: ${row.ready_stage_count}`);
    assert(row.missing_stage_count === 0, `missing stage count mismatch: ${row.missing_stage_count}`);
    assert(Array.isArray(row.source_schemas), "source schemas missing");
    assert(row.source_schemas.includes("seamgrim.numeric_track_result_summary_export.v1"), "summary source schema missing");
    assert(row.source_schemas.includes("seamgrim.numeric_report_workflow_consolidation.v1"), "workflow source schema missing");
    assert(String(result.text).includes("workflow_claim\tnumeric_result_report_consolidation"), "text claim missing");
    assert(String(result.text).includes("primary_coordinate\t마-3"), "text primary coordinate missing");
    assert(String(result.text).includes("support_coordinate\t다-2"), "text support coordinate missing");
    assert(String(result.text).includes("status\tnumeric_result_report_ready"), "text status missing");
    assert(String(result.text).includes("report_workflow_stage_count\t17"), "text workflow stage count missing");

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
