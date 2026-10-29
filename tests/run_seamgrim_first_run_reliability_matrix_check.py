#!/usr/bin/env python
from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import tempfile
import textwrap
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "out" / "seamgrim_runner_lifecycle_reliability_v1"
RUNNER = ROOT / "tests" / "seamgrim_workbench_run_archive_lineage_runner.mjs"
PACK = ROOT / "pack" / "seamgrim_runner_lifecycle_reliability_v1"


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


def parse_last_json(text: str) -> dict:
    start = text.rfind("\n{")
    candidate = text[start + 1 :].strip() if start >= 0 else text[text.find("{") :].strip()
    if not candidate:
        raise ValueError(f"missing json output: {text[-1200:]}")
    return json.loads(candidate)


def run_command(cmd: list[str], *, timeout: int, label: str, keep_stdout: bool = False) -> dict:
    start = time.time()
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
    elapsed_ms = int((time.time() - start) * 1000)
    row = {
        "label": label,
        "command": cmd,
        "returncode": proc.returncode,
        "elapsed_ms": elapsed_ms,
        "stdout_tail": proc.stdout[-3000:],
    }
    if keep_stdout:
        row["_stdout"] = proc.stdout
    return row


def require_command_pass(cmd: list[str], *, timeout: int, label: str, keep_stdout: bool = False) -> dict:
    row = run_command(cmd, timeout=timeout, label=label, keep_stdout=keep_stdout)
    if row["returncode"] != 0:
        raise RuntimeError(f"{label} failed: {' '.join(cmd)}\n{row['stdout_tail']}")
    return row


def check_lineage_payload(payload: dict, label: str) -> None:
    assertions = payload.get("assertions", {})
    if assertions.get("temp_data_dir_removed") is not True:
        raise RuntimeError(f"{label}: temp_data_dir_removed={assertions.get('temp_data_dir_removed')!r}")
    if assertions.get("no_projects_double_slash_request") is not True:
        raise RuntimeError(f"{label}: /projects// guard failed")
    if assertions.get("run_archive_project_binding_preserved") is not True:
        raise RuntimeError(f"{label}: run archive project binding not preserved")
    if assertions.get("view_capability_write_rejected") is not True:
        raise RuntimeError(f"{label}: view capability write rejection not recorded")
    teardown = payload.get("teardown", {})
    if teardown.get("residual_owned_pids"):
        raise RuntimeError(f"{label}: residual owned pids {teardown.get('residual_owned_pids')!r}")
    if teardown.get("residual_owned_processes"):
        raise RuntimeError(f"{label}: residual owned processes {teardown.get('residual_owned_processes')!r}")
    if teardown.get("residual_owned_sockets"):
        raise RuntimeError(f"{label}: residual owned sockets {teardown.get('residual_owned_sockets')!r}")
    ownership = payload.get("ownership_basis") or {}
    if ownership.get("strategy") != "pre_stop_process_identity_snapshot":
        raise RuntimeError(f"{label}: ownership strategy={ownership.get('strategy')!r}")
    if ownership.get("global_process_diff_used") is not False:
        raise RuntimeError(f"{label}: global process diff used")
    if not payload.get("owned_process_identities"):
        raise RuntimeError(f"{label}: missing owned process identities")
    if any("/projects//" in row.get("path", "") for row in payload.get("api_requests", [])):
        raise RuntimeError(f"{label}: api_requests contains /projects//")


def port_accepts_connection(port: int) -> bool:
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=0.25):
            return True
    except OSError:
        return False


