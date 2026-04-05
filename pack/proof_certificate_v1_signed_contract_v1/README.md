# proof_certificate_v1_signed_contract_v1

`proof_certificate_v1` signed contract selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_signed_contract_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 `runtime emit -> signed emit -> promotion` 문서/포인터 체인이 모두 연결되어 있음을 검증해야 한다.

## 구성
- `expected/signed_contract.stdout.txt`
