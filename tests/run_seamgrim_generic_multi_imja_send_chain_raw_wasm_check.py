#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = r"""
(전달량:수) 전달:알림씨 = {
}.

출발:알림씨 = {
}.

전달자:임자 = {
  제.기본량 <- 2.

  출발을 받으면 {
    (전달량:제.기본량) 전달 ~~> 중계자.
  }.
}.

중계자:임자 = {
  제.보정량 <- 4.

  전달을 받으면 {
    (전달량:(정보.전달량 + 제.보정량)) 전달 ~~> 목표.
  }.
}.

목표:임자 = {
  제.체력 <- 30.

  전달을 받으면 {
    제.체력 <- 제.체력 - 정보.전달량.
  }.
}.

(시작)할때 {
  단계 <- 0.
}.

(매마디)마다 {
  단계 <- 단계 + 1.
  출발 ~~> 전달자.
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
            "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
            f"detail=unexpected_error:{str(payload.get('message', '')).strip()}"
        )
        return 1
    states = payload.get("states", [])
    if len(states) != 3:
        print(
            "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
            f"detail=bad_state_count:{len(states)}"
        )
        return 1
    target_values = []
    stage_values = []
    sender_values = []
    relay_values = []
    for idx, state in enumerate(states):
        if str(state.get("schema", "")).strip() != "seamgrim.state.v0":
            print(
                "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
                f"detail=bad_schema:step{idx}:{state.get('schema')}"
            )
            return 1
        if not str(state.get("state_hash", "")).strip():
            print(
                "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
                f"detail=missing_hash:step{idx}"
            )
            return 1
        target = value_at(state, "목표__체력")
        stage = value_at(state, "단계")
        sender = value_at(state, "전달자__기본량")
        relay = value_at(state, "중계자__보정량")
        if target is None or stage is None or sender is None or relay is None:
            print(
                "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
                f"detail=missing_columns:step{idx}:{','.join(state.get('columns', []))}"
            )
            return 1
        target_values.append(target)
        stage_values.append(stage)
        sender_values.append(sender)
        relay_values.append(relay)
    if target_values != ["24", "18", "12"]:
        print(
            "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
            f"detail=bad_target_sequence:{','.join(target_values)}"
        )
        return 1
    if stage_values != ["1", "2", "3"]:
        print(
            "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
            f"detail=bad_stage_sequence:{','.join(stage_values)}"
        )
        return 1
    if sender_values != ["2", "2", "2"]:
        print(
            "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
            f"detail=bad_sender_sequence:{','.join(sender_values)}"
        )
        return 1
    if relay_values != ["4", "4", "4"]:
        print(
            "check=seamgrim_generic_multi_imja_send_chain_raw_wasm "
            f"detail=bad_relay_sequence:{','.join(relay_values)}"
        )
        return 1
    print("seamgrim generic multi imja send chain raw wasm check ok target=24,18,12 stage=1,2,3 sender=2 relay=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
