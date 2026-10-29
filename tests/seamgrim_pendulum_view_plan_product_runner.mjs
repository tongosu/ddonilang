#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_pendulum_view_plan_product: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LESSON_ID = "rep_phys_pendulum_wave1_v1";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function near(a, b, eps = 1e-6) {
  return Math.abs(Number(a) - Number(b)) <= eps;
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
      const rawPath = decodeURIComponent(url.pathname === "/" ? PROJECT_URL : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
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
      && /\/(?:graph|table|space2d|maegim_control)\.(?:json|detjson)$/i.test(pathname)
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

function installFailureCapture(page, failures) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    if (String(msg.text() ?? "").includes("Failed to load resource")) return;
    failures.push(`console error: ${msg.text()}`);
  });
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`response ${res.status()}: ${res.url()}`);
  });
}

async function readCanvasEvidence(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']");
    if (!(canvas instanceof HTMLCanvasElement)) return null;
    const rect = canvas.getBoundingClientRect();
    const readRect = (selector) => {
      const row = document.querySelector(selector)?.getBoundingClientRect();
      return row ? { left: row.left, right: row.right, top: row.top, bottom: row.bottom, width: row.width, height: row.height } : null;
    };
    const readStyle = (selector) => {
      const row = document.querySelector(selector);
      if (!(row instanceof HTMLElement)) return null;
      const style = getComputedStyle(row);
      return {
        display: style.display,
        gridTemplateColumns: style.gridTemplateColumns,
        gridTemplateRows: style.gridTemplateRows,
        width: style.width,
        height: style.height,
      };
    };
    const ctx = canvas.getContext("2d");
    if (!ctx || canvas.width <= 0 || canvas.height <= 0) return { width: rect.width, height: rect.height, colors: 0 };
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 4000) * 4);
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 12) break;
    }
    return {
      width: rect.width,
      height: rect.height,
      left: rect.left,
      right: rect.right,
      viewportWidth: window.innerWidth,
      colors: colors.size,
      rects: {
        screen: readRect("#screen-run"),
        main: readRect("#screen-run .run-main"),
        layout: readRect("#screen-run .run-layout"),
        visual: readRect("#screen-run .run-visual-column"),
        bogae: readRect("#screen-run .bogae-area"),
        frame: readRect("#screen-run .bogae-frame"),
        dotbogi: readRect("#screen-run .dotbogi-panel.subpanel"),
      },
      styles: {
        layout: readStyle("#screen-run .run-layout"),
        visual: readStyle("#screen-run .run-visual-column"),
        bogae: readStyle("#screen-run .bogae-area"),
        frame: readStyle("#screen-run .bogae-frame"),
      },
    };
  });
}

async function waitForPausedWorldDraw(page) {
  await page.waitForFunction(() => ["paused", "done"].includes(document.querySelector("#screen-run")?.dataset?.engineStatus ?? ""), null, { timeout: 10_000 });
  let previous = "";
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const current = await page.evaluate(() => document.querySelector("#screen-run")?.dataset?.worldDrawPayloadHashCandidate ?? "");
    if (current && current === previous) return current;
    previous = current;
    await page.waitForTimeout(50);
  }
  throw new Error("world draw payload did not stabilize after pause");
}

async function pauseRunningOrAcceptComplete(page) {
  const status = await page.evaluate(() => document.querySelector("#screen-run")?.dataset?.engineStatus ?? "");
  if (status === "running") {
    const pause = page.locator("#btn-pause");
    assert(await pause.isEnabled(), "pause must be enabled while execution is running");
    await pause.click();
  } else {
    assert(["paused", "done"].includes(status), `unexpected execution lifecycle before evidence: ${status}`);
  }
}

