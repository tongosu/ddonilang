# pack/bogae_api_catalog_v1_basic

evidence_tier: golden_closed

보개 API 카탈로그 V1 기본 항목(사각형/텍스트/스프라이트) 결정성 검증.

- 목적:
  - `보개_그림판_가로/세로`, `보개_바탕색`, `보개_그림판_목록`, `보개로 그려.` 경로 고정
  - 동일 입력에서 동일 `bogae_hash` 재현

- 관련 alias contract:
  - `tests/bogae_shape_alias_contract/README.md`
  - `pack/bogae_bg_key_v1`
  - `pack/bogae_canvas_key_precedence_v1`
  - `pack/bogae_canvas_ssot_key_precedence_v1`
  - `pack/bogae_drawlist_listkey_v1`
  - `pack/bogae_drawlist_trait_alias_v1`
  - `pack/bogae_shape_trait_ssot_alias_v1`
  - `python tests/run_bogae_shape_alias_contract_selftest.py`
