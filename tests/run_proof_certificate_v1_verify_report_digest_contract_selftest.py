#!/usr/bin/env python
from __future__ import annotations

import json
import os
from datetime import datetime, timezone
from pathlib import Path


SCRIPT_TAG = "proof-certificate-v1-verify-report-digest-contract-selftest"
PROGRESS_ENV_KEY = "DDN_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_JSON"
README_PATH = Path("tests/proof_certificate_v1_verify_report_digest_contract/README.md")
VERIFY_REPORT_README = Path("tests/proof_certificate_v1_verify_report/README.md")
CONSUMER_CONTRACT_README = Path("tests/proof_certificate_v1_consumer_contract/README.md")
SIGNED_CONTRACT_README = Path("tests/proof_certificate_v1_signed_contract/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion/README.md")
VERIFY_REPORT_SELFTEST = Path("tests/run_proof_certificate_v1_verify_report_selftest.py")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

DIGEST_FIELDS = (
    "`proof_subject_hash`",
    "`canonical_body_hash`",
    "`proof_runtime_hash`",
    "`solver_translation_hash`",
    "`state_hash`",
    "`trace_hash`",
    "`cert_signature`",
)

README_SNIPPETS = (
    "## Stable Contract",
    "`tools/teul-cli/src/cli/cert.rs`",
    "`tests/proof_certificate_v1_verify_report/README.md`",
    "`tests/proof_certificate_v1_consumer_contract/README.md`",
    "`tests/proof_certificate_v1_signed_contract/README.md`",
    "`tests/proof_certificate_v1_promotion/README.md`",
    "`python tests/run_proof_certificate_v1_verify_report_selftest.py`",
    "`python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`",
    "`proof_certificate_v1_verify_report_digest_contract_selftest`",
    "## Digest Surface",
    "`ddn.proof_certificate_v1.verify_report.v1`",
    *DIGEST_FIELDS,
)

POINTERS = (
    "`tests/proof_certificate_v1_verify_report_digest_contract/README.md`",
    "`python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`",
)


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_probe: str,
    last_completed_probe: str,
    completed_checks: int,
    total_checks: int,
    checks_text: str,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.proof_certificate_v1_verify_report_digest_contract_selftest.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": "-",
        "last_completed_case": "-",
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "completed_checks": int(completed_checks),
        "total_checks": int(total_checks),
        "checks_text": str(checks_text).strip() or "-",
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def fail(message: str) -> int:
    print(f"[{SCRIPT_TAG}] fail: {message}")
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
    checks_text = "verify_report_digest_contract"
    progress_path = os.environ.get(PROGRESS_ENV_KEY, "")
    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe="readme_and_field_contract",
        last_completed_probe="-",
        completed_checks=0,
        total_checks=1,
        checks_text=checks_text,
    )
    try:
        ensure_snippets(README_PATH, README_SNIPPETS)
        ensure_pointers(VERIFY_REPORT_README)
        ensure_pointers(CONSUMER_CONTRACT_README)
        ensure_pointers(SIGNED_CONTRACT_README)
        ensure_pointers(PROMOTION_README)
        ensure_snippets(VERIFY_REPORT_README, DIGEST_FIELDS)
        ensure_snippets(
            VERIFY_REPORT_SELFTEST,
            (
                '"proof_subject_hash"',
                '"canonical_body_hash"',
                '"proof_runtime_hash"',
                '"solver_translation_hash"',
                '"state_hash"',
                '"trace_hash"',
                '"cert_signature"',
            ),
        )
        ensure_snippets(
            SANITY_GATE,
            (
                '"proof_certificate_v1_verify_report_digest_contract_selftest"',
                '[py, "tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py"]',
            ),
        )
    except ValueError as exc:
        write_progress_snapshot(
            progress_path,
            status="failed",
            current_probe="readme_and_field_contract",
            last_completed_probe="-",
            completed_checks=0,
            total_checks=1,
            checks_text=checks_text,
        )
        return fail(str(exc))

    write_progress_snapshot(
        progress_path,
        status="completed",
        current_probe="-",
        last_completed_probe="readme_and_field_contract",
        completed_checks=1,
        total_checks=1,
        checks_text=checks_text,
    )
    print(f"[{SCRIPT_TAG}] ok fields=7 docs=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
