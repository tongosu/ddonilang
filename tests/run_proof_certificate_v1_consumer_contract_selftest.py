#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_consumer_contract/README.md")
PACK_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
SIGNED_PROFILES_README = Path("tests/proof_certificate_v1_signed_emit_profiles/README.md")
VERIFY_BUNDLE_README = Path("tests/proof_certificate_v1_verify_bundle/README.md")
VERIFY_REPORT_README = Path("tests/proof_certificate_v1_verify_report/README.md")
SIGNED_CONTRACT_README = Path("tests/proof_certificate_v1_signed_contract/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion/README.md")
FAMILY_README = Path("tests/proof_certificate_v1_family/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age4_proof_detjson_smoke_v1/README.md`",
    "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
    "`tests/proof_certificate_v1_verify_bundle/README.md`",
    "`tests/proof_certificate_v1_verify_report/README.md`",
    "`tests/proof_certificate_v1_verify_report_digest_contract/README.md`",
    "`tests/proof_certificate_v1_signed_contract/README.md`",
    "`tests/proof_certificate_v1_family/README.md`",
    "`python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`",
    "`python tests/run_proof_certificate_v1_verify_bundle_selftest.py`",
    "`python tests/run_proof_certificate_v1_verify_report_selftest.py`",
    "`python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_consumer_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_selftest.py`",
    "`proof_certificate_v1_consumer_contract_selftest`",
    "| signed emit profiles | `clean/abort` signed bundle parity | 두 profile 모두 같은 signed output set을 쓴다 |",
    "| verify bundle | `cert verify-proof-certificate --in <bundle>` | signed bundle의 cert/proof/runtime/source-proof parity를 직접 검증한다 |",
    "| verify report | `ddn.proof_certificate_v1.verify_report.v1` | verify 결과를 저장 가능한 consumer artifact로 남긴다 |",
    "| verify report digest contract | digest/signature parity summary | verify report가 proof digest/cert signature 축을 빠짐없이 싣는다는 상위 contract를 고정한다 |",
)
POINTERS = (
    "`tests/proof_certificate_v1_consumer_contract/README.md`",
    "`python tests/run_proof_certificate_v1_consumer_contract_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-consumer-contract-selftest] fail: {message}")
    return 1


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def ensure_snippets(path: Path, snippets: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for snippet in snippets:
        if snippet not in text:
            raise ValueError(f"missing snippet in {path}: {snippet}")


def main() -> int:
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_pointers(PACK_README)
        ensure_pointers(SIGNED_PROFILES_README)
        ensure_pointers(VERIFY_BUNDLE_README)
        ensure_pointers(VERIFY_REPORT_README)
        ensure_pointers(SIGNED_CONTRACT_README)
        ensure_pointers(PROMOTION_README)
        ensure_pointers(FAMILY_README)
        ensure_snippets(
            SIGNED_PROFILES_README,
            (
                "`tests/proof_certificate_v1_verify_bundle/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
                "`python tests/run_proof_certificate_v1_verify_bundle_selftest.py`",
            ),
        )
        ensure_snippets(
            VERIFY_BUNDLE_README,
            (
                "`tests/proof_certificate_v1_verify_report/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
                "`python tests/run_proof_certificate_v1_verify_report_selftest.py`",
            ),
        )
        ensure_snippets(
            VERIFY_REPORT_README,
            (
                "`ddn.proof_certificate_v1.verify_report.v1`",
                "`tests/proof_certificate_v1_verify_bundle/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
                "`python tests/run_proof_certificate_v1_verify_report_selftest.py`",
            ),
        )
        ensure_snippets(
            SIGNED_CONTRACT_README,
            (
                "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
                "`tests/proof_certificate_v1_verify_bundle/README.md`",
                "`tests/proof_certificate_v1_verify_report/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
            ),
        )
        ensure_snippets(
            PROMOTION_README,
            (
                "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
                "`tests/proof_certificate_v1_verify_bundle/README.md`",
                "`tests/proof_certificate_v1_verify_report/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
            ),
        )
        ensure_snippets(
            SANITY_GATE,
            (
                '"proof_certificate_v1_consumer_contract_selftest"',
                '[py, "tests/run_proof_certificate_v1_consumer_contract_selftest.py"]',
                '"proof_certificate_v1_family_selftest"',
                '[py, "tests/run_proof_certificate_v1_family_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-consumer-contract-selftest] ok chain=3 docs=5")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
