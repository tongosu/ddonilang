#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "pipeline_emit_flags_check",
    "tests/run_ci_pipeline_emit_flags_check.py",
    "pipeline_emit_flags_selftest",
    "tests/run_ci_pipeline_emit_flags_check_selftest.py",
    "E_CI_SANITY_PIPELINE_FLAGS_SELFTEST_FAIL",
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    target = root / "tests" / "run_ci_sanity_gate.py"
    if not target.exists():
        print(f"missing target: {target}")
        return 1
    text = target.read_text(encoding="utf-8")

    missing = [token for token in REQUIRED_TOKENS if token not in text]
    if missing:
        print("ci sanity gate diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci sanity gate diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
