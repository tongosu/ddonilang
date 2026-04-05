#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from run_proof_certificate_v1_consumer_transport_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/proof_certificate_v1_consumer_contract/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Transport Contract",
    "progress schema",
    "`ddn.ci.proof_certificate_v1_consumer_transport_contract_selftest.progress.v1`",
    "`proof_certificate_v1_consumer_transport_contract_selftest`",
    "`proof_certificate_v1_consumer_transport_contract_summary_selftest`",
    "`tests/proof_certificate_v1_verify_report_digest_contract/README.md`",
    "`python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_consumer_transport_contract_summary_selftest.py`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
)


def fail(msg: str) -> int:
    print(f"[proof-certificate-v1-consumer-transport-contract-summary-selftest] fail: {msg}")
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
    print("[proof-certificate-v1-consumer-transport-contract-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
