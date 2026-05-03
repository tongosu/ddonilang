#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function sortJson(value) {
  if (Array.isArray(value)) return value.map((item) => sortJson(item));
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sortJson(value[key])]));
}

function createRunScreenRoot() {
  return {
    dataset: {},
    querySelector() {
      return null;
    },
  };
}

function createFakeEl() {
  return {
    textContent: "",
    innerHTML: "",
    title: "",
    disabled: false,
    classList: {
      hidden: false,
      toggle(name, force) {
        if (name === "hidden") this.hidden = Boolean(force);
      },
    },
  };
}

async function main() {
  const root = process.cwd();
  const packDir = path.join(root, "pack", "seamgrim_editor_run_transaction_v1");
  const contract = JSON.parse(await fs.readFile(path.join(packDir, "contract.detjson"), "utf8"));
  const stateMod = await import(pathToFileURL(path.join(root, "solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js")).href);
  const runMod = await import(pathToFileURL(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js")).href);
  const appText = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/app.js"), "utf8");
  const runText = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js"), "utf8");
  const stylesText = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/styles.css"), "utf8");

  const minimalState = {
    schema: "seamgrim.state.v0",
    channels: [{ key: "인사", dtype: "str", role: "state" }],
    row: ["hello"],
    resources: { json: {}, fixed64: {}, handle: {}, value: {}, value_json: {}, component: {} },
    observation_manifest: { nodes: [{ name: "인사", dtype: "str", role: "state" }] },
  };
  const rows = stateMod.extractObservationOutputRowsFromState(minimalState);
  const expected = contract.expected_output_row ?? {};
  const hit = rows.find((row) => String(row.key) === expected.key && String(row.value) === expected.value);
  assert(hit, "transaction: minimal output row missing");
  assert(runMod.resolveRunPostExecuteTab({ outputRows: rows }) === contract.expected_post_execute_tab, "transaction: output tab not preferred");
  const mainVisual = runMod.resolveRunMainVisualMode({ outputRows: rows });
  assert(mainVisual.mode === "console-grid", "transaction: show output should use console-grid bogae");
  assert(Array.isArray(mainVisual.consoleLinesForGrid) && mainVisual.consoleLinesForGrid.includes("hello"), "transaction: console-grid should show raw value");
  assert(!mainVisual.consoleLinesForGrid.includes("인사=hello"), "transaction: console-grid should not show diagnostic key=value");
  assert(String(mainVisual.consoleHtml ?? "").includes(">hello<"), "transaction: console HTML should show raw value");
  assert(!String(mainVisual.consoleHtml ?? "").includes("인사=hello"), "transaction: console HTML should not show diagnostic key=value");
  assert(!String(mainVisual.consoleHtml ?? "").includes("[0마디] 출력"), "transaction: simple console HTML should not show tick group title");

  const loadLessonPos = appText.indexOf("runScreen.loadLesson(lesson");
  const enqueuePos = appText.indexOf("runScreen.enqueueRunRequest", loadLessonPos);
  const setScreenPos = appText.indexOf('setScreen("run");', enqueuePos);
  assert(loadLessonPos >= 0 && enqueuePos > loadLessonPos && setScreenPos > enqueuePos, "transaction: app handoff order invalid");
  assert(!appText.includes("window.setTimeout?.(() => runScreen.requestAutoExecute(), 0);"), "transaction: retry-based autoExecute should be removed");
  assert(appText.includes("runRequest: {") && appText.includes("sourceText: effectiveDdnText"), "transaction: editor run request snapshot missing");
  assert(runText.includes("pendingRunRequest") && runText.includes("activeRunRequestId") && runText.includes("completedRunRequestId"), "transaction: request state fields missing");
  assert(runText.includes("async executeRunRequest(request = {})"), "transaction: executeRunRequest missing");
  assert(runText.includes("if (runRequestId && this.activeRunRequestId !== runRequestId) return false;"), "transaction: stale result guard missing");
  assert(
    runText.includes("readConfiguredMadiFromClient(result.client)")
      && runText.includes("const shouldBatchConfiguredMadi = configuredMadi > 0 && engineMode !== RUN_ENGINE_MODE_LIVE;")
      && runText.includes("this.runtimeTickCounter = configuredMadi;"),
    "transaction: configured live 마디수 must render per frame while non-live can batch",
  );
  assert(
    runText.includes("resolveRunEngineModeFromDdnText(`${rawDdnText}\\n${wasmDdnText}`)")
      && runText.includes("resolveRunEngineModeFromDdnText(`${this.baseDdn}\\n${this.getEffectiveWasmSource(this.baseDdn)}`)"),
    "transaction: live engine mode must consider both raw and wasm source",
  );
  assert(
    runText.includes("if (maxMadi > 0 && this.runtimeTickCounter >= maxMadi)")
      && runText.includes('this.setEngineStatus("done");'),
    "transaction: live 마디수 must stop when max is reached",
  );
  assert(
    runText.includes("hasReachedRuntimeMaxMadi(this.runtimeTickCounter, this.resolveRuntimeMaxMadiLimit())")
      && runText.includes('this.runStepBtn.disabled = isRunning || engineMode === RUN_ENGINE_MODE_ONESHOT || reachedMaxMadi')
      && runText.includes("const restarted = await this.restart({ autoStartLive: false });")
      && runText.includes("const shouldAutoStartLive = Boolean(autoStartLive);")
      && runText.includes("shouldAutoStartLive && configuredMadi <= 0")
      && runText.includes('this.lastExecPathHint = "한마디 실행 준비";')
      && runText.includes('this.setEngineStatus("paused");'),
    "transaction: one-step control must initialize without starting the live wasm loop",
  );
  assert(
    runText.includes("resolveRuntimeMaxMadiLimit()")
      && runText.includes("readConfiguredMadiFromClient(this.wasmState?.client)")
      && runText.includes("const maxMadi = this.resolveRuntimeMaxMadiLimit();"),
    "transaction: pause/reset/step paths must resync 마디수 from wasm/source",
  );
  assert(runText.includes('markRuntimeDirtyForSourceEdit({ showFallbackGrid = false }'), "transaction: dirty editor should use fixed bogae shell");
  assert(runText.includes('this.renderMainVisual({ mode: "none" });'), "transaction: dirty editor should reset to fixed waiting stage");
  assert(
    runText.includes('const showConsole = nextMode === "console" || nextMode === "console-grid" || nextMode === "none";'),
    "transaction: fixed waiting stage and console-grid html fallback missing",
  );
  assert(
    stylesText.includes('#screen-run[data-main-visual-mode="console-grid"] #run-main-console-host {\n  display: flex;') ||
      stylesText.includes('#screen-run[data-main-visual-mode="console-grid"] #run-main-console-host'),
    "transaction: console-grid console host css missing",
  );
  assert(
    !stylesText.includes('#screen-run[data-main-visual-mode="console-grid"] #run-main-console-host,\n#screen-run[data-main-visual-mode="console-grid"] #overlay-description'),
    "transaction: console-grid console host must not be hidden",
  );

  const screen = new runMod.RunScreen({ root: createRunScreenRoot(), wasmState: { fpsLimit: 30, dtMax: 0.1 } });
  screen.lesson = { ddnText: "" };
  screen.baseDdn = "설정 { 마디수: 4000. }.\n(매마디)마다 {\n  t <- t + 1.\n}.";
  screen.currentDdn = screen.baseDdn;
  screen.runtimeMaxMadi = 0;
  assert(screen.resolveRuntimeMaxMadiLimit() === 4000, "transaction: runtime max 마디수 must recover from current source");
  screen.runtimeMaxMadi = 0;
  screen.currentDdn = "";
  screen.baseDdn = "";
  screen.wasmState.client = { configuredMadi: () => 4000 };
  assert(screen.resolveRuntimeMaxMadiLimit() === 4000, "transaction: runtime max 마디수 must recover from wasm client");
  let restartCount = 0;
  screen.restart = async ({ runRequestId = "" } = {}) => {
    restartCount += 1;
    assert(runRequestId === "r1", "transaction: restart receives request id");
    assert(screen.currentDdn === contract.minimal_ddn, "transaction: currentDdn pinned to request source");
    return true;
  };

  const queued = screen.enqueueRunRequest({
    id: "r1",
    sourceText: contract.minimal_ddn,
    launchKind: "editor_run",
    sourceType: "ddn",
    createdAtMs: 1,
  });
  assert(queued.id === "r1" && screen.pendingRunRequest?.id === "r1", "transaction: request should queue while hidden");
  assert(restartCount === 0, "transaction: hidden request must not execute");
  screen.screenVisible = true;
  assert(screen.consumePendingRunRequest() === true, "transaction: visible screen consumes pending request");
  await Promise.resolve();
  assert(restartCount === 1, "transaction: request executes exactly once");
  assert(screen.completedRunRequestId === "r1", "transaction: completed id recorded");
  screen.enqueueRunRequest({ id: "r1", sourceText: contract.minimal_ddn });
  await Promise.resolve();
  assert(restartCount === 1, "transaction: completed duplicate must not re-run");

  const mirrorHash = createFakeEl();
  const mirrorWorld = createFakeEl();
  const mirrorWorldSummary = createFakeEl();
  const mirrorKv = createFakeEl();
  screen.runMirrorHashEl = mirrorHash;
  screen.runMirrorWorldEl = mirrorWorld;
  screen.runMirrorWorldSummaryEl = mirrorWorldSummary;
  screen.runMirrorKvEl = mirrorKv;
  screen.lastRuntimeHash = "blake3:test";
  screen.updateMirrorTab(minimalState);
  assert(mirrorWorldSummary.textContent === "관찰 상태 · 1개", "transaction: mirror observation summary missing");
  assert(mirrorKv.innerHTML.includes("인사") && mirrorKv.innerHTML.includes("hello"), "transaction: mirror observation row missing");
  const runtimeStatus = createFakeEl();
  const execUserStatus = createFakeEl();
  const execTechSummary = createFakeEl();
  const execTechBody = createFakeEl();
  const execTech = createFakeEl();
  screen.runtimeStatusEl = runtimeStatus;
  screen.runExecUserStatusEl = execUserStatus;
  screen.runExecTechSummaryEl = execTechSummary;
  screen.runExecTechBodyEl = execTechBody;
  screen.runExecTechEl = execTech;
  screen.engineMode = "oneshot";
  screen.executionPaused = true;
  screen.runtimeTickCounter = 0;
  screen.lastMainVisualMode = "none";
  screen.lastExecPathHint = "실행 완료";
  screen.lastRuntimeDerived = {
    observation: stateMod.extractObservationChannelsFromState(minimalState),
    outputRows: rows,
    views: { families: [] },
  };
  screen.updateRuntimeHint();
  assert(runtimeStatus.textContent.includes("인사=hello"), "transaction: runtime hint should show output row preview");
  assert(!runtimeStatus.textContent.includes("0마디"), "transaction: static hint should suppress zero tick");
  assert(!runtimeStatus.textContent.includes("다음 마디"), "transaction: static hint should suppress no-next-tick detail");
  assert(!runtimeStatus.textContent.includes("보개: none"), "transaction: runtime hint should suppress none visual mode");
  assert(!runtimeStatus.textContent.includes("theta="), "transaction: static scalar hint should not leak theta");
  assert(execUserStatus.textContent === "실행 완료", "transaction: oneshot pause must not read as paused");
  screen.baseDdn = contract.minimal_ddn;
  screen.lesson = { id: "custom", ddnText: contract.minimal_ddn };
  assert(
    screen.sessionMatchesCurrentLesson({
      lesson: "custom",
      ddn_text: "채비 {\n  theta: 수 <- 0.\n  omega: 수 <- 0.\n}.",
    }) === false,
    "transaction: stale same-id pendulum session must not match new source",
  );
  assert(
    screen.runManagerSourceMatchesCurrent({
      source: {
        lessonId: "custom",
        text: "채비 {\n  theta: 수 <- 0.\n  omega: 수 <- 0.\n}.",
      },
    }) === false,
    "transaction: stale same-id pendulum run manager entry must not match new source",
  );
  assert(
    screen.runManagerRunRestorable({
      id: "run:custom:old-empty",
      source: { lessonId: "custom", text: contract.minimal_ddn },
      graph: { series: [{ id: "empty", points: [] }] },
      hash: { result: "blake3:old" },
    }) === false,
    "transaction: empty zero-point run manager entry must not restore",
  );
  assert(
    screen.sessionMatchesCurrentLesson({
      lesson: "custom",
      ddn_text: contract.minimal_ddn,
      last_state_hash: "blake3:352bd266dae53c6e6a29244011cfa029813d0ab8434b2a2b830a487d882832ba",
      runs: [{ id: "run:custom:old-empty", graph: { series: [{ id: "empty", points: [] }] } }],
    }) === false,
    "transaction: empty initial-hash runtime session must not match current lesson",
  );
  const inspectorScene = screen.buildCurrentInspectorSceneSummary({
    schema: "seamgrim.scene.v0",
    hashes: { result_hash: "old" },
    layers: [
      { id: "run:09_moyang_pendulum_working:old", label: "pendulum", points: 1 },
      { id: "run:custom:old-empty", label: "empty draft", points: 0 },
      { id: "run:custom:now", label: "current", points: 0 },
      { id: "run:custom:graph", label: "current graph", points: 2 },
    ],
  });
  assert(inspectorScene.layers.length === 1, "transaction: inspector scene should filter stale run layers");
  assert(inspectorScene.layers[0].id === "run:custom:graph", "transaction: inspector scene should keep only meaningful current graph run layer");
  assert(inspectorScene.hashes.result_hash === "blake3:test", "transaction: inspector scene result hash should prefer current hash");

  const uiDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModule = await import(pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href);
  const wrapper = await import(pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href);
  const wasmBytes = await fs.readFile(path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm"));
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }
  const preparedSource = typeof wasmModule.wasm_preprocess_source === "function"
    ? String(wasmModule.wasm_preprocess_source(contract.minimal_ddn) ?? contract.minimal_ddn)
    : contract.minimal_ddn;
  const vm = new wasmModule.DdnWasmVm(preparedSource);
  const client = new wrapper.DdnWasmVmClient(vm);
  client.updateLogicWithMode(preparedSource, "strict");
  const initialRows = stateMod.extractObservationOutputRowsFromState(client.getStateParsed());
  assert(initialRows.length === 0, "transaction: actual wasm initial state should not pretend output exists");
  const steppedState = client.stepOneParsed();
  const steppedRows = stateMod.extractObservationOutputRowsFromState(steppedState);
  assert(
    steppedRows.some((row) => String(row.key) === expected.key && String(row.value) === expected.value),
    "transaction: actual wasm oneshot step must produce editor output row",
  );
  if (typeof vm.free === "function") vm.free();

  const report = {
    schema: "ddn.seamgrim.editor_run_transaction.report.v1",
    ok: true,
    transaction_order: "load_lesson_enqueue_set_screen_execute",
    pending_waits_for_visible: true,
    visible_consumes_once: true,
    duplicate_request_blocked: true,
    stale_result_guard_present: true,
    mirror_observation_fallback: true,
    stale_run_manager_source_guard: true,
    empty_initial_session_rejected: true,
    zero_point_runs_rejected: true,
    stale_session_source_guard: true,
    inspector_scene_current_source_only: true,
    static_hint_output_first: true,
    actual_wasm_oneshot_step_output: true,
    pre_run_bogae_fixed_shell: true,
    console_grid_html_fallback: true,
    output_row: { key: String(hit.key), value: String(hit.value), source: String(hit.source ?? "") },
    console_bogae_value_only: true,
  };
  console.log(JSON.stringify(sortJson(report), null, 2));
}

main().catch((err) => {
  console.error(err?.stack ?? String(err));
  process.exit(1);
});
