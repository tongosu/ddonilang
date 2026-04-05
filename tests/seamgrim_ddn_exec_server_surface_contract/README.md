# Seamgrim DDN Exec Server Surface Contract

## Stable Contract

- 목적:
  - `ddn_exec_server.py`가 seamgrim 소비면에서 제공하는 핵심 surface를 `server gate + graph api parity + bridge surface api parity + space2d api parity` 기준으로 고정한다.
- compared surface:
  - `tests/run_seamgrim_ddn_exec_server_gate_check.py`
  - `tests/run_seamgrim_graph_api_parity_check.py`
  - `tests/run_seamgrim_bridge_surface_api_parity_check.py`
  - `tests/run_seamgrim_space2d_api_parity_check.py`
- pinned rules:
  - `ddn_exec_server_gate`는 기본 서버/seed smoke를 통과한다.
  - `graph_api_parity`는 `ddn.seamgrim_graph_api_parity.v1` report에서 모든 case가 ok다.
  - `bridge_surface_api_parity`는 `ddn.seamgrim_bridge_surface_api_parity.v1` report에서 모든 case가 ok다.
  - `space2d_api_parity`는 `ddn.seamgrim_space2d_api_parity.v1` report에서 모든 case가 ok다.

## Checks

- direct runner:
  - `python tests/run_seamgrim_ddn_exec_server_surface_contract_check.py --out build/tmp/seamgrim_ddn_exec_server_surface_contract.detjson`
- direct selftest:
  - `python tests/run_seamgrim_ddn_exec_server_surface_contract_selftest.py`

## Parent Family

- `tests/seamgrim_guard_surface_family/README.md`
- `python tests/run_seamgrim_guard_surface_family_selftest.py`
