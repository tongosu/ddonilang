# Seamgrim Gate Family

## Stable Contract

- parent line:
  - `seamgrim gate family`
- child lines:
  - `tests/seamgrim_runtime_family/README.md`
- child checks:
  - `python tests/run_seamgrim_runtime_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_group_id_summary_check.py`
  - `python tests/run_seamgrim_runtime_fallback_metrics_check.py`
  - `python tests/run_seamgrim_runtime_fallback_policy_check.py`
  - `python tests/run_seamgrim_ddn_exec_server_gate_check.py`
  - `python tests/run_seamgrim_pendulum_bogae_shape_check.py`
  - `python tests/run_seamgrim_full_gate_check.py`
  - `python tests/run_seamgrim_gate_family_selftest.py`
- fixed family line:
  - `runtime transport + group_id summary + runtime fallback metrics/policy + ddn_exec server + pendulum bogae shape + full gate`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_gate_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_gate_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `runtime_transport,group_id_summary,runtime_fallback_metrics,runtime_fallback_policy,ddn_exec_server_gate,pendulum_bogae_shape,full_gate,seamgrim_gate_family`
- progress schema:
  - `ddn.ci.seamgrim_gate_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_gate_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_gate_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,runtime_transport,group_id_summary,runtime_fallback_metrics,runtime_fallback_policy,ddn_exec_server_gate,pendulum_bogae_shape,full_gate`
- progress schema:
  - `ddn.ci.seamgrim_gate_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_gate_family_transport_contract_selftest`
  - `seamgrim_gate_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim gate tail의 실제 실행 소비면과 상위 runtime transport line을 한 줄의 최상위 umbrella로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_runtime_family_transport_contract`
  - `group_id_summary`
  - `runtime_fallback_metrics`
  - `runtime_fallback_policy`
  - `ddn_exec_server_check`
  - `pendulum_bogae_shape`
  - `full_check`
- parent family:
  - `tests/seamgrim_stack_family/README.md`
  - `python tests/run_seamgrim_stack_family_selftest.py`
  - `tests/seamgrim_release_family/README.md`
  - `python tests/run_seamgrim_release_family_selftest.py`
