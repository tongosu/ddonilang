# proof_family_v1

`proof family` selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_family_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 `proof operation family` line과 `proof_certificate family` line이 같은 최상위 proof family contract를 이룸을 검증해야 한다.

## 구성
- `expected/proof_family.stdout.txt`
