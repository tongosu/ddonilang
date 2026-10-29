# Seamgrim Bridge Surface API Parity

## Stable Contract

- 목적:
  - `export_text.py`, `export_table.py`, `export_structure.py` standalone 결과와 `ddn_exec_server.py /api/run` 결과가 같은 bridge surface를 내는지 고정한다.
- compared surface:
  - `solutions/seamgrim_ui_mvp/tools/export_text.py`
  - `solutions/seamgrim_ui_mvp/tools/export_table.py`
  - `solutions/seamgrim_ui_mvp/tools/export_structure.py`
  - `solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py`
  - `pack/seamgrim_bridge_surface_v0_basics/golden.jsonl`
- pinned rules:
  - standalone 문서와 API 문서는 canonical json 기준으로 같다.
  - 각 surface의 `meta.source_input_hash`는 standalone/API가 같다.
  - 각 surface의 `meta.title`은 standalone/API가 같다.

## Checks

- direct runner:
  - `python tests/run_seamgrim_bridge_surface_api_parity_check.py --out build/tmp/seamgrim_bridge_surface_api_parity.detjson`
- direct selftest:
  - `python tests/run_seamgrim_bridge_surface_api_parity_check_selftest.py`
- seamgrim gate:
  - `python tests/run_seamgrim_ci_gate.py`

## Parent Family

- `tests/seamgrim_bridge_family/README.md`
- `python tests/run_seamgrim_bridge_family_selftest.py`
