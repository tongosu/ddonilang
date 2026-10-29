# Seamgrim Consumer Surface Family

## Stable Contract

- parent line:
  - `seamgrim consumer surface family`
- child lines:
  - `tests/seamgrim_state_view_boundary_family/README.md`
- child checks:
  - `python tests/run_seamgrim_workflow_contract_check.py`
  - `python tests/run_seamgrim_lesson_schema_gate.py`
  - `python tests/run_seamgrim_visual_contract_check.py`
  - `python tests/run_seamgrim_browse_selection_flow_check.py`
  - `python tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_consumer_surface_family_selftest.py`
- fixed family line:
  - `workflow contract + schema gate + visual contract + browse selection flow + state/view boundary transport`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_consumer_surface_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_consumer_surface_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `workflow_contract,schema_gate,visual_contract,browse_selection_flow,state_view_boundary_transport,seamgrim_consumer_surface_family`
- progress schema:
  - `ddn.ci.seamgrim_consumer_surface_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_consumer_surface_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_consumer_surface_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,workflow_contract,schema_gate,visual_contract,browse_selection_flow,state_view_boundary_transport`
- progress schema:
  - `ddn.ci.seamgrim_consumer_surface_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_consumer_surface_family_transport_contract_selftest`
  - `seamgrim_consumer_surface_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim의 직접 소비면 계약과 `state/view hash` 경계 transport를 같은 상위 line으로 다시 묶는다.
- 현재 포함 범위:
  - `workflow_contract`
  - `schema_gate`
  - `visual_contract`
  - `browse_selection_flow`
  - `seamgrim_state_view_boundary_family_transport_contract`
- parent family:
  - `tests/seamgrim_surface_family/README.md`
  - `python tests/run_seamgrim_surface_family_selftest.py`
  - `tests/seamgrim_interaction_family/README.md`
  - `python tests/run_seamgrim_interaction_family_selftest.py`
