export function resolveAvailableFeaturedSeedIds(featuredSeedIds = [], lessonsById = null) {
  const source = Array.isArray(featuredSeedIds) ? featuredSeedIds : [];
  const out = [];
  const seen = new Set();
  source.forEach((row) => {
    const id = String(row ?? "").trim();
    if (!id || seen.has(id)) return;
    if (lessonsById && typeof lessonsById?.has === "function") {
      if (!lessonsById.has(id)) return;
    } else if (lessonsById && typeof lessonsById === "object") {
      if (!Object.prototype.hasOwnProperty.call(lessonsById, id)) return;
    }
    seen.add(id);
    out.push(id);
  });
  return out;
}

export function pickNextFeaturedSeedLaunch({
  featuredSeedIds = [],
  lessonsById = null,
  currentLessonId = "",
  cursor = -1,
} = {}) {
  const availableIds = resolveAvailableFeaturedSeedIds(featuredSeedIds, lessonsById);
  if (!availableIds.length) {
    return { availableIds, nextId: "", nextCursor: -1 };
  }

  const normalizedCurrentId = String(currentLessonId ?? "").trim();
  let currentCursor = Number.isInteger(cursor) ? cursor : -1;
  if (currentCursor < 0 || currentCursor >= availableIds.length) {
    const currentIndex = normalizedCurrentId ? availableIds.indexOf(normalizedCurrentId) : -1;
    currentCursor = currentIndex >= 0 ? currentIndex : -1;
  }

  const nextCursor = (currentCursor + 1) % availableIds.length;
  return {
    availableIds,
    nextId: availableIds[nextCursor],
    nextCursor,
  };
}

export function shouldTriggerFeaturedSeedQuickLaunch(event, { isEditableTarget = false } = {}) {
  if (!event || typeof event !== "object") return false;
  if (Boolean(isEditableTarget)) return false;
  if (!event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return false;
  if (event.repeat) return false;
  const code = String(event.code ?? "").trim();
  const key = String(event.key ?? "").trim();
  if (code === "Digit6" || key === "6") return true;
  return false;
}

export function shouldTriggerFeaturedSeedQuickPreset(
  event,
  { isEditableTarget = false, isBrowseScreen = false } = {},
) {
  if (!event || typeof event !== "object") return false;
  if (!Boolean(isBrowseScreen)) return false;
  if (Boolean(isEditableTarget)) return false;
  if (!event.altKey || !event.shiftKey || event.ctrlKey || event.metaKey) return false;
  if (event.repeat) return false;
  const code = String(event.code ?? "").trim();
  const key = String(event.key ?? "").trim();
  if (code === "Digit6" || key === "6") return true;
  return false;
}
