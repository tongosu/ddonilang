# Proof Certificate V1 Consumer Contract

## Stable Contract

- 목적:
  - `proof_certificate_v1` signed bundle의 소비 라인을 `signed emit profiles -> verify bundle -> verify report -> verify report digest contract` 4단 contract로 한 번에 묶는다.
  - 이 문서는 emit 자체가 아니라, 발급된 bundle을 소비하는 CLI surface가 같은 상위 line을 가리키는지만 확인한다.
- 대상 surface:
  - `pack/age4_proof_detjson_smoke_v1/README.md`
  - `pack/proof_certificate_v1_consumer_transport_contract_v1/README.md`
  - `tests/proof_certificate_v1_signed_emit_profiles/README.md`
  - `tests/proof_certificate_v1_verify_bundle/README.md`
  - `tests/proof_certificate_v1_verify_report/README.md`
  - `tests/proof_certificate_v1_verify_report_digest_contract/README.md`
  - `tests/proof_certificate_v1_signed_contract/README.md`
  - `tests/proof_certificate_v1_family/README.md`
  - `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
  - `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
  - `python tests/run_proof_certificate_v1_verify_report_selftest.py`
  - `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_consumer_transport_pack_check.py`
  - `python tests/run_proof_certificate_v1_family_selftest.py`
  - `proof_certificate_v1_consumer_contract_selftest`

## Stable Transport Contract

- bundle `checks_text`:
  - `signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract`
- progress schema:
  - `ddn.ci.proof_certificate_v1_consumer_transport_contract_selftest.progress.v1`
- sanity steps:
  - `proof_certificate_v1_consumer_transport_contract_selftest`
  - `proof_certificate_v1_consumer_transport_contract_summary_selftest`
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
  - `python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_consumer_transport_contract_summary_selftest.py`
  - `python tests/run_proof_certificate_v1_consumer_transport_pack_check.py`

## Consumer Chain

| layer | schema/surface | contract |
| --- | --- | --- |
| signed emit profiles | `clean/abort` signed bundle parity | 두 profile 모두 같은 signed output set을 쓴다 |
| verify bundle | `cert verify-proof-certificate --in <bundle>` | signed bundle의 cert/proof/runtime/source-proof parity를 직접 검증한다 |
| verify report | `ddn.proof_certificate_v1.verify_report.v1` | verify 결과를 저장 가능한 consumer artifact로 남긴다 |
| verify report digest contract | digest/signature parity summary | verify report가 proof digest/cert signature 축을 빠짐없이 싣는다는 상위 contract를 고정한다 |

## Consumer Surface

- `pack/age4_proof_detjson_smoke_v1/README.md`
- `pack/proof_certificate_v1_consumer_transport_contract_v1/README.md`
- `tests/proof_certificate_v1_signed_emit_profiles/README.md`
- `tests/proof_certificate_v1_verify_bundle/README.md`
- `tests/proof_certificate_v1_verify_report/README.md`
- `tests/proof_certificate_v1_verify_report_digest_contract/README.md`
- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_family/README.md`
- `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
- `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_transport_contract_summary_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_transport_pack_check.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_age5_close_combined_report_contract_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_proof_certificate_v1_consumer_transport_selftest.py`
- `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
- `python tests/run_ci_aggregate_status_line_selftest.py`
- `python tests/run_ci_gate_final_status_line_selftest.py`
- `python tests/run_ci_gate_result_check_selftest.py`
- `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
- `python tests/run_ci_gate_summary_line_check_selftest.py`
- `python tests/run_ci_final_line_emitter_check.py`
- `python tests/run_ci_gate_report_index_check_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
