# Proof Certificate V1 Draft Artifact

## Stable Contract

- 목적:
  - `proof_certificate_v1` 승격 후보 pack 위에 올라갈 profile별 draft artifact shape를 test-only fixture로 고정한다.
  - 이 문서는 실제 저장형 증명값 구현을 추가하지 않고, 현재 draft pack이 각 profile별 단일 artifact로 어떻게 펼쳐질지 최소 shape만 확인한다.
- pack 계약:
  - `pack/proof_certificate_v1_draft_artifact_v1/README.md`
- 대상 surface:
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/draft_pack.detjson`
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/shared.proof_certificate_candidate_shell.detjson`
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/clean.proof_certificate_candidate_state.detjson`
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/abort.proof_certificate_candidate_state.detjson`
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/clean.proof_certificate_candidate.detjson`
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/fixtures/abort.proof_certificate_candidate.detjson`
  - `tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson`
  - `tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson`
- selftest:
  - `python tests/run_proof_certificate_v1_draft_artifact_selftest.py`
  - `proof_certificate_v1_draft_artifact_selftest`

## Draft Artifact Fields

- `schema`
- `source_pack_schema`
- `source_pack_id`
- `profile`
- `shared_shell_key_count`
- `state_delta_key_count`
- `candidate_manifest`
- `shared_shell`
- `state_delta`

## Consumer Surface

- `tests/proof_certificate_v1_draft_artifact_layers/README.md`
- `tests/proof_certificate_v1_draft_contract/README.md`
- `tests/proof_certificate_v1_schema_candidate/README.md`
- `pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `python tests/run_proof_certificate_v1_draft_pack_selftest.py`
- `python tests/run_proof_certificate_v1_draft_artifact_selftest.py`
- `python tests/run_proof_certificate_v1_draft_artifact_layers_selftest.py`
- `python tests/run_proof_certificate_v1_draft_contract_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
