# Proof Certificate V1 Schema Candidate Split

## Stable Contract

- 목적:
  - `ddn.proof_certificate_v1_candidate.v1` clean/abort fixture pair를 공통 shell과 상태별 delta로 분리한다.
  - 이 문서는 실제 정본 스키마를 확정하지 않고, 현재 flat candidate가 `shared shell`과 `state delta`로 자연스럽게 나뉜다는 점만 확인한다.
- pack 계약:
  - `pack/proof_certificate_v1_schema_candidate_split_v1/README.md`
- 대상 surface:
  - `tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson`
  - `tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson`
  - `tests/proof_certificate_v1_schema_candidate_split/fixtures/shared.proof_certificate_v1_candidate_shell.detjson`
  - `tests/proof_certificate_v1_schema_candidate_split/fixtures/clean.proof_certificate_v1_candidate_state.detjson`
  - `tests/proof_certificate_v1_schema_candidate_split/fixtures/abort.proof_certificate_v1_candidate_state.detjson`
- selftest:
  - `python tests/run_proof_certificate_v1_schema_candidate_split_selftest.py`
  - `proof_certificate_v1_schema_candidate_split_selftest`

## Shared Shell

- `schema`
- `source_pack_schema`
- `source_pack_id`
- `proof_schema`
- `proof_kind`
- `cert_manifest_schema`
- `cert_algo`
- `proof_runtime_hash`
- `solver_translation_hash`

## State Delta

- `profile`
- `verified`
- `contract_diag_count`
- `proof_subject_hash`
- `canonical_body_hash`
- `state_hash`
- `trace_hash`

## Consumer Surface

- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_schema_candidate/README.md`
- `tests/proof_certificate_v1_draft_contract/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `python tests/run_proof_certificate_v1_schema_candidate_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_split_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
