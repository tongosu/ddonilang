# Bogae Alias Viewer Family

## Stable Contract
- 이 계약면은 보개 alias family의 viewer/export downstream 소비면이다.
- 상위 family:
  - `tests/bogae_alias_family/README.md`
- runtime/viewer surface:
  - `pack/bogae_web_viewer_v1/README.md`
  - `tools/teul-cli/tests/golden/W13/W13_G02_view_detbin_pipeline/main.ddn`
- export surface:
  - `pack/bogae_web_out_determinism/README.md`
  - `pack/bogae_web_out_determinism/golden/webout_001_manifest_hash.test.json`
- 규칙:
  - viewer surface는 canonical `보개_그림판_가로/세로`, `보개_바탕색`, `보개_그림판_목록`, `결` line을 소비한다.
  - export surface는 canonical `생김새.결`와 `살림.보개_*` line을 소비하고 manifest hash fixture를 고정한다.

## Checks
- `python tests/run_bogae_alias_viewer_family_selftest.py`
- `python tests/run_pack_golden.py bogae_web_viewer_v1`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
