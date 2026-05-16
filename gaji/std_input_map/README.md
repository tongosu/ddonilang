# 표준 입력사상 가지 (gaji/std_input_map)

InputSnapshot 기반 입력을 격자게임 동작으로 해석하는 나-1 첫실행 surface입니다.

## 공개 API

- `입력사상.만들기(묶음)`
- `입력사상.방향(입력사상)`
- `입력사상.동작(입력사상, 이름)`

## 경계

- 입력 truth는 기존 `InputSnapshot`, `눌렸나`, `막눌렸나` 경로만 사용한다.
- 입력사상은 replay-safe 해석 helper이며 UI direct state mutation을 허용하지 않는다.
- 문서/pack/예제에서는 정본 이름만 사용한다.
