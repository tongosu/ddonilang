# Proof Certificate Candidate Manifest

## Stable Contract

- 목적:
  - `저장형 증명값` / `proof certificate` 실제 구현 전 단계에서, 현재 proof artifact + cert layer만으로 채울 수 있는 최소 후보 manifest 필드 집합을 고정한다.
  - 이 문서는 정본 스키마를 선언하지 않고, 테스트 전용 후보 스키마 `ddn.proof_certificate_manifest_candidate.v1`를 통해 하위 필드 집합만 고정한다.
- pack 계약:
  - `pack/proof_certificate_candidate_manifest_v1/README.md`
- 대상 surface:
  - `pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson`
  - `pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson`
  - `pack/age4_proof_artifact_cert_subject_v1/cert_cases.json`
  - `tools/teul-cli/src/cli/cert.rs`
  - `tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson`
  - `tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson`
- selftest:
  - `python tests/run_proof_certificate_candidate_manifest_selftest.py`
  - `proof_certificate_candidate_manifest_selftest`

## Minimum Fields

- `schema`
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

- `tests/proof_certificate_candidate_layers/README.md`
- `tests/proof_certificate_candidate_profile_split/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`
- `tests/proof_certificate_digest_axes/README.md`
- `tests/proof_artifact_certificate_contract/README.md`
- `pack/age4_proof_artifact_cert_subject_v1/README.md`
- `python tests/run_proof_certificate_candidate_manifest_selftest.py`
- `python tests/run_proof_certificate_candidate_layers_selftest.py`
- `python tests/run_proof_certificate_candidate_profile_split_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
- `python tests/run_proof_certificate_v1_draft_pack_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
