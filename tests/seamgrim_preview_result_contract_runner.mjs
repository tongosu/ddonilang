import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/preview_result_contract.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const {
    buildPreviewResult,
    buildFamilyPreviewResult,
    buildPreviewResultCollection,
    buildPreviewResultCollectionHtml,
    buildPreviewSummaryStripHtml,
    pickPrimaryPreviewResult,
    wrapPreviewResultHtml,
  } = mod;

  assert(typeof buildPreviewResult === "function", "preview result contract: buildPreviewResult export");
  assert(typeof buildFamilyPreviewResult === "function", "preview result contract: buildFamilyPreviewResult export");
  assert(typeof buildPreviewResultCollection === "function", "preview result contract: buildPreviewResultCollection export");
  assert(typeof buildPreviewResultCollectionHtml === "function", "preview result contract: buildPreviewResultCollectionHtml export");
  assert(typeof buildPreviewSummaryStripHtml === "function", "preview result contract: buildPreviewSummaryStripHtml export");
  assert(typeof pickPrimaryPreviewResult === "function", "preview result contract: pickPrimaryPreviewResult export");
  assert(typeof wrapPreviewResultHtml === "function", "preview result contract: wrapPreviewResultHtml export");

  const graphResult = buildFamilyPreviewResult({
    family: "graph",
    mode: "runtime",
    payload: {
      meta: { title: "속도 변화" },
      series: [{ id: "v", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }, { x: 2, y: 1.5 }] }],
    },
    html: '<div class="runtime-graph-preview"></div>',
  });
  assert(graphResult?.family === "graph", "preview result contract: graph family");
  assert(graphResult?.mode === "runtime", "preview result contract: graph mode");
  assert(graphResult?.title === "속도 변화", "preview result contract: graph title");
  assert(Number(graphResult?.summary?.seriesCount ?? 0) === 1, "preview result contract: graph series count");
  assert(
    String(graphResult?.tooltip ?? "") === "속도 변화 · 계열 1개 · 점 3개",
    "preview result contract: graph tooltip",
  );

  const wrappedGraph = wrapPreviewResultHtml(graphResult, { className: "runtime-preview-card" });
  assert(wrappedGraph.includes('class="runtime-preview-card"'), "preview result contract: wrapper class");
  assert(wrappedGraph.includes('data-preview-family="graph"'), "preview result contract: wrapper family");
  assert(wrappedGraph.includes('data-preview-mode="runtime"'), "preview result contract: wrapper mode");
  assert(wrappedGraph.includes('title="속도 변화 · 계열 1개 · 점 3개"'), "preview result contract: wrapper tooltip");
  const textResult0 = buildFamilyPreviewResult({
    family: "text",
    mode: "runtime",
    text: "첫 줄\n둘째 줄",
    html: '<div class="runtime-text-preview"></div>',
  });
  const picked = pickPrimaryPreviewResult([textResult0, graphResult], {
    preferredFamilies: ["graph", "text"],
  });
  assert(picked?.family === "graph", "preview result contract: preferred primary family");
  const summaryStrip = buildPreviewSummaryStripHtml(graphResult, { className: "runtime-preview-summary" });
  assert(summaryStrip.includes('class="runtime-preview-summary"'), "preview result contract: summary strip class");
  assert(summaryStrip.includes('data-preview-family="graph"'), "preview result contract: summary strip family");
  assert(summaryStrip.includes("대표 보기"), "preview result contract: summary strip label");
  assert(summaryStrip.includes("속도 변화 · 계열 1개 · 점 3개"), "preview result contract: summary strip tooltip text");
  const collectionHtml = buildPreviewResultCollectionHtml([textResult0, graphResult], {
    preferredFamilies: ["graph", "text"],
    summaryClassName: "runtime-preview-summary",
    cardClassName: "runtime-preview-card",
  });
  assert(collectionHtml.includes('class="runtime-preview-summary"'), "preview result contract: collection summary strip");
  assert(collectionHtml.includes('class="runtime-preview-card"'), "preview result contract: collection cards");
  assert(collectionHtml.includes('data-preview-family="graph"'), "preview result contract: collection primary family");
  const collection = buildPreviewResultCollection([textResult0, graphResult], {
    preferredFamilies: ["graph", "text"],
    summaryClassName: "runtime-preview-summary",
    cardClassName: "runtime-preview-card",
  });
  assert(collection?.primary?.family === "graph", "preview result contract: collection primary");
  assert(Number(collection?.count ?? 0) === 2, "preview result contract: collection count");
  assert(Array.isArray(collection?.families) && collection.families.join(",") === "text,graph", "preview result contract: collection families");
  assert(collection?.html === collectionHtml, "preview result contract: collection html matches");

  const textResult = buildPreviewResult({
    descriptor: { family: "text", mode: "text" },
    html: '<div class="lesson-card-preview-text"></div>',
    text: "첫 줄\n둘째 줄\n",
  });
  assert(textResult?.family === "text", "preview result contract: text family");
  assert(Number(textResult?.summary?.lineCount ?? 0) === 2, "preview result contract: text line count");
  assert(String(textResult?.tooltip ?? "") === "텍스트 · 줄 2개", "preview result contract: text tooltip");

  console.log("seamgrim preview result contract runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
