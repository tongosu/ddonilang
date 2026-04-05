# seamgrim_wasm_viewmeta_statehash_v1

WASM 브릿지의 `state_hash`/`view_hash` 경계를 함께 검증하는 팩.

검증 항목:

- 같은 상태(`x` 동일)에서 view 전용 키(`__view_*`)만 바꾸면 `state_hash`는 동일해야 함
- 같은 조건에서 실제 `view_hash`는 달라져야 함
- 실제 상태 키(`x`)를 바꾸면 `state_hash`와 `view_hash`가 함께 달라져야 함

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_viewmeta_statehash_v1`
- parent family:
  - `tests/state_view_hash_separation_family/README.md`
  - `python tests/run_state_view_hash_separation_family_selftest.py`
