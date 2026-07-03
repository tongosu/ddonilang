# numeric_root_finding_bisection_v1

- 상태: 구현 반영
- 주제: `수치해.이분법` scalar bisection root finding

## 커버리지

- 정상 경로: `(#ascii) 수식{x - 2}`를 `[0, 4]` bracket에서 찾아 `차림[근, 잔차, 반복횟수, "이분법"]` 반환.
- 오류 경로: bracket 양끝 함수값 부호가 같으면 `E_MATH_DOMAIN`으로 거부.

## 비범위

- Newton-Raphson
- 다변수 비선형계
- 다항식 전용 solver
- ODE solver
- solver 내부 부등식 제약
