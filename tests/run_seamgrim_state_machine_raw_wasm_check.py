#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def probe() -> dict:
    script = r"""
const fs = require("fs");
const path = require("path");
const { pathToFileURL } = require("url");

async function main() {
  const root = process.cwd();
  const uiDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModuleUrl = pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href;
  const wrapperUrl = pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href;
  const wasmPath = path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm");
  const wasmModule = await import(wasmModuleUrl);
  const wrapper = await import(wrapperUrl);
  const wasmBytes = fs.readFileSync(wasmPath);
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }

  const source = `
채비 {
  현재:글 <- "".
  다음:글 <- "".
  확인:글 <- "".
}.

매틱:움직씨 = {
  기계 <- 상태머신{
    빨강, 초록, 노랑 으로 이뤄짐.
    빨강 으로 시작.
    빨강 에서 초록 으로.
    초록 에서 노랑 으로.
    노랑 에서 빨강 으로.
  }.
  현재 <- (기계) 처음으로.
  다음 <- (기계, 현재) 다음으로.
  확인 <- (기계, 다음) 지금상태.
}.
`;

  try {
    const vm = new wasmModule.DdnWasmVm(source);
      const client = new wrapper.DdnWasmVmClient(vm);
      const stepped = client.stepOneParsed();
      console.log(JSON.stringify({
        ok: true,
        state_schema: String(stepped?.schema ?? ""),
        state_hash: String(stepped?.state_hash ?? ""),
        resources: stepped?.resources?.value_json ?? {},
      }));
  } catch (err) {
    console.log(JSON.stringify({
      ok: false,
      message: String((err && err.message) || err),
    }));
  }
}

main().catch((err) => {
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
    return json.loads(proc.stdout)


def main() -> int:
    payload = probe()
    if not bool(payload.get("ok")):
        print(
            "check=seamgrim_state_machine_raw_wasm "
            f"detail=unexpected_error:{str(payload.get('message', '')).strip()}"
        )
        return 1
    if str(payload.get("state_schema", "")).strip() != "seamgrim.state.v0":
        print(
            "check=seamgrim_state_machine_raw_wasm "
            f"detail=bad_schema:{payload.get('state_schema')}"
        )
        return 1
    if not str(payload.get("state_hash", "")).strip():
        print("check=seamgrim_state_machine_raw_wasm detail=missing_hash")
        return 1
    resources = payload.get("resources", {})
    machine = resources.get("기계")
    if not isinstance(machine, dict):
        print("check=seamgrim_state_machine_raw_wasm detail=missing_machine_resource")
        return 1
    if str(machine.get("__ddn_kind", "")).strip() != "ddn.state_machine.v1":
        print(
            "check=seamgrim_state_machine_raw_wasm "
            f"detail=bad_kind:{machine.get('__ddn_kind')}"
        )
        return 1
    if str(machine.get("initial", "")).strip() != "빨강":
        print(
            "check=seamgrim_state_machine_raw_wasm "
            f"detail=bad_initial:{machine.get('initial')}"
        )
        return 1
    transitions = machine.get("transitions")
    if not isinstance(transitions, list) or len(transitions) != 3:
        print("check=seamgrim_state_machine_raw_wasm detail=bad_transition_count")
        return 1

    print("seamgrim state machine raw wasm check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