def run_lineage_once(label: str, *, timeout: int = 300) -> dict:
    row = require_command_pass(["node", str(RUNNER.relative_to(ROOT))], timeout=timeout, label=label, keep_stdout=True)
    payload = parse_last_json(row.pop("_stdout"))
    check_lineage_payload(payload, label)
    actual_bound_port = int(payload.get("actual_bound_port") or 0)
    if actual_bound_port <= 0:
        raise RuntimeError(f"{label}: missing actual_bound_port={payload.get('actual_bound_port')!r}")
    residual_socket_count = 1 if port_accepts_connection(actual_bound_port) else 0
    if residual_socket_count:
        raise RuntimeError(f"{label}: residual owned socket still accepts connections on {actual_bound_port}")
    return {
        "label": label,
        "elapsed_ms": row["elapsed_ms"],
        "requested_port": payload.get("requested_port"),
        "actual_bound_port": actual_bound_port,
        "run_owner_id_present": bool(payload.get("run_owner_id")),
        "residual_owned_pids": len(payload.get("teardown", {}).get("residual_owned_pids") or []),
        "residual_owned_processes": len(payload.get("teardown", {}).get("residual_owned_processes") or []),
        "residual_owned_sockets": residual_socket_count,
        "double_slash_requests": sum(1 for item in payload.get("api_requests", []) if "/projects//" in item.get("path", "")),
    }


def run_sequence(name: str, count: int, command: list[str], *, timeout: int) -> list[dict]:
    rows: list[dict] = []
    for index in range(1, count + 1):
        rows.append(require_command_pass(command, timeout=timeout, label=f"{name}-{index:02d}"))
    return rows


def start_contention_workers(worker_count: int = 2) -> list[subprocess.Popen[str]]:
    code = textwrap.dedent(
        """
        import hashlib
        import os
        import pathlib
        import tempfile
        import time

        root = pathlib.Path(tempfile.gettempdir()) / "seamgrim_g4_contention"
        root.mkdir(parents=True, exist_ok=True)
        payload = b"seamgrim-g4-contention" * 4096
        end = time.time() + 120
        i = 0
        while time.time() < end:
            digest = hashlib.sha256(payload + str(i).encode("ascii")).hexdigest()
            target = root / f"{os.getpid()}_{i % 8}.tmp"
            target.write_text(digest, encoding="utf-8")
            i += 1
        """
    )
    flags = getattr(subprocess, "CREATE_NO_WINDOW", 0)
    workers: list[subprocess.Popen[str]] = []
    for _ in range(worker_count):
        workers.append(
            subprocess.Popen(
                [sys.executable, "-c", code],
                cwd=ROOT,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                text=True,
                creationflags=flags,
            )
        )
    return workers


def stop_contention_workers(workers: list[subprocess.Popen[str]]) -> dict:
    residual: list[int] = []
    escalated = False
    for worker in workers:
        if worker.poll() is not None:
            continue
        worker.terminate()
    deadline = time.time() + 5
    for worker in workers:
        while worker.poll() is None and time.time() < deadline:
            time.sleep(0.05)
        if worker.poll() is None:
            escalated = True
            worker.kill()
            worker.wait(timeout=5)
        if worker.poll() is None:
            residual.append(int(worker.pid or 0))
    return {"worker_count": len(workers), "escalated": escalated, "residual_owned_pids": residual}


def run_contention_once(label: str) -> dict:
    workers = start_contention_workers()
    try:
        row = run_lineage_once(label, timeout=420)
    finally:
        cleanup = stop_contention_workers(workers)
    if cleanup["residual_owned_pids"]:
        raise RuntimeError(f"{label}: contention residual pids {cleanup['residual_owned_pids']!r}")
    return {**row, "contention": cleanup}


