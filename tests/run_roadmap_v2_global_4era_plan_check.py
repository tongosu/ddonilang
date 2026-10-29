#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GLOBAL_4ERA_PLAN_V5_20260608.md"
INDEX = ROOT / "docs" / "context" / "roadmap" / "INDEX.md"
STUDIO_3ERA = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_STUDIO_PRODUCTIZATION_3ERA_PLAN_V4_20260605.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
PACK = ROOT / "pack" / "roadmap_v2_global_4era_plan_v5"
PLAN = PACK / "plan.detjson"
CONTRACT = PACK / "contract.detjson"
TA2_FIXTURE = ROOT / "pack" / "toolchain_pack_2_v1" / "fixtures" / "work_items.detjson"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
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
        INDEX,
        STUDIO_3ERA,
        MATRIX,
        PACK / "README.md",
        CONTRACT,
        PLAN,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        TA2_FIXTURE,
        ROOT / "pack" / "toolchain_pack_2_v1",
        ROOT / "tests" / "run_roadmap_v2_work_item_evidence_check.py",
        ROOT / "tests" / "run_roadmap_v2_pack_runner_basis_check.py",
        ROOT / "tests" / "run_roadmap_v2_pack_skeleton_check.py",
        ROOT / "tests" / "run_roadmap_v2_ga2_representative_grammar_rebase_check.py",
        ROOT / "tests" / "run_roadmap_v2_ga2_final_closure_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_doc_tokens() -> int:
    checks = [
        (
            DOC,
            [
                "ROADMAP_V2 Global 4-Era Plan v5",
                "ROADMAP_V2 matrix behavior-closed: `0/90 = 0%`",
                "ROADMAP_V2 pack evidence reference: `21/90 = 23%`",
                "Studio-local super-long plan: `5/18 = 28%`",
                "does not discard `ROADMAP_V2_STUDIO_PRODUCTIZATION_3ERA_PLAN_V4_20260605.md`",
                "`타-2`, `가-2`, `사-2`",
                "`라-2`, `마-2`, `아-2`, `파-2`, `하-2`, and `거-2`",
                "Do not add a separate `ROADMAP_V2_GLOBAL_SUPER_LONG_REBASE_V1` JIT",
                "`GA2_LANGUAGE_REPRESENTATIVE_PACK_CLOSURE_V1`",
                "`SA2_SPRITE_GRID2D_CLOSURE_V1`",
                "`TA2_MATRIX_STATUS_RECONCILIATION_V1`",
                "`MA2_STUDIO_PREREQ_UNLOCK_V1`",
                "high readiness / 진행",
                "작업 단위: 1/1 = 100% (`닫힘-문서`)",
                "ROADMAP_V2 행렬 닫힘-동작: 0/90 = 0%",
                "ROADMAP_V2 pack evidence 참고값: 21/90 = 23%",
                "Studio-local 초장기 계획: 5/18 = 28%",
                "No `18/18 = 100%` or `90/90 = 100%` official progress reintroduction",
                "`SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1` is not an unlock condition",
            ],
        ),
        (
            INDEX,
            [
                "ROADMAP_V2_GLOBAL_4ERA_PLAN_V5_20260608.md",
                "ROADMAP_V2 전역 4시대",
                "0/90 = 0%",
                "5/18 = 28%",
            ],
        ),
        (
            STUDIO_3ERA,
            [
                "셈그림/Studio 제품화",
                "3단계 시대제",
                "Progress Accounting",
                "3시대",
            ],
        ),
        (
            DEV_SUMMARY,
            [
                "[ROADMAP_V2][GLOBAL] Global 4-era plan v5",
                "ROADMAP_V2 행렬 닫힘-동작: 0/90 = 0%",
                "ROADMAP_V2 pack evidence 참고값: 21/90 = 23%",
                "Studio-local 초장기 계획: 5/18 = 28%",
                "현재 스테이지: global 4-era plan lock 1/1 = 100%",
                "다음 실제 JIT: `GA2_LANGUAGE_REPRESENTATIVE_PACK_CLOSURE_V1`",
            ],
        ),
    ]
    for path, tokens in checks:
        rc = require_tokens(path, tokens, "E_ROADMAP_V2_GLOBAL_4ERA_DOC")
        if rc:
            return rc
    return 0


def check_matrix_dependencies() -> int:
    return require_tokens(
        MATRIX,
        [
            "라-2 | 가-2 대표 문법 pack, 타-2 golden/checker",
            "마-2 | 다-1 math_function, 사-1 graph/table, 타-2",
            "아-2 | 가-2, 나-1 std_grid/input, 타-2",
            "파-2 | 나-3 std_resource/network/policy, 타-2",
            "하-2 | 타-2, 하-pack연결",
            "거-2 | 타-2, 자-1 또는 자-2",
            "| 2마루 닫힘마루 | CI/golden gate | golden/checker/CI | CI PASS |",
        ],
        "E_ROADMAP_V2_GLOBAL_4ERA_MATRIX",
    )


