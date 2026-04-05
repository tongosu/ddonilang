# proof_certificate_v1_draft_artifact_layers_v1

`proof_certificate_v1` draft artifact layers selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_draft_artifact_layers_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 draft artifact fixture가 `shared header(3)`와 `profile body(6)`로 자연스럽게 분해됨을 검증해야 한다.

## 구성
- `expected/draft_artifact_layers.stdout.txt`
