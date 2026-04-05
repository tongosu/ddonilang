# gate0_contract_abort_statehash_v1

계약 `(중단)`이 발생했을 때 해당 계약 프레임의 상태 변형만 롤백되고, 그 결과가 `state_hash`에 비오염으로 남는지 확인하는 회귀 팩.

포함 범위:

- `pre_abort`: 전제 위반 시 현재 계약 프레임 상태 복원
- `post_abort`: 보장 위반 시 현재 계약 프레임 상태 복원
- `alert`: 알림 모드는 롤백 없이 계속 진행
- `nested_abort`: 안쪽 계약만 롤백하고 바깥 프레임의 선행 변경은 유지

검증:

- `python tests/run_pack_golden.py gate0_contract_abort_statehash_v1`
- `python tests/run_pack_golden.py --update gate0_contract_abort_statehash_v1`
