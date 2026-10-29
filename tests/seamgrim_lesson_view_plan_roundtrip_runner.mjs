#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_lesson_view_plan_roundtrip: ok";
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

async function dispatchChange(page, selector, value) {
  await page.locator(selector).fill(value);
  await page.locator(selector).dispatchEvent("change");
}

async function readState(page) {
  return page.evaluate(async () => {
    const code = document.querySelector("[data-lesson-plan-code]")?.value ?? "";
    const { parseLessonPlanToml, serializeLessonPlanToml, roundtripLessonPlanToml } = await import("./lesson_plan_v0.js");
    const parsed = parseLessonPlanToml(code);
    const canonical = serializeLessonPlanToml(parsed);
    const roundtrip = roundtripLessonPlanToml(code);
    return {
      code,
      canonical,
      parsed,
      roundtrip,
      artifact: window.__SEAMGRIM_VIEW_REACTION_AUTHORING__ ?? null,
      fields: {
        scene_ref: document.querySelector("[data-lesson-plan-field='scene_ref']")?.value ?? "",
        primary_view: document.querySelector("[data-lesson-plan-field='primary_view']")?.value ?? "",
        camera_policy: document.querySelector("[data-lesson-plan-field='camera_policy']")?.value ?? "",
        measurement_components: document.querySelector("[data-lesson-plan-field='measurement_components']")?.value ?? "",
        reaction_rule_refs: document.querySelector("[data-lesson-plan-field='reaction_rule_refs']")?.value ?? "",
        view_authoring_hash: document.querySelector("[data-lesson-plan-field='view_authoring_hash']")?.value ?? "",
      },
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

    await dispatchChange(page, "[data-testid='lesson-plan-scene-ref']", "scene.pendulum.main");
    await dispatchChange(page, "[data-testid='lesson-plan-primary-view']", "space2d");
    await page.selectOption("[data-view-author-camera-policy]", "follow");
    await page.locator('[data-view-author-measure="axes"]').uncheck();
    await page.locator('[data-view-author-measure="trail"]').uncheck();
    await page.selectOption("[data-view-author-condition-kind]", "extremum");
    await page.selectOption("[data-view-author-action-kind]", "camera_focus");
    await page.fill("[data-view-author-priority]", "12");
    await page.locator("[data-view-author-add-rule]").click();
    await page.waitForFunction(() => {
      const code = document.querySelector("[data-lesson-plan-code]")?.value ?? "";
      return code.includes('scene_ref = "scene.pendulum.main"')
        && code.includes('primary_view = "space2d"')
        && code.includes('camera_policy = "follow"')
        && code.includes('measurement_components = ["angle", "length", "vector"]')
        && code.includes('reaction_rule_refs = ["author.rule.1"]')
        && code.includes("candidate:fnv1a32:view_reaction_authoring:");
    }, null, { timeout: 10_000 });

    const authored = await readState(page);
    const activity = authored.parsed.activities[0];
    assert(authored.roundtrip.byte_stable === true, "lesson plan TOML must be byte-stable");
    assert(authored.roundtrip.semantic_equal === true, "lesson plan TOML must be semantically stable");
    assert(activity.scene_ref === "scene.pendulum.main", "scene_ref not preserved");
    assert(activity.primary_view === "space2d", "primary_view not preserved");
    assert(activity.camera_policy === "follow", "camera policy not linked from authoring artifact");
    assert(activity.measurement_components.join(",") === "angle,length,vector", "measurement components not preserved");
    assert(activity.reaction_rule_refs.join(",") === "author.rule.1", "reaction rule refs not preserved");
    assert(activity.view_plan_ref === "view_plan.v1", "view plan ref missing");
    assert(activity.view_authoring_hash === authored.artifact.artifact_hash, "view authoring hash not linked");

    const canonicalBefore = authored.canonical;
    await page.locator("[data-lesson-plan-action='apply-code']").click();
    await page.waitForFunction((expected) => {
      const code = document.querySelector("[data-lesson-plan-code]")?.value ?? "";
      return code === expected;
    }, canonicalBefore, { timeout: 10_000 });
    const afterApply = await readState(page);
    assert(afterApply.canonical === canonicalBefore, "apply-code changed canonical TOML");
    assert(afterApply.fields.scene_ref === "scene.pendulum.main", "structure scene field did not survive code apply");
    assert(afterApply.fields.primary_view === "space2d", "structure primary view did not survive code apply");
    assert(afterApply.fields.camera_policy === "follow", "structure camera policy did not survive code apply");
    assert(afterApply.fields.measurement_components === "angle, length, vector", "structure measurement field did not survive code apply");
    assert(afterApply.fields.reaction_rule_refs === "author.rule.1", "structure reaction refs did not survive code apply");
    assert(afterApply.fields.view_authoring_hash === authored.artifact.artifact_hash, "structure artifact hash did not survive code apply");

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
