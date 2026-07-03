#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function makeEvent(extra = {}) {
  return {
    repeat: false,
    code: "",
    defaultPrevented: false,
    preventDefault() {
      this.defaultPrevented = true;
    },
    ...extra,
  };
}

class FakeElement {
  constructor(id, opts = {}) {
    this.id = id;
    this.dataset = opts.dataset || {};
    this.disabled = false;
    this.checked = false;
    this.value = opts.value || "";
    this.max = "";
    this.textContent = "";
    this.listeners = new Map();
  }

  addEventListener(type, handler) {
    const list = this.listeners.get(type) || [];
    list.push(handler);
    this.listeners.set(type, list);
  }

  async dispatch(type, event = {}) {
    const handlers = this.listeners.get(type) || [];
    for (const handler of handlers) {
      await handler({ target: this, ...event });
    }
  }
}

class FakeCanvas extends FakeElement {
  constructor() {
    super("canvas");
    this.width = 0;
    this.height = 0;
    this.style = {};
    this.calls = {
      clearRect: 0,
      fillRect: 0,
      strokeRect: 0,
      drawImage: 0,
      fillText: 0,
      stroke: 0,
    };
    this.ctx = {
      imageSmoothingEnabled: false,
      fillStyle: "",
      strokeStyle: "",
      lineWidth: 0,
      font: "",
      textBaseline: "",
      clearRect: () => {
        this.calls.clearRect += 1;
      },
      fillRect: () => {
        this.calls.fillRect += 1;
      },
      strokeRect: () => {
        this.calls.strokeRect += 1;
      },
      drawImage: () => {
        this.calls.drawImage += 1;
      },
      fillText: () => {
        this.calls.fillText += 1;
      },
      beginPath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      bezierCurveTo: () => {},
      arc: () => {},
      fill: () => {
        this.calls.fillRect += 1;
      },
      stroke: () => {
        this.calls.stroke += 1;
      },
      save: () => {},
      restore: () => {},
    };
  }

  getContext(type) {
    return type === "2d" ? this.ctx : null;
  }
}

async function loadJson(pathname) {
  return JSON.parse(await fs.readFile(pathname, "utf8"));
}

function makeResponse(body, contentType = "json") {
  return {
    ok: true,
    async json() {
      return body;
    },
    async arrayBuffer() {
      if (Buffer.isBuffer(body)) {
        return body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength);
      }
      const bytes = Buffer.from(String(body), "utf8");
      return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
    },
    async text() {
      return contentType === "json" ? JSON.stringify(body) : String(body);
    },
  };
}

function notFound() {
  return {
    ok: false,
    async json() {
      return null;
    },
    async arrayBuffer() {
      return new ArrayBuffer(0);
    },
    async text() {
      return "";
    },
  };
}

