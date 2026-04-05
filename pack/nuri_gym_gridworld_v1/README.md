# nuri_gym_gridworld_v1

evidence_tier: golden_closed

대표 GridWorld(`nurigym.gridmaze2d`) 환경의 episode/dataset 산출물을 고정하는 팩.

## 케이스

- `input.json`: 3x3 grid에서 오른쪽 두 칸 이동으로 목표 도달

## 검증

- `python tests/run_pack_golden.py nuri_gym_gridworld_v1`
- `python tests/run_nuri_gym_contract_check.py`
