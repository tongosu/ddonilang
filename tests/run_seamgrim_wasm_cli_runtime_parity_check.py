#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wasm_cli_runtime_parity_v1"
RUNNER = ROOT / "tests" / "seamgrim_wasm_cli_runtime_parity_runner.mjs"


def fail(message: str) -> int:
    print(f"[seamgrim-wasm-cli-runtime-parity] fail: {message}")
    return 1


def main() -> int:
    contract_path = PACK / "contract.detjson"
    if not contract_path.exists():
        return fail(f"missing contract: {contract_path}")
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")

    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    case_ids = [str(case.get("id", "")).strip() for case in contract.get("cases", [])]
    expected = {
        "c01_console_grid_scalar",
        "c02_moyang_space2d",
        "c03_symbolic_simplify",
        "c04_proof_tactic_equivalence",
        "c05_lambda_store",
        "c06_chaebi_derived_show",
        "c07_hook_tick_show",
        "c08_parse_warning_reassign",
        "c09_boim_structured_view",
        "c10_setting_madi_count",
        "c11_show_and_boim_split",
        "c12_setting_madi_bad_value",
        "c13_bogae_draw_madi_view_sugar",
    }
    if set(case_ids) != expected:
        return fail(f"case set mismatch: {case_ids}")
    for case in contract.get("cases", []):
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
    for row in report.get("cases", []):
        if not row.get("ok"):
            return fail(f"{row.get('id')}: {row.get('failures')}")
        if not isinstance(row.get("cli_canonical_ddn", ""), str):
            return fail(f"{row.get('id')}: missing cli canonical ddn field")
        if not isinstance(row.get("wasm_canonical_ddn", ""), str):
            return fail(f"{row.get('id')}: missing wasm canonical ddn field")

    print("seamgrim wasm/cli runtime parity ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
