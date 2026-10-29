#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TRUE_SOURCE = r"""
(값:수) 첫알림:알림씨 = {
}.

(값:수) 둘알림:알림씨 = {
}.

(값:수) 셋알림:알림씨 = {
}.

(값:수) 넷알림:알림씨 = {
}.

관제탑:임자 = {
  제.순서 <- 0.

  첫알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 1.
    고르기:
      정보.값 > 0: {
        (값:정보.값) 둘알림 ~~> 제.
      }
      정보.값 >= 0: {
        (값:정보.값) 넷알림 ~~> 제.
      }
      아니면: {
        (값:정보.값) 셋알림 ~~> 제.
      }.
  }.

  (알림 알림.이름 == "첫알림")인 알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 2.
  }.

  둘알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 3.
  }.

  셋알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 5.
  }.

  넷알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 6.
  }.

  알림을 받으면 {
    제.순서 <- 제.순서 * 10 + 4.
  }.
}.

(시작)할때 {
  (값:1) 첫알림 ~~> 관제탑.
}.
"""
SECOND_SOURCE = TRUE_SOURCE.replace("(값:1) 첫알림", "(값:0) 첫알림")
ELSE_SOURCE = TRUE_SOURCE.replace("(값:1) 첫알림", "(값:(0 - 1)) 첫알림")
EXHAUSTIVE_SOURCE = TRUE_SOURCE.replace(
    """    고르기:
      정보.값 > 0: {
        (값:정보.값) 둘알림 ~~> 제.
      }
      정보.값 >= 0: {
        (값:정보.값) 넷알림 ~~> 제.
      }
      아니면: {
        (값:정보.값) 셋알림 ~~> 제.
      }.
""",
    """    고르기:
      정보.값 > 0 인 경우 {
        (값:정보.값) 둘알림 ~~> 제.
      }
      정보.값 >= 0 인 경우 {
        (값:정보.값) 넷알림 ~~> 제.
      }
      모든 경우 다룸.
""",
)
EXHAUSTIVE_SECOND_SOURCE = EXHAUSTIVE_SOURCE.replace("(값:1) 첫알림", "(값:0) 첫알림")


def probe(source: str) -> dict:
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
    const vm = new wasmModule.DdnWasmVm({json.dumps(source)});
    const client = new wrapper.DdnWasmVmClient(vm);
    const stepped = client.stepOneParsed();
    const snap = snapshot(client);
    console.log(JSON.stringify({{
      ok: true,
      schema: String(stepped?.schema ?? ""),
      state_hash: String(stepped?.state_hash ?? ""),
      columns: snap.columns,
      row: snap.row,
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


def value_at(payload: dict, key: str) -> str | None:
    columns = [str(name) for name in payload.get("columns", [])]
    values = [str(value) for value in payload.get("row", [])]
    try:
        idx = columns.index(key)
    except ValueError:
        return None
    if idx >= len(values):
        return None
    return values[idx]


def check_sequence(source: str, expected: str, label: str) -> tuple[bool, str]:
    payload = probe(source)
    if not bool(payload.get("ok")):
        return False, (
            "check=seamgrim_generic_imja_choose_send_raw_wasm "
            f"case={label} detail=unexpected_error:{str(payload.get('message', '')).strip()}"
        )
    if str(payload.get("schema", "")).strip() != "seamgrim.state.v0":
        return False, (
            "check=seamgrim_generic_imja_choose_send_raw_wasm "
            f"case={label} detail=bad_schema:{payload.get('schema')}"
        )
    if not str(payload.get("state_hash", "")).strip():
        return False, f"check=seamgrim_generic_imja_choose_send_raw_wasm case={label} detail=missing_hash"
    seq = value_at(payload, "순서")
    if seq is None:
        return False, (
            "check=seamgrim_generic_imja_choose_send_raw_wasm "
            f"case={label} detail=missing_columns:{','.join(payload.get('columns', []))}"
        )
    if seq != expected:
        return False, (
            "check=seamgrim_generic_imja_choose_send_raw_wasm "
            f"case={label} detail=bad_sequence:{seq}"
        )
    return True, seq


def main() -> int:
    cases = [
        ("first_branch", TRUE_SOURCE, "12434"),
        ("second_branch", SECOND_SOURCE, "12464"),
        ("else_branch", ELSE_SOURCE, "12454"),
        ("exhaustive_first_branch", EXHAUSTIVE_SOURCE, "12434"),
        ("exhaustive_second_branch", EXHAUSTIVE_SECOND_SOURCE, "12464"),
    ]
    observed: list[str] = []
    for label, source, expected in cases:
        ok, detail = check_sequence(source, expected, label)
        if not ok:
            print(detail)
            return 1
        observed.append(f"{label}={detail}")
    print("seamgrim generic imja choose send raw wasm check ok " + " ".join(observed))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
