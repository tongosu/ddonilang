# Proof Certificate V1 Verify Report Digest Contract

## Stable Contract

- 목적:
  - `ddn.proof_certificate_v1.verify_report.v1`가 단순 verify 결과를 넘어서 source proof parity 핵심 축을 직접 담는 consumer artifact인지 고정한다.
  - 이 문서는 emit line을 다시 검증하지 않고, verify report가 digest/signature 관점에서 어떤 최소 surface를 가져야 하는지만 묶는다.
- 대상 surface:
  - `tools/teul-cli/src/cli/cert.rs`
  - `tests/proof_certificate_v1_verify_report/README.md`
  - `tests/proof_certificate_v1_consumer_contract/README.md`
  - `tests/proof_certificate_v1_signed_contract/README.md`
  - `tests/proof_certificate_v1_promotion/README.md`
  - `python tests/run_proof_certificate_v1_verify_report_selftest.py`
  - `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
  - `proof_certificate_v1_verify_report_digest_contract_selftest`

## Digest Surface

- report schema:
  - `ddn.proof_certificate_v1.verify_report.v1`
- required digest/signature fields:
  - `proof_subject_hash`
  - `canonical_body_hash`
  - `proof_runtime_hash`
  - `solver_translation_hash`
  - `state_hash`
  - `trace_hash`
  - `cert_signature`

## Stable Transport Contract

- bundle `checks_text`:
  - `verify_report_digest_contract`
- progress schema:
  - `ddn.ci.proof_certificate_v1_verify_report_digest_contract_selftest.progress.v1`
- sanity steps:
  - `proof_certificate_v1_verify_report_digest_contract_selftest`
  - `proof_certificate_v1_verify_report_digest_transport_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
  - age5 close full-real report
  - aggregate preview summary
  - aggregate status line / parse
  - final status line / parse
  - gate result / summary line compact
  - `ci_fail_brief.txt`
  - `ci_fail_triage.detjson`
  - `ci_gate_report_index`
  - `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_verify_report_digest_transport_contract_summary_selftest.py`

## Consumer Surface

- `tests/proof_certificate_v1_verify_report/README.md`
- `tests/proof_certificate_v1_consumer_contract/README.md`
- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `python tests/run_proof_certificate_v1_verify_report_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_digest_transport_contract_summary_selftest.py`
- `python tests/run_age5_close_combined_report_contract_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_proof_certificate_v1_verify_report_digest_transport_selftest.py`
- `python tests/run_ci_gate_summary_report_check_selftest.py`
- `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
- `python tests/run_ci_aggregate_status_line_selftest.py`
- `python tests/run_ci_gate_final_status_line_selftest.py`
- `python tests/run_ci_gate_result_check_selftest.py`
- `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
- `python tests/run_ci_gate_summary_line_check_selftest.py`
- `python tests/run_ci_final_line_emitter_check.py`
- `python tests/run_ci_gate_report_index_check_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
