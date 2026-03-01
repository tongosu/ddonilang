const PRAGMA_RE = /^\s*#([^\s(]+)\s*(?:\((.*)\))?\s*$/;

const KNOWN_KINDS = new Set([
  "그래프",
  "점",
  "조종",
  "가져오기",
  "내보내기",
]);
const SHOW_OBJECT_PARTICLE_RE = /^(\s*)(.+?)\s*[을를]\s+보여주기\.\s*(\/\/.*)?$/;
const SHOW_STATEMENT_RE = /^(\s*)(.+?)\s+보여주기\.\s*(\/\/.*)?$/;
const MOVEMENT_SEED_OPEN_RE = /^(\s*).+:\s*움직씨\s*=\s*\{\s*$/;
const SHOW_OUTPUT_LINES_TAG = "보개_출력_줄들";
const LEGACY_ON_START_RE = /^(\s*)\(시작\)할때\s*\{\s*(?:\/\/.*)?$/;
const LEGACY_ON_TICK_RE = /^(\s*)\(매마디\)마다\s*\{\s*(?:\/\/.*)?$/;
const MOVEMENT_SEED_ANY_RE = /:\s*움직씨\s*=\s*\{/;
const BOGAE_BLOCK_OPEN_RE = /^(\s*)(보개|모양)\s*:?\s*\{\s*(?:\/\/.*)?$/;
const SHAPE_LINE_RE = /^(점|선|원)\s*\((.*)\)\s*\.\s*$/;
const BOGAE_MADANG_BLOCK_OPEN_RE = /^(\s*)보개마당\s*:?\s*\{\s*(?:\/\/.*)?$/;
const MADANG_CHUNK_OPEN_RE = /^(\s*)토막\s*\((.*)\)\s*\{\s*(?:\/\/.*)?$/;
const MADANG_SUBTITLE_LINE_RE = /^(\s*)자막\s*\((.*)\)\s*\.\s*$/;
const LEGACY_BOIM_OPEN_RE = /^(\s*)보임\s*\{\s*$/;
const LEGACY_BOIM_ITEM_RE = /^[^:]+:\s*(.+)\.\s*$/;
const LEGACY_START_ONCE_STATE_KEY = "__wasm_start_once";

function normalizeText(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function splitArgs(text) {
  const src = String(text ?? "").trim();
  if (!src) return [];
  const out = [];
  let start = 0;
  let depth = 0;
  let quote = null;
  for (let i = 0; i < src.length; i += 1) {
    const ch = src[i];
    if (quote) {
      if (ch === quote && src[i - 1] !== "\\") {
        quote = null;
      }
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      continue;
    }
    if (ch === "(" || ch === "[" || ch === "{") {
      depth += 1;
      continue;
    }
    if (ch === ")" || ch === "]" || ch === "}") {
      depth = Math.max(0, depth - 1);
      continue;
    }
    if (ch === "," && depth === 0) {
      out.push(src.slice(start, i).trim());
      start = i + 1;
    }
  }
  out.push(src.slice(start).trim());
  return out.filter(Boolean);
}

function parseArgs(argText) {
  const args = {};
  const list = splitArgs(argText);
  list.forEach((entry, idx) => {
    const eq = entry.indexOf("=");
    if (eq < 0) {
      args[`arg${idx + 1}`] = entry;
      return;
    }
    const key = entry.slice(0, eq).trim();
    const value = entry.slice(eq + 1).trim();
    if (!key) {
      args[`arg${idx + 1}`] = value;
      return;
    }
    args[key] = value;
  });
  return args;
}

export function preprocessDdnText(ddnText) {
  const lines = normalizeText(ddnText).split("\n");
  const bodyLines = [];
  const pragmas = [];
  const diags = [];

  lines.forEach((line, idx) => {
    const line1 = idx + 1;
    if (!/^\s*#/.test(line)) {
      bodyLines.push(line);
      return;
    }
    const match = line.match(PRAGMA_RE);
    if (!match) {
      pragmas.push({
        kind: "기타",
        raw: line,
        args: {},
        loc: { line1, col1: 1 },
      });
      diags.push({
        level: "WARN",
        code: "W_PRAGMA_PARSE_FAIL",
        message: "pragma 구문을 해석하지 못했습니다. raw를 보존합니다.",
        where: `line:${line1}`,
        details: line,
      });
      return;
    }
    const rawKind = String(match[1] ?? "").trim();
    const kind = KNOWN_KINDS.has(rawKind) ? rawKind : "기타";
    const args = parseArgs(match[2] ?? "");
    pragmas.push({
      kind,
      raw: line,
      args,
      loc: { line1, col1: 1 },
    });
    if (kind === "기타") {
      diags.push({
        level: "WARN",
        code: "W_PRAGMA_UNKNOWN_KIND",
        message: `알 수 없는 pragma kind: ${rawKind}`,
        where: `line:${line1}`,
        details: line,
      });
    }
  });

  const normalizedShow = rewriteShowObjectParticles(bodyLines.join("\n"));
  const normalizedMadang = rewriteBogaeMadangBlocksForWasm(normalizedShow);
  const normalizedLifecycle = rewriteLegacyLifecycleBlocksForWasm(normalizedMadang);
  const normalizedBogae = rewriteBogaeShapeBlocksForWasm(normalizedLifecycle);
  const normalizedBoim = rewriteLegacyBoimBlocksForWasm(normalizedBogae);
  const bodyText = rewriteShowStatementsForWasm(normalizedBoim);

  return {
    bodyText,
    pragmas,
    diags,
  };
}

function rewriteLegacyBoimBlocksForWasm(bodyText) {
  const lines = normalizeText(bodyText).split("\n");
  const out = [];
  let idx = 0;

  while (idx < lines.length) {
    const line = lines[idx];
    const open = line.match(LEGACY_BOIM_OPEN_RE);
    if (!open) {
      out.push(line);
      idx += 1;
      continue;
    }

    const indent = open[1] ?? "";
    const emits = [];
    let cursor = idx + 1;
    let closed = false;
    let convertible = true;

    while (cursor < lines.length) {
      const raw = String(lines[cursor] ?? "");
      const trimmed = raw.trim();
      if (trimmed === "}." || trimmed === "}") {
        closed = true;
        break;
      }
      if (!trimmed || trimmed.startsWith("//")) {
        cursor += 1;
        continue;
      }
      const row = trimmed.match(LEGACY_BOIM_ITEM_RE);
      if (!row) {
        convertible = false;
        break;
      }
      const expr = String(row[1] ?? "").trim();
      emits.push(`${indent}  ${expr} 보여주기.`);
      cursor += 1;
    }

    if (!closed || !convertible) {
      out.push(line);
      idx += 1;
      continue;
    }

    out.push(...emits);
    idx = cursor + 1;
  }

  return out.join("\n");
}

function countBraceDelta(line) {
  let delta = 0;
  const src = String(line ?? "");
  for (const ch of src) {
    if (ch === "{") delta += 1;
    if (ch === "}") delta -= 1;
  }
  return delta;
}

function findBlockCloseIndex(lines, openIndex) {
  let depth = 0;
  for (let i = openIndex; i < lines.length; i += 1) {
    depth += countBraceDelta(lines[i]);
    if (depth === 0 && i > openIndex) return i;
  }
  return -1;
}

function indentLines(lines, prefix) {
  return lines.map((line) => `${prefix}${String(line ?? "").trimStart()}`);
}

function rewriteLegacyLifecycleBlocksForWasm(bodyText) {
  const lines = normalizeText(bodyText).split("\n");
  if (lines.some((line) => MOVEMENT_SEED_ANY_RE.test(line))) {
    return bodyText;
  }

  const startOpen = lines.findIndex((line) => LEGACY_ON_START_RE.test(line));
  const tickOpen = lines.findIndex((line) => LEGACY_ON_TICK_RE.test(line));
  if (tickOpen < 0) {
    return bodyText;
  }

  const startClose = startOpen >= 0 ? findBlockCloseIndex(lines, startOpen) : -1;
  const tickClose = findBlockCloseIndex(lines, tickOpen);
  if (tickClose < 0 || (startOpen >= 0 && startClose < 0)) {
    return bodyText;
  }

  const startRange = startOpen >= 0 ? [startOpen, startClose] : null;
  const tickRange = [tickOpen, tickClose];
  const isRemoved = (index) => {
    if (startRange && index >= startRange[0] && index <= startRange[1]) return true;
    if (index >= tickRange[0] && index <= tickRange[1]) return true;
    return false;
  };

  const outerLines = lines.filter((_, index) => !isRemoved(index));
  while (outerLines.length && !outerLines[outerLines.length - 1].trim()) {
    outerLines.pop();
  }

  const startBody = startOpen >= 0 ? lines.slice(startOpen + 1, startClose) : [];
  const tickBody = lines.slice(tickOpen + 1, tickClose);

  const out = [];
  out.push(...outerLines);
  if (outerLines.some((line) => String(line ?? "").trim())) {
    out.push("");
  }
  out.push("매틱:움직씨 = {");
  if (startOpen >= 0) {
    out.push(`  { ${LEGACY_START_ONCE_STATE_KEY} <= 0 }인것 일때 {`);
    out.push(...indentLines(startBody, "    "));
    out.push(`    ${LEGACY_START_ONCE_STATE_KEY} <- 1.`);
    out.push("  }.");
  }
  out.push(...indentLines(tickBody, "  "));
  out.push("}.");

  return out.join("\n");
}

function rewriteShowObjectParticles(bodyText) {
  const lines = normalizeText(bodyText).split("\n");
  const out = lines.map((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("//")) return line;
    const match = line.match(SHOW_OBJECT_PARTICLE_RE);
    if (!match) return line;
    const indent = match[1] ?? "";
    const expr = String(match[2] ?? "").trimEnd();
    const comment = String(match[3] ?? "").trim();
    return `${indent}${expr} 보여주기.${comment ? ` ${comment}` : ""}`;
  });
  return out.join("\n");
}

function pickNamedOrPositionalArg(args, names = [], positionalKeys = ["arg1", "arg2", "arg3"]) {
  const namedList = Array.isArray(names) ? names : [names];
  for (const name of namedList) {
    const value = String(args?.[name] ?? "").trim();
    if (value) return value;
  }
  for (const key of positionalKeys) {
    const value = String(args?.[key] ?? "").trim();
    if (value) return value;
  }
  return "";
}

function parsePairTupleExpr(exprText) {
  const text = String(exprText ?? "").trim();
  const match = text.match(/^\(\s*([^,]+)\s*,\s*([^)]+)\s*\)$/);
  if (!match) return null;
  const xExpr = String(match[1] ?? "").trim();
  const yExpr = String(match[2] ?? "").trim();
  if (!xExpr || !yExpr) return null;
  return { xExpr, yExpr };
}

function buildMadangSubtitleEmitLines({
  chunkArgs = {},
  subtitleArgs = {},
  indent = "  ",
} = {}) {
  const textExpr = pickNamedOrPositionalArg(subtitleArgs, ["글", "text", "내용", "문장"]);
  if (!textExpr) return [];
  const chunkIdExpr = pickNamedOrPositionalArg(chunkArgs, ["이름", "name", "id"], ["arg1"]);
  const positionExpr = pickNamedOrPositionalArg(subtitleArgs, ["자리", "pos", "위치"], ["arg2", "arg3"]);
  const position = parsePairTupleExpr(positionExpr);
  const out = [
    `${indent}"text.overlay" 보여주기.`,
  ];
  if (chunkIdExpr) {
    out.push(`${indent}"id" 보여주기.`);
    out.push(`${indent}${chunkIdExpr} 보여주기.`);
  }
  out.push(`${indent}"markdown" 보여주기.`);
  out.push(`${indent}${textExpr} 보여주기.`);
  if (position) {
    out.push(`${indent}"x" 보여주기.`);
    out.push(`${indent}${position.xExpr} 보여주기.`);
    out.push(`${indent}"y" 보여주기.`);
    out.push(`${indent}${position.yExpr} 보여주기.`);
  }
  return out;
}

function convertBogaeMadangBlock(lines, indent = "  ") {
  const out = [];
  let idx = 0;
  while (idx < lines.length) {
    const open = String(lines[idx] ?? "").match(MADANG_CHUNK_OPEN_RE);
    if (!open) {
      idx += 1;
      continue;
    }
    const close = findBlockCloseIndex(lines, idx);
    if (close < 0) return null;
    const chunkArgs = parseArgs(open[2] ?? "");
    const chunkInner = lines.slice(idx + 1, close);
    chunkInner.forEach((rawLine) => {
      const line = stripLineComment(rawLine).trim();
      if (!line) return;
      const subtitle = line.match(MADANG_SUBTITLE_LINE_RE);
      if (!subtitle) return;
      const subtitleArgs = parseArgs(subtitle[2] ?? "");
      out.push(...buildMadangSubtitleEmitLines({ chunkArgs, subtitleArgs, indent }));
    });
    idx = close + 1;
  }
  return out.length ? out : null;
}

function rewriteBogaeMadangBlocksForWasm(bodyText) {
  const lines = normalizeText(bodyText).split("\n");
  const stripped = [];
  const emitLines = [];

  for (let i = 0; i < lines.length; i += 1) {
    const open = lines[i].match(BOGAE_MADANG_BLOCK_OPEN_RE);
    if (!open) {
      stripped.push(lines[i]);
      continue;
    }
    const close = findBlockCloseIndex(lines, i);
    if (close < 0) {
      stripped.push(lines[i]);
      continue;
    }
    const inner = lines.slice(i + 1, close);
    const converted = convertBogaeMadangBlock(inner, "  ");
    if (!converted) {
      stripped.push(lines[i]);
      continue;
    }
    emitLines.push(...converted);
    i = close;
  }

  if (!emitLines.length) return stripped.join("\n");

  const injected = [];
  let inserted = false;
  stripped.forEach((line) => {
    injected.push(line);
    if (inserted) return;
    const movementOpen = String(line ?? "").match(MOVEMENT_SEED_OPEN_RE);
    if (!movementOpen) return;
    const baseIndent = movementOpen[1] ?? "";
    emitLines.forEach((emit) => injected.push(`${baseIndent}${emit}`));
    inserted = true;
  });
  if (inserted) return injected.join("\n");

  const fallback = [...stripped];
  if (fallback.length && String(fallback[fallback.length - 1] ?? "").trim()) {
    fallback.push("");
  }
  fallback.push("(매마디)마다 {");
  emitLines.forEach((emit) => {
    fallback.push(`  ${String(emit ?? "").trimStart()}`);
  });
  fallback.push("}.");
  return fallback.join("\n");
}

function stripLineComment(line) {
  const src = String(line ?? "");
  let quote = null;
  for (let i = 0; i < src.length - 1; i += 1) {
    const ch = src[i];
    if (quote) {
      if (ch === quote && src[i - 1] !== "\\") {
        quote = null;
      }
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      continue;
    }
    if (ch === "/" && src[i + 1] === "/") {
      return src.slice(0, i);
    }
  }
  return src;
}

function parseShapeCallArgs(argText) {
  const positionals = [];
  const named = {};
  splitArgs(argText).forEach((token) => {
    const eq = token.indexOf("=");
    if (eq < 0) {
      positionals.push(token.trim());
      return;
    }
    const key = token.slice(0, eq).trim();
    const value = token.slice(eq + 1).trim();
    if (!key) {
      positionals.push(value);
      return;
    }
    named[key] = value;
  });
  return { positionals, named };
}

function pickShapeArg(args, keys = [], fallback = "") {
  const list = Array.isArray(keys) ? keys : [keys];
  for (const key of list) {
    if (typeof key === "number") {
      const value = args.positionals[key];
      if (String(value ?? "").trim()) return String(value).trim();
      continue;
    }
    const value = args.named?.[key];
    if (String(value ?? "").trim()) return String(value).trim();
  }
  return String(fallback ?? "").trim();
}

function buildPointShapeLines(args, indent) {
  const x = pickShapeArg(args, ["x", "cx", 0], "0");
  const y = pickShapeArg(args, ["y", "cy", 1], "0");
  const size = pickShapeArg(args, ["크기", "size", "r", 2], "0.05");
  const color = pickShapeArg(args, ["색", "color"], '"#22c55e"');
  return [
    `${indent}"space2d.shape" 보여주기.`,
    `${indent}"point" 보여주기.`,
    `${indent}"x" 보여주기.`,
    `${indent}${x} 보여주기.`,
    `${indent}"y" 보여주기.`,
    `${indent}${y} 보여주기.`,
    `${indent}"size" 보여주기.`,
    `${indent}${size} 보여주기.`,
    `${indent}"color" 보여주기.`,
    `${indent}${color} 보여주기.`,
  ];
}

function buildLineShapeLines(args, indent) {
  const x1 = pickShapeArg(args, ["x1", 0], "0");
  const y1 = pickShapeArg(args, ["y1", 1], "0");
  const x2 = pickShapeArg(args, ["x2", 2], "0");
  const y2 = pickShapeArg(args, ["y2", 3], "0");
  const stroke = pickShapeArg(args, ["색", "stroke"], '"#9ca3af"');
  const width = pickShapeArg(args, ["굵기", "width"], "0.02");
  return [
    `${indent}"space2d.shape" 보여주기.`,
    `${indent}"line" 보여주기.`,
    `${indent}"x1" 보여주기.`,
    `${indent}${x1} 보여주기.`,
    `${indent}"y1" 보여주기.`,
    `${indent}${y1} 보여주기.`,
    `${indent}"x2" 보여주기.`,
    `${indent}${x2} 보여주기.`,
    `${indent}"y2" 보여주기.`,
    `${indent}${y2} 보여주기.`,
    `${indent}"stroke" 보여주기.`,
    `${indent}${stroke} 보여주기.`,
    `${indent}"width" 보여주기.`,
    `${indent}${width} 보여주기.`,
  ];
}

function buildCircleShapeLines(args, indent) {
  const x = pickShapeArg(args, ["x", "cx", 0], "0");
  const y = pickShapeArg(args, ["y", "cy", 1], "0");
  const r = pickShapeArg(args, ["r", "반지름", 2], "0.08");
  const fill = pickShapeArg(args, ["색", "fill"], '"#38bdf8"');
  const stroke = pickShapeArg(args, ["선색", "stroke"], '"#0ea5e9"');
  const width = pickShapeArg(args, ["굵기", "width"], "0.02");
  return [
    `${indent}"space2d.shape" 보여주기.`,
    `${indent}"circle" 보여주기.`,
    `${indent}"x" 보여주기.`,
    `${indent}${x} 보여주기.`,
    `${indent}"y" 보여주기.`,
    `${indent}${y} 보여주기.`,
    `${indent}"r" 보여주기.`,
    `${indent}${r} 보여주기.`,
    `${indent}"fill" 보여주기.`,
    `${indent}${fill} 보여주기.`,
    `${indent}"stroke" 보여주기.`,
    `${indent}${stroke} 보여주기.`,
    `${indent}"width" 보여주기.`,
    `${indent}${width} 보여주기.`,
  ];
}

function convertBogaeShapeBlock(lines, indent) {
  const primitives = [];
  for (const rawLine of lines) {
    const line = stripLineComment(rawLine).trim();
    if (!line) continue;
    const match = line.match(SHAPE_LINE_RE);
    if (!match) return null;
    primitives.push({
      kind: String(match[1] ?? "").trim(),
      args: parseShapeCallArgs(match[2] ?? ""),
    });
  }
  if (!primitives.length) return null;

  const out = [`${indent}"space2d" 보여주기.`];
  primitives.forEach((primitive) => {
    if (primitive.kind === "점") {
      out.push(...buildPointShapeLines(primitive.args, indent));
      return;
    }
    if (primitive.kind === "선") {
      out.push(...buildLineShapeLines(primitive.args, indent));
      return;
    }
    if (primitive.kind === "원") {
      out.push(...buildCircleShapeLines(primitive.args, indent));
    }
  });
  return out;
}

function rewriteBogaeShapeBlocksForWasm(bodyText) {
  const lines = normalizeText(bodyText).split("\n");
  const out = [];
  for (let i = 0; i < lines.length; i += 1) {
    const open = lines[i].match(BOGAE_BLOCK_OPEN_RE);
    if (!open) {
      out.push(lines[i]);
      continue;
    }
    const close = findBlockCloseIndex(lines, i);
    if (close < 0) {
      out.push(lines[i]);
      continue;
    }
    const inner = lines.slice(i + 1, close);
    const indent = `${open[1] ?? ""}  `;
    const converted = convertBogaeShapeBlock(inner, indent);
    if (!converted) {
      out.push(lines[i]);
      continue;
    }
    out.push(...converted);
    i = close;
  }
  return out.join("\n");
}

function rewriteShowStatementsForWasm(bodyText) {
  const lines = normalizeText(bodyText).split("\n");
  let hasShow = false;
  const rewritten = lines.map((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("//")) return line;
    const match = line.match(SHOW_STATEMENT_RE);
    if (!match) return line;
    hasShow = true;
    const indent = match[1] ?? "";
    const expr = String(match[2] ?? "").trimEnd();
    const comment = String(match[3] ?? "").trim();
    return `${indent}${SHOW_OUTPUT_LINES_TAG} <- (${SHOW_OUTPUT_LINES_TAG}, (${expr}) 글로) 추가.${comment ? ` ${comment}` : ""}`;
  });
  if (!hasShow) return rewritten.join("\n");

  let insertedReset = false;
  const withReset = [];
  rewritten.forEach((line) => {
    withReset.push(line);
    if (insertedReset) return;
    const movementOpen = line.match(MOVEMENT_SEED_OPEN_RE);
    if (movementOpen) {
      const indent = movementOpen[1] ?? "";
      withReset.push(`${indent}  ${SHOW_OUTPUT_LINES_TAG} <- () 차림.`);
      insertedReset = true;
      return;
    }
    const tickOpen = line.match(LEGACY_ON_TICK_RE);
    if (tickOpen) {
      const indent = tickOpen[1] ?? "";
      withReset.push(`${indent}  ${SHOW_OUTPUT_LINES_TAG} <- () 차림.`);
      insertedReset = true;
    }
  });
  if (insertedReset) return withReset.join("\n");

  const wrapped = [];
  wrapped.push("매틱:움직씨 = {");
  wrapped.push(`  ${SHOW_OUTPUT_LINES_TAG} <- () 차림.`);
  rewritten.forEach((line) => {
    if (!line.trim()) {
      wrapped.push("");
      return;
    }
    wrapped.push(`  ${line}`);
  });
  wrapped.push("}.");
  return wrapped.join("\n");
}
