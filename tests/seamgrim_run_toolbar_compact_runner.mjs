import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const rootDir = process.cwd();
  const runPath = path.resolve(rootDir, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const htmlPath = path.resolve(rootDir, "solutions/seamgrim_ui_mvp/ui/index.html");
  const runMod = await import(pathToFileURL(runPath).href);
  const {
    applyLegacyAutofixToDdn,
    hasReachedRuntimeMaxMadi,
    readConfiguredMadiFromDdnText,
    resolveBogaeToolbarCompact,
    resolveRunLoopFps,
    resolveRunMainControlLabels,
  } = runMod;
  const bogaeMod = await import(pathToFileURL(path.join(rootDir, "solutions/seamgrim_ui_mvp/ui/components/bogae.js")).href);

  assert(typeof applyLegacyAutofixToDdn === "function", "toolbar compact runner: applyLegacyAutofixToDdn export missing");
  assert(typeof hasReachedRuntimeMaxMadi === "function", "toolbar compact runner: hasReachedRuntimeMaxMadi export missing");
  assert(typeof readConfiguredMadiFromDdnText === "function", "toolbar compact runner: readConfiguredMadiFromDdnText export missing");
  assert(typeof resolveBogaeToolbarCompact === "function", "toolbar compact runner: resolveBogaeToolbarCompact export missing");
  assert(typeof resolveRunLoopFps === "function", "toolbar compact runner: resolveRunLoopFps export missing");
  assert(typeof resolveRunMainControlLabels === "function", "toolbar compact runner: resolveRunMainControlLabels export missing");
  assert(
    readConfiguredMadiFromDdnText("설정 {\\n  제목: 예제.\\n  마디수: 40.\\n}.\\n(매마디)마다 { }.")
      === 40,
    "toolbar compact runner: Korean configured madi parse mismatch",
  );
  assert(
    readConfiguredMadiFromDdnText("설정 { max_madi: 12. }.") === 12,
    "toolbar compact runner: max_madi configured parse mismatch",
  );
  assert(
    readConfiguredMadiFromDdnText("채비 {\n  마디수: 수 <- (40.0) 매김 { 범위: 4..120. 간격: 1. }.\n}.\n(매마디)마다 { }.") === 40,
    "toolbar compact runner: resource 마디수 parse mismatch",
  );
  assert(
    readConfiguredMadiFromDdnText("채비 {\n  마디수: 수 <- (40) 매김 { 범위: 4..120. 간격: 1. }.\n}.\n(매마디)마다 { }.") === 40,
    "toolbar compact runner: integer resource 마디수 parse mismatch",
  );
  assert(
    readConfiguredMadiFromDdnText("채비 {\n  계수: 수 <- 1.\n}.\n(매마디)마다 { }.") === 0,
    "toolbar compact runner: missing 마디수 should stay unbounded",
  );
  const legacyMadiSource = `설정 {
  title: rep-cs-linear-search-timeline.
  desc: representative cs hybrid lesson.
}.

채비 {
  최대마디: 수 <- (40.0) 매김 { 범위: 4..120. 간격: 1. }.
}.

(매마디)마다 {
  { t < 최대마디 }인것 일때 {
    t <- t + 1.
  }.
}.`;
  const fixedMadiSource = applyLegacyAutofixToDdn(legacyMadiSource).text;
  assert(!fixedMadiSource.includes("title:") && !fixedMadiSource.includes("desc:"), "toolbar compact runner: legacy title/desc fixed");
  assert(!fixedMadiSource.includes("최대마디"), "toolbar compact runner: legacy 최대마디 fixed");
  assert(/설정\s*\{[\s\S]*?마디수\s*:\s*40\s*\./u.test(fixedMadiSource), "toolbar compact runner: legacy source gets 설정 마디수");
  assert(readConfiguredMadiFromDdnText(fixedMadiSource) === 40, "toolbar compact runner: fixed legacy source configured 마디수 mismatch");
  const exactLegacySource = `설정 {
  title: rep-cs-linear-search-timeline.
  desc: representative cs hybrid lesson.
}.

채비 {
  데이터길이: 수 <- (12) 매김 { 범위: 4..40. 간격: 1. }.
  목표인덱스: 수 <- (7) 매김 { 범위: 0..39. 간격: 1. }.
  최대마디: 수 <- (40) 매김 { 범위: 4..120. 간격: 1. }.
}.

(매마디)마다 {
  { t < 최대마디 }인것 일때 {
    t <- t + 1.
  }.
}.

보개로 그려.`;
  const exactFixed = applyLegacyAutofixToDdn(exactLegacySource).text;
  assert(
    /설정\s*\{[\s\S]*?마디수\s*:\s*40\s*\./u.test(exactFixed)
      && readConfiguredMadiFromDdnText(exactFixed) === 40,
    "toolbar compact runner: exact legacy representative source must expose 마디수 40",
  );
  const csLessonPath = path.join(
    rootDir,
    "solutions",
    "seamgrim_ui_mvp",
    "lessons",
    "rep_cs_linear_search_timeline_v1",
    "lesson.ddn",
  );
  const csLessonText = await fs.readFile(csLessonPath, "utf8");
  assert(
    /설정\s*\{[\s\S]*?마디수\s*:\s*40\s*\./u.test(csLessonText),
    "toolbar compact runner: representative lesson must expose 설정 마디수",
  );
  assert(
    readConfiguredMadiFromDdnText(csLessonText) === 40,
    "toolbar compact runner: representative lesson configured 마디수 mismatch",
  );

  assert(
    resolveBogaeToolbarCompact({ toolbarWidth: 760, thresholdPx: 860 }) === true,
    "toolbar compact runner: narrow width should enable compact",
  );
  assert(
    resolveBogaeToolbarCompact({ toolbarWidth: 960, thresholdPx: 860 }) === false,
    "toolbar compact runner: wide width should disable compact",
  );

  const labelsWide = resolveRunMainControlLabels({ isPaused: false, compact: false });
  const labelsCompact = resolveRunMainControlLabels({ isPaused: false, compact: true });
  const labelsPaused = resolveRunMainControlLabels({ isPaused: true, compact: true });
  assert(labelsWide.execute === "▶ 작업실에서 실행", "toolbar compact runner: wide execute label mismatch");
  assert(labelsCompact.execute === "▶ 실행", "toolbar compact runner: compact execute label mismatch");
  assert(labelsCompact.pause === "⏸ 일시정지", "toolbar compact runner: compact pause label mismatch");
  assert(labelsPaused.execute === "▶ 재개", "toolbar compact runner: resume label mismatch");
  assert(
    resolveRunLoopFps({ fpsLimit: 30, playbackSpeed: 1, engineMode: "live", runtimeMaxMadi: 40 }) === 6,
    "toolbar compact runner: finite live lesson should use readable default fps",
  );
  assert(
    resolveRunLoopFps({ fpsLimit: 30, playbackSpeed: 2, engineMode: "live", runtimeMaxMadi: 40 }) === 12,
    "toolbar compact runner: finite live lesson should still honor speed control",
  );
  assert(
    resolveRunLoopFps({ fpsLimit: 30, playbackSpeed: 1, engineMode: "live", runtimeMaxMadi: 0 }) === 30,
    "toolbar compact runner: unbounded live runtime should keep normal fps",
  );
  assert(hasReachedRuntimeMaxMadi(40, 40) === true, "toolbar compact runner: max madi reached at boundary");
  assert(hasReachedRuntimeMaxMadi(39, 40) === false, "toolbar compact runner: max madi not reached before boundary");
  assert(hasReachedRuntimeMaxMadi(4001, 4000) === true, "toolbar compact runner: max madi reached after boundary");
  assert(hasReachedRuntimeMaxMadi(4001, 0) === false, "toolbar compact runner: zero max madi stays unbounded");
  const bogae = new bogaeMod.Bogae({});
  bogae.render({
    schema: "seamgrim.space2d.v0",
    camera: { x_min: -1, x_max: 1, y_min: -1, y_max: 1 },
    points: [{ x: 0, y: 0 }],
    shapes: [{ kind: "circle", x: 0, y: 0, r: 0.1 }],
  });
  assert(bogae.view.autoFit === true, "toolbar compact runner: bogae initial render must remain auto-fit");
  bogae.zoomByFactor(0.9);
  assert(bogae.view.autoFit === false, "toolbar compact runner: bogae zoom must switch to manual range");
  bogae.resetView();
  assert(bogae.view.autoFit === true, "toolbar compact runner: bogae autoscale must return to auto-fit");

  const html = await fs.readFile(htmlPath, "utf8");
  assert(html.includes('id="btn-step"'), "toolbar compact runner: step button missing");
  assert(html.includes("▷ 한마디씩"), "toolbar compact runner: step label token missing");
  assert(html.includes('aria-label="한마디씩"'), "toolbar compact runner: step aria-label missing");
  assert(html.includes('id="run-madi-status"'), "toolbar compact runner: madi status missing");
  assert(html.includes("0/-마디"), "toolbar compact runner: madi status initial text missing");
  assert(html.includes('class="run-control-bar"'), "toolbar compact runner: common run control bar missing");
  assert(!html.includes("bogae-toolbar"), "toolbar compact runner: legacy bogae toolbar should be removed");
  assert(!html.includes('id="run-exec-user-status"'), "toolbar compact runner: trailing exec status should stay out of toolbar");
  assert(!html.includes('id="bogae-status-text"'), "toolbar compact runner: trailing runtime hint should stay out of toolbar");

  console.log("seamgrim run toolbar compact runner ok");
}

main().catch((error) => {
  console.error(error?.stack || String(error));
  process.exitCode = 1;
});
