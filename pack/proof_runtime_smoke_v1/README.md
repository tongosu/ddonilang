# proof_runtime_smoke_v1

`살피기`가 proof runtime item만 남기고 `state_hash`에는 영향을 주지 않는지 고정하는 팩.

## 케이스

- `input_base.ddn`: 같은 상태를 plain 값으로 끝내는 기준 run
- `input_check.ddn`: 같은 상태를 `살피기` 결과로 끝내는 proof runtime run

## 검증

- `python tests/run_pack_golden.py proof_runtime_smoke_v1`
- `python tests/run_proof_runtime_minimum_check.py`

