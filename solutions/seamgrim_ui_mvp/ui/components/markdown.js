function escapeHtml(text) {
  return String(text ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/\"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

export function markdownToHtml(markdown) {
  const lines = String(markdown ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  const blocks = [];
  let listOpen = false;

  const closeList = () => {
    if (!listOpen) return;
    blocks.push("</ul>");
    listOpen = false;
  };

  lines.forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed) {
      closeList();
      return;
    }
    if (/^#{1,3}\s+/.test(trimmed)) {
      closeList();
      const level = Math.min(3, (trimmed.match(/^#+/)?.[0].length ?? 1));
      const body = trimmed.replace(/^#{1,3}\s+/, "");
      blocks.push(`<h${level}>${escapeHtml(body)}</h${level}>`);
      return;
    }
    if (/^[-*]\s+/.test(trimmed)) {
      if (!listOpen) {
        blocks.push("<ul>");
        listOpen = true;
      }
      blocks.push(`<li>${escapeHtml(trimmed.replace(/^[-*]\s+/, ""))}</li>`);
      return;
    }
    closeList();
    blocks.push(`<p>${escapeHtml(trimmed)}</p>`);
  });

  closeList();
  return blocks.join("\n");
}
