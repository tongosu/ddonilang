#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ROADMAP = ROOT / "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md"
DOC = ROOT / "NUMERIC_SOLVER_CAPABILITY_REBASE_V1.md"
PACK = ROOT / "pack" / "numeric_solver_capability_rebase_v1"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"


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
        ROADMAP,
        DOC,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "formula_relation_solve_quadratic_v1" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_system_2x2_v1" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_ddn_bridge_v2" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_wasm_cli_parity_v2" / "golden.jsonl",
        ROOT / "pack" / "math_numeric_diff_v1" / "golden.jsonl",
        ROOT / "pack" / "math_numeric_int_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_flow_v1v_closure_v1" / "contract.detjson",
        ROOT / "tests" / "run_relation_solve_ddn_bridge_v2_pack_check.py",
        ROOT / "tests" / "run_formula_relation_solve_quadratic_pack_check.py",
        ROOT / "tests" / "run_relation_solve_system_2x2_pack_check.py",
        ROOT / "tests" / "run_relation_solve_wasm_cli_parity_v2_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_NUMERIC_SOLVER_REBASE_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    common = [
        "bounded",
        "single",
        "2x2",
        "formula_relation_solve_quadratic_v1",
        "relation_solve_system_2x2_v1",
        "relation_solve_ddn_bridge_v2",
        "relation_solve_wasm_cli_parity_v2",
        "math_numeric_diff_v1",
        "math_numeric_int_v1",
        "ODE_TICK_LOOP_LESSON_BASELINE_V1",
        "NUMERIC_ROOT_FINDING_V1",
        "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
        "docs/ssot/**",
    ]
    for path, code in ((ROADMAP, "E_NUMERIC_SOLVER_REBASE_ROADMAP"), (DOC, "E_NUMERIC_SOLVER_REBASE_DOC")):
        rc = require_tokens(path, common, code)
        if rc:
            return rc
    forbidden = [
        "connect_flow_v1w_closure_v1",
        "pack/connect_flow_v1w",
        "tests/run_connect_flow_v1w",
    ]
    for path in (ROADMAP, DOC):
        text = read(path)
        present = [token for token in forbidden if token in text]
        if present:
            return fail("E_NUMERIC_SOLVER_REBASE_FORBIDDEN", f"{path.relative_to(ROOT)} {present}")
    return 0


def check_contract_and_golden() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "numeric_solver_capability_rebase_v1",
        "kind": "numeric_solver_capability_rebase",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "NUMERIC_SOLVER_CAPABILITY_REBASE_V1",
        "current_relation_solve_boundary": "bounded_single_relation_and_exact_2x2_system",
        "generalized_linear_solver_claim": False,
        "nonlinear_solver_claim": False,
        "higher_degree_polynomial_solver_claim": False,
        "ode_solver_claim": False,
        "solver_internal_inequality_claim": False,
        "next_recommended_item": "ODE_TICK_LOOP_LESSON_BASELINE_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_NUMERIC_SOLVER_REBASE_CONTRACT", f"{key}={contract.get(key)!r}")
    evidence = contract.get("evidence_packs")
    required = {
        "formula_relation_solve_quadratic_v1",
        "relation_solve_system_2x2_v1",
        "relation_solve_ddn_bridge_v2",
        "relation_solve_wasm_cli_parity_v2",
        "math_numeric_diff_v1",
        "math_numeric_int_v1",
        "connect_flow_v1v_closure_v1",
    }
    if set(evidence or []) != required:
        return fail("E_NUMERIC_SOLVER_REBASE_EVIDENCE", repr(evidence))

    rows = [json.loads(line) for line in read(PACK / "golden.jsonl").splitlines() if line.strip()]
    if len(rows) != 1:
        return fail("E_NUMERIC_SOLVER_REBASE_GOLDEN_COUNT", repr(len(rows)))
    row = rows[0]
    expected_stdout = [
        "NUMERIC_SOLVER_CAPABILITY_REBASE_V1",
        "numeric solver capability rebase sealed",
        "boundary: bounded single relation + exact 2x2 system",
        "next: ODE_TICK_LOOP_LESSON_BASELINE_V1",
    ]
    if row.get("stdout") != expected_stdout:
        return fail("E_NUMERIC_SOLVER_REBASE_GOLDEN_STDOUT", repr(row.get("stdout")))
    return 0


def check_runtime_boundary() -> int:
    teul = read(ROOT / "tools" / "teul-cli" / "src" / "runtime" / "eval.rs")
    tool = read(ROOT / "tool" / "src" / "ddn_runtime.rs")
    for label, text in (("teul-cli", teul), ("tool", tool)):
        required = [
            "fn eval_relation_solve_result",
            "match relations.len()",
            "1 =>",
            "2 =>",
            "solve_relation_equation",
            "solve_relation_system",
            "relation solve arity",
        ]
        missing = [token for token in required if token not in text]
        if missing:
            return fail("E_NUMERIC_SOLVER_REBASE_RUNTIME_BOUNDARY", f"{label} missing {missing}")
    if "Current dispatch only covers single-equation solve and 2식 2미지수 exact system solve" not in teul:
        return fail("E_NUMERIC_SOLVER_REBASE_RUNTIME_COMMENT", "teul-cli boundary comment missing")
    return 0


def check_numeric_signatures() -> int:
    stdlib = read(ROOT / "lang" / "src" / "stdlib.rs")
    teul = read(ROOT / "tools" / "teul-cli" / "src" / "runtime" / "eval.rs")
    tool = read(ROOT / "tool" / "src" / "ddn_runtime.rs")
    for token in ("미분.중앙차분", "적분.사다리꼴", "적분.오일러", "적분.반암시적오일러", "방정식풀기"):
        if token not in stdlib:
            return fail("E_NUMERIC_SOLVER_REBASE_STDLIB_TOKEN", token)
        if token not in teul:
            return fail("E_NUMERIC_SOLVER_REBASE_TEUL_TOKEN", token)
        if token not in tool:
            return fail("E_NUMERIC_SOLVER_REBASE_TOOL_TOKEN", token)
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "numeric_solver_capability_rebase_v1"],
        ["python", "tests/run_formula_relation_solve_quadratic_pack_check.py"],
        ["python", "tests/run_relation_solve_system_2x2_pack_check.py"],
        ["python", "tests/run_relation_solve_ddn_bridge_v2_pack_check.py"],
        ["python", "tests/run_relation_solve_wasm_cli_parity_v2_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=180)
        if proc.returncode != 0:
            return fail("E_NUMERIC_SOLVER_REBASE_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    required = [
        "NUMERIC_SOLVER_CAPABILITY_REBASE_V1",
        "numeric_solver_capability_rebase_v1",
        "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1",
        "ODE_TICK_LOOP_LESSON_BASELINE_V1",
        "bounded exact relation solver",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in read(DEV_SUMMARY)]
    if missing:
        return fail("E_NUMERIC_SOLVER_REBASE_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_NUMERIC_SOLVER_REBASE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_NUMERIC_SOLVER_REBASE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_golden,
        check_runtime_boundary,
        check_numeric_signatures,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[numeric-solver-capability-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
