#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js") || file.endsWith(".mjs")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  return "application/octet-stream";
}

async function serve(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const relative = decodeURIComponent(url.pathname === "/"
        ? "/solutions/seamgrim_ui_mvp/ui/workbench/index.html"
        : url.pathname).replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, relative);
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
        res.writeHead(403).end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
      res.end(bytes);
    } catch (_) {
      res.writeHead(404).end("not found");
    }
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert(address && typeof address !== "string", "server address unavailable");
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

async function snapshot(page) {
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
}

async function waitForControls(page, projectId) {
  await page.waitForFunction((expected) => {
    const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
    return snap?.projectId === expected
      && snap?.projectControls?.project_id === expected
      && snap?.projectControls?.status === "ready";
  }, projectId, { timeout: 60000 });
  return snapshot(page);
}

async function selectProject(page, projectId) {
  const card = page.locator(`[data-testid='project-card-${projectId}']`);
  if (!(await card.isVisible())) await page.getByRole('button', { name: '프로젝트', exact: true }).click();
  await page.click(`[data-testid='project-card-${projectId}']`);
  return waitForControls(page, projectId);
}

async function main() {
  const root = process.env.DDN_TEST_ROOT
    ? path.resolve(process.env.DDN_TEST_ROOT)
    : path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { server, baseUrl } = await serve(root);
  let browser = null;
  const pageErrors = [];
  try {
    browser = await chromium.launch({ headless: true, executablePath: process.env.CHROME_EXECUTABLE || undefined });
    const page = await browser.newPage({ viewport: { width: 1360, height: 900 }, locale: "ko-KR" });
    page.on("pageerror", (err) => pageErrors.push(err.message));
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    let snap = await waitForControls(page, "rep_phys_projectile_xy_v1");

    const sourceFileCheck = await page.evaluate(async () => {
      const { readTextFromLocalFile } = await import("../screens/editor.js");
      const source = '값 <- 1.\n값 보여주기.';
      const normal = await readTextFromLocalFile(new File([source], "normal.ddn"));
      let bomError = "";
      try {
        await readTextFromLocalFile(new File([new Uint8Array([239, 187, 191]), source], "bom.ddn"));
      } catch (error) { bomError = String(error); }
      return { normal, source, bomError };
    });
    assert(sourceFileCheck.normal === sourceFileCheck.source, "browser File bytes changed accepted source");
    assert(sourceFileCheck.bomError.includes("E_SOURCE_UTF8_BOM_FORBIDDEN"), "browser File ingress silently removed BOM");
    console.log("SOURCE_FILE_BROWSER_PASS normal=1 bom_rejected=1");

    const projectileNames = snap.projectControls.controls.map((control) => control.name);
    if (!projectileNames.length) {
      console.error("control source diagnostic", await page.evaluate(async () => {
        const source = document.querySelector("#source-editor")?.value ?? "";
        const module = await import("../wasm/ddonirang_tool.js");
        await module.default();
        return { source, plan: module.wasm_canon_maegim_plan(source) };
      }));
    }
    assert(
      projectileNames.join(",") === "초기속도x,초기속도y,중력가속도,dt,마디수",
      `projectile controls did not come from its WASM maegim plan: ${JSON.stringify({names: projectileNames, controls: snap.projectControls, diagnostics: snap.diagnostics, pageErrors})}`,
    );
    assert(snap.projectControls.source === "rust_wasm_authoring_plan", "project controls did not report Rust/WASM authority");
    assert(snap.projectControls.authoring_plan_hash, "project authoring-plan hash missing");
    assert(await page.locator("[data-testid='setup-control-list'] input").count() === projectileNames.length, "setup DOM does not match the current plan");
    assert(!(await page.locator("[data-testid='action-controls']").isVisible()), "action UI appeared without a project action map");
    assert(await page.locator("[data-testid='action-left']").count() === 0, "global left arrow survived without an action map");

    const gravity = page.locator("[data-testid='setup-control-list'] input[data-control-name='중력가속도']");
    await gravity.fill("12.3");
    await page.click("[data-testid='setup-apply']");
    snap = await snapshot(page);
    assert(snap.pendingRecipe?.schema === "seamgrim.workbench.run_recipe_candidate.v1", "setup change did not create a run recipe candidate");
    assert(snap.pendingRecipe?.values?.중력가속도 === 12.3, `recipe did not capture gravity: ${JSON.stringify(snap.pendingRecipe)}`);
    assert(snap.activeFrame === null, "setup recipe mutated a world before a new run");

    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return Boolean(snap?.activeFrame) || snap?.executionLifecycle?.status === "failed";
    }, null, { timeout: 60000 });
    snap = await snapshot(page);
    assert(
      snap.activeFrame?.wasm_state?.param_overrides?.중력가속도?.raw_i64 === "52828097741",
      `WASM run did not apply the recipe override: ${JSON.stringify({ lifecycle: snap.executionLifecycle, setup: snap.runs[0]?.setup, diagnostics: snap.diagnostics, params: snap.activeFrame?.wasm_state?.param_overrides })}`,
    );
    assert(snap.runs[0]?.setup?.values?.중력가속도 === 12.3, "sealed run did not retain its setup recipe");
    assert(snap.pendingRecipe === null, "accepted recipe was not consumed by the new run");
    if (snap.executionLifecycle.status === "running") {
      await page.click("[data-testid='lifecycle-run'][data-action='pause']");
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "paused");
    }
    snap = await snapshot(page);
    const committedHashBeforeNextRecipe = snap.activeFrame.state_hash;
    const committedGravityBeforeNextRecipe = snap.activeFrame.wasm_state.param_overrides.중력가속도;
    await gravity.fill("10.7");
    await page.click("[data-testid='setup-apply']");
    snap = await snapshot(page);
    assert(snap.pendingRecipe?.values?.중력가속도 === 10.7, "active-run setup change did not remain a next-run candidate");
    assert(snap.activeFrame.state_hash === committedHashBeforeNextRecipe, "setup candidate mutated the committed active run");
    assert(JSON.stringify(snap.activeFrame.wasm_state.param_overrides.중력가속도) === JSON.stringify(committedGravityBeforeNextRecipe), "setup candidate changed the active VM parameter");

    snap = await selectProject(page, "rep_phys_pendulum_wave1_v1");
    const pendulumNames = snap.projectControls.controls.map((control) => control.name);
    assert(
      pendulumNames.join(",") === "g,L,theta0,omega0,damp,dt,t_max",
      `pendulum controls did not replace projectile controls: ${JSON.stringify(pendulumNames)}`,
    );
    assert(!pendulumNames.includes("중력가속도") && !pendulumNames.includes("초기속도x"), "previous project controls leaked after switch");
    assert(snap.pendingRecipe === null && snap.nextInput === null, "project switch retained pending world input");
    assert(!(await page.locator("[data-testid='action-controls']").isVisible()), "pendulum advertised actions without an action map");

    snap = await selectProject(page, "rep_grid_game_state_drop_v1");
    assert(snap.projectControls.controls.length === 0, `grid advertised undeclared setup controls: ${JSON.stringify(snap.projectControls.controls)}`);
    assert(!(await page.locator("[data-testid='setup-controls']").isVisible()), "empty setup card occupied the first screen");
    assert(await page.locator("[data-testid='action-controls']").isVisible(), "declared grid action map did not create action UI");
    assert(snap.projectControls.actions.map((action) => action.action_id).join(",") === "left,right,drop", "grid action map order/content mismatch");
    assert(snap.projectControls.action_map_hash, "grid action-map hash missing");

    await page.click("[data-testid='action-drop']");
    snap = await snapshot(page);
    assert(snap.nextInput?.action_id === "drop", "action did not identify its declared action-map row");
    assert(snap.nextInput?.intent === "drop" && snap.nextInput?.keys === 4 && snap.nextInput?.lastKey === "ArrowDown", "action did not create the declared next InputSnapshot");
    assert(snap.nextInput?.action_map_hash === snap.projectControls.action_map_hash, "queued input lost action-map lineage");
    const queuedDropInputId = snap.nextInput.input_id;

    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snap?.runs?.[0]?.input_snapshots?.some((input) => input?.action_id === "drop");
    }, null, { timeout: 60000 });
    snap = await snapshot(page);
    const dropInputs = snap.runs[0].input_snapshots.filter((input) => input?.action_id === "drop");
    assert(
      dropInputs.length === 1,
      `declared action committed ${dropInputs.length} times instead of once: ${JSON.stringify(snap.runs[0].input_snapshots)}`,
    );
    // I1: the first frame is now an actual calculation, not an unexecuted reset.
    assert(snap.runs[0].input_snapshots[0]?.action_id === "drop", "first actual calculation lost its queued action");
    assert(snap.runs[0].input_snapshots[0]?.keys === 4, "first actual calculation did not receive the declared keys");
    assert(dropInputs[0].input_id === queuedDropInputId, "committed InputSnapshot lost its queued input identity");
    assert(dropInputs[0].action_map_hash === snap.projectControls.action_map_hash, "committed InputSnapshot lost action-map lineage");
    assert(dropInputs[0].workspace_identity === "rep_grid_game_state_drop_v1", "committed InputSnapshot lost workspace identity");
    assert(dropInputs[0].document_identity === "seamgrim://rep_grid_game_state_drop_v1/lesson.ddn", "committed InputSnapshot lost document identity");
    assert(Number.isInteger(dropInputs[0].document_version) && dropInputs[0].document_version > 0, "committed InputSnapshot lost document version");
    assert(dropInputs[0].source_profile_identity === "v1-core-v25", "committed InputSnapshot lost source profile");
    assert(dropInputs[0].view_identity === "view_primary_1", "committed InputSnapshot lost primary view identity");
    assert(dropInputs[0].bogae_identity === "grid", "committed InputSnapshot lost Bogae identity");
    assert(dropInputs[0].provider_identity === "ddonirang_tool.wasm_runtime", "committed InputSnapshot lost shared provider identity");
    assert(dropInputs[0].source_identity === snap.runs[0].shared_runtime_result?.source_identity, "committed InputSnapshot lost shared source identity");
    assert(dropInputs[0].world_state_identity === snap.runs[0].frames.at(-1)?.state_hash, "committed InputSnapshot lost committed world identity");
    assert(/^[0-9a-f]{64}$/u.test(dropInputs[0].snapshot_hash ?? ""), "committed input must use the Rust SHA-256 receipt");

    if (snap.executionLifecycle.status === "running") {
      await page.click("[data-testid='lifecycle-run'][data-action='pause']");
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status === "paused");
    }
    snap = await snapshot(page);
    const tapeWorldHash = snap.runs[0].latest_state_hash;
    await page.locator("[data-testid='project-more-menu'] summary").click();
    await page.click("[data-testid='project-history-open']");
    const downloadPromise = page.waitForEvent("download", { timeout: 15000 });
    await page.click("[data-testid='input-tape-export']");
    const download = await downloadPromise;
    const tapePath = await download.path();
    const tapeBytes = await fs.readFile(tapePath);
    assert(tapeBytes.subarray(0, 18).toString("utf8") === "DDN_INPUT_TAPE_V1\n", "screen did not export the existing Tape V1 bytes");
    await page.locator("[data-testid='input-tape-file']").setInputFiles(tapePath);
    await page.waitForFunction((hash) => document.querySelector("[data-testid='input-tape-status']").textContent.includes(hash), tapeWorldHash, { timeout: 60000 });
    assert((await page.locator("[data-testid='input-tape-status']").textContent()).includes("입력 기록 계산 완료"), "downloaded file did not return to actual WASM");
    await page.click("[data-testid='project-history-close']");
    console.log("I1_BROWSER_TAPE_V1_FILE_ROUNDTRIP_PASS");

    const previousRunId = (await snapshot(page)).activeRunId;
    await page.locator("#source-editor").fill("(매마디)마다 { 입력키 보여주기. }.");
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction((oldId) => {
      const next = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return next.activeRunId !== oldId && next.runs.at(-1)?.frame_count > 0;
    }, previousRunId, { timeout: 60000 });
    if ((await snapshot(page)).executionLifecycle.status === "running") {
      await page.click("[data-testid='lifecycle-run'][data-action='pause']");
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status === "paused");
    }
    await page.locator("[data-testid='project-more-menu'] summary").click();
    await page.click("[data-testid='project-history-open']");
    let forbiddenDownloads = 0;
    const countDownload = () => { forbiddenDownloads += 1; };
    page.on("download", countDownload);
    await page.click("[data-testid='input-tape-export']");
    await page.waitForFunction(() => document.querySelector("[data-testid='input-tape-status']").textContent.includes("E_INPUT_TAPE_V1_UNRECORDED_READ"), null, { timeout: 15000 });
    assert(forbiddenDownloads === 0, "unsupported actual read still published a tape");
    page.off("download", countDownload);
    await page.click("[data-testid='project-history-close']");
    console.log("I1_BROWSER_TAPE_V1_READ_REJECTION_PASS");

    snap = await selectProject(page, "rep_phys_pendulum_wave1_v1");
    assert(snap.nextInput === null, "queued action leaked into the next project");
    assert(await page.locator("[data-testid='action-left']").count() === 0, "grid action DOM leaked into a project without an action map");
    assert(!(await page.locator("[data-testid='action-controls']").isVisible()), "action card remained visible after project switch");

    const viewBefore = structuredClone(snap.viewWorkspace);
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.activeFrame));
    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === "paused");
    const hashBeforeView = (await snapshot(page)).activeFrame.state_hash;
    await page.click("[data-testid='view-role-side']");
    snap = await snapshot(page);
    assert(snap.activeFrame.state_hash === hashBeforeView, "view setting entered the world state hash");
    assert(snap.pendingRecipe === null && snap.nextInput === null, "view setting created a setup recipe or InputSnapshot");
    assert(snap.viewWorkspace.instances.length > viewBefore.instances.length, "view setting did not stay in the view snapshot");

    const rawV25Source = "매마디:움직씨 := {\n    값:수 := 1.\n    값 보여주기.\n}\n";
    await page.locator("#source-editor").fill(rawV25Source);
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const next = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return Boolean(next?.runs?.at(-1)?.input_snapshots?.at(-1)?.snapshot_hash)
        || next?.executionLifecycle?.status === "failed";
    }, null, { timeout: 60000 });
    snap = await snapshot(page);
    const rawRun = snap.runs.at(-1);
    const rawSnapshot = rawRun?.input_snapshots?.at(-1);
    assert(snap.executionLifecycle.status !== "failed", `supported raw V25 source failed: ${JSON.stringify(snap.diagnostics)}`);
    assert(rawSnapshot?.document_identity === "seamgrim://rep_phys_pendulum_wave1_v1/lesson.ddn", "raw V25 run lost document identity");
    assert(Number.isInteger(rawSnapshot?.document_version) && rawSnapshot.document_version > 0, "raw V25 run lost document version");
    assert(rawSnapshot?.source_profile_identity === "v1-core-v25", "raw V25 run lost source profile");
    assert(rawSnapshot?.view_identity === "view_primary_1", "raw V25 run lost view identity");
    assert(rawSnapshot?.bogae_identity === "pendulum", "raw V25 run lost Bogae identity");
    assert(rawSnapshot?.provider_identity === "ddonirang_tool.wasm_runtime", "raw V25 run lost provider identity");
    assert(rawSnapshot?.source_identity === rawRun.shared_runtime_result?.source_identity, "raw V25 run lost shared source binding");
    assert(rawSnapshot?.world_state_identity === rawRun.frames.at(-1)?.state_hash, "raw V25 run lost world-state binding");
    assert(/^[0-9a-f]{64}$/u.test(rawSnapshot?.snapshot_hash ?? ""), "raw V25 input must use the Rust SHA-256 receipt");

    // Compare actual Worker/WASM frames, not UI intent or wall-clock metadata.
    async function inputFrames(manual, minimum) {
      const probe = await browser.newPage();
      try {
        await probe.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
        await waitForControls(probe, "rep_phys_projectile_xy_v1");
        await selectProject(probe, "rep_grid_game_state_drop_v1");
        await probe.locator("#source-editor").fill("(매마디)마다 { 입력키 보여주기. }.");
        await waitForControls(probe, "rep_grid_game_state_drop_v1");
        await probe.getByTestId("action-drop").click();
        await probe.getByTestId("lifecycle-run").click();
        await probe.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.at(-1)?.frame_count >= 1);
        if (!manual) {
          await probe.waitForFunction(n => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.at(-1)?.frame_count >= n, minimum);
        }
        await probe.locator("[data-testid='lifecycle-run'][data-action='pause']").click();
        await probe.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status === "paused");
        const initial = (await snapshot(probe)).runs.at(-1).frame_count;
        if (manual) {
          for (let i = 0; i < 3; i += 1) {
            await probe.getByTestId("lifecycle-step").click();
            await probe.waitForFunction(n => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.at(-1)?.frame_count === n, initial + i + 1);
          }
        }
        const inputs = (await snapshot(probe)).runs.at(-1).input_snapshots;
        return inputs.map(input => ({ input: input.snapshot_hash, world: input.world_state_identity }));
      } finally {
        await probe.close();
      }
    }
    const manualFrames = await inputFrames(true, 1);
    const autoFrames = await inputFrames(false, manualFrames.length);
    assert(JSON.stringify(autoFrames.slice(0, manualFrames.length)) === JSON.stringify(manualFrames),
      "continuous/manual actual input receipts or world hashes differ");
    console.log(`I1_BROWSER_CONTINUOUS_MANUAL_PASS frames=${manualFrames.length} full_input_file_ingress=0`);
    assert(pageErrors.length === 0, `page errors: ${pageErrors.join(" | ")}`);
    console.log("seamgrim_workbench_project_controls_browser: ok");
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
