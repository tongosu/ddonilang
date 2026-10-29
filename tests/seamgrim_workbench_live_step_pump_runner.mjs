#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
const { chromium } = await import(process.env.DDN_PLAYWRIGHT_MODULE
  ? pathToFileURL(path.resolve(process.env.DDN_PLAYWRIGHT_MODULE)).href : "playwright");

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
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
      const isolatedWasm = process.env.DDN_WASM_DIR
        && relative.startsWith("solutions/seamgrim_ui_mvp/ui/wasm/");
      const bytes = await fs.readFile(isolatedWasm
        ? path.join(path.resolve(process.env.DDN_WASM_DIR), path.basename(file)) : file);
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
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.());
}

async function createSingleProject(page, name) {
  const previous = await snapshot(page);
  if (await page.getByTestId('project-rail-toggle').getAttribute('aria-expanded') !== 'true') {
    await page.getByTestId('project-rail-toggle').click();
  }
  await page.click("[data-testid='new-ddn-project']");
  await page.waitForSelector("[data-testid='project-create-dialog'][open]");
  await page.fill("[data-testid='project-create-name']", name);
  await page.check("input[name='project-template'][value='single']");
  await page.click("[data-testid='project-create-submit']");
  await page.waitForFunction(
    (previousId) => {
      const current = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return current?.projectId !== previousId && current?.project?.native_creator === true;
    },
    previous?.projectId ?? "",
  );
  // Project identity changes before asynchronous source loading completes.
  // Editing at that earlier point lets the template load overwrite the test.
  await page.waitForSelector("[data-testid='project-create-dialog']", { state: "hidden" });
}

