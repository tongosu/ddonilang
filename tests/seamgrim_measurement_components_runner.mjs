#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_measurement_components: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const LESSON_ID = "rep_phys_pendulum_wave1_v1";

function assert(condition, message) {
  if (!condition) throw new Error(message);
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
    const ctx = canvas.getContext("2d");
    if (!ctx || canvas.width <= 0 || canvas.height <= 0) return { width: rect.width, height: rect.height, colors: 0 };
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 4000) * 4);
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 12) break;
    }
    return { width: rect.width, height: rect.height, colors: colors.size };
  });
}

async function readState(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const report = window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null;
    const space = window.__SEAMGRIM_LAST_VIEWS__?.space2d ?? null;
    return {
      root: { ...(root?.dataset ?? {}) },
      report,
      space,
      hash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      toggles: {
        angle: document.querySelector("#chk-view-plan-angle-measure") instanceof HTMLInputElement,
        length: document.querySelector("#chk-view-plan-length-measure") instanceof HTMLInputElement,
        vector: document.querySelector("#chk-view-plan-vector-measure") instanceof HTMLInputElement,
        axes: document.querySelector("#chk-view-plan-axes-measure") instanceof HTMLInputElement,
        trail: document.querySelector("#chk-view-plan-trail-measure") instanceof HTMLInputElement,
      },
    };
  });
}

function componentByKind(report, kind) {
  return (Array.isArray(report?.measure_components) ? report.measure_components : [])
    .find((row) => String(row?.kind ?? "") === kind);
}

async function assertToggleHidesOnlyView(page, selector, role, visibilityKey, baseHash, baseShapeCount) {
  await page.locator(selector).uncheck();
  await page.waitForFunction(
    ([roleText, key]) => {
      const root = document.querySelector("#screen-run");
      const roles = String(root?.dataset?.mainViewRoles ?? "");
      let visibility = {};
      try {
        visibility = JSON.parse(root?.dataset?.viewPlanMeasureVisibility ?? "{}");
      } catch (_) {
        visibility = {};
      }
      return !roles.includes(roleText) && visibility[key] === false;
    },
    [role, visibilityKey],
    { timeout: 10_000 },
  );
  const off = await readState(page);
  assert(off.hash === baseHash, `${visibilityKey} toggle changed state_hash`);
  assert(off.root.viewPlanStateHashParticipation === "0", `${visibilityKey} entered state_hash`);
  assert(Number(off.root.mainShapeCount ?? 0) < baseShapeCount, `${visibilityKey} did not remove any visible shapes`);
  await page.locator(selector).check();
  await page.waitForFunction(
    ([roleText, key]) => {
      const root = document.querySelector("#screen-run");
      const roles = String(root?.dataset?.mainViewRoles ?? "");
      let visibility = {};
      try {
        visibility = JSON.parse(root?.dataset?.viewPlanMeasureVisibility ?? "{}");
      } catch (_) {
        visibility = {};
      }
      return roles.includes(roleText) && visibility[key] === true;
    },
    [role, visibilityKey],
    { timeout: 10_000 },
  );
  const on = await readState(page);
  assert(on.hash === baseHash, `${visibilityKey} toggle restore changed state_hash`);
  assert(Number(on.root.mainShapeCount ?? 0) >= baseShapeCount, `${visibilityKey} did not restore visible shapes`);
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
    await page.waitForFunction(() => {
      const root = document.querySelector("#screen-run");
      const report = window.__SEAMGRIM_VIEW_PLAN_RUNTIME__;
      const roles = String(root?.dataset?.mainViewRoles ?? "");
      return root?.dataset?.viewPlanRuntime === "1"
        && root?.dataset?.mainVisualMode === "space2d"
        && root?.dataset?.mainViewSource === "view_plan"
        && roles.includes("angle_measure")
        && roles.includes("length_measure")
        && roles.includes("vector_arrow")
        && roles.includes("axis_ticks")
        && roles.includes("trajectory_trail")
        && Array.isArray(report?.measure_components)
        && report.measure_components.length >= 5;
    }, null, { timeout: 30_000 });

    await pauseRunningOrAcceptComplete(page);
    await page.waitForFunction(() => ["paused", "done"].includes(document.querySelector("#screen-run")?.dataset?.engineStatus ?? ""), null, { timeout: 10_000 });
    await page.waitForTimeout(250);

    const initial = await readState(page);
    const canvas = await readCanvasEvidence(page);
    assert(canvas && canvas.width >= 400 && canvas.height >= 240 && canvas.colors > 4, `canvas invalid: ${JSON.stringify(canvas)}`);
    assert(initial.hash && initial.hash !== "-", "state hash missing");
    assert(initial.root.viewPlanStateHashParticipation === "0", "view plan must stay outside state_hash");
    assert(String(initial.root.viewPlanHash ?? "").startsWith("candidate:fnv1a32:view_plan:"), "view hash candidate missing");
    assert(
      String(initial.root.worldDrawPayloadHashCandidate ?? "").startsWith("candidate:fnv1a32:world_draw_payload:"),
      "world draw payload hash candidate missing",
    );
    assert(initial.report?.state_hash_participation === false, "runtime report entered state_hash");
    assert(initial.report?.diagnostics?.length === 0, `unexpected diagnostics: ${JSON.stringify(initial.report?.diagnostics)}`);
    assert(Object.values(initial.toggles).every(Boolean), `measure toggles missing: ${JSON.stringify(initial.toggles)}`);

    const expected = [
      ["angle", "rad", "각도재기"],
      ["length", "m", "길이재기"],
      ["vector", "m/s", "벡터 화살표"],
      ["axes", "m", "좌표축"],
      ["trajectory", "m", "궤적선"],
    ];
    for (const [kind, unit, accessibilityToken] of expected) {
      const component = componentByKind(initial.report, kind);
      assert(component, `${kind} component missing`);
      assert(component.visible === true, `${kind} component not visible`);
      assert(component.state_hash_participation === false, `${kind} component entered state_hash`);
      assert(String(component.unit ?? "") === unit, `${kind} unit mismatch: ${JSON.stringify(component)}`);
      assert(String(component.accessibility_name ?? "").includes(accessibilityToken), `${kind} accessibility name missing`);
      assert(Array.isArray(component.object_ids) && component.object_ids.length > 0, `${kind} object ids missing`);
    }
    const shapeRoles = new Set((Array.isArray(initial.space?.shapes) ? initial.space.shapes : []).map((shape) => String(shape?.role ?? "")));
    for (const role of ["angle_measure_arc", "length_measure_line", "vector_arrow", "axis_ticks", "trajectory_trail"]) {
      assert(shapeRoles.has(role), `${role} shape role missing`);
    }

    const baseHash = initial.hash;
    const baseShapeCount = Number(initial.root.mainShapeCount ?? 0);
    const toggles = [
      ["#chk-view-plan-angle-measure", "angle_measure", "angle"],
      ["#chk-view-plan-length-measure", "length_measure", "length"],
      ["#chk-view-plan-vector-measure", "vector_arrow", "vector"],
      ["#chk-view-plan-axes-measure", "axis_ticks", "axes"],
      ["#chk-view-plan-trail-measure", "trajectory_trail", "trail"],
    ];
    for (const [selector, role, key] of toggles) {
      await assertToggleHidesOnlyView(page, selector, role, key, baseHash, baseShapeCount);
    }

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
