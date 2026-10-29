# Seamgrim Graph API Parity

## Stable Contract

- 목적:
  - `export_graph.py` standalone 결과와 `ddn_exec_server.py /api/run` 결과가 같은 `seamgrim.graph.v0` 핵심 surface를 내는지 고정한다.
- compared surface:
  - `solutions/seamgrim_ui_mvp/tools/export_graph.py`
  - `solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py`
  - `solutions/seamgrim_ui_mvp/tools/ddn_exec_server_check.py`
  - `pack/seamgrim_graph_v0_basics/golden.jsonl`
- pinned rules:
  - `graph.schema`는 둘 다 `seamgrim.graph.v0`
  - `series[0].points`는 standalone/API가 canonical json 기준으로 같다.
  - `meta.source_input_hash`, `meta.result_hash`는 standalone/API가 같다.
  - `meta.input_name`, `meta.input_desc`는 standalone/API가 같다.

## Checks

- direct runner:
  - `python tests/run_seamgrim_graph_api_parity_check.py --out build/tmp/seamgrim_graph_api_parity.detjson`
- direct selftest:
  - `python tests/run_seamgrim_graph_api_parity_check_selftest.py`
- seamgrim gate:
  - `python tests/run_seamgrim_ci_gate.py`

## Parent Family

- `tests/seamgrim_bridge_family/README.md`
- `python tests/run_seamgrim_bridge_family_selftest.py`
