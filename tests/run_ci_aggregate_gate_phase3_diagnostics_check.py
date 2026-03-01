#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "seamgrim_phase3_cleanup_base_name",
    "seamgrim_phase3_cleanup_report",
    "--phase3-cleanup-json-out",
    "seamgrim_phase3_cleanup",
    "[ci-gate-summary] seamgrim_phase3_cleanup=",
    "seamgrim_phase3_cleanup_gate_report.detjson",
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
        print("aggregate gate phase3 diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci aggregate gate phase3 diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

