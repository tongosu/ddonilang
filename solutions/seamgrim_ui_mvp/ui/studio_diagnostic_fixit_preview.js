import { normalizeDiagnosticItem } from "./play_diagnostic_contract.js";

function normalizeSourceText(sourceText = "") {
  return String(sourceText ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function splitLines(sourceText = "") {
  return normalizeSourceText(sourceText).split("\n");
}

function normalizeCode(raw = "") {
  return String(raw ?? "").trim().toUpperCase();
}

function normalizePositiveInt(raw, fallback = 1) {
  const value = Number(raw);
  if (!Number.isFinite(value)) return fallback;
  return Math.max(1, Math.trunc(value));
}

function normalizeSpan(span = null) {
  const row = span && typeof span === "object" ? span : {};
  const startLine = normalizePositiveInt(row.line ?? row.row ?? row.start_line ?? row.startLine, 1);
  const startCol = normalizePositiveInt(row.column ?? row.col ?? row.start_col ?? row.startCol, 1);
  const endLine = normalizePositiveInt(row.end_line ?? row.endLine ?? startLine, startLine);
  const endCol = normalizePositiveInt(row.end_col ?? row.endCol ?? startCol, startCol);
  return {
    start_line: startLine,
    start_col: startCol,
    end_line: Math.max(startLine, endLine),
    end_col: Math.max(1, endCol),
  };
}

function findBlockHeaderColonPatch(lines, diagnostic) {
  const preferredLine = normalizeSpan(diagnostic?.span).start_line;
  const candidates = [];
  if (preferredLine >= 1 && preferredLine <= lines.length) {
    candidates.push(preferredLine - 1);
  }
  lines.forEach((_, index) => {
    if (!candidates.includes(index)) candidates.push(index);
  });
  for (const index of candidates) {
    const line = String(lines[index] ?? "");
    const match = line.match(/^(\s*)([A-Za-z0-9_가-힣]+):(\s*)\{/);
    if (!match) continue;
    const before = line;
    const after = `${match[1]}${match[2]} {`;
    return {
      kind: "replace_line",
      title: "블록 헤더 ':' 제거",
      reason: "block_header_colon",
      range: {
        start_line: index + 1,
        start_col: 1,
        end_line: index + 1,
        end_col: line.length + 1,
      },
      replacement: after,
      before,
      after,
    };
  }
  return null;
}

function findExpectedClosePatch(lines, diagnostic, closeText, reason, title) {
  const span = normalizeSpan(diagnostic?.span);
  const index = Math.max(0, Math.min(lines.length - 1, span.start_line - 1));
  const line = String(lines[index] ?? "");
  const col = Math.max(1, Math.min(line.length + 1, span.start_col));
  const before = line;
  const after = `${line.slice(0, col - 1)}${closeText}${line.slice(col - 1)}`;
  return {
    kind: "insert_text",
    title,
    reason,
    range: {
      start_line: index + 1,
      start_col: col,
      end_line: index + 1,
      end_col: col,
    },
    replacement: closeText,
    before,
    after,
  };
}

function findHashHeaderPatch(lines, diagnostic) {
  const preferredLine = normalizeSpan(diagnostic?.span).start_line;
  const candidates = [];
  if (preferredLine >= 1 && preferredLine <= lines.length) {
    candidates.push(preferredLine - 1);
  }
  lines.forEach((_, index) => {
    if (!candidates.includes(index)) candidates.push(index);
  });
  for (const index of candidates) {
    const line = String(lines[index] ?? "");
    const match = line.match(/^\s*#\s*(이름|title)\s*[:：]?\s*(.+?)\s*$/i);
    if (!match) continue;
    const title = String(match[2] ?? "").trim().replace(/^["']|["']$/g, "");
    const after = `설정 { 제목: "${title}". }.`;
    return {
      kind: "replace_line",
      title: "legacy #이름 헤더를 설정 블록으로 전환",
      reason: "hash_header",
      range: {
        start_line: index + 1,
        start_col: 1,
        end_line: index + 1,
        end_col: line.length + 1,
      },
      replacement: after,
      before: line,
      after,
    };
  }
  return null;
}

function buildPatchForDiagnostic(lines, diagnostic) {
  const code = normalizeCode(diagnostic?.technical_code ?? diagnostic?.code);
  if (code === "E_BLOCK_HEADER_COLON_FORBIDDEN" || code === "W_BLOCK_HEADER_COLON_DEPRECATED") {
    return findBlockHeaderColonPatch(lines, diagnostic);
  }
  if (code === "E_PARSE_EXPECTED_RPAREN") {
    return findExpectedClosePatch(lines, diagnostic, ")", "expected_rparen", "닫는 괄호 ')' 삽입");
  }
  if (code === "E_PARSE_EXPECTED_RBRACE") {
    return findExpectedClosePatch(lines, diagnostic, "}", "expected_rbrace", "닫는 중괄호 '}' 삽입");
  }
  if (code === "E_BLOCK_HEADER_HASH_FORBIDDEN") {
    return findHashHeaderPatch(lines, diagnostic);
  }
  return null;
}

function patchSortKey(patch) {
  return [
    Number(patch?.range?.start_line ?? 0),
    Number(patch?.range?.start_col ?? 0),
    Number(patch?.range?.end_line ?? 0),
    Number(patch?.range?.end_col ?? 0),
  ].join(":");
}

function applyLinePatches(sourceText, patches) {
  const lines = splitLines(sourceText);
  const sorted = [...patches].sort((a, b) => patchSortKey(a).localeCompare(patchSortKey(b)));
  const touched = new Set();
  sorted.forEach((patch) => {
    const lineIndex = Number(patch?.range?.start_line ?? 0) - 1;
    if (lineIndex < 0 || lineIndex >= lines.length) return;
    if (touched.has(lineIndex)) return;
    touched.add(lineIndex);
    if (String(patch?.kind ?? "") === "insert_text") {
      const line = String(lines[lineIndex] ?? "");
      const col = Math.max(1, Math.min(line.length + 1, Number(patch?.range?.start_col ?? 1)));
      lines[lineIndex] = `${line.slice(0, col - 1)}${String(patch?.replacement ?? "")}${line.slice(col - 1)}`;
    } else {
      lines[lineIndex] = String(patch?.replacement ?? patch?.after ?? "");
    }
  });
  return lines.join("\n");
}

function buildDiffText(patches = []) {
  const lines = ["--- original", "+++ preview"];
  patches.forEach((patch) => {
    const lineNo = Number(patch?.range?.start_line ?? 0);
    lines.push(`@@ L${lineNo} @@`);
    lines.push(`-${String(patch?.before ?? "")}`);
    lines.push(`+${String(patch?.after ?? patch?.replacement ?? "")}`);
  });
  return lines.join("\n");
}

function summarizeLocation(span = null) {
  const row = normalizeSpan(span);
  return `L${row.start_line}:C${row.start_col}`;
}

export function buildDiagnosticFixitPreview({
  sourceText = "",
  diagnostics = [],
} = {}) {
  const source = normalizeSourceText(sourceText);
  const lines = splitLines(source);
  const normalized = Array.isArray(diagnostics)
    ? diagnostics.map((item) => normalizeDiagnosticItem(item)).filter(Boolean)
    : [];
  const items = normalized.map((diagnostic, index) => {
    const patch = buildPatchForDiagnostic(lines, diagnostic);
    const code = String(diagnostic?.technical_code ?? diagnostic?.code ?? "").trim();
    const common = {
      index,
      code,
      severity: String(diagnostic?.severity ?? "info"),
      message: String(diagnostic?.message ?? ""),
      location: summarizeLocation(diagnostic?.span),
    };
    if (!patch) {
      return {
        ...common,
        fixit_available: false,
        reason: "unsupported_diagnostic",
      };
    }
    return {
      ...common,
      fixit_available: true,
      preview_only: true,
      patch: {
        ...patch,
        preview_only: true,
      },
    };
  });
  const patches = items
    .filter((item) => item.fixit_available && item.patch)
    .map((item) => item.patch);
  const previewText = applyLinePatches(source, patches);
  const diffText = patches.length ? buildDiffText(patches) : "";
  return {
    __종류: "studio_diagnostic_fixit_preview",
    preview_only: true,
    auto_apply: false,
    diagnostic_count: items.length,
    fixit_count: patches.length,
    unsupported_count: items.length - patches.length,
    items,
    preview_text: previewText,
    diff_text: diffText,
  };
}

export function formatDiagnosticFixitPreviewText(preview = {}) {
  const payload = preview && typeof preview === "object" ? preview : {};
  if (String(payload.__종류 ?? "") !== "studio_diagnostic_fixit_preview") {
    throw new Error("studio_fixit_preview_expected_preview");
  }
  const lines = [
    `진단\t${Number(payload.diagnostic_count ?? 0)}`,
    `수정후보\t${Number(payload.fixit_count ?? 0)}`,
    `미지원\t${Number(payload.unsupported_count ?? 0)}`,
  ];
  const items = Array.isArray(payload.items) ? payload.items : [];
  items.forEach((item) => {
    const available = item?.fixit_available ? "있음" : "없음";
    const title = item?.fixit_available ? String(item?.patch?.title ?? "") : String(item?.reason ?? "");
    lines.push(`${String(item?.code ?? "")}\t${available}\t${String(item?.location ?? "")}\t${title}`);
  });
  if (payload.diff_text) {
    lines.push("", String(payload.diff_text));
  }
  return lines.join("\n");
}
