function normalizeNewlines(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

export const GUIDE_META_ALIASES = Object.freeze({
  name: Object.freeze([
    "이름",
    "name",
    "title",
    "제목",
    "표제",
    "헤드라인",
    "headline",
    "caption",
    "label",
    "guide_name",
    "guide-name",
    "가이드이름",
    "가이드_이름",
    "ガイド名",
    "タイトル",
  ]),
  desc: Object.freeze([
    "설명",
    "풀이",
    "해설",
    "설명글",
    "안내",
    "요지",
    "description",
    "desc",
    "summary",
    "요약",
    "guide_desc",
    "guide-desc",
    "guide_summary",
    "guide-summary",
    "subtitle",
    "sub_title",
    "sub-title",
    "caption_text",
    "guide_text",
    "guide-text",
    "説明",
    "解説",
  ]),
  default_observation: Object.freeze([
    "기본관찰",
    "기본관측",
    "기본관찰y",
    "기본관측y",
    "기본축y",
    "기본y축",
    "기본시리즈",
    "기본계열",
    "default_obs",
    "default-observation",
    "default_observation",
    "default_y",
    "default-y",
    "default_series",
    "default-series",
    "default_signal",
    "default-signal",
    "obs",
    "observation",
    "series",
    "y_axis",
    "y-axis",
    "yaxis",
    "既定観測",
    "既定系列",
  ]),
  default_observation_x: Object.freeze([
    "기본관찰x",
    "기본관측x",
    "기본x축",
    "기본축x",
    "기본관찰X",
    "기본관측X",
    "기본X축",
    "기본축X",
    "기본가로축",
    "default_obs_x",
    "default-observation-x",
    "default_observation_x",
    "default_x",
    "default-x",
    "default_x_axis",
    "default-x-axis",
    "default_xaxis",
    "x_axis",
    "x-axis",
    "xaxis",
    "既定観測X",
    "既定横軸",
  ]),
});

const GUIDE_META_BLOCK_NAMES = new Set(["설정", "보개", "슬기"]);

export function normalizeGuideMetaKey(text) {
  return String(text ?? "")
    .trim()
    .toLowerCase()
    .replace(/[\s_-]+/g, "");
}

const GUIDE_META_ALIAS_TO_CANON = (() => {
  const out = new Map();
  Object.entries(GUIDE_META_ALIASES).forEach(([canon, aliases]) => {
    aliases.forEach((alias) => {
      const normalized = normalizeGuideMetaKey(alias);
      if (!normalized) return;
      out.set(normalized, canon);
    });
  });
  return out;
})();

function canonicalGuideMetaKey(rawKey) {
  const normalized = normalizeGuideMetaKey(rawKey);
  return GUIDE_META_ALIAS_TO_CANON.get(normalized) ?? "";
}

function countCharInText(text, needle) {
  if (!text) return 0;
  return (String(text).match(new RegExp(`\\${needle}`, "g")) || []).length;
}

function parseGuideMetaFieldLine(rawLine) {
  const trimmed = String(rawLine ?? "").replace(/^[ \t\uFEFF]+/, "");
  if (!trimmed || trimmed.startsWith("//") || trimmed.startsWith("#")) return null;
  const colonIndex = trimmed.indexOf(":");
  if (colonIndex < 0) return null;
  const key = trimmed.slice(0, colonIndex).trim();
  if (!key) return null;
  let value = trimmed.slice(colonIndex + 1).trim();
  if (value.endsWith(".")) {
    value = value.slice(0, -1).trimEnd();
  }
  return { key, value };
}

function isGuideMetaBlockStart(rawLine) {
  const trimmed = String(rawLine ?? "").replace(/^[ \t\uFEFF]+/, "");
  if (!trimmed.includes("{")) return false;
  const header = trimmed.split("{", 1)[0] ?? "";
  const blockName = String(header ?? "")
    .trim()
    .replace(/:$/, "")
    .trim();
  return GUIDE_META_BLOCK_NAMES.has(blockName);
}

function putGuideMeta(rawMeta, meta, key, value) {
  rawMeta[key] = value;
  const canon = canonicalGuideMetaKey(key);
  if (canon && !(canon in meta)) {
    meta[canon] = value;
  }
}

export function findGuideMetaValue(rawMeta, canonKey) {
  if (!rawMeta || typeof rawMeta !== "object") return "";
  const target = String(canonKey ?? "").trim();
  if (!target) return "";
  for (const [rawKey, rawValue] of Object.entries(rawMeta)) {
    if (canonicalGuideMetaKey(rawKey) !== target) continue;
    const value = String(rawValue ?? "").trim();
    if (value) return value;
  }
  return "";
}

export function parseGuideMetaHeader(text) {
  const lines = normalizeNewlines(text).split("\n");
  const rawMeta = {};
  const meta = {};
  let idx = 0;

  while (idx < lines.length) {
    const raw = lines[idx];
    const trimmed = raw.replace(/^[ \t\uFEFF]+/, "");
    if (!trimmed) {
      idx += 1;
      continue;
    }
    if (trimmed.startsWith("#") && trimmed.includes(":")) {
      const sliced = trimmed.slice(1);
      const [keyRaw, ...rest] = sliced.split(":");
      const key = String(keyRaw ?? "").trim();
      if (!key) break;
      const value = rest.join(":").trim();
      putGuideMeta(rawMeta, meta, key, value);
      idx += 1;
      continue;
    }
    if (isGuideMetaBlockStart(trimmed)) {
      let depth = Math.max(1, countCharInText(trimmed, "{") - countCharInText(trimmed, "}"));
      idx += 1;
      while (idx < lines.length && depth > 0) {
        const blockLine = lines[idx];
        if (depth === 1) {
          const entry = parseGuideMetaFieldLine(blockLine);
          if (entry) {
            putGuideMeta(rawMeta, meta, entry.key, entry.value);
          }
        }
        depth += countCharInText(blockLine, "{");
        depth -= countCharInText(blockLine, "}");
        idx += 1;
      }
      continue;
    }
    break;
  }

  return {
    meta,
    rawMeta,
    body: lines.slice(idx).join("\n"),
  };
}
