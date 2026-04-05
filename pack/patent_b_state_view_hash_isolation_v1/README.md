# patent_b_state_view_hash_isolation_v1

특허 B 증빙용 상태-뷰 해시 분리 D-PACK.

검증 항목:

- `보개_카메라_x`(view meta)만 변경하면 `view_hash`는 변하지만 `state_hash`는 유지
- 상태 키(`x`)를 변경하면 `state_hash`/`view_hash`가 함께 변경

실행:

- `python tests/run_seamgrim_wasm_smoke.py patent_b_state_view_hash_isolation_v1 --skip-ui-common --skip-ui-pendulum --skip-wrapper --skip-vm-runtime --skip-space2d-source-gate`
- 갱신: 위 명령에 `--update` 추가
- parent family:
  - `tests/seamgrim_view_hash_family/README.md`
  - `python tests/run_seamgrim_view_hash_family_selftest.py`
