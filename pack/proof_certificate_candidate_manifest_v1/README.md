# proof_certificate_candidate_manifest_v1

`proof_certificate` candidate manifest selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_candidate_manifest_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 proof artifact, cert bridge, digest 축으로부터 최소 후보 manifest 필드 집합이 정확히 고정됨을 검증해야 한다.

## 구성
- `expected/candidate_manifest.stdout.txt`
