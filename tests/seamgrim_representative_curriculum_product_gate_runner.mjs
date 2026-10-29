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
const OK = "seamgrim_representative_curriculum_product_gate: ok";
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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-curriculum-gate-"));
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
    await page.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { state: "visible", timeout: 20_000 });
    await page.waitForSelector("[data-curriculum-model-comparison]", { state: "visible", timeout: 20_000 });
    await page.waitForSelector("[data-curriculum-semantic-scene]", { state: "visible", timeout: 20_000 });
    await page.fill("textarea[data-curriculum-inquiry-prediction]", `${lessonId} 대표 예측`);
    await page.waitForFunction((id) => {
      const root = document.querySelector("#screen-run");
      const inquiry = window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null;
      const bundle = window.__SEAMGRIM_CURRICULUM_ARTIFACT_BUNDLE__ ?? null;
      const model = window.__SEAMGRIM_CURRICULUM_MODEL_COMPARISON__ ?? null;
      const scene = window.__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__ ?? null;
      return root?.dataset?.curriculumInquiryLessonId === id
        && inquiry?.lesson_id === id
        && bundle?.lesson_id === id
        && model?.lesson_id === id
        && scene?.lesson_id === id
        && bundle?.status?.experiment_ready === true;
    }, lessonId, { timeout: 45_000 });
    await page.fill("textarea[data-curriculum-inquiry-explanation]", `${lessonId} 대표 설명`);
    await page.waitForFunction(() => {
      const root = document.querySelector("#screen-run");
      return root?.dataset?.curriculumInquiryPrediction === "1"
        && root?.dataset?.curriculumInquiryExplanation === "1"
        && root?.dataset?.curriculumArtifactPredictionReady === "1"
        && root?.dataset?.curriculumArtifactExperimentReady === "1"
        && root?.dataset?.curriculumArtifactExplanationReady === "1";
    }, null, { timeout: 20_000 });

    const state = await page.evaluate(async () => {
      const root = document.querySelector("#screen-run");
      const inquiry = window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null;
      const bundle = window.__SEAMGRIM_CURRICULUM_ARTIFACT_BUNDLE__ ?? null;
      const model = window.__SEAMGRIM_CURRICULUM_MODEL_COMPARISON__ ?? null;
      const scene = window.__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__ ?? null;
      const snapshot = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: "representative curriculum gate" });
      return {
        dataset: {
          inquiry: root?.dataset?.curriculumInquiryRuntime ?? "",
          inquirySchema: root?.dataset?.curriculumInquirySchema ?? "",
          bundle: root?.dataset?.curriculumArtifactBundle ?? "",
          bundleSchema: root?.dataset?.curriculumArtifactBundleSchema ?? "",
          model: root?.dataset?.curriculumModelComparison ?? "",
          modelSchema: root?.dataset?.curriculumModelComparisonSchema ?? "",
          modelPair: root?.dataset?.curriculumDeclarativeTeachingPair ?? "",
          modelRuntime: root?.dataset?.curriculumDeclarativeRuntimeLanded ?? "",
          scene: root?.dataset?.curriculumSemanticScene ?? "",
          sceneSchema: root?.dataset?.curriculumSemanticSceneSchema ?? "",
          sceneAnchorCount: Number(root?.dataset?.curriculumSemanticSceneAnchorCount ?? 0),
        },
        inquiry,
        bundle,
        model,
        scene,
        snapshot: snapshot.project_snapshot,
        roleToggleText: ["학생으로 전환", "교사로 전환"].filter((text) => document.body.innerText.includes(text)),
      };
    });

    assert(state.dataset.inquiry === "1", `${lessonId}: inquiry dataset`);
    assert(state.dataset.inquirySchema === "seamgrim.curriculum_inquiry_artifact.v0", `${lessonId}: inquiry schema`);
    assert(state.dataset.bundle === "1", `${lessonId}: bundle dataset`);
    assert(state.dataset.bundleSchema === "seamgrim.curriculum_artifact_bundle.v0", `${lessonId}: bundle schema`);
    assert(state.dataset.model === "1", `${lessonId}: model dataset`);
    assert(state.dataset.modelSchema === "seamgrim.curriculum_model_comparison.v0", `${lessonId}: model schema`);
    assert(state.dataset.scene === "1", `${lessonId}: scene dataset`);
    assert(state.dataset.sceneSchema === "seamgrim.curriculum_semantic_scene.v0", `${lessonId}: scene schema`);
    assert(state.dataset.sceneAnchorCount >= 2, `${lessonId}: semantic anchor count`);
    assert(state.inquiry?.boundaries?.state_hash_participation === false, `${lessonId}: inquiry state hash boundary`);
    assert(state.bundle?.artifacts?.length === 3, `${lessonId}: bundle artifacts`);
    assert(state.bundle.artifacts.every((row) => row.boundaries?.state_hash_participation === false), `${lessonId}: bundle state hash boundary`);
    assert(state.model?.boundaries?.declarative_runtime_completion_claim === false, `${lessonId}: model runtime overclaim`);
    assert(state.scene?.boundaries?.world_state_storage === false, `${lessonId}: scene world-state boundary`);
    assert(state.scene?.anchors?.every((anchor) => String(anchor.a11y_label ?? "").trim()), `${lessonId}: semantic a11y label`);
    assert(state.snapshot.curriculum_inquiry_artifact?.schema === "seamgrim.curriculum_inquiry_artifact.v0", `${lessonId}: snapshot inquiry`);
    assert(state.snapshot.curriculum_artifact_bundle?.schema === "seamgrim.curriculum_artifact_bundle.v0", `${lessonId}: snapshot bundle`);
    assert(state.snapshot.curriculum_model_comparison?.schema === "seamgrim.curriculum_model_comparison.v0", `${lessonId}: snapshot model`);
    assert(state.snapshot.curriculum_semantic_scene?.schema === "seamgrim.curriculum_semantic_scene.v0", `${lessonId}: snapshot scene`);
    assert(state.roleToggleText.length === 0, `${lessonId}: runtime role toggle leaked`);
    if (lessonId === "rep_phys_projectile_xy_v1") {
      assert(state.dataset.modelPair === "1", `${lessonId}: projectile model pair missing`);
    }
    assert(state.dataset.modelRuntime === "0", `${lessonId}: declarative runtime overclaim`);
    if (failures.length) throw new Error(failures.join("\n"));
    return {
      lesson_id: lessonId,
      schemas: [
        state.dataset.inquirySchema,
        state.dataset.bundleSchema,
        state.dataset.modelSchema,
        state.dataset.sceneSchema,
      ],
      semantic_anchor_count: state.dataset.sceneAnchorCount,
      snapshot_fields: [
        Boolean(state.snapshot.curriculum_inquiry_artifact),
        Boolean(state.snapshot.curriculum_artifact_bundle),
        Boolean(state.snapshot.curriculum_model_comparison),
        Boolean(state.snapshot.curriculum_semantic_scene),
      ],
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
      schema: "seamgrim.representative_curriculum_product_gate.runner_result.v1",
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
