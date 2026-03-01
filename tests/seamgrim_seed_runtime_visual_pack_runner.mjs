import fs from "fs";
import path from "path";
import os from "os";
import { spawnSync } from "child_process";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function resolveTeulCliBin(root) {
  const envBin = String(process.env.TEUL_CLI_BIN ?? "").trim();
  if (envBin && fs.existsSync(envBin)) return envBin;

  const suffix = process.platform === "win32" ? ".exe" : "";
  const candidates = [
    path.resolve(root, "target", "debug", `teul-cli${suffix}`),
    path.resolve(root, "target", "release", `teul-cli${suffix}`),
    "I:/home/urihanl/ddn/codex/target/debug/teul-cli.exe",
  ];
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) return candidate;
  }
  return "";
}

function runTeulCli(root, lessonPath, madi) {
  const directBin = resolveTeulCliBin(root);
  if (directBin) {
    return spawnSync(directBin, ["run", lessonPath, "--madi", String(madi)], {
      cwd: root,
      encoding: "utf-8",
    });
  }
  return spawnSync(
    "cargo",
    ["run", "--manifest-path", "tools/teul-cli/Cargo.toml", "--", "run", lessonPath, "--madi", String(madi)],
    { cwd: root, encoding: "utf-8" },
  );
}

function preprocessForTeul(root, lessonPath, rawText) {
  const python = String(process.env.PYTHON ?? "python").trim() || "python";
  const script = [
    "import sys, pathlib",
    "root = pathlib.Path(sys.argv[1])",
    "lesson = pathlib.Path(sys.argv[2])",
    "sys.path.insert(0, str(root / 'solutions' / 'seamgrim_ui_mvp' / 'tools'))",
    "from export_graph import preprocess_ddn_for_teul",
    "text = lesson.read_text(encoding='utf-8')",
    "out = preprocess_ddn_for_teul(text, strip_draw=True)",
    "sys.stdout.write(out)",
  ].join("\n");
  const proc = spawnSync(python, ["-X", "utf8", "-c", script, root, lessonPath], {
    cwd: root,
    encoding: "utf-8",
    env: {
      ...process.env,
      PYTHONUTF8: "1",
      PYTHONIOENCODING: "utf-8",
    },
  });
  if (proc.status !== 0) {
    return rawText;
  }
  const out = String(proc.stdout ?? "");
  return out.trim() ? out : rawText;
}

function parseNumericLines(stdout) {
  const out = [];
  const lines = String(stdout ?? "").split(/\r?\n/u);
  for (const raw of lines) {
    const line = String(raw ?? "").trim();
    if (!line) continue;
    if (line.startsWith("state_hash=") || line.startsWith("trace_hash=") || line.startsWith("bogae_hash=")) {
      continue;
    }
    const value = Number(line);
    if (Number.isFinite(value)) out.push(value);
  }
  return out;
}

function splitOutputLines(stdout) {
  return String(stdout ?? "")
    .split(/\r?\n/u)
    .map((line) => String(line ?? "").trim())
    .filter(Boolean);
}

function detectNativeSpace2d(lines) {
  const rows = Array.isArray(lines) ? lines : [];
  for (let i = 0; i < rows.length; i += 1) {
    const line = String(rows[i] ?? "").trim().toLowerCase();
    if (line !== "space2d.shape" && line !== "space2d_shape" && line !== "shape2d") continue;
    const windowEnd = Math.min(rows.length, i + 20);
    for (let j = i + 1; j < windowEnd; j += 1) {
      const next = String(rows[j] ?? "").trim().toLowerCase();
      if (next === "point" || next === "line" || next === "circle" || next === "점" || next === "선" || next === "원") {
        return true;
      }
      if (next === "space2d.shape" || next === "space2d" || next.startsWith("series:")) {
        break;
      }
    }
  }
  return false;
}

function extractSeriesPointsFromOutput(lines, seriesId) {
  const out = [];
  const rows = Array.isArray(lines) ? lines : [];
  const target = `series:${String(seriesId ?? "").trim().toLowerCase()}`;
  if (!target || target === "series:") return out;

  for (let i = 0; i < rows.length; i += 1) {
    const line = String(rows[i] ?? "").trim();
    if (line.toLowerCase() !== target) continue;
    const values = [];
    for (let j = i + 1; j < rows.length; j += 1) {
      const next = String(rows[j] ?? "").trim();
      if (!next) continue;
      if (next.toLowerCase().startsWith("series:")) break;
      if (next === "space2d" || next === "space2d.shape" || next === "space2d_shape" || next === "shape2d") {
        break;
      }
      const n = Number(next);
      if (!Number.isFinite(n)) continue;
      values.push(n);
      if (values.length >= 2) break;
    }
    if (values.length >= 2) {
      out.push({ x: values[0], y: values[1] });
    }
  }
  return out;
}

