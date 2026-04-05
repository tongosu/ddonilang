import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function makeControlElement(name) {
  const handlers = new Map();
  return {
    name,
    value: "",
    checked: false,
    max: "0",
    addEventListener(type, handler) {
      handlers.set(type, handler);
    },
    emit(type) {
      const handler = handlers.get(type);
      if (typeof handler === "function") {
        handler({ currentTarget: this });
      }
    },
    classList: { toggle() {} },
  };
}

async function main() {
  const rootDir = process.cwd();
  const runPath = path.resolve(rootDir, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const runMod = await import(pathToFileURL(runPath).href);
  const { RunScreen } = runMod;

  const controls = new Map([
    ["#btn-dock-play", makeControlElement("play")],
    ["#btn-dock-pause", makeControlElement("pause")],
    ["#btn-dock-next", makeControlElement("next")],
    ["#chk-dock-loop", makeControlElement("loop")],
    ["#btn-dock-space-autoscale", makeControlElement("space-auto")],
    ["#btn-dock-graph-autoscale", makeControlElement("graph-auto")],
    ["#btn-dock-pan-left", makeControlElement("pan-left")],
    ["#btn-dock-pan-right", makeControlElement("pan-right")],
    ["#btn-dock-pan-up", makeControlElement("pan-up")],
    ["#btn-dock-pan-down", makeControlElement("pan-down")],
    ["#btn-dock-zoom-in", makeControlElement("zoom-in")],
    ["#btn-dock-zoom-out", makeControlElement("zoom-out")],
  ]);
  const dockSpeed = makeControlElement("speed");
  dockSpeed.value = "1";
  const dockCursor = makeControlElement("cursor");
  const dockTimeText = { textContent: "" };

  const runScreen = new RunScreen({
    root: {
      querySelector(selector) {
        if (selector === "#select-dock-speed") return dockSpeed;
        if (selector === "#range-dock-time-cursor") return dockCursor;
        if (selector === "#text-dock-time") return dockTimeText;
        return controls.get(selector) ?? null;
      },
      dataset: {},
    },
    wasmState: { fpsLimit: 30, dtMax: 0.1 },
  });

  let stepFrameCount = 0;
  runScreen.stepFrame = () => {
    stepFrameCount += 1;
  };
  const cursorHistory = [];
  runScreen.dotbogi = {
    resetAxis() {},
    panByRatio() {},
    zoomByFactor() {},
    setGuides() {},
    getGuides() {
      return { showGrid: true, showAxis: true };
    },
    getCurrentAxis() {
      return null;
    },
    getTimelineLength() {
      return 5;
    },
    setPlaybackCursor(value) {
      cursorHistory.push(Number(value));
    },
    getTimelineSampleAt(index) {
      return {
        values: {
          t: Number(index),
        },
      };
    },
  };
  runScreen.bogae = {
    resetView() {},
    panByRatio() {},
    zoomByFactor() {},
    setGuides() {},
    getGuides() {
      return { showGrid: true, showAxis: true };
    },
    getCurrentRange() {
      return null;
    },
  };
  runScreen.overlay = {
    visible: false,
    show() {
      this.visible = true;
    },
    hide() {
      this.visible = false;
    },
  };
  runScreen.dockTargetSelectEl = makeControlElement("target");
  runScreen.dockTargetSelectEl.value = "graph";
  runScreen.dockGridCheckEl = makeControlElement("grid");
  runScreen.dockAxisCheckEl = makeControlElement("axis");
  runScreen.dockOverlayCheckEl = makeControlElement("overlay");
  runScreen.dockHighlightCheckEl = makeControlElement("highlight");
  runScreen.dockLoopCheckEl = controls.get("#chk-dock-loop");
  runScreen.dockLoopCheckEl.checked = true;
  runScreen.dockSpeedSelectEl = dockSpeed;
  runScreen.dockTimeCursorEl = dockCursor;
  runScreen.dockTimeTextEl = dockTimeText;
  runScreen.dockSpaceRangeEl = { textContent: "" };
  runScreen.dockGraphRangeEl = { textContent: "" };

  runScreen.bindViewDockUi();
  runScreen.runtimeTickCounter = 4;
  runScreen.runtimeTimeValue = 4;
  runScreen.playbackPaused = true;
  runScreen.dockCursorTick = 1;
  runScreen.dockCursorFollowLive = false;

  controls.get("#btn-dock-next")?.emit("click");
  assert(stepFrameCount === 0, "view dock next: must not trigger simulation step");
  assert(runScreen.dockCursorTick === 2, "view dock next: cursor advanced");

  runScreen.playbackPaused = false;
  runScreen.dockCursorTick = 4;
  runScreen.advanceDockCursor(1, { loop: false });
  assert(runScreen.dockCursorTick === 4, "view dock loop off: cursor clamps at end");
  assert(runScreen.playbackPaused === true, "view dock loop off: playback pauses at end");

  runScreen.dockCursorTick = 4;
  runScreen.advanceDockCursor(1, { loop: true });
  assert(runScreen.dockCursorTick === 0, "view dock loop on: cursor wraps to start");

  controls.get("#chk-dock-loop").checked = false;
  controls.get("#chk-dock-loop").emit("change");
  assert(runScreen.playbackLoop === false, "view dock loop toggle: playbackLoop sync");
  runScreen.dockCursorTick = 2;

  controls.get("#btn-dock-play")?.emit("click");
  assert(runScreen.playbackPaused === false, "view dock play: playback resumed");
  assert(runScreen.dockCursorFollowLive === false, "view dock play: playback stays offline");

  controls.get("#btn-dock-pause")?.emit("click");
  assert(runScreen.playbackPaused === true, "view dock pause: playback paused");

  runScreen.syncDockTimeUi();
  assert(String(runScreen.dockTimeCursorEl.value) === "2", "view dock sync: cursor preserves offline position");
  assert(
    String(runScreen.dockTimeTextEl.textContent).includes("틱=2"),
    "view dock sync: time text reflects cursor tick",
  );

  runScreen.loop = {
    started: 0,
    stopped: 0,
    start() {
      this.started += 1;
    },
    stop() {
      this.stopped += 1;
    },
  };
  runScreen.screenVisible = true;
  runScreen.playbackPaused = true;
  runScreen.syncLoopState();
  assert(runScreen.loop.started === 1, "view dock loop: simulation loop does not depend on playback pause");

  assert(cursorHistory.length > 0, "view dock cursor: dotbogi cursor sync called");
  runScreen.clearViewPlaybackTimer();
  console.log("seamgrim view dock time runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
