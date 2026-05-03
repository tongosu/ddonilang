#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function createFakeContext() {
  const calls = [];
  return {
    calls,
    fillStyle: "",
    strokeStyle: "",
    lineWidth: 0,
    font: "",
    textBaseline: "",
    fillRect() {},
    beginPath() {},
    moveTo() {},
    lineTo() {},
    stroke() {},
    fillText(text) {
      calls.push({ op: "fillText", text: String(text ?? "") });
    },
    measureText(text) {
      return { width: String(text ?? "").length * 8 };
    },
  };
}

async function main() {
  const windowListeners = new Map();
  globalThis.window = {
    addEventListener(type, listener) {
      windowListeners.set(type, listener);
    },
  };

  const canvasListeners = new Map();
  const ctx = createFakeContext();
  const canvas = {
    clientWidth: 320,
    clientHeight: 180,
    width: 320,
    height: 180,
    addEventListener(type, listener) {
      canvasListeners.set(type, listener);
    },
    getContext() {
      return ctx;
    },
  };

  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const bogaeUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "components", "bogae.js"),
  ).href;
  const { Bogae } = await import(bogaeUrl);

  const bogae = new Bogae({ canvas });
  bogae.lastAutoRange = { x_min: 0, x_max: 10, y_min: 0, y_max: 10 };

  let renderCalls = 0;
  const originalRender = bogae.render.bind(bogae);
  bogae.render = (...args) => {
    renderCalls += 1;
    return originalRender(...args);
  };

  bogae.renderConsoleGrid(["23", "15", "8"]);
  assert(bogae.renderMode === "console-grid", "console-grid render: mode pinned");
  assert(bogae.lastSpace2d === null, "console-grid render: stale space2d cleared");
  assert(bogae.lastAutoRange === null, "console-grid render: stale auto range cleared");

  canvasListeners.get("wheel")?.({
    deltaY: -1,
    preventDefault() {},
  });
  canvasListeners.get("mousedown")?.({
    clientX: 10,
    clientY: 10,
  });
  windowListeners.get("mousemove")?.({
    clientX: 20,
    clientY: 20,
  });

  assert(renderCalls === 0, "console-grid interaction: space2d rerender must not fire");
  assert(bogae.drag.active === false, "console-grid interaction: drag should stay inactive");

  const zeroCtx = createFakeContext();
  const zeroCanvas = {
    clientWidth: 0,
    clientHeight: 0,
    width: 0,
    height: 0,
    parentElement: {
      clientWidth: 240,
      clientHeight: 120,
    },
    addEventListener() {},
    getContext() {
      return zeroCtx;
    },
  };
  const zeroBogae = new Bogae({ canvas: zeroCanvas });
  zeroBogae.renderConsoleGrid(["인사=hello"]);
  assert(zeroCanvas.width === 240 && zeroCanvas.height === 120, "console-grid zero client: fallback dimensions applied");
  assert(
    zeroCtx.calls.some((call) => String(call.text ?? "").includes("h")),
    "console-grid zero client: text rendered with fallback dimensions",
  );

  console.log("seamgrim bogae console-grid interaction ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
