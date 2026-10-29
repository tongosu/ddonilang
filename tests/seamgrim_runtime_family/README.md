# Seamgrim Runtime Family

## Stable Contract

- parent line:
  - `seamgrim runtime family`
- child lines:
  - `tests/seamgrim_surface_family/README.md`
- child checks:
  - `python tests/run_seamgrim_deploy_artifacts_check.py`
  - `python tests/run_seamgrim_seed_pendulum_export_check.py`
  - `python tests/run_seamgrim_pendulum_runtime_visual_check.py`
  - `python tests/run_seamgrim_seed_runtime_visual_pack_check.py`
  - `python tests/run_seamgrim_surface_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_runtime_family_selftest.py`
- fixed family line:
  - `deploy artifacts + seed pendulum export + pendulum runtime visual + seed runtime visual pack + surface transport`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_runtime_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_runtime_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `deploy_artifacts,seed_pendulum_export,pendulum_runtime_visual,seed_runtime_visual_pack,surface_transport,seamgrim_runtime_family`
- progress schema:
  - `ddn.ci.seamgrim_runtime_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_runtime_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_runtime_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,deploy_artifacts,seed_pendulum_export,pendulum_runtime_visual,seed_runtime_visual_pack,surface_transport`
- progress schema:
  - `ddn.ci.seamgrim_runtime_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_runtime_family_transport_contract_selftest`
  - `seamgrim_runtime_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim 실행 소비면과 상위 surface transport line을 같은 상위 bundle로 다시 묶는다.
- 현재 포함 범위:
  - `deploy_artifacts`
  - `seed_pendulum_export`
  - `pendulum_runtime_visual`
  - `seed_runtime_visual_pack`
  - `seamgrim_surface_family_transport_contract`
- parent family:
  - `tests/seamgrim_gate_family/README.md`
  - `python tests/run_seamgrim_gate_family_selftest.py`
  - `tests/seamgrim_stack_family/README.md`
  - `python tests/run_seamgrim_stack_family_selftest.py`
  - `tests/seamgrim_delivery_family/README.md`
  - `python tests/run_seamgrim_delivery_family_selftest.py`
