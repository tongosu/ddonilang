# Tests

## Manual run
- `teul-cli run pack/diag_contract_mode_v1/input_alert.ddn`
- `teul-cli run pack/diag_contract_mode_v1/input_abort.ddn`

## Expected
- 알림 모드: `"알림 계속"`, `"끝"`이 출력된다.
- 중단 모드: `"중단됨"`만 출력되고 `"끝"`은 출력되지 않는다.
