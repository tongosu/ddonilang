#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

PACK_TARGETS = {
    "pack/vol4_event_dispatch_runtime_v1/input.ddn": ["모드", "마지막", "처리횟수"],
    "pack/vol4_state_transition_runtime_v1/input.ddn": ["현재상태", "체력"],
    "pack/vol4_resume_isolation_runtime_v1/input.ddn": ["모드", "격리됨", "처리건수", "보류건수"],
    "pack/vol4_multi_signal_priority_runtime_v1/input.ddn": ["격리됨", "마지막", "일반처리", "차단수"],
}


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
  const commonUrl = pathToFileURL(path.join(uiDir, "wasm_page_common.js")).href;
  const wasmPath = path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm");
  const wasmModule = await import(wasmModuleUrl);
  const wrapper = await import(wrapperUrl);
  const common = await import(commonUrl);
  const wasmBytes = fs.readFileSync(wasmPath);
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }

  const rels = [
    "pack/vol4_event_dispatch_runtime_v1/input.ddn",
    "pack/vol4_state_transition_runtime_v1/input.ddn",
    "pack/vol4_resume_isolation_runtime_v1/input.ddn",
    "pack/vol4_multi_signal_priority_runtime_v1/input.ddn",
  ];
  const payload = {};
  for (const rel of rels) {
    const source = common.stripMetaHeader(fs.readFileSync(path.join(root, rel), "utf8"));
    try {
      const vm = new wasmModule.DdnWasmVm(source);
      const client = new wrapper.DdnWasmVmClient(vm);
      const stepped = client.stepOneParsed();
      const after = client.columnsParsed();
      payload[rel] = {
        ok: true,
        state_schema: String(stepped?.schema ?? ""),
        state_hash: String(stepped?.state_hash ?? ""),
        after_columns: Array.isArray(after?.columns) ? after.columns.map((row) => String(row?.key ?? "")) : [],
      };
    } catch (err) {
      payload[rel] = {
        ok: false,
        message: String((err && err.message) || err),
      };
    }
  }
  console.log(JSON.stringify(payload));
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
    rows = probe()
    for rel_path, expected_columns in PACK_TARGETS.items():
        row = rows.get(rel_path)
        if not isinstance(row, dict):
            print(f"check=seamgrim_vol4_raw_wasm_runtime_pack detail=missing_row:{rel_path}")
            return 1
        if not bool(row.get("ok")):
            print(
                "check=seamgrim_vol4_raw_wasm_runtime_pack "
                f"detail=unexpected_error:{rel_path}:{str(row.get('message', '')).strip()}"
            )
            return 1
        if str(row.get("state_schema", "")).strip() != "seamgrim.state.v0":
            print(
                "check=seamgrim_vol4_raw_wasm_runtime_pack "
                f"detail=bad_schema:{rel_path}:{row.get('state_schema')}"
            )
            return 1
        if not str(row.get("state_hash", "")).strip():
            print(
                "check=seamgrim_vol4_raw_wasm_runtime_pack "
                f"detail=missing_hash:{rel_path}"
            )
            return 1
        columns = [str(item) for item in row.get("after_columns", [])]
        for column in expected_columns:
            if column not in columns:
                print(
                    "check=seamgrim_vol4_raw_wasm_runtime_pack "
                    f"detail=missing_column:{rel_path}:{column}:{','.join(columns)}"
                )
                return 1

    print(
        "seamgrim vol4 raw wasm runtime pack check ok "
        f"packs={len(PACK_TARGETS)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
