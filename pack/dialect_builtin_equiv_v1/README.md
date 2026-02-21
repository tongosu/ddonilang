# dialect_builtin_equiv_v1

ko/en/sym3 말씨에서 stdlib 호출(정본/별칭 혼합)이 같은 결과를 내는지 확인하는 최소 회귀 팩이다.

## 검증 포인트
- ko: `절댓값/최댓값/최솟값` 별칭 호출
- en/sym3: `abs/max/min` 호출
- 말씨별 조건 토큰(`일때/if/?=>`) 동작

## 실행
python tests/run_pack_golden.py dialect_builtin_equiv_v1
