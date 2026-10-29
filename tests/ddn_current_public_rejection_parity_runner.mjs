#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs";
import fsp from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(
  String(process.env.DDN_G14_ROOT ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..")),
);
const contractPath = path.resolve(
  root,
  process.env.DDN_G14_CONTRACT || "pack/ddn_current_public_rejection_parity_v1/contract.detjson",
);
const reportPath = String(process.env.DDN_G14_REPORT ?? "").trim();

function externalBuildRoot() {
  const configured = String(process.env.DDN_BUILD_DIR ?? "").trim();
  return configured || path.join(os.tmpdir(), "ddonirang", "build");
}

function invoke(command, args, env = {}) {
  return spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    env: { ...process.env, CARGO_NET_OFFLINE: "true", ...env },
  });
}

function requireSuccess(label, result) {
  if (result.status !== 0) {
    throw new Error(`${label} failed\n${result.stdout ?? ""}\n${result.stderr ?? ""}`);
  }
}

function resolveCli() {
  const supplied = String(process.env.DDN_G14_CLI ?? "").trim();
  if (supplied) {
    assert.ok(fs.existsSync(supplied) && fs.statSync(supplied).isFile(), `DDN_G14_CLI is not a file: ${supplied}`);
    return path.resolve(supplied);
  }
  const target = path.resolve(
    String(process.env.DDN_G14_TARGET_DIR ?? path.join(externalBuildRoot(), "ddn-long-relint-g14-target")),
  );
  const built = invoke(
    "cargo",
    ["build", "-q", "--manifest-path", "tool/Cargo.toml", "--bin", "ddonirang-tool"],
    { CARGO_TARGET_DIR: target, RUSTFLAGS: "-Awarnings", CARGO_INCREMENTAL: "0" },
  );
  requireSuccess("offline ddonirang-tool build", built);
  const exe = path.join(target, "debug", process.platform === "win32" ? "ddonirang-tool.exe" : "ddonirang-tool");
  assert.ok(fs.existsSync(exe) && fs.statSync(exe).isFile(), `built CLI is missing: ${exe}`);
  return exe;
}

function parseStructuredDiagnostic(text, label) {
  const normalized = String(text ?? "").replaceAll("\r\n", "\n").trim();
  const start = normalized.indexOf("{");
  assert.ok(start >= 0, `${label}: structured diagnostic JSON missing: ${normalized}`);
  let value;
  try {
    value = JSON.parse(normalized.slice(start));
  } catch (error) {
    throw new Error(`${label}: invalid diagnostic JSON: ${error.message}\n${normalized}`);
  }
  return value;
}

function semanticCode(diagnostic) {
  const match = String(diagnostic?.message ?? "").match(/^(E_[A-Z0-9_]+):/u);
  return match?.[1] ?? "";
}

function normalizeDiagnostic(diagnostic) {
  return {
    outer_diagnostic_code: String(diagnostic?.code ?? ""),
    diagnostic_code: semanticCode(diagnostic),
    phase: String(diagnostic?.phase ?? ""),
    source_path: String(diagnostic?.source_path ?? "").replaceAll("\\", "/"),
    source_span: diagnostic?.source_span ?? null,
    cause: diagnostic?.cause ?? null,
    offending_field: diagnostic?.offending_field ?? null,
    expected: diagnostic?.expected ?? null,
    actual: diagnostic?.actual ?? null,
  };
}

function exactExpected(expected, input) {
  return {
    outer_diagnostic_code: expected.outer_diagnostic_code,
    diagnostic_code: expected.diagnostic_code,
    phase: expected.phase,
    source_path: input.replaceAll("\\", "/"),
    source_span: expected.source_span,
    cause: null,
    offending_field: null,
    expected: null,
    actual: null,
  };
}

async function loadWasm() {
  const configured = String(process.env.DDN_G14_WASM_DIR ?? "").trim();
  const wasmDir = path.resolve(configured || path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "wasm"));
  const jsPath = path.join(wasmDir, "ddonirang_tool.js");
  const bytesPath = path.join(wasmDir, "ddonirang_tool_bg.wasm");
  assert.ok(fs.existsSync(jsPath) && fs.statSync(jsPath).isFile(), `WASM glue missing: ${jsPath}`);
  assert.ok(fs.existsSync(bytesPath) && fs.statSync(bytesPath).isFile(), `WASM bytes missing: ${bytesPath}`);
  const wasm = await import(`${pathToFileURL(jsPath).href}?g14=${Date.now()}`);
  const bytes = await fsp.readFile(bytesPath);
  if (typeof wasm.default === "function") await wasm.default({ module_or_path: bytes });
  assert.equal(typeof wasm.wasm_run_summary_json, "function", "wasm_run_summary_json export missing");
  assert.equal(
    typeof wasm.DdnWasmVm?.fromPreV25Source,
    "undefined",
    "compatibility constructor must remain excluded from the current product WASM",
  );
  return { wasm, wasmDir, artifactBytes: bytes.length };
}

