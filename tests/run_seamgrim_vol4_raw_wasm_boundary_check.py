#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def run_node_boundary_probe() -> dict:
    script = r"""
const fs = require("fs");
const path = require("path");
const { pathToFileURL } = require("url");

async function initRuntime(root) {
  const uiDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModuleUrl = pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href;
  const wrapperUrl = pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href;
  const commonUrl = pathToFileURL(path.join(uiDir, "wasm_page_common.js")).href;
  const wasmPath = path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm");

  const wasmModule = await import(wasmModuleUrl);
  const wrapper = await import(wrapperUrl);
  const common = await import(commonUrl);
  const wasmBytes = fs.readFileSync(wasmPath);

  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }
  return { wasmModule, wrapper, common };
}

async function columnsSmoke(runtime, root, relPath) {
  const sourceRaw = fs.readFileSync(path.join(root, relPath), "utf8");
  const source = runtime.common.stripMetaHeader(sourceRaw);
  const vm = new runtime.wasmModule.DdnWasmVm(source);
  const client = new runtime.wrapper.DdnWasmVmClient(vm);
  const before = client.columnsParsed();
  const stepped = client.stepOneParsed();
  const after = client.columnsParsed();
  return {
    ok: true,
    before_columns: Array.isArray(before?.columns) ? before.columns.map((row) => String(row?.key ?? "")) : [],
    after_columns: Array.isArray(after?.columns) ? after.columns.map((row) => String(row?.key ?? "")) : [],
    state_schema: String(stepped?.schema ?? ""),
    state_hash: String(stepped?.state_hash ?? ""),
  };
}

async function boundaryFail(runtime, root, relPath) {
  const sourceRaw = fs.readFileSync(path.join(root, relPath), "utf8");
  const source = runtime.common.stripMetaHeader(sourceRaw);
  try {
    const vm = new runtime.wasmModule.DdnWasmVm(source);
    const client = new runtime.wrapper.DdnWasmVmClient(vm);
    const stepped = client.stepOneParsed();
    const after = client.columnsParsed();
    return {
      ok: true,
      message: "",
      state_schema: String(stepped?.schema ?? ""),
      state_hash: String(stepped?.state_hash ?? ""),
      after_columns: Array.isArray(after?.columns) ? after.columns.map((row) => String(row?.key ?? "")) : [],
    };
  } catch (err) {
    return { ok: false, message: String((err && err.message) || err) };
  }
}

(async () => {
  const root = process.cwd();
  const runtime = await initRuntime(root);
  const positive = await columnsSmoke(runtime, root, "pack/seamgrim_interactive_event_smoke_v1/fixtures/lesson.ddn");
  const negativePaths = [
    "solutions/seamgrim_ui_mvp/lessons/rep_ddonirang_vol4_event_dispatch_v1/lesson.ddn",
    "solutions/seamgrim_ui_mvp/lessons/rep_ddonirang_vol4_state_transition_v1/lesson.ddn",
    "solutions/seamgrim_ui_mvp/lessons/rep_ddonirang_vol4_resume_isolation_v1/lesson.ddn",
    "solutions/seamgrim_ui_mvp/lessons/rep_ddonirang_vol4_multi_signal_priority_v1/lesson.ddn",
  ];
  const negatives = {};
  for (const relPath of negativePaths) {
    negatives[relPath] = await boundaryFail(runtime, root, relPath);
  }
  console.log(JSON.stringify({ positive, negatives }));
})().catch((err) => {
  console.error(String((err && err.stack) || err));
  process.exit(1);
});
"""
    proc = subprocess.run(
        ["node", "-"],
        cwd=ROOT,
        input=script,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
        raise RuntimeError(detail)
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"boundary probe json decode failed: {exc}") from exc


def main() -> int:
    payload = run_node_boundary_probe()
    positive = payload.get("positive", {})
    if not positive.get("ok"):
        print("check=seamgrim_vol4_raw_wasm_boundary detail=interactive_event_positive_missing")
        return 1
    columns = [str(item) for item in positive.get("after_columns", [])]
    if str(positive.get("state_schema", "")).strip() != "seamgrim.state.v0":
        print(
            "check=seamgrim_vol4_raw_wasm_boundary "
            f"detail=interactive_event_bad_schema:{positive.get('state_schema')}"
        )
        return 1
    if not str(positive.get("state_hash", "")).strip():
        print("check=seamgrim_vol4_raw_wasm_boundary detail=interactive_event_missing_hash")
        return 1
    if "x" not in columns:
        print(
            "check=seamgrim_vol4_raw_wasm_boundary "
            f"detail=interactive_event_missing_columns:{','.join(columns)}"
        )
        return 1

    negatives = payload.get("negatives", {})
    if not isinstance(negatives, dict) or not negatives:
        print("check=seamgrim_vol4_raw_wasm_boundary detail=representative_cases_missing")
        return 1

    expected_columns = {
        "solutions/seamgrim_ui_mvp/lessons/rep_ddonirang_vol4_event_dispatch_v1/lesson.ddn": ["모드", "마지막", "처리횟수"],
        "solutions/seamgrim_ui_mvp/lessons/rep_ddonirang_vol4_state_transition_v1/lesson.ddn": ["현재상태", "체력"],
        "solutions/seamgrim_ui_mvp/lessons/rep_ddonirang_vol4_resume_isolation_v1/lesson.ddn": ["모드", "격리됨", "처리건수", "보류건수"],
        "solutions/seamgrim_ui_mvp/lessons/rep_ddonirang_vol4_multi_signal_priority_v1/lesson.ddn": ["격리됨", "마지막", "일반처리", "차단수"],
    }

    for rel_path, row in negatives.items():
        if not bool(row.get("ok")):
            print(
                "check=seamgrim_vol4_raw_wasm_boundary "
                f"detail=unexpected_error:{rel_path}:{str(row.get('message', '')).strip()}"
            )
            return 1
        if str(row.get("state_schema", "")).strip() != "seamgrim.state.v0":
            print(
                "check=seamgrim_vol4_raw_wasm_boundary "
                f"detail=bad_schema:{rel_path}:{row.get('state_schema')}"
            )
            return 1
        if not str(row.get("state_hash", "")).strip():
            print(
                "check=seamgrim_vol4_raw_wasm_boundary "
                f"detail=missing_hash:{rel_path}"
            )
            return 1
        columns = [str(item) for item in row.get("after_columns", [])]
        for column in expected_columns.get(rel_path, []):
            if column not in columns:
                print(
                    "check=seamgrim_vol4_raw_wasm_boundary "
                    f"detail=missing_column:{rel_path}:{column}:{','.join(columns)}"
                )
                return 1

    print(
        "seamgrim vol4 raw wasm boundary check ok "
        f"positive_columns={len(columns)} representative_lessons={len(negatives)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
