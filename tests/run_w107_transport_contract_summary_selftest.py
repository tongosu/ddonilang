#!/usr/bin/env python
from __future__ import annotations

import sys
from pathlib import Path

from run_w107_progress_contract_selftest import CHECKS_TEXT


README_PATH = Path("tools/teul-cli/tests/golden/W107/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Contract",
    "checks_text",
    "ci_gate_summary_line",
    "aggregate preview summary",
    "aggregate status line",
    "final status line",
    "ci_gate_result",
    "ci_fail_brief.txt",
    "ci_fail_triage.detjson",
    "ci_gate_report_index",
    "`w107_progress_contract_selftest`",
    "`ci_gate_summary_line_check_selftest`",
    "`age5_full_real_w107_progress_contract_selftest_*`",
    "`age5_w107_contract_checks_text`",
)
ALTERNATIVE_SNIPPET_GROUPS = (
    ("`w107_golden_index_selfcheck`", "`w107_golden_index_selftest`"),
)


def fail(msg: str) -> int:
    print(f"[w107-transport-contract-summary-selftest] fail: {msg}")
    return 1


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in REQUIRED_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    for group in ALTERNATIVE_SNIPPET_GROUPS:
        if not any(token in text for token in group):
            return fail(f"missing snippet group: {', '.join(group)}")
    if CHECKS_TEXT not in text:
        return fail(f"missing checks_text csv: {CHECKS_TEXT}")
    print("[w107-transport-contract-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
