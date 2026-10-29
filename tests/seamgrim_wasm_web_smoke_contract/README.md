# Seamgrim WASM Web Smoke Contract

## Stable Contract

- parent line:
  - `seamgrim wasm web smoke contract`
- child surfaces:
  - `pack/seamgrim_wasm_v0_smoke/README.md`
  - `pack/seamgrim_interactive_event_smoke_v1/README.md`
  - `pack/seamgrim_temp_lesson_smoke_v1/README.md`
  - `pack/seamgrim_moyang_render_smoke_v1/README.md`
- child expected:
  - `pack/seamgrim_wasm_v0_smoke/expected/state_hash_trace.detjson`
  - `pack/seamgrim_interactive_event_smoke_v1/expected/interactive_event.detjson`
  - `pack/seamgrim_temp_lesson_smoke_v1/expected/temp_lesson.detjson`
  - `pack/seamgrim_moyang_render_smoke_v1/expected/moyang_render.detjson`
- parent expected:
  - `pack/seamgrim_wasm_web_smoke_contract_v1/expected/seamgrim_wasm_web_smoke_contract.stdout.txt`
  - `pack/seamgrim_wasm_web_smoke_contract_v1/expected/seamgrim_wasm_web_real_smoke.stdout.txt`
- child check:
  - `python tests/run_seamgrim_wasm_web_smoke_contract_selftest.py`
  - `python tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py`
- fixed family line:
  - `wasm bridge state_hash trace + web interactive event + web temperature table + web moyang render`

## Notes

- 이 contract는 `run_seamgrim_wasm_smoke.py`에서 실제로 사용하는 핵심 wasm/web smoke pack 4종의 expected surface를 묶어서 다시 고정한다.
- smoke 본 실행:
  - `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_v0_smoke seamgrim_interactive_event_smoke_v1 seamgrim_temp_lesson_smoke_v1 seamgrim_moyang_render_smoke_v1 --skip-ui-common --skip-ui-pendulum --skip-wrapper --skip-vm-runtime --skip-space2d-source-gate`
- pack check는 selftest stdout + real smoke stdout 두 축을 모두 `pack/seamgrim_wasm_web_smoke_contract_v1/expected/`로 고정한다.
- parent family:
  - `tests/seamgrim_surface_family/README.md`
  - `python tests/run_seamgrim_surface_family_selftest.py`
