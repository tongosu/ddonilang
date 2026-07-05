# GANADA_REVERIFICATION_TIER1_V1

작성일: 2026-07-06
브랜치: `codex/queue-20260706`
대상: `ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md` §5 중 가/나/타 줄기 18칸
성격: 진단 전용. golden 갱신, 코드 수정, 삭제 없음.

## 실행 로그

- 체커 실행 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q23-ganada-tier1/`
- roadmap_v2 팩 golden 보조 실행 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q23-ganada-tier1/pack_golden/`
- 체커 요약: 28개 체커 실행 중 PASS 2개(`ga4`, `na4`), FAIL 26개.
- 보조 pack golden 요약: 존재하는 roadmap_v2 marker/reconciliation 팩 21개는 모두 `python tests/run_pack_golden.py <pack>` PASS. 다만 다수는 실제 기능 팩이 아니라 행렬 상태를 찍는 marker/reconciliation 팩이다.

## 판정 기준

- `증거없음`: 원문 완료근거에 대응하는 실행 가능한 pack/checker를 찾지 못함.
- `존재하나FAIL`: 대응 pack/checker는 있으나 실제 실행이 FAIL.
- `존재+PASS이나형식뿐`: 실행은 PASS하지만 내용이 marker/reconciliation 또는 제한된 shell이라 원문 마일스톤의 닫힘-동작을 충분히 증명하지 못함.
- `진짜닫힘`: 실행 PASS이고, 내용도 원문 마일스톤 범위와 실질적으로 부합.

## 18칸 재검증 표

