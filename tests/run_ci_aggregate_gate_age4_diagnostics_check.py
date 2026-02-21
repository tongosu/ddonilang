#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "age4_close",
    "tests/run_age4_close.py",
    "tools/scripts/print_age4_close_digest.py",
    "--require-age4",
    "--age4-report",
    "[ci-gate-summary] age4_status=",
    "age4_close_report.detjson",
    "age4_close_pack_report.detjson",
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
        print("aggregate gate age4 diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci aggregate gate age4 diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
