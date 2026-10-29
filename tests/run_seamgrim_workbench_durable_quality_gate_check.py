#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_durable_quality_gate_v1"

GATES = [
    "tests/run_seamgrim_workbench_durable_persistence_adapters_check.py",
    "tests/run_seamgrim_workbench_server_project_save_reopen_check.py",
    "tests/run_seamgrim_workbench_revision_conflict_restore_check.py",
    "tests/run_seamgrim_workbench_run_archive_store_check.py",
    "tests/run_seamgrim_workbench_run_archive_lineage_check.py",
    "tests/run_seamgrim_workbench_capability_share_clone_check.py",
    "tests/run_seamgrim_workbench_my_work_reopen_check.py",
    "tests/run_seamgrim_workbench_storage_security_recovery_check.py",
    "tests/run_seamgrim_workbench_durable_full_workflow_viewport_check.py",
]

OWNED_TEMP_PREFIXES = (
    "seamgrim-workbench-g2-",
    "seamgrim-workbench-g3-",
    "seamgrim-workbench-g5-",
    "seamgrim-workbench-g6-",
    "seamgrim-workbench-g7-",
    "seamgrim-workbench-g9-",
    "seamgrim-run-archive-",
    "seamgrim-g8-",
    "seamgrim-g7-store-",
    "seamgrim-restart-links-",
)


def owned_temp_dirs() -> set[str]:
    root = Path(tempfile.gettempdir())
    return {
        path.name
        for path in root.iterdir()
        if path.is_dir() and path.name.startswith(OWNED_TEMP_PREFIXES)
    }


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


def run(cmd: list[str], *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
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


def main() -> int:
    required = [PACK / "README.md", PACK / "contract.detjson", PACK / "golden.jsonl", PACK / "input.ddn"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_DURABLE_QG_MISSING", json.dumps(missing, ensure_ascii=False))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for key, expected in {
        "pack": "seamgrim_workbench_durable_quality_gate_v1",
        "gate": "G10",
        "behavior_closed_claim": True,
        "actual_idc_deploy_claim": False,
        "legacy_shell_switch_claim": False,
        "login_or_public_registry_claim": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
        "explicit_readiness_gate": True,
        "owned_temp_residual_gate": True,
    }.items():
        if contract.get(key) != expected:
            return fail("E_SEAMGRIM_DURABLE_QG_CONTRACT", f"{key}={contract.get(key)!r}")
    temp_baseline = owned_temp_dirs()
    for gate in GATES:
        proc = run([sys.executable, gate], timeout=700)
        if proc.returncode:
            return fail("E_SEAMGRIM_DURABLE_QG_GATE", f"{gate}: {proc.stdout.strip()}")
        residual = sorted(owned_temp_dirs() - temp_baseline)
        if residual:
            return fail("E_SEAMGRIM_DURABLE_QG_TEMP_RESIDUAL", f"{gate}: {residual!r}")
    pack = run([sys.executable, "tests/run_pack_golden.py", "seamgrim_workbench_durable_quality_gate_v1"], timeout=120)
    if pack.returncode:
        return fail("E_SEAMGRIM_DURABLE_QG_PACK", pack.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_DURABLE_QG_SSOT", ssot.stdout.strip())
    print("[seamgrim-workbench-durable-quality-gate-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