async function makeEnv({ root, outDir, live }) {
  const elements = new Map();
  const canvas = new FakeCanvas();
  elements.set("canvas", canvas);
  for (const id of [
    "btn-play",
    "btn-step-back",
    "btn-step-forward",
    "seek",
    "status",
    "ov-grid",
    "ov-bounds",
    "ov-delta",
  ]) {
    elements.set(id, new FakeElement(id));
  }

  const windowListeners = new Map();
  const documentListeners = new Map();
  const inputFetches = [];
  const frameFetches = [];
  const manifestFetches = [];
  const timers = [];
  const input = "http://127.0.0.1:56123/input";

  const addTo = (store, type, handler) => {
    const list = store.get(type) || [];
    list.push(handler);
    store.set(type, list);
  };

  const document = {
    hidden: false,
    body: { dataset: live ? { live: "1" } : {} },
    getElementById(id) {
      return elements.get(id) || null;
    },
    addEventListener(type, handler) {
      addTo(documentListeners, type, handler);
    },
  };

  const window = {
    location: {
      search: live ? `?input=${encodeURIComponent(input)}&live=1` : "?scale=1.25",
    },
    addEventListener(type, handler) {
      addTo(windowListeners, type, handler);
    },
  };

  const manifestPath = path.join(outDir, "manifest.detjson");
  const overlayPath = path.join(outDir, "viewer", "overlay.detjson");
  const fetchImpl = async (url) => {
    const raw = String(url);
    if (raw.startsWith(input)) {
      inputFetches.push(raw);
      return makeResponse("ok", "text");
    }
    const noQuery = raw.split("?")[0];
    if (noQuery === "../manifest.detjson") {
      manifestFetches.push(raw);
      return makeResponse(await loadJson(manifestPath));
    }
    if (noQuery === "overlay.detjson") {
      return makeResponse(await loadJson(overlayPath));
    }
    if (noQuery === "skin.detjson") {
      return notFound();
    }
    if (noQuery.startsWith("../frames/")) {
      const rel = noQuery.replace(/^\.\.\//, "");
      frameFetches.push(rel);
      return makeResponse(await fs.readFile(path.join(outDir, rel)), "bytes");
    }
    return notFound();
  };

  const context = {
    window,
    document,
    console,
    URLSearchParams,
    TextDecoder,
    DataView,
    Uint8Array,
    ArrayBuffer,
    Map,
    Set,
    Number,
    Math,
    JSON,
    Date,
    Image: class {
      set src(_) {
        setTimeout(() => {
          if (typeof this.onload === "function") {
            this.onload();
          }
        }, 0);
      }
    },
    fetch: fetchImpl,
    setInterval(fn, ms) {
      timers.push({ fn, ms });
      return timers.length;
    },
    clearInterval() {},
    setTimeout,
    clearTimeout,
  };

  return {
    context,
    elements,
    canvas,
    windowListeners,
    documentListeners,
    inputFetches,
    frameFetches,
    manifestFetches,
    timers,
    input,
  };
}

async function flush() {
  for (let idx = 0; idx < 8; idx += 1) {
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
}

async function runViewer(outDir, live) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const env = await makeEnv({ root, outDir, live });
  const js = await fs.readFile(path.join(outDir, "viewer", "viewer.js"), "utf8");
  vm.runInNewContext(js, env.context, { filename: "viewer.js" });
  await flush();
  return env;
}

async function runLiveCase(outDir) {
  const env = await runViewer(outDir, true);
  assert(env.elements.get("btn-play").disabled === true, "live mode should disable playback controls");
  assert(env.timers.length === 1 && env.timers[0].ms === 300, "live mode should register refresh timer");
  assert(env.frameFetches.includes("frames/000003.bdl1.detbin"), "live mode should load latest frame");
  assert(env.canvas.calls.clearRect > 0, "live mode should render canvas");

  const keydown = env.windowListeners.get("keydown") || [];
  const keyup = env.windowListeners.get("keyup") || [];
  const blur = env.windowListeners.get("blur") || [];
  const visibility = env.documentListeners.get("visibilitychange") || [];
  assert(keydown.length === 1, "keydown listener missing");
  assert(keyup.length === 1, "keyup listener missing");
  assert(blur.length === 1, "blur listener missing");
  assert(visibility.length === 1, "visibilitychange listener missing");

  await keydown[0](makeEvent({ code: "ArrowLeft" }));
  await keydown[0](makeEvent({ code: "ArrowLeft", repeat: true }));
  await keyup[0](makeEvent({ code: "ArrowLeft" }));
  await blur[0]();
  env.context.document.hidden = true;
  await visibility[0]();
  await flush();

  const down = env.inputFetches.filter((url) => url.includes("code=ArrowLeft") && url.includes("kind=down"));
  const up = env.inputFetches.filter((url) => url.includes("code=ArrowLeft") && url.includes("kind=up"));
  const clear = env.inputFetches.filter((url) => url.includes("kind=clear"));
  assert(down.length === 1, `expected one down fetch, got ${down.length}`);
  assert(up.length === 1, `expected one up fetch, got ${up.length}`);
  assert(clear.length >= 2, `expected blur and visibility clear fetches, got ${clear.length}`);
}

async function runPlaybackCase(outDir) {
  const env = await runViewer(outDir, false);
  assert(env.elements.get("btn-play").disabled === false, "playback mode should enable play");
  assert(env.elements.get("btn-step-forward").disabled === false, "playback mode should enable step forward");
  assert(env.timers.length === 0, "playback mode should not start live refresh timer");
  assert(env.frameFetches.includes("frames/000000.bdl1.detbin"), "playback should load first frame");
  assert(env.canvas.calls.clearRect > 0, "playback should render canvas");

  const before = env.frameFetches.length;
  await env.elements.get("btn-step-forward").dispatch("click");
  await flush();
  assert(env.frameFetches.length > before, "step forward should load a frame");

  await env.elements.get("btn-step-back").dispatch("click");
  await flush();
  env.elements.get("seek").value = "2";
  await env.elements.get("seek").dispatch("input");
  await flush();
  assert(env.frameFetches.includes("frames/000002.bdl1.detbin"), "seek should load selected frame");

  env.elements.get("ov-grid").checked = true;
  await env.elements.get("ov-grid").dispatch("change");
  await flush();
  assert(env.canvas.calls.stroke > 0, "grid overlay should draw stroke calls");
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const outDir = process.argv[2]
    ? path.resolve(process.argv[2])
    : path.join(root, "build", "std_grid_game_bogae_live_web_assets_v1");
  await runLiveCase(outDir);
  await runPlaybackCase(outDir);
  console.log("std_grid_game_bogae_viewer_js_dom: ok");
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exit(1);
});
