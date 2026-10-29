#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave2c_first_success: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const WORLDS = [
  {
    id: "rep_phys_pendulum_wave1_v1",
    label: "pendulum",
    roles: ["angle_measure", "length_measure"],
    controls: [{ name: "theta0", value: "0.45" }],
    observationKeys: ["theta", "omega"],
    primary: "space2d",
  },
  {
    id: "rep_phys_projectile_xy_v1",
    label: "projectile",
    roles: ["trajectory_trail", "vector_arrow"],
    controls: [{ name: "초기속도x", value: "14" }],
    observationKeys: ["x", "y", "vx", "vy"],
    primary: "space2d",
  },
  {
    id: "rep_math_function_line_v1",
    label: "linear",
    roles: ["graph_series", "axis_ticks"],
    controls: [{ name: "기울기", value: "3.5" }],
    observationKeys: ["x", "y"],
    primary: "graph",
  },
];

const BROKEN_LEGACY_DDN = `설정 {
  제목: "Wave2C 진단 예제".
  설명: "구식 블록 표면을 제품 편집기에서 고칩니다.".
}.

채비: {
  계수:수 <- 2.
  t:수 <- 0.
  y:수 <- 0.
}.

(매마디)마다 {
  y <- 계수 * t.
  y 보여주기.
  t <- t + 1.
}.`;

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
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|detjson|md)$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function installFailureCapture(page, failures, label) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    const text = String(msg.text() ?? "");
    if (text.includes("Failed to load resource")) return;
    failures.push(`${label}: console error: ${text}`);
  });
  page.on("pageerror", (err) => failures.push(`${label}: pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`${label}: request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`${label}: response ${res.status()}: ${res.url()}`);
  });
}

async function waitRunReady(page, lessonId) {
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction((id) => {
    const root = document.querySelector("#screen-run");
    const source = String(document.querySelector("#run-ddn-preview")?.value ?? "");
    const hash = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    return new URL(location.href).searchParams.get("lesson") === id
      && source.trim().length > 80
      && hash.includes("blake3:")
      && (
        String(root?.dataset?.mainViewRoles ?? "").trim()
        || Number(root?.dataset?.mainGraphSeriesCount ?? 0) > 0
        || Number(root?.dataset?.mainTrajectorySegmentCount ?? 0) > 0
      );
  }, lessonId, { timeout: 35_000 });
}

async function waitRunAnyReady(page) {
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction(() => {
    const source = String(document.querySelector("#run-ddn-preview")?.value ?? "");
    const hash = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    return source.trim().length > 50 && hash.includes("blake3:");
  }, null, { timeout: 35_000 });
}

async function clickRunIfEnabled(page) {
  const button = page.locator("#btn-run");
  if (await button.isEnabled()) {
    await button.click();
    return true;
  }
  return false;
}

async function waitPrimaryEvidence(page, world) {
  await clickRunIfEnabled(page);
  await page.waitForFunction((target) => {
    const root = document.querySelector("#screen-run");
    const space = window.__SEAMGRIM_MAIN_VISUAL__?.space2d ?? window.__SEAMGRIM_LAST_VIEWS__?.space2d ?? null;
    const roles = new Set([
      ...String(root?.dataset?.mainViewRoles ?? "").split(",").filter(Boolean),
      ...(Array.isArray(space?.shapes) ? space.shapes : []).map((shape) => String(shape?.role ?? "")).filter(Boolean),
    ]);
    if (target.primary === "graph") {
      return root?.dataset?.primaryViewContract === "1"
        && root?.dataset?.primaryViewHeuristicSelection === "0"
        && root?.dataset?.mainVisualMode === "graph"
        && Number(root?.dataset?.mainGraphSeriesCount ?? 0) >= 1
        && Number(root?.dataset?.mainGraphPointCount ?? 0) >= 3;
    }
    return target.roles.every((role) => roles.has(role))
      || (target.label === "projectile" && roles.has("trajectory_trail") && roles.has("vector_arrow"));
  }, { label: world.label, primary: world.primary, roles: world.roles }, { timeout: 35_000 });
}