const contract = JSON.parse(await fsp.readFile(contractPath, "utf8"));
assert.equal(contract.schema, "ddn.current_public_rejection_parity.contract.v1_candidate");
assert.equal(contract.classification, "CURRENT_PUBLIC_EXPECTED_REJECTION_PARITY");
assert.equal(contract.expected_case_count, 3);
assert.deepEqual(contract.cases.map((entry) => entry.id), contract.expected_case_ids);
assert.equal(new Set(contract.expected_case_ids).size, 3, "exact case IDs must be unique");

const cli = resolveCli();
const { wasm, wasmDir, artifactBytes } = await loadWasm();
const tempDir = await fsp.mkdtemp(path.join(os.tmpdir(), "ddn-g14-rejection-parity-"));
const results = [];

try {
  for (const entry of contract.cases) {
    const sourcePath = path.resolve(root, entry.input);
    const source = await fsp.readFile(sourcePath, "utf8");
    assert.match(source, /보임\s*\{/u, `${entry.id}: preserved legacy surface missing`);
    const summaryPath = path.join(tempDir, `${entry.id}.summary.detjson`);
    const native = invoke(cli, [
      "run-summary",
      entry.input,
      "--madi",
      String(entry.ticks),
      "--summary-json",
      summaryPath,
    ]);
    assert.equal(native.status, entry.expected.exit_code, `${entry.id}: native exit`);
    assert.equal(String(native.stdout ?? "").trim(), "", `${entry.id}: native partial output`);
    assert.equal(fs.existsSync(summaryPath), false, `${entry.id}: native partial summary/state committed`);
    const nativeDiagnostic = normalizeDiagnostic(
      parseStructuredDiagnostic(native.stderr, `${entry.id}.native`),
    );

    let wasmError = null;
    let wasmReturnedSummary = false;
    try {
      wasm.wasm_run_summary_json(source, entry.input, Number(entry.ticks));
      wasmReturnedSummary = true;
    } catch (error) {
      wasmError = error;
    }
    assert.equal(wasmReturnedSummary, false, `${entry.id}: WASM fake success`);
    const wasmDiagnostic = normalizeDiagnostic(
      parseStructuredDiagnostic(wasmError?.message ?? wasmError, `${entry.id}.wasm`),
    );

    const expectedDiagnostic = exactExpected(entry.expected, entry.input);
    assert.deepEqual(nativeDiagnostic, expectedDiagnostic, `${entry.id}: native exact diagnostic`);
    assert.deepEqual(wasmDiagnostic, expectedDiagnostic, `${entry.id}: WASM exact diagnostic`);
    assert.deepEqual(nativeDiagnostic, wasmDiagnostic, `${entry.id}: native/WASM diagnostic parity`);

    const atomicity = {
      state_mutation_count: fs.existsSync(summaryPath) ? 1 : 0,
      partial_output_count: String(native.stdout ?? "").trim() ? 1 : 0,
      partial_effect_count: 0,
      compatibility_fallback_count: 0,
    };
    for (const [key, actual] of Object.entries(atomicity)) {
      assert.equal(actual, entry.expected[key], `${entry.id}: ${key}`);
    }

    results.push({
      id: entry.id,
      classification: contract.classification,
      input: entry.input,
      native_exit_code: native.status,
      wasm_exit_code: entry.expected.exit_code,
      diagnostic: nativeDiagnostic,
      atomicity,
      native_wasm_exact_parity: true,
    });
  }
} finally {
  await fsp.rm(tempDir, { recursive: true, force: true });
}

const report = {
  schema: "ddn.current_public_rejection_parity.report.v1_candidate",
  status: "PASS",
  classification: contract.classification,
  case_count: results.length,
  cases: results,
  runtime_owner: contract.runtime_owner,
  cli,
  wasm_dir: wasmDir,
  wasm_artifact_bytes: artifactBytes,
  compatibility_fallback_count: 0,
  mutation_or_product_change: 0,
};
if (reportPath) await fsp.writeFile(path.resolve(reportPath), `${JSON.stringify(report, null, 2)}\n`, "utf8");
console.log(JSON.stringify(report));
