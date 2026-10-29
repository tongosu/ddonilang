#!/usr/bin/env node

import fs from "node:fs/promises";
import fsSync from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const LESSON_IDS = [
  "elem_math_number_line",
  "elem_math_fraction_line",
  "elem_math_rectangle_area",
  "elem_science_daily_temperature",
  "elem_econ_allowance_budget",
  "elem_econ_store_stock",
  "rep_math_function_line_v1",
  "rep_phys_pendulum_wave1_v1",
  "rep_phys_projectile_xy_v1",
  "rep_econ_supply_demand_tax_v1",
  "rep_phys_damped_oscillation_v1",
  "high_math_derivative",
];

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function normalizeDdn(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").trim();
}

function sameValue(a, b) {
  if (typeof a === "number" && typeof b === "number") {
    return Math.abs(a - b) <= 1e-7;
  }
  return JSON.stringify(a) === JSON.stringify(b);
}

function comparison(a, b) {
  return stableJson(a) === stableJson(b) ? "equal" : "domain_mismatch_reported";
}

function stableJson(value) {
  return JSON.stringify(stableValue(value));
}

function stableValue(value) {
  if (Array.isArray(value)) {
    return value.map(stableValue);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, stableValue(value[key])]),
    );
  }
  return value;
}

function indexRow(channels, row) {
  const out = {};
  const channelList = Array.isArray(channels) ? channels : [];
  const rowList = Array.isArray(row) ? row : [];
  channelList.forEach((channel, idx) => {
    const key = String(channel?.key ?? "").trim();
    if (key) {
      out[key] = rowList[idx];
    }
  });
  return out;
}

function warningCodes(warnings) {
  return (Array.isArray(warnings) ? warnings : []).map((warning) => String(warning?.code ?? ""));
}

function normalizeOutputRows(rows) {
  if (!Array.isArray(rows)) return [];
  return rows.map((row) => ({
    key: String(row?.key ?? ""),
    value: String(row?.value ?? ""),
    source: String(row?.source ?? ""),
  }));
}

async function initRuntime(rootDir) {
  const uiDir = path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModule = await import(pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href);
  const wrapper = await import(pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href);
  const runtimeState = await import(pathToFileURL(path.join(uiDir, "seamgrim_runtime_state.js")).href);
  const wasmBytes = await fs.readFile(path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm"));
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }
  assert(typeof wasmModule.DdnWasmVm === "function", "DdnWasmVm export missing");
  assert(typeof wasmModule.wasm_canon_ddn === "function", "wasm_canon_ddn export missing");
  assert(typeof wasmModule.wasm_execution_envelope_json === "function", "wasm_execution_envelope_json export missing");
  assert(typeof wrapper.DdnWasmVmClient === "function", "DdnWasmVmClient export missing");
  return { wasmModule, wrapper, runtimeState };
}

function runCli(rootDir, lessonRel, summaryPath) {
  const proc = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "tool/Cargo.toml",
      "--",
      "run-summary",
      lessonRel,
      "--madi",
      "2",
      "--summary-json",
      summaryPath,
    ],
    { cwd: rootDir, encoding: "utf8", windowsHide: true, env: { ...process.env, RUSTFLAGS: "-Awarnings" } },
  );
  if (proc.status !== 0) {
    return {
      exitCode: proc.status ?? 1,
      stdout: String(proc.stdout ?? ""),
      stderr: String(proc.stderr ?? ""),
      summary: null,
    };
  }
  return {
    exitCode: 0,
    stdout: String(proc.stdout ?? ""),
    stderr: String(proc.stderr ?? ""),
    summary: JSON.parse(fsSync.readFileSync(summaryPath, "utf8")),
  };
}

