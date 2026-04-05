import { buildLessonCardPreviewHtml } from "./components/lesson_card_preview.js";
import { resolvePreviewHtmlFromDescriptors } from "./preview_payload_loader.js";
import { buildPreviewResult, buildPreviewResultCollection } from "./preview_result_contract.js";
import { buildPreviewViewModel } from "./preview_view_model.js";
import { resolveLessonPreviewDescriptors } from "./view_family_contract.js";

export async function resolveLessonCardPreviewSession(
  lesson,
  {
    cache = null,
    fetchImpl = globalThis.fetch,
    renderPreview = ({ family, payload, text }) => buildLessonCardPreviewHtml({ family, payload, text }),
  } = {},
) {
  const descriptors = resolveLessonPreviewDescriptors(lesson);
  const resolved = await resolvePreviewHtmlFromDescriptors(descriptors, {
    cache,
    fetchImpl,
    renderPreview,
  });
  const result = buildPreviewResult({
    descriptor: resolved?.descriptor ?? null,
    html: resolved?.html ?? "",
    payload: resolved?.payload ?? null,
    text: resolved?.descriptor?.mode === "text" ? resolved?.payload ?? "" : "",
  });
  return buildPreviewResultCollection(result ? [result] : [], {
    preferredFamilies: descriptors.map((descriptor) => descriptor?.family),
    summaryClassName: "lesson-card-preview-summary",
    cardClassName: "lesson-card-preview-card",
  });
}

export async function resolveLessonCardPreviewViewModel(lesson, options = {}) {
  const collection = await resolveLessonCardPreviewSession(lesson, options);
  return buildPreviewViewModel(collection, {
    sourceId: String(lesson?.id ?? ""),
  });
}
