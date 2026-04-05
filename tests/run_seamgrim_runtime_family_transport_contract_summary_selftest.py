#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from run_seamgrim_runtime_family_transport_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/seamgrim_runtime_family/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Transport Contract",
    "transport bundle `checks_text`:",
    "`ddn.ci.seamgrim_runtime_family_transport_contract_selftest.progress.v1`",
    "`seamgrim_runtime_family_transport_contract_selftest`",
    "`seamgrim_runtime_family_transport_contract_summary_selftest`",
    "`python tests/run_seamgrim_runtime_family_transport_contract_selftest.py`",
    "`python tests/run_seamgrim_runtime_family_transport_contract_summary_selftest.py`",
    "ci gate stdout",
    "*.progress.detjson",
)


def fail(msg: str) -> int:
    print(f"[seamgrim-runtime-family-transport-contract-summary-selftest] fail: {msg}")
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
    print("[seamgrim-runtime-family-transport-contract-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

