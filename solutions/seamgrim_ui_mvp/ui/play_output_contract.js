const PLAY_TABS = new Set(["diag", "obs", "mirror"]);

export function normalizePlayTab(raw) {
  const tab = String(raw ?? "").trim().toLowerCase();
  return PLAY_TABS.has(tab) ? tab : "diag";
}

export function resolvePlayTabVisibility({
  hasDiagnostics = false,
  hasObservation = false,
  hasMirror = false,
} = {}) {
  return {
    diag: true,
    obs: Boolean(hasObservation),
    mirror: Boolean(hasMirror),
    hasDiagnostics: Boolean(hasDiagnostics),
  };
}

export function resolvePreferredPlayTab({
  hasError = false,
  hasDiagnostics = false,
  hasObservation = false,
  hasMirror = false,
} = {}) {
  if (Boolean(hasError)) return "diag";
  if (Boolean(hasObservation)) return "obs";
  if (Boolean(hasDiagnostics)) return "diag";
  if (Boolean(hasMirror)) return "mirror";
  return "diag";
}

export function resolvePlayActiveTab(currentTab, summary = {}, { preserveCurrent = false } = {}) {
  const visible = resolvePlayTabVisibility(summary);
  const normalizedCurrent = normalizePlayTab(currentTab);
  if (preserveCurrent && visible[normalizedCurrent]) {
    return normalizedCurrent;
  }
  const preferred = resolvePreferredPlayTab(summary);
  if (visible[preferred]) return preferred;
  if (visible.diag) return "diag";
  if (visible.obs) return "obs";
  if (visible.mirror) return "mirror";
  return "diag";
}
