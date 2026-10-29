# Proof Certificate V1 Draft Artifact Layers

## Stable Contract

- 목적:
  - `ddn.proof_certificate_v1_draft_artifact.v1` fixture를 `shared header`와 `profile body` 두 층으로 분해해, profile별 artifact가 공통 메타와 상태별 payload로 자연스럽게 나뉜다는 점을 고정한다.
  - 이 문서는 실제 구현을 추가하지 않고, 현재 draft artifact fixture를 더 작은 layer contract로 재구성할 수 있는지만 검사한다.
- pack 계약:
  - `pack/proof_certificate_v1_draft_artifact_layers_v1/README.md`
- 대상 surface:
  - `tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson`
  - `tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson`
  - `tests/proof_certificate_v1_draft_artifact_layers/fixtures/shared.proof_certificate_v1_draft_artifact_header.detjson`
  - `tests/proof_certificate_v1_draft_artifact_layers/fixtures/clean.proof_certificate_v1_draft_artifact_profile.detjson`
  - `tests/proof_certificate_v1_draft_artifact_layers/fixtures/abort.proof_certificate_v1_draft_artifact_profile.detjson`
- selftest:
  - `python tests/run_proof_certificate_v1_draft_artifact_layers_selftest.py`
  - `proof_certificate_v1_draft_artifact_layers_selftest`

## Shared Header

- `schema`
- `source_pack_schema`
- `source_pack_id`

## Profile Body

- `profile`
- `shared_shell_key_count`
- `state_delta_key_count`
- `candidate_manifest`
- `shared_shell`
- `state_delta`

## Consumer Surface

- `tests/proof_certificate_v1_draft_artifact/README.md`
- `tests/proof_certificate_v1_draft_contract/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `python tests/run_proof_certificate_v1_draft_artifact_selftest.py`
- `python tests/run_proof_certificate_v1_draft_artifact_layers_selftest.py`
- `python tests/run_proof_certificate_v1_draft_contract_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
