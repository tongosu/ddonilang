#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "relation_solve_wasm_cli_parity_v1"
RUNNER = ROOT / "tests" / "seamgrim_wasm_cli_runtime_parity_runner.mjs"


def fail(message: str) -> int:
    print(f"[relation-solve-wasm-cli-parity] fail: {message}")
    return 1


def main() -> int:
    contract_path = PACK / "contract.detjson"
    if not contract_path.exists():
        return fail(f"missing contract: {contract_path}")
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")

    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.relation_solve_wasm_cli_parity.pack.contract.v1":
        return fail(f"bad schema: {contract.get('schema')}")
    expected = {"c01_linear_success", "c02_non_unique", "c03_unsupported"}
    cases = contract.get("cases", [])
    if {str(case.get('id', '')).strip() for case in cases} != expected:
        return fail(f"case set mismatch: {[case.get('id') for case in cases]}")
    for case in cases:
        rel = str(case.get("input", "")).strip()
        if not rel:
            return fail(f"{case.get('id')}: input missing")
        if not (PACK / rel).exists():
            return fail(f"{case.get('id')}: input not found: {rel}")

    proc = subprocess.run(
        ["node", "--no-warnings", str(RUNNER), str(PACK)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
        return fail(detail)
    try:
        report = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        return fail(f"runner emitted invalid json: {exc}")
    if report.get("schema") != "ddn.seamgrim.wasm_cli_runtime_parity.report.v1":
        return fail(f"bad report schema: {report.get('schema')}")
    if report.get("ok") is not True:
        return fail("runner report ok=false")
    if len(report.get("cases", [])) != len(expected):
        return fail("runner report case count mismatch")
    print("relation solve wasm/cli runtime parity ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
