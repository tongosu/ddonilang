# proof_certificate_digest_axes_v1

`proof certificate digest axes` selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_digest_axes_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 clean/abort proof pair에서 같아야 하는 digest와 달라야 하는 digest 축을 정확히 검증해야 한다.

## 구성
- `expected/proof_certificate_digest_axes.stdout.txt`
