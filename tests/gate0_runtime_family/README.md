# Gate0 Runtime Family

## Stable Contract

- 목적:
  - `core_lang` gate에서 이미 닫힌 상위 runtime 줄을 한 단계 위에서 함께 읽는 family contract를 고정한다.
  - 이 문서는 `lang runtime`, `W95 cert`, `W96 somssi`, `W97 self-heal`이 같은 Gate0 runtime family를 이룬다는 점만 확인한다.
- 대상 surface:
  - `tests/lang_runtime_family/README.md`
  - `pack/gogae9_w95_cert/README.md`
  - `pack/gogae9_w96_somssi_hub/README.md`
  - `pack/gogae9_w97_self_heal/README.md`
- selftest:
  - `python tests/run_lang_runtime_family_selftest.py`
  - `python tests/run_w95_cert_pack_check.py`
  - `python tests/run_w96_somssi_pack_check.py`
  - `python tests/run_w97_self_heal_pack_check.py`
  - `python tests/run_gate0_runtime_family_selftest.py`
  - `python tests/run_gate0_runtime_family_contract_selftest.py`
  - `python tests/run_gate0_runtime_family_contract_summary_selftest.py`
- sanity steps:
  - `lang_runtime_family_selftest`
  - `w95_cert_pack_check`
  - `w96_somssi_pack_check`
  - `w97_self_heal_pack_check`
  - `gate0_runtime_family_selftest`
  - `gate0_runtime_family_contract_selftest`
  - `gate0_runtime_family_contract_summary_selftest`

## Stable Bundle Contract

- bundle `checks_text`:
  - `lang_runtime_family,w95_cert,w96_somssi,w97_self_heal,gate0_runtime_family`
- progress schema:
  - `ddn.ci.gate0_runtime_family_contract_selftest.progress.v1`
- sanity steps:
  - `gate0_runtime_family_selftest`
  - `gate0_runtime_family_contract_selftest`
  - `gate0_runtime_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- progress schema:
  - `ddn.ci.gate0_runtime_family_transport_contract_selftest.progress.v1`
- raw field surface:
  - `age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks`
  - `age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks`
  - `age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text`
  - `age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe`
  - `age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe`
  - `age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present`
- compact token surface:
  - `age5_gate0_runtime_family_transport_contract_completed`
  - `age5_gate0_runtime_family_transport_contract_total`
  - `age5_gate0_runtime_family_transport_contract_checks_text`
  - `age5_gate0_runtime_family_transport_contract_current_probe`
  - `age5_gate0_runtime_family_transport_contract_last_completed_probe`
  - `age5_gate0_runtime_family_transport_contract_progress`
- sanity steps:
  - `gate0_runtime_family_transport_contract_selftest`
  - `gate0_runtime_family_transport_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
  - `age5_close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result/summary compact`
  - `ci_fail_brief/triage`
  - `ci_gate_report_index`

## Matrix

| surface line | summary | primary contract |
| --- | --- | --- |
| lang runtime line | `lang surface + stdlib catalog + tensor runtime` | `lang_runtime_family`가 상위 언어/runtime line을 닫는다 |
| W95 cert line | `sign/verify + tamper detect` | `run_w95_cert_pack_check.py`가 cert 최소 runtime contract를 닫는다 |
| W96 somssi line | `registry + sim adapter state_hash` | `run_w96_somssi_pack_check.py`가 somssi hub runtime contract를 닫는다 |
| W97 self-heal line | `checkpoint/rollback + heal_report determinism` | `run_w97_self_heal_pack_check.py`가 self-heal runtime contract를 닫는다 |

## Consumer Surface

- 상위 umbrella transport family:
  - `tests/gate0_surface_transport_family/README.md`
  - `python tests/run_gate0_surface_transport_family_selftest.py`
- 상위 umbrella family:
  - `tests/gate0_surface_family/README.md`
  - `python tests/run_gate0_surface_family_selftest.py`
- 상위 transport family:
  - `tests/gate0_transport_family/README.md`
  - `python tests/run_gate0_transport_family_selftest.py`
- 상위 family:
  - `tests/gate0_family/README.md`
  - `python tests/run_gate0_family_selftest.py`
- `tests/lang_runtime_family/README.md`
- `pack/gogae9_w95_cert/README.md`
- `pack/gogae9_w96_somssi_hub/README.md`
- `pack/gogae9_w97_self_heal/README.md`
- `python tests/run_lang_runtime_family_selftest.py`
- `python tests/run_w95_cert_pack_check.py`
- `python tests/run_w96_somssi_pack_check.py`
- `python tests/run_w97_self_heal_pack_check.py`
- `python tests/run_gate0_runtime_family_selftest.py`
- `python tests/run_gate0_runtime_family_contract_selftest.py`
- `python tests/run_gate0_runtime_family_contract_summary_selftest.py`
- `python tests/run_gate0_runtime_family_transport_contract_selftest.py`
- `python tests/run_gate0_runtime_family_transport_contract_summary_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_gate0_runtime_family_transport_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
