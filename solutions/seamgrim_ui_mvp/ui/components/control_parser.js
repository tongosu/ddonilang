function normalizeNewlines(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function countCharInText(text, needle) {
  if (!text) return 0;
  return (String(text).match(new RegExp(`\\${needle}`, "g")) || []).length;
}

function splitMetaHeader(text) {
  const lines = normalizeNewlines(text).split("\n");
  const meta = {};
  let idx = 0;
  while (idx < lines.length) {
    const raw = lines[idx];
    const trimmed = raw.replace(/^[ \t\uFEFF]+/, "");
    if (!trimmed) {
      idx += 1;
      continue;
    }
    if (trimmed.startsWith("#") && trimmed.includes(":")) {
      const sliced = trimmed.slice(1);
      const [keyRaw, ...rest] = sliced.split(":");
      const key = keyRaw.trim();
      if (!key) break;
      meta[key] = rest.join(":").trim();
      idx += 1;
      continue;
    }
    break;
  }
  return { meta, body: lines.slice(idx).join("\n") };
}

function findDefaultObservationMeta(meta) {
  if (!meta || typeof meta !== "object") return "";
  return (
    meta["기본관찰"] ??
    meta["기본관측"] ??
    meta["default_obs"] ??
    meta["default-observation"] ??
    meta["default_observation"] ??
    meta["default_y"] ??
    meta["default-y"] ??
    ""
  );
}

function findDefaultObservationXMeta(meta) {
  if (!meta || typeof meta !== "object") return "";
  return (
    meta["기본관찰x"] ??
    meta["기본관측x"] ??
    meta["기본x축"] ??
    meta["기본축x"] ??
    meta["default_obs_x"] ??
    meta["default-observation-x"] ??
    meta["default_observation_x"] ??
    meta["default_x"] ??
    meta["default-x"] ??
    meta["default_x_axis"] ??
    meta["default-x-axis"] ??
    ""
  );
}

function pickFirstIdentifier(text) {
  const match = String(text ?? "").match(/([A-Za-z0-9_가-힣]+)/u);
  return String(match?.[1] ?? "").trim();
}

function hasDefaultObservationHint(text) {
  if (!text) return false;
  return /(기본관찰(?!x)|기본관측(?!x)|default[_-]?obs(?![_-]?x)\b|default[_-]?observation(?![_-]?x)\b|default[_-]?y\b)/iu.test(String(text));
}

function hasDefaultObservationXHint(text) {
  if (!text) return false;
  return /(기본관찰x|기본관측x|기본x축|기본축x|default[_-]?obs[_-]?x|default[_-]?observation[_-]?x|default[_-]?x(?:[_-]?axis)?)/iu.test(String(text));
}

function inferStepFromValue(value) {
  const abs = Math.abs(Number(value));
  if (!Number.isFinite(abs)) return 0.1;
  if (abs >= 20) return 1;
  if (abs >= 5) return 0.5;
  if (abs >= 1) return 0.1;
  if (abs >= 0.2) return 0.05;
  if (abs >= 0.05) return 0.01;
  return 0.005;
}

function inferBounds(value) {
  const numeric = Number(value);
  const span = Math.max(Math.abs(numeric), 1);
  return {
    min: Number((numeric - span).toFixed(6)),
    max: Number((numeric + span).toFixed(6)),
    step: inferStepFromValue(numeric),
  };
}

function normalizeType(type) {
  return String(type ?? "")
    .trim()
    .toLowerCase();
}

function isVariableType(type) {
  const normalized = normalizeType(type);
  if (!normalized) return false;
  const keywords = ["변수", "var", "variable", "관찰", "observable", "obs", "axis"];
  return keywords.some((token) => normalized.includes(token));
}

function isConstantLikeType(type) {
  const normalized = normalizeType(type);
  if (!normalized) return true;
  if (isVariableType(normalized)) return false;
  const numericTypes = ["수", "number", "num", "실수", "정수", "float", "int"];
  if (numericTypes.some((token) => normalized.includes(token))) return true;
  const constTypes = ["상수", "const", "constant", "parameter", "param", "조절", "control", "고정"];
  return constTypes.some((token) => normalized.includes(token));
}

function parseRangeAnnotation(text) {
  const match = String(text ?? "").match(/범위\s*\(\s*([+-]?\d+(?:\.\d+)?)\s*,\s*([+-]?\d+(?:\.\d+)?)(?:\s*,\s*([+-]?\d+(?:\.\d+)?))?\s*\)/u);
  if (!match) return null;
  const min = Number(match[1]);
  const max = Number(match[2]);
  const step = Number(match[3]);
  if (!Number.isFinite(min) || !Number.isFinite(max)) return null;
  return {
    min: Math.min(min, max),
    max: Math.max(min, max),
    step: Number.isFinite(step) && step > 0 ? step : null,
  };
}

function splitLineBodyAndComment(line) {
  const text = String(line ?? "");
  const idx = text.indexOf("//");
  if (idx < 0) {
    return { body: text, tail: "" };
  }
  return {
    body: text.slice(0, idx),
    tail: text.slice(idx + 2).trim(),
  };
}

function parsePrepAssignmentLine(line) {
  const match = String(line ?? "").match(
    /^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*(?:<-|=)\s*(.+)$/u,
  );
  if (!match) return null;
  const name = String(match[1] ?? "").trim();
  const rawType = String(match[2] ?? "수").trim() || "수";
  const rhsAndTail = String(match[3] ?? "");
  const { body, tail } = splitLineBodyAndComment(rhsAndTail);
  let rhs = String(body ?? "").trim();
  if (rhs.endsWith(".")) {
    rhs = rhs.slice(0, -1).trim();
  }
  return {
    name,
    rawType,
    rhs,
    tail: String(tail ?? "").trim(),
  };
}

export function parsePrepBlockAssignments(text) {
  const lines = normalizeNewlines(text).split("\n");
  // Sim-core policy: UI 노출(슬라이더/축 후보)은 채비 선언면에서만 읽는다.
  const blockStartPattern = /^\s*(그릇채비|붙박이마련|붙박이채비|채비)\s*:?\s*\{/u;
  const numericPattern = /^\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?)\s*$/i;

  let inBlock = false;
  let depth = 0;

  const axisKeys = [];
  const axisSeen = new Set();
  const numericAssignments = [];
  const numericSeen = new Set();
  let defaultAxisKey = "";
  let defaultXAxisKey = "";

  lines.forEach((line) => {
    const textLine = String(line ?? "");
    const trimmed = textLine.trim();

    if (!inBlock && blockStartPattern.test(textLine)) {
      inBlock = true;
      depth = Math.max(1, countCharInText(textLine, "{") - countCharInText(textLine, "}") || 1);
      return;
    }
    if (!inBlock) return;

    if (trimmed && !trimmed.startsWith("//")) {
      const parsed = parsePrepAssignmentLine(textLine);
      if (parsed) {
        const name = parsed.name;
        const rawType = parsed.rawType;
        const rhs = parsed.rhs;
        const tail = parsed.tail;
        if (name && isVariableType(rawType) && !axisSeen.has(name)) {
          axisSeen.add(name);
          axisKeys.push(name);
        }
        if (!defaultAxisKey && name && isVariableType(rawType) && hasDefaultObservationHint(tail)) {
          defaultAxisKey = name;
        }
        if (!defaultXAxisKey && name && isVariableType(rawType) && hasDefaultObservationXHint(tail)) {
          defaultXAxisKey = name;
        }

        const numMatch = rhs.match(numericPattern);
        if (name && numMatch && isConstantLikeType(rawType) && !numericSeen.has(name)) {
          const value = Number(numMatch[1]);
          if (Number.isFinite(value)) {
            const inferred = inferBounds(value);
            const range = parseRangeAnnotation(tail);
            numericAssignments.push({
              id: name,
              name,
              type: rawType,
              value,
              min: range?.min ?? inferred.min,
              max: range?.max ?? inferred.max,
              step: range?.step ?? inferred.step,
              unit: "",
              source: "prep",
            });
            numericSeen.add(name);
          }
        }
      }
    }

    depth += countCharInText(textLine, "{");
    depth -= countCharInText(textLine, "}");
    if (depth <= 0) {
      inBlock = false;
      depth = 0;
    }
  });

  return {
    axisKeys,
    numericAssignments,
    defaultAxisKey,
    defaultXAxisKey,
  };
}

export function buildControlSpecsFromDdn(ddnText) {
  const { meta } = splitMetaHeader(ddnText);
  const defaultObsRaw = findDefaultObservationMeta(meta);
  const defaultObsXRaw = findDefaultObservationXMeta(meta);
  const prep = parsePrepBlockAssignments(ddnText);
  const prepSpecs = Array.isArray(prep.numericAssignments) ? prep.numericAssignments : [];
  const specs = prepSpecs;
  const source = prepSpecs.length ? "prep" : "none";
  const axisKeys = Array.from(new Set(Array.isArray(prep.axisKeys) ? prep.axisKeys : []));
  const preferredMetaAxis = pickFirstIdentifier(defaultObsRaw);
  const preferredPrepAxis = pickFirstIdentifier(prep.defaultAxisKey);
  const preferredMetaXAxis = pickFirstIdentifier(defaultObsXRaw);
  const preferredPrepXAxis = pickFirstIdentifier(prep.defaultXAxisKey);
  const defaultAxisKey = [preferredMetaAxis, preferredPrepAxis].find(
    (candidate) => candidate && axisKeys.includes(candidate),
  ) ?? "";
  const defaultXAxisKey = [preferredMetaXAxis, preferredPrepXAxis].find(
    (candidate) => candidate && axisKeys.includes(candidate),
  ) ?? "";
  return {
    specs,
    axisKeys,
    defaultAxisKey,
    defaultXAxisKey,
    source,
  };
}

function escapeRegExp(text) {
  return String(text ?? "").replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function formatNumber(value) {
  if (!Number.isFinite(value)) return "0";
  if (Math.abs(value) >= 1e6 || (Math.abs(value) > 0 && Math.abs(value) < 1e-5)) {
    return value.toExponential(6).replace(/\.?0+e/, "e");
  }
  return Number(value.toFixed(8)).toString();
}

export function applyControlValuesToDdnText(text, values = {}) {
  const lines = normalizeNewlines(text).split("\n");
  const valueEntries = Object.entries(values)
    .map(([key, val]) => [String(key ?? "").trim(), Number(val)])
    .filter(([key, val]) => Boolean(key) && Number.isFinite(val));
  if (!valueEntries.length) return text;

  const nextLines = [...lines];
  valueEntries.forEach(([key, value]) => {
    const pattern = new RegExp(
      `^(\\s*${escapeRegExp(key)}\\s*(?::\\s*[^<=]+)?\\s*(?:<-|=)\\s*)([+-]?(?:\\d+(?:\\.\\d+)?|\\.\\d+)(?:e[+-]?\\d+)?)(.*)$`,
      "iu",
    );
    // 새 문법 전환 후 채비/실행 블록에 같은 키가 반복될 수 있어 모든 매치를 갱신한다.
    for (let i = 0; i < nextLines.length; i += 1) {
      const line = nextLines[i];
      const match = line.match(pattern);
      if (!match) continue;
      nextLines[i] = `${match[1]}${formatNumber(value)}${match[3]}`;
    }
  });

  return nextLines.join("\n");
}
