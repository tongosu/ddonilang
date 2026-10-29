#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_lab_e1_minimal_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def check_files_and_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_lab_e1_minimal_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_LAB_E1_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("lineage_schema") != "seamgrim.lab_e1_lineage.v1":
        return fail("E_SEAMGRIM_LAB_E1_SCHEMA", repr(contract))
    for key in (
        "auto_apply_claim",
        "file_write_claim",
        "ai_authority_claim",
        "parser_change",
        "runtime_semantics_change",
        "state_hash_formula_change",
        "docs_ssot_change",
    ):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_LAB_E1_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_product_sources() -> int:
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    css = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css")
    for token in [
        "seamgrim.lab_e1_lineage.v1",
        "origin_hash",
        "delta_ref",
        "syncLabE1LineagePanel",
        "auto_apply_claim: false",
        "file_write_claim: false",
    ]:
        if token not in run_js:
            return fail("E_SEAMGRIM_LAB_E1_RUN_JS", token)
    for token in [".run-lab-e1-lineage", '.run-lab-e1-lineage[data-state="changed"]']:
        if token not in css:
            return fail("E_SEAMGRIM_LAB_E1_CSS", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LAB_E1_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_LAB_E1_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_lab_e1_minimal_runner.mjs"], "E_SEAMGRIM_LAB_E1_NODE_CHECK", 60),
        (["node", "tests/seamgrim_lab_e1_minimal_runner.mjs"], "E_SEAMGRIM_LAB_E1_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_lab_e1_minimal_v1"], "E_SEAMGRIM_LAB_E1_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-lab-e1-minimal-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
