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
    const proc = spawnSync(directBin, ["run", lessonPath, "--madi", String(madi)], {
      cwd: root,
      encoding: "utf-8",
    });
    return proc;
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
    const num = Number(line);
    if (Number.isFinite(num)) out.push(num);
  }
  return out;
}

function splitOutputLines(stdout) {
  return String(stdout ?? "")
    .split(/\r?\n/u)
    .map((line) => String(line ?? "").trim())
    .filter(Boolean);
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

function parseLengthFromLesson(lessonText) {
  const patterns = [
    /^\s*L\s*:\s*[^<>=\n]+<-\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?)\s*\./imu,
    /^\s*L\s*<-\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?)\s*\./imu,
  ];
  for (const pattern of patterns) {
    const hit = String(lessonText ?? "").match(pattern);
    if (!hit) continue;
    const value = Number(hit[1]);
    if (Number.isFinite(value)) return value;
  }
  return 1;
}

function toThetaPairs(numbers) {
  const values = [...numbers];
  if (values.length % 2 !== 0) values.pop();
  const pairs = [];
  for (let i = 0; i < values.length; i += 2) {
    pairs.push({ x: values[i], y: values[i + 1] });
  }
  const theta = [];
  for (let i = 0; i < pairs.length; i += 3) {
    theta.push(pairs[i]);
  }
  return theta;
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

async function main() {
  const root = process.cwd();
  const lessonPath = path.resolve(
    root,
    "solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1/lesson.ddn",
  );
  assert(fs.existsSync(lessonPath), `lesson missing: ${lessonPath}`);

  const lessonText = fs.readFileSync(lessonPath, "utf-8");
  const length = parseLengthFromLesson(lessonText);
  let runLessonPath = lessonPath;
  let cleanup = () => {};
  const normalized = preprocessForTeul(root, lessonPath, lessonText);
  if (normalized !== lessonText) {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "seamgrim-pendulum-"));
    runLessonPath = path.join(tmpDir, path.basename(lessonPath));
    fs.writeFileSync(runLessonPath, normalized, "utf-8");
    cleanup = () => {
      try {
        fs.rmSync(tmpDir, { recursive: true, force: true });
      } catch (_) {
        // ignore cleanup errors
      }
    };
  }

  const proc = runTeulCli(root, runLessonPath, 420);
  cleanup();
  if (proc.status !== 0) {
    const detail = String(proc.stderr || proc.stdout || `returncode=${proc.status}`).trim();
    throw new Error(`teul_run_failed:${detail}`);
  }

  const outputLines = splitOutputLines(proc.stdout);
  let thetaPoints = extractSeriesPointsFromOutput(outputLines, "theta");
  if (!thetaPoints.length) {
    const numbers = parseNumericLines(proc.stdout);
    assert(numbers.length >= 1200, `numbers_too_few:${numbers.length}`);
    thetaPoints = toThetaPairs(numbers);
  }
  assert(thetaPoints.length >= 200, `theta_points_too_few:${thetaPoints.length}`);

  const runPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const runMod = await import(pathToFileURL(runPath).href);
  const { synthesizeSpace2dFromGraph, synthesizeSpace2dFromObservation } = runMod;
  assert(typeof synthesizeSpace2dFromGraph === "function", "missing_export:synthesizeSpace2dFromGraph");
  assert(typeof synthesizeSpace2dFromObservation === "function", "missing_export:synthesizeSpace2dFromObservation");

  const graph = {
    axis: computeAxis(thetaPoints),
    series: [{ id: "theta", points: thetaPoints }],
  };
  const last = thetaPoints[thetaPoints.length - 1];
  const observation = {
    channels: [{ key: "theta" }, { key: "L" }],
    row: [last.y, length],
    values: { theta: last.y, L: length },
  };

  const fromGraph = synthesizeSpace2dFromGraph(graph, observation);
  assert(fromGraph && Array.isArray(fromGraph.shapes), "graph_fallback_shapes_missing");
  assert(fromGraph.meta?.title === "pendulum-graph-fallback", "graph_fallback_title_mismatch");

  const fromObservation = synthesizeSpace2dFromObservation(observation);
  assert(fromObservation && Array.isArray(fromObservation.shapes), "observation_fallback_shapes_missing");

  console.log(
    `seamgrim pendulum runtime visual runner ok theta_points=${thetaPoints.length} length=${length.toFixed(3)}`,
  );
}

main().catch((err) => {
  console.error(String(err?.stack || err));
  process.exit(1);
});
