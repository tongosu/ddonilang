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
const OK = "seamgrim_projectile_damping_promotion: ok";

const WORLDS = [
  {
    lessonId: "rep_phys_projectile_xy_v1",
    worldId: "rep_phys_projectile_xy_v1",
    primaryView: "space2d",
    requiredText: "포물선 운동",
    prediction: "중력을 키우면 최고점은 낮아지고 더 빨리 땅에 닿을 것이다.",
    explanation: "실행 결과 궤적과 속도 변화가 함께 바뀌었다.",
  },
  {
    lessonId: "rep_phys_damped_oscillation_v1",
    worldId: "rep_phys_damped_oscillation_v1",
    primaryView: "graph",
    requiredText: "감쇠 진동",
    prediction: "감쇠계수를 키우면 진폭이 더 빨리 줄어들 것이다.",
    explanation: "x(t) 그래프에서 진폭 감소가 빨라지는 흐름을 확인했다.",
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
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-projectile-damping-"));
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

async function waitForArtifact(page, world) {
  await page.waitForFunction((lessonId) => {
    const artifact = window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null;
    const hash = String(artifact?.run_evidence?.runtime_hash ?? "");
    return artifact?.lesson_id === lessonId && (hash.includes("blake3:") || hash.startsWith("server-fallback"));
  }, world.lessonId, { timeout: 45_000 });
  return page.evaluate(() => window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null);
}

async function verifyWorld(browser, baseUrl, world) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await context.newPage();
  const failures = [];
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("console", (msg) => {
    const text = String(msg.text() ?? "");
    if (msg.type() === "error" && !text.includes("Failed to load resource")) failures.push(`console error: ${text}`);
  });
  try {
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=studio&lesson=${world.lessonId}`, {
      waitUntil: "domcontentloaded",
    });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
    await page.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { state: "visible", timeout: 20_000 });
    await page.waitForFunction((lessonId) => {
      const root = document.querySelector("#screen-run");
      const source = String(document.querySelector("#run-ddn-preview")?.value ?? "");
      return root?.dataset?.curriculumInquiryLessonId === lessonId && source.length > 20;
    }, world.lessonId, { timeout: 20_000 });
    const routeAndSource = await page.evaluate(() => ({
      lesson: new URL(location.href).searchParams.get("lesson"),
      source: String(document.querySelector("#run-ddn-preview")?.value ?? ""),
    }));
    assert(routeAndSource.lesson === world.lessonId, `${world.lessonId}: route lesson mismatch`);
    assert(routeAndSource.source.includes(world.requiredText), `${world.lessonId}: source text missing lesson title`);

    await page.fill("textarea[data-curriculum-inquiry-prediction]", world.prediction);
    const artifactAfterRun = await waitForArtifact(page, world);
    if (world.lessonId === "rep_phys_damped_oscillation_v1") {
      await page.waitForFunction(() => {
        const scene = window.__SEAMGRIM_CURRICULUM_SEMANTIC_SCENE__ ?? null;
        const cursor = window.__SEAMGRIM_SEMANTIC_CURSOR__ ?? null;
        const ids = new Set(Array.isArray(cursor?.anchors) ? cursor.anchors.map((row) => row.anchor_id) : []);
        return scene?.runtime_binding?.complete === true
          && ids.has("obs.total_energy")
          && ids.has("obs.energy_loss");
      }, null, { timeout: 30_000 });
      const dampingEvidence = await page.evaluate(() => {
        const rows = JSON.parse(document.querySelector("#run-observe-output-body")?.dataset?.observationPayloadJson ?? "[]");
        const cursor = window.__SEAMGRIM_SEMANTIC_CURSOR__ ?? null;
        return {
          rows,
          cursor,
          semanticRuntime: document.querySelector("#screen-run")?.dataset?.semanticCursorRuntime ?? "",
        };
      });
      const observationIds = new Set(dampingEvidence.rows.map((row) => row.id));
      assert(observationIds.has("총에너지"), `${world.lessonId}: total energy observation`);
      assert(observationIds.has("에너지손실"), `${world.lessonId}: energy loss observation`);
      const totalEnergy = Number(dampingEvidence.rows.find((row) => row.id === "총에너지")?.value);
      const energyLoss = Number(dampingEvidence.rows.find((row) => row.id === "에너지손실")?.value);
      assert(Number.isFinite(totalEnergy) && totalEnergy > 0, `${world.lessonId}: total energy value`);
      assert(Number.isFinite(energyLoss) && energyLoss >= 0, `${world.lessonId}: energy loss value`);
      assert(dampingEvidence.semanticRuntime === "1", `${world.lessonId}: semantic cursor runtime`);
      assert(
        dampingEvidence.cursor?.anchors?.every((anchor) => anchor.source_available === true),
        `${world.lessonId}: semantic source provenance`,
      );
    }
    await page.fill("textarea[data-curriculum-inquiry-explanation]", world.explanation);
    const artifactBeforeSave = await page.evaluate(() => window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null);
    assert(artifactBeforeSave?.schema === "seamgrim.curriculum_inquiry_artifact.v0", `${world.lessonId}: inquiry schema`);
    assert(artifactBeforeSave.lesson_id === world.lessonId, `${world.lessonId}: inquiry lesson`);
    assert(artifactBeforeSave.world_id === world.worldId, `${world.lessonId}: inquiry world`);
    assert(artifactBeforeSave.primary_view === world.primaryView, `${world.lessonId}: primary view`);
    assert(artifactBeforeSave.response.prediction_text === world.prediction, `${world.lessonId}: prediction`);
    assert(artifactBeforeSave.response.explanation_text === world.explanation, `${world.lessonId}: explanation`);
    assert(
      String(artifactBeforeSave.run_evidence.runtime_hash ?? "").includes("blake3:")
        || String(artifactBeforeSave.run_evidence.runtime_hash ?? "").startsWith("server-fallback"),
      `${world.lessonId}: run evidence hash missing before save`,
    );
    assert(artifactBeforeSave.boundaries.state_hash_participation === false, `${world.lessonId}: state hash boundary`);

    await page.locator("#screen-run [data-shell-space='create']").click();
    await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
    await page.fill("#project-name-input", `Wave3A ${world.lessonId}`);
    await page.click("#btn-project-save");
    await page.waitForFunction(() => {
      const status = document.querySelector("#project-save-status");
      const link = document.querySelector("#project-edit-link");
      return status?.dataset?.state === "saved" && String(link?.href ?? "").includes("sg_cap=");
    }, null, { timeout: 30_000 });
    const editLink = await page.locator("#project-edit-link").getAttribute("href");
    assert(editLink && editLink.includes("#sg_project="), `${world.lessonId}: edit link`);

    const reopenedContext = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const reopened = await reopenedContext.newPage();
    try {
      await reopened.goto(editLink, { waitUntil: "domcontentloaded" });
      await reopened.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
      await reopened.locator("#screen-editor [data-shell-space='lab']").click();
      await reopened.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
      await reopened.waitForSelector("[data-curriculum-inquiry-panel]:not(.hidden)", { state: "visible", timeout: 20_000 });
      const reopenedState = await reopened.evaluate(async () => {
        const artifact = window.__SEAMGRIM_CURRICULUM_INQUIRY__ ?? null;
        const snapshot = await window.__SEAMGRIM_BUILD_CURRENT_PROJECT_SNAPSHOT__({ projectName: document.querySelector("#project-name-input")?.value ?? "" });
        return {
          prediction: String(document.querySelector("textarea[data-curriculum-inquiry-prediction]")?.value ?? ""),
          explanation: String(document.querySelector("textarea[data-curriculum-inquiry-explanation]")?.value ?? ""),
          artifact,
          snapshotArtifact: snapshot.project_snapshot.curriculum_inquiry_artifact ?? null,
        };
      });
      assert(reopenedState.prediction === world.prediction, `${world.lessonId}: reopened prediction`);
      assert(reopenedState.explanation === world.explanation, `${world.lessonId}: reopened explanation`);
      assert(reopenedState.artifact?.lesson_id === world.lessonId, `${world.lessonId}: reopened artifact lesson`);
      assert(
        String(reopenedState.snapshotArtifact?.run_evidence?.runtime_hash ?? "").includes("blake3:")
          || String(reopenedState.snapshotArtifact?.run_evidence?.runtime_hash ?? "").startsWith("server-fallback"),
        `${world.lessonId}: reopened hash`,
      );
    } finally {
      await reopenedContext.close();
    }
    if (failures.length) throw new Error(failures.join("\n"));
    return {
      lesson_id: world.lessonId,
      runtime_hash: artifactBeforeSave.run_evidence.runtime_hash,
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
    const worlds = [];
    for (const world of WORLDS) {
      worlds.push(await verifyWorld(browser, baseUrl, world));
    }
    console.log(JSON.stringify({
      schema: "seamgrim.projectile_damping_promotion.runner_result.v1",
      status: "ok",
      worlds,
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