async function readRunEvidence(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const payloadText = String(document.querySelector("#run-observe-output-body")?.dataset?.observationPayloadJson ?? "{}");
    let payload = {};
    try {
      payload = JSON.parse(payloadText);
    } catch (_) {
      payload = {};
    }
    const space = window.__SEAMGRIM_MAIN_VISUAL__?.space2d ?? window.__SEAMGRIM_LAST_VIEWS__?.space2d ?? null;
    const ddn = document.querySelector("#run-ddn-preview");
    const ddnRect = ddn?.getBoundingClientRect?.();
    return {
      shellSpace: root?.dataset?.shellSpace ?? "",
      launchKind: root?.dataset?.launchKind ?? "",
      primaryView: root?.dataset?.primaryView ?? "",
      primaryViewContract: root?.dataset?.primaryViewContract ?? "",
      primaryViewHeuristicSelection: root?.dataset?.primaryViewHeuristicSelection ?? "",
      mainVisualMode: root?.dataset?.mainVisualMode ?? "",
      mainViewRoles: String(root?.dataset?.mainViewRoles ?? "").split(",").filter(Boolean),
      shapeRoles: Array.from(new Set((Array.isArray(space?.shapes) ? space.shapes : []).map((shape) => String(shape?.role ?? "")).filter(Boolean))),
      mainGraphSeriesCount: Number(root?.dataset?.mainGraphSeriesCount ?? 0),
      mainGraphPointCount: Number(root?.dataset?.mainGraphPointCount ?? 0),
      mainTrajectorySegmentCount: Number(root?.dataset?.mainTrajectorySegmentCount ?? 0),
      semanticCursorRuntime: root?.dataset?.semanticCursorRuntime ?? "",
      semanticCursorStateHashParticipation: root?.dataset?.semanticCursorStateHashParticipation ?? "",
      semanticCursorSourceAvailable: root?.dataset?.semanticCursorSourceAvailable ?? "",
      liveTraceReactionRuntime: root?.dataset?.liveTraceReactionRuntime ?? "",
      liveTraceReactionLineageId: root?.dataset?.liveTraceReactionLineageId ?? "",
      traceReactionStateHashParticipation: root?.dataset?.traceReactionStateHashParticipation ?? "",
      viewPlanStateHashParticipation: root?.dataset?.viewPlanStateHashParticipation ?? "",
      stateHash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      source: ddn?.value ?? "",
      ddnVisible: Boolean(ddnRect && ddnRect.width > 240 && ddnRect.height > 100),
      ddnEditable: ddn instanceof HTMLTextAreaElement && ddn.readOnly === false && ddn.disabled === false,
      payload,
    };
  });
}

function assertWorldEvidence(world, evidence, phase) {
  assert(evidence.stateHash.includes("blake3:"), `${world.label} ${phase}: state hash missing`);
  assert(evidence.ddnVisible && evidence.ddnEditable, `${world.label} ${phase}: desktop DDN source not readable/editable`);
  const visibleRoles = new Set([...(evidence.mainViewRoles ?? []), ...(evidence.shapeRoles ?? [])]);
  if (world.primary !== "graph") {
    for (const role of world.roles) {
      assert(visibleRoles.has(role), `${world.label} ${phase}: role missing ${role}; ${JSON.stringify({
        mainViewRoles: evidence.mainViewRoles,
        shapeRoles: evidence.shapeRoles,
        mainVisualMode: evidence.mainVisualMode,
        primaryView: evidence.primaryView,
        primaryViewContract: evidence.primaryViewContract,
        sourceHead: evidence.source.slice(0, 220),
      })}`);
    }
  }
  if (world.primary === "graph") {
    assert(evidence.primaryViewContract === "1", `${world.label}: primary view contract missing`);
    assert(evidence.primaryViewHeuristicSelection === "0", `${world.label}: heuristic primary fallback used`);
    assert(evidence.mainVisualMode === "graph", `${world.label}: graph primary view missing`);
    assert(evidence.mainGraphSeriesCount >= 1 && evidence.mainGraphPointCount >= 3, `${world.label}: graph evidence missing ${JSON.stringify({
      mainGraphSeriesCount: evidence.mainGraphSeriesCount,
      mainGraphPointCount: evidence.mainGraphPointCount,
      mainViewRoles: evidence.mainViewRoles,
      shapeRoles: evidence.shapeRoles,
      primaryView: evidence.primaryView,
      mainVisualMode: evidence.mainVisualMode,
    })}`);
  } else {
    assert(
      evidence.mainTrajectorySegmentCount >= 1
        || [...visibleRoles].some((role) => role.includes("measure") || role === "trajectory_trail" || role === "vector_arrow"),
      `${world.label}: space2d primary evidence missing`,
    );
  }
  const payloadText = JSON.stringify(evidence.payload);
  for (const key of world.observationKeys) {
    assert(payloadText.includes(key) || evidence.source.includes(key), `${world.label} ${phase}: observation key missing ${key}`);
  }
}

