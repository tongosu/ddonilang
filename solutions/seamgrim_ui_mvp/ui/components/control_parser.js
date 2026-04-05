import { GUIDE_META_ALIASES, findGuideMetaValue, parseGuideMetaHeader } from "./guide_meta.js";

function normalizeNewlines(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function countCharInText(text, needle) {
  if (!text) return 0;
  return (String(text).match(new RegExp(`\\${needle}`, "g")) || []).length;
}

function splitMetaHeader(text) {
  const parsed = parseGuideMetaHeader(text);
  return {
    meta: parsed.meta,
    rawMeta: parsed.rawMeta,
    body: parsed.body,
  };
}

const DEFAULT_OBS_HINT_RE = new RegExp(
  `(?:${GUIDE_META_ALIASES.default_observation.map((entry) => String(entry).replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})`,
  "iu",
);
const DEFAULT_OBS_X_HINT_RE = new RegExp(
  `(?:${GUIDE_META_ALIASES.default_observation_x.map((entry) => String(entry).replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})`,
  "iu",
);

function pickFirstIdentifier(text) {
  const match = String(text ?? "").match(/([A-Za-z0-9_가-힣]+)/u);
  return String(match?.[1] ?? "").trim();
}

function hasDefaultObservationHint(text) {
  if (!text) return false;
  return DEFAULT_OBS_HINT_RE.test(String(text));
}

function hasDefaultObservationXHint(text) {
  if (!text) return false;
  return DEFAULT_OBS_X_HINT_RE.test(String(text));
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

function deriveStepFromRangeAndSplitCount(min, max, splitCount) {
  const lo = Number(min);
  const hi = Number(max);
  const count = Number(splitCount);
  if (!Number.isFinite(lo) || !Number.isFinite(hi)) return null;
  if (!Number.isFinite(count) || count <= 0) return null;
  const span = Math.abs(hi - lo);
  if (!(span > 0)) return null;
  return span / count;
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

function extractPrepBlockBody(text) {
  const lines = normalizeNewlines(text).split("\n");
  const blockStartPattern = /^\s*(그릇채비|붙박이마련|붙박이채비|채비)\s*:?\s*\{/u;

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
    const line = String(lines[lineIndex] ?? "");
    if (!blockStartPattern.test(line)) continue;
    const braceIndex = line.indexOf("{");
    if (braceIndex < 0) continue;
    let depth = 1;
    let body = "";
    for (let scanLine = lineIndex; scanLine < lines.length; scanLine += 1) {
      const segment = scanLine === lineIndex ? line.slice(braceIndex + 1) : String(lines[scanLine] ?? "");
      for (let pos = 0; pos < segment.length; pos += 1) {
        const ch = segment[pos];
        if (ch === "{") {
          depth += 1;
          body += ch;
          continue;
        }
        if (ch === "}") {
          depth -= 1;
          if (depth === 0) {
            return body;
          }
          body += ch;
          continue;
        }
        body += ch;
      }
      if (scanLine < lines.length - 1) {
        body += "\n";
      }
    }
    break;
  }
  return "";
}

const NUMERIC_LITERAL_PART = String.raw`[+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?`;
const MAEGIM_START_RE = new RegExp(
  String.raw`^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*(?:<-|=)\s*\(\s*(${NUMERIC_LITERAL_PART})\s*\)\s*(조건|매김)\s*\{(.*)$`,
  "iu",
);

function parseMaegimStarter(line) {
  const { body } = splitLineBodyAndComment(line);
  const match = String(body ?? "").match(MAEGIM_START_RE);
  if (!match) return null;
  const value = Number(match[3]);
  if (!Number.isFinite(value)) return null;
  return {
    name: String(match[1] ?? "").trim(),
    rawType: String(match[2] ?? "수").trim() || "수",
    value,
    restAfterOpen: String(match[5] ?? ""),
  };
}

function consumeBalancedBlock(lines, startIndex, initialTail) {
  let depth = 1;
  let body = "";
  for (let lineIndex = startIndex; lineIndex < lines.length; lineIndex += 1) {
    const segment = lineIndex === startIndex ? String(initialTail ?? "") : String(lines[lineIndex] ?? "");
    for (let pos = 0; pos < segment.length; pos += 1) {
      const ch = segment[pos];
      if (ch === "{") {
        depth += 1;
        body += ch;
        continue;
      }
      if (ch === "}") {
        depth -= 1;
        if (depth === 0) {
          return { body, nextIndex: lineIndex };
        }
        body += ch;
        continue;
      }
      body += ch;
    }
    if (lineIndex < lines.length - 1) {
      body += "\n";
    }
  }
  return null;
}

function parseRangeSpecFromMaegimBlock(text) {
  const rangeMatch = String(text ?? "").match(
    new RegExp(
      String.raw`범위\s*:\s*(${NUMERIC_LITERAL_PART})\s*\.\.(?:=)?\s*(${NUMERIC_LITERAL_PART})`,
      "iu",
    ),
  );
  const stepMatch = String(text ?? "").match(
    new RegExp(String.raw`간격\s*:\s*(${NUMERIC_LITERAL_PART})`, "iu"),
  );
  const splitCountMatch = String(text ?? "").match(
    new RegExp(String.raw`분할수\s*:\s*(${NUMERIC_LITERAL_PART})`, "iu"),
  );
  const min = Number(rangeMatch?.[1]);
  const max = Number(rangeMatch?.[2]);
  const step = Number(stepMatch?.[1]);
  const splitCount = Number(splitCountMatch?.[1]);
  return {
    min: Number.isFinite(min) && Number.isFinite(max) ? Math.min(min, max) : null,
    max: Number.isFinite(min) && Number.isFinite(max) ? Math.max(min, max) : null,
    step: Number.isFinite(step) && step > 0 ? step : null,
    splitCount: Number.isFinite(splitCount) && splitCount > 0 ? splitCount : null,
  };
}

function parseMaegimControlPlanJson(rawText) {
  if (!rawText) return null;
  try {
    const parsed = JSON.parse(String(rawText));
    if (!parsed || typeof parsed !== "object") return null;
    if (String(parsed.schema ?? "").trim() !== "ddn.maegim_control_plan.v1") return null;
    const controls = Array.isArray(parsed.controls) ? parsed.controls : [];
    const warnings = Array.isArray(parsed.warnings)
      ? parsed.warnings
          .filter((warning) => warning && typeof warning === "object")
          .map((warning) => ({
            code: String(warning.code ?? "").trim(),
            message: String(warning.message ?? "").trim(),
            name: String(warning.name ?? "").trim(),
            source: String(warning.source ?? "").trim(),
          }))
          .filter((warning) => warning.code)
      : [];
    return { controls, warnings };
  } catch (_) {
    return null;
  }
}

function buildControlSpecsFromMaegimControlPlan(rawText) {
  const parsed = parseMaegimControlPlanJson(rawText);
  if (!parsed) return { specs: [], warnings: [] };
  const out = [];
  const seen = new Set();
  parsed.controls.forEach((item) => {
    const name = String(item?.name ?? "").trim();
    if (!name || seen.has(name)) return;
    const initValue = Number(item?.init_expr_canon);
    if (!Number.isFinite(initValue)) return;
    const range = item?.range && typeof item.range === "object" ? item.range : null;
    const minRaw = Number(range?.min_expr_canon);
    const maxRaw = Number(range?.max_expr_canon);
    const stepRaw = Number(item?.step_expr_canon);
    const splitCountRaw = Number(item?.split_count_expr_canon);
    const derivedStep = deriveStepFromRangeAndSplitCount(minRaw, maxRaw, splitCountRaw);
    const inferred = inferBounds(initValue);
    out.push({
      id: name,
      name,
      type: String(item?.type_name ?? "수").trim() || "수",
      value: initValue,
      min: Number.isFinite(minRaw) && Number.isFinite(maxRaw) ? Math.min(minRaw, maxRaw) : inferred.min,
      max: Number.isFinite(minRaw) && Number.isFinite(maxRaw) ? Math.max(minRaw, maxRaw) : inferred.max,
      step: Number.isFinite(stepRaw) && stepRaw > 0 ? stepRaw : derivedStep ?? inferred.step,
      splitCount: Number.isFinite(splitCountRaw) && splitCountRaw > 0 ? splitCountRaw : null,
      unit: "",
      source: "maegim_control_json",
      declKind: String(item?.decl_kind ?? "").trim(),
    });
    seen.add(name);
  });
  return {
    specs: out,
    warnings: Array.isArray(parsed.warnings) ? parsed.warnings : [],
  };
}

export function parsePrepBlockAssignments(text) {
  const blockBody = extractPrepBlockBody(text);
  const lines = String(blockBody ?? "").split("\n");
  const numericPattern = /^\s*([+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:e[+-]?\d+)?)\s*$/i;

  const axisKeys = [];
  const axisSeen = new Set();
  const numericAssignments = [];
  const numericSeen = new Set();
  const warnings = [];
  let defaultAxisKey = "";
  let defaultXAxisKey = "";

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
    const line = lines[lineIndex];
    const textLine = String(line ?? "");
    const trimmed = textLine.trim();

    if (trimmed && !trimmed.startsWith("//")) {
      const maegimStarter = parseMaegimStarter(textLine);
      if (maegimStarter) {
        const { name, rawType, value, restAfterOpen } = maegimStarter;
        if (name && isVariableType(rawType) && !axisSeen.has(name)) {
          axisSeen.add(name);
          axisKeys.push(name);
        }
        const maegimBlock = consumeBalancedBlock(lines, lineIndex, restAfterOpen);
        if (name && isConstantLikeType(rawType) && !numericSeen.has(name)) {
          const inferred = inferBounds(value);
          const parsed = parseRangeSpecFromMaegimBlock(maegimBlock?.body ?? "");
          const derivedStep = deriveStepFromRangeAndSplitCount(parsed.min, parsed.max, parsed.splitCount);
          numericAssignments.push({
            id: name,
            name,
            type: rawType,
            value,
            min: parsed.min ?? inferred.min,
            max: parsed.max ?? inferred.max,
            step: parsed.step ?? derivedStep ?? inferred.step,
            splitCount: parsed.splitCount ?? null,
            unit: "",
            source: "prep",
          });
          numericSeen.add(name);
        }
        if (maegimBlock) {
          lineIndex = maegimBlock.nextIndex;
        }
        continue;
      }

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
            if (range) {
              warnings.push({
                code: "W_LEGACY_RANGE_COMMENT_DEPRECATED",
                message: "`// 범위(...)`는 deprecated입니다. `매김 {}`으로 옮기세요.",
                name,
                source: "prep_comment",
              });
            }
            numericSeen.add(name);
          }
        }
      }
    }
  }

  return {
    axisKeys,
    numericAssignments,
    defaultAxisKey,
    defaultXAxisKey,
    warnings,
  };
}

