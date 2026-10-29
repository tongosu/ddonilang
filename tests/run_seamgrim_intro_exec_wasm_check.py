#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_intro_exec_wasm_v1"


EXPECTED_CASES = {
    "c01_hello_show",
    "c02_assign_update",
    "c03_if_else",
    "c04_choose_exhaustive",
    "c05_intro_combined",
}


def fail(message: str) -> int:
    print(f"[seamgrim-intro-exec-wasm] fail: {message}", file=sys.stderr)
    return 1


def main() -> int:
    try:
        contract_path = PACK / "contract.detjson"
        contract = json.loads(contract_path.read_text(encoding="utf-8"))
        if contract.get("schema") != "ddn.seamgrim.wasm_cli_runtime_parity.pack.contract.v1":
            raise RuntimeError("contract schema mismatch")
        case_ids = {str(item.get("id", "")) for item in contract.get("cases", [])}
        if case_ids != EXPECTED_CASES:
            raise RuntimeError(f"case id mismatch: {sorted(case_ids)}")
        for item in contract.get("cases", []):
            source = PACK / str(item.get("input", ""))
            if not source.exists():
                raise RuntimeError(f"input missing: {source.relative_to(ROOT)}")

        proc = subprocess.run(
            [
                "node",
                "--no-warnings",
                "tests/seamgrim_wasm_cli_runtime_parity_runner.mjs",
                str(PACK.relative_to(ROOT)),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=240,
        )
        if proc.returncode != 0:
            detail = (proc.stderr or proc.stdout or "").strip()
            raise RuntimeError(detail)
        report = json.loads(proc.stdout)
        if report.get("schema") != "ddn.seamgrim.wasm_cli_runtime_parity.report.v1":
            raise RuntimeError("runner report schema mismatch")
        if report.get("pack_id") != "seamgrim_intro_exec_wasm_v1":
            raise RuntimeError("runner pack id mismatch")
        if not report.get("ok"):
            raise RuntimeError("runner report ok=false")
        rows = report.get("cases", [])
        if len(rows) != len(EXPECTED_CASES):
            raise RuntimeError(f"runner case count mismatch: {len(rows)}")
    except Exception as exc:
        return fail(str(exc))

    print(f"[seamgrim-intro-exec-wasm] ok cases={len(EXPECTED_CASES)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

