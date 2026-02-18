# MATH_SUPPORT_CATALOG_V0 (학부 범위)

- generated_at: 2026-02-01
- ssot_base: v20.2.19 (sha256 82624ae81f49ddf26d709e3b8728d46d03e697349b90788ce0bbd831c8bddcb9)
- 목적: 학부 수학 커버리지를 “범위 ↔ 기능 ↔ 구현 층 ↔ pack/lesson”으로 한눈에 관리

> 표기: 표현(Expression) / 계산(Eval) / 변환(Transform) / 도구(Viz)

---

## 1) 범위 표(요약)

| 영역 | 대표 주제 | 최소 지원(권장) | seamgrim demo | pack |
|---|---|---|---|---|
| 함수/대수 | 다항/유리/지수/로그 | Expr+Eval | y=f(x) 그래프/표 | `pack/math_core_functions/*` |
| 삼각/쌍곡 | sin/cos/tan, sinh/cosh | Expr+Eval | 주기/위상 비교 오버레이 | `pack/math_trig/*` |
| 극한/연속 | 수열, 연속성 | Eval(수치)+Viz | 수열 수렴/발산 | `pack/math_limits/*` |
| 미분(단변수) | 도함수 | Eval(수치) | 접선/기울기 | `pack/math_diff_num/*` |
| 적분(단변수) | 정적분 | Eval(수치) | 면적(리만합) | `pack/math_int_num/*` |
| 급수/테일러 | 근사 | Eval(수치) | 근사 오버레이 | `pack/math_series/*` |
| 선형대수 | 벡터/행렬 | Eval(수치) | 변환 전후 점군 | `pack/math_linalg/*` |
| ODE | 초기값 문제 | Eval(수치) | 궤적/위상 | `pack/math_ode/*` |
| 확률/통계 | 분포/추정 | Eval(수치) | PDF/CDF 그래프 | `pack/math_prob/*` |
| 이산수학 | 조합/그래프 | Eval | 그래프 구조(선택) | `pack/math_discrete/*` |

---

## 2) pack 규칙(초안)
- 수치 결과는 `abs_err`, `rel_err`, `ulp` 중 최소 1개로 판정(항목별 고정)
- 소수 포맷은 고정(예: 6자리) + 반올림 규칙 고정
- NaN/Inf 규약 고정(표준 출력 문자열까지)

---

## 3) seamgrim lessons(초안)
- lessons는 `#이름:`/`#설명:` 메타 헤더를 포함한다.
- 각 lesson은:
  - `lesson.ddn` (정본)
  - `meta.toml` (required_views, suggested ranges)
  - `checks/*` (bridge_check 기대값)
