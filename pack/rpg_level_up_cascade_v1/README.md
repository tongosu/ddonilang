# rpg_level_up_cascade_v1

RPG 레벨업 연쇄 알림 FIFO 스모크 팩.

## 목적

경험치 알림이 레벨업을 만들고, 같은 tick 안에서 nested `레벨업` 알림이 FIFO로 처리되는지 확인한다.

## 케이스

- `input.ddn`: 경험치 `120` 획득 뒤 경험치 `120`, 레벨 `2`, 레벨업 수 `1`, 마지막 알림 `레벨업`

## 관련

- SPEC_PHASE_TIMELINE_V1_20260322.md Phase 2 gate
