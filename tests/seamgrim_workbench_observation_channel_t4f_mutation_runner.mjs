#!/usr/bin/env node

import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();
const sourcePath = path.join(root, "solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js");
const runnerPath = path.join(root, "tests/seamgrim_workbench_observation_channel_t4f_runner.mjs");
const original = fs.readFileSync(sourcePath);
const beforeHash = crypto.createHash("sha256").update(original).digest("hex");
const text = original.toString("utf8");
const eol = text.includes("\r\n") ? "\r\n" : "\n";
const anchor = [
  `  if (inspected.status !== "available") {`,
  `    return observationProjection("unavailable", { diagnostic: inspected.diagnostic });`,
  `  }`,
].join(eol);
const mutation = [
  `  if (inspected.status !== "available") {`,
  `    const values = Object.fromEntries(rawChannels.map(({ key, index }) => [key, baseRow[index]]));`,
  `    return {`,
  `      status: "available",`,
  `      channels: rawChannels.map(({ key }) => ({ key, dtype: "unknown", role: "상태" })),`,
  `      row: baseRow,`,
  `      values,`,
  `      all_values: { ...values },`,
  `      diagnostic: null,`,
  `      diagnostics: [],`,
  `    };`,
  `  }`,
].join(eol);
assert.equal(text.split(anchor).length - 1, 1, "mutation anchor must be exact and unique");

let red = null;
try {
  fs.writeFileSync(sourcePath, text.replace(anchor, mutation), "utf8");
  red = spawnSync(process.execPath, [runnerPath], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
  });
  assert.notEqual(red.status, 0, "semantic synthesis mutation must be RED");
  assert(!String(red.stderr).includes("SyntaxError"), "compile/syntax error cannot count as RED");
} finally {
  fs.writeFileSync(sourcePath, original);
}

const restored = fs.readFileSync(sourcePath);
const afterHash = crypto.createHash("sha256").update(restored).digest("hex");
assert.equal(afterHash, beforeHash, "mutation source must restore byte-identically");
const green = spawnSync(process.execPath, [runnerPath], {
  cwd: root,
  encoding: "utf8",
  windowsHide: true,
});
assert.equal(green.status, 0, `restored runner must be GREEN: ${green.stderr}`);

console.log(JSON.stringify({
  schema: "seamgrim.workbench.observation_channel_t4f.mutation_result.v1",
  mutation: "missing_manifest_raw_channels_role_dtype_synthesis",
  green_before: "covered_by_primary_run",
  mutation_red: true,
  compile_error_counted_as_red: false,
  restore_green: true,
  residue: 0,
  source_newline_mode: eol === "\r\n" ? "CRLF" : "LF",
  restored_sha256: afterHash,
  result: "PASS",
}));