| 줄기-마루 | 원문 완료근거 | 존재확인 | 실행결과 | 표본 내용확인 | 재판정 상태 |
|---|---|---|---|---|---|
| 가-0 | `SSOT_ROADMAP/OPEN_ISSUES` | `tests/run_roadmap_v2_ga0_current_line_ledger_matrix_reconciliation_check.py`, `pack/roadmap_v2_ga0_current_line_ledger_matrix_reconciliation_v1` | 체커 FAIL: `ROADMAP_V2_GA0_CURRENT_LINE_LEDGER_MATRIX_RECONCILIATION_V1.md` 누락. 보조 pack golden PASS. | 팩 README가 "Planning/checker pack", "matrix authority update"를 기록한다. 실행 기능 닫힘이 아니라 상태 marker 성격. | 존재하나FAIL |
| 가-1 | `core smoke pack` | `tests/run_roadmap_v2_ga1_core_smoke_matrix_reconciliation_check.py`, `pack/roadmap_v2_ga1_core_smoke_matrix_reconciliation_v1` | 체커 FAIL: `ROADMAP_V2_GA1_CORE_SMOKE_MATRIX_RECONCILIATION_V1.md` 누락. 보조 pack golden PASS. | 팩 README가 core smoke 자체가 아니라 matrix reconciliation 기록임을 밝힌다. | 존재하나FAIL |
| 가-2 | `golden/checker PASS` | `tests/run_roadmap_v2_ga2_*`, `pack/roadmap_v2_ga2_matrix_status_reconciliation_v1` | `ga2_matrix`, `ga2_rebase`, `ga2_final` 모두 FAIL. 누락 문서: `GA2_MATRIX_STATUS_RECONCILIATION_V1.md`, `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md`, `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md` 등. 보조 pack golden PASS. | 팩은 `lang_core_2_v1` 대표 문법 evidence를 행렬에 반영했다는 marker다. Q18은 bare 사용자 씨앗 호출이 `E_RUNTIME_UNDEFINED`임을 확인했고, Q20은 `E_CALL_TAIL_AMBIGUOUS` 미강제를 확인했다. 대표 문법 닫힘 claim과 충돌. | 존재하나FAIL |
| 가-3 | `LSP minimum pack` | `tests/run_roadmap_v2_ga3_editor_diagnostic_reconciliation_check.py`, `pack/roadmap_v2_ga3_editor_diagnostic_reconciliation_v1` | 체커 FAIL: 하위 `studio_classroom_report_workflow_runner.mjs`에서 `failed export row missing`. 보조 pack golden PASS. | 팩은 LSP/fix-it/diagnostic evidence를 행렬에 반영하는 reconciliation이며, README가 full LSP server/auto-apply 등을 명시적으로 주장하지 않는다. | 존재하나FAIL |
| 가-4 | `registry shell pack` | `tests/run_roadmap_v2_ga4_package_gaji_reconciliation_check.py`, `pack/roadmap_v2_ga4_package_gaji_reconciliation_v1` | 체커 PASS. 보조 pack golden PASS. | 팩/체커는 gaji registry provenance와 package registry surface를 묶지만, contract가 public registry, install/update/remove execution, trust signing, cloud sync를 모두 false로 둔다. Q21 실측도 local registry minimum은 부분 착지이고 `gaji/` 30개 전체 discover/install/publish 닫힘은 아니라고 결론. | 존재+PASS이나형식뿐 |
| 가-5 | `release gate` | `tests/run_roadmap_v2_ga5_*`, `pack/roadmap_v2_ga5_*` | `ga5_docs`, `ga5_behavior`, `ga5_release`, `ga5_blocker` 모두 FAIL. 누락/불일치: `GA5_RELEASE_GATE_BEHAVIOR_CLOSURE_V1.md`, `GA5_RELEASE_GATE_BLOCKER_AUDIT_V1.md`, matrix row의 `닫힘-문서` 기대 불일치. 보조 pack golden은 PASS. | `roadmap_v2_ga5_release_gate_behavior_closure_v1` README는 `90/90 = 100%` marker를 출력하지만 체커가 필요한 release gate 문서를 찾지 못한다. | 존재하나FAIL |
| 나-0 | `stdlib proposal` | `tests/run_roadmap_v2_na0_stdlib_candidate_list_check.py`, `pack/roadmap_v2_na0_stdlib_candidate_list_v1` | 체커 FAIL: matrix counts mismatch `rows=90 behavior=90 docs=0`. 보조 pack golden PASS. | README는 새 stdlib surface/parser/runtime/UI를 주장하지 않고 기존 catalog/evidence 연결만 기록한다. | 존재하나FAIL |
| 나-1 | `std_core, std_grid, std_input_map smoke` | `tests/run_roadmap_v2_na1_*`, `pack/roadmap_v2_na1_*` | `na1_matrix` FAIL: 하위 `na2_matrix` 누락 문서. `na1_post` FAIL: `seamgrim_editor_run_transaction` one-step control 초기화 오류. 보조 pack golden PASS. | 팩은 matrix reconciliation marker. 실제 std_grid/input_map 개별 팩은 존재하지만 이 칸의 통합 체커가 PASS하지 않는다. | 존재하나FAIL |
| 나-2 | `golden pack` | `tests/run_roadmap_v2_na2_*`, `pack/roadmap_v2_na2_matrix_status_reconciliation_v1` | `na2_matrix`, `na2_event`, `na2_unit_random` 모두 FAIL. 누락 문서/팩: `ROADMAP_V2_NA2_MATRIX_STATUS_RECONCILIATION_V1.md`, `STD_EVENT_MINIMUM_CLOSURE_V1.md`, `STD_RANDOM_BAG_MINIMUM_V1.md`, `STD_INPUT_MAP_CLOSURE_V1.md` 등. 보조 pack golden PASS. | 팩은 unit/random/event evidence를 행렬에 반영했다는 marker다. 실행 체커는 닫힘 증거를 재구성하지 못한다. | 존재하나FAIL |
| 나-3 | `social-world dependency pack` | `tests/run_roadmap_v2_na3_*`, `pack/roadmap_v2_na3_resource_network_policy_rebase_v1` | `na3_matrix`, `na3_resource` 모두 FAIL: `ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1.md`, `ROADMAP_V2_NA3_RESOURCE_NETWORK_POLICY_REBASE_V1.md` 누락. 보조 pack golden PASS. | `na3_resource_network_policy_rebase_v1` README가 "keeps 나-3 out of behavior-closed status"라고 명시한다. 원문 닫힘-동작과 정면 불일치. | 존재하나FAIL |
| 나-4 | `local registry pack` | `tests/run_roadmap_v2_na4_stdlib_registry_reconciliation_check.py`, `pack/roadmap_v2_na4_stdlib_registry_reconciliation_v1` | 체커 PASS. 보조 pack golden PASS. | contract는 public registry final, network sync, trust signing, cloud install/update/remove, new stdlib surface, runtime/UI/code change를 모두 false로 둔다. Q21의 local registry 실측과 같은 제한 범위. | 존재+PASS이나형식뿐 |
| 나-5 | `stdlib LTS suite` | `tests/run_roadmap_v2_na5_stdlib_lts_reconciliation_check.py`, `pack/roadmap_v2_na5_stdlib_lts_reconciliation_v1` | 체커 FAIL: 하위 `run_lang_history_alias_stdlib_bridge_check.py`가 `LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1.md` 누락. 보조 pack golden PASS. | LTS suite로 보기에는 하위 필수 문서가 없어 체커가 닫히지 않는다. | 존재하나FAIL |
| 타-0 | `skeleton docs` | `tests/run_roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_check.py`, `pack/roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_v1` | 체커 FAIL: `ROADMAP_V2_TA0_PACK_CHECKER_SKELETON_MATRIX_RECONCILIATION_V1.md` 누락. 보조 pack golden PASS. | 팩은 skeleton 동작 검증보다 matrix authority update marker 성격. | 존재하나FAIL |
| 타-1 | `runner PASS` | `tests/run_roadmap_v2_ta1_pack_runner_basis_matrix_reconciliation_check.py`, `pack/roadmap_v2_ta1_pack_runner_basis_matrix_reconciliation_v1` | 체커 FAIL: `ROADMAP_V2_TA1_PACK_RUNNER_BASIS_MATRIX_RECONCILIATION_V1.md` 누락. 보조 pack golden PASS. | 팩은 runner 자체 닫힘보다 matrix authority update marker 성격. | 존재하나FAIL |
| 타-2 | `CI PASS` | `tests/run_roadmap_v2_ta2_matrix_status_reconciliation_check.py`, `tests/run_roadmap_v2_ta2_guide_status_reconciliation_check.py`, `pack/roadmap_v2_ta2_*` | 두 체커 모두 FAIL: `TA2_MATRIX_STATUS_RECONCILIATION_V1.md`, `ROADMAP_V2_TA2_GUIDE_STATUS_RECONCILIATION_V1.md` 누락. 보조 pack golden PASS. | 팩은 `toolchain_pack_2_v1` evidence를 matrix에 반영했다는 marker이며, 현재 CI/golden gate 자체 PASS 증거로 재검증되지 않는다. | 존재하나FAIL |
| 타-3 | `LSP pack` | `tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py` | 체커 FAIL: `TA3_DIAGNOSTIC_UI_LSP_V1.md` 누락. 대응 roadmap_v2 pack은 확인하지 못함. | `toolchain_diagnostic_ui_lsp` 자산은 있으나 이 칸의 완료근거 체커가 요구하는 산출물이 없다. | 존재하나FAIL |
| 타-4 | `registry check pack` | `tests/run_roadmap_v2_ta4_registry_verification_check.py` | 체커 FAIL: `TA4_REGISTRY_VERIFICATION_V1.md` 누락. 대응 roadmap_v2 pack은 확인하지 못함. | Q21이 registry minimum 부분 착지를 확인했지만, 타-4 registry verification 칸의 checker는 닫히지 않는다. | 존재하나FAIL |
| 타-5 | `benchmark suite` | `tests/run_roadmap_v2_ta5_benchmark_lts_check.py`, `pack/benchmark_baseline_v1` 참고 가능 | 체커 FAIL: `TA5_BENCHMARK_LTS_V1.md` 누락. | `benchmark_baseline_v1`은 baseline pack이지 LTS suite 전체 증거는 아니다. 타-5 체커는 필수 산출물 누락으로 닫히지 않는다. | 존재하나FAIL |

