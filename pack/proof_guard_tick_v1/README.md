# proof_guard_tick_v1

`지키기` 등록/해제와 tick 종료 후 자동 검사 횟수를 고정하는 팩.

## 케이스

- `input_off.ddn`: `지키기` 후 `지키기 끔`으로 해제된 기준 run
- `input_on.ddn`: `지키기`가 남아 tick마다 자동 검사되는 run

## 검증

- `python tests/run_pack_golden.py proof_guard_tick_v1`
- `python tests/run_proof_runtime_minimum_check.py`

