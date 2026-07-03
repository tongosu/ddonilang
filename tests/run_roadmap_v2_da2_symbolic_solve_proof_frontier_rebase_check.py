#!/usr/bin/env python3
"""Validate ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "다-2_RECONCILIATION_REPORT_20260608.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
PACK = ROOT / "pack" / "roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"

SOLVE_PACKS = [
    "formula_relation_solve_v1",
    "relation_solve_system_2x2_v1",
    "relation_solve_ddn_bridge_v2",
    "relation_solve_wasm_cli_parity_v2",
    "polynomial_solve_minimum_v1",
    "numeric_root_finding_bisection_v1",
    "linear_inequality_solve_minimum_v1",
]
SYMBOLIC_PACKS = [
    "symbolic_relation_canon_v1",
    "symbolic_ddn_formula_bridge_v1",
    "symbolic_ddn_cli_parity_v1",
    "symbolic_rational_expr_v1",
    "symbolic_multivar_polynomial_v1",
    "symbolic_polynomial_simplify_v1",
    "symbolic_expand_factor_v1",
    "symbolic_diff_integral_v1",
    "symbolic_equivalence_v1",
]
PROOF_PACKS = [
    "proof_ddn_relation_bridge_v1",
    "proof_relation_equivalence_v1",
    "proof_relation_solve_consistency_v1",
    "proof_seum_runtime_bridge_v1",
    "proof_numeric_factor_certificate_strength_v1",
    "proof_tactic_symbolic_eq_v1",
    "proof_tactic_rewrite_chain_v1",
    "proof_ddn_jeunggeo_bridge_v1",
    "proof_runtime_smoke_v1",
    "proof_guard_tick_v1",
    "proof_guard_rollback_v1",
    "proof_alert_continue_v1",
]


def fail(message: str) -> None:
    print(f"[roadmap-v2-da2-symbolic-solve-proof-frontier-rebase] FAIL: {message}", file=sys.stderr)
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
        ROOT / "ROADMAP_V2_DA1_MATH_FIRST_RUN_FRONTIER_REBASE_V1.md",
        ROOT / "tests" / "run_roadmap_v2_da1_math_first_run_frontier_rebase_check.py",
    ]:
        require_file(path)
    for pack in SOLVE_PACKS + SYMBOLIC_PACKS + PROOF_PACKS:
        require_file(ROOT / "pack" / pack / "golden.jsonl")


def check_docs() -> None:
    shared = [
        "ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1",
        "ROADMAP_V2_DA3_SEAMGRIM_MATH_VIEW_FRONTIER_REBASE_V1",
        "DA2 symbolic/solve/proof frontier rebase 7/7 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 47/90 = 52%",
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
        "guide_status_change:true",
        "docs_ssot_change:false",
    ]
    for path in [DOC, REPORT, DEV_SUMMARY]:
        require_tokens(path, shared)
    require_tokens(PROJECT_STATUS, ["ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1", "47/90 = 52%", "59/90 = 66%", "다-2"])
    require_tokens(CHANGELOG, ["ROADMAP_V2 DA2 symbolic/solve/proof frontier rebase", "ROADMAP_V2_DA3_SEAMGRIM_MATH_VIEW_FRONTIER_REBASE_V1"])


def check_payloads() -> None:
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_v1",
        "kind": "roadmap_v2_da2_symbolic_solve_proof_frontier_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "closed_by": "ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1",
        "selected_coordinate": "다-2",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "math_runtime_change": False,
        "math_surface_change": False,
        "guide_status_change": True,
        "current_stage": "DA2 symbolic/solve/proof frontier rebase",
        "current_stage_closed": 7,
        "current_stage_total": 7,
        "current_stage_percent": 100,
        "roadmap_v2_matrix_behavior_closed": 47,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 52,
        "roadmap_v2_pack_evidence_reference_closed": 59,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 66,
        "studio_local_super_long_closed": 9,
        "studio_local_super_long_total": 18,
        "studio_local_super_long_percent": 50,
        "selected_next_work": "ROADMAP_V2_DA3_SEAMGRIM_MATH_VIEW_FRONTIER_REBASE_V1",
        "docs_ssot_change": False,
        "requires_docs_ssot_clean": True,
    }
    contract = read_json(CONTRACT)
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")

    reconciliation = read_json(RECONCILIATION)
    if reconciliation.get("schema") != "ddn.roadmap_v2.da2_symbolic_solve_proof_frontier_rebase.v1":
        fail("reconciliation schema mismatch")
    if reconciliation.get("coordinate") != "다-2":
        fail("reconciliation coordinate mismatch")
    if reconciliation.get("matrix_status_after") != "닫힘-동작":
        fail("reconciliation matrix_status_after mismatch")
    if reconciliation.get("lanes", {}).get("solve") != SOLVE_PACKS:
        fail("solve lane mismatch")
    if reconciliation.get("lanes", {}).get("symbolic") != SYMBOLIC_PACKS:
        fail("symbolic lane mismatch")
    if reconciliation.get("lanes", {}).get("proof") != PROOF_PACKS:
        fail("proof lane mismatch")


def check_da2_authority_state() -> None:
    require_tokens(
        MATRIX,
        ["| 2마루 닫힘마루 | symbolic/solve/proof 닫힘 | 정리/전개/미분/제한 solver/proof bridge | symbolic/proof pack PASS | 닫힘-동작 |"],
    )
    require_tokens(
        GUIDE,
        [
            "#### 다-2 — symbolic/solve/proof 닫힘",
            "| 현재 상태 | 닫힘-동작 |",
            "`math_symbolic_proof_2_v1`",
            "`roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_v1`",
        ],
    )
    require_tokens(
        TRACKER,
        [
            "| 7.6 | `다-2` | symbolic/solve/proof 닫힘 | 닫힘-동작 |",
            "| `다-2` | symbolic/solve/proof 닫힘 | 닫힘-동작 |",
        ],
    )
    require_tokens(
        MANIFEST,
        [
            "| `다-2` | solve lane:",
            "symbolic lane:",
            "proof lane:",
            "행렬 정합화: `python tests/run_roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_check.py`",
        ],
    )


def check_forbidden_claims() -> None:
    forbidden = [
        "18/18 = 100%",
        "90/90 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 48/90",
        '"roadmap_v2_matrix_behavior_closed": 48',
        '"math_runtime_change": true',
        '"math_surface_change": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_v1"], timeout=120)
    run([sys.executable, "tests/run_roadmap_v2_da1_math_first_run_frontier_rebase_check.py"], timeout=180)
    run([sys.executable, "tests/run_pack_golden.py", *SOLVE_PACKS], timeout=180)
    run([sys.executable, "tests/run_pack_golden.py", *SYMBOLIC_PACKS], timeout=180)
    run([sys.executable, "tests/run_pack_golden.py", *PROOF_PACKS], timeout=180)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_files()
    check_docs()
    check_payloads()
    check_da2_authority_state()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-da2-symbolic-solve-proof-frontier-rebase] OK")


if __name__ == "__main__":
    main()
