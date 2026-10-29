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
const OK = "seamgrim_semantic_scenes_five_lessons: ok";
const LESSONS = [
  "rep_math_function_line_v1",
  "rep_phys_pendulum_wave1_v1",
  "rep_phys_projectile_xy_v1",
  "rep_phys_damped_oscillation_v1",
  "rep_econ_supply_demand_tax_v1",
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function freePort() {
  return new Promise((resolve, reject) => {
    import("node:net").then(({ default: net }) => {
      const server = net.createServer();
      server.once("error", reject);
      server.listen(0, "127.0.0.1", () => {
        const address = server.address();
        const port = address && typeof address !== "string" ? address.port : 0;
        server.close(() => resolve(port));
      });
    }).catch(reject);
  });
}

async function waitForHealth(baseUrl, child) {
  for (let i = 0; i < 100; i += 1) {
    if (child.exitCode !== null) throw new Error(`server exited: ${child.exitCode}`);
    try {
      const response = await fetch(`${baseUrl}/api/v1/projects/health`);
      if (response.ok) return;
    } catch (_) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error("server health timeout");
}

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-semantic-scenes-"));
  const port = await freePort();
  const env = { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir };
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", () => {});
  child.stderr.on("data", () => {});
  const baseUrl = `http://127.0.0.1:${port}`;
  await waitForHealth(baseUrl, child);
  return { child, baseUrl, dataDir };
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

async function verifyLesson(browser, baseUrl, lessonId) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await context.newPage();
  try {
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&lesson=${lessonId}`, {
      waitUntil: "domcontentloaded",
    });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
    await page.waitForSelector("[data-curriculum-semantic-scene]", { state: "visible", timeout: 20_000 });
    await page.waitForFunction((id) => {
      const scene = window.__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__ ?? null;
      return scene?.lesson_id === id
        && scene?.schema === "seamgrim.curriculum_semantic_scene.v0"
        && scene?.runtime_binding?.complete === true;
    }, lessonId, { timeout: 20_000 });
    const state = await page.evaluate(async () => {
      const root = document.querySelector("#screen-run");
      const scene = window.__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__ ?? null;
      const snapshot = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "semantic scene smoke" });
      return {
        text: String(document.querySelector("[data-curriculum-semantic-scene]")?.textContent ?? ""),
        scene,
        dataset: {
          schema: root?.dataset?.curriculumSemanticSceneSchema ?? "",
          count: Number(root?.dataset?.curriculumSemanticSceneAnchorCount ?? 0),
        },
        snapshotScene: snapshot.project_snapshot.curriculum_semantic_scene ?? null,
        targetMatches: Array.isArray(scene?.anchors)
          ? scene.anchors.map((anchor) => ({
              anchor_id: anchor.anchor_id,
              matched: Boolean(anchor.target_selector && document.querySelector(anchor.target_selector)),
            }))
          : [],
        primaryViewMatched: Boolean(
          scene?.primary_view_binding?.target_selector
          && document.querySelector(scene.primary_view_binding.target_selector),
        ),
      };
    });
    assert(state.text.includes("의미 장면"), `${lessonId}: semantic scene text`);
    assert(state.scene?.schema === "seamgrim.curriculum_semantic_scene.v0", `${lessonId}: scene schema`);
    assert(state.dataset.schema === "seamgrim.curriculum_semantic_scene.v0", `${lessonId}: dataset schema`);
    assert(state.dataset.count >= 2, `${lessonId}: anchor count ${state.dataset.count}`);
    assert(Array.isArray(state.scene.anchors) && state.scene.anchors.length === state.dataset.count, `${lessonId}: anchors`);
    assert(state.scene.anchors.every((anchor) => anchor.state_hash_participation === false), `${lessonId}: anchor state hash boundary`);
    assert(state.scene.anchors.every((anchor) => anchor.binding?.resolved === true), `${lessonId}: unresolved runtime anchor`);
    assert(state.targetMatches.every((row) => row.matched), `${lessonId}: anchor target missing ${JSON.stringify(state.targetMatches)}`);
    assert(state.primaryViewMatched, `${lessonId}: primary view target missing`);
    assert(state.scene.runtime_binding?.complete === true, `${lessonId}: runtime binding incomplete`);
    assert(state.scene.runtime_binding?.observation_source === "seamgrim.observation_payload.v1", `${lessonId}: observation source`);
    assert(state.scene.boundaries?.world_state_storage === false, `${lessonId}: world state storage boundary`);
    assert(state.scene.boundaries?.semantic_runtime_completion_claim === false, `${lessonId}: semantic runtime overclaim`);
    assert(state.scene.anchors.every((anchor) => String(anchor.a11y_label ?? "").trim()), `${lessonId}: a11y labels`);
    assert(state.snapshotScene?.schema === "seamgrim.curriculum_semantic_scene.v0", `${lessonId}: snapshot scene`);
    return {
      lesson_id: lessonId,
      anchor_count: state.scene.anchors.length,
      resolved_anchor_count: state.scene.runtime_binding.resolved_count,
      runtime_bound: true,
      snapshot_scene: true,
    };
  } finally {
    await context.close();
  }
}

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  try {
    const lessons = [];
    for (const lessonId of LESSONS) {
      lessons.push(await verifyLesson(browser, baseUrl, lessonId));
    }
    console.log(JSON.stringify({
      schema: "seamgrim.semantic_scenes_five_lessons.runner_result.v1",
      status: "ok",
      lessons,
      data_dir: dataDir,
    }));
    console.log(OK);
  } finally {
    await browser.close();
    await stopServer(child);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
