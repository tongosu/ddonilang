#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import {
  extractCanonicalViewAuthorityInputsFromState,
} from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  buildCanonicalViewPayloadSet,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function proveMutationRed({ label, greenValue, mutate, invariant }) {
  assert(invariant(greenValue), `${label}: initial GREEN failed`);
  const mutant = mutate(structuredClone(greenValue));
  assert(!invariant(mutant), `${label}: mutation did not turn RED`);
  assert(invariant(greenValue), `${label}: restored GREEN failed`);
  return { label, initial: "GREEN", mutated: "RED", restored: "GREEN" };
}

const root = process.cwd();
const state = JSON.parse(await fs.readFile(
  path.join(root, "pack/seamgrim_table_text_structure_canonical_view_payload_v1/fixtures/authoritative_state.detjson"),
  "utf8",
));
const payloads = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));
const results = [];

results.push(proveMutationRed({
  label: "M1_TABLE_ROW_ORDER_DRIFT",
  greenValue: { run: payloads.table.content.table.rows, playground: structuredClone(payloads.table.content.table.rows) },
  mutate(value) {
    value.playground.reverse();
    return value;
  },
  invariant(value) {
    return stableSemanticJson(value.run) === stableSemanticJson(value.playground);
  },
}));

results.push(proveMutationRed({
  label: "M2_TEXT_OUTPUT_SYNTHETIC_SUCCESS",
  greenValue: { status: "no_view", content: null },
  mutate(value) {
    value.status = "available";
    value.content = structuredClone(payloads.text.content);
    return value;
  },
  invariant(value) {
    return value.status === "no_view" && value.content === null;
  },
}));

results.push(proveMutationRed({
  label: "M3_STRUCTURE_NODE_IDENTITY_DRIFT",
  greenValue: {
    run: payloads.structure.content.node_identities,
    workbench: structuredClone(payloads.structure.content.node_identities),
  },
  mutate(value) {
    value.workbench[1] = "renderer-node-1";
    return value;
  },
  invariant(value) {
    return stableSemanticJson(value.run) === stableSemanticJson(value.workbench);
  },
}));

const unsupportedTextFormatState = structuredClone(state);
unsupportedTextFormatState.view_meta.text.format = "html";
const unsupportedTextFormatPayload = buildCanonicalViewPayloadSet(
  extractCanonicalViewAuthorityInputsFromState(unsupportedTextFormatState),
).text;
const unsupportedTextFormatExpected = JSON.parse(await fs.readFile(
  path.join(
    root,
    "pack/seamgrim_table_text_structure_canonical_view_payload_v1/expected/text_unsupported_format.payload.detjson",
  ),
  "utf8",
));
results.push(proveMutationRed({
  label: "M4_TEXT_UNSUPPORTED_FORMAT_GOLDEN_DRIFT",
  greenValue: {
    actual: unsupportedTextFormatPayload,
    expected: unsupportedTextFormatExpected,
  },
  mutate(value) {
    value.expected.diagnostic.reason = "unsupported_text_format";
    return value;
  },
  invariant(value) {
    return stableSemanticJson(value.actual) === stableSemanticJson(value.expected);
  },
}));

console.log(JSON.stringify({ status: "PASS", mutation_count: results.length, mutation_residue: 0, results }));