## 교차 확인

- 가-2: Q18 실행 보고는 `stem_alias_dop_dou.ddn`이 `E_RUNTIME_UNDEFINED ... 정의되지 않은 경로: 살림.돕기`로 실패한다고 기록한다. Q20 실행 보고는 `() 계산하기.`가 `E_CALL_TAIL_AMBIGUOUS` 없이 `계산`으로 dispatch된다고 기록한다. 따라서 "채비/훅/조건/임자/계약 대표 pack 닫힘"을 현재 제품 동작 기준으로 진짜닫힘이라고 볼 수 없다.
- 가-4/나-4: Q21 `LOCAL_REGISTRY_LANDING_AUDIT_V1.md`는 로컬 레지스트리를 "부분 착지"로 판정한다. top-level `gaji/` 30개 중 current scanner 대상은 direct `gaji.toml` 11개뿐이고, strict registry install은 `trust_root.hash` 없이는 실패한다. 따라서 PASS한 reconciliation 체커는 local shell 수준의 제한된 증거로 보아야 한다.

## 결론

Q23 대상 18칸 중 `진짜닫힘`으로 재판정한 칸은 없다. 16칸은 대응 체커가 실제 실행에서 FAIL했고, 2칸(가-4, 나-4)은 PASS하지만 marker/reconciliation 및 local minimum 범위라 원문 행렬의 `닫힘-동작` 신뢰 근거로는 부족하다.
