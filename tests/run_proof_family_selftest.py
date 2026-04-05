#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/proof_family/README.md")
OPERATION_FAMILY_README = Path("tests/proof_operation_family/README.md")
CERTIFICATE_FAMILY_README = Path("tests/proof_certificate_family/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Bundle Contract",
    "## Stable Transport Contract",
    "`tests/proof_operation_family/README.md`",
    "`tests/proof_certificate_family/README.md`",
    "`python tests/run_proof_operation_family_selftest.py`",
    "`python tests/run_proof_certificate_family_selftest.py`",
    "`python tests/run_proof_family_selftest.py`",
    "`python tests/run_proof_family_contract_selftest.py`",
    "`python tests/run_proof_family_contract_summary_selftest.py`",
    "`python tests/run_proof_family_transport_contract_selftest.py`",
    "`python tests/run_proof_family_transport_contract_summary_selftest.py`",
    "`proof_operation_family_selftest`",
    "`proof_certificate_family_selftest`",
    "`proof_family_selftest`",
    "`proof_family_contract_selftest`",
    "`ddn.ci.proof_family_contract_selftest.progress.v1`",
    "`ddn.ci.proof_family_transport_contract_selftest.progress.v1`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "aggregate status line",
    "final status line",
    "gate result / summary compact",
    "ci_fail_brief / triage",
    "ci_gate_report_index",
    "| proof operation line | `age1 immediate proof -> solver search/check parity -> proof solver family -> proof operation family` |",
    "| proof certificate line | `proof artifact cert bridge -> proof_certificate_v1 family -> proof_certificate family` |",
)
POINTERS = (
    "`tests/proof_family/README.md`",
    "`python tests/run_proof_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-family-selftest] fail: {message}")
    return 1


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def main() -> int:
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_pointers(OPERATION_FAMILY_README)
        ensure_pointers(CERTIFICATE_FAMILY_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"proof_family_selftest"',
                '[py, "tests/run_proof_family_selftest.py"]',
                '"proof_family_contract_selftest"',
                '[py, "tests/run_proof_family_contract_selftest.py"]',
                '"proof_family_contract_summary_selftest"',
                '[py, "tests/run_proof_family_contract_summary_selftest.py"]',
                '"proof_family_transport_contract_selftest"',
                '[py, "tests/run_proof_family_transport_contract_selftest.py"]',
                '"proof_family_transport_contract_summary_selftest"',
                '[py, "tests/run_proof_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-family-selftest] ok lines=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
