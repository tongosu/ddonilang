#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from run_proof_family_transport_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/proof_family/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Transport Contract",
    "transport bundle `checks_text`:",
    "`ddn.ci.proof_family_transport_contract_selftest.progress.v1`",
    "`proof_family_transport_contract_selftest`",
    "`proof_family_transport_contract_summary_selftest`",
    "`python tests/run_proof_family_transport_contract_selftest.py`",
    "`python tests/run_proof_family_transport_contract_summary_selftest.py`",
    "`python tests/run_ci_aggregate_age5_child_summary_proof_family_transport_selftest.py`",
    "`python tests/run_ci_aggregate_status_line_selftest.py`",
    "`python tests/run_ci_gate_final_status_line_selftest.py`",
    "`python tests/run_ci_gate_result_check_selftest.py`",
    "`python tests/run_ci_gate_outputs_consistency_check_selftest.py`",
    "`python tests/run_ci_gate_summary_line_check_selftest.py`",
    "`python tests/run_ci_final_line_emitter_check.py`",
    "`python tests/run_ci_gate_report_index_check_selftest.py`",
    "aggregate status line",
    "final status line",
    "gate result / summary compact",
    "ci_fail_brief / triage",
    "ci_gate_report_index",
)


def fail(msg: str) -> int:
    print(f"[proof-family-transport-contract-summary-selftest] fail: {msg}")
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
    print("[proof-family-transport-contract-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
