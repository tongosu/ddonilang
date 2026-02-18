# math_calculus_v1

미분하기/적분하기(#ascii) 변환 v1 pack.

## 포함
- 자유변수 1개 자동 선택
- 옵션 묶음(변수/차수/상수포함) 처리
- 2차 이상 미분(차수) 표기
- 적분 상수 `+ C` 표기
- 이름=식 형태 보존
- #ascii 입력/출력 고정

## 실행
- `python tests/run_pack_golden.py math_calculus_v1`
- `teul-cli run pack/math_calculus_v1/input_error_no_var.ddn` (FATAL)
- `teul-cli run pack/math_calculus_v1/input_error_multi_var.ddn` (FATAL)
