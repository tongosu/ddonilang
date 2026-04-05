# Proof Certificate V1 Draft Contract

## Stable Contract

- 목적:
  - `proof_certificate_v1` 정본화 직전의 draft line 전체를 한 문서와 selftest로 묶는다.
  - 이 문서는 `draft pack`, `profile artifact`, `artifact layers`가 서로 같은 후보 surface를 가리킨다는 점만 확인하고, 실제 저장형 구현은 추가하지 않는다.
- pack 계약:
  - `pack/proof_certificate_v1_draft_contract_v1/README.md`
- 대상 surface:
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`
  - `pack/age4_proof_certificate_v1_draft_candidate_v1/draft_pack.detjson`
  - `tests/proof_certificate_v1_draft_artifact/README.md`
  - `tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson`
  - `tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson`
  - `tests/proof_certificate_v1_draft_artifact_layers/README.md`
  - `tests/proof_certificate_v1_draft_artifact_layers/fixtures/shared.proof_certificate_v1_draft_artifact_header.detjson`
  - `tests/proof_certificate_v1_draft_artifact_layers/fixtures/clean.proof_certificate_v1_draft_artifact_profile.detjson`
  - `tests/proof_certificate_v1_draft_artifact_layers/fixtures/abort.proof_certificate_v1_draft_artifact_profile.detjson`
- selftest:
  - `python tests/run_proof_certificate_v1_draft_contract_selftest.py`
  - `proof_certificate_v1_draft_contract_selftest`

## Draft Matrix

| layer | schema | contract |
| --- | --- | --- |
| pack | `ddn.proof_certificate_v1_draft_pack.v1` | shared shell + clean/abort profile candidate pair를 self-contained bundle로 묶는다 |
| artifact | `ddn.proof_certificate_v1_draft_artifact.v1` | pack profile 하나가 단일 artifact shape로 펼쳐진다 |
| artifact layers | `shared header(3)` + `profile body(6)` | 단일 artifact는 공통 메타와 profile payload로 다시 분해된다 |

## Consumer Surface

- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_runtime_emit/README.md`
- `tests/proof_certificate_v1_schema_candidate/README.md`
- `tests/proof_certificate_v1_schema_candidate_split/README.md`
- `pack/age4_proof_certificate_v1_draft_candidate_v1/README.md`
- `tests/proof_certificate_v1_draft_artifact/README.md`
- `tests/proof_certificate_v1_draft_artifact_layers/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `python tests/run_proof_certificate_v1_draft_pack_selftest.py`
- `python tests/run_proof_certificate_v1_draft_artifact_selftest.py`
- `python tests/run_proof_certificate_v1_draft_artifact_layers_selftest.py`
- `python tests/run_proof_certificate_v1_draft_contract_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_selftest.py`
- `python tests/run_proof_certificate_v1_schema_candidate_split_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_proof_certificate_v1_runtime_emit_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
