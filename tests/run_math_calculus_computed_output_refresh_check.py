#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md"
PACK = ROOT / "pack" / "math_calculus_v1"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"

EXPECTED_STDOUT = [
    "(#ascii) 수식{ 2*x }",
    "(#ascii) 수식{ 2 }",
    "(#ascii) 수식{ y = 2*x }",
    "(#ascii) 수식{ 1/4*x^4 }",
    "(#ascii) 수식{ 1/4*x^4 + C }",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "ROADMAP_V2_DA1_MATH_REBASE_V1.md",
        QUEUE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        PACK / "input_error_no_var.ddn",
        PACK / "input_error_multi_var.ddn",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_MISSING", str(missing))
    return 0


def check_doc() -> int:
    text = read(DOC)
    required = [
        "evidence only",
        "no stdlib surface",
        "computed formula output",
        "Source-form passthrough output is no longer accepted",
        "diff(...)",
        "int(...)",
        "ROADMAP_V2_DA1_MATH_CLOSURE_V1",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_DOC", str(missing))
    return 0


def check_input_surface() -> int:
    text = read(PACK / "input.ddn")
    for token in ["미분하기", "적분하기", "차수=2", "상수포함=참"]:
        if token not in text:
            return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_INPUT", token)
    return 0


def check_golden_computed_output() -> int:
    rows = [json.loads(line) for line in read(PACK / "golden.jsonl").splitlines() if line.strip()]
    if len(rows) != 1:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_GOLDEN_ROWS", str(len(rows)))
    row = rows[0]
    if row.get("stdout") != EXPECTED_STDOUT:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_STDOUT", str(row.get("stdout")))
    if row.get("exit_code") != 0:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_EXIT", str(row.get("exit_code")))
    joined = "\n".join(row["stdout"])
    forbidden = ["diff(", "int("]
    present = [token for token in forbidden if token in joined]
    if present:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_PASSTHROUGH", str(present))
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1",
        "closed by `MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md`",
        "ROADMAP_V2_DA1_MATH_CLOSURE_V1",
        "closed by `ROADMAP_V2_DA1_MATH_CLOSURE_V1.md`",
        "MATH_VECTOR_MINIMUM_FIRST_RUN_V1",
        "math_calculus_v1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_QUEUE", str(missing))
    if "1. `MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1`" in text:
        return fail(
            "E_MATH_CALCULUS_COMPUTED_REFRESH_QUEUE_OPEN",
            "refresh is still listed as the next open item",
        )
    return 0


def run_pack_golden() -> int:
    result = subprocess.run(
        ["python", "tests/run_pack_golden.py", "math_calculus_v1"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_PACK", result.stdout.strip())
    return 0


def check_docs_ssot_clean() -> int:
    result = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_MATH_CALCULUS_COMPUTED_REFRESH_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_input_surface,
        check_golden_computed_output,
        check_queue,
        run_pack_golden,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[math-calculus-computed-output-refresh-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
