# gogae9_w97_self_heal

정본(규범): SSOT_ALL v20.8.0
기준: `docs/ssot/walks/gogae9/w97_self_heal/README.md`
Pack ID: `pack/gogae9_w97_self_heal`

## 목적

`teul-cli heal run --pack ...` 최소 계약을 검증한다.

- tick 단위 checkpoint/rollback 계약
- replay digest 기반 복구 재현성
- 복구 보고서(`heal_report.detjson`) 결정성
- gate0 runtime family:
  - `tests/gate0_runtime_family/README.md`
  - `python tests/run_gate0_runtime_family_selftest.py`

## 포함 파일

- `intent.md`
- `fault_scenarios.json`
- `golden.detjson`
- `golden.jsonl`
- `inputs/c00_contract_anchor/{input.ddn, expected_canon.ddn}`
