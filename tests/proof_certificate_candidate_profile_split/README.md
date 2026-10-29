# Proof Certificate Candidate Profile Split

## Stable Contract

- 목적:
  - `ddn.proof_certificate_manifest_candidate.v1` clean/abort fixture pair에서 공통 필드와 상태별 필드를 분리한다.
  - 이 문서는 최종 구현을 고정하지 않고, 현재 후보 manifest가 `shared profile`과 `state profile`로 자연스럽게 나뉜다는 점만 확인한다.
- pack 계약:
  - `pack/proof_certificate_candidate_profile_split_v1/README.md`
- 대상 surface:
  - `tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson`
  - `tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson`
  - `tests/proof_certificate_candidate_manifest/README.md`
- selftest:
  - `python tests/run_proof_certificate_candidate_profile_split_selftest.py`
  - `proof_certificate_candidate_profile_split_selftest`

## Shared Profile

- `schema`
- `proof_schema`
- `proof_kind`
- `cert_manifest_schema`
- `cert_algo`
- `proof_runtime_hash`
- `solver_translation_hash`

## State Profile

- `verified`
- `contract_diag_count`
- `proof_subject_hash`
- `canonical_body_hash`
- `state_hash`
- `trace_hash`

## Consumer Surface

- `tests/proof_certificate_candidate_layers/README.md`
- `tests/proof_certificate_candidate_manifest/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `python tests/run_proof_certificate_candidate_layers_selftest.py`
- `python tests/run_proof_certificate_candidate_profile_split_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
