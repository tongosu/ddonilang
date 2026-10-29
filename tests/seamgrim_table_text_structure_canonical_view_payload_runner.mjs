#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import {
  extractCanonicalViewAuthorityInputsFromState,
} from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  buildCanonicalViewPayloadSet,
  buildCommittedSemanticViewPayload,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";
import { extractRuntimeDerived } from "../solutions/seamgrim_ui_mvp/ui/screens/run.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function payloadsFor(state) {
  return buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));
}

const root = process.cwd();
const packRoot = path.join(root, "pack", "seamgrim_table_text_structure_canonical_view_payload_v1");
const state = JSON.parse(await fs.readFile(path.join(packRoot, "fixtures", "authoritative_state.detjson"), "utf8"));
const stateBefore = stableSemanticJson(state);
const direct = payloadsFor(state);
const runDerived = extractRuntimeDerived(state);
const run = runDerived.canonicalViewPayloads;
const playground = payloadsFor(state);
const workbench = buildCommittedSemanticViewPayload({
  project: { id: "t5d", primaryView: "line" },
  run: { run_id: "t5d", source_hash: state.source_hash, frames: [] },
  frame: {
    frame_id: 11,
    madi: state.tick_id,
    state_hash: state.state_hash,
    source_hash: state.source_hash,
    wasm_state: state,
  },
  canonicalViewPayloads: payloadsFor(state),
}).canonical_views;

for (const family of ["table", "text", "structure"]) {
  assert(direct[family].status === "available", `${family} canonical payload unavailable`);
  assert(
    direct[family].provenance.classification === "AUTHORITATIVE_STRUCTURED",
    `${family} authority classification missing`,
  );
  const directBytes = stableSemanticJson(direct[family]);
  assert(directBytes === stableSemanticJson(run[family]), `${family} Run parity failed`);
  assert(directBytes === stableSemanticJson(playground[family]), `${family} Playground parity failed`);
  assert(directBytes === stableSemanticJson(workbench[family]), `${family} Workbench parity failed`);
  assert(
    stableSemanticJson(runDerived.views[family]) === stableSemanticJson(direct[family].content[family]),
    `${family} Run renderer port drift`,
  );
  if (process.argv.includes("--emit")) {
    console.log(`${family.toUpperCase()}=${JSON.stringify(JSON.parse(directBytes), null, 2)}`);
  } else {
    const expected = JSON.parse(await fs.readFile(
      path.join(packRoot, "expected", `${family}.payload.detjson`),
      "utf8",
    ));
    assert(directBytes === stableSemanticJson(expected), `${family} exact structured golden mismatch`);
  }
}

assert(direct.table.content.column_identities.join(",") === "time,position", "table column identity/order drift");
assert(direct.structure.content.node_identities.join(",") === "source,result", "structure node identity/order drift");
assert(direct.text.content.text.content === "# 실행 설명\n\n위치는 12입니다.", "text content drift");

const outputOnlyText = {
  schema: "seamgrim.state.v0",
  state_hash: "sha256:t5d-output-only",
  channels: [{ key: "보개_출력_줄들", dtype: "text", role: "view" }],
  row: [JSON.stringify(["text.overlay", "markdown", "합성 금지"])],
  resources: { value: {}, value_json: {} },
};
const outputOnlyPayload = payloadsFor(outputOnlyText).text;
assert(outputOnlyPayload.status === "no_view" && outputOnlyPayload.content === null, "output text synthesized canonical text");
assert(extractRuntimeDerived(outputOnlyText).views.text === null, "Run active path retained hidden text fallback");

const shapeOnlyTable = structuredClone(state);
shapeOnlyTable.view_meta.table = { columns: [{ key: "x" }], rows: [{ x: 1 }] };
const shapeOnlyTablePayload = payloadsFor(shapeOnlyTable).table;
assert(shapeOnlyTablePayload.status === "invalid", "schema-less table must fail closed");
assert(shapeOnlyTablePayload.diagnostic?.code === "E_CANONICAL_TABLE_INVALID_CANDIDATE", "table diagnostic drift");

