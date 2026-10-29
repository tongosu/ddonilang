# Bogae Shape Alias Contract

## Stable Contract
- 이 계약면은 보개 drawlist alias 정책의 상위 소비면이다.
- pack line:
  - `pack/bogae_bg_key_v1`
  - `pack/bogae_canvas_key_precedence_v1`
  - `pack/bogae_canvas_ssot_key_precedence_v1`
  - `pack/bogae_drawlist_listkey_v1`
  - `pack/bogae_drawlist_trait_alias_v1`
  - `pack/bogae_shape_trait_ssot_alias_v1`
  - `pack/bogae_api_catalog_v1_basic/README.md`
- 경계:
  - `보개_바탕색` ≡ `bogae_bg`
  - `보개_바탕_가로/세로` 우선, `보개_그림판_가로/세로`와 `bogae_canvas_w/h`는 하위 호환 alias
  - `생김새.특성` 우선, `생김새.결`과 `모양.트레잇`는 하위 호환 alias
  - `보개_그림판_목록` item은 `특성`, `결`, `트레잇` field를 모두 수용
- parity:
  - `pack/bogae_bg_key_v1`, `pack/bogae_canvas_key_precedence_v1`, `pack/bogae_canvas_ssot_key_precedence_v1`는 같은 canonical rect hash line을 유지한다.
  - `pack/bogae_drawlist_listkey_v1`, `pack/bogae_drawlist_trait_alias_v1`, `pack/bogae_shape_trait_ssot_alias_v1`는 같은 drawlist/shape trait hash line을 유지한다.

## Checks
- `python tests/run_bogae_shape_alias_contract_selftest.py`
- `python tests/run_pack_golden.py bogae_bg_key_v1 bogae_canvas_key_precedence_v1 bogae_canvas_ssot_key_precedence_v1 bogae_drawlist_listkey_v1 bogae_drawlist_trait_alias_v1 bogae_shape_trait_ssot_alias_v1`
- `python tests/run_ci_sanity_gate.py --profile core_lang`

## Family Pointer
- 상위 family: `tests/bogae_alias_family/README.md`
- 검증: `python tests/run_bogae_alias_family_selftest.py`
