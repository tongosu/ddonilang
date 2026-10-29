export function resolveShapeModeLabel(rawMode) {
  const mode = String(rawMode ?? "").trim().toLowerCase();
  if (mode === "native") return "native";
  if (mode === "space2d") return "space2d";
  if (mode === "graph") return "graph";
  if (mode === "console") return "console";
  if (mode === "debug-fallback") return "debug-fallback";
  if (mode === "fallback") return "fallback";
  return "none";
}

function normalizeHintCoreText(rawText = "") {
  return String(rawText ?? "")
    .trim()
    .replace(/^가이드:\s*/u, "")
    .replace(/^문제:\s*/u, "")
    .replace(/^점검\s*필요:\s*/u, "")
    .replace(/^실행\s*실패\([^)]+\):\s*/u, "")
    .trim()
    .toLowerCase();
}

export function buildRuntimeHintViewModel({
  tickCount = null,
  controlCount = 0,
  runtimeGuideText = "",
  execPathHint = "",
  shapeMode = "none",
  parseWarningSummary = "",
  viewFamilies = [],
  previewSummary = "",
  nonStrictFamilies = [],
  observationSummary = "",
  labels = {},
} = {}) {
  const segments = [];
  const hasTickCount = tickCount !== null && tickCount !== undefined && String(tickCount).trim() !== "";
  const safeTickCount = hasTickCount && Number.isFinite(Number(tickCount)) ? Math.max(0, Math.trunc(Number(tickCount))) : null;
  const safeControlCount = Number.isFinite(Number(controlCount)) ? Math.max(0, Math.trunc(Number(controlCount))) : 0;
  const guideText = String(runtimeGuideText ?? "").trim();
  const pathHint = String(execPathHint ?? "").trim();
  const warningSummary = String(parseWarningSummary ?? "").trim();
  const observationText = String(observationSummary ?? "").trim();
  const guideCore = normalizeHintCoreText(guideText);
  const pathCore = normalizeHintCoreText(pathHint);
  const warningCore = normalizeHintCoreText(warningSummary);
  const isProblemGuide = /^\s*문제\s*:/u.test(guideText);

  if (safeTickCount !== null) {
    segments.push(`${safeTickCount}마디`);
  }
  if (isProblemGuide) {
    segments.push(`점검 필요: ${guideText.replace(/^\s*문제\s*:\s*/u, "")}`);
  } else if (guideText) {
    segments.push(`가이드: ${guideText}`);
  }
  const shouldShowPathHint =
    pathHint.includes("실행 실패")
    || pathHint.includes("실행 차단")
    || pathHint.includes("문법 문제")
    || pathHint.includes("입력 문제")
    || pathHint.includes("논리 문제");
  if (!isProblemGuide && shouldShowPathHint && pathHint && (!guideCore || guideCore !== pathCore)) {
    segments.push(pathHint);
  }
  if (
    warningSummary
    && !isProblemGuide
    && (!guideCore || guideCore !== warningCore)
    && (!pathCore || pathCore !== warningCore)
  ) {
    segments.push(warningSummary);
  }
  if (observationText) {
    segments.push(observationText);
  }

  const hasFailedPath = pathHint.includes("실행 실패");
  const hasWarnings = warningSummary.length > 0;
  return {
    text: segments.join(" · "),
    status: hasFailedPath || hasWarnings ? "error" : "ok",
  };
}
