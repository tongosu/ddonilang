use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::cli::bogae::OverlayConfig;
use crate::core::bogae::{BogaeCmd, BogaeCodec, BogaeDrawListV1, Rgba};

pub fn write_web_assets(
    out_dir: &Path,
    drawlist: &BogaeDrawListV1,
    detbin: &[u8],
    codec: BogaeCodec,
    skin_source: Option<&Path>,
    overlay: OverlayConfig,
) -> Result<PathBuf, String> {
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let json_text = build_drawlist_json(drawlist)?;
    let html_text = build_index_html(&json_text);
    let detbin_name = format!("drawlist.{}", codec.file_ext());
    fs::write(out_dir.join("drawlist.json"), &json_text).map_err(|e| e.to_string())?;
    fs::write(out_dir.join(detbin_name), detbin).map_err(|e| e.to_string())?;
    fs::write(out_dir.join("overlay.detjson"), overlay.to_detjson()).map_err(|e| e.to_string())?;
    let index_path = out_dir.join("index.html");
    fs::write(&index_path, html_text).map_err(|e| e.to_string())?;
    if let Some(skin_path) = skin_source {
        let target = out_dir.join("skin.detjson");
        if fs::copy(skin_path, &target).is_ok() {
            copy_skin_assets(out_dir, skin_path);
        }
    }
    Ok(index_path)
}

fn build_drawlist_json(drawlist: &BogaeDrawListV1) -> Result<String, String> {
    let cmds = drawlist.cmds.iter().map(cmd_to_json).collect::<Vec<_>>();
    let payload = json!({
        "version": 1,
        "width_px": drawlist.width_px,
        "height_px": drawlist.height_px,
        "cmds": cmds,
    });
    serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())
}

fn cmd_to_json(cmd: &BogaeCmd) -> serde_json::Value {
    match cmd {
        BogaeCmd::Clear { color, aa } => json!({
            "kind": "Clear",
            "color": rgba_to_json(*color),
            "aa": aa,
        }),
        BogaeCmd::RectFill {
            x,
            y,
            w,
            h,
            color,
            aa,
        } => json!({
            "kind": "RectFill",
            "x": x,
            "y": y,
            "w": w,
            "h": h,
            "color": rgba_to_json(*color),
            "aa": aa,
        }),
        BogaeCmd::RectStroke {
            x,
            y,
            w,
            h,
            thickness,
            color,
            aa,
        } => json!({
            "kind": "RectStroke",
            "x": x,
            "y": y,
            "w": w,
            "h": h,
            "thickness": thickness,
            "color": rgba_to_json(*color),
            "aa": aa,
        }),
        BogaeCmd::Line {
            x1,
            y1,
            x2,
            y2,
            thickness,
            color,
            aa,
        } => json!({
            "kind": "Line",
            "x1": x1,
            "y1": y1,
            "x2": x2,
            "y2": y2,
            "thickness": thickness,
            "color": rgba_to_json(*color),
            "aa": aa,
        }),
        BogaeCmd::Text {
            x,
            y,
            size_px,
            color,
            text,
            aa,
        } => json!({
            "kind": "Text",
            "x": x,
            "y": y,
            "size_px": size_px,
            "color": rgba_to_json(*color),
            "text": text,
            "aa": aa,
        }),
        BogaeCmd::Sprite {
            x,
            y,
            w,
            h,
            tint,
            asset,
            aa,
        } => json!({
            "kind": "Sprite",
            "x": x,
            "y": y,
            "w": w,
            "h": h,
            "tint": rgba_to_json(*tint),
            "uri": asset.uri,
            "aa": aa,
        }),
        BogaeCmd::CircleFill {
            cx,
            cy,
            r,
            color,
            aa,
        } => json!({
            "kind": "CircleFill",
            "cx": cx,
            "cy": cy,
            "r": r,
            "color": rgba_to_json(*color),
            "aa": aa,
        }),
        BogaeCmd::CircleStroke {
            cx,
            cy,
            r,
            thickness,
            color,
            aa,
        } => json!({
            "kind": "CircleStroke",
            "cx": cx,
            "cy": cy,
            "r": r,
            "thickness": thickness,
            "color": rgba_to_json(*color),
            "aa": aa,
        }),
        BogaeCmd::ArcStroke {
            cx,
            cy,
            r,
            start_turn,
            sweep_turn,
            thickness,
            color,
            aa,
        } => json!({
            "kind": "ArcStroke",
            "cx": cx,
            "cy": cy,
            "r": r,
            "start_turn": start_turn,
            "sweep_turn": sweep_turn,
            "thickness": thickness,
            "color": rgba_to_json(*color),
            "aa": aa,
        }),
        BogaeCmd::CurveCubicStroke {
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
        } => json!({
            "kind": "CurveCubicStroke",
            "p0x": p0x,
            "p0y": p0y,
            "p1x": p1x,
            "p1y": p1y,
            "p2x": p2x,
            "p2y": p2y,
            "p3x": p3x,
            "p3y": p3y,
            "thickness": thickness,
            "color": rgba_to_json(*color),
            "aa": aa,
        }),
    }
}

