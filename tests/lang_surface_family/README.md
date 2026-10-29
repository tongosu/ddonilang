# Lang Surface Family

## Stable Contract

- 목적:
  - 현재 gate에서 닫힌 핵심 언어 표면 5줄을 한 단계 위에서 함께 읽는 최상위 family contract를 고정한다.
  - 이 문서는 하위 line의 세부 transport를 다시 정의하지 않고, `proof`, `bogae alias`, `compound update reject`, `lang/teul-cli parser parity`, `dialect alias safety`가 같은 language surface를 이룬다는 점만 확인한다.
- 대상 surface:
  - `tests/proof_family/README.md`
  - `tests/bogae_alias_family/README.md`
  - `tests/compound_update_reject_contract/README.md`
  - `tests/lang_teulcli_parser_parity/README.md`
  - `tests/dialect_alias_collision_contract/README.md`
- selftest:
  - `python tests/run_proof_family_selftest.py`
  - `python tests/run_bogae_alias_family_selftest.py`
  - `python tests/run_compound_update_reject_contract_selftest.py`
  - `python tests/run_lang_teulcli_parser_parity_selftest.py`
  - `python tests/run_dialect_alias_collision_contract_selftest.py`
  - `python tests/run_lang_surface_family_selftest.py`
  - `python tests/run_lang_surface_family_contract_selftest.py`
  - `python tests/run_lang_surface_family_contract_summary_selftest.py`
  - `proof_family_selftest`
  - `bogae_alias_family_selftest`
  - `compound_update_reject_contract_selftest`
  - `lang_teulcli_parser_parity_selftest`
  - `dialect_alias_collision_contract_selftest`
  - `lang_surface_family_selftest`
  - `lang_surface_family_contract_selftest`

## Stable Bundle Contract

- bundle `checks_text`:
  - `proof_family,bogae_alias_family,compound_update_reject_contract,lang_teulcli_parser_parity,dialect_alias_collision_contract,lang_surface_family`
- progress schema:
  - `ddn.ci.lang_surface_family_contract_selftest.progress.v1`
- sanity steps:
  - `lang_surface_family_selftest`
  - `lang_surface_family_contract_selftest`
  - `lang_surface_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- progress schema:
  - `ddn.ci.lang_surface_family_transport_contract_selftest.progress.v1`
- raw field surface:
  - `age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks`
  - `age5_full_real_lang_surface_family_transport_contract_selftest_total_checks`
  - `age5_full_real_lang_surface_family_transport_contract_selftest_checks_text`
  - `age5_full_real_lang_surface_family_transport_contract_selftest_current_probe`
  - `age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe`
  - `age5_full_real_lang_surface_family_transport_contract_selftest_progress_present`
- compact token surface:
  - `age5_lang_surface_family_transport_contract_completed`
  - `age5_lang_surface_family_transport_contract_total`
  - `age5_lang_surface_family_transport_contract_checks_text`
  - `age5_lang_surface_family_transport_contract_current_probe`
  - `age5_lang_surface_family_transport_contract_last_completed_probe`
  - `age5_lang_surface_family_transport_contract_progress`
- sanity steps:
  - `lang_surface_family_transport_contract_selftest`
  - `lang_surface_family_transport_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout / json-out`
  - `age5_close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result / summary compact`
  - `ci_fail_brief / triage`
  - `ci_gate_report_index`

## Matrix

| surface line | summary | primary contract |
| --- | --- | --- |
| proof line | `proof operation family -> proof_certificate family -> proof_family` | 증명 연산과 증명서 surface가 상위 proof family contract로 닫혀 있다 |
| bogae alias line | `shape alias -> bogae alias family -> viewer/export alias consumer` | `모양/생김새`, canvas key, seamgrim canon alias가 보개 family contract로 닫혀 있다 |
| compound update reject line | `+<-/ -<- canonical`, `+=/-= reject` | AGE1 복합 갱신 문법의 채택/거부 경계가 reject contract로 닫혀 있다 |
| parser parity line | `tool canon <-> teul-cli canon` | 블록 헤더/보개마당/짜임 warning/매김 alias 4건의 shared parity를 같은 contract로 고정한다 |
| dialect alias safety line | `ko alias 1:1`, `known non-ko collision inventory`, `샘입력 != 입력` | dialect keyword/alias 표면에서 `ko`는 collision-free를 유지하고, 다른 말씨는 현재 collision inventory drift를 고정한다 |

## Consumer Surface

- 상위 umbrella transport family:
  - `tests/gate0_surface_transport_family/README.md`
  - `python tests/run_gate0_surface_transport_family_selftest.py`
- 상위 umbrella family:
  - `tests/gate0_surface_family/README.md`
  - `python tests/run_gate0_surface_family_selftest.py`
- `tests/lang_runtime_family/README.md`
- `tests/proof_family/README.md`
- `tests/bogae_alias_family/README.md`
- `tests/compound_update_reject_contract/README.md`
- `tests/lang_teulcli_parser_parity/README.md`
- `tests/dialect_alias_collision_contract/README.md`
- `python tests/run_lang_runtime_family_selftest.py`
- `python tests/run_proof_family_selftest.py`
- `python tests/run_bogae_alias_family_selftest.py`
- `python tests/run_compound_update_reject_contract_selftest.py`
- `python tests/run_lang_teulcli_parser_parity_selftest.py`
- `python tests/run_dialect_alias_collision_contract_selftest.py`
- `python tests/run_lang_surface_family_selftest.py`
- `python tests/run_lang_surface_family_contract_selftest.py`
- `python tests/run_lang_surface_family_contract_summary_selftest.py`
- `python tests/run_lang_surface_family_transport_contract_selftest.py`
- `python tests/run_lang_surface_family_transport_contract_summary_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_lang_surface_family_transport_selftest.py`
- `python tests/run_ci_aggregate_status_line_selftest.py`
- `python tests/run_ci_gate_final_status_line_selftest.py`
- `python tests/run_ci_gate_result_check_selftest.py`
- `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
- `python tests/run_ci_gate_summary_line_check_selftest.py`
- `python tests/run_ci_final_line_emitter_check.py`
- `python tests/run_ci_gate_report_index_check_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
