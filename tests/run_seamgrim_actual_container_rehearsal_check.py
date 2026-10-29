#!/usr/bin/env python
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "out" / "seamgrim_actual_container_rehearsal_v1"
PACK = ROOT / "pack" / "seamgrim_actual_container_rehearsal_v1"


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


def run(cmd: list[str], *, timeout: int) -> dict:
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    return {
        "command": cmd,
        "returncode": proc.returncode,
        "stdout_tail": proc.stdout[-3000:],
    }


def require_pass(row: dict) -> None:
    if row["returncode"] != 0:
        raise RuntimeError(f"{' '.join(row['command'])} failed\n{row['stdout_tail']}")


def main() -> int:
    required = [PACK / "contract.detjson"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_ACTUAL_CONTAINER_REHEARSAL_MISSING", ",".join(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("current_gate") != "G11":
        return fail("E_SEAMGRIM_ACTUAL_CONTAINER_REHEARSAL_CONTRACT", f"current_gate={contract.get('current_gate')!r}")
    if contract.get("actual_container_verified") is not False:
        return fail("E_SEAMGRIM_ACTUAL_CONTAINER_REHEARSAL_CLAIM", "actual container claim must remain false without runtime")

    commands = [
        ([sys.executable, "tests/run_seamgrim_actual_container_readiness_check.py"], 1200),
        ([sys.executable, "tests/run_seamgrim_storage_error_hardening_check.py"], 600),
        ([sys.executable, "tests/run_seamgrim_local_backup_restore_rollback_rehearsal_check.py"], 1200),
        ([sys.executable, "tests/run_seamgrim_public_physics_wedge_quality_gate_check.py"], 1200),
        ([sys.executable, "tests/run_seamgrim_workbench_durable_quality_gate_check.py"], 1500),
        ([sys.executable, "tests/run_ddn_cli_wasm_freshness_check.py"], 600),
        ([sys.executable, "tests/run_ddn_cli_wasm_true_parity_equality_check.py"], 900),
        ([sys.executable, "tests/run_ddn_cli_wasm_sync_gate_check.py"], 900),
        ([sys.executable, "tests/run_seamgrim_view_only_state_hash_invariant_check.py"], 900),
        ([sys.executable, "tests/run_pack_golden.py", "seamgrim_actual_container_rehearsal_v1"], 120),
    ]
    rows: list[dict] = []
    try:
        for cmd, timeout in commands:
            row = run(cmd, timeout=timeout)
            require_pass(row)
            rows.append(row)
    except Exception as exc:
        return fail("E_SEAMGRIM_ACTUAL_CONTAINER_REHEARSAL_CMD", str(exc))

    runtime = {
        "docker": bool(shutil.which("docker")),
        "podman": bool(shutil.which("podman")),
        "docker_compose": bool(shutil.which("docker-compose")),
        "nginx": bool(shutil.which("nginx")),
        "image_pull_approved": os.environ.get("CONTAINER_IMAGE_PULL_APPROVED") == "1",
        "runtime_install_approved": os.environ.get("CONTAINER_RUNTIME_INSTALL_APPROVED") == "1",
    }
    actual_runtime_open = (runtime["docker"] or runtime["podman"]) and runtime["image_pull_approved"]
    summary = {
        "schema": "seamgrim.actual_container_rehearsal.g11_result.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "PASS_READY_FOR_CONTAINER_RUNTIME",
        "actual_container_verified": False,
        "actual_oci_build_executed": False,
        "actual_oci_run_executed": False,
        "actual_edge_tls_executed": False,
        "actual_runtime_open": actual_runtime_open,
        "runtime_availability": runtime,
        "local_product_behavior_checks_passed": True,
        "commands": rows,
        "highest_grade": "READY_FOR_CONTAINER_RUNTIME",
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "g11_rehearsal_latest.detjson").write_text(json.dumps(summary, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot["returncode"] or ssot["stdout_tail"].strip():
        return fail("E_SEAMGRIM_ACTUAL_CONTAINER_REHEARSAL_SSOT", ssot["stdout_tail"])

    print("[seamgrim-actual-container-rehearsal] PASS READY_FOR_CONTAINER_RUNTIME")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
