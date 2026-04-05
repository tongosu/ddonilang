# rpg_combat_smoke_v1

RPG 전투 런타임 스모크 팩.

## 목적

플레이어 공격/적 공격이 같은 `임자` 내부 상태를 바꾸고, 마지막 공격에서
전투 종료 플래그가 고정되는지 확인한다.

## 케이스

- `input.ddn`: 플레이어 3회 공격(20, 20, 10), 적 2회 공격 뒤 `적_체력=0`, `플레이어_체력=80`

## 관련

- TASK_MALIM_NURI_RPG_BOX_V1_20260321.md Phase 2 R4
- PROPOSAL_MALIM_NURI_RPG_BOX_V1_20260321.md Phase 2
