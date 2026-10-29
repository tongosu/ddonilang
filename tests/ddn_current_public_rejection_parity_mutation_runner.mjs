#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs";
import fsp from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const runnerPath = path.join(root, "tests", "ddn_current_public_rejection_parity_runner.mjs");
const contractPath = path.join(root, "pack", "ddn_current_public_rejection_parity_v1", "contract.detjson");
const tempDir = await fsp.mkdtemp(path.join(os.tmpdir(), "ddn-g14-rejection-mutation-"));

function run(runner, contract) {
  return spawnSync(process.execPath, [runner], {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    env: { ...process.env, DDN_G14_ROOT: root, DDN_G14_CONTRACT: contract },
  });
}

function requireGreen(label, runner = runnerPath, contract = contractPath) {
  const result = run(runner, contract);
  assert.equal(result.status, 0, `${label} expected GREEN\n${result.stdout}\n${result.stderr}`);
  console.log(`${label}: GREEN`);
}

function requireRed(label, runner = runnerPath, contract = contractPath) {
  const result = run(runner, contract);
  assert.notEqual(result.status, 0, `${label} expected RED but passed`);
  console.log(`${label}: RED`);
}

function replaceOnce(text, before, after, label) {
  const first = text.indexOf(before);
  assert.ok(first >= 0, `${label}: anchor missing`);
  assert.equal(first, text.lastIndexOf(before), `${label}: anchor is not unique`);
  return text.slice(0, first) + after + text.slice(first + before.length);
}

const originalRunnerRaw = await fsp.readFile(runnerPath, "utf8");
const originalRunner = originalRunnerRaw.replaceAll("\r\n", "\n");
const originalContract = JSON.parse(await fsp.readFile(contractPath, "utf8"));

try {
  requireGreen("baseline");

  const badDiagnostic = structuredClone(originalContract);
  badDiagnostic.cases[0].expected.diagnostic_code = "E_WRONG_DIAGNOSTIC";
  const badDiagnosticPath = path.join(tempDir, "m1-contract.detjson");
  await fsp.writeFile(badDiagnosticPath, JSON.stringify(badDiagnostic), "utf8");
  requireRed("M1_WRONG_DIAGNOSTIC", runnerPath, badDiagnosticPath);

  const fakeSuccessRunner = replaceOnce(
    originalRunner,
    "let wasmReturnedSummary = false;",
    "let wasmReturnedSummary = true;",
    "M2",
  );
  const fakeSuccessPath = path.join(tempDir, "m2-runner.mjs");
  await fsp.writeFile(fakeSuccessPath, fakeSuccessRunner, "utf8");
  requireRed("M2_NATIVE_OR_WASM_FAKE_SUCCESS", fakeSuccessPath);

  const omitted = structuredClone(originalContract);
  omitted.cases.pop();
  omitted.expected_case_ids.pop();
  omitted.expected_case_count = 2;
  const omittedPath = path.join(tempDir, "m3-contract.detjson");
  await fsp.writeFile(omittedPath, JSON.stringify(omitted), "utf8");
  requireRed("M3_ONE_CASE_OMITTED", runnerPath, omittedPath);

  const hiddenCompatRunner = replaceOnce(
    originalRunner,
    "compatibility_fallback_count: 0,\n    };",
    "compatibility_fallback_count: entry.id === contract.expected_case_ids[0] ? 1 : 0,\n    };",
    "M4",
  );
  const hiddenCompatPath = path.join(tempDir, "m4-runner.mjs");
  await fsp.writeFile(hiddenCompatPath, hiddenCompatRunner, "utf8");
  requireRed("M4_HIDDEN_COMPATIBILITY_ALLOWED", hiddenCompatPath);

  const residueRunner = replaceOnce(
    originalRunner,
    "state_mutation_count: fs.existsSync(summaryPath) ? 1 : 0,",
    "state_mutation_count: entry.id === contract.expected_case_ids[0] ? 1 : (fs.existsSync(summaryPath) ? 1 : 0),",
    "M5",
  );
  const residuePath = path.join(tempDir, "m5-runner.mjs");
  await fsp.writeFile(residuePath, residueRunner, "utf8");
  requireRed("M5_STATE_OR_EFFECT_RESIDUE", residuePath);

  requireGreen("restored");
} finally {
  await fsp.rm(tempDir, { recursive: true, force: true });
}

assert.equal(fs.readFileSync(runnerPath, "utf8"), originalRunnerRaw, "runner mutation residue");
assert.deepEqual(JSON.parse(fs.readFileSync(contractPath, "utf8")), originalContract, "contract mutation residue");
console.log("DDN_CURRENT_PUBLIC_REJECTION_PARITY_MUTATION_PASS mutations=5 residue=0");
