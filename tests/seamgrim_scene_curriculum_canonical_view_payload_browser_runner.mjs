#!/usr/bin/env node

import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const SERVER = path.resolve(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");
const LESSON_ID = "rep_math_function_line_v1";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function freePort() {
  const { default: net } = await import("node:net");
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      server.close(() => resolve(address && typeof address !== "string" ? address.port : 0));
    });
  });
}

async function waitForHealth(baseUrl, child) {
  for (let index = 0; index < 120; index += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/v1/projects/health`);
      if (response.ok) return;
    } catch (_) {
      // Retry the bounded local-only health check.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("server health timeout");
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-t5g-browser-"));
const authoritativeFixture = JSON.parse(await fs.readFile(
  path.join(ROOT, "pack/seamgrim_scene_curriculum_canonical_view_payload_v1/fixtures/authoritative_state.detjson"),
  "utf8",
));
const port = await freePort();
const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
  cwd: ROOT,
  env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
  stdio: ["ignore", "pipe", "pipe"],
});
child.stdout.on("data", () => {});
child.stderr.on("data", () => {});
const baseUrl = `http://127.0.0.1:${port}`;
await waitForHealth(baseUrl, child);

const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&lesson=${LESSON_ID}`, {
    waitUntil: "domcontentloaded",
  });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.waitForSelector("[data-curriculum-semantic-scene]", { state: "visible", timeout: 20_000 });
  try {
    await page.waitForFunction((lessonId) => {
      const curriculum = window.__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__ ?? null;
      const scene = window.__SEAMGRIM_SCENE_SUMMARY__ ?? null;
      return curriculum?.lesson_id === lessonId
        && Boolean(curriculum?.canonical_view_payload?.status)
        && curriculum?.runtime_binding?.complete === true
        && Boolean(scene?.canonical_view_payload?.status);
    }, LESSON_ID, { timeout: 45_000 });
  } catch (error) {
    const diagnostic = await page.evaluate(() => ({
      curriculum: window.__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__ ?? null,
      scene: window.__SEAMGRIM_SCENE_SUMMARY__ ?? null,
      runCanonical: window.__SEAMGRIM_RUN_CANONICAL_VIEW_PAYLOADS__ ?? null,
    }));
    throw new Error(`T5G browser canonical readiness timeout: ${error.message}\n${JSON.stringify(diagnostic)}`);
  }
  const result = await page.evaluate(async (fixture) => {
    const curriculum = window.__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__;
    const scene = window.__SEAMGRIM_SCENE_SUMMARY__;
    const snapshot = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "T5G browser" });
    const payloadModule = await import("./workbench/semantic_view_payload.js");
    const stateModule = await import("./seamgrim_runtime_state.js");
    const sceneModule = await import("./scene_summary_contract.js");
    const curriculumModule = await import("./curriculum_semantic_scene.js");
    const curriculumBytes = payloadModule.stableSemanticJson(curriculum.canonical_view_payload);
    const sceneBytes = payloadModule.stableSemanticJson(scene.canonical_view_payload);
    const fixturePayloads = payloadModule.buildCanonicalViewPayloadSet(
      stateModule.extractCanonicalViewAuthorityInputsFromState(fixture),
    );
    const fixtureScene = sceneModule.buildSceneSummarySnapshot({
      lessonId: "browser-fixture",
      requiredViews: ["graph", "table", "text"],
      primaryView: "graph",
      canonicalViewPayloads: fixturePayloads,
      runtimeDerived: { canonicalViewPayloads: fixturePayloads },
    });
    const fixtureCurriculum = curriculumModule.buildCurriculumSemanticSceneV0({
      id: "browser-fixture",
      curriculumCard: {
        lesson_id: "browser-fixture",
        world_id: "world.browser-fixture",
        primary_view: "graph",
        allowed_controls: ["속도"],
        allowed_observations: ["위치"],
        display: { core_concepts: ["위치"] },
      },
    }, {
      controlValues: { "속도": 12 },
      observationRows: [{ id: "위치", value: 8 }],
      primaryViewMode: "graph",
      canonicalViewPayloads: fixturePayloads,
    });
    return {
      curriculum_status: curriculum.canonical_view_payload.status,
      curriculum_family: curriculum.canonical_view_payload.family,
      curriculum_bytes: curriculumBytes,
      curriculum_snapshot_bytes: payloadModule.stableSemanticJson(
        snapshot.project_snapshot.curriculum_semantic_scene.canonical_view_payload,
      ),
      curriculum_runtime_complete: curriculum.runtime_binding.complete,
      curriculum_canonical_binding_complete: curriculum.runtime_binding.canonical_binding_complete,
      scene_status: scene.canonical_view_payload.status,
      scene_family: scene.canonical_view_payload.family,
      scene_bytes: sceneBytes,
      scene_session_bytes: payloadModule.stableSemanticJson(
        JSON.parse(localStorage.getItem("seamgrim.scene_summary.v0")).scene.canonical_view_payload,
      ),
      curriculum_forbidden_tokens: ["target_selector", "a11y_label", "reading_order", "#run-main-graph-host"]
        .filter((token) => curriculumBytes.includes(token)),
      scene_forbidden_tokens: ["browser_only", "localStorage", "frame_sample", "bogae_scene"]
        .filter((token) => sceneBytes.includes(token)),
      fixture_scene_status: fixtureScene.canonical_view_payload.status,
      fixture_curriculum_status: fixtureCurriculum.canonical_view_payload.status,
      fixture_primary_parity: payloadModule.stableSemanticJson(
        fixtureScene.canonical_view_payload.content.scene_summary.primary_payload,
      ) === payloadModule.stableSemanticJson(
        fixtureCurriculum.canonical_view_payload.content.curriculum_semantic_scene.primary_view_payload,
      ),
      network_origin: location.origin,
    };
  }, authoritativeFixture);
  assert(result.curriculum_status === "no_view", "browser curriculum must preserve upstream canonical no_view");
  assert(result.curriculum_family === "curriculum-semantic-scene", "browser curriculum family drift");
  assert(result.curriculum_bytes === result.curriculum_snapshot_bytes, "browser curriculum snapshot parity drift");
  assert(result.curriculum_runtime_complete === true, "browser curriculum wrapper binding incomplete");
  assert(result.curriculum_canonical_binding_complete === false, "browser curriculum canonical absence masked");
  assert(result.scene_status === "no_view", "browser scene must preserve upstream canonical no_view");
  assert(result.scene_family === "scene-summary", "browser scene family drift");
  assert(result.scene_bytes === result.scene_session_bytes, "browser scene session parity drift");
  assert(result.curriculum_forbidden_tokens.length === 0, `browser curriculum presentation leak: ${result.curriculum_forbidden_tokens}`);
  assert(result.scene_forbidden_tokens.length === 0, `browser scene presentation leak: ${result.scene_forbidden_tokens}`);
  assert(result.fixture_scene_status === "available", "browser fixture scene canonical unavailable");
  assert(result.fixture_curriculum_status === "available", "browser fixture curriculum canonical unavailable");
  assert(result.fixture_primary_parity === true, "browser fixture primary canonical parity drift");
  console.log(JSON.stringify({ status: "PASS", lesson_id: LESSON_ID, ...result }));
} finally {
  await browser.close();
  await stopServer(child);
  const resolvedTempRoot = path.resolve(os.tmpdir());
  const resolvedDataDir = path.resolve(dataDir);
  assert(
    resolvedDataDir.startsWith(`${resolvedTempRoot}${path.sep}`)
      && path.basename(resolvedDataDir).startsWith("seamgrim-t5g-browser-"),
    `refusing to remove unexpected browser temp path: ${resolvedDataDir}`,
  );
  await fs.rm(resolvedDataDir, { recursive: true, force: true });
}
