import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/table_preview.js");
  const lessonPreviewPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/lesson_card_preview.js");
  const previewRegistryPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/preview_registry.js");
  const tableMod = await import(pathToFileURL(modulePath).href);
  const lessonPreviewMod = await import(pathToFileURL(lessonPreviewPath).href);
  const previewRegistryMod = await import(pathToFileURL(previewRegistryPath).href);
  const {
    summarizeTableView,
    buildTableSummaryMarkdown,
    buildTablePreviewHtml,
  } = tableMod;
  const { buildLessonCardTablePreviewHtml } = lessonPreviewMod;
  const {
    buildFamilyPreviewHtml,
    buildSpace2dPreviewHtml,
    buildTextPreviewHtml,
  } = previewRegistryMod;

  assert(typeof summarizeTableView === "function", "table preview export: summarizeTableView");
  assert(typeof buildTableSummaryMarkdown === "function", "table preview export: buildTableSummaryMarkdown");
  assert(typeof buildTablePreviewHtml === "function", "table preview export: buildTablePreviewHtml");
  assert(typeof buildLessonCardTablePreviewHtml === "function", "lesson card preview export: buildLessonCardTablePreviewHtml");
  assert(typeof buildFamilyPreviewHtml === "function", "preview registry export: buildFamilyPreviewHtml");
  assert(typeof buildSpace2dPreviewHtml === "function", "preview registry export: buildSpace2dPreviewHtml");
  assert(typeof buildTextPreviewHtml === "function", "preview registry export: buildTextPreviewHtml");

  const table = {
    meta: { title: "계수 표" },
    columns: [
      { key: "coef", label: "계수" },
      { key: "value", label: "값" },
    ],
    rows: [
      { coef: "a", value: 1.5 },
      { coef: "b", value: -1 },
    ],
  };

  const summary = summarizeTableView(table);
  assert(summary?.title === "계수 표", "table preview summary: title");
  assert(summary?.columnCount === 2, "table preview summary: column count");
  assert(summary?.rowCount === 2, "table preview summary: row count");
  assert(JSON.stringify(summary?.columns) === JSON.stringify(["계수", "값"]), "table preview summary: labels");

  const markdown = buildTableSummaryMarkdown(table);
  assert(markdown.includes("## 표 요약"), "table preview markdown: heading");
  assert(markdown.includes("- 제목: 계수 표"), "table preview markdown: title");
  assert(markdown.includes("- 열: 2개"), "table preview markdown: columns");

  const runtimeHtml = buildTablePreviewHtml(table);
  assert(runtimeHtml.includes("runtime-table-preview"), "table preview html: runtime class");
  assert(runtimeHtml.includes("runtime-table-preview-table"), "table preview html: runtime table class");
  assert(runtimeHtml.includes("계수"), "table preview html: header");

  const lessonHtml = buildLessonCardTablePreviewHtml(table);
  assert(lessonHtml.includes("lesson-card-preview--table"), "lesson table preview html: lesson class");
  assert(lessonHtml.includes("lesson-card-table-preview"), "lesson table preview html: lesson table class");
  assert(lessonHtml.includes("lesson-card-table-meta"), "lesson table preview html: lesson meta");
  const registryTextHtml = buildFamilyPreviewHtml({
    family: "text",
    text: "# 미리보기\n- 요약",
  });
  assert(registryTextHtml.includes("lesson-card-preview--text"), "preview registry: text renderer");
  const registrySpaceHtml = buildFamilyPreviewHtml({
    family: "space2d",
    payload: {
      meta: { title: "궤적" },
      points: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    },
  });
  assert(registrySpaceHtml.includes("lesson-card-space2d-canvas"), "preview registry: space2d renderer");
  const registryTableHtml = buildFamilyPreviewHtml({
    family: "table",
    payload: table,
  });
  assert(registryTableHtml.includes("lesson-card-table-preview"), "preview registry: table renderer");

  console.log("seamgrim preview component runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
