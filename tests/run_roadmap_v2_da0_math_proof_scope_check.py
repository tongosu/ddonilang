#!/usr/bin/env python3
"""Validate DA0_MATH_PROOF_SCOPE_RECONCILIATION_V1."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MANIFEST = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "다-0_RECONCILIATION_REPORT_20260609.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "roadmap_v2_da0_math_proof_scope_v1"
CONTRACT = PACK / "contract.detjson"
RECONCILIATION = PACK / "reconciliation.detjson"

REPRESENTATIVE_PACKS = [
    "math_vector_minimum_first_run_v1",
    "math_calculus_v1",
    "formula_relation_solve_v1",
    "relation_solve_system_2x2_v1",
    "symbolic_relation_canon_v1",
    "symbolic_ddn_formula_bridge_v1",
    "symbolic_rational_expr_v1",
    "symbolic_diff_integral_v1",
    "proof_ddn_relation_bridge_v1",
    "proof_relation_equivalence_v1",
    "proof_runtime_smoke_v1",
    "proof_guard_tick_v1",
    "proof_guard_rollback_v1",
    "proof_alert_continue_v1",
]


def fail(message: str) -> None:
    print(f"[roadmap-v2-da0-math-proof-scope] FAIL: {message}", file=sys.stderr)
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
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        RECONCILIATION,
        ROOT / "pack" / "roadmap_v2_da1_math_first_run_frontier_rebase_v1" / "contract.detjson",
        ROOT / "pack" / "roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_v1" / "contract.detjson",
        ROOT / "pack" / "roadmap_v2_da3_seamgrim_math_view_frontier_rebase_v1" / "contract.detjson",
        ROOT / "pack" / "roadmap_v2_da4_math_package_share_frontier_rebase_v1" / "contract.detjson",
        ROOT / "pack" / "roadmap_v2_da5_math_lts_frontier_rebase_v1" / "contract.detjson",
    ]:
        require_file(path)
    for pack in REPRESENTATIVE_PACKS:
        require_file(ROOT / "pack" / pack / "golden.jsonl")
    require_tokens(MATRIX, ["| 0마루 씨앗마루 | math/proof 범위 확정 | math_exact/vector/matrix/symbolic/solve/proof_min | math library proposal | 닫힘-동작 |"])
    da0_section = section(GUIDE, "#### 다-0 — math/proof 범위 확정")
    if "| 현재 상태 | 닫힘-동작 |" not in da0_section:
        fail("GUIDE 다-0 status is not 닫힘-동작")
    require_tokens(TRACKER, ["| 7.45 | `다-0` | math/proof 범위 확정 | 닫힘-동작 |", "다-0_RECONCILIATION_REPORT_20260609.md"])
    require_tokens(MANIFEST, ["| `다-0` | math/proof scope source; DA1-DA5 downstream math/proof evidence; `roadmap_v2_da0_math_proof_scope_v1` |"])
    shared = [
        "DA0_MATH_PROOF_SCOPE_RECONCILIATION_V1",
        "DA0 Math proof scope 5/5 = 100%",
        "ROADMAP_V2 행렬 닫힘-동작: 82/90 = 91%",
        "ROADMAP_V2 docs-closed: 5/90 = 6%",
        "ROADMAP_V2 pack evidence 참고값: 84/90 = 93%",
        "Studio-local 초장기 계획: 9/18 = 50%",
        "MA0_CURRICULUM_CATALOG_RECONCILIATION_V1",
    ]
    for path in [REPORT, DEV_SUMMARY, PROJECT_STATUS]:
        require_tokens(path, shared)
    require_tokens(CHANGELOG, ["ROADMAP_V2 DA0 math/proof scope reconciliation", "82/90 = 91%", "84/90 = 93%"])
    total, behavior, docs = count_matrix_statuses()
    if total != 90 or behavior != 82 or docs != 5:
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
        "roadmap_v2_matrix_behavior_closed": 82,
        "roadmap_v2_matrix_behavior_total": 90,
        "roadmap_v2_matrix_behavior_percent": 91,
        "roadmap_v2_docs_closed": 5,
        "roadmap_v2_docs_total": 90,
        "roadmap_v2_docs_percent": 6,
        "roadmap_v2_pack_evidence_reference_closed": 84,
        "roadmap_v2_pack_evidence_reference_total": 90,
        "roadmap_v2_pack_evidence_reference_percent": 93,
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
        "pack": "roadmap_v2_da0_math_proof_scope_v1",
        "kind": "roadmap_v2_da0_math_proof_scope_reconciliation",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "math_runtime_change": False,
        "math_surface_change": False,
        "parser_frontdoor_change": False,
        "symbolic_solve_proof_closure_claim": False,
        "math_lts_certification_claim": False,
        "new_theorem_prover_claim": False,
        "closed_by": "DA0_MATH_PROOF_SCOPE_RECONCILIATION_V1",
        "roadmap_coordinate": "다-0",
        "matrix_closure_claim": True,
        "matrix_closure_tier": "닫힘-동작",
        "roadmap_matrix_increment": True,
        "next_item": "MA0_CURRICULUM_CATALOG_RECONCILIATION_V1",
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
    for key in ["scope_source", "first_run", "symbolic_solve_proof", "downstream_scope_consumers"]:
        if not isinstance(reconciliation.get("evidence", {}).get(key), dict):
            fail(f"missing evidence axis: {key}")
    for key, value in reconciliation.get("false_claims", {}).items():
        if value is not False:
            fail(f"false claim {key}={value!r}")
    check_payload(RECONCILIATION)


def check_forbidden_claims() -> None:
    forbidden = [
        "ROADMAP_V2 행렬 닫힘-동작: 83/90",
        '"roadmap_v2_matrix_behavior_closed": 83',
        '"math_runtime_change": true',
        '"math_surface_change": true',
        '"parser_frontdoor_change": true',
        '"symbolic_solve_proof_closure_claim": true',
        '"math_lts_certification_claim": true',
        '"new_theorem_prover_claim": true',
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
    run([sys.executable, "tests/run_pack_golden.py", "roadmap_v2_da0_math_proof_scope_v1"], timeout=120)
    run([sys.executable, "tests/run_pack_golden.py", *REPRESENTATIVE_PACKS], timeout=240)
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_docs()
    check_contracts()
    check_forbidden_claims()
    check_gates()
    print("[roadmap-v2-da0-math-proof-scope] OK")


if __name__ == "__main__":
    main()
