#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "NUMERIC_ROOT_FINDING_V1.md"
ROADMAP = ROOT / "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md"
PACK = ROOT / "pack" / "numeric_root_finding_bisection_v1"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
LANG_STATUS = ROOT / "docs" / "status" / "LANG_STATUS.md"


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
        ROADMAP,
        ROOT / "ODE_METHOD_COMPARISON_V1.md",
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "input_bad_bracket.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_NUMERIC_ROOT_FINDING_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    rc = require_tokens(
        DOC,
        [
            "NUMERIC_ROOT_FINDING_V1",
            "수치해.이분법",
            "(#ascii) 수식",
            "차림[근, 잔차, 반복횟수",
            "Newton-Raphson",
            "POLYNOMIAL_SOLVE_MINIMUM_V1",
            "docs/ssot/**",
        ],
        "E_NUMERIC_ROOT_FINDING_DOC",
    )
    if rc:
        return rc
    return require_tokens(
        ROADMAP,
        [
            "NUMERIC_ROOT_FINDING_V1",
            "closed by `NUMERIC_ROOT_FINDING_V1.md`",
            "numeric_root_finding_bisection_v1",
            "POLYNOMIAL_SOLVE_MINIMUM_V1",
        ],
        "E_NUMERIC_ROOT_FINDING_ROADMAP",
    )


def check_contract_and_golden() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "numeric_root_finding_bisection_v1",
        "kind": "numeric_root_finding_bisection",
        "closed_by": "NUMERIC_ROOT_FINDING_V1",
        "product_code_change": True,
        "stdlib_surface": "수치해.이분법",
        "formula_contract": "#ascii 수식 + 변수 string",
        "method": "bisection",
        "requires_bracket_sign_change": True,
        "newton_claim": False,
        "polynomial_solver_claim": False,
        "ode_solver_claim": False,
        "inequality_solver_claim": False,
        "next_recommended_item": "POLYNOMIAL_SOLVE_MINIMUM_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_NUMERIC_ROOT_FINDING_CONTRACT", f"{key}={contract.get(key)!r}")

    rows = [json.loads(line) for line in read(PACK / "golden.jsonl").splitlines() if line.strip()]
    if len(rows) != 2:
        return fail("E_NUMERIC_ROOT_FINDING_GOLDEN_COUNT", repr(len(rows)))
    if rows[0].get("stdout") != ["2", "0", "1", "이분법"]:
        return fail("E_NUMERIC_ROOT_FINDING_STDOUT", repr(rows[0].get("stdout")))
    if rows[1].get("expected_error_code") != "E_MATH_DOMAIN":
        return fail("E_NUMERIC_ROOT_FINDING_ERROR_CODE", repr(rows[1]))
    return 0


def check_product_tokens() -> int:
    targets = [
        ROOT / "lang" / "src" / "stdlib.rs",
        ROOT / "tools" / "teul-cli" / "src" / "runtime" / "eval.rs",
        ROOT / "tool" / "src" / "ddn_runtime.rs",
    ]
    for path in targets:
        text = read(path)
        if "수치해.이분법" not in text:
            return fail("E_NUMERIC_ROOT_FINDING_PRODUCT_TOKEN", str(path.relative_to(ROOT)))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "numeric_root_finding_bisection_v1"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=180)
        if proc.returncode != 0:
            return fail("E_NUMERIC_ROOT_FINDING_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_status_docs() -> int:
    for path, tokens, code in [
        (
            DEV_SUMMARY,
            ["NUMERIC_ROOT_FINDING_V1", "numeric_root_finding_bisection_v1", "수치해.이분법", "docs/ssot/** 변경 없음"],
            "E_NUMERIC_ROOT_FINDING_DEV_SUMMARY",
        ),
        (
            LANG_STATUS,
            ["NUMERIC_ROOT_FINDING_V1", "수치해.이분법", "bracketed scalar", "numeric_root_finding_bisection_v1"],
            "E_NUMERIC_ROOT_FINDING_LANG_STATUS",
        ),
    ]:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_NUMERIC_ROOT_FINDING_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_NUMERIC_ROOT_FINDING_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_golden,
        check_product_tokens,
        run_required_gates,
        check_status_docs,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[numeric-root-finding-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
