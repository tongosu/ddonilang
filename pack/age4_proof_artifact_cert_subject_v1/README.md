# age4_proof_artifact_cert_subject_v1

Pack ID: `pack/age4_proof_artifact_cert_subject_v1`

정본(규범): SSOT_ALL v20.8.0

## 목적

AGE4 `ddn.proof.detjson.v0` artifact 자체가 cert subject 바이트로 서명/검증 가능한지 확인한다.

- 대상 subject:
  - `pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson`
  - `pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson`
- 핵심 계약:
  - 동일 proof artifact + 동일 seed keygen -> 동일 `subject_hash` / `signature`
  - clean/abort proof artifact 둘 다 cert subject로 서명 가능
  - signature 1바이트 변조 시 verify FAIL (`E_CERT_VERIFY_FAIL`)
  - proof artifact/cert contract:
    - `tests/proof_artifact_certificate_contract/README.md`
    - `python tests/run_proof_artifact_certificate_contract_selftest.py`
  - proof certificate digest axes:
    - `tests/proof_certificate_digest_axes/README.md`
    - `python tests/run_proof_certificate_digest_axes_selftest.py`
  - proof certificate candidate manifest:
    - `tests/proof_certificate_candidate_manifest/README.md`
    - `python tests/run_proof_certificate_candidate_manifest_selftest.py`
  - proof certificate v1 promotion candidate:
    - `tests/proof_certificate_v1_promotion_candidate/README.md`
    - `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
  - proof certificate family:
    - `tests/proof_certificate_family/README.md`
    - `python tests/run_proof_certificate_family_selftest.py`

## 포함 파일

- `intent.md`
- `cert_cases.json`
- `golden.detjson`
- `golden.jsonl`
- `inputs/c00_contract_anchor/{input.ddn, expected_canon.ddn}`
- `inputs/c01_subject/run_manifest.detjson`
