# proof_certificate_v1_family_v1

`proof_certificate_v1` family selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_family_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 signed/consumer/promotion 세 상위 contract가 같은 `proof_certificate_v1` family를 가리킴을 검증해야 한다.

## 구성
- `expected/proof_certificate_v1_family.stdout.txt`
