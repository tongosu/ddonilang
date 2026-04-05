# proof_certificate_v1_draft_artifact_v1

`proof_certificate_v1` draft artifact selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_draft_artifact_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 draft pack fixture가 profile별 draft artifact shape로 정확히 펼쳐짐을 검증해야 한다.

## 구성
- `expected/draft_artifact.stdout.txt`
