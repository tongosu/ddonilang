# Proof Certificate V1 Promotion

## Stable Contract

- 목적:
  - 현재 닫힌 `proof_certificate_v1` 후보 line 전체를 최종 승격 후보 관점에서 한 문서와 selftest로 묶는다.
  - 이 문서는 실제 저장형 구현을 추가하지 않고, `draft contract`, `flat schema candidate`, `schema split`이 서로 같은 최종 후보 surface를 가리키는지만 확인한다.
- pack 계약:
  - `pack/proof_certificate_v1_promotion_v1/README.md`
- 대상 surface:
  - `tests/proof_certificate_v1_draft_contract/README.md`
  - `tests/proof_certificate_v1_schema_candidate/README.md`
  - `tests/proof_certificate_v1_schema_candidate_split/README.md`
  - `tests/proof_certificate_v1_signed_contract/README.md`
  - `tests/proof_certificate_v1_consumer_contract/README.md`
  - `tests/proof_certificate_v1_family/README.md`
  - `tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson`
  - `tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson`
  - `tests/proof_certificate_v1_schema_candidate_split/fixtures/shared.proof_certificate_v1_candidate_shell.detjson`
  - `tests/proof_certificate_v1_schema_candidate_split/fixtures/clean.proof_certificate_v1_candidate_state.detjson`
  - `tests/proof_certificate_v1_schema_candidate_split/fixtures/abort.proof_certificate_v1_candidate_state.detjson`
- selftest:
  - `python tests/run_proof_certificate_v1_promotion_selftest.py`
  - `python tests/run_proof_certificate_v1_family_selftest.py`
  - `proof_certificate_v1_promotion_selftest`

## Promotion Matrix

| layer | schema | contract |
| --- | --- | --- |
| draft line | `draft pack -> draft artifact -> artifact layers` | nested candidate line이 이미 stable contract로 닫혀 있다 |
| flat candidate | `ddn.proof_certificate_v1_candidate.v1` | 실제 정본 스키마 논의에 가장 가까운 top-level field set을 제공한다 |
| flat split | `shared shell(9)` + `state delta(7)` | flat candidate도 공통부와 상태부로 자연스럽게 분해된다 |

## Consumer Surface

- `tests/proof_certificate_v1_draft_contract/README.md`
- `tests/proof_certificate_v1_runtime_emit/README.md`
- `tests/proof_certificate_v1_signed_emit/README.md`
- `tests/proof_certificate_v1_signed_emit_profiles/README.md`
- `tests/proof_certificate_v1_verify_bundle/README.md`
- `tests/proof_certificate_v1_verify_report/README.md`
- `tests/proof_certificate_v1_verify_report_digest_contract/README.md`
- `tests/proof_certificate_v1_consumer_contract/README.md`
- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_family/README.md`
- `tests/proof_certificate_v1_schema_candidate/README.md`
- `tests/proof_certificate_v1_schema_candidate_split/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `python tests/run_proof_certificate_v1_draft_contract_selftest.py`
- `python tests/run_proof_certificate_v1_runtime_emit_selftest.py`
- `python tests/run_proof_certificate_v1_signed_emit_selftest.py`
- `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
- `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_split_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
