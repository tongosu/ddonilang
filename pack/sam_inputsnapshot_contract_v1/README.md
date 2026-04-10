# sam_inputsnapshot_contract_v1

Sam InputSnapshot 경계 계약을 고정한다.

핵심:
- 실행 정렬 키는 `(agent_id, recv_seq)`만 사용
- `accepted_madi`/`target_madi`는 scheduling metadata
- duplicate/reverse `recv_seq`는 입력 계약 위반으로 거부
- replay는 기록된 주입만 재사용