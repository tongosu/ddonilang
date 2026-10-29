#!/usr/bin/env python
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "out" / "seamgrim_private_pilot_local_rehearsal"
PACK = ROOT / "pack" / "seamgrim_private_pilot_operations_rehearsal_v1"
IDC_INPUT = ROOT / "work" / "codex" / "20260711_idc_deployment_intake" / "IDC_INPUT_SHEET.md"


def fail(message: str) -> None:
    raise SystemExit(f"[seamgrim-private-pilot-local-rehearsal] FAIL: {message}")


def run(args: list[str], *, timeout: int) -> dict:
    start = time.time()
    proc = subprocess.run(
        args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
    )
    elapsed_ms = int((time.time() - start) * 1000)
    return {
        "command": args,
        "returncode": proc.returncode,
        "elapsed_ms": elapsed_ms,
        "stdout_tail": proc.stdout[-4000:],
        "stderr_tail": proc.stderr[-4000:],
    }


def require_pass(row: dict, marker: str = "") -> None:
    if row["returncode"] != 0:
        fail(f"{' '.join(row['command'])} failed\nSTDOUT:\n{row['stdout_tail']}\nSTDERR:\n{row['stderr_tail']}")
    if marker and marker not in f"{row['stdout_tail']}\n{row['stderr_tail']}":
        fail(f"{' '.join(row['command'])} missing marker {marker!r}")


def idc_input_complete() -> bool:
    if not IDC_INPUT.is_file():
        return False
    text = IDC_INPUT.read_text(encoding="utf-8")
    missing_markers = ["TBD", "TODO", "비워", "미정", "REPLACE"]
    required_names = ["SSH", "host", "domain", "backup", "TLS"]
    if any(marker.lower() in text.lower() for marker in missing_markers):
        return False
    return all(name.lower() in text.lower() for name in required_names)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    commands = [
        (["python", "tests/run_seamgrim_private_pilot_config_check.py"], "[seamgrim-private-pilot-config] PASS", 180),
        (["python", "tests/run_seamgrim_private_pilot_release_check.py"], "[seamgrim-private-pilot-release] PASS", 900),
        (["python", "tests/run_seamgrim_private_pilot_service_hardening_check.py"], "[seamgrim-private-pilot-service] PASS", 900),
        (["python", "tests/run_seamgrim_private_pilot_edge_policy_check.py"], "[seamgrim-private-pilot-edge] PASS", 900),
        (["python", "tests/run_seamgrim_private_pilot_observability_check.py"], "[seamgrim-private-pilot-observability] PASS", 900),
        (["python", "tests/run_seamgrim_private_pilot_backup_restore_check.py"], "[seamgrim-private-pilot-backup] PASS", 900),
        (["python", "tests/run_seamgrim_private_pilot_deploy_rollback_check.py"], "[seamgrim-private-pilot-deploy] PASS", 900),
        (["python", "tests/run_pack_golden.py", "seamgrim_private_pilot_release_contract_v1", "seamgrim_private_pilot_operations_rehearsal_v1"], "pack golden ok", 900),
    ]
    rows: list[dict] = []
    for command, marker, timeout in commands:
        row = run(command, timeout=timeout)
        require_pass(row, marker)
        rows.append(row)

    docker_available = shutil.which("docker") is not None
    compose_available = shutil.which("docker-compose") is not None
    nginx_available = shutil.which("nginx") is not None
    remote_ready = idc_input_complete() and "PRIVATE_IDC_DEPLOY_APPROVED" in (IDC_INPUT.read_text(encoding="utf-8") if IDC_INPUT.is_file() else "")
    summary = {
        "schema": "seamgrim.private_pilot.local_rehearsal_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "PASS_LOCAL_PROCESS_REHEARSAL",
        "release_candidate_ready_claim": False,
        "private_idc_deployed_claim": False,
        "external_participant_success_count": 0,
        "local_process_browser_rehearsal_pass": True,
        "container_runtime": {
            "docker_available": docker_available,
            "compose_available": compose_available,
            "container_claim_allowed": docker_available and compose_available,
        },
        "edge_runtime": {
            "nginx_available": nginx_available,
            "real_tls_edge_claim_allowed": nginx_available,
        },
        "remote_gate": {
            "idc_input_complete": idc_input_complete(),
            "private_idc_deploy_approved": False,
            "ssh_inventory_executed": False,
            "status": "blocked_until_private_idc_input_and_approval",
            "g9_remote_inventory_executed": False,
        },
        "commands": rows,
    }
    (OUT / "summary.detjson").write_text(json.dumps(summary, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    require_pass(ssot)
    print("[seamgrim-private-pilot-local-rehearsal] PASS")


if __name__ == "__main__":
    main()
