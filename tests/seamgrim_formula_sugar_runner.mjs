import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/formula_sugar.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const {
    applyFormulaDdnToSource,
    buildFormulaSugarDdn,
    parseFormulaSugarDraft,
  } = mod;

  assert(typeof parseFormulaSugarDraft === "function", "formula sugar: parse export");
  assert(typeof buildFormulaSugarDdn === "function", "formula sugar: build export");
  assert(typeof applyFormulaDdnToSource === "function", "formula sugar: apply export");

  const parsed = parseFormulaSugarDraft({
    formulaText: "y = 2*x + 1",
    axisVar: "x",
    xMin: -1,
    xMax: 3,
    step: 0.5,
  });
  assert(parsed.ok === true, "formula sugar: parse ok");
  assert(parsed.data?.outputVar === "y", "formula sugar: output var");
  assert(parsed.data?.expression === "2*x + 1", "formula sugar: expression");

  const ddn = buildFormulaSugarDdn(parsed.data);
  assert(!ddn.includes("#이름:") && !ddn.includes("#설명:"), "formula sugar: legacy hash meta absent");
  assert(ddn.includes("설정 {"), "formula sugar: settings meta block");
  assert(ddn.includes("제목: formula_y."), "formula sugar: settings title");
  assert(ddn.includes("x목록 <- (x_min, x_max, x_step) 범위."), "formula sugar: range call");
  assert(ddn.includes("(x) x목록에 대해: {"), "formula sugar: foreach block");
  assert(ddn.includes("y <- 2*x + 1."), "formula sugar: assignment");
  assert(ddn.includes("보임 {"), "formula sugar: output block");

  const replaced = applyFormulaDdnToSource("채비 { a:수 <- 1. }.\n", ddn, { mode: "replace" });
  assert(replaced.trim() === ddn.trim(), "formula sugar: replace mode");

  const inserted = applyFormulaDdnToSource("첫줄.\n둘째줄.\n", "끼움.", {
    mode: "insert",
    selectionStart: 3,
    selectionEnd: 3,
  });
  assert(inserted.includes("첫줄.\n끼움.\n\n둘째줄."), "formula sugar: insert mode");

  const invalid = parseFormulaSugarDraft({
    formulaText: "y = x = 1",
    axisVar: "x",
    xMin: 0,
    xMax: 1,
    step: 0.1,
  });
  assert(invalid.ok === false, "formula sugar: invalid equation");

  const invalidRange = parseFormulaSugarDraft({
    formulaText: "y = x + 1",
    axisVar: "x",
    xMin: 2,
    xMax: 1,
    step: 0.1,
  });
  assert(invalidRange.ok === false, "formula sugar: invalid range");

  console.log("seamgrim formula sugar runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
