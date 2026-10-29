import { buildWarningPrimaryUserMessage } from "./run_warning_contract.js";

export function buildRunExecStatusViewModel({
  warnings = [],
  execPathHint = "",
  runtimeHintText = "",
  parseWarningSummary = "",
  viewSourceStrictness = { strict: true, nonStrictFamilies: [] },
} = {}) {
  const warningMessage = buildWarningPrimaryUserMessage(warnings);
  const pathText = String(execPathHint ?? "").trim();
  const hintText = String(runtimeHintText ?? "").trim();
  const parseSummary = String(parseWarningSummary ?? "").trim();
  const nonStrictFamilies = Array.isArray(viewSourceStrictness?.nonStrictFamilies)
    ? viewSourceStrictness.nonStrictFamilies.map((row) => String(row ?? "").trim()).filter(Boolean)
    : [];
  const sourceStrict = Boolean(viewSourceStrictness?.strict ?? true);

  let userStatusText = "실행 대기";
  let status = "idle";
  if (warningMessage) {
    userStatusText = `문제: ${warningMessage}`;
    status = "error";
  } else if (pathText) {
    userStatusText = pathText;
    status = pathText.includes("실행 실패") ? "error" : "ok";
  }
  if (!sourceStrict && status !== "error") {
    status = "warn";
  }

  const techSummaryParts = [];
  if (parseSummary) techSummaryParts.push(parseSummary);
  if (pathText) techSummaryParts.push(pathText);
  if (!sourceStrict && nonStrictFamilies.length) {
    techSummaryParts.push(`보기소스경고:${nonStrictFamilies.join("+")}`);
  }
  const techSummaryText = techSummaryParts.join(" · ") || "기술 상세";
  const hasTechnicalDetails = techSummaryParts.length > 0;
  const techBodyText = hasTechnicalDetails ? (hintText || techSummaryText) : "";

  return {
    userStatusText,
    status,
    techSummaryText,
    techBodyText,
    showTechnical: hasTechnicalDetails,
  };
}
