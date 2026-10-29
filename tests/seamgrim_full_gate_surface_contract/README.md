# Seamgrim Full Gate Surface Contract

## Stable Contract

- 목적:
  - `run_seamgrim_full_check.py`가 실제로 묶는 `graph preprocess + scene/session + lesson schema gate + full gate wrapper` surface를 별도 contract로 고정한다.
- compared surface:
  - `tests/run_seamgrim_export_graph_preprocess_check.py`
  - `tests/run_seamgrim_scene_session_check.py`
  - `tests/run_seamgrim_lesson_schema_gate.py`
  - `tests/run_seamgrim_full_gate_check.py`
- pinned rules:
  - `export_graph_preprocess`는 기본 lesson 입력에서 preprocess rewrite 검사를 통과한다.
  - `scene_session`은 기본 lesson의 `seamgrim.scene.v0` / `seamgrim.session.v0` expected 문서를 통과한다.
  - `lesson_schema_gate`는 committed schema status와 promote dry-run 상태를 통과한다.
  - `full_gate`는 위 세 surface를 다시 묶는 wrapper로 성공한다.

## Checks

- direct runner:
  - `python tests/run_seamgrim_full_gate_surface_contract_check.py --out build/tmp/seamgrim_full_gate_surface_contract.detjson`
- direct selftest:
  - `python tests/run_seamgrim_full_gate_surface_contract_selftest.py`

## Parent Family

- `tests/seamgrim_guard_surface_family/README.md`
- `python tests/run_seamgrim_guard_surface_family_selftest.py`
