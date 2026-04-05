import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/preview_view_model.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const { buildPreviewViewModel, applyPreviewViewModelMetadata } = mod;

  assert(typeof buildPreviewViewModel === "function", "preview view model: buildPreviewViewModel export");
  assert(typeof applyPreviewViewModelMetadata === "function", "preview view model: applyPreviewViewModelMetadata export");

  const viewModel = buildPreviewViewModel({
    html: '<div class="lesson-card-preview-card"></div>',
    family: "graph",
    mode: "json",
    title: "속도 변화",
    tooltip: "속도 변화 · 계열 1개 · 점 3개",
    count: 1,
    families: ["graph"],
  }, { sourceId: "graph_case" });
  assert(viewModel?.primaryFamily === "graph", "preview view model: family");
  assert(viewModel?.primaryMode === "json", "preview view model: mode");
  assert(viewModel?.previewCount === 1, "preview view model: count");
  assert(viewModel?.previewFamiliesText === "graph", "preview view model: families text");
  assert(viewModel?.headerText === "그래프 미리보기", "preview view model: header text");
  assert(viewModel?.sourceId === "graph_case", "preview view model: source id");

  const target = { dataset: {}, title: "" };
  applyPreviewViewModelMetadata(target, viewModel);
  assert(target.dataset.previewFamily === "graph", "preview view model: dataset family");
  assert(target.dataset.previewMode === "json", "preview view model: dataset mode");
  assert(target.dataset.previewCount === "1", "preview view model: dataset count");
  assert(target.dataset.previewFamilies === "graph", "preview view model: dataset families");
  assert(target.dataset.previewTitle === "속도 변화", "preview view model: dataset title");
  assert(target.dataset.previewHeader === "그래프 미리보기", "preview view model: dataset header");
  assert(target.dataset.previewSummary === "속도 변화 · 계열 1개 · 점 3개", "preview view model: dataset summary");
  assert(target.title === "속도 변화 · 계열 1개 · 점 3개", "preview view model: tooltip");

  applyPreviewViewModelMetadata(target, null);
  assert(!("previewFamily" in target.dataset), "preview view model: clears family");
  assert(!("previewMode" in target.dataset), "preview view model: clears mode");
  assert(!("previewCount" in target.dataset), "preview view model: clears count");
  assert(!("previewFamilies" in target.dataset), "preview view model: clears families");
  assert(!("previewTitle" in target.dataset), "preview view model: clears title");
  assert(!("previewHeader" in target.dataset), "preview view model: clears header");
  assert(!("previewSummary" in target.dataset), "preview view model: clears summary");
  assert(target.title === "", "preview view model: clears tooltip");

  console.log("seamgrim preview view model runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
