#!/usr/bin/env python
from __future__ import annotations

import sys
from pathlib import Path

from run_age1_immediate_proof_operation_contract_selftest import CHECKS_TEXT


README_PATH = Path("tests/age1_immediate_proof_operation/README.md")
REQUIRED_SNIPPETS = (
    "## Stable Contract",
    "checks_text",
    "`run_age1_immediate_proof_operation_matrix_selftest.py`",
    "`run_age1_immediate_proof_operation_contract_selftest.py`",
    "`age1_immediate_proof_operation_matrix_selftest`",
    "`age1_immediate_proof_operation_contract_selftest`",
    "`proof_check`",
    "`exists_unique`",
    "`check`",
    "`counterexample`",
    "`solve`",
    "`tests/proof_solver_operation_family/README.md`",
    "`tests/proof_operation_family/README.md`",
    "ddn.ci.age1_immediate_proof_operation_contract_selftest.progress.v1",
)


def fail(msg: str) -> int:
    print(f"[age1-immediate-proof-operation-summary-selftest] fail: {msg}")
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
    print("[age1-immediate-proof-operation-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
