#!/usr/bin/env python3
"""Validate HA1_REPRESENTATIVE_TEACHING_SMOKE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "하-1_REPORT_20260609.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "education_curriculum_1_v1"
CONTRACT = PACK / "contract.detjson"
SMOKE = PACK / "teaching_smoke.detjson"

REPRESENTATIVES = {
    "math": ROOT / "pack" / "edu_s1_function_graph",
    "physics": ROOT / "pack" / "edu_p1_constant_accel",
    "economics": ROOT / "pack" / "edu_e1_supply_demand_tax",
}
REQUIRED_ARTIFACTS = ["lesson.ddn", "meta.toml", "view_spec.toml", "teacher_notes.md", "student_sheet.md"]


def fail(message: str) -> None:
    print(f"[roadmap-v2-ha1-teaching-smoke] FAIL: {message}", file=sys.stderr)
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
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        SMOKE,
    ]:
        require_file(path)
    for pack_dir in REPRESENTATIVES.values():
        for name in REQUIRED_ARTIFACTS:
            require_file(pack_dir / name)



def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "work_unit_closed": 5,
        "work_unit_total": 5,
        "work_unit_percent": 100,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 57,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 63,
        "roadmap_v2_pack_evidence_reference_closed": 60,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 67,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}, expected {value!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "education_curriculum_1_v1",
        "kind": "roadmap_v2_ha1_representative_teaching_smoke",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "education_runtime_change": False,
        "lesson_authoring_ui_change": False,
        "classroom_ui_change": False,
        "publication_workflow_claim": False,
        "ssot_textbook_rewrite_claim": False,
        "closed_by": "HA1_REPRESENTATIVE_TEACHING_SMOKE_V1",
        "roadmap_coordinate": "하-1",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "teaching_smoke_claim": True,
        "requires_lesson_ddn": True,
        "requires_curriculum_meta": True,
        "requires_view_spec": True,
        "requires_teacher_notes": True,
        "requires_student_sheet": True,
        "current_stage": "HA1 representative teaching smoke",
        "next_item": "ROADMAP_V2_POST_HA1_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    if contract.get("representative_subjects") != ["math", "physics", "economics"]:
        fail("representative_subjects mismatch")
    check_payload(CONTRACT)

    smoke = read_json(SMOKE)
    if smoke.get("status") != "representative_teaching_smoke_ready":
        fail(f"smoke status={smoke.get('status')!r}")
    rows = smoke.get("representatives")
    if not isinstance(rows, list) or len(rows) != 3:
        fail("representatives must contain 3 rows")
    subjects = [row.get("subject") for row in rows]
    if subjects != ["math", "physics", "economics"]:
        fail(f"representative subject mismatch: {subjects!r}")
    for row in rows:
        if row.get("requires") != REQUIRED_ARTIFACTS:
            fail(f"representative requires mismatch: {row!r}")
    check_payload(SMOKE)
    for key, value in smoke.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_representative_artifacts() -> None:
    for subject, pack_dir in REPRESENTATIVES.items():
        lesson = read(pack_dir / "lesson.ddn").strip()
        if not lesson:
            fail(f"{subject} lesson.ddn is empty")
        meta = tomllib.loads(read(pack_dir / "meta.toml"))
        if meta.get("schema") != "CurriculumMetaV1":
            fail(f"{subject} meta schema mismatch")
        if not meta.get("teacher_notes_ref") or not meta.get("student_sheet_ref"):
            fail(f"{subject} meta missing teacher/student refs")
        view = tomllib.loads(read(pack_dir / "view_spec.toml"))
        if view.get("schema") != "SeamgrimViewSpecV0":
            fail(f"{subject} view spec schema mismatch")
        if not view.get("required_views") or not view.get("required_gauges"):
            fail(f"{subject} view spec missing required views/gauges")
        for note_name in ["teacher_notes.md", "student_sheet.md"]:
            text = read(pack_dir / note_name).strip()
            if len(text.splitlines()) < 2:
                fail(f"{subject} {note_name} too small")


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 58/90",
        '"roadmap_v2_matrix_behavior_closed": 58',
        "ROADMAP_V2 행렬 닫힘-동작: 90/90",
        "Studio-local 초장기 계획: 18/18",
        '"new_education_runtime": true',
        '"education_runtime_change": true',
        '"lesson_authoring_ui_change": true',
        '"classroom_ui_change": true',
        '"publication_workflow_claim": true',
        '"ssot_textbook_rewrite_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [PACK / "README.md", CONTRACT, SMOKE]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "education_curriculum_1_v1"], timeout=120)
    for pack_name in ["edu_s1_function_graph", "edu_p1_constant_accel", "edu_e1_supply_demand_tax"]:
        run([sys.executable, "tests/run_education_curriculum_template_check.py", "--file", f"pack/{pack_name}/meta.toml"], timeout=120)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_representative_artifacts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ha1-teaching-smoke] OK")


if __name__ == "__main__":
    main()
