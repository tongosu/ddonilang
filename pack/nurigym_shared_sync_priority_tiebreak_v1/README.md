# nurigym_shared_sync_priority_tiebreak_v1

evidence_tier: runtime_pack
closure_claim: no
machine_checkable: yes

목적: shared_env(sync) + merge=priority에서 tie-break를 `agent_id 오름차순`으로 고정한다.
입력 순서가 달라져도 merge 결정은 같아야 하고, dataset header의 `source_hash`는 입력 파일 해시에 따라 달라진다.

포함 케이스:
- `input_order_a.json`: agent 배열 순서 A
- `input_order_b.json`: 동일 action 집합, agent 배열 순서 B(순서만 다름)

기대:
- `priority` tie-break는 runtime 단위테스트로 순서 독립성을 검증
- `dataset_hash`는 `source_hash` 차이 때문에 케이스별로 다를 수 있음(의도된 provenance)