async function openFromCard(page, baseUrl, world, action) {
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector(`#screen-browse .lesson-card[data-lesson-id='${world.id}']`, { state: "visible", timeout: 20_000 });
  const selector = action === "lab"
    ? `#screen-browse .lesson-card[data-lesson-id='${world.id}'] [data-launch-action='lab']`
    : `#screen-browse .lesson-card[data-lesson-id='${world.id}'] .card-launch-primary`;
  await page.locator(selector).click();
  await waitRunReady(page, world.id);
  await waitPrimaryEvidence(page, world);
  return readRunEvidence(page);
}

async function verifyRepresentativeOpenAndLab(page, baseUrl) {
  for (const world of WORLDS) {
    const openEvidence = await openFromCard(page, baseUrl, world, "learn");
    assert(openEvidence.shellSpace === "learn", `${world.label}: 수업 열기 shell mismatch ${openEvidence.shellSpace}`);
    assertWorldEvidence(world, openEvidence, "learn-open");

    const labEvidence = await openFromCard(page, baseUrl, world, "lab");
    assert(labEvidence.shellSpace === "lab", `${world.label}: 실험실에서 바꾸기 shell mismatch ${labEvidence.shellSpace}`);
    assert(labEvidence.stateHash.includes("blake3:"), `${world.label} lab-open: state hash missing`);
    assert(labEvidence.ddnVisible && labEvidence.ddnEditable, `${world.label} lab-open: DDN source not readable/editable`);
    assert(labEvidence.source.trim().length > 80, `${world.label} lab-open: source missing`);
  }
}

async function changeControlAndVerify(page, baseUrl, world) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${world.id}`, { waitUntil: "domcontentloaded" });
  await waitRunReady(page, world.id);
  await waitPrimaryEvidence(page, world);
  const before = await readRunEvidence(page);
  assertWorldEvidence(world, before, "before-change");

  const control = world.controls[0];
  const input = page.locator(`input[data-control-name='${control.name}']`).first();
  await input.waitFor({ state: "visible", timeout: 20_000 });
  await input.evaluate((node, value) => {
    node.value = String(value);
    node.dispatchEvent(new Event("input", { bubbles: true }));
    node.dispatchEvent(new Event("change", { bubbles: true }));
  }, control.value);
  await page.waitForFunction(({ oldHash, controlName, controlValue }) => {
    const source = String(document.querySelector("#run-ddn-preview")?.value ?? "");
    const hash = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    const status = String(document.querySelector("#screen-run")?.dataset?.engineStatus ?? "");
    return source.includes(controlName)
      && source.includes(String(controlValue))
      && hash.includes("blake3:")
      && hash !== oldHash
      && ["running", "paused", "done"].includes(status);
  }, { oldHash: before.stateHash, controlName: control.name, controlValue: control.value }, { timeout: 35_000 });
  await waitPrimaryEvidence(page, world);
  const after = await readRunEvidence(page);
  assert(after.stateHash !== before.stateHash, `${world.label}: control change did not change runtime hash`);
  assert(after.source.includes(control.name) && after.source.includes(control.value), `${world.label}: changed source token missing`);
  if (after.semanticCursorRuntime === "1") {
    assert(after.semanticCursorStateHashParticipation !== "1", `${world.label}: semantic cursor leaked into state hash`);
  }
  assert(after.viewPlanStateHashParticipation !== "1", `${world.label}: view plan leaked into state hash`);
  assert(after.traceReactionStateHashParticipation !== "1", `${world.label}: trace reaction leaked into state hash`);
  if (world.label === "pendulum") {
    assert(after.semanticCursorRuntime === "1", "pendulum: semantic cursor runtime missing");
    assert(after.liveTraceReactionRuntime === "1", "pendulum: live trace reaction runtime missing");
    assert(after.liveTraceReactionLineageId && after.liveTraceReactionLineageId !== before.liveTraceReactionLineageId, "pendulum: lineage did not reset after control change");
    assert(after.semanticCursorSourceAvailable === "1", "pendulum: semantic source link unavailable");
  }
  assertWorldEvidence(world, after, "after-change");
}

async function verifyMobileDdnToggle(context, baseUrl) {
  const page = await context.newPage();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=rep_phys_projectile_xy_v1`, { waitUntil: "domcontentloaded" });
  await waitRunReady(page, "rep_phys_projectile_xy_v1");
  const before = await page.evaluate(() => ({
    open: document.querySelector("#screen-run")?.dataset?.mobileDdnOpen ?? "",
    expanded: document.querySelector("#btn-run-mobile-ddn-toggle")?.getAttribute("aria-expanded") ?? "",
    sourceLength: String(document.querySelector("#run-ddn-preview")?.value ?? "").trim().length,
  }));
  assert(before.open === "0" && before.expanded === "false", `mobile DDN should start closed: ${JSON.stringify(before)}`);
  assert(before.sourceLength > 80, "mobile DDN source not loaded");
  await page.locator("#btn-run-mobile-ddn-toggle").click();
  const after = await page.evaluate(() => ({
    open: document.querySelector("#screen-run")?.dataset?.mobileDdnOpen ?? "",
    expanded: document.querySelector("#btn-run-mobile-ddn-toggle")?.getAttribute("aria-expanded") ?? "",
    buttonText: document.querySelector("#btn-run-mobile-ddn-toggle")?.textContent?.trim() ?? "",
  }));
  assert(after.open === "1" && after.expanded === "true" && after.buttonText === "보기", `mobile DDN toggle did not open: ${JSON.stringify(after)}`);
  await page.close();
}

