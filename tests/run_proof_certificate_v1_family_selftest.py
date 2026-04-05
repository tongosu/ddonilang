#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_family/README.md")
SIGNED_CONTRACT_README = Path("tests/proof_certificate_v1_signed_contract/README.md")
CONSUMER_CONTRACT_README = Path("tests/proof_certificate_v1_consumer_contract/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion/README.md")
PACK_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/proof_certificate_v1_signed_contract/README.md`",
    "`tests/proof_certificate_v1_consumer_contract/README.md`",
    "`tests/proof_certificate_v1_promotion/README.md`",
    "`pack/age4_proof_detjson_smoke_v1/README.md`",
    "`python tests/run_proof_certificate_v1_signed_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_consumer_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_promotion_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_contract_summary_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_transport_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_transport_contract_summary_selftest.py`",
    "`proof_certificate_v1_family_selftest`",
    "`proof_certificate_v1_family_contract_selftest`",
    "`proof_certificate_v1_family_transport_contract_selftest`",
    "| signed line | `runtime emit -> signed emit -> signed emit profiles -> signed contract` |",
    "| consumer line | `signed emit profiles -> verify bundle -> verify report -> verify report digest contract -> consumer transport -> consumer contract` |",
    "| promotion line | `draft contract -> flat schema candidate -> flat schema split -> promotion` |",
)
POINTERS = (
    "`tests/proof_certificate_v1_family/README.md`",
    "`python tests/run_proof_certificate_v1_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-family-selftest] fail: {message}")
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
        ensure_pointers(SIGNED_CONTRACT_README)
        ensure_pointers(CONSUMER_CONTRACT_README)
        ensure_pointers(PROMOTION_README)
        ensure_pointers(PACK_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"proof_certificate_v1_family_selftest"',
                '[py, "tests/run_proof_certificate_v1_family_selftest.py"]',
                '"proof_certificate_v1_family_contract_selftest"',
                '[py, "tests/run_proof_certificate_v1_family_contract_selftest.py"]',
                '"proof_certificate_v1_family_contract_summary_selftest"',
                '[py, "tests/run_proof_certificate_v1_family_contract_summary_selftest.py"]',
                '"proof_certificate_v1_family_transport_contract_selftest"',
                '[py, "tests/run_proof_certificate_v1_family_transport_contract_selftest.py"]',
                '"proof_certificate_v1_family_transport_contract_summary_selftest"',
                '[py, "tests/run_proof_certificate_v1_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-family-selftest] ok lines=3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