async function runWasm(runtime, sourceText, ticks) {
  const vm = new runtime.wasmModule.DdnWasmVm(sourceText);
  const client = new runtime.wrapper.DdnWasmVmClient(vm);
  try {
    const executionEnvelope = JSON.parse(
      runtime.wasmModule.wasm_execution_envelope_json(sourceText, Number(ticks)),
    );
    const state = client.runTicksParsed(ticks);
    const outputLog = runtime.runtimeState.extractObservationOutputLogFromState(state)
      .map((entry) => String(entry?.text ?? ""));
    const outputRows = normalizeOutputRows(runtime.runtimeState.extractObservationOutputRowsFromState(state));
    return {
      canonical_ddn: normalizeDdn(runtime.wasmModule.wasm_canon_ddn(sourceText)),
      preprocessed_source_hash_domain: "product_shared_frontdoor_preprocess",
      configured_ticks: client.configuredMadi(),
      tick_id: state.tick_id,
      executed_ticks: Number(state.tick_id ?? 0) + 1,
      state_hash: String(state.state_hash ?? ""),
      view_hash: String(state.view_hash ?? ""),
      final_row: indexRow(state.channels, state.row),
      output_log_texts: outputLog,
      output_rows: outputRows,
      parse_warnings: client.parseWarningsParsed(),
      input: state.input ?? null,
      resources: state.resources ?? {},
      execution_envelope: executionEnvelope,
    };
  } finally {
    if (typeof vm.free === "function") {
      vm.free();
    }
  }
}

function hydrateCliSummary(runtimeState, summary) {
  if (!summary || typeof summary !== "object") return summary;
  const state = summary.engine_state ?? summary.state ?? null;
  if (!state || typeof state !== "object") return summary;
  summary.final_row = indexRow(state.channels, state.row);
  summary.output_log_texts = runtimeState
    .extractObservationOutputLogFromState(state)
    .map((entry) => String(entry?.text ?? ""));
  summary.output_rows = normalizeOutputRows(
    runtimeState.extractObservationOutputRowsFromState(state),
  );
  return summary;
}

function compareLesson(lessonId, cli, wasm) {
  const failures = [];
  const summary = cli.summary ?? {};
  if (cli.exitCode !== 0) {
    failures.push(`cli exit ${cli.exitCode}: ${cli.stderr || cli.stdout}`);
    return { failures };
  }
  if (!normalizeDdn(summary.canonical_ddn) || !wasm.canonical_ddn) {
    failures.push("canonical_ddn missing");
  }
  const cliWarnings = warningCodes(summary.parse_warnings);
  const wasmWarnings = warningCodes(wasm.parse_warnings);
  if (JSON.stringify(cliWarnings) !== JSON.stringify(wasmWarnings)) {
    failures.push(`warning code mismatch cli=${cliWarnings.join(",")} wasm=${wasmWarnings.join(",")}`);
  }
  if (Number(summary.configured_ticks ?? 0) !== Number(wasm.configured_ticks ?? 0)) {
    failures.push(`configured ticks mismatch cli=${summary.configured_ticks} wasm=${wasm.configured_ticks}`);
  }
  if (Number(summary.effective_ticks ?? 0) !== 2 || Number(wasm.executed_ticks ?? 0) !== 2) {
    failures.push(`tick count mismatch cli=${summary.effective_ticks} wasm=${wasm.executed_ticks}`);
  }
  const missingKeys = [];
  const extraKeys = [];
  const mismatchedKeys = [];
  for (const [key, cliValue] of Object.entries(summary.final_row ?? {})) {
    if (!(key in wasm.final_row)) {
      missingKeys.push(key);
      continue;
    }
    if (!sameValue(cliValue, wasm.final_row[key])) {
      mismatchedKeys.push({ key, cli: cliValue, wasm: wasm.final_row[key] });
    }
  }
  for (const key of Object.keys(wasm.final_row ?? {})) {
    if (!(key in (summary.final_row ?? {}))) {
      extraKeys.push(key);
    }
  }
  if (missingKeys.length) {
    failures.push(`final row keys missing in wasm: ${missingKeys.join(",")}`);
  }
  if (extraKeys.length) {
    failures.push(`final row extra keys in wasm: ${extraKeys.join(",")}`);
  }
  if (mismatchedKeys.length) {
    failures.push(`final row mismatch: ${JSON.stringify(mismatchedKeys.slice(0, 4))}`);
  }
  const cliOutputRows = normalizeOutputRows(summary.output_rows ?? []);
  const cliOutputLog = (Array.isArray(summary.output_log_texts) ? summary.output_log_texts : []).map(String);
  if (!String(summary.state_hash ?? "").startsWith("blake3:")) {
    failures.push("cli state_hash missing");
  }
  if (!String(wasm.state_hash ?? "").startsWith("blake3:")) {
    failures.push("wasm state_hash missing");
  }
  if (
    String(summary.state_hash ?? "") &&
    String(wasm.state_hash ?? "") &&
    String(summary.state_hash ?? "") !== String(wasm.state_hash ?? "")
  ) {
    failures.push("state_hash mismatch");
  }
  if (wasm.input?.frame_source !== "사람" || wasm.input?.frame_source_code !== 0) {
    failures.push("wasm input frame source missing");
  }
  if (stableJson(summary.execution_envelope ?? null) !== stableJson(wasm.execution_envelope ?? null)) {
    failures.push("execution envelope mismatch");
  }
  return { failures };
}