def main() -> int:
    required = [
        RUNNER,
        PACK / "contract.detjson",
        PACK / "g4_matrix_contract.detjson",
        ROOT / "tests" / "run_seamgrim_project_run_prerequisite_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_G4_MATRIX_MISSING", ",".join(missing))

    contract = json.loads((PACK / "g4_matrix_contract.detjson").read_text(encoding="utf-8"))
    expected_counts = {
        "lineage_consecutive": 20,
        "public_wedge_consecutive": 5,
        "durable_workbench_consecutive": 5,
        "private_pilot_local_rehearsal_consecutive": 3,
        "four_viewport_workflow_consecutive": 2,
        "parallel_lineage_4x_consecutive": 3,
        "bounded_contention_consecutive": 3,
    }
    for key, expected in expected_counts.items():
        if contract.get("required_sequences", {}).get(key) != expected:
            return fail("E_SEAMGRIM_G4_MATRIX_CONTRACT", f"{key}={contract.get('required_sequences', {}).get(key)!r}")

    evidence = {
        "schema": "seamgrim.runner_lifecycle_reliability.g4_matrix_result.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "rerun_masked_failures": 0,
        "owned_residual_processes": 0,
        "owned_residual_sockets": 0,
        "owned_residual_temp_dirs": 0,
        "double_slash_requests": 0,
        "sequences": {},
    }

    try:
        evidence["sequences"]["lineage_consecutive"] = [run_lineage_once(f"lineage-{idx:02d}") for idx in range(1, 21)]
        evidence["sequences"]["public_wedge_consecutive"] = run_sequence(
            "public-wedge",
            5,
            [sys.executable, "tests/run_seamgrim_public_physics_wedge_quality_gate_check.py"],
            timeout=1200,
        )
        evidence["sequences"]["durable_workbench_consecutive"] = run_sequence(
            "durable-workbench",
            5,
            [sys.executable, "tests/run_seamgrim_workbench_durable_quality_gate_check.py"],
            timeout=1500,
        )
        evidence["sequences"]["private_pilot_local_rehearsal_consecutive"] = run_sequence(
            "private-pilot",
            3,
            [sys.executable, "tests/run_seamgrim_private_pilot_local_rehearsal_check.py"],
            timeout=1800,
        )
        evidence["sequences"]["four_viewport_workflow_consecutive"] = run_sequence(
            "four-viewport",
            2,
            [sys.executable, "tests/run_seamgrim_project_full_workflow_viewport_check.py"],
            timeout=900,
        )
        evidence["sequences"]["parallel_lineage_4x_consecutive"] = run_sequence(
            "parallel-lineage",
            3,
            [sys.executable, "tests/run_seamgrim_runner_lifecycle_process_check.py"],
            timeout=900,
        )
        evidence["sequences"]["bounded_contention_consecutive"] = [
            run_contention_once(f"bounded-contention-{idx:02d}") for idx in range(1, 4)
        ]

        ssot = require_command_pass(["git", "status", "--short", "--", "docs/ssot"], timeout=60, label="docs-ssot-status")
        if ssot["stdout_tail"].strip():
            raise RuntimeError(f"docs/ssot dirty: {ssot['stdout_tail']}")
        diff = require_command_pass([
            "git",
            "diff",
            "--check",
            "--",
            "tests/run_seamgrim_first_run_reliability_matrix_check.py",
            "tests/run_seamgrim_workbench_run_archive_lineage_runner.mjs",
            "tests/run_seamgrim_project_run_prerequisite_check.py",
            "solutions/seamgrim_ui_mvp/ui/workbench/persistence/run_archive_adapter.js",
            "pack/seamgrim_runner_lifecycle_reliability_v1",
        ], timeout=120, label="diff-check")
        if diff["stdout_tail"].strip():
            raise RuntimeError(f"diff --check output: {diff['stdout_tail']}")
    except Exception as exc:
        evidence["status"] = "FAIL"
        evidence["failure"] = str(exc)
        OUT.mkdir(parents=True, exist_ok=True)
        (OUT / "g4_matrix_latest.detjson").write_text(json.dumps(evidence, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")
        return fail("E_SEAMGRIM_G4_MATRIX_SEQUENCE", str(exc))

    evidence["status"] = "PASS"
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "g4_matrix_latest.detjson").write_text(json.dumps(evidence, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    print("[seamgrim-first-run-reliability-matrix] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
