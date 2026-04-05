# external_intent_boundary_v1

External intent boundary skeleton pack.

This pack does not execute runtime behavior directly. It pins the minimum
contract vocabulary for:

- `입력원천` 6값 (`사람`, `슬기`, `밖일`, `일정`, `이어전달`, `펼침실행`)
- `주입방식` 2값 (`실주입`, `재연주입`)
- `행동갈래` 2값 (`보기만`, `세계영향`)
- `sam` 경계 입력 정규화
- gatekeeper/policy 경계 검사
- 재연에서 원천 재호출 금지
- 주입 이후 `state_hash` 결정성 유지

The check entry is:

- `python tests/run_external_intent_boundary_pack_check.py`
