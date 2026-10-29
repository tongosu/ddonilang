#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


README_PATH = Path("tests/proof_certificate_family/README.md")
ARTIFACT_CERT_README = Path("tests/proof_artifact_certificate_contract/README.md")
V1_FAMILY_README = Path("tests/proof_certificate_v1_family/README.md")
AGE4_PROOF_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
AGE4_BRIDGE_README = Path("pack/age4_proof_artifact_cert_subject_v1/README.md")
W95_CERT_README = Path("pack/gogae9_w95_cert/README.md")
SANITY_GATE = Path("tests/run_ci_sanity_gate.py")

README_SNIPPETS = (
    "## Stable Contract",
    "## Stable Transport Contract",
    "`tests/proof_artifact_certificate_contract/README.md`",
    "`tests/proof_certificate_v1_family/README.md`",
    "`pack/age4_proof_detjson_smoke_v1/README.md`",
    "`pack/age4_proof_artifact_cert_subject_v1/README.md`",
    "`pack/gogae9_w95_cert/README.md`",
    "`python tests/run_proof_artifact_certificate_contract_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_selftest.py`",
    "`python tests/run_proof_certificate_family_selftest.py`",
    "`python tests/run_proof_certificate_family_contract_selftest.py`",
    "`python tests/run_proof_certificate_family_contract_summary_selftest.py`",
    "`python tests/run_proof_certificate_family_transport_contract_selftest.py`",
    "`python tests/run_proof_certificate_family_transport_contract_summary_selftest.py`",
    "`tests/proof_family/README.md`",
    "`python tests/run_proof_family_selftest.py`",
    "`proof_artifact_certificate_contract_selftest`",
    "`proof_certificate_v1_family_selftest`",
    "`proof_certificate_family_selftest`",
    "`proof_certificate_family_contract_selftest`",
    "`proof_certificate_family_contract_summary_selftest`",
    "`ddn.ci.proof_certificate_family_contract_selftest.progress.v1`",
    "`ddn.ci.proof_certificate_family_transport_contract_selftest.progress.v1`",
    "ci_sanity_gate stdout",
    "*.progress.detjson",
    "age5 close full-real report",
    "aggregate preview summary",
    "`python tests/run_age5_close_combined_report_contract_selftest.py`",
    "`python tests/run_ci_aggregate_age5_child_summary_proof_certificate_family_transport_selftest.py`",
    "`python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`",
    "aggregate status line",
    "final status line",
    "gate result/summary compact",
    "ci_fail_brief/triage",
    "ci_gate_report_index",
    "`python tests/run_ci_aggregate_status_line_selftest.py`",
    "`python tests/run_ci_gate_final_status_line_selftest.py`",
    "`python tests/run_ci_gate_result_check_selftest.py`",
    "`python tests/run_ci_gate_outputs_consistency_check_selftest.py`",
    "`python tests/run_ci_gate_summary_line_check_selftest.py`",
    "`python tests/run_ci_final_line_emitter_check.py`",
    "`python tests/run_ci_gate_report_index_check_selftest.py`",
    "| artifact/cert bridge line | `proof artifact emit -> proof artifact cert bridge -> w95 cert` |",
    "| proof_certificate_v1 line | `signed contract -> consumer contract -> promotion -> proof_certificate_v1 family` |",
)
POINTERS = (
    "`tests/proof_certificate_family/README.md`",
    "`python tests/run_proof_certificate_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-family-selftest] fail: {message}")
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
        ensure_pointers(ARTIFACT_CERT_README)
        ensure_pointers(V1_FAMILY_README)
        ensure_pointers(AGE4_PROOF_README)
        ensure_pointers(AGE4_BRIDGE_README)
        ensure_pointers(W95_CERT_README)
        ensure_snippets(
            SANITY_GATE,
            (
                '"proof_certificate_family_selftest"',
                '[py, "tests/run_proof_certificate_family_selftest.py"]',
                '"proof_certificate_family_contract_selftest"',
                '[py, "tests/run_proof_certificate_family_contract_selftest.py"]',
                '"proof_certificate_family_contract_summary_selftest"',
                '[py, "tests/run_proof_certificate_family_contract_summary_selftest.py"]',
                '"proof_certificate_family_transport_contract_selftest"',
                '[py, "tests/run_proof_certificate_family_transport_contract_selftest.py"]',
                '"proof_certificate_family_transport_contract_summary_selftest"',
                '[py, "tests/run_proof_certificate_family_transport_contract_summary_selftest.py"]',
            ),
        )
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-family-selftest] ok lines=2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