async function verifyProductDiagnosticWorkflow(page, baseUrl) {
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.locator("#screen-browse .main-shell-tab[data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.locator("#btn-editor-world-code").click();
  await page.locator("#ddn-textarea").fill(BROKEN_LEGACY_DDN);
  await page.locator("#ddn-textarea").dispatchEvent("input");
  await page.waitForFunction(() => document.querySelector("#editor-readiness-card")?.dataset?.stage === "autofix", null, { timeout: 20_000 });
  const diagnostic = await page.evaluate(() => {
    const panel = document.querySelector("#editor-readiness-diagnostic");
    const text = (layer) => panel?.querySelector(`[data-diagnostic-layer='${layer}']`)?.textContent?.trim() ?? "";
    return {
      stage: document.querySelector("#editor-readiness-card")?.dataset?.stage ?? "",
      fixitReady: document.querySelector("#editor-fixit-card")?.dataset?.ready ?? "",
      what: text("what"),
      where: text("where"),
      how: text("how"),
      summary: document.querySelector("#editor-fixit-summary")?.textContent?.trim() ?? "",
    };
  });
  assert(diagnostic.stage === "autofix" && diagnostic.fixitReady === "true", `diagnostic fixit not ready: ${JSON.stringify(diagnostic)}`);
  assert(diagnostic.what.includes("구식") && diagnostic.where.includes("세계 코드") && diagnostic.how.includes("수정"), `three-layer diagnostic missing: ${JSON.stringify(diagnostic)}`);
  assert(diagnostic.summary.includes("진단") && diagnostic.summary.includes("수정"), `fixit summary incomplete: ${diagnostic.summary}`);

  await page.locator("#btn-run-from-editor").click();
  await waitRunAnyReady(page);
  const preflight = await readRunEvidence(page);
  assert(preflight.source.includes("채비 {") && !preflight.source.includes("채비: {"), "editor run preflight did not use fixed source");
  assert(preflight.stateHash.includes("blake3:"), "fixed preflight run did not execute");

  await page.locator("#screen-run .main-shell-tab[data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 15_000 });
  await page.locator("#ddn-textarea").fill(BROKEN_LEGACY_DDN);
  await page.locator("#ddn-textarea").dispatchEvent("input");
  await page.waitForFunction(() => document.querySelector("#editor-fixit-card")?.dataset?.ready === "true", null, { timeout: 20_000 });
  await page.locator("#btn-editor-fixit-apply").click();
  await page.waitForFunction(() => {
    const source = String(document.querySelector("#ddn-textarea")?.value ?? "");
    return source.includes("채비 {") && !source.includes("채비: {")
      && document.querySelector("#editor-readiness-card")?.dataset?.stage === "ready";
  }, null, { timeout: 20_000 });
  await page.locator("#btn-run-from-editor").click();
  await waitRunAnyReady(page);
  const fixed = await readRunEvidence(page);
  assert(fixed.source.includes("채비 {") && fixed.stateHash.includes("blake3:"), "fixed editor source did not run");
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, locale: "ko-KR" });
    const page = await context.newPage();
    installFailureCapture(page, failures, "desktop");

    await verifyRepresentativeOpenAndLab(page, baseUrl);
    for (const world of WORLDS) {
      await changeControlAndVerify(page, baseUrl, world);
    }
    await verifyProductDiagnosticWorkflow(page, baseUrl);
    await verifyMobileDdnToggle(context, baseUrl);

    assert(failures.length === 0, failures.join("\n"));
    await page.close();
    await context.close();
  } finally {
    await browser.close();
    await closeServer(server);
  }
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
