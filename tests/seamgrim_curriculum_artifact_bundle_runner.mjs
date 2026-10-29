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
const OK = "seamgrim_curriculum_artifact_bundle: ok";

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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-artifact-bundle-"));
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
  const failures = [];
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("console", (msg) => {
    const text = String(msg.text() ?? "");
    if (msg.type() === "error" && !text.includes("Failed to load resource")) failures.push(`console error: ${text}`);
  });
  try {
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&lesson=${lessonId}`, {
      waitUntil: "domcontentloaded",
    });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
    await page.waitForSelector("[data-curriculum-artifact-bundle]", { state: "visible", timeout: 20_000 });
    await page.fill("textarea[data-curriculum-inquiry-prediction]", `${lessonId} 예측`);
    await page.waitForFunction((id) => {
      const bundle = window.__SEAMGRIM_CURRICULUM_ARTIFACT_BUNDLE__ ?? null;
      const status = bundle?.status ?? {};
      return bundle?.lesson_id === id && status.experiment_ready === true;
    }, lessonId, { timeout: 45_000 });
    await page.fill("textarea[data-curriculum-inquiry-explanation]", `${lessonId} 설명`);
    await page.waitForFunction((id) => {
      const root = document.querySelector("#screen-run");
      const bundle = window.__SEAMGRIM_CURRICULUM_ARTIFACT_BUNDLE__ ?? null;
      const status = bundle?.status ?? {};
      return root?.dataset?.curriculumArtifactBundle === "1"
        && bundle?.lesson_id === id
        && status.prediction_ready === true
        && status.experiment_ready === true
        && status.explanation_ready === true;
    }, lessonId, { timeout: 20_000 });
    const state = await page.evaluate(async () => {
      const bundle = window.__SEAMGRIM_CURRICULUM_ARTIFACT_BUNDLE__ ?? null;
      const root = document.querySelector("#screen-run");
      const snapshot = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "artifact bundle smoke" });
      return {
        bundle,
        dataset: {
          schema: root?.dataset?.curriculumArtifactBundleSchema ?? "",
          prediction: root?.dataset?.curriculumArtifactPredictionReady ?? "",
          experiment: root?.dataset?.curriculumArtifactExperimentReady ?? "",
          explanation: root?.dataset?.curriculumArtifactExplanationReady ?? "",
        },
        snapshotBundle: snapshot.project_snapshot.curriculum_artifact_bundle ?? null,
      };
    });
    assert(state.bundle?.schema === "seamgrim.curriculum_artifact_bundle.v0", `${lessonId}: bundle schema`);
    assert(state.dataset.schema === "seamgrim.curriculum_artifact_bundle.v0", `${lessonId}: dataset schema`);
    assert(`${state.dataset.prediction}${state.dataset.experiment}${state.dataset.explanation}` === "111", `${lessonId}: dataset ready flags`);
    const kinds = (state.bundle.artifacts ?? []).map((row) => row.artifact_kind).join(",");
    assert(kinds === "prediction,experiment,explanation", `${lessonId}: artifact kind order ${kinds}`);
    for (const artifact of state.bundle.artifacts ?? []) {
      assert(artifact.lesson_id === lessonId, `${lessonId}: artifact lesson mismatch`);
      assert(artifact.boundaries?.state_hash_participation === false, `${lessonId}: artifact state hash boundary`);
    }
    const experiment = state.bundle.artifacts.find((row) => row.artifact_kind === "experiment");
    assert(String(experiment?.runtime_hash ?? "").includes("blake3:") || String(experiment?.runtime_hash ?? "").startsWith("server-fallback"), `${lessonId}: experiment hash`);
    assert(state.snapshotBundle?.schema === "seamgrim.curriculum_artifact_bundle.v0", `${lessonId}: snapshot bundle`);
    if (failures.length) throw new Error(failures.join("\n"));
    return {
      lesson_id: lessonId,
      artifact_kinds: kinds,
      experiment_hash: experiment.runtime_hash,
      snapshot_bundle: true,
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
      schema: "seamgrim.curriculum_artifact_bundle.runner_result.v1",
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
