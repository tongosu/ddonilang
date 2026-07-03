#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ODE_METHOD_COMPARISON_V1.md"
PREV = ROOT / "ODE_TICK_LOOP_LESSON_BASELINE_V1.md"
ROADMAP = ROOT / "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md"
PACK = ROOT / "pack" / "ode_method_comparison_v1"
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
        DOC,
        PREV,
        ROADMAP,
        ROOT / "pack" / "ode_tick_loop_lesson_baseline_v1" / "golden.jsonl",
        ROOT / "tests" / "run_ode_tick_loop_lesson_baseline_check.py",
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ODE_METHOD_COMPARISON_MISSING", str(missing))
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
            "ODE_METHOD_COMPARISON_V1",
            "적분.오일러",
            "적분.반암시적오일러",
            "x*x + v*v",
            "No symbolic ODE solver",
            "NUMERIC_ROOT_FINDING_V1",
            "docs/ssot/**",
        ],
        "E_ODE_METHOD_COMPARISON_DOC",
    )
    if rc:
        return rc
    return require_tokens(
        ROADMAP,
        [
            "ODE_METHOD_COMPARISON_V1",
            "closed by `ODE_METHOD_COMPARISON_V1.md`",
            "ode_method_comparison_v1",
            "NUMERIC_ROOT_FINDING_V1",
        ],
        "E_ODE_METHOD_COMPARISON_ROADMAP",
    )


def check_contract_and_golden() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "ode_method_comparison_v1",
        "kind": "ode_method_comparison",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "ODE_METHOD_COMPARISON_V1",
        "based_on": "ODE_TICK_LOOP_LESSON_BASELINE_V1",
        "comparison_sample": "harmonic_oscillator_one_tick",
        "energy_indicator": "x*x+v*v",
        "ode_solver_claim": False,
        "automatic_ode_detection_claim": False,
        "rk4_claim": False,
        "adaptive_step_claim": False,
        "next_recommended_item": "NUMERIC_ROOT_FINDING_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_ODE_METHOD_COMPARISON_CONTRACT", f"{key}={contract.get(key)!r}")
    if contract.get("uses_existing_helpers") != ["적분.오일러", "적분.반암시적오일러"]:
        return fail("E_ODE_METHOD_COMPARISON_HELPERS", repr(contract.get("uses_existing_helpers")))

    rows = [json.loads(line) for line in read(PACK / "golden.jsonl").splitlines() if line.strip()]
    if len(rows) != 1:
        return fail("E_ODE_METHOD_COMPARISON_GOLDEN_COUNT", repr(len(rows)))
    expected_stdout = [
        "explicit",
        "1",
        "-0.25",
        "1.0625",
        "semi_implicit",
        "0.9375",
        "-0.25",
        "0.94140625",
    ]
    if rows[0].get("stdout") != expected_stdout:
        return fail("E_ODE_METHOD_COMPARISON_STDOUT", repr(rows[0].get("stdout")))
    return 0


def check_input_surface() -> int:
    text = read(PACK / "input.ddn")
    required = ["적분.오일러", "적분.반암시적오일러", "명시에너지1", "반에너지1", "보여주기"]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ODE_METHOD_COMPARISON_INPUT", str(missing))
    forbidden = ["RK4", "수치해", "방정식풀기"]
    present = [token for token in forbidden if token in text]
    if present:
        return fail("E_ODE_METHOD_COMPARISON_FORBIDDEN_INPUT", str(present))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "ode_method_comparison_v1"],
        ["python", "tests/run_ode_tick_loop_lesson_baseline_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=180)
        if proc.returncode != 0:
            return fail("E_ODE_METHOD_COMPARISON_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    required = [
        "ODE_METHOD_COMPARISON_V1",
        "ode_method_comparison_v1",
        "explicit Euler",
        "semi-implicit Euler",
        "NUMERIC_ROOT_FINDING_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in read(DEV_SUMMARY)]
    if missing:
        return fail("E_ODE_METHOD_COMPARISON_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_ODE_METHOD_COMPARISON_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_ODE_METHOD_COMPARISON_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_golden,
        check_input_surface,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[ode-method-comparison-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

