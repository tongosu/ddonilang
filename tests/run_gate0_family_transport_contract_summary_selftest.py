#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from run_gate0_family_transport_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/gate0_family/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Transport Contract",
    "transport bundle `checks_text`:",
    "`ddn.ci.gate0_family_transport_contract_selftest.progress.v1`",
    "`gate0_family_transport_contract_selftest`",
    "`gate0_family_transport_contract_summary_selftest`",
    "`python tests/run_gate0_family_transport_contract_selftest.py`",
    "`python tests/run_gate0_family_transport_contract_summary_selftest.py`",
    "`python tests/run_ci_aggregate_age5_child_summary_gate0_family_transport_selftest.py`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
)


def fail(msg: str) -> int:
    print(f"[gate0-family-transport-contract-summary-selftest] fail: {msg}")
    return 1


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in REQUIRED_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    if CHECKS_TEXT not in text:
        return fail(f"missing checks_text csv: {CHECKS_TEXT}")
    print("[gate0-family-transport-contract-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
