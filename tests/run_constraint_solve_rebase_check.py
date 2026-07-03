#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ROADMAP = ROOT / "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md"
DOC = ROOT / "CONSTRAINT_SOLVE_REBASE_V1.md"
PACK = ROOT / "pack" / "constraint_solve_rebase_v1"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
LANG_STATUS = ROOT / "docs" / "status" / "LANG_STATUS.md"


def fail(message: str) -> None:
    print(f"constraint_solve_rebase_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def require(path: str) -> Path:
    target = ROOT / path
    if not target.exists():
        fail(f"missing required path: {path}")
    return target


def require_tokens(path: str, tokens: list[str]) -> None:
    text = read(ROOT / path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path} missing tokens: {missing}")


def forbid_tokens(path: str, tokens: list[str]) -> None:
    text = read(ROOT / path)
    present = [token for token in tokens if token in text]
    if present:
        fail(f"{path} contains forbidden tokens: {present}")


def run(cmd: list[str], timeout: int = 180) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def require_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "constraint_solve_rebase_v1"])
    if proc.returncode != 0:
        fail("run_pack_golden constraint_solve_rebase_v1 failed:\n" + proc.stdout)


def main() -> None:
    require("CONSTRAINT_SOLVE_REBASE_V1.md")
    require("NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md")
    require("POLYNOMIAL_SOLVE_MINIMUM_V1.md")
    require("pack/constraint_solve_rebase_v1/README.md")
    require("pack/constraint_solve_rebase_v1/contract.detjson")
    require("pack/constraint_solve_rebase_v1/input.ddn")
    require("pack/constraint_solve_rebase_v1/golden.jsonl")
    for pack in [
        "connect_flow_v1n_closure_v1",
        "connect_flow_v1o_closure_v1",
        "connect_flow_v1p_closure_v1",
        "connect_flow_v1q_closure_v1",
        "connect_flow_v1r_closure_v1",
        "connect_flow_v1s_closure_v1",
        "connect_flow_v1t_closure_v1",
        "connect_flow_v1u_closure_v1",
        "connect_flow_v1v_closure_v1",
        "numeric_solver_capability_rebase_v1",
        "polynomial_solve_minimum_v1",
    ]:
        require(f"pack/{pack}/contract.detjson")

    require_tokens(
        "CONSTRAINT_SOLVE_REBASE_V1.md",
        [
            "CONSTRAINT_SOLVE_REBASE_V1",
            "post-solve validation",
            "solver-internal inequality",
            "이음관계.범위검사",
            "이음관계.풀고범위검사",
            "connect_flow_v1v_closure_v1",
            "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
            "No product code change",
            "docs/ssot/**",
        ],
    )
    require_tokens(
        "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md",
        [
            "CONSTRAINT_SOLVE_REBASE_V1",
            "pack/constraint_solve_rebase_v1",
            "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
        ],
    )
    require_tokens(
        "docs/context/all/DEV_SUMMARY.md",
        [
            "CONSTRAINT_SOLVE_REBASE_V1",
            "constraint_solve_rebase_v1",
            "post-solve validation",
            "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
            "docs/ssot/** 변경 없음",
        ],
    )
    forbid_tokens("CONSTRAINT_SOLVE_REBASE_V1.md", ["connect_flow_v1w_closure_v1", "simplex solver landed"])
    forbid_tokens("NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md", ["connect_flow_v1w_closure_v1"])
    if "CONSTRAINT_SOLVE_REBASE_V1" in read(LANG_STATUS):
        fail("LANG_STATUS must not be updated for checker-only constraint rebase")

    contract = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "runtime_claim": False,
        "product_code_change": False,
        "post_solve_range_validation_claim": True,
        "solver_internal_inequality_claim": False,
        "linear_programming_claim": False,
        "constraint_satisfaction_claim": False,
        "automatic_endpoint_solve_claim": False,
        "next_recommended_item": "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key}={contract.get(key)!r}, expected {value!r}")

    required_evidence = {
        "connect_flow_v1n_closure_v1",
        "connect_flow_v1o_closure_v1",
        "connect_flow_v1p_closure_v1",
        "connect_flow_v1q_closure_v1",
        "connect_flow_v1r_closure_v1",
        "connect_flow_v1s_closure_v1",
        "connect_flow_v1t_closure_v1",
        "connect_flow_v1u_closure_v1",
        "connect_flow_v1v_closure_v1",
        "numeric_solver_capability_rebase_v1",
        "polynomial_solve_minimum_v1",
    }
    if set(contract.get("evidence_packs") or []) != required_evidence:
        fail(f"unexpected evidence_packs: {contract.get('evidence_packs')!r}")

    for closure in ["connect_flow_v1n_closure_v1", "connect_flow_v1o_closure_v1"]:
        closure_contract = json.loads(read(ROOT / "pack" / closure / "contract.detjson"))
        if closure_contract.get("solver_constraint") is not False:
            fail(f"{closure} must keep solver_constraint=false")

    rows = [json.loads(line) for line in read(PACK / "golden.jsonl").splitlines() if line.strip()]
    if len(rows) != 1:
        fail("golden must contain one marker case")
    expected_stdout = [
        "CONSTRAINT_SOLVE_REBASE_V1",
        "constraint solve boundary rebase sealed",
        "boundary: post-solve range validation, not solver-internal inequality constraints",
        "next: LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
    ]
    if rows[0].get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {rows[0].get('stdout')!r}")

    require_pack_golden()
    require_docs_ssot_clean()
    print("constraint_solve_rebase_check: ok")


if __name__ == "__main__":
    main()

