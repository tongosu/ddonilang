# gogae9_w95_cert

정본(규범): SSOT_ALL v20.8.0
기준: `docs/ssot/walks/gogae9/w95_cert/README.md`
Pack ID: `pack/gogae9_w95_cert`

## 목적

`teul-cli cert keygen|sign|verify` 최소 계약을 검증한다.

- 정상 케이스: verify PASS
- 1바이트 변조 케이스: verify FAIL (`E_CERT_VERIFY_FAIL`)
- proof artifact/cert contract:
  - `tests/proof_artifact_certificate_contract/README.md`
  - `python tests/run_proof_artifact_certificate_contract_selftest.py`
- proof certificate family:
  - `tests/proof_certificate_family/README.md`
  - `python tests/run_proof_certificate_family_selftest.py`
- gate0 runtime family:
  - `tests/gate0_runtime_family/README.md`
  - `python tests/run_gate0_runtime_family_selftest.py`

## 포함 파일

- `intent.md`
- `cert_cases.json`
- `golden.detjson`
- `golden.jsonl`
- `inputs/c00_contract_anchor/{input.ddn, expected_canon.ddn}`
- `inputs/c01_subject/run_manifest.detjson`
