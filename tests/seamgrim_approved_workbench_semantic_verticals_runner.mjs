#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_approved_workbench_semantic_verticals: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml") || file.endsWith(".ddn") || file.endsWith(".md")) return "text/plain; charset=utf-8";
  return "application/octet-stream";
}

function createServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const rel = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html" : url.pathname).replace(/^\/+/, "");
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
        reject(new Error("failed to bind server"));
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function snapshot(page) {
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
}

async function waitFor(page, predicate, label) {
  await page.waitForFunction(predicate, label, { timeout: 60000 });
  return snapshot(page);
}

async function assertCanvasNonBlank(page) {
  const sample = await page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='bogae-canvas']");
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let nonWhite = 0;
    for (let i = 0; i < data.length; i += 4) {
      if (data[i + 3] && (data[i] < 240 || data[i + 1] < 240 || data[i + 2] < 240)) nonWhite += 1;
    }
    return { width: canvas.width, height: canvas.height, nonWhite };
  });
  assert(sample.width >= 320 && sample.height >= 240, `canvas too small: ${JSON.stringify(sample)}`);
  assert(sample.nonWhite > 500, `canvas blank: ${JSON.stringify(sample)}`);
}

async function ensurePausedRuntime(page) {
  let snap = await snapshot(page);
  if (snap?.executionLifecycle?.status === "idle" || snap?.executionLifecycle?.status === "done") {
    await page.click("[data-testid='lifecycle-run']");
    snap = await waitFor(
      page,
      () => {
        const s = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        return s?.executionLifecycle?.status === "running" && Boolean(s?.activeFrame);
      },
      "running",
    );
  }
  if (snap?.executionLifecycle?.status === "running") {
    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    snap = await waitFor(
      page,
      () => {
        const s = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        return s?.executionLifecycle?.status === "paused" && !s?.livePump?.in_flight;
      },
      "paused",
    );
  }
  assert(snap?.executionLifecycle?.status === "paused", `runtime did not settle paused: ${JSON.stringify(snap?.executionLifecycle)}`);
  return snap;
}

async function runAndStep(page) {
  let snap = await ensurePausedRuntime(page);
  const startRunId = snap.activeRunId;
  const beforeFrameCount = snap.runs.find((run) => run.run_id === startRunId)?.frame_count ?? 0;
  await page.click("[data-testid='lifecycle-step']");
  snap = await waitFor(
    page,
    ({ runId, frameCount }) => {
      const s = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return s?.runs?.find((run) => run.run_id === runId)?.frame_count >= frameCount + 1;
    },
    { runId: startRunId, frameCount: beforeFrameCount },
  );
  return snap;
}

async function branchLatest(page) {
  await page.click("[data-testid='history-back']");
  await waitFor(page, () => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runtimeStatus === "history-readonly", "history");
  await page.click("[data-testid='history-branch']");
  return waitFor(
    page,
    () => {
      const s = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return s?.runs?.length >= 2 && s?.activeRunId?.startsWith("branch_");
    },
    "branch",
  );
}

async function stopActiveRun(page) {
  const snap = await snapshot(page);
  if (!["running", "paused", "pausing"].includes(String(snap?.executionLifecycle?.status ?? ""))) return snap;
  await page.click("[data-testid='lifecycle-stop']");
  return waitFor(
    page,
    () => ["done", "idle"].includes(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status),
    "stopped",
  );
}

async function selectProjectByCard(page, testId, projectId) {
  await page.evaluate((id) => document.querySelector(`[data-testid='${id}']`)?.click(), testId);
  await waitFor(
    page,
    (expected) => document.querySelector("[data-testid='approved-workbench-root']")?.dataset.projectId === expected,
    projectId,
  );
}

async function saveEditReopen(page, marker) {
  const saved = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.saveProjectSnapshot());
  const beforeText = await page.locator("[data-testid='source-editor']").inputValue();
  await page.evaluate((suffix) => {
    const editor = document.querySelector("[data-testid='source-editor']");
    editor.value = `${editor.value}\n// ${suffix}`;
    editor.dispatchEvent(new Event("input", { bubbles: true }));
  }, marker);
  assert((await page.locator("[data-testid='source-editor']").inputValue()).includes(marker), "edit marker was not applied");
  await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.reopenProjectSnapshot());
  await page.waitForTimeout(50);
  const reopened = await page.locator("[data-testid='source-editor']").inputValue();
  assert(reopened === beforeText, "reopen did not restore saved source");
  assert(!reopened.includes(marker), "reopen kept unsaved marker");
  return saved;
}

