# proof_certificate_v1_draft_contract_v1

`proof_certificate_v1` draft contract selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_draft_contract_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 draft pack, profile artifact, artifact layers가 같은 draft 후보 surface를 가리킴을 검증해야 한다.

## 구성
- `expected/draft_contract.stdout.txt`
