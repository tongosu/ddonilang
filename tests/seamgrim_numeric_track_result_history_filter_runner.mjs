import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_numeric_track_result_history_filter: ok";
const TARGET_ID = "rep_math_function_line_v1";
const TRACK_ID = "studio_numeric_curriculum_track_v1";
const RESULT_HASH = "blake3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
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
  if (file.endsWith(".toml") || file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  if (file.endsWith(".md")) return "text/markdown; charset=utf-8";
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
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control)\.json$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/text\.md$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function seededRunPrefs() {
  return {
    lessons: {
      [TARGET_ID]: {
        lastRunKind: "space2d",
        lastRunChannels: 2,
        lastRunHash: RESULT_HASH,
        lastRunAt: "2026-06-05T09:00:00.000Z",
        lastLaunchKind: "browse_select_student",
        selectedXKey: "x",
        selectedYKey: "y",
        numericTrackRunResultLink: {
          schema: "seamgrim.numeric_track_run_result_link.v1",
          track_id: TRACK_ID,
          lesson_id: TARGET_ID,
          preset_schema: "seamgrim.numeric_track_run_preset.v1",
          preset_focus: "근/구간",
          preset_label: "수치트랙: 근/구간",
          run_kind: "space2d",
          channels: 2,
          state_hash: RESULT_HASH,
          launch_kind: "browse_select_student",
          recorded_at: "2026-06-05T09:00:00.000Z",
          evidence_packs: [
            "numeric_root_finding_bisection_v1",
            "polynomial_solve_minimum_v1",
          ],
        },
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
    const context = await browser.newContext({
      viewport: { width: 1360, height: 860 },
      locale: "ko-KR",
    });
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

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#lesson-card-grid");
    await page.waitForFunction(() => window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER__?.schema === "seamgrim.numeric_track_result_history_filter.v1");

    const initial = await page.evaluate(() => ({
      snapshot: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER__,
      text: window.__SEAMGRIM_NUMERIC_TRACK_RESULT_HISTORY_FILTER_TEXT__,
      buttonCount: document.querySelector("#btn-filter-numeric-track-results")?.dataset?.count ?? "",
      buttonDisabled: document.querySelector("#btn-filter-numeric-track-results")?.disabled ?? true,
    }));
    assert(initial.snapshot.result_count === 1, `result count mismatch: ${initial.snapshot.result_count}`);
    assert(initial.snapshot.result_lesson_ids?.[0] === TARGET_ID, "result lesson id mismatch");
    assert(initial.snapshot.rows?.[0]?.preset_focus === "근/구간", "history focus mismatch");
    assert(initial.snapshot.rows?.[0]?.state_hash_short === RESULT_HASH.slice(0, 12), "history short hash mismatch");
    assert(initial.buttonCount === "1", `button count mismatch: ${initial.buttonCount}`);
    assert(initial.buttonDisabled === false, "result filter button disabled");
    assert(String(initial.text).includes("schema\tseamgrim.numeric_track_result_history_filter.v1"), "history text schema missing");
    assert(String(initial.text).includes(`${TARGET_ID}\t`), "history text lesson missing");

    await page.click("#btn-filter-numeric-track-results");
    await page.waitForFunction(() => document.querySelector("#btn-filter-numeric-track-results")?.dataset?.active === "1");
    await page.waitForSelector(`.lesson-card[data-lesson-id="${TARGET_ID}"] .badge-numeric-track-result`);
    const filtered = await page.evaluate((targetId) => {
      const cards = Array.from(document.querySelectorAll(".lesson-card"));
      const targetCard = document.querySelector(`.lesson-card[data-lesson-id="${targetId}"]`);
      return {
        cardIds: cards.map((card) => card.dataset.lessonId),
        badgeText: targetCard?.querySelector(".badge-numeric-track-result")?.textContent?.trim() ?? "",
        hintText: Array.from(targetCard?.querySelectorAll(".card-state-hint") ?? [])
          .map((node) => node.textContent?.trim() ?? "")
          .find((text) => text.startsWith("수치결과")) ?? "",
      };
    }, TARGET_ID);
    assert(filtered.cardIds.length === 1, `filtered card count mismatch: ${filtered.cardIds.join(",")}`);
    assert(filtered.cardIds[0] === TARGET_ID, `filtered target mismatch: ${filtered.cardIds[0]}`);
    assert(filtered.badgeText === "수치결과", `result badge mismatch: ${filtered.badgeText}`);
    assert(filtered.hintText.includes("수치결과 · 근/구간"), `result hint mismatch: ${filtered.hintText}`);
    assert(filtered.hintText.includes(RESULT_HASH.slice(0, 12)), "result hint hash missing");

    await page.click("#btn-filter-numeric-track-results");
    await page.waitForFunction(() => document.querySelector("#btn-filter-numeric-track-results")?.dataset?.active === "0");

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
