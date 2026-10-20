function escapeGraphHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function finiteNumber(value) {
  const n = Number(value);
  return Number.isFinite(n) ? n : null;
}

export function summarizeGraphView(graph, { seriesLimit = 3 } = {}) {
  if (!graph || typeof graph !== "object") return null;
  const series = Array.isArray(graph?.series) ? graph.series : [];
  const usable = series
    .map((row, index) => {
      const points = Array.isArray(row?.points)
        ? row.points
            .map((point) => {
              const x = finiteNumber(point?.x);
              const y = finiteNumber(point?.y);
              return x === null || y === null ? null : { x, y };
            })
            .filter(Boolean)
        : [];
      if (!points.length) return null;
      return {
        id: String(row?.id ?? row?.label ?? `series_${index}`).trim() || `series_${index}`,
        points,
      };
    })
    .filter(Boolean);
  if (!usable.length) return null;
  return {
    title: String(graph?.meta?.title ?? "").trim(),
    seriesCount: usable.length,
    pointCount: usable.reduce((sum, row) => sum + row.points.length, 0),
    seriesLabels: usable.slice(0, Math.max(1, Number(seriesLimit) || 3)).map((row) => row.id),
  };
}

export function buildGraphSummaryMarkdown(graph, { seriesLimit = 3 } = {}) {
  const summary = summarizeGraphView(graph, { seriesLimit });
  if (!summary) return "";
  const lines = ["## 그래프 요약"];
  if (summary.title) {
    lines.push(`- 제목: ${summary.title}`);
  }
  lines.push(`- 계열: ${summary.seriesCount}개`);
  lines.push(`- 점: ${summary.pointCount}개`);
  if (summary.seriesLabels.length) {
    lines.push("### 계열");
    summary.seriesLabels.forEach((row) => lines.push(`- ${row}`));
  }
  return lines.join("\n");
}

export function buildGraphPreviewHtml(
  graph,
  {
    width = 220,
    height = 132,
    maxSeries = 3,
    containerClass = "runtime-graph-preview",
    titleClass = "runtime-graph-title",
    canvasClass = "runtime-graph-canvas",
    axisClass = "runtime-graph-axis",
    lineClass = "runtime-graph-line",
    lineVariantClasses = ["runtime-graph-line--a", "runtime-graph-line--b", "runtime-graph-line--c"],
  } = {},
) {
  if (!graph || typeof graph !== "object") return "";
  const series = Array.isArray(graph?.series) ? graph.series.slice(0, Math.max(1, Number(maxSeries) || 3)) : [];
  const usable = series
    .map((row, index) => {
      const points = Array.isArray(row?.points)
        ? row.points
            .map((point) => {
              const x = finiteNumber(point?.x);
              const y = finiteNumber(point?.y);
              return x === null || y === null ? null : { x, y };
            })
            .filter(Boolean)
        : [];
      if (!points.length) return null;
      return {
        id: String(row?.id ?? row?.label ?? `series_${index}`).trim() || `series_${index}`,
        points,
      };
    })
    .filter(Boolean);
  if (!usable.length) return "";

  const margin = 14;
  const xValues = [];
  const yValues = [];
  usable.forEach((row) => {
    row.points.forEach((point) => {
      xValues.push(point.x);
      yValues.push(point.y);
    });
  });
  const axis = graph?.axis ?? {};
  const xMin = finiteNumber(axis?.x_min) ?? Math.min(...xValues);
  const xMax = finiteNumber(axis?.x_max) ?? Math.max(...xValues);
  const yMin = finiteNumber(axis?.y_min) ?? Math.min(...yValues);
  const yMax = finiteNumber(axis?.y_max) ?? Math.max(...yValues);
  const xSpan = Math.max(1e-6, xMax - xMin);
  const ySpan = Math.max(1e-6, yMax - yMin);
  const px = (x) => margin + ((x - xMin) / xSpan) * (width - margin * 2);
  const py = (y) => margin + (1 - ((y - yMin) / ySpan)) * (height - margin * 2);
  const palette = Array.isArray(lineVariantClasses) && lineVariantClasses.length
    ? lineVariantClasses
    : [lineClass];
  const pathSvg = usable
    .map((row, index) => {
      if (row.points.length < 2) return "";
      const points = row.points
        .map((point, pointIndex) => `${pointIndex === 0 ? "M" : "L"} ${px(point.x).toFixed(1)} ${py(point.y).toFixed(1)}`)
        .join(" ");
      const variantClass = String(palette[index % palette.length] ?? "").trim();
      const classes = [lineClass, variantClass].filter(Boolean).join(" ");
      return `<path d="${points}" class="${classes}" />`;
    })
    .join("");
  const pointSvg = usable
    .map((row, index) => {
      const variantClass = String(palette[index % palette.length] ?? "").trim();
      const suffix = variantClass.replace("runtime-graph-line", "runtime-graph-point");
      const classes = ["runtime-graph-point", suffix].filter(Boolean).join(" ");
      return row.points
        .map(
          (point) =>
            `<circle cx="${px(point.x).toFixed(1)}" cy="${py(point.y).toFixed(1)}" r="3.4" class="${classes}" />`,
        )
        .join("");
    })
    .join("");
  const title = String(graph?.meta?.title ?? "").trim();
  const titleHtml = title ? `<div class="${titleClass}">${escapeGraphHtml(title)}</div>` : "";
  return `
    <div class="${containerClass}">
      ${titleHtml}
      <svg class="${canvasClass}" viewBox="0 0 ${width} ${height}" role="img" aria-label="graph preview">
        <line x1="${margin}" y1="${height - margin}" x2="${width - margin}" y2="${height - margin}" class="${axisClass}" />
        <line x1="${margin}" y1="${margin}" x2="${margin}" y2="${height - margin}" class="${axisClass}" />
        ${pathSvg}
        ${pointSvg}
      </svg>
    </div>
  `;
}
