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
  단계 <- 0.
}.

(매마디)마다 {
  단계 <- 단계 + 1.
  (피해량:10) 피해_받음 ~~> 플레이어.
}.
"""


def probe() -> dict:
    script = rf"""
const fs = require("fs");
const path = require("path");
const {{ pathToFileURL }} = require("url");

function snapshot(client) {{
  const after = client.columnsParsed();
  return {{
    row: Array.isArray(after?.row) ? after.row.map((value) => String(value ?? "")) : [],
    columns: Array.isArray(after?.columns) ? after.columns.map((row) => String(row?.key ?? "")) : [],
  }};
}}

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
    const states = [];
    for (let i = 0; i < 3; i += 1) {{
      const stepped = client.stepOneParsed();
      const snap = snapshot(client);
      states.push({{
        schema: String(stepped?.schema ?? ""),
        state_hash: String(stepped?.state_hash ?? ""),
        frame_id: Number(stepped?.frame_id ?? -1),
        columns: snap.columns,
        row: snap.row,
      }});
    }}
    console.log(JSON.stringify({{ ok: true, states }}));
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


def value_at(state: dict, key: str) -> str | None:
    columns = [str(name) for name in state.get("columns", [])]
    values = [str(value) for value in state.get("row", [])]
    try:
        idx = columns.index(key)
    except ValueError:
        return None
    if idx >= len(values):
        return None
    return values[idx]


def main() -> int:
    payload = probe()
    if not bool(payload.get("ok")):
        print(
            "check=seamgrim_generic_imja_every_raw_wasm "
            f"detail=unexpected_error:{str(payload.get('message', '')).strip()}"
        )
        return 1
    states = payload.get("states", [])
    if len(states) != 3:
        print(
            "check=seamgrim_generic_imja_every_raw_wasm "
            f"detail=bad_state_count:{len(states)}"
        )
        return 1
    hp_values = []
    stage_values = []
    for idx, state in enumerate(states):
        if str(state.get("schema", "")).strip() != "seamgrim.state.v0":
            print(
                "check=seamgrim_generic_imja_every_raw_wasm "
                f"detail=bad_schema:step{idx}:{state.get('schema')}"
            )
            return 1
        if not str(state.get("state_hash", "")).strip():
            print(
                "check=seamgrim_generic_imja_every_raw_wasm "
                f"detail=missing_hash:step{idx}"
            )
            return 1
        hp = value_at(state, "체력")
        stage = value_at(state, "단계")
        if hp is None:
            print(
                "check=seamgrim_generic_imja_every_raw_wasm "
                f"detail=missing_column:체력:step{idx}:{','.join(state.get('columns', []))}"
            )
            return 1
        if stage is None:
            print(
                "check=seamgrim_generic_imja_every_raw_wasm "
                f"detail=missing_column:단계:step{idx}:{','.join(state.get('columns', []))}"
            )
            return 1
        hp_values.append(hp)
        stage_values.append(stage)
    if hp_values != ["90", "80", "70"]:
        print(
            "check=seamgrim_generic_imja_every_raw_wasm "
            f"detail=bad_hp_sequence:{','.join(hp_values)}"
        )
        return 1
    if stage_values != ["1", "2", "3"]:
        print(
            "check=seamgrim_generic_imja_every_raw_wasm "
            f"detail=bad_stage_sequence:{','.join(stage_values)}"
        )
        return 1
    print("seamgrim generic imja every raw wasm check ok hp=90,80,70 stage=1,2,3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
