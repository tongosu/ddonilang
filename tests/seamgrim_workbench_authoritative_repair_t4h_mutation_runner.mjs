#!/usr/bin/env node

import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();
const rustPath = path.join(root, "tool/src/source_analysis.rs");
const readinessPath = path.join(root, "solutions/seamgrim_ui_mvp/ui/workbench/source_readiness.js");
const verticalRunner = path.join(root, "tests/seamgrim_workbench_authoritative_repair_t4h_runner.mjs");
const sourceWasmDir = String(process.env.DDN_T4H_SOURCE_WASM_DIR ?? "").trim();
const trackedWasmDir = String(process.env.DDN_T4H_TRACKED_WASM_DIR ?? "").trim();

assert(sourceWasmDir, "DDN_T4H_SOURCE_WASM_DIR is required");
assert(trackedWasmDir, "DDN_T4H_TRACKED_WASM_DIR is required");

function sha256(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function run(command, args, env = {}) {
  return spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    env: { ...process.env, ...env },
  });
}

function mutateAndRequireRed({ id, filePath, anchor, replacement, command, args, env = {}, expectedEvidence }) {
  const original = fs.readFileSync(filePath);
  const originalText = original.toString("utf8");
  assert.equal(originalText.split(anchor).length - 1, 1, `${id}: mutation anchor must be exact and unique`);
  let red;
  try {
    fs.writeFileSync(filePath, originalText.replace(anchor, replacement), "utf8");
    red = run(command, args, env);
    assert.notEqual(red.status, 0, `${id}: mutation must be RED`);
    const output = `${red.stdout}\n${red.stderr}`;
    assert(!output.includes("SyntaxError"), `${id}: syntax error cannot count as RED`);
    assert(!output.includes("could not compile") || expectedEvidence === "rust_assertion", `${id}: unrelated compile error cannot count as RED`);
    if (expectedEvidence === "rust_assertion") {
      assert(output.includes("multiple_occurrence_identities_are_unique_and_deterministic"), `${id}: intended Rust test did not reject mutation`);
    }
  } finally {
    fs.writeFileSync(filePath, original);
  }
  assert.equal(sha256(fs.readFileSync(filePath)), sha256(original), `${id}: source did not restore byte-identically`);
  return { id, red: true, restored_sha256: sha256(original) };
}

const rustMutation = mutateAndRequireRed({
  id: "M1_OCCURRENCE_ORDINAL_COLLAPSE",
  filePath: rustPath,
  anchor: "spans.get(ordinal).and_then(|span| {",
  replacement: "spans.first().and_then(|span| {",
  command: "cargo",
  args: ["test", "-p", "ddonirang-tool", "source_analysis::tests::multiple_occurrence_identities_are_unique_and_deterministic", "--locked", "--", "--nocapture"],
  expectedEvidence: "rust_assertion",
});

const commonEnv = {
  DDN_T4H_SOURCE_WASM_DIR: sourceWasmDir,
  DDN_T4H_TRACKED_WASM_DIR: trackedWasmDir,
};

const replacementMutation = mutateAndRequireRed({
  id: "M2_JS_REPLACEMENT_INVENTION",
  filePath: readinessPath,
  anchor: "replacement: repair.replacement_text,",
  replacement: 'replacement: "",',
  command: process.execPath,
  args: [verticalRunner],
  env: commonEnv,
});

const schemaMutation = mutateAndRequireRed({
  id: "M3_UNSUPPORTED_PROPOSAL_SCHEMA_ACCEPTANCE",
  filePath: readinessPath,
  anchor: "repair.schema !== REPAIR_PROPOSAL_SCHEMA",
  replacement: "false",
  command: process.execPath,
  args: [verticalRunner],
  env: commonEnv,
});

const restoredRust = run("cargo", [
  "test",
  "-p",
  "ddonirang-tool",
  "source_analysis::tests::multiple_occurrence_identities_are_unique_and_deterministic",
  "--locked",
  "--",
  "--nocapture",
]);
assert.equal(restoredRust.status, 0, `restored Rust test must be GREEN: ${restoredRust.stdout}\n${restoredRust.stderr}`);
const restoredVertical = run(process.execPath, [verticalRunner], commonEnv);
assert.equal(restoredVertical.status, 0, `restored vertical runner must be GREEN: ${restoredVertical.stdout}\n${restoredVertical.stderr}`);

console.log(JSON.stringify({
  schema: "seamgrim.workbench.authoritative_repair_t4h.mutation_result.v1",
  mutations: [rustMutation, replacementMutation, schemaMutation],
  mutation_red: "3/3",
  compile_error_counted_as_red: false,
  restore_green: true,
  residue: 0,
  result: "PASS",
}));
