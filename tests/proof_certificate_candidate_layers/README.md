# Proof Certificate Candidate Layers

## Stable Contract

- 목적:
  - `ddn.proof_certificate_manifest_candidate.v1` 후보를 `shared shell`과 `state delta` 두 층으로 분해해, 후속 정본화 직전 형태의 2층 contract를 고정한다.
  - 이 문서는 구현을 추가하지 않고, fixture 합성만으로 clean/abort 후보 manifest가 그대로 재구성된다는 점을 확인한다.
- pack 계약:
  - `pack/proof_certificate_candidate_layers_v1/README.md`
- 대상 surface:
  - `tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson`
  - `tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson`
  - `tests/proof_certificate_candidate_layers/fixtures/shared.proof_certificate_candidate_shell.detjson`
  - `tests/proof_certificate_candidate_layers/fixtures/clean.proof_certificate_candidate_state.detjson`
  - `tests/proof_certificate_candidate_layers/fixtures/abort.proof_certificate_candidate_state.detjson`
- selftest:
  - `python tests/run_proof_certificate_candidate_layers_selftest.py`
  - `proof_certificate_candidate_layers_selftest`

## Shared Shell

- `schema`
- `proof_schema`
- `proof_kind`
- `cert_manifest_schema`
- `cert_algo`
- `proof_runtime_hash`
- `solver_translation_hash`

## State Delta

- `verified`
- `contract_diag_count`
- `proof_subject_hash`
- `canonical_body_hash`
- `state_hash`
- `trace_hash`

## Consumer Surface

- `tests/proof_certificate_candidate_manifest/README.md`
- `tests/proof_certificate_candidate_profile_split/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`
- `python tests/run_proof_certificate_candidate_layers_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
- `python tests/run_proof_certificate_v1_draft_pack_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
