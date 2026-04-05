function escapeTableHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function normalizeColumns(table, maxCols = 3) {
  const columns = Array.isArray(table?.columns) ? table.columns : [];
  return columns.slice(0, Math.max(1, Number(maxCols) || 3)).map((column, index) => ({
    key: String(column?.key ?? `col_${index}`).trim() || `col_${index}`,
    label: String(column?.label ?? column?.key ?? `col_${index}`).trim() || `col_${index}`,
  }));
}

function normalizeRows(table, maxRows = 3) {
  const rows = Array.isArray(table?.rows) ? table.rows : [];
  return rows.slice(0, Math.max(1, Number(maxRows) || 3));
}

export function summarizeTableView(table, { maxCols = 3 } = {}) {
  const columns = normalizeColumns(table, maxCols);
  const rows = Array.isArray(table?.rows) ? table.rows : [];
  if (!columns.length || !rows.length) return null;
  return {
    title: String(table?.meta?.title ?? "").trim(),
    columnCount: columns.length,
    rowCount: rows.length,
    columns: columns.map((column) => column.label),
  };
}

export function buildTableSummaryMarkdown(table, { maxCols = 3 } = {}) {
  const summary = summarizeTableView(table, { maxCols });
  if (!summary) return "";
  const lines = ["## 표 요약"];
  if (summary.title) {
    lines.push(`- 제목: ${summary.title}`);
  }
  lines.push(`- 열: ${summary.columnCount}개`);
  lines.push(`- 행: ${summary.rowCount}개`);
  if (summary.columns.length > 0) {
    lines.push("### 열");
    summary.columns.forEach((column) => lines.push(`- ${column}`));
  }
  return lines.join("\n");
}

export function buildTablePreviewHtml(
  table,
  {
    maxRows = 3,
    maxCols = 3,
    containerClass = "runtime-table-preview",
    tableClass = "runtime-table-preview-table",
    titleClass = "runtime-table-preview-title",
    metaClass = "runtime-table-preview-meta",
  } = {},
) {
  const columns = normalizeColumns(table, maxCols);
  const rows = normalizeRows(table, maxRows);
  if (!columns.length || !rows.length) return "";
  const title = String(table?.meta?.title ?? "").trim();
  const titleHtml = title ? `<div class="${titleClass}">${escapeTableHtml(title)}</div>` : "";
  const head = columns
    .map((column) => `<th>${escapeTableHtml(column.label)}</th>`)
    .join("");
  const body = rows
    .map((row) => {
      const cells = columns
        .map((column) => `<td>${escapeTableHtml(row?.[column.key] ?? "")}</td>`)
        .join("");
      return `<tr>${cells}</tr>`;
    })
    .join("");
  const rowCount = Array.isArray(table?.rows) ? table.rows.length : rows.length;
  return `
    <div class="${containerClass}">
      ${titleHtml}
      <table class="${tableClass}">
        <thead><tr>${head}</tr></thead>
        <tbody>${body}</tbody>
      </table>
      <div class="${metaClass}">열 ${columns.length}개 · 행 ${rowCount}개</div>
    </div>
  `;
}
