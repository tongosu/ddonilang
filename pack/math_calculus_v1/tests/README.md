# Tests

## Manual run
- `teul-cli run pack/math_calculus_v1/input_error_no_var.ddn`
- `teul-cli run pack/math_calculus_v1/input_error_multi_var.ddn`

## Expected
- 자유변수 0개 또는 2개 이상인 경우 FATAL로 실패한다.
- 옵션 묶음의 차수/상수포함은 정본 표기에 반영된다.
