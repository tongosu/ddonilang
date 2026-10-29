import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_numeric_track_result_compare: ok";
const TRACK_ID = "studio_numeric_curriculum_track_v1";
const MATH_ID = "rep_math_function_line_v1";
const ECON_ID = "rep_econ_supply_demand_tax_v1";
const MATH_HASH = "blake3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const ECON_HASH = "blake3:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

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
        server.close(() => createServer(root).then(resolve, reject));
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
      if (msg.type() === "error") {
        if (String(msg.text() ?? "").includes("Failed to load resource")) return;
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400) {
        if (res.status() === 404 && isAllowedFallback404(res.url())) return;
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_TIMELINE_VIEW__?.result_count === 2);
    await page.click("#btn-toggle-numeric-track-result-timeline");
    await page.waitForSelector("#numeric-track-result-timeline-panel:not(.hidden) #btn-compare-numeric-track-results:not(:disabled)");
    await page.click("#btn-compare-numeric-track-results");
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE__?.schema === "seamgrim.numeric_track_result_compare.v1");

    const result = await page.evaluate(() => ({
      compare: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE__,
      text: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_TEXT__,
      panelHidden: document.querySelector("#numeric-track-result-compare-panel")?.classList?.contains("hidden") ?? true,
      panelSchema: document.querySelector("#numeric-track-result-compare-panel")?.dataset?.schema ?? "",
      panelText: document.querySelector("#numeric-track-result-compare-panel")?.textContent ?? "",
      buttonCount: document.querySelector("#btn-compare-numeric-track-results")?.dataset?.count ?? "",
    }));

    assert(result.buttonCount === "2", `compare button count mismatch: ${result.buttonCount}`);
    assert(result.panelHidden === false, "compare panel should be visible");
    assert(result.panelSchema === "seamgrim.numeric_track_result_compare.v1", `compare panel schema mismatch: ${result.panelSchema}`);
    assert(result.compare.source_schema === "seamgrim.numeric_track_result_timeline_view.v1", "compare source schema mismatch");
    assert(result.compare.compare_kind === "latest_vs_previous", `compare kind mismatch: ${result.compare.compare_kind}`);
    assert(result.compare.compare_claim === "metadata_only", `compare claim mismatch: ${result.compare.compare_claim}`);
    assert(result.compare.replay_claim === false, "compare must not claim replay");
    assert(result.compare.latest.lesson_id === ECON_ID, `latest lesson mismatch: ${result.compare.latest.lesson_id}`);
    assert(result.compare.previous.lesson_id === MATH_ID, `previous lesson mismatch: ${result.compare.previous.lesson_id}`);
    assert(result.compare.same_lesson === false, "same_lesson should be false");
    assert(result.compare.same_focus === false, "same_focus should be false");
    assert(result.compare.same_run_kind === false, "same_run_kind should be false");
    assert(result.compare.channel_delta === 1, `channel delta mismatch: ${result.compare.channel_delta}`);
    assert(result.compare.state_hash_changed === true, "state_hash_changed should be true");
    assert(String(result.text).includes("schema\tseamgrim.numeric_track_result_compare.v1"), "compare text schema missing");
    assert(String(result.text).includes("compare_claim\tmetadata_only"), "compare text claim missing");
    assert(String(result.text).includes("replay_claim\tfalse"), "compare text replay boundary missing");
    assert(String(result.text).includes("latest\t"), "compare text latest row missing");
    assert(String(result.text).includes("previous\t"), "compare text previous row missing");
    assert(String(result.panelText).includes("최근 2개 수치 결과 비교"), "compare panel title missing");
    assert(String(result.panelText).includes("hash 변화: 있음"), "compare panel hash summary missing");

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
