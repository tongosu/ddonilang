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
const OK = "seamgrim_curriculum_workflow_three_worlds: ok";

const WORLDS = [
  {
    lessonId: "rep_math_function_line_v1",
    worldId: "rep_math_function_line_v1",
    primaryView: "graph",
    prediction: "기울기를 키우면 같은 x에서 y가 커질 것이다.",
    explanation: "실행 뒤 그래프 기울기가 커지고 y값 증가가 빨라진 것을 확인했다.",
  },
  {
    lessonId: "rep_phys_pendulum_wave1_v1",
    worldId: "rep_phys_pendulum_wave1_v1",
    primaryView: "space2d",
    prediction: "줄 길이를 늘리면 왕복이 느려질 것이다.",
    explanation: "추 위치와 각도 그래프를 같은 마디에서 보니 긴 줄일수록 변화가 완만했다.",
  },
  {
    lessonId: "rep_econ_supply_demand_tax_v1",
    worldId: "rep_econ_supply_demand_tax_v1",
    primaryView: "graph",
    prediction: "세금을 올리면 거래량은 줄고 가격 조정은 달라질 것이다.",
    explanation: "세금 조절 뒤 가격 그래프와 거래량 출력이 함께 바뀌었다.",
  },
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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-curriculum-workflow-"));
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

async function waitForRunHash(page, label) {
  await page.waitForFunction(() => {
    const title = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    return title.includes("blake3:") || title.startsWith("server-fallback");
  }, null, { timeout: 35_000 });
  const hash = await page.locator("#run-mirror-hash").getAttribute("title");
  assert(hash && (hash.includes("blake3:") || hash.startsWith("server-fallback")), `${label}: runtime hash missing`);
  const blake = String(hash).match(/blake3:[0-9a-f]+/i)?.[0] ?? "";
  return blake || hash;
}

async function runCurrentWorldExplicitly(page, label) {
  await page.waitForFunction(() => {
    const run = document.querySelector("#btn-run");
    return run && !run.disabled;
  }, null, { timeout: 40_000 });
  await page.click("#btn-run");
  return waitForRunHash(page, label);
}

async function waitForCurriculumArtifactHash(page, world) {
  await page.waitForFunction((lessonId) => {
    const artifact = window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null;
    const hash = String(artifact?.run_evidence?.runtime_hash ?? "");
    return artifact?.lesson_id === lessonId && (hash.includes("blake3:") || hash.startsWith("server-fallback"));
  }, world.lessonId, { timeout: 40_000 });
  const artifact = await page.evaluate(() => window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null);
  const hash = String(artifact?.run_evidence?.runtime_hash ?? "");
  assert(hash, `${world.lessonId}: curriculum artifact runtime hash missing`);
  return hash;
}

async function openWorld(page, baseUrl, world) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&lesson=${world.lessonId}`, {
    waitUntil: "domcontentloaded",
  });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction((lessonId) => {
    const root = document.querySelector("#screen-run");
    return root?.dataset?.curriculumInquiryRuntime === "1"
      && root?.dataset?.curriculumInquiryLessonId === lessonId;
  }, world.lessonId, { timeout: 20_000 });
}

async function verifyWorldWorkflow(browser, baseUrl, world) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await context.newPage();
  const failures = [];
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("console", (msg) => {
    const text = String(msg.text() ?? "");
    if (msg.type() === "error" && !text.includes("Failed to load resource")) failures.push(`console error: ${text}`);
  });
  try {
    await openWorld(page, baseUrl, world);
    await page.fill("textarea[data-curriculum-inquiry-prediction]", world.prediction);
    await waitForCurriculumArtifactHash(page, world);
    await page.fill("textarea[data-curriculum-inquiry-explanation]", world.explanation);
    await page.waitForFunction((lessonId) => {
      const artifact = window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null;
      const hash = String(artifact?.run_evidence?.runtime_hash ?? "");
      return artifact?.lesson_id === lessonId && (hash.includes("blake3:") || hash.startsWith("server-fallback"));
    }, world.lessonId, { timeout: 20_000 });
    const artifactBeforeSave = await page.evaluate(() => window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null);
    const savedRuntimeHash = String(artifactBeforeSave?.run_evidence?.runtime_hash ?? "");
    assert(artifactBeforeSave?.schema === "seamgrim.curriculum_inquiry_artifact.v0", `${world.lessonId}: inquiry schema missing`);
    assert(artifactBeforeSave.lesson_id === world.lessonId, `${world.lessonId}: artifact lesson id mismatch`);
    assert(artifactBeforeSave.world_id === world.worldId, `${world.lessonId}: artifact world id mismatch`);
    assert(artifactBeforeSave.primary_view === world.primaryView, `${world.lessonId}: primary view mismatch`);
    assert(artifactBeforeSave.response.prediction_text === world.prediction, `${world.lessonId}: prediction not captured`);
    assert(artifactBeforeSave.response.explanation_text === world.explanation, `${world.lessonId}: explanation not captured`);
    assert(savedRuntimeHash.includes("blake3:") || savedRuntimeHash.startsWith("server-fallback"), `${world.lessonId}: runtime hash not captured`);
    assert(artifactBeforeSave.boundaries.state_hash_participation === false, `${world.lessonId}: inquiry leaked into state hash`);

    await page.locator("#screen-run [data-shell-space='create']").click();
    await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
    await page.fill("#project-name-input", `Wave3A ${world.lessonId}`);
    const snapshotBeforeSave = await page.evaluate(async () => {
      const artifact = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: document.querySelector("#project-name-input")?.value ?? "" });
      return artifact.project_snapshot.curriculum_inquiry_artifact ?? null;
    });
    assert(snapshotBeforeSave?.response?.prediction_text === world.prediction, `${world.lessonId}: snapshot prediction missing before save`);
    assert(snapshotBeforeSave?.response?.explanation_text === world.explanation, `${world.lessonId}: snapshot explanation missing before save`);

    await page.click("#btn-project-save");
    await page.waitForFunction(() => {
      const status = document.querySelector("#project-save-status");
      const link = document.querySelector("#project-edit-link");
      return status?.dataset?.state === "saved" && String(link?.href ?? "").includes("sg_cap=");
    }, null, { timeout: 30_000 });
    const editLink = await page.locator("#project-edit-link").getAttribute("href");
    assert(editLink && editLink.includes("#sg_project="), `${world.lessonId}: edit link missing`);

    const reopenedContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const reopened = await reopenedContext.newPage();
    try {
      await reopened.goto(editLink, { waitUntil: "domcontentloaded" });
      await reopened.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
      await reopened.locator("#screen-editor [data-shell-space='lab']").click();
      await reopened.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
      await reopened.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { state: "visible", timeout: 20_000 });
      const reopenedState = await reopened.evaluate(async () => {
        const productArtifact = window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null;
        const snapshotArtifact = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: document.querySelector("#project-name-input")?.value ?? "" });
        return {
          predictionValue: String(document.querySelector("textarea[data-curriculum-inquiry-prediction]")?.value ?? ""),
          explanationValue: String(document.querySelector("textarea[data-curriculum-inquiry-explanation]")?.value ?? ""),
          productArtifact,
          snapshotArtifact: snapshotArtifact.project_snapshot.curriculum_inquiry_artifact ?? null,
          roleToggleText: ["학생", "교사"].filter((text) => document.body.innerText.includes(`${text}로 전환`)),
        };
      });
      assert(reopenedState.predictionValue === world.prediction, `${world.lessonId}: prediction not restored`);
      assert(reopenedState.explanationValue === world.explanation, `${world.lessonId}: explanation not restored`);
      assert(reopenedState.productArtifact?.lesson_id === world.lessonId, `${world.lessonId}: reopened product artifact lesson mismatch`);
      const reopenedHash = String(reopenedState.snapshotArtifact?.run_evidence?.runtime_hash ?? "");
      assert(
        reopenedHash.includes("blake3:") || reopenedHash.startsWith("server-fallback"),
        `${world.lessonId}: reopened snapshot runtime hash missing`,
      );
      assert(reopenedState.roleToggleText.length === 0, `${world.lessonId}: runtime role toggle leaked`);
    } finally {
      await reopenedContext.close();
    }
    if (failures.length) throw new Error(failures.join("\n"));
    return {
      lesson_id: world.lessonId,
      runtime_hash: savedRuntimeHash,
      prediction_saved: true,
      explanation_saved: true,
      save_reopen_roundtrip: true,
    };
  } finally {
    await context.close();
  }
}

async function main() {
  const { child, baseUrl, dataDir } = await startServer();
  const browser = await chromium.launch({ headless: true });
  try {
    const results = [];
    for (const world of WORLDS) {
      results.push(await verifyWorldWorkflow(browser, baseUrl, world));
    }
    console.log(JSON.stringify({
      schema: "seamgrim.curriculum_workflow_three_worlds.runner_result.v1",
      status: "ok",
      worlds: results,
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
