import { summarizeGraphView } from "./components/graph_preview.js";
import { summarizeTableView } from "./components/table_preview.js";
import { summarizeStructureView } from "./seamgrim_runtime_state.js";

function escAttr(text) {
  return String(text ?? "")
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

const DEFAULT_PREVIEW_FAMILY_PRIORITY = Object.freeze([
  "space2d",
  "graph",
  "table",
  "structure",
  "text",
]);

function normalizeFamily(raw) {
  return String(raw ?? "").trim().toLowerCase();
}

function familyLabel(family) {
  const kind = normalizeFamily(family);
  if (kind === "space2d") return "공간";
  if (kind === "graph") return "그래프";
  if (kind === "table") return "표";
  if (kind === "structure") return "구조";
  if (kind === "text") return "텍스트";
  return kind || "보기";
}

function summarizeTextPreview(text) {
  const normalized = String(text ?? "")
    .replace(/\r\n/g, "\n")
    .replace(/\r/g, "\n")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
  if (!normalized.length) return null;
  return {
    title: "",
    lineCount: normalized.length,
    excerpt: normalized.slice(0, 2).join(" / "),
  };
}

function summarizeSpace2dPreview(space2d) {
  if (!space2d || typeof space2d !== "object") return null;
  const points = Array.isArray(space2d?.points) ? space2d.points.length : 0;
  const shapes = Array.isArray(space2d?.shapes) ? space2d.shapes.length : 0;
  const drawlist = Array.isArray(space2d?.drawlist) ? space2d.drawlist.length : 0;
  if (points <= 0 && shapes <= 0 && drawlist <= 0) return null;
  return {
    title: String(space2d?.meta?.title ?? "").trim(),
    pointCount: points,
    shapeCount: shapes + drawlist,
  };
}

function summarizePreviewPayload(family, payload, text) {
  const kind = String(family ?? "").trim().toLowerCase();
  if (kind === "graph") return summarizeGraphView(payload, { seriesLimit: 3 });
  if (kind === "table") return summarizeTableView(payload, { maxCols: 3 });
  if (kind === "structure") return summarizeStructureView(payload, { sampleLimit: 2 });
  if (kind === "space2d") return summarizeSpace2dPreview(payload);
  if (kind === "text") return summarizeTextPreview(text);
  return null;
}

function derivePreviewTitle(summary) {
  if (!summary || typeof summary !== "object") return "";
  return String(summary.title ?? "").trim();
}

function derivePreviewTooltip(family, summary) {
  const kind = String(family ?? "").trim().toLowerCase();
  if (!summary || typeof summary !== "object") return kind;
  const title = derivePreviewTitle(summary);
  if (kind === "graph") {
    const base = title || "그래프";
    return `${base} · 계열 ${summary.seriesCount ?? 0}개 · 점 ${summary.pointCount ?? 0}개`;
  }
  if (kind === "table") {
    const base = title || "표";
    return `${base} · 열 ${summary.columnCount ?? 0}개 · 행 ${summary.rowCount ?? 0}개`;
  }
  if (kind === "structure") {
    const base = title || "구조";
    return `${base} · 노드 ${summary.nodeCount ?? 0}개 · 간선 ${summary.edgeCount ?? 0}개`;
  }
  if (kind === "space2d") {
    const base = title || "공간";
    return `${base} · 점 ${summary.pointCount ?? 0}개 · 도형 ${summary.shapeCount ?? 0}개`;
  }
  if (kind === "text") {
    const base = title || "텍스트";
    return `${base} · 줄 ${summary.lineCount ?? 0}개`;
  }
  return title || kind;
}

export function buildPreviewResult({ descriptor = null, html = "", payload = null, text = "" } = {}) {
  if (!descriptor || !html) return null;
  const family = String(descriptor?.family ?? "").trim().toLowerCase();
  const mode = String(descriptor?.mode ?? "").trim().toLowerCase();
  const summary = summarizePreviewPayload(family, payload, text);
  return {
    family,
    mode,
    html,
    payload,
    text,
    descriptor,
    title: derivePreviewTitle(summary),
    summary,
    tooltip: derivePreviewTooltip(family, summary),
  };
}

export function buildFamilyPreviewResult({
  family = "",
  payload = null,
  html = "",
  text = "",
  mode = "runtime",
} = {}) {
  const normalizedFamily = String(family ?? "").trim().toLowerCase();
  const normalizedMode = String(mode ?? "").trim().toLowerCase() || "runtime";
  if (!normalizedFamily || !html) return null;
  return buildPreviewResult({
    descriptor: {
      family: normalizedFamily,
      mode: normalizedMode,
    },
    html,
    payload,
    text,
  });
}

export function pickPrimaryPreviewResult(results, { preferredFamilies = [] } = {}) {
  const list = Array.isArray(results) ? results.filter(Boolean) : [];
  if (!list.length) return null;
  const preferred = Array.isArray(preferredFamilies)
    ? preferredFamilies.map((item) => normalizeFamily(item)).filter(Boolean)
    : [];
  const priority = preferred.length
    ? [...preferred, ...DEFAULT_PREVIEW_FAMILY_PRIORITY.filter((item) => !preferred.includes(item))]
    : [...DEFAULT_PREVIEW_FAMILY_PRIORITY];
  let best = null;
  let bestRank = Number.POSITIVE_INFINITY;
  list.forEach((result, index) => {
    const family = normalizeFamily(result?.family);
    const rank = priority.indexOf(family);
    const score = rank >= 0 ? rank : priority.length + index;
    if (best === null || score < bestRank) {
      best = result;
      bestRank = score;
    }
  });
  return best;
}

export function wrapPreviewResultHtml(result, { className = "preview-result-card" } = {}) {
  if (!result?.html) return "";
  const classes = [String(className ?? "").trim()].filter(Boolean).join(" ");
  const family = escAttr(result?.family ?? "");
  const mode = escAttr(result?.mode ?? "");
  const title = escAttr(result?.tooltip ?? "");
  const classAttr = classes ? ` class="${escAttr(classes)}"` : "";
  const titleAttr = title ? ` title="${title}"` : "";
  return `<div${classAttr} data-preview-family="${family}" data-preview-mode="${mode}"${titleAttr}>${result.html}</div>`;
}

export function buildPreviewSummaryStripHtml(
  result,
  { className = "preview-summary-strip", label = "대표 보기" } = {},
) {
  if (!result?.html) return "";
  const classes = [String(className ?? "").trim()].filter(Boolean).join(" ");
  const family = normalizeFamily(result?.family);
  const mode = normalizeFamily(result?.mode);
  const tooltip = String(result?.tooltip ?? "").trim();
  const classAttr = classes ? ` class="${escAttr(classes)}"` : "";
  const titleAttr = tooltip ? ` title="${escAttr(tooltip)}"` : "";
  return `<div${classAttr} data-preview-family="${escAttr(family)}" data-preview-mode="${escAttr(mode)}"${titleAttr}>` +
    `<span class="preview-summary-label">${escAttr(label)}</span>` +
    `<span class="preview-summary-family">${escAttr(familyLabel(family))}</span>` +
    `<span class="preview-summary-text">${escAttr(tooltip || familyLabel(family))}</span>` +
    `</div>`;
}

export function buildPreviewResultCollectionHtml(
  results,
  {
    preferredFamilies = [],
    summaryClassName = "preview-summary-strip",
    summaryLabel = "대표 보기",
    cardClassName = "preview-result-card",
  } = {},
) {
  const collection = buildPreviewResultCollection(results, {
    preferredFamilies,
    summaryClassName,
    summaryLabel,
    cardClassName,
  });
  return String(collection?.html ?? "");
}

export function buildPreviewResultCollection(
  results,
  {
    preferredFamilies = [],
    summaryClassName = "preview-summary-strip",
    summaryLabel = "대표 보기",
    cardClassName = "preview-result-card",
  } = {},
) {
  const list = Array.isArray(results) ? results.filter(Boolean) : [];
  if (!list.length) return null;
  const primary = pickPrimaryPreviewResult(list, { preferredFamilies });
  const summaryHtml = buildPreviewSummaryStripHtml(primary, {
    className: summaryClassName,
    label: summaryLabel,
  });
  const cardsHtml = list
    .map((result) => wrapPreviewResultHtml(result, { className: cardClassName }))
    .join("");
  const families = [];
  list.forEach((result) => {
    const family = normalizeFamily(result?.family);
    if (!family || families.includes(family)) return;
    families.push(family);
  });
  return {
    results: list,
    primary,
    count: list.length,
    families,
    html: `${summaryHtml}${cardsHtml}`,
    family: String(primary?.family ?? ""),
    mode: String(primary?.mode ?? ""),
    descriptor: primary?.descriptor ?? null,
    payload: primary?.payload ?? null,
    text: String(primary?.text ?? ""),
    title: String(primary?.title ?? ""),
    summary: primary?.summary ?? null,
    tooltip: String(primary?.tooltip ?? ""),
  };
}
