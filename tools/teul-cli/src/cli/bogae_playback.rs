use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::cli::bogae::OverlayConfig;

#[derive(Debug, Clone)]
pub struct PlaybackFrameMeta {
    pub madi: u64,
    pub state_hash: String,
    pub hash: String,
    pub cmd_count: u32,
    pub file: String,
}

pub fn write_manifest(
    out_dir: &Path,
    start_madi: u64,
    end_madi: u64,
    frames: &[PlaybackFrameMeta],
    codec: &str,
) -> Result<PathBuf, String> {
    let manifest_path = out_dir.join("manifest.detjson");
    let text = build_manifest_text(start_madi, end_madi, frames, codec);
    fs::write(&manifest_path, text).map_err(|e| e.to_string())?;
    Ok(manifest_path)
}

pub fn write_viewer_assets(
    out_dir: &Path,
    skin_source: Option<&Path>,
    overlay: OverlayConfig,
) -> Result<PathBuf, String> {
    let viewer_dir = out_dir.join("viewer");
    fs::create_dir_all(&viewer_dir).map_err(|e| e.to_string())?;

    let index_path = viewer_dir.join("index.html");
    let live_path = viewer_dir.join("live.html");
    let js_path = viewer_dir.join("viewer.js");
    let overlay_path = viewer_dir.join("overlay.detjson");
    fs::write(&index_path, build_index_html(false)).map_err(|e| e.to_string())?;
    fs::write(&live_path, build_index_html(true)).map_err(|e| e.to_string())?;
    fs::write(&js_path, build_viewer_js()).map_err(|e| e.to_string())?;
    fs::write(&overlay_path, overlay.to_detjson()).map_err(|e| e.to_string())?;

    if let Some(skin_path) = skin_source {
        let target = viewer_dir.join("skin.detjson");
        if fs::copy(skin_path, &target).is_ok() {
            copy_skin_assets(&viewer_dir, skin_path);
        }
    }

    Ok(index_path)
}

fn build_manifest_text(
    start_madi: u64,
    end_madi: u64,
    frames: &[PlaybackFrameMeta],
    codec: &str,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"kind\": \"bogae_web_playback_v1\",\n");
    out.push_str(&format!("  \"codec\": \"{}\",\n", codec));
    out.push_str(&format!("  \"start_madi\": {},\n", start_madi));
    out.push_str(&format!("  \"end_madi\": {},\n", end_madi));
    out.push_str("  \"frames\": [\n");
    for (idx, frame) in frames.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"madi\": {},\n", frame.madi));
        out.push_str(&format!(
            "      \"state_hash\": \"{}\",\n",
            escape_json_string(&frame.state_hash)
        ));
        out.push_str(&format!(
            "      \"bogae_hash\": \"{}\",\n",
            escape_json_string(&frame.hash)
        ));
        out.push_str(&format!("      \"cmd_count\": {},\n", frame.cmd_count));
        out.push_str(&format!(
            "      \"file\": \"{}\"\n",
            escape_json_string(&frame.file)
        ));
        out.push_str("    }");
        if idx + 1 < frames.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn escape_json_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

fn build_index_html(live: bool) -> String {
    let title = if live {
        "Bogae Live Viewer"
    } else {
        "Bogae Playback Viewer"
    };
    let body_attr = if live { " data-live=\"1\"" } else { "" };
    let mut html = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>__TITLE__</title>
  <style>
    html, body {
      margin: 0;
      padding: 0;
      background: #111;
      color: #eaeaea;
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace;
      height: 100%;
    }
    #root {
      display: flex;
      flex-direction: column;
      height: 100%;
    }
    #toolbar {
      display: flex;
      gap: 8px;
      align-items: center;
      padding: 10px 14px;
      background: #1b1b1b;
      border-bottom: 1px solid #333;
    }
    #toolbar button {
      background: #2d2d2d;
      color: #eaeaea;
      border: 1px solid #444;
      padding: 6px 10px;
      cursor: pointer;
    }
    #toolbar button:disabled {
      opacity: 0.5;
      cursor: default;
    }
    #toolbar input[type="range"] {
      flex: 1;
    }
    #overlay {
      display: flex;
      gap: 10px;
      align-items: center;
      font-size: 12px;
      color: #cfcfcf;
    }
    #overlay label {
      display: flex;
      gap: 4px;
      align-items: center;
      cursor: pointer;
    }
    #status {
      min-width: 160px;
      text-align: right;
      font-size: 12px;
      color: #aaa;
    }
    #stage {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 16px;
    }
    canvas {
      background: #000;
      image-rendering: pixelated;
      box-shadow: 0 16px 40px rgba(0,0,0,0.45);
    }
  </style>
