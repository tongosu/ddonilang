#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "ganada_ma_assessment_workflow_v1"
WORKFLOW = PACK / "workflow.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "education_assessment_pack.js"
RUNNER = ROOT / "tests" / "ganada_ma_assessment_workflow_runner.mjs"


def fail(message: str) -> None:
    print(f"ganada_ma_assessment_workflow_check: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(cmd: list[str], *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    normalized = [sys.executable if item == "python" else item for item in cmd]
    return subprocess.run(
        normalized,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8-sig"))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def check_shape(workflow: dict) -> None:
    expected = {
        "schema": "ddn.ganada.ma.assessment_workflow.v1",
        "pack": "ganada_ma_assessment_workflow_v1",
        "cell_id": "마-2",
        "status": "product_pass_gate",
        "behavior_closed_claim": False,
        "matrix_upgrade_claim": False,
        "docs_ssot_change": False,
        "parser_change": False,
        "runtime_change": False,
        "state_hash_change": False,
    }
    for key, value in expected.items():
        if workflow.get(key) != value:
            fail(f"{key}={workflow.get(key)!r}, expected {value!r}")
    details = workflow.get("workflow")
    if not isinstance(details, dict):
        fail("workflow must be object")
    if details.get("lesson_run_result_source") != "seamgrim_lesson_run_preset_rail_v1":
        fail("lesson_run_result_source mismatch")
    selectors = details.get("required_selectors")
    if selectors != ["assessment-panel", "assessment-run", "assessment-result", "assessment-feedback"]:
        fail(f"required selectors mismatch: {selectors!r}")
    boundaries = set(details.get("required_boundaries", []))
    for boundary in ["gradebook_write:false", "student_personal_data_collection:false", "remote_classroom_sync:false"]:
        if boundary not in boundaries:
            fail(f"missing boundary: {boundary}")


def check_static_ui_tokens() -> None:
    text = UI_MODULE.read_text(encoding="utf-8-sig")
    for token in [
        'data-testid="assessment-feedback"',
        'data-testid="assessment-result"',
        'data-testid="assessment-run"',
        "lesson_run_result_source:seamgrim_lesson_run_preset_rail_v1",
        "assessmentWorkflowRan",
    ]:
        if token not in text:
            fail(f"UI module missing token: {token}")


def check_commands() -> None:
    commands = [
        (["python", "tests/run_pack_golden.py", "ganada_ma_assessment_workflow_v1"], "pack golden ok"),
        (["node", "tests/education_assessment_pack_runner.mjs"], "education_assessment_pack: ok"),
        (["node", "tests/ganada_ma_assessment_workflow_runner.mjs"], "ganada_ma_assessment_workflow: ok"),
    ]
    for cmd, marker in commands:
        proc = run(cmd, timeout=900)
        if proc.returncode != 0:
            fail(f"command failed {cmd}:\n{proc.stdout}")
        if marker not in proc.stdout:
            fail(f"marker {marker!r} missing for {cmd}:\n{proc.stdout}")


def check_boundaries() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    if not RUNNER.is_file():
        fail(f"missing runner: {RUNNER.relative_to(ROOT)}")
    workflow = read_json(WORKFLOW)
    check_shape(workflow)
    check_static_ui_tokens()
    check_commands()
    check_boundaries()
    print("ganada_ma_assessment_workflow_check: PASS")


if __name__ == "__main__":
    main()
