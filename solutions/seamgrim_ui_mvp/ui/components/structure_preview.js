import { summarizeStructureView } from "../seamgrim_runtime_state.js";

function escapeStructureHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

export function buildStructureSummaryMarkdown(structureView, { sampleLimit = 3 } = {}) {
  const summary = summarizeStructureView(structureView, { sampleLimit });
  if (!summary) return "";
  const lines = ["## 구조 요약"];
  if (summary.title) {
    lines.push(`- 제목: ${summary.title}`);
  }
  lines.push(`- 노드: ${summary.nodeCount}개`);
  lines.push(`- 간선: ${summary.edgeCount}개`);
  if (summary.directedCount > 0) {
    lines.push(`- 방향 간선: ${summary.directedCount}개`);
  }
  if (summary.nodeSamples.length > 0) {
    lines.push("### 노드");
    summary.nodeSamples.forEach((row) => lines.push(`- ${row}`));
  }
  if (summary.edgeSamples.length > 0) {
    lines.push("### 간선");
    summary.edgeSamples.forEach((row) => lines.push(`- ${row}`));
  }
  return lines.join("\n");
}

export function buildStructurePreviewHtml(structureView, { width = 280, height = 164, maxNodes = 8 } = {}) {
  const summary = summarizeStructureView(structureView, { sampleLimit: 3 });
  if (!summary) return "";
  const nodes = Array.isArray(structureView?.nodes) ? structureView.nodes.slice(0, Math.max(2, Number(maxNodes) || 8)) : [];
  if (!nodes.length) return "";
  const edges = Array.isArray(structureView?.edges) ? structureView.edges : [];
  const margin = 24;
  const innerWidth = Math.max(80, Number(width) - margin * 2);
  const innerHeight = Math.max(60, Number(height) - margin * 2);
  const hasCoords = nodes.every((row) => Number.isFinite(Number(row?.x)) && Number.isFinite(Number(row?.y)));
  const xs = hasCoords ? nodes.map((row) => Number(row.x)) : [];
  const ys = hasCoords ? nodes.map((row) => Number(row.y)) : [];
  const xMin = hasCoords ? Math.min(...xs) : 0;
  const xMax = hasCoords ? Math.max(...xs) : 1;
  const yMin = hasCoords ? Math.min(...ys) : 0;
  const yMax = hasCoords ? Math.max(...ys) : 1;
  const xSpan = hasCoords ? Math.max(1e-6, xMax - xMin) : 1;
  const ySpan = hasCoords ? Math.max(1e-6, yMax - yMin) : 1;
  const positions = new Map();
  nodes.forEach((row, index) => {
    const id = String(row?.id ?? row?.label ?? `node_${index}`).trim() || `node_${index}`;
    let px = 0;
    let py = 0;
    if (hasCoords) {
      px = margin + ((Number(row.x) - xMin) / xSpan) * innerWidth;
      py = margin + ((Number(row.y) - yMin) / ySpan) * innerHeight;
    } else {
      const angle = (Math.PI * 2 * index) / nodes.length - Math.PI / 2;
      px = margin + innerWidth / 2 + Math.cos(angle) * (innerWidth * 0.38);
      py = margin + innerHeight / 2 + Math.sin(angle) * (innerHeight * 0.34);
    }
    positions.set(id, { x: px, y: py, label: String(row?.label ?? row?.id ?? id).trim() || id });
  });
  const lineSvg = edges
    .map((row) => {
      const from = positions.get(String(row?.from ?? "").trim());
      const to = positions.get(String(row?.to ?? "").trim());
      if (!from || !to) return "";
      return `<line x1="${from.x.toFixed(1)}" y1="${from.y.toFixed(1)}" x2="${to.x.toFixed(1)}" y2="${to.y.toFixed(1)}" class="runtime-structure-edge" />`;
    })
    .filter(Boolean)
    .join("");
  const nodeSvg = Array.from(positions.values())
    .map((row) => {
      const safeLabel = escapeStructureHtml(row.label);
      return `<g class="runtime-structure-node"><circle cx="${row.x.toFixed(1)}" cy="${row.y.toFixed(1)}" r="10" class="runtime-structure-node-dot" /><text x="${row.x.toFixed(1)}" y="${(row.y + 24).toFixed(1)}" text-anchor="middle" class="runtime-structure-node-label">${safeLabel}</text></g>`;
    })
    .join("");
  const title = summary.title ? `<div class="runtime-structure-title">${escapeStructureHtml(summary.title)}</div>` : "";
  const meta = `<div class="runtime-structure-meta">노드 ${summary.nodeCount}개 · 간선 ${summary.edgeCount}개</div>`;
  return `
    <div class="runtime-structure-preview">
      ${title}
      <svg class="runtime-structure-canvas" viewBox="0 0 ${width} ${height}" role="img" aria-label="structure preview">
        ${lineSvg}
        ${nodeSvg}
      </svg>
      ${meta}
    </div>
  `;
}
