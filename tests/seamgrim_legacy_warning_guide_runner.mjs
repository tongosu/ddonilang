import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/legacy_warning_guide.js");
  const { buildLegacyGuideDraftText } = await import(pathToFileURL(modulePath).href);

  const draft = buildLegacyGuideDraftText({
    title: "pendulum",
    ddnText: "채비 {\n}.\n",
    warningNames: ["g", "L"],
    warningExamples: [
      "g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }.",
      "L <- (1) 매김 { 범위: 0.2..3. 간격: 0.1. }.",
    ],
  });

  assert(draft.startsWith("// --- 매김 전환 초안 ---"), "draft header");
  assert(draft.includes("// 대상 항목: g, L"), "draft warning names");
  assert(draft.includes("// g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }."), "draft first example");
  assert(
    draft.includes("// 주의: 아래 전환 후보를 적용한 뒤, 기존 `// 범위(...)`가 붙은 원래 선언 줄은 지워야 합니다."),
    "draft delete warning",
  );
  assert(draft.includes("채비 {\n}."), "draft includes original ddn");

  const inserted = buildLegacyGuideDraftText({
    title: "pendulum",
    ddnText: "(시작)할때 {\n  g <- 9.8. // 범위(1, 20, 0.1)\n  x <- 1.\n}.\n",
    warningNames: ["g"],
    warningExamples: ["g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }."],
  });
  assert(
    inserted.includes(
      "g <- 9.8. // 범위(1, 20, 0.1)\n  // 매김 전환 후보: g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }.\n  // 원래 줄을 지우고 아래 줄로 교체\n  g <- (9.8) 매김 { 범위: 1..20. 간격: 0.1. }.",
    ),
    "draft inserts warning example below matching declaration",
  );

  console.log("seamgrim legacy warning guide runner ok");
}

main().catch((error) => {
  console.error(error?.message ?? String(error));
  process.exit(1);
});
