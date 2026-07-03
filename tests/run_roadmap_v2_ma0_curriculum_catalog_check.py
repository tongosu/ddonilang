#!/usr/bin/env python3
"""Validate MA0_CURRICULUM_CATALOG_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "마-0_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
LESSON_CATALOG = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"
PACK = ROOT / "pack" / "roadmap_v2_ma0_curriculum_catalog_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"

CURRICULUM_PACKS = [
    "education_curriculum_1_v1",
    "seamgrim_curriculum_2_v1",
    "seamgrim_curriculum_3_v1",
    "seamgrim_curriculum_4_v1",
    "seamgrim_curriculum_5_v1",
]


def fail(message: str) -> None:
    print(f"[roadmap-v2-ma0-curriculum-catalog] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict:
    try:
        payload = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(payload, dict):
        fail(f"{path.relative_to(ROOT)} must be JSON object")
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


def section(path: Path, heading: str) -> str:
    text = read(path)
    start = text.find(heading)
    if start < 0:
        fail(f"{path.relative_to(ROOT)} missing section {heading}")
    next_heading = text.find("\n#### ", start + 1)
    if next_heading < 0:
        return text[start:]
    return text[start:next_heading]


def count_matrix_statuses() -> tuple[int, int, int]:
    rows = []
    for line in read(MATRIX).splitlines():
        if not line.startswith("| ") or "마루" not in line or line.startswith("| 마루"):
            continue
        cols = [col.strip() for col in line.strip().strip("|").split("|")]
        if len(cols) == 5 and cols[0] and cols[0][0] in "012345" and "마루" in cols[0]:
            rows.append(cols)
    return (
        len(rows),
        sum(1 for row in rows if row[-1] == "닫힘-동작"),
        sum(1 for row in rows if row[-1] == "닫힘-문서"),
    )


def check_catalog() -> None:
    payload = read_json(LESSON_CATALOG)
    lessons = payload.get("lessons")
    if not isinstance(lessons, list):
        fail("lesson catalog lessons must be a list")
    if len(lessons) < 200:
        fail(f"lesson catalog too small: {len(lessons)}")
    subjects = Counter(str(row.get("subject", "")) for row in lessons if isinstance(row, dict))
    for subject in ["math", "physics", "econ", "science"]:
        if subjects[subject] < 1:
            fail(f"lesson catalog missing subject {subject}: {subjects}")


def check_docs() -> None:
    for path in [
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        LESSON_CATALOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "pack" / "education_curriculum_template_v1" / "README.md",
        ROOT / "tests" / "run_education_curriculum_template_check.py",
        ROOT / "tests" / "run_seamgrim_education_curriculum_template_check.py",
    ]:
        require_file(path)
    for pack in CURRICULUM_PACKS:
        require_file(ROOT / "pack" / pack / "golden.jsonl")
    require_tokens(MATRIX, ["| 0마루 씨앗마루 | 교과 카탈로그 | 수학/물리/경제/과학 차시 목록 | curriculum catalog | 닫힘-동작 |"])
    ma0_section = section(GUIDE, "#### 마-0 — 교과 카탈로그")
    if "| 현재 상태 | 닫힘-동작 |" not in ma0_section:
        fail("GUIDE 마-0 status is not 닫힘-동작")
    require_tokens(TRACKER, ["| 16.65 | `마-0` | 교과 카탈로그 | 닫힘-동작 |", "마-0_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `마-0` | Seamgrim lesson catalog; `education_curriculum_template_v1`; downstream curriculum packs; `roadmap_v2_ma0_curriculum_catalog_v1` |"])
    shared = [
        "MA0_CURRICULUM_CATALOG_RECONCILIATION_V1",
        "MA0 Curriculum catalog 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 83/90 = 92%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 85/90 = 94%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "BA0_FREE_LAB_SEED_REASSESSMENT_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 MA0 curriculum catalog reconciliation", "83/90 = 92%", "85/90 = 94%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior < 83 or docs != 5:
        fail(f"matrix counts mismatch: rows={total} behavior={behavior} docs={docs}")


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
        "roadmap_v2_matrix_behavior_closed": 83,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 92,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 85,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 94,
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
        "pack": "roadmap_v2_ma0_curriculum_catalog_v1",
        "kind": "roadmap_v2_ma0_curriculum_catalog_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "lesson_authoring_ui_change": False,
        "textbook_rewrite_claim": False,
        "remote_classroom_sync_claim": False,
        "publication_workflow_claim": False,
        "new_lesson_catalog_claim": False,
        "closed_by": "MA0_CURRICULUM_CATALOG_RECONCILIATION_V1",
        "roadmap_coordinate": "마-0",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "next_item": "BA0_FREE_LAB_SEED_REASSESSMENT_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")
    check_payload(CONTRACT)
    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("status") != "behavior_closed":
        fail("reconciliation status mismatch")
    if reconciliation.get("matrix_status_record", {}).get("new_status") != "닫힘-동작":
        fail("matrix status record mismatch")
    for key in ["lesson_catalog", "curriculum_meta", "browse_detail", "downstream_curriculum"]:
        if not isinstance(reconciliation.get("evidence", {}).get(key), dict):
            fail(f"missing evidence axis: {key}")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    check_payload(RECONCILIATION)


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 84/90",
        '"roadmap_v2_matrix_behavior_closed": 84',
        '"lesson_authoring_ui_change": true',
        '"textbook_rewrite_claim": true',
        '"remote_classroom_sync_claim": true',
        '"publication_workflow_claim": true',
        '"new_lesson_catalog_claim": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_ma0_curriculum_catalog_v1"], timeout=120)
    run([sys.executable, "tests/run_education_curriculum_template_check.py"], timeout=120)
    run([sys.executable, "tests/run_seamgrim_education_curriculum_template_check.py"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", *CURRICULUM_PACKS], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_catalog()
    check_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-ma0-curriculum-catalog] OK")


if __name__ == "__main__":
    main()
