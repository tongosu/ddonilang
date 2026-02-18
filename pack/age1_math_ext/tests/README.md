# 테스트

## 수동 실행
- teul-cli run pack/age1_math_ext/input.ddn
- teul-cli run pack/age1_math_ext/input_eval_fail.ddn

## 기대 출력
- (#ascii) 수식{ sum(i, 1, 3, i) }
- (#ascii) 수식{ diff(x^2, x, 2) }
- (#ascii) 수식{ int(x, x) }
- input_eval_fail.ddn은 FATAL:FORMULA_EVAL_EXT_UNSUPPORTED로 실패
