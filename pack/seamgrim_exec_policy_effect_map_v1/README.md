실행정책(언어 축)과 open 모드(도구 축) 직교 매핑을 `canon --emit exec-policy-map-json`으로 고정하는 골든 pack.

- 스키마: `ddn.exec_policy_effect_map.v1`
- 언어 축:
  - `실행모드: 엄밀|일반`
  - `효과정책: 격리|허용`
- 도구 축:
  - open 모드: `deny|record|replay`
  - 우선순위: `cli > open_policy > default_deny`
- 결합 결과:
  - 엄밀 모드: `E_EFFECT_IN_STRICT_MODE`
  - 일반+격리: `E_EFFECT_IN_ISOLATED_MODE`
  - 일반+허용: open 모드별(`E_OPEN_DENIED`/record/replay)
  - 실행정책 구조 오류(중복/열거값): gate 오류 코드로 수렴
- 추가 커버:
  - 실행정책 블록 없음: `엄밀+격리` 기본값 수렴
  - 효과정책 열거값 오류(예: `기록`): `would_fail_code=E_EXEC_ENUM_INVALID` 고정
  - 실행모드 생략 + 효과정책만 명시(`허용`): `strict_effect_ignored=true` 고정