</head>
<body__BODY_ATTR__>
  <div id="root">
    <div id="toolbar">
      <button id="btn-play">Play</button>
      <button id="btn-step-back">&lt;</button>
      <button id="btn-step-forward">&gt;</button>
      <input id="seek" type="range" min="0" max="0" value="0" step="1" />
      <div id="overlay">
        <label><input type="checkbox" id="ov-grid" />Grid</label>
        <label><input type="checkbox" id="ov-bounds" />Bounds</label>
        <label><input type="checkbox" id="ov-delta" />Delta</label>
      </div>
      <div id="status">madi=0</div>
    </div>
    <div id="stage">
      <canvas id="canvas"></canvas>
    </div>
  </div>
  <script src="viewer.js"></script>
</body>
</html>
"#
    .to_string();
    html = html.replace("__TITLE__", title);
    html = html.replace("__BODY_ATTR__", body_attr);
    html
}

fn build_viewer_js() -> String {
    r#"(function () {
  const canvas = document.getElementById("canvas");
  const ctx = canvas.getContext("2d");
  ctx.imageSmoothingEnabled = false;

  const params = new URLSearchParams(window.location.search);
  const scaleRaw = params.get("scale") || "1.25";
  const scaleValue = Number(scaleRaw);
  const viewScale = Number.isFinite(scaleValue) && scaleValue > 0 ? scaleValue : 1.25;

  const state = {
    manifest: null,
    frames: [],
    current: 0,
    playing: false,
    timer: null,
    cache: new Map(),
    frameCache: new Map(),
    skin: null,
    overlay: { grid: false, bounds: false, delta: false },
    live: false,
    liveTimer: null,
    viewScale,
  };

  const btnPlay = document.getElementById("btn-play");
  const btnStepBack = document.getElementById("btn-step-back");
  const btnStepForward = document.getElementById("btn-step-forward");
  const seek = document.getElementById("seek");
  const status = document.getElementById("status");
  const ovGrid = document.getElementById("ov-grid");
  const ovBounds = document.getElementById("ov-bounds");
  const ovDelta = document.getElementById("ov-delta");
  const isLive =
    document.body.dataset.live === "1" ||
    params.get("live") === "1";
  state.live = isLive;
  const inputEndpoint = params.get("input");
  const inputKeys = new Set([
    "ArrowLeft",
    "ArrowRight",
    "ArrowDown",
    "ArrowUp",
    "Space",
    "Enter",
    "Escape",
    "KeyZ",
    "KeyX",
  ]);

  function rgbaToCss(c) {
    return `rgba(${c.r},${c.g},${c.b},${c.a / 255})`;
  }

  function snapValue(value) {
    if (!Number.isFinite(value)) {
      return 0;
    }
    return value >= 0 ? Math.floor(value + 0.5) : Math.ceil(value - 0.5);
  }

  function numeric(value) {
    return Number.isFinite(value) ? value : 0;
  }

  function applyAa(cmd, value) {
    const v = numeric(value);
    return cmd && cmd.aa ? v : snapValue(v);
  }

  function readU8(view, offset) {
    return view.getUint8(offset);
  }

  function readU16(view, offset) {
    return view.getUint16(offset, true);
  }

  function readU32(view, offset) {
    return view.getUint32(offset, true);
  }

  function readI32(view, offset) {
    return view.getInt32(offset, true);
  }

  function readString(view, offset) {
    const len = readU32(view, offset);
    offset += 4;
    const bytes = new Uint8Array(view.buffer, offset, len);
    offset += len;
    return { value: new TextDecoder("utf-8").decode(bytes), offset };
  }

  function parseBdl1(buffer) {
    const view = new DataView(buffer);
    let offset = 0;
    const magic = String.fromCharCode(
      readU8(view, 0),
      readU8(view, 1),
      readU8(view, 2),
      readU8(view, 3)
    );
    offset += 4;
    if (magic !== "BDL1") {
      throw new Error("Invalid BDL1");
    }
    const version = readU32(view, offset);
    offset += 4;
    if (version !== 1) {
      throw new Error("Unsupported BDL1 version");
    }
    const width = readU32(view, offset);
    offset += 4;
    const height = readU32(view, offset);
    offset += 4;
    const cmdCount = readU32(view, offset);
    offset += 4;

    const cmds = [];
    for (let i = 0; i < cmdCount; i += 1) {
      const tag = readU8(view, offset);
      offset += 1;
      switch (tag) {
        case 0x01: {
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "Clear", color, aa: false });
          break;
        }
        case 0x02: {
          const x = readI32(view, offset + 0);
          const y = readI32(view, offset + 4);
          const w = readI32(view, offset + 8);
          const h = readI32(view, offset + 12);
          offset += 16;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "RectFill", x, y, w, h, color, aa: false });
          break;
        }
        case 0x03: {
          const x = readI32(view, offset + 0);
          const y = readI32(view, offset + 4);
          const w = readI32(view, offset + 8);
          const h = readI32(view, offset + 12);
          const thickness = readI32(view, offset + 16);
          offset += 20;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "RectStroke", x, y, w, h, thickness, color, aa: false });
          break;
        }
        case 0x04: {
          const x1 = readI32(view, offset + 0);
          const y1 = readI32(view, offset + 4);
          const x2 = readI32(view, offset + 8);
          const y2 = readI32(view, offset + 12);
          const thickness = readI32(view, offset + 16);
          offset += 20;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "Line", x1, y1, x2, y2, thickness, color, aa: false });
          break;
        }
        case 0x05: {
          const x = readI32(view, offset + 0);
          const y = readI32(view, offset + 4);
          const size = readI32(view, offset + 8);
          offset += 12;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          const textRes = readString(view, offset);
          offset = textRes.offset;
          cmds.push({ kind: "Text", x, y, size, color, text: textRes.value, aa: false });
          break;
        }
        case 0x06: {
          const x = readI32(view, offset + 0);
          const y = readI32(view, offset + 4);
          const w = readI32(view, offset + 8);
          const h = readI32(view, offset + 12);
          offset += 16;
          const tint = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          const uriRes = readString(view, offset);
          offset = uriRes.offset;
          const hashKind = readU8(view, offset);
          offset += 1;
          if (hashKind === 1) {
            offset += 32;
          }
          cmds.push({ kind: "Sprite", x, y, w, h, tint, uri: uriRes.value, aa: false });
          break;
        }
        default:
          throw new Error(`Unknown tag ${tag}`);
      }
    }
    return { width, height, cmds };
  }

  function parseBdl2(buffer) {
    const view = new DataView(buffer);
    let offset = 0;
    const magic = String.fromCharCode(
      readU8(view, 0),
      readU8(view, 1),
      readU8(view, 2),
      readU8(view, 3)
    );
    offset += 4;
    if (magic !== "BDL2") {
      throw new Error("Invalid BDL2");
    }
    const version = readU32(view, offset);
    offset += 4;
    if (version !== 2) {
      throw new Error("Unsupported BDL2 version");
    }
    const width = readU32(view, offset);
    offset += 4;
    const height = readU32(view, offset);
    offset += 4;
    const fixedQ = readU8(view, offset);
    offset += 1;
    const flags = readU8(view, offset);
    offset += 1;
    offset += 2; // reserved
    if (fixedQ !== 8 || flags !== 0) {
      throw new Error("Unsupported BDL2 header flags");
    }
    const cmdCount = readU32(view, offset);
    offset += 4;
    const toFloat = (raw) => raw / 256.0;

    const cmds = [];
    for (let i = 0; i < cmdCount; i += 1) {
      const tag = readU8(view, offset);
      offset += 1;
      const cmdFlags = readU8(view, offset);
      offset += 1;
      if ((cmdFlags & 0xfe) !== 0) {
        throw new Error("Unsupported BDL2 cmd flags");
      }
      const aa = (cmdFlags & 0x1) !== 0;
      switch (tag) {
        case 0x01: {
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "Clear", color, aa });
          break;
        }
        case 0x02: {
          const x = toFloat(readI32(view, offset + 0));
          const y = toFloat(readI32(view, offset + 4));
          const w = toFloat(readI32(view, offset + 8));
          const h = toFloat(readI32(view, offset + 12));
          offset += 16;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "RectFill", x, y, w, h, color, aa });
          break;
        }
        case 0x03: {
          const x = toFloat(readI32(view, offset + 0));
          const y = toFloat(readI32(view, offset + 4));
          const w = toFloat(readI32(view, offset + 8));
          const h = toFloat(readI32(view, offset + 12));
          const thickness = toFloat(readI32(view, offset + 16));
          offset += 20;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "RectStroke", x, y, w, h, thickness, color, aa });
          break;
        }
        case 0x04: {
          const x1 = toFloat(readI32(view, offset + 0));
          const y1 = toFloat(readI32(view, offset + 4));
          const x2 = toFloat(readI32(view, offset + 8));
          const y2 = toFloat(readI32(view, offset + 12));
          const thickness = toFloat(readI32(view, offset + 16));
          offset += 20;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "Line", x1, y1, x2, y2, thickness, color, aa });
          break;
        }
        case 0x05: {
          const x = toFloat(readI32(view, offset + 0));
          const y = toFloat(readI32(view, offset + 4));
          const size = toFloat(readI32(view, offset + 8));
          offset += 12;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          const textRes = readString(view, offset);
          offset = textRes.offset;
          cmds.push({ kind: "Text", x, y, size, color, text: textRes.value, aa });
          break;
        }
        case 0x06: {
          const x = toFloat(readI32(view, offset + 0));
          const y = toFloat(readI32(view, offset + 4));
          const w = toFloat(readI32(view, offset + 8));
          const h = toFloat(readI32(view, offset + 12));
          offset += 16;
          const tint = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          const uriRes = readString(view, offset);
          offset = uriRes.offset;
          const hashKind = readU8(view, offset);
          offset += 1;
          if (hashKind === 1) {
            offset += 32;
          }
          cmds.push({ kind: "Sprite", x, y, w, h, tint, uri: uriRes.value, aa });
          break;
        }
        case 0x07: {
          const cx = toFloat(readI32(view, offset + 0));
          const cy = toFloat(readI32(view, offset + 4));
          const r = toFloat(readI32(view, offset + 8));
          offset += 12;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "CircleFill", cx, cy, r, color, aa });
          break;
        }
        case 0x08: {
          const cx = toFloat(readI32(view, offset + 0));
          const cy = toFloat(readI32(view, offset + 4));
          const r = toFloat(readI32(view, offset + 8));
          const thickness = toFloat(readI32(view, offset + 12));
          offset += 16;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({ kind: "CircleStroke", cx, cy, r, thickness, color, aa });
          break;
        }
        case 0x09: {
          const cx = toFloat(readI32(view, offset + 0));
          const cy = toFloat(readI32(view, offset + 4));
          const r = toFloat(readI32(view, offset + 8));
          const startTurn = toFloat(readI32(view, offset + 12));
          const sweepTurn = toFloat(readI32(view, offset + 16));
          const thickness = toFloat(readI32(view, offset + 20));
          offset += 24;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({
            kind: "ArcStroke",
            cx,
            cy,
            r,
            start_turn: startTurn,
            sweep_turn: sweepTurn,
            thickness,
            color,
            aa,
          });
          break;
        }
        case 0x0a: {
          const p0x = toFloat(readI32(view, offset + 0));
          const p0y = toFloat(readI32(view, offset + 4));
          const p1x = toFloat(readI32(view, offset + 8));
          const p1y = toFloat(readI32(view, offset + 12));
          const p2x = toFloat(readI32(view, offset + 16));
          const p2y = toFloat(readI32(view, offset + 20));
          const p3x = toFloat(readI32(view, offset + 24));
          const p3y = toFloat(readI32(view, offset + 28));
          const thickness = toFloat(readI32(view, offset + 32));
          offset += 36;
          const color = {
            r: readU8(view, offset + 0),
            g: readU8(view, offset + 1),
            b: readU8(view, offset + 2),
            a: readU8(view, offset + 3),
          };
          offset += 4;
          cmds.push({
            kind: "CurveCubicStroke",
            p0x,
            p0y,
            p1x,
            p1y,
            p2x,
            p2y,
            p3x,
            p3y,
            thickness,
            color,
            aa,
          });
          break;
        }
        default:
          throw new Error(`Unknown tag ${tag}`);
      }
    }
    return { width, height, cmds };
  }

  function loadImage(src) {
    if (state.cache.has(src)) {
      return state.cache.get(src);
    }
    const promise = new Promise((resolve) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = () => resolve(null);
      img.src = src;
    });
    state.cache.set(src, promise);
    return promise;
  }

  function resolveSymbol(sym, madi) {
    if (!state.skin || !state.skin.has(sym)) {
      return null;
    }
    const entry = state.skin.get(sym);
    if (entry.frames && entry.frames.length > 0) {
      const period = Math.max(1, entry.period_ticks || 1);
      const idx = Math.floor(madi / period) % entry.frames.length;
      return entry.frames[idx];
    }
    if (entry.asset_uri) {
      return entry.asset_uri;
    }
    return null;
  }

  async function drawFrame(frame, madi) {
    canvas.width = frame.width || 1;
    canvas.height = frame.height || 1;
    if (state.viewScale !== 1) {
      canvas.style.width = `${canvas.width * state.viewScale}px`;
      canvas.style.height = `${canvas.height * state.viewScale}px`;
    } else {
      canvas.style.width = "";
      canvas.style.height = "";
    }
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (const cmd of frame.cmds) {
      switch (cmd.kind) {
        case "Clear":
          ctx.fillStyle = rgbaToCss(cmd.color);
          ctx.fillRect(0, 0, canvas.width, canvas.height);
          break;
        case "RectFill": {
          const x = applyAa(cmd, cmd.x);
          const y = applyAa(cmd, cmd.y);
          const w = applyAa(cmd, cmd.w);
          const h = applyAa(cmd, cmd.h);
          ctx.fillStyle = rgbaToCss(cmd.color);
          ctx.fillRect(x, y, w, h);
          break;
        }
        case "RectStroke": {
          const x = applyAa(cmd, cmd.x);
          const y = applyAa(cmd, cmd.y);
          const w = applyAa(cmd, cmd.w);
          const h = applyAa(cmd, cmd.h);
          const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
          ctx.strokeStyle = rgbaToCss(cmd.color);
          ctx.lineWidth = thickness;
          ctx.strokeRect(x, y, w, h);
          break;
        }
        case "Line": {
          const x1 = applyAa(cmd, cmd.x1);
          const y1 = applyAa(cmd, cmd.y1);
          const x2 = applyAa(cmd, cmd.x2);
          const y2 = applyAa(cmd, cmd.y2);
          const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
          ctx.strokeStyle = rgbaToCss(cmd.color);
          ctx.lineWidth = thickness;
          ctx.beginPath();
          ctx.moveTo(x1, y1);
          ctx.lineTo(x2, y2);
          ctx.stroke();
          break;
        }
        case "CircleFill": {
          const cx = applyAa(cmd, cmd.cx);
          const cy = applyAa(cmd, cmd.cy);
          const r = applyAa(cmd, cmd.r);
          ctx.fillStyle = rgbaToCss(cmd.color);
          ctx.beginPath();
          ctx.arc(cx, cy, r, 0, Math.PI * 2);
          ctx.fill();
          break;
        }
        case "CircleStroke": {
          const cx = applyAa(cmd, cmd.cx);
          const cy = applyAa(cmd, cmd.cy);
          const r = applyAa(cmd, cmd.r);
          const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
          ctx.strokeStyle = rgbaToCss(cmd.color);
          ctx.lineWidth = thickness;
          ctx.beginPath();
          ctx.arc(cx, cy, r, 0, Math.PI * 2);
          ctx.stroke();
          break;
        }
        case "ArcStroke": {
          const cx = applyAa(cmd, cmd.cx);
          const cy = applyAa(cmd, cmd.cy);
          const r = applyAa(cmd, cmd.r);
          const start = applyAa(cmd, cmd.start_turn ?? 0) * Math.PI * 2;
          const sweep = applyAa(cmd, cmd.sweep_turn ?? 0) * Math.PI * 2;
          const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
          const end = start + sweep;
          ctx.strokeStyle = rgbaToCss(cmd.color);
          ctx.lineWidth = thickness;
          ctx.beginPath();
          ctx.arc(cx, cy, r, start, end, sweep < 0);
          ctx.stroke();
          break;
        }
        case "CurveCubicStroke": {
          const p0x = applyAa(cmd, cmd.p0x);
          const p0y = applyAa(cmd, cmd.p0y);
          const p1x = applyAa(cmd, cmd.p1x);
          const p1y = applyAa(cmd, cmd.p1y);
          const p2x = applyAa(cmd, cmd.p2x);
          const p2y = applyAa(cmd, cmd.p2y);
          const p3x = applyAa(cmd, cmd.p3x);
          const p3y = applyAa(cmd, cmd.p3y);
          const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
          ctx.strokeStyle = rgbaToCss(cmd.color);
          ctx.lineWidth = thickness;
          ctx.beginPath();
          ctx.moveTo(p0x, p0y);
          ctx.bezierCurveTo(p1x, p1y, p2x, p2y, p3x, p3y);
          ctx.stroke();
          break;
        }
        case "Text": {
          const x = applyAa(cmd, cmd.x);
          const y = applyAa(cmd, cmd.y);
          const size = applyAa(cmd, cmd.size_px ?? cmd.size ?? 12);
          ctx.fillStyle = rgbaToCss(cmd.color);
          ctx.font = `${size}px sans-serif`;
          ctx.textBaseline = "top";
          ctx.fillText(cmd.text, x, y);
          break;
        }
        case "Sprite": {
          const x = applyAa(cmd, cmd.x);
          const y = applyAa(cmd, cmd.y);
          const w = applyAa(cmd, cmd.w);
          const h = applyAa(cmd, cmd.h);
          const tint = cmd.tint || cmd.color;
          if (tint && tint.a === 0) {
            break;
          }
          let uri = cmd.uri;
          if (uri && uri.startsWith("sym:")) {
            const resolved = resolveSymbol(uri, madi);
            if (resolved) {
              uri = resolved;
            }
          }
          if (uri) {
            const img = await loadImage(uri);
            if (img) {
              const prevSmooth = ctx.imageSmoothingEnabled;
              ctx.imageSmoothingEnabled = !!cmd.aa;
              ctx.drawImage(img, x, y, w, h);
              ctx.imageSmoothingEnabled = prevSmooth;
              break;
            }
          }
          const fallbackTint = tint || { r: 200, g: 200, b: 200, a: 255 };
          ctx.strokeStyle = rgbaToCss(fallbackTint);
          ctx.strokeRect(x, y, w, h);
          break;
        }
        default:
          break;
      }
    }
  }

  async function loadFrameData(index) {
    if (!state.manifest) {
      return null;
    }
    if (state.frameCache.has(index)) {
      return state.frameCache.get(index);
    }
    const frameMeta = state.manifest.frames[index];
    if (!frameMeta) {
      return null;
    }
    const path = `../${frameMeta.file}`;
    const res = await fetch(path);
    const buf = await res.arrayBuffer();
    const codec = state.manifest.codec || "BDL1";
    const parsed = codec === "BDL2" ? parseBdl2(buf) : parseBdl1(buf);
    state.frameCache.set(index, parsed);
    return parsed;
  }

  async function renderCurrent() {
    if (!state.manifest) {
      return;
    }
    const frameMeta = state.manifest.frames[state.current];
    if (!frameMeta) {
      return;
    }
    const parsed = await loadFrameData(state.current);
    if (!parsed) {
      return;
    }
    let prev = null;
    if (state.overlay.delta && state.current > 0) {
      prev = await loadFrameData(state.current - 1);
    }
    await drawFrame(parsed, frameMeta.madi);
    drawOverlay(parsed, prev);
    const liveLabel = state.live ? " live" : "";
    status.textContent = `madi=${frameMeta.madi}${liveLabel}`;
  }

  async function loadFrame(index) {
    if (!state.manifest) {
      return;
    }
    const frameMeta = state.manifest.frames[index];
    if (!frameMeta) {
      return;
    }
    state.current = index;
    seek.value = String(index);
    await loadFrameData(index);
    await renderCurrent();
  }

  function setPlaying(next) {
    state.playing = next;
    btnPlay.textContent = next ? "Pause" : "Play";
    if (next) {
      state.timer = setInterval(() => {
        step(1);
      }, 150);
    } else if (state.timer) {
      clearInterval(state.timer);
      state.timer = null;
    }
  }

  async function step(delta) {
    if (!state.manifest) {
      return;
    }
    const maxIndex = state.manifest.frames.length - 1;
    const next = Math.min(maxIndex, Math.max(0, state.current + delta));
    await loadFrame(next);
  }

  async function loadSkin() {
    try {
      const res = await fetch("skin.detjson");
      if (!res.ok) {
        return;
      }
      const json = await res.json();
      if (!json || !Array.isArray(json.symbols)) {
        return;
      }
      const map = new Map();
      for (const sym of json.symbols) {
        if (!sym || !sym.key) {
          continue;
        }
        const entry = { asset_uri: null, frames: null, period_ticks: null };
        if (sym.web) {
          if (sym.web.asset_uri) {
            entry.asset_uri = sym.web.asset_uri;
          }
          if (Array.isArray(sym.web.frames)) {
            entry.frames = sym.web.frames;
            entry.period_ticks = sym.web.period_ticks || 1;
          }
        }
        map.set(sym.key, entry);
      }
      state.skin = map;
    } catch (_) {
      // ignore
    }
  }

  async function loadOverlayConfig() {
    try {
      const res = await fetch("overlay.detjson");
      if (!res.ok) {
        return;
      }
      const json = await res.json();
      if (!json) {
        return;
      }
      state.overlay.grid = !!json.grid;
      state.overlay.bounds = !!json.bounds;
      state.overlay.delta = !!json.delta;
    } catch (_) {
      // ignore
    }
  }

  function syncOverlayToggles() {
    ovGrid.checked = state.overlay.grid;
    ovBounds.checked = state.overlay.bounds;
    ovDelta.checked = state.overlay.delta;
  }

  function updateOverlayFromToggles() {
    state.overlay.grid = ovGrid.checked;
    state.overlay.bounds = ovBounds.checked;
    state.overlay.delta = ovDelta.checked;
  }

  function setControlsEnabled(enabled) {
    btnPlay.disabled = !enabled;
    btnStepBack.disabled = !enabled;
    btnStepForward.disabled = !enabled;
    seek.disabled = !enabled;
  }

  function sendInputEvent(code, kind) {
    if (!inputEndpoint || !state.live) {
      return;
    }
    const url = `${inputEndpoint}?code=${encodeURIComponent(code)}&kind=${encodeURIComponent(kind)}`;
    fetch(url, { method: "GET", mode: "cors", cache: "no-store" }).catch(() => {});
  }

  function clearInput() {
    if (!inputEndpoint || !state.live) {
      return;
    }
    const url = `${inputEndpoint}?kind=clear`;
    fetch(url, { method: "GET", mode: "cors", cache: "no-store" }).catch(() => {});
  }

  function handleKeyDown(ev) {
    if (!inputEndpoint || !state.live) {
      return;
    }
    const code = ev.code;
    if (!inputKeys.has(code)) {
      return;
    }
    if (ev.repeat) {
      ev.preventDefault();
      return;
    }
    sendInputEvent(code, "down");
    ev.preventDefault();
  }

  function handleKeyUp(ev) {
    if (!inputEndpoint || !state.live) {
      return;
    }
    const code = ev.code;
    if (!inputKeys.has(code)) {
      return;
    }
    sendInputEvent(code, "up");
    ev.preventDefault();
  }

  function setupInputCapture() {
    if (!inputEndpoint || !state.live) {
      return;
    }
    window.addEventListener("keydown", handleKeyDown, { passive: false });
    window.addEventListener("keyup", handleKeyUp, { passive: false });
    window.addEventListener("blur", clearInput);
    document.addEventListener("visibilitychange", () => {
      if (document.hidden) {
        clearInput();
      }
    });
  }

  async function fetchManifest(noCache) {
    try {
      const stamp = noCache ? `?t=${Date.now()}` : "";
      const res = await fetch(`../manifest.detjson${stamp}`, { cache: "no-store" });
      if (!res.ok) {
        return null;
      }
      return await res.json();
    } catch (_) {
      return null;
    }
  }

  async function refreshManifest() {
    const json = await fetchManifest(true);
    if (!json) {
      status.textContent = "waiting...";
      return;
    }
    state.manifest = json;
    state.frames = json.frames || [];
    if (state.frames.length === 0) {
      status.textContent = "waiting...";
      return;
    }
    seek.max = String(Math.max(0, state.frames.length - 1));
    const latest = state.frames.length - 1;
    await loadFrame(latest);
  }

  function commandKey(cmd) {
    return JSON.stringify(cmd);
  }

  function commandBounds(cmd, width, height, includeClear) {
    if (!cmd) {
      return null;
    }
    switch (cmd.kind) {
      case "Clear":
        return includeClear ? { x: 0, y: 0, w: width, h: height } : null;
      case "RectFill":
      case "RectStroke":
      case "Sprite": {
        const w = applyAa(cmd, cmd.w);
        const h = applyAa(cmd, cmd.h);
        if (w <= 0 || h <= 0) {
          return null;
        }
        return { x: applyAa(cmd, cmd.x), y: applyAa(cmd, cmd.y), w, h };
      }
      case "Line": {
        const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
        const x1 = applyAa(cmd, cmd.x1);
        const y1 = applyAa(cmd, cmd.y1);
        const x2 = applyAa(cmd, cmd.x2);
        const y2 = applyAa(cmd, cmd.y2);
        const minX = Math.min(x1, x2) - thickness / 2;
        const minY = Math.min(y1, y2) - thickness / 2;
        const maxX = Math.max(x1, x2) + thickness / 2;
        const maxY = Math.max(y1, y2) + thickness / 2;
        return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
      }
      case "CircleFill":
      case "CircleStroke":
      case "ArcStroke": {
        const r = applyAa(cmd, cmd.r);
        if (r <= 0) {
          return null;
        }
        const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
        const pad = cmd.kind === "CircleFill" ? 0 : thickness / 2;
        const x = applyAa(cmd, cmd.cx) - r - pad;
        const y = applyAa(cmd, cmd.cy) - r - pad;
        const size = r * 2 + pad * 2;
        return { x, y, w: size, h: size };
      }
      case "CurveCubicStroke": {
        const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
        const xs = [cmd.p0x, cmd.p1x, cmd.p2x, cmd.p3x].map((v) => applyAa(cmd, v));
        const ys = [cmd.p0y, cmd.p1y, cmd.p2y, cmd.p3y].map((v) => applyAa(cmd, v));
        const minX = Math.min(...xs) - thickness / 2;
        const minY = Math.min(...ys) - thickness / 2;
        const maxX = Math.max(...xs) + thickness / 2;
        const maxY = Math.max(...ys) + thickness / 2;
        return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
      }
      case "Text": {
        const size = applyAa(cmd, cmd.size ?? cmd.size_px ?? 12);
        const text = cmd.text || "";
        const w = Math.ceil(text.length * size * 0.6);
        return { x: applyAa(cmd, cmd.x), y: applyAa(cmd, cmd.y), w, h: size };
      }
      default:
        return null;
    }
  }

  function mergeBounds(base, next) {
    if (!next) {
      return base;
    }
    if (!base) {
      return { x: next.x, y: next.y, w: next.w, h: next.h };
    }
    const minX = Math.min(base.x, next.x);
    const minY = Math.min(base.y, next.y);
    const maxX = Math.max(base.x + base.w, next.x + next.w);
    const maxY = Math.max(base.y + base.h, next.y + next.h);
    return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
  }

  function computeBounds(cmds, width, height) {
    let bounds = null;
    for (const cmd of cmds) {
      bounds = mergeBounds(bounds, commandBounds(cmd, width, height, false));
    }
    return bounds;
  }

  function computeGridSize(cmds) {
    const counts = new Map();
    for (const cmd of cmds) {
      if (cmd.kind !== "RectFill" && cmd.kind !== "RectStroke" && cmd.kind !== "Sprite") {
        continue;
      }
      const w = applyAa(cmd, cmd.w);
      const h = applyAa(cmd, cmd.h);
      if (w <= 0 || h <= 0) {
        continue;
      }
      const key = `${w}x${h}`;
      counts.set(key, (counts.get(key) || 0) + 1);
    }
    let best = null;
    for (const [key, count] of counts.entries()) {
      const [w, h] = key.split("x").map((v) => Number(v));
      const area = w * h;
      if (!best || count > best.count || (count === best.count && area < best.area)) {
        best = { w, h, count, area };
      }
    }
    return best ? { w: best.w, h: best.h } : null;
  }

  function drawGrid(frame) {
    const size = computeGridSize(frame.cmds);
    if (!size) {
      return;
    }
    ctx.save();
    ctx.strokeStyle = "rgba(255,255,255,0.12)";
    ctx.lineWidth = 1;
    for (let x = 0; x <= frame.width; x += size.w) {
      ctx.beginPath();
      ctx.moveTo(x + 0.5, 0);
      ctx.lineTo(x + 0.5, frame.height);
      ctx.stroke();
    }
    for (let y = 0; y <= frame.height; y += size.h) {
      ctx.beginPath();
      ctx.moveTo(0, y + 0.5);
      ctx.lineTo(frame.width, y + 0.5);
      ctx.stroke();
    }
    ctx.restore();
  }

  function drawBounds(frame) {
    const bounds = computeBounds(frame.cmds, frame.width, frame.height);
    if (!bounds) {
      return;
    }
    ctx.save();
    ctx.strokeStyle = "rgba(0,200,255,0.7)";
    ctx.lineWidth = 1;
    ctx.strokeRect(bounds.x, bounds.y, bounds.w, bounds.h);
    ctx.restore();
  }

  function drawDelta(curr, prev) {
    if (!prev) {
      return;
    }
    const max = Math.max(curr.cmds.length, prev.cmds.length);
    let bounds = null;
    for (let i = 0; i < max; i += 1) {
      const a = curr.cmds[i];
      const b = prev.cmds[i];
      if (!a || !b || commandKey(a) !== commandKey(b)) {
        const box = commandBounds(a || b, curr.width, curr.height, true);
        bounds = mergeBounds(bounds, box);
      }
    }
    if (!bounds) {
      return;
    }
    ctx.save();
    ctx.fillStyle = "rgba(255,200,0,0.25)";
    ctx.fillRect(bounds.x, bounds.y, bounds.w, bounds.h);
    ctx.restore();
  }

  function drawOverlay(curr, prev) {
    if (state.overlay.grid) {
      drawGrid(curr);
    }
    if (state.overlay.bounds) {
      drawBounds(curr);
    }
    if (state.overlay.delta) {
      drawDelta(curr, prev);
    }
  }

  async function init() {
    await loadSkin();
    await loadOverlayConfig();
    syncOverlayToggles();
    if (state.live) {
      setControlsEnabled(false);
      await refreshManifest();
      state.liveTimer = setInterval(refreshManifest, 300);
    } else {
      const json = await fetchManifest(false);
      if (!json) {
        throw new Error("manifest not found");
      }
      state.manifest = json;
      state.frames = json.frames || [];
      seek.max = String(Math.max(0, state.frames.length - 1));
      seek.value = "0";
      state.current = 0;
      setControlsEnabled(true);
      await loadFrame(0);
    }

    btnPlay.addEventListener("click", () => {
      if (state.live) {
        return;
      }
      setPlaying(!state.playing);
    });
    btnStepBack.addEventListener("click", () => {
      if (state.live) {
        return;
      }
      step(-1);
    });
    btnStepForward.addEventListener("click", () => {
      if (state.live) {
        return;
      }
      step(1);
    });
    seek.addEventListener("input", async (ev) => {
      if (state.live) {
        return;
      }
      const value = Number(ev.target.value || 0);
      await loadFrame(value);
    });
    ovGrid.addEventListener("change", async () => {
      updateOverlayFromToggles();
      await renderCurrent();
    });
    ovBounds.addEventListener("change", async () => {
      updateOverlayFromToggles();
      await renderCurrent();
    });
    ovDelta.addEventListener("change", async () => {
      updateOverlayFromToggles();
      await renderCurrent();
    });
    setupInputCapture();
  }

  init().catch((err) => {
    console.error(err);
    status.textContent = "failed to load";
  });
})();
"#
        .to_string()
}

