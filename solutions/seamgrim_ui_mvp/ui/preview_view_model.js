function normalizeFamily(raw) {
  return String(raw ?? "").trim().toLowerCase();
}

function familyLabel(family) {
  const kind = normalizeFamily(family);
  if (kind === "space2d") return "공간";
  if (kind === "graph") return "그래프";
  if (kind === "table") return "표";
  if (kind === "structure") return "구조";
  if (kind === "text") return "텍스트";
  return kind || "보기";
}

export function buildPreviewViewModel(collection, { sourceId = "" } = {}) {
  if (!collection?.html) return null;
  const previewFamilies = Array.isArray(collection?.families)
    ? collection.families.map((item) => normalizeFamily(item)).filter(Boolean)
    : [];
  const primaryFamily = normalizeFamily(collection?.family ?? collection?.primary?.family ?? "");
  const primaryMode = String(collection?.mode ?? collection?.primary?.mode ?? "").trim().toLowerCase();
  const previewCount = Math.max(0, Math.trunc(Number(collection?.count) || 0));
  const tooltip = String(collection?.tooltip ?? "").trim();
  const title = String(collection?.title ?? "").trim();
  return {
    html: String(collection.html ?? ""),
    title,
    tooltip,
    summaryText: tooltip,
    primaryFamily,
    primaryMode,
    primaryLabel: familyLabel(primaryFamily),
    previewCount,
    previewFamilies,
    previewFamiliesText: previewFamilies.join(","),
    headerText: primaryFamily ? `${familyLabel(primaryFamily)} 미리보기` : "미리보기",
    sourceId: String(sourceId ?? "").trim(),
    collection,
  };
}

export function applyPreviewViewModelMetadata(target, viewModel) {
  if (!target) return;
  if (!target.dataset || typeof target.dataset !== "object") {
    target.dataset = {};
  }
  if (!viewModel) {
    delete target.dataset.previewFamily;
    delete target.dataset.previewMode;
    delete target.dataset.previewCount;
    delete target.dataset.previewFamilies;
    delete target.dataset.previewTitle;
    delete target.dataset.previewHeader;
    delete target.dataset.previewSummary;
    target.title = "";
    return;
  }
  target.dataset.previewFamily = String(viewModel.primaryFamily ?? "");
  target.dataset.previewMode = String(viewModel.primaryMode ?? "");
  target.dataset.previewCount = String(viewModel.previewCount ?? 0);
  target.dataset.previewFamilies = String(viewModel.previewFamiliesText ?? "");
  target.dataset.previewTitle = String(viewModel.title ?? "");
  target.dataset.previewHeader = String(viewModel.headerText ?? "");
  target.dataset.previewSummary = String(viewModel.summaryText ?? "");
  target.title = String(viewModel.tooltip ?? "");
}
