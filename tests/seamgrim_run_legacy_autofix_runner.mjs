import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const rootDir = process.cwd();
  const runPath = path.resolve(rootDir, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const preprocessPath = path.resolve(rootDir, "solutions/seamgrim_ui_mvp/ui/runtime/ddn_preprocess.js");
  const runMod = await import(pathToFileURL(runPath).href);
  const preprocessMod = await import(pathToFileURL(preprocessPath).href);
  const { hasLegacyAutofixCandidate, applyLegacyAutofixToDdn } = runMod;
  const { preprocessDdnText } = preprocessMod;

  assert(typeof hasLegacyAutofixCandidate === "function", "run export: hasLegacyAutofixCandidate");
  assert(typeof applyLegacyAutofixToDdn === "function", "run export: applyLegacyAutofixToDdn");
  assert(typeof preprocessDdnText === "function", "preprocess export: preprocessDdnText");

  const legacyInput = [
    "채비: {",
    "  값 <- 0. // 범위(0, 10, 1)",
    "}",
    "(처음)할때: {",
    "  x <- 1.",
    "}",
    "(매틱)마다: {",
    "  y <- y + 1.",
    "}",
    "z <- 0. #범위(0, 5, 0.5)",
  ].join("\n");

  assert(hasLegacyAutofixCandidate(legacyInput) === true, "legacy candidate: should detect");
  const applied = applyLegacyAutofixToDdn(legacyInput);
  assert(applied && typeof applied === "object", "autofix result: object");
  assert(applied.changed === true, "autofix result: changed=true");
  assert(Number(applied.total_changes || 0) >= 5, "autofix result: total_changes>=5");
  assert(String(applied.text).includes("채비 {"), "autofix: setup colon removed");
  assert(String(applied.text).includes("(시작)할때 {"), "autofix: (처음)할때 normalized");
  assert(String(applied.text).includes("(매마디)마다 {"), "autofix: (매틱)마다 normalized");
  assert(String(applied.text).includes("값 <- (0) 매김 { 범위: 0..10. 간격: 1. }."), "autofix: //범위 rewrite");
  assert(String(applied.text).includes("z <- (0) 매김 { 범위: 0..5. 간격: 0.5. }."), "autofix: #범위 rewrite");
  assert(hasLegacyAutofixCandidate(applied.text) === false, "autofix output: no legacy candidate");

  const ssotIfInput = [
    "채비 {",
    "  값: 셈수 <- 3.",
    "  판정: 글 <- \"\".",
    "}.",
    "",
    "만약 값 >= 3 이라면 {",
    "  판정 <- \"통과\".",
    "}.",
    "아니면 {",
    "  판정 <- \"보충\".",
    "}.",
    "",
    "판정 보여주기.",
  ].join("\n");
  assert(hasLegacyAutofixCandidate(ssotIfInput) === false, "ssot if: not legacy autofix candidate");
  const ifApplied = applyLegacyAutofixToDdn(ssotIfInput);
  assert(ifApplied.changed === false, "ssot if: autofix must not mutate");
  const ifPre = preprocessDdnText(ssotIfInput);
  const ifBody = String(ifPre?.bodyText ?? "");
  assert(ifBody.includes("{ 값 >= 3 }인것 일때 {"), "ssot if preprocess: lowered to runnable condition surface");
  assert(!ifBody.includes("만약 값 >= 3 이라면"), "ssot if preprocess: removes raw if keyword for wasm");
  assert(/\}\n\s*아니면\s*\{/u.test(ifBody), "ssot if preprocess: joins else without then terminator");

  const canonicalInput = [
    "설정 {",
    "  이름: \"strict\".",
    "}",
    "채비 {",
    "  a <- (1) 매김 { 범위: 0..10. 간격: 1. }.",
    "}",
    "(시작)할때 {",
    "  합 <- 0.",
    "}",
  ].join("\n");
  assert(hasLegacyAutofixCandidate(canonicalInput) === false, "canonical candidate: should be false");
  const canonicalApplied = applyLegacyAutofixToDdn(canonicalInput);
  assert(canonicalApplied.changed === false, "canonical autofix: changed=false");
  assert(String(canonicalApplied.text) === canonicalInput, "canonical autofix: no text mutation");

  console.log("seamgrim run legacy autofix runner ok");
}

main().catch((error) => {
  console.error(error?.stack || String(error));
  process.exitCode = 1;
});