fn copy_skin_assets(out_dir: &Path, skin_path: &Path) {
    let text = match fs::read_to_string(skin_path) {
        Ok(text) => text,
        Err(_) => return,
    };
    let json: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => return,
    };
    let Some(symbols) = json.get("symbols").and_then(|value| value.as_array()) else {
        return;
    };
    let base = skin_path.parent().unwrap_or_else(|| Path::new("."));
    for sym in symbols {
        let Some(web) = sym.get("web") else {
            continue;
        };
        if let Some(uri) = web.get("asset_uri").and_then(|value| value.as_str()) {
            copy_asset_if_relative(base, out_dir, uri);
        }
        if let Some(frames) = web.get("frames").and_then(|value| value.as_array()) {
            for frame in frames {
                if let Some(uri) = frame.as_str() {
                    copy_asset_if_relative(base, out_dir, uri);
                }
            }
        }
    }
}

fn copy_asset_if_relative(base: &Path, out_dir: &Path, uri: &str) {
    if !is_relative_asset(uri) {
        return;
    }
    let src = base.join(uri);
    if !src.exists() {
        return;
    }
    let dest = out_dir.join(uri);
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::copy(src, dest);
}

fn is_relative_asset(uri: &str) -> bool {
    if uri.starts_with('/') || uri.starts_with('\\') {
        return false;
    }
    if uri.contains(':') {
        return false;
    }
    true
}
