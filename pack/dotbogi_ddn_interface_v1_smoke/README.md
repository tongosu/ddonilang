# pack/dotbogi_ddn_interface_v1_smoke

dotbogi.input/output.v1 기본 입출력(빈 events) 결정성 검증.

- 목적:
  - `events == []` 고정
  - `view_meta` 정본화/해시 고정
- 검증:
  - `python tests/run_pack_golden.py dotbogi_ddn_interface_v1_smoke`
- parent family:
  - `tests/seamgrim_view_hash_family/README.md`
  - `python tests/run_seamgrim_view_hash_family_selftest.py`
