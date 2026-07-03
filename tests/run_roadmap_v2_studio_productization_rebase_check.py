#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1.md"
REPORT = ROOT / "docs" / "studio" / "STUDIO_PRODUCTIZATION_REBASE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "roadmap_v2_studio_productization_rebase_v1"
REBASE = PACK / "rebase.detjson"
STUDIO_ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
NUMERIC_ROADMAP = ROOT / "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
BASELINE_ASSESSMENT = ROOT / "docs" / "context" / "all" / "ACTUAL_BASELINE_ASSESSMENT_20260606.md"
BASELINE_UNCOMMITTED = ROOT / "docs" / "context" / "all" / "BASELINE_UNCOMMITTED_FILES_20260606.txt"
BASELINE_DIFF_STAT = ROOT / "docs" / "context" / "all" / "BASELINE_DIFF_STAT_20260606.txt"
BASELINE_TEST_RESULT = ROOT / "docs" / "context" / "all" / "BASELINE_TEST_RESULT_20260606.txt"
BASELINE_EXIT_AUDIT = ROOT / "docs" / "context" / "all" / "BASELINE_EXIT_AUDIT_20260606.txt"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def require_files() -> int:
    required = [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        REBASE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        STUDIO_ROADMAP,
        NUMERIC_ROADMAP,
        MATRIX,
        ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md",
        ROOT / "NUMERIC_ROOT_FINDING_V1.md",
        ROOT / "tests" / "run_next_work_queue_after_connect_check.py",
        ROOT / "tests" / "run_numeric_root_finding_check.py",
        BASELINE_ASSESSMENT,
        BASELINE_UNCOMMITTED,
        BASELINE_DIFF_STAT,
        BASELINE_TEST_RESULT,
        BASELINE_EXIT_AUDIT,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_STUDIO_REBASE_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    checks = [
        (
            DOC,
            [
                "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1",
                "documentation/checker-only",
                "마-3",
                "하-3",
                "라-3",
                "타-3",
                "다-1/다-2",
                "`사-3` is not used",
                "NUMERIC_ROOT_FINDING_V1",
                "NuriGym `아-2`",
                "18 fixed items",
                "Micro-Slice Gate",
                "Actual Baseline Assessment",
                "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
                "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1",
                "docs/ssot/**",
            ],
            "E_ROADMAP_V2_STUDIO_REBASE_DOC",
        ),
        (
            REPORT,
            [
                "Studio Productization Rebase V1",
                "primary coordinate: `마-3`",
                "rejected: `사-3`",
                "total items: 18",
                "5/18 = 28%",
                "matrix behavior-closed ROADMAP_V2: 0/90 = 0%",
                "pack evidence reference ROADMAP_V2: 21/90 = 23%",
                "documentation-reference ROADMAP_V2: 72/90 = 80%",
                "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
            ],
            "E_ROADMAP_V2_STUDIO_REBASE_REPORT",
        ),
        (
            INDEX,
            [
                "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1",
                "docs/studio/STUDIO_PRODUCTIZATION_REBASE_V1.md",
                "pack/roadmap_v2_studio_productization_rebase_v1",
                "tests/run_roadmap_v2_studio_productization_rebase_check.py",
            ],
            "E_ROADMAP_V2_STUDIO_REBASE_INDEX",
        ),
        (
            STUDIO_ROADMAP,
            [
                "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1",
                "primary coordinate `마-3`",
                "super-long denominator is 18 items",
                "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
            ],
            "E_ROADMAP_V2_STUDIO_REBASE_STUDIO_ROADMAP",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_matrix_alignment() -> int:
    return require_tokens(
        MATRIX,
        [
            "마 셈그림 교과 실험실",
            "수업용 작업실",
            "사 보개·공간·게임",
            "space3d/game preview",
            "다 수학·심볼릭·증명 라이브러리",
            "셈그림 수학 보개 연결",
            "타 검증·팩·도구줄",
        ],
        "E_ROADMAP_V2_STUDIO_REBASE_MATRIX",
    )


def check_numeric_anchor() -> int:
    for path, tokens in [
        (
            NUMERIC_ROADMAP,
            [
                "NUMERIC_ROOT_FINDING_V1",
                "Status: closed by `NUMERIC_ROOT_FINDING_V1.md`",
                "numeric_root_finding_bisection_v1",
            ],
        ),
        (
            ROOT / "NUMERIC_ROOT_FINDING_V1.md",
            [
                "NUMERIC_ROOT_FINDING_V1",
                "수치해.이분법",
                "pack/numeric_root_finding_bisection_v1",
                "tests/run_numeric_root_finding_check.py",
            ],
        ),
    ]:
        rc = require_tokens(path, tokens, "E_ROADMAP_V2_STUDIO_REBASE_NUMERIC")
        if rc:
            return rc
    return 0


def check_contract_and_rebase() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_studio_productization_rebase_v1",
        "kind": "roadmap_v2_studio_productization_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1",
        "primary_coordinate": "마-3",
        "rejected_coordinate": "사-3",
        "selected_next_item": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "fallback_next_item": "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1",
        "super_long_total_items": 18,
        "super_long_closed_items": 5,
        "roadmap_v2_total_cells": 90,
        "roadmap_v2_queue_expanded_closed_cells": 21,
        "roadmap_v2_behavior_closed_cells": 21,
        "roadmap_v2_behavior_percent": 23,
        "roadmap_v2_documentation_reference_closed_cells": 72,
        "roadmap_v2_documentation_reference_percent": 80,
        "baseline_assessment_status": "reassessed_pass_unlocked",
        "baseline_unlock_closed_item": "dirty baseline verification/separation",
        "baseline_unlock_super_long_delta": 1,
        "baseline_unlock_roadmap_v2_delta": 0,
        "baseline_assessment": "docs/context/all/ACTUAL_BASELINE_ASSESSMENT_20260606.md",
        "runtime_surface_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_change": False,
        "nurigym_a2_claim": False,
        "public_release_claim": False,
        "new_behavior_closure_claim": False,
        "mini_plan_progress_increase": False,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_ROADMAP_V2_STUDIO_REBASE_CONTRACT", f"{key}={contract.get(key)!r}")

    rebase = load_json(REBASE)
    if rebase.get("schema") != "ddn.roadmap_v2.studio_productization_rebase.v1":
        return fail("E_ROADMAP_V2_STUDIO_REBASE_SCHEMA", repr(rebase.get("schema")))
    if rebase.get("primary_coordinate") != "마-3":
        return fail("E_ROADMAP_V2_STUDIO_REBASE_PRIMARY", repr(rebase.get("primary_coordinate")))
    if "사-3" not in json.dumps(rebase.get("rejected_coordinates", []), ensure_ascii=False):
        return fail("E_ROADMAP_V2_STUDIO_REBASE_REJECTED", repr(rebase.get("rejected_coordinates")))
    progress = rebase.get("super_long_progress", {})
    if progress.get("total_items") != 18 or progress.get("closed_items") != 5 or progress.get("percent") != 28:
        return fail("E_ROADMAP_V2_STUDIO_REBASE_PROGRESS", repr(progress))
    if progress.get("basis") != "behavior_closed_only":
        return fail("E_ROADMAP_V2_STUDIO_REBASE_PROGRESS_BASIS", repr(progress.get("basis")))
    if progress.get("documentation_reference_closed_items") != 18 or progress.get("documentation_reference_percent") != 100:
        return fail("E_ROADMAP_V2_STUDIO_REBASE_DOC_PROGRESS", repr(progress))
    roadmap = rebase.get("roadmap_progress", {})
    expected_progress = {
        "roadmap_v2_total_cells": 90,
        "queue_expanded_closed_cells": 21,
        "queue_expanded_percent": 23,
        "behavior_closed_cells": 21,
        "behavior_percent": 23,
        "documentation_reference_closed_cells": 72,
        "documentation_reference_percent": 80,
        "primary_branch_closed_maru": 0,
        "primary_branch_total_maru": 6,
        "primary_maru_required_evidence": 4,
        "primary_maru_done_evidence": 1,
        "primary_maru_percent": 25,
    }
    for key, value in expected_progress.items():
        if roadmap.get(key) != value:
            return fail("E_ROADMAP_V2_STUDIO_REBASE_ROADMAP_PROGRESS", f"{key}={roadmap.get(key)!r}")
    baseline = rebase.get("baseline_assessment", {})
    if baseline.get("status") != "reassessed_pass_unlocked":
        return fail("E_ROADMAP_V2_STUDIO_REBASE_BASELINE_STATUS", repr(baseline))
    expected_statuses = {
        "cargo_check": "PASS",
        "cargo_test_lang": "PASS",
        "cargo_test_teul_cli": "PASS",
        "cargo_test_tool_wasm": "PASS",
        "seamgrim_intro_exec_wasm": "PASS",
        "seamgrim_playground_smoke": "PASS",
        "seamgrim_product_stabilization": "PASS",
        "seamgrim_runtime_5min": "PASS",
        "seamgrim_education_curriculum_template": "PASS",
        "nurigym_stale_hash_pack_golden": "PASS",
    }
    actual_statuses = {row.get("id"): row.get("status") for row in baseline.get("test_summary", [])}
    if actual_statuses != expected_statuses:
        return fail("E_ROADMAP_V2_STUDIO_REBASE_BASELINE_TESTS", repr(actual_statuses))
    false_claims = rebase.get("false_claims", {})
    for key, value in false_claims.items():
        if value is not False:
            return fail("E_ROADMAP_V2_STUDIO_REBASE_FALSE_CLAIM", f"{key}={value!r}")
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1",
        "studio productization coordinate lock sealed",
        "primary: 마-3",
        "next: SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "progress: 5/18 = 28%",
    ]
    if payload.get("stdout") != expected:
        return fail("E_ROADMAP_V2_STUDIO_REBASE_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "roadmap_v2_studio_productization_rebase_v1"],
        ["python", "tests/run_next_work_queue_after_connect_check.py"],
        ["python", "tests/run_numeric_root_finding_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=240)
        if proc.returncode != 0:
            return fail("E_ROADMAP_V2_STUDIO_REBASE_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    return require_tokens(
        DEV_SUMMARY,
        [
            "ROADMAP_V2_STUDIO_PRODUCTIZATION_REBASE_V1",
            "roadmap_v2_studio_productization_rebase_v1",
            "ACTUAL_BASELINE_ASSESSMENT_20260606.md",
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
            "초장기 계획: 1시대 5/5 = 100%, 전체 5/18 = 28%",
            "ROADMAP_V2: 닫힘-동작 21/90 = 23%, 닫힘-문서 72/90 = 80%",
            "docs/ssot/** 변경 없음",
        ],
        "E_ROADMAP_V2_STUDIO_REBASE_DEV_SUMMARY",
    )


def check_baseline_assessment() -> int:
    return require_tokens(
        BASELINE_ASSESSMENT,
        [
            "Actual Baseline Assessment 2026-06-06",
            "cargo check",
            "PASS",
            "cargo test --manifest-path tool/Cargo.toml --features wasm",
            "FAIL",
            "ai_prompt_output_matches_golden",
            "BrowseScreen.showLessonDetail",
            "닫힘-동작: 21/90 = 23%",
            "닫힘-문서: 72/90 = 80%",
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
            "No `docs/ssot/**` change",
        ],
        "E_ROADMAP_V2_STUDIO_REBASE_BASELINE_ASSESSMENT",
    )


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_ROADMAP_V2_STUDIO_REBASE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_ROADMAP_V2_STUDIO_REBASE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_matrix_alignment,
        check_numeric_anchor,
        check_contract_and_rebase,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_baseline_assessment,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-studio-productization-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
