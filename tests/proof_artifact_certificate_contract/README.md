# Proof Artifact Certificate Contract

## Stable Contract

- 목적:
  - `AGE4 proof.detjson` 발급면과 `w95 cert` subject-hash/certificate pack이 미래 `증명값`/proof certificate 승격의 바로 아래층 계약으로 어떤 surface를 이미 고정했는지 한 번에 확인한다.
  - proof artifact schema literal은 `ddn.proof.detjson.v0`, artifact report schema는 `ddn.proof_artifact_summary.v1`, cert report schema는 `ddn.gogae9.w95.cert_report.v1`로 고정한다.
- pack 계약:
  - `pack/proof_artifact_certificate_contract_v1/README.md`
- 대상 surface:
  - `pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson`
  - `pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson`
  - `pack/age4_proof_artifact_cert_subject_v1/cert_cases.json`
  - `pack/age4_proof_artifact_cert_subject_v1/golden.detjson`
  - `tests/proof_certificate_digest_axes/README.md`
  - `tests/run_proof_artifact_digest_selftest.py`
  - `tests/run_age4_proof_artifact_report_selftest.py`
  - `pack/gogae9_w95_cert/golden.detjson`
  - `pack/gogae9_w95_cert/cert_cases.json`
  - `tests/run_w95_cert_pack_check.py`
  - `python tests/run_w95_cert_pack_check.py --pack pack/age4_proof_artifact_cert_subject_v1`
- selftest:
  - `python tests/run_proof_artifact_certificate_contract_selftest.py`
  - `proof_artifact_certificate_contract_selftest`

## Matrix

| layer | surface | contract |
| --- | --- | --- |
| proof artifact emit | `pack/age4_proof_detjson_smoke_v1` | `clean/abort` 두 proof artifact가 모두 `ddn.proof.detjson.v0` + `kind=run_contract_certificate_v1`를 유지 |
| proof artifact cert bridge | `pack/age4_proof_artifact_cert_subject_v1` | clean/abort proof artifact 바이트가 모두 cert subject로 서명 가능하고 tamper verify FAIL을 유지 |
| proof artifact aggregate | `tests/run_proof_artifact_digest_selftest.py`, `tests/run_age4_proof_artifact_report_selftest.py` | proof summary/report가 runtime error taxonomy와 verified/unverified count를 함께 고정 |
| cert pack | `pack/gogae9_w95_cert` | `ddn.gogae9.w95.cert_cases.v1` + `ddn.gogae9.w95.cert_report.v1` + deterministic subject hash/tamper detection 고정 |

## Consumer Surface

- `pack/age4_proof_detjson_smoke_v1/README.md`
- `pack/age4_proof_artifact_cert_subject_v1/README.md`
- `tests/proof_certificate_digest_axes/README.md`
- `tests/proof_certificate_candidate_manifest/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `pack/gogae9_w95_cert/README.md`
- `tests/run_proof_artifact_digest_selftest.py`
- `tests/run_age4_proof_artifact_report_selftest.py`
- `tests/run_w95_cert_pack_check.py`
- `python tests/run_proof_certificate_digest_axes_selftest.py`
- `python tests/run_proof_certificate_candidate_manifest_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
- `python tests/run_w95_cert_pack_check.py --pack pack/age4_proof_artifact_cert_subject_v1`
- `python tests/run_proof_artifact_certificate_contract_selftest.py`
- `tests/proof_certificate_family/README.md`
- `python tests/run_proof_certificate_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
