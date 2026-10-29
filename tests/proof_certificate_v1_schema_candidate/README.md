# Proof Certificate V1 Schema Candidate

## Stable Contract

- 목적:
  - `proof_certificate_v1` draft artifact를 실제 정본 스키마에 가까운 flat candidate shape로 평탄화한다.
  - 이 문서는 실제 저장형 구현을 추가하지 않고, 현재 draft artifact에서 바로 꺼낼 수 있는 최소 top-level field 집합만 고정한다.
  - test-only 후보 스키마 literal은 `ddn.proof_certificate_v1_candidate.v1`로 둔다.
- pack 계약:
  - `pack/proof_certificate_v1_schema_candidate_v1/README.md`
- 대상 surface:
  - `tests/proof_certificate_v1_draft_contract/README.md`
  - `tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson`
  - `tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson`
  - `tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson`
  - `tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson`
- selftest:
  - `python tests/run_proof_certificate_v1_schema_candidate_selftest.py`
  - `proof_certificate_v1_schema_candidate_selftest`

## Flat Candidate Fields

- `schema`
- `source_pack_schema`
- `source_pack_id`
- `profile`
- `proof_schema`
- `proof_kind`
- `cert_manifest_schema`
- `cert_algo`
- `verified`
- `contract_diag_count`
- `proof_subject_hash`
- `canonical_body_hash`
- `proof_runtime_hash`
- `solver_translation_hash`
- `state_hash`
- `trace_hash`

## Consumer Surface

- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_schema_candidate_split/README.md`
- `tests/proof_certificate_v1_draft_contract/README.md`
- `tests/proof_certificate_v1_draft_artifact/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `python tests/run_proof_certificate_v1_draft_contract_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_split_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
