import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packRoot = path.join(repoRoot, "pack", "ddn_execution_concurrency_w1c1_sender_route_provider");
const cliPath = path.resolve(String(process.env.DDN_W1C1_CLI ?? "").trim());
const wasmDir = path.resolve(String(process.env.DDN_W1C1_WASM_DIR ?? "").trim());
const trackedWasmDir = path.resolve(String(process.env.DDN_W1C1_TRACKED_WASM_DIR ?? "").trim());
const requireTracked = String(process.env.DDN_W1C1_REQUIRE_TRACKED ?? "1") !== "0";

assert(String(process.env.DDN_W1C1_CLI ?? "").trim(), "DDN_W1C1_CLI is required");
assert(String(process.env.DDN_W1C1_WASM_DIR ?? "").trim(), "DDN_W1C1_WASM_DIR is required");
if (requireTracked) {
  assert(
    String(process.env.DDN_W1C1_TRACKED_WASM_DIR ?? "").trim(),
    "DDN_W1C1_TRACKED_WASM_DIR is required",
  );
}

const expected = fs
  .readFileSync(path.join(packRoot, "expected.jsonl"), "utf8")
  .trim()
  .split(/\r?\n/u)
  .map((line) => JSON.parse(line));
assert.equal(expected.length, 10, "W1-C1 oracle case count");
assert.equal(new Set(expected.map((row) => row.case)).size, expected.length, "case IDs unique");

const require = createRequire(import.meta.url);
const sourceBuilt = require(path.join(wasmDir, "ddonirang_tool.js"));
const tracked = requireTracked
  ? await import(pathToFileURL(path.join(trackedWasmDir, "ddonirang_tool.js")).href)
  : null;
if (tracked) {
  tracked.initSync({ module: fs.readFileSync(path.join(trackedWasmDir, "ddonirang_tool_bg.wasm")) });
}

function relativeSource(caseId) {
  return `pack/ddn_execution_concurrency_w1c1_sender_route_provider/${caseId}/input.ddn`;
}

function runCli(args) {
  const result = spawnSync(cliPath, args, {
    cwd: repoRoot,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 32 * 1024 * 1024,
  });
  return {
    status: result.status,
    stdout: result.stdout.replaceAll("\r\n", "\n"),
    stderr: result.stderr.replaceAll("\r\n", "\n"),
  };
}

function nativeCanonPresentation(canonicalBody) {
  return canonicalBody.endsWith("\n") ? canonicalBody : `${canonicalBody}\n`;
}

assert.equal(nativeCanonPresentation("본문\n"), "본문\n", "existing canonical LF is not duplicated");

function wasmRun(runtime, source, sourceRef) {
  try {
    return { ok: true, summary: JSON.parse(runtime.wasm_run_summary_json(source, sourceRef, 1)) };
  } catch (error) {
    return { ok: false, error: String(error) };
  }
}

function structuredDiagnostic(text, label) {
  const start = text.indexOf("{");
  assert.notEqual(start, -1, `${label}: structured diagnostic JSON`);
  return JSON.parse(text.slice(start));
}

function assertCandidateDiagnostic(text, row, label) {
  const diagnostic = structuredDiagnostic(text, label);
  assert.equal(diagnostic.code, row.diagnostic, `${label}: candidate code`);
  assert.equal(diagnostic.phase, row.phase, `${label}: structured phase`);
  assert.equal(Number.isInteger(diagnostic.source_span?.start), true, `${label}: span start`);
  assert.equal(Number.isInteger(diagnostic.source_span?.end), true, `${label}: span end`);
  assert.equal(diagnostic.source_span.end > diagnostic.source_span.start, true, `${label}: span width`);
}

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ddn-w1c1-"));
const results = [];
try {
  for (const row of expected) {
    const sourceRef = relativeSource(row.case);
    const source = fs.readFileSync(path.join(repoRoot, sourceRef), "utf8");
    if (row.mode === "canon") {
      const native = runCli(["canon", sourceRef]);
      assert.equal(native.status, 0, `${row.case}: native canon`);
      assert.equal(native.stdout.includes(row.canonical_contains), true, `${row.case}: canonical surface`);
      const sourceCanonBody = sourceBuilt.wasm_canon_ddn(source);
      assert.equal(
        native.stdout,
        nativeCanonPresentation(sourceCanonBody),
        `${row.case}: source WASM canon with native presentation LF`,
      );
      if (tracked) {
        const trackedCanonBody = tracked.wasm_canon_ddn(source);
        assert.equal(trackedCanonBody, sourceCanonBody, `${row.case}: source/tracked WASM canon body`);
        assert.equal(
          native.stdout,
          nativeCanonPresentation(trackedCanonBody),
          `${row.case}: tracked WASM canon with native presentation LF`,
        );
      }
      results.push({ case: row.case, outcome: "success", parity: true });
      continue;
    }

    const summaryPath = path.join(tempRoot, `${row.case}.summary.json`);
    const native = runCli(["run", sourceRef, "--madi", "1", "--summary-json", summaryPath]);
    const sourceWasm = wasmRun(sourceBuilt, source, sourceRef);
    const trackedWasm = tracked ? wasmRun(tracked, source, sourceRef) : null;

    if (row.outcome === "success") {
      assert.equal(native.status, 0, `${row.case}: native success: ${native.stderr}`);
      assert.equal(sourceWasm.ok, true, `${row.case}: source WASM success: ${sourceWasm.error}`);
      if (trackedWasm) {
        assert.equal(trackedWasm.ok, true, `${row.case}: tracked WASM success: ${trackedWasm.error}`);
      }
      const nativeSummary = JSON.parse(fs.readFileSync(summaryPath, "utf8"));
      assert.deepEqual(sourceWasm.summary, nativeSummary, `${row.case}: native/source WASM summary`);
      if (trackedWasm) {
        assert.deepEqual(trackedWasm.summary, nativeSummary, `${row.case}: native/tracked WASM summary`);
      }
      assert.deepEqual(nativeSummary.output_log_texts, row.output_log_texts, `${row.case}: output oracle`);
    } else {
      assert.equal(native.status, 1, `${row.case}: native failure`);
      assert.equal(fs.existsSync(summaryPath), false, `${row.case}: failure writes no summary`);
      assertCandidateDiagnostic(native.stderr, row, `${row.case}: native diagnostic`);
      assert.equal(sourceWasm.ok, false, `${row.case}: source WASM must fail`);
      if (trackedWasm) {
        assert.equal(trackedWasm.ok, false, `${row.case}: tracked WASM must fail`);
      }
      assertCandidateDiagnostic(sourceWasm.error, row, `${row.case}: source WASM diagnostic`);
      if (trackedWasm) {
        assertCandidateDiagnostic(trackedWasm.error, row, `${row.case}: tracked WASM diagnostic`);
      }
    }
    results.push({ case: row.case, outcome: row.outcome, parity: true });
  }
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}

console.log(JSON.stringify({
  schema: "ddn.execution_concurrency.w1c1.sender_route_provider.parity.v1",
  cases: results.length,
  success_cases: results.filter((row) => row.outcome === "success").length,
  failure_cases: results.filter((row) => row.outcome === "failure").length,
  source_built_wasm: true,
  tracked_workbench_wasm: Boolean(tracked),
  reaction_provider_public_surface: "OPEN",
}));
