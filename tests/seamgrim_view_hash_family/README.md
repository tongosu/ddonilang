# Seamgrim View Hash Family

## Stable Contract

- parent line:
  - `seamgrim view hash family`
- child lines:
  - `pack/patent_b_state_view_hash_isolation_v1/README.md`
  - `pack/seamgrim_moyang_template_instance_view_boundary_v1/README.md`
  - `pack/dotbogi_ddn_interface_v1_smoke/README.md`
  - `tests/state_view_hash_separation_family/README.md`
- child checks:
  - `python tests/run_patent_b_state_view_hash_isolation_check.py`
  - `python tests/run_seamgrim_moyang_view_boundary_pack_check.py`
  - `python tests/run_dotbogi_view_meta_hash_pack_check.py`
  - `python tests/run_state_view_hash_separation_family_selftest.py`
  - `python tests/run_seamgrim_view_hash_family_selftest.py`
- fixed family line:
  - `patent_b state/view hash isolation + moyang template instance view boundary + dotbogi view_meta hash + state_view_hash_separation_family`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_view_hash_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_view_hash_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `patent_b_view_hash_isolation,moyang_view_boundary,dotbogi_view_meta_hash,state_view_hash_separation_family,seamgrim_view_hash_family`
- progress schema:
  - `ddn.ci.seamgrim_view_hash_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_view_hash_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_view_hash_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,patent_b_view_hash_isolation,moyang_view_boundary,dotbogi_view_meta_hash,state_view_hash_separation_family`
- progress schema:
  - `ddn.ci.seamgrim_view_hash_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_view_hash_family_transport_contract_selftest`
  - `seamgrim_view_hash_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim consumer surface에서 `state_hash`와 분리된 `view_hash/view_meta` 계약을 한 줄로 다시 묶는다.
- 현재 포함 범위:
  - `patent_b_state_view_hash_isolation_v1`
  - `seamgrim_moyang_template_instance_view_boundary_v1`
  - `dotbogi_ddn_interface_v1_smoke`
  - `state_view_hash_separation_family`
- parent family:
  - `tests/seamgrim_state_view_boundary_family/README.md`
  - `python tests/run_seamgrim_state_view_boundary_family_selftest.py`
