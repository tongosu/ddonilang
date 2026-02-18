# D-PACK: bogae_bg_key_v1

## 목적
- 보개 배경색 키 표기 통일을 검증한다.
- 정본 `보개_바탕색`과 레거시 `bogae_bg`가 동일한 렌더 결과를 만든다.

## 구성
- `input_canon.ddn`: 정본 키 사용
- `input_alias.ddn`: 레거시 키 사용
- `tests/README.md`: 수동 실행 가이드

## DoD(최소)
- 두 입력의 `bogae_hash`가 동일하다.
