#!/usr/bin/env python3
"""Validate HA3_CLASSROOM_UI_PACK_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "HA3_CLASSROOM_UI_PACK_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "하-3_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "education_curriculum_3_v1"
CONTRACT = PACK / "contract.detjson"
ASSESSMENT = PACK / "classroom_ui_pack.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "education_classroom_ui_pack.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
UI_RUNNER = ROOT / "tests" / "education_classroom_ui_pack_runner.mjs"
PREREQ_PA5 = ROOT / "tests" / "run_roadmap_v2_ta2_matrix_status_reconciliation_check.py"
PREREQ_TA2 = ROOT / "tests" / "run_roadmap_v2_ha2_education_assessment_pack_check.py"
PREREQ_MA2 = ROOT / "tests" / "run_roadmap_v2_ma3_seamgrim_curriculum_3_classroom_ui_pack_closure_check.py"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ha3-education-classroom-ui-pack] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be a JSON object")
    return payload


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing {missing}")


def run(args: list[str], *, timeout: float | None = None) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def check_files_and_docs() -> None:
    for path in [
        PACK / "README.md",
        CONTRACT,
        ASSESSMENT,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        UI_MODULE,
        APP,
        DEV_SURFACES,
        STYLES,
        UI_RUNNER,
    ]:
        require_file(path)
    require_tokens(DEV_SURFACES, ["education_classroom_ui_pack.js", "education-classroom-ui-pack", "__EDUCATION_CLASSROOM_UI_PACK__"])
    require_tokens(DEV_SURFACES_CSS, [".education-classroom-ui-pack", ".education-classroom-artifacts", ".education-classroom-preview"])


def check_status_closed() -> None:
    for line in read(MATRIX).splitlines():
        if "| 3마루 작업실마루 | 교사용/학생용 UI |" in line:
            if line.rstrip().split("|")[-2].strip() != "닫힘-동작":
                fail(f"하-3 status must be 닫힘-동작: {line}")
            return
    fail("missing 하-3 matrix line")


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 30,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 33,
        "roadmap_v2_pack_evidence_reference_closed": 50,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 56,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "education_curriculum_3_v1",
        "kind": "roadmap_v2_ha3_classroom_ui_pack_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closed_by": "HA3_CLASSROOM_UI_PACK_V1",
        "roadmap_coordinate": "하-3",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ha3_matrix_status": "닫힘-동작",
        "requires_ha2_closed": True,
        "requires_ma3_closed": True,
        "requires_ta3_reference": True,
        "education_classroom_ui_pack_claim": True,
        "teacher_mode_claim": True,
        "student_mode_claim": True,
        "assignment_panel_claim": True,
        "report_review_claim": True,
        "classroom_handoff_claim": True,
        "gradebook_write_claim": False,
        "student_personal_data_collection_claim": False,
        "remote_classroom_sync_claim": False,
        "live_submission_claim": False,
        "account_permission_change_claim": False,
        "state_hash_participation_claim": False,
        "parser_frontdoor_change": False,
        "grammar_claim": False,
        "current_stage": "HA3 classroom UI pack closure",
        "next_item": "HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)
    assessment = read_json(ASSESSMENT)
    if assessment.get("status") != "education_classroom_ui_pack_ready":
        fail(f"assessment status={assessment.get('status')!r}")
    ids = [row.get("id") for row in assessment.get("rows", [])]
    if ids != ["teacher_mode", "student_mode", "assignment_panel", "report_review", "classroom_handoff"]:
        fail(f"assessment rows mismatch: {ids!r}")
    for token in ["coordinate:하-3", "gradebook_write:false", "student_personal_data_collection:false", "remote_classroom_sync:false", "state_hash_participation:false"]:
        if token not in str(assessment.get("classroom_text", "")):
            fail(f"classroom text missing {token}")
    check_payload(ASSESSMENT)
    for payload in [contract, assessment]:
        for key, value in payload.get("false_claims", {}).items():
            if value is not False:
                fail(f"false claim {key}={value!r}")


def check_forbidden_claims() -> None:
    for path in [CONTRACT, ASSESSMENT, UI_MODULE]:
        text = read(path)
        forbidden = [
            "18/18 = 100%",
            "90/90 = 100%",
            "roadmap_v2_matrix_behavior_closed\": 31",
            "Studio-local 초장기 계획: 10/18",
            "gradebook_write_claim\": true",
            "student_personal_data_collection_claim\": true",
            "remote_classroom_sync_claim\": true",
            "live_submission_claim\": true",
            "account_permission_change_claim\": true",
            "state_hash_participation_claim\": true",
            "parser_frontdoor_change\": true",
            "grammar_claim\": true",
        ]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "education_curriculum_3_v1"], timeout=240)
    run([sys.executable, "tests/run_roadmap_v2_ha2_education_assessment_pack_check.py"], timeout=600)
    run(["node", "tests/education_classroom_ui_pack_runner.mjs"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ha3-education-classroom-ui-pack] OK")


if __name__ == "__main__":
    main()
