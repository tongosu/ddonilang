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
const OK = "seamgrim_model_comparison_teaching: ok";
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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-model-comparison-"));
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
    await page.waitForSelector("[data-curriculum-model-comparison]", { state: "visible", timeout: 20_000 });
    await page.waitForFunction((id) => {
      const model = window.__SEAMGRIM_CURRICULUM_MODEL_COMPARISON__ ?? null;
      return model?.lesson_id === id && model?.schema === "seamgrim.curriculum_model_comparison.v0";
    }, lessonId, { timeout: 20_000 });
    const state = await page.evaluate(async () => {
      const root = document.querySelector("#screen-run");
      const model = window.__SEAMGRIM_CURRICULUM_MODEL_COMPARISON__ ?? null;
      const snapshot = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "model comparison smoke" });
      return {
        panelText: String(document.querySelector("[data-curriculum-model-comparison]")?.textContent ?? ""),
        model,
        dataset: {
          schema: root?.dataset?.curriculumModelComparisonSchema ?? "",
          pair: root?.dataset?.curriculumDeclarativeTeachingPair ?? "",
          runtime: root?.dataset?.curriculumDeclarativeRuntimeLanded ?? "",
        },
        snapshotModel: snapshot.project_snapshot.curriculum_model_comparison ?? null,
      };
    });
    assert(state.panelText.includes("모델 비교"), `${lessonId}: panel text`);
    assert(state.model?.explicit_model?.available === true, `${lessonId}: explicit model`);
    assert(state.model?.explicit_model?.runtime_execution === "landed", `${lessonId}: explicit runtime`);
    assert(state.model?.boundaries?.declarative_runtime_completion_claim === false, `${lessonId}: declarative overclaim`);
    assert(state.model?.boundaries?.state_hash_participation === false, `${lessonId}: state hash boundary`);
    assert(state.dataset.schema === "seamgrim.curriculum_model_comparison.v0", `${lessonId}: dataset schema`);
    assert(state.dataset.runtime === "0", `${lessonId}: declarative runtime must stay not landed`);
    if (lessonId === "rep_phys_projectile_xy_v1") {
      assert(state.dataset.pair === "1", "projectile should expose existing teaching pair");
      assert(state.model.declarative_model.available === true, "projectile declarative pair available");
      assert(state.model.declarative_model.teaching_snippet.join("\n").includes("위치_x' =:= 속도_x"), "projectile declarative snippet");
    } else {
      assert(state.dataset.pair === "0", `${lessonId}: non-projectile declarative pair should be candidate`);
      assert(state.model.declarative_model.available === false, `${lessonId}: non-projectile declarative availability`);
    }
    assert(state.snapshotModel?.schema === "seamgrim.curriculum_model_comparison.v0", `${lessonId}: snapshot model`);
    if (failures.length) throw new Error(failures.join("\n"));
    return {
      lesson_id: lessonId,
      declarative_pair: state.dataset.pair === "1",
      snapshot_model: true,
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
      schema: "seamgrim.model_comparison_teaching.runner_result.v1",
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
