#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_sealed_trace_reaction: ok";
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
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 5000) * 4);
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 14) break;
    }
    return { width: rect.width, height: rect.height, colors: colors.size };
  });
}

async function readState(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    return {
      root: { ...(root?.dataset ?? {}) },
      viewPlan: window.__SEAMGRIM_VIEW_PLAN_RUNTIME__ ?? null,
      reaction: window.__SEAMGRIM_TRACE_REACTION_RUNTIME__ ?? null,
      hash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      shapeCount: Number(root?.dataset?.mainShapeCount ?? 0),
      source: document.querySelector("#run-ddn-preview")?.value ?? "",
    };
  });
}

async function scrubToTick(page, tick) {
  await page.evaluate((targetTick) => {
    const input = document.querySelector("#range-dock-time-cursor");
    if (!(input instanceof HTMLInputElement)) throw new Error("dock time cursor input missing");
    input.value = String(targetTick);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  }, tick);
  await page.waitForFunction(
    (targetTick) => window.__SEAMGRIM_TRACE_REACTION_RUNTIME__?.tick_id === targetTick,
    tick,
    { timeout: 10_000 },
  );
  return readState(page);
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
      const reaction = window.__SEAMGRIM_TRACE_REACTION_RUNTIME__;
      return root?.dataset?.traceReactionRuntime === "1"
        && root?.dataset?.mainVisualMode === "space2d"
        && reaction?.schema === "seamgrim.trace_reaction.runtime.v1"
        && reaction?.tick_id >= 60;
    }, null, { timeout: 30_000 });

    const pause = page.locator("#btn-pause");
    if (await pause.isEnabled()) await pause.click();
    await page.waitForTimeout(150);

    const initial = await scrubToTick(page, 1);
    const canvas = await readCanvasEvidence(page);
    assert(canvas && canvas.width >= 400 && canvas.height >= 240 && canvas.colors > 4, `trace canvas invalid: ${JSON.stringify(canvas)}`);
    assert(initial.root.traceReactionSchema === "seamgrim.trace_reaction.runtime.v1", "trace reaction schema dataset missing");
    assert(initial.root.traceReactionMode === "sealed_observation_trace", "trace reaction mode mismatch");
    assert(initial.root.traceReactionSealed === "1", "trace must be sealed");
    assert(initial.root.traceReactionLiveOnlineClaim === "0", "trace must not claim live online reaction");
    assert(initial.root.traceReactionConditionEvaluationClaim === "1", "sealed trace must claim offline condition evaluation");
    assert(initial.root.traceReactionScheduleKind === "sealed_observation_trace_evaluator", "evaluator source boundary missing");
    assert(initial.root.traceReactionStateHashParticipation === "0", "trace reaction must stay outside state_hash");
    assert(Number(initial.root.traceReactionObservationSampleCount) >= 60, "sealed observation trace sample count too small");
    assert(String(initial.root.traceReactionObservationTraceHash ?? "").startsWith("candidate:fnv1a32:sealed_observation_trace:"), "sealed observation trace hash missing");
    assert(String(initial.root.traceReactionPredicateEventHash ?? "").startsWith("candidate:fnv1a32:trace_predicate_events:"), "predicate event hash missing");
    assert(initial.root.traceReactionAppliedActions === "", "trace-derived reactions must not be active before an event tick");
    assert(initial.root.traceReactionConditionKinds === "", "condition labels must not be active before an event tick");
    assert(initial.reaction?.resolved_scene_trace?.stage_plan_created === false, "StagePlan must not be created");
    assert(initial.reaction?.resolved_scene_trace?.implicit_lww_allowed === false, "implicit LWW must be false");
    assert(initial.viewPlan?.state_hash_participation === false, "view plan state hash boundary changed");

    const baseStateHash = initial.hash;
    const baseWorldDrawPayloadHash = initial.root.worldDrawPayloadHashCandidate;
    const baseReactionHash = initial.root.traceReactionFrameHash;
    await page.locator("#select-view-plan-camera-policy").selectOption("fixed");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.viewPlanCameraPolicy === "fixed");
    const fixed = await readState(page);
    assert(fixed.hash === baseStateHash, "state_hash changed after camera UI selection");
    assert(
      fixed.root.worldDrawPayloadHashCandidate === baseWorldDrawPayloadHash,
      `world draw payload changed after camera UI selection: ${baseWorldDrawPayloadHash} -> ${fixed.root.worldDrawPayloadHashCandidate}`,
    );
    assert(fixed.root.traceReactionFrameHash === baseReactionHash, "reaction frame changed without a new trace tick");

    const tolerance = await scrubToTick(page, 25);
    assert(tolerance.root.traceReactionConditionKinds === "tolerance_entry", `tolerance label mismatch: ${tolerance.root.traceReactionConditionKinds}; events=${JSON.stringify(tolerance.reaction?.predicate_events?.events?.slice(0, 8) ?? [])}`);
    assert(tolerance.root.traceReactionAppliedActions === "visibility.baseline_dim", `tolerance action mismatch: ${tolerance.root.traceReactionAppliedActions}`);
    assert(tolerance.root.traceReactionUtteranceTicks === "25", `tolerance utterance tick mismatch: ${tolerance.root.traceReactionUtteranceTicks}`);
    assert(tolerance.reaction?.hidden_shape_ids?.includes("measure.angle.theta.baseline"), "tolerance action did not dim baseline");

    const zero = await scrubToTick(page, 26);
    assert(zero.root.traceReactionConditionKinds === "zero_crossing", `zero-crossing label missing: ${zero.root.traceReactionConditionKinds}`);
    assert(zero.root.traceReactionAppliedActions === "highlight.theta_measure", `zero-crossing action mismatch: ${zero.root.traceReactionAppliedActions}`);
    assert(zero.root.traceReactionUtteranceTicks === "26", `zero-crossing utterance tick mismatch: ${zero.root.traceReactionUtteranceTicks}`);
    assert(zero.reaction?.highlighted_shape_ids?.includes("measure.angle.theta.arc"), "zero-crossing highlight did not touch angle arc");

    const extremum = await scrubToTick(page, 50);
    assert(extremum.root.traceReactionConditionKinds === "extremum", `tick 50 labels mismatch: ${extremum.root.traceReactionConditionKinds}`);
    assert(extremum.root.traceReactionAppliedActions === "camera_focus.bob", `tick 50 actions mismatch: ${extremum.root.traceReactionAppliedActions}`);
    assert(extremum.root.traceReactionUtteranceTicks === "50", `tick 50 utterance mismatch: ${extremum.root.traceReactionUtteranceTicks}`);
    assert(extremum.reaction?.camera_focuses?.some((row) => row.target_anchor === "obs.bob"), "camera focus target missing");

    const sign = await scrubToTick(page, 51);
    assert(sign.root.traceReactionConditionKinds === "sign_change", `tick 51 labels mismatch: ${sign.root.traceReactionConditionKinds}`);
    assert(sign.root.traceReactionAppliedActions === "highlight.bob_motion", `tick 51 actions mismatch: ${sign.root.traceReactionAppliedActions}`);
    assert(sign.root.traceReactionUtteranceTicks === "51", `tick 51 utterance mismatch: ${sign.root.traceReactionUtteranceTicks}`);
    assert(sign.reaction?.highlighted_shape_ids?.includes("pendulum.bob"), "sign-change highlight did not touch bob");

    const replayed = await scrubToTick(page, 51);
    assert(replayed.root.traceReactionAppliedActions === sign.root.traceReactionAppliedActions, "scrub replay action sequence changed");
    assert(replayed.root.traceReactionFrameHash === sign.root.traceReactionFrameHash, "scrub replay frame hash changed");
    assert(String(replayed.reaction?.reaction_frame_hash ?? "").startsWith("candidate:fnv1a32:trace_reaction_frame:"), "reaction frame hash missing");

    await page.setViewportSize({ width: 1440, height: 900 });
    await page.waitForTimeout(250);
    const wideCanvas = await readCanvasEvidence(page);
    assert(wideCanvas && wideCanvas.width >= 560 && wideCanvas.height >= 300 && wideCanvas.colors > 4, `1440x900 trace canvas invalid: ${JSON.stringify(wideCanvas)}`);

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
