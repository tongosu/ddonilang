# 테스트

## 수동 실행
- `teul-cli run pack/gate0_contract_abort_statehash_v1/input_pre_abort.ddn`
- `teul-cli run pack/gate0_contract_abort_statehash_v1/input_post_abort.ddn`
- `teul-cli run pack/gate0_contract_abort_statehash_v1/input_alert.ddn`
- `teul-cli run pack/gate0_contract_abort_statehash_v1/input_nested_abort.ddn`

## 기대 사항
- `pre_abort`: 전제 `(중단)` 위반 뒤 `x`는 5로 복원되고 `끝`은 실행되지 않는다.
- `post_abort`: 보장 `(중단)` 위반 뒤 `y`는 5로 복원되고 `끝`은 실행되지 않는다.
- `alert`: 알림 모드는 `z=6`, `끝=1` 상태가 유지된다.
- `nested_abort`: 안쪽 `(중단)` 위반은 안쪽 계약 프레임만 롤백한다. 바깥 블록에서 이미 끝난 `w <- (w + 1)`은 유지되어 `w=6`이 남고, 뒤의 `끝 <- 1`은 실행되지 않는다.