function readDefaultObservationKey(lessonText) {
  const hit = String(lessonText ?? "").match(/^\s*#기본관찰\s*:\s*([A-Za-z0-9_가-힣]+)\s*$/imu);
  return String(hit?.[1] ?? "").trim();
}

function readNumericAssignment(lessonText, key) {
  const escaped = String(key ?? "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const patterns = [
    new RegExp(
      `^\\s*${escaped}\\s*:\\s*[^<>=\\n]+<-\\s*([+-]?(?:\\d+(?:\\.\\d+)?|\\.\\d+)(?:e[+-]?\\d+)?)\\s*\\.`,
      "imu",
    ),
    new RegExp(`^\\s*${escaped}\\s*<-\\s*([+-]?(?:\\d+(?:\\.\\d+)?|\\.\\d+)(?:e[+-]?\\d+)?)\\s*\\.`, "imu"),
  ];
  for (const pattern of patterns) {
    const hit = String(lessonText ?? "").match(pattern);
    if (!hit) continue;
    const value = Number(hit[1]);
    if (Number.isFinite(value)) return value;
  }
  return null;
}

function extractTickBlockShowVars(lessonText) {
  const lines = String(lessonText ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  const out = [];
  let inTick = false;
  let depth = 0;
  let inBoim = false;
  let boimDepth = 0;

  function countChar(text, ch) {
    let count = 0;
    for (let i = 0; i < text.length; i += 1) {
      if (text[i] === ch) count += 1;
    }
    return count;
  }

  for (const line of lines) {
    if (!inTick) {
      if (line.includes("(매마디)마다")) {
        inTick = true;
        depth = Math.max(1, countChar(line, "{") - countChar(line, "}") || 1);
      }
      continue;
    }

    if (inBoim) {
      const boimItem = line.match(/^\s*([A-Za-z0-9_가-힣]+)\s*:\s*(.+)\.\s*$/u);
      if (boimItem) {
        out.push(String(boimItem[1] ?? "").trim());
      }
      boimDepth += countChar(line, "{");
      boimDepth -= countChar(line, "}");
      if (boimDepth <= 0) {
        inBoim = false;
        boimDepth = 0;
      }
    } else {
      const boimOpen = line.match(/^\s*보임\s*\{\s*(?:\/\/.*)?$/u);
      if (boimOpen) {
        inBoim = true;
        boimDepth = Math.max(1, countChar(line, "{") - countChar(line, "}") || 1);
      }
      const show = line.match(/^\s*([A-Za-z0-9_가-힣]+)\s+보여주기\.\s*$/u);
      if (show) {
        out.push(String(show[1] ?? "").trim());
      }
    }

    depth += countChar(line, "{");
    depth -= countChar(line, "}");
    if (depth <= 0) {
      break;
    }
  }
  return out;
}

function buildSeriesPoints(numbers, pairCount, targetPairIndex) {
  const values = [...numbers];
  if (values.length % 2 !== 0) values.pop();
  const pairs = [];
  for (let i = 0; i < values.length; i += 2) {
    pairs.push({ x: values[i], y: values[i + 1] });
  }
  const out = [];
  for (let i = targetPairIndex; i < pairs.length; i += pairCount) {
    out.push(pairs[i]);
  }
  return out;
}

function computeAxis(points) {
  const xs = points.map((row) => row.x);
  const ys = points.map((row) => row.y);
  const xMin = Math.min(...xs);
  const xMax = Math.max(...xs);
  const yMin = Math.min(...ys);
  const yMax = Math.max(...ys);
  return {
    x_min: xMin,
    x_max: xMax > xMin ? xMax : xMin + 1,
    y_min: yMin,
    y_max: yMax > yMin ? yMax : yMin + 1,
  };
}

function normalizeExpectedShape(value) {
  return String(value ?? "").trim().toLowerCase();
}

async function runCase(root, runMod, row) {
  const lessonPath = path.resolve(root, row.lessonPath);
  assert(fs.existsSync(lessonPath), `lesson_missing:${row.id}:${lessonPath}`);
  const lessonTextRaw = fs.readFileSync(lessonPath, "utf-8");
  const lessonText = preprocessForTeul(root, lessonPath, lessonTextRaw);
  let runLessonPath = lessonPath;
  let cleanup = () => {};
  if (lessonText !== lessonTextRaw) {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "seamgrim-seed-runtime-"));
    runLessonPath = path.join(tmpDir, path.basename(lessonPath));
    fs.writeFileSync(runLessonPath, lessonText, "utf-8");
    cleanup = () => {
      try {
        fs.rmSync(tmpDir, { recursive: true, force: true });
      } catch (_) {
        // ignore cleanup errors
      }
    };
  }

  const defaultObs = readDefaultObservationKey(lessonTextRaw);
  assert(defaultObs, `default_observation_missing:${row.id}`);

  const proc = runTeulCli(root, runLessonPath, 320);
  cleanup();
  if (proc.status !== 0) {
    const detail = String(proc.stderr || proc.stdout || `returncode=${proc.status}`).replace(/\uFFFD/g, "?").trim();
    throw new Error(`teul_run_failed:${row.id}:${detail}`);
  }
  const outputLines = splitOutputLines(proc.stdout);
  const numbers = parseNumericLines(proc.stdout);
  let seriesPoints = extractSeriesPointsFromOutput(outputLines, defaultObs);
  if (!seriesPoints.length) {
    const showVars = extractTickBlockShowVars(lessonText);
    assert(showVars.length >= 2, `tick_show_vars_too_few:${row.id}:${showVars.length}`);
    assert(showVars.length % 2 === 0, `tick_show_vars_not_even:${row.id}:${showVars.length}`);

    const pairs = [];
    for (let i = 0; i < showVars.length; i += 2) {
      pairs.push({ x: showVars[i], y: showVars[i + 1] });
    }
    const targetPairIndex = pairs.findIndex((pair) => pair.y === defaultObs);
    assert(targetPairIndex >= 0, `default_observation_not_in_tick_pairs:${row.id}:${defaultObs}`);

    const minNumbers = Math.max(120, pairs.length * 40);
    assert(numbers.length >= minNumbers, `numbers_too_few:${row.id}:${numbers.length}`);
    seriesPoints = buildSeriesPoints(numbers, pairs.length, targetPairIndex);
    const minSeriesPoints = Math.max(20, Math.floor(minNumbers / Math.max(1, pairs.length * 2)));
    assert(seriesPoints.length >= minSeriesPoints, `series_points_too_few:${row.id}:${seriesPoints.length}`);
  } else {
    assert(seriesPoints.length >= 20, `series_points_too_few:${row.id}:${seriesPoints.length}`);
  }

  const graph = {
    axis: computeAxis(seriesPoints),
    series: [{ id: defaultObs, points: seriesPoints }],
  };
  const observation = {
    channels: [{ key: defaultObs }],
    values: { [defaultObs]: seriesPoints[seriesPoints.length - 1]?.y ?? 0 },
  };
  const length = readNumericAssignment(lessonText, "L");
  if (length !== null) {
    observation.channels.push({ key: "L" });
    observation.values.L = length;
  }

  const result = runMod.synthesizeSpace2dFromGraph(graph, observation);
  assert(result && Array.isArray(result.shapes) && result.shapes.length > 0, `shape_fallback_missing:${row.id}`);
  const nativeSpace2d = detectNativeSpace2d(outputLines);
  const mode = nativeSpace2d ? "native" : "fallback";

  const expected = normalizeExpectedShape(row.expectedShape);
  const fallbackTitle = String(result.meta?.title ?? "").trim();
  if (expected === "pendulum") {
    assert(fallbackTitle === "pendulum-graph-fallback", `shape_title_mismatch:${row.id}:${fallbackTitle}`);
  } else {
    assert(fallbackTitle === "graph-point-fallback", `shape_title_mismatch:${row.id}:${fallbackTitle}`);
  }
  const source = nativeSpace2d ? "native-space2d" : fallbackTitle;

  return {
    id: row.id,
    points: seriesPoints.length,
    y: defaultObs,
    mode,
    source,
    fallback_title: fallbackTitle,
  };
}

async function main() {
  const root = process.cwd();
  const runPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const runMod = await import(pathToFileURL(runPath).href);
  assert(typeof runMod.synthesizeSpace2dFromGraph === "function", "missing_export:synthesizeSpace2dFromGraph");

  const cases = [
    {
      id: "physics_pendulum_seed_v1",
      lessonPath: "solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1/lesson.ddn",
      expectedShape: "pendulum",
    },
    {
      id: "econ_supply_demand_seed_v1",
      lessonPath: "solutions/seamgrim_ui_mvp/seed_lessons_v1/econ_supply_demand_seed_v1/lesson.ddn",
      expectedShape: "point",
    },
    {
      id: "econ_inventory_price_feedback_seed_v1",
      lessonPath: "solutions/seamgrim_ui_mvp/seed_lessons_v1/econ_inventory_price_feedback_seed_v1/lesson.ddn",
      expectedShape: "point",
    },
    {
      id: "bio_sir_transition_seed_v1",
      lessonPath: "solutions/seamgrim_ui_mvp/seed_lessons_v1/bio_sir_transition_seed_v1/lesson.ddn",
      expectedShape: "point",
    },
    {
      id: "math_quadratic_integral_seed_v1",
      lessonPath: "solutions/seamgrim_ui_mvp/seed_lessons_v1/math_quadratic_integral_seed_v1/lesson.ddn",
      expectedShape: "point",
    },
  ];

  const results = [];
  for (const row of cases) {
    const hit = await runCase(root, runMod, row);
    results.push(hit);
  }

  const compact = results.map((row) => `${row.id}:${row.y}:${row.points}:${row.mode}:${row.source}`).join(", ");
  console.log(`seamgrim seed runtime visual pack runner ok cases=${results.length} detail=${compact}`);
}

main().catch((err) => {
  console.error(String(err?.stack || err));
  process.exit(1);
});
