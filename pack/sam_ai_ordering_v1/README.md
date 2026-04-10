# sam_ai_ordering_v1

Sam AI execution ordering hard-cut pack.

핵심:
- ordering key = `(agent_id, recv_seq)`
- payload hash / intent kind / accepted_madi / target_madi는 execution ordering에 사용 금지
- duplicate/reverse recv_seq는 실패