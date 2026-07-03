#!/usr/bin/env python3
"""Validate ROADMAP_V2_DA1_MATH_FIRST_RUN_FRONTIER_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_DA1_MATH_FIRST_RUN_FRONTIER_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "다-1_RECONCILIATION_REPORT_20260608.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
PACK = ROOT / "pack" / "roadmap_v2_da1_math_first_run_frontier_rebase_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"


def fail(message: str) -> None:
    print(f"[roadmap-v2-da1-math-first-run-frontier-rebase] FAIL: {message}", file=sys.stderr)
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
        encoding = sys.stdout.encoding or "utf-8"
        safe_stdout = proc.stdout.encode(encoding, errors="replace").decode(encoding, errors="replace")
        print(safe_stdout, end="")
        fail(f"command failed: {' '.join(args)}")
    return proc


def check_files() -> None:
    for path in [
        DOC,
        REPORT,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        MATRIX,
        GUIDE,
        TRACKER,
        MANIFEST,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "ROADMAP_V2_POST_TA2_FRONTIER_REBASE_V1.md",
        ROOT / "ROADMAP_V2_DA1_FINAL_CLOSURE_V1.md",
        ROOT / "MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md",
        ROOT / "tests" / "run_roadmap_v2_post_ta2_frontier_rebase_check.py",
        ROOT / "tests" / "run_roadmap_v2_da1_final_closure_check.py",
        ROOT / "tests" / "run_math_vector_minimum_first_run_check.py",
    ]:
        require_file(path)


def check_docs() -> None:
    shared = [
        "ROADMAP_V2_DA1_MATH_FIRST_RUN_FRONTIER_REBASE_V1",
        "ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1",
        "DA1 math first-run frontier rebase 6/6 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 46/90 = 51%",
        "ROADMAP_V2 pack evidence 참고값: 59/90 = 66%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "닫힘-동작",
        "runtime_claim:false",
        "product_code_change:false",
        "product_ui_change:false",
        "matrix_closure_claim:true",
        "roadmap_matrix_increment:true",
        "math_runtime_change:false",
        "math_surface_change:false",
        "guide_status_change:false",
        "legacy_checker_unlock_claim:true",
        "docs_ssot_change:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_DA1_MATH_FIRST_RUN_FRONTIER_REBASE_V1", "46/90 = 51%", "59/90 = 66%", "다-1"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 DA1 math first-run frontier rebase", "ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1"])


def check_payloads() -> None:
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_da1_math_first_run_frontier_rebase_v1",
        "kind": "roadmap_v2_da1_math_first_run_frontier_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "ROADMAP_V2_DA1_MATH_FIRST_RUN_FRONTIER_REBASE_V1",
        "selected_coordinate": "다-1",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "math_runtime_change": False,
        "math_surface_change": False,
        "guide_status_change": False,
        "legacy_checker_unlock_claim": True,
        "current_stage": "DA1 math first-run frontier rebase",
        "current_stage_closed": 6,
        "current_stage_total": 6,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 46,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 51,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "selected_next_work": "ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1",
        "requires_docs_ssot_clean": True,
        "docs_ssot_change": False,
    }
    contract = read_json(CONTRACT)
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")

    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("schema") != "ddn.roadmap_v2.da1_math_first_run_frontier_rebase.v1":
        fail("reconciliation schema mismatch")
    if reconciliation.get("coordinate") != "다-1":
        fail("reconciliation coordinate mismatch")
    if reconciliation.get("matrix_status_after") != "닫힘-동작":
        fail("reconciliation matrix_status_after mismatch")
    progress = reconciliation.get("progress")
    claims = reconciliation.get("claims")
    if not isinstance(progress, dict) or not isinstance(claims, dict):
        fail("reconciliation progress/claims must be objects")
    for key in [
        "current_stage_closed",
        "current_stage_total",
        "current_stage_percent",
        "roadmap_v2_matrix_behavior_closed",
        "roadmap_v2_matrix_behavior_total",
        "roadmap_v2_matrix_behavior_percent",
        "roadmap_v2_pack_evidence_reference_closed",
        "roadmap_v2_pack_evidence_reference_total",
        "roadmap_v2_pack_evidence_reference_percent",
        "studio_local_super_long_closed",
        "studio_local_super_long_total",
        "studio_local_super_long_percent",
    ]:
        if progress.get(key) != expected[key]:
            fail(f"reconciliation progress {key}={progress.get(key)!r}")
    for key in [
        "runtime_claim",
        "product_code_change",
        "product_ui_change",
        "matrix_closure_claim",
        "roadmap_matrix_increment",
        "math_runtime_change",
        "math_surface_change",
        "guide_status_change",
        "legacy_checker_unlock_claim",
        "docs_ssot_change",
    ]:
        if claims.get(key) != expected[key]:
            fail(f"reconciliation claim {key}={claims.get(key)!r}")


def check_da1_authority_state() -> None:
    require_tokens(
        MATRIX,
        ["| 1마루 첫실행마루 | exact/vector/function 첫실행 | 정확한 수, 벡터, 함수 그래프 | math smoke pack | 닫힘-동작 |"],
    )
    require_tokens(
        GUIDE,
        [
            "#### 다-1 — exact/vector/function 첫실행",
            "| 현재 상태 | 닫힘-동작 |",
            "`math_vector_minimum_first_run_v1`",
            "`roadmap_v2_da1_math_first_run_frontier_rebase_v1`",
        ],
    )
    require_tokens(
        TRACKER,
        [
            "| 7.5 | `다-1` | exact/vector/function 첫실행 | 닫힘-동작 |",
            "| `다-1` | exact/vector/function 첫실행 | 닫힘-동작 |",
        ],
    )
    require_tokens(
        MANIFEST,
        [
            "| `다-1` | `ROADMAP_V2_DA1_FINAL_CLOSURE_V1`; `math_vector_minimum_first_run_v1`;",
            "행렬 정합화: `python tests/run_roadmap_v2_da1_math_first_run_frontier_rebase_check.py`",
        ],
    )


def check_forbidden_claims() -> None:
    forbidden = [
        "18/18 = 100%",
        "90/90 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 47/90",
        '"roadmap_v2_matrix_behavior_closed": 47',
        '"math_runtime_change": true',
        '"math_surface_change": true',
        '"guide_status_change": true',
        '"product_ui_change": true',
        '"product_code_change": true',
        '"runtime_claim": true',
    ]
    for path in [DOC, REPORT, PACK / "README.md", CONTRACT, RECONCILIATION]:
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            fail(f"{path.relative_to(ROOT)} contains forbidden claim: {present}")


def check_gates() -> None:
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_da1_math_first_run_frontier_rebase_v1"], timeout=120)
    run([sys.executable, "tests/run_math_vector_minimum_first_run_check.py"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_da1_final_closure_check.py"], timeout=180)
    run(
        [
            sys.executable,
            "tests/run_pack_golden.py",
            "math_vector_minimum_first_run_v1",
            "math_calculus_v1",
            "formula_relation_solve_v1",
            "relation_solve_system_2x2_v1",
            "relation_solve_ddn_bridge_v2",
            "relation_solve_wasm_cli_parity_v2",
            "math_numeric_int_v1",
            "math_numeric_diff_v1",
        ],
        timeout=180,
    )
    run([sys.executable, "tests/run_roadmap_v2_post_ta2_frontier_rebase_check.py"], timeout=180)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files()
    check_docs()
    check_payloads()
    check_da1_authority_state()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-da1-math-first-run-frontier-rebase] OK")


if __name__ == "__main__":
    main()
