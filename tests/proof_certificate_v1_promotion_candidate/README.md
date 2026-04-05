# Proof Certificate V1 Promotion Candidate

## Stable Contract

- 목적:
  - 현재 고정된 proof artifact/cert bridge, digest 축, candidate manifest, profile split, shared shell/state delta가 합쳐지면 `proof_certificate_v1` 정본화 직전의 승격 후보가 된다는 점을 한 문서로 묶는다.
  - 이 문서는 실제 `저장형 증명값` 구현이나 새 스키마를 추가하지 않고, 이미 닫힌 하위 contract들이 모순 없이 같은 후보 surface를 가리키는지만 검증한다.
- pack 계약:
  - `pack/proof_certificate_v1_promotion_candidate_v1/README.md`
- 대상 surface:
  - `tests/proof_artifact_certificate_contract/README.md`
  - `tests/proof_certificate_digest_axes/README.md`
  - `tests/proof_certificate_candidate_manifest/README.md`
  - `tests/proof_certificate_candidate_profile_split/README.md`
  - `tests/proof_certificate_candidate_layers/README.md`
  - `tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson`
  - `tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson`
  - `tests/proof_certificate_candidate_layers/fixtures/shared.proof_certificate_candidate_shell.detjson`
  - `tests/proof_certificate_candidate_layers/fixtures/clean.proof_certificate_candidate_state.detjson`
  - `tests/proof_certificate_candidate_layers/fixtures/abort.proof_certificate_candidate_state.detjson`
- selftest:
  - `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
  - `proof_certificate_v1_promotion_candidate_selftest`

## Promotion Matrix

| layer | surface | contract |
| --- | --- | --- |
| artifact/cert bridge | `tests/proof_artifact_certificate_contract/README.md` | proof artifact bytes와 cert subject/signature 아래층이 이미 닫혀 있다 |
| digest axes | `tests/proof_certificate_digest_axes/README.md` | clean/abort pair에서 같아야 할 digest와 달라야 할 digest 축이 분리돼 있다 |
| candidate manifest | `tests/proof_certificate_candidate_manifest/README.md` | 후보 manifest 최소 13개 필드가 clean/abort fixture pair로 고정돼 있다 |
| profile split | `tests/proof_certificate_candidate_profile_split/README.md` | 후보 manifest는 공통 7개 + 상태별 6개 필드로 자연스럽게 분해된다 |
| layers | `tests/proof_certificate_candidate_layers/README.md` | `shared shell(7)` + `state delta(6)`를 합치면 clean/abort candidate를 재구성한다 |

## Consumer Surface

- `tests/proof_artifact_certificate_contract/README.md`
- `tests/proof_certificate_digest_axes/README.md`
- `tests/proof_certificate_candidate_manifest/README.md`
- `tests/proof_certificate_candidate_profile_split/README.md`
- `tests/proof_certificate_candidate_layers/README.md`
- `tests/proof_certificate_v1_draft_artifact/README.md`
- `tests/proof_certificate_v1_draft_artifact_layers/README.md`
- `tests/proof_certificate_v1_draft_contract/README.md`
- `tests/proof_certificate_v1_schema_candidate/README.md`
- `tests/proof_certificate_v1_schema_candidate_split/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`
- `pack/age4_proof_artifact_cert_subject_v1/README.md`
- `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
- `python tests/run_proof_certificate_v1_draft_pack_selftest.py`
- `python tests/run_proof_certificate_v1_draft_artifact_selftest.py`
- `python tests/run_proof_certificate_v1_draft_artifact_layers_selftest.py`
- `python tests/run_proof_certificate_v1_draft_contract_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_split_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
