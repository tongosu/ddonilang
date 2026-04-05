# proof_artifact_certificate_contract_v1

`proof artifact certificate contract` selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_artifact_certificate_contract_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 AGE4 proof artifact emit, cert bridge, aggregate report, W95 cert pack이 같은 bridge contract를 가리킴을 검증해야 한다.

## 구성
- `expected/proof_artifact_certificate_contract.stdout.txt`
