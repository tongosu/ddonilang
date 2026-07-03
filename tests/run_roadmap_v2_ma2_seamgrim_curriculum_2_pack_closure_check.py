#!/usr/bin/env python3
"""Validate MA2_SEAMGRIM_CURRICULUM_2_PACK_CLOSURE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "MA2_SEAMGRIM_CURRICULUM_2_PACK_CLOSURE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "마-2_REPORT_20260608.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "seamgrim_curriculum_2_v1"
CONTRACT = PACK / "contract.detjson"
CLOSURE = PACK / "curriculum_2.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-ma2-curriculum-2-closure] FAIL: {message}", file=sys.stderr)
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
        DOC,
        REPORT,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        DEV_SUMMARY,
        PACK / "README.md",
        CONTRACT,
        CLOSURE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "seamgrim_curriculum_batch_smoke_v1" / "golden.jsonl",
        ROOT / "pack" / "education_curriculum_template_v1" / "README.md",
        ROOT / "tests" / "run_education_curriculum_template_check.py",
        ROOT / "tests" / "run_seamgrim_education_curriculum_template_check.py",
    ]:
        require_file(path)
    for lesson_pack in [
        "edu_seamgrim_rep_math_function_line_v1",
        "edu_seamgrim_rep_phys_projectile_xy_v1",
        "edu_seamgrim_rep_econ_supply_demand_tax_v1",
    ]:
        for name in ["lesson.ddn", "teacher_notes.md", "student_sheet.md"]:
            require_file(ROOT / "pack" / lesson_pack / name)

    shared_tokens = [
        "MA2_SEAMGRIM_CURRICULUM_2_PACK_CLOSURE_V1",
        "MA2 curriculum pack closure 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 4/90 = 4%",
        "ROADMAP_V2 pack evidence 참고값: 23/90 = 26%",
        "Studio-local 초장기 계획: 6/18 = 33%",
        "MA3_STUDIO_CLASSROOM_WORKBENCH_PREREQ_REBASE_V1",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared_tokens)
    require_tokens(
        MATRIX,
        [
            "| 2마루 닫힘마루 | 교과 pack 닫힘 | 차시별 D-PACK + 활동지 연결 | pack/checker PASS | 닫힘-동작 |",
            "| 2마루 닫힘마루 | 대표 문법 pack 닫힘 | 채비/훅/조건/임자/계약 대표 pack | golden/checker PASS | 닫힘-동작 |",
            "| 2마루 닫힘마루 | sprite/grid2d 닫힘 | sprite skin, grid2d game input | sprite/grid pack | 닫힘-동작 |",
            "| 2마루 닫힘마루 | CI/golden gate | golden/checker/CI | CI PASS | 닫힘-동작 |",
        ],
    )
    require_tokens(GUIDE, ["#### 마-2", "| 현재 상태 | 닫힘-동작 |", "| pack 후보 | `seamgrim_curriculum_2_v1` |"])
    require_tokens(TRACKER, ["| 17 | `마-2` | 교과 pack 닫힘 | 닫힘-동작 |"])
    require_tokens(MANIFEST, ["| `마-2` | `seamgrim_curriculum_2_v1`; consumes"])


def check_payload(path: Path) -> None:
    payload = read_json(path)
    progress = payload.get("progress", payload)
    expected = {
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 4,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 4,
        "roadmap_v2_pack_evidence_reference_closed": 23,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 26,
        "studio_local_super_long_closed": 6,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 33,
    }
    for key, value in expected.items():
        if progress.get(key) != value:
            fail(f"{path.relative_to(ROOT)} {key}={progress.get(key)!r}")


def check_contracts() -> None:
    contract = read_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_curriculum_2_v1",
        "kind": "roadmap_v2_ma2_curriculum_pack_closure",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "MA2_SEAMGRIM_CURRICULUM_2_PACK_CLOSURE_V1",
        "roadmap_coordinate": "마-2",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "ma2_matrix_status": "닫힘-동작",
        "lesson_smoke_cases": 201,
        "representative_lesson_count": 3,
        "requires_teacher_notes": True,
        "requires_student_sheets": True,
        "requires_curriculum_template_check": True,
        "requires_browser_template_smoke": True,
        "current_stage": "MA2 curriculum pack closure",
        "next_item": "MA3_STUDIO_CLASSROOM_WORKBENCH_PREREQ_REBASE_V1",
        "forbidden_unlock_condition": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}")
    check_payload(CONTRACT)

    closure = read_json(CLOSURE)
    if closure.get("status") != "behavior_closed":
        fail(f"closure status={closure.get('status')!r}")
    if closure.get("ma2_matrix_status") != "닫힘-동작":
        fail("closure must record 마-2 as 닫힘-동작")
    if closure.get("matrix_closure_claim") is not True:
        fail("closure must claim matrix closure")
    check_payload(CLOSURE)
    false_claims = closure.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            fail(f"false claim {key}={value!r}")


def check_curriculum_payload() -> None:
    closure = read_json(CLOSURE)
    pack = closure.get("curriculum_pack", {})
    expected_pack = {
        "pack_id": "seamgrim_curriculum_2_v1",
        "batch_smoke_pack": "seamgrim_curriculum_batch_smoke_v1",
        "batch_smoke_cases": 201,
        "template_pack": "education_curriculum_template_v1",
        "browser_template_smoke": "tests/run_seamgrim_education_curriculum_template_check.py",
    }
    for key, value in expected_pack.items():
        if pack.get(key) != value:
            fail(f"curriculum_pack {key}={pack.get(key)!r}")
    lessons = closure.get("representative_lessons", [])
    if len(lessons) != 3:
        fail(f"representative lesson count={len(lessons)}")
    required_ids = {
        "rep_math_function_line_v1",
        "rep_phys_projectile_xy_v1",
        "rep_econ_supply_demand_tax_v1",
    }
    if {row.get("lesson_id") for row in lessons} != required_ids:
        fail(f"lesson ids={lessons!r}")


def check_batch_case_count() -> None:
    golden = ROOT / "pack" / "seamgrim_curriculum_batch_smoke_v1" / "golden.jsonl"
    count = sum(1 for line in read(golden).splitlines() if line.strip())
    if count != 201:
        fail(f"seamgrim_curriculum_batch_smoke_v1 case count={count}")


def check_forbidden_progress_claims() -> None:
    for path in [DOC, REPORT, CONTRACT, CLOSURE]:
        text = read(path)
        forbidden = ["18/18 = 100%", "90/90 = 100%"]
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden progress claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "seamgrim_curriculum_2_v1"], timeout=240)
    run([sys.executable, "tests/run_pack_golden.py", "seamgrim_curriculum_batch_smoke_v1"], timeout=420)
    run([sys.executable, "tests/run_education_curriculum_template_check.py"], timeout=240)
    run([sys.executable, "tests/run_seamgrim_education_curriculum_template_check.py"], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files_and_docs()
    check_contracts()
    check_curriculum_payload()
    check_batch_case_count()
    check_forbidden_progress_claims()
    check_gates()
    print("[roadmap-v2-ma2-curriculum-2-closure] OK")


if __name__ == "__main__":
    main()
