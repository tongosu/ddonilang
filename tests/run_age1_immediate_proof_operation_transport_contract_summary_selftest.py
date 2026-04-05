#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from run_age1_immediate_proof_operation_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/age1_immediate_proof_operation/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Transport Contract",
    "checks_text",
    "ci_gate_summary_line",
    "aggregate preview summary",
    "aggregate status line",
    "final status line",
    "ci_gate_result",
    "ci_fail_brief.txt",
    "ci_fail_triage.detjson",
    "ci_gate_report_index",
    "`age1_immediate_proof_operation_contract_selftest`",
    "`age1_immediate_proof_operation_contract_summary_selftest`",
    "`age1_immediate_proof_operation_transport_contract_summary_selftest`",
    "`ci_gate_summary_line_check_selftest`",
    "`age5_full_real_age1_immediate_proof_operation_contract_selftest_*`",
    "`age5_age1_immediate_proof_operation_contract_checks_text`",
)


def fail(msg: str) -> int:
    print(f"[age1-immediate-proof-operation-transport-summary-selftest] fail: {msg}")
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
    print("[age1-immediate-proof-operation-transport-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
