export const VIEW_FAMILY_ALIASES = Object.freeze({
  "2d": "space2d",
  "3d": "space3d",
});

export const VIEW_FAMILY_PRIORITY = Object.freeze([
  "space2d",
  "grid2d",
  "space3d",
  "grid3d",
  "graph",
  "table",
  "text",
  "structure",
]);

export const SPATIAL_VIEW_FAMILIES = Object.freeze(["space2d", "grid2d", "space3d", "grid3d"]);
export const DOCK_VIEW_FAMILIES = Object.freeze(["graph", "table", "text", "structure"]);
export const LESSON_PREVIEW_FAMILIES = Object.freeze(["space2d", "graph", "table", "text", "structure"]);

function hasCandidateRows(value) {
  return Array.isArray(value) && value.some((row) => String(row ?? "").trim());
}

export function normalizeViewFamily(raw) {
  const family = String(raw ?? "").trim().toLowerCase();
  if (!family) return "";
  return VIEW_FAMILY_ALIASES[family] ?? family;
}

export function normalizeViewFamilyList(values) {
  const rows = Array.isArray(values) ? values : [values];
  const seen = new Set();
  const out = [];
  rows.forEach((row) => {
    const family = normalizeViewFamily(row);
    if (!family || seen.has(family)) return;
    seen.add(family);
    out.push(family);
  });
  return out;
}

export function orderViewFamiliesByPriority(values, priority = VIEW_FAMILY_PRIORITY) {
  const families = normalizeViewFamilyList(values);
  const priorityList = normalizeViewFamilyList(priority);
  const ordered = priorityList.filter((family) => families.includes(family));
  const unknown = families.filter((family) => !priorityList.includes(family));
  return [...ordered, ...unknown];
}

export function hasSpatialViewFamily(values) {
  const families = normalizeViewFamilyList(values);
  return families.some((family) => SPATIAL_VIEW_FAMILIES.includes(family));
}

export function resolveRunDockPanelOrderFromFamilies(requiredViews) {
  const families = normalizeViewFamilyList(requiredViews);
  const order = [];
  families.forEach((family) => {
    if (!DOCK_VIEW_FAMILIES.includes(family)) return;
    const panel = family === "structure" ? "text" : family;
    if (order.includes(panel)) return;
    order.push(panel);
  });
  ["graph", "table", "text"].forEach((panel) => {
    if (!order.includes(panel)) {
      order.push(panel);
    }
  });
  return order;
}

function buildLessonPreviewDescriptor(family, lesson) {
  const graphJsonCandidates = lesson?.graphCandidates ?? [];
  const graphTextCandidates = lesson?.textCandidates ?? [];
  if (family === "space2d") {
    return { family: "space2d", mode: "json", candidates: lesson?.space2dCandidates ?? [] };
  }
  if (family === "graph") {
    if (hasCandidateRows(graphJsonCandidates)) {
      return { family: "graph", mode: "json", candidates: graphJsonCandidates };
    }
    return { family: "graph", mode: "text", candidates: graphTextCandidates };
  }
  if (family === "table") {
    return { family: "table", mode: "json", candidates: lesson?.tableCandidates ?? [] };
  }
  if (family === "structure") {
    return { family: "structure", mode: "json", candidates: lesson?.structureCandidates ?? [] };
  }
  if (family === "text") {
    return { family: "text", mode: "text", candidates: lesson?.textCandidates ?? [] };
  }
  return null;
}

export function resolveLessonPreviewDescriptors(lesson) {
  const requiredFamilies = normalizeViewFamilyList(lesson?.requiredViews ?? []).filter((family) =>
    LESSON_PREVIEW_FAMILIES.includes(family),
  );
  const inferredFamilies = orderViewFamiliesByPriority(
    [
      hasCandidateRows(lesson?.space2dCandidates) ? "space2d" : "",
      hasCandidateRows(lesson?.graphCandidates) ? "graph" : "",
      hasCandidateRows(lesson?.tableCandidates) ? "table" : "",
      hasCandidateRows(lesson?.textCandidates) ? "text" : "",
      hasCandidateRows(lesson?.structureCandidates) ? "structure" : "",
    ],
    LESSON_PREVIEW_FAMILIES,
  );
  const familyOrder = requiredFamilies.length ? requiredFamilies : inferredFamilies;
  return familyOrder
    .map((family) => buildLessonPreviewDescriptor(family, lesson))
    .filter(Boolean);
}

export function lessonHasPreviewDescriptor(lesson) {
  return resolveLessonPreviewDescriptors(lesson).some((descriptor) =>
    Array.isArray(descriptor?.candidates) && descriptor.candidates.some((row) => String(row ?? "").trim()),
  );
}
