function buildPathCandidates(path) {
  const normalized = String(path ?? "").replace(/\\/g, "/").replace(/^\.\//, "").trim();
  if (!normalized) return [];
  if (/^https?:\/\//i.test(normalized)) return [normalized];
  const primary = normalized.startsWith("/") ? normalized : `/${normalized}`;
  return [primary];
}

function normalizePreviewCandidateList(pathCandidates) {
  const list = Array.isArray(pathCandidates)
    ? pathCandidates.flatMap((item) => buildPathCandidates(item))
    : buildPathCandidates(pathCandidates);
  return Array.from(new Set(list));
}

function buildPreviewCacheKey(descriptor) {
  const family = String(descriptor?.family ?? "").trim().toLowerCase();
  const mode = String(descriptor?.mode ?? "").trim().toLowerCase();
  const candidates = Array.isArray(descriptor?.candidates) ? descriptor.candidates.map((row) => String(row ?? "").trim()) : [];
  return `${family}:${mode}:${candidates.join("|")}`;
}

async function fetchPreviewPayloadFromCandidates(candidates, mode, fetchImpl) {
  for (const candidate of candidates) {
    try {
      const response = await fetchImpl(candidate, { cache: "no-cache" });
      if (!response.ok) continue;
      if (mode === "text") {
        return await response.text();
      }
      return await response.json();
    } catch (_) {
      // continue
    }
  }
  return mode === "text" ? "" : null;
}

export async function fetchPreviewPayload(descriptor, { cache = null, fetchImpl = globalThis.fetch } = {}) {
  const candidates = normalizePreviewCandidateList(descriptor?.candidates ?? []);
  if (!candidates.length || typeof fetchImpl !== "function") {
    return descriptor?.mode === "text" ? "" : null;
  }
  const cacheKey = buildPreviewCacheKey({ ...descriptor, candidates });
  if (cache && cache.has(cacheKey)) {
    return cache.get(cacheKey);
  }
  const payload = await fetchPreviewPayloadFromCandidates(candidates, descriptor?.mode, fetchImpl);
  if (cache) {
    cache.set(cacheKey, payload ?? null);
  }
  return payload;
}

export async function resolvePreviewHtmlFromDescriptors(
  descriptors,
  {
    cache = null,
    fetchImpl = globalThis.fetch,
    renderPreview = () => "",
  } = {},
) {
  const rows = Array.isArray(descriptors) ? descriptors : [];
  for (const descriptor of rows) {
    const payload = await fetchPreviewPayload(descriptor, { cache, fetchImpl });
    const html = renderPreview({
      family: descriptor?.family,
      payload: descriptor?.mode === "json" ? payload : null,
      text: descriptor?.mode === "text" ? payload : "",
    });
    if (!html) continue;
    return { descriptor, html, payload };
  }
  return null;
}
