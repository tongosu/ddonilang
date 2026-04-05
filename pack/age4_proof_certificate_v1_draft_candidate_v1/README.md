# age4_proof_certificate_v1_draft_candidate_v1

Pack ID: `pack/age4_proof_certificate_v1_draft_candidate_v1`

정본(규범): SSOT_ALL v20.8.0

## 목적

`proof_certificate_v1` 정본화 직전의 승격 후보를 self-contained bundle로 고정한다.

- 대상 bundle:
  - `draft_pack.detjson`
  - `fixtures/shared.proof_certificate_candidate_shell.detjson`
  - `fixtures/clean.proof_certificate_candidate_state.detjson`
  - `fixtures/abort.proof_certificate_candidate_state.detjson`
  - `fixtures/clean.proof_certificate_candidate.detjson`
  - `fixtures/abort.proof_certificate_candidate.detjson`
- 핵심 계약:
  - pack manifest schema는 `ddn.proof_certificate_v1_draft_pack.v1`
  - `shared shell(7)` + `state delta(6)`를 합치면 각 profile candidate manifest가 재구성된다
  - pack fixture 5종은 test-side candidate fixture 5종과 JSON 기준으로 동일하다
  - 이 pack은 새 proof certificate 구현이 아니라, 이미 닫힌 승격 후보면을 self-contained bundle로 복사한 것이다
  - proof certificate v1 promotion candidate:
    - `tests/proof_certificate_v1_promotion_candidate/README.md`
    - `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
  - proof certificate v1 draft artifact:
    - `tests/proof_certificate_v1_draft_artifact/README.md`
    - `python tests/run_proof_certificate_v1_draft_artifact_selftest.py`
  - proof certificate v1 draft contract:
    - `tests/proof_certificate_v1_draft_contract/README.md`
    - `python tests/run_proof_certificate_v1_draft_contract_selftest.py`
  - proof certificate candidate manifest:
    - `tests/proof_certificate_candidate_manifest/README.md`
    - `python tests/run_proof_certificate_candidate_manifest_selftest.py`
  - proof certificate candidate layers:
    - `tests/proof_certificate_candidate_layers/README.md`
    - `python tests/run_proof_certificate_candidate_layers_selftest.py`

## 포함 파일

- `draft_pack.detjson`
- `fixtures/shared.proof_certificate_candidate_shell.detjson`
- `fixtures/clean.proof_certificate_candidate_state.detjson`
- `fixtures/abort.proof_certificate_candidate_state.detjson`
- `fixtures/clean.proof_certificate_candidate.detjson`
- `fixtures/abort.proof_certificate_candidate.detjson`
