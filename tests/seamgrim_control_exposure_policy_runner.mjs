import fs from "fs";
import path from "path";
import { pathToFileURL } from "url";

function isVariableType(type) {
  const normalized = String(type ?? "").trim().toLowerCase();
  if (!normalized) return false;
  const keywords = ["변수", "var", "variable", "관찰", "observable", "obs", "axis"];
  return keywords.some((token) => normalized.includes(token));
}

function isConstantLikeType(type) {
  const normalized = String(type ?? "").trim().toLowerCase();
  if (!normalized) return true;
  if (isVariableType(normalized)) return false;
  const numericTypes = ["수", "number", "num", "실수", "정수", "float", "int"];
  if (numericTypes.some((token) => normalized.includes(token))) return true;
  const constTypes = ["상수", "const", "constant", "parameter", "param", "조절", "control", "고정"];
  return constTypes.some((token) => normalized.includes(token));
}

function countChar(text, needle) {
  const src = String(text ?? "");
  const ch = String(needle ?? "");
  if (!ch) return 0;
  let count = 0;
  for (let i = 0; i < src.length; i += 1) {
    if (src[i] === ch) count += 1;
  }
  return count;
}

function isNumericLiteral(text) {
  return /^\s*[+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?\s*$/iu.test(String(text ?? ""));
}

function extractDeclarationsByBlock(text) {
  const lines = String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  const blockStartPattern = /^\s*(그릇채비|붙박이마련|붙박이채비|채비|씨앗)\s*:?\s*\{/u;
  const assignPattern =
    /^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*(?:<-|=)\s*([^\.]+)\.?\s*$/u;

  let inBlock = false;
  let currentBlock = "";
  let depth = 0;

  const out = {
    chabiVars: new Set(),
    chabiNumericConsts: new Set(),
    seedVars: new Set(),
    seedNumericConsts: new Set(),
  };

  for (const rawLine of lines) {
    const line = String(rawLine ?? "");
    if (!inBlock) {
      const start = line.match(blockStartPattern);
      if (start) {
        inBlock = true;
        currentBlock = String(start[1] ?? "").trim();
        depth = Math.max(1, countChar(line, "{") - countChar(line, "}") || 1);
      }
      continue;
    }

    const lineNoComment = line.split("//")[0];
    const match = lineNoComment.match(assignPattern);
    if (match) {
      const name = String(match[1] ?? "").trim();
      const rawType = String(match[2] ?? "수").trim() || "수";
      const rhs = String(match[3] ?? "").trim();
      const variable = isVariableType(rawType);
      const numericConst = isConstantLikeType(rawType) && isNumericLiteral(rhs);

      const inChabiSurface = currentBlock === "채비" || currentBlock === "그릇채비" || currentBlock === "붙박이마련" || currentBlock === "붙박이채비";
      if (inChabiSurface) {
        if (variable) out.chabiVars.add(name);
        if (numericConst) out.chabiNumericConsts.add(name);
      }
      if (currentBlock === "씨앗") {
        if (variable) out.seedVars.add(name);
        if (numericConst) out.seedNumericConsts.add(name);
      }
    }

    depth += countChar(line, "{");
    depth -= countChar(line, "}");
    if (depth <= 0) {
      inBlock = false;
      currentBlock = "";
      depth = 0;
    }
  }

  return out;
}

function collectTargetFiles(root) {
  const bases = [
    path.resolve(root, "solutions/seamgrim_ui_mvp/seed_lessons_v1"),
    path.resolve(root, "solutions/seamgrim_ui_mvp/lessons"),
    path.resolve(root, "solutions/seamgrim_ui_mvp/lessons_rewrite_v1"),
  ];
  const out = [];
  for (const base of bases) {
    if (!fs.existsSync(base)) continue;
    const stack = [base];
    while (stack.length > 0) {
      const cur = stack.pop();
      const entries = fs.readdirSync(cur, { withFileTypes: true });
      for (const entry of entries) {
        const full = path.join(cur, entry.name);
        if (entry.isDirectory()) {
          stack.push(full);
          continue;
        }
        if (entry.isFile() && entry.name.toLowerCase().endsWith(".ddn")) {
          out.push(full);
        }
      }
    }
  }
  out.sort();
  return out;
}

async function main() {
  const root = process.cwd();
  const parserPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/control_parser.js");
  const { buildControlSpecsFromDdn } = await import(pathToFileURL(parserPath).href);

  const files = collectTargetFiles(root);
  const violations = [];

  for (const file of files) {
    const text = fs.readFileSync(file, "utf8");
    const controls = buildControlSpecsFromDdn(text);
    const axis = new Set(Array.isArray(controls?.axisKeys) ? controls.axisKeys.map((x) => String(x ?? "").trim()) : []);
    const sliders = new Set(Array.isArray(controls?.specs) ? controls.specs.map((s) => String(s?.name ?? "").trim()) : []);
    const decl = extractDeclarationsByBlock(text);

    for (const name of decl.chabiVars) {
      if (!axis.has(name)) {
        violations.push({ kind: "chabi_variable_not_exposed", file, name });
      }
      if (sliders.has(name)) {
        violations.push({ kind: "chabi_variable_exposed_as_slider", file, name });
      }
    }
    for (const name of decl.seedVars) {
      if (axis.has(name)) {
        violations.push({ kind: "seed_variable_exposed", file, name });
      }
      if (sliders.has(name)) {
        violations.push({ kind: "seed_variable_exposed_as_slider", file, name });
      }
    }
    for (const name of decl.chabiNumericConsts) {
      if (!sliders.has(name)) {
        violations.push({ kind: "chabi_constant_not_slider", file, name });
      }
      if (axis.has(name)) {
        violations.push({ kind: "chabi_constant_exposed_as_axis", file, name });
      }
    }
    for (const name of decl.seedNumericConsts) {
      if (sliders.has(name)) {
        violations.push({ kind: "seed_constant_exposed_as_slider", file, name });
      }
    }
  }

  if (violations.length > 0) {
    const top = violations.slice(0, 10).map((row) => {
      const rel = path.relative(root, row.file).replace(/\\/g, "/");
      return `${row.kind}:${rel}:${row.name}`;
    });
    const extra = violations.length > 10 ? ` ... (${violations.length - 10} more)` : "";
    console.log(`check=control_exposure_policy_violation detail=${top.join(", ")}${extra}`);
    console.log("seamgrim control exposure policy check failed");
    process.exit(1);
    return;
  }

  console.log(`seamgrim control exposure policy check ok files=${files.length}`);
}

main().catch((err) => {
  console.error(String(err?.stack || err));
  process.exit(1);
});
