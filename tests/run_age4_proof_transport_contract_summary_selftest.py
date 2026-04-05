#!/usr/bin/env python
from __future__ import annotations

import sys
from pathlib import Path

from run_age4_proof_transport_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/age4_proof_transport/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Contract",
    "checks_text",
    "`run_age4_proof_transport_contract_selftest.py`",
    "`run_age4_proof_transport_contract_summary_selftest.py`",
    "`age4_proof_transport_contract_selftest`",
    "`age4_proof_transport_contract_summary_selftest`",
    "`ddn.proof_artifact_summary.v1`",
    "`ddn.age4.proof_artifact_report.v1`",
    "aggregate status line",
    "`ci_gate_summary.txt`",
    "`ci_fail_brief.txt`",
    "`ci_fail_triage.detjson`",
    "`ci_gate_report_index`",
    "`age4_proof_ok`",
    "`age4_proof_failed_criteria`",
    "`age4_proof_failed_preview`",
    "`age4_proof_summary_hash`",
    "ddn.ci.age4_proof_transport_contract_selftest.progress.v1",
)


def fail(msg: str) -> int:
    print(f"[age4-proof-transport-summary-selftest] fail: {msg}")
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
    print("[age4-proof-transport-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