fn rgba_to_json(rgba: Rgba) -> serde_json::Value {
    json!({
        "r": rgba.r,
        "g": rgba.g,
        "b": rgba.b,
        "a": rgba.a,
    })
}

fn build_index_html(json_text: &str) -> String {
    let safe_json = json_text.replace("</", "<\\/");
    format!(
        r#"<!doctype html>
<html lang="ko">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Bogae Viewer</title>
  <style>
    html, body {{
      margin: 0;
      padding: 0;
      background: #111;
      color: #eee;
      height: 100%;
    }}
    #wrap {{
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100%;
    }}
    canvas {{
      background: #000;
      image-rendering: pixelated;
      box-shadow: 0 12px 30px rgba(0,0,0,0.45);
    }}
  </style>
</head>
<body>
  <div id="wrap">
    <canvas id="canvas"></canvas>
  </div>
  <script>
    const drawlist = {safe_json};
    const canvas = document.getElementById("canvas");
    canvas.width = drawlist.width_px || 1;
    canvas.height = drawlist.height_px || 1;
    const params = new URLSearchParams(window.location.search);
    const scaleRaw = params.get("scale") || "1.25";
    const scaleValue = Number(scaleRaw);
    const viewScale = Number.isFinite(scaleValue) && scaleValue > 0 ? scaleValue : 1.25;
    canvas.style.width = `${{canvas.width * viewScale}}px`;
    canvas.style.height = `${{canvas.height * viewScale}}px`;
    const ctx = canvas.getContext("2d");
    ctx.imageSmoothingEnabled = false;
    const state = {{
      cache: new Map(),
      skin: null,
      overlay: {{ grid: false, bounds: false, delta: false }},
    }};

    function rgbaToCss(color) {{
      return `rgba(${{color.r}},${{color.g}},${{color.b}},${{color.a / 255}})`;
    }}

    function snapValue(value) {{
      if (!Number.isFinite(value)) {{
        return 0;
      }}
      return value >= 0 ? Math.floor(value + 0.5) : Math.ceil(value - 0.5);
    }}

    function numeric(value) {{
      return Number.isFinite(value) ? value : 0;
    }}

    function applyAa(cmd, value) {{
      const v = numeric(value);
      return cmd && cmd.aa ? v : snapValue(v);
    }}

    function resolveSymbol(sym) {{
      if (!state.skin || !state.skin.has(sym)) {{
        return null;
      }}
      const entry = state.skin.get(sym);
      if (entry.frames && entry.frames.length > 0) {{
        return entry.frames[0];
      }}
      if (entry.asset_uri) {{
        return entry.asset_uri;
      }}
      return null;
    }}

    function resolveUri(uri) {{
      try {{
        return new URL(uri, window.location.href).href;
      }} catch (_) {{
        return uri;
      }}
    }}

    async function loadImage(src) {{
      if (!src) {{
        return null;
      }}
      if (state.cache.has(src)) {{
        return state.cache.get(src);
      }}
      const promise = new Promise((resolve) => {{
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = () => resolve(null);
        img.src = src;
      }});
      state.cache.set(src, promise);
      return promise;
    }}

    async function loadSkin() {{
      try {{
        const res = await fetch("skin.detjson");
        if (!res.ok) {{
          return;
        }}
        const json = await res.json();
        if (!json || !Array.isArray(json.symbols)) {{
          return;
        }}
        const map = new Map();
        for (const sym of json.symbols) {{
          if (!sym || !sym.key) {{
            continue;
          }}
          const entry = {{ asset_uri: null, frames: null, period_ticks: null }};
          if (sym.web) {{
            if (sym.web.asset_uri) {{
              entry.asset_uri = sym.web.asset_uri;
            }}
            if (Array.isArray(sym.web.frames)) {{
              entry.frames = sym.web.frames;
              entry.period_ticks = sym.web.period_ticks || 1;
            }}
          }}
          map.set(sym.key, entry);
        }}
        state.skin = map;
      }} catch (_) {{
        // ignore
      }}
    }}

    async function loadOverlayConfig() {{
      try {{
        const res = await fetch("overlay.detjson");
        if (!res.ok) {{
          return;
        }}
        const json = await res.json();
        if (!json) {{
          return;
        }}
        state.overlay.grid = !!json.grid;
        state.overlay.bounds = !!json.bounds;
        state.overlay.delta = !!json.delta;
      }} catch (_) {{
        // ignore
      }}
    }}

    function commandBounds(cmd, width, height) {{
      switch (cmd.kind) {{
        case "RectFill":
        case "RectStroke":
        case "Sprite": {{
          const w = applyAa(cmd, cmd.w);
          const h = applyAa(cmd, cmd.h);
          if (w <= 0 || h <= 0) {{
            return null;
          }}
          return {{
            x: applyAa(cmd, cmd.x),
            y: applyAa(cmd, cmd.y),
            w,
            h,
          }};
        }}
        case "Line": {{
          const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
          const x1 = applyAa(cmd, cmd.x1);
          const y1 = applyAa(cmd, cmd.y1);
          const x2 = applyAa(cmd, cmd.x2);
          const y2 = applyAa(cmd, cmd.y2);
          const minX = Math.min(x1, x2) - thickness / 2;
          const minY = Math.min(y1, y2) - thickness / 2;
          const maxX = Math.max(x1, x2) + thickness / 2;
          const maxY = Math.max(y1, y2) + thickness / 2;
          return {{ x: minX, y: minY, w: maxX - minX, h: maxY - minY }};
        }}
        case "CircleFill":
        case "CircleStroke":
        case "ArcStroke": {{
          const r = applyAa(cmd, cmd.r);
          if (r <= 0) {{
            return null;
          }}
          const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
          const pad = cmd.kind === "CircleFill" ? 0 : thickness / 2;
          const x = applyAa(cmd, cmd.cx) - r - pad;
          const y = applyAa(cmd, cmd.cy) - r - pad;
          const size = r * 2 + pad * 2;
          return {{ x, y, w: size, h: size }};
        }}
        case "CurveCubicStroke": {{
          const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
          const xs = [cmd.p0x, cmd.p1x, cmd.p2x, cmd.p3x].map((v) => applyAa(cmd, v));
          const ys = [cmd.p0y, cmd.p1y, cmd.p2y, cmd.p3y].map((v) => applyAa(cmd, v));
          const minX = Math.min(...xs) - thickness / 2;
          const minY = Math.min(...ys) - thickness / 2;
          const maxX = Math.max(...xs) + thickness / 2;
          const maxY = Math.max(...ys) + thickness / 2;
          return {{ x: minX, y: minY, w: maxX - minX, h: maxY - minY }};
        }}
        case "Text": {{
          const size = applyAa(cmd, cmd.size_px ?? cmd.size ?? 12);
          const w = Math.ceil((cmd.text || \"\").length * size * 0.6);
          return {{ x: applyAa(cmd, cmd.x), y: applyAa(cmd, cmd.y), w, h: size }};
        }}
        default:
          return null;
      }}
    }}

    function mergeBounds(base, next) {{
      if (!next) {{
        return base;
      }}
      if (!base) {{
        return {{ x: next.x, y: next.y, w: next.w, h: next.h }};
      }}
      const minX = Math.min(base.x, next.x);
      const minY = Math.min(base.y, next.y);
      const maxX = Math.max(base.x + base.w, next.x + next.w);
      const maxY = Math.max(base.y + base.h, next.y + next.h);
      return {{ x: minX, y: minY, w: maxX - minX, h: maxY - minY }};
    }}

    function computeBounds(cmds, width, height) {{
      let bounds = null;
      for (const cmd of cmds) {{
        bounds = mergeBounds(bounds, commandBounds(cmd, width, height));
      }}
      return bounds;
    }}

    function computeGridSize(cmds) {{
      const counts = new Map();
      for (const cmd of cmds) {{
        if (cmd.kind !== "RectFill" && cmd.kind !== "RectStroke" && cmd.kind !== "Sprite") {{
          continue;
        }}
        const w = applyAa(cmd, cmd.w);
        const h = applyAa(cmd, cmd.h);
        if (w <= 0 || h <= 0) {{
          continue;
        }}
        const key = `${{w}}x${{h}}`;
        counts.set(key, (counts.get(key) || 0) + 1);
      }}
      let best = null;
      for (const [key, count] of counts.entries()) {{
        const parts = key.split("x");
        const w = Number(parts[0]);
        const h = Number(parts[1]);
        const area = w * h;
        if (!best || count > best.count || (count === best.count && area < best.area)) {{
          best = {{ w, h, count, area }};
        }}
      }}
      return best ? {{ w: best.w, h: best.h }} : null;
    }}

    function drawGrid(cmds) {{
      const size = computeGridSize(cmds);
      if (!size) {{
        return;
      }}
      ctx.save();
      ctx.strokeStyle = "rgba(255,255,255,0.12)";
      ctx.lineWidth = 1;
      for (let x = 0; x <= canvas.width; x += size.w) {{
        ctx.beginPath();
        ctx.moveTo(x + 0.5, 0);
        ctx.lineTo(x + 0.5, canvas.height);
        ctx.stroke();
      }}
      for (let y = 0; y <= canvas.height; y += size.h) {{
        ctx.beginPath();
        ctx.moveTo(0, y + 0.5);
        ctx.lineTo(canvas.width, y + 0.5);
        ctx.stroke();
      }}
      ctx.restore();
    }}

    function drawBounds(cmds) {{
      const bounds = computeBounds(cmds, canvas.width, canvas.height);
      if (!bounds) {{
        return;
      }}
      ctx.save();
      ctx.strokeStyle = "rgba(0,200,255,0.7)";
      ctx.lineWidth = 1;
      ctx.strokeRect(bounds.x, bounds.y, bounds.w, bounds.h);
      ctx.restore();
    }}

    function drawOverlay(cmds) {{
      if (state.overlay.grid) {{
        drawGrid(cmds);
      }}
      if (state.overlay.bounds) {{
        drawBounds(cmds);
      }}
    }}

    async function render() {{
      await loadSkin();
      await loadOverlayConfig();
      for (const cmd of drawlist.cmds) {{
        switch (cmd.kind) {{
          case "Clear":
            ctx.fillStyle = rgbaToCss(cmd.color);
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            break;
          case "RectFill": {{
            const x = applyAa(cmd, cmd.x);
            const y = applyAa(cmd, cmd.y);
            const w = applyAa(cmd, cmd.w);
            const h = applyAa(cmd, cmd.h);
            ctx.fillStyle = rgbaToCss(cmd.color);
            ctx.fillRect(x, y, w, h);
            break;
          }}
          case "RectStroke": {{
            const x = applyAa(cmd, cmd.x);
            const y = applyAa(cmd, cmd.y);
            const w = applyAa(cmd, cmd.w);
            const h = applyAa(cmd, cmd.h);
            const thickness = Math.max(1, applyAa(cmd, cmd.thickness ?? 1));
            ctx.strokeStyle = rgbaToCss(cmd.color);
            ctx.lineWidth = thickness;
            ctx.strokeRect(x, y, w, h);
            break;
          }}
          case "Line": {{
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
          }}
          case "CircleFill": {{
            const cx = applyAa(cmd, cmd.cx);
            const cy = applyAa(cmd, cmd.cy);
            const r = applyAa(cmd, cmd.r);
            ctx.fillStyle = rgbaToCss(cmd.color);
            ctx.beginPath();
            ctx.arc(cx, cy, r, 0, Math.PI * 2);
            ctx.fill();
            break;
          }}
          case "CircleStroke": {{
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
          }}
          case "ArcStroke": {{
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
          }}
          case "CurveCubicStroke": {{
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
          }}
          case "Text": {{
            const x = applyAa(cmd, cmd.x);
            const y = applyAa(cmd, cmd.y);
            const size = applyAa(cmd, cmd.size_px ?? cmd.size ?? 12);
            ctx.fillStyle = rgbaToCss(cmd.color);
            ctx.font = `${{size}}px sans-serif`;
            ctx.textBaseline = "top";
            ctx.fillText(cmd.text, x, y);
            break;
          }}
          case "Sprite": {{
            const x = applyAa(cmd, cmd.x);
            const y = applyAa(cmd, cmd.y);
            const w = applyAa(cmd, cmd.w);
            const h = applyAa(cmd, cmd.h);
            const tint = cmd.tint || cmd.color;
            if (tint && tint.a === 0) {{
              break;
            }}
            let uri = cmd.uri;
            if (uri && uri.startsWith("sym:")) {{
              const resolved = resolveSymbol(uri);
              if (resolved) {{
                uri = resolved;
              }}
            }}
            if (uri) {{
              const img = await loadImage(resolveUri(uri));
              if (img) {{
                const prevSmooth = ctx.imageSmoothingEnabled;
                ctx.imageSmoothingEnabled = !!cmd.aa;
                ctx.drawImage(img, x, y, w, h);
                ctx.imageSmoothingEnabled = prevSmooth;
                break;
              }}
            }}
            const fallbackTint = tint || {{ r: 200, g: 200, b: 200, a: 255 }};
            ctx.strokeStyle = rgbaToCss(fallbackTint);
            ctx.strokeRect(x, y, w, h);
            break;
          }}
          default:
            break;
        }}
      }}
      drawOverlay(drawlist.cmds);
    }}

    render();
  </script>
</body>
</html>
"#
    )
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
