#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from run_bogae_alias_family_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/bogae_alias_family/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Transport Contract",
    "bundle `checks_text`:",
    "`ddn.ci.bogae_alias_family_contract_selftest.progress.v1`",
    "`bogae_alias_family_contract_selftest`",
    "`bogae_alias_family_contract_summary_selftest`",
    "`python tests/run_bogae_alias_family_contract_selftest.py`",
    "`python tests/run_bogae_alias_family_contract_summary_selftest.py`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "age5 close full-real report",
    "aggregate preview summary",
    "`python tests/run_bogae_shape_alias_contract_selftest.py`",
    "`python tests/run_bogae_alias_family_selftest.py`",
    "`python tests/run_bogae_alias_viewer_family_selftest.py`",
    "`python tests/run_age5_close_combined_report_contract_selftest.py`",
    "`python tests/run_ci_aggregate_age5_child_summary_bogae_alias_family_transport_selftest.py`",
    "`python tests/run_ci_gate_summary_report_check_selftest.py`",
    "`python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`",
)


def fail(msg: str) -> int:
    print(f"[bogae-alias-family-contract-summary-selftest] fail: {msg}")
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
    print("[bogae-alias-family-contract-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
