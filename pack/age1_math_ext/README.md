# D-PACK: age1_math_ext

## 목적
- ddn.math/ext 예약 호출(sum/prod/diff/int) 파싱/표시를 확인한다.
- Gate0 풀기에서 ext 호출이 FATAL로 실패하는지 확인한다.

## 구성(권장)
- input.ddn : 확장 호출을 포함한 수식 값 표시
- input_eval_fail.ddn : 풀기 실패 예시
- tests/    : 실행/검증 절차

## DoD(최소)
- input.ddn 실행 시 수식 표시가 기대값으로 나온다.
- input_eval_fail.ddn 실행 시 FATAL 코드가 출력된다.
