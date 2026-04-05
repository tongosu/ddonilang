#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_signed_contract/README.md")
PACK_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
RUNTIME_EMIT_README = Path("tests/proof_certificate_v1_runtime_emit/README.md")
SIGNED_EMIT_README = Path("tests/proof_certificate_v1_signed_emit/README.md")
SIGNED_EMIT_PROFILES_README = Path("tests/proof_certificate_v1_signed_emit_profiles/README.md")
VERIFY_BUNDLE_README = Path("tests/proof_certificate_v1_verify_bundle/README.md")
VERIFY_REPORT_README = Path("tests/proof_certificate_v1_verify_report/README.md")
CONSUMER_CONTRACT_README = Path("tests/proof_certificate_v1_consumer_contract/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion/README.md")
FAMILY_README = Path("tests/proof_certificate_v1_family/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age4_proof_detjson_smoke_v1/README.md`",
    "`tests/proof_certificate_v1_runtime_emit/README.md`",
    "`tests/proof_certificate_v1_signed_emit/README.md`",
    "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
    "`tests/proof_certificate_v1_verify_bundle/README.md`",
    "`tests/proof_certificate_v1_verify_report/README.md`",
    "`tests/proof_certificate_v1_consumer_contract/README.md`",
    "`tests/proof_certificate_v1_promotion/README.md`",
    "`tests/proof_certificate_v1_family/README.md`",
    "`python tests/run_proof_certificate_v1_runtime_emit_selftest.py`",
    "`python tests/run_proof_certificate_v1_signed_emit_selftest.py`",
    "`python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`",
    "`python tests/run_proof_certificate_v1_verify_bundle_selftest.py`",
    "`python tests/run_proof_certificate_v1_verify_report_selftest.py`",
    "`python tests/run_proof_certificate_v1_consumer_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_promotion_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_selftest.py`",
    "`python tests/run_proof_certificate_v1_signed_contract_selftest.py`",
    "`proof_certificate_v1_signed_contract_selftest`",
    "`ddn.proof_certificate_v1_runtime_candidate.v1` / `ddn.proof_certificate_v1_runtime_draft_artifact.v1`",
    "`ddn.cert_manifest.v1` / `ddn.proof_certificate_v1.v1`",
)
POINTERS = (
    "`tests/proof_certificate_v1_signed_contract/README.md`",
    "`python tests/run_proof_certificate_v1_signed_contract_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-signed-contract-selftest] fail: {message}")
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
        ensure_pointers(RUNTIME_EMIT_README)
        ensure_pointers(SIGNED_EMIT_README)
        ensure_pointers(SIGNED_EMIT_PROFILES_README)
        ensure_pointers(VERIFY_BUNDLE_README)
        ensure_pointers(VERIFY_REPORT_README)
        ensure_pointers(CONSUMER_CONTRACT_README)
        ensure_pointers(PROMOTION_README)
        ensure_pointers(FAMILY_README)
        ensure_snippets(
            RUNTIME_EMIT_README,
            (
                "`ddn.proof_certificate_v1_runtime_candidate.v1`",
                "`ddn.proof_certificate_v1_runtime_draft_artifact.v1`",
                "`tests/proof_certificate_v1_signed_emit/README.md`",
                "`tests/proof_certificate_v1_promotion/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
            ),
        )
        ensure_snippets(
            SIGNED_EMIT_README,
            (
                "`ddn.cert_manifest.v1`",
                "`ddn.proof_certificate_v1.v1`",
                "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
                "`tests/proof_certificate_v1_runtime_emit/README.md`",
                "`tests/proof_certificate_v1_promotion/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
            ),
        )
        ensure_snippets(
            SIGNED_EMIT_PROFILES_README,
            (
                "`pack/age4_proof_detjson_smoke_v1/input.ddn`",
                "`pack/age4_proof_detjson_smoke_v1/input_abort.ddn`",
                "`tests/proof_certificate_v1_signed_emit/README.md`",
                "`tests/proof_certificate_v1_signed_contract/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
                "`python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`",
            ),
        )
        ensure_snippets(
            VERIFY_BUNDLE_README,
            (
                "`tools/teul-cli/src/cli/cert.rs`",
                "`tools/teul-cli/src/main.rs`",
                "`pack/age4_proof_detjson_smoke_v1/input.ddn`",
                "`pack/age4_proof_detjson_smoke_v1/input_abort.ddn`",
                "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
                "`tests/proof_certificate_v1_signed_contract/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
                "`python tests/run_proof_certificate_v1_verify_bundle_selftest.py`",
            ),
        )
        ensure_snippets(
            VERIFY_REPORT_README,
            (
                "`tools/teul-cli/src/cli/cert.rs`",
                "`tools/teul-cli/src/main.rs`",
                "`tests/proof_certificate_v1_verify_bundle/README.md`",
                "`tests/proof_certificate_v1_signed_contract/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
                "`python tests/run_proof_certificate_v1_verify_report_selftest.py`",
            ),
        )
        ensure_snippets(
            CONSUMER_CONTRACT_README,
            (
                "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
                "`tests/proof_certificate_v1_verify_bundle/README.md`",
                "`tests/proof_certificate_v1_verify_report/README.md`",
                "`tests/proof_certificate_v1_signed_contract/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
                "`python tests/run_proof_certificate_v1_consumer_contract_selftest.py`",
            ),
        )
        ensure_snippets(
            PROMOTION_README,
            (
                "`tests/proof_certificate_v1_runtime_emit/README.md`",
                "`tests/proof_certificate_v1_signed_emit/README.md`",
                "`tests/proof_certificate_v1_signed_emit_profiles/README.md`",
                "`tests/proof_certificate_v1_verify_bundle/README.md`",
                "`tests/proof_certificate_v1_verify_report/README.md`",
                "`tests/proof_certificate_v1_consumer_contract/README.md`",
                "`tests/proof_certificate_v1_family/README.md`",
                "`python tests/run_proof_certificate_v1_promotion_selftest.py`",
            ),
        )
        ensure_snippets(
            SANITY_GATE,
            (
                '"proof_certificate_v1_signed_contract_selftest"',
                '[py, "tests/run_proof_certificate_v1_signed_contract_selftest.py"]',
                '"proof_certificate_v1_family_selftest"',
                '[py, "tests/run_proof_certificate_v1_family_selftest.py"]',
                '"proof_certificate_v1_signed_emit_profile_selftest"',
                '[py, "tests/run_proof_certificate_v1_signed_emit_profile_selftest.py"]',
                '"proof_certificate_v1_verify_bundle_selftest"',
                '[py, "tests/run_proof_certificate_v1_verify_bundle_selftest.py"]',
                '"proof_certificate_v1_verify_report_selftest"',
                '[py, "tests/run_proof_certificate_v1_verify_report_selftest.py"]',
                '"proof_certificate_v1_consumer_contract_selftest"',
                '[py, "tests/run_proof_certificate_v1_consumer_contract_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-signed-contract-selftest] ok chain=3 docs=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
