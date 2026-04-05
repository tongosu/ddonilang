const FORMULA_IDENTIFIER_RE = /^[A-Za-z_가-힣][0-9A-Za-z_가-힣]*$/;

function toFiniteNumber(raw) {
  const num = Number(raw);
  return Number.isFinite(num) ? num : null;
}

function normalizeFormulaIdentifier(raw = "", fallback = "") {
  const text = String(raw ?? "").trim();
  if (FORMULA_IDENTIFIER_RE.test(text)) return text;
  return String(fallback ?? "").trim();
}

function normalizeFormulaExpression(raw = "") {
  const text = String(raw ?? "").trim();
  if (!text) return "";
  return text.replace(/[.;]\s*$/, "").trim();
}

function formatDdnNumber(raw) {
  const num = Number(raw);
  if (!Number.isFinite(num)) return "0";
  if (Number.isInteger(num)) return String(num);
  return String(Number(num.toFixed(12)));
}

function normalizeFormulaSample({ axisVar = "x", xMin = 0, xMax = 10, step = 1 } = {}) {
  const varName = normalizeFormulaIdentifier(axisVar, "x");
  const min = toFiniteNumber(xMin);
  const max = toFiniteNumber(xMax);
  const delta = toFiniteNumber(step);
  if (!varName || min === null || max === null || delta === null || max < min || delta <= 0) {
    return null;
  }
  return {
    var: varName,
    x_min: min,
    x_max: max,
    step: delta,
  };
}

export function parseFormulaSugarDraft({
  formulaText = "",
  axisVar = "x",
  xMin = 0,
  xMax = 10,
  step = 1,
} = {}) {
  const text = String(formulaText ?? "").trim();
  if (!text) {
    return { ok: false, error: "수식이 비어 있습니다." };
  }
  const firstEq = text.indexOf("=");
  const lastEq = text.lastIndexOf("=");
  if (firstEq <= 0 || firstEq !== lastEq) {
    return { ok: false, error: "수식은 `y = 식` 형태의 단일 등식이어야 합니다." };
  }
  const outputVar = normalizeFormulaIdentifier(text.slice(0, firstEq), "");
  if (!outputVar) {
    return { ok: false, error: "좌변 변수 이름이 올바르지 않습니다." };
  }
  const expression = normalizeFormulaExpression(text.slice(firstEq + 1));
  if (!expression) {
    return { ok: false, error: "우변 식이 비어 있습니다." };
  }
  const normalizedAxisVar = normalizeFormulaIdentifier(axisVar, "x");
  if (!normalizedAxisVar) {
    return { ok: false, error: "축 변수 이름이 올바르지 않습니다." };
  }
  const sample = normalizeFormulaSample({
    axisVar: normalizedAxisVar,
    xMin,
    xMax,
    step,
  });
  if (!sample) {
    return { ok: false, error: "범위 입력이 올바르지 않습니다. (x_max >= x_min, step > 0)" };
  }
  return {
    ok: true,
    data: {
      formulaText: `${outputVar} = ${expression}`,
      outputVar,
      expression,
      axisVar: normalizedAxisVar,
      sample,
    },
  };
}

export function buildFormulaSugarDdn(draft = null) {
  const row = draft && typeof draft === "object" ? draft : {};
  const axisVar = normalizeFormulaIdentifier(row.axisVar, "x");
  const outputVar = normalizeFormulaIdentifier(row.outputVar, "y");
  const expression = normalizeFormulaExpression(row.expression);
  const sample = normalizeFormulaSample({
    axisVar,
    xMin: row?.sample?.x_min,
    xMax: row?.sample?.x_max,
    step: row?.sample?.step,
  });
  if (!axisVar || !outputVar || !expression || !sample) {
    return "";
  }
  const listVar = `${axisVar}목록`;
  const minVar = `${axisVar}_min`;
  const maxVar = `${axisVar}_max`;
  const stepVar = `${axisVar}_step`;
  const minText = formatDdnNumber(sample.x_min);
  const maxText = formatDdnNumber(sample.x_max);
  const stepText = formatDdnNumber(sample.step);

  return [
    `#이름: formula_${outputVar}`,
    "#설명: 수식 입력에서 자동 생성됨",
    "",
    "채비 {",
    `  ${minVar}:수 <- ${minText}.`,
    `  ${maxVar}:수 <- ${maxText}.`,
    `  ${stepVar}:수 <- ${stepText}.`,
    "}.",
    "",
    "(매마디)마다 {",
    `  ${listVar} <- (${minVar}, ${maxVar}, ${stepVar}) 범위.`,
    "",
    `  (${axisVar}) ${listVar}에 대해: {`,
    `    ${outputVar} <- ${expression}.`,
    "    보임 {",
    `      ${axisVar}: ${axisVar}.`,
    `      ${outputVar}: ${outputVar}.`,
    "    }.",
    "  }.",
    "}.",
    "",
  ].join("\n");
}

function spliceWithBoundedSelection(base = "", insertText = "", selectionStart = null, selectionEnd = null) {
  const source = String(base ?? "");
  const insert = String(insertText ?? "");
  const length = source.length;
  const startCandidate = Number(selectionStart);
  const endCandidate = Number(selectionEnd);
  const start = Number.isFinite(startCandidate)
    ? Math.max(0, Math.min(length, Math.trunc(startCandidate)))
    : length;
  const end = Number.isFinite(endCandidate)
    ? Math.max(start, Math.min(length, Math.trunc(endCandidate)))
    : start;
  const before = source.slice(0, start);
  const after = source.slice(end);
  const needLeadingBreak = before.length > 0 && !before.endsWith("\n");
  const needTrailingBreak = after.length > 0 && !insert.endsWith("\n");
  return `${before}${needLeadingBreak ? "\n" : ""}${insert}${needTrailingBreak ? "\n" : ""}${after}`;
}

export function applyFormulaDdnToSource(
  baseDdn = "",
  derivedDdn = "",
  { mode = "replace", selectionStart = null, selectionEnd = null } = {},
) {
  const chunk = String(derivedDdn ?? "").trim();
  if (!chunk) return String(baseDdn ?? "");
  if (String(mode ?? "").trim().toLowerCase() === "insert") {
    return spliceWithBoundedSelection(String(baseDdn ?? ""), `${chunk}\n`, selectionStart, selectionEnd);
  }
  return `${chunk}\n`;
}
