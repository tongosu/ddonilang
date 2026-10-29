import { markdownToHtml } from "./markdown.js";
import { buildGraphPreviewHtml } from "./graph_preview.js";
import { buildStructurePreviewHtml } from "./structure_preview.js";
import { buildTablePreviewHtml } from "./table_preview.js";

function escapePreviewHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function trimPreviewLines(markdown, { maxLines = 3 } = {}) {
  const lines = String(markdown ?? "")
    .replace(/\r\n/g, "\n")
    .replace(/\r/g, "\n")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
  return lines.slice(0, Math.max(1, Math.trunc(Number(maxLines) || 3))).join("\n");
}

function finiteNumber(value) {
  const n = Number(value);
  return Number.isFinite(n) ? n : null;
}

export function buildTextPreviewHtml(markdown, { maxLines = 3, containerClass = "lesson-card-preview lesson-card-preview--text" } = {}) {
  const excerpt = trimPreviewLines(markdown, { maxLines });
  if (!excerpt) return "";
  return `<div class="${containerClass}">${markdownToHtml(excerpt)}</div>`;
}

export function buildSpace2dPreviewHtml(
  space2d,
  {
    width = 220,
    height = 132,
    maxPoints = 12,
    containerClass = "lesson-card-preview lesson-card-preview--space2d",
    titleClass = "lesson-card-space2d-title",
    canvasClass = "lesson-card-space2d-canvas",
    pointClass = "lesson-card-space2d-point",
    lineClass = "lesson-card-space2d-line",
    circleClass = "lesson-card-space2d-circle",
  } = {},
) {
  if (!space2d || typeof space2d !== "object") return "";
  const points = Array.isArray(space2d?.points) ? space2d.points.slice(0, Math.max(1, Number(maxPoints) || 12)) : [];
  const shapes = Array.isArray(space2d?.shapes) ? space2d.shapes : [];
  const drawlist = Array.isArray(space2d?.drawlist) ? space2d.drawlist : [];
  if (!points.length && !shapes.length && !drawlist.length) return "";
  const margin = 14;
  const camera = space2d?.camera ?? {};
  const xValues = [];
  const yValues = [];
  points.forEach((row) => {
    const x = finiteNumber(row?.x);
    const y = finiteNumber(row?.y);
    if (x !== null) xValues.push(x);
    if (y !== null) yValues.push(y);
  });
  const shapeList = shapes.length ? shapes : drawlist;
  shapeList.forEach((row) => {
    ["x", "x1", "x2"].forEach((key) => {
      const value = finiteNumber(row?.[key]);
      if (value !== null) xValues.push(value);
    });
    ["y", "y1", "y2"].forEach((key) => {
      const value = finiteNumber(row?.[key]);
      if (value !== null) yValues.push(value);
    });
  });
  const xMin = finiteNumber(camera?.x_min) ?? (xValues.length ? Math.min(...xValues) : -1);
  const xMax = finiteNumber(camera?.x_max) ?? (xValues.length ? Math.max(...xValues) : 1);
  const yMin = finiteNumber(camera?.y_min) ?? (yValues.length ? Math.min(...yValues) : -1);
  const yMax = finiteNumber(camera?.y_max) ?? (yValues.length ? Math.max(...yValues) : 1);
  const xSpan = Math.max(1e-6, xMax - xMin);
  const ySpan = Math.max(1e-6, yMax - yMin);
  const px = (x) => margin + ((x - xMin) / xSpan) * (width - margin * 2);
  const py = (y) => margin + (1 - ((y - yMin) / ySpan)) * (height - margin * 2);
  const pointSvg = points
    .map((row) => {
      const x = finiteNumber(row?.x);
      const y = finiteNumber(row?.y);
      if (x === null || y === null) return "";
      return `<circle cx="${px(x).toFixed(1)}" cy="${py(y).toFixed(1)}" r="3.5" class="${pointClass}" />`;
    })
    .filter(Boolean)
    .join("");
  const shapeSvg = shapeList
    .map((row) => {
      const kind = String(row?.kind ?? "").trim().toLowerCase();
      if (kind === "line") {
        const x1 = finiteNumber(row?.x1);
        const y1 = finiteNumber(row?.y1);
        const x2 = finiteNumber(row?.x2);
        const y2 = finiteNumber(row?.y2);
        if (x1 === null || y1 === null || x2 === null || y2 === null) return "";
        return `<line x1="${px(x1).toFixed(1)}" y1="${py(y1).toFixed(1)}" x2="${px(x2).toFixed(1)}" y2="${py(y2).toFixed(1)}" class="${lineClass}" />`;
      }
      if (kind === "circle") {
        const x = finiteNumber(row?.x);
        const y = finiteNumber(row?.y);
        const r = finiteNumber(row?.r) ?? 0.05;
        if (x === null || y === null) return "";
        const radius = Math.max(3, (r / Math.max(xSpan, ySpan)) * width * 1.8);
        return `<circle cx="${px(x).toFixed(1)}" cy="${py(y).toFixed(1)}" r="${radius.toFixed(1)}" class="${circleClass}" />`;
      }
      return "";
    })
    .filter(Boolean)
    .join("");
  const title = String(space2d?.meta?.title ?? "").trim();
  const titleHtml = title ? `<div class="${titleClass}">${escapePreviewHtml(title)}</div>` : "";
  return `
    <div class="${containerClass}">
      ${titleHtml}
      <svg class="${canvasClass}" viewBox="0 0 ${width} ${height}" role="img" aria-label="space2d preview">
        ${shapeSvg}
        ${pointSvg}
      </svg>
    </div>
  `;
}

const FAMILY_PREVIEW_RENDERERS = Object.freeze({
  structure: ({ payload }) =>
    buildStructurePreviewHtml(payload, { width: 220, height: 132, maxNodes: 6 }),
  space2d: ({ payload }) =>
    buildSpace2dPreviewHtml(payload, { width: 220, height: 132 }),
  graph: ({ payload, text }) =>
    buildGraphPreviewHtml(payload, {
      width: 220,
      height: 132,
      maxSeries: 3,
      containerClass: "lesson-card-preview lesson-card-preview--graph",
      titleClass: "lesson-card-space2d-title",
      canvasClass: "lesson-card-graph-canvas",
      axisClass: "lesson-card-graph-axis",
      lineClass: "lesson-card-graph-line",
      lineVariantClasses: ["lesson-card-graph-line--a", "lesson-card-graph-line--b", "lesson-card-graph-line--c"],
    }) || buildTextPreviewHtml(text, { maxLines: 2 }),
  table: ({ payload }) =>
    buildTablePreviewHtml(payload, {
      maxRows: 3,
      maxCols: 3,
      containerClass: "lesson-card-preview lesson-card-preview--table",
      tableClass: "lesson-card-table-preview",
      titleClass: "lesson-card-space2d-title",
      metaClass: "lesson-card-table-meta",
    }),
  text: ({ text }) =>
    buildTextPreviewHtml(text, { maxLines: 3 }),
});

export function buildFamilyPreviewHtml({ family = "", payload = null, text = "" } = {}) {
  const kind = String(family ?? "").trim().toLowerCase();
  const renderer = FAMILY_PREVIEW_RENDERERS[kind];
  if (typeof renderer !== "function") return "";
  return renderer({ payload, text });
}
