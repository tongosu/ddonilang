function extractWarningExampleName(example) {
  const text = String(example ?? "").trim();
  if (!text) return "";
  const match = text.match(/^([A-Za-z0-9_가-힣]+)(?::[A-Za-z0-9_가-힣]+)?\s*(<-|=)\s*\(/);
  return match ? String(match[1] ?? "").trim() : "";
}

function buildWarningExampleMap(warningExamples) {
  const map = new Map();
  const rows = Array.isArray(warningExamples) ? warningExamples : [];
  rows.forEach((row) => {
    const text = String(row ?? "").trim();
    if (!text) return;
    const name = extractWarningExampleName(text);
    if (!name || map.has(name)) return;
    map.set(name, text);
  });
  return map;
}

export function buildLegacyGuideDraftText({
  title = "교과",
  ddnText = "",
  warningNames = [],
  warningExamples = [],
} = {}) {
  const normalizedTitle = String(title ?? "").trim() || "교과";
  const normalizedDdnText = String(ddnText ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  const names = Array.isArray(warningNames)
    ? warningNames.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  const examples = Array.isArray(warningExamples)
    ? warningExamples.map((item) => String(item ?? "").trim()).filter(Boolean)
    : [];
  const exampleMap = buildWarningExampleMap(examples);

  const guideLines = [
    "// --- 매김 전환 초안 ---",
    `// ${normalizedTitle}의 금지된 \`// 범위(...)\`를 \`매김 {}\`로 옮기기 위한 편집 초안입니다.`,
  ];
  if (names.length > 0) {
    guideLines.push(`// 대상 항목: ${names.join(", ")}`);
  }
  if (examples.length > 0) {
    guideLines.push("// 예시:");
    examples.slice(0, 3).forEach((row) => {
      guideLines.push(`// ${row}`);
    });
  }
  guideLines.push("// 주의: 아래 전환 후보를 적용한 뒤, 기존 `// 범위(...)`가 붙은 원래 선언 줄은 지워야 합니다.");
  guideLines.push("// 아래 원문을 참고해 해당 선언을 직접 전환하세요.", "");

  const bodyLines = normalizedDdnText.split("\n");
  const rewrittenLines = [];
  bodyLines.forEach((line) => {
    rewrittenLines.push(line);
    const match = line.match(/^(\s*)([A-Za-z0-9_가-힣]+)(?::[A-Za-z0-9_가-힣]+)?\s*(<-|=)\s*.*\/\/\s*범위\s*\(/);
    if (!match) return;
    const indent = String(match[1] ?? "");
    const name = String(match[2] ?? "").trim();
    const example = exampleMap.get(name);
    if (!example) return;
    rewrittenLines.push(`${indent}// 매김 전환 후보: ${example}`);
    rewrittenLines.push(`${indent}// 원래 줄을 지우고 아래 줄로 교체`);
    rewrittenLines.push(`${indent}${example}`);
  });

  return `${guideLines.join("\n")}${rewrittenLines.join("\n")}`;
}