async function assertSemanticPolicy(page, expectedGrade) {
  const policy = await page.evaluate(() => ({
    contract: document.querySelector("[data-testid='semantic-view-contract']")?.textContent ?? "",
    layout: document.querySelector("[data-testid='layout-policy']")?.textContent ?? "",
    roles: document.querySelector("[data-testid='view-role-summary']")?.textContent ?? "",
    grade: document.querySelector("#view-grade-label")?.textContent ?? "",
    source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
  }));
  assert(policy.contract.includes("primary") || policy.contract.includes("으뜸"), "semantic contract missing");
  assert(policy.layout.includes("layout policy") && policy.layout.includes("DDN 메인"), `layout policy mismatch: ${policy.layout}`);
  assert(policy.roles.includes("primary=") && policy.roles.includes("magnifier="), `view roles mismatch: ${policy.roles}`);
  assert(policy.grade.includes(expectedGrade), `view grade mismatch: ${policy.grade}`);
  assert(!/\b(px|css|CSS)\b/.test(policy.source), "DDN source should not own pixel/CSS layout");
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
      locale: "ko-KR",
      reducedMotion: "reduce",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      const text = msg.text();
      if (["error", "warning"].includes(msg.type()) && !text.includes("Failed to load resource")) {
        failures.push(`${msg.type()}: ${text}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("response", (res) => {
      const url = res.url();
      if (res.status() >= 400 && !url.includes("/api/v1/projects")) {
        failures.push(`response ${res.status()}: ${url}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await selectProjectByCard(page, "project-card-rep_phys_pendulum_wave1_v1", "rep_phys_pendulum_wave1_v1");

    await assertSemanticPolicy(page, "explicit");
    await ensurePausedRuntime(page);
    await page.click("[data-testid='view-role-side']");
    await page.waitForFunction(() => document.querySelector("[data-testid='bogae-stage']")?.dataset.sideOpen === "true");
    const sideText = await page.textContent("[data-testid='side-view-body']");
    assert(sideText.includes("UI layout policy"), "side view should explain UI layout policy");
    await page.click("[data-testid='view-role-overlay']");
    await assertCanvasNonBlank(page);

    let snap = await runAndStep(page);
    assert(snap.projectId === "rep_phys_pendulum_wave1_v1", "pendulum project mismatch");
    assert(snap.runs[0].frame_count >= 2, "pendulum run frames missing");
    const pendulumSaved = await saveEditReopen(page, "unsaved-pendulum-marker");
    assert(pendulumSaved.canonical_hash, "pendulum snapshot hash missing");
    snap = await runAndStep(page);
    snap = await branchLatest(page);
    assert(snap.runs.some((run) => run.parent_run_id), "pendulum branch missing");
    await stopActiveRun(page);

    await selectProjectByCard(page, "project-card-rep_grid_game_state_drop_v1", "rep_grid_game_state_drop_v1");
    await assertSemanticPolicy(page, "auto");
    await page.click("[data-testid='action-drop']");
    snap = await snapshot(page);
    assert(snap.nextInput?.intent === "drop", "grid action input missing");
    snap = await runAndStep(page);
    assert(snap.projectId === "rep_grid_game_state_drop_v1", "grid project mismatch");
    assert(snap.activeFrame.input_snapshot.schema === "seamgrim.input_snapshot.ui.v1", "grid InputSnapshot schema missing");
    const magnifierDisabled = await page.locator("[data-testid='view-role-magnifier']").isDisabled().catch(() => true);
    assert(magnifierDisabled, "magnifier should stay gated before committed target selection");
    await page.click("[data-testid='view-role-side']");
    await page.waitForFunction(() => document.querySelector("[data-testid='bogae-stage']")?.dataset.sideOpen === "true");
    await assertCanvasNonBlank(page);
    const gridSaved = await saveEditReopen(page, "unsaved-grid-marker");
    assert(gridSaved.canonical_hash, "grid snapshot hash missing");
    snap = await runAndStep(page);
    snap = await branchLatest(page);
    assert(snap.runs.some((run) => run.parent_run_id), "grid branch missing");

    assert(failures.length === 0, `browser failures: ${failures.join("\n")}`);
    await context.close();
    console.log(OK);
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
