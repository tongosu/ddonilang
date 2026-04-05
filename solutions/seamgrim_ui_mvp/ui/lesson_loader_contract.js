import { normalizeViewFamilyList } from "./view_family_contract.js";

function normalizeHeaderKey(raw) {
  return String(raw ?? "")
    .trim()
    .toLowerCase()
    .replace(/[\s_-]+/g, "");
}

function parseHeaderViewFamilies(rawValue) {
  const text = String(rawValue ?? "").trim();
  if (!text) return [];
  const inner = text.startsWith("[") && text.endsWith("]") ? text.slice(1, -1) : text;
  const rows = inner
    .split(",")
    .map((item) => String(item ?? "").trim())
    .filter(Boolean)
    .map((item) => item.replace(/^"(.*)"$/, "$1").replace(/^'(.*)'$/, "$1").trim())
    .filter(Boolean);
  return normalizeViewFamilyList(rows);
}

export function parseLessonDdnMetaHeader(ddnText, { scanLineLimit = 40 } = {}) {
  const lines = String(ddnText ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  const limit = Math.max(1, Number(scanLineLimit) || 40);
  let name = "";
  let desc = "";
  let requiredViews = [];
  for (let i = 0; i < lines.length && i < limit; i += 1) {
    const trimmed = String(lines[i] ?? "").trim();
    if (!trimmed) continue;
    if (trimmed.startsWith("//")) continue;
    if (!trimmed.startsWith("#")) break;
    const match = trimmed.match(/^#\s*([^:]+)\s*:\s*(.*)$/);
    if (!match) continue;
    const key = normalizeHeaderKey(match[1]);
    const value = String(match[2] ?? "").trim();
    if (!value) continue;
    if (!name && (key === "이름" || key === "name" || key === "title")) {
      name = value;
      continue;
    }
    if (!desc && (key === "설명" || key === "desc" || key === "description")) {
      desc = value;
      continue;
    }
    if (
      requiredViews.length === 0 &&
      (key === "requiredviews" || key === "requiredview" || key === "필수보기" || key === "보개")
    ) {
      requiredViews = parseHeaderViewFamilies(value);
      continue;
    }
  }
  return {
    name,
    desc,
    requiredViews,
    required_views: requiredViews,
    hasAny: Boolean(name || desc || requiredViews.length > 0),
  };
}

export function resolveLessonDisplayMeta({
  baseTitle = "",
  baseDescription = "",
  tomlMeta = null,
  ddnMetaHeader = null,
} = {}) {
  const meta = tomlMeta && typeof tomlMeta === "object" ? tomlMeta : {};
  const ddnMeta = ddnMetaHeader && typeof ddnMetaHeader === "object" ? ddnMetaHeader : {};
  const title = String(meta.title ?? "").trim() || String(ddnMeta.name ?? "").trim() || String(baseTitle ?? "").trim();
  const description =
    String(meta.description ?? "").trim() ||
    String(ddnMeta.desc ?? "").trim() ||
    String(baseDescription ?? "").trim();
  return {
    title,
    description,
  };
}

export function buildLessonSelectionSnapshot(lesson) {
  const row = lesson && typeof lesson === "object" ? lesson : {};
  const ddnMeta = row.ddnMetaHeader && typeof row.ddnMetaHeader === "object" ? row.ddnMetaHeader : {};
  return {
    lesson_id: String(row.id ?? "").trim(),
    title: String(row.title ?? "").trim(),
    required_views: normalizeViewFamilyList(
      row.requiredViews ?? row.required_views ?? ddnMeta.requiredViews ?? ddnMeta.required_views ?? [],
    ),
    ddn_meta: {
      name: String(ddnMeta.name ?? "").trim(),
      desc: String(ddnMeta.desc ?? "").trim(),
    },
  };
}
