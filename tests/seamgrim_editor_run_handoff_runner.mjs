#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function sortJson(value) {
  if (Array.isArray(value)) return value.map((item) => sortJson(item));
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sortJson(value[key])]));
}

async function main() {
  const root = process.cwd();
  const packDir = path.join(root, "pack", "seamgrim_editor_run_handoff_v1");
  const contract = JSON.parse(await fs.readFile(path.join(packDir, "contract.detjson"), "utf8"));
  const runtimeStateUrl = pathToFileURL(path.join(root, "solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js")).href;
  const runUrl = pathToFileURL(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js")).href;
  const runtimeState = await import(runtimeStateUrl);
  const runScreen = await import(runUrl);

  const minimalState = {
    schema: "seamgrim.state.v0",
    channels: [{ key: "인사", dtype: "str", role: "state" }],
    row: ["hello"],
    resources: { json: {}, fixed64: {}, handle: {}, value: {}, value_json: {}, component: {} },
    observation_manifest: {
      nodes: [{ name: "인사", dtype: "str", role: "state" }],
    },
  };
  const rows = runtimeState.extractObservationOutputRowsFromState(minimalState);
  const expected = contract.expected_output_row ?? {};
  const hit = rows.find((row) => String(row.key) === expected.key && String(row.value) === expected.value);
  assert(hit, "editor handoff: minimal show output row missing");

  const postTab = runScreen.resolveRunPostExecuteTab({ outputRows: rows });
  assert(postTab === contract.expected_post_execute_tab, "editor handoff: post execute tab should prefer output");

  const layout = runScreen.resolveRunLayoutProfile([]);
  assert(layout.mode === contract.expected_initial_layout_mode, "editor handoff: empty initial layout should keep fixed split stage");

  const runText = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js"), "utf8");
  const appText = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/app.js"), "utf8");
  const stylesText = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/styles.css"), "utf8");

  assert(runText.includes("this.syncInitialBogaeShellVisibility(false);"), "editor handoff: idle bogae shell initialization missing");
  assert(runText.includes("pendingRunRequest") && runText.includes("consumePendingRunRequest()"), "editor handoff: transaction pending request missing");
  assert(stylesText.includes(".run-layout.run-layout--dock-only .bogae-area") && !stylesText.includes(".run-layout.run-layout--dock-only .bogae-area {\n  display: none;"), "editor handoff: bogae area should remain a fixed shell");

  const loadLessonPos = appText.indexOf("runScreen.loadLesson(lesson");
  const setScreenPos = appText.indexOf('setScreen("run");', loadLessonPos);
  const enqueuePos = appText.indexOf("runScreen.enqueueRunRequest", loadLessonPos);
  assert(loadLessonPos >= 0 && enqueuePos > loadLessonPos && setScreenPos > enqueuePos, "editor handoff: app handoff order invalid");
  const editorRunPos = appText.indexOf('launchKind: "editor_run"');
  const editorOpenRunPos = appText.lastIndexOf("openRunWithLessonWithSource(baseLesson", editorRunPos);
  const editorHydratePos = appText.indexOf("void lessonCanonHydrator.hydrateLessonCanon(baseLesson)", editorRunPos);
  assert(editorOpenRunPos >= 0 && editorHydratePos > editorOpenRunPos, "editor handoff: hydration must not block opening run screen");
  assert(appText.includes("const model = buildStudioEditorReadinessModel({"), "editor handoff: run click must rebuild readiness from current text");
  assert(appText.includes("runRequest: {") && appText.includes("sourceText: effectiveDdnText"), "editor handoff: editor run should pass source snapshot");

  const report = {
    schema: "ddn.seamgrim.editor_run_handoff.report.v1",
    ok: true,
    output_row: {
      key: String(hit.key),
      value: String(hit.value),
      source: String(hit.source ?? ""),
    },
    post_execute_tab: postTab,
    initial_layout_mode: layout.mode,
    app_handoff_order: "load_lesson_enqueue_set_screen_execute",
    visual_idle_policy: "fixed_bogae_waiting_stage",
  };
  console.log(JSON.stringify(sortJson(report), null, 2));
}

main().catch((err) => {
  console.error(err?.stack ?? String(err));
  process.exit(1);
});
