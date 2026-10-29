#!/usr/bin/env python3
"""Gate for Wave3B G7 durable student submission exchange."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave3b_student_submission_v1"


def run(cmd: list[str], timeout: int | None = None) -> str:
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        timeout=timeout,
    )
    if proc.returncode != 0:
        detail = ((proc.stdout or "") + (proc.stderr or "")).strip()
        raise SystemExit(f"E_WAVE3B_G7_COMMAND: {' '.join(cmd)}\n{detail}")
    return ((proc.stdout or "") + (proc.stderr or "")).strip()


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    required = {
        "product_behavior_claim": True,
        "classroom_exchange_schema": "seamgrim.classroom_exchange.v0",
        "kind": "student_submission",
        "export_import_reopen_required": True,
        "origin_hash_required": True,
        "run_evidence_required": True,
        "stale_binding_hash_changes": True,
        "tamper_detection": True,
        "student_text_xss_guard": True,
        "no_runtime_role_toggle": True,
        "docs_ssot_change": False,
        "parser_change": False,
        "state_hash_formula_change": False,
    }
    for key, expected in required.items():
        actual = contract.get(key)
        if actual != expected:
            raise SystemExit(f"E_WAVE3B_G7_CONTRACT_{key}: expected {expected!r}, got {actual!r}")

    for script in [
        "solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js",
        "solutions/seamgrim_ui_mvp/ui/screens/run.js",
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "tests/seamgrim_wave3b_student_submission_runner.mjs",
    ]:
        run(["node", "--check", script], timeout=30)
    run(["python", "tests/run_pack_golden.py", "seamgrim_wave3b_student_submission_v1"], timeout=120)
    output = run(["node", "tests/seamgrim_wave3b_student_submission_runner.mjs"], timeout=360)
    if "seamgrim_wave3b_student_submission: ok" not in output:
        raise SystemExit(f"E_WAVE3B_G7_RUNNER_OUTPUT: {output}")
    print("[seamgrim-wave3b-student-submission] OK behavior-closed")


if __name__ == "__main__":
    main()
