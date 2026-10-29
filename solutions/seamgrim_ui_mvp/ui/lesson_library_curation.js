function normalizeText(value) {
  return String(value ?? "").trim();
}

function normalizeStringArray(values) {
  if (!Array.isArray(values)) return [];
  const out = [];
  values.forEach((value) => {
    const text = normalizeText(value);
    if (text) out.push(text);
  });
  return out;
}

function incrementCount(counts, raw) {
  const key = normalizeText(raw) || "unknown";
  counts.set(key, (counts.get(key) ?? 0) + 1);
}

function sortedCountObject(counts) {
  return Object.fromEntries(Array.from(counts.entries()).sort(([a], [b]) => a.localeCompare(b, "ko")));
}

function resolveAllowlistIds(allowlist) {
  if (Array.isArray(allowlist)) return normalizeStringArray(allowlist);
  if (allowlist instanceof Set) {
    const rawIds = Array.isArray(allowlist.rawIds) ? allowlist.rawIds : Array.from(allowlist);
    return normalizeStringArray(rawIds);
  }
  if (allowlist && typeof allowlist === "object") {
    return normalizeStringArray(allowlist.lesson_ids ?? allowlist.lessonIds ?? allowlist.rawIds);
  }
  return [];
}

function resolveAllowlistMode(allowlist) {
  if (allowlist instanceof Set) return normalizeText(allowlist.mode) || "unknown";
  if (allowlist && typeof allowlist === "object") return normalizeText(allowlist.mode) || "unknown";
  return "unknown";
}

function resolveCatalogMode(catalogMode) {
  return normalizeText(catalogMode) || "unknown";
}

function resolveDuplicateIds(ids) {
  const seen = new Set();
  const duplicates = [];
  const emitted = new Set();
  ids.forEach((id) => {
    if (!seen.has(id)) {
      seen.add(id);
      return;
    }
    if (!emitted.has(id)) {
      emitted.add(id);
      duplicates.push(id);
    }
  });
  return duplicates;
}

function formatCountObject(counts) {
  return Object.entries(counts)
    .map(([key, value]) => `${key}=${value}`)
    .join("|");
}

export function buildLessonLibraryCurationSnapshot({ lessons = [], allowlist = [], catalogMode = "" } = {}) {
  const lessonRows = Array.isArray(lessons) ? lessons : [];
  const lessonsById = new Map();
  lessonRows.forEach((lesson) => {
    const id = normalizeText(lesson?.id);
    if (id && !lessonsById.has(id)) lessonsById.set(id, lesson);
  });

  const allowlistIds = resolveAllowlistIds(allowlist);
  const duplicateAllowlistIds = resolveDuplicateIds(allowlistIds);
  const missingAllowlistIds = allowlistIds.filter((id) => !lessonsById.has(id));
  const activeLessonIds = allowlistIds.filter((id) => lessonsById.has(id));
  const activeIdSet = new Set(allowlistIds);
  const extraVisibleIds = lessonRows
    .map((lesson) => normalizeText(lesson?.id))
    .filter((id) => id && !activeIdSet.has(id));

  const subjectCounts = new Map();
  const gradeCounts = new Map();
  const sourceCounts = new Map();
  const requiredViewCounts = new Map();
  activeLessonIds.forEach((id) => {
    const lesson = lessonsById.get(id) ?? {};
    incrementCount(subjectCounts, lesson.subject);
    incrementCount(gradeCounts, lesson.grade);
    incrementCount(sourceCounts, lesson.source);
    normalizeStringArray(lesson.requiredViews ?? lesson.required_views).forEach((view) => {
      incrementCount(requiredViewCounts, view);
    });
  });

  return {
    schema: "seamgrim.lesson_library_curation.v1",
    mode: resolveAllowlistMode(allowlist),
    catalog_mode: resolveCatalogMode(catalogMode),
    allowlist_count: allowlistIds.length,
    active_count: activeLessonIds.length,
    visible_count: lessonRows.length,
    missing_allowlist_ids: missingAllowlistIds,
    duplicate_allowlist_ids: duplicateAllowlistIds,
    extra_visible_ids: extraVisibleIds,
    active_lesson_ids: activeLessonIds,
    subject_counts: sortedCountObject(subjectCounts),
    grade_counts: sortedCountObject(gradeCounts),
    source_counts: sortedCountObject(sourceCounts),
    required_view_counts: sortedCountObject(requiredViewCounts),
  };
}

export function formatLessonLibraryCurationText(snapshot = {}) {
  const row = snapshot && typeof snapshot === "object" ? snapshot : {};
  const lines = [
    `schema\t${normalizeText(row.schema)}`,
    `mode\t${normalizeText(row.mode)}`,
    `catalog_mode\t${normalizeText(row.catalog_mode)}`,
    `allowlist_count\t${Number(row.allowlist_count ?? 0)}`,
    `active_count\t${Number(row.active_count ?? 0)}`,
    `visible_count\t${Number(row.visible_count ?? 0)}`,
    `missing_allowlist_ids\t${normalizeStringArray(row.missing_allowlist_ids).join("|")}`,
    `duplicate_allowlist_ids\t${normalizeStringArray(row.duplicate_allowlist_ids).join("|")}`,
    `extra_visible_ids\t${normalizeStringArray(row.extra_visible_ids).join("|")}`,
    `subject_counts\t${formatCountObject(row.subject_counts ?? {})}`,
    `grade_counts\t${formatCountObject(row.grade_counts ?? {})}`,
    `source_counts\t${formatCountObject(row.source_counts ?? {})}`,
    `required_view_counts\t${formatCountObject(row.required_view_counts ?? {})}`,
  ];
  return lines.join("\n");
}
