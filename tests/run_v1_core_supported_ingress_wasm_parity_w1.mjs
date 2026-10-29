import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const pack = path.join(root, "pack", "v1_core_supported_ingress_w1", "fixtures");
const wasmDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm");
const wasm = await import(pathToFileURL(path.join(wasmDir, "ddonirang_tool.js")).href);
await wasm.default({ module_or_path: fs.readFileSync(path.join(wasmDir, "ddonirang_tool_bg.wasm")) });

assert.equal(typeof wasm.wasm_source_analysis_snapshot_json, "function");
assert.equal(typeof wasm.wasm_run_summary_with_source_profile_json, "function");

const profile = "v1-core-v25";
const source = (name) => fs.readFileSync(path.join(pack, name), "utf8");
const analyze = (text, generation = 1, identity = profile) => JSON.parse(
  wasm.wasm_source_analysis_snapshot_json(JSON.stringify({
    source_text: text,
    path: "pack/v1_core_supported_ingress_w1/fixture.ddn",
    analysis_generation: generation,
    ...(identity === null ? {} : { source_profile_identity: identity }),
  })),
);

const positive = source("positive_definition_rebinding.ddn");
const accepted = analyze(positive, 301);
assert.equal(accepted.outcome, "ACCEPTED");
assert.deepEqual(accepted.diagnostics, []);

const summaryText = wasm.wasm_run_summary_with_source_profile_json(
  positive,
  "pack/v1_core_supported_ingress_w1/fixtures/positive_definition_rebinding.ddn",
  1,
  profile,
);
const repeatedSummaryText = wasm.wasm_run_summary_with_source_profile_json(
  positive,
  "pack/v1_core_supported_ingress_w1/fixtures/positive_definition_rebinding.ddn",
  1,
  profile,
);
assert.equal(summaryText, repeatedSummaryText);
const summary = JSON.parse(summaryText);
assert.deepEqual(summary.output_log_texts, ["2"]);

const collection = source("language_core_collection.ddn");
const collectionSummaryText = wasm.wasm_run_summary_with_source_profile_json(
  collection,
  "pack/v1_core_supported_ingress_w1/fixtures/language_core_collection.ddn",
  1,
  profile,
);
const repeatedCollectionSummaryText = wasm.wasm_run_summary_with_source_profile_json(
  collection,
  "pack/v1_core_supported_ingress_w1/fixtures/language_core_collection.ddn",
  1,
  profile,
);
assert.equal(collectionSummaryText, repeatedCollectionSummaryText);
assert.deepEqual(JSON.parse(collectionSummaryText).output_log_texts, ["차림[1, 2, 3]", "2"]);

const strictSelection = source("strict_selection_out_of_range.ddn");
const strictSelectionErrors = [];
for (let repeat = 0; repeat < 2; repeat += 1) {
  try {
    wasm.wasm_run_summary_with_source_profile_json(
      strictSelection,
      "pack/v1_core_supported_ingress_w1/fixtures/strict_selection_out_of_range.ddn",
      1,
      profile,
    );
  } catch (error) {
    strictSelectionErrors.push(String(error));
  }
}
assert.equal(strictSelectionErrors.length, 2);
assert.equal(strictSelectionErrors[0], strictSelectionErrors[1]);
const strictSelectionError = JSON.parse(strictSelectionErrors[0]);
assert.equal(strictSelectionError.code, "E_RUNTIME_EXECUTION");
assert.equal(strictSelectionError.message, "FATAL:CHARIM_INDEX_OUT_OF_RANGE");
assert.doesNotMatch(strictSelectionErrors[0], /output_rows|output_log_texts/);

let typeMismatchError = "";
try {
  wasm.wasm_run_summary_with_source_profile_json(
    source("value_type_mismatch.ddn"),
    "pack/v1_core_supported_ingress_w1/fixtures/value_type_mismatch.ddn",
    1,
    profile,
  );
} catch (error) {
  typeMismatchError = String(error);
}
const typeMismatch = JSON.parse(typeMismatchError);
assert.equal(typeMismatch.code, "E_RUNTIME_TYPE_MISMATCH");
assert.equal(typeMismatch.offending_field, "값");
assert.equal(typeMismatch.expected, "수");
assert.equal(typeMismatch.actual, "글");
assert.doesNotMatch(typeMismatchError, /output_rows|output_log_texts/);

const legacy = analyze(source("legacy_definition_equals.ddn"), 302);
assert.equal(legacy.outcome, "REJECTED");
assert.equal(legacy.diagnostics[0].code, "E_V25_DEFINITION_MIGRATION_REQUIRED");

for (const [name, generation] of [["direct_recursion.ddn", 303], ["mutual_recursion.ddn", 304]]) {
  const first = analyze(source(name), generation);
  const repeated = analyze(source(name), generation);
  assert.deepEqual(first, repeated);
  assert.equal(first.outcome, "REJECTED");
  assert.equal(first.diagnostics[0].code, "E_V1_CORE_RECURSION_UNSUPPORTED");
}

const omitted = analyze(positive, 305, null);
assert.equal(omitted.outcome, "UNSUPPORTED");
assert.equal(omitted.diagnostics[0].code, "E_SOURCE_PROFILE_REQUIRED");
const unknown = analyze(positive, 306, "unknown-profile");
assert.equal(unknown.outcome, "UNSUPPORTED");
assert.equal(unknown.diagnostics[0].code, "E_SOURCE_PROFILE_UNSUPPORTED");

let legacyRunError = "";
try {
  wasm.wasm_run_summary_with_source_profile_json(
    source("legacy_definition_equals.ddn"),
    "legacy.ddn",
    1,
    profile,
  );
} catch (error) {
  legacyRunError = String(error);
}
assert.match(legacyRunError, /E_V25_DEFINITION_MIGRATION_REQUIRED/);
assert.doesNotMatch(legacyRunError, /output_rows|output_log_texts/);

console.log("P1A_WASM_PARITY_PASS cases=11 deterministic_repeats=5 partial_summary=0");
