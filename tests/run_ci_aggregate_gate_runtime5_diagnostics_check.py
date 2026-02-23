#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "seamgrim_5min_checklist_base_name",
    "seamgrim_5min_checklist_report",
    "--skip-5min-checklist",
    "include_5min_checklist = not bool(args.skip_5min_checklist)",
    "--checklist-json-out",
    "append_runtime_5min_checklist_summary_lines(",
    "[ci-gate-summary] seamgrim_5min_checklist=",
    "[ci-gate-summary] seamgrim_runtime_5min_rewrite_motion_projectile=",
    "seamgrim_5min_checklist_report.detjson",
    "ci_aggregate_gate_runtime5_diagnostics_check",
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
        print("aggregate gate runtime5 diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci aggregate gate runtime5 diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
