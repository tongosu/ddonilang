#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "age5_close",
    "tests/run_age5_close.py",
    "tools/scripts/print_age5_close_digest.py",
    "--require-age5",
    "--age5-report",
    "ci_pack_golden_overlay_compare_selftest",
    "tests/run_pack_golden_overlay_compare_selftest.py",
    "[ci-gate-summary] age5_status=",
    "age5_close_report.detjson",
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
        print("aggregate gate age5 diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci aggregate gate age5 diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
