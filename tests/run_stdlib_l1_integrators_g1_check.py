#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "stdlib_l1_integrators_g1_v1"


def fail(message: str) -> int:
    print(f"stdlib_l1_integrators_g1_check: FAIL: {message}", file=sys.stderr)
    return 1


def main() -> int:
    required = [PACK / "README.md", PACK / "input.ddn", PACK / "golden.jsonl"]
    missing = [path.relative_to(ROOT).as_posix() for path in required if not path.exists()]
    if missing:
        return fail(f"missing files: {missing}")

    readme = (PACK / "README.md").read_text(encoding="utf-8")
    input_text = (PACK / "input.ddn").read_text(encoding="utf-8")
    golden = (PACK / "golden.jsonl").read_text(encoding="utf-8")
    for token in ["적분.오일러", "적분.반암시적오일러", "보간.선형", "runner_golden"]:
        if token not in readme + input_text:
            return fail(f"missing token: {token}")
    for token in ["euler", "semi_implicit_euler", "linear_interpolation"]:
        if token not in golden:
            return fail(f"golden missing pendulum row token: {token}")

    proc = subprocess.run(
        [sys.executable, "tests/run_pack_golden.py", "stdlib_l1_integrators_g1_v1"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if proc.returncode != 0:
        return fail(proc.stderr.strip() or proc.stdout.strip() or "run_pack_golden failed")

    print("stdlib_l1_integrators_g1_check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
