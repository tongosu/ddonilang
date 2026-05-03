# W107 Golden Index

## 목적
- `diag_contract_seulgi_hook`의 `W107` golden 묶음이 어떤 규칙을 고정하는지 빠르게 훑기 위한 인덱스다.
- 현재 active walk 집합은 `W107_C01`, `W107_G01`~`W107_G53` 중 `run_golden.py --walk 107`이 실제로 집계하는 케이스들이다.
- 기계 검증용 정본 인덱스는 `index.json`(`ddn.teul_cli.golden_index.v1`)을 사용한다.

## 읽는 법
- `patch_candidate_count: 1`은 AGE2 `슬기훅정책: 실행`이 draft patch 후보를 남기는 양수 증거다.
- `patch_candidate_count: 0`은 같은 표면에서 stale/overwrite/불일치 경계 때문에 patch 후보를 비우는 음수 증거다.
- `contract target`은 계약식이 bare target이 아니라 `*`, `*`, cross-root target을 직접 쓰는 경우를 뜻한다.

## 구간 요약

| 구간 | 대표 케이스 | 의미 |
| --- | --- | --- |
| `C01`, `G01`~`G04` | `W107_C01_contract_canon_alert`, `W107_G01_contract_alert` | 계약 정본화와 AGE1 alert/abort/post 기본 diag 표면 |
| `G05`~`G07` | `W107_G05_contract_execute_age2`, `W107_G07_contract_execute_age2_false_relational` | AGE2 실행 훅과 상수 거짓 계약식 patch 후보 |
| `G08`~`G19` | `W107_G08_contract_execute_age2_observed_relax`, `W107_G19_contract_execute_age2_observed_batang_nested` | 관측값 기반 patch 후보의 상수/부호/루트/다단 경로 양수 증거 |
| `G20`~`G31` | `W107_G20_contract_execute_age2_observed_mismatch_no_patch`, `W107_G31_contract_execute_age2_observed_batang_sibling_overwrite_kept` | 불일치/간접 대입/overwrite 경계와 계산식 관측값 승격 |
| `G32`~`G39` | `W107_G32_contract_execute_age2_contract_batang_target`, `W107_G39_contract_execute_age2_contract_cross_batang_to_sallim_nested` | rooted/cross-root 계약식 target 자체를 읽는 양수 증거 |
| `G40`~`G53` | `W107_G40_contract_execute_age2_contract_sallim_target_stale_blocked`, `W107_G53_contract_execute_age2_contract_cross_batang_to_sallim_nested_stale_blocked` | contract target 위의 stale, parent overwrite, sibling overwrite 안전 경계 |

## Contract Target Matrix

| 타깃 종류 | 양수 케이스 | stale blocked | parent overwrite blocked | sibling overwrite kept |
| --- | --- | --- | --- | --- |
| 기본 루트 leaf | `G34` | `G40` | 해당 없음 | 해당 없음 |
| 명시 루트 leaf | `G32` | `G41` | 해당 없음 | 해당 없음 |
| 기본 루트 nested | `G35` | `G42` | `G44` | `G46` |
| 명시 루트 nested | `G33` | `G43` | `G45` | `G47` |
| cross-root leaf | `G36`, `G37` | helper 경계로만 고정 | helper 경계로만 고정 | helper 경계로만 고정 |
| cross-root nested | `G38`, `G39` | `G52`, `G53` | `G48`, `G49` | `G50`, `G51` |

## Helper Matrix

| 규칙 | 증거 |
| --- | --- |
| typed lhs와 ``/`` prefix는 bare 대상으로 접힌다 | `run.rs` helper test, `G16`~`G19` |
| exact/ancestor만 영향 있는 최신 대입으로 본다 | helper test, `G28`, `G29` |
| descendant 경로는 영향 대상으로 보지 않는다 | helper test |
| bare leaf `x`는 source 루트 객체 재대입을 상위 경로로 보지 않는다 | helper test 2건 |

## Stable Contract

- bundle `checks_text`: `golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index`
- sanity steps: `w107_golden_index_selfcheck`, `w107_progress_contract_selftest`, `ci_gate_summary_line_check_selftest`
- upstream raw field: `age5_full_real_w107_progress_contract_selftest_*`
- compact token: `age5_w107_contract_checks_text`
- direct/consumer surface:
  - `ci_gate_summary_line`
  - aggregate preview summary
  - aggregate status line
  - final status line
  - `ci_gate_result`
  - `ci_fail_brief.txt`
  - `ci_fail_triage.detjson`
  - `ci_gate_report_index`

## 참고
- D-PACK 요약: [pack/diag_contract_seulgi_hook/README.md](/i:/OneDrive/study/ddonilang/codex/codex/pack/diag_contract_seulgi_hook/README.md)
- 수동 실행 메모: [pack/diag_contract_seulgi_hook/tests/README.md](/i:/OneDrive/study/ddonilang/codex/codex/pack/diag_contract_seulgi_hook/tests/README.md)
- 인덱스 self-check: `python tools/teul-cli/tests/run_w107_golden_index_selfcheck.py`
- transport bundle self-check: `python tests/run_w107_progress_contract_selftest.py`
- stable contract summary self-check: `python tests/run_w107_transport_contract_summary_selftest.py`
- `core_lang/full` sanity step: `w107_golden_index_selfcheck`
- `core_lang/full` sanity step: `w107_progress_contract_selftest`
- `core_lang/full` sanity step: `ci_gate_summary_line_check_selftest`
- `ci_sanity_gate --json-out` progress schema: `ddn.ci.w107_golden_index_selfcheck.progress.v1`
- `ci_sanity_gate --json-out` progress schema: `ddn.ci.w107_progress_contract_selftest.progress.v1`
- `ci_sanity_gate` stdout token: `w107_golden_index_selfcheck_active_cases`, `...inactive_cases`, `...index_codes`, `...last_completed_probe`
- `ci_sanity_gate` stdout token: `w107_progress_contract_selftest_completed_checks`, `...total_checks`, `...checks_text`, `...last_completed_probe`
- `age5_close` full-real report field: `age5_full_real_w107_progress_contract_selftest_*`
- aggregate preview summary token: `age5_full_real_w107_progress_contract_selftest_*`
- summary/aggregate/final/result/failure compact token: `age5_w107_contract_completed`, `age5_w107_contract_total`, `age5_w107_contract_checks_text`, `age5_w107_contract_last_completed_probe`, `age5_w107_contract_progress`
- result/failure/report-index field: `age5_full_real_w107_progress_contract_selftest_*`
