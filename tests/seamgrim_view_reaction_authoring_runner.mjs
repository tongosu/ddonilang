#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_view_reaction_authoring: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";

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
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|detjson|md)$/i.test(pathname)
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

async function waitVisible(page, selector) {
  await page.waitForFunction((sel) => {
    const node = document.querySelector(sel);
    return node && !String(node.className ?? "").split(/\s+/).includes("hidden");
  }, selector, { timeout: 15_000 });
}

async function readAuthoringState(page) {
  return page.evaluate(() => {
    const panel = document.querySelector("[data-view-reaction-authoring-panel]");
    let artifact = null;
    try {
      artifact = JSON.parse(document.querySelector("[data-view-author-artifact]")?.value ?? "null");
    } catch (_) {
      artifact = null;
    }
    return {
      panel: { ...(panel?.dataset ?? {}) },
      artifact: window.__SEAMGRIM_VIEW_REACTION_AUTHORING__ ?? artifact,
      cameraOptions: Array.from(document.querySelectorAll("[data-view-author-camera-policy] option")).map((node) => node.value),
      conditionOptions: Array.from(document.querySelectorAll("[data-view-author-condition-kind] option")).map((node) => node.value),
      actionOptions: Array.from(document.querySelectorAll("[data-view-author-action-kind] option")).map((node) => node.value),
      measurementInputs: Array.from(document.querySelectorAll("[data-view-author-measure]")).map((node) => ({
        id: node.dataset.viewAuthorMeasure,
        checked: node.checked,
      })),
      diagnosticsText: document.querySelector("[data-view-author-diagnostics]")?.textContent?.trim() ?? "",
    };
  });
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 760 }, locale: "ko-KR" });
    installFailureCapture(page, failures);
    await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
    await waitVisible(page, "#screen-browse");
    await page.locator('#screen-browse .main-shell-tab[data-main-tab-target="studio"][data-shell-space="create"]').click();
    await waitVisible(page, "#screen-editor");
    await page.locator("#btn-editor-lesson-plan").click();
    await page.waitForSelector("[data-view-reaction-authoring-panel]", { state: "visible", timeout: 15_000 });

    const initial = await readAuthoringState(page);
    assert(initial.cameraOptions.join(",") === "fixed,fit,follow", `camera policy options mismatch: ${initial.cameraOptions}`);
    assert(initial.conditionOptions.join(",") === "zero_crossing,sign_change,extremum,tolerance_entry", `condition options mismatch: ${initial.conditionOptions}`);
    assert(initial.actionOptions.join(",") === "highlight,visibility,camera_focus", `action options mismatch: ${initial.actionOptions}`);
    assert(initial.measurementInputs.map((row) => row.id).join(",") === "angle,length,vector,axes,trail", "measurement inputs mismatch");
    assert(initial.artifact?.schema === "seamgrim.view_reaction_authoring.v1", "artifact schema missing");
    assert(initial.artifact?.implicit_lww_allowed === false, "implicit LWW must be disabled");
    assert(initial.artifact?.state_hash_participation === false, "authoring artifact must stay outside state_hash");
    assert(initial.artifact?.live_online_reaction_claim === false, "live online claim must stay false");
    assert(initial.artifact?.sealed_trace_replay_scope === true, "sealed trace replay scope missing");

    await page.selectOption("[data-view-author-camera-policy]", "follow");
    await page.locator('[data-view-author-measure="axes"]').uncheck();
    await page.locator('[data-view-author-measure="trail"]').uncheck();
    await page.selectOption("[data-view-author-anchor]", "obs.bob");
    await page.selectOption("[data-view-author-condition-kind]", "sign_change");
    await page.selectOption("[data-view-author-action-kind]", "camera_focus");
    await page.fill("[data-view-author-priority]", "7");
    await page.locator("[data-view-author-add-rule]").click();
    await page.waitForFunction(() => {
      const panel = document.querySelector("[data-view-reaction-authoring-panel]");
      return panel?.dataset?.viewAuthoringCameraPolicy === "follow"
        && panel?.dataset?.viewAuthoringMeasurements === "angle,length,vector"
        && panel?.dataset?.viewAuthoringRuleCount === "1";
    }, null, { timeout: 10_000 });
    const authored = await readAuthoringState(page);
    assert(authored.artifact.camera.selected_policy === "follow", "camera follow selection not persisted");
    assert(authored.artifact.measurements.selected_components.join(",") === "angle,length,vector", "measurement selection not persisted");
    assert(authored.artifact.reaction_rules[0].condition_kind === "sign_change", "condition kind not persisted");
    assert(authored.artifact.reaction_rules[0].action_kind === "camera_focus", "action kind not persisted");
    assert(authored.artifact.reaction_rules[0].priority === 7, "explicit priority not persisted");
    assert(String(authored.artifact.artifact_hash ?? "").startsWith("candidate:fnv1a32:view_reaction_authoring:"), "artifact hash missing");
    assert(authored.panel.viewAuthoringReady === "1", "valid authoring state should be ready");

    await page.locator("[data-view-author-conflict-demo]").click();
    await page.waitForFunction(() => {
      const panel = document.querySelector("[data-view-reaction-authoring-panel]");
      return String(panel?.dataset?.viewAuthoringDiagnostics ?? "").includes("E_VIEW_AUTHOR_REACTION_CONFLICT");
    }, null, { timeout: 10_000 });
    const conflict = await readAuthoringState(page);
    assert(conflict.panel.viewAuthoringReady === "0", "conflict state must not be ready");
    assert(conflict.artifact.diagnostics.some((diag) => diag.code === "E_VIEW_AUTHOR_REACTION_CONFLICT"), "conflict diagnostic missing");
    assert(conflict.diagnosticsText.includes("E_VIEW_AUTHOR_REACTION_CONFLICT"), "conflict text missing");
    assert(conflict.artifact.implicit_lww_allowed === false, "conflict path must not enable LWW");

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
