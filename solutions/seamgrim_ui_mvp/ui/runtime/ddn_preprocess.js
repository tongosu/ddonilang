const PRAGMA_RE = /^\s*#([^\s(]+)\s*(?:\((.*)\))?\s*$/;

const KNOWN_KINDS = new Set([
  "그래프",
  "점",
  "조종",
  "가져오기",
  "내보내기",
]);

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

  return {
    bodyText: bodyLines.join("\n"),
    pragmas,
    diags,
  };
}

