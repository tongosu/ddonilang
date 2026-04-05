import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/preview_session.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const { resolveLessonCardPreviewSession, resolveLessonCardPreviewViewModel } = mod;

  assert(typeof resolveLessonCardPreviewSession === "function", "preview session export: resolveLessonCardPreviewSession");
  assert(typeof resolveLessonCardPreviewViewModel === "function", "preview session export: resolveLessonCardPreviewViewModel");

  let fetchCount = 0;
  const fetchImpl = async (url) => {
    fetchCount += 1;
    const text = String(url ?? "");
    if (text.includes("/preview/space2d.json")) {
      return {
        ok: true,
        async json() {
          return {
            schema: "seamgrim.space2d.v0",
            meta: { title: "궤적" },
            points: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
          };
        },
      };
    }
    return { ok: false };
  };

  const lesson = {
    id: "preview_session_case",
    requiredViews: [],
    space2dCandidates: ["/preview/space2d.json"],
    graphCandidates: [],
    tableCandidates: [],
    textCandidates: [],
    structureCandidates: [],
  };
  const cache = new Map();
  const resolved0 = await resolveLessonCardPreviewSession(lesson, { cache, fetchImpl });
  assert(resolved0?.descriptor?.family === "space2d", "preview session: inferred descriptor family");
  assert(resolved0?.family === "space2d", "preview session: result family");
  assert(resolved0?.mode === "json", "preview session: result mode");
  assert(Number(resolved0?.count ?? 0) === 1, "preview session: result count");
  assert(Array.isArray(resolved0?.families) && resolved0.families[0] === "space2d", "preview session: result families");
  assert(resolved0?.primary?.family === "space2d", "preview session: primary result");
  assert(String(resolved0?.html ?? "").includes("lesson-card-space2d-canvas"), "preview session: html built");
  assert(String(resolved0?.html ?? "").includes("lesson-card-preview-summary"), "preview session: collection summary strip");
  assert(String(resolved0?.title ?? "") === "궤적", "preview session: title derived from payload");
  assert(Number(resolved0?.summary?.pointCount ?? 0) === 2, "preview session: summary point count");
  assert(
    String(resolved0?.tooltip ?? "") === "궤적 · 점 2개 · 도형 0개",
    "preview session: tooltip derived from summary",
  );
  const resolved1 = await resolveLessonCardPreviewSession(lesson, { cache, fetchImpl });
  assert(resolved1?.descriptor?.family === "space2d", "preview session: cached rerun family");
  assert(String(resolved1?.tooltip ?? "") === "궤적 · 점 2개 · 도형 0개", "preview session: cached tooltip");
  const viewModel = await resolveLessonCardPreviewViewModel(lesson, { cache, fetchImpl });
  assert(viewModel?.primaryFamily === "space2d", "preview session view model: primary family");
  assert(viewModel?.primaryMode === "json", "preview session view model: primary mode");
  assert(viewModel?.previewCount === 1, "preview session view model: count");
  assert(viewModel?.previewFamiliesText === "space2d", "preview session view model: families text");
  assert(viewModel?.headerText === "공간 미리보기", "preview session view model: header text");
  assert(viewModel?.title === "궤적", "preview session view model: title");
  assert(viewModel?.summaryText === "궤적 · 점 2개 · 도형 0개", "preview session view model: summary text");
  assert(fetchCount === 1, "preview session: cache reused across calls");

  console.log("seamgrim preview session runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
