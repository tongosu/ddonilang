#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "build" / "ddn_local_project_gaji_wave1_quality_gate.detjson"


def run(cmd: list[str], *, timeout: int = 600) -> dict:
    started = datetime.now(timezone.utc)
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
        "cmd": cmd,
        "returncode": proc.returncode,
        "started_at_utc": started.isoformat(),
        "stdout_tail": (proc.stdout or "")[-4000:],
    }


def write_report(payload: dict) -> None:
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    gates: list[tuple[list[str], int]] = [
        ([sys.executable, "tests/run_ddn_local_project_graph_check.py"], 600),
        ([sys.executable, "tests/run_ddn_local_project_cli_wasm_parity_check.py"], 300),
        ([sys.executable, "tests/run_ddn_local_project_workbench_multifile_check.py"], 360),
        ([sys.executable, "tests/run_seamgrim_multifile_portable_roundtrip_check.py"], 480),
        ([sys.executable, "tests/run_seamgrim_multifile_project_workbench_check.py"], 720),
        ([sys.executable, "tests/run_pack_golden.py", "ddn_local_project_graph_v1"], 600),
        ([sys.executable, "tests/run_pack_golden.py", "ddn_local_project_cli_wasm_parity_v1"], 300),
        ([sys.executable, "tests/run_pack_golden.py", "ddn_local_project_workbench_multifile_v1"], 300),
        ([sys.executable, "tests/run_pack_golden.py", "seamgrim_multifile_portable_roundtrip_v1"], 300),
        ([sys.executable, "tests/run_pack_golden.py", "seamgrim_multifile_project_workbench_v1"], 300),
        (["git", "status", "--short", "--", "docs/ssot"], 60),
    ]
    results = [run(cmd, timeout=timeout) for cmd, timeout in gates]
    docs_ssot = results[-1]
    docs_ssot_clean = docs_ssot["returncode"] == 0 and not str(docs_ssot["stdout_tail"]).strip()
    ok = all(row["returncode"] == 0 for row in results[:-1]) and docs_ssot_clean
    payload = {
        "schema": "ddn.local_project_gaji.wave1.quality_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "PASS" if ok else "FAIL",
        "orchestration_only": True,
        "product_logic_reimplemented": False,
        "docs_ssot_clean": docs_ssot_clean,
        "bounded_closure_claims": [
            "LOCAL_PROJECT_GRAPH_V1_CLOSED",
            "LOCAL_GAJI_LOCK_BRIDGE_V1_CLOSED",
            "MULTIFILE_CLI_WASM_PARITY_V1_CLOSED",
            "SEAMGRIM_MULTIFILE_AUTHORING_V1_CLOSED",
        ],
        "non_claims": [
            "full_module_system_complete",
            "public_package_ecosystem_complete",
            "all_gaji_supported",
            "all_ddn_projects_cli_wasm_equivalent",
            "registry_complete",
            "state_hash_formula_change",
        ],
        "results": results,
    }
    write_report(payload)
    if not ok:
        for row in results:
            if row["returncode"] != 0:
                print(f"E_DDN_LOCAL_PROJECT_GAJI_WAVE1_GATE: {' '.join(row['cmd'])}")
                print(row["stdout_tail"])
                break
        if not docs_ssot_clean:
            print(f"E_DDN_LOCAL_PROJECT_GAJI_WAVE1_SSOT_DIRTY: {docs_ssot['stdout_tail']}")
        return 1
    print(f"[ddn-local-project-gaji-wave1-quality-gate] PASS {REPORT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
