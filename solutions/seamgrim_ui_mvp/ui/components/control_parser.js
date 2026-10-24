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
    /^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*(?::=|<-|=)\s*(.+)$/u,
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
  String.raw`^\s*([A-Za-z0-9_가-힣]+)\s*(?::\s*([A-Za-z0-9_가-힣]+))?\s*(?::=|<-|=)\s*\(\s*(${NUMERIC_LITERAL_PART})\s*\)\s*(조건|매김)\s*\{(.*)$`,
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

const MAEGIM_CONTROL_PLAN_SCHEMA = "ddn.maegim_control_plan.v1";
const NUMERIC_CONTROL_TYPES = new Set(["수", "실수", "정수", "number", "float", "int"]);

function controlPlanDiagnostic(code, message, detail = "") {
  return {
    code,
    phase: "control_plan",
    message,
    detail: String(detail ?? ""),
  };
}

function parseMaegimControlPlanJson(rawText) {
  const text = String(rawText ?? "").trim();
  if (!text) {
    return {
      status: "missing",
      controls: [],
      warnings: [],
      diagnostic: controlPlanDiagnostic(
        "E_WORKBENCH_CONTROL_PLAN_MISSING",
        "권위 있는 매김 조절 계획이 없습니다.",
      ),
    };
  }
  try {
    const parsed = JSON.parse(text);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {
        status: "invalid",
        controls: [],
        warnings: [],
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_INVALID",
          "매김 조절 계획 payload가 객체가 아닙니다.",
        ),
      };
    }
    const schema = String(parsed.schema ?? "").trim();
    if (schema !== MAEGIM_CONTROL_PLAN_SCHEMA) {
      return {
        status: "unsupported",
        controls: [],
        warnings: [],
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_UNSUPPORTED",
          "지원하지 않는 매김 조절 계획 schema입니다.",
          schema || "schema_missing",
        ),
      };
    }
    if (!Array.isArray(parsed.controls)) {
      return {
        status: "invalid",
        controls: [],
        warnings: [],
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_INVALID",
          "매김 조절 계획 controls가 배열이 아닙니다.",
        ),
      };
    }
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
    return {
      status: parsed.controls.length ? "ready" : "empty",
      controls: parsed.controls,
      warnings,
      diagnostic: null,
    };
  } catch (error) {
    return {
      status: "invalid",
      controls: [],
      warnings: [],
      diagnostic: controlPlanDiagnostic(
        "E_WORKBENCH_CONTROL_PLAN_INVALID",
        "매김 조절 계획 JSON을 해석할 수 없습니다.",
        error?.message ?? String(error ?? ""),
      ),
    };
  }
}

function buildControlSpecsFromMaegimControlPlan(rawText) {
  const parsed = parseMaegimControlPlanJson(rawText);
  if (parsed.status !== "ready") {
    return {
      specs: [],
      warnings: parsed.warnings,
      status: parsed.status,
      diagnostic: parsed.diagnostic,
    };
  }
  const out = [];
  const seen = new Set();
  for (const item of parsed.controls) {
    const name = String(item?.name ?? "").trim();
    if (!name || seen.has(name)) {
      return {
        specs: [],
        warnings: parsed.warnings,
        status: "invalid",
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_INVALID",
          "매김 조절 identity가 없거나 중복됐습니다.",
          name || "control_name_missing",
        ),
      };
    }
    const type = String(item?.type_name ?? "").trim();
    if (!NUMERIC_CONTROL_TYPES.has(normalizeType(type))) {
      return {
        specs: [],
        warnings: parsed.warnings,
        status: "unsupported",
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_UNSUPPORTED",
          "현재 slider가 지원하지 않는 매김 조절 type입니다.",
          `${name}:${type || "type_missing"}`,
        ),
      };
    }
    const initText = String(item?.init_expr_canon ?? "").trim();
    const initValue = Number(item?.init_expr_canon);
    if (!initText || !Number.isFinite(initValue)) {
      return {
        specs: [],
        warnings: parsed.warnings,
        status: "invalid",
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_INVALID",
          "매김 조절 초기값이 유한한 정본 수가 아닙니다.",
          name,
        ),
      };
    }
    const range = item?.range && typeof item.range === "object" ? item.range : null;
    const minRaw = Number(range?.min_expr_canon);
    const maxRaw = Number(range?.max_expr_canon);
    if (!range || !Number.isFinite(minRaw) || !Number.isFinite(maxRaw) || !(maxRaw > minRaw)) {
      return {
        specs: [],
        warnings: parsed.warnings,
        status: "unsupported",
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_UNSUPPORTED",
          "slider 투영에 필요한 명시적 정본 범위가 없습니다.",
          name,
        ),
      };
    }
    const stepRaw = Number(item?.step_expr_canon);
    const splitCountRaw = Number(item?.split_count_expr_canon);
    const hasStep = String(item?.step_expr_canon ?? "").trim() !== "";
    const hasSplitCount = String(item?.split_count_expr_canon ?? "").trim() !== "";
    if (hasStep && (!Number.isFinite(stepRaw) || !(stepRaw > 0))) {
      return {
        specs: [],
        warnings: parsed.warnings,
        status: "invalid",
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_INVALID",
          "매김 조절 간격이 양의 정본 수가 아닙니다.",
          name,
        ),
      };
    }
    if (hasSplitCount && (!Number.isFinite(splitCountRaw) || !(splitCountRaw > 0))) {
      return {
        specs: [],
        warnings: parsed.warnings,
        status: "invalid",
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_INVALID",
          "매김 조절 분할수가 양의 정본 수가 아닙니다.",
          name,
        ),
      };
    }
    if (hasStep && hasSplitCount) {
      return {
        specs: [],
        warnings: parsed.warnings,
        status: "invalid",
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_INVALID",
          "매김 조절 간격과 분할수가 동시에 있습니다.",
          name,
        ),
      };
    }
    const derivedStep = deriveStepFromRangeAndSplitCount(minRaw, maxRaw, splitCountRaw);
    const step = hasStep ? stepRaw : derivedStep;
    if (!Number.isFinite(step) || !(step > 0)) {
      return {
        specs: [],
        warnings: parsed.warnings,
        status: "unsupported",
        diagnostic: controlPlanDiagnostic(
          "E_WORKBENCH_CONTROL_PLAN_UNSUPPORTED",
          "slider 투영에 필요한 명시적 간격 또는 분할수가 없습니다.",
          name,
        ),
      };
    }
    out.push({
      id: name,
      name,
      type,
      value: initValue,
      min: minRaw,
      max: maxRaw,
      step,
      splitCount: hasSplitCount ? splitCountRaw : null,
      unit: "",
      source: "maegim_control_json",
      declKind: String(item?.decl_kind ?? "").trim(),
    });
    seen.add(name);
  }
  return {
    specs: out,
    warnings: Array.isArray(parsed.warnings) ? parsed.warnings : [],
    status: "ready",
    diagnostic: null,
  };
}

