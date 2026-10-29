# Seamgrim Interaction Family

## Stable Contract

- parent line:
  - `seamgrim interaction family`
- child lines:
  - `tests/seamgrim_consumer_surface_family/README.md`
- child checks:
  - `python tests/run_seamgrim_consumer_surface_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_block_editor_smoke_check.py`
  - `python tests/run_seamgrim_playground_smoke_check.py`
  - `python tests/run_seamgrim_interaction_family_selftest.py`
- fixed family line:
  - `consumer surface transport + block editor smoke + playground smoke`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_interaction_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_interaction_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `consumer_surface_transport,block_editor_smoke,playground_smoke,seamgrim_interaction_family`
- progress schema:
  - `ddn.ci.seamgrim_interaction_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_interaction_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_interaction_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,consumer_surface_transport,block_editor_smoke,playground_smoke`
- progress schema:
  - `ddn.ci.seamgrim_interaction_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_interaction_family_transport_contract_selftest`
  - `seamgrim_interaction_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim의 실제 상호작용면을 `consumer contract + block editor + playground` 한 줄로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_consumer_surface_family_transport_contract`
  - `run_seamgrim_block_editor_smoke_check`
  - `run_seamgrim_playground_smoke_check`
- parent family:
  - `tests/seamgrim_application_family/README.md`
  - `python tests/run_seamgrim_application_family_selftest.py`