def check_contract() -> int:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_global_4era_plan_v5",
        "kind": "roadmap_v2_global_4era_plan",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "matrix_total_cells": 90,
        "matrix_behavior_closed_cells": 0,
        "matrix_behavior_percent": 0,
        "pack_evidence_reference_closed_cells": 21,
        "pack_evidence_reference_percent": 23,
        "studio_local_total_items": 18,
        "studio_local_closed_items": 5,
        "studio_local_percent": 28,
        "current_stage_closed": 1,
        "current_stage_total": 1,
        "current_stage_percent": 100,
        "next_jit": "GA2_LANGUAGE_REPRESENTATIVE_PACK_CLOSURE_V1",
        "forbidden_jit": "ROADMAP_V2_GLOBAL_SUPER_LONG_REBASE_V1",
        "numeric_track_consolidation_priority": "deferred_micro_slice_cleanup",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_ROADMAP_V2_GLOBAL_4ERA_CONTRACT", f"{key}={contract.get(key)!r}")
    return 0


def check_plan() -> int:
    plan = load_json(PLAN)
    if plan.get("schema") != "ddn.roadmap_v2.global_4era_plan.v5":
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_SCHEMA", repr(plan.get("schema")))
    progress = plan.get("progress", {})
    matrix = progress.get("roadmap_v2_matrix", {})
    evidence = progress.get("roadmap_v2_pack_evidence_reference", {})
    studio = progress.get("studio_local_super_long", {})
    stage = progress.get("current_stage", {})
    expected_progress = [
        (matrix, {"closed_cells": 0, "total_cells": 90, "percent": 0}),
        (evidence, {"closed_cells": 21, "total_cells": 90, "percent": 23}),
        (studio, {"closed_items": 5, "total_items": 18, "percent": 28}),
        (stage, {"closed": 1, "total": 1, "percent": 100, "closure_tier": "닫힘-문서"}),
    ]
    for payload, expected in expected_progress:
        for key, value in expected.items():
            if payload.get(key) != value:
                return fail("E_ROADMAP_V2_GLOBAL_4ERA_PROGRESS", f"{key}={payload.get(key)!r}")

    eras = plan.get("global_eras", [])
    if [era.get("id") for era in eras] != ["era1", "era2", "era3", "era4"]:
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_ERAS", repr(eras))
    ta2 = plan.get("ta2", {})
    if ta2.get("status_before_reconciliation") != "high_readiness_progressing":
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_TA2_STATUS", repr(ta2))
    if ta2.get("matrix_closed_claim") is not False:
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_TA2_MATRIX_CLAIM", repr(ta2))
    if ta2.get("prerequisite_for") != ["라-2", "마-2", "아-2", "파-2", "하-2", "거-2"]:
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_TA2_DEPS", repr(ta2.get("prerequisite_for")))
    if plan.get("immediate_jit_order", [None])[0] != "GA2_LANGUAGE_REPRESENTATIVE_PACK_CLOSURE_V1":
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_NEXT_JIT", repr(plan.get("immediate_jit_order")))
    forbidden = plan.get("forbidden_claims", {})
    for key, value in forbidden.items():
        if value is not False:
            return fail("E_ROADMAP_V2_GLOBAL_4ERA_FORBIDDEN", f"{key}={value!r}")
    return 0


def check_ta2_fixture() -> int:
    fixture = load_json(TA2_FIXTURE)
    work_items = {item.get("id"): item for item in fixture.get("work_items", [])}
    ta2 = work_items.get("타-2")
    if not ta2:
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_TA2_FIXTURE", "missing 타-2")
    required = {
        "report": "docs/status/roadmap_v2/타-2_REPORT_20260503.md",
        "pack": "pack/toolchain_pack_2_v1",
        "checker": "tests/run_roadmap_v2_work_item_evidence_check.py",
        "expected_artifact": "pack/toolchain_pack_2_v1/expected/work_item_evidence.detjson",
    }
    for key, value in required.items():
        if ta2.get(key) != value:
            return fail("E_ROADMAP_V2_GLOBAL_4ERA_TA2_FIXTURE", f"{key}={ta2.get(key)!r}")
    return 0


def check_pack_golden() -> int:
    proc = run(["python", "tests/run_pack_golden.py", "roadmap_v2_global_4era_plan_v5"], timeout=240)
    if proc.returncode != 0:
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_GOLDEN", proc.stdout.strip())
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_SSOT_STATUS", proc.stdout.strip())
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        return fail("E_ROADMAP_V2_GLOBAL_4ERA_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc_tokens,
        check_matrix_dependencies,
        check_contract,
        check_plan,
        check_ta2_fixture,
        check_pack_golden,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-global-4era-plan-v5] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