async function waitForState(page, state) {
  try {
    await page.waitForFunction(
      (expected) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status === expected,
      state,
      { timeout: 60000 },
    );
  } catch (err) {
    const current = await snapshot(page).catch(() => null);
    throw new Error(`state ${state} timeout: ${JSON.stringify(current)}\n${err?.message ?? err}`);
  }
  return snapshot(page);
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { server, baseUrl } = await serve(root);
  let browser = null;
  const pageErrors = [];
  const assetFailures = [];
  try {
    browser = await chromium.launch({ headless: true, executablePath: process.env.CHROME_EXECUTABLE || 'C:/Program Files/Google/Chrome/Application/chrome.exe' });
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, locale: "ko-KR" });
    page.on("pageerror", (err) => pageErrors.push(err.message));
    page.on("response", (response) => {
      if (response.status() >= 400 && /\.(?:js|wasm)(?:\?|$)/u.test(response.url())) {
        assetFailures.push(`${response.status()} ${response.url()}`);
      }
    });
    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
    await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()));

    await createSingleProject(page, "직렬 자동 펌프");
    const longLiveSource = `설정 {
  제목: "직렬 자동 펌프".
  마디수: 50.
}.

채비 {
  값: 수 := 0.
}.

(매마디)마다 {
  값 <- 값 + 1.
  값 보여주기.
}.`;
    await page.fill("[data-testid='source-editor']", longLiveSource);
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const status = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.executionLifecycle?.status;
      return status && status !== "idle" && status !== "preparing";
    }, null, { timeout: 60000 });
    const entered = await snapshot(page);
    if (process.env.DDN_EXPECT_PACING_REJECTION === "1") {
      assert(entered.executionLifecycle.status === "failed"
        && entered.diagnostics.some((row) => row.code === "E_ALPHA2_HOST_PACING_POLICY"),
      "missing host policy did not fail closed at real browser admission");
      assert(!entered.activeFrame, "host policy rejection published a world frame");
      console.log("ALPHA2_HOST_PACING_MISSING_POLICY: rejected without fallback or published frame");
      return;
    }
    assert(entered.executionLifecycle.mode === "live",
      `wrong source/mode at run entry: ${JSON.stringify({ lifecycle: entered.executionLifecycle,
        diagnostics: entered.diagnostics, source: await page.getByTestId("source-editor").inputValue() })}`);
    try {
      await page.waitForFunction(() => {
        const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        return snap?.executionLifecycle?.status === "running"
          && Number(snap?.runs?.[0]?.committed_step_count ?? 0) >= 2
          && Number(snap?.livePump?.step_commit_count ?? 0) >= 2;
      }, null, { timeout: 60000 });
    } catch (err) {
      const failed = await snapshot(page);
      throw new Error(`initial live run failed: ${JSON.stringify({lifecycle: failed?.executionLifecycle, diagnostics: failed?.diagnostics})}\n${err.message}`);
    }
    let snap = await snapshot(page);
    const runId = snap.activeRunId;
    assert(snap.livePump.step_commit_count >= 2, "live run did not advance without another click");
    assert(snap.livePump.max_in_flight === 1, `more than one WASM step was in flight: ${JSON.stringify(snap.livePump)}`);
    assert(snap.livePump.madi_hz === 60, "live pump did not consume the shared Rust 60Hz policy");
    assert(snap.livePump.scheduled_delay_ms >= 0 && snap.livePump.scheduled_delay_ms <= 1000 / 60,
      "live pump retained the old 32/250/500ms playback interval");
    assert(await page.getByTestId("playback-rate").isVisible(), "fixed host pacing is not shown to the user");
    assert(await page.getByTestId("playback-interval").count() === 0, "unapproved host rate selector remains");

    await page.click("[data-testid='lifecycle-run'][data-action='pause']");
    snap = await waitForState(page, "paused");
    const pausedCount = snap.runs[0].committed_step_count;
    await page.waitForTimeout(180);
    snap = await snapshot(page);
    assert(snap.runs[0].committed_step_count === pausedCount, "pause advanced beyond its committed boundary");
    assert(!snap.livePump.in_flight && !snap.livePump.scheduled, "paused pump retained work");
    const automaticPrefix = snap.runs[0].frames.slice(0, 3).map((frame) => ({
      madi: frame.madi, world: frame.state_hash,
      input: frame.input_snapshot.input_snapshot_sha256,
    }));
    assert(automaticPrefix.length === 3, "automatic run needs three actual committed steps");
    for (const frame of snap.runs[0].frames) {
      assert(frame.input_snapshot.dtText === "1", "host elapsed time leaked into world dt");
    }
    const semanticBeforeViewChange = snap.semanticViewPayload;
    assert(semanticBeforeViewChange?.committed_frame?.run_id === snap.activeRunId, "semantic payload run identity mismatch");
    assert(semanticBeforeViewChange?.committed_frame?.frame_id === snap.activeFrame.frame_id, "semantic payload frame identity mismatch");
    assert(semanticBeforeViewChange?.committed_frame?.state_hash === snap.activeFrame.state_hash, "semantic payload state hash mismatch");
    assert(semanticBeforeViewChange?.ordered_observations?.length > 0, "semantic payload observations missing");
    const canvasBinding = await page.evaluate(() => {
      const canvas = document.querySelector("[data-testid='bogae-canvas']");
      return {
        bound: canvas?.dataset?.committedFrameBound,
        stateHash: canvas?.dataset?.committedStateHash,
        payloadHash: canvas?.dataset?.semanticPayloadHash,
        viewArtifactHash: canvas?.dataset?.viewArtifactHash,
      };
    });
    assert(canvasBinding.bound === "true", "canvas is not bound to a committed semantic frame");
    assert(canvasBinding.stateHash === snap.activeFrame.state_hash, "canvas committed state hash mismatch");
    assert(canvasBinding.payloadHash === semanticBeforeViewChange.semantic_payload_hash, "canvas semantic payload hash mismatch");
    await page.click("[data-mode='bogae-main']");
    const alternateView = await snapshot(page);
    assert(alternateView.activeFrame.state_hash === snap.activeFrame.state_hash, "view config changed runtime state hash");
    assert(alternateView.semanticViewPayload.semantic_payload_hash !== semanticBeforeViewChange.semantic_payload_hash, "view config did not change semantic payload hash");
    assert(alternateView.semanticViewPayload.view_config.participates_in_state_hash === false, "view config entered runtime hash boundary");
    await page.click("[data-mode='ddn-main']");
    const restoredPrimary = await snapshot(page);
    const restoredArtifactHash = await page.locator("[data-testid='bogae-canvas']").getAttribute("data-view-artifact-hash");
    assert(restoredPrimary.semanticViewPayload.semantic_payload_hash === semanticBeforeViewChange.semantic_payload_hash, "same frame and restored view config changed semantic payload hash");
    assert(restoredArtifactHash === canvasBinding.viewArtifactHash, "same frame and restored view config changed view artifact hash");

    assert(!(await page.locator("[data-testid='action-controls']").isVisible()), "project without an action map advertised global actions");
    await page.click("[data-testid='lifecycle-step']");
    await page.waitForFunction(
      (count) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runs?.[0]?.committed_step_count === count + 1,
      pausedCount,
      { timeout: 60000 },
    );
    snap = await snapshot(page);
    assert(snap.executionLifecycle.status === "paused", "manual step did not remain paused");
    assert(snap.activeFrame.input_snapshot.intent === "step", "manual step did not record its InputSnapshot boundary");
    assert(snap.nextInput === null, "project without an action map created a queued action");
    await page.click("[data-testid='lifecycle-step']");
    await page.waitForFunction(
      (count) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runs?.[0]?.committed_step_count === count + 2,
      pausedCount,
      { timeout: 60000 },
    );
    const actionIntentCount = (await snapshot(page)).runs[0].input_snapshots
      .filter((input) => input?.action_id).length;
    assert(actionIntentCount === 0, `project without an action map committed ${actionIntentCount} actions`);

    const sealedSourceHash = snap.runs[0].source_hash;
    await page.fill("[data-testid='source-editor']", '"새 실행 후보" 보여주기.');
    await page.click("[data-testid='lifecycle-step']");
    await page.waitForFunction(
      (count) => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.runs?.[0]?.committed_step_count === count + 3,
      pausedCount,
      { timeout: 60000 },
    );
    snap = await snapshot(page);
    assert(snap.executionLifecycle.status === "paused", "source candidate changed the sealed active run");
    assert(snap.runs[0].source_hash === sealedSourceHash, "source candidate mutated active run source hash");

    const modifiedControl = await page.locator("[data-testid='lifecycle-run']").evaluate(button => ({
      action: button.dataset.action, label: button.getAttribute("aria-label"), enabled: !button.disabled,
    }));
    assert(modifiedControl.action === "start" && modifiedControl.label === "수정본 실행" && modifiedControl.enabled,
      "edited source must offer a new run instead of silently resuming the old source");
    await page.fill("[data-testid='source-editor']", longLiveSource);
    await page.waitForSelector("[data-testid='lifecycle-run'][data-action='resume']");
    assert((await snapshot(page)).runs[0].source_hash === sealedSourceHash, "restoring editor source mutated the active run");

    const beforeResume = snap.runs[0].committed_step_count;
    await page.click("[data-testid='lifecycle-run'][data-action='resume']");
    await page.waitForFunction(
      ({ id, count }) => {
        const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        const run = snap?.runs?.find((row) => row.run_id === id);
        return snap?.executionLifecycle?.status === "running" && Number(run?.committed_step_count ?? 0) >= count + 2;
      },
      { id: runId, count: beforeResume },
      { timeout: 60000 },
    );
    snap = await snapshot(page);
    assert(snap.activeRunId === runId && snap.runs.length === 1, "resume created a different run");

    const stopResults = await page.evaluate(async () => {
      const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
      const first = api.stopRun();
      const second = api.stopRun();
      return Promise.all([first, second]);
    });
    assert(stopResults[0] === true && stopResults[1] === false, `duplicate stop mismatch: ${JSON.stringify(stopResults)}`);
    snap = await waitForState(page, "done");
    assert(snap.runs[0].final_status === "stopped", "stop did not finalize the run once");
    assert(snap.livePump.max_in_flight === 1, "serial pump invariant regressed after resume");

    // Restart the exact same source, then advance at a deliberately slow manual
    // pace. The world/receipt equality comes from actual new WASM executions,
    // not a copied output or a replay-file format.
    await page.evaluate(async () => {
      const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
      await api.startRun();
      api.pauseRun();
    });
    snap = await waitForState(page, "paused");
    const restartedRunId = snap.activeRunId;
    let restarted = snap.runs.find((run) => run.run_id === restartedRunId);
    assert(restarted.frame_count === 1, "restart did not pause before its first automatic callback");
    for (let count = 1; count < 3; count += 1) {
      await page.waitForTimeout(80);
      await page.click("[data-testid='lifecycle-step']");
      await page.waitForFunction(({ id, count }) => {
        const snap = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
        return snap?.runs?.find((run) => run.run_id === id)?.frame_count === count;
      }, { id: restartedRunId, count: count + 1 }, { timeout: 60000 });
    }
    snap = await waitForState(page, "paused");
    restarted = snap.runs.find((run) => run.run_id === restartedRunId);
    const manualPrefix = restarted.frames.map((frame) => ({
      madi: frame.madi, world: frame.state_hash,
      input: frame.input_snapshot.input_snapshot_sha256,
    }));
    assert(JSON.stringify(manualPrefix) === JSON.stringify(automaticPrefix),
      `slow manual/restarted execution changed world or input hashes: ${JSON.stringify({automaticPrefix, manualPrefix})}`);
    assert(snap.livePump.madi_hz === 60, "new WASM policy is absent");
    assert(snap.activeFrame.wasm_state.input_receipt.input_snapshot.input.dt === "1", "actual Rust receipt changed dt");
    await page.click("[data-testid='lifecycle-stop']");
    await waitForState(page, "done");

    await createSingleProject(page, "설정 마디 자동 완료");
    const boundedLiveSource = `설정 {
  제목: "설정 마디 자동 완료".
  마디수: 3.
}.

채비 {
  값: 수 := 0.
}.

(매마디)마다 {
  값 <- 값 + 1.
}.`;
    await page.fill("[data-testid='source-editor']", boundedLiveSource);
    await page.click("[data-testid='lifecycle-run']");
    snap = await waitForState(page, "done");
    assert(snap.executionLifecycle.mode === "live", "bounded live source lost live mode");
    assert(snap.runs[0].configured_madi === 3, `configured madi mismatch: ${JSON.stringify(snap.runs[0])}`);
    assert(snap.runs[0].committed_step_count === 3, `bounded run commit count mismatch: ${JSON.stringify(snap.runs[0])}`);
    assert(snap.runs[0].final_status === "completed", "bounded run did not finalize completed");
    assert(snap.livePump.max_in_flight === 1, "bounded run exceeded one step in flight");

    // Run the actual authored collection world unchanged through the editor,
    // then edit an authored scoring rule and rerun. This is not input replay.
    const worldSource = await fs.readFile(path.join(root,
      "solutions/seamgrim_ui_mvp/samples/alpha2_collection_world.ddn"), "utf8");
    const worldExpected = ['참','1','4','5','3','2','0','0','0','참',
      '3','2','2','1','1','참','참','참','참','4',
      '200','2','1','200','50','1','참','참'];
    // A world digest is a comparison result, not a hand-calculated oracle.
    // Derive it from the exact WASM served to this browser, while retaining
    // the independent output/score/reset expectations above.
    const worldPath = path.join(root, "solutions/seamgrim_ui_mvp/samples/alpha2_collection_world.ddn");
    const wasmDirectory = path.resolve(process.env.DDN_WASM_DIR
      || path.join(root, "solutions/seamgrim_ui_mvp/ui/wasm"));
    const directWasm = await import(pathToFileURL(path.join(wasmDirectory, "ddonirang_tool.js")).href);
    await directWasm.default({module_or_path:await fs.readFile(path.join(wasmDirectory, "ddonirang_tool_bg.wasm"))});
    const directWorld = JSON.parse(directWasm.wasm_run_summary_with_source_profile_json(
      worldSource, worldPath, 1, "v1-core-v25"));
    assert(JSON.stringify(directWorld.resources?.value_json?.output_log?.map(row => row.text))
      === JSON.stringify(worldExpected), "raw WASM violates independent world rules");
    assert(/^blake3:[0-9a-f]{64}$/.test(directWorld.state_hash), "raw WASM world hash missing");
    let nativeAliases = 0;
    if (process.env.DDN_NATIVE_DIR) {
      for (const alias of ['ddonilang','ddonirang','또니랑']) {
        const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
          ['실행',worldPath,'--source-profile','v1-core-v25','--마디수','1','--json'],
          {encoding:'utf8',timeout:30000,maxBuffer:4*1024*1024});
        assert(!run.error && run.status === 0, `${alias} world execution failed: ${run.error || run.stderr}`);
        const nativeWorld = JSON.parse(run.stdout);
        assert(nativeWorld.state_hash === directWorld.state_hash, `${alias}/raw WASM world differs`);
        assert(JSON.stringify(nativeWorld.resources?.value_json?.output_log?.map(row => row.text))
          === JSON.stringify(worldExpected), `${alias} violates independent world rules`);
        nativeAliases++;
      }
    }
    await createSingleProject(page, "칸 작품 규칙");
    await page.getByTestId("source-editor").fill(worldSource);
    await page.getByTestId("lifecycle-run").click();
    snap = await waitForState(page, "done");
    assert(snap.executionLifecycle.mode === "oneshot", "collection rules became a fictitious live game");
    assert(snap.runs[0].final_status === "completed", "collection world failed in actual browser");
    const worldValues = snap.activeFrame.wasm_state.resources.value_json;
    assert(JSON.stringify(worldValues.output_log.map(row => row.text)) === JSON.stringify(worldExpected),
      "browser output differs from independently derived collection world rules");
    assert(worldValues.낙하뒤[2][0][0] === 200 && worldValues.재시작뒤[2][0][0] === 0,
      "actual browser failed score/reset state transition");
    assert(snap.activeFrame.state_hash === directWorld.state_hash,
      `actual browser differs from raw WASM world: ${snap.activeFrame.state_hash}`);
    const browserWorldHash = snap.activeFrame.state_hash;
    const browserWorldRunId = snap.activeRunId;
    const renderedText = await page.getByTestId("bogae-text-output-lines").textContent();
    assert(renderedText.includes("200"), "actual world output is not rendered for the learner");

    const changedSource = worldSource.replace("점수 + 지운줄 * 100", "점수 + 지운줄 * 125");
    assert(changedSource !== worldSource, "scoring edit did not modify the original source");
    await page.getByTestId("source-editor").fill(changedSource);
    await page.getByTestId("lifecycle-run").click();
    // The previous one-shot remains done while asynchronous preparation starts.
    // Wait for a new run identity, rather than reading that old done snapshot.
    await page.waitForFunction((previousId) => {
      const current = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return current?.activeRunId && current.activeRunId !== previousId;
    }, browserWorldRunId, {timeout:30000});
    snap = await waitForState(page, "done");
    const changedValues = snap.activeFrame.wasm_state.resources.value_json;
    assert(snap.activeRunId !== browserWorldRunId, "source edit did not create a new execution");
    assert(changedValues.낙하뒤[2][0][0] === 250 && changedValues.이동뒤[2][0][0] === 250,
      "edited authored score rule did not reach the WASM world");
    assert(changedValues.재시작뒤[2][0][0] === 0, "edited rule broke initial-state reset");
    assert(snap.activeFrame.state_hash !== browserWorldHash, "changed world retained a stale hash");
    const preservedRun = snap.runs.find(run => run.run_id === browserWorldRunId);
    assert(preservedRun?.latest_state_hash === browserWorldHash, "rerun mutated prior accepted world history");
    console.log(JSON.stringify({gate:"ALPHA2_COLLECTION_WORLD_BROWSER", result:"PASS",
      source_sha256:createHash("sha256").update(worldSource).digest("hex"),
      state_hash:browserWorldHash, output_checks:worldExpected.length,
      actual_raw_wasm:true, native_aliases:nativeAliases,
      edited_score:250, restart_score:0, source_lowering:0,
      scope:"Actual editor/one-shot output/edit/rerun/history; not interactive Tetris"}));
    assert(pageErrors.length === 0, `page errors: ${pageErrors.join(" | ")}`);
    assert(assetFailures.length === 0, `asset request failures: ${assetFailures.join(" | ")}`);

    if (process.env.DDN_WASM_DIR) {
      const bytes = await fs.readFile(path.join(process.env.DDN_WASM_DIR, "ddonirang_tool_bg.wasm"));
      console.log(JSON.stringify({ gate: "ALPHA2_HOST_PACING_BROWSER", result: "PASS", madi_hz: 60,
        world_dt: "1", actual_madi_hash_comparisons: automaticPrefix.length,
        wasm_sha256: createHash("sha256").update(bytes).digest("hex"),
        asset_failures: assetFailures.length, page_errors: pageErrors.length }));
    }

    console.log("seamgrim_workbench_live_step_pump: ok");
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
