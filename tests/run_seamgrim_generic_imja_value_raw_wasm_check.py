#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = r"""
(피해량:수) 피해_받음:알림씨 = {
}.

플레이어:임자 = {
  제.체력 <- 100.

  피해_받음을 받으면 {
    제.체력 <- 제.체력 - 정보.피해량.
  }.
}.

(시작)할때 {
  (피해량:30) 피해_받음 ~~> 플레이어.
}.
"""


def probe() -> dict:
    script = rf"""
const fs = require("fs");
const path = require("path");
const {{ pathToFileURL }} = require("url");

async function main() {{
  const root = process.cwd();
  const uiDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModuleUrl = pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href;
  const wrapperUrl = pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href;
  const wasmPath = path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm");
  const wasmModule = await import(wasmModuleUrl);
  const wrapper = await import(wrapperUrl);
  const wasmBytes = fs.readFileSync(wasmPath);
  if (typeof wasmModule.default === "function") {{
    await wasmModule.default({{ module_or_path: wasmBytes }});
  }}

  try {{
    const vm = new wasmModule.DdnWasmVm({json.dumps(SOURCE)});
    const client = new wrapper.DdnWasmVmClient(vm);
    const stepped = client.stepOneParsed();
    const after = client.columnsParsed();
    console.log(JSON.stringify({{
      ok: true,
      state_schema: String(stepped?.schema ?? ""),
      state_hash: String(stepped?.state_hash ?? ""),
      row: Array.isArray(after?.row) ? after.row.map((value) => String(value ?? "")) : [],
      columns: Array.isArray(after?.columns) ? after.columns.map((row) => String(row?.key ?? "")) : [],
    }}));
  }} catch (err) {{
    console.log(JSON.stringify({{
      ok: false,
      message: String((err && err.message) || err),
    }}));
  }}
}}

main().catch((err) => {{
  console.error(String((err && err.stack) || err));
  process.exit(1);
}});
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
    return json.loads(proc.stdout)


def main() -> int:
    row = probe()
    if not bool(row.get("ok")):
        print(
            "check=seamgrim_generic_imja_value_raw_wasm "
            f"detail=unexpected_error:{str(row.get('message', '')).strip()}"
        )
        return 1
    if str(row.get("state_schema", "")).strip() != "seamgrim.state.v0":
        print(
            "check=seamgrim_generic_imja_value_raw_wasm "
            f"detail=bad_schema:{row.get('state_schema')}"
        )
        return 1
    if not str(row.get("state_hash", "")).strip():
        print("check=seamgrim_generic_imja_value_raw_wasm detail=missing_hash")
        return 1
    columns = [str(key) for key in row.get("columns", [])]
    values = [str(value) for value in row.get("row", [])]
    try:
        idx = columns.index("체력")
    except ValueError:
        print(
            "check=seamgrim_generic_imja_value_raw_wasm "
            f"detail=missing_column:체력:{','.join(columns)}"
        )
        return 1
    if idx >= len(values) or values[idx] != "70":
        actual = values[idx] if idx < len(values) else ""
        print(
            "check=seamgrim_generic_imja_value_raw_wasm "
            f"detail=bad_value:체력:{actual}:expected=70"
        )
        return 1
    print("seamgrim generic imja value raw wasm check ok hp=70")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
