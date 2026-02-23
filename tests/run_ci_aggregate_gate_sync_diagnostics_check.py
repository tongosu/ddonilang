#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "tests/run_ci_sync_readiness_check_selftest.py",
    "check_ci_sync_readiness_selftest",
    "ci_sync_readiness_selftest",
    "check_ci_aggregate_gate_sync_diagnostics",
    "ci_aggregate_gate_sync_diagnostics_check",
    "tests/run_ci_aggregate_gate_sync_diagnostics_check.py",
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    target = root / "tests" / "run_ci_aggregate_gate.py"
    if not target.exists():
        print(f"missing target: {target}")
        return 1
    text = target.read_text(encoding="utf-8")

    missing = [token for token in REQUIRED_TOKENS if token not in text]
    if missing:
        print("aggregate gate sync diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci aggregate gate sync diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
