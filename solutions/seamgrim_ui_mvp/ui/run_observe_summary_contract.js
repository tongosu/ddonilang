import { formatDisplayLabel, formatSourceLabel } from "./display_label_contract.js";

export function buildObserveSummaryViewModel({
  channels = 0,
  displayRows = [],
  availableRows = [],
  nonStrictRows = [],
  normalizedOutputRows = [],
  outputRowsMetric = "",
  outputRowsPreview = "",
  views = {},
  observeOutputActionCode = "",
  escapeHtml = (value) => String(value ?? ""),
  summarizeFamilyMetric = () => "출력 없음",
  buildFamilyActionHint = () => "권장: 카드 클릭 후 관찰 패널에서 값을 점검하세요.",
} = {}) {
  const safeChannels = Number.isFinite(Number(channels)) ? Math.max(0, Math.trunc(Number(channels))) : 0;
  const rows = Array.isArray(displayRows) ? displayRows : [];
  const available = Array.isArray(availableRows) ? availableRows : [];
  const nonStrict = Array.isArray(nonStrictRows) ? nonStrictRows : [];
  const outputRows = Array.isArray(normalizedOutputRows) ? normalizedOutputRows : [];

  const hasAny = safeChannels > 0 || available.length > 0 || outputRows.length > 0;
  const summaryParts = [
    `관찰채널 ${safeChannels}개`,
    `구조보기 ${available.length}개`,
  ];
  if (outputRows.length > 0) {
    summaryParts.push(`보임표 ${outputRows.length}행`);
  }
  if (nonStrict.length > 0) {
    summaryParts.push(`소스경고 ${nonStrict.length}개`);
  } else {
    summaryParts.push("소스 strict");
  }

  const summaryText = hasAny
    ? summaryParts.join(" · ")
    : "구조 관찰 출력이 없습니다. lesson의 보임 {} 설정을 확인하세요.";

  let chipsHtml = rows
    .map((row) => {
      const status = row?.available ? (row?.strict ? "ok" : "warn") : "off";
      const sourceRaw = row?.available ? String(row?.source ?? "").trim() : "off";
      const sourceText = sourceRaw ? formatSourceLabel(sourceRaw) : "꺼짐";
      return `<span class="run-observe-chip" data-status="${escapeHtml(status)}">${escapeHtml(String(row?.label ?? ""))}:${escapeHtml(sourceText)}</span>`;
    })
    .join("");
  if (outputRows.length > 0) {
    const metricText = String(outputRowsMetric ?? "").trim() || `${outputRows.length}행`;
    chipsHtml += `<span class="run-observe-chip" data-status="ok">보임표:${escapeHtml(metricText)}</span>`;
  }

  const compactMetricText = (raw, fallback = "") => {
    const text = String(raw ?? "").trim() || String(fallback ?? "").trim();
    if (!text) return "";
    return text.split("·")[0].trim() || text;
  };
  const stripGuidePrefix = (raw) => String(raw ?? "").trim().replace(/^권장:\s*/u, "");

  const outputRowsMetricText = compactMetricText(outputRowsMetric, `${outputRows.length}행`);
  const outputRowsTooltip = [
    "보임표 행 점검",
    String(outputRowsMetric ?? "").trim() || `${outputRows.length}행`,
    String(outputRowsPreview ?? "").trim() || "최근 행 요약 없음",
    "DDN 탭에서 보임표 행 항목을 확인합니다.",
  ].filter(Boolean).join(" | ");
  const outputRowCardHtml = outputRows.length > 0
    ? `<button type="button" class="run-observe-channel-card" data-status="ok" data-observe-action="${escapeHtml(String(observeOutputActionCode ?? ""))}" data-observe-token="table.row" title="${escapeHtml(outputRowsTooltip)}" aria-label="${escapeHtml(outputRowsTooltip)}">
            <span class="run-observe-channel-title">보임표 행</span>
            <span class="run-observe-channel-metric">${escapeHtml(outputRowsMetricText)}</span>
          </button>`
    : "";

  const familyCardsHtml = rows
    .map((row) => {
      const family = String(row?.family ?? "").trim();
      const availableNow = Boolean(row?.available);
      const strict = availableNow ? Boolean(row?.strict) : true;
      const status = availableNow ? (strict ? "ok" : "warn") : "off";
      const sourceRaw = availableNow ? String(row?.source ?? "").trim() || "unknown" : "off";
      const sourceText = formatSourceLabel(sourceRaw);
      const metricText = availableNow ? String(summarizeFamilyMetric(family, views?.[family])) : "출력 없음";
      const guideText = stripGuidePrefix(buildFamilyActionHint({ family, available: availableNow, strict }));
      const labelText = String(row?.label ?? "").trim();
      const titleText = `${labelText || formatDisplayLabel(family)} 점검 | 출처=${sourceText} | ${metricText} | ${guideText}`;
      const disabledAttr = availableNow ? "" : " disabled";
      return `<button type="button" class="run-observe-channel-card" data-status="${escapeHtml(status)}" data-observe-family="${escapeHtml(family)}" title="${escapeHtml(titleText)}" aria-label="${escapeHtml(titleText)}"${disabledAttr}>
            <span class="run-observe-channel-title">${escapeHtml(labelText)}</span>
            <span class="run-observe-channel-metric">${escapeHtml(compactMetricText(metricText, "출력 없음"))}</span>
          </button>`;
    })
    .join("");

  return {
    hasAny,
    level: !hasAny ? "none" : nonStrict.length > 0 ? "warn" : "ok",
    summaryText,
    chipsHtml,
    cardsHtml: `${outputRowCardHtml}${familyCardsHtml}`,
  };
}
