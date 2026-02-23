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
const LEGACY_ON_START_RE = /^\s*\(시작\)할때\s*\{\s*$/;
const LEGACY_ON_TICK_RE = /^\s*\(매마디\)마다\s*\{\s*$/;
const MOVEMENT_SEED_ANY_RE = /:\s*움직씨\s*=\s*\{/;

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
  const normalizedLifecycle = rewriteLegacyLifecycleBlocksForWasm(normalizedShow);
  const bodyText = rewriteShowStatementsForWasm(normalizedLifecycle);

  return {
    bodyText,
    pragmas,
    diags,
  };
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
  out.push("매틱:움직씨 = {");
  if (startOpen >= 0) {
    out.push(...indentLines(outerLines, "    "));
    out.push(...indentLines(startBody, "    "));
  } else {
    out.push(...indentLines(outerLines, "  "));
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
    const open = line.match(MOVEMENT_SEED_OPEN_RE);
    if (!open) return;
    const indent = open[1] ?? "";
    withReset.push(`${indent}  ${SHOW_OUTPUT_LINES_TAG} <- () 차림.`);
    insertedReset = true;
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