async function main() {
  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const runtime = await initRuntime(rootDir);
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "ddn-foundation12-cli-wasm-"));
  const rows = [];
  try {
    for (const lessonId of LESSON_IDS) {
      const lessonRel = `solutions/seamgrim_ui_mvp/lessons/${lessonId}/lesson.ddn`;
      const sourceText = await fs.readFile(path.join(rootDir, lessonRel), "utf8");
      const summaryPath = path.join(tmpDir, `${lessonId}.summary.json`);
      const cli = runCli(rootDir, lessonRel, summaryPath);
      if (cli.summary) {
        hydrateCliSummary(runtime.runtimeState, cli.summary);
      }
      const wasm = await runWasm(runtime, sourceText, 2);
      const { failures } = compareLesson(lessonId, cli, wasm);
      rows.push({
        lesson_id: lessonId,
        ok: failures.length === 0,
        failures,
        cli: {
          configured_ticks: cli.summary?.configured_ticks ?? null,
          effective_ticks: cli.summary?.effective_ticks ?? null,
          state_hash: cli.summary?.state_hash ?? "",
          trace_hash: cli.summary?.trace_hash ?? "",
          final_row_key_count: Object.keys(cli.summary?.final_row ?? {}).length,
          output_row_count: Array.isArray(cli.summary?.output_rows) ? cli.summary.output_rows.length : 0,
        },
        wasm: {
          configured_ticks: wasm.configured_ticks,
          tick_id: wasm.tick_id,
          executed_ticks: wasm.executed_ticks,
          state_hash: wasm.state_hash,
          view_hash: wasm.view_hash,
          final_row_key_count: Object.keys(wasm.final_row ?? {}).length,
          output_row_count: wasm.output_rows.length,
          input_frame_source: wasm.input?.frame_source ?? "",
        },
        strict_evidence: {
          canonical_ddn_comparison: comparison(normalizeDdn(cli.summary?.canonical_ddn), wasm.canonical_ddn),
          warning_codes_equal: JSON.stringify(warningCodes(cli.summary?.parse_warnings)) === JSON.stringify(warningCodes(wasm.parse_warnings)),
          configured_ticks_equal: Number(cli.summary?.configured_ticks ?? 0) === Number(wasm.configured_ticks ?? 0),
          final_row_cli_subset_equal: failures.every((failure) => !String(failure).startsWith("final row")),
          final_row_comparison: comparison(cli.summary?.final_row ?? {}, wasm.final_row ?? {}),
          output_rows_comparison: comparison(normalizeOutputRows(cli.summary?.output_rows ?? []), wasm.output_rows),
          output_log_comparison: comparison((cli.summary?.output_log_texts ?? []).map(String), wasm.output_log_texts),
          execution_envelope_comparison: comparison(cli.summary?.execution_envelope ?? null, wasm.execution_envelope ?? null),
          state_hash_comparison: comparison(cli.summary?.state_hash ?? "", wasm.state_hash ?? ""),
          state_hash_policy: "strict_equality",
          state_hash_formula_change: false,
        },
      });
    }
  } finally {
    await fs.rm(tmpDir, { recursive: true, force: true });
  }
  const report = {
    schema: "seamgrim.foundation12.cli_raw_wasm_parity.report.v1",
    ok: rows.every((row) => row.ok),
    lesson_count: rows.length,
    state_hash_policy: "strict_equality",
    state_hash_formula_change: false,
    full_single_runtime_closure_claim: false,
    lessons: rows,
  };
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  if (!report.ok) {
    process.exit(1);
  }
}

main().catch((err) => {
  process.stderr.write(`${err?.stack ?? err}\n`);
  process.exit(1);
});
