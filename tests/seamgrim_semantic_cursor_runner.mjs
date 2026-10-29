#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";
import { buildSeamgrimSemanticCursorState } from "../solutions/seamgrim_ui_mvp/ui/semantic_cursor_runtime.js";

const OK = "seamgrim_semantic_cursor: ok";
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

function checkMissingSourceRefDiagnostic() {
  const state = buildSeamgrimSemanticCursorState({
    viewPlanRuntime: {
      views: {
        view_plan: {
          anchors: [
            {
              anchor_id: "obs.theta",
              source_ref: { kind: "observation", key: "theta" },
              label: "각도",
              unit: "rad",
              value_type: "scalar",
            },
          ],
        },
      },
    },
    observation: { all_values: { theta: 0.2 }, values: { theta: 0.2 } },
    selectedAnchorId: "obs.theta",
  });
  assert(state.selected_anchor.source_available === false, "missing source_ref fixture unexpectedly source-available");
  assert(state.diagnostics.some((row) => row.code === "W_SEMANTIC_SOURCE_REF_MISSING"), "missing source_ref diagnostic absent");
  assert(state.diagnostics.every((row) => row.what && row.where && row.fix), "semantic diagnostics need what/where/fix");
}

async function waitForAnchor(page, anchorId, origin = "") {
  await page.waitForFunction(
    ({ anchor, selectedOrigin }) => {
      const root = document.querySelector("#screen-run");
      if (root?.dataset?.semanticCursorSelectedAnchor !== anchor) return false;
      if (selectedOrigin && root?.dataset?.semanticCursorSelectedOrigin !== selectedOrigin) return false;
      return window.__SEAMGRIM_SEMANTIC_CURSOR__?.selected_anchor_id === anchor;
    },
    { anchor: anchorId, selectedOrigin: origin },
    { timeout: 15_000 },
  );
}

async function clickSemantic(page, anchorId, origin) {
  await page.locator(`button[data-semantic-anchor-action='${origin}'][data-anchor-id='${anchorId}']`).click();
  await waitForAnchor(page, anchorId, origin);
}

async function readState(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const textarea = document.querySelector("#run-ddn-preview");
    const selection = textarea instanceof HTMLTextAreaElement
      ? textarea.value.slice(textarea.selectionStart, textarea.selectionEnd)
      : "";
    return {
      root: { ...(root?.dataset ?? {}) },
      cursor: window.__SEAMGRIM_SEMANTIC_CURSOR__ ?? null,
      yAxis: document.querySelector("#select-y-axis")?.value ?? "",
      sourceSelection: selection,
      selectedCards: Array.from(document.querySelectorAll(".run-semantic-anchor-card[data-selected='1']"))
        .map((item) => item.getAttribute("data-anchor-id")),
      panelRect: document.querySelector("[data-semantic-cursor-panel]")?.getBoundingClientRect().toJSON?.() ?? null,
      canvasRect: document.querySelector("[data-testid='lesson-space2d-canvas']")?.getBoundingClientRect().toJSON?.() ?? null,
    };
  });
}

async function main() {
  checkMissingSourceRefDiagnostic();
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    installFailureCapture(page, failures);
    await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${LESSON_ID}`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
    await page.waitForFunction(() => {
      const root = document.querySelector("#screen-run");
      const cursor = window.__SEAMGRIM_SEMANTIC_CURSOR__;
      return root?.dataset?.semanticCursorRuntime === "1"
        && cursor?.schema === "seamgrim.semantic_cursor.v1"
        && Array.isArray(cursor.anchors)
        && cursor.anchors.length === 6
        && cursor.anchors.every((row) => row.ok === true && row.source_available === true)
        && cursor.anchors.find((row) => row.anchor_id === "obs.kinetic_energy")?.value_text?.includes("J/kg");
    }, null, { timeout: 35_000 });

    const pause = page.locator("#btn-pause");
    if (await pause.isEnabled()) await pause.click();
    await page.waitForTimeout(150);

    let state = await readState(page);
    assert(state.root.semanticCursorSelectedAnchor === "obs.theta", "default semantic anchor should be angle");
    assert(state.root.semanticCursorStateHashParticipation === "0", "semantic cursor must not affect state_hash");
    assert(state.cursor.anchors.map((row) => row.anchor_id).join(",") === "obs.theta,obs.angular_velocity,obs.bob,obs.speed,obs.potential_energy,obs.kinetic_energy", "anchor order mismatch");
    assert(state.cursor.anchors.find((row) => row.anchor_id === "obs.kinetic_energy")?.value_text.includes("J/kg"), "kinetic energy value missing unit");

    await clickSemantic(page, "obs.theta", "observe");
    state = await readState(page);
    assert(state.selectedCards.includes("obs.theta"), "observe entry did not select theta");

    await clickSemantic(page, "obs.theta", "graph");
    state = await readState(page);
    assert(state.yAxis === "theta", `theta graph axis mismatch: ${state.yAxis}`);
    assert(state.root.semanticCursorGraphKey === "theta", "theta graph key dataset missing");

    await clickSemantic(page, "obs.theta", "space");
    state = await readState(page);
    assert(Number(state.root.semanticCursorSpaceShapeCount) >= 1, `theta space highlight missing: ${JSON.stringify(state.root)}`);

    await clickSemantic(page, "obs.theta", "source");
    state = await readState(page);
    assert(state.root.semanticCursorSourceStatus === "selected", "theta source span not selected");
    assert(state.sourceSelection.includes("theta <- theta + omega * dt"), `theta source selection mismatch: ${state.sourceSelection}`);

    await clickSemantic(page, "obs.kinetic_energy", "graph");
    state = await readState(page);
    assert(state.yAxis === "kinetic_energy", `kinetic graph axis mismatch: ${state.yAxis}`);
    assert(state.cursor.selected_anchor.value_text.includes("J/kg"), "kinetic semantic value missing");

    await clickSemantic(page, "obs.bob", "space");
    state = await readState(page);
    assert(Number(state.root.semanticCursorSpaceShapeCount) >= 1, "bob space highlight missing");
    assert(Number(state.root.semanticCursorTickId) >= 0, "semantic cursor tick missing");

    await clickSemantic(page, "obs.kinetic_energy", "source");
    state = await readState(page);
    assert(state.sourceSelection.includes("kinetic_energy <- 0.5 * speed * speed"), `kinetic source selection mismatch: ${state.sourceSelection}`);

    await page.setViewportSize({ width: 390, height: 844 });
    await page.waitForTimeout(250);
    state = await readState(page);
    assert(state.panelRect && state.canvasRect, "mobile panel/canvas rect missing");
    const overlaps = !(state.panelRect.right <= state.canvasRect.left
      || state.panelRect.left >= state.canvasRect.right
      || state.panelRect.bottom <= state.canvasRect.top
      || state.panelRect.top >= state.canvasRect.bottom);
    assert(!overlaps, `semantic panel overlaps canvas on mobile: ${JSON.stringify({ panel: state.panelRect, canvas: state.canvasRect })}`);
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