const unsupportedText = structuredClone(state);
unsupportedText.view_meta.text.schema = "seamgrim.text.v999";
const unsupportedTextPayload = payloadsFor(unsupportedText).text;
assert(unsupportedTextPayload.status === "unsupported", "unknown text schema must fail closed");
assert(unsupportedTextPayload.diagnostic?.code === "E_CANONICAL_TEXT_UNSUPPORTED_CANDIDATE", "text diagnostic drift");

const unsupportedTextFormat = structuredClone(state);
unsupportedTextFormat.view_meta.text.format = "html";
const unsupportedTextFormatPayload = payloadsFor(unsupportedTextFormat).text;
const unsupportedTextFormatExpected = JSON.parse(await fs.readFile(
  path.join(packRoot, "expected", "text_unsupported_format.payload.detjson"),
  "utf8",
));
assert(
  stableSemanticJson(unsupportedTextFormatPayload) === stableSemanticJson(unsupportedTextFormatExpected),
  "unsupported text format exact golden mismatch",
);
assert(unsupportedTextFormatPayload.status === "invalid", "unsupported text format must fail closed");
assert(
  unsupportedTextFormatPayload.diagnostic?.code === "E_CANONICAL_TEXT_INVALID_CANDIDATE"
    && unsupportedTextFormatPayload.diagnostic?.reason === "text_format_invalid",
  "unsupported text format diagnostic drift",
);

const duplicateNode = structuredClone(state);
duplicateNode.view_meta.structure.nodes[1].id = "source";
const duplicateNodePayload = payloadsFor(duplicateNode).structure;
assert(duplicateNodePayload.status === "invalid", "duplicate structure node id must fail closed");
assert(duplicateNodePayload.diagnostic?.code === "E_CANONICAL_STRUCTURE_INVALID_CANDIDATE", "structure diagnostic drift");

const brokenEdge = structuredClone(state);
brokenEdge.view_meta.structure.edges[0].to = "missing";
assert(payloadsFor(brokenEdge).structure.status === "invalid", "missing structure endpoint must fail closed");

const ambiguousTable = structuredClone(state);
delete ambiguousTable.view_meta.table;
ambiguousTable.resources.value_json = {
  "table.a": state.view_meta.table,
  "table.b": structuredClone(state.view_meta.table),
};
assert(payloadsFor(ambiguousTable).table.status === "invalid", "ambiguous table resources must fail closed");

const identityMissing = structuredClone(state);
delete identityMissing.state_hash;
delete identityMissing.result_identity;
for (const family of ["table", "text", "structure"]) {
  assert(payloadsFor(identityMissing)[family].status === "unavailable", `${family} missing identity must fail closed`);
}

const runSource = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js"), "utf8");
const playgroundSource = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/playground.js"), "utf8");
const workbenchSource = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js"), "utf8");
assert(runSource.includes("canonicalViewPayloads?.table"), "Run table canonical binding missing");
assert(runSource.includes("allowLegacyTextCompatibility: false"), "Run hidden text fallback remains active");
assert(playgroundSource.includes("canonicalViewPayloads?.structure"), "Playground structure canonical binding missing");
assert(workbenchSource.includes("buildCanonicalViewPayloadSet"), "Workbench canonical binding missing");
assert(stableSemanticJson(state) === stateBefore, "canonical projection mutated authoritative state");

console.log(JSON.stringify({
  status: "PASS",
  schema: direct.table.schema,
  families: ["table", "text", "structure"],
  run_workbench_playground_byte_parity: true,
  hidden_text_fallback: 0,
  unsupported_text_format_golden: 1,
  ambiguous_resource_success: 0,
  state_hash_drift: 0,
  storage_drift: 0,
}));
