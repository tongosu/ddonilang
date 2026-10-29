#!/usr/bin/env python3
"""Gate for Wave3B G8 teacher-side student submission archive workflow."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave3b_teacher_submission_archive_v1"


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
        "teacher_archive_required": True,
        "three_submission_collection_required": True,
        "same_lesson_origin_required": True,
        "duplicate_detection_required": True,
        "reopen_submission_required": True,
        "archive_export_required": True,
        "control_diff_required": True,
        "run_evidence_required": True,
        "observation_refs_required": True,
        "no_runtime_role_toggle": True,
        "docs_ssot_change": False,
        "parser_change": False,
        "state_hash_formula_change": False,
        "auto_score_claim": False,
        "new_exchange_schema_claim": False,
    }
    for key, expected in required.items():
        actual = contract.get(key)
        if actual != expected:
            raise SystemExit(f"E_WAVE3B_G8_CONTRACT_{key}: expected {expected!r}, got {actual!r}")
    if contract.get("accepted_kinds") != ["lesson_bundle", "student_submission"]:
        raise SystemExit("E_WAVE3B_G8_ACCEPTED_KINDS")

    scripts = [
        "solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js",
        "solutions/seamgrim_ui_mvp/ui/screens/browse.js",
        "solutions/seamgrim_ui_mvp/ui/screens/run.js",
        "tests/seamgrim_wave3b_teacher_submission_archive_runner.mjs",
    ]
    for script in scripts:
        run(["node", "--check", script], timeout=30)

    run(["python", "tests/run_pack_golden.py", "seamgrim_wave3b_teacher_submission_archive_v1"], timeout=120)
    output = run(["node", "tests/seamgrim_wave3b_teacher_submission_archive_runner.mjs"], timeout=480)
    if "seamgrim_wave3b_teacher_submission_archive: ok" not in output:
        raise SystemExit(f"E_WAVE3B_G8_RUNNER_OUTPUT: {output}")
    print("[seamgrim-wave3b-teacher-submission-archive] OK behavior-closed")


if __name__ == "__main__":
    main()
