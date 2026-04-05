# Seamgrim Graph Bridge Contract

## Stable Contract

- 목적:
  - `solutions/seamgrim_ui_mvp/tools/export_graph.py`가 `seamgrim.graph.v0`를 만들 때 `meta header` 포함 입력을 그대로 수용한다는 점을 고정한다.
  - `seamgrim.graph.v0`의 `meta.source_input_hash`와 `meta.result_hash`가 입력 본문/점열 결과를 기준으로 재계산 가능한 값이라는 점을 고정한다.
  - 같은 graph 결과를 `seamgrim.snapshot.v0` envelope에 넣었을 때 `run.hash.input/result`가 graph meta hash와 정확히 교차 일치한다는 점을 고정한다.
- compared surface:
  - `solutions/seamgrim_ui_mvp/tools/export_graph.py`
  - `solutions/seamgrim_ui_mvp/schema/seamgrim.graph.v0.md`
  - `solutions/seamgrim_ui_mvp/schema/seamgrim.snapshot.v0.md`
  - `pack/seamgrim_graph_v0_basics/golden.jsonl`
- pinned rules:
  - meta header(`input_name`, `input_desc`)는 graph export에서 유지된다.
  - `source_input_hash == sha256(normalize_ddn_for_hash(input))`
  - `result_hash == sha256(compute_result_hash(points))`
  - `snapshot.run.hash.input == graph.meta.source_input_hash`
  - `snapshot.run.hash.result == graph.meta.result_hash`

## Checks

- direct selftest:
  - `python tests/run_seamgrim_graph_bridge_contract_selftest.py`
- runner:
  - `python tests/run_seamgrim_graph_golden.py --out build/tmp/seamgrim_graph_bridge_contract.detjson`
- seamgrim gate:
  - `python tests/run_seamgrim_ci_gate.py`

## Parent Family

- `tests/seamgrim_bridge_family/README.md`
- `python tests/run_seamgrim_bridge_family_selftest.py`
