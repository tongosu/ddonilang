#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from run_seamgrim_stack_family_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/seamgrim_stack_family/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Bundle Contract",
    "bundle `checks_text`:",
    "`ddn.ci.seamgrim_stack_family_contract_selftest.progress.v1`",
    "`python tests/run_seamgrim_stack_family_contract_selftest.py`",
    "`python tests/run_seamgrim_stack_family_contract_summary_selftest.py`",
    "ci gate stdout",
    "*.progress.detjson",
)


def fail(msg: str) -> int:
    print(f"[seamgrim-stack-family-contract-summary-selftest] fail: {msg}")
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
    print("[seamgrim-stack-family-contract-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
