# seamgrim_wasm_viewmeta_statehash_v1

WASM 브릿지의 `state_hash`가 `보개_*` 변경에 오염되지 않는지 검증하는 팩.

검증 항목:

- 같은 상태(`x` 동일)에서 `보개_*` 값만 바꿔도 `state_hash`는 동일해야 함
- 실제 상태 키(`x`)를 바꾸면 `state_hash`가 달라져야 함

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_viewmeta_statehash_v1`

