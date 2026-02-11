# D-PACK: input_key_alias_ko_v2

## 목적
- keymap_v2 확장 논의와 별개로, 현재 입력 테이프(KEY_REGISTRY_V1_MIN) 범위에서
  정본 키/별칭 키가 동일하게 동작함을 검증한다.

## 구성
- `input.ddn`: 정본/별칭 키 상태 비교 출력
- `sam/key_alias_v2.input.bin`: 1틱 입력 테이프(ArrowRight/Space/Enter/KeyZ 눌림)
- `tests/README.md`: 수동 실행 가이드

## DoD(최소)
- 정본 키/별칭 키의 `눌림` 결과가 동일하다.
- v2 범위 확장(NumPad/F13/Media/IME)은 입력 테이프 v2가 준비되면 보강한다.