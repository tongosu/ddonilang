import {
  buildFamilyPreviewHtml,
  buildSpace2dPreviewHtml,
  buildTextPreviewHtml,
} from "./preview_registry.js";
import { buildGraphPreviewHtml } from "./graph_preview.js";
import { buildTablePreviewHtml } from "./table_preview.js";

export function buildLessonCardTextPreviewHtml(markdown, { maxLines = 3 } = {}) {
  return buildTextPreviewHtml(markdown, {
    maxLines,
    containerClass: "lesson-card-preview lesson-card-preview--text",
  });
}

export function buildLessonCardGraphPreviewHtml(graph, { width = 220, height = 132, maxSeries = 3 } = {}) {
  return buildGraphPreviewHtml(graph, {
    width,
    height,
    maxSeries,
    containerClass: "lesson-card-preview lesson-card-preview--graph",
    titleClass: "lesson-card-space2d-title",
    canvasClass: "lesson-card-graph-canvas",
    axisClass: "lesson-card-graph-axis",
    lineClass: "lesson-card-graph-line",
    lineVariantClasses: ["lesson-card-graph-line--a", "lesson-card-graph-line--b", "lesson-card-graph-line--c"],
  });
}

export function buildLessonCardTablePreviewHtml(table, { maxRows = 3, maxCols = 3 } = {}) {
  return buildTablePreviewHtml(table, {
    maxRows,
    maxCols,
    containerClass: "lesson-card-preview lesson-card-preview--table",
    tableClass: "lesson-card-table-preview",
    titleClass: "lesson-card-space2d-title",
    metaClass: "lesson-card-table-meta",
  });
}

export function buildLessonCardSpace2dPreviewHtml(space2d, { width = 220, height = 132, maxPoints = 12 } = {}) {
  return buildSpace2dPreviewHtml(space2d, {
    width,
    height,
    maxPoints,
    containerClass: "lesson-card-preview lesson-card-preview--space2d",
    titleClass: "lesson-card-space2d-title",
    canvasClass: "lesson-card-space2d-canvas",
    pointClass: "lesson-card-space2d-point",
    lineClass: "lesson-card-space2d-line",
    circleClass: "lesson-card-space2d-circle",
  });
}

export function buildLessonCardPreviewHtml({ family = "", payload = null, text = "" } = {}) {
  return buildFamilyPreviewHtml({ family, payload, text });
}
