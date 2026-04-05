# rpg_invariants_smoke_v1

RPG 게임 법칙 + 계약 물림 런타임 스모크 팩.

## 목적

게임 불변을 현재 `teul-cli run`이 지원하는 계약 `(중단)` 물림으로 고정하고,
위반 시 상태가 물린 채 proof artifact에 남는지 확인한다.

## 케이스

- `input.ddn`: 골드 획득과 정상 피해는 반영되고, 과다 피해는 계약 물림으로 롤백

## 관련

- TASK_MALIM_NURI_RPG_BOX_V1_20260321.md Phase 2 R6
- PROPOSAL_MALIM_NURI_RPG_BOX_V1_20260321.md Phase 2
