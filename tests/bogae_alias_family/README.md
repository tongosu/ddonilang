# Bogae Alias Family

## Stable Contract
- 이 계약면은 보개 alias 계열의 상위 family 소비면이다.
- 하위 line:
  - `tests/bogae_shape_alias_contract/README.md`
  - `pack/seamgrim_bogae_madang_alias_v1/README.md`
- golden consumer surface:
  - `tools/teul-cli/tests/golden/W12/W12_G04_bogae_basic_rect_hash/main.ddn`
  - `tools/teul-cli/tests/golden/W13/W13_G01_web_view_artifacts/main.ddn`
  - `tools/teul-cli/tests/golden/W21/W21_G01_overlay_off/main.ddn`
  - `tools/teul-cli/tests/golden/W21/W21_G02_overlay_on/main.ddn`
- family 규칙:
  - draw/runtime surface는 legacy `bogae_bg`를 계속 소비한다.
  - shape surface는 canonical `생김새.결`를 기준으로 유지한다.
  - seamgrim canon surface는 legacy `보개장면`을 정본 `보개마당`으로 정본화하고 alias warning을 낸다.

## Checks
- `python tests/run_bogae_alias_family_selftest.py`
- `python tests/run_pack_golden.py seamgrim_bogae_madang_alias_v1`
- `python tests/run_ci_sanity_gate.py --profile core_lang`

## Viewer Pointer
- downstream viewer/export family: `tests/bogae_alias_viewer_family/README.md`
- upstream lang surface family: `tests/lang_surface_family/README.md`
- 검증: `python tests/run_bogae_alias_viewer_family_selftest.py`
- 검증: `python tests/run_lang_surface_family_selftest.py`

## Stable Transport Contract

- bundle `checks_text`:
  - `shape_alias_contract,alias_family,alias_viewer_family`
- progress schema:
  - `ddn.ci.bogae_alias_family_contract_selftest.progress.v1`
- sanity steps:
  - `bogae_alias_family_contract_selftest`
  - `bogae_alias_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
  - `age5 close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result / summary compact`
  - `ci_fail_brief / ci_fail_triage`
  - `ci_gate_report_index`
  - `python tests/run_bogae_shape_alias_contract_selftest.py`
  - `python tests/run_bogae_alias_family_selftest.py`
  - `python tests/run_bogae_alias_viewer_family_selftest.py`
  - `python tests/run_bogae_alias_family_contract_selftest.py`
  - `python tests/run_bogae_alias_family_contract_summary_selftest.py`
  - `python tests/run_age5_close_combined_report_contract_selftest.py`
  - `python tests/run_ci_aggregate_age5_child_summary_bogae_alias_family_transport_contract_selftest.py`
  - `python tests/run_ci_gate_summary_report_check_selftest.py`
  - `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
  - `python tests/run_ci_aggregate_status_line_selftest.py`
  - `python tests/run_ci_gate_final_status_line_selftest.py`
  - `python tests/run_ci_gate_result_check_selftest.py`
  - `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
  - `python tests/run_ci_gate_summary_line_check_selftest.py`
  - `python tests/run_ci_final_line_emitter_check.py`
  - `python tests/run_ci_gate_report_index_check_selftest.py`

- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- transport progress schema:
  - `ddn.ci.bogae_alias_family_transport_contract_selftest.progress.v1`
- transport sanity steps:
  - `bogae_alias_family_transport_contract_selftest`
  - `bogae_alias_family_transport_contract_summary_selftest`
- transport direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
  - `age5 close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result / summary compact`
  - `ci_fail_brief / triage`
  - `ci_gate_report_index`
  - `python tests/run_bogae_alias_family_transport_contract_selftest.py`
  - `python tests/run_bogae_alias_family_transport_contract_summary_selftest.py`
  - `python tests/run_ci_aggregate_age5_child_summary_bogae_alias_family_transport_selftest.py`
  - `python tests/run_ci_aggregate_status_line_selftest.py`
  - `python tests/run_ci_gate_final_status_line_selftest.py`
  - `python tests/run_ci_gate_result_check_selftest.py`
  - `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
  - `python tests/run_ci_gate_summary_line_check_selftest.py`
  - `python tests/run_ci_final_line_emitter_check.py`
  - `python tests/run_ci_gate_report_index_check_selftest.py`