export function buildControlSpecsFromDdn(ddnText, options = {}) {
  const { meta, rawMeta } = splitMetaHeader(ddnText);
  const defaultObsRaw =
    String(meta?.default_observation ?? "") || findGuideMetaValue(rawMeta, "default_observation");
  const defaultObsXRaw =
    String(meta?.default_observation_x ?? "") || findGuideMetaValue(rawMeta, "default_observation_x");
  const prep = parsePrepBlockAssignments(ddnText);
  const prepSpecs = Array.isArray(prep.numericAssignments) ? prep.numericAssignments : [];
  const prepWarnings = Array.isArray(prep.warnings) ? prep.warnings : [];
  const maegimParsed = buildControlSpecsFromMaegimControlPlan(options?.maegimControlJson);
  const maegimSpecs = Array.isArray(maegimParsed.specs) ? maegimParsed.specs : [];
  const maegimWarnings = Array.isArray(maegimParsed.warnings) ? maegimParsed.warnings : [];
  const specs = maegimSpecs.length ? maegimSpecs : prepSpecs;
  const source = maegimSpecs.length ? "maegim_control_json" : prepSpecs.length ? "prep" : "none";
  const warnings = maegimSpecs.length ? maegimWarnings : prepWarnings;
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
    warnings,
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
    const maegimPattern = new RegExp(
      `^(\\s*${escapeRegExp(key)}\\s*(?::\\s*[^<=]+)?\\s*(?:<-|=)\\s*\\()([+-]?(?:\\d+(?:\\.\\d+)?|\\.\\d+)(?:e[+-]?\\d+)?)(\\s*\\)\\s*(?:조건|매김)\\s*\\{.*)$`,
      "iu",
    );
    const pattern = new RegExp(
      `^(\\s*${escapeRegExp(key)}\\s*(?::\\s*[^<=]+)?\\s*(?:<-|=)\\s*)([+-]?(?:\\d+(?:\\.\\d+)?|\\.\\d+)(?:e[+-]?\\d+)?)(.*)$`,
      "iu",
    );
    // 새 문법 전환 후 채비/실행 블록에 같은 키가 반복될 수 있어 모든 매치를 갱신한다.
    for (let i = 0; i < nextLines.length; i += 1) {
      const line = nextLines[i];
      const maegimMatch = line.match(maegimPattern);
      if (maegimMatch) {
        nextLines[i] = `${maegimMatch[1]}${formatNumber(value)}${maegimMatch[3]}`;
        continue;
      }
      const match = line.match(pattern);
      if (!match) continue;
      nextLines[i] = `${match[1]}${formatNumber(value)}${match[3]}`;
    }
  });

  return nextLines.join("\n");
}
