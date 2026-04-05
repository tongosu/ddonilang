# seamgrim_bogae_madang_alias_v1

`보개장면` 별칭 입력이 정본 `보개마당`으로 정본화되고,
별칭 진단이 함께 발생하는지 검증하는 pack.

검증 포인트:

- 입력: `보개장면 { ... }`
- canon 출력: `보개마당 { ... }`
- 진단: `W_BOGAE_MADANG_ALIAS_DEPRECATED`
- 정본 입력(`보개마당`)은 무경고 유지

검증:

- `python tests/run_pack_golden.py seamgrim_bogae_madang_alias_v1`

상위 family:

- `tests/bogae_alias_family/README.md`
- `python tests/run_bogae_alias_family_selftest.py`
