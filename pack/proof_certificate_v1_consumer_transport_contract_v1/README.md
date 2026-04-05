# proof_certificate_v1_consumer_transport_contract_v1

`proof_certificate_v1` consumer transport selftest의 stdout/progress surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_consumer_transport_contract_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- progress snapshot은 `generated_at_utc`를 제외한 모든 필드가 expected JSON과 exact match여야 한다.
- `generated_at_utc`는 UTC offset이 포함된 ISO-8601 문자열이어야 한다.
- `run_proof_certificate_v1_consumer_transport_contract_summary_selftest.py`도 결정적인 stdout 한 줄을 재생성해야 한다.

## 구성
- `expected/consumer_transport.stdout.txt`
- `expected/consumer_transport.progress.detjson`
- `expected/consumer_transport_summary.stdout.txt`
