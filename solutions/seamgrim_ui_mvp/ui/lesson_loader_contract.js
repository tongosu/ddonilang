import { normalizeViewFamilyList } from "./view_family_contract.js";

const CURRICULUM_META_ALIASES = Object.freeze({
  과목: "subject",
  학년군: "grade_band",
  단원: "unit",
  차시: "lesson",
  난이도: "difficulty",
  학습목표: "learning_goals",
  핵심개념: "core_concepts",
  선수개념: "prerequisites",
  오개념: "misconceptions",
  허용조작: "allowed_controls",
  필수계기판: "required_views",
});

function normalizeHeaderKey(raw) {
  return String(raw ?? "")
    .trim()
    .toLowerCase()
    .replace(/[\s_-]+/g, "");
}

function stripTomlString(raw) {
  const text = String(raw ?? "").trim();
  if (text.length >= 2 && text.startsWith('"') && text.endsWith('"')) {
    return text.slice(1, -1).replace(/\\"/g, '"');
  }
  return text;
}

function splitTomlItems(raw) {
  const text = String(raw ?? "");
  const out = [];
  let current = "";
  let inString = false;
  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];
    if (ch === '"' && text[i - 1] !== "\\") {
      inString = !inString;
      current += ch;
      continue;
    }
    if (ch === "," && !inString) {
      const item = current.trim();
      if (item) out.push(item);
      current = "";
      continue;
    }
    current += ch;
  }
  const tail = current.trim();
  if (tail) out.push(tail);
  return out;
}

function parseTomlInlineTable(rawValue) {
  const text = String(rawValue ?? "").trim();
  if (!text.startsWith("{") || !text.endsWith("}")) return null;
  const inner = text.slice(1, -1).trim();
  const out = {};
  splitTomlItems(inner).forEach((item) => {
    const match = item.match(/^(?:"([^"]+)"|([A-Za-z0-9_]+))\s*=\s*(.+)$/);
    if (!match) return;
    const key = String(match[1] ?? match[2] ?? "").trim();
    if (!key) return;
    out[key] = parseTomlValue(match[3]);
  });
  return out;
}

function parseTomlValue(rawValue) {
  const text = String(rawValue ?? "").trim();
  if (text.startsWith("[") && text.endsWith("]")) {
    const inner = text.slice(1, -1).trim();
    return inner ? splitTomlItems(inner).map(stripTomlString).filter(Boolean) : [];
  }
  const table = parseTomlInlineTable(text);
  if (table) return table;
  return stripTomlString(text);
}

function applyCurriculumAliases(out) {
  Object.entries(CURRICULUM_META_ALIASES).forEach(([alias, canonical]) => {
    if (out[canonical] === undefined && out[alias] !== undefined) {
      out[canonical] = out[alias];
    }
  });
  if (out.grade === undefined && out.grade_band !== undefined) {
    out.grade = out.grade_band;
  }
  if (out.required_views !== undefined) {
    out.required_views = normalizeViewFamilyList(out.required_views);
  }
  return out;
}

export function parseTomlMeta(text) {
  if (!text) return {};
  const out = {};
  const lines = String(text).replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  lines.forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) return;
    const match = trimmed.match(/^(?:"([^"]+)"|([A-Za-z0-9_]+))\s*=\s*(.+)$/);
    if (!match) return;
    const key = String(match[1] ?? match[2] ?? "").trim();
    if (!key) return;
    out[key] = parseTomlValue(match[3]);
  });
  return applyCurriculumAliases(out);
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
