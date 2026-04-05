# proof_certificate_v1_promotion_candidate_v1

`proof_certificate_v1` promotion candidate selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_promotion_candidate_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 artifact/cert bridge, digest axes, candidate manifest, profile split, layers가 같은 승격 후보 surface를 가리킴을 검증해야 한다.

## 구성
- `expected/promotion_candidate.stdout.txt`
