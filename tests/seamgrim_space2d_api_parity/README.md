# Seamgrim Space2D API Parity

## Stable Contract

- 목적:
  - `export_space2d.py` standalone 결과와 `ddn_exec_server.py /api/run` 결과가 같은 `seamgrim.space2d.v0` surface를 내는지 고정한다.
- compared surface:
  - `solutions/seamgrim_ui_mvp/tools/export_space2d.py`
  - `solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py`
  - `pack/seamgrim_space2d_v0_basics/golden.jsonl`
- pinned rules:
  - standalone 문서와 API 문서는 canonical json 기준으로 같다.
  - `meta.source_input_hash`는 standalone/API가 같다.
  - `meta.title`은 standalone/API가 같다.
  - `camera`는 point가 있을 때만 생기며 standalone/API가 같다.

## Checks

- direct runner:
  - `python tests/run_seamgrim_space2d_api_parity_check.py --out build/tmp/seamgrim_space2d_api_parity.detjson`
- direct selftest:
  - `python tests/run_seamgrim_space2d_api_parity_check_selftest.py`
- seamgrim gate:
  - `python tests/run_seamgrim_ci_gate.py`

## Parent Family

- `tests/seamgrim_bridge_family/README.md`
- `python tests/run_seamgrim_bridge_family_selftest.py`
