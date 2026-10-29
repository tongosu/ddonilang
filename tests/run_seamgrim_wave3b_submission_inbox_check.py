#!/usr/bin/env python3
"""Gate for Wave3B G8 local teacher student-submission inbox."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave3b_submission_inbox_v1"


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
        raise SystemExit(f"E_WAVE3B_G8_COMMAND: {' '.join(cmd)}\n{detail}")
    return ((proc.stdout or "") + (proc.stderr or "")).strip()


def main() -> None:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    required = {
        "product_behavior_claim": True,
        "classroom_exchange_schema": "seamgrim.classroom_exchange.v0",
        "input_kind": "student_submission",
        "batch_review_schema": "seamgrim.student_submission_inbox_batch.v0",
        "reopen_button_required": True,
        "autograding_claim": False,
        "account_required": False,
        "cloud_sync": False,
        "public_upload_claim": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "state_hash_formula_change": False,
    }
    for key, expected in required.items():
        actual = contract.get(key)
        if actual != expected:
            raise SystemExit(f"E_WAVE3B_G8_CONTRACT_{key}: expected {expected!r}, got {actual!r}")
    if len(contract.get("required_row_fields", [])) < 10:
        raise SystemExit("E_WAVE3B_G8_REQUIRED_ROW_FIELDS")

    for script in [
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "solutions/seamgrim_ui_mvp/ui/screens/run.js",
        "tests/seamgrim_wave3b_submission_inbox_runner.mjs",
    ]:
        run(["node", "--check", script], timeout=30)

    run(["python", "tests/run_pack_golden.py", "seamgrim_wave3b_submission_inbox_v1"], timeout=120)
    output = run(["node", "tests/seamgrim_wave3b_submission_inbox_runner.mjs"], timeout=700)
    if "seamgrim_wave3b_submission_inbox: ok" not in output:
        raise SystemExit(f"E_WAVE3B_G8_RUNNER_OUTPUT: {output}")
    print("[seamgrim-wave3b-submission-inbox] OK behavior-closed")


if __name__ == "__main__":
    main()