async function readState(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const report = window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null;
    const obsBody = document.querySelector("#run-observe-output-body");
    let payload = [];
    try {
      payload = JSON.parse(obsBody?.dataset?.observationPayloadJson ?? "[]");
    } catch (_) {
      payload = [];
    }
    return {
      root: { ...(root?.dataset ?? {}) },
      report,
      payload,
      hash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      cameraOptions: Array.from(document.querySelectorAll("#select-view-plan-camera-policy option")).map((node) => node.value),
      source: document.querySelector("#run-ddn-preview")?.value ?? "",
    };
  });
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
    installFailureCapture(page, failures);
    await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${LESSON_ID}`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
    try {
      await page.waitForFunction(() => {
        const root = document.querySelector("#screen-run");
        const report = window.__SEAMGRIM_VIEW_PLAN_RUNTIME__;
        return root?.dataset?.viewPlanRuntime === "1"
          && root?.dataset?.mainVisualMode === "space2d"
          && root?.dataset?.viewPlanBindingMode === "explicit"
          && root?.dataset?.mainViewSource === "view_plan"
          && Number(root?.dataset?.mainShapeCount ?? 0) >= 7
          && report?.binding_mode === "explicit"
          && report?.graph_current_point;
      }, null, { timeout: 30_000 });
    } catch (error) {
      const diagnostic = await page.evaluate(() => ({
        root: { ...(document.querySelector("#screen-run")?.dataset ?? {}) },
        report: window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null,
      }));
      diagnostic.failures = failures;
      throw new Error(`pendulum view-plan readiness timeout: ${JSON.stringify(diagnostic)}`, { cause: error });
    }

    await pauseRunningOrAcceptComplete(page);
    await waitForPausedWorldDraw(page);

    const initial = await readState(page);
    const canvas = await readCanvasEvidence(page);
    assert(canvas && canvas.width >= 400 && canvas.height >= 240 && canvas.colors > 4, `pendulum canvas invalid: ${JSON.stringify(canvas)}`);
    assert(initial.source.includes("theta <- theta + omega * dt"), "pendulum DDN source not loaded");
    assert(initial.cameraOptions.join(",") === "fixed,fit,follow", `camera policies missing: ${initial.cameraOptions}`);
    assert(initial.root.viewPlanStateHashParticipation === "0", "view plan must stay outside state_hash");
    assert(initial.root.mainViewRoles.includes("semantic_binding"), "semantic binding role missing");
    assert(initial.root.mainViewRoles.includes("angle_measure"), "angle measure role missing");
    assert(initial.root.mainViewRoles.includes("same_tick_graph_cursor"), "same tick graph cursor role missing");
    assert(initial.report?.diagnostics?.length === 0, `view plan diagnostics not empty: ${JSON.stringify(initial.report?.diagnostics)}`);
    assert(initial.report?.angle_label?.includes("rad"), `angle label missing unit: ${JSON.stringify(initial.report)}`);
    assert(near(initial.report?.graph_current_point?.x, initial.report?.time), "graph cursor x must match same tick time");
    assert(near(initial.report?.graph_current_point?.y, initial.report?.theta), "graph cursor y must match same tick theta");
    assert(initial.report?.graph_current_point?.y_unit === "rad", "graph cursor theta unit missing");
    for (const id of ["theta", "omega", "bob_x", "bob_y", "t"]) {
      assert(initial.payload.some((row) => row.id === id || row.source_ref === `observation.${id}`), `observation payload missing ${id}`);
    }
    assert(initial.payload.some((row) => row.id === "theta" && row.unit === "rad"), "theta observation unit must be rad");

    const baseHash = initial.hash;
    const baseShapeCount = Number(initial.root.mainShapeCount ?? 0);
    assert(initial.root.viewPlanAngleMeasureVisible === "1", "angle measure must be visible by default");
    await page.locator("#chk-view-plan-angle-measure").uncheck();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.viewPlanAngleMeasureVisible === "0");
    const angleOff = await readState(page);
    assert(angleOff.hash === baseHash, "state_hash changed after angle measure off");
    assert(angleOff.root.viewPlanStateHashParticipation === "0", "angle measure toggle entered state_hash");
    assert(!String(angleOff.root.mainViewRoles ?? "").includes("angle_measure"), "angle measure role stayed active while hidden");
    assert(Number(angleOff.root.mainShapeCount ?? 0) < baseShapeCount, "angle measure shapes did not hide");
    await page.locator("#chk-view-plan-angle-measure").check();
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.viewPlanAngleMeasureVisible === "1");
    const angleOn = await readState(page);
    assert(angleOn.hash === baseHash, "state_hash changed after angle measure on");
    assert(
      String(angleOn.root.mainViewRoles ?? "").includes("angle_measure"),
      `angle measure role did not return: roles=${angleOn.root.mainViewRoles} report=${JSON.stringify(angleOn.report)}`,
    );
    assert(Number(angleOn.root.mainShapeCount ?? 0) >= baseShapeCount, "angle measure shapes did not return");

    const baseWorldDrawPayloadHash = angleOn.root.worldDrawPayloadHashCandidate;
    assert(
      String(baseWorldDrawPayloadHash ?? "").startsWith("candidate:fnv1a32:world_draw_payload:"),
      `world draw payload hash candidate missing: ${baseWorldDrawPayloadHash}`,
    );
    const baseFitViewPlanHash = angleOn.root.viewPlanHash;
    for (const policy of ["fixed", "follow", "fit"]) {
      await page.locator("#select-view-plan-camera-policy").selectOption(policy);
      await page.waitForFunction((expected) => document.querySelector("#screen-run")?.dataset?.viewPlanCameraPolicy === expected, policy);
      const current = await readState(page);
      assert(current.hash === baseHash, `state_hash changed after camera ${policy}: ${current.hash} != ${baseHash}`);
      assert(current.root.worldDrawPayloadHashCandidate === baseWorldDrawPayloadHash, `world draw payload changed after camera ${policy}`);
      assert(current.root.viewPlanCameraPolicy === policy, `camera policy not applied: ${policy}`);
      assert(current.root.viewPlanStateHashParticipation === "0", `camera ${policy} entered state_hash`);
    }
    const roundtrip = await readState(page);
    assert(roundtrip.root.viewPlanHash === baseFitViewPlanHash, "fit camera frame series/hash not replay-stable");

    const stepButton = page.locator("#btn-step");
    if (await stepButton.isEnabled()) {
      await stepButton.click();
      await page.waitForTimeout(600);
      const stepped = await readState(page);
      assert(stepped.report?.time !== initial.report?.time || stepped.report?.theta !== initial.report?.theta, "step did not advance view plan same-tick report");
      assert(near(stepped.report?.graph_current_point?.x, stepped.report?.time), "stepped graph cursor x mismatch");
      assert(near(stepped.report?.graph_current_point?.y, stepped.report?.theta), "stepped graph cursor y mismatch");
    } else {
      assert(initial.root.engineStatus === "done", `disabled step requires completed execution: ${initial.root.engineStatus}`);
    }

    await page.setViewportSize({ width: 390, height: 844 });
    await page.waitForTimeout(250);
    const mobileCanvas = await readCanvasEvidence(page);
    assert(mobileCanvas && mobileCanvas.width >= 300 && mobileCanvas.height >= 160 && mobileCanvas.colors > 4, `mobile pendulum canvas invalid: ${JSON.stringify(mobileCanvas)}`);
    assert(
      mobileCanvas.left >= -2 && mobileCanvas.right <= mobileCanvas.viewportWidth + 2,
      `mobile pendulum canvas is clipped outside viewport: ${JSON.stringify(mobileCanvas)}`,
    );

    assert(failures.length === 0, failures.join("\n"));
    process.stdout.write(`${OK}\n`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