export function parsePrepBlockAssignments(text) {
  const blockBody = extractPrepBlockBody(text);
  const lines = String(blockBody ?? "").split("\n");
  const axisKeys = [];
  const axisSeen = new Set();
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
        const { name, rawType, restAfterOpen } = maegimStarter;
        if (name && isVariableType(rawType) && !axisSeen.has(name)) {
          axisSeen.add(name);
          axisKeys.push(name);
        }
        const maegimBlock = consumeBalancedBlock(lines, lineIndex, restAfterOpen);
        if (maegimBlock) {
          lineIndex = maegimBlock.nextIndex;
        }
        continue;
      }

      const parsed = parsePrepAssignmentLine(textLine);
      if (parsed) {
        const name = parsed.name;
        const rawType = parsed.rawType;
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

        if (name && tail.includes("범위(")) {
          warnings.push({
            code: "W_LEGACY_RANGE_COMMENT_DEPRECATED",
            message: "`// 범위(...)`는 폐기 대상입니다. `매김 { 범위: a..b. 간격: c. }`로 옮기세요.",
            name,
            source: "legacy_range_comment",
          });
        }
      }
    }
  }

  return {
    axisKeys,
    numericAssignments: [],
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
  const prepWarnings = Array.isArray(prep.warnings) ? prep.warnings : [];
  const maegimParsed = buildControlSpecsFromMaegimControlPlan(options?.maegimControlJson);
  const maegimSpecs = Array.isArray(maegimParsed.specs) ? maegimParsed.specs : [];
  const maegimWarnings = Array.isArray(maegimParsed.warnings) ? maegimParsed.warnings : [];
  const specs = maegimParsed.status === "ready" ? maegimSpecs : [];
  const source = maegimParsed.status === "ready"
    ? "maegim_control_json"
    : maegimParsed.status === "empty"
      ? "maegim_control_json_empty"
      : "none";
  const warnings = maegimParsed.status === "ready" || maegimParsed.status === "empty"
    ? maegimWarnings
    : prepWarnings;
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
    controlPlanStatus: maegimParsed.status,
    controlPlanDiagnostic: maegimParsed.diagnostic,
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
      `^(\\s*${escapeRegExp(key)}\\s*(?::\\s*[^<=]+)?\\s*(?::=|<-|=)\\s*\\()([+-]?(?:\\d+(?:\\.\\d+)?|\\.\\d+)(?:e[+-]?\\d+)?)(\\s*\\)\\s*(?:조건|매김)\\s*\\{.*)$`,
      "iu",
    );
    const pattern = new RegExp(
      `^(\\s*${escapeRegExp(key)}\\s*(?::\\s*[^<=]+)?\\s*(?::=|<-|=)\\s*)([+-]?(?:\\d+(?:\\.\\d+)?|\\.\\d+)(?:e[+-]?\\d+)?)(.*)$`,
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
